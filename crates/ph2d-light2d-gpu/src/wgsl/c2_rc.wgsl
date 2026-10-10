// C2 — Radiance Cascades 2D (Sannikov), ramificação 4: a cascata i tem sondas a cada 2^i texels,
// 4^(i+1) direções e o intervalo [t_inicio, t_fim) = l0·[(4^i − 1)/3, (4^(i+1) − 1)/3). Traço e
// fusão no MESMO passe, de cima para baixo; cada fio é uma sonda × uma direção PRÉ-MÉDIA (a média
// das 4 subdireções 4m..4m+3), em arranjo direção-primeiro: o bloco (bx, by) de 2^i × 2^i guarda a
// direção m = by·2^i + bx. Na c0 (1 direção pré-média) a saída é a FLUÊNCIA: 2π × a média.
//
// Prelúdios (concatenados antes): `Real` (f16 | f32), `saida` (binding 4) e a `queda`.
//
// O meio num ponto (C0): DENTRO ⟺ a cobertura interpolada (bilinear entre centros) ≥ ½; o material
// (τ dentro, radiância) é o do vizinho mais coberto. Semântica (plano §2.3, a mesma do gabarito):
// - um oclusor multiplica por (1 − α) a cada ENTRADA: na transição de τ para τ' < τ entre duas
//   amostras, × τ'/τ. A entrada na FRONTEIRA do intervalo conta-se aqui: a amostra anterior é a
//   ÚLTIMA do intervalo de baixo — `recuo` = min(passo, comprimento dele) antes do início (com o
//   recuo de um passo inteiro e um intervalo de baixo mais curto, ela caía atrás da sonda e uma
//   entrada contava duas vezes). Na c0 a amostra anterior é «fora»: partir de dentro é entrada;
// - emissores são OPACOS (τ dentro = 0): ao entrar, o raio recolhe `perda · L·queda(|x − P|,
//   alcance)` e termina;
// - o céu entra pelos raios que escapam do intervalo da cascata de topo.

override FUSAO: u32 = 1u;
// O passo perto do conteúdo, em texels finos.
override PASSO: f32 = 1.0;

struct Cascata {
    tamanho: vec2<u32>,
    nivel: u32,
    topo: u32,
    t_inicio: f32,
    t_fim: f32,
    texel_m: f32,
    fator_jfa: u32,
    recuo: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
    ceu: vec4<f32>,
}

@group(0) @binding(0) var<uniform> cas: Cascata;
@group(0) @binding(1) var emissao: texture_2d<f32>;
@group(0) @binding(2) var sementes: texture_2d<u32>;
@group(0) @binding(3) var superior: texture_2d<f32>;
@group(0) @binding(5) var cobertura: texture_2d<f32>;
@group(0) @binding(6) var tau_dentro: texture_2d<f32>;
@group(0) @binding(7) var vizinhos: sampler;

const TAU: f32 = 6.283185307179586;
const VAZIO: u32 = 0xffffffffu;
const MEIA_DIAGONAL: f32 = 0.70710678;

struct Traco {
    r: vec3<Real>,
    t: Real,
}

struct Meio {
    tau: f32,
    texel: vec2<i32>,
}

// O meio em `p`: fora da grade, nada; dentro ⟺ cobertura interpolada ≥ ½.
fn meio(p: vec2<f32>) -> Meio {
    var m = Meio(1.0, vec2<i32>(0));
    let n = vec2<f32>(cas.tamanho);
    if p.x < 0.0 || p.y < 0.0 || p.x >= n.x || p.y >= n.y {
        return m;
    }
    let base = floor(p - 0.5);
    let fr = p - 0.5 - base;
    // x = (u0, v1), y = (u1, v1), z = (u1, v0), w = (u0, v0); bordas presas ao texel da borda.
    let g = textureGather(0, cobertura, vizinhos, (base + 1.0) / n);
    let c = mix(mix(g.w, g.z, fr.x), mix(g.x, g.y, fr.x), fr.y);
    if c < 0.5 {
        return m;
    }
    var melhor = g.w;
    var o = vec2<i32>(0, 0);
    if g.z > melhor {
        melhor = g.z;
        o = vec2<i32>(1, 0);
    }
    if g.x > melhor {
        melhor = g.x;
        o = vec2<i32>(0, 1);
    }
    if g.y > melhor {
        o = vec2<i32>(1, 1);
    }
    m.texel = clamp(vec2<i32>(base) + o, vec2<i32>(0), vec2<i32>(cas.tamanho) - 1);
    m.tau = textureLoad(tau_dentro, m.texel, 0).r;
    return m;
}

