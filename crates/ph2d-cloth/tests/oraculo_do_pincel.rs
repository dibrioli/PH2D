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
fn correr(nome: &str) -> Leitura {
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
