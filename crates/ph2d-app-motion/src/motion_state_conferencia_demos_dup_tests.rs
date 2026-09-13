//! Gates da cena `=110` — TODO o duplicator.
//!
//! ⚠️ **Estes gates medem o que a cena DESENHA, não o que ela monta** — a lição da cena `=9`
//! desta mesma linha: cinco gates verdes sobre uma cena cujos pares saíam iguais, porque todos
//! mediam a montagem. Aqui cada banda é COZIDA e a afirmação é sobre as colunas que saem: se
//! duas bandas que têm de diferir saírem iguais, o gate reprova antes do Enio.
//!
//! ⚠️ **Irmão de [`super::graph_tests`] por RESPONSABILIDADE** (HR-18): lá pergunta-se como a cena
//! está LIGADA (uma cadeia por saída · o que entra em cada porta), aqui o que ela DESENHA. As duas
//! perguntas nasceram de dois reports diferentes do Enio, no mesmo dia.
//!
//! ⚠️⚠️ **E o cook tem de ser o da SHELL, não um `Cook` nu.** A forma de cada banda é um
//! `source.shape`, que lê um EXTERNAL que a shell publica (`motion_shape_gen::publish`): num
//! cook virgem ele emite **zero**, e as treze bandas sairiam vazias com os gates a dizer que a
//! cena não monta. É o precedente que a `=91` já nomeia — *«medir as posições dela aqui seria a
//! sonda a acusar-se a si própria»* —, e aqui ele não é uma isenção: é a razão de a cena ser
//! construída dentro de um [`MotionState`].

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// A cena montada **dentro de um `MotionState`**, com a geometria das formas já publicada.
pub(super) fn scene() -> (MotionState, Vec<NodeId>) {
    let mut state = MotionState::new();
    // ⚠️ Um `PH2D_GPU_COOK_DEMO` no ambiente faria o `MotionState::new` semear uma cena e as
    // contagens deste ficheiro mediriam o grafo de outra pessoa. Falha alto em vez de mentir.
    assert!(
        state.doc.graph.nodes().is_empty(),
        "o `MotionState` nasceu com {} nos: desligue o PH2D_GPU_COOK_DEMO para correr estes gates",
        state.doc.graph.nodes().len()
    );
    let sinks = build_dup_demo_document(&mut state.doc, &state.registry).expect("a cena monta");
    state
        .doc
        .graph
        .validate(&state.registry)
        .expect("bem-tipada");
    // A geometria de cada `source.shape` entra no canal externo que o nó lê.
    crate::motion_shape_gen::publish(&mut state, 0.0);
    (state, sinks)
}

struct Band {
    p: Vec<[f32; 2]>,
    size: Vec<[f32; 2]>,
    rot: Vec<f32>,
    tint: Vec<[f32; 4]>,
    /// **QUAL FORMA** pousou em cada cópia — o oráculo do `Pick`, e literalmente a pergunta que
    /// ele responde. ⛔ A cor seria um substituto: duas formas podem partilhá-la.
    geom: Vec<f32>,
}

