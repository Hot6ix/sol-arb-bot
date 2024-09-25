#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub enum Formula {
    #[default]
    UnknownFormula,
    ConstantProduct,
    ConcentratedLiquidity,
    DynamicLiquidity,
    OpenBook
}

pub trait SwapSimulator {
}