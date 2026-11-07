//! **O refazer separa o relevo da tinta** (dono 07/11): a pintura aparece numa etapa e o relevo noutra. A cada passo de N
//! desfazer e N refazer, a cor, o relevo e a imagem mostrada têm de ser os do MESMO estado original.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, ponteiro, preset};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};

const DABBER: &str = "Oil/Dabber/Dabber.ph2d-brush.png";

/// O que o artista vê e o que a tinta é, num instante.
#[derive(Clone, PartialEq)]
struct Estado {
    cor: Vec<u8>,
    relevo_impasto: Vec<u32>,
    relevo_wet: Vec<u32>,
    mostrada: Vec<u8>,
}

fn estado(t: &mut PainterTool) -> Estado {
    t.wet_bring_home();
    let id = t.layers.active().expect("a camada");
    let relevo_impasto = t
        .layer_height_view(id)
        .unwrap_or_default()
        .iter()
        .map(|v| v.to_bits())
        .collect();
    let relevo_wet = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .map_or_else(Vec::new, |s| {
            let g = &s.engine.layers[0].grid;
            g.altura
                .iter()
                .chain(&g.espessura)
                .chain(&g.relevo)
                .map(|v| v.to_bits())
                .collect()
        });
    let mostrada = t
        .take_preview_arc()
        .map(|(p, _, _)| p.to_vec())
        .unwrap_or_default();
    Estado {
        cor: t.canvas_rgba.to_vec(),
        relevo_impasto,
        relevo_wet,
        mostrada,
    }
}

fn vazio_ou_zero(v: &[u32]) -> bool {
    v.iter().all(|b| f32::from_bits(*b) == 0.0)
}

/// Dois planos de relevo são o mesmo se só diferem por um deles não existir ainda.
fn relevo_igual(a: &[u32], b: &[u32]) -> bool {
    a == b || (vazio_ou_zero(a) && vazio_ou_zero(b))
}

/// A sessão adormecida não tem grade à vista: o relevo do Wet só se compara com as duas acordadas.
fn relevo_wet_igual(a: &[u32], b: &[u32]) -> bool {
    a.is_empty() || b.is_empty() || relevo_igual(a, b)
}

fn tela(w: usize, h: usize) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t
}

fn pincel_wet(t: &mut PainterTool) {
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset(DABBER)).expect("o meio");
    t.set_brush_size_px(24.0);
}

fn pincel_impasto(t: &mut PainterTool) {
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_size_px(24.0);
    t.paint.brush.impasto = true;
    t.paint.brush.impasto_depth = 0.5;
}

