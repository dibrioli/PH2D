//! ⭐⭐⭐ **PRENDER UMA IMAGEM AO ESQUELETO** — irmão do [`crate::skin_live`] pelo tecto de LOC, e o
//! corte é por RESPONSABILIDADE: ali mora o que uma FORMA faz; aqui, o que uma IMAGEM faz.
//!
//! ⚠️ **As duas mídias respondem à mesma lei e a pergunta que as separa é uma só** — *o que é um
//! ponto aqui*. Numa forma são as três metades de cada vértice; numa imagem são os vértices da
//! malha que este ficheiro constrói.
//!
//! ⚠️ A imagem já nascia com a **malha graduada pelas articulações**; foi a forma que ganhou o
//! equivalente em 2026-09-19 ([`crate::subdivisao`]).

use crate::skin_live::{skeleton_of, tendons_and_axes, world_of};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::SkinBind;

/// ⭐⭐⭐ **PRENDE UMA IMAGEM ao esqueleto** — a 2.ª mídia (ordem do dono, 2026-09-09).
///
/// A malha é traçada da própria tinta ([`crate::skin_image::mesh_from_rgba`]) e guardada
/// nos **bytes opacos** da [`SkinBind`] — ⭐ sem uma variante nova e sem tocar no schema, que é o
/// que o doc daquele campo prometia por escrito desde que ele existe.
///
/// ⚠️ **Os tendões saem da MESMA porta que os de uma forma** ([`tendons_for`]): as duas mídias
/// respondem à mesma pose ou o personagem parte-se ao meio.
///
/// ⚠️ **`pixels_per_meter` é o do PROJECTO** — o mesmo que o extract passa ao
/// `Sprite::resolve_anchor`. A régua da imagem lê a âncora resolvida ao prender e ao desenhar
/// ([`crate::skin_image::pixel_to_local`]); dois valores dariam uma âncora a cada gesto.
///
/// `false` quando não há esqueleto, quando a pose da imagem é singular, ou quando a tinta não dá
/// uma malha — e nos três casos **nada é escrito**, porque uma pele sem malha lá dentro não é uma
/// pele, é uma imagem prestes a sumir.
pub fn bind_image(
    sim: &mut SimWorld,
    e: Entity,
    rgba: &[u8],
    size_px: [u32; 2],
    pixels_per_meter: f32,
    opts: ph2d_poly2d::GridOptions,
    seed: Option<Entity>,
) -> bool {
    let ossos = skeleton_of(sim, seed);
    if ossos.is_empty() || sim.world().get_entity(e).is_err() {
        return false;
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⭐⭐⭐ **A MALHA NASCE SOBRE A CÉLULA QUE O QUAD MOSTRA, NUNCA SOBRE A FONTE INTEIRA** (F11,
    // 2026-09-17). ⚠️⚠️ **Antes disto uma FOLHA prendia-se errada em SILÊNCIO:** a malha era traçada
    // sobre a folha toda e o `pixel_to_local` espremia-a no quad de UMA célula — medido, uma folha
    // `4×1` desenhava `1 277` peças recortadas dos quatro quadros dentro do sítio de um, com a UV de
    // um só esticada por cima. *O 9-slice pelo menos avisava; este não dizia nada.*
    //
    // ⭐ A porta é a [`ph2d_render::SourceCells`], a MESMA que o extract lê para escolher a célula —
    // e com uma célula só (a sprite de sempre) ela devolve a imagem inteira, byte-a-byte.
    let Some(cells) = ph2d_render::SourceCells::of(
        size_px,
        sim.world().get::<ph2d_ecs::SpriteRegion>(e).map(|r| r.rect),
        sim.world()
            .get::<ph2d_ecs::SpriteGrid>(e)
            .map_or(1, |g| g.hframes),
        sim.world()
            .get::<ph2d_ecs::SpriteGrid>(e)
            .map_or(1, |g| g.vframes),
    ) else {
        return false;
    };
    let celula = cells.cell_px();
    // ⭐⭐⭐ **AS ARTICULAÇÕES GRADUAM A MALHA** (report do dono, 2026-09-10). Elas saem daqui e não
    // do leaf da geometria: só quem PRENDE sabe onde a dobra vai acontecer.
    //
    // ⚠️ **Em pixels da CÉLULA**, que é o espaço da malha — o `pixel_to_local` do desenho lê o
    // `mesh.size`, logo as duas pontas medem a mesma régua ou o adensamento cai fora da dobra.
    let focos = crate::skin_image::joints_in_image(sim, e, &ossos, celula, pixels_per_meter);
    let alfa = crate::skin_image::cell_alpha(rgba, size_px, &cells);
    let Some(malha) = crate::skin_image::mesh_from_alpha(&alfa, celula[0], celula[1], &focos, opts)
    else {
        return false;
    };
    let Some(shape_inv) = world_of(sim, e).inverse() else {
        return false;
    };
    let pares = tendons_and_axes(sim, &ossos, shape_inv);
    // ⭐⭐⭐ **OS PESOS DO PADRÃO-OURO, RESOLVIDOS AQUI** — uma vez, sobre a arte, ao prender.
    //
    // ⚠️ **O espaço é o da MALHA (pixels da imagem)**, e é por isso que os eixos atravessam a
    // régua `local → pixel`: o solver mede distâncias sobre a própria arte, e no espaço da forma
    // uma sprite escalada daria um osso que prende mais ou menos vértices conforme o zoom do
    // artista.
    let pesos = crate::skin_image::weights_for_mesh(sim, e, &malha, &pares, pixels_per_meter);
    // ⭐⭐⭐ **A DENSIDADE SAI DO QUADRO E VEM PARA O BIND** (F9 W1) — a malha é assada UMA vez, aqui,
    // onde o campo de pesos curva, e o quadro passa a só POSAR o que já está lá.
    //
    // ⛔⛔ **A ASSADURA NÃO ENTRA AQUI, e a 1.ª redacção desta wave punha-a** (F9 W1b → W2b): assar
    // dentro do bind SUBSTITUI a malha guardada, e isso tem três consequências que nenhum número
    // desculpa — o `Fast` deixa de ser barato, a escolha `Fast`/`Smooth` do painel COLAPSA (as duas
    // desenham a mesma malha) e a densidade fica congelada no FICHEIRO. ⇒ a malha assada é
    // **derivada** e vive num memo por bind ([`crate::skin_bake_cache`]), que é também onde a placa
    // a vai querer. *Estado derivado guardado no documento é o que envenena o undo.*
    let guardada = crate::skinned_mesh::SkinnedMesh { mesh: malha, pesos };
    let Ok(bytes) = postcard::to_allocvec(&guardada) else {
        return false;
    };
    let tendoes = pares.into_iter().map(|o| o.tendon).collect();
    sim.world_mut()
        .entity_mut(e)
        .insert(SkinBind::new(bytes, tendoes));
    true
}
