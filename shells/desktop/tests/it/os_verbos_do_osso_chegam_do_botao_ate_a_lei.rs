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

/// A fase que aplica os verbos de PELE (prender, assar, soltar, e os dois da pose de repouso).
const PELE: &str = include_str!("../../src/render_loop/fase_skeleton_verbs.rs");

/// ⭐⭐⭐ **O CENSO: TODO verbo da secção deixa RASTO na shell** — a metade que faltava, e que uma
/// mutação sobrevivente encomendou.
///
/// ⛔⛔⛔ **MEDIDO em 2026-09-19:** apagado o corpo do braço do *Add Smart Bone* na fase do quadro,
/// **`23` testes da shell ficaram verdes**. O censo da família prova que a PORTA faz efeito
/// (`ph2d_app_skeleton::verbos::censo_dos_verbos_do_osso_tests`); a costura do painel prova que o
/// clique chega ao BARRAMENTO; *nada juntava as duas pontas.* É o terceiro elo do `CLAUDE.md` §5.0 —
/// **o leitor decide, ou entrega a alguém que descarta?** — e a quarta vez que esta rota morre nesta
/// linha.
///
/// ⚠️⚠️ **Ele mede TEXTO e não uma chamada**, e a limitação é declarada: as fases são métodos de
/// `App`, que segura uma surface de janela real, logo nenhum teste as corre. *Ele apanha o braço que
/// deixou de chamar a porta; o braço que a chama com o argumento errado é apanhado do outro lado* —
/// pelo censo da família, que corre as duas portas e exige que elas **difiram**.
///
/// ⭐⭐ **A população é DERIVADA** ([`ph2d_app_skeleton::verbos::VerboDoOsso::TODOS`], guardada por um
/// `match` exaustivo): um verbo novo **não compila** até alguém dizer qual é o rasto dele. *É a
/// diferença entre uma lista que alguém tem de se lembrar de estender e uma que não fica verde sem a
/// extensão.*
#[test]
fn todo_verbo_do_osso_deixa_rasto_na_shell() {
    use ph2d_app_skeleton::verbos::VerboDoOsso;

    let texto = format!("{CLICKS}{FASE}{KNOBS}{PELE}");
    // ⛔ **O CONTROLO POSITIVO, e sem ele o censo é vácuo:** se o `include_str!` apontasse para um
    // ficheiro que encolheu, ou se a busca não funcionasse, os catorze liam-se vivos para sempre.
    assert!(
        !texto.contains("espelho::espelha_o_ramo_todo("),
        "a busca deste censo acusa como PRESENTE uma agulha que nao existe — ela nao mede nada"
    );
    assert!(
        texto.len() > 40_000,
        "as fases do osso encolheram para {} bytes: o `include_str!` esta' a ler outra coisa, e um \
         censo sobre um texto vazio fica verde sobre tudo",
        texto.len()
    );

    let mut mudos = Vec::new();
    let mut sem_rasto_declarado = Vec::new();
    for v in VerboDoOsso::TODOS {
        // ⛔⛔ **O PISO É POR VERBO e não a soma — foi uma MUTAÇÃO que o exigiu.** A 1.ª redacção
        // contava as agulhas todas contra o número de verbos, e como quatro deles declaram DUAS
        // (a porta partilhada mais o discriminador) a soma sobrava: esvaziar um verbo inteiro
        // deixava `16 >= 14` e o censo verde. *Uma lista vazia le'-se exactamente como aprovada.*
        if v.rastos_na_shell().is_empty() {
            sem_rasto_declarado.push(v);
        }
        for agulha in v.rastos_na_shell() {
            if !texto.contains(agulha) {
                mudos.push((v, *agulha));
            }
        }
    }
    assert!(
        sem_rasto_declarado.is_empty(),
        "estes verbos nao declaram rasto NENHUM: {sem_rasto_declarado:?} — eles saem da populacao \
         do censo em silencio, que e' a forma como um verbo morto passa despercebido"
    );
    assert!(
        mudos.is_empty(),
        "estes verbos do osso nao deixam rasto nenhum nas fases do quadro: {mudos:?} — o botao \
         pinta, acende sob o rato, o clique atravessa o painel, e o braco que o recebe nao chama \
         porta nenhuma"
    );
}
