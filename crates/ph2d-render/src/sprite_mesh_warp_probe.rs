//! ⭐⭐⭐ **O QUE SOBRA DEPOIS DE MEDIR AO TAMANHO DO DAB — e porque NÃO é inevitável.**
//!
//! # O report que as pediu
//!
//! *«quase bom»* (dono, 2026-09-14, quarta foto do mesmo pincel, com uma seta sobre a marca no
//! ponto de maior dobra). A [`super`] tinha chamado ao resíduo *«inerente a uma elipse por dab»* e
//! nomeado dois diminuidores — **a malha mais fina** e **o pincel menor**. ⛔ **A primeira metade
//! dessa frase está REFUTADA por medição, e a segunda é a pista.**
//!
//! # As três sondas, e o que cada uma fechou
//!
//! ## 1. ⛔ A malha mais fina NÃO cura ([`probe_o_residuo_contra_a_finura_da_malha`])
//!
//! Redondeza da marca (`1` = disco) no leque, variando só a contagem de triângulos:
//!
//! | triângulos | dab pequeno | dab grande |
//! |---|---|---|
//! | `32` | `1,347` | `1,322` |
//! | `128` | `1,129` | `1,160` |
//! | `512` | `1,101` | `1,119` |
//! | `2 048` | `1,050` | `1,108` |
//! | `8 192` | `1,050` | **`1,107`** |
//!
//! ⇒ **estanca.** Quadruplicar a malha de `2 048` para `8 192` não move o terceiro decimal. O
//! resíduo **não é facetagem** — refinar a amostragem da deformação não o alcança, e o `Smooth` do
//! esqueleto (que é exactamente essa alavanca) não é a cura deste defeito.
//!
//! ## 2. ⛔ Medir sobre a região REALMENTE PINTADA não compra nada ([`probe_a_regiao_medida_contra_a_regiao_pintada`])
//!
//! A hipótese era boa e está morta: o ajuste corre sobre o **círculo de repouso** de raio
//! `footprint_uv`, mas o que se pinta é a elipse `W⁻¹·E`, que num leque forte é **`2,1×`** mais
//! comprida — logo a região medida não é a região pintada. Uma iteração de ponto fixo (ajustar, ver
//! que elipse sai, re-ajustar sobre ELA) devolve `1,161` contra `1,162`. **Ganho zero**, e nas doze
//! células ela tanto melhora como piora. *A região do ajuste não é a variável.*
//!
//! ## 3. ⭐⭐⭐ A variável é o GRAU da pegada ([`probe_o_grau_na_direccao_directa`])
//!
//! O kernel avalia `t = |M·p|` por texel, com `M` **linear** — e uma dobra não é linear. Medido na
//! direcção que o motor de facto avalia (texel → ecrã), aproximando o mapa da malha por um
//! polinómio de grau `g` à volta do centro do dab:
//!
//! | raio do dab | `g = 1` (hoje) | `g = 2` | `g = 3` |
//! |---|---|---|---|
//! | `0,06` | `1,054` | `1,010` | `1,008` |
//! | `0,125` | `1,118` | `1,021` | `1,005` |
//! | `0,20` | `1,197` | `1,055` | **`1,006`** |
//!
//! ⭐ **O `g = 1` da sonda reproduz o produto** (`1,054`/`1,118`/`1,197` contra `1,050`/`1,108`/
//! `1,179`) — é o controlo que diz que ela mede o mesmo programa.
//!
//! ⇒ **o resíduo é a CURVATURA da dobra dentro do próprio dab**, e é por isso que ele cresce com o
//! raio do pincel e é surdo à malha. A cura não é uma elipse melhor: é a pegada deixar de ser uma
//! elipse. ⭐ **Grau 3 leva o pior caso a `1,006`** — abaixo do que a métrica distingue.
//!
//! # ⛔ Uma quarta sonda foi construída e DEITADA FORA, e a lição fica
//!
//! Uma busca pelo *«melhor que uma elipse consegue»* varrendo `(escala, flatten, ângulo)` devolveu
//! `1,068` contra `1,108` do produto — e a escala do vencedor era **`0,70` contra `1,16`**. *Ela
//! comprava redondeza ENCOLHENDO o dab*: uma marca menor vê menos curvatura. **Uma busca que deixa
//! o TAMANHO flutuar não mede um tecto — mede outra pergunta**, e a resposta dela teria mandado
//! afinar o ajuste em vez de subir o grau.

