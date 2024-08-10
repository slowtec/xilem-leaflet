use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use web_sys::wasm_bindgen::JsCast;
use xilem_web::{
    core::{MessageResult, Mut, View, ViewId, ViewMarker},
    DomNode, DynMessage, ElementProps, Pod, ViewCtx, WithStyle, HTML_NS,
};

mod events;
pub use self::events::*;

// TODO:
// How can we pass tuples, like `()`, `(C)`, `(C, C)`?
pub fn map<State, Action, C>(children: Vec<C>) -> Map<State, Action, C>
where
    C: MapChild<State, Action>,
    C::ViewState: MapChildViewState,
{
    Map {
        zoom: None,
        center: None,
        children,
        phantom: PhantomData,
    }
}

/// This is a marker trait that is used to pass only map-specific children
/// to the [`map`] function.
pub trait MapChild<State, Action>: View<State, Action, ViewCtx, DynMessage>
where
    Self::ViewState: MapChildViewState,
{
}

/// This is a hack that allows children of the map
/// to get access to [`leaflet::Map`].
pub trait MapChildViewState {
    fn after_build(&mut self, map: &leaflet::Map);
    fn after_rebuild(&mut self, map: &leaflet::Map);
}

pub struct Map<State, Action, C>
where
    C: MapChild<State, Action>,
    C::ViewState: MapChildViewState,
{
    children: Vec<C>,
    zoom: Option<f64>,
    center: Option<(f64, f64)>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action, C> Map<State, Action, C>
where
    C: MapChild<State, Action>,
    C::ViewState: MapChildViewState,
{
    pub fn zoom(mut self, value: f64) -> Self {
        self.zoom = Some(value);
        self
    }

    pub fn on_zoom<F>(&self, callback: F) -> OnZoom<State, Action, F>
    where
        F: Fn(&mut State, f64) + 'static,
    {
        on_zoom(callback)
    }

    pub fn center(mut self, lat: f64, lng: f64) -> Self {
        self.center = Some((lat, lng));
        self
    }
}

impl<State, Action, C> ViewMarker for Map<State, Action, C>
where
    C: MapChild<State, Action>,
    C::ViewState: MapChildViewState,
{
}

pub struct MapViewState<State, Action, C>
where
    C: MapChild<State, Action>,
    C::ViewState: MapChildViewState,
{
    leaflet_map: leaflet::Map,
    children: Rc<RefCell<Vec<(C::Element, C::ViewState)>>>,
}

impl<State, Action, C> View<State, Action, ViewCtx, DynMessage> for Map<State, Action, C>
where
    State: 'static,
    Action: 'static,
    C: MapChild<State, Action>,
    C::ViewState: MapChildViewState,
{
    type Element = Pod<web_sys::HtmlElement, ElementProps>;

    type ViewState = MapViewState<State, Action, C>;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        // TODO:
        // Is there a way to just use `html::div(())`?
        let mut pod = Pod::new_element(vec![], HTML_NS, "div");
        pod.props
            .styles()
            .set_style("width".into(), Some("100%".into()));
        pod.props
            .styles()
            .set_style("height".into(), Some("100%".into()));
        pod.node.apply_props(&mut pod.props);

        let map_options = leaflet::MapOptions::default();
        let html_el = pod.node.unchecked_ref::<web_sys::HtmlElement>();
        let leaflet_map = leaflet::Map::new_with_element(html_el, &map_options);

        let children = Rc::new(RefCell::new(
            self.children
                .iter()
                .map(|c| c.build(ctx))
                .collect::<Vec<_>>(),
        ));

        let view_state = MapViewState {
            leaflet_map,
            children,
        };

        let children = Rc::clone(&view_state.children);
        let leaflet_map = view_state.leaflet_map.clone();
        let zoom = self.zoom;
        let center = self.center;
        // We have to postpone the map initiation
        // because the DOM element has been created at this point in time
        // but has not yet been mounted.
        spawn_local(async move {
            apply_zoom_and_center(&leaflet_map, zoom, center);
            children.borrow_mut().iter_mut().for_each(
                |(_, child_view_state): &mut (_, C::ViewState)| {
                    child_view_state.after_build(&leaflet_map);
                },
            );
        });
        (pod.into(), view_state)
    }

    fn rebuild<'el>(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        _: &mut ViewCtx,
        element: Mut<'el, Self::Element>,
    ) -> Mut<'el, Self::Element> {
        if prev.zoom != self.zoom || prev.center != self.center {
            apply_zoom_and_center(&view_state.leaflet_map, self.zoom, self.center);
        }
        view_state.children.borrow_mut().iter_mut().for_each(
            |(_, child_view_state): &mut (_, C::ViewState)| {
                child_view_state.after_rebuild(&view_state.leaflet_map);
            },
        );
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

fn apply_zoom_and_center(map: &leaflet::Map, zoom: Option<f64>, center: Option<(f64, f64)>) {
    match (zoom, center) {
        (Some(zoom), None) => {
            map.set_zoom(zoom);
        }
        (Some(zoom), Some((lat, lng))) => {
            let center = leaflet::LatLng::new(lat, lng);
            map.set_view(&center, zoom);
        }
        _ => {}
    }
}
