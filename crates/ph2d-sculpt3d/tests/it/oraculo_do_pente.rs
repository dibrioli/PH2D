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
        // ⭐⭐ Pela PORTA DO PRODUTO, nunca o campo cru: com a topologia
        // dinâmica desarmada o pente é inerte, e o arnês que escrevesse o
        // campo à mão mediria um caminho que o produto não consegue percorrer
        // (`porta/c_nodyn` lia `4,507e-2` contra um produto exacto).
        pente: ph2d_sculpt3d::pente_do_traco(
            c.chave("topologia_dinamica") == "armada",
            c.num("pente"),
        ),
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
/// A direcção da vista que este corpus usa — a mesma em todas as células.
const OLHO: [f32; 3] = [0.0, 0.0, -1.0];

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
        // A âncora é fotografada UMA vez, no pen-down, e só para quem segura.
        let ancora = (b.verb.grip() == ph2d_sculpt3d::Grip::Hold)
            .then(|| ph2d_sculpt3d::ancora::ancora_do_gesto(&m, c.percurso[0], false));
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
                let dab = match b.verb.grip() {
                    // ⭐⭐ **O GESTO ANCORADO é conduzido pela regra que a
                    // bancada dos gestos tangenciais já PROVOU:** o centro é a
                    // âncora do pen-down e o puxão é o deslocamento **TOTAL**
                    // desde o primeiro ponto, nunca o incremento.
                    //
                    // ⚠️ A âncora passa pela porta do produto por DISCIPLINA, e
                    // isso **não está provado por mutação neste corpus**:
                    // `ancora_do_gesto(_, p, false)` devolve `p` na primeira
                    // linha, logo trocá-la pelo ponto é um NO-OP enquanto
                    // nenhuma fixtura armar a âncora em vértice. *Uma mutação
                    // que não muta lê-se como sobrevivência e não é.*
                    //
                    // ⛔⛔ Sem isto o arnês entrega um CARIMBO a um verbo que
                    // segura, e ele move **zero** vértices onde o oráculo move
                    // `56` — o que se lê na tabela como um desvio catastrófico
                    // da LEI (`6,874e-1`) quando é do ARNÊS.
                    ph2d_sculpt3d::Grip::Hold => Dab::pulling(
                        ancora.expect("um gesto que segura tem âncora"),
                        b.radius,
                        OLHO,
                        [
                            q[0] - c.percurso[0][0],
                            q[1] - c.percurso[0][1],
                            p[2] - c.percurso[0][2],
                        ],
                    ),
                    _ => Dab::at([q[0], q[1], p[2]], b.radius, OLHO),
                };
                s.dab(&mut m, &b, &dab, Symmetry::default());
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

