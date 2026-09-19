//! ⛔⛔⛔⛔ **A RAZÃO É A GRANDEZA ERRADA QUANDO O PINCEL É GRANDE — medido no
//! report de 2026-09-19** (*«ainda não ficou bom»* + *«funciona para tamanho
//! menor do pincel»*).
//!
//! # O que a reprodução deu
//!
//! Na barbatana da cena `=50`, traço de 6 dabs a força `1,00`, as costas movem-se
//! **isto** em fracção do que a frente moveu:
//!
//! | `R` | `d = 0,10` | `d = 0,25` | `d = 0,50` |
//! |---|---|---|---|
//! | `0,20` | `79,4 %` | **`1,3 %`** | **`0,0 %`** |
//! | `0,40` | `106,1 %` | `113,1 %` | `145,1 %` |
//! | `0,65` (o da 2.ª foto) | `104,6 %` | `103,2 %` | `104,3 %` |
//! | `0,90` | `103,6 %` | `103,8 %` | `104,2 %` |
//!
//! ⇒ *a lei da razão cura o pincel pequeno e é INERTE no grande* — que é à letra
//! o que o dono escreveu, achado por ele sem ver a tabela.
//!
//! # ⭐⭐⭐⭐ E o mecanismo é aritmético, não uma afinação
//!
//! Para um ponto de trás a `L` de lado do cursor, com o cursor a `d` da beira:
//! o **ar** mede `√(L² + t²)` e a **superfície** mede `2d + t + L` ⇒
//!
//! > **com `L ≫ t` a razão tende para `1`**, que é o mesmo valor que um ponto da
//! > FRENTE a `L` de lado tem.
//!
//! Medido (`R = 0,65`, `d = 0,25`, um dab), a razão dos vértices de TRÁS que
//! passam cai de `2,83` a `1,82` conforme o deslocamento lateral cresce — **todos
//! abaixo do `RAZAO_MAXIMA = 3,5`**, e todos legitimamente: a superfície de facto
//! os alcança. ⇒ ⛔ **nenhum valor de `RAZAO_MAXIMA` separa os dois lados**, e
//! apertá-lo comeria a frente. *A razão respondia «a superfície alcança?»; a
//! pergunta do artista é «é a folha que eu estou a ver?».*
//!
//! ⚠️ **E o rasgo das fotos NÃO é geometria virada:** `0` faces invertidas em
//! todas as células medidas. Com a frente a subir `0,33` e as costas a subir
//! `0,34` numa chapa de `0,06`, **as duas folhas atravessam-se** — é isso que a
//! foto mostra.
//!
//! # A candidata que esta sonda mede
//!
//! A **NORMAL**: um vértice cuja normal aponta para longe do olho não é da folha
//! que o artista vê. ⭐ Ela é imune ao tamanho do pincel **por construção** — as
//! costas de uma chapa apontam ao contrário da frente, esteja o cursor onde
//! estiver —, e o `Dab` já carrega o [`Dab::eye`].
//!
//! ⚠️ **As duas colunas têm de ser medidas JUNTAS:** o GANHO na barbatana não
//! vale nada se o RISCO comer as peças que o corpus aprova.

