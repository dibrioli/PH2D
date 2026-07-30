//! Gates do **WIDTH TOOL** — arquivo irmão de `width_handles.rs` (plano 25 §5).
//!
//! Os oráculos são do GESTO, não da fórmula: *a alça pousa na borda da fita*, *arrastar para fora
//! engrossa*, *o clique na curva não faz o desenho saltar*. Um gate que recomputasse a normal para
//! conferir a normal seria o espelho sempre-verde que esta linha já pegou.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex};

/// Uma linha RETA horizontal, com traço, posada fora da origem — a fixture atravessa a fronteira
/// local↔mundo, e numa reta a normal é constante: o que se mede é a alça, não a geometria.
fn line_scene() -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0]].map(VecVertex::corner).to_vec(),
        closed: false,
        ..VecPath::default()
    };
    p.stroke = Some(StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.4));
    let id = scene.push_path(p);
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(3.0, -2.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("Traco"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id)
}

/// Um perfil de teste, pela face de preset.
fn taper(start: f64, mid: f64, end: f64) -> WidthStops {
    ph2d_vec_scene::WidthProfile {
        start,
        mid,
        end,
        position: 0.5,
    }
    .to_stops()
}

/// **A alça pousa na BORDA da fita.** Ela é a manipulação direta de uma largura, então tem de
/// estar onde a tinta acaba. ⚠️ **Reescrito depois do report do Enio (2026-07-30)**: a FICHA
/// mudou-se para a curva e a HASTE é que vai à borda — as duas metades são afirmadas aqui, e
/// juntas são mais fortes que a asserção antiga (que só media a borda).
///
/// A ficha na borda era o defeito: a borda é `meia-largura × multiplicador` fora da curva, e num
/// multiplicador alto isso pousa **em cima da linha vizinha** — medido, `3,75 × 0,08 = 0,30`, a
/// distância exata entre os braços de um grampo. O artista clicava numa linha e via a alça nascer
/// na outra.
#[test]
fn the_handle_sits_on_the_curve_and_its_arm_reaches_the_ribbon_edge() {
    let (scene, sim, map, id) = line_scene();
    let hs = handles(&sim, &scene, &map, id);
    assert_eq!(hs.len(), 2, "o neutro tem duas paradas: {hs:?}");
    for h in &hs {
        // A FICHA está SOBRE a curva (a reta vive em y = -2 de mundo). É isto que torna
        // impossível uma alça pousar numa linha que não é a dela.
        assert!(
            (h.at[1] + 2.0).abs() < 1e-9,
            "a ficha nao esta sobre a curva: {h:?}"
        );
        // E a HASTE alcança a borda da fita — meia-largura (0,2) pela normal.
        assert!(
            ((h.tip[1] - h.at[1]).abs() - 0.2).abs() < 1e-9,
            "a haste nao alcanca a borda da fita: {h:?}"
        );
    }
    assert!(
        (hs[0].at[0] - 3.0).abs() < 1e-9,
        "a 1a alca nao esta no comeco"
    );
    assert!((hs[1].at[0] - 7.0).abs() < 1e-9, "a ultima nao esta no fim");
}

/// **Uma forma sem traço não tem largura a editar** — e a resposta é nenhuma alça, não uma alça
/// colada à curva que não faria nada.
#[test]
fn a_shape_without_a_stroke_has_no_handles() {
    let (mut scene, sim, map, id) = line_scene();
    scene.path_mut(id).expect("existe").stroke = None;
    assert!(handles(&sim, &scene, &map, id).is_empty());
}

