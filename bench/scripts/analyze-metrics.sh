#!/bin/bash
# Analyze metrics collected during benchmark run

set -e

METRICS_FILE="${1:-metrics.log}"

if [ ! -f "${METRICS_FILE}" ]; then
    echo "Error: Metrics file not found: ${METRICS_FILE}"
    echo "Usage: $0 [metrics_file]"
    exit 1
fi

echo "=== Metrics Analysis ==="
echo "File: ${METRICS_FILE}"
echo ""

# Count samples
SAMPLE_COUNT=$(wc -l < "${METRICS_FILE}")
echo "Samples collected: ${SAMPLE_COUNT}"

if [ ${SAMPLE_COUNT} -eq 0 ]; then
    echo "No data to analyze"
    exit 0
fi

echo ""
echo "=== CPU Usage (%) ==="
awk -F',' '{print $2}' "${METRICS_FILE}" | awk '
{
    sum += $1
    if (NR == 1 || $1 < min) min = $1
    if (NR == 1 || $1 > max) max = $1
    samples[NR] = $1
}
END {
    avg = sum / NR
    printf "  Min: %.2f%%\n", min
    printf "  Max: %.2f%%\n", max
    printf "  Avg: %.2f%%\n", avg
    
    # Count samples above thresholds
    count_80 = 0
    count_90 = 0
    count_100 = 0
    for (i = 1; i <= NR; i++) {
        if (samples[i] > 80) count_80++
        if (samples[i] > 90) count_90++
        if (samples[i] >= 100) count_100++
    }
    printf "  >80%%: %d samples (%.1f%%)\n", count_80, (count_80/NR)*100
    printf "  >90%%: %d samples (%.1f%%)\n", count_90, (count_90/NR)*100
    printf "  100%%: %d samples (%.1f%%)\n", count_100, (count_100/NR)*100
}'

echo ""
echo "=== Memory Usage (RSS in KB) ==="
awk -F',' '{print $3}' "${METRICS_FILE}" | awk '
{
    sum += $1
    if (NR == 1) {
        min = $1
        max = $1
        first = $1
    }
    if ($1 < min) min = $1
    if ($1 > max) max = $1
    last = $1
}
END {
    avg = sum / NR
    printf "  Min: %.0f KB (%.2f MB)\n", min, min/1024
    printf "  Max: %.0f KB (%.2f MB)\n", max, max/1024
    printf "  Avg: %.0f KB (%.2f MB)\n", avg, avg/1024
    printf "  First: %.0f KB (%.2f MB)\n", first, first/1024
    printf "  Last: %.0f KB (%.2f MB)\n", last, last/1024
    growth = last - first
    growth_pct = (growth / first) * 100
    printf "  Growth: %.0f KB (%.2f MB, %.1f%%)\n", growth, growth/1024, growth_pct
}'

echo ""
echo "=== Duration ==="
FIRST_TS=$(head -1 "${METRICS_FILE}" | cut -d',' -f1)
LAST_TS=$(tail -1 "${METRICS_FILE}" | cut -d',' -f1)
DURATION=$((LAST_TS - FIRST_TS))
HOURS=$((DURATION / 3600))
MINUTES=$(((DURATION % 3600) / 60))
SECONDS=$((DURATION % 60))
echo "  Total: ${DURATION}s (${HOURS}h ${MINUTES}m ${SECONDS}s)"
echo ""

# Pass/fail assessment
echo "=== Pass/Fail Assessment ==="
MAX_CPU=$(awk -F',' '{print $2}' "${METRICS_FILE}" | sort -n | tail -1)
AVG_CPU=$(awk -F',' '{sum += $2} END {print sum/NR}' "${METRICS_FILE}")
MEM_GROWTH=$(awk -F',' 'NR==1{first=$3} END{print (($3-first)/first)*100}' "${METRICS_FILE}")

echo "  CPU avg <80%: $(awk -v avg="${AVG_CPU}" 'BEGIN {print (avg < 80) ? "PASS" : "FAIL"}')"
echo "  No sustained 100% CPU: $(awk -F',' '{if ($2 >= 100) count++} END {print (count < 10) ? "PASS" : "FAIL"}' "${METRICS_FILE}")"
echo "  Memory stable (<20% growth): $(awk -v growth="${MEM_GROWTH}" 'BEGIN {print (growth < 20) ? "PASS" : "FAIL"}')"
echo ""
