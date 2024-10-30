use std::marker::PhantomData;
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use web_sys::wasm_bindgen::UnwrapThrowExt;
use xilem_web::{
    core::{
        frozen, AppendVec, ElementSplice, MessageResult, Mut, SuperElement, View, ViewElement,
        ViewId, ViewMarker, ViewPathTracker, ViewSequence,
    },
    elements::html,
    interfaces::{Element, HtmlElement},
    modifiers::style,
    DynMessage, MessageThunk, ViewCtx,
};

mod events;
pub use self::events::*;

pub struct MapCtx {
    id_path: Vec<ViewId>,
    map: leaflet::Map,
    thunk: Rc<MessageThunk>,
}

impl MapCtx {
    fn new(map: leaflet::Map, thunk: MessageThunk) -> Self {
        Self {
            id_path: Vec::new(),
            map,
            thunk: Rc::new(thunk),
        }
    }
    pub const fn map(&self) -> &leaflet::Map {
        &self.map
    }
}

impl ViewPathTracker for MapCtx {
    fn push_id(&mut self, id: ViewId) {
        self.id_path.push(id);
    }

    fn pop_id(&mut self) {
        self.id_path.pop();
    }

    fn view_path(&mut self) -> &[ViewId] {
        &self.id_path
    }
}

// I think Action could probably be used to signify `Map` some changes, it could match

pub enum MapAction {}

pub trait MapChildren<State>:
    ViewSequence<State, MapAction, MapCtx, MapChildElement, MapMessage>
{
}

impl<V, State> MapChildren<State> for V where
    V: ViewSequence<State, MapAction, MapCtx, MapChildElement, MapMessage>
{
}

pub fn map<State, Action, Children>(
    children: Children,
) -> Map<impl HtmlElement<State, Action>, State, Action, Children>
where
    State: 'static,
    Action: 'static,
    Children: MapChildren<State>,
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

// TODO make this actually useful with element state
pub struct MapChildElement;

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

struct MapChildrenSplice;

/// Hmm I wonder if we could provide some generic types for this, to make this simpler,
/// we could e.g. export `VecSplice` from xilem_web (again),
/// and implement that trait for `VecSplice`, when it's just necessary to maintain a Vec of elements...
impl ElementSplice<MapChildElement> for MapChildrenSplice {
    fn with_scratch<R>(&mut self, f: impl FnOnce(&mut AppendVec<MapChildElement>) -> R) -> R {
        f(&mut AppendVec::default())
    }

    fn insert(&mut self, _element: MapChildElement) {}

    fn mutate<R>(&mut self, f: impl FnOnce(Mut<'_, MapChildElement>) -> R) -> R {
        f(&mut MapChildElement)
    }

    fn skip(&mut self, _n: usize) {}

    fn delete<R>(&mut self, f: impl FnOnce(Mut<'_, MapChildElement>) -> R) -> R {
        f(&mut MapChildElement)
    }
}

/// This is a marker trait that is used to pass only map-specific children
/// to the [`map`] function.
pub trait MapChild<State, Action>:
    View<State, Action, ViewCtx, MapMessage, Element = MapChildElement>
{
}

impl<V, State, Action> MapChild<State, Action> for V where
    V: View<State, Action, ViewCtx, MapMessage, Element = MapChildElement>
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
    ) -> Map<MapDomView, State, Action, (Children, OnZoomEnd<State, F>)>
    where
        F: Fn(&mut State, f64) + 'static,
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

    pub fn on_mouse_click<F>(
        self,
        callback: F,
    ) -> Map<MapDomView, State, Action, (Children, OnMouseClick<State, F>)>
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
    map_ctx: MapCtx,
}

#[derive(Debug, Clone)]
pub enum MapMessage {
    InitMap,
    ZoomEnd(f64),                    // TODO: remove this
    MouseClick(leaflet::MouseEvent), // TODO: remove this
}

/// Distinctive ID for better debugging
const MAP_VIEW_ID: ViewId = ViewId::new(1236068);

impl<MapDomView, State, Action, Children> View<State, Action, ViewCtx, DynMessage>
    for Map<MapDomView, State, Action, Children>
where
    State: 'static,
    Action: 'static,
    MapDomView: HtmlElement<State, Action>,
    Children: MapChildren<State>,
{
    type Element = MapDomView::Element;

    type ViewState = MapViewState<MapDomView::ViewState, Children::SeqState>;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        ctx.with_id(MAP_VIEW_ID, |ctx| {
            let (map_dom_element, map_dom_state) = self.map_view.build(ctx);

            let map_options = leaflet::MapOptions::default();
            let leaflet_map =
                leaflet::Map::new_with_element(map_dom_element.node.as_ref(), &map_options);

            let mut elements = AppendVec::default();
            let mut map_ctx = MapCtx::new(leaflet_map.clone(), ctx.message_thunk());
            let children_state = self.children.seq_build(&mut map_ctx, &mut elements);

            let view_state = MapViewState {
                map_ctx,
                map_dom_state,
                children_state,
            };

            // Is the following an issue?
            // Does it require on being in the DOM tree?
            // Can the tile-layer be added before running `apply_zoom_and_center`?

            // We have to postpone the map initiation
            // because the DOM element has been created at this point in time
            // but has not yet been mounted.
            let thunk = ctx.message_thunk();
            spawn_local(async move {
                thunk.push_message(MapMessage::InitMap);
            });
            (map_dom_element, view_state)
        })
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<Self::Element>,
    ) {
        log::debug!("Rebuild map");
        ctx.with_id(MAP_VIEW_ID, |ctx| {
            self.map_view
                .rebuild(&prev.map_view, &mut view_state.map_dom_state, ctx, element);
            if prev.zoom != self.zoom || prev.center != self.center {
                apply_zoom_and_center(&view_state.map_ctx.map, self.zoom, self.center);
            }
            let mut splice = MapChildrenSplice;
            log::debug!("seq_rebuild children");
            self.children.seq_rebuild(
                &prev.children,
                &mut view_state.children_state,
                &mut view_state.map_ctx,
                &mut splice,
            );
        })
    }

    fn teardown(&self, _: &mut Self::ViewState, ctx: &mut ViewCtx, _: Mut<Self::Element>) {
        ctx.with_id(MAP_VIEW_ID, |_ctx| {
            // TODO
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        path: &[ViewId],
        message: DynMessage,
        _: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        log::debug!("Handle map message {message:?} for {path:?}");
        let (first, _) = path.split_first().unwrap_throw();
        assert_eq!(*first, MAP_VIEW_ID);
        let message = *message.downcast().unwrap();
        match message {
            MapMessage::InitMap => {
                apply_zoom_and_center(&view_state.map_ctx.map, self.zoom, self.center);
                MessageResult::RequestRebuild
            }
            _ => {
                // TODO: handle message
                MessageResult::Nop
            }
        }
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
