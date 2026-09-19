//! ⭐⭐⭐ **A LEI DO ESTILO NOS DOIS MOTORES, COM AS ENTRADAS ENTREGUES** — a outra metade da
//! atribuição que o [`super::curvatura_parity_tests`] abriu.
//!
//! # ⚠️⚠️ Porque este ficheiro tem de existir ao lado daquele
//!
//! O gate de imagem (`estilo_tests`) mede a saída de uma cadeia inteira e devolve um byte. Quando
//! ele acusa, há **duas** causas possíveis e as curas são **opostas**:
//!
//! | causa | o que é | a cura |
//! |---|---|---|
//! | a **GRANDEZA** a montante | os dois motores medem curvaturas diferentes no mesmo ponto | mexer no ESTIMADOR (o `ε`, o estêncil, o avaliador) |
//! | a **LEI** do estilo | a transcrição Rust→WGSL não é a mesma aritmética | mexer na TRANSCRIÇÃO |
//!
//! ⛔ *Sem separá-las, quem lê «163 píxeis fora» vai afinar a que não tem culpa.* Aqui a curvatura
//! e o `|N·V|` **chegam de fora**, escolhidos por uma grelha — ⇒ o que sobra é a lei, nua.
//!
//! # ⭐⭐⭐ E o ramo que o WGSL tem e o Rust não
//!
//! O `st_rim_lit` tem um `if (e.sombra.w > 0.0)` que o [`ph2d_style::Style::rim_lit`] não tem: o
//! `pow(0, 0)` é **indefinido** em WGSL e o `powf` do Rust devolve `1`. ⚠️ **A grelha põe
//! `width = 0` de propósito**, que é o único ponto onde aquele ramo é observável — e o ponto onde
//! ele estaria errado se alguém lhe mexesse.
//!
//! ⚠️⚠️ **E há um segundo sítio onde as duas linguagens podem não concordar sem ninguém o ter
//! escrito: o próprio `pow` para expoente diferente de zero.** O `powf` do Rust é a libm da máquina
//! e o `pow` do WGSL é, na maioria dos backends, `exp2(y · log2(x))` — *duas funções diferentes com
//! o mesmo nome*. A grelha varre a largura em `[0, 64]` e o `facing` em toda a faixa, e é ela que
//! diz se isso é um byte ou um ULP.

use ph2d_style::{Curvature, Rim, Style, Zones};

/// A ordem dos bindings é a do [`ph2d_field_gpu::probe::evaluate`]: uniformes, storages, entrada,
/// saída. ⚠️ **A `struct Estilo` e as duas funções vêm da [`ph2d_style::wgsl::source`]**, nunca
/// transcritas — é a mesma lei que o produto compila.
const ARNES: &str = r"
@group(0) @binding(0) var<uniform> e: Estilo;
@group(0) @binding(1) var<storage, read> entrada: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn avalia(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i * 2u + 1u >= arrayLength(&entrada)) { return; }
    // ⚠️ DUAS entradas por amostra: a lei precisa de cinco números e um `vec4` leva quatro.
    let a = entrada[i * 2u + 0u];   // xyz = a luz de cena · w = `|N·V|`
    let b = entrada[i * 2u + 1u];   // x = a curvatura em unidades da peça
    saida[i * 2u + 0u] = vec4<f32>(st_apply(e, a.xyz, a.w, b.x), 0.0);
    saida[i * 2u + 1u] = vec4<f32>(st_saturate_indirect(e, a.xyz), 0.0);
}
";

