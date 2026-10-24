//! **A cena do smoke «o Auto Key grava o que não é número»** (F27 onda E) — `PH2D_AUTOKEY_SMOKE=32`.
//!
//! Três objectos, três famílias que a onda E tornou canais, gravadas pelo Auto Key no quadro 12 e
//! desfeitas no 24; a cena pára no 18 e mede o que cada curva dá a meio:
//!
//! - **«Red» / «Blue»** — duas sprites sobrepostas; o Blue começa por cima (mesma camada,
//!   `OrderInLayer` maior). No 12 a `SortingLayer` do Red passa a «Front» → o Red fica por cima.
//! - **«Score»** — um rótulo do HUD a ler o contador `coins`; no 12 a *Source* passa a `TagCount` e o
//!   *Source Name* a `Enemy` (dois canais).
//! - **«Spinner»** — corre `spin.luau` com `speed` próprio; no 12 `speed = 6` e o ficheiro passa a
//!   `bob.luau`; no 24 volta a `speed = 1` e `spin.luau`.
//!
//! ⚠️ Os canais por DEGRAUS (camada, fonte, ficheiro) seguram o valor do 12 até ao 24; só o `speed`
//! interpola (`3,5` no 18). O dispatcher `ph2d_app_vec::autokey_smoke` reserva o `"32"` para aqui.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::sorting::{LayerId, OrderInLayer, SortingLayer, SortingLayers};
use ph2d_ecs::{Counter, Entity, LabelSource, Name, SimWorld, Transform, UiLabel};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_script::{LuauScript, ProjectScripts, ScriptId, ScriptSource, ScriptValue};
use ph2d_timeline::{PropKind, TimelineState};

/// O número da cena no `PH2D_AUTOKEY_SMOKE`.
pub const CENA: &str = "32";
/// O nome do script A.
pub const SPIN_NAME: &str = "spin.luau";
/// O nome do script B (o `bob.luau` do smoke dos scripts).
pub const BOB_NAME: &str = "bob.luau";
/// O nome da camada que a cena acrescenta.
pub const FRONT: &str = "Front";

/// ⭐ **O script A** — gira o objecto. Os comentários são para o DONO.
pub const SPIN_LUAU: &str = r#"-- spin.luau — gira o objeto.
--
-- O «speed» aparece no painel (secção Script). Com o Auto Key ligado,
-- mudar o número num quadro grava uma chave nesse quadro.

ph2d.property("speed", 1.0, { min = 0, max = 10 }) -- voltas por segundo

function update(self, dt)
  local r = ph2d.get(self.id, "rotation") or 0
  ph2d.set(self.id, "rotation", r + self.speed * 2 * math.pi * dt)
end
"#;

const QUADRO_12: f64 = 0.5;
const QUADRO_18: f64 = 0.75;
const QUADRO_24: f64 = 1.0;
const SPEED_PARTIDA: f64 = 1.0;
const SPEED_Q12: f64 = 6.0;
const COUNTER: &str = "coins";
const TAG: &str = "Enemy";
const RED_RGBA: [f32; 4] = [0.92, 0.30, 0.30, 1.0];
const BLUE_RGBA: [f32; 4] = [0.30, 0.50, 0.95, 1.0];
const SCORE_RGBA: [f32; 4] = [0.95, 0.80, 0.30, 1.0];
const SPIN_RGBA: [f32; 4] = [0.40, 0.85, 0.55, 1.0];
/// A altura de repouso dos corpos — acima do centro, para a faixa do fundo mais alta não os cortar.
const Y: f32 = 0.5;
/// A faixa do fundo com lugar para as cinco linhas e o resumo (a de fábrica corta as duas do Score).
const FUNDO_H: f32 = ph2d_editor_core::screens::layout::TIMELINE_DOCK_H + 110.0; // LITERAL-PX-OK: duas linhas da dope-sheet e a barra de estado

static ESTAGIO: AtomicU8 = AtomicU8::new(0);
static CENA_MONTADA: Mutex<Option<Montada>> = Mutex::new(None);

/// Quem a cena montou.
#[derive(Clone, Copy, Debug)]
struct Montada {
    red: u64,
    blue: u64,
    score: u64,
    spinner: u64,
    spin: ScriptId,
    bob: ScriptId,
    front: LayerId,
    default: LayerId,
}

