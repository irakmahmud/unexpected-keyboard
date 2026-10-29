#!/usr/bin/env bash
set -euo pipefail

mkdir -p dist

echo "" >> dist/SCALE_AND_POSTGRES_VERIFICATION.txt
echo "================================================================================" >> dist/SCALE_AND_POSTGRES_VERIFICATION.txt
echo "LIVE POSTGRESQL 16 DECLARATIVE RANGE + HASH PARTITIONING & MIGRATION TEST" >> dist/SCALE_AND_POSTGRES_VERIFICATION.txt
echo "================================================================================" >> dist/SCALE_AND_POSTGRES_VERIFICATION.txt

psql -h 127.0.0.1 -U postgres -d attendance_pg_test << 'EOSQL' | tee -a dist/SCALE_AND_POSTGRES_VERIFICATION.txt
SELECT version();

CREATE TABLE attendance_records (
    record_id TEXT NOT NULL,
    username TEXT NOT NULL,
    date DATE NOT NULL,
    in_time TEXT,
    out_time TEXT,
    status TEXT NOT NULL DEFAULT 'PRESENT',
    partition_year INTEGER NOT NULL,
    partition_month INTEGER NOT NULL,
    shard_bucket INTEGER NOT NULL,
    storage_tier TEXT NOT NULL DEFAULT 'HOT',
    PRIMARY KEY (partition_year, partition_month, shard_bucket, record_id)
) PARTITION BY RANGE (partition_year, partition_month);

CREATE INDEX idx_pg_att_user_date ON attendance_records (username, date) INCLUDE (status, in_time, out_time);
CREATE INDEX idx_pg_att_cursor ON attendance_records (date DESC, record_id DESC);

CREATE TABLE attendance_records_y1977_m10 PARTITION OF attendance_records
    FOR VALUES FROM (1977, 10) TO (1977, 11) PARTITION BY HASH (shard_bucket);
CREATE TABLE attendance_records_y1977_m10_s00 PARTITION OF attendance_records_y1977_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 0);
CREATE TABLE attendance_records_y1977_m10_s01 PARTITION OF attendance_records_y1977_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 1);
CREATE TABLE attendance_records_y1977_m10_s02 PARTITION OF attendance_records_y1977_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 2);
CREATE TABLE attendance_records_y1977_m10_s03 PARTITION OF attendance_records_y1977_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 3);

CREATE TABLE attendance_records_y2026_m10 PARTITION OF attendance_records
    FOR VALUES FROM (2026, 10) TO (2026, 11) PARTITION BY HASH (shard_bucket);
CREATE TABLE attendance_records_y2026_m10_s00 PARTITION OF attendance_records_y2026_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 0);
CREATE TABLE attendance_records_y2026_m10_s01 PARTITION OF attendance_records_y2026_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 1);
CREATE TABLE attendance_records_y2026_m10_s02 PARTITION OF attendance_records_y2026_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 2);
CREATE TABLE attendance_records_y2026_m10_s03 PARTITION OF attendance_records_y2026_m10 FOR VALUES WITH (MODULUS 4, REMAINDER 3);

INSERT INTO attendance_records (record_id, username, date, in_time, out_time, status, partition_year, partition_month, shard_bucket, storage_tier)
SELECT 'EMP' || g || '_2026-10-29', 'EMP' || g, DATE '2026-10-29', '09:00:00', '17:00:00', 'PRESENT', 2026, 10, (g % 64), 'HOT'
FROM generate_series(1, 50000) AS g;

INSERT INTO attendance_records (record_id, username, date, in_time, out_time, status, partition_year, partition_month, shard_bucket, storage_tier)
SELECT 'EMP' || g || '_1977-10-29', 'EMP' || g, DATE '1977-10-29', '09:00:00', '17:00:00', 'PRESENT', 1977, 10, (g % 64), 'COLD'
FROM generate_series(1, 50000) AS g;

ANALYZE attendance_records;

SELECT tableoid::regclass AS physical_pg_subpartition, COUNT(*) AS rows_in_subpartition
FROM attendance_records
GROUP BY tableoid
ORDER BY physical_pg_subpartition;

EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
SELECT record_id, username, date, status
FROM attendance_records
WHERE partition_year = 2026 AND partition_month = 10 AND shard_bucket = 27 AND username = 'EMP27';
EOSQL
