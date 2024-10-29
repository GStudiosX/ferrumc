use std::sync::Arc;
use std::io::Write;
use ferrumc::{
    macros::{NetEncode, packet},
    events::{PlayerAsyncChatEvent, GlobalState, event_handler},
    text::*, PlayerIdentity, NetEncodeOpts, StreamWriter,
    EntityExt, NetResult
};

#[derive(NetEncode)]
#[packet(packet_id = 0x6C)]
struct SystemChatMessage {
    message: TextComponent,
    overlay: bool,
}

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
    writer.send_packet(&SystemChatMessage {
        message: ComponentBuilder::translate("chat.type.text", vec![
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
        ]),
        overlay: false,
    }, &NetEncodeOpts::WithLength).await?;
    Ok(event)
}