/// Um passo da cena por quadro; no-op sem `PH2D_AUTOKEY_SMOKE=32`.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    let s = match ESTAGIO.load(Ordering::Relaxed) {
        0 => {
            let armada = std::env::var("PH2D_AUTOKEY_SMOKE").is_ok_and(|v| v == CENA);
            ESTAGIO.store(if armada { 1 } else { u8::MAX }, Ordering::Relaxed);
            if !armada {
                return None;
            }
            1
        }
        u8::MAX => return None,
        s => s,
    };
    if s < u8::MAX - 1 {
        ESTAGIO.store(s + 1, Ordering::Relaxed);
    }
    if s == 1 {
        let m = montar(sim);
        if let Ok(mut c) = CENA_MONTADA.lock() {
            *c = Some(m);
        }
        // O «Spinner» é o primário — o Inspector mostra-o; os outros dois entram na população.
        hero.gizmo.replace_selection(Some(m.spinner));
        hero.gizmo.add_to_selection(m.red);
        hero.gizmo.add_to_selection(m.score);
        hero.set_panel_visible("timeline", true);
        hero.store.set_dock_bottom_h(FUNDO_H);
        hero.set_panel_visible("inspector", true);
        hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        for id in [
            ph2d_editor_core::ids::INSP_LIVE_SCRIPT_SECTION,
            ph2d_editor_core::ids::INSP_LIVE_HUD_SECTION,
            ph2d_editor_core::ids::INSP_LIVE_ORDERING_SECTION,
        ] {
            hero.store.set_collapsed(id, false);
        }
        return Some(format!("[autokey-smoke] =32 montada · {}", medir(sim, &m)));
    }
    let m = CENA_MONTADA.lock().ok().and_then(|c| *c)?;
    match s {
        10 => {
            tl.flags.auto_key = true;
            ph.pause();
            ph.seek(QUADRO_12);
            None
        }
        16 => {
            mexer(sim, &m, true);
            None
        }
        26 => {
            ph.seek(QUADRO_24);
            None
        }
        32 => {
            mexer(sim, &m, false);
            None
        }
        38 => Some(
            [
                ("Red", m.red),
                ("Blue", m.blue),
                ("Score", m.score),
                ("Spinner", m.spinner),
            ]
            .iter()
            .map(|(n, b)| format!("[autokey-smoke] trilhas de {n}: {}", trilhas(tl, *b)))
            .collect::<Vec<_>>()
            .join("\n"),
        ),
        42 => {
            tl.flags.auto_key = false;
            ph.seek(QUADRO_18);
            None
        }
        52 => Some(veredito(sim, tl, &m)),
        90 => Some(format!("[autokey-smoke] fim · {}", medir(sim, &m))),
        _ => None,
    }
}

fn corpo(sim: &mut SimWorld, nome: &str, x: f32, y: f32, rgba: [f32; 4]) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome.to_owned()),
            Transform::from_translation(Vec2::new(x, y)),
            Sprite::atlas(WHITE_TILE_KEY, [2.0, 2.0], rgba),
        ))
        .id()
}

fn montar(sim: &mut SimWorld) -> Montada {
    let w = sim.world_mut();
    let mut camadas = w
        .remove_resource::<SortingLayers>()
        .unwrap_or_else(SortingLayers::canonical);
    let default = LayerId(camadas.default_index());
    let existente = camadas.iter().find(|(_, n)| *n == FRONT).map(|(id, _)| id);
    let front = existente.or_else(|| camadas.push(FRONT)).unwrap_or(default);
    w.insert_resource(camadas);
    let mut scripts = w.get_resource_or_init::<ProjectScripts>();
    let spin = scripts.insert(SPIN_NAME, SPIN_LUAU);
    let bob = scripts.insert(BOB_NAME, crate::script_smoke::BOB_LUAU);

    let red = corpo(sim, "Red", -3.0, Y, RED_RGBA);
    sim.world_mut()
        .entity_mut(red)
        .insert((SortingLayer(default), OrderInLayer(0)));
    let blue = corpo(sim, "Blue", -2.0, Y - 0.5, BLUE_RGBA);
    sim.world_mut().entity_mut(blue).insert(OrderInLayer(1));

    let score = corpo(sim, "Score", 1.5, Y, SCORE_RGBA);
    sim.world_mut().entity_mut(score).insert((
        UiLabel {
            source: LabelSource::Counter(COUNTER.to_owned()),
            prefix: "Coins: ".to_owned(),
            suffix: String::new(),
        },
        Counter {
            name: COUNTER.to_owned(),
            start: 5,
            keep_on_restart: false,
        },
    ));

    let spinner = corpo(sim, "Spinner", 4.5, Y, SPIN_RGBA);
    let mut cfg = LuauScript::library(spin);
    cfg.own
        .insert("speed".into(), ScriptValue::Number(SPEED_PARTIDA));
    sim.world_mut().entity_mut(spinner).insert(cfg);

    Montada {
        red: red.to_bits(),
        blue: blue.to_bits(),
        score: score.to_bits(),
        spinner: spinner.to_bits(),
        spin,
        bob,
        front,
        default,
    }
}

