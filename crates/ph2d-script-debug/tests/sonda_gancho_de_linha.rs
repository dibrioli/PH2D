//! Sonda da Onda D (plano 13 §4, `docs/CodeEditor/13_plano_bancada.md`): quanto custa o gancho de
//! linha do Luau numa cena de 50 objectos com script a 60 Hz, sobre o `ScriptHost` real. As
//! variantes trocam-se em execução no MESMO processo, intercaladas em rodadas de ordem rodada; o
//! resumo é o mínimo (`docs/DevOps/MEDIR_VELOCIDADE.md`). Cada variante armada tem um contador de
//! controlo: um gancho que não disparou não mede nada.
//!
//! `bash scripts/cargo-test-narrow.sh ph2d-script-debug --profile smoke -- --ignored --nocapture sonda_gancho_de_linha`
//! A tabela vai também para `target/sonda-gancho-luau.txt`.
#![allow(unsafe_code)]

use std::fmt::Write as _;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::time::Instant;

use mlua::ffi;
use ph2d_ecs::{Entity, SimWorld, StableId, Transform};
use ph2d_script::props::ScriptValue;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost, ScriptId};
use ph2d_script_debug::{breakpoint, set_hooks, single_step};

const GAME: &str = r#"
ph2d.property("speed", 40)
ph2d.property("phase", 1)
local function wobble(t, k)
  return math.sin(t * k) * 0.5
end
function init(self)
  self.t = 0 -- PARADA_NO_INIT
end
function update(self, dt) -- UPDATE_FROM
  self.t = (self.t or 0) + dt
  local x = ph2d.get(self.id, "x")
  local y = ph2d.get(self.id, "y")
  local sum = 0
  for i = 1, 8 do
    sum = sum + wobble(self.t + i, self.phase)
  end
  if x > 400 then
    x = 0
  end
  ph2d.set(self.id, "x", x + self.speed * dt + sum)
  ph2d.set(self.id, "y", y + sum * 0.1)
end -- UPDATE_TO
"#;

const OBJECTS: u64 = 50;
const FRAME_BUDGET_MS: f64 = 1000.0 / 60.0;
const ROUNDS: usize = 9;
const FRAMES_PER_BLOCK: usize = 120;
const DT: f64 = 1.0 / 60.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variant {
    /// Nada armado — a régua.
    Unarmed,
    /// Um ponto de parada numa linha que o quadro não percorre (no `init`): o artista com um F9
    /// posto e o jogo a correr.
    BreakNotHit,
    /// Ponto de parada em CADA linha do `update`, gancho vazio: «valor ao vivo» feito por paragens.
    BreakEveryLine,
    /// Passo-a-passo do Luau, gancho vazio.
    SingleStep,
    /// Passo-a-passo + ao mudar de linha lê todas as variáveis locais (o custo real da D18).
    SingleStepLocals,
}

/// `(nível de depuração do compilador, variante)`: o nível 1 é o de hoje (linhas); o 2 guarda os
/// nomes das variáveis locais, sem os quais `lua_getlocal` não devolve nada.
const VARIANTS: [(u8, Variant); 6] = [
    (1, Variant::Unarmed),
    (2, Variant::Unarmed),
    (2, Variant::BreakNotHit),
    (2, Variant::BreakEveryLine),
    (2, Variant::SingleStep),
    (2, Variant::SingleStepLocals),
];

static BREAKS: AtomicU64 = AtomicU64::new(0);
static STEPS: AtomicU64 = AtomicU64::new(0);
static LINES: AtomicU64 = AtomicU64::new(0);
static LOCALS: AtomicU64 = AtomicU64::new(0);
static LAST_LINE: AtomicI32 = AtomicI32::new(-1);

unsafe extern "C-unwind" fn on_break(_l: *mut ffi::lua_State, _ar: *mut ffi::lua_Debug) {
    BREAKS.fetch_add(1, Ordering::Relaxed);
}

