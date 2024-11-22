use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub const fn tile_layer(url_template: &'static str) -> TileLayer {
    TileLayer { url_template }
}

pub struct TileLayer {
    url_template: &'static str,
}

impl ViewMarker for TileLayer {}

impl<State, Action> View<State, Action, MapCtx, DynMessage> for TileLayer
where
    State: 'static,
{
    type Element = MapChildElement;

    type ViewState = ();

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let tile_layer = leaflet::TileLayer::new(self.url_template);
        ctx.map().add_layer(&tile_layer);
        (MapChildElement::TileLayer(tile_layer), ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _: &mut Self::ViewState,
        map_ctx: &mut MapCtx,
        element: Mut<Self::Element>,
    ) {
        if prev.url_template != self.url_template {
            let tile_layer = element.as_tile_layer_mut();
            tile_layer.remove();
            *tile_layer = leaflet::TileLayer::new(self.url_template);
            tile_layer.add_to(map_ctx.map());
        }
    }

    fn teardown(&self, _: &mut Self::ViewState, _: &mut MapCtx, e: Mut<Self::Element>) {
        e.as_tile_layer_mut().remove();
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        _: &[ViewId],
        _: DynMessage,
        _: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        MessageResult::Nop
    }
}
