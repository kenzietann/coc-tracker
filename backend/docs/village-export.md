# Clash of Clans village export

This document describes the shape observed in one in-game village JSON export. It is a working reference for the tracker, not an official or complete schema. Other villages, game versions, and export states may contain different fields or omit fields shown here.

The export is a snapshot, not a live feed. Re-import it when upgrades change. The official Clash API and its developer token are not needed to parse this export.

## Top-level fields

| Field | Observed type | What it appears to represent |
| --- | --- | --- |
| `tag` | string | Player tag, including `#`; use it to associate imports with a village. |
| `timestamp` | integer | Export time as a Unix timestamp in seconds. |
| `helpers` | array of objects | Helper entries and, in some cases, cooldown information. |
| `buildings`, `traps`, `decos`, `obstacles` | arrays of objects | Home Village buildings, traps, decorations, and obstacles. |
| `units`, `siege_machines`, `heroes`, `spells`, `pets`, `equipment` | arrays of objects | Home Village army and equipment entries. |
| `house_parts`, `skins`, `sceneries` | arrays of integers | Cosmetic identifiers. |
| `buildings2`, `traps2`, `decos2`, `obstacles2` | arrays of objects | Entries that appear to belong to Builder Base. |
| `units2`, `heroes2` | arrays of objects | Builder Base units and heroes. |
| `skins2`, `sceneries2` | arrays | Builder Base cosmetic collections; both were empty in this sample, so their item shapes are unconfirmed. |
| `boosts` | object | Present but empty in this sample; its contents and meaning are unconfirmed. |

Empty arrays are valid in the sample. Treat missing fields and additional fields defensively rather than assuming every export has exactly this shape.

## Common entry fields

| Field | Observed type | Interpretation |
| --- | --- | --- |
| `data` | integer | Game data identifier. The export does not provide a readable item name; a separate, maintained ID-to-name catalogue is needed for the UI. |
| `lvl` | integer | Item level. Some entries omit it; check upgrade-level semantics against the game before displaying a current-to-target level. |
| `cnt` | integer | Appears on grouped entries and likely represents a count. Do not assume every entry represents one individual building. |
| `timer` | integer | Appears on some entries and appears to be remaining time in seconds at export time. Confirm this interpretation and the relevant item against the game before relying on notifications. |
| `extra` | boolean | Seen on one timed building. Its exact meaning is not established by this sample. |
| `helper_recurrent` | boolean | Seen on timed unit and hero entries. Its effect on timing is unconfirmed. |
| `helper_cooldown` | integer | Seen on helper entries; appears to be a cooldown, but units and semantics need confirmation. |
| `gear_up`, `weapon` | integer | Seen on some buildings; exact semantics are unconfirmed. |
| `types` | array of objects | Seen on one building instead of `lvl`; each type contains a `data` identifier and a `modules` array of objects with `data` and `lvl`. |

An illustrative shape (not a complete export):

```json
{
  "tag": "#EXAMPLE",
  "timestamp": 1700000000,
  "buildings": [
    { "data": 1000005, "lvl": 10, "timer": 3600 },
    { "data": 1000008, "lvl": 8, "cnt": 2 }
  ],
  "units": [],
  "heroes": [],
  "pets": []
}
```

The identifiers and levels in this example are only placeholders, not verified upgrade data.

## Active upgrades and notifications

In the supplied sample, `timer` appeared on entries in `buildings`, `units`, `heroes`, `pets`, `buildings2`, and `units2`. Those are useful starting points when looking for active upgrades. The sample does not establish that these are the only possible collections containing timers, or that every `timer` means the same thing.

If a timer is confirmed to be seconds remaining at `timestamp`, calculate its estimated completion as **export timestamp + timer seconds**, not import time + timer. This matters when the export is pasted later. Compare the estimate with the current time before scheduling a notification; an estimate in the past may already have finished.

Timers are estimates from the snapshot. Books, Hammers, boosts, helpers, or other changes after export can make them inaccurate. Show the last export time and replace or cancel affected notifications when a newer import changes or removes an upgrade. An expired countdown alone does not confirm the current in-game state.

The sample includes `extra` on one timed building, so do not assume each timed building corresponds to a normal builder slot. Likewise, keep Home Village and Builder Base separate, and distinguish research or pet upgrades from building upgrades where the data supports it.

## Questions to verify while implementing

- Compare the export timestamp, timer values, and `lvl` values with the in-game screens for the same upgrades.
- Find and maintain a suitable source for mapping game data IDs to names and, if needed, categories and level caps.
- Determine the meaning of `extra`, helper fields, and nested building modules before using them for builder availability or finish-time adjustments.
- Decide how to identify repeated structures reliably across imports; `data` alone may not uniquely identify an individual building.
- Define how to handle invalid JSON, missing player tags, unknown identifiers, older imports, and unfamiliar fields without losing the last usable snapshot.
- Test whether notifications remain useful across app reinstalls and refresh them whenever imported data changes.

Treat a village export as player data: avoid committing real exports or logging their contents unnecessarily. Never put the developer API token in an export or documentation.
