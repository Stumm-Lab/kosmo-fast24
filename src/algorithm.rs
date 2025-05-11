/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
	access::{Access, Timestamp, Key, Size},
	curve::Curve,
};

/// An MRC generation algorithm.
pub trait Algorithm {
	// Handles one cache access, checking SHARDS if configured, and
	// processing the access using the implementing algorithm.
	fn handle(&mut self, access: &Access) {
		if !self.verify_access(access) || !self.verify_shards(access) {
			return;
		}

		self.process(access);
	}

	/// Processes one cache access.
	fn process(&mut self, _: &Access);

	/// Removes the objects with the supplied key from all
	/// internal structures.
	fn remove(&mut self, _: Key);

	/// Clears the histogram.
	fn clean(&mut self);

	fn resize(&mut self, _: u64);

	/// Returns the MRC.
	fn curve(&mut self) -> Curve;

	/// Returns `true` if we should processes the supplied access.
	fn verify_access(&self, access: &Access) -> bool {
		access.is_valid_self_populating()
	}

	/// Returns `true` if the supplied access should be sampled.
	fn verify_shards(&mut self, _: &Access) -> bool;
}

pub struct Object {
	pub timestamp: Timestamp,
	pub key: Key,
	pub size: Size,
}

impl Object {
	pub fn new(access: &Access) -> Self {
		Object {
			timestamp: access.timestamp,
			key: access.key,
			size: access.size,
		}
	}

	pub fn update(&mut self, access: &Access) {
		self.timestamp = access.timestamp;
	}
}

impl PartialEq for Object {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}

impl Eq for Object {}
