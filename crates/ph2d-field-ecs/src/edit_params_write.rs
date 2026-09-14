//! ⭐⭐ **AS CHAVES DO PAINEL, E A ESCRITA POR CHAVE** — a metade de [`super::edit_params`] que
//! **escreve**.
//!
//! ⚠️ **Por que ela vive num ficheiro próprio** (2026-09-10): o `edit_params.rs` passou o tecto de
//! `700` LOC. O corte é o mesmo que o `edit_pose`/`edit_verb` desta crate já usam — *uma
//! responsabilidade por ficheiro* —, e aqui a fronteira é **ler contra escrever**: o irmão devolve
//! o que a forma TEM (`params_of`, `radius_bound`, `mods_of`), este resolve uma **chave i18n** e
//! escreve o número nela.
//!
//! ⛔ A cura de um tecto de LOC é **cortar por responsabilidade**, nunca subir uma entrada de
//! tolerância (a lei da casa: *split, never allowlist*).

use crate::{FieldError, FieldMods, FieldNode, FieldPose};
use bevy_ecs::prelude::*;
use ph2d_field::xform::set_rotation_degree;
use ph2d_field::{NodeShape, Param, set_shape_radius};

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
        // ⭐⭐⭐ **UM NÚMERO DO MATERIAL** (`docs/Render3d/05`) — e escrever aqui **MATERIALIZA** o
        // componente, como o [`Param::Joint`] materializa o verbo.
        //
        // ⚠️ **A ausência do componente quer dizer «o de omissão»**, e é por isso que a escrita
        // começa por ele em vez de recusar: um artista que arrasta a rugosidade de uma forma que
        // nunca teve material não está a pedir um erro — está a pedir um material.
        //
        // ⚠️ **A posição é conferida pela PORTA do componente** ([`FieldMaterial::set`]), e não por
        // um `k < 5` escrito aqui: dois limites divergiriam no dia em que um sexto número entrasse,
        // e o sintoma seria uma linha pintada que a escrita recusa em silêncio.
        Param::Material(k) => {
            let mut m = world
                .get::<crate::FieldMaterial>(entity)
                .copied()
                .unwrap_or_default();
            if !m.set(k, value) {
                return Err(FieldError::BadRoot);
            }
            let Ok(mut e) = world.get_entity_mut(entity) else {
                return Err(FieldError::BadRoot);
            };
            e.insert(m);
            Ok(())
        }
        // ⭐⭐⭐ **UM NÚMERO DE UMA LUZ** — e a porta do componente é quem confere a posição, pela
        // razão exacta do [`Param::Material`] acima.
        //
        // ⚠️ **Aqui NÃO se materializa nada:** escrever numa luz que não tem `FieldLight` seria
        // transformar uma forma em lâmpada por engano. Uma luz é criada por um gesto próprio, e o
        // que não é luz recusa.
        Param::Light(k) => {
            let Some(mut l) = world.get_mut::<crate::FieldLight>(entity) else {
                return Err(FieldError::BadRoot);
            };
            if !l.set(k, value) {
                return Err(FieldError::BadRoot);
            }
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
            // ⚠️ O **carácter** da mistura sobrevive ao número novo, e a escada é a **mesma
            // porta** que o filete de um grupo usa ([`ph2d_field::Blend::with_amount`]).
            //
            // ⛔ **Isto era uma CÓPIA até 2026-08-28, e a cópia não conhecia o chanfro:** mudar o
            // raio de uma junta chanfrada transformava-a em filete, em silêncio. A afirmação de que
            // a porta era única estava escrita neste comentário e era **falsa** — quem a apanhou foi
            // a prova de mutação, com o mutante de `with_amount` a sobreviver por não haver ninguém
            // a chamá-la. *Uma lei escrita em dois sítios ainda não é uma lei.*
            crate::set_verb(
                world,
                entity,
                Some(op.with_blend(op.blend().with_amount(value))),
            )
        }
        // ⭐⭐⭐ **O SEGUNDO NÚMERO DE UMA JUNTA** (W145) — a meia-largura de um sulco ou de um
        // friso, o desequilíbrio de um chanfro.
        //
        // ⚠️ **Zero é RECUSADO aqui, ao contrário do [`Param::Joint`]**, e a assimetria é a
        // geometria: um raio de junção a zero é a aresta viva, que é um estado legítimo e o de
        // nascimento; uma meia-largura a zero é um canal sem largura — a junta perde o carácter sem
        // que o chip mude, e o artista fica com um controle que apagou a própria feição. A faixa
        // ([`ph2d_field::Span::Positive`]) já diz isso, e esta porta é quem o impõe.
        Param::Seam(slot) => {
            if value <= 0.0 {
                return Err(FieldError::NonPositive {
                    node: entity.to_bits() as u32,
                    what: "seam",
                });
            }
            match slot {
                // A mistura que o próprio grupo oferece aos filhos calados.
                0 => {
                    let Some(mut node) = world.get_mut::<FieldNode>(entity) else {
                        return Err(FieldError::BadRoot);
                    };
                    let NodeShape::Combine(op) = &mut node.shape else {
                        return Err(FieldError::BadRoot);
                    };
                    *op = op.with_blend(op.blend().with_second(value));
                    Ok(())
                }
                // A mistura com que ESTE nó se dobra nos irmãos — e escrever aqui **materializa o
                // verbo**, exactamente como o [`Param::Joint`].
                1 => {
                    let Some(op) = crate::verb_role(world, entity).and_then(|r| r.op()) else {
                        return Err(FieldError::BadRoot);
                    };
                    crate::set_verb(
                        world,
                        entity,
                        Some(op.with_blend(op.blend().with_second(value))),
                    )
                }
                _ => Err(FieldError::BadRoot),
            }
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
