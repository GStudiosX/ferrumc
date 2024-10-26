pub use ferrumc_net::errors::NetError;
pub use ferrumc_net::GlobalState;

pub use ferrumc_ecs::entities::Entity;

pub use ferrumc_macros::{event_handler, Event};
pub use ferrumc_events::{
    infrastructure::Event,
    errors::EventsError
};

pub use ferrumc_net::connection::ClientDisconnectEvent;

pub use ferrumc_net::packets::incoming::server_bound_plugin_message::LoginPluginResponseEvent;

use std::sync::{Arc, RwLock};

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
    pub fn new(event: T) -> Self {
        Self { inner: Arc::new(RwLock::new(event)) }
    }

    pub fn write(&self) -> std::sync::LockResult<std::sync::RwLockWriteGuard<'_, T>> {
        self.inner.write()
    }

    pub fn read(&self) -> std::sync::LockResult<std::sync::RwLockReadGuard<'_, T>> {
        self.inner.read()
    }

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

#[derive(Event, Clone)]
pub struct PlayerStartLoginEvent {
    pub entity: Entity,
    pub profile: ferrumc_net::connection::GameProfile,
}

#[derive(Event, Clone)]
pub struct PlayerJoinGameEvent {
    pub entity: Entity,
}
