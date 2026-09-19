//! ⭐⭐⭐⭐ **A VISTA DA GRADE** — as réguas que medem o que o artista de facto
//! VÊ quando liga a caixa, e não o que a malha tem.
//!
//! ⚠️⚠️ **A distinção é a lição desta jornada:** a régua da `grade` (a fracção
//! de arestas alinhadas) está **saturada** — o tecto dela é `66,7 %` numa grade
//! perfeita e o produto lê `65,4` —, e o que sobrou por medir era *«depois de
//! esconder as diagonais, quantos cruzamentos têm os QUATRO braços»*. Essa é a
//! grandeza que o olho usa naquela vista, e é a que esta família mede.
//!
//! ⚠️ Ele saiu do irmão por **TECTO DE LOC** (`1 013` contra `700`), e o corte é
//! por responsabilidade: ali fica a lei que arruma a malha, aqui a régua do que
//! se vê dela.

use super::*;

/// ⭐⭐⭐⭐ **SONDA — ONDE a grade é má, em bandas de distância ao percurso.**
///
/// A régua é a [`ph2d_sculpt3d::medida_do_pente::grade_por_banda`], e a
/// hipótese que ela testa é a queda do pincel: o miolo do traço recebe peso
/// cheio e a **orla** quase nada, logo a grade seria boa ao centro e má nas
/// beiras. ⛔ Ela **imprime**; quem decide é quem lê.
#[test]
#[ignore = "sonda: imprime onde a grade e' ma', nao afirma nada"]
fn diag_onde_a_grade_e_ma() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    const B: usize = 5;
    for (nome, knob) in [("por pentear", 0.0f32), ("pente no topo", 1.0)] {
        let mut soma = [(0usize, 0usize); B];
        for (_, e) in RUMOS.iter() {
            let (m, c) = super::super::traco_com(knob, *e, raio, alvo);
            for (i, (bons, n)) in ph2d_sculpt3d::medida_do_pente::grade_por_banda(&m, &c, raio, B)
                .into_iter()
                .enumerate()
            {
                soma[i].0 += bons;
                soma[i].1 += n;
            }
        }
        println!("\n{nome}:  banda (raios)   alinhadas%   arestas");
        for (i, (bons, n)) in soma.iter().enumerate() {
            let lo = i as f64 / B as f64;
            println!(
                "               {lo:4.2}..{:4.2}      {:7.2}   {n:7}",
                lo + 1.0 / B as f64,
                100.0 * *bons as f64 / (*n).max(1) as f64
            );
        }
    }
}

/// ⭐⭐⭐⭐ **SONDA — a REGULARIDADE da vista da grade: quantos cruzamentos têm
/// os QUATRO braços.**
///
/// ⛔⛔ **A régua da `grade` está SATURADA e eu não sabia**: medido numa grade
/// PERFEITA triangulada, o tecto dela é **`66,7 %`** (a diagonal é um terço das
/// arestas e mora a `45°`, no balde mais afastado), e o produto lê `64,3 %` —
/// **`96,4 %` do tecto**. *Ler `64 %` como «dois terços, há muito por ganhar»
/// era ler uma fracção sem saber de que.*
///
/// ⇒ o que sobra é a grandeza que o olho usa **naquela vista**: depois de
/// esconder as diagonais, um cruzamento de grade tem **quatro** braços. Os que
/// não têm são as células irregulares que o dono chama de *«áreas não muito
/// boas»* — e uma grade sobre superfície curva **tem** de ter algumas.
///
/// ⚠️ A lei de esconder é a do produto ([`ph2d_mesh_render::wire_indices_com`]),
/// nunca uma segunda cópia dela aqui.
#[test]
#[ignore = "sonda: imprime a regularidade, nao afirma nada"]
fn diag_a_regularidade_da_grade() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("lei              val3    val4    val5   val6+   4-braços%");
    for (nome, knob) in [("por pentear    ", 0.0f32), ("pente no topo  ", 1.0)] {
        let mut hist = [0usize; 8];
        for (_, e) in RUMOS.iter() {
            let (m, c) = super::super::traco_com(knob, *e, raio, alvo);
            let grau = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
            // ⚠️ Só os do MIOLO da faixa: um vértice na beira do traço tem
            // menos braços por estar na beira, não por ser irregular.
            let pos = m.positions();
            for v in 0..m.vert_count() {
                let q = pos[v];
                let mut d2 = f32::INFINITY;
                for t in &c {
                    let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                    d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
                }
                if d2.sqrt() > raio * 0.5 {
                    continue;
                }
                hist[(grau[v] as usize).min(7)] += 1;
            }
        }
        let n: usize = hist.iter().sum();
        println!(
            "{nome} {:6} {:7} {:7} {:7}   {:7.2}",
            hist[3],
            hist[4],
            hist[5],
            hist[6] + hist[7],
            100.0 * hist[4] as f64 / n.max(1) as f64
        );
    }
}

