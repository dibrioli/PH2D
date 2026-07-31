// Hospedado por `ph2d-mesh-render` (crates/ph2d-mesh-render/src/shaders/mesh.wgsl).
//
// O passe de matcap da W1/M2: rasteriza a malha e a sombreia por um matcap
// PROCEDURAL — sombreamento que é função apenas da normal em espaço de VISTA.
//
// Por que procedural e não uma textura de matcap: um matcap de arquivo é um
// asset, e um asset é um pipeline (importar, empacotar, versionar, escolher).
// A W1 existe para haver forma na tela; a função aqui é o mesmo modelo (o
// sombreamento depende só de `n_view`) sem nada disso. O matcap por textura
// entra com o painel que o escolhe — que é quando ele deixa de ser um arquivo
// que ninguém pode trocar.
//
// A saída é LINEAR e pode passar de 1.0: o alvo é o `game_rt` (Rgba16Float) e o
// tonemap do shell vem depois. Escrever já-tonemapeado aqui apagaria o realce.

struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> cam: Camera;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) n_view: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VsOut {
    var out: VsOut;
    out.clip = cam.view_proj * vec4<f32>(pos, 1.0);
    // `w = 0` ⇒ direção, não ponto: a translação da vista não entra. A matriz de
    // vista é ortonormal (sai de uma `look_at`), então não há inverso-transposto
    // a fazer — a normal viaja por ela como um vetor comum.
    out.n_view = (cam.view * vec4<f32>(normal, 0.0)).xyz;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // A interpolação entre vértices encurta a normal; sem renormalizar, a
    // superfície escurece no meio de cada triângulo (o facetamento clássico).
    var n = normalize(in.n_view);

    // Em espaço de vista o olho olha por `-Z`, então `n.z > 0` está de frente.
    // Uma face vista por trás (malha aberta, casca fina) tem de acender como
    // frente — senão o interior de uma peça aberta vira um buraco preto e o
    // artista lê isso como geometria faltando.
    if (n.z < 0.0) {
        n = -n;
    }

    let key_dir = normalize(vec3<f32>(-0.45, 0.65, 0.62));
    let fill_dir = normalize(vec3<f32>(0.55, -0.35, 0.45));

    let key = max(dot(n, key_dir), 0.0);
    let fill = max(dot(n, fill_dir), 0.0);
    // Fresnel barato: quanto mais a normal foge do olho, mais borda.
    let rim = pow(1.0 - clamp(n.z, 0.0, 1.0), 3.0);

    // Argila clara e dessaturada — o barro de estúdio. A cor existe para a FORMA
    // aparecer; um material de verdade é a wave do shader (docs/3D/05.1).
    let clay = vec3<f32>(0.74, 0.70, 0.66);
    let cool = vec3<f32>(0.42, 0.52, 0.68);

    var c = clay * (0.16 + 0.82 * key) + cool * (0.22 * fill);
    c = c + cool * (rim * 0.30);

    // Um realce estreito, para a curvatura ser legível onde a difusa satura.
    let h = normalize(key_dir + vec3<f32>(0.0, 0.0, 1.0));
    c = c + vec3<f32>(pow(max(dot(n, h), 0.0), 48.0) * 0.30);

    return vec4<f32>(c, 1.0);
}