/// Os estilos com que a lei é varrida — **um botão de cada vez, e depois todos juntos**.
///
/// ⚠️ A separação é o que faz a tabela nomear o culpado: uma varredura só com o estilo vestido
/// responde *«a lei diverge»* e não *«qual das quatro»*.
///
/// ⚠️ **`pub(crate)` porque a metade do PIXEL a lê** ([`super::estilo_pixel_parity_tests`]): as duas
/// metades têm de varrer a **mesma** população, senão a tabela da lei e a do pixel deixam de se
/// poder ler lado a lado no dia em que alguém acrescentar um botão a uma delas.
pub(crate) fn baterias() -> Vec<(&'static str, Style)> {
    let v = crate::gpu_frame::estilo_tests::vestido();
    let mut out = vec![
        ("a fábrica", Style::default()),
        (
            "só o contorno",
            Style {
                rim: v.rim,
                ..Style::default()
            },
        ),
        (
            "só o contorno, largura 0",
            Style {
                rim: Rim {
                    width: 0.0,
                    ..v.rim
                },
                ..Style::default()
            },
        ),
        (
            "só o contorno, largura 64 (o tecto)",
            Style {
                rim: Rim {
                    width: Rim::MAX_WIDTH,
                    ..v.rim
                },
                ..Style::default()
            },
        ),
        (
            "só a tinta por curvatura",
            Style {
                curvature: v.curvature,
                ..Style::default()
            },
        ),
        (
            "só a tinta por curvatura, nitidez 8",
            Style {
                curvature: Curvature {
                    sharpness: 8.0,
                    ..v.curvature
                },
                ..Style::default()
            },
        ),
        (
            "só as zonas",
            Style {
                zones: v.zones,
                ..Style::default()
            },
        ),
        (
            "só as zonas, pivô no piso",
            Style {
                zones: Zones {
                    pivot: Zones::MIN_PIVOT,
                    ..v.zones
                },
                ..Style::default()
            },
        ),
        (
            "só a saturação da indirecta",
            Style {
                indirect_saturation: v.indirect_saturation,
                ..Style::default()
            },
        ),
        (
            "a saturação a lavar (0)",
            Style {
                indirect_saturation: 0.0,
                ..Style::default()
            },
        ),
        ("os quatro, vestidos", v),
    ];
    // ⚠️ **Todos saneados**, que é o que o produto entrega ao `apply` dos dois lados — a porta do
    // dispositivo ([`ph2d_style::wgsl::pack`]) chama a mesma função.
    for (_, s) in &mut out {
        *s = s.sanitized();
    }
    out
}

/// ⭐⭐⭐ **A distância em ULP — com o SUBNORMAL tratado, porque ali a régua mente.**
///
/// # ⛔⛔ A 1.ª redacção era `a.to_bits().abs_diff(b.to_bits())`, e ela leu `15 141` ULP sobre uma
/// diferença absoluta de `2e-41`
///
/// O caso é o contorno com a largura no tecto: `(1 − 0,77)^64` é `1,2e-42`, um **subnormal**. O
/// Rust representa-o; a placa **esvazia subnormais para zero** (é o que toda GPU faz, e o WGSL não o
/// proíbe). ⇒ os dois números diferem por `2e-41` e os **padrões de bits** diferem por meio campo de
/// mantissa. *Contar ULPs através da fronteira do subnormal mede a REPRESENTAÇÃO, não o erro* — é a
/// mesma família do `edge_max` global cego ao quad fino.
///
/// ⇒ abaixo de [`f32::MIN_POSITIVE`] os dois valores são **indistinguíveis para todo consumidor
/// desta casa** (a menor luz que sobrevive à exposição e à descida a 8 bits é da ordem de `1e-8`), e
/// a régua devolve `0`. ⚠️ **Isto não afrouxa nada: é MAIS apertado** — ali passa a exigir-se que o
/// absoluto esteja abaixo de `1,18e-38`, que nenhuma folga de ULP exigiria.
fn distancia_em_ulp(a: f32, b: f32) -> u32 {
    if a.abs() < f32::MIN_POSITIVE && b.abs() < f32::MIN_POSITIVE {
        return 0;
    }
    a.to_bits().abs_diff(b.to_bits())
}

