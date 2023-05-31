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

use crate::{
	error::{Error, Result},
	wasm_runtime::{RuntimeCache, WasmExecutionMethod},
	RuntimeVersionOf,
};

use std::{
	marker::PhantomData,
	panic::{AssertUnwindSafe, UnwindSafe},
	path::PathBuf,
	sync::Arc,
};

use codec::Encode;
use sc_executor_common::{
	runtime_blob::RuntimeBlob,
	wasm_runtime::{
		AllocationStats, HeapAllocStrategy, WasmInstance, WasmModule, DEFAULT_HEAP_ALLOC_STRATEGY,
	},
};
use sp_core::traits::{CallContext, CodeExecutor, Externalities, RuntimeCode};
use sp_version::{GetNativeVersion, NativeVersion, RuntimeVersion};
use sp_wasm_interface::{ExtendedHostFunctions, HostFunctions};

/// Set up the externalities and safe calling environment to execute runtime calls.
///
/// If the inner closure panics, it will be caught and return an error.
/// 设置外部性和安全调用环境以执行运行时调用。
/// 如果内部闭包出现恐慌，它将被捕获并返回错误
pub fn with_externalities_safe<F, U>(ext: &mut dyn Externalities, f: F) -> Result<U>
where
	F: UnwindSafe + FnOnce() -> U,
{
	sp_externalities::set_and_run_with_externalities(ext, move || {
		// Substrate uses custom panic hook that terminates process on panic. Disable
		// termination for the native call.
		let _guard = sp_panic_handler::AbortGuard::force_unwind();
		std::panic::catch_unwind(f).map_err(|e| {
			if let Some(err) = e.downcast_ref::<String>() {
				Error::RuntimePanicked(err.clone())
			} else if let Some(err) = e.downcast_ref::<&'static str>() {
				Error::RuntimePanicked(err.to_string())
			} else {
				Error::RuntimePanicked("Unknown panic".into())
			}
		})
	})
}

/// Delegate for dispatching a CodeExecutor call.
///
/// By dispatching we mean that we execute a runtime function specified by it's name.
/// 用于调度代码执行器调用的委托。调度意味着我们执行由其名称指定的运行时函数。
/// 另一个是直接调用
pub trait NativeExecutionDispatch: Send + Sync {
	/// Host functions for custom runtime interfaces that should be callable from within the runtime
	/// besides the default Substrate runtime interfaces.
	type ExtendHostFunctions: HostFunctions;

	/// Dispatch a method in the runtime.
	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>>;

	/// Provide native runtime version.
	fn native_version() -> NativeVersion;
}

fn unwrap_heap_pages(pages: Option<HeapAllocStrategy>) -> HeapAllocStrategy {
	pages.unwrap_or_else(|| DEFAULT_HEAP_ALLOC_STRATEGY)
}

/// Builder for creating a [`WasmExecutor`] instance.
/// 用于创建 WasmExecutor 实例的builder
pub struct WasmExecutorBuilder<H> {
	_phantom: PhantomData<H>,
	method: WasmExecutionMethod,
	onchain_heap_alloc_strategy: Option<HeapAllocStrategy>,
	offchain_heap_alloc_strategy: Option<HeapAllocStrategy>,
	max_runtime_instances: usize,
	cache_path: Option<PathBuf>,
	allow_missing_host_functions: bool,
	runtime_cache_size: u8,
}

impl<H> WasmExecutorBuilder<H> {
	/// Create a new instance of `Self`
	///
	/// - `method`: The wasm execution method that should be used by the executor.
	pub fn new() -> Self {
		Self {
			_phantom: PhantomData,
			method: WasmExecutionMethod::default(),
			onchain_heap_alloc_strategy: None,
			offchain_heap_alloc_strategy: None,
			max_runtime_instances: 2,
			runtime_cache_size: 4,
			allow_missing_host_functions: false,
			cache_path: None,
		}
	}

	/// Create the wasm executor with execution method that should be used by the executor.
	pub fn with_execution_method(mut self, method: WasmExecutionMethod) -> Self {
		self.method = method;
		self
	}

	/// Create the wasm executor with the given number of `heap_alloc_strategy` for onchain runtime
	/// calls.
	/// 为链上运行时调用创建具有给定数量的“heap_alloc_strategy”的wasm 执行器。
	pub fn with_onchain_heap_alloc_strategy(
		mut self,
		heap_alloc_strategy: HeapAllocStrategy,
	) -> Self {
		self.onchain_heap_alloc_strategy = Some(heap_alloc_strategy);
		self
	}

