use xilem_leaflet::{map, tile_layer};
use xilem_web::{
    document_body,
    elements::html,
    input_event_target_value,
    interfaces::{Element, HtmlElement},
    style, App,
};

struct AppState {
    zoom_input: Option<String>,
    zoom: f64,
    center: (f64, f64),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            zoom_input: None,
            zoom: 12.0,
            center: (48.64, 9.46),
        }
    }
}

const TILE_LAYER_URL: &str = "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";

fn app_logic(state: &mut AppState) -> impl Element<AppState> {
    html::div((
        html::label((
            "Zoom:",
            html::input(())
                .attr("value", state.zoom)
                .on_input(|state: &mut AppState, ev| {
                    let Some(value) = input_event_target_value(&ev) else {
                        return;
                    };
                    state.zoom_input = if !value.trim().is_empty() {
                        Some(value)
                    } else {
                        None
                    };
                })
                .on_keyup(|state: &mut AppState, ev| {
                    match &*ev.key() {
                        "Enter" => {
                            let Some(Ok(value)) = state.zoom_input.as_ref().map(|v| v.parse())
                            else {
                                return;
                            };
                            state.zoom = value;
                        }
                        _ => {}
                    };
                }),
        )),
        map((
            tile_layer(TILE_LAYER_URL),
            // TODO:
            // on_zoom_end(|_state, _zoom| {
            //     log::debug!("Zoom ended");
            // }),
        ))
        .zoom(state.zoom)
        .center(state.center.0, state.center.1),
    ))
    .style(style("width", "100%"))
    .style(style("height", "100%"))
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    log::debug!("Start web app");
    App::new(document_body(), AppState::default(), app_logic).run();
}
