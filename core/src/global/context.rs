use nyanko::common::Param;

#[derive(Clone, Copy)]
pub struct GlobalContext<'a> {
    pub param: &'a Param,
}