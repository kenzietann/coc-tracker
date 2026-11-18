pub(super) async fn list_villages(pool: &sqlx::MySqlPool) -> Result<Vec<(String, String, Option<String>, i64)>, sqlx::Error> {
  sqlx::query_as("SELECT CAST(id AS CHAR), tag, label, exportTimestamp FROM villages ORDER BY tag")
    .fetch_all(pool)
    .await
}