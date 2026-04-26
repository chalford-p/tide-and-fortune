# Tide And Fortune - Game Design

### 1. Points of Sail

Points of sail are *always visible and always matter*, but the game handles sail selection and trim automatically. The player feels the physics without managing it.

**The Wind Rose HUD element** is always on screen — a compact compass showing:
- Current wind direction
- Your heading relative to it
- Your current point of sail, named and colour-coded

```
  ██ In Irons (no drive)        [grey]
  ██ Close-Hauled               [blue]  ~30–45° off wind
  ██ Close Reach                [cyan]  ~60°
  ██ Beam Reach                 [green] ~90°  ← fastest point (most vessels)
  ██ Broad Reach                [yellow]~120–135°
  ██ Running                    [orange]~180°  ← risk of accidental gybe
```

Auto-trim means: the game silently selects the optimal sail for your point of sail and sheets it correctly. The *consequence* of point of sail is fully real — you slow dramatically beating into wind, surge on a reach, and risk a crash-gybe dead downwind — but you don't manage sails manually until Tier 2.

**What this teaches:** Players naturally learn to use the wind without being told to. Sailing into irons feels bad. Cracking off onto a reach feels rewarding. Real sailing intuition forms organically.

**Gybe warning:** Even in Tier 1, a visual + audio warning fires when you're broad off and turning further downwind — the boom shadow swings, a creak sounds. If you cross dead downwind carelessly, an auto-gybe happens with a speed/heading penalty. Consequence without punishment.

---

### 2. Progression — Solo-Primary (Revised)

The progression is reframed around *capability and story*, not fleet size. Bigger ships are an option, not an obligation.

**New progression philosophy:** You can finish the main story in a fast schooner if you want. Larger ships open different *kinds* of stories — not better ones. A nimble sloop can go places a frigate never could.

```
Tier 1 — The Small Craft Years
  Sloop / Catboat: Solo, learn the world, small cargo runs,
  coastal exploration, first relationships formed.

Tier 2 — A Ship of Your Own
  Brigantine / Schooner: Hire a small crew, take on a first mate NPC,
  longer voyages, first real combat, treasure hunt arcs begin.

Tier 3 — The Captain's Reputation
  Brig / Sloop-of-war: Named officers, crew morale system live,
  faction standing matters. Other captains seek YOU out for missions.
  You may join convoy escorts, raid fleets, rescue operations —
  as a participant, not commander.

Tier 4 — The Fleet Moments
  Select story beats escalate to fleet engagements.
  You are always the focal point — the decisive blade —
  not the admiral shuffling pieces. Command is temporary,
  dramatic, earned. Between these moments: back to your ship, alone.
```

