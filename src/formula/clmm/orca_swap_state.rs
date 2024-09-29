use solana_sdk::pubkey::Pubkey;
use crate::r#struct::pools::WhirlpoolRewardInfo;

pub const NUM_REWARDS: usize = 3;
pub const TICK_ARRAY_SIZE: i32 = 88;
pub const TICK_ARRAY_SIZE_USIZE: usize = 88;
pub const Q64_RESOLUTION: usize = 64;
pub const PROTOCOL_FEE_RATE_MUL_VALUE: u128 = 10_000;
pub const NO_EXPLICIT_SQRT_PRICE_LIMIT: u128 = 0u128;
pub const MAX_TICK_INDEX: i32 = 443636;
pub const MIN_TICK_INDEX: i32 = -443636;

#[derive(Debug)]
pub struct PostSwapUpdate {
    pub amount_a: u64,
    pub amount_b: u64,
    pub next_liquidity: u128,
    pub next_tick_index: i32,
    pub next_sqrt_price: u128,
    pub next_fee_growth_global: u128,
    pub next_reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS],
    pub next_protocol_fee: u64,
}

pub struct SwapTickSequence {
    arrays: Vec<ProxiedTickArray>,
}

impl SwapTickSequence {
    pub fn new(
        ta0: TickArray,
        ta1: Option<TickArray>,
        ta2: Option<TickArray>,
    ) -> Self {
        Self::new_with_proxy(
            ProxiedTickArray::new_initialized(ta0),
            ta1.map(ProxiedTickArray::new_initialized),
            ta2.map(ProxiedTickArray::new_initialized),
        )
    }

    pub(crate) fn new_with_proxy(
        ta0: ProxiedTickArray,
        ta1: Option<ProxiedTickArray>,
        ta2: Option<ProxiedTickArray>,
    ) -> Self {
        let mut vec = Vec::with_capacity(3);
        vec.push(ta0);
        if let Some(ta1) = ta1 {
            vec.push(ta1);
        }
        if let Some(ta2) = ta2 {
            vec.push(ta2);
        }
        Self { arrays: vec }
    }

    pub fn get_tick(
        &self,
        array_index: usize,
        tick_index: i32,
        tick_spacing: u16,
    ) -> Result<&Tick, &'static str> {
        let array = self.arrays.get(array_index);
        match array {
            Some(array) => array.get_tick(tick_index, tick_spacing),
            _ => Err("ErrorCode::TickArrayIndexOutOfBounds"),
        }
    }

    pub fn update_tick(
        &mut self,
        array_index: usize,
        tick_index: i32,
        tick_spacing: u16,
        update: &TickUpdate,
    ) -> Result<(), &'static str> {
        let array = self.arrays.get_mut(array_index);
        match array {
            Some(array) => {
                array.update_tick(tick_index, tick_spacing, update)?;
                Ok(())
            }
            _ => Err("ErrorCode::TickArrayIndexOutOfBounds.into()"),
        }
    }

    pub fn get_tick_offset(
        &self,
        array_index: usize,
        tick_index: i32,
        tick_spacing: u16,
    ) -> Result<isize, &'static str> {
        let array = self.arrays.get(array_index);
        match array {
            Some(array) => array.tick_offset(tick_index, tick_spacing),
            _ => Err("ErrorCode::TickArrayIndexOutOfBounds"),
        }
    }

    pub fn get_next_initialized_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
        start_array_index: usize,
    ) -> Result<(usize, i32), &'static str> {
        let ticks_in_array = TICK_ARRAY_SIZE * tick_spacing as i32;
        let mut search_index = tick_index;
        let mut array_index = start_array_index;

        // Keep looping the arrays until an initialized tick index in the subsequent tick-arrays found.
        loop {
            // If we get to the end of the array sequence and next_index is still not found, throw error
            let next_array = match self.arrays.get(array_index) {
                Some(array) => array,
                None => return Err("ErrorCode::TickArraySequenceInvalidIndex"),
            };

            let next_index =
                next_array.get_next_init_tick_index(search_index, tick_spacing, a_to_b)?;

            match next_index {
                Some(next_index) => {
                    return Ok((array_index, next_index));
                }
                None => {
                    // If we are at the last valid tick array, return the min/max tick index
                    if a_to_b && next_array.is_min_tick_array() {
                        return Ok((array_index, MIN_TICK_INDEX));
                    } else if !a_to_b && next_array.is_max_tick_array(tick_spacing) {
                        return Ok((array_index, MAX_TICK_INDEX));
                    }

                    // If we are at the last tick array in the sequencer, return the last tick
                    if array_index + 1 == self.arrays.len() {
                        if a_to_b {
                            return Ok((array_index, next_array.start_tick_index()));
                        } else {
                            let last_tick = next_array.start_tick_index() + ticks_in_array - 1;
                            return Ok((array_index, last_tick));
                        }
                    }

                    // No initialized index found. Move the search-index to the 1st search position
                    // of the next array in sequence.
                    search_index = if a_to_b {
                        next_array.start_tick_index() - 1
                    } else {
                        next_array.start_tick_index() + ticks_in_array - 1
                    };

                    array_index += 1;
                }
            }
        }
    }
}