	/// Create the wasm executor with the given number of `heap_alloc_strategy` for offchain runtime
	/// calls.
	/// 为链下运行时调用创建具有给定数量的 heap_alloc_strategy wasm 执行器。
	pub fn with_offchain_heap_alloc_strategy(
		mut self,
		heap_alloc_strategy: HeapAllocStrategy,
	) -> Self {
		self.offchain_heap_alloc_strategy = Some(heap_alloc_strategy);
		self
	}

	/// Create the wasm executor with the given maximum number of `instances`.
	///
	/// The number of `instances` defines how many different instances of a runtime the cache is
	/// storing.
	///
	/// By default the maximum number of `instances` is `2`.
	/// 创建具有给定最大“实例”数的 wasm 执行器。“实例”的数量定义了缓存存储的运行时的不同实例数。默认情况下，“实例”的最大数量为“2”。
	pub fn with_max_runtime_instances(mut self, instances: usize) -> Self {
		self.max_runtime_instances = instances;
		self
	}

	/// Create the wasm executor with the given `cache_path`.
	///
	/// The `cache_path` is A path to a directory where the executor can place its files for
	/// purposes of caching. This may be important in cases when there are many different modules
	/// with the compiled execution method is used.
	///
	/// By default there is no `cache_path` given.
	/// 使用给定 cache_path的 .
	/// is cache_path 指向目录的路径，执行程序可以在其中放置其文件以进行缓存。
	/// 在使用编译执行方法的许多不同的模块的情况下，这可能很重要。
	/// 默认情况下没有 cache_path 给定
	pub fn with_cache_path(mut self, cache_path: impl Into<PathBuf>) -> Self {
		self.cache_path = Some(cache_path.into());
		self
	}

	/// Create the wasm executor and allow/forbid missing host functions.
	///
	/// If missing host functions are forbidden, the instantiation of a wasm blob will fail
	/// for imported host functions that the executor is not aware of. If they are allowed,
	/// a stub is generated that will return an error when being called while executing the wasm.
	///
	/// By default missing host functions are forbidden.
	/// 创建 wasm 执行器并允许/禁止缺少主机函数。
	/// 如果禁止缺少主机函数，则对于执行程序不知道的导入主机函数，wasm blob 的实例化将失败。
	/// 如果允许，则会生成一个存根，该存根将在执行 wasm 时调用时返回错误。
	/// 默认情况下，禁止缺少主机函数
	pub fn with_allow_missing_host_functions(mut self, allow: bool) -> Self {
		self.allow_missing_host_functions = allow;
		self
	}

	/// Create the wasm executor with the given `runtime_cache_size`.
	///
	/// Defines the number of different runtimes/instantiated wasm blobs the cache stores.
	/// Runtimes/wasm blobs are differentiated based on the hash and the number of heap pages.
	///
	/// By default this value is set to `4`.
	/// 使用给定 runtime_cache_size的 .
	/// 定义缓存存储的不同运行时/实例化 wasm blob 的数量。运行时/wasm blob 根据哈希和堆页数进行区分。
	/// 默认情况下， 4此值设置为 。
	pub fn with_runtime_cache_size(mut self, runtime_cache_size: u8) -> Self {
		self.runtime_cache_size = runtime_cache_size;
		self
	}

	/// Build the configured [`WasmExecutor`].
	pub fn build(self) -> WasmExecutor<H> {
		WasmExecutor {
			method: self.method,
			default_offchain_heap_alloc_strategy: unwrap_heap_pages(
				self.offchain_heap_alloc_strategy,
			),
			default_onchain_heap_alloc_strategy: unwrap_heap_pages(
				self.onchain_heap_alloc_strategy,
			),
			cache: Arc::new(RuntimeCache::new(
				self.max_runtime_instances,
				self.cache_path.clone(),
				self.runtime_cache_size,
			)),
			cache_path: self.cache_path,
			allow_missing_host_functions: self.allow_missing_host_functions,
			phantom: PhantomData,
		}
	}
}

/// An abstraction over Wasm code executor. Supports selecting execution backend and
/// manages runtime cache.
/// 对 Wasm 代码执行器的抽象。支持选择执行后端并管理运行时缓存
pub struct WasmExecutor<H> {
	/// Method used to execute fallback Wasm code.
	/// 用于执行回退 Wasm 代码的方法。
	method: WasmExecutionMethod,
	/// The heap allocation strategy for onchain Wasm calls.
	/// 链上 Wasm 调用的堆分配策略。
	default_onchain_heap_alloc_strategy: HeapAllocStrategy,
	/// The heap allocation strategy for offchain Wasm calls.
	/// 链下 Wasm 调用的堆分配策略。
	default_offchain_heap_alloc_strategy: HeapAllocStrategy,
	/// WASM runtime cache.
	cache: Arc<RuntimeCache>,
	/// The path to a directory which the executor can leverage for a file cache, e.g. put there
	/// compiled artifacts.
	/// 执行程序可用于文件缓存的目录路径，例如放置已编译的工件。
	cache_path: Option<PathBuf>,
	/// Ignore missing function imports.
	/// 是否忽略未知的imports function
	allow_missing_host_functions: bool,
	/// PhantomData for HostFunction
	phantom: PhantomData<H>,
}

