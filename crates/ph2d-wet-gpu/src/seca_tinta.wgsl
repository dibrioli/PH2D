// A secagem da lei `Pigmento` (E11, F4_2 §6): `dry_cell` com a porta `drying/tinta.rs`, sem a inversa de Newton. Concatenado a
// `seca.wgsl`; as constantes vêm do cabeçalho de `tinta.rs`. Assentar move a fracção da soma que a massa move; re-molhar
// devolve-a pela abertura; o espelho sRGB segue a fatia que recebeu.

@group(0) @binding(11) var<storage, read_write> filme: array<f32>;
@group(0) @binding(12) var<storage, read_write> aberto: array<f32>;
@group(0) @binding(13) var<storage, read_write> abertura: array<f32>;
@group(0) @binding(14) var<storage, read> meio: array<f32>;

fn linear_para_srgb(c0: f32) -> f32 {
    let c = clamp(c0, 0.0, 1.0);
    if c <= 0.0031308 { return 12.92 * c; }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

// `srgb_0_255` da fatia (`Soma::cor` + `oklab_para_srgb`); chamada só com `v > 0`.
fn espelho(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    let lab = vec3<f32>(a.y, a.z, b.x) / a.x;
    let r = vec3<f32>(
        dot(vec3<f32>(1.0, 0.3963377774, 0.2158037573), lab),
        dot(vec3<f32>(1.0, -0.1055613458, -0.0638541728), lab),
        dot(vec3<f32>(1.0, -0.0894841775, -1.2914855480), lab),
    );
    let lms = r * r * r;
    let lin = vec3<f32>(
        dot(vec3<f32>(4.0767416621, -3.3077115913, 0.2309699292), lms),
        dot(vec3<f32>(-1.2684380046, 2.6097574011, -0.3413193965), lms),
        dot(vec3<f32>(-0.0041960863, -0.7034186147, 1.7076147010), lms),
    );
    return clamp(
        vec3<f32>(linear_para_srgb(lin.x), linear_para_srgb(lin.y), linear_para_srgb(lin.z)) * 255.0,
        vec3<f32>(0.0),
        vec3<f32>(255.0),
    );
}

@compute @workgroup_size(16, 16)
fn seca_tinta(@builtin(global_invocation_id) gid: vec3<u32>) {
    let ii = dentro(gid);
    if ii < 0 { return; }
    let i = u32(ii);
    let film0 = film[i];
    let s_mass = susp[i];
    if film0 <= 0.0 && s_mass <= 0.0 { return; }
    let md = modo_da(i);
    if md == 0u { return; }
    let evb = select(u.evap_calmo, u.evap_base, md == 2u);
    let rwb = select(u.rewet_calmo, u.rewet_base, md == 2u);
    var susp_c = s_mass;
    var sett_c = sett[i];
    var susp_rgb = ldx(i);
    var sett_rgb = ldt(i);
    let j = 6u * i;
    var fa = vec3<f32>(filme[j], filme[j + 1u], filme[j + 2u]);
    var fb = vec3<f32>(filme[j + 3u], filme[j + 4u], filme[j + 5u]);
    var aa = vec3<f32>(aberto[j], aberto[j + 1u], aberto[j + 2u]);
    var ab = vec3<f32>(aberto[j + 3u], aberto[j + 4u], aberto[j + 5u]);
    var o = abertura[i];

    var ev = 1.0;
    if film0 > 0.0 && sett_c < 1000.0 {
        let cnt = edge[i];
        if cnt == 9u { ev = 1.0; } else { ev = f32(cnt) / 9.0; }
    }
    // `meio::retencao` e `meio::evaporacao`: `m = 0` devolve a água, sem aritmética.
    let m = meio[i];
    var reten = u.retention;
    var evapora = 1.0;
    if m > 0.0 {
        let mm = min(m, 1.0);
        reten = pow(u.retention, sqrt(pow(ETA_OLEO, -mm)));
        evapora = (1.0 - mm) + mm * ENDURECE;
    }
    let f = film0;
    var borda = u.edge_dark;
    if aux[i].w >= 0.0 { borda = aux[i].w; }
    var new_film = f * reten - evb * ((1.0 - ev) * borda + u.base_evap) * evapora;
    if new_film < 0.0001 { new_film = 0.0; }
    var lost = 1.0;
    if f > 0.0 { lost = 1.0 - clamp(new_film / f, 0.0, 1.0); }
    let film_c = new_film;

    if lost > 0.0 && s_mass > 0.0 {
        var dm = s_mass * lost;
        if u.bypass == 0u {
            let bias = clamp(1.0 + u.ext_gran * 0.6 * (0.5 - paper[i]) * 2.0, 0.3, 1.7);
            dm = min(dm * bias, s_mass);
        }
        // `tinta::assenta`: a fracção do filme passa ao aberto, o que chega reabre-o pelo volume.
        let t = clamp(dm / s_mass, 0.0, 1.0);
        let antes = aa.x;
        let na = fa - fa * t;
        let nb = fb - fb * t;
        let chega = fa.x - na.x;
        aa = aa + (fa - na);
        ab = ab + (fb - nb);
        fa = na;
        fb = nb;
        let v = antes + chega;
        if v > 0.0 { o += (1.0 - o) * (chega / v); }
        if aa.x > 0.0 { sett_rgb = espelho(aa, ab); }
        sett_c = sett_c + dm;
        susp_c = max(s_mass - dm, 0.0);
    }

    let st_m = sett_c;
    if film_c > 0.0 && st_m > 0.0 {
        let excess = max(film_c - alpha_of_mass(susp_c), 0.0);
        var b = rwb * (1.0 + excess * 50.0);
        if u.bypass == 0u {
            let sm = select((1.0 - u.ext_stain) * 2.0, 1.0 + (0.5 - u.ext_stain) * 14.0, u.ext_stain < 0.5);
            b = b * sm;
        }
        b = clamp(b, 0.0, 1.0);
        if b > 0.0 {
            // `tinta::levanta`: a fracção `b·o` do aberto volta ao filme.
            let t = clamp(b * o, 0.0, 1.0);
            let lift = st_m * t;
            susp_c = susp_c + lift;
            sett_c = st_m - lift;
            let na = aa - aa * t;
            let nb = ab - ab * t;
            fa = fa + (aa - na);
            fb = fb + (ab - nb);
            aa = na;
            ab = nb;
            if fa.x > 0.0 { susp_rgb = espelho(fa, fb); }
        }
    }

    film[i] = film_c;
    susp[i] = susp_c;
    sett[i] = sett_c;
    srgb[3u * i] = susp_rgb.x; srgb[3u * i + 1u] = susp_rgb.y; srgb[3u * i + 2u] = susp_rgb.z;
    trgb[3u * i] = sett_rgb.x; trgb[3u * i + 1u] = sett_rgb.y; trgb[3u * i + 2u] = sett_rgb.z;
    filme[j] = fa.x; filme[j + 1u] = fa.y; filme[j + 2u] = fa.z;
    filme[j + 3u] = fb.x; filme[j + 4u] = fb.y; filme[j + 5u] = fb.z;
    aberto[j] = aa.x; aberto[j + 1u] = aa.y; aberto[j + 2u] = aa.z;
    aberto[j + 3u] = ab.x; aberto[j + 4u] = ab.y; aberto[j + 5u] = ab.z;
    abertura[i] = o;
}
