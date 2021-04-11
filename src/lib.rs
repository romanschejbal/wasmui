mod react;
mod utils;

use std::{cell::RefCell, rc::Rc};

use macros::component;
use react::ReactNodeList::*;
use react::{FunctionComponentTrait, ReactNodeList};
use wasm_bindgen::prelude::*;

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
struct Header<'a> {
    title: &'a str,
}

impl<'a> react::FunctionComponentTrait for Header<'a> {
    fn render(&self) -> react::ReactNodeList {
        List(vec![
            Rc::new(Text("You said: ".into())),
            Rc::new(Text(self.title.into())),
            Rc::new(Host(
                "div",
                Some(Rc::new(FunctionComponent(Box::new(Tail {
                    title: "TAIL!",
                })))),
            )),
        ])
    }
}

#[derive(Debug)]
struct Tail<'a> {
    title: &'a str,
}

impl<'a> react::FunctionComponentTrait for Tail<'a> {
    fn render(&self) -> react::ReactNodeList {
        List(vec![
            Rc::new(Text("I did say that yeah".into())),
            Rc::new(Text(self.title.into())),
        ])
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
        Some(Rc::new(List(vec![
            Rc::new(Host(
                "div",
                Some(Rc::new(List(vec![Rc::new(Host(
                    "h1",
                    Some(Rc::new(Text("Hello World".into()))),
                ))]))),
            )),
            Rc::new(Host("p", Some(Rc::new(Text("From React in WASM".into()))))),
            Rc::new(FunctionComponent(Box::new(Header { title: "Hergooot" }))),
        ]))),
    ));
    Ok(())
}
