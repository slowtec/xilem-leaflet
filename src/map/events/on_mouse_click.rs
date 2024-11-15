use web_sys::wasm_bindgen::UnwrapThrowExt;
use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub(crate) const fn on_mouse_click<State, F>(callback: F) -> OnMouseClick<F>
where
    F: Fn(&mut State, leaflet::MouseEvent) + 'static,
{
    OnMouseClick { callback }
}

pub struct OnMouseClick<F> {
    callback: F,
}

impl<F> ViewMarker for OnMouseClick<F> {}

#[derive(Debug)]
struct ClickMessage(leaflet::MouseEvent);

/// Distinctive ID for better debugging
const ON_MOUSE_CLICK_ID: ViewId = ViewId::new(23668);

impl<State, Action, F> View<State, Action, MapCtx, DynMessage> for OnMouseClick<F>
where
    State: 'static,
    F: Fn(&mut State, leaflet::MouseEvent) + 'static,
{
    type Element = MapChildElement;

    type ViewState = ();

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        ctx.with_id(ON_MOUSE_CLICK_ID, |ctx| {
            let thunk = ctx.dom_ctx.message_thunk();
            // TODO use add/remove_event_listener, for graceful lifecycle handling
            ctx.map
                .on_mouse_click(Box::new(move |ev| thunk.push_message(ClickMessage(ev))));
            (MapChildElement::Event, ())
        })
    }

    fn rebuild(&self, _: &Self, _: &mut Self::ViewState, ctx: &mut MapCtx, _: Mut<Self::Element>) {
        ctx.with_id(ON_MOUSE_CLICK_ID, |_ctx| {
            // TODO
        })
    }

    fn teardown(&self, _: &mut Self::ViewState, ctx: &mut MapCtx, _: Mut<Self::Element>) {
        ctx.with_id(ON_MOUSE_CLICK_ID, |_ctx| {
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
        debug_assert!(id_path.len() == 1 && id_path[0] == ON_MOUSE_CLICK_ID);
        let ClickMessage(ev) = *message.downcast().unwrap_throw();
        (self.callback)(state, ev);
        MessageResult::Nop
    }
}
