pub type Path = Vec<Seg>;

#[derive(Debug, Clone)]
pub enum Seg {
    Not,
    IfCond,
    IfCons,
    ChainN(i32),
    RecordRef,
    ForeachList,
    ForeachBody,
}
