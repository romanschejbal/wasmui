mod react;
mod utils;

use macros::component;
use react::FunctionComponentTrait;
use react::ReactNodeList::*;
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
fn Header_(title: &str) {
    Text(title);
}

#[derive(Debug)]
struct Header<'a> {
    title: &'a str,
}

impl<'a> react::FunctionComponentTrait for Header<'a> {
    fn render(&self) -> react::ReactNodeList {
        List(vec![Text("You said: "), Text(self.title)])
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    utils::set_panic_hook();
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let root = react::create_root(body.into());
    root.render(&Host(
        "div",
        Some(Box::new(List(vec![
            Host("span", Some(Box::new(Text("Hello")))),
            Host("span", Some(Box::new(Text("World")))),
            FunctionComponent(Box::new(Header { title: "Hergooot" })),
        ]))),
    ));
    Ok(())
}
