import polars as pl
from hdrhistogram_pyo3 import (
    pig_latinnify,
    hdr_mean,
    hdr_percentile,
    hdr_stats_summary,
)


def test_piglatinnify():
    df = pl.DataFrame(
        {
            "english": ["this", "is", "not", "pig", "latin"],
        }
    )
    result = df.with_columns(pig_latin=pig_latinnify("english"))

    expected_df = pl.DataFrame(
        {
            "english": ["this", "is", "not", "pig", "latin"],
            "pig_latin": ["histay", "siay", "otnay", "igpay", "atinlay"],
        }
    )

    assert result.equals(expected_df)


def test_hdr_mean():
    """Test HdrHistogram mean computation"""
    latencies = [10.0, 20.0, 30.0, 40.0, 50.0]
    df = pl.DataFrame({"latency": latencies})

    result = df.select(mean=hdr_mean("latency"))
    mean_val = result["mean"][0]

    # Expected mean: (10 + 20 + 30 + 40 + 50) / 5 = 30
    assert abs(mean_val - 30.0) < 1.0  # Allow small floating point variance


def test_hdr_percentile():
    """Test HdrHistogram percentile computation"""
    latencies = [10.0, 20.0, 30.0, 40.0, 50.0]
    df = pl.DataFrame({"latency": latencies})

    result_p50 = df.select(p50=hdr_percentile("latency", 50.0))
    result_p95 = df.select(p95=hdr_percentile("latency", 95.0))

    p50_val = result_p50["p50"][0]
    p95_val = result_p95["p95"][0]

    # Percentiles should be within the recorded range
    assert 10.0 <= p50_val <= 50.0
    assert 10.0 <= p95_val <= 50.0
    # P95 should be higher than P50
    assert p95_val >= p50_val


def test_hdr_stats_summary():
    """Test HdrHistogram stats summary"""
    latencies = [10.0, 20.0, 30.0, 40.0, 50.0]
    df = pl.DataFrame({"latency": latencies})

    result = df.select(stats=hdr_stats_summary("latency"))
    summary_str = result["stats"][0]

    # Check that summary contains expected fields
    assert "count=" in summary_str
    assert "min=" in summary_str
    assert "max=" in summary_str
    assert "mean=" in summary_str

    # Verify it can be parsed back
    assert "count=5" in summary_str