use ph2d_mesh::{Face, Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

const ESPESSURA: f32 = 0.06;
const MEIO: f32 = 1.0;
const N: usize = 81;

fn d3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn f(i: usize, j: usize) -> u32 {
    (i * N + j) as u32
}
fn t(i: usize, j: usize) -> u32 {
    (N * N + i * N + j) as u32
}

/// A MESMA barbatana da cena `=50`.
fn barbatana() -> Mesh {
    let passo = 2.0 * MEIO / (N - 1) as f32;
    let meia = ESPESSURA * 0.5;
    let mut pos = Vec::with_capacity(2 * N * N);
    for z in [meia, -meia] {
        for i in 0..N {
            for j in 0..N {
                pos.push([-MEIO + passo * i as f32, -MEIO + passo * j as f32, z]);
            }
        }
    }
    let mut faces = Vec::with_capacity(2 * (N - 1) * (N - 1) + 4 * (N - 1));
    for i in 0..N - 1 {
        for j in 0..N - 1 {
            faces.push(Face::quad(
                f(i, j),
                f(i + 1, j),
                f(i + 1, j + 1),
                f(i, j + 1),
            ));
            faces.push(Face::quad(
                t(i, j),
                t(i, j + 1),
                t(i + 1, j + 1),
                t(i + 1, j),
            ));
        }
    }
    for k in 0..N - 1 {
        faces.push(Face::quad(f(k + 1, 0), f(k, 0), t(k, 0), t(k + 1, 0)));
        faces.push(Face::quad(
            f(k, N - 1),
            f(k + 1, N - 1),
            t(k + 1, N - 1),
            t(k, N - 1),
        ));
        faces.push(Face::quad(f(0, k), f(0, k + 1), t(0, k + 1), t(0, k)));
        faces.push(Face::quad(
            f(N - 1, k + 1),
            f(N - 1, k),
            t(N - 1, k),
            t(N - 1, k + 1),
        ));
    }
    Mesh::from_parts(pos, faces).expect("a barbatana é construída aqui e é válida")
}

/// Um dab pelo caminho do produto, com a máscara armada.
fn um_dab(repouso: &Mesh, centro: [f32; 3], olho: [f32; 3], raio: f32) -> Mesh {
    let mut m = repouso.clone();
    let b = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 1.0,
        surface_only: true,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    st.dab(
        &mut m,
        &b,
        &Dab::at(centro, raio, olho),
        Symmetry::default(),
    );
    m
}

/// `n · olho` no REPOUSO. ⚠️ **Tem de ser o repouso** — a normal viva inclina-se
/// sobre o barro que o próprio traço levantou, e esta linha já pagou três vezes
/// por ler uma grandeza do vivo onde a lei é do pen-down.
fn viradas_ao_olho(repouso: &Mesh, olho: [f32; 3]) -> Vec<f32> {
    repouso
        .normals()
        .iter()
        .map(|n| n[0] * olho[0] + n[1] * olho[1] + n[2] * olho[2])
        .collect()
}

/// ⭐⭐⭐ **O GANHO** — na barbatana, quanto do que ainda passa é apanhado pela
/// normal, por raio de pincel.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_a_normal_apanha_o_que_a_razao_deixa() {
    let repouso = barbatana();
    let olho = [0.0, 0.0, -1.0];
    let dot = viradas_ao_olho(&repouso, olho);
    let meia = ESPESSURA * 0.5;

    println!("\n== O GANHO: a barbatana, UM dab, forca 1,00 ==");
    println!("   `dot` = n·olho no repouso; a frente tem dot = -1, as costas +1\n");
    println!(
        "{:>6} {:>6} | {:>9} {:>9} | {:>10} {:>10} {:>8}",
        "R", "d", "frente", "costas", "n_costas", "apanhados", "%"
    );
    println!("{}", "-".repeat(70));

    for raio in [0.20f32, 0.40, 0.65, 0.90] {
        for dist in [0.10f32, 0.25, 0.50] {
            let centro = [MEIO - dist, 0.0, meia];
            let m = um_dab(&repouso, centro, olho, raio);
            let (mut mf, mut mc, mut nc, mut apanhados) = (0.0f32, 0.0f32, 0usize, 0usize);
            for (i, (&r, &q)) in repouso.positions().iter().zip(m.positions()).enumerate() {
                let dd = d3(r, q);
                if dd <= 1e-6 {
                    continue;
                }
                let costas = (r[2] + meia).abs() < 1e-5;
                if costas {
                    mc = mc.max(dd);
                    nc += 1;
                    if dot[i] > 0.0 {
                        apanhados += 1;
                    }
                } else {
                    mf = mf.max(dd);
                }
            }
            let pct = if nc == 0 {
                100.0
            } else {
                100.0 * apanhados as f32 / nc as f32
            };
            println!(
                "{raio:>6.2} {dist:>6.2} | {mf:>9.4} {mc:>9.4} | {nc:>10} {apanhados:>10} {pct:>7.1}%"
            );
        }
    }
}

