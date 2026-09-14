//! **AS FIGURAS E A TABELA do tutorial do ciclo 5** (a simulação — doc 108, passo 6).
//!
//! ⚠️ **Elas não são desenhos:** cada ponto é uma peça de verdade, na posição que o motor lhe deu
//! ao correr a cena `=113` do produto. É a mesma lei dos ciclos anteriores, e aqui ela vale
//! dobrado — *uma figura desenhada à mão de uma simulação é uma ilustração de uma teoria, não a
//! saída dela*.
//!
//! ⛔⛔ **A ESCRITA VEM DEPOIS DAS ASSERÇÕES.** No ciclo 4 a 1.ª redacção escrevia dentro do laço
//! de medição, e uma corrida VERMELHA deixou duas figuras idênticas no disco — o tutorial passou a
//! mostrar a mesma imagem debaixo de duas legendas diferentes, e só um `md5sum` à mão as apanhou.

use super::tutorial_draw as draw;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Os instantes em que a fila é fotografada. **Seis, igualmente espaçados** — a mensagem da
/// figura é o VÃO entre eles, e um vão só não tem com que ser comparado.
const AMOSTRAS: usize = 6;
/// Até quando. Antes de a primeira peça bater no bloco, de propósito: estas figuras comparam a
/// QUEDA, e uma peça já pousada mede o colisor em vez do modo da força.
const ATE: f64 = 1.0;
/// Quantas peças ADJACENTES da fila entram na figura — ver o comentário no [`colher`].
const COLUNAS: usize = 3;

/// Corre a cena `=113` com o `Acts As` que se pedir e fotografa **uma fila** em [`AMOSTRAS`]
/// instantes.
///
/// ⚠️ **A cena vem da porta `monta`, não de um documento montado à mão** — uma fixtura que
/// reconstrói a cena mede outro programa que o que o dono abre.
///
/// ⚠️ **Uma FILA e não a nuvem inteira:** as cinco fileiras da cena partem de alturas
/// diferentes, então sobrepostas elas borram exactamente o vão que a figura existe para
/// mostrar. A fila escolhida é a de cima, derivada do `y` máximo do primeiro instante.
fn colher(modo: f32) -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let (sinks, _) = crate::motion_demo_legend::monta("113", &mut doc, &reg);
    let vento = doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "force.wind")
        .expect("a cena `=113` tem um `force.wind`")
        .id;
    doc.graph.set_param(vento, "mode", modo);

    let mut cook = Cook::new();
    let last = (ATE * 60.0) as u64;
    let passo = last / (AMOSTRAS as u64 - 1);
    let mut fila: Option<Vec<usize>> = None;
    let mut fotos: Vec<Vec<[f32; 2]>> = Vec::new();
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(v)) = s.get("P") {
            let quais = fila.get_or_insert_with(|| {
                let topo = v.iter().fold(f32::MIN, |m, p| m.max(p[1]));
                let mut linha: Vec<usize> = v
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| (p[1] - topo).abs() < 1e-3)
                    .map(|(i, _)| i)
                    .collect();
                // ⚠️ **Só [`COLUNAS`] peças ADJACENTES do meio da fila, e a razão é a moldura.**
                // A fila inteira tem doze peças sobre `~4,6` unidades de largura contra `~2,5` de
                // queda: o quadro sai deitado, a figura é desenhada a `340 px` de largo e o vão
                // vertical — que É a mensagem — fica com meia dúzia de pixels. Recortar ao meio
                // deixa o quadro em pé e dá ao espaçamento a altura toda.
                let meio = linha.len() / 2;
                let ini = meio.saturating_sub(COLUNAS / 2);
                linha = linha[ini..(ini + COLUNAS).min(linha.len())].to_vec();
                linha
            });
            if k % passo == 0 && fotos.len() < AMOSTRAS {
                fotos.push(quais.iter().map(|&i| v[i]).collect());
            }
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    fotos
}

/// O vão vertical médio entre duas fotos consecutivas.
fn vaos(fotos: &[Vec<[f32; 2]>]) -> Vec<f32> {
    fotos
        .windows(2)
        .map(|w| {
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de pecas")]
            let n = w[0].len() as f32;
            w[0].iter()
                .zip(&w[1])
                .map(|(a, b)| a[1] - b[1])
                .sum::<f32>()
                / n.max(1.0)
        })
        .collect()
}

