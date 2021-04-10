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

#[component]
fn Header(title: &str) {
    // let (state, setState) = hooks.useState(None);
    // ...
}

#[derive(Debug)]
struct Header<'a> {
    title: &'a str,
    _children: Option<ReactNodeList<'a>>,
}

impl<'a> react::FunctionComponentTrait for Header<'a> {
    // fn get_children(&self) -> &ReactNodeList {
    //     self._children
    //         .as_ref()
    //         .expect("No children set for component instance")
    // }

    // fn set_children(&mut self, children: ReactNodeList<'a>) {
    // self._children = Some(children);
    // }
    // fn init(&mut self) {
    // self._children = Some(self.render());
    // }

    fn render(&mut self) -> &react::ReactNodeList<'a> {
        self._children = Some(List(vec![Text("You said: "), Text(self.title)]));
        self._children.as_ref().unwrap()
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
        Some(Box::new(List(vec![
            Host(
                "div",
                Some(Box::new(List(vec![Host(
                    "h1",
                    Some(Box::new(Text("Hello World"))),
                )]))),
            ),
            Host("p", Some(Box::new(Text("From React in WASM")))),
            FunctionComponent(Box::new(Header {
                title: "Hergooot",
                _children: None,
            })),
        ]))),
    ));
    Ok(())
}
