#[macro_use]
extern crate bitflags;
extern crate nix;
extern crate libc;

mod itimer_spec;
mod sys;

use std::mem;
use std::ptr;
use std::os::unix::io::{RawFd, AsRawFd};
use nix::{Errno, Error};
use nix::unistd;
use libc::{c_int, clockid_t};

pub use self::itimer_spec::*;

#[doc(hidden)]
const TIMERFD_DATA_SIZE: usize = 8;

bitflags! {
    /// Flags for TimerFd
    #[derive(Default)]
    pub struct TFDFlags: c_int {
        /// Set close-on-exec on TimerFd
        const TFD_CLOSEXEC = 524288;
        /// Set TimerFd to non-block mode
        const TFD_NONBLOCK = 2048;
    }
}

bitflags! {
    /// Flags for TimerFd::set_time
    #[derive(Default)]
    pub struct TFDTimerFlags: c_int {
        /// Set an absolute timer
        const TFD_TIMER_ABSTIME = 1;
    }
}


#[inline]
pub fn timerfd_create(clock_id: clockid_t, flags: TFDFlags) -> nix::Result<RawFd> {
    unsafe { Errno::result(sys::timerfd_create(clock_id, flags.bits())) }
}

#[inline]
pub fn timerfd_settime(
    fd: RawFd,
    flags: TFDTimerFlags,
    utmr: &ITimerSpec,
    otmr: Option<&mut ITimerSpec>,
) -> nix::Result<()> {
    let res = unsafe {
        sys::timerfd_settime(
            fd,
            flags.bits(),
            utmr as *const ITimerSpec,
            otmr.map(|x| x as *mut ITimerSpec)
                .unwrap_or(ptr::null_mut()),
        )
    };
    if res == -1 {
        return Err(Error::last());
    }
    Ok(())
}

#[inline]
pub fn timerfd_gettime(fd: RawFd, otmr: &mut ITimerSpec) -> nix::Result<()> {
    let res = unsafe { sys::timerfd_gettime(fd, otmr as *mut ITimerSpec) };
    if res == -1 {
        return Err(Error::last());
    }
    Ok(())
}

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum ClockId {
    /// A settable system-wide clock
    Realtime = CLOCK_REALTIME,
    /// A nonsettable clock which is not affected discontinuous changes in the system clock
    Monotonic = CLOCK_MONOTONIC,
}

/// A helper struct for creating, reading, and closing a `timerfd` instance.
///
/// ## Example
///
/// ```
/// use timerfd::{TimerFd, ClockId, ITimerSpec};
///
/// let mut timerfd = TimerFd::new(ClockId::Monotonic).unwrap();
///
/// // Set timer
/// timerfd.set_time(&ITimerSpec::seconds(3), None).unwrap();
///
/// match timerfd.read_time() {
///     // Timer is expired
///     Ok(Some(expirations)) => {},
///     // There is no expired timer. (Only happend when TFD_NONBLOCK set)
///     Ok(None) => {},
///     Err(err) =>  {}, // Some error happend
/// }
/// ```
#[derive(Debug)]
pub struct TimerFd(RawFd);

impl TimerFd {
    /// Create a new TimerFd
    pub fn new(clock_id: ClockId) -> nix::Result<TimerFd> {
        Self::with_flags(clock_id, Default::default())
    }

    /// Create a new TimerFd with flags
    pub fn with_flags(clock_id: ClockId, flags: TFDFlags) -> nix::Result<TimerFd> {
        Ok(TimerFd(timerfd_create(clock_id as clockid_t, flags)?))
    }

    /// Start or stop a timer
    pub fn set_time(
        &mut self,
        itmr: &ITimerSpec,
        otmr: Option<&mut ITimerSpec>,
    ) -> nix::Result<()> {
        self.set_time_with_flags(Default::default(), itmr, otmr)
    }

    /// Return current timer
    pub fn get_time(&self, otmr: &mut ITimerSpec) -> nix::Result<()> {
        timerfd_gettime(self.0, otmr)
    }

    /// Set a timer with flags
    pub fn set_time_with_flags(
        &mut self,
        flags: TFDTimerFlags,
        itmr: &ITimerSpec,
        otmr: Option<&mut ITimerSpec>,
    ) -> nix::Result<()> {
        timerfd_settime(self.0, flags, itmr, otmr)
    }

    pub fn read_time(&mut self) -> nix::Result<Option<u64>> {
        let mut buf: [u8; TIMERFD_DATA_SIZE] = unsafe { mem::uninitialized() };

        match unistd::read(self.0, &mut buf) {
            Ok(TIMERFD_DATA_SIZE) => Ok(Some(unsafe { mem::transmute(buf) })),
            Ok(_) => unreachable!("partial read of timerfd"),
            Err(Error::Sys(Errno::EAGAIN)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl Iterator for TimerFd {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_time().unwrap_or(None)
    }
}

impl AsRawFd for TimerFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Drop for TimerFd {
    fn drop(&mut self) {
        let _ = unistd::close(self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{time, thread};
    use nix::sys::time::{TimeSpec, TimeValLike};

    #[test]
    fn test_read_timerfd() {
        let mut timer =
            TimerFd::with_flags(ClockId::Monotonic, TFD_NONBLOCK).expect("Fail to create timerfd");
        assert_eq!(timer.read_time(), Ok(None));
        timer
            .set_time(
                &ITimerSpec {
                    it_value: TimeSpec::seconds(3),
                    it_interval: TimeSpec::seconds(0),
                },
                None,
            )
            .expect("Fail to set time");
        assert_eq!(timer.read_time(), Ok(None));
        thread::sleep(time::Duration::from_secs(3));
        assert!(timer.read_time().unwrap().is_some());
    }
}