/// ⚠️ **O CONTROLO da régua acima** — sem ele, ela poderia devolver `0` para tudo.
#[test]
fn a_regua_de_ulp_ve_um_ulp_normal_e_ignora_o_subnormal() {
    // Um ULP a sério é visto.
    let x = 1.25f32;
    let y = f32::from_bits(x.to_bits() + 1);
    assert_eq!(distancia_em_ulp(x, y), 1, "a régua não vê um ULP normal");
    // Um subnormal contra zero — o caso medido, `2e-41` de diferença absoluta.
    let d = f32::from_bits(0x0000_3b00);
    assert!(
        d > 0.0 && d < f32::MIN_POSITIVE,
        "a fixtura não é subnormal"
    );
    assert!(
        d.to_bits().abs_diff(0u32) > 10_000,
        "a fixtura não reproduz a explosão de bits que motivou a régua"
    );
    assert_eq!(
        distancia_em_ulp(d, 0.0),
        0,
        "o subnormal contra zero não foi tratado"
    );
    // ⛔ E um NORMAL pequeno contra zero continua a ser visto — a cerca não pode engolir isso.
    assert!(
        distancia_em_ulp(f32::MIN_POSITIVE, 0.0) > 1_000,
        "a cerca do subnormal engoliu um valor NORMAL"
    );
}

/// A grelha de entradas — a luz de cena, o `|N·V|` e a curvatura da peça.
///
/// ⚠️ **Ela varre a CURVATURA em toda a faixa da fixtura** (`−3,4` a `+11,7`, medida no
/// `estilo_tests`) e o `facing` **até aos extremos exactos** `0` e `1`: a silhueta é onde o
/// contorno vive, e `facing = 1` é onde o `pow` degenera.
fn grelha() -> Vec<[f32; 4]> {
    let mut v = Vec::new();
    for &l in &[0.0f32, 1e-4, 0.02, 0.18, 0.6, 1.0, 3.5, 40.0] {
        for &tint in &[[1.0f32, 1.0, 1.0], [0.9, 0.35, 0.12], [0.1, 0.6, 1.0]] {
            for &facing in &[0.0f32, 1e-6, 0.03, 0.25, 0.5, 0.77, 0.999, 1.0] {
                for &k in &[-11.7f32, -3.36, -1.0, -0.05, 0.0, 0.05, 1.82, 4.0, 11.7] {
                    v.push([l * tint[0], l * tint[1], l * tint[2], facing]);
                    v.push([k, 0.0, 0.0, 0.0]);
                }
            }
        }
    }
    v
}

