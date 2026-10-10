// A LEI DO DONO da C0 — UMA função, que a captura das formas (`c0_captura.wgsl`) e a dos sprites por
// texel (`c0_sprite.wgsl`, W3) concatenam: a cobertura soma; quem POSSUI o texel (cobertura própria
// ≥ ½) escreve a emissão (max) e a transmissão de dentro (×). Um sprite é só outra fonte de cobertura.
// ⭐ W5c: e o ALBEDO do rebote (max) — só o dono OPACO (a lei do plano §2.4: o translúcido não reflete).

struct Alvos {
    @location(0) emissao: vec4<f32>,
    @location(1) cobertura: vec4<f32>,
    @location(2) tau_dentro: vec4<f32>,
    @location(3) albedo: vec4<f32>,
}

fn c0_alvos(c: f32, cor: vec4<f32>, opacidade: f32, albedo: vec3<f32>) -> Alvos {
    let dono = c >= 0.5;
    var o: Alvos;
    o.emissao = select(vec4<f32>(0.0), cor, dono);
    o.cobertura = vec4<f32>(c);
    o.tau_dentro = vec4<f32>(select(1.0, 1.0 - opacidade, dono));
    o.albedo = select(vec4<f32>(0.0), vec4<f32>(albedo, 1.0), dono && opacidade >= 1.0);
    return o;
}
