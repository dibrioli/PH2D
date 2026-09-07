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
use ph2d_cloth::verlet::{Solver, dist, norm, unit};
use ph2d_cloth::verlet_gesto::{
    Area, Curva, FalloffForca, Modo, Passo, Pincel, PincelTecido, RAIO_DA_NORMAL,
};
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
fn faces(superficie: &str, _rest: &[V3]) -> Vec<Vec<u32>> {
    // ⭐⭐⭐ **A lista de faces é do ALVO, e vem em fixture** (espec §3.1-bis,
    // emenda Q15). Antes disto o arnês reconstruía a malha casando a NOSSA
    // esfera UV com as posições de repouso **por posição**: os índices de vértice
    // eram os do alvo e a ordem das FACES era do nosso gerador. ⛔ E a §3.1 diz
    // que a ordem do anel é a ordem das faces incidentes, logo a lei era
    // aplicável no plano (onde degenera) e **inaplicável na esfera**.
    let mut fs = Vec::new();
    for l in inflar(&format!("{superficie}.faces.txt.gz")).lines() {
        if let Some(resto) = l.strip_prefix("f ") {
            fs.push(
                resto
                    .split_whitespace()
                    .map(|t| t.parse().expect("indice"))
                    .collect(),
            );
        }
    }
    assert!(!fs.is_empty(), "{superficie}.faces sem faces");
    fs
}

/// ⭐⭐ **A ORDEM DE VISITA da malha** (espec §3.1-bis): a concatenação, por
/// célula em índice crescente, dos vértices próprios de cada uma.
///
/// ⛔ **Não é a ordem crescente**, e no plano é a identidade RODADA — a célula `1`
/// fica com `[2080..4224]` e a `2` com `[0..2079]`, com o único descenso
/// exactamente no pen-down das fixtures `_origem`.
fn ordem_de_visita(superficie: &str) -> Vec<u32> {
    let mut ordem = Vec::new();
    for l in inflar(&format!("{superficie}.celulas.txt.gz")).lines() {
        if let Some(resto) = l.strip_prefix("cv ") {
            // `cv <indice> <n> <vertices proprios...>`
            ordem.extend(
                resto
                    .split_whitespace()
                    .skip(2)
                    .map(|t| t.parse::<u32>().expect("indice de vertice")),
            );
        }
    }
    assert!(
        !ordem.is_empty(),
        "{superficie}.celulas sem vertices proprios"
    );
    ordem
}

/// O anel-1 por ARESTAS (espec §3.1), de uma lista de faces.
fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        let m = f.len();
        for k in 0..m {
            // ⛔ **Por CANTO, nunca por aresta.** Percorrer as arestas e empurrar
            // os dois lados dá `(seguinte, anterior)` ao vértice que está na
            // posição `0` da face e `(anterior, seguinte)` a todos os outros —
            // um pólo de valência 96 saía com os dois primeiros trocados.
            a[f[k] as usize].push(f[(k + m - 1) % m]);
            a[f[k] as usize].push(f[(k + 1) % m]);
        }
    }
    // ⚠️⚠️ **A ORDEM DO ANEL é a das FACES à volta do vértice, e não a dos
    // índices** (espec §3.1): ela fixa a ordem em que as `(v, n)` e os pares
    // `(a, b)` entram na lista, e a lista é resolvida em Gauss-Seidel, que não
    // comuta. ⛔ Um `sort` aqui é uma ordem NOSSA a substituir a do alvo — e era
    // o que estava escrito.
    // A deduplicação é uma varredura linear, e não um conjunto: um anel tem meia
    // dúzia de vizinhos (96 num pólo), e a casa proíbe os contentores por hash
    // porque a ordem deles é o oposto do que esta função existe para preservar.
    for l in &mut a {
        let mut visto: Vec<u32> = Vec::with_capacity(l.len());
        l.retain(|v| {
            if visto.contains(v) {
                false
            } else {
                visto.push(*v);
                true
            }
        });
    }
    a
}

