//! Gates do [`super::cut_closed`] (plano 25 §7).
//!
//! Os dois primeiros nasceram **VERMELHOS** contra o produto de 2026-07-30: ele abria a forma
//! (peça aberta) em vez de a partir em duas fechadas.

use super::{CutRefusal, cut_closed, cut_with_line};
use crate::area;
use ph2d_vec_scene::{VecPath, VecVertex};

/// Um losango FECHADO de arestas retas, centrado na origem, "raio" 2.
///
/// ⚠️ Retas de propósito: o `closed_square` da suíte vizinha tem HANDLES (as arestas dele são
/// curvas), e um oráculo de área sobre curvas mediria outra coisa que não o que estes gates
/// afirmam.
fn diamond() -> VecPath {
    VecPath {
        verts: [[-2.0, 0.0], [0.0, 2.0], [2.0, 0.0], [0.0, -2.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Uma linha ABERTA de `a` a `b`.
fn line(a: [f64; 2], b: [f64; 2]) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner(a), VecVertex::corner(b)],
        ..VecPath::default()
    }
}

/// **A LEI.** Uma forma fechada cortada dá formas **FECHADAS** — nunca fitas.
///
/// É a frase do Enio (2026-07-31) virada gate. Antes desta wave o corte abria o contorno, e o
/// resultado era um caminho aberto: este teste era vermelho por construção.
#[test]
fn a_closed_shape_cut_by_a_line_yields_closed_pieces() {
    let pieces = cut_closed(&diamond(), &line([-4.0, 0.0], [4.0, 0.0])).expect("o corte atravessa");
    assert_eq!(pieces.len(), 2, "um corte reto pelo meio dá DUAS peças");
    for p in &pieces {
        assert!(
            p.closed || p.subpaths.iter().all(|c| c.closed),
            "peça ABERTA — a lei do corte é que fechada corta em fechadas"
        );
    }
}

/// **A área é conservada.** O oráculo que pega um cortador mal construído — nenhum gate de
/// FORMA o pegaria: um `H` cuja fronteira entra na forma por onde ninguém desenhou continua a
/// devolver peças fechadas, com a contagem certa, e com área a menos.
#[test]
fn the_pieces_add_up_to_the_shape() {
    let src = diamond();
    let whole = area(&src);
    let pieces = cut_closed(&src, &line([-4.0, 0.0], [4.0, 0.0])).expect("o corte atravessa");
    let sum: f64 = pieces.iter().map(crate::area).sum();
    assert!(
        (sum - whole).abs() < whole * 1e-3,
        "as peças somam {sum} e a forma tem {whole}"
    );
}

/// Um corte pelo meio de um losango dá **metades iguais** — a asserção que distingue "cortou"
/// de "cortou em algum lugar".
#[test]
fn a_cut_through_the_middle_halves_the_shape() {
    let src = diamond();
    let half = area(&src) / 2.0;
    let pieces = cut_closed(&src, &line([-4.0, 0.0], [4.0, 0.0])).expect("o corte atravessa");
    for p in &pieces {
        assert!(
            (area(p) - half).abs() < half * 1e-2,
            "peça com área {} contra a metade {half}",
            area(p)
        );
    }
}

/// **Um corte que não atravessa não divide nada.** A região menos uma fenda continua conexa —
/// topologia, não pendência. Sem esta recusa, a extensão da linha inventaria o resto do corte.
#[test]
fn a_cut_that_stops_inside_does_not_divide() {
    let err = cut_closed(&diamond(), &line([-4.0, 0.0], [0.5, 0.0])).unwrap_err();
    assert_eq!(err, CutRefusal::DoesNotCrossThrough);
}

/// Uma linha que passa LONGE não corta — e a recusa é `Missed`, não um pânico nem uma peça só.
#[test]
fn a_line_that_misses_cuts_nothing() {
    let err = cut_closed(&diamond(), &line([-4.0, 9.0], [4.0, 9.0])).unwrap_err();
    assert_eq!(err, CutRefusal::Missed);
}

/// **Uma linha FECHADA é o próprio cortador** — cortar um losango com um quadrado desenhado por
/// cima dá o miolo e o anel. Não é caso especial: é o caso geral com zero extensões.
#[test]
fn a_closed_cut_line_is_its_own_cutter() {
    let square = VecPath {
        verts: [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        ..VecPath::default()
    };
    let src = diamond();
    let pieces = cut_closed(&src, &square).expect("um cortador fechado sempre divide");
    assert_eq!(pieces.len(), 2, "o miolo e o resto");
    let sum: f64 = pieces.iter().map(crate::area).sum();
    assert!((sum - area(&src)).abs() < 1e-3, "soma {sum}");
}

/// As peças herdam o ESTILO da fonte. Sem a re-estampagem elas sairiam transparentes: o
/// `apply_many` doa o estilo do path do TOPO, que aqui é o cortador — um objeto sem estilo.
#[test]
fn the_pieces_wear_the_sources_style() {
    let mut src = diamond();
    src.fill = Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
        10, 20, 30, 255,
    )));
    src.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(1, 2, 3, 255),
        0.25,
    ));
    let pieces = cut_closed(&src, &line([-4.0, 0.0], [4.0, 0.0])).expect("o corte atravessa");
    for p in &pieces {
        assert_eq!(p.fill, src.fill, "preenchimento perdido");
        assert_eq!(p.stroke, src.stroke, "traço perdido");
    }
}

