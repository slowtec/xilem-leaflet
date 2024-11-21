use xilem_leaflet::{map, marker, tile_layer};
use xilem_web::{
    document_body, elements::html, input_event_target_value, interfaces::Element, modifiers::style,
    App,
};

struct AppState {
    zoom_input: Option<String>,
    zoom: f64,
    center: (f64, f64),
    markers: Vec<(f64, f64)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            zoom_input: None,
            zoom: 12.0,
            center: (48.64, 9.46),
            markers: vec![(48.64, 9.46)],
        }
    }
}

const TILE_LAYER_URL: &str = "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";

fn app_logic(state: &mut AppState) -> impl Element<AppState> {
    let markers: Vec<_> = state
        .markers
        .iter()
        .map(|(lat, lng)| marker(*lat, *lng))
        .collect();
    html::div((
        html::label((
            "Zoom:",
            html::input(())
                .attr("value", state.zoom)
                .on_input(|state: &mut AppState, ev| {
                    state.zoom_input = input_event_target_value(&ev)
                        .and_then(|value| (!value.trim().is_empty()).then_some(value));
                })
                .on_keyup(|state: &mut AppState, ev| {
                    if &*ev.key() == "Enter" {
                        if let Some(Ok(value)) = state.zoom_input.as_ref().map(|v| v.parse()) {
                            state.zoom = value;
                        };
                    };
                }),
        )),
        map((tile_layer(TILE_LAYER_URL), markers))
            .center(state.center.0, state.center.1)
            .zoom(state.zoom)
            .on_zoom_end(|state: &mut AppState, map, _ev| {
                let zoom = map.get_zoom();
                log::debug!("Zoom has changed to {zoom}");
                state.zoom = zoom;
            })
            .on_move_end(|_state: &mut AppState, map, _ev| {
                let bounds = map.get_bounds();
                log::debug!("Bbox has changed to {bounds:?}");
            })
            .on_mouse_click(|state: &mut AppState, ev| {
                let lat_lng = ev.lat_lng();
                state.markers.push((lat_lng.lat(), lat_lng.lng()));
            }),
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
