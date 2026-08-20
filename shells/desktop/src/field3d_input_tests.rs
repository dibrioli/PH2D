//! Os gates da navegação da janela 3D de modelagem.
//!
//! ⚠️ **Nenhum destes afirma um sinal.** A pergunta que o artista faz é *"o modelo segue a minha
//! mão?"*, e é essa que se mede — traçando a peça e olhando **onde ela ficou na tela**. Argumentar
//! sobre `yaw += dx` é literalmente como o erro entrou na linha irmã, nos dois eixos de uma vez.

use super::law;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_render::{Orbit, trace_with};

const W: u32 = 120;
const H: u32 = 120;

/// Uma esfera pequena **do lado de cá** da origem.
///
/// ⚠️ A posição não é decorativa: com a câmera de frente (`yaw = 0`), um ponto em `+Z` projeta-se no
/// **centro** do quadro, e é justamente por isso que ele serve — qualquer giro tira-o do centro, e o
/// lado para onde ele sai responde à pergunta sem ambiguidade. Uma esfera em `+X` já começaria
/// deslocada e giraria *para dentro*, que é a armadilha desta medição.
fn nose() -> FieldDoc {
    FieldDoc::new(
        vec![Node {
            xform: Xform::at(0.0, 0.0, 0.45),
            kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.12 }),
        }],
        NodeId(0),
    )
    .expect("esfera")
}

fn front() -> Orbit {
    Orbit::from_yaw_pitch(0.0, 0.0)
}

/// O centro de massa da peça na tela, em pixels — `None` se ela não aparece.
///
/// ⚠️ **Sem anti-serrilhado de propósito**: o que se mede aqui é para onde a forma foi, e a máscara
/// crua responde a isso sem uma segunda variável no meio.
fn centroid(doc: &FieldDoc, cam: &Orbit) -> Option<(f32, f32)> {
    let g = trace_with(doc, cam, W, H, true, false);
    let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0usize);
    for i in 0..(W as usize) * (H as usize) {
        if g.hit[i] {
            sx += (i % W as usize) as f64;
            sy += (i / W as usize) as f64;
            n += 1;
        }
    }
    (n > 0).then(|| ((sx / n as f64) as f32, (sy / n as f64) as f32))
}

/// ⭐ **O modelo segue a mão** — nos dois eixos, medido na tela.
#[test]
fn dragging_right_turns_the_model_right_and_dragging_down_shows_its_top() {
    let doc = nose();
    let base = centroid(&doc, &front()).expect("a peça aparece de frente");
    // De frente, a esfera em +Z cai no centro do quadro.
    assert!(
        (base.0 - W as f32 * 0.5).abs() < 2.0,
        "de frente a peça devia estar centrada em x, e está em {}",
        base.0
    );

    let mut right = front();
    law::orbit(&mut right, 40.0, 0.0);
    let moved = centroid(&doc, &right).expect("a peça continua no quadro");
    assert!(
        moved.0 > base.0 + 5.0,
        "arrastar para a DIREITA tem de levar o modelo para a direita: {} -> {}",
        base.0,
        moved.0
    );

    let mut down = front();
    law::orbit(&mut down, 0.0, 40.0);
    let tipped = centroid(&doc, &down).expect("a peça continua no quadro");
    assert!(
        tipped.1 > base.1 + 5.0,
        "arrastar para BAIXO tem de mostrar o TOPO — a face de cá desce na tela: {} -> {}",
        base.1,
        tipped.1
    );
}

