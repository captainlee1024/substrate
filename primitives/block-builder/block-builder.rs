#![feature(prelude_import)]
//! The block builder runtime api.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::{traits::Block as BlockT, ApplyExtrinsicResult};
#[doc(hidden)]
#[allow(dead_code)]
#[allow(deprecated)]
pub mod runtime_decl_for_block_builder {
    pub use super::*;
    /// The `BlockBuilder` api trait that provides the required functionality for building a block.
    pub trait BlockBuilderV6<Block: sp_api::BlockT> {
        /// Apply the given extrinsic.
        ///
        /// Returns an inclusion outcome which specifies if this extrinsic is included in
        /// this block or not.
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult;
        /// Finish the current block.
        fn finalize_block() -> <Block as BlockT>::Header;
        /// Generate inherent extrinsics. The inherent data will vary from chain to chain.
        fn inherent_extrinsics(
            inherent: InherentData,
        ) -> sp_std::vec::Vec<<Block as BlockT>::Extrinsic>;
        /// Check that the inherents are valid. The inherent data will vary from chain to chain.
        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult;
    }
    pub use BlockBuilderV6 as BlockBuilder;
    #[inline(always)]
    pub fn runtime_metadata<Block: sp_api::BlockT>() -> sp_api::metadata_ir::RuntimeApiMetadataIR
    where
        <Block as BlockT>::Extrinsic: sp_api::scale_info::TypeInfo + 'static,
        ApplyExtrinsicResult: sp_api::scale_info::TypeInfo + 'static,
        <Block as BlockT>::Header: sp_api::scale_info::TypeInfo + 'static,
        InherentData: sp_api::scale_info::TypeInfo + 'static,
        sp_std::vec::Vec<<Block as BlockT>::Extrinsic>: sp_api::scale_info::TypeInfo + 'static,
        Block: sp_api::scale_info::TypeInfo + 'static,
        InherentData: sp_api::scale_info::TypeInfo + 'static,
        CheckInherentsResult: sp_api::scale_info::TypeInfo + 'static,
    {
        sp_api :: metadata_ir :: RuntimeApiMetadataIR { name : "BlockBuilder" , methods : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "apply_extrinsic" , inputs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([sp_api :: metadata_ir :: RuntimeApiMethodParamMetadataIR { name : "extrinsic" , ty : sp_api :: scale_info :: meta_type :: < < Block as BlockT > :: Extrinsic > () , }])) , output : sp_api :: scale_info :: meta_type :: < ApplyExtrinsicResult > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Apply the given extrinsic." , "" , " Returns an inclusion outcome which specifies if this extrinsic is included in" , " this block or not."])) , } , sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "finalize_block" , inputs : :: alloc :: vec :: Vec :: new () , output : sp_api :: scale_info :: meta_type :: < < Block as BlockT > :: Header > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Finish the current block."])) , } , sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "inherent_extrinsics" , inputs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([sp_api :: metadata_ir :: RuntimeApiMethodParamMetadataIR { name : "inherent" , ty : sp_api :: scale_info :: meta_type :: < InherentData > () , }])) , output : sp_api :: scale_info :: meta_type :: < sp_std :: vec :: Vec < < Block as BlockT > :: Extrinsic > > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Generate inherent extrinsics. The inherent data will vary from chain to chain."])) , } , sp_api :: metadata_ir :: RuntimeApiMethodMetadataIR { name : "check_inherents" , inputs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([sp_api :: metadata_ir :: RuntimeApiMethodParamMetadataIR { name : "block" , ty : sp_api :: scale_info :: meta_type :: < Block > () , } , sp_api :: metadata_ir :: RuntimeApiMethodParamMetadataIR { name : "data" , ty : sp_api :: scale_info :: meta_type :: < InherentData > () , }])) , output : sp_api :: scale_info :: meta_type :: < CheckInherentsResult > () , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" Check that the inherents are valid. The inherent data will vary from chain to chain."])) , }])) , docs : < [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([" The `BlockBuilder` api trait that provides the required functionality for building a block."])) , }
    }
    pub const VERSION: u32 = 6u32;
    pub const ID: [u8; 8] = [64u8, 254u8, 58u8, 212u8, 1u8, 248u8, 149u8, 154u8];
}
/// The `BlockBuilder` api trait that provides the required functionality for building a block.
#[cfg(any(feature = "std", test))]
pub trait BlockBuilder<Block: sp_api::BlockT>: sp_api::Core<Block> {
    /// Apply the given extrinsic.
    ///
    /// Returns an inclusion outcome which specifies if this extrinsic is included in
    /// this block or not.
    fn apply_extrinsic(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        extrinsic: <Block as BlockT>::Extrinsic,
    ) -> std::result::Result<ApplyExtrinsicResult, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&extrinsic));
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "BlockBuilder_apply_extrinsic"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <ApplyExtrinsicResult as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_apply_extrinsic",
                    error: err,
                },
            )
        })
    }
    /// Apply the given extrinsic.
    ///
    /// Returns an inclusion outcome which specifies if this extrinsic is included in
    /// this block or not.
    /// 在这里将交易池取出的extrinsic进行encode
    /// 在
    fn apply_extrinsic_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        extrinsic: <Block as BlockT>::Extrinsic,
    ) -> std::result::Result<ApplyExtrinsicResult, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&extrinsic));
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "BlockBuilder_apply_extrinsic"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <ApplyExtrinsicResult as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_apply_extrinsic",
                    error: err,
                },
            )
        })
    }
    #[deprecated]
    fn apply_extrinsic_before_version_6(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        extrinsic: <Block as BlockT>::Extrinsic,
    ) -> std::result::Result<
        sp_runtime::legacy::byte_sized_error::ApplyExtrinsicResult,
        sp_api::ApiError,
    > {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&extrinsic));
        < Self as BlockBuilder < _ > > :: __runtime_api_internal_call_api_at (self , __runtime_api_at_param__ , sp_api :: ExecutionContext :: OffchainCall (None) , __runtime_api_impl_params_encoded__ , & (| _version | { "BlockBuilder_apply_extrinsic" })) . and_then (| r | std :: result :: Result :: map_err (< sp_runtime :: legacy :: byte_sized_error :: ApplyExtrinsicResult as sp_api :: Decode > :: decode (& mut & r [..]) , | err | sp_api :: ApiError :: FailedToDecodeReturnValue { function : "BlockBuilder_apply_extrinsic" , error : err , }))
    }
    #[deprecated]
    fn apply_extrinsic_before_version_6_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        extrinsic: <Block as BlockT>::Extrinsic,
    ) -> std::result::Result<
        sp_runtime::legacy::byte_sized_error::ApplyExtrinsicResult,
        sp_api::ApiError,
    > {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&extrinsic));
        < Self as BlockBuilder < _ > > :: __runtime_api_internal_call_api_at (self , __runtime_api_at_param__ , context , __runtime_api_impl_params_encoded__ , & (| _version | { "BlockBuilder_apply_extrinsic" })) . and_then (| r | std :: result :: Result :: map_err (< sp_runtime :: legacy :: byte_sized_error :: ApplyExtrinsicResult as sp_api :: Decode > :: decode (& mut & r [..]) , | err | sp_api :: ApiError :: FailedToDecodeReturnValue { function : "BlockBuilder_apply_extrinsic" , error : err , }))
    }
    /// Finish the current block.
    fn finalize_block(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
    ) -> std::result::Result<<Block as BlockT>::Header, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| {
                if _version
                    .apis
                    .iter()
                    .any(|(s, v)| s == &runtime_decl_for_block_builder::ID && *v < 3u32)
                {
                    return "BlockBuilder_finalise_block";
                }
                "BlockBuilder_finalize_block"
            }),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <<Block as BlockT>::Header as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_finalize_block",
                    error: err,
                },
            )
        })
    }
    /// Finish the current block.
    fn finalize_block_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
    ) -> std::result::Result<<Block as BlockT>::Header, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&());
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| {
                if _version
                    .apis
                    .iter()
                    .any(|(s, v)| s == &runtime_decl_for_block_builder::ID && *v < 3u32)
                {
                    return "BlockBuilder_finalise_block";
                }
                "BlockBuilder_finalize_block"
            }),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <<Block as BlockT>::Header as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_finalize_block",
                    error: err,
                },
            )
        })
    }
    /// Generate inherent extrinsics. The inherent data will vary from chain to chain.
    fn inherent_extrinsics(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        inherent: InherentData,
    ) -> std::result::Result<sp_std::vec::Vec<<Block as BlockT>::Extrinsic>, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&inherent));
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "BlockBuilder_inherent_extrinsics"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <sp_std::vec::Vec<<Block as BlockT>::Extrinsic> as sp_api::Decode>::decode(
                    &mut &r[..],
                ),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_inherent_extrinsics",
                    error: err,
                },
            )
        })
    }
    /// Generate inherent extrinsics. The inherent data will vary from chain to chain.
    fn inherent_extrinsics_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        inherent: InherentData,
    ) -> std::result::Result<sp_std::vec::Vec<<Block as BlockT>::Extrinsic>, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&inherent));
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "BlockBuilder_inherent_extrinsics"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <sp_std::vec::Vec<<Block as BlockT>::Extrinsic> as sp_api::Decode>::decode(
                    &mut &r[..],
                ),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_inherent_extrinsics",
                    error: err,
                },
            )
        })
    }
    /// Check that the inherents are valid. The inherent data will vary from chain to chain.
    fn check_inherents(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        block: Block,
        data: InherentData,
    ) -> std::result::Result<CheckInherentsResult, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&block, &data));
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            sp_api::ExecutionContext::OffchainCall(None),
            __runtime_api_impl_params_encoded__,
            &(|_version| "BlockBuilder_check_inherents"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <CheckInherentsResult as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_check_inherents",
                    error: err,
                },
            )
        })
    }
    /// Check that the inherents are valid. The inherent data will vary from chain to chain.
    fn check_inherents_with_context(
        &self,
        __runtime_api_at_param__: <Block as sp_api::BlockT>::Hash,
        context: sp_api::ExecutionContext,
        block: Block,
        data: InherentData,
    ) -> std::result::Result<CheckInherentsResult, sp_api::ApiError> {
        let __runtime_api_impl_params_encoded__ = sp_api::Encode::encode(&(&block, &data));
        <Self as BlockBuilder<_>>::__runtime_api_internal_call_api_at(
            self,
            __runtime_api_at_param__,
            context,
            __runtime_api_impl_params_encoded__,
            &(|_version| "BlockBuilder_check_inherents"),
        )
        .and_then(|r| {
            std::result::Result::map_err(
                <CheckInherentsResult as sp_api::Decode>::decode(&mut &r[..]),
                |err| sp_api::ApiError::FailedToDecodeReturnValue {
                    function: "BlockBuilder_check_inherents",
                    error: err,
                },
            )
        })
    }
    /// !!INTERNAL USE ONLY!!
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
impl<Block: sp_api::BlockT> sp_api::RuntimeApiInfo for dyn BlockBuilder<Block> {
    const ID: [u8; 8] = [64u8, 254u8, 58u8, 212u8, 1u8, 248u8, 149u8, 154u8];
    const VERSION: u32 = 6u32;
}
