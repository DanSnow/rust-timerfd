use libc::{c_int, clockid_t};
use itimer_spec::ITimerSpec;

extern "C" {
    pub fn timerfd_create(clock_id: clockid_t, flags: c_int) -> c_int;
    pub fn timerfd_settime(
        ufd: c_int,
        flags: c_int,
        utmr: *const ITimerSpec,
        otmr: *mut ITimerSpec,
    ) -> c_int;
    pub fn timerfd_gettime(ufd: c_int, otmr: *mut ITimerSpec) -> c_int;
}
