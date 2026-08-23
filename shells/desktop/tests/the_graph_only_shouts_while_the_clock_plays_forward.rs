//! **Arch-gate da fronteira `pulse.* → ph2d-runtime`** — as coisas que decidem se o grafo grita
//! a coisa certa e que **nenhum teste de unidade alcança**: elas moram no laço de quadro do
//! shell, que exige o `HeroScreen`, o `SimWorld` e o device vivos.
//!
//! O que os gates de unidade ao lado já provam, e portanto o que este NÃO repete: *o que* uma
//! tomada lê e que **as duas rotas de cook gritam o mesmo** (`signals::tests`, que dirigem as
//! duas portas de marcha da bomba) e *qual* é a pergunta do relógio (`clock_forward::tests`).
//! O que sobra é **onde** a pergunta é feita e **quando** a leitura acontece — duas propriedades
//! de POSIÇÃO, e posição é o que um teste de unidade não vê.
//!
//! ⚠️ **A primeira versão deste arquivo afirmava a posição ERRADA, e passava.** Ela exigia que a
//! leitura estivesse *dentro do laço de tiques da bomba de CPU* — verdade sobre o código, e
//! inútil: a cena que o Enio smokou planeja **híbrida**, aquele laço não roda, e o produto ficou
//! mudo com o gate verde. *Um gate que afirma um endereço não sabe se alguém ainda mora lá.*
//! Hoje ele afirma a propriedade que sobrevive à rota: a leitura acontece onde TODAS as rotas
//! já marcharam.
//!
//! ⚠️ E cada asserção é uma PROPRIEDADE, nunca uma distância em bytes — o proxy expira quando
//! alguém acrescenta uma linha no meio, e o produto continua certo.

const LOOP: &str = include_str!("../src/render_loop/mod.rs");
const MOTION: &str = include_str!("../src/render_loop/motion_bridge.rs");
const TIMELINE: &str = include_str!("../src/render_loop/timeline_bridge.rs");

/// **Controle positivo.** Um `include_str!` apontando para um arquivo que um corte de LOC
/// esvaziou deixaria toda busca abaixo devolver *"não achei"* — e o gate falaria com confiança
/// sobre um texto que não existe.
fn sources() -> (&'static str, &'static str, &'static str) {
    assert!(
        LOOP.contains("motion_bridge::dispatch(") && LOOP.len() > 100_000,
        "o laço de quadro não foi lido: {} bytes",
        LOOP.len()
    );
    assert!(
        MOTION.contains("fn dispatch") && MOTION.len() > 10_000,
        "o motion_bridge não foi lido: {} bytes",
        MOTION.len()
    );
    assert!(
        TIMELINE.contains("fn emit") && TIMELINE.len() > 5_000,
        "o timeline_bridge não foi lido: {} bytes",
        TIMELINE.len()
    );
    (LOOP, MOTION, TIMELINE)
}

