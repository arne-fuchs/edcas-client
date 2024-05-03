pub use edcas::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod edcas {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![],
            }),
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CommodityListeningMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CommodityListeningMap",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("marketID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("commodity"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("buy_price"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sell_price"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("mean_price"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("demand"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("demand_bracket"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("stock"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("stock_bracket"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("cancel_carrier_jump"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "cancel_carrier_jump",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("carrierIDs"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("carrierIDs"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("carrierMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("carrierMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("registered"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("callsign"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("services"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("docking_access"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("allow_notorious"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_b"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("body_b"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_a"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("body_a"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("jump_timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commodities"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("commodities"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commodityMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("commodityMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("commodity"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("emit_carrier_jump"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("emit_carrier_jump"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("body"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("jump_timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("get_carrier_ids"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("get_carrier_ids"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("get_commodities"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("get_commodities"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("get_stations"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("get_stations"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct StationIdentity[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("get_systems"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("get_systems"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("planetMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("planetMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("discovered"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("mapped"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("planetProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct PlanetProperties"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("bodyProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct BodyProperties"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("registerCarrier"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("registerCarrier"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("callsign"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("services"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("docking_access"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("allow_notorious"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("register_commodity_listening"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "register_commodity_listening",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("marketID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("commodity"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("listening"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct CommodityListening",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("register_planet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("register_planet"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("discovered"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("mapped"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("planetProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct PlanetProperties"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("bodyProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct BodyProperties"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("register_star"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("register_star"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("discovered"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("mapped"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("starProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct StarProperties"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("bodyProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct BodyProperties"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("register_station"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("register_station"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("market_id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_type"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("faction"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Faction"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("government"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("economy"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("services"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("distance"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct floating"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("landingpads"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("register_system"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("register_system"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("government"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("allegiance"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("economy"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("second_economy"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("security"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("population"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("report_carrier_location"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "report_carrier_location",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("body"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("starMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("starMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("discovered"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("mapped"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("starProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct StarProperties"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("bodyProperties"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct BodyProperties"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("stationIds"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("stationIds"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("market_id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_type"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("stationMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("stationMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_marketID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("registered"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_address"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("system_name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("faction"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Faction"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("government"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("economy"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("services"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("distance"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Int(128usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct floating"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("landingpads"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("systemIdMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("systemIdMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("systemName"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("systemMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("systemMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("systemAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("name"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("allegiance"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("government"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("economy"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("second_economy"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("security"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("population"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("systems"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("systems"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("BodyRegistration"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("BodyRegistration"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("systemAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CarrierJump"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("CarrierJump"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CarrierJumpCancel"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("CarrierJumpCancel"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CarrierLocation"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("CarrierLocation"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CarrierRegistration"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CarrierRegistration",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CarrierUpdate"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("CarrierUpdate"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("carrierID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CommodityListeningUpdate"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CommodityListeningUpdate",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("marketID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("commodity"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("StarRegistration"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("StarRegistration"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("systemAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("StationRegistration"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "StationRegistration",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("marketID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("StationUpdate"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("StationUpdate"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("marketID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SystemRegistration"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("SystemRegistration"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("systemAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static EDCAS_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    pub struct EDCAS<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for EDCAS<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for EDCAS<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for EDCAS<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for EDCAS<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(EDCAS)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> EDCAS<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    EDCAS_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `CommodityListeningMap` (0x874893bc) function
        pub fn commodity_listening_map(
            &self,
            market_id: u64,
            commodity: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (u32, u32, u32, u32, u32, u32, u32),
        > {
            self.0
                .method_hash([135, 72, 147, 188], (market_id, commodity))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `cancel_carrier_jump` (0x0096afdc) function
        pub fn cancel_carrier_jump(
            &self,
            carrier_id: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([0, 150, 175, 220], carrier_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `carrierIDs` (0x4dc33122) function
        pub fn carrier_i_ds(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([77, 195, 49, 34], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `carrierMap` (0x2c594e5b) function
        pub fn carrier_map(
            &self,
            carrier_id: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                bool,
                ::ethers::core::types::U256,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                bool,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                ::ethers::core::types::U256,
            ),
        > {
            self.0
                .method_hash([44, 89, 78, 91], carrier_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commodities` (0xfcfd84e2) function
        pub fn commodities(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([252, 253, 132, 226], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commodityMap` (0x30280c85) function
        pub fn commodity_map(
            &self,
            commodity: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([48, 40, 12, 133], commodity)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `emit_carrier_jump` (0x06294ee9) function
        pub fn emit_carrier_jump(
            &self,
            carrier_id: u64,
            system: ::std::string::String,
            body: ::std::string::String,
            jump_timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [6, 41, 78, 233],
                    (carrier_id, system, body, jump_timestamp),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `get_carrier_ids` (0x87bcf475) function
        pub fn get_carrier_ids(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<u64>> {
            self.0
                .method_hash([135, 188, 244, 117], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `get_commodities` (0x9a26bb4b) function
        pub fn get_commodities(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::std::string::String>,
        > {
            self.0
                .method_hash([154, 38, 187, 75], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `get_stations` (0x6d08ca64) function
        pub fn get_stations(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<StationIdentity>,
        > {
            self.0
                .method_hash([109, 8, 202, 100], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `get_systems` (0xbfea3a1e) function
        pub fn get_systems(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::std::string::String>,
        > {
            self.0
                .method_hash([191, 234, 58, 30], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `planetMap` (0x63544387) function
        pub fn planet_map(
            &self,
            address: u64,
            p1: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::U256,
                u8,
                ::std::string::String,
                bool,
                bool,
                PlanetProperties,
                BodyProperties,
            ),
        > {
            self.0
                .method_hash([99, 84, 67, 135], (address, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `registerCarrier` (0x73047f73) function
        pub fn register_carrier(
            &self,
            carrier_id: u64,
            name: ::std::string::String,
            callsign: ::std::string::String,
            services: ::std::string::String,
            docking_access: ::std::string::String,
            allow_notorious: bool,
            timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [115, 4, 127, 115],
                    (
                        carrier_id,
                        name,
                        callsign,
                        services,
                        docking_access,
                        allow_notorious,
                        timestamp,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `register_commodity_listening` (0xe13d326f) function
        pub fn register_commodity_listening(
            &self,
            market_id: u64,
            commodity: ::std::string::String,
            listening: CommodityListening,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([225, 61, 50, 111], (market_id, commodity, listening))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `register_planet` (0x2c974f6e) function
        pub fn register_planet(
            &self,
            system_address: u64,
            id: u8,
            name: ::std::string::String,
            discovered: bool,
            mapped: bool,
            planet_properties: PlanetProperties,
            body_properties: BodyProperties,
            timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [44, 151, 79, 110],
                    (
                        system_address,
                        id,
                        name,
                        discovered,
                        mapped,
                        planet_properties,
                        body_properties,
                        timestamp,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `register_star` (0x44143959) function
        pub fn register_star(
            &self,
            system_address: u64,
            id: u8,
            name: ::std::string::String,
            discovered: bool,
            mapped: bool,
            star_properties: StarProperties,
            body_properties: BodyProperties,
            timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [68, 20, 57, 89],
                    (
                        system_address,
                        id,
                        name,
                        discovered,
                        mapped,
                        star_properties,
                        body_properties,
                        timestamp,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `register_station` (0x6ae7b282) function
        pub fn register_station(
            &self,
            market_id: u64,
            name: ::std::string::String,
            type_: ::std::string::String,
            system_address: u64,
            system_name: ::std::string::String,
            faction: Faction,
            government: ::std::string::String,
            economy: ::std::string::String,
            services: ::std::string::String,
            distance: Floating,
            landingpads: ::std::string::String,
            timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [106, 231, 178, 130],
                    (
                        market_id,
                        name,
                        type_,
                        system_address,
                        system_name,
                        faction,
                        government,
                        economy,
                        services,
                        distance,
                        landingpads,
                        timestamp,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `register_system` (0x0890b31a) function
        pub fn register_system(
            &self,
            system_address: u64,
            name: ::std::string::String,
            government: ::std::string::String,
            allegiance: ::std::string::String,
            economy: ::std::string::String,
            second_economy: ::std::string::String,
            security: ::std::string::String,
            population: u64,
            timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [8, 144, 179, 26],
                    (
                        system_address,
                        name,
                        government,
                        allegiance,
                        economy,
                        second_economy,
                        security,
                        population,
                        timestamp,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `report_carrier_location` (0x101c8d0d) function
        pub fn report_carrier_location(
            &self,
            carrier_id: u64,
            system: ::std::string::String,
            body: ::std::string::String,
            timestamp: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([16, 28, 141, 13], (carrier_id, system, body, timestamp))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `starMap` (0x98cb5700) function
        pub fn star_map(
            &self,
            address: u64,
            p1: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::U256,
                u8,
                ::std::string::String,
                bool,
                bool,
                StarProperties,
                BodyProperties,
            ),
        > {
            self.0
                .method_hash([152, 203, 87, 0], (address, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stationIds` (0xe391daf9) function
        pub fn station_ids(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::std::string::String, u64, ::std::string::String),
        > {
            self.0
                .method_hash([227, 145, 218, 249], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stationMap` (0xad62c24d) function
        pub fn station_map(
            &self,
            market_id: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                bool,
                ::ethers::core::types::U256,
                u64,
                ::std::string::String,
                Faction,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                Floating,
                ::std::string::String,
            ),
        > {
            self.0
                .method_hash([173, 98, 194, 77], market_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `systemIdMap` (0x01bbf489) function
        pub fn system_id_map(
            &self,
            system_name: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([1, 187, 244, 137], system_name)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `systemMap` (0x8c7da46d) function
        pub fn system_map(
            &self,
            system_address: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::U256,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                ::std::string::String,
                u64,
            ),
        > {
            self.0
                .method_hash([140, 125, 164, 109], system_address)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `systems` (0x0ffca604) function
        pub fn systems(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([15, 252, 166, 4], p0)
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `BodyRegistration` event
        pub fn body_registration_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            BodyRegistrationFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CarrierJump` event
        pub fn carrier_jump_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CarrierJumpFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CarrierJumpCancel` event
        pub fn carrier_jump_cancel_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CarrierJumpCancelFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CarrierLocation` event
        pub fn carrier_location_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CarrierLocationFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CarrierRegistration` event
        pub fn carrier_registration_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CarrierRegistrationFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CarrierUpdate` event
        pub fn carrier_update_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CarrierUpdateFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CommodityListeningUpdate` event
        pub fn commodity_listening_update_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CommodityListeningUpdateFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `StarRegistration` event
        pub fn star_registration_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            StarRegistrationFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `StationRegistration` event
        pub fn station_registration_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            StationRegistrationFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `StationUpdate` event
        pub fn station_update_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            StationUpdateFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SystemRegistration` event
        pub fn system_registration_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SystemRegistrationFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, EDCASEvents> {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for EDCAS<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "BodyRegistration", abi = "BodyRegistration(uint64)")]
    pub struct BodyRegistrationFilter {
        pub system_address: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "CarrierJump", abi = "CarrierJump(uint64)")]
    pub struct CarrierJumpFilter {
        pub carrier_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "CarrierJumpCancel", abi = "CarrierJumpCancel(uint64)")]
    pub struct CarrierJumpCancelFilter {
        pub carrier_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "CarrierLocation", abi = "CarrierLocation(uint64)")]
    pub struct CarrierLocationFilter {
        pub carrier_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "CarrierRegistration", abi = "CarrierRegistration(uint64)")]
    pub struct CarrierRegistrationFilter {
        pub carrier_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "CarrierUpdate", abi = "CarrierUpdate(uint64)")]
    pub struct CarrierUpdateFilter {
        pub carrier_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "CommodityListeningUpdate",
        abi = "CommodityListeningUpdate(uint64,string)"
    )]
    pub struct CommodityListeningUpdateFilter {
        pub market_id: u64,
        pub commodity: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "StarRegistration", abi = "StarRegistration(uint64)")]
    pub struct StarRegistrationFilter {
        pub system_address: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "StationRegistration", abi = "StationRegistration(uint64)")]
    pub struct StationRegistrationFilter {
        pub market_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "StationUpdate", abi = "StationUpdate(uint64)")]
    pub struct StationUpdateFilter {
        pub market_id: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "SystemRegistration", abi = "SystemRegistration(uint64)")]
    pub struct SystemRegistrationFilter {
        pub system_address: u64,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum EDCASEvents {
        BodyRegistrationFilter(BodyRegistrationFilter),
        CarrierJumpFilter(CarrierJumpFilter),
        CarrierJumpCancelFilter(CarrierJumpCancelFilter),
        CarrierLocationFilter(CarrierLocationFilter),
        CarrierRegistrationFilter(CarrierRegistrationFilter),
        CarrierUpdateFilter(CarrierUpdateFilter),
        CommodityListeningUpdateFilter(CommodityListeningUpdateFilter),
        StarRegistrationFilter(StarRegistrationFilter),
        StationRegistrationFilter(StationRegistrationFilter),
        StationUpdateFilter(StationUpdateFilter),
        SystemRegistrationFilter(SystemRegistrationFilter),
    }
    impl ::ethers::contract::EthLogDecode for EDCASEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = BodyRegistrationFilter::decode_log(log) {
                return Ok(EDCASEvents::BodyRegistrationFilter(decoded));
            }
            if let Ok(decoded) = CarrierJumpFilter::decode_log(log) {
                return Ok(EDCASEvents::CarrierJumpFilter(decoded));
            }
            if let Ok(decoded) = CarrierJumpCancelFilter::decode_log(log) {
                return Ok(EDCASEvents::CarrierJumpCancelFilter(decoded));
            }
            if let Ok(decoded) = CarrierLocationFilter::decode_log(log) {
                return Ok(EDCASEvents::CarrierLocationFilter(decoded));
            }
            if let Ok(decoded) = CarrierRegistrationFilter::decode_log(log) {
                return Ok(EDCASEvents::CarrierRegistrationFilter(decoded));
            }
            if let Ok(decoded) = CarrierUpdateFilter::decode_log(log) {
                return Ok(EDCASEvents::CarrierUpdateFilter(decoded));
            }
            if let Ok(decoded) = CommodityListeningUpdateFilter::decode_log(log) {
                return Ok(EDCASEvents::CommodityListeningUpdateFilter(decoded));
            }
            if let Ok(decoded) = StarRegistrationFilter::decode_log(log) {
                return Ok(EDCASEvents::StarRegistrationFilter(decoded));
            }
            if let Ok(decoded) = StationRegistrationFilter::decode_log(log) {
                return Ok(EDCASEvents::StationRegistrationFilter(decoded));
            }
            if let Ok(decoded) = StationUpdateFilter::decode_log(log) {
                return Ok(EDCASEvents::StationUpdateFilter(decoded));
            }
            if let Ok(decoded) = SystemRegistrationFilter::decode_log(log) {
                return Ok(EDCASEvents::SystemRegistrationFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for EDCASEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::BodyRegistrationFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CarrierJumpFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::CarrierJumpCancelFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CarrierLocationFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CarrierRegistrationFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CarrierUpdateFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CommodityListeningUpdateFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StarRegistrationFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StationRegistrationFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StationUpdateFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SystemRegistrationFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<BodyRegistrationFilter> for EDCASEvents {
        fn from(value: BodyRegistrationFilter) -> Self {
            Self::BodyRegistrationFilter(value)
        }
    }
    impl ::core::convert::From<CarrierJumpFilter> for EDCASEvents {
        fn from(value: CarrierJumpFilter) -> Self {
            Self::CarrierJumpFilter(value)
        }
    }
    impl ::core::convert::From<CarrierJumpCancelFilter> for EDCASEvents {
        fn from(value: CarrierJumpCancelFilter) -> Self {
            Self::CarrierJumpCancelFilter(value)
        }
    }
    impl ::core::convert::From<CarrierLocationFilter> for EDCASEvents {
        fn from(value: CarrierLocationFilter) -> Self {
            Self::CarrierLocationFilter(value)
        }
    }
    impl ::core::convert::From<CarrierRegistrationFilter> for EDCASEvents {
        fn from(value: CarrierRegistrationFilter) -> Self {
            Self::CarrierRegistrationFilter(value)
        }
    }
    impl ::core::convert::From<CarrierUpdateFilter> for EDCASEvents {
        fn from(value: CarrierUpdateFilter) -> Self {
            Self::CarrierUpdateFilter(value)
        }
    }
    impl ::core::convert::From<CommodityListeningUpdateFilter> for EDCASEvents {
        fn from(value: CommodityListeningUpdateFilter) -> Self {
            Self::CommodityListeningUpdateFilter(value)
        }
    }
    impl ::core::convert::From<StarRegistrationFilter> for EDCASEvents {
        fn from(value: StarRegistrationFilter) -> Self {
            Self::StarRegistrationFilter(value)
        }
    }
    impl ::core::convert::From<StationRegistrationFilter> for EDCASEvents {
        fn from(value: StationRegistrationFilter) -> Self {
            Self::StationRegistrationFilter(value)
        }
    }
    impl ::core::convert::From<StationUpdateFilter> for EDCASEvents {
        fn from(value: StationUpdateFilter) -> Self {
            Self::StationUpdateFilter(value)
        }
    }
    impl ::core::convert::From<SystemRegistrationFilter> for EDCASEvents {
        fn from(value: SystemRegistrationFilter) -> Self {
            Self::SystemRegistrationFilter(value)
        }
    }
    ///Container type for all input parameters for the `CommodityListeningMap` function with signature `CommodityListeningMap(uint64,string)` and selector `0x874893bc`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "CommodityListeningMap",
        abi = "CommodityListeningMap(uint64,string)"
    )]
    pub struct CommodityListeningMapCall {
        pub market_id: u64,
        pub commodity: ::std::string::String,
    }
    ///Container type for all input parameters for the `cancel_carrier_jump` function with signature `cancel_carrier_jump(uint64)` and selector `0x0096afdc`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "cancel_carrier_jump", abi = "cancel_carrier_jump(uint64)")]
    pub struct CancelCarrierJumpCall {
        pub carrier_id: u64,
    }
    ///Container type for all input parameters for the `carrierIDs` function with signature `carrierIDs(uint256)` and selector `0x4dc33122`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "carrierIDs", abi = "carrierIDs(uint256)")]
    pub struct CarrierIDsCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `carrierMap` function with signature `carrierMap(uint64)` and selector `0x2c594e5b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "carrierMap", abi = "carrierMap(uint64)")]
    pub struct CarrierMapCall {
        pub carrier_id: u64,
    }
    ///Container type for all input parameters for the `commodities` function with signature `commodities(uint256)` and selector `0xfcfd84e2`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "commodities", abi = "commodities(uint256)")]
    pub struct CommoditiesCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `commodityMap` function with signature `commodityMap(string)` and selector `0x30280c85`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "commodityMap", abi = "commodityMap(string)")]
    pub struct CommodityMapCall {
        pub commodity: ::std::string::String,
    }
    ///Container type for all input parameters for the `emit_carrier_jump` function with signature `emit_carrier_jump(uint64,string,string,uint256)` and selector `0x06294ee9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "emit_carrier_jump",
        abi = "emit_carrier_jump(uint64,string,string,uint256)"
    )]
    pub struct EmitCarrierJumpCall {
        pub carrier_id: u64,
        pub system: ::std::string::String,
        pub body: ::std::string::String,
        pub jump_timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `get_carrier_ids` function with signature `get_carrier_ids()` and selector `0x87bcf475`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "get_carrier_ids", abi = "get_carrier_ids()")]
    pub struct GetCarrierIdsCall;
    ///Container type for all input parameters for the `get_commodities` function with signature `get_commodities()` and selector `0x9a26bb4b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "get_commodities", abi = "get_commodities()")]
    pub struct GetCommoditiesCall;
    ///Container type for all input parameters for the `get_stations` function with signature `get_stations()` and selector `0x6d08ca64`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "get_stations", abi = "get_stations()")]
    pub struct GetStationsCall;
    ///Container type for all input parameters for the `get_systems` function with signature `get_systems()` and selector `0xbfea3a1e`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "get_systems", abi = "get_systems()")]
    pub struct GetSystemsCall;
    ///Container type for all input parameters for the `planetMap` function with signature `planetMap(uint64,uint256)` and selector `0x63544387`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "planetMap", abi = "planetMap(uint64,uint256)")]
    pub struct PlanetMapCall {
        pub address: u64,
        pub p1: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `registerCarrier` function with signature `registerCarrier(uint64,string,string,string,string,bool,uint256)` and selector `0x73047f73`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "registerCarrier",
        abi = "registerCarrier(uint64,string,string,string,string,bool,uint256)"
    )]
    pub struct RegisterCarrierCall {
        pub carrier_id: u64,
        pub name: ::std::string::String,
        pub callsign: ::std::string::String,
        pub services: ::std::string::String,
        pub docking_access: ::std::string::String,
        pub allow_notorious: bool,
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `register_commodity_listening` function with signature `register_commodity_listening(uint64,string,(uint32,uint32,uint32,uint32,uint32,uint32,uint32))` and selector `0xe13d326f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "register_commodity_listening",
        abi = "register_commodity_listening(uint64,string,(uint32,uint32,uint32,uint32,uint32,uint32,uint32))"
    )]
    pub struct RegisterCommodityListeningCall {
        pub market_id: u64,
        pub commodity: ::std::string::String,
        pub listening: CommodityListening,
    }
    ///Container type for all input parameters for the `register_planet` function with signature `register_planet(uint64,uint8,string,bool,bool,(string,string,bool,string,string,bool,uint8,(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8)),((int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8)),uint256)` and selector `0x2c974f6e`
    #[derive(Clone, ::ethers::contract::EthCall, ::ethers::contract::EthDisplay)]
    #[ethcall(
        name = "register_planet",
        abi = "register_planet(uint64,uint8,string,bool,bool,(string,string,bool,string,string,bool,uint8,(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8)),((int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8)),uint256)"
    )]
    pub struct RegisterPlanetCall {
        pub system_address: u64,
        pub id: u8,
        pub name: ::std::string::String,
        pub discovered: bool,
        pub mapped: bool,
        pub planet_properties: PlanetProperties,
        pub body_properties: BodyProperties,
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `register_star` function with signature `register_star(uint64,uint8,string,bool,bool,(uint8,uint16,string,string,(int128,uint8),(int128,uint8)),((int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8)),uint256)` and selector `0x44143959`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "register_star",
        abi = "register_star(uint64,uint8,string,bool,bool,(uint8,uint16,string,string,(int128,uint8),(int128,uint8)),((int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8)),uint256)"
    )]
    pub struct RegisterStarCall {
        pub system_address: u64,
        pub id: u8,
        pub name: ::std::string::String,
        pub discovered: bool,
        pub mapped: bool,
        pub star_properties: StarProperties,
        pub body_properties: BodyProperties,
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `register_station` function with signature `register_station(uint64,string,string,uint64,string,(string,string),string,string,string,(int128,uint8),string,uint256)` and selector `0x6ae7b282`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "register_station",
        abi = "register_station(uint64,string,string,uint64,string,(string,string),string,string,string,(int128,uint8),string,uint256)"
    )]
    pub struct RegisterStationCall {
        pub market_id: u64,
        pub name: ::std::string::String,
        pub type_: ::std::string::String,
        pub system_address: u64,
        pub system_name: ::std::string::String,
        pub faction: Faction,
        pub government: ::std::string::String,
        pub economy: ::std::string::String,
        pub services: ::std::string::String,
        pub distance: Floating,
        pub landingpads: ::std::string::String,
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `register_system` function with signature `register_system(uint64,string,string,string,string,string,string,uint64,uint256)` and selector `0x0890b31a`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "register_system",
        abi = "register_system(uint64,string,string,string,string,string,string,uint64,uint256)"
    )]
    pub struct RegisterSystemCall {
        pub system_address: u64,
        pub name: ::std::string::String,
        pub government: ::std::string::String,
        pub allegiance: ::std::string::String,
        pub economy: ::std::string::String,
        pub second_economy: ::std::string::String,
        pub security: ::std::string::String,
        pub population: u64,
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `report_carrier_location` function with signature `report_carrier_location(uint64,string,string,uint256)` and selector `0x101c8d0d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "report_carrier_location",
        abi = "report_carrier_location(uint64,string,string,uint256)"
    )]
    pub struct ReportCarrierLocationCall {
        pub carrier_id: u64,
        pub system: ::std::string::String,
        pub body: ::std::string::String,
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `starMap` function with signature `starMap(uint64,uint256)` and selector `0x98cb5700`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "starMap", abi = "starMap(uint64,uint256)")]
    pub struct StarMapCall {
        pub address: u64,
        pub p1: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `stationIds` function with signature `stationIds(uint256)` and selector `0xe391daf9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "stationIds", abi = "stationIds(uint256)")]
    pub struct StationIdsCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `stationMap` function with signature `stationMap(uint64)` and selector `0xad62c24d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "stationMap", abi = "stationMap(uint64)")]
    pub struct StationMapCall {
        pub market_id: u64,
    }
    ///Container type for all input parameters for the `systemIdMap` function with signature `systemIdMap(string)` and selector `0x01bbf489`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "systemIdMap", abi = "systemIdMap(string)")]
    pub struct SystemIdMapCall {
        pub system_name: ::std::string::String,
    }
    ///Container type for all input parameters for the `systemMap` function with signature `systemMap(uint64)` and selector `0x8c7da46d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "systemMap", abi = "systemMap(uint64)")]
    pub struct SystemMapCall {
        pub system_address: u64,
    }
    ///Container type for all input parameters for the `systems` function with signature `systems(uint256)` and selector `0x0ffca604`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "systems", abi = "systems(uint256)")]
    pub struct SystemsCall(pub ::ethers::core::types::U256);
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType)]
    pub enum EDCASCalls {
        CommodityListeningMap(CommodityListeningMapCall),
        CancelCarrierJump(CancelCarrierJumpCall),
        CarrierIDs(CarrierIDsCall),
        CarrierMap(CarrierMapCall),
        Commodities(CommoditiesCall),
        CommodityMap(CommodityMapCall),
        EmitCarrierJump(EmitCarrierJumpCall),
        GetCarrierIds(GetCarrierIdsCall),
        GetCommodities(GetCommoditiesCall),
        GetStations(GetStationsCall),
        GetSystems(GetSystemsCall),
        PlanetMap(PlanetMapCall),
        RegisterCarrier(RegisterCarrierCall),
        RegisterCommodityListening(RegisterCommodityListeningCall),
        RegisterPlanet(RegisterPlanetCall),
        RegisterStar(RegisterStarCall),
        RegisterStation(RegisterStationCall),
        RegisterSystem(RegisterSystemCall),
        ReportCarrierLocation(ReportCarrierLocationCall),
        StarMap(StarMapCall),
        StationIds(StationIdsCall),
        StationMap(StationMapCall),
        SystemIdMap(SystemIdMapCall),
        SystemMap(SystemMapCall),
        Systems(SystemsCall),
    }
    impl ::ethers::core::abi::AbiDecode for EDCASCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <CommodityListeningMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CommodityListeningMap(decoded));
            }
            if let Ok(decoded) = <CancelCarrierJumpCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CancelCarrierJump(decoded));
            }
            if let Ok(decoded) = <CarrierIDsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CarrierIDs(decoded));
            }
            if let Ok(decoded) = <CarrierMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CarrierMap(decoded));
            }
            if let Ok(decoded) = <CommoditiesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Commodities(decoded));
            }
            if let Ok(decoded) = <CommodityMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CommodityMap(decoded));
            }
            if let Ok(decoded) = <EmitCarrierJumpCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::EmitCarrierJump(decoded));
            }
            if let Ok(decoded) = <GetCarrierIdsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetCarrierIds(decoded));
            }
            if let Ok(decoded) = <GetCommoditiesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetCommodities(decoded));
            }
            if let Ok(decoded) = <GetStationsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetStations(decoded));
            }
            if let Ok(decoded) = <GetSystemsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetSystems(decoded));
            }
            if let Ok(decoded) = <PlanetMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PlanetMap(decoded));
            }
            if let Ok(decoded) = <RegisterCarrierCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterCarrier(decoded));
            }
            if let Ok(decoded) = <RegisterCommodityListeningCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterCommodityListening(decoded));
            }
            if let Ok(decoded) = <RegisterPlanetCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterPlanet(decoded));
            }
            if let Ok(decoded) = <RegisterStarCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterStar(decoded));
            }
            if let Ok(decoded) = <RegisterStationCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterStation(decoded));
            }
            if let Ok(decoded) = <RegisterSystemCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterSystem(decoded));
            }
            if let Ok(decoded) = <ReportCarrierLocationCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ReportCarrierLocation(decoded));
            }
            if let Ok(decoded) = <StarMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StarMap(decoded));
            }
            if let Ok(decoded) = <StationIdsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StationIds(decoded));
            }
            if let Ok(decoded) = <StationMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StationMap(decoded));
            }
            if let Ok(decoded) = <SystemIdMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SystemIdMap(decoded));
            }
            if let Ok(decoded) = <SystemMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SystemMap(decoded));
            }
            if let Ok(decoded) = <SystemsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Systems(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for EDCASCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::CommodityListeningMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CancelCarrierJump(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CarrierIDs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CarrierMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Commodities(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CommodityMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EmitCarrierJump(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCarrierIds(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCommodities(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetStations(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSystems(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PlanetMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterCarrier(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterCommodityListening(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterPlanet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterStar(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterStation(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterSystem(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ReportCarrierLocation(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StarMap(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::StationIds(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StationMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SystemIdMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SystemMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Systems(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for EDCASCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CommodityListeningMap(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CancelCarrierJump(element) => ::core::fmt::Display::fmt(element, f),
                Self::CarrierIDs(element) => ::core::fmt::Display::fmt(element, f),
                Self::CarrierMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::Commodities(element) => ::core::fmt::Display::fmt(element, f),
                Self::CommodityMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::EmitCarrierJump(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCarrierIds(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCommodities(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetStations(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSystems(element) => ::core::fmt::Display::fmt(element, f),
                Self::PlanetMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterCarrier(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterCommodityListening(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RegisterPlanet(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterStar(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterStation(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterSystem(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReportCarrierLocation(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StarMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::StationIds(element) => ::core::fmt::Display::fmt(element, f),
                Self::StationMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::SystemIdMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::SystemMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::Systems(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CommodityListeningMapCall> for EDCASCalls {
        fn from(value: CommodityListeningMapCall) -> Self {
            Self::CommodityListeningMap(value)
        }
    }
    impl ::core::convert::From<CancelCarrierJumpCall> for EDCASCalls {
        fn from(value: CancelCarrierJumpCall) -> Self {
            Self::CancelCarrierJump(value)
        }
    }
    impl ::core::convert::From<CarrierIDsCall> for EDCASCalls {
        fn from(value: CarrierIDsCall) -> Self {
            Self::CarrierIDs(value)
        }
    }
    impl ::core::convert::From<CarrierMapCall> for EDCASCalls {
        fn from(value: CarrierMapCall) -> Self {
            Self::CarrierMap(value)
        }
    }
    impl ::core::convert::From<CommoditiesCall> for EDCASCalls {
        fn from(value: CommoditiesCall) -> Self {
            Self::Commodities(value)
        }
    }
    impl ::core::convert::From<CommodityMapCall> for EDCASCalls {
        fn from(value: CommodityMapCall) -> Self {
            Self::CommodityMap(value)
        }
    }
    impl ::core::convert::From<EmitCarrierJumpCall> for EDCASCalls {
        fn from(value: EmitCarrierJumpCall) -> Self {
            Self::EmitCarrierJump(value)
        }
    }
    impl ::core::convert::From<GetCarrierIdsCall> for EDCASCalls {
        fn from(value: GetCarrierIdsCall) -> Self {
            Self::GetCarrierIds(value)
        }
    }
    impl ::core::convert::From<GetCommoditiesCall> for EDCASCalls {
        fn from(value: GetCommoditiesCall) -> Self {
            Self::GetCommodities(value)
        }
    }
    impl ::core::convert::From<GetStationsCall> for EDCASCalls {
        fn from(value: GetStationsCall) -> Self {
            Self::GetStations(value)
        }
    }
    impl ::core::convert::From<GetSystemsCall> for EDCASCalls {
        fn from(value: GetSystemsCall) -> Self {
            Self::GetSystems(value)
        }
    }
    impl ::core::convert::From<PlanetMapCall> for EDCASCalls {
        fn from(value: PlanetMapCall) -> Self {
            Self::PlanetMap(value)
        }
    }
    impl ::core::convert::From<RegisterCarrierCall> for EDCASCalls {
        fn from(value: RegisterCarrierCall) -> Self {
            Self::RegisterCarrier(value)
        }
    }
    impl ::core::convert::From<RegisterCommodityListeningCall> for EDCASCalls {
        fn from(value: RegisterCommodityListeningCall) -> Self {
            Self::RegisterCommodityListening(value)
        }
    }
    impl ::core::convert::From<RegisterPlanetCall> for EDCASCalls {
        fn from(value: RegisterPlanetCall) -> Self {
            Self::RegisterPlanet(value)
        }
    }
    impl ::core::convert::From<RegisterStarCall> for EDCASCalls {
        fn from(value: RegisterStarCall) -> Self {
            Self::RegisterStar(value)
        }
    }
    impl ::core::convert::From<RegisterStationCall> for EDCASCalls {
        fn from(value: RegisterStationCall) -> Self {
            Self::RegisterStation(value)
        }
    }
    impl ::core::convert::From<RegisterSystemCall> for EDCASCalls {
        fn from(value: RegisterSystemCall) -> Self {
            Self::RegisterSystem(value)
        }
    }
    impl ::core::convert::From<ReportCarrierLocationCall> for EDCASCalls {
        fn from(value: ReportCarrierLocationCall) -> Self {
            Self::ReportCarrierLocation(value)
        }
    }
    impl ::core::convert::From<StarMapCall> for EDCASCalls {
        fn from(value: StarMapCall) -> Self {
            Self::StarMap(value)
        }
    }
    impl ::core::convert::From<StationIdsCall> for EDCASCalls {
        fn from(value: StationIdsCall) -> Self {
            Self::StationIds(value)
        }
    }
    impl ::core::convert::From<StationMapCall> for EDCASCalls {
        fn from(value: StationMapCall) -> Self {
            Self::StationMap(value)
        }
    }
    impl ::core::convert::From<SystemIdMapCall> for EDCASCalls {
        fn from(value: SystemIdMapCall) -> Self {
            Self::SystemIdMap(value)
        }
    }
    impl ::core::convert::From<SystemMapCall> for EDCASCalls {
        fn from(value: SystemMapCall) -> Self {
            Self::SystemMap(value)
        }
    }
    impl ::core::convert::From<SystemsCall> for EDCASCalls {
        fn from(value: SystemsCall) -> Self {
            Self::Systems(value)
        }
    }
    ///Container type for all return fields from the `CommodityListeningMap` function with signature `CommodityListeningMap(uint64,string)` and selector `0x874893bc`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CommodityListeningMapReturn {
        pub buy_price: u32,
        pub sell_price: u32,
        pub mean_price: u32,
        pub demand: u32,
        pub demand_bracket: u32,
        pub stock: u32,
        pub stock_bracket: u32,
    }
    ///Container type for all return fields from the `carrierIDs` function with signature `carrierIDs(uint256)` and selector `0x4dc33122`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CarrierIDsReturn(pub u64);
    ///Container type for all return fields from the `carrierMap` function with signature `carrierMap(uint64)` and selector `0x2c594e5b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CarrierMapReturn {
        pub registered: bool,
        pub timestamp: ::ethers::core::types::U256,
        pub name: ::std::string::String,
        pub callsign: ::std::string::String,
        pub services: ::std::string::String,
        pub docking_access: ::std::string::String,
        pub allow_notorious: bool,
        pub system_b: ::std::string::String,
        pub body_b: ::std::string::String,
        pub system_a: ::std::string::String,
        pub body_a: ::std::string::String,
        pub jump_timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `commodities` function with signature `commodities(uint256)` and selector `0xfcfd84e2`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CommoditiesReturn(pub ::std::string::String);
    ///Container type for all return fields from the `commodityMap` function with signature `commodityMap(string)` and selector `0x30280c85`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CommodityMapReturn {
        pub id: u32,
    }
    ///Container type for all return fields from the `get_carrier_ids` function with signature `get_carrier_ids()` and selector `0x87bcf475`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetCarrierIdsReturn(pub ::std::vec::Vec<u64>);
    ///Container type for all return fields from the `get_commodities` function with signature `get_commodities()` and selector `0x9a26bb4b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetCommoditiesReturn(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all return fields from the `get_stations` function with signature `get_stations()` and selector `0x6d08ca64`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetStationsReturn(pub ::std::vec::Vec<StationIdentity>);
    ///Container type for all return fields from the `get_systems` function with signature `get_systems()` and selector `0xbfea3a1e`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetSystemsReturn(pub ::std::vec::Vec<::std::string::String>);
    ///Container type for all return fields from the `planetMap` function with signature `planetMap(uint64,uint256)` and selector `0x63544387`
    #[derive(Clone, ::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)]
    pub struct PlanetMapReturn {
        pub timestamp: ::ethers::core::types::U256,
        pub id: u8,
        pub name: ::std::string::String,
        pub discovered: bool,
        pub mapped: bool,
        pub planet_properties: PlanetProperties,
        pub body_properties: BodyProperties,
    }
    ///Container type for all return fields from the `starMap` function with signature `starMap(uint64,uint256)` and selector `0x98cb5700`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StarMapReturn {
        pub timestamp: ::ethers::core::types::U256,
        pub id: u8,
        pub name: ::std::string::String,
        pub discovered: bool,
        pub mapped: bool,
        pub star_properties: StarProperties,
        pub body_properties: BodyProperties,
    }
    ///Container type for all return fields from the `stationIds` function with signature `stationIds(uint256)` and selector `0xe391daf9`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StationIdsReturn {
        pub name: ::std::string::String,
        pub market_id: u64,
        pub type_: ::std::string::String,
    }
    ///Container type for all return fields from the `stationMap` function with signature `stationMap(uint64)` and selector `0xad62c24d`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StationMapReturn {
        pub registered: bool,
        pub timestamp: ::ethers::core::types::U256,
        pub system_address: u64,
        pub system_name: ::std::string::String,
        pub faction: Faction,
        pub government: ::std::string::String,
        pub economy: ::std::string::String,
        pub services: ::std::string::String,
        pub distance: Floating,
        pub landingpads: ::std::string::String,
    }
    ///Container type for all return fields from the `systemIdMap` function with signature `systemIdMap(string)` and selector `0x01bbf489`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SystemIdMapReturn(pub u64);
    ///Container type for all return fields from the `systemMap` function with signature `systemMap(uint64)` and selector `0x8c7da46d`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SystemMapReturn {
        pub timestamp: ::ethers::core::types::U256,
        pub name: ::std::string::String,
        pub allegiance: ::std::string::String,
        pub government: ::std::string::String,
        pub economy: ::std::string::String,
        pub second_economy: ::std::string::String,
        pub security: ::std::string::String,
        pub population: u64,
    }
    ///Container type for all return fields from the `systems` function with signature `systems(uint256)` and selector `0x0ffca604`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SystemsReturn(pub ::std::string::String);
    ///`BodyProperties((int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct BodyProperties {
        pub radius: Floating,
        pub distance_from_arrival_ls: Floating,
        pub axial_tilt: Floating,
        pub rotation_period: Floating,
        pub surface_temperature: Floating,
    }
    ///`CommodityListening(uint32,uint32,uint32,uint32,uint32,uint32,uint32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CommodityListening {
        pub buy_price: u32,
        pub sell_price: u32,
        pub mean_price: u32,
        pub demand: u32,
        pub demand_bracket: u32,
        pub stock: u32,
        pub stock_bracket: u32,
    }
    ///`Faction(string,string)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Faction {
        pub name: ::std::string::String,
        pub state: ::std::string::String,
    }
    ///`PlanetProperties(string,string,bool,string,string,bool,uint8,(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8),(int128,uint8))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct PlanetProperties {
        pub atmosphere: ::std::string::String,
        pub class: ::std::string::String,
        pub landable: bool,
        pub terraform_state: ::std::string::String,
        pub volcanism: ::std::string::String,
        pub tidal_lock: bool,
        pub parent_id: u8,
        pub mass_em: Floating,
        pub surface_gravity: Floating,
        pub surface_pressure: Floating,
        pub ascending_node: Floating,
        pub eccentricity: Floating,
        pub mean_anomaly: Floating,
        pub orbital_inclination: Floating,
        pub orbital_period: Floating,
        pub periapsis: Floating,
        pub semi_major_axis: Floating,
    }
    ///`StarProperties(uint8,uint16,string,string,(int128,uint8),(int128,uint8))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StarProperties {
        pub subclass: u8,
        pub age_my: u16,
        pub type_: ::std::string::String,
        pub luminosity: ::std::string::String,
        pub stellar_mass: Floating,
        pub absolute_magnitude: Floating,
    }
    ///`StationIdentity(string,uint64,string)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StationIdentity {
        pub name: ::std::string::String,
        pub market_id: u64,
        pub type_: ::std::string::String,
    }
    ///`Floating(int128,uint8)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Floating {
        pub decimal: i128,
        pub floating_point: u8,
    }
}
