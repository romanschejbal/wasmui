pub mod types {

    use web_sys::Element;

    pub struct Fiber {
        tag: WorkTag,
    }

    impl Fiber {
        pub fn new(tag: WorkTag) -> Fiber {
            Fiber { tag }
        }
    }

    pub struct FiberRoot {
        pub container: Element,
        pub current: Option<Fiber>,
    }

    pub enum WorkTag {
        FunctionComponent,
        HostRoot,
    }
}
