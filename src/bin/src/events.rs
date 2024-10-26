pub use ferrumc_net::errors::NetError;
pub use ferrumc_net::GlobalState;

pub use ferrumc_ecs::entities::Entity;

pub use ferrumc_macros::{event_handler, Event};
pub use ferrumc_events::{
    infrastructure::Event,
    errors::EventsError
};

pub use ferrumc_net::connection::PlayerDisconnectEvent;

pub use ferrumc_net::packets::incoming::server_bound_plugin_message::LoginPluginResponseEvent;

use std::sync::{Arc, RwLock};

/// A event that you can read or write and access it after a event has triggered.
///
/// ```ignore
/// let event = RwEvent::new(TestEvent {
///     // data 
/// });
///
/// let event = RwEvent::<TestEvent>::trigger(event.clone(), Arc::clone(&state)).await?;
/// let event = event.into_inner().unwrap();
/// // do something with event data!
/// ```
///
#[derive(Clone)]
pub struct RwEvent<T>
where
    T: Event + Send + Sync,
{
    inner: Arc<RwLock<T>>,
}

impl<T> RwEvent<T>
where
    T: Event + Send + Sync,
{
    /// Creates a new RwEvent
    pub fn new(event: T) -> Self {
        Self { inner: Arc::new(RwLock::new(event)) }
    }

    /// Get a RwLockWriteGuard
    pub fn write(&self) -> std::sync::LockResult<std::sync::RwLockWriteGuard<'_, T>> {
        self.inner.write()
    }

    /// Get a RwLockReadGuard
    pub fn read(&self) -> std::sync::LockResult<std::sync::RwLockReadGuard<'_, T>> {
        self.inner.read()
    }


    /// Returns the inner value, if the `RwEvent` has exactly one strong reference.
    ///
    /// Otherwise, [`None`] is returned and the `RwEvent` is dropped.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// If `RwEvent::into_inner` is called on every clone of this `RwEvent`,
    /// it is guaranteed that exactly one of the calls returns the inner value.
    /// This means in particular that the inner value is not dropped.
    ///
    pub fn into_inner(self) -> Option<T> {
        if let Ok(inner) = Arc::into_inner(self.inner)?.into_inner() {
            Some(inner)
        } else {
            None
        }
    }
}

impl<T> Event for RwEvent<T>
where
    T: Event + Send + Sync,
{
    type Data = Self;
    type State = T::State;
    type Error = T::Error;

    fn name() -> &'static str {
        T::name()
    }
}

/// This event is triggered when the player attempts to log on to the server.
///
/// Beware that not all components on the entity may be set yet this event is mostly for:
/// a custom handshaking protocol before the player logs in using login plugin messages/etc.
///
#[derive(Event, Clone)]
pub struct PlayerStartLoginEvent {
    /// The entity that this event was fired for.
    pub entity: Entity,

    /// This profile can be changed and after the event is finished this will be the new profile.
    ///
    /// Be warned that this event can be cancelled or this field can be overriden by other listeners and this could mean your profile
    /// will never be used!
    ///
    pub profile: ferrumc_net::connection::GameProfile,
}

/// This event is triggered right after the client acknowledges the configuration state.
///
/// This event takes place after the [LoginGamePacket](ferrumc_net::packets::outgoing::login_play::LoginPlayPacket) is sent and the [Profile](crate::Profile) component is initialized.
#[derive(Event, Clone)]
pub struct PlayerJoinGameEvent {
    /// The entity that this event was fired for.
    pub entity: Entity,
}
