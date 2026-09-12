//! Gates da cena `=49` — **A VIZINHANÇA VIRA UM NÚMERO**.
//!
//! Eles medem a geometria que a cena COZINHA, não a intenção com que ela foi
//! escrita.
//!
//! ⚠️ **A régua de um empacotamento é o VÃO entre vizinhos, nunca o Y** — cada
//! fileira carrega o próprio `offset_y`, e comparar Y entre bandas mede o
//! `BAND_GAP`. É a quinta vez que este repo paga essa lição (o `1,15` do grupo B,
//! o `offset_y` do Range no C, os gêmeos do D, os vãos do H).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Uma banda cozida: as peças ordenadas por `x`, cada uma com a meia-largura que
/// o renderer desenha — a MESMA grandeza que o `motion.proximity` lê como raio.
struct Band {
    xs: Vec<f32>,
    half: Vec<f32>,
}

impl Band {
    fn len(&self) -> usize {
        self.xs.len()
    }

    /// O vão de SUPERFÍCIE entre as peças `k` e `k+1`: negativo ⇒ elas se
    /// sobrepõem. É a régua da cena, e ela é sobre os DISCOS, não sobre os centros.
    fn surface_gaps(&self) -> Vec<f32> {
        (0..self.len().saturating_sub(1))
            .map(|k| (self.xs[k + 1] - self.xs[k]) - RADIUS * (self.half[k] + self.half[k + 1]))
            .collect()
    }
}

fn cook_band(lane: usize) -> Band {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_proximity_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BANDS, "uma fileira por banda");

    let mut cook = Cook::new();
    let s = cook.cook(&doc.graph, &reg, sinks[lane], 0.0).expect("cook")[0]
        .as_stream()
        .clone();
    let n = s.count();
    let p = match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a banda {lane} nao emite P"),
    };
    let half: Vec<f32> = match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|e| e[0].abs().max(e[1].abs())).collect(),
        _ => vec![1.0; n],
    };
    // Ordena por x carregando o tamanho junto — a lei é sobre VIZINHOS na fileira,
    // e nada garante que o índice siga a ordem espacial depois de um cull.
    let mut pairs: Vec<(f32, f32)> = p.iter().map(|q| q[0]).zip(half).collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    Band {
        xs: pairs.iter().map(|e| e.0).collect(),
        half: pairs.iter().map(|e| e.1).collect(),
    }
}

/// **A CENA MONTA AS SEIS BANDAS QUE O LOG NOMEIA.**
///
/// ⚠️ A contagem é DERIVADA (`BAND_LABELS.len()`), nunca um literal: um número
/// escrito à mão só sabe dizer *"mudou"*, e a pergunta é se a lista que o artista
/// lê corresponde ao que a cena de facto empilha.
#[test]
fn the_scene_builds_every_band_the_log_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_proximity_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BAND_LABELS.len());
    assert_eq!(sinks.len(), BANDS);
}

/// **TODA BANDA ACABA NUM `motion.output` — senão a cena não desenha NADA.**
///
/// ⚠️ O laço de render **não usa** a lista que o construtor devolve: ele re-resolve
/// os sinks a cada quadro a partir dos nós de saída do grafo. Uma cadeia terminada
/// no `motion.cull` cozinha certo, satisfaz todos os gates de geometria e **não
/// pinta um pixel** — foi o defeito que a cena `=48` shipou.
#[test]
fn every_band_ends_in_an_output_node() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_proximity_demo_document(&mut doc, &reg).expect("a cena monta");
    for (i, &sink) in sinks.iter().enumerate() {
        assert_eq!(
            doc.graph.node(sink).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "a banda {i} nao termina num motion.output"
        );
    }
}

/// **A FILEIRA DE CONTROLE CONTÉM AS DUAS ESPÉCIES.**
///
/// Sem isto nenhuma das três leituras da cena significa coisa alguma: o Hide sobre
/// um amontoado uniforme apaga a cena inteira, e *"apaga quem se sobrepõe"* fica
/// indistinguível de *"apaga tudo"*.
#[test]
fn the_control_row_holds_both_species() {
    let ctl = cook_band(0);
    let g = ctl.surface_gaps();
    let free = g.iter().filter(|&&d| d > 0.0).count();
    let touching = g.iter().filter(|&&d| d < 0.0).count();
    assert!(
        free >= 3 && touching >= 6,
        "a fileira precisa das duas especies: {free} pares livres, {touching} sobrepostos ({g:?})"
    );
}

/// **SCALE: as sobrepostas encolhem até apenas se tocar; as livres NÃO são
/// tocadas.**
///
/// ⚠️ As duas metades são o gate — sem a segunda, *"encolheu tudo"* passaria, e é
/// exactamente isso que separa a composição de um `motion.scale` global.
#[test]
fn the_scale_mode_shrinks_only_what_overlaps() {
    let ctl = cook_band(0);
    let out = cook_band(1);
    assert_eq!(ctl.len(), out.len(), "o Scale nao muda a contagem");

    let worst_before = ctl.surface_gaps().into_iter().fold(f32::INFINITY, f32::min);
    let worst_after = out.surface_gaps().into_iter().fold(f32::INFINITY, f32::min);
    assert!(
        worst_before < -0.1,
        "o controle precisa sobrepor de verdade (pior vao {worst_before:.4})"
    );
    assert!(
        worst_after > worst_before + 0.08,
        "o Scale tem de abrir o pior vao: {worst_before:.4} -> {worst_after:.4}"
    );

    // A metade que impede *"encolheu tudo"*: as peças que já estavam livres saem
    // com o MESMO tamanho, ao bit.
    let free_ctl = ctl.surface_gaps();
    let mut untouched = 0usize;
    for k in 0..ctl.len() {
        let left_free = k == 0 || free_ctl[k - 1] > 0.0;
        let right_free = k + 1 == ctl.len() || free_ctl[k] > 0.0;
        if left_free && right_free {
            assert_eq!(
                out.half[k], ctl.half[k],
                "a peca livre {k} foi encolhida sem precisar"
            );
            untouched += 1;
        }
    }
    assert!(
        untouched >= 3,
        "o gate precisa de pecas livres para vigiar (achou {untouched})"
    );
}

