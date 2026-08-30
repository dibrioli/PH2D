//! **O raio editável** — a promessa central do módulo, num arquivo.
//!
//! ⭐ *O raio do filete fica editável para sempre*, porque é parâmetro da operação e não geometria
//! assada. Tudo o que responde a *"que raio este nó tem, até onde ele vai, e o que acontece se eu o
//! mudar?"* vive aqui — e vive **uma vez**, porque a árvore tem dois donos possíveis (a arena
//! cozida e a cena ECS) e os dois têm de aplicar a mesma regra.

use crate::{FieldDoc, FieldError, NodeId, NodeKind, NodeShape, Op, Primitive};

impl FieldDoc {
    /// **O raio EDITÁVEL de um nó** — `None` quando não há nenhum.
    ///
    /// ⭐ É a promessa central do módulo virada em função: *o raio fica editável para sempre*,
    /// porque é parâmetro da operação e não geometria assada. Para uma combinação é o raio da
    /// mistura; para uma primitiva, o `round` da aresta convexa dela.
    #[must_use]
    pub fn radius_of(&self, node: NodeId) -> Option<f32> {
        match &self.node(node)?.kind {
            NodeKind::Combine { op, .. } => Some(op.blend().amount()),
            NodeKind::Leaf(p) => match p {
                Primitive::Box { round, .. }
                | Primitive::Cylinder { round, .. }
                | Primitive::Extrude { round, .. }
                | Primitive::Cone { round, .. }
                | Primitive::Prism { round, .. }
                | Primitive::Wedge { round, .. }
                | Primitive::Star { round, .. }
                | Primitive::BoxFrame { round, .. }
                | Primitive::Octahedron { round, .. }
                | Primitive::CutSphere { round, .. }
                | Primitive::HollowDome { round, .. }
                | Primitive::SolidAngle { round, .. }
                | Primitive::Gear { round, .. }
                | Primitive::Cross { round, .. }
                | Primitive::Heart { round, .. }
                | Primitive::Moon { round, .. }
                | Primitive::Drop { round, .. }
                | Primitive::Pie { round, .. }
                | Primitive::Trapezoid { round, .. }
                | Primitive::Vesica { round, .. }
                | Primitive::TorusArc { round, .. } => Some(*round),
                // ⚠️ Lista FECHADA desde a W101 (era `_ => None`): uma primitiva nova COM filete
                // caía no braço vazio e o painel dizia que ela não tinha nenhum.
                Primitive::Sphere { .. }
                | Primitive::Torus { .. }
                | Primitive::Revolve { .. }
                | Primitive::Capsule { .. }
                | Primitive::RoundCone { .. }
                | Primitive::Link { .. }
                | Primitive::Ellipsoid { .. } => None,
            },
            // Uma escultura não tem aresta autorada: o `round` dela é a malha.
            NodeKind::Sampled { .. } => None,
        }
    }

    /// Até onde esse raio pode ir, e de que natureza é o limite. Ver [`Bound`].
    #[must_use]
    pub fn radius_bound(&self, node: NodeId) -> Option<Bound> {
        match &self.node(node)?.kind {
            // Um raio de mistura não tem limite de VALIDADE: o campo continua a ser uma distância
            // com qualquer raio. O que existe é escala — e ela vem da menor peça sob este nó,
            // porque um filete maior do que ela engole-a.
            NodeKind::Combine { .. } => Some(Bound::Soft(self.subtree_scale(node))),
            NodeKind::Leaf(p) => round_limit(p).map(Bound::Hard),
            NodeKind::Sampled { .. } => None,
        }
    }

