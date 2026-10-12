// ⭐ W5h — o MIOLO dos opacos pela DIFUSÃO (a lei do `ph2d_light2d_ref::OpcoesDoMiolo`): dentro da
// união dos opacos `Δu − u/ℓ² = 0`, com `u` preso à luz do ANEL (os texels de dentro com uma vizinha de
// fora, que a `c3_fina` calcula no pé da borda) e a mistura `suavidade` com o exterior mais próximo.
//
// A pirâmide (pull-push e Jacobi por nível, Jeschke et al. 2009 «GPU-friendly diffusion curves»): a
// `c3_fina` marca o interior no alfa da fluência; `miolo_classifica` escreve o nível 0 do CONHECIDO
// (rgb = a luz, a = o peso: 1 no anel, 0 no miolo fundo, −1 fora dos opacos ou da região — a média
// do exterior numa célula grossa puxava a solução para a luz de fora);
// `miolo_puxa` sobe (o peso de um texel grosso = a fração conhecida dos filhos); do mais grosso ao
// nível 1, `miolo_desce` interpola o nível de cima e `miolo_relaxa` itera
// `u = w·k + (1 − w)·Σ u_viz / (m + (h/ℓ)²)` (as vizinhas de fora da região não contam: Neumann),
// em f32; `miolo_final` faz uma relaxação na fina sobre o nível 1, mistura a transmitida dos
// translúcidos (W5k) e escreve a fluência.
//
// `FORMATO` é o da `saida` de cada passe (a fluência, o conhecido em f16, a solução em f32), trocado
// no texto ao criar o módulo.

struct Nivel {
    // A região deste nível, `[inicio, fim)` em texels dele.
    inicio: vec2<u32>,
    fim: vec2<u32>,
    // A região do OUTRO nível que o passe lê: o de baixo (puxa) ou o de cima (desce).
    outro_inicio: vec2<u32>,
    outro_fim: vec2<u32>,
    // (h_nível / ℓ)²: 0 sem penetração.
    kh2: f32,
    suavidade: f32,
    _p: vec2<f32>,
}

@group(0) @binding(0) var<uniform> nv: Nivel;
// O que o passe lê: a fluência (classifica), o conhecido de baixo (puxa), a solução (relaxa, final)
// ou a solução de cima (desce).
@group(0) @binding(1) var entrada: texture_2d<f32>;
// O conhecido DESTE nível (relaxa, desce, final) — ou o de cima (puxa: não usado).
@group(0) @binding(2) var conhecido: texture_2d<f32>;
// O conhecido do nível de CIMA (desce: quem da interpolação existe).
@group(0) @binding(3) var conhecido_acima: texture_2d<f32>;
@group(0) @binding(4) var saida: texture_storage_2d<FORMATO, write>;
// ⭐ W5k: num texel de dentro, a luz que atravessa o corpo até ele (rgb) e a opacidade `α` (a); a
// `c3_fina` escreve-a em todo texel remapeado (`α = 1` no opaco).
@group(0) @binding(5) var transmitida: texture_2d<f32>;

const AUSENTE: f32 = -1.0;

fn na_regiao(t: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(nv.inicio)) && all(t < vec2<i32>(nv.fim));
}

fn na_outra(t: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(nv.outro_inicio)) && all(t < vec2<i32>(nv.outro_fim));
}

fn alvo(gid: vec3<u32>) -> vec2<i32> {
    return vec2<i32>(gid.xy + nv.inicio);
}

const VIZ: array<vec2<i32>, 4> = array<vec2<i32>, 4>(
    vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1)
);

@compute @workgroup_size(8, 8)
fn miolo_classifica(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let f = textureLoad(entrada, t, 0);
    // Fora dos opacos o texel não entra na pirâmide: só o anel prende a solução.
    var w = AUSENTE;
    if f.a < 0.5 {
        // Dentro: conhecido só no anel (uma vizinha de fora na região).
        w = 0.0;
        for (var k = 0; k < 4; k++) {
            let v = t + VIZ[k];
            if na_regiao(v) && textureLoad(entrada, v, 0).a >= 0.5 {
                w = 1.0;
            }
        }
    }
    textureStore(saida, t, vec4<f32>(f.rgb, w));
}

