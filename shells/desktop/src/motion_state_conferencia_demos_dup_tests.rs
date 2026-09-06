//! Gates da cena `=110` — TODO o duplicator.
//!
//! ⚠️ **Estes gates medem o que a cena DESENHA, não o que ela monta** — a lição da cena `=9`
//! desta mesma linha: cinco gates verdes sobre uma cena cujos pares saíam iguais, porque todos
//! mediam a montagem. Aqui cada banda é COZIDA e a afirmação é sobre as colunas que saem: se
//! duas bandas que têm de diferir saírem iguais, o gate reprova antes do Enio.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_dup_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

struct Band {
    p: Vec<[f32; 2]>,
    size: Vec<[f32; 2]>,
    rot: Vec<f32>,
    tint: Vec<[f32; 4]>,
}

fn bake(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Band> {
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|&s| {
            let out = cook.cook(&doc.graph, reg, s, 0.0).expect("coze");
            let st = out[0].as_stream();
            Band {
                p: match st.get("P") {
                    Some(Column::Vec2(v)) => v.clone(),
                    _ => Vec::new(),
                },
                size: match st.get("size") {
                    Some(Column::Vec2(v)) => v.clone(),
                    _ => Vec::new(),
                },
                rot: match st.get("rot") {
                    Some(Column::Scalar(v)) => v.clone(),
                    _ => Vec::new(),
                },
                tint: match st.get("tint") {
                    Some(Column::Vec4(v)) => v.clone(),
                    _ => Vec::new(),
                },
            }
        })
        .collect()
}

/// Quantas cores distintas há na banda (a `1e-4`, muito abaixo do passo da rampa).
fn distinct_tints(b: &Band) -> usize {
    let mut seen: Vec<[f32; 4]> = Vec::new();
    for t in &b.tint {
        if !seen
            .iter()
            .any(|s| s.iter().zip(t).all(|(a, c)| (a - c).abs() < 1e-4))
        {
            seen.push(*t);
        }
    }
    seen.len()
}

/// A soma dos canais RGB da banda — a régua de *«mais claro»* / *«mais escuro»*.
fn brightness(b: &Band) -> f32 {
    b.tint.iter().map(|t| t[0] + t[1] + t[2]).sum()
}

/// **A CENA MONTA AS TREZE BANDAS, E CADA UMA COSPE O QUE PROMETE.**
///
/// ⚠️ A contagem por banda é o oráculo mais barato do carimbo: o produto cartesiano tem de dar
/// `formas × pontos`, e os dois modos de variante **uma por ponto**. FALSIFICADO por qualquer
/// banda sair vazia (um fio que não ligou devolve `None` e o `expect` cai antes daqui) ou pelo
/// `pick` deixar de partir o mundo em dois.
#[test]
fn the_scene_builds_every_band_with_the_count_it_promises() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), BANDS, "um sink por banda");
    assert_eq!(band_labels().count(), BANDS, "um rotulo por banda");
    assert_eq!(captions().len(), BANDS, "uma ficha por banda");
    let n = PIECES as usize;
    let esperado = [
        n,                                   // 1 · o carimbo
        COMET.len() * COMET_POINTS as usize, // 2 · o cometa inteiro em cada ponto
        n,                                   // 3 · o giro
        TRIO_RGB.len() * n,                  // 4 · Pick Off: o PRODUTO
        n,                                   // 5 · Cycle: uma por ponto
        n,                                   // 6 · Random: uma por ponto
        n,
        n,
        n, // 7-9 · Point Scale
        n,
        n,
        n,
        n, // 10-13 · Transfer
    ];
    for (k, (b, want)) in bake(&doc, &reg, &sinks).iter().zip(esperado).enumerate() {
        assert_eq!(b.p.len(), want, "banda {}: contagem", k + 1);
    }
}

