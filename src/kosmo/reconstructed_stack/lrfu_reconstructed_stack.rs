/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BinaryHeap;

use crate::{
	access::Key,
	kosmo::{
		reconstructed_stack::ReconstructedStack,
		local_object::{LocalObject, LrfuLocalObject},
	},
};

pub struct LrfuReconstructedStack<'a> {
	max_size: u64,
	used_size: u64,

	stack: BinaryHeap<LrfuLocalObject<'a>>,
}

impl<'a> ReconstructedStack<'a> for LrfuReconstructedStack<'a> {
	type LocalObject = LrfuLocalObject<'a>;

	fn insert(&mut self, local_object: LrfuLocalObject<'a>) {
		let object_size = local_object.size();

		if local_object.exists() {
			self.stack.push(local_object);
			self.used_size += object_size as u64;
		}
	}

	fn get_eviction(&mut self, exclude_key: Key) -> Option<Key> {
		if self.used_size <= self.max_size {
			return None;
		}

		let evicted = self.stack.pop().map(|local_object| (
			local_object.key(),
			local_object.size()
		));

		if let Some((key, size)) = evicted {
			if key != exclude_key {
				self.used_size -= size as u64;
			}
		}

		evicted.map(|(key, _)| key)
	}
}

impl<'a> LrfuReconstructedStack<'a> {
	pub fn new(max_size: u64) -> Self {
		LrfuReconstructedStack {
			max_size,
			used_size: 0,

			stack: BinaryHeap::new(),
		}
	}
}
