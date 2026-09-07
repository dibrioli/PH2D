//! ⭐ **O BORDO de cada forma** — a esfera que contém a primitiva inteira.
//!
//! # Por que ele saiu do [`super::radius_tables`]
//!
//! O irmão responde *que tamanho a forma tem* (a **menor** medida, que dá escala a uma mistura);
//! este responde *que esfera a contém* (a **maior**, que dá a caixa do extrator). ⚠️ São perguntas
//! **opostas**, e é isso que torna o corte natural — a [`bounding_radius`] erra sempre para CIMA de
//! propósito, e a [`super::radius_tables::characteristic_size`] para baixo.
//!
//! O ficheiro comum passou os **700** do gate de LOC quando a W131 lhe acrescentou o triângulo.
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! # ⚠️ E o corte curou um defeito que ele não ia procurar
//!
//! O `#[must_use]` e o doc grande da [`bounding_radius`] estavam **separados dela** pela
//! [`gear_planar_reach`], que nasceu no meio dos dois em 2026-08-31 — *um atributo separado do seu
//! item por um doc muda de dono*, então o `must_use` guardava a engrenagem e a `bounding_radius`
//! shipava **sem documentação nenhuma**. Aqui a ordem é a certa: cada doc encostado ao seu item.

use crate::Primitive;

/// ⭐⭐⭐ **ATÉ ONDE UMA ENGRENAGEM CHEGA NO PLANO** — e **não** é o `outer` (2026-08-31).
///
/// # ⛔⛔⛔ Ela CORTAVA a peça, e o defeito é irmão do arco preto da cruz
///
/// A ponta de um dente é uma **corda**, não um arco: os dois **cantos** dela ficam mais longe do
/// centro do que o meio. Medido por bissecção (raio `outer = 0,45`, `round = 0`):
///
/// | dentes | alcance planar real | `outer` | excesso |
/// |---:|---:|---:|---:|
/// | `3` | `0,5050` | `0,45` | **`12,2 %`** |
/// | `5` | `0,4684` | `0,45` | `4,1 %` |
/// | `7` | `0,4593` | `0,45` | `2,1 %` |
/// | `24` | `0,4508` | `0,45` | `0,2 %` |
///
/// ⛔ **E o [`bounding_radius`] usava `hyp(outer, half_height)`**, que só sobrevivia pela folga da
/// altura: numa engrenagem **chata** ela desaparece e a peça sai cortada em **8 de 9** configurações
/// medidas — a `3` dentes, por `9 %`. *É a mesma família do report do Enio de 30/08 (quatro setas
/// para arcos pretos numa cruz), e a mesma lição: o ponto mais afastado é o CANTO, não o meio.*
///
/// ⭐ A cerca é `outer / cos(π / 2n)` — o canto de uma corda que subtende meio passo angular. Ela
/// **majora** o medido em todos os casos (`0,5196` contra `0,5050` a três dentes), que é o lado
/// certo da assimetria desta tabela.
///
/// ⚠️ O [`crate::MIN_GEAR_TEETH`] é `3`, e o `max` aqui é a rede: `n = 1` daria `cos(π/2) = 0` e uma
/// divisão por zero. *Uma cerca que confia noutra cerca escreve a rede na mesma.*
#[must_use]
pub(crate) fn gear_planar_reach(teeth: u32, outer: f32) -> f32 {
    let n = teeth.max(crate::MIN_GEAR_TEETH);
    #[allow(clippy::cast_precision_loss)]
    let meio_passo = std::f32::consts::PI / (2.0 * n as f32);
    outer / meio_passo.cos()
}

