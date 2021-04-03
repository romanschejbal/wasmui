pub mod internal;

use super::shared::ReactNodeList;
use internal::types::{Fiber, FiberRoot, WorkTag};
use web_sys::Element;

pub fn create_fiber_root(container: Element) -> FiberRoot {
    let current = Fiber::new(WorkTag::HostRoot);

    FiberRoot {
        container,
        current: Some(current),
    }
}

pub fn update_container(children: ReactNodeList, container: &mut FiberRoot) {}
