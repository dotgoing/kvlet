use anyhow::Result;
use chrono::prelude::*;
use rusqlite::named_params;
use rusqlite::Connection;
use tabled::Tabled;

#[derive(Debug, Tabled)]
pub struct Record {
    pub id: String,
    pub state: String,
    pub method: String,
    pub url: String,
    pub response_code: String,
    pub response: String,
    pub create_at: String,
    pub update_at: String,
}

#[derive(Debug)]
struct Item {
    pub id: String,
    pub state: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub response_code: Option<usize>,
    pub response: Option<String>,
    pub create_at: i64,
    pub update_at: i64,
}

impl Item {
    fn show(self) -> Record {
        Record {
            id: self.id,
            state: self.state,
            method: self.method.unwrap_or_else(|| "".to_string()),
            url: self.url.unwrap_or_else(|| "".to_string()),
            response_code: self
                .response_code
                .map(|it| it.to_string())
                .unwrap_or_else(|| "".to_string()),
            response: self.response.unwrap_or_else(|| "".to_string()),
            create_at: Self::from_time(self.create_at),
            update_at: Self::from_time(self.update_at),
        }
    }

    fn from_time(time: i64) -> String {
        Local.timestamp_millis(time).to_string()
    }
}

fn get_db() -> Result<Connection> {
    let conn = Connection::open("kvlet.db")?;
    conn.execute(
        "create table if not exists kvlet (
             id TEXT primary key,
             state TEXT not null,
             url TEXT,
             method TEXT,
             response_code INTEGER,
             response TEXT,
             create_at INTEGER not null,
             update_at INTEGER not null
         )",
        [],
    )?;
    Ok(conn)
}

pub fn set(id: &str, state: &str) -> Result<()> {
    let conn = get_db()?;
    let now = Local::now().timestamp_millis();
    let mut stmt = conn
        .prepare("INSERT INTO kvlet (id, state, create_at,update_at) VALUES (:id, :state, :create_at,:update_at)")?;
    stmt.execute(named_params! {
       ":id": id,
       ":state": state,
       ":create_at": now,
       ":update_at": now
    })?;
    Ok(())
}

pub fn get(id: &str) -> Result<Option<Record>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet where id = :id")?;
    let mut rows = stmt.query(&[(":id", id)])?;
    let mut records = vec![];

    while let Some(row) = rows.next()? {
        let r = Item {
            id: row.get(0)?,
            state: row.get(1)?,
            url: None,
            method: None,
            response_code: None,
            response: None,
            create_at: row.get(6)?,
            update_at: row.get(7)?,
        };
        records.push(r);
    }

    for record in records {
        return Ok(Some(record.show()));
    }
    Ok(None)
}

pub fn list(num: usize) -> Result<Vec<Record>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet ORDER BY create_at DESC limit :num")?;
    let mut rows = stmt.query(&[(":num", &num.to_string())])?;
    let mut records = vec![];
    while let Some(row) = rows.next()? {
        let r = Item {
            id: row.get(0)?,
            state: row.get(1)?,
            url: None,
            method: None,
            response_code: None,
            response: None,
            create_at: row.get(6)?,
            update_at: row.get(7)?,
        };
        records.push(r);
    }
    Ok(records.into_iter().map(|r| r.show()).collect())
}