pub(crate) enum ProxiedTickArray {
    Initialized(TickArray),
    Uninitialized(ZeroedTickArray),
}

impl ProxiedTickArray {
    pub fn new_initialized(refmut: TickArray) -> Self {
        ProxiedTickArray::Initialized(refmut)
    }

    pub fn new_uninitialized(start_tick_index: i32) -> Self {
        ProxiedTickArray::Uninitialized(ZeroedTickArray::new(start_tick_index))
    }

    pub fn start_tick_index(&self) -> i32 {
        self.as_ref().start_tick_index()
    }

    pub fn get_next_init_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>, &'static str> {
        self.as_ref()
            .get_next_init_tick_index(tick_index, tick_spacing, a_to_b)
    }

    pub fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<&Tick, &'static str> {
        self.as_ref().get_tick(tick_index, tick_spacing)
    }

    pub fn update_tick(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        update: &TickUpdate,
    ) -> Result<(), &'static str> {
        self.as_mut().update_tick(tick_index, tick_spacing, update)
    }

    pub fn tick_offset(&self, tick_index: i32, tick_spacing: u16) -> Result<isize, &'static str> {
        self.as_ref().tick_offset(tick_index, tick_spacing)
    }

    pub fn is_min_tick_array(&self) -> bool {
        self.as_ref().is_min_tick_array()
    }

    pub fn is_max_tick_array(&self, tick_spacing: u16) -> bool {
        self.as_ref().is_max_tick_array(tick_spacing)
    }
}

impl<'a> AsRef<dyn TickArrayType + 'a> for ProxiedTickArray {
    fn as_ref(&self) -> &(dyn TickArrayType + 'a) {
        match self {
            ProxiedTickArray::Initialized(ref array) => array,
            ProxiedTickArray::Uninitialized(ref array) => array,
        }
    }
}

impl<'a> AsMut<dyn TickArrayType + 'a> for ProxiedTickArray {
    fn as_mut(&mut self) -> &mut (dyn TickArrayType + 'a) {
        match self {
            ProxiedTickArray::Initialized(ref mut array) => &mut *array,
            ProxiedTickArray::Uninitialized(ref mut array) => array,
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct TickUpdate {
    pub initialized: bool,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_a: u128,
    pub fee_growth_outside_b: u128,
    pub reward_growths_outside: [u128; NUM_REWARDS],
}

impl TickUpdate {
    pub fn from(tick: &Tick) -> TickUpdate {
        TickUpdate {
            initialized: tick.initialized,
            liquidity_net: tick.liquidity_net,
            liquidity_gross: tick.liquidity_gross,
            fee_growth_outside_a: tick.fee_growth_outside_a,
            fee_growth_outside_b: tick.fee_growth_outside_b,
            reward_growths_outside: tick.reward_growths_outside,
        }
    }
}

#[derive(Clone)]
pub struct TickArray {
    pub start_tick_index: i32,
    pub ticks: [Tick; TICK_ARRAY_SIZE_USIZE],
    pub whirlpool: Pubkey,
}

impl TickArrayType for TickArray {

    fn start_tick_index(&self) -> i32 {
        self.start_tick_index
    }

    fn get_next_init_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>, &'static str> {
        if !self.in_search_range(tick_index, tick_spacing, !a_to_b) {
            return Err("ErrorCode::InvalidTickArraySequence");
        }

        let mut curr_offset = match self.tick_offset(tick_index, tick_spacing) {
            Ok(value) => value as i32,
            Err(e) => return Err(e),
        };

        // For a_to_b searches, the search moves to the left. The next possible init-tick can be the 1st tick in the current offset
        // For b_to_a searches, the search moves to the right. The next possible init-tick cannot be within the current offset
        if !a_to_b {
            curr_offset += 1;
        }

        while (0..TICK_ARRAY_SIZE).contains(&curr_offset) {
            let curr_tick = &self.ticks[curr_offset as usize];
            if curr_tick.initialized {
                return Ok(Some(
                    (curr_offset * tick_spacing as i32) + self.start_tick_index,
                ));
            }

            curr_offset = if a_to_b {
                curr_offset - 1
            } else {
                curr_offset + 1
            };
        }

        Ok(None)
    }

    fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<&Tick, &'static str> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err("ErrorCode::TickNotFound");
        }
        let offset = self.tick_offset(tick_index, tick_spacing)?;
        if offset < 0 {
            return Err("ErrorCode::TickNotFound");
        }
        Ok(&self.ticks[offset as usize])
    }