/// ⭐⭐⭐⭐ **A GRADE TEM DE TER OS QUATRO BRAÇOS — a régua que o olho do dono
/// usa na vista que ele liga, e a única das quatro que NÃO está saturada.**
///
/// ⛔⛔ **A fracção de arestas alinhadas tem tecto `2/3`** numa grade perfeita
/// triangulada (um terço são diagonais, a `45°` — gateado em
/// `o_tecto_da_regua_da_grade_e_dois_tercos`), e o produto lê `65,4 %`, que é
/// **`98,1 %` do tecto**. ⇒ ela não distingue mais nada, e o que o dono chama
/// de *«áreas ainda não muito boas»* é **outra grandeza**: depois de esconder as
/// diagonais, quantos cruzamentos têm os **quatro** braços.
///
/// ⚠️ **As duas metades são obrigatórias, e o CONTROLO é a metade que prova que
/// a régua vê o fenómeno:** por pentear a malha lê `48 %` — *uma régua que
/// lesse alto nos dois lados não estaria a medir o pente*.
///
/// ⚠️ Só o **miolo** da faixa entra: um vértice na beira tem menos braços por
/// estar na beira, não por ser irregular.
#[test]
fn a_grade_tem_os_quatro_bracos() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let medir = |knob: f32| -> (f64, f64) {
        let (mut quatro, mut n, mut fil) = (0usize, 0usize, 0.0f64);
        for (_, e) in RUMOS.iter() {
            let (m, c) = super::super::traco_com(knob, *e, raio, alvo);
            fil += ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio).0;
            let grau = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
            let pos = m.positions();
            for v in 0..m.vert_count() {
                let q = pos[v];
                let mut d2 = f32::INFINITY;
                for t in &c {
                    let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                    d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
                }
                if d2.sqrt() > raio * 0.5 {
                    continue;
                }
                n += 1;
                if grau[v] == 4 {
                    quatro += 1;
                }
            }
        }
        assert!(n > 2_000, "a faixa mal tem miolo: {n}");
        (100.0 * quatro as f64 / n as f64, fil / RUMOS.len() as f64)
    };

    let (sem, fil_sem) = medir(0.0);
    let (com, fil_com) = medir(1.0);
    // ⚠️⚠️ **Estas barras são da REGRA DE ESCONDER que shipa**, e mudam com ela:
    // com o emparelhamento solto (medido e REVERTIDO — ver
    // `ph2d_mesh_render::wire`) a mesma malha lê `96,1 %` e o controlo `66,9`.
    // *A malha não mexe um bit entre as duas; muda o que a vista mostra dela —
    // e o dono não viu diferença nenhuma.*
    assert!(
        sem < 60.0,
        "o CONTROLO devia ser uma sopa e le {sem:.2} % de cruzamentos regulares (medido 48,3)"
    );
    // ⭐⭐⭐⭐ **A FILEIRA — a coluna que o dono julga, e que até aqui não tinha
    // gate NENHUM.** Ela é o comprimento mediano das linhas contínuas, em
    // arestas, e é a única das quatro grandezas desta cena que distingue as
    // duas COMPOSIÇÕES das mesmas duas leis: com a troca de ligação dentro da
    // alternância lê **`38,8`**, com ela no FIM do passe lê `27,5` — e a
    // regularidade mal se move (`93,0` contra `92,4`). *Uma medida que não
    // separa as duas composições não pode defender a que foi escolhida.*
    assert!(
        fil_sem < 8.0,
        "o CONTROLO devia ter fileiras curtas e le {fil_sem:.1} arestas"
    );
    assert!(
        fil_com > 33.0,
        "a fileira devia correr e le {fil_com:.1} arestas (medido: 38,8 alternado, 27,5 com a troca no fim)"
    );
    // ⚠️⚠️ **A barra saiu de um VALE MEDIDO entre os dois lados**, e a primeira
    // redacção pô-la em `88` — onde uma MUTAÇÃO SOBREVIVEU, porque **sem** a
    // troca de ligação a malha lê `89,4 %` e **com** ela `93,0 %`: as duas
    // passavam. *Uma barra larga não é só uma afirmação fraca — é o sítio onde
    // a peça que a wave acrescentou deixa de ser load-bearing.*
    //
    // ⛔ E a alternativa — *«correr a troca outra vez sobre a saída não muda
    // nada»* (um PONTO FIXO, sem barra nenhuma) — foi construída e **é
    // arquitecturalmente impossível**: cada dab deixa a PEGADA dele no ponto
    // fixo, e o dab seguinte, que se sobrepõe, volta a mexer nela. Medido: `21`
    // trocas pendentes contra `19` numa malha por pentear ⇒ *a grandeza nem
    // sequer discrimina*.
    assert!(
        com > 91.0,
        "a grade devia ter os quatro braços e le {com:.2} % (medido 93,0; sem a troca de ligação, 89,4)"
    );
}

