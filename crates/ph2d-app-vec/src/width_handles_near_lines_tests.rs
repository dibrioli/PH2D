//! Gates do **WIDTH TOOL em linhas PRÓXIMAS ou CRUZADAS** — filho de `width_handles_tests.rs`
//! pelo teto de 600 LOC da shell, e o corte é por responsabilidade: aqui mora a família do report
//! do Enio (2026-07-30), onde o que se prova é *qual* linha o gesto resolve; lá, o que a alça é e
//! o que o arrasto faz com ela.
//!
//! ⚠️ **Módulo FILHO** (`#[path]` de dentro do irmão), então `use super::*` alcança as fixtures e
//! os itens privados do tool — um módulo de topo obrigaria a tornar `handles`/`press` públicos só
//! para os testes.

use super::*;

/// Um **GRAMPO**: dois braços quase paralelos, a `0,30` um do outro, com traço `0,16`. É a
/// fixture do report do Enio (2026-07-30) — *"linhas muito próximas ou cruzadas"* —, e o número
/// que a torna o fenômeno é a razão entre o espaçamento e a largura: `0,30 / 0,08` = um
/// multiplicador de `3,75` põe a borda da fita **exatamente sobre o braço vizinho**.
fn hairpin() -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    hairpin_gap(0.30)
}

/// O mesmo grampo com o vão dado — **o vão é o que decide se o fenômeno existe**: só abaixo do
/// raio de hit-test (`0,25` nestes gates) uma alça de um braço cai dentro do alcance de um clique
/// dirigido ao outro.
fn hairpin_gap(gap: f64) -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, gap], [0.0, gap]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.16));
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Grampo"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id)
}

/// **Entre duas linhas próximas nasce UMA alça, e na linha que o rato apontou** (report do Enio,
/// 2026-07-30: *"cria-se duas alças, 1 em cada segmento próximo — deveria criar apenas uma alça
/// na linha mais próxima do mouse"*).
///
/// ⚠️ **A escolha da linha nunca esteve errada** — o `closest_arc` já devolvia o braço mais
/// próximo. Errado estava o DESENHO: a ficha ficava na borda da fita, a `meia-largura ×
/// multiplicador` da curva, e num grampo isso a punha em cima do braço vizinho. MEDIDO com a
/// ficha na borda: clicar o braço de BAIXO e puxar produzia ficha em `y = 0,300` — o braço de
/// CIMA, ao milésimo. Com a ficha na curva: `y = 0,000`.
///
/// O gate afirma as DUAS metades, porque são dois modos de falha independentes: *uma* parada
/// (não duas), e a ficha dela **na linha certa**.
#[test]
fn a_handle_is_born_on_the_line_the_mouse_pointed_at() {
    let (scene, mut sim, map, id) = hairpin();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(1.0, 1.6, 1.0));
    let n0 = handles(&sim, &scene, &map, id).len();

    // Um clique no braço de BAIXO (y = 0), arrastado para longe dele.
    let grab = press(&mut sim, &scene, &map, id, [2.0, 0.04], 0.25).expect("pegou a curva");
    drag(&mut sim, &scene, &map, grab, [2.0, -0.3]);
    let hs = handles(&sim, &scene, &map, id);
    assert_eq!(
        hs.len(),
        n0 + 1,
        "um clique entre duas linhas proximas criou {} alcas em vez de UMA",
        hs.len() - n0
    );

    let born = hs[grab.stop];
    let to_bottom = (born.at[1] - 0.0).abs();
    let to_top = (born.at[1] - 0.30).abs();
    assert!(
        to_bottom < to_top,
        "a alca nasceu no braco ERRADO: ficha em {:?} (a {to_bottom:.3} do braco clicado, a \
         {to_top:.3} do vizinho) -- e' o report do Enio: o artista clica numa linha e a alca \
         aparece na de ao lado",
        born.at
    );
}

/// **Clicar de novo na MESMA linha agarra a alça que já está lá** — a outra metade do report. Com
/// a ficha na borda ela sentava sobre o braço vizinho, então o 2º clique na linha certa não a
/// encontrava, criava OUTRA parada, e o artista ficava com uma alça em cada segmento.
#[test]
fn a_second_click_on_the_same_line_grabs_the_handle_that_is_already_there() {
    let (scene, mut sim, map, id) = hairpin();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(1.0, 1.6, 1.0));
    let g1 = press(&mut sim, &scene, &map, id, [2.0, 0.04], 0.25).expect("pegou a curva");
    drag(&mut sim, &scene, &map, g1, [2.0, -0.3]);
    let n = handles(&sim, &scene, &map, id).len();

    let g2 = press(&mut sim, &scene, &map, id, [2.0, 0.01], 0.25).expect("pegou algo");
    assert!(
        !g2.created,
        "o 2o clique no mesmo sitio criou uma parada NOVA em vez de agarrar a que la' esta'"
    );
    assert_eq!(
        handles(&sim, &scene, &map, id).len(),
        n,
        "a contagem de alcas mudou num clique que devia so' agarrar"
    );
}

