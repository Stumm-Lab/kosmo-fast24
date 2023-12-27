/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	io::{Error, ErrorKind},
	fmt::{self, Formatter},
	str::FromStr,
};

use serde::{
	Deserialize,
	de::{self, Deserializer, Visitor},
};

#[derive(Debug, Clone, PartialEq)]
pub enum KosmoPolicy {
	Lfu,
	Fifo,
	TwoQ(f64, f64),
	Lrfu(f64, f64),
	Lru,
}

impl FromStr for KosmoPolicy {
	type Err = Error;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"lfu" => Ok(KosmoPolicy::Lfu),
			"fifo" => Ok(KosmoPolicy::Fifo),
			"lru" => Ok(KosmoPolicy::Lru),
			"2q" => Ok(KosmoPolicy::TwoQ(0.25, 0.5)),
			"lrfu" => Ok(KosmoPolicy::Lrfu(2.0, 0.5)),

			_ => Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid Kosmo policy.",
			)),
		}
	}
}

impl<'a> Deserialize<'a> for KosmoPolicy {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_str(KosmoPolicyVisitor)
	}
}

struct KosmoPolicyVisitor;

impl<'a> Visitor<'a> for KosmoPolicyVisitor {
	type Value = KosmoPolicy;

	fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
		formatter.write_str("a Kosmo policy config")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		KosmoPolicy::from_str(value)
			.map_err(|err| E::custom(err.to_string()))
	}
}
