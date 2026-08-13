#![forbid(unsafe_code)]
//! **Comprimento de arco ao longo de uma polilinha** — a folha que faz
//! *"igualmente espaçado"* querer dizer **igualmente espaçado**.
//!
//! Amostrar uma curva em *parâmetro* uniforme amontoa os pontos nas curvas
//! fechadas; amostrar em *arco* uniforme é o que o olho lê como uniforme. O
//! shell entrega a curva já achatada (o grafo nunca vê uma Bézier, e não
//! precisa), então a polilinha é a representação inteira.
//!
//! ## Por que isto é uma crate, e não um módulo
//!
//! **DOIS nós fazem a MESMA pergunta sobre a MESMA curva desenhada:**
//! `motion.path` a percorre (distribui `count` instâncias ao longo dela) e
//! `motion.spline_wrap` embrulha um layout nela. *"Onde fica a fração de arco
//! `s`?"* é um fato só, e duas cópias dele divergem em silêncio — a fatoração
//! que este repo escreve toda vez que uma pergunta ganha o segundo consumidor.
//!
//! ## ⚠️ O amostrador CLAMPA; a política de ponta é do CHAMADOR
//!
//! Os dois nós discordam sobre o que acontece **no fim da curva**, e discordam
//! com razão: o `offset` do `motion.path` desliza o conjunto *e dá a volta*
//! (uma marquise correndo por um caminho), enquanto o `from`/`to` do
//! `motion.spline_wrap` **estende-se pela curva `[0, 1]`** e o elemento em
//! `u = 1` tem de pousar no FIM dela. Se o enrolamento morasse aqui, `s = 1,0`
//! viraria `0,0` (`1 − floor(1) = 0`) e o último elemento saltaria para o
//! começo — um defeito que nenhum dos dois nós poderia consertar sem desfazer o
//! outro. Então o amostrador clampa, e quem quer o laço enrola `s` antes de
//! perguntar. *Um amostrador não tem política de ponta; um nó tem.*
//!
//! Livre de transcendentais (HR-5): comprimentos de corda são `sqrt`, o resto é
//! aritmética.

/// Comprimento cumulativo em cada vértice: `lut[i]` é a distância do começo até
/// o vértice `i`, e `lut.last()` é o total. Menos de dois pontos → vazio.
#[must_use]
pub fn lut(pts: &[[f32; 2]]) -> Vec<f32> {
    if pts.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(pts.len());
    let mut acc = 0.0f32;
    out.push(0.0);
    for w in pts.windows(2) {
        let (dx, dy) = (w[1][0] - w[0][0], w[1][1] - w[0][1]);
        acc += (dx * dx + dy * dy).sqrt();
        out.push(acc);
    }
    out
}

/// O ponto (e a tangente unitária) na fração de arco `s` da polilinha.
///
/// `s` é **CLAMPADO** em `[0, 1]` — ver a nota de ponta no topo do módulo. Uma
/// polilinha de menos de dois pontos, ou de comprimento zero, devolve o primeiro
/// ponto e a tangente `+x` (o degenerado que não inventa uma direção).
#[must_use]
pub fn at(pts: &[[f32; 2]], lut: &[f32], s: f32) -> ([f32; 2], [f32; 2]) {
    let total = *lut.last().unwrap_or(&0.0);
    if pts.len() < 2 || total <= 0.0 {
        return (*pts.first().unwrap_or(&[0.0, 0.0]), [1.0, 0.0]);
    }
    let target = s.clamp(0.0, 1.0) * total; // CLAMP-OK: uma fração da curva inteira
    // O segmento em que o alvo cai. Varredura linear: um caminho tem dezenas de
    // pontos, e uma busca binária aqui seria um segundo jeito de errar a mesma
    // pergunta.
    let mut i = 0;
    while i + 2 < lut.len() && lut[i + 1] < target {
        i += 1;
    }
    let (a, b) = (pts[i], pts[i + 1]);
    let seg = (lut[i + 1] - lut[i]).max(f32::MIN_POSITIVE);
    let t = ((target - lut[i]) / seg).clamp(0.0, 1.0); // CLAMP-OK: uma fração de UM segmento
    let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt().max(f32::MIN_POSITIVE);
    (p, [dx / len, dy / len])
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
