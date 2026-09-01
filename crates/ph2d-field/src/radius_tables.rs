//! ⭐ **A ESCALA E O BORDO de cada forma** — as duas tabelas por-primitiva que o resto do módulo
//! consulta.
//!
//! # Por que elas saíram do [`super::radius`]
//!
//! O irmão responde *que raio um nó tem e até onde ele vai* (a promessa central do módulo); este
//! responde *que tamanho a forma tem* e *que esfera a contém*. A W106 acrescentou catorze
//! primitivas e o arquivo passou dos **700** do gate de LOC.
//!
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! ⚠️ E as duas respondem a perguntas OPOSTAS, que é o que torna o corte natural: a
//! [`characteristic_size`] procura a **menor** medida (a escala do documento) e a
//! [`bounding_radius`] a **maior** (o bordo do extrator), e esta erra sempre para CIMA de propósito
//! — um bordo maior custa resolução, um bordo menor CORTA a peça e não diz nada.

use super::apothem_ratio;
use crate::Primitive;

/// **O tamanho característico de uma primitiva** — a menor dimensão que a define.
///
/// É o que dá escala a um raio de mistura: um filete maior do que a peça menor que ele junta
/// engole-a. Não é uma regra de validade (não existe nenhuma), é a escala do documento.
///
/// ⚠️ **Pública porque a mesma pergunta é feita de fora**: quando a árvore vive na cena
/// (`ph2d-field-ecs`), o limite *suave* de uma operação sai da menor peça sob ela — e ele tem de
/// ser calculado por esta função, não por uma segunda cópia. É a mesma regra do [`round_limit`].
#[must_use]
pub fn characteristic_size(p: &Primitive) -> f32 {
    match p {
        Primitive::Box { half, .. } => half[0].min(half[1]).min(half[2]),
        Primitive::Sphere { radius } => *radius,
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Torus { minor, .. } => *minor,
        Primitive::Extrude {
            profile,
            half_height,
            ..
        } => {
            let (min, max) = profile.bounds();
            half_height.min((max[0] - min[0]).min(max[1] - min[1]) * 0.5)
        }
        Primitive::Revolve { profile } => {
            let (min, max) = profile.bounds();
            (max[0] - min[0]).min(max[1] - min[1]) * 0.5
        }
        // ⚠️ **O raio MAIOR, não o menor**: num cone fechado o `top` é zero, e a menor dimensão
        // seria zero — um filete de escala zero, num nó cuja peça é perfeitamente visível. *A
        // escala do documento é o tamanho da peça, e uma ponta não é o tamanho dela.*
        Primitive::Cone {
            bottom,
            top,
            half_height,
            ..
        } => bottom.max(*top).min(*half_height),
        Primitive::Capsule {
            radius,
            half_height,
        } => radius.min(*half_height),
        // ⚠️ O apótema, pela razão do [`round_limit`]: é a parede que está mais perto do eixo.
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            ..
        } => (bottom.max(*top) * apothem_ratio(*sides)).min(*half_height),
        Primitive::Wedge { half, .. } => half[0].min(half[1]).min(half[2]),
        Primitive::TorusArc { minor, .. } => *minor,
        // ⚠️ **O raio do VALE, não o da ponta** — é a menor dimensão que define a estrela, e é
        // aquela contra a qual um filete de junção se mede (um filete maior do que o vale engole o
        // miolo e deixa só as pontas).
        Primitive::Star {
            inner, half_height, ..
        } => inner.min(*half_height),
        // ⚠️ **A ESPESSURA da viga**, e não a caixa: a peça mais fina de uma gaiola é a aresta, e
        // um filete de junção da escala da caixa engoliria a moldura inteira.
        Primitive::BoxFrame { thickness, .. } => *thickness,
        Primitive::Ellipsoid { radii } => radii[0].min(radii[1]).min(radii[2]),
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **A MENOR medida que a peca de facto tem** — e nunca uma que possa ser ZERO num
        // valor legitimo do controlo: uma escala zero daria um filete de juncao invisivel num no
        // perfeitamente visivel (a licao que o cone deixou escrita acima).
        Primitive::Octahedron { radius, .. } => *radius / 3.0_f32.sqrt(),
        // O menor dos dois raios, com o comprimento a limitar: e' a espessura da peca.
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => bottom.max(*top).min(*half_height + bottom.max(*top)),
        Primitive::CutSphere { radius, cut, .. } => (radius - cut).min(*radius),
        Primitive::HollowDome { thickness, .. } => *thickness,
        Primitive::Link { minor, .. } => *minor,
        Primitive::SolidAngle { radius, angle, .. } => radius * angle.sin().abs().max(0.05),
        // ⚠️ **O corpo, nao o dente**: o dente pode ser fino de propósito, e a escala do documento
        // e' o tamanho da peca.
        Primitive::Gear {
            root, half_height, ..
        } => root.min(*half_height),
        Primitive::Cross {
            width, half_height, ..
        } => width.min(*half_height),
        Primitive::Heart {
            size, half_height, ..
        } => size.min(*half_height),
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            ..
        } => (radius - bite + offset).max(radius * 0.1).min(*half_height),
        Primitive::Drop {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Pie {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            ..
        } => bottom.max(*top).min(*half_width).min(*half_height),
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            ..
        } => (radius - offset).max(radius * 0.1).min(*half_height),
    }
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
fn gear_planar_reach(teeth: u32, outer: f32) -> f32 {
    let n = teeth.max(crate::MIN_GEAR_TEETH);
    #[allow(clippy::cast_precision_loss)]
    let meio_passo = std::f32::consts::PI / (2.0 * n as f32);
    outer / meio_passo.cos()
}

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
        Primitive::Extrude {
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
    }
}

