use std::{
    ops::{Add, AddAssign},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualInstant {
    elapsed: Duration,
}

impl VirtualInstant {
    fn zero() -> Self {
        Self {
            elapsed: Duration::ZERO,
        }
    }
}
impl Add<Duration> for VirtualInstant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self {
            elapsed: self.elapsed + rhs,
        }
    }
}
impl AddAssign<Duration> for VirtualInstant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

/// A time manager that provides independent time control for the simulation
/// Allows speeding up, slowing down, and pausing the simulation
#[derive(Debug, Clone)]
pub struct TimeManager {
    inner: Arc<Mutex<TimeManagerInner>>,
}
#[derive(Debug)]
struct TimeManagerInner {
    /// Real time when the manager was created or last reset
    start_real_time: Instant,
    /// Virtual time elapsed since start
    virtual_instance: VirtualInstant,
    /// Last virtual delta time
    last_virtual_delta: Duration,
    /// Speed multiplier (1.0 = normal speed, 2.0 = double speed, 0.5 = half speed)
    speed_multiplier: f64,
    /// Last update time for delta calculations
    last_update: Instant,
}

impl TimeManager {
    /// Create a new TimeManager with normal speed (1.0x)
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            inner: Arc::new(Mutex::new(TimeManagerInner {
                start_real_time: now,
                virtual_instance: VirtualInstant::zero(),
                last_virtual_delta: Duration::ZERO,
                speed_multiplier: 1.0,
                last_update: now,
            })),
        }
    }

    /// Get the current virtual time
    pub fn now(&self) -> VirtualInstant {
        self.inner.lock().unwrap().virtual_instance
    }

    /// Get the last virtual delta time in seconds
    pub fn last_virtual_delta(&self) -> f32 {
        self.inner.lock().unwrap().last_virtual_delta.as_secs_f32()
    }

    /// Update the virtual time based on real time and current settings
    /// This should be called once per frame
    pub fn update(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();

        let real_delta = now.duration_since(inner.last_update);
        let virtual_delta =
            Duration::from_secs_f64(real_delta.as_secs_f64() * inner.speed_multiplier);
        inner.last_virtual_delta = virtual_delta;
        inner.virtual_instance += virtual_delta;

        inner.last_update = now;
    }

    /// Set the speed multiplier
    /// - 1.0 = normal speed
    /// - 2.0 = double speed
    /// - 0.5 = half speed
    pub fn set_speed(&mut self, multiplier: f64) {
        self.inner.lock().unwrap().speed_multiplier = multiplier.max(0.0);
    }

    /// Get the current speed multiplier
    pub fn speed(&self) -> f64 {
        self.inner.lock().unwrap().speed_multiplier
    }

    /// Reset the time manager to initial state
    pub fn reset(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();
        inner.start_real_time = now;
        inner.virtual_instance = VirtualInstant::zero();
        inner.last_update = now;
    }

    /// Get the virtual time as a formatted string (MM:SS.mmm)
    pub fn format_time(&self) -> String {
        let total_secs = self
            .inner
            .lock()
            .unwrap()
            .virtual_instance
            .elapsed
            .as_secs_f64();
        let minutes = (total_secs / 60.0) as u32;
        let seconds = total_secs % 60.0;
        format!("{minutes:02}:{seconds:06.3}")
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}
