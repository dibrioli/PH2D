//! As provas da resolução da máscara — a metade que se mede sem uma GPU.
//!
//! ⚠️ **O que este arquivo NÃO pode testar é a metade da view**, e a razão é a mesma que o
//! `appearance_of` já documenta: um `TextureAtlas` só se constrói com um contexto de GPU. O que
//! se prova aqui é a **chave** — a coisa que decide se o passe refaz os bind groups —, e ela é
//! aritmética pura.

use super::{Resolved, key_of};

fn r(texture_id: u32, uv_rect: [f32; 4]) -> Resolved {
    Resolved {
        texture_id,
        uv_rect,
    }
}

/// **A chave TEM de ver o sub-rect, não só o id.**
///
/// ⚠️ Este é o gate que impede o defeito mais silencioso desta feature: duas células do atlas
/// partilhado têm o MESMO `texture_id` (`0`), então uma chave que fosse só o id leria *"a mesma
/// imagem"* ao trocar entre duas sprites empacotadas — o artista escolheria outra coisa e a tela
/// não mudaria, sem nada vermelho em lado nenhum.
#[test]
fn two_atlas_cells_are_two_different_masks() {
    let a = key_of(r(0, [0.0, 0.0, 0.25, 0.25]));
    let b = key_of(r(0, [0.5, 0.5, 0.25, 0.25]));
    assert_ne!(a, b, "duas celulas do MESMO atlas colidiram na chave");
    // E o mesmo rect no mesmo id é a mesma escolha — senão os bind groups refar-se-iam
    // por quadro, que é exactamente o que a chave existe para evitar.
    assert_eq!(a, key_of(r(0, [0.0, 0.0, 0.25, 0.25])));
}

#[test]
fn two_individual_textures_are_two_different_masks() {
    let full = [0.0, 0.0, 1.0, 1.0];
    assert_ne!(key_of(r(7, full)), key_of(r(8, full)));
    assert_eq!(key_of(r(7, full)), key_of(r(7, full)));
}

/// Um id assado e um individual com o mesmo número baixo não existem ao mesmo tempo (o bit 31
/// separa-os), mas a chave não depende disso — ela come o id inteiro.
#[test]
fn the_cooked_namespace_does_not_collide_with_the_individual_one() {
    let full = [0.0, 0.0, 1.0, 1.0];
    let cooked = 1_u32 << 31 | 7;
    assert_ne!(key_of(r(7, full)), key_of(r(cooked, full)));
}

/// **Controle da própria régua:** uma chave que ignorasse tudo passaria os testes de igualdade
/// acima e falharia este. Ela tem de DISTINGUIR num varrimento largo.
#[test]
fn the_key_separates_a_whole_sweep_of_choices() {
    let mut seen = std::collections::BTreeSet::new();
    for id in [0_u32, 1, 2, 1 << 31] {
        for x in 0..4 {
            for y in 0..4 {
                let f = |v: i32| v as f32 * 0.25;
                seen.insert(key_of(r(id, [f(x), f(y), 0.25, 0.25])));
            }
        }
    }
    assert_eq!(seen.len(), 4 * 4 * 4, "a chave colidiu no varrimento");
}

/// **A CÉLULA DE ATLAS É A FONTE QUE DISTINGUE AS TRÊS.**
///
/// ⚠️ Este é o gate que a célula da folha 11 pedia sem lhe saber o nome: com `Individual` e
/// `CookedTexture` (rect inteiro) o aspecto da textura É o da imagem, então uma implementação
/// que lesse o da textura passaria as duas e falharia só no atlas — *"funciona com umas imagens
/// e falha em silêncio com outras"*, literal.
#[test]
fn an_atlas_cell_reports_its_own_aspect_not_the_shared_sheets() {
    // O átlas é QUADRADO (2048×2048). Uma célula de 512×256 dentro dele é 2:1.
    let cell = [0.25, 0.5, 512.0 / 2048.0, 256.0 / 2048.0];
    let got = super::image_aspect(cell, 2048, 2048);
    assert!(
        (got - 2.0).abs() < 1e-4,
        "a celula e' 2:1 e leu {got} — o aspecto veio da FOLHA, nao da imagem"
    );
    // E uma textura que é dela própria devolve o aspecto dela.
    assert!((super::image_aspect([0.0, 0.0, 1.0, 1.0], 640, 480) - 4.0 / 3.0).abs() < 1e-4);
    assert!((super::image_aspect([0.0, 0.0, 1.0, 1.0], 256, 256) - 1.0).abs() < 1e-6);
}

#[test]
fn a_degenerate_rect_is_square_never_a_nan() {
    for r in [
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ] {
        let a = super::image_aspect(r, 1024, 1024);
        assert_eq!(a, 1.0, "{r:?}");
        assert!(a.is_finite());
    }
    assert_eq!(super::image_aspect([0.0, 0.0, 1.0, 1.0], 0, 0), 1.0);
}
