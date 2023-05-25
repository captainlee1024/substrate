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

use crate::Config;
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchInfo;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, One, SignedExtension},
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionValidity, TransactionValidityError,
		ValidTransaction,
	},
};
use sp_std::vec;

/// Nonce check and increment to give replay protection for transactions.
///
/// # Transaction Validity
///
/// This extension affects `requires` and `provides` tags of validity, but DOES NOT
/// set the `priority` field. Make sure that AT LEAST one of the signed extension sets
/// some kind of priority upon validating transactions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckNonce<T: Config>(#[codec(compact)] pub T::Nonce);

impl<T: Config> CheckNonce<T> {
	/// utility constructor. Used only in client/factory code.
	pub fn from(nonce: T::Nonce) -> Self {
		Self(nonce)
	}
}

impl<T: Config> sp_std::fmt::Debug for CheckNonce<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "CheckNonce({})", self.0)
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

// CheckNonce 实际上在执行SignedExtension的交易
// 这个交易允许你使用这些hook去执行一些操作在这笔交易上执行
// 并且会影响将要发生的一些事情(交易本身要做的事情)
impl<T: Config> SignedExtension for CheckNonce<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "CheckNonce";

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	// 这是在外部交易执行期间调用的
	// 所以需要一些额外的检查
	// 比如这里的逻辑删除的话
	// 块里交易的随机数都检查不出来了
	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> Result<(), TransactionValidityError> {
		let mut account = crate::Account::<T>::get(who);
		// 我们检查账户的state nonce
		// 如果和预期的account nonce from state 就返回错误提示
		if self.0 != account.nonce {
			return Err(if self.0 < account.nonce {
				// 过时 表示已经被包含或者已经有一个相同nonce的tx了
				InvalidTransaction::Stale
			} else {
				// 未来可能有效, 但是预期的下一个交易到当前交易之前应该有一些其他的交易
				InvalidTransaction::Future
			}
			.into())
		}
		account.nonce += T::Nonce::one();
		// 验证通过后, 在这里更新账户nonce
		crate::Account::<T>::insert(who, account);
		Ok(())
	}

	// txpool每次要确定一个交易是否有效都会调用它
	// validate 是被交易吃调用在一个交易进入到交易池的时候,并且可能会调用多次
	// pre_dispatch也会调用它
	fn validate(
		&self,
		who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		// check index
		// 校验是否过时
		let account = crate::Account::<T>::get(who);
		if self.0 < account.nonce {
			return InvalidTransaction::Stale.into()
		}

		//
		let provides = vec![Encode::encode(&(who, self.0))];
		// 对于账户模型的系统,我们需要包含一个交易, 就是前一个交易
		let requires = if account.nonce < self.0 {
			vec![Encode::encode(&(who, self.0 - One::one()))]
		} else {
			vec![]
		};

		// 返回这个ValidTransaction对象
		// 最终会吧所有的拓展验证集中到一起进行交易验证
		Ok(ValidTransaction {
			priority: 0,
			requires,
			provides,
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test, CALL};
	use frame_support::{assert_noop, assert_ok};

	#[test]
	fn signed_ext_check_nonce_works() {
		new_test_ext().execute_with(|| {
			crate::Account::<Test>::insert(
				1,
				crate::AccountInfo {
					nonce: 1,
					consumers: 0,
					providers: 0,
					sufficients: 0,
					data: 0,
				},
			);
			let info = DispatchInfo::default();
			let len = 0_usize;
			// stale
			assert_noop!(
				CheckNonce::<Test>(0).validate(&1, CALL, &info, len),
				InvalidTransaction::Stale
			);
			assert_noop!(
				CheckNonce::<Test>(0).pre_dispatch(&1, CALL, &info, len),
				InvalidTransaction::Stale
			);
			// correct
			assert_ok!(CheckNonce::<Test>(1).validate(&1, CALL, &info, len));
			assert_ok!(CheckNonce::<Test>(1).pre_dispatch(&1, CALL, &info, len));
			// future
			assert_ok!(CheckNonce::<Test>(5).validate(&1, CALL, &info, len));
			assert_noop!(
				CheckNonce::<Test>(5).pre_dispatch(&1, CALL, &info, len),
				InvalidTransaction::Future
			);
		})
	}
}
