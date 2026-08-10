//! Gates da multiresolução.
//!
//! ⚠️ **O gate que decide o módulo é a IDA E VOLTA EXATA.** Toda a razão de
//! guardar detalhe em vez de posições é poder descer e subir sem perder nada; se
//! a viagem custa um erro, ela custa esse erro *a cada vez*, e a escultura
//! escorrega de forma que nenhum sintoma nomeia.

use super::*;
use crate::shapes;

/// Empurra um vértice ao longo da própria normal — o "detalhe" das fixtures.
fn bump(mesh: &mut Mesh, v: usize, amount: f32) {
    let n = mesh.normals()[v];
    let p = mesh.positions()[v];
    mesh.positions_mut()[v] = [
        p[0] + n[0] * amount,
        p[1] + n[1] * amount,
        p[2] + n[2] * amount,
    ];
    mesh.rebuild();
}

fn worst(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// ⚠️ **A IDA E VOLTA É EXATA quando nada muda embaixo.**
///
/// `previsão + (topo − previsão) = topo`, desde que o frame seja o mesmo dos dois
/// lados. Um encode e um decode escritos separadamente passariam num vértice de
/// normal alinhada ao eixo e falhariam no resto — por isso o gate mede a malha
/// TODA, e por isso o frame tem uma porta só.
#[test]
fn a_round_trip_that_changes_nothing_below_is_exact() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    // Detalhe autorado no topo, em vários lugares.
    for v in [3usize, 17, 42, 88] {
        bump(m.mesh_mut(), v, 0.15);
    }
    let before = m.mesh().positions().to_vec();

    assert!(m.lower().is_some());
    assert_eq!(m.level(), 0);
    assert!(m.higher());
    assert_eq!(m.level(), 1);

    let after = m.mesh().positions();
    assert_eq!(after.len(), before.len());
    let err = worst(&before, after);
    assert!(err < 1e-5, "a viagem custou {err} de deslocamento");
}

/// E ela continua exata depois de VÁRIAS viagens — um erro de 1e-6 por volta
/// seria invisível numa e visível em vinte.
#[test]
fn twenty_round_trips_do_not_drift() {
    let mut m = Multires::new(shapes::cube(2.0));
    assert!(m.add_level());
    assert!(m.add_level());
    bump(m.mesh_mut(), 5, 0.2);
    let before = m.mesh().positions().to_vec();
    for _ in 0..20 {
        m.select(0);
        m.select(2);
    }
    let err = worst(&before, m.mesh().positions());
    assert!(err < 1e-5, "vinte viagens acumularam {err}");
}