// Quanto se pode andar a partir de `p` sem chegar a um ponto «dentro». O JFA dá a célula-semente
// mais próxima do CENTRO da célula de `p`; desconta-se a distância de `p` a esse centro e o alcance
// da semente: os centros dos texels dela estão a (f−1)/2·√2 do centro da célula, e a interpolação
// de cada um chega a √2 dele — (f + 1)·√2/2 ao todo.
fn livre(p: vec2<f32>) -> f32 {
    let n = vec2<f32>(cas.tamanho);
    let fora = max(max(-p, p - n), vec2<f32>(0.0));
    if fora.x > 0.0 || fora.y > 0.0 {
        return length(fora);
    }
    let f = f32(cas.fator_jfa);
    let cel = vec2<i32>(floor(p / f));
    let s = textureLoad(sementes, cel, 0).x;
    if s == VAZIO {
        return 3.4e38;
    }
    let centro = (vec2<f32>(cel) + 0.5) * f;
    let semente = (vec2<f32>(f32(s & 0xffffu), f32(s >> 16u)) + 0.5) * f;
    return distance(centro, semente) - distance(p, centro) - (f + 1.0) * MEIA_DIAGONAL;
}

// O segmento A → B; `P` é a sonda (a distância da queda conta-se dela).
fn marchar(P: vec2<f32>, A: vec2<f32>, B: vec2<f32>) -> Traco {
    var tr = Traco(vec3<Real>(Real(0)), Real(1));
    let seg = B - A;
    let len = length(seg);
    if len <= 0.0 {
        return tr;
    }
    let d = seg / len;
    var tau_ant = 1.0;
    if cas.nivel > 0u {
        tau_ant = meio(A - d * cas.recuo).tau;
        // O intervalo começa DENTRO de algo opaco: todo raio que passe por aqui já foi absorvido
        // ali — (0, 0) é a resposta exata, e marchar o interior era trabalho que dependia do
        // tamanho do objeto (medido: um disco de 100 px custava 4× a penumbra).
        if tau_ant <= 0.0 {
            return Traco(vec3<Real>(Real(0)), Real(0));
        }
    }
    // ⭐ C3: as lâmpadas DIRETAS não estão na captura, mas são OPACAS (a lei do gabarito): o
    // segmento acaba na primeira que ele cruza (`c3_direta.wgsl`, círculo exato), e ali é absorvido.
    let tl = primeira_lampada(A, d, len);
    let s_fim = max(min(len, tl) - PASSO, 0.0);
    var s = 0.0;
    loop {
        let x = A + d * s;
        var passo = PASSO;
        let l = livre(x);
        if l > PASSO {
            tau_ant = 1.0;
            passo = l;
        } else {
            let m = meio(x);
            if m.tau < tau_ant {
                let e = textureLoad(emissao, m.texel, 0);
                if any(e.rgb > vec3<f32>(0.0)) {
                    let perda = 1.0 - m.tau / tau_ant;
                    let l = e.rgb * queda(distance(x, P) * cas.texel_m, e.a);
                    tr.r += tr.t * vec3<Real>(l * perda);
                }
                tr.t *= Real(m.tau / tau_ant);
                if tr.t <= Real(0) {
                    break;
                }
            }
            tau_ant = m.tau;
        }
        if s >= s_fim {
            break;
        }
        s = min(s + passo, s_fim);
    }
    if tl <= len {
        tr.t = Real(0);
    }
    return tr;
}

// O valor pré-médio da direção `k` (da cascata de cima) na sonda de cima `q`.
fn ler_superior(q: vec2<i32>, k: u32, sondas_sup: vec2<u32>) -> vec3<Real> {
    let blocos = 1u << (cas.nivel + 1u);
    let b = vec2<u32>(k % blocos, k / blocos);
    return vec3<Real>(textureLoad(superior, vec2<i32>(b * sondas_sup) + q, 0).rgb);
}

