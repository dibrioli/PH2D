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
// - ao ENTRAR num emissor o raio recolhe `L·queda(|x − P|, alcance)` (× a transmissão até ali); um
//   CORPO que brilha é opaco (τ dentro = 0) e termina-o; uma LÂMPADA (o disco de uma `Luz`) é luz,
//   não corpo (τ dentro = 1): o raio segue (smoke do dono, 10/10);
// - o céu entra pelos raios que escapam do intervalo da cascata de topo.

override FUSAO: u32 = 1u;
// O passo perto do conteúdo, em texels finos.
override PASSO: f32 = 1.0;
// ⭐ W5a: texels da captura ligada (`cobertura`, `tau_dentro`) por texel das cascatas — a luz direta
// marcha numa captura dos opacos mais fina (F1, F2); as cascatas, na delas (1).
override ESCALA_DA_CAPTURA: f32 = 1.0;
// O meio lê a EMISSÃO (a luz direta só precisa da transmissão: a captura fina não a tem).
override COM_LUZ: bool = true;
// ⭐ W5a (F3): a c0 marca no alfa (0) a sonda que cai DENTRO de um opaco — a C3 não a mistura.
override MASCARA_C0: bool = false;
// ⭐ W5c — a fusão junto a uma parede (o ABERTO 1 da W5a): um raio bilinear que acaba DENTRO de um
// opaco onde o raio verdadeiro (até `P + d·t_fim`) acaba fora — ou o contrário — vê a parede em vez
// do que está além dela. 0 = a bilinear de sempre; 1 = sem as sondas de cima que caem dentro de um
// opaco (o alfa de cada cascata); 2 = sem os raios de uma sonda de cima DENTRO de um opaco cujo FIM
// discorda do fim do verdadeiro (fora dos opacos a mistura de acertos e falhas na silhueta de uma luz
// pequena é a correção bilinear — tirá-la devolve o anel). Os pesos das que ficam renormalizam-se; se
// nenhuma fica, os de sempre.
override MASCARA_DA_FUSAO: u32 = 0u;
// ⭐ W5c — a FRONTEIRA entre intervalos junto a uma face. O intervalo de cima conta a entrada na
// fronteira pela amostra ANTERIOR na linha DELE; na fusão bilinear o raio que chega vem de outra
// sonda, por outra linha. Uma sonda de cima DENTRO de um opaco (ou sobre a face) tem essa amostra
// dentro: (1) o zero inteiro («quem passa aqui já foi absorvido») só quando a anterior E o início
// caem dentro; (2) o raio de baixo até ela amostra o próprio FIM e conta lá a entrada (senão o raio
// que acaba dentro da face perdia-a — o termo próprio de uma face que brilha). Fora dos opacos fica a
// regra de sempre: amostrar o fim também lá reforça o anel de uma luz pequena (medido, W5c).
override FRONTEIRA_NA_FACE: bool = false;
// ⭐ W5a (F1): o passo onde a captura tem cobertura (a menos de um texel de uma borda), em fração
// do `PASSO` — o raio que roça o canto do oclusor não o salta.
override PASSO_NA_BORDA: f32 = 1.0;
// ⭐ W5d — DENTRO pela borda EXATA da cobertura (não pelo iso-½ da bilinear) onde `borda_aqui`: ver
// [`dentro_exato`]. A C3 desliga-a na luz direta dos texels de FORA (a marcha deles fica no iso-½:
// a mesma foto, ~0,4 ms a menos na cena 4).
override BORDA_EXATA: bool = false;
// ⭐ W6d — a borda exata CONTÍNUA: o «dentro» pela bilinear da DISTÂNCIA com sinal que o `c3_borda`
// grava no centro de cada texel (o 4.º canal) — a reta por texel é descontínua entre texels e a busca
// do ponto de fronteira caía numa fresta «fora» entalada no corpo (handoff W6d §1).
override BORDA_CONTINUA: bool = false;
// ⭐ W6d — a luz direta dos texels de FORA também pela borda exata (o `Config::borda_fora`).
override BORDA_FORA: bool = false;
var<private> borda_aqui: bool = true;

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
    // A cobertura interpolada em `p` (0 longe de tudo).
    c: f32,
    texel: vec2<i32>,
    // Dentro de um EMISSOR (o texel dono tem radiância).
    luz: bool,
    // ⭐ W5c: a emissão do dono (rgb, 1/alcance), já lida — a entrada não a relê (o G4: a recolha na
    // entrada era o que crescia com o número de luzes).
    e: vec4<f32>,
}

