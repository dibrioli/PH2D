// A CAMINHADA DE TRÁS PARA A FRENTE, no device — a metade que faltava do kill-criterion do ADR-0156.
//
// ⚠️ ESTA É A SEGUNDA IMPLEMENTAÇÃO DA MESMA LEI, e não há como não ser.
//
// O irmão `ph2d-paint-gpu` consegue contenção ESTRUTURAL (a lei do dab é 1-D em `t`, então a CPU manda
// uma TABELA e o device só amostra). Aqui não: `DabField::at` é um CAMPO VETORIAL por dab, função de
// `rel`, `mode`, `mv`, `perp`, `signed` e `pressure` — não existe tabela que o represente. O device tem
// de CARREGAR a lei, e fingir o contrário seria teatro. O precedente honesto é o `ImpastoLightPass`,
// cuja óptica também porta: a única defesa é um gate de PARIDADE que mede, e ele é
// `the_device_walk_reproduces_the_cpu_law`.
//
// ⚠️ O que NÃO está aqui, de propósito: `value_noise`. Ele é splitmix64 — aritmética de **64 bits**,
// que o WGSL do core não tem. Portá-lo exige ou uma textura de ruído ou um hash de 32 bits, e as duas
// MUDAM os bytes. É decisão da W1, e por isso o construtor do payload (lado Rust) **RECUSA** um dab que
// carregue ruído em vez de deixar o device responder outra coisa em silêncio.

struct Params {
    side: u32,
    dab_count: u32,
    rotor_len: u32,
    _pad: u32,
    origin: vec2<f32>,
    step: vec2<f32>,
}

// ⚠️ Só escalares e `vec2` — o alinhamento 16 de um `vec3` é o que faz o wgpu recusar o bind por um
// rabo que ninguém vê (a cicatriz que o `GpuDab` do carimbo já carrega). Tamanho 56, alinhamento 8.
struct Dab {
    center: vec2<f32>,
    mv: vec2<f32>,
    perp: vec2<f32>,
    inv_r2: f32,
    radius: f32,
    signed_v: f32,
    pressure: f32,
    distortion: f32,
    twist_deg_max: f32,
    mode: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> dabs: array<Dab>;
@group(0) @binding(2) var<storage, read> rotors: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> out: array<vec2<f32>>;

const PINCH_GAIN: f32 = 0.025;
const FOLD_GAIN: f32 = 0.05;

fn falloff(t2: f32) -> f32 {
    let s = 1.0 - t2;
    return s * s;
}

// ⚠️ UMA tabela de 361 entradas serve TODOS os dabs, e a razão é aritmética em vez de conveniência:
// `build_rotor_table(n)` são as primeiras `n+1` entradas da MESMA sequência. O único ponto em que a
// tabela por-dab da CPU e esta divergiriam é `k == deg_max`, onde a CPU cai em `unwrap_or(r0)` e aqui
// há uma entrada de verdade — e ali `frac` é **exatamente** zero (`a = deg_max·f` com `f = 1`), então
// `r0 + (r1 − r0)·0` devolve `r0` nos dois lados, ao bit.
fn twist_rotor(deg_f: f32) -> vec2<f32> {
    let a = abs(deg_f);
    let fl = floor(a);
    let k = u32(fl);
    let frac = a - fl;
    let last = p.rotor_len - 1u;
    let r0 = rotors[min(k, last)];
    let r1 = rotors[min(k + 1u, last)];
    var c = r0.x + (r1.x - r0.x) * frac;
    var s = r0.y + (r1.y - r0.y) * frac;
    let len = sqrt(c * c + s * s);
    if (len > 1e-6) {
        c = c / len;
        s = s / len;
    }
    if (deg_f < 0.0) {
        return vec2<f32>(c, -s);
    }
    return vec2<f32>(c, s);
}

fn radial_gain(d: Dab, f: f32, max_gain: f32) -> f32 {
    var dir = -1.0;
    if (d.signed_v > 0.0) {
        dir = 1.0;
    }
    return dir * max_gain * d.pressure * (1.0 + abs(d.signed_v)) * f;
}

// O espelho de `DabField::at`, escrito na MESMA ordem de operações (nada de `dot`, que dá ao
// compilador a chance de contrair em FMA onde a CPU não contrai).
fn at(d: Dab, pt: vec2<f32>) -> vec2<f32> {
    let rel = vec2<f32>(pt.x - d.center.x, pt.y - d.center.y);
    let t2 = (rel.x * rel.x + rel.y * rel.y) * d.inv_r2;
    if (t2 >= 1.0) {
        return vec2<f32>(0.0, 0.0);
    }
    let f = falloff(t2);
    switch d.mode {
        case 1u: {
            var deg_f = d.twist_deg_max * f;
            if (d.signed_v < 0.0) {
                deg_f = -deg_f;
            }
            let r = twist_rotor(deg_f);
            let rm = vec2<f32>(rel.x * r.x + rel.y * r.y, -rel.x * r.y + rel.y * r.x);
            return vec2<f32>(rel.x - rm.x, rel.y - rm.y);
        }
        case 2u: {
            let g = radial_gain(d, f, PINCH_GAIN);
            return vec2<f32>(rel.x * g, rel.y * g);
        }
        case 4u: {
            let g = radial_gain(d, f, FOLD_GAIN);
            let proj = rel.x * d.perp.x + rel.y * d.perp.y;
            return vec2<f32>(d.perp.x * proj * g, d.perp.y * proj * g);
        }
        case 5u: {
            return vec2<f32>(0.0, 0.0);
        }
        default: {
            return vec2<f32>(d.mv.x * f, d.mv.y * f);
        }
    }
}

// ⚠️ A caminhada lê `p`, a lista e NADA MAIS — nenhum vizinho, nenhum acumulador partilhado. É a
// condição que o ADR-0109 pede de todo kernel paralelo desta casa, e aqui ela vale por CONSTRUÇÃO.
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.side || gid.y >= p.side) {
        return;
    }
    let node = gid.y * p.side + gid.x;
    let pt = vec2<f32>(
        p.origin.x + f32(gid.x) * p.step.x,
        p.origin.y + f32(gid.y) * p.step.y,
    );
    var q = pt;
    var d = vec2<f32>(0.0, 0.0);
    var i = p.dab_count;
    loop {
        if (i == 0u) {
            break;
        }
        i = i - 1u;
        let v = at(dabs[i], q);
        d = vec2<f32>(d.x + v.x, d.y + v.y);
        q = vec2<f32>(q.x - v.x, q.y - v.y);
    }
    out[node] = d;
}
