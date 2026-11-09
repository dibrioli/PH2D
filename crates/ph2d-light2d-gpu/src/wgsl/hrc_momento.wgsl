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
// ⭐ P14c-placa 6 — `false` = o cone 0 inteiro com `ω̂_q` (a placa 5; o controlo da mutação).
override METADES_DO_MOMENTO: bool = true;
@group(1) @binding(44) var c0_momento_x_hrc: texture_storage_2d<rgba16float, write>;
@group(1) @binding(45) var c0_momento_y_hrc: texture_storage_2d<rgba16float, write>;
@group(1) @binding(46) var<uniform> grade_do_momento: vec4<u32>;

// sin(π/4) / (π/4).
const SINC_DO_QUADRANTE: f32 = 0.9003163161571061;
// ⭐ P14c-placa 6 — sin(π/8)·sinc(π/8) = sin²(π/8)/(π/8): o peso da DIFERENÇA das metades do cone 0 do nível 0.
// As metades cobrem [θ_q − 45°, θ_q] e [θ_q, θ_q + 45°]; `∫ω̂` de cada uma é `R·ω̂(θ_q ∓ 22,5°)·sinc(π/8)`, e a
// soma das duas dá `R0·ω̂_q·cos(π/8)·sinc(π/8)` (= o SINC_DO_QUADRANTE de antes) `+ (R₊ − R₋)·ω̂_q⊥·sin(π/8)·sinc(π/8)`.
const PESO_DA_DIFERENCA: f32 = 0.37292323725032257;

// ⭐ P14c-placa 6 — onde mora a diferença `R₊ − R₋` da sonda `(u, v)` no acumulado (depois de `Mx`, `My`): um
// bloco por quadrante com os quadrantes juntos, um só em série (a soma do quadrante lê-a antes do seguinte).
fn indice_da_diferenca(u: u32, v: u32) -> u32 {
    let n = hrcs[iq()].grade.x * hrcs[iq()].grade.y;
    let bloco = select(0u, hrcs[iq()].quadrante, JUNTOS_HRC);
    return (2u + bloco) * n + v * hrcs[iq()].eixo.x + u;
}

// ⭐ P14c-placa 6 — logo depois da `hrc_desce` do nível 0 (o mesmo despacho, os mesmos R de cima): as duas metades
// do cone 0 (a conta da `descer_cones`) e a diferença delas guardada para a `hrc_soma_momento`.
@compute @workgroup_size(64)
fn hrc_metades_momento(
    @builtin(global_invocation_id) gid0: vec3<u32>,
    @builtin(num_workgroups) nw: vec3<u32>,
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_id) li: vec3<u32>,
) {
    let gid = id_do_fio(gid0, wg, li, nw);
    if hrcs[iq()].nivel != 0u {
        return;
    }
    var xi = gid.y;
    var v = gid.x;
    if GRADE3D_HRC && COLUNAS_HRC {
        if gid.x >= hrcs[iq()].eixo.y || gid.z != 0u {
            return;
        }
    } else {
        let e = entrada(gid, nw);
        if e >= hrcs[iq()].total {
            return;
        }
        let d = decompor(e, 1u);
        if d.x != 0u {
            return;
        }
        xi = d.y;
        v = d.z;
    }
    if fora_do_arquivo(xi, v) || xi >= hrcs[iq()].eixo.x {
        return;
    }
    let menos = metade_da_descida(xi, v, 0u, 0u, t_da_aresta(xi, v, 0u));
    let mais = metade_da_descida(xi, v, 1u, 1u, t_da_aresta(xi, v, 1u));
    acumulado_momento[indice_da_diferenca(xi, v)] = vec4<f32>(mais - menos, 0.0);
}

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
    let q = direcao_do_quadrante(hrcs[iq()].quadrante);
    let w = q * SINC_DO_QUADRANTE;
    var mx = vec4<f32>(p.f * w.x, 0.0);
    var my = vec4<f32>(p.f * w.y, 0.0);
    // A diferença das metades (a sonda além da última, a de fora, fica só com `ω̂_q`).
    if p.sonda.x < hrcs[iq()].nx && p.sonda.x < hrcs[iq()].eixo.x && METADES_DO_MOMENTO {
        let d = acumulado_momento[indice_da_diferenca(p.sonda.x, p.sonda.y)].rgb * p.escala;
        let perp = vec2<f32>(-q.y, q.x) * PESO_DA_DIFERENCA;
        mx += vec4<f32>(d * perp.x, 0.0);
        my += vec4<f32>(d * perp.y, 0.0);
    }
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
