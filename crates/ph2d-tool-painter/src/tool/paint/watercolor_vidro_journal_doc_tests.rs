//! **O vidro no journal por tile** (ADR-0204), segunda metade: o documento inteiro (cada vidro e cada tela) contra
//! o oráculo nos passos 5–7 — o Wet sobre a aguada, a camada nova e o tecto do desfazer. Partido de
//! `watercolor_vidro_journal_tests.rs` pelo tecto de 700 linhas; os ajudantes moram lá.

use super::watercolor_vidro_journal_tests::{H, W, ev, plano, tool_wb, traco_k, vai_e_volta};
use crate::compositor::vidro::Vidro;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::undo::journal::TILE;
use ph2d_editor_core::tool::PointerPhase;

/// Todo o documento que o oráculo compara: cada vidro e cada tela (a activa e as guardadas).
/// Cada vidro (por camada) e cada tela.
type Documento = (Vec<(crate::layers::LayerId, Vec<Vidro>)>, Vec<Vec<u8>>);

fn documento(t: &PainterTool) -> Documento {
    let vidros = t.vidros.iter().map(|(k, v)| (*k, v.to_vec())).collect();
    let mut telas = vec![t.canvas_rgba.to_vec()];
    telas.extend(t.images.values().map(|i| i.rgba8.to_vec()));
    (vidros, telas)
}

/// `j` (a chave) e `o` (o oráculo) fazem o mesmo gesto e ficam ao bit.
fn par(
    f: &mut dyn FnMut(&mut PainterTool),
    j: &mut PainterTool,
    o: &mut PainterTool,
    quando: &str,
) {
    crate::undo::window::set_vidro_pelo_journal(true);
    f(j);
    crate::undo::window::set_vidro_pelo_journal(false);
    f(o);
    crate::undo::window::set_vidro_pelo_journal(true);
    assert!(
        documento(j) == documento(o),
        "{quando}: o documento difere do oráculo"
    );
}

fn desfaz_e_refaz_tudo(j: &mut PainterTool, o: &mut PainterTool, n: usize, depois: &Documento) {
    for k in 0..n {
        par(
            &mut |t| assert!(t.undo_last(), "desfazer {k}"),
            j,
            o,
            &format!("desfeito {k}"),
        );
    }
    for k in 0..n {
        par(
            &mut |t| assert!(t.redo_last(), "refazer {k}"),
            j,
            o,
            &format!("refeito {k}"),
        );
    }
    assert!(
        documento(j) == *depois,
        "refeito tudo, o documento não é o de antes de desfazer"
    );
}

/// **ADR-0204 passo 5: o composite Wet sobre a aguada volta ao bit no desfazer.** Aguadas em papel de cor, Wet
/// Paint por cima na mesma camada (o `molhada` escreve o vidro pelo `abre_o_vidro`), drenando cada quadro; depois
/// de cada traço, de desfazer tudo e de refazer tudo, o documento é o do oráculo ao bit.
#[test]
fn o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer() {
    let mut j = tool_wb(true);
    let mut o = tool_wb(false);
    par(
        &mut |t| t.papel = Some([200, 170, 120]),
        &mut j,
        &mut o,
        "papel",
    );
    for k in 0..3 {
        par(
            &mut |t| traco_k(t, k),
            &mut j,
            &mut o,
            &format!("aguada {k}"),
        );
    }
    let aguada = plano(&j);
    par(
        &mut |t| {
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_brush_color_srgb8([200, 40, 30]);
        },
        &mut j,
        &mut o,
        "Wet",
    );
    for k in 0..3 {
        par(
            &mut |t| vai_e_volta(t, k, true),
            &mut j,
            &mut o,
            &format!("Wet {k}"),
        );
    }
    assert!(plano(&j) != aguada, "controlo: o Wet não escreveu o vidro");
    let depois = documento(&j);
    desfaz_e_refaz_tudo(&mut j, &mut o, 6, &depois);
}