impl<H> Clone for WasmExecutor<H> {
	fn clone(&self) -> Self {
		Self {
			method: self.method,
			default_onchain_heap_alloc_strategy: self.default_onchain_heap_alloc_strategy,
			default_offchain_heap_alloc_strategy: self.default_offchain_heap_alloc_strategy,
			cache: self.cache.clone(),
			cache_path: self.cache_path.clone(),
			allow_missing_host_functions: self.allow_missing_host_functions,
			phantom: self.phantom,
		}
	}
}

impl<H> WasmExecutor<H>
where
	H: HostFunctions,
{
	/// Create new instance.
	///
	/// # Parameters
	///
	/// `method` - Method used to execute Wasm code.
	///
	/// `default_heap_pages` - Number of 64KB pages to allocate for Wasm execution. Internally this
	/// will be mapped as [`HeapAllocStrategy::Static`] where `default_heap_pages` represent the
	/// static number of heap pages to allocate. Defaults to `DEFAULT_HEAP_ALLOC_STRATEGY` if `None`
	/// is provided.
	///
	/// `max_runtime_instances` - The number of runtime instances to keep in memory ready for reuse.
	///
	/// `cache_path` - A path to a directory where the executor can place its files for purposes of
	///   caching. This may be important in cases when there are many different modules with the
	///   compiled execution method is used.
	///
	/// `runtime_cache_size` - The capacity of runtime cache.
	/// 创建一个新实例
	/// method - 用于执行 Wasm 代码的方法。
	/// default_heap_pages - 为 Wasm 执行分配的 64KB 页数。
	/// 在内部，这将映射为 HeapAllocStrategy::Static 其中 default_heap_pages 表示要分配的堆页的静态数量。
	/// 默认为 DEFAULT_HEAP_ALLOC_STRATEGY if None 提供。
	/// max_runtime_instances - 要保留在内存中以供重用的运行时实例数。
	/// cache_path - 指向目录的路径，执行程序可以在其中放置其文件以进行缓存。在使用编译执行方法的许多不同的模块的情况下，这可能很重要。
	/// runtime_cache_size - 运行时缓存的容量
	#[deprecated(note = "use `Self::builder` method instead of it")]
	pub fn new(
		method: WasmExecutionMethod,
		default_heap_pages: Option<u64>,
		max_runtime_instances: usize,
		cache_path: Option<PathBuf>,
		runtime_cache_size: u8,
	) -> Self {
		WasmExecutor {
			method,
			default_onchain_heap_alloc_strategy: unwrap_heap_pages(
				default_heap_pages.map(|h| HeapAllocStrategy::Static { extra_pages: h as _ }),
			),
			default_offchain_heap_alloc_strategy: unwrap_heap_pages(
				default_heap_pages.map(|h| HeapAllocStrategy::Static { extra_pages: h as _ }),
			),
			cache: Arc::new(RuntimeCache::new(
				max_runtime_instances,
				cache_path.clone(),
				runtime_cache_size,
			)),
			cache_path,
			allow_missing_host_functions: false,
			phantom: PhantomData,
		}
	}

	/// Instantiate a builder for creating an instance of `Self`.
	pub fn builder() -> WasmExecutorBuilder<H> {
		WasmExecutorBuilder::new()
	}

	/// Ignore missing function imports if set true.
	#[deprecated(note = "use `Self::builder` method instead of it")]
	pub fn allow_missing_host_functions(&mut self, allow_missing_host_functions: bool) {
		self.allow_missing_host_functions = allow_missing_host_functions
	}

	/// Execute the given closure `f` with the latest runtime (based on `runtime_code`).
	///
	/// The closure `f` is expected to return `Err(_)` when there happened a `panic!` in native code
	/// while executing the runtime in Wasm. If a `panic!` occurred, the runtime is invalidated to
	/// prevent any poisoned state. Native runtime execution does not need to report back
	/// any `panic!`.
	///
	/// # Safety
	///
	/// `runtime` and `ext` are given as `AssertUnwindSafe` to the closure. As described above, the
	/// runtime is invalidated on any `panic!` to prevent a poisoned state. `ext` is already
	/// implicitly handled as unwind safe, as we store it in a global variable while executing the
	/// native runtime.
	/// 使用最新的运行时（基于 runtime_code）执行给定的闭包f。
	/// 当在 Wasm 中执行运行时时发生本机代码时panic!，应返回Err(_)闭包f。如果发生，panic!
	/// 运行时将失效以防止任何中毒状态。本机运行时执行不需要报告任何 panic!.
	/// 安全
	/// runtime并ext给出了关于关闭。AssertUnwindSafe如上所述，运行时在任何运行时panic!都无效，以防止中毒状态。
	/// 已经隐式处理为解除安全，因为我们在执行本机运行时时将其存储在全局变量中。 ext
	pub fn with_instance<R, F>(
		&self,
		runtime_code: &RuntimeCode,
		ext: &mut dyn Externalities,
		heap_alloc_strategy: HeapAllocStrategy,
		f: F,
	) -> Result<R>
	where
		F: FnOnce(
			AssertUnwindSafe<&dyn WasmModule>,
			AssertUnwindSafe<&mut dyn WasmInstance>,
			Option<&RuntimeVersion>,
			AssertUnwindSafe<&mut dyn Externalities>,
		) -> Result<Result<R>>,
	{
		match self.cache.with_instance::<H, _, _>(
			runtime_code,
			ext,
			self.method,
			heap_alloc_strategy,
			self.allow_missing_host_functions,
			|module, instance, version, ext| {
				let module = AssertUnwindSafe(module);
				let instance = AssertUnwindSafe(instance);
				let ext = AssertUnwindSafe(ext);
				f(module, instance, version, ext)
			},
		)? {
			Ok(r) => r,
			Err(e) => Err(e),
		}
	}

	/// Perform a call into the given runtime.
	///
	/// The runtime is passed as a [`RuntimeBlob`]. The runtime will be instantiated with the
	/// parameters this `WasmExecutor` was initialized with.
	///
	/// In case of problems with during creation of the runtime or instantiation, a `Err` is
	/// returned. that describes the message.
	/// 调用runtime 编译成wams之后的内部函数
	/// 从这里追踪HostFunction是怎么import到runtime wasm module里的
	#[doc(hidden)] // We use this function for tests across multiple crates.
	pub fn uncached_call(
		&self,
		runtime_blob: RuntimeBlob,
		ext: &mut dyn Externalities,
		allow_missing_host_functions: bool,
		export_name: &str,
		call_data: &[u8],
	) -> std::result::Result<Vec<u8>, Error> {
		// 进入到该函数
		self.uncached_call_impl(
			runtime_blob,
			ext,
			allow_missing_host_functions,
			export_name,
			call_data,
			&mut None,
		)
	}

	/// Same as `uncached_call`, except it also returns allocation statistics.
	/// 与 相同 uncached_call，不同之处在于它还返回分配统计信息。
	#[doc(hidden)] // We use this function in tests.
	pub fn uncached_call_with_allocation_stats(
		&self,
		runtime_blob: RuntimeBlob,
		ext: &mut dyn Externalities,
		allow_missing_host_functions: bool,
		export_name: &str,
		call_data: &[u8],
	) -> (std::result::Result<Vec<u8>, Error>, Option<AllocationStats>) {
		let mut allocation_stats = None;
		let result = self.uncached_call_impl(
			runtime_blob,
			ext,
			allow_missing_host_functions,
			export_name,
			call_data,
			&mut allocation_stats,
		);
		(result, allocation_stats)
	}

	fn uncached_call_impl(
		&self,
		runtime_blob: RuntimeBlob,
		ext: &mut dyn Externalities,
		allow_missing_host_functions: bool,
		export_name: &str,
		call_data: &[u8],
		allocation_stats_out: &mut Option<AllocationStats>,
	) -> std::result::Result<Vec<u8>, Error> {
		// 这里把Runtime编译成wasm module, 并且把HostFunction module import到runtime wasm module了
		// 这里这个H是	phantom: PhantomData<H> 因为在wasmtime进行实例化runtiem wasm module的时候才用
		// 其他地方用不到, 所以使用phantomdata
		let module = crate::wasm_runtime::create_wasm_runtime_with_code::<H>(
			self.method,
			self.default_onchain_heap_alloc_strategy,
			runtime_blob,
			allow_missing_host_functions,
			self.cache_path.as_deref(),
		)
		.map_err(|e| format!("Failed to create module: {}", e))?;

		// 实例化
		let instance =
			module.new_instance().map_err(|e| format!("Failed to create instance: {}", e))?;

		// 断言成instance
		let mut instance = AssertUnwindSafe(instance);
		let mut ext = AssertUnwindSafe(ext);
		let mut allocation_stats_out = AssertUnwindSafe(allocation_stats_out);

		with_externalities_safe(&mut **ext, move || {
			let (result, allocation_stats) =
				// 在此 WASM 实例上调用方法。
				// 在执行之前，实例被重置。
				// 成功时返回编码结果
				instance.call_with_allocation_stats(export_name.into(), call_data);
			**allocation_stats_out = allocation_stats;
			result
		})
		.and_then(|r| r)
	}
}

