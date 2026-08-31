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
pub(crate) fn cena_15() -> Result<FieldDoc, ph2d_field::FieldError> {
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
pub(crate) fn cena_16() -> Result<FieldDoc, ph2d_field::FieldError> {
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
pub(crate) fn cena_17() -> Result<FieldDoc, ph2d_field::FieldError> {
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
