use bevy::prelude::*;
use crate::hcl::{
    loader::HclSceneAsset,
    registry::ApplyCtx,
    schema::{ActionDef, ConditionDef, EventDef, FunctionDecl, SceneDoc, Selector},
    types::HclTags,
};
use ahash::AHashMap as HashMap;
use std::sync::RwLock;

#[derive(Resource, Default)]
pub struct HclRuntime {
    compiled: Vec<CompiledTrigger>,
    prefabs: HashMap<String, serde_json::Value>,
    startup_fired: bool,
    compiled_for: Option<AssetId<HclSceneAsset>>,
    vars: HashMap<String, f64>,
    functions: HashMap<String, FunctionDecl>,
    timers: HashMap<String, Timer>,
    pub recent: std::collections::VecDeque<String>,
}

impl HclRuntime {
    pub fn overlay_line(&self, max_vars: usize, max_recent: usize) -> String {
        let mut parts: Vec<String> = Vec::new();
        // Vars (sorted, limited)
        let mut kv: Vec<_> = self.vars.iter().collect();
        kv.sort_by(|a, b| a.0.cmp(b.0));
        let mut taken = 0usize;
        for (k, v) in kv {
            if taken >= max_vars { break; }
            if k == "dt" { continue; }
            parts.push(format!("{}={:.2}", k, v));
            taken += 1;
        }
        // Recent
        if !self.recent.is_empty() {
            let mut tail = Vec::new();
            for s in self.recent.iter().rev().take(max_recent) { tail.push(s.clone()); }
            parts.push(format!("recent=[{}]", tail.join(" | ")));
        }
        parts.join("  ")
    }
    pub fn get_var(&self, key: &str) -> Option<f64> { self.vars.get(key).copied() }
}

struct CompiledTrigger {
    name: Option<String>,
    on: EventMatcher,
    when: Vec<ConditionDef>,
    actions: Vec<ActionDef>,
    target: Option<Selector>,
}

enum EventMatcher {
    KeyPressed(KeyCode),
    KeyHeld(KeyCode),
    Tick(Timer),
    Startup,
    Bus(String),
    Timer(String),
}

static EVENTS: once_cell::sync::Lazy<RwLock<EventBus>> = once_cell::sync::Lazy::new(|| RwLock::new(EventBus::default()));

pub fn process_triggers(
    mut runtime: ResMut<HclRuntime>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    assets: Res<Assets<HclSceneAsset>>,
    entry: Option<Res<crate::hcl::HclEntry>>,
    mut commands: Commands,
    mut q_vis: Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
    mut q_xform: Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Transform>)>,
    mut q_mat: Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut bevy::pbr::MeshMaterial3d<StandardMaterial>>)>,
    registry: Res<crate::hcl::registry::ComponentRegistry>,
    mut ctx: ResMut<ApplyCtx>,
    // Editor mode gate
    mode: Option<Res<crate::hcl::EditorState>>,
) {
    // Pause gameplay logic in Edit mode
    if let Some(mode) = mode.as_ref() {
        if matches!(mode.0, crate::hcl::EditorMode::Edit) { return; }
    }

    // Lazy compile when the entry scene is available
    if let Some(entry) = entry.as_ref().and_then(|e| e.0.as_ref()) {
        if let Some(asset) = assets.get(entry) {
            let id = entry.id();
            let needs_compile = runtime.compiled_for.map(|x| x != id).unwrap_or(true);
            if needs_compile {
                compile_runtime(&mut runtime, &asset.doc);
                runtime.compiled_for = Some(id);
                runtime.startup_fired = false;
            }
        }
    }

    if runtime.compiled.is_empty() { return; }

    // Move vars and timers out to avoid overlapping mutable borrows of runtime
    let mut vars = std::mem::take(&mut runtime.vars);
    let mut timers = std::mem::take(&mut runtime.timers);

    // Update dt and advance timers
    vars.insert("dt".into(), time.delta_secs_f64());
    for (_name, t) in timers.iter_mut() { t.tick(time.delta()); }

    let startup_now = if !runtime.startup_fired { runtime.startup_fired = true; true } else { false };
    EVENTS.write().expect("event bus lock").flip();
    let prefabs = runtime.prefabs.clone();
    let mut compiled = std::mem::take(&mut runtime.compiled);
    for trig in &mut compiled {
        let fired = match &mut trig.on {
            EventMatcher::KeyPressed(k) => keys.just_pressed(*k),
            EventMatcher::KeyHeld(k) => keys.pressed(*k),
            EventMatcher::Tick(timer) => timer.tick(time.delta()).just_finished(),
            EventMatcher::Startup => startup_now,
            EventMatcher::Bus(name) => EVENTS.read().expect("event bus lock").contains(name),
            EventMatcher::Timer(name) => timers.get(name).map(|t| t.just_finished()).unwrap_or(false),
        };
        if !fired { continue; }
        if !evaluate_conditions(&trig.when, &q_vis) { continue; }
        let tname = trig.name.as_deref().unwrap_or("<unnamed>").to_string();
        info!("HCL Trigger fired: {:?}", tname);
        runtime.recent.push_back(format!("fired {tname}"));
        while runtime.recent.len() > 32 { runtime.recent.pop_front(); }
        let sel = trig.target.clone();
        for a in &trig.actions {
            log_action_debug(a);
            apply_action(a, sel.as_ref(), &mut commands, &mut q_vis, &mut q_xform, &mut q_mat, &prefabs, &registry, &mut ctx, &mut vars, &mut timers);
        }
    }
    // Write back
    runtime.compiled = compiled;
    runtime.vars = vars;
    runtime.timers = timers;
}

