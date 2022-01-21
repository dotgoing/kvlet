use anyhow::Result;
use chrono::prelude::*;
use rusqlite::named_params;
use rusqlite::{Connection, Rows};
use tabled::Tabled;

/// 输出给用户可见的对象
#[derive(Debug, Tabled)]
pub struct OutRecord {
    pub id: String,
    pub state: String,
    pub method: String,
    pub url: String,
    pub response_code: String,
    pub response: String,
    pub create_at: String,
    pub update_at: String,
}

pub struct InRecord {
    pub id: String,
    pub state: String,
    pub notify: Option<Notify>,
}

/// 是否要通知
pub struct Notify {
    pub method: String,
    pub url: String,
}

/// 数据库每一行对应的对象
#[derive(Debug)]
struct TableItem {
    pub id: String,
    pub state: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub response_code: Option<usize>,
    pub response: Option<String>,
    pub create_at: i64,
    pub update_at: i64,
}

impl TableItem {
    fn show(self) -> OutRecord {
        OutRecord {
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
             method TEXT,
             url TEXT,
             response_code INTEGER,
             response TEXT,
             create_at INTEGER not null,
             update_at INTEGER not null
         )",
        [],
    )?;
    Ok(conn)
}

pub fn set(record: InRecord) -> Result<usize> {
    let conn = get_db()?;
    let now = Local::now().timestamp_millis();
    let mut stmt = conn.prepare("INSERT INTO kvlet (id, state,method,url, create_at,update_at) VALUES (:id, :state,:method,:url, :create_at,:update_at)")?;
    let InRecord { id, state, notify } = record;
    let affect = match notify {
        Some(Notify { method, url }) => stmt.execute(named_params! {
           ":id": id,
           ":state": state,
           ":method": method,
           ":url": url,
           ":create_at": now,
           ":update_at": now
        })?,
        None => stmt.execute(named_params! {
           ":id": id,
           ":state": state,
           ":method": "",
           ":url": "",
           ":create_at": now,
           ":update_at": now
        })?,
    };
    Ok(affect)
}

pub fn get(id: &str) -> Result<Option<OutRecord>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet where id = :id")?;
    let mut rows = stmt.query(&[(":id", id)])?;
    let records = parse_rows(&mut rows)?;
    for record in records {
        return Ok(Some(record.show()));
    }
    Ok(None)
}

pub fn list(num: usize) -> Result<Vec<OutRecord>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet ORDER BY create_at DESC limit :num")?;
    let mut rows = stmt.query(&[(":num", &num.to_string())])?;
    let records = parse_rows(&mut rows)?;
    Ok(records.into_iter().map(|r| r.show()).collect())
}

fn parse_rows(rows: &mut Rows) -> Result<Vec<TableItem>> {
    let mut records = vec![];
    while let Some(row) = rows.next()? {
        records.push(TableItem {
            id: row.get(0)?,
            state: row.get(1)?,
            method: row.get(2)?,
            url: row.get(3)?,
            response_code: None,
            response: None,
            create_at: row.get(6)?,
            update_at: row.get(7)?,
        });
    }
    Ok(records)
}
