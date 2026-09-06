//! ⭐⭐⭐ **PARIDADE COM O ORÁCULO** — os 46 traços do pincel de tecido da referência
//! sobre malhas NOSSAS ([`docs/3D/cleanroom/fixtures/cloth/`](../../../docs/3D/cleanroom/fixtures/cloth/README.md)),
//! corridos pela NOSSA lei ([`ph2d_cloth::verlet_gesto`]) e comparados vértice a
//! vértice.
//!
//! ⚠️ **É o primeiro instrumento desta linha com um LADO APROVADO.** Toda régua
//! anterior (`espinho`/`rasgo`/`estica`/`ondula`) comparava os nossos resultados
//! uns com os outros e elegeu como «melhor» a célula que o dono chamou de pior
//! ([auditoria §8-quinquies](../../../docs/3D/cloth/03_auditoria_2026-09-05.md)).
//! Aqui o outro lado é a saída do binário da referência sobre a mesma malha.
//!
//! ⚠️ **As fixtures são DADOS, não expressão** (GPLv2 §0: a saída do programa só
//! é obra derivada se o seu conteúdo o for — posições de uma malha nossa não são).
//! O I lê-as; regenerá-las é acto do E.

use ph2d_cloth::V3;
use ph2d_cloth::verlet::{Solver, dist};
use ph2d_cloth::verlet_gesto::{Area, Curva, FalloffForca, Modo, Passo, Pincel, PincelTecido};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/cloth")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da do `ph2d-quadfill`:
/// o cabeçalho do gzip tem tamanho variável e o rabo são oito bytes.
fn inflar(nome: &str) -> String {
    let raw = std::fs::read(fixture_dir().join(nome)).unwrap_or_else(|e| panic!("{nome}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{nome}: nao e' gzip"
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        let xlen = usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8);
        off += 2 + xlen;
    }
    if flg & 0x08 != 0 {
        while raw[off] != 0 {
            off += 1;
        }
        off += 1;
    }
    if flg & 0x10 != 0 {
        while raw[off] != 0 {
            off += 1;
        }
        off += 1;
    }
    if flg & 0x02 != 0 {
        off += 2;
    }
    let bytes = miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8])
        .unwrap_or_else(|e| panic!("{nome}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

fn v3(campos: &[&str]) -> V3 {
    [
        campos[0].parse().expect("x"),
        campos[1].parse().expect("y"),
        campos[2].parse().expect("z"),
    ]
}

/// As posições de repouso de uma superfície.
fn repouso(superficie: &str) -> Vec<V3> {
    inflar(&format!("{superficie}.repouso.txt.gz"))
        .lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| v3(&l.split_whitespace().skip(1).collect::<Vec<_>>()))
        .collect()
}

/// Um traço do oráculo.
struct Traco {
    chaves: BTreeMap<String, String>,
    caminho: Vec<V3>,
    depois: Vec<V3>,
}

fn traco(nome: &str) -> Traco {
    let texto = inflar(&format!("{nome}.deformado.txt.gz"));
    let mut chaves = BTreeMap::new();
    let (mut caminho, mut depois) = (Vec::new(), Vec::new());
    for l in texto.lines() {
        if l.starts_with('#') || l.trim().is_empty() {
            continue;
        }
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos[0] {
            "c" => caminho.push(v3(&campos[1..])),
            "d" => depois.push(v3(&campos[1..])),
            k => {
                if campos.len() >= 2 {
                    chaves.insert(k.to_string(), campos[1].to_string());
                }
            }
        }
    }
    Traco {
        chaves,
        caminho,
        depois,
    }
}

impl Traco {
    fn s(&self, k: &str) -> &str {
        self.chaves.get(k).map_or("", String::as_str)
    }
    fn f(&self, k: &str) -> f64 {
        self.s(k).parse().unwrap_or_else(|_| panic!("chave {k}"))
    }
    fn pincel(&self) -> Pincel {
        Pincel {
            modo: match self.s("modo") {
                "arrastar" => Modo::Arrastar,
                "empurrar" => Modo::Empurrar,
                "apertar_ponto" => Modo::ApertarPonto,
                "apertar_linha" => Modo::ApertarLinha,
                "inflar" => Modo::Inflar,
                "agarrar" => Modo::Agarrar,
                "gancho" => Modo::Gancho,
                "expandir" => Modo::Expandir,
                m => panic!("modo {m}"),
            },
            area: match self.s("area") {
                "local" => Area::Local,
                "global" => Area::Global,
                "dinamica" => Area::Dinamica,
                a => panic!("area {a}"),
            },
            falloff_forca: match self.s("falloff_da_forca") {
                "radial" => FalloffForca::Radial,
                "plano" => FalloffForca::Plano,
                f => panic!("falloff {f}"),
            },
            curva: match self.s("curva") {
                "smooth" => Curva::Suave,
                "sharp" => Curva::Aguda,
                "constant" => Curva::Constante,
                c => panic!("curva {c}"),
            },
            raio: self.f("raio"),
            forca: self.f("forca"),
            dureza: 0.0,
            limite: self.f("limite"),
            banda: self.f("banda"),
            pino: self.f("pino") > 0.5,
            flip: 1.0,
            // As fixtures do oráculo correm SEM simetria ⇒ uma passagem, e a
            // área *Local* constrói a lista `passagens + 1 = 2` vezes.
            passagens: 1,
            escala_phi: std::env::var("PH2D_ESC_PHI")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            escala_retencao: std::env::var("PH2D_ESC_RET")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            solver: Solver {
                massa: self.f("massa"),
                amortecimento: self.f("amortecimento"),
                plasticidade: self.f("plasticidade"),
                // Experimento (`PH2D_VARREDURAS`): quantas varreduras por passo.
                varreduras: std::env::var("PH2D_VARREDURAS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(ph2d_cloth::verlet::VARREDURAS),
            },
        }
    }
}

/// As faces de uma superfície, em índices do FICHEIRO de repouso.
fn faces(superficie: &str, rest: &[V3]) -> Vec<Vec<u32>> {
    match superficie {
        "plano" => {
            // Detecção da grelha: as coordenadas distintas de x e de y, por ordem.
            let mut xs: Vec<f64> = rest.iter().map(|p| p[0]).collect();
            let mut ys: Vec<f64> = rest.iter().map(|p| p[1]).collect();
            xs.sort_by(f64::total_cmp);
            ys.sort_by(f64::total_cmp);
            xs.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            ys.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            let ix = |x: f64| {
                xs.iter()
                    .position(|q| (q - x).abs() < 1e-6)
                    .expect("x na grelha")
            };
            let iy = |y: f64| {
                ys.iter()
                    .position(|q| (q - y).abs() < 1e-6)
                    .expect("y na grelha")
            };
            let mut em = vec![u32::MAX; xs.len() * ys.len()];
            for (v, p) in rest.iter().enumerate() {
                em[iy(p[1]) * xs.len() + ix(p[0])] = u32::try_from(v).expect("u32");
            }
            let id = |i: usize, j: usize| em[j * xs.len() + i];
            // Experimento (`PH2D_TRI=1`): a grelha TRIANGULADA — cada quad partido
            // pela mesma diagonal — para medir se o anel-1 do oráculo inclui a
            // diagonal do quad (a espec §3.1 diz que não; a medição decide).
            let mut f = Vec::new();
            for j in 0..ys.len() - 1 {
                for i in 0..xs.len() - 1 {
                    f.push(vec![id(i, j), id(i + 1, j), id(i + 1, j + 1), id(i, j + 1)]);
                }
            }
            let f = triangular(f);
            assert!(
                f.iter().all(|q| q.iter().all(|v| *v != u32::MAX)),
                "grelha com buraco"
            );
            f
        }
        "esfera" => {
            // A nossa esfera UV, casada por POSIÇÃO com o ficheiro de repouso.
            // ⚠️ A nossa é Y-up e a do oráculo é Z-up: a rotação de +90° em X
            // (`(x, y, z) → (x, −z, y)`) leva o pólo de `(0, r, 0)` a `(0, 0, r)`
            // e PRESERVA a orientação das faces (uma troca de eixos espelharia
            // as normais para dentro). A chave é a `1e-3`, porque vértices
            // vizinhos distam `> 0,05` e o `f32` da nossa esfera contra as seis
            // decimais do ficheiro não sobrevive a `1e-4` nas fronteiras de
            // arredondamento.
            // ⚠️ Medido: uma chave exacta a `1e-3` casa 6 046 de 6 050 — os quatro
            // que faltam estão na fronteira de arredondamento. ⇒ células de
            // `1e-2` e procura nas 27 vizinhas, aceitando a `< 1e-3`.
            let celula = |p: V3| {
                (
                    (p[0] * 1e2).floor() as i64,
                    (p[1] * 1e2).floor() as i64,
                    (p[2] * 1e2).floor() as i64,
                )
            };
            let mut alvo: BTreeMap<(i64, i64, i64), Vec<u32>> = BTreeMap::new();
            for (v, p) in rest.iter().enumerate() {
                alvo.entry(celula(*p))
                    .or_default()
                    .push(u32::try_from(v).expect("u32"));
            }
            let mais_perto = |q: V3| -> Option<u32> {
                let (cx, cy, cz) = celula(q);
                let mut melhor: Option<(f64, u32)> = None;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            if let Some(lista) = alvo.get(&(cx + dx, cy + dy, cz + dz)) {
                                for &v in lista {
                                    let d = dist(q, rest[v as usize]);
                                    if d < 1e-3 && melhor.is_none_or(|(m, _)| d < m) {
                                        melhor = Some((d, v));
                                    }
                                }
                            }
                        }
                    }
                }
                melhor.map(|(_, v)| v)
            };
            for (rings, segs) in [(64usize, 96usize), (96, 64)] {
                let m = ph2d_mesh::shapes::uv_sphere(rings, segs, 1.0);
                if m.vert_count() != rest.len() {
                    continue;
                }
                let mapa: Option<Vec<u32>> = m
                    .positions()
                    .iter()
                    .map(|p| mais_perto([f64::from(p[0]), -f64::from(p[2]), f64::from(p[1])]))
                    .collect();
                if let Some(mapa) = mapa {
                    return triangular(
                        m.faces()
                            .iter()
                            .map(|f| f.verts().iter().map(|v| mapa[*v as usize]).collect())
                            .collect(),
                    );
                }
            }
            panic!(
                "nenhuma esfera UV nossa casa por posicao com o repouso de {} vertices",
                rest.len()
            );
        }
        s => panic!("superficie {s}"),
    }
}

