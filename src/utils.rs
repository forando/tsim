use crate::utils::ShiftDirection::{LEFT, RIGHT};
use core::fmt;
use serde::export::fmt::Display;
use serde::export::Formatter;

pub struct FloatWrapper(f32);

impl FloatWrapper {
    pub fn new(float: f32) -> FloatWrapper {
        FloatWrapper(float)
    }

    pub fn rnd_decimals(&self, decimals: u32) -> f32 {
        let scale = 10_i32.pow(decimals) as f32;
        let y: f32 = (self.0 * scale).round() / scale;
        y
    }
}

struct Frequencies {
    results: Vec<usize>,
    max_freq: usize,
    canvas_height: usize,
}

impl Frequencies {
    fn new(results: Vec<usize>, canvas_height: usize) -> Frequencies {
        let max = results.iter().max().unwrap().clone();
        Frequencies {
            results,
            max_freq: max,
            canvas_height,
        }
    }

    fn canvas_val(&self, index: usize) -> usize {
        let freq = self.results[index];

        freq * self.canvas_height / self.max_freq
    }
}

pub struct Similarities {
    pub results: Vec<f32>,
    pub avg: f32,
    pub min: f32,
    pub max: f32,
    pub precision: usize,
}

impl Similarities {
    pub fn new(results: Vec<f32>, avg: f32, min: f32, max: f32, precision: usize) -> Similarities {
        Similarities {
            results,
            avg,
            min,
            max,
            precision,
        }
    }

    pub fn to_string(&self) -> String {
        let return_char_len = 1_usize;
        let similarity_text_len = self.precision + 2;
        let line_len = similarity_text_len + return_char_len;
        let container = String::with_capacity(self.results.len() * line_len);
        let mut result = self.results.iter().fold(container, |mut acc, sim| {
            let result = sim.to_string() + "\n";
            acc.push_str(&result);
            acc
        });
        result.pop().unwrap();
        result
    }

    fn build_x_axis(&self, width: usize) -> String {
        let min_shift_left = calculate_shift(self.min, LEFT);
        let min_shift_right = calculate_shift(self.min, RIGHT);
        let avg_shift_left = calculate_shift(self.avg, LEFT);
        let avg_shift_right = calculate_shift(self.avg, RIGHT);
        let max_shift_left = calculate_shift(self.max, LEFT);
        let max_shift_right = calculate_shift(self.max, RIGHT);

        let start_min_width = min_shift_left + 1;
        let start_to_min = "-".repeat(start_min_width - 1) + "|";

        let min_avg_width = self.calculate_min_avg_width(width);
        let min_to_avg = "-".repeat(min_avg_width - 1) + "|";

        let avg_max_width = self.calculate_avg_max_width(width);
        let avg_to_max = "-".repeat(avg_max_width - 1) + "|" + &"-".repeat(max_shift_right);

        let axis_line = start_to_min + &min_to_avg + &avg_to_max + "\n";

        let numbers = " ".repeat(start_min_width - min_shift_left - 1)
            + &self.min.to_string()
            + &" ".repeat(min_avg_width - min_shift_right - avg_shift_left - 1)
            + &self.avg.to_string()
            + &" ".repeat(avg_max_width - avg_shift_right - max_shift_left - 1)
            + &self.max.to_string();

        axis_line + &numbers
    }

    fn build_distributions(&self, width: usize) -> String {
        let canvas_width = self.canvas_width(width);
        let canvas_height = canvas_width * 30 / 100;

        let frequencies: Vec<usize> =
            self.results
                .iter()
                .fold(vec![0; canvas_width], |mut acc, item| {
                    let index = self.sim_to_axis_x_units(item, canvas_width);
                    acc[index] += 1;
                    acc
                });

        let frequencies = Frequencies::new(frequencies, canvas_height);

        let shift = self.canvas_shift_right();

        (0..canvas_height).rev().fold(
            String::with_capacity((canvas_width + 2) * canvas_height),
            |mut acc, height| {
                acc.push('\n');
                acc.push_str(&" ".repeat(shift));

                for i in 0..canvas_width {
                    if frequencies.canvas_val(i) >= height {
                        acc.push('x');
                    } else {
                        acc.push(' ');
                    }
                }
                acc
            },
        )
    }

