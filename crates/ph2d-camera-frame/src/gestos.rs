//! GEF 20 C4/C6 — os GESTOS das alças (duplo-clique repõe, Shift encaixa, a ficha no ponteiro) e os limites que
//! nascem no contorno do nível. Irmão do `lib.rs` pelo tecto de LOC.

use super::*;

/// ⭐ GEF 20 C4 — **o duplo-clique numa alça devolve-a à fábrica**: o zoom à altura de omissão, a zona morta a
/// zero, o offset do `follow` a zero, os limites ao `contorno` do nível. `None` = a alça não tem padrão (o miolo
/// sem `follow` move a pose: não há pose de fábrica) ou não há contorno.
#[must_use]
pub fn padrao(m: &Moldura, alca: Alca, contorno: Option<([f32; 2], [f32; 2])>) -> Option<Escrita> {
    match alca {
        Alca::Canto(_) => Some(Escrita::Altura(GameCamera::default().height_world)),
        Alca::ZonaMorta(_) => Some(Escrita::ZonaMorta(CameraFollow::default().dead_zone)),
        Alca::Dentro => m
            .segue
            .as_ref()
            .map(|_| Escrita::FollowOffset(CameraFollow::default().offset)),
        Alca::LimiteCanto(_) | Alca::LimiteBorda(_) | Alca::LimiteCentro => {
            contorno.map(|(min, max)| Escrita::Limites(min, max))
        }
    }
}

/// ⭐ GEF 20 C6 — **o contorno do nível**: a caixa de mundo de toda imagem visível que anda com o mundo (o céu
/// parado e os fundos em parallax, `ScrollFactor ≠ 1`, ficam de fora — andam com a câmera e não dizem onde o
/// nível acaba). `None` = nenhuma imagem.
#[must_use]
pub fn contorno_do_nivel(world: &mut World) -> Option<([f32; 2], [f32; 2])> {
    let mut q = world.query::<(Entity, &ph2d_render::Sprite)>();
    let sprites: Vec<(Entity, [f32; 2])> = q.iter(world).map(|(e, s)| (e, s.size)).collect();
    let mut caixa: Option<([f32; 2], [f32; 2])> = None;
    for (e, tam) in sprites {
        let fundo = world
            .get::<ph2d_ecs::ScrollFactor>(e)
            .is_some_and(|s| s.k != [1.0, 1.0]);
        let escondida = world
            .get::<ph2d_ecs::Visibility>(e)
            .is_some_and(|v| v.hidden);
        let Some(t) = ph2d_ecs::world_transform(world, e).filter(|_| !fundo && !escondida) else {
            continue;
        };
        let (s, c) = t.rotation.sin_cos();
        let (hw, hh) = (
            tam[0] * t.scale.x.abs() * 0.5,
            tam[1] * t.scale.y.abs() * 0.5,
        );
        let meia = [c.abs() * hw + s.abs() * hh, s.abs() * hw + c.abs() * hh];
        let p = [t.translation.x, t.translation.y];
        let (lo, hi) = (
            [p[0] - meia[0], p[1] - meia[1]],
            [p[0] + meia[0], p[1] + meia[1]],
        );
        caixa = Some(match caixa {
            None => (lo, hi),
            Some((a, b)) => (
                [a[0].min(lo[0]), a[1].min(lo[1])],
                [b[0].max(hi[0]), b[1].max(hi[1])],
            ),
        });
    }
    caixa.filter(|(lo, hi)| hi[0] - lo[0] > LIMITE_VAO_MIN && hi[1] - lo[1] > LIMITE_VAO_MIN)
}

/// ⭐ GEF 20 C6 — **a semente dos `CameraLimits`** (a porta de anexar, `component_seed`): a cerca nasce no
/// contorno do nível, e não nos ±10 m de fábrica. Sem imagem nenhuma fica a de fábrica.
pub fn semeia_limites(sim: &mut ph2d_ecs::SimWorld, entity_bits: u64) {
    let Some((min, max)) = contorno_do_nivel(sim.world_mut()) else {
        return;
    };
    if let Some(mut l) = sim
        .world_mut()
        .get_mut::<CameraLimits>(Entity::from_bits(entity_bits))
    {
        *l = CameraLimits { min, max };
    }
}

/// Os passos do Shift — os MESMOS do scrub do Inspector (`ph2d-panel-inspector/src/sections/camera.rs`): a
/// altura e os limites de meio metro, os offsets de 10 cm, a zona morta de 5 %.
pub const PASSO_ALTURA: f32 = 0.5; // LITERAL-PX-OK: metros (o passo do campo Height)
pub const PASSO_LIMITES: f32 = 0.5; // LITERAL-PX-OK: metros (o passo dos campos dos Limits)
pub const PASSO_OFFSET: f32 = 0.1; // LITERAL-PX-OK: metros (o passo dos offsets)
pub const PASSO_ZONA: f32 = 0.05; // LITERAL-PX-OK: fracção (o passo da Dead Zone)

