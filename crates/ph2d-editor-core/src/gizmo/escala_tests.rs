//! ⭐ **A escala do canvas tem a lei do par travado** (`crate::par_travado`): o sinal é do objecto e um
//! zero recupera-se. Por gesto: a alça acha-se por clique na caixa pintada, e o pen-down, o movimento
//! e a conta são os do arrasto real. A caixa pintada segue `vec_gizmo_view::gizmo_view_from`.

use super::*;
use crate::zones::Rect;

const PX_POR_MUNDO: f32 = 60.0; // janela de 600 px e 10 de mundo

fn camera() -> GizmoCamera {
    GizmoCamera {
        center: [0.0, 0.0],
        height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
    }
}

fn pose(scale: [f32; 2]) -> TransformSnapshot {
    TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale,
    }
}

fn caixa(p: TransformSnapshot, pai: TransformSnapshot, meio: [f32; 2], rotacao: f32) -> GizmoView {
    let m = compose_snapshot(pai, p);
    let h = [(meio[0] * m.scale[0]).abs(), (meio[1] * m.scale[1]).abs()];
    let c = m.translation;
    GizmoView {
        bbox_min_world: [c[0] - h[0], c[1] - h[1]],
        bbox_max_world: [c[0] + h[0], c[1] + h[1]],
        rotation: rotacao,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
        cursor_screen: None,
        pivot_world: c,
        pivot_tool_active: false,
        model: MODEL_IDENTITY,
    }
}

fn ecra(w: [f32; 2]) -> (f32, f32) {
    (400.0 + w[0] * PX_POR_MUNDO, 300.0 - w[1] * PX_POR_MUNDO)
}

/// O clique no ponto de mundo `onde`, na caixa pintada: que alça é, e o arrasto que ela abre.
fn agarra(
    p: TransformSnapshot,
    pai: TransformSnapshot,
    meio: [f32; 2],
    rotacao_pintada: f32,
    onde: [f32; 2],
) -> GizmoDragState {
    let (x, y) = ecra(onde);
    let id = sprite_gizmo_hit_at(&caixa(p, pai, meio, rotacao_pintada), false, false, x, y)
        .expect("a alça não respondeu ao clique");
    let kind = gizmo_kind_for_id(id).expect("não é uma alça");
    let inicio = camera().screen_to_world((x, y));
    GizmoDragState {
        kind,
        entity_bits: 1,
        start_screen: (x, y),
        cursor_screen: (x, y),
        start_transform: p,
        pivot_world: grip_pivot_world(kind, [0.0, 0.0], meio, compose_snapshot(pai, p), inicio),
        start_cursor_world: inicio,
        sprite_half_intrinsic: meio,
        anchor_is_center: false,
        target: GizmoTarget::PrimaryIndividual,
        parent_world: pai,
        turns: 0,
    }
}

fn move_para(drag: &GizmoDragState, onde: [f32; 2], shift: bool) -> TransformSnapshot {
    let mut d = *drag;
    d.advance_cursor(ecra(onde), &camera());
    let modificadores = GizmoModifiers {
        shift,
        ..GizmoModifiers::default()
    };
    compute_gizmo_transform(&d, &camera(), modificadores, GizmoSnap::default(), None)
}

fn perto(a: f32, b: f32, o_que: &str) {
    assert!((a - b).abs() < 1e-2, "{o_que}: {a} × {b}");
}

/// As bordas de mundo [esquerda, direita] do eixo X da forma.
fn bordas_x(t: TransformSnapshot, meio: f32) -> [f32; 2] {
    let h = (t.scale[0] * meio).abs();
    [t.translation[0] - h, t.translation[0] + h]
}

const MEIO: [f32; 2] = [2.0, 1.0];
const RAIZ: TransformSnapshot = TransformSnapshot::IDENTITY;

