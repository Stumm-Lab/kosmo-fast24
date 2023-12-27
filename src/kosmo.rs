/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod policy;
mod global_object;
mod eviction_map;
mod local_object;
mod reconstructed_stack;
mod evictions;

use rustc_hash::FxHashMap;
use rayon::prelude::*;
use kwik::math;

use crate::{
	access::{Access, Key},
	algorithm::Algorithm,
	histogram::Histogram,
	shards::Shards,
	curve::Curve,
	kosmo::{
		global_object::GlobalObject,
		reconstructed_stack::ReconstructedStackPolicy,
		evictions::Evictions,
		eviction_map::EvictionMap,
	},
};

pub use crate::kosmo::policy::KosmoPolicy;

const GRANULARITY: u32 = 10;
const MIN_RECONSTRUCTED_STACK_SIZE: u64 = 1024;

pub struct Kosmo {
	global_table: FxHashMap<Key, GlobalObject>,
	total_size: u64,

	policies: Vec<KosmoPolicy>,
	granularity: u32,

	shards: Option<Box<dyn Shards>>,
	histograms: Vec<Histogram>,
}

impl Algorithm for Kosmo {
	fn process(&mut self, access: &Access) {
		let max_reuse_distance = self.update_histograms(access);

		if max_reuse_distance.is_none() {
			self.total_size += access.size as u64;

			self.global_table.insert(access.key, GlobalObject::new(
				access,
				&self.policies
			));
		}

		let simulate_size = max_reuse_distance.unwrap_or(self.total_size);

		self.perform_evictions(access, simulate_size);
	}

	fn remove(&mut self, key: Key) {
		self.global_table.remove(&key);
	}

	fn clean(&mut self) {
		self.histograms.iter_mut().for_each(|histogram| histogram.clear());
	}

	fn resize(&mut self, size: u64) {
		self.global_table.retain(|_, global_object| global_object.exists_at(size));
		self.histograms.iter_mut().for_each(|histogram| histogram.resize(size));
	}

	fn curve(&mut self) -> Curve {
		let policy = self.policies[0].clone();
		self.policy_curve(&policy).unwrap_or_default()
	}

	fn verify_shards(&mut self, access: &Access) -> bool {
		if let Some(ref mut shards) = self.shards {
			if !shards.sample(access) {
				return false;
			}

			if let Some(key) = shards.get_removal() {
				self.remove(key);
			}
		}

		true
	}
}

impl Kosmo {
	pub fn new(
		policies: &[KosmoPolicy],
		shards: Option<Box<dyn Shards>>,
	) -> Self {
		assert!(!policies.is_empty(), "Kosmo must be configured with at least one policy.");
		assert!(!has_duplicate_policies(policies), "Kosmo cannot have duplicate policies.");

		let histograms = policies
			.iter()
			.map(|_| Histogram::new(shards.as_deref()))
			.collect::<Vec<Histogram>>();

		Kosmo {
			global_table: FxHashMap::default(),
			total_size: 0,

			policies: policies.to_vec(),
			granularity: GRANULARITY,

			shards,
			histograms,
		}
	}

	pub fn policy_curve(&mut self, policy: &KosmoPolicy) -> Option<Curve> {
		let policy_index = find_policy_index(&self.policies, policy)?;

		let curve = self.shards
			.as_deref()
			.map(|shards| {
				self.histograms[policy_index].rescale_buckets(shards);

				Curve::from_corrected_histogram(
					&self.histograms[policy_index],
					shards,
				)
			})
			.unwrap_or(Curve::from_histogram(&self.histograms[policy_index]));

		Some(curve)
	}

	fn update_histograms(&mut self, access: &Access) -> Option<u64> {
		match self.global_table.get_mut(&access.key) {
			Some(global_object) => {
				let reuse_distances = global_object.reuse_distances();

				global_object.update(access);

				for (histogram, reuse_distance) in self.histograms.iter_mut().zip(&reuse_distances) {
					histogram.increment(self.shards.as_deref(), *reuse_distance);
				}

				math::max(&reuse_distances)
			},

			None => {
				for histogram in self.histograms.iter_mut() {
					histogram.increment(self.shards.as_deref(), None);
				}

				None
			},
		}
	}

	fn perform_evictions(&mut self, access: &Access, simulate_size: u64) {
		let step_size = math::max(&[
			MIN_RECONSTRUCTED_STACK_SIZE,
			access.size as u64,
			(simulate_size as f64 / self.granularity as f64).ceil() as u64
		]) as usize;

		if step_size > simulate_size as usize {
			return;
		}

		let mut policy_evictions: Vec<Evictions> = (step_size..(simulate_size as usize + step_size))
			.into_par_iter()
			.step_by(step_size)
			.map(|size| Kosmo::reconstruct_policy_stacks(
				&self.policies,
				size as u64,
				&self.global_table,
				access.key,
			))
			.collect();

		for (index, evictions) in policy_evictions.iter_mut().enumerate().rev() {
			let cache_size = ((index + 1) * step_size) as u64;

			for policy_index in 0..self.policies.len() {
				while let Some(key) = evictions.get_key(policy_index) {
					self.evict_with_key(policy_index, key, cache_size);
				}
			}
		}
	}

	fn evict_with_key(
		&mut self,
		policy_index: usize,
		key: Key,
		cache_size: u64
	) {
		if let Some(global_object) = self.global_table.get_mut(&key) {
			global_object.evict_by_policy_index(policy_index, cache_size);
		}
	}

	fn reconstruct_policy_stacks(
		policies: &[KosmoPolicy],
		size: u64,
		global_table: &FxHashMap<Key, GlobalObject>,
		exclude_key: Key,
	) -> Evictions {
		let mut stacks = init_reconstructed_stacks(policies, size);

		for global_object in global_table.values() {
			for (stack, eviction_map) in stacks.iter_mut().zip(global_object.eviction_maps()) {
				stack.insert(eviction_map.as_local_object(global_object, size));
			}
		}

		Evictions::new(stacks, exclude_key)
	}
}

fn init_reconstructed_stacks(policies: &[KosmoPolicy], size: u64) -> Vec<ReconstructedStackPolicy> {
	policies
		.iter()
		.map(|policy| ReconstructedStackPolicy::new(policy, size))
		.collect()
}

fn find_policy_index(policies: &[KosmoPolicy], policy: &KosmoPolicy) -> Option<usize> {
	for (index, kosmo_policy) in policies.iter().enumerate() {
		if kosmo_policy == policy {
			return Some(index);
		}
	}

	None
}

fn has_duplicate_policies(policies: &[KosmoPolicy]) -> bool {
	for (index, policy) in policies.iter().enumerate().take(policies.len() - 1) {
		for other in policies.iter().skip(index + 1) {
			if policy.eq(other) {
				return true;
			}
		}
	}

	false
}
