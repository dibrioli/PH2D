//! ⭐ **A aguada RE-MOLHA o óleo** (dono 30/10, J10; prancha escolhida `docs/Painter/Pinceis/pranchas/J10_remolha_*.png`,
//! «re-molha médio»): a água do pincel dissolve `Re-wet × Solubility` do assentado solúvel debaixo dela; o
//! dissolvido viaja na água para fora do óleo. Cena = a de `agua_sobre_oleo_tests` (Dabber azul, Wet Flat amarelo).

use super::agua_sobre_oleo_tests::{H, PAPEL, W, grava, mostrada, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

/// O óleo azul (Solubility `soluvel`) e, por cima, a aguada amarela (Re-wet `dissolve`, Drag `arrasto`), a meio.
fn cena(papel: [u8; 3], soluvel: f64, dissolve: f64, arrasto: f64) -> PainterTool {
    cena_com(papel, soluvel, dissolve, arrasto, false, false)
}

/// A [`cena`] com o Real Oil (`oleo`) e o Pigment mixing (`km`) ligados à mão na aguada (smoke do dono 30/10).
fn cena_com(
    papel: [u8; 3],
    soluvel: f64,
    dissolve: f64,
    arrasto: f64,
    oleo: bool,
    km: bool,
) -> PainterTool {
    let mut t = oleo_azul(papel, soluvel);
    aguada(&mut t, dissolve, arrasto, oleo, km);
    t
}

/// O óleo azul do Dabber (Solubility `soluvel`), pousado e com 2 s de repouso.
fn oleo_azul(papel: [u8; 3], soluvel: f64) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", papel[0], papel[1], papel[2]),
    ));
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Solubility, soluvel);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(ponteiro(40.0, 90.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(200.0, 90.0, PointerPhase::Up));
    t.ticks_em_blocos(120);
    t
}