/// **TRIANGULA os quads** (`PH2D_TRI`): `1` = diagonal do 1.º ao 3.º canto ·
/// `2` = do 2.º ao 4.º · `0`/ausente = quads intactos.
///
/// ⭐ Medido em 2026-09-06: a ordem das restrições mal mexe (`0,55`–`0,64` no
/// centro, em cinco ordens), e a triangulação leva o `Local` de `0,60` para
/// `0,35` contra `0,33` do oráculo — **o anel-1 do alvo é o da malha
/// TRIANGULADA**, e a leitura «4 + 2 + 4 por vértice» da espec §3.1 descrevia
/// os quads que o alvo não vê. Registado no INBOX para o E emendar a espec.
fn triangular(faces: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
    let modo = std::env::var("PH2D_TRI")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);
    if modo == 0 {
        return faces;
    }
    let mut out = Vec::with_capacity(faces.len() * 2);
    for f in faces {
        if f.len() == 4 {
            let (a, b, c, d) = (f[0], f[1], f[2], f[3]);
            if modo == 1 {
                out.push(vec![a, b, c]);
                out.push(vec![a, c, d]);
            } else {
                out.push(vec![a, b, d]);
                out.push(vec![b, c, d]);
            }
        } else {
            out.push(f);
        }
    }
    out
}

/// O anel-1 por ARESTAS (espec §3.1), de uma lista de faces.
fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        for k in 0..f.len() {
            let (p, q) = (f[k] as usize, f[(k + 1) % f.len()] as usize);
            a[p].push(q as u32);
            a[q].push(p as u32);
        }
    }
    for l in &mut a {
        l.sort_unstable();
        l.dedup();
    }
    a
}

/// Normais por vértice, ponderadas pela área, das posições ACTUAIS.
fn normais(pos: &[V3], faces: &[Vec<u32>]) -> Vec<V3> {
    let mut n = vec![[0.0f64; 3]; pos.len()];
    for f in faces {
        // Newell: normal de um polígono qualquer, com área embutida.
        let mut fnrm = [0.0f64; 3];
        for k in 0..f.len() {
            let (a, b) = (pos[f[k] as usize], pos[f[(k + 1) % f.len()] as usize]);
            fnrm[0] += (a[1] - b[1]) * (a[2] + b[2]);
            fnrm[1] += (a[2] - b[2]) * (a[0] + b[0]);
            fnrm[2] += (a[0] - b[0]) * (a[1] + b[1]);
        }
        for v in f {
            for c in 0..3 {
                n[*v as usize][c] += fnrm[c];
            }
        }
    }
    for v in &mut n {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 0.0 {
            *v = [v[0] / l, v[1] / l, v[2] / l];
        }
    }
    n
}