    fn update_tick(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        update: &TickUpdate,
    ) -> Result<(), &'static str> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err("ErrorCode::TickNotFound");
        }
        let offset = self.tick_offset(tick_index, tick_spacing)?;
        if offset < 0 {
            return Err("ErrorCode::TickNotFound");
        }
        self.ticks.get_mut(offset as usize).unwrap().update(update);
        Ok(())
    }
}

pub(crate) struct ZeroedTickArray {
    pub start_tick_index: i32,
    zeroed_tick: Tick,
}

impl ZeroedTickArray {
    pub fn new(start_tick_index: i32) -> Self {
        ZeroedTickArray {
            start_tick_index,
            zeroed_tick: Tick::default(),
        }
    }
}

impl TickArrayType for ZeroedTickArray {
    fn start_tick_index(&self) -> i32 {
        self.start_tick_index
    }

    fn get_next_init_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>, &'static str> {
        if !self.in_search_range(tick_index, tick_spacing, !a_to_b) {
            return Err("ErrorCode::InvalidTickArraySequence");
        }

        self.tick_offset(tick_index, tick_spacing)?;

        // no initialized tick
        Ok(None)
    }

    fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<&Tick, &'static str> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err("ErrorCode::TickNotFound");
        }
        let offset = self.tick_offset(tick_index, tick_spacing)?;
        if offset < 0 {
            return Err("ErrorCode::TickNotFound");
        }

        // always return the zeroed tick
        Ok(&self.zeroed_tick)
    }

    fn update_tick(
        &mut self,
        _tick_index: i32,
        _tick_spacing: u16,
        _update: &TickUpdate,
    ) -> Result<(), &'static str> {
        panic!("ZeroedTickArray must not be updated");
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    // Total 137 bytes
    pub initialized: bool,     // 1
    pub liquidity_net: i128,   // 16
    pub liquidity_gross: u128, // 16

    // Q64.64
    pub fee_growth_outside_a: u128, // 16
    // Q64.64
    pub fee_growth_outside_b: u128, // 16

    // Array of Q64.64
    pub reward_growths_outside: [u128; NUM_REWARDS], // 48 = 16 * 3
}

impl Tick {
    pub fn check_is_out_of_bounds(tick_index: i32) -> bool {
        !(MIN_TICK_INDEX..=MAX_TICK_INDEX).contains(&tick_index)
    }

    pub fn check_is_usable_tick(tick_index: i32, tick_spacing: u16) -> bool {
        if Tick::check_is_out_of_bounds(tick_index) {
            return false;
        }

        tick_index % tick_spacing as i32 == 0
    }

    pub fn update(&mut self, update: &TickUpdate) {
        self.initialized = update.initialized;
        self.liquidity_net = update.liquidity_net;
        self.liquidity_gross = update.liquidity_gross;
        self.fee_growth_outside_a = update.fee_growth_outside_a;
        self.fee_growth_outside_b = update.fee_growth_outside_b;
        self.reward_growths_outside = update.reward_growths_outside;
    }
}

pub trait TickArrayType {
    fn start_tick_index(&self) -> i32;

    fn get_next_init_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>, &'static str>;

    fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<&Tick, &'static str>;

    fn in_search_range(&self, tick_index: i32, tick_spacing: u16, shifted: bool) -> bool {
        let mut lower = self.start_tick_index();
        let mut upper = self.start_tick_index() + TICK_ARRAY_SIZE * tick_spacing as i32;
        if shifted {
            lower -= tick_spacing as i32;
            upper -= tick_spacing as i32;
        }
        tick_index >= lower && tick_index < upper
    }

    fn check_in_array_bounds(&self, tick_index: i32, tick_spacing: u16) -> bool {
        self.in_search_range(tick_index, tick_spacing, false)
    }

    fn is_min_tick_array(&self) -> bool {
        self.start_tick_index() <= MIN_TICK_INDEX
    }

