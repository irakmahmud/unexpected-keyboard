# Google BigQuery Setup & Migration Guide
## Architecture: SQLite (Authoritative Operational) + BigQuery (Analytics & Outbox)

The pure Rust Axum Attendance System replaces AWS RDS and S3 by designating local **SQLite (with WAL mode)** as the authoritative transactional database, and streaming event changes to **Google BigQuery** for cloud analytics and durable reporting using a **Transactional Outbox Pattern**.

---

### 1. Selecting or Creating a Google Cloud Project
Run the following in the Google Cloud SDK CLI (`gcloud`) or Google Cloud Console:
```bash
# Create a new project (or select an existing one)
gcloud projects create attendance-analytics-prod --name="Attendance Analytics"
gcloud config set project attendance-analytics-prod
```

### 2. Enabling the BigQuery API
Enable the required BigQuery and BigQuery Storage APIs:
```bash
gcloud services enable bigquery.googleapis.com
gcloud services enable bigquerystorage.googleapis.com
```

### 3. Creating a BigQuery Dataset
Create the analytics dataset:
```bash
bq --location=US mk -d \
   --description "Attendance System Analytics & Historical Outbox Data" \
   attendance-analytics-prod:attendance_analytics
```

### 4. Choosing the Dataset Location
- Default: `US` (multi-region) or `asia-south1` (Mumbai) / `asia-southeast1` (Singapore) for lowest latency to Bangladesh (`Asia/Dhaka`).
- Set in configuration: `BIGQUERY_LOCATION=US` or `BIGQUERY_LOCATION=asia-south1`.

### 5. Configuring IAM Permissions & Service Account
Create a dedicated least-privilege service account:
```bash
# Create Service Account
gcloud iam service-accounts create attendance-bq-sync \
    --description="Service account for Attendance System Outbox Sync" \
    --display-name="attendance-bq-sync"

# Grant BigQuery Data Editor and Job User permissions
gcloud projects add-iam-policy-binding attendance-analytics-prod \
    --member="serviceAccount:attendance-bq-sync@attendance-analytics-prod.iam.gserviceaccount.com" \
    --role="roles/bigquery.dataEditor"

gcloud projects add-iam-policy-binding attendance-analytics-prod \
    --member="serviceAccount:attendance-bq-sync@attendance-analytics-prod.iam.gserviceaccount.com" \
    --role="roles/bigquery.jobUser"
```

### 6. Service Account Key & Application Default Credentials (ADC)
Generate the key file and securely place it on the server:
```bash
gcloud iam service-accounts keys create /home/opc/attendance_system/config/gcp-service-account.json \
    --iam-account=attendance-bq-sync@attendance-analytics-prod.iam.gserviceaccount.com

# Restrict permissions
chmod 600 /home/opc/attendance_system/config/gcp-service-account.json
```

### 7. Environment Variables Configuration
Set the configuration in `.env` or systemd service:
```bash
DATABASE_URL=sqlite:///home/opc/attendance_system/data/attendance.db
PORT=8080
GOOGLE_CLOUD_PROJECT=attendance-analytics-prod
BIGQUERY_DATASET=attendance_analytics
BIGQUERY_LOCATION=US
GOOGLE_APPLICATION_CREDENTIALS=/home/opc/attendance_system/config/gcp-service-account.json
```

### 8. Testing Authentication & BigQuery Streaming Ingestion
Test authentication using `gcloud` and BigQuery CLI:
```bash
# Test authentication
gcloud auth activate-service-account --key-file=/home/opc/attendance_system/config/gcp-service-account.json

# Test query execution
bq query --use_legacy_sql=false 'SELECT CURRENT_TIMESTAMP() AS current_time;'

# Test triggering the Rust sync worker
curl -X POST http://127.0.0.1:8080/api/v1/sync/trigger
```

### 9. BigQuery Analytics Tables Created by Rust Outbox
The sync worker automatically streams rows into:
1. `attendance_records_analytics`:
   - `record_id STRING`, `username STRING`, `date DATE`, `in_time STRING`, `out_time STRING`, `with_camera STRING`, `_synced_at TIMESTAMP`
2. `users_analytics`:
   - `company_user_uuid STRING`, `username STRING`, `user_designation STRING`, `created_at TIMESTAMP`
