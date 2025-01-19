use chrono::Utc;

/// Like [`Config`], but configurable from the frontend.
#[derive(Debug)]
pub struct LiveConfig {
    pub nomedia_support: bool,

    pub komga_oneshot_support: bool,
    pub komga_recycle_support: bool,

    pub rescan_enabled: bool,
    pub rescan_interval_in_minutes: i32,
}

impl LiveConfig {
    pub async fn from_db(db: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        let nomedia_support =
            sqlx::query!("SELECT value FROM live_config WHERE id = 'nomedia_support'")
                .fetch_optional(db)
                .await?
                .map_or_else(|| false, |record| record.value == "true");

        let komga_oneshot_support =
            sqlx::query!("SELECT value FROM live_config WHERE id = 'komga_oneshot_support'")
                .fetch_optional(db)
                .await?
                .map_or_else(|| false, |record| record.value == "true");

        let komga_recycle_support =
            sqlx::query!("SELECT value FROM live_config WHERE id = 'komga_recycle_support'")
                .fetch_optional(db)
                .await?
                .map_or_else(|| false, |record| record.value == "true");

        let rescan_enabled =
            sqlx::query!("SELECT value FROM live_config WHERE id = 'rescan_enabled'")
                .fetch_optional(db)
                .await?
                .map_or_else(|| false, |record| record.value == "true");

        let rescan_interval_in_minutes =
            sqlx::query!("SELECT value FROM live_config WHERE id = 'rescan_interval_in_minutes'")
                .fetch_optional(db)
                .await?
                .map_or_else(|| 60, |record| record.value.parse::<i32>().unwrap_or(60));

        Ok(Self {
            nomedia_support,
            komga_oneshot_support,
            komga_recycle_support,
            rescan_enabled,
            rescan_interval_in_minutes,
        })
    }

    pub async fn configure_nomedia_support(
        &mut self,
        db: &sqlx::PgPool,
        value: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3",
            "nomedia_support",
            if value { "true" } else { "false" },
            Utc::now()
        )
        .execute(db)
        .await?;

        self.nomedia_support = value;

        Ok(())
    }

    pub async fn configure_komga_oneshot_support(
        &mut self,
        db: &sqlx::PgPool,
        value: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3",
            "komga_oneshot_support",
            if value { "true" } else { "false" },
            Utc::now()
        )
        .execute(db)
        .await?;

        self.komga_oneshot_support = value;

        Ok(())
    }

    pub async fn configure_komga_recycle_support(
        &mut self,
        db: &sqlx::PgPool,
        value: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3",
            "komga_recycle_support",
            if value { "true" } else { "false" },
            Utc::now()
        )
        .execute(db)
        .await?;

        self.komga_recycle_support = value;

        Ok(())
    }

    pub async fn configure_rescan_enabled(
        &mut self,
        db: &sqlx::PgPool,
        value: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3",
            "rescan_enabled",
            if value { "true" } else { "false" },
            Utc::now()
        )
        .execute(db)
        .await?;

        self.rescan_enabled = value;

        Ok(())
    }

    pub async fn configure_rescan_interval_in_minutes(
        &mut self,
        db: &sqlx::PgPool,
        value: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3",
            "rescan_interval_in_minutes",
            format!("{}", value),
            Utc::now()
        )
        .execute(db)
        .await?;

        self.rescan_interval_in_minutes = value;

        Ok(())
    }
}