// O meio em `pc` (texels das cascatas): fora da grade, nada; dentro ⟺ cobertura interpolada ≥ ½ —
// na captura ligada, na escala dela.
fn meio(pc: vec2<f32>) -> Meio {
    var m = Meio(1.0, 0.0, vec2<i32>(0), false, vec4<f32>(0.0));
    let p = pc * ESCALA_DA_CAPTURA;
    let n = vec2<f32>(cas.tamanho) * ESCALA_DA_CAPTURA;
    if p.x < 0.0 || p.y < 0.0 || p.x >= n.x || p.y >= n.y {
        return m;
    }
    let base = floor(p - 0.5);
    let fr = p - 0.5 - base;
    // x = (u0, v1), y = (u1, v1), z = (u1, v0), w = (u0, v0); bordas presas ao texel da borda.
    let g = textureGather(0, cobertura, vizinhos, (base + 1.0) / n);
    let c = mix(mix(g.w, g.z, fr.x), mix(g.x, g.y, fr.x), fr.y);
    m.c = c;
    if BORDA_EXATA && BORDA_CONTINUA && borda_aqui {
        // ⭐ W6d — a distância com sinal no centro de cada texel (`c3_borda`), bilinear: contínua.
        let s = textureGather(3, cobertura, vizinhos, (base + 1.0) / n);
        if mix(mix(s.w, s.z, fr.x), mix(s.x, s.y, fr.x), fr.y) < 0.0 {
            return m;
        }
    } else if BORDA_EXATA && borda_aqui {
        if !dentro_exato(p, n, c) {
            return m;
        }
    } else if c < 0.5 {
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
    m.texel = clamp(vec2<i32>(base) + o, vec2<i32>(0), vec2<i32>(n) - 1);
    m.tau = textureLoad(tau_dentro, m.texel, 0).r;
    if COM_LUZ {
        m.e = textureLoad(emissao, m.texel, 0);
        m.luz = any(m.e.rgb > vec3<f32>(0.0));
    }
    return m;
}

// ⭐ W5k — `pc` está DENTRO de um CORPO (τ < 1: todo emissor e todo oclusor de opacidade > 0): o
// predicado GEOMÉTRICO da C3 (preencher, o pé na face, o `x⁺` do rebote). A marcha lê o τ (transporte).
fn num_corpo(pc: vec2<f32>) -> bool {
    return meio(pc).tau < 1.0;
}

// ⭐ W5d — `p` (texels da captura ligada, grade `n`) está DENTRO pela borda EXATA da cobertura: a
// reta de cada texel parcial que o `c3_borda` (`c3_borda.wgsl`, o AAEDT) gravou no lugar da cobertura
// (`(cobertura, normal, recuo)`); texel cheio dentro, vazio fora, sem reta `c` (o iso-½). O iso-½ da
// bilinear desvia-se de uma borda reta até ~0,08 texel conforme a posição (o ABERTO 5 da W5c).
fn dentro_exato(p: vec2<f32>, n: vec2<f32>, c: f32) -> bool {
    let e = textureLoad(cobertura, clamp(vec2<i32>(floor(p)), vec2<i32>(0), vec2<i32>(n) - 1), 0);
    if e.r >= 1.0 {
        return true;
    }
    if e.r <= 0.0 {
        return false;
    }
    if e.g == 0.0 && e.b == 0.0 {
        return c >= 0.5;
    }
    return dot(p - floor(p) - 0.5, e.gb) >= e.a;
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
    return marchar_ate(P, A, B, false);
}

// [`marchar`]; `ate_o_fim` = a última amostra é o próprio B (a fronteira é deste raio).
fn marchar_ate(P: vec2<f32>, A: vec2<f32>, B: vec2<f32>, ate_o_fim: bool) -> Traco {
    var tr = Traco(vec3<Real>(Real(0)), Real(1));
    let seg = B - A;
    let len = length(seg);
    if len <= 0.0 {
        return tr;
    }
    let d = seg / len;
    var tau_ant = 1.0;
    // A amostra anterior estava dentro de um emissor (a ENTRADA nele conta uma vez).
    var luz_ant = false;
    if cas.nivel > 0u {
        let antes = meio(A - d * cas.recuo);
        tau_ant = antes.tau;
        luz_ant = antes.luz;
        // O intervalo começa DENTRO de algo opaco: todo raio que passe por aqui já foi absorvido
        // ali — (0, 0) é a resposta exata, e marchar o interior era trabalho que dependia do
        // tamanho do objeto (medido: um disco de 100 px custava 4× a penumbra).
        if tau_ant <= 0.0 && (!FRONTEIRA_NA_FACE || meio(A).tau <= 0.0) {
            return Traco(vec3<Real>(Real(0)), Real(0));
        }
    }
    let s_fim = select(max(len - PASSO, 0.0), len, ate_o_fim);
    var s = 0.0;
    loop {
        let x = A + d * s;
        var passo = PASSO;
        let l = livre(x);
        if l > PASSO {
            tau_ant = 1.0;
            luz_ant = false;
            passo = l;
        } else {
            let m = meio(x);
            if m.luz && !luz_ant {
                // Alcance infinito (1/alcance = 0): a queda vale 1 exatamente.
                var l = m.e.rgb;
                if m.e.a > 0.0 {
                    l *= queda(distance(x, P) * cas.texel_m, m.e.a);
                }
                tr.r += tr.t * vec3<Real>(l);
            }
            if m.tau < tau_ant {
                tr.t *= Real(m.tau / tau_ant);
                if tr.t <= Real(0) {
                    break;
                }
            }
            tau_ant = m.tau;
            luz_ant = m.luz;
            if m.c > 0.0 {
                passo = PASSO * PASSO_NA_BORDA;
            }
        }
        if s >= s_fim {
            break;
        }
        s = min(s + passo, s_fim);
    }
    return tr;
}

// O valor pré-médio da direção `k` (da cascata de cima) na sonda de cima `q` — rgb e a máscara (a).
fn ler_superior4(q: vec2<i32>, k: u32, sondas_sup: vec2<u32>) -> vec4<f32> {
    let blocos = 1u << (cas.nivel + 1u);
    let b = vec2<u32>(k % blocos, k / blocos);
    return textureLoad(superior, vec2<i32>(b * sondas_sup) + q, 0);
}

fn ler_superior(q: vec2<i32>, k: u32, sondas_sup: vec2<u32>) -> vec3<Real> {
    return vec3<Real>(ler_superior4(q, k, sondas_sup).rgb);
}

// ⭐ W5c: os pesos bilineares das 4 sondas de cima × a máscara (`MASCARA_DA_FUSAO`); `fim` = o fim do
// raio verdadeiro, `espac_sup` e `d` para o fim de cada raio bilinear.
fn pesos_da_fusao(
    base: vec2<i32>,
    fr: vec2<f32>,
    k: u32,
    sondas_sup: vec2<u32>,
    ultimo: vec2<i32>,
    fim: vec2<f32>,
    passo_sup: f32,
    dir: vec2<f32>,
) -> vec4<f32> {
    var w = vec4<f32>(0.0);
    var m = vec4<f32>(1.0);
    var fora_verdadeiro = true;
    if MASCARA_DA_FUSAO == 2u {
        fora_verdadeiro = meio(fim).tau > 0.0;
    }
    for (var j = 0u; j < 4u; j++) {
        let o = vec2<u32>(j & 1u, j >> 1u);
        let ww = mix(1.0 - fr, fr, vec2<f32>(o));
        w[j] = ww.x * ww.y;
        let q = clamp(base + vec2<i32>(o), vec2<i32>(0), ultimo);
        if MASCARA_DA_FUSAO == 1u {
            m[j] = ler_superior4(q, k, sondas_sup).a;
        } else if MASCARA_DA_FUSAO == 2u && ler_superior4(q, k, sondas_sup).a <= 0.0 {
            let fim_j = (vec2<f32>(q) + 0.5) * passo_sup + dir * cas.t_fim;
            m[j] = select(0.0, 1.0, (meio(fim_j).tau > 0.0) == fora_verdadeiro);
        }
    }
    let wm = w * m;
    let soma = wm.x + wm.y + wm.z + wm.w;
    if soma <= 0.0 {
        return w;
    }
    return wm / soma;
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

    let pesos = pesos_da_fusao(base, fr, k, sondas_sup, ultimo, P + d * cas.t_fim, espac_sup, d);
    switch FUSAO {
        case 0u: {
            let tr = marchar(P, A, P + d * cas.t_fim);
            var cima = vec3<Real>(Real(0));
            for (var j = 0u; j < 4u; j++) {
                let o = vec2<u32>(j & 1u, j >> 1u);
                let q = clamp(base + vec2<i32>(o), vec2<i32>(0), ultimo);
                cima += Real(pesos[j]) * ler_superior(q, k, sondas_sup);
            }
            return tr.r + tr.t * cima;
        }
        case 1u: {
            var soma = vec3<Real>(Real(0));
            for (var j = 0u; j < 4u; j++) {
                if pesos[j] <= 0.0 {
                    continue;
                }
                let o = vec2<u32>(j & 1u, j >> 1u);
                let q = clamp(base + vec2<i32>(o), vec2<i32>(0), ultimo);
                let Q = (vec2<f32>(q) + 0.5) * espac_sup;
                let sup = ler_superior4(q, k, sondas_sup);
                let tr = marchar_ate(P, A, Q + d * cas.t_fim, FRONTEIRA_NA_FACE && sup.a <= 0.0);
                soma += Real(pesos[j]) * (tr.r + tr.t * vec3<Real>(sup.rgb));
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
    var fora = 1.0;
    if i == 0u {
        // A c0 é a fluência das CASCATAS; a luz direta das lâmpadas soma-se na grade fina (C3).
        valor *= TAU;
    }
    if ((i == 0u && MASCARA_C0) || (i > 0u && (MASCARA_DA_FUSAO > 0u || FRONTEIRA_NA_FACE)))
        && meio(P).tau <= 0.0 {
        fora = 0.0;
    }
    textureStore(saida, id.xy, vec4<f32>(valor, fora));
}
