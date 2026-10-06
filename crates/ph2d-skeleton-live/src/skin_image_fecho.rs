//! ⭐⭐⭐ **A MALHA QUE O QUADRO DESENHA, E A ORDEM DAS FACES** da imagem presa.
//!
//! ⛔⛔ **Nada se cose entre membros** (decisão do dono, 2026-10-06: *«mesmo se sobrepondo não puxe
//! nada da imagem cuja influência é do outro osso»*). Cada membro desenha só a SUA imagem: a malha
//! desenhada é a do bind, posada pela pele. Onde dois membros se sobrepõem manda a ordem das faces;
//! onde só se encostam vê-se o contorno real de cada um. As leis recusadas — a bola da silhueta, a
//! costura sobre a borda da malha e a do anel da arte — estão na fila (§F48, §F49, §F59).
//!
//! # ⭐⭐ A ORDEM DAS FACES: o osso mais adiante na corrente pinta por cima
//!
//! Ordem do dono (2026-10-02): *«as faces influenciadas por um osso têm z-index aleatório, e ao se
//! sobrepor às do outro osso misturam-se; melhor seria as do último osso por cima»*. A ordem dos
//! triângulos É a ordem do desenho (as duas portas, placa e CPU), e a da grelha é a das CÉLULAS — na
//! dobra forte os pedaços dos dois membros intercalavam-se. ⇒ [`ordena_pelo_osso`], UMA vez por bind,
//! na gaveta da malha desenhada ([`crate::skin_bake_cache::desenhada_da_arte`]).

use ph2d_poly2d::Mesh2d;
use ph2d_render::SpriteMesh;
use ph2d_skeleton::{Correccao, Skin, Xform};

/// ⭐⭐⭐ **A malha que o quadro desenha** — a porta do [`crate::skin_image::attach_skin_meshes`]:
/// a PLACA posa por omissão (o `local` traz o repouso e a tabela que o `vs_main` lê), a CPU com
/// `PH2D_SKIN_GPU=0` (o `local` traz o posado). A lei que decide mora no cabeçalho do
/// [`crate::skin_image_gpu`].
#[must_use]
pub fn malha_desenhada(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    anchor: [f32; 2],
    size: [f32; 2],
    correcoes: &[Correccao],
) -> Option<SpriteMesh> {
    let construtor = if crate::skin_image_gpu::a_placa_posa() {
        crate::skin_image_gpu::sprite_mesh_para_a_placa
    } else {
        crate::skin_image::posed_sprite_mesh_corrigida
    };
    construtor(mesh, p2l, pele, pesos, anchor, size, correcoes)
}

/// ⭐⭐⭐ **A CHAVE DE OSSO de uma linha de pesos** — a profundidade MÉDIA, pesada, dos ossos que a
/// movem (`Σ wⱼ·pⱼ / Σ wⱼ`, `pⱼ` de [`crate::esqueletos::profundidades`]): quem tem chave maior pinta
/// por cima. ⚠️ Uma média e não o osso dominante: uma zona de mistura fica ENTRE os dois membros, e
/// a ordem não dá um salto onde o peso cruza `0,5`. `prof` que não fecha com a linha cai no índice
/// da coluna (o que a chave era até 2026-10-04 — ⛔ errado na ordem `to_bits`, ver a porta).
#[must_use]
pub fn chave_de_osso(w: &[f64], prof: &[f64]) -> f64 {
    let soma: f64 = w.iter().sum();
    #[expect(clippy::cast_precision_loss, reason = "índice de osso")]
    let pos: f64 = w
        .iter()
        .enumerate()
        .map(|(j, p)| {
            p * prof
                .get(j)
                .copied()
                .filter(|_| prof.len() == w.len())
                .unwrap_or(j as f64)
        })
        .sum();
    if soma > 0.0 { pos / soma } else { 0.0 }
}

/// ⭐⭐ **Ordena os triângulos pelo OSSO que os move** — o mais FUNDO na corrente desenha-se depois.
///
/// A chave de um triângulo é a média da [`chave_de_osso`] dos três vértices. A ordenação é ESTÁVEL —
/// faces do mesmo osso mantêm a ordem da grelha. Sem tabela (a lei derivada) a ordem fica como está.
pub fn ordena_pelo_osso(tris: &mut [[u32; 3]], pesos: &[f64], vertices: usize, prof: &[f64]) {
    let ossos = pesos.len() / vertices.max(1);
    if ossos < 2 || pesos.len() != ossos * vertices {
        return;
    }
    let chave: Vec<f64> = pesos
        .chunks_exact(ossos)
        .map(|w| chave_de_osso(w, prof))
        .collect();
    let de = |t: &[u32; 3]| {
        t.iter()
            .map(|&v| chave.get(v as usize).copied().unwrap_or(0.0))
            .sum::<f64>()
    };
    tris.sort_by(|a, b| de(a).total_cmp(&de(b)));
}

#[cfg(test)]
#[path = "skin_image_fecho_tests.rs"]
mod tests;
