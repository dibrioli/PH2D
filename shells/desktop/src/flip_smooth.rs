//! ADR-0114 W2 T2.7 — **active smoothing** do traço do Flip (o "assentar" que
//! separa uma mão de desenho premium de uma medíocre). Clean-room do
//! `paint.cc:544-621` (Grease Pencil 5.2): a intenção, não o código.
//!
//! **O mecanismo, e por que a cauda assenta enquanto a ponta ainda vibra:** o
//! smoothing é um **blur binomial 1D** (kernel `[1,2,1]/4` repetido — a
//! aproximação gaussiana transcendental-**free**, HR-5 limpo). O blur é **local**:
//! o valor suavizado de um ponto só depende dos ~`raio` vizinhos dele. Então, ao
//! adicionar uma amostra NOVA na ponta, a vizinhança de um ponto ANTIGO não muda
//! → ele "congela" sozinho, sem bookkeeping de convergência. Só os pontos perto
//! do cursor (vizinhança ainda incompleta) continuam se ajustando. Reblurar o
//! buffer inteiro a partir do CRU a cada frame dá exatamente esse comportamento.
//!
//! **Endpoints ancorados:** o 1º ponto (início do traço) e o ÚLTIMO (o cursor)
//! não se movem — a ponta segue o cursor sem lag; o interior é que assenta.

use ph2d_core::Vec2;

/// Máximo de iterações de blur (em `smoothing = 1.0`). Escala linear com o
/// slider Smoothing do painel.
const MAX_ITERS: u32 = 24;

/// Nº de iterações de blur para uma dada intensidade `smoothing ∈ [0,1]`.
#[must_use]
fn iters_for(smoothing: f32) -> u32 {
    (smoothing.clamp(0.0, 1.0) * MAX_ITERS as f32).round() as u32
}

/// Suaviza `raw` por blur binomial `[1,2,1]/4` com as PONTAS ancoradas, `iters`
/// vezes (de `smoothing`). Mantém a contagem de pontos (as pressões seguem 1:1);
/// a reamostragem/fit é do pen-up (T2.8). `iters = 0` (ou `< 3` pontos) devolve o
/// cru intacto.
///
/// **O kernel tem um dono só** (`ph2d_flip_reshape::blur`, W5): o mesmo alisador
/// serve o traço em curso (aqui, influência uniforme) e o pincel **Smooth** da
/// escultura (influência POR PONTO — o pincel alisa só onde alcança). Duas cópias
/// dele derivariam, e "o Smooth da caneta e o Smooth do pincel alisam diferente" é
/// exatamente o tipo de bug que ninguém procura.
#[must_use]
pub(crate) fn active_smooth(raw: &[Vec2], smoothing: f32) -> Vec<Vec2> {
    let iters = iters_for(smoothing);
    if iters == 0 || raw.len() < 3 {
        return raw.to_vec();
    }
    ph2d_flip_reshape::binomial_uniform(
        raw,
        iters,
        1.0, // o traço em curso alisa por igual (a "influência" é o nº de iterações)
        ph2d_flip_reshape::Ends::Anchored, // a ponta segue o cursor sem lag
        false, // um traço em curso nunca é fechado
    )
}

/// Distância perpendicular de `p` à reta `a→b` (transcendental-free além do
/// `sqrt` do comprimento). `|cross(ab, ap)| / |ab|`.
fn perp_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len = (ab.x * ab.x + ab.y * ab.y).sqrt();
    if len < 1e-9 {
        return (ap.x * ap.x + ap.y * ap.y).sqrt();
    }
    (ab.x * ap.y - ab.y * ap.x).abs() / len
}

/// **Simplify RDP** (Ramer–Douglas–Peucker) do pen-up (`paint.cc:1673-1799`):
/// devolve os ÍNDICES a manter (assim o chamador filtra as pressões junto),
/// preservando os pontos que desviam > `tol` (em MUNDO) da corda. Reduz a
/// contagem de pontos sem mudar a forma visível. `< 3` pontos = mantém tudo.
#[must_use]
pub(crate) fn simplify_rdp(points: &[Vec2], tol: f32) -> Vec<usize> {
    let n = points.len();
    if n < 3 {
        return (0..n).collect();
    }
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    // Pilha explícita (evita recursão profunda num traço longo).
    let mut stack = vec![(0usize, n - 1)];
    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }
        let mut max_d = 0.0;
        let mut idx = first;
        for i in (first + 1)..last {
            let d = perp_dist(points[i], points[first], points[last]);
            if d > max_d {
                max_d = d;
                idx = i;
            }
        }
        if max_d > tol {
            keep[idx] = true;
            stack.push((first, idx));
            stack.push((idx, last));
        }
    }
    (0..n).filter(|&i| keep[i]).collect()
}

