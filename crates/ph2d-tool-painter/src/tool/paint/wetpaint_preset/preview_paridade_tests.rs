//! ⭐ **O preview é a TELA** (dono 22/10: «o preview dos brushes precisa refletir essas novas features»):
//! para cada capacidade nova de pincel, o último quadro do [`pintar_preview`] tem de ser, AO BYTE, o que a
//! tela pinta com o mesmo pincel armado à mão e o mesmo gesto ([`gesto_do_preview`]). E a capacidade tem de
//! MUDAR o preview (senão a igualdade seria vácua). Um campo que a [`CenaDoPreview`] não leva para o tool
//! do preview sangra aqui; o censo (`preview_censo_tests`) impede um
//! campo NOVO de nascer fora dela.

use super::*;
use crate::tool::paint::composite::{CompositeLayer, CompositeOp};
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_painter_brush::{DualTip, TextureKind};
use ph2d_wet_paint::tuning::Knob;

type Arma = &'static (dyn Fn(&mut PainterTool) + Sync);

const RAIO: f32 = 14.0;
const AZUL: [u8; 3] = [30, 60, 220];

fn verde_vermelho() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// As Grain Colors (`ae082ffb2e`): o Grain xadrez indexa a rampa verde→vermelho.
fn grain_colors(t: &mut PainterTool) {
    t.set_brush_texture_kind(TextureKind::Checker.to_u8());
    t.paint.brush.granulation_use_paper = false;
    t.set_texture_ramp(verde_vermelho());
    t.set_texture_ramp_enabled(true);
}

/// Uma silhueta `32²`: um anel (a imagem da ponta, da 2.ª ponta).
fn anel() -> Vec<u8> {
    (0..32 * 32)
        .map(|i| {
            let (x, y) = ((i % 32) as f32 - 15.5, (i / 32) as f32 - 15.5);
            let r = (x * x + y * y).sqrt();
            if (9.0..15.0).contains(&r) { 255 } else { 0 }
        })
        .collect()
}

fn dual_de_imagem(t: &mut PainterTool) {
    t.paint.brush.dual = Some(DualTip {
        escala: 1.0,
        ..DualTip::default()
    });
    t.set_brush_dual_image(anel(), 32, 32);
}

fn ponta_de_duas_imagens(t: &mut PainterTool) {
    t.add_brush_shape_hose_image(vec![255; 32 * 32], 32, 32);
    t.add_brush_shape_hose_image(anel(), 32, 32);
}

fn composite(t: &mut PainterTool) {
    t.paint.composite_enabled = true;
    t.paint.composite_len = 2;
    t.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        ..CompositeLayer::default()
    };
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Erase,
        strength: 0.6,
        ..CompositeLayer::default()
    };
    t.set_composite_layer_size(1, 0.2);
}

/// O tamanho DEIXA de seguir a pressão (o pincel de fábrica segue-a): só muda o preview se o gesto tiver a
/// rampa de pressão — com a pressão fixa `1` do rato as duas pintavam igual.
fn sem_pressao_no_tamanho(t: &mut PainterTool) {
    t.paint.dynamics.size_pressure = false;
}

/// A Pen «Pressure hardness» (J4): a pressão endurece a queda.
fn pressure_hardness(t: &mut PainterTool) {
    t.paint.brush.hardness = 0.1;
    t.paint.dynamics.pressure_hardness = 0.8;
}

