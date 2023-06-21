#![feature(prelude_import)]
//! Types and traits for interfacing between the host and the wasm runtime.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use sp_std::{borrow::Cow, iter::Iterator, marker::PhantomData, mem, result, vec, vec::Vec};
#[cfg(feature = "wasmi")]
mod wasmi_impl {
    //! Implementation of conversions between Substrate and wasmi types.
    use crate::{Signature, Value, ValueType};
    use sp_std::vec::Vec;
    impl From<Value> for wasmi::RuntimeValue {
        fn from(value: Value) -> Self {
            match value {
                Value::I32(val) => Self::I32(val),
                Value::I64(val) => Self::I64(val),
                Value::F32(val) => Self::F32(val.into()),
                Value::F64(val) => Self::F64(val.into()),
            }
        }
    }
    impl From<wasmi::RuntimeValue> for Value {
        fn from(value: wasmi::RuntimeValue) -> Self {
            match value {
                wasmi::RuntimeValue::I32(val) => Self::I32(val),
                wasmi::RuntimeValue::I64(val) => Self::I64(val),
                wasmi::RuntimeValue::F32(val) => Self::F32(val.into()),
                wasmi::RuntimeValue::F64(val) => Self::F64(val.into()),
            }
        }
    }
    impl From<ValueType> for wasmi::ValueType {
        fn from(value: ValueType) -> Self {
            match value {
                ValueType::I32 => Self::I32,
                ValueType::I64 => Self::I64,
                ValueType::F32 => Self::F32,
                ValueType::F64 => Self::F64,
            }
        }
    }
    impl From<wasmi::ValueType> for ValueType {
        fn from(value: wasmi::ValueType) -> Self {
            match value {
                wasmi::ValueType::I32 => Self::I32,
                wasmi::ValueType::I64 => Self::I64,
                wasmi::ValueType::F32 => Self::F32,
                wasmi::ValueType::F64 => Self::F64,
            }
        }
    }
    impl From<Signature> for wasmi::Signature {
        fn from(sig: Signature) -> Self {
            let args = sig.args.iter().map(|a| (*a).into()).collect::<Vec<_>>();
            wasmi::Signature::new(args, sig.return_value.map(Into::into))
        }
    }
    impl From<&wasmi::Signature> for Signature {
        fn from(sig: &wasmi::Signature) -> Self {
            Signature::new(
                sig.params()
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
                sig.return_type().map(Into::into),
            )
        }
    }
}
pub use wasmtime;
pub use anyhow;
/// Result type used by traits in this crate.
#[cfg(feature = "std")]
pub type Result<T> = result::Result<T, String>;
/// Value types supported by Substrate on the boundary between host/Wasm.
pub enum ValueType {
    /// An `i32` value type.
    I32,
    /// An `i64` value type.
    I64,
    /// An `f32` value type.
    F32,
    /// An `f64` value type.
    F64,
}
#[automatically_derived]
impl ::core::marker::Copy for ValueType {}
#[automatically_derived]
impl ::core::clone::Clone for ValueType {
    #[inline]
    fn clone(&self) -> ValueType {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ValueType {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ValueType {
    #[inline]
    fn eq(&self, other: &ValueType) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ValueType {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ValueType::I32 => ::core::fmt::Formatter::write_str(f, "I32"),
            ValueType::I64 => ::core::fmt::Formatter::write_str(f, "I64"),
            ValueType::F32 => ::core::fmt::Formatter::write_str(f, "F32"),
            ValueType::F64 => ::core::fmt::Formatter::write_str(f, "F64"),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for ValueType {}
#[automatically_derived]
impl ::core::cmp::Eq for ValueType {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
impl From<ValueType> for u8 {
    fn from(val: ValueType) -> u8 {
        match val {
            ValueType::I32 => 0,
            ValueType::I64 => 1,
            ValueType::F32 => 2,
            ValueType::F64 => 3,
        }
    }
}
impl TryFrom<u8> for ValueType {
    type Error = ();
    fn try_from(val: u8) -> sp_std::result::Result<ValueType, ()> {
        match val {
            0 => Ok(Self::I32),
            1 => Ok(Self::I64),
            2 => Ok(Self::F32),
            3 => Ok(Self::F64),
            _ => Err(()),
        }
    }
}
/// Values supported by Substrate on the boundary between host/Wasm.
pub enum Value {
    /// A 32-bit integer.
    I32(i32),
    /// A 64-bit integer.
    I64(i64),
    /// A 32-bit floating-point number stored as raw bit pattern.
    ///
    /// You can materialize this value using `f32::from_bits`.
    F32(u32),
    /// A 64-bit floating-point number stored as raw bit pattern.
    ///
    /// You can materialize this value using `f64::from_bits`.
    F64(u64),
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Value {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Value {
    #[inline]
    fn eq(&self, other: &Value) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
            && match (self, other) {
                (Value::I32(__self_0), Value::I32(__arg1_0)) => *__self_0 == *__arg1_0,
                (Value::I64(__self_0), Value::I64(__arg1_0)) => *__self_0 == *__arg1_0,
                (Value::F32(__self_0), Value::F32(__arg1_0)) => *__self_0 == *__arg1_0,
                (Value::F64(__self_0), Value::F64(__arg1_0)) => *__self_0 == *__arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() },
            }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Value {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Value::I32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I32", &__self_0)
            }
            Value::I64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I64", &__self_0)
            }
            Value::F32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "F32", &__self_0)
            }
            Value::F64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "F64", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Value {
    #[inline]
    fn clone(&self) -> Value {
        let _: ::core::clone::AssertParamIsClone<i32>;
        let _: ::core::clone::AssertParamIsClone<i64>;
        let _: ::core::clone::AssertParamIsClone<u32>;
        let _: ::core::clone::AssertParamIsClone<u64>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for Value {}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for Value {
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            match *self {
                Value::I32(ref aa) => {
                    __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                    ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                Value::I64(ref aa) => {
                    __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                Value::F32(ref aa) => {
                    __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                    ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                Value::F64(ref aa) => {
                    __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                    ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                _ => (),
            }
        }
    }
    #[automatically_derived]
    impl ::codec::EncodeLike for Value {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for Value {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            match __codec_input_edqy
                .read_byte()
                .map_err(|e| e.chain("Could not decode `Value`, failed to read variant byte"))?
            {
                __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(Value::I32({
                        let __codec_res_edqy = <i32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Value::I32.0`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    }))
                }
                __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(Value::I64({
                        let __codec_res_edqy = <i64 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Value::I64.0`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    }))
                }
                __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(Value::F32({
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Value::F32.0`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    }))
                }
                __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(Value::F64({
                        let __codec_res_edqy = <u64 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Value::F64.0`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    }))
                }
                _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                    "Could not decode `Value`, variant doesn't exist",
                )),
            }
        }
    }
};
impl Value {
    /// Returns the type of this value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        }
    }
    /// Return `Self` as `i32`.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::I32(val) => Some(*val),
            _ => None,
        }
    }
}
/// Provides `Sealed` trait to prevent implementing trait `PointerType` and `WasmTy` outside of this
/// crate.
mod private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
}
/// Something that can be wrapped in a wasm `Pointer`.
///
/// This trait is sealed.
pub trait PointerType: Sized + private::Sealed {
    /// The size of the type in wasm.
    const SIZE: u32 = mem::size_of::<Self>() as u32;
}
impl PointerType for u8 {}
impl PointerType for u16 {}
impl PointerType for u32 {}
impl PointerType for u64 {}
/// Type to represent a pointer in wasm at the host.
pub struct Pointer<T: PointerType> {
    ptr: u32,
    _marker: PhantomData<T>,
}
#[automatically_derived]
impl<T: ::core::fmt::Debug + PointerType> ::core::fmt::Debug for Pointer<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Pointer",
            "ptr",
            &&self.ptr,
            "_marker",
            &&self._marker,
        )
    }
}
#[automatically_derived]
impl<T: PointerType> ::core::marker::StructuralPartialEq for Pointer<T> {}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq + PointerType> ::core::cmp::PartialEq for Pointer<T> {
    #[inline]
    fn eq(&self, other: &Pointer<T>) -> bool {
        self.ptr == other.ptr && self._marker == other._marker
    }
}
#[automatically_derived]
impl<T: PointerType> ::core::marker::StructuralEq for Pointer<T> {}
#[automatically_derived]
impl<T: ::core::cmp::Eq + PointerType> ::core::cmp::Eq for Pointer<T> {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u32>;
        let _: ::core::cmp::AssertParamIsEq<PhantomData<T>>;
    }
}
#[automatically_derived]
impl<T: ::core::clone::Clone + PointerType> ::core::clone::Clone for Pointer<T> {
    #[inline]
    fn clone(&self) -> Pointer<T> {
        Pointer {
            ptr: ::core::clone::Clone::clone(&self.ptr),
            _marker: ::core::clone::Clone::clone(&self._marker),
        }
    }
}
#[automatically_derived]
impl<T: ::core::marker::Copy + PointerType> ::core::marker::Copy for Pointer<T> {}
impl<T: PointerType> Pointer<T> {
    /// Create a new instance of `Self`.
    pub fn new(ptr: u32) -> Self {
        Self {
            ptr,
            _marker: Default::default(),
        }
    }
    /// Calculate the offset from this pointer.
    ///
    /// `offset` is in units of `T`. So, `3` means `3 * mem::size_of::<T>()` as offset to the
    /// pointer.
    ///
    /// Returns an `Option` to respect that the pointer could probably overflow.
    pub fn offset(self, offset: u32) -> Option<Self> {
        offset
            .checked_mul(T::SIZE)
            .and_then(|o| self.ptr.checked_add(o))
            .map(|ptr| Self {
                ptr,
                _marker: Default::default(),
            })
    }
    /// Create a null pointer.
    pub fn null() -> Self {
        Self::new(0)
    }
    /// Cast this pointer of type `T` to a pointer of type `R`.
    pub fn cast<R: PointerType>(self) -> Pointer<R> {
        Pointer::new(self.ptr)
    }
}
impl<T: PointerType> From<u32> for Pointer<T> {
    fn from(ptr: u32) -> Self {
        Pointer::new(ptr)
    }
}
impl<T: PointerType> From<Pointer<T>> for u32 {
    fn from(ptr: Pointer<T>) -> Self {
        ptr.ptr
    }
}
impl<T: PointerType> From<Pointer<T>> for u64 {
    fn from(ptr: Pointer<T>) -> Self {
        u64::from(ptr.ptr)
    }
}
impl<T: PointerType> From<Pointer<T>> for usize {
    fn from(ptr: Pointer<T>) -> Self {
        ptr.ptr as _
    }
}
impl<T: PointerType> IntoValue for Pointer<T> {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self.ptr as _)
    }
}
impl<T: PointerType> TryFromValue for Pointer<T> {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(Self::new(val as _)),
            _ => None,
        }
    }
}
/// The word size used in wasm. Normally known as `usize` in Rust.
pub type WordSize = u32;
/// The Signature of a function
pub struct Signature {
    /// The arguments of a function.
    pub args: Cow<'static, [ValueType]>,
    /// The optional return value of a function.
    pub return_value: Option<ValueType>,
}
#[automatically_derived]
impl ::core::marker::StructuralEq for Signature {}
#[automatically_derived]
impl ::core::cmp::Eq for Signature {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<Cow<'static, [ValueType]>>;
        let _: ::core::cmp::AssertParamIsEq<Option<ValueType>>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Signature {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Signature {
    #[inline]
    fn eq(&self, other: &Signature) -> bool {
        self.args == other.args && self.return_value == other.return_value
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Signature {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Signature",
            "args",
            &&self.args,
            "return_value",
            &&self.return_value,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Signature {
    #[inline]
    fn clone(&self) -> Signature {
        Signature {
            args: ::core::clone::Clone::clone(&self.args),
            return_value: ::core::clone::Clone::clone(&self.return_value),
        }
    }
}
impl Signature {
    /// Create a new instance of `Signature`.
    pub fn new<T: Into<Cow<'static, [ValueType]>>>(
        args: T,
        return_value: Option<ValueType>,
    ) -> Self {
        Self {
            args: args.into(),
            return_value,
        }
    }
    /// Create a new instance of `Signature` with the given `args` and without any return value.
    pub fn new_with_args<T: Into<Cow<'static, [ValueType]>>>(args: T) -> Self {
        Self {
            args: args.into(),
            return_value: None,
        }
    }
}
/// A trait that requires `RefUnwindSafe` when `feature = std`.
#[cfg(feature = "std")]
pub trait MaybeRefUnwindSafe: std::panic::RefUnwindSafe {}
#[cfg(feature = "std")]
impl<T: std::panic::RefUnwindSafe> MaybeRefUnwindSafe for T {}
/// Something that provides a function implementation on the host for a wasm function.
/// 在主机上为 wasm 函数提供函数实现的东西。
pub trait Function: MaybeRefUnwindSafe + Send + Sync {
    /// Returns the name of this function.
    fn name(&self) -> &str;
    /// Returns the signature of this function.
    fn signature(&self) -> Signature;
    /// Execute this function with the given arguments.
    fn execute(
        &self,
        context: &mut dyn FunctionContext,
        args: &mut dyn Iterator<Item = Value>,
    ) -> Result<Option<Value>>;
}
impl PartialEq for dyn Function {
    fn eq(&self, other: &Self) -> bool {
        other.name() == self.name() && other.signature() == self.signature()
    }
}
/// Context used by `Function` to interact with the allocator and the memory of the wasm instance.
/// “函数”用于与分配器和 wasm 实例的内存交互的上下文。
pub trait FunctionContext {
    /// Read memory from `address` into a vector.
    fn read_memory(&self, address: Pointer<u8>, size: WordSize) -> Result<Vec<u8>> {
        let mut vec = ::alloc::vec::from_elem(0, size as usize);
        self.read_memory_into(address, &mut vec)?;
        Ok(vec)
    }
    /// Read memory into the given `dest` buffer from `address`.
    fn read_memory_into(&self, address: Pointer<u8>, dest: &mut [u8]) -> Result<()>;
    /// Write the given data at `address` into the memory.
    fn write_memory(&mut self, address: Pointer<u8>, data: &[u8]) -> Result<()>;
    /// Allocate a memory instance of `size` bytes.
    fn allocate_memory(&mut self, size: WordSize) -> Result<Pointer<u8>>;
    /// Deallocate a given memory instance.
    fn deallocate_memory(&mut self, ptr: Pointer<u8>) -> Result<()>;
    /// Registers a panic error message within the executor.
    ///
    /// This is meant to be used in situations where the runtime
    /// encounters an unrecoverable error and intends to panic.
    ///
    /// Panicking in WASM is done through the [`unreachable`](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction which causes an unconditional trap and immediately aborts
    /// the execution. It does not however allow for any diagnostics to be
    /// passed through to the host, so while we do know that *something* went
    /// wrong we don't have any direct indication of what *exactly* went wrong.
    ///
    /// As a workaround we use this method right before the execution is
    /// actually aborted to pass an error message to the host so that it
    /// can associate it with the next trap, and return that to the caller.
    ///
    /// A WASM trap should be triggered immediately after calling this method;
    /// otherwise the error message might be associated with a completely
    /// unrelated trap.
    ///
    /// It should only be called once, however calling it more than once
    /// is harmless and will overwrite the previously set error message.
    fn register_panic_error_message(&mut self, message: &str);
}
/// A trait used to statically register host callbacks with the WASM executor,
/// so that they call be called from within the runtime with minimal overhead.
///
/// This is used internally to interface the wasmtime-based executor with the
/// host functions' definitions generated through the runtime interface macro,
/// and is not meant to be used directly.
/// 一种特征，用于向 WASM 执行程序静态注册主机回调，以便以最小的开销从运行时调用它们。
/// 这在内部用于将基于 wasmtime 的执行程序与通过运行时接口宏生成的主机函数定义进行接口，而不是直接使用。
/// 即sp_io里定义的几组runtime_interface接口
pub trait HostFunctionRegistry {
    type State;
    type Error;
    type FunctionContext: FunctionContext;
    /// Wraps the given `caller` in a type which implements `FunctionContext`
    /// and calls the given `callback`.
    fn with_function_context<R>(
        caller: wasmtime::Caller<Self::State>,
        callback: impl FnOnce(&mut dyn FunctionContext) -> R,
    ) -> R;
    /// Registers a given host function with the WASM executor.
    ///
    /// The function has to be statically callable, and all of its arguments
    /// and its return value have to be compatible with WASM FFI.
    fn register_static<Params, Results>(
        &mut self,
        fn_name: &str,
        func: impl wasmtime::IntoFunc<Self::State, Params, Results> + 'static,
    ) -> core::result::Result<(), Self::Error>;
}
/// Something that provides implementations for host functions.
pub trait HostFunctions: 'static + Send + Sync {
    /// Returns the host functions `Self` provides.
    fn host_functions() -> Vec<&'static dyn Function>;
    /// Statically registers the host functions.
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry;
}
#[allow(unused)]
impl HostFunctions for () {
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        Ok(())
    }
}
#[allow(unused)]
impl<TupleElement0: HostFunctions> HostFunctions for (TupleElement0,) {
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<TupleElement0: HostFunctions, TupleElement1: HostFunctions> HostFunctions
    for (TupleElement0, TupleElement1)
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<TupleElement0: HostFunctions, TupleElement1: HostFunctions, TupleElement2: HostFunctions>
    HostFunctions for (TupleElement0, TupleElement1, TupleElement2)
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
    > HostFunctions for (TupleElement0, TupleElement1, TupleElement2, TupleElement3)
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
    > HostFunctions
    for (
        TupleElement0,
        TupleElement1,
        TupleElement2,
        TupleElement3,
        TupleElement4,
    )
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
    > HostFunctions
    for (
        TupleElement0,
        TupleElement1,
        TupleElement2,
        TupleElement3,
        TupleElement4,
        TupleElement5,
    )
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
    > HostFunctions
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
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
    > HostFunctions
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
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
    > HostFunctions
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
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
    > HostFunctions
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
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
        TupleElement24: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions.extend(TupleElement24::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        TupleElement24::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
        TupleElement24: HostFunctions,
        TupleElement25: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions.extend(TupleElement24::host_functions());
        host_functions.extend(TupleElement25::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        TupleElement24::register_static(registry)?;
        TupleElement25::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
        TupleElement24: HostFunctions,
        TupleElement25: HostFunctions,
        TupleElement26: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions.extend(TupleElement24::host_functions());
        host_functions.extend(TupleElement25::host_functions());
        host_functions.extend(TupleElement26::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        TupleElement24::register_static(registry)?;
        TupleElement25::register_static(registry)?;
        TupleElement26::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
        TupleElement24: HostFunctions,
        TupleElement25: HostFunctions,
        TupleElement26: HostFunctions,
        TupleElement27: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions.extend(TupleElement24::host_functions());
        host_functions.extend(TupleElement25::host_functions());
        host_functions.extend(TupleElement26::host_functions());
        host_functions.extend(TupleElement27::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        TupleElement24::register_static(registry)?;
        TupleElement25::register_static(registry)?;
        TupleElement26::register_static(registry)?;
        TupleElement27::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
        TupleElement24: HostFunctions,
        TupleElement25: HostFunctions,
        TupleElement26: HostFunctions,
        TupleElement27: HostFunctions,
        TupleElement28: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions.extend(TupleElement24::host_functions());
        host_functions.extend(TupleElement25::host_functions());
        host_functions.extend(TupleElement26::host_functions());
        host_functions.extend(TupleElement27::host_functions());
        host_functions.extend(TupleElement28::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        TupleElement24::register_static(registry)?;
        TupleElement25::register_static(registry)?;
        TupleElement26::register_static(registry)?;
        TupleElement27::register_static(registry)?;
        TupleElement28::register_static(registry)?;
        Ok(())
    }
}
#[allow(unused)]
impl<
        TupleElement0: HostFunctions,
        TupleElement1: HostFunctions,
        TupleElement2: HostFunctions,
        TupleElement3: HostFunctions,
        TupleElement4: HostFunctions,
        TupleElement5: HostFunctions,
        TupleElement6: HostFunctions,
        TupleElement7: HostFunctions,
        TupleElement8: HostFunctions,
        TupleElement9: HostFunctions,
        TupleElement10: HostFunctions,
        TupleElement11: HostFunctions,
        TupleElement12: HostFunctions,
        TupleElement13: HostFunctions,
        TupleElement14: HostFunctions,
        TupleElement15: HostFunctions,
        TupleElement16: HostFunctions,
        TupleElement17: HostFunctions,
        TupleElement18: HostFunctions,
        TupleElement19: HostFunctions,
        TupleElement20: HostFunctions,
        TupleElement21: HostFunctions,
        TupleElement22: HostFunctions,
        TupleElement23: HostFunctions,
        TupleElement24: HostFunctions,
        TupleElement25: HostFunctions,
        TupleElement26: HostFunctions,
        TupleElement27: HostFunctions,
        TupleElement28: HostFunctions,
        TupleElement29: HostFunctions,
    > HostFunctions
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
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut host_functions = Vec::new();
        host_functions.extend(TupleElement0::host_functions());
        host_functions.extend(TupleElement1::host_functions());
        host_functions.extend(TupleElement2::host_functions());
        host_functions.extend(TupleElement3::host_functions());
        host_functions.extend(TupleElement4::host_functions());
        host_functions.extend(TupleElement5::host_functions());
        host_functions.extend(TupleElement6::host_functions());
        host_functions.extend(TupleElement7::host_functions());
        host_functions.extend(TupleElement8::host_functions());
        host_functions.extend(TupleElement9::host_functions());
        host_functions.extend(TupleElement10::host_functions());
        host_functions.extend(TupleElement11::host_functions());
        host_functions.extend(TupleElement12::host_functions());
        host_functions.extend(TupleElement13::host_functions());
        host_functions.extend(TupleElement14::host_functions());
        host_functions.extend(TupleElement15::host_functions());
        host_functions.extend(TupleElement16::host_functions());
        host_functions.extend(TupleElement17::host_functions());
        host_functions.extend(TupleElement18::host_functions());
        host_functions.extend(TupleElement19::host_functions());
        host_functions.extend(TupleElement20::host_functions());
        host_functions.extend(TupleElement21::host_functions());
        host_functions.extend(TupleElement22::host_functions());
        host_functions.extend(TupleElement23::host_functions());
        host_functions.extend(TupleElement24::host_functions());
        host_functions.extend(TupleElement25::host_functions());
        host_functions.extend(TupleElement26::host_functions());
        host_functions.extend(TupleElement27::host_functions());
        host_functions.extend(TupleElement28::host_functions());
        host_functions.extend(TupleElement29::host_functions());
        host_functions
    }
    #[cfg(all(feature = "std", feature = "wasmtime"))]
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        TupleElement0::register_static(registry)?;
        TupleElement1::register_static(registry)?;
        TupleElement2::register_static(registry)?;
        TupleElement3::register_static(registry)?;
        TupleElement4::register_static(registry)?;
        TupleElement5::register_static(registry)?;
        TupleElement6::register_static(registry)?;
        TupleElement7::register_static(registry)?;
        TupleElement8::register_static(registry)?;
        TupleElement9::register_static(registry)?;
        TupleElement10::register_static(registry)?;
        TupleElement11::register_static(registry)?;
        TupleElement12::register_static(registry)?;
        TupleElement13::register_static(registry)?;
        TupleElement14::register_static(registry)?;
        TupleElement15::register_static(registry)?;
        TupleElement16::register_static(registry)?;
        TupleElement17::register_static(registry)?;
        TupleElement18::register_static(registry)?;
        TupleElement19::register_static(registry)?;
        TupleElement20::register_static(registry)?;
        TupleElement21::register_static(registry)?;
        TupleElement22::register_static(registry)?;
        TupleElement23::register_static(registry)?;
        TupleElement24::register_static(registry)?;
        TupleElement25::register_static(registry)?;
        TupleElement26::register_static(registry)?;
        TupleElement27::register_static(registry)?;
        TupleElement28::register_static(registry)?;
        TupleElement29::register_static(registry)?;
        Ok(())
    }
}
/// A wrapper which merges two sets of host functions, and allows the second set to override
/// the host functions from the first set.
pub struct ExtendedHostFunctions<Base, Overlay> {
    phantom: PhantomData<(Base, Overlay)>,
}
impl<Base, Overlay> HostFunctions for ExtendedHostFunctions<Base, Overlay>
where
    Base: HostFunctions,
    Overlay: HostFunctions,
{
    fn host_functions() -> Vec<&'static dyn Function> {
        let mut base = Base::host_functions();
        let overlay = Overlay::host_functions();
        base.retain(|host_fn| {
            !overlay
                .iter()
                .any(|ext_host_fn| host_fn.name() == ext_host_fn.name())
        });
        base.extend(overlay);
        base
    }
    fn register_static<T>(registry: &mut T) -> core::result::Result<(), T::Error>
    where
        T: HostFunctionRegistry,
    {
        struct Proxy<'a, T> {
            registry: &'a mut T,
            seen_overlay: std::collections::HashSet<String>,
            seen_base: std::collections::HashSet<String>,
            overlay_registered: bool,
        }
        impl<'a, T> HostFunctionRegistry for Proxy<'a, T>
        where
            T: HostFunctionRegistry,
        {
            type State = T::State;
            type Error = T::Error;
            type FunctionContext = T::FunctionContext;
            fn with_function_context<R>(
                caller: wasmtime::Caller<Self::State>,
                callback: impl FnOnce(&mut dyn FunctionContext) -> R,
            ) -> R {
                T::with_function_context(caller, callback)
            }
            fn register_static<Params, Results>(
                &mut self,
                fn_name: &str,
                func: impl wasmtime::IntoFunc<Self::State, Params, Results> + 'static,
            ) -> core::result::Result<(), Self::Error> {
                if self.overlay_registered {
                    if !self.seen_base.insert(fn_name.to_owned()) {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::core::fmt::Arguments::new_v1(
                                        &["Duplicate base host function: \'", "\'"],
                                        &[::core::fmt::ArgumentV1::new_display(&fn_name)],
                                    ),
                                    lvl,
                                    &(
                                        "extended_host_functions",
                                        "sp_wasm_interface",
                                        "primitives/wasm-interface/src/lib.rs",
                                        455u32,
                                    ),
                                    ::log::__private_api::Option::None,
                                );
                            }
                        };
                        return Ok(());
                    }
                    if self.seen_overlay.contains(fn_name) {
                        {
                            let lvl = ::log::Level::Debug;
                            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                ::log::__private_api_log(
                                    ::core::fmt::Arguments::new_v1(
                                        &["Overriding base host function: \'", "\'"],
                                        &[::core::fmt::ArgumentV1::new_display(&fn_name)],
                                    ),
                                    lvl,
                                    &(
                                        "extended_host_functions",
                                        "sp_wasm_interface",
                                        "primitives/wasm-interface/src/lib.rs",
                                        467u32,
                                    ),
                                    ::log::__private_api::Option::None,
                                );
                            }
                        };
                        return Ok(());
                    }
                } else if !self.seen_overlay.insert(fn_name.to_owned()) {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api_log(
                                ::core::fmt::Arguments::new_v1(
                                    &["Duplicate overlay host function: \'", "\'"],
                                    &[::core::fmt::ArgumentV1::new_display(&fn_name)],
                                ),
                                lvl,
                                &(
                                    "extended_host_functions",
                                    "sp_wasm_interface",
                                    "primitives/wasm-interface/src/lib.rs",
                                    476u32,
                                ),
                                ::log::__private_api::Option::None,
                            );
                        }
                    };
                    return Ok(());
                }
                self.registry.register_static(fn_name, func)
            }
        }
        let mut proxy = Proxy {
            registry,
            seen_overlay: Default::default(),
            seen_base: Default::default(),
            overlay_registered: false,
        };
        Overlay::register_static(&mut proxy)?;
        proxy.overlay_registered = true;
        Base::register_static(&mut proxy)?;
        Ok(())
    }
}
/// A trait for types directly usable at the WASM FFI boundary without any conversion at all.
///
/// This trait is sealed and should not be implemented downstream.
#[cfg(all(feature = "std", feature = "wasmtime"))]
pub trait WasmTy: wasmtime::WasmTy + private::Sealed {}
impl WasmTy for i32 {}
impl WasmTy for u32 {}
impl WasmTy for i64 {}
impl WasmTy for u64 {}
/// Something that can be converted into a wasm compatible `Value`.
pub trait IntoValue {
    /// The type of the value in wasm.
    const VALUE_TYPE: ValueType;
    /// Convert `self` into a wasm `Value`.
    fn into_value(self) -> Value;
}
/// Something that can may be created from a wasm `Value`.
pub trait TryFromValue: Sized {
    /// Try to convert the given `Value` into `Self`.
    fn try_from_value(val: Value) -> Option<Self>;
}
impl IntoValue for u8 {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self as _)
    }
}
impl TryFromValue for u8 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for u16 {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self as _)
    }
}
impl TryFromValue for u16 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for u32 {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self as _)
    }
}
impl TryFromValue for u32 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for u64 {
    const VALUE_TYPE: ValueType = ValueType::I64;
    fn into_value(self) -> Value {
        Value::I64(self as _)
    }
}
impl TryFromValue for u64 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I64(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for i8 {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self as _)
    }
}
impl TryFromValue for i8 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for i16 {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self as _)
    }
}
impl TryFromValue for i16 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for i32 {
    const VALUE_TYPE: ValueType = ValueType::I32;
    fn into_value(self) -> Value {
        Value::I32(self as _)
    }
}
impl TryFromValue for i32 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I32(val) => Some(val as _),
            _ => None,
        }
    }
}
impl IntoValue for i64 {
    const VALUE_TYPE: ValueType = ValueType::I64;
    fn into_value(self) -> Value {
        Value::I64(self as _)
    }
}
impl TryFromValue for i64 {
    fn try_from_value(val: Value) -> Option<Self> {
        match val {
            Value::I64(val) => Some(val as _),
            _ => None,
        }
    }
}
/// Typed value that can be returned from a function.
///
/// Basically a `TypedValue` plus `Unit`, for functions which return nothing.
pub enum ReturnValue {
    /// For returning nothing.
    Unit,
    /// For returning some concrete value.
    Value(Value),
}
#[automatically_derived]
impl ::core::clone::Clone for ReturnValue {
    #[inline]
    fn clone(&self) -> ReturnValue {
        let _: ::core::clone::AssertParamIsClone<Value>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for ReturnValue {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ReturnValue {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ReturnValue {
    #[inline]
    fn eq(&self, other: &ReturnValue) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
            && match (self, other) {
                (ReturnValue::Value(__self_0), ReturnValue::Value(__arg1_0)) => {
                    *__self_0 == *__arg1_0
                }
                _ => true,
            }
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for ReturnValue {
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            match *self {
                ReturnValue::Unit => {
                    __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                }
                ReturnValue::Value(ref aa) => {
                    __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                }
                _ => (),
            }
        }
    }
    #[automatically_derived]
    impl ::codec::EncodeLike for ReturnValue {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for ReturnValue {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            match __codec_input_edqy.read_byte().map_err(|e| {
                e.chain("Could not decode `ReturnValue`, failed to read variant byte")
            })? {
                __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(ReturnValue::Unit)
                }
                __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(ReturnValue::Value({
                        let __codec_res_edqy =
                            <Value as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `ReturnValue::Value.0`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    }))
                }
                _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                    "Could not decode `ReturnValue`, variant doesn't exist",
                )),
            }
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for ReturnValue {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ReturnValue::Unit => ::core::fmt::Formatter::write_str(f, "Unit"),
            ReturnValue::Value(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Value", &__self_0)
            }
        }
    }
}
impl From<Value> for ReturnValue {
    fn from(v: Value) -> ReturnValue {
        ReturnValue::Value(v)
    }
}
impl ReturnValue {
    /// Maximum number of bytes `ReturnValue` might occupy when serialized with `SCALE`.
    ///
    /// Breakdown:
    ///  1 byte for encoding unit/value variant
    ///  1 byte for encoding value type
    ///  8 bytes for encoding the biggest value types available in wasm: f64, i64.
    pub const ENCODED_MAX_SIZE: usize = 10;
}