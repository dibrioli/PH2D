//! A cena do **PORTÃO ESPACIAL** (`PH2D_GPU_COOK_DEMO=23`) — o `SUPERAR:` item 3 da folha
//! 12 do doc 89, montado como documento pronto para smoke.
//!
//! Irmã das cenas de campo ao lado, e a pergunta que ela responde é outra: as `=17..=22`
//! mostram **o que um campo desenha**; esta mostra **um campo decidindo QUEM ouve um
//! evento**. É a combinação que nenhuma referência tem — o C4D tem Fields e zero eventos, o
//! Niagara tem eventos e zero campos componíveis —, e até 2026-08-09 nós tínhamos os dois e
//! eles não se encontravam: um pulso não virava número (`pulse.level`) e o peso de um campo
//! não era legível (o canal **Falloff** do `value.attribute`).
//!
//! **A cadeia inteira, e cada elo é um dos dois:**
//! `pulse.beat → pulse.level → value.math(Multiply, Falloff) → pulse.compare → pulse.counter
//! → motion.drive(Size)`.
//!
//! ⚠️ **O CAMPO É UM RAMO LATERAL, e essa é a decisão que torna a cena honesta.** O
//! `motion.drive` **lê a coluna `falloff`** como máscara de força própria (`gpu.rs`, o
//! fallback é 1.0 quando a coluna está ausente), então bastaria pôr o `field.box` no caminho
//! de INSTÂNCIAS para a cena mostrar o quadro certo **pelo motivo errado**: o crescimento
//! ficaria confinado à caixa mesmo com o portão de pulso removido. Aqui a caixa alimenta
//! **só** o `value.attribute`, o drive recebe o stream sem `falloff`, e a única coisa que
//! confina o pisca-pisca é o portão. O gate `the_gate_is_the_pulse_not_the_drives_own_mask`
//! pina exatamente isso.
//!
//! ⚠️ **Por que um CONTADOR no fim:** o `pulse.level` é MOMENTÂNEO por natureza — vale 1 no
//! tique em que o beat dispara e 0 nos outros —, então dirigir o Size direto daria um piscão
//! de **um quadro** (1/60 s), que é honesto e ilegível. O `pulse.counter(count_max = 2,
//! Wrap)` é o **toggle** que a wave mediu já existir (`tick mod 2` = 0,1,0,1): cada pulso
//! recebido inverte, e o ponto fica grande por meio período inteiro. ⚠️ Não use
//! `value.smooth` para isso — ele é um blur sobre a ORDEM das instâncias, não sobre o tempo.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Lado da grade — `SIDE²` pontos, a mesma densidade das cenas de campo vizinhas.
///
/// ⚠️ **O número saiu da MEDIÇÃO, não de um palpite** (§0), e o recurso que ele gasta é o
/// QUADRO: a cadeia é CPU-only por natureza (os seis `pulse.*`/`value.*` não têm kernel, e
/// não é uma omissão — um pulso é um evento por linha, não um mapa por texel). Medido pela
/// sonda `measure_the_gate_scene_tick`, em `--release`, que é o perfil do smoke:
///
/// | pontos | ms/tique | ns/ponto | do quadro de 60 fps |
/// |---|---|---|---|
/// | 40.000 | 0,503 | 12,6 | 3 % |
/// | **262.144** | **3,96** | **15,1** | **24 %** |
///
/// ⚠️ E o número de DEBUG é 11× pior (5,44 ms já a 40.000) — um lado a lado que só existe
/// porque medi no perfil errado primeiro.
pub(super) const SIDE: f32 = 512.0;
/// Passo da grade: `SIDE × GAP ≈ 12` unidades, a mesma moldura das cenas de campo — ela
/// ENQUADRA no zoom default em vez de ser uma parede de 200 unidades.
const GAP: f32 = 0.024;
/// Tamanho do ponto em repouso (< `GAP`, então os pontos ficam distintos em vez de ladrilhar
/// numa folha).
pub(super) const DOT: f32 = 0.018;
/// Quanto o Size CRESCE com o toggle ligado (`motion.drive` mode Add, valor 0/1 × isto).
const POP: f32 = 0.03;