/// **Esculpir EMBAIXO move o de cima** — a metade que dá sentido a descer.
///
/// ⚠️ **Dois oráculos, e o primeiro é exato de propósito.** Transladar a base
/// inteira tem de transladar o topo pelo MESMO vetor: a tabela de pesos é afim,
/// então ela comuta com uma translação, e o detalhe é uma diferença — que uma
/// translação não muda. Isso é derivação, não um número escolhido.
///
/// ⚠️ O segundo é o empurrão LOCAL, e ali a barra é medida: um empurrão de
/// **0,4** num vértice da base chega ao topo como **0,150**, porque a regra par
/// atenua (é o `α` de Loop/Catmull-Clark). A primeira versão deste gate cravava
/// `> 0,2` — um palpite meu, e o produto estava certo.
#[test]
fn sculpting_the_base_moves_the_level_above_it() {
    // — a translação, com oráculo exato —
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    let before = m.mesh().positions().to_vec();
    assert!(m.lower().is_some());
    for p in m.mesh_mut().positions_mut() {
        p[0] += 0.37;
    }
    m.mesh_mut().rebuild();
    assert!(m.higher());
    let worst_err = before
        .iter()
        .zip(m.mesh().positions())
        .map(|(a, b)| {
            ((b[0] - a[0] - 0.37).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst_err < 1e-4,
        "transladar a base tem de transladar o topo igual, e desviou {worst_err}"
    );

    // — o empurrão local, atenuado pelo peso par —
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    let before = m.mesh().positions().to_vec();
    assert!(m.lower().is_some());
    bump(m.mesh_mut(), 4, 0.4);
    assert!(m.higher());
    let moved = worst(&before, m.mesh().positions());
    assert!(
        (0.10..0.40).contains(&moved),
        "um empurrão de 0,4 na base chega ao topo atenuado (medido 0,150), e mediu {moved}"
    );
}

/// **E o DETALHE sobrevive à mudança da base** — a razão inteira do módulo.
///
/// O oráculo é a distância do vértice detalhado à superfície que a subdivisão
/// PORIA ali: ela é o detalhe, e tem de continuar a mesma depois de a base
/// andar.
#[test]
fn the_detail_survives_a_change_to_the_base() {
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    const V: usize = 30;
    const D: f32 = 0.25;
    bump(m.mesh_mut(), V, D);

    // Quanto o vértice está fora da previsão, antes de mexer embaixo.
    let detail_before = {
        let p = predict(&Multires::new(shapes::uv_sphere(8, 12, 1.0)).mesh().clone());
        let q = m.mesh().positions()[V];
        ((q[0] - p.positions[V][0]).powi(2)
            + (q[1] - p.positions[V][1]).powi(2)
            + (q[2] - p.positions[V][2]).powi(2))
        .sqrt()
    };
    assert!(
        (detail_before - D).abs() < 0.02,
        "a fixture tem de ter detalhe: {detail_before}"
    );

    assert!(m.lower().is_some());
    // Translada a base INTEIRA — o teste mais limpo, porque uma translação não
    // gira frame nenhum e isola *o detalhe sobreviveu?* de *o frame girou?*.
    for p in m.mesh_mut().positions_mut() {
        p[0] += 0.5;
    }
    m.mesh_mut().rebuild();
    assert!(m.higher());

    let predicted = predict(&{
        let mut base = shapes::uv_sphere(8, 12, 1.0);
        for p in base.positions_mut() {
            p[0] += 0.5;
        }
        base.rebuild();
        base
    });
    let q = m.mesh().positions()[V];
    let detail_after = ((q[0] - predicted.positions[V][0]).powi(2)
        + (q[1] - predicted.positions[V][1]).powi(2)
        + (q[2] - predicted.positions[V][2]).powi(2))
    .sqrt();
    assert!(
        (detail_after - detail_before).abs() < 1e-4,
        "o detalhe era {detail_before} e virou {detail_after}"
    );
}

/// ⚠️ **A base compartilha os V primeiros vértices com o topo**, e é disso que
/// o carimbo da descida depende. Se a subdivisão numerasse os vértices novos
/// antes dos velhos, descer somaria diferenças nos vértices errados sem levantar
/// erro.
#[test]
fn the_even_vertices_keep_their_index_through_a_subdivision() {
    let mesh = shapes::uv_sphere(9, 13, 1.0);
    let out = subdivide(&mesh);
    // Não é que as POSIÇÕES sejam iguais (a regra par as move) — é que o
    // vértice `i` de cima é o descendente do vértice `i` de baixo. O que se
    // afirma é a correspondência: cada um está mais perto do seu original do
    // que da média das arestas que o cercam.
    for v in 0..mesh.vert_count() {
        let (a, b) = (mesh.positions()[v], out.positions()[v]);
        let moved = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        assert!(
            moved < 0.2,
            "o vértice {v} andou {moved}: ele não é o descendente do original"
        );
    }
}

/// Subdividir só do topo, e a recusa é `false` — nunca uma pilha que descarta
/// trabalho em silêncio.
#[test]
fn a_level_is_only_added_from_the_top() {
    let mut m = Multires::new(shapes::cube(2.0));
    assert!(m.add_level());
    assert!(m.lower().is_some());
    assert!(!m.add_level(), "do meio, não");
    assert_eq!(m.level_count(), 2);
    assert!(m.higher());
    assert!(m.add_level(), "do topo, sim");
    assert_eq!(m.level_count(), 3);
}

/// As bordas da pilha: descer do 0 e subir do topo são no-ops que dizem `false`.
#[test]
fn the_ends_of_the_stack_refuse_instead_of_wrapping() {
    let mut m = Multires::new(shapes::cube(2.0));
    assert!(m.lower().is_none());
    assert!(!m.higher());
    assert_eq!(m.level(), 0);
    assert!(m.add_level());
    assert!(!m.higher());
    assert_eq!(m.level(), 1);
}

/// A MÁSCARA viaja pela pilha — pintar no nível 2, descer e voltar não é uma
/// forma de perder a proteção.
#[test]
fn a_painted_mask_survives_the_round_trip() {
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    let n = m.mesh().vert_count();
    {
        let masks = m.mesh_mut().masks_mut();
        for (i, x) in masks.iter_mut().enumerate().take(n) {
            *x = if i % 3 == 0 { 1.0 } else { 0.0 };
        }
    }
    let before = m.mesh().masks().expect("pintada").to_vec();
    m.select(0);
    m.select(1);
    let after = m.mesh().masks().expect("viaja");
    let err = before
        .iter()
        .zip(after)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(err < 1e-5, "a máscara desviou {err} na viagem");
}

/// `select` caminha os dois sentidos e para onde foi pedido.
#[test]
fn select_walks_to_the_level_it_was_asked_for() {
    let mut m = Multires::new(shapes::cube(2.0));
    for _ in 0..3 {
        assert!(m.add_level());
    }
    assert_eq!(m.level(), 3);
    m.select(0);
    assert_eq!(m.level(), 0);
    m.select(2);
    assert_eq!(m.level(), 2);
    // Fora de alcance para em quem existe, em vez de panicar.
    m.select(99);
    assert_eq!(m.level(), 3);
}

/// **Um nível destacado volta EXATAMENTE como saiu** — a inversa que o refazer
/// de uma subdivisão precisa.
#[test]
fn a_detached_level_comes_back_the_way_it_left() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    for v in [3usize, 17, 42] {
        bump(m.mesh_mut(), v, 0.2);
    }
    let top = m.mesh().positions().to_vec();

    let gone = m.drop_top().expect("havia um topo");
    assert_eq!(m.level(), 0);
    assert_eq!(m.level_count(), 1);

    assert!(m.push_level(gone));
    assert_eq!(
        m.level(),
        1,
        "recolocar também SELECIONA — é a inversa inteira"
    );
    assert_eq!(m.level_count(), 2);
    assert_eq!(
        m.mesh().positions(),
        &top[..],
        "o nível voltou aproximado em vez de exato"
    );
}

/// ⚠️ **E é por isso que ele é CARREGADO em vez de recomputado.**
///
/// A alternativa barata — refazer uma subdivisão chamando `add_level` de novo —
/// só reproduz o nível enquanto o de baixo estiver como estava, e **`lower`
/// escreve no de baixo** (o carimbo). Depois de UMA viagem para baixo COM trabalho, a
/// recomputação devolve uma malha *parecida*, que é a pior forma de errado
/// porque ninguém a vê. Este gate mede as duas rotas sobre a mesma pilha.
#[test]
fn recomputing_the_subdivision_is_not_the_level_that_was_dropped() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    // Num vértice COMPARTILHADO: é o que a descida carimba na base.
    bump(m.mesh_mut(), 7, 0.3);
    assert!(m.lower().is_some());
    assert!(m.higher());
    let top = m.mesh().positions().to_vec();

    // A rota que shipa: destaca e devolve.
    let gone = m.drop_top().expect("havia um topo");
    assert!(m.push_level(gone));
    assert_eq!(m.mesh().positions(), &top[..], "carregar o nível é exato");

    // A rota barata: destaca, joga fora, subdivide de novo.
    let gone = m.drop_top().expect("havia um topo");
    drop(gone);
    assert!(m.add_level());
    let err = worst(&top, m.mesh().positions());
    assert!(
        err > 0.05,
        "recomputar tinha de DIVERGIR depois de uma descida, e desviou só {err}"
    );
}