/// Experimento (`PH2D_ORDEM`): a ORDEM de resolução das restrições — `indice`
/// (a de criação, vértice a vértice), `inversa`, ou `celula:<tamanho>` (por
/// célula espacial do vértice de origem, em ordem de varrimento, que é a
/// família da ordem do oráculo — espec §3.1: «célula a célula, vértice a
/// vértice»). A dispersão entre ordens NOSSAS é a barra do gate 15 (espec §14).
fn reordenar(sim: &mut ph2d_cloth::verlet::Verlet) {
    // Experimento (`PH2D_PARES=0`): SÓ as restrições de aresta (sem os pares de
    // vizinhos), para medir o que os pares compram.
    if std::env::var("PH2D_PARES").as_deref() == Ok("0") {
        let rep = sim.repouso.clone();
        // Uma aresta liga vizinhos do anel-1: a distância de repouso é a de uma
        // aresta da malha, que é a MENOR distância entre dois vértices ligados.
        // Como o harness não guarda o anel aqui, o critério é geométrico: manter
        // só as restrições cujo comprimento é o de uma aresta (≤ 1,05 × a menor).
        let menor = sim
            .restricoes
            .iter()
            .map(|r| r.l)
            .filter(|l| *l > 0.0)
            .fold(f64::MAX, f64::min);
        let _ = rep;
        sim.restricoes.retain(|r| {
            !matches!(r.b, ph2d_cloth::verlet::Alvo::Vertice(_)) || r.l <= menor * 1.05
        });
    }
    let Ok(ordem) = std::env::var("PH2D_ORDEM") else {
        return;
    };
    if ordem == "inversa" {
        sim.restricoes.reverse();
        return;
    }
    if let Some(t) = ordem.strip_prefix("celula:") {
        let tam: f64 = t.parse().expect("tamanho da celula");
        let rep = sim.repouso.clone();
        sim.restricoes.sort_by_key(|r| {
            let p = rep[r.a as usize];
            let cx = (p[0] / tam).floor() as i64;
            let cy = (p[1] / tam).floor() as i64;
            let cz = (p[2] / tam).floor() as i64;
            // Serpentina em x dentro de cada linha de células, para a varredura
            // não saltar de uma ponta à outra a cada linha.
            let sx = if cy.rem_euclid(2) == 0 { cx } else { -cx };
            (cz, cy, sx)
        });
    }
}

/// O que a comparação devolve, por traço.
struct Leitura {
    movidos_nos: usize,
    movidos_oraculo: usize,
    max_nos: f64,
    max_oraculo: f64,
    erro_max: f64,
    erro_rms: f64,
}

/// Corre a NOSSA lei sobre o traço e compara com a saída do oráculo.
/// **Corre a NOSSA lei sobre o traço e devolve as POSIÇÕES finais.** Extraída
/// de [`correr`] para que uma sonda possa medir outra grandeza sobre a mesma
/// corrida sem reescrever o laço — *duas cópias do laço seriam duas leis.*
fn correr_posicoes(nome: &str) -> Vec<V3> {
    let t = traco(nome);
    let sup = t.s("superficie").to_string();
    let rest = repouso(&sup);
    assert_eq!(
        rest.len(),
        t.depois.len(),
        "{nome}: repouso e deformado nao batem"
    );
    let fs = faces(&sup, &rest);
    let an = aneis(rest.len(), &fs);
    let anel = |v: u32| an[v as usize].clone();
    let pincel = t.pincel();
    let passos = t.f("passos") as usize;
    assert_eq!(
        passos,
        t.caminho.len(),
        "{nome}: passos != pontos do caminho"
    );

    let mut pos = rest.clone();
    let mut tecido = PincelTecido::pen_down(pincel, &pos, t.caminho[0]);
    for k in 0..passos {
        let cursor = t.caminho[k];
        let prev = t.caminho[k.saturating_sub(1)];
        let delta = if pincel.modo == Modo::Agarrar {
            let c0 = t.caminho[0];
            [cursor[0] - c0[0], cursor[1] - c0[1], cursor[2] - c0[2]]
        } else {
            [
                cursor[0] - prev[0],
                cursor[1] - prev[1],
                cursor[2] - prev[2],
            ]
        };
        let parado = k == 0 || dist(cursor, prev) == 0.0;
        let nrm = normais(&pos, &fs);
        // A normal da ÁREA: a média das normais sob o pincel (espec §4.4).
        let mut na = [0.0f64; 3];
        for (v, p) in pos.iter().enumerate() {
            if dist(*p, cursor) < pincel.raio {
                for c in 0..3 {
                    na[c] += nrm[v][c];
                }
            }
        }
        let passo = Passo {
            cursor,
            delta,
            parado,
            normal_area: na,
            normais: &nrm,
            pressao: 1.0,
        };
        let simulou = tecido.passo(&pos, &anel, &passo);
        if k == 0 {
            reordenar(&mut tecido.sim);
        }
        if simulou {
            for (v, act) in tecido.sim.activo.iter().enumerate() {
                if *act {
                    pos[v] = tecido.sim.x[v];
                }
            }
        }
    }

    pos
}

/// As posições que o ORÁCULO gravou para este traço.
fn deformado(nome: &str) -> Vec<V3> {
    traco(nome).depois
}

/// Corre a NOSSA lei sobre o traço e compara com a saída do oráculo.
fn correr(nome: &str) -> Leitura {
    let t = traco(nome);
    let sup = t.s("superficie").to_string();
    let rest = repouso(&sup);
    assert_eq!(
        rest.len(),
        t.depois.len(),
        "{nome}: repouso e deformado nao batem"
    );
    let pos = correr_posicoes(nome);
    let (mut movidos_nos, mut movidos_oraculo) = (0usize, 0usize);
    let (mut max_nos, mut max_oraculo, mut erro_max, mut soma2) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for v in 0..rest.len() {
        let un = dist(rest[v], pos[v]);
        let uo = dist(rest[v], t.depois[v]);
        if un > 1e-5 {
            movidos_nos += 1;
        }
        if uo > 1e-5 {
            movidos_oraculo += 1;
        }
        max_nos = max_nos.max(un);
        max_oraculo = max_oraculo.max(uo);
        let e = dist(pos[v], t.depois[v]);
        erro_max = erro_max.max(e);
        soma2 += e * e;
    }
    Leitura {
        movidos_nos,
        movidos_oraculo,
        max_nos,
        max_oraculo,
        erro_max,
        erro_rms: (soma2 / rest.len() as f64).sqrt(),
    }
}