/// ⭐⭐ **A FORMA INTEIRA POUSA EM CADA PONTO, E O PONTO SOMA O LUGAR DELE.**
///
/// A banda 2 tem três pontos e um cometa de três peças: as nove cópias têm de estar em **nove**
/// x distintos, e cada trio tem de repetir os MESMOS deslocamentos internos. FALSIFICADO por o
/// `P` do ponto substituir o da forma em vez de somar — aí as três peças colapsavam num ponto só
/// e sobravam três x.
#[test]
fn the_whole_shape_lands_on_each_point_and_the_point_adds_its_place() {
    let (doc, reg, sinks) = scene();
    let b = &bake(&doc, &reg, &sinks)[1];
    let mut xs: Vec<f32> = b.p.iter().map(|p| p[0]).collect();
    xs.sort_by(f32::total_cmp);
    xs.dedup_by(|a, c| (*a - *c).abs() < 1e-4);
    assert_eq!(xs.len(), 9, "tres pontos x tres pecas em nove x distintos");
    // Os deslocamentos INTERNOS repetem-se ponto a ponto: a distância entre a 1.ª e a 3.ª peça
    // de cada trio é sempre a do cometa.
    let vao = COMET[2].0 - COMET[0].0;
    for t in 0..COMET_POINTS as usize {
        let d = xs[t * 3 + 2] - xs[t * 3];
        assert!(
            (d - vao).abs() < 1e-3,
            "trio {t}: o cometa chegou deformado ({d} contra {vao})"
        );
    }
}

/// ⭐⭐ **O GIRO DO PONTO SOMA-SE AO DA FORMA** — e a banda vizinha, sem ele, não tem giro
/// nenhum. FALSIFICADO por o `rot` do ponto ser deitado fora (a fila sairia toda direita) ou por
/// a rampa não crescer.
#[test]
fn the_point_adds_its_turn_and_the_plain_band_has_none() {
    let (doc, reg, sinks) = scene();
    let bandas = bake(&doc, &reg, &sinks);
    let turn = &bandas[2];
    assert_eq!(turn.rot.len(), PIECES as usize, "toda copia tem giro");
    for w in turn.rot.windows(2) {
        assert!(w[1] > w[0], "a fila tem de torcer sempre no mesmo sentido");
    }
    assert!(turn.rot[0].abs() < 1e-4, "a primeira nao gira");
    assert!(
        (turn.rot[PIECES as usize - 1] - ROT_SPAN).abs() < 1e-3,
        "a ultima gira o vao inteiro, e girou {}",
        turn.rot[PIECES as usize - 1]
    );
    // O CONTROLE: sem a rampa o carimbo não inventa giro nenhum.
    assert!(
        bandas[0].rot.iter().all(|r| r.abs() < 1e-4),
        "a banda 1 nao tem giro para somar"
    );
}

/// ⭐⭐⭐ **AS TRÊS BANDAS DO `PICK` SÃO TRÊS FIGURAS DIFERENTES.**
///
/// `Off` é o produto; `Cycle` reparte as formas **na ordem**; `Random` reparte-as **fora dela**.
/// ⚠️ A régua do `Random` é a SEQUÊNCIA de cores, não a contagem — os dois modos de variante
/// emitem o mesmo número de peças, e um `Random` que saísse na ordem do `Cycle` seria uma banda
/// a ensinar o contrário do que diz. FALSIFICADO por qualquer par sair igual.
#[test]
fn cycle_deals_the_shapes_in_order_and_random_does_not() {
    let (doc, reg, sinks) = scene();
    let bandas = bake(&doc, &reg, &sinks);
    let (cycle, random) = (&bandas[4], &bandas[5]);
    // A cor de cada cópia diz QUAL forma pousou ali; o `y` diz o mesmo, e por construção.
    let cor = |b: &Band| -> Vec<usize> {
        b.tint
            .iter()
            .map(|t| {
                TRIO_RGB
                    .iter()
                    .position(|c| c.iter().zip(t).all(|(a, v)| (a - v).abs() < 1e-3))
                    .unwrap_or(usize::MAX)
            })
            .collect()
    };
    let seq_cycle = cor(cycle);
    let seq_random = cor(random);
    assert!(
        seq_cycle.iter().all(|&i| i != usize::MAX),
        "as copias do Cycle sao as formas do trio, e sairam {seq_cycle:?}"
    );
    let n = TRIO_RGB.len();
    for (i, forma) in seq_cycle.iter().enumerate() {
        assert_eq!(
            *forma,
            i % n,
            "Cycle: a copia {i} tem de ser a forma {}",
            i % n
        );
    }
    assert_ne!(
        seq_random, seq_cycle,
        "a semente escolhida faz o Random sair na ORDEM: a banda ensinaria o contrario do rotulo"
    );
}

