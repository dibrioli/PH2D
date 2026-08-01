//! **O DELTA QUE VEM DO JOURNAL É O MESMO QUE VEM DE DOIS SNAPSHOTS** — o gate do degrau 2 do S3
//! (doc 28 §5.58.2).
//!
//! O degrau 2 não compra um milissegundo: ele troca a ORIGEM do lado `before` do relevo (de um segundo
//! snapshot para os bytes que o journal capturou na hora da escrita) e prova que o resultado é o
//! mesmo. É essa prova que torna o degrau 4 — onde o `cursor` e o `stroke_undo` largam os planos e os
//! ~9,6 ms do fold caem — uma mudança mecânica em vez de uma aposta.
//!
//! # O oráculo é o ENDPOINT MATERIALIZADO, não a forma do enum
//!
//! ⚠️ Comparar os dois `StoredPlane` **campo a campo estaria errado**, e falharia sobre produto
//! correto: a janela do journal é uma caixa de tiles apertada pela declarada, e a do `split` clássico
//! sai do `diff_window` quando não há declaração — as duas contêm o escrito e podem ter tamanhos
//! diferentes. O que o undo consome não é a janela: é o plano que a materialização devolve. É ele que
//! tem de ser igual, **ao byte**, nas duas direções.
//!
//! # E o contador é o que impede o verde por vácuo
//!
//! ⚠️ Um gate que compara as duas rotas passa perfeitamente quando **as duas caem no fallback** — aí
//! ele compara o caminho de sempre contra ele mesmo, sobre uma rota que nunca rodou (a armadilha do
//! ADR-0120, que o oráculo de undo do ADR-0124 pagou uma segunda vez). Por isso todo teste daqui
//! afirma primeiro que o journal de fato respondeu.

use super::measure_stroke_owners::{armed, cp, stroke};
use crate::undo::ModelSnapshot;
use crate::undo_planes::{PlaneDeltas, RELIEF_FROM_JOURNAL, ReliefSource};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// Um traço com o **FOLD já corrido** e o commit ainda não — o estado exato em que
/// `record_structural_hinted` parte o delta.
///
/// ⚠️ **Duas armadilhas de fixture, e a segunda custou uma rodada.**
///
/// 1. O pen-up **commita**, e um commit zera o journal (`set_cursor`). Medir depois dele é medir um
///    journal vazio, e a igualdade sairia verdadeira por construção — o que o gate da máscara
///    (`journal_tests`) já pagou uma vez.
/// 2. ⚠️ **O relevo NÃO é escrito por dab.** O impasto é *por-traço*: os dabs alimentam um envelope e
///    quem escreve `heights`/`covers`/`mats` — pelas portas nomeadas, que é o que enche os journals —
///    é o **fold** (`commit_stroke_height`), no pen-up. A primeira versão desta fixture parava nos
///    Moves e media `relief_state = SEM-RELEVO`: o guard recusava, e o gate acusava a rota como não
///    executada. A fixture tem de conter o fold, e é ele que a torna o estado do commit.
///
/// ⚠️ **E a TELA é parte da fixture.** A 256² um tile (128 elementos de lado) mede meia tela: a caixa
/// do journal colapsa no plano inteiro, as duas rotas caem no mesmo `Whole`, e três mutações reais
/// **sobreviveram** ao gate na primeira rodada. A 512² a caixa é grossa o bastante para diferir da
/// janela declarada e fina o bastante para não engolir o plano — que é onde a diferença entre as rotas
/// existe para ser vista.
fn tool_mid_step(side: u32) -> crate::tool::PainterTool {
    let mut t = armed(side);
    stroke(&mut t, 40.0); // o 1º traço instala o histórico, o cursor e o relevo da camada
    t.begin_undo_step();
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Down));
    for k in 1..=6u8 {
        t.on_canvas_pointer(cp([60.0 + f32::from(k) * 20.0, 120.0], PointerPhase::Move));
    }
    t.commit_stroke_height(); // o fold — a metade do pen-up que escreve os três planos
    t
}

