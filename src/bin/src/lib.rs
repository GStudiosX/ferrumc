// Security
#![forbid(unsafe_code)]

use std::sync::Arc;
use lazy_static::lazy_static;

use ferrumc_scheduler::Scheduler;

pub use ferrumc_config::statics::get_global_config;

pub use ferrumc_net_codec::{
    encode::{NetEncodeOpts, NetEncode},
    decode::{NetDecodeOpts, NetDecode}
};
pub use ferrumc_net::{
    NetResult, ServerState,
    connection::{
        StreamWriter, StreamReader, ConnectionState
    }, 
    utils::ecs_helpers::EntityExt,
    packets::outgoing::*
};

pub use ferrumc_core::identity::player_identity::PlayerIdentity;
pub use ferrumc_net::connection::{Profile, GameProfile};
pub use ferrumc_net_codec::*;

pub mod macros {
    pub use ferrumc_macros::{NetEncode, NetDecode, packet};
}

pub mod text {
    pub use ferrumc_text::*;
}

/// Event API
pub mod events;

/// INTERNAL
pub mod internal {
    use super::*;
    use ferrumc_net::packets::outgoing::login_success::LoginSuccessPacket;

    pub async fn send_login_success(conn_id: usize, game_profile: GameProfile, state: Arc<ServerState>) -> NetResult<()> {
        let mut profile = state
           .universe
           .get_mut::<Profile>(conn_id)?;

        let mut writer = state
            .universe
            .get_mut::<StreamWriter>(conn_id)?;

        let response = LoginSuccessPacket::new(game_profile.clone());
        writer.send_packet(&response, &NetEncodeOpts::WithLength).await?;

        // Add the player identity component to the ECS for the entity.
        state.universe.add_component::<PlayerIdentity>(
            conn_id,
            PlayerIdentity::new(game_profile.username.clone(), game_profile.uuid),
        )?;

        profile.profile = Some(game_profile);

        Ok(())
    }
}

lazy_static! {
    static ref SCHEDULER: Arc<Scheduler> = Arc::new(Scheduler::default());
}

pub fn get_scheduler() -> Arc<Scheduler> {
    SCHEDULER.clone()
}

/*use std::sync::OnceLock;
pub fn get_scheduler() -> &'static Scheduler {
    static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();
    SCHEDULER.get_or_init(|| Scheduler::new())
}*/