/// **Os DOIS emissores fazem a MESMA pergunta, e a fazem à mesma função.**
///
/// Escrever a condição duas vezes é como uma delas ganha um caso especial e a outra não. A
/// pergunta não é de nenhum dos dois — é do relógio —, e por isso ela mora num terceiro arquivo.
#[test]
fn os_dois_emissores_perguntam_a_mesma_lei() {
    let (r#loop, _, timeline) = sources();
    for (quem, src) in [("motion", r#loop), ("timeline", timeline)] {
        assert!(
            src.contains("clock_forward::clock_is_playing_forward("),
            "o emissor de {quem} tem de perguntar a lei ÚNICA, não re-escrever a condição"
        );
    }
    // E o SÍTIO DA DECISÃO não a re-deriva: `is_advancing_forward` sozinho é METADE da lei
    // (falta o `jumped`), e é essa metade que transforma um seek para a frente numa metralhadora.
    //
    // ⚠️ **A busca é escopada ao corpo do emissor, e não ao arquivo** — uma versão anterior
    // proibia o nome no `timeline_bridge` inteiro e REPROVOU sobre produto correto: o
    // `set_reverse_play` (o sentido do onion) é um segundo leitor legítimo do playhead, e uma
    // pergunta parecida não é a mesma pergunta.
    let emissor = {
        let a = timeline.find("fn emit(").expect("o emissor da timeline");
        let b = timeline[a..].find("\n    }").expect("o corpo dele fecha");
        &timeline[a..a + b]
    };
    assert!(
        !emissor.contains("is_advancing_forward"),
        "o emissor da timeline decidiu pelo playhead direto — a lei tem DUAS metades"
    );
}

/// **A leitura acontece onde TODA rota de cook já marchou** — depois do `dispatch` de Motion e
/// antes do dreno.
///
/// É a propriedade que a versão anterior deste gate não tinha: o `dispatch` tem **três** saídas
/// (a bomba de CPU, o prefixo da GPU híbrida, e o retorno antecipado de um quadro 100% GPU), e
/// uma leitura dentro de qualquer uma delas é uma leitura que as outras não fazem.
#[test]
fn a_leitura_acontece_depois_de_qualquer_rota_e_antes_do_dreno() {
    let (r#loop, motion, _) = sources();
    let despacho = r#loop
        .find("motion_bridge::dispatch(")
        .expect("o quadro despacha a ponte de Motion");
    let leitura = r#loop
        .find("motion_bridge::signals::collect_signals(motion)")
        .expect("o quadro lê o livro-razão das tomadas");
    let dreno = r#loop
        .find("for sig in motion.signals_out.drain(..)")
        .expect("o quadro drena o que o grafo gritou");
    assert!(
        despacho < leitura && leitura < dreno,
        "a ordem é despachar (marchar) -> ler -> drenar; medido {despacho}/{leitura}/{dreno}"
    );
    // E a ponte NÃO lê por conta própria: uma segunda leitura publicaria o mesmo grito duas
    // vezes, e nasceria justamente de alguém "consertando" uma rota isolada.
    assert!(
        !motion.contains("collect_signals("),
        "a leitura tem UMA casa — a ponte marcha, o quadro lê"
    );
}

/// **A publicação é gateada pela lei.**
#[test]
fn a_publicacao_e_gateada_pela_lei() {
    let (r#loop, _, _) = sources();
    let lei = r#loop
        .find("if clock_forward::clock_is_playing_forward(")
        .expect("a lei é perguntada antes de ler o livro-razão");
    let leitura = r#loop
        .find("motion_bridge::signals::collect_signals(motion)")
        .expect("o quadro lê o livro-razão");
    assert!(
        lei < leitura,
        "a leitura tem de estar sob o gate da lei — sem ele, arrastar a régua grita a cada tique"
    );
    assert!(
        r#loop[lei..leitura].len() < 200,
        "a lei gateia a LEITURA, não um bloco distante dela"
    );
}

/// **A ponte ARMA a bomba e limpa o livro-razão — as duas metades sem as quais não há nada a
/// ler.**
///
/// Sem o arm, nenhuma tomada é cozida por marcha nenhuma e o livro fica vazio para sempre; sem a
/// limpeza, o grito de ontem é republicado amanhã. As duas ficam juntas, no mesmo lugar onde o
/// quadro zera o que publica.
#[test]
fn a_ponte_arma_as_tomadas_e_zera_o_livro() {
    let (_, motion, _) = sources();
    assert!(
        motion.contains("motion.pump.set_taps("),
        "a tomada é ARMADA na bomba — é isso que a faz cavalgar qualquer rota de marcha"
    );
    // ⚠️ **E o que é armado tem de PARTIR das tomadas de sinal.** A lista deixou de ser
    // `signal_taps` directamente em 2026-08-23 (o gizmo de canvas dos deformadores de
    // quadrilátero precisa do stream que entra no nó seleccionado, e entra na mesma
    // lista), então a agulha literal antiga deixaria de casar sobre código correcto. O que
    // este gate protege continua a ser o mesmo: *as tomadas dos SINAIS estão lá dentro*.
    // Sem esta metade, alguém que substituísse a lista por outra coisa passaria.
    assert!(
        motion.contains("motion.signal_taps.clone()"),
        "e a lista armada PARTE das tomadas de sinal — um segundo pedido junta-se a elas, \
         nunca as substitui"
    );
    assert!(
        motion.contains("motion.pump.clear_tap_fires()"),
        "e o livro-razão é zerado por quadro, ao lado do que mais o quadro publica"
    );
}
