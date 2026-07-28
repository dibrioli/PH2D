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

/// **O NÚMERO que decide quem toma a lei de mistura** — a medição que o `FxKindSpec::takes_blend`
/// cita, aqui, executável, em vez de prosa num doc-comment.
///
/// Um halo EXTERNO entra POR BAIXO da entrada: `out = over + halo·(1 − a)`, com `a = over.a`. Se a
/// cor do halo passasse pela lei do W3C — `Cs' = (1 − a)·Cs + a·B(Cb, Cs)` — a diferença VISÍVEL no
/// resultado seria
///
/// ```text
/// |out' − out| = |Cs' − Cs| · halo.a · (1 − a) = a · (1 − a) · |B − Cs| · halo.a
/// ```
///
/// e `a·(1 − a)` pica em **0,25 em `a = 0,5`** — o meio da rampa de anti-aliasing. Fora dela o
/// fator cai a zero pelas DUAS pontas: sem nada por baixo não há com que misturar, e sob cobertura
/// cheia o halo não aparece.
///
/// ⚠️ **Não é "inerte", e a diferença importa:** sobre conteúdo TRANSLÚCIDO de largura real (um
/// glow macio de um degrau anterior) o fator é grande numa banda inteira. O que este gate afirma é
/// mais estreito e é o que decide o produto: sobre uma forma OPACA — o caso normal — o alcance é a
/// orla. Um controle cujo efeito inteiro é uma orla de 1 px lê como quebrado.
#[test]
fn the_blend_of_an_outer_halo_only_reaches_the_antialiased_fringe() {
    // O peso com que a lei alcança o resultado, por cobertura do fundo.
    let reach = |a: f32| a * (1.0 - a);

    // Nas duas pontas o alcance é EXACTAMENTE zero — é isso que torna a orla o único lugar.
    assert_eq!(reach(0.0), 0.0, "sem nada por baixo não há mistura");
    assert_eq!(reach(1.0), 0.0, "sob cobertura cheia o halo não aparece");

    // O pico, e onde ele mora.
    let peak = (0..=1000)
        .map(|i| reach(i as f32 / 1000.0))
        .fold(0.0f32, f32::max);
    assert!(
        (peak - 0.25).abs() < 1e-4,
        "o pico do alcance é 0,25 (medido {peak:.4})"
    );

    // E a FAIXA em que ele é sequer perceptível: com `|B − Cs| = 1` (o contraste máximo), meio
    // nível de 255 exige `a·(1−a) ≥ 1/510` — o que acontece só entre `a ≈ 0,002` e `a ≈ 0,998`,
    // isto é, dentro da rampa de AA. Numa forma opaca a rampa mede ~1 px.
    let visible: Vec<f32> = (0..=1000)
        .map(|i| i as f32 / 1000.0)
        .filter(|a| reach(*a) >= 1.0 / 510.0)
        .collect();
    let (lo, hi) = (visible[0], visible[visible.len() - 1]);
    assert!(
        lo > 0.0 && hi < 1.0,
        "a faixa visível tem de excluir as duas pontas (medido {lo:.3}..{hi:.3})"
    );

    // ⚠️ E o CONTRASTE com quem TOMA a lei: um degrau de dentro aplica-a por `inner_tint`, cujo
    // peso é o mesmo `a` — mas SEM o `(1 − a)`, porque ele tinge o que está lá em vez de entrar por
    // baixo. Sob cobertura cheia o alcance é **1,0**, não zero: quatro vezes o pico do halo, e no
    // MIOLO da forma em vez de na orla.
    let inner_reach = |a: f32| a;
    assert_eq!(inner_reach(1.0), 1.0);
    assert!(
        inner_reach(1.0) / peak >= 4.0,
        "quem tinge alcança ao menos 4× o pico de quem entra por baixo"
    );
}

/// **As leis de COBERTURA ficam de fora, e o teto do painel sabe disso.**
///
/// `Behind` e `Clear` (20/21) não são leis de cor — o `apply` do Rust as desvia antes da função de
/// mistura. Um degrau de FX aplica a lei dele onde a cobertura já está decidida pela lei DELE (o
/// `inner_tint` existe precisamente para não a mover), então elas não teriam onde pousar.
///
/// ⚠️ O gate lê o `BlendMode` REAL (dev-dep) e conta: se alguém acrescentar uma lei de cor ao enum,
/// este número tem de subir junto, senão a última nasce inalcançável — o modo de falha silencioso
/// que o `MAX_FILTER_KINDS` já teve.
#[test]
fn the_fx_offers_the_colour_laws_and_not_the_coverage_ones() {
    use ph2d_painter_effects::{BlendMode, MAX_BLEND_MODES};
    assert_eq!(
        u32::from(MAX_BLEND_MODES) - u32::from(ph2d_ecs::FxOp::BLEND_KINDS),
        2,
        "o FX deixa de fora exactamente as duas leis de COBERTURA"
    );
    // …e elas são estas duas, nomeadas — não "as duas últimas", que é o que apodrece.
    for (code, name) in [(20u8, "Behind"), (21u8, "Clear")] {
        assert_eq!(BlendMode::from_u8(code).name(), name);
        assert!(
            code >= ph2d_ecs::FxOp::BLEND_KINDS,
            "{name} tem de ficar fora do alcance do FX"
        );
    }
    // O neutro é o código 0 dos dois lados.
    assert_eq!(BlendMode::from_u8(0), BlendMode::Normal);
    assert_eq!(ph2d_ecs::FxOp::BLEND_NORMAL, 0);
}

/// **O early-out do `fx_blend` em Normal é LOAD-BEARING, não higiene** — e é isto que o prova.
///
/// Sem o `return` antecipado, o caminho neutro sairia por `mix(colour, b, ab)` com `b == colour`,
/// e `mix(x, x, a)` é `x·(1−a) + x·a`. A adição em ponto flutuante **não devolve `x`** para todo
/// par `(x, a)`: os dois produtos arredondam para lados diferentes e a soma erra o último bit.
///
/// ⚠️ Se este gate um dia passar a medir ZERO casos, a frase do shader passou a ser falsa e o
/// `return` vira higiene — mas a asserção abaixo é que ele NÃO é, e é ela que tem de cair primeiro.
#[test]
fn the_normal_early_out_is_load_bearing_because_mix_is_not_the_identity() {
    // O `mix` do WGSL, escrito como o hardware o computa.
    let mix = |x: f32, a: f32| x * (1.0 - a) + x * a;
    let mut differing = 0u32;
    let mut total = 0u32;
    // Uma varredura sobre valores de cor e coberturas plausíveis.
    for xi in 1..=255u32 {
        let x = xi as f32 / 255.0;
        for ai in 1..255u32 {
            let a = ai as f32 / 255.0;
            total += 1;
            if mix(x, a).to_bits() != x.to_bits() {
                differing += 1;
            }
        }
    }
    assert!(
        differing > 0,
        "mix(x, x, a) devolveu EXACTAMENTE x em todos os {total} casos — se isto for verdade, o \
         early-out do `fx_blend` virou higiene e o doc dele tem de ser corrigido"
    );
    // E o número, para quem vier: não é um caso de canto raro.
    let pct = f64::from(differing) * 100.0 / f64::from(total);
    assert!(
        pct > 1.0,
        "só {pct:.2}% dos casos divergem — confira se a conta ainda é a que o shader faz"
    );
    eprintln!("mix(x,x,a) != x em {differing}/{total} ({pct:.1}%)");
}
