//! **Os shaders do preview são compilados no BIND, não no primeiro traço** (doc 28 §4.8).
//!
//! ⚠️ Arquitetural porque o defeito é **invisível em comportamento**: a sessão GPU existe nos dois
//! casos e produz os mesmos pixels — o que muda é **QUANDO** ela é construída. Um gate de unidade não
//! pode ver isso, e um gate de perf não a alcança (a glue exige janela + adapter).
//!
//! O número que o justifica está medido em `ph2d-render/tests/measure_first_stroke_pipelines.rs`, com o
//! driver já quente: `LayerCompositor` **6,01 ms** + `ImpastoLightPass` **16,30** + `PreviewPremul`
//! **5,70** = **28,01 ms** de compilação de pipeline, que o `get_or_insert_with` do `drive` cobrava do
//! primeiro traço do artista — quase dois quadros de 60 fps, no gesto em que ele está esperando.

/// O `bind_document` da ponte tem de ser seguido do pré-aquecimento, **na mesma cadeia de guards**.
///
/// A afirmação é POSICIONAL de propósito: o que importa não é que a chamada exista em algum lugar do
/// arquivo, é que ela aconteça **onde o documento entra** — que é o vão humano entre escolher o sprite e
/// levar o mouse à tela.
#[test]
fn the_bind_that_hands_the_painter_a_document_also_warms_the_gpu_preview() {
    let src = include_str!("../src/render_loop/painter_bridge.rs");

    // Controle positivo: os dois alvos têm de EXISTIR. Sem isto o gate passa por não achar nada — a
    // falha que o arch-gate do Shape Flow pegou em si mesmo.
    let bind = src
        .find("painter.bind_document(")
        .expect("controle: a ponte tem de entregar um documento ao painter");
    let warm = src
        .find("painter_gpu_preview::prewarm(")
        .expect("controle: a ponte tem de pre-aquecer o preview GPU");

    assert!(
        warm > bind,
        "o pre-aquecimento tem de vir DEPOIS do bind (ele so faz sentido quando ha documento)"
    );
    // E dentro do MESMO bloco: entre os dois não pode haver um `}` que feche a cadeia de guards, senão
    // o pré-aquecimento roda noutra condição — por exemplo em todo frame, que é pior que o lazy.
    let between = &src[bind..warm];
    let opens = between.matches('{').count();
    let closes = between.matches('}').count();
    assert!(
        closes <= opens,
        "o pre-aquecimento saiu do bloco do bind ({closes} fechamentos contra {opens} aberturas entre \
         os dois) — ele tem de rodar na MESMA condicao, nao numa que o chame mais vezes"
    );
}

/// **E o pré-aquecimento tem de COZINHAR UM FRAME**, não só construir os pipelines.
///
/// ⚠️ Esta metade nasceu do smoke do Enio (*"quanto menor o IMG menor o atraso; 1024 nem se percebe"*),
/// que **refutou** a versão anterior deste gate: compilação de pipeline **independe do tamanho do
/// canvas**, então ela não pode explicar uma escada com a tela. O que escala são os RECURSOS — as
/// texturas dos passes nascem no tamanho do canvas — e eles só existem depois de um `drive`.
///
/// ⚠️ E o `drive` do pré-aquecimento **não pode passar pela `gpu_eligible`**: uma pilha recém-bindada é
/// trivial e sem relevo, então ela recusa — que é precisamente o motivo de o custo cair no primeiro
/// traço com relevo. O gate afirma as duas coisas: o pré-aquecimento chama `drive`, e **não** chama a
/// porta que o recusaria.
#[test]
fn the_prewarm_cooks_a_frame_and_does_not_ask_the_eligibility_gate() {
    let src = include_str!("../src/render_loop/painter_gpu_preview.rs");
    let warm = src
        .find("pub(crate) fn prewarm(")
        .expect("controle: o pre-aquecimento tem de existir");
    let body = &src[warm..];
    let end = body
        .find("\n}\n")
        .expect("controle: a funcao tem de terminar");
    let body = &body[..end];
    assert!(
        body.contains("drive("),
        "o pre-aquecimento tem de COZINHAR um frame (so construir os pipelines nao aloca as texturas, \
         que sao o custo que escala com a tela)"
    );
    // ⚠️ E tem de ALCANÇAR o `drive`. A asserção acima sozinha é satisfeita por codigo MORTO — um
    // `if true { return; }` acima dele deixa o texto no lugar e a chamada inalcancavel, e foi
    // exatamente essa mutacao que SOBREVIVEU a primeira versao deste gate. *"Contem a chamada"* nao e
    // *"a chamada roda"*.
    //
    // A regra estrutural: o pre-aquecimento pode desistir por DOIS guards documentados — a pilha que a
    // `flatten_for_gpu` recusa, e um canvas de tamanho zero — e por mais nenhum.
    let bails = body.matches("return;").count();
    assert_eq!(
        bails, 2,
        "o pre-aquecimento tem exatamente DOIS guards (flatten recusou · canvas 0x0) e achei {bails} \
         saidas — uma saida a mais torna o `drive` inalcancavel com o texto dele ainda no lugar"
    );
    assert!(
        !body.contains("gpu_eligible("),
        "o pre-aquecimento nao pode consultar a `gpu_eligible`: uma pilha recem-bindada e trivial e \
         sem relevo, entao ela RECUSA — que e exatamente por que o custo caia no primeiro traco"
    );
}

/// E o `drive` **mantém** o `get_or_insert_with`, que é o fallback.
///
/// ⚠️ Não é redundância: o pré-aquecimento é uma OTIMIZAÇÃO de quando, e o produto não pode depender
/// dela para funcionar. Uma rota que chegue ao `drive` sem passar pelo bind (um teste, um caminho
/// futuro) tem de continuar produzindo preview — só que pagando os 28 ms ali.
#[test]
fn the_drive_still_builds_the_session_when_nobody_warmed_it() {
    let src = include_str!("../src/render_loop/painter_gpu_preview.rs");
    assert!(
        src.contains("session_slot.get_or_insert_with(|| PainterGpuPreview::new("),
        "o `drive` tem de seguir construindo a sessao sozinho — o pre-aquecimento e sobre QUANDO, e o \
         produto nao pode depender dele para funcionar"
    );
}
