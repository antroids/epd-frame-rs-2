use crate::errors::DeviceError;
use crate::http::*;
use crate::storage::PersistentState;
use alloc::string::String;
use alloc::vec;
use defmt_or_log::{derive_format_or_debug, info};
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_sync::channel::Channel;
use picoserve::extract::State;
use picoserve::response::{IntoResponse, Response};
use picoserve::routing::PathRouter;
use picoserve::{AppBuilder, AppRouter, Router, make_static};

mod html;

// ── Handlers ──────────────────────────────────────────────────────────────────

/// Actions sent from the HTTP server back to the application.
#[derive_format_or_debug]
pub enum ServerAction {
    WriteState(PersistentState),
    Restart,
}

pub const ACTION_CHANNEL_CAPACITY: usize = 4;
const TCP_RX_BUFFER_SIZE: usize = 1024 * 8;
const TCP_TX_BUFFER_SIZE: usize = 1024 * 8;
const HTTP_BUFFER_SIZE: usize = 1024 * 8;
const WEB_TASK_POOL: usize = 3;

pub type ActionChannel = Channel<crate::RawMutex, ServerAction, ACTION_CHANNEL_CAPACITY>;

#[derive(Clone)]
pub struct AppState {
    pub current_state: PersistentState,
    pub action_channel: &'static ActionChannel,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn handle_index(State(state): State<AppState>) -> impl IntoResponse {
    let markup = html::render_page(&state.current_state);
    Response::ok(markup.into_string()).with_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML)
}

async fn handle_alpinejs() -> impl IntoResponse {
    static ALPINEJS: &str = include_str!("../../resources/js/npm/alpinejs@3.x.x/dist/cdn.min.js");
    Response::ok(ALPINEJS).with_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JAVASCRIPT)
}

/// Parse the JSON body into `PersistentState`, preserving the stored `version`.
fn parse_json_body(body: &str, base: &PersistentState) -> Option<PersistentState> {
    let mut new: PersistentState = serde_json::from_str(body).ok()?;
    new.version = crate::storage::VERSION;
    new.wifi_join_options.passphrase_is_prehashed = base.wifi_join_options.passphrase_is_prehashed;
    Some(new)
}

async fn handle_save(State(state): State<AppState>, body: String) -> impl IntoResponse {
    match parse_json_body(&body, &state.current_state) {
        Some(new_state) => {
            let _ = state
                .action_channel
                .try_send(ServerAction::WriteState(new_state));
            Response::ok("Saved").with_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN)
        }
        None => Response::new(picoserve::response::StatusCode::BAD_REQUEST, "Invalid JSON")
            .with_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN),
    }
}

async fn handle_save_restart(State(state): State<AppState>, body: String) -> impl IntoResponse {
    match parse_json_body(&body, &state.current_state) {
        Some(new_state) => {
            let _ = state
                .action_channel
                .try_send(ServerAction::WriteState(new_state));
            let _ = state.action_channel.try_send(ServerAction::Restart);
            Response::ok("Restarting").with_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN)
        }
        None => Response::new(picoserve::response::StatusCode::BAD_REQUEST, "Invalid JSON")
            .with_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN),
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

impl AppBuilder for AppState {
    type PathRouter = impl PathRouter;

    fn build_app(self) -> Router<Self::PathRouter> {
        Router::new()
            .route("/", picoserve::routing::get(handle_index))
            .route("/alpine.js", picoserve::routing::get(handle_alpinejs))
            .route("/save", picoserve::routing::post(handle_save))
            .route(
                "/save_restart",
                picoserve::routing::post(handle_save_restart),
            )
            .with_state(self)
    }
}

// ── Embassy task ──────────────────────────────────────────────────────────────

#[embassy_executor::task(pool_size = WEB_TASK_POOL)]
async fn web_task(
    id: usize,
    stack: Stack<'static>,
    config: &'static picoserve::Config,
    app: &'static AppRouter<AppState>,
) {
    let mut tcp_rx = vec![0u8; TCP_RX_BUFFER_SIZE];
    let mut tcp_tx = vec![0u8; TCP_TX_BUFFER_SIZE];
    let mut http_buffer = vec![0; HTTP_BUFFER_SIZE];

    info!("Starting web server task with ID {}", id);
    picoserve::Server::new(app, config, &mut http_buffer)
        .listen_and_serve(id, stack, 80, &mut tcp_rx, &mut tcp_tx)
        .await;
}

// ── Public entry-point ────────────────────────────────────────────────────────

pub fn start_http_server(
    spawner: &Spawner,
    stack: Stack<'static>,
    _seed: u64,
    initial_state: PersistentState,
    action_channel: &'static ActionChannel,
) -> Result<(), DeviceError> {
    use core::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    let app_state = AppState {
        current_state: initial_state,
        action_channel,
    };
    let app = make_static!(AppRouter<AppState>, app_state.build_app());
    let config = make_static!(
        picoserve::Config,
        picoserve::Config::const_default().keep_connection_alive()
    );

    for _ in 0..WEB_TASK_POOL {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        spawner.spawn(web_task(id, stack, config, app)?);
    }
    Ok(())
}
