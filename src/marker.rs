use std::marker::PhantomData;

use xilem_web::core::{MessageResult, Mut, View, ViewId, ViewMarker};

use crate::{MapAction, MapChildElement, MapCtx, MapMessage};

pub const fn marker<State>(lat: f64, lng: f64) -> Marker<State> {
    Marker {
        lat,
        lng,
        phantom: PhantomData,
    }
}

pub struct Marker<State> {
    lat: f64,
    lng: f64,
    phantom: PhantomData<fn() -> State>,
}

impl<State> ViewMarker for Marker<State> {}

pub struct MarkerViewState {
    marker: leaflet::Marker,
}

impl<State: 'static> View<State, MapAction, MapCtx, MapMessage> for Marker<State> {
    type Element = MapChildElement;

    type ViewState = MarkerViewState;

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let marker = leaflet::Marker::new(&leaflet::LatLng::new(self.lat, self.lng));
        marker.add_to(ctx.map());
        let view_state = MarkerViewState { marker };
        (MapChildElement, view_state)
    }

    fn rebuild(&self, _: &Self, _: &mut Self::ViewState, _: &mut MapCtx, _: Mut<Self::Element>) {}

    fn teardown(&self, view_state: &mut Self::ViewState, ctx: &mut MapCtx, _: Mut<Self::Element>) {
        view_state.marker.remove_from(ctx.map());
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        _: &[ViewId],
        _: MapMessage,
        _: &mut State,
    ) -> MessageResult<MapAction, MapMessage> {
        MessageResult::Nop
    }
}