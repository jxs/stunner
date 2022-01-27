use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use stun_coder::{StunAttribute, StunMessage, StunMessageClass, StunMessageMethod};
use tokio::net::{ToSocketAddrs, UdpSocket};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Specify the listening port where the server should run,
    /// by default 19302 is used
    #[clap(long, default_value = "3478")]
    port: u16,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let opt = Cli::parse();
    serve(("0", opt.port))
        .await
        .expect("could not start server")
}

/// Listen for STUN requests on the given address and reply to valid STUN Binding Requests
async fn serve(addr: impl ToSocketAddrs) -> Result<()> {
    let sock = UdpSocket::bind(addr).await?;
    log::info!("serving on addr: {}", sock.local_addr().unwrap());

    loop {
        let mut buf = [0; 1024];
        let (_, src_addr) = sock.recv_from(&mut buf).await?;
        // Process the response in case of a STUN binding request
        if let Some(message) = parse_message(&buf, src_addr) {
            log::trace!("replied {:?} to {:?}", message, src_addr);
            if let Err(err) = sock.send_to(&message.encode(None).unwrap(), src_addr).await {
                log::error!(
                    "could not send response {:?} to address {:?}, reason: {}",
                    message,
                    src_addr,
                    err
                );
            }
        }
    }
}

/// Parse the stun request and create the appropriate response message.
fn parse_message(buf: &[u8], src_addr: SocketAddr) -> Option<StunMessage> {
    let message = match StunMessage::decode(buf, None) {
        Ok(message) => message,
        Err(err) => {
            log::debug!(
                "could not parse packet from {:?} : {:?} as a STUN message",
                src_addr,
                err
            );
            return None;
        }
    };
    let header = message.get_header();
    match (header.message_method, header.message_class) {
        (StunMessageMethod::BindingRequest, StunMessageClass::Request) => {
            log::debug!(
                "STUN binding request received {:?} from source address: {:?}",
                message,
                src_addr
            );
            let response = StunMessage::new(
                StunMessageMethod::BindingRequest,
                StunMessageClass::SuccessResponse,
            )
            .set_transaction_id(header.transaction_id)
            .add_attribute(StunAttribute::XorMappedAddress {
                socket_addr: src_addr,
            });
            Some(response)
        }
        (StunMessageMethod::BindingRequest, StunMessageClass::Indication) => {
            log::debug!(
                "STUN indication received {:?} from source address: {:?}",
                message,
                src_addr
            );
            // No response is generated for an indication https://datatracker.ietf.org/doc/html/rfc5389#section-7.3.2
            None
        }
        (StunMessageMethod::BindingRequest, class @ StunMessageClass::ErrorResponse)
        | (StunMessageMethod::BindingRequest, class @ StunMessageClass::SuccessResponse) => {
            log::debug!("STUN binding {:?}", class);
            // Reply with BAD REQUEST see https://datatracker.ietf.org/doc/html/rfc5389#section-15.6
            let response = StunMessage::new(
                StunMessageMethod::BindingRequest,
                StunMessageClass::ErrorResponse,
            )
            .add_attribute(StunAttribute::ErrorCode {
                class: 4,
                number: 0,
                reason: "Invalid binding request class".into(),
            });
            Some(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use stun_coder::{StunAttribute, StunMessage, StunMessageClass, StunMessageMethod};

    use super::parse_message;

    #[test]
    fn server_responds_successful_to_binding_request() {
        let req_msg =
            StunMessage::new(StunMessageMethod::BindingRequest, StunMessageClass::Request);
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let response = parse_message(&req_msg.encode(None).unwrap(), socket).unwrap();
        let header = response.get_header();
        let attributes = response.get_attributes();
        assert!(matches!(
            header.message_method,
            StunMessageMethod::BindingRequest
        ));
        assert!(matches!(
            header.message_class,
            StunMessageClass::SuccessResponse
        ));
        assert_eq!(attributes.len(), 1);
        assert!(
            matches!(attributes[0], StunAttribute::XorMappedAddress { socket_addr} if socket_addr == socket)
        );
    }

    #[test]
    fn server_doesnt_respond_to_indication_request() {
        let req_msg = StunMessage::new(
            StunMessageMethod::BindingRequest,
            StunMessageClass::Indication,
        );
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let response = parse_message(&req_msg.encode(None).unwrap(), socket);
        assert!(response.is_none());
    }

    #[test]
    fn server_responds_with_error_to_success_response() {
        let req_msg = StunMessage::new(
            StunMessageMethod::BindingRequest,
            StunMessageClass::SuccessResponse,
        );
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let response = parse_message(&req_msg.encode(None).unwrap(), socket).unwrap();
        let header = response.get_header();
        let attributes = response.get_attributes();
        assert!(matches!(
            header.message_method,
            StunMessageMethod::BindingRequest
        ));
        assert!(matches!(
            header.message_class,
            StunMessageClass::ErrorResponse
        ));
        assert_eq!(attributes.len(), 1);
        assert!(
            matches!(&attributes[0], StunAttribute::ErrorCode { class, number, reason } if class == &4u8 && number == &0u8 && reason == "Invalid binding request class")
        );
    }

    #[test]
    fn server_responds_with_error_to_error_response() {
        let req_msg = StunMessage::new(
            StunMessageMethod::BindingRequest,
            StunMessageClass::ErrorResponse,
        );
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let response = parse_message(&req_msg.encode(None).unwrap(), socket).unwrap();
        let header = response.get_header();
        let attributes = response.get_attributes();
        assert!(matches!(
            header.message_method,
            StunMessageMethod::BindingRequest
        ));
        assert!(matches!(
            header.message_class,
            StunMessageClass::ErrorResponse
        ));
        assert_eq!(attributes.len(), 1);
        assert!(
            matches!(&attributes[0], StunAttribute::ErrorCode { class, number, reason } if class == &4u8 && number == &0u8 && reason == "Invalid binding request class")
        );
    }
}