    fn sim_to_axis_x_units(&self, val: &f32, canvas_width: usize) -> usize {
        ((val - self.min) * canvas_width as f32 / (self.max - self.min)) as usize - 1
    }

    fn sim_to_int(&self, val: f32) -> usize {
        (10_i32.pow(self.precision as u32) as f32 * val) as usize
    }

    fn canvas_width(&self, width: usize) -> usize {
        let min_avg_width = self.calculate_min_avg_width(width);
        let avg_max_width = self.calculate_avg_max_width(width);

        min_avg_width + avg_max_width + 1
    }

    fn canvas_shift_right(&self) -> usize {
        let min_shift_left = calculate_shift(self.min, LEFT);
        min_shift_left
    }

    fn calculate_min_avg_width(&self, width: usize) -> usize {
        let min_shift_left = calculate_shift(self.min, LEFT);
        let max_shift_right = calculate_shift(self.max, RIGHT);
        let max = self.sim_to_int(self.max);
        let min = self.sim_to_int(self.min);
        let avg = self.sim_to_int(self.avg);

        (width - min_shift_left - max_shift_right) * (avg - min) / (max - min)
    }

    fn calculate_avg_max_width(&self, width: usize) -> usize {
        let min_shift_left = calculate_shift(self.min, LEFT);
        let max_shift_right = calculate_shift(self.max, RIGHT);
        let min_avg_width = self.calculate_min_avg_width(width);

        width - max_shift_right - (min_shift_left + 1) - min_avg_width
    }
}

enum ShiftDirection {
    LEFT,
    RIGHT,
}

fn calculate_shift(val: f32, direction: ShiftDirection) -> usize {
    let size = val.to_string().len();
    if size < 3 {
        return 0;
    }

    let division = size / 2;
    let remainder = size % 2;
    let left_shift = division + remainder - 1;

    if let RIGHT = direction {
        return size - left_shift - 1;
    }

    left_shift
}

impl Display for Similarities {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut terminal_width = 100_usize;

        if let Some((w, _)) = term_size::dimensions() {
            terminal_width = w;
        }

        writeln!(f, "{}", self.build_distributions(terminal_width))?;
        writeln!(f, "{}", self.build_x_axis(terminal_width))?;

        writeln!(f, "Mean: {}", self.avg)?;
        writeln!(f, "Total similarities: {}", self.results.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::calculate_shift;
    use crate::utils::ShiftDirection::{LEFT, RIGHT};

    #[test]
    fn should_calculate_shift_left() {
        let val = 0.0_f32;
        let shift = calculate_shift(val, LEFT);
        assert_eq!(0, shift, "val=0.0");

        let val = 0.2_f32;
        let shift = calculate_shift(val, LEFT);
        assert_eq!(1, shift, "val=0.2");

        let val = 0.20_f32;
        let shift = calculate_shift(val, LEFT);
        assert_eq!(1, shift, "val=0.20");

        let val = 0.21_f32;
        let shift = calculate_shift(val, LEFT);
        assert_eq!(1, shift, "val=0.21");

        let val = 0.210_f32;
        let shift = calculate_shift(val, LEFT);
        assert_eq!(1, shift, "val=0.210");

        let val = 0.215_f32;
        let shift = calculate_shift(val, LEFT);
        assert_eq!(2, shift, "val=0.215");
    }

    #[test]
    fn should_calculate_shift_right() {
        let val = 0.0_f32;
        let shift = calculate_shift(val, RIGHT);
        assert_eq!(0, shift, "val=0.0");

        let val = 0.2_f32;
        let shift = calculate_shift(val, RIGHT);
        assert_eq!(1, shift, "val=0.2");

        let val = 0.20_f32;
        let shift = calculate_shift(val, RIGHT);
        assert_eq!(1, shift, "val=0.20");

        let val = 0.21_f32;
        let shift = calculate_shift(val, RIGHT);
        assert_eq!(2, shift, "val=0.21");

        let val = 0.210_f32;
        let shift = calculate_shift(val, RIGHT);
        assert_eq!(2, shift, "val=0.210");

        let val = 0.215_f32;
        let shift = calculate_shift(val, RIGHT);
        assert_eq!(2, shift, "val=0.215");
    }
}
