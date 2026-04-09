import polars as pl
from hdrhistogram_pyo3 import pig_latinnify, hdr_mean, hdr_percentile, hdr_stats_summary


# Example 1: Pig Latin (legacy)
print("=== Pig Latin Example ===")
df_pig = pl.DataFrame(
    {
        "english": ["this", "is", "not", "pig", "latin"],
    }
)
result_pig = df_pig.with_columns(pig_latin=pig_latinnify("english"))
print(result_pig)
print()

# Example 2: HdrHistogram statistics
print("=== HdrHistogram Statistics Example ===")
latencies = [10.0, 15.0, 12.0, 20.0, 18.0, 25.0, 22.0, 19.0, 17.0, 23.0, 21.0, 24.0, 16.0, 18.0, 20.0]
df_latency = pl.DataFrame({"latency_ms": latencies})

result_stats = df_latency.select(
    mean=hdr_mean("latency_ms"),
    p50=hdr_percentile("latency_ms", 50.0),
    p95=hdr_percentile("latency_ms", 95.0),
    p99=hdr_percentile("latency_ms", 99.0),
    summary=hdr_stats_summary("latency_ms"),
)
print(result_stats)


