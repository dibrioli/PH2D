//! ⭐ **A CURVA DO PULO no canvas** (ficha `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`,
//! dono 10/11: «B, mas com linhas coloridas»): o player selecionado com «Show Jump Arc» marcado mostra o
//! pulo inteiro forte e o curto, o parado e os do ar finos e tracejados, cada um na sua cor; o nome
//! aparece com o mouse em cima. A alça do topo é uma [`PointHandleKind::JumpApex`]: cima = Jump Height,
//! lado = Speed. As curvas saem da LEI (`ph2d_platformer::arc`), nunca de uma parábola.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::gizmo::{PointHandle, PointHandleKind};
use ph2d_editor_core::project::ProjectSettings;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::jump_arc::{ArcWorld, JumpArc, JumpArcs};
use ph2d_physics_ecs::{PhysicsSettings, PlatformPlayer, arc_world};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Stroke, VectorScene};

use crate::{CanvasCtx, physics_state::PhysicsState};

/// Distância ao traço, px de ecrã, em que o nome da curva aparece. A meia largura da alça padrão.
const HOVER_PX: f32 = 9.0; // LITERAL-PX-OK: alvo de hover, o mesmo do ponto da âncora
/// Caixa do nome e do readout, px de ecrã.
const LABEL_W_PX: f32 = 140.0; // LITERAL-PX-OK: chrome de overlay
const LABEL_H_PX: f32 = 14.0; // LITERAL-PX-OK: chrome de overlay
const LABEL_FONT_PX: f32 = 11.0; // LITERAL-PX-OK: o mesmo do readout das juntas

/// As quatro curvas, pela ordem de pintura (as finas antes da forte).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Curva {
    Standing,
    Short,
    Air,
    Full,
}

impl Curva {
    #[must_use]
    pub const fn cor(self) -> ColorToken {
        match self {
            Self::Full => ColorToken::ObjPlayer,
            Self::Short => ColorToken::Warn,
            Self::Standing => ColorToken::Info,
            Self::Air => ColorToken::ObjMacro,
        }
    }

    #[must_use]
    pub const fn chave(self) -> &'static str {
        match self {
            Self::Full => "app.physics.jump_arc.full",
            Self::Short => "app.physics.jump_arc.short",
            Self::Standing => "app.physics.jump_arc.standing",
            Self::Air => "app.physics.jump_arc.air",
        }
    }
}

/// Um player com a curva ligada: onde ela começa (o chão sob ele, em mundo) e as curvas.
#[derive(Clone, Debug)]
pub struct Desenho {
    pub player: Entity,
    pub chao: [f32; 2],
    pub mundo: ArcWorld,
    pub arcos: JumpArcs,
    /// A velocidade máxima do player (m/s): o readout da alça.
    pub speed: f32,
}

impl Desenho {
    /// O ápice do pulo inteiro, em mundo — onde a alça mora.
    #[must_use]
    pub fn apice(&self) -> [f32; 2] {
        add(self.chao, self.arcos.full.apex)
    }

    /// Cada curva com o seu tipo, em mundo.
    pub fn curvas(&self) -> impl Iterator<Item = (Curva, Vec<[f32; 2]>)> + '_ {
        let em_mundo = |a: &JumpArc| a.points.iter().map(|p| add(self.chao, *p)).collect();
        [
            (Curva::Standing, &self.arcos.standing),
            (Curva::Short, &self.arcos.short),
        ]
        .into_iter()
        .chain(self.arcos.air.iter().map(|a| (Curva::Air, a)))
        .chain(std::iter::once((Curva::Full, &self.arcos.full)))
        .map(move |(c, a)| (c, em_mundo(a)))
    }
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