/// ⭐⭐⭐ **A LEI DO ESTILO SÓ DIVERGE PELO QUE A LINGUAGEM IMPÕE — e as DUAS metades que têm de
/// ficar AO BIT ficam.**
///
/// ```text
/// env PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d \
///   estilo_lei_parity_tests::a_lei_do_estilo_so_diverge_pelo_que_a_linguagem_impoe \
///   -- --ignored --nocapture
/// ```
///
/// # ⛔⛔⛔ A 1.ª redacção exigia ZERO ULP em toda a bateria, e ela REPROVOU — sobre uma lei correcta
///
/// A [`de_quem_e_o_ulp_da_lei_do_estilo`] nomeia as duas causas, e **nenhuma está no texto que
/// alguém escreveu**: a placa **contrai** `a*b + c` num `fma` (a forma fundida bate em `1680/1680`
/// amostras, a solta em `1463`) e o `pow` do WGSL **não é** o `powf` do Rust (`870/1680` iguais,
/// pior `44` ULP). ⇒ *uma lei sobre o que se ESCREVE não é uma lei sobre o que CORRE*, e o gate
/// `nenhuma_conta_desta_crate_e_fundida` — que está verde — mede o ficheiro, não o binário.
///
/// ⚠️⚠️ **Afrouxar para «alguns ULP» em toda a linha seria uma LICENÇA**, e é por isso que este
/// gate tem TRÊS metades e só a terceira tem folga:
///
/// 1. **a fábrica é EXACTA** — zero ULP, zero absoluto. É a asserção de que tudo o resto depende,
///    e nenhuma contracção a pode mover: as formas de fábrica são `x·1` e `x + 0`, exactas em
///    `f32` **fundidas ou não** (ver o cabeçalho da [`ph2d_style`]).
/// 2. **a TINTA POR CURVATURA é EXACTA** — e esta é a que decide a leitura do gate de imagem: se a
///    lei dela bate ao bit e o pixel dela diverge `45` bytes, **a divergência é da GRANDEZA** e
///    mexer na lei seria afinar o inocente.
/// 3. o resto fica com um tecto **MEDIDO e por bateria**, e ele nomeia o recurso: a largura em ULP
///    de uma contracção e de um `pow`.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_lei_do_estilo_so_diverge_pelo_que_a_linguagem_impoe() {
    let Some(t) = crate::gpu_frame::shared() else {
        panic!("sem adaptador de GPU — este gate não pode ser saltado em silêncio");
    };
    let amostras = grelha();
    assert!(amostras.len() > 3_000, "grelha pobre: {}", amostras.len());
    let fonte = format!("{}\n{ARNES}", ph2d_style::wgsl::source());

    // ⭐ **As duas que têm de ser EXACTAS, e porquê** — ver o cabeçalho. ⛔ Uma bateria nova não
    // entra aqui sem a razão de ela não poder ser movida por uma contracção.
    const EXACTAS: [&str; 3] = [
        "a fábrica",
        "só a tinta por curvatura",
        "só a tinta por curvatura, nitidez 8",
    ];
    // ⭐⭐⭐ **DOIS tectos, porque são DUAS causas com recursos diferentes** — e cada um nomeia o seu.
    //
    // ⚠️ **A CONTRACÇÃO** (`a*b + c` → `fma`) desloca o resultado de **uma** arredondamento: medido
    // `1`–`2` ULP em todas as baterias sem contorno. `8` é a folga de uma placa a mais.
    const TECTO_CONTRACCAO: u32 = 8;
    // ⚠️⚠️ **O `pow`** do WGSL é, nos backends, `exp2(y · log2(x))` — ⇒ o erro do `log2` entra
    // **multiplicado pelo expoente**, e o erro relativo cresce com `y`. *Não é uma folga, é a forma
    // da função*: medido `19` ULP com `y = 2,5` e `66` com `y = 64`, que é o [`Rim::MAX_WIDTH`].
    // ⇒ o tecto é derivado do TECTO DO BOTÃO e não escolhido.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tecto_pow: u32 = (Rim::MAX_WIDTH as u32) * 2;
    let mut acusadas: Vec<String> = Vec::new();
    println!("\n  BATERIA                              pior |Δ| absoluto   pior ULPs");
    for (rot, style) in baterias() {
        let guarda = t.lock().expect("o traçador");
        let (device, queue) = guarda.parts();
        let saida = ph2d_field_gpu::probe::evaluate(
            device,
            queue,
            &fonte,
            "avalia",
            &[&ph2d_style::wgsl::pack(&style)],
            &[],
            &amostras,
            1,
        );
        drop(guarda);

        let mut pior = 0.0f64;
        let mut pior_ulp = 0u32;
        // ⚠️⚠️ **DUAS localizações e não uma.** A 1.ª redacção deste arnês guardava o sítio do pior
        // ABSOLUTO e imprimia-o ao lado do pior ULP — e as duas colunas apontam para amostras
        // DIFERENTES (`15 141` ULP num valor minúsculo, `3,8e-6` num valor de `41`). *Uma tabela
        // que dá o extremo de uma métrica e o endereço de outra convida à leitura errada*, e ela
        // custou-me uma hipótese antes de a apanhar.
        let mut onde = String::new();
        let mut onde_ulp = String::new();
        for (i, par) in amostras.as_chunks::<2>().0.iter().enumerate() {
            let scene = [par[0][0], par[0][1], par[0][2]];
            let at = ph2d_style::Point {
                facing: par[0][3],
                curvature: par[1][0],
            };
            let cpu = [style.apply(scene, at), style.saturate_indirect(scene)];
            for (m, esperado) in cpu.iter().enumerate() {
                let got = saida[i * 2 + m];
                for c in 0..3 {
                    let d = f64::from(got[c]) - f64::from(esperado[c]);
                    // ⚠️ **ULPs e não só o absoluto**: numa luz de `40` um `1e-5` é ruído e num
                    // `1e-4` é uma lei diferente. *Uma barra absoluta sobre uma grandeza HDR mede a
                    // magnitude da amostra, não o erro.*
                    let ulp = distancia_em_ulp(esperado[c], got[c]);
                    if d.abs() > pior {
                        pior = d.abs();
                        onde = format!(
                            "cena={scene:?} facing={} k={} canal={c} cpu={} gpu={}",
                            at.facing, at.curvature, esperado[c], got[c]
                        );
                    }
                    if ulp > pior_ulp {
                        pior_ulp = ulp;
                        onde_ulp = format!(
                            "cena={scene:?} facing={} k={} canal={c} cpu={} gpu={}",
                            at.facing, at.curvature, esperado[c], got[c]
                        );
                    }
                }
            }
        }
        println!("  {rot:<36} {pior:>12.4e}   {pior_ulp:>9}");
        if pior_ulp > 0 {
            println!("      pior |Δ| em: {onde}");
            println!("      pior ULP em: {onde_ulp}");
        }
        let exacta = EXACTAS.contains(&rot);
        if exacta && pior_ulp > 0 {
            acusadas.push(format!(
                "«{rot}» TINHA de ser exacta e deu {pior_ulp} ULP (|Δ| {pior:.3e}) — {onde_ulp}"
            ));
        }
        // ⚠️ **A bateria classifica-se pelo que ela ACENDE, não pelo nome** — uma bateria nova cai
        // no tecto certo sem ninguém a inscrever numa lista.
        let acende_o_contorno = style.rim.strength != 0.0;
        let tecto = if acende_o_contorno {
            tecto_pow
        } else {
            TECTO_CONTRACCAO
        };
        if !exacta && pior_ulp > tecto {
            acusadas.push(format!(
                "«{rot}» passou o tecto medido: {pior_ulp} ULP > {tecto} — {onde_ulp}"
            ));
        }
    }
    println!();
    assert!(
        acusadas.is_empty(),
        "a LEI do estilo diverge para além do que a linguagem impõe: {acusadas:?}"
    );
}

