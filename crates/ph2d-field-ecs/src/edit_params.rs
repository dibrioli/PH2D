//! ⭐ **AS PERGUNTAS E OS NÚMEROS de um nó** — o que a Hierarquia enumera e o que o painel edita.
//!
//! ⚠️ Nenhuma regra é inventada aqui: o raio muda por [`ph2d_field::set_shape_radius`] e o teto sai
//! de [`ph2d_field::round_limit`] / [`ph2d_field::characteristic_size`] — as **mesmas** funções que
//! a validação do documento cozido usa. Um painel que calculasse o próprio teto ofereceria valores
//! que a peça recusa, e o artista veria o controle parar sem explicação.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::world::World;
use ph2d_field::xform::set_rotation_degree;
use ph2d_field::{
    Bound, Dim, FieldError, NodeShape, Param, Span, Unary, Xform, characteristic_size, round_limit,
    set_shape_radius,
};

use crate::{FieldMods, FieldNode, FieldPose};

/// **A árvore em pré-ordem**, com a profundidade de cada nó — a mesma ordem e o mesmo aninhamento
/// que a Hierarquia mostra.
///
/// ⭐ É a ordem certa para o painel: uma lista que discordasse da Hierarquia obrigaria o artista a
/// manter dois mapas na cabeça da mesma peça.
#[must_use]
pub fn walk(world: &World, root: Entity) -> Vec<(Entity, u8)> {
    let mut out = Vec::new();
    let mut stack = vec![(root, 0u8)];
    while let Some((e, depth)) = stack.pop() {
        if world.get::<FieldNode>(e).is_none() {
            continue;
        }
        out.push((e, depth));
        if let Some(children) = world.get::<Children>(e) {
            // Invertido para o `pop` sair na ordem de `Children`.
            for c in children
                .iter()
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                stack.push((c, depth.saturating_add(1)));
            }
        }
    }
    out
}

/// O raio editável deste nó, ou `None` quando ele não tem nenhum.
#[must_use]
pub fn radius_of(world: &World, entity: Entity) -> Option<f32> {
    world.get::<FieldNode>(entity)?.shape.radius()
}

/// Até onde esse raio pode ir, e **de que natureza é o limite**.
///
/// - Numa **primitiva** é uma parede ([`Bound::Hard`]): acima dela a forma deixa de existir e
///   o campo deixa de ser uma distância.
/// - Numa **operação** não há limite de validade nenhum ([`Bound::Soft`]) — o campo continua
///   correto com qualquer raio. O que existe é *escala*: um filete maior do que a menor peça que ele
///   junta engole-a. O número vem daí.
#[must_use]
pub fn radius_bound(world: &World, entity: Entity) -> Option<Bound> {
    match &world.get::<FieldNode>(entity)?.shape {
        NodeShape::Leaf(p) => round_limit(p).map(Bound::Hard),
        NodeShape::Combine(_) => Some(Bound::Soft(subtree_scale(world, entity))),
        // Uma escultura não tem raio autorado: a aresta dela é a malha.
        NodeShape::Sampled { .. } => None,
    }
}

/// A menor peça sob um nó, **com a escala da cadeia acumulada**.
///
/// ⚠️ A escala acumula de propósito: um cilindro de 0,1 dentro de um grupo escalado 3× mede 0,3 na
/// peça, e é esse o número que dá sentido a um raio de mistura. Usar só a escala do próprio nó
/// (como a versão de arena fazia, onde não havia cadeia) subestimaria a peça em cada nível de
/// agrupamento.
fn subtree_scale(world: &World, root: Entity) -> f32 {
    let mut best = f32::INFINITY;
    let mut stack = vec![(root, 1.0f32)];
    while let Some((e, acc)) = stack.pop() {
        let Some(node) = world.get::<FieldNode>(e) else {
            continue;
        };
        let acc = acc * world.get::<FieldPose>(e).map_or(1.0, |p| p.xform.scale);
        match &node.shape {
            NodeShape::Leaf(p) => best = best.min(characteristic_size(p) * acc),
            // ⚠️ A escala característica de uma escultura é a caixa dela, e a caixa vive no campo
            // amostrado, que o mundo não conhece. Não contribuir é a resposta certa quando há outra
            // peça a dar a escala — e o item aberto quando ela for a única.
            NodeShape::Sampled { .. } => {}
            NodeShape::Combine(_) => {
                if let Some(children) = world.get::<Children>(e) {
                    for c in children.iter().copied().collect::<Vec<_>>() {
                        stack.push((c, acc));
                    }
                }
            }
        }
    }
    if best.is_finite() && best > 0.0 {
        best
    } else {
        1.0
    }
}

