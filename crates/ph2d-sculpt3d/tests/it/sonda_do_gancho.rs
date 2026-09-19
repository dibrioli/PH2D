//! ⛔⛔⛔ **A REPRODUÇÃO DO REPORT DE 2026-09-19** — *«Snake Hook se dá muito mal
//! com `Connected Only`, deformando com má remesh ou má topologia a face
//! POSTERIOR do traço»* (foto: a bossa puxada com as costas pretas, rasgadas e
//! com um refino explodido).
//!
//! # A pergunta, e porque ela se responde SEM topologia dinâmica
//!
//! A foto tem duas coisas ao mesmo tempo — um **rasgo** e um **refino
//! explodido** — e elas não são independentes: o passe de topologia parte
//! arestas **longas**, logo um rasgo a montante fabrica o refino a jusante.
//! ⇒ a pergunta que decide é a de montante, e ela é de GEOMETRIA: *com a máscara
//! ligada, o barro das costas do traço fica para trás enquanto o vizinho dele
//! viaja?*
//!
//! ⚠️ **A régua é o RASGO ENTRE VIZINHOS** — a maior diferença de deslocamento
//! entre dois vértices ligados por uma aresta —, que é a mesma que o §29 desta
//! linha usou para o `Scene Project` (`0,6066 → 0,0793`). *Um extremo global de
//! deslocamento não vê um rasgo: ele vê o puxão, que é suposto ser grande.*

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// A peça da cena: a esfera de escultura, que é o que a foto mostra.
fn esfera() -> Mesh {
    ph2d_mesh::shapes::sculpt_sphere(1.0)
}

/// O olho da cena (a câmara olha no `−z`), e o gesto é o da foto: o cursor
/// **arrasta de lado** enquanto o gancho puxa.
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];

fn norma(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Um arrasto de gancho de `dabs` passos, `len` de comprimento no `+x`, a partir
/// do polo virado ao artista.
fn gancho(raio: f32, len: f32, dabs: usize, mascara: bool) -> (Mesh, Vec<[f32; 3]>) {
    let mut m = esfera();
    let antes: Vec<[f32; 3]> = m.positions().to_vec();
    let b = Brush {
        verb: Verb::SnakeHook,
        radius: raio,
        strength: 1.0,
        surface_only: mascara,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    let passo = len / dabs as f32;
    let mut centro = [0.0f32, 0.0, 1.0];
    for _ in 0..dabs {
        centro[0] += passo;
        st.dab(
            &mut m,
            &b,
            &Dab::hooking(centro, raio, OLHO, [passo, 0.0, 0.0]),
            Symmetry::default(),
        );
    }
    (m, antes)
}

/// O que a foto mostra, em números.
struct Leitura {
    movidos: usize,
    /// O maior deslocamento — o puxão.
    puxao: f32,
    /// ⭐ **O RASGO:** a maior diferença de deslocamento entre dois vértices que
    /// partilham uma aresta, em unidades da aresta média da peça de repouso.
    rasgo: f32,
    /// Faces cuja normal passou a apontar ao CONTRÁRIO da vizinhança — o preto
    /// da foto.
    avesso: usize,
    /// A maior aresta depois do traço, em unidades da aresta média de repouso —
    /// é ela que o passe de topologia parte, e é daí que vem o refino.
    estica: f32,
}

fn mede(m: &Mesh, antes: &[[f32; 3]]) -> Leitura {
    let pos = m.positions();
    let desl: Vec<f32> = pos
        .iter()
        .zip(antes)
        .map(|(p, a)| norma([p[0] - a[0], p[1] - a[1], p[2] - a[2]]))
        .collect();
    let movidos = desl.iter().filter(|&&d| d > 1e-6).count();
    let puxao = desl.iter().cloned().fold(0.0f32, f32::max);

    // A aresta média da peça de REPOUSO — a unidade de tudo o que se segue.
    let viz = &m.adjacency().vert_verts;
    let (mut soma, mut n) = (0.0f64, 0usize);
    for u in 0..antes.len() {
        for &v in viz.neighbours(u) {
            let a = antes[u];
            let b = antes[v as usize];
            soma += f64::from(norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]));
            n += 1;
        }
    }
    let unidade = if n == 0 {
        1.0
    } else {
        (soma / n as f64) as f32
    };

    let mut rasgo = 0.0f32;
    let mut estica = 0.0f32;
    for u in 0..pos.len() {
        for &v in viz.neighbours(u) {
            let vi = v as usize;
            if vi >= antes.len() || u >= antes.len() {
                continue;
            }
            if desl[u] > 1e-6 || desl[vi] > 1e-6 {
                rasgo = rasgo.max((desl[u] - desl[vi]).abs());
            }
            let (a, b) = (pos[u], pos[vi]);
            estica = estica.max(norma([a[0] - b[0], a[1] - b[1], a[2] - b[2]]));
        }
    }

    // ⚠️ **Do AVESSO mede-se contra a vizinhança e não contra o repouso:** um
    // gancho vira as faces da bossa de propósito, e o que o olho lê como preto é
    // a face que discorda de QUEM ESTÁ AO LADO dela.
    let nrm = m.normals();
    let mut avesso = 0usize;
    for f in m.faces() {
        let vs = f.verts();
        if vs.len() < 3 {
            continue;
        }
        let (mut acc, mut k) = ([0.0f32; 3], 0.0f32);
        for &v in vs {
            for &w in viz.neighbours(v as usize) {
                let n = nrm[w as usize];
                acc = [acc[0] + n[0], acc[1] + n[1], acc[2] + n[2]];
                k += 1.0;
            }
        }
        if k == 0.0 {
            continue;
        }
        let media = [acc[0] / k, acc[1] / k, acc[2] / k];
        let lm = norma(media);
        if lm < 1e-6 {
            continue;
        }
        let fa = nrm[vs[0] as usize];
        let dot = (fa[0] * media[0] + fa[1] * media[1] + fa[2] * media[2]) / lm;
        if dot < -0.2 {
            avesso += 1;
        }
    }

    Leitura {
        movidos,
        puxao,
        rasgo: rasgo / unidade,
        avesso,
        estica: estica / unidade,
    }
}