/// 读取runtime_version, 调用上面的uncached_call 执行"Core_version"方法
impl<H> sp_core::traits::ReadRuntimeVersion for WasmExecutor<H>
where
	H: HostFunctions,
{
	fn read_runtime_version(
		&self,
		wasm_code: &[u8],
		ext: &mut dyn Externalities,
	) -> std::result::Result<Vec<u8>, String> {
		let runtime_blob = RuntimeBlob::uncompress_if_needed(wasm_code)
			.map_err(|e| format!("Failed to create runtime blob: {:?}", e))?;

		if let Some(version) = crate::wasm_runtime::read_embedded_version(&runtime_blob)
			.map_err(|e| format!("Failed to read the static section: {:?}", e))
			.map(|v| v.map(|v| v.encode()))?
		{
			return Ok(version)
		}

		// If the blob didn't have embedded runtime version section, we fallback to the legacy
		// way of fetching the version: i.e. instantiating the given instance and calling
		// `Core_version` on it.

		self.uncached_call(
			runtime_blob,
			ext,
			// If a runtime upgrade introduces new host functions that are not provided by
			// the node, we should not fail at instantiation. Otherwise nodes that are
			// updated could run this successfully and it could lead to a storage root
			// mismatch when importing this block.
			true,
			"Core_version",
			&[],
		)
		.map_err(|e| e.to_string())
	}
}

