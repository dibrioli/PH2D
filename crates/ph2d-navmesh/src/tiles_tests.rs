use std::collections::BTreeSet;

use super::*;

/// ⭐ O ORÁCULO da montagem por blocos (W10): a montagem INTEIRA de antes — os vértices de todos os mosaicos fundidos por posição na grelha, as junções em
/// T das costuras reparadas, e a [`NavMesh`] pela porta única dela.
fn monta(mosaicos: &BTreeMap<(i64, i64), Mosaico>, lado: i64) -> NavMesh {
    // ⚠️ Só um vértice NUMA LINHA DE COSTURA (`x = k·lado` ou `y = k·lado`) pode ser de dois
    // mosaicos: os outros entram direitos, e só estes passam pelo índice (medido: a montagem é a
    // maior parte de uma mudança, e o índice de todos os vértices era um terço dela).
    let mut indice: BTreeMap<P, u32> = BTreeMap::new();
    let mut verticais: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    let mut horizontais: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    let mut pts: Vec<P> = Vec::new();
    // Os anéis CONTÍGUOS, como a `NavMesh` os guarda (nenhuma lista por polígono — W9).
    let mut cru_off: Vec<u32> = vec![0];
    let mut cru: Vec<u32> = Vec::new();
    let mut ids: Vec<u16> = Vec::new();
    let mut mapa: Vec<u32> = Vec::new();
    for m in mosaicos.values() {
        ids.extend_from_slice(&m.ids);
        mapa.clear();
        mapa.extend(m.pts.iter().map(|&p| {
            let (vx, hy) = (p.0.rem_euclid(lado) == 0, p.1.rem_euclid(lado) == 0);
            if !(vx || hy) {
                pts.push(p);
                return (pts.len() - 1) as u32;
            }
            *indice.entry(p).or_insert_with(|| {
                if vx {
                    verticais.entry(p.0).or_default().insert(p.1);
                }
                if hy {
                    horizontais.entry(p.1).or_default().insert(p.0);
                }
                pts.push(p);
                (pts.len() - 1) as u32
            })
        }));
        for p in &m.polys {
            cru.extend(p.iter().map(|&v| mapa[v as usize]));
            cru_off.push(cru.len() as u32);
        }
    }
    let mut ring_off: Vec<u32> = Vec::with_capacity(cru_off.len());
    ring_off.push(0);
    let mut ring: Vec<u32> = Vec::with_capacity(cru.len());
    for w in cru_off.windows(2) {
        let p = &cru[w[0] as usize..w[1] as usize];
        let n = p.len();
        for i in 0..n {
            let (a, b) = (pts[p[i] as usize], pts[p[(i + 1) % n] as usize]);
            ring.push(p[i]);
            if a.0 == b.0 && a.0.rem_euclid(lado) == 0 {
                ring.extend(entre(verticais.get(&a.0), a.1, b.1).map(|y| indice[&(a.0, y)]));
            } else if a.1 == b.1 && a.1.rem_euclid(lado) == 0 {
                ring.extend(entre(horizontais.get(&a.1), a.0, b.0).map(|x| indice[&(x, a.1)]));
            }
        }
        ring_off.push(ring.len() as u32);
    }
    let verts: Vec<V2> = pts.iter().map(|&p| to_world(p)).collect();
    NavMesh::from_rings(verts, ring_off, ring, ids).unwrap_or_else(|_| vazia())
}

/// Os valores de `linha` estritamente entre `de` e `para`, pela ordem de `de` para `para`.
fn entre(linha: Option<&BTreeSet<i64>>, de: i64, para: i64) -> Box<dyn Iterator<Item = i64> + '_> {
    let Some(l) = linha else {
        return Box::new(std::iter::empty());
    };
    if de < para {
        Box::new(l.range(de + 1..para).copied())
    } else if para < de {
        Box::new(l.range(para + 1..de).rev().copied())
    } else {
        Box::new(std::iter::empty())
    }
}

