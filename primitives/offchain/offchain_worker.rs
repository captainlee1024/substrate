#![feature(prelude_import)]
//! The Offchain Worker runtime api primitives.
#![warn(missing_docs)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
/// Re-export of parent module scope storage prefix.
pub use sp_core::offchain::STORAGE_PREFIX;
#[doc(hidden)]
#[allow(dead_code)]
#[allow(deprecated)]
pub mod runtime_decl_for_offchain_worker_api {
    pub use super::*;
    /// The offchain worker api.
    pub trait OffchainWorkerApiV2<Block: sp_api::BlockT> {
        /// Starts the off-chain task for given block header.
        fn offchain_worker(header: &Block::Header);
    }
    pub use OffchainWorkerApiV2 as OffchainWorkerApi;
    #[inline(always)]
    pub fn runtime_metadata<Block: sp_api::BlockT>() -> sp_api::metadata_ir::RuntimeApiMetadataIR
    where
        Block::Header: sp_api::scale_info::TypeInfo + 'static,
    {
        sp_api::metadata_ir::RuntimeApiMetadataIR {
            name: "OffchainWorkerApi",
            methods: <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([sp_api::metadata_ir::RuntimeApiMethodMetadataIR {
                    name: "offchain_worker",
                    inputs: <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            sp_api::metadata_ir::RuntimeApiMethodParamMetadataIR {
                                name: "header",
                                ty: sp_api::scale_info::meta_type::<&Block::Header>(),
                            },
                        ]),
                    ),
                    output: sp_api::scale_info::meta_type::<()>(),
                    docs: <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            " Starts the off-chain task for given block header.",
                        ]),
                    ),
                }]),
            ),
            docs: <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([" The offchain worker api."]),
            ),
        }
    }
    pub const VERSION: u32 = 2u32;
    pub const ID: [u8; 8] = [247u8, 139u8, 39u8, 139u8, 229u8, 63u8, 69u8, 76u8];
}
/// The offchain worker api.
#[cfg(any(feature = "std", test))]
pub trait OffchainWorkerApi<Block: sp_api::BlockT>: sp_api::Core<Block> {
    /// Starts the off-chain task for given block number.
    #[deprecated]
    fn offchain_worker_before_version_2(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        number: sp_runtime::traits::NumberFor<Block>,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&number));
        <Self as OffchainWorkerApi<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "OffchainWorkerApi_offchain_worker"),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "OffchainWorkerApi_offchain_worker",
                    error: err,
                }
            })
        })
    }
    /// Starts the off-chain task for given block number.
    #[deprecated]
    fn offchain_worker_before_version_2_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        number: sp_runtime::traits::NumberFor<Block>,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&number));
        <Self as OffchainWorkerApi<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "OffchainWorkerApi_offchain_worker"),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "OffchainWorkerApi_offchain_worker",
                    error: err,
                }
            })
        })
    }
    /// Starts the off-chain task for given block header.
    fn offchain_worker(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        header: &Block::Header,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&header));
        <Self as OffchainWorkerApi<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "OffchainWorkerApi_offchain_worker"),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "OffchainWorkerApi_offchain_worker",
                    error: err,
                }
            })
        })
    }
    /// Starts the off-chain task for given block header.
    fn offchain_worker_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        header: &Block::Header,
    ) -> std::result::Result<(), sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&header));
        <Self as OffchainWorkerApi<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "OffchainWorkerApi_offchain_worker"),
        )
        .and_then(|r| {
            std::result::Result::map_err(<() as sp_api::Decode>::decode(&mut &r[..]), |err| {
                sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "OffchainWorkerApi_offchain_worker",
                    error: err,
                }
            })
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
impl<Block: sp_api::BlockT> sp_api::RuntimeApiInfo for dyn OffchainWorkerApi<Block> {
    const ID: [u8; 8] = [247u8, 139u8, 39u8, 139u8, 229u8, 63u8, 69u8, 76u8];
    const VERSION: u32 = 2u32;
}
