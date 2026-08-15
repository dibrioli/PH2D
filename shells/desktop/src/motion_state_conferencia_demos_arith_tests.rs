//! Gates da cena `=41` — **a ARITMÉTICA do domínio de valor** (doc 89, grupo A).
//!
//! ⚠️ A cena é um gráfico de dez perfis, e um gráfico tem TRÊS modos de falhar em
//! silêncio, não um: uma fileira **chata** (a cadeia não produziu nada), duas
//! fileiras **iguais** (o param de modo não chegou ao kernel) e duas fileiras que
//! **se cruzam** (a amplitude passou da distância entre linhas, e os dois gráficos
//! viraram um borrão). Os gates abaixo afirmam os três — e o segundo é o que
//! importa, porque é a falha que um `if/else if` de WGSL produz quando o ramo novo
//! não é alcançado.
//!
//! ⚠️ **A ordem da tabela É a ordem na tela**, e um gate depende disso: os pares
//! que têm de diferir são VIZINHOS. Reordenar `ROWS_TABLE` sem reordenar
//! `MUST_DIFFER` deixaria os gates a comparar coisas que não são par.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// O perfil de uma fileira: o `y` de cada peça, MENOS a linha de base dela — é o
/// valor que o `motion.drive` somou, isolado do lugar onde a fileira foi posta.
fn profile(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    let ys: Vec<f32> = match Stream::get(s, "P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => Vec::new(),
    };
    // A grade tem UMA linha, então o `y` da grade é constante e o piso é a linha
    // de base: subtraí-lo deixa só a excursão do valor.
    let base = ys.iter().copied().fold(f32::INFINITY, f32::min);
    ys.into_iter().map(|y| y - base).collect()
}

fn excursion(p: &[f32]) -> f32 {
    p.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

/// Os pares que TÊM de desenhar coisas diferentes, por índice de fileira. Cada um
/// é *o mesmo nó, a mesma entrada, outro modo* — e cada um carrega a **sua**
/// barra.
///
/// ⚠️ **A barra é por-par porque uma barra única nasceu INATINGÍVEL, e o gate
/// reprovou código correto.** A primeira versão pedia `0,05` de todos; o par
/// cúbica × quíntica mediu **4,29e-2** e ficou VERMELHO. O número não é ruído: a
/// separação máxima entre `3t²−2t³` e `6t⁵−15t⁴+10t³` sobre `[0,1]` é **0,0527**,
/// no quarto de caminho, e a fileira a escala por `0,8` ⇒ o teto FÍSICO daquele
/// par é `0,042`. Uma barra de `0,05` exigia mais do que a matemática permite.
///
/// *Uma barra que código correto não consegue satisfazer não é rigor, é um gate
/// partido* — e o doc do módulo já dizia que aquele par é o que o olho não
/// separa. A barra dele é derivada daquele 0,0527, não escolhida.
const MUST_DIFFER: &[(usize, usize, f32, &str)] = &[
    (0, 1, 0.05, "os dois modulos"),
    (2, 3, 0.05, "Floor contra Truncate"),
    (4, 5, 0.05, "a rampa reta contra a escada"),
    (4, 6, 0.05, "a rampa reta contra o S"),
    (5, 6, 0.05, "a escada contra o S"),
    (7, 8, 0.03, "a banda cubica contra a quintica"),
];

/// **A cena constrói as dez fileiras.** Se um `wire` falhasse, o roteador cairia
/// no `unwrap_or_default()` — uma tela VAZIA, que num smoke lê como *"a feature
/// não foi construída"* em vez de *"a cena está partida"*.
#[test]
fn the_arithmetic_scene_builds_every_row() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_arith_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS, "uma fileira por linha da tabela");
    assert_eq!(
        ROWS_TABLE.len(),
        ROWS,
        "a constante e a tabela dizem o mesmo numero"
    );
}

/// **Nenhuma fileira é CHATA.** Um perfil sem excursão é uma cadeia que não
/// produziu nada — e ele concordaria com qualquer outro perfil chato, o que faria
/// o gate de distinção abaixo passar por vácuo.
#[test]
fn every_row_draws_something() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_arith_demo_document(&mut doc, &reg).expect("a cena constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let p = profile(&doc, &reg, *sink);
        assert_eq!(p.len(), COLS as usize, "fileira {k}: contagem de pecas");
        let e = excursion(&p);
        assert!(
            e > 0.2,
            "fileira {k} ({}) e' chata: excursao {e:e}",
            ROWS_TABLE[k].label
        );
    }
}