fn compile_runtime(rt: &mut HclRuntime, doc: &SceneDoc) {
    rt.compiled.clear();
    rt.prefabs.clear();
    rt.functions.clear();
    for p in &doc.prefab { rt.prefabs.insert(p.name.clone(), p.components.clone()); }
    for (k, v) in &doc.vars { rt.vars.entry(k.clone()).or_insert(*v); }
    for f in &doc.functions { rt.functions.insert(f.name.clone(), f.clone()); }
    rt.compiled.reserve(doc.triggers.len());
    for t in &doc.triggers {
        if let Some(on) = compile_event(&t.on) {
            rt.compiled.push(CompiledTrigger { name: t.name.clone(), on, when: t.when.clone(), actions: t.actions.clone(), target: t.target.clone() });
        }
    }
    info!("HCL compiled: {} prefabs, {} triggers, {} functions", rt.prefabs.len(), rt.compiled.len(), rt.functions.len());
}

fn compile_event(ev: &EventDef) -> Option<EventMatcher> {
    match ev {
        EventDef::KeyPressed { key_pressed } => parse_key_code(key_pressed).map(EventMatcher::KeyPressed),
        EventDef::KeyHeld { key_held } => parse_key_code(key_held).map(EventMatcher::KeyHeld),
        EventDef::Tick { tick } => Some(EventMatcher::Tick(Timer::from_seconds(tick.every.max(0.0001), TimerMode::Repeating))),
        EventDef::Startup { .. } => Some(EventMatcher::Startup),
        EventDef::Event { event } => Some(EventMatcher::Bus(event.clone())),
        EventDef::Timer { timer } => Some(EventMatcher::Timer(timer.clone())),
    }
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    use KeyCode::*;
    let s = s.trim();
    Some(match s {
        "Space" => Space,
        "Enter" => Enter,
        "Escape" => Escape,
        "ArrowLeft" => ArrowLeft,
        "ArrowRight" => ArrowRight,
        "ArrowUp" => ArrowUp,
        "ArrowDown" => ArrowDown,
        "KeyW" | "W" | "w" => KeyW,
        "KeyA" | "A" | "a" => KeyA,
        "KeyS" | "S" | "s" => KeyS,
        "KeyD" | "D" | "d" => KeyD,
        "KeyE" | "E" | "e" => KeyE,
        other => { warn!("Unknown KeyCode: {other}"); return None; }
    })
}

fn evaluate_conditions(
    conds: &Vec<ConditionDef>,
    q_vis: &Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
) -> bool {
    if conds.is_empty() { return true; }
    conds.iter().all(|c| eval_cond(c, q_vis))
}

fn eval_cond(
    cond: &ConditionDef,
    q_vis: &Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
) -> bool {
    match cond {
        ConditionDef::Any { any_visible } => any_selected(q_vis, any_visible, |vis| match vis { Some(v) => matches!(*v, Visibility::Visible), None => false }),
        ConditionDef::All { all_visible } => all_selected(q_vis, all_visible, |vis| match vis { Some(v) => matches!(*v, Visibility::Visible), None => false }),
        ConditionDef::Not { not } => !eval_cond(not, q_vis),
        ConditionDef::Expr { expr } => eval_expr_bool(expr),
    }
}

