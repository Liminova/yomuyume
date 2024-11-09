use chrono::Utc;

/// Like [`Config`], but configurable from the frontend.
#[derive(Debug)]
pub struct LiveConfig {
    pub nomedia_support: bool,
    pub oneshot_postfix_support: bool,
    pub rescan_interval_in_minutes: i32,
}

impl LiveConfig {
    pub async fn from_db(db: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        let record = sqlx::query!(r#"SELECT value FROM live_config WHERE id = 'nomedia_support'"#)
            .fetch_optional(db)
            .await?;
        let nomedia_support = record
            .map(|record| record.value == "true")
            .unwrap_or_default();

        let record =
            sqlx::query!(r#"SELECT value FROM live_config WHERE id = 'oneshot_postfix_support'"#)
                .fetch_optional(db)
                .await?;
        let oneshot_postfix_support = record
            .map(|record| record.value == "true")
            .unwrap_or_default();

        let record = sqlx::query!(
            r#"SELECT value FROM live_config WHERE id = 'rescan_interval_in_minutes'"#
        )
        .fetch_optional(db)
        .await?;
        let rescan_interval_in_minutes = record
            .map(|record| record.value.parse::<i32>().unwrap_or_else(|_| 60))
            .unwrap_or_else(|| 60);

        Ok(Self {
            nomedia_support,
            oneshot_postfix_support,
            rescan_interval_in_minutes,
        })
    }

    pub async fn configure_nomedia_support(
        &mut self,
        db: &sqlx::PgPool,
        value: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3"#,
            "nomedia_support",
            if value { "true" } else { "false" },
            Utc::now()
        )
        .execute(db)
        .await?;

        self.nomedia_support = value;

        Ok(())
    }

    pub async fn configure_oneshot_postfix_support(
        &mut self,
        db: &sqlx::PgPool,
        value: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3"#,
            "oneshot_postfix_support",
            if value { "true" } else { "false" },
            Utc::now()
        )
        .execute(db)
        .await?;

        self.oneshot_postfix_support = value;

        Ok(())
    }

    pub async fn configure_rescan_interval_in_minutes(
        &mut self,
        db: &sqlx::PgPool,
        value: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET value = $2, last_updated_at = $3"#,
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
