//! ⭐⭐⭐ **A LEI DO ARCO — numa porta só** (2026-09-16).
//!
//! Um perfil com arcos ([`ph2d_field::Profile::arcs`]) é avaliado por **três** leitores: a árvore
//! GLOBAL ([`crate::profile::sd_profile`], a do dispositivo), a árvore ESPECIALIZADA por região
//! ([`crate::profile::sd_profile_in_region`], a do traçado em CPU — o modo MODEL) e o
//! [`crate::profile_index::ProfileIndex`], que corta as primitivas de cada região e responde
//! consultas directas. *Uma lei escrita em três sítios ainda não é uma lei* — e foi exactamente isso
//! que o smoke do dono apanhou: a wave do arco ensinou a lei à árvore global e deixou as outras duas
//! na polilinha, e o modo MODEL continuou a mostrar **faixas** (`12 196` picos de faceta contra `0`
//! na placa, cena `5`, vista de frente, `1920×1080`).
//!
//! # A convenção
//!
//! O *bulge* é o do DXF, `tan(θ/4)`, com `θ` o ângulo abarcado. **Positivo = o arco curva para a
//! ESQUERDA de `a→b`** — que é o lado onde `(b−a)×(p−a) > 0`. `|bulge| < 1` (arco menor) é garantido
//! pelo cozedor. Da corda e do bulge sai tudo o resto:
//!
//! ```text
//! s = bulge·|b−a|/2                       (a flecha, com sinal)
//! k = (s² − (|b−a|/2)²)/(2s)              (o centro, sobre a mediatriz)
//! c = meio + n̂·k        r = |s − k|       (n̂ = a normal ESQUERDA da corda)
//! ```
//!
//! # ⛔⛔ O EMPATE sobre a corda — o segundo defeito da wave
//!
//! O sinal de um perfil com arcos é o enrolamento das **cordas** mais a correcção da **meia-lua**
//! (a região entre a corda e o arco vale `∓1`). Um ponto **exactamente sobre a corda** está no
//! interior ou no exterior da figura verdadeira — a corda não é bordo de nada —, e as duas metades
//! têm de concordar sobre de que lado ele está. **A 1.ª versão não concordava**: o enrolamento pelo
//! raio `+x` põe esse ponto do lado `−dir` (depende do sentido vertical da aresta) e a meia-lua
//! usava um teste estrito que o punha sempre fora dela. Medido: um ponto a `0,0437` de profundidade
//! dentro da peça lia `+0,0437` — **fora** —, e uma grelha comum nunca o veria, porque nunca cai
//! numa corda (o gate que o apanhou **põe** os pontos lá).
//!
//! ⇒ **cada caminho do enrolamento tem a sua regra de empate, e a meia-lua copia a do caminho que a
//! acompanha**:
//!
//! | caminho | um ponto sobre a corda fica do lado… | a meia-lua conta-o sse… |
//! |---|---|---|
//! | raio `+x` (árvore global) | `−dir`; ou `sinal(eₓ)` se a corda é horizontal | esse lado é o do arco |
//! | caminho âncora→ponto (região, índice) | `(b−a)×(p−a) ≥ 0` — o semi-aberto do `path_crossing` | o arco é o da esquerda |

use fidget::context::Tree;

/// As constantes de um arco — calculadas UMA vez, na construção, e nunca por amostra.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Arco {
    /// O centro.
    pub c: [f64; 2],
    /// O raio.
    pub r: f64,
    /// A direcção unitária do centro para o MEIO do arco.
    pub m: [f64; 2],
    /// `cos(θ/2)`: um ponto está na cunha do arco sse `(p−c)·m ≥ cos(θ/2)·|p−c|`.
    pub cos_meio: f64,
    /// `|s|` — a maior distância entre o arco e a corda. ⭐ Todo ponto do arco está a menos disto
    /// da corda, e é isso que torna o corte espacial CONSERVADOR sem conhecer o arco.
    pub flecha: f64,
    /// `+1` se o arco curva para a esquerda de `a→b`, `−1` para a direita.
    pub lado: f64,
}

/// ⭐ **A porta das constantes.** Os três leitores chamam-na; nenhum faz a conta à mão.
pub(crate) fn arco(a: [f64; 2], b: [f64; 2], bulge: f64) -> Arco {
    let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
    let l = ex.hypot(ey);
    let s = bulge * l * 0.5;
    let k = (s * s - (l * 0.5) * (l * 0.5)) / (2.0 * s);
    let meio = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let n = [-ey / l, ex / l];
    let c = [meio[0] + n[0] * k, meio[1] + n[1] * k];
    let r = (s - k).abs();
    let topo = [meio[0] + n[0] * s - c[0], meio[1] + n[1] * s - c[1]];
    let lt = topo[0].hypot(topo[1]);
    let m = [topo[0] / lt, topo[1] / lt];
    let cos_meio = ((a[0] - c[0]) * m[0] + (a[1] - c[1]) * m[1]) / r;
    Arco {
        c,
        r,
        m,
        cos_meio,
        flecha: s.abs(),
        lado: bulge.signum(),
    }
}

// ─── as árvores ──────────────────────────────────────────────────────────────────────────────────

