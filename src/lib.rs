mod react;
mod utils;

use std::{collections::HashMap, rc::Rc};

use react::{EventListener, HostAttribute, ReactNodeList::*};
use react::{FunctionComponent, ReactNodeList};
use wasm_bindgen::prelude::*;
use web_sys::Element;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Debug)]
struct Button<'a> {
    title: &'a str,
}

impl<'a> FunctionComponent for Button<'a> {
    fn render(&self) -> ReactNodeList {
        let mut props = HashMap::new();
        let on_click: Box<dyn HostAttribute<Type = Element>> =
            Box::new(EventListener(Closure::wrap(Box::new(move || {
                log("HELLO FROM BUTTON")
            }))));
        props.insert("click", on_click);
        Host(
            "button",
            props,
            Some(Rc::new(Text(format!("CLICKED ME {} TIMES", 0)))),
        )
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    utils::set_panic_hook();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let mut root = react::create_root(body.into());
    root.render(Host(
        "div",
        HashMap::new(),
        Some(Rc::new(List(vec![
            Rc::new(Host(
                "div",
                HashMap::new(),
                Some(Rc::new(List(vec![Rc::new(Host(
                    "h1",
                    HashMap::new(),
                    Some(Rc::new(Text("Hello World".into()))),
                ))]))),
            )),
            Rc::new(Host(
                "p",
                HashMap::new(),
                Some(Rc::new(Text("From React in WASM".into()))),
            )),
            Rc::new(FunctionComponent(Box::new(Button { title: "Click me!" }))),
        ]))),
    ));
    Ok(())
}
