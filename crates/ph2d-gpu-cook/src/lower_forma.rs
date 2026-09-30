//! ⭐⭐⭐ **O BAIXAMENTO DAS FORMAS** (doc 121 do Motion, W3) — o gémeo do [`crate::lower`] para
//! as linhas que carregam uma GEOMETRIA viva (`geometry_id > 0,5`: a `source.shape`, os glifos do
//! `source.text`, a fita do L-System).
//!
//! Até esta wave a placa não tinha rota para um `geometry_id`: a ponte recusava pelo TIPO do nó
//! qualquer grafo com uma forma viva, e a simulação inteira corria na CPU só porque a fronteira a
//! arrastava (doc 120 §8.1: `~80 %` do tique). Agora cada linha de forma é escrita aqui como uma
//! cópia do passe instanciado (`ph2d_shape_gpu::ShapeInstance`, `16` palavras), num buffer
//! PRÓPRIO que o passe lê sem nunca o trazer de volta à CPU.
//!
//! ⚠️ **As palavras são as do `ShapeInstance`, e são pregadas por um gate** contra o
//! `#[repr(C)]` daquela crate (`tests/it/as_formas_no_dispositivo.rs`): `pos` 0-1 · `size` 2-3 ·
//! `basis` 4-7 · `anchor` 8-9 · `geometry` 10 · almofada 11 · `tint` 12-15.
//!
//! ⚠️⚠️ **As contas são as da CPU, uma a uma** (`ph2d_eval_motion::lower_vector_onto`, o braço
//! `RowMedium::Shape`):
//! - o `size` AUSENTE é `[1, 1]` e **não** o `default_size` do chamador — a geometria da forma já
//!   traz o tamanho autorado, e a sprite é que precisa do tamanho de omissão;
//! - `basis = [cos, sin, −sin, cos]` do `rot` em GRAUS;
//! - `anchor = pivô × size` (o `SinkStyle::anchor_for`), com o pivô ZERO cravado — um `size` não
//!   finito propagaria o `NaN` para a âncora onde a CPU escreve `0` (a lição do `lower`);
//! - a linha é forma quando `geometry_id > 0,5` — o `MediaColumns::at`, à letra. Uma linha que não
//!   é forma escreve a geometria `0xffffffff`, que o passe não acha e desenha como NADA: ela é
//!   desenhada pelo baixamento das sprites (e esse, em troca, cala as linhas de forma).

use crate::codegen::WORKGROUP_SIZE;

/// `ShapeInstance` são `64` bytes = `16` palavras de 32 bits.
pub const FORMA_WORDS: u32 = 16;

/// A geometria que o passe NÃO acha — uma linha que não é forma.
pub const SEM_GEOMETRIA: u32 = 0xffff_ffff;

/// As colunas que este baixamento lê, pela ordem das ligações. A presença de cada uma é o bit `i`
/// da assinatura; ausente ⇒ o MESMO valor de omissão que a CPU aplica.
pub const FORMA_COLUMNS: [&str; 5] = ["P", "size", "rot", "tint", "geometry_id"];

/// O tipo WGSL de cada coluna de [`FORMA_COLUMNS`].
const TIPOS: [&str; 5] = ["vec2<f32>", "vec2<f32>", "f32", "vec4<f32>", "f32"];

/// O valor de omissão de cada coluna — os `*_at(.., default)` do braço de forma da CPU.
const OMISSOES: [&str; 5] = [
    "vec2<f32>(0.0, 0.0)",           // P
    "vec2<f32>(1.0, 1.0)",           // size — ⚠️ `[1, 1]`, NÃO o `default_size`
    "0.0",                           // rot
    "vec4<f32>(1.0, 1.0, 1.0, 1.0)", // tint
    "0.0",                           // geometry_id — sem coluna, nenhuma linha é forma
];

