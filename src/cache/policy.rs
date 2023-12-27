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

use crate::cache::{
	Cache,
	LfuCache,
	FifoCache,
	TwoQCache,
	LruCache,
	LrfuCache,
};

#[derive(Debug, Clone)]
pub enum CachePolicy {
	Lfu,
	Fifo,
	TwoQ(f64, f64),
	Lrfu(f64, f64),
	Lru,
}

impl CachePolicy {
	pub fn new_cache(&self, size: u64) -> Box<dyn Cache> {
		match self {
			CachePolicy::Lfu => Box::new(LfuCache::new(size)),
			CachePolicy::Fifo => Box::new(FifoCache::new(size)),
			CachePolicy::TwoQ(kin, kout) => Box::new(TwoQCache::new(size, *kin, *kout)),
			CachePolicy::Lrfu(p, lambda) => Box::new(LrfuCache::new(size, *p, *lambda)),
			CachePolicy::Lru => Box::new(LruCache::new(size)),
		}
	}
}

impl FromStr for CachePolicy {
	type Err = Error;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"lfu" => Ok(CachePolicy::Lfu),
			"fifo" => Ok(CachePolicy::Fifo),
			"lru" => Ok(CachePolicy::Lru),

			value if value.starts_with("2q") => parse_two_q_config(value),
			value if value.starts_with("lrfu") => parse_lrfu_config(value),

			_ => Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid cache policy.",
			)),
		}
	}
}

impl<'a> Deserialize<'a> for CachePolicy {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_str(CachePolicyVisitor)
	}
}

struct CachePolicyVisitor;

impl<'a> Visitor<'a> for CachePolicyVisitor {
	type Value = CachePolicy;

	fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
		formatter.write_str("a cache policy config")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		CachePolicy::from_str(value)
			.map_err(|err| E::custom(err.to_string()))
	}
}

fn parse_two_q_config(value: &str) -> Result<CachePolicy, Error> {
	let replaced = value.replace("2q-", "");

	let values = replaced
		.split('-')
		.collect::<Vec<&str>>();

	if values.len() != 2 {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Invalid 2Q policy config."
		));
	}

	let Ok(kin) = values[0].parse::<f64>() else {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Invalid 2Q policy config Kin value."
		));
	};

	let Ok(kout) = values[1].parse::<f64>() else {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Invalid 2Q policy config Kout value."
		));
	};

	Ok(CachePolicy::TwoQ(kin, kout))
}

fn parse_lrfu_config(value: &str) -> Result<CachePolicy, Error> {
	let replaced = value.replace("lrfu-", "");

	let values = replaced
		.split('-')
		.collect::<Vec<&str>>();

	if values.len() != 2 {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Invalid LRFU policy config."
		));
	}

	let Ok(p) = values[0].parse::<f64>() else {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Invalid LRFU policy config p value."
		));
	};

	let Ok(lambda) = values[1].parse::<f64>() else {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Invalid LRFU policy config lambda value."
		));
	};

	Ok(CachePolicy::Lrfu(p, lambda))
}
