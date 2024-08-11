use std::marker::PhantomData;

use wasm_bindgen_futures::spawn_local;
use web_sys::wasm_bindgen::UnwrapThrowExt;
use xilem_web::{
    core::{
        AppendVec, ElementSplice, MessageResult, Mut, SuperElement, View, ViewElement, ViewId,
        ViewMarker, ViewPathTracker, ViewSequence,
    },
    elements::html,
    interfaces::HtmlElement,
    style, DynMessage, Style, ViewCtx,
};

mod events;
pub use self::events::*;

// I think Action could probably be used to signify `Map` some changes, it could match

pub enum MapAction {}

pub trait MapChildren<State>:
    ViewSequence<State, MapAction, ViewCtx, MapChildElement, MapMessage>
{
}

impl<V, State> MapChildren<State> for V where
    V: ViewSequence<State, MapAction, ViewCtx, MapChildElement, MapMessage>
{
}

type MapDomView<T, A> = Style<html::Div<T, A>, T, A>;

pub fn map<State, Action, Children>(children: Children) -> Map<State, Action, Children>
where
    State: 'static,
    Action: 'static,
    Children: MapChildren<State>,
{
    Map {
        // A Frozen view would be even better (perf), but that basically requires TAITs
        // alternatively the view could be created in `View::build` as well (and stored in `View::State`)
        map_view: html::div(()).style([style("width", "100%"), style("height", "100%")]),
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
impl SuperElement<MapChildElement> for MapChildElement {
    fn upcast(child: MapChildElement) -> Self {
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

pub struct Map<State, Action, Children> {
    map_view: MapDomView<State, Action>,
    children: Children,
    zoom: Option<f64>,
    center: Option<(f64, f64)>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action, Children> Map<State, Action, Children> {
    pub fn zoom(mut self, value: f64) -> Self {
        self.zoom = Some(value);
        self
    }

    pub fn on_zoom_end<F>(&self, callback: F) -> OnZoomEnd<State, F>
    where
        F: Fn(&mut State, f64) + 'static,
    {
        on_zoom_end(callback)
    }

    pub fn center(mut self, lat: f64, lng: f64) -> Self {
        self.center = Some((lat, lng));
        self
    }
}

impl<State, Action, Children> ViewMarker for Map<State, Action, Children> {}

pub struct MapViewState<DS, CS> {
    map_dom_state: DS,
    children_state: CS,
    leaflet_map: leaflet::Map,
}

#[derive(Debug, Clone)]
pub enum MapMessage {
    InitMap,
    MapHasMounted(leaflet::Map),
    ZoomEnd(f64),
}

/// Distinctive id for better debugging
const MAP_VIEW_ID: ViewId = ViewId::new(1236068);

impl<State, Action, Children> View<State, Action, ViewCtx, DynMessage>
    for Map<State, Action, Children>
where
    State: 'static,
    Action: 'static,
    Children: MapChildren<State>,
{
    type Element = <MapDomView<State, Action> as View<State, Action, ViewCtx, DynMessage>>::Element;

    type ViewState = MapViewState<
        <MapDomView<State, Action> as View<State, Action, ViewCtx, DynMessage>>::ViewState,
        Children::SeqState,
    >;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        ctx.with_id(MAP_VIEW_ID, |ctx| {
            let (map_dom_element, map_dom_state) = self.map_view.build(ctx);

            let map_options = leaflet::MapOptions::default();
            let leaflet_map = leaflet::Map::new_with_element(&map_dom_element.node, &map_options);

            let mut elements = AppendVec::default();
            let children_state = self.children.seq_build(ctx, &mut elements);

            let view_state = MapViewState {
                leaflet_map,
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

    fn rebuild<'el>(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'el, Self::Element>,
    ) -> Mut<'el, Self::Element> {
        ctx.with_id(MAP_VIEW_ID, |ctx| {
            let element =
                self.map_view
                    .rebuild(&prev.map_view, &mut view_state.map_dom_state, ctx, element);
            if prev.zoom != self.zoom || prev.center != self.center {
                apply_zoom_and_center(&view_state.leaflet_map, self.zoom, self.center);
            }
            let mut splice = MapChildrenSplice;
            self.children.seq_rebuild(
                &prev.children,
                &mut view_state.children_state,
                ctx,
                &mut splice,
            );
            element
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
        app_state: &mut State,
    ) -> MessageResult<Action, DynMessage> {
        log::debug!("Handle map message {message:?}");
        let (first, rest) = path.split_first().unwrap_throw();
        assert_eq!(*first, MAP_VIEW_ID);
        let message = *message.downcast().unwrap();
        match message {
            // if the message itself is not for this view, it could just be redirected with the path to a child
            MapMessage::InitMap => {
                apply_zoom_and_center(&view_state.leaflet_map, self.zoom, self.center);
                let children_message = self.children.seq_message(
                    &mut view_state.children_state,
                    rest,
                    MapMessage::MapHasMounted(view_state.leaflet_map.clone()),
                    app_state,
                );
                match children_message {
                    #[allow(unreachable_code)]
                    // Could do something with the action message
                    MessageResult::Action(action) => MessageResult::Action(match action {}),
                    MessageResult::RequestRebuild => MessageResult::RequestRebuild,
                    MessageResult::Nop => MessageResult::Nop,
                    MessageResult::Stale(_) => MessageResult::Stale(Box::new(())),
                }
            }
            _ => MessageResult::Nop,
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
