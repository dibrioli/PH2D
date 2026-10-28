//! Cena 11 — **Painel do jogo** (`docs/BlenderConstraints/06_W5_resto.md` §3.3, o HUD que se arruma): um canvas de
//! HUD com a PONTUAÇÃO no canto de cima à esquerda (um texto que lê um contador, que sobe sozinho com o Play), a
//! MOEDA colada à direita dela e o MINIMAPA preso ao canto de cima à direita. As relações (ferramenta «HUD Layout»):
//! a moeda AO LADO da pontuação, a moeda NÃO INVADE o minimapa, a pontuação com a largura do texto. A pontuação
//! cresce e a moeda acompanha; com a janela estreita, quando já não cabe, a pontuação ENCOLHE antes de invadir.
//!
//! Dois quadros: o documento semeia as três formas (o `quadro` tem a cena), e o `step` veste-as quando as entidades
//! nascem (o sync da shell dá-lhes entidade entre os dois).

use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::prelude::Resource;
use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::hud_arranjo::{Forca, Lado, Relacao, UiArranjo};
use ph2d_ecs::{
    ChildOf, Counter, CounterRuntime, Entity, Fit, GameCamera, LabelSource, Name, SignalAction,
    SignalActions, SignalFrom, SignalTarget, SignalVerb, SimWorld, StableId, Timer, TimerRuntime,
    TimerState, Timers, Transform, UiCanvas, UiLabel, VecAnchors, VecPathRef, VecShape,
    VecTextParams,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::TimelineState;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, ellipse, rectangle};
use ph2d_vec_text::TextAlign;
use ph2d_vec_text::glyph::{
    TextLayout, TextPlacement, kerning_by_default, offset_path, path_center, text_to_compound_path,
};

/// A caixa de referência do canvas (`16:9`, centrada).
pub const REF: [f32; 2] = [32.0, 18.0];
/// O tamanho do texto da pontuação, em unidades do canvas.
pub const TEXTO: f64 = 2.5;
/// A borda esquerda da pontuação e a altura da linha de cima.
pub const PONT_X: f32 = -15.0;
pub const LINHA_Y: f32 = 7.0;
/// A folga moeda ↔ pontuação e moeda ↔ minimapa.
pub const FOLGA: f64 = 0.4;
/// O raio da moeda; o minimapa (largura, altura) e a margem dele ao canto.
pub const MOEDA_R: f64 = 0.7;
pub const MAPA: [f64; 2] = [13.0, 5.0];
pub const MARGEM: f32 = 1.0;
/// O contador: quanto soma, de quanto em quanto tempo (µs).
pub const SOMA: &str = "1009";
/// Cada tique é um aviso «Signal» na tela (o motor avisa todo sinal): de 2 em 2 s, como a cena do HUD.
pub const TIQUE_US: u64 = 2_000_000;
/// A altura da vista do jogo: o HUD inteiro cabe na vista do editor.
pub const CAMERA_M: f32 = 6.0;
const PLACAR: &str = "Placar";
const CONTADOR: &str = "Pontos";
const SINAL: &str = "tique";

/// O documento pedido (o `encenar` não tem a cena vetorial).
#[derive(Resource)]
pub struct Semear;

/// As três formas semeadas, à espera das entidades.
#[derive(Resource, Clone)]
pub struct Painel {
    pub pont: VecPathId,
    pub moeda: VecPathId,
    pub mapa: VecPathId,
    /// A largura do «Score: 0» autorado.
    pub largura: f64,
    pub texto: VecTextParams,
}

fn solido(rgb: [u8; 3]) -> Option<Paint> {
    Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)))
}

fn largura(p: &VecPath) -> f64 {
    let xs = p
        .verts_all()
        .flat_map(|v| [v.anchor[0], v.in_handle[0], v.out_handle[0]]);
    let (lo, hi) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| {
        (a.min(x), b.max(x))
    });
    (hi - lo).max(0.0)
}

