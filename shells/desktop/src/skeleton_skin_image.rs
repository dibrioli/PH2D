//! ⭐⭐⭐ **UMA IMAGEM OBEDECE AO ESQUELETO** — a segunda mídia (ordem do dono, 2026-09-09).
//!
//! Pesquisa e recusas medidas: [`docs/Skeleton/02_pesquisa_a_malha_sobre_a_imagem.md`].
//!
//! # As quatro perguntas, e onde cada uma é respondida
//!
//! ```text
//! onde a tinta acaba?      ph2d_poly2d::mesh_of      (uma vez, ao PRENDER)
//! que peso cada ponto tem? Skeleton::weights_at      (já existia)
//! onde o ponto vai parar?  Skeleton::deform_points   (já existia)
//! como se desenha isso?    ESTE módulo               (por quadro)
//! ```
//!
//! ⭐⭐ **A metade difícil já estava feita.** O peso é derivado por distância ao osso e não é
//! guardado, e a [`ph2d_skeleton_ecs::SkinBind`] já nascera agnóstica de mídia — o doc dela dizia,
//! por escrito, *«serve um `VecPath` hoje e uma malha raster amanhã sem uma variante nova nem um
//! schema por mídia»*. ⇒ o que esta wave acrescenta é a **malha** e o **desenho**, e nada mais.
//!
//! # ⛔ Porque o desenho é UM AFIM POR TRIÂNGULO, e não um pipeline novo
//!
//! Um triângulo de repouso e um de destino determinam um afim exactamente
//! ([`ph2d_affine::Xform::from_triangle`]), então uma malha deformada desenha-se como *N* pedaços
//! da MESMA imagem, cada um recortado ao seu triângulo. As duas peças já existiam — o
//! [`ph2d_vector::VectorScene::push_clip`] e o `draw_image_rgba_transformed` — e o compositor põe
//! o Vello **por cima** do passe de sprites.
//!
//! ⛔ **A rota por INSTÂNCIA de sprite foi medida e recusada:** a instância carrega um basis 2×2,
//! logo cada célula sairia **paralelogramo**, e duas células vizinhas de um warp real não partilham
//! aresta — a costura abre fenda. ⏸️ Um pipeline de triângulos texturados é a optimização, **com
//! razão medida**, se a rota de hoje não couber no quadro.

use ph2d_affine::Xform;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_poly2d::{Mesh2d, MeshOptions};
use ph2d_render::Sprite;

/// ⭐⭐⭐ **PIXEL DA IMAGEM → PONTO LOCAL DA SPRITE** — a lei que ata as duas réguas.
///
/// A sprite ocupa `size` unidades de mundo, centrada no `anchor` (que é o vector do pivô ao centro
/// do quad, em metros locais), e a textura é lida com `v = 0` **em cima**. ⇒ o pixel `(0, 0)` é o
/// canto **superior esquerdo** do quad e o pixel `(w, h)` o inferior direito.
///
/// ⚠️⚠️ **O `y` VIRA, e é a única inversão de todo o módulo.** Uma imagem tem o `y` a crescer para
/// baixo e o mundo tem-no a crescer para cima; esquecê-la desenha o personagem **de cabeça para
/// baixo**, e é o tipo de defeito que passa por todo gate de geometria (a malha está certa, a
/// deformação está certa, e a imagem está ao contrário).
///
/// `None` quando a imagem tem lado zero — ali não há régua.
#[must_use]
pub(crate) fn pixel_to_local(sprite: &Sprite, size_px: [u32; 2]) -> Option<Xform> {
    let (w, h) = (f64::from(size_px[0]), f64::from(size_px[1]));
    if !(w > 0.0 && h > 0.0) {
        return None;
    }
    let (sx, sy) = (f64::from(sprite.size[0]), f64::from(sprite.size[1]));
    let (ax, ay) = (f64::from(sprite.anchor[0]), f64::from(sprite.anchor[1]));
    Some(Xform([
        sx / w,
        0.0,
        0.0,
        -sy / h,
        ax - sx / 2.0,
        ay + sy / 2.0,
    ]))
}

/// ⭐⭐ **A MALHA DESTA IMAGEM, a partir dos pixels dela** — a porta do gesto de prender.
///
/// ⚠️ **Só o canal ALFA entra.** A cobertura é o que decide a silhueta, e passar as três cores
/// junto seria dar ao traçador três respostas para a mesma pergunta.
#[must_use]
pub(crate) fn mesh_from_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    opts: MeshOptions,
) -> Option<Mesh2d> {
    let alfa: Vec<u8> = rgba.iter().skip(3).step_by(4).copied().collect();
    ph2d_poly2d::mesh_of(&alfa, width, height, opts)
}

/// ⭐⭐⭐ **A MALHA POSADA, em espaço LOCAL da sprite** — o que o desenho vai recortar.
///
/// ⚠️ **A pele resolve-se no espaço da própria coisa** ([`crate::skeleton_live`] faz o mesmo para
/// uma forma vectorial), então os pontos que se deformam têm de estar nesse espaço — e é por isso
/// que o [`pixel_to_local`] corre ANTES da deformação e não depois.
///
/// Devolve os pontos posados na ordem de [`Mesh2d::rest`]. `None` quando a pele não resolve (todos
/// os ossos apagados, ou a pose da sprite é singular) — e aí quem chama desenha a sprite normal.
#[must_use]
pub(crate) fn posed_local(sim: &SimWorld, e: Entity, mesh: &Mesh2d) -> Option<Vec<[f64; 2]>> {
    let sprite = sim.world().get::<Sprite>(e)?;
    let p2l = pixel_to_local(sprite, mesh.size)?;
    let pele = crate::skeleton_live::skin_of(sim, e)?;
    let mut pts: Vec<[f64; 2]> = mesh.rest.iter().map(|&p| p2l.apply(p)).collect();
    pele.deform_points(pts.iter_mut());
    Some(pts)
}

/// ⭐⭐⭐ **O AFIM DE UM TRIÂNGULO — de pixel da imagem a espaço LOCAL da sprite.**
///
/// ⚠️ **O repouso entra em PIXELS**, e não em local: assim o afim devolvido já compõe as duas
/// coisas (a régua da imagem e a deformação), e o desenho passa-o directamente ao
/// `draw_image_rgba_transformed`, que espera um mapa a partir do espaço da imagem. Compor as duas
/// à mão em cada chamada seria a segunda resposta à mesma pergunta.
#[must_use]
pub(crate) fn triangle_xform(
    mesh: &Mesh2d,
    posed: &[[f64; 2]],
    tri: [u32; 3],
) -> Option<(Xform, [[f64; 2]; 3])> {
    let i = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
    let rest = [
        *mesh.rest.get(i[0])?,
        *mesh.rest.get(i[1])?,
        *mesh.rest.get(i[2])?,
    ];
    let alvo = [*posed.get(i[0])?, *posed.get(i[1])?, *posed.get(i[2])?];
    Some((Xform::from_triangle(rest, alvo)?, alvo))
}

#[cfg(test)]
#[path = "skeleton_skin_image_tests.rs"]
mod tests;
