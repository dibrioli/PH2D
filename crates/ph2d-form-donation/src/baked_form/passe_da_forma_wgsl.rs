//! ⭐⭐⭐ **O TEXTO DO SHADER da luz da forma** — e só ele.
//!
//! Irmão (`#[path]`) do [`super`], que é quem o CORRE. ⚠️ **O corte é por responsabilidade e não
//! por tamanho:** aqui vive WGSL (um `&str`, mais os dois stubs do ambiente e a montagem), e ali
//! vive o passe de computação que o compila, empacota o uniform e despacha. *Duas coisas que só um
//! `format!` liga.*
//!
//! ⚠️ A ordem da composição, os dois stubs e a razão de cada um estão no cabeçalho do [`super`].

use super::{MAX_LAMPADAS, gemeo};

/// ⭐ **O PONTO DE ENTRADA — e a única coisa deste ficheiro que é WGSL.**
///
/// Ver o cabeçalho do módulo para a ordem da composição e para os dois stubs do ambiente.
pub(super) const ENTRADA: &str = r#"
struct Globais {
    material: Mat,
    // ⚠️ `rgb` = RESERVA DECLARADA (era o ambiente, que hoje vive na ranhura do ambiente e é função da
    // normal); `a` = os stops de exposição do olhar. *Uma posição sem dono e sem régua é onde o
    // campo seguinte aterra por engano* — o `Globais::novo` deixa-a a ZERO, e há gate.
    olhar: vec4<f32>,
    // ⭐ O CÉU (`ph2d_form_pbr::Ceu`), em `rgb`: a base da rampa e a inclinação dela.
    //
    // ⚠️ Ele viaja como DADOS e não como constantes geradas: a rampa é derivada do RIG (ver o
    // `ceu_do_rig`), logo ela muda quando o artista mexe numa lâmpada — e uma constante no shader
    // pediria uma recompilação por gesto.
    ceu_base: vec4<f32>,
    ceu_inclinacao: vec4<f32>,
    // x = o código da vista (`ph2d_view_transform::wgsl::view_code`).
    // y = A MATÉRIA É A FORMA (0 ou 1) — ver `BakedForm::materia_da_forma`. ⚠️ Ele mora no `vec4`
    //     da vista e não num slot novo de propósito: `z` e `w` continuam reserva DECLARADA, e o
    //     `Globais::FLOATS` não se mexe, logo a disposição que o gate prende fica intacta.
    vista: vec4<u32>,
    lampadas: Lampadas,
};

@group(0) @binding(0) var base_tex: texture_2d<f32>;
@group(0) @binding(1) var form_tex: texture_2d<f32>;
@group(0) @binding(2) var occ_tex: texture_2d<f32>;
@group(0) @binding(3) var saida: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> g: Globais;

// ⭐⭐⭐ **IEC 61966-2-1, linear → sRGB, por canal** — o GEMEO do
// `ph2d_color::linear_to_srgb_unit`, que é quem a `ph2d_form_pbr::imagem` chama.
//
// ⚠️ **Ele NÃO pode vir do backend:** a saída é um `rgba8unorm` (armazenamento LINEAR) e o
// `textureStore` não codifica nada. Um `rgba8unorm-srgb` como alvo de armazenamento **não é
// suportado** pelo `wgpu`, logo a curva é nossa — como a quantização já era, e pela mesma razão.
//
// ⚠️ A forma é a que esta casa já escreve em `band_blit.wgsl` e `compositor.wgsl` (`step` + `mix`,
// sem ramo), e o `clamp` vem primeiro para o `pow` nunca ver um negativo.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let cutoff = step(vec3<f32>(0.04045), c);
    return mix(lo, hi, cutoff);
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let safe = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = safe * 12.92;
    let hi = 1.055 * pow(safe, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let cutoff = step(vec3<f32>(0.0031308), safe);
    return mix(lo, hi, cutoff);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(saida);
    if (gid.x >= dim.x || gid.y >= dim.y) { return; }
    let p = vec2<i32>(i32(gid.x), i32(gid.y));

    // ⚠️ A textura é `rgba8unorm` e NUNCA `…Srgb` — logo o `textureLoad` entrega o **código**
    // normalizado e não a luz, e quem descodifica somos nós. ⛔ Pedir a ranhura como `…Srgb` para o
    // backend o fazer **não serve**: o `base` vai ao device pelo mesmo caminho do passe da TINTA,
    // que é RELATIVO e trata estes bytes como códigos de propósito. *Cada lei paga a convenção
    // dela na porta dela.* Ver o cabeçalho da [`ph2d_form_pbr::imagem`].
    let px = textureLoad(base_tex, p, 0);
    let f = textureLoad(form_tex, p, 0);
    let occ = textureLoad(occ_tex, p, 0).r;
    // ⭐⭐⭐⭐ **SEM ARTE, o albedo é o NEUTRO** — ver `BakedForm::materia_da_forma`. É exactamente
    // o branco que o `veste_a_forma` grava no `base`, logo na pose em que o bake correu isto não
    // muda um bit; o que ele compra é a peça poder VIRAR sem arrastar o recorte do primeiro gesto.
    let materia_e_a_forma = g.vista.y != 0u;
    let albedo = select(srgb_to_linear(px.rgb), vec3<f32>(1.0), materia_e_a_forma);

    // ⚠️ **O céu entra pelos `var<private>` ANTES da chamada** — ver o `CEU_NA_RANHURA` para
    // porque ele não pode ler o `g` directamente.
    ceu_base = g.ceu_base.rgb;
    ceu_inclinacao = g.ceu_inclinacao.rgb;

    let c = forma_acende_texel(
        g.material,
        f.xyz,
        albedo,
        f.w,
        occ,
        g.lampadas,
        g.olhar.a,
        g.vista.x,
    );

    // ⚠️ **O ALFA atravessa intacto** — ele é a silhueta do SPRITE, e uma lei de luz que lhe
    // tocasse mudaria o RECORTE do objecto ao mover a lâmpada.
    // ⭐⭐⭐⭐ **A não ser que não haja sprite nenhum por baixo:** quando a matéria é a forma, o
    // alfa é a COBERTURA DESTE quadro. Sem esta linha a peça roda por baixo do recorte que o
    // primeiro bake lhe deu — o report da «máscara» (2026-09-21, com foto).
    // ⚠️ E a quantização é nossa, não do backend — ver o cabeçalho do módulo.
    // ⭐ A CURVA vem ANTES dela, e a ordem é load-bearing: quantizar o linear e codificar depois
    // dava os degraus do linear espalhados pela curva (bandas visíveis no escuro).
    let alfa = select(px.a, floor(clamp(f.w, 0.0, 1.0) * 255.0 + 0.5) / 255.0, materia_e_a_forma);
    let q = floor(linear_to_srgb(c) * 255.0 + 0.5) / 255.0;
    textureStore(saida, p, vec4<f32>(q, alfa));
}
"#;

