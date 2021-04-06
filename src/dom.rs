use super::{
    shared::{ReactNodeList, RootType},
    utils::RequestIdleCallback,
};
use crate::{log, reconciler};
use std::rc::Rc;
use web_sys::Element;

struct ReactDOMRoot {
    root: Rc<reconciler::internal::types::FiberRoot>,
}

impl RootType for ReactDOMRoot {
    fn render(&self, children: ReactNodeList) {
        let root = self.root.clone();
        let ric = RequestIdleCallback::new(Box::new(move || {
            log(&format!("Current {:?}", root.current));
        }));
        ric.start();
    }
}

pub fn create_root(container: Element) -> Box<dyn RootType> {
    Box::new(ReactDOMRoot {
        root: Rc::new(reconciler::create_root_fiber(container)),
    })
}