/// **O 1º gesto numa forma VIRGEM acrescenta uma parada — não sequestra a do FIM.**
///
/// ⚠️ Achado pela sonda do report do Enio, e é um 2º defeito independente do que ele descreveu.
/// A parada criada nasce com o multiplicador que o perfil já tem ali (para o desenho não saltar),
/// então sobre o NEUTRO a lista continua uniforme — e o `arm` remove um perfil uniforme (a lei
/// deste módulo). O `press` devolvia então um índice para uma lista que nunca foi guardada, e o
/// `drag` relia o neutro (duas paradas) editando a de índice 1: **a ponta do traço**. MEDIDO:
/// `[(0, 1), (1, 1)]` virava `[(0, 1), (0.241, 5)]` — o artista puxava no meio e o fim do traço
/// mudava de sítio, com a metade final a engrossar toda.
///
/// É o primeiro gesto que qualquer artista faz nesta ferramenta.
#[test]
fn the_first_gesture_on_a_virgin_shape_adds_a_stop_instead_of_hijacking_the_end() {
    let (scene, mut sim, map, id) = line_scene();
    assert!(
        crate::profile_live::spec_of(&sim, &map, id).is_none(),
        "a fixture tem de ser VIRGEM: e' o unico estado onde o defeito existe"
    );
    let grab = press(&mut sim, &scene, &map, id, [5.0, -2.0], 0.05).expect("pegou a curva");
    drag(&mut sim, &scene, &map, grab, [5.0, -1.6]);
    let st = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    let v = st.as_slice();
    assert_eq!(
        v.len(),
        3,
        "o 1o gesto numa forma virgem devia deixar TRES paradas (as duas do neutro + a nova); \
         deixou {}: {v:?}",
        v.len()
    );
    assert!(
        v.last().is_some_and(|s| (s.pos - 1.0).abs() < 1e-12),
        "a parada do FIM sumiu -- o arrasto editou-a em vez de acrescentar: {v:?}"
    );
    assert!(
        v.iter().any(|s| (s.pos - 0.5).abs() < 1e-6 && s.mult > 1.5),
        "a parada nova nao esta' onde o dedo apontou: {v:?}"
    );
}

/// **Uma alça numa linha não engole o clique dirigido à linha vizinha** (Enio, 2026-07-30:
/// *"próximo da linha de cima não consigo clicar na linha cruzada abaixo"*).
///
/// ⚠️ **É o gate que só a pergunta certa pode passar.** A busca antiga era no PLANO — *existe
/// alguma ficha a menos do raio?* — e com as linhas mais juntas que o raio isso é **indecidível**:
/// a alça de uma cai sempre dentro do alcance de um clique dirigido à outra. Nenhum ajuste do raio
/// salva; junto a um cruzamento a distância entre as linhas tende a zero.
///
/// ⚠️ **O vão da fixture é `0,15` contra um raio de `0,25`, e o número é o gate.** Com o vão de
/// `0,30` do irmão acima o fenômeno **não existe** (a alça cai fora do raio) e a mutação que repõe
/// a busca no plano passaria — foi o que aconteceu na 1ª tentativa, com um X cujo arrasto saltava
/// de perna. *A fixture tem de conter o fenômeno.*
///
/// Em ARCO as duas linhas estão a **meio traço** uma da outra (`0,509` de fração contra um alcance
/// de `0,031`), e é isso que torna a escolha decidível.
#[test]
fn a_handle_on_one_line_does_not_swallow_a_click_aimed_at_the_neighbour() {
    let (scene, mut sim, map, id) = hairpin_gap(0.15);
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(1.0, 1.6, 1.0));

    // Uma alça no braço de BAIXO (y = 0), arrastada para fora — ela FICA nesse braço.
    let g1 = press(&mut sim, &scene, &map, id, [2.0, 0.02], 0.25).expect("pegou o braco de baixo");
    drag(&mut sim, &scene, &map, g1, [2.0, -0.3]);
    let n = handles(&sim, &scene, &map, id).len();
    let born = handles(&sim, &scene, &map, id)[g1.stop];
    let pos1 = crate::profile_live::spec_of(&sim, &map, id)
        .expect("armado")
        .as_slice()[g1.stop]
        .pos;

    // O clique dirigido ao braço de CIMA. No PLANO ele está a 0,13 da ficha de baixo — dentro do
    // raio de 0,25, e é exatamente o clique que ela engolia.
    let aim = [2.0, 0.13];
    let d_plane = (born.at[0] - aim[0]).hypot(born.at[1] - aim[1]);
    assert!(
        d_plane <= 0.25,
        "a fixture nao contem o fenomeno: a alca de baixo esta' a {d_plane:.3} do clique, fora do \
         raio -- a busca no plano acertaria por acidente e a mutacao nao sangraria"
    );

    let g2 = press(&mut sim, &scene, &map, id, aim, 0.25).expect("pegou o braco de cima");
    assert!(
        g2.created,
        "o clique no braco de CIMA agarrou a alca do de BAIXO em vez de criar a dele -- e' o \
         report: nao se consegue clicar na linha vizinha"
    );
    assert_eq!(
        handles(&sim, &scene, &map, id).len(),
        n + 1,
        "a parada nova nao entrou"
    );
    let st = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    let pos2 = st.as_slice()[g2.stop].pos;
    assert!(
        (pos2 - pos1).abs() > 0.25,
        "a parada nova caiu no MESMO braco da alca antiga ({pos1:.3} vs {pos2:.3}) -- o clique nao \
         foi resolvido pela linha que o rato apontou"
    );
}

