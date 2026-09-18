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

/// ⭐⭐⭐ **A LUZ QUE ATRAVESSA A PEÇA** (17/09, `docs/Render3d/10`) — a PAREDE FINA e a MACIÇA, lado
/// a lado, sobre a mesma forma.
///
/// # ⚠️ A cena é DUAS peças e não uma, e a razão é que são DOIS fenómenos
///
/// A subsuperfície do OpenPBR não é um grau de um efeito — é uma escolha entre dois:
///
/// | a peça | o que se vê |
/// |---|---|
/// | **parede fina** (uma folha, uma pétala, um abajur) | com a luz ATRÁS ela acende inteira |
/// | **maciça** (jade, cera, mármore fino) | a luz CONTORNA a quina e o terminador amacia |
///
/// ⛔ Uma cena com uma peça só ensinaria metade e deixaria a outra a parecer um knob sem efeito.
///
/// # ⚠️ A ESPESSURA da fina é load-bearing, e ela não entra na lei
///
/// A lei da parede fina não lê espessura nenhuma — ela é a lambertiana do lado de lá. Mas o que o
/// artista VÊ depende da peça parecer fina: a mesma lei numa bola maciça lê-se como *«a bola ficou
/// clara»*, e numa lâmina lê-se como *«a luz passa através»*. ⇒ a fina é uma **lâmina** de `0,03`
/// contra `0,90` de largura, que é a proporção de uma folha.
///
/// ⚠️ **E a maciça é uma esfera**, pela razão oposta: a lei dela mede a CURVATURA, e uma lâmina é
/// plana — ali o piso do GLSL entregaria o raio de `100` e a lei ficaria indistinguível de uma
/// difusa. *Cada metade tem a forma que a lei dela precisa.*
pub fn cena_33() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 33 — A LUZ QUE ATRAVESSA A PECA. A' esquerda uma LAMINA (a folha), a' \
         direita uma ESFERA (o jade). As duas abrem OPACAS de proposito."
    );
    println!(
        "[field-smoke]            (1) MODEL · Shading · Render — (2) escolha a LAMINA e suba \
         `Subsurface` ao maximo, e ligue `Thin Walled` — (3) arraste a LUZ para TRAS da peca: ela \
         acende inteira."
    );
    println!(
        "[field-smoke]            (4) escolha a ESFERA, suba `Subsurface`, deixe `Thin Walled` em \
         `Solid` e baixe `Subsurface Radius` — a luz contorna a quina em vez de parar nela."
    );
    let lamina = leaf(
        Primitive::Box {
            half: [0.45, 0.42, 0.015],
            round: 0.012,
            chamfer: 0.0,
        },
        Xform {
            translation: [-0.55, 0.0, 0.0],
            ..Xform::IDENTITY
        },
    );
    let bola = leaf(
        Primitive::Sphere { radius: 0.42 },
        Xform {
            translation: [0.55, 0.0, 0.0],
            ..Xform::IDENTITY
        },
    );
    FieldDoc::new(
        vec![
            lamina,
            bola,
            combine(
                Op::Union(ph2d_field::Blend::Sharp),
                vec![NodeId(0), NodeId(1)],
            ),
        ],
        NodeId(2),
    )
}