/// ⭐ **Todos os números autorados de um nó**, na ordem em que o painel os mostra.
///
/// Posição · rotação · escala (só onde ela não compete com nada) · dimensões da forma.
///
/// ⚠️ **A ordem é a do Inspector de objeto que todo modelador tem** (posição, rotação, escala, e só
/// depois o que a forma mede). Ela não é decorativa: os três primeiros trios existem em **todo** nó,
/// então uma peça inteira lê-se com o olho no mesmo sítio de linha para linha.
///
/// ⚠️ **A escala aparece só numa OPERAÇÃO.** Numa folha, o tamanho visível são as dimensões — e
/// mostrar as duas coisas daria ao artista dois controles para a mesma coisa, sem forma de saber
/// qual o próximo gesto mexe. Ver [`ph2d_field::scale_primitive`], que é o outro lado da mesma
/// decisão.
#[must_use]
pub fn params_of(world: &World, entity: Entity) -> Vec<(Param, Dim)> {
    let Some(node) = world.get::<FieldNode>(entity) else {
        return Vec::new();
    };
    let pose = world
        .get::<FieldPose>(entity)
        .map_or(Xform::IDENTITY, |p| p.xform);
    let degrees = ph2d_field::xform::rotation_degrees(pose);
    let mut out: Vec<(Param, Dim)> = (0..3u8)
        .map(|k| {
            (
                Param::Pos(k),
                Dim {
                    key: POS_KEYS[k as usize],
                    value: pose.translation[k as usize],
                    // ⚠️ Uma posição não tem parede **nem piso**: a origem não é um canto do mundo.
                    // Quem fecha as duas pontas é a vista (ver `Span::Free`).
                    span: Span::Free,
                },
            )
        })
        .chain((0..3u8).map(|k| {
            (
                Param::Rot(k),
                Dim {
                    key: ROT_KEYS[k as usize],
                    value: degrees[k as usize],
                    // ⚠️ **A mesma porta que recusa a escrita** decide se há faixa: um eixo que a
                    // trava de cardan tirou do mapa não é um slider curto, é um facto sem controle.
                    span: if ph2d_field::xform::rotation_axis_is_free(pose, k) {
                        Span::Turn(ROT_SPAN_DEG[k as usize])
                    } else {
                        Span::Locked
                    },
                },
            )
        }))
        .collect();
    match &node.shape {
        // Uma escultura tem pose e mais nada: o que ela é vive na malha.
        NodeShape::Sampled { .. } => {}
        NodeShape::Combine(_) => {
            out.push((
                Param::Scale,
                Dim {
                    key: "field.dim.scale",
                    value: pose.scale,
                    span: Span::Positive,
                },
            ));
            // A única dimensão de uma operação é o raio da mistura, e ela entra pela porta de
            // sempre — `Dim(0)`, que o `set_param` reencaminha.
            if let Some(v) = node.shape.radius() {
                out.push((
                    Param::Dim(0),
                    Dim {
                        // ⭐ **"Joint" e não "Fillet" desde a W98**, e é o rótulo a apanhar o modelo:
                        // depois do verbo por forma, o raio de um grupo é o **raio de junção
                        // padrão** — o que as formas caladas usam. É a mesma grandeza que a linha
                        // [`Param::Joint`] de cada filho escreve, e duas palavras para uma grandeza
                        // é o que faz o artista pensar que são duas.
                        key: "field.dim.joint",
                        value: v,
                        // ⚠️ **Uma mistura não tem parede**: o campo continua a ser uma distância com
                        // qualquer raio ([`radius_bound`] devolve sempre `Soft` aqui).
                        //
                        // ⏸️ O `radius_bound` sabe um alcance mais **apertado** do que o da vista — a
                        // menor peça sob o nó, que é o raio a partir do qual a mistura a engole.
                        // Trocá-lo pelo da vista muda o **tato** do arrasto desta linha e de mais
                        // nenhuma, e isso é número do Enio, com a peça à frente.
                        span: Span::Positive,
                    },
                ));
            }
        }
        NodeShape::Leaf(p) => out.extend(
            ph2d_field::dims(p)
                .into_iter()
                .enumerate()
                .map(|(i, d)| (Param::Dim(i as u16), d)),
        ),
    }
    // ⭐⭐⭐ **O RAIO DA JUNÇÃO desta forma** (W98) — logo depois do que ela mede, porque é sobre o
    // **encontro** dela com o resto e não sobre o que ela é.
    //
    // ⚠️ **A pergunta é feita ao PAPEL, e não à forma** ([`crate::verb_role`]): a base não se junta
    // a nada, e a raiz da peça também não. Oferecer a linha ali seria um controle que escreve num
    // verbo que ninguém lê — a affordance que mente, e a mesma lei da W34 que a fileira do painel
    // já honra.
    //
    // ⭐ **E ela aparece também para quem HERDA**, com o valor herdado: *"quero a boca deste furo
    // mais macia"* não pode exigir que o artista entenda o modelo do verbo primeiro. Escrever
    // materializa (ver [`Param::Joint`]), e o chip `Inherit` apaga-se à vista.
    if let Some(op) = crate::verb_role(world, entity).and_then(|r| r.op()) {
        out.push((
            Param::Joint,
            Dim {
                key: "field.dim.joint",
                value: op.blend().amount(),
                // ⚠️ **Sem parede**, como a do grupo: o campo continua a ser uma distância com
                // qualquer raio de mistura. Quem fecha o teto é a vista.
                span: Span::Positive,
            },
        ));
    }
    // ⭐⭐ **A RESOLUÇÃO do contorno vivo** (W55) — logo depois do que a forma mede, e antes do que
    // se fez a ela.
    //
    // ⚠️ **A pergunta é feita ao VÍNCULO, não à forma.** Um `Extrude` cujo desenho foi largado é uma
    // extrusão normal, com as mesmas dimensões e sem esta linha — e é o componente ausente que o
    // diz. Perguntar `matches!(shape, Extrude | Revolve)` ofereceria um controle que não tem onde
    // escrever.
    if let Some(src) = world.get::<crate::FieldProfileSource>(entity) {
        out.push((
            Param::Resolution,
            Dim {
                key: "field.dim.resolution",
                value: src.level as f32,
                // ⭐ **Uma CONTAGEM**: o passo do arrasto é 1, não há meio nível, e as duas pontas
                // são do documento — o piso é o joelho que a W54 mediu e o teto é o custo do
                // traçado assente (ver [`ph2d_field::MAX_PROFILE_RESOLUTION`]).
                span: Span::Count {
                    max: ph2d_field::MAX_PROFILE_RESOLUTION,
                },
            },
        ));
    }
    // ⭐ **Os modificadores vêm por ÚLTIMO**, e é a ordem em que eles correm: primeiro o que a forma
    // é, depois o que se fez a ela. Uma linha de casca acima da largura da caixa leria como se a
    // parede fosse uma propriedade da caixa, e ela é uma operação sobre o resultado.
    // ⚠️ **Um modificador pode ter VÁRIOS números** (uma matriz tem contagem e espaçamento) — e
    // pode não ter nenhum (o espelho). O `flat_map` é o que exprime as duas pontas sem um caso
    // especial para cada.
    out.extend(
        mods_of(world, entity)
            .into_iter()
            .enumerate()
            .flat_map(|(slot, m)| {
                m.dims().into_iter().enumerate().map(move |(field, d)| {
                    (
                        Param::Mod {
                            slot: slot as u16,
                            field: field as u8,
                        },
                        d,
                    )
                })
            }),
    );
    out
}

