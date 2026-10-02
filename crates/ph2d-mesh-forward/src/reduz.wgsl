// A REDUCAO de um nivel da cobertura para o seguinte: a media de 2x2. A cobertura (r) e a
// profundidade ponderada (g = z * cobertura) somam-se linearmente, logo a media do nivel grosso e'
// a cobertura media e a profundidade media DOS BLOQUEADORES.
@group(0) @binding(0) var fonte: texture_2d<f32>;

@vertex
fn vs_reduz(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[k], 0.0, 1.0);
}

@fragment
fn fs_reduz(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let b = vec2<i32>(q.xy) * 2;
    let lim = vec2<i32>(textureDimensions(fonte)) - vec2<i32>(1);
    var s = vec4<f32>(0.0);
    for (var j = 0; j < 2; j = j + 1) {
        for (var i = 0; i < 2; i = i + 1) {
            s = s + textureLoad(fonte, min(b + vec2<i32>(i, j), lim), 0);
        }
    }
    return s * 0.25;
}