#[test]
fn o_cruzamento_e_o_mesmo_dos_dois_lados_da_costura() {
    // Um quadrado rodado que atravessa a linha x = 100: o mosaico da esquerda e o da direita
    // cortam-no por lados opostos e têm de escrever os MESMOS pontos na linha.
    let poly: Vec<P> = vec![(37, -91), (163, 7), (71, 141), (-29, 33)];
    let esq = corta(&poly, (-200, -200), (100, 200));
    let dir = corta(&poly, (100, -200), (300, 200));
    let na_linha = |v: &[P]| {
        let mut l: Vec<P> = v.iter().copied().filter(|p| p.0 == 100).collect();
        l.sort_unstable();
        l
    };
    assert_eq!(na_linha(&esq).len(), 2);
    assert_eq!(na_linha(&esq), na_linha(&dir));
}

#[test]
fn o_cruzamento_sai_da_aresta_original_e_nao_do_pedaco() {
    // Uma aresta comprida cortada primeiro por x = 10 e depois por y = 50: o ponto em y = 50 tem
    // de ser o da aresta ORIGINAL — o mesmo que um mosaico que só a corta por y = 50 escreve.
    let tri: Vec<P> = vec![(0, 0), (97, 103), (-50, 80)];
    let a = corta(&tri, (10, -1000), (1000, 50));
    let b = corta(&tri, (-1000, -1000), (1000, 50));
    let em_y = |v: &[P]| {
        v.iter()
            .copied()
            .filter(|p| p.1 == 50 && p.0 > 10)
            .collect::<Vec<_>>()
    };
    assert_eq!(em_y(&a), em_y(&b));
    assert_eq!(em_y(&a).len(), 1);
}

/// ⭐ **O caso que a aresta ORIGINAL decide** (achado por busca, depois de a prova de mutação o
/// mostrar sem régua): dois mosaicos EMPILHADOS, e uma aresta que atravessa o de baixo de ponta a
/// ponta. No de baixo, o corte por `y = −10` vem ANTES do de `y = 60`; do pedaço, o ponto em `y = 60`
/// arredondava para `x = −3`, e o de cima (que só corta por `y = 60`) escreve `x = −2`.
#[test]
fn a_costura_de_mosaicos_empilhados_concorda_quando_a_aresta_atravessa_um_inteiro() {
    let tri: Vec<P> = vec![(-9, -51), (0, 102), (-40, 20)];
    let baixo = corta(&tri, (-100, -10), (100, 60));
    let cima = corta(&tri, (-100, 60), (100, 130));
    let em_60 = |v: &[P]| {
        let mut l: Vec<P> = v.iter().copied().filter(|p| p.1 == 60).collect();
        l.sort_unstable();
        l
    };
    assert_eq!(em_60(&baixo), em_60(&cima));
    assert!(
        em_60(&baixo).contains(&(-2, 60)),
        "o ponto é o da aresta ORIGINAL"
    );
}

#[test]
fn a_divisao_arredonda_ao_mais_perto() {
    assert_eq!(divide_ao_mais_perto(7, 2), 4);
    assert_eq!(divide_ao_mais_perto(-7, 2), -4);
    assert_eq!(divide_ao_mais_perto(5, 3), 2);
    assert_eq!(divide_ao_mais_perto(-5, 3), -2);
    assert_eq!(divide_ao_mais_perto(5, -3), -2);
}

#[test]
fn um_poligono_fora_do_rectangulo_some() {
    let q: Vec<P> = vec![(0, 0), (10, 0), (10, 10), (0, 10)];
    assert!(corta(&q, (20, 20), (30, 30)).is_empty());
    assert_eq!(corta(&q, (-5, -5), (50, 50)), q);
}

