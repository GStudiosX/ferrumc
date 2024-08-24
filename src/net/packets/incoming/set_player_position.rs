use tracing::{debug, trace};

use ferrumc_macros::{packet, Decode};

use crate::net::packets::{ConnectionId, IncomingPacket};
use crate::state::GlobalState;
use crate::utils::encoding::position::Position;
use crate::Connection;

/// The set player position packet is sent by the client to the server to update the player's position.
#[derive(Decode)]
#[packet(packet_id = 0x14, state = "play")]
pub struct SetPlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

impl IncomingPacket for SetPlayerPosition {
    async fn handle(
        self,
        conn_id: ConnectionId,
        state: GlobalState,
    ) -> crate::utils::prelude::Result<()> {
        trace!("SetPlayerPosition packet received");
        trace!("X: {}", self.x);
        trace!("Y: {}", self.y);
        trace!("Z: {}", self.z);

        let my_entity_id = conn_id;

        let component_storage = state.world.get_component_storage();

        let mut position = component_storage
            .get_mut::<Position>(my_entity_id)
            .await
            .ok_or(Error::from(crate::ecs::error::Error::ComponentNotFound))?;

        *position = Position {
            x: self.x as i32,
            y: self.y as i16,
            z: self.z as i32,
        };

        Ok(())
    }
}
