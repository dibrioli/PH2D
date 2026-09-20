//! Os gates da paleta de pincéis.

use super::{build, build_from, verb_at};
use crate::ids::SCULPT3D_VERB;
use ph2d_sculpt3d::Verb;

/// ⛔⛔⛔ **O CATÁLOGO E A LISTA DE IDS TÊM O MESMO TAMANHO** — o gate que impede o pincel
/// inalcançável.
///
/// Os dois são emparelhados por índice (a convenção que o `seg` do painel usa desde sempre), logo
/// um verbo acrescentado sem o id correspondente **existe, tem lei, tem gates, e o artista não lhe
/// chega**. ⚠️ É exactamente o defeito que o `Density` desta mesma família pagou em 2026-09-14: o
/// `SCULPT3D_VERB` é um array escrito à mão e o gate leu `28` contra `29`.
///
/// *Mutação que sangra:* encurtar qualquer um dos dois.
#[test]
fn o_catalogo_e_a_lista_de_ids_tem_o_mesmo_tamanho() {
    assert_eq!(
        Verb::ALL.len(),
        SCULPT3D_VERB.len(),
        "o catálogo tem {} verbos e a lista de ids tem {} — o par é por ÍNDICE, logo a diferença \
         é um pincel sem porta (ou um id sem pincel).",
        Verb::ALL.len(),
        SCULPT3D_VERB.len(),
    );
}

/// ⭐⭐⭐ **TODO pincel do catálogo chega à paleta, e cada um com o id que o painel já despachava.**
///
/// ⚠️ As **duas** metades: a contagem (nenhum se perde) e a IDENTIDADE (o id é o do painel). Sem a
/// segunda, uma paleta que cunhasse ids próprios passaria a contagem e deixaria todo item morto —
/// *«um comando com dois ids tem dois sítios a apodrecer em separado»*.
///
/// *Mutação que sangra:* a paleta cunhar o id a partir do rótulo, como a das formas faz.
#[test]
fn todo_pincel_chega_a_paleta_com_o_id_do_painel() {
    let m = build();
    let ids: Vec<_> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|i| i.id)
        .collect();
    assert_eq!(ids.len(), Verb::ALL.len(), "a paleta perdeu pincéis");
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(*id, SCULPT3D_VERB[i], "o item {i} não usa o id do painel");
    }
}

/// ⭐⭐ **E a volta: todo id da paleta resolve para o verbo certo.**
///
/// ⚠️ **Ida-e-volta nos DOIS sentidos**, porque uma ponte que colapsasse dois verbos num só
/// passaria a ida (a contagem bate) e daria o pincel errado — a mesma lei que a ponte do modo de
/// luz desta família já paga.
///
/// *Mutação que sangra:* o `verb_at` devolver `Verb::ALL[0]` para tudo.
#[test]
fn o_id_da_paleta_resolve_para_o_verbo_certo() {
    for (i, v) in Verb::ALL.iter().enumerate() {
        assert_eq!(
            verb_at(SCULPT3D_VERB[i]),
            Some(*v),
            "o id {i} devolveu o verbo errado",
        );
    }
    // ⚠️ **O CONTROLO**: um id que não é de verbo não resolve. Sem ele, um `verb_at` que
    // devolvesse `Some(Draw)` para qualquer coisa passaria o laço de cima.
    assert_eq!(
        verb_at(crate::ids::SCULPT3D_SEC_TOOL),
        None,
        "o cabeçalho da secção não é um pincel",
    );
}

/// ⚠️ **UM verbo sem id é SALTADO e não estoura** — a cerca que faz o `build_from` ser total.
///
/// ⛔ Ela não substitui o [`o_catalogo_e_a_lista_de_ids_tem_o_mesmo_tamanho`]: aquele é quem acusa
/// o desemparelhamento, este só garante que a paleta não mata o quadro enquanto isso.
#[test]
fn um_catalogo_maior_que_a_lista_de_ids_nao_estoura() {
    let demais: Vec<Verb> = Verb::ALL.iter().copied().cycle().take(100).collect();
    let m = build_from(&demais);
    let n = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .count();
    assert_eq!(n, SCULPT3D_VERB.len(), "a paleta emitiu item sem id");
}