/// A aguada amarela do Wet Flat (Re-wet `dissolve`, Drag `arrasto`, Real Oil `oleo`, Pigment mixing `km`) a meio.
fn aguada(t: &mut PainterTool, dissolve: f64, arrasto: f64, oleo: bool, km: bool) {
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Dissolve, dissolve);
    t.paint.wetpaint.knobs.set(Knob::RewetDrag, arrasto);
    if oleo {
        t.set_wet_oleo_real_ligado(true);
    }
    if km {
        t.paint.wetpaint.km_mixing = true;
    }
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 200, 40]);
    t.on_canvas_pointer(ponteiro(120.0, 30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(120.0, 30.0 + 2.8 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
}

/// Acaba o traço da [`cena`] e deixa assentar.
fn assenta(t: &mut PainterTool) {
    t.on_canvas_pointer(ponteiro(120.0, 142.0, PointerPhase::Up));
    t.ticks_em_blocos(600);
}

/// O assentado AZUL debaixo da aguada (`x 108..132`, as linhas do óleo `78..102`) e o pigmento azul que saiu do óleo
/// para a água (as linhas da aguada acima `40..70` e abaixo `112..140`: massa × a falta de vermelho da cor).
fn medidas(t: &mut PainterTool) -> (f64, f64) {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    let azul = |c: [f32; 3]| (f64::from(c[2]) - f64::from(c[0].max(c[1]))).max(0.0) / 255.0;
    let celula = |x: usize, y: usize| (x + 1) + (y + 1) * g.s;
    let mut oleo = 0.0;
    for y in 78..102 {
        for x in 108..132 {
            let i = celula(x, y);
            oleo += f64::from(g.sett[i]) * azul(g.sett_rgb[i]);
        }
    }
    // Fora do óleo o azul chega MISTURADO com o amarelo (verde, azul-ardósia): conta-se a falta de VERMELHO, que separa
    // o amarelo da aguada (r 220) do azul e do verde (r ≤ ~90) — a dominância do azul lia o verde como nada.
    let sem_vermelho = |c: [f32; 3]| (1.0 - f64::from(c[0]) / 255.0).max(0.0);
    let mut fora = 0.0;
    for y in (40..70).chain(112..140) {
        for x in 108..132 {
            let i = celula(x, y);
            fora += f64::from(g.sett[i]) * sem_vermelho(g.sett_rgb[i])
                + f64::from(g.susp[i]) * sem_vermelho(g.susp_rgb[i]);
        }
    }
    (oleo, fora)
}

/// ⛔ **O neutro é a tinta de sempre, ao bit:** Re-wet `0` (com o óleo solúvel) e Solubility `0` (com a água que
/// re-molha) dão a imagem de hoje.
#[test]
fn sem_re_molhar_a_aguada_sobre_o_oleo_e_a_de_sempre() {
    let hoje = {
        let mut t = cena(PAPEL, 0.0, 0.0, 0.0);
        assenta(&mut t);
        mostrada(&mut t)
    };
    for (soluvel, dissolve, arrasto) in [(1.0, 0.0, 0.0), (0.0, 0.35, 0.5)] {
        let mut t = cena(PAPEL, soluvel, dissolve, arrasto);
        assenta(&mut t);
        assert!(
            mostrada(&mut t) == hoje,
            "Solubility {soluvel} × Re-wet {dissolve} × Drag {arrasto} mudou a imagem"
        );
    }
}

/// ⭐ **Re-wet 0,35 × Solubility 1 dissolve ~35 % do azul debaixo da aguada e leva-o para a água** (a prancha
/// «re-molha médio»): o assentado azul sob a aguada cai 25–45 % e a aguada fora do óleo fica com azul.
/// Verde desde a E10g (`docs/Painter/Arquitetura/E10g_oleo_diluido.md`): o meio da célula mistura-se pelo volume
/// MOLHADO e a aguada sobre o óleo seco é água — o Bingham deixou de a prender (era vermelho desde `c5e6daa0a9`).
/// A régua mede com o K–M desligado na aguada: com ele, o verde que o amarelo faz sobre o azul lê-se como «menos
/// azul» sem nada dissolver (`3eb13d38db`, E10e2 §E3).
#[test]
fn a_aguada_re_molha_o_oleo_soluvel() {
    let corre = |soluvel: f64, dissolve: f64| {
        let mut t = oleo_azul(PAPEL, soluvel);
        t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
            .expect("o meio");
        t.paint.wetpaint.km_mixing = false;
        aguada_sem_preset(&mut t, dissolve);
        assenta(&mut t);
        medidas(&mut t)
    };
    let (oleo0, fora0) = corre(0.0, 0.0);
    let (oleo, fora) = corre(1.0, 0.35);
    let caiu = 1.0 - oleo / oleo0;
    assert!(
        (0.25..=0.45).contains(&caiu),
        "o azul debaixo da aguada caiu {caiu:.3} (alvo 0,35)"
    );
    assert!(
        fora > 1.15 * fora0,
        "o azul dissolvido não saiu do óleo: fora {fora:.0} (hoje {fora0:.0})"
    );
}

/// SONDA — a queda do [`a_aguada_re_molha_o_oleo_soluvel`] com o K–M e o Bingham ligados/desligados (E3,
/// `docs/Painter/Arquitetura/E10e2_km_e_remolha.md` §E3: os dois degraus da queda são estas duas leis).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_queda_por_lei() {
    use ph2d_wet_paint::solver::PAPEL_BINGHAM;
    for bingham in [true, false] {
        let _r = ph2d_wet_paint::solver::regua::Regua::poe(
            &PAPEL_BINGHAM,
            if bingham { 0 } else { (-1.0f32).to_bits() },
        );
        for km in [None, Some(false), Some(true)] {
            let corre = |s: f64, d: f64| {
                let mut t = oleo_azul(PAPEL, s);
                t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
                    .expect("o meio");
                let km0 = t.paint.wetpaint.km_mixing;
                if let Some(k) = km {
                    t.paint.wetpaint.km_mixing = k;
                }
                let km1 = t.paint.wetpaint.km_mixing;
                aguada_sem_preset(&mut t, d);
                assenta(&mut t);
                let (o, f) = medidas(&mut t);
                (o, f, km0, km1)
            };
            let (o0, f0, km0, km1) = corre(0.0, 0.0);
            let (o, f, _, _) = corre(1.0, 0.35);
            relato(format!(
                "bingham {bingham} km {km:?} (preset {km0} → {km1}): oleo {o0:.0} → {o:.0} caiu {:.3} · fora {f0:.0} → {f:.0}",
                1.0 - o / o0
            ));
        }
    }
}

/// A [`aguada`] sem reaplicar o preset (Drag 0), para a sonda mexer no K–M entre o preset e o traço.
fn aguada_sem_preset(t: &mut PainterTool, dissolve: f64) {
    t.paint.wetpaint.knobs.set(Knob::Dissolve, dissolve);
    t.paint.wetpaint.knobs.set(Knob::RewetDrag, 0.0);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 200, 40]);
    t.on_canvas_pointer(ponteiro(120.0, 30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(120.0, 30.0 + 2.8 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
}

/// SONDA — a cena a meio e assentada com Re-wet 0,35 e Drag 0 · 0,15 · 0,3 · 0,5 (Solubility 1; `rw00` = hoje, sem
/// nada), no bege (`p`) e no branco (`b`).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_aguada_re_molha_o_oleo() {
    for (pre, papel) in [("p", PAPEL), ("b", [255; 3])] {
        for (nome, d, a) in [
            ("00", 0.0, 0.0),
            ("15", 0.35, 0.15),
            ("30", 0.35, 0.3),
            ("50", 0.35, 0.5),
        ] {
            let mut t = cena(papel, 1.0, d, a);
            grava(&format!("{pre}_rw{nome}_meio"), &mostrada(&mut t));
            assenta(&mut t);
            grava(&format!("{pre}_rw{nome}_fim"), &mostrada(&mut t));
            let (oleo, fora) = medidas(&mut t);
            relato(format!(
                "{pre} Re-wet {d} Drag {a}: azul sob a aguada {oleo:.0} · azul fora do óleo {fora:.0}"
            ));
        }
    }
}

/// A cor MOSTRADA média de uma caixa `x0..x1 × y0..y1` e quantos píxeis são claros (todos os canais ≥ 190: o
/// branco-rosado do smoke; o amarelo tem b 40, o azul r 40).
fn caixa(px: &[u8], x0: usize, x1: usize, y0: usize, y1: usize) -> ([f64; 3], usize) {
    let (mut s, mut claros, mut n) = ([0.0; 3], 0, 0.0);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = &px[(y * W + x) * 4..][..3];
            for k in 0..3 {
                s[k] += f64::from(p[k]);
            }
            claros += usize::from(p.iter().all(|&v| v >= 190));
            n += 1.0;
        }
    }
    (s.map(|v| v / n), claros)
}

