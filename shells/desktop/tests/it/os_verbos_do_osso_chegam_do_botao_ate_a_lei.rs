//! ⭐⭐⭐ **OS VERBOS DO OSSO CHEGAM DO BOTÃO ATÉ À LEI** — o *Look At*, o desvio e o *Mirror Branch*,
//! cada um com o passo que esta casa já viu morrer.
//!
//! ⚠️ **A lei já tem os gates dela** (sete, em `ph2d_app_skeleton::goal::sonda_do_apontar_tests`, e
//! dois na costura do painel). O que só aqui se pode afirmar é o FIO: um id que o `fase_bus_clicks`
//! não conhece morre dentro do painel; um verbo que nenhuma fase aplica acende e não faz nada; e um
//! campo cujo dreno não escreve no componente aceita teclas e não fala com ninguém.
//!
//! ⚠️ **A agulha vive NESTE ficheiro e o sujeito nos outros** — um `include_str!` cuja agulha está
//! escrita dentro do ficheiro que ele lê conta-se a si mesmo e não afirma nada.

const CLICKS: &str = include_str!("../../src/render_loop/fase_bus_clicks.rs");
const CAMPOS: &str = include_str!("../../src/render_loop/fase_bus_tool_panel.rs");
const FASE: &str = include_str!("../../src/render_loop/fase_bone_ik_and_limits.rs");
const KNOBS: &str = include_str!("../../src/render_loop/fase_bone_smart_and_knobs.rs");
const ESPELHO: &str = include_str!("../../src/render_loop/fase_selection_mirror_bone_focus.rs");

/// ⭐ **O botão é encaminhado, e a fase aplica-o pela porta que nasce a apontar.**
///
/// ⛔ `goal::add` **não** serve aqui: ele dá a corrente de fábrica (`2`), e o verbo seria um *Add
/// IK* com outro rótulo.
#[test]
fn o_botao_look_at_chega_a_porta_que_nasce_a_apontar() {
    assert!(
        CLICKS.contains("VECTOR_BONE_LOOK_AT"),
        "o botao Look At nao e' encaminhado: ele pinta, acende sob o rato e o clique morre dentro \
         do painel"
    );
    assert!(
        FASE.contains("goal::add_look_at("),
        "a fase da ancora deixou de chamar a porta do apontar: o botao acende e nao faz nada"
    );
}

/// ⭐⭐ **O campo do desvio chega ao componente, e a conversão de unidade acontece UMA vez.**
///
/// ⚠️ **GRAUS na tela e RADIANOS no documento** — a mesma lei do limite da junta. ⛔ Duas conversões
/// (ou nenhuma) é como um número passa a significar outra coisa sem ninguém dar por isso.
#[test]
fn o_campo_do_desvio_chega_ao_componente_em_radianos() {
    assert!(
        CAMPOS.contains("VECTOR_BONE_IK_OFFSET"),
        "o campo do desvio nao e' encaminhado: ele aceita teclas e nao fala com ninguem"
    );
    assert!(
        KNOBS.contains("IkKnob::Offset => g.offset = v.to_radians()"),
        "o dreno do desvio nao escreve no componente (ou nao converte de graus): o campo mostra um \
         numero e o osso nao roda, ou roda 57 vezes mais"
    );
}

/// ⭐⭐⭐ **A LENTE DO PAINEL LÊ A MESMA PORTA QUE O SOLVER.**
///
/// ⛔⛔ *Com duas respostas à mesma pergunta, o painel promete um controlo que o solver não lê no
/// primeiro ajuste* — e o report chega como *«mexo no Aim Offset e não acontece nada»*. A porta é a
/// `goal::aponta`, e o espelho tem de a chamar em vez de reconstruir a conta.
#[test]
fn o_espelho_pergunta_a_porta_e_nao_reconstroi_a_conta() {
    assert!(
        ESPELHO.contains("goal::aponta(sim, e)"),
        "o espelho do painel deixou de perguntar a` porta do apontar: a lente do painel e o solver \
         passam a poder discordar"
    );
    assert!(
        ESPELHO.contains("set_current_bone_aim("),
        "a lente do apontar nao e' publicada: o painel nunca esconde os dois knobs inertes nem \
         pinta o desvio"
    );
    // ⚠️ **E ela publica GRAUS**, que é o que o campo mostra — sem isto o artista lê `0,52` onde
    // escreveu `30`.
    assert!(
        ESPELHO.contains("g.offset.to_degrees()"),
        "o desvio e' publicado em radianos: o campo mostraria 0,52 onde o artista escreveu 30"
    );
}

/// ⭐⭐⭐ **E O ESPELHO CHEGA DO BOTÃO ATÉ À LEI** — o mesmo fio, com o terceiro passo a ser o
/// REGISTO.
///
/// ⚠️ **A cópia profunda só sabe copiar o que o registo descreve**, logo passar-lhe outra coisa (ou
/// não lhe passar nada) faria o ramo espelhado perder o limite de ângulo, a curvatura e o repouso —
/// e o artista veria uma cópia *quase* igual, que é pior que uma que falha.
#[test]
fn o_espelho_chega_do_botao_ate_a_lei_com_o_registo() {
    assert!(
        CLICKS.contains("VECTOR_BONE_MIRROR"),
        "o botao Mirror Branch nao e' encaminhado: ele pinta, acende e o clique morre no painel"
    );
    assert!(
        FASE.contains("espelho::espelha(sim, component_registry, osso)"),
        "a fase deixou de chamar a lei do espelho com o REGISTO: a copia perde os componentes que \
         esta shell nao conhece, em silencio"
    );
}