/// Os casos: o meio, o nome, como se arma.
const CASOS: [(PaintMedia, &str, Arma); 13] = [
    (PaintMedia::Digital, "Digital · Grain Colors", &grain_colors),
    (
        PaintMedia::Watercolor,
        "Watercolor · Grain Colors",
        &grain_colors,
    ),
    (PaintMedia::Impasto, "Impasto · Grain Colors", &grain_colors),
    (PaintMedia::WetPaint, "Wet · Grain Colors", &grain_colors),
    (PaintMedia::WetPaint, "Wet · StrokeFilm", &|t| {
        t.paint.wetpaint.knobs.set(Knob::StrokeFilm, 1.0);
    }),
    (PaintMedia::WetPaint, "Wet · bristleStreak", &|t| {
        t.paint.wetpaint.knobs.set(Knob::BristleStreak, 0.8);
    }),
    (PaintMedia::WetPaint, "Wet · PressureGrain", &|t| {
        t.set_brush_texture_kind(TextureKind::Checker.to_u8());
        t.paint.wetpaint.knobs.set(Knob::PressureGrain, 1.0);
    }),
    (
        PaintMedia::Digital,
        "Digital · dual (imagem)",
        &dual_de_imagem,
    ),
    (
        PaintMedia::Digital,
        "Digital · ponta de N imagens",
        &ponta_de_duas_imagens,
    ),
    (PaintMedia::Digital, "Digital · Composite", &composite),
    (
        PaintMedia::Digital,
        "Digital · Pressure hardness",
        &pressure_hardness,
    ),
    (
        PaintMedia::Digital,
        "Digital · sem pressão no tamanho",
        &sem_pressao_no_tamanho,
    ),
    (
        PaintMedia::WetPaint,
        "Wet · sem pressão no tamanho",
        &|t| {
            t.paint.wetpaint.knobs.set(Knob::PressureSize, 0.0);
        },
    ),
];

fn pincel(meio: PaintMedia, t: &mut PainterTool, arma: Arma) {
    t.set_paint_media(meio);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(RAIO);
    t.set_brush_color_srgb8(AZUL);
    arma(t);
}

/// O último quadro do preview do pincel armado num tool qualquer (a tela dele não entra).
fn do_preview(meio: PaintMedia, arma: Arma) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    pincel(meio, &mut t, arma);
    let mut ultimo = Vec::new();
    assert!(pintar_preview(
        &t.wet_preview_cena(),
        &|| false,
        &mut |px, _, _| {
            ultimo = px.to_vec();
        }
    ));
    ultimo
}

/// O último quadro do MESMO gesto sobre a tela do próprio tool armado à mão.
fn da_tela(meio: PaintMedia, arma: Arma) -> Vec<u8> {
    let (lw, lh) = tela_do_preview(RAIO);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (lw * lh * 4) as usize], lw, lh);
    pincel(meio, &mut t, arma);
    let mut ultimo = Vec::new();
    assert!(gesto_do_preview(
        &mut t,
        lw,
        lh,
        &|| false,
        &mut |px, _, _| {
            ultimo = px.to_vec();
        }
    ));
    ultimo
}

fn diferentes(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len())
}