/// Espelhado em X: agarrar a borda da direita não colapsa o objecto, mantém o sinal, a borda segue a
/// mão e a esquerda fica. *Mutação: `grip_pivot_world` sem o reflexo, ou o `.max(0.001)` antigo.*
#[test]
fn um_objecto_espelhado_escala_com_o_sinal_e_a_borda_oposta_fica() {
    let p = pose([-1.0, 1.0]);
    let d = agarra(p, RAIZ, MEIO, 0.0, [2.0, 0.0]);
    let t = move_para(&d, [3.0, 0.0], false);
    perto(t.scale[0], -1.25, "X");
    perto(t.scale[1], 1.0, "Y");
    let [e, r] = bordas_x(t, MEIO[0]);
    perto(e, -2.0, "a borda oposta");
    perto(r, 3.0, "a borda na mão");
}

/// O mesmo em Y, e o espelho do sprite em X (pintado com meia volta): o lado é o da mão.
#[test]
fn o_lado_da_alca_e_o_da_mao_em_qualquer_convencao() {
    let d = agarra(pose([1.0, -1.0]), RAIZ, MEIO, 0.0, [0.0, 1.0]);
    let t = move_para(&d, [0.0, 2.0], false);
    perto(t.scale[1], -1.5, "Y");
    perto(t.translation[1], 0.5, "a borda de baixo fica em -1");

    let d = agarra(
        pose([-1.0, 1.0]),
        RAIZ,
        MEIO,
        core::f32::consts::PI,
        [2.0, 0.0],
    );
    let t = move_para(&d, [3.0, 0.0], false);
    perto(t.scale[0], -1.25, "sprite, X");
    perto(bordas_x(t, MEIO[0])[0], -2.0, "sprite, a borda oposta");
}

/// Atravessar o pivô não vira o objecto: pára no piso, com o sinal de partida, nos dois sentidos.
/// Decisão registada: espelhar é verbo do cartão. *Mutação: tirar o `signum` ⇒ vermelho.*
#[test]
fn atravessar_o_pivo_para_no_piso_com_o_sinal() {
    for (x, esperado) in [(1.0_f32, 0.001_f32), (-1.0, -0.001)] {
        let d = agarra(pose([x, 1.0]), RAIZ, MEIO, 0.0, [2.0, 0.0]);
        let t = move_para(&d, [-9.0, 0.0], false);
        assert_eq!(t.scale[0], esperado, "partida {x}");
    }
}

/// Um eixo a zero recupera-se pela alça, as duas bordas abrem com a da mão sob o cursor, e o gesto é
/// reversível. *Mutação: voltar a `partida * razão` ⇒ fica a 0 (ou ao piso), vermelho.*
#[test]
fn um_eixo_a_zero_abre_pela_alca() {
    let p = pose([0.0, 3.0]);
    let d = agarra(p, RAIZ, MEIO, 0.0, [0.0, 0.0]);
    assert!(
        matches!(d.kind, GizmoDragKind::ScaleEdge { axis: 0, .. }),
        "a caixa a zero em X abre pela alça de X: {:?}",
        d.kind
    );
    let parado = move_para(&d, [0.0, 0.0], false);
    assert_eq!(parado.scale, [0.0, 3.0], "só clicar não muda nada");
    assert_eq!(parado.translation, [0.0, 0.0], "só clicar não anda");

    let t = move_para(&d, [1.0, 0.0], false);
    perto(t.scale[0], 0.5, "X abre");
    assert_eq!(t.scale[1], 3.0, "Y não tem nada com isto");
    let [e, r] = bordas_x(t, MEIO[0]);
    perto(e, -1.0, "as duas bordas abrem");
    perto(r, 1.0, "a da mão sob o cursor");

    let t = move_para(&d, [-1.0, 0.0], false);
    perto(
        t.scale[0],
        0.5,
        "para o outro lado também abre: um zero não tem sinal",
    );

    assert_eq!(
        move_para(&d, [0.0, 0.0], false).scale,
        [0.0, 3.0],
        "e volta"
    );
}