/// ⭐⭐⭐ **A ASSINATURA DA FORMA, dos dois lados** — as cinco grandezas que a
/// espec §3.2 mede sobre o alvo, corridas também sobre nós.
///
/// O desvio máximo diz QUANTO erramos; estas dizem **EM QUÊ**. A decisiva é a
/// última: `‖Σ Δ‖ / Σ‖Δ‖` mede se o material VIAJA. No alvo ela dá `3,84 %`
/// (a soma cancela a `96,2 %`) ⇒ *ele não arrasta, ele RELAXA* — cada vértice
/// desliza sobre a superfície até as arestas à volta ficarem alinhadas.
///
/// ⚠️ **O Δ é do PENTE e não do traço**: `p100 − p000` de cada lado, senão o
/// empurrão do verbo (que é ao longo da normal) domina tudo e a soma deixa de
/// cancelar por construção.
#[test]
#[ignore = "sonda: a assinatura da forma"]
fn diag_a_assinatura_da_forma() {
    println!(
        "{:<26} {:>9} {:>9} {:>9} {:>7} {:>8} {:>8}",
        "celula/lado", "|dx|", "|dy|", "|dz|", "tan/nor", "viaja%", "max/ar"
    );
    for (fam, base) in [
        ("mecanismo", "x_man_x01"),
        ("mecanismo", "x_man_x02"),
        ("mecanismo", "x_man_x04"),
        ("mecanismo", "x_man_x08"),
        ("mecanismo", "x_man_x16"),
        ("composicao", "m_manual"),
        ("mecanismo", "y_arco"),
    ] {
        let off = ler(fam, &format!("{base}_p000"));
        let on = ler(fam, &format!("{base}_p100"));
        let aresta = aresta_media(&entrada(&off));
        for (rotulo, a, b) in [
            ("ALVO", off.saida.clone(), on.saida.clone()),
            ("nosso", correr(&off), correr(&on)),
        ] {
            let d: Vec<[f32; 3]> = a
                .iter()
                .zip(&b)
                .map(|(p, q)| [q[0] - p[0], q[1] - p[1], q[2] - p[2]])
                .filter(|v| v[0].abs() + v[1].abs() + v[2].abs() > 1e-9)
                .collect();
            if d.is_empty() {
                println!("{fam}/{base:<12} {rotulo:<6} (inerte)");
                continue;
            }
            let n = d.len() as f64;
            let (mut sx, mut sy, mut sz) = (0.0f64, 0.0f64, 0.0f64);
            let (mut vx, mut vy, mut vz) = (0.0f64, 0.0f64, 0.0f64);
            let (mut soma_norma, mut maior) = (0.0f64, 0.0f64);
            for v in &d {
                sx += f64::from(v[0].abs());
                sy += f64::from(v[1].abs());
                sz += f64::from(v[2].abs());
                vx += f64::from(v[0]);
                vy += f64::from(v[1]);
                vz += f64::from(v[2]);
                let nv = f64::from(v[0])
                    .hypot(f64::from(v[1]))
                    .hypot(f64::from(v[2]));
                soma_norma += nv;
                maior = maior.max(nv);
            }
            let tan = (sx / n).hypot(sy / n);
            let viaja = vx.hypot(vy).hypot(vz) / soma_norma * 100.0;
            let rumo = if vx.hypot(vy).hypot(vz) > 1e-9 {
                let l = vx.hypot(vy).hypot(vz);
                format!("({:+.2},{:+.2},{:+.2})", vx / l, vy / l, vz / l)
            } else {
                "-".to_string()
            };
            println!(
                "{fam}/{base:<12} {rotulo:<6} {:9.6} {:9.6} {:9.6} {:7.2} {:7.2}% {:8.3} {rumo}",
                sx / n,
                sy / n,
                sz / n,
                tan / (sz / n).max(1e-12),
                viaja,
                maior / f64::from(aresta)
            );
        }
    }
}

/// O comprimento médio de aresta de uma malha — a unidade em que a espec
/// exprime o maior deslocamento do pente (`0,34` no alvo).
fn aresta_media(m: &Mesh) -> f32 {
    let p = m.positions();
    let (mut soma, mut n) = (0.0f64, 0usize);
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (p[v[k] as usize], p[v[(k + 1) % v.len()] as usize]);
            soma += f64::from((a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]));
            n += 1;
        }
    }
    (soma / n as f64) as f32
}

/// ⭐⭐⭐ **O PERCURSO contra o PASSO do traço.**
///
/// A escada `x_man_x01..x16` mostra que as passagens **não** são o defeito: as
/// cinco colunas da assinatura acompanham o alvo. O que destoa é `m_manual` e
/// `y_arco`, que não são mais passagens — são outro PERCURSO. Se os pontos do
/// cabeçalho estiverem muito mais juntos do que o passo do pincel, o arnês
/// carimba (e penteia) muito mais vezes do que o alvo, e o pente **acumula**.
#[test]
#[ignore = "sonda: o percurso contra o passo"]
fn diag_o_percurso_contra_o_passo() {
    println!(
        "{:<26} {:>6} {:>10} {:>10} {:>8} {:>7}",
        "celula", "pontos", "passo", "vao_medio", "vao/passo", "passag"
    );
    for (fam, nome) in celulas_sem_remalha() {
        if !nome.ends_with("_p100") {
            continue;
        }
        let c = ler(&fam, &nome);
        let b = pincel(&c);
        let passo = ph2d_sculpt3d::passo_do_traco(&b, b.radius);
        let mut soma = 0.0f64;
        for w in c.percurso.windows(2) {
            soma += f64::from((w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]));
        }
        let n = (c.percurso.len().max(2) - 1) as f64;
        let vao = soma / n;
        println!(
            "{fam}/{:<16} {:>6} {:>10.5} {:>10.5} {:>8.2} {:>7.0}",
            nome.trim_end_matches("_p100"),
            c.percurso.len(),
            passo,
            vao,
            vao / f64::from(passo),
            c.num("PASSAGENS").max(1.0)
        );
    }
}

