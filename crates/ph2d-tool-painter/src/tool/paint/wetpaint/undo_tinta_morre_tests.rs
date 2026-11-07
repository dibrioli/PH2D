//! **«Porque a tinta morre com UNDO?»** (dono 06/11): quando o desfazer acaba a sessão e a tinta que ficou vira morta.

use super::agua_sobre_oleo_tests::{PAPEL, ponteiro, preset};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};

/// O tecto do undo vivo, à mão: os limites sem pintar gigabytes.
pub(super) mod teto {
    use std::cell::Cell;
    thread_local! { static V: Cell<Option<usize>> = const { Cell::new(None) }; }
    pub(in crate::tool::paint) fn get() -> Option<usize> {
        V.with(Cell::get)
    }
    pub(super) fn set(v: Option<usize>) {
        V.with(|c| c.set(v));
    }
}

const DABBER: &str = "Oil/Dabber/Dabber.ph2d-brush.png";
const AGUADA: &str = "Watercolor/Fluid/Wet Flat.ph2d-brush.png";

fn tela(w: usize, h: usize, meio: &str, tamanho: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset(meio)).expect("o meio");
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([40, 90, 220]);
    t
}

/// Um traço horizontal de `x0` a `x1` na linha `y`, em 40 movimentos.
fn traco(t: &mut PainterTool, x0: f32, x1: f32, y: f32) {
    t.on_canvas_pointer(ponteiro(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(
            x0 + (x1 - x0) * k as f32 / 40.0,
            y,
            PointerPhase::Move,
        ));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(x1, y, PointerPhase::Up));
    t.ticks_em_blocos(2);
}

/// A tinta que a grade ainda tem na linha `y`: 0 = aquela tinta é morta (ou a sessão acabou).
fn tinta_viva_na_linha(t: &mut PainterTool, y: usize) -> f32 {
    t.wet_bring_home();
    let Some(s) = t.paint.wetpaint.session.as_ref() else {
        return 0.0;
    };
    let g = &s.engine.layers[0].grid;
    let o = (y / usize::from(s.ratio) + 1) * g.s;
    (o..o + g.s).map(|c| g.sett[c] + g.susp[c]).sum()
}

fn estado(t: &mut PainterTool, y: usize) -> String {
    let sessao = t.paint.wetpaint.session.is_some();
    let tinta = tinta_viva_na_linha(t, y);
    format!(
        "{}{}",
        if sessao { "S" } else { "-" },
        if tinta > 0.0 { "V" } else { "m" }
    )
}

/// S = a sessão existe · V = a tinta do 1.º traço ainda é viva · m = morta. `-m` depois de desfazer um traço que não o
/// 1.º é o defeito do dono.
#[test]
#[ignore = "sonda: corre por nome com --ignored --nocapture"]
fn sonda_quando_o_undo_mata() {
    for meio in [DABBER, AGUADA] {
        for n in [1usize, 2, 6, 7, 8, 12] {
            let mut t = tela(600, 400, meio, 20.0);
            for i in 0..n {
                traco(&mut t, 40.0, 560.0, 20.0 + 30.0 * i as f32);
            }
            let mut linha = format!("{meio:.6} {n:>2} traços: {}", estado(&mut t, 20));
            for _ in 0..n {
                assert!(t.undo_last());
                linha += &format!(" {}", estado(&mut t, 20));
            }
            linha += " | refaz:";
            for _ in 0..n {
                assert!(t.redo_last());
                linha += &format!(" {}", estado(&mut t, 20));
            }
            eprintln!("{linha}");
        }
    }
    // Com o worker e o compositor, desfazendo logo depois de soltar: a mão real.
    for espera in [0usize, 3] {
        let mut t = tela(600, 400, AGUADA, 20.0);
        t.set_wet_relogio_fixo(false);
        t.set_compor_por_quadro(true);
        let mut linha = format!("app, espera {espera}:");
        for i in 0..3 {
            traco(&mut t, 40.0, 560.0, 20.0 + 30.0 * i as f32);
            for _ in 0..espera {
                std::thread::sleep(std::time::Duration::from_millis(16));
                t.on_tick(1000.0 / 60.0);
            }
        }
        for _ in 0..3 {
            assert!(t.undo_last());
            linha += &format!(" {}", estado(&mut t, 20));
        }
        eprintln!("{linha}");
    }
    // Com a Borracha e com outro modo no meio.
    let mut t = tela(600, 400, DABBER, 20.0);
    traco(&mut t, 40.0, 560.0, 20.0);
    traco(&mut t, 40.0, 560.0, 60.0);
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_paint_tool_mode("digital");
    traco(&mut t, 40.0, 560.0, 100.0);
    t.set_paint_tool_mode("wetpaint");
    t.ticks_em_blocos(2);
    let mut linha = format!("outro modo: {}", estado(&mut t, 20));
    for _ in 0..3 {
        assert!(t.undo_last());
        linha += &format!(" {}", estado(&mut t, 20));
    }
    eprintln!("{linha}");
}

