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
/// ⚠️ **Os campos que não se autoram ficam no padrão da nodedef**: um material com **dez** das
/// **quinze** entradas não é um OpenPBR diferente, é o mesmo com dez entradas escolhidas. *Inventar
/// valores para as outras cinco seria escrever um material que ninguém pediu.*
///
/// ⚠️⚠️ **As contagens desta página CONTAM-SE daqui, nunca de cabeça** — as cinco que faltam são
/// `base_weight`, `base_diffuse_roughness`, `specular_weight`, `specular_color` e `specular_ior`, e
/// esta linha já esteve errada duas vezes no mesmo dia (*«sete»*, *«doze»*) por ser somada de
/// memória enquanto a lista crescia.
///
/// ⭐⭐ **A EMISSÃO entrou em 2026-09-14** (`docs/Render3d/05` §20), e a razão é o inverso da regra
/// acima: ela **já era paga** — a [`ph2d_material::Surface::emission`] corria por amostra e somava
/// `[0,0,0]`, `4,6 %` do relógio de sombreamento — e nenhum controlo lhe chegava. *Uma capacidade
/// viva sem botão nenhum é o defeito de que o §5.1 do `CLAUDE.md` fala, não uma poupança.*
#[must_use]
pub fn surface_of(m: FieldMaterial) -> ph2d_material::Surface {
    ph2d_material::OpenPbr {
        base_weight: m.base_weight,
        base_color: m.base_color,
        base_diffuse_roughness: m.base_diffuse_roughness,
        base_metalness: m.metalness,
        specular_weight: m.specular_weight,
        specular_color: m.specular_color,
        specular_roughness: m.roughness,
        specular_ior: m.specular_ior,
        coat_weight: m.coat,
        coat_color: m.coat_color,
        coat_roughness: m.coat_roughness,
        coat_ior: m.coat_ior,
        coat_darkening: m.coat_darkening,
        emission_luminance: m.emission,
        emission_color: m.emission_color,
        subsurface_weight: m.subsurface_weight,
        subsurface_color: m.subsurface_color,
        subsurface_radius: m.subsurface_radius,
        subsurface_radius_scale: m.subsurface_radius_scale,
        subsurface_scatter_anisotropy: m.subsurface_scatter_anisotropy,
        // ⚠️ O booleano viaja como número porque a tabela do painel é de `f32` — ver
        // [`ph2d_field_ecs::FieldMaterial::thin_walled`].
        geometry_thin_walled: m.thin_walled > 0.5,
    }
    .prepare()
}

/// ⭐⭐⭐ **A TRAVESSIA sRGB → LINEAR DE UMA COR AUTORADA, e o seu par** — num sítio só.
///
/// # ⚠️ Por que é uma porta, e não duas linhas onde cada uma é precisa
///
/// O documento guarda as cores em **linear** (é isso que o OpenPBR integra) e o selector de cor da
/// casa fala **sRGB8** (é isso que um humano escolhe). A conversão é precisa em **dois** sítios
/// distantes — a construção da linha do painel ([`crate::scene_panel::param_rows`]) e o dreno do
/// pedido ([`crate::scene_intents`]) —, e escrita duas vezes seriam duas curvas: o dia em que uma
/// ganhasse um `clamp` ou um arredondamento diferente, o ida-e-volta deixaria de fechar e a cor
/// **derivaria a cada abertura do selector**, um passo de undo de cada vez.
///
/// ⚠️ **E o ida-e-volta TEM de fechar ao bit**, porque o painel pergunta *«mudou?»* comparando
/// bytes: uma cor que não voltasse ao mesmo byte pediria uma edição **por quadro** enquanto o
/// selector estivesse aberto. É isso que o gate
/// [`the_round_trip_through_the_document_is_exact`](crate::materials::colour_row_tests::the_round_trip_through_the_document_is_exact)
/// mede, sobre os 256 bytes.
///
/// ⛔ **A curva não é local:** ela é a do [`ph2d_color::srgb`], que é a mesma que o resto do app usa.
///
/// ⚠️⚠️ **O nome deixou de dizer «base» em 14/09**, e isso é a lei e não arrumação: com o brilho
/// próprio (§20) há **duas** cores autoradas a atravessar aqui, e uma porta chamada `base_color_*`
/// convida a segunda a escrever a conversão outra vez ao lado. *Uma lei escrita em dois sítios ainda
/// não é uma lei — só uma PORTA é.*
#[must_use]
pub fn colour_srgb8(linear: [f32; 3]) -> [u8; 3] {
    linear.map(ph2d_color::srgb::linear_to_srgb_byte)
}

/// O outro sentido de [`colour_srgb8`] — o que o artista apontou, no espaço do documento.
#[must_use]
pub fn colour_from_srgb8(srgb: [u8; 3]) -> [f32; 3] {
    srgb.map(ph2d_color::srgb::srgb_to_linear_byte)
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

/// ⭐⭐⭐ **A LINHA DA COR** — os três canais dobrados numa amostra (Enio, 2026-09-14). Irmão por
/// assunto: ele mede a ponte painel↔documento, não a tabela de materiais.
#[cfg(test)]
#[path = "colour_row_tests.rs"]
mod colour_row_tests;

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

/// ⏱️⭐ **A EMISSÃO** — o que um brilho pinta e o que a chamada custa. Irmão por assunto: ele mede
/// uma capacidade do motor que o painel ainda não alcança.
#[cfg(test)]
#[path = "emission_tests.rs"]
mod emission_tests;

/// ⏱️⭐ **O VERNIZ** — quais dos cinco números dele movem o pixel. Irmão por assunto do
/// [`emission_tests`], e pela mesma razão: ele mede uma capacidade do motor antes de ela ter botão.
#[cfg(test)]
#[path = "coat_tests.rs"]
mod coat_tests;

/// ⏱️⭐ **AS CINCO QUE SOBRAM** do OpenPBR — a sonda que mede se elas ganham linha.
#[cfg(test)]
#[path = "base_specular_tests.rs"]
mod base_specular_tests;
