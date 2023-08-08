use crate::{event_handler::Event, settings, worker::WorkerMessage};
use tokio::sync::mpsc;

/// Create an [mpsc::Sender], [mpsc::Receiver] pair for [Event]s.
pub(crate) fn setup_event_channel(
    settings: settings::Node,
) -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
    mpsc::channel(settings.network.events_buffer_len)
}

/// Create an [mpsc::Sender], [mpsc::Receiver] pair for worker messages.
pub(crate) fn setup_worker_channel(
    settings: settings::Node,
) -> (mpsc::Sender<WorkerMessage>, mpsc::Receiver<WorkerMessage>) {
    mpsc::channel(settings.network.events_buffer_len)
}