fn bake(state: &mut MotionState, sinks: &[NodeId]) -> Vec<Band> {
    let mut out = Vec::with_capacity(sinks.len());
    for &s in sinks {
        let cooked = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, s, 0.0)
            .expect("coze");
        let st = cooked[0].as_stream();
        let vec2 = |n: &str| match st.get(n) {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let scalar = |n: &str| match st.get(n) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        out.push(Band {
            p: vec2("P"),
            size: vec2("size"),
            rot: scalar("rot"),
            tint: match st.get("tint") {
                Some(Column::Vec4(v)) => v.clone(),
                _ => Vec::new(),
            },
            geom: scalar("geometry_id"),
        });
    }
    out
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

/// A meia-extensão de uma cópia. ⚠️ **É o `size`, não metade dele:** a geometria de um
/// `source.shape` vive em RAIO 1, então uma cópia de `size` ocupa `size` para cada lado.
fn half_extent(b: &Band) -> f32 {
    let s = b.size.iter().fold(0.0_f32, |m, s| m.max(s[0].max(s[1])));
    let s = if s > 0.0 { s } else { PIECE };
    // Uma forma GIRADA varre um disco do próprio raio, então a caixa dela não cresce — o raio
    // já é o pior caso em qualquer ângulo.
    s
}

/// **A CENA MONTA AS TREZE BANDAS, E CADA UMA COSPE O QUE PROMETE.**
///
/// ⚠️ A contagem por banda é o oráculo mais barato do carimbo: o produto cartesiano tem de dar
/// `formas × pontos`, e os dois modos de variante **uma por ponto**. FALSIFICADO por qualquer
/// banda sair vazia — que é exactamente o que acontece se a geometria do `source.shape` não for
/// publicada — ou pelo `pick` deixar de partir o mundo em dois.
#[test]
fn the_scene_builds_every_band_with_the_count_it_promises() {
    let (mut state, sinks) = scene();
    assert_eq!(sinks.len(), BANDS, "um sink por banda");
    assert_eq!(band_labels().count(), BANDS, "um rotulo por banda");
    assert_eq!(captions().len(), BANDS, "uma ficha por banda");
    let n = PIECES as usize;
    let sc = SCALE_POINTS as usize;
    let esperado = [
        n,                                   // 1 · o carimbo
        COMET.len() * COMET_POINTS as usize, // 2 · o cometa inteiro em cada ponto
        n,                                   // 3 · o giro
        TRIO_KINDS.len() * n,                // 4 · Pick Off: o PRODUTO
        n,                                   // 5 · Cycle: uma por ponto
        n,                                   // 6 · Random: uma por ponto
        sc,
        sc,
        sc, // 7-9 · Point Scale
        n,
        n,
        n,
        n, // 10-13 · Transfer
    ];
    for (k, (b, want)) in bake(&mut state, &sinks).iter().zip(esperado).enumerate() {
        assert_eq!(b.p.len(), want, "banda {}: contagem", k + 1);
        assert!(
            !b.geom.is_empty(),
            "banda {}: toda copia carrega a geometria da forma",
            k + 1
        );
    }
}

/// ⭐⭐ **AS TRÊS BANDAS DO `PICK` CARIMBAM AS MESMAS TRÊS FORMAS** — a propriedade que a partilha
/// comprava, agora medida na SAÍDA em vez de garantida por referência.
///
/// ⚠️ **O oráculo é o `geometry_id`**, que é literalmente *qual forma*. FALSIFICADO por mudar a
/// silhueta de uma das três numa das bandas.
#[test]
fn the_three_pick_bands_stamp_the_same_shapes() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
    let formas = |b: &Band| -> Vec<i64> {
        let mut v: Vec<i64> = b.geom.iter().map(|g| *g as i64).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let (off, cycle, random) = (formas(&bandas[3]), formas(&bandas[4]), formas(&bandas[5]));
    assert_eq!(
        off.len(),
        TRIO_KINDS.len(),
        "o produto pousa as tres formas, e pousou {off:?}"
    );
    for (nome, a) in [("Cycle", &cycle), ("Random", &random)] {
        assert!(
            a.iter().all(|c| off.contains(c)),
            "{nome} carimba uma forma que o Off nao tem: {a:?} contra {off:?}"
        );
    }
    // E o CONTROLE do outro lado: as quatro bandas do `Transfer` carimbam UMA forma só, a mesma.
    let da_cor: Vec<i64> = (9..BANDS).map(|k| formas(&bandas[k])[0]).collect();
    assert!(
        da_cor.iter().all(|g| *g == da_cor[0]) && formas(&bandas[9]).len() == 1,
        "as quatro do Transfer tem de carimbar a MESMA forma: {da_cor:?}"
    );
}

/// ⭐⭐ **A FORMA INTEIRA POUSA EM CADA PONTO, E O PONTO SOMA O LUGAR DELE.**
///
/// A banda 2 tem três pontos e um cometa de três peças: as nove cópias têm de estar em **nove**
/// x distintos, e cada trio tem de repetir os MESMOS deslocamentos internos. FALSIFICADO por o
/// `P` do ponto substituir o da forma em vez de somar — aí as três peças colapsavam num ponto só
/// e sobravam três x.
#[test]
fn the_whole_shape_lands_on_each_point_and_the_point_adds_its_place() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
    let b = &bandas[1];
    let mut xs: Vec<f32> = b.p.iter().map(|p| p[0]).collect();
    xs.sort_by(f32::total_cmp);
    xs.dedup_by(|a, c| (*a - *c).abs() < 1e-4);
    assert_eq!(xs.len(), 9, "tres pontos x tres pecas em nove x distintos");
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
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
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
    assert!(
        bandas[0].rot.iter().all(|r| r.abs() < 1e-4),
        "a banda 1 nao tem giro para somar"
    );
}