/// Uma QUINA (`dot(dir_in, dir_out) < cos 60° = 0.5`, giro > 60°) NÃO é suavizada — a curva a
/// atravessa com tangente de UM lado só (mantém o vinco que o artista fez, o "quinas sobrevivem"
/// do RDP). Um traço à mão gentil (giros pequenos entre pontos) fica todo suave; um L de 90° (giro
/// 90° ⇒ dot 0 < 0.5) mantém a ponta. As PONTAS do traço são tratadas como quina (tangente unilateral).
const CORNER_COS: f32 = 0.5;

/// O intervalo de NÓ entre dois pontos da Catmull-Rom: `|Δ|^α` com **`α = ½` — a parametrização
/// CENTRÍPETA**, a única das três que é provadamente livre de cúspide E de auto-interseção
/// (Yuksel, Schaefer & Keyser, *Parameterization and Applications of Catmull-Rom Curves*, CAD 2011;
/// `α = 0` é a uniforme, `α = 1` a cordal). `½` sai como a raiz da distância — **dois `sqrt`, zero
/// transcendental** (HR-5 limpo), que é também por que α vive aqui na aritmética e não numa
/// constante: um expoente que ninguém lê é um knob que não gira.
///
/// ⚠️ **A uniforme NÃO bastava, e o comentário que dizia o contrário estava errado.** Ele afirmava
/// que *"as quinas — onde a uniforme dispararia — são tratadas à parte"*, ou seja que o único regime
/// perigoso era o giro AGUDO. O regime perigoso de verdade é o **ESPAÇAMENTO DESIGUAL com giro
/// GENTIL** — e espaçamento desigual é exatamente o que o RDP PRODUZ (ele apaga os pontos do trecho
/// reto e mantém os da curva), então o par RDP→reamostragem *fabricava* o caso ruim.
///
/// Medido (`measure_the_uniform_parameterisation_on_uneven_spacing`), sobre uma curva a 45°
/// (dot 0,71, bem dentro do "liso") cujo span curto tem corda 2,83 — **uniforme → centrípeta**:
///
/// | espaçamento | desvio à polilinha | inflação do arco | **giro máx. da saída** |
/// |---|---|---|---|
/// | par (CONTROLE) | 0,359 → 0,351 | 1,002 → 1,002 | 4,8° → 4,7° |
/// | 8:1  | 2,07 → 1,24 | 1,024 → 1,005 | 46,5° → 14,5° |
/// | 20:1 | 3,30 → 1,32 | 1,059 → 1,004 | 135,3° → 19,4° |
/// | 50:1 | **5,55 → 1,51** | 1,081 → **1,002** | **158,5° → 29,6°** |
///
/// Uma reversão de **158,5°** É uma cúspide: a "suavização" deslocava a tinta **duas vezes o
/// comprimento do span que ela estava suavizando**, numa barriga que ninguém desenhou.
///
/// Distância zero devolve zero, e o chamador nunca chega aqui com pontos coincidentes (o
/// `is_corner` já os trata como quina, então nenhuma divisão por nó estoura).
fn knot_step(a: Vec2, b: Vec2) -> f32 {
    let d = Vec2::new(b.x - a.x, b.y - a.y);
    (d.x * d.x + d.y * d.y).sqrt().sqrt()
}

/// A VELOCIDADE de Catmull-Rom num ponto interior liso, no espaço de NÓ (Barry–Goldman):
///
/// ```text
/// v_i = (P_i − P_{i−1})/dt_prev − (P_{i+1} − P_{i−1})/(dt_prev + dt_next) + (P_{i+1} − P_i)/dt_next
/// ```
///
/// Com `dt_prev == dt_next` ela REDUZ, termo a termo, a `½·(P[i+1] − P[i−1])/dt` — ou seja, a
/// tangente de Hermite sai **matematicamente idêntica** à uniforme quando o espaçamento é par
/// (⚠️ não *bit*-idêntica: sobra um `/dt` seguido de `·dt` em `f32`).
fn knot_velocity(prev: Vec2, cur: Vec2, next: Vec2, dt_prev: f32, dt_next: f32) -> Vec2 {
    let inv_p = 1.0 / dt_prev;
    let inv_n = 1.0 / dt_next;
    let inv_s = 1.0 / (dt_prev + dt_next);
    Vec2::new(
        (cur.x - prev.x) * inv_p - (next.x - prev.x) * inv_s + (next.x - cur.x) * inv_n,
        (cur.y - prev.y) * inv_p - (next.y - prev.y) * inv_s + (next.y - cur.y) * inv_n,
    )
}

