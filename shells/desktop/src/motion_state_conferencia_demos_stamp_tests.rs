//! Gates da cena `=98` — o vocabulário do carimbo (folha 08).
//!
//! ⚠️ **Estes gates medem o que a cena DESENHA, não o que ela monta.** A lição é da cena
//! `=9` desta mesma linha: cinco gates verdes sobre uma cena em que os quatro pares saíam
//! iguais, porque todos mediam a montagem. Aqui cada par é cozido e a afirmação é sobre as
//! COLUNAS que saem — se um par sair igual dos dois lados, o gate reprova antes do Enio.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_stamp_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

struct Band {
    p: Vec<[f32; 2]>,
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
                tint: match st.get("tint") {
                    Some(Column::Vec4(v)) => v.clone(),
                    _ => Vec::new(),
                },
            }
        })
        .collect()
}

/// Quantas cores distintas há na banda (a `1e-4`, que é muito abaixo do passo da rampa).
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

/// A cena monta as seis bandas, e as seis cospem as `PIECES` peças.
#[test]
fn the_stamp_scene_builds_all_six_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 6, "tres pares");
    assert_eq!(band_labels().count(), 6, "um rotulo por banda");
    assert_eq!(captions().len(), 6, "uma ficha por banda");
    for (k, b) in bake(&doc, &reg, &sinks).into_iter().enumerate() {
        assert_eq!(b.p.len(), PIECES as usize, "banda {k}: a fila inteira");
        assert_eq!(
            b.tint.len(),
            PIECES as usize,
            "banda {k}: toda peca tem cor"
        );
    }
}

/// ⭐⭐ **O PRIMEIRO PAR: a cor do arranjo SOME à esquerda e CHEGA à direita.** É o defeito
/// que a folha 08 escondia sob *"modos de transferência de atributo"*, e é a razão desta
/// cena existir.
#[test]
fn the_first_pair_loses_the_arrangements_colour_on_the_left_and_keeps_it_on_the_right() {
    let (doc, reg, sinks) = scene();
    let b = bake(&doc, &reg, &sinks);
    assert_eq!(
        distinct_tints(&b[0]),
        1,
        "`Shape Wins`: as 16 copias tem de sair TODAS com a cor da forma"
    );
    assert!(
        distinct_tints(&b[1]) > 8,
        "`Point Wins`: a rampa tem de chegar (saiu com {} cores)",
        distinct_tints(&b[1])
    );
}

/// ⭐ **O SEGUNDO PAR: `Multiply` não é `Point Wins`.** Um par que saísse igual seria um
/// controlo sem diferença — e o `Multiply` tem de ESCURECER, porque a cor da forma é um
/// cinzento e o neutro daquele modo é o branco.
#[test]
fn the_second_pair_tints_the_ramp_instead_of_replacing_it() {
    let (doc, reg, sinks) = scene();
    let b = bake(&doc, &reg, &sinks);
    let sum = |x: &Band| -> f32 {
        x.tint.iter().map(|t| t[0] + t[1] + t[2]).sum::<f32>() / x.tint.len() as f32
    };
    assert!(
        sum(&b[3]) < sum(&b[2]) - 1e-3,
        "`Multiply` ({:.4}) tem de ser mais escuro que `Point Wins` ({:.4})",
        sum(&b[3]),
        sum(&b[2])
    );
    assert!(
        distinct_tints(&b[3]) > 8,
        "e continua a ser uma RAMPA, nao uma cor so'"
    );
}

/// ⚠️ **O CONTROLO do 2.º par:** as duas bandas `Point Wins` (a da direita do 1.º par e a da
/// esquerda do 2.º) são a MESMA cena e têm de sair iguais. Sem isto, uma diferença de
/// montagem passaria por uma diferença de lei.
#[test]
fn the_two_point_wins_bands_are_the_same_scene() {
    let (doc, reg, sinks) = scene();
    let b = bake(&doc, &reg, &sinks);
    let (l, r) = (&b[1].tint, &b[2].tint);
    assert_eq!(l.len(), r.len());
    for (i, (a, c)) in l.iter().zip(r).enumerate() {
        assert!(
            a.iter().zip(c).all(|(x, y)| (x - y).abs() < 1e-6),
            "peca {i}: {a:?} contra {c:?} -- as duas bandas `Point Wins` divergiram"
        );
    }
}