/// **SONDA — o PERFIL ao longo do eixo do traço**, nosso contra o oráculo, para
/// UM traço (`PH2D_TRACO`, omissão `plano_arrastar_radial_local`).
#[test]
#[ignore = "sonda"]
fn sonda_do_perfil() {
    let nome = std::env::var("PH2D_TRACO").unwrap_or_else(|_| "plano_arrastar_radial_local".into());
    let t = traco(&nome);
    let sup = t.s("superficie").to_string();
    let rest = repouso(&sup);
    let fs = faces(&sup, &rest);
    let an = aneis(rest.len(), &fs);
    let anel = |v: u32| an[v as usize].clone();
    let pincel = t.pincel();
    let mut pos = rest.clone();
    let mut tecido = PincelTecido::pen_down(pincel, &pos, t.caminho[0]);
    for k in 0..t.caminho.len() {
        let cursor = t.caminho[k];
        let prev = t.caminho[k.saturating_sub(1)];
        let delta = [
            cursor[0] - prev[0],
            cursor[1] - prev[1],
            cursor[2] - prev[2],
        ];
        let nrm = normais(&pos, &fs);
        let passo = Passo {
            cursor,
            delta,
            parado: k == 0,
            normal_area: [0.0, 0.0, 1.0],
            normais: &nrm,
            pressao: 1.0,
        };
        let simulou = tecido.passo(&pos, &anel, &passo);
        if k == 0 {
            reordenar(&mut tecido.sim);
        }
        if simulou {
            for (v, act) in tecido.sim.activo.iter().enumerate() {
                if *act {
                    pos[v] = tecido.sim.x[v];
                }
            }
        }
    }
    println!(
        "{nome}: restricoes={} activos={}",
        tecido.sim.restricoes.len(),
        tecido.dentro.len()
    );
    println!(
        "{:>8} {:>8} {:>8} {:>6} {:>6}",
        "x", "nos", "oraculo", "phi", "w0"
    );
    let mut linha: Vec<usize> = (0..rest.len())
        .filter(|v| rest[*v][1].abs() < 1e-6)
        .collect();
    linha.sort_by(|a, b| rest[*a][0].total_cmp(&rest[*b][0]));
    for v in linha {
        if rest[v][0] < -0.8 || rest[v][0] > 1.1 {
            continue;
        }
        println!(
            "{:>8.3} {:>8.4} {:>8.4} {:>6.3} {:>6.3}",
            rest[v][0],
            dist(rest[v], pos[v]),
            dist(rest[v], t.depois[v]),
            tecido.sim.phi[v],
            tecido.sim.w_repouso[v]
        );
    }
}

/// **SONDA — a tensão numa CADEIA com parede**: 40 vértices numa linha, aresta
/// `0,05`, a ponta `0` com `φ = 0` (a parede), força constante no vértice `20`
/// durante 11 passos. Sem parede (`φ ≡ 1`) o mesmo. Se a parede não reduzir o
/// deslocamento do `20`, a tensão não atravessa a cadeia em 5 varreduras.
#[test]
#[ignore = "sonda"]
fn sonda_da_cadeia_com_parede() {
    use ph2d_cloth::verlet::{Solver, Verlet};
    for parede in [false, true] {
        let n = 40usize;
        let rest: Vec<V3> = (0..n).map(|i| [i as f64 * 0.05, 0.0, 0.0]).collect();
        let mut sim = Verlet::nascer(rest.clone());
        for v in 0..n as u32 {
            let mut anel = Vec::new();
            if v > 0 {
                anel.push(v - 1);
            }
            if (v as usize) + 1 < n {
                anel.push(v + 1);
            }
            sim.construir(v, &anel);
        }
        for i in 0..n {
            sim.activo[i] = true;
            sim.phi[i] = if parede && i == 0 { 0.0 } else { 1.0 };
            sim.w_repouso[i] = 1.0;
        }
        let solver = Solver::default();
        let mut pos = rest.clone();
        for _ in 0..11 {
            sim.x.copy_from_slice(&pos);
            sim.a[20] = [10.0, 0.0, 0.0];
            sim.passo(&solver);
            for (i, p) in pos.iter_mut().enumerate() {
                if sim.activo[i] {
                    *p = sim.x[i];
                }
            }
        }
        let d: Vec<String> = [0usize, 5, 10, 15, 20, 25, 30, 39]
            .iter()
            .map(|i| format!("{}:{:+.4}", i, pos[*i][0] - rest[*i][0]))
            .collect();
        println!("parede={parede}  {}", d.join("  "));
    }
}

/// Um dump POR PASSO do oráculo: o caminho e os blocos de posições DEPOIS de
/// cada passo (`passo k` + `N` linhas `d`).
struct PorPasso {
    caminho: Vec<V3>,
    blocos: Vec<Vec<V3>>,
}

fn por_passo(nome: &str) -> PorPasso {
    let texto = inflar(&format!("{nome}.porpasso.txt.gz"));
    let (mut caminho, mut blocos): (Vec<V3>, Vec<Vec<V3>>) = (Vec::new(), Vec::new());
    for l in texto.lines() {
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos.first().copied() {
            Some("c") => caminho.push(v3(&campos[1..])),
            Some("passo") => blocos.push(Vec::new()),
            Some("d") => blocos
                .last_mut()
                .expect("bloco antes de d")
                .push(v3(&campos[1..])),
            _ => {}
        }
    }
    PorPasso { caminho, blocos }
}

