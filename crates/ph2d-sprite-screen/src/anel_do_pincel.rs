//! ⭐⭐⭐ **O ANEL DO PINCEL DA REMOÇÃO DE FUNDO, ONDE O PINCEL PINTA** — o contorno, em px de ecrã,
//! do disco que o pincel de protecção pousa na imagem de origem.
//!
//! # O defeito (medido em 2026-09-15, entregue em 2026-09-16)
//!
//! O anel era um CÍRCULO com o raio da escala do **quad de repouso** (`|coluna 0|` do afim da
//! sprite). Numa arte presa ao esqueleto e dobrada o pincel pinta pela **malha** — a mesma porta
//! ([`crate::uv_sob_o_ponteiro`]) resolve o ponteiro na imagem —, e ali cada texel ocupa no ecrã o
//! que a dobra lhe dá: o anel dizia um tamanho e o pincel pintava outro. Medido na cena do smoke
//! (`PH2D_VEC_BONE_PAINT_SMOKE`, `25°` por junta; sonda `sonda_o_anel_da_remocao_de_fundo` da
//! `ph2d-app-vec`), cada ponto do anel devolvido à imagem pela porta do ponteiro:
//!
//! | a arte | erro do raio, mediana | p90 | pior |
//! |---|---:|---:|---:|
//! | sem dobra (o CONTROLO) | `0,0 %` | `0,0 %` | `0,0 %` |
//! | dobrada, o anel de ANTES | `14,4 %` | `33,2 %` | **`46,0 %`** (junto de uma junta) |
//! | dobrada, **este anel** | `≤ 2,5e-5` | | |
//!
//! ⚠️ **A nota de 2026-09-15 dizia `8,8 %`**, medida em doze pontos e com a lei de pesos de antes
//! do padrão-ouro: *um número de uma amostra pequena sobre outro campo não é o número de hoje.*
//!
//! # ⭐ A lei: o disco é da IMAGEM, e vai ao ecrã pelo mapa que DESENHA
//!
//! O pincel pinta um disco de `raio` px **de origem** à volta da UV debaixo do ponteiro. O anel é
//! esse disco, amostrado na imagem e levado ao ecrã pela **malha desenhada**
//! ([`ph2d_render::DrawnMesh::world_at_uv`]) — ou pelo afim do quad, quando a sprite não é malha.
//! *Um controlo DESENHADO por um mapa e AGARRADO por outro é um controlo morto sob o dedo* (F6-m):
//! aqui o ponteiro e o anel são as duas metades do mesmo par.
//!
//! ⚠️ **Fora da arte dobrada o anel PARTE-SE em arcos**, e não é defeito: o texel existe e o pincel
//! pinta-o, mas ele **não é desenhado** (a malha cobre só a silhueta) — uma corda por cima da arte
//! para o ligar desenharia um sítio onde nada acontece. E com o ponteiro FORA da arte não há anel:
//! a porta do ponteiro recusa ali, e o pincel não pinta.

use std::f32::consts::TAU;

/// Quantos lados tem o anel — o mesmo número do anel do Painter, que é o que lê liso a um
/// pincel grande.
pub const LADOS_DO_ANEL: u32 = 64;