#[test]
fn o_preview_pinta_cada_capacidade_nova_como_a_tela() {
    let sem: Vec<(PaintMedia, Vec<u8>)> = std::thread::scope(|s| {
        let h: Vec<_> = [
            PaintMedia::Digital,
            PaintMedia::Watercolor,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ]
        .into_iter()
        .map(|m| s.spawn(move || (m, do_preview(m, &|_| {}))))
        .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let medidas: Vec<(&str, usize, usize)> = std::thread::scope(|s| {
        let h: Vec<_> = CASOS
            .iter()
            .map(|&(meio, nome, arma)| {
                let base = &sem.iter().find(|(m, _)| *m == meio).unwrap().1;
                s.spawn(move || {
                    let p = do_preview(meio, arma);
                    (
                        nome,
                        diferentes(&p, &da_tela(meio, arma)),
                        diferentes(&p, base),
                    )
                })
            })
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (nome, contra_tela, contra_sem) in &medidas {
        eprintln!(
            "[paridade] {nome:32} bytes ≠ tela {contra_tela:7} · ≠ sem a capacidade {contra_sem:7}"
        );
    }
    let falhas: Vec<_> = medidas
        .iter()
        .filter(|(_, tela, sem)| *tela != 0 || *sem == 0)
        .collect();
    assert!(
        falhas.is_empty(),
        "o preview não pinta como a tela (ou a capacidade não muda o preview): {falhas:?}"
    );
}

/// ⭐ **O pincel GRAVADO leva as Grain Colors, e a miniatura dele mostra-as** (nos 4 meios): a miniatura de um
/// preset do artista é pintada pelo mesmo caminho do preview vivo, sobre a receita do ficheiro — e a receita
/// não as levava (22/10). Aplicar o ficheiro num tool novo devolve-as.
#[test]
fn o_pincel_gravado_leva_as_grain_colors_e_a_miniatura_mostra_as() {
    use crate::tool::paint::wetpaint_preset::{miniatura_do_preset, nome_do_meio};
    use ph2d_brush_presets::{Miniatura, Origem, Parametros, Preset};
    std::thread::scope(|s| {
        for meio in [
            PaintMedia::Digital,
            PaintMedia::Watercolor,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ] {
            s.spawn(move || {
                let mut t = PainterTool::default();
                t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
                pincel(meio, &mut t, &grain_colors);
                let base = Parametros::novo(nome_do_meio(meio), "teste", "grão");
                let (parametros, texturas) = t.preset_capturar(&base);
                let p = Preset {
                    parametros,
                    origem: Origem::Utilizador,
                    miniatura: Miniatura {
                        largura: 1,
                        altura: 1,
                        rgba: vec![0; 4],
                    },
                    texturas,
                };
                assert!(
                    p.parametros.grain_color.is_some(),
                    "{meio:?}: o preset não as grava"
                );
                let mut sem = p.clone();
                sem.parametros.grain_color = None;
                assert_ne!(
                    miniatura_do_preset(&p).rgba,
                    miniatura_do_preset(&sem).rgba,
                    "{meio:?}: a miniatura do pincel gravado não mostra as Grain Colors"
                );
                let mut u = PainterTool::default();
                assert!(u.preset_aplicar(&p).is_ok());
                assert!(
                    u.paint.texture_ramp_enabled,
                    "{meio:?}: aplicar não as liga"
                );
                assert_eq!(
                    u.paint.texture_ramp,
                    verde_vermelho(),
                    "{meio:?}: a rampa não voltou"
                );
                assert!(u.preset_aplicar(&sem).is_ok());
                assert!(
                    !u.paint.texture_ramp_enabled,
                    "{meio:?}: um preset sem elas não as tira"
                );
            });
        }
    });
}

/// **Sonda (ignorada): o custo do preview** — no quadro do ecrã, a CHAVE (comparada a cada quadro) e a CENA
/// (copiada quando a chave muda), com um pincel leve e um pesado (Shape RGBA 512², duas imagens da ponta
/// 512², Grain Colors); na thread, o primeiro quadro e o fim. O mínimo de `N` corridas.
#[test]
#[ignore = "sonda: o custo do preview; corre-se à mão (--profile smoke)"]
fn sonda_custo_do_preview() {
    use std::time::Instant;
    const N: usize = 200;
    let minimo = |f: &mut dyn FnMut()| {
        (0..N)
            .map(|_| {
                let t0 = Instant::now();
                f();
                t0.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min)
    };
    let pesado = |t: &mut PainterTool| {
        let rgba: Vec<u8> = (0..512 * 512 * 4).map(|i| (i % 251) as u8).collect();
        t.set_brush_shape_image_rgba(&rgba, 512, 512, None);
        let cinza: Vec<u8> = (0..512 * 512).map(|i| (i % 253) as u8).collect();
        t.add_brush_shape_hose_image(cinza.clone(), 512, 512);
        t.add_brush_shape_hose_image(cinza, 512, 512);
        grain_colors(t);
    };
    for (nome, meio, arma) in [
        (
            "leve · Digital",
            PaintMedia::Digital,
            &(|_: &mut PainterTool| {}) as &dyn Fn(&mut PainterTool),
        ),
        ("pesado · Digital", PaintMedia::Digital, &pesado),
        (
            "leve · Wet",
            PaintMedia::WetPaint,
            &|_: &mut PainterTool| {},
        ),
        ("Grain Colors · Wet", PaintMedia::WetPaint, &grain_colors),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(meio);
        t.set_brush_size_px(RAIO);
        arma(&mut t);
        let chave = minimo(&mut || {
            std::hint::black_box(t.wet_preview_chave());
        });
        let cena = minimo(&mut || {
            std::hint::black_box(t.wet_preview_cena());
        });
        let c = t.wet_preview_cena();
        let t0 = Instant::now();
        let mut primeiro = None;
        let mut quadros = 0;
        pintar_preview(&c, &|| false, &mut |_, _, _| {
            quadros += 1;
            primeiro.get_or_insert_with(|| t0.elapsed().as_secs_f64() * 1e3);
        });
        let total = t0.elapsed().as_secs_f64() * 1e3;
        eprintln!(
            "[custo] {nome:20} chave {chave:.4} ms · cena {cena:.4} ms · thread: 1.º quadro {:.1} ms, {quadros} quadros em {total:.0} ms",
            primeiro.unwrap_or(f64::NAN)
        );
    }
}