/// **SONDA — PASSO A PASSO contra o oráculo**, para UM traço (`PH2D_TRACO`).
///
/// ⭐ É a régua «fase a fase onde houver dumps» do §7.2 da skill: a comparação
/// final diz QUANTO diverge; esta diz EM QUE PASSO a divergência nasce, e em
/// que vértice (sob o cursor inicial · a 1R e 2R dele, perpendicular ao traço ·
/// no início, meio e limite da banda · sob o cursor de cada passo).
/// ⚠️ Um bloco cujas posições sejam o REPOUSO é um glitch do arnês do oráculo
/// e é saltado com aviso — não é «erro zero».
#[test]
#[ignore = "sonda"]
fn sonda_passo_a_passo() {
    let nome = std::env::var("PH2D_TRACO").unwrap_or_else(|_| "plano_arrastar_radial_local".into());
    let pp = por_passo(&nome);
    let t = traco(&nome);
    let sup = t.s("superficie").to_string();
    let rest = repouso(&sup);
    let fs = faces(&sup, &rest);
    let an = aneis(rest.len(), &fs);
    let anel = |v: u32| an[v as usize].clone();
    let pincel = t.pincel();
    let c0 = pp.caminho[0];
    let r = pincel.raio;
    // Os vértices nomeados: o mais próximo de cada ponto de referência.
    let perto = |p: V3| -> usize {
        (0..rest.len())
            .min_by(|a, b| dist(rest[*a], p).total_cmp(&dist(rest[*b], p)))
            .expect("malha vazia")
    };
    let nomeados: Vec<(String, usize)> = [
        ("cursor0", 0.0),
        ("1R", 1.0),
        ("2R", 2.0),
        ("2.875R", 2.875),
        ("3.2R", 3.2),
        ("3.5R", 3.5),
        ("4R", 4.0),
    ]
    .iter()
    .map(|(n, k)| ((*n).to_string(), perto([c0[0], c0[1] + k * r, c0[2]])))
    .collect();

    let mut pos = rest.clone();
    let mut tecido = PincelTecido::pen_down(pincel, &pos, c0);
    // ⭐⭐ **As colunas nomeadas são as do anel PRÓXIMO (`1R`, `2R`)**, e a troca
    // é de 06/09: um defeito que vive num vértice só não se vê nas colunas do
    // ARO, que são justamente onde a lei já bate. O aperto de ponto lê `1R`
    // `0,02102` contra `0,02096` do oráculo e `c0` `0,0975` contra `0,1842` —
    // *a vizinhança inteira concorda a cinco casas e discorda UM vértice.*
    println!(
        "{nome}: {} passos, {} blocos",
        pp.caminho.len(),
        pp.blocos.len()
    );
    // ⭐ As colunas do ARO são o discriminador Local/Global que o oráculo entregou
    // em 06/09: no Local o `3,5R` mexe `≤0,0003` e o `4R` é **zero exacto** (o aro
    // é âncora, porque o raio de construção é o LIMITE e ali `w = 0`); no Global
    // os dois passam de `0,03`. *Um port cujo aro se mexe no Local tem o aro
    // livre, e é isso que faz o Local render como o Global.*
    println!(
        "{:>4} | {:>7} {:>7} | {:>7} {:>7} | {:>7} {:>7} | {:>7} {:>7} | {:>8} {:>8}",
        "k",
        "c0 nos",
        "c0 orac",
        "1R nos",
        "1R or",
        "2R nos",
        "2R or",
        "4R nos",
        "4R orac",
        "max nos",
        "max orac"
    );
    println!("      (as duas ultimas colunas sao a distancia do PICO ao cursor, em raios)");
    for k in 0..pp.caminho.len() {
        let cursor = pp.caminho[k];
        let prev = pp.caminho[k.saturating_sub(1)];
        let delta = if pincel.modo == Modo::Agarrar {
            [cursor[0] - c0[0], cursor[1] - c0[1], cursor[2] - c0[2]]
        } else {
            [
                cursor[0] - prev[0],
                cursor[1] - prev[1],
                cursor[2] - prev[2],
            ]
        };
        let nrm = normais(&pos, &fs);
        let mut na = [0.0f64; 3];
        for (v, p) in pos.iter().enumerate() {
            if dist(*p, cursor) < r {
                for c in 0..3 {
                    na[c] += nrm[v][c];
                }
            }
        }
        let passo = Passo {
            cursor,
            delta,
            parado: k == 0,
            normal_area: na,
            normais: &nrm,
            pressao: 1.0,
        };
        let simulou = tecido.passo(&pos, &anel, &passo);
        if k == 0 {
            reordenar(&mut tecido.sim);
        }
        if simulou {
            for (v, act) in tecido.sim.activo.iter().enumerate() {
                if *act {
                    pos[v] = tecido.sim.x[v];
                }
            }
        }
        let Some(bloco) = pp.blocos.get(k) else {
            continue;
        };
        let glitch = bloco.iter().zip(&rest).all(|(a, b)| a == b) && k > 0;
        let u_n = |v: usize| dist(rest[v], pos[v]);
        let u_o = |v: usize| dist(rest[v], bloco[v]);
        let ck = perto(cursor);
        let (mut max_n, mut max_o, mut err) = (0.0f64, 0.0f64, 0.0f64);
        // ⭐ ONDE está o pico, não só quanto ele vale: um modo cuja amplitude
        // bate e cujo `arg max` está noutro sítio tem defeito de LUGAR, e as
        // colunas de amplitude são cegas a ele (medido no Snake Hook, 06/09).
        let (mut arg_n, mut arg_o) = (0usize, 0usize);
        for v in 0..rest.len() {
            if u_n(v) > max_n {
                max_n = u_n(v);
                arg_n = v;
            }
            if u_o(v) > max_o {
                max_o = u_o(v);
                arg_o = v;
            }
            err = err.max(dist(pos[v], bloco[v]));
        }
        let _ = ck;
        // A distância do pico ao cursor DESTE passo, em raios de pincel.
        let (pico_n, pico_o) = (dist(rest[arg_n], cursor) / r, dist(rest[arg_o], cursor) / r);
        let (c0v, bnd, lim, fora) = (nomeados[0].1, nomeados[1].1, nomeados[2].1, nomeados[6].1);
        // ⭐ Onde o vértice do PEN-DOWN está AGORA, em raios: um vértice que se
        // encostou ao cursor deixa de ter direcção de puxão.
        let (dc_n, dc_o) = (dist(pos[c0v], cursor) / r, dist(bloco[c0v], cursor) / r);
        // ⭐ O ANEL IMEDIATO de `c0` (a coluna `1R` está a ~4 células daqui): é
        // contra ele que a relaxação puxa o vértice de volta.
        let (mut an_n, mut an_o, mut nn) = (0.0f64, 0.0f64, 0usize);
        for &w in &an[c0v] {
            let w = w as usize;
            an_n += u_n(w);
            an_o += u_o(w);
            nn += 1;
        }
        let (an_n, an_o) = (an_n / nn as f64, an_o / nn as f64);
        // ⭐ A componente FORA DO PLANO do pen-down (a superfície plana é z = 0):
        // uma folha que dobra sob o aperto move-se em z; uma que só desliza não.
        let (uz_n, uz_o) = (pos[c0v][2] - rest[c0v][2], bloco[c0v][2] - rest[c0v][2]);
        let _ = (uz_n, uz_o);
        let vn = [
            pos[c0v][0] - rest[c0v][0],
            pos[c0v][1] - rest[c0v][1],
            pos[c0v][2] - rest[c0v][2],
        ];
        let vo = [
            bloco[c0v][0] - rest[c0v][0],
            bloco[c0v][1] - rest[c0v][1],
            bloco[c0v][2] - rest[c0v][2],
        ];
        let dcur = [
            cursor[0] - rest[c0v][0],
            cursor[1] - rest[c0v][1],
            cursor[2] - rest[c0v][2],
        ];
        println!(
            "{:>4} | {:>7.4} {:>7.4} | {:>8.5} {:>8.5} | {:>8.5} {:>8.5} | {:>7.5} {:>7.5} | {:>8.4} {:>8.4} | {:>5.2}R {:>5.2}R | ERRO {:>8.5} | anel {:>7.4} {:>7.4} | c0a {:>5.2}R {:>5.2}R | u_nos [{:>7.4} {:>7.4} {:>7.4}] u_or [{:>7.4} {:>7.4} {:>7.4}] cursor-rest [{:>7.4} {:>7.4} {:>7.4}]{}",
            k + 1,
            u_n(c0v),
            u_o(c0v),
            u_n(bnd),
            u_o(bnd),
            u_n(lim),
            u_o(lim),
            u_n(fora),
            u_o(fora),
            max_n,
            max_o,
            pico_n,
            pico_o,
            err,
            an_n,
            an_o,
            dc_n,
            dc_o,
            vn[0],
            vn[1],
            vn[2],
            vo[0],
            vo[1],
            vo[2],
            dcur[0],
            dcur[1],
            dcur[2],
            if glitch {
                "  (bloco = repouso: glitch do dump, ignorar)"
            } else {
                ""
            }
        );
    }
}