fn semear(cena: &mut VecScene) -> Option<Painel> {
    let texto = VecTextParams {
        text: "Score: 0".to_owned(),
        origin: [0.0, 0.0],
        family: None,
        size: TEXTO,
        weight: 700.0,
        line_height: 1.25,
        tracking: 0.0,
        align: 0,
        axes: Vec::new(),
        wrap_width: None,
    };
    let layout = TextLayout {
        size: texto.size,
        line_height: texto.line_height,
        tracking: texto.tracking,
        align: TextAlign::Left,
        wrap_width: None,
        kerning: kerning_by_default(),
    };
    let font = ph2d_system_fonts::library::resolve(None);
    let axes = [(ph2d_vector_font::AxisTag::WEIGHT, texto.weight)];
    let fill = solido([245, 240, 225]);
    let mut p = text_to_compound_path(
        &font,
        &texto.text,
        &layout,
        &axes,
        &TextPlacement::At([0.0, 0.0]),
        &fill,
        &None,
    )?;
    let c = path_center(&p);
    offset_path(&mut p, [-c[0], -c[1]]);
    let largura = largura(&p);
    let pont = cena.push_path(p);
    let mut m = ellipse([0.0, 0.0], MOEDA_R, MOEDA_R);
    m.fill = solido([250, 200, 40]);
    let moeda = cena.push_path(m);
    let mut q = rectangle(
        [-MAPA[0] / 2.0, -MAPA[1] / 2.0],
        [MAPA[0] / 2.0, MAPA[1] / 2.0],
    );
    q.fill = solido([40, 70, 90]);
    let mapa = cena.push_path(q);
    Some(Painel {
        pont,
        moeda,
        mapa,
        largura,
        texto,
    })
}

/// Semeia o documento se a cena 11 foi pedida (uma vez).
pub(crate) fn semear_se_pedida(sim: &mut SimWorld, cena: &mut VecScene) {
    if sim.world_mut().remove_resource::<Semear>().is_some()
        && let Some(p) = semear(cena)
    {
        sim.world_mut().insert_resource(p);
    }
}

/// A âncora a um canto da caixa de referência: `f = [0, 1]` = cima-esquerda, `[1, 1]` = cima-direita.
fn canto(f: [f64; 2]) -> VecAnchors {
    let (w, h) = (f64::from(REF[0]) / 2.0, f64::from(REF[1]) / 2.0);
    VecAnchors {
        min: f,
        max: f,
        base: [-w, -h, w, h],
    }
}

/// As relações da cena, com os ids que as entidades receberam.
#[must_use]
pub fn relacoes(
    pont: StableId,
    moeda: StableId,
    mapa: StableId,
) -> Vec<(Relacao<StableId>, Forca)> {
    vec![
        (
            Relacao::AoLado {
                a: moeda,
                b: pont,
                lado: Lado::Direita,
                folga: FOLGA,
            },
            Forca::Requerida,
        ),
        (Relacao::LarguraDoConteudo { a: pont }, Forca::Media),
        (
            Relacao::NaoInvade {
                a: moeda,
                b: mapa,
                lado: Lado::Direita,
                folga: FOLGA,
            },
            Forca::Requerida,
        ),
    ]
}