/// ⚠️ **O QUE O ARTISTA VÊ AO DESCER** — o gate que faltava, achado por mutação.
///
/// Apagar o carimbo da descida **sobrevive a todos os gates de viagem**: sem ele
/// a base fica como estava, a previsão sai a mesma dos dois lados, e a ida e
/// volta continua EXATA — só que descer passa a mostrar uma malha que ignora
/// tudo o que o artista esculpiu em cima. *O trabalho está guardado no detalhe e
/// invisível no lugar onde ele foi ao procurá-lo.*
#[test]
fn descending_shows_the_work_that_was_done_above() {
    // ⚠️ **A régua é a posição da BASE, não a do topo.** A primeira versão deste
    // gate media o deslocamento da base a partir de onde o TOPO estava — dois
    // pontos diferentes (a regra par separa os dois níveis por até 0,038), e
    // ele acusou o produto de mover 83% do que devia.
    let start = shapes::uv_sphere(10, 14, 1.0);
    let before = start.positions().to_vec();
    let mut m = Multires::new(start);
    assert!(m.add_level());
    const V: usize = 7; // um vértice que a base COMPARTILHA com o topo
    let was = before[V];
    let top_was = m.mesh().positions()[V];
    bump(m.mesh_mut(), V, 0.3);
    let pushed = sub3(m.mesh().positions()[V], top_was); // o que o artista moveu

    assert!(m.lower().is_some());
    let base = m.mesh().positions()[V];
    let moved = norm3(sub3(base, was));
    assert!(
        moved > 0.25,
        "a base tem de mostrar o empurrão de 0,3 e mostrou {moved}"
    );

    // ⚠️ **E ela move o que o ARTISTA moveu, nem um décimo a mais.**
    //
    // A metade anterior deste gate dizia *a base é EXATAMENTE o topo* — e era
    // essa frase que continha o defeito: para um vértice par, `topo[i]` é a
    // REGRA PAR aplicada à base, ou seja um alisamento. Afirmá-la obrigava a
    // descida a carimbar esse alisamento na base, e o preço, medido, era o
    // modelo do artista encolhendo 2,81% no primeiro `,` e mais a cada ciclo.
    for k in 0..3 {
        let got = base[k] - was[k];
        assert!(
            (got - pushed[k]).abs() < 1e-6,
            "eixo {k}: o artista moveu {} e a base andou {got}",
            pushed[k]
        );
    }

    // ⚠️ E o outro lado da mesma lei: **quem não foi tocado não anda.** É esta
    // metade que o alisamento violava em TODO vértice da malha.
    let quiet = before
        .iter()
        .zip(m.mesh().positions())
        .enumerate()
        .filter(|(i, _)| *i != V)
        .map(|(_, (a, b))| norm3(sub3(*a, *b)))
        .fold(0.0f32, f32::max);
    assert!(
        quiet == 0.0,
        "descer moveu {quiet} num vértice que ninguém esculpiu"
    );
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// ⚠️ **ANDAR ENTRE NÍVEIS NÃO É EDITAR** — o gate do defeito que o Enio
/// reportou, e o oráculo é BIT-EXATO de propósito.
///
/// Antes, `K` seguido de `,` — **sem esculpir nada** — encolhia a base de raio
/// médio 1,000 para **0,972** e a deslocava 0,038, porque a descida copiava
/// `topo[..V]` (a regra par, um alisamento) para a base. E **compunha**: 2,81%
/// num ciclo, 3,32% em dois, 3,45% em três. O artista perdia o modelo por ter
/// olhado, e nada na tela dizia por quê.
#[test]
fn walking_the_levels_without_sculpting_does_not_move_a_single_vertex() {
    let start = shapes::uv_sphere(10, 14, 1.0);
    let orig = start.positions().to_vec();
    let mut m = Multires::new(start);

    for round in 1..=3 {
        m.select(m.level_count() - 1);
        assert!(m.add_level());
        // Sobe e desce várias vezes — é o gesto de quem está conferindo a forma.
        for _ in 0..4 {
            m.select(0);
            m.select(m.level_count() - 1);
        }
        m.select(0);
        assert_eq!(
            m.mesh().positions(),
            &orig[..],
            "depois de {round} ciclo(s) de resolução a base tem de estar INTACTA"
        );
    }

    // E o carimbo de um passeio ocioso diz que não carimbou nada — é o que faz a
    // entrada de desfazer dele custar dezesseis bytes.
    m.select(1);
    let stamped = m.lower().expect("desceu");
    assert!(
        stamped.is_noop(),
        "descer sem esculpir tem de reportar carimbo VAZIO"
    );
    assert_eq!(stamped.level(), 0);
}

/// ⚠️ **E o passeio continua ocioso DEPOIS de o artista ter esculpido** — o
/// gate que faltava, achado por mutação.
///
/// O irmão acima nunca esculpe, então o detalhe fica exatamente zero e a régua
/// recomputada acerta ao bit: **ele fica verde com o piso em 0**. É com detalhe
/// que a ida-e-volta do frame erra 1,49e-8, e sem piso *cada* volta carimbaria
/// esse ulp na base — uma entrada de desfazer do tamanho da base por gesto de
/// quem só foi olhar, e uma deriva que ninguém nomeia.
#[test]
fn a_second_walk_stamps_nothing_once_the_work_is_already_down() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    for v in [3usize, 7, 11] {
        bump(m.mesh_mut(), v, 0.3);
    }
    // A primeira descida carimba — é o trabalho descendo.
    assert!(!m.lower().expect("desceu").is_noop());
    let base = m.mesh().positions().to_vec();

    // As seguintes não têm o que carimbar, e a base não anda um bit.
    for round in 1..=5 {
        assert!(m.higher());
        let stamped = m.lower().expect("desceu");
        assert!(
            stamped.is_noop(),
            "a volta {round} carimbou o que ninguém esculpiu"
        );
        assert_eq!(
            m.mesh().positions(),
            &base[..],
            "e a base andou na volta {round}"
        );
    }
}

