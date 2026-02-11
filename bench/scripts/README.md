# Benchmark Scripts

Helper scripts for running and analyzing benchmarks.

## run-with-metrics.sh

Starts the server with automatic metrics collection:

```bash
./run-with-metrics.sh
```

- Builds the server if needed
- Starts server in background
- Collects CPU and memory metrics every 5 seconds
- Writes to `metrics.log` (CSV format: timestamp,cpu%,rss)
- Ctrl+C to stop

Configuration:

- `METRICS_FILE`: output file (default: metrics.log)
- `METRICS_INTERVAL`: sampling interval in seconds (default: 5)

## analyze-metrics.sh

Analyzes collected metrics:

```bash
./analyze-metrics.sh [metrics_file]
```

Outputs:

- CPU statistics (min/max/avg, threshold violations)
- Memory statistics (min/max/avg, growth)
- Duration
- Pass/fail assessment

## Usage Example

```bash
# Terminal 1: Start server with metrics
cd bench/scripts
./run-with-metrics.sh

# Terminal 2: Run workloads
cd bench/workloads
chmod +x *.sh
./run-all.sh

# After workloads complete, stop server (Ctrl+C in terminal 1)

# Analyze results
cd bench/scripts
./analyze-metrics.sh
```

## Integration with Soak Test

For 8-12 hour soak tests:

```bash
# Terminal 1: Start server with metrics
./run-with-metrics.sh

# Terminal 2: Run soak test
cd ../workloads
DURATION_HOURS=8 ./05-soak-test.sh

# After completion, analyze
cd ../scripts
./analyze-metrics.sh
```

## Pass Criteria

From requirements:

- Distribution accuracy: <10% error (checked in workload output)
- CPU efficiency: avg <80% under moderate load
- No sustained 100% CPU
- Memory stable: <20% growth over duration
- No crashes: server runs to completion
