//! **O raio editável** — a promessa central do módulo, num arquivo.
//!
//! ⭐ *O raio do filete fica editável para sempre*, porque é parâmetro da operação e não geometria
//! assada. Tudo o que responde a *"que raio este nó tem, até onde ele vai, e o que acontece se eu o
//! mudar?"* vive aqui — e vive **uma vez**, porque a árvore tem dois donos possíveis (a arena
//! cozida e a cena ECS) e os dois têm de aplicar a mesma regra.

use crate::{Blend, FieldDoc, FieldError, NodeId, NodeKind, NodeShape, Op, Primitive};

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
                | Primitive::Extrude { round, .. } => Some(*round),
                _ => None,
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
                | Primitive::Extrude { round, .. } => Some(*round),
                _ => None,
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
            let blend = match op.blend() {
                // O caráter ORGÂNICO é uma escolha de produto e sobrevive a mudar o número;
                // trocá-lo aqui seria decidir por quem só mexeu num slider.
                Blend::Organic { .. } => Blend::Organic { k: radius },
                _ if radius <= 0.0 => Blend::Sharp,
                _ => Blend::Exact { radius },
            };
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
                | Primitive::Extrude { round, .. } => *round = radius,
                // Inalcançável: `round_limit` já devolveu `None` para estas acima.
                Primitive::Sphere { .. } | Primitive::Torus { .. } | Primitive::Revolve { .. } => {}
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
        Primitive::Sphere { .. } | Primitive::Torus { .. } | Primitive::Revolve { .. } => None,
    }
}

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
    }
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
