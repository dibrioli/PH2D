// ⭐ W5z — os T e os R da holográfica em BUFFERS (`Config::hrc_em_texturas = false`): o índice é o do buffer.
@group(3) @binding(5) var<storage, read> t_a: array<vec2<u32>>;
@group(3) @binding(6) var<storage, read> t_b: array<vec2<u32>>;
@group(3) @binding(7) var<storage, read_write> t_saida: array<vec2<u32>>;
@group(3) @binding(8) var<storage, read> r_cima: array<vec2<u32>>;
@group(3) @binding(9) var<storage, read_write> r_saida: array<vec2<u32>>;

fn ler_t_a(i: u32) -> vec2<u32> { return t_a[i]; }
fn ler_t_b(i: u32) -> vec2<u32> { return t_b[i]; }
fn ler_r_cima(i: u32) -> vec2<u32> { return r_cima[i]; }
fn escrever_t(i: u32, e: vec2<u32>) { t_saida[i] = e; }
fn escrever_r(i: u32, e: vec2<u32>) { r_saida[i] = e; }
