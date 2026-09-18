//! ⭐ **O painel pinta tudo o que o modelo aceita** — os tectos das listas autoráveis e as tabelas de
//! ids das secções do Inspector são o MESMO facto, e só a shell vê as duas crates.
//!
//! ⚠️ *Um modelo que aceita o que o painel não mostra produz estado inalcançável* (a lei do
//! `ANIM_TAGS_MAX`). As duas secções abaixo afirmavam, no cabeçalho dos ids delas, que *«um gate na
//! shell»* amarrava os dois números — e para a do CÉREBRO (TOP-20 #15) esse gate **não existia**:
//! a afirmação foi escrita e o teste não. *Uma lei escrita sem o gate que ela nomeia é um palpite com
//! cara de medição* — ele nasce aqui, com o do SCRIPT (TOP-20 #16).

use ph2d_panel_inspector::ids;

/// **Mutação que deve sangrar:** mudar o `PROPS_MAX` ou encurtar uma das tabelas.
#[test]
fn o_script_nao_declara_mais_propriedades_do_que_a_seccao_pinta() {
    let tecto = ph2d_script::PROPS_MAX;
    for (nome, n) in [
        ("INSP_SCRIPT_NUM", ids::INSP_SCRIPT_NUM.len()),
        ("INSP_SCRIPT_BOOL", ids::INSP_SCRIPT_BOOL.len()),
        ("INSP_SCRIPT_TEXT", ids::INSP_SCRIPT_TEXT.len()),
        ("INSP_SCRIPT_RESET", ids::INSP_SCRIPT_RESET.len()),
    ] {
        assert_eq!(
            n, tecto,
            "a tabela `{nome}` tem {n} linhas e um script pode declarar {tecto} propriedades"
        );
    }
}

/// **Mutação que deve sangrar:** mudar o `STATES_MAX`/`TRANSITIONS_MAX` ou encurtar uma tabela.
#[test]
fn o_cerebro_nao_aceita_mais_estados_nem_setas_do_que_a_seccao_pinta() {
    assert_eq!(ids::INSP_SM_STATE_ROW.len(), ph2d_ecs::STATES_MAX);
    assert_eq!(ids::INSP_SM_TRANS_ROW.len(), ph2d_ecs::TRANSITIONS_MAX);
}

/// ⭐⭐ **O documento não aceita mais cutscenes do que o selector consegue endereçar** (TOP-20 #19).
///
/// ⚠️ **O chrome não sabe cunhar um id em runtime**, logo a 17.ª cutscene seria uma que o artista vê
/// na aba *Containers* da timeline e **não consegue escolher** no Inspector — e o sintoma seria uma
/// linha que simplesmente não aparece na lista, sem uma palavra a dizer porquê.
///
/// ⚠️ **Este gate mora AQUI e não no painel:** a `ph2d-panel-inspector` não depende da
/// `ph2d-timeline` **de propósito** (o snapshot leva-lhe os nomes prontos), e a shell é a única
/// crate que vê as duas.
///
/// **Mutação que deve sangrar:** mudar o `MAX_CONTAINERS` ou encurtar a tabela de ids.
#[test]
fn o_documento_nao_aceita_mais_cutscenes_do_que_o_selector_enderaca() {
    assert_eq!(
        ids::INSP_SEQ_OPT.len(),
        ph2d_timeline::MAX_CONTAINERS,
        "o selector da cutscene endereça {} containers e o documento aceita {}",
        ids::INSP_SEQ_OPT.len(),
        ph2d_timeline::MAX_CONTAINERS
    );
}

/// ⭐⭐ **O modelo não aceita mais regras do que o selector consegue endereçar.**
///
/// **Mutação que deve sangrar:** mudar o `WATCHES_MAX` ou encurtar a tabela de ids.
#[test]
fn o_modelo_nao_aceita_mais_regras_do_que_a_seccao_pinta() {
    assert_eq!(
        ids::INSP_WATCH_ROW.len(),
        ph2d_ecs::WATCHES_MAX,
        "a lista da vigia endereça {} regras e o modelo aceita {}",
        ids::INSP_WATCH_ROW.len(),
        ph2d_ecs::WATCHES_MAX
    );
}

