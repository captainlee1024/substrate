pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(9);
    ///
    ///			The [pallet](https://docs.substrate.io/reference/frame-pallets/#pallets) implementing
    ///			the on-chain logic.
    ///
    pub struct Pallet<T>(PhantomData<T>);
    const _: () = {
        impl<T> core::clone::Clone for Pallet<T> {
            fn clone(&self) -> Self {
                Self(core::clone::Clone::clone(&self.0))
            }
        }
    };
    const _: () = {
        impl<T> core::cmp::Eq for Pallet<T> {}
    };
    const _: () = {
        impl<T> core::cmp::PartialEq for Pallet<T> {
            fn eq(&self, other: &Self) -> bool {
                true && self.0 == other.0
            }
        }
    };
    const _: () = {
        impl<T> core::fmt::Debug for Pallet<T> {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                fmt.debug_tuple("Pallet").field(&self.0).finish()
            }
        }
    };
    ///
    ///			Configuration trait of this pallet.
    ///
    ///			Implement this type for a runtime in order to customize this pallet.
    ///
    pub trait Config: frame_system::Config {
        /// The time implementation used to supply timestamps to contracts through `seal_now`.
        type Time: Time;
        /// The generator used to supply randomness to contracts through `seal_random`.
        ///
        /// # Deprecated
        ///
        /// Codes using the randomness functionality cannot be uploaded. Neither can contracts
        /// be instantiated from existing codes that use this deprecated functionality. It will
        /// be removed eventually. Hence for new `pallet-contracts` deployments it is okay
        /// to supply a dummy implementation for this type (because it is never used).
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        /// The currency in which fees are paid and contract balances are held.
        type Currency: ReservableCurrency<Self::AccountId>
        + Inspect<Self::AccountId, Balance = BalanceOf<Self>>;
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The overarching call type.
        type RuntimeCall: Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
        + GetDispatchInfo
        + codec::Decode
        + IsType<<Self as frame_system::Config>::RuntimeCall>;
        /// Filter that is applied to calls dispatched by contracts.
        ///
        /// Use this filter to control which dispatchables are callable by contracts.
        /// This is applied in **addition** to [`frame_system::Config::BaseCallFilter`].
        /// It is recommended to treat this as a whitelist.
        ///
        /// # Stability
        ///
        /// The runtime **must** make sure that all dispatchables that are callable by
        /// contracts remain stable. In addition [`Self::RuntimeCall`] itself must remain stable.
        /// This means that no existing variants are allowed to switch their positions.
        ///
        /// # Note
        ///
        /// Note that dispatchables that are called via contracts do not spawn their
        /// own wasm instance for each call (as opposed to when called via a transaction).
        /// Therefore please make sure to be restrictive about which dispatchables are allowed
        /// in order to not introduce a new DoS vector like memory allocation patterns that can
        /// be exploited to drive the runtime into a panic.
        type CallFilter: Contains<<Self as frame_system::Config>::RuntimeCall>;
        /// Used to answer contracts' queries regarding the current weight price. This is **not**
        /// used to calculate the actual fee and is only for informational purposes.
        type WeightPrice: Convert<Weight, BalanceOf<Self>>;
        /// Describes the weights of the dispatchables of this module and is also used to
        /// construct a default cost schedule.
        type WeightInfo: WeightInfo;
        /// Type that allows the runtime authors to add new host functions for a contract to call.
        type ChainExtension: chain_extension::ChainExtension<Self> + Default;
        /// Cost schedule and limits.
        type Schedule: Get<Schedule<Self>>;
        /// The type of the call stack determines the maximum nesting depth of contract calls.
        ///
        /// The allowed depth is `CallStack::size() + 1`.
        /// Therefore a size of `0` means that a contract cannot use call or instantiate.
        /// In other words only the origin called "root contract" is allowed to execute then.
        ///
        /// This setting along with [`MaxCodeLen`](#associatedtype.MaxCodeLen) directly affects
        /// memory usage of your runtime.
        type CallStack: Array<Item = Frame<Self>>;
        /// The amount of balance a caller has to pay for each byte of storage.
        ///
        /// # Note
        ///
        /// Changing this value for an existing chain might need a storage migration.
        type DepositPerByte: Get<BalanceOf<Self>>;
        /// Fallback value to limit the storage deposit if it's not being set by the caller.
        type DefaultDepositLimit: Get<BalanceOf<Self>>;
        /// The amount of balance a caller has to pay for each storage item.
        ///
        /// # Note
        ///
        /// Changing this value for an existing chain might need a storage migration.
        type DepositPerItem: Get<BalanceOf<Self>>;
        /// The address generator used to generate the addresses of contracts.
        type AddressGenerator: AddressGenerator<Self>;
        /// The maximum length of a contract code in bytes. This limit applies to the instrumented
        /// version of the code. Therefore `instantiate_with_code` can fail even when supplying
        /// a wasm binary below this maximum size.
        ///
        /// The value should be chosen carefully taking into the account the overall memory limit
        /// your runtime has, as well as the [maximum allowed callstack
        /// depth](#associatedtype.CallStack). Look into the `integrity_test()` for some insights.
        type MaxCodeLen: Get<u32>;
        /// The maximum allowable length in bytes for storage keys.
        type MaxStorageKeyLen: Get<u32>;
        /// Make contract callable functions marked as `#[unstable]` available.
        ///
        /// Contracts that use `#[unstable]` functions won't be able to be uploaded unless
        /// this is set to `true`. This is only meant for testnets and dev nodes in order to
        /// experiment with new features.
        ///
        /// # Warning
        ///
        /// Do **not** set to `true` on productions chains.
        type UnsafeUnstableInterface: Get<bool>;
        /// The maximum length of the debug buffer in bytes.
        type MaxDebugBufferLen: Get<u32>;
    }
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_idle(_block: T::BlockNumber, remaining_weight: Weight) -> Weight {
            ContractInfo::<T>::process_deletion_queue_batch(remaining_weight)
                .saturating_add(T::WeightInfo::on_process_deletion_queue_batch())
        }
        fn integrity_test() {
            let max_runtime_mem: u32 = T::Schedule::get().limits.runtime_memory;
            const MAX_STACK_SIZE: u32 = 1024 * 1024;
            let max_heap_size = T::Schedule::get().limits.max_memory_size();
            let max_call_depth = u32::try_from(T::CallStack::size().saturating_add(1))
                .expect("CallStack size is too big");
            let code_len_limit = max_runtime_mem
                .saturating_div(2)
                .saturating_div(max_call_depth)
                .saturating_sub(max_heap_size)
                .saturating_sub(MAX_STACK_SIZE)
                .saturating_div(18 * 4);
            if !(T::MaxCodeLen::get() < code_len_limit) {
                :: core :: panicking :: panic_fmt (format_args ! ("Given `CallStack` height {0:?}, `MaxCodeLen` should be set less than {1:?} (current value is {2:?}), to avoid possible runtime oom issues." , max_call_depth , code_len_limit , T :: MaxCodeLen :: get ()))
            };
            const MIN_DEBUG_BUF_SIZE: u32 = 256;
            if !(T::MaxDebugBufferLen::get() > MIN_DEBUG_BUF_SIZE) {
                ::core::panicking::panic_fmt(format_args!(
                    "Debug buffer should have minimum size of {0} (current setting is {1})",
                    MIN_DEBUG_BUF_SIZE,
                    T::MaxDebugBufferLen::get()
                ))
            }
        }
    }
    impl<T: Config> Pallet<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        /// Deprecated version if [`Self::call`] for use in an in-storage `Call`.
        #[allow(deprecated)]
        #[deprecated(note = "1D weight is used in this extrinsic, please migrate to `call`")]
        pub fn call_old_weight(
            origin: OriginFor<T>,
            dest: AccountIdLookupOf<T>,
            value: BalanceOf<T>,
            gas_limit: OldWeight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            data: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                Self::call(
                    origin,
                    dest,
                    value,
                    <Pallet<T>>::compat_weight_limit(gas_limit),
                    storage_deposit_limit,
                    data,
                )
            })
        }
        /// Deprecated version if [`Self::instantiate_with_code`] for use in an in-storage `Call`.
        #[allow(deprecated)]
        #[deprecated(
        note = "1D weight is used in this extrinsic, please migrate to `instantiate_with_code`"
        )]
        pub fn instantiate_with_code_old_weight(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            gas_limit: OldWeight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                Self::instantiate_with_code(
                    origin,
                    value,
                    <Pallet<T>>::compat_weight_limit(gas_limit),
                    storage_deposit_limit,
                    code,
                    data,
                    salt,
                )
            })
        }
        /// Deprecated version if [`Self::instantiate`] for use in an in-storage `Call`.
        #[allow(deprecated)]
        #[deprecated(note = "1D weight is used in this extrinsic, please migrate to `instantiate`")]
        pub fn instantiate_old_weight(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            gas_limit: OldWeight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                Self::instantiate(
                    origin,
                    value,
                    <Pallet<T>>::compat_weight_limit(gas_limit),
                    storage_deposit_limit,
                    code_hash,
                    data,
                    salt,
                )
            })
        }
        /// Upload new `code` without instantiating a contract from it.
        ///
        /// If the code does not already exist a deposit is reserved from the caller
        /// and unreserved only when [`Self::remove_code`] is called. The size of the reserve
        /// depends on the instrumented size of the the supplied `code`.
        ///
        /// If the code already exists in storage it will still return `Ok` and upgrades
        /// the in storage version to the current
        /// [`InstructionWeights::version`](InstructionWeights).
        ///
        /// - `determinism`: If this is set to any other value but [`Determinism::Enforced`] then
        ///   the only way to use this code is to delegate call into it from an offchain execution.
        ///   Set to [`Determinism::Enforced`] if in doubt.
        ///
        /// # Note
        ///
        /// Anyone can instantiate a contract from any uploaded code and thus prevent its removal.
        /// To avoid this situation a constructor could employ access control so that it can
        /// only be instantiated by permissioned entities. The same is true when uploading
        /// through [`Self::instantiate_with_code`].
        pub fn upload_code(
            origin: OriginFor<T>,
            code: Vec<u8>,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            determinism: Determinism,
        ) -> DispatchResult {
            frame_support::storage::with_storage_layer(|| {
                let origin = ensure_signed(origin)?;
                Self::bare_upload_code(
                    origin,
                    code,
                    storage_deposit_limit.map(Into::into),
                    determinism,
                )
                    .map(|_| ())
            })
        }
        /// Remove the code stored under `code_hash` and refund the deposit to its owner.
        ///
        /// A code can only be removed by its original uploader (its owner) and only if it is
        /// not used by any contract.
        pub fn remove_code(
            origin: OriginFor<T>,
            code_hash: CodeHash<T>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                let origin = ensure_signed(origin)?;
                <PrefabWasmModule<T>>::remove(&origin, code_hash)?;
                Ok(Pays::No.into())
            })
        }
        /// Privileged function that changes the code of an existing contract.
        ///
        /// This takes care of updating refcounts and all other necessary operations. Returns
        /// an error if either the `code_hash` or `dest` do not exist.
        ///
        /// # Note
        ///
        /// This does **not** change the address of the contract in question. This means
        /// that the contract address is no longer derived from its code hash after calling
        /// this dispatchable.
        pub fn set_code(
            origin: OriginFor<T>,
            dest: AccountIdLookupOf<T>,
            code_hash: CodeHash<T>,
        ) -> DispatchResult {
            frame_support::storage::with_storage_layer(|| {
                ensure_root(origin)?;
                let dest = T::Lookup::lookup(dest)?;
                <ContractInfoOf<T>>::try_mutate(&dest, |contract| {
                    let contract = if let Some(contract) = contract {
                        contract
                    } else {
                        return Err(<Error<T>>::ContractNotFound.into());
                    };
                    <PrefabWasmModule<T>>::add_user(code_hash)?;
                    <PrefabWasmModule<T>>::remove_user(contract.code_hash);
                    Self::deposit_event(
                        <[_]>::into_vec(
                            #[rustc_box]
                                ::alloc::boxed::Box::new([
                                T::Hashing::hash_of(&dest),
                                code_hash,
                                contract.code_hash,
                            ]),
                        ),
                        Event::ContractCodeUpdated {
                            contract: dest.clone(),
                            new_code_hash: code_hash,
                            old_code_hash: contract.code_hash,
                        },
                    );
                    contract.code_hash = code_hash;
                    Ok(())
                })
            })
        }
        /// Makes a call to an account, optionally transferring some balance.
        ///
        /// # Parameters
        ///
        /// * `dest`: Address of the contract to call.
        /// * `value`: The balance to transfer from the `origin` to `dest`.
        /// * `gas_limit`: The gas limit enforced when executing the constructor.
        /// * `storage_deposit_limit`: The maximum amount of balance that can be charged from the
        ///   caller to pay for the storage consumed.
        /// * `data`: The input data to pass to the contract.
        ///
        /// * If the account is a smart-contract account, the associated code will be
        /// executed and any value will be transferred.
        /// * If the account is a regular account, any value will be transferred.
        /// * If no account exists and the call value is not less than `existential_deposit`,
        /// a regular account will be created and any value will be transferred.
        pub fn call(
            origin: OriginFor<T>,
            dest: AccountIdLookupOf<T>,
            value: BalanceOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            data: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                let common = CommonInput {
                    origin: Origin::from_runtime_origin(origin)?,
                    value,
                    data,
                    gas_limit: gas_limit.into(),
                    storage_deposit_limit: storage_deposit_limit.map(Into::into),
                    debug_message: None,
                };
                let dest = T::Lookup::lookup(dest)?;
                let mut output = CallInput::<T> {
                    dest,
                    determinism: Determinism::Enforced,
                }
                    .run_guarded(common);
                if let Ok(retval) = &output.result {
                    if retval.did_revert() {
                        output.result = Err(<Error<T>>::ContractReverted.into());
                    }
                }
                output
                    .gas_meter
                    .into_dispatch_result(output.result, T::WeightInfo::call())
            })
        }
        /// Instantiates a new contract from the supplied `code` optionally transferring
        /// some balance.
        ///
        /// This dispatchable has the same effect as calling [`Self::upload_code`] +
        /// [`Self::instantiate`]. Bundling them together provides efficiency gains. Please
        /// also check the documentation of [`Self::upload_code`].
        ///
        /// # Parameters
        ///
        /// * `value`: The balance to transfer from the `origin` to the newly created contract.
        /// * `gas_limit`: The gas limit enforced when executing the constructor.
        /// * `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved
        ///   from the caller to pay for the storage consumed.
        /// * `code`: The contract code to deploy in raw bytes.
        /// * `data`: The input data to pass to the contract constructor.
        /// * `salt`: Used for the address derivation. See [`Pallet::contract_address`].
        ///
        /// Instantiation is executed as follows:
        ///
        /// - The supplied `code` is instrumented, deployed, and a `code_hash` is created for that
        ///   code.
        /// - If the `code_hash` already exists on the chain the underlying `code` will be shared.
        /// - The destination address is computed based on the sender, code_hash and the salt.
        /// - The smart-contract account is created at the computed address.
        /// - The `value` is transferred to the new account.
        /// - The `deploy` function is executed in the context of the newly-created account.
        pub fn instantiate_with_code(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                let code_len = code.len() as u32;
                let data_len = data.len() as u32;
                let salt_len = salt.len() as u32;
                let common = CommonInput {
                    origin: Origin::from_runtime_origin(origin)?,
                    value,
                    data,
                    gas_limit,
                    storage_deposit_limit: storage_deposit_limit.map(Into::into),
                    debug_message: None,
                };
                let mut output = InstantiateInput::<T> {
                    code: Code::Upload(code),
                    salt,
                }
                    .run_guarded(common);
                if let Ok(retval) = &output.result {
                    if retval.1.did_revert() {
                        output.result = Err(<Error<T>>::ContractReverted.into());
                    }
                }
                output.gas_meter.into_dispatch_result(
                    output.result.map(|(_address, result)| result),
                    T::WeightInfo::instantiate_with_code(code_len, data_len, salt_len),
                )
            })
        }
        /// Instantiates a contract from a previously deployed wasm binary.
        ///
        /// This function is identical to [`Self::instantiate_with_code`] but without the
        /// code deployment step. Instead, the `code_hash` of an on-chain deployed wasm binary
        /// must be supplied.
        pub fn instantiate(
            origin: OriginFor<T>,
            value: BalanceOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            frame_support::storage::with_storage_layer(|| {
                let data_len = data.len() as u32;
                let salt_len = salt.len() as u32;
                let common = CommonInput {
                    origin: Origin::from_runtime_origin(origin)?,
                    value,
                    data,
                    gas_limit,
                    storage_deposit_limit: storage_deposit_limit.map(Into::into),
                    debug_message: None,
                };
                let mut output = InstantiateInput::<T> {
                    code: Code::Existing(code_hash),
                    salt,
                }
                    .run_guarded(common);
                if let Ok(retval) = &output.result {
                    if retval.1.did_revert() {
                        output.result = Err(<Error<T>>::ContractReverted.into());
                    }
                }
                output.gas_meter.into_dispatch_result(
                    output.result.map(|(_address, output)| output),
                    T::WeightInfo::instantiate(data_len, salt_len),
                )
            })
        }
    }
    ///
    ///			The [event](https://docs.substrate.io/main-docs/build/events-errors/) emitted
    ///			by this pallet.
    ///
    #[scale_info(skip_type_params(T), capture_docs = "always")]
    pub enum Event<T: Config> {
        /// Contract deployed by address at the specified address.
        Instantiated {
            deployer: T::AccountId,
            contract: T::AccountId,
        },
        /// Contract has been removed.
        ///
        /// # Note
        ///
        /// The only way for a contract to be removed and emitting this event is by calling
        /// `seal_terminate`.
        Terminated {
            /// The contract that was terminated.
            contract: T::AccountId,
            /// The account that received the contracts remaining balance
            beneficiary: T::AccountId,
        },
        /// Code with the specified hash has been stored.
        CodeStored { code_hash: T::Hash },
        /// A custom event emitted by the contract.
        ContractEmitted {
            /// The contract that emitted the event.
            contract: T::AccountId,
            /// Data supplied by the contract. Metadata generated during contract compilation
            /// is needed to decode it.
            data: Vec<u8>,
        },
        /// A code with the specified hash was removed.
        CodeRemoved { code_hash: T::Hash },
        /// A contract's code was updated.
        ContractCodeUpdated {
            /// The contract that has been updated.
            contract: T::AccountId,
            /// New code hash that was set for the contract.
            new_code_hash: T::Hash,
            /// Previous code hash of the contract.
            old_code_hash: T::Hash,
        },
        /// A contract was called either by a plain account or another contract.
        ///
        /// # Note
        ///
        /// Please keep in mind that like all events this is only emitted for successful
        /// calls. This is because on failure all storage changes including events are
        /// rolled back.
        Called {
            /// The caller of the `contract`.
            caller: Origin<T>,
            /// The contract that was called.
            contract: T::AccountId,
        },
        /// A contract delegate called a code hash.
        ///
        /// # Note
        ///
        /// Please keep in mind that like all events this is only emitted for successful
        /// calls. This is because on failure all storage changes including events are
        /// rolled back.
        DelegateCalled {
            /// The contract that performed the delegate call and hence in whose context
            /// the `code_hash` is executed.
            contract: T::AccountId,
            /// The code hash that was delegate called.
            code_hash: CodeHash<T>,
        },
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(
            frame_support::sp_std::marker::PhantomData<(T)>,
            frame_support::Never,
        ),
    }
    const _: () = {
        impl<T: Config> core::clone::Clone for Event<T> {
            fn clone(&self) -> Self {
                match self {
                    Self::Instantiated {
                        ref deployer,
                        ref contract,
                    } => Self::Instantiated {
                        deployer: core::clone::Clone::clone(deployer),
                        contract: core::clone::Clone::clone(contract),
                    },
                    Self::Terminated {
                        ref contract,
                        ref beneficiary,
                    } => Self::Terminated {
                        contract: core::clone::Clone::clone(contract),
                        beneficiary: core::clone::Clone::clone(beneficiary),
                    },
                    Self::CodeStored { ref code_hash } => Self::CodeStored {
                        code_hash: core::clone::Clone::clone(code_hash),
                    },
                    Self::ContractEmitted {
                        ref contract,
                        ref data,
                    } => Self::ContractEmitted {
                        contract: core::clone::Clone::clone(contract),
                        data: core::clone::Clone::clone(data),
                    },
                    Self::CodeRemoved { ref code_hash } => Self::CodeRemoved {
                        code_hash: core::clone::Clone::clone(code_hash),
                    },
                    Self::ContractCodeUpdated {
                        ref contract,
                        ref new_code_hash,
                        ref old_code_hash,
                    } => Self::ContractCodeUpdated {
                        contract: core::clone::Clone::clone(contract),
                        new_code_hash: core::clone::Clone::clone(new_code_hash),
                        old_code_hash: core::clone::Clone::clone(old_code_hash),
                    },
                    Self::Called {
                        ref caller,
                        ref contract,
                    } => Self::Called {
                        caller: core::clone::Clone::clone(caller),
                        contract: core::clone::Clone::clone(contract),
                    },
                    Self::DelegateCalled {
                        ref contract,
                        ref code_hash,
                    } => Self::DelegateCalled {
                        contract: core::clone::Clone::clone(contract),
                        code_hash: core::clone::Clone::clone(code_hash),
                    },
                    Self::__Ignore(ref _0, ref _1) => {
                        Self::__Ignore(core::clone::Clone::clone(_0), core::clone::Clone::clone(_1))
                    }
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> core::cmp::Eq for Event<T> {}
    };
    const _: () = {
        impl<T: Config> core::cmp::PartialEq for Event<T> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (
                        Self::Instantiated { deployer, contract },
                        Self::Instantiated {
                            deployer: _0,
                            contract: _1,
                        },
                    ) => true && deployer == _0 && contract == _1,
                    (
                        Self::Terminated {
                            contract,
                            beneficiary,
                        },
                        Self::Terminated {
                            contract: _0,
                            beneficiary: _1,
                        },
                    ) => true && contract == _0 && beneficiary == _1,
                    (Self::CodeStored { code_hash }, Self::CodeStored { code_hash: _0 }) => {
                        true && code_hash == _0
                    }
                    (
                        Self::ContractEmitted { contract, data },
                        Self::ContractEmitted {
                            contract: _0,
                            data: _1,
                        },
                    ) => true && contract == _0 && data == _1,
                    (Self::CodeRemoved { code_hash }, Self::CodeRemoved { code_hash: _0 }) => {
                        true && code_hash == _0
                    }
                    (
                        Self::ContractCodeUpdated {
                            contract,
                            new_code_hash,
                            old_code_hash,
                        },
                        Self::ContractCodeUpdated {
                            contract: _0,
                            new_code_hash: _1,
                            old_code_hash: _2,
                        },
                    ) => true && contract == _0 && new_code_hash == _1 && old_code_hash == _2,
                    (
                        Self::Called { caller, contract },
                        Self::Called {
                            caller: _0,
                            contract: _1,
                        },
                    ) => true && caller == _0 && contract == _1,
                    (
                        Self::DelegateCalled {
                            contract,
                            code_hash,
                        },
                        Self::DelegateCalled {
                            contract: _0,
                            code_hash: _1,
                        },
                    ) => true && contract == _0 && code_hash == _1,
                    (Self::__Ignore(_0, _1), Self::__Ignore(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (Self::Instantiated { .. }, Self::Terminated { .. }) => false,
                    (Self::Instantiated { .. }, Self::CodeStored { .. }) => false,
                    (Self::Instantiated { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::Instantiated { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::Instantiated { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::Instantiated { .. }, Self::Called { .. }) => false,
                    (Self::Instantiated { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::Instantiated { .. }, Self::__Ignore { .. }) => false,
                    (Self::Terminated { .. }, Self::Instantiated { .. }) => false,
                    (Self::Terminated { .. }, Self::CodeStored { .. }) => false,
                    (Self::Terminated { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::Terminated { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::Terminated { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::Terminated { .. }, Self::Called { .. }) => false,
                    (Self::Terminated { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::Terminated { .. }, Self::__Ignore { .. }) => false,
                    (Self::CodeStored { .. }, Self::Instantiated { .. }) => false,
                    (Self::CodeStored { .. }, Self::Terminated { .. }) => false,
                    (Self::CodeStored { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::CodeStored { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::CodeStored { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::CodeStored { .. }, Self::Called { .. }) => false,
                    (Self::CodeStored { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::CodeStored { .. }, Self::__Ignore { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::Instantiated { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::Terminated { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::CodeStored { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::Called { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::ContractEmitted { .. }, Self::__Ignore { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::Instantiated { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::Terminated { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::CodeStored { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::Called { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::CodeRemoved { .. }, Self::__Ignore { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::Instantiated { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::Terminated { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::CodeStored { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::Called { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::ContractCodeUpdated { .. }, Self::__Ignore { .. }) => false,
                    (Self::Called { .. }, Self::Instantiated { .. }) => false,
                    (Self::Called { .. }, Self::Terminated { .. }) => false,
                    (Self::Called { .. }, Self::CodeStored { .. }) => false,
                    (Self::Called { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::Called { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::Called { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::Called { .. }, Self::DelegateCalled { .. }) => false,
                    (Self::Called { .. }, Self::__Ignore { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::Instantiated { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::Terminated { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::CodeStored { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::Called { .. }) => false,
                    (Self::DelegateCalled { .. }, Self::__Ignore { .. }) => false,
                    (Self::__Ignore { .. }, Self::Instantiated { .. }) => false,
                    (Self::__Ignore { .. }, Self::Terminated { .. }) => false,
                    (Self::__Ignore { .. }, Self::CodeStored { .. }) => false,
                    (Self::__Ignore { .. }, Self::ContractEmitted { .. }) => false,
                    (Self::__Ignore { .. }, Self::CodeRemoved { .. }) => false,
                    (Self::__Ignore { .. }, Self::ContractCodeUpdated { .. }) => false,
                    (Self::__Ignore { .. }, Self::Called { .. }) => false,
                    (Self::__Ignore { .. }, Self::DelegateCalled { .. }) => false,
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> core::fmt::Debug for Event<T> {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                match *self {
                    Self::Instantiated {
                        ref deployer,
                        ref contract,
                    } => fmt
                        .debug_struct("Event::Instantiated")
                        .field("deployer", &deployer)
                        .field("contract", &contract)
                        .finish(),
                    Self::Terminated {
                        ref contract,
                        ref beneficiary,
                    } => fmt
                        .debug_struct("Event::Terminated")
                        .field("contract", &contract)
                        .field("beneficiary", &beneficiary)
                        .finish(),
                    Self::CodeStored { ref code_hash } => fmt
                        .debug_struct("Event::CodeStored")
                        .field("code_hash", &code_hash)
                        .finish(),
                    Self::ContractEmitted {
                        ref contract,
                        ref data,
                    } => fmt
                        .debug_struct("Event::ContractEmitted")
                        .field("contract", &contract)
                        .field("data", &data)
                        .finish(),
                    Self::CodeRemoved { ref code_hash } => fmt
                        .debug_struct("Event::CodeRemoved")
                        .field("code_hash", &code_hash)
                        .finish(),
                    Self::ContractCodeUpdated {
                        ref contract,
                        ref new_code_hash,
                        ref old_code_hash,
                    } => fmt
                        .debug_struct("Event::ContractCodeUpdated")
                        .field("contract", &contract)
                        .field("new_code_hash", &new_code_hash)
                        .field("old_code_hash", &old_code_hash)
                        .finish(),
                    Self::Called {
                        ref caller,
                        ref contract,
                    } => fmt
                        .debug_struct("Event::Called")
                        .field("caller", &caller)
                        .field("contract", &contract)
                        .finish(),
                    Self::DelegateCalled {
                        ref contract,
                        ref code_hash,
                    } => fmt
                        .debug_struct("Event::DelegateCalled")
                        .field("contract", &contract)
                        .field("code_hash", &code_hash)
                        .finish(),
                    Self::__Ignore(ref _0, ref _1) => fmt
                        .debug_tuple("Event::__Ignore")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                }
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Encode for Event<T>
            where
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                Origin<T>: ::codec::Encode,
                Origin<T>: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                CodeHash<T>: ::codec::Encode,
                CodeHash<T>: ::codec::Encode,
        {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    Event::Instantiated {
                        ref deployer,
                        ref contract,
                    } => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(deployer, __codec_dest_edqy);
                        ::codec::Encode::encode_to(contract, __codec_dest_edqy);
                    }
                    Event::Terminated {
                        ref contract,
                        ref beneficiary,
                    } => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(contract, __codec_dest_edqy);
                        ::codec::Encode::encode_to(beneficiary, __codec_dest_edqy);
                    }
                    Event::CodeStored { ref code_hash } => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(code_hash, __codec_dest_edqy);
                    }
                    Event::ContractEmitted {
                        ref contract,
                        ref data,
                    } => {
                        __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(contract, __codec_dest_edqy);
                        ::codec::Encode::encode_to(data, __codec_dest_edqy);
                    }
                    Event::CodeRemoved { ref code_hash } => {
                        __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(code_hash, __codec_dest_edqy);
                    }
                    Event::ContractCodeUpdated {
                        ref contract,
                        ref new_code_hash,
                        ref old_code_hash,
                    } => {
                        __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(contract, __codec_dest_edqy);
                        ::codec::Encode::encode_to(new_code_hash, __codec_dest_edqy);
                        ::codec::Encode::encode_to(old_code_hash, __codec_dest_edqy);
                    }
                    Event::Called {
                        ref caller,
                        ref contract,
                    } => {
                        __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(caller, __codec_dest_edqy);
                        ::codec::Encode::encode_to(contract, __codec_dest_edqy);
                    }
                    Event::DelegateCalled {
                        ref contract,
                        ref code_hash,
                    } => {
                        __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(contract, __codec_dest_edqy);
                        ::codec::Encode::encode_to(code_hash, __codec_dest_edqy);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl<T: Config> ::codec::EncodeLike for Event<T>
            where
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                T::Hash: ::codec::Encode,
                Origin<T>: ::codec::Encode,
                Origin<T>: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                T::AccountId: ::codec::Encode,
                CodeHash<T>: ::codec::Encode,
                CodeHash<T>: ::codec::Encode,
        {
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Decode for Event<T>
            where
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::Hash: ::codec::Decode,
                T::Hash: ::codec::Decode,
                Origin<T>: ::codec::Decode,
                Origin<T>: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                T::AccountId: ::codec::Decode,
                CodeHash<T>: ::codec::Decode,
                CodeHash<T>: ::codec::Decode,
        {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| e.chain("Could not decode `Event`, failed to read variant byte"))?
                {
                    __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::Instantiated {
                            deployer: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::Instantiated::deployer`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            contract: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::Instantiated::contract`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::Terminated {
                            contract: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::Terminated::contract`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            beneficiary: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::Terminated::beneficiary`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::CodeStored {
                            code_hash: {
                                let __codec_res_edqy =
                                    <T::Hash as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::CodeStored::code_hash`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::ContractEmitted {
                            contract: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::ContractEmitted::contract`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            data: {
                                let __codec_res_edqy =
                                    <Vec<u8> as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::ContractEmitted::data`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 4usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::CodeRemoved {
                            code_hash: {
                                let __codec_res_edqy =
                                    <T::Hash as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::CodeRemoved::code_hash`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 5usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::ContractCodeUpdated {
                            contract: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy { :: core :: result :: Result :: Err (e) => return :: core :: result :: Result :: Err (e . chain ("Could not decode `Event::ContractCodeUpdated::contract`")) , :: core :: result :: Result :: Ok (__codec_res_edqy) => __codec_res_edqy , }
                            },
                            new_code_hash: {
                                let __codec_res_edqy =
                                    <T::Hash as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy { :: core :: result :: Result :: Err (e) => return :: core :: result :: Result :: Err (e . chain ("Could not decode `Event::ContractCodeUpdated::new_code_hash`")) , :: core :: result :: Result :: Ok (__codec_res_edqy) => __codec_res_edqy , }
                            },
                            old_code_hash: {
                                let __codec_res_edqy =
                                    <T::Hash as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy { :: core :: result :: Result :: Err (e) => return :: core :: result :: Result :: Err (e . chain ("Could not decode `Event::ContractCodeUpdated::old_code_hash`")) , :: core :: result :: Result :: Ok (__codec_res_edqy) => __codec_res_edqy , }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 6usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::Called {
                            caller: {
                                let __codec_res_edqy =
                                    <Origin<T> as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `Event::Called::caller`"),
                                        )
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            contract: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `Event::Called::contract`"),
                                        )
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    __codec_x_edqy if __codec_x_edqy == 7usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Event::<T>::DelegateCalled {
                            contract: {
                                let __codec_res_edqy =
                                    <T::AccountId as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::DelegateCalled::contract`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            code_hash: {
                                let __codec_res_edqy =
                                    <CodeHash<T> as ::codec::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `Event::DelegateCalled::code_hash`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `Event`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T: Config> ::scale_info::TypeInfo for Event<T>
            where
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::Hash: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::Hash: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::Hash: ::scale_info::TypeInfo + 'static,
                T::Hash: ::scale_info::TypeInfo + 'static,
                Origin<T>: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                T::AccountId: ::scale_info::TypeInfo + 'static,
                CodeHash<T>: ::scale_info::TypeInfo + 'static,
                frame_support::sp_std::marker::PhantomData<(T)>: ::scale_info::TypeInfo + 'static,
                T: Config + 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("Event" , "pallet_contracts::pallet")) . type_params (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([:: scale_info :: TypeParameter :: new ("T" , :: core :: option :: Option :: None)]))) . docs_always (& ["\n\t\t\tThe [event](https://docs.substrate.io/main-docs/build/events-errors/) emitted\n\t\t\tby this pallet.\n\t\t\t"]) . variant (:: scale_info :: build :: Variants :: new () . variant ("Instantiated" , | v | v . index (0usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: AccountId > () . name ("deployer") . type_name ("T::AccountId")) . field (| f | f . ty :: < T :: AccountId > () . name ("contract") . type_name ("T::AccountId"))) . docs_always (& ["Contract deployed by address at the specified address."])) . variant ("Terminated" , | v | v . index (1usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: AccountId > () . name ("contract") . type_name ("T::AccountId") . docs_always (& ["The contract that was terminated."])) . field (| f | f . ty :: < T :: AccountId > () . name ("beneficiary") . type_name ("T::AccountId") . docs_always (& ["The account that received the contracts remaining balance"]))) . docs_always (& ["Contract has been removed." , "" , "# Note" , "" , "The only way for a contract to be removed and emitting this event is by calling" , "`seal_terminate`."])) . variant ("CodeStored" , | v | v . index (2usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: Hash > () . name ("code_hash") . type_name ("T::Hash"))) . docs_always (& ["Code with the specified hash has been stored."])) . variant ("ContractEmitted" , | v | v . index (3usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: AccountId > () . name ("contract") . type_name ("T::AccountId") . docs_always (& ["The contract that emitted the event."])) . field (| f | f . ty :: < Vec < u8 > > () . name ("data") . type_name ("Vec<u8>") . docs_always (& ["Data supplied by the contract. Metadata generated during contract compilation" , "is needed to decode it."]))) . docs_always (& ["A custom event emitted by the contract."])) . variant ("CodeRemoved" , | v | v . index (4usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: Hash > () . name ("code_hash") . type_name ("T::Hash"))) . docs_always (& ["A code with the specified hash was removed."])) . variant ("ContractCodeUpdated" , | v | v . index (5usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: AccountId > () . name ("contract") . type_name ("T::AccountId") . docs_always (& ["The contract that has been updated."])) . field (| f | f . ty :: < T :: Hash > () . name ("new_code_hash") . type_name ("T::Hash") . docs_always (& ["New code hash that was set for the contract."])) . field (| f | f . ty :: < T :: Hash > () . name ("old_code_hash") . type_name ("T::Hash") . docs_always (& ["Previous code hash of the contract."]))) . docs_always (& ["A contract's code was updated."])) . variant ("Called" , | v | v . index (6usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < Origin < T > > () . name ("caller") . type_name ("Origin<T>") . docs_always (& ["The caller of the `contract`."])) . field (| f | f . ty :: < T :: AccountId > () . name ("contract") . type_name ("T::AccountId") . docs_always (& ["The contract that was called."]))) . docs_always (& ["A contract was called either by a plain account or another contract." , "" , "# Note" , "" , "Please keep in mind that like all events this is only emitted for successful" , "calls. This is because on failure all storage changes including events are" , "rolled back."])) . variant ("DelegateCalled" , | v | v . index (7usize as :: core :: primitive :: u8) . fields (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < T :: AccountId > () . name ("contract") . type_name ("T::AccountId") . docs_always (& ["The contract that performed the delegate call and hence in whose context" , "the `code_hash` is executed."])) . field (| f | f . ty :: < CodeHash < T > > () . name ("code_hash") . type_name ("CodeHash<T>") . docs_always (& ["The code hash that was delegate called."]))) . docs_always (& ["A contract delegate called a code hash." , "" , "# Note" , "" , "Please keep in mind that like all events this is only emitted for successful" , "calls. This is because on failure all storage changes including events are" , "rolled back."])))
            }
        };
    };
    #[scale_info(skip_type_params(T), capture_docs = "always")]
    ///
    ///			Custom [dispatch errors](https://docs.substrate.io/main-docs/build/events-errors/)
    ///			of this pallet.
    ///
    pub enum Error<T> {
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(
            frame_support::sp_std::marker::PhantomData<(T)>,
            frame_support::Never,
        ),
        /// A new schedule must have a greater version than the current one.
        InvalidScheduleVersion,
        /// Invalid combination of flags supplied to `seal_call` or `seal_delegate_call`.
        InvalidCallFlags,
        /// The executed contract exhausted its gas limit.
        OutOfGas,
        /// The output buffer supplied to a contract API call was too small.
        OutputBufferTooSmall,
        /// Performing the requested transfer failed. Probably because there isn't enough
        /// free balance in the sender's account.
        TransferFailed,
        /// Performing a call was denied because the calling depth reached the limit
        /// of what is specified in the schedule.
        MaxCallDepthReached,
        /// No contract was found at the specified address.
        ContractNotFound,
        /// The code supplied to `instantiate_with_code` exceeds the limit specified in the
        /// current schedule.
        CodeTooLarge,
        /// No code could be found at the supplied code hash.
        CodeNotFound,
        /// A buffer outside of sandbox memory was passed to a contract API function.
        OutOfBounds,
        /// Input passed to a contract API function failed to decode as expected type.
        DecodingFailed,
        /// Contract trapped during execution.
        ContractTrapped,
        /// The size defined in `T::MaxValueSize` was exceeded.
        ValueTooLarge,
        /// Termination of a contract is not allowed while the contract is already
        /// on the call stack. Can be triggered by `seal_terminate`.
        TerminatedWhileReentrant,
        /// `seal_call` forwarded this contracts input. It therefore is no longer available.
        InputForwarded,
        /// The subject passed to `seal_random` exceeds the limit.
        RandomSubjectTooLong,
        /// The amount of topics passed to `seal_deposit_events` exceeds the limit.
        TooManyTopics,
        /// The chain does not provide a chain extension. Calling the chain extension results
        /// in this error. Note that this usually  shouldn't happen as deploying such contracts
        /// is rejected.
        NoChainExtension,
        /// A contract with the same AccountId already exists.
        DuplicateContract,
        /// A contract self destructed in its constructor.
        ///
        /// This can be triggered by a call to `seal_terminate`.
        TerminatedInConstructor,
        /// A call tried to invoke a contract that is flagged as non-reentrant.
        /// The only other cause is that a call from a contract into the runtime tried to call back
        /// into `pallet-contracts`. This would make the whole pallet reentrant with regard to
        /// contract code execution which is not supported.
        ReentranceDenied,
        /// Origin doesn't have enough balance to pay the required storage deposits.
        StorageDepositNotEnoughFunds,
        /// More storage was created than allowed by the storage deposit limit.
        StorageDepositLimitExhausted,
        /// Code removal was denied because the code is still in use by at least one contract.
        CodeInUse,
        /// The contract ran to completion but decided to revert its storage changes.
        /// Please note that this error is only returned from extrinsics. When called directly
        /// or via RPC an `Ok` will be returned. In this case the caller needs to inspect the flags
        /// to determine whether a reversion has taken place.
        ContractReverted,
        /// The contract's code was found to be invalid during validation or instrumentation.
        ///
        /// The most likely cause of this is that an API was used which is not supported by the
        /// node. This happens if an older node is used with a new version of ink!. Try updating
        /// your node to the newest available version.
        ///
        /// A more detailed error can be found on the node console if debug messages are enabled
        /// by supplying `-lruntime::contracts=debug`.
        CodeRejected,
        /// An indetermistic code was used in a context where this is not permitted.
        Indeterministic,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T> ::codec::Encode for Error<T> {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    Error::InvalidScheduleVersion => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                    }
                    Error::InvalidCallFlags => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    }
                    Error::OutOfGas => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                    }
                    Error::OutputBufferTooSmall => {
                        __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                    }
                    Error::TransferFailed => {
                        __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                    }
                    Error::MaxCallDepthReached => {
                        __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                    }
                    Error::ContractNotFound => {
                        __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                    }
                    Error::CodeTooLarge => {
                        __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                    }
                    Error::CodeNotFound => {
                        __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                    }
                    Error::OutOfBounds => {
                        __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                    }
                    Error::DecodingFailed => {
                        __codec_dest_edqy.push_byte(10usize as ::core::primitive::u8);
                    }
                    Error::ContractTrapped => {
                        __codec_dest_edqy.push_byte(11usize as ::core::primitive::u8);
                    }
                    Error::ValueTooLarge => {
                        __codec_dest_edqy.push_byte(12usize as ::core::primitive::u8);
                    }
                    Error::TerminatedWhileReentrant => {
                        __codec_dest_edqy.push_byte(13usize as ::core::primitive::u8);
                    }
                    Error::InputForwarded => {
                        __codec_dest_edqy.push_byte(14usize as ::core::primitive::u8);
                    }
                    Error::RandomSubjectTooLong => {
                        __codec_dest_edqy.push_byte(15usize as ::core::primitive::u8);
                    }
                    Error::TooManyTopics => {
                        __codec_dest_edqy.push_byte(16usize as ::core::primitive::u8);
                    }
                    Error::NoChainExtension => {
                        __codec_dest_edqy.push_byte(17usize as ::core::primitive::u8);
                    }
                    Error::DuplicateContract => {
                        __codec_dest_edqy.push_byte(18usize as ::core::primitive::u8);
                    }
                    Error::TerminatedInConstructor => {
                        __codec_dest_edqy.push_byte(19usize as ::core::primitive::u8);
                    }
                    Error::ReentranceDenied => {
                        __codec_dest_edqy.push_byte(20usize as ::core::primitive::u8);
                    }
                    Error::StorageDepositNotEnoughFunds => {
                        __codec_dest_edqy.push_byte(21usize as ::core::primitive::u8);
                    }
                    Error::StorageDepositLimitExhausted => {
                        __codec_dest_edqy.push_byte(22usize as ::core::primitive::u8);
                    }
                    Error::CodeInUse => {
                        __codec_dest_edqy.push_byte(23usize as ::core::primitive::u8);
                    }
                    Error::ContractReverted => {
                        __codec_dest_edqy.push_byte(24usize as ::core::primitive::u8);
                    }
                    Error::CodeRejected => {
                        __codec_dest_edqy.push_byte(25usize as ::core::primitive::u8);
                    }
                    Error::Indeterministic => {
                        __codec_dest_edqy.push_byte(26usize as ::core::primitive::u8);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl<T> ::codec::EncodeLike for Error<T> {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T> ::codec::Decode for Error<T> {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| e.chain("Could not decode `Error`, failed to read variant byte"))?
                {
                    __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::InvalidScheduleVersion)
                    }
                    __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::InvalidCallFlags)
                    }
                    __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::OutOfGas)
                    }
                    __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::OutputBufferTooSmall)
                    }
                    __codec_x_edqy if __codec_x_edqy == 4usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::TransferFailed)
                    }
                    __codec_x_edqy if __codec_x_edqy == 5usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::MaxCallDepthReached)
                    }
                    __codec_x_edqy if __codec_x_edqy == 6usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::ContractNotFound)
                    }
                    __codec_x_edqy if __codec_x_edqy == 7usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::CodeTooLarge)
                    }
                    __codec_x_edqy if __codec_x_edqy == 8usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::CodeNotFound)
                    }
                    __codec_x_edqy if __codec_x_edqy == 9usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::OutOfBounds)
                    }
                    __codec_x_edqy if __codec_x_edqy == 10usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::DecodingFailed)
                    }
                    __codec_x_edqy if __codec_x_edqy == 11usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::ContractTrapped)
                    }
                    __codec_x_edqy if __codec_x_edqy == 12usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::ValueTooLarge)
                    }
                    __codec_x_edqy if __codec_x_edqy == 13usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::TerminatedWhileReentrant)
                    }
                    __codec_x_edqy if __codec_x_edqy == 14usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::InputForwarded)
                    }
                    __codec_x_edqy if __codec_x_edqy == 15usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::RandomSubjectTooLong)
                    }
                    __codec_x_edqy if __codec_x_edqy == 16usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::TooManyTopics)
                    }
                    __codec_x_edqy if __codec_x_edqy == 17usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::NoChainExtension)
                    }
                    __codec_x_edqy if __codec_x_edqy == 18usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::DuplicateContract)
                    }
                    __codec_x_edqy if __codec_x_edqy == 19usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::TerminatedInConstructor)
                    }
                    __codec_x_edqy if __codec_x_edqy == 20usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::ReentranceDenied)
                    }
                    __codec_x_edqy if __codec_x_edqy == 21usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::StorageDepositNotEnoughFunds)
                    }
                    __codec_x_edqy if __codec_x_edqy == 22usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::StorageDepositLimitExhausted)
                    }
                    __codec_x_edqy if __codec_x_edqy == 23usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::CodeInUse)
                    }
                    __codec_x_edqy if __codec_x_edqy == 24usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::ContractReverted)
                    }
                    __codec_x_edqy if __codec_x_edqy == 25usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::CodeRejected)
                    }
                    __codec_x_edqy if __codec_x_edqy == 26usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(Error::<T>::Indeterministic)
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `Error`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T> ::scale_info::TypeInfo for Error<T>
            where
                frame_support::sp_std::marker::PhantomData<(T)>: ::scale_info::TypeInfo + 'static,
                T: 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("Error" , "pallet_contracts::pallet")) . type_params (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([:: scale_info :: TypeParameter :: new ("T" , :: core :: option :: Option :: None)]))) . docs_always (& ["\n\t\t\tCustom [dispatch errors](https://docs.substrate.io/main-docs/build/events-errors/)\n\t\t\tof this pallet.\n\t\t\t"]) . variant (:: scale_info :: build :: Variants :: new () . variant ("InvalidScheduleVersion" , | v | v . index (0usize as :: core :: primitive :: u8) . docs_always (& ["A new schedule must have a greater version than the current one."])) . variant ("InvalidCallFlags" , | v | v . index (1usize as :: core :: primitive :: u8) . docs_always (& ["Invalid combination of flags supplied to `seal_call` or `seal_delegate_call`."])) . variant ("OutOfGas" , | v | v . index (2usize as :: core :: primitive :: u8) . docs_always (& ["The executed contract exhausted its gas limit."])) . variant ("OutputBufferTooSmall" , | v | v . index (3usize as :: core :: primitive :: u8) . docs_always (& ["The output buffer supplied to a contract API call was too small."])) . variant ("TransferFailed" , | v | v . index (4usize as :: core :: primitive :: u8) . docs_always (& ["Performing the requested transfer failed. Probably because there isn't enough" , "free balance in the sender's account."])) . variant ("MaxCallDepthReached" , | v | v . index (5usize as :: core :: primitive :: u8) . docs_always (& ["Performing a call was denied because the calling depth reached the limit" , "of what is specified in the schedule."])) . variant ("ContractNotFound" , | v | v . index (6usize as :: core :: primitive :: u8) . docs_always (& ["No contract was found at the specified address."])) . variant ("CodeTooLarge" , | v | v . index (7usize as :: core :: primitive :: u8) . docs_always (& ["The code supplied to `instantiate_with_code` exceeds the limit specified in the" , "current schedule."])) . variant ("CodeNotFound" , | v | v . index (8usize as :: core :: primitive :: u8) . docs_always (& ["No code could be found at the supplied code hash."])) . variant ("OutOfBounds" , | v | v . index (9usize as :: core :: primitive :: u8) . docs_always (& ["A buffer outside of sandbox memory was passed to a contract API function."])) . variant ("DecodingFailed" , | v | v . index (10usize as :: core :: primitive :: u8) . docs_always (& ["Input passed to a contract API function failed to decode as expected type."])) . variant ("ContractTrapped" , | v | v . index (11usize as :: core :: primitive :: u8) . docs_always (& ["Contract trapped during execution."])) . variant ("ValueTooLarge" , | v | v . index (12usize as :: core :: primitive :: u8) . docs_always (& ["The size defined in `T::MaxValueSize` was exceeded."])) . variant ("TerminatedWhileReentrant" , | v | v . index (13usize as :: core :: primitive :: u8) . docs_always (& ["Termination of a contract is not allowed while the contract is already" , "on the call stack. Can be triggered by `seal_terminate`."])) . variant ("InputForwarded" , | v | v . index (14usize as :: core :: primitive :: u8) . docs_always (& ["`seal_call` forwarded this contracts input. It therefore is no longer available."])) . variant ("RandomSubjectTooLong" , | v | v . index (15usize as :: core :: primitive :: u8) . docs_always (& ["The subject passed to `seal_random` exceeds the limit."])) . variant ("TooManyTopics" , | v | v . index (16usize as :: core :: primitive :: u8) . docs_always (& ["The amount of topics passed to `seal_deposit_events` exceeds the limit."])) . variant ("NoChainExtension" , | v | v . index (17usize as :: core :: primitive :: u8) . docs_always (& ["The chain does not provide a chain extension. Calling the chain extension results" , "in this error. Note that this usually  shouldn't happen as deploying such contracts" , "is rejected."])) . variant ("DuplicateContract" , | v | v . index (18usize as :: core :: primitive :: u8) . docs_always (& ["A contract with the same AccountId already exists."])) . variant ("TerminatedInConstructor" , | v | v . index (19usize as :: core :: primitive :: u8) . docs_always (& ["A contract self destructed in its constructor." , "" , "This can be triggered by a call to `seal_terminate`."])) . variant ("ReentranceDenied" , | v | v . index (20usize as :: core :: primitive :: u8) . docs_always (& ["A call tried to invoke a contract that is flagged as non-reentrant." , "The only other cause is that a call from a contract into the runtime tried to call back" , "into `pallet-contracts`. This would make the whole pallet reentrant with regard to" , "contract code execution which is not supported."])) . variant ("StorageDepositNotEnoughFunds" , | v | v . index (21usize as :: core :: primitive :: u8) . docs_always (& ["Origin doesn't have enough balance to pay the required storage deposits."])) . variant ("StorageDepositLimitExhausted" , | v | v . index (22usize as :: core :: primitive :: u8) . docs_always (& ["More storage was created than allowed by the storage deposit limit."])) . variant ("CodeInUse" , | v | v . index (23usize as :: core :: primitive :: u8) . docs_always (& ["Code removal was denied because the code is still in use by at least one contract."])) . variant ("ContractReverted" , | v | v . index (24usize as :: core :: primitive :: u8) . docs_always (& ["The contract ran to completion but decided to revert its storage changes." , "Please note that this error is only returned from extrinsics. When called directly" , "or via RPC an `Ok` will be returned. In this case the caller needs to inspect the flags" , "to determine whether a reversion has taken place."])) . variant ("CodeRejected" , | v | v . index (25usize as :: core :: primitive :: u8) . docs_always (& ["The contract's code was found to be invalid during validation or instrumentation." , "" , "The most likely cause of this is that an API was used which is not supported by the" , "node. This happens if an older node is used with a new version of ink!. Try updating" , "your node to the newest available version." , "" , "A more detailed error can be found on the node console if debug messages are enabled" , "by supplying `-lruntime::contracts=debug`."])) . variant ("Indeterministic" , | v | v . index (26usize as :: core :: primitive :: u8) . docs_always (& ["An indetermistic code was used in a context where this is not permitted."])))
            }
        };
    };
    const _: () = {
        impl<T> frame_support::traits::PalletError for Error<T> {
            const MAX_ENCODED_SIZE: usize = 1;
        }
    };
    /// A mapping from an original code hash to the original code, untouched by instrumentation.
    #[allow(type_alias_bounds)]
    pub(crate) type PristineCode<T: Config> =
    StorageMap<_GeneratedPrefixForStoragePristineCode<T>, Identity, CodeHash<T>, CodeVec<T>>;
    /// A mapping between an original code hash and instrumented wasm code, ready for execution.
    #[allow(type_alias_bounds)]
    pub(crate) type CodeStorage<T: Config> = StorageMap<
        _GeneratedPrefixForStorageCodeStorage<T>,
        Identity,
        CodeHash<T>,
        PrefabWasmModule<T>,
    >;
    /// A mapping between an original code hash and its owner information.
    #[allow(type_alias_bounds)]
    pub(crate) type OwnerInfoOf<T: Config> =
    StorageMap<_GeneratedPrefixForStorageOwnerInfoOf<T>, Identity, CodeHash<T>, OwnerInfo<T>>;
    /// This is a **monotonic** counter incremented on contract instantiation.
    ///
    /// This is used in order to generate unique trie ids for contracts.
    /// The trie id of a new contract is calculated from hash(account_id, nonce).
    /// The nonce is required because otherwise the following sequence would lead to
    /// a possible collision of storage:
    ///
    /// 1. Create a new contract.
    /// 2. Terminate the contract.
    /// 3. Immediately recreate the contract with the same account_id.
    ///
    /// This is bad because the contents of a trie are deleted lazily and there might be
    /// storage of the old instantiation still in it when the new contract is created. Please
    /// note that we can't replace the counter by the block number because the sequence above
    /// can happen in the same block. We also can't keep the account counter in memory only
    /// because storage is the only way to communicate across different extrinsics in the
    /// same block.
    ///
    /// # Note
    ///
    /// Do not use it to determine the number of contracts. It won't be decremented if
    /// a contract is destroyed.
    #[allow(type_alias_bounds)]
    pub(crate) type Nonce<T: Config> =
    StorageValue<_GeneratedPrefixForStorageNonce<T>, u64, ValueQuery>;
    /// The code associated with a given account.
    ///
    /// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
    #[allow(type_alias_bounds)]
    pub(crate) type ContractInfoOf<T: Config> = StorageMap<
        _GeneratedPrefixForStorageContractInfoOf<T>,
        Twox64Concat,
        T::AccountId,
        ContractInfo<T>,
    >;
    /// Evicted contracts that await child trie deletion.
    ///
    /// Child trie deletion is a heavy operation depending on the amount of storage items
    /// stored in said trie. Therefore this operation is performed lazily in `on_idle`.
    #[allow(type_alias_bounds)]
    pub(crate) type DeletionQueue<T: Config> =
    StorageMap<_GeneratedPrefixForStorageDeletionQueue<T>, Twox64Concat, u32, TrieId>;
    /// A pair of monotonic counters used to track the latest contract marked for deletion
    /// and the latest deleted contract in queue.
    #[allow(type_alias_bounds)]
    pub(crate) type DeletionQueueCounter<T: Config> = StorageValue<
        _GeneratedPrefixForStorageDeletionQueueCounter<T>,
        DeletionQueueManager<T>,
        ValueQuery,
    >;
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn pallet_documentation_metadata() -> frame_support::sp_std::vec::Vec<&'static str> {
            ::alloc::vec::Vec::new()
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn pallet_constants_metadata(
        ) -> frame_support::sp_std::vec::Vec<frame_support::metadata_ir::PalletConstantMetadataIR>
        {
            <[_]>::into_vec(
                #[rustc_box]
                    ::alloc::boxed::Box::new([
                    {
                        frame_support::metadata_ir::PalletConstantMetadataIR {
                            name: "Schedule",
                            ty: frame_support::scale_info::meta_type::<Schedule<T>>(),
                            value: {
                                let value =
                                    <<T as Config>::Schedule as frame_support::traits::Get<
                                        Schedule<T>,
                                    >>::get();
                                frame_support::codec::Encode::encode(&value)
                            },
                            docs: <[_]>::into_vec(
                                #[rustc_box]
                                    ::alloc::boxed::Box::new([" Cost schedule and limits."]),
                            ),
                        }
                    },
                    {
                        frame_support :: metadata_ir :: PalletConstantMetadataIR { name : "DepositPerByte" , ty : frame_support :: scale_info :: meta_type :: < BalanceOf < T > > () , value : { let value = < < T as Config > :: DepositPerByte as frame_support :: traits :: Get < BalanceOf < T > > > :: get () ; frame_support :: codec :: Encode :: encode (& value) } , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" The amount of balance a caller has to pay for each byte of storage." , "" , " # Note" , "" , " Changing this value for an existing chain might need a storage migration."])) , }
                    },
                    {
                        frame_support :: metadata_ir :: PalletConstantMetadataIR { name : "DefaultDepositLimit" , ty : frame_support :: scale_info :: meta_type :: < BalanceOf < T > > () , value : { let value = < < T as Config > :: DefaultDepositLimit as frame_support :: traits :: Get < BalanceOf < T > > > :: get () ; frame_support :: codec :: Encode :: encode (& value) } , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Fallback value to limit the storage deposit if it\'s not being set by the caller."])) , }
                    },
                    {
                        frame_support :: metadata_ir :: PalletConstantMetadataIR { name : "DepositPerItem" , ty : frame_support :: scale_info :: meta_type :: < BalanceOf < T > > () , value : { let value = < < T as Config > :: DepositPerItem as frame_support :: traits :: Get < BalanceOf < T > > > :: get () ; frame_support :: codec :: Encode :: encode (& value) } , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" The amount of balance a caller has to pay for each storage item." , "" , " # Note" , "" , " Changing this value for an existing chain might need a storage migration."])) , }
                    },
                    {
                        frame_support :: metadata_ir :: PalletConstantMetadataIR { name : "MaxCodeLen" , ty : frame_support :: scale_info :: meta_type :: < u32 > () , value : { let value = < < T as Config > :: MaxCodeLen as frame_support :: traits :: Get < u32 > > :: get () ; frame_support :: codec :: Encode :: encode (& value) } , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" The maximum length of a contract code in bytes. This limit applies to the instrumented" , " version of the code. Therefore `instantiate_with_code` can fail even when supplying" , " a wasm binary below this maximum size." , "" , " The value should be chosen carefully taking into the account the overall memory limit" , " your runtime has, as well as the [maximum allowed callstack" , " depth](#associatedtype.CallStack). Look into the `integrity_test()` for some insights."])) , }
                    },
                    {
                        frame_support::metadata_ir::PalletConstantMetadataIR {
                            name: "MaxStorageKeyLen",
                            ty: frame_support::scale_info::meta_type::<u32>(),
                            value: {
                                let value = < < T as Config > :: MaxStorageKeyLen as frame_support :: traits :: Get < u32 > > :: get () ;
                                frame_support::codec::Encode::encode(&value)
                            },
                            docs: <[_]>::into_vec(
                                #[rustc_box]
                                    ::alloc::boxed::Box::new([
                                    " The maximum allowable length in bytes for storage keys.",
                                ]),
                            ),
                        }
                    },
                    {
                        frame_support :: metadata_ir :: PalletConstantMetadataIR { name : "UnsafeUnstableInterface" , ty : frame_support :: scale_info :: meta_type :: < bool > () , value : { let value = < < T as Config > :: UnsafeUnstableInterface as frame_support :: traits :: Get < bool > > :: get () ; frame_support :: codec :: Encode :: encode (& value) } , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Make contract callable functions marked as `#[unstable]` available." , "" , " Contracts that use `#[unstable]` functions won\'t be able to be uploaded unless" , " this is set to `true`. This is only meant for testnets and dev nodes in order to" , " experiment with new features." , "" , " # Warning" , "" , " Do **not** set to `true` on productions chains."])) , }
                    },
                    {
                        frame_support::metadata_ir::PalletConstantMetadataIR {
                            name: "MaxDebugBufferLen",
                            ty: frame_support::scale_info::meta_type::<u32>(),
                            value: {
                                let value = < < T as Config > :: MaxDebugBufferLen as frame_support :: traits :: Get < u32 > > :: get () ;
                                frame_support::codec::Encode::encode(&value)
                            },
                            docs: <[_]>::into_vec(
                                #[rustc_box]
                                    ::alloc::boxed::Box::new([
                                    " The maximum length of the debug buffer in bytes.",
                                ]),
                            ),
                        }
                    },
                ]),
            )
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn error_metadata() -> Option<frame_support::metadata_ir::PalletErrorMetadataIR> {
            Some(frame_support::metadata_ir::PalletErrorMetadataIR {
                ty: frame_support::scale_info::meta_type::<Error<T>>(),
            })
        }
    }
    /// Type alias to `Pallet`, to be used by `construct_runtime`.
    ///
    /// Generated by `pallet` attribute macro.
    #[deprecated(note = "use `Pallet` instead")]
    #[allow(dead_code)]
    pub type Module<T> = Pallet<T>;
    impl<T: Config> frame_support::traits::GetStorageVersion for Pallet<T> {
        type CurrentStorageVersion = frame_support::traits::StorageVersion;
        fn current_storage_version() -> Self::CurrentStorageVersion {
            STORAGE_VERSION
        }
        fn on_chain_storage_version() -> frame_support::traits::StorageVersion {
            frame_support::traits::StorageVersion::get::<Self>()
        }
    }
    impl<T: Config> frame_support::traits::OnGenesis for Pallet<T> {
        fn on_genesis() {
            let storage_version: frame_support::traits::StorageVersion = STORAGE_VERSION;
            storage_version.put::<Self>();
        }
    }
    impl<T: Config> frame_support::traits::PalletInfoAccess for Pallet<T> {
        fn index() -> usize {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<
                Self,
            >()
                .expect(
                    "Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime",
                )
        }
        fn name() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Self,
            >()
                .expect(
                    "Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime",
                )
        }
        fn module_name() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: module_name :: < Self > () . expect ("Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime")
        }
        fn crate_version() -> frame_support::traits::CrateVersion {
            frame_support::traits::CrateVersion {
                major: 4u16,
                minor: 0u8,
                patch: 0u8,
            }
        }
    }
    impl<T: Config> frame_support::traits::PalletsInfoAccess for Pallet<T> {
        fn count() -> usize {
            1
        }
        fn infos() -> frame_support::sp_std::vec::Vec<frame_support::traits::PalletInfoData> {
            use frame_support::traits::PalletInfoAccess;
            let item = frame_support::traits::PalletInfoData {
                index: Self::index(),
                name: Self::name(),
                module_name: Self::module_name(),
                crate_version: Self::crate_version(),
            };
            <[_]>::into_vec(
                #[rustc_box]
                    ::alloc::boxed::Box::new([item]),
            )
        }
    }
    impl<T: Config> frame_support::traits::StorageInfoTrait for Pallet<T> {
        fn storage_info() -> frame_support::sp_std::vec::Vec<frame_support::traits::StorageInfo> {
            #[allow(unused_mut)]
                let mut res = ::alloc::vec::Vec::new();
            {
                let mut storage_info =
                    <PristineCode<T> as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info =
                    <CodeStorage<T> as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info =
                    <OwnerInfoOf<T> as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info =
                    <Nonce<T> as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info =
                    <ContractInfoOf<T> as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info =
                    <DeletionQueue<T> as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info = < DeletionQueueCounter < T > as frame_support :: traits :: StorageInfoTrait > :: storage_info () ;
                res.append(&mut storage_info);
            }
            res
        }
    }
    use frame_support::traits::{StorageInfoTrait, TrackedStorageKey, WhitelistedStorageKeys};
    impl<T: Config> WhitelistedStorageKeys for Pallet<T> {
        fn whitelisted_storage_keys() -> frame_support::sp_std::vec::Vec<TrackedStorageKey> {
            use frame_support::sp_std::vec;
            ::alloc::vec::Vec::new()
        }
    }
    mod warnings {}
    #[doc(hidden)]
    pub mod __substrate_call_check {
        #[doc(hidden)]
        pub use __is_call_part_defined_0 as is_call_part_defined;
    }
    ///Contains one variant per dispatchable that can be called by an extrinsic.
    #[codec(encode_bound())]
    #[codec(decode_bound())]
    #[scale_info(skip_type_params(T), capture_docs = "always")]
    #[allow(non_camel_case_types)]
    pub enum Call<T: Config>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(
            frame_support::sp_std::marker::PhantomData<(T,)>,
            frame_support::Never,
        ),
        /// Deprecated version if [`Self::call`] for use in an in-storage `Call`.
        #[codec(index = 0u8)]
        call_old_weight {
            #[allow(missing_docs)]
            dest: AccountIdLookupOf<T>,
            #[allow(missing_docs)]
            #[codec(compact)]
            value: BalanceOf<T>,
            #[allow(missing_docs)]
            #[codec(compact)]
            gas_limit: OldWeight,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            data: Vec<u8>,
        },
        /// Deprecated version if [`Self::instantiate_with_code`] for use in an in-storage `Call`.
        #[codec(index = 1u8)]
        instantiate_with_code_old_weight {
            #[allow(missing_docs)]
            #[codec(compact)]
            value: BalanceOf<T>,
            #[allow(missing_docs)]
            #[codec(compact)]
            gas_limit: OldWeight,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            code: Vec<u8>,
            #[allow(missing_docs)]
            data: Vec<u8>,
            #[allow(missing_docs)]
            salt: Vec<u8>,
        },
        /// Deprecated version if [`Self::instantiate`] for use in an in-storage `Call`.
        #[codec(index = 2u8)]
        instantiate_old_weight {
            #[allow(missing_docs)]
            #[codec(compact)]
            value: BalanceOf<T>,
            #[allow(missing_docs)]
            #[codec(compact)]
            gas_limit: OldWeight,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            code_hash: CodeHash<T>,
            #[allow(missing_docs)]
            data: Vec<u8>,
            #[allow(missing_docs)]
            salt: Vec<u8>,
        },
        /// Upload new `code` without instantiating a contract from it.
        ///
        /// If the code does not already exist a deposit is reserved from the caller
        /// and unreserved only when [`Self::remove_code`] is called. The size of the reserve
        /// depends on the instrumented size of the the supplied `code`.
        ///
        /// If the code already exists in storage it will still return `Ok` and upgrades
        /// the in storage version to the current
        /// [`InstructionWeights::version`](InstructionWeights).
        ///
        /// - `determinism`: If this is set to any other value but [`Determinism::Enforced`] then
        ///   the only way to use this code is to delegate call into it from an offchain execution.
        ///   Set to [`Determinism::Enforced`] if in doubt.
        ///
        /// # Note
        ///
        /// Anyone can instantiate a contract from any uploaded code and thus prevent its removal.
        /// To avoid this situation a constructor could employ access control so that it can
        /// only be instantiated by permissioned entities. The same is true when uploading
        /// through [`Self::instantiate_with_code`].
        #[codec(index = 3u8)]
        upload_code {
            #[allow(missing_docs)]
            code: Vec<u8>,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            determinism: Determinism,
        },
        /// Remove the code stored under `code_hash` and refund the deposit to its owner.
        ///
        /// A code can only be removed by its original uploader (its owner) and only if it is
        /// not used by any contract.
        #[codec(index = 4u8)]
        remove_code {
            #[allow(missing_docs)]
            code_hash: CodeHash<T>,
        },
        /// Privileged function that changes the code of an existing contract.
        ///
        /// This takes care of updating refcounts and all other necessary operations. Returns
        /// an error if either the `code_hash` or `dest` do not exist.
        ///
        /// # Note
        ///
        /// This does **not** change the address of the contract in question. This means
        /// that the contract address is no longer derived from its code hash after calling
        /// this dispatchable.
        #[codec(index = 5u8)]
        set_code {
            #[allow(missing_docs)]
            dest: AccountIdLookupOf<T>,
            #[allow(missing_docs)]
            code_hash: CodeHash<T>,
        },
        /// Makes a call to an account, optionally transferring some balance.
        ///
        /// # Parameters
        ///
        /// * `dest`: Address of the contract to call.
        /// * `value`: The balance to transfer from the `origin` to `dest`.
        /// * `gas_limit`: The gas limit enforced when executing the constructor.
        /// * `storage_deposit_limit`: The maximum amount of balance that can be charged from the
        ///   caller to pay for the storage consumed.
        /// * `data`: The input data to pass to the contract.
        ///
        /// * If the account is a smart-contract account, the associated code will be
        /// executed and any value will be transferred.
        /// * If the account is a regular account, any value will be transferred.
        /// * If no account exists and the call value is not less than `existential_deposit`,
        /// a regular account will be created and any value will be transferred.
        #[codec(index = 6u8)]
        call {
            #[allow(missing_docs)]
            dest: AccountIdLookupOf<T>,
            #[allow(missing_docs)]
            #[codec(compact)]
            value: BalanceOf<T>,
            #[allow(missing_docs)]
            gas_limit: Weight,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            data: Vec<u8>,
        },
        /// Instantiates a new contract from the supplied `code` optionally transferring
        /// some balance.
        ///
        /// This dispatchable has the same effect as calling [`Self::upload_code`] +
        /// [`Self::instantiate`]. Bundling them together provides efficiency gains. Please
        /// also check the documentation of [`Self::upload_code`].
        ///
        /// # Parameters
        ///
        /// * `value`: The balance to transfer from the `origin` to the newly created contract.
        /// * `gas_limit`: The gas limit enforced when executing the constructor.
        /// * `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved
        ///   from the caller to pay for the storage consumed.
        /// * `code`: The contract code to deploy in raw bytes.
        /// * `data`: The input data to pass to the contract constructor.
        /// * `salt`: Used for the address derivation. See [`Pallet::contract_address`].
        ///
        /// Instantiation is executed as follows:
        ///
        /// - The supplied `code` is instrumented, deployed, and a `code_hash` is created for that
        ///   code.
        /// - If the `code_hash` already exists on the chain the underlying `code` will be shared.
        /// - The destination address is computed based on the sender, code_hash and the salt.
        /// - The smart-contract account is created at the computed address.
        /// - The `value` is transferred to the new account.
        /// - The `deploy` function is executed in the context of the newly-created account.
        #[codec(index = 7u8)]
        instantiate_with_code {
            #[allow(missing_docs)]
            #[codec(compact)]
            value: BalanceOf<T>,
            #[allow(missing_docs)]
            gas_limit: Weight,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            code: Vec<u8>,
            #[allow(missing_docs)]
            data: Vec<u8>,
            #[allow(missing_docs)]
            salt: Vec<u8>,
        },
        /// Instantiates a contract from a previously deployed wasm binary.
        ///
        /// This function is identical to [`Self::instantiate_with_code`] but without the
        /// code deployment step. Instead, the `code_hash` of an on-chain deployed wasm binary
        /// must be supplied.
        #[codec(index = 8u8)]
        instantiate {
            #[allow(missing_docs)]
            #[codec(compact)]
            value: BalanceOf<T>,
            #[allow(missing_docs)]
            gas_limit: Weight,
            #[allow(missing_docs)]
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            #[allow(missing_docs)]
            code_hash: CodeHash<T>,
            #[allow(missing_docs)]
            data: Vec<u8>,
            #[allow(missing_docs)]
            salt: Vec<u8>,
        },
    }
    impl<T: Config> Call<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        ///Create a call with the variant `call_old_weight`.
        pub fn new_call_variant_call_old_weight(
            dest: AccountIdLookupOf<T>,
            value: BalanceOf<T>,
            gas_limit: OldWeight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            data: Vec<u8>,
        ) -> Self {
            Self::call_old_weight {
                dest,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            }
        }
        ///Create a call with the variant `instantiate_with_code_old_weight`.
        pub fn new_call_variant_instantiate_with_code_old_weight(
            value: BalanceOf<T>,
            gas_limit: OldWeight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> Self {
            Self::instantiate_with_code_old_weight {
                value,
                gas_limit,
                storage_deposit_limit,
                code,
                data,
                salt,
            }
        }
        ///Create a call with the variant `instantiate_old_weight`.
        pub fn new_call_variant_instantiate_old_weight(
            value: BalanceOf<T>,
            gas_limit: OldWeight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> Self {
            Self::instantiate_old_weight {
                value,
                gas_limit,
                storage_deposit_limit,
                code_hash,
                data,
                salt,
            }
        }
        ///Create a call with the variant `upload_code`.
        pub fn new_call_variant_upload_code(
            code: Vec<u8>,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            determinism: Determinism,
        ) -> Self {
            Self::upload_code {
                code,
                storage_deposit_limit,
                determinism,
            }
        }
        ///Create a call with the variant `remove_code`.
        pub fn new_call_variant_remove_code(code_hash: CodeHash<T>) -> Self {
            Self::remove_code { code_hash }
        }
        ///Create a call with the variant `set_code`.
        pub fn new_call_variant_set_code(
            dest: AccountIdLookupOf<T>,
            code_hash: CodeHash<T>,
        ) -> Self {
            Self::set_code { dest, code_hash }
        }
        ///Create a call with the variant `call`.
        pub fn new_call_variant_call(
            dest: AccountIdLookupOf<T>,
            value: BalanceOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            data: Vec<u8>,
        ) -> Self {
            Self::call {
                dest,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            }
        }
        ///Create a call with the variant `instantiate_with_code`.
        pub fn new_call_variant_instantiate_with_code(
            value: BalanceOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code: Vec<u8>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> Self {
            Self::instantiate_with_code {
                value,
                gas_limit,
                storage_deposit_limit,
                code,
                data,
                salt,
            }
        }
        ///Create a call with the variant `instantiate`.
        pub fn new_call_variant_instantiate(
            value: BalanceOf<T>,
            gas_limit: Weight,
            storage_deposit_limit: Option<<BalanceOf<T> as codec::HasCompact>::Type>,
            code_hash: CodeHash<T>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> Self {
            Self::instantiate {
                value,
                gas_limit,
                storage_deposit_limit,
                code_hash,
                data,
                salt,
            }
        }
    }
    impl<T: Config> frame_support::dispatch::GetDispatchInfo for Call<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
            match *self {
                Self::call_old_weight {
                    ref dest,
                    ref value,
                    ref gas_limit,
                    ref storage_deposit_limit,
                    ref data,
                } => {
                    let __pallet_base_weight = T::WeightInfo::call()
                        .saturating_add(<Pallet<T>>::compat_weight_limit(*gas_limit));
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &AccountIdLookupOf<T>,
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (dest, value, gas_limit, storage_deposit_limit, data),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &AccountIdLookupOf<T>,
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (dest, value, gas_limit, storage_deposit_limit, data),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &AccountIdLookupOf<T>,
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (dest, value, gas_limit, storage_deposit_limit, data),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::instantiate_with_code_old_weight {
                    ref value,
                    ref gas_limit,
                    ref storage_deposit_limit,
                    ref code,
                    ref data,
                    ref salt,
                } => {
                    let __pallet_base_weight = T::WeightInfo::instantiate_with_code(
                        code.len() as u32,
                        data.len() as u32,
                        salt.len() as u32,
                    )
                        .saturating_add(<Pallet<T>>::compat_weight_limit(*gas_limit));
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (value, gas_limit, storage_deposit_limit, code, data, salt),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (value, gas_limit, storage_deposit_limit, code, data, salt),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (value, gas_limit, storage_deposit_limit, code, data, salt),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::instantiate_old_weight {
                    ref value,
                    ref gas_limit,
                    ref storage_deposit_limit,
                    ref code_hash,
                    ref data,
                    ref salt,
                } => {
                    let __pallet_base_weight =
                        T::WeightInfo::instantiate(data.len() as u32, salt.len() as u32)
                            .saturating_add(<Pallet<T>>::compat_weight_limit(*gas_limit));
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &CodeHash<T>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (
                            value,
                            gas_limit,
                            storage_deposit_limit,
                            code_hash,
                            data,
                            salt,
                        ),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &CodeHash<T>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (
                            value,
                            gas_limit,
                            storage_deposit_limit,
                            code_hash,
                            data,
                            salt,
                        ),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &BalanceOf<T>,
                        &OldWeight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &CodeHash<T>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (
                            value,
                            gas_limit,
                            storage_deposit_limit,
                            code_hash,
                            data,
                            salt,
                        ),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::upload_code {
                    ref code,
                    ref storage_deposit_limit,
                    ref determinism,
                } => {
                    let __pallet_base_weight = T::WeightInfo::upload_code(code.len() as u32);
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &Vec<u8>,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Determinism,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (code, storage_deposit_limit, determinism),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &Vec<u8>,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Determinism,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (code, storage_deposit_limit, determinism),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &Vec<u8>,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Determinism,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (code, storage_deposit_limit, determinism),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::remove_code { ref code_hash } => {
                    let __pallet_base_weight = T::WeightInfo::remove_code();
                    let __pallet_weight =
                        <dyn frame_support::dispatch::WeighData<(&CodeHash<T>,)>>::weigh_data(
                            &__pallet_base_weight,
                            (code_hash,),
                        );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &CodeHash<T>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight, (code_hash,)
                    );
                    let __pallet_pays_fee =
                        <dyn frame_support::dispatch::PaysFee<(&CodeHash<T>,)>>::pays_fee(
                            &__pallet_base_weight,
                            (code_hash,),
                        );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::set_code {
                    ref dest,
                    ref code_hash,
                } => {
                    let __pallet_base_weight = T::WeightInfo::set_code();
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &AccountIdLookupOf<T>,
                        &CodeHash<T>,
                    )>>::weigh_data(
                        &__pallet_base_weight, (dest, code_hash)
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &AccountIdLookupOf<T>,
                        &CodeHash<T>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight, (dest, code_hash)
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &AccountIdLookupOf<T>,
                        &CodeHash<T>,
                    )>>::pays_fee(
                        &__pallet_base_weight, (dest, code_hash)
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::call {
                    ref dest,
                    ref value,
                    ref gas_limit,
                    ref storage_deposit_limit,
                    ref data,
                } => {
                    let __pallet_base_weight = T::WeightInfo::call().saturating_add(*gas_limit);
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &AccountIdLookupOf<T>,
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (dest, value, gas_limit, storage_deposit_limit, data),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &AccountIdLookupOf<T>,
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (dest, value, gas_limit, storage_deposit_limit, data),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &AccountIdLookupOf<T>,
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (dest, value, gas_limit, storage_deposit_limit, data),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::instantiate_with_code {
                    ref value,
                    ref gas_limit,
                    ref storage_deposit_limit,
                    ref code,
                    ref data,
                    ref salt,
                } => {
                    let __pallet_base_weight = T::WeightInfo::instantiate_with_code(
                        code.len() as u32,
                        data.len() as u32,
                        salt.len() as u32,
                    )
                        .saturating_add(*gas_limit);
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (value, gas_limit, storage_deposit_limit, code, data, salt),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (value, gas_limit, storage_deposit_limit, code, data, salt),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &Vec<u8>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (value, gas_limit, storage_deposit_limit, code, data, salt),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::instantiate {
                    ref value,
                    ref gas_limit,
                    ref storage_deposit_limit,
                    ref code_hash,
                    ref data,
                    ref salt,
                } => {
                    let __pallet_base_weight =
                        T::WeightInfo::instantiate(data.len() as u32, salt.len() as u32)
                            .saturating_add(*gas_limit);
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<(
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &CodeHash<T>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::weigh_data(
                        &__pallet_base_weight,
                        (
                            value,
                            gas_limit,
                            storage_deposit_limit,
                            code_hash,
                            data,
                            salt,
                        ),
                    );
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &CodeHash<T>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::classify_dispatch(
                        &__pallet_base_weight,
                        (
                            value,
                            gas_limit,
                            storage_deposit_limit,
                            code_hash,
                            data,
                            salt,
                        ),
                    );
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &BalanceOf<T>,
                        &Weight,
                        &Option<<BalanceOf<T> as codec::HasCompact>::Type>,
                        &CodeHash<T>,
                        &Vec<u8>,
                        &Vec<u8>,
                    )>>::pays_fee(
                        &__pallet_base_weight,
                        (
                            value,
                            gas_limit,
                            storage_deposit_limit,
                            code_hash,
                            data,
                            salt,
                        ),
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::__Ignore(_, _) => ::core::panicking::panic_fmt(format_args!(
                    "internal error: entered unreachable code: {0}",
                    format_args!("__Ignore cannot be used")
                )),
            }
        }
    }
    #[allow(deprecated)]
    impl<T: Config> frame_support::weights::GetDispatchInfo for Call<T> where
        <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode
    {
    }
    impl<T: Config> frame_support::dispatch::GetCallName for Call<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        fn get_call_name(&self) -> &'static str {
            match *self {
                Self::call_old_weight { .. } => "call_old_weight",
                Self::instantiate_with_code_old_weight { .. } => "instantiate_with_code_old_weight",
                Self::instantiate_old_weight { .. } => "instantiate_old_weight",
                Self::upload_code { .. } => "upload_code",
                Self::remove_code { .. } => "remove_code",
                Self::set_code { .. } => "set_code",
                Self::call { .. } => "call",
                Self::instantiate_with_code { .. } => "instantiate_with_code",
                Self::instantiate { .. } => "instantiate",
                Self::__Ignore(_, _) => ::core::panicking::panic_fmt(format_args!(
                    "internal error: entered unreachable code: {0}",
                    format_args!("__PhantomItem cannot be used.")
                )),
            }
        }
        fn get_call_names() -> &'static [&'static str] {
            &[
                "call_old_weight",
                "instantiate_with_code_old_weight",
                "instantiate_old_weight",
                "upload_code",
                "remove_code",
                "set_code",
                "call",
                "instantiate_with_code",
                "instantiate",
            ]
        }
    }
    impl<T: Config> frame_support::dispatch::GetCallIndex for Call<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        fn get_call_index(&self) -> u8 {
            match *self {
                Self::call_old_weight { .. } => 0u8,
                Self::instantiate_with_code_old_weight { .. } => 1u8,
                Self::instantiate_old_weight { .. } => 2u8,
                Self::upload_code { .. } => 3u8,
                Self::remove_code { .. } => 4u8,
                Self::set_code { .. } => 5u8,
                Self::call { .. } => 6u8,
                Self::instantiate_with_code { .. } => 7u8,
                Self::instantiate { .. } => 8u8,
                Self::__Ignore(_, _) => ::core::panicking::panic_fmt(format_args!(
                    "internal error: entered unreachable code: {0}",
                    format_args!("__PhantomItem cannot be used.")
                )),
            }
        }
        fn get_call_indices() -> &'static [u8] {
            &[0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8]
        }
    }
    impl<T: Config> frame_support::traits::UnfilteredDispatchable for Call<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        type RuntimeOrigin = frame_system::pallet_prelude::OriginFor<T>;
        fn dispatch_bypass_filter(
            self,
            origin: Self::RuntimeOrigin,
        ) -> frame_support::dispatch::DispatchResultWithPostInfo {
            frame_support::dispatch_context::run_in_context(|| match self {
                Self::call_old_weight {
                    dest,
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    data,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "call_old_weight",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    #[allow(deprecated)]
                    <Pallet<T>>::call_old_weight(
                        origin,
                        dest,
                        value,
                        gas_limit,
                        storage_deposit_limit,
                        data,
                    )
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::instantiate_with_code_old_weight {
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    code,
                    data,
                    salt,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "instantiate_with_code_old_weight",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    #[allow(deprecated)]
                    <Pallet<T>>::instantiate_with_code_old_weight(
                        origin,
                        value,
                        gas_limit,
                        storage_deposit_limit,
                        code,
                        data,
                        salt,
                    )
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::instantiate_old_weight {
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    code_hash,
                    data,
                    salt,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "instantiate_old_weight",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    #[allow(deprecated)]
                    <Pallet<T>>::instantiate_old_weight(
                        origin,
                        value,
                        gas_limit,
                        storage_deposit_limit,
                        code_hash,
                        data,
                        salt,
                    )
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::upload_code {
                    code,
                    storage_deposit_limit,
                    determinism,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "upload_code",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    <Pallet<T>>::upload_code(origin, code, storage_deposit_limit, determinism)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::remove_code { code_hash } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "remove_code",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    <Pallet<T>>::remove_code(origin, code_hash)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::set_code { dest, code_hash } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "set_code",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    <Pallet<T>>::set_code(origin, dest, code_hash)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::call {
                    dest,
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    data,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "call",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    <Pallet<T>>::call(origin, dest, value, gas_limit, storage_deposit_limit, data)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::instantiate_with_code {
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    code,
                    data,
                    salt,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "instantiate_with_code",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    <Pallet<T>>::instantiate_with_code(
                        origin,
                        value,
                        gas_limit,
                        storage_deposit_limit,
                        code,
                        data,
                        salt,
                    )
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::instantiate {
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    code_hash,
                    data,
                    salt,
                } => {
                    let __within_span__ = {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "instantiate",
                                    "pallet_contracts::pallet",
                                    ::tracing::Level::TRACE,
                                    Some("frame/contracts/src/lib.rs"),
                                    Some(175u32),
                                    Some("pallet_contracts::pallet"),
                                    ::tracing_core::field::FieldSet::new(
                                        &[],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::SPAN,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let mut interest = ::tracing::subscriber::Interest::never();
                        if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::TRACE
                            <= ::tracing::level_filters::LevelFilter::current()
                            && {
                            interest = CALLSITE.interest();
                            !interest.is_never()
                        }
                            && ::tracing::__macro_support::__is_enabled(
                            CALLSITE.metadata(),
                            interest,
                        )
                        {
                            let meta = CALLSITE.metadata();
                            ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                        } else {
                            let span =
                                ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                            {};
                            span
                        }
                    };
                    let __tracing_guard__ = __within_span__.enter();
                    <Pallet<T>>::instantiate(
                        origin,
                        value,
                        gas_limit,
                        storage_deposit_limit,
                        code_hash,
                        data,
                        salt,
                    )
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::__Ignore(_, _) => {
                    let _ = origin;
                    ::core::panicking::panic_fmt(format_args!(
                        "internal error: entered unreachable code: {0}",
                        format_args!("__PhantomItem cannot be used.")
                    ));
                }
            })
        }
    }
    impl<T: Config> frame_support::dispatch::Callable<T> for Pallet<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        type RuntimeCall = Call<T>;
    }
    impl<T: Config> Pallet<T>
        where
            <BalanceOf<T> as HasCompact>::Type: Clone + Eq + PartialEq + Debug + TypeInfo + Encode,
    {
        #[doc(hidden)]
        pub fn call_functions() -> frame_support::metadata_ir::PalletCallMetadataIR {
            frame_support::scale_info::meta_type::<Call<T>>().into()
        }
    }
    impl<T: Config> frame_support::sp_std::fmt::Debug for Error<T> {
        fn fmt(
            &self,
            f: &mut frame_support::sp_std::fmt::Formatter<'_>,
        ) -> frame_support::sp_std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    impl<T: Config> Error<T> {
        #[doc(hidden)]
        pub fn as_str(&self) -> &'static str {
            match &self {
                Self::__Ignore(_, _) => ::core::panicking::panic_fmt(format_args!(
                    "internal error: entered unreachable code: {0}",
                    format_args!("`__Ignore` can never be constructed")
                )),
                Self::InvalidScheduleVersion => "InvalidScheduleVersion",
                Self::InvalidCallFlags => "InvalidCallFlags",
                Self::OutOfGas => "OutOfGas",
                Self::OutputBufferTooSmall => "OutputBufferTooSmall",
                Self::TransferFailed => "TransferFailed",
                Self::MaxCallDepthReached => "MaxCallDepthReached",
                Self::ContractNotFound => "ContractNotFound",
                Self::CodeTooLarge => "CodeTooLarge",
                Self::CodeNotFound => "CodeNotFound",
                Self::OutOfBounds => "OutOfBounds",
                Self::DecodingFailed => "DecodingFailed",
                Self::ContractTrapped => "ContractTrapped",
                Self::ValueTooLarge => "ValueTooLarge",
                Self::TerminatedWhileReentrant => "TerminatedWhileReentrant",
                Self::InputForwarded => "InputForwarded",
                Self::RandomSubjectTooLong => "RandomSubjectTooLong",
                Self::TooManyTopics => "TooManyTopics",
                Self::NoChainExtension => "NoChainExtension",
                Self::DuplicateContract => "DuplicateContract",
                Self::TerminatedInConstructor => "TerminatedInConstructor",
                Self::ReentranceDenied => "ReentranceDenied",
                Self::StorageDepositNotEnoughFunds => "StorageDepositNotEnoughFunds",
                Self::StorageDepositLimitExhausted => "StorageDepositLimitExhausted",
                Self::CodeInUse => "CodeInUse",
                Self::ContractReverted => "ContractReverted",
                Self::CodeRejected => "CodeRejected",
                Self::Indeterministic => "Indeterministic",
            }
        }
    }
    impl<T: Config> From<Error<T>> for &'static str {
        fn from(err: Error<T>) -> &'static str {
            err.as_str()
        }
    }
    impl<T: Config> From<Error<T>> for frame_support::sp_runtime::DispatchError {
        fn from(err: Error<T>) -> Self {
            use frame_support::codec::Encode;
            let index = < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: index :: < Pallet < T > > () . expect ("Every active module has an index in the runtime; qed") as u8 ;
            let mut encoded = err.encode();
            encoded.resize(frame_support::MAX_MODULE_ERROR_ENCODED_SIZE, 0);
            frame_support :: sp_runtime :: DispatchError :: Module (frame_support :: sp_runtime :: ModuleError { index , error : TryInto :: try_into (encoded) . expect ("encoded error is resized to be equal to the maximum encoded error size; qed") , message : Some (err . as_str ()) , })
        }
    }
    pub use __tt_error_token_1 as tt_error_token;
    #[doc(hidden)]
    pub mod __substrate_event_check {
        #[doc(hidden)]
        pub use __is_event_part_defined_2 as is_event_part_defined;
    }
    impl<T: Config> From<Event<T>> for () {
        fn from(_: Event<T>) {}
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn storage_metadata() -> frame_support::metadata_ir::PalletStorageMetadataIR {
            frame_support :: metadata_ir :: PalletStorageMetadataIR { prefix : < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.") , entries : { # [allow (unused_mut)] let mut entries = :: alloc :: vec :: Vec :: new () ; { < PristineCode < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" A mapping from an original code hash to the original code, untouched by instrumentation."])) , & mut entries) ; } { < CodeStorage < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" A mapping between an original code hash and instrumented wasm code, ready for execution."])) , & mut entries) ; } { < OwnerInfoOf < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" A mapping between an original code hash and its owner information."])) , & mut entries) ; } { < Nonce < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" This is a **monotonic** counter incremented on contract instantiation." , "" , " This is used in order to generate unique trie ids for contracts." , " The trie id of a new contract is calculated from hash(account_id, nonce)." , " The nonce is required because otherwise the following sequence would lead to" , " a possible collision of storage:" , "" , " 1. Create a new contract." , " 2. Terminate the contract." , " 3. Immediately recreate the contract with the same account_id." , "" , " This is bad because the contents of a trie are deleted lazily and there might be" , " storage of the old instantiation still in it when the new contract is created. Please" , " note that we can\'t replace the counter by the block number because the sequence above" , " can happen in the same block. We also can\'t keep the account counter in memory only" , " because storage is the only way to communicate across different extrinsics in the" , " same block." , "" , " # Note" , "" , " Do not use it to determine the number of contracts. It won\'t be decremented if" , " a contract is destroyed."])) , & mut entries) ; } { < ContractInfoOf < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" The code associated with a given account." , "" , " TWOX-NOTE: SAFE since `AccountId` is a secure hash."])) , & mut entries) ; } { < DeletionQueue < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Evicted contracts that await child trie deletion." , "" , " Child trie deletion is a heavy operation depending on the amount of storage items" , " stored in said trie. Therefore this operation is performed lazily in `on_idle`."])) , & mut entries) ; } { < DeletionQueueCounter < T > as frame_support :: storage :: StorageEntryMetadataBuilder > :: build_metadata (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" A pair of monotonic counters used to track the latest contract marked for deletion" , " and the latest deleted contract in queue."])) , & mut entries) ; } entries } , }
        }
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStoragePristineCode<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStoragePristineCode<T>
    {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "PristineCode";
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStorageCodeStorage<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageCodeStorage<T>
    {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "CodeStorage";
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStorageOwnerInfoOf<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageOwnerInfoOf<T>
    {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "OwnerInfoOf";
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStorageNonce<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance for _GeneratedPrefixForStorageNonce<T> {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "Nonce";
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStorageContractInfoOf<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageContractInfoOf<T>
    {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "ContractInfoOf";
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStorageDeletionQueue<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageDeletionQueue<T>
    {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "DeletionQueue";
    }
    #[doc(hidden)]
    pub(crate) struct _GeneratedPrefixForStorageDeletionQueueCounter<T>(
        core::marker::PhantomData<(T,)>,
    );
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageDeletionQueueCounter<T>
    {
        fn pallet_prefix() -> &'static str {
            < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.")
        }
        const STORAGE_PREFIX: &'static str = "DeletionQueueCounter";
    }
    #[doc(hidden)]
    pub mod __substrate_inherent_check {
        #[doc(hidden)]
        pub use __is_inherent_part_defined_3 as is_inherent_part_defined;
    }
    /// Hidden instance generated to be internally used when module is used without
    /// instance.
    #[doc(hidden)]
    pub type __InherentHiddenInstance = ();
    impl<T: Config> frame_support::traits::OnFinalize<<T as frame_system::Config>::BlockNumber>
    for Pallet<T>
    {
        fn on_finalize(n: <T as frame_system::Config>::BlockNumber) {
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "on_finalize",
                            "pallet_contracts::pallet",
                            ::tracing::Level::TRACE,
                            Some("frame/contracts/src/lib.rs"),
                            Some(175u32),
                            Some("pallet_contracts::pallet"),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE <= ::tracing::level_filters::LevelFilter::current()
                    && {
                    interest = CALLSITE.interest();
                    !interest.is_never()
                }
                    && ::tracing::__macro_support::__is_enabled(CALLSITE.metadata(), interest)
                {
                    let meta = CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                    {};
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: on_finalize (n)
        }
    }
    impl<T: Config> frame_support::traits::OnIdle<<T as frame_system::Config>::BlockNumber>
    for Pallet<T>
    {
        fn on_idle(
            n: <T as frame_system::Config>::BlockNumber,
            remaining_weight: frame_support::weights::Weight,
        ) -> frame_support::weights::Weight {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: on_idle (n , remaining_weight)
        }
    }
    impl<T: Config> frame_support::traits::OnInitialize<<T as frame_system::Config>::BlockNumber>
    for Pallet<T>
    {
        fn on_initialize(
            n: <T as frame_system::Config>::BlockNumber,
        ) -> frame_support::weights::Weight {
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "on_initialize",
                            "pallet_contracts::pallet",
                            ::tracing::Level::TRACE,
                            Some("frame/contracts/src/lib.rs"),
                            Some(175u32),
                            Some("pallet_contracts::pallet"),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE <= ::tracing::level_filters::LevelFilter::current()
                    && {
                    interest = CALLSITE.interest();
                    !interest.is_never()
                }
                    && ::tracing::__macro_support::__is_enabled(CALLSITE.metadata(), interest)
                {
                    let meta = CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                    {};
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: on_initialize (n)
        }
    }
    impl<T: Config> frame_support::traits::OnRuntimeUpgrade for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "on_runtime_update",
                            "pallet_contracts::pallet",
                            ::tracing::Level::TRACE,
                            Some("frame/contracts/src/lib.rs"),
                            Some(175u32),
                            Some("pallet_contracts::pallet"),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE <= ::tracing::level_filters::LevelFilter::current()
                    && {
                    interest = CALLSITE.interest();
                    !interest.is_never()
                }
                    && ::tracing::__macro_support::__is_enabled(CALLSITE.metadata(), interest)
                {
                    let meta = CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(CALLSITE.metadata());
                    {};
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            let pallet_name = < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Self > () . unwrap_or ("<unknown pallet name>") ;
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api_log(
                        format_args!("✅ no migration for {0}", pallet_name),
                        lvl,
                        &(
                            frame_support::LOG_TARGET,
                            "pallet_contracts::pallet",
                            "frame/contracts/src/lib.rs",
                            175u32,
                        ),
                        ::log::__private_api::Option::None,
                    );
                }
            };
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: on_runtime_upgrade ()
        }
    }
    impl<T: Config> frame_support::traits::OffchainWorker<<T as frame_system::Config>::BlockNumber>
    for Pallet<T>
    {
        fn offchain_worker(n: <T as frame_system::Config>::BlockNumber) {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: offchain_worker (n)
        }
    }
    impl<T: Config> frame_support::traits::IntegrityTest for Pallet<T> {
        fn integrity_test() {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: integrity_test ()
        }
    }
    #[doc(hidden)]
    pub mod __substrate_genesis_config_check {
        #[doc(hidden)]
        pub use __is_genesis_config_defined_4 as is_genesis_config_defined;
        #[doc(hidden)]
        pub use __is_std_enabled_for_genesis_4 as is_std_enabled_for_genesis;
    }
    #[doc(hidden)]
    pub mod __substrate_origin_check {
        #[doc(hidden)]
        pub use __is_origin_part_defined_5 as is_origin_part_defined;
    }
    #[doc(hidden)]
    pub mod __substrate_validate_unsigned_check {
        #[doc(hidden)]
        pub use __is_validate_unsigned_part_defined_6 as is_validate_unsigned_part_defined;
    }
    pub use __tt_default_parts_7 as tt_default_parts;
}
