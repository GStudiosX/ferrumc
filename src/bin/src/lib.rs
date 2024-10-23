// Security
#![forbid(unsafe_code)]

use std::sync::Arc;
use ferrumc_scheduler::Scheduler;
use lazy_static::lazy_static;

pub use ferrumc_net_codec::encode::{NetEncodeOpts, NetEncode};
pub use ferrumc_net_codec::decode::{NetDecodeOpts, NetDecode};
pub use ferrumc_net::utils::ecs_helpers::EntityExt;
pub use ferrumc_net::{ServerState, connection::{StreamWriter, StreamReader, GameProfile, ConnectionState, Profile}};
pub use ferrumc_net::NetResult;

pub use ferrumc_core::identity::player_identity::PlayerIdentity;

pub use ferrumc_net_codec::*;

/// Event API
pub mod events;

/// INTERNAL do not use unless necessary
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

/*lazy_static! {
    static ref SCHEDULER: Arc<Scheduler> = Arc::new(Scheduler::new());
}

pub fn get_scheduler() -> Arc<Scheduler> {
    SCHEDULER.clone()
}*/

use std::sync::OnceLock;
pub fn get_scheduler() -> &'static Scheduler {
    static SCHEDULER: OnceLock<Arc<Scheduler>> = OnceLock::new();
    SCHEDULER.get_or_init(|| Arc::new(Scheduler::new())).clone()
}
