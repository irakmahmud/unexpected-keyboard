mod commands;

use attendance_common::{AppConfig, Result};
use attendance_database::DbPool;
use commands::backup::BackupCommand;
use commands::diagnose::DiagnoseCommand;
use commands::export::ExportCommand;
use commands::inspect::InspectCommand;
use commands::migrate::MigrateCommand;
use commands::seed::SeedCommand;
use commands::sync::SyncCommand;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = args[1].as_str();

    if command == "help" || command == "--help" || command == "-h" {
        print_usage();
        return Ok(());
    }

    if command == "version" || command == "--version" || command == "-v" {
        println!("attendance-admin v0.1.0 (Pure Rust Axum Enterprise Attendance System)");
        return Ok(());
    }

    let config = AppConfig::default();
    let db = DbPool::new(&config.database_url.replace("sqlite://", ""))?;

    match command {
        "migrate" => {
            MigrateCommand::execute(&db)?;
        }
        "seed" => {
            SeedCommand::execute(&db)?;
        }
        "backup" => {
            let target_dir = args.get(2).map(|s| s.as_str());
            BackupCommand::execute(&db, target_dir)?;
        }
        "sync" => {
            SyncCommand::execute(&db)?;
        }
        "export" => {
            let month = args.get(2).map(|s| s.as_str());
            let format = args.get(3).map(|s| s.as_str());
            ExportCommand::execute(&db, month, format)?;
        }
        "diagnose" => {
            DiagnoseCommand::execute(&db)?;
        }
        "inspect" => {
            InspectCommand::execute(&db)?;
        }
        unknown => {
            eprintln!("Unknown command: '{}'", unknown);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        r#"
================================================================================
          ATTENDANCE SYSTEM ENTERPRISE MANAGEMENT UTILITY (CLI)
================================================================================
USAGE:
    attendance-admin <COMMAND> [OPTIONS]

COMMANDS:
    migrate             Check and apply SQLite schema migrations and run integrity check
    seed                Seed demo enterprise company, shifts, users, and 10 employees
    backup [DIR]        Perform online hot backup using SQLite VACUUM INTO
    sync                Inspect and trigger Google BigQuery outbox queue synchronization
    export [MONTH] [FMT] Batch export Form-12 Muster Roll (e.g. export 2026-10 html|csv)
    diagnose            Run complete hardware, camera, disk, and timezone diagnostic audit
    inspect             Inspect 512-D face biometric vector embeddings and HNSW metrics
    version             Print application version and pure-Rust architecture details
    help                Show this help information

EXAMPLES:
    attendance-admin migrate
    attendance-admin seed
    attendance-admin export 2026-10 html
    attendance-admin backup ./data/backups
    attendance-admin diagnose
================================================================================
"#
    );
}
