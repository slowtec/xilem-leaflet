use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub const fn marker(lat: f64, lng: f64) -> Marker {
    Marker {
        lat,
        lng,
        options: MarkerOptions::new(),
    }
}

pub const fn marker_with_options(lat: f64, lng: f64, options: MarkerOptions) -> Marker {
    Marker { lat, lng, options }
}

#[derive(PartialEq)]
pub struct Marker {
    lat: f64,
    lng: f64,
    options: MarkerOptions,
}

// TODO:
// Add other options:
// <https://docs.rs/leaflet/latest/leaflet/struct.MarkerOptions.html>
#[derive(PartialEq)]
pub struct MarkerOptions {
    title: Option<String>,
    rise_on_hover: Option<bool>,
    opacity: Option<f64>,
}

impl MarkerOptions {
    pub const fn new() -> Self {
        MarkerOptions {
            title: None,
            rise_on_hover: None,
            opacity: None,
        }
    }

    pub const fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub const fn rise_on_hover(&self) -> Option<bool> {
        self.rise_on_hover
    }

    pub fn with_rise_on_hover(mut self, rise: bool) -> Self {
        self.rise_on_hover = Some(rise);
        self
    }

    pub const fn opacity(&self) -> Option<f64> {
        self.opacity
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = Some(opacity);
        self
    }
}

impl Default for MarkerOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewMarker for Marker {}

impl<State, Action> View<State, Action, MapCtx, DynMessage> for Marker {
    type Element = MapChildElement;

    type ViewState = ();

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let options = leaflet::MarkerOptions::new();
        if let Some(title) = &self.options.title {
            options.set_title(title.clone());
        }
        if let Some(rise) = &self.options.rise_on_hover {
            options.set_rise_on_hover(*rise);
        }
        let marker =
            leaflet::Marker::new_with_options(&leaflet::LatLng::new(self.lat, self.lng), &options);
        marker.add_to(ctx.map());
        (MapChildElement::Marker(marker), ())
    }

    fn rebuild(&self, prev: &Self, _: &mut Self::ViewState, _: &mut MapCtx, e: Mut<Self::Element>) {
        debug_assert!(
            matches!(e, MapChildElement::Marker(_)),
            "not a marker: {e:?}"
        );
        if self != prev {
            e.as_marker_mut()
                .set_lat_lng(&leaflet::LatLng::new(self.lat, self.lng));
            // TODO:
            // Update options:
            // <https://github.com/slowtec/leaflet-rs/issues/37>
        }
    }

    fn teardown(&self, _: &mut Self::ViewState, _: &mut MapCtx, e: Mut<Self::Element>) {
        e.as_marker_mut().remove();
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        _: &[ViewId],
        message: DynMessage,
        _: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        MessageResult::Stale(message)
    }
}
