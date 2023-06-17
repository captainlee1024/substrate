#![feature(prelude_import)]
//! Shareable Substrate types.
#![warn(missing_docs)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
#[doc(hidden)]
pub use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "serde")]
pub use serde;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use sp_runtime_interface::pass_by::{PassByEnum, PassByInner};
use sp_std::{ops::Deref, prelude::*};
pub use sp_debug_derive::RuntimeDebug;
#[cfg(feature = "serde")]
pub use impl_serde::serialize as bytes;
#[cfg(feature = "full_crypto")]
pub mod hashing {
    //! Hashing functions.
    //!
    //! This module is gated by `full-crypto` feature. If you intend to use any of the functions
    //! defined here within your runtime, you should most likely rather use `sp_io::hashing` instead,
    //! unless you know what you're doing. Using `sp_io` will be more performant, since instead of
    //! computing the hash in WASM it delegates that computation to the host client.
    pub use sp_core_hashing::*;
}
#[cfg(feature = "full_crypto")]
pub use hashing::{blake2_128, blake2_256, keccak_256, twox_128, twox_256, twox_64};
pub mod crypto {
    //! Cryptographic utilities.
    use crate::{ed25519, sr25519};
    #[cfg(feature = "std")]
    use bip39::{Language, Mnemonic, MnemonicType};
    use codec::{Decode, Encode, MaxEncodedLen};
    #[cfg(feature = "std")]
    use rand::{rngs::OsRng, RngCore};
    #[cfg(feature = "std")]
    use regex::Regex;
    use scale_info::TypeInfo;
    #[cfg(feature = "std")]
    pub use secrecy::{ExposeSecret, SecretString};
    use sp_runtime_interface::pass_by::PassByInner;
    #[doc(hidden)]
    pub use sp_std::ops::Deref;
    use sp_std::{hash::Hash, str, vec::Vec};
    pub use ss58_registry::{from_known_address_format, Ss58AddressFormat, Ss58AddressFormatRegistry};
    /// Trait to zeroize a memory buffer.
    pub use zeroize::Zeroize;
    /// The root phrase for our publicly known keys.
    pub const DEV_PHRASE: &str =
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
    /// The address of the associated root phrase for our publicly known keys.
    pub const DEV_ADDRESS: &str = "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV";
    /// The length of the junction identifier. Note that this is also referred to as the
    /// `CHAIN_CODE_LENGTH` in the context of Schnorrkel.
    pub const JUNCTION_ID_LEN: usize = 32;
    /// Similar to `From`, except that the onus is on the part of the caller to ensure
    /// that data passed in makes sense. Basically, you're not guaranteed to get anything
    /// sensible out.
    pub trait UncheckedFrom<T> {
        /// Convert from an instance of `T` to Self. This is not guaranteed to be
        /// whatever counts as a valid instance of `T` and it's up to the caller to
        /// ensure that it makes sense.
        fn unchecked_from(t: T) -> Self;
    }
    /// The counterpart to `UncheckedFrom`.
    pub trait UncheckedInto<T> {
        /// The counterpart to `unchecked_from`.
        fn unchecked_into(self) -> T;
    }
    impl<S, T: UncheckedFrom<S>> UncheckedInto<T> for S {
        fn unchecked_into(self) -> T {
            T::unchecked_from(self)
        }
    }
    /// An error with the interpretation of a secret.
    #[cfg(feature = "full_crypto")]
    pub enum SecretStringError {
        /// The overall format was invalid (e.g. the seed phrase contained symbols).
        #[error("Invalid format")]
        InvalidFormat,
        /// The seed phrase provided is not a valid BIP39 phrase.
        #[error("Invalid phrase")]
        InvalidPhrase,
        /// The supplied password was invalid.
        #[error("Invalid password")]
        InvalidPassword,
        /// The seed is invalid (bad content).
        #[error("Invalid seed")]
        InvalidSeed,
        /// The seed has an invalid length.
        #[error("Invalid seed length")]
        InvalidSeedLength,
        /// The derivation path was invalid (e.g. contains soft junctions when they are not supported).
        #[error("Invalid path")]
        InvalidPath,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for SecretStringError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                SecretStringError::InvalidFormat => {
                    ::core::fmt::Formatter::write_str(f, "InvalidFormat")
                }
                SecretStringError::InvalidPhrase => {
                    ::core::fmt::Formatter::write_str(f, "InvalidPhrase")
                }
                SecretStringError::InvalidPassword => {
                    ::core::fmt::Formatter::write_str(f, "InvalidPassword")
                }
                SecretStringError::InvalidSeed => {
                    ::core::fmt::Formatter::write_str(f, "InvalidSeed")
                }
                SecretStringError::InvalidSeedLength => {
                    ::core::fmt::Formatter::write_str(f, "InvalidSeedLength")
                }
                SecretStringError::InvalidPath => {
                    ::core::fmt::Formatter::write_str(f, "InvalidPath")
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for SecretStringError {
        #[inline]
        fn clone(&self) -> SecretStringError {
            match self {
                SecretStringError::InvalidFormat => SecretStringError::InvalidFormat,
                SecretStringError::InvalidPhrase => SecretStringError::InvalidPhrase,
                SecretStringError::InvalidPassword => SecretStringError::InvalidPassword,
                SecretStringError::InvalidSeed => SecretStringError::InvalidSeed,
                SecretStringError::InvalidSeedLength => SecretStringError::InvalidSeedLength,
                SecretStringError::InvalidPath => SecretStringError::InvalidPath,
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for SecretStringError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for SecretStringError {
        #[inline]
        fn eq(&self, other: &SecretStringError) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for SecretStringError {}
    #[automatically_derived]
    impl ::core::cmp::Eq for SecretStringError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for SecretStringError {}
    #[allow(unused_qualifications)]
    impl std::fmt::Display for SecretStringError {
        fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                SecretStringError::InvalidFormat {} => {
                    __formatter.write_fmt(::core::fmt::Arguments::new_v1(&["Invalid format"], &[]))
                }
                SecretStringError::InvalidPhrase {} => {
                    __formatter.write_fmt(::core::fmt::Arguments::new_v1(&["Invalid phrase"], &[]))
                }
                SecretStringError::InvalidPassword {} => __formatter
                    .write_fmt(::core::fmt::Arguments::new_v1(&["Invalid password"], &[])),
                SecretStringError::InvalidSeed {} => {
                    __formatter.write_fmt(::core::fmt::Arguments::new_v1(&["Invalid seed"], &[]))
                }
                SecretStringError::InvalidSeedLength {} => __formatter.write_fmt(
                    ::core::fmt::Arguments::new_v1(&["Invalid seed length"], &[]),
                ),
                SecretStringError::InvalidPath {} => {
                    __formatter.write_fmt(::core::fmt::Arguments::new_v1(&["Invalid path"], &[]))
                }
            }
        }
    }
    /// An error when deriving a key.
    #[cfg(feature = "full_crypto")]
    pub enum DeriveError {
        /// A soft key was found in the path (and is unsupported).
        #[error("Soft key in path")]
        SoftKeyInPath,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DeriveError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "SoftKeyInPath")
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for DeriveError {
        #[inline]
        fn clone(&self) -> DeriveError {
            DeriveError::SoftKeyInPath
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for DeriveError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for DeriveError {
        #[inline]
        fn eq(&self, other: &DeriveError) -> bool {
            true
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for DeriveError {}
    #[automatically_derived]
    impl ::core::cmp::Eq for DeriveError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for DeriveError {}
    #[allow(unused_qualifications)]
    impl std::fmt::Display for DeriveError {
        fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                DeriveError::SoftKeyInPath {} => __formatter
                    .write_fmt(::core::fmt::Arguments::new_v1(&["Soft key in path"], &[])),
            }
        }
    }
    /// A since derivation junction description. It is the single parameter used when creating
    /// a new secret key from an existing secret key and, in the case of `SoftRaw` and `SoftIndex`
    /// a new public key from an existing public key.
    #[cfg(any(feature = "full_crypto", feature = "serde"))]
    pub enum DeriveJunction {
        /// Soft (vanilla) derivation. Public keys have a correspondent derivation.
        Soft([u8; JUNCTION_ID_LEN]),
        /// Hard ("hardened") derivation. Public keys do not have a correspondent derivation.
        Hard([u8; JUNCTION_ID_LEN]),
    }
    #[automatically_derived]
    impl ::core::marker::Copy for DeriveJunction {}
    #[automatically_derived]
    impl ::core::clone::Clone for DeriveJunction {
        #[inline]
        fn clone(&self) -> DeriveJunction {
            let _: ::core::clone::AssertParamIsClone<[u8; JUNCTION_ID_LEN]>;
            let _: ::core::clone::AssertParamIsClone<[u8; JUNCTION_ID_LEN]>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for DeriveJunction {}
    #[automatically_derived]
    impl ::core::cmp::Eq for DeriveJunction {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; JUNCTION_ID_LEN]>;
            let _: ::core::cmp::AssertParamIsEq<[u8; JUNCTION_ID_LEN]>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for DeriveJunction {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for DeriveJunction {
        #[inline]
        fn eq(&self, other: &DeriveJunction) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (DeriveJunction::Soft(__self_0), DeriveJunction::Soft(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (DeriveJunction::Hard(__self_0), DeriveJunction::Hard(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => unsafe { ::core::intrinsics::unreachable() },
                }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for DeriveJunction {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state);
            match self {
                DeriveJunction::Soft(__self_0) => ::core::hash::Hash::hash(__self_0, state),
                DeriveJunction::Hard(__self_0) => ::core::hash::Hash::hash(__self_0, state),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DeriveJunction {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                DeriveJunction::Soft(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Soft", &__self_0)
                }
                DeriveJunction::Hard(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Hard", &__self_0)
                }
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for DeriveJunction {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    DeriveJunction::Soft(ref aa) => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    DeriveJunction::Hard(ref aa) => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for DeriveJunction {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for DeriveJunction {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy.read_byte().map_err(|e| {
                    e.chain("Could not decode `DeriveJunction`, failed to read variant byte")
                })? {
                    __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(DeriveJunction::Soft({
                            let __codec_res_edqy =
                                <[u8; JUNCTION_ID_LEN] as ::codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `DeriveJunction::Soft.0`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(DeriveJunction::Hard({
                            let __codec_res_edqy =
                                <[u8; JUNCTION_ID_LEN] as ::codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `DeriveJunction::Hard.0`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `DeriveJunction`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    #[cfg(any(feature = "full_crypto", feature = "serde"))]
    impl DeriveJunction {
        /// Consume self to return a soft derive junction with the same chain code.
        pub fn soften(self) -> Self {
            DeriveJunction::Soft(self.unwrap_inner())
        }
        /// Consume self to return a hard derive junction with the same chain code.
        pub fn harden(self) -> Self {
            DeriveJunction::Hard(self.unwrap_inner())
        }
        /// Create a new soft (vanilla) DeriveJunction from a given, encodable, value.
        ///
        /// If you need a hard junction, use `hard()`.
        pub fn soft<T: Encode>(index: T) -> Self {
            let mut cc: [u8; JUNCTION_ID_LEN] = Default::default();
            index.using_encoded(|data| {
                if data.len() > JUNCTION_ID_LEN {
                    cc.copy_from_slice(&sp_core_hashing::blake2_256(data));
                } else {
                    cc[0..data.len()].copy_from_slice(data);
                }
            });
            DeriveJunction::Soft(cc)
        }
        /// Create a new hard (hardened) DeriveJunction from a given, encodable, value.
        ///
        /// If you need a soft junction, use `soft()`.
        pub fn hard<T: Encode>(index: T) -> Self {
            Self::soft(index).harden()
        }
        /// Consume self to return the chain code.
        pub fn unwrap_inner(self) -> [u8; JUNCTION_ID_LEN] {
            match self {
                DeriveJunction::Hard(c) | DeriveJunction::Soft(c) => c,
            }
        }
        /// Get a reference to the inner junction id.
        pub fn inner(&self) -> &[u8; JUNCTION_ID_LEN] {
            match self {
                DeriveJunction::Hard(ref c) | DeriveJunction::Soft(ref c) => c,
            }
        }
        /// Return `true` if the junction is soft.
        pub fn is_soft(&self) -> bool {
            match *self {
                DeriveJunction::Soft(_) => true,
                _ => false,
            }
        }
        /// Return `true` if the junction is hard.
        pub fn is_hard(&self) -> bool {
            match *self {
                DeriveJunction::Hard(_) => true,
                _ => false,
            }
        }
    }
    #[cfg(any(feature = "full_crypto", feature = "serde"))]
    impl<T: AsRef<str>> From<T> for DeriveJunction {
        fn from(j: T) -> DeriveJunction {
            let j = j.as_ref();
            let (code, hard) = if let Some(stripped) = j.strip_prefix('/') {
                (stripped, true)
            } else {
                (j, false)
            };
            let res = if let Ok(n) = str::parse::<u64>(code) {
                DeriveJunction::soft(n)
            } else {
                DeriveJunction::soft(code)
            };
            if hard {
                res.harden()
            } else {
                res
            }
        }
    }
    /// An error type for SS58 decoding.
    #[allow(missing_docs)]
    #[cfg(any(feature = "full_crypto", feature = "serde"))]
    pub enum PublicError {
        #[error("Base 58 requirement is violated")]
        BadBase58,
        #[error("Length is bad")]
        BadLength,
        #[error(
            "Unknown SS58 address format `{}`. ` \
		`To support this address format, you need to call `set_default_ss58_version` at node start up.",
            _0
        )]
        UnknownSs58AddressFormat(Ss58AddressFormat),
        #[error("Invalid checksum")]
        InvalidChecksum,
        #[error("Invalid SS58 prefix byte.")]
        InvalidPrefix,
        #[error("Invalid SS58 format.")]
        InvalidFormat,
        #[error("Invalid derivation path.")]
        InvalidPath,
        #[error("Disallowed SS58 Address Format for this datatype.")]
        FormatNotAllowed,
    }
    #[automatically_derived]
    #[allow(missing_docs)]
    impl ::core::clone::Clone for PublicError {
        #[inline]
        fn clone(&self) -> PublicError {
            let _: ::core::clone::AssertParamIsClone<Ss58AddressFormat>;
            *self
        }
    }
    #[automatically_derived]
    #[allow(missing_docs)]
    impl ::core::marker::Copy for PublicError {}
    #[allow(missing_docs)]
    #[automatically_derived]
    impl ::core::marker::StructuralEq for PublicError {}
    #[automatically_derived]
    #[allow(missing_docs)]
    impl ::core::cmp::Eq for PublicError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Ss58AddressFormat>;
        }
    }
    #[allow(missing_docs)]
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PublicError {}
    #[automatically_derived]
    #[allow(missing_docs)]
    impl ::core::cmp::PartialEq for PublicError {
        #[inline]
        fn eq(&self, other: &PublicError) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        PublicError::UnknownSs58AddressFormat(__self_0),
                        PublicError::UnknownSs58AddressFormat(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    _ => true,
                }
        }
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for PublicError {}
    #[allow(unused_qualifications)]
    impl std::fmt::Display for PublicError {
        fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            # [allow (unused_variables , deprecated , clippy :: used_underscore_binding)] match self { PublicError :: BadBase58 { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Base 58 requirement is violated"] , & [])) , PublicError :: BadLength { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Length is bad"] , & [])) , PublicError :: UnknownSs58AddressFormat (_0) => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Unknown SS58 address format `" , "`. ` `To support this address format, you need to call `set_default_ss58_version` at node start up."] , & [:: core :: fmt :: ArgumentV1 :: new_display (& _0)])) , PublicError :: InvalidChecksum { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Invalid checksum"] , & [])) , PublicError :: InvalidPrefix { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Invalid SS58 prefix byte."] , & [])) , PublicError :: InvalidFormat { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Invalid SS58 format."] , & [])) , PublicError :: InvalidPath { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Invalid derivation path."] , & [])) , PublicError :: FormatNotAllowed { } => __formatter . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["Disallowed SS58 Address Format for this datatype."] , & [])) , }
        }
    }
    #[cfg(feature = "std")]
    impl sp_std::fmt::Debug for PublicError {
        fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(&self)],
            ))
        }
    }
    /// Key that can be encoded to/from SS58.
    ///
    /// See <https://docs.substrate.io/v3/advanced/ss58/>
    /// for information on the codec.
    pub trait Ss58Codec: Sized + AsMut<[u8]> + AsRef<[u8]> + ByteArray {
        /// A format filterer, can be used to ensure that `from_ss58check` family only decode for
        /// allowed identifiers. By default just refuses the two reserved identifiers.
        fn format_is_allowed(f: Ss58AddressFormat) -> bool {
            !f.is_reserved()
        }
        /// Some if the string is a properly encoded SS58Check address.
        #[cfg(feature = "serde")]
        fn from_ss58check(s: &str) -> Result<Self, PublicError> {
            Self::from_ss58check_with_version(s).and_then(|(r, v)| match v {
                v if !v.is_custom() => Ok(r),
                v if v == default_ss58_version() => Ok(r),
                v => Err(PublicError::UnknownSs58AddressFormat(v)),
            })
        }
        /// Some if the string is a properly encoded SS58Check address.
        #[cfg(feature = "serde")]
        fn from_ss58check_with_version(s: &str) -> Result<(Self, Ss58AddressFormat), PublicError> {
            const CHECKSUM_LEN: usize = 2;
            let body_len = Self::LEN;
            let data = bs58::decode(s)
                .into_vec()
                .map_err(|_| PublicError::BadBase58)?;
            if data.len() < 2 {
                return Err(PublicError::BadLength);
            }
            let (prefix_len, ident) = match data[0] {
                0..=63 => (1, data[0] as u16),
                64..=127 => {
                    let lower = (data[0] << 2) | (data[1] >> 6);
                    let upper = data[1] & 0b00111111;
                    (2, (lower as u16) | ((upper as u16) << 8))
                }
                _ => return Err(PublicError::InvalidPrefix),
            };
            if data.len() != prefix_len + body_len + CHECKSUM_LEN {
                return Err(PublicError::BadLength);
            }
            let format = ident.into();
            if !Self::format_is_allowed(format) {
                return Err(PublicError::FormatNotAllowed);
            }
            let hash = ss58hash(&data[0..body_len + prefix_len]);
            let checksum = &hash[0..CHECKSUM_LEN];
            if data[body_len + prefix_len..body_len + prefix_len + CHECKSUM_LEN] != *checksum {
                return Err(PublicError::InvalidChecksum);
            }
            let result = Self::from_slice(&data[prefix_len..body_len + prefix_len])
                .map_err(|()| PublicError::BadLength)?;
            Ok((result, format))
        }
        /// Some if the string is a properly encoded SS58Check address, optionally with
        /// a derivation path following.
        #[cfg(feature = "std")]
        fn from_string(s: &str) -> Result<Self, PublicError> {
            Self::from_string_with_version(s).and_then(|(r, v)| match v {
                v if !v.is_custom() => Ok(r),
                v if v == default_ss58_version() => Ok(r),
                v => Err(PublicError::UnknownSs58AddressFormat(v)),
            })
        }
        /// Return the ss58-check string for this key.
        #[cfg(feature = "serde")]
        fn to_ss58check_with_version(&self, version: Ss58AddressFormat) -> String {
            let ident: u16 = u16::from(version) & 0b0011_1111_1111_1111;
            let mut v = match ident {
                0..=63 => <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([ident as u8]),
                ),
                64..=16_383 => {
                    let first = ((ident & 0b0000_0000_1111_1100) as u8) >> 2;
                    let second =
                        ((ident >> 8) as u8) | ((ident & 0b0000_0000_0000_0011) as u8) << 6;
                    <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([first | 0b01000000, second]),
                    )
                }
                _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                    &["internal error: entered unreachable code: "],
                    &[::core::fmt::ArgumentV1::new_display(
                        &::core::fmt::Arguments::new_v1(
                            &["masked out the upper two bits; qed"],
                            &[],
                        ),
                    )],
                )),
            };
            v.extend(self.as_ref());
            let r = ss58hash(&v);
            v.extend(&r[0..2]);
            bs58::encode(v).into_string()
        }
        /// Return the ss58-check string for this key.
        #[cfg(feature = "serde")]
        fn to_ss58check(&self) -> String {
            self.to_ss58check_with_version(default_ss58_version())
        }
        /// Some if the string is a properly encoded SS58Check address, optionally with
        /// a derivation path following.
        #[cfg(feature = "std")]
        fn from_string_with_version(s: &str) -> Result<(Self, Ss58AddressFormat), PublicError> {
            Self::from_ss58check_with_version(s)
        }
    }
    /// Derivable key trait.
    pub trait Derive: Sized {
        /// Derive a child key from a series of given junctions.
        ///
        /// Will be `None` for public keys if there are any hard junctions in there.
        #[cfg(feature = "serde")]
        fn derive<Iter: Iterator<Item = DeriveJunction>>(&self, _path: Iter) -> Option<Self> {
            None
        }
    }
    #[cfg(feature = "serde")]
    const PREFIX: &[u8] = b"SS58PRE";
    #[cfg(feature = "serde")]
    fn ss58hash(data: &[u8]) -> Vec<u8> {
        use blake2::{Blake2b512, Digest};
        let mut ctx = Blake2b512::new();
        ctx.update(PREFIX);
        ctx.update(data);
        ctx.finalize().to_vec()
    }
    /// Default prefix number
    #[cfg(feature = "serde")]
    static DEFAULT_VERSION: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(
        from_known_address_format(Ss58AddressFormatRegistry::SubstrateAccount),
    );
    /// Returns default SS58 format used by the current active process.
    #[cfg(feature = "serde")]
    pub fn default_ss58_version() -> Ss58AddressFormat {
        DEFAULT_VERSION
            .load(core::sync::atomic::Ordering::Relaxed)
            .into()
    }
    /// Returns either the input address format or the default.
    #[cfg(feature = "serde")]
    pub fn unwrap_or_default_ss58_version(network: Option<Ss58AddressFormat>) -> Ss58AddressFormat {
        network.unwrap_or_else(default_ss58_version)
    }
    /// Set the default SS58 "version".
    ///
    /// This SS58 version/format will be used when encoding/decoding SS58 addresses.
    ///
    /// If you want to support a custom SS58 prefix (that isn't yet registered in the `ss58-registry`),
    /// you are required to call this function with your desired prefix [`Ss58AddressFormat::custom`].
    /// This will enable the node to decode ss58 addresses with this prefix.
    ///
    /// This SS58 version/format is also only used by the node and not by the runtime.
    #[cfg(feature = "serde")]
    pub fn set_default_ss58_version(new_default: Ss58AddressFormat) {
        DEFAULT_VERSION.store(new_default.into(), core::sync::atomic::Ordering::Relaxed);
    }
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct SS58_REGEX {
        __private_field: (),
    }
    #[doc(hidden)]
    static SS58_REGEX: SS58_REGEX = SS58_REGEX {
        __private_field: (),
    };
    impl ::lazy_static::__Deref for SS58_REGEX {
        type Target = Regex;
        fn deref(&self) -> &Regex {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"^(?P<ss58>[\w\d ]+)?(?P<path>(//?[^/]+)*)$")
                    .expect("constructed from known-good static value; qed")
            }
            #[inline(always)]
            fn __stability() -> &'static Regex {
                static LAZY: ::lazy_static::lazy::Lazy<Regex> = ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for SS58_REGEX {
        fn initialize(lazy: &Self) {
            let _ = &**lazy;
        }
    }
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct SECRET_PHRASE_REGEX {
        __private_field: (),
    }
    #[doc(hidden)]
    static SECRET_PHRASE_REGEX: SECRET_PHRASE_REGEX = SECRET_PHRASE_REGEX {
        __private_field: (),
    };
    impl ::lazy_static::__Deref for SECRET_PHRASE_REGEX {
        type Target = Regex;
        fn deref(&self) -> &Regex {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"^(?P<phrase>[\d\w ]+)?(?P<path>(//?[^/]+)*)(///(?P<password>.*))?$")
                    .expect("constructed from known-good static value; qed")
            }
            #[inline(always)]
            fn __stability() -> &'static Regex {
                static LAZY: ::lazy_static::lazy::Lazy<Regex> = ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for SECRET_PHRASE_REGEX {
        fn initialize(lazy: &Self) {
            let _ = &**lazy;
        }
    }
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct JUNCTION_REGEX {
        __private_field: (),
    }
    #[doc(hidden)]
    static JUNCTION_REGEX: JUNCTION_REGEX = JUNCTION_REGEX {
        __private_field: (),
    };
    impl ::lazy_static::__Deref for JUNCTION_REGEX {
        type Target = Regex;
        fn deref(&self) -> &Regex {
            #[inline(always)]
            fn __static_ref_initialize() -> Regex {
                Regex::new(r"/(/?[^/]+)").expect("constructed from known-good static value; qed")
            }
            #[inline(always)]
            fn __stability() -> &'static Regex {
                static LAZY: ::lazy_static::lazy::Lazy<Regex> = ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for JUNCTION_REGEX {
        fn initialize(lazy: &Self) {
            let _ = &**lazy;
        }
    }
    #[cfg(feature = "std")]
    impl<T: Sized + AsMut<[u8]> + AsRef<[u8]> + Public + Derive> Ss58Codec for T {
        fn from_string(s: &str) -> Result<Self, PublicError> {
            let cap = SS58_REGEX.captures(s).ok_or(PublicError::InvalidFormat)?;
            let s = cap.name("ss58").map(|r| r.as_str()).unwrap_or(DEV_ADDRESS);
            let addr = if let Some(stripped) = s.strip_prefix("0x") {
                let d = array_bytes::hex2bytes(stripped).map_err(|_| PublicError::InvalidFormat)?;
                Self::from_slice(&d).map_err(|()| PublicError::BadLength)?
            } else {
                Self::from_ss58check(s)?
            };
            if cap["path"].is_empty() {
                Ok(addr)
            } else {
                let path = JUNCTION_REGEX
                    .captures_iter(&cap["path"])
                    .map(|f| DeriveJunction::from(&f[1]));
                addr.derive(path).ok_or(PublicError::InvalidPath)
            }
        }
        fn from_string_with_version(s: &str) -> Result<(Self, Ss58AddressFormat), PublicError> {
            let cap = SS58_REGEX.captures(s).ok_or(PublicError::InvalidFormat)?;
            let (addr, v) = Self::from_ss58check_with_version(
                cap.name("ss58").map(|r| r.as_str()).unwrap_or(DEV_ADDRESS),
            )?;
            if cap["path"].is_empty() {
                Ok((addr, v))
            } else {
                let path = JUNCTION_REGEX
                    .captures_iter(&cap["path"])
                    .map(|f| DeriveJunction::from(&f[1]));
                addr.derive(path)
                    .ok_or(PublicError::InvalidPath)
                    .map(|a| (a, v))
            }
        }
    }
    /// Trait used for types that are really just a fixed-length array.
    pub trait ByteArray: AsRef<[u8]> + AsMut<[u8]> + for<'a> TryFrom<&'a [u8], Error = ()> {
        /// The "length" of the values of this type, which is always the same.
        const LEN: usize;
        /// A new instance from the given slice that should be `Self::LEN` bytes long.
        fn from_slice(data: &[u8]) -> Result<Self, ()> {
            Self::try_from(data)
        }
        /// Return a `Vec<u8>` filled with raw data.
        fn to_raw_vec(&self) -> Vec<u8> {
            self.as_slice().to_vec()
        }
        /// Return a slice filled with raw data.
        fn as_slice(&self) -> &[u8] {
            self.as_ref()
        }
    }
    /// Trait suitable for typical cryptographic key public type.
    pub trait Public:
        ByteArray + Derive + CryptoType + PartialEq + Eq + Clone + Send + Sync
    {
    }
    /// An opaque 32-byte cryptographic identifier.
    pub struct AccountId32([u8; 32]);
    #[automatically_derived]
    impl ::core::hash::Hash for AccountId32 {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AccountId32 {
        #[inline]
        fn clone(&self) -> AccountId32 {
            AccountId32(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for AccountId32 {}
    #[automatically_derived]
    impl ::core::cmp::Eq for AccountId32 {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 32]>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AccountId32 {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AccountId32 {
        #[inline]
        fn eq(&self, other: &AccountId32) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for AccountId32 {
        #[inline]
        fn cmp(&self, other: &AccountId32) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for AccountId32 {
        #[inline]
        fn partial_cmp(
            &self,
            other: &AccountId32,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for AccountId32 {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for AccountId32 {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for AccountId32 {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(AccountId32({
                    let __codec_res_edqy =
                        <[u8; 32] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `AccountId32.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for AccountId32 {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 32]>::max_encoded_len())
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for AccountId32 {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("AccountId32", "sp_core::crypto"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["An opaque 32-byte cryptographic identifier."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 32]>().type_name("[u8; 32]")),
                    )
            }
        };
    };
    impl AccountId32 {
        /// Create a new instance from its raw inner byte value.
        ///
        /// Equivalent to this types `From<[u8; 32]>` implementation. For the lack of const
        /// support in traits we have this constructor.
        pub const fn new(inner: [u8; 32]) -> Self {
            Self(inner)
        }
    }
    impl UncheckedFrom<crate::hash::H256> for AccountId32 {
        fn unchecked_from(h: crate::hash::H256) -> Self {
            AccountId32(h.into())
        }
    }
    impl ByteArray for AccountId32 {
        const LEN: usize = 32;
    }
    #[cfg(feature = "serde")]
    impl Ss58Codec for AccountId32 {}
    impl AsRef<[u8]> for AccountId32 {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for AccountId32 {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    impl AsRef<[u8; 32]> for AccountId32 {
        fn as_ref(&self) -> &[u8; 32] {
            &self.0
        }
    }
    impl AsMut<[u8; 32]> for AccountId32 {
        fn as_mut(&mut self) -> &mut [u8; 32] {
            &mut self.0
        }
    }
    impl From<[u8; 32]> for AccountId32 {
        fn from(x: [u8; 32]) -> Self {
            Self::new(x)
        }
    }
    impl<'a> TryFrom<&'a [u8]> for AccountId32 {
        type Error = ();
        fn try_from(x: &'a [u8]) -> Result<AccountId32, ()> {
            if x.len() == 32 {
                let mut data = [0; 32];
                data.copy_from_slice(x);
                Ok(AccountId32(data))
            } else {
                Err(())
            }
        }
    }
    impl From<AccountId32> for [u8; 32] {
        fn from(x: AccountId32) -> [u8; 32] {
            x.0
        }
    }
    impl From<sr25519::Public> for AccountId32 {
        fn from(k: sr25519::Public) -> Self {
            k.0.into()
        }
    }
    impl From<ed25519::Public> for AccountId32 {
        fn from(k: ed25519::Public) -> Self {
            k.0.into()
        }
    }
    #[cfg(feature = "std")]
    impl std::fmt::Display for AccountId32 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(&self.to_ss58check())],
            ))
        }
    }
    impl sp_std::fmt::Debug for AccountId32 {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            let s = self.to_ss58check();
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &["", " (", "...)"],
                &[
                    ::core::fmt::ArgumentV1::new_display(&crate::hexdisplay::HexDisplay::from(
                        &self.0,
                    )),
                    ::core::fmt::ArgumentV1::new_display(&&s[0..8]),
                ],
            ))
        }
    }
    #[cfg(feature = "serde")]
    impl serde::Serialize for AccountId32 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.to_ss58check())
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> serde::Deserialize<'de> for AccountId32 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ss58Codec::from_ss58check(&String::deserialize(deserializer)?).map_err(|e| {
                serde::de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    #[cfg(feature = "std")]
    impl sp_std::str::FromStr for AccountId32 {
        type Err = &'static str;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let hex_or_ss58_without_prefix = s.trim_start_matches("0x");
            if hex_or_ss58_without_prefix.len() == 64 {
                array_bytes::hex_n_into(hex_or_ss58_without_prefix)
                    .map_err(|_| "invalid hex address.")
            } else {
                Self::from_ss58check(s).map_err(|_| "invalid ss58 address.")
            }
        }
    }
    #[cfg(feature = "std")]
    pub use self::dummy::*;
    #[cfg(feature = "std")]
    mod dummy {
        use super::*;
        /// Dummy cryptography. Doesn't do anything.
        pub struct Dummy;
        #[automatically_derived]
        impl ::core::clone::Clone for Dummy {
            #[inline]
            fn clone(&self) -> Dummy {
                Dummy
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for Dummy {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
        }
        #[automatically_derived]
        impl ::core::default::Default for Dummy {
            #[inline]
            fn default() -> Dummy {
                Dummy {}
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for Dummy {}
        #[automatically_derived]
        impl ::core::cmp::Eq for Dummy {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Dummy {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Dummy {
            #[inline]
            fn eq(&self, other: &Dummy) -> bool {
                true
            }
        }
        impl AsRef<[u8]> for Dummy {
            fn as_ref(&self) -> &[u8] {
                &b""[..]
            }
        }
        impl AsMut<[u8]> for Dummy {
            fn as_mut(&mut self) -> &mut [u8] {
                unsafe {
                    #[allow(mutable_transmutes)]
                    sp_std::mem::transmute::<_, &'static mut [u8]>(&b""[..])
                }
            }
        }
        impl<'a> TryFrom<&'a [u8]> for Dummy {
            type Error = ();
            fn try_from(_: &'a [u8]) -> Result<Self, ()> {
                Ok(Self)
            }
        }
        impl CryptoType for Dummy {
            type Pair = Dummy;
        }
        impl Derive for Dummy {}
        impl ByteArray for Dummy {
            const LEN: usize = 0;
            fn from_slice(_: &[u8]) -> Result<Self, ()> {
                Ok(Self)
            }
            #[cfg(feature = "std")]
            fn to_raw_vec(&self) -> Vec<u8> {
                ::alloc::vec::Vec::new()
            }
            fn as_slice(&self) -> &[u8] {
                b""
            }
        }
        impl Public for Dummy {}
        impl Pair for Dummy {
            type Public = Dummy;
            type Seed = Dummy;
            type Signature = Dummy;
            #[cfg(feature = "std")]
            fn generate_with_phrase(_: Option<&str>) -> (Self, String, Self::Seed) {
                Default::default()
            }
            #[cfg(feature = "std")]
            fn from_phrase(
                _: &str,
                _: Option<&str>,
            ) -> Result<(Self, Self::Seed), SecretStringError> {
                Ok(Default::default())
            }
            fn derive<Iter: Iterator<Item = DeriveJunction>>(
                &self,
                _: Iter,
                _: Option<Dummy>,
            ) -> Result<(Self, Option<Dummy>), DeriveError> {
                Ok((Self, None))
            }
            fn from_seed_slice(_: &[u8]) -> Result<Self, SecretStringError> {
                Ok(Self)
            }
            fn sign(&self, _: &[u8]) -> Self::Signature {
                Self
            }
            fn verify<M: AsRef<[u8]>>(_: &Self::Signature, _: M, _: &Self::Public) -> bool {
                true
            }
            fn public(&self) -> Self::Public {
                Self
            }
            fn to_raw_vec(&self) -> Vec<u8> {
                ::alloc::vec::Vec::new()
            }
        }
    }
    /// A secret uri (`SURI`) that can be used to generate a key pair.
    ///
    /// The `SURI` can be parsed from a string. The string is interpreted in the following way:
    ///
    /// - If `string` is a possibly `0x` prefixed 64-digit hex string, then it will be interpreted
    /// directly as a `MiniSecretKey` (aka "seed" in `subkey`).
    /// - If `string` is a valid BIP-39 key phrase of 12, 15, 18, 21 or 24 words, then the key will
    /// be derived from it. In this case:
    ///   - the phrase may be followed by one or more items delimited by `/` characters.
    ///   - the path may be followed by `///`, in which case everything after the `///` is treated
    /// as a password.
    /// - If `string` begins with a `/` character it is prefixed with the Substrate public `DEV_PHRASE`
    ///   and interpreted as above.
    ///
    /// In this case they are interpreted as HDKD junctions; purely numeric items are interpreted as
    /// integers, non-numeric items as strings. Junctions prefixed with `/` are interpreted as soft
    /// junctions, and with `//` as hard junctions.
    ///
    /// There is no correspondence mapping between `SURI` strings and the keys they represent.
    /// Two different non-identical strings can actually lead to the same secret being derived.
    /// Notably, integer junction indices may be legally prefixed with arbitrary number of zeros.
    /// Similarly an empty password (ending the `SURI` with `///`) is perfectly valid and will
    /// generally be equivalent to no password at all.
    ///
    /// # Example
    ///
    /// Parse [`DEV_PHRASE`] secret uri with junction:
    ///
    /// ```
    /// # use sp_core::crypto::{SecretUri, DeriveJunction, DEV_PHRASE, ExposeSecret};
    /// # use std::str::FromStr;
    /// let suri = SecretUri::from_str("//Alice").expect("Parse SURI");
    ///
    /// assert_eq!(vec![DeriveJunction::from("Alice").harden()], suri.junctions);
    /// assert_eq!(DEV_PHRASE, suri.phrase.expose_secret());
    /// assert!(suri.password.is_none());
    /// ```
    ///
    /// Parse [`DEV_PHRASE`] secret ui with junction and password:
    ///
    /// ```
    /// # use sp_core::crypto::{SecretUri, DeriveJunction, DEV_PHRASE, ExposeSecret};
    /// # use std::str::FromStr;
    /// let suri = SecretUri::from_str("//Alice///SECRET_PASSWORD").expect("Parse SURI");
    ///
    /// assert_eq!(vec![DeriveJunction::from("Alice").harden()], suri.junctions);
    /// assert_eq!(DEV_PHRASE, suri.phrase.expose_secret());
    /// assert_eq!("SECRET_PASSWORD", suri.password.unwrap().expose_secret());
    /// ```
    ///
    /// Parse [`DEV_PHRASE`] secret ui with hex phrase and junction:
    ///
    /// ```
    /// # use sp_core::crypto::{SecretUri, DeriveJunction, DEV_PHRASE, ExposeSecret};
    /// # use std::str::FromStr;
    /// let suri = SecretUri::from_str("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a//Alice").expect("Parse SURI");
    ///
    /// assert_eq!(vec![DeriveJunction::from("Alice").harden()], suri.junctions);
    /// assert_eq!("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a", suri.phrase.expose_secret());
    /// assert!(suri.password.is_none());
    /// ```
    #[cfg(feature = "std")]
    pub struct SecretUri {
        /// The phrase to derive the private key.
        ///
        /// This can either be a 64-bit hex string or a BIP-39 key phrase.
        pub phrase: SecretString,
        /// Optional password as given as part of the uri.
        pub password: Option<SecretString>,
        /// The junctions as part of the uri.
        pub junctions: Vec<DeriveJunction>,
    }
    #[cfg(feature = "std")]
    impl sp_std::str::FromStr for SecretUri {
        type Err = SecretStringError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let cap = SECRET_PHRASE_REGEX
                .captures(s)
                .ok_or(SecretStringError::InvalidFormat)?;
            let junctions = JUNCTION_REGEX
                .captures_iter(&cap["path"])
                .map(|f| DeriveJunction::from(&f[1]))
                .collect::<Vec<_>>();
            let phrase = cap.name("phrase").map(|r| r.as_str()).unwrap_or(DEV_PHRASE);
            let password = cap.name("password");
            Ok(Self {
                phrase: SecretString::from_str(phrase).expect("Returns infallible error; qed"),
                password: password.map(|v| {
                    SecretString::from_str(v.as_str()).expect("Returns infallible error; qed")
                }),
                junctions,
            })
        }
    }
    /// Trait suitable for typical cryptographic PKI key pair type.
    ///
    /// For now it just specifies how to create a key from a phrase and derivation path.
    #[cfg(feature = "full_crypto")]
    pub trait Pair: CryptoType + Sized + Clone + Send + Sync + 'static {
        /// The type which is used to encode a public key.
        type Public: Public + Hash;
        /// The type used to (minimally) encode the data required to securely create
        /// a new key pair.
        type Seed: Default + AsRef<[u8]> + AsMut<[u8]> + Clone;
        /// The type used to represent a signature. Can be created from a key pair and a message
        /// and verified with the message and a public key.
        type Signature: AsRef<[u8]>;
        /// Generate new secure (random) key pair.
        ///
        /// This is only for ephemeral keys really, since you won't have access to the secret key
        /// for storage. If you want a persistent key pair, use `generate_with_phrase` instead.
        #[cfg(feature = "std")]
        fn generate() -> (Self, Self::Seed) {
            let mut seed = Self::Seed::default();
            OsRng.fill_bytes(seed.as_mut());
            (Self::from_seed(&seed), seed)
        }
        /// Generate new secure (random) key pair and provide the recovery phrase.
        ///
        /// You can recover the same key later with `from_phrase`.
        ///
        /// This is generally slower than `generate()`, so prefer that unless you need to persist
        /// the key from the current session.
        #[cfg(feature = "std")]
        fn generate_with_phrase(password: Option<&str>) -> (Self, String, Self::Seed) {
            let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
            let phrase = mnemonic.phrase();
            let (pair, seed) = Self::from_phrase(phrase, password)
                .expect("All phrases generated by Mnemonic are valid; qed");
            (pair, phrase.to_owned(), seed)
        }
        /// Returns the KeyPair from the English BIP39 seed `phrase`, or `None` if it's invalid.
        #[cfg(feature = "std")]
        fn from_phrase(
            phrase: &str,
            password: Option<&str>,
        ) -> Result<(Self, Self::Seed), SecretStringError> {
            let mnemonic = Mnemonic::from_phrase(phrase, Language::English)
                .map_err(|_| SecretStringError::InvalidPhrase)?;
            let big_seed =
                substrate_bip39::seed_from_entropy(mnemonic.entropy(), password.unwrap_or(""))
                    .map_err(|_| SecretStringError::InvalidSeed)?;
            let mut seed = Self::Seed::default();
            let seed_slice = seed.as_mut();
            let seed_len = seed_slice.len();
            if true {
                if !(seed_len <= big_seed.len()) {
                    ::core::panicking::panic("assertion failed: seed_len <= big_seed.len()")
                };
            };
            seed_slice[..seed_len].copy_from_slice(&big_seed[..seed_len]);
            Self::from_seed_slice(seed_slice).map(|x| (x, seed))
        }
        /// Derive a child key from a series of given junctions.
        fn derive<Iter: Iterator<Item = DeriveJunction>>(
            &self,
            path: Iter,
            seed: Option<Self::Seed>,
        ) -> Result<(Self, Option<Self::Seed>), DeriveError>;
        /// Generate new key pair from the provided `seed`.
        ///
        /// @WARNING: THIS WILL ONLY BE SECURE IF THE `seed` IS SECURE. If it can be guessed
        /// by an attacker then they can also derive your key.
        fn from_seed(seed: &Self::Seed) -> Self {
            Self::from_seed_slice(seed.as_ref()).expect("seed has valid length; qed")
        }
        /// Make a new key pair from secret seed material. The slice must be the correct size or
        /// it will return `None`.
        ///
        /// @WARNING: THIS WILL ONLY BE SECURE IF THE `seed` IS SECURE. If it can be guessed
        /// by an attacker then they can also derive your key.
        fn from_seed_slice(seed: &[u8]) -> Result<Self, SecretStringError>;
        /// Sign a message.
        fn sign(&self, message: &[u8]) -> Self::Signature;
        /// Verify a signature on a message. Returns true if the signature is good.
        fn verify<M: AsRef<[u8]>>(sig: &Self::Signature, message: M, pubkey: &Self::Public)
            -> bool;
        /// Get the public key.
        fn public(&self) -> Self::Public;
        /// Interprets the string `s` in order to generate a key Pair. Returns both the pair and an
        /// optional seed, in the case that the pair can be expressed as a direct derivation from a seed
        /// (some cases, such as Sr25519 derivations with path components, cannot).
        ///
        /// This takes a helper function to do the key generation from a phrase, password and
        /// junction iterator.
        ///
        /// - If `s` is a possibly `0x` prefixed 64-digit hex string, then it will be interpreted
        /// directly as a `MiniSecretKey` (aka "seed" in `subkey`).
        /// - If `s` is a valid BIP-39 key phrase of 12, 15, 18, 21 or 24 words, then the key will
        /// be derived from it. In this case:
        ///   - the phrase may be followed by one or more items delimited by `/` characters.
        ///   - the path may be followed by `///`, in which case everything after the `///` is treated
        /// as a password.
        /// - If `s` begins with a `/` character it is prefixed with the Substrate public `DEV_PHRASE`
        ///   and
        /// interpreted as above.
        ///
        /// In this case they are interpreted as HDKD junctions; purely numeric items are interpreted as
        /// integers, non-numeric items as strings. Junctions prefixed with `/` are interpreted as soft
        /// junctions, and with `//` as hard junctions.
        ///
        /// There is no correspondence mapping between SURI strings and the keys they represent.
        /// Two different non-identical strings can actually lead to the same secret being derived.
        /// Notably, integer junction indices may be legally prefixed with arbitrary number of zeros.
        /// Similarly an empty password (ending the SURI with `///`) is perfectly valid and will
        /// generally be equivalent to no password at all.
        ///
        /// `None` is returned if no matches are found.
        #[cfg(feature = "std")]
        fn from_string_with_seed(
            s: &str,
            password_override: Option<&str>,
        ) -> Result<(Self, Option<Self::Seed>), SecretStringError> {
            use sp_std::str::FromStr;
            let SecretUri {
                junctions,
                phrase,
                password,
            } = SecretUri::from_str(s)?;
            let password =
                password_override.or_else(|| password.as_ref().map(|p| p.expose_secret().as_str()));
            let (root, seed) = if let Some(stripped) = phrase.expose_secret().strip_prefix("0x") {
                array_bytes::hex2bytes(stripped)
                    .ok()
                    .and_then(|seed_vec| {
                        let mut seed = Self::Seed::default();
                        if seed.as_ref().len() == seed_vec.len() {
                            seed.as_mut().copy_from_slice(&seed_vec);
                            Some((Self::from_seed(&seed), seed))
                        } else {
                            None
                        }
                    })
                    .ok_or(SecretStringError::InvalidSeed)?
            } else {
                Self::from_phrase(phrase.expose_secret().as_str(), password)
                    .map_err(|_| SecretStringError::InvalidPhrase)?
            };
            root.derive(junctions.into_iter(), Some(seed))
                .map_err(|_| SecretStringError::InvalidPath)
        }
        /// Interprets the string `s` in order to generate a key pair.
        ///
        /// See [`from_string_with_seed`](Pair::from_string_with_seed) for more extensive documentation.
        #[cfg(feature = "std")]
        fn from_string(
            s: &str,
            password_override: Option<&str>,
        ) -> Result<Self, SecretStringError> {
            Self::from_string_with_seed(s, password_override).map(|x| x.0)
        }
        /// Return a vec filled with raw data.
        fn to_raw_vec(&self) -> Vec<u8>;
    }
    /// One type is wrapped by another.
    pub trait IsWrappedBy<Outer>: From<Outer> + Into<Outer> {
        /// Get a reference to the inner from the outer.
        fn from_ref(outer: &Outer) -> &Self;
        /// Get a mutable reference to the inner from the outer.
        fn from_mut(outer: &mut Outer) -> &mut Self;
    }
    /// Opposite of `IsWrappedBy` - denotes a type which is a simple wrapper around another type.
    pub trait Wraps: Sized {
        /// The inner type it is wrapping.
        type Inner: IsWrappedBy<Self>;
        /// Get a reference to the inner type that is wrapped.
        fn as_inner_ref(&self) -> &Self::Inner {
            Self::Inner::from_ref(self)
        }
    }
    impl<T, Outer> IsWrappedBy<Outer> for T
    where
        Outer: AsRef<Self> + AsMut<Self> + From<Self>,
        T: From<Outer>,
    {
        /// Get a reference to the inner from the outer.
        fn from_ref(outer: &Outer) -> &Self {
            outer.as_ref()
        }
        /// Get a mutable reference to the inner from the outer.
        fn from_mut(outer: &mut Outer) -> &mut Self {
            outer.as_mut()
        }
    }
    impl<Inner, Outer, T> UncheckedFrom<T> for Outer
    where
        Outer: Wraps<Inner = Inner>,
        Inner: IsWrappedBy<Outer> + UncheckedFrom<T>,
    {
        fn unchecked_from(t: T) -> Self {
            let inner: Inner = t.unchecked_into();
            inner.into()
        }
    }
    /// Type which has a particular kind of crypto associated with it.
    pub trait CryptoType {
        /// The pair key type of this crypto.
        #[cfg(feature = "full_crypto")]
        type Pair: Pair;
    }
    /// An identifier for a type of cryptographic key.
    ///
    /// To avoid clashes with other modules when distributing your module publicly, register your
    /// `KeyTypeId` on the list here by making a PR.
    ///
    /// Values whose first character is `_` are reserved for private use and won't conflict with any
    /// public modules.
    pub struct KeyTypeId(pub [u8; 4]);
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for KeyTypeId {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                _serde::Serializer::serialize_newtype_struct(__serializer, "KeyTypeId", &self.0)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for KeyTypeId {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<KeyTypeId>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = KeyTypeId;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "tuple struct KeyTypeId",
                        )
                    }
                    #[inline]
                    fn visit_newtype_struct<__E>(
                        self,
                        __e: __E,
                    ) -> _serde::__private::Result<Self::Value, __E::Error>
                    where
                        __E: _serde::Deserializer<'de>,
                    {
                        let __field0: [u8; 4] =
                            match <[u8; 4] as _serde::Deserialize>::deserialize(__e) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            };
                        _serde::__private::Ok(KeyTypeId(__field0))
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match match _serde::de::SeqAccess::next_element::<[u8; 4]>(
                            &mut __seq,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    0usize,
                                    &"tuple struct KeyTypeId with 1 element",
                                ));
                            }
                        };
                        _serde::__private::Ok(KeyTypeId(__field0))
                    }
                }
                _serde::Deserializer::deserialize_newtype_struct(
                    __deserializer,
                    "KeyTypeId",
                    __Visitor {
                        marker: _serde::__private::PhantomData::<KeyTypeId>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::marker::Copy for KeyTypeId {}
    #[automatically_derived]
    impl ::core::clone::Clone for KeyTypeId {
        #[inline]
        fn clone(&self) -> KeyTypeId {
            let _: ::core::clone::AssertParamIsClone<[u8; 4]>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for KeyTypeId {
        #[inline]
        fn default() -> KeyTypeId {
            KeyTypeId(::core::default::Default::default())
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for KeyTypeId {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for KeyTypeId {
        #[inline]
        fn eq(&self, other: &KeyTypeId) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for KeyTypeId {}
    #[automatically_derived]
    impl ::core::cmp::Eq for KeyTypeId {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 4]>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for KeyTypeId {
        #[inline]
        fn partial_cmp(&self, other: &KeyTypeId) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for KeyTypeId {
        #[inline]
        fn cmp(&self, other: &KeyTypeId) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for KeyTypeId {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for KeyTypeId {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for KeyTypeId {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for KeyTypeId {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(KeyTypeId({
                    let __codec_res_edqy = <[u8; 4] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `KeyTypeId.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for KeyTypeId {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 4]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for KeyTypeId {
            type Inner = [u8; 4];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    impl core::fmt::Debug for KeyTypeId {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_tuple("KeyTypeId").field(&self.0).finish()
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for KeyTypeId {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("KeyTypeId" , "sp_core::crypto")) . type_params (:: alloc :: vec :: Vec :: new ()) . docs (& ["An identifier for a type of cryptographic key." , "" , "To avoid clashes with other modules when distributing your module publicly, register your" , "`KeyTypeId` on the list here by making a PR." , "" , "Values whose first character is `_` are reserved for private use and won't conflict with any" , "public modules."]) . composite (:: scale_info :: build :: Fields :: unnamed () . field (| f | f . ty :: < [u8 ; 4] > () . type_name ("[u8; 4]")))
            }
        };
    };
    impl From<u32> for KeyTypeId {
        fn from(x: u32) -> Self {
            Self(x.to_le_bytes())
        }
    }
    impl From<KeyTypeId> for u32 {
        fn from(x: KeyTypeId) -> Self {
            u32::from_le_bytes(x.0)
        }
    }
    impl<'a> TryFrom<&'a str> for KeyTypeId {
        type Error = ();
        fn try_from(x: &'a str) -> Result<Self, ()> {
            let b = x.as_bytes();
            if b.len() != 4 {
                return Err(());
            }
            let mut res = KeyTypeId::default();
            res.0.copy_from_slice(&b[0..4]);
            Ok(res)
        }
    }
    /// Trait grouping types shared by a VRF signer and verifiers.
    pub trait VrfCrypto {
        /// VRF input.
        type VrfInput;
        /// VRF output.
        type VrfOutput;
        /// VRF signing data.
        type VrfSignData;
        /// VRF signature.
        type VrfSignature;
    }
    /// VRF Secret Key.
    pub trait VrfSecret: VrfCrypto {
        /// Get VRF-specific output .
        fn vrf_output(&self, data: &Self::VrfInput) -> Self::VrfOutput;
        /// Sign VRF-specific data.
        fn vrf_sign(&self, input: &Self::VrfSignData) -> Self::VrfSignature;
    }
    /// VRF Public Key.
    pub trait VrfPublic: VrfCrypto {
        /// Verify input data signature.
        fn vrf_verify(&self, data: &Self::VrfSignData, signature: &Self::VrfSignature) -> bool;
    }
    /// An identifier for a specific cryptographic algorithm used by a key pair
    pub struct CryptoTypeId(pub [u8; 4]);
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for CryptoTypeId {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                _serde::Serializer::serialize_newtype_struct(__serializer, "CryptoTypeId", &self.0)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for CryptoTypeId {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<CryptoTypeId>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = CryptoTypeId;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "tuple struct CryptoTypeId",
                        )
                    }
                    #[inline]
                    fn visit_newtype_struct<__E>(
                        self,
                        __e: __E,
                    ) -> _serde::__private::Result<Self::Value, __E::Error>
                    where
                        __E: _serde::Deserializer<'de>,
                    {
                        let __field0: [u8; 4] =
                            match <[u8; 4] as _serde::Deserialize>::deserialize(__e) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            };
                        _serde::__private::Ok(CryptoTypeId(__field0))
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match match _serde::de::SeqAccess::next_element::<[u8; 4]>(
                            &mut __seq,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    0usize,
                                    &"tuple struct CryptoTypeId with 1 element",
                                ));
                            }
                        };
                        _serde::__private::Ok(CryptoTypeId(__field0))
                    }
                }
                _serde::Deserializer::deserialize_newtype_struct(
                    __deserializer,
                    "CryptoTypeId",
                    __Visitor {
                        marker: _serde::__private::PhantomData::<CryptoTypeId>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for CryptoTypeId {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "CryptoTypeId", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for CryptoTypeId {}
    #[automatically_derived]
    impl ::core::clone::Clone for CryptoTypeId {
        #[inline]
        fn clone(&self) -> CryptoTypeId {
            let _: ::core::clone::AssertParamIsClone<[u8; 4]>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for CryptoTypeId {
        #[inline]
        fn default() -> CryptoTypeId {
            CryptoTypeId(::core::default::Default::default())
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for CryptoTypeId {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for CryptoTypeId {
        #[inline]
        fn eq(&self, other: &CryptoTypeId) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for CryptoTypeId {}
    #[automatically_derived]
    impl ::core::cmp::Eq for CryptoTypeId {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 4]>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for CryptoTypeId {
        #[inline]
        fn partial_cmp(
            &self,
            other: &CryptoTypeId,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for CryptoTypeId {
        #[inline]
        fn cmp(&self, other: &CryptoTypeId) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for CryptoTypeId {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for CryptoTypeId {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for CryptoTypeId {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for CryptoTypeId {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(CryptoTypeId({
                    let __codec_res_edqy = <[u8; 4] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `CryptoTypeId.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    /// Known key types; this also functions as a global registry of key types for projects wishing to
    /// avoid collisions with each other.
    ///
    /// It's not universal in the sense that *all* key types need to be mentioned here, it's just a
    /// handy place to put common key types.
    pub mod key_types {
        use super::KeyTypeId;
        /// Key type for Babe module, built-in. Identified as `babe`.
        pub const BABE: KeyTypeId = KeyTypeId(*b"babe");
        /// Key type for Grandpa module, built-in. Identified as `gran`.
        pub const GRANDPA: KeyTypeId = KeyTypeId(*b"gran");
        /// Key type for controlling an account in a Substrate runtime, built-in. Identified as `acco`.
        pub const ACCOUNT: KeyTypeId = KeyTypeId(*b"acco");
        /// Key type for Aura module, built-in. Identified as `aura`.
        pub const AURA: KeyTypeId = KeyTypeId(*b"aura");
        /// Key type for ImOnline module, built-in. Identified as `imon`.
        pub const IM_ONLINE: KeyTypeId = KeyTypeId(*b"imon");
        /// Key type for AuthorityDiscovery module, built-in. Identified as `audi`.
        pub const AUTHORITY_DISCOVERY: KeyTypeId = KeyTypeId(*b"audi");
        /// Key type for staking, built-in. Identified as `stak`.
        pub const STAKING: KeyTypeId = KeyTypeId(*b"stak");
        /// A key type for signing statements
        pub const STATEMENT: KeyTypeId = KeyTypeId(*b"stmt");
        /// A key type ID useful for tests.
        pub const DUMMY: KeyTypeId = KeyTypeId(*b"dumy");
    }
}
pub mod hexdisplay {
    //! Wrapper type for byte collections that outputs hex.
    /// Simple wrapper to display hex representation of bytes.
    pub struct HexDisplay<'a>(&'a [u8]);
    impl<'a> HexDisplay<'a> {
        /// Create new instance that will display `d` as a hex string when displayed.
        pub fn from<R: AsBytesRef>(d: &'a R) -> Self {
            HexDisplay(d.as_bytes_ref())
        }
    }
    impl<'a> sp_std::fmt::Display for HexDisplay<'a> {
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> Result<(), sp_std::fmt::Error> {
            if self.0.len() < 1027 {
                for byte in self.0 {
                    f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_lower_hex(&byte)],
                        &[::core::fmt::rt::v1::Argument {
                            position: 0usize,
                            format: ::core::fmt::rt::v1::FormatSpec {
                                fill: ' ',
                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                flags: 8u32,
                                precision: ::core::fmt::rt::v1::Count::Implied,
                                width: ::core::fmt::rt::v1::Count::Is(2usize),
                            },
                        }],
                        unsafe { ::core::fmt::UnsafeArg::new() },
                    ))?;
                }
            } else {
                for byte in &self.0[0..512] {
                    f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_lower_hex(&byte)],
                        &[::core::fmt::rt::v1::Argument {
                            position: 0usize,
                            format: ::core::fmt::rt::v1::FormatSpec {
                                fill: ' ',
                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                flags: 8u32,
                                precision: ::core::fmt::rt::v1::Count::Implied,
                                width: ::core::fmt::rt::v1::Count::Is(2usize),
                            },
                        }],
                        unsafe { ::core::fmt::UnsafeArg::new() },
                    ))?;
                }
                f.write_str("...")?;
                for byte in &self.0[self.0.len() - 512..] {
                    f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_lower_hex(&byte)],
                        &[::core::fmt::rt::v1::Argument {
                            position: 0usize,
                            format: ::core::fmt::rt::v1::FormatSpec {
                                fill: ' ',
                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                flags: 8u32,
                                precision: ::core::fmt::rt::v1::Count::Implied,
                                width: ::core::fmt::rt::v1::Count::Is(2usize),
                            },
                        }],
                        unsafe { ::core::fmt::UnsafeArg::new() },
                    ))?;
                }
            }
            Ok(())
        }
    }
    impl<'a> sp_std::fmt::Debug for HexDisplay<'a> {
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> Result<(), sp_std::fmt::Error> {
            for byte in self.0 {
                f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                    &[""],
                    &[::core::fmt::ArgumentV1::new_lower_hex(&byte)],
                    &[::core::fmt::rt::v1::Argument {
                        position: 0usize,
                        format: ::core::fmt::rt::v1::FormatSpec {
                            fill: ' ',
                            align: ::core::fmt::rt::v1::Alignment::Unknown,
                            flags: 8u32,
                            precision: ::core::fmt::rt::v1::Count::Implied,
                            width: ::core::fmt::rt::v1::Count::Is(2usize),
                        },
                    }],
                    unsafe { ::core::fmt::UnsafeArg::new() },
                ))?;
            }
            Ok(())
        }
    }
    /// Simple trait to transform various types to `&[u8]`
    pub trait AsBytesRef {
        /// Transform `self` into `&[u8]`.
        fn as_bytes_ref(&self) -> &[u8];
    }
    impl AsBytesRef for &[u8] {
        fn as_bytes_ref(&self) -> &[u8] {
            self
        }
    }
    impl AsBytesRef for [u8] {
        fn as_bytes_ref(&self) -> &[u8] {
            self
        }
    }
    impl AsBytesRef for sp_std::vec::Vec<u8> {
        fn as_bytes_ref(&self) -> &[u8] {
            self
        }
    }
    impl AsBytesRef for sp_storage::StorageKey {
        fn as_bytes_ref(&self) -> &[u8] {
            self.as_ref()
        }
    }
    impl AsBytesRef for [u8; 1] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 2] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 3] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 4] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 5] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 6] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 7] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 8] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 10] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 12] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 14] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 16] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 20] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 24] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 28] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 32] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 40] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 48] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 56] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 64] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 65] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 80] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 96] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 112] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 128] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    impl AsBytesRef for [u8; 144] {
        fn as_bytes_ref(&self) -> &[u8] {
            &self[..]
        }
    }
    /// Format into ASCII + # + hex, suitable for storage key preimages.
    #[cfg(feature = "std")]
    pub fn ascii_format(asciish: &[u8]) -> String {
        let mut r = String::new();
        let mut latch = false;
        for c in asciish {
            match (latch, *c) {
                (false, 32..=127) => r.push(*c as char),
                _ => {
                    if !latch {
                        r.push('#');
                        latch = true;
                    }
                    r.push_str(&{
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1_formatted(
                            &[""],
                            &[::core::fmt::ArgumentV1::new_lower_hex(&*c)],
                            &[::core::fmt::rt::v1::Argument {
                                position: 0usize,
                                format: ::core::fmt::rt::v1::FormatSpec {
                                    fill: ' ',
                                    align: ::core::fmt::rt::v1::Alignment::Unknown,
                                    flags: 8u32,
                                    precision: ::core::fmt::rt::v1::Count::Implied,
                                    width: ::core::fmt::rt::v1::Count::Is(2usize),
                                },
                            }],
                            unsafe { ::core::fmt::UnsafeArg::new() },
                        ));
                        res
                    });
                }
            }
        }
        r
    }
}
pub use paste;
pub mod defer {
    //! Contains the [`crate::defer!`] macro for *deferring* the execution
    //! of code until the current scope is dropped.
    //! This helps with *always* executing cleanup code.
    /// Executes the wrapped closure on drop.
    ///
    /// Should be used together with the [`crate::defer!`] macro.
    #[must_use]
    pub struct DeferGuard<F: FnOnce()>(pub Option<F>);
    impl<F: FnOnce()> Drop for DeferGuard<F> {
        fn drop(&mut self) {
            self.0.take().map(|f| f());
        }
    }
}
pub mod ecdsa {
    //! Simple ECDSA secp256k1 API.
    use codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_runtime_interface::pass_by::PassByInner;
    #[cfg(feature = "serde")]
    use crate::crypto::Ss58Codec;
    use crate::crypto::{
        ByteArray, CryptoType, CryptoTypeId, Derive, Public as TraitPublic, UncheckedFrom,
    };
    #[cfg(feature = "full_crypto")]
    use crate::{
        crypto::{DeriveError, DeriveJunction, Pair as TraitPair, SecretStringError},
        hashing::blake2_256,
    };
    #[cfg(feature = "std")]
    use secp256k1::SECP256K1;
    #[cfg(feature = "full_crypto")]
    use secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Message, PublicKey, SecretKey,
    };
    #[cfg(feature = "serde")]
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    #[cfg(feature = "full_crypto")]
    use sp_std::vec::Vec;
    /// An identifier used to match public keys against ecdsa keys
    pub const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"ecds");
    /// A secret seed (which is bytewise essentially equivalent to a SecretKey).
    ///
    /// We need it as a different type because `Seed` is expected to be AsRef<[u8]>.
    #[cfg(feature = "full_crypto")]
    type Seed = [u8; 32];
    /// The ECDSA compressed public key.
    pub struct Public(pub [u8; 33]);
    #[automatically_derived]
    impl ::core::clone::Clone for Public {
        #[inline]
        fn clone(&self) -> Public {
            let _: ::core::clone::AssertParamIsClone<[u8; 33]>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Public {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Public {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Public {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Public {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Public({
                    let __codec_res_edqy =
                        <[u8; 33] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Public.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Public {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 33]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Public {
            type Inner = [u8; 33];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for Public {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 33]>::max_encoded_len())
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Public {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Public", "sp_core::ecdsa"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["The ECDSA compressed public key."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 33]>().type_name("[u8; 33]")),
                    )
            }
        };
    };
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Public {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Public {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 33]>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Public {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Public {
        #[inline]
        fn eq(&self, other: &Public) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Public {
        #[inline]
        fn partial_cmp(&self, other: &Public) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Public {
        #[inline]
        fn cmp(&self, other: &Public) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Public {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl Public {
        /// A new instance from the given 33-byte `data`.
        ///
        /// NOTE: No checking goes on to ensure this is a real public key. Only use it if
        /// you are certain that the array actually is a pubkey. GIGO!
        pub fn from_raw(data: [u8; 33]) -> Self {
            Self(data)
        }
        /// Create a new instance from the given full public key.
        ///
        /// This will convert the full public key into the compressed format.
        #[cfg(feature = "std")]
        pub fn from_full(full: &[u8]) -> Result<Self, ()> {
            let pubkey = if full.len() == 64 {
                let mut tagged_full = [0u8; 65];
                tagged_full[0] = 0x04;
                tagged_full[1..].copy_from_slice(full);
                secp256k1::PublicKey::from_slice(&tagged_full)
            } else {
                secp256k1::PublicKey::from_slice(full)
            };
            pubkey.map(|k| Self(k.serialize())).map_err(|_| ())
        }
    }
    impl ByteArray for Public {
        const LEN: usize = 33;
    }
    impl TraitPublic for Public {}
    impl Derive for Public {}
    impl AsRef<[u8]> for Public {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for Public {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    impl TryFrom<&[u8]> for Public {
        type Error = ();
        fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
            if data.len() != Self::LEN {
                return Err(());
            }
            let mut r = [0u8; Self::LEN];
            r.copy_from_slice(data);
            Ok(Self::unchecked_from(r))
        }
    }
    #[cfg(feature = "full_crypto")]
    impl From<Pair> for Public {
        fn from(x: Pair) -> Self {
            x.public()
        }
    }
    impl UncheckedFrom<[u8; 33]> for Public {
        fn unchecked_from(x: [u8; 33]) -> Self {
            Public(x)
        }
    }
    #[cfg(feature = "std")]
    impl std::fmt::Display for Public {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(&self.to_ss58check())],
            ))
        }
    }
    impl sp_std::fmt::Debug for Public {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            let s = self.to_ss58check();
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &["", " (", "...)"],
                &[
                    ::core::fmt::ArgumentV1::new_display(&crate::hexdisplay::HexDisplay::from(
                        &self.as_ref(),
                    )),
                    ::core::fmt::ArgumentV1::new_display(&&s[0..8]),
                ],
            ))
        }
    }
    #[cfg(feature = "serde")]
    impl Serialize for Public {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_ss58check())
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for Public {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Public::from_ss58check(&String::deserialize(deserializer)?).map_err(|e| {
                de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    /// A signature (a 512-bit value, plus 8 bits for recovery ID).
    pub struct Signature(pub [u8; 65]);
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Signature {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Signature {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Signature {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Signature({
                    let __codec_res_edqy =
                        <[u8; 65] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Signature.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for Signature {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 65]>::max_encoded_len())
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Signature {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 65]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Signature {
            type Inner = [u8; 65];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Signature {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Signature", "sp_core::ecdsa"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["A signature (a 512-bit value, plus 8 bits for recovery ID)."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 65]>().type_name("[u8; 65]")),
                    )
            }
        };
    };
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Signature {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Signature {
        #[inline]
        fn eq(&self, other: &Signature) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Signature {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Signature {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 65]>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Signature {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl TryFrom<&[u8]> for Signature {
        type Error = ();
        fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
            if data.len() == 65 {
                let mut inner = [0u8; 65];
                inner.copy_from_slice(data);
                Ok(Signature(inner))
            } else {
                Err(())
            }
        }
    }
    #[cfg(feature = "serde")]
    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&array_bytes::bytes2hex("", self.as_ref()))
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let signature_hex = array_bytes::hex2bytes(&String::deserialize(deserializer)?)
                .map_err(|e| {
                    de::Error::custom({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &[""],
                            &[::core::fmt::ArgumentV1::new_debug(&e)],
                        ));
                        res
                    })
                })?;
            Signature::try_from(signature_hex.as_ref()).map_err(|e| {
                de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    impl Clone for Signature {
        fn clone(&self) -> Self {
            let mut r = [0u8; 65];
            r.copy_from_slice(&self.0[..]);
            Signature(r)
        }
    }
    impl Default for Signature {
        fn default() -> Self {
            Signature([0u8; 65])
        }
    }
    impl From<Signature> for [u8; 65] {
        fn from(v: Signature) -> [u8; 65] {
            v.0
        }
    }
    impl AsRef<[u8; 65]> for Signature {
        fn as_ref(&self) -> &[u8; 65] {
            &self.0
        }
    }
    impl AsRef<[u8]> for Signature {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for Signature {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    impl sp_std::fmt::Debug for Signature {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(
                    &crate::hexdisplay::HexDisplay::from(&self.0),
                )],
            ))
        }
    }
    impl UncheckedFrom<[u8; 65]> for Signature {
        fn unchecked_from(data: [u8; 65]) -> Signature {
            Signature(data)
        }
    }
    impl Signature {
        /// A new instance from the given 65-byte `data`.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_raw(data: [u8; 65]) -> Signature {
            Signature(data)
        }
        /// A new instance from the given slice that should be 65 bytes long.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_slice(data: &[u8]) -> Option<Self> {
            if data.len() != 65 {
                return None;
            }
            let mut r = [0u8; 65];
            r.copy_from_slice(data);
            Some(Signature(r))
        }
        /// Recover the public key from this signature and a message.
        #[cfg(feature = "full_crypto")]
        pub fn recover<M: AsRef<[u8]>>(&self, message: M) -> Option<Public> {
            self.recover_prehashed(&blake2_256(message.as_ref()))
        }
        /// Recover the public key from this signature and a pre-hashed message.
        #[cfg(feature = "full_crypto")]
        pub fn recover_prehashed(&self, message: &[u8; 32]) -> Option<Public> {
            let rid = RecoveryId::from_i32(self.0[64] as i32).ok()?;
            let sig = RecoverableSignature::from_compact(&self.0[..64], rid).ok()?;
            let message = Message::from_slice(message).expect("Message is 32 bytes; qed");
            #[cfg(feature = "std")]
            let context = SECP256K1;
            context
                .recover_ecdsa(&message, &sig)
                .ok()
                .map(|pubkey| Public(pubkey.serialize()))
        }
    }
    #[cfg(feature = "full_crypto")]
    impl From<RecoverableSignature> for Signature {
        fn from(recsig: RecoverableSignature) -> Signature {
            let mut r = Self::default();
            let (recid, sig) = recsig.serialize_compact();
            r.0[..64].copy_from_slice(&sig);
            r.0[64] = recid.to_i32() as u8;
            r
        }
    }
    /// Derive a single hard junction.
    #[cfg(feature = "full_crypto")]
    fn derive_hard_junction(secret_seed: &Seed, cc: &[u8; 32]) -> Seed {
        ("Secp256k1HDKD", secret_seed, cc).using_encoded(sp_core_hashing::blake2_256)
    }
    /// A key pair.
    #[cfg(feature = "full_crypto")]
    pub struct Pair {
        public: Public,
        secret: SecretKey,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Pair {
        #[inline]
        fn clone(&self) -> Pair {
            Pair {
                public: ::core::clone::Clone::clone(&self.public),
                secret: ::core::clone::Clone::clone(&self.secret),
            }
        }
    }
    #[cfg(feature = "full_crypto")]
    impl TraitPair for Pair {
        type Public = Public;
        type Seed = Seed;
        type Signature = Signature;
        /// Make a new key pair from secret seed material. The slice must be 32 bytes long or it
        /// will return `None`.
        ///
        /// You should never need to use this; generate(), generate_with_phrase
        fn from_seed_slice(seed_slice: &[u8]) -> Result<Pair, SecretStringError> {
            let secret = SecretKey::from_slice(seed_slice)
                .map_err(|_| SecretStringError::InvalidSeedLength)?;
            #[cfg(feature = "std")]
            let context = SECP256K1;
            let public = PublicKey::from_secret_key(&context, &secret);
            let public = Public(public.serialize());
            Ok(Pair { public, secret })
        }
        /// Derive a child key from a series of given junctions.
        fn derive<Iter: Iterator<Item = DeriveJunction>>(
            &self,
            path: Iter,
            _seed: Option<Seed>,
        ) -> Result<(Pair, Option<Seed>), DeriveError> {
            let mut acc = self.seed();
            for j in path {
                match j {
                    DeriveJunction::Soft(_cc) => return Err(DeriveError::SoftKeyInPath),
                    DeriveJunction::Hard(cc) => acc = derive_hard_junction(&acc, &cc),
                }
            }
            Ok((Self::from_seed(&acc), Some(acc)))
        }
        /// Get the public key.
        fn public(&self) -> Public {
            self.public
        }
        /// Sign a message.
        fn sign(&self, message: &[u8]) -> Signature {
            self.sign_prehashed(&blake2_256(message))
        }
        /// Verify a signature on a message. Returns true if the signature is good.
        fn verify<M: AsRef<[u8]>>(
            sig: &Self::Signature,
            message: M,
            public: &Self::Public,
        ) -> bool {
            sig.recover(message)
                .map(|actual| actual == *public)
                .unwrap_or_default()
        }
        /// Return a vec filled with raw data.
        fn to_raw_vec(&self) -> Vec<u8> {
            self.seed().to_vec()
        }
    }
    #[cfg(feature = "full_crypto")]
    impl Pair {
        /// Get the seed for this key.
        pub fn seed(&self) -> Seed {
            self.secret.secret_bytes()
        }
        /// Exactly as `from_string` except that if no matches are found then, the the first 32
        /// characters are taken (padded with spaces as necessary) and used as the MiniSecretKey.
        #[cfg(feature = "std")]
        pub fn from_legacy_string(s: &str, password_override: Option<&str>) -> Pair {
            Self::from_string(s, password_override).unwrap_or_else(|_| {
                let mut padded_seed: Seed = [b' '; 32];
                let len = s.len().min(32);
                padded_seed[..len].copy_from_slice(&s.as_bytes()[..len]);
                Self::from_seed(&padded_seed)
            })
        }
        /// Sign a pre-hashed message
        pub fn sign_prehashed(&self, message: &[u8; 32]) -> Signature {
            let message = Message::from_slice(message).expect("Message is 32 bytes; qed");
            #[cfg(feature = "std")]
            let context = SECP256K1;
            context
                .sign_ecdsa_recoverable(&message, &self.secret)
                .into()
        }
        /// Verify a signature on a pre-hashed message. Return `true` if the signature is valid
        /// and thus matches the given `public` key.
        pub fn verify_prehashed(sig: &Signature, message: &[u8; 32], public: &Public) -> bool {
            match sig.recover_prehashed(message) {
                Some(actual) => actual == *public,
                None => false,
            }
        }
        /// Verify a signature on a message. Returns true if the signature is good.
        /// Parses Signature using parse_overflowing_slice.
        #[deprecated(note = "please use `verify` instead")]
        pub fn verify_deprecated<M: AsRef<[u8]>>(
            sig: &Signature,
            message: M,
            pubkey: &Public,
        ) -> bool {
            let message = libsecp256k1::Message::parse(&blake2_256(message.as_ref()));
            let parse_signature_overflowing = |x: [u8; 65]| {
                let sig = libsecp256k1::Signature::parse_overflowing_slice(&x[..64]).ok()?;
                let rid = libsecp256k1::RecoveryId::parse(x[64]).ok()?;
                Some((sig, rid))
            };
            let (sig, rid) = match parse_signature_overflowing(sig.0) {
                Some(sigri) => sigri,
                _ => return false,
            };
            match libsecp256k1::recover(&message, &sig, &rid) {
                Ok(actual) => pubkey.0 == actual.serialize_compressed(),
                _ => false,
            }
        }
    }
    #[cfg(feature = "full_crypto")]
    impl Drop for Pair {
        fn drop(&mut self) {
            let ptr = self.secret.as_mut_ptr();
            for off in 0..self.secret.len() {
                unsafe {
                    core::ptr::write_volatile(ptr.add(off), 0);
                }
            }
        }
    }
    impl CryptoType for Public {
        #[cfg(feature = "full_crypto")]
        type Pair = Pair;
    }
    impl CryptoType for Signature {
        #[cfg(feature = "full_crypto")]
        type Pair = Pair;
    }
    #[cfg(feature = "full_crypto")]
    impl CryptoType for Pair {
        type Pair = Pair;
    }
}
pub mod ed25519 {
    //! Simple Ed25519 API.
    #[cfg(feature = "full_crypto")]
    use sp_std::vec::Vec;
    use crate::{
        crypto::ByteArray,
        hash::{H256, H512},
    };
    use codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    #[cfg(feature = "serde")]
    use crate::crypto::Ss58Codec;
    use crate::crypto::{CryptoType, CryptoTypeId, Derive, Public as TraitPublic, UncheckedFrom};
    #[cfg(feature = "full_crypto")]
    use crate::crypto::{DeriveError, DeriveJunction, Pair as TraitPair, SecretStringError};
    #[cfg(feature = "full_crypto")]
    use core::convert::TryFrom;
    #[cfg(feature = "full_crypto")]
    use ed25519_zebra::{SigningKey, VerificationKey};
    #[cfg(feature = "serde")]
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use sp_runtime_interface::pass_by::PassByInner;
    use sp_std::ops::Deref;
    /// An identifier used to match public keys against ed25519 keys
    pub const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"ed25");
    /// A secret seed. It's not called a "secret key" because ring doesn't expose the secret keys
    /// of the key pair (yeah, dumb); as such we're forced to remember the seed manually if we
    /// will need it later (such as for HDKD).
    #[cfg(feature = "full_crypto")]
    type Seed = [u8; 32];
    /// A public key.
    pub struct Public(pub [u8; 32]);
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Public {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Public {
        #[inline]
        fn eq(&self, other: &Public) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Public {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Public {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 32]>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Public {
        #[inline]
        fn partial_cmp(&self, other: &Public) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Public {
        #[inline]
        fn cmp(&self, other: &Public) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Public {
        #[inline]
        fn clone(&self) -> Public {
            let _: ::core::clone::AssertParamIsClone<[u8; 32]>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Public {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Public {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Public {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Public {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Public({
                    let __codec_res_edqy =
                        <[u8; 32] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Public.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Public {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 32]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Public {
            type Inner = [u8; 32];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for Public {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 32]>::max_encoded_len())
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Public {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Public", "sp_core::ed25519"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["A public key."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 32]>().type_name("[u8; 32]")),
                    )
            }
        };
    };
    #[automatically_derived]
    impl ::core::hash::Hash for Public {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    /// A key pair.
    #[cfg(feature = "full_crypto")]
    pub struct Pair {
        public: VerificationKey,
        secret: SigningKey,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Pair {}
    #[automatically_derived]
    impl ::core::clone::Clone for Pair {
        #[inline]
        fn clone(&self) -> Pair {
            let _: ::core::clone::AssertParamIsClone<VerificationKey>;
            let _: ::core::clone::AssertParamIsClone<SigningKey>;
            *self
        }
    }
    impl AsRef<[u8; 32]> for Public {
        fn as_ref(&self) -> &[u8; 32] {
            &self.0
        }
    }
    impl AsRef<[u8]> for Public {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for Public {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    impl Deref for Public {
        type Target = [u8];
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl TryFrom<&[u8]> for Public {
        type Error = ();
        fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
            if data.len() != Self::LEN {
                return Err(());
            }
            let mut r = [0u8; Self::LEN];
            r.copy_from_slice(data);
            Ok(Self::unchecked_from(r))
        }
    }
    impl From<Public> for [u8; 32] {
        fn from(x: Public) -> Self {
            x.0
        }
    }
    #[cfg(feature = "full_crypto")]
    impl From<Pair> for Public {
        fn from(x: Pair) -> Self {
            x.public()
        }
    }
    impl From<Public> for H256 {
        fn from(x: Public) -> Self {
            x.0.into()
        }
    }
    #[cfg(feature = "std")]
    impl std::str::FromStr for Public {
        type Err = crate::crypto::PublicError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::from_ss58check(s)
        }
    }
    impl UncheckedFrom<[u8; 32]> for Public {
        fn unchecked_from(x: [u8; 32]) -> Self {
            Public::from_raw(x)
        }
    }
    impl UncheckedFrom<H256> for Public {
        fn unchecked_from(x: H256) -> Self {
            Public::from_h256(x)
        }
    }
    #[cfg(feature = "std")]
    impl std::fmt::Display for Public {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(&self.to_ss58check())],
            ))
        }
    }
    impl sp_std::fmt::Debug for Public {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            let s = self.to_ss58check();
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &["", " (", "...)"],
                &[
                    ::core::fmt::ArgumentV1::new_display(&crate::hexdisplay::HexDisplay::from(
                        &self.0,
                    )),
                    ::core::fmt::ArgumentV1::new_display(&&s[0..8]),
                ],
            ))
        }
    }
    #[cfg(feature = "serde")]
    impl Serialize for Public {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_ss58check())
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for Public {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Public::from_ss58check(&String::deserialize(deserializer)?).map_err(|e| {
                de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    /// A signature (a 512-bit value).
    pub struct Signature(pub [u8; 64]);
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Signature {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Signature {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Signature {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Signature({
                    let __codec_res_edqy =
                        <[u8; 64] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Signature.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for Signature {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 64]>::max_encoded_len())
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Signature {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 64]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Signature {
            type Inner = [u8; 64];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Signature {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Signature", "sp_core::ed25519"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["A signature (a 512-bit value)."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 64]>().type_name("[u8; 64]")),
                    )
            }
        };
    };
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Signature {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Signature {
        #[inline]
        fn eq(&self, other: &Signature) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Signature {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Signature {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 64]>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Signature {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl TryFrom<&[u8]> for Signature {
        type Error = ();
        fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
            if data.len() == 64 {
                let mut inner = [0u8; 64];
                inner.copy_from_slice(data);
                Ok(Signature(inner))
            } else {
                Err(())
            }
        }
    }
    #[cfg(feature = "serde")]
    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&array_bytes::bytes2hex("", self.as_ref()))
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let signature_hex = array_bytes::hex2bytes(&String::deserialize(deserializer)?)
                .map_err(|e| {
                    de::Error::custom({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &[""],
                            &[::core::fmt::ArgumentV1::new_debug(&e)],
                        ));
                        res
                    })
                })?;
            Signature::try_from(signature_hex.as_ref()).map_err(|e| {
                de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    impl Clone for Signature {
        fn clone(&self) -> Self {
            let mut r = [0u8; 64];
            r.copy_from_slice(&self.0[..]);
            Signature(r)
        }
    }
    impl From<Signature> for H512 {
        fn from(v: Signature) -> H512 {
            H512::from(v.0)
        }
    }
    impl From<Signature> for [u8; 64] {
        fn from(v: Signature) -> [u8; 64] {
            v.0
        }
    }
    impl AsRef<[u8; 64]> for Signature {
        fn as_ref(&self) -> &[u8; 64] {
            &self.0
        }
    }
    impl AsRef<[u8]> for Signature {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for Signature {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    impl sp_std::fmt::Debug for Signature {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(
                    &crate::hexdisplay::HexDisplay::from(&self.0),
                )],
            ))
        }
    }
    impl UncheckedFrom<[u8; 64]> for Signature {
        fn unchecked_from(data: [u8; 64]) -> Signature {
            Signature(data)
        }
    }
    impl Signature {
        /// A new instance from the given 64-byte `data`.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_raw(data: [u8; 64]) -> Signature {
            Signature(data)
        }
        /// A new instance from the given slice that should be 64 bytes long.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_slice(data: &[u8]) -> Option<Self> {
            if data.len() != 64 {
                return None;
            }
            let mut r = [0u8; 64];
            r.copy_from_slice(data);
            Some(Signature(r))
        }
        /// A new instance from an H512.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_h512(v: H512) -> Signature {
            Signature(v.into())
        }
    }
    impl Public {
        /// A new instance from the given 32-byte `data`.
        ///
        /// NOTE: No checking goes on to ensure this is a real public key. Only use it if
        /// you are certain that the array actually is a pubkey. GIGO!
        pub fn from_raw(data: [u8; 32]) -> Self {
            Public(data)
        }
        /// A new instance from an H256.
        ///
        /// NOTE: No checking goes on to ensure this is a real public key. Only use it if
        /// you are certain that the array actually is a pubkey. GIGO!
        pub fn from_h256(x: H256) -> Self {
            Public(x.into())
        }
        /// Return a slice filled with raw data.
        pub fn as_array_ref(&self) -> &[u8; 32] {
            self.as_ref()
        }
    }
    impl ByteArray for Public {
        const LEN: usize = 32;
    }
    impl TraitPublic for Public {}
    impl Derive for Public {}
    /// Derive a single hard junction.
    #[cfg(feature = "full_crypto")]
    fn derive_hard_junction(secret_seed: &Seed, cc: &[u8; 32]) -> Seed {
        ("Ed25519HDKD", secret_seed, cc).using_encoded(sp_core_hashing::blake2_256)
    }
    #[cfg(feature = "full_crypto")]
    impl TraitPair for Pair {
        type Public = Public;
        type Seed = Seed;
        type Signature = Signature;
        /// Make a new key pair from secret seed material. The slice must be 32 bytes long or it
        /// will return `None`.
        ///
        /// You should never need to use this; generate(), generate_with_phrase
        fn from_seed_slice(seed_slice: &[u8]) -> Result<Pair, SecretStringError> {
            let secret = SigningKey::try_from(seed_slice)
                .map_err(|_| SecretStringError::InvalidSeedLength)?;
            let public = VerificationKey::from(&secret);
            Ok(Pair { secret, public })
        }
        /// Derive a child key from a series of given junctions.
        fn derive<Iter: Iterator<Item = DeriveJunction>>(
            &self,
            path: Iter,
            _seed: Option<Seed>,
        ) -> Result<(Pair, Option<Seed>), DeriveError> {
            let mut acc = self.secret.into();
            for j in path {
                match j {
                    DeriveJunction::Soft(_cc) => return Err(DeriveError::SoftKeyInPath),
                    DeriveJunction::Hard(cc) => acc = derive_hard_junction(&acc, &cc),
                }
            }
            Ok((Self::from_seed(&acc), Some(acc)))
        }
        /// Get the public key.
        fn public(&self) -> Public {
            Public(self.public.into())
        }
        /// Sign a message.
        fn sign(&self, message: &[u8]) -> Signature {
            Signature::from_raw(self.secret.sign(message).into())
        }
        /// Verify a signature on a message.
        ///
        /// Returns true if the signature is good.
        fn verify<M: AsRef<[u8]>>(
            sig: &Self::Signature,
            message: M,
            public: &Self::Public,
        ) -> bool {
            let Ok (public) = VerificationKey :: try_from (public . as_slice ()) else { return false } ;
            let Ok (signature) = ed25519_zebra :: Signature :: try_from (sig . as_ref ()) else { return false } ;
            public.verify(&signature, message.as_ref()).is_ok()
        }
        /// Return a vec filled with raw data.
        fn to_raw_vec(&self) -> Vec<u8> {
            self.seed().to_vec()
        }
    }
    #[cfg(feature = "full_crypto")]
    impl Pair {
        /// Get the seed for this key.
        pub fn seed(&self) -> Seed {
            self.secret.into()
        }
        /// Exactly as `from_string` except that if no matches are found then, the the first 32
        /// characters are taken (padded with spaces as necessary) and used as the MiniSecretKey.
        #[cfg(feature = "std")]
        pub fn from_legacy_string(s: &str, password_override: Option<&str>) -> Pair {
            Self::from_string(s, password_override).unwrap_or_else(|_| {
                let mut padded_seed: Seed = [b' '; 32];
                let len = s.len().min(32);
                padded_seed[..len].copy_from_slice(&s.as_bytes()[..len]);
                Self::from_seed(&padded_seed)
            })
        }
    }
    impl CryptoType for Public {
        #[cfg(feature = "full_crypto")]
        type Pair = Pair;
    }
    impl CryptoType for Signature {
        #[cfg(feature = "full_crypto")]
        type Pair = Pair;
    }
    #[cfg(feature = "full_crypto")]
    impl CryptoType for Pair {
        type Pair = Pair;
    }
}
pub mod hash {
    //! A fixed hash type.
    pub use primitive_types::{H160, H256, H512};
    /// Hash conversion. Used to convert between unbound associated hash types in traits,
    /// implemented by the same hash type.
    /// Panics if used to convert between different hash types.
    pub fn convert_hash<H1: Default + AsMut<[u8]>, H2: AsRef<[u8]>>(src: &H2) -> H1 {
        let mut dest = H1::default();
        match (&dest.as_mut().len(), &src.as_ref().len()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        dest.as_mut().copy_from_slice(src.as_ref());
        dest
    }
}
#[cfg(feature = "std")]
mod hasher {
    //! Substrate Blake2b Hasher implementation
    pub mod blake2 {
        use crate::hash::H256;
        use hash256_std_hasher::Hash256StdHasher;
        use hash_db::Hasher;
        /// Concrete implementation of Hasher using Blake2b 256-bit hashes
        pub struct Blake2Hasher;
        #[automatically_derived]
        impl ::core::fmt::Debug for Blake2Hasher {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Blake2Hasher")
            }
        }
        impl Hasher for Blake2Hasher {
            type Out = H256;
            type StdHasher = Hash256StdHasher;
            const LENGTH: usize = 32;
            fn hash(x: &[u8]) -> Self::Out {
                crate::hashing::blake2_256(x).into()
            }
        }
    }
    pub mod keccak {
        use crate::hash::H256;
        use hash256_std_hasher::Hash256StdHasher;
        use hash_db::Hasher;
        /// Concrete implementation of Hasher using Keccak 256-bit hashes
        pub struct KeccakHasher;
        #[automatically_derived]
        impl ::core::fmt::Debug for KeccakHasher {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "KeccakHasher")
            }
        }
        impl Hasher for KeccakHasher {
            type Out = H256;
            type StdHasher = Hash256StdHasher;
            const LENGTH: usize = 32;
            fn hash(x: &[u8]) -> Self::Out {
                crate::hashing::keccak_256(x).into()
            }
        }
    }
}
pub mod offchain {
    //! Offchain workers types
    use crate::{OpaquePeerId, RuntimeDebug};
    use codec::{Decode, Encode};
    use scale_info::TypeInfo;
    use sp_runtime_interface::pass_by::{PassByCodec, PassByEnum, PassByInner};
    use sp_std::prelude::{Box, Vec};
    pub use crate::crypto::KeyTypeId;
    #[cfg(feature = "std")]
    pub mod storage {
        //! In-memory implementation of offchain workers database.
        use crate::offchain::OffchainStorage;
        use std::{
            collections::hash_map::{Entry, HashMap},
            iter::Iterator,
        };
        /// In-memory storage for offchain workers.
        pub struct InMemOffchainStorage {
            storage: HashMap<Vec<u8>, Vec<u8>>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InMemOffchainStorage {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "InMemOffchainStorage",
                    "storage",
                    &&self.storage,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for InMemOffchainStorage {
            #[inline]
            fn clone(&self) -> InMemOffchainStorage {
                InMemOffchainStorage {
                    storage: ::core::clone::Clone::clone(&self.storage),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for InMemOffchainStorage {
            #[inline]
            fn default() -> InMemOffchainStorage {
                InMemOffchainStorage {
                    storage: ::core::default::Default::default(),
                }
            }
        }
        impl InMemOffchainStorage {
            /// Consume the offchain storage and iterate over all key value pairs.
            pub fn into_iter(self) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> {
                self.storage.into_iter()
            }
            /// Iterate over all key value pairs by reference.
            pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &Vec<u8>)> {
                self.storage.iter()
            }
            /// Remove a key and its associated value from the offchain database.
            pub fn remove(&mut self, prefix: &[u8], key: &[u8]) {
                let key: Vec<u8> = prefix.iter().chain(key).cloned().collect();
                self.storage.remove(&key);
            }
        }
        impl OffchainStorage for InMemOffchainStorage {
            fn set(&mut self, prefix: &[u8], key: &[u8], value: &[u8]) {
                let key = prefix.iter().chain(key).cloned().collect();
                self.storage.insert(key, value.to_vec());
            }
            fn remove(&mut self, prefix: &[u8], key: &[u8]) {
                let key: Vec<u8> = prefix.iter().chain(key).cloned().collect();
                self.storage.remove(&key);
            }
            fn get(&self, prefix: &[u8], key: &[u8]) -> Option<Vec<u8>> {
                let key: Vec<u8> = prefix.iter().chain(key).cloned().collect();
                self.storage.get(&key).cloned()
            }
            fn compare_and_set(
                &mut self,
                prefix: &[u8],
                key: &[u8],
                old_value: Option<&[u8]>,
                new_value: &[u8],
            ) -> bool {
                let key = prefix.iter().chain(key).cloned().collect();
                match self.storage.entry(key) {
                    Entry::Vacant(entry) => {
                        if old_value.is_none() {
                            entry.insert(new_value.to_vec());
                            true
                        } else {
                            false
                        }
                    }
                    Entry::Occupied(ref mut entry) if Some(entry.get().as_slice()) == old_value => {
                        entry.insert(new_value.to_vec());
                        true
                    }
                    _ => false,
                }
            }
        }
    }
    #[cfg(feature = "std")]
    pub mod testing {
        //! Utilities for offchain calls testing.
        //!
        //! Namely all ExecutionExtensions that allow mocking
        //! the extra APIs.
        use crate::{
            offchain::{
                self, storage::InMemOffchainStorage, HttpError, HttpRequestId as RequestId,
                HttpRequestStatus as RequestStatus, OffchainOverlayedChange, OffchainStorage,
                OpaqueNetworkState, StorageKind, Timestamp, TransactionPool,
            },
            OpaquePeerId,
        };
        use std::{
            collections::{BTreeMap, VecDeque},
            sync::Arc,
        };
        use parking_lot::RwLock;
        /// Pending request.
        pub struct PendingRequest {
            /// HTTP method
            pub method: String,
            /// URI
            pub uri: String,
            /// Encoded Metadata
            pub meta: Vec<u8>,
            /// Request headers
            pub headers: Vec<(String, String)>,
            /// Request body
            pub body: Vec<u8>,
            /// Has the request been sent already.
            pub sent: bool,
            /// Response body
            pub response: Option<Vec<u8>>,
            /// Number of bytes already read from the response body.
            pub read: usize,
            /// Response headers
            pub response_headers: Vec<(String, String)>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PendingRequest {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "method",
                    "uri",
                    "meta",
                    "headers",
                    "body",
                    "sent",
                    "response",
                    "read",
                    "response_headers",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &&self.method,
                    &&self.uri,
                    &&self.meta,
                    &&self.headers,
                    &&self.body,
                    &&self.sent,
                    &&self.response,
                    &&self.read,
                    &&self.response_headers,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "PendingRequest",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for PendingRequest {
            #[inline]
            fn default() -> PendingRequest {
                PendingRequest {
                    method: ::core::default::Default::default(),
                    uri: ::core::default::Default::default(),
                    meta: ::core::default::Default::default(),
                    headers: ::core::default::Default::default(),
                    body: ::core::default::Default::default(),
                    sent: ::core::default::Default::default(),
                    response: ::core::default::Default::default(),
                    read: ::core::default::Default::default(),
                    response_headers: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PendingRequest {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PendingRequest {
            #[inline]
            fn eq(&self, other: &PendingRequest) -> bool {
                self.method == other.method
                    && self.uri == other.uri
                    && self.meta == other.meta
                    && self.headers == other.headers
                    && self.body == other.body
                    && self.sent == other.sent
                    && self.response == other.response
                    && self.read == other.read
                    && self.response_headers == other.response_headers
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for PendingRequest {}
        #[automatically_derived]
        impl ::core::cmp::Eq for PendingRequest {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
                let _: ::core::cmp::AssertParamIsEq<Vec<(String, String)>>;
                let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
                let _: ::core::cmp::AssertParamIsEq<bool>;
                let _: ::core::cmp::AssertParamIsEq<Option<Vec<u8>>>;
                let _: ::core::cmp::AssertParamIsEq<usize>;
                let _: ::core::cmp::AssertParamIsEq<Vec<(String, String)>>;
            }
        }
        /// Sharable "persistent" offchain storage for test.
        pub struct TestPersistentOffchainDB {
            persistent: Arc<RwLock<InMemOffchainStorage>>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TestPersistentOffchainDB {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "TestPersistentOffchainDB",
                    "persistent",
                    &&self.persistent,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TestPersistentOffchainDB {
            #[inline]
            fn clone(&self) -> TestPersistentOffchainDB {
                TestPersistentOffchainDB {
                    persistent: ::core::clone::Clone::clone(&self.persistent),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for TestPersistentOffchainDB {
            #[inline]
            fn default() -> TestPersistentOffchainDB {
                TestPersistentOffchainDB {
                    persistent: ::core::default::Default::default(),
                }
            }
        }
        impl TestPersistentOffchainDB {
            const PREFIX: &'static [u8] = b"";
            /// Create a new and empty offchain storage db for persistent items
            pub fn new() -> Self {
                Self {
                    persistent: Arc::new(RwLock::new(InMemOffchainStorage::default())),
                }
            }
            /// Apply a set of off-chain changes directly to the test backend
            pub fn apply_offchain_changes(
                &mut self,
                changes: impl Iterator<Item = ((Vec<u8>, Vec<u8>), OffchainOverlayedChange)>,
            ) {
                let mut me = self.persistent.write();
                for ((_prefix, key), value_operation) in changes {
                    match value_operation {
                        OffchainOverlayedChange::SetValue(val) => {
                            me.set(Self::PREFIX, key.as_slice(), val.as_slice())
                        }
                        OffchainOverlayedChange::Remove => me.remove(Self::PREFIX, key.as_slice()),
                    }
                }
            }
            /// Retrieve a key from the test backend.
            pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
                OffchainStorage::get(self, Self::PREFIX, key)
            }
        }
        impl OffchainStorage for TestPersistentOffchainDB {
            fn set(&mut self, prefix: &[u8], key: &[u8], value: &[u8]) {
                self.persistent.write().set(prefix, key, value);
            }
            fn remove(&mut self, prefix: &[u8], key: &[u8]) {
                self.persistent.write().remove(prefix, key);
            }
            fn get(&self, prefix: &[u8], key: &[u8]) -> Option<Vec<u8>> {
                self.persistent.read().get(prefix, key)
            }
            fn compare_and_set(
                &mut self,
                prefix: &[u8],
                key: &[u8],
                old_value: Option<&[u8]>,
                new_value: &[u8],
            ) -> bool {
                self.persistent
                    .write()
                    .compare_and_set(prefix, key, old_value, new_value)
            }
        }
        /// Internal state of the externalities.
        ///
        /// This can be used in tests to respond or assert stuff about interactions.
        pub struct OffchainState {
            /// A list of pending requests.
            pub requests: BTreeMap<RequestId, PendingRequest>,
            expected_requests: VecDeque<PendingRequest>,
            /// Persistent local storage
            pub persistent_storage: TestPersistentOffchainDB,
            /// Local storage
            pub local_storage: InMemOffchainStorage,
            /// A supposedly random seed.
            pub seed: [u8; 32],
            /// A timestamp simulating the current time.
            pub timestamp: Timestamp,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for OffchainState {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "requests",
                    "expected_requests",
                    "persistent_storage",
                    "local_storage",
                    "seed",
                    "timestamp",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &&self.requests,
                    &&self.expected_requests,
                    &&self.persistent_storage,
                    &&self.local_storage,
                    &&self.seed,
                    &&self.timestamp,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "OffchainState",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for OffchainState {
            #[inline]
            fn default() -> OffchainState {
                OffchainState {
                    requests: ::core::default::Default::default(),
                    expected_requests: ::core::default::Default::default(),
                    persistent_storage: ::core::default::Default::default(),
                    local_storage: ::core::default::Default::default(),
                    seed: ::core::default::Default::default(),
                    timestamp: ::core::default::Default::default(),
                }
            }
        }
        impl OffchainState {
            /// Asserts that pending request has been submitted and fills it's response.
            pub fn fulfill_pending_request(
                &mut self,
                id: u16,
                expected: PendingRequest,
                response: impl Into<Vec<u8>>,
                response_headers: impl IntoIterator<Item = (String, String)>,
            ) {
                match self.requests.get_mut(&RequestId(id)) {
                    None => {
                        ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                            &["Missing pending request: ", ".\n\nAll: "],
                            &[
                                ::core::fmt::ArgumentV1::new_debug(&id),
                                ::core::fmt::ArgumentV1::new_debug(&self.requests),
                            ],
                        ));
                    }
                    Some(req) => {
                        match (&*req, &expected) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    let kind = ::core::panicking::AssertKind::Eq;
                                    ::core::panicking::assert_failed(
                                        kind,
                                        &*left_val,
                                        &*right_val,
                                        ::core::option::Option::None,
                                    );
                                }
                            }
                        };
                        req.response = Some(response.into());
                        req.response_headers = response_headers.into_iter().collect();
                    }
                }
            }
            fn fulfill_expected(&mut self, id: u16) {
                if let Some(mut req) = self.expected_requests.pop_back() {
                    let response = req.response.take().expect("Response checked when added.");
                    let headers = std::mem::take(&mut req.response_headers);
                    self.fulfill_pending_request(id, req, response, headers);
                }
            }
            /// Add expected HTTP request.
            ///
            /// This method can be used to initialize expected HTTP requests and their responses
            /// before running the actual code that utilizes them (for instance before calling into
            /// runtime). Expected request has to be fulfilled before this struct is dropped,
            /// the `response` and `response_headers` fields will be used to return results to the callers.
            /// Requests are expected to be performed in the insertion order.
            pub fn expect_request(&mut self, expected: PendingRequest) {
                if expected.response.is_none() {
                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Expected request needs to have a response."],
                        &[],
                    ));
                }
                self.expected_requests.push_front(expected);
            }
        }
        impl Drop for OffchainState {
            fn drop(&mut self) {
                if !self.expected_requests.is_empty() && !std::thread::panicking() {
                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Unfulfilled expected requests: "],
                        &[::core::fmt::ArgumentV1::new_debug(&self.expected_requests)],
                    ));
                }
            }
        }
        /// Implementation of offchain externalities used for tests.
        pub struct TestOffchainExt(pub Arc<RwLock<OffchainState>>);
        #[automatically_derived]
        impl ::core::clone::Clone for TestOffchainExt {
            #[inline]
            fn clone(&self) -> TestOffchainExt {
                TestOffchainExt(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for TestOffchainExt {
            #[inline]
            fn default() -> TestOffchainExt {
                TestOffchainExt(::core::default::Default::default())
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TestOffchainExt {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "TestOffchainExt", &&self.0)
            }
        }
        impl TestOffchainExt {
            /// Create new `TestOffchainExt` and a reference to the internal state.
            pub fn new() -> (Self, Arc<RwLock<OffchainState>>) {
                let ext = Self::default();
                let state = ext.0.clone();
                (ext, state)
            }
            /// Create new `TestOffchainExt` and a reference to the internal state.
            pub fn with_offchain_db(
                offchain_db: TestPersistentOffchainDB,
            ) -> (Self, Arc<RwLock<OffchainState>>) {
                let (ext, state) = Self::new();
                ext.0.write().persistent_storage = offchain_db;
                (ext, state)
            }
        }
        impl offchain::Externalities for TestOffchainExt {
            fn is_validator(&self) -> bool {
                true
            }
            fn network_state(&self) -> Result<OpaqueNetworkState, ()> {
                Ok(OpaqueNetworkState {
                    peer_id: Default::default(),
                    external_addresses: ::alloc::vec::Vec::new(),
                })
            }
            fn timestamp(&mut self) -> Timestamp {
                self.0.read().timestamp
            }
            fn sleep_until(&mut self, deadline: Timestamp) {
                self.0.write().timestamp = deadline;
            }
            fn random_seed(&mut self) -> [u8; 32] {
                self.0.read().seed
            }
            fn http_request_start(
                &mut self,
                method: &str,
                uri: &str,
                meta: &[u8],
            ) -> Result<RequestId, ()> {
                let mut state = self.0.write();
                let id = RequestId(state.requests.len() as u16);
                state.requests.insert(
                    id,
                    PendingRequest {
                        method: method.into(),
                        uri: uri.into(),
                        meta: meta.into(),
                        ..Default::default()
                    },
                );
                Ok(id)
            }
            fn http_request_add_header(
                &mut self,
                request_id: RequestId,
                name: &str,
                value: &str,
            ) -> Result<(), ()> {
                let mut state = self.0.write();
                if let Some(req) = state.requests.get_mut(&request_id) {
                    req.headers.push((name.into(), value.into()));
                    Ok(())
                } else {
                    Err(())
                }
            }
            fn http_request_write_body(
                &mut self,
                request_id: RequestId,
                chunk: &[u8],
                _deadline: Option<Timestamp>,
            ) -> Result<(), HttpError> {
                let mut state = self.0.write();
                let sent = {
                    let req = state
                        .requests
                        .get_mut(&request_id)
                        .ok_or(HttpError::IoError)?;
                    req.body.extend(chunk);
                    if chunk.is_empty() {
                        req.sent = true;
                    }
                    req.sent
                };
                if sent {
                    state.fulfill_expected(request_id.0);
                }
                Ok(())
            }
            fn http_response_wait(
                &mut self,
                ids: &[RequestId],
                _deadline: Option<Timestamp>,
            ) -> Vec<RequestStatus> {
                let state = self.0.read();
                ids.iter()
                    .map(|id| match state.requests.get(id) {
                        Some(req) if req.response.is_none() => {
                            ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                                &["No `response` provided for request with id: "],
                                &[::core::fmt::ArgumentV1::new_debug(&id)],
                            ))
                        }
                        None => RequestStatus::Invalid,
                        _ => RequestStatus::Finished(200),
                    })
                    .collect()
            }
            fn http_response_headers(&mut self, request_id: RequestId) -> Vec<(Vec<u8>, Vec<u8>)> {
                let state = self.0.read();
                if let Some(req) = state.requests.get(&request_id) {
                    req.response_headers
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k.into_bytes(), v.into_bytes()))
                        .collect()
                } else {
                    Default::default()
                }
            }
            fn http_response_read_body(
                &mut self,
                request_id: RequestId,
                buffer: &mut [u8],
                _deadline: Option<Timestamp>,
            ) -> Result<usize, HttpError> {
                let mut state = self.0.write();
                if let Some(req) = state.requests.get_mut(&request_id) {
                    let response = req.response.as_mut().unwrap_or_else(|| {
                        ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                            &["No response provided for request: "],
                            &[::core::fmt::ArgumentV1::new_debug(&request_id)],
                        ))
                    });
                    if req.read >= response.len() {
                        state.requests.remove(&request_id);
                        Ok(0)
                    } else {
                        let read = std::cmp::min(buffer.len(), response[req.read..].len());
                        buffer[0..read].copy_from_slice(&response[req.read..req.read + read]);
                        req.read += read;
                        Ok(read)
                    }
                } else {
                    Err(HttpError::IoError)
                }
            }
            fn set_authorized_nodes(&mut self, _nodes: Vec<OpaquePeerId>, _authorized_only: bool) {
                ::core::panicking::panic("not implemented")
            }
        }
        impl offchain::DbExternalities for TestOffchainExt {
            fn local_storage_set(&mut self, kind: StorageKind, key: &[u8], value: &[u8]) {
                let mut state = self.0.write();
                match kind {
                    StorageKind::LOCAL => state.local_storage.set(b"", key, value),
                    StorageKind::PERSISTENT => state.persistent_storage.set(b"", key, value),
                };
            }
            fn local_storage_clear(&mut self, kind: StorageKind, key: &[u8]) {
                let mut state = self.0.write();
                match kind {
                    StorageKind::LOCAL => state.local_storage.remove(b"", key),
                    StorageKind::PERSISTENT => state.persistent_storage.remove(b"", key),
                };
            }
            fn local_storage_compare_and_set(
                &mut self,
                kind: StorageKind,
                key: &[u8],
                old_value: Option<&[u8]>,
                new_value: &[u8],
            ) -> bool {
                let mut state = self.0.write();
                match kind {
                    StorageKind::LOCAL => state
                        .local_storage
                        .compare_and_set(b"", key, old_value, new_value),
                    StorageKind::PERSISTENT => state
                        .persistent_storage
                        .compare_and_set(b"", key, old_value, new_value),
                }
            }
            fn local_storage_get(&mut self, kind: StorageKind, key: &[u8]) -> Option<Vec<u8>> {
                let state = self.0.read();
                match kind {
                    StorageKind::LOCAL => state
                        .local_storage
                        .get(TestPersistentOffchainDB::PREFIX, key),
                    StorageKind::PERSISTENT => state.persistent_storage.get(key),
                }
            }
        }
        /// The internal state of the fake transaction pool.
        pub struct PoolState {
            /// A vector of transactions submitted from the runtime.
            pub transactions: Vec<Vec<u8>>,
        }
        #[automatically_derived]
        impl ::core::default::Default for PoolState {
            #[inline]
            fn default() -> PoolState {
                PoolState {
                    transactions: ::core::default::Default::default(),
                }
            }
        }
        /// Implementation of transaction pool used for test.
        ///
        /// Note that this implementation does not verify correctness
        /// of sent extrinsics. It's meant to be used in contexts
        /// where an actual runtime is not known.
        ///
        /// It's advised to write integration tests that include the
        /// actual transaction pool to make sure the produced
        /// transactions are valid.
        pub struct TestTransactionPoolExt(Arc<RwLock<PoolState>>);
        #[automatically_derived]
        impl ::core::default::Default for TestTransactionPoolExt {
            #[inline]
            fn default() -> TestTransactionPoolExt {
                TestTransactionPoolExt(::core::default::Default::default())
            }
        }
        impl TestTransactionPoolExt {
            /// Create new `TestTransactionPoolExt` and a reference to the internal state.
            pub fn new() -> (Self, Arc<RwLock<PoolState>>) {
                let ext = Self::default();
                let state = ext.0.clone();
                (ext, state)
            }
        }
        impl TransactionPool for TestTransactionPoolExt {
            fn submit_transaction(&mut self, extrinsic: Vec<u8>) -> Result<(), ()> {
                self.0.write().transactions.push(extrinsic);
                Ok(())
            }
        }
    }
    /// Persistent storage prefix used by the Offchain Worker API when creating a DB key.
    pub const STORAGE_PREFIX: &[u8] = b"storage";
    /// 提供了直接操作数据库的基本操作, offchain worker的 DBExternalities是对该接口的封装
    /// Offchain DB persistent (non-fork-aware) storage.
    pub trait OffchainStorage: Clone + Send + Sync {
        /// Persist a value in storage under given key and prefix.
        fn set(&mut self, prefix: &[u8], key: &[u8], value: &[u8]);
        /// Clear a storage entry under given key and prefix.
        fn remove(&mut self, prefix: &[u8], key: &[u8]);
        /// Retrieve a value from storage under given key and prefix.
        fn get(&self, prefix: &[u8], key: &[u8]) -> Option<Vec<u8>>;
        /// Replace the value in storage if given old_value matches the current one.
        ///
        /// Returns `true` if the value has been set and false otherwise.
        fn compare_and_set(
            &mut self,
            prefix: &[u8],
            key: &[u8],
            old_value: Option<&[u8]>,
            new_value: &[u8],
        ) -> bool;
    }
    /// A type of supported crypto.
    #[repr(C)]
    pub enum StorageKind {
        /// Persistent storage is non-revertible and not fork-aware. It means that any value
        /// set by the offchain worker triggered at block `N(hash1)` is persisted even
        /// if that block is reverted as non-canonical and is available for the worker
        /// that is re-run at block `N(hash2)`.
        /// This storage can be used by offchain workers to handle forks
        /// and coordinate offchain workers running on different forks.
        PERSISTENT = 1_isize,
        /// Local storage is revertible and fork-aware. It means that any value
        /// set by the offchain worker triggered at block `N(hash1)` is reverted
        /// if that block is reverted as non-canonical and is NOT available for the worker
        /// that is re-run at block `N(hash2)`.
        LOCAL = 2_isize,
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for StorageKind {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    StorageKind::PERSISTENT => _serde::Serializer::serialize_unit_variant(
                        __serializer,
                        "StorageKind",
                        0u32,
                        "PERSISTENT",
                    ),
                    StorageKind::LOCAL => _serde::Serializer::serialize_unit_variant(
                        __serializer,
                        "StorageKind",
                        1u32,
                        "LOCAL",
                    ),
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for StorageKind {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "variant identifier")
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Err(_serde::de::Error::invalid_value(
                                _serde::de::Unexpected::Unsigned(__value),
                                &"variant index 0 <= i < 2",
                            )),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "PERSISTENT" => _serde::__private::Ok(__Field::__field0),
                            "LOCAL" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Err(_serde::de::Error::unknown_variant(
                                __value, VARIANTS,
                            )),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"PERSISTENT" => _serde::__private::Ok(__Field::__field0),
                            b"LOCAL" => _serde::__private::Ok(__Field::__field1),
                            _ => {
                                let __value = &_serde::__private::from_utf8_lossy(__value);
                                _serde::__private::Err(_serde::de::Error::unknown_variant(
                                    __value, VARIANTS,
                                ))
                            }
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<StorageKind>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = StorageKind;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "enum StorageKind")
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match match _serde::de::EnumAccess::variant(__data) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            (__Field::__field0, __variant) => {
                                match _serde::de::VariantAccess::unit_variant(__variant) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                                _serde::__private::Ok(StorageKind::PERSISTENT)
                            }
                            (__Field::__field1, __variant) => {
                                match _serde::de::VariantAccess::unit_variant(__variant) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                                _serde::__private::Ok(StorageKind::LOCAL)
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &["PERSISTENT", "LOCAL"];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "StorageKind",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<StorageKind>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for StorageKind {
        #[inline]
        fn clone(&self) -> StorageKind {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for StorageKind {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for StorageKind {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for StorageKind {
        #[inline]
        fn eq(&self, other: &StorageKind) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for StorageKind {}
    #[automatically_derived]
    impl ::core::cmp::Eq for StorageKind {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for StorageKind {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    StorageKind::PERSISTENT => {
                        __codec_dest_edqy.push_byte(1_isize as ::core::primitive::u8);
                    }
                    StorageKind::LOCAL => {
                        __codec_dest_edqy.push_byte(2_isize as ::core::primitive::u8);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for StorageKind {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for StorageKind {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy.read_byte().map_err(|e| {
                    e.chain("Could not decode `StorageKind`, failed to read variant byte")
                })? {
                    __codec_x_edqy if __codec_x_edqy == 1_isize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(StorageKind::PERSISTENT)
                    }
                    __codec_x_edqy if __codec_x_edqy == 2_isize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(StorageKind::LOCAL)
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `StorageKind`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    impl core::fmt::Debug for StorageKind {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            match self {
                Self::PERSISTENT => fmt.debug_tuple("StorageKind::PERSISTENT").finish(),
                Self::LOCAL => fmt.debug_tuple("StorageKind::LOCAL").finish(),
                _ => Ok(()),
            }
        }
    }
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for StorageKind {
            type PassBy = proc_macro_runtime_interface::pass_by::Enum<StorageKind>;
        }
        impl TryFrom<u8> for StorageKind {
            type Error = ();
            fn try_from(
                inner: u8,
            ) -> proc_macro_runtime_interface::sp_std::result::Result<Self, ()> {
                match inner {
                    0u8 => Ok(StorageKind::PERSISTENT),
                    1u8 => Ok(StorageKind::LOCAL),
                    _ => Err(()),
                }
            }
        }
        impl From<StorageKind> for u8 {
            fn from(var: StorageKind) -> u8 {
                match var {
                    StorageKind::PERSISTENT => 0u8,
                    StorageKind::LOCAL => 1u8,
                }
            }
        }
    };
    impl TryFrom<u32> for StorageKind {
        type Error = ();
        fn try_from(kind: u32) -> Result<Self, Self::Error> {
            match kind {
                e if e == u32::from(StorageKind::PERSISTENT as u8) => Ok(StorageKind::PERSISTENT),
                e if e == u32::from(StorageKind::LOCAL as u8) => Ok(StorageKind::LOCAL),
                _ => Err(()),
            }
        }
    }
    impl From<StorageKind> for u32 {
        fn from(c: StorageKind) -> Self {
            c as u8 as u32
        }
    }
    /// Opaque type for offchain http requests.
    pub struct HttpRequestId(pub u16);
    #[automatically_derived]
    impl ::core::hash::Hash for HttpRequestId {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HttpRequestId {
        #[inline]
        fn clone(&self) -> HttpRequestId {
            let _: ::core::clone::AssertParamIsClone<u16>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for HttpRequestId {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for HttpRequestId {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for HttpRequestId {
        #[inline]
        fn eq(&self, other: &HttpRequestId) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for HttpRequestId {}
    #[automatically_derived]
    impl ::core::cmp::Eq for HttpRequestId {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u16>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for HttpRequestId {
        #[inline]
        fn partial_cmp(
            &self,
            other: &HttpRequestId,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for HttpRequestId {
        #[inline]
        fn cmp(&self, other: &HttpRequestId) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    impl core::fmt::Debug for HttpRequestId {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_tuple("HttpRequestId").field(&self.0).finish()
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for HttpRequestId {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for HttpRequestId {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for HttpRequestId {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(HttpRequestId({
                    let __codec_res_edqy = <u16 as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `HttpRequestId.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for HttpRequestId {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, u16>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for HttpRequestId {
            type Inner = u16;
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    impl From<HttpRequestId> for u32 {
        fn from(c: HttpRequestId) -> Self {
            c.0 as u32
        }
    }
    /// An error enum returned by some http methods.
    #[repr(C)]
    pub enum HttpError {
        /// The requested action couldn't been completed within a deadline.
        DeadlineReached = 1_isize,
        /// There was an IO Error while processing the request.
        IoError = 2_isize,
        /// The ID of the request is invalid in this context.
        Invalid = 3_isize,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HttpError {
        #[inline]
        fn clone(&self) -> HttpError {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for HttpError {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for HttpError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for HttpError {
        #[inline]
        fn eq(&self, other: &HttpError) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for HttpError {}
    #[automatically_derived]
    impl ::core::cmp::Eq for HttpError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl core::fmt::Debug for HttpError {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            match self {
                Self::DeadlineReached => fmt.debug_tuple("HttpError::DeadlineReached").finish(),
                Self::IoError => fmt.debug_tuple("HttpError::IoError").finish(),
                Self::Invalid => fmt.debug_tuple("HttpError::Invalid").finish(),
                _ => Ok(()),
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for HttpError {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    HttpError::DeadlineReached => {
                        __codec_dest_edqy.push_byte(1_isize as ::core::primitive::u8);
                    }
                    HttpError::IoError => {
                        __codec_dest_edqy.push_byte(2_isize as ::core::primitive::u8);
                    }
                    HttpError::Invalid => {
                        __codec_dest_edqy.push_byte(3_isize as ::core::primitive::u8);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for HttpError {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for HttpError {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy.read_byte().map_err(|e| {
                    e.chain("Could not decode `HttpError`, failed to read variant byte")
                })? {
                    __codec_x_edqy if __codec_x_edqy == 1_isize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpError::DeadlineReached)
                    }
                    __codec_x_edqy if __codec_x_edqy == 2_isize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpError::IoError)
                    }
                    __codec_x_edqy if __codec_x_edqy == 3_isize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpError::Invalid)
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `HttpError`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for HttpError {
            type PassBy = proc_macro_runtime_interface::pass_by::Enum<HttpError>;
        }
        impl TryFrom<u8> for HttpError {
            type Error = ();
            fn try_from(
                inner: u8,
            ) -> proc_macro_runtime_interface::sp_std::result::Result<Self, ()> {
                match inner {
                    0u8 => Ok(HttpError::DeadlineReached),
                    1u8 => Ok(HttpError::IoError),
                    2u8 => Ok(HttpError::Invalid),
                    _ => Err(()),
                }
            }
        }
        impl From<HttpError> for u8 {
            fn from(var: HttpError) -> u8 {
                match var {
                    HttpError::DeadlineReached => 0u8,
                    HttpError::IoError => 1u8,
                    HttpError::Invalid => 2u8,
                }
            }
        }
    };
    impl TryFrom<u32> for HttpError {
        type Error = ();
        fn try_from(error: u32) -> Result<Self, Self::Error> {
            match error {
                e if e == HttpError::DeadlineReached as u8 as u32 => Ok(HttpError::DeadlineReached),
                e if e == HttpError::IoError as u8 as u32 => Ok(HttpError::IoError),
                e if e == HttpError::Invalid as u8 as u32 => Ok(HttpError::Invalid),
                _ => Err(()),
            }
        }
    }
    impl From<HttpError> for u32 {
        fn from(c: HttpError) -> Self {
            c as u8 as u32
        }
    }
    /// Status of the HTTP request
    pub enum HttpRequestStatus {
        /// Deadline was reached while we waited for this request to finish.
        ///
        /// Note the deadline is controlled by the calling part, it not necessarily
        /// means that the request has timed out.
        DeadlineReached,
        /// An error has occurred during the request, for example a timeout or the
        /// remote has closed our socket.
        ///
        /// The request is now considered destroyed. To retry the request you need
        /// to construct it again.
        IoError,
        /// The passed ID is invalid in this context.
        Invalid,
        /// The request has finished with given status code.
        Finished(u16),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HttpRequestStatus {
        #[inline]
        fn clone(&self) -> HttpRequestStatus {
            let _: ::core::clone::AssertParamIsClone<u16>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for HttpRequestStatus {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for HttpRequestStatus {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for HttpRequestStatus {
        #[inline]
        fn eq(&self, other: &HttpRequestStatus) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        HttpRequestStatus::Finished(__self_0),
                        HttpRequestStatus::Finished(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    _ => true,
                }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for HttpRequestStatus {}
    #[automatically_derived]
    impl ::core::cmp::Eq for HttpRequestStatus {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u16>;
        }
    }
    impl core::fmt::Debug for HttpRequestStatus {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            match self {
                Self::DeadlineReached => fmt
                    .debug_tuple("HttpRequestStatus::DeadlineReached")
                    .finish(),
                Self::IoError => fmt.debug_tuple("HttpRequestStatus::IoError").finish(),
                Self::Invalid => fmt.debug_tuple("HttpRequestStatus::Invalid").finish(),
                Self::Finished(ref a0) => fmt
                    .debug_tuple("HttpRequestStatus::Finished")
                    .field(a0)
                    .finish(),
                _ => Ok(()),
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for HttpRequestStatus {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    HttpRequestStatus::DeadlineReached => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                    }
                    HttpRequestStatus::IoError => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    }
                    HttpRequestStatus::Invalid => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                    }
                    HttpRequestStatus::Finished(ref aa) => {
                        __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for HttpRequestStatus {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for HttpRequestStatus {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy.read_byte().map_err(|e| {
                    e.chain("Could not decode `HttpRequestStatus`, failed to read variant byte")
                })? {
                    __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpRequestStatus::DeadlineReached)
                    }
                    __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpRequestStatus::IoError)
                    }
                    __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpRequestStatus::Invalid)
                    }
                    __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(HttpRequestStatus::Finished({
                            let __codec_res_edqy =
                                <u16 as ::codec::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `HttpRequestStatus::Finished.0`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `HttpRequestStatus`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for HttpRequestStatus {
            type PassBy = proc_macro_runtime_interface::pass_by::Codec<HttpRequestStatus>;
        }
    };
    impl From<HttpRequestStatus> for u32 {
        fn from(status: HttpRequestStatus) -> Self {
            match status {
                HttpRequestStatus::Invalid => 0,
                HttpRequestStatus::DeadlineReached => 10,
                HttpRequestStatus::IoError => 20,
                HttpRequestStatus::Finished(code) => u32::from(code),
            }
        }
    }
    impl TryFrom<u32> for HttpRequestStatus {
        type Error = ();
        fn try_from(status: u32) -> Result<Self, Self::Error> {
            match status {
                0 => Ok(HttpRequestStatus::Invalid),
                10 => Ok(HttpRequestStatus::DeadlineReached),
                20 => Ok(HttpRequestStatus::IoError),
                100..=999 => u16::try_from(status)
                    .map(HttpRequestStatus::Finished)
                    .map_err(|_| ()),
                _ => Err(()),
            }
        }
    }
    /// A blob to hold information about the local node's network state
    /// without committing to its format.
    pub struct OpaqueNetworkState {
        /// PeerId of the local node in SCALE encoded.
        pub peer_id: OpaquePeerId,
        /// List of addresses the node knows it can be reached as.
        pub external_addresses: Vec<OpaqueMultiaddr>,
    }
    #[automatically_derived]
    impl ::core::default::Default for OpaqueNetworkState {
        #[inline]
        fn default() -> OpaqueNetworkState {
            OpaqueNetworkState {
                peer_id: ::core::default::Default::default(),
                external_addresses: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for OpaqueNetworkState {
        #[inline]
        fn clone(&self) -> OpaqueNetworkState {
            OpaqueNetworkState {
                peer_id: ::core::clone::Clone::clone(&self.peer_id),
                external_addresses: ::core::clone::Clone::clone(&self.external_addresses),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for OpaqueNetworkState {}
    #[automatically_derived]
    impl ::core::cmp::Eq for OpaqueNetworkState {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<OpaquePeerId>;
            let _: ::core::cmp::AssertParamIsEq<Vec<OpaqueMultiaddr>>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for OpaqueNetworkState {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for OpaqueNetworkState {
        #[inline]
        fn eq(&self, other: &OpaqueNetworkState) -> bool {
            self.peer_id == other.peer_id && self.external_addresses == other.external_addresses
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for OpaqueNetworkState {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&self.peer_id, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.external_addresses, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for OpaqueNetworkState {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for OpaqueNetworkState {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(OpaqueNetworkState {
                    peer_id: {
                        let __codec_res_edqy =
                            <OpaquePeerId as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `OpaqueNetworkState::peer_id`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    external_addresses: {
                        let __codec_res_edqy =
                            <Vec<OpaqueMultiaddr> as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `OpaqueNetworkState::external_addresses`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                })
            }
        }
    };
    impl core::fmt::Debug for OpaqueNetworkState {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_struct("OpaqueNetworkState")
                .field("peer_id", &self.peer_id)
                .field("external_addresses", &self.external_addresses)
                .finish()
        }
    }
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for OpaqueNetworkState {
            type PassBy = proc_macro_runtime_interface::pass_by::Codec<OpaqueNetworkState>;
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for OpaqueNetworkState {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new(
                        "OpaqueNetworkState",
                        "sp_core::offchain",
                    ))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&[
                        "A blob to hold information about the local node's network state",
                        "without committing to its format.",
                    ])
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f.ty::<OpaquePeerId>()
                                    .name("peer_id")
                                    .type_name("OpaquePeerId")
                                    .docs(&["PeerId of the local node in SCALE encoded."])
                            })
                            .field(|f| {
                                f.ty::<Vec<OpaqueMultiaddr>>()
                                    .name("external_addresses")
                                    .type_name("Vec<OpaqueMultiaddr>")
                                    .docs(&[
                                        "List of addresses the node knows it can be reached as.",
                                    ])
                            }),
                    )
            }
        };
    };
    /// Simple blob to hold a `Multiaddr` without committing to its format.
    pub struct OpaqueMultiaddr(pub Vec<u8>);
    #[automatically_derived]
    impl ::core::clone::Clone for OpaqueMultiaddr {
        #[inline]
        fn clone(&self) -> OpaqueMultiaddr {
            OpaqueMultiaddr(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for OpaqueMultiaddr {}
    #[automatically_derived]
    impl ::core::cmp::Eq for OpaqueMultiaddr {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for OpaqueMultiaddr {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for OpaqueMultiaddr {
        #[inline]
        fn eq(&self, other: &OpaqueMultiaddr) -> bool {
            self.0 == other.0
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for OpaqueMultiaddr {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for OpaqueMultiaddr {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for OpaqueMultiaddr {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(OpaqueMultiaddr({
                    let __codec_res_edqy = <Vec<u8> as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `OpaqueMultiaddr.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    impl core::fmt::Debug for OpaqueMultiaddr {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_tuple("OpaqueMultiaddr").field(&self.0).finish()
        }
    }
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for OpaqueMultiaddr {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, Vec<u8>>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for OpaqueMultiaddr {
            type Inner = Vec<u8>;
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for OpaqueMultiaddr {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new(
                        "OpaqueMultiaddr",
                        "sp_core::offchain",
                    ))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["Simple blob to hold a `Multiaddr` without committing to its format."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                    )
            }
        };
    };
    impl OpaqueMultiaddr {
        /// Create new `OpaqueMultiaddr`
        pub fn new(vec: Vec<u8>) -> Self {
            OpaqueMultiaddr(vec)
        }
    }
    /// Opaque timestamp type
    pub struct Timestamp(u64);
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Timestamp {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                _serde::Serializer::serialize_newtype_struct(__serializer, "Timestamp", &self.0)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Timestamp {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Timestamp>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Timestamp;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "tuple struct Timestamp",
                        )
                    }
                    #[inline]
                    fn visit_newtype_struct<__E>(
                        self,
                        __e: __E,
                    ) -> _serde::__private::Result<Self::Value, __E::Error>
                    where
                        __E: _serde::Deserializer<'de>,
                    {
                        let __field0: u64 = match <u64 as _serde::Deserialize>::deserialize(__e) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::__private::Ok(Timestamp(__field0))
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 =
                            match match _serde::de::SeqAccess::next_element::<u64>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"tuple struct Timestamp with 1 element",
                                        ),
                                    );
                                }
                            };
                        _serde::__private::Ok(Timestamp(__field0))
                    }
                }
                _serde::Deserializer::deserialize_newtype_struct(
                    __deserializer,
                    "Timestamp",
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Timestamp>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Timestamp {
        #[inline]
        fn clone(&self) -> Timestamp {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Timestamp {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Timestamp {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Timestamp {
        #[inline]
        fn eq(&self, other: &Timestamp) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Timestamp {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Timestamp {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Timestamp {
        #[inline]
        fn cmp(&self, other: &Timestamp) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Timestamp {
        #[inline]
        fn partial_cmp(&self, other: &Timestamp) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Timestamp {
        #[inline]
        fn default() -> Timestamp {
            Timestamp(::core::default::Default::default())
        }
    }
    impl core::fmt::Debug for Timestamp {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_tuple("Timestamp").field(&self.0).finish()
        }
    }
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Timestamp {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, u64>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Timestamp {
            type Inner = u64;
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Timestamp {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Timestamp {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Timestamp {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Timestamp({
                    let __codec_res_edqy = <u64 as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Timestamp.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    /// Duration type
    pub struct Duration(u64);
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Duration {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                _serde::Serializer::serialize_newtype_struct(__serializer, "Duration", &self.0)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Duration {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Duration>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Duration;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "tuple struct Duration",
                        )
                    }
                    #[inline]
                    fn visit_newtype_struct<__E>(
                        self,
                        __e: __E,
                    ) -> _serde::__private::Result<Self::Value, __E::Error>
                    where
                        __E: _serde::Deserializer<'de>,
                    {
                        let __field0: u64 = match <u64 as _serde::Deserialize>::deserialize(__e) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::__private::Ok(Duration(__field0))
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 =
                            match match _serde::de::SeqAccess::next_element::<u64>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"tuple struct Duration with 1 element",
                                        ),
                                    );
                                }
                            };
                        _serde::__private::Ok(Duration(__field0))
                    }
                }
                _serde::Deserializer::deserialize_newtype_struct(
                    __deserializer,
                    "Duration",
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Duration>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Duration {
        #[inline]
        fn clone(&self) -> Duration {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Duration {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Duration {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Duration {
        #[inline]
        fn eq(&self, other: &Duration) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Duration {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Duration {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Duration {
        #[inline]
        fn cmp(&self, other: &Duration) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Duration {
        #[inline]
        fn partial_cmp(&self, other: &Duration) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Duration {
        #[inline]
        fn default() -> Duration {
            Duration(::core::default::Default::default())
        }
    }
    impl core::fmt::Debug for Duration {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_tuple("Duration").field(&self.0).finish()
        }
    }
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Duration {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, u64>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Duration {
            type Inner = u64;
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Duration {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Duration {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Duration {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Duration({
                    let __codec_res_edqy = <u64 as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Duration.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    impl Duration {
        /// Create new duration representing given number of milliseconds.
        pub const fn from_millis(millis: u64) -> Self {
            Duration(millis)
        }
        /// Returns number of milliseconds this Duration represents.
        pub fn millis(&self) -> u64 {
            self.0
        }
    }
    impl Timestamp {
        /// Creates new `Timestamp` given unix timestamp in milliseconds.
        pub fn from_unix_millis(millis: u64) -> Self {
            Timestamp(millis)
        }
        /// Increase the timestamp by given `Duration`.
        pub fn add(&self, duration: Duration) -> Timestamp {
            Timestamp(self.0.saturating_add(duration.0))
        }
        /// Decrease the timestamp by given `Duration`
        pub fn sub(&self, duration: Duration) -> Timestamp {
            Timestamp(self.0.saturating_sub(duration.0))
        }
        /// Returns a saturated difference (Duration) between two Timestamps.
        pub fn diff(&self, other: &Self) -> Duration {
            Duration(self.0.saturating_sub(other.0))
        }
        /// Return number of milliseconds since UNIX epoch.
        pub fn unix_millis(&self) -> u64 {
            self.0
        }
    }
    /// Execution context extra capabilities.
    pub struct Capabilities {
        bits: u32,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Capabilities {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Capabilities {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Capabilities {
        #[inline]
        fn eq(&self, other: &Capabilities) -> bool {
            self.bits == other.bits
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Capabilities {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Capabilities {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Capabilities {
        #[inline]
        fn clone(&self) -> Capabilities {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Capabilities {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Capabilities,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.bits, &other.bits)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Capabilities {
        #[inline]
        fn cmp(&self, other: &Capabilities) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.bits, &other.bits)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Capabilities {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.bits, state)
        }
    }
    impl ::bitflags::_core::fmt::Debug for Capabilities {
        fn fmt(&self, f: &mut ::bitflags::_core::fmt::Formatter) -> ::bitflags::_core::fmt::Result {
            #[allow(non_snake_case)]
            trait __BitFlags {
                #[inline]
                fn TRANSACTION_POOL(&self) -> bool {
                    false
                }
                #[inline]
                fn HTTP(&self) -> bool {
                    false
                }
                #[inline]
                fn KEYSTORE(&self) -> bool {
                    false
                }
                #[inline]
                fn RANDOMNESS(&self) -> bool {
                    false
                }
                #[inline]
                fn NETWORK_STATE(&self) -> bool {
                    false
                }
                #[inline]
                fn OFFCHAIN_DB_READ(&self) -> bool {
                    false
                }
                #[inline]
                fn OFFCHAIN_DB_WRITE(&self) -> bool {
                    false
                }
                #[inline]
                fn NODE_AUTHORIZATION(&self) -> bool {
                    false
                }
                #[inline]
                fn TIME(&self) -> bool {
                    false
                }
                #[inline]
                fn STATEMENT_STORE(&self) -> bool {
                    false
                }
            }
            #[allow(non_snake_case)]
            impl __BitFlags for Capabilities {
                #[allow(deprecated)]
                #[inline]
                fn TRANSACTION_POOL(&self) -> bool {
                    if Self::TRANSACTION_POOL.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::TRANSACTION_POOL.bits == Self::TRANSACTION_POOL.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn HTTP(&self) -> bool {
                    if Self::HTTP.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::HTTP.bits == Self::HTTP.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn KEYSTORE(&self) -> bool {
                    if Self::KEYSTORE.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::KEYSTORE.bits == Self::KEYSTORE.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn RANDOMNESS(&self) -> bool {
                    if Self::RANDOMNESS.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::RANDOMNESS.bits == Self::RANDOMNESS.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn NETWORK_STATE(&self) -> bool {
                    if Self::NETWORK_STATE.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::NETWORK_STATE.bits == Self::NETWORK_STATE.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn OFFCHAIN_DB_READ(&self) -> bool {
                    if Self::OFFCHAIN_DB_READ.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::OFFCHAIN_DB_READ.bits == Self::OFFCHAIN_DB_READ.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn OFFCHAIN_DB_WRITE(&self) -> bool {
                    if Self::OFFCHAIN_DB_WRITE.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::OFFCHAIN_DB_WRITE.bits == Self::OFFCHAIN_DB_WRITE.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn NODE_AUTHORIZATION(&self) -> bool {
                    if Self::NODE_AUTHORIZATION.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::NODE_AUTHORIZATION.bits == Self::NODE_AUTHORIZATION.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn TIME(&self) -> bool {
                    if Self::TIME.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::TIME.bits == Self::TIME.bits
                    }
                }
                #[allow(deprecated)]
                #[inline]
                fn STATEMENT_STORE(&self) -> bool {
                    if Self::STATEMENT_STORE.bits == 0 && self.bits != 0 {
                        false
                    } else {
                        self.bits & Self::STATEMENT_STORE.bits == Self::STATEMENT_STORE.bits
                    }
                }
            }
            let mut first = true;
            if <Self as __BitFlags>::TRANSACTION_POOL(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("TRANSACTION_POOL")?;
            }
            if <Self as __BitFlags>::HTTP(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("HTTP")?;
            }
            if <Self as __BitFlags>::KEYSTORE(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("KEYSTORE")?;
            }
            if <Self as __BitFlags>::RANDOMNESS(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("RANDOMNESS")?;
            }
            if <Self as __BitFlags>::NETWORK_STATE(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("NETWORK_STATE")?;
            }
            if <Self as __BitFlags>::OFFCHAIN_DB_READ(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("OFFCHAIN_DB_READ")?;
            }
            if <Self as __BitFlags>::OFFCHAIN_DB_WRITE(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("OFFCHAIN_DB_WRITE")?;
            }
            if <Self as __BitFlags>::NODE_AUTHORIZATION(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("NODE_AUTHORIZATION")?;
            }
            if <Self as __BitFlags>::TIME(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("TIME")?;
            }
            if <Self as __BitFlags>::STATEMENT_STORE(self) {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("STATEMENT_STORE")?;
            }
            let extra_bits = self.bits & !Self::all().bits();
            if extra_bits != 0 {
                if !first {
                    f.write_str(" | ")?;
                }
                first = false;
                f.write_str("0x")?;
                ::bitflags::_core::fmt::LowerHex::fmt(&extra_bits, f)?;
            }
            if first {
                f.write_str("(empty)")?;
            }
            Ok(())
        }
    }
    impl ::bitflags::_core::fmt::Binary for Capabilities {
        fn fmt(&self, f: &mut ::bitflags::_core::fmt::Formatter) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::Binary::fmt(&self.bits, f)
        }
    }
    impl ::bitflags::_core::fmt::Octal for Capabilities {
        fn fmt(&self, f: &mut ::bitflags::_core::fmt::Formatter) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::Octal::fmt(&self.bits, f)
        }
    }
    impl ::bitflags::_core::fmt::LowerHex for Capabilities {
        fn fmt(&self, f: &mut ::bitflags::_core::fmt::Formatter) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::LowerHex::fmt(&self.bits, f)
        }
    }
    impl ::bitflags::_core::fmt::UpperHex for Capabilities {
        fn fmt(&self, f: &mut ::bitflags::_core::fmt::Formatter) -> ::bitflags::_core::fmt::Result {
            ::bitflags::_core::fmt::UpperHex::fmt(&self.bits, f)
        }
    }
    #[allow(dead_code)]
    impl Capabilities {
        /// Access to transaction pool.
        pub const TRANSACTION_POOL: Self = Self {
            bits: 0b0000_0000_0001,
        };
        /// External http calls.
        pub const HTTP: Self = Self {
            bits: 0b0000_0000_0010,
        };
        /// Keystore access.
        pub const KEYSTORE: Self = Self {
            bits: 0b0000_0000_0100,
        };
        /// Randomness source.
        pub const RANDOMNESS: Self = Self {
            bits: 0b0000_0000_1000,
        };
        /// Access to opaque network state.
        pub const NETWORK_STATE: Self = Self {
            bits: 0b0000_0001_0000,
        };
        /// Access to offchain worker DB (read only).
        pub const OFFCHAIN_DB_READ: Self = Self {
            bits: 0b0000_0010_0000,
        };
        /// Access to offchain worker DB (writes).
        pub const OFFCHAIN_DB_WRITE: Self = Self {
            bits: 0b0000_0100_0000,
        };
        /// Manage the authorized nodes
        pub const NODE_AUTHORIZATION: Self = Self {
            bits: 0b0000_1000_0000,
        };
        /// Access time related functionality
        pub const TIME: Self = Self {
            bits: 0b0001_0000_0000,
        };
        /// Access the statement store.
        pub const STATEMENT_STORE: Self = Self {
            bits: 0b0010_0000_0000,
        };
        /// Returns an empty set of flags.
        #[inline]
        pub const fn empty() -> Self {
            Self { bits: 0 }
        }
        /// Returns the set containing all flags.
        #[inline]
        pub const fn all() -> Self {
            #[allow(non_snake_case)]
            trait __BitFlags {
                const TRANSACTION_POOL: u32 = 0;
                const HTTP: u32 = 0;
                const KEYSTORE: u32 = 0;
                const RANDOMNESS: u32 = 0;
                const NETWORK_STATE: u32 = 0;
                const OFFCHAIN_DB_READ: u32 = 0;
                const OFFCHAIN_DB_WRITE: u32 = 0;
                const NODE_AUTHORIZATION: u32 = 0;
                const TIME: u32 = 0;
                const STATEMENT_STORE: u32 = 0;
            }
            #[allow(non_snake_case)]
            impl __BitFlags for Capabilities {
                #[allow(deprecated)]
                const TRANSACTION_POOL: u32 = Self::TRANSACTION_POOL.bits;
                #[allow(deprecated)]
                const HTTP: u32 = Self::HTTP.bits;
                #[allow(deprecated)]
                const KEYSTORE: u32 = Self::KEYSTORE.bits;
                #[allow(deprecated)]
                const RANDOMNESS: u32 = Self::RANDOMNESS.bits;
                #[allow(deprecated)]
                const NETWORK_STATE: u32 = Self::NETWORK_STATE.bits;
                #[allow(deprecated)]
                const OFFCHAIN_DB_READ: u32 = Self::OFFCHAIN_DB_READ.bits;
                #[allow(deprecated)]
                const OFFCHAIN_DB_WRITE: u32 = Self::OFFCHAIN_DB_WRITE.bits;
                #[allow(deprecated)]
                const NODE_AUTHORIZATION: u32 = Self::NODE_AUTHORIZATION.bits;
                #[allow(deprecated)]
                const TIME: u32 = Self::TIME.bits;
                #[allow(deprecated)]
                const STATEMENT_STORE: u32 = Self::STATEMENT_STORE.bits;
            }
            Self {
                bits: <Self as __BitFlags>::TRANSACTION_POOL
                    | <Self as __BitFlags>::HTTP
                    | <Self as __BitFlags>::KEYSTORE
                    | <Self as __BitFlags>::RANDOMNESS
                    | <Self as __BitFlags>::NETWORK_STATE
                    | <Self as __BitFlags>::OFFCHAIN_DB_READ
                    | <Self as __BitFlags>::OFFCHAIN_DB_WRITE
                    | <Self as __BitFlags>::NODE_AUTHORIZATION
                    | <Self as __BitFlags>::TIME
                    | <Self as __BitFlags>::STATEMENT_STORE,
            }
        }
        /// Returns the raw value of the flags currently stored.
        #[inline]
        pub const fn bits(&self) -> u32 {
            self.bits
        }
        /// Convert from underlying bit representation, unless that
        /// representation contains bits that do not correspond to a flag.
        #[inline]
        pub const fn from_bits(bits: u32) -> ::bitflags::_core::option::Option<Self> {
            if (bits & !Self::all().bits()) == 0 {
                ::bitflags::_core::option::Option::Some(Self { bits })
            } else {
                ::bitflags::_core::option::Option::None
            }
        }
        /// Convert from underlying bit representation, dropping any bits
        /// that do not correspond to flags.
        #[inline]
        pub const fn from_bits_truncate(bits: u32) -> Self {
            Self {
                bits: bits & Self::all().bits,
            }
        }
        /// Convert from underlying bit representation, preserving all
        /// bits (even those not corresponding to a defined flag).
        ///
        /// # Safety
        ///
        /// The caller of the `bitflags!` macro can chose to allow or
        /// disallow extra bits for their bitflags type.
        ///
        /// The caller of `from_bits_unchecked()` has to ensure that
        /// all bits correspond to a defined flag or that extra bits
        /// are valid for this bitflags type.
        #[inline]
        pub const unsafe fn from_bits_unchecked(bits: u32) -> Self {
            Self { bits }
        }
        /// Returns `true` if no flags are currently stored.
        #[inline]
        pub const fn is_empty(&self) -> bool {
            self.bits() == Self::empty().bits()
        }
        /// Returns `true` if all flags are currently set.
        #[inline]
        pub const fn is_all(&self) -> bool {
            Self::all().bits | self.bits == self.bits
        }
        /// Returns `true` if there are flags common to both `self` and `other`.
        #[inline]
        pub const fn intersects(&self, other: Self) -> bool {
            !(Self {
                bits: self.bits & other.bits,
            })
            .is_empty()
        }
        /// Returns `true` if all of the flags in `other` are contained within `self`.
        #[inline]
        pub const fn contains(&self, other: Self) -> bool {
            (self.bits & other.bits) == other.bits
        }
        /// Inserts the specified flags in-place.
        #[inline]
        pub fn insert(&mut self, other: Self) {
            self.bits |= other.bits;
        }
        /// Removes the specified flags in-place.
        #[inline]
        pub fn remove(&mut self, other: Self) {
            self.bits &= !other.bits;
        }
        /// Toggles the specified flags in-place.
        #[inline]
        pub fn toggle(&mut self, other: Self) {
            self.bits ^= other.bits;
        }
        /// Inserts or removes the specified flags depending on the passed value.
        #[inline]
        pub fn set(&mut self, other: Self, value: bool) {
            if value {
                self.insert(other);
            } else {
                self.remove(other);
            }
        }
        /// Returns the intersection between the flags in `self` and
        /// `other`.
        ///
        /// Specifically, the returned set contains only the flags which are
        /// present in *both* `self` *and* `other`.
        ///
        /// This is equivalent to using the `&` operator (e.g.
        /// [`ops::BitAnd`]), as in `flags & other`.
        ///
        /// [`ops::BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
        #[inline]
        #[must_use]
        pub const fn intersection(self, other: Self) -> Self {
            Self {
                bits: self.bits & other.bits,
            }
        }
        /// Returns the union of between the flags in `self` and `other`.
        ///
        /// Specifically, the returned set contains all flags which are
        /// present in *either* `self` *or* `other`, including any which are
        /// present in both (see [`Self::symmetric_difference`] if that
        /// is undesirable).
        ///
        /// This is equivalent to using the `|` operator (e.g.
        /// [`ops::BitOr`]), as in `flags | other`.
        ///
        /// [`ops::BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
        #[inline]
        #[must_use]
        pub const fn union(self, other: Self) -> Self {
            Self {
                bits: self.bits | other.bits,
            }
        }
        /// Returns the difference between the flags in `self` and `other`.
        ///
        /// Specifically, the returned set contains all flags present in
        /// `self`, except for the ones present in `other`.
        ///
        /// It is also conceptually equivalent to the "bit-clear" operation:
        /// `flags & !other` (and this syntax is also supported).
        ///
        /// This is equivalent to using the `-` operator (e.g.
        /// [`ops::Sub`]), as in `flags - other`.
        ///
        /// [`ops::Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
        #[inline]
        #[must_use]
        pub const fn difference(self, other: Self) -> Self {
            Self {
                bits: self.bits & !other.bits,
            }
        }
        /// Returns the [symmetric difference][sym-diff] between the flags
        /// in `self` and `other`.
        ///
        /// Specifically, the returned set contains the flags present which
        /// are present in `self` or `other`, but that are not present in
        /// both. Equivalently, it contains the flags present in *exactly
        /// one* of the sets `self` and `other`.
        ///
        /// This is equivalent to using the `^` operator (e.g.
        /// [`ops::BitXor`]), as in `flags ^ other`.
        ///
        /// [sym-diff]: https://en.wikipedia.org/wiki/Symmetric_difference
        /// [`ops::BitXor`]: https://doc.rust-lang.org/std/ops/trait.BitXor.html
        #[inline]
        #[must_use]
        pub const fn symmetric_difference(self, other: Self) -> Self {
            Self {
                bits: self.bits ^ other.bits,
            }
        }
        /// Returns the complement of this set of flags.
        ///
        /// Specifically, the returned set contains all the flags which are
        /// not set in `self`, but which are allowed for this type.
        ///
        /// Alternatively, it can be thought of as the set difference
        /// between [`Self::all()`] and `self` (e.g. `Self::all() - self`)
        ///
        /// This is equivalent to using the `!` operator (e.g.
        /// [`ops::Not`]), as in `!flags`.
        ///
        /// [`Self::all()`]: Self::all
        /// [`ops::Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
        #[inline]
        #[must_use]
        pub const fn complement(self) -> Self {
            Self::from_bits_truncate(!self.bits)
        }
    }
    impl ::bitflags::_core::ops::BitOr for Capabilities {
        type Output = Self;
        /// Returns the union of the two sets of flags.
        #[inline]
        fn bitor(self, other: Capabilities) -> Self {
            Self {
                bits: self.bits | other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::BitOrAssign for Capabilities {
        /// Adds the set of flags.
        #[inline]
        fn bitor_assign(&mut self, other: Self) {
            self.bits |= other.bits;
        }
    }
    impl ::bitflags::_core::ops::BitXor for Capabilities {
        type Output = Self;
        /// Returns the left flags, but with all the right flags toggled.
        #[inline]
        fn bitxor(self, other: Self) -> Self {
            Self {
                bits: self.bits ^ other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::BitXorAssign for Capabilities {
        /// Toggles the set of flags.
        #[inline]
        fn bitxor_assign(&mut self, other: Self) {
            self.bits ^= other.bits;
        }
    }
    impl ::bitflags::_core::ops::BitAnd for Capabilities {
        type Output = Self;
        /// Returns the intersection between the two sets of flags.
        #[inline]
        fn bitand(self, other: Self) -> Self {
            Self {
                bits: self.bits & other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::BitAndAssign for Capabilities {
        /// Disables all flags disabled in the set.
        #[inline]
        fn bitand_assign(&mut self, other: Self) {
            self.bits &= other.bits;
        }
    }
    impl ::bitflags::_core::ops::Sub for Capabilities {
        type Output = Self;
        /// Returns the set difference of the two sets of flags.
        #[inline]
        fn sub(self, other: Self) -> Self {
            Self {
                bits: self.bits & !other.bits,
            }
        }
    }
    impl ::bitflags::_core::ops::SubAssign for Capabilities {
        /// Disables all flags enabled in the set.
        #[inline]
        fn sub_assign(&mut self, other: Self) {
            self.bits &= !other.bits;
        }
    }
    impl ::bitflags::_core::ops::Not for Capabilities {
        type Output = Self;
        /// Returns the complement of this set of flags.
        #[inline]
        fn not(self) -> Self {
            Self { bits: !self.bits } & Self::all()
        }
    }
    impl ::bitflags::_core::iter::Extend<Capabilities> for Capabilities {
        fn extend<T: ::bitflags::_core::iter::IntoIterator<Item = Self>>(&mut self, iterator: T) {
            for item in iterator {
                self.insert(item)
            }
        }
    }
    impl ::bitflags::_core::iter::FromIterator<Capabilities> for Capabilities {
        fn from_iter<T: ::bitflags::_core::iter::IntoIterator<Item = Self>>(iterator: T) -> Self {
            let mut result = Self::empty();
            result.extend(iterator);
            result
        }
    }
    /// An extended externalities for offchain workers.
    pub trait Externalities: Send {
        /// Returns if the local node is a potential validator.
        ///
        /// Even if this function returns `true`, it does not mean that any keys are configured
        /// and that the validator is registered in the chain.
        fn is_validator(&self) -> bool;
        /// Returns information about the local node's network state.
        fn network_state(&self) -> Result<OpaqueNetworkState, ()>;
        /// Returns current UNIX timestamp (in millis)
        fn timestamp(&mut self) -> Timestamp;
        /// Pause the execution until `deadline` is reached.
        fn sleep_until(&mut self, deadline: Timestamp);
        /// Returns a random seed.
        ///
        /// This is a truly random non deterministic seed generated by host environment.
        /// Obviously fine in the off-chain worker context.
        fn random_seed(&mut self) -> [u8; 32];
        /// Initiates a http request given HTTP verb and the URL.
        ///
        /// Meta is a future-reserved field containing additional, parity-scale-codec encoded
        /// parameters. Returns the id of newly started request.
        ///
        /// Returns an error if:
        /// - No new request identifier could be allocated.
        /// - The method or URI contain invalid characters.
        fn http_request_start(
            &mut self,
            method: &str,
            uri: &str,
            meta: &[u8],
        ) -> Result<HttpRequestId, ()>;
        /// Append header to the request.
        ///
        /// Calling this function multiple times with the same header name continues appending new
        /// headers. In other words, headers are never replaced.
        ///
        /// Returns an error if:
        /// - The request identifier is invalid.
        /// - You have called `http_request_write_body` on that request.
        /// - The name or value contain invalid characters.
        ///
        /// An error doesn't poison the request, and you can continue as if the call had never been
        /// made.
        fn http_request_add_header(
            &mut self,
            request_id: HttpRequestId,
            name: &str,
            value: &str,
        ) -> Result<(), ()>;
        /// Write a chunk of request body.
        ///
        /// Calling this function with a non-empty slice may or may not start the
        /// HTTP request. Calling this function with an empty chunks finalizes the
        /// request and always starts it. It is no longer valid to write more data
        /// afterwards.
        /// Passing `None` as deadline blocks forever.
        ///
        /// Returns an error if:
        /// - The request identifier is invalid.
        /// - `http_response_wait` has already been called on this request.
        /// - The deadline is reached.
        /// - An I/O error has happened, for example the remote has closed our request. The request is
        ///   then considered invalid.
        fn http_request_write_body(
            &mut self,
            request_id: HttpRequestId,
            chunk: &[u8],
            deadline: Option<Timestamp>,
        ) -> Result<(), HttpError>;
        /// Block and wait for the responses for given requests.
        ///
        /// Returns a vector of request statuses (the len is the same as ids).
        /// Note that if deadline is not provided the method will block indefinitely,
        /// otherwise unready responses will produce `DeadlineReached` status.
        ///
        /// If a response returns an `IoError`, it is then considered destroyed.
        /// Its id is then invalid.
        ///
        /// Passing `None` as deadline blocks forever.
        fn http_response_wait(
            &mut self,
            ids: &[HttpRequestId],
            deadline: Option<Timestamp>,
        ) -> Vec<HttpRequestStatus>;
        /// Read all response headers.
        ///
        /// Returns a vector of pairs `(HeaderKey, HeaderValue)`.
        ///
        /// Dispatches the request if it hasn't been done yet. It is no longer
        /// valid to modify the headers or write data to the request.
        ///
        /// Returns an empty list if the identifier is unknown/invalid, hasn't
        /// received a response, or has finished.
        fn http_response_headers(&mut self, request_id: HttpRequestId) -> Vec<(Vec<u8>, Vec<u8>)>;
        /// Read a chunk of body response to given buffer.
        ///
        /// Dispatches the request if it hasn't been done yet. It is no longer
        /// valid to modify the headers or write data to the request.
        ///
        /// Returns the number of bytes written or an error in case a deadline
        /// is reached or server closed the connection.
        /// Passing `None` as a deadline blocks forever.
        ///
        /// If `Ok(0)` or `Err(IoError)` is returned, the request is considered
        /// destroyed. Doing another read or getting the response's headers, for
        /// example, is then invalid.
        ///
        /// Returns an error if:
        /// - The request identifier is invalid.
        /// - The deadline is reached.
        /// - An I/O error has happened, for example the remote has closed our request. The request is
        ///   then considered invalid.
        fn http_response_read_body(
            &mut self,
            request_id: HttpRequestId,
            buffer: &mut [u8],
            deadline: Option<Timestamp>,
        ) -> Result<usize, HttpError>;
        /// Set the authorized nodes from runtime.
        ///
        /// In a permissioned network, the connections between nodes need to reach a
        /// consensus between participants.
        ///
        /// - `nodes`: a set of nodes which are allowed to connect for the local node.
        /// each one is identified with an `OpaquePeerId`, here it just use plain bytes
        /// without any encoding. Invalid `OpaquePeerId`s are silently ignored.
        /// - `authorized_only`: if true, only the authorized nodes are allowed to connect,
        /// otherwise unauthorized nodes can also be connected through other mechanism.
        fn set_authorized_nodes(&mut self, nodes: Vec<OpaquePeerId>, authorized_only: bool);
    }
    impl<T: Externalities + ?Sized> Externalities for Box<T> {
        fn is_validator(&self) -> bool {
            (&**self).is_validator()
        }
        fn network_state(&self) -> Result<OpaqueNetworkState, ()> {
            (&**self).network_state()
        }
        fn timestamp(&mut self) -> Timestamp {
            (&mut **self).timestamp()
        }
        fn sleep_until(&mut self, deadline: Timestamp) {
            (&mut **self).sleep_until(deadline)
        }
        fn random_seed(&mut self) -> [u8; 32] {
            (&mut **self).random_seed()
        }
        fn http_request_start(
            &mut self,
            method: &str,
            uri: &str,
            meta: &[u8],
        ) -> Result<HttpRequestId, ()> {
            (&mut **self).http_request_start(method, uri, meta)
        }
        fn http_request_add_header(
            &mut self,
            request_id: HttpRequestId,
            name: &str,
            value: &str,
        ) -> Result<(), ()> {
            (&mut **self).http_request_add_header(request_id, name, value)
        }
        fn http_request_write_body(
            &mut self,
            request_id: HttpRequestId,
            chunk: &[u8],
            deadline: Option<Timestamp>,
        ) -> Result<(), HttpError> {
            (&mut **self).http_request_write_body(request_id, chunk, deadline)
        }
        fn http_response_wait(
            &mut self,
            ids: &[HttpRequestId],
            deadline: Option<Timestamp>,
        ) -> Vec<HttpRequestStatus> {
            (&mut **self).http_response_wait(ids, deadline)
        }
        fn http_response_headers(&mut self, request_id: HttpRequestId) -> Vec<(Vec<u8>, Vec<u8>)> {
            (&mut **self).http_response_headers(request_id)
        }
        fn http_response_read_body(
            &mut self,
            request_id: HttpRequestId,
            buffer: &mut [u8],
            deadline: Option<Timestamp>,
        ) -> Result<usize, HttpError> {
            (&mut **self).http_response_read_body(request_id, buffer, deadline)
        }
        fn set_authorized_nodes(&mut self, nodes: Vec<OpaquePeerId>, authorized_only: bool) {
            (&mut **self).set_authorized_nodes(nodes, authorized_only)
        }
    }
    /// An `*Externalities` implementation with limited capabilities.
    pub struct LimitedExternalities<T> {
        capabilities: Capabilities,
        externalities: T,
    }
    impl<T> LimitedExternalities<T> {
        /// Create new externalities limited to given `capabilities`.
        pub fn new(capabilities: Capabilities, externalities: T) -> Self {
            Self {
                capabilities,
                externalities,
            }
        }
        /// Check if given capability is allowed.
        ///
        /// Panics in case it is not.
        fn check(&self, capability: Capabilities, name: &'static str) {
            if !self.capabilities.contains(capability) {
                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                    &["Accessing a forbidden API: ", ". No: ", " capability."],
                    &[
                        ::core::fmt::ArgumentV1::new_display(&name),
                        ::core::fmt::ArgumentV1::new_debug(&capability),
                    ],
                ));
            }
        }
    }
    impl<T: Externalities> Externalities for LimitedExternalities<T> {
        fn is_validator(&self) -> bool {
            self.check(Capabilities::KEYSTORE, "is_validator");
            self.externalities.is_validator()
        }
        fn network_state(&self) -> Result<OpaqueNetworkState, ()> {
            self.check(Capabilities::NETWORK_STATE, "network_state");
            self.externalities.network_state()
        }
        fn timestamp(&mut self) -> Timestamp {
            self.check(Capabilities::TIME, "timestamp");
            self.externalities.timestamp()
        }
        fn sleep_until(&mut self, deadline: Timestamp) {
            self.check(Capabilities::TIME, "sleep_until");
            self.externalities.sleep_until(deadline)
        }
        fn random_seed(&mut self) -> [u8; 32] {
            self.check(Capabilities::RANDOMNESS, "random_seed");
            self.externalities.random_seed()
        }
        fn http_request_start(
            &mut self,
            method: &str,
            uri: &str,
            meta: &[u8],
        ) -> Result<HttpRequestId, ()> {
            self.check(Capabilities::HTTP, "http_request_start");
            self.externalities.http_request_start(method, uri, meta)
        }
        fn http_request_add_header(
            &mut self,
            request_id: HttpRequestId,
            name: &str,
            value: &str,
        ) -> Result<(), ()> {
            self.check(Capabilities::HTTP, "http_request_add_header");
            self.externalities
                .http_request_add_header(request_id, name, value)
        }
        fn http_request_write_body(
            &mut self,
            request_id: HttpRequestId,
            chunk: &[u8],
            deadline: Option<Timestamp>,
        ) -> Result<(), HttpError> {
            self.check(Capabilities::HTTP, "http_request_write_body");
            self.externalities
                .http_request_write_body(request_id, chunk, deadline)
        }
        fn http_response_wait(
            &mut self,
            ids: &[HttpRequestId],
            deadline: Option<Timestamp>,
        ) -> Vec<HttpRequestStatus> {
            self.check(Capabilities::HTTP, "http_response_wait");
            self.externalities.http_response_wait(ids, deadline)
        }
        fn http_response_headers(&mut self, request_id: HttpRequestId) -> Vec<(Vec<u8>, Vec<u8>)> {
            self.check(Capabilities::HTTP, "http_response_headers");
            self.externalities.http_response_headers(request_id)
        }
        fn http_response_read_body(
            &mut self,
            request_id: HttpRequestId,
            buffer: &mut [u8],
            deadline: Option<Timestamp>,
        ) -> Result<usize, HttpError> {
            self.check(Capabilities::HTTP, "http_response_read_body");
            self.externalities
                .http_response_read_body(request_id, buffer, deadline)
        }
        fn set_authorized_nodes(&mut self, nodes: Vec<OpaquePeerId>, authorized_only: bool) {
            self.check(Capabilities::NODE_AUTHORIZATION, "set_authorized_nodes");
            self.externalities
                .set_authorized_nodes(nodes, authorized_only)
        }
    }
    /// The offchain worker extension that will be registered at the Substrate externalities.
    pub struct OffchainWorkerExt(pub Box<dyn Externalities>);
    impl ::sp_externalities::Extension for OffchainWorkerExt {
        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    impl std::ops::Deref for OffchainWorkerExt {
        type Target = Box<dyn Externalities>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl std::ops::DerefMut for OffchainWorkerExt {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl From<Box<dyn Externalities>> for OffchainWorkerExt {
        fn from(inner: Box<dyn Externalities>) -> Self {
            Self(inner)
        }
    }
    #[cfg(feature = "std")]
    impl OffchainWorkerExt {
        /// Create a new instance of `Self`.
        pub fn new<O: Externalities + 'static>(offchain: O) -> Self {
            Self(Box::new(offchain))
        }
    }
    /// offchain worker的DbExternalities, 是对数据库基本操作的封装
    /// 是对OffchainStorage的封装, OffchainStorage提供了一些基本接口
    /// 直接对数据库进行操作的
    /// A externalities extension for accessing the Offchain DB.
    pub trait DbExternalities: Send {
        /// Sets a value in the local storage.
        ///
        /// Note this storage is not part of the consensus, it's only accessible by
        /// offchain worker tasks running on the same machine. It _is_ persisted between runs.
        fn local_storage_set(&mut self, kind: StorageKind, key: &[u8], value: &[u8]);
        /// Removes a value in the local storage.
        ///
        /// Note this storage is not part of the consensus, it's only accessible by
        /// offchain worker tasks running on the same machine. It _is_ persisted between runs.
        fn local_storage_clear(&mut self, kind: StorageKind, key: &[u8]);
        /// Sets a value in the local storage if it matches current value.
        ///
        /// Since multiple offchain workers may be running concurrently, to prevent
        /// data races use CAS to coordinate between them.
        ///
        /// Returns `true` if the value has been set, `false` otherwise.
        ///
        /// Note this storage is not part of the consensus, it's only accessible by
        /// offchain worker tasks running on the same machine. It _is_ persisted between runs.
        fn local_storage_compare_and_set(
            &mut self,
            kind: StorageKind,
            key: &[u8],
            old_value: Option<&[u8]>,
            new_value: &[u8],
        ) -> bool;
        /// Gets a value from the local storage.
        ///
        /// If the value does not exist in the storage `None` will be returned.
        /// Note this storage is not part of the consensus, it's only accessible by
        /// offchain worker tasks running on the same machine. It _is_ persisted between runs.
        fn local_storage_get(&mut self, kind: StorageKind, key: &[u8]) -> Option<Vec<u8>>;
    }
    impl<T: DbExternalities + ?Sized> DbExternalities for Box<T> {
        fn local_storage_set(&mut self, kind: StorageKind, key: &[u8], value: &[u8]) {
            (&mut **self).local_storage_set(kind, key, value)
        }
        fn local_storage_clear(&mut self, kind: StorageKind, key: &[u8]) {
            (&mut **self).local_storage_clear(kind, key)
        }
        fn local_storage_compare_and_set(
            &mut self,
            kind: StorageKind,
            key: &[u8],
            old_value: Option<&[u8]>,
            new_value: &[u8],
        ) -> bool {
            (&mut **self).local_storage_compare_and_set(kind, key, old_value, new_value)
        }
        fn local_storage_get(&mut self, kind: StorageKind, key: &[u8]) -> Option<Vec<u8>> {
            (&mut **self).local_storage_get(kind, key)
        }
    }
    impl<T: DbExternalities> DbExternalities for LimitedExternalities<T> {
        fn local_storage_set(&mut self, kind: StorageKind, key: &[u8], value: &[u8]) {
            self.check(Capabilities::OFFCHAIN_DB_WRITE, "local_storage_set");
            self.externalities.local_storage_set(kind, key, value)
        }
        fn local_storage_clear(&mut self, kind: StorageKind, key: &[u8]) {
            self.check(Capabilities::OFFCHAIN_DB_WRITE, "local_storage_clear");
            self.externalities.local_storage_clear(kind, key)
        }
        fn local_storage_compare_and_set(
            &mut self,
            kind: StorageKind,
            key: &[u8],
            old_value: Option<&[u8]>,
            new_value: &[u8],
        ) -> bool {
            self.check(
                Capabilities::OFFCHAIN_DB_WRITE,
                "local_storage_compare_and_set",
            );
            self.externalities
                .local_storage_compare_and_set(kind, key, old_value, new_value)
        }
        fn local_storage_get(&mut self, kind: StorageKind, key: &[u8]) -> Option<Vec<u8>> {
            self.check(Capabilities::OFFCHAIN_DB_READ, "local_storage_get");
            self.externalities.local_storage_get(kind, key)
        }
    }
    /// The offchain database extension that will be registered at the Substrate externalities.
    pub struct OffchainDbExt(pub Box<dyn DbExternalities>);
    impl ::sp_externalities::Extension for OffchainDbExt {
        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    impl std::ops::Deref for OffchainDbExt {
        type Target = Box<dyn DbExternalities>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl std::ops::DerefMut for OffchainDbExt {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl From<Box<dyn DbExternalities>> for OffchainDbExt {
        fn from(inner: Box<dyn DbExternalities>) -> Self {
            Self(inner)
        }
    }
    #[cfg(feature = "std")]
    impl OffchainDbExt {
        /// Create a new instance of `OffchainDbExt`.
        pub fn new<O: DbExternalities + 'static>(offchain: O) -> Self {
            Self(Box::new(offchain))
        }
    }
    /// Abstraction over transaction pool.
    ///
    /// This trait is currently used within the `ExternalitiesExtension`
    /// to provide offchain calls with access to the transaction pool without
    /// tight coupling with any pool implementation.
    #[cfg(feature = "std")]
    pub trait TransactionPool {
        /// Submit transaction.
        ///
        /// The transaction will end up in the pool and be propagated to others.
        fn submit_transaction(&mut self, extrinsic: Vec<u8>) -> Result<(), ()>;
    }
    /// An externalities extension to submit transactions to the pool.
    pub struct TransactionPoolExt(pub Box<dyn TransactionPool + Send>);
    impl ::sp_externalities::Extension for TransactionPoolExt {
        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    impl std::ops::Deref for TransactionPoolExt {
        type Target = Box<dyn TransactionPool + Send>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl std::ops::DerefMut for TransactionPoolExt {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl From<Box<dyn TransactionPool + Send>> for TransactionPoolExt {
        fn from(inner: Box<dyn TransactionPool + Send>) -> Self {
            Self(inner)
        }
    }
    #[cfg(feature = "std")]
    impl TransactionPoolExt {
        /// Create a new instance of `TransactionPoolExt`.
        pub fn new<O: TransactionPool + Send + 'static>(pool: O) -> Self {
            Self(Box::new(pool))
        }
    }
    /// Change to be applied to the offchain worker db in regards to a key.
    pub enum OffchainOverlayedChange {
        /// Remove the data associated with the key
        Remove,
        /// Overwrite the value of an associated key
        SetValue(Vec<u8>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for OffchainOverlayedChange {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                OffchainOverlayedChange::Remove => ::core::fmt::Formatter::write_str(f, "Remove"),
                OffchainOverlayedChange::SetValue(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "SetValue", &__self_0)
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for OffchainOverlayedChange {
        #[inline]
        fn clone(&self) -> OffchainOverlayedChange {
            match self {
                OffchainOverlayedChange::Remove => OffchainOverlayedChange::Remove,
                OffchainOverlayedChange::SetValue(__self_0) => {
                    OffchainOverlayedChange::SetValue(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for OffchainOverlayedChange {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state);
            match self {
                OffchainOverlayedChange::SetValue(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
                _ => {}
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for OffchainOverlayedChange {}
    #[automatically_derived]
    impl ::core::cmp::Eq for OffchainOverlayedChange {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for OffchainOverlayedChange {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for OffchainOverlayedChange {
        #[inline]
        fn eq(&self, other: &OffchainOverlayedChange) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        OffchainOverlayedChange::SetValue(__self_0),
                        OffchainOverlayedChange::SetValue(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    _ => true,
                }
        }
    }
}
pub mod sr25519 {
    //! Simple sr25519 (Schnorr-Ristretto) API.
    //!
    //! Note: `CHAIN_CODE_LENGTH` must be equal to `crate::crypto::JUNCTION_ID_LEN`
    //! for this to work.
    #[cfg(any(feature = "full_crypto", feature = "serde"))]
    use crate::crypto::DeriveJunction;
    #[cfg(feature = "serde")]
    use crate::crypto::Ss58Codec;
    #[cfg(feature = "full_crypto")]
    use crate::crypto::{DeriveError, Pair as TraitPair, SecretStringError};
    #[cfg(feature = "full_crypto")]
    use schnorrkel::{
        derive::CHAIN_CODE_LENGTH, signing_context, ExpansionMode, Keypair, MiniSecretKey,
        SecretKey,
    };
    #[cfg(any(feature = "full_crypto", feature = "serde"))]
    use schnorrkel::{
        derive::{ChainCode, Derivation},
        PublicKey,
    };
    use sp_std::vec::Vec;
    use crate::{
        crypto::{
            ByteArray, CryptoType, CryptoTypeId, Derive, Public as TraitPublic, UncheckedFrom,
        },
        hash::{H256, H512},
    };
    use codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_std::ops::Deref;
    #[cfg(feature = "full_crypto")]
    use schnorrkel::keys::{MINI_SECRET_KEY_LENGTH, SECRET_KEY_LENGTH};
    #[cfg(feature = "serde")]
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use sp_runtime_interface::pass_by::PassByInner;
    #[cfg(feature = "full_crypto")]
    const SIGNING_CTX: &[u8] = b"substrate";
    /// An identifier used to match public keys against sr25519 keys
    pub const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"sr25");
    /// An Schnorrkel/Ristretto x25519 ("sr25519") public key.
    pub struct Public(pub [u8; 32]);
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Public {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Public {
        #[inline]
        fn eq(&self, other: &Public) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Public {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Public {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 32]>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Public {
        #[inline]
        fn partial_cmp(&self, other: &Public) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Public {
        #[inline]
        fn cmp(&self, other: &Public) -> ::core::cmp::Ordering {
            ::core::cmp::Ord::cmp(&self.0, &other.0)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Public {
        #[inline]
        fn clone(&self) -> Public {
            let _: ::core::clone::AssertParamIsClone<[u8; 32]>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Public {}
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Public {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Public {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Public {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Public({
                    let __codec_res_edqy =
                        <[u8; 32] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Public.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Public {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 32]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Public {
            type Inner = [u8; 32];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for Public {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 32]>::max_encoded_len())
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Public {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Public", "sp_core::sr25519"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["An Schnorrkel/Ristretto x25519 (\"sr25519\") public key."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 32]>().type_name("[u8; 32]")),
                    )
            }
        };
    };
    #[automatically_derived]
    impl ::core::hash::Hash for Public {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    /// An Schnorrkel/Ristretto x25519 ("sr25519") key pair.
    #[cfg(feature = "full_crypto")]
    pub struct Pair(Keypair);
    #[cfg(feature = "full_crypto")]
    impl Clone for Pair {
        fn clone(&self) -> Self {
            Pair(schnorrkel::Keypair {
                public: self.0.public,
                secret: schnorrkel::SecretKey::from_bytes(&self.0.secret.to_bytes()[..])
                    .expect("key is always the correct size; qed"),
            })
        }
    }
    impl AsRef<[u8; 32]> for Public {
        fn as_ref(&self) -> &[u8; 32] {
            &self.0
        }
    }
    impl AsRef<[u8]> for Public {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for Public {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    impl Deref for Public {
        type Target = [u8];
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl From<Public> for [u8; 32] {
        fn from(x: Public) -> [u8; 32] {
            x.0
        }
    }
    impl From<Public> for H256 {
        fn from(x: Public) -> H256 {
            x.0.into()
        }
    }
    #[cfg(feature = "std")]
    impl std::str::FromStr for Public {
        type Err = crate::crypto::PublicError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::from_ss58check(s)
        }
    }
    impl TryFrom<&[u8]> for Public {
        type Error = ();
        fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
            if data.len() != Self::LEN {
                return Err(());
            }
            let mut r = [0u8; 32];
            r.copy_from_slice(data);
            Ok(Self::unchecked_from(r))
        }
    }
    impl UncheckedFrom<[u8; 32]> for Public {
        fn unchecked_from(x: [u8; 32]) -> Self {
            Public::from_raw(x)
        }
    }
    impl UncheckedFrom<H256> for Public {
        fn unchecked_from(x: H256) -> Self {
            Public::from_h256(x)
        }
    }
    #[cfg(feature = "std")]
    impl std::fmt::Display for Public {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(&self.to_ss58check())],
            ))
        }
    }
    impl sp_std::fmt::Debug for Public {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            let s = self.to_ss58check();
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &["", " (", "...)"],
                &[
                    ::core::fmt::ArgumentV1::new_display(&crate::hexdisplay::HexDisplay::from(
                        &self.0,
                    )),
                    ::core::fmt::ArgumentV1::new_display(&&s[0..8]),
                ],
            ))
        }
    }
    #[cfg(feature = "serde")]
    impl Serialize for Public {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_ss58check())
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for Public {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Public::from_ss58check(&String::deserialize(deserializer)?).map_err(|e| {
                de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    /// An Schnorrkel/Ristretto x25519 ("sr25519") signature.
    pub struct Signature(pub [u8; 64]);
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Signature {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
            }
            fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
                ::codec::Encode::encode(&&self.0)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::codec::Encode::using_encoded(&&self.0, f)
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Signature {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Signature {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Signature({
                    let __codec_res_edqy =
                        <[u8; 64] as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `Signature.0`"),
                            )
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }))
            }
        }
    };
    const _: () = {
        impl ::codec::MaxEncodedLen for Signature {
            fn max_encoded_len() -> ::core::primitive::usize {
                0_usize.saturating_add(<[u8; 64]>::max_encoded_len())
            }
        }
    };
    const _: () = {
        #[doc(hidden)]
        extern crate sp_runtime_interface as proc_macro_runtime_interface;
        impl proc_macro_runtime_interface::pass_by::PassBy for Signature {
            type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, [u8; 64]>;
        }
        impl proc_macro_runtime_interface::pass_by::PassByInner for Signature {
            type Inner = [u8; 64];
            fn into_inner(self) -> Self::Inner {
                self.0
            }
            fn inner(&self) -> &Self::Inner {
                &self.0
            }
            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Signature {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Signature", "sp_core::sr25519"))
                    .type_params(::alloc::vec::Vec::new())
                    .docs(&["An Schnorrkel/Ristretto x25519 (\"sr25519\") signature."])
                    .composite(
                        ::scale_info::build::Fields::unnamed()
                            .field(|f| f.ty::<[u8; 64]>().type_name("[u8; 64]")),
                    )
            }
        };
    };
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Signature {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Signature {
        #[inline]
        fn eq(&self, other: &Signature) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Signature {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Signature {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<[u8; 64]>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Signature {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl TryFrom<&[u8]> for Signature {
        type Error = ();
        fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
            if data.len() == 64 {
                let mut inner = [0u8; 64];
                inner.copy_from_slice(data);
                Ok(Signature(inner))
            } else {
                Err(())
            }
        }
    }
    #[cfg(feature = "serde")]
    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&array_bytes::bytes2hex("", self.as_ref()))
        }
    }
    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let signature_hex = array_bytes::hex2bytes(&String::deserialize(deserializer)?)
                .map_err(|e| {
                    de::Error::custom({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &[""],
                            &[::core::fmt::ArgumentV1::new_debug(&e)],
                        ));
                        res
                    })
                })?;
            Signature::try_from(signature_hex.as_ref()).map_err(|e| {
                de::Error::custom({
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &[""],
                        &[::core::fmt::ArgumentV1::new_debug(&e)],
                    ));
                    res
                })
            })
        }
    }
    impl Clone for Signature {
        fn clone(&self) -> Self {
            let mut r = [0u8; 64];
            r.copy_from_slice(&self.0[..]);
            Signature(r)
        }
    }
    impl From<Signature> for [u8; 64] {
        fn from(v: Signature) -> [u8; 64] {
            v.0
        }
    }
    impl From<Signature> for H512 {
        fn from(v: Signature) -> H512 {
            H512::from(v.0)
        }
    }
    impl AsRef<[u8; 64]> for Signature {
        fn as_ref(&self) -> &[u8; 64] {
            &self.0
        }
    }
    impl AsRef<[u8]> for Signature {
        fn as_ref(&self) -> &[u8] {
            &self.0[..]
        }
    }
    impl AsMut<[u8]> for Signature {
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0[..]
        }
    }
    #[cfg(feature = "full_crypto")]
    impl From<schnorrkel::Signature> for Signature {
        fn from(s: schnorrkel::Signature) -> Signature {
            Signature(s.to_bytes())
        }
    }
    impl sp_std::fmt::Debug for Signature {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &[""],
                &[::core::fmt::ArgumentV1::new_display(
                    &crate::hexdisplay::HexDisplay::from(&self.0),
                )],
            ))
        }
    }
    impl UncheckedFrom<[u8; 64]> for Signature {
        fn unchecked_from(data: [u8; 64]) -> Signature {
            Signature(data)
        }
    }
    impl Signature {
        /// A new instance from the given 64-byte `data`.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use
        /// it if you are certain that the array actually is a signature, or if you
        /// immediately verify the signature.  All functions that verify signatures
        /// will fail if the `Signature` is not actually a valid signature.
        pub fn from_raw(data: [u8; 64]) -> Signature {
            Signature(data)
        }
        /// A new instance from the given slice that should be 64 bytes long.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_slice(data: &[u8]) -> Option<Self> {
            if data.len() != 64 {
                return None;
            }
            let mut r = [0u8; 64];
            r.copy_from_slice(data);
            Some(Signature(r))
        }
        /// A new instance from an H512.
        ///
        /// NOTE: No checking goes on to ensure this is a real signature. Only use it if
        /// you are certain that the array actually is a signature. GIGO!
        pub fn from_h512(v: H512) -> Signature {
            Signature(v.into())
        }
    }
    impl Derive for Public {
        /// Derive a child key from a series of given junctions.
        ///
        /// `None` if there are any hard junctions in there.
        #[cfg(feature = "serde")]
        fn derive<Iter: Iterator<Item = DeriveJunction>>(&self, path: Iter) -> Option<Public> {
            let mut acc = PublicKey::from_bytes(self.as_ref()).ok()?;
            for j in path {
                match j {
                    DeriveJunction::Soft(cc) => acc = acc.derived_key_simple(ChainCode(cc), &[]).0,
                    DeriveJunction::Hard(_cc) => return None,
                }
            }
            Some(Self(acc.to_bytes()))
        }
    }
    impl Public {
        /// A new instance from the given 32-byte `data`.
        ///
        /// NOTE: No checking goes on to ensure this is a real public key. Only use it if
        /// you are certain that the array actually is a pubkey. GIGO!
        pub fn from_raw(data: [u8; 32]) -> Self {
            Public(data)
        }
        /// A new instance from an H256.
        ///
        /// NOTE: No checking goes on to ensure this is a real public key. Only use it if
        /// you are certain that the array actually is a pubkey. GIGO!
        pub fn from_h256(x: H256) -> Self {
            Public(x.into())
        }
        /// Return a slice filled with raw data.
        pub fn as_array_ref(&self) -> &[u8; 32] {
            self.as_ref()
        }
    }
    impl ByteArray for Public {
        const LEN: usize = 32;
    }
    impl TraitPublic for Public {}
    #[cfg(feature = "std")]
    impl From<MiniSecretKey> for Pair {
        fn from(sec: MiniSecretKey) -> Pair {
            Pair(sec.expand_to_keypair(ExpansionMode::Ed25519))
        }
    }
    #[cfg(feature = "std")]
    impl From<SecretKey> for Pair {
        fn from(sec: SecretKey) -> Pair {
            Pair(Keypair::from(sec))
        }
    }
    #[cfg(feature = "full_crypto")]
    impl From<schnorrkel::Keypair> for Pair {
        fn from(p: schnorrkel::Keypair) -> Pair {
            Pair(p)
        }
    }
    #[cfg(feature = "full_crypto")]
    impl From<Pair> for schnorrkel::Keypair {
        fn from(p: Pair) -> schnorrkel::Keypair {
            p.0
        }
    }
    #[cfg(feature = "full_crypto")]
    impl AsRef<schnorrkel::Keypair> for Pair {
        fn as_ref(&self) -> &schnorrkel::Keypair {
            &self.0
        }
    }
    /// Derive a single hard junction.
    #[cfg(feature = "full_crypto")]
    fn derive_hard_junction(secret: &SecretKey, cc: &[u8; CHAIN_CODE_LENGTH]) -> MiniSecretKey {
        secret
            .hard_derive_mini_secret_key(Some(ChainCode(*cc)), b"")
            .0
    }
    /// The raw secret seed, which can be used to recreate the `Pair`.
    #[cfg(feature = "full_crypto")]
    type Seed = [u8; MINI_SECRET_KEY_LENGTH];
    #[cfg(feature = "full_crypto")]
    impl TraitPair for Pair {
        type Public = Public;
        type Seed = Seed;
        type Signature = Signature;
        /// Get the public key.
        fn public(&self) -> Public {
            let mut pk = [0u8; 32];
            pk.copy_from_slice(&self.0.public.to_bytes());
            Public(pk)
        }
        /// Make a new key pair from raw secret seed material.
        ///
        /// This is generated using schnorrkel's Mini-Secret-Keys.
        ///
        /// A `MiniSecretKey` is literally what Ed25519 calls a `SecretKey`, which is just 32 random
        /// bytes.
        fn from_seed_slice(seed: &[u8]) -> Result<Pair, SecretStringError> {
            match seed.len() {
                MINI_SECRET_KEY_LENGTH => Ok(Pair(
                    MiniSecretKey::from_bytes(seed)
                        .map_err(|_| SecretStringError::InvalidSeed)?
                        .expand_to_keypair(ExpansionMode::Ed25519),
                )),
                SECRET_KEY_LENGTH => Ok(Pair(
                    SecretKey::from_bytes(seed)
                        .map_err(|_| SecretStringError::InvalidSeed)?
                        .to_keypair(),
                )),
                _ => Err(SecretStringError::InvalidSeedLength),
            }
        }
        fn derive<Iter: Iterator<Item = DeriveJunction>>(
            &self,
            path: Iter,
            seed: Option<Seed>,
        ) -> Result<(Pair, Option<Seed>), DeriveError> {
            let seed = seed
                .and_then(|s| MiniSecretKey::from_bytes(&s).ok())
                .filter(|msk| msk.expand(ExpansionMode::Ed25519) == self.0.secret);
            let init = self.0.secret.clone();
            let (result, seed) =
                path.fold((init, seed), |(acc, acc_seed), j| match (j, acc_seed) {
                    (DeriveJunction::Soft(cc), _) => {
                        (acc.derived_key_simple(ChainCode(cc), &[]).0, None)
                    }
                    (DeriveJunction::Hard(cc), maybe_seed) => {
                        let seed = derive_hard_junction(&acc, &cc);
                        (
                            seed.expand(ExpansionMode::Ed25519),
                            maybe_seed.map(|_| seed),
                        )
                    }
                });
            Ok((
                Self(result.into()),
                seed.map(|s| MiniSecretKey::to_bytes(&s)),
            ))
        }
        fn sign(&self, message: &[u8]) -> Signature {
            let context = signing_context(SIGNING_CTX);
            self.0.sign(context.bytes(message)).into()
        }
        fn verify<M: AsRef<[u8]>>(
            sig: &Self::Signature,
            message: M,
            pubkey: &Self::Public,
        ) -> bool {
            let Ok (signature) = schnorrkel :: Signature :: from_bytes (sig . as_ref ()) else { return false } ;
            let Ok (public) = PublicKey :: from_bytes (pubkey . as_ref ()) else { return false } ;
            public
                .verify_simple(SIGNING_CTX, message.as_ref(), &signature)
                .is_ok()
        }
        fn to_raw_vec(&self) -> Vec<u8> {
            self.0.secret.to_bytes().to_vec()
        }
    }
    #[cfg(feature = "std")]
    impl Pair {
        /// Verify a signature on a message. Returns `true` if the signature is good.
        /// Supports old 0.1.1 deprecated signatures and should be used only for backward
        /// compatibility.
        pub fn verify_deprecated<M: AsRef<[u8]>>(
            sig: &Signature,
            message: M,
            pubkey: &Public,
        ) -> bool {
            match PublicKey::from_bytes(pubkey.as_ref()) {
                Ok(pk) => pk
                    .verify_simple_preaudit_deprecated(SIGNING_CTX, message.as_ref(), &sig.0[..])
                    .is_ok(),
                Err(_) => false,
            }
        }
    }
    impl CryptoType for Public {
        #[cfg(feature = "full_crypto")]
        type Pair = Pair;
    }
    impl CryptoType for Signature {
        #[cfg(feature = "full_crypto")]
        type Pair = Pair;
    }
    #[cfg(feature = "full_crypto")]
    impl CryptoType for Pair {
        type Pair = Pair;
    }
    /// Schnorrkel VRF related types and operations.
    pub mod vrf {
        use super::*;
        #[cfg(feature = "full_crypto")]
        use crate::crypto::VrfSecret;
        use crate::crypto::{VrfCrypto, VrfPublic};
        use schnorrkel::{
            errors::MultiSignatureStage,
            vrf::{VRF_OUTPUT_LENGTH, VRF_PROOF_LENGTH},
            SignatureError,
        };
        const DEFAULT_EXTRA_DATA_LABEL: &[u8] = b"VRF";
        /// Transcript ready to be used for VRF related operations.
        pub struct VrfTranscript(pub merlin::Transcript);
        #[automatically_derived]
        impl ::core::clone::Clone for VrfTranscript {
            #[inline]
            fn clone(&self) -> VrfTranscript {
                VrfTranscript(::core::clone::Clone::clone(&self.0))
            }
        }
        impl VrfTranscript {
            /// Build a new transcript instance.
            ///
            /// Each `data` element is a tuple `(domain, message)` composing the transcipt.
            pub fn new(label: &'static [u8], data: &[(&'static [u8], &[u8])]) -> Self {
                let mut transcript = merlin::Transcript::new(label);
                data.iter()
                    .for_each(|(l, b)| transcript.append_message(l, b));
                VrfTranscript(transcript)
            }
            /// Map transcript to `VrfSignData`.
            pub fn into_sign_data(self) -> VrfSignData {
                self.into()
            }
        }
        /// VRF input.
        ///
        /// Technically a transcript used by the Fiat-Shamir transform.
        pub type VrfInput = VrfTranscript;
        /// VRF input ready to be used for VRF sign and verify operations.
        pub struct VrfSignData {
            /// Transcript data contributing to VRF output.
            pub(super) transcript: VrfTranscript,
            /// Extra transcript data to be signed by the VRF.
            pub(super) extra: Option<VrfTranscript>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for VrfSignData {
            #[inline]
            fn clone(&self) -> VrfSignData {
                VrfSignData {
                    transcript: ::core::clone::Clone::clone(&self.transcript),
                    extra: ::core::clone::Clone::clone(&self.extra),
                }
            }
        }
        impl From<VrfInput> for VrfSignData {
            fn from(transcript: VrfInput) -> Self {
                VrfSignData {
                    transcript,
                    extra: None,
                }
            }
        }
        impl AsRef<VrfInput> for VrfSignData {
            fn as_ref(&self) -> &VrfInput {
                &self.transcript
            }
        }
        impl VrfSignData {
            /// Build a new instance ready to be used for VRF signer and verifier.
            ///
            /// `input` will contribute to the VRF output bytes.
            pub fn new(input: VrfTranscript) -> Self {
                input.into()
            }
            /// Add some extra data to be signed.
            ///
            /// `extra` will not contribute to the VRF output bytes.
            pub fn with_extra(mut self, extra: VrfTranscript) -> Self {
                self.extra = Some(extra);
                self
            }
        }
        /// VRF signature data
        pub struct VrfSignature {
            /// VRF output.
            pub output: VrfOutput,
            /// VRF proof.
            pub proof: VrfProof,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for VrfSignature {
            #[inline]
            fn clone(&self) -> VrfSignature {
                VrfSignature {
                    output: ::core::clone::Clone::clone(&self.output),
                    proof: ::core::clone::Clone::clone(&self.proof),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for VrfSignature {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "VrfSignature",
                    "output",
                    &&self.output,
                    "proof",
                    &&self.proof,
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for VrfSignature {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for VrfSignature {
            #[inline]
            fn eq(&self, other: &VrfSignature) -> bool {
                self.output == other.output && self.proof == other.proof
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for VrfSignature {}
        #[automatically_derived]
        impl ::core::cmp::Eq for VrfSignature {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<VrfOutput>;
                let _: ::core::cmp::AssertParamIsEq<VrfProof>;
            }
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::codec::Encode for VrfSignature {
                fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                    &self,
                    __codec_dest_edqy: &mut __CodecOutputEdqy,
                ) {
                    ::codec::Encode::encode_to(&self.output, __codec_dest_edqy);
                    ::codec::Encode::encode_to(&self.proof, __codec_dest_edqy);
                }
            }
            #[automatically_derived]
            impl ::codec::EncodeLike for VrfSignature {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::codec::Decode for VrfSignature {
                fn decode<__CodecInputEdqy: ::codec::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::codec::Error> {
                    ::core::result::Result::Ok(VrfSignature {
                        output: {
                            let __codec_res_edqy =
                                <VrfOutput as ::codec::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `VrfSignature::output`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        proof: {
                            let __codec_res_edqy =
                                <VrfProof as ::codec::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `VrfSignature::proof`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                    })
                }
            }
        };
        const _: () = {
            impl ::codec::MaxEncodedLen for VrfSignature {
                fn max_encoded_len() -> ::core::primitive::usize {
                    0_usize
                        .saturating_add(<VrfOutput>::max_encoded_len())
                        .saturating_add(<VrfProof>::max_encoded_len())
                }
            }
        };
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for VrfSignature {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(::scale_info::Path::new(
                            "VrfSignature",
                            "sp_core::sr25519::vrf",
                        ))
                        .type_params(::alloc::vec::Vec::new())
                        .docs(&["VRF signature data"])
                        .composite(
                            ::scale_info::build::Fields::named()
                                .field(|f| {
                                    f.ty::<VrfOutput>()
                                        .name("output")
                                        .type_name("VrfOutput")
                                        .docs(&["VRF output."])
                                })
                                .field(|f| {
                                    f.ty::<VrfProof>()
                                        .name("proof")
                                        .type_name("VrfProof")
                                        .docs(&["VRF proof."])
                                }),
                        )
                }
            };
        };
        /// VRF output type suitable for schnorrkel operations.
        pub struct VrfOutput(pub schnorrkel::vrf::VRFOutput);
        #[automatically_derived]
        impl ::core::clone::Clone for VrfOutput {
            #[inline]
            fn clone(&self) -> VrfOutput {
                VrfOutput(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for VrfOutput {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "VrfOutput", &&self.0)
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for VrfOutput {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for VrfOutput {
            #[inline]
            fn eq(&self, other: &VrfOutput) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for VrfOutput {}
        #[automatically_derived]
        impl ::core::cmp::Eq for VrfOutput {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<schnorrkel::vrf::VRFOutput>;
            }
        }
        impl Encode for VrfOutput {
            fn encode(&self) -> Vec<u8> {
                self.0.as_bytes().encode()
            }
        }
        impl Decode for VrfOutput {
            fn decode<R: codec::Input>(i: &mut R) -> Result<Self, codec::Error> {
                let decoded = <[u8; VRF_OUTPUT_LENGTH]>::decode(i)?;
                Ok(Self(
                    schnorrkel::vrf::VRFOutput::from_bytes(&decoded).map_err(convert_error)?,
                ))
            }
        }
        impl MaxEncodedLen for VrfOutput {
            fn max_encoded_len() -> usize {
                <[u8; VRF_OUTPUT_LENGTH]>::max_encoded_len()
            }
        }
        impl TypeInfo for VrfOutput {
            type Identity = [u8; VRF_OUTPUT_LENGTH];
            fn type_info() -> scale_info::Type {
                Self::Identity::type_info()
            }
        }
        /// VRF proof type suitable for schnorrkel operations.
        pub struct VrfProof(pub schnorrkel::vrf::VRFProof);
        #[automatically_derived]
        impl ::core::clone::Clone for VrfProof {
            #[inline]
            fn clone(&self) -> VrfProof {
                VrfProof(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for VrfProof {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "VrfProof", &&self.0)
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for VrfProof {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for VrfProof {
            #[inline]
            fn eq(&self, other: &VrfProof) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for VrfProof {}
        #[automatically_derived]
        impl ::core::cmp::Eq for VrfProof {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<schnorrkel::vrf::VRFProof>;
            }
        }
        impl Encode for VrfProof {
            fn encode(&self) -> Vec<u8> {
                self.0.to_bytes().encode()
            }
        }
        impl Decode for VrfProof {
            fn decode<R: codec::Input>(i: &mut R) -> Result<Self, codec::Error> {
                let decoded = <[u8; VRF_PROOF_LENGTH]>::decode(i)?;
                Ok(Self(
                    schnorrkel::vrf::VRFProof::from_bytes(&decoded).map_err(convert_error)?,
                ))
            }
        }
        impl MaxEncodedLen for VrfProof {
            fn max_encoded_len() -> usize {
                <[u8; VRF_PROOF_LENGTH]>::max_encoded_len()
            }
        }
        impl TypeInfo for VrfProof {
            type Identity = [u8; VRF_PROOF_LENGTH];
            fn type_info() -> scale_info::Type {
                Self::Identity::type_info()
            }
        }
        #[cfg(feature = "full_crypto")]
        impl VrfCrypto for Pair {
            type VrfInput = VrfTranscript;
            type VrfOutput = VrfOutput;
            type VrfSignData = VrfSignData;
            type VrfSignature = VrfSignature;
        }
        #[cfg(feature = "full_crypto")]
        impl VrfSecret for Pair {
            fn vrf_sign(&self, data: &Self::VrfSignData) -> Self::VrfSignature {
                let inout = self.0.vrf_create_hash(data.transcript.0.clone());
                let extra = data
                    .extra
                    .as_ref()
                    .map(|e| e.0.clone())
                    .unwrap_or_else(|| merlin::Transcript::new(DEFAULT_EXTRA_DATA_LABEL));
                let proof = self.0.dleq_proove(extra, &inout, true).0;
                VrfSignature {
                    output: VrfOutput(inout.to_output()),
                    proof: VrfProof(proof),
                }
            }
            fn vrf_output(&self, input: &Self::VrfInput) -> Self::VrfOutput {
                let output = self.0.vrf_create_hash(input.0.clone()).to_output();
                VrfOutput(output)
            }
        }
        impl VrfCrypto for Public {
            type VrfInput = VrfTranscript;
            type VrfOutput = VrfOutput;
            type VrfSignData = VrfSignData;
            type VrfSignature = VrfSignature;
        }
        impl VrfPublic for Public {
            fn vrf_verify(&self, data: &Self::VrfSignData, signature: &Self::VrfSignature) -> bool {
                let do_verify = || {
                    let public = schnorrkel::PublicKey::from_bytes(self)?;
                    let inout = signature
                        .output
                        .0
                        .attach_input_hash(&public, data.transcript.0.clone())?;
                    let extra = data
                        .extra
                        .as_ref()
                        .map(|e| e.0.clone())
                        .unwrap_or_else(|| merlin::Transcript::new(DEFAULT_EXTRA_DATA_LABEL));
                    public.dleq_verify(extra, &inout, &signature.proof.0, true)
                };
                do_verify().is_ok()
            }
        }
        fn convert_error(e: SignatureError) -> codec::Error {
            use MultiSignatureStage::*;
            use SignatureError::*;
            match e {
                EquationFalse => "Signature error: `EquationFalse`".into(),
                PointDecompressionError => "Signature error: `PointDecompressionError`".into(),
                ScalarFormatError => "Signature error: `ScalarFormatError`".into(),
                NotMarkedSchnorrkel => "Signature error: `NotMarkedSchnorrkel`".into(),
                BytesLengthError { .. } => "Signature error: `BytesLengthError`".into(),
                MuSigAbsent {
                    musig_stage: Commitment,
                } => "Signature error: `MuSigAbsent` at stage `Commitment`".into(),
                MuSigAbsent {
                    musig_stage: Reveal,
                } => "Signature error: `MuSigAbsent` at stage `Reveal`".into(),
                MuSigAbsent {
                    musig_stage: Cosignature,
                } => "Signature error: `MuSigAbsent` at stage `Commitment`".into(),
                MuSigInconsistent {
                    musig_stage: Commitment,
                    duplicate: true,
                } => {
                    "Signature error: `MuSigInconsistent` at stage `Commitment` on duplicate".into()
                }
                MuSigInconsistent {
                    musig_stage: Commitment,
                    duplicate: false,
                } => "Signature error: `MuSigInconsistent` at stage `Commitment` on not duplicate"
                    .into(),
                MuSigInconsistent {
                    musig_stage: Reveal,
                    duplicate: true,
                } => "Signature error: `MuSigInconsistent` at stage `Reveal` on duplicate".into(),
                MuSigInconsistent {
                    musig_stage: Reveal,
                    duplicate: false,
                } => {
                    "Signature error: `MuSigInconsistent` at stage `Reveal` on not duplicate".into()
                }
                MuSigInconsistent {
                    musig_stage: Cosignature,
                    duplicate: true,
                } => "Signature error: `MuSigInconsistent` at stage `Cosignature` on duplicate"
                    .into(),
                MuSigInconsistent {
                    musig_stage: Cosignature,
                    duplicate: false,
                } => "Signature error: `MuSigInconsistent` at stage `Cosignature` on not duplicate"
                    .into(),
            }
        }
        #[cfg(feature = "full_crypto")]
        impl Pair {
            /// Generate output bytes from the given VRF configuration.
            pub fn make_bytes<const N: usize>(&self, context: &[u8], input: &VrfInput) -> [u8; N]
            where
                [u8; N]: Default,
            {
                let inout = self.0.vrf_create_hash(input.0.clone());
                inout.make_bytes::<[u8; N]>(context)
            }
        }
        impl Public {
            /// Generate output bytes from the given VRF configuration.
            pub fn make_bytes<const N: usize>(
                &self,
                context: &[u8],
                input: &VrfInput,
                output: &VrfOutput,
            ) -> Result<[u8; N], codec::Error>
            where
                [u8; N]: Default,
            {
                let pubkey = schnorrkel::PublicKey::from_bytes(&self.0).map_err(convert_error)?;
                let inout = output
                    .0
                    .attach_input_hash(&pubkey, input.0.clone())
                    .map_err(convert_error)?;
                Ok(inout.make_bytes::<[u8; N]>(context))
            }
        }
        impl VrfOutput {
            /// Generate output bytes from the given VRF configuration.
            pub fn make_bytes<const N: usize>(
                &self,
                context: &[u8],
                input: &VrfInput,
                public: &Public,
            ) -> Result<[u8; N], codec::Error>
            where
                [u8; N]: Default,
            {
                public.make_bytes(context, input, self)
            }
        }
    }
}
pub mod testing {
    //! Types that should only be used for testing!
    use crate::crypto::KeyTypeId;
    /// Key type for generic Ed25519 key.
    pub const ED25519: KeyTypeId = KeyTypeId(*b"ed25");
    /// Key type for generic Sr 25519 key.
    pub const SR25519: KeyTypeId = KeyTypeId(*b"sr25");
    /// Key type for generic ECDSA key.
    pub const ECDSA: KeyTypeId = KeyTypeId(*b"ecds");
    /// Key type for generic BLS12-377 key.
    pub const BLS377: KeyTypeId = KeyTypeId(*b"bls7");
    /// Key type for generic BLS12-381 key.
    pub const BLS381: KeyTypeId = KeyTypeId(*b"bls8");
    /// A task executor that can be used in tests.
    ///
    /// Internally this just wraps a `ThreadPool` with a pool size of `8`. This
    /// should ensure that we have enough threads in tests for spawning blocking futures.
    #[cfg(feature = "std")]
    pub struct TaskExecutor(futures::executor::ThreadPool);
    #[automatically_derived]
    impl ::core::clone::Clone for TaskExecutor {
        #[inline]
        fn clone(&self) -> TaskExecutor {
            TaskExecutor(::core::clone::Clone::clone(&self.0))
        }
    }
    #[cfg(feature = "std")]
    impl TaskExecutor {
        /// Create a new instance of `Self`.
        pub fn new() -> Self {
            let mut builder = futures::executor::ThreadPoolBuilder::new();
            Self(
                builder
                    .pool_size(8)
                    .create()
                    .expect("Failed to create thread pool"),
            )
        }
    }
    #[cfg(feature = "std")]
    impl Default for TaskExecutor {
        fn default() -> Self {
            Self::new()
        }
    }
    #[cfg(feature = "std")]
    impl crate::traits::SpawnNamed for TaskExecutor {
        fn spawn_blocking(
            &self,
            _name: &'static str,
            _group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            self.0.spawn_ok(future);
        }
        fn spawn(
            &self,
            _name: &'static str,
            _group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            self.0.spawn_ok(future);
        }
    }
    #[cfg(feature = "std")]
    impl crate::traits::SpawnEssentialNamed for TaskExecutor {
        fn spawn_essential_blocking(
            &self,
            _: &'static str,
            _: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            self.0.spawn_ok(future);
        }
        fn spawn_essential(
            &self,
            _: &'static str,
            _: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            self.0.spawn_ok(future);
        }
    }
}
#[cfg(feature = "std")]
pub mod traits {
    //! Shareable Substrate traits.
    use std::{
        borrow::Cow,
        fmt::{Debug, Display},
    };
    pub use sp_externalities::{Externalities, ExternalitiesExt};
    /// The context in which a call is done.
    ///
    /// Depending on the context the executor may chooses different kind of heap sizes for the runtime
    /// instance.
    pub enum CallContext {
        /// The call is happening in some offchain context.
        Offchain,
        /// The call is happening in some on-chain context like building or importing a block.
        Onchain,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for CallContext {
        #[inline]
        fn clone(&self) -> CallContext {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for CallContext {}
    #[automatically_derived]
    impl ::core::fmt::Debug for CallContext {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                CallContext::Offchain => ::core::fmt::Formatter::write_str(f, "Offchain"),
                CallContext::Onchain => ::core::fmt::Formatter::write_str(f, "Onchain"),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for CallContext {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for CallContext {
        #[inline]
        fn eq(&self, other: &CallContext) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for CallContext {}
    #[automatically_derived]
    impl ::core::cmp::Eq for CallContext {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for CallContext {
        #[inline]
        fn cmp(&self, other: &CallContext) -> ::core::cmp::Ordering {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_tag, &__arg1_tag)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for CallContext {
        #[inline]
        fn partial_cmp(
            &self,
            other: &CallContext,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_tag, &__arg1_tag)
        }
    }
    /// Code execution engine.
    /// 代码执行引擎
    pub trait CodeExecutor: Sized + Send + Sync + ReadRuntimeVersion + Clone + 'static {
        /// Externalities error type.
        type Error: Display + Debug + Send + Sync + 'static;
        /// Call a given method in the runtime.
        ///
        /// Returns a tuple of the result (either the output data or an execution error) together with a
        /// `bool`, which is true if native execution was used.
        /// 在运行时调用给定的方法。
        /// 返回结果的元组（输出数据或执行错误）以及 bool，如果使用本机执行，则为 true。
        fn call(
            &self,
            ext: &mut dyn Externalities,
            runtime_code: &RuntimeCode,
            method: &str,
            data: &[u8],
            use_native: bool,
            context: CallContext,
        ) -> (Result<Vec<u8>, Self::Error>, bool);
    }
    /// Something that can fetch the runtime `:code`.
    pub trait FetchRuntimeCode {
        /// Fetch the runtime `:code`.
        ///
        /// If the `:code` could not be found/not available, `None` should be returned.
        fn fetch_runtime_code(&self) -> Option<Cow<[u8]>>;
    }
    /// Wrapper to use a `u8` slice or `Vec` as [`FetchRuntimeCode`].
    pub struct WrappedRuntimeCode<'a>(pub std::borrow::Cow<'a, [u8]>);
    impl<'a> FetchRuntimeCode for WrappedRuntimeCode<'a> {
        fn fetch_runtime_code(&self) -> Option<Cow<[u8]>> {
            Some(self.0.as_ref().into())
        }
    }
    /// Type that implements [`FetchRuntimeCode`] and always returns `None`.
    pub struct NoneFetchRuntimeCode;
    impl FetchRuntimeCode for NoneFetchRuntimeCode {
        fn fetch_runtime_code(&self) -> Option<Cow<[u8]>> {
            None
        }
    }
    /// The Wasm code of a Substrate runtime.
    pub struct RuntimeCode<'a> {
        /// The code fetcher that can be used to lazily fetch the code.
        pub code_fetcher: &'a dyn FetchRuntimeCode,
        /// The optional heap pages this `code` should be executed with.
        ///
        /// If `None` are given, the default value of the executor will be used.
        pub heap_pages: Option<u64>,
        /// The SCALE encoded hash of `code`.
        ///
        /// The hashing algorithm isn't that important, as long as all runtime
        /// code instances use the same.
        pub hash: Vec<u8>,
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for RuntimeCode<'a> {
        #[inline]
        fn clone(&self) -> RuntimeCode<'a> {
            RuntimeCode {
                code_fetcher: ::core::clone::Clone::clone(&self.code_fetcher),
                heap_pages: ::core::clone::Clone::clone(&self.heap_pages),
                hash: ::core::clone::Clone::clone(&self.hash),
            }
        }
    }
    impl<'a> PartialEq for RuntimeCode<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.hash == other.hash
        }
    }
    impl<'a> RuntimeCode<'a> {
        /// Create an empty instance.
        ///
        /// This is only useful for tests that don't want to execute any code.
        pub fn empty() -> Self {
            Self {
                code_fetcher: &NoneFetchRuntimeCode,
                hash: Vec::new(),
                heap_pages: None,
            }
        }
    }
    impl<'a> FetchRuntimeCode for RuntimeCode<'a> {
        fn fetch_runtime_code(&self) -> Option<Cow<[u8]>> {
            self.code_fetcher.fetch_runtime_code()
        }
    }
    /// Could not find the `:code` in the externalities while initializing the [`RuntimeCode`].
    pub struct CodeNotFound;
    #[automatically_derived]
    impl ::core::fmt::Debug for CodeNotFound {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "CodeNotFound")
        }
    }
    impl std::fmt::Display for CodeNotFound {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &["the storage entry `:code` doesn\'t have any code"],
                &[],
            ))
        }
    }
    /// A trait that allows reading version information from the binary.
    /// 允许从二进制文件中读取版本信息
    pub trait ReadRuntimeVersion: Send + Sync {
        /// Reads the runtime version information from the given wasm code.
        ///
        /// The version information may be embedded into the wasm binary itself. If it is not present,
        /// then this function may fallback to the legacy way of reading the version.
        ///
        /// The legacy mechanism involves instantiating the passed wasm runtime and calling
        /// `Core_version` on it. This is a very expensive operation.
        ///
        /// `ext` is only needed in case the calling into runtime happens. Otherwise it is ignored.
        ///
        /// Compressed wasm blobs are supported and will be decompressed if needed. If uncompression
        /// fails, the error is returned.
        ///
        /// # Errors
        ///
        /// If the version information present in binary, but is corrupted - returns an error.
        ///
        /// Otherwise, if there is no version information present, and calling into the runtime takes
        /// place, then an error would be returned if `Core_version` is not provided.
        fn read_runtime_version(
            &self,
            wasm_code: &[u8],
            ext: &mut dyn Externalities,
        ) -> Result<Vec<u8>, String>;
    }
    impl ReadRuntimeVersion for std::sync::Arc<dyn ReadRuntimeVersion> {
        fn read_runtime_version(
            &self,
            wasm_code: &[u8],
            ext: &mut dyn Externalities,
        ) -> Result<Vec<u8>, String> {
            (**self).read_runtime_version(wasm_code, ext)
        }
    }
    /// An extension that provides functionality to read version information from a given wasm blob.
    pub struct ReadRuntimeVersionExt(pub Box<dyn ReadRuntimeVersion>);
    impl ::sp_externalities::Extension for ReadRuntimeVersionExt {
        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    impl std::ops::Deref for ReadRuntimeVersionExt {
        type Target = Box<dyn ReadRuntimeVersion>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl std::ops::DerefMut for ReadRuntimeVersionExt {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl From<Box<dyn ReadRuntimeVersion>> for ReadRuntimeVersionExt {
        fn from(inner: Box<dyn ReadRuntimeVersion>) -> Self {
            Self(inner)
        }
    }
    impl ReadRuntimeVersionExt {
        /// Creates a new instance of the extension given a version determinator instance.
        pub fn new<T: ReadRuntimeVersion + 'static>(inner: T) -> Self {
            Self(Box::new(inner))
        }
    }
    /// Something that can spawn tasks (blocking and non-blocking) with an assigned name
    /// and optional group.
    pub trait SpawnNamed: dyn_clonable::dyn_clone::DynClone + Send + Sync {
        /// Spawn the given blocking future.
        ///
        /// The given `group` and `name` is used to identify the future in tracing.
        fn spawn_blocking(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        );
        /// Spawn the given non-blocking future.
        ///
        /// The given `group` and `name` is used to identify the future in tracing.
        fn spawn(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        );
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<dyn SpawnNamed + 'clone>
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<dyn SpawnNamed + ::dyn_clone::__private::Send + 'clone>
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<dyn SpawnNamed + ::dyn_clone::__private::Sync + 'clone>
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<
            dyn SpawnNamed + ::dyn_clone::__private::Send + ::dyn_clone::__private::Sync + 'clone,
        >
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl SpawnNamed for Box<dyn SpawnNamed> {
        fn spawn_blocking(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            (**self).spawn_blocking(name, group, future)
        }
        fn spawn(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            (**self).spawn(name, group, future)
        }
    }
    /// Something that can spawn essential tasks (blocking and non-blocking) with an assigned name
    /// and optional group.
    ///
    /// Essential tasks are special tasks that should take down the node when they end.
    pub trait SpawnEssentialNamed: dyn_clonable::dyn_clone::DynClone + Send + Sync {
        /// Spawn the given blocking future.
        ///
        /// The given `group` and `name` is used to identify the future in tracing.
        fn spawn_essential_blocking(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        );
        /// Spawn the given non-blocking future.
        ///
        /// The given `group` and `name` is used to identify the future in tracing.
        fn spawn_essential(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        );
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<dyn SpawnEssentialNamed + 'clone>
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<
            dyn SpawnEssentialNamed + ::dyn_clone::__private::Send + 'clone,
        >
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<
            dyn SpawnEssentialNamed + ::dyn_clone::__private::Sync + 'clone,
        >
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl<'clone> ::dyn_clone::__private::Clone
        for ::dyn_clone::__private::Box<
            dyn SpawnEssentialNamed
                + ::dyn_clone::__private::Send
                + ::dyn_clone::__private::Sync
                + 'clone,
        >
    {
        fn clone(&self) -> Self {
            ::dyn_clone::clone_box(&**self)
        }
    }
    impl SpawnEssentialNamed for Box<dyn SpawnEssentialNamed> {
        fn spawn_essential_blocking(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            (**self).spawn_essential_blocking(name, group, future)
        }
        fn spawn_essential(
            &self,
            name: &'static str,
            group: Option<&'static str>,
            future: futures::future::BoxFuture<'static, ()>,
        ) {
            (**self).spawn_essential(name, group, future)
        }
    }
}
pub mod uint {
    //! An unsigned fixed-size integer.
    pub use primitive_types::{U256, U512};
}
pub use self::{
    hash::{convert_hash, H160, H256, H512},
    uint::{U256, U512},
};
#[cfg(feature = "full_crypto")]
pub use crypto::{ByteArray, DeriveJunction, Pair, Public};
#[cfg(feature = "std")]
pub use self::hasher::blake2::Blake2Hasher;
#[cfg(feature = "std")]
pub use self::hasher::keccak::KeccakHasher;
pub use hash_db::Hasher;
pub use bounded_collections as bounded;
#[cfg(feature = "std")]
pub use bounded_collections::{bounded_btree_map, bounded_vec};
pub use bounded_collections::{
    parameter_types, ConstBool, ConstI128, ConstI16, ConstI32, ConstI64, ConstI8, ConstU128,
    ConstU16, ConstU32, ConstU64, ConstU8, Get, GetDefault, TryCollect, TypedGet,
};
pub use sp_storage as storage;
#[doc(hidden)]
pub use sp_std;
/// Context for executing a call into the runtime.
pub enum ExecutionContext {
    /// Context used for general block import (including locally authored blocks).
    Importing,
    /// Context used for importing blocks as part of an initial sync of the blockchain.
    ///
    /// We distinguish between major sync and import so that validators who are running
    /// their initial sync (or catching up after some time offline) can use the faster
    /// native runtime (since we can reasonably assume the network as a whole has already
    /// come to a broad consensus on the block and it probably hasn't been crafted
    /// specifically to attack this node), but when importing blocks at the head of the
    /// chain in normal operation they can use the safer Wasm version.
    Syncing,
    /// Context used for block construction.
    BlockConstruction,
    /// Context used for offchain calls.
    ///
    /// This allows passing offchain extension and customizing available capabilities.
    OffchainCall(Option<(Box<dyn offchain::Externalities>, offchain::Capabilities)>),
}
impl ExecutionContext {
    /// Returns the capabilities of particular context.
    pub fn capabilities(&self) -> offchain::Capabilities {
        use ExecutionContext::*;
        match self {
            Importing | Syncing | BlockConstruction => offchain::Capabilities::empty(),
            OffchainCall(None) => {
                offchain::Capabilities::KEYSTORE
                    | offchain::Capabilities::OFFCHAIN_DB_READ
                    | offchain::Capabilities::TRANSACTION_POOL
            }
            OffchainCall(Some((_, capabilities))) => *capabilities,
        }
    }
}
/// Hex-serialized shim for `Vec<u8>`.
pub struct Bytes(#[serde(with = "bytes")] pub Vec<u8>);
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Bytes {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serializer::serialize_newtype_struct(__serializer, "Bytes", {
                #[doc(hidden)]
                struct __SerializeWith<'__a> {
                    values: (&'__a Vec<u8>,),
                    phantom: _serde::__private::PhantomData<Bytes>,
                }
                impl<'__a> _serde::Serialize for __SerializeWith<'__a> {
                    fn serialize<__S>(
                        &self,
                        __s: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                    {
                        bytes::serialize(self.values.0, __s)
                    }
                }
                &__SerializeWith {
                    values: (&self.0,),
                    phantom: _serde::__private::PhantomData::<Bytes>,
                }
            })
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Bytes {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Bytes>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Bytes;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "tuple struct Bytes")
                }
                #[inline]
                fn visit_newtype_struct<__E>(
                    self,
                    __e: __E,
                ) -> _serde::__private::Result<Self::Value, __E::Error>
                where
                    __E: _serde::Deserializer<'de>,
                {
                    let __field0: Vec<u8> = match bytes::deserialize(__e) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::__private::Ok(Bytes(__field0))
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match {
                        #[doc(hidden)]
                        struct __DeserializeWith<'de> {
                            value: Vec<u8>,
                            phantom: _serde::__private::PhantomData<Bytes>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::Deserialize<'de> for __DeserializeWith<'de> {
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                            {
                                _serde::__private::Ok(__DeserializeWith {
                                    value: match bytes::deserialize(__deserializer) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                    phantom: _serde::__private::PhantomData,
                                    lifetime: _serde::__private::PhantomData,
                                })
                            }
                        }
                        _serde::__private::Option::map(
                            match _serde::de::SeqAccess::next_element::<__DeserializeWith<'de>>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                            |__wrap| __wrap.value,
                        )
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"tuple struct Bytes with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(Bytes(__field0))
                }
            }
            _serde::Deserializer::deserialize_newtype_struct(
                __deserializer,
                "Bytes",
                __Visitor {
                    marker: _serde::__private::PhantomData::<Bytes>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::hash::Hash for Bytes {
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        ::core::hash::Hash::hash(&self.0, state)
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for Bytes {
    #[inline]
    fn partial_cmp(&self, other: &Bytes) -> ::core::option::Option<::core::cmp::Ordering> {
        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for Bytes {
    #[inline]
    fn cmp(&self, other: &Bytes) -> ::core::cmp::Ordering {
        ::core::cmp::Ord::cmp(&self.0, &other.0)
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Bytes {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Bytes {
    #[inline]
    fn eq(&self, other: &Bytes) -> bool {
        self.0 == other.0
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for Bytes {}
#[automatically_derived]
impl ::core::cmp::Eq for Bytes {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Bytes {
    #[inline]
    fn clone(&self) -> Bytes {
        Bytes(::core::clone::Clone::clone(&self.0))
    }
}
impl core::fmt::Debug for Bytes {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_tuple("Bytes").field(&self.0).finish()
    }
}
impl From<Vec<u8>> for Bytes {
    fn from(s: Vec<u8>) -> Self {
        Bytes(s)
    }
}
impl From<OpaqueMetadata> for Bytes {
    fn from(s: OpaqueMetadata) -> Self {
        Bytes(s.0)
    }
}
impl Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}
impl codec::WrapperTypeEncode for Bytes {}
impl codec::WrapperTypeDecode for Bytes {
    type Wrapped = Vec<u8>;
}
#[cfg(feature = "std")]
impl sp_std::str::FromStr for Bytes {
    type Err = bytes::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        bytes::from_hex(s).map(Bytes)
    }
}
/// Stores the encoded `RuntimeMetadata` for the native side as opaque type.
pub struct OpaqueMetadata(Vec<u8>);
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for OpaqueMetadata {
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
        }
        fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
            ::codec::Encode::encode(&&self.0)
        }
        fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
            &self,
            f: F,
        ) -> R {
            ::codec::Encode::using_encoded(&&self.0, f)
        }
    }
    #[automatically_derived]
    impl ::codec::EncodeLike for OpaqueMetadata {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for OpaqueMetadata {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            ::core::result::Result::Ok(OpaqueMetadata({
                let __codec_res_edqy = <Vec<u8> as ::codec::Decode>::decode(__codec_input_edqy);
                match __codec_res_edqy {
                    ::core::result::Result::Err(e) => {
                        return ::core::result::Result::Err(
                            e.chain("Could not decode `OpaqueMetadata.0`"),
                        )
                    }
                    ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                }
            }))
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for OpaqueMetadata {}
#[automatically_derived]
impl ::core::cmp::PartialEq for OpaqueMetadata {
    #[inline]
    fn eq(&self, other: &OpaqueMetadata) -> bool {
        self.0 == other.0
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for OpaqueMetadata {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(::scale_info::Path::new("OpaqueMetadata", "sp_core"))
                .type_params(::alloc::vec::Vec::new())
                .docs(&["Stores the encoded `RuntimeMetadata` for the native side as opaque type."])
                .composite(
                    ::scale_info::build::Fields::unnamed()
                        .field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                )
        }
    };
};
impl OpaqueMetadata {
    /// Creates a new instance with the given metadata blob.
    pub fn new(metadata: Vec<u8>) -> Self {
        OpaqueMetadata(metadata)
    }
}
impl sp_std::ops::Deref for OpaqueMetadata {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// Simple blob to hold a `PeerId` without committing to its format.
pub struct OpaquePeerId(pub Vec<u8>);
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for OpaquePeerId {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serializer::serialize_newtype_struct(__serializer, "OpaquePeerId", &self.0)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for OpaquePeerId {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<OpaquePeerId>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = OpaquePeerId;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "tuple struct OpaquePeerId",
                    )
                }
                #[inline]
                fn visit_newtype_struct<__E>(
                    self,
                    __e: __E,
                ) -> _serde::__private::Result<Self::Value, __E::Error>
                where
                    __E: _serde::Deserializer<'de>,
                {
                    let __field0: Vec<u8> = match <Vec<u8> as _serde::Deserialize>::deserialize(__e)
                    {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::__private::Ok(OpaquePeerId(__field0))
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 =
                        match match _serde::de::SeqAccess::next_element::<Vec<u8>>(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    0usize,
                                    &"tuple struct OpaquePeerId with 1 element",
                                ));
                            }
                        };
                    _serde::__private::Ok(OpaquePeerId(__field0))
                }
            }
            _serde::Deserializer::deserialize_newtype_struct(
                __deserializer,
                "OpaquePeerId",
                __Visitor {
                    marker: _serde::__private::PhantomData::<OpaquePeerId>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
impl ::core::default::Default for OpaquePeerId {
    #[inline]
    fn default() -> OpaquePeerId {
        OpaquePeerId(::core::default::Default::default())
    }
}
#[automatically_derived]
impl ::core::clone::Clone for OpaquePeerId {
    #[inline]
    fn clone(&self) -> OpaquePeerId {
        OpaquePeerId(::core::clone::Clone::clone(&self.0))
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for OpaquePeerId {}
#[automatically_derived]
impl ::core::cmp::Eq for OpaquePeerId {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<Vec<u8>>;
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for OpaquePeerId {}
#[automatically_derived]
impl ::core::cmp::PartialEq for OpaquePeerId {
    #[inline]
    fn eq(&self, other: &OpaquePeerId) -> bool {
        self.0 == other.0
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for OpaquePeerId {
    #[inline]
    fn cmp(&self, other: &OpaquePeerId) -> ::core::cmp::Ordering {
        ::core::cmp::Ord::cmp(&self.0, &other.0)
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for OpaquePeerId {
    #[inline]
    fn partial_cmp(&self, other: &OpaquePeerId) -> ::core::option::Option<::core::cmp::Ordering> {
        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for OpaquePeerId {
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
        }
        fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
            ::codec::Encode::encode(&&self.0)
        }
        fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
            &self,
            f: F,
        ) -> R {
            ::codec::Encode::using_encoded(&&self.0, f)
        }
    }
    #[automatically_derived]
    impl ::codec::EncodeLike for OpaquePeerId {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for OpaquePeerId {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            ::core::result::Result::Ok(OpaquePeerId({
                let __codec_res_edqy = <Vec<u8> as ::codec::Decode>::decode(__codec_input_edqy);
                match __codec_res_edqy {
                    ::core::result::Result::Err(e) => {
                        return ::core::result::Result::Err(
                            e.chain("Could not decode `OpaquePeerId.0`"),
                        )
                    }
                    ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                }
            }))
        }
    }
};
impl core::fmt::Debug for OpaquePeerId {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_tuple("OpaquePeerId").field(&self.0).finish()
    }
}
const _: () = {
    #[doc(hidden)]
    extern crate sp_runtime_interface as proc_macro_runtime_interface;
    impl proc_macro_runtime_interface::pass_by::PassBy for OpaquePeerId {
        type PassBy = proc_macro_runtime_interface::pass_by::Inner<Self, Vec<u8>>;
    }
    impl proc_macro_runtime_interface::pass_by::PassByInner for OpaquePeerId {
        type Inner = Vec<u8>;
        fn into_inner(self) -> Self::Inner {
            self.0
        }
        fn inner(&self) -> &Self::Inner {
            &self.0
        }
        fn from_inner(inner: Self::Inner) -> Self {
            Self(inner)
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for OpaquePeerId {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(::scale_info::Path::new("OpaquePeerId", "sp_core"))
                .type_params(::alloc::vec::Vec::new())
                .docs(&["Simple blob to hold a `PeerId` without committing to its format."])
                .composite(
                    ::scale_info::build::Fields::unnamed()
                        .field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                )
        }
    };
};
impl OpaquePeerId {
    /// Create new `OpaquePeerId`
    pub fn new(vec: Vec<u8>) -> Self {
        OpaquePeerId(vec)
    }
}
/// Provide a simple 4 byte identifier for a type.
pub trait TypeId {
    /// Simple 4 byte identifier.
    const TYPE_ID: [u8; 4];
}
/// A log level matching the one from `log` crate.
///
/// Used internally by `sp_io::logging::log` method.
pub enum LogLevel {
    /// `Error` log level.
    Error = 1_isize,
    /// `Warn` log level.
    Warn = 2_isize,
    /// `Info` log level.
    Info = 3_isize,
    /// `Debug` log level.
    Debug = 4_isize,
    /// `Trace` log level.
    Trace = 5_isize,
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for LogLevel {
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            match *self {
                LogLevel::Error => {
                    __codec_dest_edqy.push_byte(1_isize as ::core::primitive::u8);
                }
                LogLevel::Warn => {
                    __codec_dest_edqy.push_byte(2_isize as ::core::primitive::u8);
                }
                LogLevel::Info => {
                    __codec_dest_edqy.push_byte(3_isize as ::core::primitive::u8);
                }
                LogLevel::Debug => {
                    __codec_dest_edqy.push_byte(4_isize as ::core::primitive::u8);
                }
                LogLevel::Trace => {
                    __codec_dest_edqy.push_byte(5_isize as ::core::primitive::u8);
                }
                _ => (),
            }
        }
    }
    #[automatically_derived]
    impl ::codec::EncodeLike for LogLevel {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for LogLevel {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            match __codec_input_edqy
                .read_byte()
                .map_err(|e| e.chain("Could not decode `LogLevel`, failed to read variant byte"))?
            {
                __codec_x_edqy if __codec_x_edqy == 1_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevel::Error)
                }
                __codec_x_edqy if __codec_x_edqy == 2_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevel::Warn)
                }
                __codec_x_edqy if __codec_x_edqy == 3_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevel::Info)
                }
                __codec_x_edqy if __codec_x_edqy == 4_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevel::Debug)
                }
                __codec_x_edqy if __codec_x_edqy == 5_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevel::Trace)
                }
                _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                    "Could not decode `LogLevel`, variant doesn't exist",
                )),
            }
        }
    }
};
const _: () = {
    #[doc(hidden)]
    extern crate sp_runtime_interface as proc_macro_runtime_interface;
    impl proc_macro_runtime_interface::pass_by::PassBy for LogLevel {
        type PassBy = proc_macro_runtime_interface::pass_by::Enum<LogLevel>;
    }
    impl TryFrom<u8> for LogLevel {
        type Error = ();
        fn try_from(inner: u8) -> proc_macro_runtime_interface::sp_std::result::Result<Self, ()> {
            match inner {
                0u8 => Ok(LogLevel::Error),
                1u8 => Ok(LogLevel::Warn),
                2u8 => Ok(LogLevel::Info),
                3u8 => Ok(LogLevel::Debug),
                4u8 => Ok(LogLevel::Trace),
                _ => Err(()),
            }
        }
    }
    impl From<LogLevel> for u8 {
        fn from(var: LogLevel) -> u8 {
            match var {
                LogLevel::Error => 0u8,
                LogLevel::Warn => 1u8,
                LogLevel::Info => 2u8,
                LogLevel::Debug => 3u8,
                LogLevel::Trace => 4u8,
            }
        }
    }
};
#[automatically_derived]
impl ::core::marker::Copy for LogLevel {}
#[automatically_derived]
impl ::core::clone::Clone for LogLevel {
    #[inline]
    fn clone(&self) -> LogLevel {
        *self
    }
}
impl From<u32> for LogLevel {
    fn from(val: u32) -> Self {
        match val {
            x if x == LogLevel::Warn as u32 => LogLevel::Warn,
            x if x == LogLevel::Info as u32 => LogLevel::Info,
            x if x == LogLevel::Debug as u32 => LogLevel::Debug,
            x if x == LogLevel::Trace as u32 => LogLevel::Trace,
            _ => LogLevel::Error,
        }
    }
}
impl From<log::Level> for LogLevel {
    fn from(l: log::Level) -> Self {
        use log::Level::*;
        match l {
            Error => Self::Error,
            Warn => Self::Warn,
            Info => Self::Info,
            Debug => Self::Debug,
            Trace => Self::Trace,
        }
    }
}
impl From<LogLevel> for log::Level {
    fn from(l: LogLevel) -> Self {
        use self::LogLevel::*;
        match l {
            Error => Self::Error,
            Warn => Self::Warn,
            Info => Self::Info,
            Debug => Self::Debug,
            Trace => Self::Trace,
        }
    }
}
/// Log level filter that expresses which log levels should be filtered.
///
/// This enum matches the [`log::LevelFilter`] enum.
pub enum LogLevelFilter {
    /// `Off` log level filter.
    Off = 0_isize,
    /// `Error` log level filter.
    Error = 1_isize,
    /// `Warn` log level filter.
    Warn = 2_isize,
    /// `Info` log level filter.
    Info = 3_isize,
    /// `Debug` log level filter.
    Debug = 4_isize,
    /// `Trace` log level filter.
    Trace = 5_isize,
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for LogLevelFilter {
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            match *self {
                LogLevelFilter::Off => {
                    __codec_dest_edqy.push_byte(0_isize as ::core::primitive::u8);
                }
                LogLevelFilter::Error => {
                    __codec_dest_edqy.push_byte(1_isize as ::core::primitive::u8);
                }
                LogLevelFilter::Warn => {
                    __codec_dest_edqy.push_byte(2_isize as ::core::primitive::u8);
                }
                LogLevelFilter::Info => {
                    __codec_dest_edqy.push_byte(3_isize as ::core::primitive::u8);
                }
                LogLevelFilter::Debug => {
                    __codec_dest_edqy.push_byte(4_isize as ::core::primitive::u8);
                }
                LogLevelFilter::Trace => {
                    __codec_dest_edqy.push_byte(5_isize as ::core::primitive::u8);
                }
                _ => (),
            }
        }
    }
    #[automatically_derived]
    impl ::codec::EncodeLike for LogLevelFilter {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for LogLevelFilter {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            match __codec_input_edqy.read_byte().map_err(|e| {
                e.chain("Could not decode `LogLevelFilter`, failed to read variant byte")
            })? {
                __codec_x_edqy if __codec_x_edqy == 0_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevelFilter::Off)
                }
                __codec_x_edqy if __codec_x_edqy == 1_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevelFilter::Error)
                }
                __codec_x_edqy if __codec_x_edqy == 2_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevelFilter::Warn)
                }
                __codec_x_edqy if __codec_x_edqy == 3_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevelFilter::Info)
                }
                __codec_x_edqy if __codec_x_edqy == 4_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevelFilter::Debug)
                }
                __codec_x_edqy if __codec_x_edqy == 5_isize as ::core::primitive::u8 => {
                    ::core::result::Result::Ok(LogLevelFilter::Trace)
                }
                _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                    "Could not decode `LogLevelFilter`, variant doesn't exist",
                )),
            }
        }
    }
};
const _: () = {
    #[doc(hidden)]
    extern crate sp_runtime_interface as proc_macro_runtime_interface;
    impl proc_macro_runtime_interface::pass_by::PassBy for LogLevelFilter {
        type PassBy = proc_macro_runtime_interface::pass_by::Enum<LogLevelFilter>;
    }
    impl TryFrom<u8> for LogLevelFilter {
        type Error = ();
        fn try_from(inner: u8) -> proc_macro_runtime_interface::sp_std::result::Result<Self, ()> {
            match inner {
                0u8 => Ok(LogLevelFilter::Off),
                1u8 => Ok(LogLevelFilter::Error),
                2u8 => Ok(LogLevelFilter::Warn),
                3u8 => Ok(LogLevelFilter::Info),
                4u8 => Ok(LogLevelFilter::Debug),
                5u8 => Ok(LogLevelFilter::Trace),
                _ => Err(()),
            }
        }
    }
    impl From<LogLevelFilter> for u8 {
        fn from(var: LogLevelFilter) -> u8 {
            match var {
                LogLevelFilter::Off => 0u8,
                LogLevelFilter::Error => 1u8,
                LogLevelFilter::Warn => 2u8,
                LogLevelFilter::Info => 3u8,
                LogLevelFilter::Debug => 4u8,
                LogLevelFilter::Trace => 5u8,
            }
        }
    }
};
#[automatically_derived]
impl ::core::marker::Copy for LogLevelFilter {}
#[automatically_derived]
impl ::core::clone::Clone for LogLevelFilter {
    #[inline]
    fn clone(&self) -> LogLevelFilter {
        *self
    }
}
impl From<LogLevelFilter> for log::LevelFilter {
    fn from(l: LogLevelFilter) -> Self {
        use self::LogLevelFilter::*;
        match l {
            Off => Self::Off,
            Error => Self::Error,
            Warn => Self::Warn,
            Info => Self::Info,
            Debug => Self::Debug,
            Trace => Self::Trace,
        }
    }
}
impl From<log::LevelFilter> for LogLevelFilter {
    fn from(l: log::LevelFilter) -> Self {
        use log::LevelFilter::*;
        match l {
            Off => Self::Off,
            Error => Self::Error,
            Warn => Self::Warn,
            Info => Self::Info,
            Debug => Self::Debug,
            Trace => Self::Trace,
        }
    }
}
/// The void type - it cannot exist.
pub enum Void {}
#[automatically_derived]
impl ::core::clone::Clone for Void {
    #[inline]
    fn clone(&self) -> Void {
        unsafe { ::core::intrinsics::unreachable() }
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Decode for Void {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            match __codec_input_edqy
                .read_byte()
                .map_err(|e| e.chain("Could not decode `Void`, failed to read variant byte"))?
            {
                _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                    "Could not decode `Void`, variant doesn't exist",
                )),
            }
        }
    }
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::codec::Encode for Void {}
    #[automatically_derived]
    impl ::codec::EncodeLike for Void {}
};
#[automatically_derived]
impl ::core::marker::StructuralEq for Void {}
#[automatically_derived]
impl ::core::cmp::Eq for Void {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Void {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Void {
    #[inline]
    fn eq(&self, other: &Void) -> bool {
        unsafe { ::core::intrinsics::unreachable() }
    }
}
impl core::fmt::Debug for Void {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            _ => Ok(()),
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for Void {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(::scale_info::Path::new("Void", "sp_core"))
                .type_params(::alloc::vec::Vec::new())
                .docs(&["The void type - it cannot exist."])
                .variant(::scale_info::build::Variants::new())
        }
    };
};
const _: () = {
    impl ::codec::MaxEncodedLen for Void {
        fn max_encoded_len() -> ::core::primitive::usize {
            0_usize.saturating_add(1)
        }
    }
};
/// The maximum number of bytes that can be allocated at one time.
pub const MAX_POSSIBLE_ALLOCATION: u32 = 33554432;