/// Sob um pai escalado, a borda continua sob a mão. *Mutação: tirar o `pai` de `abre_do_zero`.*
#[test]
fn um_eixo_a_zero_sob_pai_escalado_segue_a_mao() {
    let pai = TransformSnapshot {
        scale: [2.0, 2.0],
        ..RAIZ
    };
    let d = agarra(pose([0.0, 1.0]), pai, MEIO, 0.0, [0.0, 0.0]);
    let t = move_para(&d, [2.0, 0.0], false);
    perto(t.scale[0], 0.5, "local");
    perto(
        compose_snapshot(pai, t).scale[0] * MEIO[0],
        2.0,
        "a borda de mundo",
    );
}

/// Os dois a zero, canto com Shift: sem proporção para ler é 1:1. Sem Shift, cada eixo é o seu.
/// *Mutação: tirar o ramo `shift && zero_x && zero_y`.*
#[test]
fn os_dois_a_zero_com_shift_sao_um_para_um() {
    let d = agarra(pose([0.0, 0.0]), RAIZ, MEIO, 0.0, [0.0, 0.0]);
    let canto = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        ..d
    };
    let t = move_para(&canto, [2.0, 0.5], true);
    assert_eq!(t.scale[0], t.scale[1], "1:1");
    perto(t.scale[0], 1.0, "a maior das duas");
    let t = move_para(&canto, [2.0, 0.5], false);
    perto(t.scale[0], 1.0, "X");
    perto(t.scale[1], 0.5, "Y");
    // A borda: uma alça de lado com Shift leva o outro eixo a zero com ela.
    let lado = GizmoDragState {
        kind: GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 },
        ..d
    };
    let t = move_para(&lado, [1.0, 0.0], true);
    assert_eq!(t.scale[0], t.scale[1], "1:1 na alça de lado");
}

/// Shift sem proporção para ler (X 0 × Y 9): o eixo a zero não é multiplicado nem levado ao piso, e o
/// outro não herda uma razão inventada. *Mutação: o `.max(0.001)` antigo no outro eixo ⇒ 0,001.*
#[test]
fn shift_com_um_eixo_a_zero_nao_inventa_proporcao() {
    let p = pose([0.0, 1.5]);
    // O canto de cima: X a zero, Y lê a sua razão.
    let d = agarra(p, RAIZ, MEIO, 0.0, [0.0, 1.5]);
    let canto = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        ..d
    };
    let t = move_para(&canto, [0.0, 3.0], true);
    perto(
        t.scale[1],
        2.25,
        "Y: a borda de baixo fica, a de cima na mão",
    );
    assert_eq!(t.scale[0], 0.0, "X fica: a mão não andou em X");

    // A mão pega ao lado do pivô em X: essa razão é ruído e não pode mandar em Y.
    // *Mutação: Shift sem a guarda dos zeros ⇒ Y sai 15.*
    let d = agarra(p, RAIZ, MEIO, 0.0, [0.05, 1.5]);
    let canto = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        ..d
    };
    let t = move_para(&canto, [0.5, 3.0], true);
    perto(t.scale[1], 2.25, "Y não herda a razão de X");

    // A ponta de cima com Shift, com X a zero (o canto ganhou a alça do meio, ver `alca_de_lado_cede`):
    // a mão não andou em X, logo Y anda, X fica a zero, sem piso.
    let d = agarra(p, RAIZ, MEIO, 0.0, [0.0, 1.5]);
    assert!(matches!(d.kind, GizmoDragKind::ScaleCorner { .. }));
    let t = move_para(&d, [0.0, 3.0], true);
    assert_eq!(t.scale[0], 0.0, "o outro eixo a zero fica a zero");
    perto(
        t.scale[1],
        2.25,
        "Y: a borda de baixo fica, a de cima na mão",
    );
}

