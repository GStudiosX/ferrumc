use crate::experiments::*;
use std::sync::Arc;
use std::io::Write;
use ferrumc::{
    events::{PlayerAsyncChatEvent, GlobalState, event_handler},
    text::*, PlayerIdentity, NetEncodeOpts, StreamWriter,
    EntityExt, NetResult
};

#[event_handler]
async fn test_chat_handler(
    event: PlayerAsyncChatEvent,
    state: GlobalState,
) -> NetResult<PlayerAsyncChatEvent> {
    let entity = event.entity;
    let mut writer = entity
        .get_mut::<StreamWriter>(Arc::clone(&state))?;
    let profile = entity
        .get::<PlayerIdentity>(Arc::clone(&state))?;
    writer.send_packet(&SystemChatMessage::message(
        ComponentBuilder::translate("chat.type.text", vec![
            ComponentBuilder::text(&profile.username)
                .color(NamedColor::Blue)
                .click_event(ClickEvent::SuggestCommand(format!("/msg {}", profile.username)))
                .hover_event(HoverEvent::ShowEntity {
                    entity_type: "minecraft:player".to_string(),
                    id: uuid::Uuid::from_u128(profile.uuid),
                    name: Some(profile.username.clone())
                })
                .build(),
            ComponentBuilder::text(&event.message.message).build(),
        ])
    ), &NetEncodeOpts::WithLength).await?;
    Ok(event)
}

