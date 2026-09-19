//! ⭐⭐⭐ **AS TRÊS CAIXAS QUE ENCURTARAM GUARDAM A EXPLICAÇÃO NUM BALÃO.**
//!
//! # A ordem, e por que ela tem DUAS metades
//!
//! Em 2026-09-19 a varredura de elisões — com o Inspector armado — mostrou que
//! `Center (makes it a 9-slice Region)` mede `~190 px` numa coluna de `174` e sai
//! `Center (makes it a 9-slice R…`. As três saídas foram devolvidas ao dono com os números, e ele
//! escolheu: ***«Encurtar esses nomes»***.
//!
//! ⚠️⚠️ **Uma ordem que MOVE tem duas metades — tirar e pôr — e elas caem em sítios diferentes do
//! código.** Cumprir só a que REMOVE apaga a explicação **em silêncio**: o rótulo passa a caber,
//! todo gate de largura fica verde, e a frase que dizia o que a caixa FAZ deixou de existir. É a
//! lei que este repo pagou com o gesto de colapsar uma coluna (o gesto saiu, o item de menu nunca
//! foi escrito, e durante dez dias não havia como fechar uma coluna).
//!
//! ⇒ este gate é a metade que PÕE.
//!
//! # ⛔ E ele tem controlo positivo
//!
//! Sem o controlo, um `tooltip_for` que devolvesse sempre `Some` deixaria isto verde sobre um
//! painel sem balão nenhum. O lado que prova que a régua vê a diferença é uma caixa irmã da MESMA
//! secção que legitimamente **não** tem balão — o nome dela já diz tudo.

use ph2d_panel_inspector::{InspectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

/// As que encurtaram, e a explicação que cada uma passou a guardar.
const ENCURTADAS: &[(&str, ph2d_a11y::NodeId)] = &[
    ("Bounds", ids::INSP_ANCHOR_BOUNDS_ON),
    ("Center", ids::INSP_ANCHOR_CENTER_ON),
    // ⚠️ Esta já tinha balão antes da ordem — o que ela perdeu foi o `(no game runtime yet)` do
    //    rótulo, que o balão já dizia por extenso.
    ("Show anchors at runtime", ids::INSP_ANCHOR_VIS_RUNTIME),
];

#[test]
fn as_caixas_que_encurtaram_guardam_a_explicacao_no_balao() {
    let host = MockPanelHost::with_panel::<InspectorPanel>();
    let store = host.store();
    let mudas: Vec<&str> = ENCURTADAS
        .iter()
        .filter(|(_, id)| store.tooltip_for(*id).is_none_or(str::is_empty))
        .map(|(nome, _)| *nome)
        .collect();
    assert!(
        mudas.is_empty(),
        "estas caixas encurtaram o nome e NÃO guardaram a explicação em balão nenhum — a ordem do \
         dono foi cumprida só na metade que remove: {}",
        mudas.join(", ")
    );
    // ⛔ **O CONTROLO POSITIVO**: uma caixa irmã da mesma secção sem balão, que prova que a régua
    //    distingue os dois casos. Se ela passar a ter um, troque-a por outra — *uma régua cujo
    //    controlo desapareceu deixa de afirmar o que diz afirmar*.
    assert!(
        store.tooltip_for(ids::INSP_ANCHOR_VIS_EDITOR).is_none(),
        "o controlo desta régua ganhou um balão: ela já não prova que sabe ver a ausência de um"
    );
}

/// ⭐⭐ **E a explicação não é o rótulo outra vez.**
///
/// ⚠️ Um balão que repetisse o nome da caixa seria a metade que PÕE cumprida na letra e não no
/// espírito: o artista continuaria sem saber o que o interruptor FAZ.
#[test]
fn o_balao_diz_mais_do_que_o_nome() {
    let host = MockPanelHost::with_panel::<InspectorPanel>();
    let store = host.store();
    for (nome, id) in ENCURTADAS {
        let balao = store.tooltip_for(*id).unwrap_or_default();
        assert!(
            balao.len() > nome.len(),
            "o balão de {nome:?} diz {balao:?} — isso não explica nada que o nome já não diga"
        );
    }
}