fn eval_expr_bool(expr: &str) -> bool { evalexpr::eval_boolean(expr).unwrap_or(false) }
fn eval_expr_f64(expr: &str, vars: &HashMap<String, f64>) -> Option<f64> {
    use evalexpr::*;
    let mut ctx = HashMapContext::new();
    for (k, v) in vars { let _ = ctx.set_value(k.clone(), (*v).into()); }
    eval_with_context(expr, &ctx).ok().and_then(|v| v.as_number().ok()).map(|n| n as f64)
}

fn apply_action(
    action: &ActionDef,
    inherited_sel: Option<&Selector>,
    commands: &mut Commands,
    q_vis: &mut Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
    q_xform: &mut Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Transform>)>,
    q_mat: &mut Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut bevy::pbr::MeshMaterial3d<StandardMaterial>>)>,
    prefabs: &HashMap<String, serde_json::Value>,
    registry: &crate::hcl::registry::ComponentRegistry,
    ctx: &mut ApplyCtx,
    vars: &mut HashMap<String, f64>,
    timers: &mut HashMap<String, Timer>,
) {
    match action {
        ActionDef::ToggleVisibility { toggle_visibility } => {
            let targets = toggle_visibility.targets.as_ref().or(inherited_sel);
            for_each_selected_vis(q_vis, targets, |vis| {
                *vis = match *vis { Visibility::Visible => Visibility::Hidden, _ => Visibility::Visible };
            });
        }
        ActionDef::SetVisibility { set_visibility } => {
            let targets = set_visibility.targets.as_ref().or(inherited_sel);
            let value = set_visibility
                .value
                .as_deref()
                .unwrap_or("Visible");
            let new_vis = match value { "Hidden" => Visibility::Hidden, "Inherited" => Visibility::Inherited, _ => Visibility::Visible };
            for_each_selected_vis(q_vis, targets, |vis| { *vis = new_vis; });
        }
        ActionDef::Translate { translate } => {
            let targets = translate.targets.as_ref().or(inherited_sel);
            let by = translate.by;
            for_each_selected_xform(q_xform, targets, |t| { t.translation += Vec3::new(by[0], by[1], by[2]); });
        }
        ActionDef::TranslateAxis { translate_axis } => {
            let targets = translate_axis.targets.as_ref().or(inherited_sel);
            let spd = *vars.get(&translate_axis.speed_var).unwrap_or(&0.0) as f32;
            let dt = if translate_axis.use_dt { *vars.get("dt").unwrap_or(&0.0) as f32 } else { 1.0 };
            let v = Vec3::new(translate_axis.vec[0], translate_axis.vec[1], translate_axis.vec[2]) * (spd * dt);
            for_each_selected_xform(q_xform, targets, |t| { t.translation += v; });
        }
        ActionDef::RotateEuler { rotate_euler } => {
            let targets = rotate_euler.targets.as_ref().or(inherited_sel);
            let by = &rotate_euler.by;
            let rot = Quat::from_euler(EulerRot::YXZ, by.y.to_radians(), by.x.to_radians(), by.z.to_radians());
            for_each_selected_xform(q_xform, targets, |t| { t.rotation = rot * t.rotation; });
        }
        ActionDef::SetMaterial { set_material } => {
            let targets = set_material.targets.as_ref().or(inherited_sel);
            let Some(mat_h) = ctx.materials.get(&set_material.material).cloned() else { warn!("Unknown material {}", set_material.material); return; };
            for_each_selected_mat(q_mat, targets, |m| { m.0 = mat_h.clone(); });
        }
        ActionDef::Spawn { spawn } => {
            let mut merged = serde_json::json!({});
            if let Some(pref) = &spawn.prefab { if let Some(p) = prefabs.get(pref) { merge_json(&mut merged, p.clone()); } }
            merge_json(&mut merged, spawn.components.clone());
            let mut ec = commands.spawn((
                Name::new("Spawned"),
                HclTags::default(),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
            ));
            if let Some(parent_sel) = spawn.parent.as_ref() { if let Some(parent) = find_first_entity(parent_sel, q_vis) { ec.insert(ChildOf(parent)); } }
            let mut items: Vec<(&'static str, &Box<dyn crate::hcl::registry::ComponentApplier>)> = registry.iter().collect();
            items.sort_by_key(|(_, a)| a.priority());
            if let Some(obj) = merged.as_object() {
                let mut scratch = crate::hcl::registry::EntityScratch::default();
                for (key, applier) in items { if let Some(payload) = obj.get(key) { let _ = applier.apply(payload, &mut ec, &mut scratch, ctx); } }
            }
            info!("  action: spawn -> entity {:?}", ec.id());
        }
        ActionDef::Despawn { despawn } => {
            let targets = despawn.targets.as_ref().or(inherited_sel);
            for_each_selected(q_vis, targets.unwrap_or(&Selector::All { all: true }), |e, _, _| { commands.entity(e).despawn(); });
        }
        ActionDef::SetVar { set_var } => { vars.insert(set_var.name.clone(), set_var.value); }
        ActionDef::AddVar { add_var } => { let e = vars.entry(add_var.name.clone()).or_insert(0.0); *e += add_var.by; }
        ActionDef::MulVar { mul_var } => { let e = vars.entry(mul_var.name.clone()).or_insert(0.0); *e *= mul_var.by; }
        ActionDef::Emit { emit } => {
            if let Some(p) = &emit.payload { let map: HashMap<String, f64> = p.iter().map(|(k,v)| (k.clone(), *v)).collect(); EVENTS.write().expect("event bus lock").emit_with_payload(emit.name.clone(), map); }
            else { EVENTS.write().expect("event bus lock").emit(emit.name.clone()); }
        }
        ActionDef::Eval { eval } => { if let Some(v) = eval_expr_f64(&eval.expr, vars) { if let Some(k) = &eval.store_as { vars.insert(k.clone(), v); } } }
        ActionDef::SetTimer { set_timer } => {
            let mode = if set_timer.repeating.unwrap_or(true) { TimerMode::Repeating } else { TimerMode::Once };
            timers.insert(set_timer.name.clone(), Timer::from_seconds(set_timer.seconds.max(0.0001), mode));
        }
        ActionDef::Apply { apply } => {
            let targets = apply.targets.as_ref().or(inherited_sel);
            // Support a minimal subset of paths: Transform.t, Transform.s, Visibility, StandardMaterialRef.material
            let path = apply.path.as_str();
            if path == "Visibility" {
                for_each_selected_vis(q_vis, targets, |_vis| { /* could toggle/set via value string later */ });
            } else if path == "Transform.t" {
                if let Some(arr) = apply.value.as_array().and_then(|a| if a.len()==3 { Some([a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32]) } else { None }) {
                    for_each_selected_xform(q_xform, targets, |t| { t.translation = Vec3::new(arr[0], arr[1], arr[2]); });
                }
            } else if path == "Transform.s" {
                if let Some(arr) = apply.value.as_array().and_then(|a| if a.len()==3 { Some([a[0].as_f64().unwrap_or(1.0) as f32, a[1].as_f64().unwrap_or(1.0) as f32, a[2].as_f64().unwrap_or(1.0) as f32]) } else { None }) {
                    for_each_selected_xform(q_xform, targets, |t| { t.scale = Vec3::new(arr[0], arr[1], arr[2]); });
                }
            } else if path == "StandardMaterialRef.material" {
                if let Some(mat_name) = apply.value.as_str() {
                    if let Some(mat) = ctx.materials.get(mat_name).cloned() {
                        for_each_selected_mat(q_mat, targets, |m| { m.0 = mat.clone(); });
                    }
                }
            }
        }
        _ => {}
    }
}

fn log_action_debug(a: &ActionDef) {
    match a {
        ActionDef::ToggleVisibility { .. } => info!("  action: toggle_visibility"),
        ActionDef::SetVisibility { .. } => info!("  action: set_visibility"),
        ActionDef::Translate { translate } => info!("  action: translate by {:?}", translate.by),
        ActionDef::TranslateAxis { translate_axis } => info!("  action: translate_axis vec={:?} speed_var={}", translate_axis.vec, translate_axis.speed_var),
        ActionDef::RotateEuler { .. } => info!("  action: rotate_euler"),
        ActionDef::SetMaterial { set_material } => info!("  action: set_material {}", set_material.material),
        ActionDef::Spawn { .. } => info!("  action: spawn"),
        ActionDef::Despawn { .. } => info!("  action: despawn"),
        ActionDef::SetVar { set_var } => info!("  action: set_var {}={} ", set_var.name, set_var.value),
        ActionDef::AddVar { add_var } => info!("  action: add_var {}+= {}", add_var.name, add_var.by),
        ActionDef::MulVar { mul_var } => info!("  action: mul_var {}*= {}", mul_var.name, mul_var.by),
        ActionDef::Emit { emit } => info!("  action: emit {}", emit.name),
        ActionDef::Eval { eval } => info!("  action: eval '{}' store_as={:?}", eval.expr, eval.store_as),
        ActionDef::SetTimer { set_timer } => info!("  action: set_timer name={} seconds={} repeating={}", set_timer.name, set_timer.seconds, set_timer.repeating.unwrap_or(true)),
        ActionDef::Apply { apply } => info!("  action: apply path={} value={}", apply.path, apply.value),
    }
}

#[derive(Default)]
struct EventBus {
    current: std::collections::HashSet<String>,
    next: std::collections::HashSet<String>,
    payloads: HashMap<String, HashMap<String, f64>>, // simple numeric payloads
}

impl EventBus {
    fn flip(&mut self) { self.current.clear(); std::mem::swap(&mut self.current, &mut self.next); self.payloads.clear(); }
    fn contains(&self, name: &str) -> bool { self.current.contains(name) }
    fn emit(&mut self, name: String) { self.next.insert(name); }
    fn emit_with_payload(&mut self, name: String, payload: HashMap<String, f64>) { self.next.insert(name.clone()); self.payloads.insert(name, payload); }
    fn get_payload(&self, name: &str) -> Option<&HashMap<String, f64>> { self.payloads.get(name) }
}

fn for_each_selected<'a>(
    q: &Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
    selector: &Selector,
    mut f: impl FnMut(Entity, Option<&Name>, Option<&HclTags>),
) {
    for (e, name, tags, _vis) in q.iter() {
        if matches_selector(selector, name, tags) { f(e, name, tags); }
    }
}

