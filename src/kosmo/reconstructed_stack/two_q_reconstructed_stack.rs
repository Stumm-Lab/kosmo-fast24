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
		local_object::{
			LocalObject,
			TwoQLocalObject,
			two_q_local_object::StackLocation,
		},
	},
};

pub struct TwoQReconstructedStack<'a> {
	max_size: u64,

	a1_used_size: u64,
	am_used_size: u64,

	kin: f64,
	kout: f64,

	a1: BinaryHeap<TwoQLocalObject<'a>>,
	am: BinaryHeap<TwoQLocalObject<'a>>,
}

impl<'a> ReconstructedStack<'a> for TwoQReconstructedStack<'a> {
	type LocalObject = TwoQLocalObject<'a>;

	fn insert(&mut self, local_object: TwoQLocalObject<'a>) {
		let object_size = local_object.size();

		if let Some(stack_location) = local_object.stack_location() {
			match stack_location {
				StackLocation::A1(_) => {
					self.a1.push(local_object);
					self.a1_used_size += object_size as u64;
				},

				StackLocation::Am => {
					self.am.push(local_object);
					self.am_used_size += object_size as u64;
				},
			}
		}
	}

	fn get_eviction(&mut self, exclude_key: Key) -> Option<Key> {
		let ain_size = (self.max_size as f64 * self.kin) as u64;
		let a1_size = (self.max_size as f64 * (self.kin + self.kout)) as u64;
		let used_size = self.a1_used_size + self.am_used_size;

		if self.a1_used_size > a1_size || self.a1_used_size > ain_size && used_size > self.max_size {
			return self.get_a1_eviction(exclude_key);
		}

		if used_size <= self.max_size {
			return None;
		}

		self.get_am_eviction(exclude_key)
	}
}

impl<'a> TwoQReconstructedStack<'a> {
	pub fn new(max_size: u64, kin: f64, kout: f64) -> Self {
		TwoQReconstructedStack {
			max_size,

			a1_used_size: 0,
			am_used_size: 0,

			kin,
			kout,

			a1: BinaryHeap::new(),
			am: BinaryHeap::new(),
		}
	}

	fn get_a1_eviction(&mut self, exclude_key: Key) -> Option<Key> {
		let evicted = self.a1.pop().map(|local_object| (
			local_object.key(),
			local_object.size()
		));

		if let Some((key, size)) = evicted {
			if key != exclude_key {
				self.a1_used_size -= size as u64;
			}
		}

		evicted.map(|(key, _)| key)
	}

	fn get_am_eviction(&mut self, exclude_key: Key) -> Option<Key> {
		let evicted = self.am.pop().map(|local_object| (
			local_object.key(),
			local_object.size()
		));

		if let Some((key, size)) = evicted {
			if key != exclude_key {
				self.am_used_size -= size as u64;
			}
		}

		evicted.map(|(key, _)| key)
	}
}