/// 内部调用with_instance去执行 call_export
impl<H> CodeExecutor for WasmExecutor<H>
where
	H: HostFunctions,
{
	type Error = Error;

	fn call(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &RuntimeCode,
		method: &str,
		data: &[u8],
		_use_native: bool,
		context: CallContext,
	) -> (Result<Vec<u8>>, bool) {
		tracing::trace!(
			target: "executor",
			%method,
			"Executing function",
		);

		let on_chain_heap_alloc_strategy = runtime_code
			.heap_pages
			.map(|h| HeapAllocStrategy::Static { extra_pages: h as _ })
			.unwrap_or_else(|| self.default_onchain_heap_alloc_strategy);

		let heap_alloc_strategy = match context {
			CallContext::Offchain => self.default_offchain_heap_alloc_strategy,
			CallContext::Onchain => on_chain_heap_alloc_strategy,
		};

		let result = self.with_instance(
			runtime_code,
			ext,
			heap_alloc_strategy,
			|_, mut instance, _onchain_version, mut ext| {
				with_externalities_safe(&mut **ext, move || instance.call_export(method, data))
			},
		);

		(result, false)
	}
}

/// 提取指定版本的runtime wasm code
impl<H> RuntimeVersionOf for WasmExecutor<H>
where
	H: HostFunctions,
{
	fn runtime_version(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &RuntimeCode,
	) -> Result<RuntimeVersion> {
		let on_chain_heap_pages = runtime_code
			.heap_pages
			.map(|h| HeapAllocStrategy::Static { extra_pages: h as _ })
			.unwrap_or_else(|| self.default_onchain_heap_alloc_strategy);

		self.with_instance(
			runtime_code,
			ext,
			on_chain_heap_pages,
			|_module, _instance, version, _ext| {
				Ok(version.cloned().ok_or_else(|| Error::ApiError("Unknown version".into())))
			},
		)
	}
}

/// A generic `CodeExecutor` implementation that uses a delegate to determine wasm code equivalence
/// and dispatch to native code when possible, falling back on `WasmExecutor` when not.
/// 一种通用 CodeExecutor 实现，它使用委托来确定 wasm 代码等效性，并在可能的情况下调度到本机代码，如果不是，则回 WasmExecutor 退。
pub struct NativeElseWasmExecutor<D: NativeExecutionDispatch> {
	/// Native runtime version info.
	native_version: NativeVersion,
	/// Fallback wasm executor.
	/// 会退到wasm的执行器
	wasm:
		WasmExecutor<ExtendedHostFunctions<sp_io::SubstrateHostFunctions, D::ExtendHostFunctions>>,
}

