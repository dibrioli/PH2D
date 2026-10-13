// ⭐ W5n — o ANEL fora da REGIÃO (W5h §3.1): o miolo dos opacos resolve-se no corpo INTEIRO, não só na
// parte dele que a janela vê. Com a região menor que a grade, este passe escreve o conhecido do nível 0
// do `c3_miolo.wgsl` (o `miolo_fino`) onde o `miolo_classifica` não chega: fora da região (−1 fora dos
// corpos, 0 no miolo fundo, a luz do texel e 1 no anel — calculada como a `c3_fina` a calcularia) e na
// MOLDURA de dentro dela (os texels com uma vizinha fora: o anel deles depende dela). O predicado de
// «dentro» fora da região é o do corpo (`num_corpo`); dentro, o alfa da fluência que a `c3_fina` marcou.

// A fluência que a `c3_fina` acabou de escrever (a luz e o alfa do interior na região).
@group(2) @binding(7) var fluencia_lida: texture_2d<f32>;

const VIZ_ANEL: array<vec2<i32>, 4> = array<vec2<i32>, 4>(
    vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1)
);

fn na_janela(t: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(fina.inicio)) && all(t < vec2<i32>(fina.fim));
}

fn na_grade_fina(t: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(0)) && all(t < vec2<i32>(fina.tamanho));
}

// O texel `t` é de FORA dos corpos (o alfa ≥ ½ que o `miolo_classifica` lê).
fn anel_de_fora(t: vec2<i32>) -> bool {
    if na_janela(t) {
        return textureLoad(fluencia_lida, t, 0).a >= 0.5;
    }
    return !num_corpo(ponto_fino(vec2<u32>(t)));
}

// O que o passe faz no texel: `sair` (dentro da região, longe da borda dela) ou o conhecido `k`;
// `calcular` = um texel do anel FORA da região, cuja luz o chamador calcula.
struct Anel {
    k: vec4<f32>,
    calcular: bool,
    sair: bool,
}

fn anel_do_texel(alvo: vec2<u32>) -> Anel {
    let t = vec2<i32>(alvo);
    var r = Anel(vec4<f32>(0.0, 0.0, 0.0, -1.0), false, true);
    if !na_grade_fina(t) {
        return r;
    }
    let dentro = na_janela(t);
    if dentro {
        var moldura = false;
        for (var k = 0; k < 4; k++) {
            let v = t + VIZ_ANEL[k];
            moldura = moldura || (na_grade_fina(v) && !na_janela(v));
        }
        if !moldura {
            return r;
        }
    }
    r.sair = false;
    var rgb = vec3<f32>(0.0);
    if dentro {
        rgb = textureLoad(fluencia_lida, t, 0).rgb;
    }
    if anel_de_fora(t) {
        r.k = vec4<f32>(rgb, -1.0);
        return r;
    }
    var w = 0.0;
    for (var k = 0; k < 4; k++) {
        let v = t + VIZ_ANEL[k];
        if na_grade_fina(v) && anel_de_fora(v) {
            w = 1.0;
        }
    }
    r.k = vec4<f32>(rgb, w);
    r.calcular = !dentro && w >= 1.0;
    return r;
}

@compute @workgroup_size(8, 8)
fn c3_fina_anel(@builtin(global_invocation_id) gid: vec3<u32>) {
    let a = anel_do_texel(gid.xy);
    if a.sair {
        return;
    }
    var k = a.k;
    if a.calcular {
        // A sequência da `c3_fina` (sem a transmitida).
        borda_aqui = true;
        let onde = onde_calcular(gid.xy);
        k = vec4<f32>(luz_do_texel(gid.xy, onde, 0.0, false, false).l, 1.0);
    }
    textureStore(fina_saida, gid.xy, k);
}

@compute @workgroup_size(8, 8)
fn c3_fina_exata_anel(@builtin(global_invocation_id) gid: vec3<u32>) {
    let a = anel_do_texel(gid.xy);
    if a.sair {
        return;
    }
    var k = a.k;
    if a.calcular {
        // A sequência da `c3_fina_exata` (o translúcido lê as duas faces sem a borda exata).
        borda_aqui = true;
        let tau = meio(ponto_fino(gid.xy)).tau;
        let onde = onde_calcular(gid.xy);
        k = vec4<f32>(luz_do_texel(gid.xy, onde, tau, true, false).l, 1.0);
    }
    textureStore(fina_saida, gid.xy, k);
}