/// O desenho do player selecionado, se ele tem a caixa marcada e o relógio está parado (a curva
/// descreve a pose AUTORADA; durante o Play o pulo de verdade está na tela).
#[must_use]
pub fn desenho(
    sim: &SimWorld,
    settings: &PhysicsSettings,
    selection: Option<u64>,
    at_rest: bool,
) -> Option<Desenho> {
    let player = Entity::from_bits(selection?);
    let p = sim.world().get::<PlatformPlayer>(player)?;
    if !at_rest || !p.show_jump_arc {
        return None;
    }
    let t = sim.world().get::<Transform>(player)?;
    let facing = if t.scale.x < 0.0 { -1.0 } else { 1.0 };
    let mundo = arc_world(settings, facing);
    let up = ph2d_physics_ecs::jump_arc::up_of_gravity(mundo.gravity);
    // A curva é a do CENTRO do corpo; desenhada a partir do chão sob ele (a perna em repouso).
    let chao = [
        t.translation.x - up[0] * p.float_height,
        t.translation.y - up[1] * p.float_height,
    ];
    Some(Desenho {
        player,
        chao,
        mundo,
        arcos: p.jump_arcs(mundo),
        speed: p.speed,
    })
}

/// A alça do topo, para a lista de alças de ponto do canvas.
#[must_use]
pub fn handles(d: Option<&Desenho>) -> Vec<PointHandle> {
    d.map(|d| PointHandle {
        key: d.player.to_bits(),
        kind: PointHandleKind::JumpApex,
        world: d.apice(),
    })
    .into_iter()
    .collect()
}

/// O arrasto da alça em curso.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JumpArcDrag {
    pub player_bits: u64,
    /// Ápice − cursor no Down: a alça não salta para debaixo do cursor.
    grab: [f32; 2],
    chao: [f32; 2],
    mundo: ArcWorld,
}

/// Um Down numa alça [`PointHandleKind::JumpApex`] abre o arrasto. `false` para qualquer outra.
pub fn open(
    st: &mut PhysicsState,
    sim: &SimWorld,
    settings: &PhysicsSettings,
    hit: (Entity, PointHandleKind),
    cursor: [f32; 2],
) -> bool {
    let (player, kind) = hit;
    if kind != PointHandleKind::JumpApex || ph2d_ecs::is_locked_for_edit(sim.world(), player) {
        return false;
    }
    let Some(d) = desenho(sim, settings, Some(player.to_bits()), true) else {
        return false;
    };
    let a = d.apice();
    st.jump_arc_drag = Some(JumpArcDrag {
        player_bits: player.to_bits(),
        grab: [a[0] - cursor[0], a[1] - cursor[1]],
        chao: d.chao,
        mundo: d.mundo,
    });
    true
}

/// Segue o cursor: escreve Jump Height e Speed no componente (o undo grava o gesto inteiro no soltar).
pub fn advance(st: &PhysicsState, ctx: &mut CanvasCtx<'_>, pointer: (f32, f32)) {
    let Some(d) = st.jump_arc_drag else {
        return;
    };
    let c = ctx.camera.screen_to_world(pointer, ctx.window);
    let alvo = [c[0] + d.grab[0] - d.chao[0], c[1] + d.grab[1] - d.chao[1]];
    let up = ph2d_physics_ecs::jump_arc::up_of_gravity(d.mundo.gravity);
    let lado = [up[1] * d.mundo.facing, -up[0] * d.mundo.facing];
    let acima = alvo[0] * up[0] + alvo[1] * up[1];
    let frente = alvo[0] * lado[0] + alvo[1] * lado[1];
    let e = Entity::from_bits(d.player_bits);
    if let Some(mut p) = ctx.sim.world_mut().get_mut::<PlatformPlayer>(e) {
        p.pull_apex(frente, acima, d.mundo);
        p.sanitize();
    }
}

fn caminho(pts: &[[f32; 2]], camera: &Camera2d, window: WindowSize) -> BezPath {
    let mut path = BezPath::new();
    for (i, p) in pts.iter().enumerate() {
        let (x, y) = camera.world_to_screen(*p, window);
        let q = (f64::from(x), f64::from(y));
        if i == 0 {
            path.move_to(q)
        } else {
            path.line_to(q)
        }
    }
    path
}