    /// A menor peça sob um nó — a escala que dá sentido a um raio de mistura.
    ///
    /// ⚠️ Uma passagem **de baixo para cima**, sem recursão: a invariante da arena (todo filho antes
    /// do pai) já garante que os filhos foram vistos quando se chega ao pai.
    fn subtree_scale(&self, node: NodeId) -> f32 {
        let mut scale = vec![f32::INFINITY; self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            scale[i] = match &n.kind {
                NodeKind::Leaf(p) => characteristic_size(p) * n.xform.scale,
                NodeKind::Combine { children, .. } => children
                    .iter()
                    .map(|c| scale[c.0 as usize])
                    .fold(f32::INFINITY, f32::min),
                // ⚠️ A escala característica de uma escultura é a caixa dela, e a caixa vive no
                // campo amostrado, que o documento não conhece. `INFINITY` faz o `min` do pai
                // ignorá-la — o que é a resposta certa quando há outra peça a dar a escala, e a
                // resposta a melhorar quando a escultura for a única (nomeado, não escondido).
                NodeKind::Sampled { .. } => f32::INFINITY,
            };
        }
        let s = scale[node.0 as usize];
        if s.is_finite() && s > 0.0 { s } else { 1.0 }
    }

    /// **Muda o raio de um nó**, e **revalida**.
    ///
    /// ⚠️ A revalidação é a razão de esta ser a única porta: a invariante da crate é *um documento
    /// que existe está válido*, e um `set` que a quebrasse produziria a forma errada em silêncio, e
    /// não um erro. Quando ela recusa, o documento fica **como estava** — um documento meio-mudado
    /// seria pior do que a recusa.
    ///
    /// Numa mistura viva (`Blend::Sharp`), um raio positivo acorda-a como [`Blend::Exact`]: a aresta
    /// viva é o raio zero, e não um modo à parte.
    ///
    /// # Errors
    /// Ver [`FieldError`]. `BadRoot` se o nó não existe; `NonPositive` para um raio não-finito.
    pub fn set_radius(&mut self, node: NodeId, radius: f32) -> Result<(), FieldError> {
        let idx = node.0 as usize;
        if idx >= self.nodes.len() {
            return Err(FieldError::BadRoot);
        }
        if !radius.is_finite() || radius < 0.0 {
            return Err(FieldError::NonPositive {
                node: node.0,
                what: "radius",
            });
        }
        let previous = self.nodes[idx].clone();
        let mut shape = self.nodes[idx].kind.shape();
        set_shape_radius(&mut shape, node.0, radius)?;
        self.nodes[idx].kind = match (shape, &self.nodes[idx].kind) {
            (NodeShape::Leaf(p), _) => NodeKind::Leaf(p),
            (NodeShape::Sampled { key }, _) => NodeKind::Sampled { key },
            (NodeShape::Combine(op), NodeKind::Combine { children, .. }) => NodeKind::Combine {
                op,
                children: children.clone(),
            },
            // Impossível: `shape()` preserva a variante. O braço existe porque um `unreachable!`
            // numa porta de escrita é uma aposta, e devolver o nó intacto é a resposta segura.
            (NodeShape::Combine(_), k) => k.clone(),
        };
        if let Err(e) = self.validate() {
            self.nodes[idx] = previous;
            return Err(e);
        }
        Ok(())
    }
}

impl NodeShape {
    /// **O raio EDITÁVEL desta forma** — `None` quando não há nenhum.
    #[must_use]
    pub fn radius(&self) -> Option<f32> {
        match self {
            NodeShape::Combine(op) => Some(op.blend().amount()),
            NodeShape::Sampled { .. } => None,
            NodeShape::Leaf(p) => match p {
                Primitive::Box { round, .. }
                | Primitive::Cylinder { round, .. }
                | Primitive::Extrude { round, .. }
                | Primitive::Cone { round, .. }
                | Primitive::Prism { round, .. }
                | Primitive::Wedge { round, .. }
                | Primitive::Star { round, .. }
                | Primitive::BoxFrame { round, .. }
                | Primitive::Octahedron { round, .. }
                | Primitive::CutSphere { round, .. }
                | Primitive::HollowDome { round, .. }
                | Primitive::SolidAngle { round, .. }
                | Primitive::Gear { round, .. }
                | Primitive::Cross { round, .. }
                | Primitive::Heart { round, .. }
                | Primitive::Moon { round, .. }
                | Primitive::Drop { round, .. }
                | Primitive::Pie { round, .. }
                | Primitive::Trapezoid { round, .. }
                | Primitive::Vesica { round, .. }
                | Primitive::TorusArc { round, .. } => Some(*round),
                // ⚠️ Lista FECHADA desde a W101 (era `_ => None`): uma primitiva nova COM filete
                // caía no braço vazio e o painel dizia que ela não tinha nenhum.
                Primitive::Sphere { .. }
                | Primitive::Torus { .. }
                | Primitive::Revolve { .. }
                | Primitive::Capsule { .. }
                | Primitive::RoundCone { .. }
                | Primitive::Link { .. }
                | Primitive::Ellipsoid { .. } => None,
            },
        }
    }
}

