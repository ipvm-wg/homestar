use crate::{
    channel::{AsyncChannel, AsyncChannelReceiver, AsyncChannelSender},
    event_handler::Event,
    settings,
    worker::WorkerMessage,
};

/// Create an [AsynBoundedChannelSender], [AsyncChannelReceiver] pair for [Event]s.
pub(crate) fn setup_event_channel(
    settings: settings::Node,
) -> (AsyncChannelSender<Event>, AsyncChannelReceiver<Event>) {
    AsyncChannel::with(settings.network.events_buffer_len)
}

/// Create an [AsyncChannelSender], [AsyncChannelReceiver] pair for worker messages.
pub(crate) fn setup_worker_channel(
    settings: settings::Node,
) -> (
    AsyncChannelSender<WorkerMessage>,
    AsyncChannelReceiver<WorkerMessage>,
) {
    AsyncChannel::with(settings.network.events_buffer_len)
}