use super::{SIZE, ecra, leque, redondeza};
use crate::sprite_mesh::SpriteMesh;
use crate::sprite_mesh_warp::warp_over;

/// Os monómios de grau `<= g` em `(x, y)`, sem termo constante (a origem é o centro do dab).
fn monomios(g: usize, x: f64, y: f64, out: &mut [f64; 9]) -> usize {
    let mut n = 0;
    out[n] = x;
    n += 1;
    out[n] = y;
    n += 1;
    if g >= 2 {
        out[n] = x * x;
        n += 1;
        out[n] = x * y;
        n += 1;
        out[n] = y * y;
        n += 1;
    }
    if g >= 3 {
        out[n] = x * x * x;
        n += 1;
        out[n] = x * x * y;
        n += 1;
        out[n] = x * y * y;
        n += 1;
        out[n] = y * y * y;
        n += 1;
    }
    n
}

/// Gauss com pivô parcial sobre `nt` incógnitas e DOIS lados direitos.
fn resolve(ata: &[[f64; 9]; 9], atb: &[[f64; 9]; 2], nt: usize) -> Vec<[f64; 2]> {
    let mut m = [[0.0f64; 11]; 9];
    for i in 0..nt {
        m[i][..nt].copy_from_slice(&ata[i][..nt]);
        m[i][nt] = atb[0][i];
        m[i][nt + 1] = atb[1][i];
    }
    for c in 0..nt {
        let piv = (c..nt)
            .max_by(|&a, &b| m[a][c].abs().total_cmp(&m[b][c].abs()))
            .expect("ha' pelo menos uma linha");
        m.swap(c, piv);
        let pv = m[c][c];
        for v in m[c].iter_mut().take(nt + 2) {
            *v /= pv;
        }
        for r in 0..nt {
            if r != c {
                let f = m[r][c];
                for k in 0..nt + 2 {
                    m[r][k] -= f * m[c][k];
                }
            }
        }
    }
    (0..nt).map(|i| [m[i][nt], m[i][nt + 1]]).collect()
}

/// Ver a §1 do cabeçalho: o resíduo contra a CONTAGEM DE TRIÂNGULOS.
#[test]
#[ignore = "sonda: o residuo contra a finura da malha"]
fn probe_o_residuo_contra_a_finura_da_malha() {
    for theta in [1.2_f32, 1.8] {
        println!("\n=== leque theta={theta} ===");
        println!("    n    tri    raio 0.06   raio 0.125");
        for n in [4usize, 8, 16, 32, 64] {
            let mesh = leque(n, theta, 1.4);
            let col: Vec<f32> = [0.06_f32, 0.125]
                .iter()
                .map(|&raio| {
                    let centro = [0.531_f32, 0.719_f32];
                    let p = ecra(&mesh, centro).map(|s| [s[0], -s[1]]).expect("posado");
                    let w = warp_over(&mesh, p, SIZE, [raio, raio]).expect("footprint");
                    redondeza(&mesh, centro, raio, w)
                })
                .collect();
            println!("  {n:3}  {:5}      {:6.3}       {:6.3}", 2 * n * n, col[0], col[1]);
        }
    }
}

