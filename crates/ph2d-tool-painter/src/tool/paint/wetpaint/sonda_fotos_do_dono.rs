//! SONDAS das duas fotos para o dono conferir (05/11), sem mudar motor: o branco opaco em papel de COR nas duas leis
//! (BUGS #54) e o Knife Stain muito pequeno. `PH2D_SONDA_DIR` recebe um `.ppm` por painel e o `relato.txt`.

use super::agua_sobre_oleo_tests::{H, W, grava, mostrada, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{PanelEvent, PointerPhase};
use ph2d_wet_paint::sim::LeiDaTinta;
use ph2d_wet_paint::tuning::Knob;

const WET_FLAT: &str = "Watercolor/Fluid/Wet Flat.ph2d-brush.png";
const DABBER: &str = "Oil/Dabber/Dabber.ph2d-brush.png";
const MARKER: &str = "Marker/Marker/Marker Bullet.ph2d-brush.png";
const GOUACHE: &str = "Watercolor/Gouache/Gouache Opaque.ph2d-brush.png";
const AZUL: [u8; 3] = [40, 90, 220];
const BRANCO: [u8; 3] = [255, 255, 255];
const LARANJA: [u8; 3] = [240, 130, 60];
const PAPEIS: [[u8; 3]; 3] = [[150, 130, 100], [40, 70, 90], [150, 70, 50]];
const LEIS: [(&str, LeiDaTinta); 2] = [
    ("velha", LeiDaTinta::Velha),
    ("pigmento", LeiDaTinta::Pigmento),
];
/// `(base, pincel, tinta)`: as filas da prancha. A 3 é a cena do #54; as duas últimas, a referência sobre o óleo.
const FILAS: [(&str, &str, [u8; 3]); 6] = [
    (WET_FLAT, GOUACHE, BRANCO),
    (WET_FLAT, MARKER, BRANCO),
    (WET_FLAT, DABBER, BRANCO),
    (WET_FLAT, MARKER, LARANJA),
    (DABBER, MARKER, BRANCO),
    (DABBER, MARKER, LARANJA),
];

fn lum(p: &[u8]) -> f32 {
    0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
}

/// A cor média de uma caixa de 7×7 centrada em `(x, y)`.
fn caixa(img: &[u8], x: usize, y: usize) -> [u32; 3] {
    let mut s = [0u32; 3];
    for j in y - 3..=y + 3 {
        for i in x - 3..=x + 3 {
            for (k, v) in s.iter_mut().enumerate() {
                *v += u32::from(img[(j * W + i) * 4 + k]);
            }
        }
    }
    s.map(|v| v / 49)
}

/// Um painel: base azul horizontal em `y = 90` (molhada, ou óleo assentado), depois o traço diagonal do pincel.
fn painel(fila: usize, p: usize, nome_lei: &str, lei: LeiDaTinta) -> String {
    let (base, pincel, tinta) = FILAS[fila];
    let papel = PAPEIS[p];
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", papel[0], papel[1], papel[2]),
    ));
    t.set_wet_lei_da_tinta(lei);
    // Como o censo do papel de cor: fora do óleo, sem o Real Oil e sem o Emboss (a luz deles clareia por lei).
    let arma = |t: &mut PainterTool, rel: &str| {
        t.preset_aplicar(&preset(rel)).expect("o meio");
        if !rel.starts_with("Oil/") {
            t.set_wet_oleo_real_ligado(false);
            t.paint.wetpaint.knobs.set(Knob::Emboss, 0.0);
        }
        t.sonda_raio(16.0);
    };
    arma(&mut t, base);
    t.set_brush_color_srgb8(AZUL);
    t.on_canvas_pointer(ponteiro(40.0, 90.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(200.0, 90.0, PointerPhase::Up));
    t.ticks_em_blocos(if base == DABBER { 120 } else { 40 });
    arma(&mut t, pincel);
    t.set_brush_color_srgb8(tinta);
    t.on_canvas_pointer(ponteiro(60.0, 130.0, PointerPhase::Down));
    for k in 1..=30 {
        let (x, y) = (60.0 + 4.0 * k as f32, 130.0 - 8.0 / 3.0 * k as f32);
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    let meio = mostrada(&mut t);
    t.on_canvas_pointer(ponteiro(180.0, 50.0, PointerPhase::Up));
    t.ticks_em_blocos(300);
    let fim = mostrada(&mut t);
    grava(&format!("f{fila}_p{p}_{nome_lei}"), &fim);
    let teto = lum(&papel).max(lum(&AZUL)).max(lum(&tinta));
    let claros = |img: &[u8]| img.chunks(4).filter(|px| lum(px) > teto + 30.0).count();
    let no_papel = caixa(&fim, 72, 122);
    let cobre = if tinta == BRANCO {
        (0..3)
            .map(|k| (no_papel[k] as f32 - f32::from(papel[k])) / (255.0 - f32::from(papel[k])))
            .sum::<f32>()
            / 3.0
    } else {
        f32::NAN
    };
    format!(
        "f{fila} {pincel} tinta {tinta:?} sobre {base} papel {papel:?} {nome_lei}: no papel {no_papel:?} (cobre {cobre:.2}) · no cruzamento {:?} · claros meio {} assentado {}",
        caixa(&fim, 120, 90),
        claros(&meio),
        claros(&fim)
    )
}

/// SONDA — a prancha do branco opaco em papel de cor, nas duas leis.
#[test]
#[ignore = "foto para o dono"]
fn sonda_fotos_branco_em_papel_de_cor() {
    let mut casos = Vec::new();
    for f in 0..FILAS.len() {
        for p in 0..PAPEIS.len() {
            for (n, l) in LEIS {
                casos.push((f, p, n, l));
            }
        }
    }
    let fatias: Vec<_> = casos.chunks(casos.len().div_ceil(8)).collect();
    let linhas: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = fatias
            .iter()
            .map(|c| {
                s.spawn(move || {
                    c.iter()
                        .map(|&(f, p, n, l)| painel(f, p, n, l))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        h.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    for l in linhas {
        relato(l);
    }
}

/// SONDA — o Knife Stain nos raios pequenos: `crua` = o raio do MOTOR (a régua da tabela), senão o Size do artista.
#[test]
#[ignore = "foto para o dono"]
fn sonda_fotos_knife_stain_pequeno() {
    for crua in [true, false] {
        for r in [1.5f32, 2.0, 2.5, 2.8, 3.5] {
            let mut t = PainterTool::default();
            t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
            t.set_paint_tool_mode("wetpaint");
            t.set_wet_relogio_fixo(true);
            t.preset_aplicar(&preset("Oil/Knife/Knife Stain.ph2d-brush.png"))
                .expect("o meio");
            if crua {
                t.set_wet_largura_crua(1.0);
            }
            t.sonda_raio(r);
            t.set_brush_color_srgb8(AZUL);
            t.on_canvas_pointer(ponteiro(40.0, 80.0, PointerPhase::Down));
            for k in 1..=60 {
                t.on_canvas_pointer(ponteiro(40.0 + 2.5 * k as f32, 80.0, PointerPhase::Move));
                t.on_tick(1000.0 / 60.0);
            }
            t.on_canvas_pointer(ponteiro(190.0, 80.0, PointerPhase::Up));
            t.ticks_em_blocos(120);
            let img = mostrada(&mut t);
            grava(
                &format!("knife_{}_{r}", if crua { "motor" } else { "size" }),
                &img,
            );
            let dif = |px: &[u8]| px[..3].iter().map(|v| 255 - *v).max().unwrap_or(0);
            let conta = |n: u8| img.chunks(4).filter(|px| dif(px) > n).count();
            let maximo = img.chunks(4).map(dif).max().unwrap_or(0);
            // A largura a meio do traço: as linhas da coluna `x = 115` que a tinta tocou.
            let larg = |n: u8| {
                (0..H)
                    .filter(|&y| dif(&img[(y * W + 115) * 4..(y * W + 115) * 4 + 4]) > n)
                    .count()
            };
            relato(format!(
                "knife {} raio {r}: máximo {maximo} · px>2 {} · px>8 {} · px>32 {} · largura a meio >2 {} · >8 {} · >32 {}",
                if crua { "motor" } else { "size" },
                conta(2),
                conta(8),
                conta(32),
                larg(2),
                larg(8),
                larg(32)
            ));
        }
    }
}
