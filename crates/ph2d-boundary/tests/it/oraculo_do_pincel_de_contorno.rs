//! **A bancada de paridade do pincel de CONTORNO** — a nossa lei contra os `61`
//! traços do oráculo, sobre malhas **nossas**.
//!
//! ⚠️⚠️ **A barra é DERIVADA, não escolhida** (espec §19.3): o alvo guarda
//! posições em `f32`, as nossas peças têm `|posição| ≤ 2`, o ULP ali é `2,4e-7`
//! e uma cadeia de ~`20` operações acumula `~5e-6` no pior caso ⇒ **`1e-5` em
//! posição absoluta**.
//!
//! ⛔ **Bit-parity não é a meta e não é prometida** (ADR-0162).
//!
//! ⚠️ **E a barra tem de incluir o lado APROVADO:** o corpus mede
//! `dispersao_entre_realizacoes = 0,00000000` em todas as fixtures com duas
//! corridas ⇒ o alvo é determinístico ao bit, logo qualquer barra positiva já é
//! folgada em relação ao ruído dele. *Uma barra que reprove o próprio alvo é um
//! defeito da barra — esta linha já retirou duas assim, na obra do tecido.*

use std::collections::BTreeMap;
use std::path::PathBuf;

use ph2d_boundary::vetor::{V3, distancia};
use ph2d_boundary::{Contorno, Controlos, Evento, Modo, QuedaNoContorno, Topologia};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/boundary")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore — a gémea da bancada da pose: o
/// cabeçalho do gzip tem tamanho variável e o rabo são oito bytes.
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
    for bit in [0x08u8, 0x10] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
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

/// A malha de repouso. ⚠️ **Uma face tem TRÊS ou QUATRO índices** — quem assumir
/// um dos dois lê outra malha, e o erro é mudo.
struct Superficie {
    pos: Vec<V3>,
    faces: Vec<Vec<u32>>,
}

fn superficie(nome: &str) -> Superficie {
    let texto = inflar(&format!("{nome}.repouso.txt.gz"));
    let (mut pos, mut faces) = (Vec::new(), Vec::new());
    for l in texto.lines() {
        let campos: Vec<&str> = l.split_whitespace().collect();
        match campos.first().copied() {
            Some("v") => pos.push(v3(&campos[1..])),
            Some("f") => faces.push(
                campos[1..]
                    .iter()
                    .map(|c| c.parse::<u32>().expect("indice"))
                    .collect(),
            ),
            _ => {}
        }
    }
    Superficie { pos, faces }
}

