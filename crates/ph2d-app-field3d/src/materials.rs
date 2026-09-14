//! ⭐⭐⭐ **A TABELA DE MATERIAIS DA PEÇA** — o que cada folha é à luz, e como um pixel a encontra.
//!
//! # As duas metades, e porque têm ritmos diferentes
//!
//! | metade | do que depende | quando se refaz |
//! |---|---|---|
//! | [`Table::owners`] | a **GEOMETRIA** (uma fita compilada por folha) | quando o documento muda |
//! | [`Table::surfaces`] | os **NÚMEROS** do material | quando um deles muda |
//!
//! ⚠️ **Separá-las não é arrumação, é preço:** compilar a fita de uma folha é um **JIT**, e arrastar
//! um slider de cor não muda geometria nenhuma. Uma tabela só, refeita sempre que qualquer das duas
//! mudasse, pagaria um JIT por folha a cada quadro de um arrasto de cor.
//!
//! ⚠️ **A ORDEM é a mesma nas duas**, e é a de [`ph2d_field_ecs::walk`] filtrada a folhas — a mesma
//! que o [`crate::pick`] usa. ⛔ Duas ordens pintariam cada peça com a cor da vizinha, sem erro
//! nenhum: *um índice válido nunca parece errado.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, NodeShape};
use ph2d_field_ecs::{FieldMaterial, FieldNode};

/// **O que o sombreamento precisa de saber sobre os materiais de uma peça.**
pub struct Table {
    /// De quem é cada ponto. `None` numa peça de uma folha só — ver [`Table::surfaces_for`].
    pub owners: Option<ph2d_field_eval::owners::Owners>,
    /// Um material por folha, na ordem das folhas. **Nunca vazio.**
    pub surfaces: Vec<ph2d_material::Surface>,
    /// Os números de que as [`Table::surfaces`] foram feitas — a chave que diz se elas envelheceram.
    pub authored: Vec<FieldMaterial>,
}

/// ⭐ **O OpenPBR de um material autorado** — a tradução, num sítio só.
///
/// ⚠️ **Os campos que não se autoram ficam no padrão da nodedef** (o verniz, a emissão, a difusa
/// especular): um material com três números não é um OpenPBR diferente, é o mesmo com três números
/// escolhidos. *Inventar valores para os outros doze seria escrever um material que ninguém pediu.*
#[must_use]
pub fn surface_of(m: FieldMaterial) -> ph2d_material::Surface {
    ph2d_material::OpenPbr {
        base_color: m.base_color,
        base_metalness: m.metalness,
        specular_roughness: m.roughness,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()
}

/// As folhas de uma peça, na ordem canónica: o par `(material autorado, documento de um nó posto no
/// mundo)`.
///
/// ⚠️ **A pose é a de MUNDO**, e não a local: quem avalia a folha avalia-a onde ela está. É a mesma
/// lei (e o mesmo erro evitado) do [`crate::pick::owners_under`].
fn leaves(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
) -> (Vec<FieldMaterial>, Vec<FieldDoc>) {
    ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter_map(|(e, _)| {
            let FieldNode {
                shape: NodeShape::Leaf(prim),
            } = world.get::<FieldNode>(e)?
            else {
                return None;
            };
            let placed = FieldDoc::new(
                vec![Node {
                    xform: ph2d_field_ecs::world_xform(world, e),
                    kind: NodeKind::Leaf(prim.clone()),
                    mods: Vec::new(),
                    verb: None,
                }],
                NodeId(0),
            )
            .ok()?;
            // ⚠️ **A ausência do componente é o material de OMISSÃO** — ver [`FieldMaterial`].
            Some((
                world.get::<FieldMaterial>(e).copied().unwrap_or_default(),
                placed,
            ))
        })
        .unzip()
}

impl Table {
    /// ⭐⭐⭐ **Constrói a tabela de uma peça**, com a geometria compilada.
    ///
    /// ⚠️ **`half_extent` e `side_px` só entram para a MARGEM** ([`ph2d_field_render::hit_tolerance`]):
    /// o ponto que esta tabela vai receber foi produzido por uma marcha, e a tolerância dela é o que
    /// diz quão fora da superfície ele pode estar. *Uma margem inventada aqui seria a segunda
    /// resposta, e a que envelhece.*
    #[must_use]
    pub fn build(
        world: &bevy_ecs::world::World,
        root: bevy_ecs::entity::Entity,
        half_extent: f32,
        side_px: f32,
    ) -> Self {
        let (authored, placed) = leaves(world, root);
        // ⭐ **Uma folha só não precisa de dono**, e não perguntar é exactamente o custo zero — é
        // isto que faz o quadro de uma peça simples continuar a ser o de sempre.
        let owners = (placed.len() > 1).then(|| {
            ph2d_field_eval::owners::Owners::new(
                &placed,
                &crate::smoke::sampled_registry(),
                ph2d_field_render::hit_tolerance(half_extent, side_px),
            )
        });
        let surfaces = if authored.is_empty() {
            // ⚠️ **Nunca vazia**: uma peça sem folha nenhuma (uma cena a ser apagada) ainda tem de
            // poder ser sombreada, e o `Surfaces::of` indexa `all[0]` como rede.
            vec![surface_of(FieldMaterial::default())]
        } else {
            authored.iter().copied().map(surface_of).collect()
        };
        Self {
            owners,
            surfaces,
            authored,
        }
    }

    /// ⭐⭐ **Re-traduz só os NÚMEROS**, sem tocar na geometria compilada — `true` se algo mudou.
    ///
    /// ⚠️ É esta metade que faz arrastar um slider de cor **não** custar um JIT por folha.
    pub fn refresh_authored(
        &mut self,
        world: &bevy_ecs::world::World,
        root: bevy_ecs::entity::Entity,
    ) -> bool {
        let (authored, _) = leaves(world, root);
        if authored == self.authored || authored.is_empty() {
            return false;
        }
        self.surfaces = authored.iter().copied().map(surface_of).collect();
        self.authored = authored;
        true
    }

    /// A vista que o [`ph2d_field_render::shade_render`] consome.
    #[must_use]
    pub fn surfaces_for(&self) -> ph2d_field_render::Surfaces<'_> {
        ph2d_field_render::Surfaces {
            all: &self.surfaces,
            owners: self.owners.as_ref(),
        }
    }
}

