use crate::{
    channel::{AsyncBoundedChannel, AsyncBoundedChannelReceiver, AsyncBoundedChannelSender},
    event_handler::Event,
    settings,
    worker::WorkerMessage,
};

/// Create an [AsynBoundedChannelSender], [AsyncBoundedChannelReceiver] pair for [Event]s.
pub(crate) fn setup_event_channel(
    settings: settings::Node,
) -> (
    AsyncBoundedChannelSender<Event>,
    AsyncBoundedChannelReceiver<Event>,
) {
    AsyncBoundedChannel::with(settings.network.events_buffer_len)
}

/// Create an [AsyncBoundedChannelSender], [AsyncBoundedChannelReceiver] pair for worker messages.
pub(crate) fn setup_worker_channel(
    settings: settings::Node,
) -> (
    AsyncBoundedChannelSender<WorkerMessage>,
    AsyncBoundedChannelReceiver<WorkerMessage>,
) {
    AsyncBoundedChannel::with(settings.network.events_buffer_len)
}
