//! Os gates da lei. ⚠️ Eles medem **propriedades do paper**, nunca números que eu escolhi.

use super::{Handle, Options, bounded_biharmonic};
use ph2d_poly2d::Mesh2d;

/// Uma grelha rectangular de `cols × rows` células sobre `[0,w] × [0,h]`, partida em dois.
fn grelha(w: f64, h: f64, cols: usize, rows: usize) -> Mesh2d {
    let mut m = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: [w as u32, h as u32],
    };
    for j in 0..=rows {
        for i in 0..=cols {
            m.rest
                .push([w * i as f64 / cols as f64, h * j as f64 / rows as f64]);
        }
    }
    let id = |i: usize, j: usize| (j * (cols + 1) + i) as u32;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

/// ⭐⭐⭐ **AS TRÊS RESTRIÇÕES DO PAPER SÃO HONRADAS** — e as três são medidas, não afirmadas.
#[test]
fn as_tres_restricoes_do_paper_sao_honradas() {
    let m = grelha(300.0, 100.0, 24, 8);
    let ossos = [
        Handle {
            a: [10.0, 50.0],
            b: [140.0, 50.0],
        },
        Handle {
            a: [160.0, 50.0],
            b: [290.0, 50.0],
        },
    ];
    let r = bounded_biharmonic(&m, &ossos, Options::default()).expect("a malha tem area e ossos");
    assert!(
        r.report.convergiu,
        "o conjunto activo tocou a rede em {} rondas — a solucao e' admissivel e pode nao ser a minima",
        r.report.rondas
    );
    assert!(
        r.report.residuo < 1e-6,
        "o CG deixou residuo {:.3e}: a solucao nao e' a do sistema",
        r.report.residuo
    );
    assert!(
        r.report.soma_pior < 1e-9,
        "a particao de unidade falhou por {:.3e}",
        r.report.soma_pior
    );
    assert_eq!(
        r.report.fora_de_banda, 0.0,
        "ha' peso fora de [0,1] — as caixas sao metade da lei, e sem elas um osso EMPURRA arte que \
         devia ignorar"
    );
    assert!(
        r.report.presos >= 2,
        "so' {} vertices presos: um osso sem sujeito e' uma equacao sem condicao de fronteira",
        r.report.presos
    );
}

/// ⭐⭐⭐ **NÃO HÁ ÓRFÃO — e este é o defeito de 2026-09-15 curado POR CONSTRUÇÃO.**
///
/// ⛔ A lei anterior dava a cada osso um RAIO euclidiano; fora dele a arte ficava sem dono e saltava
/// em salto seco para o osso mais próximo — *um salto seco num mapa contínuo é um rasgo*, e foi
/// isso que o dono fotografou. Aqui a energia é resolvida **sobre a arte inteira**: não existe
/// «fora».
///
/// ⚠️ A fixtura é o caso que a lei anterior reprovava: arte **muito mais alta** que o osso é longo.
#[test]
fn nao_existe_orfao_mesmo_com_a_arte_muito_maior_que_o_osso() {
    // Um osso curtíssimo ao meio de uma arte grande — a lei anterior daria raio `20` e deixaria
    // ~toda a arte órfã.
    let m = grelha(300.0, 300.0, 20, 20);
    let ossos = [
        Handle {
            a: [140.0, 150.0],
            b: [160.0, 150.0],
        },
        Handle {
            a: [160.0, 150.0],
            b: [180.0, 150.0],
        },
    ];
    let r = bounded_biharmonic(&m, &ossos, Options::default()).expect("resolve");
    for (v, linha) in r.por_vertice.iter().enumerate() {
        let s: f64 = linha.iter().sum();
        assert!(
            (s - 1.0).abs() < 1e-9,
            "o vertice {v} soma {s}: ha' arte sem dono, e e' ali que ela RASGA"
        );
    }
    assert_eq!(r.report.fora_de_banda, 0.0);
}

