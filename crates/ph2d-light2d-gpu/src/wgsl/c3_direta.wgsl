// C3 — a luz DIRETA das lâmpadas (`Luz`), somada à fluência da c0 (smoke do dono, 2026-10-09/10:
// «a sombra não fica reta», «o tamanho afeta a qualidade»). As cascatas dão a borda da sombra com a
// resolução das sondas de longe (medido: ondulação 0,8–4 px, e manchas numa lâmpada pequena); aqui a
// visibilidade do DISCO é medida de cada sonda da c0 por raios até ele, pela MESMA marcha da C2
// (`marchar`: a mesma lei da entrada e dos opacos). Fluência de um disco uniforme no plano:
// F = L · ∫ T(θ) dθ sobre a abertura dele (2·asin(r/d)); de um PONTO (r = 0): F = I·queda/r.
//
// Concatenado DEPOIS de `c2_rc.wgsl` (usa `cas`, `marchar`, `livre`, `queda`, `TAU`). As lâmpadas
// estão na grade em texels finos; as de fora da grade também iluminam (a marcha só vê a grade).

struct Lampada {
    centro: vec2<f32>,
    raio: f32,
    inv_alcance: f32,
    // rgb: a radiância do disco (r > 0) ou a intensidade do ponto (r = 0).
    radiancia: vec4<f32>,
}

struct Lampadas {
    n: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
    itens: array<Lampada>,
}

@group(1) @binding(0) var<storage, read> lampadas: Lampadas;

// A abertura do disco na penumbra: `RAIOS_DIRETOS + 1` raios FIXOS nas fronteiras de
// `RAIOS_DIRETOS` fatias e, numa fatia em que a transmissão muda, `BISSECOES` passos de bissecção
// atrás do ângulo da borda — determinístico (sem sorteio: o «pontilhado» do smoke do dono), contínuo
// e reto (a borda é achada a 1/(RAIOS·2^BISSECOES) da abertura).
override RAIOS_DIRETOS: u32 = 8u;
override BISSECOES: u32 = 5u;
// Quanto antes do disco a marcha para (a lâmpada direta não está na captura; meio texel de folga
// contra o que estiver colado a ela).
const ANTES_DO_DISCO: f32 = 0.5;

// Onde o raio A + d·t (t ∈ [0, len]) entra no PRIMEIRO disco de lâmpada direta; `len + 1` se em
// nenhum. Partir de DENTRO de um disco é t = 0 (a lei da entrada, como nos opacos da captura).
fn primeira_lampada(A: vec2<f32>, d: vec2<f32>, len: f32) -> f32 {
    var melhor = len + 1.0;
    for (var k = 0u; k < lampadas.n; k++) {
        let L = lampadas.itens[k];
        if L.raio <= 0.0 {
            continue;
        }
        let o = A - L.centro;
        let c = dot(o, o) - L.raio * L.raio;
        if c <= 0.0 {
            return 0.0;
        }
        let b = dot(o, d);
        let disc = b * b - c;
        if b < 0.0 && disc >= 0.0 {
            let t = -b - sqrt(disc);
            if t <= len {
                melhor = min(melhor, t);
            }
        }
    }
    return melhor;
}

// A transmissão de P até B (o meio entre eles; o emissor de chegada fica de fora).
fn transmissao(P: vec2<f32>, B: vec2<f32>) -> f32 {
    return f32(marchar(P, P, B).t);
}

// A transmissão de P até o DISCO pelo ângulo `a` (a marcha para antes dele).
fn ate_o_disco(P: vec2<f32>, v: vec2<f32>, d: f32, raio: f32, a: f32) -> f32 {
    let dj = vec2<f32>(cos(a), sin(a));
    let b = dot(dj, v);
    let t = b - sqrt(max(b * b - (d * d - raio * raio), 0.0));
    return transmissao(P, P + dj * max(t - ANTES_DO_DISCO, 0.0));
}

