use crate::events::*;
use ferrumc_state::GlobalState;
use ferrumc_net::{NetResult, errors::NetError};
use ferrumc_macros::event_handler;
use ferrumc_text::*;
use ferrumc_config::statics::{get_global_config, get_whitelist};

#[event_handler]
async fn handle_login_start(
    event: PlayerStartLoginEvent,
    _state: GlobalState,
) -> NetResult<PlayerStartLoginEvent> {
    if get_global_config().whitelist {
        let whitelist = get_whitelist();

        if whitelist.get(&event.profile.uuid).is_none() {
            Err(NetError::Kick(
                ComponentBuilder::translate("multiplayer.disconnect.not_whitelisted", Vec::new())
            ))
        } else {
            Ok(event)
        }
    } else {
        Ok(event)
    }
}