/// ⭐⭐⭐⭐ **A TABELA DO REPORT** — o mesmo gancho, com e sem `Connected Only`.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-sculpt3d --test it \
///   diag_o_gancho_contra_a_mascara -- --ignored --nocapture --test-threads=1
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do report, nao afirma nada"]
fn diag_o_gancho_contra_a_mascara() {
    println!("\n== O GANCHO CONTRA A MASCARA (report de 19/09) ==");
    println!("   esfera de escultura · olho em -z · arrasto no +x\n");
    println!(
        "{:>6} {:>6} {:>4} | {:>8} {:>8} {:>8} {:>8} {:>7}",
        "raio", "len", "mask", "movidos", "puxao", "rasgo", "estica", "avesso"
    );
    println!("{}", "-".repeat(74));
    for raio in [0.25f32, 0.35, 0.50] {
        for len in [0.3f32, 0.6, 0.9] {
            for mascara in [false, true] {
                let (m, antes) = gancho(raio, len, 12, mascara);
                let l = mede(&m, &antes);
                println!(
                    "{raio:>6.2} {len:>6.2} {:>4} | {:>8} {:>8.4} {:>8.3} {:>8.3} {:>7}",
                    if mascara { "ON" } else { "off" },
                    l.movidos,
                    l.puxao,
                    l.rasgo,
                    l.estica,
                    l.avesso
                );
            }
        }
    }
    println!(
        "\n   rasgo e estica em unidades da ARESTA MEDIA de repouso.\n   \
         O refino do passe de topologia parte quem estica — logo a coluna\n   \
         `estica` e' a que explica o remesh da foto."
    );
}

/// ⭐⭐⭐ **ONDE o corte cai, dab a dab** — a pegada que o olho aprova contra a
/// que a superfície liga, ao longo do arrasto.
///
/// ⚠️ Ela não mede a máscara por dentro (isso é `pub(crate)`): mede o que **o
/// barro** faz, que é o que o dono vê — quanto o lado do traço virado ao artista
/// anda contra o lado de trás.
#[test]
#[ignore = "sonda: imprime a assimetria frente/costas, nao afirma nada"]
fn diag_a_frente_e_as_costas_do_traco() {
    let (raio, len) = (0.35f32, 0.9f32);
    println!("\n== A FRENTE E AS COSTAS DO TRACO ==");
    println!(
        "   raio {raio} · len {len} · a peca de repouso decide quem e' frente\n   \
         (normal contra o olho) e quem e' costas\n"
    );
    println!(
        "{:>5} | {:>9} {:>9} | {:>9} {:>9} | {:>8}",
        "mask", "fr movid", "fr p50", "cos movid", "cos p50", "rasgo"
    );
    println!("{}", "-".repeat(64));
    for mascara in [false, true] {
        let (m, antes) = gancho(raio, len, 12, mascara);
        let pos = m.positions();
        let base = esfera();
        let nrm0 = base.normals();
        let (mut fr, mut cos) = (Vec::new(), Vec::new());
        for i in 0..antes.len() {
            let d = norma([
                pos[i][0] - antes[i][0],
                pos[i][1] - antes[i][1],
                pos[i][2] - antes[i][2],
            ]);
            if d <= 1e-6 {
                continue;
            }
            // O olho aponta no `−z`, logo a face virada ao artista tem `n·(−olho) > 0`.
            let n = nrm0[i];
            if -(n[0] * OLHO[0] + n[1] * OLHO[1] + n[2] * OLHO[2]) > 0.0 {
                fr.push(d);
            } else {
                cos.push(d);
            }
        }
        let p50 = |v: &mut Vec<f32>| {
            if v.is_empty() {
                return 0.0;
            }
            v.sort_by(f32::total_cmp);
            v[v.len() / 2]
        };
        let l = mede(&m, &antes);
        let (mut a, mut b) = (fr, cos);
        let (na, nb) = (a.len(), b.len());
        println!(
            "{:>5} | {na:>9} {:>9.4} | {nb:>9} {:>9.4} | {:>8.3}",
            if mascara { "ON" } else { "off" },
            p50(&mut a),
            p50(&mut b),
            l.rasgo
        );
    }
}
