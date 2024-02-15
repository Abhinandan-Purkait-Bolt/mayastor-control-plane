use events_api::event::{EventAction, EventMessage, EventMeta};

/// Event trait definition for creating events.
pub trait Event {
    /// Create event message.
    fn event(&self, event_action: EventAction) -> EventMessage;
}

/// Event trait definition for creating events and adding meta data.
pub trait EventWithMeta {
    /// Create event message with meta data.
    fn event(&self, action: EventAction, meta: EventMeta) -> EventMessage;
}

/// A trait for generating event metadata.
pub trait EventMetaGen {
    /// Create metadata to be included with the event.
    fn meta(&self) -> EventMeta;
}