fn radiancia(P: vec2<f32>, sonda: vec2<u32>, d: vec2<f32>, k: u32, s: u32) -> vec3<Real> {
    let A = P + d * cas.t_inicio;
    if cas.topo == 1u {
        let tr = marchar(P, A, P + d * cas.t_fim);
        return tr.r + tr.t * vec3<Real>(cas.ceu.rgb);
    }
    let sondas_sup = cas.tamanho >> vec2<u32>(cas.nivel + 1u);
    let espac_sup = f32(2u << cas.nivel);
    let ultimo = vec2<i32>(sondas_sup) - 1;
    // A sonda em coordenadas de sonda de cima: `sonda/2 − ¼` — pesos ¾ e ¼ por eixo.
    let u = vec2<f32>(sonda) * 0.5 - 0.25;
    let base = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(base);

    switch FUSAO {
        case 0u: {
            let tr = marchar(P, A, P + d * cas.t_fim);
            var cima = vec3<Real>(Real(0));
            for (var j = 0u; j < 4u; j++) {
                let o = vec2<u32>(j & 1u, j >> 1u);
                let w = mix(1.0 - fr, fr, vec2<f32>(o));
                let q = clamp(base + vec2<i32>(o), vec2<i32>(0), ultimo);
                cima += Real(w.x * w.y) * ler_superior(q, k, sondas_sup);
            }
            return tr.r + tr.t * cima;
        }
        case 1u: {
            var soma = vec3<Real>(Real(0));
            for (var j = 0u; j < 4u; j++) {
                let o = vec2<u32>(j & 1u, j >> 1u);
                let w = mix(1.0 - fr, fr, vec2<f32>(o));
                let q = clamp(base + vec2<i32>(o), vec2<i32>(0), ultimo);
                let Q = (vec2<f32>(q) + 0.5) * espac_sup;
                let tr = marchar(P, A, Q + d * cas.t_fim);
                soma += Real(w.x * w.y) * (tr.r + tr.t * ler_superior(q, k, sondas_sup));
            }
            return soma;
        }
        default: {
            // Por eixo, a vizinha LONGE (peso ¼) numa das 4 subdireções, a PERTO (¾) nas outras
            // três; o turno roda com a sonda e a cascata para vizinhas não partilharem a escolha.
            let r = (sonda.x + 2u * sonda.y + cas.nivel) & 3u;
            let longe = vec2<bool>(s == r, s == ((r + 2u) & 3u));
            let perto = select(base, base + 1, fr > vec2<f32>(0.5));
            let outra = select(base + 1, base, fr > vec2<f32>(0.5));
            let q = clamp(select(perto, outra, longe), vec2<i32>(0), ultimo);
            let Q = (vec2<f32>(q) + 0.5) * espac_sup;
            let tr = marchar(P, A, Q + d * cas.t_fim);
            return tr.r + tr.t * ler_superior(q, k, sondas_sup);
        }
    }
}

@compute @workgroup_size(8, 8)
fn cascata(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= cas.tamanho.x || id.y >= cas.tamanho.y {
        return;
    }
    let i = cas.nivel;
    let sondas = cas.tamanho >> vec2<u32>(i);
    let bloco = id.xy / sondas;
    let sonda = id.xy % sondas;
    let m = bloco.y * (1u << i) + bloco.x;
    let P = (vec2<f32>(sonda) + 0.5) * f32(1u << i);
    let n_dir = 4u << (2u * i);
    var soma = vec3<Real>(Real(0));
    for (var s = 0u; s < 4u; s++) {
        let k = 4u * m + s;
        let ang = TAU * (f32(k) + 0.5) / f32(n_dir);
        soma += radiancia(P, sonda, vec2<f32>(cos(ang), sin(ang)), k, s);
    }
    var valor = vec3<f32>(soma) * 0.25;
    if i == 0u {
        valor *= TAU;
        // ⭐ C3: a luz DIRETA das lâmpadas (`c3_direta.wgsl`, concatenado depois deste).
        valor += direta(P);
    }
    textureStore(saida, id.xy, vec4<f32>(valor, 1.0));
}
