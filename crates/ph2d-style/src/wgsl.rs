//! ⭐⭐⭐ **A CAMADA DE ESTILO, em WGSL** — a MESMA lei do [`crate`], para quem pinta no dispositivo.
//!
//! # ⚠️ Porque este ficheiro existe, e o que ele NÃO é
//!
//! Ele **não** é uma segunda redacção da lei: é uma transcrição linha a linha, e a única razão de
//! ele ser aceitável é que o gémeo de CPU é a testemunha — o consumidor fecha os dois com um gate de
//! paridade, como o [`ph2d_view_transform::wgsl`] já faz.
//!
//! ⛔⛔ **NENHUMA constante é escrita aqui à mão.** Os pesos da luminância são **substituídos** do
//! sítio que os declara ([`crate::LUMA`]) por [`source`] — *uma constante transcrita é uma
//! divergência à espera de um dia em que alguém mexa na outra*, que é a lei que o pintor do campo já
//! aplica ao `BLUR_COS` e ao `PISO_LUZ`.
//!
//! ⭐ **E as outras duas — o tecto da largura e o piso do pivô — NÃO aparecem aqui de todo**, porque
//! elas são cercas de BOTÃO e vivem na porta ([`crate::Style::sanitized`]), do lado da CPU. *Uma
//! cerca que corre uma vez por quadro não tem nada que fazer num shader que corre por pixel.*
//!
//! # ⭐ E a ARRUMAÇÃO dos números também não se escreve duas vezes
//!
//! O bloco viaja como **cinco `vec4`** ([`PACKED`] floats), e quem o arruma é [`pack`] — uma função
//! só, de que o WGSL é o **leitor**. ⚠️ Cada `vec4` leva uma COR nos três primeiros e um **escalar
//! sem parentesco** no quarto: as cores é que são naturalmente `vec3`, e os escalares enchem os
//! buracos que elas deixariam. *É arrumação de bytes, não uma agregação com sentido — e é por isso
//! que ela tem um gate de ida-e-volta em vez de um comentário a pedir cuidado.*

/// Quantos `f32` o bloco de estilo ocupa — **cinco `vec4`**.
///
/// ⚠️ O consumidor declara o campo como `array<vec4<f32>, 5>` ou como cinco campos nomeados; o que
/// não pode é contar este número de cabeça. *Uma contagem escrita de memória é como a arrumação
/// deixa de bater sem ninguém notar.*
pub const PACKED: usize = 20;

/// ⭐⭐⭐ **A ARRUMAÇÃO** — de [`crate::Style`] para os `f32` que o uniforme leva.
///
/// | `vec4` | `xyz` | `w` |
/// |---|---|---|
/// | `0` | [`crate::Rim::color`] | [`crate::Rim::strength`] |
/// | `1` | [`crate::Curvature::convex`] | [`crate::Curvature::sharpness`] |
/// | `2` | [`crate::Curvature::concave`] | [`crate::Zones::pivot`] |
/// | `3` | [`crate::Zones::shadow`] | [`crate::Rim::width`] |
/// | `4` | [`crate::Zones::highlight`] | [`crate::Style::indirect_saturation`] |
///
/// ⚠️ **Os `w` não pertencem à cor ao lado deles** (a largura do contorno viaja com a tinta das
/// sombras), e isso é deliberado: são cinco cores e cinco escalares, e emparelhá-los *por assunto*
/// exigiria um sexto `vec4` só para o quinto escalar. O gate de ida-e-volta é quem impede que a
/// escolha se perca.
///
/// ⭐⭐⭐ **ELA É A PORTA DO DISPOSITIVO, e por isso SANEIA** ([`crate::Style::sanitized`]): o
/// gémeo em WGSL não tem cerca nenhuma sobre os botões — ela vive na CPU, uma vez por quadro, dos
/// dois lados. ⛔ *Se o saneamento fosse do chamador, o dispositivo ficaria a um esquecimento de
/// distância de um bloco sujo, e um `NaN` num uniforme não deixa rasto nenhum no ecrã: pinta.*
#[must_use]
pub fn pack(s: &crate::Style) -> [f32; PACKED] {
    let s = &s.sanitized();
    [
        s.rim.color[0],
        s.rim.color[1],
        s.rim.color[2],
        s.rim.strength,
        s.curvature.convex[0],
        s.curvature.convex[1],
        s.curvature.convex[2],
        s.curvature.sharpness,
        s.curvature.concave[0],
        s.curvature.concave[1],
        s.curvature.concave[2],
        s.zones.pivot,
        s.zones.shadow[0],
        s.zones.shadow[1],
        s.zones.shadow[2],
        s.rim.width,
        s.zones.highlight[0],
        s.zones.highlight[1],
        s.zones.highlight[2],
        s.indirect_saturation,
    ]
}