/// ⭐ GEF 20 C4 — **o Shift encaixa** o que o arrasto escreve no passo do campo que ele mexe.
#[must_use]
pub fn encaixa(e: Escrita) -> Escrita {
    let r = |v: f32, p: f32| (v / p).round() * p;
    let r2 = |v: [f32; 2], p: f32| [r(v[0], p), r(v[1], p)];
    match e {
        Escrita::Altura(h) => Escrita::Altura(r(h, PASSO_ALTURA).max(PASSO_ALTURA)),
        Escrita::Pose(p) => Escrita::Pose(r2(p, PASSO_OFFSET)),
        Escrita::FollowOffset(o) => Escrita::FollowOffset(r2(o, PASSO_OFFSET)),
        Escrita::Limites(min, max) => {
            let (min, mut max) = (r2(min, PASSO_LIMITES), r2(max, PASSO_LIMITES));
            for i in 0..2 {
                max[i] = max[i].max(min[i] + PASSO_LIMITES);
            }
            Escrita::Limites(min, max)
        }
        Escrita::ZonaMorta(z) => Escrita::ZonaMorta(r2(z, PASSO_ZONA)),
    }
}

/// ⭐ GEF 20 C4 — **o duplo-clique conta-se aqui** (o canvas não emite `DoubleClick`, como nas alças do Vector):
/// o último press numa alça (instante, câmera, alça).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UltimoPress(Option<(u128, u64, Alca)>);

impl UltimoPress {
    /// O press de agora em `(camera, alca)`: `true` = é o 2.º na MESMA alça dentro da janela do duplo-clique.
    pub fn duplo(&mut self, now_ns: u128, camera: u64, alca: Alca) -> bool {
        let duplo = self.0.is_some_and(|(t, c, a)| {
            c == camera
                && a == alca
                && now_ns.saturating_sub(t) < ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS
        });
        self.0 = (!duplo).then_some((now_ns, camera, alca));
        duplo
    }
}

/// ⭐ GEF 20 C4 — **a etiqueta da alça** (o nome do que ela mexe e o valor), junto do ponteiro sob o hover e a
/// meio do arrasto — o idioma das alças das formas do Vector (`shape_handles_face::readout`).
#[must_use]
pub fn etiqueta(
    m: &Moldura,
    alca: Alca,
    unidade: ph2d_editor_core::project::DisplayUnit,
    ppm: f32,
) -> String {
    let casas = usize::from(unidade == ph2d_editor_core::project::DisplayUnit::Meters);
    let v = |metros: f32| {
        format!(
            "{:.casas$} {}",
            unidade.from_meters(metros, ppm),
            unidade.suffix()
        )
    };
    let par = |p: [f32; 2]| {
        format!(
            "{:.casas$}, {:.casas$} {}",
            unidade.from_meters(p[0], ppm),
            unidade.from_meters(p[1], ppm),
            unidade.suffix()
        )
    };
    let tr = |k: &str, a: &[(&str, &dyn std::fmt::Display)]| ph2d_i18n::tr_with(k, a);
    match alca {
        Alca::Canto(_) => tr(
            "game.camera_frame.handle.zoom",
            &[("V", &v(m.meia[1] * 2.0))],
        ),
        Alca::Dentro => match m.segue.as_ref() {
            Some(s) => tr(
                "game.camera_frame.handle.follow_offset",
                &[("V", &par(s.offset))],
            ),
            None => tr("game.camera_frame.handle.move", &[("V", &par(m.pose))]),
        },
        Alca::ZonaMorta(i) => {
            let z = m.segue.as_ref().map_or(0.0, |s| s.zona[usize::from(i / 2)]);
            tr(
                "game.camera_frame.handle.dead_zone",
                &[("V", &format!("{:.0}", z * 100.0))],
            )
        }
        Alca::LimiteBorda(i) => {
            let (min, max) = m.limites.unwrap_or_default();
            let (k, x) = match i {
                0 => ("game.camera_frame.handle.limit_left", min[0]),
                1 => ("game.camera_frame.handle.limit_right", max[0]),
                2 => ("game.camera_frame.handle.limit_bottom", min[1]),
                _ => ("game.camera_frame.handle.limit_top", max[1]),
            };
            tr(k, &[("V", &v(x))])
        }
        Alca::LimiteCanto(i) => {
            let (min, max) = m.limites.unwrap_or_default();
            tr(
                "game.camera_frame.handle.limit_corner",
                &[("V", &par(cantos(min, max)[usize::from(i)]))],
            )
        }
        Alca::LimiteCentro => tr("game.camera_frame.handle.limit_move", &[]),
    }
}
