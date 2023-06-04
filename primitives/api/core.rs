#![feature(prelude_import)]
//! Substrate runtime api
//!
//! The Substrate runtime api is the interface between the node and the runtime. There isn't a fixed
//! set of runtime apis, instead it is up to the user to declare and implement these runtime apis.
//! The declaration of a runtime api is normally done outside of a runtime, while the implementation
//! of it has to be done in the runtime. We provide the [`decl_runtime_apis!`] macro for declaring
//! a runtime api and the [`impl_runtime_apis!`] for implementing them. The macro docs provide more
//! information on how to use them and what kind of attributes we support.
//!
//! It is required that each runtime implements at least the [`Core`] runtime api. This runtime api
//! provides all the core functions that Substrate expects from a runtime.
//!
//! # Versioning
//!
//! Runtime apis support versioning. Each runtime api itself has a version attached. It is also
//! supported to change function signatures or names in a non-breaking way. For more information on
//! versioning check the [`decl_runtime_apis!`] macro.
//!
//! All runtime apis and their versions are returned as part of the [`RuntimeVersion`]. This can be
//! used to check which runtime api version is currently provided by the on-chain runtime.
//!
//! # Testing
//!
//! For testing we provide the [`mock_impl_runtime_apis!`] macro that lets you implement a runtime
//! api for a mocked object to use it in tests.
//!
//! # Logging
//!
//! Substrate supports logging from the runtime in native and in wasm. For that purpose it provides
//! the [`RuntimeLogger`](sp_runtime::runtime_logger::RuntimeLogger). This runtime logger is
//! automatically enabled for each call into the runtime through the runtime api. As logging
//! introduces extra code that isn't actually required for the logic of your runtime and also
//! increases the final wasm blob size, it is recommended to disable the logging for on-chain
//! wasm blobs. This can be done by enabling the `disable-logging` feature of this crate. Be aware
//! that this feature instructs `log` and `tracing` to disable logging at compile time by setting
//! the `max_level_off` feature for these crates. So, you should not enable this feature for a
//! native build as otherwise the node will not output any log messages.
//!
//! # How does it work?
//!
//! Each runtime api is declared as a trait with functions. When compiled to WASM, each implemented
//! runtime api function is exported as a function with the following naming scheme
//! `${TRAIT_NAME}_${FUNCTION_NAME}`. Such a function has the following signature
//! `(ptr: *u8, length: u32) -> u64`. It takes a pointer to an `u8` array and its length as an
//! argument. This `u8` array is expected to be the SCALE encoded parameters of the function as
//! defined in the trait. The return value is an `u64` that represents `length << 32 | pointer` of
//! an `u8` array. This return value `u8` array contains the SCALE encoded return value as defined
//! by the trait function. The macros take care to encode the parameters and to decode the return
//! value.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate self as sp_api;
#[doc(hidden)]
pub use codec::{self, Decode, DecodeLimit, Encode};
#[doc(hidden)]
#[cfg(feature = "std")]
pub use hash_db::Hasher;
#[doc(hidden)]
pub use scale_info;
use sp_core::OpaqueMetadata;
#[doc(hidden)]
pub use sp_core::{offchain, ExecutionContext};
#[doc(hidden)]
pub use sp_metadata_ir::{self as metadata_ir, frame_metadata as metadata};
#[doc(hidden)]
#[cfg(feature = "std")]
pub use sp_runtime::StateVersion;
#[doc(hidden)]
pub use sp_runtime::{
    generic::BlockId,
    traits::{
        Block as BlockT, GetNodeBlockType, GetRuntimeBlockType, Hash as HashT, HashFor,
        Header as HeaderT, NumberFor,
    },
    transaction_validity::TransactionValidity,
    RuntimeString, TransactionOutcome,
};
#[doc(hidden)]
#[cfg(feature = "std")]
pub use sp_state_machine::{
    backend::AsTrieBackend, Backend as StateBackend, InMemoryBackend, OverlayedChanges,
    StorageProof, TrieBackend, TrieBackendBuilder,
};
#[doc(hidden)]
pub use sp_std::{mem, slice, vec};
#[doc(hidden)]
pub use sp_version::{create_apis_vec, ApiId, ApisVec, RuntimeVersion};
#[cfg(feature = "std")]
use std::cell::RefCell;
/// Maximum nesting level for extrinsics.
pub const MAX_EXTRINSIC_DEPTH: u32 = 256;
/// Declares given traits as runtime apis.
///
/// The macro will create two declarations, one for using on the client side and one for using
/// on the runtime side. The declaration for the runtime side is hidden in its own module.
/// The client side declaration gets two extra parameters per function,
/// `&self` and `at: Block::Hash`. The runtime side declaration will match the given trait
/// declaration. Besides one exception, the macro adds an extra generic parameter `Block:
/// BlockT` to the client side and the runtime side. This generic parameter is usable by the
/// user.
///
/// For implementing these macros you should use the
/// [`impl_runtime_apis!`] macro.
///
/// # Example
///
/// ```rust
/// sp_api::decl_runtime_apis! {
///     /// Declare the api trait.
///     pub trait Balance {
///         /// Get the balance.
///         fn get_balance() -> u64;
///         /// Set the balance.
///         fn set_balance(val: u64);
///     }
///
///     /// You can declare multiple api traits in one macro call.
///     /// In one module you can call the macro at maximum one time.
///     pub trait BlockBuilder {
///         /// The macro adds an explicit `Block: BlockT` generic parameter for you.
///         /// You can use this generic parameter as you would defined it manually.
///         fn build_block() -> Block;
///     }
/// }
///
/// # fn main() {}
/// ```
///
/// # Runtime api trait versioning
///
/// To support versioning of the traits, the macro supports the attribute `#[api_version(1)]`.
/// The attribute supports any `u32` as version. By default, each trait is at version `1`, if
/// no version is provided. We also support changing the signature of a method. This signature
/// change is highlighted with the `#[changed_in(2)]` attribute above a method. A method that
/// is tagged with this attribute is callable by the name `METHOD_before_version_VERSION`. This
/// method will only support calling into wasm, trying to call into native will fail (change
/// the spec version!). Such a method also does not need to be implemented in the runtime. It
/// is required that there exist the "default" of the method without the `#[changed_in(_)]`
/// attribute, this method will be used to call the current default implementation.
///
/// ```rust
/// sp_api::decl_runtime_apis! {
///     /// Declare the api trait.
///     #[api_version(2)]
///     pub trait Balance {
///         /// Get the balance.
///         fn get_balance() -> u64;
///         /// Set balance.
///         fn set_balance(val: u64);
///         /// Set balance, old version.
///         ///
///         /// Is callable by `set_balance_before_version_2`.
///         #[changed_in(2)]
///         fn set_balance(val: u16);
///         /// In version 2, we added this new function.
///         fn increase_balance(val: u64);
///     }
/// }
///
/// # fn main() {}
/// ```
///
/// To check if a given runtime implements a runtime api trait, the `RuntimeVersion` has the
/// function `has_api<A>()`. Also the `ApiExt` provides a function `has_api<A>(at: Hash)`
/// to check if the runtime at the given block id implements the requested runtime api trait.
///
/// # Declaring multiple api versions
///
/// Optionally multiple versions of the same api can be declared. This is useful for
/// development purposes. For example you want to have a testing version of the api which is
/// available only on a testnet. You can define one stable and one development version. This
/// can be done like this:
/// ```rust
/// sp_api::decl_runtime_apis! {
///     /// Declare the api trait.
/// 	#[api_version(2)]
///     pub trait Balance {
///         /// Get the balance.
///         fn get_balance() -> u64;
///         /// Set the balance.
///         fn set_balance(val: u64);
///         /// Transfer the balance to another user id
///         #[api_version(3)]
///         fn transfer_balance(uid: u64);
///     }
/// }
///
/// # fn main() {}
/// ```
/// The example above defines two api versions - 2 and 3. Version 2 contains `get_balance` and
/// `set_balance`. Version 3 additionally contains `transfer_balance`, which is not available
/// in version 2. Version 2 in this case is considered the default/base version of the api.
/// More than two versions can be defined this way. For example:
/// ```rust
/// sp_api::decl_runtime_apis! {
///     /// Declare the api trait.
///     #[api_version(2)]
///     pub trait Balance {
///         /// Get the balance.
///         fn get_balance() -> u64;
///         /// Set the balance.
///         fn set_balance(val: u64);
///         /// Transfer the balance to another user id
///         #[api_version(3)]
///         fn transfer_balance(uid: u64);
///         /// Clears the balance
///         #[api_version(4)]
///         fn clear_balance();
///     }
/// }
///
/// # fn main() {}
/// ```
/// Note that the latest version (4 in our example above) always contains all methods from all
/// the versions before.
pub use sp_api_proc_macro::decl_runtime_apis;
/// Tags given trait implementations as runtime apis.
///
/// All traits given to this macro, need to be declared with the
/// [`decl_runtime_apis!`](macro.decl_runtime_apis.html) macro. The implementation of the trait
/// should follow the declaration given to the
/// [`decl_runtime_apis!`](macro.decl_runtime_apis.html) macro, besides the `Block` type that
/// is required as first generic parameter for each runtime api trait. When implementing a
/// runtime api trait, it is required that the trait is referenced by a path, e.g. `impl
/// my_trait::MyTrait for Runtime`. The macro will use this path to access the declaration of
/// the trait for the runtime side.
///
/// The macro also generates the api implementations for the client side and provides it
/// through the `RuntimeApi` type. The `RuntimeApi` is hidden behind a `feature` called `std`.
///
/// To expose version information about all implemented api traits, the constant
/// `RUNTIME_API_VERSIONS` is generated. This constant should be used to instantiate the `apis`
/// field of `RuntimeVersion`.
///
/// # Example
///
/// ```rust
/// use sp_version::create_runtime_str;
/// #
/// # use sp_runtime::traits::{GetNodeBlockType, Block as BlockT};
/// # use sp_test_primitives::Block;
/// #
/// # /// The declaration of the `Runtime` type and the implementation of the `GetNodeBlockType`
/// # /// trait are done by the `construct_runtime!` macro in a real runtime.
/// # pub struct Runtime {}
/// # impl GetNodeBlockType for Runtime {
/// #     type NodeBlock = Block;
/// # }
/// #
/// # sp_api::decl_runtime_apis! {
/// #     /// Declare the api trait.
/// #     pub trait Balance {
/// #         /// Get the balance.
/// #         fn get_balance() -> u64;
/// #         /// Set the balance.
/// #         fn set_balance(val: u64);
/// #     }
/// #     pub trait BlockBuilder {
/// #        fn build_block() -> Block;
/// #     }
/// # }
///
/// /// All runtime api implementations need to be done in one call of the macro!
/// sp_api::impl_runtime_apis! {
/// #   impl sp_api::Core<Block> for Runtime {
/// #       fn version() -> sp_version::RuntimeVersion {
/// #           unimplemented!()
/// #       }
/// #       fn execute_block(_block: Block) {}
/// #       fn initialize_block(_header: &<Block as BlockT>::Header) {}
/// #   }
///
///     impl self::Balance<Block> for Runtime {
///         fn get_balance() -> u64 {
///             1
///         }
///         fn set_balance(_bal: u64) {
///             // Store the balance
///         }
///     }
///
///     impl self::BlockBuilder<Block> for Runtime {
///         fn build_block() -> Block {
///              unimplemented!("Please implement me!")
///         }
///     }
/// }
///
/// /// Runtime version. This needs to be declared for each runtime.
/// pub const VERSION: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
///     spec_name: create_runtime_str!("node"),
///     impl_name: create_runtime_str!("test-node"),
///     authoring_version: 1,
///     spec_version: 1,
///     impl_version: 0,
///     // Here we are exposing the runtime api versions.
///     apis: RUNTIME_API_VERSIONS,
///     transaction_version: 1,
///     state_version: 1,
/// };
///
/// # fn main() {}
/// ```
///
/// # Implementing specific api version
///
/// If `decl_runtime_apis!` declares multiple versions for an api `impl_runtime_apis!`
/// should specify which version it implements by adding `api_version` attribute to the
/// `impl` block. If omitted - the base/default version is implemented. Here is an example:
/// ```ignore
/// sp_api::impl_runtime_apis! {
///     #[api_version(3)]
///     impl self::Balance<Block> for Runtime {
///          // implementation
///     }
/// }
/// ```
/// In this case `Balance` api version 3 is being implemented for `Runtime`. The `impl` block
/// must contain all methods declared in version 3 and below.
pub use sp_api_proc_macro::impl_runtime_apis;
/// Mocks given trait implementations as runtime apis.
///
/// Accepts similar syntax as [`impl_runtime_apis!`] and generates
/// simplified mock implementations of the given runtime apis. The difference in syntax is that
/// the trait does not need to be referenced by a qualified path, methods accept the `&self`
/// parameter and the error type can be specified as associated type. If no error type is
/// specified [`String`] is used as error type.
///
/// Besides implementing the given traits, the [`Core`](sp_api::Core) and
/// [`ApiExt`](sp_api::ApiExt) are implemented automatically.
///
/// # Example
///
/// ```rust
/// # use sp_runtime::traits::Block as BlockT;
/// # use sp_test_primitives::Block;
/// #
/// # sp_api::decl_runtime_apis! {
/// #     /// Declare the api trait.
/// #     pub trait Balance {
/// #         /// Get the balance.
/// #         fn get_balance() -> u64;
/// #         /// Set the balance.
/// #         fn set_balance(val: u64);
/// #     }
/// #     pub trait BlockBuilder {
/// #        fn build_block() -> Block;
/// #     }
/// # }
/// struct MockApi {
///     balance: u64,
/// }
///
/// /// All runtime api mock implementations need to be done in one call of the macro!
/// sp_api::mock_impl_runtime_apis! {
///     impl Balance<Block> for MockApi {
///         /// Here we take the `&self` to access the instance.
///         fn get_balance(&self) -> u64 {
///             self.balance
///         }
///         fn set_balance(_bal: u64) {
///             // Store the balance
///         }
///     }
///
///     impl BlockBuilder<Block> for MockApi {
///         fn build_block() -> Block {
///              unimplemented!("Not Required in tests")
///         }
///     }
/// }
///
/// # fn main() {}
/// ```
///
/// # `advanced` attribute
///
/// This attribute can be placed above individual function in the mock implementation to
/// request more control over the function declaration. From the client side each runtime api
/// function is called with the `at` parameter that is a [`Hash`](sp_runtime::traits::Hash).
/// When using the `advanced` attribute, the macro expects that the first parameter of the
/// function is this `at` parameter. Besides that the macro also doesn't do the automatic
/// return value rewrite, which means that full return value must be specified. The full return
/// value is constructed like [`Result`]`<<ReturnValue>, Error>` while `ReturnValue` being the
/// return value that is specified in the trait declaration.
///
/// ## Example
/// ```rust
/// # use sp_runtime::traits::Block as BlockT;
/// # use sp_test_primitives::Block;
/// # use codec;
/// #
/// # sp_api::decl_runtime_apis! {
/// #     /// Declare the api trait.
/// #     pub trait Balance {
/// #         /// Get the balance.
/// #         fn get_balance() -> u64;
/// #         /// Set the balance.
/// #         fn set_balance(val: u64);
/// #     }
/// # }
/// struct MockApi {
///     balance: u64,
/// }
///
/// sp_api::mock_impl_runtime_apis! {
///     impl Balance<Block> for MockApi {
///         #[advanced]
///         fn get_balance(&self, at: <Block as BlockT>::Hash) -> Result<u64, sp_api::ApiError> {
///             println!("Being called at: {}", at);
///
///             Ok(self.balance.into())
///         }
///         #[advanced]
///         fn set_balance(at: <Block as BlockT>::Hash, val: u64) -> Result<(), sp_api::ApiError> {
///             println!("Being called at: {}", at);
///
///             Ok(().into())
///         }
///     }
/// }
///
/// # fn main() {}
/// ```
pub use sp_api_proc_macro::mock_impl_runtime_apis;
/// A type that records all accessed trie nodes and generates a proof out of it.
#[cfg(feature = "std")]
pub type ProofRecorder<B> = sp_trie::recorder::Recorder<HashFor<B>>;
/// A type that is used as cache for the storage transactions.
#[cfg(feature = "std")]
pub type StorageTransactionCache<Block, Backend> = sp_state_machine::StorageTransactionCache<
    <Backend as StateBackend<HashFor<Block>>>::Transaction,
    HashFor<Block>,