/// **Desfazer uma descida devolve à base o que ela carimbou** — e o topo
/// continua exato depois disso.
#[test]
fn undoing_a_descent_gives_the_base_back() {
    // ⚠️ **O carimbo é IDEMPOTENTE**, então a fixture só tem o que carimbar na
    // PRIMEIRA descida depois de esculpir — a primeira versão deste gate descia
    // duas vezes e depois exigia um carimbo não-vazio da segunda.
    let start = shapes::uv_sphere(10, 14, 1.0);
    let base_before = start.positions().to_vec();
    let mut m = Multires::new(start);
    assert!(m.add_level());
    for v in [3usize, 7, 11] {
        bump(m.mesh_mut(), v, 0.3);
    }
    let top = m.mesh().positions().to_vec();

    let stamped = m.lower().expect("desceu");
    let base_stamped = m.mesh().positions().to_vec();
    assert!(!stamped.is_noop(), "a fixture tem de ter o que carimbar");
    assert_ne!(
        base_stamped, base_before,
        "e a descida tem de ter mesmo mexido na base"
    );

    assert!(m.undo_descent(&stamped), "desfazer a descida");
    assert_eq!(m.level(), 1, "e ela volta ao nível de onde saiu");
    assert_eq!(m.mesh().positions(), &top[..], "o topo não foi tocado");

    // ⚠️ **E descer OUTRA VEZ, à mão, carimba a MESMA coisa** — é este o dente
    // do gate, e é a propriedade que se perde quando o desfazer re-encoda em vez
    // de restaurar o detalhe: o re-encode faz o detalhe absorver a escultura, a
    // descida seguinte mede diferença zero, e a base deixa de mostrar o trabalho
    // que o artista continua vendo lá em cima.
    m.select(0);
    assert_eq!(
        m.mesh().positions(),
        &base_stamped[..],
        "descer de novo tem de carimbar o mesmo — o desfazer devolveu base E detalhe"
    );
    m.select(1);
    // ⚠️ Aqui o oráculo é o mesmo do resto do módulo (< 1e-5) e não a igualdade
    // de bits: a volta passa por um encode e uma síntese, cujo round-trip de
    // frame erra um ulp. As bases acima SÃO bit-exatas porque nada as projeta.
    let err = worst(&top, m.mesh().positions());
    assert!(err < 1e-5, "o topo se reconstrói, e desviou {err}");
}

