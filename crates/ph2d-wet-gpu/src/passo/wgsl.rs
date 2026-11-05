//! Os shaders pequenos do passo: o entrelaçar do `st` e a porta do `has_fluid`.

/// O `st` entrelaçado (film, susp) do advect ⇄ os `film`/`susp` da secagem, NA placa (defeito 1).
pub(super) const ENTRELACA: &str = r"
@group(0) @binding(0) var<storage, read_write> st: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> film: array<f32>;
@group(0) @binding(2) var<storage, read_write> susp: array<f32>;
fn idx(g: vec3<u32>, n: vec3<u32>) -> u32 { return g.x + g.y * n.x * 256u; }
@compute @workgroup_size(256)
fn separa(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    let i = idx(g, n);
    if i >= arrayLength(&film) { return; }
    film[i] = st[i].x;
    susp[i] = st[i].y;
}
@compute @workgroup_size(256)
fn junta(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    let i = idx(g, n);
    if i >= arrayLength(&film) { return; }
    st[i] = vec2(film[i], susp[i]);
}
";

/// A porta do `has_fluid` NA placa (item 3 de LIGACAO_PLACA.md): depois do rebuild, os grupos de cada passada
/// do resto do passo = os da CPU se `caixa[14]` (has_fluid), senão 0 — o passo inteiro num submit e numa espera.
pub(super) const PORTA: &str = r"
@group(0) @binding(0) var<storage, read> caixa: array<i32>;
@group(0) @binding(1) var<storage, read> base: array<u32>;
@group(0) @binding(2) var<storage, read_write> args: array<u32>;
@compute @workgroup_size(96)
fn porta(@builtin(global_invocation_id) g: vec3<u32>) {
    if g.x >= arrayLength(&base) { return; }
    args[g.x] = select(0u, base[g.x], caixa[14] != 0);
}
";