fn todas() -> Vec<String> {
    let mut nomes: Vec<String> = std::fs::read_dir(fixture_dir())
        .expect("fixtures/cloth")
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.strip_suffix(".deformado.txt.gz").map(str::to_string)
        })
        .collect();
    nomes.sort();
    nomes
}

/// ⭐⭐⭐ **GATE — os traços de UM passo de força saem AO BIT do oráculo.**
///
/// Sete traços: os seis modos de força (arrastar · apertar_ponto · apertar_linha
/// · empurrar · inflar · forca05) e a massa `2`. Num passo só, as restrições
/// ainda não responderam (espec §5.2: a relaxação vem antes da integração, e no
/// 1.º passo simulado corre sobre a malha em repouso) ⇒ o deslocamento é a lei
/// da força PURA: `f · B · dt / massa` na direcção do modo. **Não há solver a
/// esconder um erro de força aqui**, e é por isso que estes sete são a
/// fundação: a curva de queda, o `10·α`, o `2R` do Push, a normal do Inflate e
/// o ganho inverso da massa.
///
/// ⚠️ **A barra é a PRECISÃO DO FICHEIRO, não um epsilon de conforto:** as
/// fixtures trazem seis decimais (piso de arredondamento `5e-7`); `1e-5` é `20×`
/// o piso. ⛔ O bug da massa contada duas vezes lia `0,0248` contra `0,0496`
/// (erro `0,025`) — `2 500×` esta barra.
#[test]
fn os_tracos_de_um_passo_de_forca_saem_ao_bit() {
    const UM_PASSO: [&str; 7] = [
        "plano_arrastar_radial_local_1passo",
        "plano_apertar_ponto_radial_local_1passo",
        "plano_apertar_linha_radial_local_1passo",
        "plano_empurrar_radial_local_1passo",
        "plano_inflar_radial_local_1passo",
        "plano_arrastar_radial_local_forca05_1passo",
        "plano_arrastar_radial_local_massa2_1passo",
    ];
    for nome in UM_PASSO {
        let l = correr(nome);
        // Controlo anti-vácuo: o traço tem de ter MOVIDO alguma coisa nos dois
        // lados, senão `0 == 0` aprovaria um pincel morto.
        assert!(
            l.movidos_oraculo > 100 && l.movidos_nos > 100,
            "{nome}: movidos {} (nos) / {} (oraculo) -- vacuo",
            l.movidos_nos,
            l.movidos_oraculo
        );
        assert!(
            l.erro_max <= 1e-5,
            "{nome}: pior erro por vertice {:.3e} contra a precisao do ficheiro \
             (barra 1e-5); max nosso {:.5} / oraculo {:.5}",
            l.erro_max,
            l.max_nos,
            l.max_oraculo
        );
    }
}

/// **SONDA — a tabela de paridade dos 46 traços.** `erro/max` é o pior erro por
/// vértice sobre o maior deslocamento do oráculo: `0` seria o bit, `1` seria não
/// ter feito nada.
#[test]
#[ignore = "sonda"]
fn sonda_da_paridade_com_o_oraculo() {
    println!(
        "{:<46} {:>6} {:>6} | {:>8} {:>8} | {:>8} {:>8} {:>7}",
        "traco", "mov_n", "mov_o", "max_n", "max_o", "err_max", "err_rms", "err/max"
    );
    for nome in todas() {
        let l = correr(&nome);
        println!(
            "{:<46} {:>6} {:>6} | {:>8.4} {:>8.4} | {:>8.4} {:>8.4} {:>7.3}",
            nome,
            l.movidos_nos,
            l.movidos_oraculo,
            l.max_nos,
            l.max_oraculo,
            l.erro_max,
            l.erro_rms,
            l.erro_max / l.max_oraculo.max(1e-12)
        );
    }
}

/// A barra da PARIDADE (espec §14 gate 15) — o pior erro por vértice de um
/// traço, em unidades do deslocamento máximo do oráculo.
///
/// ⚠️ **Ela sai de um VALE MEDIDO, não de conforto:** com a lei de 06/09 os 53
/// traços partem-se em `28` com `≤ 0,095` e `25` com `≥ 0,175`, e não há nada
/// entre os dois. `0,13` é o meio desse vazio.
///
/// ⚠️⚠️ **E não se aperta em direcção ao grupo, porque a ORDEM de resolução
/// custa mais do que o resíduo:** inverter a ordem das restrições — que é uma
/// escolha nossa, e Gauss-Seidel não comuta — move a nossa resposta em média
/// `0,0985` e até `0,256` nos mesmos 28 traços. *Estamos a bater o oráculo com
/// folga MENOR do que o ruído de ordenação, o que só é possível se a nossa
/// ordem for a dele; uma barra colada em `0,10` mediria a ordem, não a lei.*
const BARRA_PARIDADE: f64 = 0.13;

/// Os traços que a lei REPRODUZ (espec §14 gate 15).
const PARIDADE: [&str; VERDE_N] = [
    "esfera_arrastar_radial_dinamica",
    "plano_agarrar_radial_local",
    "plano_agarrar_radial_local_1passo",
    "plano_agarrar_radial_local_24passos",
    "plano_agarrar_radial_local_2passos",
    "plano_agarrar_radial_local_2passos_origem",
    "plano_agarrar_radial_local_amort06",
    "plano_agarrar_radial_local_preset",
    "plano_apertar_linha_radial_local_1passo",
    "plano_apertar_ponto_radial_local_1passo",
    "plano_arrastar_radial_dinamica_preset",
    "plano_arrastar_radial_local",
    "plano_arrastar_radial_local_1passo",
    "plano_arrastar_radial_local_2passos",
    "plano_arrastar_radial_local_amort05",
    "plano_arrastar_radial_local_amort1",
    "plano_arrastar_radial_local_forca05",
    "plano_arrastar_radial_local_forca05_1passo",
    "plano_arrastar_radial_local_massa2",
    "plano_arrastar_radial_local_massa2_1passo",
    "plano_arrastar_radial_local_origem",
    "plano_arrastar_radial_local_pino",
    "plano_arrastar_radial_local_plast05",
    "plano_empurrar_radial_local_1passo",
    "plano_gancho_radial_local",
    "plano_gancho_radial_local_24passos",
    "plano_gancho_radial_local_amort06",
    "plano_inflar_radial_local_1passo",
];
const VERDE_N: usize = 28;

