use yew::prelude::*;
use crate::graph_render;
use crate::data::*;
use crate::data_source;
use std::future::Future;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

pub struct Graph {
    canvas_ref : NodeRef,
    data : Option<Result<DataSet, String>>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    GotSeriesData(Result<DataSet, String>)
}

impl Component for Graph {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Graph {
            canvas_ref: NodeRef::default(),
            data: None,
            link: link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotSeriesData(res) => {
                self.data = Some(res);
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
            <div>
                <canvas width=600 height=400 ref=self.canvas_ref.clone()>
                </canvas>
                {countries_selector}
            </div>
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        if self.data.is_none() {
            // Construct a future to retrieve series data.
            let fut = async {
                Msg::GotSeriesData(data_source::query().await)
            };
            send_future(&self.link, fut);
        }

        let canvas_opt: Option<HtmlCanvasElement> = self.canvas_ref.cast::<HtmlCanvasElement>();
        if let Some(canvas) = canvas_opt {
            graph_render::draw(canvas);
        }
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

    let mut link = link.clone();
    let js_future = async move {
        link.send_message(future.await);
        Ok(JsValue::NULL)
    };

    future_to_promise(js_future);
}