/// O arnês que pergunta às sub-leis, uma a uma — ver [`de_quem_e_o_ulp_da_lei_do_estilo`].
/// ⚠️ **Sem uniforme**, e isso não é economia: o `naga` **poda** um binding que o ponto de entrada
/// não alcança, e o grupo passa a ter menos entradas do que o arnês declara — `Number of bindings
/// in bind group descriptor (3) does not match ... (2)`. *Um binding que ninguém lê deixa de
/// existir, e o erro fala do descritor e não da causa.*
const ARNES_SUB: &str = r"
@group(0) @binding(0) var<storage, read> entrada: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> saida: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn avalia(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i >= arrayLength(&entrada)) { return; }
    let a = entrada[i];
    // x = a luminância (SÓ produtos somados: o candidato número um a ser FUNDIDO pela placa)
    // y = o `pow` do contorno, nu (a única chamada de biblioteca da lei)
    // z = uma multiplicação-soma solta e isolada: `a.x * a.y + a.z`
    saida[i] = vec4<f32>(st_luma(a.xyz), pow(a.x, a.w), a.x * a.y + a.z, 0.0);
}
";

/// ⭐⭐⭐ **DE QUEM É O ULP: da PLACA A FUNDIR, ou do `pow`?**
///
/// # ⛔⛔⛔ O que este gate achou, e porque nenhuma leitura do código o podia achar
///
/// A [`ph2d_style`] declara no cabeçalho, como **lei**, que *nenhuma conta dela usa `f32::mul_add`*,
/// porque *«escrever a forma fundida de um lado e a solta do outro é uma divergência por
/// construção … e ela apareceria num **byte, um dia, num pixel**»*. Há até um gate a varrer o
/// ficheiro por aquele nome, e ele está **verde**.
///
/// ⚠️⚠️ **E a divergência acontece na mesma.** A lei manda em quem ESCREVE o código; ela não manda
/// no **compilador da placa**, que é livre de contrair `a*b + c` num `fma` — o WGSL não o proíbe e
/// nenhum backend o promete. ⇒ *uma lei sobre o que se escreve não é uma lei sobre o que corre.*
///
/// As três colunas separam as três hipóteses, e cada uma reprova sozinha:
///
/// | coluna | o que ela pergunta | o que a resposta quer dizer |
/// |---|---|---|
/// | `st_luma` | três produtos somados | se bate com a forma **fundida** e não com a solta ⇒ a placa contrai |
/// | `pow` | a única chamada de biblioteca | se diverge ⇒ `powf` (libm) ≠ `pow` (`exp2·log2`) |
/// | `a*b + c` | uma multiplicação-soma **isolada** | o controlo mínimo da primeira |
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn de_quem_e_o_ulp_da_lei_do_estilo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let mut amostras: Vec<[f32; 4]> = Vec::new();
    for &x in &[0.013f32, 0.18, 0.37, 0.5, 0.9, 1.0, 3.7, 40.0] {
        for &y in &[0.021f32, 0.19, 0.44, 0.63, 1.0, 2.5] {
            for &z in &[0.007f32, 0.11, 0.55, 1.0, 9.0] {
                for &w in &[0.0f32, 0.5, 1.0, 2.5, 3.0, 8.0, 64.0] {
                    amostras.push([x, y, z, w]);
                }
            }
        }
    }
    assert!(amostras.len() > 1_000, "grelha pobre: {}", amostras.len());
    let fonte = format!("{}\n{ARNES_SUB}", ph2d_style::wgsl::source());
    let guarda = t.lock().expect("o traçador");
    let (device, queue) = guarda.parts();
    let saida =
        ph2d_field_gpu::probe::evaluate(device, queue, &fonte, "avalia", &[], &[], &amostras, 1);
    drop(guarda);

    let l = ph2d_style::LUMA;
    let (mut luma_solto, mut luma_fundido) = (0usize, 0usize);
    let (mut ma_solto, mut ma_fundido) = (0usize, 0usize);
    let (mut pow_igual, mut pow_ulp_max) = (0usize, 0u32);
    for (i, a) in amostras.iter().enumerate() {
        let got = saida[i];
        // (1) a luminância, nas DUAS redacções — a solta é a que o Rust corre.
        let solto = l[0] * a[0] + l[1] * a[1] + l[2] * a[2];
        let fundido = l[2].mul_add(a[2], l[1].mul_add(a[1], l[0] * a[0]));
        luma_solto += usize::from(got[0].to_bits() == solto.to_bits());
        luma_fundido += usize::from(got[0].to_bits() == fundido.to_bits());
        // (2) o `pow` nu.
        let p = a[0].powf(a[3]);
        let u = got[1].to_bits().abs_diff(p.to_bits());
        pow_igual += usize::from(u == 0);
        pow_ulp_max = pow_ulp_max.max(u);
        // (3) a multiplicação-soma isolada.
        let ms_solto = a[0] * a[1] + a[2];
        let ms_fundido = a[0].mul_add(a[1], a[2]);
        ma_solto += usize::from(got[2].to_bits() == ms_solto.to_bits());
        ma_fundido += usize::from(got[2].to_bits() == ms_fundido.to_bits());
    }
    let n = amostras.len();
    println!("\n  {n} amostras");
    println!("  st_luma   bate com a forma SOLTA .... {luma_solto:>6} / {n}");
    println!("            bate com a forma FUNDIDA .. {luma_fundido:>6} / {n}");
    println!("  a*b + c   bate com a forma SOLTA .... {ma_solto:>6} / {n}");
    println!("            bate com a forma FUNDIDA .. {ma_fundido:>6} / {n}");
    println!(
        "  pow(x,w)  igual ao `powf` do Rust ... {pow_igual:>6} / {n}   pior {pow_ulp_max} ULP"
    );
    println!();
    // ⚠️ **Nenhuma asserção de veredito aqui** — este é o instrumento que ATRIBUI, e as três
    // colunas são o achado. Quem decide o que fazer com a contracção é o dono do produto.
    assert!(
        luma_solto + luma_fundido > 0,
        "o `st_luma` do dispositivo não bate com NENHUMA das duas redacções — procure uma 3.ª lei"
    );
}

