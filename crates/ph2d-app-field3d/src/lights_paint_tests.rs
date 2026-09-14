//! Os gates da MARCA de uma luz — ver [`super`].

use super::*;
use crate::lights::{MARK_GRAB_PX, SceneLight, marks, under};

fn uma(bits: u64, world: [f32; 3], on: bool) -> SceneLight {
    SceneLight {
        bits,
        world,
        light: ph2d_field_ecs::FieldLight::default(),
        on,
    }
}

fn palco() -> (ph2d_field_render::Orbit, ph2d_field_render::Screen) {
    let cam = ph2d_field_render::Orbit::default();
    (
        cam,
        ph2d_field_render::Screen::new(400, 300, cam.half_extent),
    )
}

/// ⭐⭐⭐ **O CLIQUE APANHA A MARCA ONDE ELA É DESENHADA** — a lei inteira desta wave, e a única que
/// nenhum dos dois lados consegue afirmar sozinho.
///
/// # ⚠️ Porque ela é medida pelo PAR
///
/// Um gate sobre o pintor diz *«desenhei alguma coisa»*; um sobre o teste de acerto diz *«apanhei
/// alguma coisa»*. **As duas passam sobre uma marca desenhada num sítio e apanhada noutro**, que é
/// o defeito que se lê como *«o clique não pega»*. ⇒ o que se afirma é que o ponto que o pintor
/// recebe é o ponto onde o clique acerta.
#[test]
fn the_click_finds_the_mark_where_it_is_drawn() {
    let (cam, screen) = palco();
    let luzes = [uma(7, [0.5, 0.4, 0.2], true)];
    let m = marks(&luzes, &cam, screen);
    assert_eq!(m.len(), 1, "a luz não projectou");
    let centro = m[0].px;
    assert_eq!(
        under(&luzes, &cam, screen, centro),
        Some(7),
        "o clique no centro do desenho não apanhou a marca"
    );
    // ⭐ **E o agarre ACABA** — sem esta metade, um `under` que devolvesse sempre a primeira luz
    // passaria: a marca engoliria o clique na peça inteira.
    let longe = [centro[0] + MARK_GRAB_PX + 2.0, centro[1]];
    assert_eq!(
        under(&luzes, &cam, screen, longe),
        None,
        "o agarre não acaba: um clique a {MARK_GRAB_PX} + 2 px ainda é da marca"
    );
    // ⭐ E a borda do agarre ainda é dela.
    assert_eq!(
        under(
            &luzes,
            &cam,
            screen,
            [centro[0] + MARK_GRAB_PX - 0.5, centro[1]]
        ),
        Some(7)
    );
}

/// ⭐⭐ **A MAIS PRÓXIMA NO ECRÃ ganha** — com duas marcas ao alcance, a resposta não pode depender
/// da ordem da lista.
///
/// # ⚠️⚠️ A primeira fixtura desta não distinguia nada, e uma mutação provou-o
///
/// Ela punha as duas luzes em lados opostos da peça: ao clicar numa, a outra caía **fora do
/// agarre**, logo o filtro já a tinha deitado fora e trocar o `min_by` por um `next()` devolvia o
/// mesmo. *Uma fixtura em que só um candidato sobrevive ao filtro não testa o critério de desempate.*
///
/// ⇒ as duas ficam **as duas dentro do agarre** do mesmo clique, e o gate afirma isso primeiro.
#[test]
fn the_nearest_mark_wins_and_not_the_first() {
    let (cam, screen) = palco();
    // Perto uma da outra: o que decide passa a ser a distância, e não o filtro.
    let luzes = [
        uma(1, [0.055, 0.0, 0.0], true),
        uma(2, [0.02, 0.0, 0.0], true),
    ];
    let m = marks(&luzes, &cam, screen);
    assert_eq!(m.len(), 2);
    let clique = m[1].px;
    let d0 = (m[0].px[0] - clique[0]).hypot(m[0].px[1] - clique[1]);
    assert!(
        d0 < MARK_GRAB_PX && d0 > 1.0,
        "a fixtura não põe as duas ao alcance ({d0:.2} px contra um agarre de {MARK_GRAB_PX}) — \
         este gate não testaria o desempate"
    );
    assert_eq!(under(&luzes, &cam, screen, clique), Some(2));
    assert_eq!(under(&luzes, &cam, screen, m[0].px), Some(1));
}