/// ⚠️ **As normais são DERIVADAS aqui, e a ponderação é uma escolha nomeada:**
/// a soma das normais de face **pesadas pela área** (o produto externo não
/// normalizado é exactamente isso), normalizada no fim. ⛔ A espec não fixa a
/// ponderação e as grelhas planas não a discriminam — quem a quiser medir tem
/// as peças **curvas** (cúpula e tubo) do corpus, e é lá que uma escolha errada
/// aparece.
fn normais(sup: &Superficie) -> Vec<V3> {
    let mut n = vec![[0.0f32; 3]; sup.pos.len()];
    for face in &sup.faces {
        for k in 1..face.len().saturating_sub(1) {
            let (a, b, c) = (
                sup.pos[face[0] as usize],
                sup.pos[face[k] as usize],
                sup.pos[face[k + 1] as usize],
            );
            let cr = ph2d_boundary::vetor::cruz(
                ph2d_boundary::vetor::sub(b, a),
                ph2d_boundary::vetor::sub(c, a),
            );
            for &v in face.iter() {
                n[v as usize] = ph2d_boundary::vetor::add(n[v as usize], cr);
            }
        }
    }
    for v in &mut n {
        *v = ph2d_boundary::vetor::normalizar(*v).unwrap_or([0.0, 0.0, 1.0]);
    }
    // ⚠️⚠️ **A ORIENTAÇÃO é uma propriedade da FIXTURA, não da lei — e a cúpula
    // vem enrolada ao contrário.** Medido: o enrolamento das faces dá
    // `n · radial = −1,0` na cúpula e `+` no tubo, sendo as três peças do mesmo
    // gerador. Com as normais para dentro, o `inflar` empurra para dentro e o
    // eixo do `dobrar` inverte-se — as duas fixturas da cúpula reprovam por
    // **sinal**, e o corpo delas bate ao `1e-7`.
    //
    // ⇒ a bancada orienta a malha inteira para FORA do centroide dela quando a
    // maioria das faces aponta para dentro. ⛔ **Isto não é uma lei do pincel**:
    // a lei lê *«a normal de repouso do vértice»*, e quem as produz é a malha.
    // *Uma bancada que alimente a lei com outra convenção de normais mede outro
    // programa* — e no produto as normais vêm da `ph2d-mesh`, com a convenção
    // dela. Item aberto para a regeneração das fixtures (acto de E).
    let mut centroide = [0.0f32; 3];
    for p in &sup.pos {
        centroide = ph2d_boundary::vetor::add(centroide, *p);
    }
    centroide = ph2d_boundary::vetor::escalar(centroide, 1.0 / sup.pos.len() as f32);
    let para_fora = sup
        .pos
        .iter()
        .zip(&n)
        .filter(|(p, _)| ph2d_boundary::vetor::distancia(**p, centroide) > 1e-6)
        .map(|(p, nv)| ph2d_boundary::vetor::ponto(*nv, ph2d_boundary::vetor::sub(*p, centroide)))
        .filter(|d| d.abs() > 1e-6)
        .map(|d| if d > 0.0 { 1i32 } else { -1 })
        .sum::<i32>();
    if para_fora < 0 {
        for v in &mut n {
            *v = ph2d_boundary::vetor::escalar(*v, -1.0);
        }
    }
    n
}

/// Um traço do oráculo: cabeçalho, percurso do cursor e posições finais.
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
            "caminho" | "vertices" => {}
            chave => {
                chaves.insert(chave.to_string(), campos[1..].join(" "));
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
        self.chaves
            .get(k)
            .unwrap_or_else(|| panic!("cabecalho sem `{k}`"))
    }
    fn f(&self, k: &str) -> f32 {
        self.s(k)
            .parse()
            .unwrap_or_else(|_| panic!("`{k}` nao e' numero"))
    }
    fn u(&self, k: &str) -> usize {
        self.s(k)
            .parse()
            .unwrap_or_else(|_| panic!("`{k}` nao e' inteiro"))
    }
}

fn constante(_p: f32) -> f32 {
    1.0
}
fn linear(p: f32) -> f32 {
    p
}
fn aguda(p: f32) -> f32 {
    p * p
}

fn curva_de(t: &Traco) -> fn(f32) -> f32 {
    match t.s("curva") {
        "constant" => constante,
        "linear" => linear,
        "sharp" => aguda,
        "smooth" => ph2d_boundary::suave,
        outra => panic!("curva desconhecida: {outra}"),
    }
}

fn controlos(t: &Traco) -> Controlos {
    let raio = t.f("raio");
    Controlos {
        modo: match t.s("modo") {
            "dobrar" => Modo::Dobrar,
            "expandir" => Modo::Expandir,
            "inflar" => Modo::Inflar,
            "agarrar" => Modo::Agarrar,
            "torcer" => Modo::Torcer,
            "suavizar" => Modo::Suavizar,
            outro => panic!("modo desconhecido: {outro}"),
        },
        queda_no_contorno: match t.s("queda_no_contorno") {
            "constante" => QuedaNoContorno::Constante,
            "raio" => QuedaNoContorno::Raio,
            "laco" => QuedaNoContorno::Laco,
            "laco_invertido" => QuedaNoContorno::LacoInvertido,
            outra => panic!("queda desconhecida: {outra}"),
        },
        deslocamento_da_origem: t.f("deslocamento_da_origem"),
        raio_inicial: raio,
        // ⚠️ **Sem pressão a modular o tamanho, os dois raios coincidem** — e o
        // corpus confirma-o: a fixtura de pressão `0,3` é idêntica à de `1`.
        raio_dinamico: raio,
        forca: t.f("forca"),
        esbatimento_da_simetria: 1.0,
        encaixar_angulo: t.s("inverter") == "1",
        simetria: [t.s("simetria_x") == "1", false, false],
    }
}

