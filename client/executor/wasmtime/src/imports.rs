// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{host::HostContext, runtime::StoreData};
use sc_executor_common::error::WasmError;
use sp_wasm_interface::{FunctionContext, HostFunctions};
use std::collections::HashMap;
use wasmtime::{ExternType, FuncType, ImportType, Linker, Module};

/// Goes over all imports of a module and prepares the given linker for instantiation of the module.
/// Returns an error if there are imports that cannot be satisfied.
/// 遍历runtime wasm module 的 import 段，并为模块的实例化准备给定的linker。如果存在无法满足的import，则返回错误。
pub(crate) fn prepare_imports<H>(
	linker: &mut Linker<StoreData>,
	module: &Module,
	allow_missing_func_imports: bool,
) -> Result<(), WasmError>
where
	H: HostFunctions,
{
	// 创建pending map 用于保存准备导入到runtime wasm module 的host wasm module
	let mut pending_func_imports = HashMap::new();
	// importType 包含3个部分
	// module import项所在的module的名字
	// name import项字段的名字
	// ty import项的类型
	// host function 的module名字是 env
	//
	// runtime 编译成wasm module 之后里面有import段
	// import段放了所有需要从外部import的 item 目前只有function类型
	// 这里我们把这些runtime 需要import的function name找出来
	// 然后再实例化
	for import_ty in module.imports() {
		let name = import_ty.name();

		// runtime import的host function 的module name 是 env
		if import_ty.module() != "env" {
			return Err(WasmError::Other(format!(
				"host doesn't provide any imports from non-env module: {}:{}",
				import_ty.module(),
				name,
			)))
		}

		// 目前可导入的类型只有function
		match import_ty.ty() {
			ExternType::Func(func_ty) => {
				// 如果是function insert到pending map
				pending_func_imports.insert(name.to_owned(), (import_ty, func_ty));
			},
			_ =>
				return Err(WasmError::Other(format!(
					"host doesn't provide any non function imports: {}:{}",
					import_ty.module(),
					name,
				))),
		};
	}

	// 构建Registry用来注册待导入的func
	// Registry: 一种特征，用于向 WASM 执行程序静态注册主机回调，以便以最小的开销从运行时调用它们。
	// 这在内部用于将基于 wasmtime 的执行程序与通过运行时接口宏生成的主机函数定义进行接口，而不是直接使用。
	let mut registry = Registry { linker, pending_func_imports };
	// 开始导入import
	// 在这里面进行注册的
	// 在这里面会调用io 展开之后生成的register_static函数
	// 展开的register_static会在里面调用传入的参数的register_static
	// 即 registry.register_static()
	// 然后把自己生成的runtime-interface的方法的实现依次注册进去
	// 这些runtime-interface的实现实际上是对标准Externalities和拓展的Extension的实现的封装
	// 而Externalities和它的拓展Extension以及runtime-interface的wrap实现都是client端的, 都是std实现的
	H::register_static(&mut registry)?;

	if !registry.pending_func_imports.is_empty() {
		if allow_missing_func_imports {
			for (name, (import_ty, func_ty)) in registry.pending_func_imports {
				let error = format!("call to a missing function {}:{}", import_ty.module(), name);
				log::debug!("Missing import: '{}' {:?}", name, func_ty);
				linker
					.func_new("env", &name, func_ty.clone(), move |_, _, _| {
					    Err(anyhow::Error::msg(error.clone()))
					})
					.expect("adding a missing import stub can only fail when the item already exists, and it is missing here; qed");
			}
		} else {
			let mut names = Vec::new();
			for (name, (import_ty, _)) in registry.pending_func_imports {
				names.push(format!("'{}:{}'", import_ty.module(), name));
			}
			let names = names.join(", ");
			return Err(WasmError::Other(format!(
				"runtime requires function imports which are not present on the host: {}",
				names
			)))
		}
	}

	Ok(())
}

struct Registry<'a, 'b> {
	linker: &'a mut Linker<StoreData>,
	pending_func_imports: HashMap<String, (ImportType<'b>, FuncType)>,
}

impl<'a, 'b> sp_wasm_interface::HostFunctionRegistry for Registry<'a, 'b> {
	type State = StoreData;
	type Error = WasmError;
	type FunctionContext = HostContext<'a>;

	fn with_function_context<R>(
		caller: wasmtime::Caller<Self::State>,
		callback: impl FnOnce(&mut dyn FunctionContext) -> R,
	) -> R {
		callback(&mut HostContext { caller })
	}

	fn register_static<Params, Results>(
		&mut self,
		fn_name: &str,
		func: impl wasmtime::IntoFunc<Self::State, Params, Results>,
	) -> Result<(), Self::Error> {
		// HostFunction 按名称依次导入
		// pending func imports 是从runtime wasm module 的 import 段找到的
		// HostFunction每次传入一个function 从pending中查找并移除
		// 理论上hostfunction提供的和runtime wasm module 需要的应该一样
		// 所以最终 pending func imports应该刚好被置空
		if self.pending_func_imports.remove(fn_name).is_some() {
			/*
				在此链接器中定义主机函数。
				有关主机函数如何操作的信息，请参见 Func::wrap。这包括有关将 Rust 类型转换为 WebAssembly 本机类型的信息。
				此方法在此链接器中以提供的名称创建主机提供的函数。此方法在创建 Store-独立函数的能力方面与众不同。这意味着此处定义的函数可用于实例化多个不同存储中的实例，或者换句话说，可以将该函数加载到不同的存储中。
				请注意，此处提到的功能也适用于所有其他 Linker 主机函数定义方法。所有这些都可用于在多个商店中创建实例 Func 。例如，在多线程程序中，这意味着如果不同的存储在不同的线程上执行，则可以并发调用主机函数。
				错误
				如果 和 module name 已标识与提供的项类型 item 相同的项，并且不允许阴影，则返回错误。有关详细信息，请参见 上的 Linker文档。
				例子
				let mut linker = Linker::new(&engine);
				linker.func_wrap("host", "double", |x: i32| x * 2)?;
				linker.func_wrap("host", "log_i32", |x: i32| println!("{}", x))?;
				linker.func_wrap("host", "log_str", |caller: Caller<'_, ()>, ptr: i32, len: i32| {
				    // ...
				})?;

				let wat = r#"
				    (module
				        (import "host" "double" (func (param i32) (result i32)))
				        (import "host" "log_i32" (func (param i32)))
				        (import "host" "log_str" (func (param i32 i32)))
				    )
				"#;
				let module = Module::new(&engine, wat)?;

				// instantiate in multiple different stores
				for _ in 0..10 {
				    let mut store = Store::new(&engine, ());
				    linker.instantiate(&mut store, &module)?;
				}
			 */
			self.linker.func_wrap("env", fn_name, func).map_err(|error| {
				WasmError::Other(format!(
					"failed to register host function '{}' with the WASM linker: {:#}",
					fn_name, error
				))
			})?;
		}

		Ok(())
	}
}
