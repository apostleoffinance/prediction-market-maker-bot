use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub side: String,
    pub size: f64,
    pub price: f64,
    pub timestamp: f64,
}

impl Fill {
    pub fn new(side: &str, size: f64, price: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        
        Fill {
            side: side.to_string(),
            size,
            price,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub name: String,
    pub mid: f64,           // mid probability (0..1)
    pub spread: f64,        // absolute spread (probability points)
    pub inventory: f64,
    pub exposure: f64,
    pub pnl: f64,
    pub fills: Vec<Fill>,
    pub fill_count: u64,
    pub notional: f64,
    pub max_drawdown: f64,
    pub peak_pnl: f64,
    // risk parameters
    pub inventory_limit: f64,
    pub exposure_limit: f64,
    pub fee: f64,
}

impl MarketState {
    pub fn new(name: &str, initial_mid: f64) -> Self {
        MarketState {
            name: name.to_string(),
            mid: initial_mid,
            spread: 0.05,
            inventory: 0.0,
            exposure: 0.0,
            pnl: 0.0,
            fills: Vec::new(),
            fill_count: 0,
            notional: 0.0,
            max_drawdown: 0.0,
            peak_pnl: 0.0,
            inventory_limit: 100.0,
            exposure_limit: 10000.0,
            fee: 0.0,
        }
    }

    pub fn record_fill(&mut self, side: &str, size: f64, price: f64) {
        let fill = Fill::new(side, size, price);
        self.fills.push(fill);
        self.fill_count += 1;
        self.notional += size.abs() * price;
        
        match side {
            "buy" => self.inventory += size,
            "sell" => self.inventory -= size,
            _ => {}
        }
        
        self.exposure = self.inventory.abs() * self.mid;
    }

    pub fn snapshot(&self) -> MarketSnapshot {
        MarketSnapshot {
            name: self.name.clone(),
            mid: self.mid,
            spread: self.spread,
            inventory: self.inventory,
            exposure: self.exposure,
            pnl: self.pnl,
            fill_count: self.fill_count,
            notional: self.notional,
            max_drawdown: self.max_drawdown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub name: String,
    pub mid: f64,
    pub spread: f64,
    pub inventory: f64,
    pub exposure: f64,
    pub pnl: f64,
    pub fill_count: u64,
    pub notional: f64,
    pub max_drawdown: f64,
}
