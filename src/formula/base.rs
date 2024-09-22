#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub enum Formula {
    #[default]
    UnknownFormula,
    ConstantProduct,
    ConcentratedLiquidity,
    DynamicLiquidity
}

pub trait SwapSimulator {
}