/// Distância, px de ecrã, do ponto ao polígono.
fn distancia(pts: &[(f32, f32)], p: (f32, f32)) -> f32 {
    pts.windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l2 = dx * dx + dy * dy;
            let t = if l2 > 0.0 {
                (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (qx, qy) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
            (qx * qx + qy * qy).sqrt()
        })
        .fold(f32::INFINITY, f32::min)
}

/// A curva sob o cursor, se alguma está a [`HOVER_PX`]. A forte ganha num empate (é pintada por cima).
#[must_use]
pub fn sob_o_cursor(
    d: &Desenho,
    camera: &Camera2d,
    window: WindowSize,
    pointer: (f32, f32),
) -> Option<Curva> {
    d.curvas()
        .filter_map(|(c, pts)| {
            let s: Vec<_> = pts
                .iter()
                .map(|p| camera.world_to_screen(*p, window))
                .collect();
            let dist = distancia(&s, pointer);
            (dist <= HOVER_PX).then_some((c, dist))
        })
        .fold(None, |best: Option<(Curva, f32)>, (c, dist)| match best {
            Some((_, b)) if b < dist => best,
            _ => Some((c, dist)),
        })
        .map(|(c, _)| c)
}

/// Pinta as curvas, o nome da que está sob o cursor e, a arrastar, o readout de altura e velocidade.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    d: Option<&Desenho>,
    dragging: bool,
    project: &ProjectSettings,
    pointer: (f32, f32),
    camera: &Camera2d,
    window: WindowSize,
    theme: Theme,
    scene: &mut VectorScene,
    text: &mut ph2d_text::TextSystem,
) {
    let Some(d) = d else {
        return;
    };
    let fino = f64::from(StrokeToken::Default.px());
    let forte = f64::from(StrokeToken::Heavy.px());
    let traco = [f64::from(Spacing::Sm.px()), f64::from(Spacing::Xs.px())];
    for (c, pts) in d.curvas() {
        let cor = ph2d_editor_core::paint::resolve(c.cor(), theme);
        let stroke = if c == Curva::Full {
            Stroke::new(forte)
        } else {
            Stroke::new(fino).with_dashes(0.0, traco)
        };
        let path = caminho(&pts, camera, window);
        scene
            .inner_mut()
            .stroke(&stroke, Affine::IDENTITY, &Brush::Solid(cor), None, &path);
    }
    let rotulo =
        |texto: &str, at: (f32, f32), cor: ColorToken, scene: &mut VectorScene, text: &mut _| {
            let rect = ph2d_editor_core::zones::Rect::new(
                at.0 + HOVER_PX,
                at.1 - HOVER_PX - LABEL_H_PX,
                LABEL_W_PX,
                LABEL_H_PX,
            );
            ph2d_editor_core::paint::paint_text_centered(
                text,
                scene,
                texto,
                rect,
                LABEL_FONT_PX,
                ph2d_editor_core::paint::resolve(cor, theme),
            );
        };
    if dragging {
        let u = project.display_unit;
        let ppm = project.pixels_per_meter;
        let altura = d
            .arcos
            .full
            .apex
            .iter()
            .zip(up(d))
            .map(|(a, b)| a * b)
            .sum::<f32>();
        let fmt = |m: f32| format!("{:.0} {}", u.from_meters(m, ppm), u.suffix());
        let s = ph2d_i18n::tr_with(
            "app.physics.jump_arc.readout",
            &[("height", &fmt(altura)), ("speed", &fmt(d.speed))],
        );
        rotulo(
            &s,
            camera.world_to_screen(d.apice(), window),
            ColorToken::Text1,
            scene,
            text,
        );
    } else if let Some(c) = sob_o_cursor(d, camera, window, pointer) {
        rotulo(ph2d_i18n::tr(c.chave()), pointer, c.cor(), scene, text);
    }
}

fn up(d: &Desenho) -> [f32; 2] {
    ph2d_physics_ecs::jump_arc::up_of_gravity(d.mundo.gravity)
}

#[cfg(test)]
#[path = "jump_arc_tests.rs"]
mod tests;
