// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

//! Substrate externalities abstraction
//!
//! The externalities mainly provide access to storage and to registered extensions. Extensions
//! are for example the keystore or the offchain externalities. These externalities are used to
//! access the node from the runtime via the runtime interfaces.
//!
//! This crate exposes the main [`Externalities`] trait.
//!
//! substrate externalities 抽象
//! 外部性主要提供对存储和注册扩展的访问。例如，扩展是密钥库或链下外部性。这些外部性用于通过运行时接口从运行时访问节点。
//! 这个板条箱暴露了主要 Externalities 特征

use sp_std::{
	any::{Any, TypeId},
	boxed::Box,
	vec::Vec,
};

use sp_storage::{ChildInfo, StateVersion, TrackedStorageKey};

pub use extensions::{Extension, ExtensionStore, Extensions};
pub use scope_limited::{set_and_run_with_externalities, with_externalities};

mod extensions;
mod scope_limited;

/// Externalities error.
#[derive(Debug)]
pub enum Error {
	/// Same extension cannot be registered twice.
	ExtensionAlreadyRegistered,
	/// Extensions are not supported.
	ExtensionsAreNotSupported,
	/// Extension `TypeId` is not registered.
	ExtensionIsNotRegistered(TypeId),
	/// Failed to update storage,
	StorageUpdateFailed(&'static str),
}

/// Results concerning an operation to remove many keys.
#[derive(codec::Encode, codec::Decode)]
#[must_use]
pub struct MultiRemovalResults {
	/// A continuation cursor which, if `Some` must be provided to the subsequent removal call.
	/// If `None` then all removals are complete and no further calls are needed.
	pub maybe_cursor: Option<Vec<u8>>,
	/// The number of items removed from the backend database.
	pub backend: u32,
	/// The number of unique keys removed, taking into account both the backend and the overlay.
	pub unique: u32,
	/// The number of iterations (each requiring a storage seek/read) which were done.
	pub loops: u32,
}

impl MultiRemovalResults {
	/// Deconstruct into the internal components.
	///
	/// Returns `(maybe_cursor, backend, unique, loops)`.
	pub fn deconstruct(self) -> (Option<Vec<u8>>, u32, u32, u32) {
		(self.maybe_cursor, self.backend, self.unique, self.loops)
	}
}

/// The Substrate externalities.
///
/// Provides access to the storage and to other registered extensions.
/// 每一个注册的 externalities 模块都叫 Extension
/// 默认提供了 Store 的 Extension
/// 用来存储注册的其他 Extension
pub trait Externalities: ExtensionStore {
	/// Write a key value pair to the offchain storage database.
	/// 将键值对写入链下存储数据库。
	fn set_offchain_storage(&mut self, key: &[u8], value: Option<&[u8]>);

	/// Read runtime storage.
	/// 专门给runtime使用的
	/// 读取Runtime的storage
	fn storage(&self, key: &[u8]) -> Option<Vec<u8>>;

	/// Get storage value hash.
	///
	/// This may be optimized for large values.
	/// 获取存储值哈希。这可以针对大值进行优化。
	fn storage_hash(&self, key: &[u8]) -> Option<Vec<u8>>;

	/// Get child storage value hash.
	///
	/// This may be optimized for large values.
	///
	/// Returns an `Option` that holds the SCALE encoded hash.
	/// 获取子存储值哈希。这可以针对大值进行优化。返回保存 SCALE 编码哈希的“选项”
	fn child_storage_hash(&self, child_info: &ChildInfo, key: &[u8]) -> Option<Vec<u8>>;

	/// Read child runtime storage.
	///
	/// Returns an `Option` that holds the SCALE encoded hash.
	///
	/// 读取子运行时存储。返回保存 SCALE 编码哈希的“选项”。
	fn child_storage(&self, child_info: &ChildInfo, key: &[u8]) -> Option<Vec<u8>>;

	/// Set storage entry `key` of current contract being called (effective immediately).
	/// 设置正在调用的当前合约的存储条目“key”（立即生效）。
	fn set_storage(&mut self, key: Vec<u8>, value: Vec<u8>) {
		self.place_storage(key, Some(value));
	}

	/// Set child storage entry `key` of current contract being called (effective immediately).
	/// 设置正在调用的当前合约的子存储条目“key”（立即生效）。
	fn set_child_storage(&mut self, child_info: &ChildInfo, key: Vec<u8>, value: Vec<u8>) {
		self.place_child_storage(child_info, key, Some(value))
	}

	/// Clear a storage entry (`key`) of current contract being called (effective immediately).
	/// 清除正在调用的当前合约的存储条目（“key”）（立即生效）。
	fn clear_storage(&mut self, key: &[u8]) {
		self.place_storage(key.to_vec(), None);
	}

