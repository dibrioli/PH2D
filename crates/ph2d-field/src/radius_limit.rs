//! ⭐ **ATÉ ONDE O FILETE DE CADA FORMA PODE IR** — a tabela por-primitiva que o painel e a
//! validação partilham.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::radius`] responde *que raio um nó TEM, e quem o escreve*; este responde *até onde ele
//! pode ir em cada forma*. A W119 acrescentou seis primitivas e o arquivo passou as `700` linhas do
//! gate de LOC da workspace. ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! ⭐ **É a MESMA função que a validação usa, e é essa a razão de ela ser pública** — um painel que
//! calculasse o próprio teto ofereceria valores que o documento recusa, e o utilizador veria o
//! controle parar sem explicação.

use super::radius::{apothem_ratio, cone_round_limit};
use crate::Primitive;

/// **Até onde o `round` desta primitiva pode ir** — `None` se ela não tem `round`.
///
/// ⭐ **É a MESMA função que a validação usa.** Um painel que calculasse o próprio teto ofereceria
/// valores que o documento recusa, e o utilizador veria o controle parar sem explicação — a forma
/// clássica de dois lados divergirem sobre a mesma regra.
#[must_use]
pub fn round_limit(p: &Primitive) -> Option<f32> {
    match p {
        // A MENOR meia-extensão: a receita encolhe a caixa em `round` nos três eixos, e uma delas
        // ficando ≤ 0 não é "quase" — é uma caixa que deixou de existir naquele eixo.
        Primitive::Box { half, .. } => Some(half[0].min(half[1]).min(half[2])),
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => Some(radius.min(*half_height)),
        // Só a meia-altura: um `round` maior que a meia-largura do perfil é uma ABERTURA, não um
        // erro (ver a nota de [`Primitive::Extrude`]).
        Primitive::Extrude { half_height, .. } => Some(*half_height),
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => None,
        // ⚠️ **O raio do TUBO**: o filete come o aro do corte de dentro para fora, e a `minor` ele
        // teria comido o tubo inteiro — a face cortada deixaria de existir.
        Primitive::TorusArc { minor, .. } => Some(*minor),

        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Cada uma diz de que RECURSO é o limite** — a menor meia-medida que a aresta come.
        // Um teto que só dissesse «por segurança» seria um palpite à espera de um smoke (§0.0).
        //
        // ⭐ **Toda CHAPA leva `.min(half_height)`**: o aro entre a parede e a tampa é uma aresta
        // como as outras, e um filete maior que a meia-espessura comeria a chapa inteira.

        // O INRAIO: a receita recua as oito faces de `round`, e a `radius/√3` elas cruzam-se no
        // centro — o octaedro deixaria de existir.
        Primitive::Octahedron { radius, .. } => Some(*radius / 3.0_f32.sqrt()),
        // A aresta é o aro do corte. Ela é comida de dois lados: pela **altura da calota** que
        // sobra (`radius − cut`) e pelo **raio da tampa** (`√(r²−cut²)`).
        Primitive::CutSphere { radius, cut, .. } => {
            Some((radius - cut).min((radius * radius - cut * cut).max(0.0).sqrt()))
        }
        // ⚠️ **A PAREDE, não a esfera**: a casca tem `thickness` de espessura, e um filete acima de
        // metade dela atravessa-a de lado a lado.
        Primitive::HollowDome { thickness, .. } => Some(*thickness * 0.5),
        // A aresta é o arco onde a calota encontra o cone. Ela é comida pelo raio e pela abertura:
        // num ângulo pequeno a fatia é fina, e é a espessura dela que manda.
        Primitive::SolidAngle { radius, angle, .. } => Some(radius * angle.sin().abs().min(1.0)),
        // ⚠️ **O DENTE é a peça pequena**, e é ele que o filete come primeiro: metade da largura
        // dele na base, e a altura dele (`outer − root`). O corpo é sempre maior.
        Primitive::Gear {
            teeth,
            root,
            outer,
            tooth,
            half_height,
            ..
        } => {
            let passo = std::f32::consts::TAU / (*teeth).max(3) as f32;
            let meia_largura = root * passo * 0.5 * tooth.clamp(0.05, 0.95);
            Some(meia_largura.min(outer - root).min(*half_height).max(0.0))
        }
        // A meia-largura do braço, e a profundidade da cova (`arm − width`).
        Primitive::Cross {
            arm,
            width,
            half_height,
            ..
        } => Some(width.min(arm - width).min(*half_height).max(0.0)),
        // O lóbulo tem raio `size/√2`; a ponta de baixo é a quina que o filete come.
        Primitive::Heart {
            size, half_height, ..
        } => Some((size * 0.5).min(*half_height)),
        // ⚠️ **A ESPESSURA do crescente no dorso** — `radius − bite + offset`. É ela que some
        // primeiro, e não o raio: um crescente fino com um raio grande parte na cintura.
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            ..
        } => Some(((radius - bite + offset) * 0.5).max(0.0).min(*half_height)),
        // A bolha manda: a ponta é tangente e não tem quina para arredondar.
        Primitive::Drop {
            radius,
            half_height,
            ..
        } => Some((radius * 0.5).min(*half_height)),
        // Como no ângulo sólido: o raio e a abertura, o que for menor.
        Primitive::Pie {
            radius,
            angle,
            half_height,
            ..
        } => Some((radius * angle.sin().abs().min(1.0)).min(*half_height)),
        // A menor das três meias-medidas — a base estreita é a que desaparece.
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            ..
        } => Some(bottom.min(*top).min(*half_width).min(*half_height)),
        // ⚠️ **A meia-largura da LENTE** (`radius − offset`), não o raio: a vesica é fina de
        // propósito, e é a espessura dela que o filete come.
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            ..
        } => Some(((radius - offset) * 0.5).max(0.0).min(*half_height)),
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **A HASTE manda, e não o comprimento**: o filete come a chapa de fora para dentro, e
        // uma haste fina desaparece muito antes de a ponta o sentir.
        Primitive::Arrow {
            shaft, half_height, ..
        } => Some(shaft.min(*half_height)),
        // A espessura da banda é toda a matéria que o chevron tem na transversal.
        Primitive::Chevron {
            thickness,
            half_height,
            ..
        } => Some((thickness * 0.5).min(*half_height)),
        Primitive::BentArrow {
            shaft, half_height, ..
        } => Some(shaft.min(*half_height)),
        // ⚠️ **A meia-altura do losango na DIREÇÃO MAIS CURTA** — a erosão de um losango por `r`
        // fecha quando `r` chega ao raio inscrito, que é `a·b/√(a²+b²)` e nunca a menor diagonal.
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            ..
        } => Some(
            (half_width * half_span / (half_width * half_width + half_span * half_span).sqrt())
                .min(*half_height),
        ),
        // ⚠️ **A PAREDE do tubo** (`outer − inner`), não o bordo: o filete come os dois aros ao mesmo
        // tempo, e a meia-parede é onde eles se encontram.
        Primitive::Tube {
            outer,
            inner,
            half_height,
            ..
        } => Some((((outer - inner) * 0.5).max(0.0)).min(*half_height)),
        // ⚠️ **A ALTURA DA CALOTE** (`radius − cut`), e não o raio: um segmento raso é uma lasca, e
        // é a espessura dela que o filete come.
        Primitive::CircleSegment {
            radius,
            cut,
            half_height,
            ..
        } => Some((((radius - cut) * 0.5).max(0.0)).min(*half_height)),
        // ─────────────────────────── W120 ───────────────────────────
        // ⚠️ **A MENOR meia-medida manda**, e a erosão de uma chapa fecha ali: um balão come-se pelo
        // lado curto, e a cauda dele é o que sobra por último.
        Primitive::SpeechRect {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::SpeechOval {
            half_width,
            half_span,
            half_height,
            ..
        } => Some(half_width.min(*half_span).min(*half_height)),
        // ⚠️ **A menor BOSSA** — ver a fórmula: a mais pequena mede `0,52 × half_span`.
        Primitive::Cloud {
            half_span,
            half_height,
            ..
        } => Some((half_span * 0.52).min(*half_height)),
        // ⚠️ **A banda do meio do zigue-zague** é a parte fina do raio, e ela vale `0,2 h` na
        // fórmula; o filete come-a pelos dois lados.
        // ⚠️ **A banda do meio mede `0,2 × half_span` NA VERTICAL**, e é ela que o filete come — a
        // 1.ª redacção tomava `min(largura, altura)`, que numa peça alta e estreita descreve a
        // LARGURA e aperta o teto a metade sem razão. *Uma parede tem de dizer de que medida ela é.*
        Primitive::Bolt {
            half_span,
            half_height,
            ..
        } => Some((half_span * 0.10).min(*half_height)),
        Primitive::Shield {
            half_width,
            half_span,
            half_height,
            ..
        } => Some((half_width.min(*half_span) * 0.5).min(*half_height)),
        // ⚠️ **A ponta e o furo mandam**: o bico afila até nada, e o filete não pode comer a boca do
        // furo.
        Primitive::Tag {
            half_span,
            point,
            hole,
            half_height,
            ..
        } => Some(half_span.min(point * 0.5).min(*hole).min(*half_height)),
        // A meia-espessura da faixa é toda a matéria que um visto tem na transversal.
        Primitive::Check {
            thickness,
            half_height,
            ..
        } => Some((thickness * 0.5).min(*half_height)),
        // ⛔ **A chave fica na METADE, e recuar para `0,45` foi MEDIDO e REVERTIDO** (05/09): a
        // marcha nem se mexeu (`1,0234 → 1,0233`) e o chanfro passou a cortar `35,6 %` das arestas
        // em vez de `54,8 %` — a sonda usa `metade da parede`, então apertar a parede aperta o
        // chanfro que ela mede. *Uma mudança que não move a régua que a motivou e piora outra é uma
        // regressão com cara de cura.*
        Primitive::Brace {
            thickness,
            half_height,
            ..
        } => Some((thickness * 0.5).min(*half_height)),
        // ─────────────────────────── W122 ───────────────────────────
        // ⭐ **A meia-largura de um paralelogramo é a distância entre os DOIS FLANCOS**, e ela
        // encolhe com a inclinação: `half_width/√(1+k²)`. ⚠️ Usar `half_width` cru prometeria um
        // filete que a peça inclinada já não comporta.
        Primitive::Parallelogram {
            half_width,
            half_span,
            skew,
            half_height,
            ..
        } => {
            let k = skew / half_span.max(f32::MIN_POSITIVE);
            Some(
                (half_width / k.mul_add(k, 1.0).sqrt())
                    .min(*half_span)
                    .min(*half_height),
            )
        }
        // ⚠️ **A envergadura manda**: ela é o raio da tampa redonda, e o filete come a chapa de
        // fora para dentro.
        Primitive::Delay {
            half_span,
            half_height,
            ..
        } => Some(half_span.min(*half_height)),
        // ⚠️ **O BICO é mais apertado que a envergadura** quando ele é curto: a meia-largura útil
        // junto da ponta é `point·half_span/√(point² + half_span²)`, que é a altura do triângulo
        // do bico sobre a hipotenusa. ⛔ Sem ela o filete engolia a ponta inteira.
        Primitive::Display {
            half_span,
            point,
            half_height,
            ..
        } => Some(half_span.min(bico(*point, *half_span)).min(*half_height)),
        Primitive::OffPage {
            half_width,
            half_span,
            point,
            half_height,
            ..
        } => Some(
            half_width
                .min(*half_span)
                .min(bico(*point, *half_width))
                .min(*half_height),
        ),
        // ─────────────────────────── W123 ───────────────────────────
        // ⚠️ **A meia-espessura da FITA manda** — o filete come-a dos dois lados.
        Primitive::Spiral {
            thickness,
            half_height,
            ..
        } => Some(thickness.min(*half_height)),
        // ⚠️ **O que sobra entre a CRISTA da onda e o topo** — é a parte mais fina da peça.
        Primitive::Document {
            half_width,
            half_span,
            wave,
            half_height,
            ..
        } => Some(
            half_width
                .min((half_span - wave * 0.5).max(0.0))
                .min(*half_height),
        ),
        // ⚠️ **O que sobra da fita entre o entalhe e a ponta** — não a meia-largura inteira.
        Primitive::Banner {
            half_width,
            half_span,
            notch,
            half_height,
            ..
        } => Some(
            half_span
                .min(((half_width - notch) * 0.5).max(0.0))
                .min(*half_height),
        ),
        // ⭐⭐ **A INCLINAÇÃO ENTRA NA CONTA, e é onde o filete SATURA** (W101).
        //
        // A parede é a reta `ρ = a + m·z` no plano `(ρ, z)`; recuá-la de `round` na perpendicular
        // baixa `a` de `round·√(1+m²)`. No limite, a parede recuada passa pelo **eixo**: dali para
        // cima não há mais parede lateral para arredondar.
        //
        // # ⚠️ Este limite NÃO é uma parede de validade — a medição refutou a redação anterior
        //
        // Ela dizia que sem o `√(1+m²)` *«um cone raso com filete sairia MAIOR do que o pedido»*, e
        // uma mutação que o apagasse **sobreviveu com razão**. Sondado com `round` a `1,4×` o
        // limite (e acima da própria meia-altura):
        //
        // | round | raio máximo | meia-altura | `‖∇f‖` |
        // |---|---|---|---|
        // | `0,2575` (o limite) | `0,4497` | `0,3498` | `1,0000` |
        // | `0,3990` (**1,55× o limite**) | `0,4497` | `0,3498` | `1,0000` |
        //
        // (autorados: raio `0,4500`, meia-altura `0,3500`.)
        //
        // ⭐ **O `max` + `offset` é auto-corretivo**: o que o recuo tira, o deslocamento repõe, e a
        // silhueta é **exatamente** `ρ ≤ a + m·z` para qualquer `round`. É a diferença para a caixa
        // e o cilindro, onde o termo axial **inverte** de sinal com uma meia-extensão negativa (a
        // nota do [`crate::Primitive::Extrude`] diz-o) — ali o limite é validade, aqui é **produto**.
        //
        // ⇒ o número fica, porque é o ponto onde o filete deixa de ter parede para comer e o
        // controle deixaria de fazer alguma coisa; ⛔ mas nenhum gate o pode defender como
        // correção, e inventar um seria escrever uma afirmação sobre nada.
        Primitive::Cone {
            bottom,
            top,
            half_height,
            ..
        } => Some(cone_round_limit(*bottom, *top, *half_height)),
        // ⚠️ **O apótema, não o circunraio**: a parede de um prisma está a `radius·cos(π/n)` do
        // eixo, e usar o circunraio deixaria o filete comer a parede antes de o limite o dizer.
        // ⚠️ **O apótema do LADO MAIS LARGO**, e a inclinação entra como no cone: o prisma
        // estreitado tem a parede inclinada, e recuá-la de `round` na perpendicular custa
        // `round·√(1+m²)`. A conta é a mesma porta do cone, com o apótema no lugar do raio.
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            ..
        } => {
            let k = apothem_ratio(*sides);
            Some(cone_round_limit(bottom * k, top * k, *half_height))
        }
        // ⚠️ **A parede inclinada da cunha é a que manda**, e ela recua `round·√(1+m²)` com
        // `m = hx/hz` — a mesma lei do cone, noutro plano. O `min` com as três meias-extensões
        // fecha as faces rectas.
        Primitive::Wedge { half, .. } => {
            let d = (half[0] * half[0] + half[2] * half[2]).sqrt();
            let plano = if d > f32::MIN_POSITIVE {
                half[0] * half[2] / d
            } else {
                0.0
            };
            Some(half[0].min(half[1]).min(half[2]).min(plano))
        }
        // ⭐⭐ **Aqui o limite NÃO é «a peça deixa de existir» — é «a ESTRELA deixa de ser uma
        // estrela»**, e a distinção é o que o torna o número certo.
        //
        // O filete é o do **aro**, e a pegada dele na tampa é a **erosão 2D** da estrela por
        // `round`: a ponta recua e o vale avança, cada um `round/sin α` (ver [`star_round_limit`]).
        // No limite os dois chegam ao MESMO raio e a tampa vira um polígono regular de `2n` lados —
        // que é o maior filete que ainda arredonda uma estrela. Um passo acima, a ponta fica
        // **dentro** do vale e a tampa deixa de ser a forma que o artista autorou.
        Primitive::Star {
            points,
            outer,
            inner,
            half_height,
            ..
        } => Some(half_height.min(star_round_limit(*points, *outer, *inner))),
        // ⚠️ **Metade da espessura**: o recuo come a viga dos DOIS lados, e a `e/2` ela desaparece.
        // O `min` com as meias-extensões fecha o caso da moldura mais fina que baixa que grossa.
        Primitive::BoxFrame {
            half, thickness, ..
        } => Some((thickness * 0.5).min(half[0]).min(half[1]).min(half[2])),
    }
}