/// **O pan leva a peça para onde a mão vai**, e o passo é em fração de tela: o mesmo arrasto move o
/// mesmo tanto de modelo em qualquer zoom.
#[test]
fn panning_carries_the_model_with_the_hand_at_any_zoom() {
    let doc = nose();
    let base = centroid(&doc, &front()).expect("a peça aparece");

    let mut moved = front();
    law::pan(&mut moved, 20.0, 12.0, H as f32 * 0.5);
    let after = centroid(&doc, &moved).expect("a peça continua no quadro");
    assert!(
        after.0 > base.0 + 5.0 && after.1 > base.1 + 3.0,
        "a peça tem de andar com a mão: ({}, {}) -> ({}, {})",
        base.0,
        base.1,
        after.0,
        after.1
    );

    // ⭐ E o deslocamento em PIXELS é o mesmo com outro zoom — é isso que "fração de tela"
    // significa, e o que impede o pan de ficar inútil de perto e insano de longe.
    let mut near = front();
    near.half_extent = 0.2;
    let base_near = centroid(&doc, &near).expect("a peça aparece de perto");
    let mut near_moved = near;
    law::pan(&mut near_moved, 20.0, 12.0, H as f32 * 0.5);
    let after_near = centroid(&doc, &near_moved).expect("a peça continua no quadro");
    let (a, b) = (after.0 - base.0, after_near.0 - base_near.0);
    assert!(
        (a - b).abs() < 2.0,
        "o mesmo arrasto tem de mover os mesmos pixels em qualquer zoom: {a:.1} contra {b:.1}"
    );
}

/// **A roda aproxima**, e a peça cresce na tela.
#[test]
fn the_wheel_zooms_in_and_the_part_grows() {
    let doc = nose();
    let cam = front();
    let before = trace_with(&doc, &cam, W, H, true, false).hits();
    let mut closer = cam;
    law::zoom(&mut closer, 3.0);
    let after = trace_with(&doc, &closer, W, H, true, false).hits();
    assert!(
        after > before,
        "três passos de roda para a frente têm de APROXIMAR: {before} -> {after} pixels"
    );
    assert!(
        closer.half_extent < cam.half_extent,
        "aproximar é reduzir o enquadramento"
    );
}

/// ⚠️ **Os limites de zoom existem e saturam** — e são os únicos dois números de faixa deste
/// arquivo, cada um com o recurso de que é (ver as constantes).
#[test]
fn the_zoom_saturates_instead_of_running_away() {
    let mut cam = front();
    for _ in 0..500 {
        law::zoom(&mut cam, 10.0);
    }
    assert!(
        cam.half_extent > 0.0 && cam.half_extent.is_finite(),
        "aproximar sem parar não pode chegar a zero nem a NaN: {}",
        cam.half_extent
    );
    let floor = cam.half_extent;
    let mut out = front();
    for _ in 0..500 {
        law::zoom(&mut out, -10.0);
    }
    assert!(
        out.half_extent.is_finite() && out.half_extent > floor,
        "afastar sem parar tem de saturar acima do piso: {}",
        out.half_extent
    );
}

/// ⭐ **A rotação é LIVRE: não há polo onde o gesto morra.**
///
/// Este gate substitui o que prendia a elevação em ±90°, e a troca é o registo de um veredito de
/// produto: *"só rotaciona em uma direção. não tem rot livre"* (Enio, 2026-08-19). A câmera de dois
/// ângulos batia na parede ao fim de ~105 px de arrasto para baixo — e a cura não foi um limite
/// maior, foi trocar a representação (`ph2d_field_render::Orbit`).
///
/// # ⚠️ A primeira versão deste gate mediu a coisa errada, e vale registar
///
/// Ela seguia o **centro de massa da peça na tela** e chamava "polo" a qualquer passo em que ele
/// não se mexia. Reprovou com **1 parada em 200** — e a parada era real e não era um polo: o
/// centroide de um ponto que gira num plano descreve uma senóide, e uma senóide tem **pontos de
/// retorno**, onde a velocidade é zero por geometria. *O sintoma que se procura e o sintoma que se
/// mede podem ter a mesma forma por motivos diferentes.*
///
/// O que se afirma agora é **exato**: cada chamada é uma rotação de `|dy|·k` em torno de um eixo
/// local fixo, logo a direção da vista tem de virar **exatamente** isso — nos 400 passos, que são
/// mais de uma volta inteira.
#[test]
fn vertical_dragging_never_hits_a_wall() {
    const DY: f32 = 2.0;
    // A lei da casa: 0,01 rad por pixel (ver `ORBIT_RAD_PER_PX`).
    const EXPECTED: f32 = DY * 0.01;
    let mut cam = front();
    let (_, _, mut prev) = cam.basis();
    let mut worst = f32::INFINITY;
    for _ in 0..400 {
        law::orbit(&mut cam, 0.0, DY);
        let (_, _, fwd) = cam.basis();
        let dot = (0..3)
            .map(|k| prev[k] * fwd[k])
            .sum::<f32>()
            .clamp(-1.0, 1.0);
        worst = worst.min(dot.acos());
        prev = fwd;
    }
    assert!(
        worst > EXPECTED * 0.99,
        "algum passo vertical virou só {worst} rad em vez de {EXPECTED} — isso é o polo da câmera \
         de dois ângulos, e é exatamente o que se removeu"
    );
}

