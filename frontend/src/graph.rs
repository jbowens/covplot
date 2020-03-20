use yew::prelude::*;
use crate::graph_render;
use crate::data::*;
use crate::data_source;
use std::future::Future;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use plotters::style::Color;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub enum Msg {
    GotSeriesData(Result<DataSet, String>),
    ToggleRegion(Region),
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
            },
            Msg::ToggleRegion(region) => {
                if self.selected.contains(&region) {
                    // don't allow removing the last country
                    if self.selected.len() > 1 {
                        self.selected.retain(|r| region != *r);
                    }
                } else {
                    self.selected.push(region.clone());
                };
                self.redraw_canvas();
            },
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
                    .map(|r| {
                        let curr = Region{country: r.0.clone(), state: "".to_string()};
                        let styling = match self.selected.contains(&curr) {
                            true => (curr.color().rgb(), "selected"),
                            false => ((204, 204, 204), ""),
                        };

                        // TODO: stop cloning regions every which way
                        let reg = Region::new(&r.0.clone(), "");

                        html!{
                            <li
                                class=styling.1
                                style={format!("color: rgb({},{},{});", (styling.0).0, (styling.0).1, (styling.0).2)}
                                onclick=self.link.callback(move |_| Msg::ToggleRegion(reg.clone()))>
                                {format!("{}", r.0)}
                            </li>
                        }
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
                    <div class="source">
                        {"Source: Data is pulled from John Hopkins University CSSE: "}<a href="https://github.com/CSSEGISandData/COVID-19" target="_blank">{"github.com/CSSEGISandData/COVID-19"}</a>
                        {". Data is downloaded directly from the JHU CSSE GitHub repository on page load. Currently, only countries with at least 100 confirmed cases are shown."}
                    </div>
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