unsafe extern "C-unwind" fn on_step(_l: *mut ffi::lua_State, _ar: *mut ffi::lua_Debug) {
    STEPS.fetch_add(1, Ordering::Relaxed);
}

unsafe extern "C-unwind" fn on_step_locals(l: *mut ffi::lua_State, ar: *mut ffi::lua_Debug) {
    STEPS.fetch_add(1, Ordering::Relaxed);
    // SAFETY: o Luau entrega um `ar` válido durante o gancho.
    let line = unsafe { (*ar).currentline };
    if LAST_LINE.swap(line, Ordering::Relaxed) == line {
        return;
    }
    LINES.fetch_add(1, Ordering::Relaxed);
    let mut n = 1;
    loop {
        // SAFETY: nível 0 = a função em curso; cada `lua_getlocal` bem-sucedido empurra 1 valor,
        // tirado logo a seguir.
        unsafe {
            if ffi::lua_getlocal(l, 0, n).is_null() {
                break;
            }
            std::hint::black_box(ffi::lua_tonumberx(l, -1, std::ptr::null_mut()));
            ffi::lua_settop(l, -2);
        }
        LOCALS.fetch_add(1, Ordering::Relaxed);
        n += 1;
    }
}

fn line_of(marker: &str) -> i32 {
    let i = GAME
        .lines()
        .position(|l| l.contains(marker))
        .unwrap_or_else(|| panic!("marcador {marker}"));
    i32::try_from(i + 1).expect("linha")
}

struct Scene {
    host: ScriptHost,
    sim: SimWorld,
    id: ScriptId,
    actors: Vec<Entity>,
}

impl Scene {
    fn new(debug_level: u8) -> Self {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let id = w
            .get_resource_or_init::<ProjectScripts>()
            .insert("game.luau", GAME.to_owned());
        let actors = (1..=OBJECTS)
            .map(|sid| {
                let mut s = LuauScript::library(id);
                #[expect(clippy::cast_precision_loss, reason = "≤ 50")]
                s.own
                    .insert("phase".to_owned(), ScriptValue::Number(sid as f64 * 0.1));
                w.spawn((Transform::IDENTITY, StableId(sid), s)).id()
            })
            .collect();
        let host = ScriptHost::new().expect("host");
        host.runtime()
            .lua()
            .set_compiler(mlua::chunk::Compiler::new().set_debug_level(debug_level));
        let mut scene = Self {
            host,
            sim,
            id,
            actors,
        };
        for _ in 0..30 {
            scene.frame();
        }
        scene
    }

    fn frame(&mut self) {
        self.host.scene_sync(self.sim.world_mut());
        let r = self.host.scene_tick(&mut self.sim, DT);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
    }

    fn arm(&self, v: Variant) {
        let lua = self.host.runtime().lua();
        let upd = self.host.scene().hook(self.id, "update").expect("update");
        let init = self.host.scene().hook(self.id, "init").expect("init");
        let (from, to, not_hit) = (
            line_of("UPDATE_FROM"),
            line_of("UPDATE_TO"),
            line_of("PARADA_NO_INIT"),
        );
        set_hooks(lua, None, None).expect("ganchos");
        single_step(lua, false).expect("passo");
        for line in from..=to {
            breakpoint(lua, &upd, line, false).expect("parada");
        }
        breakpoint(lua, &init, not_hit, false).expect("parada");
        match v {
            Variant::Unarmed => {}
            Variant::BreakNotHit => {
                set_hooks(lua, Some(on_break), None).expect("ganchos");
                let got = breakpoint(lua, &init, not_hit, true).expect("parada");
                assert_eq!(got, Some(not_hit));
            }
            Variant::BreakEveryLine => {
                set_hooks(lua, Some(on_break), None).expect("ganchos");
                for line in from..=to {
                    breakpoint(lua, &upd, line, true).expect("parada");
                }
            }
            Variant::SingleStep => {
                set_hooks(lua, None, Some(on_step)).expect("ganchos");
                single_step(lua, true).expect("passo");
            }
            Variant::SingleStepLocals => {
                set_hooks(lua, None, Some(on_step_locals)).expect("ganchos");
                single_step(lua, true).expect("passo");
            }
        }
    }