/// O que o motor tem de tinta, células por plano, para comparar duas linhas do tempo.
fn grade(t: &mut PainterTool) -> Vec<Vec<u32>> {
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
    [
        &g.sett,
        &g.susp,
        &g.film,
        &g.altura,
        &g.oleo_antes,
        &g.flow_x,
        &g.flow_y,
    ]
    .iter()
    .map(|p| p.iter().map(|v| v.to_bits()).collect())
    .collect()
}

fn linha_y(i: usize) -> f32 {
    20.0 + 30.0 * i as f32
}

/// N traços e desfazer quase todos dá a linha do tempo de só ter pintado os que ficaram.
#[test]
fn desfazer_fundo_nao_mata_a_tinta_que_ficou() {
    for meio in [DABBER, AGUADA] {
        let mut sem = tela(600, 400, meio, 20.0);
        for i in 0..3 {
            traco(&mut sem, 40.0, 560.0, linha_y(i));
        }
        let mut com = tela(600, 400, meio, 20.0);
        for i in 0..12 {
            traco(&mut com, 40.0, 560.0, linha_y(i));
        }
        let cheia = com.canvas_rgba.to_vec();
        for k in 0..9 {
            assert!(com.undo_last());
            assert!(
                com.paint.wetpaint.session.is_some(),
                "{meio}: o desfazer {} acabou a sessão",
                k + 1
            );
        }
        assert!(com.canvas_rgba[..] == sem.canvas_rgba[..], "{meio}: a tela");
        assert!(grade(&mut com) == grade(&mut sem), "{meio}: a grade");
        // Refazer tudo volta à tinta cheia, viva.
        for _ in 0..9 {
            assert!(com.redo_last());
        }
        assert!(com.canvas_rgba[..] == cheia[..], "{meio}: a tela refeita");
        assert!(tinta_viva_na_linha(&mut com, linha_y(11) as usize) > 0.0);
    }
}

/// O 1.º traço da sessão também tem o seu passo: desfeito e refeito, a tinta continua viva.
#[test]
fn o_primeiro_traco_desfeito_e_refeito_fica_vivo() {
    for meio in [DABBER, AGUADA] {
        let mut sem = tela(600, 400, meio, 20.0);
        traco(&mut sem, 40.0, 560.0, 60.0);
        let antes = {
            let t = tela(600, 400, meio, 20.0);
            t.canvas_rgba.to_vec()
        };
        let mut com = tela(600, 400, meio, 20.0);
        traco(&mut com, 40.0, 560.0, 60.0);
        assert!(com.undo_last());
        assert!(com.paint.wetpaint.session.is_some(), "{meio}: o desfazer");
        assert_eq!(
            tinta_viva_na_linha(&mut com, 60),
            0.0,
            "{meio}: tinta invisível"
        );
        assert!(com.canvas_rgba[..] == antes[..], "{meio}: a tela");
        assert!(com.redo_last());
        assert!(com.paint.wetpaint.session.is_some(), "{meio}: o refazer");
        assert!(
            com.canvas_rgba[..] == sem.canvas_rgba[..],
            "{meio}: a tela refeita"
        );
        assert!(
            grade(&mut com) == grade(&mut sem),
            "{meio}: a grade refeita"
        );
    }
}

