use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker as _},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub const fn on_zoom_end<State, F>(callback: F) -> OnZoomEnd<F>
where
    F: Fn(&mut State, leaflet::Map, leaflet::Event) + 'static,
{
    OnZoomEnd { callback }
}

pub struct OnZoomEnd<F> {
    callback: F,
}

impl<F> ViewMarker for OnZoomEnd<F> {}

#[derive(Debug)]
struct ZoomEndMessage(leaflet::Map, leaflet::Event);

/// Distinctive ID for better debugging
const ON_ZOOM_END_ID: ViewId = ViewId::new(23668);

impl<State, Action, F> View<State, Action, MapCtx, DynMessage> for OnZoomEnd<F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, leaflet::Map, leaflet::Event) + 'static,
{
    type Element = MapChildElement;

    type ViewState = ();

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        ctx.with_id(ON_ZOOM_END_ID, |ctx| {
            let thunk = ctx.dom_ctx.message_thunk();
            let map = ctx.map.clone();
            // TODO use add/remove_event_listener, for graceful lifecycle handling
            ctx.map.on_zoom_end(Box::new(move |ev| {
                thunk.enqueue_message(ZoomEndMessage(map.clone(), ev));
            }));
            (MapChildElement::Event, ())
        })
    }

    fn rebuild(&self, _: &Self, _: &mut Self::ViewState, ctx: &mut MapCtx, _: Mut<Self::Element>) {
        ctx.with_id(ON_ZOOM_END_ID, |_ctx| {
            // TODO
        })
    }

    fn teardown(&self, _: &mut Self::ViewState, ctx: &mut MapCtx, _: Mut<Self::Element>) {
        ctx.with_id(ON_ZOOM_END_ID, |_ctx| {
            // TODO
        })
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        id_path: &[ViewId],
        message: DynMessage,
        state: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        debug_assert!(id_path.len() == 1 && id_path[0] == ON_ZOOM_END_ID);
        let ZoomEndMessage(map, ev) = *message.downcast().unwrap();
        (self.callback)(state, map, ev);
        MessageResult::Nop
    }
}
