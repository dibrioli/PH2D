//! **ONDE a trajetória é oferecida** — as duas metades da porta única `active_path`.
//!
//! Ela responde *"existe trajetória a mostrar AQUI?"*, e "aqui" tem dois eixos: o CLIP
//! (só o ativo, e só se ele autorou uma) e a ABA (só Keys). Os dois gates vivem juntos
//! porque são a MESMA porta e a mesma classe de defeito — *nada desenhado, tudo
//! agarrável* —, um pelo clip errado, outro pela aba errada.
//!
//! Módulo irmão do `motion_path_overlay_tests`, que mede *como* a trajetória é desenhada;
//! as fixtures são as DELE (`doc_with`/`doc_shaped`/`camera`/`window`), importadas em vez
//! de recopiadas — uma segunda fixture deriva, e aí os gates falariam de documentos
//! diferentes.

use super::marks;
use super::tests::{camera, doc_shaped, doc_with, window};
use ph2d_anim::Interp;
use ph2d_timeline::{MotionPath, PathAnchor};

/// **As CINCO portas concordam sobre a aba** — a de pintar e as quatro de agarrar.
///
/// A metade de PRESENÇA existe para a de AUSÊNCIA não ser vácua: sem ela, um `active_path`
/// que devolvesse `None` sempre (fixture quebrada, seleção errada, câmera fora) passaria
/// verde afirmando exatamente nada. Cada porta é exercida no MESMO documento, com a MESMA
/// câmera e nas MESMAS coordenadas nas duas metades; a única coisa que muda é `keys_tab`.
///
/// ⚠️ O oráculo de agarrar é lido do DESENHO (`anchor_screen`/`tangent_screen` dão os px
/// que o `marks` pinta) — apertar num ponto chutado devolveria `None` nas duas metades e
/// metade do gate ficaria verde sobre nada. É a mesma disciplina do irmão
/// `a_clip_that_does_not_animate_the_path_shows_no_handles`.
#[test]
fn the_trajectory_is_offered_only_on_the_keys_tab() {
    let (doc, e) = doc_shaped();
    let (cam, win) = (camera(), window());

    // --- ABA KEYS: a trajetória inteira é oferecida. ---
    assert!(
        !marks(true, &doc, Some(e), &cam, win).is_empty(),
        "a fixture não contém o fenômeno: nada é desenhado nem na aba Keys"
    );
    let anchors = super::anchor_screen(true, &doc, Some(e), &cam, win);
    let tangents = super::tangent_screen(true, &doc, Some(e), &cam, win);
    assert!(!anchors.is_empty(), "sem âncora a enumerar na aba Keys");
    assert!(!tangents.is_empty(), "sem alça a enumerar na aba Keys");

    // O ponto que de fato agarra a âncora, e o que de fato acerta a curva — os dois lidos
    // do desenho, nunca chutados.
    let a = anchors[0].2;
    let (hx, hy) = (a.x as f32, a.y as f32);
    assert!(
        super::motion_path_hit(true, &doc, Some(e), &cam, win, hx, hy).is_some(),
        "a âncora desenhada não é agarrável nem na aba Keys"
    );
    let on_curve = (0..1000)
        .map(|k| {
            (
                100.0 + f32::from(u16::try_from(k % 100).unwrap_or(0)) * 8.0,
                100.0 + f32::from(u16::try_from(k / 100).unwrap_or(0)) * 80.0,
            )
        })
        .find(|&(x, y)| {
            super::motion_path_curve_hit(true, &doc, Some(e), &cam, win, x, y).is_some()
        })
        .expect("algum ponto de tela cai sobre a curva na aba Keys");

    // --- FORA DELA (Arrange, um container, ou o painel fechado): nada. ---
    assert!(
        marks(false, &doc, Some(e), &cam, win).is_empty(),
        "a trajetória foi DESENHADA fora da aba Keys"
    );
    assert!(
        super::anchor_screen(false, &doc, Some(e), &cam, win).is_empty()
            && super::tangent_screen(false, &doc, Some(e), &cam, win).is_empty(),
        "há âncora/alça a enumerar fora da aba Keys — o desenho some e o alvo fica"
    );
    assert!(
        super::motion_path_hit(false, &doc, Some(e), &cam, win, hx, hy).is_none(),
        "o ponto que agarra a âncora na aba Keys ainda agarra fora dela"
    );
    assert!(
        super::motion_path_curve_hit(false, &doc, Some(e), &cam, win, on_curve.0, on_curve.1)
            .is_none(),
        "o duplo-clique ainda insere âncora na curva fora da aba Keys"
    );
}

