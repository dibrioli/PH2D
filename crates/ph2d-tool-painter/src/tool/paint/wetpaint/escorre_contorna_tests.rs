//! SONDA J8 (smoke do dono 01/11: «o escorrimento ignora o relevo da tinta já pousada — passa por cima em vez de
//! desviar»): um traço A (o obstáculo) em `y = 90`, espera, e um Wet Drip diluído B em `y = 55` que escorre para baixo
//! na tela inclinada de fábrica. Mede a massa nova de B no miolo de A, abaixo de A (atravessou) e acima (segurou), com
//! e sem a ladeira do óleo (`LADEIRA_ROTA = 2`). Medição, nunca portão.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, grava, mostrada, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;
use std::sync::atomic::Ordering::Relaxed;

/// O sufixo dos ficheiros gravados (a variante da sonda que corre).
pub(super) static SUFIXO: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

#[derive(Clone, Copy)]
pub(super) enum ObstaculoA {
    Dabber,
    DabberDiluido,
    WetRound,
    /// O Dabber curto (`x` 100–140) debaixo do Wet Drip largo (60–180): a água tem por onde contornar.
    DabberCurto,
}

impl ObstaculoA {
    fn nome(self) -> &'static str {
        match self {
            Self::Dabber => "Dabber",
            Self::DabberDiluido => "Dabber_d05_Oil",
            Self::WetRound => "WetRound_Water",
            Self::DabberCurto => "DabberCurto",
        }
    }
}

pub(super) fn grid(t: &PainterTool) -> &ph2d_wet_paint::grid::Grid {
    &t.paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid
}

impl PainterTool {
    /// `n` quadros de 60 Hz sem ninguém ler a tela entre eles. No relógio fixo, em blocos de 30: um bloco dá os
    /// mesmos 20 passos que 30 tiques e a tela compõe-se uma vez por bloco (prova: `o_tempo_em_blocos_e_o_por_quadro`).
    /// Fora do relógio fixo, um tique por quadro.
    pub(in crate::tool::paint) fn ticks_em_blocos(&mut self, n: impl TryInto<u32>) {
        let n = n.try_into().ok().expect("quadros");
        let bloco = if self.paint.wetpaint.relogio_fixo.is_some() {
            30
        } else {
            1
        };
        for _ in 0..n / bloco {
            #[expect(clippy::cast_precision_loss, reason = "1 ou 30")]
            self.on_tick(bloco as f32 * (1000.0 / 60.0));
        }
        for _ in 0..n % bloco {
            self.on_tick(1000.0 / 60.0);
        }
    }
}

pub(in crate::tool::paint) fn ticks(t: &mut PainterTool, n: u32) {
    t.ticks_em_blocos(n);
}

/// ⭐ Os blocos de [`PainterTool::ticks_em_blocos`] pintam, ao byte, o que pinta um tique por quadro: óleo e aguada, um
/// número que não é múltiplo de 30, e um traço depois da espera (o que sobra da dívida do relógio).
#[test]
fn o_tempo_em_blocos_e_o_por_quadro() {
    let cena = |blocos: bool| {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        let espera = |t: &mut PainterTool, n: u32| {
            if blocos {
                t.ticks_em_blocos(n);
            } else {
                for _ in 0..n {
                    t.on_tick(1000.0 / 60.0);
                }
            }
        };
        t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
            .expect("o meio");
        t.set_brush_size_px(30.0);
        traco(&mut t, 40.0, 200.0, 90.0);
        espera(&mut t, 301);
        t.preset_aplicar(&preset("Watercolor/Fluid/Wet Drip.ph2d-brush.png"))
            .expect("a aguada");
        t.set_brush_size_px(30.0);
        traco(&mut t, 60.0, 180.0, 60.0);
        espera(&mut t, 247);
        traco(&mut t, 60.0, 180.0, 70.0);
        espera(&mut t, 90);
        t.run_full().0
    };
    let (por_quadro, em_blocos) = (cena(false), cena(true));
    assert!(por_quadro == em_blocos, "os blocos mudaram a tela");
}