/// ⭐⭐⭐⭐ **SONDA — o que SÃO os cruzamentos irregulares que sobram.**
///
/// ⚠️⚠️ **A pista é a ASSIMETRIA:** a vista lê `323` cruzamentos com CINCO
/// braços e **ZERO** com três. Num grafo de grade os defeitos topológicos vêm
/// **aos pares** (um `+1` e um `−1` cancelam-se na conta de Euler) ⇒ *`323`
/// contra `0` não pode ser topologia*.
///
/// A outra explicação é a **DIRECÇÃO DA DIAGONAL**: num quadrado triangulado
/// cada vértice interior deve ter **duas** diagonais escondidas para lhe
/// sobrarem quatro braços. Se as diagonais à volta dele não forem consistentes,
/// ele fica com **uma** escondida e lê `5` — *com a malha perfeitamente
/// regular*.
///
/// ⇒ ela cruza as duas: a valência na MALHA contra os braços na VISTA.
#[test]
#[ignore = "sonda: imprime o que sao os irregulares, nao afirma nada"]
fn diag_o_que_sao_os_irregulares() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let mut cruz = std::collections::BTreeMap::<(usize, usize), usize>::new();
    for (_, e) in RUMOS.iter() {
        let (m, c) = super::super::traco_com(1.0, *e, raio, alvo);
        let bracos = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
        // A valencia NA MALHA: quantas arestas cada vertice tem de facto.
        let mut arestas = Vec::new();
        ph2d_mesh_render::wire_indices_com(&m, false, &mut arestas);
        let mut val = vec![0usize; m.vert_count()];
        for par in arestas.as_chunks::<2>().0 {
            val[par[0] as usize] += 1;
            val[par[1] as usize] += 1;
        }
        let pos = m.positions();
        for v in 0..m.vert_count() {
            let q = pos[v];
            let mut d2 = f32::INFINITY;
            for t in &c {
                let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
            }
            if d2.sqrt() > raio * 0.5 {
                continue;
            }
            *cruz
                .entry((val[v].min(9), (bracos[v] as usize).min(9)))
                .or_default() += 1;
        }
    }
    println!("\nvalencia na MALHA x bracos na VISTA:");
    println!("   val  bracos   quantos   escondidas");
    for ((val, br), n) in &cruz {
        if *n < 5 {
            continue;
        }
        println!("{val:6} {br:7} {n:9} {:12}", val.saturating_sub(*br));
    }
}

