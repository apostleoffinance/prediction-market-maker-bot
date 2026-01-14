use crate::market_maker::{FillResult, MarketMaker, Order};
use crate::market_state::MarketState;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub fills: Vec<FillInfo>,
    pub mid: Decimal,
    pub inventory: Decimal,
    pub pnl: Decimal,
    pub spread: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillInfo {
    pub side: String,
    pub size: Decimal,
    pub price: Decimal,
}

impl From<&FillResult> for FillInfo {
    fn from(fill: &FillResult) -> Self {
        FillInfo {
            side: fill.side.clone(),
            size: fill.size,
            price: fill.price,
        }
    }
}

pub struct ExecutionEngine {
    pub markets: HashMap<String, MarketState>,
    pub market_makers: HashMap<String, MarketMaker>,
    pub time: u64,
    pub rng: ChaCha8Rng,
}

impl ExecutionEngine {
    pub fn new(markets: HashMap<String, MarketState>, rng_seed: u64) -> Self {
        let market_makers: HashMap<String, MarketMaker> = markets
            .iter()
            .map(|(name, state)| (name.clone(), MarketMaker::new(state, None)))
            .collect();

        ExecutionEngine {
            markets,
            market_makers,
            time: 0,
            rng: ChaCha8Rng::seed_from_u64(rng_seed),
        }
    }

    /// Simulate random market order flow for a given market
    fn simulate_order_flow(&mut self, market_name: &str) -> Result<Vec<Order>, Box<dyn Error>> {
        let state = self.markets.get(market_name)
            .ok_or_else(|| format!("Market '{}' not found", market_name))?;
        let mut orders = Vec::new();
        
        // Generate 1-3 orders per tick
        let n = self.rng.gen_range(1..=3);
        
        for _ in 0..n {
            // Bias toward mid: higher mid -> more buys, lower mid -> more sells
            let noise: f64 = self.rng.gen_range(-0.15..0.15);
            let prob_f64 = state.mid.to_f64().ok_or("Failed to convert mid to f64")? + noise;
            
            let side = if prob_f64 > 0.5 { "buy" } else { "sell" };
            
            // Size follows a normal-ish distribution clamped to [1, 30]
            let size_f64: f64 = (self.rng.gen::<f64>() * 4.0 + 4.0).max(1.0).min(30.0);
            let size = Decimal::from_f64(size_f64).ok_or("Failed to convert size to Decimal")?;
            
            // Price: buyers willing to pay 1.0, sellers accept 0.0
            let price = if side == "buy" { Decimal::ONE } else { Decimal::ZERO };
            
            orders.push(Order {
                side: side.to_string(),
                size,
                price,
            });
        }
        
        Ok(orders)
    }

    /// Execute one simulation step across all markets
    pub fn step(&mut self) -> Result<HashMap<String, StepResult>, Box<dyn Error>> {
        let mut results = HashMap::new();
        
        let market_names: Vec<String> = self.markets.keys().cloned().collect();
        
        for name in market_names {
            let orders = self.simulate_order_flow(&name)?;
            
            // Get mutable references
            let state = self.markets.get_mut(&name)
                .ok_or_else(|| format!("Market '{}' not found", name))?;
            let mm = self.market_makers.get_mut(&name)
                .ok_or_else(|| format!("Market maker for '{}' not found", name))?;
            
            let fills = mm.on_tick(state, &orders)?;
            
            // Update PnL for each fill
            for fill in &fills {
                let signed = if fill.side == "buy" { fill.size } else { -fill.size };
                let prev_mid = state.mid;
                state.pnl += -signed * (fill.price - prev_mid);
                state.peak_pnl = state.peak_pnl.max(state.pnl);
                let dd = state.peak_pnl - state.pnl;
                state.max_drawdown = state.max_drawdown.max(dd);
            }
            
            // Small mean reversion toward 0.5
            let alpha = Decimal::new(995, 3);  // 0.995
            let beta = Decimal::new(5, 3);     // 0.005
            let half = Decimal::new(5, 1);     // 0.5
            state.mid = state.mid * alpha + half * beta;
            
            results.insert(
                name,
                StepResult {
                    fills: fills.iter().map(FillInfo::from).collect(),
                    mid: state.mid,
                    inventory: state.inventory,
                    pnl: state.pnl,
                    spread: state.spread,
                },
            );
        }
        
        self.time += 1;
        Ok(results)
    }

    /// Run simulation for a given number of steps
    pub fn run(&mut self, steps: usize) -> Result<Vec<HashMap<String, StepResult>>, Box<dyn Error>> {
        let mut trace = Vec::with_capacity(steps);
        
        for _ in 0..steps {
            trace.push(self.step()?);
        }
        
        Ok(trace)
    }
}