/// **O PORTÃO ESPACIAL** (`PH2D_GPU_COOK_DEMO=23`): 262.144 pontos, um metrônomo, e um
/// losango de pontos que pisca no compasso enquanto o resto da grade fica parado.
///
/// O que o artista vê: um campo denso e imóvel; a cada meio segundo, **só os pontos dentro
/// da caixa girada** trocam de tamanho. Fora dela nada acontece, nunca — o beat chega a
/// todas as linhas e o campo decide quem o escuta.
pub(super) fn build_gpu_pulse_gate_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", GAP);
    g.set_param(grid, "gap_y", GAP);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", DOT);

    // O RAMO LATERAL: a caixa girada 45° é um LOSANGO, e um losango é a forma que não sai de
    // um campo ordinal nem de um `motion.falloff` radial — é espacial e tem orientação.
    let field = g.add_node("field.box");
    g.set_param(field, "width", 5.0);
    g.set_param(field, "height", 5.0);
    g.set_param(field, "soft", 0.6);
    g.set_param(field, "rotation", 45.0);
    // O canal que faltava: o peso que a caixa deixou, lido como valor.
    let mask = g.add_node("value.attribute");
    g.set_text_param(mask, "attr", "falloff");
    g.set_param(mask, "mode", 0.0);

    // O metrônomo: meio segundo, o compasso que se conta olhando.
    let beat = g.add_node("pulse.beat");
    g.set_param(beat, "period", 0.5);
    // A ponte que a wave construiu: o pulso vira um número que o domínio de valor lê.
    let level = g.add_node("pulse.level");
    // O PORTÃO: o nível × o peso do campo. Fora da caixa o produto é 0 e nenhuma borda sobe.
    let gate = g.add_node("value.math");
    g.set_param(gate, "op", 2.0); // Multiply
    // De volta ao domínio de pulso — o ida-e-volta é a identidade onde o peso é 1.
    let cmp = g.add_node("pulse.compare");
    g.set_param(cmp, "rise", 0.5);
    g.set_param(cmp, "fall", 0.25);
    // O toggle (ver o doc do módulo): `count_max = 2` + Wrap.
    let toggle = g.add_node("pulse.counter");
    g.set_param(toggle, "count_max", 2.0);
    g.set_param(toggle, "mode", 0.0); // Wrap

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 3.0); // Size
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", POP);
    let out = g.add_node("motion.output");

    for (i, n) in [grid, scale, field, mask, gate, cmp, toggle, drive, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 170.0,
                y: 200.0,
            },
        );
    }
    // O metrônomo e o nível moram acima da fileira: eles são a outra entrada do portão.
    for (i, n) in [beat, level].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 420.0 + i as f32 * 170.0,
                y: 40.0,
            },
        );
    }

    for (a, ap, b, bp) in [
        (grid, 0u16, scale, 0u16),
        // O ramo lateral do campo — NÃO passa pelo drive (ver o doc do módulo).
        (scale, 0, field, 0),
        (field, 0, mask, 0),
        // O metrônomo conta as linhas do mesmo stream, sem o `falloff`.
        (scale, 0, beat, 0),
        (beat, 0, level, 0),
        (level, 0, gate, 0),
        (mask, 0, gate, 1),
        (gate, 0, cmp, 0),
        (cmp, 0, toggle, 0),
        // O drive recebe a arte SEM máscara e o valor já gateado.
        (scale, 0, drive, 0),
        (toggle, 0, drive, 1),
        (drive, 0, out, 0),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .ok()?;
    }
    // Os três `pre` self-loops que a família de pulso carrega (memória de borda). O editor os
    // plumba ao SOLTAR um nó; um documento montado por `add_node` os escreve à mão.
    for (n, port) in [(beat, 1u16), (cmp, 1), (toggle, 1)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}