/// ⭐ **O raio de uma esfera, centrada na origem local, que contém a primitiva INTEIRA.**
///
/// # Por que uma ESFERA, e não uma caixa
///
/// ⚠️ Uma esfera é **invariante à rotação**: subir a cadeia de poses custa `centro' = pose(centro)`
/// e `raio' = raio · escala`, sem inflar nada. Uma caixa teria de ser re-envolvida a cada nível
/// rodado — e cada re-envolvimento cresce, então uma peça com três agrupamentos girados acabaria com
/// uma caixa muito maior do que ela. *A moeda certa para compor bordos é a que a composição não
/// estraga.*
///
/// # ⚠️ Conservador é a direção SEGURA, e a assimetria é o critério
///
/// Este número decide a caixa da grade do extrator ([`ph2d_field_eval::extract`]). Um bordo **maior**
/// do que a peça custa **resolução**; um bordo **menor** **CORTA a peça** e não diz nada. Toda
/// aproximação aqui erra para cima, de propósito.
///
/// ⚠️ O arredondamento de uma caixa/cilindro **não cresce** o bordo: a lei encolhe a fonte e
/// re-cresce por fora, então a extensão externa continua a ser a que o artista digitou.
#[must_use]
pub fn bounding_radius(p: &Primitive) -> f32 {
    let hyp = |a: f32, b: f32| a.hypot(b);
    match p {
        Primitive::Box { half, .. } => {
            (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt()
        }
        Primitive::Sphere { radius } => *radius,
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => hyp(*radius, *half_height),
        // O tubo mais afastado do centro está a `major + minor`.
        Primitive::Torus { major, minor } => major + minor,
        // ⚠️ O polígono responde como a extrusão: o contorno dá as duas do plano, a
        // altura dá a terceira. *Mesma superfície, mesma caixa.*
        Primitive::Extrude {
            profile,
            half_height,
            ..
        }
        | Primitive::Polygon {
            profile,
            half_height,
            ..
        } => {
            let (min, max) = profile.bounds();
            let r = hyp(
                min[0].abs().max(max[0].abs()),
                min[1].abs().max(max[1].abs()),
            );
            hyp(r, *half_height)
        }
        // ⚠️ O torno gira em torno de **Y**: o raio do sólido é o maior `|x|` do contorno, e a altura
        // é o maior `|y|`.
        Primitive::Revolve { profile } => {
            let (min, max) = profile.bounds();
            hyp(
                min[0].abs().max(max[0].abs()),
                min[1].abs().max(max[1].abs()),
            )
        }
        // O ponto mais afastado é uma das duas quinas do aro — a maior das duas.
        Primitive::Cone {
            bottom,
            top,
            half_height,
            ..
        } => hyp(bottom.max(*top), *half_height),
        // ⚠️ **`half_height + radius`, e não a hipotenusa**: a ponta da cápsula está no EIXO, a
        // `h + r` do centro, e ela é o ponto mais afastado. Uma hipotenusa daria `√(h²+r²)`, que é
        // MENOR — e um raio de contenção pequeno demais corta a peça na caixa do mundo.
        Primitive::Capsule {
            radius,
            half_height,
        } => half_height + radius,
        // ⚠️ O `radius` de um prisma é o CIRCUNRAIO (a quina), então ele já é a distância máxima no
        // plano — nenhum `cos` entra aqui.
        Primitive::Prism {
            bottom,
            top,
            half_height,
            ..
        } => hyp(bottom.max(*top), *half_height),
        // A cunha cabe na caixa de que ela é uma metade.
        Primitive::Wedge { half, .. } => {
            (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt()
        }
        // ⚠️ **Um ARCO cabe no toro inteiro**, e é o bordo honesto: apertá-lo pelo sector exigiria
        // a caixa de um sector de anel, e um bordo menor **corta a peça** sem dizer nada.
        Primitive::TorusArc { major, minor, .. } => major + minor,
        // A ponta é o ponto mais afastado no plano, e ela está a `outer` do eixo.
        Primitive::Star {
            outer, half_height, ..
        } => hyp(*outer, *half_height),
        // A gaiola cabe na caixa de que ela é o esqueleto.
        Primitive::BoxFrame { half, .. } => {
            (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt()
        }
        // ⚠️ **O MAIOR semi-eixo** — o menor daria uma esfera que corta a peça nos outros dois, e a
        // assimetria desta função é a lei (errar para cima custa resolução, errar para baixo corta).
        Primitive::Ellipsoid { radii } => radii[0].max(radii[1]).max(radii[2]),
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Erra para CIMA, sempre** — um bordo maior custa resolucao, um bordo menor CORTA a
        // peca e nao diz nada (a assimetria escrita no doc desta funcao).
        Primitive::Octahedron { radius, .. } => *radius,
        // A ponta mais afastada esta' no EIXO, a `h + r` — como na capsula, e nao a hipotenusa.
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => half_height + bottom.max(*top),
        Primitive::CutSphere { radius, .. } => *radius,
        Primitive::HollowDome {
            radius, thickness, ..
        } => radius + thickness * 0.5,
        // O tubo mais afastado esta' a `length + major + minor` na diagonal do estadio.
        Primitive::Link {
            major,
            minor,
            length,
        } => hyp(major + minor, length + major + minor),
        Primitive::SolidAngle { radius, .. } => *radius,
        // ⛔ **O CANTO da ponta do dente, e não o `outer`** — ver [`gear_planar_reach`], e a peça
        // que era cortada em 8 de 9 configurações medidas.
        Primitive::Gear {
            teeth,
            outer,
            half_height,
            ..
        } => hyp(gear_planar_reach(*teeth, *outer), *half_height),
        // ⛔⛔⛔ **A LARGURA DO BRAÇO ENTRA, e não entrava** (report do Enio, 30/08, com quatro
        // setas para arcos pretos). O ponto mais afastado de uma cruz é o **canto** do braço,
        // `(arm, width, half_height)` — não o meio da ponta dele.
        //
        // ⚠️ Medido na cruz que a paleta cria (`arm 0,5 · width 0,15 · half_height 0,125`): a
        // caixa dizia `0,5154` e o canto está a **`0,5368`** ⇒ a peça era **4,1 % maior do que a
        // esfera que a contém**, e o traçador corta o que fica de fora. *Um bordo menor do que a
        // peça CORTA-A e não diz nada* — é a assimetria que o doc desta função já declarava, e eu
        // caí do lado errado dela.
        //
        // ⭐ **O corte é ESFÉRICO, e é isso que o denuncia:** um arco preto a atravessar a peça,
        // e não uma linha recta. *A forma do artefacto nomeia o recurso que o causou.*
        Primitive::Cross {
            arm,
            width,
            half_height,
            ..
        } => hyp(hyp(*arm, *width), *half_height),
        // ⚠️⚠️ **`size·√2`, e o censo do módulo corrigiu-me:** o ponto mais afastado NÃO está no
        // eixo — está no lóbulo. O centro dele fica em `(±s/2, s/2)`, a `s/√2` da origem, e o raio
        // dele é `s/√2` também ⇒ a soma é `s·√2`. A 1.ª escrita somava a altura em vez da
        // distância radial e devolvia `s·1,207`, cortando a peça na caixa do mundo.
        Primitive::Heart {
            size, half_height, ..
        } => hyp(size * 2.0_f32.sqrt(), *half_height),
        Primitive::Moon {
            radius,
            half_height,
            ..
        } => hyp(*radius, *half_height),
        // A ponta esta' em `height`, que pode passar o raio.
        Primitive::Drop {
            radius,
            height,
            half_height,
            ..
        } => hyp(height.max(*radius), *half_height),
        Primitive::Pie {
            radius,
            half_height,
            ..
        } => hyp(*radius, *half_height),
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            ..
        } => hyp(hyp(bottom.max(*top), *half_width), *half_height),
        Primitive::Vesica {
            radius,
            half_height,
            ..
        } => hyp(*radius, *half_height),
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **O ponto mais afastado de uma seta é uma FARPA, não o bico**: a farpa está em
        // `(±half_length ∓ head_length, head)` e o bico em `(half_length, 0)`. Errar para CIMA é o
        // desenho desta função, e é por isso que ela toma o maior dos dois.
        Primitive::Arrow {
            half_length,
            head,
            head_length,
            half_height,
            ..
        } => hyp(
            half_length.max(hyp(half_length - head_length, *head)),
            *half_height,
        ),
        Primitive::Chevron {
            half_length,
            half_span,
            thickness,
            half_height,
            ..
        } => hyp(hyp(*half_length, half_span + thickness), *half_height),
        // ⛔⛔ **A PONTA PASSA O `run`, e a 1.ª redacção disto dizia que ela «cabe por
        // construção»:** o braço de pé está encostado em `run − shaft` e a ponta abre `head` para
        // cada lado, logo ela chega a `run − shaft + head`, que com `head > shaft` (a cerca que faz
        // dela uma seta) é SEMPRE maior que `run`. *Uma caixa menor que a peça corta-a e não diz
        // nada* — foi assim que o arco preto da cruz nasceu na W106-bis.
        Primitive::BentArrow {
            run,
            rise,
            shaft,
            head,
            half_height,
            ..
        } => hyp(hyp((run - shaft + head).max(*run), *rise), *half_height),
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            ..
        } => hyp(half_width.max(*half_span), *half_height),
        Primitive::Tube {
            outer, half_height, ..
        } => hyp(*outer, *half_height),
        Primitive::CircleSegment {
            radius,
            half_height,
            ..
        } => hyp(*radius, *half_height),
        // ─────────────────────────── W120 ───────────────────────────
        // ⚠️ **A CAUDA conta**, e é o que faz a caixa de um balão não ser a do corpo dele.
        Primitive::SpeechRect {
            half_width,
            half_span,
            tail,
            half_height,
            ..
        }
        | Primitive::SpeechOval {
            half_width,
            half_span,
            tail,
            half_height,
            ..
        } => hyp(hyp(*half_width, half_span + tail), *half_height),
        // ⚠️ **A fieira do pensamento desce `1,32 × tail` e ainda tem raio** — ver a fórmula.
        // ⚠️ **Com o inchaço da mistura dentro** — ver o irmão [`crate::bounding_half_extents`]: a
        // esfera nunca pode ser MENOR que a caixa por eixo, e há gate a afirmá-lo.
        Primitive::Cloud {
            half_width,
            half_span,
            tail,
            half_height,
            ..
        } => hyp(
            hyp(
                half_width * crate::primitive_limits::CLOUD_BLEND_SWELL,
                half_span.mul_add(
                    crate::primitive_limits::CLOUD_BLEND_SWELL - 1.0,
                    tail.mul_add(1.6, *half_span),
                ),
            ),
            *half_height,
        ),
        Primitive::Bolt {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Shield {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Tag {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Banner {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Delay {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Display {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::OffPage {
            half_width,
            half_span,
            half_height,
            ..
        } => hyp(hyp(*half_width, *half_span), *half_height),
        // ⚠️ **A inclinação entra no raio** — o canto mais afastado está em `half_width + |skew|`.
        Primitive::Parallelogram {
            half_width,
            half_span,
            skew,
            half_height,
            ..
        } => hyp(hyp(half_width + skew.abs(), *half_span), *half_height),
        Primitive::Spiral {
            radius,
            pitch,
            turns,
            thickness,
            half_height,
            ..
        } => hyp(pitch.mul_add(*turns, *radius) + thickness, *half_height),
        Primitive::Document {
            half_width,
            half_span,
            wave,
            half_height,
            ..
        } => hyp(hyp(*half_width, half_span + wave), *half_height),
        Primitive::Helix {
            radius,
            pitch,
            turns,
            thickness,
            ..
        } => hyp(radius + thickness, pitch * turns * 0.5),
        Primitive::Gyroid { half, .. } => hyp(hyp(half[0], half[1]), half[2]),
        Primitive::RoundedCylinder {
            radius,
            half_height,
            ..
        } => hyp(*radius, *half_height),
        // ⭐ O canto da caixa: a bola da norma-`n` está dentro da norma-∞ para todo `n ≥ 1`, e
        // encosta lá em cima.
        Primitive::Superquadric { half, .. } | Primitive::Superformula { half, .. } => {
            hyp(hyp(half[0], half[1]), half[2])
        }
        // ─────────────────────────── W134 ───────────────────────────
        // ⭐ **Exacto**: toda a corda vive a `tube + cord` do anel de raio `radius`, e o ponto mais
        // longe do centro é o de fora do anel, no plano dele.
        Primitive::TorusKnot {
            radius, tube, cord, ..
        } => radius + tube + cord,
        Primitive::Triangle {
            a,
            b,
            c,
            half_height,
            ..
        } => hyp(
            hyp(a[0], a[1]).max(hyp(b[0], b[1])).max(hyp(c[0], c[1])),
            *half_height,
        ),
        // ⚠️ **As faixas do visto passam do vértice, e a espessura sai para fora das pontas.**
        Primitive::Check {
            half_width,
            half_span,
            thickness,
            half_height,
            ..
        } => hyp(
            hyp(half_width + thickness, half_span + thickness),
            *half_height,
        ),
        // ⚠️ **O arco de fora chega a `1,06 × half_span` em X** — ver a fórmula da chave.
        Primitive::Brace {
            half_span,
            thickness,
            half_height,
            ..
        } => hyp(
            hyp(half_span.mul_add(1.1, *thickness), half_span + thickness),
            *half_height,
        ),
    }
}
