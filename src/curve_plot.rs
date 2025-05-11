/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use kwik::plot::{
	Plot,
	AxisFormat,
	line_plot::{LinePlot, Line},
};

use crate::curve::Curve;

/// A plot with multiple MRC curves.
#[derive(Default)]
pub struct CurvePlot {
	curves: Vec<Curve>,
	labels: Vec<Option<String>>,
}

impl CurvePlot {
	/// Returns the maximum size of all curves in the plot.
	pub fn get_max_size(&self) -> u64 {
		self.curves
			.iter()
			.max_by_key(|curve| curve.get_max_size())
			.map(|curve| curve.get_max_size())
			.unwrap_or(0)
	}

	/// Adds an MRC to the plot.
	pub fn add(&mut self, curve: Curve, label: Option<&str>) {
		self.curves.push(curve);
		self.labels.push(label.map(|label| label.to_owned()));
	}

	/// Converts the curves to a line plot.
	pub fn to_plot(&self) -> LinePlot {
		let max_size = self.get_max_size();
		let step_size = max_size / max_size.clamp(1, 100);

		let mut plot = LinePlot::default()
			.with_x_label("Size")
			.with_y_label("Miss ratio")
			.with_x_max(max_size)
			.with_y_min(0)
			.with_y_max(1)
			.with_y_tick(0.2)
			.with_x_format(AxisFormat::Memory);

		for (index, curve) in self.curves.iter().enumerate() {
			if curve.is_empty() {
				continue;
			}

			let mut line = Line::default();

			if let Some(label) = &self.labels[index] {
				line.set_label(label);
			}

			for size in (0..=max_size).step_by(step_size as usize) {
				line.push(size, curve.get_miss_ratio(size));
			}

			plot.line(line);
		}

		plot
	}
}
