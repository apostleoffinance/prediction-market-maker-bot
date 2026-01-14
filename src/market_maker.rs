use crate::market_state::MarketState;
use rust_decimal::Decimal;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MarketMakerConfig {
    pub window_size: usize,
    pub base_spread: Decimal,
    pub min_spread: Decimal,
    pub max_spread: Decimal,
    pub inventory_skew: Decimal,
}

impl Default for MarketMakerConfig {
    fn default() -> Self {
        MarketMakerConfig {
            window_size: 20,
            base_spread: Decimal::new(5, 2),  // 0.05
            min_spread: Decimal::new(1, 2),   // 0.01
            max_spread: Decimal::new(5, 1),   // 0.5
            inventory_skew: Decimal::new(1, 3), // 0.001
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub side: String,
    pub size: Decimal,
    pub price: Decimal,
}

#[derive(Debug, Clone)]
pub struct FillResult {
    pub side: String,
    pub size: Decimal,
    pub price: Decimal,
}

pub struct MarketMaker {
    pub config: MarketMakerConfig,
    pub imbalance_window: VecDeque<Decimal>,
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
    pub fn quote(&mut self, state: &mut MarketState) -> (Decimal, Decimal, Decimal) {
        let mid = state.mid;
        
        // Calculate imbalance from recent window
        let imbalance: Decimal = self.imbalance_window
            .iter()
            .rev()
            .take(self.config.window_size)
            .sum();
        
        let abs_imb = imbalance.abs();
        
        // Adaptive spread: widens with imbalance and inventory
        let ten = Decimal::new(10, 0);
        let spread = self.config.base_spread 
            * (Decimal::ONE + abs_imb / ten + state.inventory.abs() * self.config.inventory_skew);
        let spread = spread.max(self.config.min_spread).min(self.config.max_spread);
        
        // Inventory skew: shade mid price based on inventory
        let skew = state.inventory * self.config.inventory_skew;
        let mid_shaded = (mid - skew).max(Decimal::new(1, 2)).min(Decimal::new(99, 2));  // 0.01 to 0.99
        
        // Calculate bid/ask
        let two = Decimal::new(2, 0);
        let bid = (mid_shaded - spread / two).max(Decimal::ZERO);
        let ask = (mid_shaded + spread / two).min(Decimal::ONE);
        
        // Size inversely related to inventory
        let size_calc = Decimal::new(10, 0) - state.inventory.abs() / Decimal::new(10, 0);
        let size = size_calc.max(Decimal::ONE).min(Decimal::new(20, 0));
        
        // Update state spread
        state.spread = spread;
        
        (bid, ask, size)
    }

    /// Process a fill and update internal state
    pub fn on_fill(&mut self, state: &mut MarketState, side: &str, size: Decimal) {
        let delta = if side == "buy" { size } else { -size };
        
        // Update imbalance window
        self.imbalance_window.push_back(delta);
        let max_window = (self.config.window_size * 4).max(100);
        while self.imbalance_window.len() > max_window {
            self.imbalance_window.pop_front();
        }
        
        // Adjust mid based on flow
        let alpha = Decimal::new(5, 2);  // 0.05
        let flow = delta;
        let ten = Decimal::new(10, 0);
        let mid_adjustment = alpha * (flow / (ten + flow.abs()));
        state.mid = (state.mid + mid_adjustment).max(Decimal::new(1, 2)).min(Decimal::new(99, 2));
        
        // Defensive adjustment when inventory is high
        let inv = state.inventory;
        let threshold = state.inventory_limit * Decimal::new(8, 1);  // 0.8
        if inv.abs() > threshold {
            let correction = if inv > Decimal::ZERO { 
                -Decimal::new(5, 2)  // -0.05
            } else { 
                Decimal::new(5, 2)   // 0.05
            };
            state.mid = (state.mid + correction).max(Decimal::new(1, 2)).min(Decimal::new(99, 2));
        }
    }

    /// Process incoming market orders and generate fills
    pub fn on_tick(&mut self, state: &mut MarketState, market_order_flow: &[Order]) -> Result<Vec<FillResult>, Box<dyn std::error::Error>> {
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
            state.record_fill(&fill.side, fill.size, fill.price)?;
            self.on_fill(state, &fill.side, fill.size);
        }
        
        Ok(fills)
    }
}