fn traco(t: &mut PainterTool, i: usize, w: f32, h: f32) {
    let y = 16.0 + (h - 32.0) * (i as f32 % 6.0) / 6.0;
    t.set_brush_color_srgb8([40 + 30 * i as u8, 90, 220 - 30 * i as u8]);
    t.on_canvas_pointer(ponteiro(10.0, y, PointerPhase::Down));
    for k in 1..=24 {
        t.on_canvas_pointer(ponteiro(
            10.0 + (w - 20.0) * k as f32 / 24.0,
            y + 18.0 * (k as f32 / 4.0).sin(),
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(w - 10.0, y, PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// `passos`: o pincel de cada traço. Devolve os passos em que a cor, o relevo ou a imagem não são do estado pedido.
fn cena(
    nome: &str,
    (w, h): (usize, usize),
    passos: &[fn(&mut PainterTool)],
    assenta: bool,
) -> Vec<String> {
    let mut t = tela(w, h);
    let mut orig = vec![estado(&mut t)];
    for (i, p) in passos.iter().enumerate() {
        p(&mut t);
        traco(&mut t, i, w as f32, h as f32);
        if assenta {
            t.ticks_em_blocos(120);
        }
        orig.push(estado(&mut t));
    }
    let mut erros = Vec::new();
    let confere = |t: &mut PainterTool, passo: String, quer: usize, erros: &mut Vec<String>| {
        let (e, q) = (estado(t), &orig[quer]);
        for (parte, certo) in [
            ("cor", q.cor == e.cor),
            (
                "relevo do impasto",
                relevo_igual(&q.relevo_impasto, &e.relevo_impasto),
            ),
            // Na pilha mista, desfazer através do outro meio esquece os passos: o refazer devolve a tinta morta, de propósito.
            (
                "relevo do wet",
                nome.starts_with("misto") || relevo_wet_igual(&q.relevo_wet, &e.relevo_wet),
            ),
            ("imagem", q.mostrada == e.mostrada),
        ] {
            if !certo {
                erros.push(format!("{nome} {passo}→{quer}: {parte}"));
            }
        }
    };
    let n = passos.len();
    for rodada in ["1.ª", "2.ª"] {
        for k in (0..n).rev() {
            assert!(t.undo_last(), "{nome}: desfazer");
            confere(&mut t, format!("{rodada} desfaz"), k, &mut erros);
        }
        for k in 1..=n {
            assert!(t.redo_last(), "{nome}: refazer");
            confere(&mut t, format!("{rodada} refaz"), k, &mut erros);
        }
    }
    erros
}

/// ⭐ Wet com óleo, Impasto e as pilhas mistas: em cada passo de N desfazer e N refazer, tudo é do mesmo estado.
#[test]
fn cada_passo_devolve_cor_relevo_e_imagem_do_mesmo_estado() {
    let wet: fn(&mut PainterTool) = pincel_wet;
    let imp: fn(&mut PainterTool) = pincel_impasto;
    let cenas: [(&str, (usize, usize), Vec<fn(&mut PainterTool)>, bool); 6] = [
        ("wet", (W, H), vec![wet; 5], true),
        ("wet sem assentar", (W, H), vec![wet; 5], false),
        ("impasto", (W, H), vec![imp; 5], false),
        ("impasto a 1024", (1024, 1024), vec![imp; 5], false),
        ("misto", (W, H), vec![imp, wet, imp, wet, imp, wet], false),
        (
            "misto ao contrário",
            (W, H),
            vec![wet, imp, wet, imp, wet, imp],
            true,
        ),
    ];
    let erros: Vec<String> = cenas
        .iter()
        .flat_map(|(nome, tam, passos, assenta)| cena(nome, *tam, passos, *assenta))
        .collect();
    assert!(erros.is_empty(), "{erros:#?}");
}

/// A imagem que a tela mostra: o último quadro que o compositor entregou.
fn quadro(t: &mut PainterTool, visto: &mut Vec<u8>) -> bool {
    std::thread::sleep(std::time::Duration::from_millis(4));
    t.on_tick(1000.0 / 60.0);
    match t.take_preview_arc() {
        Some((p, _, _)) if *p != *visto => {
            visto.clone_from(&p);
            true
        }
        _ => false,
    }
}

/// Quadros até 30 seguidos sem mudar (máximo 600).
fn assenta_real(t: &mut PainterTool, visto: &mut Vec<u8>) {
    let mut calmos = 0;
    for _ in 0..600 {
        calmos = if quadro(t, visto) { 0 } else { calmos + 1 };
        if calmos >= 30 {
            return;
        }
    }
    eprintln!("⚠️ não assentou em 600 quadros");
}

/// Com o relógio do app: em cada comando, a sequência de estados originais que a tela mostra quadro a quadro.
/// `seguidos`: aperta o comando sem quadro nenhum entre dois.
fn cena_real(
    nome: &str,
    (w, h): (usize, usize),
    tamanho: f32,
    pincel: fn(&mut PainterTool),
    n: usize,
    seguidos: bool,
) -> Vec<String> {
    let mut t = tela(w, h);
    t.set_wet_relogio_fixo(false);
    t.set_compor_por_quadro(true);
    let mut visto = Vec::new();
    pincel(&mut t);
    t.set_brush_size_px(tamanho);
    assenta_real(&mut t, &mut visto);
    let mut orig = vec![visto.clone()];
    for i in 0..n {
        traco(&mut t, i, w as f32, h as f32);
        assenta_real(&mut t, &mut visto);
        orig.push(visto.clone());
    }
    let qual = |v: &[u8]| -> String {
        orig.iter()
            .rposition(|o| o == v)
            .map_or_else(|| "?".to_owned(), |i| i.to_string())
    };
    let comando = |t: &mut PainterTool,
                   desfaz: bool,
                   quer: usize,
                   linha: &mut Vec<String>,
                   visto: &mut Vec<u8>| {
        assert!(if desfaz { t.undo_last() } else { t.redo_last() });
        if seguidos {
            return;
        }
        let mut seq = vec![qual(visto)];
        for _ in 0..60 {
            if quadro(t, visto) {
                seq.push(qual(visto));
            }
        }
        let fim = seq.last().cloned().unwrap_or_default();
        linha.push(format!(
            "{}→{quer}: {}{}",
            if desfaz { "desfaz" } else { "refaz" },
            seq.join(">"),
            if fim == quer.to_string() { "" } else { " ⛔" }
        ));
    };
    let mut linha = Vec::new();
    for k in (0..n).rev() {
        comando(&mut t, true, k, &mut linha, &mut visto);
    }
    if seguidos {
        assenta_real(&mut t, &mut visto);
        if qual(&visto) != "0" {
            linha.push(format!("{n} desfazer seguidos: {} ⛔", qual(&visto)));
        }
    }
    for k in 1..=n {
        comando(&mut t, false, k, &mut linha, &mut visto);
    }
    if seguidos {
        assenta_real(&mut t, &mut visto);
        if qual(&visto) != n.to_string() {
            linha.push(format!("{n} refazer seguidos: {} ⛔", qual(&visto)));
        }
    }
    linha.retain(|l| l.contains('⛔'));
    linha
}

fn pincel_de_fabrica(t: &mut PainterTool, nome: &str) {
    t.preset_aplicar(&preset(nome)).expect("o meio");
    t.set_brush_size_px(24.0);
}

/// ⭐ O defeito do dono (07/11): refazer o 1.º traço de óleo devolvia a tinta e, no quadro seguinte, tirava-lhe o relevo
/// (13 796 px, Δ até 187), que só voltava no refazer seguinte. `restaura` contava «há Óleo Real?» antes de pôr os tiles.
#[test]
fn refazer_o_primeiro_traco_nao_perde_o_relevo() {
    for nome in [
        DABBER,
        "Oil/Flat/Flat Bristle.ph2d-brush.png",
        "Oil/Flat/Thick Impasto.ph2d-brush.png",
        "Oil/Knife/Knife Oily.ph2d-brush.png",
    ] {
        let mut t = tela(W, H);
        t.set_compor_por_quadro(true);
        let mut visto = Vec::new();
        pincel_de_fabrica(&mut t, nome);
        assenta_real(&mut t, &mut visto);
        traco(&mut t, 0, W as f32, H as f32);
        assenta_real(&mut t, &mut visto);
        let (original, cor) = (visto.clone(), t.canvas_rgba.to_vec());
        assert!(
            original.chunks_exact(4).any(|p| p[3] > 0),
            "{nome}: a fixture não pintou"
        );
        assert!(t.undo_last(), "{nome}: desfazer");
        assenta_real(&mut t, &mut visto);
        t.wet_bring_home();
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        assert!(
            !s.engine.layers[0].grid.oleo_real,
            "{nome}: sem tinta, o Óleo Real não desliga"
        );
        assert!(t.redo_last(), "{nome}: refazer");
        for q in 0..60 {
            quadro(&mut t, &mut visto);
            assert!(visto == original, "{nome}: a imagem do quadro {q} mudou");
            assert!(
                t.canvas_rgba[..] == cor[..],
                "{nome}: a tela do quadro {q} mudou"
            );
        }
        t.wet_bring_home();
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        assert!(
            s.engine.layers[0].grid.oleo_real,
            "{nome}: o Óleo Real não voltou"
        );
        assert!(
            !s.planos.sombra.is_empty(),
            "{nome}: a sombra do relevo não voltou"
        );
    }
}

/// O relógio do app: a tela acaba no estado pedido, passo a passo e com os comandos sem quadro entre eles. Sensível à
/// carga por ser o relógio da parede: o portão é [`desfazer_e_refazer_recusam_a_imagem_no_ar`].
#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn a_mao_real_com_os_comandos_seguidos_acaba_no_estado_certo() {
    for (nome, tam, s, n) in [("tela", (W, H), 24.0, 5), ("1024", (1024, 768), 80.0, 8)] {
        for pincel in [pincel_wet as fn(&mut PainterTool), pincel_impasto] {
            for seguidos in [false, true] {
                let erros = cena_real(nome, tam, s, pincel, n, seguidos);
                assert!(erros.is_empty(), "{erros:#?}");
            }
        }
    }
}

/// ⭐ A segunda cara do defeito do dono: uma imagem pedida antes de desfazer ou refazer chegava depois e ficava na tela,
/// um passo atrás (`refaz→6: 5`, `desfaz→6: 7>6>7`), até ao comando seguinte. Não se aplica; vai pela dívida.
#[test]
fn desfazer_e_refazer_recusam_a_imagem_no_ar() {
    use super::undo_tinta_morre_tests as m;
    for refaz in [false, true] {
        let (mut aplicadas, mut no_ar) = (0, 0);
        for rodada in 0..6 {
            super::offthread::otica::forca_no_teste(Some(true));
            let mut t = m::tela(600, 400, m::AGUADA, 40.0);
            t.set_wet_relogio_fixo(false);
            t.set_compor_por_quadro(true);
            m::traco(&mut t, 40.0, 560.0, 170.0);
            m::traco(&mut t, 40.0, 560.0, 230.0);
            if refaz {
                assert!(t.undo_last());
            }
            for _ in 0..300 {
                std::thread::sleep(std::time::Duration::from_millis(1 + rodada % 3));
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
                let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
                if s.otica.no_ar.is_some() {
                    break;
                }
            }
            let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
            let (epoca, antes) = (s.otica.no_ar, s.otica.aplicadas);
            no_ar += usize::from(epoca.is_some());
            assert!(if refaz { t.redo_last() } else { t.undo_last() });
            // Só até a resposta do pedido de antes do comando: as imagens seguintes são novas.
            for _ in 0..500 {
                std::thread::sleep(std::time::Duration::from_millis(2));
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
                let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
                if s.otica.no_ar != epoca {
                    aplicadas += usize::from(s.otica.aplicadas > antes);
                    break;
                }
            }
            super::offthread::otica::forca_no_teste(None);
        }
        assert!(no_ar > 0, "CONTROLO: nenhuma imagem no ar");
        assert_eq!(
            aplicadas, 0,
            "refaz={refaz}: {aplicadas} de {no_ar} imagens velhas"
        );
    }
}
