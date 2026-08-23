//! **O `bloom.wgsl` PARSA E VALIDA** — o irmão exacto do `sprite_wgsl_valid`, e pela mesma
//! razão: um erro de shader só aparece quando o pipeline é construído, ou seja **no arranque do
//! app**, e não numa corrida de `cargo test`.
//!
//! ⚠️ **Ele não existia, e foi a folha 11 que o notou** (doc 89): os quatro passes do halo
//! moram neste arquivo, e a wave dos modos (`Glow Operation`, `Glow Based On`) editou o
//! fragmento do bright-pass. Um `select` com os argumentos trocados compila em Rust, passa em
//! todo gate de CPU, e falha na primeira janela que o artista abre — com uma mensagem de naga
//! no terminal e a tela preta.
//!
//! *O gate custa o mesmo que o do `sprite.wgsl` e cobre o outro shader que este crate possui.*

const BLOOM_WGSL: &str = include_str!("../src/shaders/bloom.wgsl");

#[test]
fn bloom_wgsl_parses_and_validates() {
    let module = naga::front::wgsl::parse_str(BLOOM_WGSL).unwrap_or_else(|e| {
        panic!(
            "bloom.wgsl failed naga parse:\n{}",
            e.emit_to_string(BLOOM_WGSL)
        )
    });
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|e| panic!("bloom.wgsl failed naga validation: {e:?}"));
}

/// **OS QUATRO PASSES CONTINUAM A EXISTIR, COM OS NOMES QUE O RUST PEDE.**
///
/// ⚠️ Um `entry_point` renomeado é um `create_render_pipeline` que falha **no arranque**, e o
/// lado Rust escreve o nome como string — nada os liga em tempo de compilação.
#[test]
fn every_entry_point_the_pipelines_ask_for_is_here() {
    let module = naga::front::wgsl::parse_str(BLOOM_WGSL).expect("parse");
    let names: Vec<&str> = module
        .entry_points
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    for want in [
        "vs_main",
        "fs_prefilter",
        "fs_downsample",
        "fs_upsample",
        "fs_composite",
    ] {
        assert!(
            names.contains(&want),
            "o `motion_fx.rs` pede o entry point `{want}` e ele nao esta' no shader: {names:?}"
        );
    }
}