/// ⭐ **A BANDA `RANDOM` MOSTRA AS TRÊS FORMAS** — a semente é uma CALIBRAÇÃO, e sem gate ela é
/// um número que alguém mexe sem saber o que perde: das 24 primeiras, **nove** deixam uma forma
/// de fora, e a banda passaria a ler-se como *«Random escolhe entre duas»*. FALSIFICADO por
/// trocar a semente por uma dessas.
#[test]
fn the_random_band_shows_every_shape() {
    let (doc, reg, sinks) = scene();
    let b = &bake(&doc, &reg, &sinks)[5];
    let mut alturas: Vec<i32> =
        b.p.iter()
            .map(|p| ((p[1] - ROW_Y[1]) / SPREAD).round() as i32)
            .collect();
    alturas.sort_unstable();
    alturas.dedup();
    assert_eq!(
        alturas.len(),
        TRIO_RGB.len(),
        "a semente {PICK_SEED} deixa uma forma de fora: sairam as alturas {alturas:?}"
    );
}

/// ⭐⭐ **O `POINT SCALE` FAZ AS CÓPIAS CRESCEREM, E `0` É O MUNDO DE SEMPRE.**
///
/// FALSIFICADO por as três bandas saírem do mesmo tamanho (o param inerte) ou por a de `0` já
/// trazer a escala do ponto (o mundo de sempre teria mudado).
#[test]
fn point_scale_grows_the_copies_and_zero_is_the_old_world() {
    let (doc, reg, sinks) = scene();
    let bandas = bake(&doc, &reg, &sinks);
    let lado = |b: &Band| b.size.first().map_or(0.0, |s| s[0]);
    let (zero, meio, um) = (lado(&bandas[6]), lado(&bandas[7]), lado(&bandas[8]));
    assert!(
        (zero - PIECE).abs() < 1e-4,
        "com Point Scale = 0 a copia tem o tamanho da FORMA ({zero} contra {PIECE})"
    );
    assert!(
        (um - PIECE * POINT_MUL).abs() < 1e-4,
        "com Point Scale = 1 a escala do ponto entra inteira ({um})"
    );
    assert!(zero < meio && meio < um, "{zero} < {meio} < {um}");
    // E o CONTROLE do outro lado: a banda 1, cujos pontos não trazem escala, fica no tamanho da
    // forma — senão «0 é o mundo de sempre» seria uma frase sobre nada.
    assert!((lado(&bandas[0]) - PIECE).abs() < 1e-4);
}

/// ⭐⭐⭐ **AS QUATRO BANDAS DO `TRANSFER` SÃO QUATRO FIGURAS DIFERENTES.**
///
/// `Shape Wins` deita fora a rampa (uma cor só); os outros três deixam-na chegar, cada um à sua
/// maneira — `Add` CLAREIA e `Multiply` ESCURECE contra o `Point Wins`. FALSIFICADO por dois
/// modos quaisquer saírem com as mesmas cores.
#[test]
fn the_four_transfer_modes_are_four_different_pictures() {
    let (doc, reg, sinks) = scene();
    let bandas = bake(&doc, &reg, &sinks);
    let (shape, point, add, mul) = (&bandas[9], &bandas[10], &bandas[11], &bandas[12]);
    assert_eq!(
        distinct_tints(shape),
        1,
        "Shape Wins: as cinco copias saem com a cor da FORMA"
    );
    for (nome, b) in [("Point Wins", point), ("Add", add), ("Multiply", mul)] {
        assert!(
            distinct_tints(b) > 1,
            "{nome}: a rampa do arranjo tem de chegar"
        );
    }
    let (bp, ba, bm) = (brightness(point), brightness(add), brightness(mul));
    assert!(ba > bp, "Add tem de CLAREAR ({ba} contra {bp})");
    assert!(bm < bp, "Multiply tem de ESCURECER ({bm} contra {bp})");
    for (a, b) in [(9, 10), (9, 11), (9, 12), (10, 11), (10, 12), (11, 12)] {
        assert!(
            bandas[a].tint != bandas[b].tint,
            "as bandas {} e {} do Transfer saem iguais",
            a + 1,
            b + 1
        );
    }
}