/// ⭐⭐⭐ **O CÉU DA CASA, em WGSL — as duas funções que a ranhura `{ENV}` pede.**
///
/// ⚠️ **As três constantes são GERADAS da [`ph2d_light`] e nunca transcritas**, que é o que impede a
/// segunda cópia de um número. O `{:?}` de um `f32` é a representação mais curta que faz round-trip,
/// e é um literal válido de WGSL.
///
/// ⚠️ **O `fma` é EXPLÍCITO e isso é para a paridade**: a [`ph2d_light::env_ambient`] escreve
/// `ENV_SLOPE[i].mul_add(up, ENV_BASE[i])`, que é uma multiplicação-soma com **um** arredondamento.
/// Escrito como `base - slope * n.y`, o WGSL fica livre de contrair ou não — e esta casa já mediu
/// uma placa a contrair `a*b + c` num `fma` onde o fonte não o pedia. *Pedir o `fma` nos dois lados
/// é a única forma de a igualdade não depender do compilador.*
///
/// ⭐⭐⭐ **O `env_radiance` DEIXOU DE SER ZERO em 2026-09-20 (a coluna B3)** — e a redacção anterior
/// desta linha dizia *«ele fica a ZERO de propósito: nada nesta lei o chama»*. Chama.
pub(super) const CEU_NA_RANHURA: &str = r#"
// **O CEU** — a rampa linear na altura da TELA (`ph2d_form_pbr::Ceu`).
//
// ⛔⛔ Os dois valores chegam por `var<private>` e NAO por uma leitura do `g`, e a razao e' a ORDEM
// da composicao: esta ranhura e' preenchida DENTRO da fonte da lei, que vem antes do `ENTRADA` —
// logo o `struct Globais` ainda nao existe aqui. Quem os escreve e' o `cs_main`, no topo.
var<private> ceu_base: vec3<f32>;
var<private> ceu_inclinacao: vec3<f32>;

// ⚠️ O ceu e' o topo da TELA, e neste referencial (CANVAS) o topo e' `-y`.
// ⚠️ **`fma` EXPLICITO** — o gemeo em Rust usa `mul_add`, que tem UM arredondamento; escrito solto,
// a igualdade ao bit passaria a depender de o compilador do WGSL contrair a multiplicacao-soma.
fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    return fma(ceu_inclinacao, vec3<f32>(-n.y), ceu_base);
}

// ⭐⭐⭐ **A ESPELHADA PRE'-FILTRADA** — o gemeo da `ph2d_form_pbr::Ceu::radiancia`.
//
// ⚠️ **O `alpha` NAO e' lido e o `shrink` e'**: o encolhimento do lobulo e' constante por MATERIAL e
// viaja PRONTO dentro do `Mat` empacotado. Correr aqui o logaritmo que o produz seria por a mesma
// conta a dar o mesmo numero um milhao de vezes. ⛔ O NOME da funcao que o calcula nao se escreve
// neste texto: ha' gate a varre'-lo, e uma agulha num comentario le-se igual a uma chamada.
//
// ⚠️ **`1.5` e' `1/(2/3)`**: o `ceu_inclinacao` e' a inclinacao da IRRADIANCIA, que ja' traz o `A1`
// do lobulo cosseno la' dentro; a radiancia quer a rampa crua. ⚠️ E a ASSOCIACAO e' a do gemeo em
// Rust — o `1.5 * inc` primeiro, e so' depois o `fma`.
fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    let up = shrink * -dir.y;
    return fma(1.5 * ceu_inclinacao, vec3<f32>(up), ceu_base);
}
"#;

/// ⭐⭐ **O corpo do shader, composto** — e uma função pública porque o gate o quer **sem placa**.
///
/// ⚠️ Um WGSL que ninguém compila é prosa, e todo gate desta casa que olha para um shader precisa de
/// adapter e é `#[ignore]`. Com a montagem numa porta, a `naga` pode parsá-la e validá-la como
/// aritmética — que é o que apanhou os dois defeitos que o gémeo tinha antes de haver um pixel.
#[must_use]
pub fn fonte() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        ph2d_view_transform::wgsl::SOURCE,
        gemeo::SOURCE_DA_LEI.replace(gemeo::ENV_SLOT, CEU_NA_RANHURA),
        gemeo::SOURCE.replace(gemeo::CAP_SLOT, &format!("{MAX_LAMPADAS}u")),
        ENTRADA,
    )
}
