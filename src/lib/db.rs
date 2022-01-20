use anyhow::{anyhow, Result};
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
}

#[derive(Debug)]
struct Item {
    pub id: String,
    pub state: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub response_code: Option<usize>,
    pub response: Option<String>,
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
        }
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
             response TEXT
         )",
        [],
    )?;
    Ok(conn)
}

pub fn set(id: &str, state: &str) -> Result<()> {
    let conn = get_db()?;
    let err = |e| anyhow!(format!("fail to save {}, {}", id, e));
    let mut stmt = conn
        .prepare("INSERT INTO kvlet (id, state) VALUES (:id, :state)")
        .map_err(err)?;
    stmt.execute(named_params! { ":id": id, ":state": state })
        .map_err(err)?;
    Ok(())
}

pub fn get(id: &str) -> Result<Vec<Record>> {
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
        };
        records.push(r);
    }
    Ok(records.into_iter().map(|r| r.show()).collect())
}

pub fn list(num: usize) -> Result<Vec<Record>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet limit :num")?;
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
        };
        records.push(r);
    }
    Ok(records.into_iter().map(|r| r.show()).collect())
}
