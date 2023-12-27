/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::cmp;

use gnuplot::{
	Axes2D,
	Caption,
	Color,
	LineWidth,
	LineStyle,
	DashType,
	AxesCommon,
	AutoOption,
	BorderLocation2D,
	TickOption,
	PointSymbol,
	PointSize,
};

use kwik::{
	math,
	fmt::MEMORY_UNITS,
};

use crate::{
	figure::Plot,
	curve::Curve,
};

const COLOURS: &[&str] = &[
	"#c4342b",
	"#0071ad",
	"#71ad00",
	"#554ec9",
	"#f7790d",
	"#e0ca3c",
	"#47a8bd",
];

const DASH_TYPES: &[DashType] = &[
	DashType::Solid,
	DashType::Dash,
	DashType::DotDash,
	DashType::DotDotDash,
	DashType::Dot,
];

#[derive(Default)]
pub struct CurvePlot {
	title: Option<String>,
	scaler: SizeScaler,

	curve_plots_data: Vec<CurvePlotData>,

	lines: Vec<u64>,
	points: Vec<(u64, f64)>,
}

struct CurvePlotData {
	name: String,

	cache_sizes: Vec<u64>,
	miss_ratios: Vec<f64>,
}

pub struct SizeScaler {
	unit: &'static str,
	denominator: f64,
}

impl CurvePlot {
	pub fn get_max_size(&self) -> u64 {
		let mut max_size: u64 = 0;

		for curve_plots_data in &self.curve_plots_data {
			max_size = math::max(&[
				max_size,
				curve_plots_data.get_max_size()
			]);
		}

		max_size
	}

	pub fn add(&mut self, name: &str, curve: &Curve) {
		let curve_plot_data = CurvePlotData::new(name, curve);

		if !curve_plot_data.is_empty() {
			self.scaler.update(curve_plot_data.get_max_size());
			self.curve_plots_data.push(curve_plot_data);
		}
	}
}

impl Plot for CurvePlot {
	fn is_empty(&self) -> bool {
		self.curve_plots_data.is_empty()
	}

	fn configure(&mut self, axes: &mut Axes2D) {
		axes
			.set_border(
				false,
				&[
					BorderLocation2D::Bottom,
					BorderLocation2D::Left,
				],
				&[]
			)
			.set_x_range(
				AutoOption::Fix(0.0),
				AutoOption::Fix(self.scaler.scale(self.get_max_size()))
			)
			.set_y_range(
				AutoOption::Fix(0.0),
				AutoOption::Fix(1.0)
			)
			.set_x_ticks(
				Some((
					AutoOption::Auto,
					0
				)),
				&[
					TickOption::Mirror(false),
					TickOption::Inward(false)
				],
				&[]
			)
			.set_y_ticks(
				Some((
					AutoOption::Fix(0.2),
					0
				)),
				&[
					TickOption::Mirror(false),
					TickOption::Inward(false)
				],
				&[]
			)
			.set_grid_options(false, &[
				Color("#bbbbbb"),
				LineWidth(2.0),
				LineStyle(DashType::Dot),
			])
			.set_x_grid(true)
			.set_y_grid(true)
			.set_x_label(&self.scaler.get_x_label(), &[])
			.set_y_label("Miss ratio (1)", &[]);

		if let Some(title) = &self.title {
			axes.set_title(title, &[]);
		}

		let max_size = self.get_max_size();
		let step_size = max_size / cmp::min(cmp::max(max_size, 1), 100);
		let mut line_index: usize = 0;

		if step_size == 0 {
			return;
		}

		for curve_plot_data in &self.curve_plots_data {
			if curve_plot_data.is_empty() {
				continue;
			}

			let mut curve = Curve::default();
			let cache_sizes = curve_plot_data.get_cache_sizes();
			let miss_ratios = curve_plot_data.get_miss_ratios();

			for (cache_size, miss_ratio) in cache_sizes.iter().zip(miss_ratios.iter()) {
				curve.add(*cache_size, *miss_ratio);
			}

			let mut x = Vec::<f64>::new();
			let mut y = Vec::<f64>::new();

			(0..(max_size + step_size))
				.step_by(step_size as usize)
				.for_each(|size| {
					x.push(self.scaler.scale(size));
					y.push(curve.get_miss_ratio(size));
				});

			axes.lines(x, y, &[
				Caption(curve_plot_data.name.as_str()),
				LineWidth(3.0),
				Color(COLOURS[line_index % COLOURS.len()]),
				LineStyle(DASH_TYPES[line_index% DASH_TYPES.len()]),
			]);

			line_index += 1;
		}

		for line in &self.lines {
			let scaled_size = self.scaler.scale(*line);

			let x = vec![scaled_size, scaled_size];
			let y = vec![0f64, 1f64];

			axes.lines(x, y, &[
				LineWidth(1.0),
				Color("blue"),
			]);
		}

		for point in &self.points {
			let scaled_size = self.scaler.scale(point.0);

			let x = vec![scaled_size];
			let y = vec![point.1];

			axes.points(x, y, &[
				PointSymbol('o'),
				PointSize(0.75),
				Color("green"),
			]);
		}
	}
}

impl CurvePlotData {
	fn new(name: &str, curve: &Curve) -> Self {
		let mut cache_sizes = Vec::<u64>::new();
		let mut miss_ratios = Vec::<f64>::new();

		for point in curve.into_iter() {
			cache_sizes.push(point.get_size());
			miss_ratios.push(point.get_miss_ratio());
		}

		CurvePlotData {
			name: name.to_owned(),

			cache_sizes,
			miss_ratios,
		}
	}

	fn is_empty(&self) -> bool {
		self.cache_sizes.is_empty()
	}

	fn get_max_size(&self) -> u64 {
		match self.cache_sizes.last() {
			Some(cache_size) => *cache_size,
			None => 0,
		}
	}

	fn get_cache_sizes(&self) -> &[u64] {
		&self.cache_sizes
	}

	fn get_miss_ratios(&self) -> &[f64] {
		&self.miss_ratios
	}
}

impl SizeScaler {
	fn new(max_size: u64) -> Self {
		let (count, denominator) = SizeScaler::get_scalers(max_size);

		Self {
			unit: MEMORY_UNITS[count],
			denominator,
		}
	}

	fn update(&mut self, max_size: u64) {
		let (count, denominator) = SizeScaler::get_scalers(max_size);

		if denominator > self.denominator {
			self.unit = MEMORY_UNITS[count];
			self.denominator = denominator;
		}
	}

	pub fn scale(&self, size: u64) -> f64 {
		size as f64 / self.denominator
	}

	pub fn get_x_label(&self) -> String {
		let mut xlabel: String = "Size (".to_owned();

		xlabel.push_str(self.unit);
		xlabel.push(')');

		xlabel
	}

	fn get_scalers(mut max_size: u64) -> (usize, f64) {
		let mut count: usize = 0;
		let mut denominator: f64 = 1.0;

		while max_size / 1024 > 0 {
			denominator *= 1024.0;
			max_size /= 1024;
			count += 1;
		}

		(count, denominator)
	}
}

impl Default for SizeScaler {
	fn default() -> Self {
		SizeScaler::new(0)
	}
}
