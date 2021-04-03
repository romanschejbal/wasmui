pub type ReactNodeList = Vec<usize>;
pub trait RootType {
    fn render(&mut self, children: ReactNodeList) {}
}