// ── O ACHATAR ───────────────────────────────────────────────────────────────

/// ⚠️ **A malha que fica é a do TOPO, e o gate mede isso DE PÉ NO NÍVEL 0** —
/// que é o único lugar onde a resposta errada é plausível.
///
/// `levels[k]` acima do selecionado está obsoleto (é a `higher` quem o
/// sintetiza), então a implementação preguiçosa — *ficar com a malha que o
/// artista está vendo* — devolveria a BASE, com todo o detalhe do topo perdido
/// em silêncio. O oráculo é a malha que a subida produz, e ele não conhece o
/// `flatten`: ele é a pilha antiga, subida à mão.
#[test]
fn flattening_from_the_bottom_keeps_the_detail_that_lives_on_top() {
    let mut m = Multires::new(shapes::uv_sphere(10, 14, 1.0));
    assert!(m.add_level());
    for v in [3usize, 17, 42, 88] {
        bump(m.mesh_mut(), v, 0.2);
    }
    // O oráculo: a malha do topo, lida ANTES de descer.
    let top = m.mesh().positions().to_vec();

    // O artista desce para mexer na forma grande, e acha o botão daqui.
    assert!(m.lower().is_some());
    assert_eq!(m.level(), 0);
    let base_len = m.mesh().vert_count();
    assert!(
        base_len < top.len(),
        "a fixture não contém o fenômeno: base e topo têm a mesma contagem"
    );

    assert!(m.flatten().is_some());
    assert_eq!(m.level_count(), 1, "sobrou pilha depois de achatar");
    assert_eq!(m.level(), 0);
    assert_eq!(
        m.mesh().vert_count(),
        top.len(),
        "ficou com a BASE: o detalhe do topo foi jogado fora"
    );
    let err = worst(&top, m.mesh().positions());
    assert!(
        err < 1e-5,
        "o topo não sobreviveu ao achatar: desviou {err}"
    );
}