/// Os dois endpoints que o commit veria agora: o `before` que o pen-down guardou e o `after` vivo.
fn endpoints(t: &crate::tool::PainterTool) -> (ModelSnapshot, ModelSnapshot) {
    let before = t
        .paint
        .stroke_undo
        .clone()
        .expect("o pen-down guarda o `before` do passo");
    (before, t.snapshot_model())
}

/// Parte os dois endpoints pelas DUAS rotas e devolve `(do journal, de dois snapshots)`.
///
/// Cada rota come as suas próprias cópias — `split` esvazia o que recebe, e uma cópia de
/// `ModelSnapshot` é um punhado de refcounts.
fn both_routes(t: &crate::tool::PainterTool) -> (PlaneDeltas, PlaneDeltas, ModelSnapshot) {
    let (before, after) = endpoints(t);
    let hint = t.undo.write_state.get().hint_for(before.writes);
    let cursor = after.clone();

    let (mut b1, mut a1) = (before.clone(), after.clone());
    let seen = RELIEF_FROM_JOURNAL.with(std::cell::Cell::get);
    let from_journal = PlaneDeltas::split(
        &mut b1,
        &mut a1,
        hint,
        Some(ReliefSource {
            state: &t.undo.write_state,
            writes: before.writes,
            layer: t.layers.active().expect("o fixture pinta numa camada"),
        }),
    );
    assert_eq!(
        RELIEF_FROM_JOURNAL.with(std::cell::Cell::get),
        seen + 1,
        "a rota do JOURNAL nao rodou — o guard de proveniencia a recusou, e comparar o caminho de \
         sempre contra ele mesmo e' verde por vacuo (doc 28 §5.58.2)"
    );

    let (mut b2, mut a2) = (before, after);
    let classic = PlaneDeltas::split(&mut b2, &mut a2, hint, None);
    (from_journal, classic, cursor)
}

/// O lado `want_before` do relevo, materializado a partir do cursor — o que o `restore_model`
/// instalaria.
fn relief_side(
    d: &PlaneDeltas,
    cursor: &ModelSnapshot,
    want_before: bool,
) -> (Vec<f32>, Vec<u8>, Vec<[u8; 7]>) {
    fn plane<T: Clone>(
        m: &std::collections::BTreeMap<crate::layers::LayerId, std::sync::Arc<Vec<T>>>,
        layer: crate::layers::LayerId,
    ) -> Vec<T> {
        m.get(&layer).map(|v| (**v).clone()).unwrap_or_default()
    }
    let mut out = cursor.clone();
    d.side(cursor, &mut out, want_before)
        .expect("o cursor descreve o delta que acabamos de partir");
    let l = cursor.layers.active().expect("a camada ativa do fixture");
    (
        plane(&out.heights, l),
        plane(&out.covers, l),
        plane(&out.mats, l),
    )
}

/// **O gate central do degrau 2.** As duas rotas descrevem os MESMOS dois endpoints do relevo.
///
/// ⚠️ **Mutação que sangra:** trocar a origem da janela em
/// [`StoredPlane::from_journal`](crate::undo_delta::StoredPlane::from_journal) — deslocar o `win` em
/// uma linha, ou trocar o `unwrap_or(live[i])` por um valor fixo. O lado `before` passa a descrever
/// texels que nunca existiram, e é exatamente o que o undo instalaria.
#[test]
fn the_journal_delta_describes_the_same_two_endpoints_as_two_snapshots() {
    assert_routes_agree(&tool_mid_step(512));
}

