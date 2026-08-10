//! **Arch-gate da fronteira `pulse.* → ph2d-runtime`** — as três coisas que decidem se o grafo
//! grita a coisa certa, e que **nenhum teste de unidade alcança**: elas moram no
//! `motion_bridge::dispatch`, que toma o `HeroScreen`, o `SimWorld` e o device vivos.
//!
//! O que os gates de unidade ao lado já provam, e portanto o que este NÃO repete: *o que* uma
//! tomada lê (`signals::tests`, que dirigem o pump pela porta do produto) e *qual* é a pergunta
//! do relógio (`clock_forward::tests`). O que sobra é **onde** a pergunta é feita e **quando** a
//! leitura acontece — duas propriedades de POSIÇÃO, e posição é o que um teste de unidade não vê.
//!
//! ⚠️ **Cada asserção é uma PROPRIEDADE, nunca uma distância em bytes.** A `line/Vector` fechou
//! uma jornada com dois arch-gates vermelhos por terem sido escritos como *"a menos de 400 bytes
//! de"*: o proxy expira quando alguém acrescenta uma linha no meio, e o produto continua certo.

const MOTION: &str = include_str!("../src/render_loop/motion_bridge.rs");
const TIMELINE: &str = include_str!("../src/render_loop/timeline_bridge.rs");

/// **Controle positivo.** Um `include_str!` apontando para um arquivo que um corte de LOC
/// esvaziou deixaria toda busca abaixo devolver *"não achei"* — e o gate falaria com confiança
/// sobre um texto que não existe.
fn sources() -> (&'static str, &'static str) {
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
    (MOTION, TIMELINE)
}

/// **Os DOIS emissores fazem a MESMA pergunta, e a fazem à mesma função.**
///
/// Escrever a condição duas vezes é como uma delas ganha um caso especial e a outra não. A
/// pergunta não é de nenhum dos dois — é do relógio —, e por isso ela mora num terceiro arquivo.
#[test]
fn os_dois_emissores_perguntam_a_mesma_lei() {
    let (motion, timeline) = sources();
    for (quem, src) in [("motion", motion), ("timeline", timeline)] {
        assert!(
            src.contains("clock_forward::clock_is_playing_forward("),
            "o emissor de {quem} tem de perguntar a lei ÚNICA, não re-escrever a condição"
        );
    }
    // E o SÍTIO DA DECISÃO não a re-deriva: `is_advancing_forward` sozinho é METADE da lei
    // (falta o `jumped`), e é essa metade que transforma um seek para a frente numa metralhadora.
    //
    // ⚠️ **A busca é escopada ao corpo do emissor, e não ao arquivo** — a primeira versão deste
    // gate proibia o nome no `timeline_bridge` inteiro e REPROVOU sobre produto correto: o
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

/// **A leitura das tomadas acontece DENTRO do laço de tiques.**
///
/// O `tap_streams` é limpo a cada cook, então ler depois do laço deixaria só o último tique — e
/// um quadro lento deve dois ou três. A perda seria **silenciosa**, que é a classe de defeito
/// que esta casa mais paga.
///
/// A propriedade é afirmada pelo que separa os dois pontos no texto: entre o `for` e a chamada
/// não pode haver o fecho do laço. ⚠️ Não é uma distância — é a AUSÊNCIA de uma saída.
#[test]
fn a_tomada_e_lida_dentro_do_laco_de_tiques() {
    let (motion, _) = sources();
    let laco = motion
        .find("for tick in ticks_owed(")
        .expect("o dispatch percorre os tiques devidos");
    let leitura = motion[laco..]
        .find("collect_signals(motion, tick)")
        .map(|i| laco + i)
        .expect("o dispatch lê as tomadas depois de abrir o laço");
    let entre = &motion[laco..leitura];
    assert!(
        entre.contains("advance_or_scrub_with_taps_scoped"),
        "a leitura vem depois do cook do tique, não antes dele"
    );
    // O fecho do laço está na indentação de 4; o corpo dele, na de 8. Encontrar `\n    }` antes
    // da leitura é a prova textual de que ela saiu para fora.
    assert!(
        !entre.contains("\n    }"),
        "`collect_signals` caiu para FORA do laço — só o último tique de um quadro lento gritaria"
    );
}

/// **A publicação é gateada pela lei, e o gate é lido de uma variável — não de uma segunda
/// cópia da condição.**
#[test]
fn a_publicacao_e_gateada_pela_lei() {
    let (motion, _) = sources();
    let armado = motion
        .find("let armed = super::clock_forward::clock_is_playing_forward(")
        .expect("a lei é perguntada UMA vez, antes do laço");
    let laco = motion
        .find("for tick in ticks_owed(")
        .expect("o dispatch percorre os tiques devidos");
    assert!(
        armado < laco,
        "a lei é perguntada ANTES do laço: ela é do quadro, não do tique"
    );
    let leitura = motion[laco..]
        .find("collect_signals(motion, tick)")
        .map(|i| laco + i)
        .expect("o dispatch lê as tomadas");
    assert!(
        motion[laco..leitura].contains("if armed"),
        "a leitura tem de estar sob o gate da lei — sem ele, arrastar a régua grita a cada tique"
    );
}
