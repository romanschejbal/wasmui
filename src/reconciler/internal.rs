pub mod types {

    use std::{cell::RefCell, rc::Rc};
    use web_sys::Element;

    #[derive(Debug)]
    pub struct Fiber {
        key: Option<usize>,
        work_tag: WorkTag,
    }

    impl Fiber {
        pub fn new(work_tag: WorkTag, key: Option<usize>) -> Fiber {
            Fiber { key, work_tag }
        }
    }

    pub struct FiberRoot {
        pub container: Rc<Element>,
        pub current: Rc<RefCell<Option<Fiber>>>,
        pub wip: Rc<RefCell<Option<Fiber>>>,
    }

    #[derive(Debug)]
    pub enum WorkTag {
        FunctionComponent,
        HostRoot,
    }

    pub enum EffectTag {
        Insert,
        Update,
        Delete,
    }
}