/// Veste as três formas (já com entidade): devolve o canvas. `None` = ainda não nasceram.
pub fn vestir(sim: &mut SimWorld) -> Option<Entity> {
    let p = sim.world().get_resource::<Painel>()?.clone();
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &VecPathRef)>();
    let mut de = |id: VecPathId, w: &bevy_ecs::world::World| {
        q.iter(w).find(|(_, r)| r.0 == id).map(|(e, _)| e)
    };
    let (pont, moeda, mapa) = (de(p.pont, w)?, de(p.moeda, w)?, de(p.mapa, w)?);
    w.remove_resource::<Painel>();
    let canvas = w
        .spawn((
            Transform::default(),
            UiCanvas {
                ref_w: REF[0],
                ref_h: REF[1],
                fit: Fit::Expand,
            },
            Name::new("HUD"),
        ))
        .id();
    w.spawn((
        Transform::default(),
        GameCamera {
            height_world: CAMERA_M,
            ..GameCamera::default()
        },
        Name::new("Câmera"),
    ));
    #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
    let (meio, r, mapa_xy) = (
        (p.largura / 2.0) as f32,
        MOEDA_R as f32,
        [
            REF[0] / 2.0 - MARGEM - MAPA[0] as f32 / 2.0,
            REF[1] / 2.0 - MARGEM - MAPA[1] as f32 / 2.0,
        ],
    );
    #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
    let moeda_x = PONT_X + 2.0 * meio + FOLGA as f32 + r;
    let at = |x: f32, y: f32| Transform::from_translation(Vec2::new(x, y));
    w.entity_mut(pont).insert((
        ChildOf(canvas),
        at(PONT_X + meio, LINHA_Y),
        VecShape::Text(p.texto.clone()),
        UiLabel {
            source: LabelSource::Counter(CONTADOR.to_owned()),
            prefix: "Score: ".to_owned(),
            suffix: String::new(),
        },
        canto([0.0, 1.0]),
        Name::new("Pontuação"),
    ));
    w.entity_mut(moeda)
        .insert((ChildOf(canvas), at(moeda_x, LINHA_Y), Name::new("Moeda")));
    w.entity_mut(mapa).insert((
        ChildOf(canvas),
        at(mapa_xy[0], mapa_xy[1]),
        canto([1.0, 1.0]),
        Name::new("Minimapa"),
    ));
    w.spawn((
        Transform::default(),
        Counter {
            name: CONTADOR.to_owned(),
            start: 0,
            keep_on_restart: false,
        },
        CounterRuntime { value: 0 },
        SignalActions(vec![SignalAction {
            on: SINAL.to_owned(),
            target: PLACAR.to_owned(),
            verb: SignalVerb::AddToCounter,
            arg: SOMA.to_owned(),
            target_by: SignalTarget::Named,
            from: SignalFrom::Anyone,
        }]),
        Name::new(PLACAR),
    ));
    let relogio = Timer {
        name: SINAL.to_owned(),
        duration_us: TIQUE_US,
        repeat: true,
        autostart: true,
        signal: SINAL.to_owned(),
    };
    let estado: TimerState = ph2d_ecs::timer::born(&relogio);
    w.spawn((
        Transform::default(),
        Timers(vec![relogio]),
        TimerRuntime(vec![estado]),
        Name::new("Relógio"),
    ));
    ph2d_ecs::assign_missing_stable_ids(w);
    let id = |e: Entity| w.get::<StableId>(e).copied();
    let lacos = relacoes(id(pont)?, id(moeda)?, id(mapa)?);
    w.entity_mut(canvas).insert(UiArranjo { lacos });
    Some(canvas)
}

static FEITA: AtomicBool = AtomicBool::new(false);

/// Uma vez, com `PH2D_CONSTRAINT_SMOKE=11` e as formas já com entidade: veste-as, liga o relógio e toca.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if crate::smoke::nivel_armado() != Some(11) || FEITA.load(Ordering::Relaxed) {
        return None;
    }
    let canvas = vestir(sim)?;
    FEITA.store(true, Ordering::Relaxed);
    // O passo fixo (onde os relógios correm) só anda com a simulação armada.
    tl.flags.simulate_physics = true;
    hero.set_panel_visible("timeline", false);
    hero.set_panel_visible("inspector", true);
    hero.gizmo.selection = Some(canvas.to_bits());
    ph.rewind();
    ph.play();
    Some(
        "[constraint-smoke] =11 o HUD a tocar: a pontuação sobe, a moeda acompanha; estreite a janela e ela encolhe \
         antes de invadir o minimapa; a ferramenta «HUD Layout» mostra as setas"
            .to_owned(),
    )
}
