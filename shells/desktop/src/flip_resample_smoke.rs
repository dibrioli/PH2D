//! **A cena pronta para o smoke da REAMOSTRAGEM SUAVE** (`PH2D_FLIP_RESAMPLE_SMOKE=1`, T2.8).
//!
//! O report do Enio (2026-07-25, com screenshot): *"o traço de qualquer modo tem baixo número de
//! vértices e assim fica tracejado e não arredondado nas curvas. dê mais resolução ao traço."* O
//! RDP e o render ligam os pontos por RETAS, então poucos pontos = curvas facetadas. A reamostragem
//! interpola uma **Catmull-Rom** pelos pontos (o traço passa exato por eles) e a densifica — as
//! curvas ficam arredondadas, as quinas ficam.
//!
//! A cena põe o MESMO "C" esparso (6 pontos) lado a lado:
//!
//! - **ESQUERDA (antes)** — os 6 pontos crus como `FlipStroke` (o render liga por retas): o C fica
//!   FACETADO/tracejado, como no screenshot.
//! - **DIREITA (depois)** — os MESMOS 6 pontos pela porta REAL `stroke_from_samples`: reamostrados
//!   numa curva densa e LISA.
//!
//! **Números MEDIDOS** (o gate `a_sparse_stroke_becomes_a_smooth_curve`, arco esparso de passos ~45°):
//! o giro máximo entre segmentos cai de **~45° (facetado) para < 15° (liso)**, e a contagem de
//! pontos sobe de 6 para dezenas (densificado). A cena imprime as duas contagens.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_RESAMPLE_SMOKE").is_some())
}

/// Um "C" esparso (6 pontos, passos de ~45°, raio 0.7), deslocado em `dx`. As amostras crus que a
/// mão do artista deixaria num arco desenhado rápido.
fn c_samples(dx: f32) -> Vec<Vec2> {
    (0..6)
        .map(|k| {
            let a = (235.0 - 45.0 * k as f32).to_radians();
            Vec2::new(dx + 0.7 * a.cos(), 0.7 * a.sin())
        })
        .collect()
}

/// **Monta a cena** — porta única. Uma camada, um quadro: o C cru (facetado) à esquerda, o C
/// reamostrado (liso) à direita. Devolve `(n_antes, n_depois)` das contagens de pontos.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> (usize, usize) {
    obj.fps = 12.0;
    obj.onion.enabled = false;

    let style = FlipStyleSnapshot::default();
    let ink = Rgba::new(0.75, 0.78, 0.85, 1.0);
    let base_w = ph2d_tool_flip::size_to_world(style.width_px);

    // ANTES: os 6 pontos crus, ligados por retas (facetado).
    let raw = c_samples(-1.4);
    let mut before = FlipStroke::new();
    for &p in &raw {
        before.push_point(Point {
            pos: p,
            width: base_w,
            opacity: 1.0,
            color: ink,
        });
    }
    before.hardness = style.hardness;
    let n_before = before.positions().len();

    // DEPOIS: os MESMOS 6 pontos pela porta real → reamostrados (liso).
    let after = crate::flip_draw::stroke_from_samples(
        &style,
        &c_samples(1.4),
        &[1.0; 6],
        &Xform::IDENTITY,
    );
    let n_after = after.positions().len();

    let layer = obj.add_layer("Resample");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(before);
        strokes.push(after);
    }
    (n_before, n_after)
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_resample_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Resample Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let (n_before, n_after) = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[resample-smoke] cena montada: C esparso {n_before} pontos (ANTES, x=-1.4, facetado) \
             -> {n_after} pontos (DEPOIS, x=1.4, liso). Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[resample-smoke] cena montada'? Se NAO, PARE:\n\
             o smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: o MESMO 'C' esparso (6 pontos), duas vezes:\n\
               - ESQUERDA : os 6 pontos crus ligados por RETAS -- o C fica\n\
                            FACETADO/tracejado, como no seu screenshot.\n\
               - DIREITA  : os MESMOS 6 pontos pela porta real de desenho\n\
                            -- reamostrados numa curva densa e ARREDONDADA.\n\
             \n\
             ------------------------------------------------------------\n\
             Medido: o giro maximo entre segmentos cai de ~45° (facetado)\n\
             para < 15° (liso); a contagem sobe de 6 para dezenas.\n\
             ------------------------------------------------------------\n\
             \n\
             Agora DESENHE (modo Draw) uma curva/circulo devagar e rapido:\n\
             ela deve sair arredondada, sem os vincos/facetas do report. As\n\
             QUINAS deliberadas (um L, um V agudo) continuam agudas. Se algo\n\
             ainda destoar -- muito ou pouco arredondado -- me diga o que.\n"
        );
    }
}
