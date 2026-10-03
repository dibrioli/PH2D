// ── A CADEIA DO BRILHO em passes de DESENHO — a lei do `ph2d_bloom::wgsl` (a mesma do Motion e do
// Render tracado), com a porta de leitura por `textureLoad`: o WebGL2 nao tem compute nem
// armazenamento. Cada passe desenha um triangulo que cobre o nivel de destino.
struct Passe {
    // x, y = o lado da origem · z, w = o lado do destino
    dims: vec4<u32>,
    base: vec4<f32>,
    // x = limiar · y = joelho · z = tecto · w = 1 so' no degrau que le a cena (o CORTE entra ali)
    corte: vec4<f32>,
};

@group(0) @binding(0) var origem: texture_2d<f32>;
// O nivel a SOMAR na subida (nos degraus que descem e' a propria origem, e ninguem o le).
@group(0) @binding(1) var nivel: texture_2d<f32>;
@group(0) @binding(2) var<uniform> P: Passe;

fn bl_le(i: u32) -> vec3<f32> {
    let w = P.dims.x;
    let c = textureLoad(origem, vec2<i32>(i32(i % w), i32(i / w)), 0).xyz;
    if (P.corte.w != 0.0) { return bl_corte(c, P.corte.x, P.corte.y, P.corte.z); }
    return c;
}

@vertex
fn vs_cheio(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[k], 0.0, 1.0);
}

@fragment
fn fs_desce(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(bl_desce13(P.dims.x, P.dims.y, P.dims.z, P.dims.w, u32(q.x), u32(q.y)), 0.0);
}

@fragment
fn fs_sobe(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let t = bl_sobe_tenda(P.dims.x, P.dims.y, P.dims.z, P.dims.w, u32(q.x), u32(q.y), P.base);
    return vec4<f32>(t + textureLoad(nivel, vec2<i32>(q.xy), 0).xyz, 0.0);
}