/// ⭐⭐⭐ **AS DUAS FIGURAS DA QUEDA — e a régua é como o vão CRESCE, não o vão.**
///
/// ⛔⛔ **Duas redacções desta sonda tiveram a régua errada, e a segunda foi apanhada por ela
/// própria.**
///
/// A 1.ª comparava a queda **total**: `2,458` contra `1,824` unidades — uma diferença real e
/// **12 % da moldura**, que saiu como duas imagens que o olho lê como a mesma, com uma legenda a
/// dizer *«acelera»* e outra *«satura»* por cima delas. *Uma figura que precisa da legenda para
/// se distinguir da irmã não é uma figura, é uma afirmação.*
///
/// A 2.ª fotografou a fila seis vezes e exigiu que sob `Target Velocity` os **vãos fossem
/// iguais** — e reprovou, com razão: com a `Air Resistance` no valor da cena (`1`) a constante de
/// tempo é **um segundo**, e o segundo que a queda dura antes de bater no bloco não chega para os
/// vãos assentarem (medido: `6,83×` do primeiro ao último). ⛔ **A cura NÃO foi subir a
/// resistência até a figura ficar bonita** — isso mediria um knob que a cena não tem.
///
/// ⭐ **A grandeza que separa os dois modos em QUALQUER janela é a segunda diferença.** Com
/// aceleração constante o vão cresce sempre pela mesma quantidade (o último acréscimo sobre o
/// primeiro é **1**); com saturação cada acréscimo é menor que o anterior, e a razão **desce**.
/// É isso que a figura mostra e é isso que a legenda manda olhar — *os vãos crescem sempre
/// igual* contra *os vãos crescem cada vez menos*.
///
/// ⚠️ **As barras saem da física, não de gosto**, e o vale medido nesta cena é largo:
/// **`1,00×`** sob `Force` contra **`0,55×`** sob `Target Velocity`.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib -- --ignored --nocapture write_the_sim_figures
/// ```
#[test]
#[ignore = "escreve ficheiros — corra à mão"]
fn write_the_sim_figures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    std::fs::create_dir_all(&dir).expect("a pasta");

    let forca = colher(0.0);
    let alvo = colher(1.0);
    let (vf, va) = (vaos(&forca), vaos(&alvo));
    // Como o vão CRESCE: o último acréscimo sobre o primeiro. Ver o doc acima.
    let crescimento = |v: &[f32]| -> f32 {
        let d: Vec<f32> = v.windows(2).map(|w| w[1] - w[0]).collect();
        d.last().copied().unwrap_or(0.0) / d.first().copied().unwrap_or(1e-6)
    };
    let (rf, ra) = (crescimento(&vf), crescimento(&va));
    eprintln!(
        "
  vãos Force            │ {vf:?}
  vãos Target Velocity  │ {va:?}"
    );
    eprintln!("  último/1.º acréscimo  │ Force {rf:.2}×  ·  Target Velocity {ra:.2}×");

    assert_eq!(forca.len(), AMOSTRAS, "seis fotos sob `Force`");
    assert_eq!(alvo.len(), AMOSTRAS, "seis fotos sob `Target Velocity`");
    assert!(
        !forca[0].is_empty(),
        "a fila de cima tem de ter pecas -- o filtro do `y` maximo nao apanhou nenhuma"
    );
    assert!(
        (rf - 1.0).abs() < 0.15,
        "sob `Force` o vão cresce {rf:.2}× -- com aceleração constante ele tem de crescer sempre \
         pela MESMA quantidade (razão 1), e se não cresce assim a figura da esquerda não mostra \
         aceleração nenhuma"
    );
    assert!(
        ra < 0.75,
        "sob `Target Velocity` o vão cresce {ra:.2}× -- a saturação tem de fazer cada acréscimo \
         ser CLARAMENTE menor que o anterior, senão as duas figuras contam a mesma história"
    );

    // A escrita, só agora — ⛔ ver o cabeçalho do módulo.
    let todas: Vec<&[[f32; 2]]> = forca
        .iter()
        .chain(&alvo)
        .map(std::vec::Vec::as_slice)
        .collect();
    let quadro = draw::moldura(&todas);
    for (ficheiro, fotos) in [("sim_force", &forca), ("sim_target", &alvo)] {
        let fantasma: Vec<[f32; 2]> = fotos[..AMOSTRAS - 1].concat();
        let svg = draw::svg(&fotos[AMOSTRAS - 1], &fantasma, quadro, false);
        std::fs::write(dir.join(format!("{ficheiro}.svg")), svg).expect("escrever");
        eprintln!("  figura                │ {ficheiro}.svg");
    }

    let path = dir.join("params_sim.html");
    // ⚠️ O cartão da FORMA entra com o `Collide` LIGADO — é o estado em que o capítulo 3 o põe, e
    // é onde a secção `Collision` existe. Ver [`tutorial_table::derive_ligado`].
    let tabela = super::tutorial_table::derive_ligado(
        TABELA,
        &[("shape", ph2d_node_motion_shape::param::COLLIDE, 1.0)],
    );
    std::fs::write(&path, tabela).expect("a tabela");
    eprintln!("  tabela derivada       │ {}", path.display());
}

/// As âncoras da tabela «o que cada controlo faz» — a mesma porta dos ciclos 1 a 4.
static TABELA: &[(&str, &str)] = &[
    // ⭐ A FORMA entra nesta tabela desde o doc 109: o colisor e o material moram no cartão dela,
    // e um tutorial de simulação que não os liste manda o artista procurá-los no nó errado.
    ("shape", "source.shape"),
    ("collider", "sim.collide"),
    ("wind", "force.wind"),
    ("zone", "sim.zone"),
    ("step", "sim.step"),
    ("attractor", "force.attractor"),
    ("vortex", "force.vortex"),
    ("curl", "force.curl"),
    ("drag", "force.drag"),
    ("buoyancy", "force.buoyancy"),
    ("spawn", "sim.spawn"),
    ("lifetime", "sim.lifetime"),
];
