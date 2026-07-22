//! **O onion fill (fatia C3) alcança os quadros selecionados — e o ajuste ao vivo também.**
//!
//! O rabisco do Colorize é autorado em MUNDO por cima das poses empilhadas: com chaves
//! marcadas na tira, o MESMO traço tem de colorir todas. Isso é uma propriedade do
//! `flip_colorize_apply`, e **nenhum teste de unidade a alcança**: a função precisa de
//! `gfx` (a janela + a GPU + o `FlipDoc` que mora dentro do `AppGfx`), e `App::new()` nasce
//! headless de propósito. É a mesma cerca que fez o `Join` da física virar arch-gate
//! (`selection_gestures_are_not_fanned_out`) e a ordem do frame virar
//! `the_z_projection_reads_the_tree_after_the_sync`.
//!
//! O gate lê o FONTE, e o que ele pina é a **forma**: o Apply pergunta os alvos ao
//! `flip_multiframe::targets` e os entrega ao `colorize_frames`; e o re-Apply ao vivo
//! percorre **todos** os quadros que o gesto escreveu.
//!
//! ⚠️ **Um gate de fonte pina forma, não comportamento** — uma mutação que preserva o texto e
//! neutraliza o laço (`.take(0)`) passa por ele. Foi medido, e por isso o laço foi EXTRAÍDO
//! para `colorize_frames`, que roda sobre um `FlipDoc` montado headless: quem prova que os
//! quadros de fato ganham cor é `the_fan_out_writes_a_region_into_every_frame_it_is_given`
//! (mutação `take(0)` sangra lá). Aqui fica só a metade que nenhum teste alcança — **quem
//! PERGUNTA à tira**.

const SRC: &str = include_str!("../src/flip_colorize.rs");

/// O corpo do `flip_colorize_apply`, do `pub(crate) fn` até o fecho da função seguinte.
fn apply_body() -> &'static str {
    let start = SRC
        .find("pub(crate) fn flip_colorize_apply(&mut self)")
        .expect(
            "o `flip_colorize_apply` foi renomeado — este gate aponta para nada e tem de ser \
         re-mirado",
        );
    let rest = &SRC[start..];
    let end = rest
        .find("\n    /// **Trap/Bleed em tempo real")
        .unwrap_or(rest.len());
    &rest[..end]
}

fn live_body() -> &'static str {
    let start = SRC
        .find("pub(crate) fn flip_colorize_live_adjust(&mut self)")
        .expect("o `flip_colorize_live_adjust` foi renomeado — re-mire o gate");
    &SRC[start..]
}

#[test]
fn the_apply_asks_the_strip_which_frames_the_gesture_writes() {
    let body = apply_body();
    assert!(
        body.contains("flip_multiframe::targets"),
        "o Apply tem de perguntar os alvos ao multiframe — sem isso o rabisco colore só o \
         quadro ativo e a fatia C3 não existe no produto"
    );
    assert!(
        body.contains("strip.selected_keys()"),
        "os alvos saem da SELEÇÃO da tira (é o gesto que o artista faz), não de uma \
         janela inventada aqui"
    );
    assert!(
        body.contains("selected_keys()"),
        "controle positivo: a fonte da seleção tem de aparecer"
    );
}

#[test]
fn the_targets_are_handed_to_the_fan_out_deduped() {
    let body = apply_body();
    // ⚠️ **A frase inteira, não só o nome da função.** Mencionar o `colorize_frames` e jogar
    // o resultado fora (`let _ = colorize_frames(…)`) passa por um gate que só procura o
    // nome — foi MEDIDO, sobreviveu, e este é o único gate que pode ver isso (o Apply não é
    // dirigível). O que se pina é a LIGAÇÃO: os quadros que o fan-out escreveu entram na
    // sessão viva, senão o Trap seguinte re-rodaria só no ativo e a tira mostraria dois
    // ajustes para uma operação só.
    assert!(
        body.contains("frames.extend(colorize_frames("),
        "os alvos que a tira devolveu têm de ser ENTREGUES ao fan-out E os quadros escritos \
         têm de entrar na sessão viva — perguntar e não usar é a feature morta da DIRETIVA §2"
    );
    assert!(
        body.contains("filter(|d| *d != did)"),
        "o alvo ativo já foi escrito acima: sem dedup ele levaria as regiões DUAS vezes"
    );
    // A `LiveFrame` do ativo é empilhada ANTES do fan-out, então a sessão viva sempre tem o
    // quadro que o artista está olhando na 1ª posição.
    let active = body
        .find("frames = vec![LiveFrame")
        .expect("o quadro ATIVO deixou de abrir a sessão viva");
    let fanout = body.find("colorize_frames(").expect("o fan-out sumiu");
    assert!(
        active < fanout,
        "o quadro ativo entra na sessão ANTES dos vizinhos (é a âncora do gesto)"
    );
}

