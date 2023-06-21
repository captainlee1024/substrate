pub mod chain_extension {
    //! A mechanism for runtime authors to augment the functionality of contracts.
    //!
    //! The runtime is able to call into any contract and retrieve the result using
    //! [`bare_call`](crate::Pallet::bare_call). This already allows customization of runtime
    //! behaviour by user generated code (contracts). However, often it is more straightforward
    //! to allow the reverse behaviour: The contract calls into the runtime. We call the latter
    //! one a "chain extension" because it allows the chain to extend the set of functions that are
    //! callable by a contract.
    //!
    //! In order to create a chain extension the runtime author implements the [`ChainExtension`]
    //! trait and declares it in this pallet's [configuration Trait](crate::Config). All types
    //! required for this endeavour are defined or re-exported in this module. There is an
    //! implementation on `()` which can be used to signal that no chain extension is available.
    //!
    //! # Using multiple chain extensions
    //!
    //! Often there is a need for having multiple chain extensions. This is often the case when
    //! some generally useful off-the-shelf extensions should be included. To have multiple chain
    //! extensions they can be put into a tuple which is then passed to [`Config::ChainExtension`] like
    //! this `type Extensions = (ExtensionA, ExtensionB)`.
    //!
    //! However, only extensions implementing [`RegisteredChainExtension`] can be put into a tuple.
    //! This is because the [`RegisteredChainExtension::ID`] is used to decide which of those extensions
    //! should be used when the contract calls a chain extensions. Extensions which are generally
    //! useful should claim their `ID` with [the registry](https://github.com/paritytech/chainextension-registry)
    //! so that no collisions with other vendors will occur.
    //!
    //! **Chain specific extensions must use the reserved `ID = 0` so that they can't be registered with
    //! the registry.**
    //!
    //! # Security
    //!
    //! The chain author alone is responsible for the security of the chain extension.
    //! This includes avoiding the exposure of exploitable functions and charging the
    //! appropriate amount of weight. In order to do so benchmarks must be written and the
    //! [`charge_weight`](Environment::charge_weight) function must be called **before**
    //! carrying out any action that causes the consumption of the chargeable weight.
    //! It cannot be overstated how delicate of a process the creation of a chain extension
    //! is. Check whether using [`bare_call`](crate::Pallet::bare_call) suffices for the
    //! use case at hand.
    //!
    //! # Benchmarking
    //!
    //! The builtin contract callable functions that pallet-contracts provides all have
    //! benchmarks that determine the correct weight that an invocation of these functions
    //! induces. In order to be able to charge the correct weight for the functions defined
    //! by a chain extension benchmarks must be written, too. In the near future this crate
    //! will provide the means for easier creation of those specialized benchmarks.
    //!
    //! # Example
    //!
    //! The ink-examples repository maintains an
    //! [end-to-end example](https://github.com/paritytech/ink-examples/tree/main/rand-extension)
    //! on how to use a chain extension in order to provide new features to ink! contracts.
    use crate::{
        wasm::{Runtime, RuntimeCosts},
        Error,
    };
    use codec::{Decode, MaxEncodedLen};
    use frame_support::weights::Weight;
    use sp_runtime::DispatchError;
    use sp_std::{marker::PhantomData, vec::Vec};
    pub use crate::{exec::Ext, gas::ChargedAmount, Config};
    pub use frame_system::Config as SysConfig;
    pub use pallet_contracts_primitives::ReturnFlags;
    /// Result that returns a [`DispatchError`] on error.
    pub type Result<T> = sp_std::result::Result<T, DispatchError>;
    /// A trait used to extend the set of contract callable functions.
    ///
    /// In order to create a custom chain extension this trait must be implemented and supplied
    /// to the pallet contracts configuration trait as the associated type of the same name.
    /// Consult the [module documentation](self) for a general explanation of chain extensions.
    ///
    /// # Lifetime
    ///
    /// The extension will be [`Default`] initialized at the beginning of each call
    /// (**not** per call stack) and dropped afterwards. Hence any value held inside the extension
    /// can be used as a per-call scratch buffer.
    ///
    ///
    /// 用于扩展协定可调用函数集的特征。
    /// 为了创建自定义链扩展，必须实现此特征并将其作为同名的关联类型提供给托盘合同配置特征。
    /// 有关链扩展的一般说明，请参阅 模块文档 。
    /// LifeTime
    /// 扩展将在 Default 每次调用开始时初始化（而不是 每个调用堆栈），然后删除。
    /// 因此，扩展中保存的任何值都可以用作每次调用的暂存缓冲区。
    pub trait ChainExtension<C: Config> {
        /// Call the chain extension logic.
        ///
        /// This is the only function that needs to be implemented in order to write a
        /// chain extensions. It is called whenever a contract calls the `seal_call_chain_extension`
        /// imported wasm function.
        ///
        /// # Parameters
        /// - `env`: Access to the remaining arguments and the execution environment.
        ///
        /// # Return
        ///
        /// In case of `Err` the contract execution is immediately suspended and the passed error
        /// is returned to the caller. Otherwise the value of [`RetVal`] determines the exit
        /// behaviour.
        fn call<E: Ext<T = C>>(&mut self, env: Environment<E, InitState>) -> Result<RetVal>;
        /// Determines whether chain extensions are enabled for this chain.
        ///
        /// The default implementation returns `true`. Therefore it is not necessary to overwrite
        /// this function when implementing a chain extension. In case of `false` the deployment of
        /// a contract that references `seal_call_chain_extension` will be denied and calling this
        /// function will return [`NoChainExtension`](Error::NoChainExtension) without first calling
        /// into [`call`](Self::call).
        fn enabled() -> bool {
            true
        }
    }
    /// A [`ChainExtension`] that can be composed with other extensions using a tuple.
    ///
    /// An extension that implements this trait can be put in a tuple in order to have multiple
    /// extensions available. The tuple implementation routes requests based on the first two
    /// most significant bytes of the `id` passed to `call`.
    ///
    /// If this extensions is to be used by multiple runtimes consider
    /// [registering it](https://github.com/paritytech/chainextension-registry) to ensure that there
    /// are no collisions with other vendors.
    ///
    /// # Note
    ///
    /// Currently, we support tuples of up to ten registered chain extensions. If more chain extensions
    /// are needed consider opening an issue.
    pub trait RegisteredChainExtension<C: Config>: ChainExtension<C> {
        /// The extensions globally unique identifier.
        const ID: u16;
    }
    #[allow(unused)]
    impl<C: Config> ChainExtension<C> for () {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            false
        }
    }
    #[allow(unused)]
    impl<C: Config, TupleElement0: RegisteredChainExtension<C>> ChainExtension<C> for (TupleElement0,) {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
        > ChainExtension<C> for (TupleElement0, TupleElement1)
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
        > ChainExtension<C> for (TupleElement0, TupleElement1, TupleElement2)
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
        > ChainExtension<C> for (TupleElement0, TupleElement1, TupleElement2, TupleElement3)
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
            TupleElement4: RegisteredChainExtension<C>,
        > ChainExtension<C>
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
        )
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            if (TupleElement4::ID == env.ext_id()) && TupleElement4::enabled() {
                return self.4.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            if TupleElement4::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
            TupleElement4: RegisteredChainExtension<C>,
            TupleElement5: RegisteredChainExtension<C>,
        > ChainExtension<C>
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
        )
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            if (TupleElement4::ID == env.ext_id()) && TupleElement4::enabled() {
                return self.4.call(env);
            }
            if (TupleElement5::ID == env.ext_id()) && TupleElement5::enabled() {
                return self.5.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            if TupleElement4::enabled() {
                return true;
            }
            if TupleElement5::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
            TupleElement4: RegisteredChainExtension<C>,
            TupleElement5: RegisteredChainExtension<C>,
            TupleElement6: RegisteredChainExtension<C>,
        > ChainExtension<C>
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
        )
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            if (TupleElement4::ID == env.ext_id()) && TupleElement4::enabled() {
                return self.4.call(env);
            }
            if (TupleElement5::ID == env.ext_id()) && TupleElement5::enabled() {
                return self.5.call(env);
            }
            if (TupleElement6::ID == env.ext_id()) && TupleElement6::enabled() {
                return self.6.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            if TupleElement4::enabled() {
                return true;
            }
            if TupleElement5::enabled() {
                return true;
            }
            if TupleElement6::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
            TupleElement4: RegisteredChainExtension<C>,
            TupleElement5: RegisteredChainExtension<C>,
            TupleElement6: RegisteredChainExtension<C>,
            TupleElement7: RegisteredChainExtension<C>,
        > ChainExtension<C>
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
            TupleElement7,
        )
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            if (TupleElement4::ID == env.ext_id()) && TupleElement4::enabled() {
                return self.4.call(env);
            }
            if (TupleElement5::ID == env.ext_id()) && TupleElement5::enabled() {
                return self.5.call(env);
            }
            if (TupleElement6::ID == env.ext_id()) && TupleElement6::enabled() {
                return self.6.call(env);
            }
            if (TupleElement7::ID == env.ext_id()) && TupleElement7::enabled() {
                return self.7.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            if TupleElement4::enabled() {
                return true;
            }
            if TupleElement5::enabled() {
                return true;
            }
            if TupleElement6::enabled() {
                return true;
            }
            if TupleElement7::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
            TupleElement4: RegisteredChainExtension<C>,
            TupleElement5: RegisteredChainExtension<C>,
            TupleElement6: RegisteredChainExtension<C>,
            TupleElement7: RegisteredChainExtension<C>,
            TupleElement8: RegisteredChainExtension<C>,
        > ChainExtension<C>
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
            TupleElement7,
            TupleElement8,
        )
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            if (TupleElement4::ID == env.ext_id()) && TupleElement4::enabled() {
                return self.4.call(env);
            }
            if (TupleElement5::ID == env.ext_id()) && TupleElement5::enabled() {
                return self.5.call(env);
            }
            if (TupleElement6::ID == env.ext_id()) && TupleElement6::enabled() {
                return self.6.call(env);
            }
            if (TupleElement7::ID == env.ext_id()) && TupleElement7::enabled() {
                return self.7.call(env);
            }
            if (TupleElement8::ID == env.ext_id()) && TupleElement8::enabled() {
                return self.8.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            if TupleElement4::enabled() {
                return true;
            }
            if TupleElement5::enabled() {
                return true;
            }
            if TupleElement6::enabled() {
                return true;
            }
            if TupleElement7::enabled() {
                return true;
            }
            if TupleElement8::enabled() {
                return true;
            }
            false
        }
    }
    #[allow(unused)]
    impl<
            C: Config,
            TupleElement0: RegisteredChainExtension<C>,
            TupleElement1: RegisteredChainExtension<C>,
            TupleElement2: RegisteredChainExtension<C>,
            TupleElement3: RegisteredChainExtension<C>,
            TupleElement4: RegisteredChainExtension<C>,
            TupleElement5: RegisteredChainExtension<C>,
            TupleElement6: RegisteredChainExtension<C>,
            TupleElement7: RegisteredChainExtension<C>,
            TupleElement8: RegisteredChainExtension<C>,
            TupleElement9: RegisteredChainExtension<C>,
        > ChainExtension<C>
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
            TupleElement7,
            TupleElement8,
            TupleElement9,
        )
    {
        fn call<E: Ext<T = C>>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal> {
            if (TupleElement0::ID == env.ext_id()) && TupleElement0::enabled() {
                return self.0.call(env);
            }
            if (TupleElement1::ID == env.ext_id()) && TupleElement1::enabled() {
                return self.1.call(env);
            }
            if (TupleElement2::ID == env.ext_id()) && TupleElement2::enabled() {
                return self.2.call(env);
            }
            if (TupleElement3::ID == env.ext_id()) && TupleElement3::enabled() {
                return self.3.call(env);
            }
            if (TupleElement4::ID == env.ext_id()) && TupleElement4::enabled() {
                return self.4.call(env);
            }
            if (TupleElement5::ID == env.ext_id()) && TupleElement5::enabled() {
                return self.5.call(env);
            }
            if (TupleElement6::ID == env.ext_id()) && TupleElement6::enabled() {
                return self.6.call(env);
            }
            if (TupleElement7::ID == env.ext_id()) && TupleElement7::enabled() {
                return self.7.call(env);
            }
            if (TupleElement8::ID == env.ext_id()) && TupleElement8::enabled() {
                return self.8.call(env);
            }
            if (TupleElement9::ID == env.ext_id()) && TupleElement9::enabled() {
                return self.9.call(env);
            }
            Err(Error::<E::T>::NoChainExtension.into())
        }
        fn enabled() -> bool {
            if TupleElement0::enabled() {
                return true;
            }
            if TupleElement1::enabled() {
                return true;
            }
            if TupleElement2::enabled() {
                return true;
            }
            if TupleElement3::enabled() {
                return true;
            }
            if TupleElement4::enabled() {
                return true;
            }
            if TupleElement5::enabled() {
                return true;
            }
            if TupleElement6::enabled() {
                return true;
            }
            if TupleElement7::enabled() {
                return true;
            }
            if TupleElement8::enabled() {
                return true;
            }
            if TupleElement9::enabled() {
                return true;
            }
            false
        }
    }
    /// Determines the exit behaviour and return value of a chain extension.
    pub enum RetVal {
        /// The chain extensions returns the supplied value to its calling contract.
        Converging(u32),
        /// The control does **not** return to the calling contract.
        ///
        /// Use this to stop the execution of the contract when the chain extension returns.
        /// The semantic is the same as for calling `seal_return`: The control returns to
        /// the caller of the currently executing contract yielding the supplied buffer and
        /// flags.
        Diverging { flags: ReturnFlags, data: Vec<u8> },
    }
    /// Grants the chain extension access to its parameters and execution environment.
    ///
    /// It uses [typestate programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)
    /// to enforce the correct usage of the parameters passed to the chain extension.
    pub struct Environment<'a, 'b, E: Ext, S: State> {
        /// The actual data of this type.
        inner: Inner<'a, 'b, E>,
        /// `S` is only used in the type system but never as value.
        phantom: PhantomData<S>,
    }
    /// Functions that are available in every state of this type.
    impl<'a, 'b, E: Ext, S: State> Environment<'a, 'b, E, S> {
        /// The function id within the `id` passed by a contract.
        ///
        /// It returns the two least significant bytes of the `id` passed by a contract as the other
        /// two bytes represent the chain extension itself (the code which is calling this function).
        pub fn func_id(&self) -> u16 {
            (self.inner.id & 0x0000FFFF) as u16
        }
        /// The chain extension id within the `id` passed by a contract.
        ///
        /// It returns the two most significant bytes of the `id` passed by a contract which represent
        /// the chain extension itself (the code which is calling this function).
        pub fn ext_id(&self) -> u16 {
            (self.inner.id >> 16) as u16
        }
        /// Charge the passed `amount` of weight from the overall limit.
        ///
        /// It returns `Ok` when there the remaining weight budget is larger than the passed
        /// `weight`. It returns `Err` otherwise. In this case the chain extension should
        /// abort the execution and pass through the error.
        ///
        /// The returned value can be used to with [`Self::adjust_weight`]. Other than that
        /// it has no purpose.
        ///
        /// # Note
        ///
        /// Weight is synonymous with gas in substrate.
        pub fn charge_weight(&mut self, amount: Weight) -> Result<ChargedAmount> {
            self.inner
                .runtime
                .charge_gas(RuntimeCosts::ChainExtension(amount))
        }
        /// Adjust a previously charged amount down to its actual amount.
        ///
        /// This is when a maximum a priori amount was charged and then should be partially
        /// refunded to match the actual amount.
        pub fn adjust_weight(&mut self, charged: ChargedAmount, actual_weight: Weight) {
            self.inner
                .runtime
                .adjust_gas(charged, RuntimeCosts::ChainExtension(actual_weight))
        }
        /// Grants access to the execution environment of the current contract call.
        ///
        /// Consult the functions on the returned type before re-implementing those functions.
        pub fn ext(&mut self) -> &mut E {
            self.inner.runtime.ext()
        }
    }
    /// Functions that are only available in the initial state of this type.
    ///
    /// Those are the functions that determine how the arguments to the chain extensions
    /// should be consumed.
    impl<'a, 'b, E: Ext> Environment<'a, 'b, E, InitState> {
        /// Creates a new environment for consumption by a chain extension.
        ///
        /// It is only available to this crate because only the wasm runtime module needs to
        /// ever create this type. Chain extensions merely consume it.
        pub(crate) fn new(
            runtime: &'a mut Runtime<'b, E>,
            memory: &'a mut [u8],
            id: u32,
            input_ptr: u32,
            input_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> Self {
            Environment {
                inner: Inner {
                    runtime,
                    memory,
                    id,
                    input_ptr,
                    input_len,
                    output_ptr,
                    output_len_ptr,
                },
                phantom: PhantomData,
            }
        }
        /// Use all arguments as integer values.
        pub fn only_in(self) -> Environment<'a, 'b, E, OnlyInState> {
            Environment {
                inner: self.inner,
                phantom: PhantomData,
            }
        }
        /// Use input arguments as integer and output arguments as pointer to a buffer.
        pub fn prim_in_buf_out(self) -> Environment<'a, 'b, E, PrimInBufOutState> {
            Environment {
                inner: self.inner,
                phantom: PhantomData,
            }
        }
        /// Use input and output arguments as pointers to a buffer.
        pub fn buf_in_buf_out(self) -> Environment<'a, 'b, E, BufInBufOutState> {
            Environment {
                inner: self.inner,
                phantom: PhantomData,
            }
        }
    }
    /// Functions to use the input arguments as integers.
    impl<'a, 'b, E: Ext, S: PrimIn> Environment<'a, 'b, E, S> {
        /// The `input_ptr` argument.
        pub fn val0(&self) -> u32 {
            self.inner.input_ptr
        }
        /// The `input_len` argument.
        pub fn val1(&self) -> u32 {
            self.inner.input_len
        }
    }
    /// Functions to use the output arguments as integers.
    impl<'a, 'b, E: Ext, S: PrimOut> Environment<'a, 'b, E, S> {
        /// The `output_ptr` argument.
        pub fn val2(&self) -> u32 {
            self.inner.output_ptr
        }
        /// The `output_len_ptr` argument.
        pub fn val3(&self) -> u32 {
            self.inner.output_len_ptr
        }
    }
    /// Functions to use the input arguments as pointer to a buffer.
    impl<'a, 'b, E: Ext, S: BufIn> Environment<'a, 'b, E, S> {
        /// Reads `min(max_len, in_len)` from contract memory.
        ///
        /// This does **not** charge any weight. The caller must make sure that the an
        /// appropriate amount of weight is charged **before** reading from contract memory.
        /// The reason for that is that usually the costs for reading data and processing
        /// said data cannot be separated in a benchmark. Therefore a chain extension would
        /// charge the overall costs either using `max_len` (worst case approximation) or using
        /// [`in_len()`](Self::in_len).
        pub fn read(&self, max_len: u32) -> Result<Vec<u8>> {
            self.inner.runtime.read_sandbox_memory(
                self.inner.memory,
                self.inner.input_ptr,
                self.inner.input_len.min(max_len),
            )
        }
        /// Reads `min(buffer.len(), in_len) from contract memory.
        ///
        /// This takes a mutable pointer to a buffer fills it with data and shrinks it to
        /// the size of the actual data. Apart from supporting pre-allocated buffers it is
        /// equivalent to to [`read()`](Self::read).
        pub fn read_into(&self, buffer: &mut &mut [u8]) -> Result<()> {
            let len = buffer.len();
            let sliced = {
                let buffer = core::mem::take(buffer);
                &mut buffer[..len.min(self.inner.input_len as usize)]
            };
            self.inner.runtime.read_sandbox_memory_into_buf(
                self.inner.memory,
                self.inner.input_ptr,
                sliced,
            )?;
            *buffer = sliced;
            Ok(())
        }
        /// Reads and decodes a type with a size fixed at compile time from contract memory.
        ///
        /// This function is secure and recommended for all input types of fixed size
        /// as long as the cost of reading the memory is included in the overall already charged
        /// weight of the chain extension. This should usually be the case when fixed input types
        /// are used.
        pub fn read_as<T: Decode + MaxEncodedLen>(&mut self) -> Result<T> {
            self.inner
                .runtime
                .read_sandbox_memory_as(self.inner.memory, self.inner.input_ptr)
        }
        /// Reads and decodes a type with a dynamic size from contract memory.
        ///
        /// Make sure to include `len` in your weight calculations.
        pub fn read_as_unbounded<T: Decode>(&mut self, len: u32) -> Result<T> {
            self.inner.runtime.read_sandbox_memory_as_unbounded(
                self.inner.memory,
                self.inner.input_ptr,
                len,
            )
        }
        /// The length of the input as passed in as `input_len`.
        ///
        /// A chain extension would use this value to calculate the dynamic part of its
        /// weight. For example a chain extension that calculates the hash of some passed in
        /// bytes would use `in_len` to charge the costs of hashing that amount of bytes.
        /// This also subsumes the act of copying those bytes as a benchmarks measures both.
        pub fn in_len(&self) -> u32 {
            self.inner.input_len
        }
    }
    /// Functions to use the output arguments as pointer to a buffer.
    impl<'a, 'b, E: Ext, S: BufOut> Environment<'a, 'b, E, S> {
        /// Write the supplied buffer to contract memory.
        ///
        /// If the contract supplied buffer is smaller than the passed `buffer` an `Err` is returned.
        /// If `allow_skip` is set to true the contract is allowed to skip the copying of the buffer
        /// by supplying the guard value of `pallet-contracts::SENTINEL` as `out_ptr`. The
        /// `weight_per_byte` is only charged when the write actually happens and is not skipped or
        /// failed due to a too small output buffer.
        pub fn write(
            &mut self,
            buffer: &[u8],
            allow_skip: bool,
            weight_per_byte: Option<Weight>,
        ) -> Result<()> {
            self.inner.runtime.write_sandbox_output(
                self.inner.memory,
                self.inner.output_ptr,
                self.inner.output_len_ptr,
                buffer,
                allow_skip,
                |len| {
                    weight_per_byte
                        .map(|w| RuntimeCosts::ChainExtension(w.saturating_mul(len.into())))
                },
            )
        }
    }
    /// The actual data of an `Environment`.
    ///
    /// All data is put into this struct to easily pass it around as part of the typestate
    /// pattern. Also it creates the opportunity to box this struct in the future in case it
    /// gets too large.
    ///
    /// 实际的 Environment数据。
    /// 所有数据都放入此结构中，以便将其作为类型状态模式的一部分轻松传递。
    /// 此外，它还创造了将来将此结构框起来的机会，以防它变得太大
    struct Inner<'a, 'b, E: Ext> {
        /// The runtime contains all necessary functions to interact with the running contract.
        runtime: &'a mut Runtime<'b, E>,
        /// Reference to the contracts memory.
        memory: &'a mut [u8],
        /// Verbatim argument passed to `seal_call_chain_extension`.
        id: u32,
        /// Verbatim argument passed to `seal_call_chain_extension`.
        input_ptr: u32,
        /// Verbatim argument passed to `seal_call_chain_extension`.
        input_len: u32,
        /// Verbatim argument passed to `seal_call_chain_extension`.
        output_ptr: u32,
        /// Verbatim argument passed to `seal_call_chain_extension`.
        output_len_ptr: u32,
    }
    /// Any state of an [`Environment`] implements this trait.
    /// See [typestate programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html).
    pub trait State: sealed::Sealed {}
    /// A state that uses primitive inputs.
    pub trait PrimIn: State {}
    /// A state that uses primitive outputs.
    pub trait PrimOut: State {}
    /// A state that uses a buffer as input.
    pub trait BufIn: State {}
    /// A state that uses a buffer as output.
    pub trait BufOut: State {}
    /// The initial state of an [`Environment`].
    pub enum InitState {}
    /// A state that uses all arguments as primitive inputs.
    pub enum OnlyInState {}
    /// A state that uses two arguments as primitive inputs and the other two as buffer output.
    pub enum PrimInBufOutState {}
    /// Uses a buffer for input and a buffer for output.
    pub enum BufInBufOutState {}
    mod sealed {
        use super::*;
        /// Trait to prevent users from implementing `State` for anything else.
        pub trait Sealed {}
        impl Sealed for InitState {}
        impl Sealed for OnlyInState {}
        impl Sealed for PrimInBufOutState {}
        impl Sealed for BufInBufOutState {}
        impl State for InitState {}
        impl State for OnlyInState {}
        impl State for PrimInBufOutState {}
        impl State for BufInBufOutState {}
        impl PrimIn for OnlyInState {}
        impl PrimOut for OnlyInState {}
        impl PrimIn for PrimInBufOutState {}
        impl BufOut for PrimInBufOutState {}
        impl BufIn for BufInBufOutState {}
        impl BufOut for BufInBufOutState {}
    }
}
