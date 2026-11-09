// ⭐ P14 «L» (P14c-placa 5) — o primeiro momento `M` na HOLOGRÁFICA (`docs/PixelArt/P14_plano.md` §4 e §11).
// Duas entradas PRÓPRIAS (as de hoje não ganham ligação: o defeito da `980da1d295`), só com receptor de normal:
//
// · `hrc_soma_momento`, no laço dos quadrantes, logo depois da `hrc_soma` (o quadrante seguinte reescreve os R):
//   a MESMA parcela `R0` do texel (o «+1», o τ do translúcido), × `ω̂_q · sin h / h` com `h = π/4` — o cone 0 do
//   nível 0 cobre ±45° em volta da direção do quadrante (`∫ω̂ dθ` sobre ele é `2 sin h · ω̂_q`, e `R0 = L · 2h`).
// · `hrc_saida_momento`: o borrão em cruz da `hrc_saida` (o mesmo peso, o mesmo meio) para `Mx` e `My`, escritos
//   na c0 de `M` que a fina de `M` (`c3_momento`) lê.
//
// Concatenado depois de `c2_hrc.wgsl`. Grupo 1 (as pipelines da holográfica não o usam). A saída não liga o grupo 3
// da holográfica (as texturas de escrita dele somadas às duas daqui passam o limite de 4 por estágio): a grade vem
// no uniforme 46.

@group(1) @binding(40) var<storage, read_write> acumulado_momento: array<vec4<f32>>;
@group(1) @binding(44) var c0_momento_x_hrc: texture_storage_2d<rgba16float, write>;
@group(1) @binding(45) var c0_momento_y_hrc: texture_storage_2d<rgba16float, write>;
@group(1) @binding(46) var<uniform> grade_do_momento: vec4<u32>;

// sin(π/4) / (π/4).
const SINC_DO_QUADRANTE: f32 = 0.9003163161571061;

// A direção do quadrante na grade (0 = +x, 1 = +y, 2 = −x, 3 = −y: o `na_grade`).
fn direcao_do_quadrante(q: u32) -> vec2<f32> {
    switch q {
        case 1u: { return vec2<f32>(0.0, 1.0); }
        case 2u: { return vec2<f32>(-1.0, 0.0); }
        case 3u: { return vec2<f32>(0.0, -1.0); }
        default: { return vec2<f32>(1.0, 0.0); }
    }
}

@compute @workgroup_size(64)
fn hrc_soma_momento(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(num_workgroups) nw: vec3<u32>,
) {
    let p = parcela_da_soma(gid, nw);
    if !p.ok {
        return;
    }
    let n = hrcs[iq()].grade.x * hrcs[iq()].grade.y;
    let w = direcao_do_quadrante(hrcs[iq()].quadrante) * SINC_DO_QUADRANTE;
    let mx = vec4<f32>(p.f * w.x, 0.0);
    let my = vec4<f32>(p.f * w.y, 0.0);
    if hrcs[iq()].quadrante == 0u {
        acumulado_momento[p.i] = mx;
        acumulado_momento[n + p.i] = my;
    } else {
        acumulado_momento[p.i] += mx;
        acumulado_momento[n + p.i] += my;
    }
}

@compute @workgroup_size(8, 8)
fn hrc_saida_momento(@builtin(global_invocation_id) gid: vec3<u32>) {
    let g = grade_do_momento.xy;
    if any(gid.xy >= g) {
        return;
    }
    let n = g.x * g.y;
    let tau_p = meio(vec2<f32>(gid.xy) + 0.5).tau;
    let i = gid.y * g.x + gid.x;
    var mx = acumulado_momento[i].rgb;
    var my = acumulado_momento[n + i].rgb;
    if BORRAO_HRC {
        var sx = 4.0 * mx;
        var sy = 4.0 * my;
        var peso = 4.0;
        let d = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
        for (var j = 0u; j < 4u; j++) {
            let q = vec2<i32>(gid.xy) + d[j];
            if any(q < vec2<i32>(0)) || any(q >= vec2<i32>(g)) {
                continue;
            }
            if meio(vec2<f32>(q) + 0.5).tau != tau_p {
                continue;
            }
            let k = u32(q.y) * g.x + u32(q.x);
            sx += acumulado_momento[k].rgb;
            sy += acumulado_momento[n + k].rgb;
            peso += 1.0;
        }
        mx = sx / peso;
        my = sy / peso;
    }
    textureStore(c0_momento_x_hrc, gid.xy, vec4<f32>(mx, 1.0));
    textureStore(c0_momento_y_hrc, gid.xy, vec4<f32>(my, 1.0));
}
