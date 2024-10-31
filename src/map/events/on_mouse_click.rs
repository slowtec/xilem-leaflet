use std::{marker::PhantomData, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    DynMessage,
};

use crate::{MapChildElement, MapCtx};

pub(crate) const fn on_mouse_click<State, F>(callback: F) -> OnMouseClick<State, F>
where
    F: Fn(&mut State, leaflet::MouseEvent) + 'static,
{
    OnMouseClick {
        callback,
        phantom: PhantomData,
    }
}

pub struct OnMouseClick<State, F> {
    callback: F,
    phantom: PhantomData<fn() -> State>,
}

impl<State, F> ViewMarker for OnMouseClick<State, F> {}

pub struct OnMouseClickViewState;

#[derive(Debug)]
struct ClickMessage(leaflet::MouseEvent);

impl<State, Action, F> View<State, Action, MapCtx, DynMessage> for OnMouseClick<State, F>
where
    State: 'static,
    F: Fn(&mut State, leaflet::MouseEvent) + 'static,
{
    type Element = MapChildElement;

    type ViewState = OnMouseClickViewState;

    fn build(&self, ctx: &mut MapCtx) -> (Self::Element, Self::ViewState) {
        let thunk = Rc::clone(&ctx.thunk);
        ctx.map.on_mouse_click(Box::new(move |ev| {
            let thunk = Rc::clone(&thunk);
            spawn_local(async move {
                thunk.push_message(ClickMessage(ev));
            });
        }));
        let view_state = OnMouseClickViewState {};
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
        let ClickMessage(ev) = *message.downcast().unwrap();
        (self.callback)(state, ev);
        MessageResult::Nop
    }
}
