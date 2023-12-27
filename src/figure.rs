/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	io::{Error, ErrorKind},
	cmp,
};

use gnuplot::{
	Figure as GnuplotFigure,
	Axes2D,
};

pub struct Figure {
	figure: GnuplotFigure,

	columns: usize,
	count: usize,

	plot_width_px: f32,
	plot_height_px: f32,
}

const DPI: f32 = 72.0;

pub trait Plot {
	fn is_empty(&self) -> bool;
	fn configure(&mut self, _: &mut Axes2D);
}

impl Figure {
	pub fn new(columns: usize) -> Self {
		Figure {
			figure: GnuplotFigure::new(),

			columns,
			count: 0,

			plot_width_px: 323.0,
			plot_height_px: 201.0,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.count == 0
	}

	pub fn add(&mut self, plot: &mut impl Plot) {
		if plot.is_empty() {
			return;
		}

		self.count += 1;

		self.figure.set_multiplot_layout(
			(self.count as f32 / self.columns as f32).ceil() as usize,
			*cmp::min(&self.count, &self.columns)
		);

		plot.configure(self.figure.axes2d());
	}

	pub fn save(&mut self, path: &str) -> Result<(), Error> {
		if self.is_empty() {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Could not save figure with no plots."
			));
		}

		let columns = cmp::min(&self.count, &self.columns);
		let rows = (self.count as f32 / self.columns as f32).ceil() as u32;

		let plot_width_in = self.plot_width_px / DPI;
		let plot_height_in = self.plot_height_px / DPI;

		let width = *columns as f32 * plot_width_in;
		let height = rows as f32 * plot_height_in;

		match self.figure.save_to_pdf(path, width, height) {
			Ok(_) => Ok(()),
			Err(_) => Err(Error::new(
				ErrorKind::PermissionDenied,
				"Could not save figure."
			))
		}
	}
}
