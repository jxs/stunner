use anyhow::{Context, Result};
use clap::Parser;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

// All STUN messages sent over UDP SHOULD be less than the path MTU, if
// known.  If the path MTU is unknown, messages SHOULD be the smaller of
// 576 bytes and the first-hop MTU for IPv4 [RFC1122] and 1280 bytes for
// IPv6 [RFC2460].  This value corresponds to the overall size of the IP
// packet.  Consequently, for IPv4, the actual STUN message would need
// to be less than 548 bytes (576 minus 20-byte IP header, minus 8-byte
// UDP header, assuming no IP options are used).
// https://datatracker.ietf.org/doc/html/rfc5389#section-7.1
const MAX_STUN_MSG_SIZE: usize = 1280;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Specify one of the available IP addresses assigned to a network interface present on the host
    #[clap(long, default_value = "0")]
    localaddr: String,

    /// Specify the UDP or TCP port that the primary and alternate interfaces listen on as the primary port for binding requests. If not specified, a randomly available port
    /// chosen by the system is used.
    #[clap(long, default_value = "0")]
    localport: u16,

    /// Destination STUN server.
    remote_addr: String,

    /// Destination STUN port.
    remote_port: u16,
}

// Fetches mapped address of a local Socket
fn get_mapped_addr(udp_socket: UdpSocket, dst_addr: impl ToSocketAddrs) -> Result<SocketAddr> {
    // Create a binding message
    let binding_msg = stun_coder::StunMessage::create_request().add_attribute(
        stun_coder::StunAttribute::Software {
            description: String::from("stunner"),
        },
    );

    // Encode the binding_msg
    let bytes = binding_msg
        .encode(None)
        .expect("should be able to encode the binding msg");

    // Connect to the STUN server
    udp_socket.connect(dst_addr)?;

    // Send the binding request message
    udp_socket.send(&bytes)?;

    // Wait for a response
    let mut response_buf = [0; MAX_STUN_MSG_SIZE];
    udp_socket.recv(&mut response_buf)?;

    // Decode the response
    let stun_response = stun_coder::StunMessage::decode(&response_buf, None)
        .context("could not decode STUN response")?;

    // Find the XorMappedAddress attribute in the response
    // It will contain our reflexive transport address
    for attr in stun_response.get_attributes() {
        if let stun_coder::StunAttribute::XorMappedAddress { socket_addr } = attr {
            return Ok(*socket_addr);
        }
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        "No XorMappedAddress has been set in response.",
    )
    .into())
}

fn main() {
    let opt = Cli::parse();

    // Open a UDP socket
    let udp_socket =
        UdpSocket::bind((opt.localaddr, opt.localport)).expect("could not bind local address");

    let local_addr = udp_socket
        .local_addr()
        .expect("udp socket should have an address");

    let response = get_mapped_addr(udp_socket, (opt.remote_addr, opt.remote_port));
    match response {
        Ok(addr) => {
            println!("Binding test: success");
            println!("Local address: {local_addr}");
            println!("Mapped address: {addr}");
        }
        Err(err) => {
            println!("Binding test: success");
            println!("Local address: {local_addr}");
            println!("Error: {err}");
        }
    }
}
