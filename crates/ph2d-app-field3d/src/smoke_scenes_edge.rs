//! ⭐⭐ **AS CENAS DOS DOIS RECUOS DE UMA ARESTA** (Enio, 2026-08-30) — o chanfro, a costura entre
//! cópias, e o prisma do report.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::field3d_smoke_scenes`] é o **roteador**; estas três são um assunto fechado dentro dele
//! (*«o que os dois recuos fazem, e onde eles falharam»*), e o arquivo passou as `600` linhas do
//! gate de LOC do shell. ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ **O gate que o apanhou vive em `shells/desktop/tests/`** — e o `cargo test --bins` **não lhe
//! toca**. É a mesma cegueira que o §5 do `CLAUDE.md` já nomeia.

// ⚠️ Módulo-filho do roteador: o `use super::*` traz os construtores (`leaf`, `combine`) que ele
// já tem, e que continuam a existir **uma vez**.
use super::*;

/// A cena `=15` — ver o roteador.
pub fn cena_15() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 15 — O CHANFRO (Enio, 30/08): caixa VIVA · CHANFRADA · \
                 chanfrada e depois FILETADA. As três medem o mesmo; só a aresta muda."
    );
    // ⚠️ **Três caixas IGUAIS**, pela lei da cena 14: uma aresta mostrada sozinha não diz se
    // ela foi chanfrada — diz que a forma é assim. A da esquerda é a régua.
    //
    // ⭐ O recuo é `0,10` numa caixa de meia-extensão `0,34`: quase um terço da face, que é
    // onde o corte a 45° se lê de longe sem esconder a forma.
    let caixa = |x: f32, chamfer: f32, round: f32| {
        leaf(
            Primitive::Box {
                half: [0.34, 0.34, 0.34],
                round,
                chamfer,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            caixa(-0.85, 0.0, 0.0),
            caixa(0.0, 0.10, 0.0),
            // ⭐⭐ **A TERCEIRA é o pedido inteiro** — *«chamfer antes de fillet para a
            // possibilidade de arredondar as bordas geradas por chamfer»*. O corte a 45°
            // cria duas arestas novas por quina, e o arco de `0,03` come as duas.
            caixa(0.85, 0.10, 0.03),
            combine(
                Op::Union(Blend::Sharp),
                vec![NodeId(0), NodeId(1), NodeId(2)],
            ),
        ],
        NodeId(3),
    )
}

/// A cena `=16` — ver o roteador.
pub fn cena_16() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 16 — A COSTURA ENTRE AS CÓPIAS (Enio, 30/08): coroa de 8 tubos, \
                 costura VIVA · FILETADA · CHANFRADA"
    );
    // ⚠️ **Os tubos TÊM de se cruzar**, senão não há vinco para costurar: com o braço a
    // `0,30` e o raio a `0,17`, os centros de duas cópias vizinhas ficam a `0,23` e as
    // secções sobrepõem-se — é a mesma fixtura da foto do Enio, uma coroa de tubos.
    //
    // ⚠️ **A forma tem de estar fora do eixo NO ESPAÇO DO MODIFICADOR**, e a pilha corre
    // ANTES da pose do nó: é por isso que a coroa vive no GRUPO e o tubo é filho posado
    // dele. Pôr a pose no próprio nó-folha repetiria um cilindro centrado — invariante à
    // rotação — e a cena mostraria um tubo só.
    let anel = |x: f32, joint: ph2d_field::Joint, filho: u32| {
        let mut g = ph2d_field::Node::new(
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
            ph2d_field::NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![NodeId(filho)],
            },
        );
        g.mods = vec![ph2d_field::Unary::Radial {
            count: 8,
            joint,
            axis: ph2d_field::mods::RADIAL_AXIS,
        }];
        g
    };
    let tubo = || {
        leaf(
            Primitive::Cylinder {
                radius: 0.17,
                half_height: 0.30,
                round: 0.03,
                chamfer: 0.0,
            },
            Xform {
                translation: [0.30, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            tubo(),
            anel(-0.95, ph2d_field::Joint::SHARP, 0),
            tubo(),
            anel(
                0.0,
                ph2d_field::Joint {
                    chamfer: 0.0,
                    fillet: 0.09,
                },
                2,
            ),
            tubo(),
            // ⭐ O chanfro morde `1,71×` o que o filete morde com o mesmo número — é a
            // FORMA dele, medida em `the_four_characters`. Aqui os dois levam `0,09` de
            // propósito: é a diferença de carácter que a cena mostra, não a de tamanho.
            anel(
                0.95,
                ph2d_field::Joint {
                    chamfer: 0.09,
                    fillet: 0.0,
                },
                4,
            ),
            combine(
                Op::Union(Blend::Sharp),
                vec![NodeId(1), NodeId(3), NodeId(5)],
            ),
        ],
        NodeId(6),
    )
}

/// A cena `=17` — ver o roteador.
pub fn cena_17() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 17 — O PRISMA (report do Enio, 30/08): viva · CHANFRADA · \
                 chanfrada e FILETADA. Gire a câmera: nenhuma aresta pode mudar de aspecto."
    );
    // ⛔⛔ **Ela existe por um report com duas metades, e as duas eram defeitos diferentes**
    // (*«algumas arestas não receberam o fillet e ao rotacionar a aparência da aresta
    // muda»*):
    //
    // 1. as quinas **LATERAIS** de um prisma fecham num sítio do código e o **aro** noutro,
    //    e o chanfro tinha sido ligado só ao segundo — ⇒ a sonda por PONTO
    //    (`the_chamfer_reaches_every_edge_of_every_shape`), que também apanhou a engrenagem;
    // 2. a composição chanfro-e-filete **misturava duas vezes**, e cada nível encaixado soma
    //    um quadrado na lei de Cauchy–Schwarz: medido `passo × ‖∇f‖ = 1,4061` num prisma —
    //    acima de `1` a marcha atravessa a superfície, e o ponto em que ela pára passa a
    //    depender da direcção do raio. *É literalmente isso que «muda ao rotacionar» é.*
    //
    // ⚠️ **Hexagonal de propósito**: num prisma de seis lados as quinas laterais são doze e
    // ficam todas à vista de uma volta de câmera. Num cubo elas confundem-se com o aro.
    let prisma = |x: f32, chamfer: f32, round: f32| {
        leaf(
            Primitive::Prism {
                sides: 6,
                bottom: 0.36,
                top: 0.36,
                half_height: 0.42,
                round,
                chamfer,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            prisma(-0.9, 0.0, 0.0),
            prisma(0.0, 0.09, 0.0),
            prisma(0.9, 0.09, 0.03),
            combine(
                Op::Union(Blend::Sharp),
                vec![NodeId(0), NodeId(1), NodeId(2)],
            ),
        ],
        NodeId(3),
    )
}

