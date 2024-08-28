use std::marker::PhantomData;

use xilem_web::core::{MessageResult, Mut, View, ViewId, ViewMarker};

use crate::{MapAction, MapChildElement, MapCtx, MapMessage};

pub fn tile_layer<State>(url_template: &'static str) -> TileLayer<State> {
    TileLayer {
        url_template,
        phantom: PhantomData,
    }
}

pub struct TileLayer<State> {
    url_template: &'static str,
    phantom: PhantomData<fn() -> State>,
}

impl<State> ViewMarker for TileLayer<State> {}

pub struct TileLayerViewState {
    tile_layer: leaflet::TileLayer,
    added_to_map: bool,
}

impl<State: 'static> View<State, MapAction, MapCtx, MapMessage> for TileLayer<State> {
    type Element = MapChildElement;

    type ViewState = TileLayerViewState;

    fn build(&self, _: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let tile_layer = leaflet::TileLayer::new(self.url_template);
        let added_to_map = false;
        let view_state = TileLayerViewState {
            tile_layer,
            added_to_map,
        };
        (MapChildElement, view_state)
    }

    fn rebuild<'el>(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        map_ctx: &mut MapCtx,
        element: Mut<'el, Self::Element>,
    ) -> Mut<'el, Self::Element> {
        if prev.url_template != self.url_template {
            view_state.tile_layer = leaflet::TileLayer::new(self.url_template);
        }
        if !view_state.added_to_map {
            view_state.tile_layer.add_to(map_ctx.map());
            view_state.added_to_map = true;
        }

        element
    }

    fn teardown(&self, _: &mut Self::ViewState, _: &mut MapCtx, _: Mut<Self::Element>) {
        // TODO
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
