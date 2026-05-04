use bevy::app::App;
use bevy::ecs::prelude::Resource;
use bevy::log::tracing_subscriber::layer::{Context, Layer};
use bevy::log::tracing_subscriber::registry::LookupSpan;
use bevy::log::BoxedLayer;
use bevy::utils::tracing::field::{Field, Visit};
use bevy::utils::tracing::{Event, Subscriber};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

static LOG_CAPTURE_SENDER: OnceLock<Sender<String>> = OnceLock::new();

#[derive(Resource)]
pub struct LogCaptureReceiver {
    receiver: Mutex<Receiver<String>>,
}

impl LogCaptureReceiver {
    pub fn new(receiver: Receiver<String>) -> Self {
        Self {
            receiver: Mutex::new(receiver),
        }
    }

    pub fn try_drain(&self) -> Vec<String> {
        let Ok(receiver) = self.receiver.lock() else {
            return Vec::new();
        };

        let mut lines = Vec::new();
        while let Ok(line) = receiver.try_recv() {
            lines.push(line);
        }
        lines
    }
}

pub fn init_log_capture_receiver() -> LogCaptureReceiver {
    let (sender, receiver) = mpsc::channel::<String>();
    let _ = LOG_CAPTURE_SENDER.set(sender);
    LogCaptureReceiver::new(receiver)
}

pub fn make_in_game_console_layer(_app: &mut App) -> Option<BoxedLayer> {
    LOG_CAPTURE_SENDER
        .get()
        .cloned()
        .map(|sender| Box::new(InGameConsoleLogLayer { sender }) as BoxedLayer)
}

struct InGameConsoleLogLayer {
    sender: Sender<String>,
}

impl<S> Layer<S> for InGameConsoleLogLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let message = if visitor.message.is_empty() {
            String::from("(log without message field)")
        } else {
            visitor.message
        };

        let _ = self
            .sender
            .send(message);
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}
