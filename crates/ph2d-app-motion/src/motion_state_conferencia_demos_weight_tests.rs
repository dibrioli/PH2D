//! Os gates da cena `=52` — o peso por partícula e os sub-passos.
//!
//! ⚠️ **Toda fixture que mede simulação precisa do `advance_tick`:** sem ele a
//! aresta `pre` nunca carrega estado, a cena fica no repouso e **todo gate de
//! realimentação é VERDE por vácuo** (as duas bandas de um par saem bit a bit
//! idênticas, e o número idêntico nas duas colunas é o que denuncia).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze a cena `ticks` tiques e devolve o `P` de cada banda.
fn run(ticks: usize) -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_weight_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..ticks {
        let t = k as f64 * DT;
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("o tique avança o `pre`");
        for (i, s) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, &reg, *s, t).expect("a banda coze");
            if let Some(Column::Vec2(p)) = v[0].as_stream().get("P") {
                out[i] = p.clone();
            }
        }
    }
    out
}

/// O maior vão entre dois nós vizinhos de uma corda — a régua do esticão.
fn worst_span(p: &[[f32; 2]]) -> f32 {
    p.windows(2)
        .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

/// A extensão vertical de um corpo — quão longe a cauda foi.
fn drop_depth(p: &[[f32; 2]]) -> f32 {
    let hi = p.iter().map(|q| q[1]).fold(f32::NEG_INFINITY, f32::max);
    let lo = p.iter().map(|q| q[1]).fold(f32::INFINITY, f32::min);
    hi - lo
}

/// A pose de um ponto **relativa à linha pinada da própria banda**.
///
/// ⚠️ **Comparar coordenadas CRUAS entre duas bandas mede o LAYOUT, não o
/// campo:** a cena separa as bandas com um `motion.move` de `BAND_DY`, e a
/// primeira versão desta sonda acusou a linha do meio de derivar **8,9901** —
/// que é `9,0`, o vão. É a quarta vez que este repo paga a lição (o `1,15` do
/// grupo B, o `offset_y` do C, o `BAND_GAP` do D), e a cura é sempre a mesma:
/// o oráculo lê a forma, ancorada em algo que a própria banda carrega.
fn relative_to_pin(p: &[[f32; 2]], i: usize) -> [f32; 2] {
    [p[i][0] - p[0][0], p[i][1] - p[0][1]]
}

/// **A CENA MONTA AS QUATRO BANDAS E CADA UMA TERMINA NUM OUTPUT.**
///
/// ⚠️ O laço de render **re-resolve** os sinks a partir dos nós de saída do
/// grafo, então uma banda sem `motion.output` cozinha certo, satisfaz todo gate
/// de comportamento e **desenha nada** — indistinguível da feature quebrada.
#[test]
fn the_scene_builds_four_bands_that_all_end_in_an_output() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_weight_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas, dois pares");
    for s in &sinks {
        assert_eq!(
            doc.graph.node(*s).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "toda banda termina num Output"
        );
    }
    assert_eq!(band_labels().count(), 4, "um rótulo por banda");
}