/// A distância² ao arco, como árvore: radial dentro da cunha, a ponta mais próxima fora dela —
/// **sem uma trigonométrica por amostra**.
pub(crate) fn dist2_tree(u: &Tree, v: &Tree, a: [f64; 2], b: [f64; 2], k: &Arco) -> Tree {
    let cwx = u.clone() - Tree::constant(k.c[0]);
    let cwy = v.clone() - Tree::constant(k.c[1]);
    let d = crate::ops::safe_sqrt(cwx.clone().square() + cwy.clone().square());
    let proj = cwx * Tree::constant(k.m[0]) + cwy * Tree::constant(k.m[1]);
    let na_cunha = proj
        .compare(d.clone() * Tree::constant(k.cos_meio))
        .max(0.0);
    let radial2 = (d - Tree::constant(k.r)).square();
    let pa2 =
        (u.clone() - Tree::constant(a[0])).square() + (v.clone() - Tree::constant(a[1])).square();
    let pb2 =
        (u.clone() - Tree::constant(b[0])).square() + (v.clone() - Tree::constant(b[1])).square();
    na_cunha.clone() * radial2 + (Tree::constant(1.0) - na_cunha) * pa2.min(pb2)
}

/// `[|p−c| < r]`, como árvore. ⚠️ Estrito: o círculo do lado do arco É o arco, e um empate ali é
/// um empate sobre a figura verdadeira — medida nula, como o de uma recta.
fn dentro_do_circulo_tree(u: &Tree, v: &Tree, k: &Arco) -> Tree {
    let d2 = (u.clone() - Tree::constant(k.c[0])).square()
        + (v.clone() - Tree::constant(k.c[1])).square();
    Tree::constant(k.r * k.r).compare(d2).max(0.0)
}

/// O termo do enrolamento que a MEIA-LUA acrescenta, `∓1` ou `0` (ver a nota do módulo).
fn termo(meia_lua: Tree, k: &Arco, non_zero: bool) -> Tree {
    // O ciclo «arco de a→b, corda de b→a» dá a volta à meia-lua uma vez, no sentido HORÁRIO quando o
    // arco curva para a esquerda ⇒ `−1`. Em paridade o sinal não importa.
    if non_zero {
        meia_lua * Tree::constant(-k.lado)
    } else {
        meia_lua
    }
}

/// ⭐ **A meia-lua para o caminho do RAIO `+x`** — a da árvore global.
///
/// `cross` e `dir` têm de ser **os mesmos nós** que o enrolamento da corda usa (`(b−a)×(p−a)` e
/// `above[b] − above[a]`): é a igualdade bit a bit das duas metades que torna o empate consistente.
pub(crate) fn meia_lua_raio_tree(
    u: &Tree,
    v: &Tree,
    corda: [f64; 2],
    k: &Arco,
    cross: &Tree,
    dir: &Tree,
    non_zero: bool,
) -> Tree {
    // Fora da corda: o lado do arco, estrito.
    let do_lado = (cross.clone() * Tree::constant(k.lado))
        .compare(0.0)
        .max(0.0);
    // SOBRE a corda: o raio conta-a do lado `−dir`; numa corda horizontal (`dir ≡ 0`) do lado
    // `sinal(eₓ)`, porque os `above` dos vizinhos tratam o ponto como estando por CIMA dela.
    let sobre = Tree::constant(1.0) - cross.clone().compare(0.0).abs();
    let pelo_dir = (dir.clone() * Tree::constant(-k.lado)).max(0.0);
    let horizontal = Tree::constant(1.0) - dir.clone().abs();
    let pelo_ex = if corda[0].signum() == k.lado {
        1.0
    } else {
        0.0
    };
    let empate = sobre * (pelo_dir + horizontal * Tree::constant(pelo_ex));
    termo(
        dentro_do_circulo_tree(u, v, k) * (do_lado + empate),
        k,
        non_zero,
    )
}

/// ⭐ **A meia-lua para o caminho ÂNCORA→PONTO** — a da árvore especializada por região.
///
/// `cross` tem de ser o **mesmo nó** que o `d2 = orient(a, b, p)` do atravessamento: o semi-aberto
/// dele põe um ponto sobre a corda do lado `≥ 0`, que é a ESQUERDA.
pub(crate) fn meia_lua_caminho_tree(
    u: &Tree,
    v: &Tree,
    k: &Arco,
    cross: &Tree,
    non_zero: bool,
) -> Tree {
    let negativo = Tree::constant(0.0).compare(cross.clone()).max(0.0);
    let do_lado = if k.lado > 0.0 {
        Tree::constant(1.0) - negativo
    } else {
        negativo
    };
    termo(dentro_do_circulo_tree(u, v, k) * do_lado, k, non_zero)
}

// ─── a mesma lei, em escalar (o índice) ─────────────────────────────────────────────────────────

/// A distância² ao arco, em escalar — a MESMA conta do [`dist2_tree`].
pub(crate) fn dist2(p: [f64; 2], a: [f64; 2], b: [f64; 2], k: &Arco) -> f64 {
    let w = [p[0] - k.c[0], p[1] - k.c[1]];
    let d = w[0].hypot(w[1]);
    if w[0] * k.m[0] + w[1] * k.m[1] > d * k.cos_meio {
        (d - k.r) * (d - k.r)
    } else {
        let pa = (p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2);
        let pb = (p[0] - b[0]).powi(2) + (p[1] - b[1]).powi(2);
        pa.min(pb)
    }
}

/// O termo da meia-lua para o caminho âncora→ponto, em escalar — a MESMA regra do
/// [`meia_lua_caminho_tree`]. `cross` é o `orient(a, b, p)` que o atravessamento usou.
pub(crate) fn meia_lua_caminho(p: [f64; 2], k: &Arco, cross: f32, non_zero: bool) -> i32 {
    let dentro = (p[0] - k.c[0]).powi(2) + (p[1] - k.c[1]).powi(2) < k.r * k.r;
    let do_lado = if k.lado > 0.0 {
        cross >= 0.0
    } else {
        cross < 0.0
    };
    if !(dentro && do_lado) {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation)]
    if non_zero { -(k.lado as i32) } else { 1 }
}
