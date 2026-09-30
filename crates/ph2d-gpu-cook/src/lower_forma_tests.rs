//! Os gates PUROS do baixamento das formas (doc 121 W3) — a fonte WGSL, sem adaptador. A paridade
//! de VALOR contra a CPU vive em `tests/it/as_formas_no_dispositivo.rs`, num adaptador real.

use super::*;

/// ⚠️ **O `size` ausente é `[1, 1]` e NÃO o `default_size` do chamador** — a forma já traz o
/// tamanho autorado na geometria, e é essa a omissão do braço de forma da CPU.
#[test]
fn sem_colunas_le_as_omissoes_da_cpu() {
    let src = forma_module([false; 5], [0.0, 0.0]);
    assert!(
        src.contains("fn read_size(i: u32) -> vec2<f32> { _ = i; return vec2<f32>(1.0, 1.0); }")
    );
    assert!(
        !src.contains("default_size"),
        "a forma não lê o tamanho da sprite"
    );
    assert!(src.contains("fn read_geometry_id(i: u32) -> f32 { _ = i; return 0.0; }"));
    assert!(!src.contains("var<storage, read> in_"));
}

/// A linha é forma quando `geometry_id > 0,5` (o `MediaColumns::at`, à letra) — e a que não é
/// leva a geometria que o passe não acha.
#[test]
fn a_linha_e_forma_acima_de_meio() {
    let mut present = [false; 5];
    present[4] = true;
    let src = forma_module(present, [0.0, 0.0]);
    assert!(src.contains("var<storage, read> in_geometry_id: array<f32>;"));
    assert!(src.contains("if (g > 0.5) {"));
    assert!(src.contains(&format!("var h = {SEM_GEOMETRIA}u;")));
    assert!(src.contains("instances[base + 10u] = h;"));
}

/// ⚠️ **O pivô ZERO é a palavra CRAVADA** — `s.x * 0.0` propagaria um `size` não finito para a
/// âncora onde a CPU escreve `0`. Com pivô, a multiplicação pelo `size` desta linha.
#[test]
fn o_pivo_zero_crava_a_ancora() {
    let zero = forma_module([true; 5], [0.0, 0.0]);
    assert!(zero.contains("instances[base + 8u] = 0u;"));
    assert!(zero.contains("instances[base + 9u] = 0u;"));
    let meio = forma_module([true; 5], [0.5, -0.25]);
    assert!(meio.contains("wf(base + 8u, s.x * 0.5);"));
    assert!(meio.contains("wf(base + 9u, s.y * -0.25);"));
}

/// A assinatura separa colunas E pivôs, e o pivô zero dá só os bits das colunas.
#[test]
fn a_assinatura_separa_colunas_e_pivos() {
    let mut a = [false; 5];
    a[0] = true;
    assert_eq!(forma_signature(a, [0.0, 0.0]), 1);
    assert_ne!(
        forma_signature(a, [0.5, 0.0]),
        forma_signature(a, [0.0, 0.0])
    );
    assert_ne!(
        forma_signature(a, [0.5, 0.0]),
        forma_signature(a, [0.5, 1e-7])
    );
    assert_ne!(
        forma_signature([true; 5], [0.0, 0.0]),
        forma_signature(a, [0.0, 0.0])
    );
}

/// ⭐ **A sprite CALA a linha de forma** — o baixamento irmão escreve-a com tamanho e opacidade
/// zero, porque a CPU só baixa sprites das linhas `Sprite`. Sem a coluna, a condição lê a omissão
/// `0` e nunca arma.
#[test]
fn a_sprite_cala_a_linha_de_forma() {
    let mut present = [false; 9];
    present[8] = true;
    let src = crate::lower::lower_module(present, ph2d_render::SinkStyle::PLAIN);
    assert!(src.contains("if (read_geometry_id(i) > 0.5) {"));
    assert!(src.contains("instances[base + 35u] = 0u;"));
    let nu = crate::lower::lower_module([false; 9], ph2d_render::SinkStyle::PLAIN);
    assert!(nu.contains("fn read_geometry_id(i: u32) -> f32 { _ = i; return 0.0; }"));
}
