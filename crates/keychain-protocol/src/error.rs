#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error("Failed to decode packet because opcode was unrecognized.")]
    UnrecognizedOpcode,
    #[error("Failed to decode packet because the length was formed in a way exceeding any respectable size for a packet.")]
    MalformedLength,
    #[error("Failed to decode because there was a unexpected end of data.")]
    UnexpectedEof
}