struct Corrida {
    pior: f32,
    movidos_nossos: usize,
    movidos_oraculo: usize,
    max_nosso: f32,
    max_oraculo: f32,
    recusa: Option<String>,
}

fn correr(nome: &str) -> Corrida {
    let t = traco(nome);
    let sup = superficie(t.s("superficie"));
    let nrm = normais(&sup);
    let ctrl = controlos(&t);
    assert_eq!(
        sup.pos.len(),
        t.depois.len(),
        "{nome}: contagem de vertices"
    );

    let escondido = vec![false; sup.pos.len()];
    let topo = Topologia::construir(
        sup.pos.len(),
        sup.faces.iter().map(Vec::as_slice),
        &escondido,
    );
    let curva = curva_de(&t);
    let curva: ph2d_boundary::Curva<'_> = &curva;

    // §8.2 — a máscara do corpus tem duas formas, e o cabeçalho diz qual.
    let mascara: Option<Vec<f32>> = match t.s("mascara") {
        "nenhuma" => None,
        "metade_valor" => Some(vec![0.5; sup.pos.len()]),
        // ⚠️ **`>= 0` e não `> 0`**, medido: com `>` movemos `85` vértices e o
        // alvo move `80` — os `5` a mais são exactamente a coluna do meio
        // (`x = 0`), que tem de entrar na máscara.
        "x_positivo" => Some(
            sup.pos
                .iter()
                .map(|p| if p[0] >= 0.0 { 1.0 } else { 0.0 })
                .collect(),
        ),
        outra => panic!("mascara desconhecida: {outra}"),
    };
    let fatores = ph2d_boundary::Fatores {
        mascara: mascara.as_deref(),
        ..Default::default()
    };

    let contacto = v3(&t.s("contacto").split_whitespace().collect::<Vec<_>>());
    let arrasto = v3(&t.s("arrasto").split_whitespace().collect::<Vec<_>>());
    let passos = t.u("passos");

    let mut posicoes = sup.pos.clone();
    let mut recusa = None;
    // §12 — cada eixo espelhado duplica as passagens, e cada passagem refaz as
    // fases A–E do zero para a sua região.
    let passagens: Vec<(V3, V3)> = if ctrl.simetria[0] {
        vec![
            (contacto, arrasto),
            (
                [-contacto[0], contacto[1], contacto[2]],
                [-arrasto[0], arrasto[1], arrasto[2]],
            ),
        ]
    } else {
        vec![(contacto, arrasto)]
    };
    for (c, a) in passagens {
        let Some(sob) = ph2d_boundary::ancora::mais_proximo(&sup.pos, &escondido, c) else {
            continue;
        };
        let contorno = match Contorno::comecar(
            &topo, &sup.pos, &nrm, &escondido, sob, c, &ctrl, curva, fatores,
        ) {
            Ok(k) => k,
            Err(e) => {
                recusa = Some(format!("{e:?}"));
                continue;
            }
        };
        // ⚠️ **O resultado é função do arrasto TOTAL** nos cinco modos
        // conduzidos por `s` — mas o alisar acumula com o número de eventos, e
        // por isso a bancada corre **todos** os passos, sempre.
        for k in 1..=passos {
            let t = k as f32 / passos as f32;
            let ev = Evento {
                arrasto: [a[0] * t, a[1] * t, a[2] * t],
            };
            contorno.passo(&topo, &ctrl, &ev, &sup.pos, &nrm, &mut posicoes);
        }
    }

    let mut pior = 0.0f32;
    let mut movidos_nossos = 0usize;
    let mut movidos_oraculo = 0usize;
    let (mut max_nosso, mut max_oraculo) = (0.0f32, 0.0f32);
    for ((antes, nosso), deles) in sup.pos.iter().zip(&posicoes).zip(&t.depois) {
        pior = pior.max(distancia(*nosso, *deles));
        let dn = distancia(*antes, *nosso);
        let doo = distancia(*antes, *deles);
        if dn > 0.0 {
            movidos_nossos += 1;
        }
        if doo > 0.0 {
            movidos_oraculo += 1;
        }
        max_nosso = max_nosso.max(dn);
        max_oraculo = max_oraculo.max(doo);
    }
    Corrida {
        pior,
        movidos_nossos,
        movidos_oraculo,
        max_nosso,
        max_oraculo,
        recusa,
    }
}

