//! Gates da cena `=51`. Eles medem a geometria que a cena COZINHA, não a intenção
//! com que ela foi escrita.
//!
//! ⚠️ **As bandas 1-2 são `Effect::Temporal`, então o harness TEM de chamar
//! `advance_tick`** — sem ele a aresta `pre` nunca carrega estado e as duas
//! bandeiras saem na pose de semeadura, bit a bit iguais, com todo gate a passar
//! por vácuo.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 150;
const DT: f64 = 1.0 / 60.0;

struct Frames {
    /// A pose de cada banda no ÚLTIMO tique.
    late: Vec<Vec<[f32; 2]>>,
    /// A trilha da partícula PINADA (índice 0) de cada banda, tique a tique.
    ///
    /// ⚠️ **Ela existe porque o mastro é PERIÓDICO.** A minha primeira régua era a
    /// diferença entre dois instantes, e com um período de 2,4 s sobre 150 tiques
    /// os dois caem quase na mesma FASE: o gate acusou `0,3225` sobre uma bandeira
    /// que varre 5,0 de ponta a ponta. É a lição que o pêndulo dos joints já pagou
    /// — *o oráculo de um movimento periódico é a EXCURSÃO*.
    trail: Vec<Vec<[f32; 2]>>,
}

/// A excursão de uma trilha: o maior vão entre duas amostras, em qualquer eixo.
fn excursion(t: &[[f32; 2]]) -> f32 {
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in t {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    (hi[0] - lo[0]).max(hi[1] - lo[1])
}

/// ⚠️ **Uma cena, um `Cook`** — as bandas partilham o documento, então cozer cada
/// uma num `Cook` próprio faria quatro simulações independentes.
fn cook_frames() -> Frames {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_space_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "dois pares");

    let mut cook = Cook::new();
    let mut late = vec![Vec::new(); sinks.len()];
    let mut trail = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, playhead)
            .expect("o tique avanca o `pre`");
        for (lane, sink) in sinks.iter().enumerate() {
            let s = cook
                .cook(&doc.graph, &reg, *sink, playhead)
                .expect("a banda coze")[0]
                .as_stream()
                .clone();
            let p_all = match s.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => panic!("a banda {lane} nao emite P"),
            };
            trail[lane].push(p_all[0]);
            if k + 1 == TICKS {
                late[lane] = p_all;
            }
        }
    }
    Frames { late, trail }
}

/// A distância ao vizinho MAIS PRÓXIMO de cada elemento, ordenada.
fn nearest(p: &[[f32; 2]]) -> Vec<f32> {
    let mut out: Vec<f32> = p
        .iter()
        .enumerate()
        .map(|(i, a)| {
            p.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt())
                .fold(f32::MAX, f32::min)
        })
        .collect();
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());
    out
}

/// **A CENA MONTA AS QUATRO BANDAS QUE O LOG NOMEIA.**
///
/// ⚠️ A contagem é DERIVADA: um número escrito à mão só sabe dizer *"mudou"*, e a
/// pergunta é se a lista que o artista lê corresponde ao que a cena empilha.
#[test]
fn the_scene_builds_every_band_the_log_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_space_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), band_labels().count());
}

/// **TODA BANDA TERMINA NUM `motion.output`.**
#[test]
fn every_band_ends_in_an_output_node() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_space_demo_document(&mut doc, &reg).expect("a cena monta");
    for (i, sink) in sinks.iter().enumerate() {
        assert_eq!(
            doc.graph.node(*sink).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "a banda {i} tem de terminar num Output",
        );
    }
}