/// **Arrastar para FORA engrossa; para dentro afina.** É o gesto inteiro numa frase, e a direção
/// invertida faria a ferramenta desenhar o oposto do que a mão pede.
#[test]
fn dragging_outward_thickens_and_inward_thins() {
    let (scene, mut sim, map, id) = line_scene();
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[0].at, 0.05).expect("agarrou a 1a alca");
    // ⚠️ O lado sai da PONTA da haste: a ficha está sobre a curva, e um `signum` de zero
    // daria um alvo de arrasto em cima da própria curva (multiplicador zero, gate verde por
    // acidente).
    let side = (hs[0].tip[1] + 2.0).signum();

    drag(&mut sim, &scene, &map, grab, [3.0, -2.0 + side * 0.4]);
    let thick = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert!(
        (thick.at(0.0) - 2.0).abs() < 1e-6,
        "afastar nao engrossou: {:.3}",
        thick.at(0.0)
    );

    drag(&mut sim, &scene, &map, grab, [3.0, -2.0 + side * 0.1]);
    let thin = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert!(
        (thin.at(0.0) - 0.5).abs() < 1e-6,
        "aproximar nao afinou: {:.3}",
        thin.at(0.0)
    );
}

/// **O lado não importa: a distância é ABSOLUTA.** A alça vive de um lado só; deixar o
/// multiplicador ficar negativo do outro viraria a fita do avesso, e uma largura negativa não é
/// uma largura.
#[test]
fn crossing_to_the_other_side_does_not_invert_the_ribbon() {
    let (scene, mut sim, map, id) = line_scene();
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[0].at, 0.05).expect("agarrou");
    let side = (hs[0].tip[1] + 2.0).signum();
    drag(&mut sim, &scene, &map, grab, [3.0, -2.0 - side * 0.4]);
    let st = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert!(
        st.at(0.0) > 0.0,
        "atravessar a curva deu multiplicador nao-positivo: {:.3}",
        st.at(0.0)
    );
}

/// **Arrastar ao longo da curva MOVE a parada.** O dedo aponta um lugar, e o lugar responde as
/// duas perguntas (que espessura, e onde) — separá-las pediria ao artista que soubesse qual metade
/// da alça ele está a mover.
#[test]
fn dragging_along_the_curve_moves_the_stop() {
    let (scene, mut sim, map, id) = line_scene();
    // ⚠️ A fixture parte de um perfil NÃO-uniforme de propósito: mover uma parada de um perfil
    // uniforme deixa-o uniforme, e um perfil uniforme não é guardado (o neutro-é-ausência). O
    // gate mediria a ausência e falharia sobre produto correto.
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.4, 1.4, 0.4));
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[0].at, 0.05).expect("agarrou");
    let side = (hs[0].tip[1] + 2.0).signum();
    drag(&mut sim, &scene, &map, grab, [5.0, -2.0 + side * 0.2]);
    let st = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    let moved = st.as_slice()[grab.stop].pos;
    assert!(
        (moved - 0.5).abs() < 1e-6,
        "a parada nao andou para o meio: {moved:.3}"
    );
}

/// **Clicar na curva ACRESCENTA uma parada com o multiplicador que o perfil já tem ali** — a
/// espessura NO PONTO CLICADO não muda, e o arrasto seguinte é que a move. Uma parada que
/// nascesse em `1.0` faria a fita saltar sob o dedo antes de o artista pedir qualquer coisa.
///
/// ⚠️ **O que NÃO é preservado é a forma entre as paradas vizinhas, e isso é uma propriedade da
/// representação — não um defeito a consertar.** O `smoothstep` liga paradas CONSECUTIVAS, então
/// inserir uma re-parametriza os dois vãos que ela divide: medido, o desvio máximo num afinamento
/// de ponta é **0,058 de multiplicador** (~7% da faixa, ~0,012 unidade de mundo num traço de
/// 0,4 — sub-pixel no zoom de trabalho). Trocar por interpolação LINEAR tornaria a inserção
/// exata e poria um VINCO em cada parada, que é o que o `WidthProfile` recusou desde o 1º dia;
/// o trade está tomado, e é este gate que o pina para ninguém "consertá-lo" de volta.
#[test]
fn clicking_the_curve_adds_a_stop_at_the_thickness_it_already_had() {
    let (scene, mut sim, map, id) = line_scene();
    let t = taper(1.0, 1.0, 0.2);
    crate::profile_live::arm(&mut sim, &map, &[id], &t);

    let n0 = t.as_slice().len();
    let grab = press(&mut sim, &scene, &map, id, [6.0, -2.0], 0.05).expect("acrescentou");
    let after = crate::profile_live::spec_of(&sim, &map, id).expect("armado");
    assert_eq!(
        after.as_slice().len(),
        n0 + 1,
        "o clique na curva nao acrescentou parada"
    );
    // A parada nova está onde o dedo apontou, e com a espessura que havia ali.
    let st = after.as_slice()[grab.stop];
    assert!(
        (st.pos - 0.75).abs() < 1e-6,
        "a parada nasceu em {:.3}",
        st.pos
    );
    assert!(
        (st.mult - t.at(0.75)).abs() < 1e-9,
        "a espessura no ponto clicado MUDOU: {:.4} contra {:.4}",
        st.mult,
        t.at(0.75)
    );
    // E o resto do perfil segue o mesmo perfil — o desvio é o da re-parametrização, nomeado
    // acima, e não uma forma diferente.
    let worst = (0..=100)
        .map(|k| {
            let x = f64::from(k) / 100.0;
            (after.at(x) - t.at(x)).abs()
        })
        .fold(0.0_f64, f64::max);
    // ⚠️ **13,1% da faixa, e é ESTRUTURAL** — medido nos quatro perfis do sweep, sempre o mesmo
    // (é o máximo entre um smoothstep e dois meio-smoothsteps, não um acidente de dados). O
    // número está aqui para ninguém o descobrir de novo, e a razão de o artista nunca o ver é o
    // `Grab::created`: um clique que não arrastou é desfeito no release.
    let range = 1.0 - 0.2;
    assert!(
        worst / range < 0.14,
        "inserir uma parada mudou a forma em {worst:.4} ({:.1}% da faixa) — mais que a \
         re-parametrizacao estrutural de 13,1%",
        100.0 * worst / range
    );
}