fn corpus() -> Vec<String> {
    let mut nomes: Vec<String> = std::fs::read_dir(fixture_dir())
        .expect("pasta de fixtures")
        .filter_map(|e| {
            let f = e.ok()?.file_name().to_string_lossy().into_owned();
            f.strip_suffix(".deformado.txt.gz").map(str::to_string)
        })
        .collect();
    nomes.sort();
    nomes
}

/// A corrida do corpus inteiro, **a imprimir a tabela**. Este é o instrumento;
/// os gates com barra vêm depois dele, quando a tabela disser onde pô-los.
#[test]
fn mede_o_corpus_contra_o_oraculo() {
    let nomes = corpus();
    assert!(
        nomes.len() >= 61,
        "o corpus encolheu: {} fixtures (esperadas 61) — uma varredura partida \
         devolve zero e le-se como aprovada",
        nomes.len()
    );
    println!(
        "\n{:<52} {:>10} {:>7} {:>7} {:>10} {:>10}",
        "fixture", "pior", "nossos", "deles", "max nosso", "max deles"
    );
    let (mut verdes, mut piores) = (0usize, Vec::new());
    for nome in &nomes {
        let c = correr(nome);
        if c.pior <= 1e-5 {
            verdes += 1;
        }
        println!(
            "{nome:<52} {:>10.3e} {:>7} {:>7} {:>10.6} {:>10.6}{}",
            c.pior,
            c.movidos_nossos,
            c.movidos_oraculo,
            c.max_nosso,
            c.max_oraculo,
            c.recusa.map_or(String::new(), |r| format!("  [{r}]"))
        );
        piores.push((c.pior, nome.clone()));
    }
    piores.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("\n=== {verdes} de {} dentro de 1e-5 ===", nomes.len());
    println!("piores:");
    for (p, n) in piores.iter().take(12) {
        println!("  {p:>10.3e}  {n}");
    }
}

/// Sonda de diagnóstico: o que a fase D produz em cada peça.
#[test]
#[ignore = "sonda: imprime a estrutura de cada fixture"]
fn sonda_a_estrutura() {
    for nome in [
        "grade_expandir_constante",
        "tubo_expandir_constante",
        "cupula_expandir_constante",
        "grade_triangulada_dobrar_constante",
        "grade_pequena_dobrar_origem0",
        "grade_dobrar_constante_simetriax",
    ] {
        let t = traco(nome);
        let sup = superficie(t.s("superficie"));
        let nrm = normais(&sup);
        let ctrl = controlos(&t);
        let escondido = vec![false; sup.pos.len()];
        let topo = Topologia::construir(
            sup.pos.len(),
            sup.faces.iter().map(Vec::as_slice),
            &escondido,
        );
        let curva = curva_de(&t);
        let curva: ph2d_boundary::Curva<'_> = &curva;
        let contacto = v3(&t.s("contacto").split_whitespace().collect::<Vec<_>>());
        let arrasto = v3(&t.s("arrasto").split_whitespace().collect::<Vec<_>>());
        let sob = ph2d_boundary::ancora::mais_proximo(&sup.pos, &escondido, contacto).unwrap();
        let k = Contorno::comecar(
            &topo,
            &sup.pos,
            &nrm,
            &escondido,
            sob,
            contacto,
            &ctrl,
            curva,
            ph2d_boundary::Fatores::default(),
        )
        .expect("sem recusa");
        let e = k.estrutura();
        let (a, o) = k.linha_da_profundidade(&sup.pos);
        let dir = ph2d_boundary::vetor::normalizar(ph2d_boundary::vetor::sub(o, contacto))
            .unwrap_or([0.0; 3]);
        println!(
            "{nome:<38} v={:<6} borda={:<4} cadeia={:<4} K={:<2} contacto={contacto:?}\n  \
             ancora={a:?} origem={o:?}\n  n={dir:?} s={:.6} arrasto={arrasto:?}",
            sup.pos.len(),
            topo.conta_vertices_de_borda(),
            e.cadeia.len(),
            e.alcance,
            ph2d_boundary::leis::avanco(arrasto, dir),
        );
    }
}

