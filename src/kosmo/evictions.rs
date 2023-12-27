/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use rustc_hash::FxHashSet;

use crate::{
	access::Key,
	kosmo::reconstructed_stack::ReconstructedStackPolicy,
};

pub struct Evictions {
	policy_evictions: Vec<Vec<Key>>,
	evicted_policy_keys: Vec<FxHashSet<Key>>,
}

impl Evictions {
	pub fn new(
		mut stacks: Vec<ReconstructedStackPolicy>,
		exclude_key: Key,
	) -> Self {
		let mut policy_evictions = Vec::<Vec<Key>>::with_capacity(stacks.len());
		let mut evicted_policy_keys = Vec::<FxHashSet<Key>>::with_capacity(stacks.len());

		for stack in stacks.iter_mut() {
			policy_evictions.push(stack.get_evictions(exclude_key));
			evicted_policy_keys.push(FxHashSet::default());
		}

		Evictions {
			policy_evictions,
			evicted_policy_keys,
		}
	}

	pub fn get_key(&mut self, policy_index: usize) -> Option<Key> {
		let key = self.policy_evictions[policy_index].pop()?;

		match self.save_key(policy_index, key) {
			true => Some(key),
			false => None,
		}
	}

	fn save_key(&mut self, policy_index: usize, key: Key) -> bool {
		self.evicted_policy_keys[policy_index].insert(key)
	}
}