/// ⭐ (W9) **Onde a malha mudou**: tudo na 1.ª construção, nada quando nada muda, e só o mosaico de uma
/// pedra que se mexe — é o que deixa a fila do replaneio não percorrer os caminhos longe dela.
#[test]
fn a_area_que_mudou_e_o_mosaico_da_pedra() {
    let reg = vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]];
    let pedra = |x: f64| Shape::Circle {
        center: [x, 22.0],
        radius: 0.5,
    };
    let mut t = TiledMesh::new(Params::default(), 10.0);
    t.update(&reg, &[pedra(25.0)]);
    assert_eq!(t.changed_area(), None, "a 1.ª construção muda tudo");
    t.update(&reg, &[pedra(25.0)]);
    assert_eq!(t.changed_area(), Some(Vec::new()), "nada mudou");
    t.update(&reg, &[pedra(25.5)]);
    let a = t.changed_area().expect("só os mosaicos refeitos");
    assert_eq!(a.len(), 1, "{a:?}");
    let (lo, hi) = a[0];
    assert!(
        lo[0] <= 25.5 && 25.5 <= hi[0] && lo[1] <= 22.0 && 22.0 <= hi[1],
        "{a:?}"
    );
    assert!(hi[0] - lo[0] <= 10.0 + 1e-9, "um mosaico de 10 m: {a:?}");
}

/// Um gerador pequeno e determinístico (os gates desta crate não alcançam `tests/it/cena.rs`).
struct Lcg(u64);

impl Lcg {
    fn f(&mut self, a: f64, b: f64) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        a + (b - a) * ((self.0 >> 11) as f64 / (1u64 << 53) as f64)
    }
}

fn forma(r: &mut Lcg) -> Shape {
    let c = [r.f(1.0, 15.0), r.f(1.0, 11.0)];
    if r.f(0.0, 1.0) < 0.5 {
        Shape::Circle {
            center: c,
            radius: r.f(0.3, 1.2),
        }
    } else {
        let (hx, hy, a) = (r.f(0.2, 1.5), r.f(0.2, 1.5), r.f(0.0, 3.0));
        let (co, si) = (a.cos(), a.sin());
        Shape::Convex(
            [[-hx, -hy], [hx, -hy], [hx, hy], [-hx, hy]]
                .iter()
                .map(|q| [c[0] + q[0] * co - q[1] * si, c[1] + q[0] * si + q[1] * co])
                .collect(),
        )
    }
}