/// ⛔⛔⛔ **O RISCO** — nas peças que o corpus APROVA, quantos vértices que a lei
/// de hoje mantém é que a normal cortaria.
///
/// ⚠️⚠️ *Uma cura medida só do lado do defeito é metade de uma medição* — esta
/// linha retirou duas barras do gate do tecido por elas reprovarem a saída do
/// PRÓPRIO alvo, e a régua aqui é a mesma: o número que interessa é quanto a
/// candidata come de quem já estava bem.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_o_que_a_normal_comeria_nas_pecas_aprovadas() {
    println!("\n== O RISCO: o que a NORMAL cortaria de quem a lei de hoje mantem ==");
    println!("   um dab no polo, olho = -z, forca 1,00\n");
    println!(
        "{:>22} {:>6} | {:>8} {:>9} | {:>9} {:>9}",
        "peca", "R", "movidos", "cortados", "%", "dot max"
    );
    println!("{}", "-".repeat(76));

    let pecas: Vec<(&str, Mesh)> = vec![
        ("esfera lisa", shapes::uv_sphere(64, 128, 1.0)),
        ("sculpt_sphere", shapes::sculpt_sphere(1.0)),
        ("tubo (torus 0,10)", shapes::torus(96, 48, 1.0, 0.10)),
        (
            "rugosa amp 0,08",
            shapes::uv_sphere_noisy(48, 96, 1.0, 0.08),
        ),
        (
            "rugosa amp 0,16",
            shapes::uv_sphere_noisy(48, 96, 1.0, 0.16),
        ),
        (
            "rugosa amp 0,24",
            shapes::uv_sphere_noisy(48, 96, 1.0, 0.24),
        ),
        ("cratera 0,50", cratera_de(0.50)),
        ("cilindro", shapes::cylinder(64, 1.0, 2.0)),
    ];

    let olho = [0.0, 0.0, -1.0];
    for (nome, repouso) in &pecas {
        let dot = viradas_ao_olho(repouso, olho);
        // O ponto mais perto do olho — é onde o raio do pick aterra.
        let alvo = repouso
            .positions()
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a[2].partial_cmp(&b[2]).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0usize, |(i, _)| i);
        let centro = repouso.positions()[alvo];
        for raio in [0.20f32, 0.40, 0.65] {
            let m = um_dab(repouso, centro, olho, raio);
            let (mut mov, mut cort, mut dmax) = (0usize, 0usize, f32::NEG_INFINITY);
            for (i, (&r, &q)) in repouso.positions().iter().zip(m.positions()).enumerate() {
                if d3(r, q) <= 1e-6 {
                    continue;
                }
                mov += 1;
                dmax = dmax.max(dot[i]);
                if dot[i] > 0.0 {
                    cort += 1;
                }
            }
            let pct = if mov == 0 {
                0.0
            } else {
                100.0 * cort as f32 / mov as f32
            };
            println!(
                "{nome:>22} {raio:>6.2} | {mov:>8} {cort:>9} | {pct:>8.2}% {:>9.3}",
                if dmax.is_finite() { dmax } else { 0.0 }
            );
        }
    }
}

/// Uma cratera analítica com a profundidade do corpus.
fn cratera_de(profundidade: f32) -> Mesh {
    let base = shapes::uv_sphere(64, 128, 1.0);
    let mut pos = base.positions().to_vec();
    let topo = [0.0f32, 0.0, 1.0];
    for p in &mut pos {
        let d = d3(*p, topo);
        if d < 0.55 {
            let k = 1.0 - (d / 0.55);
            let s = 1.0 - profundidade * k * k;
            p[0] *= s;
            p[1] *= s;
            p[2] *= s;
        }
    }
    Mesh::from_parts(pos, base.faces().to_vec()).expect("a cratera é derivada de uma esfera válida")
}

