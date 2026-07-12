// Flip — preenchimento (fill) de traços fechados (ADR-0114, W1 T1.6). Triângulos
// já triangulados na CPU (ear-clipping): o vertex só transforma mundo→clip e
// carrega a profundidade da ordem 2D; o fragment devolve a cor chapada premult.
// Compartilha a MESMA câmera (group 0, binding 0) e o mesmo depth-buffer do
// passe de traço; a profundidade menor (2·sid+1) deixa o traço por cima.

struct Camera {
    world_to_clip: mat4x4<f32>,
    viewport: vec2<f32>,
    px_per_world: f32,
    _pad: f32,
    // Ghost Frames (W3) — o mesmo uniform do passe de traço: `a > 0` = fantasma,
    // e o fill entra na SILHUETA junto (senão o ghost sairia com o miolo colorido
    // e só o contorno tingido).
    ghost_tint: vec4<f32>,
}

@group(0) @binding(0) var<uniform> cam: Camera;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) depth: f32,
    @location(2) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    let c = cam.world_to_clip * vec4<f32>(pos, 0.0, 1.0);
    out.clip = vec4<f32>(c.xy / c.w, depth, 1.0);
    out.color = color; // já premultiplicado (opacidade do fill no alpha)
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // A cor chega PREMULTIPLICADA; o fantasma recolore a silhueta e reaplica o fade
    // (com o alpha premultiplicado, o tint entra como `tint.rgb · alpha`).
    var c = in.color;
    if (cam.ghost_tint.a > 0.0) {
        let a = c.a * cam.ghost_tint.a;
        c = vec4<f32>(cam.ghost_tint.rgb * a, a);
    }
    // GP `gpencil_frag.glsl`: fragmento ~transparente descarta (a < 0.001) — um
    // fill invisível não pode escrever depth e ocultar o que está abaixo.
    if (c.a < 0.001) {
        discard;
    }
    return c; // premult over (blend One, OneMinusSrcAlpha)
}