/// ⭐⭐ (W10, plano 30 §18) **A montagem por blocos É a montagem inteira de antes**, campo a campo
/// (vértices ao bit, anéis cosidos, vizinhos, gémeos, cantos, vértice→polígonos, ilhas, áreas, paredes,
/// caixa) e com a mesma resposta de `locate_all` — a frio, depois de cada mudança, com lamas, e quando
/// mosaicos SAEM da região.
#[test]
fn a_montagem_por_blocos_e_a_montagem_inteira_ao_bit() {
    let (mut cosidos, mut parciais, mut comparados, mut saidas) = (0usize, 0, 0, 0);
    let mut paredes = OraculoDasParedes::default();
    for seed in 1..=12u64 {
        let mut r = Lcg(seed);
        let p = Params {
            agent_radius: [0.0, 0.3, 0.6][(seed % 3) as usize],
            corner: if seed % 4 == 0 {
                crate::Corner::Miter
            } else {
                crate::Corner::Round
            },
            disk_sides: 8,
            merge: seed % 2 == 0,
        };
        let lado = [2.0, 3.0, 5.0][(seed % 3) as usize];
        let mut reg = vec![[0.0, 0.0], [16.0, 0.0], [16.0, 12.0], [0.0, 12.0]];
        let mut obs: Vec<Shape> = (0..8).map(|_| forma(&mut r)).collect();
        if p.agent_radius == 0.0 {
            // As JUNÇÕES EM T: um obstáculo que encosta a uma costura só de UM lado (a aresta e o
            // vértice em cima da linha) — o mosaico do outro lado não tem esses pontos na aresta dele.
            let (x, z) = (2.0 * lado, 4.0 * lado);
            obs.push(Shape::Convex(vec![
                [x, 2.3],
                [x + 1.0, 2.3],
                [x + 1.0, 4.7],
                [x, 4.7],
            ]));
            obs.push(Shape::Convex(vec![
                [z, 7.3],
                [z + 1.0, 6.5],
                [z + 2.0, 7.3],
                [z + 1.0, 8.1],
            ]));
            // (W11) A FRONTEIRA QUE SE TOCA: dois quadrados encostados por uma quina, em cima de uma
            // costura — o vértice da quina tem duas continuações.
            let c = [lado, 2.0 * lado];
            for (sx, sy) in [(-1.0, -1.0), (1.0, 1.0)] {
                let o = [c[0] + sx, c[1] + sy];
                obs.push(Shape::Convex(vec![
                    [c[0].min(o[0]), c[1].min(o[1])],
                    [c[0].max(o[0]), c[1].min(o[1])],
                    [c[0].max(o[0]), c[1].max(o[1])],
                    [c[0].min(o[0]), c[1].max(o[1])],
                ]));
            }
        }
        let areas = vec![Area {
            shape: forma(&mut r),
            id: 1,
        }];
        let mut t = TiledMesh::new(p, lado);
        let mut pb = ph2d_orca::ParedesPorBlocos::new();
        for passo in 0..8 {
            if passo > 0 {
                let i = passo % obs.len();
                obs[i] = forma(&mut r);
            }
            if passo == 6 {
                reg = vec![[0.0, 0.0], [11.0, 0.0], [11.0, 9.0], [0.0, 9.0]];
            }
            let ars: &[Area] = if seed % 2 == 0 { &areas } else { &[] };
            let antes = t.mosaicos.len();
            t.update_with_areas(&reg, &obs, ars);
            saidas += usize::from(t.mosaicos.len() < antes);
            parciais += usize::from(t.stats().rebuilt < t.stats().tiles);
            let inteira = monta(&t.mosaicos, t.lado);
            let m = t.mesh();
            assert_eq!(m.diferenca(&inteira), None, "semente {seed}, passo {passo}");
            paredes.confere(&t, &mut pb, &mut r);
            let crus: usize = t
                .mosaicos
                .values()
                .flat_map(|q| &q.polys)
                .map(Vec::len)
                .sum();
            cosidos += m.polys().map(|q| q.len()).sum::<usize>() - crus;
            // A localização: pontos ao acaso e pontos EXACTAMENTE nas linhas das costuras.
            let (mut a, mut b) = (Vec::new(), Vec::new());
            for k in 0..300 {
                let mut q = [r.f(-0.5, 16.5), r.f(-0.5, 12.5)];
                if k % 3 == 0 {
                    q[k % 2] = (q[k % 2] / lado).round() * lado;
                }
                m.locate_all(q, &mut a);
                inteira.locate_all(q, &mut b);
                assert_eq!(a, b, "semente {seed}, passo {passo}: localizar {q:?}");
                comparados += usize::from(!a.is_empty());
            }
        }
    }
    // Os CONTROLOS de população: houve junções em T, actualizações parciais, mosaicos que saíram, e
    // pontos dentro da malha.
    assert!(
        cosidos >= 100,
        "só {cosidos} vértices cosidos (medido: 115)"
    );
    assert!(
        parciais >= 60,
        "só {parciais} actualizações parciais (medido: 72)"
    );
    assert!(
        saidas >= 12,
        "só {saidas} regiões encolheram (uma por semente)"
    );
    assert!(
        comparados >= 15_000,
        "só {comparados} pontos localizados (medido: 16 078)"
    );
    paredes.populacao();
}

/// ⭐ (W11, plano 30 §19) **As paredes do desvio por mosaicos são as da malha inteira, ao bit**: as
/// faixas de [`TiledMesh::paredes_por_mosaico`] dadas a uma `ParedesPorBlocos` que VIVE entre as
/// actualizações (como na ponte) = `Walls::from_walkable_walls(m.verts(), m.walls())`, campo a campo, e
/// a mesma resposta de `near`.
#[derive(Clone, Copy, Default)]
struct OraculoDasParedes {
    /// Entradas cujo seguinte é de OUTRO mosaico (a cadeia atravessa a costura).
    atravessam: usize,
    /// Pontos onde começam várias entradas (a fronteira toca-se).
    tocam: usize,
    perguntas: usize,
}