/// ⭐⭐ **UMA LUZ APAGADA CONTINUA A TER MARCA** — a ordem do dono da §23, aplicada aqui: *«não devem
/// desaparecer, mas apenas serem inativados, mas sempre visíveis»*.
///
/// ⛔ Sem isto, apagar uma luz pelo olho da Hierarquia tirava-a do canvas, e voltar a acendê-la só
/// era possível pela Hierarquia — que é de onde esta wave a tirou.
#[test]
fn a_light_that_is_off_still_has_a_mark() {
    let (cam, screen) = palco();
    let apagada = [uma(3, [0.5, 0.4, 0.2], false)];
    assert_eq!(marks(&apagada, &cam, screen).len(), 1);
    assert_eq!(
        under(&apagada, &cam, screen, marks(&apagada, &cam, screen)[0].px),
        Some(3)
    );
    // ⭐ E ela NÃO acende — o olho é sobre a luz, não sobre a marca.
    assert!(crate::lights::lamps_of(&apagada).is_empty());

    // ⭐⭐ **E ela pinta-se DIFERENTE de uma acesa.** ⚠️ Sem esta metade, uma mutação que apagasse o
    // esmaecido sobrevivia: *«a marca continua lá»* é satisfeito por uma marca que MENTE sobre o
    // estado da luz — e o artista concluiria que ela está acesa e que o defeito é outro.
    let acesa = [uma(3, [0.5, 0.4, 0.2], true)];
    let tinta = |l: &[SceneLight]| {
        let mut scene = ph2d_vector::VectorScene::new();
        paint(
            &mut scene,
            &marks(l, &cam, screen),
            None,
            ph2d_tokens::Theme::default(),
            [0.0, 0.0],
        );
        format!("{:?}", scene.inner().encoding().draw_data)
    };
    assert_ne!(
        tinta(&apagada),
        tinta(&acesa),
        "a marca de uma luz apagada pinta-se igual à de uma acesa"
    );
}

/// ⭐⭐⭐ **A MARCA É PINTADA** — a costura, medida na geometria que sai para a cena.
///
/// ⚠️ **O controlo é pintar SEM marca nenhuma:** sem ele, este gate afirmaria que uma cena com
/// alguma coisa dentro tem bytes, que é verdade de toda cena.
#[test]
fn the_mark_puts_geometry_in_the_scene() {
    let pinta = |luzes: &[SceneLight]| {
        let (cam, screen) = palco();
        let m = marks(luzes, &cam, screen);
        let mut scene = ph2d_vector::VectorScene::new();
        paint(
            &mut scene,
            &m,
            None,
            ph2d_tokens::Theme::default(),
            [0.0, 0.0],
        );
        scene.inner().encoding().path_data.clone()
    };
    let vazio = pinta(&[]);
    let uma_luz = pinta(&[uma(1, [0.5, 0.4, 0.2], true)]);
    let duas = pinta(&[
        uma(1, [0.5, 0.4, 0.2], true),
        uma(2, [-0.5, 0.0, 0.0], true),
    ]);
    assert!(
        uma_luz.len() > vazio.len(),
        "uma luz não pôs geometria nenhuma na cena ({} contra {})",
        vazio.len(),
        uma_luz.len()
    );
    assert!(
        duas.len() > uma_luz.len(),
        "a segunda luz não acrescentou nada — o pintor desenha uma e pára"
    );
}

/// ⭐⭐ **A ESCOLHIDA pinta-se diferente** — é o que diz ao artista qual das lâmpadas o gizmo comanda.
#[test]
fn the_selected_mark_is_painted_differently() {
    let (cam, screen) = palco();
    let luzes = [uma(9, [0.5, 0.4, 0.2], true)];
    let m = marks(&luzes, &cam, screen);
    let pinta = |sel: Option<u64>| {
        let mut scene = ph2d_vector::VectorScene::new();
        paint(
            &mut scene,
            &m,
            sel,
            ph2d_tokens::Theme::default(),
            [0.0, 0.0],
        );
        // ⚠️ **As CORES e não a geometria**: a marca escolhida tem a mesma forma, e comparar
        // `path_data` daria dois iguais sobre um defeito real.
        format!("{:?}", scene.inner().encoding().draw_data)
    };
    assert_ne!(
        pinta(None),
        pinta(Some(9)),
        "escolher a luz não mudou uma tinta — nada na tela diz qual o gizmo comanda"
    );
    assert_eq!(
        pinta(Some(1234)),
        pinta(None),
        "uma entidade que não é esta marca realçou-a — o realce não está a comparar quem"
    );
}

