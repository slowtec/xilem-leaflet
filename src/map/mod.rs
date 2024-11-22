use std::marker::PhantomData;

use wasm_bindgen_futures::spawn_local;
use xilem_web::{
    core::{
        frozen, AppendVec, ElementSplice, MessageResult, Mut, SuperElement, View, ViewElement,
        ViewId, ViewMarker, ViewPathTracker, ViewSequence,
    },
    elements::html,
    interfaces::{Element, HtmlElement},
    modifiers::style,
    DynMessage, ViewCtx,
};

mod events;
pub use self::events::*;

pub struct MapCtx {
    dom_ctx: ViewCtx,
    map: leaflet::Map,
}

impl MapCtx {
    fn new(dom_ctx: ViewCtx, map: leaflet::Map) -> Self {
        Self { dom_ctx, map }
    }
    pub const fn map(&self) -> &leaflet::Map {
        &self.map
    }
}

impl ViewPathTracker for MapCtx {
    fn push_id(&mut self, id: ViewId) {
        self.dom_ctx.push_id(id);
    }

    fn pop_id(&mut self) {
        self.dom_ctx.pop_id();
    }

    fn view_path(&mut self) -> &[ViewId] {
        self.dom_ctx.view_path()
    }
}

pub trait MapChildren<State, Action>:
    ViewSequence<State, Action, MapCtx, MapChildElement, DynMessage>
{
}

impl<V, State, Action> MapChildren<State, Action> for V where
    V: ViewSequence<State, Action, MapCtx, MapChildElement, DynMessage>
{
}

pub fn map<State, Action, Children>(
    children: Children,
) -> Map<impl HtmlElement<State, Action>, State, Action, Children>
where
    State: 'static,
    Action: 'static,
    Children: MapChildren<State, Action>,
{
    let map_view =
        frozen(|| html::div(()).style([style("width", "100%"), style("height", "100%")]));
    Map {
        map_view,
        zoom: None,
        center: None,
        children,
        phantom: PhantomData,
    }
}

#[derive(Debug)]
pub enum MapChildElement {
    Marker(leaflet::Marker),
    TileLayer(leaflet::TileLayer),
    Event,
}

impl MapChildElement {
    /// # Panics
    ///
    /// If it's not a marker.
    pub fn as_marker_mut(&mut self) -> &mut leaflet::Marker {
        match self {
            MapChildElement::Marker(marker) => marker,
            _ => panic!("Element is not a marker"),
        }
    }
    /// # Panics
    ///
    /// If it's not a tile layer.
    pub fn as_tile_layer_mut(&mut self) -> &mut leaflet::TileLayer {
        match self {
            MapChildElement::TileLayer(layer) => layer,
            _ => panic!("Element is not a marker"),
        }
    }
}

impl ViewElement for MapChildElement {
    type Mut<'a> = &'a mut MapChildElement;
}

// Necessary for the `ViewSequence`
// This could theoretically cast more specific elements into a (type-erased) `AnyMapChildElement` if necessary.
impl SuperElement<MapChildElement, MapCtx> for MapChildElement {
    fn upcast(_: &mut MapCtx, child: MapChildElement) -> Self {
        child
    }

    fn with_downcast_val<R>(
        this: Mut<'_, Self>,
        f: impl FnOnce(Mut<'_, MapChildElement>) -> R,
    ) -> (Self::Mut<'_>, R) {
        let r = f(this);
        (this, r)
    }
}

struct MapChildrenSplice<'a> {
    idx: usize,
    children: &'a mut Vec<MapChildElement>,
}

impl<'a> MapChildrenSplice<'a> {
    pub(crate) fn new(children: &'a mut Vec<MapChildElement>) -> Self {
        Self { idx: 0, children }
    }
}

