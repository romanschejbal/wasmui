use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub struct RequestIdleCallback {
    callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl RequestIdleCallback {
    pub fn new(mut fun: Box<dyn FnMut()>) -> Self {
        let f: Rc<RefCell<Option<Closure<_>>>> = Rc::new(RefCell::new(None));
        let g = Rc::clone(&f);

        fun();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // fun();
            // let window = web_sys::window().unwrap();
            // window
            //     .request_idle_callback(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            //     .unwrap();
        }) as Box<dyn FnMut()>));

        Self { callback: g }
    }

    pub fn start(&self) {
        let window = web_sys::window().unwrap();
        window
            .request_idle_callback(
                self.callback
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }
}