/// **A tecla de repor devolve o enquadramento inicial** — a volta que a rotação livre torna
/// necessária, porque ela inclina o horizonte de propósito.
#[test]
fn home_restores_the_opening_view() {
    let home = Orbit::default();
    let mut cam = home;
    law::orbit(&mut cam, 137.0, -89.0);
    law::zoom(&mut cam, 6.0);
    law::pan(&mut cam, 50.0, -30.0, 60.0);
    assert!(cam != home, "o teste tem de partir de uma vista MEXIDA");

    law::home(&mut cam);
    assert_eq!(
        cam, home,
        "repor tem de devolver exatamente o enquadramento de abertura"
    );
}

/// ⭐ **A costura ponteiro → gizmo → peça**, no caminho de produção inteiro.
///
/// ⚠️ **É este o gate que a `DIRETIVA_IMPLEMENTACAO` §1 exige**, e não os dois de cima. Ele pergunta
/// *"clicar numa seta faz a peça andar?"* — a pergunta que a lei pura e a pintura, cada uma verde no
/// seu canto, **não** respondem. A causa nº 1 da semana perdida no Painter foi exatamente esta:
/// costura não-testada, com os dois lados dela corretos.
mod seam {
    use crate::field3d_gizmo::{self, Handle};
    use crate::field3d_input::{advance, begin, hot_handle};
    use crate::field3d_smoke::{Drag, set_armed_by_panel, with_smoke};
    use ph2d_field_render::Screen;

    const AREA: ph2d_editor::zones::Rect = ph2d_editor::zones::Rect {
        x: 40.0,
        y: 24.0,
        w: 800.0,
        h: 600.0,
    };

    /// Arma o módulo e põe o smoke num estado de quadro: com área desenhada e com o gizmo ancorado
    /// na origem. É o que a ponte com a cena publica.
    fn armed<R>(f: impl FnOnce(&mut crate::field3d_smoke::Smoke) -> R) -> R {
        set_armed_by_panel(true);
        with_smoke(|s| {
            s.area = Some(AREA);
            s.gizmo = Some(field3d_gizmo::Anchor {
                entity: 7,
                origin: [0.0, 0.0, 0.0],
            });
            s.pending_move = None;
            s.drag = None;
            s.gizmo_hot = None;
            f(s)
        })
        .expect("o módulo está armado")
    }

    fn translation_of(m: field3d_gizmo::Motion) -> [f32; 3] {
        match m {
            field3d_gizmo::Motion::Translate(d) => d,
            other => panic!("o modo de mover pede translação, e veio {other:?}"),
        }
    }

    fn screen_of(s: &crate::field3d_smoke::Smoke) -> Screen {
        Screen::new(AREA.w as u32, AREA.h as u32, s.cam.half_extent)
    }

