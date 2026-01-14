use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub side: String,
    pub size: Decimal,
    pub price: Decimal,
    pub timestamp: u64,
}

impl Fill {
    pub fn new(side: &str, size: Decimal, price: Decimal) -> Result<Self, Box<dyn Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        Ok(Fill {
            side: side.to_string(),
            size,
            price,
            timestamp,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub name: String,
    pub mid: Decimal,           // mid probability (0..1)
    pub spread: Decimal,        // absolute spread (probability points)
    pub inventory: Decimal,
    pub exposure: Decimal,
    pub pnl: Decimal,
    pub fills: Vec<Fill>,
    pub fill_count: u64,
    pub notional: Decimal,
    pub max_drawdown: Decimal,
    pub peak_pnl: Decimal,
    // risk parameters
    pub inventory_limit: Decimal,
    pub exposure_limit: Decimal,
    pub fee: Decimal,
}

impl MarketState {
    pub fn new(name: &str, initial_mid: Decimal) -> Self {
        MarketState {
            name: name.to_string(),
            mid: initial_mid,
            spread: Decimal::new(5, 2),  // 0.05
            inventory: Decimal::ZERO,
            exposure: Decimal::ZERO,
            pnl: Decimal::ZERO,
            fills: Vec::new(),
            fill_count: 0,
            notional: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            peak_pnl: Decimal::ZERO,
            inventory_limit: Decimal::new(100, 0),  // 100.0
            exposure_limit: Decimal::new(10000, 0), // 10000.0
            fee: Decimal::ZERO,
        }
    }

    pub fn record_fill(&mut self, side: &str, size: Decimal, price: Decimal) -> Result<(), Box<dyn Error>> {
        let fill = Fill::new(side, size, price)?;
        self.fills.push(fill);
        self.fill_count += 1;
        self.notional += size.abs() * price;
        
        match side {
            "buy" => self.inventory += size,
            "sell" => self.inventory -= size,
            _ => {}
        }
        
        self.exposure = self.inventory.abs() * self.mid;
        Ok(())
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
    pub mid: Decimal,
    pub spread: Decimal,
    pub inventory: Decimal,
    pub exposure: Decimal,
    pub pnl: Decimal,
    pub fill_count: u64,
    pub notional: Decimal,
    pub max_drawdown: Decimal,
}
