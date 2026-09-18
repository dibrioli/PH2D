//! ⭐⭐⭐ **A BANCADA DO PENTE — a nossa malha contra a DELE, vértice a vértice.**
//!
//! # ⛔⛔⛔ Porque ela existe, e porque não existia
//!
//! O pente inteiro foi validado contra uma **BARRA DERIVADA** (*«o `Q` da faixa
//! tem de passar `+0,0465`»*), e nunca contra as malhas do oráculo — que estão
//! no repo, com **vértices E FACES**, desde o primeiro dia. ⇒ *as `221` corridas
//! do alvo viraram UM número.* O pincel de tecido fez `86` traços ⇒ `86` gates;
//! este fez `221` ficheiros ⇒ `1` barra.
//!
//! O preço apareceu num report do dono (*«não sei o que é para esperar. não vejo
//! diferença»*): desenhado, o A/B do nosso motor é indistinguível — e o do alvo
//! também, numa passagem. ⚠️ **Mas em oito passagens o dele mostra uma escada e o
//! nosso não**, e sem esta bancada não há como dizer se é a lei ou o arranjo.
//!
//! # ⭐ A chave: 64 das 221 células PRESERVAM a topologia
//!
//! Com o modo de detalhe **manual** — ou com o passe autorizado só a colapsar e
//! sem aresta curta — o passe de refino não parte nada, e a espec §3.1 diz que
//! ali *«o pente não muda UMA aresta: ele MOVE VÉRTICES»*. Nessas células
//! `v_entrada == v_saida`, a ordem dos vértices é a da entrada, e a comparação é
//! **exacta**. ⛔ Nas outras `143` a conectividade diverge por construção (dois
//! remalhadores diferentes) e uma comparação vértice a vértice não afirma nada.
//!
//! # ⚠️ O CONTROLO vem antes da medida
//!
//! Cada célula tem um par `p000` (pente desligado) e `p100` (no tecto). O `p000`
//! exercita **tudo menos o pente** — a nossa malha de entrada, o nosso percurso,
//! o nosso passo de traço, a nossa lei de `Draw`. *Sem ele, um desvio no `p100`
//! não se sabe de quem é.*

use std::collections::BTreeMap;
use std::path::PathBuf;

use ph2d_mesh::{Face, Mesh};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

use crate::oraculo_gz::inflar;

fn pasta() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures/rake")
}

/// Uma célula do corpus: o cabeçalho, o percurso do cursor e a malha de saída.
pub struct Celula {
    pub nome: String,
    cab: BTreeMap<String, String>,
    pub percurso: Vec<[f32; 3]>,
    pub saida: Vec<[f32; 3]>,
    pub faces: Vec<Face>,
}

impl Celula {
    pub fn chave(&self, k: &str) -> &str {
        self.cab
            .get(k)
            .unwrap_or_else(|| panic!("{}: falta a chave `{k}`", self.nome))
    }
    pub fn num(&self, k: &str) -> f32 {
        self.chave(k)
            .parse()
            .unwrap_or_else(|_| panic!("{}: `{k}` nao e' numero", self.nome))
    }
    pub fn sim(&self, k: &str) -> bool {
        self.chave(k) == "True"
    }
}

fn v3(c: &[&str]) -> [f32; 3] {
    [
        c[0].parse().expect("x"),
        c[1].parse().expect("y"),
        c[2].parse().expect("z"),
    ]
}

