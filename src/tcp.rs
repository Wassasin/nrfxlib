//! # TCP Sockets for nrfxlib
//!
//! TCP socket related code.
//!
//! Copyright (c) 42 Technology Ltd 2019
//!
//! Dual-licensed under MIT and Apache 2.0. See the [README](../README.md) for
//! more details.

//******************************************************************************
// Sub-Modules
//******************************************************************************

// None

//******************************************************************************
// Imports
//******************************************************************************

use super::{get_last_error, Error};
use crate::raw::*;
use log::debug;
use nrfxlib_sys as sys;

//******************************************************************************
// Types
//******************************************************************************

/// Represents a connection to a remote TCP/IP device using plain TCP
#[derive(Debug)]
pub struct TcpSocket {
	socket: Socket,
}

//******************************************************************************
// Constants
//******************************************************************************

// None

//******************************************************************************
// Global Variables
//******************************************************************************

// None

//******************************************************************************
// Macros
//******************************************************************************

// None

//******************************************************************************
// Public Functions and Impl on Public Types
//******************************************************************************

// None

//******************************************************************************
// Private Functions and Impl on Private Types
//******************************************************************************

impl TcpSocket {
	/// Create a new TCP socket.
	pub fn new() -> Result<TcpSocket, Error> {
		let socket = Socket::new(SocketDomain::Inet, SocketType::Stream, SocketProtocol::Tcp)?;

		// Now configure this socket

		Ok(TcpSocket { socket })
	}

	/// Look up the hostname and for each result returned, try to connect to
	/// it.
	pub fn connect(&self, hostname: &str, port: u16, timeout: u32) -> Result<(), Error> {
		use core::fmt::Write;

		debug!("Connecting via TCP to {}:{}", hostname, port);

		// Now, make a null-terminated hostname
		let mut hostname_smallstring: heapless::String<heapless::consts::U64> =
			heapless::String::new();
		write!(hostname_smallstring, "{}\0", hostname).map_err(|_| Error::HostnameTooLong)?;
		// Now call getaddrinfo with some hints
		let hints = sys::nrf_addrinfo {
			ai_flags: 0,
			ai_family: sys::NRF_AF_INET as i32,
			ai_socktype: sys::NRF_SOCK_STREAM as i32,
			ai_protocol: 0,
			ai_addrlen: 0,
			ai_addr: core::ptr::null_mut(),
			ai_canonname: core::ptr::null_mut(),
			ai_next: core::ptr::null_mut(),
		};
		let mut output_ptr: *mut sys::nrf_addrinfo = core::ptr::null_mut();
		let mut result = unsafe {
			sys::nrf_getaddrinfo(
				// hostname
				hostname_smallstring.as_ptr(),
				// service
				core::ptr::null(),
				// hints
				&hints,
				// output pointer
				&mut output_ptr,
			)
		};
		if (result == 0) && (!output_ptr.is_null()) {
			let mut record: &sys::nrf_addrinfo = unsafe { &*output_ptr };
			loop {
				let dns_addr: &sys::nrf_sockaddr_in =
					unsafe { &*(record.ai_addr as *const sys::nrf_sockaddr_in) };
				// Create a new sockaddr_in with the right port
				let connect_addr = sys::nrf_sockaddr_in {
					sin_len: core::mem::size_of::<sys::nrf_sockaddr_in>() as u8,
					sin_family: sys::NRF_AF_INET as i32,
					sin_port: htons(port),
					sin_addr: dns_addr.sin_addr.clone(),
				};

				debug!("Trying IP address {}", &crate::NrfSockAddrIn(connect_addr));

				let flags = unsafe { sys::nrf_fcntl(self.socket.fd, sys::NRF_F_GETFL as i32, 0) };

				unsafe {
					sys::nrf_fcntl(
						self.socket.fd,
						sys::NRF_F_SETFL as i32,
						flags | sys::NRF_O_NONBLOCK as i32,
					);
				}

				// try and connect to this result
				result = unsafe {
					sys::nrf_connect(
						self.socket.fd,
						&connect_addr as *const sys::nrf_sockaddr_in as *const _,
						connect_addr.sin_len as u32,
					)
				};

				let mut fdset: sys::nrf_fd_set = 1 << (self.socket.fd);
				let select_result = unsafe {
					sys::nrf_select(
						self.socket.fd + 1,
						core::ptr::null::<sys::nrf_fd_set>() as *mut sys::nrf_fd_set,
						&mut fdset,
						core::ptr::null::<sys::nrf_fd_set>() as *mut sys::nrf_fd_set,
						&sys::nrf_timeval {
							tv_sec: timeout,
							tv_usec: 0,
						},
					)
				};

				if select_result == 1 {
					let mut value = 0u16;
					let mut length: u32 = core::mem::size_of_val(&value) as u32;
					unsafe {
						sys::nrf_getsockopt(
							self.socket.fd,
							sys::NRF_SOL_SOCKET as i32,
							sys::NRF_SO_ERROR as i32,
							&mut value as *mut u16 as *mut sys::ctypes::c_void,
							&mut length as *mut u32,
						)
					};

					result = value as i32;
				} else if select_result == 0 {
					// Timeout
					break;
				}

				// Reset the descriptor flags to the original
				unsafe { sys::nrf_fcntl(self.socket.fd, sys::NRF_F_SETFL as i32, flags) };

				if result == 0 {
					break;
				}
				if !record.ai_next.is_null() {
					record = unsafe { &*record.ai_next };
				} else {
					break;
				}
			}
			unsafe {
				sys::nrf_freeaddrinfo(output_ptr);
			}
		}
		if result != 0 {
			Err(Error::Nordic("tcp_connect", result, get_last_error()))
		} else {
			Ok(())
		}
	}
}

impl Pollable for TcpSocket {
	/// Get the underlying socket ID for this socket.
	fn get_fd(&self) -> i32 {
		self.socket.fd
	}
}

impl core::ops::DerefMut for TcpSocket {
	fn deref_mut(&mut self) -> &mut Socket {
		&mut self.socket
	}
}

impl core::ops::Deref for TcpSocket {
	type Target = Socket;
	fn deref(&self) -> &Socket {
		&self.socket
	}
}

//******************************************************************************
// End of File
//******************************************************************************
