use crate::experiments::*;
use ferrumc::{
    text::*, get_scheduler, 
    events::{PlayerJoinGameEvent, event_handler, GlobalState},
    NetResult, EntityExt, NetEncodeOpts, StreamWriter
};
use std::sync::Arc;
use std::io::Write;

#[event_handler]
async fn test_join_handler(
    event: PlayerJoinGameEvent,
    state: GlobalState,
) -> NetResult<PlayerJoinGameEvent> {
    let entity = event.entity;

    get_scheduler().schedule_tick(move |state| async move {
        let mut writer = entity
            .get_mut::<StreamWriter>(Arc::clone(&state))?;
        writer.send_packet(&SystemChatMessage::message(format!("10 ticks have passed", ms)), &NetEncodeOpts::WithLength).await?;
        Ok(())
    }, 10).await.unwrap();

    Ok(event)
}