/// ⭐⭐⭐⭐ **SONDA — por QUANTO a diagonal perde, nos cruzamentos que a vista
/// não fecha.**
///
/// A regra esconde a aresta que é a **mais longa dos DOIS** triângulos que a
/// partilham. Uma aresta que é a mais longa de **um só** é a candidata que
/// falha, e a pergunta é a margem: se ela perde por `1 %`, as células estão a
/// um empurrão de fechar; se perde por `30 %`, a célula é genuinamente
/// **enviesada** e nenhuma regra de esconder a salva.
#[test]
#[ignore = "sonda: imprime a margem, nao afirma nada"]
fn diag_por_quanto_a_diagonal_perde() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let mut razoes: Vec<f64> = Vec::new();
    for (_, e) in RUMOS.iter() {
        let (m, c) = super::super::traco_com(1.0, *e, raio, alvo);
        let pos = m.positions();
        // Por aresta: (n triangulos, em quantos e' a mais longa, razao minima
        // contra a mais longa do triangulo onde perde).
        let mut conta: std::collections::BTreeMap<(u32, u32), (u32, u32, f64)> =
            std::collections::BTreeMap::new();
        for f in m.faces() {
            for t in 0..f.tri_count() {
                let tri = f.tri_at(t);
                let lado = |i: usize| -> f64 {
                    let (a, b) = (pos[tri[i] as usize], pos[tri[(i + 1) % 3] as usize]);
                    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                    f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))).sqrt()
                };
                let mut maior = 0usize;
                for i in 1..3 {
                    if lado(i) > lado(maior) {
                        maior = i;
                    }
                }
                for i in 0..3 {
                    let (a, b) = (tri[i], tri[(i + 1) % 3]);
                    let ent = conta.entry((a.min(b), a.max(b))).or_insert((0, 0, 1.0));
                    ent.0 += 1;
                    if i == maior {
                        ent.1 += 1;
                    } else {
                        ent.2 = ent.2.min(lado(i) / lado(maior));
                    }
                }
            }
        }
        for ((a, b), (faces, maiores, razao)) in conta {
            if faces != 2 || maiores != 1 {
                continue;
            }
            // So' as do MIOLO da faixa.
            let meio = {
                let (pa, pb) = (pos[a as usize], pos[b as usize]);
                [
                    (pa[0] + pb[0]) * 0.5,
                    (pa[1] + pb[1]) * 0.5,
                    (pa[2] + pb[2]) * 0.5,
                ]
            };
            let mut d2 = f32::INFINITY;
            for t in &c {
                let w = [meio[0] - t[0], meio[1] - t[1], meio[2] - t[2]];
                d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
            }
            if d2.sqrt() <= raio * 0.5 {
                razoes.push(razao);
            }
        }
    }
    razoes.sort_by(f64::total_cmp);
    let n = razoes.len();
    println!("\n{n} arestas sao a mais longa de UM dos dois triangulos.");
    if n == 0 {
        return;
    }
    for q in [0.10, 0.25, 0.50, 0.75, 0.90] {
        println!(
            "  p{:02.0}  razao {:.4}",
            q * 100.0,
            razoes[((n - 1) as f64 * q) as usize]
        );
    }
    let perto = razoes.iter().filter(|r| **r > 0.97).count();
    println!(
        "  a menos de 3 % de fechar: {perto} ({:.1} %)",
        100.0 * perto as f64 / n as f64
    );
}

