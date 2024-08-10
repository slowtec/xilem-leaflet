use std::{marker::PhantomData, rc::Rc};

use xilem_web::{
    core::{MessageResult, Mut, NoElement, View, ViewId, ViewMarker},
    DynMessage, MessageThunk, ViewCtx,
};

use crate::MapChildViewState;

pub fn on_zoom_end<State, Action, F>(callback: F) -> OnZoomEnd<State, Action, F>
where
    F: Fn(&mut State, f64) + 'static,
{
    OnZoomEnd {
        callback,
        phantom: PhantomData,
    }
}

pub struct OnZoomEnd<State, Action, F>
where
    F: Fn(&mut State, f64) + 'static,
{
    callback: F,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action, F> ViewMarker for OnZoomEnd<State, Action, F> where
    F: Fn(&mut State, f64) + 'static
{
}

pub struct OnZoomEndViewState {
    thunk: Rc<MessageThunk>,
}

#[derive(Debug)]
enum Message {
    ZoomEnd(f64),
}

impl MapChildViewState for OnZoomEndViewState {
    fn after_build(&mut self, map: &leaflet::Map) {
        let thunk = Rc::clone(&self.thunk);
        let map = map.clone();
        map.clone().on_zoom_end(Box::new(move |_| {
            let zoom = map.get_zoom();
            thunk.push_message(Message::ZoomEnd(zoom));
        }));
    }

    fn after_rebuild(&mut self, _: &leaflet::Map) {}
}

impl<State, Action, F> View<State, Action, ViewCtx, DynMessage> for OnZoomEnd<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, f64) + 'static,
{
    type Element = NoElement;

    type ViewState = OnZoomEndViewState;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let thunk = Rc::new(ctx.message_thunk());
        let view_state = OnZoomEndViewState { thunk };
        (NoElement, view_state)
    }

    fn rebuild<'el>(
        &self,
        _: &Self,
        _: &mut Self::ViewState,
        _: &mut ViewCtx,
        _: Mut<'el, Self::Element>,
    ) -> Mut<'el, Self::Element> {
        todo!()
    }

    fn teardown(&self, _: &mut Self::ViewState, _: &mut ViewCtx, _: Mut<Self::Element>) {
        // TODO
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        _: &[ViewId],
        message: DynMessage,
        state: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        match *message.downcast::<Message>().unwrap() {
            Message::ZoomEnd(zoom) => {
                (self.callback)(state, zoom);
            }
        }
        MessageResult::Nop
    }
}