/// ⭐⭐ **NENHUMA BANDA INVADE A VIZINHA** — treze blocos numa tela, e a leitura depende de cada
/// um ficar no seu quadrante.
///
/// FALSIFICADO por qualquer mudança de `COL_STEP`/`ROW_Y`/`GAP`/`SPREAD` que faça duas caixas
/// tocarem-se: é o «algum bloco invadir o vizinho» do anúncio, medido em vez de olhado.
#[test]
fn no_band_runs_into_its_neighbour() {
    let (doc, reg, sinks) = scene();
    let caixas: Vec<[f32; 4]> = bake(&doc, &reg, &sinks)
        .iter()
        .map(|b| {
            // A caixa inclui METADE do lado de cada peça — uma peça é desenhada centrada no
            // `P`. ⚠️ **Uma peça GIRADA é mais larga**: um quadrado a 45° mede `lado·√2`, e a
            // banda do giro leria 41 % mais estreita do que desenha.
            let girada = b.rot.iter().any(|r| r.abs() > 1e-4);
            let diagonal = if girada {
                std::f32::consts::SQRT_2
            } else {
                1.0
            };
            let meio = b.size.first().map_or(PIECE, |s| s[0].max(s[1])) * 0.5 * diagonal;
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for p in &b.p {
                x0 = x0.min(p[0] - meio);
                y0 = y0.min(p[1] - meio);
                x1 = x1.max(p[0] + meio);
                y1 = y1.max(p[1] + meio);
            }
            [x0, y0, x1, y1]
        })
        .collect();
    for i in 0..caixas.len() {
        for j in (i + 1)..caixas.len() {
            let (a, b) = (caixas[i], caixas[j]);
            let sobrepoe = a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3];
            assert!(
                !sobrepoe,
                "as bandas {} e {} sobrepoem-se: {a:?} contra {b:?}",
                i + 1,
                j + 1
            );
        }
    }
}

/// ⭐⭐ **AS FICHAS SÃO FICHAS, E CADA UMA POUSA SOBRE A SUA BANDA.**
///
/// ⚠️ **O gate da legenda da casa não alcança esta cena, e a razão é estrutural:** ele lê uma
/// lista escrita à mão e afirma que as fichas vêm aos PARES (esquerda contra direita), que é a
/// forma das cenas de conferência. Esta tem fileiras de três e de quatro. ⛔ Afrouxar aquele
/// gate para caber aqui tiraria a régua às seis cenas que ele defende — então a régua desta cena
/// vive aqui, e afirma o mesmo: uma ficha é CURTA, diz alguma coisa, e pousa sobre a figura que
/// explica e não sobre a vizinha.
#[test]
fn every_caption_is_a_chip_over_its_own_band() {
    /// O mesmo número do gate da casa: quantos caracteres cabem sem a ficha invadir a vizinha.
    const MAX_CHARS: usize = 40;
    let (doc, reg, sinks) = scene();
    let bandas = bake(&doc, &reg, &sinks);
    let caps = captions();
    assert_eq!(caps.len(), BANDS);
    for (k, c) in caps.iter().enumerate() {
        assert!(
            !c.text.trim().is_empty() && c.text.chars().count() <= MAX_CHARS,
            "banda {}: ficha de {} chars: {:?}",
            k + 1,
            c.text.chars().count(),
            c.text
        );
        // Ela pousa no x da banda e ACIMA das peças dela.
        let at = quadrant(k);
        assert!(
            (c.world[0] - at[0]).abs() < 1e-4,
            "banda {}: fora do x",
            k + 1
        );
        let topo = bandas[k].p.iter().fold(f32::MIN, |m, p| m.max(p[1]));
        assert!(
            c.world[1] > topo,
            "banda {}: a ficha caiu sobre as pecas ({} contra {topo})",
            k + 1,
            c.world[1]
        );
    }
    // E o CONTROLE de que a barra não é folgada de graça.
    assert!(
        caps.iter().any(|c| c.text.chars().count() > MAX_CHARS / 2),
        "as fichas dizem alguma coisa"
    );
    // ⚠️ **Nenhuma ficha pousa sobre a fileira DE CIMA** — é o defeito que uma legenda tem e um
    // `eprintln!` não tem: o rótulo certo sobre a figura errada.
    for (k, c) in caps.iter().enumerate() {
        for (j, b) in bandas.iter().enumerate() {
            if quadrant(j)[1] <= quadrant(k)[1] {
                continue; // só as fileiras ACIMA desta
            }
            let baixo = b.p.iter().fold(f32::MAX, |m, p| m.min(p[1])) - PIECE;
            assert!(
                c.world[1] < baixo,
                "a ficha da banda {} sobe ate' a banda {}",
                k + 1,
                j + 1
            );
        }
    }
}