/// ⭐⭐⭐ **AS CINCO JUNTAS NOVAS, lado a lado sobre a MESMA peça** (W145, pedido do Enio de 09/09).
///
/// # Por que um SALIENTE SOBRE UMA CHAPA, e não duas caixas a cruzar
///
/// A costura de um saliente sobre uma chapa é um **anel fechado**, e é a figura em que estas juntas
/// existem para trabalhar: um cordão de solda corre à volta da base, uma linha de painel contorna-a,
/// um friso reforça-a. ⚠️ **Duas caixas a cruzar dariam uma costura RECTA**, e uma decoração recta
/// lê-se como um bisel — *a metade que interessa é a costura VIRAR, e só uma costura fechada a
/// mostra*.
///
/// ⭐ E as seis peças são a **mesma geometria**: o que muda de uma para a outra é uma palavra.
pub fn cena_32() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 32 — AS JUNTAS NOVAS, da esquerda para a direita: \
         Fillet (referência) · Soft · Bead · Groove · Ridge · Chamfer desigual"
    );
    println!(
        "[field-smoke]            a peça é a MESMA nas seis; o que muda é o carácter da junta \
         entre o saliente e a chapa."
    );
    const PASSO: f32 = 0.62;
    const R: f32 = 0.06;
    // ⚠️⚠️⚠️ **A PENETRAÇÃO É LOAD-BEARING, e foi um report que o ensinou** (Enio, 09/09): o
    // saliente entra `0,09` numa chapa de `0,18`, e é essa sobreposição que a guarda do sulco
    // protege. Com uma penetração RASA (a 1.ª versão desta cena entrava `0,04`) o vinco fica a menos
    // de um raio de canal de toda a face de baixo do saliente, e nem a guarda o salva.
    //
    // ⚠️⚠️⚠️ **E A ESPESSURA TEM UM NÚMERO DERIVADO, não escolhido:** o sulco localiza-se por
    // `‖(a,b)‖`, e entre duas faces PARALELAS sem relação (a base do saliente e o fundo da chapa) o
    // mínimo dessa distância é `h/√2`, onde `h` é o vão entre elas. ⇒ para o canal não morder ali é
    // preciso **`h > R·√2`** — aqui `h = 0,12` contra `R·√2 = 0,085`. *Com `h = 0,08` ele mordia, e
    // foi o gate da perfuração que o disse.*
    //
    // ⚠️ A espessura também importa pelo óbvio: o sulco escava `R` a partir da superfície, e numa
    // chapa fina ele **perfura** — o que se veria como um rasgo à volta da base e se leria como
    // um defeito da peça, não como a feição. Com `0,14` de espessura e `R = 0,06` sobram `0,08`, e o
    // gate `the_new_junctions_scene_does_not_perforate_the_plate` mede-o.
    //
    // ⚠️ **E o saliente ENTRA na chapa** (`z` de `−0,04` a `0,44`) em vez de pousar nela: duas faces
    // exactamente coincidentes são o caso degenerado de toda booleana, e a costura de uma união
    // assim é uma REGIÃO em vez de uma curva.
    let chapa = |x: f32| {
        leaf(
            Primitive::Box {
                half: [0.26, 0.26, 0.11],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, -0.11],
                ..Xform::IDENTITY
            },
        )
    };
    let saliente = |x: f32| {
        leaf(
            Primitive::Box {
                half: [0.12, 0.12, 0.28],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.18],
                ..Xform::IDENTITY
            },
        )
    };
    let juntas = [
        Blend::Exact { radius: R },
        Blend::Soft { radius: R },
        Blend::Bead { radius: R },
        Blend::Groove { radius: R },
        Blend::Ridge {
            radius: R,
            width: R * Blend::SEAM_WIDTH_RATIO,
        },
        Blend::Bevel {
            radius: R,
            bias: 3.0,
        },
    ];
    let mut nodes = Vec::new();
    let mut grupos = Vec::new();
    for (i, b) in juntas.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = (i as f32 - 2.5) * PASSO;
        let base = NodeId(nodes.len() as u32);
        nodes.push(chapa(x));
        nodes.push(saliente(x));
        grupos.push(NodeId(nodes.len() as u32));
        nodes.push(combine(Op::Union(b), vec![base, NodeId(base.0 + 1)]));
    }
    // ⚠️ **O topo é `Sharp`**, e é load-bearing: com uma mistura aqui as seis peças derretiam umas
    // nas outras e nenhuma das seis leituras seria a da junta que ela nomeia.
    let raiz = NodeId(nodes.len() as u32);
    nodes.push(combine(Op::Union(Blend::Sharp), grupos));
    FieldDoc::new(nodes, raiz)
}