/// ⭐ **A pilha de modificadores deste nó** — vazia quando ele não tem nenhum.
#[must_use]
pub fn mods_of(world: &World, entity: Entity) -> Vec<Unary> {
    world
        .get::<FieldMods>(entity)
        .map(|m| m.stack.clone())
        .unwrap_or_default()
}

/// ⭐ **Acrescenta um modificador ao nó**, no ponto neutro da natureza dele.
///
/// ⚠️ **O tamanho de nascimento vem da PEÇA**, não de uma constante: uma casca é uma fração da
/// menor peça sob o nó ([`subtree_scale`]), porque só quem vê a peça sabe o que é fino nela. Um
/// número absoluto seria invisível numa peça grande e engoliria uma pequena — nos dois casos o
/// artista conclui que o botão não fez nada.
///
/// Devolve `false` sem escrever nada quando a entidade não é um nó, ou é uma **escultura** (ver
/// abaixo) — e é o `false` que deixa quem chamou dizê-lo ao artista.
pub fn add_mod(world: &mut World, entity: Entity, kind: ph2d_field::UnaryKind) -> bool {
    let Some(node) = world.get::<FieldNode>(entity) else {
        return false;
    };
    // ⭐ **Uma escultura NÃO aceita modificadores, e a recusa é aqui** (W25).
    //
    // ⚠️ **A regra já existia — no documento** ([`ph2d_field::FieldError::ModsOnSampled`]) — e
    // ninguém a consultava antes de escrever. O componente entrava no mundo, o cozimento do quadro
    // seguinte recusava o documento inteiro, e a peça **inteira** desaparecia da tela com a
    // Hierarquia intacta. *Uma invariante que só o validador conhece é uma invariante que a UI
    // descobre partindo-se.*
    if matches!(node.shape, NodeShape::Sampled { .. }) {
        return false;
    }
    let born = Unary::born(kind, subtree_scale(world, entity));
    let mut e = world.entity_mut(entity);
    if let Some(mut m) = e.get_mut::<FieldMods>() {
        m.stack.push(born);
    } else {
        e.insert(FieldMods { stack: vec![born] });
    }
    true
}