/// **A volta** — dos `f32` do uniforme para o [`crate::Style`].
///
/// ⚠️ Ela existe **para o gate**, e não para o produto: o dispositivo lê os floats e nunca
/// reconstrói a struct. *Uma arrumação sem volta é uma afirmação que ninguém pode contradizer* — com
/// ela, a prova de que a tabela acima está certa é `desempacota(pack(s)) == s`, que sangra à primeira
/// troca de dois campos.
#[must_use]
pub fn unpack(v: &[f32; PACKED]) -> crate::Style {
    crate::Style {
        rim: crate::Rim {
            color: [v[0], v[1], v[2]],
            strength: v[3],
            width: v[15],
        },
        curvature: crate::Curvature {
            convex: [v[4], v[5], v[6]],
            concave: [v[8], v[9], v[10]],
            sharpness: v[7],
        },
        zones: crate::Zones {
            shadow: [v[12], v[13], v[14]],
            highlight: [v[16], v[17], v[18]],
            pivot: v[11],
        },
        indirect_saturation: v[19],
    }
}

/// ⭐ **O corpo do shader, com as constantes JÁ substituídas** dos sítios que as declaram.
///
/// O consumidor concatena isto no shader dele e chama `st_apply` / `st_saturate_indirect`. A struct
/// `Estilo` é declarada aqui — quem monta o uniforme só precisa de garantir que o campo dele tem a
/// arrumação do [`pack`].
#[must_use]
pub fn source() -> String {
    BODY.replace("{LUMA_R}", &lit(crate::LUMA[0]))
        .replace("{LUMA_G}", &lit(crate::LUMA[1]))
        .replace("{LUMA_B}", &lit(crate::LUMA[2]))
}

