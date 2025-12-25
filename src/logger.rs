use crate::market_state::MarketState;
use csv::Writer;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, serde::Serialize)]
struct ReportRow {
    market: String,
    mid: f64,
    spread: f64,
    inventory: f64,
    pnl: f64,
    fill_count: u64,
    notional: f64,
    max_drawdown: f64,
}

pub fn write_report(
    states: &HashMap<String, MarketState>,
    out_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(out_path)?;
    let mut writer = Writer::from_writer(file);

    for (name, state) in states {
        let row = ReportRow {
            market: name.clone(),
            mid: state.mid,
            spread: state.spread,
            inventory: state.inventory,
            pnl: state.pnl,
            fill_count: state.fill_count,
            notional: state.notional,
            max_drawdown: state.max_drawdown,
        };
        writer.serialize(row)?;
    }

    writer.flush()?;
    Ok(())
}
