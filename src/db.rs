use crate::settings::Settings;
use rusqlite::{Connection, OptionalExtension};
use std::path::PathBuf;

const APP_TABLE_SETTINGS: &str = "app_settings";
const APP_TABLE_COUNTERS: &str = "app_counters";

fn db_path() -> PathBuf {
    match std::env::var("XDG_DATA_HOME") {
        Ok(data_home) if !data_home.is_empty() => PathBuf::from(data_home)
            .join("roth-pomodoro")
            .join("roth-pomodoro.sqlite"),
        _ => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("roth-pomodoro")
                .join("roth-pomodoro.sqlite")
        }
    }
}

fn open() -> rusqlite::Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    Connection::open(path)
}

fn init(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {APP_TABLE_SETTINGS} (\
                id INTEGER PRIMARY KEY CHECK (id = 1),\
                work_seconds INTEGER NOT NULL,\
                short_break_seconds INTEGER NOT NULL,\
                long_break_seconds INTEGER NOT NULL,\
                long_break_every INTEGER NOT NULL\
            )"
        ),
        (),
    )?;

    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {APP_TABLE_COUNTERS} (\
                id INTEGER PRIMARY KEY CHECK (id = 1),\
                completed_pomodoros INTEGER NOT NULL\
            )"
        ),
        (),
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO {APP_TABLE_SETTINGS} \
                (id, work_seconds, short_break_seconds, long_break_seconds, long_break_every) \
             VALUES (1, ?1, ?2, ?3, ?4)"
        ),
        (
            Settings::default().work_seconds,
            Settings::default().short_break_seconds,
            Settings::default().long_break_seconds,
            Settings::default().long_break_every,
        ),
    )?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO {APP_TABLE_COUNTERS} (id, completed_pomodoros) VALUES (1, 0)"
        ),
        (),
    )?;

    Ok(())
}

pub fn load_settings() -> Settings {
    let Ok(conn) = open() else {
        return Settings::default();
    };
    if init(&conn).is_err() {
        return Settings::default();
    }

    let row = conn
        .query_row(
            &format!(
                "SELECT work_seconds, short_break_seconds, long_break_seconds, long_break_every \
                 FROM {APP_TABLE_SETTINGS} WHERE id = 1"
            ),
            (),
            |r| {
                Ok(Settings {
                    work_seconds: r.get::<_, i64>(0)? as u32,
                    short_break_seconds: r.get::<_, i64>(1)? as u32,
                    long_break_seconds: r.get::<_, i64>(2)? as u32,
                    long_break_every: r.get::<_, i64>(3)? as u32,
                })
            },
        )
        .optional();

    match row {
        Ok(Some(settings)) if settings.long_break_every > 0 => settings,
        _ => Settings::default(),
    }
}

pub fn save_settings(settings: Settings) {
    let Ok(conn) = open() else {
        return;
    };
    if init(&conn).is_err() {
        return;
    }

    let _ = conn.execute(
        &format!(
            "UPDATE {APP_TABLE_SETTINGS} \
             SET work_seconds = ?1, short_break_seconds = ?2, long_break_seconds = ?3, long_break_every = ?4 \
             WHERE id = 1"
        ),
        (
            settings.work_seconds,
            settings.short_break_seconds,
            settings.long_break_seconds,
            settings.long_break_every,
        ),
    );
}

pub fn load_completed_pomodoros() -> u32 {
    let Ok(conn) = open() else {
        return 0;
    };
    if init(&conn).is_err() {
        return 0;
    }

    let row: rusqlite::Result<Option<u32>> = conn
        .query_row(
            &format!("SELECT completed_pomodoros FROM {APP_TABLE_COUNTERS} WHERE id = 1"),
            (),
            |r| Ok(r.get::<_, i64>(0)? as u32),
        )
        .optional();

    row.ok().flatten().unwrap_or(0)
}

pub fn save_completed_pomodoros(completed: u32) {
    let Ok(conn) = open() else {
        return;
    };
    if init(&conn).is_err() {
        return;
    }

    let _ = conn.execute(
        &format!("UPDATE {APP_TABLE_COUNTERS} SET completed_pomodoros = ?1 WHERE id = 1"),
        (completed,),
    );
}