/// Um `f32` como literal de WGSL.
///
/// ⚠️ **O `{:?}` do Rust é o mais curto que volta ao MESMO `f32`**, que é exactamente o que uma
/// paridade ao bit precisa. ⛔ Mas ele pode devolver `1` para `1.0`, e o WGSL lê isso como um
/// **inteiro** — daí o remate: um literal sem `.` e sem expoente ganha `.0`.
fn lit(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

/// ⚠️ **Privado de propósito** — quem o quiser chama [`source`], que é a única forma de o obter com
/// as constantes certas. *Um `SOURCE` público com marcas por preencher é uma forma de alguém
/// esquecer uma.*
const BODY: &str = r"
// ⭐⭐⭐ A CAMADA DE ESTILO — o gémeo do `ph2d_style`, linha a linha.
//
// ⛔ Nenhuma conta aqui usa `fma`: o gémeo de CPU não usa `mul_add`, e uma multiplicação-soma
// FUNDIDA de um lado e SOLTA do outro é uma divergência por construção. Ver o cabeçalho da crate.
struct Estilo {
    // xyz = a cor do contorno · w = a força dele
    rim: vec4<f32>,
    // xyz = a tinta da ARESTA · w = a nitidez da curvatura
    convexo: vec4<f32>,
    // xyz = a tinta da COVA · w = o pivô das zonas
    concavo: vec4<f32>,
    // xyz = a tinta das SOMBRAS · w = a largura do contorno
    sombra: vec4<f32>,
    // xyz = a tinta das LUZES · w = a saturação da indirecta
    luz: vec4<f32>,
};

const ST_LUMA: vec3<f32> = vec3<f32>({LUMA_R}, {LUMA_G}, {LUMA_B});

// ⚠️⚠️ **NÃO há cerca nenhuma sobre os BOTÕES aqui, e isso é a arquitectura:** eles chegam saneados
// pelo `ph2d_style::wgsl::pack`, uma vez por quadro. O que esta função guarda são as **duas entradas
// de GEOMETRIA por pixel** — uma normal degenerada entrega `NaN` a quem o não espera.
//
// `NaN` por COMPARAÇÃO, porque o WGSL não tem `is_nan` e `v != v` só é verdade para ele. ⛔ E o
// sinal SOBREVIVE: a entrada mais importante desta lei é a curvatura, e o sinal dela é a lei
// inteira.
fn st_sane(v: f32) -> f32 {
    if (v != v) { return 0.0; }
    if (v > 3.40282347e38 || v < -3.40282347e38) { return 0.0; }
    return v;
}

// A luminância de cena, nunca negativa e sempre finita — o tecto impede o `inf/inf` do `st_graded`.
fn st_luma(rgb: vec3<f32>) -> f32 {
    let c = max(vec3<f32>(st_sane(rgb.x), st_sane(rgb.y), st_sane(rgb.z)), vec3<f32>(0.0));
    return max(st_sane(ST_LUMA.x * c.x + ST_LUMA.y * c.y + ST_LUMA.z * c.z), 0.0);
}

// (1) A tinta da curvatura. ⚠️ Sem ramo: só um dos dois pesos pode ser diferente de zero.
fn st_curvature_tinted(e: Estilo, rgb: vec3<f32>, curvature: f32) -> vec3<f32> {
    let c = clamp(st_sane(curvature) * e.convexo.w, -1.0, 1.0);
    let wc = max(c, 0.0);
    let wv = max(-c, 0.0);
    // `1 + (t − 1)·w`, e nunca `mix`: com a tinta em `1` o parêntesis é exactamente `0`.
    let tint = vec3<f32>(1.0)
        + (e.convexo.xyz - vec3<f32>(1.0)) * wc
        + (e.concavo.xyz - vec3<f32>(1.0)) * wv;
    return rgb * tint;
}

// (2) A luz de contorno.
fn st_rim_lit(e: Estilo, rgb: vec3<f32>, facing: f32) -> vec3<f32> {
    let base = 1.0 - clamp(st_sane(facing), 0.0, 1.0);
    // ⛔ **O ramo do expoente ZERO não é conveniência: o `pow(0, 0)` é INDEFINIDO em WGSL** e o
    // `powf` do Rust devolve `1`. Sem ele os dois motores discordariam na silhueta exacta de uma
    // peça com o contorno em faixa larga — um pixel, e não reproduzível entre placas.
    var grazing = 1.0;
    if (e.sombra.w > 0.0) { grazing = pow(base, e.sombra.w); }
    let k = e.rim.w * grazing;
    return rgb + e.rim.xyz * k;
}

// (3) A grade por zona.
fn st_graded(e: Estilo, rgb: vec3<f32>) -> vec3<f32> {
    let l = st_luma(rgb);
    let h = l / (l + e.concavo.w);
    // `s + (t − s)·h`: com as duas tintas iguais o parêntesis é `0` e o resultado é a tinta, exacta.
    let tint = e.sombra.xyz + (e.luz.xyz - e.sombra.xyz) * h;
    return rgb * tint;
}

// ⭐⭐⭐ A LEI INTEIRA, sobre a luz que sai do ponto — em linear de CENA, antes do olhar.
fn st_apply(e: Estilo, scene: vec3<f32>, facing: f32, curvature: f32) -> vec3<f32> {
    return st_graded(e, st_rim_lit(e, st_curvature_tinted(e, scene, curvature), facing));
}

// ⭐⭐⭐ A saturação da luz INDIRECTA — chamada dentro do sombreador, sobre a parcela indirecta.
// ⚠️ `rgb·s + luma·(1 − s)`: só esta forma devolve `rgb` ao bit em `s = 1`, que é a fábrica.
fn st_saturate_indirect(e: Estilo, rgb: vec3<f32>) -> vec3<f32> {
    let s = e.luz.w;
    let l = st_luma(rgb) * (1.0 - s);
    return rgb * s + vec3<f32>(l);
}
";
