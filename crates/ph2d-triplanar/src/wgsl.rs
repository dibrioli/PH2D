//! ⭐⭐ **O GÉMEO EM WGSL** — o [`crate::avalia`] conta a conta, na mesma ordem.
//!
//! Quem chama escreve as duas portas de leitura (as texturas são dele; o amostrador repete e é
//! trilinear):
//!
//! ```wgsl
//! fn tri_cor_ler(camada: i32, uv: vec2<f32>, lod: f32) -> vec4<f32>   // cor JÁ em linear
//! fn tri_nrh_ler(camada: i32, uv: vec2<f32>, lod: f32) -> vec4<f32>
//! ```
//!
//! e recebe `tri_avalia(par, p, n, dx, dy) -> TriResultado`. ⚠️ As derivadas (`dx`, `dy`) têm de
//! ser tiradas por quem chama em fluxo UNIFORME (antes de qualquer ramo).

/// A fonte.
#[must_use]
pub fn fonte() -> &'static str {
    CORPO
}

const CORPO: &str = r"
struct TriParams {
    // (1/tamanho, 1/(tamanho*aspecto))
    escalas: vec2<f32>,
    blend: f32,
    relevo: f32,
    lado: f32,
    camada: i32,
    tem_normal: bool,
    tem_rugosidade: bool,
};

struct TriResultado { cor: vec3<f32>, rugosidade: f32, normal: vec3<f32> };

struct TriVista { t: vec3<f32>, b: vec3<f32>, a: vec3<f32>, k: f32 };

fn tri_par(a: f32, b: f32, blend: f32) -> vec2<f32> {
    let q = clamp((a / (a + b) - 0.5 * (1.0 - blend)) / blend, 0.0, 1.0);
    return vec2<f32>(q, 1.0 - q);
}

// O Blend do Blender, nos eixos DELE: (x, y, z)_b = (x, -z, y)_nosso.
fn tri_pesos(n: vec3<f32>, blend: f32) -> vec3<f32> {
    let s = abs(n.x) + abs(n.z) + abs(n.y);
    if (s <= 0.0) { return vec3<f32>(1.0, 0.0, 0.0); }
    let x = abs(n.x) / s;
    let y = abs(n.z) / s;
    let z = abs(n.y) / s;
    let l = 0.5 * (1.0 + blend);
    var w = vec3<f32>(0.0);
    if (x > l * (x + y) && x > l * (x + z)) {
        w.x = 1.0;
    } else if (y > l * (x + y) && y > l * (y + z)) {
        w.y = 1.0;
    } else if (z > l * (x + z) && z > l * (y + z)) {
        w.z = 1.0;
    } else if (blend > 0.0) {
        if (z < (1.0 - l) * (y + x)) {
            let p = tri_par(x, y, blend);
            w.x = p.x;
            w.y = p.y;
        } else if (x < (1.0 - l) * (y + z)) {
            let p = tri_par(y, z, blend);
            w.y = p.x;
            w.z = p.y;
        } else if (y < (1.0 - l) * (x + z)) {
            let p = tri_par(x, z, blend);
            w.x = p.x;
            w.z = p.y;
        } else {
            let k = 2.0 * l - 1.0;
            w = ((2.0 - l) * vec3<f32>(x, y, z) + (l - 1.0)) / k;
        }
    } else {
        w.x = 1.0;
    }
    return vec3<f32>(w.x, w.z, w.y);
}

fn tri_vista(eixo: u32, n_eixo: f32) -> TriVista {
    let p = n_eixo >= 0.0;
    let s = select(-1.0, 1.0, p);
    if (eixo == 0u) {
        return TriVista(vec3<f32>(0.0, 0.0, -s), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(s, 0.0, 0.0), select(1.0, 0.0, p));
    }
    if (eixo == 1u) {
        return TriVista(vec3<f32>(0.0, 0.0, s), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, s, 0.0), select(0.0, 1.0, p));
    }
    return TriVista(vec3<f32>(s, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, s), select(1.0, 0.0, p));
}

fn tri_nivel(w: TriVista, dx: vec3<f32>, dy: vec3<f32>, eu: f32, ev: f32, lado: f32) -> f32 {
    let ax = dot(dx, w.t) * (eu * lado);
    let bx = dot(dx, w.b) * (ev * lado);
    let ay = dot(dy, w.t) * (eu * lado);
    let by = dot(dy, w.b) * (ev * lado);
    return 0.5 * log2(max(max(ax * ax + bx * bx, ay * ay + by * by), 1.0e-12));
}

fn tri_normal_do_mapa(h: vec4<f32>, k: f32) -> vec3<f32> {
    let x = (h.x * 2.0 - 1.0) * k;
    let y = (h.y * 2.0 - 1.0) * k;
    return vec3<f32>(x, y, sqrt(1.0 - min(x * x + y * y, 1.0)));
}

fn tri_avalia(par: TriParams, p: vec3<f32>, n: vec3<f32>, dx: vec3<f32>, dy: vec3<f32>) -> TriResultado {
    let w = tri_pesos(n, par.blend);
    let eu = par.escalas.x;
    let ev = par.escalas.y;
    var cor = vec3<f32>(0.0);
    var rug = 0.0;
    var soma_n = vec3<f32>(0.0);
    let relevo = select(0.0, par.relevo, par.tem_normal);
    let usa_nrh = (par.tem_normal && par.relevo != 0.0) || par.tem_rugosidade;
    for (var e = 0u; e < 3u; e = e + 1u) {
        let we = w[e];
        if (we <= 0.0) { continue; }
        let vi = tri_vista(e, n[e]);
        let uv = vec2<f32>(dot(p, vi.t) * eu + vi.k, dot(p, vi.b) * ev);
        let lod = tri_nivel(vi, dx, dy, eu, ev, par.lado);
        cor = cor + tri_cor_ler(par.camada, uv, lod).rgb * we;
        if (!usa_nrh) { continue; }
        let h = tri_nrh_ler(par.camada, uv, lod);
        rug = rug + h.z * we;
        let tn = tri_normal_do_mapa(h, relevo);
        let x = tn.x + dot(n, vi.t);
        let y = tn.y + dot(n, vi.b);
        let z = tn.z * dot(n, vi.a);
        soma_n = soma_n + (vi.t * x + vi.b * y + vi.a * z) * we;
    }
    var normal = n;
    if (par.tem_normal && par.relevo != 0.0) {
        let l = sqrt(dot(soma_n, soma_n));
        if (l > 0.0) { normal = soma_n / l; }
    }
    return TriResultado(cor, rug, normal);
}
";
