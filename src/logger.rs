use crate::market_state::MarketState;
use csv::Writer;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, serde::Serialize)]
struct ReportRow {
    market: String,
    mid: Decimal,
    spread: Decimal,
    inventory: Decimal,
    pnl: Decimal,
    fill_count: u64,
    notional: Decimal,
    max_drawdown: Decimal,
}

pub fn write_report(
    states: &HashMap<String, MarketState>,
    out_path: &Path,
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