    /// Milissegundos por quadro de um bloco.
    fn block(&mut self) -> f64 {
        let t = Instant::now();
        for _ in 0..FRAMES_PER_BLOCK {
            self.frame();
        }
        #[expect(clippy::cast_precision_loss, reason = "120")]
        let per = t.elapsed().as_secs_f64() * 1000.0 / FRAMES_PER_BLOCK as f64;
        per
    }
}

fn counters() -> [u64; 4] {
    [
        BREAKS.swap(0, Ordering::Relaxed),
        STEPS.swap(0, Ordering::Relaxed),
        LINES.swap(0, Ordering::Relaxed),
        LOCALS.swap(0, Ordering::Relaxed),
    ]
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn min(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

#[test]
#[ignore = "sonda de medição da Onda D — corre-se à mão, perfil smoke"]
fn sonda_gancho_de_linha() {
    let mut scenes = [Scene::new(1), Scene::new(2)];
    let mut times: Vec<Vec<f64>> = vec![Vec::new(); VARIANTS.len()];
    let mut ctl: Vec<[u64; 4]> = vec![[0; 4]; VARIANTS.len()];
    for round in 0..ROUNDS {
        for k in 0..VARIANTS.len() {
            let i = (k + round) % VARIANTS.len();
            let (level, v) = VARIANTS[i];
            let scene = &mut scenes[usize::from(level - 1)];
            scene.arm(v);
            counters();
            LAST_LINE.store(-1, Ordering::Relaxed);
            times[i].push(scene.block());
            scene.arm(Variant::Unarmed);
            for (a, b) in ctl[i].iter_mut().zip(counters()) {
                *a += b;
            }
        }
    }
    for scene in &scenes {
        for e in &scene.actors {
            assert!(
                scene.host.scene().failure(*e).is_none(),
                "um objecto falhou"
            );
        }
    }

    let frames = (ROUNDS * FRAMES_PER_BLOCK) as u64;
    let base = min(&times[0]);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "cena: {OBJECTS} objectos, update a 60 Hz · {ROUNDS} rodadas × {FRAMES_PER_BLOCK} quadros, ordem rodada · resumo = mínimo"
    );
    let _ = writeln!(
        out,
        "| nível | variante | ms/quadro (mín) | mediana | + sobre desarmado | % do quadro 60 Hz | paragens/q | passos/q | linhas/q | locais lidas/q |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|");
    for (i, (level, v)) in VARIANTS.iter().enumerate() {
        let m = min(&times[i]);
        let med = median(times[i].clone());
        let extra = m - base;
        let c = ctl[i];
        let _ = writeln!(
            out,
            "| {level} | {v:?} | {m:.4} | {med:.4} | {extra:+.4} ms | {:+.2} % | {} | {} | {} | {} |",
            extra / FRAME_BUDGET_MS * 100.0,
            c[0] / frames,
            c[1] / frames,
            c[2] / frames,
            c[3] / frames,
        );
    }
    let _ = std::fs::write(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/sonda-gancho-luau.txt"
        ),
        &out,
    );
    println!("{out}");

    // Controlos: o desarmado e a parada não atingida não disparam; os armados disparam.
    assert_eq!(ctl[0], [0; 4], "desarmado disparou");
    assert_eq!(ctl[1], [0; 4], "desarmado (nível 2) disparou");
    assert_eq!(ctl[2][0], 0, "a parada no init foi atingida");
    assert!(
        ctl[3][0] >= frames * OBJECTS,
        "parada em cada linha não disparou"
    );
    assert!(ctl[4][1] >= frames * OBJECTS, "passo-a-passo não disparou");
    assert!(ctl[5][3] >= frames * OBJECTS, "as locais não foram lidas");
}