/// ⭐⭐⭐ **AS TRÊS BANDAS DO `PICK` SÃO TRÊS FIGURAS DIFERENTES.**
///
/// `Off` é o produto; `Cycle` reparte as formas **na ordem**; `Random` reparte-as **fora dela**.
/// ⚠️ A régua do `Random` é a SEQUÊNCIA de formas, não a contagem — os dois modos de variante
/// emitem o mesmo número de peças, e um `Random` que saísse na ordem do `Cycle` seria uma banda
/// a ensinar o contrário do que diz. FALSIFICADO por qualquer par sair igual.
#[test]
fn cycle_deals_the_shapes_in_order_and_random_does_not() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
    // A ordem em que as formas aparecem no PRODUTO é a ordem do trio: o `Off` é shape-major, e
    // as primeiras `PIECES` cópias são todas a forma 0.
    let ordem: Vec<i64> = (0..TRIO_KINDS.len())
        .map(|i| bandas[3].geom[i * PIECES as usize] as i64)
        .collect();
    let posto = |b: &Band| -> Vec<usize> {
        b.geom
            .iter()
            .map(|g| {
                ordem
                    .iter()
                    .position(|o| *o == *g as i64)
                    .unwrap_or(usize::MAX)
            })
            .collect()
    };
    let (seq_cycle, seq_random) = (posto(&bandas[4]), posto(&bandas[5]));
    assert!(
        seq_cycle.iter().all(|&i| i != usize::MAX),
        "as copias do Cycle sao as formas do trio, e sairam {seq_cycle:?}"
    );
    let n = TRIO_KINDS.len();
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
/// de fora, e a banda passaria a ler-se como *«Random escolhe entre duas»*.
#[test]
fn the_random_band_shows_every_shape() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
    let mut formas: Vec<i64> = bandas[5].geom.iter().map(|g| *g as i64).collect();
    formas.sort_unstable();
    formas.dedup();
    assert_eq!(
        formas.len(),
        TRIO_KINDS.len(),
        "a semente {PICK_SEED} deixa uma forma de fora: sairam {formas:?}"
    );
}

/// ⭐⭐ **O `POINT SCALE` FAZ AS CÓPIAS CRESCEREM, E `0` É O MUNDO DE SEMPRE.**
///
/// FALSIFICADO por as três bandas saírem do mesmo tamanho (o param inerte) ou por a de `0` já
/// trazer a escala do ponto (o mundo de sempre teria mudado).
#[test]
fn point_scale_grows_the_copies_and_zero_is_the_old_world() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
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
    assert!((lado(&bandas[0]) - PIECE).abs() < 1e-4);
}

/// ⭐⭐⭐ **AS QUATRO BANDAS DO `TRANSFER` SÃO QUATRO FIGURAS DIFERENTES.**
///
/// `Shape Wins` deita fora a rampa (uma cor só); os outros três deixam-na chegar, cada um à sua
/// maneira — `Add` CLAREIA e `Multiply` ESCURECE contra o `Point Wins`. FALSIFICADO por dois
/// modos quaisquer saírem com as mesmas cores.
#[test]
fn the_four_transfer_modes_are_four_different_pictures() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
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
    // E o CONTROLE: a forma cinzenta é mesmo a da cena (senão «a cor da forma» seria uma frase
    // sobre nada).
    assert!(
        (shape.tint[0][0] - SHAPE_RGB[0]).abs() < 1e-3,
        "a forma do Transfer e' a cinzenta da cena, e saiu {:?}",
        shape.tint[0]
    );
}