/// ⭐⭐⭐⭐ **SONDA — a CINTILAÇÃO da vista: mover UM vértice muda quantas linhas?**
///
/// ⚠️⚠️ **Esta é a pergunta que decide a regra de esconder**, e não a fracção de
/// cruzamentos regulares. A regra de hoje é **LOCAL** (o destino de uma aresta
/// depende só dos dois triângulos dela), logo mexer num vértice só mexe nas
/// linhas à volta dele. Um **emparelhamento guloso** é uma ordenação GLOBAL: um
/// vértice que se mexe reordena a lista e pode cascatear por toda a peça.
///
/// *Numa vista que o artista olha ENQUANTO esculpe, uma cascata é cintilação* —
/// e uma grade que pisca é pior que uma grade com células irregulares.
#[test]
#[ignore = "sonda: imprime a cintilacao, nao afirma nada"]
fn diag_a_cintilacao_da_vista() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let (m, c) = super::super::traco_com(1.0, RUMOS[0].1, raio, alvo);
    let esconde = |malha: &ph2d_mesh::Mesh| -> std::collections::BTreeSet<(u32, u32)> {
        let mut cheio = Vec::new();
        let mut grade = Vec::new();
        ph2d_mesh_render::wire_indices_com(malha, false, &mut cheio);
        ph2d_mesh_render::wire_indices_com(malha, true, &mut grade);
        let g: std::collections::BTreeSet<(u32, u32)> = grade
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0].min(p[1]), p[0].max(p[1])))
            .collect();
        cheio
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0].min(p[1]), p[0].max(p[1])))
            .filter(|e| !g.contains(e))
            .collect()
    };
    let antes = esconde(&m);
    // Mexer UM vertice do miolo, por uma fracção pequena da aresta.
    let centro = c[c.len() / 2];
    let alvo_v = {
        let pos = m.positions();
        let mut melhor = (f32::INFINITY, 0usize);
        for (v, q) in pos.iter().enumerate() {
            let d = [q[0] - centro[0], q[1] - centro[1], q[2] - centro[2]];
            let r = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]));
            if r < melhor.0 {
                melhor = (r, v);
            }
        }
        melhor.1
    };
    // ⚠️⚠️ **A perturbação é a que um DAB faz: MUITOS vértices de uma vez.**
    // As duas primeiras redacções mexiam UM vértice e liam `0` nas duas regras
    // até `20 %` da aresta — *um vértice sozinho quase nunca troca qual aresta
    // é a mais longa*, e a pergunta é sobre o que o artista vê entre dois dabs.
    let aresta = {
        let pos = m.positions();
        let anel = m.adjacency().vert_verts.neighbours(alvo_v);
        let mut soma = 0.0f32;
        for &w in anel {
            let (a, b) = (pos[alvo_v], pos[w as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            soma += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        }
        soma / anel.len().max(1) as f32
    };
    assert!(aresta > 0.0, "a aresta media do anel e' nula");
    let _ = alvo_v;
    for fraccao in [0.001f32, 0.01, 0.05] {
        let mut m2 = m.clone();
        {
            // Um deslocamento DETERMINÍSTICO e sem direcção preferida, do
            // tamanho do que um dab entrega.
            let mut semente = 0x9E37_79B9u32;
            for q in m2.positions_mut() {
                for eixo in q.iter_mut() {
                    semente = semente.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    let r = (semente >> 8) as f32 / 16_777_216.0 - 0.5;
                    *eixo += aresta * fraccao * r;
                }
            }
        }
        let depois = esconde(&m2);
        let mudaram = antes.symmetric_difference(&depois).count();
        println!(
            "sacudir a malha em {:.1} % da aresta: {mudaram} linhas mudam de estado (de {}) = {:.2} %",
            fraccao * 100.0,
            antes.len(),
            100.0 * mudaram as f64 / antes.len().max(1) as f64
        );
    }
}

/// ⭐⭐⭐⭐ **A VISTA NÃO PISCA — a coluna que defende a regra de esconder que
/// shipa contra a mais ambiciosa.**
///
/// A regra é um **emparelhamento**, logo mexer num vértice pode em princípio
/// reordenar a lista e cascatear por toda a peça. *Numa vista que o artista olha
/// ENQUANTO esculpe, uma cascata é cintilação — e uma grade que pisca é pior que
/// uma grade com células irregulares.*
///
/// A régua sacode a malha inteira por uma fracção da aresta (o que um dab
/// entrega) e conta que fracção das linhas muda de estado:
///
/// | regra | com pente | sem pente | **sacudir `1 %`** |
/// |---|---|---|---|
/// | mútua (a anterior) | `93,0 %` | `48,3 %` | `0,86 %` |
/// | **esta** | **`96,1 %`** | `66,9 %` | **`1,09 %`** |
/// | guloso sem cerca | `96,2 %` | `72,0 %` | `1,30 %` |
///
/// ⇒ a cerca da plausibilidade compra a mesma regularidade **e** fica no lado
/// bom das outras duas colunas. A barra sai do **vale entre esta e a gulosa**.
///
/// ⚠️⚠️ **A perturbação é de MUITOS vértices e isso é a lei da sonda:** as duas
/// primeiras redacções mexiam **um** vértice e liam `0` nas duas regras até
/// `20 %` da aresta — *um vértice sozinho quase nunca troca qual aresta é a mais
/// longa*, e a pergunta é sobre o que muda entre dois dabs.
#[test]
fn a_vista_da_grade_nao_pisca() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let (m, _c) = super::super::traco_com(1.0, RUMOS[0].1, raio, alvo);
    // (`_c` é o percurso; ele dá a unidade da sacudidela, abaixo.)
    let escondidas = |malha: &ph2d_mesh::Mesh| -> std::collections::BTreeSet<(u32, u32)> {
        let (mut cheio, mut grade) = (Vec::new(), Vec::new());
        ph2d_mesh_render::wire_indices_com(malha, false, &mut cheio);
        ph2d_mesh_render::wire_indices_com(malha, true, &mut grade);
        let g: std::collections::BTreeSet<(u32, u32)> = grade
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0].min(p[1]), p[0].max(p[1])))
            .collect();
        cheio
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0].min(p[1]), p[0].max(p[1])))
            .filter(|e| !g.contains(e))
            .collect()
    };
    let antes = escondidas(&m);
    assert!(
        antes.len() > 1_000,
        "a peça mal tem linhas escondidas: {}",
        antes.len()
    );

    // ⚠️⚠️ **A unidade é a aresta DA FAIXA, nunca a da peça inteira.** A faixa
    // está refinada e o resto não, logo a média da peça é muito maior — a 1.ª
    // redacção usou-a e sacudiu tão forte que leu `4,22 %` onde a sonda lia
    // `1,09`. *O mesmo «`1 %`» eram dois tamanhos.*
    let aresta = {
        let pos = m.positions();
        let adj = m.adjacency();
        let centro = _c[_c.len() / 2];
        let (mut soma, mut n) = (0.0f32, 0usize);
        for v in 0..m.vert_count() {
            let q = pos[v];
            let d = [q[0] - centro[0], q[1] - centro[1], q[2] - centro[2]];
            if d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt() > raio {
                continue;
            }
            for &w in adj.vert_verts.neighbours(v) {
                let (a, b) = (pos[v], pos[w as usize]);
                let e = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                soma += e[0].mul_add(e[0], e[1].mul_add(e[1], e[2] * e[2])).sqrt();
                n += 1;
            }
        }
        assert!(n > 100, "a faixa mal tem arestas para dar unidade: {n}");
        soma / n as f32
    };

    let mut m2 = m.clone();
    {
        let mut semente = 0x9E37_79B9u32;
        for q in m2.positions_mut() {
            for eixo in q.iter_mut() {
                semente = semente.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let r = (semente >> 8) as f32 / 16_777_216.0 - 0.5;
                *eixo += aresta * 0.01 * r;
            }
        }
    }
    let depois = escondidas(&m2);
    let mudaram = antes.symmetric_difference(&depois).count();
    let pct = 100.0 * mudaram as f64 / antes.len() as f64;
    // ⭐ **A metade de baixo importa tanto como a de cima:** uma regra que não
    // reagisse nada à geometria não estaria a ler a malha.
    assert!(
        pct > 0.2,
        "a vista não reagiu à malha ({pct:.2} %) — ela está a ignorar a geometria?"
    );
    assert!(
        pct < 1.0,
        "a vista pisca {pct:.2} % ao sacudir 1 % (medido 0,86; a regra solta 1,09 e a gulosa 1,30)"
    );
}

#[path = "scenes_pente_memoria_tests.rs"]
mod memoria;

#[path = "scenes_pente_relogio_tests.rs"]
mod relogio;