/// Lê `V n` + posições e `F n` + faces de um texto do corpus.
fn malha_do_texto(texto: &str) -> (Vec<[f32; 3]>, Vec<Face>) {
    let (mut pos, mut faces) = (Vec::new(), Vec::new());
    let mut modo = ' ';
    for l in texto.lines() {
        if l.starts_with('#') {
            continue;
        }
        let t: Vec<&str> = l.split_whitespace().collect();
        if t.is_empty() {
            continue;
        }
        match t[0] {
            "V" => modo = 'v',
            "F" => modo = 'f',
            _ if modo == 'v' => pos.push(v3(&t)),
            _ if modo == 'f' => {
                let idx: Vec<u32> = t.iter().map(|x| x.parse().expect("indice")).collect();
                faces.push(match idx.len() {
                    3 => Face::tri(idx[0], idx[1], idx[2]),
                    4 => Face::quad(idx[0], idx[1], idx[2], idx[3]),
                    n => panic!("face de {n} cantos"),
                });
            }
            _ => {}
        }
    }
    (pos, faces)
}

pub fn ler(familia: &str, nome: &str) -> Celula {
    let texto = inflar(&pasta().join(format!("{familia}/{nome}.txt.gz")));
    let mut cab = BTreeMap::new();
    let mut percurso = Vec::new();
    for l in texto.lines() {
        let Some(resto) = l.strip_prefix("# ") else {
            continue;
        };
        if let Some(p) = resto.strip_prefix("ponto ") {
            percurso.push(v3(&p.split_whitespace().collect::<Vec<_>>()));
        } else if let Some((k, v)) = resto.split_once('=') {
            cab.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    let (saida, faces) = malha_do_texto(&texto);
    Celula {
        nome: format!("{familia}/{nome}"),
        cab,
        percurso,
        saida,
        faces,
    }
}

/// A malha de ENTRADA que o cabeçalho da célula nomeia.
pub fn entrada(c: &Celula) -> Mesh {
    let ficheiro = c.chave("malha_de_entrada");
    let texto = inflar(&pasta().join("entrada").join(ficheiro));
    let (pos, faces) = malha_do_texto(&texto);
    Mesh::from_parts(pos, faces).expect("a malha de entrada e' valida")
}

/// O pincel que o cabeçalho descreve.
pub fn pincel(c: &Celula) -> Brush {
    let verb = match c.chave("verbo") {
        "DRAW" => Verb::Draw,
        "SMOOTH" => Verb::Smooth,
        "CLAY" => Verb::Clay,
        "CREASE" => Verb::Crease,
        "FLATTEN" => Verb::Flatten,
        "INFLATE" => Verb::Inflate,
        "PINCH" => Verb::Pinch,
        "SCRAPE" => Verb::Scrape,
        "LAYER" => Verb::Layer,
        "FILL" => Verb::Fill,
        "BLOB" => Verb::Blob,
        "NUDGE" => Verb::Nudge,
        "THUMB" => Verb::Thumb,
        "GRAB" => Verb::Move,
        "SNAKE_HOOK" => Verb::SnakeHook,
        "ROTATE" => Verb::Twist,
        "MASK" => Verb::Mask,
        "DRAW_SHARP" => Verb::DrawSharp,
        outro => panic!("{}: verbo `{outro}` sem traducao", c.nome),
    };
    let falloff = match c.chave("curva_de_queda") {
        "SMOOTH" => Falloff::Smooth,
        "SHARP" => Falloff::Sharp,
        "ROOT" => Falloff::Root,
        "SPHERE" => Falloff::Sphere,
        "POW4" => Falloff::Sharper,
        "CONSTANT" => Falloff::Constant,
        "LIN" | "LINEAR" => Falloff::Linear,
        outra => panic!("{}: curva `{outra}` sem traducao", c.nome),
    };
    Brush {
        verb,
        radius: c.num("raio_em_unidades_de_objecto"),
        strength: c.num("forca"),
        hardness: c.num("dureza"),
        falloff,
        pente: c.num("pente"),
        accumulate: c.sim("acumular"),
        // ⚠️⚠️ **O MODO de referência é o do CORPUS.** Sem esta linha o `Draw`
        // corre com a lei do `s-mode` e o controlo desvia `8,33×` — os dois
        // lados movem os MESMOS `56` vértices e por quantidades diferentes, que
        // é a assinatura de *«a pegada está certa, a lei não»*.
        mode: ph2d_sculpt3d::RefMode::B,
        // ⛔⛔ **E o traço NÃO é «arrastado» no sentido desta casa.** Esse campo
        // liga a ATENUAÇÃO por espaçamento, que é lei do pincel de PLANO e do
        // AFIADO (espec §14.4 daquele) e que este corpus não exercita: com ela o
        // nosso dab lê `0,05155` contra `0,08592` do alvo, e sem ela lê
        // **`0,08592`** — o mesmo número. ⚠️ O `espacamento_pct` FICA, porque
        // quem o lê aqui é o PASSO do traço (onde cada carimbo cai), não a
        // atenuação.
        traco_arrastado: false,
        espacamento_pct: Some(c.num("espacamento_pct")),
        ..Brush::default()
    }
}

/// **CORRE a célula pelo caminho do PRODUTO** e devolve as posições.
///
/// ⚠️ **Os dabs são RE-AMOSTRADOS**, porque o cabeçalho publica só os
/// `pontos_pedidos` do cursor e declara `metodo_do_traco=SPACE`: quem decide
/// onde cada carimbo cai é o passo do traço, não o ponto pedido.
pub fn correr(c: &Celula) -> Vec<[f32; 3]> {
    correr_com(c, std::env::var("PENTE_REAMOSTRA").is_ok())
}

/// `reamostra`: se os carimbos são re-amostrados pelo passo do traço, ou se os
/// pontos do cabeçalho **são** os carimbos.
pub fn correr_com(c: &Celula, reamostra: bool) -> Vec<[f32; 3]> {
    let mut m = entrada(c);
    let b = pincel(c);
    let passagens = c.num("PASSAGENS").round().max(1.0) as usize;
    let passo = ph2d_sculpt3d::passo_do_traco(&b, b.radius);
    // ⚠️⚠️ **AS NORMAIS SÃO PREGADAS ENTRE DABS**, como na bancada do pincel
    // afiado: sem isso a normal da área inclina-se sobre o relevo que o próprio
    // traço levanta.
    let normais0 = m.normals().to_vec();
    for _ in 0..passagens {
        let mut s = SculptStroke::default();
        s.begin(&m);
        let mut anterior: Option<[f32; 2]> = None;
        for p in &c.percurso {
            let alvo = [p[0], p[1]];
            let mut carimbos: Vec<[f32; 2]> = Vec::new();
            match (anterior, reamostra) {
                (Some(de), true) => {
                    if let Some(w) = ph2d_sculpt3d::walk(de, alvo, passo) {
                        carimbos.extend(w);
                    }
                }
                _ => carimbos.push(alvo),
            }
            for q in carimbos {
                s.dab(
                    &mut m,
                    &b,
                    &Dab::at([q[0], q[1], p[2]], b.radius, [0.0, 0.0, -1.0]),
                    Symmetry::default(),
                );
                m.pregar_normais_para_teste(&normais0);
                anterior = Some(q);
            }
        }
    }
    m.positions().to_vec()
}

/// **QUANTOS vértices esta saída moveu** contra a malha de ENTRADA.
///
/// ⛔⛔ **É o controlo que separa parity de VÁCUO:** a máscara não move um
/// vértice, logo a nossa saída e a dele coincidem **ao bit** com qualquer lei —
/// *um zero de «igual» e um de «nada aconteceu» são o mesmo byte*.
pub fn movidos(c: &Celula, pos: &[[f32; 3]]) -> usize {
    let repouso = entrada(c);
    repouso
        .positions()
        .iter()
        .zip(pos)
        .filter(|(a, b)| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs() > 1e-7)
        .count()
}

/// A maior distância entre duas nuvens do MESMO tamanho.
pub fn maior_distancia(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    assert_eq!(a.len(), b.len(), "as nuvens tem tamanhos diferentes");
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// **AS CÉLULAS DO CORPUS QUE PRESERVAM A TOPOLOGIA** — derivadas da árvore, e
/// nunca escritas à mão.
///
/// ⛔ **Derivadas porque uma lista à mão apodrece:** o corpus cresce a cada
/// emenda do E, e uma célula nova que preservasse a topologia ficaria fora da
/// bancada **em silêncio**. O piso de população abaixo é o que o torna visível.
pub fn celulas_sem_remalha() -> Vec<(String, String)> {
    let mut fora = Vec::new();
    let mut dirs: Vec<_> = std::fs::read_dir(pasta())
        .expect("a pasta do corpus")
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    dirs.sort();
    for fam in dirs {
        if fam == "entrada" {
            continue;
        }
        let mut nomes: Vec<String> = std::fs::read_dir(pasta().join(&fam))
            .expect("familia")
            .filter_map(Result::ok)
            .filter_map(|e| {
                e.file_name()
                    .to_string_lossy()
                    .strip_suffix(".txt.gz")
                    .map(std::borrow::ToOwned::to_owned)
            })
            .collect();
        nomes.sort();
        for nome in nomes {
            let c = ler(&fam, &nome);
            if c.chave("v_entrada") == c.chave("v_saida") {
                fora.push((fam.clone(), nome));
            }
        }
    }
    fora
}

/// **SONDA — o placar da bancada**, célula a célula.
#[test]
#[ignore = "sonda"]
fn diag_o_placar_do_pente() {
    let mut piores: Vec<(f32, String)> = Vec::new();
    for (fam, nome) in celulas_sem_remalha() {
        let c = ler(&fam, &nome);
        let nosso = correr(&c);
        let d = maior_distancia(&nosso, &c.saida);
        let (nm, om) = (movidos(&c, &nosso), movidos(&c, &c.saida));
        piores.push((d, format!("{fam}/{nome}")));
        eprintln!(
            "{fam}/{nome:<18} pente {:.2} {:<12} v={:<5} movidos {nm:>4}/{om:<4} desvio {d:.3e}",
            c.num("pente"),
            c.chave("verbo"),
            c.saida.len()
        );
    }
    piores.sort_by(|a, b| b.0.total_cmp(&a.0));
    eprintln!("--- {} celulas · piores ---", piores.len());
    for (d, n) in piores.iter().take(8) {
        eprintln!("  {d:.3e}  {n}");
    }
}

/// **SONDA — o que cada lado FEZ**, na célula de um dab só.
#[test]
#[ignore = "sonda"]
fn diag_um_dab_lado_a_lado() {
    let nome = std::env::var("CELULA").unwrap_or_else(|_| "y_umdab_p000".into());
    let c = ler("mecanismo", &nome);
    let m = entrada(&c);
    let repouso = m.positions().to_vec();
    let nosso = correr(&c);
    let resumo = |rotulo: &str, pos: &[[f32; 3]]| {
        let mut movidos = 0usize;
        let (mut maxz, mut maxxy) = (0.0f32, 0.0f32);
        for (a, b) in repouso.iter().zip(pos) {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            if d[0].abs() + d[1].abs() + d[2].abs() > 1e-7 {
                movidos += 1;
            }
            maxz = maxz.max(d[2].abs());
            maxxy = maxxy.max(d[0].hypot(d[1]));
        }
        eprintln!("{rotulo:<10} movidos {movidos:>4}  max|dz| {maxz:.5}  max|dxy| {maxxy:.5}");
    };
    resumo("NOSSO", &nosso);
    // As duas cercas que podem estar a atenuar: o traço arrastado e o
    // espaçamento declarado.
    for (rotulo, arrastado, esp) in [
        ("sem arrasto", false, true),
        ("sem esp", true, false),
        ("sem os dois", false, false),
    ] {
        let mut m = entrada(&c);
        let mut b = pincel(&c);
        b.traco_arrastado = arrastado;
        if !esp {
            b.espacamento_pct = None;
        }
        let mut st = SculptStroke::default();
        st.begin(&m);
        st.dab(
            &mut m,
            &b,
            &Dab::at(c.percurso[0], b.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        resumo(rotulo, m.positions());
    }
    resumo("ORACULO", &c.saida);
    eprintln!(
        "forca {:.3} raio {:.3} queda {}",
        c.num("forca"),
        c.num("raio_em_unidades_de_objecto"),
        c.chave("curva_de_queda")
    );
}

/// ⭐⭐⭐ **A SEGUNDA COLUNA: quanto o PENTE move na saída DELE.**
///
/// O desvio sozinho não tem escala. `2,8e-2` é um defeito de `5 %` ou de
/// `500 %` conforme o que o knob de facto faz no alvo — e é essa razão que
/// responde à pergunta que o dono comprou: *a nossa lei é a dele?*
///
/// Para cada par `(p000, p100)` da MESMA célula:
///   - `dele`  = max |saída_p100 − saída_p000| do ORÁCULO (o efeito do pente nele)
///   - `nosso` = o mesmo, calculado pela nossa lei
///   - `erro`  = max |nossa saída_p100 − saída_p100 dele|
///
/// ⚠️ `erro / dele` é a régua honesta. `erro ≈ dele` quer dizer que erramos o
/// pente por inteiro; `erro ≪ dele` quer dizer que o temos quase certo.
#[test]
#[ignore = "sonda: corre a bancada inteira"]
fn diag_o_pente_contra_o_efeito_dele() {
    let mut linhas = Vec::new();
    for (fam, nome) in celulas_sem_remalha() {
        let Some(base) = nome.strip_suffix("_p100") else {
            continue;
        };
        let off = format!("{base}_p000");
        let (a, b) = (ler(&fam, &off), ler(&fam, &nome));
        if a.saida.len() != b.saida.len() {
            continue;
        }
        let dele = maior_distancia(&a.saida, &b.saida);
        let (na, nb) = (correr(&a), correr(&b));
        let nosso = maior_distancia(&na, &nb);
        let erro = maior_distancia(&nb, &b.saida);
        let erro_off = maior_distancia(&na, &a.saida);
        linhas.push((
            dele,
            format!(
                "{fam}/{base:<14} dele {dele:.3e}  nosso {nosso:.3e}  \
             erro_on {erro:.3e}  erro_off {erro_off:.3e}  \
             razao {:.2}",
                if dele > 0.0 { erro / dele } else { f32::NAN }
            ),
        ));
    }
    linhas.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap());
    for (_, l) in &linhas {
        eprintln!("{l}");
    }
    eprintln!("\n{} pares", linhas.len());
}

/// ⭐⭐⭐ **A LIGAÇÃO: as faces da saída dele são as mesmas da ENTRADA?**
///
/// Esta bancada compara POSIÇÕES, e uma grade é propriedade da
/// CONECTIVIDADE — se o pente do alvo virasse arestas, um placar de posições
/// ficaria cego exactamente onde a lei vive. A `Celula::faces` era parseada e
/// deitada fora, e foi o `dead_code` do compilador que o disse.
#[test]
#[ignore = "sonda: corre a bancada inteira"]
fn diag_a_ligacao_muda() {
    let mut mudam = 0;
    let mut total = 0;
    for (fam, nome) in celulas_sem_remalha() {
        let c = ler(&fam, &nome);
        let dentro = entrada(&c);
        let a: Vec<_> = dentro.faces().to_vec();
        total += 1;
        let igual = a.len() == c.faces.len() && a.iter().zip(&c.faces).all(|(x, y)| x == y);
        if !igual {
            mudam += 1;
            eprintln!(
                "{fam}/{nome}: faces entrada {} · saída {} · iguais? NÃO",
                a.len(),
                c.faces.len()
            );
        }
    }
    eprintln!("\n{mudam} de {total} células mudam a LIGAÇÃO");
}