**Joining other ships on missions:** Allied captains (NPCs you've built trust with) send messenger boats, letters at port, or hail you at sea. Mission types:
- *Sail with me* — escort/accompany, you maintain your own helm
- *Cover my flank* — tactical positioning, semi-coordinated
- *We take that fort together* — a scripted joint operation with fleet moments

You never lose control of your own ship except in cutscenes.

---

### 2a Sailing Physics Model

This is the heart of the game. Three tiers:

Tier 1 — Arcade (all players, always):

WASD / arrows = 8 heading directions, ship steers toward target heading at a rate influenced by wind
Speed is automatic based on heading vs. wind angle — player feels the drag on a beat, the surge on a run
Wind direction shown via a compass rose / sock on screen
No manual sail trim needed
Point of sail zones: close-hauled, beam reach, broad reach, run — shown visually
Irons zone: enter it and you wallow, must fall off

Tier 2 — Intermediate (unlocked mid-game):

Sail selection: foresail, mainsail, topsails — each togglable
Sheeting: a "trim" bar per sail, optimised position shown by telltales

Tier 3 — Expert (unlocked late-game, optional):

True wind vs. apparent wind distinction
Tacking and gybing have timing windows — mistimed gybe can cost a spar
Tide modelling: tidal currents on charts, timing harbour entries
Heel angle affecting speed and handling
Jury rigging after battle damage

#### Physics model (Bevy implementation approach):

The wind field is a 2D vector field, queryable at any world position. Each ship has:
apparent_wind = true_wind_at_position - ship_velocity_vector
wind_angle_of_attack = angle between apparent_wind and ship_heading
drive_force = sail_area × cos(angle_of_attack) × trim_efficiency
leeway = lateral_force / lateral_resistance
Simple enough for arcade feel, correct enough that real sailing intuitions apply.

### 3. Island Exploration — On-Foot Mode

This is now a full design layer, not an afterthought.

---

#### 3a. The Transition

Anchoring is intentional and physical. You sail into a cove or approach a beach, drop anchor (key press, with a satisfying chain-rattle animation), then lower the jolly boat. The camera smoothly shifts from isometric sailing view to... the same isometric perspective, now at human scale. The tonal shift is handled through scale and sound design, not a loading screen.

Your ship sits anchored in the background — visible, vulnerable. If you linger, tide can shift it. If enemies patrol, they may find it. Time pressure as exploration mechanic.

---

#### 3b. On-Foot Systems

**Movement:** WASD again — same controls, human scale. Your character moves through:
- **Beaches** — open, fast movement, visible from sea
- **Jungle** — slow, obscures vision, hides threats and secrets
- **Ruins** — multi-level, isometric depth used beautifully here
- **Settlements / Ports** — NPC hubs, shops, taverns, mission givers
- **Caves** — torch-lit, narrow, treasure and danger

**The exploration toolkit** (acquired over time):
- Compass + partial map (fills in as you explore)
- Treasure maps (found, bought, stolen — cryptic landmarks)
- Pry bar, shovel (for locked chests, buried treasure)
- Pistol + cutlass (light combat system — see below)
- Spyglass (scout from high ground, spots ships at sea too)

---

#### 3c. On-Foot Combat

Light, not a full action game. Think a simplified *Sea of Thieves* meets top-down brawler.

- **Cutlass:** A 3-hit combo, parry window, stamina-gated
- **Pistol:** Single shot, long reload — a commitment
- **Fleeing:** Always a valid option; jungle cover, distance, losing pursuers
- **Officers help:** If crew accompany you ashore, they fight alongside (simple AI)

You can bring 0–4 crew ashore. More crew = safer, but ship is more vulnerable (skeleton crew). A genuine tension.

---

#### 3d. Treasure & Discovery

Treasure is layered — not just chests:

| Type | How found | Reward |
|---|---|---|
| Buried chest | Treasure map + landmark matching + dig | Gold, gems, rare goods |
| Wreck cargo | Coastal/shallow diving (minigame) | Mixed cargo, ship parts |
| Ruin artefact | Exploration + puzzle | Story items, faction standing |
| Indigenous knowledge | NPC relationship + trust | Hidden coves, secret routes, rare charts |
| Planted cache | Other captains' side quests | Shared reward, relationship boost |

**Treasure maps** are physical objects in your inventory — rendered as aged paper, with illustrated landmarks (a distinctive rock formation, a tree line, a bearing from a named cape). You match them against the world. No waypoint markers. Navigation is the puzzle.

---

#### 3e. NPC Interaction Ashore

Settlements have a small cast of recurring and procedural NPCs:

- **Tavern keeper** — rumours, crew hiring, black market access
- **Harbormaster** — legitimate cargo, port reputation, letters
- **Governor / official** — faction quests, legal standing, intrigue
- **Mysterious stranger** — story-thread hooks, unreliable information
- **Fellow captain** — relationship scenes, joint mission offers

Dialogue is choice-based but lightweight — 2–3 options, consequences tracked. No dialogue trees that take 10 minutes. The *relationship graph* does the heavy lifting; what you say matters less than your accumulated reputation and trust.

---

### 4. Revised Bevy Architecture

The mode switch (sailing ↔ on-foot) is the key new architectural concern.

---

#### 4a. Game Mode State Machine

```rust
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum GameMode {
    #[default]
    Sailing,
    Anchoring,      // transition animation
    OnFoot,         // island exploration
    Combat,         // on-foot combat sub-mode
    PortInterior,   // indoor NPC spaces
    FleetBattle,    // zoomed-out fleet command
    Cutscene,
}
```

Systems are gated with `.run_if(in_state(GameMode::Sailing))` etc. The sailing world is *not unloaded* when you go ashore — your ship stays simulated (at reduced tick rate). Enemies can still find it.

---

#### 4b. World Representation

Two spatial scales share the same world coordinate space — just different camera zoom/projection:

```rust
// Sailing scale: 1 unit ≈ 10 metres
// On-foot scale: 1 unit ≈ 0.5 metres
// Islands exist as collision + content in both scales
// — a hill visible from sea is the same entity you climb on foot

#[derive(Component)]
struct Island {
    id: IslandId,
    sailing_mesh: Handle<Mesh>,       // simplified, for sea view
    onfoot_tilemap: Handle<TileMap>,  // detailed, loaded on approach
}
```

Island detail tiles load when your ship comes within a threshold distance — seamless LOD. The on-foot tilemap is a `bevy_ecs_tilemap` asset, streamed in. When you re-embark, it can be retained briefly in case you return, then unloaded.

---

#### 4c. The Anchor / Embark System

```rust
// Player action: anchor
fn handle_anchor_command(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameMode>>,
    player_ship: Query<(Entity, &Transform, &Velocity), With<PlayerShip>>,
    islands: Query<(&Island, &Transform)>,
) {
    let (ship_entity, ship_xform, vel) = player_ship.single();
    // Validate: near shore, speed low enough, depth ok
    if vel.linvel.length() < ANCHOR_MAX_SPEED 
        && near_anchorage(ship_xform, &islands) 
    {
        commands.entity(ship_entity).insert(Anchored {
            position: ship_xform.translation,
            chain_length: 20.0,
            drift_factor: current_tide_factor(),
        });
        next_state.set(GameMode::Anchoring); // animation state
    }
}

// After anchor animation completes → OnFoot
// Player entity spawned at beach landing point
// Camera lerps from ship to player
```

---

#### 4d. On-Foot ECS Structure

```rust
#[derive(Bundle)]
struct PlayerAshoreBundle {
    player: PlayerAshore,
    inventory: Inventory,
    equipped: EquippedItems,   // cutlass, pistol, tools
    stamina: Stamina,
    transform: Transform,
    sprite: SpriteSheetBundle, // 8-directional walk/idle/combat frames
    collider: Collider,
}

#[derive(Bundle)]
struct TreasureBundle {
    treasure: Treasure,
    state: TreasureState,     // Buried | Exposed | Looted
    map_reference: Option<MapId>,
    transform: Transform,
    sprite: SpriteBundle,
    collider: Collider,
}

#[derive(Component)]
struct NpcAshore {
    role: NpcRole,
    relationship: RelationshipId,  // links to the global relationship graph
    dialogue_state: DialogueState,
    schedule: DailySchedule,       // where they are at what time of day
}
```

---

#### 4e. Shared Systems That Run Across Modes

Some systems run regardless of mode:

```rust
// Always running:
// - Wind field update
// - Tide model
// - NPC ship AI (even while you're ashore)
// - Time of day / weather
// - Relationship graph event processing
// - Your anchored ship: drift check, enemy detection

app.add_systems(Update, (
    update_wind_field,
    update_tide,
    tick_npc_ship_ai,
    advance_time_of_day,
    check_anchored_ship_safety,  // warns player if ship in danger
    process_relationship_events,
));

// Sailing-only:
app.add_systems(Update,
    sailing_physics_system
        .run_if(in_state(GameMode::Sailing)
            .or_else(in_state(GameMode::Anchoring)))
);

// OnFoot-only:
app.add_systems(Update, (
    player_ashore_movement,
    treasure_interaction,
    npc_dialogue_system,
    on_foot_combat_system,
).run_if(in_state(GameMode::OnFoot)
    .or_else(in_state(GameMode::Combat)))
);
```

---

### 5. Naval Combat
Scale 1 — Ship-to-ship: Real-time, direct control. Positioning and wind gauge matter enormously — being upwind lets you dictate range. Broadside timing, chain shot for rigging, grape for crew.
Scale 2 — Small fleet: You command flagship directly, give orders to 2–4 other ships via a simple command wheel: Engage / Follow / Hold position / Flee / Board.
Boarding: A simplified crew vs. crew minigame — not a full action game, but your officer stats, crew morale, and weapon choices create meaningful decisions.

Damage model:
- Hull (zones: bow, amidships, stern, waterline) — flooding is slow death
- Rigging (masts, spars, sails) — affects sail plan
- Crew casualties — affects boarding, reload rate, morale
- Powder magazine — catastrophic if hit

### 5. Crew & Resource Management
Crew have roles: Captain, First Mate, Bosun, Gunner, Surgeon, Navigator, Cook. Each role has stat impact. Named NPCs can become officers with relationships, backstories, side quests.
Morale system: Influenced by food quality, prize money shares, discipline, captain reputation, time at sea. Low morale → slower work, theft, eventual mutiny.
Supplies: Food, water, powder, shot, spare spars, medical supplies, trade goods (legal), contraband.
Cargo economy: Dynamic pricing between ports. Smuggling profitable but reputation-risking. Some goods only findable at treasure sites or indigenous traders.

### 6. Story & Relationships
Narrative approach: Procedurally assembled story beats from a authored library, stitched by relationship state and faction standing. Similar in spirit to Dwarf Fortress's emergent storytelling but with hand-crafted story arcs at key milestones.
Relationship system: Named NPCs (rival captains, port governors, merchants, potential romantic interests) have:

Trust / Hostility axis
Debt / Favour axis
Secret (discoverable facts that change the relationship)

Story threads: The Rival (another captain always one step ahead of the player), The Governor's Daughter/Son, The Old Chart (a treasure hunt MacGuffin running the full game), The Mutineer (someone in your crew with a secret agenda).

8. Multiplayer Design
Modes:

Co-op fleet: 2–4 players each captain a ship in the same fleet
Rival captains: Shared world, competing for treasure and trade dominance, PvP optional
Faction war: Team-based, players pick colonial factions, port control over sessions

Architecture: Authoritative server model. Bevy's networking story (via bevy_replicon or lightyear) suits this — more below.

## Revised Build Order

The on-foot layer slots in as Slice 3, after the sailing core is fun:

| Slice | Focus | Done when... |
|---|---|---|
| 1 | Wind, ship physics, arcade sailing, single island, camera | Sailing one island to another feels good |
| 2 | Ports, cargo, NPC ships, basic economy | You have a reason to sail somewhere |
| 3 | Anchor → on-foot → explore → re-embark | One island has something worth finding on foot |
| 4 | Treasure system, maps, on-foot combat, crew ashore | A full treasure hunt loop works end-to-end |
| 5 | Relationships, story events, dialogue, faction standing | The world feels inhabited |
| 6 | Naval combat, damage model, boarding | Fighting has consequence |
| 7 | Crew/officer system, morale, supplies | Longer voyages have texture |
| 8 | Fleet moments, join/assist missions | The big story beats land |
| 9 | Multiplayer | Co-op first |
| 10 | World generation, full story arcs, polish | Ship it |

---

This design now has a really satisfying rhythm: **sail → discover → anchor → explore → uncover → sail on, changed**. The solo focus keeps the emotional core intimate, while the fleet moments feel genuinely epic because they're rare and earned.

Where do you want to go next? I'd suggest either diving into the **sailing physics implementation** in detail, or sketching out the **island/world data model** — both are foundational to everything above.