	/// Clear a child storage entry (`key`) of current contract being called (effective
	/// immediately).
	fn clear_child_storage(&mut self, child_info: &ChildInfo, key: &[u8]) {
		self.place_child_storage(child_info, key.to_vec(), None)
	}

	/// Whether a storage entry exists.
	/// 查询一个key是否存在
	fn exists_storage(&self, key: &[u8]) -> bool {
		self.storage(key).is_some()
	}

	/// Whether a child storage entry exists.
	/// 查询一个key是否在子存储里
	fn exists_child_storage(&self, child_info: &ChildInfo, key: &[u8]) -> bool {
		self.child_storage(child_info, key).is_some()
	}

	/// Returns the key immediately following the given key, if it exists.
	/// 返回紧跟在给定键之后的键（如果存在）。
	/// 类似迭代器
	fn next_storage_key(&self, key: &[u8]) -> Option<Vec<u8>>;

	/// Returns the key immediately following the given key, if it exists, in child storage.
	/// 类似子存储的迭代器
	fn next_child_storage_key(&self, child_info: &ChildInfo, key: &[u8]) -> Option<Vec<u8>>;

	/// Clear an entire child storage.
	///
	/// Deletes all keys from the overlay and up to `maybe_limit` keys from the backend. No
	/// limit is applied if `maybe_limit` is `None`. Returns the cursor for the next call as `Some`
	/// if the child trie deletion operation is incomplete. In this case, it should be passed into
	/// the next call to avoid unaccounted iterations on the backend. Returns also the the number
	/// of keys that were removed from the backend, the number of unique keys removed in total
	/// (including from the overlay) and the number of backend iterations done.
	///
	/// As long as `maybe_cursor` is passed from the result of the previous call, then the number of
	/// iterations done will only ever be one more than the number of keys removed.
	///
	/// # Note
	///
	/// An implementation is free to delete more keys than the specified limit as long as
	/// it is able to do that in constant time.
	fn kill_child_storage(
		&mut self,
		child_info: &ChildInfo,
		maybe_limit: Option<u32>,
		maybe_cursor: Option<&[u8]>,
	) -> MultiRemovalResults;

	/// Clear storage entries which keys are start with the given prefix.
	///
	/// `maybe_limit`, `maybe_cursor` and result works as for `kill_child_storage`.
	/// 根据前缀匹配进行删除,
	fn clear_prefix(
		&mut self,
		prefix: &[u8],
		maybe_limit: Option<u32>,
		maybe_cursor: Option<&[u8]>,
	) -> MultiRemovalResults;

	/// Clear child storage entries which keys are start with the given prefix.
	///
	/// `maybe_limit`, `maybe_cursor` and result works as for `kill_child_storage`.
	fn clear_child_prefix(
		&mut self,
		child_info: &ChildInfo,
		prefix: &[u8],
		maybe_limit: Option<u32>,
		maybe_cursor: Option<&[u8]>,
	) -> MultiRemovalResults;

	/// Set or clear a storage entry (`key`) of current contract being called (effective
	/// immediately).
	/// 设置或删除正在调用的当前合约的存储条目（“key”）（立即生效）。
	fn place_storage(&mut self, key: Vec<u8>, value: Option<Vec<u8>>);

	/// Set or clear a child storage entry.
	fn place_child_storage(&mut self, child_info: &ChildInfo, key: Vec<u8>, value: Option<Vec<u8>>);

	/// Get the trie root of the current storage map.
	///
	/// This will also update all child storage keys in the top-level storage map.
	///
	/// The returned hash is defined by the `Block` and is SCALE encoded.
	///
	/// 获取当前存储映射的 trie 根。
	/// 这还将更新顶级存储映射中的所有子存储key。
	/// 返回的哈希由 Block 和 SCALE 编码定义
	fn storage_root(&mut self, state_version: StateVersion) -> Vec<u8>;

	/// Get the trie root of a child storage map.
	///
	/// This will also update the value of the child storage keys in the top-level storage map.
	///
	/// If the storage root equals the default hash as defined by the trie, the key in the top-level
	/// storage map will be removed.
	fn child_storage_root(
		&mut self,
		child_info: &ChildInfo,
		state_version: StateVersion,
	) -> Vec<u8>;

	/// Append storage item.
	///
	/// This assumes specific format of the storage item. Also there is no way to undo this
	/// operation.
	fn storage_append(&mut self, key: Vec<u8>, value: Vec<u8>);

