use std::marker::PhantomData;

use xilem_web::{
    core::{MessageResult, Mut, NoElement, View, ViewId, ViewMarker},
    DynMessage, ViewCtx,
};

pub fn on_zoom<State, Action, F>(callback: F) -> OnZoom<State, Action, F>
where
    F: Fn(&mut State, f64) + 'static,
{
    OnZoom {
        callback,
        phantom: PhantomData,
    }
}

pub struct OnZoom<State, Action, F>
where
    F: Fn(&mut State, f64) + 'static,
{
    callback: F,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action, F> ViewMarker for OnZoom<State, Action, F> where F: Fn(&mut State, f64) + 'static
{}

pub struct OnZoomViewState;

impl<State, Action, F> View<State, Action, ViewCtx, DynMessage> for OnZoom<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, f64) + 'static,
{
    type Element = NoElement;

    type ViewState = OnZoomViewState;

    fn build(&self, _: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        // TODO:
        todo!()
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
        _: DynMessage,
        _: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        todo!()
    }
}
