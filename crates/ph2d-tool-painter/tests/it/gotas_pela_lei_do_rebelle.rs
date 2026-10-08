//! **As gotas pela lei do Rebelle** (W1b, doc 06 §5) — a bancada do doc 06 como GATE.
//!
//! Os quatro traços de `ferramentas/gotas.py` (150 px a 40 px de diâmetro, pressão 1, água 1,0 · 0,7 ·
//! 0,4 · 0,2) numa tela branca 1000×640, no relógio fixo, 30 s de água. A barra é a do vale entre o
//! nosso «antes» e o Rebelle (`fixtures/rebelle_gotas/`): tela direita ou a meio ⇒ 0; inclinação máxima
//! ⇒ uma gota ≥ 250 px nos traços de água ≥ 0,7 e 0 nos de 0,4 e 0,2, 1–2 fios por traço, ≤ 4 px de
//! largura; Create Drips off ⇒ 0 com a orla de baixo escura; Size/Length 10/10 ⇒ ≥ 450 px.

use super::sonda_gestos::{H, W, tela};
use ph2d_brush_presets::Volumes;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};

pub const AGUAS: [f64; 4] = [1.0, 0.7, 0.4, 0.2];
const Y: f32 = 150.0;
/// A régua do doc 06: tinta = `255 − min(rgb) > 25`.
const TINTA: u8 = 25;

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn escuro(px: &[u8], x: usize, y: usize) -> u8 {
    let i = (y * W as usize + x) * 4;
    255 - px[i..i + 3].iter().copied().min().unwrap_or(255)
}

/// A inclinação da bancada: `None` = desligada; `Some(anel)` = para baixo, no anel `0..=8`.
pub fn inclina(t: &mut PainterTool, anel: Option<u8>) {
    match anel {
        Some(a) => {
            t.handle_panel_event(PanelEvent::SetValue(ids::PAINTER_WETPAINT_TILT_SPOKE, 3.0));
            t.handle_panel_event(PanelEvent::SetValue(
                ids::PAINTER_WETPAINT_TILT_RING,
                f64::from(a),
            ));
        }
        None => {
            if t.brush_settings().wet_tilt_on {
                t.handle_panel_event(PanelEvent::Click(ids::PAINTER_WETPAINT_TILT_TOGGLE));
            }
        }
    }
    assert_eq!(t.brush_settings().wet_tilt_on, anel.is_some());
}