/// Ver o cabeçalho do módulo. Devolve os ARCOS do anel em px de ecrã (um só, fechado, quando o
/// disco cabe inteiro na arte desenhada), ou `None` quando o ponteiro não está sobre a arte.
///
/// `raio` é em px da imagem de ORIGEM, e `origem` é o tamanho dela — `(largura, altura)`.
#[allow(clippy::too_many_arguments)]
pub fn anel_do_pincel(
    sim: &ph2d_ecs::SimWorld,
    present: &mut ph2d_ecs::World,
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    bits: u64,
    ponteiro: (f32, f32),
    raio: f32,
    origem: (u32, u32),
) -> Option<Vec<Vec<[f64; 2]>>> {
    let crate::UvSobOPonteiro::Uv(u0, v0) =
        crate::uv_sob_o_ponteiro(sim, present, camera, window, bits, ponteiro.0, ponteiro.1)
    else {
        return None;
    };
    // Um raio `NaN`, nulo ou negativo não é um disco — não há anel a desenhar.
    if origem.0 == 0 || origem.1 == 0 || raio.is_nan() || raio <= 0.0 {
        return None;
    }
    let (ru, rv) = (raio / origem.0 as f32, raio / origem.1 as f32);
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let tr = ph2d_ecs::world_transform(sim.world(), entity)?;
    let sprite = sim.world().get::<ph2d_render::Sprite>(entity)?;
    let grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    // A UNIDADE, como na porta do ponteiro: `uv / 1` já é a fracção.
    let quad = crate::sprite_image_to_screen_affine(1, 1, tr, sprite, grid, camera, window);
    let malha = ph2d_render::drawn_mesh_of(present, bits);

    let pontos: Vec<Option<[f64; 2]>> = (0..LADOS_DO_ANEL)
        .map(|i| {
            let a = i as f32 / LADOS_DO_ANEL as f32 * TAU;
            let uv = [u0 + ru * a.cos(), v0 + rv * a.sin()];
            match &malha {
                Some(m) => m.world_at_uv(uv).map(|w| {
                    let (x, y) = camera.world_to_screen(w, window);
                    [f64::from(x), f64::from(y)]
                }),
                None => {
                    let p = quad * ph2d_vector::Point::new(f64::from(uv[0]), f64::from(uv[1]));
                    Some([p.x, p.y])
                }
            }
        })
        .collect();
    Some(arcos_de(&pontos))
}

/// Os pontos de uma volta em ARCOS: as corridas de pontos presentes, com a que atravessa o início
/// da volta costurada numa só, e a volta inteira FECHADA sobre o primeiro ponto.
fn arcos_de(pontos: &[Option<[f64; 2]>]) -> Vec<Vec<[f64; 2]>> {
    let Some(corte) = pontos.iter().position(Option::is_none) else {
        let mut fechado: Vec<[f64; 2]> = pontos.iter().flatten().copied().collect();
        if let Some(&p) = fechado.first() {
            fechado.push(p);
        }
        return vec![fechado];
    };
    // ⭐ A volta começa num ângulo arbitrário: percorrê-la a partir de um buraco faz com que a
    // corrida que o atravessaria saia inteira, sem costura à mão.
    let n = pontos.len();
    let mut arcos: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut atual: Vec<[f64; 2]> = Vec::new();
    for k in 1..=n {
        match pontos[(corte + k) % n] {
            Some(p) => atual.push(p),
            None => {
                if atual.len() >= 2 {
                    arcos.push(std::mem::take(&mut atual));
                }
                atual.clear();
            }
        }
    }
    if atual.len() >= 2 {
        arcos.push(atual);
    }
    arcos
}

#[cfg(test)]
#[path = "anel_do_pincel_tests.rs"]
mod gates;

#[cfg(test)]
mod tests {
    use super::arcos_de;

    fn p(x: f64) -> Option<[f64; 2]> {
        Some([x, 0.0])
    }

    #[test]
    fn a_whole_turn_is_one_closed_arc() {
        let a = arcos_de(&[p(0.0), p(1.0), p(2.0)]);
        assert_eq!(
            a,
            vec![vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [0.0, 0.0]]]
        );
    }

    /// A corrida que atravessa o início da volta sai INTEIRA, e um ponto isolado não é arco.
    #[test]
    fn a_run_across_the_start_is_one_arc_and_a_lone_point_is_none() {
        let a = arcos_de(&[p(0.0), p(1.0), None, p(3.0), None, p(5.0), p(6.0)]);
        assert_eq!(
            a,
            vec![vec![[5.0, 0.0], [6.0, 0.0], [0.0, 0.0], [1.0, 0.0]]],
            "a corrida 5-6-0-1 atravessa o inicio; o 3 esta' sozinho"
        );
    }
}