/// ⭐⭐⭐ **AS DUAS PORTAS SANEIAM O MESMO CONJUNTO** — e a prova é que o dispositivo recebe
/// **exactamente** `s.sanitized()`, campo a campo.
///
/// ⚠️ **Um campo saneado só de um lado é uma divergência que só aparece com valor mau** — o painel
/// entrega um `NaN`, a CPU volta ao valor de fábrica e a placa pinta o `NaN`. *Um gate que
/// comparasse só a saída com botões BONS nunca o veria.*
///
/// ⛔ A prova não é «a `pack` chama a `sanitized`» (isso é uma leitura do código): é a **volta**,
/// que compara os vinte números com a struct saneada. *Uma arrumação sem volta é uma afirmação que
/// ninguém pode contradizer.*
#[test]
fn as_duas_portas_saneiam_o_mesmo_conjunto() {
    let maus = [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        0.0,
        1e30,
        -1e30,
        1000.0,
    ];
    let mut vistos = 0usize;
    for &m in &maus {
        // Um estilo em que TODOS os campos levam o valor mau — se algum não for saneado por uma das
        // portas, a volta acusa nesse campo.
        let sujo = Style {
            rim: Rim {
                color: [m; 3],
                strength: m,
                width: m,
            },
            curvature: Curvature {
                convex: [m; 3],
                concave: [m; 3],
                sharpness: m,
            },
            zones: Zones {
                shadow: [m; 3],
                highlight: [m; 3],
                pivot: m,
            },
            indirect_saturation: m,
        };
        let pelo_dispositivo = ph2d_style::wgsl::unpack(&ph2d_style::wgsl::pack(&sujo));
        let pela_cpu = sujo.sanitized();
        assert_eq!(
            pelo_dispositivo, pela_cpu,
            "com o valor {m} as duas portas entregam estilos diferentes"
        );
        // ⭐ E a porta é IDEMPOTENTE também pelo lado do dispositivo — sem isto, um bloco já
        // arrumado que voltasse a passar pela porta podia mudar.
        assert_eq!(
            ph2d_style::wgsl::pack(&sujo),
            ph2d_style::wgsl::pack(&pela_cpu),
            "a arrumação do sujo e a do saneado diferem com o valor {m}"
        );
        vistos += 1;
    }
    assert_eq!(vistos, maus.len(), "a varredura não correu toda");
}