>;
#[cfg(feature = "std")]
pub type StorageChanges<SBackend, Block> = sp_state_machine::StorageChanges<
    <SBackend as StateBackend<HashFor<Block>>>::Transaction,
    HashFor<Block>,
>;
/// Extract the state backend type for a type that implements `ProvideRuntimeApi`.
#[cfg(feature = "std")]
pub type StateBackendFor<P, Block> =
    <<P as ProvideRuntimeApi<Block>>::Api as ApiExt<Block>>::StateBackend;
/// Extract the state backend transaction type for a type that implements `ProvideRuntimeApi`.
#[cfg(feature = "std")]
pub type TransactionFor<P, Block> =
    <StateBackendFor<P, Block> as StateBackend<HashFor<Block>>>::Transaction;
/// Something that can be constructed to a runtime api.
#[cfg(feature = "std")]
pub trait ConstructRuntimeApi<Block: BlockT, C: CallApiAt<Block>> {
    /// The actual runtime api that will be constructed.
    type RuntimeApi: ApiExt<Block>;
    /// Construct an instance of the runtime api.
    fn construct_runtime_api(call: &C) -> ApiRef<Self::RuntimeApi>;
}
/// Init the [`RuntimeLogger`](sp_runtime::runtime_logger::RuntimeLogger).
pub fn init_runtime_logger() {
    #[cfg(not(feature = "disable-logging"))]
    sp_runtime::runtime_logger::RuntimeLogger::init();
}
/// An error describing which API call failed.
#[cfg(feature = "std")]
pub enum ApiError {
    #[error("Failed to decode return value of {function}")]
    FailedToDecodeReturnValue {
        function: &'static str,
        #[source]
        error: codec::Error,
    },
    #[error("Failed to convert return value from runtime to node of {function}")]
    FailedToConvertReturnValue {
        function: &'static str,
        #[source]
        error: codec::Error,
    },
    #[error("Failed to convert parameter `{parameter}` from node to runtime of {function}")]
    FailedToConvertParameter {
        function: &'static str,
        parameter: &'static str,
        #[source]
        error: codec::Error,
    },
    #[error("The given `StateBackend` isn't a `TrieBackend`.")]
    StateBackendIsNotTrie,
    #[error(transparent)]
    Application(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Api called for an unknown Block: {0}")]
    UnknownBlock(String),
}
#[automatically_derived]
impl ::core::fmt::Debug for ApiError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ApiError::FailedToDecodeReturnValue {
                function: __self_0,
                error: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "FailedToDecodeReturnValue",
                "function",
                &__self_0,
                "error",
                &__self_1,
            ),
            ApiError::FailedToConvertReturnValue {
                function: __self_0,
                error: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "FailedToConvertReturnValue",
                "function",
                &__self_0,
                "error",
                &__self_1,
            ),
            ApiError::FailedToConvertParameter {
                function: __self_0,
                parameter: __self_1,
                error: __self_2,
            } => ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "FailedToConvertParameter",
                "function",
                &__self_0,
                "parameter",
                &__self_1,
                "error",
                &__self_2,
            ),
            ApiError::StateBackendIsNotTrie => {
                ::core::fmt::Formatter::write_str(f, "StateBackendIsNotTrie")
            }
            ApiError::Application(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Application", &__self_0)
            }
            ApiError::UnknownBlock(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "UnknownBlock", &__self_0)
            }
        }
    }
}
#[allow(unused_qualifications)]
impl std::error::Error for ApiError {
    fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
        use thiserror::__private::AsDynError;
        #[allow(deprecated)]
        match self {
            ApiError::FailedToDecodeReturnValue { error: source, .. } => {
                std::option::Option::Some(source.as_dyn_error())
            }
            ApiError::FailedToConvertReturnValue { error: source, .. } => {
                std::option::Option::Some(source.as_dyn_error())
            }
            ApiError::FailedToConvertParameter { error: source, .. } => {
                std::option::Option::Some(source.as_dyn_error())
            }
            ApiError::StateBackendIsNotTrie { .. } => std::option::Option::None,
            ApiError::Application { 0: transparent } => {
                std::error::Error::source(transparent.as_dyn_error())
            }
            ApiError::UnknownBlock { .. } => std::option::Option::None,
        }
    }
}
#[allow(unused_qualifications)]
impl std::fmt::Display for ApiError {
    fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[allow(unused_imports)]
        use thiserror::__private::{DisplayAsDisplay, PathAsDisplay};
        #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
        match self {
            ApiError::FailedToDecodeReturnValue { function, error } => {
                __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Failed to decode return value of "],
                    &[::core::fmt::ArgumentV1::new_display(&function.as_display())],
                ))
            }
            ApiError::FailedToConvertReturnValue { function, error } => {
                __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Failed to convert return value from runtime to node of "],
                    &[::core::fmt::ArgumentV1::new_display(&function.as_display())],
                ))
            }
            ApiError::FailedToConvertParameter {
                function,
                parameter,
                error,
            } => __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                &[
                    "Failed to convert parameter `",
                    "` from node to runtime of ",
                ],
                &[
                    ::core::fmt::ArgumentV1::new_display(&parameter.as_display()),
                    ::core::fmt::ArgumentV1::new_display(&function.as_display()),
                ],
            )),
            ApiError::StateBackendIsNotTrie {} => {
                __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                    &["The given `StateBackend` isn\'t a `TrieBackend`."],
                    &[],
                ))
            }
            ApiError::Application(_0) => std::fmt::Display::fmt(_0, __formatter),
            ApiError::UnknownBlock(_0) => __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                &["Api called for an unknown Block: "],
                &[::core::fmt::ArgumentV1::new_display(&_0.as_display())],
            )),
        }
    }
}
#[allow(unused_qualifications)]
impl std::convert::From<Box<dyn std::error::Error + Send + Sync>> for ApiError {
    #[allow(deprecated)]
    fn from(source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ApiError::Application { 0: source }
    }
}
/// Extends the runtime api implementation with some common functionality.
#[cfg(feature = "std")]
pub trait ApiExt<Block: BlockT> {
    /// The state backend that is used to store the block states.
    type StateBackend: StateBackend<HashFor<Block>>;
    /// Execute the given closure inside a new transaction.
    ///
    /// Depending on the outcome of the closure, the transaction is committed or rolled-back.
    ///
    /// The internal result of the closure is returned afterwards.
    fn execute_in_transaction<F: FnOnce(&Self) -> TransactionOutcome<R>, R>(&self, call: F) -> R
    where
        Self: Sized;
    /// Checks if the given api is implemented and versions match.
    fn has_api<A: RuntimeApiInfo + ?Sized>(&self, at_hash: Block::Hash) -> Result<bool, ApiError>
    where
        Self: Sized;
    /// Check if the given api is implemented and the version passes a predicate.
    fn has_api_with<A: RuntimeApiInfo + ?Sized, P: Fn(u32) -> bool>(
        &self,
        at_hash: Block::Hash,
        pred: P,
    ) -> Result<bool, ApiError>
    where
        Self: Sized;
    /// Returns the version of the given api.
    fn api_version<A: RuntimeApiInfo + ?Sized>(
        &self,
        at_hash: Block::Hash,
    ) -> Result<Option<u32>, ApiError>
    where
        Self: Sized;
    /// Start recording all accessed trie nodes for generating proofs.
    fn record_proof(&mut self);
    /// Extract the recorded proof.
    ///
    /// This stops the proof recording.
    ///
    /// If `record_proof` was not called before, this will return `None`.
    fn extract_proof(&mut self) -> Option<StorageProof>;
    /// Returns the current active proof recorder.
    fn proof_recorder(&self) -> Option<ProofRecorder<Block>>;
    /// Convert the api object into the storage changes that were done while executing runtime
    /// api functions.
    ///
    /// After executing this function, all collected changes are reset.
    fn into_storage_changes(
        &self,
        backend: &Self::StateBackend,
        parent_hash: Block::Hash,
    ) -> Result<StorageChanges<Self::StateBackend, Block>, String>
    where
        Self: Sized;
}
/// Parameters for [`CallApiAt::call_api_at`].
#[cfg(feature = "std")]
pub struct CallApiAtParams<'a, Block: BlockT, Backend: StateBackend<HashFor<Block>>> {
    /// The block id that determines the state that should be setup when calling the function.
    pub at: Block::Hash,
    /// The name of the function that should be called.
    pub function: &'static str,
    /// The encoded arguments of the function.
    pub arguments: Vec<u8>,
    /// The overlayed changes that are on top of the state.
    pub overlayed_changes: &'a RefCell<OverlayedChanges>,
    /// The cache for storage transactions.
    pub storage_transaction_cache: &'a RefCell<StorageTransactionCache<Block, Backend>>,
    /// The context this function is executed in.
    pub context: ExecutionContext,
    /// The optional proof recorder for recording storage accesses.
    pub recorder: &'a Option<ProofRecorder<Block>>,
}
/// Something that can call into the an api at a given block.
#[cfg(feature = "std")]
pub trait CallApiAt<Block: BlockT> {
    /// The state backend that is used to store the block states.
    type StateBackend: StateBackend<HashFor<Block>> + AsTrieBackend<HashFor<Block>>;
    /// Calls the given api function with the given encoded arguments at the given block and returns
    /// the encoded result.
    fn call_api_at(
        &self,
        params: CallApiAtParams<Block, Self::StateBackend>,
    ) -> Result<Vec<u8>, ApiError>;
    /// Returns the runtime version at the given block.
    fn runtime_version_at(&self, at_hash: Block::Hash) -> Result<RuntimeVersion, ApiError>;
    /// Get the state `at` the given block.
    fn state_at(&self, at: Block::Hash) -> Result<Self::StateBackend, ApiError>;
}
/// Auxiliary wrapper that holds an api instance and binds it to the given lifetime.
#[cfg(feature = "std")]
pub struct ApiRef<'a, T>(T, std::marker::PhantomData<&'a ()>);
#[cfg(feature = "std")]
impl<'a, T> From<T> for ApiRef<'a, T> {
    fn from(api: T) -> Self {
        ApiRef(api, Default::default())
    }
}
#[cfg(feature = "std")]
impl<'a, T> std::ops::Deref for ApiRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(feature = "std")]
impl<'a, T> std::ops::DerefMut for ApiRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
/// Something that provides a runtime api.
#[cfg(feature = "std")]
pub trait ProvideRuntimeApi<Block: BlockT> {
    /// The concrete type that provides the api.
    type Api: ApiExt<Block>;
    /// Returns the runtime api.
    /// The returned instance will keep track of modifications to the storage. Any successful
    /// call to an api function, will `commit` its changes to an internal buffer. Otherwise,
    /// the modifications will be `discarded`. The modifications will not be applied to the
    /// storage, even on a `commit`.
    fn runtime_api(&self) -> ApiRef<Self::Api>;
}
/// Something that provides information about a runtime api.
#[cfg(feature = "std")]
pub trait RuntimeApiInfo {
    /// The identifier of the runtime api.
    const ID: [u8; 8];
    /// The version of the runtime api.
    const VERSION: u32;
}
/// The number of bytes required to encode a [`RuntimeApiInfo`].
///
/// 8 bytes for `ID` and 4 bytes for a version.
pub const RUNTIME_API_INFO_SIZE: usize = 12;
/// Crude and simple way to serialize the `RuntimeApiInfo` into a bunch of bytes.
pub const fn serialize_runtime_api_info(id: [u8; 8], version: u32) -> [u8; RUNTIME_API_INFO_SIZE] {
    let version = version.to_le_bytes();
    let mut r = [0; RUNTIME_API_INFO_SIZE];
    r[0] = id[0];
    r[1] = id[1];
    r[2] = id[2];
    r[3] = id[3];
    r[4] = id[4];
    r[5] = id[5];
    r[6] = id[6];
    r[7] = id[7];
    r[8] = version[0];
    r[9] = version[1];
    r[10] = version[2];
    r[11] = version[3];
    r
}
/// Deserialize the runtime API info serialized by [`serialize_runtime_api_info`].
pub fn deserialize_runtime_api_info(bytes: [u8; RUNTIME_API_INFO_SIZE]) -> ([u8; 8], u32) {
    let id: [u8; 8] = bytes[0..8]
        .try_into()
        .expect("the source slice size is equal to the dest array length; qed");
    let version = u32::from_le_bytes(
        bytes[8..12]
            .try_into()
            .expect("the source slice size is equal to the array length; qed"),
    );
    (id, version)
}
#[doc(hidden)]
#[allow(dead_code)]
#[allow(deprecated)]
pub mod runtime_decl_for_core {
    pub use super::*;
    /// The `Core` runtime api that every Substrate runtime needs to implement.
    pub trait CoreV4<Block: sp_api::BlockT> {
        /// Returns the version of the runtime.
        fn version() -> RuntimeVersion;
        /// Execute the given block.
        fn execute_block(block: Block);
        /// Initialize a block with the given header.
        fn initialize_block(header: &<Block as BlockT>::Header);
    }
    pub use CoreV4 as Core;
    #[inline(always)]
    pub fn runtime_metadata<Block: sp_api::BlockT>() -> sp_api::metadata_ir::RuntimeApiMetadataIR
    where
        RuntimeVersion: sp_api::scale_info::TypeInfo + 'static,
        Block: sp_api::scale_info::TypeInfo + 'static,
        <Block as BlockT>::Header: sp_api::scale_info::TypeInfo + 'static,
    {
        sp_api::metadata_ir::RuntimeApiMetadataIR {
            name: "Core",
            methods: <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    sp_api::metadata_ir::RuntimeApiMethodMetadataIR {
                        name: "version",
                        inputs: ::alloc::vec::Vec::new(),
                        output: sp_api::scale_info::meta_type::<RuntimeVersion>(),
                        docs: <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([" Returns the version of the runtime."]),
                        ),
                    },
                    sp_api::metadata_ir::RuntimeApiMethodMetadataIR {
                        name: "execute_block",
                        inputs: <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([
                                sp_api::metadata_ir::RuntimeApiMethodParamMetadataIR {
                                    name: "block",
                                    ty: sp_api::scale_info::meta_type::<Block>(),
                                },
                            ]),
                        ),
                        output: sp_api::scale_info::meta_type::<()>(),
                        docs: <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([" Execute the given block."]),
                        ),
                    },
                    sp_api::metadata_ir::RuntimeApiMethodMetadataIR {
                        name: "initialize_block",
                        inputs: <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([
                                sp_api::metadata_ir::RuntimeApiMethodParamMetadataIR {
                                    name: "header",
                                    ty: sp_api::scale_info::meta_type::<&<Block as BlockT>::Header>(
                                    ),
                                },
                            ]),
                        ),
                        output: sp_api::scale_info::meta_type::<()>(),
                        docs: <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([
                                " Initialize a block with the given header.",
                            ]),
                        ),
                    },
                ]),
            ),
            docs: <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    " The `Core` runtime api that every Substrate runtime needs to implement.",
                ]),
            ),
        }
    }
    pub const VERSION: u32 = 4u32;
    pub const ID: [u8; 8] = [223u8, 106u8, 203u8, 104u8, 153u8, 7u8, 96u8, 155u8];
}
#[doc(hidden)]
#[allow(dead_code)]
#[allow(deprecated)]
pub mod runtime_decl_for_metadata {
    pub use super::*;
    /// The `Metadata` api trait that returns metadata for the runtime.
    pub trait MetadataV2<Block: sp_api::BlockT> {
        /// Returns the metadata of a runtime.
        fn metadata() -> OpaqueMetadata;
        /// Returns the metadata at a given version.
        ///
        /// If the given `version` isn't supported, this will return `None`.
        /// Use [`Self::metadata_versions`] to find out about supported metadata version of the runtime.
        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata>;
        /// Returns the supported metadata versions.
        ///
        /// This can be used to call `metadata_at_version`.
        fn metadata_versions() -> sp_std::vec::Vec<u32>;
    }
    pub use MetadataV2 as Metadata;
    #[inline(always)]
    pub fn runtime_metadata<Block: sp_api::BlockT>() -> sp_api::metadata_ir::RuntimeApiMetadataIR
    where
        OpaqueMetadata: sp_api::scale_info::TypeInfo + 'static,
        u32: sp_api::scale_info::TypeInfo + 'static,
        Option<OpaqueMetadata>: sp_api::scale_info::TypeInfo + 'static,
        sp_std::vec::Vec<u32>: sp_api::scale_info::TypeInfo + 'static,
    {
        sp_api :: metadata_ir :: RuntimeApiMetadataIR { name : "Metadata" , methods : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "metadata" , inputs : :: alloc :: vec :: Vec :: new () , output : sp_api :: scale_info :: meta_type :: < OpaqueMetadata > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Returns the metadata of a runtime."])) , } , sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "metadata_at_version" , inputs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([sp_api :: metadata_ir :: RuntimeApiMethodParamMetadataIR { name : "version" , ty : sp_api :: scale_info :: meta_type :: < u32 > () , }])) , output : sp_api :: scale_info :: meta_type :: < Option < OpaqueMetadata > > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Returns the metadata at a given version." , "" , " If the given `version` isn\'t supported, this will return `None`." , " Use [`Self::metadata_versions`] to find out about supported metadata version of the runtime."])) , } , sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "metadata_versions" , inputs : :: alloc :: vec :: Vec :: new () , output : sp_api :: scale_info :: meta_type :: < sp_std :: vec :: Vec < u32 > > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Returns the supported metadata versions." , "" , " This can be used to call `metadata_at_version`."])) , }])) , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" The `Metadata` api trait that returns metadata for the runtime."])) , }
    }
    pub const VERSION: u32 = 2u32;
    pub const ID: [u8; 8] = [55u8, 227u8, 151u8, 252u8, 124u8, 145u8, 245u8, 228u8];
}
/// The `Core` runtime api that every Substrate runtime needs to implement.
#[cfg(any(feature = "std", test))]
pub trait Core<Block: sp_api::BlockT>: 'static + Send {
    /// Returns the version of the runtime.
    fn version(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
    ) -> std::result::Result<RuntimeVersion, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as Core<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "Core_version"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <RuntimeVersion as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Core_version",
                    error: err,
                },
            )
        })
    }
    /// Returns the version of the runtime.
    fn version_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
    ) -> std::result::Result<RuntimeVersion, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as Core<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "Core_version"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <RuntimeVersion as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Core_version",
                    error: err,
                },
            )
        })
    }
    /// Execute the given block.
    fn execute_block(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        block: Block,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&block));
        <Self as Core<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "Core_execute_block"),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Core_execute_block",
                    error: err,
                }
            })
        })
    }
    /// Execute the given block.
    fn execute_block_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        block: Block,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&block));
        <Self as Core<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "Core_execute_block"),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Core_execute_block",
                    error: err,
                }
            })
        })
    }
    /// Initialize a block with the given header.
    fn initialize_block(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        header: &<Block as BlockT>::Header,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&header));
        <Self as Core<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| {
                if _version
                    .apis
                    .iter()
                    .any(|(s, v)| s == &runtime_decl_for_core::ID && *v < 2u32)
                {
                    return "Core_initialise_block";
                }
                "Core_initialize_block"
            }),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Core_initialize_block",
                    error: err,
                }
            })
        })
    }
    /// Initialize a block with the given header.
    fn initialize_block_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        header: &<Block as BlockT>::Header,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&header));
        <Self as Core<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| {
                if _version
                    .apis
                    .iter()
                    .any(|(s, v)| s == &runtime_decl_for_core::ID && *v < 2u32)
                {
                    return "Core_initialise_block";
                }
                "Core_initialize_block"
            }),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Core_initialize_block",
                    error: err,
                }
            })
        })
    }
    /// !!INTERNAL USE ONLY!!
    /// decl宏默认实现所有的trait method, 只需要在impl_runtime_api宏里实现该方法就行
    /// 用于转换 impl Api for Runtime 为 impl Api for RuntimeApi 的辅助数据结构。
    /// 这要求我们将runtime Block 替换为 node Block，
    /// 将“impl Api for runtime”替换为“impl Api for RuntimeApi”，并将方法实现替换为调用运行时的代码。
    #[doc(hidden)]
    fn __runtime_api_internal_call_api_at(
        &self,
        at: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        params: std::vec::Vec<u8>,
        fn_name: &dyn Fn(sp_api::RuntimeVersion) -> &'static str,
    ) -> std::result::Result<std::vec::Vec<u8>, sp_api::ApiError>;
}
#[cfg(any(feature = "std", test))]
impl<Block: sp_api::BlockT> sp_api::RuntimeApiInfo for dyn Core<Block> {
    const ID: [u8; 8] = [223u8, 106u8, 203u8, 104u8, 153u8, 7u8, 96u8, 155u8];
    const VERSION: u32 = 4u32;
}
/// The `Metadata` api trait that returns metadata for the runtime.
#[cfg(any(feature = "std", test))]
pub trait Metadata<Block: sp_api::BlockT>: sp_api::Core<Block> {
    /// Returns the metadata of a runtime.
    fn metadata(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
    ) -> std::result::Result<OpaqueMetadata, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as Metadata<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "Metadata_metadata"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <OpaqueMetadata as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Metadata_metadata",
                    error: err,
                },
            )
        })
    }
    /// Returns the metadata of a runtime.
    fn metadata_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
    ) -> std::result::Result<OpaqueMetadata, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as Metadata<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "Metadata_metadata"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <OpaqueMetadata as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Metadata_metadata",
                    error: err,
                },
            )
        })
    }
    /// Returns the metadata at a given version.
    ///
    /// If the given `version` isn't supported, this will return `None`.
    /// Use [`Self::metadata_versions`] to find out about supported metadata version of the runtime.
    fn metadata_at_version(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        version: u32,
    ) -> std::result::Result<Option<OpaqueMetadata>, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&version));
        <Self as Metadata<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "Metadata_metadata_at_version"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <Option<OpaqueMetadata> as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Metadata_metadata_at_version",
                    error: err,
                },
            )
        })
    }
    /// Returns the metadata at a given version.
    ///
    /// If the given `version` isn't supported, this will return `None`.
    /// Use [`Self::metadata_versions`] to find out about supported metadata version of the runtime.
    fn metadata_at_version_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        version: u32,
    ) -> std::result::Result<Option<OpaqueMetadata>, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&version));
        <Self as Metadata<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "Metadata_metadata_at_version"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <Option<OpaqueMetadata> as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Metadata_metadata_at_version",
                    error: err,
                },
            )
        })
    }
    /// Returns the supported metadata versions.
    ///
    /// This can be used to call `metadata_at_version`.
    fn metadata_versions(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
    ) -> std::result::Result<sp_std::vec::Vec<u32>, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as Metadata<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "Metadata_metadata_versions"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <sp_std::vec::Vec<u32> as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Metadata_metadata_versions",
                    error: err,
                },
            )
        })
    }
    /// Returns the supported metadata versions.
    ///
    /// This can be used to call `metadata_at_version`.
    fn metadata_versions_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
    ) -> std::result::Result<sp_std::vec::Vec<u32>, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as Metadata<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "Metadata_metadata_versions"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <sp_std::vec::Vec<u32> as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "Metadata_metadata_versions",
                    error: err,
                },
            )
        })
    }
    /// !!INTERNAL USE ONLY!!
    #[doc(hidden)]
    fn __runtime_api_internal_call_api_at(
        &self,
        at: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        params: std::vec::Vec<u8>,
        fn_name: &dyn Fn(sp_api::RuntimeVersion) -> &'static str,
    ) -> std::result::Result<std::vec::Vec<u8>, sp_api::ApiError>;
}
#[cfg(any(feature = "std", test))]
impl<Block: sp_api::BlockT> sp_api::RuntimeApiInfo for dyn Metadata<Block> {
    const ID: [u8; 8] = [55u8, 227u8, 151u8, 252u8, 124u8, 145u8, 245u8, 228u8];
    const VERSION: u32 = 2u32;
}
