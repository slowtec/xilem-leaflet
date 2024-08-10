use web_sys::wasm_bindgen::{JsCast, UnwrapThrowExt};
use xilem_leaflet::{map, tile_layer};
use xilem_web::{
    document_body,
    elements::html,
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

fn app_logic(state: &mut AppState) -> impl Element<AppState> {
    html::div((
        html::label((
            "Zoom:",
            html::input(())
                .attr("value", state.zoom)
                .on_input(|state: &mut AppState, ev| {
                    let value = event_target_value(ev);
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
        map(vec![tile_layer(
            "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        )])
        .zoom(state.zoom)
        .center(state.center.0, state.center.1), // TODO .on_zoom(|state, zoom|{ })
    ))
    .style(style("width", "100%"))
    .style(style("height", "100%"))
}

fn event_target_value(ev: web_sys::Event) -> String {
    ev.target()
        .unwrap_throw()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    log::debug!("Start web app");
    App::new(document_body(), AppState::default(), app_logic).run();
}
