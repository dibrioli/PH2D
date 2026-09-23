//! ⭐⭐⭐ **O PINTOR DE MATCAP, em WGSL** — o gémeo exacto do
//! [`ph2d_field_render::shade_with`], e o passe mais barato deste módulo.
//!
//! # ⭐⭐⭐ Porque ele é um passe PRÓPRIO e não um modo do pintor de material
//!
//! Três razões, todas medidas:
//!
//! 1. **ARMAZÉNS.** O [`crate::paint`] liga **`9`** (`crate::paint::ARMAZENS`) e o piso garantido
//!    do WebGPU é **`8`** — numa placa que fique no piso o caminho de render **não corre**. Este
//!    passe liga **`8`** (os seis do grupo `0`, a saída e a fotografia), logo **cabe no piso**.
//!    ⚠️ *É esta a diferença entre «o melhor render» e «o melhor render onde há folga»:* o modo de
//!    omissão do modelador passa a correr na placa em toda placa conforme, e o de material não.
//! 2. **UM MATCAP NÃO LÊ O CAMPO DA PEÇA.** A lei dele é `uv = n.xy·0,5 + 0,5` sobre uma
//!    fotografia: ele quer o **NORMAL** e mais nada — nem o ponto, nem o material, nem o céu, nem a
//!    curvatura. ⇒ o texto deste shader **não tem [`crate::FIELD_SLOT`]**, e o cache de pipelines,
//!    cuja chave é o TEXTO, acerta para sempre: *acrescentar uma forma não recompila o modo de
//!    omissão do modelador.* Há gate ([`o_passe_do_matcap_nao_le_o_campo_da_peca`]).
//! 3. **COMPILAR É O CARO** (`docs/Render3d/03` §W9). Arrastar o pintor de material para o caminho
//!    de omissão traria o OpenPBR, as sondas, o estilo e a lei do dono para um quadro que não lê
//!    nenhum deles.
//!
//! # ⚠️ O normal já chega em espaço de VISTA
//!
//! O `centro[i].yzw` é `(n·right, n·up, n·fwd)` — ver o `trace_wgsl`, linha do `vec4` da normal —,
//! que é **exactamente** a convenção em que o matcap vive (o doc do
//! [`ph2d_field_render::shade_with`] escreve-a). ⇒ não há base nenhuma para aplicar aqui, e é por
//! isso que este passe não precisa de saber onde a câmera está.
//!
//! # ⚠️ Amostra BILINEAR À MÃO, sobre um ARMAZÉM e não sobre uma textura
//!
//! A CPU faz a bilinear à mão sobre `f32` linear, indexando `(y·lado + x)·3`. ⇒ este passe usa a
//! **mesma aritmética, no mesmo tipo de dados**, e a paridade é por CONSTRUÇÃO e não por um
//! número.
//!
//! ⛔ **Uma textura foi considerada e não escolhida**, e a troca está dita: ela custaria uma
//! ranhura de *textura* em vez de armazém (`7` de `8` em vez de `8`), e paga por isso um
//! **repacote 3 → 4 canais** — uma transformação de dados a mais entre a lei da CPU e a do
//! dispositivo, que é exactamente onde uma última casa se perde. Um amostrador de hardware está
//! fora por duas razões: ele daria outra última casa, e `rgba32float` **filtrável** pede uma
//! extensão que o piso do WebGPU não garante. ⏳ *Se alguma placa precisar da 8.ª ranhura de volta,
//! a textura é a cura e o repacote é o preço dela.*

/// O corpo. Composto pelo [`super::matcap::fonte`] com as declarações do grupo `0`, o olhar e o
/// empacotamento.
pub(crate) const MATCAP: &str = r"
// ⭐ **O que só este passe lê.** ⚠️ A ordem é a lei — ver o `super::matcap::arruma`.
struct Mat {
    // `x` = a exposição em paragens. Os outros três são reserva, e o `arruma` deixa-os a ZERO.
    olhar: vec4<f32>,
    // ⭐ O fundo em LINEAR e PRÉ-MULTIPLICADO, com o alfa em `w` — é ele que entra na média de um
    // pixel de borda, exactamente como o `bg` do `shade_with`.
    fundo_lin: vec4<f32>,
    // `x` = o código da vista · `y` = quantas bordas · `z` = os bytes EXACTOS do fundo ·
    // `w` = o lado do matcap em texels.
    modo: vec4<u32>,
};
@group(1) @binding(0) var<uniform> mc: Mat;
@group(1) @binding(1) var<storage, read_write> saida: array<u32>;
// ⭐ A fotografia, em LINEAR e `lado × lado × 3` — o MESMO vector que a CPU indexa.
@group(1) @binding(2) var<storage, read> mc_img: array<f32>;

