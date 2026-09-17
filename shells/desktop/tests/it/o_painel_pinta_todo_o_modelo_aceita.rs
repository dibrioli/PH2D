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
