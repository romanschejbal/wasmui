pub mod internal;

use super::shared::ReactNodeList;
use internal::types::{Fiber, FiberRoot, WorkTag};
use std::{cell::RefCell, rc::Rc};
use web_sys::Element;

pub fn create_root_fiber(container: Element) -> FiberRoot {
    let current = Fiber::new(WorkTag::HostRoot, None);

    FiberRoot {
        container: Rc::new(container),
        current: Rc::new(RefCell::new(None)),
        wip: Rc::new(RefCell::new(Some(current))),
    }
}

pub fn create_fiber(element: ReactNodeList) -> Fiber {
    Fiber::new(
        match element {
            ReactNodeList::FunctionComponent => WorkTag::FunctionComponent,
            _ => WorkTag::HostRoot,
        },
        None,
    )
}

// fn perform_unit_of_work(unit: &mut Fiber) -> Option<&mut Fiber> {
//     Some(unit)
// }
