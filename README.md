# CoC Tracker

A small Clash of Clans upgrade tracker for my main and alt accounts. It's also how I'm learning SwiftUI and Rust.

Paste a village export into the app to see upgrades and estimated countdowns. The Rust backend saves the latest snapshot for each village in MariaDB and rejects older exports. To pick up changes from the game, import a fresh export.

## Project layout

- `frontend/` hosts the SwiftUI app.
- `backend/` hosts the Rust API. It uses Hyper and Tokio for HTTP and SQLx for raw SQL queries against MariaDB.

## Run locally

You'll need Xcode, Rust with Cargo (edition 2024), and MariaDB 10.7 or later for the native UUID column.

If you already have a `coc.villages` table from the NestJS backend, you can reuse it. For a fresh setup, run this SQL as a MariaDB admin:

```sql
CREATE DATABASE IF NOT EXISTS coc;

CREATE TABLE IF NOT EXISTS coc.villages (
    id UUID NOT NULL PRIMARY KEY,
    tag VARCHAR(32) NOT NULL UNIQUE,
    label VARCHAR(100) NULL,
    exportTimestamp BIGINT NOT NULL,
    rawExport TEXT NOT NULL
) ENGINE=InnoDB;

CREATE USER 'coc_tracker'@'127.0.0.1' IDENTIFIED BY 'replace_with_a_password';
GRANT SELECT, INSERT, UPDATE ON coc.villages TO 'coc_tracker'@'127.0.0.1';
```

If your database account already exists, skip `CREATE USER` and grant it access to the table. Create `backend/.env` with your local connection string:

```dotenv
DATABASE_URL=mysql://coc_tracker:replace_with_a_password@127.0.0.1:3306/coc
```

Percent-encode any URL special characters in the password. Then start the backend:

```sh
cd backend
cargo run --locked
```

The API listens at `http://127.0.0.1:4444`. Open `frontend/frontend.xcodeproj` in Xcode and run the app on your Mac or in an iPhone simulator on the same Mac. Tap **Import** and paste a village export. After a successful import, the dashboard refreshes automatically.

## API

| Method | Path | What it does |
| --- | --- | --- |
| GET | `/health` | Checks that the server is up. |
| GET | `/clash/villages` | Lists saved villages and their timed upgrades. |
| POST | `/clash/import` | Saves a village export. Returns `409 Conflict` if a newer export is already saved. |

Tap the bell to enable local reminders one minute before an upgrade’s estimated finish. Import a fresh export to update the reminders. Village renaming is planned.

This is a fan project and is not affiliated with Supercell.

## Artwork credits

Clash of Clans artwork belongs to Supercell and is used under its [Fan Content Policy](https://supercell.com/en/fan-content-policy/). The bundled thumbnails come from [ClashKing Assets](https://github.com/ClashKingInc/ClashKingAssets/tree/6da8ce864afe2b4510d3342e1c7374e48d7f5a49). Item IDs use the project's existing lookup and the [community village export mapping](https://gist.github.com/rahulkhatri137/a8449943df45100c5f1e1359cd9ec67a).

The dashboard uses one representative image per item type. Buildings use the highest numbered render available in this snapshot, while the countdowns use the saved village export.
