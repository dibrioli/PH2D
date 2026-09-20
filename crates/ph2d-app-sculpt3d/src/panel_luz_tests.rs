//! **A PONTE DA LUZ, nos dois sentidos** — irmão (`#[path]`) do [`super`].

use ph2d_mesh_render::{Lighting, MATCAPS};
use ph2d_panel_sculpt3d::state::LightMode;

use super::{luz_do_painel, luz_para_o_painel};

/// Toda luz do device: ida, volta, e o MESMO valor.
///
/// ⚠️ **DERIVADA da crate que declara o tipo, e não escrita aqui.** A redacção anterior listava
/// `[Flat, Rig]` à mão e ficou para trás no dia em que o terceiro modo fixo chegou — o gate acusou
/// `12` contra `13`, que é ele a funcionar, e a cura é a lista deixar de ter dois donos.
fn corpus() -> Vec<Lighting> {
    Lighting::todos()
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
        MATCAPS.len() + FIXOS,
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
    for i in 0..MATCAPS.len() + FIXOS {
        assert_eq!(
            LightMode::from_option_index(i).option_index(),
            i,
            "a opção {i} da fileira não volta a si mesma"
        );
    }
    // O CONTROLO: a escada não é a identidade sobre o índice do matcap — o plano, o rig e a lei
    // que assa ocupam os três primeiros lugares.
    assert_eq!(LightMode::Matcap(0).option_index(), FIXOS);
}

/// **Quantos modos da fileira NÃO são matcaps** — o plano, o rig e a lei que assa.
///
/// ⚠️ Ele é uma constante com nome e não um `3` escrito em duas asserções: *o `2` esteve aqui e
/// sobreviveu à wave que acrescentou o quarto modo* — o gate reprovou com `left: 3, right: 2`, que é
/// exactamente ele a funcionar.
///
/// ⛔⛔ **E ele fica ESCRITO À MÃO de propósito, agora que o PINTOR o deriva** (a
/// `LightMode::FIXOS`): ligá-lo à porta faria o gate comparar a porta consigo mesma e passar a
/// afirmar **nada**. *Um oráculo derivado do sujeito que ele mede não é um oráculo* — e esta casa já
/// o pagou no `the_families_that_the_ui_asks_about_agree_with_the_verb_list`, que comparava a
/// lista consigo própria e ficou
/// verde sobre uma omissão até alguém a procurar.
const FIXOS: usize = 3;