/// **O botão direito apaga a alça da linha que o rato apontou, não a da vizinha** — o report do
/// Enio na outra metade do gesto. Apagar tem a MESMA ambiguidade que agarrar, e errar aqui custa
/// mais: some uma parada que o artista nem estava a olhar.
#[test]
fn the_right_click_never_deletes_the_neighbours_handle() {
    let (scene, mut sim, map, id) = hairpin_gap(0.15);
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(1.0, 1.6, 1.0));
    let g = press(&mut sim, &scene, &map, id, [2.0, 0.02], 0.25).expect("pegou o braco de baixo");
    drag(&mut sim, &scene, &map, g, [2.0, -0.3]);
    let n = crate::profile_live::spec_of(&sim, &map, id)
        .expect("armado")
        .as_slice()
        .len();

    // O direito mirado no braço de CIMA, onde NÃO há alça — mas onde a de baixo está a 0,13 no
    // plano, dentro do raio.
    assert!(
        !remove(&mut sim, &scene, &map, id, [2.0, 0.13], 0.25),
        "o direito no braco de CIMA apagou alguma coisa -- nao ha alca nenhuma ali"
    );
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("intacto")
            .as_slice()
            .len(),
        n,
        "o direito no braco vizinho apagou a alca do braco de baixo"
    );
}

/// **Com duas paradas dentro do alcance, agarra-se a MAIS PRÓXIMA** — não a primeira da lista.
/// A ordem é um acidente da ordenação por posição; agarrar pela ordem move um ponto que o artista
/// não apontou, e o desenho salta no sítio errado.
#[test]
fn two_stops_within_reach_grab_the_nearer_one() {
    let (scene, mut sim, map, id) = line_scene();
    // A reta mede 4 de mundo e o raio do gate é 0,05 ⇒ o alcance em arco é 0,0125. Duas paradas
    // a 0,01 uma da outra cabem as DUAS nele — é a fixture que contém o fenômeno.
    let close = WidthStops::new(vec![
        WidthStop {
            pos: 0.0,
            mult: 1.0,
        },
        WidthStop {
            pos: 0.50,
            mult: 1.4,
        },
        WidthStop {
            pos: 0.51,
            mult: 1.8,
        },
        WidthStop {
            pos: 1.0,
            mult: 1.0,
        },
    ]);
    crate::profile_live::arm(&mut sim, &map, &[id], &close);
    // O clique cai em pos ≈ 0,5075 — mais perto da SEGUNDA (0,51) que da primeira (0,50).
    // A reta vive em x ∈ [3, 7] de mundo, y = −2.
    let g = press(&mut sim, &scene, &map, id, [3.0 + 4.0 * 0.5075, -2.0], 0.05).expect("agarrou");
    assert!(!g.created, "devia ter agarrado uma parada que ja' existe");
    assert_eq!(
        g.stop,
        2,
        "agarrou a parada {} (pos {:.3}) em vez da MAIS PROXIMA (pos 0.510)",
        g.stop,
        close.as_slice()[g.stop].pos
    );
}