/// Uma **CASCA CURVA FINA** — a peça que faltava ao corpus: uma orelha, não uma
/// chapa.
///
/// ⚠️⚠️ **Numa chapa a normal das costas é `+1` EXACTO**, logo qualquer barra
/// abaixo de `1` a apanha e o número não diz nada sobre a barra. Numa casca
/// curva as costas apontam para todo o lado, e é ELA que decide quanto de folga
/// a barra pode ter. *Uma fixtura plana não mede um limiar de normal.*
fn casca_curva(raio: f32, espessura: f32, aberta_ate: f32) -> Mesh {
    let (aneis, segs) = (40usize, 80usize);
    let mut pos = Vec::new();
    for lado in 0..2 {
        let r = if lado == 0 { raio } else { raio - espessura };
        for i in 0..=aneis {
            let th = aberta_ate * i as f32 / aneis as f32;
            for j in 0..segs {
                let ph = std::f32::consts::TAU * j as f32 / segs as f32;
                pos.push([
                    r * th.sin() * ph.cos(),
                    r * th.sin() * ph.sin(),
                    r * th.cos(),
                ]);
            }
        }
    }
    let base = (aneis + 1) * segs;
    let id = |lado: usize, i: usize, j: usize| (lado * base + i * segs + j % segs) as u32;
    let mut faces = Vec::new();
    for i in 0..aneis {
        for j in 0..segs {
            faces.push(Face::quad(
                id(0, i, j),
                id(0, i + 1, j),
                id(0, i + 1, j + 1),
                id(0, i, j + 1),
            ));
            faces.push(Face::quad(
                id(1, i, j),
                id(1, i, j + 1),
                id(1, i + 1, j + 1),
                id(1, i + 1, j),
            ));
        }
    }
    for j in 0..segs {
        faces.push(Face::quad(
            id(0, aneis, j),
            id(0, aneis, j + 1),
            id(1, aneis, j + 1),
            id(1, aneis, j),
        ));
    }
    Mesh::from_parts(pos, faces).expect("a casca é construída aqui e é válida")
}

/// ⭐⭐⭐⭐ **O VALE** — a barra do limiar sai do vazio entre os dois lados, e os
/// DOIS têm de estar na tabela.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_o_vale_do_limiar_da_normal() {
    let olho = [0.0f32, 0.0, -1.0];
    let barras = [0.0f32, 0.10, 0.20, 0.30, 0.45, 0.60, 0.80];

    println!("\n== O VALE: % do que a lei de hoje MANTEM que a normal cortaria ==");
    println!("   um dab, forca 1,00 · a coluna e' `dot > barra`\n");
    print!("{:>26} {:>5} |", "peca (R = 0,65)", "");
    for b in barras {
        print!(" {b:>7.2}");
    }
    println!();
    println!("{}", "-".repeat(90));

    // ⭐ O lado que TEM de ser cortado (o defeito) e o que NÃO pode (o aprovado).
    let casos: Vec<(&str, Mesh, bool)> = vec![
        ("DEFEITO barbatana", barbatana(), true),
        ("DEFEITO casca fina", casca_curva(1.0, 0.06, 1.2), true),
        ("APROV. casca grossa", casca_curva(1.0, 0.35, 1.2), false),
        ("APROV. esfera lisa", shapes::uv_sphere(64, 128, 1.0), false),
        ("APROV. sculpt_sphere", shapes::sculpt_sphere(1.0), false),
        ("APROV. cratera 0,50", cratera_de(0.50), false),
        (
            "APROV. rugosa 0,16",
            shapes::uv_sphere_noisy(48, 96, 1.0, 0.16),
            false,
        ),
        (
            "APROV. rugosa 0,24",
            shapes::uv_sphere_noisy(48, 96, 1.0, 0.24),
            false,
        ),
    ];

    for (nome, repouso, e_defeito) in &casos {
        let dot = viradas_ao_olho(repouso, olho);
        let alvo = repouso
            .positions()
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a[2].partial_cmp(&b[2]).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0usize, |(i, _)| i);
        let mut centro = repouso.positions()[alvo];
        // ⛔⛔ **A FIXTURA TEM DE CONTER O FENÓMENO, e a 1.ª redacção não continha:**
        // com o dab no POLO de uma casca, a beira fica a ~`2,4` de superfície e o
        // tecto absoluto (`2,00 × R = 1,30`) já corta tudo ⇒ as duas cascas liam
        // `0,0 %` e a coluna que decide a barra media o nada. *A terceira vez que
        // esta wave paga a mesma armadilha.* ⇒ nas cascas o dab vai para junto da
        // BEIRA, que é onde o artista de uma orelha carimba.
        if nome.contains("casca") {
            centro = repouso
                .positions()
                .iter()
                .copied()
                .filter(|p| p[1].abs() < 0.05 && p[0] > 0.0)
                .max_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(centro);
            // recua um pouco da beira, para dentro da folha da frente
            centro = [centro[0] * 0.93, centro[1], centro[2] + 0.12];
        }
        if *e_defeito && nome.contains("barbatana") {
            centro = [MEIO - 0.25, 0.0, ESPESSURA * 0.5];
        }
        let m = um_dab(repouso, centro, olho, 0.65);
        let movidos: Vec<usize> = (0..repouso.positions().len())
            .filter(|&i| d3(repouso.positions()[i], m.positions()[i]) > 1e-6)
            .collect();
        print!("{nome:>26} {:>5} |", movidos.len());
        for b in barras {
            let c = movidos.iter().filter(|&&i| dot[i] > b).count();
            let pct = if movidos.is_empty() {
                0.0
            } else {
                100.0 * c as f32 / movidos.len() as f32
            };
            print!(" {pct:>6.1}%");
        }
        println!();
    }
    println!("\n   ⇒ a barra vive onde as linhas DEFEITO ainda cortam e as APROV. leem 0,0%");
}

