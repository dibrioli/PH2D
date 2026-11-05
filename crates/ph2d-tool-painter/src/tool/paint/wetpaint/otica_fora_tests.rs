//! J13b P4, portão 1 ([plano](../../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md) §4): a óptica
//! composta pelo ESPELHO ([`super::otica_fora_porta`]) dá a tela de casa AO BYTE, a cada tique — a cena do J13
//! (Dabber com Real Oil, seca, Wet Drip, corre) numa tela pequena, relógio fixo, com Smooth Edges e o véu ligados e
//! desligados e uma troca de azimute da luz a meio.

use super::agua_sobre_oleo_tests::preset;
use super::otica_fora_porta::{esquece, pela_porta};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const LADO: usize = 256;
const DT_MS: f32 = 1000.0 / 60.0;
/// `(pincel, cor, tiques com o traço, tiques depois)`: 30 + 18 + 30 + 36 = 114 tiques.
const FASES: [(&str, [u8; 3], usize, usize); 2] = [
    ("Oil/Dabber/Dabber.ph2d-brush.png", [40, 90, 220], 30, 18),
    (
        "Watercolor/Fluid/Wet Drip.ph2d-brush.png",
        [230, 200, 30],
        30,
        36,
    ),
];
/// O tique (contado desde o início) em que a luz roda.
const TIQUE_DO_AZIMUTE: usize = 90;

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.7,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t
}

/// Os passos da cena, um a um; `b` corre sempre pela porta.
struct Cena {
    a: PainterTool,
    b: PainterTool,
    smooth: f32,
    veu: bool,
    tique: usize,
    oleo_visto: bool,
}

impl Cena {
    fn ambos(&mut self, f: impl Fn(&mut PainterTool)) {
        f(&mut self.a);
        pela_porta(|| f(&mut self.b));
    }

    fn confere(&mut self, onde: &str) {
        let (a, b) = (self.a.canvas_rgba.as_ref(), self.b.canvas_rgba.as_ref());
        assert_eq!(a.len(), b.len());
        if let Some(i) = a.iter().zip(b).position(|(x, y)| x != y) {
            let px = i / 4;
            panic!(
                "smooth {} véu {} tique {} ({onde}): a tela pelo espelho difere da de casa em ({}, {}) — casa {:?} \
                 espelho {:?}; {} bytes diferentes",
                self.smooth,
                self.veu,
                self.tique,
                px % LADO,
                px / LADO,
                &a[px * 4..px * 4 + 4],
                &b[px * 4..px * 4 + 4],
                a.iter().zip(b).filter(|(x, y)| x != y).count(),
            );
        }
        self.oleo_visto |= self.a.paint.wetpaint.oleo_na_tela;
    }

    fn tique(&mut self) {
        self.ambos(|t| {
            t.on_tick(DT_MS);
            let _ = t.take_preview_arc();
        });
        self.tique += 1;
        self.confere("tique");
        if self.tique == TIQUE_DO_AZIMUTE {
            self.ambos(|t| {
                let mut o = t.wet_oleo_real_valores();
                o.azimute = (o.azimute + 1.9).rem_euclid(std::f64::consts::TAU);
                t.set_wet_oleo_real(Some(o));
            });
            self.confere("azimute");
        }
    }
}

fn corre(smooth: f32, veu: bool) -> (usize, bool) {
    esquece();
    let _ = super::otica_fora_porta::take_caixas();
    let mut c = Cena {
        a: tela(),
        b: tela(),
        smooth,
        veu,
        tique: 0,
        oleo_visto: false,
    };
    let lado = LADO as f32;
    for (fi, &(rel, cor, com_traco, depois)) in FASES.iter().enumerate() {
        let y = lado * (0.35 + 0.3 * fi as f32);
        c.ambos(|t| {
            t.preset_aplicar(&preset(rel)).expect("o preset");
            if fi == 0 {
                t.set_wet_oleo_real_ligado(true);
            }
            t.set_brush_size_px(18.0);
            t.set_brush_color_srgb8(cor);
            // É do traço: o 1.º a 0 faz o tecto do espelho subir depois de ele nascer.
            t.set_wet_smooth_edges(if fi % 2 == 0 { 0.0 } else { smooth });
            t.paint.wetpaint.show_wet = veu;
        });
        // Wet Drip escorre na diagonal: o traço desce um pouco para a água ter onde correr.
        let x = |k: usize| lado * 0.15 + lado * 0.7 * k as f32 / com_traco as f32;
        c.ambos(|t| {
            let _ = t.on_canvas_pointer(ponto(x(0), y, PointerPhase::Down));
        });
        for k in 1..=com_traco {
            c.ambos(|t| {
                for s in 0..4 {
                    let f = (k - 1) as f32 + (s + 1) as f32 / 4.0;
                    let xx = lado * 0.15 + lado * 0.7 * f / com_traco as f32;
                    let _ = t.on_canvas_pointer(ponto(xx, y + 0.08 * f, PointerPhase::Move));
                }
            });
            c.tique();
        }
        c.ambos(|t| {
            let _ = t.on_canvas_pointer(ponto(x(com_traco), y, PointerPhase::Up));
        });
        c.confere("soltar");
        for _ in 0..depois {
            c.tique();
        }
    }
    esquece();
    let caixas = super::otica_fora_porta::take_caixas();
    assert!(caixas >= 96, "a porta compôs só {caixas} caixas");
    assert!(
        c.a.canvas_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| *p != [255; 4]),
        "a cena não pintou nada"
    );
    (c.tique, c.oleo_visto)
}

#[test]
fn a_otica_pelo_espelho_e_a_tela_de_casa_ao_byte() {
    for (smooth, veu) in [(0.0, false), (0.6, true), (0.6, false), (0.0, true)] {
        let (tiques, oleo) = corre(smooth, veu);
        assert!(tiques >= 96, "{tiques} tiques");
        assert!(
            oleo,
            "smooth {smooth} véu {veu}: o Real Oil nunca chegou à tela — a cena não mede o óleo"
        );
    }
}