/// **Dentro da caixa há tiles que NINGUÉM tomou, e ali o `before` é o plano VIVO** — a metade da lei
/// da §5.28 que um traço sozinho não alcança.
///
/// ⚠️ **Um fold declara UMA região, e uma região vira uma caixa de tiles CHEIA** — então no traço
/// comum `j.get(i)` responde para todo elemento da janela e o `unwrap_or(vivo[i])` **nunca roda**. Duas
/// mutações reais dele sobreviveram ao gate central por isso. A lei só é load-bearing quando o passo
/// escreve em regiões DISJUNTAS (um deposit mais um warp, um sculpt mais um smear), e é isso que esta
/// fixture encena — pela porta de verdade, como o gate da gota faz com o `fork_canvas`.
///
/// ⚠️ **E o miolo intacto tem de carregar relevo VARIADO**, senão ele é um campo de zeros e trocar o
/// vivo por zero seria indistinguível: daí o traço anterior ficar na faixa que este passo não toca.
///
/// ⚠️ **Mutação que sangra:** `unwrap_or(live[i])` → `unwrap_or(live[0])` (ou `live[i-1]`).
#[test]
fn inside_the_box_an_untaken_tile_reads_the_before_from_the_live_plane() {
    let mut t = armed(1024);
    stroke(&mut t, 300.0); // o relevo do MIOLO — a faixa que o passo seguinte não toca
    t.begin_undo_step();
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Down));
    for k in 1..=6u8 {
        t.on_canvas_pointer(cp([60.0 + f32::from(k) * 20.0, 120.0], PointerPhase::Move));
    }
    t.commit_stroke_height();

    // A SEGUNDA região do mesmo passo, longe da primeira — é ela que abre buracos na caixa.
    let (w, h) = t.source_size;
    let layer = t.layers.active().expect("a camada ativa do fixture");
    let n = (w as usize) * (h as usize);
    let far = crate::compositor::Region {
        x: 60,
        y: 384,
        w: 32,
        h: 32,
    };
    let entry = t.heights.get_mut(&layer).expect("a camada tem relevo");
    assert_eq!(n, entry.len(), "controle: o relevo nao e' canvas-shaped");
    let dst = super::plane_fork::fork_heights(entry, &t.undo.write_state, layer, (w, h), Some(far));
    for y in far.y..far.y + far.h {
        for x in far.x..far.x + far.w {
            dst[(y * w + x) as usize] = 7.5;
        }
    }
    t.declare_wrote(Some(far));

    assert_routes_agree(&t);
}

/// A comparação em si — **uma porta**, para que o caso esparso não nasça com a asserção fraca.
fn assert_routes_agree(t: &crate::tool::PainterTool) {
    let (j, c, cursor) = both_routes(t);

    for want_before in [true, false] {
        let (jh, jc, jm) = relief_side(&j, &cursor, want_before);
        let (ch, cc, cm) = relief_side(&c, &cursor, want_before);
        let side = if want_before { "before" } else { "after" };

        // Controle: o fixture tem de CONTER o fenômeno. Um relevo vazio faria as duas rotas
        // concordarem sobre nada.
        assert!(
            !ch.is_empty() && ch.iter().any(|&h| h != 0.0),
            "controle: o fixture nao deixou relevo, entao a igualdade nao diz nada"
        );

        assert_eq!(
            jh.len(),
            ch.len(),
            "heights/{side}: as duas rotas devolvem planos de tamanhos diferentes"
        );
        let dh = jh.iter().zip(&ch).filter(|(a, b)| a != b).count();
        let dc = jc.iter().zip(&cc).filter(|(a, b)| a != b).count();
        let dm = jm.iter().zip(&cm).filter(|(a, b)| a != b).count();
        assert_eq!(
            (dh, dc, dm),
            (0, 0, 0),
            "o relevo materializado do JOURNAL diverge do de dois snapshots no lado {side}: \
             heights {dh}, covers {dc}, mats {dm} elementos. O degrau 4 instalaria esses bytes."
        );
    }
}