/// ⭐ **Tira do nó o PRIMEIRO modificador daquela natureza**, e diz se tirou algum.
///
/// ⚠️ O primeiro, e não todos: a pilha é ordenada, e apagar em bloco tiraria um que o artista pôs
/// de propósito depois de outro. Devolve `false` quando não havia nenhum — é o que faz o botão ser
/// um interruptor honesto.
pub fn remove_mod(world: &mut World, entity: Entity, kind: ph2d_field::UnaryKind) -> bool {
    let Some(mut m) = world.get_mut::<FieldMods>(entity) else {
        return false;
    };
    let Some(i) = m.stack.iter().position(|u| u.kind() == kind) else {
        return false;
    };
    m.stack.remove(i);
    // ⚠️ **A pilha vazia sai do nó.** Um componente presente e vazio não muda a forma, mas muda os
    // BYTES — e o undo compara bytes: acrescentar e tirar um modificador deixaria a peça diferente
    // de si mesma, e o desfazer teria um passo a mais do que o artista fez.
    let empty = m.stack.is_empty();
    if empty {
        world.entity_mut(entity).remove::<FieldMods>();
    }
    true
}

/// As chaves i18n dos três eixos da posição.
const POS_KEYS: [&str; 3] = ["field.dim.pos_x", "field.dim.pos_y", "field.dim.pos_z"];

/// As chaves i18n dos três ângulos.
const ROT_KEYS: [&str; 3] = ["field.dim.rot_x", "field.dim.rot_y", "field.dim.rot_z"];

/// ⭐ **A faixa canónica de cada posição do trio, em graus** — e ela **não é a mesma nas três**.
///
/// ⚠️ Não é uma escolha de UI: é o alcance da própria representação. Num XYZ Euler o ângulo do
/// **meio** vive em `[−90°, 90°]` e os de fora em `(−180°, 180°]`, e é por isso que a linha do meio
/// tem metade do curso. Dar-lhe 180 foi o defeito da primeira versão: o slider oferecia sítios que a
/// leitura seguinte **renomeava**, e num arrasto isso vira um ciclo de dois (ver
/// [`ph2d_field::xform::set_rotation_degree`], que é onde a lei e a medição estão).
///
/// ⚠️ **Prender o do meio não perde orientação nenhuma** — toda orientação tem um trio canónico com
/// `|β| ≤ 90°`. Perde-se o *nome*: «Y = 120» escreve-se `X = 180 · Y = 60 · Z = 180`.
const ROT_SPAN_DEG: [f32; 3] = [180.0, 90.0, 180.0];