/// **Muda o raio de uma forma**, ou recusa.
///
/// ⭐ É a **lei**, e os dois donos de árvore a chamam: o documento cozido
/// ([`FieldDoc::set_radius`], que revalida a árvore inteira depois) e a cena
/// (`ph2d-field-ecs`, que valida o nó). Duas cópias divergiriam no dia em que uma primitiva nova
/// ganhasse `round` — e a que ficasse para trás recusaria em silêncio um raio legítimo.
///
/// Numa mistura viva (`Blend::Sharp`), um raio positivo acorda-a como [`Blend::Exact`]: a aresta
/// viva é o raio zero, e não um modo à parte.
///
/// `node` entra só na mensagem de erro — a forma sozinha não sabe onde vive.
///
/// # Errors
/// [`FieldError::NonPositive`] para um raio não-finito, negativo, ou numa forma sem raio nenhum;
/// [`FieldError::RoundTooLarge`] quando ele não cabe na primitiva.
pub fn set_shape_radius(shape: &mut NodeShape, node: u32, radius: f32) -> Result<(), FieldError> {
    if !radius.is_finite() || radius < 0.0 {
        return Err(FieldError::NonPositive {
            node,
            what: "radius",
        });
    }
    match shape {
        // Uma escultura não tem raio autorado: recusar é o que impede um slider de existir sem nada
        // do outro lado.
        NodeShape::Sampled { .. } => Err(FieldError::NonPositive {
            node,
            what: "radius",
        }),
        NodeShape::Combine(op) => {
            // ⭐ O CARÁCTER é uma escolha de produto e sobrevive a mudar o número; trocá-lo aqui
            // seria decidir por quem só mexeu num slider. ⚠️ **A escada vive numa porta só**
            // ([`Blend::with_amount`]): enquanto foi copiada, os dois caminhos que a usam
            // discordavam sobre o que um zero faz a um chanfro.
            let blend = op.blend().with_amount(radius);
            *op = match *op {
                Op::Union(_) => Op::Union(blend),
                Op::Intersection(_) => Op::Intersection(blend),
                Op::Difference(_) => Op::Difference(blend),
            };
            Ok(())
        }
        NodeShape::Leaf(p) => {
            let limit = round_limit(p).ok_or(FieldError::NonPositive {
                node,
                what: "radius",
            })?;
            if radius >= limit {
                return Err(FieldError::RoundTooLarge {
                    node,
                    round: radius,
                    limit,
                });
            }
            match p {
                Primitive::Box { round, .. }
                | Primitive::Cylinder { round, .. }
                | Primitive::Extrude { round, .. }
                | Primitive::Cone { round, .. }
                | Primitive::Prism { round, .. }
                | Primitive::Wedge { round, .. }
                | Primitive::Star { round, .. }
                | Primitive::BoxFrame { round, .. }
                | Primitive::Octahedron { round, .. }
                | Primitive::CutSphere { round, .. }
                | Primitive::HollowDome { round, .. }
                | Primitive::SolidAngle { round, .. }
                | Primitive::Gear { round, .. }
                | Primitive::Cross { round, .. }
                | Primitive::Heart { round, .. }
                | Primitive::Moon { round, .. }
                | Primitive::Drop { round, .. }
                | Primitive::Pie { round, .. }
                | Primitive::Trapezoid { round, .. }
                | Primitive::Vesica { round, .. }
                | Primitive::TorusArc { round, .. } => *round = radius,
                // Inalcançável: `round_limit` já devolveu `None` para estas acima.
                Primitive::Sphere { .. }
                | Primitive::Torus { .. }
                | Primitive::Revolve { .. }
                | Primitive::Capsule { .. }
                | Primitive::RoundCone { .. }
                | Primitive::Link { .. }
                | Primitive::Ellipsoid { .. } => {}
            }
            Ok(())
        }
    }
}

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

