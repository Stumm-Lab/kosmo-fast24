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

pub struct LrfuLocalObject<'a> {
	global_object: &'a GlobalObject,
	crf: Option<f64>,
}

impl<'a> LocalObject<'a> for LrfuLocalObject<'a> {
	fn key(&self) -> Key {
		self.global_object.object().key
	}

	fn size(&self) -> Size {
		self.global_object.object().size
	}

	fn exists(&self) -> bool {
		self.crf.is_some()
	}
}

impl<'a> LrfuLocalObject<'a> {
	pub fn new(global_object: &'a GlobalObject, crf: Option<f64>) -> Self {
		LrfuLocalObject {
			global_object,
			crf,
		}
	}
}

impl<'a> Ord for LrfuLocalObject<'a> {
	fn cmp(&self, other: &Self) -> Ordering {
		match other.crf.unwrap().total_cmp(&self.crf.unwrap()) {
			Ordering::Equal => {
				let timestamp = self.global_object.object().timestamp;
				let other_timestamp = other.global_object.object().timestamp;

				other_timestamp.cmp(&timestamp)
			},

			ordering => ordering,
		}
	}
}

impl<'a> PartialOrd for LrfuLocalObject<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialEq for LrfuLocalObject<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.global_object.eq(other.global_object)
	}
}

impl<'a> Eq for LrfuLocalObject<'a> {}