/// O módulo WGSL para um conjunto concreto de colunas e um pivô. Ligação 0 = `{count, primeiro}`,
/// ligação 1 = as cópias; depois uma ligação `read` por coluna presente.
#[must_use]
pub fn forma_module(present: [bool; 5], pivot: [f32; 2]) -> String {
    let ancora = if pivot == [0.0, 0.0] {
        "\x20   instances[base + 8u] = 0u;\n\x20   instances[base + 9u] = 0u;\n".to_string()
    } else {
        format!(
            "\x20   wf(base + 8u, s.x * {px:?});\n\x20   wf(base + 9u, s.y * {py:?});\n",
            px = pivot[0],
            py = pivot[1],
        )
    };
    let mut src = String::with_capacity(2048);
    src.push_str(
        "struct FormaParams {\n\
         \x20   count: u32,\n\
         \x20   primeiro: u32,\n\
         }\n\
         @group(0) @binding(0) var<uniform> params: FormaParams;\n\
         @group(0) @binding(1) var<storage, read_write> instances: array<u32>;\n",
    );
    let mut slot = 2u32;
    for (i, col) in FORMA_COLUMNS.iter().enumerate() {
        if present[i] {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read> in_{col}: array<{}>;\n",
                TIPOS[i]
            ));
            slot += 1;
        }
    }
    src.push('\n');
    for (i, col) in FORMA_COLUMNS.iter().enumerate() {
        if present[i] {
            src.push_str(&format!(
                "fn read_{col}(i: u32) -> {ty} {{ return in_{col}[i]; }}\n",
                ty = TIPOS[i]
            ));
        } else {
            src.push_str(&format!(
                "fn read_{col}(i: u32) -> {ty} {{ _ = i; return {d}; }}\n",
                ty = TIPOS[i],
                d = OMISSOES[i]
            ));
        }
    }
    src.push_str(&format!(
        "\n\
        fn wf(w: u32, v: f32) {{ instances[w] = bitcast<u32>(v); }}\n\
        \n\
        @compute @workgroup_size({WORKGROUP_SIZE})\n\
        fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
        \x20   let i = gid.x;\n\
        \x20   if (i >= params.count) {{ return; }}\n\
        \x20   // A LEITURA e' `i` (dentro desta corrente); a ESCRITA e' `primeiro + i`.\n\
        \x20   let base = (params.primeiro + i) * {FORMA_WORDS}u;\n\
        \x20   let p = read_P(i);\n\
        \x20   wf(base + 0u, p.x);\n\
        \x20   wf(base + 1u, p.y);\n\
        \x20   let s = read_size(i);\n\
        \x20   wf(base + 2u, s.x);\n\
        \x20   wf(base + 3u, s.y);\n\
        \x20   let rad = read_rot(i) * 0.017453292519943295;\n\
        \x20   let sn = sin(rad);\n\
        \x20   let cs = cos(rad);\n\
        \x20   wf(base + 4u, cs);\n\
        \x20   wf(base + 5u, sn);\n\
        \x20   wf(base + 6u, -sn);\n\
        \x20   wf(base + 7u, cs);\n\
        {ancora}\
        \x20   // geometry (10): o handle quando a linha e' forma (`> 0.5`, o `MediaColumns::at`),\n\
        \x20   // senao a geometria que o passe nao acha. `u32(f32)` trunca como o `as u32`.\n\
        \x20   let g = read_geometry_id(i);\n\
        \x20   var h = {SEM_GEOMETRIA}u;\n\
        \x20   if (g > 0.5) {{\n\
        \x20       h = u32(g);\n\
        \x20   }}\n\
        \x20   instances[base + 10u] = h;\n\
        \x20   instances[base + 11u] = 0u;\n\
        \x20   let t = read_tint(i);\n\
        \x20   wf(base + 12u, t.x);\n\
        \x20   wf(base + 13u, t.y);\n\
        \x20   wf(base + 14u, t.z);\n\
        \x20   wf(base + 15u, t.w);\n\
        }}\n"
    ));
    src
}

/// A assinatura da cache de pipelines: um bit por coluna presente e os BITS do pivô (dois pivôs
/// que diferem no último dígito dão fontes diferentes). Pivô zero ⇒ só os bits das colunas.
#[must_use]
pub fn forma_signature(present: [bool; 5], pivot: [f32; 2]) -> u64 {
    let cols = present
        .iter()
        .enumerate()
        .fold(0u64, |sig, (i, &p)| sig | (u64::from(p) << i));
    if pivot == [0.0, 0.0] {
        return cols;
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for w in [pivot[0].to_bits(), pivot[1].to_bits()] {
        h ^= u64::from(w);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    cols | (h << FORMA_COLUMNS.len())
}

#[cfg(test)]
#[path = "lower_forma_tests.rs"]
mod tests;
