mod dom;
mod react;
mod reconciler;
mod shared;
mod utils;

// use react::ReactNodeList::*;
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

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let mut root = react::create_root(body.into());
    // root.render(Element(
    //     "div",
    //     Some(Box::new(List(vec![Element(
    //         "span",
    //         Some(Box::new(Text("Ahoj"))),
    //     )]))),
    // ));
    Ok(())
}