/// Até onde o filete de um [`Primitive::Star`] pode ir — ver a nota em [`round_limit`].
///
/// ⭐ **A conta é a do canto deslocado**, e vale para os dois cantos de uma vez: recuar as duas
/// arestas de um vértice de meio-ângulo interno `α` move-o `round/sin α` ao longo da bissetriz. Com
/// `β = π/n` e `|u|` o comprimento de uma aresta, `sin α` vale `q·sin β/|u|` na ponta e
/// `R·sin β/|u|` no vale — as duas saem da MESMA aresta, e é por isso que uma função só as
/// responde. Igualar `R'` a `q'` dá o número.
///
/// ⚠️ **A erosão não está no CAMPO** — desde a W103 a estrela é construída com as paredes onde foram
/// autoradas, e quem arredonda é a interseção com a laje. Esta conta responde só *«até onde o filete
/// ainda arredonda uma estrela»*, que é a pegada dele na tampa.
#[must_use]
pub fn star_round_limit(points: u32, outer: f32, inner: f32) -> f32 {
    let n = points.max(crate::MIN_STAR_POINTS);
    let beta = std::f32::consts::PI / n as f32;
    let u = (outer * outer + inner * inner - 2.0 * outer * inner * beta.cos()).sqrt();
    if u <= f32::MIN_POSITIVE || outer <= inner {
        return 0.0;
    }
    (outer - inner) * inner * outer * beta.sin() / (u * (outer + inner))
}

/// ⭐ **A meia-espessura ÚTIL de um bico** — a altura do triângulo `(base, altura)` medida sobre a
/// hipotenusa, `base·altura/√(base² + altura²)`.
///
/// ⚠️ **Com o bico a zero ela é zero, e isso está certo**: sem bico a forma não tem ponta nenhuma, e
/// quem manda no filete é a outra meia-medida — os chamadores tomam o `min` com ela.
fn bico(base: f32, altura: f32) -> f32 {
    if base <= 0.0 {
        return f32::INFINITY;
    }
    base * altura / base.hypot(altura)
}
