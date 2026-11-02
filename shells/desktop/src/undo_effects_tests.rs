//! ⭐ **Os LIVE PATH EFFECTS no undo** — irmão por assunto do [`super`] (a captura do mundo e da
//! geometria), como o `undo_tags_tests`: pôr, tirar e ajustar um efeito, cada um o seu passo. O teto
//! de LOC do shell exige o corte — ⛔ *split, nunca a marca de isenção*.

use super::*;

/// REGRESSÃO (Enio reportou 3×: *"undo/redo ainda não implementado para efeitos"*). O undo é
/// dirigido por diff de `ProjectState`, e o `vec` (a `VecScene`, que carrega a pilha de Live
/// Path Effects em `VecPath::effects`) está no `PartialEq`. A prova de que "undo funciona para
/// efeitos" tem TRÊS partes, e este gate as afirma para CADA tipo de efeito:
///
/// 1. **pôr um efeito muda o estado capturado** — a pilha vai de 0 para 1 entrada, mesmo que o
///    efeito nasça NEUTRO ⇒ o `post_frame_undo` registaria um passo (o diff não é vazio);
/// 2. **restaurar o pré-estado tira o efeito** — o Ctrl+Z reinstala a `VecScene` inteira;
/// 3. **capturar depois do restore é ponto FIXO** (nos dois sentidos: sem e com efeito) — senão
///    um passo espúrio nasceria a seguir e o 1º Ctrl+Z gastar-se-ia a desfazê-lo (a classe do
///    z-order), que é exatamente como *"o undo não funciona"* se sente do lado de fora.
///
/// A pilha inteira (input→bus→mutação→diff→restore) já estava correta; o que faltava era um gate
/// que o prova sem uma janela. Se algum efeito falhar aqui, o bug é ESTE, headless e reproduzível.
#[test]
fn putting_or_removing_any_effect_round_trips_through_undo() {
    let reg = registry();
    for kind in 0..ph2d_vec_scene::effect::PathEffect::KINDS.len() {
        let (mut sim, vec) = scene();
        let id = vec.paths()[0].id;
        let baseline = capture(&mut sim, &vec, &reg);

        // (1) PÔR um efeito muda o estado — a `VecScene` deixa de ser igual ao pré-estado.
        let mut with_fx = vec.clone();
        crate::fx_bridge::add(&mut with_fx, id, kind);
        assert_eq!(
            with_fx.paths()[0].effects.len(),
            1,
            "kind {kind}: add não pôs efeito nenhum"
        );
        let cap_fx = capture(&mut sim, &with_fx, &reg);
        assert_ne!(
            baseline, cap_fx,
            "kind {kind}: pôr um efeito não registaria passo de undo (diff vazio)"
        );

        // (2) restaurar o PRÉ-estado (o Ctrl+Z) tira o efeito.
        let (rvec, _, _, _) = baseline.restore(&mut sim, &reg);
        assert!(
            rvec.paths()[0].effects.is_empty(),
            "kind {kind}: o restore do pré-estado não removeu o efeito"
        );
        // (3a) e é ponto fixo — capturar de novo dá o pré-estado ao bit (sem passo espúrio).
        let again = capture(&mut sim, &rvec, &reg);
        assert_eq!(
            baseline, again,
            "kind {kind}: capture pos-restore != pre-estado (passo espurio -> Ctrl+Z 'nao faz nada')"
        );

        // (3b) o sentido do REDO: o estado COM efeito também sobrevive a capture→restore→capture.
        let (rvec_fx, _, _, _) = cap_fx.restore(&mut sim, &reg);
        assert_eq!(
            rvec_fx.paths()[0].effects.len(),
            1,
            "kind {kind}: o restore do estado-com-efeito perdeu o efeito"
        );
        let again_fx = capture(&mut sim, &rvec_fx, &reg);
        assert_eq!(
            cap_fx, again_fx,
            "kind {kind}: capture pós-restore(com efeito) != estado-com-efeito"
        );
    }
}

/// AJUSTAR um parâmetro de um efeito é o SEU próprio passo de undo — mover um slider muda o
/// estado outra vez, e o restore devolve o valor anterior. (O `set_param` passa pela mesma
/// `VecScene` e o mesmo `PartialEq`, então cobre também Repeater/Bloat/Trim por construção; o
/// Zig Zag é o representante concreto.)
#[test]
fn editing_an_effect_param_is_its_own_undo_step() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let id = vec.paths()[0].id;

    // Adiciona um Zig Zag (kind 1) e captura o estado NEUTRO com o efeito.
    let mut with_fx = vec.clone();
    crate::fx_bridge::add(&mut with_fx, id, 1);
    let cap_neutral = capture(&mut sim, &with_fx, &reg);

    // Acha o 1º parâmetro NÃO-caixinha cujo máximo difere do valor neutro, e leva-o ao máximo.
    let params = with_fx.paths()[0].effects[0].effect.params();
    let pi = params
        .iter()
        .enumerate()
        .find(|(i, d)| {
            !d.toggle && (d.max - with_fx.paths()[0].effects[0].effect.get(*i)).abs() > 1e-9
        })
        .map(|(i, _)| i)
        .expect("o Zig Zag tem um parâmetro contínuo não-neutro no máximo");

    let mut edited = with_fx.clone();
    crate::fx_bridge::set_param(&mut edited, id, 0, pi, 1.0); // track 1.0 = o máximo da faixa
    assert_ne!(
        edited.paths()[0].effects[0].effect.get(pi),
        with_fx.paths()[0].effects[0].effect.get(pi),
        "set_param não mudou o valor"
    );
    let cap_edited = capture(&mut sim, &edited, &reg);
    assert_ne!(
        cap_neutral, cap_edited,
        "ajustar o parâmetro não registaria passo de undo"
    );

    // O Ctrl+Z (restaurar o estado neutro) devolve o parâmetro ao valor anterior.
    let (rvec, _, _, _) = cap_neutral.restore(&mut sim, &reg);
    let restored = rvec.paths()[0]
        .effects
        .first()
        .and_then(|e| e.effect.as_zigzag().map(|_| e.effect.get(pi)));
    assert_eq!(
        restored,
        Some(with_fx.paths()[0].effects[0].effect.get(pi)),
        "o restore não devolveu o parâmetro ao valor anterior"
    );
}