/// ⛔⛔⛔ **A FORÇA contra o desvio do lado DESLIGADO.**
///
/// O «controlo» que ilibava a lei base (`1,624e-4` no `x_man_x01_p000`) foi
/// medido a força `0,2`. As células que desviam `2,98e-2` com o pente
/// DESLIGADO diferem daquela em **um** campo do cabeçalho — a força, `0,5` —,
/// com a mesma malha, o mesmo percurso e as mesmas passagens.
///
/// ⇒ *se o desvio do lado desligado seguir a força, o defeito não é do pente:
/// é da lei base, e o pente só o amplifica.*
#[test]
#[ignore = "sonda: a forca contra o desvio"]
fn diag_a_forca_contra_o_desvio() {
    let mut linhas: Vec<(f32, String)> = Vec::new();
    for (fam, nome) in celulas_sem_remalha() {
        if !nome.ends_with("_p000") {
            continue;
        }
        let c = ler(&fam, &nome);
        let d = maior_distancia(&correr(&c), &c.saida);
        let f = c.num("forca");
        linhas.push((
            f,
            format!(
                "forca {f:.2}  {fam}/{:<16} verbo {:<12} desvio {d:.3e}",
                nome.trim_end_matches("_p000"),
                c.chave("verbo")
            ),
        ));
    }
    linhas.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (_, l) in &linhas {
        println!("{l}");
    }
}

/// ⭐⭐⭐ **O campo do pente, VÉRTICE A VÉRTICE.**
///
/// As cinco grandezas da assinatura são AGREGADAS: no `x_man_x01` (força
/// `0,2`, uma passagem) elas **batem** e o desvio é na mesma `2,8e-2`. ⇒ *ou a
/// lei é a mesma e a ARRUMAÇÃO difere, ou é outra lei com as mesmas
/// estatísticas* — e o que separa as duas é o cosseno por vértice.
///
/// Imprime, sobre os vértices que qualquer um dos lados move:
/// - `cos` ponderado pela norma (⚠️ ponderado, senão os vértices que mal se
///   mexem, que são ruído, pesam tanto como os que carregam o efeito);
/// - a razão das normas (nós / ele);
/// - quantos vértices cada lado move, e quantos são COMUNS.
#[test]
#[ignore = "sonda: o campo vertice a vertice"]
fn diag_o_campo_do_pente_vertice_a_vertice() {
    println!(
        "{:<24} {:>7} {:>8} {:>7} {:>7} {:>7} {:>7}",
        "celula", "cos", "|nos|/|ele|", "movN", "movE", "comuns", "so'nos"
    );
    for (fam, base) in [
        ("mecanismo", "x_man_x01"),
        ("mecanismo", "x_man_x04"),
        ("composicao", "m_manual"),
        ("mecanismo", "y_arco"),
    ] {
        let off = ler(fam, &format!("{base}_p000"));
        let on = ler(fam, &format!("{base}_p100"));
        let delta = |a: &[[f32; 3]], b: &[[f32; 3]]| -> Vec<[f64; 3]> {
            a.iter()
                .zip(b)
                .map(|(p, q)| {
                    [
                        f64::from(q[0] - p[0]),
                        f64::from(q[1] - p[1]),
                        f64::from(q[2] - p[2]),
                    ]
                })
                .collect()
        };
        let dele = delta(&off.saida, &on.saida);
        let nosso = delta(&correr(&off), &correr(&on));
        let norma = |v: &[f64; 3]| v[0].hypot(v[1]).hypot(v[2]);
        let vivo = 1e-9;

        let (mut num, mut den) = (0.0f64, 0.0f64);
        let (mut rn, mut re) = (0.0f64, 0.0f64);
        let (mut mn, mut me, mut comuns) = (0usize, 0usize, 0usize);
        for (a, b) in dele.iter().zip(&nosso) {
            let (na, nb) = (norma(a), norma(b));
            if na > vivo {
                me += 1;
            }
            if nb > vivo {
                mn += 1;
            }
            if na > vivo && nb > vivo {
                comuns += 1;
                let c = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (na * nb);
                num += c * na;
                den += na;
            }
            re += na;
            rn += nb;
        }
        println!(
            "{fam}/{base:<12} {:>7.3} {:>11.3} {mn:>7} {me:>7} {comuns:>7} {:>7}",
            if den > 0.0 { num / den } else { f64::NAN },
            if re > 0.0 { rn / re } else { f64::NAN },
            mn.saturating_sub(comuns)
        );
    }
}