/// ⭐⭐⭐ **O FILETE DESTA FORMA INFLA O GRADIENTE?** (W103) — a pergunta que a marcha faz.
///
/// # Por que a resposta não é a mesma para todas
///
/// ⭐ **O chanfro desta forma, ou `0` se ela não tiver aresta.**
///
/// ⚠️ **Derivado da [`crate::dims`], e não de uma segunda lista escrita à mão** — as 21 primitivas
/// com aresta já estão enumeradas em três sítios desta crate, e uma quarta cópia divergiria no dia
/// em que a vigésima segunda nascesse.
#[must_use]
pub fn chamfer_of(p: &Primitive) -> f32 {
    crate::dims(p)
        .iter()
        .find(|d| d.key == "field.dim.chamfer")
        .map_or(0.0, |d| d.value)
}

/// O filete desta forma, pela mesma porta do [`chamfer_of`].
#[must_use]
pub fn round_of(p: &Primitive) -> f32 {
    crate::dims(p)
        .iter()
        .find(|d| d.key == "field.dim.round")
        .map_or(0.0, |d| d.value)
}

/// ⭐⭐⭐ **POR QUANTO O CAMPO DE UMA FORMA ENCOLHE quando ela tem os DOIS recuos** — e por que ele é
/// um divisor **local** e não um passo mais curto para o documento inteiro.
///
/// # ⛔ O report que isto fecha: *«o fillet só muda a posição do chamfer»* (Enio, 2026-08-30)
///
/// Arredondar as arestas que um chanfro cria **exige** o operador de mistura — a alternativa
/// («encolher, chanfrar, deslocar») foi construída, medida e **não arredonda**: o giro da normal na
/// quina fica cravado em `45,000°` para qualquer filete, e só a posição dela desliza. É a lei que a
/// W104 já tinha escrito neste módulo — *deslocar um semiespaço dá outro semiespaço*.
///
/// ⇒ a mistura fica, e com ela o campo deixa de ser uma distância: medido `‖∇f‖` até **`5,02`** num
/// prisma com os dois recuos a meia parede.
///
/// # ⭐ Por que um DIVISOR e não um passo mais curto
///
/// O `ph2d_field_eval::gradient_bound` é do **documento**: baixar o passo ali castiga uma cena
/// inteira por causa de uma forma chanfrada — o §0 do `CLAUDE.md` ao contrário. O divisor deixa o
/// campo ser um **minorante honesto** dessa forma, o passo do documento fica cheio, e quem paga é
/// só a marcha que atravessa aquela peça (o orçamento sobe pelo `field_shrink`, a mesma
/// arquitectura que a torção e a dobra já usam).
///
/// ⚠️ **Os números são MEDIDOS e GATEADOS** — `every_shape_marches_safely_with_both_recesses_on`
/// varre as vinte formas com aresta e reprova se `passo × ‖∇f‖` passar de `1`. Uma primitiva nova
/// que os estoure fica **vermelha**, que é a resposta certa.
///
/// | família | pior `‖∇f‖` medido | divisor |
/// |---|---:|---:|
/// | as quatro **exactas** (caixa · cilindro · extrusão · moldura) | `1,73` | `2` |
/// | as de parede **não-ortogonal** | `5,02` (prisma) | `4` |
#[must_use]
pub fn edge_shrink(p: &Primitive) -> f32 {
    // ⚠️ Só o PAR encolhe: cada recuo sozinho já está dentro do balde que o `fillet_inflates` paga.
    if round_of(p) <= 0.0 || chamfer_of(p) <= 0.0 {
        return 1.0;
    }
    if fillet_inflates(p) { 4.0 } else { 2.0 }
}

