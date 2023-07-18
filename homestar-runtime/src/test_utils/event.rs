use crate::{event_handler::Event, settings::Settings};
use tokio::sync::mpsc;

/// Create an [mpsc::Sender], [mpsc::Receiver] pair for [Event]s.
pub fn setup_channel(settings: Settings) -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
    mpsc::channel(settings.node.network.events_buffer_len)
}
