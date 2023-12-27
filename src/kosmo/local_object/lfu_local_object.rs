/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::cmp::{Ord, Ordering};

use crate::{
	access::{Key, Size},
	kosmo::global_object::GlobalObject,
	kosmo::local_object::LocalObject,
};

pub struct LfuLocalObject<'a> {
	global_object: &'a GlobalObject,
	count: Option<u64>,
}

impl<'a> LocalObject<'a> for LfuLocalObject<'a> {
	fn key(&self) -> Key {
		self.global_object.object().key
	}

	fn size(&self) -> Size {
		self.global_object.object().size
	}

	fn exists(&self) -> bool {
		self.count.is_some()
	}
}

impl<'a> LfuLocalObject<'a> {
	pub fn new(global_object: &'a GlobalObject, count: Option<u64>) -> Self {
		LfuLocalObject {
			global_object,
			count,
		}
	}
}

impl<'a> Ord for LfuLocalObject<'a> {
	fn cmp(&self, other: &Self) -> Ordering {
		match other.count.cmp(&self.count) {
			Ordering::Equal => {
				let timestamp = self.global_object.object().timestamp;
				let other_timestamp = other.global_object.object().timestamp;

				other_timestamp.cmp(&timestamp)
			},

			ord => ord,
		}
	}
}

impl<'a> PartialOrd for LfuLocalObject<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialEq for LfuLocalObject<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.global_object.eq(other.global_object)
	}
}

impl<'a> Eq for LfuLocalObject<'a> {}
