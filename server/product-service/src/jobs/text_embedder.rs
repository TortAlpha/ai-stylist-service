//! Scheduled text embedder.
//!
//! Runs in its own process (`SERVICE_MODE=text-embedder`). On each tick:
//!  1. Pull a batch of product ids whose `text_hash` is missing or stale.
//!  2. Recompute SHA256 over `generate_product_text(id)`.
//!  3. Drop rows where the hash matches — those don't need re-embedding.
//!  4. Send the remaining texts to the embedder in one batch.
//!  5. UPSERT vectors into `product_text_embeddings`.
//!
//! A daily counter caps total Cohere calls so a runaway loop can't blow
//! through the quota.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use pgvector::Vector;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::TextEmbedderConfig;
use crate::embeddings::MultimodalEmbedder;

pub struct TextEmbedderWorker {
    pool: PgPool,
    embedder: Arc<dyn MultimodalEmbedder>,
    cfg: TextEmbedderConfig,
}

impl TextEmbedderWorker {
    pub fn new(
        pool: PgPool,
        embedder: Arc<dyn MultimodalEmbedder>,
        cfg: TextEmbedderConfig,
    ) -> Self {
        Self {
            pool,
            embedder,
            cfg,
        }
    }

    /// Run forever. Returns only on fatal init errors.
    pub async fn run(self) {
        let mut counter = DailyCounter::new(Utc::now());
        info!(
            batch_size = self.cfg.batch_size,
            interval_s = self.cfg.interval.as_secs(),
            daily_cap = self.cfg.daily_cap,
            "text_embedder.start"
        );
        loop {
            if let Err(e) = self.tick(&mut counter).await {
                warn!(error = %e, "text_embedder.tick_failed");
            }
            tokio::time::sleep(self.cfg.interval).await;
        }
    }

    async fn tick(&self, counter: &mut DailyCounter) -> Result<(), Box<dyn std::error::Error>> {
        counter.maybe_roll_over(Utc::now());
        if counter.remaining(self.cfg.daily_cap) == 0 {
            debug!("text_embedder.tick_skip_cap");
            return Ok(());
        }

        let batch_size = self.cfg.batch_size.min(counter.remaining(self.cfg.daily_cap));
        let candidates = self.fetch_candidates(batch_size as i64).await?;
        if candidates.is_empty() {
            debug!("text_embedder.tick_empty");
            return Ok(());
        }

        // Filter out unchanged hashes before paying for an embed call.
        let mut to_embed: Vec<Candidate> = Vec::with_capacity(candidates.len());
        for c in candidates {
            if Some(c.text_hash.as_str()) == c.stored_hash.as_deref() {
                continue;
            }
            to_embed.push(c);
        }
        if to_embed.is_empty() {
            debug!("text_embedder.all_unchanged");
            return Ok(());
        }

        let texts: Vec<String> = to_embed.iter().map(|c| c.text.clone()).collect();
        let vectors = self.embedder.embed_text(&texts).await?;
        if vectors.len() != to_embed.len() {
            warn!(
                got = vectors.len(),
                expected = to_embed.len(),
                "text_embedder.size_mismatch"
            );
            return Ok(());
        }

        let mut written = 0usize;
        for (c, raw) in to_embed.iter().zip(vectors.into_iter()) {
            let pgv = Vector::from(raw);
            sqlx::query(
                r#"
                INSERT INTO product_text_embeddings (product_id, embedding, text_hash, updated_at)
                VALUES ($1, $2, $3, now())
                ON CONFLICT (product_id) DO UPDATE
                    SET embedding  = EXCLUDED.embedding,
                        text_hash  = EXCLUDED.text_hash,
                        updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(c.product_id)
            .bind(pgv)
            .bind(&c.text_hash)
            .execute(&self.pool)
            .await?;
            written += 1;
        }

        counter.add(written);
        info!(written, "text_embedder.batch_done");
        Ok(())
    }

    async fn fetch_candidates(&self, limit: i64) -> Result<Vec<Candidate>, sqlx::Error> {
        // Stale rules:
        //  - product exists, ready, not deleted
        //  - either no row in product_text_embeddings yet, OR product.updated_at
        //    is newer than the stored embedding row.
        //
        // We compute generate_product_text(id) in-DB to avoid round-tripping
        // the whole product row to the worker.
        let rows: Vec<DbRow> = sqlx::query_as::<_, DbRow>(
            r#"
            SELECT
                p.id                                AS product_id,
                generate_product_text(p.id)         AS text,
                pte.text_hash                       AS stored_hash
            FROM product p
            LEFT JOIN product_text_embeddings pte ON pte.product_id = p.id
            WHERE p.status = 'ready'
              AND p.is_deleted = false
              AND (pte.product_id IS NULL OR pte.updated_at < p.updated_at)
            ORDER BY p.updated_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let text = r.text.unwrap_or_default();
                let text_hash = sha256_hex(text.as_bytes());
                Candidate {
                    product_id: r.product_id,
                    text,
                    text_hash,
                    stored_hash: r.stored_hash,
                }
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct DbRow {
    product_id: Uuid,
    text: Option<String>,
    stored_hash: Option<String>,
}

struct Candidate {
    product_id: Uuid,
    text: String,
    text_hash: String,
    stored_hash: Option<String>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Rolls over at UTC midnight. Worker-local — no DB round trip per tick.
struct DailyCounter {
    day: i64,
    used: usize,
}

impl DailyCounter {
    fn new(now: DateTime<Utc>) -> Self {
        Self {
            day: now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
            used: 0,
        }
    }

    fn maybe_roll_over(&mut self, now: DateTime<Utc>) {
        let today = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        if today != self.day {
            self.day = today;
            self.used = 0;
        }
    }

    fn remaining(&self, cap: usize) -> usize {
        cap.saturating_sub(self.used)
    }

    fn add(&mut self, n: usize) {
        self.used = self.used.saturating_add(n);
    }
}

/// Used by the next-tick log to advertise how long until restart.
pub const POLL_FLOOR: Duration = Duration::from_secs(5);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_stable() {
        let a = sha256_hex(b"hello");
        let b = sha256_hex(b"hello");
        assert_eq!(a, b);
        assert_ne!(a, sha256_hex(b"world"));
        // Sanity: hex length = 64.
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn daily_counter_rolls_over_on_new_day() {
        let day1 = "2026-05-19T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let day2 = "2026-05-20T01:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let mut c = DailyCounter::new(day1);
        c.add(7);
        assert_eq!(c.remaining(10), 3);
        c.maybe_roll_over(day2);
        assert_eq!(c.remaining(10), 10);
    }
}