/// **A CAUDA DESPRENDE, E O CORPO NÃO A SEGUE.**
///
/// O CONTROLE é a primeira metade: sem ele *"as duas diferem"* seria satisfeito
/// por um corpo que simplesmente caísse mais — a pergunta é se a cauda **sai** e
/// se o resto **fica**.
#[test]
fn the_band_lets_the_tail_fall_off_without_dragging_the_body() {
    let p = run(150);
    let plain = &p[0];
    let banded = &p[1];
    assert!(!plain.is_empty() && plain.len() == banded.len());

    // ⚠️ **A régua é a SEPARAÇÃO, não a forma da cauda** — e a medição derrubou
    // a primeira versão deste gate, que comparava a extensão INTERNA das duas
    // últimas linhas (0,7000 contra 0,6999, indistinguíveis). No shape matching
    // **não existe restrição de distância entre partículas**: o único fio é o
    // puxão ao goal, então uma cauda de peso zero não tem fio nenhum e cai
    // RÍGIDA, mantendo o próprio espaçamento. Ela não se deforma — ela SAI.
    let plain_span = drop_depth(plain);
    let banded_span = drop_depth(banded);
    assert!(
        banded_span > plain_span * 2.0,
        "a cauda de peso zero tem de SEPARAR-SE do corpo; o controle mede \
         {plain_span:.4} de ponta a ponta e a banda {banded_span:.4}"
    );

    // E o miolo (a linha do meio) tem de ficar onde estava: um corpo que a
    // seguisse teria a forma inteira arrastada pela cauda solta. ⚠️ **Relativo
    // à linha pinada**, senão isto mede o `BAND_DY` que a cena põe entre as
    // duas bandas (medido: 8,9901, que é o vão de 9,0).
    let drift = (12..18)
        .map(|i| {
            let (a, b) = (relative_to_pin(plain, i), relative_to_pin(banded, i));
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(
        drift < 0.5,
        "o corpo NÃO segue o que se soltou dele; a linha do meio andou {drift:.4}"
    );
}

/// **A CORDA DE UM SUB-PASSO ESTICA, A DE OITO NÃO.**
///
/// A régua é o VÃO entre nós vizinhos contra o comprimento de repouso — a
/// grandeza que o solver de distância existe para segurar.
#[test]
fn the_substepped_rope_keeps_its_length_where_the_single_step_one_stretches() {
    let p = run(150);
    let one = worst_span(&p[2]);
    let many = worst_span(&p[3]);
    let rest = 6.0f32 / (ROPE_COUNT - 1.0);
    assert!(
        one > rest * 1.15,
        "o controle TEM de esticar, senão a cena não contém o fenômeno; \
         vão {one:.4} contra repouso {rest:.4}"
    );
    assert!(
        many < one * 0.75,
        "oito sub-passos têm de segurar o comprimento; um passo esticou até \
         {one:.4} e oito até {many:.4} (repouso {rest:.4})"
    );
}

/// **A SONDA** — imprime os números que a mensagem do smoke cita, para nenhum
/// deles ser escrito de memória.
#[test]
#[ignore = "sonda"]
fn measure_what_the_scene_shows() {
    let p = run(150);
    let n = p[0].len();
    let tail = n - 12;
    println!("corpo (150 tiques, malha 6x6):");
    println!(
        "  controle : cauda cai {:.4}   corpo inteiro {:.4}",
        drop_depth(&p[0][tail..]),
        drop_depth(&p[0])
    );
    println!(
        "  com banda: cauda cai {:.4}   corpo inteiro {:.4}",
        drop_depth(&p[1][tail..]),
        drop_depth(&p[1])
    );
    let drift = (12..18)
        .map(|i| {
            let (a, b) = (relative_to_pin(&p[0], i), relative_to_pin(&p[1], i));
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    let raw = (12..18)
        .map(|i| {
            let (a, b) = (p[0][i], p[1][i]);
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    println!("  deriva da linha do MEIO, RELATIVA ao pino: {drift:.4}");
    println!("  (a mesma medida em coordenada CRUA: {raw:.4} = o BAND_DY da cena, nao o corpo)");
    let rest = 6.0f32 / (ROPE_COUNT - 1.0);
    println!("corda (repouso por segmento {rest:.4}):");
    println!(
        "  substeps  1: pior vao {:.4}  ({:.1}% esticado)",
        worst_span(&p[2]),
        (worst_span(&p[2]) / rest - 1.0) * 100.0
    );
    println!(
        "  substeps {}: pior vao {:.4}  ({:.1}% esticado)",
        SUBSTEPS,
        worst_span(&p[3]),
        (worst_span(&p[3]) / rest - 1.0) * 100.0
    );
}

/// **A CENA TEM DE DIZER O MESMO NO CAMINHO QUE O APP CORRE** — e é exactamente aqui que um
/// defeito viveu três dias sem ninguém o ver.
///
/// O `run` acima marcha com `cook` + `advance_tick` e mais nada. O app põe **o pump** à frente:
/// `MotionCookPump::substep_declared_zones` procura declarantes de `substeps` e sub-tica o cone de
/// cada um. Enquanto o param desta corda se chamava `substeps` (a chave que a folha 13 reservou
/// para o relógio do GRAFO), ela virava declarante sem querer e o app corria **as duas leis** — o
/// laço interno dela dentro de cada sub-passada do relógio. Medido na altura: a cauda caía
/// **−1,2384** contra os **−5,9299** que os gates do crate da corda medem.
///
/// ⚠️ **A cura foi a chave (`solver_substeps`), e este gate é o que impede a recaída AQUI:** ele
/// marcha a cena pelas duas rotas e exige o MESMO resultado, ao bit. Se alguém devolver o nome
/// reservado a esta corda, é este teste que fica vermelho na cena que o Enio smokou.
#[test]
fn the_scenes_ropes_read_the_same_through_the_pump_the_app_runs() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_weight_demo_document(&mut doc, &reg).expect("a cena monta");
    let plain = run(60);

    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..60 {
        let t = k as f64 * DT;
        // A ordem do app: os substeps declarados ANTES do cook do quadro.
        if let Some(frame_start) = cook.prev_playhead() {
            for island in ph2d_nodegraph::cook::substep_islands(&doc.graph, &reg) {
                cook.substep(
                    &doc.graph,
                    &reg,
                    island.root,
                    frame_start,
                    t,
                    island.substeps,
                )
                .expect("substep");
            }
        }
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("o tique avança o `pre`");
        for (i, s) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, &reg, *s, t).expect("a banda coze");
            if let Some(Column::Vec2(p)) = v[0].as_stream().get("P") {
                out[i] = p.clone();
            }
        }
    }

    // O CONTROLE de que a fixture não está morta: a cena de facto simula.
    assert!(
        worst_span(&plain[3]) > 0.0,
        "a corda substepada precisa de ter pose"
    );
    for (band, (a, b)) in plain.iter().zip(&out).enumerate() {
        assert_eq!(a.len(), b.len(), "a banda {band} muda de tamanho pelo pump");
        for (i, (p, q)) in a.iter().zip(b).enumerate() {
            for c in 0..2 {
                assert_eq!(
                    p[c].to_bits(),
                    q[c].to_bits(),
                    "banda {band}, ponto {i}, eixo {c}: o pump mudou a cena ({} vs {})",
                    p[c],
                    q[c]
                );
            }
        }
    }
}