#[test]
fn the_neighbours_never_speak_for_the_active_frame() {
    let body = apply_body();
    // A política herdada do balde (`09 §5.2`): o toast fala pelo quadro ATIVO — que é onde o
    // artista está olhando. Um vizinho que não fecha falha em silêncio, e a prova de que ele
    // volta INTOCADO é comportamental (`the_fan_out_writes_...`). O que se pina aqui é que
    // nenhum toast nasce DEPOIS do fan-out: um "não deu regiões" disparado por um vizinho
    // contradiria a tela, onde o quadro ativo está colorido.
    let fanout = body.find("colorize_frames(").expect("o fan-out sumiu");
    assert!(
        !body[fanout..].contains("Toast::"),
        "nenhum toast pode nascer do fan-out — um vizinho que não fecha falharia em voz \
         alta, contradizendo o quadro ativo que ESTÁ colorido na tela"
    );
}

#[test]
fn the_live_adjust_reruns_every_frame_the_gesture_wrote() {
    let body = live_body();
    assert!(
        body.contains("for f in &mut live.frames"),
        "o Trap/Bleed ao vivo tem de re-rodar em TODOS os quadros escritos — senão os \
         vizinhos ficam presos no Trap da 1ª rodada e a tira mostra dois ajustes para uma \
         operação só"
    );
    // O guard é perguntado a todos ANTES de escrever em qualquer um: meia re-aplicação
    // deixaria a tira inconsistente e não há como desfazê-la pela metade.
    let guard = body
        .find("let intact = ")
        .expect("o guard de segurança por-quadro sumiu");
    let write = body
        .find("for f in &mut live.frames")
        .expect("o laço de reescrita sumiu");
    assert!(
        guard < write,
        "o guard tem de rodar ANTES da 1ª escrita (é uma operação só: ou re-roda inteira, \
         ou não re-roda)"
    );
}

const SMOKE: &str = include_str!("../src/flip_colorize_smoke.rs");

/// 🔴 **A cena de smoke ARMA a seleção que a fatia precisa.**
///
/// A C3 só age com **2+ chaves marcadas** (`flip_multiframe::targets` devolve o caminho de
/// sempre abaixo disso), e marcar é Shift/Ctrl+clique célula a célula na tira. Uma cena que
/// não arma deixa a fatia atrás de um gesto que ela nem diz onde fica — e o Apply colore um
/// quadro só, **indistinguível da feature quebrada**. Foi exatamente essa a dúvida do 1º
/// smoke (*"não sei se você preparou corretamente o arquivo de teste"*), e a resposta certa
/// não é explicar melhor: é a cena nascer clicável (plano §8, ready-to-smoke).
///
/// E ela tem de **DIZER o que construiu**: *"o arquivo de teste está certo?"* é uma pergunta
/// que o smoke responde sozinho, senão um Apply de um quadro só é indistinguível de uma cena
/// com um quadro só.
#[test]
fn the_smoke_scene_arms_the_multiframe_selection_and_reports_it() {
    assert!(
        SMOKE.contains("self.flip_strip.selection.clone_from(&keys)"),
        "a cena tem de MARCAR as chaves na tira — sem isso a C3 fica atrás de um gesto \
         manual e o Apply colore um quadro só, indistinguível da feature quebrada"
    );
    assert!(
        SMOKE.contains("insert_frame(l, frame,"),
        "a cena tem de criar os quadros EXTRA (com o divisor deslocado) — uma chave só não \
         contém o fenômeno que a fatia existe para resolver"
    );
    assert!(
        SMOKE.contains("chave(s) em {keys:?}"),
        "a cena tem de IMPRIMIR quantas chaves montou e quantas marcou — o smoke responde \
         sozinho se o arquivo de teste está certo"
    );
}
