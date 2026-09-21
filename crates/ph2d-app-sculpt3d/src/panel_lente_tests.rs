//! **A PONTE DA LENTE, nos dois sentidos** — irmão (`#[path]`) do [`super`].
//!
//! Report do dono, 2026-09-21: *«só temos a visão em perspectiva em sculpt. Não temos
//! Ortográfica. Precisamos de ambas»*. A metade do teclado é o `Numpad5`; esta é a do painel, e o
//! que a torna honesta é a ida-e-volta — o molde é o irmão [`super::panel_luz_tests`].

use ph2d_mesh_render::Lens;
use ph2d_panel_sculpt3d::state::LensMode;

use super::{lente_do_painel, lente_para_o_painel};

/// ⚠️ **DERIVADO da crate que declara o tipo**, nunca escrito à mão aqui: uma lista própria
/// ficaria para trás no dia da terceira lente, que é exactamente o defeito que o gate da LUZ
/// apanhou em si mesmo (`12` contra `13`).
fn corpus() -> Vec<Lens> {
    Lens::ALL.to_vec()
}

/// ⭐⭐ **A IDA-E-VOLTA, e é ela que torna a duplicação honesta.**
///
/// ⛔ O conceito *«com que lente»* está escrito em DOIS tipos — o do painel e o da câmera —
/// porque o painel é UI e não arrasta o `wgpu`. Sem este gate, uma lente nova nasceria de um lado
/// só e o chip escolheria **outra coisa em silêncio**.
#[test]
fn toda_lente_da_camera_atravessa_o_painel_e_volta_a_si_mesma() {
    for l in corpus() {
        assert_eq!(
            lente_do_painel(lente_para_o_painel(l)),
            l,
            "a lente {l:?} não volta a si mesma"
        );
    }
}

/// ⭐ **E a VOLTA é o que prova que nenhum modo do painel é INALCANÇÁVEL** — a metade que a ida
/// sozinha não cobre: uma ponte que mandasse as duas lentes para o mesmo modo passaria no gate de
/// cima e deixaria um chip **morto sob o dedo**, que é o report que esta família já pagou sete
/// vezes.
#[test]
fn todo_modo_do_painel_e_alcancavel_pela_camera() {
    let mut vistos: Vec<LensMode> = corpus().into_iter().map(lente_para_o_painel).collect();
    vistos.sort_by_key(|m| m.option_index());
    vistos.dedup();
    assert_eq!(
        vistos.len(),
        MODOS_DO_PAINEL,
        "a ponte colapsa dois modos do painel num só: {vistos:?}"
    );
    assert!(
        vistos.contains(&LensMode::Perspective) && vistos.contains(&LensMode::Ortho),
        "uma das duas lentes não é alcançável da câmera: {vistos:?}"
    );
}

/// ⚠️ **A ESCADA da fileira é a mesma nos dois sítios** — o pintor lê o índice para acender o chip
/// e o despacho inverte-o. ⛔ Uma segunda cópia da aritmética divergiria no dia da terceira lente.
#[test]
fn a_escada_da_fileira_e_uma_involucao() {
    for i in 0..MODOS_DO_PAINEL {
        assert_eq!(
            LensMode::from_option_index(i).option_index(),
            i,
            "a opção {i} da fileira não volta a si mesma"
        );
    }
    // O CONTROLO: a fábrica é a PRIMEIRA opção, e é a convergente — um artista que não toque na
    // fileira continua a ver o que sempre viu.
    assert_eq!(LensMode::default().option_index(), 0);
    assert_eq!(LensMode::default(), LensMode::Perspective);
}

/// **Quantas lentes a fileira oferece.**
///
/// ⛔⛔ Ele fica **ESCRITO À MÃO de propósito**: derivá-lo de `LensMode::ALL` faria o gate comparar
/// a lista consigo mesma e passar a afirmar **nada** — *um oráculo derivado do sujeito que ele mede
/// não é um oráculo*, e esta casa já o pagou no `the_families_that_the_ui_asks_about…`, que ficou
/// verde sobre uma omissão até alguém a procurar.
const MODOS_DO_PAINEL: usize = 2;
