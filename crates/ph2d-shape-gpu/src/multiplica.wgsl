
// B5 — o MULTIPLY exacto por cópia (`mistura.rs`): `out = s·(1−αb) + b·(1−αs+s)`, o W3C que o Vello
// compõe. A 1.ª saída é a cópia (factor `1−αb`), a 2.ª é o factor do que está por baixo.
struct DuasSaidas {
    @location(0) @blend_src(0) cor: vec4<f32>,
    @location(0) @blend_src(1) factor: vec4<f32>,
}

@fragment
fn fs_multiplica(in: VsOut) -> DuasSaidas {
    let c = cor_de_mistura(in);
    return DuasSaidas(c, vec4<f32>(vec3<f32>(1.0 - c.a) + c.rgb, 1.0));
}
