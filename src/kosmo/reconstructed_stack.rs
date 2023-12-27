/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod lfu_reconstructed_stack;
mod fifo_reconstructed_stack;
mod two_q_reconstructed_stack;
mod lrfu_reconstructed_stack;
mod lru_reconstructed_stack;

use crate::{
	access::Key,
	kosmo::{
		KosmoPolicy,
		local_object::{LocalObject, LocalObjectPolicy},
	},
};

pub trait ReconstructedStack<'a> {
	type LocalObject: LocalObject<'a>;

	fn insert(&mut self, _: Self::LocalObject);
	fn get_eviction(&mut self, _: Key) -> Option<Key>;

	fn get_evictions(&mut self, exclude_key: Key) -> Vec<Key> {
		let mut keys = Vec::<Key>::new();

		while let Some(key) = self.get_eviction(exclude_key) {
			if key != exclude_key {
				keys.push(key);
			}
		}

		keys
	}
}


pub enum ReconstructedStackPolicy<'a> {
	Lfu(LfuReconstructedStack<'a>),
	Fifo(FifoReconstructedStack<'a>),
	TwoQ(TwoQReconstructedStack<'a>),
	Lrfu(LrfuReconstructedStack<'a>),
	Lru(LruReconstructedStack<'a>),
}

impl<'a> ReconstructedStackPolicy<'a> {
	pub fn new(policy: &KosmoPolicy, size: u64) -> Self {
		match policy {
			KosmoPolicy::Lfu => ReconstructedStackPolicy::Lfu(
				LfuReconstructedStack::new(size)
			),

			KosmoPolicy::Fifo => ReconstructedStackPolicy::Fifo(
				FifoReconstructedStack::new(size)
			),

			KosmoPolicy::TwoQ(kin, kout) => ReconstructedStackPolicy::TwoQ(
				TwoQReconstructedStack::new(size, *kin, *kout)
			),

			KosmoPolicy::Lrfu(_, _) => ReconstructedStackPolicy::Lrfu(
				LrfuReconstructedStack::new(size)
			),

			KosmoPolicy::Lru => ReconstructedStackPolicy::Lru(
				LruReconstructedStack::new(size)
			),
		}
	}

	pub fn insert(&mut self, local_object: LocalObjectPolicy<'a>) {
		match (self, local_object) {
			(ReconstructedStackPolicy::Lfu(stack), LocalObjectPolicy::Lfu(local_object))
				=> stack.insert(local_object),

			(ReconstructedStackPolicy::Fifo(stack), LocalObjectPolicy::Fifo(local_object))
				=> stack.insert(local_object),

			(ReconstructedStackPolicy::TwoQ(stack), LocalObjectPolicy::TwoQ(local_object))
				=> stack.insert(local_object),

			(ReconstructedStackPolicy::Lrfu(stack), LocalObjectPolicy::Lrfu(local_object))
				=> stack.insert(local_object),

			(ReconstructedStackPolicy::Lru(stack), LocalObjectPolicy::Lru(local_object))
				=> stack.insert(local_object),

			_ => panic!("Invalid local object type for reconstructed stack."),
		}
	}

	pub fn get_evictions(&mut self, exclude_key: Key) -> Vec<Key> {
		match self {
			ReconstructedStackPolicy::Lfu(stack) => stack.get_evictions(exclude_key),
			ReconstructedStackPolicy::Fifo(stack) => stack.get_evictions(exclude_key),
			ReconstructedStackPolicy::TwoQ(stack) => stack.get_evictions(exclude_key),
			ReconstructedStackPolicy::Lrfu(stack) => stack.get_evictions(exclude_key),
			ReconstructedStackPolicy::Lru(stack) => stack.get_evictions(exclude_key),
		}
	}
}

pub use crate::kosmo::reconstructed_stack::{
	lfu_reconstructed_stack::LfuReconstructedStack,
	fifo_reconstructed_stack::FifoReconstructedStack,
	two_q_reconstructed_stack::TwoQReconstructedStack,
	lrfu_reconstructed_stack::LrfuReconstructedStack,
	lru_reconstructed_stack::LruReconstructedStack,
};