/// Um traço horizontal de `x0` a `x1` em `y`, 40 passos, um tick por passo.
pub(super) fn traco(t: &mut PainterTool, x0: f32, x1: f32, y: f32) {
    t.on_canvas_pointer(ponteiro(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        let x = x0 + (x1 - x0) * k as f32 / 40.0;
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(x1, y, PointerPhase::Up));
}

/// `vezes` = quantas vezes B passa no mesmo sítio (o volume que empoça contra A).
pub(super) fn linha(a: ObstaculoA, espera: u32, m_b: f64, rota: u8, vezes: u32) -> String {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    // ── A, o obstáculo.
    let rel_a = match a {
        ObstaculoA::Dabber | ObstaculoA::DabberDiluido | ObstaculoA::DabberCurto => {
            "Oil/Dabber/Dabber.ph2d-brush.png"
        }
        ObstaculoA::WetRound => "Watercolor/Fluid/Wet Round.ph2d-brush.png",
    };
    t.preset_aplicar(&preset(rel_a)).expect("o meio de A");
    match a {
        ObstaculoA::Dabber | ObstaculoA::DabberCurto => {}
        ObstaculoA::DabberDiluido => {
            t.paint.wetpaint.knobs.set(Knob::Dilution, 0.5);
            t.paint.wetpaint.knobs.set(Knob::Medium, 1.0);
        }
        ObstaculoA::WetRound => t.paint.wetpaint.knobs.set(Knob::Medium, 0.0),
    }
    let oleo_a = t.wet_oleo_real().is_some();
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    if matches!(a, ObstaculoA::DabberCurto) {
        traco(&mut t, 100.0, 140.0, 90.0);
    } else {
        traco(&mut t, 40.0, 200.0, 90.0);
    }
    ticks(&mut t, espera);
    t.wet_bring_home();

    // ── A fotografia de A no instante em que B começa.
    let (antes, sett_a, campo_oleo, ganho_k, gs, gw, gh) = {
        let g = grid(&t);
        let antes: Vec<f32> = (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect();
        let sett: Vec<f32> = g.sett.clone();
        let ganho: Vec<f64> = (0..g.cells)
            .map(|i| ph2d_wet_paint::grid::oleo_celula::ganho(&g.oleo, i))
            .collect();
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        (
            antes,
            sett,
            !g.oleo.is_empty(),
            ganho,
            g.s,
            s.grid.0,
            s.grid.1,
        )
    };
    let max_a = antes.iter().copied().fold(0f32, f32::max);
    let limiar = 1000f32.min(0.25 * max_a);
    // Orla de cima e de baixo de A por coluna (células com massa > limiar).
    let orla: Vec<Option<(usize, usize)>> = (1..=gw)
        .map(|cx| {
            let top = (1..=gh).find(|&cy| antes[cx + cy * gs] > limiar)?;
            let bot = (1..=gh).rev().find(|&cy| antes[cx + cy * gs] > limiar)?;
            Some((top, bot))
        })
        .collect();
    let miolo_de = |cx: usize, cy: usize, top: usize| cy >= top + 4 && antes[cx + cy * gs] > limiar;
    let (mut ss, mut sm, mut sg, mut n_miolo) = (0f64, 0f64, 0f64, 0usize);
    for (k, o) in orla.iter().enumerate() {
        let Some((top, bot)) = *o else { continue };
        let cx = k + 1;
        for cy in top..=bot {
            if miolo_de(cx, cy, top) {
                let i = cx + cy * gs;
                ss += f64::from(sett_a[i].max(0.0));
                sm += f64::from(antes[i]);
                sg += ganho_k[i];
                n_miolo += 1;
            }
        }
    }
    let assente_a = ss / sm.max(1e-9);
    let ganho_medio = sg / (n_miolo.max(1) as f64);

    // ── B, o escorrimento.
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Drip.ph2d-brush.png"))
        .expect("o Wet Drip");
    t.paint.wetpaint.knobs.set(Knob::Dilution, 0.9);
    t.paint.wetpaint.knobs.set(Knob::Medium, m_b);
    let oleo_b = t.wet_oleo_real().is_some();
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    ph2d_wet_paint::solver::LADEIRA_ROTA.store(rota, Relaxed);
    // `m_b < 0` = o controlo SEM B: o que A faz sozinho nos mesmos 900 ticks (o «novo» que não é de B).
    ph2d_wet_paint::solver::MEDIDA_DO_CHAO.with(|c| c.set(Default::default()));
    if m_b >= 0.0 {
        for _ in 0..vezes.max(1) {
            traco(&mut t, 60.0, 180.0, 55.0);
        }
    }
    // A parede que o motor fotografou no início de B (só existe com o Real Oil na mão e óleo na grade).
    let (foto, foto_h, foto_ass) = {
        let g = grid(&t);
        let foto = g.oleo_antes.len() == g.cells;
        let (mut h, mut fa, mut n) = (0f64, 0f64, 0usize);
        if foto {
            for (k, o) in orla.iter().enumerate() {
                let Some((top, bot)) = *o else { continue };
                for cy in top..=bot {
                    let cx = k + 1;
                    if miolo_de(cx, cy, top) {
                        let i = cx + cy * gs;
                        h += f64::from(g.oleo_antes[i]);
                        fa += f64::from(g.oleo_antes_assente.get(i).copied().unwrap_or(1.0));
                        n += 1;
                    }
                }
            }
        }
        let n = n.max(1) as f64;
        (foto, h / n, fa / n)
    };
    ticks(&mut t, 900);
    let md = ph2d_wet_paint::solver::MEDIDA_DO_CHAO.with(std::cell::Cell::get);
    let nome = format!(
        "{}_e{espera}_B{}_{}{}",
        a.nome(),
        nome_b(m_b),
        if rota == 2 { "semladeira" } else { "ladeira" },
        SUFIXO.lock().expect("o sufixo")
    );
    grava(&nome, &mostrada(&mut t));
    t.wet_bring_home();
    let g = grid(&t);
    let (mut miolo, mut c50, mut abaixo, mut acima) = (0f64, 0usize, 0f64, 0f64);
    // Contornou: a massa nova abaixo da orla de baixo de A, nas colunas FORA de A.
    let fundo = orla.iter().flatten().map(|o| o.1).max().unwrap_or(gh);
    let mut lado = 0f64;
    // A poça: o `film` máximo nas 6 linhas acima da orla de A; o chão: a média de `Grid::chao` no miolo.
    let (mut poca, mut chao_s, mut chao_n, mut chao_max) = (0f32, 0f64, 0usize, 0f32);
    for (k, o) in orla.iter().enumerate() {
        let Some((top, bot)) = *o else { continue };
        let cx = k + 1;
        for cy in top.saturating_sub(6)..top {
            poca = poca.max(g.film[cx + cy * gs]);
        }
        if g.chao.len() == g.cells {
            for cy in top..=bot {
                let v = g.chao[cx + cy * gs];
                chao_s += f64::from(v);
                chao_n += 1;
                chao_max = chao_max.max(v);
            }
        }
    }
    let chao_m = chao_s / chao_n.max(1) as f64;
    for (k, o) in orla.iter().enumerate() {
        if o.is_some() {
            continue;
        }
        for cy in fundo + 1..=gh {
            let i = k + 1 + cy * gs;
            lado += f64::from(g.sett[i] + g.susp[i] - antes[i]).max(0.0);
        }
    }
    for (k, o) in orla.iter().enumerate() {
        let Some((top, bot)) = *o else { continue };
        let cx = k + 1;
        for cy in 1..=gh {
            let i = cx + cy * gs;
            let nova = f64::from(g.sett[i] + g.susp[i] - antes[i]).max(0.0);
            if cy < top {
                acima += nova;
            } else if cy > bot {
                abaixo += nova;
            } else if miolo_de(cx, cy, top) {
                miolo += nova;
                if nova > 50.0 {
                    c50 += 1;
                }
            }
        }
    }
    let nn = md.n.max(1) as f64;
    let chao = format!(
        " | lado {lado:>9.0} | poca {poca:.2} chao {chao_m:.2} max {chao_max:.2} | chao n {} grav {:.5} forca {:.5} decl {:.4} lad {:.5} livre {:.5} clamp {:.3}",
        md.n,
        md.gravidade / nn,
        md.chao / nn,
        md.declive / nn,
        md.ladeira / nn,
        md.ladeira_livre / nn,
        md.clamp_morde as f64 / nn
    );
    format!(
        "{:<15}| esp {:>4} | B {:<5}| {:<11}| oleoA {} oleoB {:<5}| limiar {:>5.0} miolo# {:>4} | assenteA {:.3} | campo {:<5} ganho {:.3} | foto {:<5} h {:.3} ass {:.3} | miolo {:>9.0} c>50 {:>4} | abaixo {:>9.0} | acima {:>9.0}",
        a.nome(),
        espera,
        nome_b(m_b),
        if rota == 2 { "sem ladeira" } else { "ladeira" },
        u8::from(oleo_a),
        oleo_b,
        limiar,
        n_miolo,
        assente_a,
        campo_oleo,
        ganho_medio,
        foto,
        foto_h,
        foto_ass,
        miolo,
        c50,
        abaixo,
        acima
    ) + &chao
}

fn nome_b(m_b: f64) -> &'static str {
    if m_b < 0.0 {
        "nenhum"
    } else if m_b > 0.5 {
        "Oil"
    } else {
        "Water"
    }
}

fn bloco(rota: u8) {
    let _r = ph2d_wet_paint::solver::regua::Regua::poe(&ph2d_wet_paint::solver::LADEIRA_ROTA, rota);
    let mut casos = Vec::new();
    for a in [
        ObstaculoA::Dabber,
        ObstaculoA::DabberDiluido,
        ObstaculoA::WetRound,
    ] {
        for espera in [120u32, 1200] {
            for m_b in [-1.0, 0.0, 1.0] {
                casos.push((a, espera, m_b));
            }
        }
    }
    let linhas: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = casos
            .iter()
            .map(|&(a, e, m)| {
                std::thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn_scoped(s, move || linha(a, e, m, rota, 1))
                    .expect("a thread")
            })
            .collect();
        h.into_iter()
            .zip(&casos)
            .map(|(h, &(a, e, m))| {
                h.join().unwrap_or_else(|_| {
                    format!("{} | esp {e} | B {m} | rota {rota} | REBENTOU", a.nome())
                })
            })
            .collect()
    });
    for l in linhas {
        relato(l);
    }
}

/// SONDA J8 — o escorrimento contorna (ou não) a tinta já pousada; 12 linhas com a ladeira, 12 sem.
#[test]
#[ignore = "sonda J8"]
fn diag_o_escorrer_contorna_o_relevo() {
    bloco(0);
    bloco(2);
}
