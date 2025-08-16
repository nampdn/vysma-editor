# MOBA Server (Authoritative state, persistence, MMO hooks)

# Persistence models
models {
  model "Player"      { key = "id", fields = { hp = 100.0, xp = 0.0, mmr = 1000.0 } }
  model "Leaderboard" { key = "id", fields = { score = 0.0 } }
}

# Bind a player record to entity scope variables (hp/xp)
persist_bind "PlayerState" { model = "Player", id = "${player_id}", scope = "entity", targets = { name = "Player" }, map = { hp = "hp", xp = "xp" } }

vars = { tick_rate = 20.0 }

triggers {
  # Load player state on startup and when player joins (presence event)
  trigger "load_on_start" { authority = "server" on = { startup = true } actions = [ { persist_load = { binding = "PlayerState" } } ] }
  trigger "load_on_join"  { authority = "server" on = { event = "presence.join" } actions = [ { persist_load = { binding = "PlayerState" } } ] }

  # Save periodically and on checkpoint
  trigger "autosave" { authority = "server" on = { tick = { every = 30.0 } } actions = [ { persist_save = { binding = "PlayerState" } } ] }
  trigger "save_checkpoint" { authority = "server" on = { event = "checkpoint" } actions = [ { persist_save = { binding = "PlayerState" } } ] }

  # Combat: award xp on death; update leaderboard on score change
  trigger "award_xp" { authority = "server" on = { event = "combat.death" } actions = [ { add_var = { name = "xp", by = 10.0, scope = "entity" }, targets = { name = "Player" } }, { persist_save = { binding = "PlayerState" } } ] }
  trigger "leaderboard_top" { authority = "server" on = { tick = { every = 10.0 } } actions = [ { persist_query = { model = "Leaderboard", where = [], order_by = [ { field = "score", dir = "desc" } ], limit = 10, store_as = "lb_" } } ] }

  # Matchmaking and instances
  trigger "queue" { authority = "server" on = { event = "ui.queue" } actions = [ { match_queue = { mode = "pvp", size = 10 } } ] }
  trigger "on_match_found" { authority = "server" on = { event = "party.match_found" } actions = [
    { instance_create = { template = "classic_lane", store_as = "inst" } },
    { send_chat = { channel = "party", text = "Match found!" } },
    { instance_transfer = { targets = { tag = "party" }, instance_id = "${inst}", spawn = [0,1,0] } }
  ] }

  # Presence announcements
  trigger "announce_join" { authority = "server" on = { event = "presence.join" } actions = [ { send_chat = { channel = "global", text = "A player has joined" } } ] }
}

# Lanes, creeps, towers (server authority)

vars = { gold = 0.0 }

# Spawn creeps periodically per lane
triggers {
  trigger "lane_top"    { authority = "server" on = { tick = { every = 30.0 } } actions = [ { spawn = { prefab = "core::Creep", components = { Name = "CreepTop" } } } ] }
  trigger "lane_middle" { authority = "server" on = { tick = { every = 30.0 } } actions = [ { spawn = { prefab = "core::Creep", components = { Name = "CreepMid" } } } ] }
  trigger "lane_bot"    { authority = "server" on = { tick = { every = 30.0 } } actions = [ { spawn = { prefab = "core::Creep", components = { Name = "CreepBot" } } } ] }

  # Simple creep AI: move towards enemy base (placeholder path)
  trigger "creep_ai" { authority = "server" on = { tick = { every = 0.5 } } target = { tag = "Creep" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "creep_speed", use_dt = false } } ] }

  # Towers: target nearest enemy within range and emit combat.hit
  trigger "tower_target" {
    authority = "server"
    on = { tick = { every = 0.5 } }
    actions = [ { emit = { name = "combat.hit", payload = { damage = 20 } } } ]
  }

  # On enemy death: award gold/xp and respawn after delay
  trigger "on_enemy_death" {
    authority = "server"
    on = { event = "combat.death" }
    actions = [
      { add_var = { name = "gold", by = 50.0 } },
      { set_timer = { name = "respawn_enemy", seconds = 15.0, repeating = false } }
    ]
  }
  trigger "respawn_enemy" { authority = "server" on = { timer = "respawn_enemy" } actions = [ { spawn = { prefab = "core::Creep" } } ] }

  # Objective: destroy tower when hp<=0, announce
  trigger "tower_destroyed" { authority = "server" on = { event = "combat.hit" } when = [ { expr = "tower_hp <= 0" } ] actions = [ { send_chat = { channel = "global", text = "A tower has fallen!" } } ] }
} 

# Server-authoritative input handling and replication