/// **HIDE: quem se sobrepõe SAI, e quem estava livre FICA.**
#[test]
fn the_hide_mode_deletes_exactly_the_crowd() {
    let ctl = cook_band(2);
    let out = cook_band(3);
    let g = ctl.surface_gaps();
    let expected: Vec<f32> = (0..ctl.len())
        .filter(|&k| {
            let left_free = k == 0 || g[k - 1] > 0.0;
            let right_free = k + 1 == ctl.len() || g[k] > 0.0;
            left_free && right_free
        })
        .map(|k| ctl.xs[k])
        .collect();

    assert!(
        !expected.is_empty() && expected.len() < ctl.len(),
        "a fixture nao contem o fenomeno: {} de {} sobreviveriam",
        expected.len(),
        ctl.len()
    );
    assert_eq!(
        out.len(),
        expected.len(),
        "o Hide devia manter {} de {} pecas, manteve {}",
        expected.len(),
        ctl.len(),
        out.len()
    );
    for (k, (&got, &want)) in out.xs.iter().zip(expected.iter()).enumerate() {
        assert!(
            (got - want).abs() < 1e-4,
            "a peca {k} que sobrou nao e' a que estava livre: {got} vs {want}"
        );
    }
}

/// **A CONTAGEM: o tamanho vira um medidor — livre pequeno, amontoado inchado.**
///
/// ⚠️ O oráculo NÃO é *"as duas bandas diferem"* (a banda 6 tem outra lei de
/// tamanho, então diferiria de qualquer jeito): é que o tamanho da banda 6
/// **CRESCE com o número de vizinhos que a banda 5 tem naquele lugar**.
#[test]
fn the_count_mode_turns_size_into_a_neighbour_meter() {
    let ctl = cook_band(4);
    let out = cook_band(5);
    assert_eq!(ctl.len(), out.len(), "a contagem nao muda a contagem");

    // A vizinhança que a banda de controle de facto tem, computada aqui pela lei
    // do nó (`d < r_i + r_j`) — um oráculo independente do kernel.
    let count: Vec<usize> = (0..ctl.len())
        .map(|i| {
            (0..ctl.len())
                .filter(|&j| {
                    j != i
                        && (ctl.xs[j] - ctl.xs[i]).abs()
                            < RADIUS * (ctl.half[i] + ctl.half[j]) - 1e-6
                })
                .count()
        })
        .collect();

    let free = (0..ctl.len())
        .find(|&i| count[i] == 0)
        .expect("uma peca livre");
    let crowded = (0..ctl.len())
        .max_by_key(|&i| count[i])
        .expect("uma peca amontoada");
    assert!(
        count[crowded] >= 2,
        "a fixture precisa de alguem com 2+ vizinhos (max {})",
        count[crowded]
    );
    assert!(
        out.half[crowded] > out.half[free] + 0.2,
        "o medidor nao inchou onde ha vizinhos: livre {:.4} vs amontoado {:.4}",
        out.half[free],
        out.half[crowded]
    );
    // E o piso do medidor é o que faz a peça livre continuar VISÍVEL.
    assert!(
        out.half[free] > 0.0,
        "a peca livre sumiu: {:.4}",
        out.half[free]
    );

    // ⚠️ A metade que torna o gate um ORÁCULO e não um *"as duas diferem"*: o
    // tamanho de CADA peça é a contagem dela mapeada pela faixa que a cena autora.
    // A contagem vem da lei do disco computada aqui, e o mapeamento das constantes
    // da cena — a cadeia inteira (kernel → attribute → map_range → drive) é o que
    // está sob teste, não a aritmética do `map_range`.
    for (i, (&n, &got)) in count.iter().zip(out.half.iter()).enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let want =
            COUNT_SIZE_LO + (COUNT_SIZE_HI - COUNT_SIZE_LO) * (n as f32 / COUNT_HI).clamp(0.0, 1.0);
        assert!(
            (got - want).abs() < 1e-4,
            "a peca {i} toca {n} vizinhos e devia medir {want:.4}, mede {got:.4}"
        );
    }
}

/// A sonda que produziu os números da mensagem de smoke. Não afirma nada: ela
/// IMPRIME, e o que ela imprime é o que a mensagem cita.
#[test]
#[ignore = "sonda: imprime os numeros da mensagem de smoke"]
fn probe_the_scene_numbers() {
    for lane in 0..BANDS {
        let b = cook_band(lane);
        let g = b.surface_gaps();
        let worst = g.iter().copied().fold(f32::INFINITY, f32::min);
        let lo = b.half.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = b.half.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let span = b.xs.last().unwrap_or(&0.0) - b.xs.first().unwrap_or(&0.0);
        eprintln!(
            "banda {lane}: n={} | meia-largura {lo:.4}..{hi:.4} | pior vao {worst:.4} | span {span:.4}",
            b.len()
        );
        eprintln!(
            "   vaos: {:?}",
            g.iter()
                .map(|d| (d * 1e4).round() / 1e4)
                .collect::<Vec<_>>()
        );
    }
}