/// Sonda: o que o ORÁCULO fez, vértice a vértice, nas fixtures que divergem.
#[test]
#[ignore = "sonda: le' o campo de deslocamento do oraculo"]
fn sonda_o_campo_do_oraculo() {
    for nome in [
        "tubo_expandir_constante",
        "cupula_expandir_constante",
        "grade_triangulada_dobrar_constante",
        "grade_pequena_dobrar_origem0",
    ] {
        let t = traco(nome);
        let sup = superficie(t.s("superficie"));
        let contacto = v3(&t.s("contacto").split_whitespace().collect::<Vec<_>>());
        let mut mexidos: Vec<(f32, usize)> = sup
            .pos
            .iter()
            .zip(&t.depois)
            .enumerate()
            .map(|(i, (a, b))| (distancia(*a, *b), i))
            .filter(|(d, _)| *d > 0.0)
            .collect();
        mexidos.sort_by(|a, b| b.0.total_cmp(&a.0));
        println!(
            "\n=== {nome}  contacto={contacto:?}  movidos={}",
            mexidos.len()
        );
        for &(d, i) in mexidos.iter().take(4) {
            let p = sup.pos[i];
            let q = t.depois[i];
            let u = ph2d_boundary::vetor::sub(q, p);
            println!(
                "  v{i:<5} |d|={d:.6}  p0={:?}\n            u={:?}  u/|d|={:?}",
                p,
                u,
                ph2d_boundary::vetor::normalizar(u).unwrap_or([0.0; 3])
            );
        }
        // O vértice de borda mais perto do contacto — a coluna da âncora.
        let mut perto: Vec<(f32, usize)> = mexidos
            .iter()
            .map(|&(_, i)| (distancia(sup.pos[i], contacto), i))
            .collect();
        perto.sort_by(|a, b| a.0.total_cmp(&b.0));
        for &(_, i) in perto.iter().take(3) {
            let u = ph2d_boundary::vetor::sub(t.depois[i], sup.pos[i]);
            println!(
                "  perto v{i:<5} p0={:?} u={:?} |u|={:.6}",
                sup.pos[i],
                u,
                ph2d_boundary::vetor::comprimento(u)
            );
        }
    }
}

