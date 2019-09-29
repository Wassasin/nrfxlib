//! # FFI (Foreign Function Interface) Module
//!
//! This module contains implementations of functions that libbsd.a expects to
//! be able to call.
//!
//! Copyright (c) 42 Technology, 2019
//!
//! Dual-licensed under MIT and Apache 2.0. See the [README](../README.md) for
//! more details.

use nrf91;

/// Stores the last error from the library. See `bsd_os_errno_set` and
/// `get_last_error`.
static LAST_ERROR: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);

extern "C" {
	// This function is in the C library but not in the headers generated by
	// nrfxlib-sys.
	pub fn IPC_IRQHandler();
}

/// Function required by BSD library. We need to set the EGU1 interrupt.
#[no_mangle]
pub extern "C" fn bsd_os_application_irq_set() {
	cortex_m::peripheral::NVIC::pend(nrf91::Interrupt::EGU1);
}

/// Function required by BSD library. We need to clear the EGU1 interrupt.
#[no_mangle]
pub extern "C" fn bsd_os_application_irq_clear() {
	cortex_m::peripheral::NVIC::unpend(nrf91::Interrupt::EGU1);
}

/// Function required by BSD library. We need to set the EGU2 interrupt.
#[no_mangle]
pub extern "C" fn bsd_os_trace_irq_set() {
	cortex_m::peripheral::NVIC::pend(nrf91::Interrupt::EGU2);
}

/// Function required by BSD library. We need to clear the EGU2 interrupt.
#[no_mangle]
pub extern "C" fn bsd_os_trace_irq_clear() {
	cortex_m::peripheral::NVIC::unpend(nrf91::Interrupt::EGU2);
}

/// Function required by BSD library. We have no init to do.
#[no_mangle]
pub extern "C" fn bsd_os_init() {
	// Nothing
}

/// Function required by BSD library. Stores an error code we can read later.
#[no_mangle]
pub extern "C" fn bsd_os_errno_set(errno: i32) {
	LAST_ERROR.store(errno, core::sync::atomic::Ordering::SeqCst);
}

/// Return the last error stored by the nrfxlib C library.
pub fn get_last_error() -> i32 {
	LAST_ERROR.load(core::sync::atomic::Ordering::SeqCst)
}

/// Function required by BSD library
#[no_mangle]
pub extern "C" fn bsd_os_timedwait(_context: u32, p_timeout_ms: *const i32) -> i32 {
	let timeout_ms = unsafe { *p_timeout_ms };
	if timeout_ms < 0 {
		// With Zephyr, negative timeouts pend on a semaphore with K_FOREVER.
		// We can't do that here.
		0i32
	} else {
		// NRF9160 runs at 64 MHz, so this is close enough
		cortex_m::asm::delay((timeout_ms as u32) * 64_000);
		nrfxlib_sys::NRF_ETIMEDOUT as i32
	}
}

/// Function required by BSD library
#[no_mangle]
pub extern "C" fn bsd_os_trace_put(_data: *const u8, _len: u32) -> i32 {
	// Do nothing
	0
}

/// Function required by BSD library
#[no_mangle]
pub extern "C" fn bsd_irrecoverable_error_handler(err: u32) -> ! {
	panic!("bsd_irrecoverable_error_handler({})", err);
}