/// **A rota do journal não muda o que o histórico RETÉM** — o outro eixo, e ele tem gate próprio
/// porque a igualdade de conteúdo não o implica.
///
/// ⚠️ Foi ele que reprovou a primeira versão da wave: a caixa de tiles do journal é 128-alinhada, então
/// sem a interseção com a janela declarada o passo típico saltou de **2,51 para 8,23 MB a 1024²** — com
/// os endpoints materializados **idênticos** o tempo todo. Conteúdo e memória são perguntas separadas,
/// e o `measure_undo_capacity` só as vê no agregado.
///
/// ⚠️ **Mutação que sangra:** tirar o `intersect` do
/// [`StoredPlane::from_journal`](crate::undo_delta::StoredPlane::from_journal).
#[test]
fn the_journal_route_retains_what_the_classic_route_retains() {
    let t = tool_mid_step(512);
    let (j, c, _) = both_routes(&t);
    let (bj, bc) = (j.heap_bytes(), c.heap_bytes());
    assert!(
        bc > 0,
        "controle: o passo do fixture nao retem nada, entao a comparacao de bytes nao diz nada"
    );
    assert!(
        bj <= bc + bc / 8,
        "a rota do journal retem {bj} bytes contra {bc} da classica (>12,5% a mais): a janela dele \
         nao foi apertada pela declarada, e o delta perde a profundidade de undo que a §5.28 mediu"
    );
}

/// **Sem proveniência o journal NÃO é consultado** — a metade que torna a migração *lenta nunca,
/// errada jamais*.
///
/// Um `before` que não abriu o passo (uma transação aninhada, um layer op atravessando um traço) vê um
/// journal ancorado noutro ponto; usá-lo daria o lado `before` de um passado que não é o dele. O guard
/// recusa e o commit deriva como sempre.
///
/// ⚠️ **Mutação que sangra:** fazer o
/// [`journal_describes_step_at`](crate::undo::window::WriteState::journal_describes_step_at) devolver
/// `true` sempre — o contador passa a subir com um `before` que o journal não descreve.
#[test]
fn a_before_the_journal_does_not_describe_never_reaches_the_journal_route() {
    let t = tool_mid_step(512);
    let (before, after) = endpoints(&t);
    let (mut b, mut a) = (before.clone(), after.clone());
    let seen = RELIEF_FROM_JOURNAL.with(std::cell::Cell::get);
    let _ = PlaneDeltas::split(
        &mut b,
        &mut a,
        None,
        Some(ReliefSource {
            state: &t.undo.write_state,
            // O passo foi aberto com o contador do `before`; qualquer outro valor descreve outro passo.
            writes: before.writes.wrapping_add(1),
            layer: t.layers.active().expect("a camada ativa do fixture"),
        }),
    );
    assert_eq!(
        RELIEF_FROM_JOURNAL.with(std::cell::Cell::get),
        seen,
        "o journal respondeu por um passo que nao e' o dele — a proveniencia nao esta' guardando nada"
    );
}

/// **Os journals de OUTRA camada não respondem** — a lei que sustenta o `Unchanged` das demais.
///
/// O [`StoredMap::from_journal`](crate::undo_delta::StoredMap::from_journal) declara as outras camadas
/// inalteradas *por lei* (uma camada por passo, toda escrita por porta nomeada). Se o guard aceitasse
/// uma camada de que os journals não falam, essa lei viraria uma afirmação sobre bytes que ninguém
/// olhou.
#[test]
fn the_journals_refuse_to_answer_for_a_layer_they_did_not_capture() {
    let t = tool_mid_step(512);
    let active = t.layers.active().expect("a camada ativa do fixture");
    let other = crate::layers::LayerId(active.0.wrapping_add(1));
    let (before, after) = endpoints(&t);
    let (mut b, mut a) = (before.clone(), after);
    let seen = RELIEF_FROM_JOURNAL.with(std::cell::Cell::get);
    let _ = PlaneDeltas::split(
        &mut b,
        &mut a,
        None,
        Some(ReliefSource {
            state: &t.undo.write_state,
            writes: before.writes,
            layer: other,
        }),
    );
    assert_eq!(
        RELIEF_FROM_JOURNAL.with(std::cell::Cell::get),
        seen,
        "os journals responderam por uma camada que eles nao capturaram (`speaks_for` nao esta' \
         guardando nada) — e os bytes de um plano de outra camada tem a MESMA forma, entao a troca \
         seria silenciosa"
    );
}