/// Uma forma ABERTA não é assunto da porta do FECHADO — cada topologia tem a sua, e a escolha
/// entre elas é da `cut_with_line`.
#[test]
fn an_open_source_is_refused_by_the_closed_door() {
    let open = line([-2.0, 0.0], [2.0, 0.0]);
    assert_eq!(
        cut_closed(&open, &line([0.0, -4.0], [0.0, 4.0])).unwrap_err(),
        CutRefusal::Degenerate
    );
}

/// **Uma FITA cortada parte em duas fitas.** A única resposta possível: uma fita não tem interior,
/// então não há região a dividir — o corte a PARTE.
///
/// Antes desta fatia a fonte aberta era um **no-op silencioso**: o motor devolvia `Degenerate` e a
/// shell seguia adiante. O artista desenhava uma linha, cortava-a, e nada acontecia.
#[test]
fn an_open_source_is_split_into_open_pieces() {
    let ribbon = line([-4.0, 0.0], [4.0, 0.0]);
    let pieces = cut_with_line(&ribbon, &line([0.0, -2.0], [0.0, 2.0])).expect("a lâmina cruza");
    assert_eq!(pieces.len(), 2, "uma fita cortada uma vez da' DUAS");
    for p in &pieces {
        assert!(!p.closed, "o corte FECHOU uma fita -- ela nao tem interior");
        assert!(p.verts.len() >= 2, "peça degenerada");
    }
    // As duas metades encostam onde a lâmina passou (`x = 0`), e cada uma fica do seu lado.
    let ends: Vec<f64> = pieces
        .iter()
        .map(|p| p.verts.last().expect("nao vazia").anchor[0])
        .collect();
    assert!(
        (ends[0] - 0.0).abs() < 1e-6,
        "a 1a metade nao acaba no corte: {ends:?}"
    );
    assert!(
        (ends[1] - 4.0).abs() < 1e-6,
        "a 2a metade nao acaba na ponta: {ends:?}"
    );
}

