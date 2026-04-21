use crate::db::timestamp::now_iso;
use crate::models::device::{Device, DeviceSnapshotEntry};
use sqlx::SqlitePool;

/// CRUD de la table `devices` + application d'un snapshot complet par hub.
#[derive(Clone)]
pub struct DeviceRepository {
    pool: SqlitePool,
}

impl DeviceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Remplace l'état des devices pour un `hub_id` donné.
    /// Les MACs présentes dans `entries` sont upsertées (connected=1, last_seen_at rafraîchi).
    /// Les MACs précédemment connectées et absentes du snapshot sont marquées connected=0.
    ///
    /// Retourne le nombre de devices connectés après application.
    pub async fn apply_snapshot(
        &self,
        hub_id: &str,
        entries: &[DeviceSnapshotEntry],
    ) -> Result<usize, sqlx::Error> {
        let ts = now_iso();
        let mut tx = self.pool.begin().await?;

        // 1) marquer tous les devices du hub comme déconnectés — on rallumera ceux vus
        sqlx::query("UPDATE devices SET connected = 0 WHERE hub_id = ?")
            .bind(hub_id)
            .execute(&mut *tx)
            .await?;

        // 2) upsert chaque device vu
        for e in entries {
            let mac = normalize_mac(&e.mac_address);
            sqlx::query(
                "INSERT INTO devices (mac_address, hub_id, ip_address, hostname, rssi,
                                      first_seen_at, last_seen_at, connected)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 1)
                 ON CONFLICT(mac_address) DO UPDATE SET
                     hub_id       = excluded.hub_id,
                     ip_address   = excluded.ip_address,
                     hostname     = COALESCE(excluded.hostname, devices.hostname),
                     rssi         = excluded.rssi,
                     last_seen_at = excluded.last_seen_at,
                     connected    = 1",
            )
            .bind(&mac)
            .bind(hub_id)
            .bind(&e.ip_address)
            .bind(&e.hostname)
            .bind(e.rssi)
            .bind(&ts)
            .bind(&ts)
            .execute(&mut *tx)
            .await?;
        }

        // 3) compte les connectés
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM devices WHERE hub_id = ? AND connected = 1",
        )
        .bind(hub_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(count as usize)
    }

    /// Retourne tous les devices connus, connectés ou non.
    pub async fn find_all(&self) -> Result<Vec<Device>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DeviceRow>(
            "SELECT mac_address, hub_id, ip_address, hostname, rssi,
                    first_seen_at, last_seen_at, connected
             FROM devices ORDER BY last_seen_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Devices actuellement connectés à un hub donné.
    pub async fn find_connected_by_hub(&self, hub_id: &str) -> Result<Vec<Device>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DeviceRow>(
            "SELECT mac_address, hub_id, ip_address, hostname, rssi,
                    first_seen_at, last_seen_at, connected
             FROM devices WHERE hub_id = ? AND connected = 1
             ORDER BY last_seen_at DESC",
        )
        .bind(hub_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

fn normalize_mac(mac: &str) -> String {
    mac.trim().to_ascii_lowercase().replace('-', ":")
}

#[derive(sqlx::FromRow)]
struct DeviceRow {
    mac_address: String,
    hub_id: String,
    ip_address: Option<String>,
    hostname: Option<String>,
    rssi: Option<i32>,
    first_seen_at: String,
    last_seen_at: String,
    connected: i64,
}

impl From<DeviceRow> for Device {
    fn from(r: DeviceRow) -> Self {
        Device {
            mac_address: r.mac_address,
            hub_id: r.hub_id,
            ip_address: r.ip_address,
            hostname: r.hostname,
            rssi: r.rssi,
            first_seen_at: r.first_seen_at,
            last_seen_at: r.last_seen_at,
            connected: r.connected != 0,
        }
    }
}
