mod admin_message;
mod circuit_error;
mod circuit_message;
mod direct_message;
mod service_handlers;

use protobuf::Message;

use crate::protos::circuit::{CircuitMessage, CircuitMessageType};
use crate::protos::network::{NetworkMessage, NetworkMessageType};

pub use self::admin_message::AdminDirectMessageHandler;
pub use self::circuit_error::CircuitErrorHandler;
pub use self::circuit_message::CircuitMessageHandler;
pub use self::direct_message::CircuitDirectMessageHandler;
pub use self::service_handlers::ServiceConnectForwardHandler;
pub use self::service_handlers::ServiceConnectRequestHandler;
pub use self::service_handlers::ServiceDisconnectForwardHandler;
pub use self::service_handlers::ServiceDisconnectRequestHandler;

fn create_message(
    payload: Vec<u8>,
    circuit_message_type: CircuitMessageType,
) -> Result<Vec<u8>, protobuf::error::ProtobufError> {
    let mut circuit_msg = CircuitMessage::new();
    circuit_msg.set_message_type(circuit_message_type);
    circuit_msg.set_payload(payload);
    let circuit_bytes = circuit_msg.write_to_bytes()?;

    let mut network_msg = NetworkMessage::new();
    network_msg.set_message_type(NetworkMessageType::CIRCUIT);
    network_msg.set_payload(circuit_bytes);
    network_msg.write_to_bytes()
}