fn mc_texel(x: i32, y: i32) -> vec3<f32> {
    let t = (u32(y) * mc.modo.w + u32(x)) * 3u;
    return vec3<f32>(mc_img[t], mc_img[t + 1u], mc_img[t + 2u]);
}

// ⭐⭐⭐ **A BILINEAR, linha a linha a do `ph2d_field_render::shade::Matcap::sample`.**
//
// ⚠️ O `−0,5` está lá porque o texel é uma ÁREA e a coordenada dele é o CENTRO dela: sem isso a
// imagem desloca-se meio texel e as bordas do matcap espelham-se erradas.
fn mc_sample(u: f32, v: f32) -> vec3<f32> {
    let lado = f32(mc.modo.w);
    let ultimo = lado - 1.0;
    let fx = clamp(u * lado - 0.5, 0.0, ultimo);
    let fy = clamp(v * lado - 0.5, 0.0, ultimo);
    let x0 = i32(floor(fx));
    let y0 = i32(floor(fy));
    let x1 = min(x0 + 1, i32(ultimo));
    let y1 = min(y0 + 1, i32(ultimo));
    let tx = fx - f32(x0);
    let ty = fy - f32(y0);
    let a = mc_texel(x0, y0);
    let b = mc_texel(x1, y0);
    let c = mc_texel(x0, y1);
    let d = mc_texel(x1, y1);
    let cima = a + (b - a) * tx;
    let baixo = c + (d - c) * tx;
    return cima + (baixo - cima) * ty;
}

// ⭐⭐⭐ **A COR DE UMA NORMAL DE VISTA, já sob o OLHAR** — o
// `look.apply(m.colour(n))` do `shade_with`, na mesma ordem: amostrar e **depois** transformar.
//
// ⚠️ O olhar vale para o matcap como no Blender (a *Color Management* dele pinta o modo sólido): a
// exposição e a vista são da CENA, e uma vista em matcap ao lado de uma em render que as ignorasse
// mostraria duas cenas diferentes.
fn mc_cor(n: vec3<f32>) -> vec3<f32> {
    let u = clamp(n.x * 0.5 + 0.5, 0.0, 1.0);
    let v = clamp(1.0 - (n.y * 0.5 + 0.5), 0.0, 1.0);
    return vt_to_display(mc_sample(u, v), mc.olhar.x, mc.modo.x);
}

@compute @workgroup_size(8, 8, 1)
fn pinta_matcap(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    if (c.x < 0.0) {
        // ⚠️ **COPIADO, e não passado pela conversão** — a cerca que a CPU declara: um pixel de
        // fundo puro tem de sair **exactamente** com os bytes que o chamador pediu, senão a cor do
        // fundo passa a depender da precisão de uma tabela.
        saida[i] = mc.modo.z;
        return;
    }
    saida[i] = empacota(vec4<f32>(mc_cor(c.yzw), 1.0));
}

// ⭐⭐⭐ **A BORDA: quatro sub-amostras, resolvidas em COR e não pela média das NORMAIS.**
//
// ⚠️ A diferença aparece exactamente onde uma superfície passa à frente de outra: ali as duas
// normais podem ser quase opostas, e a média delas aponta para um sítio do matcap que não é nenhuma
// das duas cores. *Média de normais é interpolar a GEOMETRIA; o que se quer é interpolar o que se
// vê* — a nota é do `shade_with`, e este passe é o gémeo dela.
@compute @workgroup_size(64, 1, 1)
fn pinta_matcap_bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    let slot = g.x;
    if (slot >= mc.modo.y) { return; }
    let i = bitcast<u32>(borda[slot * 5u].x);
    if (i >= s.w * s.h) { return; }
    var acc = vec4<f32>(0.0);
    for (var j = 0u; j < 4u; j = j + 1u) {
        let q = borda[slot * 5u + 1u + j];
        var cor = mc.fundo_lin;
        if (q.x >= 0.0) { cor = vec4<f32>(mc_cor(q.yzw), 1.0); }
        acc = acc + cor * 0.25;
    }
    saida[i] = empacota(acc);
}
";