    fn is_max_tick_array(&self, tick_spacing: u16) -> bool {
        self.start_tick_index() + TICK_ARRAY_SIZE * (tick_spacing as i32) > MAX_TICK_INDEX
    }

    fn tick_offset(&self, tick_index: i32, tick_spacing: u16) -> Result<isize, &'static str> {
        if tick_spacing == 0 {
            return Err("ErrorCode::InvalidTickSpacing");
        }

        Ok(get_offset(
            tick_index,
            self.start_tick_index(),
            tick_spacing,
        ))
    }

    fn update_tick(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        update: &TickUpdate,
    ) -> Result<(), &'static str>;
}

fn get_offset(tick_index: i32, start_tick_index: i32, tick_spacing: u16) -> isize {
    // TODO: replace with i32.div_floor once not experimental
    let lhs = tick_index - start_tick_index;
    let rhs = tick_spacing as i32;
    let d = lhs / rhs;
    let r = lhs % rhs;
    let o = if (r > 0 && rhs < 0) || (r < 0 && rhs > 0) {
        d - 1
    } else {
        d
    };
    o as isize
}

pub fn next_tick_cross_update(
    tick: &Tick,
    fee_growth_global_a: u128,
    fee_growth_global_b: u128,
    reward_infos: &[WhirlpoolRewardInfo; NUM_REWARDS],
) -> Result<TickUpdate, &'static str> {
    let mut update = TickUpdate::from(tick);

    update.fee_growth_outside_a = fee_growth_global_a.wrapping_sub(tick.fee_growth_outside_a);
    update.fee_growth_outside_b = fee_growth_global_b.wrapping_sub(tick.fee_growth_outside_b);

    for (i, reward_info) in reward_infos.iter().enumerate() {
        if !reward_info.initialized() {
            continue;
        }

        update.reward_growths_outside[i] = reward_info
            .growth_global_x64
            .wrapping_sub(tick.reward_growths_outside[i]);
    }
    Ok(update)
}

pub fn checked_mul_div(n0: u128, n1: u128, d: u128) -> Result<u128, &'static str> {
    checked_mul_div_round_up_if(n0, n1, d, false)
}

pub fn checked_mul_div_round_up(n0: u128, n1: u128, d: u128) -> Result<u128, &'static str> {
    checked_mul_div_round_up_if(n0, n1, d, true)
}

pub fn checked_mul_div_round_up_if(
    n0: u128,
    n1: u128,
    d: u128,
    round_up: bool,
) -> Result<u128, &'static str> {
    if d == 0 {
        return Err("ErrorCode::DivideByZero");
    }

    let p = n0.checked_mul(n1).ok_or("ErrorCode::MulDivOverflow")?;
    let n = p / d;

    Ok(if round_up && p % d > 0 { n + 1 } else { n })
}

// #[cfg(test)]
pub mod tick_builder {
    use super::{NUM_REWARDS, Tick};

    #[derive(Default)]
    pub struct TickBuilder {
        initialized: bool,
        liquidity_net: i128,
        liquidity_gross: u128,
        fee_growth_outside_a: u128,
        fee_growth_outside_b: u128,
        reward_growths_outside: [u128; NUM_REWARDS],
    }

    impl TickBuilder {
        pub fn initialized(mut self, initialized: bool) -> Self {
            self.initialized = initialized;
            self
        }

        pub fn liquidity_net(mut self, liquidity_net: i128) -> Self {
            self.liquidity_net = liquidity_net;
            self
        }

        pub fn liquidity_gross(mut self, liquidity_gross: u128) -> Self {
            self.liquidity_gross = liquidity_gross;
            self
        }

        pub fn fee_growth_outside_a(mut self, fee_growth_outside_a: u128) -> Self {
            self.fee_growth_outside_a = fee_growth_outside_a;
            self
        }

        pub fn fee_growth_outside_b(mut self, fee_growth_outside_b: u128) -> Self {
            self.fee_growth_outside_b = fee_growth_outside_b;
            self
        }

        pub fn reward_growths_outside(
            mut self,
            reward_growths_outside: [u128; NUM_REWARDS],
        ) -> Self {
            self.reward_growths_outside = reward_growths_outside;
            self
        }

        pub fn build(self) -> Tick {
            Tick {
                initialized: self.initialized,
                liquidity_net: self.liquidity_net,
                liquidity_gross: self.liquidity_gross,
                fee_growth_outside_a: self.fee_growth_outside_a,
                fee_growth_outside_b: self.fee_growth_outside_b,
                reward_growths_outside: self.reward_growths_outside,
            }
        }
    }
}