impl OraculoDasParedes {
    fn confere(&mut self, t: &TiledMesh, pb: &mut ph2d_orca::ParedesPorBlocos, r: &mut Lcg) {
        let (m, faixas) = (t.mesh(), t.paredes_por_mosaico());
        let (verts, walls) = (m.verts(), m.walls());
        // As faixas cobrem as paredes, por ordem e sem buracos.
        let mut fim = 0;
        for f in faixas {
            assert_eq!(f.paredes.start, fim);
            fim = f.paredes.end;
        }
        assert_eq!(fim, walls.len());
        pb.retem(|k| faixas.iter().any(|f| f.chave == k));
        for f in faixas {
            pb.poe(
                f.chave,
                f.lo,
                f.hi,
                walls[f.paredes.clone()]
                    .iter()
                    .map(|&(de, para)| (verts[de as usize], verts[para as usize])),
            );
        }
        let w = pb.monta();
        let o = ph2d_orca::Walls::from_walkable_walls(verts, walls);
        let b = |p: V2| [p[0].to_bits(), p[1].to_bits()];
        assert_eq!(w.len(), o.len());
        let mosaico_de = |i: usize| faixas.partition_point(|f| f.paredes.end <= i);
        let mut comecos = vec![0u32; verts.len()];
        for &(_, para) in walls {
            comecos[para as usize] += 1;
        }
        self.tocam += comecos.iter().filter(|&&c| c > 1).count();
        for i in 0..o.len() {
            assert_eq!(b(w.point(i)), b(o.point(i)), "point {i}");
            assert_eq!(w.next(i), o.next(i), "next {i}");
            assert_eq!(w.prev(i), o.prev(i), "prev {i}");
            assert_eq!(b(w.dir(i)), b(o.dir(i)), "dir {i}");
            assert_eq!(w.convex(i), o.convex(i), "convex {i}");
            self.atravessam += usize::from(mosaico_de(i) != mosaico_de(o.next(i)));
        }
        let (mut x, mut y) = (Vec::new(), Vec::new());
        for k in 0..100 {
            let q = [r.f(-0.5, 16.5), r.f(-0.5, 12.5)];
            let alcance = if k % 10 == 0 {
                f64::INFINITY
            } else {
                r.f(0.0, 4.0)
            };
            w.near(q, alcance, &mut x);
            o.near(q, alcance, &mut y);
            assert_eq!(x, y, "near {q:?} {alcance}");
            self.perguntas += usize::from(!x.is_empty());
        }
    }

    /// Os CONTROLOS de população. ⚠️ Sem os dois quadrados encostados, `tocam` era ZERO.
    fn populacao(&self) {
        let OraculoDasParedes {
            atravessam,
            tocam,
            perguntas,
        } = *self;
        assert!(atravessam >= 2_500, "só {atravessam} (medido: 3 200)");
        assert!(tocam >= 16, "só {tocam} pontos que se tocam (medido: 24)");
        assert!(perguntas >= 5_000, "só {perguntas} (medido: 6 546)");
    }
}

/// (W12) A assinatura de um obstáculo distingue formas que só diferem no SINAL de dois números: por
/// palavra sem mistura, a diferença do bit do sinal ficava nesse bit e duas anulavam-se — o mosaico
/// não se refazia.
#[test]
fn a_assinatura_distingue_os_sinais_trocados() {
    let c = |x: f64, y: f64, r: f64| Shape::Circle {
        center: [x, y],
        radius: r,
    };
    let pares = [
        (c(1.0, 2.0, 0.5), c(-1.0, -2.0, 0.5)),
        (c(1.0, 2.0, 0.5), c(-1.0, 2.0, -0.5)),
        (c(3.0, 4.0, 0.5), c(3.0, -4.0, -0.5)),
        (
            Shape::Convex(vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]]),
            Shape::Convex(vec![[-1.0, -1.0], [2.0, 1.0], [2.0, 2.0]]),
        ),
    ];
    for (a, b) in &pares {
        assert_ne!(assinatura(a), assinatura(b), "{a:?} e {b:?}");
    }
}