/// ⭐⭐⭐⭐ **A PERGUNTA DO §5.0: a COMPOSIÇÃO já exprime isto?**
///
/// O app já tem *Front Faces Only* — bandeira de pincel, rótulo público da
/// referência, lei **contínua** `fator × max(n · olho, 0)`. Numa chapa a normal
/// das costas dá `max(+1, 0)`… ⚠️ **com o sinal do `olho` a decidir**: o `olho`
/// vai do olho PARA a superfície, logo a frente lê `n·olho = −1` e o factor da
/// referência é `max(−n·olho, 0)`.
///
/// ⇒ **se ela sozinha já zera as costas, não há lei nova a escrever** — há uma
/// caixa a encontrar, e isso é decisão de produto e não de engenharia.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_a_caixa_que_ja_existe_cura_sozinha() {
    let repouso = barbatana();
    let olho = [0.0f32, 0.0, -1.0];
    let meia = ESPESSURA * 0.5;

    println!("\n== A COMPOSICAO: `Front Faces Only` sozinho, sem lei nova ==");
    println!("   barbatana · UM dab · forca 1,00 · Connected Only LIGADO\n");
    println!(
        "{:>6} {:>6} | {:>10} {:>9} {:>9} | {:>10} {:>9} {:>9}",
        "R", "d", "ff=OFF f", "ff=OFF c", "%", "ff=ON f", "ff=ON c", "%"
    );
    println!("{}", "-".repeat(82));

    for raio in [0.20f32, 0.40, 0.65, 0.90] {
        for dist in [0.10f32, 0.25, 0.50] {
            let centro = [MEIO - dist, 0.0, meia];
            let mut linha = Vec::new();
            for ff in [false, true] {
                let mut m = repouso.clone();
                let b = Brush {
                    verb: Verb::Draw,
                    radius: raio,
                    strength: 1.0,
                    surface_only: true,
                    front_faces_only: ff,
                    ..Brush::default()
                };
                let mut st = SculptStroke::default();
                st.begin(&m);
                st.dab(
                    &mut m,
                    &b,
                    &Dab::at(centro, raio, olho),
                    Symmetry::default(),
                );
                let (mut mf, mut mc) = (0.0f32, 0.0f32);
                for i in 0..repouso.positions().len() {
                    let dd = d3(repouso.positions()[i], m.positions()[i]);
                    if (repouso.positions()[i][2] + meia).abs() < 1e-5 {
                        mc = mc.max(dd);
                    } else {
                        mf = mf.max(dd);
                    }
                }
                linha.push((mf, mc));
            }
            let p0 = if linha[0].0 > 0.0 {
                100.0 * linha[0].1 / linha[0].0
            } else {
                0.0
            };
            let p1 = if linha[1].0 > 0.0 {
                100.0 * linha[1].1 / linha[1].0
            } else {
                0.0
            };
            println!(
                "{raio:>6.2} {dist:>6.2} | {:>10.4} {:>9.4} {p0:>8.1}% | {:>10.4} {:>9.4} {p1:>8.1}%",
                linha[0].0, linha[0].1, linha[1].0, linha[1].1
            );
        }
    }
}