/// **Reamostragem SUAVE do traço** (T2.8): interpola uma **Catmull-Rom** pelos pontos (o traço passa
/// EXATAMENTE por eles) e a reamostra a cada `step` de arco, densificando as curvas — o antídoto do
/// *"traço com poucos vértices fica tracejado e não arredondado nas curvas"* (Enio 2026-07-25). O
/// render liga os pontos por RETAS (cápsulas), então poucos pontos = facetas; adicionar vértices
/// sobre a curva suave dá o arredondado. As pressões (largura por-ponto) interpolam LINEARMENTE ao
/// longo do arco. **Quinas preservadas** (`CORNER_COS`): a curva não suaviza através de um vinco.
/// `< 3` pontos (ou `step ≤ 0`) devolve o cru — um traço de 2 pontos já é uma reta.
#[must_use]
pub(crate) fn resample_smooth(pts: &[Vec2], prs: &[f32], step: f32) -> (Vec<Vec2>, Vec<f32>) {
    let n = pts.len();
    if n < 3 || step <= 0.0 || pts.len() != prs.len() {
        return (pts.to_vec(), prs.to_vec());
    }
    // Direção unitária a→b, ou `None` se degenerada (pontos coincidentes).
    let dir = |a: Vec2, b: Vec2| -> Option<Vec2> {
        let d = Vec2::new(b.x - a.x, b.y - a.y);
        let l = (d.x * d.x + d.y * d.y).sqrt();
        (l >= 1e-9).then(|| Vec2::new(d.x / l, d.y / l))
    };
    // Um ponto é quina se o giro é agudo (ou é ponta, ou tem vizinho degenerado).
    let is_corner = |i: usize| -> bool {
        if i == 0 || i == n - 1 {
            return true;
        }
        match (dir(pts[i - 1], pts[i]), dir(pts[i], pts[i + 1])) {
            (Some(a), Some(b)) => a.x * b.x + a.y * b.y < CORNER_COS,
            _ => true,
        }
    };
    // Os intervalos de NÓ da parametrização centrípeta — um por span.
    let knots: Vec<f32> = (0..n - 1).map(|i| knot_step(pts[i], pts[i + 1])).collect();
    // Tangente de SAÍDA em P[i] (m0 do span i→i+1) e de ENTRADA em P[i] (m1 do span i−1→i): numa
    // quina, a corda do lado respectivo (vinco C0); num ponto liso, a de Catmull-Rom CENTRÍPETA.
    //
    // ⚠️ **A velocidade é do PONTO; a tangente é do SPAN.** A base de Hermite roda em `t ∈ [0,1]`,
    // então a mesma velocidade `v_i` entra escalada pelo intervalo de nó DAQUELE span — e é por
    // isso que `tan_out` e `tan_in` deixam de ser o mesmo número num ponto liso (na uniforme eram,
    // porque os dois intervalos valiam 1). Escalar pelo span errado devolve a barriga pela porta
    // dos fundos: é o span CURTO que precisa da tangente curta.
    let tan_out = |i: usize| -> Vec2 {
        if is_corner(i) {
            Vec2::new(pts[i + 1].x - pts[i].x, pts[i + 1].y - pts[i].y)
        } else {
            let v = knot_velocity(pts[i - 1], pts[i], pts[i + 1], knots[i - 1], knots[i]);
            Vec2::new(v.x * knots[i], v.y * knots[i])
        }
    };
    let tan_in = |i: usize| -> Vec2 {
        if is_corner(i) {
            Vec2::new(pts[i].x - pts[i - 1].x, pts[i].y - pts[i - 1].y)
        } else {
            let v = knot_velocity(pts[i - 1], pts[i], pts[i + 1], knots[i - 1], knots[i]);
            Vec2::new(v.x * knots[i - 1], v.y * knots[i - 1])
        }
    };

    let mut out_p = vec![pts[0]];
    let mut out_r = vec![prs[0]];
    for i in 0..n - 1 {
        let (p0, p1) = (pts[i], pts[i + 1]);
        let (m0, m1) = (tan_out(i), tan_in(i + 1));
        let chord = {
            let d = Vec2::new(p1.x - p0.x, p1.y - p0.y);
            (d.x * d.x + d.y * d.y).sqrt()
        };
        // Sub-passos: chord/step, capado (span longo não explode; span curto ganha 1 = a reta).
        let sub = ((chord / step).ceil() as usize).clamp(1, MAX_SUB_PER_SPAN);
        for k in 1..=sub {
            let t = k as f32 / sub as f32;
            let (t2, t3) = (t * t, t * t * t);
            // Base de Hermite.
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            out_p.push(Vec2::new(
                h00 * p0.x + h10 * m0.x + h01 * p1.x + h11 * m1.x,
                h00 * p0.y + h10 * m0.y + h01 * p1.y + h11 * m1.y,
            ));
            out_r.push(prs[i] + (prs[i + 1] - prs[i]) * t);
        }
    }
    (out_p, out_r)
}

