# Building IDs in the sample village export

The `data` number identifies a building type. This reference covers the distinct IDs found in the supplied export, not every building in the game. Names come from community-maintained mappings, not an official Supercell schema, and may change or contain errors. Checked 2026-11-13.

Sources: [community export mapping, Home Village and Builder Base](https://gist.github.com/rahulkhatri137/a8449943df45100c5f1e1359cd9ec67a), [another export mapping](https://gist.github.com/pghant/0717bb1e0e4e0d1373e90bdb3057d9dd/b8792f7d14404068f47570f4a78db7048893bfac), and [independent Home Village building mapping](https://github.com/cerealkillahh/clash-of-clans-data/blob/main/data/buildings.json).

The same ID can appear more than once: it identifies a type, not a unique individual structure. Use the collection (`buildings` or `buildings2`) to tell the villages apart. Keep unknown IDs in the result and show their number rather than dropping them.

## Home Village (`buildings`)

| `data` ID | Building |
| --- | --- |
| `1000000` | Army Camp |
| `1000001` | Town Hall |
| `1000002` | Elixir Collector |
| `1000003` | Elixir Storage |
| `1000004` | Gold Mine |
| `1000005` | Gold Storage |
| `1000006` | Barracks |
| `1000007` | Laboratory |
| `1000008` | Cannon |
| `1000009` | Archer Tower |
| `1000010` | Wall |
| `1000011` | Wizard Tower |
| `1000012` | Air Defense |
| `1000013` | Mortar |
| `1000014` | Clan Castle |
| `1000015` | Builder's Hut |
| `1000019` | Hidden Tesla |
| `1000020` | Spell Factory |
| `1000021` | X-Bow |
| `1000023` | Dark Elixir Drill |
| `1000024` | Dark Elixir Storage |
| `1000026` | Dark Barracks |
| `1000027` | Inferno Tower |
| `1000028` | Air Sweeper |
| `1000029` | Dark Spell Factory |
| `1000031` | Eagle Artillery |
| `1000032` | Bomb Tower |
| `1000059` | Workshop |
| `1000064` | B.O.B's Hut |
| `1000067` | Scattershot |
| `1000068` | Pet House |
| `1000070` | Blacksmith |
| `1000071` | Hero Hall |
| `1000072` | Spell Tower |
| `1000077` | Monolith |
| `1000093` | Helper Hut |
| `1000097` | Crafting Station / Crafted Defense (sources disagree; verify in-game) |

## Builder Base (`buildings2`)

| `data` ID | Building |
| --- | --- |
| `1000033` | Wall |
| `1000034` | Builder Hall |
| `1000035` | Elixir Collector |
| `1000036` | Elixir Storage |
| `1000037` | Gold Mine |
| `1000038` | Gold Storage |
| `1000039` | Clock Tower |
| `1000040` | Builder Barracks |
| `1000041` | Double Cannon |
| `1000042` | Army Camp |
| `1000043` | Hidden Tesla |
| `1000044` | Cannon |
| `1000045` | Multi Mortar |
| `1000046` | Star Laboratory |
| `1000048` | Archer Tower |
| `1000049` | Reinforcement Camp |
| `1000050` | Firecrackers |
| `1000051` | Guard Post |
| `1000052` | Mega Tesla |
| `1000053` | Battle Machine Altar |
| `1000054` | Air Bombs |
| `1000055` | Crusher |
| `1000056` | Roaster |
| `1000057` | Giant Cannon |
| `1000058` | Gem Mine |
| `1000063` | Lava Launcher |
| `1000065` | B.O.B Control |
| `1000078` | O.T.T.O's Outpost |
| `1000080` | Battle Copter Altar |
| `1000082` | Healing Hut |

## Timed buildings in the sample

The sample includes five timed entries in `buildings`: two Gold Storages (`1000005`), two Army Camps (`1000000`), and one Dark Elixir Storage (`1000024`). It also includes two in `buildings2`: a Giant Cannon (`1000057`) and a Mega Tesla (`1000052`). These describe that one snapshot, not necessarily their current status.

The `1000097` entry contains nested `types` and `modules`; the sources call it either Crafting Station or Crafted Defense. Verify its in-game name and semantics before using it as a definitive label. See [village-export.md](village-export.md) for the export fields and timer limitations.
