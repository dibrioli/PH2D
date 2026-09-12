//! **O teclado da cena 3D só é dela enquanto ela está EM USO** — e nunca quando um campo
//! tem o foco (report do Enio, 2026-08-17: *"depois de abrir outros módulos como Sculpt, o
//! módulo Motion nodes não consegue usar os atalhos do teclado e nem mesmo digitar um número
//! no painel motion"*).
//!
//! ## Por que isto é um arch-gate e não um teste de comportamento
//!
//! A porta é `App::sculpt3d_key`, e para ela consumir alguma coisa é preciso uma
//! `Sculpt3dScene` — que precisa de um `wgpu::Device`. Headless o `App` nasce sem `gfx`, então
//! todo braço devolve `false` e um teste de comportamento seria **verde por vácuo**: ele passa
//! igualzinho com o bug de volta. O que se pode afirmar sem janela é a **ORDEM DAS GUARDAS**
//! dentro da função, e é ela que carrega a cura.
//!
//! ## O defeito, medido
//!
//! O portão perguntava `sculpt3d_scene_mut().is_some()` — *"existe uma cena?"* —, e o próprio
//! módulo do teclado escreve o preço: *"com uma cena armada este teclado consome quase toda
//! letra (os dez dígitos são verbos, `G`/`H`/`T`/`S`/`A` são verbos, …)"*. Como **sair do modo
//! nunca destrói a cena** (é decisão declarada do `sculpt3d_mode`), o primeiro clique no pill
//! armava aquele portão **para o resto da sessão**, e ele corre ANTES do `handler.on_key` que
//! alimenta o store — então o painel do Motion não via nem atalho nem texto.
//!
//! Medido no catálogo do grafo, o que morria era exatamente o conjunto **nu** (`F` Fit · `A`
//! Add · `H` Bypass · `K` Knife · `P` Probe) mais os dez dígitos; os `Ctrl+…` do Motion já
//! passavam, porque o braço de `ctrl` desta cena só reclama o `Ctrl+Z`.
//!
//! ⚠️ **A causa é uma assimetria entre duas portas que respondem à MESMA pergunta:** o ponteiro
//! da cena já cedia (ele pergunta `FormRole::draws_clay`), o teclado não. As duas perguntam o
//! mesmo agora.

use std::fs;

const KEYS: &str = "../../crates/ph2d-app-sculpt3d/src/keys.rs";

fn keys_src() -> String {
    fs::read_to_string(KEYS).unwrap_or_else(|e| panic!("não consegui ler {KEYS}: {e}"))
}

/// Onde a âncora aparece — com **controle positivo**: uma âncora que sumiu vira falha alta,
/// nunca uma varredura vazia que passa por acidente.
fn at(src: &str, needle: &str) -> usize {
    src.find(needle)
        .unwrap_or_else(|| panic!("âncora sumiu de {KEYS}: {needle:?} — o gate mede outra coisa"))
}

/// **Um campo focado é dono do teclado, e a pergunta vem ANTES de tudo.**
///
/// É a metade GERAL da cura: os dez dígitos são verbos desta cena, então sem esta guarda um
/// chip numérico focado em QUALQUER painel do app não recebe um dígito.
#[test]
fn a_focused_field_owns_the_keyboard_before_any_branch() {
    let src = keys_src();
    let guard = at(&src, "factos.text_focused");
    // A primeira coisa que a função pode CONSUMIR é o bake (`Shift+B`); a guarda tem de vir
    // antes dela, senão a metade que ela protege depende de qual tecla foi apertada.
    // ⚠️ **`sculpt3d_req.bake_request` desde 2026-09-11 (W2/L3-A2)** — os cinco pedidos da
    // escultura passaram a um `Sculpt3dRequests`. O `at()` faz `expect`, logo este gate
    // reprova ALTO quando o nome muda, em vez de deixar de medir a ordem em silêncio.
    let first_consumer = at(&src, "req.bake_request = true");
    assert!(
        guard < first_consumer,
        "a guarda do campo focado corre DEPOIS de um braço que já consome — digitar num painel \
         volta a disparar um gesto de escultura"
    );
}

/// **A ESCULTURA exige o barro na tela — a mesma pergunta que o ponteiro faz.**
///
/// A guarda tem de preceder o empréstimo da cena, que é onde mora o teclado inteiro dos verbos,
/// da máscara, da topologia, do espelho e da luz.
#[test]
fn the_sculpture_keys_require_the_clay_to_be_on_screen() {
    let src = keys_src();
    let guard = at(&src, "if !keys_live");
    let borrow = at(&src, "fn key(");
    // ⚠️⚠️ **O SUJEITO desta ordem mudou em 2026-09-11 (W2/L3-B), e a propriedade é a mesma.**
    // Ela media *o guarda vem antes do EMPRÉSTIMO da cena* (`self.sculpt3d_scene_mut()`), porque
    // era esse empréstimo que abria a parte reivindicadora do teclado. Hoje a cena chega por
    // parâmetro e não há empréstimo nenhum — o que o guarda protege é o **corpo que reivindica
    // teclas**, e o primeiro item dele é a delegação ao teclado da câmara.
    //
    // ⇒ o gate mede o que a lei diz: *nada que reivindique uma tecla corre antes de o barro estar
    // na tela*. ⛔ Medir a posição de `fn key(` seria medir o começo da função, que está antes de
    // tudo por construção — a asserção passaria a ser trivialmente falsa (e foi).
    let reivindica = at(&src, "keys_view::camera_key(");
    assert!(
        guard < reivindica,
        "o teclado da escultura é reivindicado sem o barro na tela — é o bug do report: abrir o \
         Sculpt uma vez cala os atalhos de todo painel para o resto da sessão"
    );
    assert!(
        borrow < guard,
        "controle positivo: a função tem de começar antes do guarda dela"
    );
}

