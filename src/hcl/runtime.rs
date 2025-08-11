use bevy::prelude::*;
use crate::hcl::{
    loader::HclSceneAsset,
    registry::ApplyCtx,
    schema::{ActionDef, ConditionDef, EventDef, SceneDoc, Selector},
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
) {
    // Lazy compile when the entry scene is available
    if let Some(entry) = entry.as_ref().and_then(|e| e.0.as_ref()) {
        if let Some(asset) = assets.get(entry) {
            let id = entry.id();
            let needs_compile = runtime.compiled_for.map(|x| x != id).unwrap_or(true);
            if needs_compile {
                compile_runtime(&mut runtime, &asset.doc);
                runtime.compiled_for = Some(id);
                // Do not reset vars; users may want to retain, but ensure startup triggers re-fire
                runtime.startup_fired = false;
            }
        }
    }

    if runtime.compiled.is_empty() { return; }
    // dt variable for movement calculations
    runtime.vars.insert("dt".into(), time.delta_secs_f64());
    let startup_now = if !runtime.startup_fired { runtime.startup_fired = true; true } else { false };
    // flip event bus for this frame
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
        };
        if !fired { continue; }
        if !evaluate_conditions(&trig.when, &q_vis) { continue; }
        let sel = trig.target.clone();
        for a in &trig.actions {
            apply_action(a, sel.as_ref(), &mut commands, &mut q_vis, &mut q_xform, &mut q_mat, &prefabs, &registry, &mut ctx, &mut runtime.vars);
        }
    }
    runtime.compiled = compiled;
}

fn compile_runtime(rt: &mut HclRuntime, doc: &SceneDoc) {
    rt.compiled.clear();
    rt.prefabs.clear();
    for p in &doc.prefab {
        rt.prefabs.insert(p.name.clone(), p.components.clone());
    }
    for (k, v) in &doc.vars { rt.vars.entry(k.clone()).or_insert(*v); }
    for t in &doc.triggers {
        if let Some(on) = compile_event(&t.on) {
            rt.compiled.push(CompiledTrigger {
                name: t.name.clone(),
                on,
                when: t.when.clone(),
                actions: t.actions.clone(),
                target: t.target.clone(),
            });
        }
    }
}

fn compile_event(ev: &EventDef) -> Option<EventMatcher> {
    match ev {
        EventDef::KeyPressed { key_pressed } => parse_key_code(key_pressed).map(EventMatcher::KeyPressed),
        EventDef::KeyHeld { key_held } => parse_key_code(key_held).map(EventMatcher::KeyHeld),
        EventDef::Tick { tick } => Some(EventMatcher::Tick(Timer::from_seconds(tick.every.max(0.0001), TimerMode::Repeating))),
        EventDef::Startup { .. } => Some(EventMatcher::Startup),
        EventDef::Event { event } => Some(EventMatcher::Bus(event.clone())),
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
    }
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
            // Spawning via runtime is not yet implemented with modular appliers
            // This can be wired to a dedicated spawn queue if needed
        }
        ActionDef::Despawn { despawn } => {
            let targets = despawn.targets.as_ref().or(inherited_sel);
            for_each_selected(q_vis, targets.unwrap_or(&Selector::All { all: true }), |e, _, _| { commands.entity(e).despawn_recursive(); });
        }
        ActionDef::SetVar { set_var } => { vars.insert(set_var.name.clone(), set_var.value); }
        ActionDef::AddVar { add_var } => { let e = vars.entry(add_var.name.clone()).or_insert(0.0); *e += add_var.by; }
        ActionDef::MulVar { mul_var } => { let e = vars.entry(mul_var.name.clone()).or_insert(0.0); *e *= mul_var.by; }
        ActionDef::Emit { emit } => { EVENTS.write().expect("event bus lock").emit(emit.name.clone()); }
        _ => { /* MOBA-specific actions not yet implemented at runtime */ }
    }
}

#[derive(Default)]
struct EventBus {
    current: std::collections::HashSet<String>,
    next: std::collections::HashSet<String>,
}

impl EventBus {
    fn flip(&mut self) { self.current.clear(); std::mem::swap(&mut self.current, &mut self.next); }
    fn contains(&self, name: &str) -> bool { self.current.contains(name) }
    fn emit(&mut self, name: String) { self.next.insert(name); }
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