/// ⭐⭐⭐⭐ **A PROPRIEDADE QUE SEPARA O PADRÃO-OURO DA LEI ANTERIOR: ele é sensível à FORMA.**
///
/// ⛔⛔ Dois braços de um **U** estão a `20 px` um do outro no plano e a `~400 px` **pela arte**. A
/// lei euclidiana mede a primeira distância e contamina o braço vizinho: rodar um braço arrasta o
/// outro. A energia bilaplaciana é integrada **sobre a arte**, logo ela mede a segunda.
///
/// ⚠️ **A régua é a RAZÃO entre os dois braços, não um valor absoluto** — um número absoluto seria
/// função da densidade da malha, e a propriedade é sobre o CONTRASTE.
#[test]
fn o_peso_nao_atravessa_o_vazio_entre_dois_bracos_de_um_u() {
    // Um U: dois braços verticais (x em 0..40 e 100..140) unidos por baixo (y em 0..40).
    let mut m = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: [140, 200],
    };
    let passo = 10.0;
    let dentro = |x: f64, y: f64| {
        (y < 40.0 && (0.0..=140.0).contains(&x))
            || ((0.0..=40.0).contains(&x) || (100.0..=140.0).contains(&x))
    };
    let mut idx = std::collections::BTreeMap::new();
    let (nx, ny) = (14usize, 20usize);
    for j in 0..=ny {
        for i in 0..=nx {
            let (x, y) = (i as f64 * passo, j as f64 * passo);
            if dentro(x, y) {
                idx.insert((i, j), m.rest.len() as u32);
                m.rest.push([x, y]);
            }
        }
    }
    for j in 0..ny {
        for i in 0..nx {
            let (Some(&a), Some(&b), Some(&c), Some(&d)) = (
                idx.get(&(i, j)),
                idx.get(&(i + 1, j)),
                idx.get(&(i + 1, j + 1)),
                idx.get(&(i, j + 1)),
            ) else {
                continue;
            };
            // ⚠️ Só a célula INTEIRA vira quad: meia célula na fronteira do U faria um triângulo
            // que atravessa o vazio, e a fixtura mediria a malha em vez da lei.
            let centro = [(i as f64 + 0.5) * passo, (j as f64 + 0.5) * passo];
            if !dentro(centro[0], centro[1]) {
                continue;
            }
            m.tris.push([a, b, c]);
            m.tris.push([a, c, d]);
        }
    }
    // Um osso no braço ESQUERDO, em cima. E um na base, para haver com quem repartir.
    let ossos = [
        Handle {
            a: [20.0, 120.0],
            b: [20.0, 190.0],
        },
        Handle {
            a: [10.0, 20.0],
            b: [130.0, 20.0],
        },
    ];
    let r = bounded_biharmonic(&m, &ossos, Options::default()).expect("resolve");

    // O topo do braço ESQUERDO (o do osso) e o topo do DIREITO, à mesma altura.
    let acha = |x: f64, y: f64| {
        m.rest
            .iter()
            .enumerate()
            .min_by(|(_, p), (_, q)| {
                let dp = (p[0] - x).hypot(p[1] - y);
                let dq = (q[0] - x).hypot(q[1] - y);
                dp.total_cmp(&dq)
            })
            .map(|(i, _)| i)
            .expect("a malha tem vertices")
    };
    let esquerdo = r.por_vertice[acha(20.0, 180.0)][0];
    let direito = r.por_vertice[acha(120.0, 180.0)][0];
    assert!(
        esquerdo > 0.9,
        "o braco DO OSSO recebeu {esquerdo:.3} — a condicao de Dirichlet nao chegou la'"
    );
    assert!(
        direito < 0.1,
        "o braco VIZINHO recebeu {direito:.3}: o peso atravessou o vazio, que e' exactamente o \
         defeito da distancia euclidiana que esta lei existe para apagar"
    );
    assert!(
        esquerdo / direito.max(1e-12) > 20.0,
        "o contraste entre os dois bracos e' so' {:.1}x",
        esquerdo / direito.max(1e-12)
    );
}

/// ⭐⭐ **UM OSSO SÓ FICA COM A ARTE INTEIRA** — o caso degenerado, e o controlo da normalização.
#[test]
fn um_osso_so_fica_com_tudo() {
    let m = grelha(100.0, 100.0, 8, 8);
    let ossos = [Handle {
        a: [20.0, 50.0],
        b: [80.0, 50.0],
    }];
    let r = bounded_biharmonic(&m, &ossos, Options::default()).expect("resolve");
    for linha in &r.por_vertice {
        assert!(
            (linha[0] - 1.0).abs() < 1e-9,
            "um osso so' tem de levar tudo"
        );
    }
}

