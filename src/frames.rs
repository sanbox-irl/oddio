use std::{
    alloc,
    cell::Cell,
    mem,
    ops::{Deref, DerefMut},
    ptr,
    sync::Arc,
};

use crate::{frame, Frame, Signal};

/// A sequence of static audio frames at a particular sample rate
///
/// Used to store e.g. sound effects decoded from files on disk.
///
/// Dynamically sized type. Typically stored inside an `Arc`, allowing efficient simultaneous use by
/// multiple signals.
#[derive(Debug)]
pub struct Frames<T> {
    rate: f64,
    samples: [T],
}

impl<T: Frame + Copy> Frames<T> {
    /// Construct samples from existing memory
    pub fn from_slice(rate: u32, samples: &[T]) -> Arc<Self> {
        let header_layout = alloc::Layout::new::<f64>();
        let (layout, payload_offset) = header_layout
            .extend(
                alloc::Layout::from_size_align(
                    mem::size_of::<T>() * samples.len(),
                    mem::align_of::<T>(),
                )
                .unwrap(),
            )
            .unwrap();
        let layout = layout.pad_to_align();
        unsafe {
            let mem = alloc::alloc(layout);
            mem.cast::<f64>().write(rate.into());
            let payload = mem.add(payload_offset).cast::<T>();
            for (i, &x) in samples.iter().enumerate() {
                payload.add(i).write(x);
            }
            Box::from_raw(ptr::slice_from_raw_parts_mut(mem, samples.len()) as *mut Self).into()
        }
    }

    /// Generate samples from an iterator
    pub fn from_iter<I>(rate: u32, iter: I) -> Arc<Self>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let len = iter.len();
        let header_layout = alloc::Layout::new::<f64>();
        let (layout, payload_offset) = header_layout
            .extend(
                alloc::Layout::from_size_align(mem::size_of::<T>() * len, mem::align_of::<T>())
                    .unwrap(),
            )
            .unwrap();
        let layout = layout.pad_to_align();
        unsafe {
            let mem = alloc::alloc(layout);
            mem.cast::<f64>().write(rate.into());
            let payload = mem.add(payload_offset).cast::<T>();
            let mut n = 0;
            for (i, x) in iter.enumerate() {
                payload.add(i).write(x);
                n += 1;
            }
            assert_eq!(n, len, "iterator returned incorrect length");
            Box::from_raw(ptr::slice_from_raw_parts_mut(mem, len) as *mut Self).into()
        }
    }

    /// Number of samples per second
    pub fn rate(&self) -> u32 {
        self.rate as u32
    }

    /// Interpolate a frame for position `s`
    ///
    /// Note that `s` is in samples, not seconds. Whole numbers are always an exact sample, and
    /// out-of-range positions yield 0.
    pub fn interpolate(&self, s: f64) -> T {
        let x0 = s.trunc() as isize;
        let fract = s.fract() as f32;
        let x1 = x0 + 1;
        let a = self.get(x0);
        let b = self.get(x1);
        frame::lerp(&a, &b, fract)
    }

    fn get(&self, sample: isize) -> T {
        if sample < 0 {
            return T::ZERO;
        }
        let sample = sample as usize;
        if sample >= self.samples.len() {
            return T::ZERO;
        }
        self.samples[sample]
    }
}

impl<T> Deref for Frames<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.samples
    }
}

impl<T> DerefMut for Frames<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.samples
    }
}

/// An audio signal backed by a static sequence of samples
#[derive(Debug, Clone)]
pub struct FramesSignal<T> {
    /// Frames to play
    data: Arc<Frames<T>>,
    /// Playback position in seconds
    t: Cell<f64>,
}

impl<T> FramesSignal<T> {
    /// Create an audio signal from some samples
    ///
    /// `start_seconds` adjusts the initial playback position, and may be negative.
    pub fn new(data: Arc<Frames<T>>, start_seconds: f64) -> Self {
        Self {
            t: Cell::new(start_seconds),
            data,
        }
    }
}

impl<T: Frame + Copy> Signal for FramesSignal<T> {
    type Frame = T;

    #[inline]
    fn sample(&self, interval: f32, out: &mut [T]) {
        let s0 = self.t.get() * self.data.rate;
        let ds = f64::from(interval) * self.data.rate;
        for (i, o) in out.iter_mut().enumerate() {
            *o = self.data.interpolate(s0 + ds * i as f64);
        }
        self.t
            .set(self.t.get() + f64::from(interval) * out.len() as f64);
    }

    #[inline]
    fn remaining(&self) -> f32 {
        (self.data.samples.len() as f64 / self.data.rate - self.t.get()) as f32
    }
}

impl<T> From<Arc<Frames<T>>> for FramesSignal<T> {
    fn from(samples: Arc<Frames<T>>) -> Self {
        Self::new(samples, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slice() {
        const DATA: &[f32] = &[1.0, 2.0, 3.0];
        let frames = Frames::from_slice(1, DATA);
        assert_eq!(&frames[..], DATA);
    }
}