/// **O interruptor da doação (`D`) SOBREVIVE fora do barro — senão `FormRole::Off` fica
/// inalcançável.**
///
/// O pill é binário por desenho (de qualquer papel ele ENTRA no barro), então o `D` é o único
/// caminho até `Off`. Debaixo da guarda do barro o ciclo daria `Clay --D--> Light` e parava ali.
///
/// ⚠️ Esta é a metade que impede a cura de trocar um bug por uma regressão.
#[test]
fn the_donation_switch_survives_outside_the_clay() {
    let src = keys_src();
    let cycle = at(&src, "cycle_role()");
    let guard = at(&src, "if !keys_live");
    assert!(
        cycle < guard,
        "o ciclo de papel caiu debaixo da guarda do barro — `FormRole::Off` fica inalcançável, \
         porque o pill só sabe ENTRAR no barro"
    );
    // E ele continua atrás da guarda geral: digitar `d` num campo não pode virar um gesto.
    assert!(
        at(&src, "factos.text_focused") < cycle,
        "o `D` do papel corre antes da guarda do campo focado — digitar `d` num painel cicla a \
         doação"
    );
}

/// **Uma FERRAMENTA em mãos ganha as teclas nuas — mesmo com o barro na tela.**
///
/// Este é o segundo caso do report, e o barro sozinho não o cobre: pegar o Motion no rail não
/// tira o barro da tela, então sem esta metade os atalhos do grafo continuariam mudos
/// exatamente no gesto que o Enio descreveu (*"logo que ele for aberto"*).
#[test]
fn a_tool_in_hand_wins_the_bare_keys() {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    // ⚠️ **A PORTA MUDOU DE CASA em 2026-09-04, e o gate seguiu-a** — as três perguntas saíram
    // para a `sculpt3d_keys_dead_reason`, que devolve **por que** as teclas estão mortas (o
    // report *«corrija o deletar com a tecla del»*: a tecla morria num guarda e nenhum dizia
    // nada). O `sculpt3d_keys_live` passou a DERIVAR dela. *Um censo de fonte tem de saber a
    // forma do que conta* — e a lei que este gate defende é a mesma, uma casa adiante.
    assert!(
        src.contains("self.sculpt3d_keys_dead_reason().is_empty()"),
        "o `sculpt3d_keys_live` tem de DERIVAR da razao -- duas copias das tres perguntas \
         divergem no dia do quarto guarda"
    );
    let live = src
        .find("fn sculpt3d_keys_dead_reason")
        .expect("a porta sumiu — o gate mede outra coisa");
    let body_end = src[live..]
        .find("\n    }")
        .map(|e| live + e)
        .expect("corpo da porta");
    let body = &src[live..body_end];
    assert!(
        body.contains("a_tool_owns_the_bare_keys()"),
        "a porta não pergunta se uma ferramenta está em mãos — com o barro na tela o Motion \
         volta a ficar mudo, que é metade do report"
    );
    assert!(
        body.contains("sculpt3d_clay_on_screen()") && body.contains("text_entry_focused()"),
        "controle: a porta perdeu uma das outras duas metades"
    );
}

/// **O `Shift+D` (duplicar peça) continua sendo da ESCULTURA.**
///
/// Controle do hoist: ele foi feito com `!shift` justamente para não roubar o irmão, e sem esta
/// asserção um `D` hoistado sem a condição passaria nos três testes acima enquanto engolia o
/// duplicar.
#[test]
fn the_hoisted_role_key_does_not_swallow_shift_d() {
    let src = keys_src();
    let hoisted = at(&src, "if code == K::KeyD && !ctrl && !shift");
    // ⚠️⚠️ **A ordem que este gate mede ATRAVESSA DOIS ficheiros desde 2026-09-10**, quando o
    // `sculpt3d/keys.rs` ficou vermelho no teto de LOC por ACUMULAÇÃO de duas linhas e os verbos da
    // LISTA saíram para o irmão. ⛔ **Concatenar os dois textos e comparar posições seria FRAUDE:**
    // tudo o que está no filho aparece depois de tudo o que está no pai, então a asserção passaria
    // por construção, sobre qualquer ordem.
    //
    // ⭐ A propriedade parte-se em duas metades que falham por motivos DIFERENTES, e juntas dizem o
    // mesmo que a comparação antiga dizia — mais o nome da costura:
    //   (a) no PAI, o braço hoistado precede a CHAMADA que despacha o `Shift`;
    //   (b) no FILHO, o `Shift+D` é quem duplica.
    // Se alguém subir o bloco do `shift` para cima do `D` hoistado, (a) reprova; se o duplicar
    // desaparecer do filho, (b) reprova com o controlo positivo do `at`.
    let despacho = at(&src, "sculpt3d_shift_verbs(scene, code)");
    assert!(
        hoisted < despacho,
        "controle: o braço hoistado deixou de preceder o despacho do Shift — a ordem que o gate \
         mede mudou"
    );
    assert!(
        src[hoisted..despacho].contains("cycle_role()"),
        "o `D` hoistado não é o do ciclo de papel"
    );
    let filho = std::fs::read_to_string("../../crates/ph2d-app-sculpt3d/src/keys_scene.rs")
        .expect("os verbos da LISTA vivem no irmão desde 2026-09-10");
    assert!(
        filho.contains("if code == K::KeyD") && filho.contains("scene.duplicate_active()"),
        "o `Shift+D` deixou de duplicar no ficheiro dos verbos da LISTA — a outra metade da ordem"
    );
}
