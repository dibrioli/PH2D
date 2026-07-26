//! **A cena pronta para o smoke da DINÂMICA DE PRESSÃO** (`PH2D_FLIP_PRESSURE_SMOKE=1`, T2.6).
//!
//! A pressão da caneta vira largura (`ph2d_tool_flip::pressure_width_factor`), com dois controles do
//! artista: **Min Width** (o piso em pressão zero) e **Response** (a curva macia ⇔ dura). No mouse a
//! pressão é sempre 1 (largura cheia); no tablet, a caneta AFINA e ENGROSSA o traço.
//!
//! A cena põe TRÊS traços horizontais, cada um com a pressão subindo 0 → 1 da esquerda para a
//! direita, com dinâmicas diferentes:
//!
//! - **CIMA (default)** — Min Width 5 %, linear: taper forte, fininho na ponta e grosso no fim.
//! - **MEIO (Min Width 60 %)** — piso alto: o traço já nasce grosso e engorda pouco.
//! - **BAIXO (Response dura)** — segura fininho e só engrossa na pressão alta (o fim).
//!
//! **Números MEDIDOS** (o gate `pressure_tapers_the_stroke_width`, pressão 0→1, min 20 %): a ponta
//! (pressão 0) fica no piso e o fim (pressão 1) na espessura CHEIA — 5× mais grosso. No painel Draw
//! (abaixo de Smoothing) há os sliders **Min Width** e **Response**; num mouse a pressão é 1, então
//! o efeito só aparece com tablet OU nesta cena (que injeta a rampa de pressão).

use ph2d_core::Vec2;
use ph2d_flip::{Hold, KeyKind};
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_PRESSURE_SMOKE").is_some())
}

/// Um traço horizontal em `y`, com a pressão subindo 0 → 1 da esquerda para a direita.
fn ramped(y: f32) -> (Vec<Vec2>, Vec<f32>) {
    let n = 9;
    let pts = (0..n)
        .map(|i| Vec2::new(-1.5 + 3.0 * i as f32 / (n - 1) as f32, y))
        .collect();
    let prs = (0..n).map(|i| i as f32 / (n - 1) as f32).collect();
    (pts, prs)
}

/// **Monta a cena** — porta única. Uma camada, um quadro: os 3 traços com pressão rampa e dinâmicas
/// diferentes. Devolve os 3 rótulos das dinâmicas.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> [&'static str; 3] {
    obj.fps = 12.0;
    obj.onion.enabled = false;

    // width_px maior para o taper ser visível; opacity cheia; cor cinza-clara (srgb8).
    let base = FlipStyleSnapshot {
        smoothing: 0.0,
        width_px: 40.0,
        stroke: [200, 205, 220, 255],
        ..Default::default()
    };
    let variants: [(f32, f32, &'static str, f32); 3] = [
        (0.05, 0.5, "default (Min 5%, linear)", 0.8),
        (0.60, 0.5, "Min Width 60%", 0.0),
        (0.05, 0.9, "Response dura", -0.8),
    ];

    let layer = obj.add_layer("Pressure");
    let mut labels = ["", "", ""];
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        for (i, &(min_w, resp, label, y)) in variants.iter().enumerate() {
            let style = FlipStyleSnapshot {
                pressure_min_width: min_w,
                pressure_response: resp,
                ..base
            };
            let (pts, prs) = ramped(y);
            let s = crate::flip_draw::stroke_from_samples(&style, &pts, &prs, &Xform::IDENTITY);
            obj.drawing_mut(d).expect("desenho").strokes.push(s);
            labels[i] = label;
        }
    }
    labels
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_pressure_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Pressure Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let labels = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[pressure-smoke] cena montada: 3 tracos com rampa de pressao 0->1 -- CIMA {}, MEIO {}, \
             BAIXO {}. Ferramenta flip ativa: {}.",
            labels[0],
            labels[1],
            labels[2],
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
             comecando com '[pressure-smoke] cena montada'? Se NAO, PARE:\n\
             o smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: 3 tracos horizontais, cada um com a\n\
             pressao subindo 0 (esquerda) -> 1 (direita):\n\
               - CIMA  : default (Min 5%, linear) -- fininho na ponta,\n\
                         grosso no fim (taper forte).\n\
               - MEIO  : Min Width 60% -- ja nasce grosso, engorda pouco.\n\
               - BAIXO : Response dura -- segura fino e so engrossa no fim.\n\
             \n\
             ------------------------------------------------------------\n\
             Medido: pressao 0 = o piso (Min Width); pressao 1 = espessura\n\
             CHEIA (5x mais grossa com Min 20%). Response 0.5 = linear.\n\
             ------------------------------------------------------------\n\
             \n\
             No painel Draw, abaixo de Smoothing, os sliders 'Min Width' e\n\
             'Response'. Com TABLET, desenhe variando a pressao: o traco\n\
             afina/engrossa. Com mouse a pressao e 1 (largura cheia). Se a\n\
             faixa/curva destoar do que voce espera, me diga.\n"
        );
    }
}