/// ⭐⭐ **NENHUMA BANDA INVADE A VIZINHA** — treze blocos numa tela, e a leitura depende de cada
/// um ficar no seu quadrante.
///
/// FALSIFICADO por qualquer mudança de `COL_STEP`/`ROW_Y`/`GAP`/`SPREAD`/`PIECE` que faça duas
/// caixas tocarem-se: é o «algum bloco invadir o vizinho» do anúncio, medido em vez de olhado.
#[test]
fn no_band_runs_into_its_neighbour() {
    let (mut state, sinks) = scene();
    let caixas: Vec<[f32; 4]> = bake(&mut state, &sinks)
        .iter()
        .map(|b| {
            let meio = half_extent(b);
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
/// vive aqui, e afirma o mesmo.
#[test]
fn every_caption_is_a_chip_over_its_own_band() {
    /// O mesmo número do gate da casa: quantos caracteres cabem sem a ficha invadir a vizinha.
    const MAX_CHARS: usize = 40;
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
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
        let at = quadrant(k);
        assert!(
            (c.world[0] - at[0]).abs() < 1e-4,
            "banda {}: fora do x",
            k + 1
        );
        let topo = bandas[k].p.iter().fold(f32::MIN, |m, p| m.max(p[1])) + half_extent(&bandas[k]);
        assert!(
            c.world[1] > topo,
            "banda {}: a ficha caiu sobre as pecas ({} contra {topo})",
            k + 1,
            c.world[1]
        );
    }
    assert!(
        caps.iter().any(|c| c.text.chars().count() > MAX_CHARS / 2),
        "as fichas dizem alguma coisa"
    );
    for (k, c) in caps.iter().enumerate() {
        for (j, b) in bandas.iter().enumerate() {
            if quadrant(j)[1] <= quadrant(k)[1] {
                continue; // só as fileiras ACIMA desta
            }
            let baixo = b.p.iter().fold(f32::MAX, |m, p| m.min(p[1])) - half_extent(b);
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
/// conjunto e o tamanho de cada ilha do grafo.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib measure_the_dup_scene -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn measure_the_dup_scene() {
    let (mut state, sinks) = scene();
    let bandas = bake(&mut state, &sinks);
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    eprintln!("\n  banda | pecas |            caixa            | rotulo");
    for ((k, label), b) in band_labels().zip(&bandas) {
        let meio = half_extent(b);
        let a0 = b.p.iter().fold(f32::MAX, |m, p| m.min(p[0] - meio));
        let b0 = b.p.iter().fold(f32::MAX, |m, p| m.min(p[1] - meio));
        let a1 = b.p.iter().fold(f32::MIN, |m, p| m.max(p[0] + meio));
        let b1 = b.p.iter().fold(f32::MIN, |m, p| m.max(p[1] + meio));
        (x0, y0, x1, y1) = (x0.min(a0), y0.min(b0), x1.max(a1), y1.max(b1));
        eprintln!(
            "   {:>2}   |  {:>3}  | {a0:>6.2} {b0:>6.2} {a1:>6.2} {b1:>6.2} | {}",
            k + 1,
            b.p.len(),
            short_of(label)
        );
    }
    eprintln!(
        "\n  a cena inteira: x {x0:.2}..{x1:.2} ({:.2} de largura) · y {y0:.2}..{y1:.2} ({:.2} de altura)",
        x1 - x0,
        y1 - y0
    );
    let total = state.doc.graph.nodes().len();
    eprintln!(
        "  o grafo: {total} nos em {BANDS} cadeias fechadas ({:.1} nos por cadeia, em media)\n",
        total as f32 / BANDS as f32
    );
}

/// **QUE SEMENTE FAZ O `RANDOM` MOSTRAR AS TRÊS FORMAS** — a sonda que escolheu o `PICK_SEED`.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib which_seed_shows_every_shape -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de calibracao, nao um gate"]
fn which_seed_shows_every_shape() {
    for seed in 0..24u32 {
        let (mut state, sinks) = scene();
        let alvo = state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.duplicator")
            .nth(5)
            .map(|n| n.id)
            .expect("a sexta banda");
        state.doc.graph.set_param(alvo, "seed", seed as f32);
        let b = &bake(&mut state, &sinks)[5];
        let seq: Vec<i64> = b.geom.iter().map(|g| *g as i64).collect();
        let mut distintas = seq.clone();
        distintas.sort_unstable();
        distintas.dedup();
        eprintln!(
            "  seed {seed:>2}: {seq:?}  ({} formas distintas)",
            distintas.len()
        );
    }
}