/// ⭐⭐ **O TERCEIRO PAR: o baralhamento global espalha e o por-grupos NÃO.**
///
/// ⚠️ **A 1.ª versão deste gate media a COR, e a régua era minha:** ela ordenava as peças
/// pelo canal VERMELHO da rampa para inferir o posto — e o gradiente por omissão não é
/// monótono no vermelho, então «o quarto de onde a cor vem» era ruído. Deu `5 de 16` com o
/// produto certo. *Uma régua derivada da apresentação mede a apresentação.*
///
/// A régua a sério é a ORDEM, e ela tem uma coluna: o `motion.sort` renumera o `Index` para
/// o posto. A cor existe para o olho do Enio; o gate lê o posto.
#[test]
fn the_third_pair_confines_the_shuffle_to_each_group() {
    let (doc, reg, sinks) = scene();
    let mut cook = Cook::new();
    let rank_and_place = |sink: NodeId, cook: &mut Cook| -> Vec<(usize, usize)> {
        let out = cook.cook(&doc.graph, &reg, sink, 0.0).expect("coze");
        let st = out[0].as_stream();
        let p = match st.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let rank = match st.get("Index") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        assert_eq!(
            rank.len(),
            p.len(),
            "o sort renumera o `Index` para o posto"
        );
        let mut by_x: Vec<usize> = (0..p.len()).collect();
        by_x.sort_by(|&a, &b| p[a][0].total_cmp(&p[b][0]));
        let per = (PIECES / GROUPS) as usize;
        (0..p.len())
            .map(|i| {
                let place = by_x.iter().position(|&j| j == i).unwrap() / per;
                (place, (rank[i] as usize) / per)
            })
            .collect()
    };
    let global = rank_and_place(sinks[4], &mut cook);
    let grouped = rank_and_place(sinks[5], &mut cook);
    let agree = |v: &[(usize, usize)]| v.iter().filter(|(a, b)| a == b).count();
    assert_eq!(
        agree(&grouped),
        PIECES as usize,
        "com grupos, TODA peca tem de ficar com um posto do quarto DELA (deu {} de {})",
        agree(&grouped),
        PIECES as usize
    );
    assert!(
        agree(&global) < PIECES as usize,
        "sem grupos o baralhamento e' global -- se coincidisse em todas, a semente \
         escolheu a identidade e a cena nao prova nada"
    );
    // ⚠️ E a metade que o olho vê: a rampa TEM de estar lá, senão a cena é ilegível.
    let b = bake(&doc, &reg, &sinks);
    assert!(
        distinct_tints(&b[5]) > 8,
        "a fileira agrupada continua a ser uma rampa (saiu com {} cores)",
        distinct_tints(&b[5])
    );
}

/// Nenhuma banda sai do quadrante dela — a lei da cena `=73`, herdada.
#[test]
fn no_band_leaves_its_slot() {
    let (doc, reg, sinks) = scene();
    let want = (PIECES - 1.0) * GAP;
    for (k, b) in bake(&doc, &reg, &sinks).into_iter().enumerate() {
        let lo = b.p.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
        let hi = b.p.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
        assert!(
            (hi - lo - want).abs() < 1e-3,
            "banda {k}: a fila mede {:.4} e a caixa autorada e' {want:.4}",
            hi - lo
        );
        let at = quadrant(k);
        let mid = (lo + hi) * 0.5;
        assert!(
            (mid - at[0]).abs() < 1e-3,
            "banda {k}: o centro em x e' {mid:.4} e devia ser {:.4}",
            at[0]
        );
        let y = b.p[0][1];
        assert!(
            (y - at[1]).abs() < 1e-3,
            "banda {k}: a fileira esta' em y = {y:.4} e devia estar em {:.4}",
            at[1]
        );
    }
}

/// ⚠️ **Os NÚMEROS que a mensagem cita vivem num `const`, e este gate lê o fonte da
/// narração.** É a lei do ritual desta conferência: uma mensagem que diz *"carimbada 16
/// vezes"* enquanto o `const` diz outra coisa manda o Enio procurar um defeito que não
/// existe — e é o género de divergência que nenhum teste de comportamento apanha, porque a
/// prosa não é executada.
#[test]
fn the_announcement_cites_the_numbers_the_scene_actually_uses() {
    let src = include_str!("motion_state_demo_conferencia_stamp.rs");
    let pieces = format!("{} vezes", PIECES as usize);
    assert!(
        src.contains(&pieces),
        "o anuncio tem de dizer «{pieces}» -- o `const` PIECES e' {PIECES}"
    );
    assert!(
        src.contains("quatro faixas"),
        "o anuncio promete as faixas dos grupos"
    );
    assert_eq!(GROUPS, 4.0, "…e sao QUATRO porque o `const` GROUPS o diz");
    // E o mesmo para os rótulos das bandas, que o anúncio imprime a seguir.
    let labels: Vec<&str> = band_labels().map(|(_, l)| l).collect();
    assert!(
        labels[0].contains(&format!("{} copias", PIECES as usize)),
        "o rotulo da 1.a banda cita a contagem: {}",
        labels[0]
    );
}
