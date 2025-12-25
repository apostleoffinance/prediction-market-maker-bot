use crate::market_state::MarketState;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MarketMakerConfig {
    pub window_size: usize,
    pub base_spread: f64,
    pub min_spread: f64,
    pub max_spread: f64,
    pub inventory_skew: f64,
}

impl Default for MarketMakerConfig {
    fn default() -> Self {
        MarketMakerConfig {
            window_size: 20,
            base_spread: 0.05,
            min_spread: 0.01,
            max_spread: 0.5,
            inventory_skew: 0.001,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub side: String,
    pub size: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct FillResult {
    pub side: String,
    pub size: f64,
    pub price: f64,
}

pub struct MarketMaker {
    pub config: MarketMakerConfig,
    pub imbalance_window: VecDeque<f64>,
}

impl MarketMaker {
    pub fn new(state: &MarketState, config: Option<MarketMakerConfig>) -> Self {
        let mut cfg = config.unwrap_or_default();
        cfg.base_spread = state.spread;
        
        MarketMaker {
            config: cfg,
            imbalance_window: VecDeque::new(),
        }
    }

    /// Generate bid/ask quotes based on current market state
    /// Returns (bid, ask, size)
    pub fn quote(&mut self, state: &mut MarketState) -> (f64, f64, f64) {
        let mid = state.mid;
        
        // Calculate imbalance from recent window
        let imbalance: f64 = self.imbalance_window
            .iter()
            .rev()
            .take(self.config.window_size)
            .sum();
        
        let abs_imb = imbalance.abs();
        
        // Adaptive spread: widens with imbalance and inventory
        let spread = self.config.base_spread 
            * (1.0 + abs_imb / 10.0 + state.inventory.abs() * self.config.inventory_skew);
        let spread = spread.max(self.config.min_spread).min(self.config.max_spread);
        
        // Inventory skew: shade mid price based on inventory
        let skew = state.inventory * self.config.inventory_skew;
        let mid_shaded = (mid - skew).max(0.01).min(0.99);
        
        // Calculate bid/ask
        let bid = (mid_shaded - spread / 2.0).max(0.0);
        let ask = (mid_shaded + spread / 2.0).min(1.0);
        
        // Size inversely related to inventory
        let size = (10.0 - state.inventory.abs() / 10.0).max(1.0).min(20.0);
        
        // Update state spread
        state.spread = spread;
        
        (bid, ask, size)
    }

    /// Process a fill and update internal state
    pub fn on_fill(&mut self, state: &mut MarketState, side: &str, size: f64) {
        let delta = if side == "buy" { size } else { -size };
        
        // Update imbalance window
        self.imbalance_window.push_back(delta);
        let max_window = (self.config.window_size * 4).max(100);
        while self.imbalance_window.len() > max_window {
            self.imbalance_window.pop_front();
        }
        
        // Adjust mid based on flow
        let alpha = 0.05;
        let flow = delta;
        let mid_adjustment = alpha * (flow / (10.0 + flow.abs()));
        state.mid = (state.mid + mid_adjustment).max(0.01).min(0.99);
        
        // Defensive adjustment when inventory is high
        let inv = state.inventory;
        if inv.abs() > state.inventory_limit * 0.8 {
            let correction = if inv > 0.0 { -0.05 } else { 0.05 };
            state.mid = (state.mid + correction).max(0.01).min(0.99);
        }
    }

    /// Process incoming market orders and generate fills
    pub fn on_tick(&mut self, state: &mut MarketState, market_order_flow: &[Order]) -> Vec<FillResult> {
        let mut fills = Vec::new();
        let (bid, ask, _size) = self.quote(state);
        
        for order in market_order_flow {
            match order.side.as_str() {
                "buy" if order.price >= ask => {
                    // Taker buys, we sell
                    fills.push(FillResult {
                        side: "sell".to_string(),
                        size: order.size,
                        price: ask,
                    });
                }
                "sell" if order.price <= bid => {
                    // Taker sells, we buy
                    fills.push(FillResult {
                        side: "buy".to_string(),
                        size: order.size,
                        price: bid,
                    });
                }
                _ => {}
            }
        }
        
        // Record fills and update state
        for fill in &fills {
            state.record_fill(&fill.side, fill.size, fill.price);
            self.on_fill(state, &fill.side, fill.size);
        }
        
        fills
    }
}
