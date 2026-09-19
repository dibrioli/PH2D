//! ⭐⭐⭐ **TODO CONTROLO DA SEÇÃO SKELETON DECLARA DE QUEM É O SUJEITO DELE.**
//!
//! ⛔⛔ **Achado da auditoria de 2026-09-08.** O braço da shell que diz *«nenhum osso em foco»*
//! existe porque *um verbo que morre em SILÊNCIO dá o mesmo sintoma que uma rota cortada* — foi
//! esse, letra por letra, o report de 2026-09-07 (*«Add IK não funciona»*), cuja causa era outra.
//!
//! ⚠️ **E ele era uma disjunção escrita à mão:** nasceu com **dois** verbos (os da âncora), tinha
//! **oito** quando a auditoria o apanhou, e os **nove campos** e as **duas fileiras de chips** nunca
//! lá entraram. *Uma cura escrita para os verbos que existiam não segue os que vêm.*
//!
//! ⇒ hoje a pergunta é **derivada** ([`ph2d_editor_core::ids::needs_focused_bone`]) e a única lista
//! à mão que resta é a das excepções. Este gate é o censo que a mantém honesta: cada id da seção cai
//! de **um** dos dois lados, e nenhum cai dos dois.

use ph2d_editor_core::ids;

/** A população: as QUATRO tabelas que a seção Skeleton tem. ⚠️ Um id da seção que não esteja em
nenhuma delas não é «esquecido por este gate» — ele já está morto para o encaminhamento, e é o
`table_driven_chips_are_registered_too` que o apanha. */
fn toda_a_seccao() -> Vec<ph2d_a11y::NodeId> {
    let mut v: Vec<ph2d_a11y::NodeId> = Vec::new();
    v.extend(ids::VECTOR_BONE_VERBS);
    v.extend(ids::VECTOR_BONE_FIELDS);
    v.extend(ids::VECTOR_BONE_BEND_IDS);
    v.extend(ids::VECTOR_BONE_SMART_CLIP_IDS);
    // ⛔⛔ **TRÊS tabelas da seção faltavam a esta população, e o `needs_focused_bone` já as lia.**
    // A ausência era MUDA nos dois gates deste ficheiro: a cobertura media uma seção mais pequena
    // do que a real, e a lista de excepções passou a acusar um FANTASMA no dia em que um controlo
    // de selecção nasceu numa delas (a fileira `Deform By`, 2026-09-19). *Um censo cuja população
    // é escrita à mão mede o que alguém se lembrou de escrever.*
    v.extend(ids::VECTOR_BONE_HANDLES_IDS);
    v.extend(ids::VECTOR_BONE_TIP_IDS);
    v.extend(ids::VECTOR_BONE_SKIN_LAW_IDS);
    v
}

/// ⭐⭐⭐ **A PERGUNTA DERIVADA COBRE A SEÇÃO INTEIRA** — nenhum controlo cai fora dos dois lados.
///
/// ⚠️⚠️ **PONTO CEGO NOMEADO, achado pela prova de mutação desta wave:** tirar o *Bind* da lista de
/// excepções **SOBREVIVE** a este gate. Ele fica a contar como *«precisa de osso em foco»*, o que é
/// falso, e nada aqui o pode saber: um id é um hash, e *de quem é o sujeito* só se lê no DRENO, que
/// vive na shell. ⇒ o que este gate mede é a **cobertura** da derivação — que foi exactamente a
/// falha real (a condição escrita à mão cobria `8` de `30` controlos) —, e não a correcção de cada
/// lado. *Um gate que se diz partição e mede cobertura mente sobre a metade que ele não vê.*
#[test]
fn the_derived_question_covers_the_whole_section() {
    for id in toda_a_seccao() {
        let osso = ids::needs_focused_bone(id);
        let formas = ids::VECTOR_BONE_ON_SELECTION.contains(&id);
        assert!(
            osso != formas,
            "o controlo {id:?} da seccao Skeleton caiu FORA dos dois lados (osso em foco: {osso}, \
             seleccao de formas: {formas}) -- com os dois falsos ele morre CALADO na janela «nenhum \
             osso em foco», que e' indistinguivel de uma rota cortada"
        );
    }
}

/// ⛔ **A lista de excepções não nomeia fantasmas.** Um id que saia das tabelas e fique aqui faria a
/// excepção descrever um controlo que já não existe — a forma como uma catraca vira licença.
#[test]
fn the_selection_exceptions_are_all_real_controls() {
    let secao = toda_a_seccao();
    for id in ids::VECTOR_BONE_ON_SELECTION {
        assert!(
            secao.contains(&id),
            "{id:?} esta' declarado como «age sobre as formas» e nao pertence a tabela nenhuma da \
             seccao -- a excepcao descreve um controlo que ja' nao existe"
        );
    }
}

/// ⭐ **E o de FORA da seção não é reclamado por ela** — a pergunta responde `false` a um id
/// qualquer, senão ela transformaria todo clique do painel num pedido de osso em foco.
#[test]
fn a_control_from_another_section_is_not_claimed() {
    for id in [
        ids::VECTOR_COMPOUND_MAKE,
        ids::VECTOR_SNAP_ON,
        ids::VECTOR_SECTION_BONE,
        ids::VECTOR_MODE_BONE,
    ] {
        assert!(
            !ids::needs_focused_bone(id),
            "{id:?} nao e' um controlo da seccao Skeleton e a pergunta reclamou-o -- todo clique do \
             painel passaria a pedir um osso em foco"
        );
    }
}