/// **A PEGADA DA CENA** — quantas peças cada banda desenha e onde ela cai, mais a caixa do
/// conjunto. É o número que diz se a tela cabe no enquadramento com que o canvas abre.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins measure_the_dup_scene -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn measure_the_dup_scene() {
    let (doc, reg, sinks) = scene();
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    eprintln!("\n  banda | pecas |            caixa            | rotulo");
    for ((k, label), b) in band_labels().zip(bake(&doc, &reg, &sinks)) {
        let meio = b.size.first().map_or(PIECE, |s| s[0].max(s[1])) * 0.5;
        let (a0, b0) = (
            b.p.iter().fold(f32::MAX, |m, p| m.min(p[0] - meio)),
            b.p.iter().fold(f32::MAX, |m, p| m.min(p[1] - meio)),
        );
        let (a1, b1) = (
            b.p.iter().fold(f32::MIN, |m, p| m.max(p[0] + meio)),
            b.p.iter().fold(f32::MIN, |m, p| m.max(p[1] + meio)),
        );
        (x0, y0, x1, y1) = (x0.min(a0), y0.min(b0), x1.max(a1), y1.max(b1));
        eprintln!(
            "   {:>2}   |  {:>3}  | {a0:>6.2} {b0:>6.2} {a1:>6.2} {b1:>6.2} | {}",
            k + 1,
            b.p.len(),
            short_of(label)
        );
    }
    eprintln!(
        "\n  a cena inteira: x {x0:.2}..{x1:.2} ({:.2} de largura) · y {y0:.2}..{y1:.2} ({:.2} de altura)\n",
        x1 - x0,
        y1 - y0
    );
}

/// **QUE SEMENTE FAZ O `RANDOM` MOSTRAR AS TRÊS FORMAS** — cinco sorteios sobre três formas
/// deixam uma de fora com facilidade, e uma banda «Random» que só mostra duas cores ensina
/// menos do que promete. Esta sonda escolhe o número que está no `PICK_SEED`.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins which_seed_shows_every_shape -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de calibracao, nao um gate"]
fn which_seed_shows_every_shape() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registra");
    for seed in 0..24u32 {
        let mut doc = MotionDoc::default();
        let sinks = build_dup_demo_document(&mut doc, &reg).expect("monta");
        // A banda 6 é a do Random: reescrever a semente dela e recozinhar.
        let alvo = doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.duplicator")
            .nth(5)
            .map(|n| n.id)
            .expect("a sexta banda");
        doc.graph.set_param(alvo, "seed", seed as f32);
        let b = &bake(&doc, &reg, &sinks)[5];
        let seq: Vec<i32> =
            b.p.iter()
                .map(|p| ((p[1] - ROW_Y[1]) / SPREAD).round() as i32)
                .collect();
        let mut distintas = seq.clone();
        distintas.sort_unstable();
        distintas.dedup();
        eprintln!(
            "  seed {seed:>2}: {seq:?}  ({} formas distintas)",
            distintas.len()
        );
    }
}
