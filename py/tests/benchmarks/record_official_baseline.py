#!/usr/bin/env python3
"""
Record Official SDK Baseline Performance

Records baseline latency metrics from the official fugle-marketdata-python SDK.
Run this once with FUGLE_API_KEY to establish comparison baseline.

Usage: python record_official_baseline.py
Output: baseline.json with median latencies for each operation
"""
import os
import json
import time
import statistics
from pathlib import Path


def main():
    api_key = os.environ.get("FUGLE_API_KEY")
    if not api_key:
        print("Error: FUGLE_API_KEY not set")
        print("Set your API key to record official SDK baseline:")
        print("  export FUGLE_API_KEY=your_key")
        print("  python record_official_baseline.py")
        return 1

    try:
        from fugle_marketdata import RestClient
    except ImportError:
        print("Error: fugle-marketdata not installed")
        print("Install official SDK: pip install fugle-marketdata")
        return 1

    print("Recording official SDK baseline...")
    client = RestClient(api_key=api_key)

    operations = {
        "quote": lambda: client.stock.intraday.quote(symbol="2330"),
        "ticker": lambda: client.stock.intraday.ticker(symbol="2330"),
        "trades": lambda: client.stock.intraday.trades(symbol="2330"),
    }

    results = {}
    warmup_rounds = 3
    measure_rounds = 10

    for name, operation in operations.items():
        print(f"  Benchmarking {name}...")

        # Warmup
        for _ in range(warmup_rounds):
            try:
                operation()
            except Exception:
                pass
            time.sleep(0.5)

        # Measure
        latencies = []
        for i in range(measure_rounds):
            start = time.perf_counter()
            try:
                operation()
                elapsed = time.perf_counter() - start
                latencies.append(elapsed * 1000)  # Convert to ms
            except Exception as e:
                print(f"    Round {i+1} failed: {e}")
            time.sleep(0.5)

        if latencies:
            results[name] = {
                "median_ms": statistics.median(latencies),
                "mean_ms": statistics.mean(latencies),
                "min_ms": min(latencies),
                "max_ms": max(latencies),
                "stdev_ms": statistics.stdev(latencies) if len(latencies) > 1 else 0,
                "rounds": len(latencies),
            }
            print(f"    Median: {results[name]['median_ms']:.2f}ms")

    # Write baseline
    baseline_path = Path(__file__).parent / "baseline.json"
    with open(baseline_path, "w") as f:
        json.dump({
            "sdk": "fugle-marketdata-python",
            "recorded_at": time.strftime("%Y-%m-%d %H:%M:%S"),
            "operations": results,
        }, f, indent=2)

    print(f"\nBaseline written to: {baseline_path}")
    return 0


if __name__ == "__main__":
    exit(main())