fn for_each_selected_vis<'a>(
    q: &mut Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&'a mut Visibility>)>,
    selector: Option<&Selector>,
    mut f: impl FnMut(&mut Visibility),
) {
    let Some(sel) = selector else { return; };
    for (_e, name, tags, vis) in q.iter_mut() {
        if matches_selector(sel, name, tags) {
            if let Some(mut v) = vis { f(&mut v); }
        }
    }
}

fn for_each_selected_xform<'a>(
    q: &mut Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&'a mut Transform>)>,
    selector: Option<&Selector>,
    mut f: impl FnMut(&mut Transform),
) {
    let Some(sel) = selector else { return; };
    for (_e, name, tags, tf) in q.iter_mut() {
        if matches_selector(sel, name, tags) {
            if let Some(mut t) = tf { f(&mut t); }
        }
    }
}

fn for_each_selected_mat<'a>(
    q: &mut Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&'a mut bevy::pbr::MeshMaterial3d<StandardMaterial>>)>,
    selector: Option<&Selector>,
    mut f: impl FnMut(&mut bevy::pbr::MeshMaterial3d<StandardMaterial>),
) {
    let Some(sel) = selector else { return; };
    for (_e, name, tags, mm) in q.iter_mut() {
        if matches_selector(sel, name, tags) {
            if let Some(mut m) = mm { f(&mut m); }
        }
    }
}

