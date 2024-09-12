#[derive(Clone)]
pub enum Formula {
    ConstantProduct,
    ConcentratedLiquidity,
    DynamicLiquidity
}

pub trait SwapSimulator {
}