/// ⭐⭐⭐ **Os dois campos DECOMPOSTOS no quadro do traço.**
///
/// O cosseno ponderado diz que erramos a direcção; isto diz **em que eixo**.
/// Para cada vértice projecta-se o Δ do pente em `ao_longo` (a direcção do
/// traço) e em `atraves`, e mede-se a correlação de cada componente
/// separadamente. *Uma componente bem correlacionada e outra anti-correlacionada
/// é um erro de sinal; as duas fracas é outra lei.*
#[test]
#[ignore = "sonda: os campos decompostos"]
fn diag_os_campos_decompostos() {
    println!(
        "{:<24} {:>8} {:>8} {:>9} {:>9}",
        "celula", "corr_ao", "corr_at", "amp_ao", "amp_at"
    );
    for (fam, base) in [
        ("mecanismo", "x_man_x01"),
        ("composicao", "m_manual"),
        ("mecanismo", "y_arco"),
    ] {
        let off = ler(fam, &format!("{base}_p000"));
        let on = ler(fam, &format!("{base}_p100"));
        // O quadro: a direcção do traço é a do último par de pontos.
        let p = &off.percurso;
        let (dx, dy) = (
            p[p.len() - 1][0] - p[p.len() - 2][0],
            p[p.len() - 1][1] - p[p.len() - 2][1],
        );
        let l = dx.hypot(dy).max(1e-12);
        let (ax, ay) = (f64::from(dx / l), f64::from(dy / l));
        let d = |a: &[[f32; 3]], b: &[[f32; 3]]| -> Vec<(f64, f64)> {
            a.iter()
                .zip(b)
                .map(|(u, v)| {
                    let (ex, ey) = (f64::from(v[0] - u[0]), f64::from(v[1] - u[1]));
                    (ex * ax + ey * ay, -ex * ay + ey * ax)
                })
                .collect()
        };
        let dele = d(&off.saida, &on.saida);
        let nosso = d(&correr(&off), &correr(&on));
        let corr = |f: &dyn Fn(&(f64, f64)) -> f64| {
            let (mut sxy, mut sxx, mut syy, mut sa, mut sb) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for (x, y) in dele.iter().map(f).zip(nosso.iter().map(f)) {
                sxy += x * y;
                sxx += x * x;
                syy += y * y;
                sa += x.abs();
                sb += y.abs();
            }
            (
                sxy / (sxx.sqrt() * syy.sqrt()).max(1e-30),
                sb / sa.max(1e-30),
            )
        };
        let (cao, aao) = corr(&|t: &(f64, f64)| t.0);
        let (cat, aat) = corr(&|t: &(f64, f64)| t.1);
        println!("{fam}/{base:<12} {cao:>8.3} {cat:>8.3} {aao:>9.3} {aat:>9.3}");
    }
}