fn any_selected(
    q: &Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
    selector: &Selector,
    predicate: impl Fn(Option<&Visibility>) -> bool,
) -> bool {
    let mut any = false;
    for (_e, name, tags, vis) in q.iter() {
        if matches_selector(selector, name, tags) {
            if predicate(vis.as_deref()) { any = true; break; }
        }
    }
    any
}

fn all_selected(
    q: &Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
    selector: &Selector,
    predicate: impl Fn(Option<&Visibility>) -> bool,
) -> bool {
    let mut found = false;
    for (_e, name, tags, vis) in q.iter() {
        if matches_selector(selector, name, tags) {
            found = true;
            if !predicate(vis.as_deref()) { return false; }
        }
    }
    found
}

fn matches_selector(sel: &Selector, name: Option<&Name>, tags: Option<&HclTags>) -> bool {
    match sel {
        Selector::All { all } => *all,
        Selector::Name { name: n } => name.map(|nm| nm.as_str() == n.as_str()).unwrap_or(false),
        Selector::Tag { tag } => tags.map(|t| t.0.iter().any(|x| x == tag)).unwrap_or(false),
    }
}

fn find_first_entity(
    selector: &Selector,
    q: &Query<(Entity, Option<&Name>, Option<&HclTags>, Option<&mut Visibility>)>,
) -> Option<Entity> {
    for (e, name, tags, _vis) in q.iter() {
        if matches_selector(selector, name, tags) { return Some(e); }
    }
    None
}

fn merge_json(dst: &mut serde_json::Value, src: serde_json::Value) {
    match (dst, src) {
        (serde_json::Value::Object(d), serde_json::Value::Object(s)) => {
            for (k, v) in s { merge_json(d.entry(k).or_insert(serde_json::Value::Null), v); }
        }
        (d, s) => *d = s,
    }
} 