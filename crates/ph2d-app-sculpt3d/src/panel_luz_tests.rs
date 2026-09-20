//! **A PONTE DA LUZ, nos dois sentidos** — irmão (`#[path]`) do [`super`].

use ph2d_mesh_render::{Lighting, MATCAPS};
use ph2d_panel_sculpt3d::state::LightMode;

use super::{luz_do_painel, luz_para_o_painel};

/// Toda luz do device: ida, volta, e o MESMO valor.
fn corpus() -> Vec<Lighting> {
    let mut v = vec![Lighting::Flat, Lighting::Rig];
    v.extend((0..MATCAPS.len()).map(|i| Lighting::Matcap(u8::try_from(i).expect("cabe"))));
    v
}

/// ⭐⭐ **A IDA-E-VOLTA, e é ela que torna a duplicação honesta.**
///
/// ⛔ O conceito *«com que luz»* está escrito em DOIS tipos — o do painel e o do
/// device — porque o painel não conhece o renderizador. Sem este gate, um modo
/// novo nasceria de um lado só e o chip escolheria **outra coisa em silêncio**.
#[test]
fn toda_luz_do_device_atravessa_o_painel_e_volta_a_si_mesma() {
    for l in corpus() {
        assert_eq!(
            luz_do_painel(luz_para_o_painel(l)),
            l,
            "a luz {l:?} não volta a si mesma"
        );
    }
}

/// ⭐ **E a VOLTA é o que prova que nenhum modo do painel é inalcançável** — a
/// metade que a ida sozinha não cobre: uma ponte que mandasse dois modos do
/// painel para a mesma luz passaria no gate de cima e deixaria um chip morto.
#[test]
fn todo_modo_do_painel_e_alcancavel_pelo_device() {
    let mut vistos: Vec<LightMode> = corpus().into_iter().map(luz_para_o_painel).collect();
    vistos.sort_by_key(|m| m.option_index());
    vistos.dedup();
    assert_eq!(
        vistos.len(),
        MATCAPS.len() + 2,
        "a ponte colapsa dois modos do painel num só: {vistos:?}"
    );
    assert!(
        vistos.contains(&LightMode::Flat) && vistos.contains(&LightMode::Rig),
        "o plano ou o rig não são alcançáveis do device: {vistos:?}"
    );
}

/// ⚠️ **A ESCADA da fileira é a mesma nos dois sítios** — o pintor lê o índice
/// para acender o chip e o despacho inverte-o. ⛔ Uma segunda cópia da
/// aritmética divergiria no dia do quarto modo, que é o que esta wave foi.
#[test]
fn a_escada_da_fileira_e_uma_involucao() {
    for i in 0..MATCAPS.len() + 2 {
        assert_eq!(
            LightMode::from_option_index(i).option_index(),
            i,
            "a opção {i} da fileira não volta a si mesma"
        );
    }
    // O CONTROLO: a escada não é a identidade sobre o índice do matcap — o
    // plano e o rig ocupam os dois primeiros lugares.
    assert_eq!(LightMode::Matcap(0).option_index(), 2);
}
