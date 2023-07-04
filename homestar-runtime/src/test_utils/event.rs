use crate::{settings::Settings, Event};
use tokio::sync::mpsc;

pub fn setup_channel(settings: Settings) -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
    mpsc::channel(settings.node().network.events_buffer_len)
}