/// Agarrar um objecto mais pequeno do que o piso não o faz saltar para cima.
/// *Mutação: `piso` sem o `min` ⇒ vermelho.*
#[test]
fn agarrar_abaixo_do_piso_nao_salta() {
    let s = 0.000_5;
    let mut d = agarra(pose([1.0, 1.0]), RAIZ, MEIO, 0.0, [2.0, 0.0]);
    d.start_transform.scale = [s, 1.0];
    assert_eq!(
        compute_gizmo_transform(
            &d,
            &camera(),
            GizmoModifiers::default(),
            GizmoSnap::default(),
            None
        )
        .scale[0],
        s
    );
}

/// Sem espelho, o pivô é o de `anchor_pivot_world`, ao bit, e quem não escala nem pergunta.
#[test]
fn sem_espelho_o_pivo_e_o_de_antes_ao_bit() {
    let snap = TransformSnapshot {
        translation: [0.3, -0.7],
        rotation: 0.37,
        scale: [1.3, 0.9],
    };
    let kind = GizmoDragKind::ScaleCorner {
        dx_sign: 1.0,
        dy_sign: 1.0,
    };
    let a = anchor_pivot_world(kind, [0.1, 0.2], MEIO, snap, false);
    let mao = anchor_pivot_world(
        GizmoDragKind::ScaleCorner {
            dx_sign: -1.0,
            dy_sign: -1.0,
        },
        [0.1, 0.2],
        MEIO,
        snap,
        false,
    );
    assert_eq!(grip_pivot_world(kind, [0.1, 0.2], MEIO, snap, mao), a);
    assert_eq!(
        grip_pivot_world(GizmoDragKind::Rotate, [0.1, 0.2], MEIO, snap, mao),
        anchor_pivot_world(GizmoDragKind::Rotate, [0.1, 0.2], MEIO, snap, false)
    );
}

/// Gizmo GLOBAL com o primário a zero em X: o fator é o do grupo, e o primário fica a zero.
/// *Mutação: `partida_do_grupo` sem efeito ⇒ o fator cai a 1 e os outros não mudam.*
#[test]
fn o_grupo_escala_os_outros_com_o_primario_a_zero() {
    let mut d = agarra(pose([1.0, 1.0]), RAIZ, MEIO, 0.0, [2.0, 0.0]);
    d.start_transform = pose([0.0, 1.0]);
    d.target = GizmoTarget::Global;
    d.pivot_world = [-2.0, 0.0];
    let conta = partida_do_grupo(d);
    let nova = move_para(&conta, [3.0, 0.0], false).scale;
    let fator = fator_do_grupo(conta.start_transform.scale, nova);
    perto(fator[0], 1.25, "o fator do grupo");
    perto(fator[1], 1.0, "Y");
    assert_eq!(
        escala_do_primario(&d, nova)[0],
        0.0,
        "o primário fica a zero"
    );
}

/// Fora do global nada muda: o eixo a zero abre, e os outros não mudam nele.
#[test]
fn fora_do_global_o_zero_abre_e_o_fator_e_um() {
    let d = agarra(pose([0.0, 1.0]), RAIZ, MEIO, 0.0, [0.0, 0.0]);
    assert_eq!(partida_do_grupo(d).start_transform.scale, [0.0, 1.0]);
    assert_eq!(escala_do_primario(&d, [0.5, 1.0]), [0.5, 1.0]);
    assert_eq!(fator_do_grupo([0.0, 1.0], [0.5, 1.0]), [1.0, 1.0]);
}

/// Seleção rodada: proporcional pelo eixo que a mão move; o eixo a zero abre, senão a alça fica morta.
/// *Mutação: o eixo a zero a `partida × u` ⇒ não abre.*
#[test]
fn a_selecao_rodada_e_proporcional_e_o_zero_abre() {
    assert_eq!(uniforme_do_grupo([2.0, 1.0], [3.0, 1.0]), [3.0, 1.5]);
    assert_eq!(uniforme_do_grupo([0.0, 1.0], [0.5, 1.0]), [0.5, 1.0]);
    assert_eq!(uniforme_do_grupo([0.0, 1.0], [0.0, 2.0]), [0.0, 2.0]);
}