impl<D: NativeExecutionDispatch> NativeElseWasmExecutor<D> {
	///
	/// Create new instance.
	///
	/// # Parameters
	///
	/// `fallback_method` - Method used to execute fallback Wasm code.
	///
	/// `default_heap_pages` - Number of 64KB pages to allocate for Wasm execution. Internally this
	/// will be mapped as [`HeapAllocStrategy::Static`] where `default_heap_pages` represent the
	/// static number of heap pages to allocate. Defaults to `DEFAULT_HEAP_ALLOC_STRATEGY` if `None`
	/// is provided.
	///
	/// `max_runtime_instances` - The number of runtime instances to keep in memory ready for reuse.
	///
	/// `runtime_cache_size` - The capacity of runtime cache.
	/// fallback_method - 用于执行回退 Wasm 代码的方法。
	/// default_heap_pages - 为 Wasm 执行分配的 64KB 页数。在内部，这将映射为 HeapAllocStrategy::Static 其中 default_heap_pages 表示要分配的堆页的静态数量。默认为 DEFAULT_HEAP_ALLOC_STRATEGY if None 提供。
	/// max_runtime_instances - 要保留在内存中以供重用的运行时实例数。
	/// runtime_cache_size - 运行时缓存的容量。
	#[deprecated(note = "use `Self::new_with_wasm_executor` method instead of it")]
	pub fn new(
		fallback_method: WasmExecutionMethod,
		default_heap_pages: Option<u64>,
		max_runtime_instances: usize,
		runtime_cache_size: u8,
	) -> Self {
		let heap_pages = default_heap_pages.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| {
			HeapAllocStrategy::Static { extra_pages: h as _ }
		});
		let wasm = WasmExecutor::builder()
			.with_execution_method(fallback_method)
			.with_onchain_heap_alloc_strategy(heap_pages)
			.with_offchain_heap_alloc_strategy(heap_pages)
			.with_max_runtime_instances(max_runtime_instances)
			.with_runtime_cache_size(runtime_cache_size)
			.build();

		NativeElseWasmExecutor { native_version: D::native_version(), wasm }
	}

	/// Create a new instance using the given [`WasmExecutor`].
	/// 使用给定的 ['WasmExecutor'] (用于回退执行)创建一个新实例。
	pub fn new_with_wasm_executor(
		executor: WasmExecutor<
			ExtendedHostFunctions<sp_io::SubstrateHostFunctions, D::ExtendHostFunctions>,
		>,
	) -> Self {
		Self { native_version: D::native_version(), wasm: executor }
	}

	/// Ignore missing function imports if set true.
	#[deprecated(note = "use `Self::new_with_wasm_executor` method instead of it")]
	pub fn allow_missing_host_functions(&mut self, allow_missing_host_functions: bool) {
		self.wasm.allow_missing_host_functions = allow_missing_host_functions
	}
}

/// 根据指定版本获取runtime wasm code
impl<D: NativeExecutionDispatch> RuntimeVersionOf for NativeElseWasmExecutor<D> {
	fn runtime_version(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &RuntimeCode,
	) -> Result<RuntimeVersion> {
		self.wasm.runtime_version(ext, runtime_code)
	}
}

/// 获取native的版本号
impl<D: NativeExecutionDispatch> GetNativeVersion for NativeElseWasmExecutor<D> {
	fn native_version(&self) -> &NativeVersion {
		&self.native_version
	}
}