/// ⭐⭐⭐ **A ESCADA DA ACUMULAÇÃO, com o pente INERTE.**
///
/// O §73.3 mediu que a força por si não é o defeito — o que diverge é a
/// acumulação ao longo de um traço de carimbos. Estas três células isolam-na
/// sem percurso, sem direcção e sem pente:
///
/// - `y_umdab` — **um** carimbo · `y_doisdab` — **dois** · `y_parado` —
///   **catorze no mesmo sítio** (o pente é inerte nas três: §4.3).
///
/// Compara `saída − entrada` (o que o VERBO fez) vértice a vértice.
#[test]
#[ignore = "sonda: a escada da acumulacao"]
fn diag_a_escada_da_acumulacao() {
    println!(
        "{:<20} {:>7} {:>6} {:>7} {:>8} {:>9} {:>9}",
        "celula", "dabs", "cos", "nos/ele", "movN/movE", "desvio", "|maior|"
    );
    for base in ["y_umdab", "y_doisdab", "y_parado", "y_ida"] {
        let c = ler("mecanismo", &format!("{base}_p000"));
        let dentro = entrada(&c);
        let nosso = correr(&c);
        let d = |a: &[[f32; 3]]| -> Vec<[f64; 3]> {
            dentro
                .positions()
                .iter()
                .zip(a)
                .map(|(p, q)| {
                    [
                        f64::from(q[0] - p[0]),
                        f64::from(q[1] - p[1]),
                        f64::from(q[2] - p[2]),
                    ]
                })
                .collect()
        };
        let (dele, nos) = (d(&c.saida), d(&nosso));
        let norma = |v: &[f64; 3]| v[0].hypot(v[1]).hypot(v[2]);
        let (mut num, mut den, mut rn, mut re) = (0.0, 0.0, 0.0, 0.0);
        let (mut mn, mut me, mut maior) = (0usize, 0usize, 0.0f64);
        for (a, b) in dele.iter().zip(&nos) {
            let (na, nb) = (norma(a), norma(b));
            if na > 1e-9 {
                me += 1;
            }
            if nb > 1e-9 {
                mn += 1;
            }
            if na > 1e-9 && nb > 1e-9 {
                num += (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (na * nb) * na;
                den += na;
            }
            re += na;
            rn += nb;
            maior = maior.max(na);
        }
        println!(
            "{base:<20} {:>7} {:>6.3} {:>7.3} {:>8} {:>9.3e} {:>9.5}",
            c.percurso.len(),
            if den > 0.0 { num / den } else { f64::NAN },
            if re > 0.0 { rn / re } else { f64::NAN },
            format!("{mn}/{me}"),
            maior_distancia(&nosso, &c.saida),
            maior
        );
    }
}

/// ⭐⭐⭐⭐ **A ESCADA DO INTERRUPTOR — a metade da tabela-verdade que faltava.**
///
/// Colhida em 2026-09-18 pela janela **E** (família `acumula/`), ela arbitra a
/// contradição do §75: o mesmo carimbo repetido `N` vezes no mesmo sítio, nos
/// **dois** estados do `acumular`.
///
/// ⚠️ A âncora que a torna de confiar é uma IDENTIDADE: `escada_n14_off` é
/// byte-idêntica (no corpo) a `mecanismo/y_parado_p000`, cujo cabeçalho diz
/// `acumular=False`. *O interruptor foi identificado por identidade, nunca por
/// leitura do rótulo* — e o sentido dele é contra-intuitivo.
#[test]
#[ignore = "sonda: a escada do interruptor"]
fn diag_a_escada_do_interruptor() {
    println!(
        "{:>4} {:>14} {:>14} {:>8} {:>14} {:>14} {:>8}",
        "N", "ALVO off", "nosso off", "razao", "ALVO on", "nosso on", "razao"
    );
    // ⚠️ **As contagens DERIVAM do corpus, nunca de uma lista escrita à mão.**
    // Uma célula nova que chegue à pasta aparece aqui sozinha; com a lista à
    // mão ela ficava invisível e a sonda lia-se como completa. *É a mesma lei
    // que este repo cobra dos censos por prefixo de nome.*
    let mut ns: Vec<usize> = celulas_sem_remalha()
        .into_iter()
        .filter(|(f, n)| f == "acumula" && n.starts_with("escada_n") && n.ends_with("_on"))
        .filter_map(|(_, n)| n["escada_n".len()..n.len() - 3].parse().ok())
        .collect();
    ns.sort_unstable();
    assert!(
        ns.len() >= 7,
        "a escada do interruptor encolheu para {} contagens — alguém apagou corpus",
        ns.len()
    );
    for n in ns {
        let mut col = Vec::new();
        for lado in ["off", "on"] {
            let c = ler("acumula", &format!("escada_n{n}_{lado}"));
            let dentro = entrada(&c);
            let maior = |p: &[[f32; 3]]| -> f64 {
                dentro
                    .positions()
                    .iter()
                    .zip(p)
                    .map(|(a, b)| {
                        f64::from(b[0] - a[0])
                            .hypot(f64::from(b[1] - a[1]))
                            .hypot(f64::from(b[2] - a[2]))
                    })
                    .fold(0.0, f64::max)
            };
            col.push((maior(&c.saida), maior(&correr(&c))));
        }
        println!(
            "{n:>4} {:>14.8} {:>14.8} {:>8.3} {:>14.8} {:>14.8} {:>8.3}",
            col[0].0,
            col[0].1,
            col[0].1 / col[0].0,
            col[1].0,
            col[1].1,
            col[1].1 / col[1].0
        );
    }
}
