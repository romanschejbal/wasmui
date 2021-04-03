use crate::reconciler::update_container;

use super::reconciler::{create_fiber_root, internal::types::FiberRoot};
use super::shared::{ReactNodeList, RootType};
use web_sys::Element;

struct ReactDOMRoot {
    root: FiberRoot,
}

impl RootType for ReactDOMRoot {
    fn render(&mut self, children: ReactNodeList) {
        update_container(children, &mut self.root);
    }
}

pub fn create_root(container: Element) -> Box<dyn RootType> {
    Box::new(ReactDOMRoot {
        root: create_fiber_root(container),
    })
}
