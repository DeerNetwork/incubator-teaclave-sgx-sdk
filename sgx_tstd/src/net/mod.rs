// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

//! Networking primitives for TCP/UDP communication.
//!
//! This module provides networking functionality for the Transmission Control and User
//! Datagram Protocols, as well as types for IP and socket addresses.
//!
//! # Organization
//!
//! * [`TcpListener`] and [`TcpStream`] provide functionality for communication over TCP
//! * [`UdpSocket`] provides functionality for communication over UDP
//! * [`IpAddr`] represents IP addresses of either IPv4 or IPv6; [`Ipv4Addr`] and
//!   [`Ipv6Addr`] are respectively IPv4 and IPv6 addresses
//! * [`SocketAddr`] represents socket addresses of either IPv4 or IPv6; [`SocketAddrV4`]
//!   and [`SocketAddrV6`] are respectively IPv4 and IPv6 socket addresses
//! * [`ToSocketAddrs`] is a trait that used for generic address resolution when interacting
//!   with networking objects like [`TcpListener`], [`TcpStream`] or [`UdpSocket`]
//! * Other types are return or parameter types for various methods in this module

use crate::io::{self, Error, ErrorKind};

pub use self::addr::{SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
pub use self::ip::{IpAddr, Ipv4Addr, Ipv6Addr, Ipv6MulticastScope};
pub use self::parser::AddrParseError;
#[cfg(feature = "net")]
pub use self::tcp::{Incoming, TcpListener, TcpStream};
#[cfg(feature = "net")]
pub use self::udp::UdpSocket;

mod addr;
mod ip;
mod parser;
#[cfg(feature = "net")]
mod tcp;
#[cfg(feature = "net")]
mod udp;

/// Possible values which can be passed to the [`TcpStream::shutdown`] method.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Shutdown {
    /// The reading portion of the [`TcpStream`] should be shut down.
    ///
    /// All currently blocked and future [reads] will return [`Ok`]`(0)`.
    ///
    /// [reads]: crate::io::Read
    Read,
    /// The writing portion of the [`TcpStream`] should be shut down.
    ///
    /// All currently blocked and future [writes] will return an error.
    ///
    /// [writes]: crate::io::Write
    Write,
    /// Both the reading and the writing portions of the [`TcpStream`] should be shut down.
    ///
    /// See [`Shutdown::Read`] and [`Shutdown::Write`] for more information.
    Both,
}

#[inline]
const fn htons(i: u16) -> u16 {
    i.to_be()
}
#[inline]
const fn ntohs(i: u16) -> u16 {
    u16::from_be(i)
}

fn each_addr<A: ToSocketAddrs, F, T>(addr: A, mut f: F) -> io::Result<T>
where
    F: FnMut(io::Result<&SocketAddr>) -> io::Result<T>,
{
    let addrs = match addr.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(e) => return f(Err(e)),
    };
    let mut last_err = None;
    for addr in addrs {
        match f(Ok(&addr)) {
            Ok(l) => return Ok(l),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        Error::new_const(ErrorKind::InvalidInput, &"could not resolve to any addresses")
    }))
}