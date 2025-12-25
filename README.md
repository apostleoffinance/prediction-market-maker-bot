# Quant Execution Bot - Rust Implementation

A high-performance market-making execution bot for binary event contracts (prediction markets) implemented in Rust.

## 🚀 Features

- **Adaptive Spread Management**: Dynamically adjusts spreads based on order flow imbalance and inventory
- **Inventory Skew**: Shades mid prices to manage accumulated positions
- **Risk Controls**: Inventory limits, exposure limits, and drawdown tracking
- **Mean Reversion**: Mid prices slowly revert toward fair value (0.5)
- **Deterministic Simulation**: Reproducible results with seeded RNG

## 📁 Project Structure

```
quant_bot_rust/
├── Cargo.toml                 # Project dependencies
├── src/
│   ├── main.rs                # Entry point and simulation orchestrator
│   ├── market_state.rs        # Market state container and trade recording
│   ├── market_maker.rs        # Core quoting logic and adaptation algorithms
│   ├── execution_engine.rs    # Simulation driver and order flow generator
│   └── logger.rs              # CSV report writer
├── simulation_report.csv      # Final metrics (generated)
└── trace.json                 # Time-series data (generated)
```

## 🛠️ Build & Run

### Prerequisites
- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --release
```

### Expected Output
```
🚀 Quant Execution Bot - Rust Implementation
============================================

📊 Running simulation with 200 steps...

✅ Simulation complete. Report written to: simulation_report.csv
✅ Trace data written to: trace.json

📈 Final Market States:
------------------------
🏪 inflation_gt_20 { ... }
🏪 team_x_wins { ... }
🏪 election_candidate_a { ... }

📊 Summary Statistics:
----------------------
Total PnL: 565.4478
Total Fills: 1189
...
```

## 📊 Output Files

| File | Description |
|------|-------------|
| `simulation_report.csv` | Final metrics for all markets (PnL, fills, drawdown, etc.) |
| `trace.json` | Step-by-step time-series data for analysis |

## 🏪 Simulated Markets

| Market | Initial Mid | Description |
|--------|-------------|-------------|
| `inflation_gt_20` | 0.30 | "Will inflation exceed 20%?" |
| `election_candidate_a` | 0.55 | "Will Candidate A win?" |
| `team_x_wins` | 0.50 | "Will Team X win?" |

## 🔧 Configuration

Modify market parameters in `main.rs`:

```rust
let mut market = MarketState::new("my_market", 0.50);
market.inventory_limit = 200.0;   // Max inventory
market.exposure_limit = 10000.0;  // Max exposure
market.spread = 0.05;             // Initial spread
```

## 📈 Performance

- Zero-copy operations where possible
- Efficient HashMap-based market lookups
- Deterministic PRNG (ChaCha8) for reproducibility
- Release mode optimizations
- Memory-safe with no garbage collection overhead

## 📄 License

MIT
