//! ⚠️⚠️ **ESTE GATE VIVE NUM FICHEIRO IRMÃO, e a razão é MEDIDA.**
//!
//! Ele lê o `smoke_bone_envelope.rs` por `include_str!` e CONTA a agulha. ⛔ Escrito lá dentro, a
//! agulha do próprio teste entrava na conta — a 1.ª redacção reprovou exactamente assim (`2` contra
//! `1`) —, e a cura barata (subir o número esperado) tornaria o gate **inerte**: apagar a peça de
//! produto deixá-lo-ia verde.
//!
//! *Um `include_str!` que procura uma string escrita nele próprio não afirma nada.*

/// ⭐⭐ **A CENA TEM O PAR E O CONTROLO** — e as duas metades do report do dono.
///
/// ⚠️ **Por texto e não por comportamento, com a razão declarada:** montar a cena pede um
/// `VecScene` e o `vec_entities::sync`, que vivem no quadro. ⭐ E o `include_str!` deixa de
/// **compilar** se o ficheiro mudar de sítio, em vez de passar a varrer zero e ficar verde.
#[test]
fn a_cena_tem_o_par_aberto_e_o_controlo_fechado() {
    const FONTE: &str = include_str!("smoke_bone_envelope.rs");
    // ⚠️ **A agulha é a TUPLA do construtor**, não o nome da forma: a 1.ª redacção contava
    // `ShapeKind::Arc` e reprovou quando o traço passou a perguntar `if kind == ShapeKind::Arc` —
    // *uma agulha que apanha a PERGUNTA junto com a DECLARAÇÃO conta duas coisas diferentes.*
    assert_eq!(
        FONTE.matches("(ShapeKind::Arc, ").count(),
        1,
        "o par ABERTO deixou de ser declarado num sitio so': as duas cordas podem divergir, e a \
         cena passa a comparar dois desenhos em vez de dois alcances"
    );
    // ⛔⛔ **A PREMISSA ANTERIOR MORREU NA FOTO, e a morte fica visível no diff.** O controlo era
    // um `RoundRect`, e ele saía como uma **barra recta com um vinco**: a deformação vectorial
    // corre nos PONTOS DE CONTROLO, e oito pontos não desenham a curva de três ossos. *Uma peça de
    // controlo que se lê como partida ensina que o app está partido.*
    assert!(
        FONTE.contains("(ShapeKind::Segment, "),
        "o CONTROLO deixou de ser o MESMO arco fechado pela corda — ou ele sumiu (e nao ha' onde \
         ler que o gizmo desaparece, que e' metade do report do dono), ou voltou a ser uma forma \
         com poucos pontos, que a foto ja' reprovou"
    );
    assert!(
        FONTE.contains("if k == 1 { ALCANCE_FORTE } else { 1.0 }"),
        "o alcance deixou de ser escrito SO' na segunda corda: se as duas levarem o mesmo, a \
         cena mostra duas cordas iguais e nao responde a nada"
    );
}

/// ⭐⭐⭐ **E A SHELL APLICA-O MESMO** — sem esta metade a lei do prólogo afirma sobre código que
/// ninguém corre.
///
/// ⛔⛔ *Uma cena que declara o prólogo certo e uma ponte que nunca o consulta leem-se exactamente
/// igual num teste de unidade* — e a foto que reprovou a 1.ª redacção desta cena foi tirada com o
/// prólogo a existir só no papel.
///
/// ⚠️ **Por texto e com a razão declarada:** a ponte vive no prólogo do quadro e pede uma janela.
/// ⭐ E o `include_str!` deixa de **compilar** se o ficheiro mudar de sítio, em vez de passar a
/// varrer zero e ficar verde.
#[test]
fn a_shell_aplica_o_prologo_desta_cena() {
    const PONTE: &str = include_str!("../../../shells/desktop/src/vec_bone_smoke.rs");
    for (agulha, porque) in [
        (
            "smoke_bone_envelope::prologo_do_nivel(",
            "a ponte deixou de perguntar a' cena o que armar — a decisao voltou a viver nela",
        ),
        (
            "ph2d_app_vec::smoke_bone::nivel(),",
            "a ponte deixou de passar o NIVEL: ou ela arma a cena =1 (que o dono aprovou sem \
             prologo), ou deixa de armar a =2",
        ),
        (
            "set_panel_visible(",
            "a timeline deixou de ser fechada: com ela aberta o Frame All corta sempre, e a \
             terceira fileira volta a ficar debaixo da borda de baixo (medido na foto)",
        ),
        (
            "kind: ph2d_editor_core::ViewFocusKind::All",
            "o enquadramento saiu da ponte: a cena abre na camera de omissao, descentrada",
        ),
    ] {
        assert!(
            PONTE.contains(agulha),
            "{porque} (agulha ausente: {agulha:?})"
        );
    }
    // ⚠️⚠️ **A ORDEM é LEI e não conforto:** enquadrar com a timeline ainda aberta mede a janela
    // com um terço a menos de altura util — e é exactamente esse o corte que a foto apanhou.
    let fecha = PONTE.find("set_panel_visible(").expect("fecha");
    let enquadra = PONTE
        .find("kind: ph2d_editor_core::ViewFocusKind::All")
        .expect("enquadra");
    assert!(
        fecha < enquadra,
        "a ponte enquadra ANTES de fechar a timeline: o Frame All volta a medir a janela com um \
         terco a menos, e a cena volta a sair cortada"
    );
}

/// ⛔⛔⛔ **O LIMITE DESTE GATE ESTÁ DECLARADO, e ele saiu de uma mutação SOBREVIVENTE.**
///
/// A `V9` da prova de mutação trocou o guarda da ponte por `if false` e **nada acusou**: as agulhas
/// continuavam no texto. *Um gate que lê texto afirma que o código EXISTE, nunca que ele CORRE.*
///
/// ⭐ A cura que se pôde fazer foi **tirar o guarda**: a ponte passou a ter UMA chamada
/// incondicional e a inércia do `=1` virou uma lei PURA, que o
/// [`super::tests::o_prologo_fecha_a_timeline_e_enquadra`] mede pelos dois lados. ⇒ o que sobra por
/// medir é a ponte CORRER, e isso pede um arnês de quadro que esta crate não tem — **dívida
/// nomeada**, e não silêncio.
///
/// ⚠️ Este teste existe para a dívida ter endereço: ele falha se alguém puser um guarda de volta,
/// que é a forma como a `V9` renasceria.
#[test]
fn a_ponte_aplica_o_prologo_sem_guarda_de_nivel() {
    const PONTE: &str = include_str!("../../../shells/desktop/src/vec_bone_smoke.rs");
    assert!(
        !PONTE.contains("nivel() == 2"),
        "a ponte voltou a ter um guarda de nivel a' volta do prologo: a mutacao `V9` (troca-lo por \
         `if false`) volta a SOBREVIVER, e o prologo deixa de correr sem nada acusar"
    );
}
