use serde_json::{Value, json};

// Community ID mappings ported from the original TypeScript backend.
fn item_name(id: i64) -> Option<&'static str> {
    match id {
        1000000 => Some("Army Camp"),
        1000001 => Some("Town Hall"),
        1000002 => Some("Elixir Collector"),
        1000003 => Some("Elixir Storage"),
        1000004 => Some("Gold Mine"),
        1000005 => Some("Gold Storage"),
        1000006 => Some("Barracks"),
        1000007 => Some("Laboratory"),
        1000008 => Some("Cannon"),
        1000009 => Some("Archer Tower"),
        1000010 => Some("Wall"),
        1000011 => Some("Wizard Tower"),
        1000012 => Some("Air Defense"),
        1000013 => Some("Mortar"),
        1000014 => Some("Clan Castle"),
        1000015 => Some("Builder's Hut"),
        1000019 => Some("Hidden Tesla"),
        1000020 => Some("Spell Factory"),
        1000021 => Some("X-Bow"),
        1000023 => Some("Dark Elixir Drill"),
        1000024 => Some("Dark Elixir Storage"),
        1000026 => Some("Dark Barracks"),
        1000027 => Some("Inferno Tower"),
        1000028 => Some("Air Sweeper"),
        1000029 => Some("Dark Spell Factory"),
        1000031 => Some("Eagle Artillery"),
        1000032 => Some("Bomb Tower"),
        1000033 => Some("Wall"),
        1000034 => Some("Builder Hall"),
        1000035 => Some("Elixir Collector"),
        1000036 => Some("Elixir Storage"),
        1000037 => Some("Gold Mine"),
        1000038 => Some("Gold Storage"),
        1000039 => Some("Clock Tower"),
        1000040 => Some("Builder Barracks"),
        1000041 => Some("Double Cannon"),
        1000042 => Some("Army Camp"),
        1000043 => Some("Hidden Tesla"),
        1000044 => Some("Cannon"),
        1000045 => Some("Multi Mortar"),
        1000046 => Some("Star Laboratory"),
        1000048 => Some("Archer Tower"),
        1000049 => Some("Reinforcement Camp"),
        1000050 => Some("Firecrackers"),
        1000051 => Some("Guard Post"),
        1000052 => Some("Mega Tesla"),
        1000053 => Some("Battle Machine Altar"),
        1000054 => Some("Air Bombs"),
        1000055 => Some("Crusher"),
        1000056 => Some("Roaster"),
        1000057 => Some("Giant Cannon"),
        1000058 => Some("Gem Mine"),
        1000059 => Some("Workshop"),
        1000063 => Some("Lava Launcher"),
        1000064 => Some("B.O.B's Hut"),
        1000065 => Some("B.O.B Control"),
        1000067 => Some("Scattershot"),
        1000068 => Some("Pet House"),
        1000070 => Some("Blacksmith"),
        1000071 => Some("Hero Hall"),
        1000072 => Some("Spell Tower"),
        1000077 => Some("Monolith"),
        1000078 => Some("O.T.T.O's Outpost"),
        1000080 => Some("Battle Copter Altar"),
        1000082 => Some("Healing Hut"),
        1000093 => Some("Helper Hut"),
        4000036 => Some("Power P.E.K.K.A"),
        4000053 => Some("Yeti"),
        28000000 => Some("Barbarian King"),
        28000006 => Some("Minion Prince"),
        73000004 => Some("Phoenix"),
        4000015 => Some("Witch"),
        4000033 => Some("Beta Minion"),
        28000001 => Some("Archer Queen"),
        1000106 => Some("Squad Barracks"),
        73000000 => Some("L.A.S.S.I"),
        4000022 => Some("Bowler"),
        _ => None,
    }
}

pub(super) fn collect_timed_entries(value: &Value, entries: &mut Vec<Value>) {
    collect_timed_entries_in_base(value, entries, None, true);
}

fn collect_timed_entries_in_base(
    value: &Value,
    entries: &mut Vec<Value>,
    base: Option<&'static str>,
    is_export_root: bool,
) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_timed_entries_in_base(item, entries, base, false);
            }
        }
        Value::Object(fields) => {
            if value.get("timer").and_then(Value::as_i64).is_some() {
                let mut entry = fields.clone();
                if let Some(level) = entry.remove("lvl") {
                    entry.insert("level".to_string(), level);
                }
                let name = fields
                    .get("data")
                    .and_then(Value::as_i64)
                    .and_then(item_name);
                entries.push(json!({ "name": name, "entry": entry, "base": base }));
            }

            for (key, child) in fields {
                let child_base = if is_export_root {
                    match key.as_str() {
                        "buildings2" | "traps2" | "units2" | "heroes2" | "building_variants2" => {
                            Some("builderBase")
                        }
                        "buildings" | "traps" | "units" | "heroes" | "spells" | "pets"
                        | "siege_machines" | "equipment" | "building_variants" => Some("homeVillage"),
                        _ => None,
                    }
                } else {
                    base
                };
                collect_timed_entries_in_base(child, entries, child_base, false);
            }
        }
        _ => {}
    }
}

pub(super) async fn list_villages(
    pool: &sqlx::MySqlPool,
) -> Result<Vec<(String, String, Option<String>, i64, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT CAST(id AS CHAR), tag, label, exportTimestamp, rawExport
         FROM villages
         ORDER BY tag",
    )
    .fetch_all(pool)
    .await
}

pub(super) async fn save_village(
  pool: &sqlx::MySqlPool,
  tag: &str,
  timestamp: i64,
  raw_export: &str,
) -> Result<bool, sqlx::Error> {
  let mut transaction = pool.begin().await?;

  sqlx::query(
    "INSERT INTO villages (id, tag, exportTimestamp, rawExport)
     VALUES (UUID(), ?, ?, ?)
     ON DUPLICATE KEY UPDATE
        rawExport = IF(
          VALUES(exportTimestamp) >= exportTimestamp,
          VALUES(rawExport),
          rawExport
        ),
        exportTimestamp = GREATEST(exportTimestamp, VALUES(exportTimestamp))"
  ).bind(tag).bind(timestamp).bind(raw_export).execute(&mut *transaction).await?;

  let stored_timestamp: i64 = sqlx::query_scalar("SELECT exportTimestamp FROM villages WHERE tag = ?")
    .bind(tag)
    .fetch_one(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(stored_timestamp == timestamp)
}

pub(super) async fn update_village_label(
    pool: &sqlx::MySqlPool,
    id: &str,
    label: &str,
) -> Result<bool, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let result = sqlx::query("UPDATE villages SET label = ? WHERE id = ?")
        .bind(label)
        .bind(id)
        .execute(&mut *transaction)
        .await?;

    let exists = if result.rows_affected() > 0 {
        true
    } else {
        let row: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM villages WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?;
        row.is_some()
    };

    transaction.commit().await?;
    Ok(exists)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_teslas_by_export_section() {
        let export = json!({
            "buildings": [{"data": 1000019, "lvl": 12, "timer": 100}],
            "buildings2": [{"data": 1000043, "lvl": 4, "timer": 200}],
        });
        let mut entries = Vec::new();
        collect_timed_entries(&export, &mut entries);
        assert_eq!(
            entries,
            vec![
                json!({"name": "Hidden Tesla", "base": "homeVillage", "entry": {"data": 1000019, "level": 12, "timer": 100}}),
                json!({"name": "Hidden Tesla", "base": "builderBase", "entry": {"data": 1000043, "level": 4, "timer": 200}}),
            ]
        );
    }
}
