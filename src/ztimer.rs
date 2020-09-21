//! # [ztimer high level timer](https://riot-os.org/api/group__sys__ztimer.html)

use core::convert::TryInto;

use riot_sys::{ztimer_clock_t};

/// A ZTimer that knows about its frequency. The pulse length is not given in core::time::Duration
/// as that's not even supported by non-`min_` `const_generics`. This is likely to change, even
/// though it breaks the API.
pub struct ZTimer<const HZ: u32>(*mut ztimer_clock_t);

impl<const HZ: u32> ZTimer<HZ> {
    /// Pause the current thread for the duration of ticks in the timer's time scale.
    ///
    /// Wraps [ztimer_sleep](https://riot-os.org/api/group__sys__ztimer.html#gade98636e198f2d571c8acd861d29d360)
    pub fn sleep_ticks(&self, duration: u32) {
        unsafe { riot_sys::ztimer_sleep(self.0, duration) };
    }

    /// Keep the current thread in a busy loop until the duration of ticks in the timer's tim scale
    /// has passed
    ///
    /// Quoting the original documentation, "This blocks lower priority threads. Use only for
    /// *very* short delays.".
    ///
    /// Wraps [ztimer_spin](https://riot-os.org/api/group__sys__ztimer.html#ga9de3d9e3290746b856bb23eb2dccaa7c)
    pub fn spin_ticks(&self, duration: u32) {
        unsafe { riot_sys::ztimer_spin(self.0 as _ /* INLINE CAST */, duration) };
    }

    /// Pause the current thread for the given duration.
    ///
    /// The duration is converted into ticks (rounding up), and overflows are caught by sleeping
    /// multiple times.
    ///
    /// It is up to the caller to select the ZTimer suitable for efficiency. (Even sleeping for
    /// seconds on the microseconds timer would not overflow the timer's interface's u32, but the
    /// same multiple-sleeps trick may need to be employed by the implementation, *and* would keep
    /// the system from entering deeper sleep modes).
    pub fn sleep(&self, duration: core::time::Duration) {
        // Convert to ticks, rounding up as per Duration documentation
        let mut ticks = (duration * HZ - core::time::Duration::new(0, 1)).as_secs() + 1;
        while ticks > u32::MAX.into() {
            self.sleep_ticks(u32::MAX);
            ticks -= u64::from(u32::MAX);
        }
        self.sleep_ticks(ticks.try_into().expect("Was just checked manually above"));
    }
}

impl ZTimer<1000> {
    /// Get the global milliseconds ZTimer, ZTIMER_MSEC.
    ///
    /// This function is only available if the ztimer_msec module is built.
    #[cfg(riot_module_ztimer_msec)]
    pub fn msec() -> Self {
        ZTimer(unsafe { riot_sys::ZTIMER_MSEC })
    }
}

impl ZTimer<1000000> {
    /// Get the global microseconds ZTimer, ZTIMER_USEC.
    ///
    /// This function is only available if the ztimer_usec module is built.
    #[cfg(riot_module_ztimer_usec)]
    pub fn usec() -> Self {
        ZTimer(unsafe { riot_sys::ZTIMER_USEC })
    }
}

impl embedded_hal::blocking::delay::DelayMs<u32> for ZTimer<1000> {
    fn delay_ms(&mut self, ms: u32) {
        self.sleep_ticks(ms.into());
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for ZTimer<1000000> {
    fn delay_us(&mut self, us: u32) {
        self.sleep_ticks(us);
    }
}