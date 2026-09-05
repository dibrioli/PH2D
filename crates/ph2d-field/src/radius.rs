//! **O raio editável** — a promessa central do módulo, num arquivo.
//!
//! ⭐ *O raio do filete fica editável para sempre*, porque é parâmetro da operação e não geometria
//! assada. Tudo o que responde a *"que raio este nó tem, até onde ele vai, e o que acontece se eu o
//! mudar?"* vive aqui — e vive **uma vez**, porque a árvore tem dois donos possíveis (a arena
//! cozida e a cena ECS) e os dois têm de aplicar a mesma regra.

use super::radius_limit::round_limit;
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
                | Primitive::Arrow { round, .. }
                | Primitive::Chevron { round, .. }
                | Primitive::BentArrow { round, .. }
                | Primitive::Rhombus { round, .. }
                | Primitive::Tube { round, .. }
                | Primitive::CircleSegment { round, .. }
                | Primitive::SpeechRect { round, .. }
                | Primitive::SpeechOval { round, .. }
                | Primitive::Cloud { round, .. }
                | Primitive::Bolt { round, .. }
                | Primitive::Shield { round, .. }
                | Primitive::Tag { round, .. }
                | Primitive::Check { round, .. }
                | Primitive::Banner { round, .. }
                | Primitive::Brace { round, .. }
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
                | Primitive::Arrow { round, .. }
                | Primitive::Chevron { round, .. }
                | Primitive::BentArrow { round, .. }
                | Primitive::Rhombus { round, .. }
                | Primitive::Tube { round, .. }
                | Primitive::CircleSegment { round, .. }
                | Primitive::SpeechRect { round, .. }
                | Primitive::SpeechOval { round, .. }
                | Primitive::Cloud { round, .. }
                | Primitive::Bolt { round, .. }
                | Primitive::Shield { round, .. }
                | Primitive::Tag { round, .. }
                | Primitive::Check { round, .. }
                | Primitive::Banner { round, .. }
                | Primitive::Brace { round, .. }
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
                | Primitive::Arrow { round, .. }
                | Primitive::Chevron { round, .. }
                | Primitive::BentArrow { round, .. }
                | Primitive::Rhombus { round, .. }
                | Primitive::Tube { round, .. }
                | Primitive::CircleSegment { round, .. }
                | Primitive::SpeechRect { round, .. }
                | Primitive::SpeechOval { round, .. }
                | Primitive::Cloud { round, .. }
                | Primitive::Bolt { round, .. }
                | Primitive::Shield { round, .. }
                | Primitive::Tag { round, .. }
                | Primitive::Check { round, .. }
                | Primitive::Banner { round, .. }
                | Primitive::Brace { round, .. }
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
        // ⭐ **As QUATRO de peças ORTOGONAIS** (as três lajes de uma caixa, a parede e a tampa de um
        // cilindro). Sem chanfro elas arredondam por **encolher-e-deslocar** e o campo é
        // exactamente `1`-Lipschitz — é o caminho de omissão, e é intocado.
        //
        // ⛔⛔ **COM chanfro elas misturam como toda a gente, e INFLAM** — esta nota já disse o
        // contrário («`1,0000` nas quatro colunas»), e isso descrevia a construção
        // *encolher-chanfrar-deslocar* que o report do Enio de 2026-08-30 **retirou** (ela não
        // arredondava: deslocar um semiespaço dá outro semiespaço).
        //
        // ⚠️ **O que este `false` responde não é «não infla»: é «infla POUCO»** — o bastante para o
        // divisor `2` do [`edge_shrink`] chegar. Medido perto da superfície, com os dois recuos a
        // meio do limite (`‖∇f‖` do campo **cru**, isto é, já multiplicado de volta pelo divisor):
        //
        // | forma | cru | dividido por `2` |
        // |---|---:|---:|
        // | caixa | `1,574` | `0,787` |
        // | cilindro | `1,398` | `0,699` |
        // | moldura | `1,594` | `0,797` |
        //
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
        | Primitive::Arrow { .. }
        | Primitive::Chevron { .. }
        | Primitive::BentArrow { .. }
        | Primitive::Rhombus { .. }
        | Primitive::Tube { .. }
        | Primitive::CircleSegment { .. }
        | Primitive::SpeechRect { .. }
        | Primitive::SpeechOval { .. }
        | Primitive::Cloud { .. }
        | Primitive::Bolt { .. }
        | Primitive::Shield { .. }
        | Primitive::Tag { .. }
        | Primitive::Check { .. }
        | Primitive::Banner { .. }
        | Primitive::Brace { .. }
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
pub(crate) mod radius_tables;
pub use radius_tables::{bounding_radius, characteristic_size};
