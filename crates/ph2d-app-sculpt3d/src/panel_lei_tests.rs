//! ⭐⭐ **A FILEIRA DA LEI ALCANÇA TODA LEI** — os gates que só esta crate pode escrever.
//!
//! ⚠️ **Ela é a ÚNICA que vê os dois lados:** quem define as leis é a `ph2d-form-donation` (que
//! puxa o `wgpu`) e quem pinta os chips é a `ph2d-panel-sculpt3d` (que não depende dela, de
//! propósito). ⇒ *a amarra entre as duas listas não é escrevível em nenhuma delas*, e sem estes
//! gates uma lei nova nasceria com lei, com gates e **sem chip** — o defeito exacto que o `Density`
//! desta família já pagou (`left: 28, right: 29`).

use ph2d_form_donation::lei_da_luz::{CHAVES_DOS_ROTULOS, Lei};
use ph2d_panel_sculpt3d::ids::SCULPT3D_BAKE_LAW;

/// ⛔⛔ **TODA LEI TEM CHIP, E TODO CHIP TEM LEI** — as duas metades, porque as curas são opostas.
///
/// Uma lei a mais que o array é uma lei **inalcançável** (cura: acrescentar o id); um id a mais é
/// um chip **que não despacha nada** (cura: apagá-lo). *Uma contagem só não separa as duas.*
#[test]
fn toda_lei_tem_chip_e_todo_chip_tem_lei() {
    assert_eq!(
        SCULPT3D_BAKE_LAW.len(),
        Lei::ALL.len(),
        "o array de ids da fileira e o `Lei::ALL` têm de ter o mesmo tamanho — senão uma lei nasce \
         inalcançável pelo artista, ou um chip pinta sem ter para onde despachar"
    );
    assert_eq!(
        CHAVES_DOS_ROTULOS.len(),
        SCULPT3D_BAKE_LAW.len(),
        "cada chip pinta o rótulo da lei de MESMO índice — um a menos deixa um chip sem nome"
    );

    // ⭐ **O CONTROLO da régua:** a população não pode ser vazia, senão as duas igualdades acima
    // são verdadeiras por vácuo e o gate passa a afirmar nada.
    assert!(
        Lei::ALL.len() >= 2,
        "controlo: a fileira existe porque HÁ escolha — com uma lei só ela é um selector morto"
    );

    // ⛔ E os ids têm de ser DISTINTOS: dois chips com o mesmo `NodeId` colapsam num controlo só,
    // que é o defeito que as cinco amostras de cor do modelador pagaram em 19/09.
    assert_ne!(
        SCULPT3D_BAKE_LAW[0], SCULPT3D_BAKE_LAW[1],
        "dois chips com o mesmo id são UM controlo a fingir que são dois"
    );
}

/// ⛔⛔ **O DESPACHO DELEGA A TRADUÇÃO** — e sem isto o `apply_panel_intent` podia ficar com uma
/// segunda resposta a *«que lei é o chip 1?»*.
///
/// ⚠️ A régua é o TEXTO porque o braço vive num `match` sobre `&mut Sculpt3dScene`, e construir uma
/// cena pede um `wgpu::Device` ⇒ um gate a sério nasceria `#[ignore]` e **o CI nunca o correria**.
/// *Quando um gate precisa de um device para medir uma decisão que não tem pixel nenhum, a lei está
/// no sítio errado* — e aqui ela está no sítio certo; o que se mede é a FIAÇÃO.
#[test]
fn o_despacho_do_chip_pergunta_a_porta_da_lei() {
    let fonte = include_str!("panel.rs");
    let i = fonte
        .find("Sculpt3dIntent::LeiDoAlvo(i) =>")
        .expect("controlo: o braço do intent tem de existir com este nome");
    let corpo = &fonte[i..i + 400.min(fonte.len() - i)];
    assert!(
        corpo.contains("Lei::from_index(i)"),
        "o braço tem de traduzir pela porta da crate que define as leis — um `match` local seria a \
         segunda resposta, e as duas divergiriam no dia da terceira lei"
    );
    // ⭐ **A metade que impede a cura barata:** um índice desconhecido tem de ser DESCARTADO, e o
    // `map` sobre o `Option` é o que o faz. Com um `unwrap` ali, um painel desalinhado do motor
    // derrubava o app em vez de não fazer nada.
    assert!(
        !corpo.contains("unwrap()") && !corpo.contains("expect("),
        "um índice que a lei não conhece é descartado, nunca desembrulhado"
    );
}

/// ⛔⛔ **A LEI ATRAVESSA A PONTE ATÉ AO RETRATO** — as três pernas, e a do meio é a que uma
/// mutação sobrevivente obrigou a escrever.
///
/// ⚠️ **O sweep de pintura não a cobre:** ele alimenta o retrato à mão (`lei_do_alvo: Some(1)`) e
/// fica **verde** com um `panel_snapshot` que publique `None` sempre — *um arnês que monta o
/// retrato entra ABAIXO da rotura*, que é a lei desta casa pela quinta vez. O que falta gatear é o
/// FIO: quem recebe a lei tem de a publicar.
///
/// ⚠️ **A régua é o TEXTO** porque construir uma `Sculpt3dScene` pede um `wgpu::Device` — o gate
/// nasceria `#[ignore]` e o CI nunca o correria.
///
/// **Mutação que deve sangrar:** `lei_do_alvo,` → `lei_do_alvo: None,` no retrato.
#[test]
fn a_lei_atravessa_a_ponte_ate_ao_retrato() {
    let bridge = include_str!("panel_bridge.rs");
    let panel = include_str!("panel.rs");

    // (1) A PONTE recebe-a de fora — quem tem o mapa dos objectos assados é o shell.
    assert!(
        bridge.contains("lei_do_alvo: Option<usize>"),
        "o `dispatch` tem de RECEBER a lei: a escultura não sabe que um sprite foi assado"
    );
    // (2) …e entrega-a ao retrato, sem a inventar pelo caminho.
    assert!(
        bridge.contains("panel_snapshot(has_bake_target, lei_do_alvo)"),
        "a ponte tem de PASSAR a lei ao retrato — com um `None` aqui a fileira nunca é pintada e o \
         artista lê isso como «o objecto não tem lei»"
    );
    // (3) …e o retrato PUBLICA o que recebeu. ⚠️ A agulha é montada: um censo que contém a própria
    // agulha encontra-se sempre a si mesmo.
    let i = panel
        .find("fn panel_snapshot")
        .expect("controlo: o retrato tem de existir com este nome");
    let corpo = &panel[i..];
    assert!(
        corpo.contains(concat!("lei_do_alvo", ",\n")),
        "o retrato tem de publicar o parâmetro que recebeu — qualquer constante aqui apaga a \
         fileira em silêncio, com o sweep de pintura VERDE"
    );
}
