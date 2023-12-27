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

pub struct TwoQLocalObject<'a> {
	global_object: &'a GlobalObject,
	stack_location: Option<StackLocation>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StackLocation {
	A1(Timestamp),
	Am,
}

impl<'a> LocalObject<'a> for TwoQLocalObject<'a> {
	fn key(&self) -> Key {
		self.global_object.object().key
	}

	fn size(&self) -> Size {
		self.global_object.object().size
	}

	fn exists(&self) -> bool {
		self.stack_location.is_some()
	}
}

impl<'a> TwoQLocalObject<'a> {
	pub fn new(
		global_object: &'a GlobalObject,
		stack_location: Option<StackLocation>,
	) -> Self {
		TwoQLocalObject {
			global_object,
			stack_location,
		}
	}

	pub fn stack_location(&self) -> Option<&StackLocation> {
		self.stack_location.as_ref()
	}
}

impl<'a> Ord for TwoQLocalObject<'a> {
	fn cmp(&self, other: &Self) -> Ordering {
		let Some(stack_location) = &self.stack_location else {
			return Ordering::Greater;
		};

		let Some(other_stack_location) = &other.stack_location else {
			return Ordering::Less;
		};

		match (stack_location, other_stack_location) {
			(StackLocation::Am, StackLocation::Am) => {
				let timestamp = self.global_object.object().timestamp;
				let other_timestamp = other.global_object.object().timestamp;

				other_timestamp.cmp(&timestamp)
			},

			(StackLocation::Am, _) => Ordering::Less,
			(_, StackLocation::Am) => Ordering::Greater,

			(
				StackLocation::A1(inserted_timestamp),
				StackLocation::A1(other_inserted_timestamp),
			) => other_inserted_timestamp.cmp(inserted_timestamp),
		}
	}
}

impl<'a> PartialOrd for TwoQLocalObject<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialEq for TwoQLocalObject<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.global_object.eq(other.global_object)
	}
}

impl<'a> Eq for TwoQLocalObject<'a> {}
