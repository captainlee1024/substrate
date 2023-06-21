mod schedule {
    //! This module contains the cost schedule and supporting code that constructs a
    //! sane default schedule from a `WeightInfo` implementation.
    use crate::{wasm::Determinism, weights::WeightInfo, Config};
    use codec::{Decode, Encode};
    use frame_support::{weights::Weight, DefaultNoBound};
    use pallet_contracts_proc_macro::{ScheduleDebug, WeightDebug};
    use scale_info::TypeInfo;
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};
    use sp_runtime::RuntimeDebug;
    use sp_std::marker::PhantomData;
    use wasm_instrument::{gas_metering, parity_wasm::elements};
    /// Definition of the cost schedule and other parameterizations for the wasm vm.
    ///
    /// Its [`Default`] implementation is the designated way to initialize this type. It uses
    /// the benchmarked information supplied by [`Config::WeightInfo`]. All of its fields are
    /// public and can therefore be modified. For example in order to change some of the limits
    /// and set a custom instruction weight version the following code could be used:
    /// ```rust
    /// use pallet_contracts::{Schedule, Limits, InstructionWeights, Config};
    ///
    /// fn create_schedule<T: Config>() -> Schedule<T> {
    ///     Schedule {
    ///         limits: Limits {
    /// 		        globals: 3,
    /// 		        parameters: 3,
    /// 		        memory_pages: 16,
    /// 		        table_size: 3,
    /// 		        br_table_size: 3,
    /// 		        .. Default::default()
    /// 	        },
    ///         instruction_weights: InstructionWeights {
    /// 	            version: 5,
    ///             .. Default::default()
    ///         },
    /// 	        .. Default::default()
    ///     }
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// Please make sure to bump the [`InstructionWeights::version`] whenever substantial
    /// changes are made to its values.
    #[serde(bound(serialize = "", deserialize = ""))]
    #[scale_info(skip_type_params(T))]
    pub struct Schedule<T: Config> {
        /// Describes the upper limits on various metrics.
        pub limits: Limits,
        /// The weights for individual wasm instructions.
        pub instruction_weights: InstructionWeights<T>,
        /// The weights for each imported function a contract is allowed to call.
        pub host_fn_weights: HostFnWeights<T>,
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone + Config> ::core::clone::Clone for Schedule<T> {
        #[inline]
        fn clone(&self) -> Schedule<T> {
            Schedule {
                limits: ::core::clone::Clone::clone(&self.limits),
                instruction_weights: ::core::clone::Clone::clone(&self.instruction_weights),
                host_fn_weights: ::core::clone::Clone::clone(&self.host_fn_weights),
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Encode for Schedule<T>
        where
            InstructionWeights<T>: ::codec::Encode,
            InstructionWeights<T>: ::codec::Encode,
            HostFnWeights<T>: ::codec::Encode,
            HostFnWeights<T>: ::codec::Encode,
        {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&self.limits, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.instruction_weights, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.host_fn_weights, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl<T: Config> ::codec::EncodeLike for Schedule<T>
        where
            InstructionWeights<T>: ::codec::Encode,
            InstructionWeights<T>: ::codec::Encode,
            HostFnWeights<T>: ::codec::Encode,
            HostFnWeights<T>: ::codec::Encode,
        {
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Decode for Schedule<T>
        where
            InstructionWeights<T>: ::codec::Decode,
            InstructionWeights<T>: ::codec::Decode,
            HostFnWeights<T>: ::codec::Decode,
            HostFnWeights<T>: ::codec::Decode,
        {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Schedule::<T> {
                    limits: {
                        let __codec_res_edqy =
                            <Limits as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Schedule::limits`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    instruction_weights: {
                        let __codec_res_edqy =
                            <InstructionWeights<T> as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Schedule::instruction_weights`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    host_fn_weights: {
                        let __codec_res_edqy =
                            <HostFnWeights<T> as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Schedule::host_fn_weights`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                })
            }
        }
    };
    #[automatically_derived]
    impl<T: Config> ::core::marker::StructuralPartialEq for Schedule<T> {}
    #[automatically_derived]
    impl<T: ::core::cmp::PartialEq + Config> ::core::cmp::PartialEq for Schedule<T> {
        #[inline]
        fn eq(&self, other: &Schedule<T>) -> bool {
            self.limits == other.limits
                && self.instruction_weights == other.instruction_weights
                && self.host_fn_weights == other.host_fn_weights
        }
    }
    #[automatically_derived]
    impl<T: Config> ::core::marker::StructuralEq for Schedule<T> {}
    #[automatically_derived]
    impl<T: ::core::cmp::Eq + Config> ::core::cmp::Eq for Schedule<T> {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Limits>;
            let _: ::core::cmp::AssertParamIsEq<InstructionWeights<T>>;
            let _: ::core::cmp::AssertParamIsEq<HostFnWeights<T>>;
        }
    }
    impl<T: Config> core::fmt::Debug for Schedule<T> {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ::sp_runtime::{FixedPointNumber, FixedU128 as Fixed};
            let mut formatter = formatter.debug_struct("Schedule");
            formatter.field("limits", &self.limits);
            formatter.field("instruction_weights", &self.instruction_weights);
            formatter.field("host_fn_weights", &self.host_fn_weights);
            formatter.finish()
        }
    }
    const _: () = {
        impl<T: Config> core::default::Default for Schedule<T> {
            fn default() -> Self {
                Self {
                    limits: core::default::Default::default(),
                    instruction_weights: core::default::Default::default(),
                    host_fn_weights: core::default::Default::default(),
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T: Config> ::scale_info::TypeInfo for Schedule<T>
        where
            InstructionWeights<T>: ::scale_info::TypeInfo + 'static,
            HostFnWeights<T>: ::scale_info::TypeInfo + 'static,
            T: Config + 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("Schedule" , "pallet_contracts::schedule")) . type_params (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([:: scale_info :: TypeParameter :: new ("T" , :: core :: option :: Option :: None)]))) . docs (& ["Definition of the cost schedule and other parameterizations for the wasm vm." , "" , "Its [`Default`] implementation is the designated way to initialize this type. It uses" , "the benchmarked information supplied by [`Config::WeightInfo`]. All of its fields are" , "public and can therefore be modified. For example in order to change some of the limits" , "and set a custom instruction weight version the following code could be used:" , "```rust" , "use pallet_contracts::{Schedule, Limits, InstructionWeights, Config};" , "" , "fn create_schedule<T: Config>() -> Schedule<T> {" , "    Schedule {" , "        limits: Limits {" , "\t\t        globals: 3," , "\t\t        parameters: 3," , "\t\t        memory_pages: 16," , "\t\t        table_size: 3," , "\t\t        br_table_size: 3," , "\t\t        .. Default::default()" , "\t        }," , "        instruction_weights: InstructionWeights {" , "\t            version: 5," , "            .. Default::default()" , "        }," , "\t        .. Default::default()" , "    }" , "}" , "```" , "" , "# Note" , "" , "Please make sure to bump the [`InstructionWeights::version`] whenever substantial" , "changes are made to its values."]) . composite (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < Limits > () . name ("limits") . type_name ("Limits") . docs (& ["Describes the upper limits on various metrics."])) . field (| f | f . ty :: < InstructionWeights < T > > () . name ("instruction_weights") . type_name ("InstructionWeights<T>") . docs (& ["The weights for individual wasm instructions."])) . field (| f | f . ty :: < HostFnWeights < T > > () . name ("host_fn_weights") . type_name ("HostFnWeights<T>") . docs (& ["The weights for each imported function a contract is allowed to call."])))
            }
        };
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<T: Config> _serde::Serialize for Schedule<T> {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = match _serde::Serializer::serialize_struct(
                    __serializer,
                    "Schedule",
                    false as usize + 1 + 1 + 1,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "limits",
                    &self.limits,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "instruction_weights",
                    &self.instruction_weights,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "host_fn_weights",
                    &self.host_fn_weights,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de, T: Config> _serde::Deserialize<'de> for Schedule<T> {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "field identifier")
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
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            "limits" => _serde::__private::Ok(__Field::__field0),
                            "instruction_weights" => _serde::__private::Ok(__Field::__field1),
                            "host_fn_weights" => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            b"limits" => _serde::__private::Ok(__Field::__field0),
                            b"instruction_weights" => _serde::__private::Ok(__Field::__field1),
                            b"host_fn_weights" => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                struct __Visitor<'de, T: Config> {
                    marker: _serde::__private::PhantomData<Schedule<T>>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de, T: Config> _serde::de::Visitor<'de> for __Visitor<'de, T> {
                    type Value = Schedule<T>;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "struct Schedule")
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
                            match match _serde::de::SeqAccess::next_element::<Limits>(&mut __seq) {
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
                                            &"struct Schedule with 3 elements",
                                        ),
                                    );
                                }
                            };
                        let __field1 = match match _serde::de::SeqAccess::next_element::<
                            InstructionWeights<T>,
                        >(&mut __seq)
                        {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct Schedule with 3 elements",
                                ));
                            }
                        };
                        let __field2 = match match _serde::de::SeqAccess::next_element::<
                            HostFnWeights<T>,
                        >(&mut __seq)
                        {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    2usize,
                                    &"struct Schedule with 3 elements",
                                ));
                            }
                        };
                        _serde::__private::Ok(Schedule {
                            limits: __field0,
                            instruction_weights: __field1,
                            host_fn_weights: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Limits> =
                            _serde::__private::None;
                        let mut __field1: _serde::__private::Option<InstructionWeights<T>> =
                            _serde::__private::None;
                        let mut __field2: _serde::__private::Option<HostFnWeights<T>> =
                            _serde::__private::None;
                        while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "limits",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Limits>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "instruction_weights",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<
                                            InstructionWeights<T>,
                                        >(&mut __map)
                                        {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "host_fn_weights",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<HostFnWeights<T>>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                _ => {
                                    let _ = match _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)
                                    {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("limits") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("instruction_weights") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("host_fn_weights") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        _serde::__private::Ok(Schedule {
                            limits: __field0,
                            instruction_weights: __field1,
                            host_fn_weights: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] =
                    &["limits", "instruction_weights", "host_fn_weights"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Schedule",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Schedule<T>>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    /// Describes the upper limits on various metrics.
    ///
    /// # Note
    ///
    /// The values in this struct should never be decreased. The reason is that decreasing those
    /// values will break existing contracts which are above the new limits when a
    /// re-instrumentation is triggered.
    pub struct Limits {
        /// The maximum number of topics supported by an event.
        pub event_topics: u32,
        /// Maximum number of globals a module is allowed to declare.
        ///
        /// Globals are not limited through the linear memory limit `memory_pages`.
        pub globals: u32,
        /// Maximum number of locals a function can have.
        ///
        /// As wasm engine initializes each of the local, we need to limit their number to confine
        /// execution costs.
        pub locals: u32,
        /// Maximum numbers of parameters a function can have.
        ///
        /// Those need to be limited to prevent a potentially exploitable interaction with
        /// the stack height instrumentation: The costs of executing the stack height
        /// instrumentation for an indirectly called function scales linearly with the amount
        /// of parameters of this function. Because the stack height instrumentation itself is
        /// is not weight metered its costs must be static (via this limit) and included in
        /// the costs of the instructions that cause them (call, call_indirect).
        pub parameters: u32,
        /// Maximum number of memory pages allowed for a contract.
        pub memory_pages: u32,
        /// Maximum number of elements allowed in a table.
        ///
        /// Currently, the only type of element that is allowed in a table is funcref.
        pub table_size: u32,
        /// Maximum number of elements that can appear as immediate value to the br_table instruction.
        pub br_table_size: u32,
        /// The maximum length of a subject in bytes used for PRNG generation.
        pub subject_len: u32,
        /// The maximum size of a storage value and event payload in bytes.
        pub payload_len: u32,
        /// The maximum node runtime memory. This is for integrity checks only and does not affect the
        /// real setting.
        pub runtime_memory: u32,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Limits {
        #[inline]
        fn clone(&self) -> Limits {
            Limits {
                event_topics: ::core::clone::Clone::clone(&self.event_topics),
                globals: ::core::clone::Clone::clone(&self.globals),
                locals: ::core::clone::Clone::clone(&self.locals),
                parameters: ::core::clone::Clone::clone(&self.parameters),
                memory_pages: ::core::clone::Clone::clone(&self.memory_pages),
                table_size: ::core::clone::Clone::clone(&self.table_size),
                br_table_size: ::core::clone::Clone::clone(&self.br_table_size),
                subject_len: ::core::clone::Clone::clone(&self.subject_len),
                payload_len: ::core::clone::Clone::clone(&self.payload_len),
                runtime_memory: ::core::clone::Clone::clone(&self.runtime_memory),
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Encode for Limits {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&self.event_topics, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.globals, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.locals, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.parameters, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.memory_pages, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.table_size, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.br_table_size, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.subject_len, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.payload_len, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.runtime_memory, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::codec::EncodeLike for Limits {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::codec::Decode for Limits {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(Limits {
                    event_topics: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::event_topics`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    globals: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::globals`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    locals: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::locals`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    parameters: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::parameters`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    memory_pages: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::memory_pages`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    table_size: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::table_size`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    br_table_size: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::br_table_size`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    subject_len: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::subject_len`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    payload_len: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::payload_len`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    runtime_memory: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Limits::runtime_memory`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Limits {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Limits {
        #[inline]
        fn eq(&self, other: &Limits) -> bool {
            self.event_topics == other.event_topics
                && self.globals == other.globals
                && self.locals == other.locals
                && self.parameters == other.parameters
                && self.memory_pages == other.memory_pages
                && self.table_size == other.table_size
                && self.br_table_size == other.br_table_size
                && self.subject_len == other.subject_len
                && self.payload_len == other.payload_len
                && self.runtime_memory == other.runtime_memory
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Limits {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Limits {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    impl core::fmt::Debug for Limits {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_struct("Limits")
                .field("event_topics", &self.event_topics)
                .field("globals", &self.globals)
                .field("locals", &self.locals)
                .field("parameters", &self.parameters)
                .field("memory_pages", &self.memory_pages)
                .field("table_size", &self.table_size)
                .field("br_table_size", &self.br_table_size)
                .field("subject_len", &self.subject_len)
                .field("payload_len", &self.payload_len)
                .field("runtime_memory", &self.runtime_memory)
                .finish()
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Limits {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("Limits" , "pallet_contracts::schedule")) . type_params (:: alloc :: vec :: Vec :: new ()) . docs (& ["Describes the upper limits on various metrics." , "" , "# Note" , "" , "The values in this struct should never be decreased. The reason is that decreasing those" , "values will break existing contracts which are above the new limits when a" , "re-instrumentation is triggered."]) . composite (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < u32 > () . name ("event_topics") . type_name ("u32") . docs (& ["The maximum number of topics supported by an event."])) . field (| f | f . ty :: < u32 > () . name ("globals") . type_name ("u32") . docs (& ["Maximum number of globals a module is allowed to declare." , "" , "Globals are not limited through the linear memory limit `memory_pages`."])) . field (| f | f . ty :: < u32 > () . name ("locals") . type_name ("u32") . docs (& ["Maximum number of locals a function can have." , "" , "As wasm engine initializes each of the local, we need to limit their number to confine" , "execution costs."])) . field (| f | f . ty :: < u32 > () . name ("parameters") . type_name ("u32") . docs (& ["Maximum numbers of parameters a function can have." , "" , "Those need to be limited to prevent a potentially exploitable interaction with" , "the stack height instrumentation: The costs of executing the stack height" , "instrumentation for an indirectly called function scales linearly with the amount" , "of parameters of this function. Because the stack height instrumentation itself is" , "is not weight metered its costs must be static (via this limit) and included in" , "the costs of the instructions that cause them (call, call_indirect)."])) . field (| f | f . ty :: < u32 > () . name ("memory_pages") . type_name ("u32") . docs (& ["Maximum number of memory pages allowed for a contract."])) . field (| f | f . ty :: < u32 > () . name ("table_size") . type_name ("u32") . docs (& ["Maximum number of elements allowed in a table." , "" , "Currently, the only type of element that is allowed in a table is funcref."])) . field (| f | f . ty :: < u32 > () . name ("br_table_size") . type_name ("u32") . docs (& ["Maximum number of elements that can appear as immediate value to the br_table instruction."])) . field (| f | f . ty :: < u32 > () . name ("subject_len") . type_name ("u32") . docs (& ["The maximum length of a subject in bytes used for PRNG generation."])) . field (| f | f . ty :: < u32 > () . name ("payload_len") . type_name ("u32") . docs (& ["The maximum size of a storage value and event payload in bytes."])) . field (| f | f . ty :: < u32 > () . name ("runtime_memory") . type_name ("u32") . docs (& ["The maximum node runtime memory. This is for integrity checks only and does not affect the" , "real setting."])))
            }
        };
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Limits {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = match _serde::Serializer::serialize_struct(
                    __serializer,
                    "Limits",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "event_topics",
                    &self.event_topics,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "globals",
                    &self.globals,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "locals",
                    &self.locals,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "parameters",
                    &self.parameters,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "memory_pages",
                    &self.memory_pages,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "table_size",
                    &self.table_size,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "br_table_size",
                    &self.br_table_size,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "subject_len",
                    &self.subject_len,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "payload_len",
                    &self.payload_len,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "runtime_memory",
                    &self.runtime_memory,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Limits {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "field identifier")
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
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            "event_topics" => _serde::__private::Ok(__Field::__field0),
                            "globals" => _serde::__private::Ok(__Field::__field1),
                            "locals" => _serde::__private::Ok(__Field::__field2),
                            "parameters" => _serde::__private::Ok(__Field::__field3),
                            "memory_pages" => _serde::__private::Ok(__Field::__field4),
                            "table_size" => _serde::__private::Ok(__Field::__field5),
                            "br_table_size" => _serde::__private::Ok(__Field::__field6),
                            "subject_len" => _serde::__private::Ok(__Field::__field7),
                            "payload_len" => _serde::__private::Ok(__Field::__field8),
                            "runtime_memory" => _serde::__private::Ok(__Field::__field9),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            b"event_topics" => _serde::__private::Ok(__Field::__field0),
                            b"globals" => _serde::__private::Ok(__Field::__field1),
                            b"locals" => _serde::__private::Ok(__Field::__field2),
                            b"parameters" => _serde::__private::Ok(__Field::__field3),
                            b"memory_pages" => _serde::__private::Ok(__Field::__field4),
                            b"table_size" => _serde::__private::Ok(__Field::__field5),
                            b"br_table_size" => _serde::__private::Ok(__Field::__field6),
                            b"subject_len" => _serde::__private::Ok(__Field::__field7),
                            b"payload_len" => _serde::__private::Ok(__Field::__field8),
                            b"runtime_memory" => _serde::__private::Ok(__Field::__field9),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                    marker: _serde::__private::PhantomData<Limits>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Limits;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "struct Limits")
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
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
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
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field1 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field2 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field3 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field4 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field5 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            5usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field6 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            6usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field7 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            7usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field8 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            8usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        let __field9 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            9usize,
                                            &"struct Limits with 10 elements",
                                        ),
                                    );
                                }
                            };
                        _serde::__private::Ok(Limits {
                            event_topics: __field0,
                            globals: __field1,
                            locals: __field2,
                            parameters: __field3,
                            memory_pages: __field4,
                            table_size: __field5,
                            br_table_size: __field6,
                            subject_len: __field7,
                            payload_len: __field8,
                            runtime_memory: __field9,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<u32> = _serde::__private::None;
                        while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "event_topics",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "globals",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "locals",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "parameters",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "memory_pages",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "table_size",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "br_table_size",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "subject_len",
                                            ),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "payload_len",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "runtime_memory",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                _ => {
                                    let _ = match _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)
                                    {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("event_topics") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("globals") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("locals") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("parameters") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("memory_pages") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("table_size") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("br_table_size") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("subject_len") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("payload_len") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("runtime_memory") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        _serde::__private::Ok(Limits {
                            event_topics: __field0,
                            globals: __field1,
                            locals: __field2,
                            parameters: __field3,
                            memory_pages: __field4,
                            table_size: __field5,
                            br_table_size: __field6,
                            subject_len: __field7,
                            payload_len: __field8,
                            runtime_memory: __field9,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "event_topics",
                    "globals",
                    "locals",
                    "parameters",
                    "memory_pages",
                    "table_size",
                    "br_table_size",
                    "subject_len",
                    "payload_len",
                    "runtime_memory",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Limits",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Limits>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl Limits {
        /// The maximum memory size in bytes that a contract can occupy.
        pub fn max_memory_size(&self) -> u32 {
            self.memory_pages * 64 * 1024
        }
    }
    /// Describes the weight for all categories of supported wasm instructions.
    ///
    /// There there is one field for each wasm instruction that describes the weight to
    /// execute one instruction of that name. There are a few exceptions:
    ///
    /// 1. If there is a i64 and a i32 variant of an instruction we use the weight
    ///    of the former for both.
    /// 2. The following instructions are free of charge because they merely structure the
    ///    wasm module and cannot be spammed without making the module invalid (and rejected):
    ///    End, Unreachable, Return, Else
    /// 3. The following instructions cannot be benchmarked because they are removed by any
    ///    real world execution engine as a preprocessing step and therefore don't yield a
    ///    meaningful benchmark result. However, in contrast to the instructions mentioned
    ///    in 2. they can be spammed. We price them with the same weight as the "default"
    ///    instruction (i64.const): Block, Loop, Nop
    /// 4. We price both i64.const and drop as InstructionWeights.i64const / 2. The reason
    ///    for that is that we cannot benchmark either of them on its own but we need their
    ///    individual values to derive (by subtraction) the weight of all other instructions
    ///    that use them as supporting instructions. Supporting means mainly pushing arguments
    ///    and dropping return values in order to maintain a valid module.
    #[scale_info(skip_type_params(T))]
    pub struct InstructionWeights<T: Config> {
        /// Version of the instruction weights.
        ///
        /// # Note
        ///
        /// Should be incremented whenever any instruction weight is changed. The
        /// reason is that changes to instruction weights require a re-instrumentation
        /// in order to apply the changes to an already deployed code. The re-instrumentation
        /// is triggered by comparing the version of the current schedule with the version the code was
        /// instrumented with. Changes usually happen when pallet_contracts is re-benchmarked.
        ///
        /// Changes to other parts of the schedule should not increment the version in
        /// order to avoid unnecessary re-instrumentations.
        pub version: u32,
        /// Weight to be used for instructions which don't have benchmarks assigned.
        ///
        /// This weight is used whenever a code is uploaded with [`Determinism::Relaxed`]
        /// and an instruction (usually a float instruction) is encountered. This weight is **not**
        /// used if a contract is uploaded with [`Determinism::Enforced`]. If this field is set to
        /// `0` (the default) only deterministic codes are allowed to be uploaded.
        pub fallback: u32,
        pub i64const: u32,
        pub i64load: u32,
        pub i64store: u32,
        pub select: u32,
        pub r#if: u32,
        pub br: u32,
        pub br_if: u32,
        pub br_table: u32,
        pub br_table_per_entry: u32,
        pub call: u32,
        pub call_indirect: u32,
        pub call_per_local: u32,
        pub local_get: u32,
        pub local_set: u32,
        pub local_tee: u32,
        pub global_get: u32,
        pub global_set: u32,
        pub memory_current: u32,
        pub memory_grow: u32,
        pub i64clz: u32,
        pub i64ctz: u32,
        pub i64popcnt: u32,
        pub i64eqz: u32,
        pub i64extendsi32: u32,
        pub i64extendui32: u32,
        pub i32wrapi64: u32,
        pub i64eq: u32,
        pub i64ne: u32,
        pub i64lts: u32,
        pub i64ltu: u32,
        pub i64gts: u32,
        pub i64gtu: u32,
        pub i64les: u32,
        pub i64leu: u32,
        pub i64ges: u32,
        pub i64geu: u32,
        pub i64add: u32,
        pub i64sub: u32,
        pub i64mul: u32,
        pub i64divs: u32,
        pub i64divu: u32,
        pub i64rems: u32,
        pub i64remu: u32,
        pub i64and: u32,
        pub i64or: u32,
        pub i64xor: u32,
        pub i64shl: u32,
        pub i64shrs: u32,
        pub i64shru: u32,
        pub i64rotl: u32,
        pub i64rotr: u32,
        /// The type parameter is used in the default implementation.
        #[codec(skip)]
        pub _phantom: PhantomData<T>,
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone + Config> ::core::clone::Clone for InstructionWeights<T> {
        #[inline]
        fn clone(&self) -> InstructionWeights<T> {
            InstructionWeights {
                version: ::core::clone::Clone::clone(&self.version),
                fallback: ::core::clone::Clone::clone(&self.fallback),
                i64const: ::core::clone::Clone::clone(&self.i64const),
                i64load: ::core::clone::Clone::clone(&self.i64load),
                i64store: ::core::clone::Clone::clone(&self.i64store),
                select: ::core::clone::Clone::clone(&self.select),
                r#if: ::core::clone::Clone::clone(&self.r#if),
                br: ::core::clone::Clone::clone(&self.br),
                br_if: ::core::clone::Clone::clone(&self.br_if),
                br_table: ::core::clone::Clone::clone(&self.br_table),
                br_table_per_entry: ::core::clone::Clone::clone(&self.br_table_per_entry),
                call: ::core::clone::Clone::clone(&self.call),
                call_indirect: ::core::clone::Clone::clone(&self.call_indirect),
                call_per_local: ::core::clone::Clone::clone(&self.call_per_local),
                local_get: ::core::clone::Clone::clone(&self.local_get),
                local_set: ::core::clone::Clone::clone(&self.local_set),
                local_tee: ::core::clone::Clone::clone(&self.local_tee),
                global_get: ::core::clone::Clone::clone(&self.global_get),
                global_set: ::core::clone::Clone::clone(&self.global_set),
                memory_current: ::core::clone::Clone::clone(&self.memory_current),
                memory_grow: ::core::clone::Clone::clone(&self.memory_grow),
                i64clz: ::core::clone::Clone::clone(&self.i64clz),
                i64ctz: ::core::clone::Clone::clone(&self.i64ctz),
                i64popcnt: ::core::clone::Clone::clone(&self.i64popcnt),
                i64eqz: ::core::clone::Clone::clone(&self.i64eqz),
                i64extendsi32: ::core::clone::Clone::clone(&self.i64extendsi32),
                i64extendui32: ::core::clone::Clone::clone(&self.i64extendui32),
                i32wrapi64: ::core::clone::Clone::clone(&self.i32wrapi64),
                i64eq: ::core::clone::Clone::clone(&self.i64eq),
                i64ne: ::core::clone::Clone::clone(&self.i64ne),
                i64lts: ::core::clone::Clone::clone(&self.i64lts),
                i64ltu: ::core::clone::Clone::clone(&self.i64ltu),
                i64gts: ::core::clone::Clone::clone(&self.i64gts),
                i64gtu: ::core::clone::Clone::clone(&self.i64gtu),
                i64les: ::core::clone::Clone::clone(&self.i64les),
                i64leu: ::core::clone::Clone::clone(&self.i64leu),
                i64ges: ::core::clone::Clone::clone(&self.i64ges),
                i64geu: ::core::clone::Clone::clone(&self.i64geu),
                i64add: ::core::clone::Clone::clone(&self.i64add),
                i64sub: ::core::clone::Clone::clone(&self.i64sub),
                i64mul: ::core::clone::Clone::clone(&self.i64mul),
                i64divs: ::core::clone::Clone::clone(&self.i64divs),
                i64divu: ::core::clone::Clone::clone(&self.i64divu),
                i64rems: ::core::clone::Clone::clone(&self.i64rems),
                i64remu: ::core::clone::Clone::clone(&self.i64remu),
                i64and: ::core::clone::Clone::clone(&self.i64and),
                i64or: ::core::clone::Clone::clone(&self.i64or),
                i64xor: ::core::clone::Clone::clone(&self.i64xor),
                i64shl: ::core::clone::Clone::clone(&self.i64shl),
                i64shrs: ::core::clone::Clone::clone(&self.i64shrs),
                i64shru: ::core::clone::Clone::clone(&self.i64shru),
                i64rotl: ::core::clone::Clone::clone(&self.i64rotl),
                i64rotr: ::core::clone::Clone::clone(&self.i64rotr),
                _phantom: ::core::clone::Clone::clone(&self._phantom),
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Encode for InstructionWeights<T> {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&self.version, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.fallback, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64const, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64load, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64store, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.select, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.r#if, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.br, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.br_if, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.br_table, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.br_table_per_entry, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.call, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.call_indirect, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.call_per_local, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.local_get, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.local_set, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.local_tee, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.global_get, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.global_set, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.memory_current, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.memory_grow, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64clz, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64ctz, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64popcnt, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64eqz, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64extendsi32, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64extendui32, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i32wrapi64, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64eq, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64ne, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64lts, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64ltu, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64gts, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64gtu, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64les, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64leu, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64ges, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64geu, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64add, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64sub, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64mul, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64divs, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64divu, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64rems, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64remu, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64and, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64or, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64xor, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64shl, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64shrs, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64shru, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64rotl, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.i64rotr, __codec_dest_edqy);
                let _ = &self._phantom;
            }
        }
        #[automatically_derived]
        impl<T: Config> ::codec::EncodeLike for InstructionWeights<T> {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Decode for InstructionWeights<T>
        where
            PhantomData<T>: Default,
        {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(InstructionWeights::<T> {
                    version: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::version`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    fallback: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::fallback`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64const: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64const`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64load: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64load`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64store: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64store`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    select: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::select`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    r#if: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::r#if`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    br: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::br`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    br_if: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::br_if`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    br_table: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::br_table`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    br_table_per_entry: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `InstructionWeights::br_table_per_entry`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    call: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::call`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    call_indirect: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::call_indirect`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    call_per_local: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `InstructionWeights::call_per_local`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    local_get: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::local_get`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    local_set: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::local_set`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    local_tee: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::local_tee`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    global_get: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::global_get`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    global_set: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::global_set`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    memory_current: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `InstructionWeights::memory_current`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    memory_grow: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::memory_grow`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64clz: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64clz`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64ctz: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64ctz`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64popcnt: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64popcnt`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64eqz: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64eqz`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64extendsi32: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64extendsi32`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64extendui32: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64extendui32`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i32wrapi64: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i32wrapi64`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64eq: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64eq`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64ne: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64ne`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64lts: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64lts`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64ltu: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64ltu`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64gts: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64gts`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64gtu: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64gtu`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64les: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64les`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64leu: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64leu`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64ges: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64ges`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64geu: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64geu`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64add: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64add`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64sub: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64sub`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64mul: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64mul`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64divs: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64divs`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64divu: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64divu`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64rems: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64rems`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64remu: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64remu`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64and: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64and`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64or: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64or`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64xor: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64xor`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64shl: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64shl`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64shrs: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64shrs`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64shru: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64shru`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64rotl: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64rotl`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    i64rotr: {
                        let __codec_res_edqy = <u32 as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InstructionWeights::i64rotr`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    _phantom: ::core::default::Default::default(),
                })
            }
        }
    };
    #[automatically_derived]
    impl<T: Config> ::core::marker::StructuralPartialEq for InstructionWeights<T> {}
    #[automatically_derived]
    impl<T: ::core::cmp::PartialEq + Config> ::core::cmp::PartialEq for InstructionWeights<T> {
        #[inline]
        fn eq(&self, other: &InstructionWeights<T>) -> bool {
            self.version == other.version
                && self.fallback == other.fallback
                && self.i64const == other.i64const
                && self.i64load == other.i64load
                && self.i64store == other.i64store
                && self.select == other.select
                && self.r#if == other.r#if
                && self.br == other.br
                && self.br_if == other.br_if
                && self.br_table == other.br_table
                && self.br_table_per_entry == other.br_table_per_entry
                && self.call == other.call
                && self.call_indirect == other.call_indirect
                && self.call_per_local == other.call_per_local
                && self.local_get == other.local_get
                && self.local_set == other.local_set
                && self.local_tee == other.local_tee
                && self.global_get == other.global_get
                && self.global_set == other.global_set
                && self.memory_current == other.memory_current
                && self.memory_grow == other.memory_grow
                && self.i64clz == other.i64clz
                && self.i64ctz == other.i64ctz
                && self.i64popcnt == other.i64popcnt
                && self.i64eqz == other.i64eqz
                && self.i64extendsi32 == other.i64extendsi32
                && self.i64extendui32 == other.i64extendui32
                && self.i32wrapi64 == other.i32wrapi64
                && self.i64eq == other.i64eq
                && self.i64ne == other.i64ne
                && self.i64lts == other.i64lts
                && self.i64ltu == other.i64ltu
                && self.i64gts == other.i64gts
                && self.i64gtu == other.i64gtu
                && self.i64les == other.i64les
                && self.i64leu == other.i64leu
                && self.i64ges == other.i64ges
                && self.i64geu == other.i64geu
                && self.i64add == other.i64add
                && self.i64sub == other.i64sub
                && self.i64mul == other.i64mul
                && self.i64divs == other.i64divs
                && self.i64divu == other.i64divu
                && self.i64rems == other.i64rems
                && self.i64remu == other.i64remu
                && self.i64and == other.i64and
                && self.i64or == other.i64or
                && self.i64xor == other.i64xor
                && self.i64shl == other.i64shl
                && self.i64shrs == other.i64shrs
                && self.i64shru == other.i64shru
                && self.i64rotl == other.i64rotl
                && self.i64rotr == other.i64rotr
                && self._phantom == other._phantom
        }
    }
    #[automatically_derived]
    impl<T: Config> ::core::marker::StructuralEq for InstructionWeights<T> {}
    #[automatically_derived]
    impl<T: ::core::cmp::Eq + Config> ::core::cmp::Eq for InstructionWeights<T> {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
            let _: ::core::cmp::AssertParamIsEq<PhantomData<T>>;
        }
    }
    impl<T: Config> core::fmt::Debug for InstructionWeights<T> {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ::sp_runtime::{FixedPointNumber, FixedU128 as Fixed};
            let mut formatter = formatter.debug_struct("InstructionWeights");
            formatter.field("version", &self.version);
            formatter.field("fallback", &self.fallback);
            formatter.field("i64const", &self.i64const);
            formatter.field("i64load", &self.i64load);
            formatter.field("i64store", &self.i64store);
            formatter.field("select", &self.select);
            formatter.field("r#if", &self.r#if);
            formatter.field("br", &self.br);
            formatter.field("br_if", &self.br_if);
            formatter.field("br_table", &self.br_table);
            formatter.field("br_table_per_entry", &self.br_table_per_entry);
            formatter.field("call", &self.call);
            formatter.field("call_indirect", &self.call_indirect);
            formatter.field("call_per_local", &self.call_per_local);
            formatter.field("local_get", &self.local_get);
            formatter.field("local_set", &self.local_set);
            formatter.field("local_tee", &self.local_tee);
            formatter.field("global_get", &self.global_get);
            formatter.field("global_set", &self.global_set);
            formatter.field("memory_current", &self.memory_current);
            formatter.field("memory_grow", &self.memory_grow);
            formatter.field("i64clz", &self.i64clz);
            formatter.field("i64ctz", &self.i64ctz);
            formatter.field("i64popcnt", &self.i64popcnt);
            formatter.field("i64eqz", &self.i64eqz);
            formatter.field("i64extendsi32", &self.i64extendsi32);
            formatter.field("i64extendui32", &self.i64extendui32);
            formatter.field("i32wrapi64", &self.i32wrapi64);
            formatter.field("i64eq", &self.i64eq);
            formatter.field("i64ne", &self.i64ne);
            formatter.field("i64lts", &self.i64lts);
            formatter.field("i64ltu", &self.i64ltu);
            formatter.field("i64gts", &self.i64gts);
            formatter.field("i64gtu", &self.i64gtu);
            formatter.field("i64les", &self.i64les);
            formatter.field("i64leu", &self.i64leu);
            formatter.field("i64ges", &self.i64ges);
            formatter.field("i64geu", &self.i64geu);
            formatter.field("i64add", &self.i64add);
            formatter.field("i64sub", &self.i64sub);
            formatter.field("i64mul", &self.i64mul);
            formatter.field("i64divs", &self.i64divs);
            formatter.field("i64divu", &self.i64divu);
            formatter.field("i64rems", &self.i64rems);
            formatter.field("i64remu", &self.i64remu);
            formatter.field("i64and", &self.i64and);
            formatter.field("i64or", &self.i64or);
            formatter.field("i64xor", &self.i64xor);
            formatter.field("i64shl", &self.i64shl);
            formatter.field("i64shrs", &self.i64shrs);
            formatter.field("i64shru", &self.i64shru);
            formatter.field("i64rotl", &self.i64rotl);
            formatter.field("i64rotr", &self.i64rotr);
            formatter.finish()
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T: Config> ::scale_info::TypeInfo for InstructionWeights<T>
        where
            PhantomData<T>: ::scale_info::TypeInfo + 'static,
            T: Config + 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("InstructionWeights" , "pallet_contracts::schedule")) . type_params (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([:: scale_info :: TypeParameter :: new ("T" , :: core :: option :: Option :: None)]))) . docs (& ["Describes the weight for all categories of supported wasm instructions." , "" , "There there is one field for each wasm instruction that describes the weight to" , "execute one instruction of that name. There are a few exceptions:" , "" , "1. If there is a i64 and a i32 variant of an instruction we use the weight" , "   of the former for both." , "2. The following instructions are free of charge because they merely structure the" , "   wasm module and cannot be spammed without making the module invalid (and rejected):" , "   End, Unreachable, Return, Else" , "3. The following instructions cannot be benchmarked because they are removed by any" , "   real world execution engine as a preprocessing step and therefore don't yield a" , "   meaningful benchmark result. However, in contrast to the instructions mentioned" , "   in 2. they can be spammed. We price them with the same weight as the \"default\"" , "   instruction (i64.const): Block, Loop, Nop" , "4. We price both i64.const and drop as InstructionWeights.i64const / 2. The reason" , "   for that is that we cannot benchmark either of them on its own but we need their" , "   individual values to derive (by subtraction) the weight of all other instructions" , "   that use them as supporting instructions. Supporting means mainly pushing arguments" , "   and dropping return values in order to maintain a valid module."]) . composite (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < u32 > () . name ("version") . type_name ("u32") . docs (& ["Version of the instruction weights." , "" , "# Note" , "" , "Should be incremented whenever any instruction weight is changed. The" , "reason is that changes to instruction weights require a re-instrumentation" , "in order to apply the changes to an already deployed code. The re-instrumentation" , "is triggered by comparing the version of the current schedule with the version the code was" , "instrumented with. Changes usually happen when pallet_contracts is re-benchmarked." , "" , "Changes to other parts of the schedule should not increment the version in" , "order to avoid unnecessary re-instrumentations."])) . field (| f | f . ty :: < u32 > () . name ("fallback") . type_name ("u32") . docs (& ["Weight to be used for instructions which don't have benchmarks assigned." , "" , "This weight is used whenever a code is uploaded with [`Determinism::Relaxed`]" , "and an instruction (usually a float instruction) is encountered. This weight is **not**" , "used if a contract is uploaded with [`Determinism::Enforced`]. If this field is set to" , "`0` (the default) only deterministic codes are allowed to be uploaded."])) . field (| f | f . ty :: < u32 > () . name ("i64const") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64load") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64store") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("select") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("r#if") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("br") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("br_if") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("br_table") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("br_table_per_entry") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("call") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("call_indirect") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("call_per_local") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("local_get") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("local_set") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("local_tee") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("global_get") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("global_set") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("memory_current") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("memory_grow") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64clz") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64ctz") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64popcnt") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64eqz") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64extendsi32") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64extendui32") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i32wrapi64") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64eq") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64ne") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64lts") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64ltu") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64gts") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64gtu") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64les") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64leu") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64ges") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64geu") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64add") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64sub") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64mul") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64divs") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64divu") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64rems") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64remu") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64and") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64or") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64xor") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64shl") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64shrs") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64shru") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64rotl") . type_name ("u32")) . field (| f | f . ty :: < u32 > () . name ("i64rotr") . type_name ("u32")))
            }
        };
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<T: Config> _serde::Serialize for InstructionWeights<T> {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = match _serde::Serializer::serialize_struct(
                    __serializer,
                    "InstructionWeights",
                    false as usize
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "version",
                    &self.version,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "fallback",
                    &self.fallback,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64const",
                    &self.i64const,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64load",
                    &self.i64load,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64store",
                    &self.i64store,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "select",
                    &self.select,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "if",
                    &self.r#if,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "br",
                    &self.br,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "br_if",
                    &self.br_if,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "br_table",
                    &self.br_table,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "br_table_per_entry",
                    &self.br_table_per_entry,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "call",
                    &self.call,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "call_indirect",
                    &self.call_indirect,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "call_per_local",
                    &self.call_per_local,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "local_get",
                    &self.local_get,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "local_set",
                    &self.local_set,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "local_tee",
                    &self.local_tee,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "global_get",
                    &self.global_get,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "global_set",
                    &self.global_set,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "memory_current",
                    &self.memory_current,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "memory_grow",
                    &self.memory_grow,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64clz",
                    &self.i64clz,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64ctz",
                    &self.i64ctz,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64popcnt",
                    &self.i64popcnt,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64eqz",
                    &self.i64eqz,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64extendsi32",
                    &self.i64extendsi32,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64extendui32",
                    &self.i64extendui32,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i32wrapi64",
                    &self.i32wrapi64,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64eq",
                    &self.i64eq,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64ne",
                    &self.i64ne,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64lts",
                    &self.i64lts,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64ltu",
                    &self.i64ltu,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64gts",
                    &self.i64gts,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64gtu",
                    &self.i64gtu,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64les",
                    &self.i64les,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64leu",
                    &self.i64leu,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64ges",
                    &self.i64ges,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64geu",
                    &self.i64geu,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64add",
                    &self.i64add,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64sub",
                    &self.i64sub,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64mul",
                    &self.i64mul,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64divs",
                    &self.i64divs,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64divu",
                    &self.i64divu,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64rems",
                    &self.i64rems,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64remu",
                    &self.i64remu,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64and",
                    &self.i64and,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64or",
                    &self.i64or,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64xor",
                    &self.i64xor,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64shl",
                    &self.i64shl,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64shrs",
                    &self.i64shrs,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64shru",
                    &self.i64shru,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64rotl",
                    &self.i64rotl,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i64rotr",
                    &self.i64rotr,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "_phantom",
                    &self._phantom,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de, T: Config> _serde::Deserialize<'de> for InstructionWeights<T> {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __field11,
                    __field12,
                    __field13,
                    __field14,
                    __field15,
                    __field16,
                    __field17,
                    __field18,
                    __field19,
                    __field20,
                    __field21,
                    __field22,
                    __field23,
                    __field24,
                    __field25,
                    __field26,
                    __field27,
                    __field28,
                    __field29,
                    __field30,
                    __field31,
                    __field32,
                    __field33,
                    __field34,
                    __field35,
                    __field36,
                    __field37,
                    __field38,
                    __field39,
                    __field40,
                    __field41,
                    __field42,
                    __field43,
                    __field44,
                    __field45,
                    __field46,
                    __field47,
                    __field48,
                    __field49,
                    __field50,
                    __field51,
                    __field52,
                    __field53,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "field identifier")
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
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            11u64 => _serde::__private::Ok(__Field::__field11),
                            12u64 => _serde::__private::Ok(__Field::__field12),
                            13u64 => _serde::__private::Ok(__Field::__field13),
                            14u64 => _serde::__private::Ok(__Field::__field14),
                            15u64 => _serde::__private::Ok(__Field::__field15),
                            16u64 => _serde::__private::Ok(__Field::__field16),
                            17u64 => _serde::__private::Ok(__Field::__field17),
                            18u64 => _serde::__private::Ok(__Field::__field18),
                            19u64 => _serde::__private::Ok(__Field::__field19),
                            20u64 => _serde::__private::Ok(__Field::__field20),
                            21u64 => _serde::__private::Ok(__Field::__field21),
                            22u64 => _serde::__private::Ok(__Field::__field22),
                            23u64 => _serde::__private::Ok(__Field::__field23),
                            24u64 => _serde::__private::Ok(__Field::__field24),
                            25u64 => _serde::__private::Ok(__Field::__field25),
                            26u64 => _serde::__private::Ok(__Field::__field26),
                            27u64 => _serde::__private::Ok(__Field::__field27),
                            28u64 => _serde::__private::Ok(__Field::__field28),
                            29u64 => _serde::__private::Ok(__Field::__field29),
                            30u64 => _serde::__private::Ok(__Field::__field30),
                            31u64 => _serde::__private::Ok(__Field::__field31),
                            32u64 => _serde::__private::Ok(__Field::__field32),
                            33u64 => _serde::__private::Ok(__Field::__field33),
                            34u64 => _serde::__private::Ok(__Field::__field34),
                            35u64 => _serde::__private::Ok(__Field::__field35),
                            36u64 => _serde::__private::Ok(__Field::__field36),
                            37u64 => _serde::__private::Ok(__Field::__field37),
                            38u64 => _serde::__private::Ok(__Field::__field38),
                            39u64 => _serde::__private::Ok(__Field::__field39),
                            40u64 => _serde::__private::Ok(__Field::__field40),
                            41u64 => _serde::__private::Ok(__Field::__field41),
                            42u64 => _serde::__private::Ok(__Field::__field42),
                            43u64 => _serde::__private::Ok(__Field::__field43),
                            44u64 => _serde::__private::Ok(__Field::__field44),
                            45u64 => _serde::__private::Ok(__Field::__field45),
                            46u64 => _serde::__private::Ok(__Field::__field46),
                            47u64 => _serde::__private::Ok(__Field::__field47),
                            48u64 => _serde::__private::Ok(__Field::__field48),
                            49u64 => _serde::__private::Ok(__Field::__field49),
                            50u64 => _serde::__private::Ok(__Field::__field50),
                            51u64 => _serde::__private::Ok(__Field::__field51),
                            52u64 => _serde::__private::Ok(__Field::__field52),
                            53u64 => _serde::__private::Ok(__Field::__field53),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            "version" => _serde::__private::Ok(__Field::__field0),
                            "fallback" => _serde::__private::Ok(__Field::__field1),
                            "i64const" => _serde::__private::Ok(__Field::__field2),
                            "i64load" => _serde::__private::Ok(__Field::__field3),
                            "i64store" => _serde::__private::Ok(__Field::__field4),
                            "select" => _serde::__private::Ok(__Field::__field5),
                            "if" => _serde::__private::Ok(__Field::__field6),
                            "br" => _serde::__private::Ok(__Field::__field7),
                            "br_if" => _serde::__private::Ok(__Field::__field8),
                            "br_table" => _serde::__private::Ok(__Field::__field9),
                            "br_table_per_entry" => _serde::__private::Ok(__Field::__field10),
                            "call" => _serde::__private::Ok(__Field::__field11),
                            "call_indirect" => _serde::__private::Ok(__Field::__field12),
                            "call_per_local" => _serde::__private::Ok(__Field::__field13),
                            "local_get" => _serde::__private::Ok(__Field::__field14),
                            "local_set" => _serde::__private::Ok(__Field::__field15),
                            "local_tee" => _serde::__private::Ok(__Field::__field16),
                            "global_get" => _serde::__private::Ok(__Field::__field17),
                            "global_set" => _serde::__private::Ok(__Field::__field18),
                            "memory_current" => _serde::__private::Ok(__Field::__field19),
                            "memory_grow" => _serde::__private::Ok(__Field::__field20),
                            "i64clz" => _serde::__private::Ok(__Field::__field21),
                            "i64ctz" => _serde::__private::Ok(__Field::__field22),
                            "i64popcnt" => _serde::__private::Ok(__Field::__field23),
                            "i64eqz" => _serde::__private::Ok(__Field::__field24),
                            "i64extendsi32" => _serde::__private::Ok(__Field::__field25),
                            "i64extendui32" => _serde::__private::Ok(__Field::__field26),
                            "i32wrapi64" => _serde::__private::Ok(__Field::__field27),
                            "i64eq" => _serde::__private::Ok(__Field::__field28),
                            "i64ne" => _serde::__private::Ok(__Field::__field29),
                            "i64lts" => _serde::__private::Ok(__Field::__field30),
                            "i64ltu" => _serde::__private::Ok(__Field::__field31),
                            "i64gts" => _serde::__private::Ok(__Field::__field32),
                            "i64gtu" => _serde::__private::Ok(__Field::__field33),
                            "i64les" => _serde::__private::Ok(__Field::__field34),
                            "i64leu" => _serde::__private::Ok(__Field::__field35),
                            "i64ges" => _serde::__private::Ok(__Field::__field36),
                            "i64geu" => _serde::__private::Ok(__Field::__field37),
                            "i64add" => _serde::__private::Ok(__Field::__field38),
                            "i64sub" => _serde::__private::Ok(__Field::__field39),
                            "i64mul" => _serde::__private::Ok(__Field::__field40),
                            "i64divs" => _serde::__private::Ok(__Field::__field41),
                            "i64divu" => _serde::__private::Ok(__Field::__field42),
                            "i64rems" => _serde::__private::Ok(__Field::__field43),
                            "i64remu" => _serde::__private::Ok(__Field::__field44),
                            "i64and" => _serde::__private::Ok(__Field::__field45),
                            "i64or" => _serde::__private::Ok(__Field::__field46),
                            "i64xor" => _serde::__private::Ok(__Field::__field47),
                            "i64shl" => _serde::__private::Ok(__Field::__field48),
                            "i64shrs" => _serde::__private::Ok(__Field::__field49),
                            "i64shru" => _serde::__private::Ok(__Field::__field50),
                            "i64rotl" => _serde::__private::Ok(__Field::__field51),
                            "i64rotr" => _serde::__private::Ok(__Field::__field52),
                            "_phantom" => _serde::__private::Ok(__Field::__field53),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            b"version" => _serde::__private::Ok(__Field::__field0),
                            b"fallback" => _serde::__private::Ok(__Field::__field1),
                            b"i64const" => _serde::__private::Ok(__Field::__field2),
                            b"i64load" => _serde::__private::Ok(__Field::__field3),
                            b"i64store" => _serde::__private::Ok(__Field::__field4),
                            b"select" => _serde::__private::Ok(__Field::__field5),
                            b"if" => _serde::__private::Ok(__Field::__field6),
                            b"br" => _serde::__private::Ok(__Field::__field7),
                            b"br_if" => _serde::__private::Ok(__Field::__field8),
                            b"br_table" => _serde::__private::Ok(__Field::__field9),
                            b"br_table_per_entry" => _serde::__private::Ok(__Field::__field10),
                            b"call" => _serde::__private::Ok(__Field::__field11),
                            b"call_indirect" => _serde::__private::Ok(__Field::__field12),
                            b"call_per_local" => _serde::__private::Ok(__Field::__field13),
                            b"local_get" => _serde::__private::Ok(__Field::__field14),
                            b"local_set" => _serde::__private::Ok(__Field::__field15),
                            b"local_tee" => _serde::__private::Ok(__Field::__field16),
                            b"global_get" => _serde::__private::Ok(__Field::__field17),
                            b"global_set" => _serde::__private::Ok(__Field::__field18),
                            b"memory_current" => _serde::__private::Ok(__Field::__field19),
                            b"memory_grow" => _serde::__private::Ok(__Field::__field20),
                            b"i64clz" => _serde::__private::Ok(__Field::__field21),
                            b"i64ctz" => _serde::__private::Ok(__Field::__field22),
                            b"i64popcnt" => _serde::__private::Ok(__Field::__field23),
                            b"i64eqz" => _serde::__private::Ok(__Field::__field24),
                            b"i64extendsi32" => _serde::__private::Ok(__Field::__field25),
                            b"i64extendui32" => _serde::__private::Ok(__Field::__field26),
                            b"i32wrapi64" => _serde::__private::Ok(__Field::__field27),
                            b"i64eq" => _serde::__private::Ok(__Field::__field28),
                            b"i64ne" => _serde::__private::Ok(__Field::__field29),
                            b"i64lts" => _serde::__private::Ok(__Field::__field30),
                            b"i64ltu" => _serde::__private::Ok(__Field::__field31),
                            b"i64gts" => _serde::__private::Ok(__Field::__field32),
                            b"i64gtu" => _serde::__private::Ok(__Field::__field33),
                            b"i64les" => _serde::__private::Ok(__Field::__field34),
                            b"i64leu" => _serde::__private::Ok(__Field::__field35),
                            b"i64ges" => _serde::__private::Ok(__Field::__field36),
                            b"i64geu" => _serde::__private::Ok(__Field::__field37),
                            b"i64add" => _serde::__private::Ok(__Field::__field38),
                            b"i64sub" => _serde::__private::Ok(__Field::__field39),
                            b"i64mul" => _serde::__private::Ok(__Field::__field40),
                            b"i64divs" => _serde::__private::Ok(__Field::__field41),
                            b"i64divu" => _serde::__private::Ok(__Field::__field42),
                            b"i64rems" => _serde::__private::Ok(__Field::__field43),
                            b"i64remu" => _serde::__private::Ok(__Field::__field44),
                            b"i64and" => _serde::__private::Ok(__Field::__field45),
                            b"i64or" => _serde::__private::Ok(__Field::__field46),
                            b"i64xor" => _serde::__private::Ok(__Field::__field47),
                            b"i64shl" => _serde::__private::Ok(__Field::__field48),
                            b"i64shrs" => _serde::__private::Ok(__Field::__field49),
                            b"i64shru" => _serde::__private::Ok(__Field::__field50),
                            b"i64rotl" => _serde::__private::Ok(__Field::__field51),
                            b"i64rotr" => _serde::__private::Ok(__Field::__field52),
                            b"_phantom" => _serde::__private::Ok(__Field::__field53),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                struct __Visitor<'de, T: Config> {
                    marker: _serde::__private::PhantomData<InstructionWeights<T>>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de, T: Config> _serde::de::Visitor<'de> for __Visitor<'de, T> {
                    type Value = InstructionWeights<T>;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct InstructionWeights",
                        )
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
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
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
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field1 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field2 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field3 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field4 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field5 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            5usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field6 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            6usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field7 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            7usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field8 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            8usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field9 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            9usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field10 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            10usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field11 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            11usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field12 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            12usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field13 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            13usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field14 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            14usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field15 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            15usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field16 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            16usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field17 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            17usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field18 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            18usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field19 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            19usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field20 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            20usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field21 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            21usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field22 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            22usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field23 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            23usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field24 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            24usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field25 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            25usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field26 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            26usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field27 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            27usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field28 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            28usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field29 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            29usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field30 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            30usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field31 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            31usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field32 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            32usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field33 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            33usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field34 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            34usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field35 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            35usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field36 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            36usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field37 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            37usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field38 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            38usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field39 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            39usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field40 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            40usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field41 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            41usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field42 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            42usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field43 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            43usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field44 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            44usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field45 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            45usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field46 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            46usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field47 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            47usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field48 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            48usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field49 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            49usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field50 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            50usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field51 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            51usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field52 =
                            match match _serde::de::SeqAccess::next_element::<u32>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            52usize,
                                            &"struct InstructionWeights with 54 elements",
                                        ),
                                    );
                                }
                            };
                        let __field53 = match match _serde::de::SeqAccess::next_element::<
                            PhantomData<T>,
                        >(&mut __seq)
                        {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    53usize,
                                    &"struct InstructionWeights with 54 elements",
                                ));
                            }
                        };
                        _serde::__private::Ok(InstructionWeights {
                            version: __field0,
                            fallback: __field1,
                            i64const: __field2,
                            i64load: __field3,
                            i64store: __field4,
                            select: __field5,
                            r#if: __field6,
                            br: __field7,
                            br_if: __field8,
                            br_table: __field9,
                            br_table_per_entry: __field10,
                            call: __field11,
                            call_indirect: __field12,
                            call_per_local: __field13,
                            local_get: __field14,
                            local_set: __field15,
                            local_tee: __field16,
                            global_get: __field17,
                            global_set: __field18,
                            memory_current: __field19,
                            memory_grow: __field20,
                            i64clz: __field21,
                            i64ctz: __field22,
                            i64popcnt: __field23,
                            i64eqz: __field24,
                            i64extendsi32: __field25,
                            i64extendui32: __field26,
                            i32wrapi64: __field27,
                            i64eq: __field28,
                            i64ne: __field29,
                            i64lts: __field30,
                            i64ltu: __field31,
                            i64gts: __field32,
                            i64gtu: __field33,
                            i64les: __field34,
                            i64leu: __field35,
                            i64ges: __field36,
                            i64geu: __field37,
                            i64add: __field38,
                            i64sub: __field39,
                            i64mul: __field40,
                            i64divs: __field41,
                            i64divu: __field42,
                            i64rems: __field43,
                            i64remu: __field44,
                            i64and: __field45,
                            i64or: __field46,
                            i64xor: __field47,
                            i64shl: __field48,
                            i64shrs: __field49,
                            i64shru: __field50,
                            i64rotl: __field51,
                            i64rotr: __field52,
                            _phantom: __field53,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field10: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field11: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field12: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field13: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field14: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field15: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field16: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field17: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field18: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field19: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field20: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field21: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field22: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field23: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field24: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field25: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field26: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field27: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field28: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field29: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field30: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field31: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field32: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field33: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field34: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field35: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field36: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field37: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field38: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field39: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field40: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field41: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field42: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field43: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field44: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field45: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field46: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field47: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field48: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field49: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field50: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field51: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field52: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field53: _serde::__private::Option<PhantomData<T>> =
                            _serde::__private::None;
                        while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "version",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "fallback",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64const",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64load",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64store",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "select",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "if",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "br",
                                            ),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "br_if",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "br_table",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "br_table_per_entry",
                                            ),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field11 => {
                                    if _serde::__private::Option::is_some(&__field11) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "call",
                                            ),
                                        );
                                    }
                                    __field11 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field12 => {
                                    if _serde::__private::Option::is_some(&__field12) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "call_indirect",
                                            ),
                                        );
                                    }
                                    __field12 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field13 => {
                                    if _serde::__private::Option::is_some(&__field13) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "call_per_local",
                                            ),
                                        );
                                    }
                                    __field13 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field14 => {
                                    if _serde::__private::Option::is_some(&__field14) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "local_get",
                                            ),
                                        );
                                    }
                                    __field14 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field15 => {
                                    if _serde::__private::Option::is_some(&__field15) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "local_set",
                                            ),
                                        );
                                    }
                                    __field15 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field16 => {
                                    if _serde::__private::Option::is_some(&__field16) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "local_tee",
                                            ),
                                        );
                                    }
                                    __field16 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field17 => {
                                    if _serde::__private::Option::is_some(&__field17) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "global_get",
                                            ),
                                        );
                                    }
                                    __field17 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field18 => {
                                    if _serde::__private::Option::is_some(&__field18) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "global_set",
                                            ),
                                        );
                                    }
                                    __field18 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field19 => {
                                    if _serde::__private::Option::is_some(&__field19) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "memory_current",
                                            ),
                                        );
                                    }
                                    __field19 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field20 => {
                                    if _serde::__private::Option::is_some(&__field20) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "memory_grow",
                                            ),
                                        );
                                    }
                                    __field20 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field21 => {
                                    if _serde::__private::Option::is_some(&__field21) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64clz",
                                            ),
                                        );
                                    }
                                    __field21 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field22 => {
                                    if _serde::__private::Option::is_some(&__field22) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64ctz",
                                            ),
                                        );
                                    }
                                    __field22 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field23 => {
                                    if _serde::__private::Option::is_some(&__field23) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64popcnt",
                                            ),
                                        );
                                    }
                                    __field23 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field24 => {
                                    if _serde::__private::Option::is_some(&__field24) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64eqz",
                                            ),
                                        );
                                    }
                                    __field24 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field25 => {
                                    if _serde::__private::Option::is_some(&__field25) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64extendsi32",
                                            ),
                                        );
                                    }
                                    __field25 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field26 => {
                                    if _serde::__private::Option::is_some(&__field26) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64extendui32",
                                            ),
                                        );
                                    }
                                    __field26 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field27 => {
                                    if _serde::__private::Option::is_some(&__field27) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i32wrapi64",
                                            ),
                                        );
                                    }
                                    __field27 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field28 => {
                                    if _serde::__private::Option::is_some(&__field28) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64eq",
                                            ),
                                        );
                                    }
                                    __field28 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field29 => {
                                    if _serde::__private::Option::is_some(&__field29) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64ne",
                                            ),
                                        );
                                    }
                                    __field29 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field30 => {
                                    if _serde::__private::Option::is_some(&__field30) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64lts",
                                            ),
                                        );
                                    }
                                    __field30 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field31 => {
                                    if _serde::__private::Option::is_some(&__field31) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64ltu",
                                            ),
                                        );
                                    }
                                    __field31 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field32 => {
                                    if _serde::__private::Option::is_some(&__field32) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64gts",
                                            ),
                                        );
                                    }
                                    __field32 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field33 => {
                                    if _serde::__private::Option::is_some(&__field33) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64gtu",
                                            ),
                                        );
                                    }
                                    __field33 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field34 => {
                                    if _serde::__private::Option::is_some(&__field34) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64les",
                                            ),
                                        );
                                    }
                                    __field34 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field35 => {
                                    if _serde::__private::Option::is_some(&__field35) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64leu",
                                            ),
                                        );
                                    }
                                    __field35 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field36 => {
                                    if _serde::__private::Option::is_some(&__field36) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64ges",
                                            ),
                                        );
                                    }
                                    __field36 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field37 => {
                                    if _serde::__private::Option::is_some(&__field37) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64geu",
                                            ),
                                        );
                                    }
                                    __field37 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field38 => {
                                    if _serde::__private::Option::is_some(&__field38) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64add",
                                            ),
                                        );
                                    }
                                    __field38 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field39 => {
                                    if _serde::__private::Option::is_some(&__field39) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64sub",
                                            ),
                                        );
                                    }
                                    __field39 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field40 => {
                                    if _serde::__private::Option::is_some(&__field40) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64mul",
                                            ),
                                        );
                                    }
                                    __field40 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field41 => {
                                    if _serde::__private::Option::is_some(&__field41) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64divs",
                                            ),
                                        );
                                    }
                                    __field41 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field42 => {
                                    if _serde::__private::Option::is_some(&__field42) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64divu",
                                            ),
                                        );
                                    }
                                    __field42 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field43 => {
                                    if _serde::__private::Option::is_some(&__field43) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64rems",
                                            ),
                                        );
                                    }
                                    __field43 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field44 => {
                                    if _serde::__private::Option::is_some(&__field44) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64remu",
                                            ),
                                        );
                                    }
                                    __field44 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field45 => {
                                    if _serde::__private::Option::is_some(&__field45) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64and",
                                            ),
                                        );
                                    }
                                    __field45 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field46 => {
                                    if _serde::__private::Option::is_some(&__field46) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64or",
                                            ),
                                        );
                                    }
                                    __field46 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field47 => {
                                    if _serde::__private::Option::is_some(&__field47) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64xor",
                                            ),
                                        );
                                    }
                                    __field47 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field48 => {
                                    if _serde::__private::Option::is_some(&__field48) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64shl",
                                            ),
                                        );
                                    }
                                    __field48 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field49 => {
                                    if _serde::__private::Option::is_some(&__field49) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64shrs",
                                            ),
                                        );
                                    }
                                    __field49 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field50 => {
                                    if _serde::__private::Option::is_some(&__field50) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64shru",
                                            ),
                                        );
                                    }
                                    __field50 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field51 => {
                                    if _serde::__private::Option::is_some(&__field51) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64rotl",
                                            ),
                                        );
                                    }
                                    __field51 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field52 => {
                                    if _serde::__private::Option::is_some(&__field52) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "i64rotr",
                                            ),
                                        );
                                    }
                                    __field52 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<u32>(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field53 => {
                                    if _serde::__private::Option::is_some(&__field53) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "_phantom",
                                            ),
                                        );
                                    }
                                    __field53 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<PhantomData<T>>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                _ => {
                                    let _ = match _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)
                                    {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("version") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("fallback") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64const") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64load") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64store") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("select") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("if") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("br") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("br_if") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("br_table") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("br_table_per_entry") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field11 = match __field11 {
                            _serde::__private::Some(__field11) => __field11,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("call") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field12 = match __field12 {
                            _serde::__private::Some(__field12) => __field12,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("call_indirect") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field13 = match __field13 {
                            _serde::__private::Some(__field13) => __field13,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("call_per_local") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field14 = match __field14 {
                            _serde::__private::Some(__field14) => __field14,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("local_get") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field15 = match __field15 {
                            _serde::__private::Some(__field15) => __field15,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("local_set") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field16 = match __field16 {
                            _serde::__private::Some(__field16) => __field16,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("local_tee") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field17 = match __field17 {
                            _serde::__private::Some(__field17) => __field17,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("global_get") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field18 = match __field18 {
                            _serde::__private::Some(__field18) => __field18,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("global_set") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field19 = match __field19 {
                            _serde::__private::Some(__field19) => __field19,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("memory_current") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field20 = match __field20 {
                            _serde::__private::Some(__field20) => __field20,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("memory_grow") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field21 = match __field21 {
                            _serde::__private::Some(__field21) => __field21,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64clz") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field22 = match __field22 {
                            _serde::__private::Some(__field22) => __field22,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64ctz") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field23 = match __field23 {
                            _serde::__private::Some(__field23) => __field23,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64popcnt") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field24 = match __field24 {
                            _serde::__private::Some(__field24) => __field24,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64eqz") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field25 = match __field25 {
                            _serde::__private::Some(__field25) => __field25,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64extendsi32") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field26 = match __field26 {
                            _serde::__private::Some(__field26) => __field26,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64extendui32") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field27 = match __field27 {
                            _serde::__private::Some(__field27) => __field27,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i32wrapi64") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field28 = match __field28 {
                            _serde::__private::Some(__field28) => __field28,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64eq") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field29 = match __field29 {
                            _serde::__private::Some(__field29) => __field29,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64ne") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field30 = match __field30 {
                            _serde::__private::Some(__field30) => __field30,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64lts") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field31 = match __field31 {
                            _serde::__private::Some(__field31) => __field31,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64ltu") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field32 = match __field32 {
                            _serde::__private::Some(__field32) => __field32,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64gts") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field33 = match __field33 {
                            _serde::__private::Some(__field33) => __field33,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64gtu") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field34 = match __field34 {
                            _serde::__private::Some(__field34) => __field34,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64les") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field35 = match __field35 {
                            _serde::__private::Some(__field35) => __field35,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64leu") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field36 = match __field36 {
                            _serde::__private::Some(__field36) => __field36,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64ges") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field37 = match __field37 {
                            _serde::__private::Some(__field37) => __field37,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64geu") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field38 = match __field38 {
                            _serde::__private::Some(__field38) => __field38,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64add") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field39 = match __field39 {
                            _serde::__private::Some(__field39) => __field39,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64sub") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field40 = match __field40 {
                            _serde::__private::Some(__field40) => __field40,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64mul") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field41 = match __field41 {
                            _serde::__private::Some(__field41) => __field41,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64divs") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field42 = match __field42 {
                            _serde::__private::Some(__field42) => __field42,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64divu") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field43 = match __field43 {
                            _serde::__private::Some(__field43) => __field43,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64rems") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field44 = match __field44 {
                            _serde::__private::Some(__field44) => __field44,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64remu") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field45 = match __field45 {
                            _serde::__private::Some(__field45) => __field45,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64and") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field46 = match __field46 {
                            _serde::__private::Some(__field46) => __field46,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64or") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field47 = match __field47 {
                            _serde::__private::Some(__field47) => __field47,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64xor") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field48 = match __field48 {
                            _serde::__private::Some(__field48) => __field48,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64shl") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field49 = match __field49 {
                            _serde::__private::Some(__field49) => __field49,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64shrs") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field50 = match __field50 {
                            _serde::__private::Some(__field50) => __field50,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64shru") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field51 = match __field51 {
                            _serde::__private::Some(__field51) => __field51,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64rotl") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field52 = match __field52 {
                            _serde::__private::Some(__field52) => __field52,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("i64rotr") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field53 = match __field53 {
                            _serde::__private::Some(__field53) => __field53,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("_phantom") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        _serde::__private::Ok(InstructionWeights {
                            version: __field0,
                            fallback: __field1,
                            i64const: __field2,
                            i64load: __field3,
                            i64store: __field4,
                            select: __field5,
                            r#if: __field6,
                            br: __field7,
                            br_if: __field8,
                            br_table: __field9,
                            br_table_per_entry: __field10,
                            call: __field11,
                            call_indirect: __field12,
                            call_per_local: __field13,
                            local_get: __field14,
                            local_set: __field15,
                            local_tee: __field16,
                            global_get: __field17,
                            global_set: __field18,
                            memory_current: __field19,
                            memory_grow: __field20,
                            i64clz: __field21,
                            i64ctz: __field22,
                            i64popcnt: __field23,
                            i64eqz: __field24,
                            i64extendsi32: __field25,
                            i64extendui32: __field26,
                            i32wrapi64: __field27,
                            i64eq: __field28,
                            i64ne: __field29,
                            i64lts: __field30,
                            i64ltu: __field31,
                            i64gts: __field32,
                            i64gtu: __field33,
                            i64les: __field34,
                            i64leu: __field35,
                            i64ges: __field36,
                            i64geu: __field37,
                            i64add: __field38,
                            i64sub: __field39,
                            i64mul: __field40,
                            i64divs: __field41,
                            i64divu: __field42,
                            i64rems: __field43,
                            i64remu: __field44,
                            i64and: __field45,
                            i64or: __field46,
                            i64xor: __field47,
                            i64shl: __field48,
                            i64shrs: __field49,
                            i64shru: __field50,
                            i64rotl: __field51,
                            i64rotr: __field52,
                            _phantom: __field53,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "version",
                    "fallback",
                    "i64const",
                    "i64load",
                    "i64store",
                    "select",
                    "if",
                    "br",
                    "br_if",
                    "br_table",
                    "br_table_per_entry",
                    "call",
                    "call_indirect",
                    "call_per_local",
                    "local_get",
                    "local_set",
                    "local_tee",
                    "global_get",
                    "global_set",
                    "memory_current",
                    "memory_grow",
                    "i64clz",
                    "i64ctz",
                    "i64popcnt",
                    "i64eqz",
                    "i64extendsi32",
                    "i64extendui32",
                    "i32wrapi64",
                    "i64eq",
                    "i64ne",
                    "i64lts",
                    "i64ltu",
                    "i64gts",
                    "i64gtu",
                    "i64les",
                    "i64leu",
                    "i64ges",
                    "i64geu",
                    "i64add",
                    "i64sub",
                    "i64mul",
                    "i64divs",
                    "i64divu",
                    "i64rems",
                    "i64remu",
                    "i64and",
                    "i64or",
                    "i64xor",
                    "i64shl",
                    "i64shrs",
                    "i64shru",
                    "i64rotl",
                    "i64rotr",
                    "_phantom",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "InstructionWeights",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<InstructionWeights<T>>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    /// Describes the weight for each imported function that a contract is allowed to call.
    #[scale_info(skip_type_params(T))]
    pub struct HostFnWeights<T: Config> {
        /// Weight of calling `seal_caller`.
        pub caller: Weight,
        /// Weight of calling `seal_is_contract`.
        pub is_contract: Weight,
        /// Weight of calling `seal_code_hash`.
        pub code_hash: Weight,
        /// Weight of calling `seal_own_code_hash`.
        pub own_code_hash: Weight,
        /// Weight of calling `seal_caller_is_origin`.
        pub caller_is_origin: Weight,
        /// Weight of calling `seal_caller_is_root`.
        pub caller_is_root: Weight,
        /// Weight of calling `seal_address`.
        pub address: Weight,
        /// Weight of calling `seal_gas_left`.
        pub gas_left: Weight,
        /// Weight of calling `seal_balance`.
        pub balance: Weight,
        /// Weight of calling `seal_value_transferred`.
        pub value_transferred: Weight,
        /// Weight of calling `seal_minimum_balance`.
        pub minimum_balance: Weight,
        /// Weight of calling `seal_block_number`.
        pub block_number: Weight,
        /// Weight of calling `seal_now`.
        pub now: Weight,
        /// Weight of calling `seal_weight_to_fee`.
        pub weight_to_fee: Weight,
        /// Weight of calling `gas`.
        pub gas: Weight,
        /// Weight of calling `seal_input`.
        pub input: Weight,
        /// Weight per input byte copied to contract memory by `seal_input`.
        pub input_per_byte: Weight,
        /// Weight of calling `seal_return`.
        pub r#return: Weight,
        /// Weight per byte returned through `seal_return`.
        pub return_per_byte: Weight,
        /// Weight of calling `seal_terminate`.
        pub terminate: Weight,
        /// Weight of calling `seal_random`.
        pub random: Weight,
        /// Weight of calling `seal_reposit_event`.
        pub deposit_event: Weight,
        /// Weight per topic supplied to `seal_deposit_event`.
        pub deposit_event_per_topic: Weight,
        /// Weight per byte of an event deposited through `seal_deposit_event`.
        pub deposit_event_per_byte: Weight,
        /// Weight of calling `seal_debug_message`.
        pub debug_message: Weight,
        /// Weight of calling `seal_debug_message` per byte of the message.
        pub debug_message_per_byte: Weight,
        /// Weight of calling `seal_set_storage`.
        pub set_storage: Weight,
        /// Weight per written byten of an item stored with `seal_set_storage`.
        pub set_storage_per_new_byte: Weight,
        /// Weight per overwritten byte of an item stored with `seal_set_storage`.
        pub set_storage_per_old_byte: Weight,
        /// Weight of calling `seal_set_code_hash`.
        pub set_code_hash: Weight,
        /// Weight of calling `seal_clear_storage`.
        pub clear_storage: Weight,
        /// Weight of calling `seal_clear_storage` per byte of the stored item.
        pub clear_storage_per_byte: Weight,
        /// Weight of calling `seal_contains_storage`.
        pub contains_storage: Weight,
        /// Weight of calling `seal_contains_storage` per byte of the stored item.
        pub contains_storage_per_byte: Weight,
        /// Weight of calling `seal_get_storage`.
        pub get_storage: Weight,
        /// Weight per byte of an item received via `seal_get_storage`.
        pub get_storage_per_byte: Weight,
        /// Weight of calling `seal_take_storage`.
        pub take_storage: Weight,
        /// Weight per byte of an item received via `seal_take_storage`.
        pub take_storage_per_byte: Weight,
        /// Weight of calling `seal_transfer`.
        pub transfer: Weight,
        /// Weight of calling `seal_call`.
        pub call: Weight,
        /// Weight of calling `seal_delegate_call`.
        pub delegate_call: Weight,
        /// Weight surcharge that is claimed if `seal_call` does a balance transfer.
        pub call_transfer_surcharge: Weight,
        /// Weight per byte that is cloned by supplying the `CLONE_INPUT` flag.
        pub call_per_cloned_byte: Weight,
        /// Weight of calling `seal_instantiate`.
        pub instantiate: Weight,
        /// Weight surcharge that is claimed if `seal_instantiate` does a balance transfer.
        pub instantiate_transfer_surcharge: Weight,
        /// Weight per input byte supplied to `seal_instantiate`.
        pub instantiate_per_input_byte: Weight,
        /// Weight per salt byte supplied to `seal_instantiate`.
        pub instantiate_per_salt_byte: Weight,
        /// Weight of calling `seal_hash_sha_256`.
        pub hash_sha2_256: Weight,
        /// Weight per byte hashed by `seal_hash_sha_256`.
        pub hash_sha2_256_per_byte: Weight,
        /// Weight of calling `seal_hash_keccak_256`.
        pub hash_keccak_256: Weight,
        /// Weight per byte hashed by `seal_hash_keccak_256`.
        pub hash_keccak_256_per_byte: Weight,
        /// Weight of calling `seal_hash_blake2_256`.
        pub hash_blake2_256: Weight,
        /// Weight per byte hashed by `seal_hash_blake2_256`.
        pub hash_blake2_256_per_byte: Weight,
        /// Weight of calling `seal_hash_blake2_128`.
        pub hash_blake2_128: Weight,
        /// Weight per byte hashed by `seal_hash_blake2_128`.
        pub hash_blake2_128_per_byte: Weight,
        /// Weight of calling `seal_ecdsa_recover`.
        pub ecdsa_recover: Weight,
        /// Weight of calling `seal_ecdsa_to_eth_address`.
        pub ecdsa_to_eth_address: Weight,
        /// Weight of calling `sr25519_verify`.
        pub sr25519_verify: Weight,
        /// Weight per byte of calling `sr25519_verify`.
        pub sr25519_verify_per_byte: Weight,
        /// Weight of calling `reentrance_count`.
        pub reentrance_count: Weight,
        /// Weight of calling `account_reentrance_count`.
        pub account_reentrance_count: Weight,
        /// Weight of calling `instantiation_nonce`.
        pub instantiation_nonce: Weight,
        /// The type parameter is used in the default implementation.
        #[codec(skip)]
        pub _phantom: PhantomData<T>,
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone + Config> ::core::clone::Clone for HostFnWeights<T> {
        #[inline]
        fn clone(&self) -> HostFnWeights<T> {
            HostFnWeights {
                caller: ::core::clone::Clone::clone(&self.caller),
                is_contract: ::core::clone::Clone::clone(&self.is_contract),
                code_hash: ::core::clone::Clone::clone(&self.code_hash),
                own_code_hash: ::core::clone::Clone::clone(&self.own_code_hash),
                caller_is_origin: ::core::clone::Clone::clone(&self.caller_is_origin),
                caller_is_root: ::core::clone::Clone::clone(&self.caller_is_root),
                address: ::core::clone::Clone::clone(&self.address),
                gas_left: ::core::clone::Clone::clone(&self.gas_left),
                balance: ::core::clone::Clone::clone(&self.balance),
                value_transferred: ::core::clone::Clone::clone(&self.value_transferred),
                minimum_balance: ::core::clone::Clone::clone(&self.minimum_balance),
                block_number: ::core::clone::Clone::clone(&self.block_number),
                now: ::core::clone::Clone::clone(&self.now),
                weight_to_fee: ::core::clone::Clone::clone(&self.weight_to_fee),
                gas: ::core::clone::Clone::clone(&self.gas),
                input: ::core::clone::Clone::clone(&self.input),
                input_per_byte: ::core::clone::Clone::clone(&self.input_per_byte),
                r#return: ::core::clone::Clone::clone(&self.r#return),
                return_per_byte: ::core::clone::Clone::clone(&self.return_per_byte),
                terminate: ::core::clone::Clone::clone(&self.terminate),
                random: ::core::clone::Clone::clone(&self.random),
                deposit_event: ::core::clone::Clone::clone(&self.deposit_event),
                deposit_event_per_topic: ::core::clone::Clone::clone(&self.deposit_event_per_topic),
                deposit_event_per_byte: ::core::clone::Clone::clone(&self.deposit_event_per_byte),
                debug_message: ::core::clone::Clone::clone(&self.debug_message),
                debug_message_per_byte: ::core::clone::Clone::clone(&self.debug_message_per_byte),
                set_storage: ::core::clone::Clone::clone(&self.set_storage),
                set_storage_per_new_byte: ::core::clone::Clone::clone(
                    &self.set_storage_per_new_byte,
                ),
                set_storage_per_old_byte: ::core::clone::Clone::clone(
                    &self.set_storage_per_old_byte,
                ),
                set_code_hash: ::core::clone::Clone::clone(&self.set_code_hash),
                clear_storage: ::core::clone::Clone::clone(&self.clear_storage),
                clear_storage_per_byte: ::core::clone::Clone::clone(&self.clear_storage_per_byte),
                contains_storage: ::core::clone::Clone::clone(&self.contains_storage),
                contains_storage_per_byte: ::core::clone::Clone::clone(
                    &self.contains_storage_per_byte,
                ),
                get_storage: ::core::clone::Clone::clone(&self.get_storage),
                get_storage_per_byte: ::core::clone::Clone::clone(&self.get_storage_per_byte),
                take_storage: ::core::clone::Clone::clone(&self.take_storage),
                take_storage_per_byte: ::core::clone::Clone::clone(&self.take_storage_per_byte),
                transfer: ::core::clone::Clone::clone(&self.transfer),
                call: ::core::clone::Clone::clone(&self.call),
                delegate_call: ::core::clone::Clone::clone(&self.delegate_call),
                call_transfer_surcharge: ::core::clone::Clone::clone(&self.call_transfer_surcharge),
                call_per_cloned_byte: ::core::clone::Clone::clone(&self.call_per_cloned_byte),
                instantiate: ::core::clone::Clone::clone(&self.instantiate),
                instantiate_transfer_surcharge: ::core::clone::Clone::clone(
                    &self.instantiate_transfer_surcharge,
                ),
                instantiate_per_input_byte: ::core::clone::Clone::clone(
                    &self.instantiate_per_input_byte,
                ),
                instantiate_per_salt_byte: ::core::clone::Clone::clone(
                    &self.instantiate_per_salt_byte,
                ),
                hash_sha2_256: ::core::clone::Clone::clone(&self.hash_sha2_256),
                hash_sha2_256_per_byte: ::core::clone::Clone::clone(&self.hash_sha2_256_per_byte),
                hash_keccak_256: ::core::clone::Clone::clone(&self.hash_keccak_256),
                hash_keccak_256_per_byte: ::core::clone::Clone::clone(
                    &self.hash_keccak_256_per_byte,
                ),
                hash_blake2_256: ::core::clone::Clone::clone(&self.hash_blake2_256),
                hash_blake2_256_per_byte: ::core::clone::Clone::clone(
                    &self.hash_blake2_256_per_byte,
                ),
                hash_blake2_128: ::core::clone::Clone::clone(&self.hash_blake2_128),
                hash_blake2_128_per_byte: ::core::clone::Clone::clone(
                    &self.hash_blake2_128_per_byte,
                ),
                ecdsa_recover: ::core::clone::Clone::clone(&self.ecdsa_recover),
                ecdsa_to_eth_address: ::core::clone::Clone::clone(&self.ecdsa_to_eth_address),
                sr25519_verify: ::core::clone::Clone::clone(&self.sr25519_verify),
                sr25519_verify_per_byte: ::core::clone::Clone::clone(&self.sr25519_verify_per_byte),
                reentrance_count: ::core::clone::Clone::clone(&self.reentrance_count),
                account_reentrance_count: ::core::clone::Clone::clone(
                    &self.account_reentrance_count,
                ),
                instantiation_nonce: ::core::clone::Clone::clone(&self.instantiation_nonce),
                _phantom: ::core::clone::Clone::clone(&self._phantom),
            }
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Encode for HostFnWeights<T> {
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::codec::Encode::encode_to(&self.caller, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.is_contract, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.code_hash, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.own_code_hash, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.caller_is_origin, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.caller_is_root, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.address, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.gas_left, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.balance, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.value_transferred, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.minimum_balance, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.block_number, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.now, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.weight_to_fee, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.gas, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.input, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.input_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.r#return, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.return_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.terminate, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.random, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.deposit_event, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.deposit_event_per_topic, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.deposit_event_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.debug_message, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.debug_message_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.set_storage, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.set_storage_per_new_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.set_storage_per_old_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.set_code_hash, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.clear_storage, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.clear_storage_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.contains_storage, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.contains_storage_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.get_storage, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.get_storage_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.take_storage, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.take_storage_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.transfer, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.call, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.delegate_call, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.call_transfer_surcharge, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.call_per_cloned_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.instantiate, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.instantiate_transfer_surcharge, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.instantiate_per_input_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.instantiate_per_salt_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_sha2_256, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_sha2_256_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_keccak_256, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_keccak_256_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_blake2_256, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_blake2_256_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_blake2_128, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.hash_blake2_128_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.ecdsa_recover, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.ecdsa_to_eth_address, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.sr25519_verify, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.sr25519_verify_per_byte, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.reentrance_count, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.account_reentrance_count, __codec_dest_edqy);
                ::codec::Encode::encode_to(&self.instantiation_nonce, __codec_dest_edqy);
                let _ = &self._phantom;
            }
        }
        #[automatically_derived]
        impl<T: Config> ::codec::EncodeLike for HostFnWeights<T> {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Decode for HostFnWeights<T>
        where
            PhantomData<T>: Default,
        {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                ::core::result::Result::Ok(HostFnWeights::<T> {
                    caller: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::caller`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    is_contract: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::is_contract`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    code_hash: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::code_hash`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    own_code_hash: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::own_code_hash`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    caller_is_origin: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::caller_is_origin`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    caller_is_root: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::caller_is_root`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    address: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::address`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    gas_left: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::gas_left`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    balance: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::balance`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    value_transferred: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::value_transferred`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    minimum_balance: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::minimum_balance`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    block_number: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::block_number`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    now: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::now`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    weight_to_fee: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::weight_to_fee`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    gas: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::gas`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    input: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::input`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    input_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::input_per_byte`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    r#return: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::r#return`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    return_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::return_per_byte`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    terminate: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::terminate`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    random: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::random`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    deposit_event: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::deposit_event`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    deposit_event_per_topic: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::deposit_event_per_topic`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    deposit_event_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::deposit_event_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    debug_message: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::debug_message`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    debug_message_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::debug_message_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    set_storage: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::set_storage`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    set_storage_per_new_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::set_storage_per_new_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    set_storage_per_old_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::set_storage_per_old_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    set_code_hash: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::set_code_hash`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    clear_storage: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::clear_storage`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    clear_storage_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::clear_storage_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    contains_storage: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::contains_storage`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    contains_storage_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::contains_storage_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    get_storage: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::get_storage`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    get_storage_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::get_storage_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    take_storage: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::take_storage`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    take_storage_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::take_storage_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    transfer: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::transfer`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    call: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::call`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    delegate_call: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::delegate_call`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    call_transfer_surcharge: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::call_transfer_surcharge`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    call_per_cloned_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::call_per_cloned_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    instantiate: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::instantiate`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    instantiate_transfer_surcharge: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy { :: core :: result :: Result :: Err (e) => return :: core :: result :: Result :: Err (e . chain ("Could not decode `HostFnWeights::instantiate_transfer_surcharge`")) , :: core :: result :: Result :: Ok (__codec_res_edqy) => __codec_res_edqy , }
                    },
                    instantiate_per_input_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::instantiate_per_input_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    instantiate_per_salt_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::instantiate_per_salt_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_sha2_256: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::hash_sha2_256`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_sha2_256_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::hash_sha2_256_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_keccak_256: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::hash_keccak_256`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_keccak_256_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::hash_keccak_256_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_blake2_256: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::hash_blake2_256`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_blake2_256_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::hash_blake2_256_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_blake2_128: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::hash_blake2_128`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    hash_blake2_128_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::hash_blake2_128_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    ecdsa_recover: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::ecdsa_recover`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    ecdsa_to_eth_address: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::ecdsa_to_eth_address`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    sr25519_verify: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::sr25519_verify`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    sr25519_verify_per_byte: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::sr25519_verify_per_byte`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    reentrance_count: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `HostFnWeights::reentrance_count`"),
                                )
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    account_reentrance_count: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::account_reentrance_count`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    instantiation_nonce: {
                        let __codec_res_edqy =
                            <Weight as ::codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(e.chain(
                                    "Could not decode `HostFnWeights::instantiation_nonce`",
                                ))
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    _phantom: ::core::default::Default::default(),
                })
            }
        }
    };
    #[automatically_derived]
    impl<T: Config> ::core::marker::StructuralPartialEq for HostFnWeights<T> {}
    #[automatically_derived]
    impl<T: ::core::cmp::PartialEq + Config> ::core::cmp::PartialEq for HostFnWeights<T> {
        #[inline]
        fn eq(&self, other: &HostFnWeights<T>) -> bool {
            self.caller == other.caller
                && self.is_contract == other.is_contract
                && self.code_hash == other.code_hash
                && self.own_code_hash == other.own_code_hash
                && self.caller_is_origin == other.caller_is_origin
                && self.caller_is_root == other.caller_is_root
                && self.address == other.address
                && self.gas_left == other.gas_left
                && self.balance == other.balance
                && self.value_transferred == other.value_transferred
                && self.minimum_balance == other.minimum_balance
                && self.block_number == other.block_number
                && self.now == other.now
                && self.weight_to_fee == other.weight_to_fee
                && self.gas == other.gas
                && self.input == other.input
                && self.input_per_byte == other.input_per_byte
                && self.r#return == other.r#return
                && self.return_per_byte == other.return_per_byte
                && self.terminate == other.terminate
                && self.random == other.random
                && self.deposit_event == other.deposit_event
                && self.deposit_event_per_topic == other.deposit_event_per_topic
                && self.deposit_event_per_byte == other.deposit_event_per_byte
                && self.debug_message == other.debug_message
                && self.debug_message_per_byte == other.debug_message_per_byte
                && self.set_storage == other.set_storage
                && self.set_storage_per_new_byte == other.set_storage_per_new_byte
                && self.set_storage_per_old_byte == other.set_storage_per_old_byte
                && self.set_code_hash == other.set_code_hash
                && self.clear_storage == other.clear_storage
                && self.clear_storage_per_byte == other.clear_storage_per_byte
                && self.contains_storage == other.contains_storage
                && self.contains_storage_per_byte == other.contains_storage_per_byte
                && self.get_storage == other.get_storage
                && self.get_storage_per_byte == other.get_storage_per_byte
                && self.take_storage == other.take_storage
                && self.take_storage_per_byte == other.take_storage_per_byte
                && self.transfer == other.transfer
                && self.call == other.call
                && self.delegate_call == other.delegate_call
                && self.call_transfer_surcharge == other.call_transfer_surcharge
                && self.call_per_cloned_byte == other.call_per_cloned_byte
                && self.instantiate == other.instantiate
                && self.instantiate_transfer_surcharge == other.instantiate_transfer_surcharge
                && self.instantiate_per_input_byte == other.instantiate_per_input_byte
                && self.instantiate_per_salt_byte == other.instantiate_per_salt_byte
                && self.hash_sha2_256 == other.hash_sha2_256
                && self.hash_sha2_256_per_byte == other.hash_sha2_256_per_byte
                && self.hash_keccak_256 == other.hash_keccak_256
                && self.hash_keccak_256_per_byte == other.hash_keccak_256_per_byte
                && self.hash_blake2_256 == other.hash_blake2_256
                && self.hash_blake2_256_per_byte == other.hash_blake2_256_per_byte
                && self.hash_blake2_128 == other.hash_blake2_128
                && self.hash_blake2_128_per_byte == other.hash_blake2_128_per_byte
                && self.ecdsa_recover == other.ecdsa_recover
                && self.ecdsa_to_eth_address == other.ecdsa_to_eth_address
                && self.sr25519_verify == other.sr25519_verify
                && self.sr25519_verify_per_byte == other.sr25519_verify_per_byte
                && self.reentrance_count == other.reentrance_count
                && self.account_reentrance_count == other.account_reentrance_count
                && self.instantiation_nonce == other.instantiation_nonce
                && self._phantom == other._phantom
        }
    }
    #[automatically_derived]
    impl<T: Config> ::core::marker::StructuralEq for HostFnWeights<T> {}
    #[automatically_derived]
    impl<T: ::core::cmp::Eq + Config> ::core::cmp::Eq for HostFnWeights<T> {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Weight>;
            let _: ::core::cmp::AssertParamIsEq<PhantomData<T>>;
        }
    }
    impl<T: Config> core::fmt::Debug for HostFnWeights<T> {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ::sp_runtime::{FixedPointNumber, FixedU128 as Fixed};
            let mut formatter = formatter.debug_struct("HostFnWeights");
            formatter.field(
                "caller",
                &if self.caller.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.caller.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.caller.proof_size()
                        ));
                        res
                    }
                } else if self.caller.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.caller.ref_time(), 1_000_000)
                                .to_float(),
                            self.caller.proof_size()
                        ));
                        res
                    }
                } else if self.caller.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.caller.ref_time(), 1_000)
                                .to_float(),
                            self.caller.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.caller.ref_time(),
                            self.caller.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "is_contract",
                &if self.is_contract.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.is_contract.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.is_contract.proof_size()
                        ));
                        res
                    }
                } else if self.is_contract.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.is_contract.ref_time(), 1_000_000)
                                .to_float(),
                            self.is_contract.proof_size()
                        ));
                        res
                    }
                } else if self.is_contract.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.is_contract.ref_time(), 1_000)
                                .to_float(),
                            self.is_contract.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.is_contract.ref_time(),
                            self.is_contract.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "code_hash",
                &if self.code_hash.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.code_hash.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.code_hash.proof_size()
                        ));
                        res
                    }
                } else if self.code_hash.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.code_hash.ref_time(), 1_000_000)
                                .to_float(),
                            self.code_hash.proof_size()
                        ));
                        res
                    }
                } else if self.code_hash.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.code_hash.ref_time(), 1_000)
                                .to_float(),
                            self.code_hash.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.code_hash.ref_time(),
                            self.code_hash.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "own_code_hash",
                &if self.own_code_hash.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.own_code_hash.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.own_code_hash.proof_size()
                        ));
                        res
                    }
                } else if self.own_code_hash.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.own_code_hash.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.own_code_hash.proof_size()
                        ));
                        res
                    }
                } else if self.own_code_hash.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.own_code_hash.ref_time(), 1_000)
                                .to_float(),
                            self.own_code_hash.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.own_code_hash.ref_time(),
                            self.own_code_hash.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "caller_is_origin",
                &if self.caller_is_origin.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.caller_is_origin.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.caller_is_origin.proof_size()
                        ));
                        res
                    }
                } else if self.caller_is_origin.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.caller_is_origin.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.caller_is_origin.proof_size()
                        ));
                        res
                    }
                } else if self.caller_is_origin.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.caller_is_origin.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.caller_is_origin.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.caller_is_origin.ref_time(),
                            self.caller_is_origin.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "caller_is_root",
                &if self.caller_is_root.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.caller_is_root.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.caller_is_root.proof_size()
                        ));
                        res
                    }
                } else if self.caller_is_root.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.caller_is_root.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.caller_is_root.proof_size()
                        ));
                        res
                    }
                } else if self.caller_is_root.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.caller_is_root.ref_time(), 1_000)
                                .to_float(),
                            self.caller_is_root.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.caller_is_root.ref_time(),
                            self.caller_is_root.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "address",
                &if self.address.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.address.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.address.proof_size()
                        ));
                        res
                    }
                } else if self.address.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.address.ref_time(), 1_000_000)
                                .to_float(),
                            self.address.proof_size()
                        ));
                        res
                    }
                } else if self.address.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.address.ref_time(), 1_000)
                                .to_float(),
                            self.address.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.address.ref_time(),
                            self.address.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "gas_left",
                &if self.gas_left.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.gas_left.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.gas_left.proof_size()
                        ));
                        res
                    }
                } else if self.gas_left.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.gas_left.ref_time(), 1_000_000)
                                .to_float(),
                            self.gas_left.proof_size()
                        ));
                        res
                    }
                } else if self.gas_left.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.gas_left.ref_time(), 1_000)
                                .to_float(),
                            self.gas_left.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.gas_left.ref_time(),
                            self.gas_left.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "balance",
                &if self.balance.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.balance.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.balance.proof_size()
                        ));
                        res
                    }
                } else if self.balance.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.balance.ref_time(), 1_000_000)
                                .to_float(),
                            self.balance.proof_size()
                        ));
                        res
                    }
                } else if self.balance.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.balance.ref_time(), 1_000)
                                .to_float(),
                            self.balance.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.balance.ref_time(),
                            self.balance.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "value_transferred",
                &if self.value_transferred.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.value_transferred.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.value_transferred.proof_size()
                        ));
                        res
                    }
                } else if self.value_transferred.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.value_transferred.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.value_transferred.proof_size()
                        ));
                        res
                    }
                } else if self.value_transferred.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.value_transferred.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.value_transferred.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.value_transferred.ref_time(),
                            self.value_transferred.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "minimum_balance",
                &if self.minimum_balance.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.minimum_balance.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.minimum_balance.proof_size()
                        ));
                        res
                    }
                } else if self.minimum_balance.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.minimum_balance.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.minimum_balance.proof_size()
                        ));
                        res
                    }
                } else if self.minimum_balance.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.minimum_balance.ref_time(), 1_000)
                                .to_float(),
                            self.minimum_balance.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.minimum_balance.ref_time(),
                            self.minimum_balance.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "block_number",
                &if self.block_number.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.block_number.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.block_number.proof_size()
                        ));
                        res
                    }
                } else if self.block_number.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.block_number.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.block_number.proof_size()
                        ));
                        res
                    }
                } else if self.block_number.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.block_number.ref_time(), 1_000)
                                .to_float(),
                            self.block_number.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.block_number.ref_time(),
                            self.block_number.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "now",
                &if self.now.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.now.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.now.proof_size()
                        ));
                        res
                    }
                } else if self.now.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.now.ref_time(), 1_000_000)
                                .to_float(),
                            self.now.proof_size()
                        ));
                        res
                    }
                } else if self.now.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.now.ref_time(), 1_000).to_float(),
                            self.now.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.now.ref_time(),
                            self.now.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "weight_to_fee",
                &if self.weight_to_fee.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.weight_to_fee.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.weight_to_fee.proof_size()
                        ));
                        res
                    }
                } else if self.weight_to_fee.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.weight_to_fee.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.weight_to_fee.proof_size()
                        ));
                        res
                    }
                } else if self.weight_to_fee.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.weight_to_fee.ref_time(), 1_000)
                                .to_float(),
                            self.weight_to_fee.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.weight_to_fee.ref_time(),
                            self.weight_to_fee.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "gas",
                &if self.gas.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.gas.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.gas.proof_size()
                        ));
                        res
                    }
                } else if self.gas.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.gas.ref_time(), 1_000_000)
                                .to_float(),
                            self.gas.proof_size()
                        ));
                        res
                    }
                } else if self.gas.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.gas.ref_time(), 1_000).to_float(),
                            self.gas.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.gas.ref_time(),
                            self.gas.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "input",
                &if self.input.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.input.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.input.proof_size()
                        ));
                        res
                    }
                } else if self.input.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.input.ref_time(), 1_000_000)
                                .to_float(),
                            self.input.proof_size()
                        ));
                        res
                    }
                } else if self.input.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.input.ref_time(), 1_000)
                                .to_float(),
                            self.input.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.input.ref_time(),
                            self.input.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "input_per_byte",
                &if self.input_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.input_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.input_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.input_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.input_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.input_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.input_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.input_per_byte.ref_time(), 1_000)
                                .to_float(),
                            self.input_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.input_per_byte.ref_time(),
                            self.input_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "r#return",
                &if self.r#return.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.r#return.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.r#return.proof_size()
                        ));
                        res
                    }
                } else if self.r#return.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.r#return.ref_time(), 1_000_000)
                                .to_float(),
                            self.r#return.proof_size()
                        ));
                        res
                    }
                } else if self.r#return.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.r#return.ref_time(), 1_000)
                                .to_float(),
                            self.r#return.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.r#return.ref_time(),
                            self.r#return.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "return_per_byte",
                &if self.return_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.return_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.return_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.return_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.return_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.return_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.return_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.return_per_byte.ref_time(), 1_000)
                                .to_float(),
                            self.return_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.return_per_byte.ref_time(),
                            self.return_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "terminate",
                &if self.terminate.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.terminate.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.terminate.proof_size()
                        ));
                        res
                    }
                } else if self.terminate.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.terminate.ref_time(), 1_000_000)
                                .to_float(),
                            self.terminate.proof_size()
                        ));
                        res
                    }
                } else if self.terminate.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.terminate.ref_time(), 1_000)
                                .to_float(),
                            self.terminate.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.terminate.ref_time(),
                            self.terminate.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "random",
                &if self.random.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.random.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.random.proof_size()
                        ));
                        res
                    }
                } else if self.random.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.random.ref_time(), 1_000_000)
                                .to_float(),
                            self.random.proof_size()
                        ));
                        res
                    }
                } else if self.random.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.random.ref_time(), 1_000)
                                .to_float(),
                            self.random.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.random.ref_time(),
                            self.random.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "deposit_event",
                &if self.deposit_event.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.deposit_event.proof_size()
                        ));
                        res
                    }
                } else if self.deposit_event.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.deposit_event.proof_size()
                        ));
                        res
                    }
                } else if self.deposit_event.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.deposit_event.ref_time(), 1_000)
                                .to_float(),
                            self.deposit_event.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.deposit_event.ref_time(),
                            self.deposit_event.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "deposit_event_per_topic",
                &if self.deposit_event_per_topic.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event_per_topic.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.deposit_event_per_topic.proof_size()
                        ));
                        res
                    }
                } else if self.deposit_event_per_topic.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event_per_topic.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.deposit_event_per_topic.proof_size()
                        ));
                        res
                    }
                } else if self.deposit_event_per_topic.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event_per_topic.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.deposit_event_per_topic.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.deposit_event_per_topic.ref_time(),
                            self.deposit_event_per_topic.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "deposit_event_per_byte",
                &if self.deposit_event_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.deposit_event_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.deposit_event_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.deposit_event_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.deposit_event_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.deposit_event_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.deposit_event_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.deposit_event_per_byte.ref_time(),
                            self.deposit_event_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "debug_message",
                &if self.debug_message.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.debug_message.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.debug_message.proof_size()
                        ));
                        res
                    }
                } else if self.debug_message.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.debug_message.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.debug_message.proof_size()
                        ));
                        res
                    }
                } else if self.debug_message.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.debug_message.ref_time(), 1_000)
                                .to_float(),
                            self.debug_message.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.debug_message.ref_time(),
                            self.debug_message.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "debug_message_per_byte",
                &if self.debug_message_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.debug_message_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.debug_message_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.debug_message_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.debug_message_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.debug_message_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.debug_message_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.debug_message_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.debug_message_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.debug_message_per_byte.ref_time(),
                            self.debug_message_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "set_storage",
                &if self.set_storage.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.set_storage.proof_size()
                        ));
                        res
                    }
                } else if self.set_storage.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.set_storage.ref_time(), 1_000_000)
                                .to_float(),
                            self.set_storage.proof_size()
                        ));
                        res
                    }
                } else if self.set_storage.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.set_storage.ref_time(), 1_000)
                                .to_float(),
                            self.set_storage.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.set_storage.ref_time(),
                            self.set_storage.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "set_storage_per_new_byte",
                &if self.set_storage_per_new_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage_per_new_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.set_storage_per_new_byte.proof_size()
                        ));
                        res
                    }
                } else if self.set_storage_per_new_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage_per_new_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.set_storage_per_new_byte.proof_size()
                        ));
                        res
                    }
                } else if self.set_storage_per_new_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage_per_new_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.set_storage_per_new_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.set_storage_per_new_byte.ref_time(),
                            self.set_storage_per_new_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "set_storage_per_old_byte",
                &if self.set_storage_per_old_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage_per_old_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.set_storage_per_old_byte.proof_size()
                        ));
                        res
                    }
                } else if self.set_storage_per_old_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage_per_old_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.set_storage_per_old_byte.proof_size()
                        ));
                        res
                    }
                } else if self.set_storage_per_old_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_storage_per_old_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.set_storage_per_old_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.set_storage_per_old_byte.ref_time(),
                            self.set_storage_per_old_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "set_code_hash",
                &if self.set_code_hash.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_code_hash.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.set_code_hash.proof_size()
                        ));
                        res
                    }
                } else if self.set_code_hash.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.set_code_hash.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.set_code_hash.proof_size()
                        ));
                        res
                    }
                } else if self.set_code_hash.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.set_code_hash.ref_time(), 1_000)
                                .to_float(),
                            self.set_code_hash.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.set_code_hash.ref_time(),
                            self.set_code_hash.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "clear_storage",
                &if self.clear_storage.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.clear_storage.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.clear_storage.proof_size()
                        ));
                        res
                    }
                } else if self.clear_storage.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.clear_storage.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.clear_storage.proof_size()
                        ));
                        res
                    }
                } else if self.clear_storage.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.clear_storage.ref_time(), 1_000)
                                .to_float(),
                            self.clear_storage.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.clear_storage.ref_time(),
                            self.clear_storage.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "clear_storage_per_byte",
                &if self.clear_storage_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.clear_storage_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.clear_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.clear_storage_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.clear_storage_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.clear_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.clear_storage_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.clear_storage_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.clear_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.clear_storage_per_byte.ref_time(),
                            self.clear_storage_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "contains_storage",
                &if self.contains_storage.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.contains_storage.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.contains_storage.proof_size()
                        ));
                        res
                    }
                } else if self.contains_storage.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.contains_storage.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.contains_storage.proof_size()
                        ));
                        res
                    }
                } else if self.contains_storage.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.contains_storage.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.contains_storage.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.contains_storage.ref_time(),
                            self.contains_storage.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "contains_storage_per_byte",
                &if self.contains_storage_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.contains_storage_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.contains_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.contains_storage_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.contains_storage_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.contains_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.contains_storage_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.contains_storage_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.contains_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.contains_storage_per_byte.ref_time(),
                            self.contains_storage_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "get_storage",
                &if self.get_storage.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.get_storage.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.get_storage.proof_size()
                        ));
                        res
                    }
                } else if self.get_storage.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.get_storage.ref_time(), 1_000_000)
                                .to_float(),
                            self.get_storage.proof_size()
                        ));
                        res
                    }
                } else if self.get_storage.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.get_storage.ref_time(), 1_000)
                                .to_float(),
                            self.get_storage.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.get_storage.ref_time(),
                            self.get_storage.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "get_storage_per_byte",
                &if self.get_storage_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.get_storage_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.get_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.get_storage_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.get_storage_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.get_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.get_storage_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.get_storage_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.get_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.get_storage_per_byte.ref_time(),
                            self.get_storage_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "take_storage",
                &if self.take_storage.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.take_storage.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.take_storage.proof_size()
                        ));
                        res
                    }
                } else if self.take_storage.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.take_storage.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.take_storage.proof_size()
                        ));
                        res
                    }
                } else if self.take_storage.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.take_storage.ref_time(), 1_000)
                                .to_float(),
                            self.take_storage.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.take_storage.ref_time(),
                            self.take_storage.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "take_storage_per_byte",
                &if self.take_storage_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.take_storage_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.take_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.take_storage_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.take_storage_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.take_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.take_storage_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.take_storage_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.take_storage_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.take_storage_per_byte.ref_time(),
                            self.take_storage_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "transfer",
                &if self.transfer.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.transfer.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.transfer.proof_size()
                        ));
                        res
                    }
                } else if self.transfer.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.transfer.ref_time(), 1_000_000)
                                .to_float(),
                            self.transfer.proof_size()
                        ));
                        res
                    }
                } else if self.transfer.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.transfer.ref_time(), 1_000)
                                .to_float(),
                            self.transfer.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.transfer.ref_time(),
                            self.transfer.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "call",
                &if self.call.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(self.call.ref_time(), 1_000_000_000)
                                .to_float(),
                            self.call.proof_size()
                        ));
                        res
                    }
                } else if self.call.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.call.ref_time(), 1_000_000)
                                .to_float(),
                            self.call.proof_size()
                        ));
                        res
                    }
                } else if self.call.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.call.ref_time(), 1_000).to_float(),
                            self.call.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.call.ref_time(),
                            self.call.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "delegate_call",
                &if self.delegate_call.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.delegate_call.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.delegate_call.proof_size()
                        ));
                        res
                    }
                } else if self.delegate_call.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.delegate_call.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.delegate_call.proof_size()
                        ));
                        res
                    }
                } else if self.delegate_call.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.delegate_call.ref_time(), 1_000)
                                .to_float(),
                            self.delegate_call.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.delegate_call.ref_time(),
                            self.delegate_call.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "call_transfer_surcharge",
                &if self.call_transfer_surcharge.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.call_transfer_surcharge.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.call_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                } else if self.call_transfer_surcharge.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.call_transfer_surcharge.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.call_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                } else if self.call_transfer_surcharge.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.call_transfer_surcharge.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.call_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.call_transfer_surcharge.ref_time(),
                            self.call_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "call_per_cloned_byte",
                &if self.call_per_cloned_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.call_per_cloned_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.call_per_cloned_byte.proof_size()
                        ));
                        res
                    }
                } else if self.call_per_cloned_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.call_per_cloned_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.call_per_cloned_byte.proof_size()
                        ));
                        res
                    }
                } else if self.call_per_cloned_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.call_per_cloned_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.call_per_cloned_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.call_per_cloned_byte.ref_time(),
                            self.call_per_cloned_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "instantiate",
                &if self.instantiate.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.instantiate.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(self.instantiate.ref_time(), 1_000_000)
                                .to_float(),
                            self.instantiate.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.instantiate.ref_time(), 1_000)
                                .to_float(),
                            self.instantiate.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.instantiate.ref_time(),
                            self.instantiate.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "instantiate_transfer_surcharge",
                &if self.instantiate_transfer_surcharge.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_transfer_surcharge.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.instantiate_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate_transfer_surcharge.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_transfer_surcharge.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.instantiate_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate_transfer_surcharge.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_transfer_surcharge.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.instantiate_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.instantiate_transfer_surcharge.ref_time(),
                            self.instantiate_transfer_surcharge.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "instantiate_per_input_byte",
                &if self.instantiate_per_input_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_per_input_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.instantiate_per_input_byte.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate_per_input_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_per_input_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.instantiate_per_input_byte.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate_per_input_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_per_input_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.instantiate_per_input_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.instantiate_per_input_byte.ref_time(),
                            self.instantiate_per_input_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "instantiate_per_salt_byte",
                &if self.instantiate_per_salt_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_per_salt_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.instantiate_per_salt_byte.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate_per_salt_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_per_salt_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.instantiate_per_salt_byte.proof_size()
                        ));
                        res
                    }
                } else if self.instantiate_per_salt_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiate_per_salt_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.instantiate_per_salt_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.instantiate_per_salt_byte.ref_time(),
                            self.instantiate_per_salt_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_sha2_256",
                &if self.hash_sha2_256.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_sha2_256.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_sha2_256.proof_size()
                        ));
                        res
                    }
                } else if self.hash_sha2_256.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_sha2_256.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_sha2_256.proof_size()
                        ));
                        res
                    }
                } else if self.hash_sha2_256.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.hash_sha2_256.ref_time(), 1_000)
                                .to_float(),
                            self.hash_sha2_256.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_sha2_256.ref_time(),
                            self.hash_sha2_256.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_sha2_256_per_byte",
                &if self.hash_sha2_256_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_sha2_256_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_sha2_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_sha2_256_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_sha2_256_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_sha2_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_sha2_256_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_sha2_256_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.hash_sha2_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_sha2_256_per_byte.ref_time(),
                            self.hash_sha2_256_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_keccak_256",
                &if self.hash_keccak_256.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_keccak_256.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_keccak_256.proof_size()
                        ));
                        res
                    }
                } else if self.hash_keccak_256.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_keccak_256.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_keccak_256.proof_size()
                        ));
                        res
                    }
                } else if self.hash_keccak_256.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.hash_keccak_256.ref_time(), 1_000)
                                .to_float(),
                            self.hash_keccak_256.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_keccak_256.ref_time(),
                            self.hash_keccak_256.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_keccak_256_per_byte",
                &if self.hash_keccak_256_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_keccak_256_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_keccak_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_keccak_256_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_keccak_256_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_keccak_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_keccak_256_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_keccak_256_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.hash_keccak_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_keccak_256_per_byte.ref_time(),
                            self.hash_keccak_256_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_blake2_256",
                &if self.hash_blake2_256.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_256.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_blake2_256.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_256.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_256.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_blake2_256.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_256.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.hash_blake2_256.ref_time(), 1_000)
                                .to_float(),
                            self.hash_blake2_256.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_blake2_256.ref_time(),
                            self.hash_blake2_256.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_blake2_256_per_byte",
                &if self.hash_blake2_256_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_256_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_blake2_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_256_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_256_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_blake2_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_256_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_256_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.hash_blake2_256_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_blake2_256_per_byte.ref_time(),
                            self.hash_blake2_256_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_blake2_128",
                &if self.hash_blake2_128.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_128.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_blake2_128.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_128.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_128.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_blake2_128.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_128.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.hash_blake2_128.ref_time(), 1_000)
                                .to_float(),
                            self.hash_blake2_128.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_blake2_128.ref_time(),
                            self.hash_blake2_128.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "hash_blake2_128_per_byte",
                &if self.hash_blake2_128_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_128_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.hash_blake2_128_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_128_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_128_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.hash_blake2_128_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.hash_blake2_128_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.hash_blake2_128_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.hash_blake2_128_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.hash_blake2_128_per_byte.ref_time(),
                            self.hash_blake2_128_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "ecdsa_recover",
                &if self.ecdsa_recover.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.ecdsa_recover.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.ecdsa_recover.proof_size()
                        ));
                        res
                    }
                } else if self.ecdsa_recover.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.ecdsa_recover.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.ecdsa_recover.proof_size()
                        ));
                        res
                    }
                } else if self.ecdsa_recover.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.ecdsa_recover.ref_time(), 1_000)
                                .to_float(),
                            self.ecdsa_recover.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.ecdsa_recover.ref_time(),
                            self.ecdsa_recover.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "ecdsa_to_eth_address",
                &if self.ecdsa_to_eth_address.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.ecdsa_to_eth_address.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.ecdsa_to_eth_address.proof_size()
                        ));
                        res
                    }
                } else if self.ecdsa_to_eth_address.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.ecdsa_to_eth_address.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.ecdsa_to_eth_address.proof_size()
                        ));
                        res
                    }
                } else if self.ecdsa_to_eth_address.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.ecdsa_to_eth_address.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.ecdsa_to_eth_address.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.ecdsa_to_eth_address.ref_time(),
                            self.ecdsa_to_eth_address.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "sr25519_verify",
                &if self.sr25519_verify.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.sr25519_verify.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.sr25519_verify.proof_size()
                        ));
                        res
                    }
                } else if self.sr25519_verify.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.sr25519_verify.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.sr25519_verify.proof_size()
                        ));
                        res
                    }
                } else if self.sr25519_verify.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(self.sr25519_verify.ref_time(), 1_000)
                                .to_float(),
                            self.sr25519_verify.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.sr25519_verify.ref_time(),
                            self.sr25519_verify.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "sr25519_verify_per_byte",
                &if self.sr25519_verify_per_byte.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.sr25519_verify_per_byte.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.sr25519_verify_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.sr25519_verify_per_byte.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.sr25519_verify_per_byte.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.sr25519_verify_per_byte.proof_size()
                        ));
                        res
                    }
                } else if self.sr25519_verify_per_byte.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.sr25519_verify_per_byte.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.sr25519_verify_per_byte.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.sr25519_verify_per_byte.ref_time(),
                            self.sr25519_verify_per_byte.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "reentrance_count",
                &if self.reentrance_count.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.reentrance_count.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.reentrance_count.proof_size()
                        ));
                        res
                    }
                } else if self.reentrance_count.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.reentrance_count.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.reentrance_count.proof_size()
                        ));
                        res
                    }
                } else if self.reentrance_count.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.reentrance_count.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.reentrance_count.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.reentrance_count.ref_time(),
                            self.reentrance_count.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "account_reentrance_count",
                &if self.account_reentrance_count.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.account_reentrance_count.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.account_reentrance_count.proof_size()
                        ));
                        res
                    }
                } else if self.account_reentrance_count.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.account_reentrance_count.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.account_reentrance_count.proof_size()
                        ));
                        res
                    }
                } else if self.account_reentrance_count.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.account_reentrance_count.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.account_reentrance_count.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.account_reentrance_count.ref_time(),
                            self.account_reentrance_count.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.field(
                "instantiation_nonce",
                &if self.instantiation_nonce.ref_time() > 1_000_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ms, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiation_nonce.ref_time(),
                                1_000_000_000
                            )
                            .to_float(),
                            self.instantiation_nonce.proof_size()
                        ));
                        res
                    }
                } else if self.instantiation_nonce.ref_time() > 1_000_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} µs, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiation_nonce.ref_time(),
                                1_000_000
                            )
                            .to_float(),
                            self.instantiation_nonce.proof_size()
                        ));
                        res
                    }
                } else if self.instantiation_nonce.ref_time() > 1_000 {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0:.1?} ns, {1} bytes",
                            Fixed::saturating_from_rational(
                                self.instantiation_nonce.ref_time(),
                                1_000
                            )
                            .to_float(),
                            self.instantiation_nonce.proof_size()
                        ));
                        res
                    }
                } else {
                    {
                        let res = ::alloc::fmt::format(format_args!(
                            "{0} ps, {1} bytes",
                            self.instantiation_nonce.ref_time(),
                            self.instantiation_nonce.proof_size()
                        ));
                        res
                    }
                },
            );
            formatter.finish()
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T: Config> ::scale_info::TypeInfo for HostFnWeights<T>
        where
            PhantomData<T>: ::scale_info::TypeInfo + 'static,
            T: Config + 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                :: scale_info :: Type :: builder () . path (:: scale_info :: Path :: new ("HostFnWeights" , "pallet_contracts::schedule")) . type_params (< [_] > :: into_vec (# [rustc_box] :: alloc :: boxed :: Box :: new ([:: scale_info :: TypeParameter :: new ("T" , :: core :: option :: Option :: None)]))) . docs (& ["Describes the weight for each imported function that a contract is allowed to call."]) . composite (:: scale_info :: build :: Fields :: named () . field (| f | f . ty :: < Weight > () . name ("caller") . type_name ("Weight") . docs (& ["Weight of calling `seal_caller`."])) . field (| f | f . ty :: < Weight > () . name ("is_contract") . type_name ("Weight") . docs (& ["Weight of calling `seal_is_contract`."])) . field (| f | f . ty :: < Weight > () . name ("code_hash") . type_name ("Weight") . docs (& ["Weight of calling `seal_code_hash`."])) . field (| f | f . ty :: < Weight > () . name ("own_code_hash") . type_name ("Weight") . docs (& ["Weight of calling `seal_own_code_hash`."])) . field (| f | f . ty :: < Weight > () . name ("caller_is_origin") . type_name ("Weight") . docs (& ["Weight of calling `seal_caller_is_origin`."])) . field (| f | f . ty :: < Weight > () . name ("caller_is_root") . type_name ("Weight") . docs (& ["Weight of calling `seal_caller_is_root`."])) . field (| f | f . ty :: < Weight > () . name ("address") . type_name ("Weight") . docs (& ["Weight of calling `seal_address`."])) . field (| f | f . ty :: < Weight > () . name ("gas_left") . type_name ("Weight") . docs (& ["Weight of calling `seal_gas_left`."])) . field (| f | f . ty :: < Weight > () . name ("balance") . type_name ("Weight") . docs (& ["Weight of calling `seal_balance`."])) . field (| f | f . ty :: < Weight > () . name ("value_transferred") . type_name ("Weight") . docs (& ["Weight of calling `seal_value_transferred`."])) . field (| f | f . ty :: < Weight > () . name ("minimum_balance") . type_name ("Weight") . docs (& ["Weight of calling `seal_minimum_balance`."])) . field (| f | f . ty :: < Weight > () . name ("block_number") . type_name ("Weight") . docs (& ["Weight of calling `seal_block_number`."])) . field (| f | f . ty :: < Weight > () . name ("now") . type_name ("Weight") . docs (& ["Weight of calling `seal_now`."])) . field (| f | f . ty :: < Weight > () . name ("weight_to_fee") . type_name ("Weight") . docs (& ["Weight of calling `seal_weight_to_fee`."])) . field (| f | f . ty :: < Weight > () . name ("gas") . type_name ("Weight") . docs (& ["Weight of calling `gas`."])) . field (| f | f . ty :: < Weight > () . name ("input") . type_name ("Weight") . docs (& ["Weight of calling `seal_input`."])) . field (| f | f . ty :: < Weight > () . name ("input_per_byte") . type_name ("Weight") . docs (& ["Weight per input byte copied to contract memory by `seal_input`."])) . field (| f | f . ty :: < Weight > () . name ("r#return") . type_name ("Weight") . docs (& ["Weight of calling `seal_return`."])) . field (| f | f . ty :: < Weight > () . name ("return_per_byte") . type_name ("Weight") . docs (& ["Weight per byte returned through `seal_return`."])) . field (| f | f . ty :: < Weight > () . name ("terminate") . type_name ("Weight") . docs (& ["Weight of calling `seal_terminate`."])) . field (| f | f . ty :: < Weight > () . name ("random") . type_name ("Weight") . docs (& ["Weight of calling `seal_random`."])) . field (| f | f . ty :: < Weight > () . name ("deposit_event") . type_name ("Weight") . docs (& ["Weight of calling `seal_reposit_event`."])) . field (| f | f . ty :: < Weight > () . name ("deposit_event_per_topic") . type_name ("Weight") . docs (& ["Weight per topic supplied to `seal_deposit_event`."])) . field (| f | f . ty :: < Weight > () . name ("deposit_event_per_byte") . type_name ("Weight") . docs (& ["Weight per byte of an event deposited through `seal_deposit_event`."])) . field (| f | f . ty :: < Weight > () . name ("debug_message") . type_name ("Weight") . docs (& ["Weight of calling `seal_debug_message`."])) . field (| f | f . ty :: < Weight > () . name ("debug_message_per_byte") . type_name ("Weight") . docs (& ["Weight of calling `seal_debug_message` per byte of the message."])) . field (| f | f . ty :: < Weight > () . name ("set_storage") . type_name ("Weight") . docs (& ["Weight of calling `seal_set_storage`."])) . field (| f | f . ty :: < Weight > () . name ("set_storage_per_new_byte") . type_name ("Weight") . docs (& ["Weight per written byten of an item stored with `seal_set_storage`."])) . field (| f | f . ty :: < Weight > () . name ("set_storage_per_old_byte") . type_name ("Weight") . docs (& ["Weight per overwritten byte of an item stored with `seal_set_storage`."])) . field (| f | f . ty :: < Weight > () . name ("set_code_hash") . type_name ("Weight") . docs (& ["Weight of calling `seal_set_code_hash`."])) . field (| f | f . ty :: < Weight > () . name ("clear_storage") . type_name ("Weight") . docs (& ["Weight of calling `seal_clear_storage`."])) . field (| f | f . ty :: < Weight > () . name ("clear_storage_per_byte") . type_name ("Weight") . docs (& ["Weight of calling `seal_clear_storage` per byte of the stored item."])) . field (| f | f . ty :: < Weight > () . name ("contains_storage") . type_name ("Weight") . docs (& ["Weight of calling `seal_contains_storage`."])) . field (| f | f . ty :: < Weight > () . name ("contains_storage_per_byte") . type_name ("Weight") . docs (& ["Weight of calling `seal_contains_storage` per byte of the stored item."])) . field (| f | f . ty :: < Weight > () . name ("get_storage") . type_name ("Weight") . docs (& ["Weight of calling `seal_get_storage`."])) . field (| f | f . ty :: < Weight > () . name ("get_storage_per_byte") . type_name ("Weight") . docs (& ["Weight per byte of an item received via `seal_get_storage`."])) . field (| f | f . ty :: < Weight > () . name ("take_storage") . type_name ("Weight") . docs (& ["Weight of calling `seal_take_storage`."])) . field (| f | f . ty :: < Weight > () . name ("take_storage_per_byte") . type_name ("Weight") . docs (& ["Weight per byte of an item received via `seal_take_storage`."])) . field (| f | f . ty :: < Weight > () . name ("transfer") . type_name ("Weight") . docs (& ["Weight of calling `seal_transfer`."])) . field (| f | f . ty :: < Weight > () . name ("call") . type_name ("Weight") . docs (& ["Weight of calling `seal_call`."])) . field (| f | f . ty :: < Weight > () . name ("delegate_call") . type_name ("Weight") . docs (& ["Weight of calling `seal_delegate_call`."])) . field (| f | f . ty :: < Weight > () . name ("call_transfer_surcharge") . type_name ("Weight") . docs (& ["Weight surcharge that is claimed if `seal_call` does a balance transfer."])) . field (| f | f . ty :: < Weight > () . name ("call_per_cloned_byte") . type_name ("Weight") . docs (& ["Weight per byte that is cloned by supplying the `CLONE_INPUT` flag."])) . field (| f | f . ty :: < Weight > () . name ("instantiate") . type_name ("Weight") . docs (& ["Weight of calling `seal_instantiate`."])) . field (| f | f . ty :: < Weight > () . name ("instantiate_transfer_surcharge") . type_name ("Weight") . docs (& ["Weight surcharge that is claimed if `seal_instantiate` does a balance transfer."])) . field (| f | f . ty :: < Weight > () . name ("instantiate_per_input_byte") . type_name ("Weight") . docs (& ["Weight per input byte supplied to `seal_instantiate`."])) . field (| f | f . ty :: < Weight > () . name ("instantiate_per_salt_byte") . type_name ("Weight") . docs (& ["Weight per salt byte supplied to `seal_instantiate`."])) . field (| f | f . ty :: < Weight > () . name ("hash_sha2_256") . type_name ("Weight") . docs (& ["Weight of calling `seal_hash_sha_256`."])) . field (| f | f . ty :: < Weight > () . name ("hash_sha2_256_per_byte") . type_name ("Weight") . docs (& ["Weight per byte hashed by `seal_hash_sha_256`."])) . field (| f | f . ty :: < Weight > () . name ("hash_keccak_256") . type_name ("Weight") . docs (& ["Weight of calling `seal_hash_keccak_256`."])) . field (| f | f . ty :: < Weight > () . name ("hash_keccak_256_per_byte") . type_name ("Weight") . docs (& ["Weight per byte hashed by `seal_hash_keccak_256`."])) . field (| f | f . ty :: < Weight > () . name ("hash_blake2_256") . type_name ("Weight") . docs (& ["Weight of calling `seal_hash_blake2_256`."])) . field (| f | f . ty :: < Weight > () . name ("hash_blake2_256_per_byte") . type_name ("Weight") . docs (& ["Weight per byte hashed by `seal_hash_blake2_256`."])) . field (| f | f . ty :: < Weight > () . name ("hash_blake2_128") . type_name ("Weight") . docs (& ["Weight of calling `seal_hash_blake2_128`."])) . field (| f | f . ty :: < Weight > () . name ("hash_blake2_128_per_byte") . type_name ("Weight") . docs (& ["Weight per byte hashed by `seal_hash_blake2_128`."])) . field (| f | f . ty :: < Weight > () . name ("ecdsa_recover") . type_name ("Weight") . docs (& ["Weight of calling `seal_ecdsa_recover`."])) . field (| f | f . ty :: < Weight > () . name ("ecdsa_to_eth_address") . type_name ("Weight") . docs (& ["Weight of calling `seal_ecdsa_to_eth_address`."])) . field (| f | f . ty :: < Weight > () . name ("sr25519_verify") . type_name ("Weight") . docs (& ["Weight of calling `sr25519_verify`."])) . field (| f | f . ty :: < Weight > () . name ("sr25519_verify_per_byte") . type_name ("Weight") . docs (& ["Weight per byte of calling `sr25519_verify`."])) . field (| f | f . ty :: < Weight > () . name ("reentrance_count") . type_name ("Weight") . docs (& ["Weight of calling `reentrance_count`."])) . field (| f | f . ty :: < Weight > () . name ("account_reentrance_count") . type_name ("Weight") . docs (& ["Weight of calling `account_reentrance_count`."])) . field (| f | f . ty :: < Weight > () . name ("instantiation_nonce") . type_name ("Weight") . docs (& ["Weight of calling `instantiation_nonce`."])))
            }
        };
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<T: Config> _serde::Serialize for HostFnWeights<T> {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = match _serde::Serializer::serialize_struct(
                    __serializer,
                    "HostFnWeights",
                    false as usize
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1
                        + 1,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "caller",
                    &self.caller,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "is_contract",
                    &self.is_contract,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "code_hash",
                    &self.code_hash,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "own_code_hash",
                    &self.own_code_hash,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "caller_is_origin",
                    &self.caller_is_origin,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "caller_is_root",
                    &self.caller_is_root,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "address",
                    &self.address,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "gas_left",
                    &self.gas_left,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "balance",
                    &self.balance,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "value_transferred",
                    &self.value_transferred,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "minimum_balance",
                    &self.minimum_balance,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "block_number",
                    &self.block_number,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "now",
                    &self.now,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "weight_to_fee",
                    &self.weight_to_fee,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "gas",
                    &self.gas,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "input",
                    &self.input,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "input_per_byte",
                    &self.input_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "return",
                    &self.r#return,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "return_per_byte",
                    &self.return_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "terminate",
                    &self.terminate,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "random",
                    &self.random,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "deposit_event",
                    &self.deposit_event,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "deposit_event_per_topic",
                    &self.deposit_event_per_topic,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "deposit_event_per_byte",
                    &self.deposit_event_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "debug_message",
                    &self.debug_message,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "debug_message_per_byte",
                    &self.debug_message_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "set_storage",
                    &self.set_storage,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "set_storage_per_new_byte",
                    &self.set_storage_per_new_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "set_storage_per_old_byte",
                    &self.set_storage_per_old_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "set_code_hash",
                    &self.set_code_hash,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "clear_storage",
                    &self.clear_storage,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "clear_storage_per_byte",
                    &self.clear_storage_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "contains_storage",
                    &self.contains_storage,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "contains_storage_per_byte",
                    &self.contains_storage_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "get_storage",
                    &self.get_storage,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "get_storage_per_byte",
                    &self.get_storage_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "take_storage",
                    &self.take_storage,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "take_storage_per_byte",
                    &self.take_storage_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "transfer",
                    &self.transfer,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "call",
                    &self.call,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "delegate_call",
                    &self.delegate_call,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "call_transfer_surcharge",
                    &self.call_transfer_surcharge,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "call_per_cloned_byte",
                    &self.call_per_cloned_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "instantiate",
                    &self.instantiate,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "instantiate_transfer_surcharge",
                    &self.instantiate_transfer_surcharge,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "instantiate_per_input_byte",
                    &self.instantiate_per_input_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "instantiate_per_salt_byte",
                    &self.instantiate_per_salt_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_sha2_256",
                    &self.hash_sha2_256,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_sha2_256_per_byte",
                    &self.hash_sha2_256_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_keccak_256",
                    &self.hash_keccak_256,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_keccak_256_per_byte",
                    &self.hash_keccak_256_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_blake2_256",
                    &self.hash_blake2_256,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_blake2_256_per_byte",
                    &self.hash_blake2_256_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_blake2_128",
                    &self.hash_blake2_128,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hash_blake2_128_per_byte",
                    &self.hash_blake2_128_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "ecdsa_recover",
                    &self.ecdsa_recover,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "ecdsa_to_eth_address",
                    &self.ecdsa_to_eth_address,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "sr25519_verify",
                    &self.sr25519_verify,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "sr25519_verify_per_byte",
                    &self.sr25519_verify_per_byte,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "reentrance_count",
                    &self.reentrance_count,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "account_reentrance_count",
                    &self.account_reentrance_count,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "instantiation_nonce",
                    &self.instantiation_nonce,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                match _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "_phantom",
                    &self._phantom,
                ) {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                };
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de, T: Config> _serde::Deserialize<'de> for HostFnWeights<T> {
            fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __field11,
                    __field12,
                    __field13,
                    __field14,
                    __field15,
                    __field16,
                    __field17,
                    __field18,
                    __field19,
                    __field20,
                    __field21,
                    __field22,
                    __field23,
                    __field24,
                    __field25,
                    __field26,
                    __field27,
                    __field28,
                    __field29,
                    __field30,
                    __field31,
                    __field32,
                    __field33,
                    __field34,
                    __field35,
                    __field36,
                    __field37,
                    __field38,
                    __field39,
                    __field40,
                    __field41,
                    __field42,
                    __field43,
                    __field44,
                    __field45,
                    __field46,
                    __field47,
                    __field48,
                    __field49,
                    __field50,
                    __field51,
                    __field52,
                    __field53,
                    __field54,
                    __field55,
                    __field56,
                    __field57,
                    __field58,
                    __field59,
                    __field60,
                    __field61,
                    __field62,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "field identifier")
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
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            11u64 => _serde::__private::Ok(__Field::__field11),
                            12u64 => _serde::__private::Ok(__Field::__field12),
                            13u64 => _serde::__private::Ok(__Field::__field13),
                            14u64 => _serde::__private::Ok(__Field::__field14),
                            15u64 => _serde::__private::Ok(__Field::__field15),
                            16u64 => _serde::__private::Ok(__Field::__field16),
                            17u64 => _serde::__private::Ok(__Field::__field17),
                            18u64 => _serde::__private::Ok(__Field::__field18),
                            19u64 => _serde::__private::Ok(__Field::__field19),
                            20u64 => _serde::__private::Ok(__Field::__field20),
                            21u64 => _serde::__private::Ok(__Field::__field21),
                            22u64 => _serde::__private::Ok(__Field::__field22),
                            23u64 => _serde::__private::Ok(__Field::__field23),
                            24u64 => _serde::__private::Ok(__Field::__field24),
                            25u64 => _serde::__private::Ok(__Field::__field25),
                            26u64 => _serde::__private::Ok(__Field::__field26),
                            27u64 => _serde::__private::Ok(__Field::__field27),
                            28u64 => _serde::__private::Ok(__Field::__field28),
                            29u64 => _serde::__private::Ok(__Field::__field29),
                            30u64 => _serde::__private::Ok(__Field::__field30),
                            31u64 => _serde::__private::Ok(__Field::__field31),
                            32u64 => _serde::__private::Ok(__Field::__field32),
                            33u64 => _serde::__private::Ok(__Field::__field33),
                            34u64 => _serde::__private::Ok(__Field::__field34),
                            35u64 => _serde::__private::Ok(__Field::__field35),
                            36u64 => _serde::__private::Ok(__Field::__field36),
                            37u64 => _serde::__private::Ok(__Field::__field37),
                            38u64 => _serde::__private::Ok(__Field::__field38),
                            39u64 => _serde::__private::Ok(__Field::__field39),
                            40u64 => _serde::__private::Ok(__Field::__field40),
                            41u64 => _serde::__private::Ok(__Field::__field41),
                            42u64 => _serde::__private::Ok(__Field::__field42),
                            43u64 => _serde::__private::Ok(__Field::__field43),
                            44u64 => _serde::__private::Ok(__Field::__field44),
                            45u64 => _serde::__private::Ok(__Field::__field45),
                            46u64 => _serde::__private::Ok(__Field::__field46),
                            47u64 => _serde::__private::Ok(__Field::__field47),
                            48u64 => _serde::__private::Ok(__Field::__field48),
                            49u64 => _serde::__private::Ok(__Field::__field49),
                            50u64 => _serde::__private::Ok(__Field::__field50),
                            51u64 => _serde::__private::Ok(__Field::__field51),
                            52u64 => _serde::__private::Ok(__Field::__field52),
                            53u64 => _serde::__private::Ok(__Field::__field53),
                            54u64 => _serde::__private::Ok(__Field::__field54),
                            55u64 => _serde::__private::Ok(__Field::__field55),
                            56u64 => _serde::__private::Ok(__Field::__field56),
                            57u64 => _serde::__private::Ok(__Field::__field57),
                            58u64 => _serde::__private::Ok(__Field::__field58),
                            59u64 => _serde::__private::Ok(__Field::__field59),
                            60u64 => _serde::__private::Ok(__Field::__field60),
                            61u64 => _serde::__private::Ok(__Field::__field61),
                            62u64 => _serde::__private::Ok(__Field::__field62),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            "caller" => _serde::__private::Ok(__Field::__field0),
                            "is_contract" => _serde::__private::Ok(__Field::__field1),
                            "code_hash" => _serde::__private::Ok(__Field::__field2),
                            "own_code_hash" => _serde::__private::Ok(__Field::__field3),
                            "caller_is_origin" => _serde::__private::Ok(__Field::__field4),
                            "caller_is_root" => _serde::__private::Ok(__Field::__field5),
                            "address" => _serde::__private::Ok(__Field::__field6),
                            "gas_left" => _serde::__private::Ok(__Field::__field7),
                            "balance" => _serde::__private::Ok(__Field::__field8),
                            "value_transferred" => _serde::__private::Ok(__Field::__field9),
                            "minimum_balance" => _serde::__private::Ok(__Field::__field10),
                            "block_number" => _serde::__private::Ok(__Field::__field11),
                            "now" => _serde::__private::Ok(__Field::__field12),
                            "weight_to_fee" => _serde::__private::Ok(__Field::__field13),
                            "gas" => _serde::__private::Ok(__Field::__field14),
                            "input" => _serde::__private::Ok(__Field::__field15),
                            "input_per_byte" => _serde::__private::Ok(__Field::__field16),
                            "return" => _serde::__private::Ok(__Field::__field17),
                            "return_per_byte" => _serde::__private::Ok(__Field::__field18),
                            "terminate" => _serde::__private::Ok(__Field::__field19),
                            "random" => _serde::__private::Ok(__Field::__field20),
                            "deposit_event" => _serde::__private::Ok(__Field::__field21),
                            "deposit_event_per_topic" => _serde::__private::Ok(__Field::__field22),
                            "deposit_event_per_byte" => _serde::__private::Ok(__Field::__field23),
                            "debug_message" => _serde::__private::Ok(__Field::__field24),
                            "debug_message_per_byte" => _serde::__private::Ok(__Field::__field25),
                            "set_storage" => _serde::__private::Ok(__Field::__field26),
                            "set_storage_per_new_byte" => _serde::__private::Ok(__Field::__field27),
                            "set_storage_per_old_byte" => _serde::__private::Ok(__Field::__field28),
                            "set_code_hash" => _serde::__private::Ok(__Field::__field29),
                            "clear_storage" => _serde::__private::Ok(__Field::__field30),
                            "clear_storage_per_byte" => _serde::__private::Ok(__Field::__field31),
                            "contains_storage" => _serde::__private::Ok(__Field::__field32),
                            "contains_storage_per_byte" => {
                                _serde::__private::Ok(__Field::__field33)
                            }
                            "get_storage" => _serde::__private::Ok(__Field::__field34),
                            "get_storage_per_byte" => _serde::__private::Ok(__Field::__field35),
                            "take_storage" => _serde::__private::Ok(__Field::__field36),
                            "take_storage_per_byte" => _serde::__private::Ok(__Field::__field37),
                            "transfer" => _serde::__private::Ok(__Field::__field38),
                            "call" => _serde::__private::Ok(__Field::__field39),
                            "delegate_call" => _serde::__private::Ok(__Field::__field40),
                            "call_transfer_surcharge" => _serde::__private::Ok(__Field::__field41),
                            "call_per_cloned_byte" => _serde::__private::Ok(__Field::__field42),
                            "instantiate" => _serde::__private::Ok(__Field::__field43),
                            "instantiate_transfer_surcharge" => {
                                _serde::__private::Ok(__Field::__field44)
                            }
                            "instantiate_per_input_byte" => {
                                _serde::__private::Ok(__Field::__field45)
                            }
                            "instantiate_per_salt_byte" => {
                                _serde::__private::Ok(__Field::__field46)
                            }
                            "hash_sha2_256" => _serde::__private::Ok(__Field::__field47),
                            "hash_sha2_256_per_byte" => _serde::__private::Ok(__Field::__field48),
                            "hash_keccak_256" => _serde::__private::Ok(__Field::__field49),
                            "hash_keccak_256_per_byte" => _serde::__private::Ok(__Field::__field50),
                            "hash_blake2_256" => _serde::__private::Ok(__Field::__field51),
                            "hash_blake2_256_per_byte" => _serde::__private::Ok(__Field::__field52),
                            "hash_blake2_128" => _serde::__private::Ok(__Field::__field53),
                            "hash_blake2_128_per_byte" => _serde::__private::Ok(__Field::__field54),
                            "ecdsa_recover" => _serde::__private::Ok(__Field::__field55),
                            "ecdsa_to_eth_address" => _serde::__private::Ok(__Field::__field56),
                            "sr25519_verify" => _serde::__private::Ok(__Field::__field57),
                            "sr25519_verify_per_byte" => _serde::__private::Ok(__Field::__field58),
                            "reentrance_count" => _serde::__private::Ok(__Field::__field59),
                            "account_reentrance_count" => _serde::__private::Ok(__Field::__field60),
                            "instantiation_nonce" => _serde::__private::Ok(__Field::__field61),
                            "_phantom" => _serde::__private::Ok(__Field::__field62),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                            b"caller" => _serde::__private::Ok(__Field::__field0),
                            b"is_contract" => _serde::__private::Ok(__Field::__field1),
                            b"code_hash" => _serde::__private::Ok(__Field::__field2),
                            b"own_code_hash" => _serde::__private::Ok(__Field::__field3),
                            b"caller_is_origin" => _serde::__private::Ok(__Field::__field4),
                            b"caller_is_root" => _serde::__private::Ok(__Field::__field5),
                            b"address" => _serde::__private::Ok(__Field::__field6),
                            b"gas_left" => _serde::__private::Ok(__Field::__field7),
                            b"balance" => _serde::__private::Ok(__Field::__field8),
                            b"value_transferred" => _serde::__private::Ok(__Field::__field9),
                            b"minimum_balance" => _serde::__private::Ok(__Field::__field10),
                            b"block_number" => _serde::__private::Ok(__Field::__field11),
                            b"now" => _serde::__private::Ok(__Field::__field12),
                            b"weight_to_fee" => _serde::__private::Ok(__Field::__field13),
                            b"gas" => _serde::__private::Ok(__Field::__field14),
                            b"input" => _serde::__private::Ok(__Field::__field15),
                            b"input_per_byte" => _serde::__private::Ok(__Field::__field16),
                            b"return" => _serde::__private::Ok(__Field::__field17),
                            b"return_per_byte" => _serde::__private::Ok(__Field::__field18),
                            b"terminate" => _serde::__private::Ok(__Field::__field19),
                            b"random" => _serde::__private::Ok(__Field::__field20),
                            b"deposit_event" => _serde::__private::Ok(__Field::__field21),
                            b"deposit_event_per_topic" => _serde::__private::Ok(__Field::__field22),
                            b"deposit_event_per_byte" => _serde::__private::Ok(__Field::__field23),
                            b"debug_message" => _serde::__private::Ok(__Field::__field24),
                            b"debug_message_per_byte" => _serde::__private::Ok(__Field::__field25),
                            b"set_storage" => _serde::__private::Ok(__Field::__field26),
                            b"set_storage_per_new_byte" => {
                                _serde::__private::Ok(__Field::__field27)
                            }
                            b"set_storage_per_old_byte" => {
                                _serde::__private::Ok(__Field::__field28)
                            }
                            b"set_code_hash" => _serde::__private::Ok(__Field::__field29),
                            b"clear_storage" => _serde::__private::Ok(__Field::__field30),
                            b"clear_storage_per_byte" => _serde::__private::Ok(__Field::__field31),
                            b"contains_storage" => _serde::__private::Ok(__Field::__field32),
                            b"contains_storage_per_byte" => {
                                _serde::__private::Ok(__Field::__field33)
                            }
                            b"get_storage" => _serde::__private::Ok(__Field::__field34),
                            b"get_storage_per_byte" => _serde::__private::Ok(__Field::__field35),
                            b"take_storage" => _serde::__private::Ok(__Field::__field36),
                            b"take_storage_per_byte" => _serde::__private::Ok(__Field::__field37),
                            b"transfer" => _serde::__private::Ok(__Field::__field38),
                            b"call" => _serde::__private::Ok(__Field::__field39),
                            b"delegate_call" => _serde::__private::Ok(__Field::__field40),
                            b"call_transfer_surcharge" => _serde::__private::Ok(__Field::__field41),
                            b"call_per_cloned_byte" => _serde::__private::Ok(__Field::__field42),
                            b"instantiate" => _serde::__private::Ok(__Field::__field43),
                            b"instantiate_transfer_surcharge" => {
                                _serde::__private::Ok(__Field::__field44)
                            }
                            b"instantiate_per_input_byte" => {
                                _serde::__private::Ok(__Field::__field45)
                            }
                            b"instantiate_per_salt_byte" => {
                                _serde::__private::Ok(__Field::__field46)
                            }
                            b"hash_sha2_256" => _serde::__private::Ok(__Field::__field47),
                            b"hash_sha2_256_per_byte" => _serde::__private::Ok(__Field::__field48),
                            b"hash_keccak_256" => _serde::__private::Ok(__Field::__field49),
                            b"hash_keccak_256_per_byte" => {
                                _serde::__private::Ok(__Field::__field50)
                            }
                            b"hash_blake2_256" => _serde::__private::Ok(__Field::__field51),
                            b"hash_blake2_256_per_byte" => {
                                _serde::__private::Ok(__Field::__field52)
                            }
                            b"hash_blake2_128" => _serde::__private::Ok(__Field::__field53),
                            b"hash_blake2_128_per_byte" => {
                                _serde::__private::Ok(__Field::__field54)
                            }
                            b"ecdsa_recover" => _serde::__private::Ok(__Field::__field55),
                            b"ecdsa_to_eth_address" => _serde::__private::Ok(__Field::__field56),
                            b"sr25519_verify" => _serde::__private::Ok(__Field::__field57),
                            b"sr25519_verify_per_byte" => _serde::__private::Ok(__Field::__field58),
                            b"reentrance_count" => _serde::__private::Ok(__Field::__field59),
                            b"account_reentrance_count" => {
                                _serde::__private::Ok(__Field::__field60)
                            }
                            b"instantiation_nonce" => _serde::__private::Ok(__Field::__field61),
                            b"_phantom" => _serde::__private::Ok(__Field::__field62),
                            _ => _serde::__private::Ok(__Field::__ignore),
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
                struct __Visitor<'de, T: Config> {
                    marker: _serde::__private::PhantomData<HostFnWeights<T>>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de, T: Config> _serde::de::Visitor<'de> for __Visitor<'de, T> {
                    type Value = HostFnWeights<T>;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "struct HostFnWeights")
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
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
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
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field1 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field2 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field3 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field4 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field5 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            5usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field6 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            6usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field7 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            7usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field8 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            8usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field9 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            9usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field10 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            10usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field11 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            11usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field12 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            12usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field13 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            13usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field14 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            14usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field15 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            15usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field16 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            16usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field17 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            17usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field18 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            18usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field19 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            19usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field20 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            20usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field21 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            21usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field22 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            22usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field23 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            23usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field24 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            24usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field25 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            25usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field26 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            26usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field27 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            27usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field28 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            28usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field29 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            29usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field30 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            30usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field31 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            31usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field32 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            32usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field33 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            33usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field34 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            34usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field35 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            35usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field36 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            36usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field37 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            37usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field38 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            38usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field39 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            39usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field40 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            40usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field41 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            41usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field42 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            42usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field43 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            43usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field44 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            44usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field45 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            45usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field46 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            46usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field47 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            47usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field48 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            48usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field49 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            49usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field50 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            50usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field51 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            51usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field52 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            52usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field53 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            53usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field54 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            54usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field55 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            55usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field56 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            56usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field57 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            57usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field58 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            58usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field59 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            59usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field60 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            60usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field61 =
                            match match _serde::de::SeqAccess::next_element::<Weight>(&mut __seq) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            61usize,
                                            &"struct HostFnWeights with 63 elements",
                                        ),
                                    );
                                }
                            };
                        let __field62 = match match _serde::de::SeqAccess::next_element::<
                            PhantomData<T>,
                        >(&mut __seq)
                        {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    62usize,
                                    &"struct HostFnWeights with 63 elements",
                                ));
                            }
                        };
                        _serde::__private::Ok(HostFnWeights {
                            caller: __field0,
                            is_contract: __field1,
                            code_hash: __field2,
                            own_code_hash: __field3,
                            caller_is_origin: __field4,
                            caller_is_root: __field5,
                            address: __field6,
                            gas_left: __field7,
                            balance: __field8,
                            value_transferred: __field9,
                            minimum_balance: __field10,
                            block_number: __field11,
                            now: __field12,
                            weight_to_fee: __field13,
                            gas: __field14,
                            input: __field15,
                            input_per_byte: __field16,
                            r#return: __field17,
                            return_per_byte: __field18,
                            terminate: __field19,
                            random: __field20,
                            deposit_event: __field21,
                            deposit_event_per_topic: __field22,
                            deposit_event_per_byte: __field23,
                            debug_message: __field24,
                            debug_message_per_byte: __field25,
                            set_storage: __field26,
                            set_storage_per_new_byte: __field27,
                            set_storage_per_old_byte: __field28,
                            set_code_hash: __field29,
                            clear_storage: __field30,
                            clear_storage_per_byte: __field31,
                            contains_storage: __field32,
                            contains_storage_per_byte: __field33,
                            get_storage: __field34,
                            get_storage_per_byte: __field35,
                            take_storage: __field36,
                            take_storage_per_byte: __field37,
                            transfer: __field38,
                            call: __field39,
                            delegate_call: __field40,
                            call_transfer_surcharge: __field41,
                            call_per_cloned_byte: __field42,
                            instantiate: __field43,
                            instantiate_transfer_surcharge: __field44,
                            instantiate_per_input_byte: __field45,
                            instantiate_per_salt_byte: __field46,
                            hash_sha2_256: __field47,
                            hash_sha2_256_per_byte: __field48,
                            hash_keccak_256: __field49,
                            hash_keccak_256_per_byte: __field50,
                            hash_blake2_256: __field51,
                            hash_blake2_256_per_byte: __field52,
                            hash_blake2_128: __field53,
                            hash_blake2_128_per_byte: __field54,
                            ecdsa_recover: __field55,
                            ecdsa_to_eth_address: __field56,
                            sr25519_verify: __field57,
                            sr25519_verify_per_byte: __field58,
                            reentrance_count: __field59,
                            account_reentrance_count: __field60,
                            instantiation_nonce: __field61,
                            _phantom: __field62,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field1: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field2: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field3: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field4: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field5: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field6: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field7: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field8: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field9: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field10: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field11: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field12: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field13: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field14: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field15: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field16: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field17: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field18: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field19: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field20: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field21: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field22: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field23: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field24: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field25: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field26: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field27: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field28: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field29: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field30: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field31: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field32: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field33: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field34: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field35: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field36: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field37: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field38: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field39: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field40: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field41: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field42: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field43: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field44: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field45: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field46: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field47: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field48: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field49: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field50: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field51: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field52: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field53: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field54: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field55: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field56: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field57: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field58: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field59: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field60: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field61: _serde::__private::Option<Weight> =
                            _serde::__private::None;
                        let mut __field62: _serde::__private::Option<PhantomData<T>> =
                            _serde::__private::None;
                        while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "caller",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "is_contract",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "code_hash",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "own_code_hash",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "caller_is_origin",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "caller_is_root",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "address",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "gas_left",
                                            ),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "balance",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "value_transferred",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "minimum_balance",
                                            ),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field11 => {
                                    if _serde::__private::Option::is_some(&__field11) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "block_number",
                                            ),
                                        );
                                    }
                                    __field11 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field12 => {
                                    if _serde::__private::Option::is_some(&__field12) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "now",
                                            ),
                                        );
                                    }
                                    __field12 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field13 => {
                                    if _serde::__private::Option::is_some(&__field13) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "weight_to_fee",
                                            ),
                                        );
                                    }
                                    __field13 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field14 => {
                                    if _serde::__private::Option::is_some(&__field14) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "gas",
                                            ),
                                        );
                                    }
                                    __field14 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field15 => {
                                    if _serde::__private::Option::is_some(&__field15) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "input",
                                            ),
                                        );
                                    }
                                    __field15 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field16 => {
                                    if _serde::__private::Option::is_some(&__field16) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "input_per_byte",
                                            ),
                                        );
                                    }
                                    __field16 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field17 => {
                                    if _serde::__private::Option::is_some(&__field17) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "return",
                                            ),
                                        );
                                    }
                                    __field17 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field18 => {
                                    if _serde::__private::Option::is_some(&__field18) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "return_per_byte",
                                            ),
                                        );
                                    }
                                    __field18 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field19 => {
                                    if _serde::__private::Option::is_some(&__field19) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "terminate",
                                            ),
                                        );
                                    }
                                    __field19 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field20 => {
                                    if _serde::__private::Option::is_some(&__field20) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "random",
                                            ),
                                        );
                                    }
                                    __field20 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field21 => {
                                    if _serde::__private::Option::is_some(&__field21) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "deposit_event",
                                            ),
                                        );
                                    }
                                    __field21 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field22 => {
                                    if _serde::__private::Option::is_some(&__field22) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "deposit_event_per_topic",
                                            ),
                                        );
                                    }
                                    __field22 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field23 => {
                                    if _serde::__private::Option::is_some(&__field23) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "deposit_event_per_byte",
                                            ),
                                        );
                                    }
                                    __field23 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field24 => {
                                    if _serde::__private::Option::is_some(&__field24) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "debug_message",
                                            ),
                                        );
                                    }
                                    __field24 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field25 => {
                                    if _serde::__private::Option::is_some(&__field25) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "debug_message_per_byte",
                                            ),
                                        );
                                    }
                                    __field25 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field26 => {
                                    if _serde::__private::Option::is_some(&__field26) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "set_storage",
                                            ),
                                        );
                                    }
                                    __field26 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field27 => {
                                    if _serde::__private::Option::is_some(&__field27) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "set_storage_per_new_byte",
                                            ),
                                        );
                                    }
                                    __field27 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field28 => {
                                    if _serde::__private::Option::is_some(&__field28) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "set_storage_per_old_byte",
                                            ),
                                        );
                                    }
                                    __field28 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field29 => {
                                    if _serde::__private::Option::is_some(&__field29) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "set_code_hash",
                                            ),
                                        );
                                    }
                                    __field29 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field30 => {
                                    if _serde::__private::Option::is_some(&__field30) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "clear_storage",
                                            ),
                                        );
                                    }
                                    __field30 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field31 => {
                                    if _serde::__private::Option::is_some(&__field31) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "clear_storage_per_byte",
                                            ),
                                        );
                                    }
                                    __field31 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field32 => {
                                    if _serde::__private::Option::is_some(&__field32) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "contains_storage",
                                            ),
                                        );
                                    }
                                    __field32 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field33 => {
                                    if _serde::__private::Option::is_some(&__field33) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "contains_storage_per_byte",
                                            ),
                                        );
                                    }
                                    __field33 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field34 => {
                                    if _serde::__private::Option::is_some(&__field34) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "get_storage",
                                            ),
                                        );
                                    }
                                    __field34 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field35 => {
                                    if _serde::__private::Option::is_some(&__field35) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "get_storage_per_byte",
                                            ),
                                        );
                                    }
                                    __field35 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field36 => {
                                    if _serde::__private::Option::is_some(&__field36) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "take_storage",
                                            ),
                                        );
                                    }
                                    __field36 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field37 => {
                                    if _serde::__private::Option::is_some(&__field37) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "take_storage_per_byte",
                                            ),
                                        );
                                    }
                                    __field37 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field38 => {
                                    if _serde::__private::Option::is_some(&__field38) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "transfer",
                                            ),
                                        );
                                    }
                                    __field38 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field39 => {
                                    if _serde::__private::Option::is_some(&__field39) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "call",
                                            ),
                                        );
                                    }
                                    __field39 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field40 => {
                                    if _serde::__private::Option::is_some(&__field40) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "delegate_call",
                                            ),
                                        );
                                    }
                                    __field40 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field41 => {
                                    if _serde::__private::Option::is_some(&__field41) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "call_transfer_surcharge",
                                            ),
                                        );
                                    }
                                    __field41 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field42 => {
                                    if _serde::__private::Option::is_some(&__field42) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "call_per_cloned_byte",
                                            ),
                                        );
                                    }
                                    __field42 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field43 => {
                                    if _serde::__private::Option::is_some(&__field43) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "instantiate",
                                            ),
                                        );
                                    }
                                    __field43 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field44 => {
                                    if _serde::__private::Option::is_some(&__field44) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "instantiate_transfer_surcharge",
                                            ),
                                        );
                                    }
                                    __field44 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field45 => {
                                    if _serde::__private::Option::is_some(&__field45) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "instantiate_per_input_byte",
                                            ),
                                        );
                                    }
                                    __field45 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field46 => {
                                    if _serde::__private::Option::is_some(&__field46) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "instantiate_per_salt_byte",
                                            ),
                                        );
                                    }
                                    __field46 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field47 => {
                                    if _serde::__private::Option::is_some(&__field47) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_sha2_256",
                                            ),
                                        );
                                    }
                                    __field47 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field48 => {
                                    if _serde::__private::Option::is_some(&__field48) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_sha2_256_per_byte",
                                            ),
                                        );
                                    }
                                    __field48 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field49 => {
                                    if _serde::__private::Option::is_some(&__field49) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_keccak_256",
                                            ),
                                        );
                                    }
                                    __field49 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field50 => {
                                    if _serde::__private::Option::is_some(&__field50) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_keccak_256_per_byte",
                                            ),
                                        );
                                    }
                                    __field50 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field51 => {
                                    if _serde::__private::Option::is_some(&__field51) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_blake2_256",
                                            ),
                                        );
                                    }
                                    __field51 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field52 => {
                                    if _serde::__private::Option::is_some(&__field52) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_blake2_256_per_byte",
                                            ),
                                        );
                                    }
                                    __field52 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field53 => {
                                    if _serde::__private::Option::is_some(&__field53) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_blake2_128",
                                            ),
                                        );
                                    }
                                    __field53 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field54 => {
                                    if _serde::__private::Option::is_some(&__field54) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hash_blake2_128_per_byte",
                                            ),
                                        );
                                    }
                                    __field54 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field55 => {
                                    if _serde::__private::Option::is_some(&__field55) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "ecdsa_recover",
                                            ),
                                        );
                                    }
                                    __field55 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field56 => {
                                    if _serde::__private::Option::is_some(&__field56) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "ecdsa_to_eth_address",
                                            ),
                                        );
                                    }
                                    __field56 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field57 => {
                                    if _serde::__private::Option::is_some(&__field57) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "sr25519_verify",
                                            ),
                                        );
                                    }
                                    __field57 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field58 => {
                                    if _serde::__private::Option::is_some(&__field58) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "sr25519_verify_per_byte",
                                            ),
                                        );
                                    }
                                    __field58 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field59 => {
                                    if _serde::__private::Option::is_some(&__field59) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "reentrance_count",
                                            ),
                                        );
                                    }
                                    __field59 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field60 => {
                                    if _serde::__private::Option::is_some(&__field60) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "account_reentrance_count",
                                            ),
                                        );
                                    }
                                    __field60 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field61 => {
                                    if _serde::__private::Option::is_some(&__field61) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "instantiation_nonce",
                                            ),
                                        );
                                    }
                                    __field61 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<Weight>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                __Field::__field62 => {
                                    if _serde::__private::Option::is_some(&__field62) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "_phantom",
                                            ),
                                        );
                                    }
                                    __field62 = _serde::__private::Some(
                                        match _serde::de::MapAccess::next_value::<PhantomData<T>>(
                                            &mut __map,
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        },
                                    );
                                }
                                _ => {
                                    let _ = match _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)
                                    {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("caller") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("is_contract") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("code_hash") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("own_code_hash") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("caller_is_origin") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("caller_is_root") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("address") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("gas_left") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("balance") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("value_transferred") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("minimum_balance") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field11 = match __field11 {
                            _serde::__private::Some(__field11) => __field11,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("block_number") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field12 = match __field12 {
                            _serde::__private::Some(__field12) => __field12,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("now") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field13 = match __field13 {
                            _serde::__private::Some(__field13) => __field13,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("weight_to_fee") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field14 = match __field14 {
                            _serde::__private::Some(__field14) => __field14,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("gas") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field15 = match __field15 {
                            _serde::__private::Some(__field15) => __field15,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("input") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field16 = match __field16 {
                            _serde::__private::Some(__field16) => __field16,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("input_per_byte") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field17 = match __field17 {
                            _serde::__private::Some(__field17) => __field17,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("return") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field18 = match __field18 {
                            _serde::__private::Some(__field18) => __field18,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("return_per_byte") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field19 = match __field19 {
                            _serde::__private::Some(__field19) => __field19,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("terminate") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field20 = match __field20 {
                            _serde::__private::Some(__field20) => __field20,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("random") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field21 = match __field21 {
                            _serde::__private::Some(__field21) => __field21,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("deposit_event") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field22 = match __field22 {
                            _serde::__private::Some(__field22) => __field22,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "deposit_event_per_topic",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field23 = match __field23 {
                            _serde::__private::Some(__field23) => __field23,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("deposit_event_per_byte")
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field24 = match __field24 {
                            _serde::__private::Some(__field24) => __field24,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("debug_message") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field25 = match __field25 {
                            _serde::__private::Some(__field25) => __field25,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("debug_message_per_byte")
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field26 = match __field26 {
                            _serde::__private::Some(__field26) => __field26,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("set_storage") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field27 = match __field27 {
                            _serde::__private::Some(__field27) => __field27,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "set_storage_per_new_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field28 = match __field28 {
                            _serde::__private::Some(__field28) => __field28,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "set_storage_per_old_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field29 = match __field29 {
                            _serde::__private::Some(__field29) => __field29,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("set_code_hash") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field30 = match __field30 {
                            _serde::__private::Some(__field30) => __field30,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("clear_storage") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field31 = match __field31 {
                            _serde::__private::Some(__field31) => __field31,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("clear_storage_per_byte")
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field32 = match __field32 {
                            _serde::__private::Some(__field32) => __field32,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("contains_storage") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field33 = match __field33 {
                            _serde::__private::Some(__field33) => __field33,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "contains_storage_per_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field34 = match __field34 {
                            _serde::__private::Some(__field34) => __field34,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("get_storage") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field35 = match __field35 {
                            _serde::__private::Some(__field35) => __field35,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("get_storage_per_byte") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field36 = match __field36 {
                            _serde::__private::Some(__field36) => __field36,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("take_storage") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field37 = match __field37 {
                            _serde::__private::Some(__field37) => __field37,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("take_storage_per_byte")
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field38 = match __field38 {
                            _serde::__private::Some(__field38) => __field38,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("transfer") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field39 = match __field39 {
                            _serde::__private::Some(__field39) => __field39,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("call") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field40 = match __field40 {
                            _serde::__private::Some(__field40) => __field40,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("delegate_call") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field41 = match __field41 {
                            _serde::__private::Some(__field41) => __field41,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "call_transfer_surcharge",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field42 = match __field42 {
                            _serde::__private::Some(__field42) => __field42,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("call_per_cloned_byte") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field43 = match __field43 {
                            _serde::__private::Some(__field43) => __field43,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("instantiate") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field44 = match __field44 {
                            _serde::__private::Some(__field44) => __field44,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "instantiate_transfer_surcharge",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field45 = match __field45 {
                            _serde::__private::Some(__field45) => __field45,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "instantiate_per_input_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field46 = match __field46 {
                            _serde::__private::Some(__field46) => __field46,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "instantiate_per_salt_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field47 = match __field47 {
                            _serde::__private::Some(__field47) => __field47,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("hash_sha2_256") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field48 = match __field48 {
                            _serde::__private::Some(__field48) => __field48,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("hash_sha2_256_per_byte")
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field49 = match __field49 {
                            _serde::__private::Some(__field49) => __field49,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("hash_keccak_256") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field50 = match __field50 {
                            _serde::__private::Some(__field50) => __field50,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "hash_keccak_256_per_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field51 = match __field51 {
                            _serde::__private::Some(__field51) => __field51,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("hash_blake2_256") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field52 = match __field52 {
                            _serde::__private::Some(__field52) => __field52,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "hash_blake2_256_per_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field53 = match __field53 {
                            _serde::__private::Some(__field53) => __field53,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("hash_blake2_128") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field54 = match __field54 {
                            _serde::__private::Some(__field54) => __field54,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "hash_blake2_128_per_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field55 = match __field55 {
                            _serde::__private::Some(__field55) => __field55,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("ecdsa_recover") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field56 = match __field56 {
                            _serde::__private::Some(__field56) => __field56,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("ecdsa_to_eth_address") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field57 = match __field57 {
                            _serde::__private::Some(__field57) => __field57,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("sr25519_verify") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field58 = match __field58 {
                            _serde::__private::Some(__field58) => __field58,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "sr25519_verify_per_byte",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field59 = match __field59 {
                            _serde::__private::Some(__field59) => __field59,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("reentrance_count") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field60 = match __field60 {
                            _serde::__private::Some(__field60) => __field60,
                            _serde::__private::None => match _serde::__private::de::missing_field(
                                "account_reentrance_count",
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                        };
                        let __field61 = match __field61 {
                            _serde::__private::Some(__field61) => __field61,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("instantiation_nonce") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        let __field62 = match __field62 {
                            _serde::__private::Some(__field62) => __field62,
                            _serde::__private::None => {
                                match _serde::__private::de::missing_field("_phantom") {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                            }
                        };
                        _serde::__private::Ok(HostFnWeights {
                            caller: __field0,
                            is_contract: __field1,
                            code_hash: __field2,
                            own_code_hash: __field3,
                            caller_is_origin: __field4,
                            caller_is_root: __field5,
                            address: __field6,
                            gas_left: __field7,
                            balance: __field8,
                            value_transferred: __field9,
                            minimum_balance: __field10,
                            block_number: __field11,
                            now: __field12,
                            weight_to_fee: __field13,
                            gas: __field14,
                            input: __field15,
                            input_per_byte: __field16,
                            r#return: __field17,
                            return_per_byte: __field18,
                            terminate: __field19,
                            random: __field20,
                            deposit_event: __field21,
                            deposit_event_per_topic: __field22,
                            deposit_event_per_byte: __field23,
                            debug_message: __field24,
                            debug_message_per_byte: __field25,
                            set_storage: __field26,
                            set_storage_per_new_byte: __field27,
                            set_storage_per_old_byte: __field28,
                            set_code_hash: __field29,
                            clear_storage: __field30,
                            clear_storage_per_byte: __field31,
                            contains_storage: __field32,
                            contains_storage_per_byte: __field33,
                            get_storage: __field34,
                            get_storage_per_byte: __field35,
                            take_storage: __field36,
                            take_storage_per_byte: __field37,
                            transfer: __field38,
                            call: __field39,
                            delegate_call: __field40,
                            call_transfer_surcharge: __field41,
                            call_per_cloned_byte: __field42,
                            instantiate: __field43,
                            instantiate_transfer_surcharge: __field44,
                            instantiate_per_input_byte: __field45,
                            instantiate_per_salt_byte: __field46,
                            hash_sha2_256: __field47,
                            hash_sha2_256_per_byte: __field48,
                            hash_keccak_256: __field49,
                            hash_keccak_256_per_byte: __field50,
                            hash_blake2_256: __field51,
                            hash_blake2_256_per_byte: __field52,
                            hash_blake2_128: __field53,
                            hash_blake2_128_per_byte: __field54,
                            ecdsa_recover: __field55,
                            ecdsa_to_eth_address: __field56,
                            sr25519_verify: __field57,
                            sr25519_verify_per_byte: __field58,
                            reentrance_count: __field59,
                            account_reentrance_count: __field60,
                            instantiation_nonce: __field61,
                            _phantom: __field62,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "caller",
                    "is_contract",
                    "code_hash",
                    "own_code_hash",
                    "caller_is_origin",
                    "caller_is_root",
                    "address",
                    "gas_left",
                    "balance",
                    "value_transferred",
                    "minimum_balance",
                    "block_number",
                    "now",
                    "weight_to_fee",
                    "gas",
                    "input",
                    "input_per_byte",
                    "return",
                    "return_per_byte",
                    "terminate",
                    "random",
                    "deposit_event",
                    "deposit_event_per_topic",
                    "deposit_event_per_byte",
                    "debug_message",
                    "debug_message_per_byte",
                    "set_storage",
                    "set_storage_per_new_byte",
                    "set_storage_per_old_byte",
                    "set_code_hash",
                    "clear_storage",
                    "clear_storage_per_byte",
                    "contains_storage",
                    "contains_storage_per_byte",
                    "get_storage",
                    "get_storage_per_byte",
                    "take_storage",
                    "take_storage_per_byte",
                    "transfer",
                    "call",
                    "delegate_call",
                    "call_transfer_surcharge",
                    "call_per_cloned_byte",
                    "instantiate",
                    "instantiate_transfer_surcharge",
                    "instantiate_per_input_byte",
                    "instantiate_per_salt_byte",
                    "hash_sha2_256",
                    "hash_sha2_256_per_byte",
                    "hash_keccak_256",
                    "hash_keccak_256_per_byte",
                    "hash_blake2_256",
                    "hash_blake2_256_per_byte",
                    "hash_blake2_128",
                    "hash_blake2_128_per_byte",
                    "ecdsa_recover",
                    "ecdsa_to_eth_address",
                    "sr25519_verify",
                    "sr25519_verify_per_byte",
                    "reentrance_count",
                    "account_reentrance_count",
                    "instantiation_nonce",
                    "_phantom",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "HostFnWeights",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<HostFnWeights<T>>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl Default for Limits {
        fn default() -> Self {
            Self {
                event_topics: 4,
                globals: 256,
                locals: 1024,
                parameters: 128,
                memory_pages: 16,
                table_size: 4096,
                br_table_size: 256,
                subject_len: 32,
                payload_len: 16 * 1024,
                runtime_memory: 1024 * 1024 * 128,
            }
        }
    }
    impl<T: Config> Default for InstructionWeights<T> {
        fn default() -> Self {
            Self {
                version: 4,
                fallback: 0,
                i64const: ((T::WeightInfo::instr_i64const(1)
                    .saturating_sub(T::WeightInfo::instr_i64const(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                i64load: ((T::WeightInfo::instr_i64load(1)
                    .saturating_sub(T::WeightInfo::instr_i64load(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64store: ((T::WeightInfo::instr_i64store(1)
                    .saturating_sub(T::WeightInfo::instr_i64store(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                select: ((T::WeightInfo::instr_select(1)
                    .saturating_sub(T::WeightInfo::instr_select(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(4),
                    ),
                r#if: ((T::WeightInfo::instr_if(1).saturating_sub(T::WeightInfo::instr_if(0)))
                    .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                br: ((T::WeightInfo::instr_br(1).saturating_sub(T::WeightInfo::instr_br(0)))
                    .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                br_if: ((T::WeightInfo::instr_br_if(1)
                    .saturating_sub(T::WeightInfo::instr_br_if(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                br_table: ((T::WeightInfo::instr_br_table(1)
                    .saturating_sub(T::WeightInfo::instr_br_table(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                br_table_per_entry: ((T::WeightInfo::instr_br_table_per_entry(1)
                    .saturating_sub(T::WeightInfo::instr_br_table_per_entry(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(0),
                    ),
                call: ((T::WeightInfo::instr_call(1).saturating_sub(T::WeightInfo::instr_call(0)))
                    .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                call_indirect: ((T::WeightInfo::instr_call_indirect(1)
                    .saturating_sub(T::WeightInfo::instr_call_indirect(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                call_per_local: ((T::WeightInfo::instr_call_per_local(1)
                    .saturating_sub(T::WeightInfo::instr_call_per_local(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(0),
                    ),
                local_get: ((T::WeightInfo::instr_local_get(1)
                    .saturating_sub(T::WeightInfo::instr_local_get(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                local_set: ((T::WeightInfo::instr_local_set(1)
                    .saturating_sub(T::WeightInfo::instr_local_set(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                local_tee: ((T::WeightInfo::instr_local_tee(1)
                    .saturating_sub(T::WeightInfo::instr_local_tee(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                global_get: ((T::WeightInfo::instr_global_get(1)
                    .saturating_sub(T::WeightInfo::instr_global_get(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                global_set: ((T::WeightInfo::instr_global_set(1)
                    .saturating_sub(T::WeightInfo::instr_global_set(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                memory_current: ((T::WeightInfo::instr_memory_current(1)
                    .saturating_sub(T::WeightInfo::instr_memory_current(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                memory_grow: ((T::WeightInfo::instr_memory_grow(1)
                    .saturating_sub(T::WeightInfo::instr_memory_grow(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(1),
                    ),
                i64clz: ((T::WeightInfo::instr_i64clz(1)
                    .saturating_sub(T::WeightInfo::instr_i64clz(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64ctz: ((T::WeightInfo::instr_i64ctz(1)
                    .saturating_sub(T::WeightInfo::instr_i64ctz(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64popcnt: ((T::WeightInfo::instr_i64popcnt(1)
                    .saturating_sub(T::WeightInfo::instr_i64popcnt(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64eqz: ((T::WeightInfo::instr_i64eqz(1)
                    .saturating_sub(T::WeightInfo::instr_i64eqz(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64extendsi32: ((T::WeightInfo::instr_i64extendsi32(1)
                    .saturating_sub(T::WeightInfo::instr_i64extendsi32(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64extendui32: ((T::WeightInfo::instr_i64extendui32(1)
                    .saturating_sub(T::WeightInfo::instr_i64extendui32(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i32wrapi64: ((T::WeightInfo::instr_i32wrapi64(1)
                    .saturating_sub(T::WeightInfo::instr_i32wrapi64(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(2),
                    ),
                i64eq: ((T::WeightInfo::instr_i64eq(1)
                    .saturating_sub(T::WeightInfo::instr_i64eq(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64ne: ((T::WeightInfo::instr_i64ne(1)
                    .saturating_sub(T::WeightInfo::instr_i64ne(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64lts: ((T::WeightInfo::instr_i64lts(1)
                    .saturating_sub(T::WeightInfo::instr_i64lts(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64ltu: ((T::WeightInfo::instr_i64ltu(1)
                    .saturating_sub(T::WeightInfo::instr_i64ltu(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64gts: ((T::WeightInfo::instr_i64gts(1)
                    .saturating_sub(T::WeightInfo::instr_i64gts(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64gtu: ((T::WeightInfo::instr_i64gtu(1)
                    .saturating_sub(T::WeightInfo::instr_i64gtu(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64les: ((T::WeightInfo::instr_i64les(1)
                    .saturating_sub(T::WeightInfo::instr_i64les(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64leu: ((T::WeightInfo::instr_i64leu(1)
                    .saturating_sub(T::WeightInfo::instr_i64leu(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64ges: ((T::WeightInfo::instr_i64ges(1)
                    .saturating_sub(T::WeightInfo::instr_i64ges(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64geu: ((T::WeightInfo::instr_i64geu(1)
                    .saturating_sub(T::WeightInfo::instr_i64geu(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64add: ((T::WeightInfo::instr_i64add(1)
                    .saturating_sub(T::WeightInfo::instr_i64add(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64sub: ((T::WeightInfo::instr_i64sub(1)
                    .saturating_sub(T::WeightInfo::instr_i64sub(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64mul: ((T::WeightInfo::instr_i64mul(1)
                    .saturating_sub(T::WeightInfo::instr_i64mul(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64divs: ((T::WeightInfo::instr_i64divs(1)
                    .saturating_sub(T::WeightInfo::instr_i64divs(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64divu: ((T::WeightInfo::instr_i64divu(1)
                    .saturating_sub(T::WeightInfo::instr_i64divu(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64rems: ((T::WeightInfo::instr_i64rems(1)
                    .saturating_sub(T::WeightInfo::instr_i64rems(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64remu: ((T::WeightInfo::instr_i64remu(1)
                    .saturating_sub(T::WeightInfo::instr_i64remu(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64and: ((T::WeightInfo::instr_i64and(1)
                    .saturating_sub(T::WeightInfo::instr_i64and(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64or: ((T::WeightInfo::instr_i64or(1)
                    .saturating_sub(T::WeightInfo::instr_i64or(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64xor: ((T::WeightInfo::instr_i64xor(1)
                    .saturating_sub(T::WeightInfo::instr_i64xor(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64shl: ((T::WeightInfo::instr_i64shl(1)
                    .saturating_sub(T::WeightInfo::instr_i64shl(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64shrs: ((T::WeightInfo::instr_i64shrs(1)
                    .saturating_sub(T::WeightInfo::instr_i64shrs(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64shru: ((T::WeightInfo::instr_i64shru(1)
                    .saturating_sub(T::WeightInfo::instr_i64shru(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64rotl: ((T::WeightInfo::instr_i64rotl(1)
                    .saturating_sub(T::WeightInfo::instr_i64rotl(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                i64rotr: ((T::WeightInfo::instr_i64rotr(1)
                    .saturating_sub(T::WeightInfo::instr_i64rotr(0)))
                .ref_time() as u32)
                    .saturating_sub(
                        ((T::WeightInfo::instr_i64const(1)
                            .saturating_sub(T::WeightInfo::instr_i64const(0)))
                        .ref_time() as u32
                            / 2)
                        .saturating_mul(3),
                    ),
                _phantom: PhantomData,
            }
        }
    }
    impl<T: Config> Default for HostFnWeights<T> {
        fn default() -> Self {
            Self {
                caller: (T::WeightInfo::seal_caller(1)
                    .saturating_sub(T::WeightInfo::seal_caller(0))),
                is_contract: (T::WeightInfo::seal_is_contract(1)
                    .saturating_sub(T::WeightInfo::seal_is_contract(0))),
                code_hash: (T::WeightInfo::seal_code_hash(1)
                    .saturating_sub(T::WeightInfo::seal_code_hash(0))),
                own_code_hash: (T::WeightInfo::seal_own_code_hash(1)
                    .saturating_sub(T::WeightInfo::seal_own_code_hash(0))),
                caller_is_origin: (T::WeightInfo::seal_caller_is_origin(1)
                    .saturating_sub(T::WeightInfo::seal_caller_is_origin(0))),
                caller_is_root: (T::WeightInfo::seal_caller_is_root(1)
                    .saturating_sub(T::WeightInfo::seal_caller_is_root(0))),
                address: (T::WeightInfo::seal_address(1)
                    .saturating_sub(T::WeightInfo::seal_address(0))),
                gas_left: (T::WeightInfo::seal_gas_left(1)
                    .saturating_sub(T::WeightInfo::seal_gas_left(0))),
                balance: (T::WeightInfo::seal_balance(1)
                    .saturating_sub(T::WeightInfo::seal_balance(0))),
                value_transferred: (T::WeightInfo::seal_value_transferred(1)
                    .saturating_sub(T::WeightInfo::seal_value_transferred(0))),
                minimum_balance: (T::WeightInfo::seal_minimum_balance(1)
                    .saturating_sub(T::WeightInfo::seal_minimum_balance(0))),
                block_number: (T::WeightInfo::seal_block_number(1)
                    .saturating_sub(T::WeightInfo::seal_block_number(0))),
                now: (T::WeightInfo::seal_now(1).saturating_sub(T::WeightInfo::seal_now(0))),
                weight_to_fee: (T::WeightInfo::seal_weight_to_fee(1)
                    .saturating_sub(T::WeightInfo::seal_weight_to_fee(0))),
                gas: (T::WeightInfo::seal_gas(1).saturating_sub(T::WeightInfo::seal_gas(0)))
                    .set_proof_size(0),
                input: (T::WeightInfo::seal_input(1).saturating_sub(T::WeightInfo::seal_input(0))),
                input_per_byte: (T::WeightInfo::seal_input_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_input_per_byte(0))),
                r#return: (T::WeightInfo::seal_return(1)
                    .saturating_sub(T::WeightInfo::seal_return(0))),
                return_per_byte: (T::WeightInfo::seal_return_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_return_per_byte(0))),
                terminate: (T::WeightInfo::seal_terminate(1)
                    .saturating_sub(T::WeightInfo::seal_terminate(0))),
                random: (T::WeightInfo::seal_random(1)
                    .saturating_sub(T::WeightInfo::seal_random(0))),
                deposit_event: (T::WeightInfo::seal_deposit_event(1)
                    .saturating_sub(T::WeightInfo::seal_deposit_event(0))),
                deposit_event_per_topic: (T::WeightInfo::seal_deposit_event_per_topic_and_byte(
                    1, 0,
                )
                .saturating_sub(T::WeightInfo::seal_deposit_event_per_topic_and_byte(0, 0))),
                deposit_event_per_byte: (T::WeightInfo::seal_deposit_event_per_topic_and_byte(
                    0, 1,
                )
                .saturating_sub(T::WeightInfo::seal_deposit_event_per_topic_and_byte(0, 0))),
                debug_message: (T::WeightInfo::seal_debug_message(1)
                    .saturating_sub(T::WeightInfo::seal_debug_message(0))),
                debug_message_per_byte: (T::WeightInfo::seal_debug_message_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_debug_message_per_byte(0))),
                set_storage: (T::WeightInfo::seal_set_storage(1)
                    .saturating_sub(T::WeightInfo::seal_set_storage(0))),
                set_code_hash: (T::WeightInfo::seal_set_code_hash(1)
                    .saturating_sub(T::WeightInfo::seal_set_code_hash(0))),
                set_storage_per_new_byte: (T::WeightInfo::seal_set_storage_per_new_byte(1)
                    .saturating_sub(T::WeightInfo::seal_set_storage_per_new_byte(0))),
                set_storage_per_old_byte: (T::WeightInfo::seal_set_storage_per_old_byte(1)
                    .saturating_sub(T::WeightInfo::seal_set_storage_per_old_byte(0))),
                clear_storage: (T::WeightInfo::seal_clear_storage(1)
                    .saturating_sub(T::WeightInfo::seal_clear_storage(0))),
                clear_storage_per_byte: (T::WeightInfo::seal_clear_storage_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_clear_storage_per_byte(0))),
                contains_storage: (T::WeightInfo::seal_contains_storage(1)
                    .saturating_sub(T::WeightInfo::seal_contains_storage(0))),
                contains_storage_per_byte: (T::WeightInfo::seal_contains_storage_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_contains_storage_per_byte(0))),
                get_storage: (T::WeightInfo::seal_get_storage(1)
                    .saturating_sub(T::WeightInfo::seal_get_storage(0))),
                get_storage_per_byte: (T::WeightInfo::seal_get_storage_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_get_storage_per_byte(0))),
                take_storage: (T::WeightInfo::seal_take_storage(1)
                    .saturating_sub(T::WeightInfo::seal_take_storage(0))),
                take_storage_per_byte: (T::WeightInfo::seal_take_storage_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_take_storage_per_byte(0))),
                transfer: (T::WeightInfo::seal_transfer(1)
                    .saturating_sub(T::WeightInfo::seal_transfer(0))),
                call: (T::WeightInfo::seal_call(1).saturating_sub(T::WeightInfo::seal_call(0))),
                delegate_call: (T::WeightInfo::seal_delegate_call(1)
                    .saturating_sub(T::WeightInfo::seal_delegate_call(0))),
                call_transfer_surcharge: (T::WeightInfo::seal_call_per_transfer_clone_byte(1, 0)
                    .saturating_sub(T::WeightInfo::seal_call_per_transfer_clone_byte(0, 0))),
                call_per_cloned_byte: (T::WeightInfo::seal_call_per_transfer_clone_byte(0, 1)
                    .saturating_sub(T::WeightInfo::seal_call_per_transfer_clone_byte(0, 0))),
                instantiate: (T::WeightInfo::seal_instantiate(1)
                    .saturating_sub(T::WeightInfo::seal_instantiate(0))),
                instantiate_transfer_surcharge:
                    (T::WeightInfo::seal_instantiate_per_transfer_input_salt_byte(1, 0, 0)
                        .saturating_sub(
                            T::WeightInfo::seal_instantiate_per_transfer_input_salt_byte(0, 0, 0),
                        )),
                instantiate_per_input_byte:
                    (T::WeightInfo::seal_instantiate_per_transfer_input_salt_byte(0, 1, 0)
                        .saturating_sub(
                            T::WeightInfo::seal_instantiate_per_transfer_input_salt_byte(0, 0, 0),
                        )),
                instantiate_per_salt_byte:
                    (T::WeightInfo::seal_instantiate_per_transfer_input_salt_byte(0, 0, 1)
                        .saturating_sub(
                            T::WeightInfo::seal_instantiate_per_transfer_input_salt_byte(0, 0, 0),
                        )),
                hash_sha2_256: (T::WeightInfo::seal_hash_sha2_256(1)
                    .saturating_sub(T::WeightInfo::seal_hash_sha2_256(0))),
                hash_sha2_256_per_byte: (T::WeightInfo::seal_hash_sha2_256_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_hash_sha2_256_per_byte(0))),
                hash_keccak_256: (T::WeightInfo::seal_hash_keccak_256(1)
                    .saturating_sub(T::WeightInfo::seal_hash_keccak_256(0))),
                hash_keccak_256_per_byte: (T::WeightInfo::seal_hash_keccak_256_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_hash_keccak_256_per_byte(0))),
                hash_blake2_256: (T::WeightInfo::seal_hash_blake2_256(1)
                    .saturating_sub(T::WeightInfo::seal_hash_blake2_256(0))),
                hash_blake2_256_per_byte: (T::WeightInfo::seal_hash_blake2_256_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_hash_blake2_256_per_byte(0))),
                hash_blake2_128: (T::WeightInfo::seal_hash_blake2_128(1)
                    .saturating_sub(T::WeightInfo::seal_hash_blake2_128(0))),
                hash_blake2_128_per_byte: (T::WeightInfo::seal_hash_blake2_128_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_hash_blake2_128_per_byte(0))),
                ecdsa_recover: (T::WeightInfo::seal_ecdsa_recover(1)
                    .saturating_sub(T::WeightInfo::seal_ecdsa_recover(0))),
                sr25519_verify: (T::WeightInfo::seal_sr25519_verify(1)
                    .saturating_sub(T::WeightInfo::seal_sr25519_verify(0))),
                sr25519_verify_per_byte: (T::WeightInfo::seal_sr25519_verify_per_byte(1)
                    .saturating_sub(T::WeightInfo::seal_sr25519_verify_per_byte(0))),
                ecdsa_to_eth_address: (T::WeightInfo::seal_ecdsa_to_eth_address(1)
                    .saturating_sub(T::WeightInfo::seal_ecdsa_to_eth_address(0))),
                reentrance_count: (T::WeightInfo::seal_reentrance_count(1)
                    .saturating_sub(T::WeightInfo::seal_reentrance_count(0))),
                account_reentrance_count: (T::WeightInfo::seal_account_reentrance_count(1)
                    .saturating_sub(T::WeightInfo::seal_account_reentrance_count(0))),
                instantiation_nonce: (T::WeightInfo::seal_instantiation_nonce(1)
                    .saturating_sub(T::WeightInfo::seal_instantiation_nonce(0))),
                _phantom: PhantomData,
            }
        }
    }
    struct ScheduleRules<'a, T: Config> {
        schedule: &'a Schedule<T>,
        determinism: Determinism,
    }
    impl<T: Config> Schedule<T> {
        pub(crate) fn rules(&self, determinism: Determinism) -> impl gas_metering::Rules + '_ {
            ScheduleRules {
                schedule: self,
                determinism,
            }
        }
    }
    impl<'a, T: Config> gas_metering::Rules for ScheduleRules<'a, T> {
        fn instruction_cost(&self, instruction: &elements::Instruction) -> Option<u32> {
            use self::elements::Instruction::*;
            let w = &self.schedule.instruction_weights;
            let weight = match *instruction {
                End | Unreachable | Return | Else => 0,
                I32Const(_) | I64Const(_) | Block(_) | Loop(_) | Nop | Drop => w.i64const,
                I32Load(_, _)
                | I32Load8S(_, _)
                | I32Load8U(_, _)
                | I32Load16S(_, _)
                | I32Load16U(_, _)
                | I64Load(_, _)
                | I64Load8S(_, _)
                | I64Load8U(_, _)
                | I64Load16S(_, _)
                | I64Load16U(_, _)
                | I64Load32S(_, _)
                | I64Load32U(_, _) => w.i64load,
                I32Store(_, _)
                | I32Store8(_, _)
                | I32Store16(_, _)
                | I64Store(_, _)
                | I64Store8(_, _)
                | I64Store16(_, _)
                | I64Store32(_, _) => w.i64store,
                Select => w.select,
                If(_) => w.r#if,
                Br(_) => w.br,
                BrIf(_) => w.br_if,
                Call(_) => w.call,
                GetLocal(_) => w.local_get,
                SetLocal(_) => w.local_set,
                TeeLocal(_) => w.local_tee,
                GetGlobal(_) => w.global_get,
                SetGlobal(_) => w.global_set,
                CurrentMemory(_) => w.memory_current,
                GrowMemory(_) => w.memory_grow,
                CallIndirect(_, _) => w.call_indirect,
                BrTable(ref data) => w
                    .br_table
                    .saturating_add(w.br_table_per_entry.saturating_mul(data.table.len() as u32)),
                I32Clz | I64Clz => w.i64clz,
                I32Ctz | I64Ctz => w.i64ctz,
                I32Popcnt | I64Popcnt => w.i64popcnt,
                I32Eqz | I64Eqz => w.i64eqz,
                I64ExtendSI32 => w.i64extendsi32,
                I64ExtendUI32 => w.i64extendui32,
                I32WrapI64 => w.i32wrapi64,
                I32Eq | I64Eq => w.i64eq,
                I32Ne | I64Ne => w.i64ne,
                I32LtS | I64LtS => w.i64lts,
                I32LtU | I64LtU => w.i64ltu,
                I32GtS | I64GtS => w.i64gts,
                I32GtU | I64GtU => w.i64gtu,
                I32LeS | I64LeS => w.i64les,
                I32LeU | I64LeU => w.i64leu,
                I32GeS | I64GeS => w.i64ges,
                I32GeU | I64GeU => w.i64geu,
                I32Add | I64Add => w.i64add,
                I32Sub | I64Sub => w.i64sub,
                I32Mul | I64Mul => w.i64mul,
                I32DivS | I64DivS => w.i64divs,
                I32DivU | I64DivU => w.i64divu,
                I32RemS | I64RemS => w.i64rems,
                I32RemU | I64RemU => w.i64remu,
                I32And | I64And => w.i64and,
                I32Or | I64Or => w.i64or,
                I32Xor | I64Xor => w.i64xor,
                I32Shl | I64Shl => w.i64shl,
                I32ShrS | I64ShrS => w.i64shrs,
                I32ShrU | I64ShrU => w.i64shru,
                I32Rotl | I64Rotl => w.i64rotl,
                I32Rotr | I64Rotr => w.i64rotr,
                _ if (match self.determinism {
                    Determinism::Relaxed => true,
                    _ => false,
                }) && w.fallback > 0 =>
                {
                    w.fallback
                }
                _ => return None,
            };
            Some(weight)
        }
        fn memory_grow_cost(&self) -> gas_metering::MemoryGrowCost {
            gas_metering::MemoryGrowCost::Free
        }
        fn call_per_local_cost(&self) -> u32 {
            self.schedule.instruction_weights.call_per_local
        }
    }
}
