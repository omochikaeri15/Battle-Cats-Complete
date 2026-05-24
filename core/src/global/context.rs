use crate::global::game::param::Param;

#[derive(Clone, Copy)]
pub struct GlobalContext<'a> {
    pub param: &'a Param,
}