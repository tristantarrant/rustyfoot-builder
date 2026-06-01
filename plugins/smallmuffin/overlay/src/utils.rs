/*
 * Copyright (c) 2026 YimRakhee
 * Licensed under the GNU General Public License v3.0 (or later).
 * See the LICENSE file in the project root for more information.
 */

use std::f32::consts::PI;

#[derive(Clone, Default)]
pub struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl Biquad {
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1; self.x1 = x;
        self.y2 = self.y1; self.y1 = y;
        y
    }

    pub fn make_low_shelf(&mut self, sample_rate: f32, freq: f32, q: f32, gain_linear: f32) {
        let a = gain_linear.sqrt();
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();
        let a_plus_1 = a + 1.0; let a_minus_1 = a - 1.0;
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * (a_plus_1 - a_minus_1 * cos_w0 + two_sqrt_a_alpha);
        let b1 = 2.0 * a * (a_minus_1 - a_plus_1 * cos_w0);
        let b2 = a * (a_plus_1 - a_minus_1 * cos_w0 - two_sqrt_a_alpha);
        let a0 = a_plus_1 + a_minus_1 * cos_w0 + two_sqrt_a_alpha;
        let a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_w0);
        let a2 = a_plus_1 + a_minus_1 * cos_w0 - two_sqrt_a_alpha;

        self.set_coeffs(b0, b1, b2, a0, a1, a2);
    }

    pub fn make_high_shelf(&mut self, sample_rate: f32, freq: f32, q: f32, gain_linear: f32) {
        let a = gain_linear.sqrt();
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();
        let a_plus_1 = a + 1.0; let a_minus_1 = a - 1.0;
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * (a_plus_1 + a_minus_1 * cos_w0 + two_sqrt_a_alpha);
        let b1 = -2.0 * a * (a_minus_1 + a_plus_1 * cos_w0);
        let b2 = a * (a_plus_1 + a_minus_1 * cos_w0 - two_sqrt_a_alpha);
        let a0 = a_plus_1 - a_minus_1 * cos_w0 + two_sqrt_a_alpha;
        let a1 = 2.0 * (a_minus_1 - a_plus_1 * cos_w0);
        let a2 = a_plus_1 - a_minus_1 * cos_w0 - two_sqrt_a_alpha;

        self.set_coeffs(b0, b1, b2, a0, a1, a2);
    }

    pub fn make_peak(&mut self, sample_rate: f32, freq: f32, q: f32, gain_db: f32) {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha / a;

        self.set_coeffs(b0, b1, b2, a0, a1, a2);
    }

    pub fn make_high_pass(&mut self, sample_rate: f32, freq: f32, q: f32) {
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let b0 = (1.0 + cos_w0) / 2.0;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        self.set_coeffs(b0, b1, b2, a0, a1, a2);
    }

    pub fn make_low_pass(&mut self, sample_rate: f32, freq: f32, q: f32) {
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let b0 = (1.0 - cos_w0) / 2.0;
        let b1 = 1.0 - cos_w0;
        let b2 = (1.0 - cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        self.set_coeffs(b0, b1, b2, a0, a1, a2);
    }

    fn set_coeffs(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0; self.b1 = b1 / a0; self.b2 = b2 / a0;
        self.a1 = a1 / a0; self.a2 = a2 / a0;
    }
}

pub fn db_to_gain(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.log10()
}

pub struct Oversampler2x {
    up_filter: Biquad,
    down_filter: Biquad,
}

impl Oversampler2x {
    pub fn new() -> Self {
        Self {
            up_filter: Biquad::default(),
            down_filter: Biquad::default(),
        }
    }

    pub fn reset(&mut self, sample_rate: f32) {
        self.up_filter.make_low_pass(sample_rate * 2.0, sample_rate * 0.45, 0.707);
        self.down_filter.make_low_pass(sample_rate * 2.0, sample_rate * 0.45, 0.707);
    }

    pub fn upsample(&mut self, sample: f32) -> [f32; 2] {
        let s1 = self.up_filter.process(sample * 2.0);
        let s2 = self.up_filter.process(0.0);
        [s1, s2]
    }

    pub fn downsample(&mut self, samples: [f32; 2]) -> f32 {
        self.down_filter.process(samples[0]);
        self.down_filter.process(samples[1])
    }
}


pub fn soft_clip(sample: f32, drive: f32) -> f32 {
    (sample * drive).tanh()
}

pub fn asym_hard_clip(sample: f32, bias: f32) -> f32 {
    let offset = bias * 0.8;
    let mut clp = sample + offset;
    clp = clp.clamp(-1.0, 1.0 - bias * 0.5);
    clp - offset * 0.5
}