/// ⭐ **Escreve um número autorado de um nó**, ou recusa — a porta ÚNICA do painel.
///
/// # Errors
/// Ver [`set_dim`] e [`ph2d_field::set_shape_radius`]. [`FieldError::BadRoot`] se a entidade não é
/// um nó, e [`FieldError::NonPositive`] para uma escala não-positiva.
pub fn set_param(
    world: &mut World,
    entity: Entity,
    param: Param,
    value: f32,
) -> Result<(), FieldError> {
    if !value.is_finite() {
        return Err(FieldError::NonPositive {
            node: entity.to_bits() as u32,
            what: "param",
        });
    }
    match param {
        Param::Pos(k) if k < 3 => {
            let Some(mut pose) = world.get_mut::<FieldPose>(entity) else {
                return Err(FieldError::BadRoot);
            };
            pose.xform.translation[k as usize] = value;
            Ok(())
        }
        Param::Pos(_) => Err(FieldError::BadRoot),
        // ⚠️ **Escrever um ângulo lê os outros dois primeiro.** A pose guarda um quaternion, e três
        // ângulos são o nome canónico dele: pôr só um sem os companheiros seria construir uma
        // orientação a partir de um terço da informação. A lei — e o que ela renomeia — está em
        // [`set_rotation_degree`].
        Param::Rot(k) if k < 3 => {
            let Some(mut pose) = world.get_mut::<FieldPose>(entity) else {
                return Err(FieldError::BadRoot);
            };
            set_rotation_degree(&mut pose.xform, k, value);
            Ok(())
        }
        Param::Rot(_) => Err(FieldError::BadRoot),
        Param::Scale => {
            if value <= 0.0 {
                return Err(FieldError::NonPositive {
                    node: entity.to_bits() as u32,
                    what: "scale",
                });
            }
            let Some(mut pose) = world.get_mut::<FieldPose>(entity) else {
                return Err(FieldError::BadRoot);
            };
            pose.xform.scale = value;
            Ok(())
        }
        Param::Dim(i) => set_dim(world, entity, i as usize, value),
        // ⭐⭐⭐ **O RAIO DA JUNÇÃO** (W98) — e escrever aqui **MATERIALIZA o verbo**.
        //
        // ⚠️ Uma forma que herdava passa a ter o verbo por escrito, com o **mesmo** verbo que ela
        // já usava e o raio novo: *pedir um raio de junção próprio é pronunciar-se*. Sem esta
        // metade, arrastar a linha de uma forma calada escreveria no grupo — e mudaria as outras
        // caladas com ela, que é exactamente o defeito que a wave do verbo existe para curar.
        //
        // ⚠️ **Zero é a aresta VIVA**, e não uma recusa: é a mesma lei do `set_shape_radius`, onde o
        // raio zero é `Sharp` e não um erro. Negativo é que não existe.
        Param::Joint => {
            if value < 0.0 {
                return Err(FieldError::NonPositive {
                    node: entity.to_bits() as u32,
                    what: "joint",
                });
            }
            let Some(op) = crate::verb_role(world, entity).and_then(|r| r.op()) else {
                // A base não se junta a nada, e a raiz também não — a mesma recusa que faz a linha
                // não ser oferecida.
                return Err(FieldError::BadRoot);
            };
            let blend = if value > 0.0 {
                // ⚠️ O **carácter** da mistura sobrevive: quem já era orgânica continua orgânica,
                // e uma aresta viva acorda como `Exact` — é a lei que o `set_shape_radius` já
                // escreve para o filete de uma forma, e duas leis para o mesmo gesto divergiriam.
                match op.blend() {
                    ph2d_field::Blend::Organic { .. } => ph2d_field::Blend::Organic { k: value },
                    _ => ph2d_field::Blend::Exact { radius: value },
                }
            } else {
                ph2d_field::Blend::Sharp
            };
            crate::set_verb(
                world,
                entity,
                Some(match op {
                    ph2d_field::Op::Union(_) => ph2d_field::Op::Union(blend),
                    ph2d_field::Op::Intersection(_) => ph2d_field::Op::Intersection(blend),
                    ph2d_field::Op::Difference(_) => ph2d_field::Op::Difference(blend),
                }),
            )
        }
        // ⭐⭐ **O NÍVEL DE RESOLUÇÃO** (W55) — e escrever aqui não muda geometria nenhuma: muda a
        // **intenção**, e quem a converte é o recozimento do quadro seguinte.
        //
        // ⚠️ **A lei é a da contagem da matriz, copiada de propósito** ([`ph2d_field::Unary::set_dim`]):
        // abaixo de 1 **recusa** (não existe meio nível, e zero seria uma peça sem contorno), acima
        // do teto **limita em silêncio** — e o silêncio é visível, porque o número que a linha mostra
        // vem do componente e muda à vista. Uma segunda lei aqui daria dois tatos ao mesmo tipo de
        // controle no mesmo painel.
        Param::Resolution => {
            let Some(mut src) = world.get_mut::<crate::FieldProfileSource>(entity) else {
                return Err(FieldError::BadRoot);
            };
            if value < 1.0 {
                return Err(FieldError::NonPositive {
                    node: entity.to_bits() as u32,
                    what: "resolution",
                });
            }
            src.level = (value.round() as u32).min(ph2d_field::MAX_PROFILE_RESOLUTION);
            Ok(())
        }
        // ⚠️ A escrita passa pela porta do próprio modificador (`Unary::set_value`), que é a mesma
        // que a validação do documento usa — ver a nota lá sobre duas listas de regras.
        Param::Mod { slot, field } => {
            let id = entity.to_bits() as u32;
            let Some(mut m) = world.get_mut::<FieldMods>(entity) else {
                return Err(FieldError::BadRoot);
            };
            let Some(target) = m.stack.get_mut(slot as usize) else {
                return Err(FieldError::BadRoot);
            };
            let previous = *target;
            target.set_dim(id, field, value).inspect_err(|_| {
                // Uma recusa deixa o nó **como estava** — a invariante do módulo.
                *target = previous;
            })
        }
    }
}

