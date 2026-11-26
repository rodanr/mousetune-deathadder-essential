// SPDX-License-Identifier: GPL-3.0-or-later
//! The single thread that owns the mouse. All device I/O happens here, so
//! the UI thread never blocks and commands can never interleave.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use mousetune_core::{Error, Model, TransportError};
use mousetune_hid::{
    ConnectError, Connected, Connection, Discovery, HidapiTransport, InputAccess,
    input_monitoring_access, request_input_monitoring,
};

use crate::device_ops::{Change, apply, read_state};
use crate::prefs::Prefs;
use crate::state::{AppState, DeviceState, Section};

/// How often the worker looks for (un)plugged mice while idle.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Messages from the UI to the worker.
#[derive(Debug)]
pub enum Request {
    /// Apply a settings change.
    Apply(Change),
    /// Show the OS permission prompt, then retry connecting.
    GrantAccess,
    /// Stop the worker.
    Shutdown,
}

/// Messages from the worker to the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The overall state changed.
    State(AppState),
    /// A change was applied.
    Applied(Section),
    /// A change failed; the message is user facing.
    Failed(Section, String),
}

/// Handle to a running worker.
pub struct WorkerHandle {
    requests: Sender<Request>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    /// Queues a request. Ignored if the worker has already stopped.
    pub fn send(&self, request: Request) {
        let _ = self.requests.send(request);
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.requests.send(Request::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts the worker thread. `emit` is called from that thread for every
/// event, so it must hand work over to the UI thread itself.
///
/// # Errors
///
/// Returns an error if hidapi cannot initialise or the thread cannot start.
pub fn spawn(prefs: Prefs, emit: impl Fn(Event) + Send + 'static) -> anyhow::Result<WorkerHandle> {
    let discovery = Discovery::new()?;
    let (requests, inbox) = mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("mouse-worker".into())
        .spawn(move || {
            let mut worker = Worker::new(discovery, prefs, emit);
            worker.run(&inbox);
        })?;
    Ok(WorkerHandle {
        requests,
        thread: Some(thread),
    })
}

struct Active {
    device: mousetune_core::RazerDevice<HidapiTransport>,
    state: DeviceState,
}

struct Worker<F: Fn(Event)> {
    discovery: Discovery,
    prefs: Prefs,
    emit: F,
    active: Option<Active>,
    shown: Option<AppState>,
    asked_for_access: bool,
}

impl<F: Fn(Event)> Worker<F> {
    fn new(discovery: Discovery, prefs: Prefs, emit: F) -> Self {
        Self {
            discovery,
            prefs,
            emit,
            active: None,
            shown: None,
            asked_for_access: false,
        }
    }

    fn run(&mut self, inbox: &mpsc::Receiver<Request>) {
        self.tick();
        loop {
            match inbox.recv_timeout(POLL_INTERVAL) {
                Ok(Request::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
                Ok(Request::Apply(change)) => self.apply(change),
                Ok(Request::GrantAccess) => {
                    let _ = request_input_monitoring();
                    self.tick();
                }
                Err(RecvTimeoutError::Timeout) => self.tick(),
            }
        }
    }

    /// Periodic work: find a mouse when none is open, notice an unplug otherwise.
    fn tick(&mut self) {
        if let Some(active) = &self.active {
            if !self.discovery.is_present(active.device.model()) {
                tracing::info!("mouse unplugged");
                self.active = None;
                self.show(AppState::Searching);
            }
            return;
        }
        self.connect();
    }

    fn connect(&mut self) {
        match self.discovery.connect() {
            Connection::Absent => self.show(AppState::Searching),
            Connection::Unsupported { product_id } => {
                tracing::info!(product_id, "unsupported Razer mouse");
                self.show(AppState::Unsupported);
            }
            Connection::Failed { model, error } => {
                let state = self.state_for_failure(model, &error);
                self.show(state);
            }
            Connection::Connected(connected) => self.adopt(*connected),
        }
    }

    fn adopt(&mut self, connected: Connected) {
        let Connected {
            mut device,
            firmware,
        } = connected;
        match read_state(&mut device, firmware, &self.prefs) {
            Ok(state) => {
                tracing::info!(model = state.model.name(), ?firmware, "connected");
                self.show(AppState::Connected(state.clone()));
                self.active = Some(Active { device, state });
            }
            Err(error) => self.show(AppState::Error(error.to_string())),
        }
    }

    fn state_for_failure(&mut self, model: Model, error: &ConnectError) -> AppState {
        tracing::debug!(model = model.name(), ?error, "could not open the mouse");
        match error {
            ConnectError::PermissionDenied => {
                if !self.asked_for_access && input_monitoring_access() == InputAccess::Denied {
                    self.asked_for_access = true;
                    let _ = request_input_monitoring();
                }
                AppState::NeedsPermission
            }
            ConnectError::InUse => AppState::Busy,
            ConnectError::NoResponse(message) => AppState::Error(message.clone()),
        }
    }

    fn apply(&mut self, change: Change) {
        let section = change.section();
        let Some(active) = &mut self.active else {
            (self.emit)(Event::Failed(section, "No mouse connected.".to_owned()));
            return;
        };
        match apply(&mut active.device, &mut active.state, change) {
            Ok(()) => {
                if let Change::Effect(led, effect) = change {
                    self.prefs.set_effect(&active.state.serial, led, effect);
                }
                let state = active.state.clone();
                (self.emit)(Event::Applied(section));
                self.show(AppState::Connected(state));
            }
            Err(error) => {
                let lost = matches!(error, Error::Transport(TransportError::Disconnected));
                (self.emit)(Event::Failed(section, error.to_string()));
                if lost {
                    self.active = None;
                    self.show(AppState::Searching);
                }
            }
        }
    }

    /// Emits a state only when it differs from what the UI already shows.
    fn show(&mut self, state: AppState) {
        if self.shown.as_ref() != Some(&state) {
            tracing::info!(?state, "state changed");
            self.shown = Some(state.clone());
            (self.emit)(Event::State(state));
        }
    }
}