/// A bancada: a tela, o meio, `prepara` (a inclinação e as gotas), os quatro traços e `segundos` de
/// água a 60 Hz. Devolve `run_full()` (RGBA8, `W`×`H`).
pub fn bancada(segundos: u32, prepara: &dyn Fn(&mut PainterTool)) -> Vec<u8> {
    let mut t = tela();
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    prepara(&mut t);
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([23, 101, 179]);
    for (i, &agua) in AGUAS.iter().enumerate() {
        t.wet_preset_aplicar_volumes(&Volumes {
            water: Some(agua),
            ..Volumes::default()
        });
        let x0 = 60.0 + i as f32 * 240.0;
        t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
        for k in 1..=50 {
            t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(ponto(x0 + 150.0, PointerPhase::Up));
    }
    for _ in 0..segundos * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full().0
}

/// A janela de colunas do traço `c` (a do doc 06: 170 px a partir de `50 + c·240`).
fn colunas(c: usize) -> std::ops::Range<usize> {
    let x0 = 50 + c * 240;
    x0..x0 + 170
}

/// A primeira linha abaixo do traço (a orla de baixo está em `Y + 20`; a régua do doc 06 começa 5 px abaixo).
const BASE: usize = Y as usize + 25;

/// A gota mais longa abaixo do traço `c`, em px (a régua do doc 06).
pub fn gota_mais_longa(px: &[u8], c: usize) -> u32 {
    let mut fundo = BASE;
    for y in BASE..H as usize {
        if colunas(c).any(|x| escuro(px, x, y) > TINTA) {
            fundo = y;
        }
    }
    (fundo - BASE) as u32
}

/// Os fios que cruzam a linha `y`, abaixo do traço `c`: a largura de cada corrida de tinta.
pub fn fios(px: &[u8], c: usize, y: usize) -> Vec<u32> {
    let mut out = Vec::new();
    let mut corre = 0u32;
    for x in colunas(c) {
        if escuro(px, x, y) > TINTA {
            corre += 1;
        } else if corre > 0 {
            out.push(corre);
            corre = 0;
        }
    }
    if corre > 0 {
        out.push(corre);
    }
    out
}

/// A orla de cima e a de baixo do traço `c` — a MESMA régua com que se mediram as fotos do Rebelle
/// (doc 06 §2): o perfil vertical do miolo da largura (longe das pontas), as linhas com tinta, e o escuro
/// médio das 3 primeiras e das 3 últimas.
pub fn orlas(px: &[u8], c: usize) -> (f64, f64) {
    let x0 = 60 + c * 240;
    let perfil: Vec<f64> = (Y as usize - 30..Y as usize + 30)
        .map(|y| {
            let s: u64 = (x0 + 30..x0 + 120)
                .map(|x| u64::from(escuro(px, x, y)))
                .sum();
            s as f64 / 90.0
        })
        .collect();
    let tinta: Vec<usize> = (0..perfil.len())
        .filter(|&k| perfil[k] > f64::from(TINTA))
        .collect();
    let (Some(&t), Some(&b)) = (tinta.first(), tinta.last()) else {
        return (0.0, 0.0);
    };
    (
        perfil[t..t + 3].iter().sum::<f64>() / 3.0,
        perfil[b - 2..=b].iter().sum::<f64>() / 3.0,
    )
}

/// O escuro máximo na linha `y` das colunas do traço `c` (o fio que a cruza).
fn escuro_da_linha(px: &[u8], c: usize, y: usize) -> u8 {
    colunas(c).map(|x| escuro(px, x, y)).max().unwrap_or(0)
}

/// O escuro máximo do próprio traço `c` (±15 px em volta do eixo).
fn escuro_do_traco(px: &[u8], c: usize) -> u8 {
    (Y as usize - 15..Y as usize + 15)
        .flat_map(|y| colunas(c).map(move |x| escuro(px, x, y)))
        .max()
        .unwrap_or(0)
}

/// FNV-1a 64 dos pixels.
pub fn fnv(px: &[u8]) -> u64 {
    px.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn relata(nome: &str, px: &[u8]) -> [u32; 4] {
    if let Ok(out) = std::env::var("PH2D_SONDA_OUT") {
        let mut f = format!("P6\n{W} {H}\n255\n").into_bytes();
        for p in px.as_chunks::<4>().0 {
            f.extend_from_slice(&p[..3]);
        }
        std::fs::write(format!("{out}/gotas_{nome}.ppm"), f).unwrap();
    }
    let g: [u32; 4] = std::array::from_fn(|c| gota_mais_longa(px, c));
    let f: Vec<Vec<u32>> = (0..4).map(|c| fios(px, c, BASE + 60)).collect();
    let o: Vec<(f64, f64)> = (0..4).map(|c| orlas(px, c)).collect();
    println!(
        "[gotas] {nome:<10} gota {g:?}  fios a +60 {f:?}  orlas (cima, baixo) {o:.1?}  fnv {:#018x}",
        fnv(px)
    );
    g
}

/// ⛔ O CONTROLO: inclinação desligada (e as gotas no neutro) ⇒ o traço de antes da W1b, AO BYTE.
#[test]
fn com_a_tela_direita_o_traco_e_o_de_antes_ao_byte() {
    let px = bancada(30, &|t| inclina(t, None));
    let g = relata("direita", &px);
    assert_eq!(g, [0; 4], "tela direita: nada corre");
    assert_eq!(
        fnv(&px),
        0xafa3_56df_56d7_b7fa,
        "o traço de antes da W1b, ao byte"
    );
}

/// Inclinação a meio (o Rebelle `(0, 0,5)`): nenhuma gota — só a orla de baixo escurece.
#[test]
fn a_meia_inclinacao_nada_pinga() {
    let px = bancada(30, &|t| inclina(t, Some(4)));
    let g = relata("meio", &px);
    assert_eq!(g, [0; 4], "a meia inclinação nada pinga");
}

/// Inclinação máxima (o Rebelle `(0, 1)`): uma gota longa e fina nos traços molhados, nenhuma nos secos.
#[test]
fn a_inclinacao_maxima_solta_um_fio_longo_e_fino_nos_tracos_molhados() {
    let px = bancada(30, &|t| inclina(t, Some(8)));
    let g = relata("maximo", &px);
    for c in 0..2 {
        assert!(
            g[c] >= 250,
            "água {}: gota de {} px (< 250)",
            AGUAS[c],
            g[c]
        );
        let f = fios(&px, c, BASE + 60);
        assert!(
            (1..=2).contains(&f.len()),
            "água {}: {} fios (1–2)",
            AGUAS[c],
            f.len()
        );
        assert!(
            f.iter().all(|&w| w <= 4),
            "água {}: fios {f:?} (≤ 4 px)",
            AGUAS[c]
        );
    }
    assert_eq!(&g[2..], &[0, 0], "água 0,4 e 0,2 não pingam");
    // O fio é PÁLIDO e não clareia a meio (Rebelle: fio claro e uniforme até à ponta; o funil 10/10 dele,
    // largo o bastante para a foto o medir, tem 40–52 % do escuro do traço — a barra 25–65 % é o vale):
    // junto ao traço e a 80 % do comprimento.
    for c in 0..2 {
        let (traco, perto, longe) = (
            escuro_do_traco(&px, c),
            escuro_da_linha(&px, c, BASE + 20),
            escuro_da_linha(&px, c, BASE + (g[c] as usize * 8) / 10),
        );
        let r = f64::from(perto) / f64::from(traco);
        assert!(
            (0.25..=0.65).contains(&r),
            "água {}: o fio tem {r:.2} do escuro do traço ({perto} / {traco}; Rebelle 0,40–0,52)",
            AGUAS[c]
        );
        assert!(
            f64::from(longe) >= 0.6 * f64::from(perto),
            "água {}: o fio clareia ({perto} junto ao traço → {longe} a 80 %)",
            AGUAS[c]
        );
    }
}

fn papel(t: &mut PainterTool, create: bool, size: u8, length: u8) {
    t.set_gotas(ph2d_wet_paint::drips::DripPaper {
        create,
        size,
        length,
        easy: false,
    });
}

/// Create Drips desligado: nenhum fio, mas o pigmento desce para a orla de baixo (Rebelle: baixo/cima
/// 2,3–3,1 nos dois traços molhados; tela direita ~1 — a barra 1,5 fica no vale).
#[test]
fn sem_create_drips_a_orla_de_baixo_escurece_e_nada_pinga() {
    let px = bancada(30, &|t| {
        papel(t, false, 5, 5);
        inclina(t, Some(8));
    });
    let g = relata("sem_gotas", &px);
    assert_eq!(g, [0; 4], "Create Drips off: nenhum fio");
    for (c, agua) in AGUAS.iter().enumerate().take(2) {
        let (cima, baixo) = orlas(&px, c);
        assert!(
            baixo >= 1.5 * cima,
            "água {agua}: orla de baixo {baixo:.1} contra cima {cima:.1} (≥ 1,5×)"
        );
    }
}

/// Drip Size e Length a 10: um funil até ao fundo (Rebelle 463–465 px).
#[test]
fn com_size_e_length_a_10_o_funil_chega_ao_fundo() {
    let px = bancada(30, &|t| {
        papel(t, true, 10, 10);
        inclina(t, Some(8));
    });
    let g = relata("gotas_max", &px);
    // O funil não ESCURECE ao estreitar (o do Rebelle clareia: 106 → 26 e 70 → 54 na foto).
    for (c, agua) in AGUAS.iter().enumerate().take(2) {
        let (topo, baixo) = (
            escuro_da_linha(&px, c, BASE + 20),
            escuro_da_linha(&px, c, BASE + 230),
        );
        assert!(
            f64::from(baixo) <= 1.2 * f64::from(topo),
            "água {agua}: o funil escurece ao descer ({topo} → {baixo})"
        );
    }
    for c in 0..2 {
        assert!(g[c] >= 450, "água {}: {} px (≥ 450)", AGUAS[c], g[c]);
    }
}

/// Um traço do pincel de FÁBRICA `nome` (Size px 40), na tela inclinada `anel`, com as gotas `easy` ou
/// não: a gota mais longa abaixo dele, depois de 30 s.
fn traco_de_fabrica(nome: &str, anel: u8, easy: bool) -> u32 {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let e = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o pincel de fábrica");
    let mut t = tela();
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    inclina(&mut t, Some(anel));
    t.set_gotas(ph2d_wet_paint::drips::DripPaper {
        easy,
        ..ph2d_wet_paint::drips::DripPaper::DEFAULT
    });
    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(40.0),
        ..Volumes::default()
    });
    t.set_brush_color_srgb8([23, 101, 179]);
    t.on_canvas_pointer(ponto(60.0, PointerPhase::Down));
    for k in 1..=50 {
        t.on_canvas_pointer(ponto(60.0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(210.0, PointerPhase::Up));
    for _ in 0..30 * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    let px = t.run_full().0;
    gota_mais_longa(&px, 0)
}

/// **As duas versões do papel** (pedido do dono 08/10): com «Easy Drips» o Wet Round de fábrica pinga na
/// tela no máximo como o Round do Rebelle (302 px); sem a caixa, não (a lei da bancada). A meio, nada
/// pinga nas duas; o Dry Brush nunca. Mutação que sangra: o `hold` do motor a ignorar `easy`.
#[test]
fn com_easy_drips_o_wet_round_de_fabrica_pinga_como_o_round_do_rebelle() {
    let sem = traco_de_fabrica("Wet Round", 8, false);
    let com = traco_de_fabrica("Wet Round", 8, true);
    let meio = traco_de_fabrica("Wet Round", 4, true);
    let seco = traco_de_fabrica("Wet Dry Brush", 8, true);
    println!("[gotas] Wet Round: sem a caixa {sem} · com {com} · a meio {meio} · Dry Brush {seco}");
    assert_eq!(sem, 0, "sem Easy Drips o Wet Round não pinga");
    assert!(
        com >= 250,
        "com Easy Drips o Wet Round pinga {com} px (Rebelle 302)"
    );
    assert_eq!(meio, 0, "a meia inclinação nada pinga, nem com Easy Drips");
    assert_eq!(seco, 0, "o Dry Brush não tem água para pingar");
}
