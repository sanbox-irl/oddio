use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Controlled, Filter, Signal};

const PLAY: usize = 0;
const PAUSE: usize = 1;
const STOP: usize = 2;

/// A source that can be paused or permanently stopped
pub struct Stop<T: ?Sized> {
    state: AtomicUsize,
    inner: T,
}

impl<T> Stop<T> {
    pub(crate) fn new(signal: T) -> Self {
        Self {
            state: AtomicUsize::new(PLAY),
            inner: signal,
        }
    }
}

impl<T: ?Sized> Stop<T> {
    /// Stop the source for good
    pub(crate) fn stop(&self) {
        self.state.store(STOP, Ordering::Relaxed);
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.state.load(Ordering::Relaxed) == PAUSE
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.state.load(Ordering::Relaxed) == STOP
    }
}

impl<T: Signal + ?Sized> Signal for Stop<T> {
    type Frame = T::Frame;

    fn sample(&self, interval: f32, out: &mut [T::Frame]) {
        self.inner.sample(interval, out);
    }

    fn remaining(&self) -> f32 {
        let state = self.state.load(Ordering::Relaxed);
        match state {
            PLAY => self.inner.remaining(),
            PAUSE => f32::INFINITY,
            _ => 0.0,
        }
    }
}

impl<T> Filter for Stop<T> {
    type Inner = T;
    fn inner(&self) -> &T {
        &self.inner
    }
}

/// Thread-safe control for a [`Stop`] filter
#[derive(Copy, Clone)]
pub struct StopControl<'a, T>(&'a Stop<T>);

unsafe impl<'a, T: 'a> Controlled<'a> for Stop<T> {
    type Control = StopControl<'a, T>;

    unsafe fn make_control(signal: &'a Stop<T>) -> Self::Control {
        StopControl(signal)
    }
}

impl<'a, T> StopControl<'a, T> {
    /// Suspend playback of the source
    pub fn pause(&self) {
        self.0.state.store(PAUSE, Ordering::Relaxed);
    }

    /// Resume the paused source
    pub fn resume(&self) {
        self.0.state.store(PLAY, Ordering::Relaxed);
    }

    /// Stop the source for good
    pub fn stop(&self) {
        self.0.state.store(STOP, Ordering::Relaxed);
    }

    /// Whether the source is paused
    pub fn is_paused(&self) -> bool {
        self.0.state.load(Ordering::Relaxed) == PAUSE
    }

    /// Whether the source has stopped
    pub fn is_stopped(&self) -> bool {
        self.0.state.load(Ordering::Relaxed) == STOP
    }
}
