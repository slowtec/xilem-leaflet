use std::{marker::PhantomData, rc::Rc};

use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    MessageThunk, ViewCtx,
};

use crate::{MapAction, MapChildElement, MapMessage};

pub fn on_zoom_end<State, F>(callback: F) -> OnZoomEnd<State, F>
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

pub struct OnZoomEndViewState {
    thunk: Rc<MessageThunk>,
}

impl<State, F> View<State, MapAction, ViewCtx, MapMessage> for OnZoomEnd<State, F>
where
    State: 'static,
    F: Fn(&mut State, f64) + 'static,
{
    type Element = MapChildElement;

    type ViewState = OnZoomEndViewState;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let thunk = Rc::new(ctx.message_thunk());
        let view_state = OnZoomEndViewState { thunk };
        (MapChildElement, view_state)
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
        view_state: &mut Self::ViewState,
        _: &[ViewId],
        message: MapMessage,
        state: &mut State,
    ) -> MessageResult<MapAction, MapMessage> {
        match message {
            MapMessage::MapHasMounted(map) => {
                let thunk = Rc::clone(&view_state.thunk);
                let map = map.clone();
                map.clone().on_zoom_end(Box::new(move |_| {
                    let zoom = map.get_zoom();
                    thunk.push_message(MapMessage::ZoomEnd(zoom));
                }));
            }
            MapMessage::ZoomEnd(zoom) => {
                (self.callback)(state, zoom);
            }
            _ => {}
        }
        MessageResult::Nop
    }
}