/// 代码执行引擎
/// 这里的实现不同于wasmExecutor, wasmExecutor是直接执行, 效率更高
/// 这里代理执行使用dispatch
impl<D: NativeExecutionDispatch + 'static> CodeExecutor for NativeElseWasmExecutor<D> {
	type Error = Error;

	fn call(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &RuntimeCode,
		method: &str,
		data: &[u8],
		use_native: bool,
		context: CallContext,
	) -> (Result<Vec<u8>>, bool) {
		tracing::trace!(
			target: "executor",
			function = %method,
			"Executing function",
		);

		let on_chain_heap_alloc_strategy = runtime_code
			.heap_pages
			.map(|h| HeapAllocStrategy::Static { extra_pages: h as _ })
			.unwrap_or_else(|| self.wasm.default_onchain_heap_alloc_strategy);

		let heap_alloc_strategy = match context {
			CallContext::Offchain => self.wasm.default_offchain_heap_alloc_strategy,
			CallContext::Onchain => on_chain_heap_alloc_strategy,
		};

		let mut used_native = false;
		let result = self.wasm.with_instance(
			runtime_code,
			ext,
			heap_alloc_strategy,
			|_, mut instance, onchain_version, mut ext| {
				let onchain_version =
					onchain_version.ok_or_else(|| Error::ApiError("Unknown version".into()))?;

				// 判断版本号是否和native version相同
				let can_call_with =
					onchain_version.can_call_with(&self.native_version.runtime_version);

				if use_native && can_call_with {
					tracing::trace!(
						target: "executor",
						native = %self.native_version.runtime_version,
						chain = %onchain_version,
						"Request for native execution succeeded",
					);

					used_native = true;
					// 这里使用native执行
					// 即使用dispatch
					// 这个dispatch 由项目的runtime lib.rs里的impl_runtime_apis!宏实现
					// 在Runtime结构体里有一个关联类型RuntimeCall, RuntimeCall是 Dispatchable
					// Dispatchable是primitive/runtime/trait.rs里面的trait
					// 它包含了origin, dispatchInfo等信息和一个dispatch方法,
					// 可以通过其“dispatch”方法执行的延迟调用（模块函数和参数值）。
					// 该trait 用于实际调用
					// impl_runtime_apis!宏会创建出RuntimeCall的具体类型Call
					// 是一个enum它包含了所有的construct_runtime的pallet
					// 然后Call实现了dispatch, dispatch会调度根据提供的信息找到目标pallet并调用其方法
					// getDispatchInfo, getCallMetadata， getCallName等实现的方式都类似
					/* Call 构造大致如下
						pub enum Call {
						    #[codec(index = 0u8)]
						    System(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            System,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 2u8)]
						    Timestamp(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            Timestamp,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 4u8)]
						    Grandpa(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            Grandpa,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 5u8)]
						    Balances(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            Balances,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 7u8)]
						    Sudo(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            Sudo,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 8u8)]
						    TemplateModule(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            TemplateModule,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 9u8)]
						    FactPallet(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            FactPallet,
						            Runtime,
						        >,
						    ),
						    #[codec(index = 10u8)]
						    UseStorage(
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<
						            UseStorage,
						            Runtime,
						        >,
						    ),
						}
					 */
					/* Call的调度大致如下
						impl self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::Dispatchable
						    for Call
						{
						    type Origin = Origin;
						    type Config = Call;
						    type Info =
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::weights::DispatchInfo;
						    type PostInfo =
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::weights::PostDispatchInfo;
						    fn dispatch (self , origin : Origin)
									-> self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::DispatchResultWithPostInfo{
						        if ! < Self::Origin as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait>::filter_call(&origin,&self) {
										return self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result::Err(frame_system::Error::<Runtime>::CallFiltered.into());
								}
						        self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(self,origin)
						    }
						}

						// 这里每个pallet又都由宏在各自内部生成一个Call, 并实现了UnfilteredDispatchable, 这里会调用每个pallet内部实现的dispatch_by_filter
						// 在每个pallet的实现里会match 各自pallet里提供的所有方法, 以此来完成对所有pallet提供的方法的调用
						impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable
						    for Call
						{
						    type Origin = Origin;    fn dispatch_bypass_filter (self , origin : Origin) -> self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: dispatch :: DispatchResultWithPostInfo{

						        match self {
						            Call :: System (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: Timestamp (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: Grandpa (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: Balances (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: Sudo (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: TemplateModule (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: FactPallet (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) ,
						            Call :: UseStorage (call) => self :: sp_api_hidden_includes_construct_runtime :: hidden_include :: traits :: UnfilteredDispatchable :: dispatch_bypass_filter (call , origin) , }
						    }
						}
					 */

					// 而每个pallet里宏也会生成一个Call enum, 也会实现一些方法, 大致如下
					/* Call 大致如下
						pub enum Call<T: Config> {
    					    #[doc(hidden)]
    					    #[codec(skip)]
    					    __Ignore(
    					        frame_support::sp_std::marker::PhantomData<(T,)>,
    					        frame_support::Never,
    					    ),
    					    #[codec(index = 0u8)]
    					    create_claim {
    					        #[allow(missing_docs)]
    					        id: u32,
    					        #[allow(missing_docs)]
    					        claim: u128,
    					    },
    					}
					 */
					/* 调度的逻辑大致如下, 我们的pallet就提供了一个方法create_claim, 所以match里就一个方法
						所有的方法都会在match里供调度.
						impl<T: Config> frame_support::traits::UnfilteredDispatchable for Call<T> {
    					    type Origin = frame_system::pallet_prelude::OriginFor<T>;
    					    fn dispatch_bypass_filter(
    					        self,
    					        origin: Self::Origin,
    					    ) -> frame_support::dispatch::DispatchResultWithPostInfo {
    					        match self {
    					            Self::create_claim { id, claim } => {
    					                let __within_span__ = {
    					                    use ::tracing::__macro_support::Callsite as _;
    					                    static CALLSITE: ::tracing::__macro_support::MacroCallsite = {
    					                        use ::tracing::__macro_support::MacroCallsite;
    					                        static META: ::tracing::Metadata<'static> = {
    					                            ::tracing_core::metadata::Metadata::new(
    					                                "create_claim",
    					                                "pallet_fact::pallet",
    					                                ::tracing::Level::TRACE,
    					                                Some("pallets/fact/src/lib_bac"),
    					                                Some(17u32),
    					                                Some("pallet_fact::pallet"),
    					                                ::tracing_core::field::FieldSet::new(
    					                                    &[],
    					                                    ::tracing_core::callsite::Identifier(&CALLSITE),
    					                                ),
    					                                ::tracing::metadata::Kind::SPAN,
    					                            )
    					                        };
    					                        MacroCallsite::new(&META)
    					                    };
    					                    let mut interest = ::tracing::subscriber::Interest::never();
    					                    if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
    					                        && ::tracing::Level::TRACE
    					                            <= ::tracing::level_filters::LevelFilter::current()
    					                        && {
    					                            interest = CALLSITE.interest();
    					                            !interest.is_never()
    					                        }
    					                        && CALLSITE.is_enabled(interest)
    					                    {
    					                        let meta = CALLSITE.metadata();
    					                        ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
    					                    } else {
    					                        let span = CALLSITE.disabled_span();
    					                        {};
    					                        span
    					                    }
    					                };
    					                let __tracing_guard__ = __within_span__.enter();
    					                // 最终在这里调用我们提供的函数create_claim
    					                <Pallet<T>>::create_claim(origin, id, claim)
    					                    .map(Into::into)
    					                    .map_err(Into::into)
    					            }
    					            Self::__Ignore(_, _) => {
    					                let _ = origin;
    					                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
    					                    &["internal error: entered unreachable code: "],
    					                    &[::core::fmt::ArgumentV1::new_display(
    					                        &::core::fmt::Arguments::new_v1(
    					                            &["__PhantomItem cannot be used."],
    					                            &[],
    					                        ),
    					                    )],
    					                ));
    					            }
    					        }
    					    }
    					}
					 */
					// 这个宏会dispatch所有的pallet
					Ok(with_externalities_safe(&mut **ext, move || D::dispatch(method, data))?
						.ok_or_else(|| Error::MethodNotFound(method.to_owned())))
				} else {
					// 如果不相同 则回退到使用wasmExecutor执行
					// 使用call_export执行
					if !can_call_with {
						tracing::trace!(
							target: "executor",
							native = %self.native_version.runtime_version,
							chain = %onchain_version,
							"Request for native execution failed",
						);
					}

					with_externalities_safe(&mut **ext, move || instance.call_export(method, data))
				}
			},
		);
		(result, used_native)
	}
}