/// `ida` = o quadro 12 (Front · TagCount Enemy · bob 6); senão a volta do 24.
fn mexer(sim: &mut SimWorld, m: &Montada, ida: bool) {
    let w = sim.world_mut();
    let layer = if ida { m.front } else { m.default };
    w.entity_mut(Entity::from_bits(m.red))
        .insert(SortingLayer(layer));
    if let Some(mut l) = w.get_mut::<UiLabel>(Entity::from_bits(m.score)) {
        l.source = if ida {
            LabelSource::TagCount(TAG.to_owned())
        } else {
            LabelSource::Counter(COUNTER.to_owned())
        };
    }
    if let Some(mut s) = w.get_mut::<LuauScript>(Entity::from_bits(m.spinner)) {
        let (speed, id) = if ida {
            (SPEED_Q12, m.bob)
        } else {
            (SPEED_PARTIDA, m.spin)
        };
        s.own.insert("speed".into(), ScriptValue::Number(speed));
        s.source = ScriptSource::Library(id);
    }
}

/// «Nome N×Mch» por grupo de canais, como a dope-sheet os junta.
fn trilhas(tl: &TimelineState, bits: u64) -> String {
    let mut por: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for b in tl.doc.bindings().iter().filter(|b| b.entity == bits) {
        let nome = match b.prop {
            PropKind::Field(k) => {
                format!("{}[{}:{}]", k.info().map_or("?", |i| i.rotulo), k.i, k.j)
            }
            p => format!("{p:?}"),
        };
        let n = tl
            .doc
            .active_clip()
            .track(b.target)
            .map_or(0, |t| t.keys().len());
        let e = por.entry(nome).or_default();
        e.0 += 1;
        e.1 = e.1.max(n);
    }
    if por.is_empty() {
        return "nenhuma".to_owned();
    }
    por.iter()
        .map(|(g, (n, ch))| format!("{g} {n}×{ch}ch"))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn ok(cond: bool, falha: &str) -> String {
    if cond {
        "OK".to_owned()
    } else {
        format!("FAIL — {falha}")
    }
}

fn veredito(sim: &SimWorld, tl: &TimelineState, m: &Montada) -> String {
    let w = sim.world();
    let layer = w.get::<SortingLayer>(Entity::from_bits(m.red)).map(|l| l.0);
    let fonte = w
        .get::<UiLabel>(Entity::from_bits(m.score))
        .map(|l| l.source.clone());
    let cfg = w.get::<LuauScript>(Entity::from_bits(m.spinner));
    let source = cfg.map(|c| c.source.clone());
    let speed = cfg.and_then(|c| match c.own.get("speed") {
        Some(ScriptValue::Number(n)) => Some(*n),
        _ => None,
    });
    let meio = (SPEED_PARTIDA + SPEED_Q12) * 0.5;
    let faixas = tl
        .doc
        .bindings()
        .iter()
        .filter(|b| [m.red, m.blue, m.score, m.spinner].contains(&b.entity))
        .count();
    [
        format!(
            "[autokey-smoke] q18 Red layer {layer:?} (Front = {:?}): {}",
            m.front,
            ok(
                layer == Some(m.front),
                "o degrau do 12 não segura o Red por cima"
            )
        ),
        format!(
            "[autokey-smoke] q18 Score source {fonte:?}: {}",
            ok(
                fonte == Some(LabelSource::TagCount(TAG.to_owned())),
                "a fonte ou o nome não seguiram a chave do 12"
            )
        ),
        format!(
            "[autokey-smoke] q18 Spinner script {source:?} (bob = {:?}): {}",
            m.bob,
            ok(
                source == Some(ScriptSource::Library(m.bob)),
                "o ficheiro não seguiu a chave do 12"
            )
        ),
        format!(
            "[autokey-smoke] q18 Spinner speed {speed:?} (perto de {meio:.1}): {}",
            ok(
                speed.is_some_and(|v| (v - meio).abs() < 0.3),
                "a curva do speed não conduz o próprio"
            )
        ),
        format!(
            "[autokey-smoke] {faixas} faixas (tem de ser {})",
            FAIXAS_ESPERADAS
        ),
    ]
    .join("\n")
}

/// Red camada 1 · Score fonte 1 + nome 1 · Spinner ficheiro 1 + speed 1.
const FAIXAS_ESPERADAS: usize = 5;

fn medir(sim: &SimWorld, m: &Montada) -> String {
    let w = sim.world();
    format!(
        "Red {:?} · Score {:?} · Spinner {:?}",
        w.get::<SortingLayer>(Entity::from_bits(m.red)).map(|l| l.0),
        w.get::<UiLabel>(Entity::from_bits(m.score))
            .map(|l| l.source.clone()),
        w.get::<LuauScript>(Entity::from_bits(m.spinner))
            .map(|c| (c.source.clone(), c.own.get("speed").cloned())),
    )
}