/// ⭐⭐⭐ **AS MEIAS-EXTENSÕES da caixa alinhada aos eixos que contém a peça** — a irmã por EIXO do
/// [`bounding_radius`] (Enio, 2026-08-31).
///
/// # ⛔⛔⛔ Por que ela existe: uma esfera não tem lados, e três dívidas medidas vinham daí
///
/// O [`bounding_radius`] devolve **um** número, e quem precisa de *«quão longe a peça chega NAQUELE
/// eixo»* tem de usar esse número em todos os três. Numa chapa alta e fina isso erra por muito, e a
/// conta paga-se em sítios que não se parecem uns com os outros:
///
/// | quem lê | o que ele queria | o que a bola dá | erro |
/// |---|---|---|---|
/// | a **parede da dobra** (`κ·W ≤ 0,9`) | a meia-espessura na direcção em que ela deflecte | o raio, dominado pela **altura** | **`17×`** |
/// | a **faixa da banda** (`Span::Along`) | a extensão no eixo do deformador | o raio | **`15×`** |
/// | o **bordo da dobra** (`bounds::step_mod`) | a meia-altura no eixo dobrado | o raio | **`1,9×`**, e ele entra num `sin` |
///
/// # ⚠️ A ASSIMETRIA é a mesma do [`bounding_radius`], e ela manda aqui
///
/// **Errar para CIMA custa resolução; errar para BAIXO corta a peça e não diz nada.** ⇒ onde a
/// orientação de uma forma no plano não é óbvia, esta tabela usa o **raio planar nos dois eixos**
/// do plano em vez de tentar ser esperta. Continua muito mais apertada do que a esfera, e não pode
/// cortar.
///
/// ⚠️ **O gate é o CAMPO, e não esta fórmula contra a outra** — ver
/// `ph2d_field_eval::the_bounding_half_extents_contain_the_piece`. Comparar duas contas nossas seria
/// cego a uma mutação que mexesse nas duas.
#[must_use]
#[allow(clippy::match_same_arms)]
pub fn bounding_half_extents(p: &Primitive) -> [f32; 3] {
    // Uma forma **plana em Z** (a família esmagadora desta tabela): raio no plano, altura no eixo.
    let chata = |r: f32, h: f32| [r, r, h];
    match p {
        Primitive::Box { half, .. } | Primitive::Wedge { half, .. } => *half,
        Primitive::BoxFrame { half, .. } => *half,
        Primitive::Sphere { radius } => [*radius; 3],
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => chata(*radius, *half_height),
        // ⚠️ O toro vive no plano **XY**: a espessura fora dele é só o tubo.
        Primitive::Torus { major, minor } | Primitive::TorusArc { major, minor, .. } => {
            [major + minor, major + minor, *minor]
        }
        // O perfil dá as duas do plano; a extrusão dá a terceira.
        Primitive::Extrude {
            profile,
            half_height,
            ..
        } => {
            let (min, max) = profile.bounds();
            [
                min[0].abs().max(max[0].abs()),
                min[1].abs().max(max[1].abs()),
                *half_height,
            ]
        }
        // ⚠️ O torno gira em torno de **Y**: o `x` do perfil vira o raio (X e Z), o `y` a altura.
        Primitive::Revolve { profile } => {
            let (min, max) = profile.bounds();
            let r = min[0].abs().max(max[0].abs());
            [r, min[1].abs().max(max[1].abs()), r]
        }
        Primitive::Cone {
            bottom,
            top,
            half_height,
            ..
        }
        | Primitive::Prism {
            bottom,
            top,
            half_height,
            ..
        } => chata(bottom.max(*top), *half_height),
        // ⚠️ **A ponta está no EIXO, a `h + r`** — a mesma nota da [`bounding_radius`].
        Primitive::Capsule {
            radius,
            half_height,
        } => chata(*radius, half_height + radius),
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => {
            let r = bottom.max(*top);
            chata(r, half_height + r)
        }
        Primitive::Star {
            outer, half_height, ..
        } => chata(*outer, *half_height),
        // ⛔ Ver [`gear_planar_reach`] — a ponta de um dente é uma corda, e os cantos dela passam
        // do `outer`.
        Primitive::Gear {
            teeth,
            outer,
            half_height,
            ..
        } => chata(gear_planar_reach(*teeth, *outer), *half_height),
        // ⭐ **A única forma cuja caixa é EXACTA por eixo** — e é por isso que ela existe.
        Primitive::Ellipsoid { radii } => *radii,
        Primitive::Octahedron { radius, .. } | Primitive::CutSphere { radius, .. } => [*radius; 3],
        Primitive::SolidAngle { radius, .. } => [*radius; 3],
        Primitive::HollowDome {
            radius, thickness, ..
        } => [radius + thickness * 0.5; 3],
        // ⚠️ O elo é um estádio no plano **XY**, alongado em Y; fora dele é só o tubo.
        Primitive::Link {
            major,
            minor,
            length,
        } => [major + minor, length + major + minor, *minor],
        // ⚠️ **O canto do braço**, e não o meio da ponta — a nota que o report dos arcos pretos
        // deixou na [`bounding_radius`]. Aqui os dois braços dão a mesma extensão.
        Primitive::Cross {
            arm, half_height, ..
        } => chata(*arm, *half_height),
        // ⚠️ **O raio PLANAR nos dois eixos**, de propósito: a extensão exacta do coração em `x` e
        // em `y` é `s·(½ + 1/√2)`, mas as duas dependem da orientação dos lóbulos, e a assimetria
        // desta função manda errar para cima. Ver o doc.
        Primitive::Heart {
            size, half_height, ..
        } => chata(size * std::f32::consts::SQRT_2, *half_height),
        Primitive::Moon {
            radius,
            half_height,
            ..
        }
        | Primitive::Pie {
            radius,
            half_height,
            ..
        }
        | Primitive::Vesica {
            radius,
            half_height,
            ..
        } => chata(*radius, *half_height),
        Primitive::Drop {
            radius,
            height,
            half_height,
            ..
        } => chata(height.max(*radius), *half_height),
        // ⚠️ **As duas do plano são DIFERENTES aqui** — é a única da família plana em que a largura
        // e a profundidade do contorno não são a mesma grandeza.
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            ..
        } => [bottom.max(*top), *half_width, *half_height],
    }
}