/// ⭐ **O que este nó mede** — vazio para uma operação, que não tem forma própria.
#[must_use]
pub fn dims_of(world: &World, entity: Entity) -> Vec<ph2d_field::Dim> {
    match &world.get::<FieldNode>(entity).map(|n| &n.shape) {
        Some(NodeShape::Leaf(p)) => ph2d_field::dims(p),
        _ => Vec::new(),
    }
}

/// ⭐ **Escreve uma dimensão de um nó**, ou recusa — e uma recusa deixa-o **como estava**.
///
/// ⚠️ **Encolher uma forma encolhe o filete dela**, em silêncio mas à vista (ver
/// [`ph2d_field::set_dim`]). Aqui isso é feito **depois** da escrita e **antes** de devolver: um
/// filete que ficasse por limitar deixaria o nó inválido, e a invariante do módulo é *um nó que
/// existe está válido*.
///
/// ⚠️ Numa **operação** o índice 0 é o raio da mistura — é a única dimensão que ela tem, e é por
/// aqui que o painel a edita, com a mesma porta das outras.
///
/// # Errors
/// Ver [`ph2d_field::set_dim`]. [`FieldError::BadRoot`] se a entidade não é um nó.
pub fn set_dim(
    world: &mut World,
    entity: Entity,
    index: usize,
    value: f32,
) -> Result<(), FieldError> {
    let Some(mut node) = world.get_mut::<FieldNode>(entity) else {
        return Err(FieldError::BadRoot);
    };
    let id = entity.to_bits() as u32;
    match &mut node.shape {
        NodeShape::Sampled { .. } => Err(FieldError::BadRoot),
        // Uma operação tem uma dimensão só: o raio da mistura.
        NodeShape::Combine(_) if index == 0 => {
            let mut shape = node.shape.clone();
            set_shape_radius(&mut shape, id, value)?;
            node.shape = shape;
            Ok(())
        }
        NodeShape::Combine(_) => Err(FieldError::BadRoot),
        NodeShape::Leaf(p) => {
            let previous = p.clone();
            match ph2d_field::set_dim(p, id, index, value) {
                Ok(()) => {
                    ph2d_field::clamp_round(p);
                    Ok(())
                }
                Err(e) => {
                    *p = previous;
                    Err(e)
                }
            }
        }
    }
}
