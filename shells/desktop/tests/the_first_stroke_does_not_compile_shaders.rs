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