/// **N cruzamentos dão N+1 peças** — e a ordem delas percorre a fita original.
///
/// ⚠️ A fixture tem TRÊS cruzamentos de propósito: com um só, cortar de trás para a frente e
/// cortar de frente para trás dão o mesmo resultado, e o gate não distinguiria os dois. É a
/// travessia descendente que mantém os índices dos cruzamentos que faltam válidos.
#[test]
fn three_crossings_give_four_pieces_in_order() {
    let ribbon = VecPath {
        verts: [[-6.0, 0.0], [6.0, 0.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    };
    // Uma lâmina em ZIGUE-ZAGUE que cruza a fita em x = -3, 0, 3.
    let blade = VecPath {
        verts: [[-4.5, -1.0], [-1.5, 1.0], [1.5, -1.0], [4.5, 1.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    };
    let pieces = cut_with_line(&ribbon, &blade).expect("três cruzamentos");
    assert_eq!(pieces.len(), 4, "3 cortes dao 4 pedacos");
    // ⚠️ **As fronteiras sao MEDIDAS, nao so' ordenadas.** Com `x < x'` apenas, um corte que pousa
    // no lugar errado sai com o numero certo de pecas, em ordem, e passa -- foi exactamente assim
    // que a mutacao da travessia sobreviveu ao 1o corte deste gate. Os tres cruzamentos estao em
    // -3, 0 e 3 por construcao do zigue-zague.
    let starts: Vec<f64> = pieces
        .iter()
        .map(|p| p.verts.first().expect("nao vazia").anchor[0])
        .collect();
    for (got, want) in starts.iter().zip([-6.0, -3.0, 0.0, 3.0]) {
        assert!(
            (got - want).abs() < 1e-6,
            "peca comeca em {got}, devia comecar em {want}: {starts:?}"
        );
    }
}

/// A fita herda o estilo, como as peças fechadas herdam o delas.
#[test]
fn the_ribbon_pieces_wear_the_sources_stroke() {
    let mut ribbon = line([-4.0, 0.0], [4.0, 0.0]);
    ribbon.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(7, 8, 9, 255),
        0.5,
    ));
    let pieces = cut_with_line(&ribbon, &line([0.0, -2.0], [0.0, 2.0])).expect("cruza");
    for p in &pieces {
        assert_eq!(p.stroke, ribbon.stroke, "traço perdido");
    }
}

/// Uma lâmina que **não toca** a fita não a parte — e a recusa e' `Missed`, nao um pedaco so'.
#[test]
fn a_blade_that_misses_the_ribbon_splits_nothing() {
    let ribbon = line([-4.0, 0.0], [4.0, 0.0]);
    assert_eq!(
        cut_with_line(&ribbon, &line([0.0, 2.0], [0.0, 5.0])).unwrap_err(),
        CutRefusal::Missed
    );
}

/// **A porta única despacha pela TOPOLOGIA da fonte** — fechada para o fechado, aberta para o
/// aberto. É o que impede o chamador de decidir, e com ele o de decidir diferente.
#[test]
fn the_single_door_dispatches_on_the_sources_topology() {
    let blade = line([0.0, -6.0], [0.0, 6.0]);
    let closed = cut_with_line(&diamond(), &blade).expect("fechada corta");
    assert!(closed.iter().all(|p| p.closed), "fechada deu peça aberta");
    let open = cut_with_line(&line([-4.0, 0.0], [4.0, 0.0]), &blade).expect("aberta parte");
    assert!(open.iter().all(|p| !p.closed), "aberta deu peça fechada");
}

/// Um **C** (côncavo): retângulo `[-2,2]×[-1,1]` com uma mordida no lado direito. A concavidade
/// é o que o losango não tem — e é exactamente onde as premissas do cortador podem falhar.
fn c_shape() -> VecPath {
    VecPath {
        verts: [
            [-2.0, -1.0],
            [2.0, -1.0],
            [2.0, -0.5],
            [-1.0, -0.5],
            [-1.0, 0.5],
            [2.0, 0.5],
            [2.0, 1.0],
            [-2.0, 1.0],
        ]
        .into_iter()
        .map(VecVertex::corner)
        .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// **Uma ponta PRESA é recusada, não adivinhada.** A ponta desta lâmina cai na mordida do C: a
/// extensão dela atravessaria o braço de baixo, e cortar ali seria cortar por uma linha que
/// ninguém desenhou. **Recusar em voz alta é melhor que cortar errado.**
#[test]
fn a_trapped_endpoint_is_refused_not_guessed() {
    let err = cut_closed(&c_shape(), &line([1.0, 0.0], [1.0, 2.0])).unwrap_err();
    assert_eq!(err, CutRefusal::Trapped);
}

/// **TODO corte que acontece preserva a área** — a varredura, e o gate mais forte desta suíte.
///
/// É ele que prova a razão de ser da EXTENSÃO: sem ela, uma ponta que fica **fora da forma mas
/// dentro da caixa** deixa o fecho partir dali, e o fecho atravessa a forma por um sítio que
/// ninguém desenhou — a área desaparece sem que a contagem de peças mude, sem que nenhuma peça
/// fique aberta, e sem que nada pareça errado.
///
/// ⚠️ Uma fixture só **não bastou** (a mutação `reach = 0` sobreviveu ao losango, onde as pontas
/// já estavam longe da caixa). O que contém o fenômeno não é uma linha: é a VARREDURA — em
/// alguma destas posições a ponta cai dentro da caixa, e é aí que a extensão passa a decidir.
#[test]
fn every_cut_that_happens_preserves_the_area() {
    for src in [diamond(), c_shape()] {
        let whole = area(&src);
        let mut cuts = 0usize;
        for k in 0..24 {
            let t = f64::from(k) / 24.0;
            // Uma família de lâminas: ângulos e alturas diferentes, pontas a distâncias variadas
            // do centro — de longe (fora da caixa) a perto (dentro dela).
            let r = 1.6 + t * 2.4;
            let (dx, dy) = (t * 6.0 - 3.0, t * 3.0 - 1.5);
            let cut = line([-r + dx * 0.2, dy], [r + dx * 0.2, -dy]);
            let Ok(pieces) = cut_closed(&src, &cut) else {
                continue;
            };
            cuts += 1;
            let sum: f64 = pieces.iter().map(crate::area).sum();
            assert!(
                (sum - whole).abs() < whole * 5e-3,
                "lamina {k}: as pecas somam {sum} e a forma tem {whole} -- o cortador entrou \
                 na forma por onde ninguem desenhou"
            );
        }
        assert!(
            cuts >= 4,
            "a varredura cortou {cuts} vezes -- ela nao exercita nada"
        );
    }
}

/// **Uma linha colada a uma aresta não deixa lascas.** Cortar exactamente por cima de uma borda é
/// o que o artista faz quando encaixa a lâmina nela, e é o caso canônico em que a booleana devolve
/// fatias de área ~0 (o resíduo de tolerância que o `drop_slivers` do Shape Builder já nomeou).
///
/// Sem o filtro elas viram peças fantasma na Hierarquia: contorno longo, área nula, e na tela só
/// o traço da fonte — uma linha solta que ninguém desenhou.
#[test]
fn a_cut_along_an_edge_leaves_no_slivers() {
    let src = c_shape();
    // Exactamente sobre a aresta de baixo (`y = -1`).
    let pieces = cut_closed(&src, &line([-4.0, -1.0], [4.0, -1.0]));
    match pieces {
        // Ou o motor não vê divisão nenhuma (a aresta não parte a forma) …
        Err(CutRefusal::Missed) => {}
        // … ou vê, e então NENHUMA peça pode ser uma lasca.
        Ok(ps) => {
            let whole = area(&src);
            for p in &ps {
                assert!(
                    crate::area(p) > whole * 0.005,
                    "lasca de area {} sobreviveu (forma: {whole})",
                    crate::area(p)
                );
            }
        }
        Err(other) => panic!("recusa inesperada: {other:?}"),
    }
}
