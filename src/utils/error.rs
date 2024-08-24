use std::convert::Infallible;

use config::ConfigError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    TomlSe(#[from] toml::ser::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(u32),
    #[error("Invalid packet id: {0}")]
    InvalidPacketId(u32),
    #[error("Invalid state: {0:x}")]
    InvalidState(i32),
    #[error("Invalid Connection Metadata: {0}")]
    InvalidConnectionMetadata(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Invalid component storage: {0}")]
    InvalidComponentStorage(String),
    #[error("Component {0} not found for entity {1}")]
    ComponentNotFound(String, u64),

    #[error(transparent)]
    ECSError(#[from] crate::ecs::error::Error),

    #[error(transparent)]
    FastAnvilError(#[from] fastanvil::Error),
    #[error("Chunk at ({0}, {1}) not found")]
    ChunkNotFound(i32, i32),

    #[error(transparent)]
    SimdNbtError(#[from] simdnbt::Error),
    #[error("Invalid NBT: {0}")]
    InvalidNbt(String),
    #[error(transparent)]
    NbtDeserializeError(#[from] simdnbt::DeserializeError),
    #[error(transparent)]
    NBTError(#[from] nbt_lib::NBTError),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid directive: {0}")]
    InvalidDirective(String),

    #[error("TCP Error: {0}")]
    TcpError(String),

    #[error("Invalid NBT: {0}")]
    GenericNbtError(String),

    #[error("Serialization failed: {0}")]
    SerializationError(String),
    #[error("Deserialization failed: {0}")]
    DeserializationError(String),

    #[error("Attempted to output more bits that will fit in output type: {0} attempted, {1} max")]
    BitOutputOverflow(usize, usize),
    #[error("Attempted to read more bits than are available: {0} attempted, {1} available")]
    BitReadOverflow(usize, usize),

    #[error("Codec error")]
    CodecError(#[from] ferrumc_codec::error::CodecError),
    #[error("Conversion error")]
    ConversionError,
}

impl From<Infallible> for Error {
    fn from(e: Infallible) -> Self {
        return Error::Generic(format!("{:?}", e));
    }
}