/// O ajuste de mínimos quadrados do mapa da malha sobre deslocamentos ARBITRÁRIOS em UV de repouso
/// — a mesma conta do produto, com a região a medir livre.
fn ajuste(mesh: &SpriteMesh, centro: [f32; 2], duvs: &[[f32; 2]]) -> Option<[[f32; 2]; 2]> {
    let c0 = ecra(mesh, centro)?;
    let (mut ata, mut atb) = ([[0.0f64; 9]; 9], [[0.0f64; 9]; 2]);
    let mut n = 0;
    for d in duvs {
        let Some(q) = ecra(mesh, [centro[0] + d[0], centro[1] + d[1]]) else {
            continue;
        };
        let b = [f64::from(d[0] * SIZE[0]), f64::from(d[1] * SIZE[1])];
        let qv = [f64::from(q[0] - c0[0]), f64::from(q[1] - c0[1])];
        for i in 0..2 {
            for j in 0..2 {
                ata[i][j] += b[i] * b[j];
            }
            atb[0][i] += b[i] * qv[0];
            atb[1][i] += b[i] * qv[1];
        }
        n += 1;
    }
    if n < 3 || ata[0][0] * ata[1][1] - ata[0][1] * ata[1][0] == 0.0 {
        return None;
    }
    let c = resolve(&ata, &atb, 2);
    Some([
        [c[0][0] as f32, c[1][0] as f32],
        [c[0][1] as f32, c[1][1] as f32],
    ])
}

/// Os `n` deslocamentos do bordo de uma elipse de semi-eixos `(a, b)` rodada de `ang` graus.
fn bordo(a: f32, b: f32, ang: u16, n: usize) -> Vec<[f32; 2]> {
    let [c, s] = ph2d_painter_brush::texture::rotate_by_degrees(ang);
    (0..n)
        .map(|k| {
            let t = (k as f32) * std::f32::consts::TAU / (n as f32);
            let (x, y) = (a * t.cos(), b * t.sin());
            [x * c - y * s, x * s + y * c]
        })
        .collect()
}

/// Ver a §2 do cabeçalho: re-ajustar sobre a elipse REALMENTE pintada devolve o mesmo número.
#[test]
#[ignore = "sonda: a regiao medida contra a regiao pintada"]
fn probe_a_regiao_medida_contra_a_regiao_pintada() {
    for (n, theta) in [(8usize, 1.2f32), (8, 1.8), (16, 1.8)] {
        let mesh = leque(n, theta, 1.4);
        println!("\n=== leque n={n} theta={theta} ===");
        println!("  centro              raio    crua  facete  circulo  ELIPSE  (escala)");
        for raio in [0.06_f32, 0.125] {
            for centro in [[0.531_f32, 0.719_f32], [0.719, 0.806], [0.40, 0.65]] {
                let p = ecra(&mesh, centro).map(|s| [s[0], -s[1]]).expect("posado");
                let facete = warp_over(&mesh, p, SIZE, [0.0, 0.0]).expect("facete");
                let circulo = warp_over(&mesh, p, SIZE, [raio, raio]).expect("circulo");
                let d = ph2d_painter_brush::canvas_warp::warped_dab(circulo, 0.0, 0);
                let (a, b) = (
                    raio * d.radius_scale,
                    raio * d.radius_scale * (1.0 - d.flatten),
                );
                let elipse =
                    ajuste(&mesh, centro, &bordo(a, b, d.angle_deg, 16)).unwrap_or(circulo);
                println!(
                    "  {centro:?} {raio:6}  {:6.3}  {:6.3}   {:6.3}  {:6.3}   ({:.2})",
                    redondeza(&mesh, centro, raio, [[1.0, 0.0], [0.0, 1.0]]),
                    redondeza(&mesh, centro, raio, facete),
                    redondeza(&mesh, centro, raio, circulo),
                    redondeza(&mesh, centro, raio, elipse),
                    d.radius_scale,
                );
            }
        }
    }
}