/// **ADR-0204 passo 6: a camada nova e o vidro elidido dão o bit.** Dois traços WB na base, «+», dois na nova,
/// um na base; desfaz tudo até antes do «+» e refaz tudo: o documento ao bit do oráculo. O 1.º traço na base
/// depois do «+» não copia o plano do vidro (o oráculo copia: o controlo).
#[test]
fn a_camada_nova_e_o_vidro_elidido_dao_o_bit() {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    let mut j = tool_wb(true);
    let mut o = tool_wb(false);
    par(
        &mut |t| t.papel = Some([200, 170, 120]),
        &mut j,
        &mut o,
        "papel",
    );
    let base = j.layers.active().unwrap();
    assert_eq!(o.layers.active(), Some(base));
    for k in 0..2 {
        par(&mut |t| traco_k(t, k), &mut j, &mut o, &format!("base {k}"));
    }
    par(
        &mut |t| {
            t.add_raster_layer("nova").expect("+");
        },
        &mut j,
        &mut o,
        "+",
    );
    for k in 2..4 {
        par(&mut |t| traco_k(t, k), &mut j, &mut o, &format!("nova {k}"));
    }
    par(
        &mut |t| t.select_layer(base),
        &mut j,
        &mut o,
        "volta à base",
    );
    COPIAS_DO_VIDRO.set(0);
    crate::undo::window::set_vidro_pelo_journal(true);
    traco_k(&mut j, 4);
    let copias = COPIAS_DO_VIDRO.get();
    crate::undo::window::set_vidro_pelo_journal(false);
    COPIAS_DO_VIDRO.set(0);
    traco_k(&mut o, 4);
    let copias_oraculo = COPIAS_DO_VIDRO.get();
    crate::undo::window::set_vidro_pelo_journal(true);
    assert!(
        documento(&j) == documento(&o),
        "base depois do +: o documento difere do oráculo"
    );
    println!(
        "cópias do plano do vidro no 1.º traço na base depois do «+»: {copias} (oráculo {copias_oraculo})"
    );
    assert!(
        copias_oraculo >= 1,
        "controlo: sem a chave o traço na base depois do «+» copia o plano ({copias_oraculo})"
    );
    assert_eq!(
        copias, 0,
        "pelo journal o 1.º traço na base depois do «+» não copia o plano do vidro"
    );
    assert_eq!(j.vidros.len(), 2, "controlo: as duas camadas têm vidro");
    let depois = documento(&j);
    // Desfaz tudo (traço, troca, dois traços, «+», dois traços), nos dois ao mesmo passo.
    let mut n = 0;
    loop {
        crate::undo::window::set_vidro_pelo_journal(true);
        let a = j.undo_last();
        crate::undo::window::set_vidro_pelo_journal(false);
        let b = o.undo_last();
        crate::undo::window::set_vidro_pelo_journal(true);
        assert_eq!(a, b, "desfazer {n}: um lado parou antes");
        if !a {
            break;
        }
        assert!(
            documento(&j) == documento(&o),
            "desfeito {n}: o documento difere do oráculo"
        );
        n += 1;
    }
    assert!(
        n >= 6,
        "controlo: só {n} passos desfeitos (o «+» ficou de fora)"
    );
    assert!(
        j.vidros.len() < 2,
        "controlo: desfeito tudo, a camada nova ainda tem vidro"
    );
    for k in 0..n {
        par(
            &mut |t| assert!(t.redo_last(), "refazer {k}"),
            &mut j,
            &mut o,
            &format!("refeito {k}"),
        );
    }
    assert!(
        documento(&j) == depois,
        "refeito tudo, o documento não é o de antes de desfazer"
    );
}

/// Os bytes dos tiles onde `a` e `b` diferem (cada tile pelo tamanho real).
fn bytes_dos_tiles_que_diferem(a: &[Vidro], b: &[Vidro]) -> usize {
    let (w, h) = (W as usize, H as usize);
    let mut soma = 0;
    for ty in 0..h.div_ceil(TILE) {
        for tx in 0..w.div_ceil(TILE) {
            let (x0, x1, y0, y1) = (
                tx * TILE,
                ((tx + 1) * TILE).min(w),
                ty * TILE,
                ((ty + 1) * TILE).min(h),
            );
            if (y0..y1).any(|y| a[y * w + x0..y * w + x1] != b[y * w + x0..y * w + x1]) {
                soma += (x1 - x0) * (y1 - y0) * size_of::<Vidro>();
            }
        }
    }
    soma
}

