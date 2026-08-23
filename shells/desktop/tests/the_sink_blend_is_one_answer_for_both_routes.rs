//! **Arch-gate do modo de composição do sink** (doc 89, folha 17) — o `blend` do
//! `motion.output`.
//!
//! A shell é o ÚNICO lugar que enxerga as três peças ao mesmo tempo: o nó que
//! DECLARA o param, o substrato que o LÊ, e as duas rotas de render que o
//! consomem. `ph2d-node-motion-output` e `ph2d-eval-motion` são as duas folhas do
//! sistema de nós e nenhuma pode depender da outra, então a única coisa que impede
//! as duas de nomearem o param de maneiras diferentes é um gate daqui.
//!
//! Três maneiras de partir a wave deixando a suíte inteira verde:
//!
//! 1. **as duas strings divergem** — o painel escreve `"blend"`, o pump procura
//!    `"blend_mode"`, e o chip do artista fica **inerte**: não há erro, não há
//!    warning, e todo gate de unidade dos dois lados continua verde porque cada um
//!    usa a sua própria constante;
//! 2. **a rota da GPU deixa de perguntar** — o documento composita `Add` na CPU e
//!    `Normal` no device, e como o `PH2D_GPU_COOK` decide qual roda, o artista vê a
//!    feature funcionar e depois parar sem mexer em nada;
//! 3. **um segundo leitor** resolve o tag por conta própria (arredondando ou
//!    clampando diferente) e as duas rotas discordam num valor de fronteira.
//!
//! ⚠️ As asserções afirmam uma RELAÇÃO ou um CONTEÚDO, nunca uma distância em
//! bytes: esta casa já teve arch-gates apodrecerem por medirem bytes.

const GPU_BRIDGE: &str = include_str!("../src/render_loop/motion_bridge_gpu.rs");

/// **As duas folhas nomeiam o MESMO param.** O nó o declara, o substrato o
/// procura, e ninguém pode depender do outro para descobrir isso.
/// FALSIFICADO por qualquer renome de um lado só.
#[test]
fn the_node_and_the_substrate_agree_on_the_param_name() {
    assert_eq!(
        ph2d_node_motion_output::BLEND_PARAM,
        ph2d_eval_motion::SINK_BLEND_PARAM,
        "o nó e o substrato divergiram no nome do param de blend — o chip do \
         artista fica inerte, sem erro e com os dois lados verdes"
    );
}

/// **O param que o nó declara EXISTE no manifesto dele.** Um `ParamUiHint` para um
/// param que o `NodeManifest` não declara faz o `validate` recusar o grafo inteiro
/// — foi exatamente o que derrubou três demos na integração do `motion.color_ramp`.
#[test]
fn the_manifest_declares_the_param_the_hint_paints() {
    let declared = ph2d_node_motion_output::MANIFEST
        .params
        .iter()
        .any(|p| p.name == ph2d_node_motion_output::BLEND_PARAM);
    assert!(
        declared,
        "o `motion.output` pinta um chip de blend cujo param o manifesto não \
         declara — o `validate` recusaria o grafo inteiro"
    );
}

/// **Os rótulos cobrem exatamente os modos que o renderer sabe desenhar.** Menos
/// rótulos que pipelines deixa um modo INALCANÇÁVEL pelo menu (o censo do teto de
/// opções, doc 89 W3); mais rótulos que pipelines oferece um modo que o
/// `sink_blend_tag` clampa de volta — um item de menu morto.
#[test]
fn every_blend_the_renderer_can_draw_has_a_name_and_no_more() {
    assert_eq!(
        ph2d_node_motion_output::BLEND_LABELS.len(),
        ph2d_render::pipeline::BLEND_PIPELINE_COUNT,
        "os rótulos do chip e as pipelines do renderer discordam — ou um modo \
         é inalcançável, ou o menu oferece um que o clamp devolve"
    );
}

