pub enum ReactNodeList {
    FunctionComponent,
    HostElement(&'static str),
    List(Vec<ReactNodeList>),
}

pub trait RootType {
    fn render(&self, children: ReactNodeList) {}
}