/// Os traços AINDA por explicar, com o valor MEDIDO em 2026-09-06 ao lado.
///
/// ⚠️ **A lista tem censo de obsolescência nas DUAS pontas** (CLAUDE.md §5.0:
/// *uma catraca sem censo não desce, vira licença*): um traço daqui que passe a
/// bater é acusado — tem de migrar para [`PARIDADE`] —, e um que se degrade
/// acima da folga também. ⭐ **É o segundo ramo que fecha o gate 17:** um port
/// que dobre a relaxação em TODA a parte passaria o gate 16 e mandaria o
/// `plano_arrastar_radial_global` de `0,301` para `0,583` (medido), o que cai
/// fora da folga e reprova aqui.
const ABERTOS: [(&str, f64); ABERTO_N] = [
    ("esfera_agarrar_radial_dinamica", 0.265),
    ("esfera_apertar_linha_radial_dinamica", 0.588),
    ("esfera_apertar_ponto_radial_dinamica", 0.542),
    ("esfera_empurrar_radial_dinamica", 0.303),
    ("esfera_expandir_radial_dinamica", 0.557),
    ("esfera_gancho_radial_dinamica", 0.351),
    ("esfera_inflar_radial_dinamica", 0.378),
    ("plano_agarrar_plano_local", 0.180),
    ("plano_apertar_linha_radial_local", 1.024),
    ("plano_apertar_linha_radial_local_origem", 0.263),
    ("plano_apertar_ponto_plano_local", 0.613),
    ("plano_apertar_ponto_radial_local", 1.380),
    ("plano_apertar_ponto_radial_local_origem", 1.079),
    ("plano_arrastar_plano_local", 0.233),
    ("plano_arrastar_radial_dinamica", 0.181),
    ("plano_arrastar_radial_global", 0.175),
    ("plano_arrastar_radial_global_origem", 0.301),
    ("plano_empurrar_plano_local", 0.944),
    ("plano_empurrar_radial_local", 0.329),
    ("plano_expandir_radial_local", 0.192),
    ("plano_expandir_radial_local_1passo", 0.560),
    ("plano_gancho_radial_local_1passo", 0.416),
    ("plano_gancho_radial_local_2passos", 0.388),
    ("plano_gancho_radial_local_2passos_origem", 0.420),
    ("plano_inflar_radial_local", 0.253),
];
const ABERTO_N: usize = 25;

/// A folga de regressão sobre o valor medido de um traço ABERTO.
const FOLGA_ABERTO: f64 = 1.25;

/// **GATE — a paridade com o oráculo não regride** (espec §14 gates 15 e 17).
#[test]
fn a_paridade_com_o_oraculo_nao_regride() {
    // Censo: as duas listas juntas TÊM de ser o corpus inteiro. Sem isto, uma
    // fixture nova entra sem régua nenhuma e a suíte fica verde sobre ela.
    let mut nomeados: Vec<String> = PARIDADE.iter().map(|s| (*s).to_string()).collect();
    nomeados.extend(ABERTOS.iter().map(|(s, _)| (*s).to_string()));
    nomeados.sort();
    let mut corpus = todas();
    corpus.sort();
    assert_eq!(
        nomeados, corpus,
        "as listas do gate nao descrevem o corpus -- ha fixture sem regua ou nome morto"
    );

    for nome in PARIDADE {
        let l = correr(nome);
        assert!(
            l.movidos_nos > 100 && l.movidos_oraculo > 100,
            "{nome}: movidos {} / {} -- vacuo",
            l.movidos_nos,
            l.movidos_oraculo
        );
        let e = l.erro_max / l.max_oraculo.max(1e-12);
        assert!(
            e <= BARRA_PARIDADE,
            "{nome}: erro relativo {e:.3} passa a barra {BARRA_PARIDADE} \
             (max nosso {:.4} / oraculo {:.4}; rms {:.4})",
            l.max_nos,
            l.max_oraculo,
            l.erro_rms
        );
    }

    for (nome, medido) in ABERTOS {
        let l = correr(nome);
        let e = l.erro_max / l.max_oraculo.max(1e-12);
        assert!(
            e > BARRA_PARIDADE,
            "{nome}: erro relativo {e:.3} JA' BATE a barra {BARRA_PARIDADE} -- \
             este traco deixou de estar aberto e tem de migrar para PARIDADE"
        );
        assert!(
            e <= medido * FOLGA_ABERTO,
            "{nome}: erro relativo {e:.3} regrediu contra o medido {medido:.3} \
             (folga {FOLGA_ABERTO}x)"
        );
    }
}

/// **GATE — a lista de restrições do *Local* vem em DUPLICADO** (espec §14
/// gate 16, §5.2-bis).
///
/// Duas réguas, e a segunda é a que importa: a contagem de restrições é a
/// estrutura, e a **contagem de VÉRTICES MOVIDOS num traço de âncora de UM
/// passo simulado** é o comportamento — um inteiro, dos dois lados, sem
/// acumulação possível. ⚠️ *Numa relaxação sequencial o alcance por passo É o
/// número de passagens*, então essa contagem mede directamente quantas cópias
/// a lista tem: com uma só ela lê `869` contra `1324` do oráculo.
///
/// ⚠️ **A lei geral é `n + 1` cópias para `n` passagens de simetria** (§5.2-bis);
/// as fixtures correm sem simetria (`n = 1`), logo aqui o factor é `2`. ⛔ Não
/// escreva `2` como se fosse a lei.
#[test]
fn a_lista_do_local_vem_em_duplicado() {
    let t = traco("plano_arrastar_radial_local");
    let sup = t.s("superficie").to_string();
    let rest = repouso(&sup);
    let fs = faces(&sup, &rest);
    let an = aneis(rest.len(), &fs);
    let anel = |v: u32| an[v as usize].clone();
    let normais = vec![[0.0, 0.0, 1.0]; rest.len()];
    let c0 = t.caminho[0];

    let lista_de = |area: Area| -> Vec<ph2d_cloth::verlet::Restricao> {
        let pincel = Pincel { area, ..t.pincel() };
        let mut tecido = PincelTecido::pen_down(pincel, &rest, c0);
        let passo = Passo {
            cursor: c0,
            delta: [0.0; 3],
            parado: true,
            normal_area: [0.0, 0.0, 1.0],
            normais: &normais,
            pressao: 1.0,
        };
        tecido.passo(&rest, &anel, &passo);
        tecido.sim.restricoes.clone()
    };
    // ⭐ A régua da ESTRUTURA não precisa de botão de controlo, e é mais forte
    // que uma contagem: a espec diz «duas cópias IDÊNTICAS, na ordem *a lista
    // inteira, e a seguir a lista inteira outra vez*» — então a lista tem de
    // partir-se ao meio em duas metades iguais, restrição a restrição.
    let lista = lista_de(Area::Local);
    assert!(lista.len() > 1000, "vacuo: {} restricoes", lista.len());
    assert_eq!(lista.len() % 2, 0, "lista impar: {}", lista.len());
    let meio = lista.len() / 2;
    for i in 0..meio {
        let (x, y) = (lista[i], lista[i + meio]);
        assert!(
            x.a == y.a
                && x.b == y.b
                && x.l.to_bits() == y.l.to_bits()
                && x.s.to_bits() == y.s.to_bits(),
            "restricao {i} difere da copia {}: {x:?} contra {y:?} -- a lista do Local \
             tem de ser a mesma lista duas vezes, na mesma ordem",
            i + meio
        );
    }
    // E o outro lado da lei: fora do *Local* a lista vem UMA vez.
    for area in [Area::Global, Area::Dinamica] {
        let p = Pincel { area, ..t.pincel() };
        assert_eq!(
            p.construcoes(),
            1,
            "{area:?} nao pode construir mais de uma vez (espec §14 gate 17)"
        );
    }
    let p = Pincel {
        area: Area::Local,
        passagens: 3,
        ..t.pincel()
    };
    assert_eq!(
        p.construcoes(),
        4,
        "a lei geral e' n+1 copias para n passagens"
    );

    // A régua de COMPORTAMENTO: os inteiros dos dois lados.
    for nome in [
        "plano_agarrar_radial_local_1passo",
        "plano_expandir_radial_local_1passo",
        "plano_arrastar_radial_local_2passos",
    ] {
        let l = correr(nome);
        let dif = (l.movidos_nos as f64 - l.movidos_oraculo as f64).abs()
            / f64::from(u32::try_from(l.movidos_oraculo).unwrap_or(1));
        assert!(
            dif <= 0.02,
            "{nome}: movemos {} vertices contra {} do oraculo ({:.1} %) -- com UMA \
             copia da lista esta conta le 869 contra 1324",
            l.movidos_nos,
            l.movidos_oraculo,
            dif * 100.0
        );
    }
}