/// Sonda: **resolve** o `s` que o oráculo usou, e compara-o com as candidatas.
///
/// Nos modos `EXPAND` e `INFLATE` o deslocamento de um vértice do anel `0` é
/// `direcção × (F·s·w)` com `w = 1` ⇒ o `s` do alvo lê-se directamente da
/// fixtura. *É a única forma honesta de escolher entre duas leis que coincidem
/// na grelha plana.*
#[test]
#[ignore = "sonda: resolve o avanco do oraculo e compara com as candidatas"]
fn sonda_resolve_o_avanco() {
    println!(
        "\n{:<38} {:>10} | {:>10} {:>10} {:>10} {:>10}",
        "fixture", "s do alvo", "contacto", "-origem", "ancora", "centroide"
    );
    for nome in [
        "grade_expandir_constante",
        "grade_inflar_constante",
        "grade_expandir_constante_para_dentro",
        "tubo_expandir_constante",
        "tubo_inflar_constante",
        "cupula_expandir_constante",
        "cupula_inflar_constante",
    ] {
        let t = traco(nome);
        let sup = superficie(t.s("superficie"));
        let nrm = normais(&sup);
        let ctrl = controlos(&t);
        let escondido = vec![false; sup.pos.len()];
        let topo = Topologia::construir(
            sup.pos.len(),
            sup.faces.iter().map(Vec::as_slice),
            &escondido,
        );
        let curva = curva_de(&t);
        let curva: ph2d_boundary::Curva<'_> = &curva;
        let contacto = v3(&t.s("contacto").split_whitespace().collect::<Vec<_>>());
        let arrasto = v3(&t.s("arrasto").split_whitespace().collect::<Vec<_>>());
        let sob = ph2d_boundary::ancora::mais_proximo(&sup.pos, &escondido, contacto).unwrap();
        let k = Contorno::comecar(
            &topo,
            &sup.pos,
            &nrm,
            &escondido,
            sob,
            contacto,
            &ctrl,
            curva,
            ph2d_boundary::Fatores::default(),
        )
        .expect("sem recusa");
        let e = k.estrutura();
        let origem = e.ponto_origem;
        let ancora = sup.pos[e.ancora as usize];
        // O `s` do alvo, lido na ÂNCORA (anel 0, peso 1).
        let u = ph2d_boundary::vetor::sub(t.depois[e.ancora as usize], ancora);
        let dir = if ctrl.modo == Modo::Inflar {
            nrm[e.ancora as usize]
        } else {
            let fundo = e
                .fundo_da_coluna(e.ancora)
                .expect("a coluna da ancora chega ao fundo");
            ph2d_boundary::vetor::normalizar(ph2d_boundary::vetor::sub(
                ancora,
                sup.pos[fundo as usize],
            ))
            .unwrap_or([0.0; 3])
        };
        let s_alvo = ph2d_boundary::vetor::ponto(u, dir);
        let uni = |v: V3| ph2d_boundary::vetor::normalizar(v).unwrap_or([0.0; 3]);
        let mut centroide = [0.0f32; 3];
        for &c in &e.cadeia {
            centroide = ph2d_boundary::vetor::add(centroide, sup.pos[c as usize]);
        }
        centroide = ph2d_boundary::vetor::escalar(centroide, 1.0 / e.cadeia.len() as f32);
        let p = |n: V3| ph2d_boundary::vetor::ponto(arrasto, n);
        println!(
            "{nome:<38} {s_alvo:>10.6} | {:>10.6} {:>10.6} {:>10.6} {:>10.6}",
            p(uni(ph2d_boundary::vetor::sub(origem, contacto))),
            p(ph2d_boundary::vetor::escalar(uni(origem), -1.0)),
            p(uni(ph2d_boundary::vetor::sub(origem, ancora))),
            p(uni(ph2d_boundary::vetor::sub(centroide, origem))),
        );
    }
}

/// Sonda: a normal que eu derivo contra o deslocamento que o alvo produziu.
#[test]
#[ignore = "sonda: orientacao das normais por peca"]
fn sonda_as_normais() {
    for nome in [
        "grade_inflar_constante",
        "cupula_inflar_constante",
        "cupula_expandir_constante",
        "tubo_inflar_constante",
    ] {
        let t = traco(nome);
        let sup = superficie(t.s("superficie"));
        let nrm = normais(&sup);
        let contacto = v3(&t.s("contacto").split_whitespace().collect::<Vec<_>>());
        let escondido = vec![false; sup.pos.len()];
        let a =
            ph2d_boundary::ancora::mais_proximo(&sup.pos, &escondido, contacto).unwrap() as usize;
        let u = ph2d_boundary::vetor::sub(t.depois[a], sup.pos[a]);
        println!(
            "{nome:<34} p0={:?}\n   normal={:?}  u={:?}  u.n={:.6}",
            sup.pos[a],
            nrm[a],
            u,
            ph2d_boundary::vetor::ponto(u, nrm[a])
        );
    }
}

