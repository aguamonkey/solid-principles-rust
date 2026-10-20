use solid_workbook::console::{
    exports,
    http::{self, AppState},
    memory::{MemoryJournal, MemoryPreviews},
    sqlite::{SqliteJournal, SqlitePreviews},
};
use std::{path::PathBuf, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut port = 5190_u16;
    let mut database = PathBuf::from("data/releases.sqlite3");
    let mut memory = false;
    let mut database_selected = false;
    let mut markdown = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" => port = args.next().ok_or("Missing port")?.parse()?,
            "--database" => {
                database_selected = true;
                database = PathBuf::from(args.next().ok_or("Missing database path")?);
            }
            "--memory" => memory = true,
            "--markdown" => markdown = true,
            "--help" => {
                println!("release-console [--port 5190] [--database data/releases.sqlite3 | --memory] [--markdown]");
                return Ok(());
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    if memory && database_selected {
        return Err("Choose --memory or --database, not both".into());
    }
    let reports = Arc::new(exports::defaults(markdown));
    let state = if memory {
        AppState {
            journal: Arc::new(MemoryJournal::default()),
            previews: Arc::new(MemoryPreviews::default()),
            reports,
        }
    } else {
        if let Some(parent) = database
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let journal = Arc::new(SqliteJournal::open(&database)?);
        AppState {
            previews: Arc::new(SqlitePreviews(journal.clone())),
            journal,
            reports,
        }
    };
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    println!(
        "Release Console: http://{}/ · {} storage · {}",
        listener.local_addr()?,
        if memory { "ephemeral memory" } else { "SQLite" },
        if markdown {
            "Markdown extension enabled"
        } else {
            "JSON + text reports"
        }
    );
    axum::serve(listener, http::router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