#[cfg(test)]
#[path = "materials_tests.rs"]
mod tests;

/// ⭐⭐⭐ **A TABELA SEGUE A PEÇA** — chamada uma vez por quadro, depois do cozimento.
///
/// `doc_mudou` decide qual das duas metades se refaz (ver [`Table`]): a geometria compilada só com
/// documento novo, os números sempre que alguém lhes tocar.
///
/// ⚠️ **E as duas largam o pedido guardado de todos os viewports**, como o `Smoke::set_look`: um
/// material novo sobre o traçado velho é o congelador que o doc do `Viewport::requested` descreve.
pub(crate) fn sync(sim: &mut ph2d_ecs::SimWorld, doc_mudou: bool) {
    // ⚠️ `&mut` para uma LEITURA porque `World::query` o exige — a mesma nota do
    // [`crate::scene::world_has_a_part`]. O empréstimo mutável acaba aqui, de propósito.
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).next().map(|(e, _)| e)
    };
    let Some(root) = root else {
        return;
    };
    let world = sim.world();
    crate::smoke::with_smoke(|s| {
        let (he, lado) = {
            let vp = s.vp();
            (
                vp.cam.half_extent,
                vp.area.map_or(480.0, |r| r.w.min(r.h).max(1.0)),
            )
        };
        let refez = match (&mut s.materials, doc_mudou) {
            (None, _) | (_, true) => {
                s.materials = Some(std::sync::Arc::new(Table::build(world, root, he, lado)));
                true
            }
            // ⚠️ **`get_mut` devolve `None` enquanto uma thread de traçado segura o `Arc`**, e isso
            // é a resposta certa: aquele traçado está a usar esta tabela **agora**. O quadro
            // seguinte apanha-a — e o artista vê a cor mudar um quadro depois, não nunca.
            (Some(t), false) => {
                std::sync::Arc::get_mut(t).is_some_and(|t| t.refresh_authored(world, root))
            }
        };
        if refez {
            s.forget_requests();
        }
    });
}
