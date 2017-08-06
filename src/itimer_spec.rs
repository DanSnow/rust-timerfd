use std::time::Duration;
use nix::sys::time::{TimeSpec, TimeValLike};
pub use libc::{CLOCK_REALTIME, CLOCK_MONOTONIC};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ITimerSpec {
    pub it_interval: TimeSpec,
    pub it_value: TimeSpec,
}

impl ITimerSpec {
    pub fn new(interval: Duration, value: Duration) -> Self {
        ITimerSpec {
            it_interval: duration_to_timespec(interval),
            it_value: duration_to_timespec(value),
        }
    }

    pub fn seconds(value: i64) -> Self {
        ITimerSpec {
            it_interval: TimeSpec::zero(),
            it_value: TimeSpec::seconds(value),
        }
    }

    pub fn nanoseconds(value: i64) -> Self {
        ITimerSpec {
            it_interval: TimeSpec::zero(),
            it_value: TimeSpec::nanoseconds(value),
        }
    }

    pub fn interval_seconds(&self, value: i64) -> Self {
        ITimerSpec {
            it_interval: TimeSpec::seconds(value),
            it_value: self.it_value,
        }
    }

    pub fn interval_nanoseconds(&self, value: i64) -> Self {
        ITimerSpec {
            it_interval: TimeSpec::nanoseconds(value),
            it_value: self.it_value,
        }
    }
}

impl From<Duration> for ITimerSpec {
    fn from(duration: Duration) -> Self {
        ITimerSpec {
            it_interval: TimeSpec::zero(),
            it_value: duration_to_timespec(duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{size_of, align_of};

    #[test]
    fn test_itimer_spec_layout() {
        assert_eq!(
            size_of::<ITimerSpec>(),
            32usize,
            concat!("Size of: ", stringify!(ITimerSpec))
        );
        assert_eq!(
            align_of::<ITimerSpec>(),
            8usize,
            concat!("Alignment of ", stringify!(ITimerSpec))
        );
    }
}

#[inline]
fn duration_to_timespec(duration: Duration) -> TimeSpec {
    TimeSpec::nanoseconds(duration.as_secs() as i64 + duration.subsec_nanos() as i64)
}
