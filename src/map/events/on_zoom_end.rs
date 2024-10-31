use std::{marker::PhantomData, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub(crate) const fn on_zoom_end<State, F>(callback: F) -> OnZoomEnd<State, F>
where
    F: Fn(&mut State, f64) + 'static,
{
    OnZoomEnd {
        callback,
        phantom: PhantomData,
    }
}

pub struct OnZoomEnd<State, F> {
    callback: F,
    phantom: PhantomData<fn() -> State>,
}

impl<State, F> ViewMarker for OnZoomEnd<State, F> {}

pub struct OnZoomEndViewState;

#[derive(Debug)]
struct ZoomEndMessage(f64);

impl<State, Action, F> View<State, Action, MapCtx, DynMessage> for OnZoomEnd<State, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, f64) + 'static,
{
    type Element = MapChildElement;

    type ViewState = OnZoomEndViewState;

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let thunk = Rc::clone(&ctx.thunk);
        let map = ctx.map.clone();
        ctx.map.on_zoom_end(Box::new(move |_| {
            log::debug!("Zoom changed");
            let zoom = map.get_zoom();
            let thunk = Rc::clone(&thunk);
            spawn_local(async move {
                thunk.push_message(ZoomEndMessage(zoom));
            });
        }));
        let view_state = OnZoomEndViewState { /*thunk */};
        (MapChildElement, view_state)
    }

    fn rebuild(&self, _: &Self, _: &mut Self::ViewState, _: &mut MapCtx, _: Mut<Self::Element>) {
        // TODO:
    }

    fn teardown(&self, _: &mut Self::ViewState, _: &mut MapCtx, _: Mut<Self::Element>) {
        // TODO
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        _: &[ViewId],
        message: DynMessage,
        state: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        let ZoomEndMessage(zoom) = *message.downcast().unwrap();
        (self.callback)(state, zoom);
        MessageResult::Nop
    }
}
