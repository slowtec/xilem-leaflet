use std::marker::PhantomData;

use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    ViewCtx,
};

use crate::{MapAction, MapChildElement, MapMessage};

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
}

impl<State: 'static> View<State, MapAction, ViewCtx, MapMessage> for TileLayer<State> {
    type Element = MapChildElement;

    type ViewState = TileLayerViewState;

    fn build(&self, _: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let tile_layer = leaflet::TileLayer::new(self.url_template);
        let view_state = TileLayerViewState { tile_layer };
        (MapChildElement, view_state)
    }

    fn rebuild<'el>(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        _: &mut ViewCtx,
        element: Mut<'el, Self::Element>,
    ) -> Mut<'el, Self::Element> {
        if prev.url_template != self.url_template {
            view_state.tile_layer = leaflet::TileLayer::new(self.url_template);
        }
        element
    }

    fn teardown(&self, _: &mut Self::ViewState, _: &mut ViewCtx, _: Mut<Self::Element>) {
        // TODO
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        _: &[ViewId],
        message: MapMessage,
        _: &mut State,
    ) -> MessageResult<MapAction, MapMessage> {
        log::debug!("Handle message: {message:?}");
        if let MapMessage::MapHasMounted(map) = message {
            view_state.tile_layer.add_to(&map);
        }
        MessageResult::Nop
    }
}