/// Há **duas** maneiras de arredondar uma aresta convexa neste módulo, e elas têm campos diferentes:
///
/// - **encolher uma distância EXATA e deslocá-la** (`box_raw`, `cylinder_raw`): a dilatação de uma
///   distância exata é o corpo com os cantos redondos, e o campo continua `1`-Lipschitz;
/// - **interseção ARREDONDADA** (`Blended::Exact`): é a única saída quando as paredes não são
///   ortogonais — e ela **infla**. Medido, num cone de declive `0,47`: `‖∇f‖ = 1,1943`, que é
///   exatamente o `√(1 − cos φ)` do canto de ângulo interno `φ`.
///
/// ⚠️ **A marcha TEM de saber**, e é a mesma lei que o report do Enio de 2026-08-29 pagou: o
/// o tecto de `‖∇f‖` lia a mistura do **grupo** e não a de cada forma, o passo ficava em `1,0` sobre
/// um campo de `1,17`, e o raio atravessava a superfície. Uma primitiva que arredonda por interseção
/// é **outro** produtor da mesma inflação — e o `NodeKind::Leaf` valia `0` para todos.
///
/// ⚠️ **Lista FECHADA**: uma primitiva nova é erro de compilação aqui, e quem a escrever tem de
/// dizer por que porta ela arredonda. `false` quando o raio é zero — é assim que o produto exprime
/// *«aresta viva»*, e é o estado que não pode custar passo nenhum.
#[must_use]
pub fn fillet_inflates(p: &Primitive) -> bool {
    let (r, c) = (round_of(p), chamfer_of(p));
    match p {
        // ⭐ **As QUATRO exactas**: a fonte encolhe e o deslocamento repõe, com o `length` do canto a
        // fazer o arredondamento de verdade — e as peças delas são **ortogonais** (as três lajes de
        // uma caixa, a parede e a tampa de um cilindro).
        //
        // ⭐⭐⭐ **NEM o chanfro NEM o par as inflam** — medido `1,0000` nas quatro colunas do censo
        // (viva · só filete · só chanfro · o par), a `ε = 1e-5`.
        //
        // ⛔ **A 1.ª versão desta wave dizia que o par inflava, e isso era um defeito da CONSTRUÇÃO,
        // não uma propriedade da forma:** ela misturava (`intersection(f, plano, Exact(round))`,
        // encaixado), e cada nível encaixado soma um quadrado na lei de Cauchy–Schwarz — medido
        // `‖∇f‖ = 1,7306` numa caixa, que é `√3`. Com **encolher-chanfrar-deslocar** não há mistura
        // nenhuma, e um `max` de 1-Lipschitz é 1-Lipschitz. *O preço não era da feature: era de como
        // ela estava escrita.*
        // ⚠️ **A gaiola entra aqui**: ela é a união de três caixas, cada uma pela receita da caixa,
        // e o `min` de uma união não infla.
        Primitive::Box { .. }
        | Primitive::Cylinder { .. }
        | Primitive::Extrude { .. }
        | Primitive::BoxFrame { .. } => false,
        // ⭐⭐⭐ **As de parede NÃO-ORTOGONAL: QUALQUER um dos dois recuos infla** (2026-08-30).
        //
        // ⛔ A 1.ª redacção desta wave dizia que o chanfro sozinho nunca inflava, e o censo
        // refutou-a com o número que este arquivo já tinha escrito: **`1,1943` no cone**, o
        // `√(1 − cos φ)` do canto. *O plano do chanfro herda o ângulo das duas faces que ele corta* —
        // e a demonstração «um `max` de 1-Lipschitz é 1-Lipschitz» só vale enquanto as normais são
        // ortogonais, que é precisamente o que uma parede inclinada não é.
        Primitive::Cone { .. }
        | Primitive::Prism { .. }
        | Primitive::Wedge { .. }
        | Primitive::Star { .. }
        | Primitive::Octahedron { .. }
        | Primitive::CutSphere { .. }
        | Primitive::HollowDome { .. }
        | Primitive::SolidAngle { .. }
        | Primitive::Gear { .. }
        | Primitive::Cross { .. }
        | Primitive::Heart { .. }
        | Primitive::Moon { .. }
        | Primitive::Drop { .. }
        | Primitive::Pie { .. }
        | Primitive::Trapezoid { .. }
        | Primitive::Vesica { .. }
        | Primitive::TorusArc { .. } => r != 0.0 || c != 0.0,
        // ⚠️ **Lista FECHADA**: uma primitiva nova é erro de compilação aqui, e quem a escrever tem
        // de dizer se as peças dela são ortogonais.
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => false,
    }
}