// `entrada` = o conhecido do nível de baixo; a região é a DESTE nível.
@compute @workgroup_size(8, 8)
fn miolo_puxa(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let n = vec2<i32>(textureDimensions(entrada));
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    var presentes = 0.0;
    for (var k = 0; k < 4; k++) {
        let c = 2 * t + vec2<i32>(k & 1, k >> 1);
        if any(c >= n) || !na_outra(c) {
            continue;
        }
        let f = textureLoad(entrada, c, 0);
        if f.a < 0.0 {
            continue;
        }
        presentes += 1.0;
        peso += f.a;
        soma += f.a * f.rgb;
    }
    if presentes == 0.0 {
        textureStore(saida, t, vec4<f32>(0.0, 0.0, 0.0, AUSENTE));
        return;
    }
    textureStore(saida, t, vec4<f32>(soma / max(peso, 1e-20), peso / presentes));
}

// Uma relaxação de Jacobi em `t` sobre a solução `entrada`, com o conhecido `k` dele.
fn relaxado(t: vec2<i32>, k: vec4<f32>) -> vec3<f32> {
    if k.a >= 1.0 {
        return k.rgb;
    }
    var soma = vec3<f32>(0.0);
    var m = 0.0;
    for (var j = 0; j < 4; j++) {
        let v = t + VIZ[j];
        if na_regiao(v) && textureLoad(conhecido, v, 0).a >= 0.0 {
            soma += textureLoad(entrada, v, 0).rgb;
            m += 1.0;
        }
    }
    let livre = select(k.rgb, soma / (m + nv.kh2), m > 0.0);
    return mix(livre, k.rgb, k.a);
}

@compute @workgroup_size(8, 8)
fn miolo_relaxa(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    textureStore(saida, t, vec4<f32>(relaxado(t, k), 1.0));
}

// O nível mais grosso começa no próprio conhecido.
@compute @workgroup_size(8, 8)
fn miolo_inicia(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    textureStore(saida, t, vec4<f32>(select(k.rgb, vec3<f32>(0.0), k.a < 0.0), 1.0));
}

// A bilinear da solução de CIMA (`entrada`) no texel `t` deste nível, só das vizinhas presentes;
// `w = 0` se nenhuma.
fn grosso_em(t: vec2<i32>) -> vec4<f32> {
    let n = vec2<i32>(textureDimensions(entrada));
    let u = (vec2<f32>(t) + 0.5) * 0.5 - 0.5;
    let b = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(b);
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    for (var j = 0; j < 4; j++) {
        let o = vec2<i32>(j & 1, j >> 1);
        let c = clamp(b + o, vec2<i32>(0), n - 1);
        if !na_outra(c) || textureLoad(conhecido_acima, c, 0).a < 0.0 {
            continue;
        }
        let wx = select(1.0 - fr.x, fr.x, o.x == 1);
        let wy = select(1.0 - fr.y, fr.y, o.y == 1);
        soma += wx * wy * textureLoad(entrada, c, 0).rgb;
        peso += wx * wy;
    }
    return vec4<f32>(soma / max(peso, 1e-20), peso);
}

// `entrada` = a solução do nível de cima: a bilinear dela onde este nível não é conhecido.
@compute @workgroup_size(8, 8)
fn miolo_desce(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    let g = grosso_em(t);
    let grosso = select(k.rgb, g.rgb, g.a > 0.0);
    textureStore(saida, t, vec4<f32>(mix(grosso, k.rgb, k.a), 1.0));
}

// A FINA: no miolo fundo, uma relaxação sobre a solução do nível 1 interpolada (o anel com a luz
// dele) — a fluência = o exterior mais próximo misturado com ela pela `suavidade`. Sem iterar na
// fina: o armazenamento dela (f16) estagna o Jacobi e afasta-o da solução (medido, W5h).
@compute @workgroup_size(8, 8)
fn miolo_final(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    var l = k.rgb;
    if k.a >= 0.0 && k.a < 1.0 {
        var soma = vec3<f32>(0.0);
        var m = 0.0;
        for (var j = 0; j < 4; j++) {
            let v = t + VIZ[j];
            if !na_regiao(v) {
                continue;
            }
            let kv = textureLoad(conhecido, v, 0);
            if kv.a >= 1.0 {
                soma += kv.rgb;
                m += 1.0;
            } else if kv.a >= 0.0 {
                let g = grosso_em(v);
                if g.a > 0.0 {
                    soma += g.rgb;
                    m += 1.0;
                }
            }
        }
        if m > 0.0 {
            l = mix(k.rgb, soma / (m + nv.kh2), nv.suavidade);
        }
    }
    // ⭐ W5k: dentro de um translúcido, `mix(transmitida, miolo, α)` — o opaco (`α = 1`) fica no bit.
    if k.a >= 0.0 {
        let tr = textureLoad(transmitida, t, 0);
        l = select(mix(tr.rgb, l, tr.a), l, tr.a >= 1.0);
    }
    textureStore(saida, t, vec4<f32>(l, 1.0));
}