// Hmm I wonder if we could provide some generic types for this, to make this simpler,
// we could e.g. export `VecSplice` from xilem_web (again),
// and implement that trait for `VecSplice`, when it's just necessary to maintain a Vec of elements...
// The current naive implementation is rather inefficient O(n^2) (resulting in a lot of shifting of elements and allocations)
impl ElementSplice<MapChildElement> for MapChildrenSplice<'_> {
    fn with_scratch<R>(&mut self, f: impl FnOnce(&mut AppendVec<MapChildElement>) -> R) -> R {
        let mut scratch = AppendVec::default();
        let ret_val = f(&mut scratch);
        let new_elements = scratch.into_inner();
        let len = new_elements.len();
        self.children.splice(self.idx..self.idx, new_elements);
        self.idx += len;
        ret_val
    }

    fn insert(&mut self, element: MapChildElement) {
        self.children.insert(self.idx, element);
        self.idx += 1;
    }

    fn mutate<R>(&mut self, f: impl FnOnce(Mut<'_, MapChildElement>) -> R) -> R {
        self.idx += 1;
        f(&mut self.children[self.idx - 1])
    }

    fn skip(&mut self, n: usize) {
        self.idx += n;
    }

    fn delete<R>(&mut self, f: impl FnOnce(Mut<'_, MapChildElement>) -> R) -> R {
        f(&mut self.children.remove(self.idx))
    }
}

/// This is a marker trait that is used to pass only map-specific children
/// to the [`map`] function.
pub trait MapChild<State, Action>:
    View<State, Action, ViewCtx, DynMessage, Element = MapChildElement>
{
}

impl<V, State, Action> MapChild<State, Action> for V where
    V: View<State, Action, ViewCtx, DynMessage, Element = MapChildElement>
{
}

pub struct Map<MapDomView, State, Action, Children> {
    map_view: MapDomView,
    children: Children,
    zoom: Option<f64>,
    center: Option<(f64, f64)>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<MapDomView, State, Action, Children> Map<MapDomView, State, Action, Children> {
    pub fn zoom(mut self, value: f64) -> Self {
        self.zoom = Some(value);
        self
    }

    pub fn on_zoom_end<F>(
        self,
        callback: F,
    ) -> Map<MapDomView, State, Action, (Children, OnZoomEnd<F>)>
    where
        F: Fn(&mut State, leaflet::Map, leaflet::Event) + 'static,
    {
        let Self {
            map_view,
            children,
            zoom,
            center,
            phantom,
        } = self;
        let children = (children, on_zoom_end(callback));
        Map {
            map_view,
            children,
            zoom,
            center,
            phantom,
        }
    }

    pub fn on_move_end<F>(
        self,
        callback: F,
    ) -> Map<MapDomView, State, Action, (Children, OnMoveEnd<F>)>
    where
        F: Fn(&mut State, leaflet::Map, leaflet::Event) + 'static,
    {
        let Self {
            map_view,
            children,
            zoom,
            center,
            phantom,
        } = self;
        let children = (children, on_move_end(callback));
        Map {
            map_view,
            children,
            zoom,
            center,
            phantom,
        }
    }

    pub fn on_mouse_click<F>(
        self,
        callback: F,
    ) -> Map<MapDomView, State, Action, (Children, OnMouseClick<F>)>
    where
        F: Fn(&mut State, leaflet::MouseEvent) + 'static,
    {
        let Self {
            map_view,
            children,
            zoom,
            center,
            phantom,
        } = self;
        let children = (children, on_mouse_click(callback));
        Map {
            map_view,
            children,
            zoom,
            center,
            phantom,
        }
    }

    pub fn center(mut self, lat: f64, lng: f64) -> Self {
        self.center = Some((lat, lng));
        self
    }
}

impl<Styles, State, Action, Children> ViewMarker for Map<Styles, State, Action, Children> {}

pub struct MapViewState<DS, CS> {
    map_dom_state: DS,
    children_state: CS,
    children: Vec<MapChildElement>,
    leaflet_map: leaflet::Map,
}

#[derive(Debug, Clone)]
pub enum MapMessage {
    InitMap,
}

impl<MapDomView, State, Action, Children> View<State, Action, ViewCtx, DynMessage>
    for Map<MapDomView, State, Action, Children>
where
    State: 'static,
    Action: 'static,
    MapDomView: HtmlElement<State, Action>,
    Children: MapChildren<State, Action>,
{
    type Element = MapDomView::Element;

    type ViewState = MapViewState<MapDomView::ViewState, Children::SeqState>;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let (map_dom_element, map_dom_state) = self.map_view.build(ctx);

        let map_options = leaflet::MapOptions::default();
        let leaflet_map =
            leaflet::Map::new_with_element(map_dom_element.node.as_ref(), &map_options);

        let mut elements = AppendVec::default();
        let view_state = ctx.as_owned(|dom_ctx| {
            let mut map_ctx = MapCtx::new(dom_ctx, leaflet_map.clone());
            let children_state = self.children.seq_build(&mut map_ctx, &mut elements);
            let view_state = MapViewState {
                leaflet_map: map_ctx.map,
                map_dom_state,
                children: elements.into_inner(),
                children_state,
            };
            (map_ctx.dom_ctx, view_state)
        });

        // We have to postpone the map initiation
        // because the DOM element has been created at this point in time
        // but has not yet been mounted.
        {
            let map = view_state.leaflet_map.clone();
            let zoom = self.zoom;
            let center = self.center;
            spawn_local(async move { apply_zoom_and_center(&map, zoom, center) });
        }

        (map_dom_element, view_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<Self::Element>,
    ) {
        self.map_view
            .rebuild(&prev.map_view, &mut view_state.map_dom_state, ctx, element);
        if prev.zoom != self.zoom || prev.center != self.center {
            apply_zoom_and_center(&view_state.leaflet_map, self.zoom, self.center);
        }
        ctx.as_owned(|dom_ctx| {
            let mut map_ctx = MapCtx::new(dom_ctx, view_state.leaflet_map.clone());
            self.children.seq_rebuild(
                &prev.children,
                &mut view_state.children_state,
                &mut map_ctx,
                &mut MapChildrenSplice::new(&mut view_state.children),
            );
            (map_ctx.dom_ctx, ())
        });
    }

    fn teardown(&self, view_state: &mut Self::ViewState, ctx: &mut ViewCtx, _: Mut<Self::Element>) {
        ctx.as_owned(|dom_ctx| {
            let mut map_ctx = MapCtx::new(dom_ctx, view_state.leaflet_map.clone());
            self.children.seq_teardown(
                &mut view_state.children_state,
                &mut map_ctx,
                &mut MapChildrenSplice::new(&mut view_state.children),
            );
            (map_ctx.dom_ctx, ())
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        id_path: &[ViewId],
        message: DynMessage,
        app_state: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        self.children
            .seq_message(&mut view_state.children_state, id_path, message, app_state)
    }
}

fn apply_zoom_and_center(map: &leaflet::Map, zoom: Option<f64>, center: Option<(f64, f64)>) {
    log::debug!("apply zoom ({zoom:?} and center ({center:?})");
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