/// **O gesto tem inverso EXATO** — o que ele devolve é a pilha de antes, e
/// instalá-la de volta reproduz nível, contagem e geometria.
///
/// ⚠️ É o que torna a entrada de desfazer honesta: sem esta propriedade o
/// Ctrl+Z devolveria uma pilha *parecida*, que é a pior forma de errado porque
/// ninguém vê.
#[test]
fn what_the_flatten_hands_back_restores_the_stack_it_took() {
    let mut m = Multires::new(shapes::uv_sphere(8, 12, 1.0));
    assert!(m.add_level());
    for v in [5usize, 21] {
        bump(m.mesh_mut(), v, 0.1);
    }
    assert!(m.lower().is_some());
    let want_level = m.level();
    let want_count = m.level_count();
    let want_mesh = m.mesh().positions().to_vec();

    let previous = m.flatten().expect("havia dois níveis");
    assert_eq!(m.level_count(), 1);

    // O desfazer: a pilha inteira volta ao lugar.
    m = previous;
    assert_eq!(m.level_count(), want_count);
    assert_eq!(m.level(), want_level);
    assert_eq!(m.mesh().positions(), &want_mesh[..]);
}

/// **Com um nível só ele RECUSA**, e não devolve uma pilha igual.
///
/// ⚠️ A distinção importa para quem chama: um `Some` aqui gravaria uma entrada
/// de desfazer que carrega o documento inteiro para não mudar nada — e o teto em
/// bytes da história pagaria por ela.
#[test]
fn a_stack_of_one_has_nothing_to_flatten() {
    let mut m = Multires::new(shapes::uv_sphere(6, 8, 1.0));
    assert_eq!(m.level_count(), 1);
    assert!(m.flatten().is_none());
    assert_eq!(m.level_count(), 1);
}

/// **Achatar leva a UM nível, de QUALQUER altura** — e era esta a saída que
/// faltava.
///
/// ⚠️ **Cinco recusas do shell mandavam o artista *"reverter os níveis antes"*
/// de reconstruir, fundir ou ligar a topologia dinâmica — e seguir o conselho
/// tornava a recusa mais CERTA.** A reversão insere um nível por BAIXO (o gate
/// `reversing_inserts_a_level_below_...` do irmão pina isso em `level_count == 2`),
/// então ela é o oposto de uma saída. Este gate é a metade nova: o verbo que de
/// fato reduz a pilha reduz a UM, venha o artista do nível que vier.
#[test]
fn flattening_lands_on_one_level_from_any_height() {
    let mut m = Multires::new(shapes::uv_sphere(6, 8, 1.0));
    for _ in 0..3 {
        assert!(m.add_level());
    }
    assert_eq!(m.level_count(), 4);
    // Do MEIO da pilha — o caso em que ficar com a malha vista perderia dois
    // níveis de detalhe.
    m.select(1);
    let top_verts = m.level_mesh(3).expect("o topo existe").vert_count();

    assert!(m.flatten().is_some());
    assert_eq!(m.level_count(), 1);
    assert_eq!(m.level(), 0);
    assert_eq!(
        m.mesh().vert_count(),
        top_verts,
        "achatou para a malha VISTA, não para a do topo"
    );
}
