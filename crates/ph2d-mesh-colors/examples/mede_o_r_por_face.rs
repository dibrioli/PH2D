//! ⭐⭐⭐ **O QUE A P2 COMPRA, ANTES DE A CONSTRUIR** — a densidade de amostras
//! por área, face a face, com o nível UNIFORME de hoje e com o `R` POR FACE.
//!
//! ⚠️ **Ela corre-se ANTES da primeira linha de produto**, que é a lei do §5.0
//! (*«antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime»*). O handoff de 21/09 escreveu que a dispersão é `3,1×`–`18,3×` e
//! que *«o chão teórico é `√2 = 1,41×`»* — ⛔ **as duas afirmações não são a
//! mesma grandeza**, e esta sonda existe para dizer qual é qual:
//!
//! * a **dispersão** é uma razão entre DUAS faces (`p99/p1`);
//! * o `√2` é o **desvio ao alvo** de UMA face, que é meia escada.
//!
//! *Uma razão entre dois desvios de `√2` vale `2`.* Se a sonda ler `2` e não
//! `1,41`, o chão está escrito errado no handoff e é o handoff que muda.
//!
//! ```text
//! gunzip -c crates/ph2d-quadfill/tests/fixtures/pontas/sculpt_antes.obj.gz > /tmp/x.obj
//! <o corredor> run -p ph2d-mesh-colors --release --example mede_o_r_por_face -- /tmp/x.obj
//! ```
//!
//! ⛔ Ela lê o `.obj` **por argumento** e não de uma fixtura: esta crate é de
//! ZERO dependências e não vai ganhar um leitor de malhas para uma medição.

use std::collections::BTreeMap;

/// Uma malha crua: posições e faces (3 ou 4 cantos).
struct Malha {
    pos: Vec<[f64; 3]>,
    faces: Vec<Vec<u32>>,
}

/// ⚠️ Um parser MÍNIMO de propósito: ele lê `v` e `f` e mais nada. Um `.obj` com
/// grupos, materiais ou normais passa — os campos depois da `/` são deitados
/// fora, que é o que uma medição de ÁREA precisa.
fn le_obj(txt: &str) -> Malha {
    let mut pos = Vec::new();
    let mut faces = Vec::new();
    for l in txt.lines() {
        let mut it = l.split_ascii_whitespace();
        match it.next() {
            Some("v") => {
                let c: Vec<f64> = it.take(3).filter_map(|s| s.parse().ok()).collect();
                if c.len() == 3 {
                    pos.push([c[0], c[1], c[2]]);
                }
            }
            Some("f") => {
                let f: Vec<u32> = it
                    .filter_map(|s| s.split('/').next())
                    .filter_map(|s| s.parse::<i64>().ok())
                    // ⚠️ o `.obj` conta de 1, e um índice NEGATIVO conta do fim.
                    .map(|i| {
                        if i > 0 {
                            u32::try_from(i - 1).unwrap_or(0)
                        } else {
                            u32::try_from(pos.len() as i64 + i).unwrap_or(0)
                        }
                    })
                    .collect();
                if f.len() == 3 || f.len() == 4 {
                    faces.push(f);
                }
            }
            _ => {}
        }
    }
    Malha { pos, faces }
}

fn area(m: &Malha, f: &[u32]) -> f64 {
    let p = |i: usize| m.pos[f[i] as usize];
    let tri = |a: [f64; 3], b: [f64; 3], c: [f64; 3]| {
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
    };
    if f.len() == 3 {
        tri(p(0), p(1), p(2))
    } else {
        tri(p(0), p(1), p(2)) + tri(p(0), p(2), p(3))
    }
}

/// O quantil `q` de uma lista JÁ ordenada.
fn quantil(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    v[i]
}

/// As arestas da malha, cada uma com as faces que a tocam.
fn arestas(m: &Malha) -> BTreeMap<(u32, u32), Vec<usize>> {
    let mut a: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (fi, f) in m.faces.iter().enumerate() {
        for s in 0..f.len() {
            let (u, v) = (f[s], f[(s + 1) % f.len()]);
            a.entry((u.min(v), u.max(v))).or_default().push(fi);
        }
    }
    a
}

/// Quantas amostras o plano tem com um nível POR FACE.
///
/// ⚠️ A aresta leva o **MÁXIMO** dos vizinhos — é a lei que a fronteira
/// partilhada obriga, e é ela que faz a face grossa ler um SUBCONJUNTO exacto.
fn amostras(m: &Malha, k: &[u8], ars: &BTreeMap<(u32, u32), Vec<usize>>) -> u64 {
    let mut n = m.pos.len() as u64;
    for fs in ars.values() {
        let le = 1u64 << fs.iter().map(|&f| k[f]).max().unwrap_or(0);
        n += le - 1;
    }
    for (fi, f) in m.faces.iter().enumerate() {
        let l = 1u64 << k[fi];
        n += if f.len() == 3 {
            l.saturating_sub(1) * l.saturating_sub(2) / 2
        } else {
            l.saturating_sub(1) * l.saturating_sub(1)
        };
    }
    n
}