/// **A rota da GPU pergunta à MESMA porta.** Ela não pode derivar o tag sozinha: o
/// `ph2d-gpu-cook` mantém o `ph2d-eval-motion` como dev-dependency de propósito (o
/// motor de cook não depende do avaliador de que é o caminho rápido), então quem
/// resolve é a shell — e tem de ser por `sink_blend_tag`, do sink que o plano
/// escolheu. FALSIFICADO por um `0` cravado, ou por uma 2ª leitura do param.
#[test]
fn the_gpu_route_reads_the_tag_from_the_one_door() {
    assert!(
        GPU_BRIDGE.contains("ph2d_eval_motion::sink_blend_tag(&motion.doc.graph, motion.sinks[0])"),
        "a ponte da GPU deixou de perguntar à porta única — o mesmo documento \
         passa a compositar diferente conforme o `PH2D_GPU_COOK`"
    );
    // O tag tem de CHEGAR ao cook, em toda chamada. Três hoje (fully-GPU,
    // híbrida sequenciada, híbrida stateless) — a contagem é DERIVADA das
    // chamadas, não escrita à mão, senão uma rota nova nasce descoberta.
    let calls = GPU_BRIDGE.matches("motion.default_size,").count();
    let handed = GPU_BRIDGE.matches("blend,").count();
    assert!(calls > 0, "nenhuma chamada de cook encontrada — gate cego");
    assert_eq!(
        handed, calls,
        "{handed} de {calls} chamadas de cook recebem o blend — a que falta \
         composita em Normal em silêncio"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// **O OPERADOR POR-LINHA** (doc 89, folha 07 — o *Echo Operator* do AE, e o
// *Strobe Operator* que a mesma folha diz ser o mesmo conserto).
//
// Aqui há QUATRO peças que não podem ver-se umas às outras: o `motion.trail` e o
// `motion.strobe` (duas folhas irmãs, e um nó não depende de outro), o
// `motion.output` (que possui o vocabulário do sink) e o `ph2d-eval-motion` (que
// lê a coluna). A shell é o único sítio que enxerga as quatro.
// ─────────────────────────────────────────────────────────────────────────────

/// **OS TRÊS NÓS FALAM O MESMO VOCABULÁRIO.**
///
/// Os rótulos são copiados à mão em cada folha (nenhuma pode importar da outra), então a
/// única coisa que impede o `Screen` de um ser o `Multiply` do outro é isto. ⚠️ O do sink
/// não tem o `Sink` na frente — ele É o sink —, e é essa a única diferença permitida.
#[test]
fn the_row_operators_speak_the_sinks_vocabulary() {
    let sink = ph2d_node_motion_output::BLEND_LABELS;
    for (who, labels) in [
        ("trail", ph2d_node_motion_trail::ECHO_BLEND_LABELS),
        ("strobe", ph2d_node_motion_strobe::FLASH_BLEND_LABELS),
        // ⚠️ A SOMBRA entra aqui como UMA LINHA (doc 89 folha 11): ela é o terceiro nó a
        // escrever a coluna, e o vocabulário dela é copiado à mão como o dos outros dois.
        ("drop_shadow", ph2d_node_fx_drop_shadow::SHADOW_BLEND_LABELS),
    ] {
        assert_eq!(
            labels[0], "Sink",
            "{who}: o indice 0 da coluna quer dizer *o modo do sink*, e o rotulo tem de o dizer"
        );
        assert_eq!(
            &labels[1..],
            &sink[..],
            "{who}: os modos divergiram do vocabulario do sink — o artista escolhe um nome \
             e recebe outro modo, com os dois lados verdes"
        );
    }
}

/// **OS DOIS NÓS ESCREVEM A MESMA COLUNA** — senão um deles seria simplesmente ignorado
/// pelo lowering, sem erro nenhum.
#[test]
fn both_nodes_write_the_column_the_lowering_reads() {
    assert_eq!(
        ph2d_node_motion_trail::BLEND_COLUMN,
        ph2d_node_motion_strobe::BLEND_COLUMN
    );
    // ⚠️ E a TERCEIRA, que é o `fx.drop_shadow` (doc 89 folha 11): um nome diferente aqui
    // faria o modo da sombra ser escrito numa coluna que o lowering não lê — o fantasma
    // pintaria no modo do sink e o dropdown pareceria morto.
    assert_eq!(
        ph2d_node_motion_trail::BLEND_COLUMN,
        ph2d_node_fx_drop_shadow::BLEND_COLUMN
    );
    // E ela é o mesmo nome do param do sink, de propósito: é a mesma grandeza, e um
    // `value.attribute` a jusante lê o que o rastro escolheu.
    assert_eq!(
        ph2d_node_motion_trail::BLEND_COLUMN,
        ph2d_node_motion_output::BLEND_PARAM
    );
}

/// **O VOCABULÁRIO DOS NÓS CABE NOS PIPELINES DO RENDERER.**
///
/// ⚠️ Um nó é uma FOLHA e não alcança o renderer, então ele clampa pela própria lista; se
/// essa lista crescesse além dos pipelines, o último modo do dropdown seria escolhível e
/// silenciosamente rebaixado no lowering. É a metade que nenhum dos dois lados vê.
#[test]
fn no_node_offers_a_mode_the_renderer_cannot_draw() {
    let pipelines = ph2d_render::pipeline::BLEND_PIPELINE_COUNT;
    assert_eq!(
        ph2d_node_motion_output::BLEND_LABELS.len(),
        pipelines,
        "o vocabulario do sink e o array de pipelines tem de ser a mesma lista"
    );
    for (who, labels) in [
        ("trail", ph2d_node_motion_trail::ECHO_BLEND_LABELS.len()),
        ("strobe", ph2d_node_motion_strobe::FLASH_BLEND_LABELS.len()),
        (
            "drop_shadow",
            ph2d_node_fx_drop_shadow::SHADOW_BLEND_LABELS.len(),
        ),
    ] {
        assert_eq!(
            labels - 1,
            pipelines,
            "{who}: os modos (fora o `Sink`) tem de ser exactamente os pipelines"
        );
    }
}
