/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub mod lfu_local_object;
pub mod fifo_local_object;
pub mod two_q_local_object;
pub mod lrfu_local_object;
pub mod lru_local_object;

use crate::access::{Key, Size};

pub trait LocalObject<'a> {
	fn key(&self) -> Key;
	fn size(&self) -> Size;

	fn exists(&self) -> bool;
}

pub enum LocalObjectPolicy<'a> {
	Lfu(LfuLocalObject<'a>),
	Fifo(FifoLocalObject<'a>),
	TwoQ(TwoQLocalObject<'a>),
	Lrfu(LrfuLocalObject<'a>),
	Lru(LruLocalObject<'a>),
}

impl<'a> LocalObject<'a> for LocalObjectPolicy<'a> {
	fn key(&self) -> Key {
		match self {
			LocalObjectPolicy::Lfu(local_object) => local_object.key(),
			LocalObjectPolicy::Fifo(local_object) => local_object.key(),
			LocalObjectPolicy::TwoQ(local_object) => local_object.key(),
			LocalObjectPolicy::Lrfu(local_object) => local_object.key(),
			LocalObjectPolicy::Lru(local_object) => local_object.key(),
		}
	}

	fn size(&self) -> Size {
		match self {
			LocalObjectPolicy::Lfu(local_object) => local_object.size(),
			LocalObjectPolicy::Fifo(local_object) => local_object.size(),
			LocalObjectPolicy::TwoQ(local_object) => local_object.size(),
			LocalObjectPolicy::Lrfu(local_object) => local_object.size(),
			LocalObjectPolicy::Lru(local_object) => local_object.size(),
		}
	}

	fn exists(&self) -> bool {
		match self {
			LocalObjectPolicy::Lfu(local_object) => local_object.exists(),
			LocalObjectPolicy::Fifo(local_object) => local_object.exists(),
			LocalObjectPolicy::TwoQ(local_object) => local_object.exists(),
			LocalObjectPolicy::Lrfu(local_object) => local_object.exists(),
			LocalObjectPolicy::Lru(local_object) => local_object.exists(),
		}
	}
}

pub use crate::kosmo::local_object::{
	lfu_local_object::LfuLocalObject,
	fifo_local_object::FifoLocalObject,
	two_q_local_object::TwoQLocalObject,
	lrfu_local_object::LrfuLocalObject,
	lru_local_object::LruLocalObject,
};