/// **ADR-0204 D5: o tecto conta o journal do vidro.** Com o gesto ABERTO (3.º traço WB a meio), o journal do passo
/// soma em `journal_bytes_retidos` e o da sessão no instantâneo: largar a sessão baixa o instantâneo em pelo menos os
/// tiles que mudaram desde o início dela (o oráculo `GUARDA_O_INICIO_DA_SESSAO`) e em menos que o plano. Com o mesmo
/// `max_bytes`, a pilha pelo journal retém pelo menos os passos do oráculo (a chave desligada).
#[test]
fn o_tecto_do_desfazer_conta_o_journal_do_vidro() {
    use super::watercolor_vidro::GUARDA_O_INICIO_DA_SESSAO;
    GUARDA_O_INICIO_DA_SESSAO.set(true);
    let mut t = tool_wb(true);
    traco_k(&mut t, 0);
    traco_k(&mut t, 1);
    // O 3.º traço, aberto.
    ev(&mut t, [30.0, 40.0 + 82.0], PointerPhase::Down);
    for s in 1..=8 {
        let f = s as f32 / 8.0;
        ev(
            &mut t,
            [30.0 + 270.0 * f, 122.0 + 30.0 * f],
            PointerPhase::Move,
        );
    }
    let ws = &t.undo.write_state;
    let passo = ws.vidro_journal_bytes();
    assert!(
        passo > 0,
        "controlo: o gesto aberto capturou tiles do vidro no passo"
    );
    assert!(
        ws.journal_bytes_retidos() >= passo,
        "o journal do passo do vidro ({passo} B) conta em journal_bytes_retidos"
    );
    let inicio = t
        .vidro_congelado
        .aguada
        .as_ref()
        .and_then(|c| c.2.inicio.clone())
        .expect("controlo: a aguada tem sessão viva com o oráculo do início");
    let mudou = bytes_dos_tiles_que_diferem(&inicio, &plano(&t));
    let plano_inteiro = (W * H) as usize * size_of::<Vidro>();
    let com = t.bytes_do_instantaneo();
    let sessao = std::mem::take(&mut t.vidro_congelado);
    let sem = t.bytes_do_instantaneo();
    t.vidro_congelado = sessao;
    GUARDA_O_INICIO_DA_SESSAO.set(false);
    println!(
        "passo {passo} B · sessão no instantâneo {} B · tiles mudados desde o início {mudou} B · plano {plano_inteiro} B",
        com - sem
    );
    assert!(mudou > 0, "controlo: a sessão mudou o vidro");
    assert!(
        com - sem >= mudou && com - sem < plano_inteiro,
        "o instantâneo conta a sessão do vidro: {} B, os tiles mudados pedem ≥ {mudou} B e < {plano_inteiro} B",
        com - sem
    );
    ev(&mut t, [300.0, 152.0], PointerPhase::Up);

    // A pilha: o mesmo orçamento, as duas rotas.
    const N: usize = 8;
    let cheia = {
        let mut t = tool_wb(false);
        (0..N).for_each(|k| traco_k(&mut t, k));
        t.undo.retained_bytes()
    };
    let passos = |journal: bool| {
        let mut t = tool_wb(journal);
        t.undo.set_max_bytes(cheia / 2);
        (0..N).for_each(|k| traco_k(&mut t, k));
        t.undo.undo_depth()
    };
    let (com, sem) = (passos(true), passos(false));
    crate::undo::window::set_vidro_pelo_journal(true);
    println!(
        "com max_bytes = {} B: passos retidos com a chave {com} · sem {sem}",
        cheia / 2
    );
    assert!(
        sem < N,
        "controlo: o orçamento morde o oráculo ({sem} de {N})"
    );
    assert!(
        com >= sem,
        "pelo journal a pilha retém ≥ os passos do oráculo: {com} < {sem}"
    );
}