/// ⭐ (W14, ADR-0178) **Os mosaicos feitos em PARALELO são os de UMA thread, ao bit** — a frio e depois
/// de mudanças que refazem vários mosaicos (o `rayon` acorda a partir de dois), com lamas.
#[test]
fn os_mosaicos_feitos_em_paralelo_sao_os_de_uma_thread_ao_bit() {
    let mut r = Lcg(0x0057_1714);
    let lado = 60.0;
    let reg = vec![[0.0, 0.0], [lado, 0.0], [lado, lado], [0.0, lado]];
    let caixa = |r: &mut Lcg, h: (f64, f64)| -> Shape {
        let c = [r.f(0.0, lado), r.f(0.0, lado)];
        let (hx, hy) = (r.f(h.0, h.1), r.f(h.0, h.1));
        let (a, b) = (r.f(-1.0, 1.0), r.f(-1.0, 1.0));
        let l = (a * a + b * b).sqrt().max(1e-6);
        let (co, si) = (a / l, b / l);
        Shape::Convex(
            [[-hx, -hy], [hx, -hy], [hx, hy], [-hx, hy]]
                .iter()
                .map(|q| [c[0] + q[0] * co - q[1] * si, c[1] + q[0] * si + q[1] * co])
                .collect(),
        )
    };
    let obs: Vec<Shape> = (0..300).map(|_| caixa(&mut r, (0.2, 1.2))).collect();
    let areas: Vec<Area> = (0..30)
        .map(|i| Area {
            shape: caixa(&mut r, (1.0, 3.0)),
            id: i + 1,
        })
        .collect();
    // As mudanças: cinco obstáculos espalhados mexem, depois uma lama, depois saem dois.
    let mut passos = vec![(obs.clone(), areas.clone())];
    let mut o = obs.clone();
    for i in [3, 70, 140, 210, 280] {
        if let Shape::Convex(pts) = &mut o[i] {
            pts.iter_mut().for_each(|p| p[0] += 0.7);
        }
    }
    passos.push((o.clone(), areas.clone()));
    let mut a = areas.clone();
    if let Shape::Convex(pts) = &mut a[5].shape {
        pts.iter_mut().for_each(|p| p[1] += 0.9);
    }
    passos.push((o.clone(), a.clone()));
    o.remove(250);
    o.remove(20);
    passos.push((o, a));
    let corre = |n: usize| -> Vec<(NavMesh, TileStats)> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .expect("o pool");
        pool.install(|| {
            let params = Params {
                agent_radius: 0.4,
                ..Params::default()
            };
            let mut t = TiledMesh::new(params, 10.0);
            passos
                .iter()
                .map(|(o, a)| {
                    t.update_with_areas(&reg, o, a);
                    (t.mesh().clone(), t.stats())
                })
                .collect()
        })
    };
    let (um, oito) = (corre(1), corre(8));
    for (k, ((m1, s1), (m8, s8))) in um.iter().zip(&oito).enumerate() {
        assert_eq!(m1.diferenca(m8), None, "o passo {k}");
        assert_eq!(s1, s8, "o passo {k}");
        // A população: cada passo refaz vários mosaicos — o paralelo acorda.
        assert!(
            s1.rebuilt >= MOSAICOS_EM_PARALELO,
            "o passo {k} refez {}",
            s1.rebuilt
        );
    }
    assert!(
        um[0].1.rebuilt >= 36 && um[0].0.has_areas(),
        "a frio: {:?}",
        um[0].1
    );
}