/// **Cada modo NOVO desenha coisa DIFERENTE do vizinho** — o oráculo da cena,
/// medido no stream que o render de facto consome.
///
/// ⚠️ Sem este gate a cena passaria com um kernel que ignorasse o param de modo:
/// as dez fileiras apareceriam, todas com excursão, e o olho num smoke rápido
/// veria "dez gráficos". A pergunta é *dez gráficos DIFERENTES*.
#[test]
fn each_new_mode_draws_a_different_profile_from_its_control() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_arith_demo_document(&mut doc, &reg).expect("a cena constroi");
    let profiles: Vec<Vec<f32>> = sinks.iter().map(|s| profile(&doc, &reg, *s)).collect();
    for &(a, b, bar, what) in MUST_DIFFER {
        let d = profiles[a]
            .iter()
            .zip(&profiles[b])
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        assert!(
            d > bar,
            "{what} (fileiras {a} e {b}) desenham o MESMO perfil: max |d| = {d:e} \
             contra a barra {bar:e}"
        );
    }
}

/// **A assinatura do `Truncate`: um degrau de largura DUPLA sobre a origem.** É a
/// leitura que a cena promete no anúncio, e a que distingue o modo novo do Floor
/// pelo OLHO — os dois lados do eixo colapsam para o mesmo nível.
///
/// ⚠️ Este gate é mais afiado que o de distinção acima: dois perfis podem diferir
/// por qualquer razão, e este diz **onde** e **como**.
#[test]
fn the_truncate_row_has_a_double_width_tread_over_the_origin() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_arith_demo_document(&mut doc, &reg).expect("a cena constroi");
    let floor = profile(&doc, &reg, sinks[2]);
    let trunc = profile(&doc, &reg, sinks[3]);

    // Quantos degraus distintos cada escada visita. O `Truncate` funde os dois que
    // tocam a origem num só, entao ele tem exactamente UM nivel a menos.
    let levels = |p: &[f32]| {
        let mut seen: Vec<f32> = Vec::new();
        for v in p {
            if !seen.iter().any(|s| (s - v).abs() < 1e-4) {
                seen.push(*v);
            }
        }
        seen.len()
    };
    let (lf, lt) = (levels(&floor), levels(&trunc));
    assert_eq!(
        lt,
        lf - 1,
        "o Truncate funde os dois degraus da origem: Floor {lf} niveis, Truncate {lt}"
    );
}

/// **Duas fileiras vizinhas NÃO se cruzam.** A amplitude por-fileira existe
/// exactamente por isto (a coluna `scale` da tabela), e sem o gate a próxima
/// fileira com alcance maior sobreporia a de baixo — dois gráficos que se cruzam
/// deixam de ser dois gráficos, e o smoke passaria a julgar um borrão.
#[test]
fn no_row_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_arith_demo_document(&mut doc, &reg).expect("a cena constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let e = excursion(&profile(&doc, &reg, *sink));
        assert!(
            e < ROW_GAP,
            "fileira {k} ({}) sobe {e:e}, mais que o vao de {ROW_GAP:e}",
            ROWS_TABLE[k].label
        );
    }
}

/// **A cena inteira é reivindicada pelo DEVICE.** Os cinco nós que este grupo
/// tocou têm kernel, e o valor de os ter é o sequenciador não cair para a CPU —
/// uma fileira que cozesse na CPU desenharia a mesma imagem e a cena não diria
/// nada sobre o WGSL que os gates de paridade medem.
///
/// ⚠️ `plan` é headless: ele responde *quem reivindica o quê* sem adapter, então
/// este gate corre na suíte normal em vez de ficar `#[ignore]`.
#[test]
fn every_row_is_claimed_end_to_end_by_the_device() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_arith_demo_document(&mut doc, &reg).expect("a cena constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let plan = ph2d_gpu_cook::plan(&doc.graph, &reg, &reg, *sink);
        assert!(
            plan.is_fully_gpu(),
            "fileira {k} ({}) cai para a CPU",
            ROWS_TABLE[k].label
        );
    }
}

/// **O anúncio descreve as fileiras que existem.** O roteador imprime uma linha
/// por fileira a partir da MESMA tabela que as constrói — este gate é o que
/// impede o texto de descrever uma cena que a tabela deixou de montar.
#[test]
fn the_announcement_names_every_row_that_is_built() {
    let labels: Vec<_> = row_labels().collect();
    assert_eq!(labels.len(), ROWS);
    for (i, l) in labels {
        assert!(!l.is_empty(), "fileira {i} sem rotulo");
    }
}