/// ⭐⭐⭐ **A tabela de opções do chip cobre as variantes do `Compare`, e a ORDEM é a mesma.**
///
/// ⚠️ **As duas metades, e a segunda é a que importa:** uma quarta comparação no motor sem uma
/// quarta entrada no painel é uma comparação que o artista nunca escolhe; e a **posição** na tabela
/// de ids É o `u8` que a edição carrega, logo reordenar uma delas reescreve o sentido de toda regra
/// já gravada, **em silêncio**.
///
/// **Mutação que deve sangrar:** trocar dois braços do `compare_de_u8`, ou encurtar
/// `INSP_WATCH_CMP_OPT`.
#[test]
fn o_chip_da_comparacao_cobre_o_enum_e_respeita_a_ordem_dele() {
    use ph2d_app_components::counter_watch_inspector::{compare_de_u8, u8_de_compare};
    assert_eq!(
        ids::INSP_WATCH_CMP_OPT.len(),
        ph2d_app_components::counter_watch_inspector::comparacoes(),
        "o chip endereça um número de comparações diferente do que o motor tem"
    );
    // A ORDEM, nomeada — ⛔ não é um laço sobre um `ALL`, porque o que se afirma é o VALOR de cada
    // posição, e um laço derivado da mesma fonte concordaria consigo mesmo.
    assert_eq!(compare_de_u8(0), ph2d_ecs::Compare::AtMost);
    assert_eq!(compare_de_u8(1), ph2d_ecs::Compare::AtLeast);
    assert_eq!(compare_de_u8(2), ph2d_ecs::Compare::Exactly);
    // E a ida-e-volta, que é a metade que apanha uma tradução partida só de um lado.
    for i in 0..u8::try_from(ids::INSP_WATCH_CMP_OPT.len()).unwrap() {
        assert_eq!(u8_de_compare(compare_de_u8(i)), i, "a volta perdeu o {i}");
    }
}

/// ⭐⭐⭐ **E o SINAL que o chip mostra é o da comparação que ele escolhe.**
///
/// ⚠️ **Este é o gate que o irmão acima NÃO podia ser.** Ele afirma que a POSIÇÃO `1` do chip
/// significa `AtLeast`; esta afirma que a posição `1` **desenha `≥`**. Trocar os dois glifos deixa
/// a ordem intacta, a ida-e-volta intacta, os cinco gates de costura intactos — e entrega ao
/// artista uma regra que faz o CONTRÁRIO do que o botão diz.
///
/// ⛔ **Ele mora na shell porque é o único sítio onde o enum do motor e o painel se veem** — a
/// `ph2d-panel-inspector` fala `u8` de propósito, e a `ph2d-ecs` não conhece painel nenhum.
///
/// **Mutação que deve sangrar:** trocar dois braços de `simbolo_da_comparacao`.
#[test]
fn o_sinal_do_chip_e_o_da_comparacao_que_ele_escolhe() {
    use ph2d_app_components::counter_watch_inspector::u8_de_compare;
    use ph2d_ecs::Compare;
    use ph2d_panel_inspector::simbolo_da_comparacao;

    // ⛔ Nomeados um a um, e não por um laço sobre um `ALL`: o que se afirma é o VALOR de cada
    //    célula, e um laço derivado da mesma fonte concorda consigo mesmo.
    for (compare, sinal) in [
        (Compare::AtMost, "\u{2264}"),  // ≤
        (Compare::AtLeast, "\u{2265}"), // ≥
        (Compare::Exactly, "="),
    ] {
        assert_eq!(
            simbolo_da_comparacao(u8_de_compare(compare)),
            sinal,
            "o chip desenha o sinal errado para {compare:?} — a regra diz o contrário do botão"
        );
    }
}
