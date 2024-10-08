#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(packed)]
pub struct LeafNode {
    tag: u32,
    owner_slot: u8,
    fee_tier: u8,
    padding: [u8; 2],
    key: u128,
    owner: [u64; 4],
    quantity: u64,
    client_order_id: u64,
}