//! **O LAÇO rótulo↔rota, nos gates** — módulo filho do [`super`] (teto de 600 LOC por arquivo da
//! shell). Herda `Doc`, `near` e `text_params` pelo `use super::*`.
//!
//! Os dois gates aqui são os que o bug do Enio derrubou, e os dois só mordem porque o [`super::Doc`]
//! roda a **sequência EXATA do `render_loop`** — inclusive o `upkeep_pending`, que pendura o
//! `VecLabel` DEPOIS de o `recook` já ter montado as paredes. A versão antiga do harness pendurava
//! o vínculo à mão antes do 1º frame e media a partir do frame 1: verde, com o bug na tela.

use super::*;

/// **O LAÇO** — "linha e texto pulando sem parar" (Enio, rodando o app).
///
/// O rótulo de um conector mora EM CIMA da rota dele: é onde ele nasce, no meio por comprimento de
/// arco. Se a rota enxergar o rótulo — como parede ou como alvo de ponta —, o sistema realimenta:
/// a linha desvia da própria legenda, o rótulo se re-centra na rota desviada, a linha desvia de
/// novo. Os dois pulam, para sempre.
///
/// # Por que o gate ANTERIOR ficava VERDE com o bug na tela
///
/// Duas razões, e as duas são a mesma lição ([[feedback_harness_reproduces_mechanism_not_context]]):
///
/// 1. **Ele pendurava o `VecLabel` à mão, antes do 1º frame.** No app o vínculo é pendurado pelo
///    `upkeep_pending`, que roda **DEPOIS** do `connector_live::recook` — quem monta as paredes.
///    Existe, portanto, uma janela real (o frame em que a 1ª letra materializa o texto) em que o
///    objeto está na cena e o vínculo não. O harness antigo nunca simulava essa janela: ele media
///    o único mundo em que a isenção já valia desde sempre.
/// 2. **Ele media a partir do frame 1.** O frame 0 é justamente o do desvio.
///
/// Agora o rótulo nasce como no app ([`Doc::label_born_the_real_way`]), o frame é a sequência
/// EXATA do `render_loop` ([`Doc::frame`]) — e a asserção é em **TODO** frame, do ZERO ao 31.
///
/// # E o gate mede o PERÍODO
///
/// Uma oscilação de período 2 é invisível para quem olha um frame sim, outro não. Então, além do
/// valor certo, o gate exige **período zero**: a rota e a pose de um frame são BYTE-idênticas às do
/// anterior, do 1º frame em diante. Nada se move sozinho.
#[test]
fn a_label_sitting_on_the_route_never_pushes_the_line_aside() {
    let mut d = Doc::new();
    // Duas caixas lado a lado: a rota é a reta y = 0.5, de (2, 0.5) a (8, 0.5).
    let a = d.shape([0.0, 0.0], [2.0, 1.0]);
    let b = d.shape([8.0, 0.0], [10.0, 1.0]);
    let conn = d.connector(a, b);
    d.frame();
    let seg = |p: [f64; 2], q: [f64; 2]| (q[0] - p[0]).hypot(q[1] - p[1]);
    let length = |pts: &[[f64; 2]]| -> f64 { pts.windows(2).map(|w| seg(w[0], w[1])).sum() };
    let straight = length(&d.route(conn));
    assert!(
        (straight - 6.0).abs() < 1e-6,
        "a rota limpa e reta: {straight}"
    );

    // O rótulo do conector, uma caixa BEM em cima da rota — e nascendo como no app.
    let label = d.label_born_the_real_way([4.0, 0.2], [6.0, 0.8], conn);

    let mut history: Vec<(Vec<[f64; 2]>, Transform)> = Vec::new();
    for f in 0..32 {
        d.frame();
        let route = d.route(conn);
        let len = length(&route);
        assert!(
            (len - straight).abs() < 1e-6,
            "frame {f}: a rota desviou do proprio ROTULO ({len:.3} contra {straight:.3} da reta) \
             — texto nao e parede, e um conector nao pode fugir da propria legenda"
        );
        assert!(
            near(d.centre(label), [5.0, 0.5]),
            "frame {f}: o rotulo tem de seguir parado no meio da linha: {:?}",
            d.centre(label)
        );
        let pose = d
            .sim
            .world()
            .get::<Transform>(d.entity(label))
            .copied()
            .expect("a pose do rotulo");
        history.push((route, pose));
    }

    // **PERÍODO ZERO.** Do 1º frame em diante nada se mexe — nem a rota, nem a pose. Um ciclo de
    // período 2 (o que o bug produz) morre exatamente aqui.
    for f in 1..history.len() {
        assert_eq!(
            history[f],
            history[f - 1],
            "frame {f}: a rota e/ou o rotulo se moveram sozinhos — isto e a oscilacao \
             (periodo != 0), e nenhum frame par a esconde"
        );
    }
}

/// **O texto que o usuário largou sobre a linha com a ferramenta T também não a empurra.**
///
/// O par do gate acima, e o que prova que a isenção não depende do VÍNCULO. Um texto solto nunca
/// tem `VecLabel` — nenhum passe vai pendurá-lo, em nenhum frame. Se a parede fosse decidida pelo
/// vínculo, este texto empurraria a linha **para sempre** (não por um frame), e o gate de cima
/// jamais o pegaria.
///
/// A regra certa é a que o `connector_walls` implementa: *texto é anotação, não estrutura*.
#[test]
fn a_loose_text_object_dropped_on_the_route_is_not_a_wall_either() {
    let mut d = Doc::new();
    let a = d.shape([0.0, 0.0], [2.0, 1.0]);
    let b = d.shape([8.0, 0.0], [10.0, 1.0]);
    let conn = d.connector(a, b);
    d.frame();
    let seg = |p: [f64; 2], q: [f64; 2]| (q[0] - p[0]).hypot(q[1] - p[1]);
    let length = |pts: &[[f64; 2]]| -> f64 { pts.windows(2).map(|w| seg(w[0], w[1])).sum() };
    let straight = length(&d.route(conn));

    // Um TEXTO comum (sem vínculo nenhum) bem em cima da rota.
    let text = d.scene.push_path(rectangle([4.0, 0.2], [6.0, 0.8]));
    crate::vec_entities::sync(&mut d.sim, &mut d.scene, &mut d.map);
    let e = d.entity(text);
    if let Ok(mut ent) = d.sim.world_mut().get_entity_mut(e) {
        ent.insert(VecShape::Text(text_params()));
    }
    for f in 0..8 {
        d.frame();
        assert!(
            d.sim.world().get::<VecLabel>(d.entity(text)).is_none(),
            "frame {f}: este texto e SOLTO — ninguem vai vincula-lo"
        );
        let len = length(&d.route(conn));
        assert!(
            (len - straight).abs() < 1e-6,
            "frame {f}: a linha desviou de um TEXTO ({len:.3} contra {straight:.3}) — anotacao \
             nao e estrutura, tenha ela dono ou nao"
        );
    }
}