/// A densidade LINEAR de amostras de uma face: `lado / √área`.
fn densidades(m: &Malha, k: &[u8]) -> Vec<f64> {
    let mut d: Vec<f64> = m
        .faces
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let a = area(m, f);
            if a <= 0.0 {
                return f64::NAN;
            }
            f64::from(1u32 << k[i]) / a.sqrt()
        })
        .filter(|x| x.is_finite())
        .collect();
    d.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN nesta lista"));
    d
}

fn linha(rotulo: &str, d: &[f64], n: u64) {
    let (p1, p50, p99) = (quantil(d, 0.01), quantil(d, 0.5), quantil(d, 0.99));
    println!(
        "  {rotulo:<26} p1={p1:>9.1}  p50={p50:>9.1}  p99={p99:>9.1}  \
         dispersao={:>6.2}x  amostras={n:>12}",
        p99 / p1,
    );
}

/// O nível de cada face para um ALVO de densidade linear, quantizado à escada.
///
/// ⚠️ `tecto_de_salto` é a cerca que a fronteira partilhada pede: a face grossa
/// lê um SUBCONJUNTO da aresta fina, logo um salto grande é detalhe que o lado
/// grosso não consegue mostrar. `None` = sem cerca, que é o controlo.
fn niveis(m: &Malha, alvo: f64, tecto_de_salto: Option<u8>) -> Vec<u8> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut k: Vec<u8> = m
        .faces
        .iter()
        .map(|f| {
            let a = area(m, f).max(1e-30);
            (alvo * a.sqrt())
                .log2()
                .round()
                .clamp(0.0, f64::from(ph2d_mesh_colors::NIVEL_MAX)) as u8
        })
        .collect();
    let Some(tecto) = tecto_de_salto else {
        return k;
    };
    // ⭐ A cerca SOBE o vizinho grosso, nunca desce o fino: descer apagaria
    //   detalhe que o artista pediu, e a conta de memória fica no relatório.
    //   Corre até ao ponto fixo — subir um vizinho pode obrigar o seguinte.
    let vizinhos = arestas(m);
    loop {
        let mut mexeu = false;
        for fs in vizinhos.values() {
            let Some(&hi) = fs.iter().map(|&f| &k[f]).max() else {
                continue;
            };
            for &f in fs {
                if hi - k[f] > tecto {
                    k[f] = hi - tecto;
                    mexeu = true;
                }
            }
        }
        if !mexeu {
            return k;
        }
    }
}

fn salto(m: &Malha, k: &[u8]) -> (u8, BTreeMap<u8, usize>) {
    let mut pior = 0u8;
    let mut hist: BTreeMap<u8, usize> = BTreeMap::new();
    for fs in arestas(m).values() {
        if fs.len() < 2 {
            continue;
        }
        let lo = fs.iter().map(|&f| k[f]).min().unwrap_or(0);
        let hi = fs.iter().map(|&f| k[f]).max().unwrap_or(0);
        pior = pior.max(hi - lo);
        *hist.entry(hi - lo).or_default() += 1;
    }
    (pior, hist)
}