/// ⭐ **A porta RECUSA em voz alta** — ⛔ nunca um vector de zeros que o consumidor normalizaria em
/// `NaN`.
#[test]
fn a_porta_recusa_o_que_nao_tem_resposta() {
    let m = grelha(100.0, 100.0, 4, 4);
    assert!(
        bounded_biharmonic(&m, &[], Options::default()).is_none(),
        "sem ossos"
    );
    let vazia = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: [1, 1],
    };
    assert!(
        bounded_biharmonic(
            &vazia,
            &[Handle {
                a: [0.0, 0.0],
                b: [1.0, 0.0]
            }],
            Options::default()
        )
        .is_none(),
        "sem malha"
    );
}

/// ⭐⭐⭐ **A FOLGA DA JUNTA DÁ VOTO À ENERGIA JUNTO DELA** — sem ela, os pinos de dois ossos
/// encostam-se e a mistura não tem onde acontecer.
///
/// ⚠️ **A régua é o LARGURA DA TRANSIÇÃO ao longo do eixo**: por quantos vértices da linha central o
/// peso do 1.º osso viaja de `0,9` a `0,1`. Com os pinos encostados ela é `0` — o peso salta de `1`
/// para `0` entre vizinhos —, e a mistura fica sem domínio.
///
/// ⛔ **Não é um gate de suavidade e nem podia ser:** medir o peso sobre a linha do eixo com folga
/// `0` mede a **condição de fronteira**, não a solução (foi como eu quase concluí que o padrão-ouro
/// era uma escada). O que este gate afirma é que a linha do eixo **deixa de ser toda Dirichlet**.
///
/// (Mutação: `folga_da_junta: 0.0` no `Default` ⇒ RED, com a largura a ler `0`.)
#[test]
fn a_folga_da_junta_deixa_a_energia_decidir_junto_da_junta() {
    // Dois ossos colineares que se tocam — a forma que produz o defeito.
    let m = grelha_de_teste(40, 24, 400.0, 240.0);
    let ossos = [
        Handle {
            a: [0.0, 120.0],
            b: [200.0, 120.0],
        },
        Handle {
            a: [200.0, 120.0],
            b: [400.0, 120.0],
        },
    ];
    let largura = |folga: f64| -> usize {
        let w = bounded_biharmonic(
            &m,
            &ossos,
            Options {
                folga_da_junta: folga,
                ..Options::default()
            },
        )
        .expect("resolve");
        // Os vértices da linha do eixo, por ordem de `x`, e quantos ficam na transição.
        let mut linha: Vec<(f64, f64)> = m
            .rest
            .iter()
            .enumerate()
            .filter(|(_, p)| (p[1] - 120.0).abs() < 1e-9)
            .map(|(v, p)| (p[0], w.por_vertice[v][0]))
            .collect();
        linha.sort_by(|a, b| a.0.total_cmp(&b.0));
        linha
            .iter()
            .filter(|(_, v)| (0.1..=0.9).contains(v))
            .count()
    };
    let encostado = largura(0.0);
    let com_folga = largura(Options::default().folga_da_junta);
    assert_eq!(
        encostado, 0,
        "com os pinos ENCOSTADOS a transicao ja' tem {encostado} vertices — a fixtura deixou de \
         produzir o defeito, e o gate abaixo nao esta' a afirmar nada"
    );
    assert!(
        com_folga >= 3,
        "com a folga de fabrica a transicao no eixo tem {com_folga} vertices: a energia continua \
         sem ter onde misturar os dois ossos"
    );
}

/// Uma grelha regular de teste — `cols × rows` células sobre `larg × alt`.
fn grelha_de_teste(cols: usize, rows: usize, larg: f64, alt: f64) -> ph2d_poly2d::Mesh2d {
    let mut m = ph2d_poly2d::Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: [larg as u32, alt as u32],
    };
    for j in 0..=rows {
        for i in 0..=cols {
            m.rest
                .push([larg * i as f64 / cols as f64, alt * j as f64 / rows as f64]);
        }
    }
    let id = |i: usize, j: usize| (j * (cols + 1) + i) as u32;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}