/// **A DIRECÇÃO DA SUPERFÍCIE PARA O OLHO** de um corpus de fixtures (espec §4.3
/// e §4.2-bis).
///
/// ⚠️ **Ele NÃO está no cabeçalho das fixtures** e é diferente nos dois corpora:
/// no plano a folha vive em `z = 0` e a vista é ao longo de `z` — a projecção é
/// um **no-op** e `δ` é a diferença dos pontos ao bit; na esfera a vista é ao
/// longo de `y` (o caminho pousa em `y = −√(1−x²)`) e é a componente `y` que se
/// perde. ⭐ A prova de que a vista é ORTOGRÁFICA está nos números: o passo do
/// caminho é `0,6/11 = 0,054545…` e `δ` mede `0,05455` nos doze passos da
/// esfera, apesar de os pontos estarem a profundidades diferentes.
fn eixo_da_vista(sup: &str) -> V3 {
    if sup.starts_with("esfera") {
        // O caminho pousa em `y = −√(1−x²)`, logo o olho está do lado `−y`.
        // ⚠️ A PROJECÇÃO do `δ` não distingue o sinal; quem o distingue é o
        // desempate dos dois baldes da normal da área (§4.2-bis).
        [0.0, -1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// `δ` = a diferença dos pontos PROJECTADA no plano do ecrã.
fn projecta(d: V3, v: V3) -> V3 {
    let k = d[0] * v[0] + d[1] * v[1] + d[2] * v[2];
    [d[0] - v[0] * k, d[1] - v[1] * k, d[2] - v[2] * k]
}

/// A normal (não normalizada) de UMA face, por Newell.
fn normal_da_face(pos: &[V3], f: &[u32]) -> V3 {
    let mut n = [0.0f64; 3];
    for k in 0..f.len() {
        let (a, b) = (pos[f[k] as usize], pos[f[(k + 1) % f.len()] as usize]);
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

/// Normais por vértice das posições ACTUAIS, somadas face a face e
/// normalizadas no fim (espec §4.6 linha 4).
///
/// ⭐⭐⭐ **O PESO de cada face nessa soma é UNIFORME, e isso deixou de ser uma
/// escolha em 2026-09-07: é MEDIDO** (a §4.6 linha 4 declarava-o aberto).
///
/// A régua é a fixture de dois traços (`plano_inflar_radial_local_1passo_2tracos`):
/// o 2.º traço lê as normais da cova que o 1.º abriu, logo a direcção por vértice
/// do impulso dele **é** a normal que se procura, e ela lê-se dos ficheiros do
/// oráculo sem passar pelo nosso código. Mediana do desvio de direcção:
///
/// | peso | mediana | contra as normais PLANAS |
/// |---|---|---|
/// | **uniforme** (a normal da face, normalizada) | **`1,7·10⁻⁵`** | `0,299` |
/// | área (Newell traz a área embutida) | `6,6·10⁻⁴` | `0,299` |
///
/// ⇒ **`38×`**, e o `1,7·10⁻⁵` é o dígito que a espec §10.11 cita para esta
/// fixture. ⛔ Não é o chão da resolução do ficheiro: restringindo aos vértices
/// que se movem mais de `10⁻²`, o uniforme desce a `9,4·10⁻⁶` e a área ESTACIONA
/// em `3,7·10⁻⁴` — *um resíduo que não encolhe com o sinal não é ruído, é lei
/// errada.* O experimento `PH2D_PESO_NORMAL=area` corre a alternativa.
fn normais(pos: &[V3], faces: &[Vec<u32>]) -> Vec<V3> {
    let uniforme = std::env::var("PH2D_PESO_NORMAL").as_deref() != Ok("area");
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
        if uniforme {
            let l = (fnrm[0] * fnrm[0] + fnrm[1] * fnrm[1] + fnrm[2] * fnrm[2]).sqrt();
            if l > 0.0 {
                fnrm = [fnrm[0] / l, fnrm[1] / l, fnrm[2] / l];
            }
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
///
/// ⚠️ O envelope sem argumento (`reordenar`) MORREU em 07/09 com o último laço
/// irmão que o chamava: quem quer a ordem do ambiente lê-a onde entra no laço
/// único. *Uma função cujo único chamador era um laço que media outro programa
/// não é uma facilidade — é o rasto dele.*
fn reordenar_com(sim: &mut ph2d_cloth::verlet::Verlet, ordem: Option<&str>) {
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
    let Some(ordem) = ordem else {
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
    correr_posicoes_com(nome, std::env::var("PH2D_ORDEM").ok().as_deref())
}

/// Idem, com a ORDEM de resolução dada em vez de lida do ambiente.
fn correr_posicoes_com(nome: &str, ordem: Option<&str>) -> Vec<V3> {
    correr_com_pincel(nome, ordem).0
}

/// A MESMA corrida, devolvendo também o pincel no fim — é o que uma sonda
/// precisa para imprimir `φ` e a retenção sem reescrever o laço.
///
/// ⛔⛔ **Nenhuma sonda desta bancada volta a ter laço próprio.** A
/// `sonda_do_perfil` teve um durante três jornadas e ele divergiu do produto em
/// TRÊS sítios de uma vez — o eixo da vista escrito à mão (`+z`, logo errado em
/// toda fixture de esfera), a condição de «passo parado» sem a metade do `δ`
/// nulo, e o delta do Agarrar sem o ramo que o acumula. *Uma sonda que mede
/// outro programa que o produto responde com confiança a perguntas sobre uma
/// coisa que ninguém corre.*
fn correr_com_pincel(nome: &str, ordem: Option<&str>) -> (Vec<V3>, PincelTecido) {
    let (pos, tecido, _, _) = correr_com_blocos(nome, ordem, None);
    (pos, tecido)
}

/// A MESMA corrida, com o CAMINHO dado (o dos dumps por passo, quando difere) e
/// devolvendo o estado da malha DEPOIS DE CADA PASSO.
///
/// ⛔⛔⛔ **É o único laço da bancada, e isso é uma cura, não arrumação.** A
/// `sonda_do_perfil` e a `correr_por_passo` tiveram laços próprios, e os dois
/// divergiram do produto — o segundo escrevia o guarda de «passo parado» como
/// `k == 0`, sem a metade do `δ` nulo, logo aplicava força nos dez passos em que
/// o alvo não aplica nenhuma. *Medido: no `plano_empurrar_radial_local_origem_parado`
/// isso dá `0,1810` contra `0,1214` já no passo 3, enquanto o gate de paridade —
/// que corre o laço CERTO — lê `0,001` no mesmo traço.* ⇒ **três laços irmãos,
/// três divergências, num dia.**
fn correr_com_blocos(
    nome: &str,
    ordem: Option<&str>,
    caminho: Option<&[V3]>,
) -> (Vec<V3>, PincelTecido, Vec<Vec<V3>>, Vec<V3>) {
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
    // ⚠️ O caminho DADO ganha ao do cabeçalho — os dumps por passo trazem o
    // deles, e correr dois caminhos diferentes pelo mesmo nome foi o defeito.
    let caminho: Vec<V3> = caminho.map_or_else(|| t.caminho.clone(), <[V3]>::to_vec);
    let passos = caminho.len();
    assert_eq!(
        t.f("passos") as usize,
        t.caminho.len(),
        "{nome}: passos != pontos do caminho"
    );

    // ⚠️⚠️ **`tracos` é quantas vezes o MESMO gesto é repetido sobre a malha que o
    // anterior deixou** (omissão `1`). Ele existe porque é a única leitura que
    // separa *«a superfície do INÍCIO do traço»* de *«a superfície original do
    // objecto»* (espec §4.2-ter): o 2.º traço lê as normais da cova que o 1.º
    // abriu, não as planas.
    let tracos = if t.chaves.contains_key("tracos") {
        t.f("tracos") as usize
    } else {
        1
    };
    let mut pos = rest.clone();
    let mut blocos: Vec<Vec<V3>> = Vec::with_capacity(passos);
    // ⭐ A NORMAL DA ÁREA de cada passo (espec §4.2-bis): o vector NULO diz que o
    // gesto se calou naquele passo, que é o regime normal do Push (§4.2-bis (8)).
    let mut gestos: Vec<V3> = Vec::with_capacity(passos);
    let mut tecido = PincelTecido::pen_down(pincel, &rest, caminho[0], ordem_de_visita(&sup));
    for _traco in 0..tracos {
        // ⭐ O repouso do traço e as normais dele são a malha que ESTE traço encontra.
        let repouso_do_traco = pos.clone();
        let normais_do_traco = normais(&pos, &fs);
        tecido = PincelTecido::pen_down(pincel, &pos, caminho[0], ordem_de_visita(&sup));
        let _ = &repouso_do_traco;
        for k in 0..passos {
            let cursor = caminho[k];
            let prev = caminho[k.saturating_sub(1)];
            let d3 = if pincel.modo == Modo::Agarrar {
                let c0 = caminho[0];
                [cursor[0] - c0[0], cursor[1] - c0[1], cursor[2] - c0[2]]
            } else {
                [
                    cursor[0] - prev[0],
                    cursor[1] - prev[1],
                    cursor[2] - prev[2],
                ]
            };
            let delta = projecta(d3, eixo_da_vista(&sup));
            let parado = k == 0 || dist(delta, [0.0; 3]) == 0.0;
            // ⭐⭐⭐ **As normais são as da superfície que o TRAÇO ENCONTROU**, não as
            // de agora (espec §4.2-ter): dentro de um traço o pincel deforma a malha
            // e continua a ler as normais com que o traço começou. ⛔ Recalculá-las
            // por passo é o que fazia o Push e o Inflate errarem `0,24`.
            let nrm = &normais_do_traco;
            let passo = Passo {
                cursor,
                delta,
                delta_3d: d3,
                parado,
                vista: eixo_da_vista(&sup),
                normais: nrm,
                pressao: 1.0,
            };
            let simulou = tecido.passo(&pos, &anel, &passo);
            if k == 0 {
                reordenar_com(&mut tecido.sim, ordem);
            }
            if simulou {
                for (v, act) in tecido.sim.activo.iter().enumerate() {
                    if *act {
                        pos[v] = tecido.sim.x[v];
                    }
                }
            }
            blocos.push(pos.clone());
            // ⭐⭐⭐ **O vector do gesto deste passo** (espec §4.2-bis, §10.11): a
            // normal da área, e o **vector NULO** quando o Push não escreve força
            // nenhuma. São DUAS as razões de ele ser nulo e as duas contam como
            // «calado» no instrumento do oráculo — o passo sem movimento (§4.3) e
            // o **disco de amostragem vazio** (§4.2-bis (8)).
            gestos.push(if simulou && !parado {
                tecido.normal_da_area
            } else {
                [0.0; 3]
            });
        }
    }

    (pos, tecido, blocos, gestos)
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
    let rest = repouso(t.s("superficie"));
    // ⛔ A MESMA corrida do produto (ver `correr_com_pincel`) — não um laço irmão.
    let (pos, tecido) = correr_com_pincel(&nome, std::env::var("PH2D_ORDEM").ok().as_deref());
    println!(
        "{nome}: restricoes={} activos={}",
        tecido.sim.restricoes.len(),
        tecido.dentro.len()
    );
    println!(
        "{:>8} {:>8} {:>8} {:>6} {:>6}",
        "x", "nos", "oraculo", "phi", "w0"
    );
    // A linha do traço: os vértices sobre o caminho. ⚠️ No plano ela é `y = 0`;
    // na esfera o caminho pousa em `y = −√(1−x²)`, e uma sonda que fixasse
    // `y ≈ 0` imprimiria a linha ERRADA (ou nenhuma).
    let na_linha = |v: usize| {
        if t.s("superficie") == "plano" {
            rest[v][1].abs() < 1e-6
        } else {
            rest[v][2].abs() < 1e-3 && rest[v][1] < 0.0
        }
    };
    let mut linha: Vec<usize> = (0..rest.len()).filter(|v| na_linha(*v)).collect();
    linha.sort_by(|a, b| rest[*a][0].total_cmp(&rest[*b][0]));
    for v in linha {
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

    // ⛔⛔ **O laço é o de [`correr_com_blocos`]** — esta sonda teve um próprio até
    // 07/09, com `parado: k == 0` (sem a metade do `δ` nulo) e as normais lidas
    // por passo. *O quarto laço-irmão desta bancada, com a mesma causa dos três
    // primeiros: uma sonda com laço próprio responde com confiança a perguntas
    // sobre um programa que ninguém corre.*
    let (_, _, nossos, gestos) = correr_com_blocos(
        &nome,
        std::env::var("PH2D_ORDEM").ok().as_deref(),
        Some(&pp.caminho),
    );
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
        let pos = &nossos[k];
        // ⭐ `*` diz que o gesto DISPAROU neste passo; um espaço, que ele se calou
        // por o disco de amostragem ter ficado vazio (espec §4.2-bis (8)).
        let disparou = if dist(gestos[k], [0.0; 3]) > 0.0 {
            "*"
        } else {
            " "
        };
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
            "{:>4}{} | {:>7.4} {:>7.4} | {:>8.5} {:>8.5} | {:>8.5} {:>8.5} | {:>7.5} {:>7.5} | {:>8.4} {:>8.4} | {:>5.2}R {:>5.2}R | ERRO {:>8.5} | anel {:>7.4} {:>7.4} | c0a {:>5.2}R {:>5.2}R | u_nos [{:>7.4} {:>7.4} {:>7.4}] u_or [{:>7.4} {:>7.4} {:>7.4}] cursor-rest [{:>7.4} {:>7.4} {:>7.4}]{}",
            k + 1,
            disparou,
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
        "{:<46} {:>6} {:>6} | {:>10} {:>10} | {:>8} {:>8} {:>7}",
        "traco", "mov_n", "mov_o", "max_n", "max_o", "err_max", "err_rms", "err/max"
    );
    // ⚠️ Os máximos saem a SEIS casas, que é a precisão com que o cabeçalho de
    // cada fixture grava o dele — a quatro casas a razão Local/Global de um
    // traço de Expand tem `±4 %` de incerteza só do arredondamento.
    for nome in todas() {
        let l = correr(&nome);
        println!(
            "{:<46} {:>6} {:>6} | {:>10.6} {:>10.6} | {:>8.4} {:>8.4} {:>7.3}",
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
/// ⭐⭐⭐ **Ela sai de um VALE MEDIDO, e o vale MUDOU de sítio em 06/09**, quando a
/// ordem de visita passou a ser a da partição em células (§3.1-bis). Os `65`
/// traços partem-se hoje assim:
///
/// ```text
/// 47 traços ≤ 0,024   ·   0,093 · 0,096 · 0,105   ·   [vazio de 0,045]   ·   0,150 …
/// ```
///
/// ⇒ **`0,13` continua dentro de um vazio** (entre `0,105` e `0,150`), e o corpus
/// deixou de ter meio-termo: `47` dos `65` estão a menos de `1/5` da barra e
/// `17` saem praticamente ao bit.
///
/// ⚠️⚠️ **E a justificação ANTIGA desta barra caducou, ainda que o número não:**
/// ela dizia *«os 53 traços partem-se em 28 com ≤ 0,095 e 25 com ≥ 0,175»* e
/// apoiava-se em a dispersão entre duas ordens NOSSAS ser `0,0985` — o argumento
/// era *«não aperte, senão mede a sua própria ordenação»*. Hoje a nossa ordem é a
/// do alvo (medida, §3.1-bis), e o que sobra do outro lado do vazio já não é
/// ordenação: é o regime em que o próprio alvo deixa de ser determinista
/// (§5.2-ter) e os modos ainda por explicar.
const BARRA_PARIDADE: f64 = 0.13;

/// Os traços que a lei REPRODUZ (espec §14 gate 15).
const PARIDADE: [&str; VERDE_N] = [
    "esfera_arrastar_radial_dinamica",
    "esfera_empurrar_radial_dinamica",
    "esfera_empurrar_radial_local_1passo",
    "esfera_inflar_radial_dinamica",
    "esfera_inflar_radial_local_1passo",
    "plano_agarrar_plano_local",
    "plano_agarrar_radial_global_origem_1passo",
    "plano_agarrar_radial_local",
    "plano_agarrar_radial_local_1passo",
    "plano_agarrar_radial_local_24passos",
    "plano_agarrar_radial_local_2passos",
    "plano_agarrar_radial_local_2passos_origem",
    "plano_agarrar_radial_local_amort06",
    "plano_agarrar_radial_local_origem_1passo",
    "plano_agarrar_radial_local_preset",
    "plano_apertar_linha_radial_local",
    "plano_apertar_linha_radial_local_1passo",
    "plano_apertar_linha_radial_local_origem",
    "plano_apertar_ponto_radial_local_1passo",
    "plano_apertar_ponto_radial_local_origem_fraco",
    "plano_arrastar_plano_local",
    "plano_arrastar_radial_dinamica",
    "plano_arrastar_radial_dinamica_preset",
    "plano_arrastar_radial_global",
    "plano_arrastar_radial_global_origem",
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
    "plano_empurrar_plano_local",
    "plano_empurrar_radial_global_origem",
    "plano_empurrar_radial_local",
    "plano_empurrar_radial_local_1passo",
    "plano_empurrar_radial_local_origem",
    "plano_empurrar_radial_local_origem_amort1",
    "plano_empurrar_radial_local_origem_forca025",
    "plano_empurrar_radial_local_origem_forca05",
    "plano_empurrar_radial_local_origem_massa2",
    "plano_empurrar_radial_local_origem_parado",
    "plano_expandir_radial_global_origem_1passo",
    "plano_expandir_radial_local",
    "plano_expandir_radial_local_1passo",
    "plano_expandir_radial_local_origem_1passo",
    "plano_expandir_radial_local_origem_1passo_forca05",
    "plano_gancho_radial_global_origem_1passo",
    "plano_gancho_radial_local",
    "plano_gancho_radial_local_1passo",
    "plano_gancho_radial_local_24passos",
    "plano_gancho_radial_local_2passos",
    "plano_gancho_radial_local_2passos_origem",
    "plano_gancho_radial_local_amort06",
    "plano_gancho_radial_local_origem_1passo",
    "plano_gancho_radial_local_origem_1passo_constante",
    "plano_gancho_radial_local_origem_1passo_curto",
    "plano_inflar_radial_local",
    "plano_inflar_radial_local_1passo",
    "plano_inflar_radial_local_1passo_2tracos",
    "plano_inflar_radial_local_origem",
    "plano_inflar_radial_local_origem_massa2",
    "plano_inflar_radial_local_origem_parado",
];
const VERDE_N: usize = 68;

/// Os traços AINDA por explicar, com o valor MEDIDO em 2026-09-06 ao lado.
///
/// ⚠️ **A lista tem censo de obsolescência nas DUAS pontas** (CLAUDE.md §5.0:
/// *uma catraca sem censo não desce, vira licença*): um traço daqui que passe a
/// bater é acusado — tem de migrar para [`PARIDADE`] —, e um que se degrade
/// acima da folga também. ⭐ **É o segundo ramo que fecha o gate 17:** um port
/// que dobre a relaxação em TODA a parte passaria o gate 16 e mandaria o
/// `plano_arrastar_radial_global` de `0,301` para `0,583` (medido), o que cai
/// fora da folga e reprova aqui.
/// ⚠️ **As SETE de esfera mudaram em 06/09 com a ordem do ANEL** (§3.1: ela é a
/// das FACES, não a dos índices) — `4` pioraram e `3` melhoraram, e **todas as
/// mudanças cabem dentro da dispersão de ordem do próprio traço** (`1 %` a `30 %`
/// dela, medida pela [`sonda_do_chao_de_ruido`]). ⇒ *o corpus não discrimina as
/// duas leis aqui*, e o que shipa é a da espec. ⛔ No plano as duas coincidem ao
/// bit — o percurso face a face de um vértice interior de grelha devolve
/// `[S, O, E, N]`, que já é a ordem crescente de índice.
const ABERTOS: [(&str, f64); ABERTO_N] = [
    ("esfera_agarrar_radial_dinamica", 0.182),
    ("esfera_apertar_linha_radial_dinamica", 0.630),
    ("esfera_apertar_ponto_radial_dinamica", 0.646),
    ("esfera_expandir_radial_dinamica", 0.581),
    ("esfera_gancho_radial_dinamica", 0.255),
    ("plano_apertar_ponto_plano_local", 0.636),
    ("plano_apertar_ponto_radial_local", 0.600),
    ("plano_apertar_ponto_radial_local_origem", 0.908),
];
const ABERTO_N: usize = 8;

/// A folga de regressão sobre o valor medido de um traço ABERTO.
const FOLGA_ABERTO: f64 = 1.25;

/// ⭐ **A RESOLUÇÃO DO FICHEIRO, acumulada nos doze passos** — a barra dos traços
/// cuja lei é EXACTA, em unidades ABSOLUTAS de posição.
///
/// Cada coordenada dos dumps traz seis casas ⇒ `≤ 5·10⁻⁷` de arredondamento por
/// número, e um traço integra doze blocos desses.
///
/// ⚠️ **A espec §14 gate 41 escreve `5·10⁻⁶` e a tabela dela lê `0,000003`–`0,000005`;
/// nós lemos `5,2·10⁻⁶` no pior dos dez** (o `_parado`), que arredonda ao mesmo
/// `0,000005`. ⛔ Pôr a barra no valor medido seria pôr a barra EM CIMA da
/// medição: ela fica na casa seguinte, que é a que o ficheiro nomeia.
///
/// ⚠️⚠️ **Ela é ABSOLUTA de propósito.** A barra relativa da [`BARRA_PARIDADE`]
/// divide pelo maior deslocamento do traço, e os traços do corpus têm sinais que
/// diferem por `80×` — a fixture de força fraca move `0,004` e o arrasto move
/// `0,33`. *Comparar erros relativos entre eles é comparar duas resoluções de
/// ficheiro diferentes e chamar-lhe qualidade de lei.*
const BARRA_DO_FICHEIRO: f64 = 1e-5;

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
        let mut tecido = PincelTecido::pen_down(pincel, &rest, c0, ordem_de_visita(&sup));
        let passo = Passo {
            cursor: c0,
            delta: [0.0; 3],
            delta_3d: [0.0; 3],
            parado: true,
            vista: [0.0, 0.0, 1.0],
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

/// **OS DEZ TRAÇOS DE EMPURRAR/INFLAR POR PASSO** (espec §10.11) — os sete do
/// Push e os três do Inflate, cada um com os doze blocos do oráculo.
///
/// ⚠️ Eles são a população dos gates 40 e 41, e a lista é dos FICHEIROS: um
/// `.porpasso` novo de empurrar/inflar que não entre aqui não é medido.
const POR_PASSO_DE_FORCA: [&str; 10] = [
    "plano_empurrar_radial_local_origem",
    "plano_empurrar_radial_global_origem",
    "plano_empurrar_radial_local_origem_forca05",
    "plano_empurrar_radial_local_origem_forca025",
    "plano_empurrar_radial_local_origem_massa2",
    "plano_empurrar_radial_local_origem_amort1",
    "plano_empurrar_radial_local_origem_parado",
    "plano_inflar_radial_local_origem",
    "plano_inflar_radial_local_origem_massa2",
    "plano_inflar_radial_local_origem_parado",
];

/// **SONDA — O DISPARO DO PUSH, passo a passo** (espec §4.2-bis (8), §10.11).
///
/// ⭐⭐⭐ O disco que decide se a normal da área existe tem raio `R · 0,5` e
/// mede-se contra as posições **de agora**. Numa folha que o próprio Push já
/// afundou chega um passo em que nenhum vértice está a menos disso do cursor —
/// e nesse passo o gesto **não escreve aceleração nenhuma**. Depois volta a
/// disparar, à medida que o cursor avança para terreno pouco afundado.
///
/// Colunas: `min` é o `min_v |p_v − c_k|` do ORÁCULO (a malha com que o passo
/// começa, que é o bloco `k−1` do dump), `nos` diz se NÓS disparámos, e `erro` é
/// o pior erro por vértice contra o bloco do oráculo no fim do passo.
#[test]
#[ignore = "sonda"]
fn sonda_do_disparo_do_empurrar() {
    for nome in POR_PASSO_DE_FORCA {
        let pp = por_passo(nome);
        let t = traco(nome);
        let sup = t.s("superficie").to_string();
        let rest = repouso(&sup);
        let alcance = t.pincel().raio * ph2d_cloth::verlet_gesto::RAIO_DA_NORMAL;
        let (_, _, nossos, gestos) = correr_com_blocos(nome, None, Some(&pp.caminho));
        let mut linha = String::new();
        let mut pior = 0.0f64;
        for k in 0..pp.caminho.len() {
            let antes: &[V3] = if k == 0 { &rest } else { &pp.blocos[k - 1] };
            let min = antes
                .iter()
                .map(|p| dist(*p, pp.caminho[k]))
                .fold(f64::INFINITY, f64::min);
            let nos = dist(gestos[k], [0.0; 3]) > 0.0;
            let erro = pp.blocos.get(k).map_or(0.0, |b| {
                b.iter()
                    .zip(&nossos[k])
                    .map(|(a, c)| dist(*a, *c))
                    .fold(0.0, f64::max)
            });
            pior = pior.max(erro);
            linha += &format!(
                "  {:>2}{} min {:.5}{} erro {:.6}\n",
                k + 1,
                if nos { "*" } else { " " },
                min,
                if min <= alcance { "<" } else { ">" },
                erro
            );
        }
        println!("{nome}  (disco {alcance:.5})  pior erro {pior:.6}\n{linha}");
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
    // ⚠️ UMA vez, no pen-down (espec §4.2-ter).
    let normais_do_traco = normais(&pos, &fs);
    let mut tecido = PincelTecido::pen_down(pincel, &pos, c0, ordem_de_visita(&sup));
    let mut argmax = (0usize, 0.0f64);
    let mut cursor = c0;
    for k in 0..2 {
        cursor = pp.caminho[k];
        let prev = pp.caminho[k.saturating_sub(1)];
        let d3 = [
            cursor[0] - prev[0],
            cursor[1] - prev[1],
            cursor[2] - prev[2],
        ];
        let delta = projecta(d3, eixo_da_vista(&sup));
        // ⭐⭐⭐ **As normais são as da superfície que o TRAÇO ENCONTROU**, não as
        // de agora (espec §4.2-ter): dentro de um traço o pincel deforma a malha
        // e continua a ler as normais com que o traço começou. ⛔ Recalculá-las
        // por passo é o que fazia o Push e o Inflate errarem `0,24`.
        let nrm = &normais_do_traco;
        let passo = Passo {
            cursor,
            delta,
            delta_3d: d3,
            parado: k == 0,
            vista: eixo_da_vista(&sup),
            normais: nrm,
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
        "{:<44} | {:>8} {:>8} | {:>8} {:>8} | {:>7} {:>7} | {:>5} {:>5} | {:>7} {:>7}",
        "traco",
        "esp_nos",
        "esp_orac",
        "rasg_nos",
        "rasg_or",
        "est_nos",
        "est_or",
        "iv_n",
        "iv_o",
        "cmp_n",
        "cmp_o"
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
        // ⭐⭐ **AS FACES INVERTIDAS**, que é a régua que separa «lei por
        // descobrir» de «o alvo deixou de ser determinista». Um campo de força
        // convergente empurra o vértice para ALÉM do alvo; a face vira do
        // avesso; os pares ficam comprimidos e o factor de correcção inverte o
        // sinal e cresce sem tecto — a partir daí o resultado por vértice é
        // decidido pela ORDEM da lista, e essa ordem sai de uma árvore espacial
        // que não é a nossa. *Onde o ORÁCULO inverte faces, a paridade não é
        // alcançável, e perseguí-la é perseguir um defeito aberto do alvo.*
        let invertidas = |p: &[V3]| -> usize {
            fs.iter()
                .filter(|f| {
                    let n0 = normal_da_face(&rest, f);
                    let n1 = normal_da_face(p, f);
                    n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2] < 0.0
                })
                .count()
        };
        let (inv_n, inv_o) = (invertidas(&nosso), invertidas(&orac));
        // ⭐⭐⭐ **A COMPRESSÃO do par mais apertado**, que é a régua que o
        // contador de faces invertidas NÃO é: a correcção de uma restrição vale
        // `RIGIDEZ · (1 − ℓ/D)`, então quando `D/ℓ` fica pequeno ela inverte o
        // sinal e cresce **sem tecto** — e é aí que a ORDEM da lista passa a
        // decidir o resultado por vértice.
        let compressao = |p: &[V3]| -> f64 {
            let mut m = f64::MAX;
            for (v, viz) in an.iter().enumerate() {
                for &w in viz {
                    let l = dist(rest[v], rest[w as usize]);
                    if l > 1e-12 {
                        m = m.min(dist(p[v], p[w as usize]) / l);
                    }
                }
            }
            m
        };
        let (cmp_n, cmp_o) = (compressao(&nosso), compressao(&orac));
        let (a, b, c) = tres(&nosso);
        let (x, y, z) = tres(&orac);
        println!(
            "{nome:<44} | {a:>8.4} {x:>8.4} | {b:>8.5} {y:>8.5} | {c:>7.4} {z:>7.4} | \
             {inv_n:>5} {inv_o:>5} | {cmp_n:>7.4} {cmp_o:>7.4}"
        );
    }
}

/// **As posições NOSSAS depois de CADA passo** de um traço com dump por passo.
///
/// ⚠️ Extraída da [`sonda_passo_a_passo`] para os gates 19–21 medirem sobre a
/// MESMA corrida que a sonda imprime — duas cópias do laço seriam duas leis.
fn correr_por_passo(nome: &str) -> (Vec<V3>, Vec<Vec<u32>>, Vec<Vec<V3>>) {
    // ⛔ O laço do PRODUTO, com o caminho do dump por passo — ver
    // [`correr_com_blocos`]. Zero laço irmão.
    let pp = por_passo(nome);
    let sup = traco(nome).s("superficie").to_string();
    let rest = repouso(&sup);
    let fs = faces(&sup, &rest);
    let (_, _, blocos, _) = correr_com_blocos(
        nome,
        std::env::var("PH2D_ORDEM").ok().as_deref(),
        Some(&pp.caminho),
    );
    (rest, fs, blocos)
}

/// **QUADRILÁTEROS DE ORIENTAÇÃO INVERTIDA** (espec §5.2-ter): a face cuja
/// normal depois do passo aponta ao contrário da dela em repouso.
///
/// ⛔ **Não é «somar as duas metades triangulares»** — essa leitura conta também
/// o quadrilátero apenas DOBRADO, que não inverteu, e devolve `11/26/88` onde
/// esta devolve `10/18/52` (a espec nomeia a leitura errada com os números).
fn invertidos(rest: &[V3], pos: &[V3], fs: &[Vec<u32>]) -> usize {
    fs.iter()
        .filter(|f| {
            let a = normal_da_face(rest, f);
            let b = normal_da_face(pos, f);
            a[0] * b[0] + a[1] * b[1] + a[2] * b[2] < 0.0
        })
        .count()
}

/// O vértice reflectido no plano `y = 0` (o plano do traço destas fixtures).
fn espelho_de(rest: &[V3]) -> Vec<usize> {
    let chave = |p: V3| {
        (
            (p[0] * 1e4).round() as i64,
            (p[1] * 1e4).round() as i64,
            (p[2] * 1e4).round() as i64,
        )
    };
    let mapa: std::collections::BTreeMap<_, usize> = rest
        .iter()
        .enumerate()
        .map(|(v, p)| (chave(*p), v))
        .collect();
    rest.iter()
        .enumerate()
        .map(|(v, p)| *mapa.get(&chave([p[0], -p[1], p[2]])).unwrap_or(&v))
        .collect()
}

/// **ASSIMETRIA DE ESPELHO ÷ `|u|max`** (espec §5.2-ter).
///
/// ⚠️ **As duas normas são diferentes de propósito**: o numerador é a norma do
/// MÁXIMO por componente e o denominador a EUCLIDIANA — trocá-las muda o número.
fn assimetria(rest: &[V3], pos: &[V3], esp: &[usize]) -> f64 {
    let u = |v: usize| {
        [
            pos[v][0] - rest[v][0],
            pos[v][1] - rest[v][1],
            pos[v][2] - rest[v][2],
        ]
    };
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for (v, &e) in esp.iter().enumerate() {
        let (a, b) = (u(v), u(e));
        // `M` reflecte o próprio vector: a componente `y` troca de sinal.
        let m = [b[0], -b[1], b[2]];
        for c in 0..3 {
            num = num.max((a[c] - m[c]).abs());
        }
        den = den.max((a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt());
    }
    num / den.max(1e-12)
}

/// **GATE 19 — o APERTO inverte a malha no 1.º passo simulado, e o arrasto não.**
///
/// ⭐ É o facto que separa «lei em falta» de «o alvo deixou de ser determinista»
/// (espec §5.2-ter): a força do aperto não decresce com a proximidade, logo o
/// vértice ao lado do cursor anda MAIS do que a distância a que estava dele.
/// A partir daí a relaxação recebe pares comprimidos e a ordem da lista decide.
///
/// ⚠️ **A terceira linha é a INTERVENÇÃO**: o mesmo traço com a força `1 → 0,2`
/// não inverte nada nos doze passos. *Tira-se a inversão e o modo, a lei e a
/// maquinaria não mudaram.*
#[test]
fn o_aperto_inverte_a_malha_no_primeiro_passo_e_o_arrastar_nao() {
    let casos: [(&str, bool); 3] = [
        ("plano_apertar_ponto_radial_local_origem", true),
        ("plano_arrastar_radial_local_origem", false),
        ("plano_apertar_ponto_radial_local_origem_fraco", false),
    ];
    for (nome, inverte) in casos {
        let (rest, fs, passos) = correr_por_passo(nome);
        // O 1.º passo simulado é o 2.º ponto do caminho (o 1.º constrói e não simula).
        let n1 = invertidos(&rest, &passos[1], &fs);
        // Anti-vácuo: o traço tem de ter deformado alguma coisa.
        let movidos = passos[1]
            .iter()
            .zip(&rest)
            .filter(|(a, b)| dist(**a, **b) > 1e-9)
            .count();
        assert!(movidos > 50, "{nome}: so' {movidos} movidos -- vacuo");
        if inverte {
            assert!(
                n1 > 0,
                "{nome}: ZERO quadrilateros invertidos no 1.º passo simulado -- \
                 o oraculo da' 10, e sem a inversao o resto da §5.2-ter nao se aplica"
            );
        } else {
            assert_eq!(
                n1, 0,
                "{nome}: {n1} quadrilateros invertidos no 1.º passo simulado, e este \
                 traco NAO pode inverter (o oraculo da' 0)"
            );
            // E o controlo de força fraca não inverte em passo NENHUM.
            for (k, p) in passos.iter().enumerate() {
                let n = invertidos(&rest, p, &fs);
                assert_eq!(n, 0, "{nome}: {n} invertidos no passo {}", k + 1);
            }
        }
    }
}

/// **GATE 20 — a barra dos apertos é a da ORDEM, e mede-se em DOIS regimes.**
///
/// ⚠️⚠️ **A espec propõe este gate como «a nossa assimetria nunca passa a do
/// oráculo no mesmo passo», e a MEDIÇÃO mostra que isso não é propriedade de
/// nenhum dos dois lados.** Num passo com faces invertidas o resultado por
/// vértice é decidido pela ORDEM da lista (§5.2-ter) e nenhum dos lados domina o
/// outro: no aperto a força cheia nós ficamos acima em `k = 5, 7, 11` e abaixo
/// nos outros nove. *Uma barra «sempre abaixo» sobre um regime caótico é uma
/// barra que reprova por sorteio.* ⇒ o gate mede os dois regimes com réguas
/// diferentes, e as duas barras saem da tabela abaixo:
///
/// | traço | passos SEM inversão (nós ÷ oráculo) | passos COM inversão |
/// |---|---|---|
/// | arrastar *Local* | `1,15`–`1,26` (12 passos) | — |
/// | aperto de ponto, força `0,2` | `1,49`–`1,67` (12 passos) | — |
/// | aperto de ponto, força `1` | — | envelope `1,979` contra `1,463` = `1,35×` |
///
/// - **Sem inversão** a comparação por passo VALE, e a barra `2,0` fica no vazio
///   medido entre o pior caso são (`1,67`) e o pior do regime caótico (`2,12`).
/// - **Com inversão** a comparação por passo não vale; o que se afirma é o
///   ENVELOPE do traço, com a barra `2,0` sobre a razão medida de `1,35`.
///
/// ⛔ As duas barras são derivadas da medição, ⛔ nenhuma é um epsilon de conforto,
/// e a segunda **não** afirma que nós reproduzimos o alvo ali — afirma que não
/// somos pior por uma ordem de grandeza num regime que o alvo também não
/// controla.
#[test]
fn a_assimetria_de_espelho_fica_no_patamar_do_oraculo() {
    for nome in [
        "plano_apertar_ponto_radial_local_origem",
        "plano_apertar_ponto_radial_local_origem_fraco",
        "plano_arrastar_radial_local_origem",
    ] {
        let (rest, fsx, passos) = correr_por_passo(nome);
        let esp = espelho_de(&rest);
        let pp = por_passo(nome);
        // Controlo: o espelho tem de ser uma involução sobre a malha inteira.
        assert!(
            (0..rest.len()).all(|v| esp[esp[v]] == v),
            "{nome}: o espelho nao e' involucao -- a malha nao e' simetrica no traco"
        );
        let (mut env_n, mut env_o, mut houve_inversao, mut sem_inv) =
            (0.0f64, 0.0f64, false, 0usize);
        for (k, nosso) in passos.iter().enumerate() {
            let Some(bloco) = pp.blocos.get(k) else {
                continue;
            };
            if bloco.iter().zip(&rest).all(|(a, b)| a == b) && k > 0 {
                continue; // bloco = repouso: glitch do dump do oráculo
            }
            let (an, ao) = (
                assimetria(&rest, nosso, &esp),
                assimetria(&rest, bloco, &esp),
            );
            env_n = env_n.max(an);
            env_o = env_o.max(ao);
            if invertidos(&rest, bloco, &fsx) > 0 {
                houve_inversao = true;
            } else {
                sem_inv += 1;
                assert!(
                    an <= ao * 2.0 + 1e-9,
                    "{nome} passo {}: SEM inversao a nossa assimetria {an:.4} passa \
                     {:.4} = 2x a do oraculo {ao:.4} -- fora da inversao a barra e' \
                     por passo, e o pior caso medido e' 1,67x",
                    k + 1,
                    ao * 2.0
                );
            }
        }
        // Anti-vácuo: tem de haver passos medidos, e assimetria de facto.
        assert!(sem_inv > 2 || houve_inversao, "{nome}: nada medido");
        assert!(
            env_o > 0.05,
            "{nome}: o oraculo nao e' assimetrico ({env_o:.4})"
        );
        if houve_inversao {
            assert!(
                env_n <= env_o * 2.0,
                "{nome}: o envelope da nossa assimetria {env_n:.4} passa {:.4} = 2x o \
                 do oraculo {env_o:.4} -- no regime invertido a barra e' o envelope, \
                 e a razao medida e' 1,35x",
                env_o * 2.0
            );
        }
    }
}

/// **GATE 21 — FORA da inversão o aperto é tão comparável quanto o arrasto.**
///
/// ⭐⭐ **É o gate que ILIBA a lei do aperto**: sobre a fixture de força fraca,
/// que não inverte uma única face nos doze passos, a paridade por vértice do
/// aperto tem de ficar no patamar da do arrasto. *Se ficar pior, o defeito não é
/// a ordem e há lei em falta* — e a `1,079` do irmão a força cheia deixaria de
/// ter explicação.
#[test]
fn fora_da_inversao_o_aperto_e_tao_comparavel_quanto_o_arrastar() {
    let fraco = correr("plano_apertar_ponto_radial_local_origem_fraco");
    let arrasto = correr("plano_arrastar_radial_local_origem");
    let rel = |l: &Leitura| l.erro_max / l.max_oraculo.max(1e-12);
    assert!(
        fraco.movidos_nos > 100 && fraco.movidos_oraculo > 100,
        "vacuo: {} / {}",
        fraco.movidos_nos,
        fraco.movidos_oraculo
    );
    // ⚠️⚠️ **A régua é ABSOLUTA, e a troca é de 07/09.** Ela era
    // `rel(fraco) ≤ rel(arrasto) · 1,5` — uma razão cujo DENOMINADOR é a nossa
    // própria qualidade: quando a cura do `φ` da integração levou o arrasto de
    // `1,1·10⁻²` para `1,1·10⁻⁵` de erro relativo, a barra apertou-se sozinha
    // por `1000×` e o gate reprovou sobre produto que só tinha MELHORADO.
    // ⛔ E o aperto de força fraca nem podia acompanhar: ele move `0,004` contra
    // `0,33` do arrasto, logo a MESMA resolução de ficheiro vale `80×` mais em
    // unidades relativas (medido: `7,5·10⁻⁶` e `3,7·10⁻⁶` de erro ABSOLUTO, que
    // são a mesma coisa; `1,8·10⁻³` e `1,1·10⁻⁵` de erro relativo, que não são).
    for (nome, l) in [("aperto fraco", &fraco), ("arrasto", &arrasto)] {
        assert!(
            l.erro_max <= BARRA_DO_FICHEIRO,
            "{nome}: erra {:.3e} em posicao, contra a resolucao do ficheiro \
             ({BARRA_DO_FICHEIRO:.0e}) -- fora da inversao os dois reproduzem-se \
             ao ficheiro, e se o aperto nao o fizer ha' lei em falta",
            l.erro_max
        );
    }
    // ⛔ **O discriminador fica:** o irmão à força CHEIA erra `0,908` relativo —
    // é ele que a §5.2-ter explica pela ORDEM, e é por isso que este gate tem de
    // correr sobre a fixture que NÃO inverte uma única face.
    let cheio = correr("plano_apertar_ponto_radial_local_origem");
    assert!(
        rel(&cheio) > 0.5,
        "o aperto a forca cheia erra so' {:.3} -- sem a divergencia dele este \
         gate nao ILIBA nada",
        rel(&cheio)
    );
}

/// **O CHÃO DE RUÍDO de um traço** — quanto a nossa própria resposta se move
/// quando muda uma escolha que é NOSSA: a ordem em que as restrições são
/// resolvidas.
///
/// ⭐⭐⭐ **Gauss–Seidel não comuta**, e a ordem não vem do oráculo — vem de nós.
/// ⇒ para cada traço há um chão abaixo do qual *«o nosso erro»* deixa de medir a
/// lei e passa a medir a nossa ordenação. A grandeza é a **mesma** da barra de
/// paridade (pior diferença por vértice, em unidades do deslocamento máximo do
/// oráculo), para que as duas se possam comparar número a número.
fn chao_de_ruido(nome: &str) -> (f64, f64) {
    let indice = correr_posicoes_com(nome, None);
    let inversa = correr_posicoes_com(nome, Some("inversa"));
    let alvo = deformado(nome);
    let rest = repouso(traco(nome).s("superficie"));
    let max_o = alvo
        .iter()
        .zip(&rest)
        .map(|(a, r)| dist(*a, *r))
        .fold(0.0f64, f64::max);
    let erro = indice
        .iter()
        .zip(&alvo)
        .map(|(n, o)| dist(*n, *o))
        .fold(0.0f64, f64::max);
    let ruido = indice
        .iter()
        .zip(&inversa)
        .map(|(a, b)| dist(*a, *b))
        .fold(0.0f64, f64::max);
    (erro / max_o.max(1e-12), ruido / max_o.max(1e-12))
}

/// **SONDA — o resíduo de cada traço em unidades do que uma ORDEM ERRADA custa.**
///
/// ⛔⛔⛔ **Leia o que esta sonda NÃO é, antes de a usar.** A 1.ª redacção dela
/// chamava à coluna `ordem` um *chão de ruído* e partia o corpus em «ruído de
/// ordem» contra «lei em falta». **As duas coisas estavam erradas**, e a medição
/// derrubou-as no mesmo dia:
///
/// 1. **Inverter a ordem não é ruído — é OUTRA LEI, e uma lei errada.** A nossa
///    ordem é a do oráculo, e a prova é esta sonda: `plano_arrastar_radial_local`
///    erra `0,0713` contra o oráculo e move-se `0,2932` ao inverter a ordem, ou
///    seja *estamos QUATRO vezes mais perto do alvo do que a ordem inversa está
///    de nós.* Um chão de ruído nunca é maior que a distância ao alvo.
/// 2. **A partição não tinha VALE.** As razões dos `25` abertos são um contínuo
///    de `0,30` a `3,39` e o maior vazio é `0,92`, no topo, entre os dois últimos
///    — qualquer barra a meio seria escolhida, não medida (CLAUDE.md §0.0).
///
/// ⭐ **O que ela mede, e para o que serve:** a coluna `ordem` é *quanto custa
/// resolver as restrições na ordem errada*, na MESMA unidade da barra de
/// paridade, e a razão `erro/ordem` diz **quão estrutural** é o resíduo de cada
/// traço. Um traço com razão alta erra mais do que uma ordem errada custa — ali
/// falta lei, e ela não se esconde atrás da não-comutatividade. Um traço com
/// razão baixa vive num regime em que a ordem manda (a família do §5.2-ter).
///
/// ⇒ *é uma ORDENAÇÃO da fila, não um veredito por traço, e é por isso que não
/// tem gate.*
#[test]
#[ignore = "sonda"]
fn sonda_do_chao_de_ruido() {
    let mut linhas: Vec<(f64, String)> = Vec::new();
    for nome in todas() {
        let (erro, ordem) = chao_de_ruido(&nome);
        let razao = erro / ordem.max(1e-12);
        let marca = if erro <= BARRA_PARIDADE { "bate" } else { "" };
        linhas.push((
            if erro <= BARRA_PARIDADE { -1.0 } else { razao },
            format!("{nome:<46} {erro:>8.4} {ordem:>8.4} {razao:>7.2}  {marca}"),
        ));
    }
    linhas.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!(
        "{:<46} {:>8} {:>8} {:>7}",
        "traco (do mais ESTRUTURAL ao menos)", "erro", "ordem", "razao"
    );
    for (_, l) in linhas {
        println!("{l}");
    }
}

/// ⭐⭐ **GATE — o anel do ARNÊS vem pela ordem das FACES, não pela dos índices**
/// (espec §3.1).
///
/// ⛔⛔ **O gate de paridade não o apanha, e a razão é a forma dele:** os
/// [`ABERTOS`] só acusam um traço que se DEGRADE acima da folga ou que passe a
/// bater. Voltar a ordenar o anel por índice **melhora** quatro traços de esfera
/// e piora três, todos longe da barra ⇒ a lista fica verde sobre a lei errada.
/// *Um censo que só olha para um lado da mudança não gateia a lei, gateia a
/// regressão.*
///
/// ⚠️ **No plano as duas ordens coincidem ao bit** — o percurso face a face de um
/// vértice interior de grelha devolve `[S, O, E, N]`, que já é a ordem crescente
/// de índice —, e é por isso que este gate corre sobre a ESFERA.
#[test]
fn o_anel_do_arnes_vem_pela_ordem_das_faces() {
    let rest = repouso("esfera");
    let fs = faces("esfera", &rest);
    let an = aneis(rest.len(), &fs);

    // A primeira face que contém cada vértice, na ordem da lista de faces.
    let mut primeira: Vec<Option<usize>> = vec![None; rest.len()];
    for (i, f) in fs.iter().enumerate() {
        for &v in f {
            primeira[v as usize].get_or_insert(i);
        }
    }
    let mut fora_de_ordem = 0usize;
    for (v, anel) in an.iter().enumerate() {
        assert!(!anel.is_empty(), "vertice {v} sem anel");
        if anel.windows(2).any(|w| w[0] > w[1]) {
            fora_de_ordem += 1;
        }
        let f = &fs[primeira[v].expect("todo vertice esta' nalguma face")];
        let n = f.len();
        let k = f
            .iter()
            .position(|c| *c as usize == v)
            .expect("v na face dele");
        let (ant, seg) = (f[(k + n - 1) % n], f[(k + 1) % n]);
        assert_eq!(
            (anel[0], anel[1]),
            (ant, seg),
            "o anel de {v} nao comeca pelos dois cantos vizinhos na 1.a face dele: {anel:?}"
        );
    }
    assert!(
        fora_de_ordem > 0,
        "nenhum anel sai fora da ordem dos indices -- a fixtura nao separa as \
         duas leis, e este gate passaria com um `sort` la' dentro"
    );
}

/// Quantos vértices um traço move DENTRO e FORA do disco do pincel.
///
/// A régua, por inteiro (espec §14 gate 25): **movido** = `|u| > 1e-5` sobre as
/// posições a seis casas — a mesma com que o cabeçalho de cada fixture enche o
/// campo `movidos`; **disco** = posição de **repouso** a menos de `R` da **ponta
/// do caminho**, que é o cursor do passo simulado.
fn dentro_e_fora(nome: &str, pos: &[V3]) -> (usize, usize) {
    let t = traco(nome);
    let rest = repouso(t.s("superficie"));
    let r = t.f("raio");
    let ponta = *t.caminho.last().expect("caminho nao vazio");
    let (mut dentro, mut fora) = (0usize, 0usize);
    for (v, p) in pos.iter().enumerate() {
        if dist(*p, rest[v]) <= 1e-5 {
            continue;
        }
        if dist(rest[v], ponta) < r {
            dentro += 1;
        } else {
            fora += 1;
        }
    }
    (dentro, fora)
}

/// ⛔⛔⛔ **GATE 25 — UM PASSO DE ACELERAÇÃO NÃO MOVE NADA FORA DO DISCO**, e é o
/// CONTROLO de que a relaxação corre ANTES da integração (espec §5.2-quater).
///
/// Num traço de um passo de um modo que escreve aceleração, a malha com que a
/// relaxação se encontra está **em repouso**: todo par está exactamente no
/// comprimento de repouso e `τ` é zero ⇒ **todas as correcções que ela calcula
/// são identicamente zero**. O que sobra é a força, e ela só alcança o disco.
///
/// ⚠️⚠️ **É por isso que «o traço de um passo sai ao bit» NÃO prova que a rede de
/// restrições está certa** — foi a premissa da minha própria pergunta Q14, e ela
/// estava errada: aqueles traços **não exercitam uma única restrição de
/// distância**. Os três que falham movem entre `5` e `8,5` vezes mais vértices,
/// e a esmagadora maioria é material que força nenhuma tocou.
///
/// ⚠️ **Duas metades, e a 1.ª é o CONTROLO:** um port cuja relaxação corra DEPOIS
/// da integração passa a 2.ª e reprova a 1.ª.
#[test]
fn um_passo_de_aceleracao_nao_move_nada_fora_do_disco() {
    // (1.ª metade) os SETE traços dos cinco modos que escrevem aceleração.
    // ⚠️ **A §4.2 chama «modos de força» a SEIS** — o Expand é um deles e está do
    // OUTRO lado desta partição, porque escreve repouso e não aceleração.
    let aceleracao: [(&str, usize); 7] = [
        ("plano_arrastar_radial_local_1passo", 171),
        ("plano_arrastar_radial_local_massa2_1passo", 171),
        ("plano_arrastar_radial_local_forca05_1passo", 168),
        ("plano_empurrar_radial_local_1passo", 171),
        ("plano_inflar_radial_local_1passo", 171),
        ("plano_apertar_ponto_radial_local_1passo", 171),
        ("plano_apertar_linha_radial_local_1passo", 156),
    ];
    for (nome, esperado) in aceleracao {
        for (lado, pos) in [("nos", correr_posicoes(nome)), ("oraculo", deformado(nome))] {
            let (dentro, fora) = dentro_e_fora(nome, &pos);
            assert_eq!(
                fora, 0,
                "{nome} ({lado}): {fora} vertices movidos FORA do disco -- a \
                 relaxacao correu sobre uma malha que ja' tinha sido integrada"
            );
            assert_eq!(dentro, esperado, "{nome} ({lado}): movidos dentro do disco");
        }
    }

    // (2.ª metade) os TRÊS que escrevem âncora ou repouso: a resposta deles É a
    // rede. ⚠️ Os números exactos são os do ORÁCULO; os nossos ainda divergem
    // nos dois de âncora, e é isso que sobra na fila.
    let rede: [(&str, usize, usize); 3] = [
        ("plano_expandir_radial_local_1passo", 173, 675),
        ("plano_agarrar_radial_local_1passo", 173, 1151),
        ("plano_gancho_radial_local_1passo", 173, 1279),
    ];
    for (nome, d_o, f_o) in rede {
        let (d, f) = dentro_e_fora(nome, &deformado(nome));
        assert_eq!((d, f), (d_o, f_o), "{nome} (oraculo): dentro/fora do disco");
        let (dn, fnn) = dentro_e_fora(nome, &correr_posicoes(nome));
        assert!(
            fnn > 0,
            "{nome} (nos): {fnn} fora do disco -- a nossa relaxacao nao correu"
        );
        assert_eq!(dn, d_o, "{nome} (nos): o disco tem de mover-se todo");
    }
}

/// ⭐⭐⭐ **GATE 32 — a ORDEM DE VISITA é a da partição em células, e NÃO a ordem
/// crescente de índice** (espec §3.1-bis).
///
/// Cada vértice é próprio de **uma só** célula, logo a concatenação `célula 0 →
/// célula 1 → …` dos próprios de cada uma é uma **permutação** da malha. No plano
/// ela é a identidade **rodada** — a célula `1` fica com `[2080..4224]` e a `2`
/// com `[0..2079]` —, com **um único descenso**, exactamente no meio da grelha,
/// que é onde o pen-down das fixtures `_origem` está.
///
/// ⛔⛔ **Foi ela que o corpus estava a pagar:** com a ordem crescente `50` dos
/// `65` traços erravam acima da barra; com a ordem da partição são `15`, e `17`
/// saem praticamente ao bit. *A ordem de resolução não é um detalhe do solver —
/// é metade da lei.*
#[test]
fn a_ordem_de_visita_e_uma_permutacao_agrupada_por_celula() {
    for (sup, n_esperado, descensos) in [("plano", 4225usize, 1usize), ("esfera", 6050, 3)] {
        let ordem = ordem_de_visita(sup);
        assert_eq!(
            ordem.len(),
            n_esperado,
            "{sup}: a ordem de visita nao cobre a malha"
        );
        // É uma PERMUTAÇÃO: cada vértice aparece exactamente uma vez.
        let mut visto = vec![false; n_esperado];
        for &v in &ordem {
            let vi = v as usize;
            assert!(
                !visto[vi],
                "{sup}: o vertice {v} e' proprio de DUAS celulas"
            );
            visto[vi] = true;
        }
        assert!(
            visto.iter().all(|b| *b),
            "{sup}: ha' vertices que nenhuma celula reclama"
        );
        // ⛔ E NÃO é a ordem crescente: conta os descensos.
        let d = ordem.windows(2).filter(|w| w[0] > w[1]).count();
        assert_eq!(
            d, descensos,
            "{sup}: {d} descensos na ordem de visita, esperava {descensos} \
             (zero significaria a ordem crescente, que e' a lei ERRADA)"
        );
    }
}

/// ⭐⭐⭐ **GATE 33 — a NOSSA partição reproduz a do alvo, elemento a elemento**
/// (espec §3.1-bis).
///
/// A ordem de visita que [`ph2d_cloth::particao::ordem_de_visita`] deriva das
/// posições e das faces tem de ser **a mesma sequência** que a fixture de células
/// traz, nas DUAS malhas. ⇒ *é o que torna a lei aplicável a uma malha qualquer
/// da cena, e não só às duas do oráculo.*
///
/// ⚠️ **É a única maneira honesta de levar esta lei ao produto:** o adaptador não
/// tem fixture nenhuma para consultar, então ou ele deriva a partição pela mesma
/// lei, ou o produto e a bancada medem programas diferentes.
#[test]
fn a_nossa_particao_reproduz_a_do_alvo() {
    let compara = |sup: &str, nossa: &[u32], dele: &[u32]| -> Option<usize> {
        if nossa.len() != dele.len() {
            panic!("{sup}: {} vertices contra {}", nossa.len(), dele.len());
        }
        nossa.iter().zip(dele).position(|(a, b)| a != b)
    };

    // ⭐ O PLANO não tem ambiguidade nenhuma: `x = y = 3` e `z = 0`, o empate
    // entre `x` e `y` dá `Y` pela regra da espec, e é por `Y` que o alvo parte.
    let rest = repouso("plano");
    let fs = faces("plano", &rest);
    let nossa = ph2d_cloth::particao::ordem_de_visita(&rest, &fs);
    let dele = ordem_de_visita("plano");
    assert_eq!(
        compara("plano", &nossa, &dele),
        None,
        "plano: a nossa particao nao reproduz a do alvo"
    );

    // ⛔⛔ **A ESFERA é irresolúvel com a precisão que temos, e o gate diz
    // EXACTAMENTE em quê.** As posições de repouso vêm a seis casas, e nelas as
    // extensões de `x` e de `y` são iguais ao bit (`2,000001` as duas) contra
    // `2,000000` de `z`. A regra da §3.1-bis dá `Y` a esse empate; o alvo parte
    // por `X`. ⇒ *nos dados dele `x` era estritamente maior, e as seis casas
    // apagaram os bits que o decidiam.*
    let rest = repouso("esfera");
    let fs = faces("esfera", &rest);
    let dele = ordem_de_visita("esfera");
    let por_x = ph2d_cloth::particao::ordem_de_visita_com_eixo_raiz(&rest, &fs, Some(0));
    assert_eq!(
        compara("esfera", &por_x, &dele),
        None,
        "esfera: com o empate resolvido por X a particao tinha de reproduzir o alvo"
    );
    // ⚠️ **A segunda metade é o CONTROLO** — sem ela este gate estaria verde sobre
    // «qualquer eixo serve», e o achado (que a ambiguidade é de UM bit) evapora.
    let por_y = ph2d_cloth::particao::ordem_de_visita_com_eixo_raiz(&rest, &fs, Some(1));
    assert!(
        compara("esfera", &por_y, &dele).is_some(),
        "esfera: resolver o empate por Y tambem reproduz o alvo -- entao nao ha' \
         ambiguidade nenhuma e esta secao inteira esta' errada"
    );
    // ⚠️ E as duas dão o MESMO tamanho de folha, que é o que a simetria da esfera
    // impõe: *contar não discrimina aqui, e foi por isso que a 1.ª leitura desta
    // medição concluiu «bate» sobre uma partição espelhada.*
    let tamanhos = |o: &[u32]| o.len();
    assert_eq!(tamanhos(&por_x), tamanhos(&por_y));
}

/// O `|u|` de um vértice de repouso dado, passo a passo, nosso e do oráculo.
fn por_vertice(nome: &str, repouso_do_vertice: V3) -> (Vec<f64>, Vec<f64>) {
    let (rest, _, nossos) = correr_por_passo(nome);
    let v = (0..rest.len())
        .min_by(|a, b| {
            dist(rest[*a], repouso_do_vertice).total_cmp(&dist(rest[*b], repouso_do_vertice))
        })
        .expect("malha nao vazia");
    assert!(
        dist(rest[v], repouso_do_vertice) < 1e-6,
        "{nome}: nao ha' vertice em {repouso_do_vertice:?}"
    );
    let pp = por_passo(nome);
    let nosso = nossos.iter().map(|p| dist(p[v], rest[v])).collect();
    let alvo = pp.blocos.iter().map(|b| dist(b[v], rest[v])).collect();
    (nosso, alvo)
}

/// ⭐⭐⭐ **A ASSIMETRIA DE ESPELHO é um retrato da ORDEM, e nós temos de
/// a reproduzir** (espec §10.10).
///
/// Numa cena com simetria de espelho perfeita — a mesma malha, a mesma queda, a
/// mesma força — a única coisa que distingue os dois lados do traço é a **ORDEM**,
/// e ela é toda conduzida por ÍNDICE, que não é simétrico: a partição do plano
/// parte exactamente na fileira do pen-down (§3.1-bis), logo um dos lados é
/// visitado antes do outro, e o anel de cada vértice sai das faces por índice
/// crescente (§3.1).
///
/// ⇒ o oráculo desloca o vértice a `(0, +0,328125, 0)` de `0,00421` no passo `3`
/// e o espelhado dele de `0,00477` — **`13 %` acima**. ⛔ *Um port que não
/// reproduza a assimetria tem a ordem errada mesmo quando o `máx |u|` bate*, e é
/// por isso que este gate mede a RAZÃO entre os dois lados e não cada lado.
///
/// ⚠️ **A fixture é a `_parado`, de propósito:** ali a força só existe no passo 2,
/// e do 3 ao 12 tudo o que acontece é solver — *a assimetria não pode vir da fase
/// do gesto, porque ela está calada.*
///
/// ⚠️⚠️ **E é esta régua que QUALIFICA o «o solver está exonerado» que a fixture
/// `_parado` parecia dizer.** Ela bate a `0,001` de `err/máx`, o que exonera a
/// AMPLITUDE do solver; a assimetria de espelho mede a ORDEM dele, e aí nós
/// reproduzimo-la sem a acertar: `1,1102` contra `1,1333` no passo `3`, e a do
/// alvo **decai mais depressa** que a nossa (no passo `5` do Inflate são `1,0940`
/// contra `1,0570`). *Um número agregado pequeno não exonera a estrutura fina que
/// ele agrega.*
#[test]
fn a_assimetria_de_espelho_reproduz_a_ordem_do_oraculo() {
    for nome in [
        "plano_empurrar_radial_local_origem_parado",
        "plano_inflar_radial_local_origem_parado",
    ] {
        let (cima_n, cima_o) = por_vertice(nome, [0.0, 0.328_125, 0.0]);
        let (baixo_n, baixo_o) = por_vertice(nome, [0.0, -0.328_125, 0.0]);
        // O passo 3 é o primeiro em que a relaxação trabalha (§5.2-quater), e é
        // onde o oráculo tem a assimetria mais nítida.
        let k = 2usize;
        let (ro, rn) = (baixo_o[k] / cima_o[k], baixo_n[k] / cima_n[k]);
        assert!(
            (ro - 1.0).abs() > 0.05,
            "{nome}: o ORACULO nao e' assimetrico neste passo ({ro:.4}) -- a \
             fixtura nao produz o fenomeno e este gate seria vacuo"
        );
        assert!(
            (rn - 1.0).abs() > 0.05,
            "{nome}: NOS nao somos assimetricos ({rn:.4}) -- a nossa ordem nao e' \
             conduzida por indice, ou nao e' a do alvo de todo"
        );
        assert!(
            (rn - 1.0).signum() == (ro - 1.0).signum(),
            "{nome}: a assimetria vai para o LADO CONTRARIO ({rn:.4} contra {ro:.4})"
        );
        // ⏳ **O que sobra, medido e registado:** reproduzimos a assimetria e não
        // o valor dela. A folga é a mesma dos [`ABERTOS`] — ela só encolhe.
        let medido = match nome {
            "plano_empurrar_radial_local_origem_parado" => 0.0231,
            _ => 0.0161,
        };
        assert!(
            (rn - ro).abs() <= medido * FOLGA_ABERTO,
            "{nome}: a nossa assimetria e' {rn:.4} contra {ro:.4} do oraculo -- \
             o desvio {:.4} passa o medido {medido:.4} (folga {FOLGA_ABERTO}x)",
            (rn - ro).abs()
        );
    }
}

/// O `máx |u|` sobre a malha, por passo, nosso e do oráculo.
fn maximos_por_passo(nome: &str) -> (Vec<f64>, Vec<f64>) {
    let (rest, _, nossos) = correr_por_passo(nome);
    let pp = por_passo(nome);
    let pico = |bloco: &Vec<V3>| {
        bloco
            .iter()
            .zip(&rest)
            .map(|(p, r)| dist(*p, *r))
            .fold(0.0f64, f64::max)
    };
    (
        nossos.iter().map(pico).collect(),
        pp.blocos.iter().map(pico).collect(),
    )
}

/// ⭐⭐⭐ **GATE 35 — o SOLVER SOZINHO, sem uma única leitura da malha pela fase do
/// gesto** (espec §5.7-bis · §10.10).
///
/// Nas fixtures `_parado` a força só existe no passo `2`; do `3` ao `12` não há
/// força, nem curva de queda, nem normal — **só retenção e projecções**. ⇒ é o
/// gate mais forte do corpus para a relaxação.
///
/// ⚠️ **É também o DISCRIMINADOR:** um port que passe aqui e falhe o
/// `plano_empurrar_radial_local_origem` tem o defeito na **fase do gesto**; um
/// que falhe aqui tem-no no **solver**, e o primeiro passo a divergir diz onde.
/// ⛔ *Um port não pode falhar os dois e declarar «é a relaxação» sem correr
/// este.*
#[test]
fn o_solver_sozinho_reproduz_o_oraculo_nos_doze_passos() {
    for nome in [
        "plano_empurrar_radial_local_origem_parado",
        "plano_inflar_radial_local_origem_parado",
    ] {
        let (rest, _, nossos) = correr_por_passo(nome);
        let pp = por_passo(nome);
        assert_eq!(nossos.len(), pp.blocos.len(), "{nome}: passos diferentes");
        for (k, (nosso, alvo)) in nossos.iter().zip(&pp.blocos).enumerate() {
            let max_o = alvo
                .iter()
                .zip(&rest)
                .map(|(p, r)| dist(*p, *r))
                .fold(0.0f64, f64::max);
            if max_o <= 0.0 {
                continue;
            }
            let erro = nosso
                .iter()
                .zip(alvo)
                .map(|(a, b)| dist(*a, *b))
                .fold(0.0f64, f64::max);
            assert!(
                erro / max_o <= BARRA_PARIDADE,
                "{nome}: passo {} erra {:.4} da barra {BARRA_PARIDADE} -- o \
                 defeito esta' no SOLVER, e este e' o primeiro passo a divergir",
                k + 1,
                erro / max_o
            );
        }
    }
}

/// ⭐⭐ **GATE 36 — a resposta NÃO é proporcional ao impulso** (espec §10.10).
///
/// O impulso do passo `2` escala com o **quadrado** da força (`0,2500` e
/// `0,0625` para força `0,5` e `0,25`) e o `|u|` do passo `12` escala `0,6677` e
/// `0,3388`. ⇒ *um port que compare só o fim do traço não pode concluir nada
/// sobre uma lei.*
///
/// ⛔⛔ **A barra é a do FICHEIRO, não a do `f32`:** o oráculo lê `0,2499924` e
/// `0,0624943` a seis casas, e uma barra de `f32` **reprovaria a fixture que a
/// define**. `±2·10⁻⁵` sai da resolução do ficheiro.
#[test]
fn a_resposta_nao_e_proporcional_ao_impulso() {
    const BARRA_DO_FICHEIRO: f64 = 2e-5;
    let pen_down = [0.0, 0.0, 0.0];
    let (cheia, _) = por_vertice("plano_empurrar_radial_local_origem", pen_down);
    for (nome, razao_impulso) in [
        ("plano_empurrar_radial_local_origem_forca05", 0.25),
        ("plano_empurrar_radial_local_origem_forca025", 0.0625),
        ("plano_empurrar_radial_local_origem_massa2", 0.5),
    ] {
        let (nosso, alvo) = por_vertice(nome, pen_down);
        // (1.ª metade) o passo 2 é aritmética da fase do gesto: exacta.
        let (r_n, r_o) = (nosso[1] / cheia[1], alvo[1] / cheia[1]);
        assert!(
            (r_o - razao_impulso).abs() <= BARRA_DO_FICHEIRO,
            "{nome}: o ORACULO da' {r_o:.7} e nao {razao_impulso} -- a fixtura \
             nao e' o que este gate julga"
        );
        assert!(
            (r_n - r_o).abs() <= BARRA_DO_FICHEIRO,
            "{nome}: o nosso impulso do passo 2 e' {r_n:.7} contra {r_o:.7}"
        );
        // (2.ª metade) o fim do traço vive noutro regime, e a razão NÃO é a mesma.
        let fim = alvo[alvo.len() - 1] / cheia[cheia.len() - 1];
        assert!(
            (fim - razao_impulso).abs() > 10.0 * BARRA_DO_FICHEIRO,
            "{nome}: o fim do traco escala {fim:.4} como o impulso ({razao_impulso}) \
             -- entao nao ha' nao-linearidade e esta secao esta' errada"
        );
    }
}

/// ⭐ **GATE 37 — uma projecção vale `15,1 %` do planalto, e é a régua de
/// qualquer resíduo** (espec §5.2-bis · §10.10).
///
/// No mesmo traço, `Global` (`5` projecções por restrição) e `Local` (`10`) dão a
/// **mesma malha no passo 2** — ali a rede ainda não agiu — e planaltos na razão
/// `0,8493`.
///
/// ⇒ ⚠️ **a conversão traz o VÃO da régua dentro:** os `15,1 %` medem **`5`**
/// projecções a mais, logo um resíduo de `x %` no planalto vale `5·x/15,1`
/// projecções — a `4 %` isso é `1,3`.
///
/// ⚠️ **A 1.ª metade é o CONTROLO da régua:** se o passo `2` já diferir, o defeito
/// não é a contagem de projecções.
#[test]
fn uma_projeccao_vale_quinze_por_cento_do_planalto() {
    let l = por_passo("plano_empurrar_radial_local_origem");
    let g = por_passo("plano_empurrar_radial_global_origem");
    // (1.ª metade) no passo 2 as duas áreas dão a MESMA malha, sobre os 4 225.
    let diff = l.blocos[1]
        .iter()
        .zip(&g.blocos[1])
        .map(|(a, b)| dist(*a, *b))
        .fold(0.0f64, f64::max);
    assert!(
        diff < 5e-7,
        "as duas areas ja' diferem no passo 2 (max {diff:.7}) -- o controlo da \
         regua caiu, e a razao abaixo nao mede projeccoes"
    );
    // (2.ª metade) os PLANALTOS — a série do vértice do pen-down (§10.10),
    // ⛔ não o `máx |u|` sobre a malha, que é a régua do gate 38.
    // ⚠️ **Cada um no SEU passo de planalto** — o `Local` culmina no passo `7`
    // (`0,23968`) e o `Global` no `9` (`0,28222`) —, e a razão é `Local/Global`.
    // *Comparar os dois no mesmo passo lê `1,17` e não `0,85`.*
    let planalto = |nome: &str| {
        let (nosso, alvo) = por_vertice(nome, [0.0; 3]);
        let pico = |v: &[f64]| v.iter().copied().fold(0.0f64, f64::max);
        (pico(&nosso), pico(&alvo))
    };
    let (nl, ol) = planalto("plano_empurrar_radial_local_origem");
    let (ng, og) = planalto("plano_empurrar_radial_global_origem");
    assert!(
        (ol / og - 0.8493).abs() < 5e-4,
        "o ORACULO da' {:.4} e nao 0,8493 -- a fixtura nao e' o que este gate julga",
        ol / og
    );
    let razao = nl / ng;
    assert!(
        (razao - 0.8493).abs() < 5e-3,
        "a nossa razao Local/Global do planalto e' {razao:.4} contra 0,8493 do \
         oraculo -- uma projeccao deixou de valer 15,1 %"
    );
}

/// **GATE 38 — sem memória de velocidade o traço deixa de assentar** (espec §5.3
/// · §5.4 · §10.10).
///
/// ⚠️ **A régua é o `máx |u|` sobre a MALHA, por passo** — ⛔ não o vértice do
/// pen-down, onde as duas configurações passam por um máximo e o discriminador
/// desaparece.
///
/// ⛔⛔ **E a régua da 2.ª metade é o ARGMAX, nunca «decresce depois do
/// máximo»:** a cauda do `_origem` **volta a subir** nos dois últimos passos,
/// porque o vértice que realiza o máximo **viaja com o cursor** — exigir
/// decrescimento reprovaria o próprio oráculo.
#[test]
fn sem_memoria_de_velocidade_o_traco_deixa_de_assentar() {
    // (a) `damping = 1`: estritamente crescente nos onze passos simulados.
    let (nosso, alvo) = maximos_por_passo("plano_empurrar_radial_local_origem_amort1");
    for (lado, seq) in [("oraculo", &alvo), ("nos", &nosso)] {
        assert!(
            seq[1..].windows(2).all(|w| w[1] > w[0]),
            "{lado}: com damping 1 a sequencia tinha de ser estritamente crescente: {seq:?}"
        );
    }
    // (b) `damping = 0,01`: o ORÁCULO tem o máximo dos onze no passo 6 e a cauda
    // volta a SUBIR — é o controlo da régua.
    let (nosso, alvo) = maximos_por_passo("plano_empurrar_radial_local_origem");
    let argmax = |seq: &[f64]| {
        (1..seq.len())
            .max_by(|a, b| seq[*a].total_cmp(&seq[*b]))
            .expect("passos")
            + 1
    };
    assert_eq!(argmax(&alvo), 6, "o ORACULO assenta no passo 6: {alvo:?}");
    let n = alvo.len();
    assert!(
        alvo[n - 1] > alvo[n - 2],
        "a cauda do ORACULO tinha de VOLTAR A SUBIR -- o vertice do maximo viaja \
         com o cursor, e exigir decrescimento reprovaria o proprio oraculo"
    );

    // ⏳ **ABERTO, e é o mesmo resíduo de `5 %` do Push:** nós assentamos **um
    // passo mais cedo** (`5`) e **mais baixo** (`0,2627` contra `0,27794`), e a
    // nossa cauda **não** volta a subir no último passo. ⚠️ *A forma bate quase
    // toda e o valor não* — que é exactamente o que o gate 38 avisa que pode
    // acontecer: «um port cuja retenção esteja errada pode acertar a forma e
    // falhar o valor».
    assert!(
        argmax(&nosso).abs_diff(6) <= 1,
        "nos assentamos no passo {} -- a mais de um passo do oraculo, o defeito \
         deixou de ser o residuo conhecido: {nosso:?}",
        argmax(&nosso)
    );
    let pico_n = nosso.iter().copied().fold(0.0f64, f64::max);
    let pico_o = alvo.iter().copied().fold(0.0f64, f64::max);
    assert!(
        (1.0 - pico_n / pico_o) <= 0.055 * FOLGA_ABERTO,
        "o nosso planalto e' {:.4} do oraculo -- o defice passou o medido de 5,5 %",
        pico_n / pico_o
    );
}

/// ⭐⭐ **GATE 24 — a razão `2R` do Push, e a igualdade Push/Inflate no 1.º passo
/// simulado** (espec §4.2-bis · §10.7).
///
/// No passo `2` dos dois traços o vértice do pen-down move `0,06543` e `0,09347`,
/// razão **`0,7000 = 2·R`** — é o Push a empurrar `2R` contra o Inflate a empurrar
/// `1`.
///
/// ⚠️⚠️ **E a divergência entre os dois só pode começar no passo `3`.** Numa folha
/// **plana e em repouso** a normal da ÁREA e a normal do VÉRTICE são a mesma
/// coisa; ⇒ *se um port os separa já no passo `2`, ele está a ler duas normais
/// diferentes onde só existe uma.*
#[test]
fn a_razao_do_push_e_dois_raios_e_os_dois_modos_so_divergem_no_passo_tres() {
    let (emp_n, emp_o) = por_vertice("plano_empurrar_radial_local_origem", [0.0; 3]);
    let (inf_n, inf_o) = por_vertice("plano_inflar_radial_local_origem", [0.0; 3]);
    let r = traco("plano_empurrar_radial_local_origem").f("raio");
    for (lado, emp, inf) in [("oraculo", &emp_o, &inf_o), ("nos", &emp_n, &inf_n)] {
        let razao = emp[1] / inf[1];
        assert!(
            (razao - 2.0 * r).abs() < 1e-4,
            "{lado}: a razao Push/Inflate no passo 2 e' {razao:.5} e nao 2R = {:.5}",
            2.0 * r
        );
    }
    // ⚠️ A DIREÇÃO tem de ser a mesma no passo 2 — a divergência começa no 3.
    let (rest, _, nossos) = correr_por_passo("plano_empurrar_radial_local_origem");
    let (_, _, inflar) = correr_por_passo("plano_inflar_radial_local_origem");
    let v = (0..rest.len())
        .min_by(|a, b| dist(rest[*a], [0.0; 3]).total_cmp(&dist(rest[*b], [0.0; 3])))
        .expect("malha");
    let unit_de = |p: V3, q: V3| {
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-30);
        [d[0] / n, d[1] / n, d[2] / n]
    };
    let (a, b) = (
        unit_de(nossos[1][v], rest[v]),
        unit_de(inflar[1][v], rest[v]),
    );
    let cos = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    assert!(
        (cos.abs() - 1.0).abs() < 1e-9,
        "no passo 2 as duas direccoes ja' diferem (cos {cos:.9}) -- numa folha \
         plana em repouso a normal da AREA e a do VERTICE sao a mesma coisa"
    );
}

/// **A distância entre duas direcções UNITÁRIAS**, `‖û − v̂‖` — a régua irmã do
/// §10.11, a que **não** depende do `a_v` do ajuste.
///
/// ⚠️ Ela existe porque o resíduo do ajuste mede a direcção **e** a amplitude ao
/// mesmo tempo, logo os dígitos dele só se reproduzem com a queda do §4.1 já
/// exacta. Esta lê só a direcção, e o vão que interessa é de ordens de grandeza
/// (`10⁻⁵` contra `10⁻¹`), nunca o dígito.
fn desvio_de_direccao(a: V3, b: V3) -> f64 {
    let (a, b) = (unit(a), unit(b));
    dist(a, b)
}

/// A mediana de uma amostra (a régua do §10.11 para o campo por vértice).
fn mediana(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// ⭐⭐⭐ **GATE 39 — AS NORMAIS SÃO AS DA SUPERFÍCIE QUE O TRAÇO ENCONTROU**
/// (espec §14 gate 39, §4.2-ter, §10.11).
///
/// **Três metades, e cada uma mata uma leitura diferente:**
///
/// **(b) o Push** — em `plano_empurrar_radial_local_origem` a normal da área é
/// `(0, 0, 1)` a cinco casas em **todos** os passos em que o gesto dispara, sobre
/// uma folha que já afundou `0,2`. ⛔ Um port que some as normais de AGORA lê ali
/// um vector inclinado, porque a cova o inclina.
///
/// **(c) e não são as do OBJECTO, são as do TRAÇO** — em
/// `plano_inflar_radial_local_1passo_2tracos` a direcção por vértice do 2.º traço
/// bate as normais da malha **no pen-down dele** e **não** as planas, e o vão é
/// de ordens de grandeza. ⛔ **É esta metade que apanha um port que fotografe as
/// normais uma vez, no início da SESSÃO:** ele passa (a) e (b) e reprova aqui.
///
/// **(a) o Inflate** — a metade edificável dela é o DISCRIMINANTE: no fim de
/// `plano_inflar_radial_local_origem` as duas leis candidatas (normais de repouso
/// do traço · normais de agora) estão a uma distância de direcção da ordem de
/// `10⁻¹` por vértice, e é com as de repouso que os doze passos saem à resolução
/// do ficheiro (gate 41). ⚠️ **Sem o discriminante o gate 41 seria vácuo naquele
/// traço** — uma paridade que passa com as duas leis não escolhe nenhuma.
/// ⛔ A forma do §10.11 (resíduo do ajuste por mínimos quadrados) mede a direcção
/// **e** a amplitude ao mesmo tempo; a régua irmã, que é esta, isola a direcção.
#[test]
fn as_normais_sao_as_da_superficie_que_o_traco_encontrou() {
    // ── (b) o Push, sobre a folha já afundada ──────────────────────────────
    let nome = "plano_empurrar_radial_local_origem";
    let pp = por_passo(nome);
    let rest = repouso("plano");
    let (_, _, nossos, gestos) = correr_com_blocos(nome, None, Some(&pp.caminho));
    let mut disparos = 0;
    for (k, g) in gestos.iter().enumerate() {
        if norm(*g) == 0.0 {
            continue;
        }
        disparos += 1;
        let desvio = desvio_de_direccao(*g, [0.0, 0.0, 1.0]);
        assert!(
            desvio < 1e-5,
            "{nome}: no passo {} a normal da area e' [{:.6} {:.6} {:.6}], a \
             {desvio:.6} da vertical -- com as normais de AGORA a cova inclina-a",
            k + 1,
            g[0],
            g[1],
            g[2]
        );
    }
    assert!(
        disparos >= 4,
        "so' {disparos} passos dispararam (anti-vacuo)"
    );
    // E a folha AFUNDOU de verdade — senão a asserção acima é sobre um plano.
    let cova = nossos
        .last()
        .expect("sem passos")
        .iter()
        .zip(&rest)
        .map(|(p, r)| dist(*p, *r))
        .fold(0.0f64, f64::max);
    assert!(
        cova > 0.15,
        "a folha so' afundou {cova:.4} -- num plano em repouso as duas leis \
         coincidem ao bit e este gate nao distingue nada"
    );

    // ── (c) as normais são as do TRAÇO, não as do objecto ──────────────────
    let nome = "plano_inflar_radial_local_1passo_2tracos";
    let fs = faces("plano", &rest);
    let planas = normais(&rest, &fs);
    // ⚠️⚠️ **A malha que o 1.º traço deixou é a do ORÁCULO**, não a da nossa
    // corrida: `plano_inflar_radial_local_1passo` é literalmente o 1.º traço
    // desta fixture (mesmo caminho, mesmos parâmetros, `tracos 1` contra `2`).
    // ⛔ Com a nossa, o `u` do 2.º traço leva o nosso resíduo do 1.º (`~10⁻⁶`)
    // dentro dele, e num vértice que se move `10⁻⁵` isso É a direcção: a mediana
    // subia de `10⁻⁵` para `6·10⁻⁴` e o vão caía de `18 000×` para `451×`.
    // *Um instrumento que mede uma direcção do alvo não pode ter o nosso erro no
    // numerador.*
    let depois_do_1o = deformado("plano_inflar_radial_local_1passo");
    let do_2o_traco = normais(&depois_do_1o, &fs);
    let final_do_alvo = deformado(nome);
    let (mut contra_traco, mut contra_planas): (Vec<(f64, f64)>, Vec<f64>) =
        (Vec::new(), Vec::new());
    for v in 0..rest.len() {
        let u = [
            final_do_alvo[v][0] - depois_do_1o[v][0],
            final_do_alvo[v][1] - depois_do_1o[v][1],
            final_do_alvo[v][2] - depois_do_1o[v][2],
        ];
        // Só quem se moveu tem direcção (a régua do campo `movidos` das fixtures).
        if norm(u) <= 1e-5 {
            continue;
        }
        contra_traco.push((norm(u), desvio_de_direccao(u, do_2o_traco[v])));
        contra_planas.push(desvio_de_direccao(u, planas[v]));
    }
    assert!(
        contra_traco.len() >= 100,
        "so' {} vertices se moveram no 2.º traco (anti-vacuo)",
        contra_traco.len()
    );
    let (m_traco, m_planas) = (
        mediana(contra_traco.iter().map(|(_, d)| *d).collect()),
        mediana(contra_planas),
    );
    assert!(
        m_traco < 1e-4,
        "{nome}: a direccao do 2.º traco desvia {m_traco:.6} das normais da malha \
         que ELE encontrou -- deviam ser as mesmas"
    );
    assert!(
        m_planas > 0.1,
        "{nome}: as normais PLANAS desviam so' {m_planas:.6} -- sem vao nao ha' \
         nada a distinguir, e a metade (c) fica vacua"
    );
    assert!(
        m_planas / m_traco > 1e3,
        "{nome}: o vao e' de {:.0}x -- a espec mede ordens de grandeza",
        m_planas / m_traco
    );

    // ── (a) o Inflate: o DISCRIMINANTE das duas leis no fim do traço ───────
    let nome = "plano_inflar_radial_local_origem";
    let fim = deformado(nome);
    let agora = normais(&fim, &fs);
    let desvios: Vec<f64> = (0..rest.len())
        .filter(|v| dist(rest[*v], fim[*v]) > 1e-5)
        .map(|v| desvio_de_direccao(planas[v], agora[v]))
        .collect();
    assert!(
        desvios.len() >= 100,
        "anti-vacuo: {} movidos",
        desvios.len()
    );
    let m = mediana(desvios);
    assert!(
        m > 0.05,
        "{nome}: as normais de repouso e as de agora so' diferem {m:.6} -- o \
         gate 41 nao escolheria entre as duas leis neste traco"
    );
}

/// ⭐⭐⭐ **GATE 40 — O PUSH CALA-SE QUANDO A COVA PASSA O DISCO DE AMOSTRAGEM, E
/// VOLTA A DISPARAR** (espec §14 gate 40, §4.2-bis (8), §10.11).
///
/// A régua, por inteiro: no passo `k`, `min_v |p_v − c_k|` com `p_v` as posições
/// do bloco `k−1` do ficheiro por passo (a malha com que o passo começa), `c_k` o
/// `k`-ésimo ponto do caminho, o mínimo sobre **todos** os vértices e a distância
/// **3D** — contra `R · «Normal Radius»` (`0,175` com as omissões).
///
/// **(a) o PADRÃO** de disparo tem de bater o do oráculo passo a passo. ⛔ Um
/// port que se cale para sempre à primeira falha reprova: o oráculo **volta** a
/// disparar quando o cursor avança para terreno pouco afundado.
///
/// **(b) o LIMIAR não é escolhido:** o maior `min_v` de um passo em que NÓS
/// disparámos e o menor de um em que nos calámos deixam entre si um vão de
/// `0,003` — `1,6 %` do próprio disco —, e `R · 0,5` cai lá dentro. ⚠️ **A
/// classificação de (b) é a NOSSA**, de propósito: classificá-la pelo limiar
/// tornaria a asserção circular, e um port que empurre em todos os passos
/// passaria por vacuidade — é o que os dois censos abaixo impedem.
#[test]
fn o_push_cala_se_quando_a_cova_passa_o_disco_e_volta_a_disparar() {
    let (mut com, mut sem) = (0.0f64, f64::INFINITY);
    let (mut n_com, mut n_sem) = (0usize, 0usize);
    let mut alcance_comum = 0.0f64;
    for nome in &POR_PASSO_DE_FORCA[..7] {
        let pp = por_passo(nome);
        let t = traco(nome);
        let sup = t.s("superficie").to_string();
        let rest = repouso(&sup);
        let alcance = t.pincel().raio * RAIO_DA_NORMAL;
        alcance_comum = alcance;
        let vista = eixo_da_vista(&sup);
        let (_, _, _, gestos) = correr_com_blocos(nome, None, Some(&pp.caminho));
        for (k, gesto) in gestos.iter().enumerate().skip(1) {
            let (c, prev) = (pp.caminho[k], pp.caminho[k - 1]);
            let d3 = [c[0] - prev[0], c[1] - prev[1], c[2] - prev[2]];
            // ⛔ O passo SEM movimento não escreve força por outra razão (§4.3) e
            // fica fora desta régua — é o que a espec chama «os 67 passos com o
            // cursor em movimento».
            if norm(projecta(d3, vista)) == 0.0 {
                continue;
            }
            // `blocos[k]` e' a malha DEPOIS do passo `k+1` (1-based) ⇒ a malha
            // com que o passo `k` comeca e' `blocos[k-1]`, e o repouso no 1.º.
            let antes: &[V3] = if k == 0 { &rest } else { &pp.blocos[k - 1] };
            let min = antes
                .iter()
                .map(|p| dist(*p, c))
                .fold(f64::INFINITY, f64::min);
            let nosso = norm(*gesto) > 0.0;
            assert_eq!(
                nosso,
                min <= alcance,
                "{nome}: no passo {} o disco mede {min:.5} contra {alcance:.5} e \
                 nos {} -- o padrao do oraculo e' 'dispara sse a cova nao passou \
                 o disco', e ele VOLTA a disparar",
                k + 1,
                if nosso { "disparamos" } else { "calamo-nos" }
            );
            if nosso {
                com = com.max(min);
                n_com += 1;
            } else {
                sem = sem.min(min);
                n_sem += 1;
            }
        }
    }
    // ⛔ Anti-vácuo dos DOIS lados: um port que empurre sempre esvazia `n_sem`, e
    // um que se cale para sempre esvazia `n_com`.
    assert_eq!(
        (n_com, n_sem),
        (53, 14),
        "o censo dos sete tracos de empurrar mudou -- a espec mede 53 passos com \
         gesto e 14 calados sobre os 67 com o cursor em movimento"
    );
    assert!(
        com < alcance_comum && alcance_comum <= sem,
        "o limiar {alcance_comum:.5} nao separa: com gesto ate' {com:.5}, sem \
         gesto desde {sem:.5}"
    );
    assert!(
        sem - com < 0.005,
        "o vao e' {:.5} -- a espec mede 0,00273, e um vao largo diria que a \
         constante e' escolhida em vez de medida",
        sem - com
    );
}

/// ⭐⭐ **GATE 41 — OS DEZ TRAÇOS DE EMPURRAR/INFLAR POR PASSO, À RESOLUÇÃO DO
/// FICHEIRO** (espec §14 gate 41, §10.11).
///
/// Com os gates 39 e 40 honrados, os dez traços reproduzem-se sobre a **malha
/// inteira** e nos **doze** passos com `err_max ≤ 5·10⁻⁶` — ⛔ não «na barra do
/// gate 15»: aqui a barra é a discretização de seis casas do próprio ficheiro,
/// porque a lei é exacta.
///
/// ⚠️ **Isto substitui a leitura antiga de que Push e Inflate ficavam `5`–`9 %`
/// abaixo:** aquele défice era o vector do gesto, não a resposta ao esticão.
#[test]
fn os_dez_tracos_de_empurrar_e_inflar_saem_a_resolucao_do_ficheiro() {
    for nome in POR_PASSO_DE_FORCA {
        let pp = por_passo(nome);
        let sup = traco(nome).s("superficie").to_string();
        let rest = repouso(&sup);
        let (_, _, nossos, _) = correr_com_blocos(nome, None, Some(&pp.caminho));
        assert_eq!(nossos.len(), pp.blocos.len(), "{nome}: passos diferentes");
        for (k, (nosso, alvo)) in nossos.iter().zip(&pp.blocos).enumerate() {
            let erro = nosso
                .iter()
                .zip(alvo)
                .map(|(a, b)| dist(*a, *b))
                .fold(0.0f64, f64::max);
            assert!(
                erro <= BARRA_DO_FICHEIRO,
                "{nome}: passo {} erra {erro:.6} sobre a malha inteira, contra a \
                 resolucao do ficheiro ({BARRA_DO_FICHEIRO:.0e})",
                k + 1
            );
        }
        // Anti-vácuo: o traço tem de ter deformado.
        let max_o = pp
            .blocos
            .last()
            .expect("sem blocos")
            .iter()
            .zip(&rest)
            .map(|(p, r)| dist(*p, *r))
            .fold(0.0f64, f64::max);
        assert!(max_o > 0.05, "{nome}: o oraculo mal deformou ({max_o:.4})");
    }
}

/// ⭐ **GATE 42 — A DIRECÇÃO DO PUSH É A NORMAL DA ÁREA, NÃO A DA VISTA** (espec
/// §14 gate 42, §4.2-bis, §4.2-ter, §10.11) — e só uma superfície CURVA o diz.
///
/// Em `esfera_empurrar_radial_local_1passo` a relaxação é um no-op por construção
/// (um passo simulado a partir do repouso) ⇒ o deslocamento **é** `u · f(v) · dt`,
/// e o campo inteiro é COLINEAR. Essa direcção tem de estar praticamente sobre a
/// normal da superfície no cursor e **longe** da normal da vista.
///
/// ⚠️ **O controlo ao lado é obrigatório:** sem o `esfera_inflar_radial_local_1passo`
/// (direcção por VÉRTICE, campo não-colinear), um port que ponha a direcção certa
/// pela razão errada — o vector cursor→centro do objecto, que nesta cena coincide
/// com a normal — passaria. ⭐ E a razão dos dois módulos é o `2R` do §4.2-bis (7).
#[test]
fn a_direccao_do_push_e_a_normal_da_area_nao_a_da_vista() {
    let rest = repouso("esfera");
    let vista = eixo_da_vista("esfera");
    let t = traco("esfera_empurrar_radial_local_1passo");
    // ⚠️ **O cursor do passo SIMULADO**, não o do pen-down: o 1.º passo nunca
    // simula (§1 fase 0), e o disco da normal é centrado no cursor do passo que
    // corre. Na esfera os dois estão a `34,9°` um do outro — medi-lo pelo
    // pen-down lê exactamente esse ângulo e acusa a lei certa.
    let cursor = t.caminho[1];
    // A normal da superfície no cursor: a esfera das fixtures é centrada, logo é
    // radial — e o centroide do repouso é quem o diz, não um número escrito.
    let centro = {
        let n = rest.len() as f64;
        let mut c = [0.0; 3];
        for p in &rest {
            for k in 0..3 {
                c[k] += p[k] / n;
            }
        }
        c
    };
    let n_superficie = unit([
        cursor[0] - centro[0],
        cursor[1] - centro[1],
        cursor[2] - centro[2],
    ]);
    let campo = |nome: &str| -> Vec<(usize, V3)> {
        let fim = deformado(nome);
        (0..rest.len())
            .map(|v| {
                (
                    v,
                    [
                        fim[v][0] - rest[v][0],
                        fim[v][1] - rest[v][1],
                        fim[v][2] - rest[v][2],
                    ],
                )
            })
            .filter(|(_, u)| norm(*u) > 1e-5)
            .collect()
    };
    let empurrar = campo("esfera_empurrar_radial_local_1passo");
    let inflar = campo("esfera_inflar_radial_local_1passo");
    assert!(
        empurrar.len() >= 100 && inflar.len() >= 100,
        "anti-vacuo: {} e {} vertices movidos",
        empurrar.len(),
        inflar.len()
    );
    // A direcção do Push: a soma normalizada do campo (ele é colinear).
    let mut soma = [0.0; 3];
    for (_, u) in &empurrar {
        for k in 0..3 {
            soma[k] += u[k];
        }
    }
    let u_hat = unit(soma);
    let graus = |a: V3, b: V3| -> f64 {
        let c: f64 = (0..3).map(|k| a[k] * b[k]).sum();
        c.clamp(-1.0, 1.0).acos().to_degrees()
    };
    // O Push empurra para DENTRO ⇒ a direcção é `−n̂_área`.
    let ao_normal = graus(unit([-u_hat[0], -u_hat[1], -u_hat[2]]), n_superficie);
    let a_vista = graus(unit([-u_hat[0], -u_hat[1], -u_hat[2]]), unit(vista));
    assert!(
        ao_normal < 0.1,
        "a direccao do Push esta' a {ao_normal:.3}° da normal da superficie"
    );
    assert!(
        a_vista > 10.0,
        "a direccao do Push esta' a {a_vista:.3}° da normal da VISTA -- nesta \
         cena as duas estao a ~17,4°, e um port que use a vista le' ~0°"
    );
    // ⛔ O CONTROLO: o campo do Push é colinear e o do Inflate NÃO é — se os dois
    // fossem colineares, a direcção do Push podia vir de qualquer coisa que
    // coincida com a normal nesta cena.
    let espalhamento = |campo: &[(usize, V3)]| -> f64 {
        let mut s = [0.0; 3];
        for (_, u) in campo {
            let d = unit(*u);
            for k in 0..3 {
                s[k] += d[k];
            }
        }
        1.0 - norm(s) / campo.len() as f64
    };
    let (e_push, e_inflar) = (espalhamento(&empurrar), espalhamento(&inflar));
    assert!(
        e_push < 1e-5,
        "o campo do Push nao e' colinear ({e_push:.2e}) -- ele tem UM vector por passo"
    );
    assert!(
        e_inflar > 1e-3,
        "o campo do Inflate e' colinear ({e_inflar:.2e}) -- ele le' a normal de \
         CADA vertice, e sem isso o controlo deste gate nao controla nada"
    );
    // E a direcção por vértice do Inflate é a normal do próprio vértice.
    let fs = faces("esfera", &rest);
    let nrm = normais(&rest, &fs);
    let m = mediana(
        inflar
            .iter()
            .map(|(v, u)| desvio_de_direccao(*u, nrm[*v]))
            .collect(),
    );
    assert!(
        m < 1e-3,
        "a direccao do Inflate desvia {m:.6} da normal de repouso de cada vertice"
    );
    // ⭐⭐ **E a NOSSA direcção é a mesma** — sem esta metade o gate é uma
    // afirmação sobre o oráculo que mutação nenhuma do nosso lado pode matar.
    //
    // ⛔⛔ **E ela lê a SAÍDA, não a normal da área que guardamos.** A 1.ª
    // redacção perguntava ao `PincelTecido::normal_da_area` — o estado do
    // PRODUTOR —, e a mutação que troca a direcção do Push pela da vista
    // **SOBREVIVEU-LHE**: a normal da área continua certa, e é o consumidor que
    // a deixa de usar. *Um gate que pergunta ao produtor é cego a quem lê.*
    let nossas = correr_posicoes("esfera_empurrar_radial_local_1passo");
    let mut soma_nossa = [0.0; 3];
    for (v, u) in &empurrar {
        let _ = u;
        for k in 0..3 {
            soma_nossa[k] += nossas[*v][k] - rest[*v][k];
        }
    }
    let nosso = unit(soma_nossa);
    assert!(
        norm(soma_nossa) > 1e-5,
        "o nosso Push nao moveu nada no unico passo simulado desta cena"
    );
    let nosso_ao_normal = graus(unit([-nosso[0], -nosso[1], -nosso[2]]), n_superficie);
    assert!(
        nosso_ao_normal < 0.1,
        "a NOSSA direccao esta' a {nosso_ao_normal:.3}° da normal da superficie, \
         e a do alvo a {ao_normal:.3}°"
    );
    // ⭐ A razão dos módulos é o `2R` (espec §4.2-bis (7)): os dois traços têm o
    // mesmo campo de queda e só o `|u|` os separa.
    let pico = |campo: &[(usize, V3)]| campo.iter().map(|(_, u)| norm(*u)).fold(0.0, f64::max);
    let razao = pico(&empurrar) / pico(&inflar);
    let dois_r = 2.0 * t.pincel().raio;
    assert!(
        (razao - dois_r).abs() < 2e-5,
        "a razao Push/Inflate e' {razao:.6} e 2R e' {dois_r:.6}"
    );
}

/// ⭐⭐⭐ **GATE — O NOSSO RELEVO LOCAL NÃO PASSA O DO ORÁCULO** (a régua da AGULHA
/// do `ph2d-sculpt3d`, corrida com o LADO APROVADO ao lado).
///
/// A régua é `max_v |u_v − média(u dos 4 vizinhos de grelha)| / max(u) / (h/R)²`
/// — o pior vértice contra a própria vizinhança, em unidades do chão da
/// discretização.
///
/// ⚠️⚠️ **Existe porque a barra daquele gate (`20`) foi calibrada sobre a lei
/// VBD**, que o dono reprovou três vezes, e o caminho de omissão passou a ser a
/// da referência. *Uma barra calibrada sem o lado aprovado mede os nossos
/// próprios defeitos* — e aqui há lado aprovado.
///
/// ⭐⭐ **E o que ele diz é que `20` é uma barra que o PRÓPRIO ALVO não passa:**
/// sobre os 56 traços de plano do corpus, a saída do oráculo lê `3,8` a **`62,7`**
/// nesta régua, e passa `20` em catorze deles (o aperto de linha lê `45`–`63`, o
/// Expand `31`–`33`, o Snake Hook `25`–`38`). ⇒ é a **terceira** das barras de
/// artefacto desta linha a ser reprovada pela saída do alvo.
///
/// O que se pode afirmar com o lado aprovado é isto: o nosso relevo local não
/// passa o dele. E passa — traço a traço, quase sempre ao décimo.
#[test]
fn o_nosso_relevo_local_nao_passa_o_do_oraculo() {
    let rest = repouso("plano");
    // A grelha das fixtures: `(n+1)²` vértices em `[-1,1]²`. O `(i,j)` de cada
    // vértice deriva-se da POSIÇÃO, nunca da ordem do ficheiro.
    let n = (rest.len() as f64).sqrt().round() as usize - 1;
    let h = 2.0 / n as f64;
    let idx = |i: usize, j: usize| -> usize {
        let (x, y) = (-1.0 + i as f64 * h, -1.0 + j as f64 * h);
        (0..rest.len())
            .min_by(|a, b| dist(rest[*a], [x, y, 0.0]).total_cmp(&dist(rest[*b], [x, y, 0.0])))
            .expect("malha vazia")
    };
    let grelha: Vec<usize> = (0..=n)
        .flat_map(|j| (0..=n).map(move |i| (i, j)))
        .map(|(i, j)| idx(i, j))
        .collect();
    let regua = |depois: &[V3], raio: f64| -> f64 {
        let u = |v: usize| dist(rest[v], depois[v]);
        let max = (0..rest.len()).map(u).fold(0.0f64, f64::max);
        let piso = (h / raio).powi(2);
        let at = |i: usize, j: usize| grelha[j * (n + 1) + i];
        let mut pior = 0.0f64;
        for j in 1..n {
            for i in 1..n {
                let d = u(at(i, j));
                let m =
                    (u(at(i - 1, j)) + u(at(i + 1, j)) + u(at(i, j - 1)) + u(at(i, j + 1))) / 4.0;
                pior = pior.max((d - m).abs() / max.max(1e-12) / piso);
            }
        }
        pior
    };
    /// A folga sobre o lado aprovado. ⚠️ Ela é `1,4` e não `1,05` por causa de
    /// UM traço: `plano_apertar_ponto_radial_local` lê `60,2` contra `46,5`
    /// (`1,29×`) — e ele é um dos oito ABERTOS, no regime em que o próprio alvo
    /// deixa de ser determinista (§5.2-ter). Nos outros 55 a razão é `1,00`.
    const FOLGA: f64 = 1.4;
    let (mut medidos, mut pior) = (0usize, 0.0f64);
    for nome in todas() {
        if !nome.starts_with("plano_") {
            continue;
        }
        let raio = traco(&nome).pincel().raio;
        let (nosso, alvo) = (
            regua(&correr_posicoes(&nome), raio),
            regua(&deformado(&nome), raio),
        );
        // Anti-vácuo: um traço que não deformou dá `0` dos dois lados.
        assert!(alvo > 1.0, "{nome}: o oraculo mal tem relevo ({alvo:.2})");
        medidos += 1;
        pior = pior.max(nosso / alvo);
        assert!(
            nosso <= alvo * FOLGA,
            "{nome}: o nosso relevo local e' {nosso:.1} contra {alvo:.1} do \
             oraculo ({:.2}x, folga {FOLGA}) -- a agulha e' NOSSA",
            nosso / alvo
        );
    }
    assert!(
        medidos >= 50,
        "so' {medidos} tracos de plano medidos (anti-vacuo)"
    );
    // ⛔ A outra metade: a folga tem de ser JUSTA. Se o pior descer muito abaixo
    // dela, ela deixou de descrever o corpus e esconde uma degradação futura.
    assert!(
        pior > FOLGA / 2.0,
        "o pior e' {pior:.2}x contra a folga {FOLGA} -- a folga ja' nao descreve \
         o corpus e tem de descer"
    );
}