/// Em que classe cada fixtura cai — e **porquê**.
///
/// ⚠️⚠️ **A lista é EXPLÍCITA e medida, nunca um padrão de nome.** A irmã da
/// pose pagou essa lição: um predicado por prefixo arrastou `12` fixturas para
/// uma barra que só `8` precisavam, e foi o censo de obsolescência que o
/// apanhou. *Uma classe atribuída por nome descreve o nome, não a medição.*
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Classe {
    /// Dentro da barra derivada de `f32`.
    Normal,
    /// ⛔ **Fora do alcance desta espec**: o alvo de deformação é o solver de
    /// PANO, e ali a lei do modo **não escreve posições** — escreve as
    /// posições-alvo das restrições, e é o solver que produz o resultado. Aquele
    /// solver é outra espec desta linha.
    ForaDeEspec,
    /// ⭐ **Divergência DECLARADA** (§13.1): atamos o alcance ao anel mais fundo
    /// que existe; o alvo deixa os dados por preencher e o `EXPAND` fica **mudo**.
    DivergenciaDeclarada,
    /// ⛔ **A espec não fixa a lei**: o esbatimento da simetria (§12.3) diz
    /// *«a força é dividida pela sobreposição das passagens»* e não dá a
    /// fórmula. ⇒ não há o que reproduzir — há o que medir, numa emenda.
    SemLeiNaEspec,
    /// ⏳ **ABERTO**: divergência real, com o diagnóstico registado no handoff.
    Aberto,
}

fn classe(nome: &str) -> Classe {
    match nome {
        // O alvo é o solver de pano — a lei do modo não escreve posições.
        "grade_dobrar_constante_pano_local" | "grade_dobrar_constante_pano_dinamica" => {
            Classe::ForaDeEspec
        }
        // §13.1: a peça pequena com deslocamento `2`, onde o alcance excede a
        // geometria. ⭐ No `expandir` o alvo move **`0`** e nós movemos `20`.
        "grade_pequena_expandir_origem2" | "grade_pequena_dobrar_origem2" => {
            Classe::DivergenciaDeclarada
        }
        "grade_agarrar_raio_simetriax_perto_feather" => Classe::SemLeiNaEspec,
        // ⏳ Os cinco abertos, com o número de cada um no handoff.
        "grade_triangulada_dobrar_constante"
        | "grade_triangulada_expandir_constante"
        | "tubo_torcer_constante"
        | "tubo_torcer_raio"
        | "cupula_inflar_constante" => Classe::Aberto,
        _ => Classe::Normal,
    }
}

/// A barra, **derivada** do `f32` do alvo (§19.3).
const BARRA: f32 = 1e-5;

/// ⭐⭐ **A PARIDADE FECHA POR CLASSE.**
#[test]
fn a_paridade_fecha_por_classe() {
    let nomes = corpus();
    let mut falhas = Vec::new();
    let (mut normais_n, mut abertos_n) = (0usize, 0usize);
    for nome in &nomes {
        let c = correr(nome);
        match classe(nome) {
            Classe::Normal => {
                normais_n += 1;
                if c.pior > BARRA {
                    falhas.push(format!("{nome}: {:.3e} > {BARRA:e}", c.pior));
                }
            }
            Classe::Aberto => abertos_n += 1,
            _ => {}
        }
    }
    assert!(
        falhas.is_empty(),
        "{} fixturas NORMAIS fora da barra:\n  {}",
        falhas.len(),
        falhas.join("\n  ")
    );
    // ⭐ **Os pisos de população**, porque uma varredura partida devolve zero e
    // lê-se exactamente como aprovada.
    assert!(
        normais_n >= 51,
        "so' {normais_n} fixturas na classe normal — o corpus ou a tabela encolheu"
    );
    assert_eq!(abertos_n, 5, "a lista de abertos mudou de tamanho");
}

/// ⚠️⚠️ **O CENSO DE OBSOLESCÊNCIA da tabela de classes.**
///
/// *Uma catraca sem censo de obsolescência não desce: ela vira licença.* Este
/// gate pergunta as três coisas que envelhecem: **o alvo ainda existe? ainda
/// diverge? a classe ainda o descreve?**
#[test]
fn o_censo_das_excepcoes_nao_descreve_nada_de_obsoleto() {
    let nomes: std::collections::BTreeSet<String> = corpus().into_iter().collect();
    for nome in [
        "grade_dobrar_constante_pano_local",
        "grade_dobrar_constante_pano_dinamica",
        "grade_pequena_expandir_origem2",
        "grade_pequena_dobrar_origem2",
        "grade_agarrar_raio_simetriax_perto_feather",
        "grade_triangulada_dobrar_constante",
        "grade_triangulada_expandir_constante",
        "tubo_torcer_constante",
        "tubo_torcer_raio",
        "cupula_inflar_constante",
    ] {
        assert!(
            nomes.contains(nome),
            "a tabela de classes nomeia `{nome}`, que ja' nao existe no corpus"
        );
        // ⭐ E a metade que importa: se ela **deixou de divergir**, a excepção
        // virou licença e tem de sair.
        let c = correr(nome);
        assert!(
            c.pior > BARRA,
            "`{nome}` esta' na tabela de excepcoes e ja' fecha a {:.3e} — a \
             entrada e' obsoleta e tem de SAIR",
            c.pior
        );
    }
}