/// O tecto corta os passos mais velhos: registo e fotografia juntos, e a conta fica dentro.
#[test]
fn o_tecto_conta_os_registos_e_as_fotos() {
    let mut t = tela(600, 400, DABBER, 20.0);
    traco(&mut t, 40.0, 560.0, 20.0);
    t.wet_bring_home();
    let um = {
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        s.engine.bytes_do_historico(0) + super::undo_vivo::bytes_das_fotos(&s.antes_do_passo)
    };
    let teto_de_tres = 4 * um;
    teto::set(Some(teto_de_tres));
    for i in 1..12 {
        traco(&mut t, 40.0, 560.0, linha_y(i));
    }
    // O passo aberto ainda escreve depois de contado: o tecto vale até ao pen-down seguinte.
    t.on_canvas_pointer(ponteiro(40.0, 390.0, PointerPhase::Down));
    t.on_canvas_pointer(ponteiro(40.0, 390.0, PointerPhase::Up));
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let passos = s.antes_do_passo.len();
    let usado =
        s.engine.bytes_do_historico(0) + super::undo_vivo::bytes_das_fotos(&s.antes_do_passo);
    teto::set(None);
    assert!(passos < 12, "o tecto não cortou: {passos} passos");
    assert!(
        usado <= teto_de_tres + um,
        "{usado} > {teto_de_tres} + {um}"
    );
    assert!(
        s.engine.can_undo(),
        "os registos têm de acompanhar os passos"
    );
}

/// ⭐ Para lá do tecto: só o que se desfez acaba. A tinta dos traços que ficaram continua viva, a tela é a de antes do
/// traço, e nada reaparece nas células do traço desfeito (a tinta invisível), nem depois da água correr.
#[test]
fn para_la_do_tecto_so_o_traco_desfeito_acaba() {
    for meio in [DABBER, AGUADA] {
        teto::set(Some(1));
        let mut t = tela(600, 400, meio, 20.0);
        let mut telas = vec![t.canvas_rgba.to_vec()];
        for i in 0..6 {
            traco(&mut t, 40.0, 560.0, 20.0 + 60.0 * i as f32);
            telas.push(t.canvas_rgba.to_vec());
        }
        for i in (1..6).rev() {
            assert!(t.undo_last());
            assert!(t.paint.wetpaint.session.is_some(), "{meio}: desfazer {i}");
            assert!(t.canvas_rgba[..] == telas[i][..], "{meio}: a tela {i}");
            assert_eq!(
                tinta_viva_na_linha(&mut t, (20 + 60 * i) as usize),
                0.0,
                "{meio}: tinta invisível no traço {i}"
            );
            assert!(
                tinta_viva_na_linha(&mut t, 20) > 0.0,
                "{meio}: o 1.º traço morreu ao desfazer o {i}"
            );
        }
        teto::set(None);
        t.ticks_em_blocos(600);
        let d = t.canvas_rgba[600 * 60 * 4..]
            .iter()
            .zip(&telas[1][600 * 60 * 4..])
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(d, 0, "{meio}: {d} bytes reapareceram onde se desfez");
        // Refazer depois do tecto: a tinta volta à tela, e a que ficou continua viva.
        assert!(t.redo_last());
        assert!(t.canvas_rgba[..] == telas[2][..], "{meio}: refazer");
        assert!(t.paint.wetpaint.session.is_some(), "{meio}: refazer");
        assert!(
            tinta_viva_na_linha(&mut t, 20) > 0.0,
            "{meio}: o 1.º, refeito"
        );
        assert!(t.undo_last());
        assert!(
            t.canvas_rgba[..] == telas[1][..],
            "{meio}: desfazer de novo"
        );
    }
}

