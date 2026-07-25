use crate::core::state::{ChatMessage, Role};
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};

pub struct Store {
    conn: Connection,
}

pub struct Loaded {
    pub messages: Vec<ChatMessage>,
    pub summary: Option<String>,
    pub summarized_upto: usize,
}

impl Store {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS messages (pos INTEGER PRIMARY KEY, role TEXT NOT NULL, content TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        Ok(Self { conn })
    }

    pub fn load(&self) -> Result<Loaded> {
        let mut stmt = self
            .conn
            .prepare("SELECT role, content FROM messages ORDER BY pos")?;
        let rows = stmt.query_map([], |r| {
            Ok(ChatMessage {
                role: if r.get::<_, String>(0)? == "user" {
                    Role::User
                } else {
                    Role::Assistant
                },
                content: r.get(1)?,
                activity: Vec::new(),
            })
        })?;
        let messages = rows.collect::<Result<Vec<_>, _>>()?;

        let summary = self.get_meta("summary")?.filter(|s| !s.is_empty());
        let summarized_upto = self
            .get_meta("summarized_upto")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        Ok(Loaded {
            messages,
            summary,
            summarized_upto,
        })
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let value = self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?;
        Ok(value)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        Ok(())
    }

    // ponytail: snapshot completo por turno; O(n) escrituras, sobra hasta miles de mensajes
    pub fn save(
        &self,
        messages: &[ChatMessage],
        summary: Option<&str>,
        summarized_upto: usize,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM messages", [])?;
        {
            let mut stmt =
                tx.prepare("INSERT INTO messages (pos, role, content) VALUES (?1, ?2, ?3)")?;
            for (pos, m) in messages.iter().enumerate() {
                let role = match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                stmt.execute(params![pos as i64, role, m.content])?;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('summary', ?1)",
            [summary.unwrap_or("")],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('summarized_upto', ?1)",
            [summarized_upto.to_string()],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.conn
            .execute_batch("DELETE FROM messages; DELETE FROM meta;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let store = Store::open(":memory:").unwrap();
        let msgs = vec![
            ChatMessage { role: Role::User, content: "hola".into(), activity: Vec::new() },
            ChatMessage { role: Role::Assistant, content: "buenas".into(), activity: Vec::new() },
        ];
        store.save(&msgs, Some("resumen previo"), 1).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.messages.len(), 2);
        assert!(matches!(loaded.messages[0].role, Role::User));
        assert_eq!(loaded.messages[1].content, "buenas");
        assert_eq!(loaded.summary.as_deref(), Some("resumen previo"));
        assert_eq!(loaded.summarized_upto, 1);

        store.clear().unwrap();
        let empty = store.load().unwrap();
        assert!(empty.messages.is_empty());
        assert!(empty.summary.is_none());
        assert_eq!(empty.summarized_upto, 0);
    }
}
