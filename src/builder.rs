use crate::Result;
#[cfg(target_os = "linux")]
use crate::linux::params::Params;
#[cfg(target_os = "linux")]
use crate::tun::Tun;
use core::convert::From;
use libc::{IFF_NO_PI, IFF_TAP, IFF_TUN};
use std::net::Ipv4Addr;

/// Represents a factory to build new instances of [`Tun`](struct.Tun.html).
pub struct TunBuilder {
    name: String,
    is_tap: bool,
    packet_info: bool,
    persist: bool,
    up: bool,
    mtu: Option<i32>,
    owner: Option<i32>,
    group: Option<i32>,
    address: Option<Ipv4Addr>,
    destination: Option<Ipv4Addr>,
    broadcast: Option<Ipv4Addr>,
    netmask: Option<Ipv4Addr>,
    queues: Option<usize>,
    cloexec: bool,
}

impl Default for TunBuilder {
    fn default() -> Self {
        Self {
            name: "".into(),
            owner: None,
            group: None,
            is_tap: false,
            persist: false,
            up: false,
            mtu: None,
            packet_info: false,
            address: None,
            destination: None,
            broadcast: None,
            netmask: None,
            queues: None,
            cloexec: true,
        }
    }
}

impl TunBuilder {
    /// Creates a new instance of [`TunBuilder`](struct.TunBuilder.html).
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the name of device (max length: 16 characters), if it is empty, then device name is set by kernel. Default value is empty.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    /// Allocates a TAP device. If not set, a TUN device is created.
    ///
    /// *TAP* devices are layer 2 devices which will result in ethernet frames (or other protocols)
    /// being transmitted over this device.
    ///
    /// In contrast, *TUN* devices are layer 3 devices which means that IP packets are transmitted
    /// over it.
    pub fn tap(mut self) -> Self {
        self.is_tap = true;
        self
    }

    /// Builds multiple instances of [`Tun`](struct.Tun.html) with `IFF_MULTI_QUEUE` flag.
    ///
    /// Internally this creates multiple file descriptors to parallelize packet sending and receiving.
    /// Default value is `1`.
    pub fn queues(mut self, queues: usize) -> Self {
        self.queues = Some(queues);
        self
    }

    /// Set `packet_info` to `true` (default is `false`), thereby unsetting the `IFF_NO_PI` flag on
    /// allocation.
    ///
    /// Instructing the kernel to provide packet information results in it adding 4 extra bytes to
    /// the beginning of the packet (2 flag bytes and 2 protocol bytes). These bytes tell you
    /// what kind of package was just delivered.
    ///
    /// If you don't need this kind of information, you do not need to set this option. In this
    /// case, only raw packets (whatever they may be) are received.
    pub fn packet_info(mut self) -> Self {
        self.packet_info = true;
        self
    }

    /// Sets the MTU (Maximum Transfer Unit) of device.
    ///
    /// MTU defines the maximum size of packets which this device will allow being transmitted or
    /// received.
    ///
    /// It can also be used as an estimate to size input buffers although the exact packet size
    /// may vary to account for some protocol overhead.
    pub fn mtu(mut self, mtu: i32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets the owner of device.
    ///
    /// This is the numeric UID of the user who will own the created device.
    pub fn owner(mut self, owner: i32) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Sets the group of device.
    ///
    /// This is the numeric GID of the group that will own the created device.
    pub fn group(mut self, group: i32) -> Self {
        self.group = Some(group);
        self
    }

    /// Sets IPv4 address of device.
    ///
    /// Sending packets to this address is how they are delivered to your program.
    /// This includes routing via this address.
    pub fn address(mut self, address: Ipv4Addr) -> Self {
        self.address = Some(address);
        self
    }

    /// Sets IPv4 destination address of device.
    ///
    /// Defining a destination address results in the device only being "connected" to that address.
    /// In practice this means that some routing mechanisms (like ARP and NDP lookups) are skipped
    /// because this device can only reach the destination address.
    ///
    /// See the [linux manpage](https://man7.org/linux/man-pages/man7/netdevice.7.html) under section *SIOCGIFDSTADDR, SIOCSIFDSTADDR* for an official description.
    pub fn destination(mut self, dst: Ipv4Addr) -> Self {
        self.destination = Some(dst);
        self
    }

    /// Sets IPv4 broadcast address of device.
    pub fn broadcast(mut self, broadcast: Ipv4Addr) -> Self {
        self.broadcast = Some(broadcast);
        self
    }

    /// Sets IPv4 netmask address of device.
    pub fn netmask(mut self, netmask: Ipv4Addr) -> Self {
        self.netmask = Some(netmask);
        self
    }

    /// Removes CLOEXEC flag on all FDs. This will allow passing tun/tap FDs to any exec-ed
    /// child processes.
    /// Default behaviour is to prevent passing fds flag.
    pub fn no_close_on_exec(mut self) -> Self {
        self.cloexec = false;
        self
    }

    /// Makes the device persistent.
    ///
    /// Persistent devices stay registered as long as the computer is not restarted.
    /// If another device with the same name is created afterwards, it will override the existing
    /// one.
    ///
    /// Non-persistent devices on the other hand, are removed as soon as the controlling process
    /// exits.
    pub fn persist(mut self) -> Self {
        self.persist = true;
        self
    }

    /// Sets up the device.
    ///
    /// This means the interface is immediately put into the *up* state.
    /// It is thus immediately able to service incoming and outgoing packets.
    pub fn up(mut self) -> Self {
        self.up = true;
        self
    }

    /// Builds a new instance of [`Tun`](struct.Tun.html).
    pub fn build(self) -> Result<Vec<Tun>> {
        match self.queues {
            Some(queues) if queues > 1 => Tun::new_mq(self.into(), queues),
            _ => Tun::new(self.into()).map(|tun| vec![tun]),
        }
    }
}

impl From<TunBuilder> for Params {
    #[cfg(target_os = "linux")]
    fn from(builder: TunBuilder) -> Self {
        Params {
            name: if builder.name.is_empty() {
                None
            } else {
                Some(builder.name)
            },
            flags: {
                let mut flags = if builder.is_tap { IFF_TAP } else { IFF_TUN } as _;
                if !builder.packet_info {
                    flags |= IFF_NO_PI as i16;
                }
                flags
            },
            persist: builder.persist,
            up: builder.up,
            mtu: builder.mtu,
            owner: builder.owner,
            group: builder.group,
            address: builder.address,
            destination: builder.destination,
            broadcast: builder.broadcast,
            netmask: builder.netmask,
            cloexec: builder.cloexec,
        }
    }

    #[cfg(not(any(target_os = "linux")))]
    fn from(builder: TunBuilder) -> Self {
        unimplemented!()
    }
}