/// O traço de água pura sobre a tinta: desfeito para lá do tecto, nada corre a mais.
#[test]
fn para_la_do_tecto_o_traco_que_cruza_nao_deixa_nada() {
    teto::set(Some(1));
    let mut t = tela(600, 400, AGUADA, 20.0);
    traco(&mut t, 40.0, 560.0, 100.0);
    t.ticks_em_blocos(120);
    let antes = t.canvas_rgba.to_vec();
    t.set_brush_color_srgb8([220, 200, 40]);
    t.on_canvas_pointer(ponteiro(300.0, 20.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(300.0, 20.0 + 9.0 * k as f32, PointerPhase::Move));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(300.0, 380.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    assert!(t.undo_last());
    teto::set(None);
    assert!(t.paint.wetpaint.session.is_some());
    assert!(t.canvas_rgba[..] == antes[..], "a tela de antes");
    // A tinta do traço de baixo continua a poder correr; a cor do traço desfeito não pode voltar.
    t.ticks_em_blocos(600);
    let amarelo = t
        .canvas_rgba
        .chunks_exact(4)
        .zip(antes.chunks_exact(4))
        .filter(|(a, b)| {
            i32::from(a[0]) > i32::from(b[0]) + 12 && i32::from(a[2]) < i32::from(b[2]) - 12
        })
        .count();
    assert_eq!(amarelo, 0, "{amarelo} px do traço desfeito voltaram");
}

/// Comprimir a tinta adormecida guarda as fotografias: o desfazer depois de acordar é o de sempre.
#[test]
fn desfazer_depois_de_comprimir_nao_muda_nada() {
    let pinta = |comprime: bool| {
        let mut t = tela(600, 400, DABBER, 20.0);
        for i in 0..4 {
            traco(&mut t, 40.0, 560.0, linha_y(i));
        }
        t.set_paint_media(PaintMedia::Digital);
        if comprime {
            t.paint.wetpaint.adormecidas[0].encolhe_ja();
        }
        t.set_paint_media(PaintMedia::WetPaint);
        t.wet_acorda();
        assert!(t.undo_last());
        assert!(t.undo_last());
        assert!(t.paint.wetpaint.session.is_some());
        t
    };
    let (mut a, mut b) = (pinta(false), pinta(true));
    assert!(a.canvas_rgba[..] == b.canvas_rgba[..], "a tela");
    assert!(grade(&mut a) == grade(&mut b), "a grade");
}

/// Tiles (`TILE`²) com algum valor diferente de zero, e os iguais aos da fotografia anterior.
fn tiles(p: &[f32], antes: &[f32], s: usize) -> (usize, usize, usize) {
    let t = ph2d_wet_paint::grid::TILE;
    let rows = p.len().checked_div(s).unwrap_or(0);
    let (mut total, mut cheios, mut iguais) = (0, 0, 0);
    for ty in 0..rows.div_ceil(t) {
        for tx in 0..s.div_ceil(t) {
            total += 1;
            let (mut algo, mut igual) = (false, true);
            for y in ty * t..((ty + 1) * t).min(rows) {
                let (a, b) = (y * s + tx * t, y * s + ((tx + 1) * t).min(s));
                algo |= p[a..b].iter().any(|v| *v != 0.0);
                igual &= antes.get(a..b).is_some_and(|q| q == &p[a..b]);
            }
            cheios += usize::from(algo);
            iguais += usize::from(igual);
        }
    }
    (total, cheios, iguais)
}

/// O que UM passo do desfazer custa em memória e no pen-down, traço a traço: os registos e as fotografias.
#[test]
#[ignore = "sonda: corre por nome com --ignored --nocapture"]
fn sonda_o_preco_de_um_passo() {
    let cenas: [(usize, usize, &str, f32, f32); 6] = [
        (1920, 1080, DABBER, 60.0, 0.25),
        (1920, 1080, DABBER, 120.0, 1.0),
        (1920, 1080, AGUADA, 120.0, 1.0),
        (4096, 2560, DABBER, 120.0, 0.25),
        (4096, 2560, DABBER, 240.0, 1.0),
        (4096, 2560, AGUADA, 240.0, 1.0),
    ];
    for (w, h, meio, tam, parte) in cenas {
        let mut t = tela(w, h, meio, tam);
        let teto = super::undo_vivo::orcamento_das_fotos(w, h);
        eprintln!(
            "== {w}×{h} {meio:.6} tamanho {tam} comprimento {parte}: orçamento {} MB",
            teto >> 20
        );
        let (mut fx0, mut fy0, mut ol0) = (Vec::new(), Vec::new(), Vec::new());
        let mut hist_antes = 0usize;
        for i in 0..40 {
            let y = ((tam * 0.4 * i as f32) % (h as f32 - 2.0 * tam)) + tam;
            let _ = (tam * 1.2 * (i as f32 + 1.0)).min(h as f32 - tam);
            let x1 = 40.0 + (w as f32 - 80.0) * parte;
            let a = std::time::Instant::now();
            t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
            let down = a.elapsed().as_secs_f64() * 1e3;
            for k in 1..=40 {
                t.on_canvas_pointer(ponteiro(
                    40.0 + (x1 - 40.0) * k as f32 / 40.0,
                    y,
                    PointerPhase::Move,
                ));
                t.ticks_em_blocos(1);
            }
            t.on_canvas_pointer(ponteiro(x1, y, PointerPhase::Up));
            t.ticks_em_blocos(2);
            t.wet_bring_home();
            let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
            let g = &s.engine.layers[0].grid;
            let hist = s.engine.bytes_do_historico(0);
            let fotos = super::undo_vivo::bytes_das_fotos(&s.antes_do_passo);
            let (tot, fx_c, fx_i) = tiles(&g.flow_x, &fx0, g.s);
            let (_, _, fy_i) = tiles(&g.flow_y, &fy0, g.s);
            let (_, ol_c, ol_i) = tiles(&g.oleo_antes, &ol0, g.s);
            eprintln!(
                "traço {i}: pen-down {down:6.1} ms · passos {} · registos {:5} MB (+{:4}) · fotos {:5} MB · \
                 tiles {tot}: fluxo cheios {fx_c} iguais {fx_i}/{fy_i} · ladeira ({} células) cheios {ol_c} iguais {ol_i}",
                s.antes_do_passo.len(),
                hist >> 20,
                hist.saturating_sub(hist_antes) >> 20,
                fotos >> 20,
                g.oleo_antes.len(),
            );
            hist_antes = hist;
            fx0.clone_from(&g.flow_x);
            fy0.clone_from(&g.flow_y);
            ol0.clone_from(&g.oleo_antes);
        }
    }
}

/// A fotografia mais velha da pilha conta inteira, as outras só o que não partilham com a de antes: valores exactos.
/// Depois de o tecto largar as mais velhas, a que ficou à frente ainda partilha tiles com a largada — contá-la só pelos
/// «novos» subconta a pilha.
#[test]
fn a_foto_mais_velha_conta_inteira() {
    let mut t = tela(600, 400, DABBER, 20.0);
    for i in 0..3 {
        traco(&mut t, 40.0, 560.0, linha_y(i));
    }
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let todos = &s.antes_do_passo;
    assert!(todos.len() >= 3, "{} passos", todos.len());
    let fotos = super::undo_vivo::bytes_das_fotos;
    // A 1.ª, sozinha, não partilha com ninguém: é a fotografia inteira (a grade é a mesma, o tamanho também).
    let inteira = fotos(&todos[..1]);
    assert!(inteira > 0);
    // A 2.ª e a 3.ª à frente da pilha (as de trás largadas): inteiras, e não o que acrescentavam às largadas.
    assert_eq!(fotos(&todos[1..2]), inteira, "a 2.ª, à frente");
    // A 3.ª já tem o fluxo (as fotografias de antes dos dois primeiros traços ainda não): é a do motor inteiro.
    let do_motor = super::undo_vivo::FotoDoMotor::bytes_de_para_sonda(&s.engine);
    assert_eq!(fotos(&todos[2..3]), do_motor, "a 3.ª, à frente");
    // CONTROLO: com a 1.ª por baixo, a 2.ª só acrescenta o que mudou.
    let a_mais = fotos(&todos[..2]) - fotos(&todos[..1]);
    assert!(
        a_mais > 0 && a_mais < inteira,
        "CONTROLO: {a_mais} de {inteira}"
    );
}

/// Desfazer um passo que o tecto já largou: os passos que restam e os registos do motor já não descrevem a história, e
/// ficam esquecidos — senão o redo/undo seguinte devolve ao motor tinta de um passo que a tela já não tem.
#[test]
fn para_la_do_tecto_os_passos_que_restam_ficam_esquecidos() {
    teto::set(Some(1));
    let mut t = tela(600, 400, DABBER, 20.0);
    for i in 0..4 {
        traco(&mut t, 40.0, 560.0, linha_y(i));
    }
    t.wet_bring_home();
    let antes = t.paint.wetpaint.session.as_ref().expect("a sessão");
    assert!(
        !antes.antes_do_passo.is_empty() && antes.engine.can_undo(),
        "CONTROLO: o último passo tem registo"
    );
    // O último passo ainda está no histórico: desfaz-se com a fotografia, e o redo dele fica à espera no motor.
    assert!(t.undo_last());
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    assert!(s.engine.can_redo(), "CONTROLO: o redo do último passo");
    // O seguinte já saiu do histórico (tecto = 1): acaba só o dele, e o resto esquece-se.
    assert!(t.undo_last());
    teto::set(None);
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    assert!(
        s.antes_do_passo.is_empty(),
        "{} passos",
        s.antes_do_passo.len()
    );
    assert!(
        !s.engine.can_undo() && !s.engine.can_redo(),
        "os registos do motor sobreviveram ao esquecimento"
    );
}
