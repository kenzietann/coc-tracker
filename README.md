# CoC Tracker

A small Clash of Clans upgrade tracker for my main and alt accounts. It's a fun project and a way for me to learn Swift and iOS development.

The iOS app shows upgrades and countdowns from a village export. The NestJS backend imports and stores village data in MariaDB. The timers are estimates based on the most recent export, so changes in the game need another import.

## Project layout

- `frontend/` — iOS app built with SwiftUI
- `backend/` — NestJS API using TypeORM and MariaDB

## Run locally

Start MariaDB and create a database. In `backend/`, copy `.env.example` to `.env` and set your database credentials. Then run:

```sh
npm install
npm run migration:run
npm run start:dev
```

The backend uses port `4444` by default. Open `frontend/frontend.xcodeproj` in Xcode and run the app in an iPhone simulator. The app currently connects to the backend on the same Mac; a physical iPhone will need a reachable backend address.

Village JSON can currently be imported through `POST /clash/import`. In-app import and upgrade notifications are planned.

This is a fan project and is not affiliated with Supercell.