/// Máximo de sub-pontos por span da reamostragem (evita explosão num span longo com passo fino).
const MAX_SUB_PER_SPAN: usize = 24;

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: usize) -> Vec<Vec2> {
        (0..n).map(|i| Vec2::new(i as f32, 0.0)).collect()
    }

    #[test]
    fn anchors_endpoints() {
        let raw = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 5.0),
            Vec2::new(2.0, -5.0),
            Vec2::new(3.0, 0.0),
        ];
        let s = active_smooth(&raw, 1.0);
        assert_eq!(s.first(), raw.first(), "1º ponto ancorado");
        assert_eq!(s.last(), raw.last(), "último (cursor) ancorado");
    }

    #[test]
    fn smoothing_zero_is_identity() {
        let raw = line(10);
        assert_eq!(active_smooth(&raw, 0.0), raw);
    }

    #[test]
    fn blur_reduces_a_spike() {
        // Um pico no meio de uma linha reta deve baixar (interior suavizado).
        let mut raw = line(9);
        raw[4].y = 10.0;
        let s = active_smooth(&raw, 1.0);
        assert!(s[4].y < 10.0, "o pico foi suavizado ({} < 10)", s[4].y);
        assert!(s[4].y > 0.0, "mas não zerado de vez");
    }

    /// A propriedade PREMIUM: adicionar uma amostra na PONTA não mexe num ponto
    /// já longe do cursor (ele "congelou") — o blur é local.
    #[test]
    fn far_points_freeze_when_the_tip_grows() {
        let mut raw = line(20);
        // dá um caráter ao traço pra o blur ter o que fazer
        for (i, p) in raw.iter_mut().enumerate() {
            p.y = if i % 2 == 0 { 1.0 } else { -1.0 };
        }
        raw[0].y = 0.0;
        let before = active_smooth(&raw, 1.0);
        // cresce a ponta com mais uma amostra
        let mut grown = raw.clone();
        grown.push(Vec2::new(20.0, 0.5));
        let after = active_smooth(&grown, 1.0);
        // Um ponto bem atrás (índice 5) não pode ter mudado — está fora do alcance
        // do kernel a partir da nova ponta (24 iters ⇒ raio ~5; índice 5 vs ponta
        // ~20 está muito além).
        let d = (after[5] - before[5]).x.abs() + (after[5] - before[5]).y.abs();
        assert!(d < 1e-4, "ponto distante congelou (Δ={d})");
    }

    #[test]
    fn rdp_collapses_a_straight_line_to_endpoints() {
        let raw = line(20);
        let keep = simplify_rdp(&raw, 0.5);
        assert_eq!(keep, vec![0, 19], "reta vira só as pontas");
    }

    #[test]
    fn rdp_keeps_a_corner() {
        // "V": desce e sobe → o vértice do fundo é preservado.
        let raw = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(2.0, -2.0), // vértice
            Vec2::new(3.0, -1.0),
            Vec2::new(4.0, 0.0),
        ];
        let keep = simplify_rdp(&raw, 0.2);
        assert!(keep.contains(&2), "o canto do V é mantido: {keep:?}");
        assert!(keep.len() < 5, "mas os colineares somem");
    }

    #[test]
    fn rdp_preserves_short_strokes() {
        assert_eq!(simplify_rdp(&line(2), 0.5), vec![0, 1]);
    }

    // ── Reamostragem suave (T2.8) ────────────────────────────────────────────

    /// O maior GIRO (graus) entre segmentos consecutivos da polilinha — o oráculo de
    /// "facetado vs liso": num traço liso os segmentos viram pouco a pouco.
    fn max_turn_deg(p: &[Vec2]) -> f32 {
        let mut worst = 0.0_f32;
        for w in p.windows(3) {
            let a = Vec2::new(w[1].x - w[0].x, w[1].y - w[0].y);
            let b = Vec2::new(w[2].x - w[1].x, w[2].y - w[1].y);
            let la = (a.x * a.x + a.y * a.y).sqrt();
            let lb = (b.x * b.x + b.y * b.y).sqrt();
            if la < 1e-6 || lb < 1e-6 {
                continue;
            }
            let cos = ((a.x * b.x + a.y * b.y) / (la * lb)).clamp(-1.0, 1.0);
            worst = worst.max(cos.acos().to_degrees());
        }
        worst
    }

    #[test]
    fn resample_short_stroke_is_identity() {
        let p = vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)];
        let (rp, rr) = resample_smooth(&p, &[1.0, 1.0], 1.0);
        assert_eq!(rp, p, "2 pontos = uma reta, intacta");
        assert_eq!(rr, vec![1.0, 1.0]);
    }

    #[test]
    fn resample_keeps_a_straight_line_on_the_line() {
        // Uma reta amostrada grosso: reamostrar adiciona pontos, mas TODOS na reta.
        let p = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            Vec2::new(60.0, 0.0),
        ];
        let (rp, _) = resample_smooth(&p, &[1.0; 3], 5.0);
        assert!(rp.len() > 3, "densificou: {} pontos", rp.len());
        for q in &rp {
            assert!(q.y.abs() < 1e-3, "ponto fora da reta: {q:?}");
        }
    }

    /// 🔴 **A curva grossa/esparsa é DENSIFICADA e fica LISA** (T2.8 — o report do Enio). Um arco
    /// amostrado grosso (giros de ~45° entre pontos = facetado) vira, depois da reamostragem, uma
    /// polilinha densa cujos giros são PEQUENOS (arredondada). Mutação que sangra: `resample_smooth`
    /// devolver o cru (sem densificar) — o giro máximo continua ~45° e o gate falha.
    #[test]
    fn resample_densifies_and_smooths_a_coarse_arc() {
        // Um semicírculo de raio 40 amostrado a cada 45° (5 pontos) — bem facetado.
        let mut arc = Vec::new();
        for k in 0..5 {
            let a = std::f32::consts::PI * (k as f32) / 4.0;
            arc.push(Vec2::new(40.0 * a.cos(), 40.0 * a.sin()));
        }
        let coarse_turn = max_turn_deg(&arc);
        assert!(coarse_turn > 40.0, "o arco cru É facetado: {coarse_turn}°");

        let (rp, _) = resample_smooth(&arc, &[1.0; 5], 3.0);
        assert!(rp.len() > 20, "densificou o arco: {} pontos", rp.len());
        let smooth_turn = max_turn_deg(&rp);
        assert!(
            smooth_turn < 15.0,
            "a curva reamostrada tem de ser LISA (giro máx {smooth_turn}° < 15; cru era {coarse_turn}°)"
        );
    }

    #[test]
    fn resample_preserves_a_sharp_corner() {
        // Um "L" de 90°: o vinco em (10,0) tem de sobreviver (não arredondar).
        let l = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
        ];
        let (rp, _) = resample_smooth(&l, &[1.0; 3], 2.0);
        // Algum vértice de saída ainda gira agudo (a quina foi preservada, não suavizada).
        assert!(
            max_turn_deg(&rp) > 60.0,
            "a quina de 90° tem de sobreviver: giro máx {}°",
            max_turn_deg(&rp)
        );
        // E cada braço do L continua reto (o vinco não vazou curvatura para os lados).
        for q in &rp {
            let on_h = q.y.abs() < 1e-3 && q.x <= 10.001;
            let on_v = (q.x - 10.0).abs() < 1e-3 && q.y >= -1e-3;
            assert!(on_h || on_v, "ponto fora dos braços do L: {q:?}");
        }
    }

    #[test]
    fn resample_interpolates_pressure_along_the_arc() {
        // Pressão 0→1 pelas pontas: os pontos densificados no meio ficam entre 0 e 1.
        let p = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 5.0),
            Vec2::new(40.0, 0.0),
        ];
        let (_, rr) = resample_smooth(&p, &[0.0, 0.5, 1.0], 3.0);
        assert_eq!(*rr.first().unwrap(), 0.0);
        assert_eq!(*rr.last().unwrap(), 1.0);
        assert!(
            rr.iter().all(|&r| (0.0..=1.0).contains(&r)),
            "pressões interpoladas ficam no intervalo"
        );
    }
}

#[cfg(test)]
#[path = "flip_smooth_resample_tests.rs"]
mod resample_measurement;
