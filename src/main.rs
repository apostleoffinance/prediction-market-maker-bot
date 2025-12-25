mod execution_engine;
mod logger;
mod market_maker;
mod market_state;

use execution_engine::ExecutionEngine;
use market_state::MarketState;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;

fn build_markets() -> HashMap<String, MarketState> {
    let mut markets = HashMap::new();

    // Create three prediction markets
    let mut inflation = MarketState::new("inflation_gt_20", 0.30);
    inflation.inventory_limit = 200.0;
    inflation.exposure_limit = 10000.0;
    inflation.spread = 0.05;
    markets.insert("inflation_gt_20".to_string(), inflation);

    let mut election = MarketState::new("election_candidate_a", 0.55);
    election.inventory_limit = 200.0;
    election.exposure_limit = 10000.0;
    election.spread = 0.05;
    markets.insert("election_candidate_a".to_string(), election);

    let mut team = MarketState::new("team_x_wins", 0.50);
    team.inventory_limit = 200.0;
    team.exposure_limit = 10000.0;
    team.spread = 0.05;
    markets.insert("team_x_wins".to_string(), team);

    markets
}

fn run_demo() -> Result<String, Box<dyn std::error::Error>> {
    println!("🚀 Quant Execution Bot - Rust Implementation");
    println!("============================================\n");

    let markets = build_markets();
    let mut engine = ExecutionEngine::new(markets, 123);

    println!("📊 Running simulation with 200 steps...\n");
    let trace = engine.run(200);

    // Get output directory (current executable's directory or current dir)
    let out_dir = env::current_dir()?;
    let csv_path = out_dir.join("simulation_report.csv");
    let trace_path = out_dir.join("trace.json");

    // Write CSV report
    logger::write_report(&engine.markets, csv_path.to_str().unwrap())?;
    println!("✅ Simulation complete. Report written to: {}", csv_path.display());

    // Write trace JSON
    let trace_json = serde_json::to_string_pretty(&trace)?;
    let mut trace_file = File::create(&trace_path)?;
    trace_file.write_all(trace_json.as_bytes())?;
    println!("✅ Trace data written to: {}\n", trace_path.display());

    // Print final market states
    println!("📈 Final Market States:");
    println!("------------------------");
    for (name, state) in &engine.markets {
        let snapshot = state.snapshot();
        println!(
            "\n🏪 {} {{",
            name
        );
        println!("    mid: {:.4}", snapshot.mid);
        println!("    spread: {:.4}", snapshot.spread);
        println!("    inventory: {:.2}", snapshot.inventory);
        println!("    pnl: {:.4}", snapshot.pnl);
        println!("    fill_count: {}", snapshot.fill_count);
        println!("    notional: {:.2}", snapshot.notional);
        println!("    max_drawdown: {:.4}", snapshot.max_drawdown);
        println!("}}");
    }

    // Print summary statistics
    println!("\n📊 Summary Statistics:");
    println!("----------------------");
    let total_pnl: f64 = engine.markets.values().map(|s| s.pnl).sum();
    let total_fills: u64 = engine.markets.values().map(|s| s.fill_count).sum();
    let total_notional: f64 = engine.markets.values().map(|s| s.notional).sum();
    let max_dd: f64 = engine.markets.values().map(|s| s.max_drawdown).fold(0.0, f64::max);

    println!("Total PnL: {:.4}", total_pnl);
    println!("Total Fills: {}", total_fills);
    println!("Total Notional: {:.2}", total_notional);
    println!("Max Drawdown: {:.4}", max_dd);

    Ok(csv_path.to_string_lossy().to_string())
}

fn main() {
    match run_demo() {
        Ok(_) => {
            println!("\n✨ Simulation completed successfully!");
        }
        Err(e) => {
            eprintln!("❌ Error running simulation: {}", e);
            std::process::exit(1);
        }
    }
}
