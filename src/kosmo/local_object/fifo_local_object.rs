/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::cmp::{Ord, Ordering};

use crate::{
	access::{Timestamp, Key, Size},
	kosmo::global_object::GlobalObject,
	kosmo::local_object::LocalObject,
};

pub struct FifoLocalObject<'a> {
	global_object: &'a GlobalObject,
	inserted_timestamp: Option<u64>,
}

impl<'a> LocalObject<'a> for FifoLocalObject<'a> {
	fn key(&self) -> Key {
		self.global_object.object().key
	}

	fn size(&self) -> Size {
		self.global_object.object().size
	}

	fn exists(&self) -> bool {
		self.inserted_timestamp.is_some()
	}
}

impl<'a> FifoLocalObject<'a> {
	pub fn new(global_object: &'a GlobalObject, inserted_timestamp: Option<Timestamp>) -> Self {
		FifoLocalObject {
			global_object,
			inserted_timestamp,
		}
	}
}

impl<'a> Ord for FifoLocalObject<'a> {
	fn cmp(&self, other: &Self) -> Ordering {
		other.inserted_timestamp.cmp(&self.inserted_timestamp)
	}
}

impl<'a> PartialOrd for FifoLocalObject<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialEq for FifoLocalObject<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.global_object.eq(other.global_object)
	}
}

impl<'a> Eq for FifoLocalObject<'a> {}