/// **Um clique que NÃO arrastou não deixa nada** — é o que torna os 13,1% da re-parametrização
/// invisíveis. Com o Width Tool cria-se um ponto de largura ARRASTANDO a partir da curva; um
/// clique solto é um clique solto.
#[test]
fn a_click_that_never_dragged_leaves_the_profile_untouched() {
    let (scene, mut sim, map, id) = line_scene();
    let t = taper(1.0, 1.0, 0.2);
    crate::profile_live::arm(&mut sim, &map, &[id], &t);
    let before: Vec<f64> = (0..=20).map(|k| t.at(f64::from(k) / 20.0)).collect();

    let grab = press(&mut sim, &scene, &map, id, [6.0, -2.0], 0.05).expect("acrescentou");
    assert!(grab.created, "a parada nasceu neste gesto");
    discard_if_untouched(&mut sim, &map, grab);

    let after = crate::profile_live::spec_of(&sim, &map, id).expect("o perfil sobreviveu");
    assert_eq!(after.as_slice().len(), t.as_slice().len());
    for (k, b) in before.iter().enumerate() {
        let x = f64::from(u8::try_from(k).unwrap_or(0)) / 20.0;
        assert!(
            (after.at(x) - b).abs() < 1e-12,
            "t={x}: o clique solto mexeu no desenho ({:.5} contra {b:.5})",
            after.at(x)
        );
    }
}

/// **Uma alça AGARRADA não é desfeita.** Ela já existia antes do gesto; tratá-la como recém-criada
/// apagaria uma parada do artista a cada clique que não arrastasse.
#[test]
fn grabbing_an_existing_handle_is_never_discarded() {
    let (scene, mut sim, map, id) = line_scene();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.4, 1.4, 0.4));
    let hs = handles(&sim, &scene, &map, id);
    let grab = press(&mut sim, &scene, &map, id, hs[1].at, 0.05).expect("agarrou");
    assert!(!grab.created, "uma alca agarrada nao 'nasceu' no gesto");
    discard_if_untouched(&mut sim, &map, grab);
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("intacto")
            .as_slice()
            .len(),
        3
    );
}

/// **Um clique LONGE da curva não faz nada.** Sem isto todo clique no vazio acrescentaria uma
/// parada na projeção mais próxima, e o artista acumularia paradas que não pediu.
#[test]
fn clicking_far_from_the_curve_does_nothing() {
    let (scene, mut sim, map, id) = line_scene();
    assert!(press(&mut sim, &scene, &map, id, [5.0, 5.0], 0.05).is_none());
    assert!(crate::profile_live::spec_of(&sim, &map, id).is_none());
}

