#![feature(prelude_import)]
//! Substrate runtime interface
//!
//! This crate provides types, traits and macros around runtime interfaces. A runtime interface is
//! a fixed interface between a Substrate runtime and a Substrate node. For a native runtime the
//! interface maps to a direct function call of the implementation. For a wasm runtime the interface
//! maps to an external function call. These external functions are exported by the wasm executor
//! and they map to the same implementation as the native calls.
//!
//! # Using a type in a runtime interface
//!
//! Any type that should be used in a runtime interface as argument or return value needs to
//! implement [`RIType`]. The associated type
//! [`FFIType`](./trait.RIType.html#associatedtype.FFIType) is the type that is used in the FFI
//! function to represent the actual type. For example `[T]` is represented by an `u64`. The slice
//! pointer and the length will be mapped to an `u64` value. For more information see this
//! [table](#ffi-type-and-conversion). The FFI function definition is used when calling from the
//! wasm runtime into the node.
//!
//! Traits are used to convert from a type to the corresponding
//! [`RIType::FFIType`](./trait.RIType.html#associatedtype.FFIType).
//! Depending on where and how a type should be used in a function signature, a combination of the
//! following traits need to be implemented:
//! <!-- markdown-link-check-enable -->
//! 1. Pass as function argument: [`wasm::IntoFFIValue`] and [`host::FromFFIValue`]
//! 2. As function return value: [`wasm::FromFFIValue`] and [`host::IntoFFIValue`]
//! 3. Pass as mutable function argument: [`host::IntoPreallocatedFFIValue`]
//!
//! The traits are implemented for most of the common types like `[T]`, `Vec<T>`, arrays and
//! primitive types.
//!
//! For custom types, we provide the [`PassBy`](./pass_by#PassBy) trait and strategies that define
//! how a type is passed between the wasm runtime and the node. Each strategy also provides a derive
//! macro to simplify the implementation.
//!
//! # Performance
//!
//! To not waste any more performance when calling into the node, not all types are SCALE encoded
//! when being passed as arguments between the wasm runtime and the node. For most types that
//! are raw bytes like `Vec<u8>`, `[u8]` or `[u8; N]` we pass them directly, without SCALE encoding
//! them in front of. The implementation of [`RIType`] each type provides more information on how
//! the data is passed.
//!
//! # Declaring a runtime interface
//!
//! Declaring a runtime interface is similar to declaring a trait in Rust:
//!
//! ```
//! #[sp_runtime_interface::runtime_interface]
//! trait RuntimeInterface {
//!     fn some_function(value: &[u8]) -> bool {
//!         value.iter().all(|v| *v > 125)
//!     }
//! }
//! ```
//!
//! For more information on declaring a runtime interface, see
//! [`#[runtime_interface]`](./attr.runtime_interface.html).
//!
//! # FFI type and conversion
//!
//! The following table documents how values of types are passed between the wasm and
//! the host side and how they are converted into the corresponding type.
//!
//! | Type | FFI type | Conversion |
//! |----|----|----|
//! | `u8` | `u32` | zero-extended to 32-bits |
//! | `u16` | `u32` | zero-extended to 32-bits |
//! | `u32` | `u32` | `Identity` |
//! | `u64` | `u64` | `Identity` |
//! | `i128` | `u32` | `v.as_ptr()` (pointer to a 16 byte array) |
//! | `i8` | `i32` | sign-extended to 32-bits |
//! | `i16` | `i32` | sign-extended to 32-bits |
//! | `i32` | `i32` | `Identity` |
//! | `i64` | `i64` | `Identity` |
//! | `u128` | `u32` | `v.as_ptr()` (pointer to a 16 byte array) |
//! | `bool` | `u32` | `if v { 1 } else { 0 }` |
//! | `&str` | `u64` | <code>v.len() 32bit << 32 &#124; v.as_ptr() 32bit</code> |
//! | `&[u8]` | `u64` | <code>v.len() 32bit << 32 &#124; v.as_ptr() 32bit</code> |
//! | `Vec<u8>` | `u64` | <code>v.len() 32bit << 32 &#124; v.as_ptr() 32bit</code> |
//! | `Vec<T> where T: Encode` | `u64` | `let e = v.encode();`<br><br><code>e.len() 32bit << 32 &#124; e.as_ptr() 32bit</code> |
//! | `&[T] where T: Encode` | `u64` | `let e = v.encode();`<br><br><code>e.len() 32bit << 32 &#124; e.as_ptr() 32bit</code> |
//! | `[u8; N]` | `u32` | `v.as_ptr()` |
//! | `*const T` | `u32` | `Identity` |
//! | `Option<T>` | `u64` | `let e = v.encode();`<br><br><code>e.len() 32bit << 32 &#124; e.as_ptr() 32bit</code> |
//! | [`T where T: PassBy<PassBy=Inner>`](./pass_by#Inner) | Depends on inner | Depends on inner |
//! | [`T where T: PassBy<PassBy=Codec>`](./pass_by#Codec)|`u64`|<code>v.len() 32bit << 32 &#124;v.as_ptr() 32bit</code>|
//!
//! `Identity` means that the value is converted directly into the corresponding FFI type.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate self as sp_runtime_interface;
#[doc(hidden)]
#[cfg(feature = "std")]
pub use sp_wasm_interface;
#[doc(hidden)]
pub use sp_tracing;
#[doc(hidden)]
pub use sp_std;
/// Attribute macro for transforming a trait declaration into a runtime interface.
///
/// A runtime interface is a fixed interface between a Substrate compatible runtime and the
/// native node. This interface is callable from a native and a wasm runtime. The macro will
/// generate the corresponding code for the native implementation and the code for calling from
/// the wasm side to the native implementation.
///
/// The macro expects the runtime interface declaration as trait declaration:
///
/// ```
/// # use sp_runtime_interface::runtime_interface;
///
/// #[runtime_interface]
/// trait Interface {
///     /// A function that can be called from native/wasm.
///     ///
///     /// The implementation given to this function is only compiled on native.
///     fn call(data: &[u8]) -> Vec<u8> {
///         // Here you could call some rather complex code that only compiles on native or
///         // is way faster in native than executing it in wasm.
///         Vec::new()
///     }
///     /// Call function, but different version.
///     ///
///     /// For new runtimes, only function with latest version is reachable.
///     /// But old version (above) is still accessible for old runtimes.
///     /// Default version is 1.
///     #[version(2)]
///     fn call(data: &[u8]) -> Vec<u8> {
///         // Here you could call some rather complex code that only compiles on native or
///         // is way faster in native than executing it in wasm.
///         [17].to_vec()
///     }
///
///     /// Call function, different version and only being registered.
///     ///
///     /// This `register_only` version is only being registered, aka exposed to the runtime,
///     /// but the runtime will still use the version 2 of this function. This is useful for when
///     /// new host functions should be introduced. Adding new host functions requires that all
///     /// nodes have the host functions available, because otherwise they fail at instantiation
///     /// of the runtime. With `register_only` the function will not be used when compiling the
///     /// runtime, but it will already be there for a future version of the runtime that will
///     /// switch to using these host function.
///     #[version(3, register_only)]
///     fn call(data: &[u8]) -> Vec<u8> {
///         // Here you could call some rather complex code that only compiles on native or
///         // is way faster in native than executing it in wasm.
///         [18].to_vec()
///     }
///
///     /// A function can take a `&self` or `&mut self` argument to get access to the
///     /// `Externalities`. (The generated method does not require
///     /// this argument, so the function can be called just with the `optional` argument)
///     fn set_or_clear(&mut self, optional: Option<Vec<u8>>) {
///         match optional {
///             Some(value) => self.set_storage([1, 2, 3, 4].to_vec(), value),
///             None => self.clear_storage(&[1, 2, 3, 4]),
///         }
///     }
/// }
/// ```
///
///
/// The given example will generate roughly the following code for native:
///
/// ```
/// // The name of the trait is converted to snake case and used as mod name.
/// //
/// // Be aware that this module is not `public`, the visibility of the module is determined based
/// // on the visibility of the trait declaration.
/// mod interface {
///     trait Interface {
///         fn call_version_1(data: &[u8]) -> Vec<u8>;
///         fn call_version_2(data: &[u8]) -> Vec<u8>;
///         fn call_version_3(data: &[u8]) -> Vec<u8>;
///         fn set_or_clear_version_1(&mut self, optional: Option<Vec<u8>>);
///     }
///
///     impl Interface for &mut dyn sp_externalities::Externalities {
///         fn call_version_1(data: &[u8]) -> Vec<u8> { Vec::new() }
///         fn call_version_2(data: &[u8]) -> Vec<u8> { [17].to_vec() }
///         fn call_version_3(data: &[u8]) -> Vec<u8> { [18].to_vec() }
///         fn set_or_clear_version_1(&mut self, optional: Option<Vec<u8>>) {
///             match optional {
///                 Some(value) => self.set_storage([1, 2, 3, 4].to_vec(), value),
///                 None => self.clear_storage(&[1, 2, 3, 4]),
///             }
///         }
///     }
///
///     pub fn call(data: &[u8]) -> Vec<u8> {
///         // only latest version is exposed
///         call_version_2(data)
///     }
///
///     fn call_version_1(data: &[u8]) -> Vec<u8> {
///         <&mut dyn sp_externalities::Externalities as Interface>::call_version_1(data)
///     }
///
///     fn call_version_2(data: &[u8]) -> Vec<u8> {
///         <&mut dyn sp_externalities::Externalities as Interface>::call_version_2(data)
///     }
///
///     fn call_version_3(data: &[u8]) -> Vec<u8> {
///         <&mut dyn sp_externalities::Externalities as Interface>::call_version_3(data)
///     }
///
///     pub fn set_or_clear(optional: Option<Vec<u8>>) {
///         set_or_clear_version_1(optional)
///     }
///
///     fn set_or_clear_version_1(optional: Option<Vec<u8>>) {
///         sp_externalities::with_externalities(|mut ext| Interface::set_or_clear_version_1(&mut ext, optional))
///             .expect("`set_or_clear` called outside of an Externalities-provided environment.")
///     }
///
///     /// This type implements the `HostFunctions` trait (from `sp-wasm-interface`) and
///     /// provides the host implementation for the wasm side. The host implementation converts the
///     /// arguments from wasm to native and calls the corresponding native function.
///     ///
///     /// This type needs to be passed to the wasm executor, so that the host functions will be
///     /// registered in the executor.
///     pub struct HostFunctions;
/// }
/// ```
///
///
/// The given example will generate roughly the following code for wasm:
///
/// ```
/// mod interface {
///     mod extern_host_functions_impls {
///         extern "C" {
///             /// Every function is exported as `ext_TRAIT_NAME_FUNCTION_NAME_version_VERSION`.
///             ///
///             /// `TRAIT_NAME` is converted into snake case.
///             ///
///             /// The type for each argument of the exported function depends on
///             /// `<ARGUMENT_TYPE as RIType>::FFIType`.
///             ///
///             /// `data` holds the pointer and the length to the `[u8]` slice.
///             pub fn ext_Interface_call_version_1(data: u64) -> u64;
///             /// `optional` holds the pointer and the length of the encoded value.
///             pub fn ext_Interface_set_or_clear_version_1(optional: u64);
///         }
///     }
///
///     /// The type is actually `ExchangeableFunction` (from `sp-runtime-interface`).
///     ///
///     /// This can be used to replace the implementation of the `call` function.
///     /// Instead of calling into the host, the callee will automatically call the other
///     /// implementation.
///     ///
///     /// To replace the implementation:
///     ///
///     /// `host_call.replace_implementation(some_other_impl)`
///     pub static host_call: () = ();
///     pub static host_set_or_clear: () = ();
///
///     pub fn call(data: &[u8]) -> Vec<u8> {
///         // This is the actual call: `host_call.get()(data)`
///         //
///         // But that does not work for several reasons in this example, so we just return an
///         // empty vector.
///         Vec::new()
///     }
///
///     pub fn set_or_clear(optional: Option<Vec<u8>>) {
///         // Same as above
///     }
/// }
/// ```
///
/// # Argument types
///
/// The macro supports any kind of argument type, as long as it implements [`RIType`] and the
/// required `FromFFIValue`/`IntoFFIValue`. The macro will convert each
/// argument to the corresponding FFI representation and will call into the host using this FFI
/// representation. On the host each argument is converted back to the native representation
/// and the native implementation is called. Any return value is handled in the same way.
///
/// # Wasm only interfaces
///
/// Some interfaces are only required from within the wasm runtime e.g. the allocator
/// interface. To support this, the macro can be called like `#[runtime_interface(wasm_only)]`.
/// This instructs the macro to make two significant changes to the generated code:
///
/// 1. The generated functions are not callable from the native side.
/// 2. The trait as shown above is not implemented for [`Externalities`] and is instead
/// implemented for `FunctionContext` (from `sp-wasm-interface`).
///
/// # Disable tracing
/// By adding `no_tracing` to the list of options you can prevent the wasm-side interface from
/// generating the default `sp-tracing`-calls. Note that this is rarely needed but only meant
/// for the case when that would create a circular dependency. You usually _do not_ want to add
/// this flag, as tracing doesn't cost you anything by default anyways (it is added as a no-op)
/// but is super useful for debugging later.
pub use sp_runtime_interface_proc_macro::runtime_interface;
#[doc(hidden)]
#[cfg(feature = "std")]
pub use sp_externalities::{
    set_and_run_with_externalities, with_externalities, ExtensionStore, Externalities,
    ExternalitiesExt,
};
#[doc(hidden)]
pub use codec;
#[cfg(feature = "std")]
pub mod host {
    //! Traits required by the runtime interface from the host side.
    //! 运行时接口从主机端所需的特征。
    use crate::RIType;
    use sp_wasm_interface::{FunctionContext, Result};
    /// Something that can be converted into a ffi value.
    pub trait IntoFFIValue: RIType {
        /// Convert `self` into a ffi value.
        fn into_ffi_value(self, context: &mut dyn FunctionContext) -> Result<Self::FFIType>;
    }
    /// Something that can be converted into a preallocated ffi value.
    ///
    /// Every type parameter that should be given as `&mut` into a runtime interface function, needs
    /// to implement this trait. After executing the host implementation of the runtime interface
    /// function, the value is copied into the preallocated wasm memory.
    ///
    /// This should only be used for types which have a fixed size, like slices. Other types like a vec
    /// do not work with this interface, as we can not call into wasm to reallocate memory. So, this
    /// trait should be implemented carefully.
    pub trait IntoPreallocatedFFIValue: RIType {
        /// As `Self` can be an unsized type, it needs to be represented by a sized type at the host.
        /// This `SelfInstance` is the sized type.
        type SelfInstance;
        /// Convert `self_instance` into the given preallocated ffi value.
        fn into_preallocated_ffi_value(
            self_instance: Self::SelfInstance,
            context: &mut dyn FunctionContext,
            allocated: Self::FFIType,
        ) -> Result<()>;
    }
    /// Something that can be created from a ffi value.
    /// Implementations are safe to assume that the `arg` given to `from_ffi_value`
    /// is only generated by the corresponding [`wasm::IntoFFIValue`](crate::wasm::IntoFFIValue)
    /// implementation.
    pub trait FromFFIValue: RIType {
        /// As `Self` can be an unsized type, it needs to be represented by a sized type at the host.
        /// This `SelfInstance` is the sized type.
        type SelfInstance;
        /// Create `SelfInstance` from the given
        fn from_ffi_value(
            context: &mut dyn FunctionContext,
            arg: Self::FFIType,
        ) -> Result<Self::SelfInstance>;
    }
}
pub(crate) mod impls {
    //! Provides implementations for the runtime interface traits.
    //! 提供运行时接口特征的实现。
    #[cfg(feature = "std")]
    use crate::host::*;
    use crate::{
        pass_by::{Codec, Enum, Inner, PassBy, PassByInner},
        util::{pack_ptr_and_len, unpack_ptr_and_len},
        Pointer, RIType,
    };
    #[cfg(feature = "std")]
    use sp_wasm_interface::{FunctionContext, Result};
    use codec::{Decode, Encode};
    use sp_std::{any::TypeId, mem, vec::Vec};
    #[cfg(feature = "std")]
    use sp_std::borrow::Cow;
    /// The type is passed directly.
    impl RIType for u8 {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for u8 {
        type SelfInstance = u8;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: u32) -> Result<u8> {
            Ok(arg as u8)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for u8 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<u32> {
            Ok(self as u32)
        }
    }
    /// The type is passed directly.
    impl RIType for u16 {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for u16 {
        type SelfInstance = u16;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: u32) -> Result<u16> {
            Ok(arg as u16)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for u16 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<u32> {
            Ok(self as u32)
        }
    }
    /// The type is passed directly.
    impl RIType for u32 {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for u32 {
        type SelfInstance = u32;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: u32) -> Result<u32> {
            Ok(arg as u32)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for u32 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<u32> {
            Ok(self as u32)
        }
    }
    /// The type is passed directly.
    impl RIType for u64 {
        type FFIType = u64;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for u64 {
        type SelfInstance = u64;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: u64) -> Result<u64> {
            Ok(arg as u64)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for u64 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<u64> {
            Ok(self as u64)
        }
    }
    /// The type is passed directly.
    impl RIType for i8 {
        type FFIType = i32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for i8 {
        type SelfInstance = i8;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: i32) -> Result<i8> {
            Ok(arg as i8)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for i8 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<i32> {
            Ok(self as i32)
        }
    }
    /// The type is passed directly.
    impl RIType for i16 {
        type FFIType = i32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for i16 {
        type SelfInstance = i16;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: i32) -> Result<i16> {
            Ok(arg as i16)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for i16 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<i32> {
            Ok(self as i32)
        }
    }
    /// The type is passed directly.
    impl RIType for i32 {
        type FFIType = i32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for i32 {
        type SelfInstance = i32;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: i32) -> Result<i32> {
            Ok(arg as i32)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for i32 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<i32> {
            Ok(self as i32)
        }
    }
    /// The type is passed directly.
    impl RIType for i64 {
        type FFIType = i64;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for i64 {
        type SelfInstance = i64;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: i64) -> Result<i64> {
            Ok(arg as i64)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for i64 {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<i64> {
            Ok(self as i64)
        }
    }
    /// `bool` is passed as `u32`.
    ///
    /// - `1`: true
    /// - `0`: false
    impl RIType for bool {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for bool {
        type SelfInstance = bool;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: u32) -> Result<bool> {
            Ok(arg == 1)
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for bool {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<u32> {
            Ok(if self { 1 } else { 0 })
        }
    }
    /// The type is passed as `u64`.
    ///
    /// The `u64` value is build by `length 32bit << 32 | pointer 32bit`
    ///
    /// If `T == u8` the length and the pointer are taken directly from `Self`.
    /// Otherwise `Self` is encoded and the length and the pointer are taken from the encoded vector.
    impl<T> RIType for Vec<T> {
        type FFIType = u64;
    }
    #[cfg(feature = "std")]
    impl<T: 'static + Encode> IntoFFIValue for Vec<T> {
        fn into_ffi_value(self, context: &mut dyn FunctionContext) -> Result<u64> {
            let vec: Cow<'_, [u8]> = if TypeId::of::<T>() == TypeId::of::<u8>() {
                unsafe { Cow::Borrowed(mem::transmute(&self[..])) }
            } else {
                Cow::Owned(self.encode())
            };
            let ptr = context.allocate_memory(vec.as_ref().len() as u32)?;
            context.write_memory(ptr, &vec)?;
            Ok(pack_ptr_and_len(ptr.into(), vec.len() as u32))
        }
    }
    #[cfg(feature = "std")]
    impl<T: 'static + Decode> FromFFIValue for Vec<T> {
        type SelfInstance = Vec<T>;
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u64) -> Result<Vec<T>> {
            <[T] as FromFFIValue>::from_ffi_value(context, arg)
        }
    }
    /// The type is passed as `u64`.
    ///
    /// The `u64` value is build by `length 32bit << 32 | pointer 32bit`
    ///
    /// If `T == u8` the length and the pointer are taken directly from `Self`.
    /// Otherwise `Self` is encoded and the length and the pointer are taken from the encoded vector.
    impl<T> RIType for [T] {
        type FFIType = u64;
    }
    #[cfg(feature = "std")]
    impl<T: 'static + Decode> FromFFIValue for [T] {
        type SelfInstance = Vec<T>;
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u64) -> Result<Vec<T>> {
            let (ptr, len) = unpack_ptr_and_len(arg);
            let vec = context.read_memory(Pointer::new(ptr), len)?;
            if TypeId::of::<T>() == TypeId::of::<u8>() {
                Ok(unsafe { mem::transmute(vec) })
            } else {
                Ok(Vec::<T>::decode(&mut &vec[..])
                    .expect("Wasm to host values are encoded correctly; qed"))
            }
        }
    }
    #[cfg(feature = "std")]
    impl IntoPreallocatedFFIValue for [u8] {
        type SelfInstance = Vec<u8>;
        fn into_preallocated_ffi_value(
            self_instance: Self::SelfInstance,
            context: &mut dyn FunctionContext,
            allocated: u64,
        ) -> Result<()> {
            let (ptr, len) = unpack_ptr_and_len(allocated);
            if (len as usize) < self_instance.len() {
                Err({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[
                            "Preallocated buffer is not big enough (given ",
                            " vs needed ",
                            ")!",
                        ],
                        &[
                            ::core::fmt::ArgumentV1::new_display(&len),
                            ::core::fmt::ArgumentV1::new_display(&self_instance.len()),
                        ],
                    ));
                    res
                })
            } else {
                context.write_memory(Pointer::new(ptr), &self_instance)
            }
        }
    }
    /// The type is passed as `u32`.
    ///
    /// The `u32` is the pointer to the array.
    impl<const N: usize> RIType for [u8; N] {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl<const N: usize> FromFFIValue for [u8; N] {
        type SelfInstance = [u8; N];
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u32) -> Result<[u8; N]> {
            let mut res = [0u8; N];
            context.read_memory_into(Pointer::new(arg), &mut res)?;
            Ok(res)
        }
    }
    #[cfg(feature = "std")]
    impl<const N: usize> IntoFFIValue for [u8; N] {
        fn into_ffi_value(self, context: &mut dyn FunctionContext) -> Result<u32> {
            let addr = context.allocate_memory(N as u32)?;
            context.write_memory(addr, &self)?;
            Ok(addr.into())
        }
    }
    #[cfg(feature = "std")]
    impl<const N: usize> IntoPreallocatedFFIValue for [u8; N] {
        type SelfInstance = [u8; N];
        fn into_preallocated_ffi_value(
            self_instance: Self::SelfInstance,
            context: &mut dyn FunctionContext,
            allocated: u32,
        ) -> Result<()> {
            context.write_memory(Pointer::new(allocated), &self_instance)
        }
    }
    impl<T: codec::Codec, E: codec::Codec> PassBy for sp_std::result::Result<T, E> {
        type PassBy = Codec<Self>;
    }
    impl<T: codec::Codec> PassBy for Option<T> {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl PassBy for ()
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<TupleElement0> PassBy for (TupleElement0,)
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<TupleElement0, TupleElement1> PassBy for (TupleElement0, TupleElement1)
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<TupleElement0, TupleElement1, TupleElement2> PassBy
        for (TupleElement0, TupleElement1, TupleElement2)
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<TupleElement0, TupleElement1, TupleElement2, TupleElement3> PassBy
        for (TupleElement0, TupleElement1, TupleElement2, TupleElement3)
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<TupleElement0, TupleElement1, TupleElement2, TupleElement3, TupleElement4> PassBy
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
        > PassBy
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
        > PassBy
        for (
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
            TupleElement7,
        > PassBy
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
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
            TupleElement0,
            TupleElement1,
            TupleElement2,
            TupleElement3,
            TupleElement4,
            TupleElement5,
            TupleElement6,
            TupleElement7,
            TupleElement8,
        > PassBy
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
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
        > PassBy
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
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
        > PassBy
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
            TupleElement10,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
        > PassBy
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
            TupleElement10,
            TupleElement11,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
            TupleElement27,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
            TupleElement27,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
            TupleElement27,
            TupleElement28,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
            TupleElement27,
            TupleElement28,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    #[allow(unused)]
    impl<
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
            TupleElement27,
            TupleElement28,
            TupleElement29,
        > PassBy
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
            TupleElement10,
            TupleElement11,
            TupleElement12,
            TupleElement13,
            TupleElement14,
            TupleElement15,
            TupleElement16,
            TupleElement17,
            TupleElement18,
            TupleElement19,
            TupleElement20,
            TupleElement21,
            TupleElement22,
            TupleElement23,
            TupleElement24,
            TupleElement25,
            TupleElement26,
            TupleElement27,
            TupleElement28,
            TupleElement29,
        )
    where
        Self: codec::Codec,
    {
        type PassBy = Codec<Self>;
    }
    impl PassBy for primitive_types::H160 {
        type PassBy = Inner<Self, [u8; 20]>;
    }
    impl PassByInner for primitive_types::H160 {
        type Inner = [u8; 20];
        fn inner(&self) -> &Self::Inner {
            &self.0
        }
        fn into_inner(self) -> Self::Inner {
            self.0
        }
        fn from_inner(inner: Self::Inner) -> Self {
            Self(inner)
        }
    }
    impl PassBy for primitive_types::H256 {
        type PassBy = Inner<Self, [u8; 32]>;
    }
    impl PassByInner for primitive_types::H256 {
        type Inner = [u8; 32];
        fn inner(&self) -> &Self::Inner {
            &self.0
        }
        fn into_inner(self) -> Self::Inner {
            self.0
        }
        fn from_inner(inner: Self::Inner) -> Self {
            Self(inner)
        }
    }
    impl PassBy for primitive_types::H512 {
        type PassBy = Inner<Self, [u8; 64]>;
    }
    impl PassByInner for primitive_types::H512 {
        type Inner = [u8; 64];
        fn inner(&self) -> &Self::Inner {
            &self.0
        }
        fn into_inner(self) -> Self::Inner {
            self.0
        }
        fn from_inner(inner: Self::Inner) -> Self {
            Self(inner)
        }
    }
    /// The type is passed as `u64`.
    ///
    /// The `u64` value is build by `length 32bit << 32 | pointer 32bit`
    ///
    /// The length and the pointer are taken directly from `Self`.
    impl RIType for str {
        type FFIType = u64;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for str {
        type SelfInstance = String;
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u64) -> Result<String> {
            let (ptr, len) = unpack_ptr_and_len(arg);
            let vec = context.read_memory(Pointer::new(ptr), len)?;
            String::from_utf8(vec).map_err(|_| "Invalid utf8 data provided".into())
        }
    }
    #[cfg(feature = "std")]
    impl<T: sp_wasm_interface::PointerType> RIType for Pointer<T> {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl<T: sp_wasm_interface::PointerType> FromFFIValue for Pointer<T> {
        type SelfInstance = Self;
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: u32) -> Result<Self> {
            Ok(Pointer::new(arg))
        }
    }
    #[cfg(feature = "std")]
    impl<T: sp_wasm_interface::PointerType> IntoFFIValue for Pointer<T> {
        fn into_ffi_value(self, _: &mut dyn FunctionContext) -> Result<u32> {
            Ok(self.into())
        }
    }
    /// `u128`/`i128` is passed as `u32`.
    ///
    /// The `u32` is a pointer to an `[u8; 16]` array.
    impl RIType for u128 {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for u128 {
        type SelfInstance = u128;
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u32) -> Result<u128> {
            let mut res = [0u8; mem::size_of::<u128>()];
            context.read_memory_into(Pointer::new(arg), &mut res)?;
            Ok(<u128>::from_le_bytes(res))
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for u128 {
        fn into_ffi_value(self, context: &mut dyn FunctionContext) -> Result<u32> {
            let addr = context.allocate_memory(mem::size_of::<u128>() as u32)?;
            context.write_memory(addr, &self.to_le_bytes())?;
            Ok(addr.into())
        }
    }
    /// `u128`/`i128` is passed as `u32`.
    ///
    /// The `u32` is a pointer to an `[u8; 16]` array.
    impl RIType for i128 {
        type FFIType = u32;
    }
    #[cfg(feature = "std")]
    impl FromFFIValue for i128 {
        type SelfInstance = i128;
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: u32) -> Result<i128> {
            let mut res = [0u8; mem::size_of::<i128>()];
            context.read_memory_into(Pointer::new(arg), &mut res)?;
            Ok(<i128>::from_le_bytes(res))
        }
    }
    #[cfg(feature = "std")]
    impl IntoFFIValue for i128 {
        fn into_ffi_value(self, context: &mut dyn FunctionContext) -> Result<u32> {
            let addr = context.allocate_memory(mem::size_of::<i128>() as u32)?;
            context.write_memory(addr, &self.to_le_bytes())?;
            Ok(addr.into())
        }
    }
    impl PassBy for sp_wasm_interface::ValueType {
        type PassBy = Enum<sp_wasm_interface::ValueType>;
    }
    impl PassBy for sp_wasm_interface::Value {
        type PassBy = Codec<sp_wasm_interface::Value>;
    }
    impl PassBy for sp_storage::TrackedStorageKey {
        type PassBy = Codec<Self>;
    }
    impl PassBy for sp_storage::StateVersion {
        type PassBy = Enum<Self>;
    }
    impl PassBy for sp_externalities::MultiRemovalResults {
        type PassBy = Codec<Self>;
    }
}
pub mod pass_by {
    //! Provides the [`PassBy`](PassBy) trait to simplify the implementation of the
    //! runtime interface traits for custom types.
    //!
    //! [`Codec`], [`Inner`] and [`Enum`] are the provided strategy implementations.
    use crate::{
        util::{pack_ptr_and_len, unpack_ptr_and_len},
        RIType,
    };
    #[cfg(feature = "std")]
    use crate::host::*;
    #[cfg(feature = "std")]
    use sp_wasm_interface::{FunctionContext, Pointer, Result};
    use sp_std::marker::PhantomData;
    /// Derive macro for implementing [`PassBy`] with the [`Codec`] strategy.
    ///
    /// This requires that the type implements [`Encode`](codec::Encode) and
    /// [`Decode`](codec::Decode) from `parity-scale-codec`.
    ///
    /// # Example
    ///
    /// ```
    /// # use sp_runtime_interface::pass_by::PassByCodec;
    /// # use codec::{Encode, Decode};
    /// #[derive(PassByCodec, Encode, Decode)]
    /// struct EncodableType {
    ///     name: Vec<u8>,
    ///     param: u32,
    /// }
    /// ```
    pub use sp_runtime_interface_proc_macro::PassByCodec;
    /// Derive macro for implementing [`PassBy`] with the [`Inner`] strategy.
    ///
    /// Besides implementing [`PassBy`], this derive also implements the helper trait
    /// [`PassByInner`].
    ///
    /// The type is required to be a struct with just one field. The field type needs to implement
    /// the required traits to pass it between the wasm and the native side. (See the runtime
    /// interface crate for more information about these traits.)
    ///
    /// # Example
    ///
    /// ```
    /// # use sp_runtime_interface::pass_by::PassByInner;
    /// #[derive(PassByInner)]
    /// struct Data([u8; 32]);
    /// ```
    ///
    /// ```
    /// # use sp_runtime_interface::pass_by::PassByInner;
    /// #[derive(PassByInner)]
    /// struct Data {
    ///     data: [u8; 32],
    /// }
    /// ```
    pub use sp_runtime_interface_proc_macro::PassByInner;
    /// Derive macro for implementing [`PassBy`] with the [`Enum`] strategy.
    ///
    /// Besides implementing [`PassBy`], this derive also implements `TryFrom<u8>` and
    /// `From<Self> for u8` for the type.
    ///
    /// The type is required to be an enum with only unit variants and at maximum `256` variants.
    /// Also it is required that the type implements `Copy`.
    ///
    /// # Example
    ///
    /// ```
    /// # use sp_runtime_interface::pass_by::PassByEnum;
    /// #[derive(PassByEnum, Copy, Clone)]
    /// enum Data {
    ///     Okay,
    ///     NotOkay,
    ///     // This will not work with the derive.
    ///     //Why(u32),
    /// }
    /// ```
    pub use sp_runtime_interface_proc_macro::PassByEnum;
    /// Something that should be passed between wasm and the host using the given strategy.
    ///
    /// See [`Codec`], [`Inner`] or [`Enum`] for more information about the provided strategies.
    pub trait PassBy: Sized {
        /// The strategy that should be used to pass the type.
        type PassBy: PassByImpl<Self>;
    }
    /// Something that provides a strategy for passing a type between wasm and the host.
    ///
    /// This trait exposes the same functionality as [`crate::host::IntoFFIValue`] and
    /// [`crate::host::FromFFIValue`] to delegate the implementation for a type to a different type.
    ///
    /// This trait is used for the host implementation.
    #[cfg(feature = "std")]
    pub trait PassByImpl<T>: RIType {
        /// Convert the given instance to the ffi value.
        ///
        /// For more information see: [`crate::host::IntoFFIValue::into_ffi_value`]
        fn into_ffi_value(instance: T, context: &mut dyn FunctionContext) -> Result<Self::FFIType>;
        /// Create `T` from the given ffi value.
        ///
        /// For more information see: [`crate::host::FromFFIValue::from_ffi_value`]
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: Self::FFIType) -> Result<T>;
    }
    impl<T: PassBy> RIType for T {
        type FFIType = <T::PassBy as RIType>::FFIType;
    }
    #[cfg(feature = "std")]
    impl<T: PassBy> IntoFFIValue for T {
        fn into_ffi_value(
            self,
            context: &mut dyn FunctionContext,
        ) -> Result<<T::PassBy as RIType>::FFIType> {
            T::PassBy::into_ffi_value(self, context)
        }
    }
    #[cfg(feature = "std")]
    impl<T: PassBy> FromFFIValue for T {
        type SelfInstance = Self;
        fn from_ffi_value(
            context: &mut dyn FunctionContext,
            arg: <T::PassBy as RIType>::FFIType,
        ) -> Result<Self> {
            T::PassBy::from_ffi_value(context, arg)
        }
    }
    /// The implementation of the pass by codec strategy. This strategy uses a SCALE encoded
    /// representation of the type between wasm and the host.
    ///
    /// Use this type as associated type for [`PassBy`] to implement this strategy for a type.
    ///
    /// This type expects the type that wants to implement this strategy as generic parameter.
    ///
    /// [`PassByCodec`](derive.PassByCodec.html) is a derive macro to implement this strategy.
    ///
    /// # Example
    /// ```
    /// # use sp_runtime_interface::pass_by::{PassBy, Codec};
    /// #[derive(codec::Encode, codec::Decode)]
    /// struct Test;
    ///
    /// impl PassBy for Test {
    ///     type PassBy = Codec<Self>;
    /// }
    /// ```
    pub struct Codec<T: codec::Codec>(PhantomData<T>);
    #[cfg(feature = "std")]
    impl<T: codec::Codec> PassByImpl<T> for Codec<T> {
        fn into_ffi_value(instance: T, context: &mut dyn FunctionContext) -> Result<Self::FFIType> {
            let vec = instance.encode();
            let ptr = context.allocate_memory(vec.len() as u32)?;
            context.write_memory(ptr, &vec)?;
            Ok(pack_ptr_and_len(ptr.into(), vec.len() as u32))
        }
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: Self::FFIType) -> Result<T> {
            let (ptr, len) = unpack_ptr_and_len(arg);
            let vec = context.read_memory(Pointer::new(ptr), len)?;
            T::decode(&mut &vec[..]).map_err(|e| {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["Could not decode value from wasm: "],
                    &[::core::fmt::ArgumentV1::new_display(&e)],
                ));
                res
            })
        }
    }
    /// The type is passed as `u64`.
    ///
    /// The `u64` value is build by `length 32bit << 32 | pointer 32bit`
    ///
    /// `Self` is encoded and the length and the pointer are taken from the encoded vector.
    impl<T: codec::Codec> RIType for Codec<T> {
        type FFIType = u64;
    }
    /// Trait that needs to be implemented by a type that should be passed between wasm and the host,
    /// by using the inner type. See [`Inner`] for more information.
    pub trait PassByInner: Sized {
        /// The inner type that is wrapped by `Self`.
        type Inner: RIType;
        /// Consumes `self` and returns the inner type.
        fn into_inner(self) -> Self::Inner;
        /// Returns the reference to the inner type.
        fn inner(&self) -> &Self::Inner;
        /// Construct `Self` from the given `inner`.
        fn from_inner(inner: Self::Inner) -> Self;
    }
    /// The implementation of the pass by inner type strategy. The type that uses this strategy will be
    /// passed between wasm and the host by using the wrapped inner type. So, this strategy is only
    /// usable by newtype structs.
    ///
    /// Use this type as associated type for [`PassBy`] to implement this strategy for a type. Besides
    /// that the `PassByInner` trait need to be implemented as well.
    ///
    /// This type expects the type that wants to use this strategy as generic parameter `T` and the
    /// inner type as generic parameter `I`.
    ///
    /// [`PassByInner`](derive.PassByInner.html) is a derive macro to implement this strategy.
    ///
    /// # Example
    /// ```
    /// # use sp_runtime_interface::pass_by::{PassBy, Inner, PassByInner};
    /// struct Test([u8; 32]);
    ///
    /// impl PassBy for Test {
    ///     type PassBy = Inner<Self, [u8; 32]>;
    /// }
    ///
    /// impl PassByInner for Test {
    ///     type Inner = [u8; 32];
    ///
    ///     fn into_inner(self) -> [u8; 32] {
    ///         self.0
    ///     }
    ///     fn inner(&self) -> &[u8; 32] {
    ///         &self.0
    ///     }
    ///     fn from_inner(inner: [u8; 32]) -> Self {
    ///         Self(inner)
    ///     }
    /// }
    /// ```
    pub struct Inner<T: PassByInner<Inner = I>, I: RIType>(PhantomData<(T, I)>);
    #[cfg(feature = "std")]
    impl<T: PassByInner<Inner = I>, I: RIType> PassByImpl<T> for Inner<T, I>
    where
        I: IntoFFIValue + FromFFIValue<SelfInstance = I>,
    {
        fn into_ffi_value(instance: T, context: &mut dyn FunctionContext) -> Result<Self::FFIType> {
            instance.into_inner().into_ffi_value(context)
        }
        fn from_ffi_value(context: &mut dyn FunctionContext, arg: Self::FFIType) -> Result<T> {
            I::from_ffi_value(context, arg).map(T::from_inner)
        }
    }
    /// The type is passed as the inner type.
    impl<T: PassByInner<Inner = I>, I: RIType> RIType for Inner<T, I> {
        type FFIType = I::FFIType;
    }
    /// The implementation of the pass by enum strategy. This strategy uses an `u8` internally to pass
    /// the enum between wasm and the host. So, this strategy only supports enums with unit variants.
    ///
    /// Use this type as associated type for [`PassBy`] to implement this strategy for a type.
    ///
    /// This type expects the type that wants to implement this strategy as generic parameter. Besides
    /// that the type needs to implement `TryFrom<u8>` and `From<Self> for u8`.
    ///
    /// [`PassByEnum`](derive.PassByEnum.html) is a derive macro to implement this strategy.
    ///
    /// # Example
    /// ```
    /// # use sp_runtime_interface::pass_by::{PassBy, Enum};
    /// #[derive(Clone, Copy)]
    /// enum Test {
    ///     Test1,
    ///     Test2,
    /// }
    ///
    /// impl From<Test> for u8 {
    ///     fn from(val: Test) -> u8 {
    ///         match val {
    ///             Test::Test1 => 0,
    ///             Test::Test2 => 1,
    ///         }
    ///     }
    /// }
    ///
    /// impl TryFrom<u8> for Test {
    ///     type Error = ();
    ///
    ///     fn try_from(val: u8) -> Result<Test, ()> {
    ///         match val {
    ///             0 => Ok(Test::Test1),
    ///             1 => Ok(Test::Test2),
    ///             _ => Err(()),
    ///         }
    ///     }
    /// }
    ///
    /// impl PassBy for Test {
    ///     type PassBy = Enum<Self>;
    /// }
    /// ```
    pub struct Enum<T: Copy + Into<u8> + TryFrom<u8>>(PhantomData<T>);
    #[cfg(feature = "std")]
    impl<T: Copy + Into<u8> + TryFrom<u8>> PassByImpl<T> for Enum<T> {
        fn into_ffi_value(instance: T, _: &mut dyn FunctionContext) -> Result<Self::FFIType> {
            Ok(instance.into() as u32)
        }
        fn from_ffi_value(_: &mut dyn FunctionContext, arg: Self::FFIType) -> Result<T> {
            T::try_from(arg as u8).map_err(|_| {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["Invalid enum discriminant: "],
                    &[::core::fmt::ArgumentV1::new_display(&arg)],
                ));
                res
            })
        }
    }
    /// The type is passed as `u32`.
    ///
    /// The value is corresponds to the discriminant of the variant.
    impl<T: Copy + Into<u8> + TryFrom<u8>> RIType for Enum<T> {
        type FFIType = u32;
    }
}
mod util {
    //! Various utilities that help interfacing with wasm runtime code.
    //! 帮助与 wasm 运行时代码接口的各种实用程序。
    /// Pack a pointer and length into an `u64`.
    /// 将指针和长度打包到 u64.
    pub fn pack_ptr_and_len(ptr: u32, len: u32) -> u64 {
        (u64::from(len) << 32) | u64::from(ptr)
    }
    /// Unpacks an `u64` into the pointer and length.
    ///
    /// Runtime API functions return a 64-bit value which encodes a pointer in the least-significant
    /// 32-bits and a length in the most-significant 32 bits. This interprets the returned value as a
    /// pointer, length tuple.
    ///
    /// 解压缩为 u64 指针和长度。
    /// 运行时 API 函数返回一个 64 位值，该值以最低有效 32 位对指针进行编码，以最高有效 32 位对长度进行编码。这会将返回的值解释为指针、长度元组。
    pub fn unpack_ptr_and_len(val: u64) -> (u32, u32) {
        let ptr = (val & (!0u32 as u64)) as u32;
        let len = (val >> 32) as u32;
        (ptr, len)
    }
}
pub use util::{pack_ptr_and_len, unpack_ptr_and_len};
/// Something that can be used by the runtime interface as type to communicate between wasm and the
/// host.
///
/// Every type that should be used in a runtime interface function signature needs to implement
/// this trait.
pub trait RIType {
    /// The ffi type that is used to represent `Self`.
    #[cfg(feature = "std")]
    type FFIType: sp_wasm_interface::IntoValue
        + sp_wasm_interface::TryFromValue
        + sp_wasm_interface::WasmTy;
}
/// A pointer that can be used in a runtime interface function signature.
#[cfg(feature = "std")]
pub type Pointer<T> = sp_wasm_interface::Pointer<T>;