/// ⭐⭐ **O CABEÇALHO DE CADA FIXTURA CONCORDA COM O PERCURSO QUE ELA CARREGA.**
///
/// Cada traço do oráculo traz **duas** descrições do mesmo gesto: as chaves
/// `contacto` e `arrasto`, que é o que a bancada lê, e as linhas `c`, que são o
/// percurso do cursor ponto a ponto. Até 2026-09-14 o percurso era **parsado e
/// deitado fora** — o compilador dizia-o (`field caminho is never read`) e a
/// leitura era *«sobra do formato»*.
///
/// ⚠️ **Ele não é sobra: é o ÚNICO controlo independente do cabeçalho.** Toda a
/// paridade desta crate é medida contra números que a própria linha `#` declara;
/// se uma delas estivesse errada — um `arrasto` copiado do traço vizinho, um
/// `contacto` com o sinal trocado —, nós reproduziríamos fielmente o gesto
/// ERRADO e a tabela fecharia verde. *Um corpus com duas descrições do mesmo
/// facto e só uma lida tem metade da prova por gastar.*
#[test]
fn o_cabecalho_de_cada_fixtura_concorda_com_o_percurso_dela() {
    let nomes = corpus();
    assert!(nomes.len() >= 61, "o corpus encolheu: {}", nomes.len());
    let mut queixas = Vec::new();
    let mut parados = 0usize;
    for nome in &nomes {
        let t = traco(nome);
        assert!(!t.caminho.is_empty(), "{nome}: percurso vazio");
        let contacto = v3(&t.s("contacto").split_whitespace().collect::<Vec<_>>());
        let arrasto = v3(&t.s("arrasto").split_whitespace().collect::<Vec<_>>());
        let primeiro = t.caminho[0];
        let ultimo = t.caminho[t.caminho.len() - 1];
        let total = ph2d_boundary::vetor::sub(ultimo, primeiro);
        if t.caminho.len() == 1 {
            parados += 1;
        }
        let dc = ph2d_boundary::vetor::distancia(contacto, primeiro);
        let da = ph2d_boundary::vetor::distancia(arrasto, total);
        if dc > BARRA || da > BARRA {
            queixas.push(format!(
                "{nome}: contacto {contacto:?} vs c[0] {primeiro:?} (d={dc:.3e}) · \
                 arrasto {arrasto:?} vs c[n]-c[0] {total:?} (d={da:.3e})"
            ));
        }
    }
    assert!(
        queixas.is_empty(),
        "{} de {} fixturas descrevem dois gestos diferentes:\n{}",
        queixas.len(),
        nomes.len(),
        queixas.join("\n")
    );
    // ⭐ **O gesto PARADO é uma classe, não um defeito da fixtura, e a régua
    // acima trata-o sem um ramo próprio**: com um ponto só o percurso `c[n]−c[0]`
    // é o vector nulo, e o cabeçalho tem de trazer `arrasto = 0` para concordar —
    // que é exactamente o que o [`Modo::Suavizar`] é (ele lê a posição actual e
    // age com o cursor imóvel). ⚠️ O piso está aqui para a excepção não evaporar
    // em silêncio: se ela chegar a zero, ou o corpus perdeu o único traço parado
    // que tem, ou o formato mudou.
    assert_eq!(
        parados, 1,
        "o corpus tinha exactamente UM traço de cursor parado (o do alisar); \
         agora tem {parados}"
    );
}
