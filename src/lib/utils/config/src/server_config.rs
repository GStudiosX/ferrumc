//! # The server configuration module.
//!
//! Contains the server configuration struct and its related functions.

use serde_derive::{Deserialize, Serialize};



/// The server configuration struct.
///
/// Fields:
/// - `host`: The IP/host that the server will bind to.
/// - `port`: The port that the server will bind to. (0-65535)
/// - `motd`: The message of the day that is displayed to clients. It will randomly select one from the list.
/// - `max_players`: The maximum number of players that can be connected to the server.
/// - `network_tick_rate`: How many network updates to process per second per user.
/// - `database` - [DatabaseConfig]: The configuration for the database.
/// - `world`: The name of the world that the server will load.
/// - `network_compression_threshold`: The threshold at which the server will compress network packets.
/// - `lan`: Open to LAN settings.
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16, // 0-65535
    pub motd: Vec<String>,
    pub max_players: u32,
    pub network_tick_rate: u32,
    pub database: DatabaseConfig,
    pub world: String,
    pub network_compression_threshold: i32, // Can be negative
    #[serde(default)]
    pub velocity: VelocityConfig,
    #[serde(default)]
    pub lan: LanConfig,
}

/// The velocity configuration struct.
///
/// Fields:
/// - `enabled`: If velocity support should be enabled.
/// - `secret`: The velocity secret used for modern forwarding.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct VelocityConfig {
    /// see [velocity_secret](VelocityConfig::secret)
    pub enabled: bool,
    pub secret: String,
}

/// The LAN configuration struct.
///
/// Fields:
/// - `enabled`: If LAN should be enabled.
/// - `ping_interval`: The interval to ping in seconds.
#[derive(Debug, Deserialize, Serialize)]
pub struct LanConfig {
    pub enabled: bool,
    pub ping_interval: f32,
}

impl Default for LanConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ping_interval: 1.5f32,
        }
    }
}

/// The database configuration section from [ServerConfig].
///
/// Fields:
/// - `cache_size`: The cache size in KB.
/// - `compression` - [DatabaseCompression]: The compression algorithm to use.
#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub cache_size: u32,
    pub compression: DatabaseCompression,
}

/// The database compression enum for [DatabaseConfig].
///
/// Variants:
/// - `none`: No compression.
/// - `fast`: Fast compression.
/// - `best`: Best compression.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum DatabaseCompression {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "fast")]
    Fast,
    #[serde(rename = "best")]
    Best,
}
