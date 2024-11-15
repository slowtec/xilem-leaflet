use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub const fn marker(lat: f64, lng: f64) -> Marker {
    Marker { lat, lng }
}

#[derive(PartialEq)]
pub struct Marker {
    lat: f64,
    lng: f64,
}

impl ViewMarker for Marker {}

impl<State, Action> View<State, Action, MapCtx, DynMessage> for Marker {
    type Element = MapChildElement;

    type ViewState = ();

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let marker = leaflet::Marker::new(&leaflet::LatLng::new(self.lat, self.lng));
        marker.add_to(ctx.map());
        (MapChildElement::Marker(marker), ())
    }

    fn rebuild(&self, prev: &Self, _: &mut Self::ViewState, _: &mut MapCtx, e: Mut<Self::Element>) {
        if self != prev {
            e.as_marker_mut()
                .set_lat_lng(&leaflet::LatLng::new(self.lat, self.lng));
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