/// **GATE — o centro do Snake Hook está UM PASSO atrasado** (espec §14 gate 18,
/// §4.3, §10.4).
///
/// No 1.º passo simulado o vértice mais deslocado é o do **pen-down**, e não o
/// que está sob o cursor: o gancho arrasta o que agarrou em vez de apanhar
/// material novo. ⚠️ **É defeito de LUGAR e toda régua de amplitude é cega a
/// ele** — com o centro no cursor a amplitude fica certa e o pico salta para
/// `0,05R` do cursor, contra `0,86R` do oráculo.
#[test]
fn o_centro_do_snake_hook_esta_um_passo_atrasado() {
    let nome = "plano_gancho_radial_local_2passos_origem";
    let pp = por_passo(nome);
    let t = traco(nome);
    let sup = t.s("superficie").to_string();
    let rest = repouso(&sup);
    let fs = faces(&sup, &rest);
    let an = aneis(rest.len(), &fs);
    let anel = |v: u32| an[v as usize].clone();
    let pincel = t.pincel();
    let r = pincel.raio;
    let c0 = pp.caminho[0];
    let mut pos = rest.clone();
    let mut tecido = PincelTecido::pen_down(pincel, &pos, c0);
    let mut argmax = (0usize, 0.0f64);
    let mut cursor = c0;
    for k in 0..2 {
        cursor = pp.caminho[k];
        let prev = pp.caminho[k.saturating_sub(1)];
        let delta = [
            cursor[0] - prev[0],
            cursor[1] - prev[1],
            cursor[2] - prev[2],
        ];
        let nrm = normais(&pos, &fs);
        let mut na = [0.0f64; 3];
        for (v, p) in pos.iter().enumerate() {
            if dist(*p, cursor) < r {
                for c in 0..3 {
                    na[c] += nrm[v][c];
                }
            }
        }
        let passo = Passo {
            cursor,
            delta,
            parado: k == 0,
            normal_area: na,
            normais: &nrm,
            pressao: 1.0,
        };
        if tecido.passo(&pos, &anel, &passo) {
            for (v, act) in tecido.sim.activo.iter().enumerate() {
                if *act {
                    pos[v] = tecido.sim.x[v];
                }
            }
        }
    }
    for (v, p) in pos.iter().enumerate() {
        let u = dist(rest[v], *p);
        if u > argmax.1 {
            argmax = (v, u);
        }
    }
    // Anti-vácuo: tem de ter deformado.
    assert!(
        argmax.1 > 0.05,
        "o traco nao deformou (max {:.4})",
        argmax.1
    );
    let ao_cursor = dist(rest[argmax.0], cursor) / r;
    let ao_pendown = dist(rest[argmax.0], c0) / r;
    assert!(
        ao_pendown < 0.15,
        "o pico esta a {ao_pendown:.2}R do pen-down -- no 1.º passo simulado ele TEM \
         de ser o vertice do pen-down (espec §4.3)"
    );
    assert!(
        ao_cursor > 0.5,
        "o pico esta a {ao_cursor:.2}R do cursor -- com o centro da queda NO cursor \
         esta conta le ~0,05R, e o oraculo le 0,86R"
    );
}

/// **SONDA — as três grandezas de ARTEFATO, medidas na saída do ORÁCULO.**
///
/// ⚠️ **Existe porque as barras do gate de artefatos da `ph2d-sculpt3d` foram
/// calibradas sobre a lei VBD**, que o dono reprovou três vezes — *uma barra
/// calibrada sem o lado aprovado mede os nossos próprios defeitos*. Agora há
/// lado aprovado: a saída do alvo, para o mesmo traço.
///
/// espinho = o maior deslocamento · rasgo = a maior diferença de deslocamento
/// entre vizinhos de aresta · estica = a maior razão aresta/repouso.
#[test]
#[ignore = "sonda"]
fn sonda_dos_artefatos_do_oraculo() {
    println!(
        "{:<44} | {:>8} {:>8} | {:>8} {:>8} | {:>7} {:>7}",
        "traco", "esp_nos", "esp_orac", "rasg_nos", "rasg_or", "est_nos", "est_or"
    );
    for nome in todas() {
        let t = traco(&nome);
        let sup = t.s("superficie").to_string();
        let rest = repouso(&sup);
        let fs = faces(&sup, &rest);
        let an = aneis(rest.len(), &fs);
        let nosso = correr_posicoes(&nome);
        let orac = deformado(&nome);
        let tres = |p: &[V3]| -> (f64, f64, f64) {
            let d: Vec<f64> = (0..rest.len()).map(|v| dist(rest[v], p[v])).collect();
            let (mut esp, mut rasg, mut est) = (0.0f64, 0.0f64, 1.0f64);
            for v in 0..rest.len() {
                esp = esp.max(d[v]);
                for &n in &an[v] {
                    let n = n as usize;
                    rasg = rasg.max((d[v] - d[n]).abs());
                    let l0 = dist(rest[v], rest[n]);
                    if l0 > 1e-9 {
                        est = est.max(dist(p[v], p[n]) / l0);
                    }
                }
            }
            (esp, rasg, est)
        };
        let (a, b, c) = tres(&nosso);
        let (x, y, z) = tres(&orac);
        println!("{nome:<44} | {a:>8.4} {x:>8.4} | {b:>8.5} {y:>8.5} | {c:>7.4} {z:>7.4}");
    }
}
