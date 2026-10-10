//! ⭐ **O GIZMO da luz no canvas** (smoke do dono, 2026-10-09: *«luz sem gizmo»*).
//!
//! O anel do objeto vazio (`empty_object_overlay`) diz ONDE o objeto está e é o que o clique pega;
//! ele não diz que aquilo é uma LUZ, nem de que tamanho, nem até onde chega. Este desenho diz:
//!
//! - **a lâmpada no tamanho real** (`tamanho_m` — o que decide a penumbra), na cor da luz, com um
//!   piso em px para uma lâmpada minúscula não sumir;
//! - **raios** à volta, fora do anel, de tamanho fixo na TELA — lê-se «luz» em qualquer zoom;
//! - **o alcance**, tracejado, só na lâmpada selecionada e só se for finito;
//! - a **Ambiente** não tem lâmpada nem alcance (vem de todo lado): só raios, mais compridos.
//!
//! A geometria é uma função pura ([`glifo`]) com gate; [`desenhar`] só a pinta.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_light2d_ecs::{Luz, TipoDeLuz};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Circle, Color, Fill, Line, Stroke, VectorScene};

/// O raio mínimo da lâmpada na tela.
pub const LAMPADA_MIN_PX: f32 = 3.0; // LITERAL-PX-OK: chrome de overlay, piso de legibilidade
/// A folga entre o anel (ou a lâmpada) e o início de um raio.
const FOLGA_PX: f32 = 4.0; // LITERAL-PX-OK: chrome de overlay
/// O comprimento de um raio (Ponto · Ambiente).
const RAIO_PX: [f32; 2] = [7.0, 11.0]; // LITERAL-PX-OK: chrome de overlay
/// Quantos raios (Ponto · Ambiente).
const RAIOS: [usize; 2] = [8, 12];
/// A espessura do traço do raio e do contorno escuro por baixo (o contraste sobre chão aceso).
const TRACO_PX: [f64; 2] = [2.0, 4.0]; // LITERAL-PX-OK: chrome de overlay
/// O tracejado do alcance.
const TRACEJADO_PX: [f64; 2] = [6.0, 4.0]; // LITERAL-PX-OK: chrome de overlay

/// O desenho de uma luz, em px de TELA.
#[derive(Clone, Debug, PartialEq)]
pub struct Glifo {
    pub centro: [f32; 2],
    /// `0` na Ambiente (não há lâmpada).
    pub lampada_px: f32,
    /// `(início, fim)` de cada raio.
    pub raios: Vec<([f32; 2], [f32; 2])>,
    pub alcance_px: Option<f32>,
    /// A cor da luz, em sRGB (a do seletor).
    pub cor: [u8; 3],
}

/// ⭐ A geometria do gizmo da luz `luz` com centro `centro` na tela, a `px_por_metro` de tela, com
/// o anel do objeto vazio de raio `anel_px`.
#[must_use]
pub fn glifo(
    luz: &Luz,
    centro: [f32; 2],
    px_por_metro: f32,
    anel_px: f32,
    selecionada: bool,
) -> Glifo {
    let ambiente = luz.tipo == TipoDeLuz::Ambiente;
    let k = usize::from(ambiente);
    let lampada_px = if ambiente {
        0.0
    } else {
        (luz.tamanho_m * px_por_metro).max(LAMPADA_MIN_PX)
    };
    let dentro = lampada_px.max(anel_px) + FOLGA_PX;
    let fora = dentro + RAIO_PX[k];
    let raios = (0..RAIOS[k])
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / RAIOS[k] as f32;
            let (s, c) = a.sin_cos();
            (
                [centro[0] + dentro * c, centro[1] + dentro * s],
                [centro[0] + fora * c, centro[1] + fora * s],
            )
        })
        .collect();
    let alcance_px = (selecionada && !ambiente && luz.alcance_m.is_finite())
        .then_some(luz.alcance_m * px_por_metro);
    Glifo {
        centro,
        lampada_px,
        raios,
        alcance_px,
        cor: luz.cor.map(ph2d_color::srgb::linear_to_srgb_byte),
    }
}

/// **Desenha o gizmo de toda `Luz` visível**. `pixels_per_meter` é o do PROJETO (o anel do objeto
/// vazio mede `EMPTY_HALF_PX` px de arte); `selecionada` é o objeto ativo.
pub fn desenhar(
    world: &World,
    selecionada: Option<u64>,
    pixels_per_meter: f32,
    theme: Theme,
    camera: &Camera2d,
    window: ph2d_host::WindowSize,
    cena: &mut VectorScene,
) {
    let Some(mut q) = world.try_query::<(Entity, &Luz)>() else {
        return;
    };
    let contorno = ph2d_editor_core::paint::resolve(ColorToken::Bg0, theme);
    let selecao = ph2d_editor_core::paint::resolve(ColorToken::Selection, theme);
    let mut scratch = Vec::new();
    for (e, luz) in q.iter(world) {
        if ph2d_ecs::is_hidden_in_tree(world, e) {
            continue;
        }
        let Some(t) = ph2d_ecs::world_transform_into(world, e, &mut scratch) else {
            continue;
        };
        let c = [t.translation.x, t.translation.y];
        // ⚠️ A escala é MEDIDA na tela pela câmera, nunca convertida à mão (a lição do anel).
        let (sx, sy) = camera.world_to_screen(c, window);
        let (ex, ey) = camera.world_to_screen([c[0] + 1.0, c[1]], window);
        let ppm_tela = (ex - sx).hypot(ey - sy);
        let anel_px = 2.0 * ph2d_editor_core::HANDLE_SIZE_PX / pixels_per_meter.max(f32::EPSILON)
            * (t.scale.x * t.scale.y).abs().sqrt()
            * ppm_tela;
        if !(ppm_tela.is_finite() && ppm_tela > 0.0) {
            continue;
        }
        let g = glifo(
            luz,
            [sx, sy],
            ppm_tela,
            anel_px,
            selecionada == Some(e.to_bits()),
        );
        pintar(&g, contorno, selecao, cena);
    }
}

fn pintar(g: &Glifo, contorno: Color, selecao: Color, cena: &mut VectorScene) {
    let s = cena.inner_mut();
    let p = |v: [f32; 2]| (f64::from(v[0]), f64::from(v[1]));
    let [r, gr, b] = g.cor;
    let cor = Color::from_rgba8(r, gr, b, u8::MAX);
    if let Some(a) = g.alcance_px.filter(|a| a.is_finite() && *a > 0.0) {
        s.stroke(
            &Stroke::new(1.0).with_dashes(0.0, TRACEJADO_PX),
            Affine::IDENTITY,
            selecao,
            None,
            &Circle::new(p(g.centro), f64::from(a)),
        );
    }
    for (tracos, tinta) in [(TRACO_PX[1], contorno), (TRACO_PX[0], cor)] {
        for (a, b) in &g.raios {
            s.stroke(
                &Stroke::new(tracos),
                Affine::IDENTITY,
                tinta,
                None,
                &Line::new(p(*a), p(*b)),
            );
        }
    }
    if g.lampada_px > 0.0 {
        let disco = Circle::new(p(g.centro), f64::from(g.lampada_px));
        s.fill(Fill::NonZero, Affine::IDENTITY, cor, None, &disco);
        s.stroke(&Stroke::new(1.0), Affine::IDENTITY, contorno, None, &disco);
    }
}