/// **O botão direito APAGA a parada sob a alça**, e abaixo de duas o perfil inteiro sai — o traço
/// volta ao uniforme em vez de ficar com uma parada solta a governar a largura por um caminho que
/// mais nenhuma rota usa.
#[test]
fn removing_below_two_stops_clears_the_profile() {
    let (scene, mut sim, map, id) = line_scene();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.3, 1.4, 0.3));

    let hs = handles(&sim, &scene, &map, id);
    assert_eq!(hs.len(), 3);
    assert!(
        remove(&mut sim, &scene, &map, id, hs[1].at, 0.05),
        "nao apagou"
    );
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("ainda ha perfil")
            .as_slice()
            .len(),
        2
    );
    let hs = handles(&sim, &scene, &map, id);
    assert!(remove(&mut sim, &scene, &map, id, hs[0].at, 0.05));
    assert!(
        crate::profile_live::spec_of(&sim, &map, id).is_none(),
        "sobrou um perfil de uma parada so"
    );
}

/// **O direito longe de uma alça não apaga nada** — o mesmo cuidado do clique no vazio.
#[test]
fn removing_far_from_a_handle_does_nothing() {
    let (scene, mut sim, map, id) = line_scene();
    crate::profile_live::arm(&mut sim, &map, &[id], &taper(0.3, 1.4, 0.3));
    assert!(!remove(&mut sim, &scene, &map, id, [5.0, 5.0], 0.05));
    assert_eq!(
        crate::profile_live::spec_of(&sim, &map, id)
            .expect("intacto")
            .as_slice()
            .len(),
        3
    );
}

/// **A alça segue a POSE.** A forma é desenhada onde a entidade a põe, e uma alça que ignorasse o
/// `Transform` pousaria longe da tinta assim que o artista movesse a forma.
#[test]
fn the_handles_follow_the_shapes_pose() {
    let (scene, mut sim, map, id) = line_scene();
    let before = handles(&sim, &scene, &map, id);
    let e = ph2d_ecs::Entity::from_bits(*map.get(&id).expect("mapeada"));
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.translation.x += 5.0;
    }
    let after = handles(&sim, &scene, &map, id);
    assert_eq!(before.len(), after.len());
    for (a, b) in before.iter().zip(&after) {
        assert!(
            (b.at[0] - a.at[0] - 5.0).abs() < 1e-9,
            "a alca andou {:.3} em vez dos 5,0 da pose",
            b.at[0] - a.at[0]
        );
    }
}

/// **A alça NÃO escala com a pose**, porque a fita também não: o `bake_xform` transforma pontos e
/// comprimentos de path e deixa `stroke.width` como está, então o `power_stroke` molda a fita na
/// largura autorada mesmo sob uma pose escalada. Uma alça que multiplicasse pela escala pousaria
/// fora da tinta — as duas TÊM de concordar.
#[test]
fn the_handle_offset_does_not_scale_with_the_pose() {
    let (scene, mut sim, map, id) = line_scene();
    let plain = handles(&sim, &scene, &map, id)[0].tip[1];
    let e = ph2d_ecs::Entity::from_bits(*map.get(&id).expect("mapeada"));
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.scale = ph2d_core::Vec2::new(3.0, 3.0);
    }
    let scaled = handles(&sim, &scene, &map, id)[0].tip[1];
    assert!(
        (scaled - plain).abs() < 1e-9,
        "o desvio da alca escalou com a pose: {plain:.4} -> {scaled:.4}"
    );
}

/// Um **GRAMPO**: dois braços quase paralelos, a `0,30` um do outro, com traço `0,16`. É a
/// fixture do report do Enio (2026-07-30) — *"linhas muito próximas ou cruzadas"* —, e o número
/// que a torna o fenômeno é a razão entre o espaçamento e a largura: `0,30 / 0,08` = um
/// multiplicador de `3,75` põe a borda da fita **exatamente sobre o braço vizinho**.
fn hairpin() -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let mut p = VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 0.30], [0.0, 0.30]]
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