/// ⭐⭐⭐ Ver a §3 do cabeçalho: **o GRAU da pegada, na direcção que o motor avalia.** Aproxima o
/// mapa da malha (texel → ecrã) por um polinómio de grau `g` sobre o disco que o dab ocupa, acha a
/// fronteira `|P(p)| = ρ` que o kernel pintaria, e leva-a ao ecrã PELA MALHA.
#[test]
#[ignore = "sonda: o grau da pegada, na direccao directa"]
fn probe_o_grau_na_direccao_directa() {
    for (n, theta) in [(32usize, 1.2f32), (32, 1.8)] {
        let mesh = leque(n, theta, 1.4);
        println!("\n=== leque n={n} theta={theta} ===");
        println!("  raio    g=1 (hoje)     g=2       g=3");
        for raio in [0.06_f32, 0.125, 0.2] {
            let centro = [0.531_f32, 0.719_f32];
            let p0 = ecra(&mesh, centro).map(|s| [s[0], -s[1]]).expect("posado");
            let w = warp_over(&mesh, p0, SIZE, [raio, raio]).expect("footprint");
            let d = ph2d_painter_brush::canvas_warp::warped_dab(w, 0.0, 0);
            let c0 = ecra(&mesh, centro).expect("centro");
            let rho = raio * d.radius_scale * SIZE[0] * (1.0 - d.flatten * 0.5);
            let ext = f64::from(raio * d.radius_scale);
            let mut linha = format!("  {raio:5}  ");
            let mut fora = false;
            for g in [1usize, 2, 3] {
                let nt = monomios(g, 1.0, 1.0, &mut [0.0; 9]);
                let (mut ata, mut atb) = ([[0.0f64; 9]; 9], [[0.0f64; 9]; 2]);
                for ri in 1..=6 {
                    let rr = ext * f64::from(ri) / 6.0;
                    for k in 0..24 {
                        let t = (k as f64) * std::f64::consts::TAU / 24.0;
                        let (dx, dy) = (rr * t.cos(), rr * t.sin());
                        let Some(s) = ecra(&mesh, [centro[0] + dx as f32, centro[1] + dy as f32])
                        else {
                            fora = true;
                            continue;
                        };
                        let mut b = [0.0f64; 9];
                        monomios(g, dx, dy, &mut b);
                        let q = [f64::from(s[0] - c0[0]), f64::from(s[1] - c0[1])];
                        for i in 0..nt {
                            for j in 0..nt {
                                ata[i][j] += b[i] * b[j];
                            }
                            atb[0][i] += b[i] * q[0];
                            atb[1][i] += b[i] * q[1];
                        }
                    }
                }
                let coef = resolve(&ata, &atb, nt);
                let poly = |dx: f64, dy: f64| {
                    let mut b = [0.0f64; 9];
                    monomios(g, dx, dy, &mut b);
                    (0..nt).fold([0.0f64; 2], |mut v, i| {
                        v[0] += coef[i][0] * b[i];
                        v[1] += coef[i][1] * b[i];
                        v
                    })
                };
                let (mut lo, mut hi) = (f32::INFINITY, 0.0f32);
                for k in 0..360 {
                    let t = (k as f64) * std::f64::consts::TAU / 360.0;
                    let (dx, dy) = (t.cos(), t.sin());
                    let (mut a, mut b) = (0.0f64, ext * 4.0);
                    for _ in 0..60 {
                        let mid = 0.5 * (a + b);
                        let v = poly(mid * dx, mid * dy);
                        if v[0].hypot(v[1]) < f64::from(rho) {
                            a = mid;
                        } else {
                            b = mid;
                        }
                    }
                    let r = 0.5 * (a + b);
                    let Some(s) = ecra(
                        &mesh,
                        [centro[0] + (r * dx) as f32, centro[1] + (r * dy) as f32],
                    ) else {
                        fora = true;
                        continue;
                    };
                    let rr = (s[0] - c0[0]).hypot(s[1] - c0[1]);
                    lo = lo.min(rr);
                    hi = hi.max(rr);
                }
                linha.push_str(&format!("   {:7.3} ", hi / lo));
            }
            println!("{linha}{}", if fora { "  (tocou fora da arte)" } else { "" });
        }
    }
}