/// ⛔ **O Real Oil na mão não empalidece a aguada sobre o óleo** (smoke do dono 30/10, J11 item 0): a lei do pincel de
/// óleo guardava o tapar extra (o branco) sobre o óleo e a caixa das tintas mostrava-o como papel — o miolo
/// branco-rosado. Assentada, a caixa sobre o óleo fica a ≤ 6 níveis da de sem o Real Oil (medido antes da cura:
/// 87,125,195 → 126,147,205, claros 1 → 11).
/// Verde desde a E10e1 (`docs/Painter/Arquitetura/E10e1_oleo_na_celula.md`): o ramo «óleo sobre óleo» lê a CÉLULA
/// (`OleoCelula::pincel_de_oleo`), não o Real Oil da sessão. A 1.ª cura (`43b535277b`) tirava o ramo de todo e o óleo
/// deixava de cobrir o óleo — revertida.
#[test]
fn o_real_oil_na_mao_nao_empalidece_a_aguada_sobre_o_oleo() {
    let fotografa = |oleo: bool| {
        let mut t = cena_com(PAPEL, 1.0, 0.35, 0.15, oleo, false);
        assenta(&mut t);
        caixa(&mostrada(&mut t), 108, 132, 78, 102)
    };
    let ((sem, claros_sem), (com, claros_com)) = (fotografa(false), fotografa(true));
    // Empalidecer = ficar mais CLARO. Com a água solta (E10g) o Real Oil na aguada cobre o óleo como a opaca (E10e1)
    // e a caixa muda de cor — mais escura, nunca mais clara (medido 74,120,85 contra 76,125,100; o pai com o Bingham
    // desligado dava 75,113,91 contra 66,110,140 e 7 claros).
    let d = com.iter().sum::<f64>() / 3.0 - sem.iter().sum::<f64>() / 3.0;
    assert!(
        d <= 6.0 && claros_com <= claros_sem + 4,
        "o Real Oil empalideceu a aguada: {com:.0?} (claros {claros_com}) contra {sem:.0?} (claros {claros_sem})"
    );
}

/// SONDA (J11, item 0 + 1) — «Real Oil empalidece» e «a água tingida sai verde com Pigment mixing»: a cena do smoke
/// (Solubility 1, Re-wet 0,35, Drag 0,15) com o Real Oil e o Pigment mixing ligados ou não na aguada; a caixa sobre o
/// óleo (`108..132 × 78..102`) e a aguada abaixo dele (`112..140`), a meio e assentada.
#[test]
#[ignore = "diagnóstico"]
fn diag_real_oil_e_pigment_mixing_na_aguada() {
    for (pre, papel) in [("p", PAPEL), ("b", [255; 3])] {
        for (nome, oleo, km) in [
            ("hoje", false, false),
            ("oleo", true, false),
            ("km", false, true),
            ("oleo_km", true, true),
        ] {
            for (fase, assentar) in [("meio", false), ("fim", true)] {
                let mut t = cena_com(papel, 1.0, 0.35, 0.15, oleo, km);
                if assentar {
                    assenta(&mut t);
                }
                let px = mostrada(&mut t);
                grava(&format!("{pre}_{nome}_{fase}"), &px);
                let (c, claros) = caixa(&px, 108, 132, 78, 102);
                let (d, _) = caixa(&px, 108, 132, 112, 140);
                relato(format!(
                    "{pre} {nome:8} {fase}: sobre o óleo {:.0},{:.0},{:.0} claros {claros:3} · abaixo {:.0},{:.0},{:.0}",
                    c[0], c[1], c[2], d[0], d[1], d[2]
                ));
            }
        }
    }
}