	/// Start a new nested transaction.
	///
	/// This allows to either commit or roll back all changes made after this call to the
	/// top changes or the default child changes. For every transaction there cam be a
	/// matching call to either `storage_rollback_transaction` or `storage_commit_transaction`.
	/// Any transactions that are still open after returning from runtime are committed
	/// automatically.
	///
	/// Changes made without any open transaction are committed immediately.
	///
	/// 启动新的嵌套事务。这允许提交或回滚在此调用后所做的所有更改到顶部更改或默认子更改。
	/// 对于每个事务，都有一个匹配的调用“storage_rollback_transaction”
	/// 或“storage_commit_transaction”。
	/// 从Runtime返回后仍处于打开状态的任何事务都将自动提交。
	/// 在没有任何打开事务的情况下所做的更改将立即提交。
	fn storage_start_transaction(&mut self);

	/// Rollback the last transaction started by `storage_start_transaction`.
	///
	/// Any changes made during that storage transaction are discarded. Returns an error when
	/// no transaction is open that can be closed.
	///
	/// 回滚由“storage_start_transaction”启动的最后一个事务。
	/// 在该存储事务期间所做的任何更改都将被丢弃。当没有打开可以关闭的事务时返回错误
	fn storage_rollback_transaction(&mut self) -> Result<(), ()>;

	/// Commit the last transaction started by `storage_start_transaction`.
	///
	/// Any changes made during that storage transaction are committed. Returns an error when
	/// no transaction is open that can be closed.
	///
	/// 提交由“storage_start_transaction”启动的最后一个事务。
	/// 将提交在该存储事务期间所做的任何更改。当没有打开可以关闭的事务时返回错误。
	fn storage_commit_transaction(&mut self) -> Result<(), ()>;

	/// Index specified transaction slice and store it.
	/// 索引指定的事务切片并存储它。
	fn storage_index_transaction(&mut self, _index: u32, _hash: &[u8], _size: u32) {
		unimplemented!("storage_index_transaction");
	}

	/// Renew existing piece of transaction storage.
	/// 续订现有的事务存储。
	fn storage_renew_transaction_index(&mut self, _index: u32, _hash: &[u8]) {
		unimplemented!("storage_renew_transaction_index");
	}

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Wipes all changes from caches and the database.
	///
	/// The state will be reset to genesis.
	fn wipe(&mut self);

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Commits all changes to the database and clears all caches.
	fn commit(&mut self);

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Gets the current read/write count for the benchmarking process.
	fn read_write_count(&self) -> (u32, u32, u32, u32);

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Resets read/write count for the benchmarking process.
	fn reset_read_write_count(&mut self);

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Gets the current DB tracking whitelist.
	fn get_whitelist(&self) -> Vec<TrackedStorageKey>;

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Adds new storage keys to the DB tracking whitelist.
	fn set_whitelist(&mut self, new: Vec<TrackedStorageKey>);

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Returns estimated proof size for the state queries so far.
	/// Proof is reset on commit and wipe.
	fn proof_size(&self) -> Option<u32> {
		None
	}

	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	/// Benchmarking related functionality and shouldn't be used anywhere else!
	/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
	///
	/// Get all the keys that have been read or written to during the benchmark.
	fn get_read_and_written_keys(&self) -> Vec<(Vec<u8>, u32, u32, bool)>;
}

/// Extension for the [`Externalities`] trait.
/// Externalities 的 Extension
pub trait ExternalitiesExt {
	/// Tries to find a registered extension and returns a mutable reference.
	/// 尝试查找已注册的扩展并返回可变引用。
	fn extension<T: Any + Extension>(&mut self) -> Option<&mut T>;

	/// Register extension `ext`.
	///
	/// Should return error if extension is already registered or extensions are not supported.
	/// 注册扩展 ext.
	/// 如果扩展已注册或不支持扩展，则应返回错误。
	fn register_extension<T: Extension>(&mut self, ext: T) -> Result<(), Error>;

	/// Deregister and drop extension of `T` type.
	///
	/// Should return error if extension of type `T` is not registered or
	/// extensions are not supported.
	/// 取消注册并删除类型的扩展 T 名。
	/// 如果未注册类型的 T 扩展名或不支持扩展名，则应返回错误。
	fn deregister_extension<T: Extension>(&mut self) -> Result<(), Error>;
}

/// 这些方法的实现实际上调用了ExtensionStore的能力
/// 因为trait约束了Externalities必须实现ExtensionStore的trait
/// 也就是这个trait提供了ExtensionStore的能力, 并且包装了一层做了类型转换等
/// 更方便使用
impl ExternalitiesExt for &mut dyn Externalities {
	fn extension<T: Any + Extension>(&mut self) -> Option<&mut T> {
		self.extension_by_type_id(TypeId::of::<T>()).and_then(<dyn Any>::downcast_mut)
	}

	fn register_extension<T: Extension>(&mut self, ext: T) -> Result<(), Error> {
		self.register_extension_with_type_id(TypeId::of::<T>(), Box::new(ext))
	}

	fn deregister_extension<T: Extension>(&mut self) -> Result<(), Error> {
		self.deregister_extension_by_type_id(TypeId::of::<T>())
	}
}