/// ⭐⭐⭐ **A COSTURA, das duas pontas** — o quadro do produto DESENHA a marca, e um clique nela
/// ESCOLHE a luz.
///
/// # ⚠️ Porque as duas metades vivem no mesmo gate
///
/// Os gates acima chamam o pintor e o teste de acerto **directamente**, e um pintor correcto que
/// ninguém chama produz exactamente o mesmo app que um pintor ausente. *É a lição que este módulo
/// já pagou com o `_alpha` do céu (§16) e com o laço.*
///
/// ⚠️⚠️ **E a luz é uma de VERDADE, no mundo.** Semear `s.lights` à mão não serviria: o
/// [`crate::lights::sync`] corre dentro do `ecs_bridge` e reescreve a lista a partir do mundo — o
/// gate estaria a medir a sua própria semente, e a varredura podia estar partida sem ninguém ver.
#[test]
fn the_frame_paints_the_mark_and_a_click_on_it_picks_the_light() {
    use crate::scene::SelectRequest;
    use crate::scene::lasso_tests::{AREA, armed_with};
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.3 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a peça");
    armed_with(&doc, |sim| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        let mut pinta = || {
            crate::smoke::with_smoke(|s| s.gizmo = None);
            let mut scene = ph2d_vector::VectorScene::new();
            crate::smoke::draw(AREA, ph2d_tokens::Theme::default(), &mut text, &mut scene);
            scene.inner().encoding().path_data.clone()
        };
        let sem = pinta();

        // ⭐ Uma luz de VERDADE, **fora** da peça (que tem raio `0,3`).
        let bits = ph2d_field_ecs::add_light(
            sim.world_mut(),
            [0.55, 0.45, 0.0],
            ph2d_field_ecs::FieldLight::default(),
        )
        .to_bits();
        // Um quadro para a varredura a apanhar.
        let _ = crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        let recolhidas = crate::smoke::with_smoke(|s| s.lights.clone()).expect("smoke armado");
        // ⭐⭐ **São DUAS**, e a primeira é a que a CENA trouxe: o `sync_scene` planta uma luz com a
        // peça (§25.3). *A fixtura prova a lei de abertura de lado, e sem ela este assert leria `1`
        // e eu teria concluído que a varredura funcionava por outra razão.*
        let quais: Vec<u64> = recolhidas.iter().map(|l| l.bits).collect();
        assert_eq!(quais.len(), 2, "luzes recolhidas: {quais:?}");
        assert!(
            quais.contains(&bits),
            "a varredura não apanhou a luz que acabou de nascer: {quais:?}"
        );

        let com = pinta();
        assert!(
            com.len() > sem.len(),
            "o quadro não desenhou a marca da luz ({} contra {}) — o artista não tem o que apontar",
            sem.len(),
            com.len()
        );

        // ── E o clique ────────────────────────────────────────────────────────────────────────
        let (cam, screen) = crate::smoke::with_smoke(|s| {
            (s.vp().cam, crate::input::area_screen(s).expect("écran"))
        })
        .expect("smoke armado");
        let m = marks(&recolhidas, &cam, screen);
        let minha = m
            .iter()
            .find(|k| k.light.bits == bits)
            .expect("a marca da luz que eu criei");
        crate::smoke::with_smoke(|s| s.pending_pick = Some((minha.px, false)));
        let req = crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        assert!(
            matches!(req, Some(SelectRequest::Entity(b)) if b == bits),
            "o clique na marca não escolheu a luz: {req:?}"
        );
        // ⭐ **E o meio da peça continua a ser a peça** — a marca é um sobreposto, não uma manta.
        // ⛔ Sem esta metade, um `under` que devolvesse sempre a luz passaria, e o artista deixaria
        // de conseguir escolher a forma.
        crate::smoke::with_smoke(|s| s.pending_pick = Some(([AREA.w * 0.5, AREA.h * 0.5], false)));
        let req = crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        assert!(
            matches!(req, Some(SelectRequest::Entity(b)) if b != bits),
            "um clique no meio da peça escolheu a luz: {req:?}"
        );
    });
}