/// A razão apótema/circunraio de um polígono regular de `n` lados — `cos(π/n)`.
///
/// ⚠️ **Uma função e não um literal por forma**: ela tem três leitores (o limite do filete acima, o
/// tamanho característico e a fórmula do campo), e a mesma conta escrita três vezes é a lei escrita
/// em três sítios, que este módulo já pagou.
#[must_use]
pub fn apothem_ratio(sides: u32) -> f32 {
    (std::f32::consts::PI / sides.max(crate::MIN_PRISM_SIDES) as f32).cos()
}

/// Até onde o filete de um [`Primitive::Cone`] pode ir — ver a nota em [`round_limit`].
#[must_use]
pub fn cone_round_limit(bottom: f32, top: f32, half_height: f32) -> f32 {
    let a = (bottom + top) * 0.5;
    let m = (top - bottom) / (2.0 * half_height);
    half_height.min(a / (1.0 + m * m).sqrt())
}

/// Até onde uma dimensão pode ir, e **de que natureza é esse limite**.
///
/// ⚠️ **Chamou-se `RadiusBound` até 20/08**, e o nome deixou de dizer a verdade no dia em que as
/// outras dimensões ficaram editáveis (a largura de uma caixa tem um alcance de gesto tanto quanto
/// um filete tem uma parede). *Um nome que descreve o primeiro uso passa a mentir no segundo.*
///
/// ⚠️ A distinção não é decorativa e por isso está no tipo, em vez de num comentário: um limite de
/// **validade** é uma parede (o documento recusa), e um de **escala** é uma sugestão (a forma
/// continua correta, só deixa de ser útil). Um controle que os pintasse igual mentiria numa das
/// duas direções — ou proibiria o que é legítimo, ou ofereceria o que vai ser recusado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bound {
    /// O documento **recusa** acima disto.
    Hard(f32),
    /// Não há limite de validade. Este é o alcance **útil**, derivado do tamanho da peça.
    Soft(f32),
    /// ⭐ **A ponta é a própria REPRESENTAÇÃO.** Um ângulo canónico não passa de meia volta, e um
    /// número maior não é recusado nem cortado: ele é **renomeado** para o sítio equivalente dentro
    /// da faixa. Nem o documento nem a vista escolhem isto — ver
    /// [`crate::xform::set_rotation_degree`].
    Wrap(f32),
}

impl Bound {
    #[must_use]
    pub fn value(self) -> f32 {
        match self {
            Bound::Hard(v) | Bound::Soft(v) | Bound::Wrap(v) => v,
        }
    }
}

/// ⭐ As duas tabelas por-forma — ver [`radius_tables`].
#[path = "radius_tables.rs"]
mod radius_tables;
pub use radius_tables::{bounding_radius, characteristic_size};