/// **Num clip que NÃO anima a trajetória, não há âncora para pintar nem para agarrar**
/// (report do Enio, 2026-07-30: *"o Path criado em um Clip contamina e aparece alças em
/// outro Clip criado depois"*).
///
/// Naquele desenho o caminho morava no BINDING (do DOCUMENTO) e as keys no CLIP: o `marks`
/// já perguntava pela track do clip ATIVO e devolvia zero marcas, mas o `anchor_screen` e o
/// `tangent_screen` liam `b.path` direto, então um clip criado depois herdava as âncoras e
/// as alças do outro — **nada desenhado e tudo agarrável** (`marks=0 ancoras=2 alcas=2
/// agarravel=SIM`). O clique numa alça invisível arrastava a trajetória do OUTRO clip, e o
/// duplo-clique inseria âncora numa curva que ninguém via.
///
/// Hoje a trajetória é do CLIP (`NamedClip::paths`), então o clip B deste gate não tem
/// nenhuma — e é isso que o gate afirma continuar valendo: as quatro perguntas passam pela
/// mesma porta e a ausência é estrutural.
///
/// ⚠️ O oráculo pergunta as TRÊS coisas (pintar · agarrar · a curva), porque as três liam o
/// caminho por portas diferentes e uma só ficaria verde sobre as outras duas.
///
/// **Mutação que deve sangrar:** o `active_path` trocar o `clip_path` (leitura CRUA) pelo
/// `path_for` (que tem recuo para o avaliador) — a alça fantasma volta na hora.
#[test]
fn a_clip_that_does_not_animate_the_path_shows_no_handles() {
    let (mut doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (1.0, 20.0, Interp::Linear)]);
    // ⚠️ Âncoras COM alça: o `doc_with` usa `PathAnchor::corner`, cujas alças têm
    // comprimento zero e o `tangent_screen` PULA (`TANGENT_MIN_PX`) — com elas a metade
    // das alças deste gate seria verde sobre um conjunto vazio, que é a fixture não
    // conter o fenômeno. A asserção de controle logo abaixo é o que prende isso.
    {
        let t = doc.bindings()[0].target;
        doc.set_clip_path(
            doc.active_index(),
            t,
            MotionPath::new(vec![
                PathAnchor {
                    anchor: [0.0, 0.0],
                    out_handle: [5.0, 3.0],
                    ..PathAnchor::corner([0.0, 0.0])
                },
                PathAnchor {
                    anchor: [20.0, 0.0],
                    in_handle: [-5.0, 3.0],
                    ..PathAnchor::corner([20.0, 0.0])
                },
            ]),
        );
    }
    let (cam, win) = (camera(), window());

    // O clip que AUTOROU a trajetória: tudo presente. CONTROLE POSITIVO — sem isto, um
    // conjunto vazio nos dois clips passaria por "não vaza".
    let a_marks = marks(true, &doc, Some(e), &cam, win).len();
    let a_anchors = super::anchor_screen(true, &doc, Some(e), &cam, win);
    let a_tangents = super::tangent_screen(true, &doc, Some(e), &cam, win);
    assert!(
        a_marks > 0 && a_anchors.len() == 2 && !a_tangents.is_empty(),
        "o clip que anima a trajetória a desenha: marks={a_marks} âncoras={} alças={}",
        a_anchors.len(),
        a_tangents.len()
    );
    let on_anchor = a_anchors[0].2;
    let (hx, hy) = (on_anchor.x as f32, on_anchor.y as f32);
    assert!(
        super::motion_path_hit(true, &doc, Some(e), &cam, win, hx, hy).is_some(),
        "e a âncora dele é agarrável"
    );
    // ⚠️ O ponto SOBRE A CURVA é pedido ao próprio produto (uma varredura em px de tela até
    // ele dizer "aqui"), não chutado: um ponto fora do raio de pega devolveria `None` nos
    // DOIS clips e a metade do duplo-clique ficaria verde sobre nada.
    let on_curve = (0..1000)
        .map(|k| {
            (
                100.0 + f32::from(u16::try_from(k % 100).unwrap_or(0)) * 8.0,
                100.0 + f32::from(u16::try_from(k / 100).unwrap_or(0)) * 80.0,
            )
        })
        .find(|&(x, y)| {
            super::motion_path_curve_hit(true, &doc, Some(e), &cam, win, x, y).is_some()
        })
        .expect("algum ponto de tela cai sobre a curva no clip que a anima");

    // Um clip criado DEPOIS: o mesmo objeto, o mesmo binding — e trajetória NENHUMA.
    let b = doc.add_clip("B".into());
    doc.set_active(b);

    assert!(
        marks(true, &doc, Some(e), &cam, win).is_empty(),
        "nada é DESENHADO num clip que não anima a trajetória"
    );
    assert!(
        super::anchor_screen(true, &doc, Some(e), &cam, win).is_empty()
            && super::tangent_screen(true, &doc, Some(e), &cam, win).is_empty(),
        "e não há âncora nem alça a enumerar"
    );
    assert!(
        super::motion_path_hit(true, &doc, Some(e), &cam, win, hx, hy).is_none(),
        "nem a agarrar: o ponto que pegava a âncora no clip A não pega nada aqui"
    );
    assert!(
        super::motion_path_curve_hit(true, &doc, Some(e), &cam, win, on_curve.0, on_curve.1)
            .is_none(),
        "nem a curva: o duplo-clique no ponto que ACERTAVA a curva no clip A não insere \
         âncora numa trajetória que este clip não anima"
    );
}
