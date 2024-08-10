use std::marker::PhantomData;

use xilem_web::{
    core::{MessageResult, Mut, NoElement, View, ViewId, ViewMarker},
    DynMessage, ViewCtx,
};

use crate::{MapChild, MapChildViewState};

impl<State, Action> ViewMarker for TileLayer<State, Action> {}
pub fn tile_layer<State, Action>(url_template: &'static str) -> TileLayer<State, Action> {
    TileLayer {
        url_template,
        phantom: PhantomData,
    }
}

pub struct TileLayer<State, Action> {
    url_template: &'static str,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action> MapChild<State, Action> for TileLayer<State, Action>
where
    State: 'static,
    Action: 'static,
{
}

pub struct TileLayerViewState {
    tile_layer: leaflet::TileLayer,
}

impl MapChildViewState for TileLayerViewState {
    fn after_build(&mut self, map: &leaflet::Map) {
        self.tile_layer.add_to(map);
    }

    fn after_rebuild(&mut self, _map: &leaflet::Map) {
        // TODO replace tile layer
    }
}

impl<State, Action> View<State, Action, ViewCtx, DynMessage> for TileLayer<State, Action>
where
    State: 'static,
    Action: 'static,
{
    type Element = NoElement;

    type ViewState = TileLayerViewState;

    fn build(&self, _: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let tile_layer = leaflet::TileLayer::new(self.url_template);
        let view_state = TileLayerViewState { tile_layer };
        (NoElement, view_state)
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
        _: &mut Self::ViewState,
        _: &[ViewId],
        _: DynMessage,
        _: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        todo!()
    }
}