// A fração VISÍVEL da abertura [base − α, base + α]: raios fixos nas fronteiras das fatias e, onde
// a transmissão muda, a bissecção da borda.
fn visivel_do_disco(P: vec2<f32>, v: vec2<f32>, d: f32, raio: f32, base: f32, alfa: f32) -> f32 {
    let n = RAIOS_DIRETOS;
    let largura = 2.0 * alfa / f32(n);
    var a0 = base - alfa;
    var t0 = ate_o_disco(P, v, d, raio, a0);
    var soma = 0.0;
    for (var i = 0u; i < n; i++) {
        let a1 = a0 + largura;
        let t1 = ate_o_disco(P, v, d, raio, a1);
        if abs(t1 - t0) < 1e-4 {
            soma += 0.5 * (t0 + t1) * largura;
        } else {
            var lo = a0;
            var hi = a1;
            for (var k = 0u; k < BISSECOES; k++) {
                let m = 0.5 * (lo + hi);
                let tm = ate_o_disco(P, v, d, raio, m);
                if abs(tm - t0) <= abs(tm - t1) {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let borda = 0.5 * (lo + hi);
            soma += t0 * (borda - a0) + t1 * (a1 - borda);
        }
        a0 = a1;
        t0 = t1;
    }
    return soma / (2.0 * alfa);
}

// O cone de P ao disco está LIVRE? — nenhum ponto «dentro» a menos da meia-largura do cone até
// `fim` (o caminho barato: fora da penumbra a visibilidade é exatamente 1).
fn cone_livre(P: vec2<f32>, dir: vec2<f32>, fim: f32, tg: f32, alvo: u32) -> bool {
    // As OUTRAS lâmpadas diretas (fora da captura): o disco delas toca o cone?
    for (var k = 0u; k < lampadas.n; k++) {
        let L = lampadas.itens[k];
        if k == alvo || L.raio <= 0.0 {
            continue;
        }
        let s_k = clamp(dot(L.centro - P, dir), 0.0, fim);
        if distance(P + dir * s_k, L.centro) < L.raio + s_k * tg + 1.0 {
            return false;
        }
    }
    var s = 0.0;
    loop {
        if s >= fim {
            break;
        }
        let folga = livre(P + dir * s) - s * tg;
        if folga <= 1.0 {
            return false;
        }
        s += folga;
    }
    return true;
}

fn direta(P: vec2<f32>) -> vec3<f32> {
    var soma = vec3<f32>(0.0);
    for (var k = 0u; k < lampadas.n; k++) {
        let L = lampadas.itens[k];
        let v = L.centro - P;
        let d = length(v);
        if L.raio <= 0.0 {
            // Ponto: um raio, `I·queda(r)/r`.
            if d <= 0.0 {
                continue;
            }
            let q = queda(d * cas.texel_m, L.inv_alcance);
            if q > 0.0 {
                soma += L.radiancia.rgb * (q / (d * cas.texel_m)) * transmissao(P, L.centro);
            }
            continue;
        }
        if d <= L.raio {
            // Dentro do disco: ele ocupa a volta inteira.
            soma += TAU * L.radiancia.rgb * queda(0.0, L.inv_alcance);
            continue;
        }
        if queda((d - L.raio) * cas.texel_m, L.inv_alcance) <= 0.0 {
            continue;
        }
        let alfa = asin(min(L.raio / d, 1.0));
        let dir = v / d;
        let fim = max(d - L.raio - ANTES_DO_DISCO, 0.0);
        if cone_livre(P, dir, fim, tan(alfa), k) {
            soma += L.radiancia.rgb * (2.0 * alfa * queda(d * cas.texel_m, L.inv_alcance));
            continue;
        }
        let base = atan2(dir.y, dir.x);
        let vis = visivel_do_disco(P, v, d, L.raio, base, alfa);
        let q = queda(d * cas.texel_m, L.inv_alcance);
        soma += L.radiancia.rgb * (2.0 * alfa * vis * q);
    }
    return soma;
}