    /// O ponto de janela, em pixels, do meio da haste do eixo `n`.
    fn mid_of_axis(s: &crate::field3d_smoke::Smoke, n: usize) -> (f32, f32) {
        let anchor = s.gizmo.expect("ancorado");
        let handles = field3d_gizmo::project(anchor, &s.cam, screen_of(s), s.gizmo_mode);
        let h = handles
            .iter()
            .find(|h| h.handle == Handle::Axis(n))
            .expect("o eixo existe");
        let field3d_gizmo::Shape::Arrow { from, to } = h.shape else {
            panic!("um eixo é uma seta");
        };
        (
            AREA.x + (from[0] + to[0]) * 0.5,
            AREA.y + (from[1] + to[1]) * 0.5,
        )
    }

    /// ⭐ **Carregar numa seta agarra a seta — e não orbita a câmera.**
    #[test]
    fn pressing_on_an_arrow_grabs_it_instead_of_orbiting() {
        armed(|s| {
            let p = mid_of_axis(s, 0);
            let before = s.cam;
            assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, p));
            assert_eq!(
                s.drag,
                Some(Drag::Gizmo(Handle::Axis(0))),
                "o clique sobre a seta virou gesto de câmera — a alça está pintada e morta"
            );
            assert_eq!(hot_handle(s), Some(Handle::Axis(0)), "e ela acende");

            // E arrastar move a PEÇA, não a vista.
            assert!(advance(s, p.0 + 60.0, p.1));
            assert_eq!(s.cam, before, "a câmera não pode ter-se mexido");
            let (entity, motion) = s.pending_move.expect("o arrasto tem de pedir um movimento");
            assert_eq!(entity, 7, "e tem de pedi-lo para a entidade da âncora");
            assert!(
                !motion.is_idle(),
                "o pedido saiu vazio: {motion:?} — o ponteiro não chegou à lei do arrasto"
            );
        });
    }

    /// **Longe do gizmo, o botão esquerdo continua a orbitar.** Sem isto o gizmo sequestraria a
    /// navegação da janela inteira.
    #[test]
    fn pressing_away_from_the_gizmo_still_orbits() {
        armed(|s| {
            let far = (AREA.x + AREA.w - 5.0, AREA.y + 5.0);
            assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, far));
            assert_eq!(s.drag, Some(Drag::Orbit));
            assert!(s.pending_move.is_none());
        });
    }

    /// ⚠️ **O botão DIREITO orbita mesmo por cima da alça** — é a saída de quem quer girar a vista
    /// sem primeiro tirar o rato de cima da peça.
    #[test]
    fn the_right_button_orbits_even_over_a_handle() {
        armed(|s| {
            let p = mid_of_axis(s, 0);
            assert!(begin(s, winit::event::MouseButton::Right, Drag::Orbit, p));
            assert_eq!(s.drag, Some(Drag::Orbit));
        });
    }

    /// ⭐ **Os pedidos ACUMULAM entre quadros.**
    ///
    /// ⚠️ Entre dois quadros chegam vários eventos de ponteiro. Guardar só o último faria a peça
    /// andar menos do que a mão — devagar, e **só quando o rato vai depressa**, que é o defeito mais
    /// difícil de acreditar quando alguém o reporta.
    #[test]
    fn pointer_events_between_two_frames_add_up() {
        armed(|s| {
            let p = mid_of_axis(s, 0);
            begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
            advance(s, p.0 + 30.0, p.1);
            let one = translation_of(s.pending_move.expect("primeiro evento").1);
            advance(s, p.0 + 60.0, p.1);
            let two = translation_of(s.pending_move.expect("segundo evento").1);
            assert!(
                (two[0] - one[0] * 2.0).abs() < one[0].abs() * 1e-3,
                "dois passos iguais têm de somar: {one:?} depois {two:?}"
            );
        });
    }

    /// **Sem arrasto, mover o rato acende a alça e NÃO consome o evento.**
    ///
    /// ⚠️ As duas metades importam: sem a primeira o artista não sabe o que vai agarrar; com a
    /// segunda invertida, a janela 3D engoliria todo movimento de rato do app 2D.
    #[test]
    fn hover_lights_the_handle_without_swallowing_the_event() {
        armed(|s| {
            let p = mid_of_axis(s, 1);
            assert!(!advance(s, p.0, p.1), "hover não é um gesto desta janela");
            assert_eq!(hot_handle(s), Some(Handle::Axis(1)));
            assert!(!advance(s, AREA.x + 2.0, AREA.y + 2.0));
            assert_eq!(hot_handle(s), None, "e apaga-se ao sair");
        });
    }

    /// ⚠️ **Durante o arrasto o realce fica na alça AGARRADA**, mesmo com o cursor longe dela — que
    /// é onde o cursor está, porque arrastar é isso.
    #[test]
    fn the_grabbed_handle_stays_lit_while_the_cursor_walks_away() {
        armed(|s| {
            let p = mid_of_axis(s, 2);
            begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
            advance(s, AREA.x + AREA.w - 3.0, AREA.y + AREA.h - 3.0);
            assert_eq!(hot_handle(s), Some(Handle::Axis(2)));
        });
    }
}