/// ⭐⭐ **O CENSO DOS RAMOS: quantos `if` tem o gémeo em WGSL que o Rust não tem?**
///
/// ⚠️ Um ramo a mais de um lado é a forma mais barata de os dois motores divergirem **só num valor
/// de fronteira** — e uma paridade de imagem varrida com botões bonitos nunca lá chega. O censo
/// **nomeia** os que existem; a grelha do gate irmão é quem prova que eles concordam.
///
/// ⛔ A lista é uma catraca: um ramo NOVO no WGSL reprova até alguém o pôr aqui com a razão dele.
#[test]
fn o_censo_dos_ramos_do_gemeo_em_wgsl() {
    let src = ph2d_style::wgsl::source();
    // Os ramos DECLARADOS, com a razão de cada um.
    let declarados = [
        // as duas metades da cerca de geometria por pixel — o Rust tem `is_finite`, o WGSL não.
        ("if (v != v) { return 0.0; }", "o `NaN` por comparação"),
        (
            "if (v > 3.40282347e38 || v < -3.40282347e38) { return 0.0; }",
            "o infinito por magnitude",
        ),
        (
            "if (e.sombra.w > 0.0) { grazing = pow(base, e.sombra.w); }",
            "o `pow(0,0)` indefinido em WGSL, que o `powf` do Rust dá `1`",
        ),
    ];
    for (agulha, razao) in declarados {
        assert!(
            src.contains(agulha),
            "o ramo «{agulha}» ({razao}) desapareceu do gémeo — ou ele mudou, ou este censo mente"
        );
    }
    let ifs = src.matches("if (").count();
    assert_eq!(
        ifs,
        declarados.len(),
        "o gémeo em WGSL tem {ifs} ramos e o censo declara {} — um ramo que o Rust não tem é uma \
         divergência de fronteira à espera; ponha-o na lista com a razão",
        declarados.len()
    );
}
