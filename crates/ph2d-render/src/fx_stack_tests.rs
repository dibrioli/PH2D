//! Gates CPU-ONLY do [`crate::fx_stack`] — irmão por `#[path]`, e portanto FILHO: `use super::*`
//! alcança os `pub(crate)` que estes gates precisam de ler.
//!
//! ⚠️ **Por que este arquivo nasceu:** a pilha de FX **não tinha gate de naga**, ao contrário do
//! compositor de camadas, que tem dois. Um erro de WGSL nela só aparecia quando um pipeline REAL
//! era construído — isto é, nos gates `#[ignore]` que precisam de adaptador. A wave que concatenou
//! um bloco NOVO nos dois módulos (as leis de mistura) é exactamente a classe de mudança que esse
//! vão deixava passar até a máquina do smoke.

use super::*;

/// **Os dois módulos da pilha PARSEIAM e VALIDAM.** Parse pega sintaxe; validação pega o resto (um
/// tipo que não fecha, um builtin com aridade errada, um nome que não existe) — e é a validação que
/// vê uma concatenação em que a metade de cima não declara o que a de baixo chama.
#[test]
fn the_fx_stack_modules_parse_and_validate_via_naga() {
    for (label, src) in module_sources() {
        let module = naga::front::wgsl::parse_str(&src)
            .unwrap_or_else(|e| panic!("fx_stack `{label}` não parseia: {e:?}"));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|e| panic!("fx_stack `{label}` não valida: {e:?}"));
    }
}

/// **As leis de mistura chegam à pilha pelo arquivo COMPARTILHADO.**
///
/// ⚠️ O gate não é decorativo: sem o prefixo, `blend_sep`/`blend_hsl` são nomes que o WGSL não
/// conhece e o `fx_blend` deixa de compilar — mas *só quando um pipeline é construído*. E se
/// alguém "resolver" isso colando uma cópia das funções dentro do `fx_stack_shader.rs`, tudo passa
/// a compilar e a divergência nasce silenciosa, no único lugar onde ninguém lê um número.
#[test]
fn the_blend_laws_come_from_the_shared_file_not_a_copy() {
    // O bloco compartilhado tem de estar lá, e é ELE que declara as duas portas.
    assert!(
        crate::layer_compositor::BLEND_MODES_WGSL.contains("fn blend_sep(")
            && crate::layer_compositor::BLEND_MODES_WGSL.contains("fn blend_hsl("),
        "o arquivo compartilhado deixou de declarar as portas de mistura"
    );
    // …e o shader da pilha NÃO pode declarar as suas.
    let own = format!("{FX_STACK_WGSL}{FX_STACK_MID_WGSL}{FX_STACK_OUT_WGSL}");
    assert!(
        !own.contains("fn blend_sep(") && !own.contains("fn blend_hsl("),
        "a pilha declarou a própria lei de mistura — é a segunda resposta que esta wave existe \
         para não ter"
    );
    for (label, src) in module_sources() {
        assert!(
            src.contains("fn blend_sep(") && src.contains("fn blend_hsl("),
            "o módulo `{label}` não recebeu o bloco compartilhado"
        );
    }
}

/// **O `Globals` do Rust e o do WGSL têm os mesmos membros, na mesma ordem.**
///
/// Um uniform é lido por OFFSET: trocar `n_segs` com `blend` não falha a ligação — passa a ler a
/// contagem de segmentos como um código de mistura, e a pilha desenha outra coisa sem um erro em
/// lugar nenhum. O `size_of` sozinho é cego a isso (os dois são `u32`).
#[test]
fn the_wgsl_globals_members_match_the_rust_struct() {
    let start = FX_STACK_WGSL
        .find("struct Globals {")
        .expect("o WGSL declara `struct Globals`");
    let body = &FX_STACK_WGSL[start..][..FX_STACK_WGSL[start..].find('}').expect("fecha")];
    let fields: Vec<&str> = body
        .lines()
        .skip(1)
        .filter_map(|l| l.trim().split(':').next())
        .filter(|n| !n.is_empty() && !n.starts_with("//"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "dims",
            "half",
            "kind",
            "tint",
            "inv_two_sigma2",
            "opacity",
            "off_x",
            "off_y",
            "jump",
            "band",
            "n_segs",
            "blend",
        ],
        "os membros do `Globals` do WGSL derivaram do struct do Rust"
    );
    // 64 bytes de propósito (o `min_binding_size` do layout) — o `blend` ocupou o padding.
    assert_eq!(core::mem::size_of::<Globals>(), 64);
}
