mod runtime {
    //! Environment definition of the wasm smart-contract runtime.
    //! wasm 智能合约运行时的环境定义。
    use crate::{
        exec::{ExecError, ExecResult, Ext, Key, TopicOf},
        gas::{ChargedAmount, Token},
        schedule::HostFnWeights,
        BalanceOf, CodeHash, Config, DebugBufferVec, Error, SENTINEL,
    };
    use bitflags::bitflags;
    use codec::{Decode, DecodeLimit, Encode, MaxEncodedLen};
    use frame_support::{
        dispatch::DispatchError, ensure, traits::Get, weights::Weight, RuntimeDebug,
    };
    use pallet_contracts_primitives::{ExecReturnValue, ReturnFlags};
    use pallet_contracts_proc_macro::define_env;
    use sp_io::hashing::{blake2_128, blake2_256, keccak_256, sha2_256};
    use sp_runtime::traits::{Bounded, Zero};
    use sp_std::{fmt, prelude::*};
    use wasmi::{core::HostError, errors::LinkerError, Linker, Memory, Store};
    /// The maximum nesting depth a contract can use when encoding types.
    const MAX_DECODE_NESTING: u32 = 256;
    /// Passed to [`Environment`] to determine whether it should expose deprecated interfaces.
    pub enum AllowDeprecatedInterface {
        /// No deprecated interfaces are exposed.
        No,
        /// Deprecated interfaces are exposed.
        Yes,
    }
    /// Passed to [`Environment`] to determine whether it should expose unstable interfaces.
    pub enum AllowUnstableInterface {
        /// No unstable interfaces are exposed.
        No,
        /// Unstable interfaces are exposed.
        Yes,
    }
    /// Trait implemented by the [`define_env`](pallet_contracts_proc_macro::define_env) macro for the
    /// emitted `Env` struct.
    pub trait Environment<HostState> {
        /// Adds all declared functions to the supplied [`Linker`](wasmi::Linker) and
        /// [`Store`](wasmi::Store).
        fn define(
            store: &mut Store<HostState>,
            linker: &mut Linker<HostState>,
            allow_unstable: AllowUnstableInterface,
            allow_deprecated: AllowDeprecatedInterface,
        ) -> Result<(), LinkerError>;
    }
    /// Type of a storage key.
    #[allow(dead_code)]
    enum KeyType {
        /// Legacy fix sized key `[u8;32]`.
        Fix,
        /// Variable sized key used in transparent hashing,
        /// cannot be larger than MaxStorageKeyLen.
        Var(u32),
    }
    /// Every error that can be returned to a contract when it calls any of the host functions.
    ///
    /// # Note
    ///
    /// This enum can be extended in the future: New codes can be added but existing codes
    /// will not be changed or removed. This means that any contract **must not** exhaustively
    /// match return codes. Instead, contracts should prepare for unknown variants and deal with
    /// those errors gracefully in order to be forward compatible.
    #[repr(u32)]
    pub enum ReturnCode {
        /// API call successful.
        Success = 0,
        /// The called function trapped and has its state changes reverted.
        /// In this case no output buffer is returned.
        CalleeTrapped = 1,
        /// The called function ran to completion but decided to revert its state.
        /// An output buffer is returned when one was supplied.
        CalleeReverted = 2,
        /// The passed key does not exist in storage.
        KeyNotFound = 3,
        /// See [`Error::TransferFailed`].
        TransferFailed = 5,
        /// No code could be found at the supplied code hash.
        CodeNotFound = 7,
        /// The contract that was called is no contract (a plain account).
        NotCallable = 8,
        /// The call dispatched by `seal_call_runtime` was executed but returned an error.
        CallRuntimeFailed = 10,
        /// ECDSA pubkey recovery failed (most probably wrong recovery id or signature), or
        /// ECDSA compressed pubkey conversion into Ethereum address failed (most probably
        /// wrong pubkey provided).
        EcdsaRecoverFailed = 11,
        /// sr25519 signature verification failed.
        Sr25519VerifyFailed = 12,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ReturnCode {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ReturnCode::Success => "Success",
                    ReturnCode::CalleeTrapped => "CalleeTrapped",
                    ReturnCode::CalleeReverted => "CalleeReverted",
                    ReturnCode::KeyNotFound => "KeyNotFound",
                    ReturnCode::TransferFailed => "TransferFailed",
                    ReturnCode::CodeNotFound => "CodeNotFound",
                    ReturnCode::NotCallable => "NotCallable",
                    ReturnCode::CallRuntimeFailed => "CallRuntimeFailed",
                    ReturnCode::EcdsaRecoverFailed => "EcdsaRecoverFailed",
                    ReturnCode::Sr25519VerifyFailed => "Sr25519VerifyFailed",
                },
            )
        }
    }
    impl From<ExecReturnValue> for ReturnCode {
        fn from(from: ExecReturnValue) -> Self {
            if from.flags.contains(ReturnFlags::REVERT) {
                Self::CalleeReverted
            } else {
                Self::Success
            }
        }
    }
    impl From<ReturnCode> for u32 {
        fn from(code: ReturnCode) -> u32 {
            code as u32
        }
    }
    /// The data passed through when a contract uses `seal_return`.
    pub struct ReturnData {
        /// The flags as passed through by the contract. They are still unchecked and
        /// will later be parsed into a `ReturnFlags` bitflags struct.
        flags: u32,
        /// The output buffer passed by the contract as return data.
        data: Vec<u8>,
    }
    impl core::fmt::Debug for ReturnData {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_struct("ReturnData")
                .field("flags", &self.flags)
                .field("data", &self.data)
                .finish()
        }
    }
    /// Enumerates all possible reasons why a trap was generated.
    ///
    /// This is either used to supply the caller with more information about why an error
    /// occurred (the SupervisorError variant).
    /// The other case is where the trap does not constitute an error but rather was invoked
    /// as a quick way to terminate the application (all other variants).
    pub enum TrapReason {
        /// The supervisor trapped the contract because of an error condition occurred during
        /// execution in privileged code.
        SupervisorError(DispatchError),
        /// Signals that trap was generated in response to call `seal_return` host function.
        Return(ReturnData),
        /// Signals that a trap was generated in response to a successful call to the
        /// `seal_terminate` host function.
        Termination,
    }
    impl core::fmt::Debug for TrapReason {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            match self {
                Self::SupervisorError(ref a0) => fmt
                    .debug_tuple("TrapReason::SupervisorError")
                    .field(a0)
                    .finish(),
                Self::Return(ref a0) => {
                    fmt.debug_tuple("TrapReason::Return").field(a0).finish()
                }
                Self::Termination => fmt.debug_tuple("TrapReason::Termination").finish(),
                _ => Ok(()),
            }
        }
    }
    impl<T: Into<DispatchError>> From<T> for TrapReason {
        fn from(from: T) -> Self {
            Self::SupervisorError(from.into())
        }
    }
    impl fmt::Display for TrapReason {
        fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
            Ok(())
        }
    }
    impl HostError for TrapReason {}
    pub enum RuntimeCosts {
        /// Charge the gas meter with the cost of a metering block. The charged costs are
        /// the supplied cost of the block plus the overhead of the metering itself.
        MeteringBlock(u64),
        /// Weight charged for copying data from the sandbox.
        CopyFromContract(u32),
        /// Weight charged for copying data to the sandbox.
        CopyToContract(u32),
        /// Weight of calling `seal_caller`.
        Caller,
        /// Weight of calling `seal_is_contract`.
        IsContract,
        /// Weight of calling `seal_code_hash`.
        CodeHash,
        /// Weight of calling `seal_own_code_hash`.
        OwnCodeHash,
        /// Weight of calling `seal_caller_is_origin`.
        CallerIsOrigin,
        /// Weight of calling `caller_is_root`.
        CallerIsRoot,
        /// Weight of calling `seal_address`.
        Address,
        /// Weight of calling `seal_gas_left`.
        GasLeft,
        /// Weight of calling `seal_balance`.
        Balance,
        /// Weight of calling `seal_value_transferred`.
        ValueTransferred,
        /// Weight of calling `seal_minimum_balance`.
        MinimumBalance,
        /// Weight of calling `seal_block_number`.
        BlockNumber,
        /// Weight of calling `seal_now`.
        Now,
        /// Weight of calling `seal_weight_to_fee`.
        WeightToFee,
        /// Weight of calling `seal_input` without the weight of copying the input.
        InputBase,
        /// Weight of calling `seal_return` for the given output size.
        Return(u32),
        /// Weight of calling `seal_terminate`.
        Terminate,
        /// Weight of calling `seal_random`. It includes the weight for copying the subject.
        Random,
        /// Weight of calling `seal_deposit_event` with the given number of topics and event size.
        DepositEvent { num_topic: u32, len: u32 },
        /// Weight of calling `seal_debug_message` per byte of passed message.
        DebugMessage(u32),
        /// Weight of calling `seal_set_storage` for the given storage item sizes.
        SetStorage { old_bytes: u32, new_bytes: u32 },
        /// Weight of calling `seal_clear_storage` per cleared byte.
        ClearStorage(u32),
        /// Weight of calling `seal_contains_storage` per byte of the checked item.
        ContainsStorage(u32),
        /// Weight of calling `seal_get_storage` with the specified size in storage.
        GetStorage(u32),
        /// Weight of calling `seal_take_storage` for the given size.
        TakeStorage(u32),
        /// Weight of calling `seal_transfer`.
        Transfer,
        /// Base weight of calling `seal_call`.
        CallBase,
        /// Weight of calling `seal_delegate_call` for the given input size.
        DelegateCallBase,
        /// Weight of the transfer performed during a call.
        CallSurchargeTransfer,
        /// Weight per byte that is cloned by supplying the `CLONE_INPUT` flag.
        CallInputCloned(u32),
        /// Weight of calling `seal_instantiate` for the given input length and salt.
        InstantiateBase { input_data_len: u32, salt_len: u32 },
        /// Weight of the transfer performed during an instantiate.
        InstantiateSurchargeTransfer,
        /// Weight of calling `seal_hash_sha_256` for the given input size.
        HashSha256(u32),
        /// Weight of calling `seal_hash_keccak_256` for the given input size.
        HashKeccak256(u32),
        /// Weight of calling `seal_hash_blake2_256` for the given input size.
        HashBlake256(u32),
        /// Weight of calling `seal_hash_blake2_128` for the given input size.
        HashBlake128(u32),
        /// Weight of calling `seal_ecdsa_recover`.
        EcdsaRecovery,
        /// Weight of calling `seal_sr25519_verify` for the given input size.
        Sr25519Verify(u32),
        /// Weight charged by a chain extension through `seal_call_chain_extension`.
        ChainExtension(Weight),
        /// Weight charged for calling into the runtime.
        CallRuntime(Weight),
        /// Weight of calling `seal_set_code_hash`
        SetCodeHash,
        /// Weight of calling `ecdsa_to_eth_address`
        EcdsaToEthAddress,
        /// Weight of calling `reentrance_count`
        ReentrantCount,
        /// Weight of calling `account_reentrance_count`
        AccountEntranceCount,
        /// Weight of calling `instantiation_nonce`
        InstantationNonce,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for RuntimeCosts {}
    #[automatically_derived]
    impl ::core::clone::Clone for RuntimeCosts {
        #[inline]
        fn clone(&self) -> RuntimeCosts {
            let _: ::core::clone::AssertParamIsClone<u64>;
            let _: ::core::clone::AssertParamIsClone<u32>;
            let _: ::core::clone::AssertParamIsClone<Weight>;
            *self
        }
    }
    impl RuntimeCosts {
        fn token<T: Config>(&self, s: &HostFnWeights<T>) -> RuntimeToken {
            use self::RuntimeCosts::*;
            let weight = match *self {
                MeteringBlock(amount) => s.gas.saturating_add(Weight::from_parts(amount, 0)),
                CopyFromContract(len) => s.return_per_byte.saturating_mul(len.into()),
                CopyToContract(len) => s.input_per_byte.saturating_mul(len.into()),
                Caller => s.caller,
                IsContract => s.is_contract,
                CodeHash => s.code_hash,
                OwnCodeHash => s.own_code_hash,
                CallerIsOrigin => s.caller_is_origin,
                CallerIsRoot => s.caller_is_root,
                Address => s.address,
                GasLeft => s.gas_left,
                Balance => s.balance,
                ValueTransferred => s.value_transferred,
                MinimumBalance => s.minimum_balance,
                BlockNumber => s.block_number,
                Now => s.now,
                WeightToFee => s.weight_to_fee,
                InputBase => s.input,
                Return(len) => s
                    .r#return
                    .saturating_add(s.return_per_byte.saturating_mul(len.into())),
                Terminate => s.terminate,
                Random => s.random,
                DepositEvent { num_topic, len } => s
                    .deposit_event
                    .saturating_add(s.deposit_event_per_topic.saturating_mul(num_topic.into()))
                    .saturating_add(s.deposit_event_per_byte.saturating_mul(len.into())),
                DebugMessage(len) => s
                    .debug_message
                    .saturating_add(s.deposit_event_per_byte.saturating_mul(len.into())),
                SetStorage {
                    new_bytes,
                    old_bytes,
                } => s
                    .set_storage
                    .saturating_add(s.set_storage_per_new_byte.saturating_mul(new_bytes.into()))
                    .saturating_add(
                        s.set_storage_per_old_byte.saturating_mul(old_bytes.into()),
                    ),
                ClearStorage(len) => s
                    .clear_storage
                    .saturating_add(s.clear_storage_per_byte.saturating_mul(len.into())),
                ContainsStorage(len) => s
                    .contains_storage
                    .saturating_add(s.contains_storage_per_byte.saturating_mul(len.into())),
                GetStorage(len) => s
                    .get_storage
                    .saturating_add(s.get_storage_per_byte.saturating_mul(len.into())),
                TakeStorage(len) => s
                    .take_storage
                    .saturating_add(s.take_storage_per_byte.saturating_mul(len.into())),
                Transfer => s.transfer,
                CallBase => s.call,
                DelegateCallBase => s.delegate_call,
                CallSurchargeTransfer => s.call_transfer_surcharge,
                CallInputCloned(len) => s.call_per_cloned_byte.saturating_mul(len.into()),
                InstantiateBase {
                    input_data_len,
                    salt_len,
                } => s
                    .instantiate
                    .saturating_add(
                        s.instantiate_per_input_byte
                            .saturating_mul(input_data_len.into()),
                    )
                    .saturating_add(
                        s.instantiate_per_salt_byte.saturating_mul(salt_len.into()),
                    ),
                InstantiateSurchargeTransfer => s.instantiate_transfer_surcharge,
                HashSha256(len) => s
                    .hash_sha2_256
                    .saturating_add(s.hash_sha2_256_per_byte.saturating_mul(len.into())),
                HashKeccak256(len) => s
                    .hash_keccak_256
                    .saturating_add(s.hash_keccak_256_per_byte.saturating_mul(len.into())),
                HashBlake256(len) => s
                    .hash_blake2_256
                    .saturating_add(s.hash_blake2_256_per_byte.saturating_mul(len.into())),
                HashBlake128(len) => s
                    .hash_blake2_128
                    .saturating_add(s.hash_blake2_128_per_byte.saturating_mul(len.into())),
                EcdsaRecovery => s.ecdsa_recover,
                Sr25519Verify(len) => s
                    .sr25519_verify
                    .saturating_add(s.sr25519_verify_per_byte.saturating_mul(len.into())),
                ChainExtension(weight) => weight,
                CallRuntime(weight) => weight,
                SetCodeHash => s.set_code_hash,
                EcdsaToEthAddress => s.ecdsa_to_eth_address,
                ReentrantCount => s.reentrance_count,
                AccountEntranceCount => s.account_reentrance_count,
                InstantationNonce => s.instantiation_nonce,
            };
            RuntimeToken { weight }
        }
    }
    struct RuntimeToken {
        weight: Weight,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for RuntimeToken {}
    #[automatically_derived]
    impl ::core::clone::Clone for RuntimeToken {
        #[inline]
        fn clone(&self) -> RuntimeToken {
            let _: ::core::clone::AssertParamIsClone<Weight>;
            *self
        }
    }
    impl<T: Config> Token<T> for RuntimeToken {
        fn weight(&self) -> Weight {
            self.weight
        }
    }
    /// Flags used to change the behaviour of `seal_call` and `seal_delegate_call`.
    pub struct CallFlags {
        bits: u32,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for CallFlags {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for CallFlags {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for CallFlags {
        #[inline]
        fn eq(&self, other: &CallFlags) -> bool {
            self.bits == other.bits
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for CallFlags {}
    #[automatically_derived]
    impl ::core::cmp::Eq for CallFlags {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for CallFlags {
        #[inline]
        fn clone(&self) -> CallFlags {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for CallFlags {
        #[inline]
        fn partial_cmp(
            &self,
            other: &CallFlags,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.bits, &other.bits)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for CallFlags {
        #[inline]
        fn cmp(&self, other: &CallFlags) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.bits, &other.bits)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for CallFlags {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.bits, state)
        }
    }
    impl ::bitflags::_core::fmt::Debug for CallFlags {
        fn fmt(
            &self,
            f: &mut ::bitflags::_core::fmt::Formatter,
        ) -> ::bitflags::_core::fmt::Result {
            #[allow(non_snake_case)]
            trait __BitFlags {
                #[inline]
                fn FORWARD_INPUT(&self) -> bool {
                    false
                }
                #[inline]
                fn CLONE_INPUT(&self) -> bool {
                    false
                }
                #[inline]
                fn TAIL_CALL(&self) -> bool {
                    false
                }
                #[inline]
                fn ALLOW_REENTRY(&self) -> bool {
                    false
                }
            }
            #[allow(non_snake_case)]
            impl __BitFlags for CallFlags {
                #[allow(deprecated)]
                #[inline]
                fn FORWARD_INPUT(&self) -> bool {
                    if Self::FORWARD_INPUT.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::FORWARD_INPUT.bits == Self::FORWARD_INPUT.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn CLONE_INPUT(&self) -> bool {
                    if Self::CLONE_INPUT.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::CLONE_INPUT.bits == Self::CLONE_INPUT.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn TAIL_CALL(&self) -> bool {
                    if Self::TAIL_CALL.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::TAIL_CALL.bits == Self::TAIL_CALL.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn ALLOW_REENTRY(&self) -> bool {
                    if Self::ALLOW_REENTRY.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::ALLOW_REENTRY.bits == Self::ALLOW_REENTRY.bits
                    }
                }
            }
            let mut first = true;
            if <Self as __BitFlags>::FORWARD_INPUT(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("FORWARD_INPUT")?;
            }
            if <Self as __BitFlags>::CLONE_INPUT(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("CLONE_INPUT")?;
            }
            if <Self as __BitFlags>::TAIL_CALL(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("TAIL_CALL")?;
            }
            if <Self as __BitFlags>::ALLOW_REENTRY(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("ALLOW_REENTRY")?;
            }
            let extra_bits = self.bits & !Self::all().bits();
            if extra_bits != 0 {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("0x")?;
                ::bitflags::_core::fmt::LowerHex::fmt(&extra_bits, f)?;
            }
            if first {
                f.write_str("(empty)")?;
            }
            Ok(())
        }
    }
    impl ::bitflags::_core::fmt::Binary for CallFlags {
        fn fmt(
            &self,
            f: &mut ::bitflags::_core::fmt::Formatter,
        ) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::Binary::fmt(&self.bits, f)
        }
    }
    impl ::bitflags::_core::fmt::Octal for CallFlags {
        fn fmt(
            &self,
            f: &mut ::bitflags::_core::fmt::Formatter,
        ) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::Octal::fmt(&self.bits, f)
        }
    }
    impl ::bitflags::_core::fmt::LowerHex for CallFlags {
        fn fmt(
            &self,
            f: &mut ::bitflags::_core::fmt::Formatter,
        ) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::LowerHex::fmt(&self.bits, f)
        }
    }
    impl ::bitflags::_core::fmt::UpperHex for CallFlags {
        fn fmt(
            &self,
            f: &mut ::bitflags::_core::fmt::Formatter,
        ) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::UpperHex::fmt(&self.bits, f)
        }
    }
    #[allow(dead_code)]
    impl CallFlags {
        /// Forward the input of current function to the callee.
        ///
        /// Supplied input pointers are ignored when set.
        ///
        /// # Note
        ///
        /// A forwarding call will consume the current contracts input. Any attempt to
        /// access the input after this call returns will lead to [`Error::InputForwarded`].
        /// It does not matter if this is due to calling `seal_input` or trying another
        /// forwarding call. Consider using [`Self::CLONE_INPUT`] in order to preserve
        /// the input.
        pub const FORWARD_INPUT: Self = Self { bits: 0b0000_0001 };
        /// Identical to [`Self::FORWARD_INPUT`] but without consuming the input.
        ///
        /// This adds some additional weight costs to the call.
        ///
        /// # Note
        ///
        /// This implies [`Self::FORWARD_INPUT`] and takes precedence when both are set.
        pub const CLONE_INPUT: Self = Self { bits: 0b0000_0010 };
        /// Do not return from the call but rather return the result of the callee to the
        /// callers caller.
        ///
        /// # Note
        ///
        /// This makes the current contract completely transparent to its caller by replacing
        /// this contracts potential output by the callee ones. Any code after `seal_call`
        /// can be safely considered unreachable.
        pub const TAIL_CALL: Self = Self { bits: 0b0000_0100 };
        /// Allow the callee to reenter into the current contract.
        ///
        /// Without this flag any reentrancy into the current contract that originates from
        /// the callee (or any of its callees) is denied. This includes the first callee:
        /// You cannot call into yourself with this flag set.
        ///
        /// # Note
        ///
        /// For `seal_delegate_call` should be always unset, otherwise
        /// [`Error::InvalidCallFlags`] is returned.
        pub const ALLOW_REENTRY: Self = Self { bits: 0b0000_1000 };
        /// Returns an empty set of flags.
        #[inline]
        pub const fn empty() -> Self {
            Self { bits: 0 }
        }
        /// Returns the set containing all flags.
        #[inline]
        pub const fn all() -> Self {
            #[allow(non_snake_case)]
            trait __BitFlags {
                const FORWARD_INPUT: u32 = 0;
                const CLONE_INPUT: u32 = 0;
                const TAIL_CALL: u32 = 0;
                const ALLOW_REENTRY: u32 = 0;
            }
            #[allow(non_snake_case)]
            impl __BitFlags for CallFlags {
                #[allow(deprecated)]
                const FORWARD_INPUT: u32 = Self::FORWARD_INPUT.bits;
                #[allow(deprecated)]
                const CLONE_INPUT: u32 = Self::CLONE_INPUT.bits;
                #[allow(deprecated)]
                const TAIL_CALL: u32 = Self::TAIL_CALL.bits;
                #[allow(deprecated)]
                const ALLOW_REENTRY: u32 = Self::ALLOW_REENTRY.bits;
            }
            Self {
                bits: <Self as __BitFlags>::FORWARD_INPUT
                    | <Self as __BitFlags>::CLONE_INPUT
                    | <Self as __BitFlags>::TAIL_CALL
                    | <Self as __BitFlags>::ALLOW_REENTRY,
            }
        }
        /// Returns the raw value of the flags currently stored.
        #[inline]
        pub const fn bits(&self) -> u32 {
            self.bits
        }
        /// Convert from underlying bit representation, unless that
        /// representation contains bits that do not correspond to a flag.
        #[inline]
        pub const fn from_bits(bits: u32) -> ::bitflags::_core::option::Option<Self> {
            if (bits & !Self::all().bits()) == 0 {
                ::bitflags::_core::option::Option::Some(Self { bits })
            } else {
                ::bitflags::_core::option::Option::None
            }
        }
        /// Convert from underlying bit representation, dropping any bits
        /// that do not correspond to flags.
        #[inline]
        pub const fn from_bits_truncate(bits: u32) -> Self {
            Self {
                bits: bits & Self::all().bits,
            }
        }
        /// Convert from underlying bit representation, preserving all
        /// bits (even those not corresponding to a defined flag).
        ///
        /// # Safety
        ///
        /// The caller of the `bitflags!` macro can chose to allow or
        /// disallow extra bits for their bitflags type.
        ///
        /// The caller of `from_bits_unchecked()` has to ensure that
        /// all bits correspond to a defined flag or that extra bits
        /// are valid for this bitflags type.
        #[inline]
        pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
            Self { bits }
        }
        /// Returns `true` if no flags are currently stored.
        #[inline]
        pub const fn is_empty(&self) -> bool {
            self.bits() == Self::empty().bits()
        }
        /// Returns `true` if all flags are currently set.
        #[inline]
        pub const fn is_all(&self) -> bool {
            Self::all().bits | self.bits == self.bits
        }
        /// Returns `true` if there are flags common to both `self` and `other`.
        #[inline]
        pub const fn intersects(&self, other: Self) -> bool {
            !(Self {
                bits: self.bits & other.bits,
            })
                .is_empty()
        }
        /// Returns `true` if all of the flags in `other` are contained within `self`.
        #[inline]
        pub const fn contains(&self, other: Self) -> bool {
            (self.bits & other.bits) == other.bits
        }
        /// Inserts the specified flags in-place.
        #[inline]
        pub fn insert(&mut self, other: Self) {
            self.bits |= other.bits;
        }
        /// Removes the specified flags in-place.
        #[inline]
        pub fn remove(&mut self, other: Self) {
            self.bits &= !other.bits;
        }
        /// Toggles the specified flags in-place.
        #[inline]
        pub fn toggle(&mut self, other: Self) {
            self.bits ^= other.bits;
        }
        /// Inserts or removes the specified flags depending on the passed value.
        #[inline]
        pub fn set(&mut self, other: Self, value: bool) {
            if value {
                self.insert(other);
            } else {
                self.remove(other);
            }
        }
        /// Returns the intersection between the flags in `self` and
        /// `other`.
        ///
        /// Specifically, the returned set contains only the flags which are
        /// present in *both* `self` *and* `other`.
        ///
        /// This is equivalent to using the `&` operator (e.g.
        /// [`ops::BitAnd`]), as in `flags & other`.
        ///
        /// [`ops::BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
        #[inline]
        #[must_use]
        pub const fn intersection(self, other: Self) -> Self {
            Self {
                bits: self.bits & other.bits,
            }
        }
        /// Returns the union of between the flags in `self` and `other`.
        ///
        /// Specifically, the returned set contains all flags which are
        /// present in *either* `self` *or* `other`, including any which are
        /// present in both (see [`Self::symmetric_difference`] if that
        /// is undesirable).
        ///
        /// This is equivalent to using the `|` operator (e.g.
        /// [`ops::BitOr`]), as in `flags | other`.
        ///
        /// [`ops::BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
        #[inline]
        #[must_use]
        pub const fn union(self, other: Self) -> Self {
            Self {
                bits: self.bits | other.bits,
            }
        }
        /// Returns the difference between the flags in `self` and `other`.
        ///
        /// Specifically, the returned set contains all flags present in
        /// `self`, except for the ones present in `other`.
        ///
        /// It is also conceptually equivalent to the "bit-clear" operation:
        /// `flags & !other` (and this syntax is also supported).
        ///
        /// This is equivalent to using the `-` operator (e.g.
        /// [`ops::Sub`]), as in `flags - other`.
        ///
        /// [`ops::Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
        #[inline]
        #[must_use]
        pub const fn difference(self, other: Self) -> Self {
            Self {
                bits: self.bits & !other.bits,
            }
        }
        /// Returns the [symmetric difference][sym-diff] between the flags
        /// in `self` and `other`.
        ///
        /// Specifically, the returned set contains the flags present which
        /// are present in `self` or `other`, but that are not present in
        /// both. Equivalently, it contains the flags present in *exactly
        /// one* of the sets `self` and `other`.
        ///
        /// This is equivalent to using the `^` operator (e.g.
        /// [`ops::BitXor`]), as in `flags ^ other`.
        ///
        /// [sym-diff]: https://en.wikipedia.org/wiki/Symmetric_difference
        /// [`ops::BitXor`]: https://doc.rust-lang.org/std/ops/trait.BitXor.html
        #[inline]
        #[must_use]
        pub const fn symmetric_difference(self, other: Self) -> Self {
            Self {
                bits: self.bits ^ other.bits,
            }
        }
        /// Returns the complement of this set of flags.
        ///
        /// Specifically, the returned set contains all the flags which are
        /// not set in `self`, but which are allowed for this type.
        ///
        /// Alternatively, it can be thought of as the set difference
        /// between [`Self::all()`] and `self` (e.g. `Self::all() - self`)
        ///
        /// This is equivalent to using the `!` operator (e.g.
        /// [`ops::Not`]), as in `!flags`.
        ///
        /// [`Self::all()`]: Self::all
        /// [`ops::Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
        #[inline]
        #[must_use]
        pub const fn complement(self) -> Self {
            Self::from_bits_truncate(!self.bits)
        }
    }
    impl ::bitflags::_core::ops::BitOr for CallFlags {
        type Output = Self;
        /// Returns the union of the two sets of flags.
        #[inline]
        fn bitor(self, other: CallFlags) -> Self {
            Self {
                bits: self.bits | other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::BitOrAssign for CallFlags {
        /// Adds the set of flags.
        #[inline]
        fn bitor_assign(&mut self, other: Self) {
            self.bits |= other.bits;
        }
    }
    impl ::bitflags::_core::ops::BitXor for CallFlags {
        type Output = Self;
        /// Returns the left flags, but with all the right flags toggled.
        #[inline]
        fn bitxor(self, other: Self) -> Self {
            Self {
                bits: self.bits ^ other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::BitXorAssign for CallFlags {
        /// Toggles the set of flags.
        #[inline]
        fn bitxor_assign(&mut self, other: Self) {
            self.bits ^= other.bits;
        }
    }
    impl ::bitflags::_core::ops::BitAnd for CallFlags {
        type Output = Self;
        /// Returns the intersection between the two sets of flags.
        #[inline]
        fn bitand(self, other: Self) -> Self {
            Self {
                bits: self.bits & other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::BitAndAssign for CallFlags {
        /// Disables all flags disabled in the set.
        #[inline]
        fn bitand_assign(&mut self, other: Self) {
            self.bits &= other.bits;
        }
    }
    impl ::bitflags::_core::ops::Sub for CallFlags {
        type Output = Self;
        /// Returns the set difference of the two sets of flags.
        #[inline]
        fn sub(self, other: Self) -> Self {
            Self {
                bits: self.bits & !other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::SubAssign for CallFlags {
        /// Disables all flags enabled in the set.
        #[inline]
        fn sub_assign(&mut self, other: Self) {
            self.bits &= !other.bits;
        }
    }
    impl ::bitflags::_core::ops::Not for CallFlags {
        type Output = Self;
        /// Returns the complement of this set of flags.
        #[inline]
        fn not(self) -> Self {
            Self { bits: !self.bits } & Self::all()
        }
    }
    impl ::bitflags::_core::iter::Extend<CallFlags> for CallFlags {
        fn extend<T: ::bitflags::_core::iter::IntoIterator<Item = Self>>(
            &mut self,
            iterator: T,
        ) {
            for item in iterator {
                self.insert(item)
            }
        }
    }
    impl ::bitflags::_core::iter::FromIterator<CallFlags> for CallFlags {
        fn from_iter<T: ::bitflags::_core::iter::IntoIterator<Item = Self>>(
            iterator: T,
        ) -> Self {
            let mut result = Self::empty();
            result.extend(iterator);
            result
        }
    }
    /// The kind of call that should be performed.
    enum CallType {
        /// Execute another instantiated contract
        Call {
            callee_ptr: u32,
            value_ptr: u32,
            deposit_ptr: u32,
            weight: Weight,
        },
        /// Execute deployed code in the context (storage, account ID, value) of the caller contract
        DelegateCall { code_hash_ptr: u32 },
    }
    impl CallType {
        fn cost(&self) -> RuntimeCosts {
            match self {
                CallType::Call { .. } => RuntimeCosts::CallBase,
                CallType::DelegateCall { .. } => RuntimeCosts::DelegateCallBase,
            }
        }
    }
    /// This is only appropriate when writing out data of constant size that does not depend on user
    /// input. In this case the costs for this copy was already charged as part of the token at
    /// the beginning of the API entry point.
    fn already_charged(_: u32) -> Option<RuntimeCosts> {
        None
    }
    /// Can only be used for one call.
    /// 只能用于依次调用
    pub struct Runtime<'a, E: Ext + 'a> {
        ext: &'a mut E,
        input_data: Option<Vec<u8>>,
        memory: Option<Memory>,
        chain_extension: Option<Box<<E::T as Config>::ChainExtension>>,
    }
    impl<'a, E: Ext + 'a> Runtime<'a, E> {
        pub fn new(ext: &'a mut E, input_data: Vec<u8>) -> Self {
            Runtime {
                ext,
                input_data: Some(input_data),
                memory: None,
                chain_extension: Some(Box::new(Default::default())),
            }
        }
        pub fn memory(&self) -> Option<Memory> {
            self.memory
        }
        pub fn set_memory(&mut self, memory: Memory) {
            self.memory = Some(memory);
        }
        /// Converts the sandbox result and the runtime state into the execution outcome.
        pub fn to_execution_result(
            self,
            sandbox_result: Result<(), wasmi::Error>,
        ) -> ExecResult {
            use TrapReason::*;
            match sandbox_result {
                Ok(_) => Ok(ExecReturnValue {
                    flags: ReturnFlags::empty(),
                    data: Vec::new(),
                }),
                Err(wasmi::Error::Trap(trap)) => {
                    let reason: TrapReason =
                        trap.downcast().ok_or(Error::<E::T>::ContractTrapped)?;
                    match reason {
                        Return(ReturnData { flags, data }) => {
                            let flags = ReturnFlags::from_bits(flags)
                                .ok_or(Error::<E::T>::InvalidCallFlags)?;
                            Ok(ExecReturnValue { flags, data })
                        }
                        Termination => Ok(ExecReturnValue {
                            flags: ReturnFlags::empty(),
                            data: Vec::new(),
                        }),
                        SupervisorError(error) => return Err(error.into()),
                    }
                }
                Err(_) => Err(Error::<E::T>::CodeRejected.into()),
            }
        }
        /// Get a mutable reference to the inner `Ext`.
        ///
        /// This is mainly for the chain extension to have access to the environment the
        /// contract is executing in.
        pub fn ext(&mut self) -> &mut E {
            self.ext
        }
        /// Charge the gas meter with the specified token.
        ///
        /// Returns `Err(HostError)` if there is not enough gas.
        pub fn charge_gas(
            &mut self,
            costs: RuntimeCosts,
        ) -> Result<ChargedAmount, DispatchError> {
            {
                let token = costs.token(&self.ext.schedule().host_fn_weights);
                self.ext.gas_meter().charge(token)
            }
        }
        /// Adjust a previously charged amount down to its actual amount.
        ///
        /// This is when a maximum a priori amount was charged and then should be partially
        /// refunded to match the actual amount.
        pub fn adjust_gas(&mut self, charged: ChargedAmount, actual_costs: RuntimeCosts) {
            let token = actual_costs.token(&self.ext.schedule().host_fn_weights);
            self.ext.gas_meter().adjust_gas(charged, token);
        }
        /// Read designated chunk from the sandbox memory.
        ///
        /// Returns `Err` if one of the following conditions occurs:
        ///
        /// - requested buffer is not within the bounds of the sandbox memory.
        /// 从沙盒内存中读取指定的区块。
        /// 如果出现下列情况之一，则返回 Err ：
        /// 请求的缓冲区不在沙盒内存的范围内
        pub fn read_sandbox_memory(
            &self,
            memory: &[u8],
            ptr: u32,
            len: u32,
        ) -> Result<Vec<u8>, DispatchError> {
            {
                if !(len <= self.ext.schedule().limits.max_memory_size()) {
                    {
                        return Err(Error::<E::T>::OutOfBounds.into());
                    };
                }
            };
            let mut buf = ::alloc::vec::from_elem(0u8, len as usize);
            self.read_sandbox_memory_into_buf(memory, ptr, buf.as_mut_slice())?;
            Ok(buf)
        }
        /// Read designated chunk from the sandbox memory into the supplied buffer.
        ///
        /// Returns `Err` if one of the following conditions occurs:
        ///
        /// - requested buffer is not within the bounds of the sandbox memory.
        pub fn read_sandbox_memory_into_buf(
            &self,
            memory: &[u8],
            ptr: u32,
            buf: &mut [u8],
        ) -> Result<(), DispatchError> {
            let ptr = ptr as usize;
            let bound_checked = memory
                .get(ptr..ptr + buf.len())
                .ok_or_else(|| Error::<E::T>::OutOfBounds)?;
            buf.copy_from_slice(bound_checked);
            Ok(())
        }
        /// Reads and decodes a type with a size fixed at compile time from contract memory.
        ///
        /// # Note
        ///
        /// The weight of reading a fixed value is included in the overall weight of any
        /// contract callable function.
        pub fn read_sandbox_memory_as<D: Decode + MaxEncodedLen>(
            &self,
            memory: &[u8],
            ptr: u32,
        ) -> Result<D, DispatchError> {
            let ptr = ptr as usize;
            let mut bound_checked = memory
                .get(ptr..ptr + D::max_encoded_len() as usize)
                .ok_or_else(|| Error::<E::T>::OutOfBounds)?;
            let decoded =
                D::decode_all_with_depth_limit(MAX_DECODE_NESTING, &mut bound_checked)
                    .map_err(|_| DispatchError::from(Error::<E::T>::DecodingFailed))?;
            Ok(decoded)
        }
        /// Read designated chunk from the sandbox memory and attempt to decode into the specified type.
        ///
        /// Returns `Err` if one of the following conditions occurs:
        ///
        /// - requested buffer is not within the bounds of the sandbox memory.
        /// - the buffer contents cannot be decoded as the required type.
        ///
        /// # Note
        ///
        /// There must be an extra benchmark for determining the influence of `len` with
        /// regard to the overall weight.
        pub fn read_sandbox_memory_as_unbounded<D: Decode>(
            &self,
            memory: &[u8],
            ptr: u32,
            len: u32,
        ) -> Result<D, DispatchError> {
            let ptr = ptr as usize;
            let mut bound_checked = memory
                .get(ptr..ptr + len as usize)
                .ok_or_else(|| Error::<E::T>::OutOfBounds)?;
            let decoded =
                D::decode_all_with_depth_limit(MAX_DECODE_NESTING, &mut bound_checked)
                    .map_err(|_| DispatchError::from(Error::<E::T>::DecodingFailed))?;
            Ok(decoded)
        }
        /// Write the given buffer and its length to the designated locations in sandbox memory and
        /// charge gas according to the token returned by `create_token`.
        /// `out_ptr` is the location in sandbox memory where `buf` should be written to.
        /// `out_len_ptr` is an in-out location in sandbox memory. It is read to determine the
        /// length of the buffer located at `out_ptr`. If that buffer is large enough the actual
        /// `buf.len()` is written to this location.
        ///
        /// If `out_ptr` is set to the sentinel value of `SENTINEL` and `allow_skip` is true the
        /// operation is skipped and `Ok` is returned. This is supposed to help callers to make copying
        /// output optional. For example to skip copying back the output buffer of an `seal_call`
        /// when the caller is not interested in the result.
        ///
        /// `create_token` can optionally instruct this function to charge the gas meter with the token
        /// it returns. `create_token` receives the variable amount of bytes that are about to be copied
        /// by this function.
        ///
        /// In addition to the error conditions of `write_sandbox_memory` this functions returns
        /// `Err` if the size of the buffer located at `out_ptr` is too small to fit `buf`.
        pub fn write_sandbox_output(
            &mut self,
            memory: &mut [u8],
            out_ptr: u32,
            out_len_ptr: u32,
            buf: &[u8],
            allow_skip: bool,
            create_token: impl FnOnce(u32) -> Option<RuntimeCosts>,
        ) -> Result<(), DispatchError> {
            if allow_skip && out_ptr == SENTINEL {
                return Ok(());
            }
            let buf_len = buf.len() as u32;
            let len: u32 = self.read_sandbox_memory_as(memory, out_len_ptr)?;
            if len < buf_len {
                return Err(Error::<E::T>::OutputBufferTooSmall.into());
            }
            if let Some(costs) = create_token(buf_len) {
                self.charge_gas(costs)?;
            }
            self.write_sandbox_memory(memory, out_ptr, buf)?;
            self.write_sandbox_memory(memory, out_len_ptr, &buf_len.encode())
        }
        /// Write the given buffer to the designated location in the sandbox memory.
        ///
        /// Returns `Err` if one of the following conditions occurs:
        ///
        /// - designated area is not within the bounds of the sandbox memory.
        fn write_sandbox_memory(
            &self,
            memory: &mut [u8],
            ptr: u32,
            buf: &[u8],
        ) -> Result<(), DispatchError> {
            let ptr = ptr as usize;
            let bound_checked = memory
                .get_mut(ptr..ptr + buf.len())
                .ok_or_else(|| Error::<E::T>::OutOfBounds)?;
            bound_checked.copy_from_slice(buf);
            Ok(())
        }
        /// Computes the given hash function on the supplied input.
        ///
        /// Reads from the sandboxed input buffer into an intermediate buffer.
        /// Returns the result directly to the output buffer of the sandboxed memory.
        ///
        /// It is the callers responsibility to provide an output buffer that
        /// is large enough to hold the expected amount of bytes returned by the
        /// chosen hash function.
        ///
        /// # Note
        ///
        /// The `input` and `output` buffers may overlap.
        fn compute_hash_on_intermediate_buffer<F, R>(
            &self,
            memory: &mut [u8],
            hash_fn: F,
            input_ptr: u32,
            input_len: u32,
            output_ptr: u32,
        ) -> Result<(), DispatchError>
            where
                F: FnOnce(&[u8]) -> R,
                R: AsRef<[u8]>,
        {
            let input = self.read_sandbox_memory(memory, input_ptr, input_len)?;
            let hash = hash_fn(&input);
            self.write_sandbox_memory(memory, output_ptr, hash.as_ref())?;
            Ok(())
        }
        /// Fallible conversion of `DispatchError` to `ReturnCode`.
        fn err_into_return_code(from: DispatchError) -> Result<ReturnCode, DispatchError> {
            use ReturnCode::*;
            let transfer_failed = Error::<E::T>::TransferFailed.into();
            let no_code = Error::<E::T>::CodeNotFound.into();
            let not_found = Error::<E::T>::ContractNotFound.into();
            match from {
                x if x == transfer_failed => Ok(TransferFailed),
                x if x == no_code => Ok(CodeNotFound),
                x if x == not_found => Ok(NotCallable),
                err => Err(err),
            }
        }
        /// Fallible conversion of a `ExecResult` to `ReturnCode`.
        fn exec_into_return_code(from: ExecResult) -> Result<ReturnCode, DispatchError> {
            use crate::exec::ErrorOrigin::Callee;
            let ExecError { error, origin } = match from {
                Ok(retval) => return Ok(retval.into()),
                Err(err) => err,
            };
            match (error, origin) {
                (_, Callee) => Ok(ReturnCode::CalleeTrapped),
                (err, _) => Self::err_into_return_code(err),
            }
        }
        fn decode_key(
            &self,
            memory: &[u8],
            key_type: KeyType,
            key_ptr: u32,
        ) -> Result<crate::exec::Key<E::T>, TrapReason> {
            let res = match key_type {
                KeyType::Fix => {
                    let key = self.read_sandbox_memory(memory, key_ptr, 32u32)?;
                    Key::try_from_fix(key)
                }
                KeyType::Var(len) => {
                    {
                        if !(len <= <<E as Ext>::T as Config>::MaxStorageKeyLen::get()) {
                            {
                                return Err(Error::<E::T>::DecodingFailed.into());
                            };
                        }
                    };
                    let key = self.read_sandbox_memory(memory, key_ptr, len)?;
                    Key::try_from_var(key)
                }
            };
            res.map_err(|_| Error::<E::T>::DecodingFailed.into())
        }
        fn set_storage(
            &mut self,
            memory: &[u8],
            key_type: KeyType,
            key_ptr: u32,
            value_ptr: u32,
            value_len: u32,
        ) -> Result<u32, TrapReason> {
            let max_size = self.ext.max_value_size();
            let charged = self.charge_gas(RuntimeCosts::SetStorage {
                new_bytes: value_len,
                old_bytes: max_size,
            })?;
            if value_len > max_size {
                return Err(Error::<E::T>::ValueTooLarge.into());
            }
            let key = self.decode_key(memory, key_type, key_ptr)?;
            let value = Some(self.read_sandbox_memory(memory, value_ptr, value_len)?);
            let write_outcome = self.ext.set_storage(&key, value, false)?;
            self.adjust_gas(
                charged,
                RuntimeCosts::SetStorage {
                    new_bytes: value_len,
                    old_bytes: write_outcome.old_len(),
                },
            );
            Ok(write_outcome.old_len_with_sentinel())
        }
        fn clear_storage(
            &mut self,
            memory: &[u8],
            key_type: KeyType,
            key_ptr: u32,
        ) -> Result<u32, TrapReason> {
            let charged =
                self.charge_gas(RuntimeCosts::ClearStorage(self.ext.max_value_size()))?;
            let key = self.decode_key(memory, key_type, key_ptr)?;
            let outcome = self.ext.set_storage(&key, None, false)?;
            self.adjust_gas(charged, RuntimeCosts::ClearStorage(outcome.old_len()));
            Ok(outcome.old_len_with_sentinel())
        }
        fn get_storage(
            &mut self,
            memory: &mut [u8],
            key_type: KeyType,
            key_ptr: u32,
            out_ptr: u32,
            out_len_ptr: u32,
        ) -> Result<ReturnCode, TrapReason> {
            let charged =
                self.charge_gas(RuntimeCosts::GetStorage(self.ext.max_value_size()))?;
            let key = self.decode_key(memory, key_type, key_ptr)?;
            let outcome = self.ext.get_storage(&key);
            if let Some(value) = outcome {
                self.adjust_gas(charged, RuntimeCosts::GetStorage(value.len() as u32));
                self.write_sandbox_output(
                    memory,
                    out_ptr,
                    out_len_ptr,
                    &value,
                    false,
                    already_charged,
                )?;
                Ok(ReturnCode::Success)
            } else {
                self.adjust_gas(charged, RuntimeCosts::GetStorage(0));
                Ok(ReturnCode::KeyNotFound)
            }
        }
        fn contains_storage(
            &mut self,
            memory: &[u8],
            key_type: KeyType,
            key_ptr: u32,
        ) -> Result<u32, TrapReason> {
            let charged =
            // 选中项目的每个字节的调用 seal_contains_storage 权重。
            // 使用指定的令牌为燃气表充电。
                // 如果没有足够的气体，则返回 Err(HostError)
                self.charge_gas(RuntimeCosts::ContainsStorage(self.ext.max_value_size()))?;
            // 从memory中decode出key
            let key = self.decode_key(memory, key_type, key_ptr)?;
            // 如果存储项存在于 处key，则返回Some(len)（以字节为单位）。
            // 如果以前未设置set_storage或删除，key则返回None
            //
            //        /// 返回值的长度（以字节为单位），而不读取它。 None 如果不存在。
            //         pub fn size(&self, key: &Key<T>) -> Option<u32> {
            //             child::len(&self.child_trie_info(), key.hash().as_slice())
            //         }
            let outcome = self.ext.get_storage_size(&key);
            self.adjust_gas(charged, RuntimeCosts::ClearStorage(outcome.unwrap_or(0)));
            Ok(outcome.unwrap_or(SENTINEL))
        }
        fn call(
            &mut self,
            memory: &mut [u8],
            flags: CallFlags,
            call_type: CallType,
            input_data_ptr: u32,
            input_data_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> Result<ReturnCode, TrapReason> {
            self.charge_gas(call_type.cost())?;
            let input_data = if flags.contains(CallFlags::CLONE_INPUT) {
                let input = self
                    .input_data
                    .as_ref()
                    .ok_or(Error::<E::T>::InputForwarded)?;
                {
                    let token = RuntimeCosts::CallInputCloned(input.len() as u32)
                        .token(&self.ext.schedule().host_fn_weights);
                    self.ext.gas_meter().charge(token)
                }?;
                input.clone()
            } else if flags.contains(CallFlags::FORWARD_INPUT) {
                self.input_data
                    .take()
                    .ok_or(Error::<E::T>::InputForwarded)?
            } else {
                self.charge_gas(RuntimeCosts::CopyFromContract(input_data_len))?;
                self.read_sandbox_memory(memory, input_data_ptr, input_data_len)?
            };
            let call_outcome = match call_type {
                CallType::Call {
                    callee_ptr,
                    value_ptr,
                    deposit_ptr,
                    weight,
                } => {
                    let callee: <<E as Ext>::T as frame_system::Config>::AccountId =
                        self.read_sandbox_memory_as(memory, callee_ptr)?;
                    let deposit_limit: BalanceOf<<E as Ext>::T> = if deposit_ptr == SENTINEL {
                        BalanceOf::<<E as Ext>::T>::zero()
                    } else {
                        self.read_sandbox_memory_as(memory, deposit_ptr)?
                    };
                    let value: BalanceOf<<E as Ext>::T> =
                        self.read_sandbox_memory_as(memory, value_ptr)?;
                    if value > 0u32.into() {
                        self.charge_gas(RuntimeCosts::CallSurchargeTransfer)?;
                    }
                    self.ext.call(
                        weight,
                        deposit_limit,
                        callee,
                        value,
                        input_data,
                        flags.contains(CallFlags::ALLOW_REENTRY),
                    )
                }
                CallType::DelegateCall { code_hash_ptr } => {
                    if flags.contains(CallFlags::ALLOW_REENTRY) {
                        return Err(Error::<E::T>::InvalidCallFlags.into());
                    }
                    let code_hash = self.read_sandbox_memory_as(memory, code_hash_ptr)?;
                    self.ext.delegate_call(code_hash, input_data)
                }
            };
            if flags.contains(CallFlags::TAIL_CALL) {
                if let Ok(return_value) = call_outcome {
                    return Err(TrapReason::Return(ReturnData {
                        flags: return_value.flags.bits(),
                        data: return_value.data,
                    }));
                }
            }
            if let Ok(output) = &call_outcome {
                self.write_sandbox_output(
                    memory,
                    output_ptr,
                    output_len_ptr,
                    &output.data,
                    true,
                    |len| Some(RuntimeCosts::CopyToContract(len)),
                )?;
            }
            Ok(Runtime::<E>::exec_into_return_code(call_outcome)?)
        }
        fn instantiate(
            &mut self,
            memory: &mut [u8],
            code_hash_ptr: u32,
            weight: Weight,
            deposit_ptr: u32,
            value_ptr: u32,
            input_data_ptr: u32,
            input_data_len: u32,
            address_ptr: u32,
            address_len_ptr: u32,
            output_ptr: u32,
            output_len_ptr: u32,
            salt_ptr: u32,
            salt_len: u32,
        ) -> Result<ReturnCode, TrapReason> {
            self.charge_gas(RuntimeCosts::InstantiateBase {
                input_data_len,
                salt_len,
            })?;
            let deposit_limit: BalanceOf<<E as Ext>::T> = if deposit_ptr == SENTINEL {
                BalanceOf::<<E as Ext>::T>::zero()
            } else {
                self.read_sandbox_memory_as(memory, deposit_ptr)?
            };
            let value: BalanceOf<<E as Ext>::T> =
                self.read_sandbox_memory_as(memory, value_ptr)?;
            if value > 0u32.into() {
                self.charge_gas(RuntimeCosts::InstantiateSurchargeTransfer)?;
            }
            let code_hash: CodeHash<<E as Ext>::T> =
                self.read_sandbox_memory_as(memory, code_hash_ptr)?;
            let input_data =
                self.read_sandbox_memory(memory, input_data_ptr, input_data_len)?;
            let salt = self.read_sandbox_memory(memory, salt_ptr, salt_len)?;
            let instantiate_outcome = self.ext.instantiate(
                weight,
                deposit_limit,
                code_hash,
                value,
                input_data,
                &salt,
            );
            if let Ok((address, output)) = &instantiate_outcome {
                if !output.flags.contains(ReturnFlags::REVERT) {
                    self.write_sandbox_output(
                        memory,
                        address_ptr,
                        address_len_ptr,
                        &address.encode(),
                        true,
                        already_charged,
                    )?;
                }
                self.write_sandbox_output(
                    memory,
                    output_ptr,
                    output_len_ptr,
                    &output.data,
                    true,
                    |len| Some(RuntimeCosts::CopyToContract(len)),
                )?;
            }
            Ok(Runtime::<E>::exec_into_return_code(
                instantiate_outcome.map(|(_, retval)| retval),
            )?)
        }
        fn terminate(&mut self, memory: &[u8], beneficiary_ptr: u32) -> Result<(), TrapReason> {
            self.charge_gas(RuntimeCosts::Terminate)?;
            let beneficiary: <<E as Ext>::T as frame_system::Config>::AccountId =
                self.read_sandbox_memory_as(memory, beneficiary_ptr)?;
            self.ext.terminate(&beneficiary)?;
            Err(TrapReason::Termination)
        }
    }
    
    // Env的define会把所有的
    // #[define_env(doc)]
    // pub mod env {}
    // 里的Host Function函数注册到linker里, 在合约初始化的时候导入到ink contract wasm module里
    pub struct Env;
    // 将host function 注册到wasmi中
    // Linker: 用于定义模块导入和实例化模块实例的链接器。
    // linker.define用于注册, 在此 Linker中定义一个新项目。
    // 如果已经有一个同名 Linker的定义会返回一个错误
    // 该函数如下
    //     pub fn define(
    //         &mut self,
    //         module: &str,
    //         name: &str,
    //         item: impl Into<Extern>,
    //     ) -> Result<&mut Self, LinkerError> {
    //         let key = self.import_key(module, name);
    //         self.insert(key, Definition::Extern(item.into()))?;
    //         Ok(self)
    //     }
    // 其中前一个str为module名称 例如 seal1
    // 其中前二个str为该item的名称 例如 instantiate
    // 第三个参数为导入的item
    // item如下目前wam只支持4中类型
    // WebAssembly 模块的外部项。
    //  这是从 Instance::exports 或 Instance::get_export返回的
    // #[derive(Debug, Copy, Clone)]
    // pub enum Extern {
    //     /// A WebAssembly global which acts like a [`Cell<T>`] of sorts, supporting `get` and `set` operations.
    //     ///
    //     /// [`Cell<T>`]: https://doc.rust-lang.org/core/cell/struct.Cell.html
    //     Global(Global),
    //     /// A WebAssembly table which is an array of funtion references.
    //     Table(Table),
    //     /// A WebAssembly linear memory.
    //     Memory(Memory),
    //     /// A WebAssembly function which can be called.
    //     Func(Func),
    // }
    // 我们这里只是到如function(host function)
    // 以为例, 导入过程如下:
    /*
        linker.define(
                    "seal0",
                    "contains_storage",
                    ::wasmi::Func::wrap(
                        &mut *store,
                        |mut __caller__: ::wasmi::Caller<crate::wasm::Runtime<E>>,
                         key_ptr: u32|
                         -> ::core::result::Result<
                             ::core::primitive::u32,
                             ::wasmi::core::Trap,
                         > {
                            let mut func = || -> Result<u32, TrapReason> {
                                let (memory, ctx) = __caller__
                                    .data()
                                    .memory()
                                    .expect("Memory must be set when setting up host data; qed")
                                    .data_and_store_mut(&mut __caller__);
                                if {
                                    let lvl = ::log::Level::Trace;
                                    lvl <= ::log::STATIC_MAX_LEVEL
                                        && lvl <= ::log::max_level()
                                        && ::log::__private_api_enabled(
                                        lvl,
                                        "runtime::contracts::strace",
                                    )
                                } {
                                    let result =
                                        { ctx.contains_storage(memory, KeyType::Fix, key_ptr) };
                                    {
                                        use sp_std::fmt::Write;
                                        let mut w = sp_std::Writer::default();
                                        let _ = (&mut w).write_fmt(format_args!(
                                            "seal0::contains_storage(key_ptr: {0:?}) = {1:?}\n",
                                            key_ptr, result
                                        ));
                                        let msg = core::str::from_utf8(&w.inner())
                                            .unwrap_or_default();
                                        ctx.ext().append_debug_buffer(msg);
                                    }
                                    result
                                } else {
                                    {
                                        ctx.contains_storage(memory, KeyType::Fix, key_ptr)
                                    }
                                }
                            };
                            func()
                                .map_err(|reason| ::wasmi::core::Trap::from(reason))
                                .map(::core::convert::Into::into)
                        },
                    ),
                )?;
     */
    /// #[define_env(doc)]
    /// pub mod env {}
    /// 展开的实现就是下面的内容, 就是把所有的Host Function注册到linker里
    /// 注册的函数并不是pub mod env里的, pub mod env里的函数作用就是确定module名称和注册Host Function
    /// 而真正的HostFunction是Runtime里实现的这一套函数, 在pod mod env里进行注册这一套函数
    impl<'a, E: Ext> crate::wasm::Environment<crate::wasm::runtime::Runtime<'a, E>> for Env {
        fn define(
            store: &mut ::wasmi::Store<crate::wasm::Runtime<E>>,
            linker: &mut ::wasmi::Linker<crate::wasm::Runtime<E>>,
            allow_unstable: AllowUnstableInterface,
            allow_deprecated: AllowDeprecatedInterface,
        ) -> Result<(), ::wasmi::errors::LinkerError> {
            let __allow_unstable__ = match allow_unstable {
                AllowUnstableInterface::Yes => true,
                _ => false,
            };
            let __allow_deprecated__ = match allow_deprecated {
                AllowDeprecatedInterface::Yes => true,
                _ => false,
            };
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "gas" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , amount : u64 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: MeteringBlock (amount)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::gas(amount: {0:?}) = {1:?}\n" , amount , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: MeteringBlock (amount)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) . map (| _ | ()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::set_storage(key_ptr: {0:?}, value_ptr: {1:?}, value_len: {2:?}) = {3:?}\n" , key_ptr , value_ptr , value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) . map (| _ | ()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::set_storage(key_ptr: {0:?}, value_ptr: {1:?}, value_len: {2:?}) = {3:?}\n" , key_ptr , value_ptr , value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal2" , "set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . set_storage (memory , KeyType :: Var (key_len) , key_ptr , value_ptr , value_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal2::set_storage(key_ptr: {0:?}, key_len: {1:?}, value_ptr: {2:?}, value_len: {3:?}) = {4:?}\n" , key_ptr , key_len , value_ptr , value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . set_storage (memory , KeyType :: Var (key_len) , key_ptr , value_ptr , value_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . clear_storage (memory , KeyType :: Fix , key_ptr) . map (| _ | ()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::clear_storage(key_ptr: {0:?}) = {1:?}\n" , key_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . clear_storage (memory , KeyType :: Fix , key_ptr) . map (| _ | ()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . clear_storage (memory , KeyType :: Var (key_len) , key_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::clear_storage(key_ptr: {0:?}, key_len: {1:?}) = {2:?}\n" , key_ptr , key_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . clear_storage (memory , KeyType :: Var (key_len) , key_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . get_storage (memory , KeyType :: Fix , key_ptr , out_ptr , out_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::get_storage(key_ptr: {0:?}, out_ptr: {1:?}, out_len_ptr: {2:?}) = {3:?}\n" , key_ptr , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . get_storage (memory , KeyType :: Fix , key_ptr , out_ptr , out_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . get_storage (memory , KeyType :: Var (key_len) , key_ptr , out_ptr , out_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::get_storage(key_ptr: {0:?}, key_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , key_ptr , key_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . get_storage (memory , KeyType :: Var (key_len) , key_ptr , out_ptr , out_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker.define(
                    "seal0",
                    "contains_storage",
                    // 从给定的静态类型闭包创建新的主机函数蹦床。
                    // 把host function包装了一下
                    // 从给定的闭包创建新的主机函数。
                    ::wasmi::Func::wrap(
                        // 其中 store就保存了一个Runtime实例, 也就是说store持有了Runtime
                        // 并且后面调用时会把这个store传递到闭包里, 也就是下面的caller
                        // caller.data就是返回 Store的data
                        // Store定义如下
                        // /// 拥有与 Wasm 模块关联的所有数据的存储。
                        // #[derive(Debug)]
                        // pub struct Store<T> {
                        //     /// All data that is not associated to `T`.
                        //     ///
                        //     /// # Note
                        //     ///
                        //     /// This is re-exported to the rest of the crate since
                        //     /// it is used directly by the engine's executor.
                        //     pub(crate) inner: StoreInner,
                        //     /// Stored host function trampolines.
                        //     trampolines: Arena<TrampolineIdx, TrampolineEntity<T>>,
                        //     /// User provided host data owned by the [`Store`].
                        //     data: T,
                        // }
                        // caller.data就是store.data, 就是T, 也就是持有的Runtime
                        // 那么闭包里真正执行的ctx.contains_storage, 所以HostFunction实际上时Runtime提供的各个
                        // 函数, Runtime持有一个Ext, Runtime对Ext进行了包装,
                        // Runtime 拿到最原始的数据, 比如 input_ptr和input_len_ptr
                        // Runtime将这些数据读取出来组装到input原本的数据结构里然后调用Ext的对应的方法
                        // Ext是HostFunction的底层实现, 它拿到Rust这侧直接可以使用的各种参数然后执行返回到Runtime
                        // Runtime 将结果再转换成内存地址表示的方式返回到Contract wasm中
                        &mut *store,
                        // 该闭内会调用contains_storage执行并返回结果
                        |mut __caller__: ::wasmi::Caller<crate::wasm::Runtime<E>>,
                         key_ptr: u32|
                         -> ::core::result::Result<
                             ::core::primitive::u32,
                             ::wasmi::core::Trap,
                         > {
                            // 构造执行host function的闭包
                            // 在这里面执行contains_storage
                            let mut func = || -> Result<u32, TrapReason> {
                                let (memory, ctx) = __caller__
                                    // 这里data就是返回的这里的Runtime
                                    .data()
                                    // 这里返回的是Runtime的memory
                                    .memory()
                                    // 设置主机数据时必须设置内存
                                    .expect("Memory must be set when setting up host data; qed")
                                    // 返回对 基础 Memory字节的独占片，以及对用户提供的状态的独占引用。
                                    // panic，如果 ctx 不拥有这个 Memory.
                                    // 即Runtime不拥有这个Memory
                                    .data_and_store_mut(&mut __caller__);
                                if {
                                    let lvl = ::log::Level::Trace;
                                    lvl <= ::log::STATIC_MAX_LEVEL
                                        && lvl <= ::log::max_level()
                                        && ::log::__private_api_enabled(
                                        lvl,
                                        "runtime::contracts::strace",
                                    )
                                } {
                                    // 执行该函数
                                    let result =
                                        { ctx.contains_storage(memory, KeyType::Fix, key_ptr) };
                                    {
                                        use sp_std::fmt::Write;
                                        let mut w = sp_std::Writer::default();
                                        let _ = (&mut w).write_fmt(format_args!(
                                            "seal0::contains_storage(key_ptr: {0:?}) = {1:?}\n",
                                            key_ptr, result
                                        ));
                                        let msg = core::str::from_utf8(&w.inner())
                                            .unwrap_or_default();
                                        ctx.ext().append_debug_buffer(msg);
                                    }
                                    result
                                } else {
                                    {
                                        ctx.contains_storage(memory, KeyType::Fix, key_ptr)
                                    }
                                }
                            };
                            // 这里真正调用执行
                            func()
                                .map_err(|reason| ::wasmi::core::Trap::from(reason))
                                .map(::core::convert::Into::into)
                        },
                    ),
                )?;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . contains_storage (memory , KeyType :: Var (key_len) , key_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::contains_storage(key_ptr: {0:?}, key_len: {1:?}) = {2:?}\n" , key_ptr , key_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . contains_storage (memory , KeyType :: Var (key_len) , key_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "take_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let charged = ctx . charge_gas (RuntimeCosts :: TakeStorage (ctx . ext . max_value_size ())) ? ; { if ! (key_len <= < < E as Ext > :: T as Config > :: MaxStorageKeyLen :: get ()) { { return Err (Error :: < E :: T > :: DecodingFailed . into ()) } ; } } ; let key = ctx . read_sandbox_memory (memory , key_ptr , key_len) ? ; if let crate :: storage :: WriteOutcome :: Taken (value) = ctx . ext . set_storage (& Key :: < E :: T > :: try_from_var (key) . map_err (| _ | Error :: < E :: T > :: DecodingFailed) ? , None , true) ? { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (value . len () as u32)) ; ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (0)) ; Ok (ReturnCode :: KeyNotFound) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::take_storage(key_ptr: {0:?}, key_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , key_ptr , key_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let charged = ctx . charge_gas (RuntimeCosts :: TakeStorage (ctx . ext . max_value_size ())) ? ; { if ! (key_len <= < < E as Ext > :: T as Config > :: MaxStorageKeyLen :: get ()) { { return Err (Error :: < E :: T > :: DecodingFailed . into ()) } ; } } ; let key = ctx . read_sandbox_memory (memory , key_ptr , key_len) ? ; if let crate :: storage :: WriteOutcome :: Taken (value) = ctx . ext . set_storage (& Key :: < E :: T > :: try_from_var (key) . map_err (| _ | Error :: < E :: T > :: DecodingFailed) ? , None , true) ? { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (value . len () as u32)) ; ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (0)) ; Ok (ReturnCode :: KeyNotFound) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "transfer" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 , _account_len : u32 , value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Transfer) ? ; let callee : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; let value : BalanceOf < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , value_ptr) ? ; let result = ctx . ext . transfer (& callee , value) ; match result { Ok (()) => Ok (ReturnCode :: Success) , Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::transfer(account_ptr: {0:?}, _account_len: {1:?}, value_ptr: {2:?}, _value_len: {3:?}) = {4:?}\n" , account_ptr , _account_len , value_ptr , _value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Transfer) ? ; let callee : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; let value : BalanceOf < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , value_ptr) ? ; let result = ctx . ext . transfer (& callee , value) ; match result { Ok (()) => Ok (ReturnCode :: Success) , Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , callee_ptr : u32 , _callee_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: ALLOW_REENTRY , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::call(callee_ptr: {0:?}, _callee_len: {1:?}, gas: {2:?}, value_ptr: {3:?}, _value_len: {4:?}, input_data_ptr: {5:?}, input_data_len: {6:?}, output_ptr: {7:?}, output_len_ptr: {8:?}) = {9:?}\n" , callee_ptr , _callee_len , gas , value_ptr , _value_len , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: ALLOW_REENTRY , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , flags : u32 , callee_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::call(flags: {0:?}, callee_ptr: {1:?}, gas: {2:?}, value_ptr: {3:?}, input_data_ptr: {4:?}, input_data_len: {5:?}, output_ptr: {6:?}, output_len_ptr: {7:?}) = {8:?}\n" , flags , callee_ptr , gas , value_ptr , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal2" , "call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , flags : u32 , callee_ptr : u32 , ref_time_limit : u64 , proof_size_limit : u64 , deposit_ptr : u32 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: Call { callee_ptr , value_ptr , deposit_ptr , weight : Weight :: from_parts (ref_time_limit , proof_size_limit) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal2::call(flags: {0:?}, callee_ptr: {1:?}, ref_time_limit: {2:?}, proof_size_limit: {3:?}, deposit_ptr: {4:?}, value_ptr: {5:?}, input_data_ptr: {6:?}, input_data_len: {7:?}, output_ptr: {8:?}, output_len_ptr: {9:?}) = {10:?}\n" , flags , callee_ptr , ref_time_limit , proof_size_limit , deposit_ptr , value_ptr , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: Call { callee_ptr , value_ptr , deposit_ptr , weight : Weight :: from_parts (ref_time_limit , proof_size_limit) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "delegate_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , flags : u32 , code_hash_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: DelegateCall { code_hash_ptr } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::delegate_call(flags: {0:?}, code_hash_ptr: {1:?}, input_data_ptr: {2:?}, input_data_len: {3:?}, output_ptr: {4:?}, output_len_ptr: {5:?}) = {6:?}\n" , flags , code_hash_ptr , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: DelegateCall { code_hash_ptr } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 , _code_hash_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::instantiate(code_hash_ptr: {0:?}, _code_hash_len: {1:?}, gas: {2:?}, value_ptr: {3:?}, _value_len: {4:?}, input_data_ptr: {5:?}, input_data_len: {6:?}, address_ptr: {7:?}, address_len_ptr: {8:?}, output_ptr: {9:?}, output_len_ptr: {10:?}, salt_ptr: {11:?}, salt_len: {12:?}) = {13:?}\n" , code_hash_ptr , _code_hash_len , gas , value_ptr , _value_len , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::instantiate(code_hash_ptr: {0:?}, gas: {1:?}, value_ptr: {2:?}, input_data_ptr: {3:?}, input_data_len: {4:?}, address_ptr: {5:?}, address_len_ptr: {6:?}, output_ptr: {7:?}, output_len_ptr: {8:?}, salt_ptr: {9:?}, salt_len: {10:?}) = {11:?}\n" , code_hash_ptr , gas , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal2" , "instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 , ref_time_limit : u64 , proof_size_limit : u64 , deposit_ptr : u32 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (ref_time_limit , proof_size_limit) , deposit_ptr , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal2::instantiate(code_hash_ptr: {0:?}, ref_time_limit: {1:?}, proof_size_limit: {2:?}, deposit_ptr: {3:?}, value_ptr: {4:?}, input_data_ptr: {5:?}, input_data_len: {6:?}, address_ptr: {7:?}, address_len_ptr: {8:?}, output_ptr: {9:?}, output_len_ptr: {10:?}, salt_ptr: {11:?}, salt_len: {12:?}) = {13:?}\n" , code_hash_ptr , ref_time_limit , proof_size_limit , deposit_ptr , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (ref_time_limit , proof_size_limit) , deposit_ptr , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , beneficiary_ptr : u32 , _beneficiary_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . terminate (memory , beneficiary_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::terminate(beneficiary_ptr: {0:?}, _beneficiary_len: {1:?}) = {2:?}\n" , beneficiary_ptr , _beneficiary_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . terminate (memory , beneficiary_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , beneficiary_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . terminate (memory , beneficiary_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::terminate(beneficiary_ptr: {0:?}) = {1:?}\n" , beneficiary_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . terminate (memory , beneficiary_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "input" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: InputBase) ? ; if let Some (input) = ctx . input_data . take () { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & input , false , | len | { Some (RuntimeCosts :: CopyToContract (len)) }) ? ; ctx . input_data = Some (input) ; Ok (()) } else { Err (Error :: < E :: T > :: InputForwarded . into ()) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::input(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: InputBase) ? ; if let Some (input) = ctx . input_data . take () { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & input , false , | len | { Some (RuntimeCosts :: CopyToContract (len)) }) ? ; ctx . input_data = Some (input) ; Ok (()) } else { Err (Error :: < E :: T > :: InputForwarded . into ()) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_return" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , flags : u32 , data_ptr : u32 , data_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Return (data_len)) ? ; Err (TrapReason :: Return (ReturnData { flags , data : ctx . read_sandbox_memory (memory , data_ptr , data_len) ? , })) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_return(flags: {0:?}, data_ptr: {1:?}, data_len: {2:?}) = {3:?}\n" , flags , data_ptr , data_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Return (data_len)) ? ; Err (TrapReason :: Return (ReturnData { flags , data : ctx . read_sandbox_memory (memory , data_ptr , data_len) ? , })) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "caller" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Caller) ? ; let caller = ctx . ext . caller () . account_id () ? . clone () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & caller . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::caller(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Caller) ? ; let caller = ctx . ext . caller () . account_id () ? . clone () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & caller . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "is_contract" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: IsContract) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; Ok (ctx . ext . is_contract (& address) as u32) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::is_contract(account_ptr: {0:?}) = {1:?}\n" , account_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: IsContract) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; Ok (ctx . ext . is_contract (& address) as u32) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: CodeHash) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; if let Some (value) = ctx . ext . code_hash (& address) { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value . encode () , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { Ok (ReturnCode :: KeyNotFound) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::code_hash(account_ptr: {0:?}, out_ptr: {1:?}, out_len_ptr: {2:?}) = {3:?}\n" , account_ptr , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: CodeHash) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; if let Some (value) = ctx . ext . code_hash (& address) { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value . encode () , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { Ok (ReturnCode :: KeyNotFound) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "own_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: OwnCodeHash) ? ; let code_hash_encoded = & ctx . ext . own_code_hash () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , code_hash_encoded , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::own_code_hash(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: OwnCodeHash) ? ; let code_hash_encoded = & ctx . ext . own_code_hash () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , code_hash_encoded , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "caller_is_origin" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: CallerIsOrigin) ? ; Ok (ctx . ext . caller_is_origin () as u32) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::caller_is_origin() = {0:?}\n" , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: CallerIsOrigin) ? ; Ok (ctx . ext . caller_is_origin () as u32) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "caller_is_root" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: CallerIsRoot) ? ; Ok (ctx . ext . caller_is_root () as u32) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::caller_is_root() = {0:?}\n" , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: CallerIsRoot) ? ; Ok (ctx . ext . caller_is_root () as u32) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Address) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . address () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::address(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Address) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . address () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "weight_to_fee" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , gas : u64 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let gas = Weight :: from_parts (gas , 0) ; ctx . charge_gas (RuntimeCosts :: WeightToFee) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . get_weight_price (gas) . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::weight_to_fee(gas: {0:?}, out_ptr: {1:?}, out_len_ptr: {2:?}) = {3:?}\n" , gas , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let gas = Weight :: from_parts (gas , 0) ; ctx . charge_gas (RuntimeCosts :: WeightToFee) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . get_weight_price (gas) . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "weight_to_fee" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , ref_time_limit : u64 , proof_size_limit : u64 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let weight = Weight :: from_parts (ref_time_limit , proof_size_limit) ; ctx . charge_gas (RuntimeCosts :: WeightToFee) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . get_weight_price (weight) . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::weight_to_fee(ref_time_limit: {0:?}, proof_size_limit: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , ref_time_limit , proof_size_limit , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let weight = Weight :: from_parts (ref_time_limit , proof_size_limit) ; ctx . charge_gas (RuntimeCosts :: WeightToFee) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . get_weight_price (weight) . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "gas_left" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: GasLeft) ? ; let gas_left = & ctx . ext . gas_meter () . gas_left () . ref_time () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , gas_left , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::gas_left(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: GasLeft) ? ; let gas_left = & ctx . ext . gas_meter () . gas_left () . ref_time () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , gas_left , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "gas_left" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: GasLeft) ? ; let gas_left = & ctx . ext . gas_meter () . gas_left () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , gas_left , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::gas_left(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: GasLeft) ? ; let gas_left = & ctx . ext . gas_meter () . gas_left () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , gas_left , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Balance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . balance () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::balance(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Balance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . balance () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "value_transferred" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: ValueTransferred) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . value_transferred () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::value_transferred(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: ValueTransferred) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . value_transferred () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . 0 . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::random(subject_ptr: {0:?}, subject_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , subject_ptr , subject_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . 0 . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal1" , "random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::random(subject_ptr: {0:?}, subject_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , subject_ptr , subject_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "now" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Now) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . now () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::now(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Now) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . now () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "minimum_balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: MinimumBalance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . minimum_balance () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::minimum_balance(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: MinimumBalance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . minimum_balance () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "tombstone_deposit" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let deposit = < BalanceOf < E :: T > > :: zero () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & deposit , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::tombstone_deposit(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let deposit = < BalanceOf < E :: T > > :: zero () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & deposit , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _dest_ptr : u32 , _dest_len : u32 , _code_hash_ptr : u32 , _code_hash_len : u32 , _rent_allowance_ptr : u32 , _rent_allowance_len : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::restore_to(_dest_ptr: {0:?}, _dest_len: {1:?}, _code_hash_ptr: {2:?}, _code_hash_len: {3:?}, _rent_allowance_ptr: {4:?}, _rent_allowance_len: {5:?}, _delta_ptr: {6:?}, _delta_count: {7:?}) = {8:?}\n" , _dest_ptr , _dest_len , _code_hash_ptr , _code_hash_len , _rent_allowance_ptr , _rent_allowance_len , _delta_ptr , _delta_count , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal1" , "restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _dest_ptr : u32 , _code_hash_ptr : u32 , _rent_allowance_ptr : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::restore_to(_dest_ptr: {0:?}, _code_hash_ptr: {1:?}, _rent_allowance_ptr: {2:?}, _delta_ptr: {3:?}, _delta_count: {4:?}) = {5:?}\n" , _dest_ptr , _code_hash_ptr , _rent_allowance_ptr , _delta_ptr , _delta_count , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::set_rent_allowance(_value_ptr: {0:?}, _value_len: {1:?}) = {2:?}\n" , _value_ptr , _value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal1" , "set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _value_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::set_rent_allowance(_value_ptr: {0:?}) = {1:?}\n" , _value_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let rent_allowance = < BalanceOf < E :: T > > :: max_value () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & rent_allowance , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::rent_allowance(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let rent_allowance = < BalanceOf < E :: T > > :: max_value () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & rent_allowance , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "deposit_event" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , topics_ptr : u32 , topics_len : u32 , data_ptr : u32 , data_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let num_topic = topics_len . checked_div (sp_std :: mem :: size_of :: < TopicOf < E :: T > > () as u32) . ok_or ("Zero sized topics are not allowed") ? ; ctx . charge_gas (RuntimeCosts :: DepositEvent { num_topic , len : data_len , }) ? ; if data_len > ctx . ext . max_value_size () { return Err (Error :: < E :: T > :: ValueTooLarge . into ()) } let topics : Vec < TopicOf < < E as Ext > :: T > > = match topics_len { 0 => Vec :: new () , _ => ctx . read_sandbox_memory_as_unbounded (memory , topics_ptr , topics_len) ? , } ; if topics . len () > ctx . ext . schedule () . limits . event_topics as usize { return Err (Error :: < E :: T > :: TooManyTopics . into ()) } let event_data = ctx . read_sandbox_memory (memory , data_ptr , data_len) ? ; ctx . ext . deposit_event (topics , event_data) ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::deposit_event(topics_ptr: {0:?}, topics_len: {1:?}, data_ptr: {2:?}, data_len: {3:?}) = {4:?}\n" , topics_ptr , topics_len , data_ptr , data_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let num_topic = topics_len . checked_div (sp_std :: mem :: size_of :: < TopicOf < E :: T > > () as u32) . ok_or ("Zero sized topics are not allowed") ? ; ctx . charge_gas (RuntimeCosts :: DepositEvent { num_topic , len : data_len , }) ? ; if data_len > ctx . ext . max_value_size () { return Err (Error :: < E :: T > :: ValueTooLarge . into ()) } let topics : Vec < TopicOf < < E as Ext > :: T > > = match topics_len { 0 => Vec :: new () , _ => ctx . read_sandbox_memory_as_unbounded (memory , topics_ptr , topics_len) ? , } ; if topics . len () > ctx . ext . schedule () . limits . event_topics as usize { return Err (Error :: < E :: T > :: TooManyTopics . into ()) } let event_data = ctx . read_sandbox_memory (memory , data_ptr , data_len) ? ; ctx . ext . deposit_event (topics , event_data) ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "block_number" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: BlockNumber) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . block_number () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::block_number(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: BlockNumber) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . block_number () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "hash_sha2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashSha256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , sha2_256 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::hash_sha2_256(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashSha256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , sha2_256 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "hash_keccak_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashKeccak256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , keccak_256 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::hash_keccak_256(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashKeccak256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , keccak_256 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "hash_blake2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashBlake256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_256 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::hash_blake2_256(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashBlake256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_256 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "hash_blake2_128" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashBlake128 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_128 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::hash_blake2_128(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashBlake128 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_128 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "call_chain_extension" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , id : u32 , input_ptr : u32 , input_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { use crate :: chain_extension :: { ChainExtension , Environment , RetVal , } ; if ! < E :: T as Config > :: ChainExtension :: enabled () { return Err (Error :: < E :: T > :: NoChainExtension . into ()) } let mut chain_extension = ctx . chain_extension . take () . expect ("Constructor initializes with `Some`. This is the only place where it is set to `None`.\
			It is always reset to `Some` afterwards. qed") ; let env = Environment :: new (ctx , memory , id , input_ptr , input_len , output_ptr , output_len_ptr) ; let ret = match chain_extension . call (env) ? { RetVal :: Converging (val) => Ok (val) , RetVal :: Diverging { flags , data } => Err (TrapReason :: Return (ReturnData { flags : flags . bits () , data , })) , } ; ctx . chain_extension = Some (chain_extension) ; ret } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::call_chain_extension(id: {0:?}, input_ptr: {1:?}, input_len: {2:?}, output_ptr: {3:?}, output_len_ptr: {4:?}) = {5:?}\n" , id , input_ptr , input_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { use crate :: chain_extension :: { ChainExtension , Environment , RetVal , } ; if ! < E :: T as Config > :: ChainExtension :: enabled () { return Err (Error :: < E :: T > :: NoChainExtension . into ()) } let mut chain_extension = ctx . chain_extension . take () . expect ("Constructor initializes with `Some`. This is the only place where it is set to `None`.\
			It is always reset to `Some` afterwards. qed") ; let env = Environment :: new (ctx , memory , id , input_ptr , input_len , output_ptr , output_len_ptr) ; let ret = match chain_extension . call (env) ? { RetVal :: Converging (val) => Ok (val) , RetVal :: Diverging { flags , data } => Err (TrapReason :: Return (ReturnData { flags : flags . bits () , data , })) , } ; ctx . chain_extension = Some (chain_extension) ; ret } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "debug_message" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , str_ptr : u32 , str_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let str_len = str_len . min (DebugBufferVec :: < E :: T > :: bound () as u32) ; ctx . charge_gas (RuntimeCosts :: DebugMessage (str_len)) ? ; if ctx . ext . append_debug_buffer ("") { let data = ctx . read_sandbox_memory (memory , str_ptr , str_len) ? ; if let Some (msg) = core :: str :: from_utf8 (& data) . ok () { ctx . ext . append_debug_buffer (msg) ; } } Ok (ReturnCode :: Success) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::debug_message(str_ptr: {0:?}, str_len: {1:?}) = {2:?}\n" , str_ptr , str_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let str_len = str_len . min (DebugBufferVec :: < E :: T > :: bound () as u32) ; ctx . charge_gas (RuntimeCosts :: DebugMessage (str_len)) ? ; if ctx . ext . append_debug_buffer ("") { let data = ctx . read_sandbox_memory (memory , str_ptr , str_len) ? ; if let Some (msg) = core :: str :: from_utf8 (& data) . ok () { ctx . ext . append_debug_buffer (msg) ; } } Ok (ReturnCode :: Success) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "call_runtime" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , call_ptr : u32 , call_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { use frame_support :: dispatch :: { extract_actual_weight , GetDispatchInfo , } ; ctx . charge_gas (RuntimeCosts :: CopyFromContract (call_len)) ? ; let call : < E :: T as Config > :: RuntimeCall = ctx . read_sandbox_memory_as_unbounded (memory , call_ptr , call_len) ? ; let dispatch_info = call . get_dispatch_info () ; let charged = ctx . charge_gas (RuntimeCosts :: CallRuntime (dispatch_info . weight)) ? ; let result = ctx . ext . call_runtime (call) ; let actual_weight = extract_actual_weight (& result , & dispatch_info) ; ctx . adjust_gas (charged , RuntimeCosts :: CallRuntime (actual_weight)) ; match result { Ok (_) => Ok (ReturnCode :: Success) , Err (e) => { if ctx . ext . append_debug_buffer ("") { ctx . ext . append_debug_buffer ("seal0::call_runtime failed with: ") ; ctx . ext . append_debug_buffer (e . into ()) ; } ; Ok (ReturnCode :: CallRuntimeFailed) } } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::call_runtime(call_ptr: {0:?}, call_len: {1:?}) = {2:?}\n" , call_ptr , call_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { use frame_support :: dispatch :: { extract_actual_weight , GetDispatchInfo , } ; ctx . charge_gas (RuntimeCosts :: CopyFromContract (call_len)) ? ; let call : < E :: T as Config > :: RuntimeCall = ctx . read_sandbox_memory_as_unbounded (memory , call_ptr , call_len) ? ; let dispatch_info = call . get_dispatch_info () ; let charged = ctx . charge_gas (RuntimeCosts :: CallRuntime (dispatch_info . weight)) ? ; let result = ctx . ext . call_runtime (call) ; let actual_weight = extract_actual_weight (& result , & dispatch_info) ; ctx . adjust_gas (charged , RuntimeCosts :: CallRuntime (actual_weight)) ; match result { Ok (_) => Ok (ReturnCode :: Success) , Err (e) => { if ctx . ext . append_debug_buffer ("") { ctx . ext . append_debug_buffer ("seal0::call_runtime failed with: ") ; ctx . ext . append_debug_buffer (e . into ()) ; } ; Ok (ReturnCode :: CallRuntimeFailed) } } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "ecdsa_recover" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , signature_ptr : u32 , message_hash_ptr : u32 , output_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: EcdsaRecovery) ? ; let mut signature : [u8 ; 65] = [0 ; 65] ; ctx . read_sandbox_memory_into_buf (memory , signature_ptr , & mut signature) ? ; let mut message_hash : [u8 ; 32] = [0 ; 32] ; ctx . read_sandbox_memory_into_buf (memory , message_hash_ptr , & mut message_hash) ? ; let result = ctx . ext . ecdsa_recover (& signature , & message_hash) ; match result { Ok (pub_key) => { ctx . write_sandbox_memory (memory , output_ptr , pub_key . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::ecdsa_recover(signature_ptr: {0:?}, message_hash_ptr: {1:?}, output_ptr: {2:?}) = {3:?}\n" , signature_ptr , message_hash_ptr , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: EcdsaRecovery) ? ; let mut signature : [u8 ; 65] = [0 ; 65] ; ctx . read_sandbox_memory_into_buf (memory , signature_ptr , & mut signature) ? ; let mut message_hash : [u8 ; 32] = [0 ; 32] ; ctx . read_sandbox_memory_into_buf (memory , message_hash_ptr , & mut message_hash) ? ; let result = ctx . ext . ecdsa_recover (& signature , & message_hash) ; match result { Ok (pub_key) => { ctx . write_sandbox_memory (memory , output_ptr , pub_key . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "sr25519_verify" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , signature_ptr : u32 , pub_key_ptr : u32 , message_len : u32 , message_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Sr25519Verify (message_len)) ? ; let mut signature : [u8 ; 64] = [0 ; 64] ; ctx . read_sandbox_memory_into_buf (memory , signature_ptr , & mut signature) ? ; let mut pub_key : [u8 ; 32] = [0 ; 32] ; ctx . read_sandbox_memory_into_buf (memory , pub_key_ptr , & mut pub_key) ? ; let message : Vec < u8 > = ctx . read_sandbox_memory (memory , message_ptr , message_len) ? ; if ctx . ext . sr25519_verify (& signature , & message , & pub_key) { Ok (ReturnCode :: Success) } else { Ok (ReturnCode :: Sr25519VerifyFailed) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::sr25519_verify(signature_ptr: {0:?}, pub_key_ptr: {1:?}, message_len: {2:?}, message_ptr: {3:?}) = {4:?}\n" , signature_ptr , pub_key_ptr , message_len , message_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Sr25519Verify (message_len)) ? ; let mut signature : [u8 ; 64] = [0 ; 64] ; ctx . read_sandbox_memory_into_buf (memory , signature_ptr , & mut signature) ? ; let mut pub_key : [u8 ; 32] = [0 ; 32] ; ctx . read_sandbox_memory_into_buf (memory , pub_key_ptr , & mut pub_key) ? ; let message : Vec < u8 > = ctx . read_sandbox_memory (memory , message_ptr , message_len) ? ; if ctx . ext . sr25519_verify (& signature , & message , & pub_key) { Ok (ReturnCode :: Success) } else { Ok (ReturnCode :: Sr25519VerifyFailed) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "set_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: SetCodeHash) ? ; let code_hash : CodeHash < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , code_hash_ptr) ? ; match ctx . ext . set_code_hash (code_hash) { Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } Ok (()) => Ok (ReturnCode :: Success) , } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::set_code_hash(code_hash_ptr: {0:?}) = {1:?}\n" , code_hash_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: SetCodeHash) ? ; let code_hash : CodeHash < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , code_hash_ptr) ? ; match ctx . ext . set_code_hash (code_hash) { Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } Ok (()) => Ok (ReturnCode :: Success) , } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "ecdsa_to_eth_address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , out_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: EcdsaToEthAddress) ? ; let mut compressed_key : [u8 ; 33] = [0 ; 33] ; ctx . read_sandbox_memory_into_buf (memory , key_ptr , & mut compressed_key) ? ; let result = ctx . ext . ecdsa_to_eth_address (& compressed_key) ; match result { Ok (eth_address) => { ctx . write_sandbox_memory (memory , out_ptr , eth_address . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::ecdsa_to_eth_address(key_ptr: {0:?}, out_ptr: {1:?}) = {2:?}\n" , key_ptr , out_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: EcdsaToEthAddress) ? ; let mut compressed_key : [u8 ; 33] = [0 ; 33] ; ctx . read_sandbox_memory_into_buf (memory , key_ptr , & mut compressed_key) ? ; let result = ctx . ext . ecdsa_to_eth_address (& compressed_key) ; match result { Ok (eth_address) => { ctx . write_sandbox_memory (memory , out_ptr , eth_address . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "reentrance_count" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: ReentrantCount) ? ; Ok (ctx . ext . reentrance_count ()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::reentrance_count() = {0:?}\n" , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: ReentrantCount) ? ; Ok (ctx . ext . reentrance_count ()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "account_reentrance_count" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: AccountEntranceCount) ? ; let account_id : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; Ok (ctx . ext . account_reentrance_count (& account_id)) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::account_reentrance_count(account_ptr: {0:?}) = {1:?}\n" , account_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: AccountEntranceCount) ? ; let account_id : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; Ok (ctx . ext . account_reentrance_count (& account_id)) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "instantiation_nonce" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > | -> :: core :: result :: Result < :: core :: primitive :: u64 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u64 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: InstantationNonce) ? ; Ok (ctx . ext . nonce ()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::instantiation_nonce() = {0:?}\n" , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: InstantationNonce) ? ; Ok (ctx . ext . nonce ()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) . map (| _ | ()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_set_storage(key_ptr: {0:?}, value_ptr: {1:?}, value_len: {2:?}) = {3:?}\n" , key_ptr , value_ptr , value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) . map (| _ | ()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_set_storage(key_ptr: {0:?}, value_ptr: {1:?}, value_len: {2:?}) = {3:?}\n" , key_ptr , value_ptr , value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . set_storage (memory , KeyType :: Fix , key_ptr , value_ptr , value_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal2" , "seal_set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . set_storage (memory , KeyType :: Var (key_len) , key_ptr , value_ptr , value_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal2::seal_set_storage(key_ptr: {0:?}, key_len: {1:?}, value_ptr: {2:?}, value_len: {3:?}) = {4:?}\n" , key_ptr , key_len , value_ptr , value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . set_storage (memory , KeyType :: Var (key_len) , key_ptr , value_ptr , value_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . clear_storage (memory , KeyType :: Fix , key_ptr) . map (| _ | ()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_clear_storage(key_ptr: {0:?}) = {1:?}\n" , key_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . clear_storage (memory , KeyType :: Fix , key_ptr) . map (| _ | ()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . clear_storage (memory , KeyType :: Var (key_len) , key_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_clear_storage(key_ptr: {0:?}, key_len: {1:?}) = {2:?}\n" , key_ptr , key_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . clear_storage (memory , KeyType :: Var (key_len) , key_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . get_storage (memory , KeyType :: Fix , key_ptr , out_ptr , out_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_get_storage(key_ptr: {0:?}, out_ptr: {1:?}, out_len_ptr: {2:?}) = {3:?}\n" , key_ptr , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . get_storage (memory , KeyType :: Fix , key_ptr , out_ptr , out_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . get_storage (memory , KeyType :: Var (key_len) , key_ptr , out_ptr , out_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_get_storage(key_ptr: {0:?}, key_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , key_ptr , key_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . get_storage (memory , KeyType :: Var (key_len) , key_ptr , out_ptr , out_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . contains_storage (memory , KeyType :: Fix , key_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_contains_storage(key_ptr: {0:?}) = {1:?}\n" , key_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . contains_storage (memory , KeyType :: Fix , key_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . contains_storage (memory , KeyType :: Var (key_len) , key_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_contains_storage(key_ptr: {0:?}, key_len: {1:?}) = {2:?}\n" , key_ptr , key_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . contains_storage (memory , KeyType :: Var (key_len) , key_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_take_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let charged = ctx . charge_gas (RuntimeCosts :: TakeStorage (ctx . ext . max_value_size ())) ? ; { if ! (key_len <= < < E as Ext > :: T as Config > :: MaxStorageKeyLen :: get ()) { { return Err (Error :: < E :: T > :: DecodingFailed . into ()) } ; } } ; let key = ctx . read_sandbox_memory (memory , key_ptr , key_len) ? ; if let crate :: storage :: WriteOutcome :: Taken (value) = ctx . ext . set_storage (& Key :: < E :: T > :: try_from_var (key) . map_err (| _ | Error :: < E :: T > :: DecodingFailed) ? , None , true) ? { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (value . len () as u32)) ; ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (0)) ; Ok (ReturnCode :: KeyNotFound) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_take_storage(key_ptr: {0:?}, key_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , key_ptr , key_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let charged = ctx . charge_gas (RuntimeCosts :: TakeStorage (ctx . ext . max_value_size ())) ? ; { if ! (key_len <= < < E as Ext > :: T as Config > :: MaxStorageKeyLen :: get ()) { { return Err (Error :: < E :: T > :: DecodingFailed . into ()) } ; } } ; let key = ctx . read_sandbox_memory (memory , key_ptr , key_len) ? ; if let crate :: storage :: WriteOutcome :: Taken (value) = ctx . ext . set_storage (& Key :: < E :: T > :: try_from_var (key) . map_err (| _ | Error :: < E :: T > :: DecodingFailed) ? , None , true) ? { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (value . len () as u32)) ; ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { ctx . adjust_gas (charged , RuntimeCosts :: TakeStorage (0)) ; Ok (ReturnCode :: KeyNotFound) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_transfer" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 , _account_len : u32 , value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Transfer) ? ; let callee : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; let value : BalanceOf < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , value_ptr) ? ; let result = ctx . ext . transfer (& callee , value) ; match result { Ok (()) => Ok (ReturnCode :: Success) , Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_transfer(account_ptr: {0:?}, _account_len: {1:?}, value_ptr: {2:?}, _value_len: {3:?}) = {4:?}\n" , account_ptr , _account_len , value_ptr , _value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Transfer) ? ; let callee : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; let value : BalanceOf < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , value_ptr) ? ; let result = ctx . ext . transfer (& callee , value) ; match result { Ok (()) => Ok (ReturnCode :: Success) , Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , callee_ptr : u32 , _callee_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: ALLOW_REENTRY , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_call(callee_ptr: {0:?}, _callee_len: {1:?}, gas: {2:?}, value_ptr: {3:?}, _value_len: {4:?}, input_data_ptr: {5:?}, input_data_len: {6:?}, output_ptr: {7:?}, output_len_ptr: {8:?}) = {9:?}\n" , callee_ptr , _callee_len , gas , value_ptr , _value_len , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: ALLOW_REENTRY , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , flags : u32 , callee_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_call(flags: {0:?}, callee_ptr: {1:?}, gas: {2:?}, value_ptr: {3:?}, input_data_ptr: {4:?}, input_data_len: {5:?}, output_ptr: {6:?}, output_len_ptr: {7:?}) = {8:?}\n" , flags , callee_ptr , gas , value_ptr , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: Call { callee_ptr , value_ptr , deposit_ptr : SENTINEL , weight : Weight :: from_parts (gas , 0) , } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_delegate_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , flags : u32 , code_hash_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: DelegateCall { code_hash_ptr } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_delegate_call(flags: {0:?}, code_hash_ptr: {1:?}, input_data_ptr: {2:?}, input_data_len: {3:?}, output_ptr: {4:?}, output_len_ptr: {5:?}) = {6:?}\n" , flags , code_hash_ptr , input_data_ptr , input_data_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . call (memory , CallFlags :: from_bits (flags) . ok_or (Error :: < E :: T > :: InvalidCallFlags) ? , CallType :: DelegateCall { code_hash_ptr } , input_data_ptr , input_data_len , output_ptr , output_len_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 , _code_hash_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_instantiate(code_hash_ptr: {0:?}, _code_hash_len: {1:?}, gas: {2:?}, value_ptr: {3:?}, _value_len: {4:?}, input_data_ptr: {5:?}, input_data_len: {6:?}, address_ptr: {7:?}, address_len_ptr: {8:?}, output_ptr: {9:?}, output_len_ptr: {10:?}, salt_ptr: {11:?}, salt_len: {12:?}) = {13:?}\n" , code_hash_ptr , _code_hash_len , gas , value_ptr , _value_len , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_instantiate(code_hash_ptr: {0:?}, gas: {1:?}, value_ptr: {2:?}, input_data_ptr: {3:?}, input_data_len: {4:?}, address_ptr: {5:?}, address_len_ptr: {6:?}, output_ptr: {7:?}, output_len_ptr: {8:?}, salt_ptr: {9:?}, salt_len: {10:?}) = {11:?}\n" , code_hash_ptr , gas , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . instantiate (memory , code_hash_ptr , Weight :: from_parts (gas , 0) , SENTINEL , value_ptr , input_data_ptr , input_data_len , address_ptr , address_len_ptr , output_ptr , output_len_ptr , salt_ptr , salt_len) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , beneficiary_ptr : u32 , _beneficiary_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . terminate (memory , beneficiary_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_terminate(beneficiary_ptr: {0:?}, _beneficiary_len: {1:?}) = {2:?}\n" , beneficiary_ptr , _beneficiary_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . terminate (memory , beneficiary_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , beneficiary_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . terminate (memory , beneficiary_ptr) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_terminate(beneficiary_ptr: {0:?}) = {1:?}\n" , beneficiary_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . terminate (memory , beneficiary_ptr) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_input" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: InputBase) ? ; if let Some (input) = ctx . input_data . take () { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & input , false , | len | { Some (RuntimeCosts :: CopyToContract (len)) }) ? ; ctx . input_data = Some (input) ; Ok (()) } else { Err (Error :: < E :: T > :: InputForwarded . into ()) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_input(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: InputBase) ? ; if let Some (input) = ctx . input_data . take () { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & input , false , | len | { Some (RuntimeCosts :: CopyToContract (len)) }) ? ; ctx . input_data = Some (input) ; Ok (()) } else { Err (Error :: < E :: T > :: InputForwarded . into ()) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_caller" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Caller) ? ; let caller = ctx . ext . caller () . account_id () ? . clone () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & caller . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_caller(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Caller) ? ; let caller = ctx . ext . caller () . account_id () ? . clone () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & caller . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_is_contract" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: IsContract) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; Ok (ctx . ext . is_contract (& address) as u32) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_is_contract(account_ptr: {0:?}) = {1:?}\n" , account_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: IsContract) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; Ok (ctx . ext . is_contract (& address) as u32) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , account_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: CodeHash) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; if let Some (value) = ctx . ext . code_hash (& address) { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value . encode () , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { Ok (ReturnCode :: KeyNotFound) } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_code_hash(account_ptr: {0:?}, out_ptr: {1:?}, out_len_ptr: {2:?}) = {3:?}\n" , account_ptr , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: CodeHash) ? ; let address : < < E as Ext > :: T as frame_system :: Config > :: AccountId = ctx . read_sandbox_memory_as (memory , account_ptr) ? ; if let Some (value) = ctx . ext . code_hash (& address) { ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & value . encode () , false , already_charged) ? ; Ok (ReturnCode :: Success) } else { Ok (ReturnCode :: KeyNotFound) } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_own_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: OwnCodeHash) ? ; let code_hash_encoded = & ctx . ext . own_code_hash () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , code_hash_encoded , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_own_code_hash(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: OwnCodeHash) ? ; let code_hash_encoded = & ctx . ext . own_code_hash () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , code_hash_encoded , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_caller_is_origin" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: CallerIsOrigin) ? ; Ok (ctx . ext . caller_is_origin () as u32) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_caller_is_origin() = {0:?}\n" , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: CallerIsOrigin) ? ; Ok (ctx . ext . caller_is_origin () as u32) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Address) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . address () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_address(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Address) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . address () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_weight_to_fee" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , gas : u64 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let gas = Weight :: from_parts (gas , 0) ; ctx . charge_gas (RuntimeCosts :: WeightToFee) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . get_weight_price (gas) . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_weight_to_fee(gas: {0:?}, out_ptr: {1:?}, out_len_ptr: {2:?}) = {3:?}\n" , gas , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let gas = Weight :: from_parts (gas , 0) ; ctx . charge_gas (RuntimeCosts :: WeightToFee) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . get_weight_price (gas) . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_gas_left" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: GasLeft) ? ; let gas_left = & ctx . ext . gas_meter () . gas_left () . ref_time () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , gas_left , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_gas_left(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: GasLeft) ? ; let gas_left = & ctx . ext . gas_meter () . gas_left () . ref_time () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , gas_left , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Balance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . balance () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_balance(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Balance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . balance () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_value_transferred" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: ValueTransferred) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . value_transferred () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_value_transferred(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: ValueTransferred) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . value_transferred () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . 0 . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_random(subject_ptr: {0:?}, subject_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , subject_ptr , subject_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . 0 . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_random(subject_ptr: {0:?}, subject_len: {1:?}, out_ptr: {2:?}, out_len_ptr: {3:?}) = {4:?}\n" , subject_ptr , subject_len , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Random) ? ; if subject_len > ctx . ext . schedule () . limits . subject_len { return Err (Error :: < E :: T > :: RandomSubjectTooLong . into ()) } let subject_buf = ctx . read_sandbox_memory (memory , subject_ptr , subject_len) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . random (& subject_buf) . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_now" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Now) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . now () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_now(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Now) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . now () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_minimum_balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: MinimumBalance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . minimum_balance () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_minimum_balance(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: MinimumBalance) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . minimum_balance () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_tombstone_deposit" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let deposit = < BalanceOf < E :: T > > :: zero () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & deposit , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_tombstone_deposit(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let deposit = < BalanceOf < E :: T > > :: zero () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & deposit , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _dest_ptr : u32 , _dest_len : u32 , _code_hash_ptr : u32 , _code_hash_len : u32 , _rent_allowance_ptr : u32 , _rent_allowance_len : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_restore_to(_dest_ptr: {0:?}, _dest_len: {1:?}, _code_hash_ptr: {2:?}, _code_hash_len: {3:?}, _rent_allowance_ptr: {4:?}, _rent_allowance_len: {5:?}, _delta_ptr: {6:?}, _delta_count: {7:?}) = {8:?}\n" , _dest_ptr , _dest_len , _code_hash_ptr , _code_hash_len , _rent_allowance_ptr , _rent_allowance_len , _delta_ptr , _delta_count , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _dest_ptr : u32 , _code_hash_ptr : u32 , _rent_allowance_ptr : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_restore_to(_dest_ptr: {0:?}, _code_hash_ptr: {1:?}, _rent_allowance_ptr: {2:?}, _delta_ptr: {3:?}, _delta_count: {4:?}) = {5:?}\n" , _dest_ptr , _code_hash_ptr , _rent_allowance_ptr , _delta_ptr , _delta_count , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_set_rent_allowance(_value_ptr: {0:?}, _value_len: {1:?}) = {2:?}\n" , _value_ptr , _value_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal1" , "seal_set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , _value_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal1::seal_set_rent_allowance(_value_ptr: {0:?}) = {1:?}\n" , _value_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: DebugMessage (0)) ? ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let rent_allowance = < BalanceOf < E :: T > > :: max_value () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & rent_allowance , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_rent_allowance(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: Balance) ? ; let rent_allowance = < BalanceOf < E :: T > > :: max_value () . encode () ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & rent_allowance , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_deposit_event" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , topics_ptr : u32 , topics_len : u32 , data_ptr : u32 , data_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let num_topic = topics_len . checked_div (sp_std :: mem :: size_of :: < TopicOf < E :: T > > () as u32) . ok_or ("Zero sized topics are not allowed") ? ; ctx . charge_gas (RuntimeCosts :: DepositEvent { num_topic , len : data_len , }) ? ; if data_len > ctx . ext . max_value_size () { return Err (Error :: < E :: T > :: ValueTooLarge . into ()) } let topics : Vec < TopicOf < < E as Ext > :: T > > = match topics_len { 0 => Vec :: new () , _ => ctx . read_sandbox_memory_as_unbounded (memory , topics_ptr , topics_len) ? , } ; if topics . len () > ctx . ext . schedule () . limits . event_topics as usize { return Err (Error :: < E :: T > :: TooManyTopics . into ()) } let event_data = ctx . read_sandbox_memory (memory , data_ptr , data_len) ? ; ctx . ext . deposit_event (topics , event_data) ; Ok (()) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_deposit_event(topics_ptr: {0:?}, topics_len: {1:?}, data_ptr: {2:?}, data_len: {3:?}) = {4:?}\n" , topics_ptr , topics_len , data_ptr , data_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let num_topic = topics_len . checked_div (sp_std :: mem :: size_of :: < TopicOf < E :: T > > () as u32) . ok_or ("Zero sized topics are not allowed") ? ; ctx . charge_gas (RuntimeCosts :: DepositEvent { num_topic , len : data_len , }) ? ; if data_len > ctx . ext . max_value_size () { return Err (Error :: < E :: T > :: ValueTooLarge . into ()) } let topics : Vec < TopicOf < < E as Ext > :: T > > = match topics_len { 0 => Vec :: new () , _ => ctx . read_sandbox_memory_as_unbounded (memory , topics_ptr , topics_len) ? , } ; if topics . len () > ctx . ext . schedule () . limits . event_topics as usize { return Err (Error :: < E :: T > :: TooManyTopics . into ()) } let event_data = ctx . read_sandbox_memory (memory , data_ptr , data_len) ? ; ctx . ext . deposit_event (topics , event_data) ; Ok (()) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_block_number" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: BlockNumber) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . block_number () . encode () , false , already_charged) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_block_number(out_ptr: {0:?}, out_len_ptr: {1:?}) = {2:?}\n" , out_ptr , out_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: BlockNumber) ? ; Ok (ctx . write_sandbox_output (memory , out_ptr , out_len_ptr , & ctx . ext . block_number () . encode () , false , already_charged) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_hash_sha2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashSha256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , sha2_256 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_hash_sha2_256(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashSha256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , sha2_256 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_hash_keccak_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashKeccak256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , keccak_256 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_hash_keccak_256(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashKeccak256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , keccak_256 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_hash_blake2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashBlake256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_256 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_hash_blake2_256(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashBlake256 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_256 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_hash_blake2_128" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> Result < () , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: HashBlake128 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_128 , input_ptr , input_len , output_ptr) ?) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_hash_blake2_128(input_ptr: {0:?}, input_len: {1:?}, output_ptr: {2:?}) = {3:?}\n" , input_ptr , input_len , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: HashBlake128 (input_len)) ? ; Ok (ctx . compute_hash_on_intermediate_buffer (memory , blake2_128 , input_ptr , input_len , output_ptr) ?) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_call_chain_extension" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , id : u32 , input_ptr : u32 , input_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < u32 , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { use crate :: chain_extension :: { ChainExtension , Environment , RetVal , } ; if ! < E :: T as Config > :: ChainExtension :: enabled () { return Err (Error :: < E :: T > :: NoChainExtension . into ()) } let mut chain_extension = ctx . chain_extension . take () . expect ("Constructor initializes with `Some`. This is the only place where it is set to `None`.\
			It is always reset to `Some` afterwards. qed") ; let env = Environment :: new (ctx , memory , id , input_ptr , input_len , output_ptr , output_len_ptr) ; let ret = match chain_extension . call (env) ? { RetVal :: Converging (val) => Ok (val) , RetVal :: Diverging { flags , data } => Err (TrapReason :: Return (ReturnData { flags : flags . bits () , data , })) , } ; ctx . chain_extension = Some (chain_extension) ; ret } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_call_chain_extension(id: {0:?}, input_ptr: {1:?}, input_len: {2:?}, output_ptr: {3:?}, output_len_ptr: {4:?}) = {5:?}\n" , id , input_ptr , input_len , output_ptr , output_len_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { use crate :: chain_extension :: { ChainExtension , Environment , RetVal , } ; if ! < E :: T as Config > :: ChainExtension :: enabled () { return Err (Error :: < E :: T > :: NoChainExtension . into ()) } let mut chain_extension = ctx . chain_extension . take () . expect ("Constructor initializes with `Some`. This is the only place where it is set to `None`.\
			It is always reset to `Some` afterwards. qed") ; let env = Environment :: new (ctx , memory , id , input_ptr , input_len , output_ptr , output_len_ptr) ; let ret = match chain_extension . call (env) ? { RetVal :: Converging (val) => Ok (val) , RetVal :: Diverging { flags , data } => Err (TrapReason :: Return (ReturnData { flags : flags . bits () , data , })) , } ; ctx . chain_extension = Some (chain_extension) ; ret } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_debug_message" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , str_ptr : u32 , str_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { let str_len = str_len . min (DebugBufferVec :: < E :: T > :: bound () as u32) ; ctx . charge_gas (RuntimeCosts :: DebugMessage (str_len)) ? ; if ctx . ext . append_debug_buffer ("") { let data = ctx . read_sandbox_memory (memory , str_ptr , str_len) ? ; if let Some (msg) = core :: str :: from_utf8 (& data) . ok () { ctx . ext . append_debug_buffer (msg) ; } } Ok (ReturnCode :: Success) } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_debug_message(str_ptr: {0:?}, str_len: {1:?}) = {2:?}\n" , str_ptr , str_len , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { let str_len = str_len . min (DebugBufferVec :: < E :: T > :: bound () as u32) ; ctx . charge_gas (RuntimeCosts :: DebugMessage (str_len)) ? ; if ctx . ext . append_debug_buffer ("") { let data = ctx . read_sandbox_memory (memory , str_ptr , str_len) ? ; if let Some (msg) = core :: str :: from_utf8 (& data) . ok () { ctx . ext . append_debug_buffer (msg) ; } } Ok (ReturnCode :: Success) } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_ecdsa_recover" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , signature_ptr : u32 , message_hash_ptr : u32 , output_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: EcdsaRecovery) ? ; let mut signature : [u8 ; 65] = [0 ; 65] ; ctx . read_sandbox_memory_into_buf (memory , signature_ptr , & mut signature) ? ; let mut message_hash : [u8 ; 32] = [0 ; 32] ; ctx . read_sandbox_memory_into_buf (memory , message_hash_ptr , & mut message_hash) ? ; let result = ctx . ext . ecdsa_recover (& signature , & message_hash) ; match result { Ok (pub_key) => { ctx . write_sandbox_memory (memory , output_ptr , pub_key . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_ecdsa_recover(signature_ptr: {0:?}, message_hash_ptr: {1:?}, output_ptr: {2:?}) = {3:?}\n" , signature_ptr , message_hash_ptr , output_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: EcdsaRecovery) ? ; let mut signature : [u8 ; 65] = [0 ; 65] ; ctx . read_sandbox_memory_into_buf (memory , signature_ptr , & mut signature) ? ; let mut message_hash : [u8 ; 32] = [0 ; 32] ; ctx . read_sandbox_memory_into_buf (memory , message_hash_ptr , & mut message_hash) ? ; let result = ctx . ext . ecdsa_recover (& signature , & message_hash) ; match result { Ok (pub_key) => { ctx . write_sandbox_memory (memory , output_ptr , pub_key . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_set_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , code_hash_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: SetCodeHash) ? ; let code_hash : CodeHash < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , code_hash_ptr) ? ; match ctx . ext . set_code_hash (code_hash) { Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } Ok (()) => Ok (ReturnCode :: Success) , } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_set_code_hash(code_hash_ptr: {0:?}) = {1:?}\n" , code_hash_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: SetCodeHash) ? ; let code_hash : CodeHash < < E as Ext > :: T > = ctx . read_sandbox_memory_as (memory , code_hash_ptr) ? ; match ctx . ext . set_code_hash (code_hash) { Err (err) => { let code = Runtime :: < E > :: err_into_return_code (err) ? ; Ok (code) } Ok (()) => Ok (ReturnCode :: Success) , } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                linker . define ("seal0" , "seal_ecdsa_to_eth_address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < crate :: wasm :: Runtime < E > > , key_ptr : u32 , out_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> Result < ReturnCode , TrapReason > { let (memory , ctx) = __caller__ . data () . memory () . expect ("Memory must be set when setting up host data; qed") . data_and_store_mut (& mut __caller__) ; if { let lvl = :: log :: Level :: Trace ; lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () && :: log :: __private_api_enabled (lvl , "runtime::contracts::strace") } { let result = { ctx . charge_gas (RuntimeCosts :: EcdsaToEthAddress) ? ; let mut compressed_key : [u8 ; 33] = [0 ; 33] ; ctx . read_sandbox_memory_into_buf (memory , key_ptr , & mut compressed_key) ? ; let result = ctx . ext . ecdsa_to_eth_address (& compressed_key) ; match result { Ok (eth_address) => { ctx . write_sandbox_memory (memory , out_ptr , eth_address . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } ; { use sp_std :: fmt :: Write ; let mut w = sp_std :: Writer :: default () ; let _ = (& mut w) . write_fmt (format_args ! ("seal0::seal_ecdsa_to_eth_address(key_ptr: {0:?}, out_ptr: {1:?}) = {2:?}\n" , key_ptr , out_ptr , result)) ; let msg = core :: str :: from_utf8 (& w . inner ()) . unwrap_or_default () ; ctx . ext () . append_debug_buffer (msg) ; } result } else { { ctx . charge_gas (RuntimeCosts :: EcdsaToEthAddress) ? ; let mut compressed_key : [u8 ; 33] = [0 ; 33] ; ctx . read_sandbox_memory_into_buf (memory , key_ptr , & mut compressed_key) ? ; let result = ctx . ext . ecdsa_to_eth_address (& compressed_key) ; match result { Ok (eth_address) => { ctx . write_sandbox_memory (memory , out_ptr , eth_address . as_ref ()) ? ; Ok (ReturnCode :: Success) } Err (_) => Ok (ReturnCode :: EcdsaRecoverFailed) , } } } } ; func () . map_err (| reason | { :: wasmi :: core :: Trap :: from (reason) }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            Ok(())
        }
    }
    
    // 下面这个应该是没有提供默认实现时调用的, 会直接panic "unreachable"
    impl crate::wasm::Environment<()> for Env {
        fn define(
            store: &mut ::wasmi::Store<()>,
            linker: &mut ::wasmi::Linker<()>,
            allow_unstable: AllowUnstableInterface,
            allow_deprecated: AllowDeprecatedInterface,
        ) -> Result<(), ::wasmi::errors::LinkerError> {
            let __allow_unstable__ = match allow_unstable {
                AllowUnstableInterface::Yes => true,
                _ => false,
            };
            let __allow_deprecated__ = match allow_deprecated {
                AllowDeprecatedInterface::Yes => true,
                _ => false,
            };
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "gas" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , amount : u64 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal2" , "set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "take_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "transfer" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 , _account_len : u32 , value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , callee_ptr : u32 , _callee_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , flags : u32 , callee_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal2" , "call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , flags : u32 , callee_ptr : u32 , ref_time_limit : u64 , proof_size_limit : u64 , deposit_ptr : u32 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "delegate_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , flags : u32 , code_hash_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 , _code_hash_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal2" , "instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 , ref_time_limit : u64 , proof_size_limit : u64 , deposit_ptr : u32 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , beneficiary_ptr : u32 , _beneficiary_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , beneficiary_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "input" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_return" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , flags : u32 , data_ptr : u32 , data_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "caller" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "is_contract" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "own_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "caller_is_origin" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "caller_is_root" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "weight_to_fee" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , gas : u64 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "weight_to_fee" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , ref_time_limit : u64 , proof_size_limit : u64 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "gas_left" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "gas_left" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "value_transferred" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "now" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "minimum_balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "tombstone_deposit" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _dest_ptr : u32 , _dest_len : u32 , _code_hash_ptr : u32 , _code_hash_len : u32 , _rent_allowance_ptr : u32 , _rent_allowance_len : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _dest_ptr : u32 , _code_hash_ptr : u32 , _rent_allowance_ptr : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _value_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "deposit_event" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , topics_ptr : u32 , topics_len : u32 , data_ptr : u32 , data_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "block_number" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "hash_sha2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "hash_keccak_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "hash_blake2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "hash_blake2_128" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "call_chain_extension" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , id : u32 , input_ptr : u32 , input_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "debug_message" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , str_ptr : u32 , str_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "call_runtime" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , call_ptr : u32 , call_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "ecdsa_recover" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , signature_ptr : u32 , message_hash_ptr : u32 , output_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "sr25519_verify" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , signature_ptr : u32 , pub_key_ptr : u32 , message_len : u32 , message_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "set_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "ecdsa_to_eth_address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , out_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "reentrance_count" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "account_reentrance_count" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((false || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "instantiation_nonce" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > | -> :: core :: result :: Result < :: core :: primitive :: u64 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u64 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal2" , "seal_set_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 , value_ptr : u32 , value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_clear_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_get_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_contains_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_take_storage" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , key_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_transfer" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 , _account_len : u32 , value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , callee_ptr : u32 , _callee_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , flags : u32 , callee_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_delegate_call" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , flags : u32 , code_hash_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 , _code_hash_len : u32 , gas : u64 , value_ptr : u32 , _value_len : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_instantiate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 , gas : u64 , value_ptr : u32 , input_data_ptr : u32 , input_data_len : u32 , address_ptr : u32 , address_len_ptr : u32 , output_ptr : u32 , output_len_ptr : u32 , salt_ptr : u32 , salt_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , beneficiary_ptr : u32 , _beneficiary_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_terminate" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , beneficiary_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_input" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_caller" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_is_contract" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , account_ptr : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_own_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_caller_is_origin" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_weight_to_fee" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , gas : u64 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_gas_left" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_value_transferred" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_random" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , subject_ptr : u32 , subject_len : u32 , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_now" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_minimum_balance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_tombstone_deposit" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _dest_ptr : u32 , _dest_len : u32 , _code_hash_ptr : u32 , _code_hash_len : u32 , _rent_allowance_ptr : u32 , _rent_allowance_len : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_restore_to" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _dest_ptr : u32 , _code_hash_ptr : u32 , _rent_allowance_ptr : u32 , _delta_ptr : u32 , _delta_count : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _value_ptr : u32 , _value_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal1" , "seal_set_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , _value_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (false || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_rent_allowance" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_deposit_event" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , topics_ptr : u32 , topics_len : u32 , data_ptr : u32 , data_len : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_block_number" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , out_ptr : u32 , out_len_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_hash_sha2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_hash_keccak_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_hash_blake2_256" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_hash_blake2_128" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , input_ptr : u32 , input_len : u32 , output_ptr : u32 | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < () , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_call_chain_extension" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , id : u32 , input_ptr : u32 , input_len : u32 , output_ptr : u32 , output_len_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_debug_message" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , str_ptr : u32 , str_len : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_ecdsa_recover" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , signature_ptr : u32 , message_hash_ptr : u32 , output_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_set_code_hash" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , code_hash_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            if false || ((true || __allow_unstable__) && (true || __allow_deprecated__)) {
                # [allow (unused_variables)] linker . define ("seal0" , "seal_ecdsa_to_eth_address" , :: wasmi :: Func :: wrap (& mut * store , | mut __caller__ : :: wasmi :: Caller < () > , key_ptr : u32 , out_ptr : u32 | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { let mut func = | | -> :: core :: result :: Result < :: core :: primitive :: u32 , :: wasmi :: core :: Trap > { :: core :: panicking :: panic ("internal error: entered unreachable code") } ; func () . map_err (| reason | { reason }) . map (:: core :: convert :: Into :: into) })) ? ;
            }
            Ok(())
        }
    }
}