/// Os gates do **undo de um arrasto**.
///
/// ⚠️ A lei do shell («um gesto em andamento espera o fim») lê o `held_button`, e o gancho deste
/// módulo consome o `Down` e volta **antes** da linha que o escreve. A lei estava certa e **não
/// alcançava este gesto** — arrastar uma seta registava um passo de undo por quadro.
mod undo_seam {
    use crate::field3d_gizmo::Handle;
    use crate::field3d_smoke::{Drag, gesture_in_progress, set_armed_by_panel, with_smoke};

    fn armed<R>(f: impl FnOnce(&mut crate::field3d_smoke::Smoke) -> R) -> R {
        set_armed_by_panel(true);
        with_smoke(f).expect("o módulo está armado")
    }

    /// ⭐ **Só o arrasto do gizmo é um gesto de AUTORIA.**
    ///
    /// Orbitar e deslocar a vista não tocam no documento; suprimir o undo neles não estragaria
    /// nada, mas afirmaria uma coisa falsa sobre o que eles fazem — e um dia alguém acreditaria.
    #[test]
    fn only_a_gizmo_drag_counts_as_a_gesture_in_progress() {
        armed(|s| s.drag = None);
        assert!(!gesture_in_progress(), "parado não é gesto");

        armed(|s| s.drag = Some(Drag::Orbit));
        assert!(!gesture_in_progress(), "girar a vista não autora nada");

        armed(|s| s.drag = Some(Drag::Pan));
        assert!(!gesture_in_progress(), "deslocar a vista também não");

        armed(|s| s.drag = Some(Drag::Gizmo(Handle::Axis(1))));
        assert!(gesture_in_progress(), "mover a peça É autoria");

        armed(|s| s.drag = None);
        assert!(!gesture_in_progress(), "e soltar fecha o gesto");
    }

    /// ⚠️ **O `post_frame_undo` tem de PERGUNTAR.**
    ///
    /// Este gate lê a fonte, e diz exatamente o que prova: que **o cano está ligado**. Ele não
    /// prova que a supressão funciona — isso é a lei do shell, que já tem os gates dela. O que ele
    /// impede é o modo de falha que este módulo acabou de pagar: as duas metades corretas, e
    /// ninguém a ligá-las. É a fiação órfã da `DIRETIVA_IMPLEMENTACAO` §1.
    #[test]
    fn the_undo_pass_asks_whether_this_module_is_mid_gesture() {
        let src = include_str!("undo.rs");
        assert!(
            src.contains("field3d_smoke::gesture_in_progress()"),
            "o `post_frame_undo` deixou de perguntar — um arrasto volta a ser N passos de undo"
        );
    }
}
