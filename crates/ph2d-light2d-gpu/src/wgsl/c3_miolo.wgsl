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
    // ⭐ W5n: 1 = a região é menor que a grade — a moldura dela é do `c3_anel.wgsl` (que vê as vizinhas
    // de fora), e o `classifica` deixa-a.
    moldura: f32,
    // ⭐ W5q: 1 = o `puxa` do nível 1 lê um ladrilho apagado da fina como fora (o −1 do anel).
    ladrilhos: f32,
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
// ⭐ W5q: os ladrilhos da fina que o anel resolveu (`ladrilhos.wgsl`).
@group(0) @binding(6) var<storage, read> ladrilhos: array<u32>;

const AUSENTE: f32 = -1.0;

fn na_regiao(t: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(nv.inicio)) && all(t < vec2<i32>(nv.fim));
}

fn na_outra(t: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(nv.outro_inicio)) && all(t < vec2<i32>(nv.outro_fim));
}

// `t` está no nível que a `entrada`, a `saida` ou o `conhecido` deste passe têm (a grade INTEIRA dele).
fn no_nivel(t: vec2<i32>, n: vec2<i32>) -> bool {
    return all(t >= vec2<i32>(0)) && all(t < n);
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
    if nv.moldura > 0.5 {
        let n = vec2<i32>(textureDimensions(entrada));
        for (var k = 0; k < 4; k++) {
            let v = t + VIZ[k];
            if no_nivel(v, n) && !na_regiao(v) {
                return;
            }
        }
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
        if any(c >= n) || !na_outra(c) || (nv.ladrilhos > 0.5 && !ladrilho_aceso(c, n.x)) {
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
        // As vizinhas na grade inteira com o anel de fora da região (W5n); sem ele, só na região.
        let n = vec2<i32>(textureDimensions(conhecido));
        for (var j = 0; j < 4; j++) {
            let v = t + VIZ[j];
            if !no_nivel(v, n) || (nv.moldura < 0.5 && !na_regiao(v)) {
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

// ⭐ W5n — os V-ciclos de CORREÇÃO (Briggs, Henson e McCormick, «A Multigrid Tutorial», cap. 3): a
// cascata acima só DESCE a solução, e o erro de média frequência que os níveis grossos não acertam
// ficava (medido: a caixa de 2,2 m no zoom 2,5 com 2 058 texels a mais de 10 % do gabarito; a face do
// rebote +49 %). Sobre a solução u do nível 1: o defeito d = T(u) − u (T = uma relaxação); o erro e
// satisfaz e = (1 − w)·Σ e_viz/(m + (h/ℓ)²) + d, com e = 0 no conhecido; d desce (a média dos filhos
// presentes × σ, a escala do operador entre níveis), e resolve-se no mais grosso, sobe pela bilinear
// com relaxações amortecidas em cada nível e soma-se a u (que relaxa mais uma vez). Nos níveis do erro
// (≥ 2) todo texel com algum filho conhecido (w > 0) PRENDE o erro a zero: com o w fracionário o
// operador grosso quase não tem fronteira e a correção divergia (medido: ×10⁴ em dois ciclos; prender
// só com mais de metade conhecida também divergia).

// O amortecimento das relaxações do erro (Jacobi puro não apaga o modo xadrez).
const AMORTECE: f32 = 0.8;

// `entrada` = u do nível 1; `saida` = d.
@compute @workgroup_size(8, 8)
fn miolo_defeito(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    let d = relaxado(t, k) - textureLoad(entrada, t, 0).rgb;
    textureStore(saida, t, vec4<f32>(d, 1.0));
}

// `entrada` = d do nível de baixo, `conhecido_acima` = o conhecido de BAIXO (quem está presente);
// `saida` = d deste nível: σ × a média dos filhos presentes; σ = 4·(4 + κ_baixo)/(4 + κ), κ = (h/ℓ)².
@compute @workgroup_size(8, 8)
fn miolo_restringe(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a != 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    let n = vec2<i32>(textureDimensions(entrada));
    var soma = vec3<f32>(0.0);
    var presentes = 0.0;
    for (var j = 0; j < 4; j++) {
        let c = 2 * t + vec2<i32>(j & 1, j >> 1);
        if any(c >= n) || !na_outra(c) || textureLoad(conhecido_acima, c, 0).a < 0.0 {
            continue;
        }
        soma += textureLoad(entrada, c, 0).rgb;
        presentes += 1.0;
    }
    let sigma = (16.0 + nv.kh2) / (4.0 + nv.kh2);
    textureStore(saida, t, vec4<f32>(sigma * soma / max(presentes, 1.0), 1.0));
}

// O erro no nível mais grosso parte de d (a 1.ª relaxação a partir de zero): `entrada` = d.
@compute @workgroup_size(8, 8)
fn miolo_erro_inicia(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    var e = vec3<f32>(0.0);
    if k.a == 0.0 {
        e = textureLoad(entrada, t, 0).rgb;
    }
    textureStore(saida, t, vec4<f32>(e, 1.0));
}

// Uma relaxação amortecida do erro: `entrada` = e, `conhecido_acima` = d (o lado direito).
@compute @workgroup_size(8, 8)
fn miolo_erro_relaxa(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a != 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
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
    let d = textureLoad(conhecido_acima, t, 0).rgb;
    let e = textureLoad(entrada, t, 0).rgb;
    var livre = d;
    if m > 0.0 {
        livre = (1.0 - k.a) * soma / (m + nv.kh2) + d;
    }
    textureStore(saida, t, vec4<f32>(mix(e, livre, AMORTECE), 1.0));
}

// O erro de cima pela bilinear: `entrada` = e de cima, `conhecido_acima` = o conhecido de cima.
@compute @workgroup_size(8, 8)
fn miolo_erro_sobe(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    var e = vec3<f32>(0.0);
    if k.a == 0.0 {
        let g = grosso_em(t);
        e = select(vec3<f32>(0.0), g.rgb, g.a > 0.0);
    }
    textureStore(saida, t, vec4<f32>(e, 1.0));
}

// u + e no nível 1: `entrada` = e do nível 2, `conhecido_acima` = o conhecido dele, e u no binding 5.
@compute @workgroup_size(8, 8)
fn miolo_soma(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    let u = textureLoad(transmitida, t, 0).rgb;
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    var e = vec3<f32>(0.0);
    if k.a < 1.0 {
        let g = grosso_em(t);
        e = select(vec3<f32>(0.0), g.rgb, g.a > 0.0);
    }
    textureStore(saida, t, vec4<f32>(u + e, 1.0));
}