fn main() {
    for arg in std::env::args().skip(1) {
        let txt = std::fs::read_to_string(&arg).expect("o .obj");
        let m = le_obj(&txt);
        let ars = arestas(&m);
        let nome = arg.rsplit('/').next().unwrap_or(&arg).to_string();
        println!(
            "\n== {nome} — {} V, {} F, {} A ==",
            m.pos.len(),
            m.faces.len(),
            ars.len()
        );

        for k_ref in [2u8, 3] {
            // ---- hoje: UM nível para a peça toda ----
            let uniforme = vec![k_ref; m.faces.len()];
            let d0 = densidades(&m, &uniforme);
            linha(
                &format!("uniforme k={k_ref} (lado {})", 1u32 << k_ref),
                &d0,
                amostras(&m, &uniforme, &ars),
            );

            // ---- a P2: o nível POR FACE, nas DUAS leituras do knob ----
            //
            // ⭐⭐⭐ **As duas leituras não são a mesma pergunta, e o preço é
            //   OPOSTO.** Um artista que carrega em `8x` pode querer dizer:
            //
            //   * **MEDIANA** — *«a face típica fica a `8x` e as outras
            //     igualam-se a ela»*. O alvo sai da mediana da peça ao `k`
            //     uniforme ⇒ comparação a orçamento parecido; ela baixa a
            //     dispersão e SOBE a contagem de amostras.
            //   * **TECTO** — *«nenhuma face passa de `8x`»*. O alvo é escalado
            //     para que o nível MÁXIMO seja exactamente `k` ⇒ ela só
            //     engrossa as faces grandes e DESCE a contagem.
            //
            //   ⚠️ A escala é EXACTA e não uma busca: a quantização é
            //   `round(log2(alvo·√a))`, logo multiplicar o alvo por `2^d`
            //   desloca **todos** os níveis por exactamente `d`.
            let alvo = quantil(&d0, 0.5);
            for tecto in [None, Some(1u8)] {
                let k = niveis(&m, alvo, tecto);
                let rot = match tecto {
                    None => format!("mediana k={k_ref}, sem cerca"),
                    Some(t) => format!("mediana k={k_ref}, salto<={t}"),
                };
                linha(&rot, &densidades(&m, &k), amostras(&m, &k, &ars));
                let (pior, hist) = salto(&m, &k);
                println!("      salto entre vizinhos: max={pior}  {hist:?}");
            }
            let maior = niveis(&m, alvo, None)
                .iter()
                .copied()
                .max()
                .unwrap_or(k_ref);
            let alvo_tecto = alvo * 2f64.powi(i32::from(k_ref) - i32::from(maior));
            let k_tecto = niveis(&m, alvo_tecto, Some(1));
            linha(
                &format!("TECTO  k={k_ref}, salto<=1"),
                &densidades(&m, &k_tecto),
                amostras(&m, &k_tecto, &ars),
            );
            let (pior, hist) = salto(&m, &k_tecto);
            println!(
                "      salto entre vizinhos: max={pior}  {hist:?}  nivel {}..{}",
                k_tecto.iter().copied().min().unwrap_or(0),
                k_tecto.iter().copied().max().unwrap_or(0)
            );

            // ⭐⭐⭐⭐ **A QUARTA leitura, e é a que o report de 23/09 obriga a
            //   medir: o `k` como PISO.** O dono reprovou a MEDIANA com a frase
            //   *«a resolução fica bem baixa»* — e medida na peça que ele smoka
            //   ela só sabe DESCER (`0,83×` das amostras, e nenhuma face acima
            //   de `k`). ⇒ aqui o alvo sai da face MAIS PEQUENA, logo
            //   `ideal = k + ½·log2(a/a_min) ≥ k` para toda face: *ninguém fica
            //   mais grosso do que o degrau que o artista pediu*.
            //
            //   ⚠️ E a variante BARATA dela: a mediana com um PISO em `k`, que
            //   nunca desce mas só sobe quem está acima da mediana.
            let a_min = m
                .faces
                .iter()
                .map(|f| area(&m, f))
                .filter(|a| *a > 0.0)
                .fold(f64::MAX, f64::min);
            let alvo_piso = f64::from(1u32 << k_ref) / a_min.sqrt();
            for (nome, ks) in [
                ("PISO   ", niveis(&m, alvo_piso, Some(1))),
                (
                    "MED+PISO",
                    niveis(&m, alvo, Some(1))
                        .iter()
                        .map(|x| (*x).max(k_ref))
                        .collect::<Vec<u8>>(),
                ),
            ] {
                linha(
                    &format!("{nome} k={k_ref}, salto<=1"),
                    &densidades(&m, &ks),
                    amostras(&m, &ks, &ars),
                );
                println!(
                    "      nivel {}..{}",
                    ks.iter().copied().min().unwrap_or(0),
                    ks.iter().copied().max().unwrap_or(0)
                );
            }

            // ---- e o que o EMPACOTADOR faz com os dois, no FICHEIRO ----
            //
            // ⚠️ É esta a coluna que o artista vê: o lado da textura e a
            //    fracção dela que carrega uma amostra de verdade.
            let faces_it = || m.faces.iter().map(Vec::as_slice);
            let k = niveis(&m, alvo, Some(1));
            let assa =
                |t: &ph2d_mesh_colors::Tinta| match ph2d_mesh_colors::assar(t, faces_it(), 16384) {
                    Ok(a) => format!(
                        "{}x{} px, aproveitamento {:.1} %",
                        a.lado_px,
                        a.lado_px,
                        a.relatorio.aproveitamento() * 100.0
                    ),
                    Err(e) => format!("RECUSA {e}"),
                };
            let uni = ph2d_mesh_colors::Tinta::nova(m.pos.len(), faces_it(), k_ref);
            println!("      textura uniforme : {}", assa(&uni));
            for (rot, ks) in [("mediana", &k), ("TECTO  ", &k_tecto)] {
                if let Some(g) = ph2d_mesh_colors::Tinta::graduada(
                    m.pos.len(),
                    faces_it(),
                    ks,
                    ks.iter().copied().min().unwrap_or(0),
                ) {
                    println!("      textura {rot} : {}", assa(&g));
                }
            }
        }
    }
}
