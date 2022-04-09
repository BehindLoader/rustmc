#[derive(Debug)]
pub(super) enum RsCraftError {
    TcpError(String),
    PacketParsingError(String),
}