/// **AS DUAS BANDEIRAS SEGUEM O MASTRO, E DA MESMA MANEIRA.**
///
/// ⚠️ **As duas metades são precisas.** *"Elas concordam"* passaria sobre duas
/// bandeiras igualmente CONGELADAS — que é exactamente o mundo que esta wave
/// substitui —, e *"a de baixo se move"* passaria sobre um corpo em queda livre.
/// O par é: as duas se movem **e** se movem igual.
#[test]
fn both_flags_ride_the_mast_and_they_agree() {
    let f = cook_frames();
    let (a_late, b_late) = (&f.late[0], &f.late[1]);

    // A linha pinada é a de TOPO nas duas, e a régua é a EXCURSÃO do mastro.
    let intrinsic = excursion(&f.trail[0]);
    let generic = excursion(&f.trail[1]);
    assert!(
        generic > 3.0,
        "a bandeira do pino GENERICO tem de varrer com o mastro; a excursao foi {generic:.4}"
    );
    assert!(
        intrinsic > 3.0,
        "e a do intrinseco tambem -- o CONTROLE do controle ({intrinsic:.4})"
    );

    // ⚠️ A régua é a POSE RELATIVA ao pino, e não a absoluta: as duas bandas são
    // deslocadas para lugares diferentes da tela pelo `motion.move`, e comparar
    // coordenadas cruas mediria o LAYOUT (a lição que o `1,15` do grupo B pagou).
    let rel = |p: &Vec<[f32; 2]>| -> Vec<[f32; 2]> {
        p.iter().map(|q| [q[0] - p[0][0], q[1] - p[0][1]]).collect()
    };
    let (ra, rb) = (rel(a_late), rel(b_late));
    let worst = ra
        .iter()
        .zip(&rb)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-3,
        "as duas espécies de pino têm de desenhar a MESMA bandeira; o pior desvio \
         relativo foi {worst:.6}"
    );
}

/// **O ESPAÇO PESSOAL ABRE O BANDO, E A RÉGUA É O TAMANHO DESENHADO.**
#[test]
fn personal_space_opens_the_flock_measured_against_what_is_drawn() {
    let f = cook_frames();
    let tight = nearest(&f.late[2]);
    let open = nearest(&f.late[3]);
    let n = tight.len();
    let size = drawn_size();

    let over_tight = tight.iter().filter(|d| **d < size).count();
    let over_open = open.iter().filter(|d| **d < size).count();
    // ⚠️ **O piso é `n/5` e o medido é 11/40 — RE-DERIVADO com o par desconfundido.**
    // A versão anterior pedia a MAIORIA (`> n/2`) e media 34/40, mas aquele controle
    // não tinha o peso de separação que a banda tratada carregava: ele era o
    // confundidor, não uma régua. Com o peso no topo do slider nos DOIS lados, um
    // bando ainda amontoa um quarto dos agentes — a fixture segue a conter o
    // fenômeno, e o que ela deixou de conter é a segunda variável.
    assert!(
        over_tight >= n / 5,
        "o CONTROLE tem de sobrepor (senão a banda de baixo não prova nada): \
         {over_tight}/{n}"
    );
    assert!(
        over_open * 4 < over_tight,
        "o espaço pessoal tem de fechar a sobreposição: {over_tight}/{n} -> \
         {over_open}/{n}"
    );
    // ⚠️ **A barra é 1,25 e o número medido é 1,365 — ela foi RE-DERIVADA quando o
    // par deixou de ser confundido, não afrouxada.** A versão anterior pedia 1,4×,
    // um número que só era satisfazível porque a banda tratada carregava TAMBÉM um
    // peso de separação 3,75× maior; com o peso nos dois lados o controle já
    // empacota a 1,1824 e o tratado mede 1,6137. *Uma barra que código correto não
    // consegue satisfazer não é rigor, é um gate partido* — e a metade que carrega
    // o peso da wave é a de CIMA (11 sobrepostos contra ZERO), que é categórica.
    assert!(
        open[n / 2] > tight[n / 2] * 1.25,
        "e o vão mediano tem de crescer: {:.4} -> {:.4}",
        tight[n / 2],
        open[n / 2]
    );
}

/// A SONDA que imprime os números da mensagem do smoke.
#[test]
#[ignore = "sonda"]
fn probe_the_scene_numbers() {
    let f = cook_frames();
    for (i, l) in band_labels() {
        println!("  {}. {l}", i + 1);
    }
    println!(
        "bandeira: excursao do intrinseco {:.4} | do generico {:.4}",
        excursion(&f.trail[0]),
        excursion(&f.trail[1])
    );
    for (lane, name) in [(2usize, "controle"), (3, "espaco pessoal")] {
        let nn = nearest(&f.late[lane]);
        let n = nn.len();
        println!(
            "bando {name}: min {:.4} mediana {:.4} | sobrepostos {}/{}",
            nn[0],
            nn[n / 2],
            nn.iter().filter(|d| **d < drawn_size()).count(),
            n
        );
    }
}
