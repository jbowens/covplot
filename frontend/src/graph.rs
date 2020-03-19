use yew::prelude::*;
use crate::graph_render;
use crate::data::*;
use crate::data_source;
use std::future::Future;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub enum Msg {
    GotSeriesData(Result<DataSet, String>)
}

pub struct Graph {
    canvas_ref : NodeRef,
    data : Option<Result<DataSet, String>>,
    selected : Vec<Region>,
    link: ComponentLink<Self>,
}

impl Graph {
    fn redraw_canvas(&self) {
        match &self.data {
            None => {
                // Construct a future to retrieve series data.
                let fut = async {
                    Msg::GotSeriesData(data_source::query().await)
                };
                send_future(&self.link, fut);
            },
            Some(Ok(data_set)) => {
                let canvas_opt: Option<HtmlCanvasElement> = self.canvas_ref.cast::<HtmlCanvasElement>();
                if let Some(canvas) = canvas_opt {
                    let selected = data_set.select(&self.selected);
                    graph_render::draw(&data_set, selected, canvas);
                }
            },
            Some(Err(e)) => {
                log!("error: {}", e);
            },
        }
    }
}

impl Component for Graph {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Graph {
            canvas_ref: NodeRef::default(),
            data: None,
            selected: vec![
                Region::new("US", ""),
                Region::new("Italy", ""),
                Region::new("China", ""),
                Region::new("Spain", ""),
            ],
            link: link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotSeriesData(res) => {
                self.data = Some(res);
                self.redraw_canvas();
            }
        }
        true
    }

    fn view(&self) -> Html {
        let countries_selector = match &self.data {
            None => html!{<div>{"Loading..."}</div>},
            Some(Ok(data_set)) => html!{
                <ul id="regions">
                {for data_set
                    .regions
                    .iter()
                    .map(|s| html!{
                        <li>{format!("{}", s.0)}</li>
                    })
                }
                </ul>
            },
            Some(Err(e)) => html!{<div>{format!("Error: {}", e)}</div>},
        };

        html! {
            <div id="graph">
                {countries_selector}
                <div id="canvas-container">
                    <canvas width=1200 height=800 ref=self.canvas_ref.clone()>
                    </canvas>
                </div>
            </div>
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        self.redraw_canvas();
        false
    }
 }

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi"), not(cargo_web)))]
/// This method processes a Future that returns a message and sends it back to the component's
/// loop.
///
/// # Panics
/// If the future panics, then the promise will not resolve, and will leak.
pub fn send_future<COMP: Component, F>(link: &ComponentLink<COMP>, future: F)
where
    F: Future<Output = COMP::Message> + 'static,
{
    use wasm_bindgen_futures::future_to_promise;

    let link = link.clone();
    let js_future = async move {
        link.send_message(future.await);
        Ok(JsValue::NULL)
    };

    future_to_promise(js_future);
}