triggers {
  # Movement from input events
  trigger "srv_move_w" { authority = "server" on = { event = "input.move_w" } target = { name = "Player" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed", use_dt = true } } ] }
  trigger "srv_move_s" { authority = "server" on = { event = "input.move_s" } target = { name = "Player" } actions = [ { translate_axis = { vec = [0,0, 1], speed_var = "speed", use_dt = true } } ] }
  trigger "srv_move_a" { authority = "server" on = { event = "input.move_a" } target = { name = "Player" } actions = [ { translate_axis = { vec = [-1,0,0], speed_var = "speed", use_dt = true } } ] }
  trigger "srv_move_d" { authority = "server" on = { event = "input.move_d" } target = { name = "Player" } actions = [ { translate_axis = { vec = [ 1,0,0], speed_var = "speed", use_dt = true } } ] }

  # Dash: apply cooldown server-side, replicate state
  trigger "srv_dash" {
    authority = "server"
    on = { event = "input.dash" }
    when = [ { expr = "dash_cd <= 0" } ]
    actions = [
      { set_var = { name = "dash_cd", value = 1.0 } }
    ]
  }
  trigger "srv_dash_tick"  { authority = "server" on = { tick = { every = 0.1 } } actions = [ { add_var = { name = "dash_cd", by = -0.1 } } ] }
  trigger "srv_dash_clamp" { authority = "server" on = { tick = { every = 0.2 } } when = [ { expr = "dash_cd < 0" } ] actions = [ { set_var = { name = "dash_cd", value = 0.0 } } ] }

  # Ability inputs → authoritative effects
  trigger "srv_q" { authority = "server" on = { event = "ability.q" } actions = [ { emit = { name = "combat.hit", payload = { damage = 40 } } ] }
  trigger "srv_w" { authority = "server" on = { event = "ability.w" } actions = [ { mul_var = { name = "speed", by = 1.4 } }, { set_timer = { name = "w_buff", seconds = 4.0, repeating = false } } ] }
  trigger "srv_w_end" { authority = "server" on = { timer = "w_buff" } actions = [ { mul_var = { name = "speed", by = 1.0/1.4 } } ] }
  trigger "srv_e" { authority = "server" on = { event = "ability.e" } actions = [ { emit = { name = "combat.hit", payload = { damage = 30 } } ] }
  trigger "srv_r" { authority = "server" on = { event = "ability.r" } actions = [ { emit = { name = "combat.hit", payload = { damage = 200 } } ] }

  # Damage application (example): reduce hp on hit
  trigger "apply_damage" { authority = "server" on = { event = "combat.hit" } actions = [ { add_var = { name = "hp", by = -10.0 } } ] }
  trigger "death_check" { authority = "server" on = { event = "combat.hit" } when = [ { expr = "hp <= 0" } ] actions = [ { emit = { name = "combat.death" } } ] }
} 

# Minimal position/combat model (no physics) using vars and tags

vars = { creep_speed = 1.0, tower_range = 8.0, tower_damage = 20.0, player_hp = 100.0, nexus_hp = 500.0 }

triggers {
  # Creep marching towards nexus: translate along -Z
  trigger "creep_march" { authority = "server" on = { tick = { every = 0.5 } } target = { tag = "Creep" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "creep_speed", use_dt = false } } ] }

  # Tower hits when in range (placeholder: periodic hit event)
  trigger "tower_fire" { authority = "server" on = { tick = { every = 1.0 } } actions = [ { emit = { name = "combat.hit", payload = { damage = "tower_damage" } } ] }

  # Apply damage to player/nexus (demo vars)
  trigger "apply_player_damage" { authority = "server" on = { event = "combat.hit" } actions = [ { add_var = { name = "player_hp", by = -10.0 } } ] }
  trigger "player_death" { authority = "server" on = { event = "combat.hit" } when = [ { expr = "player_hp <= 0" } ] actions = [ { send_chat = { channel = "global", text = "Player down!" } } ] }

  trigger "apply_nexus_damage" { authority = "server" on = { event = "combat.hit" } actions = [ { add_var = { name = "nexus_hp", by = -5.0 } } ] }
  trigger "victory" { authority = "server" on = { event = "combat.hit" } when = [ { expr = "nexus_hp <= 0" } ] actions = [ { send_chat = { channel = "global", text = "Victory!" } } ] }
} 

# Extended core systems: wards, shop, jungle respawn, hero respawn, teams

vars = { ward_duration = 90.0, gold = 0.0, kill_gold = 50.0, respawn_time = 10.0 }

triggers {
  # Ward placement on client signal (e.g., UI event)
  trigger "place_ward" { authority = "server" on = { event = "input.place_ward" } actions = [
    { spawn = { prefab = "core::Ward", components = { Name = "Ward" } } },
    { set_timer = { name = "ward_expire", seconds = "ward_duration", repeating = false } }
  ] }
  trigger "ward_expire" { authority = "server" on = { timer = "ward_expire" } actions = [ { despawn = { targets = { name = "Ward" } } } ] }

  # Shop purchase: spend gold to set a buff var (example)
  trigger "shop_buy_speed" { authority = "server" on = { event = "input.shop_speed" } when = [ { expr = "gold >= 300" } ] actions = [
    { add_var = { name = "gold", by = -300.0 } },
    { mul_var = { name = "speed", by = 1.1 } }
  ] }

  # Jungle camp respawn after defeated
  trigger "jungle_defeated" { authority = "server" on = { event = "jungle.defeated" } actions = [ { set_timer = { name = "jungle_respawn", seconds = 60.0, repeating = false } } ] }
  trigger "jungle_respawn"  { authority = "server" on = { timer = "jungle_respawn" } actions = [ { spawn = { prefab = "core::JungleCamp" } } ] }

  # Hero death → respawn after delay
  trigger "hero_death" { authority = "server" on = { event = "combat.death" } actions = [ { set_timer = { name = "hero_respawn", seconds = "respawn_time", repeating = false } }, { add_var = { name = "gold", by = "kill_gold" } } ] }
  trigger "hero_respawn" { authority = "server" on = { timer = "hero_respawn" } actions = [ { spawn = { prefab = "axe::Axe", components = { Name = "Player" } } } ] }

  # Team tags and objective (simple placeholders)
  trigger "team_assign" { authority = "server" on = { presence.join = true } actions = [ ] }
} 