impl<D: NativeExecutionDispatch> Clone for NativeElseWasmExecutor<D> {
	fn clone(&self) -> Self {
		NativeElseWasmExecutor { native_version: D::native_version(), wasm: self.wasm.clone() }
	}
}

/// 允许从二进制文件中读取版本信息
impl<D: NativeExecutionDispatch> sp_core::traits::ReadRuntimeVersion for NativeElseWasmExecutor<D> {
	fn read_runtime_version(
		&self,
		wasm_code: &[u8],
		ext: &mut dyn Externalities,
	) -> std::result::Result<Vec<u8>, String> {
		self.wasm.read_runtime_version(wasm_code, ext)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_runtime_interface::runtime_interface;

	#[runtime_interface]
	trait MyInterface {
		fn say_hello_world(data: &str) {
			println!("Hello world from: {}", data);
		}
	}

	pub struct MyExecutorDispatch;

	impl NativeExecutionDispatch for MyExecutorDispatch {
		type ExtendHostFunctions = (my_interface::HostFunctions, my_interface::HostFunctions);

		fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
			substrate_test_runtime::api::dispatch(method, data)
		}

		fn native_version() -> NativeVersion {
			substrate_test_runtime::native_version()
		}
	}

	#[test]
	fn native_executor_registers_custom_interface() {
		let executor = NativeElseWasmExecutor::<MyExecutorDispatch>::new_with_wasm_executor(
			WasmExecutor::builder().build(),
		);

		fn extract_host_functions<H>(
			_: &WasmExecutor<H>,
		) -> Vec<&'static dyn sp_wasm_interface::Function>
		where
			H: HostFunctions,
		{
			H::host_functions()
		}

		my_interface::HostFunctions::host_functions().iter().for_each(|function| {
			assert_eq!(
				extract_host_functions(&executor.wasm).iter().filter(|f| f == &function).count(),
				2
			);
		});

		my_interface::say_hello_world("hey");
	}
}
