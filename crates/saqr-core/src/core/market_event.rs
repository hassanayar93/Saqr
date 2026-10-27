pub const MARKET_EVENT_TRADE: u8 = 1;
pub const MARKET_EVENT_DEPTH: u8 = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MarketEvent {
    pub kind: u8,
    pub symbol_hash: u64,
    pub price_scaled: i64,
    pub volume_scaled: i64,
    pub bid_depth_scaled: i64,
    pub ask_depth_scaled: i64,
    pub is_buyer_maker: bool,
    pub timestamp_ns: u64,
}

impl MarketEvent {
    pub fn trade(
        symbol_hash: u64,
        price_scaled: i64,
        volume_scaled: i64,
        is_buyer_maker: bool,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            kind: MARKET_EVENT_TRADE,
            symbol_hash,
            price_scaled,
            volume_scaled,
            bid_depth_scaled: 0,
            ask_depth_scaled: 0,
            is_buyer_maker,
            timestamp_ns,
        }
    }

    pub fn depth(
        symbol_hash: u64,
        bid_depth_scaled: i64,
        ask_depth_scaled: i64,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            kind: MARKET_EVENT_DEPTH,
            symbol_hash,
            price_scaled: 0,
            volume_scaled: 0,
            bid_depth_scaled,
            ask_depth_scaled,
            is_buyer_maker: false,
            timestamp_ns,
        }
    }
}
