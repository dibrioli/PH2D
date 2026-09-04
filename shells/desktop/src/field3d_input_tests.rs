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
            mods: Vec::new(),
            verb: None,
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
    let g = trace_with(
        doc,
        &crate::field3d_smoke::sampled_registry(),
        cam,
        W,
        H,
        true,
        false,
    );
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

    // ⭐ E o deslocamento em PIXELS é **exatamente** o do arrasto, em qualquer zoom e com qualquer
    // lente — é isso que "fração de tela" significa, e o que impede o pan de ficar inútil de perto e
    // insano de longe.
    //
    // ⚠️ **Isto media o centroide do TRAÇADO, e a fixture deixou de conter o fenómeno** quando a
    // lente convergente entrou: a `half_extent = 0,2` põe o olho a 0,55 da peça, ela transborda o
    // quadro, e o centroide de um quadro cheio é o centro dele — parado, com a lei correta por
    // trás. *Uma fixture só prova o que ela contém.*
    //
    // A pergunta certa é sobre um ponto do **plano do alvo**, que é onde a lei do pan fala: mover o
    // alvo `half_extent/half_px · dx` desloca-o `dx` pixels, e o fator da lente vale 1 ali. Um
    // ponto, e não uma peça: uma peça tem profundidade, e sob convergência cada pedaço dela anda um
    // tanto diferente — o que é a lente a funcionar, não o pan a falhar.
    for lens in [
        ph2d_field_render::Lens::Ortho,
        ph2d_field_render::Lens::Perspective {
            half_fov: ph2d_field_render::DEFAULT_HALF_FOV,
        },
    ] {
        for zoom in [0.2f32, 0.8, 3.0] {
            let mut cam = front();
            cam.lens = lens;
            cam.half_extent = zoom;
            let screen = ph2d_field_render::Screen::new(W, H, cam.half_extent);
            // Um ponto SOBRE o plano do alvo, e fora do centro — o centro andaria com o alvo.
            let (right, _, _) = cam.basis();
            let mark = [
                cam.target[0] + right[0] * zoom * 0.3,
                cam.target[1] + right[1] * zoom * 0.3,
                cam.target[2] + right[2] * zoom * 0.3,
            ];
            let before = cam.project(mark, screen).expect("está à frente do olho").0;
            law::pan(&mut cam, 20.0, 12.0, H as f32 * 0.5);
            let screen = ph2d_field_render::Screen::new(W, H, cam.half_extent);
            let after = cam.project(mark, screen).expect("continua à frente").0;
            let moved = (after[0] - before[0], after[1] - before[1]);
            assert!(
                (moved.0 - 20.0).abs() < 0.05 && (moved.1 - 12.0).abs() < 0.05,
                "{lens:?} @ zoom {zoom}: o arrasto foi (20, 12) px e o ponto andou {moved:?}"
            );
        }
    }
}

/// **A roda aproxima**, e a peça cresce na tela.
#[test]
fn the_wheel_zooms_in_and_the_part_grows() {
    let doc = nose();
    let cam = front();
    let before = trace_with(
        &doc,
        &crate::field3d_smoke::sampled_registry(),
        &cam,
        W,
        H,
        true,
        false,
    )
    .hits();
    let mut closer = cam;
    law::zoom(&mut closer, 3.0);
    let after = trace_with(
        &doc,
        &crate::field3d_smoke::sampled_registry(),
        &closer,
        W,
        H,
        true,
        false,
    )
    .hits();
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

#[path = "field3d_input_seam_tests.rs"]
mod seam;

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
        let src = concat!(include_str!("undo.rs"), include_str!("undo_app.rs"));
        assert!(
            src.contains("field3d_smoke::gesture_in_progress()"),
            "o `post_frame_undo` deixou de perguntar — um arrasto volta a ser N passos de undo"
        );
        // ⭐⭐⭐ **E a metade de W115**: sem esta, uma forma nascida da paleta não tem passo próprio
        // e funde-se na acção seguinte. ⚠️ Tem de ser lida **ao lado** do `any_input_this_frame`,
        // isto é ANTES de qualquer `return` — deixá-la pousada registaria um passo repetido.
        assert!(
            src.contains("field3d_smoke::take_authored_change()"),
            "o `post_frame_undo` deixou de ler o que o módulo autorou sem evento — a forma da \
             paleta volta a nascer sem passo"
        );
        let marca = src
            .find("field3d_smoke::take_authored_change()")
            .expect("lê a marca");
        let guarda = src
            .find("let motivo = if !had_input")
            .expect("a guarda dos cinco motivos");
        assert!(
            marca < guarda,
            "a marca é lida DEPOIS da guarda — nos quadros suprimidos ela fica pousada e o passo \
             seguinte sai duplicado"
        );
    }

    /// ⭐⭐⭐ **AS TRÊS SAÍDAS DE UM GESTO DE GIZMO MARCAM O QUADRO — e o `Cancel` não** (W113).
    ///
    /// # ⛔⛔ O report do Enio (2026-09-03): *«o undo/redo não obedece cada etapa, principalmente se
    /// transformação»*
    ///
    /// Um arrasto de alça acaba de **três** maneiras, e até aqui só uma delas dizia ao undo que a
    /// cena tinha sido autorada:
    ///
    /// | saída | marcava o quadro? | o mundo mudou? |
    /// |---|---|---|
    /// | largar o botão (`finish`) | ✅ | sim |
    /// | `Enter` / número (`typed_key`) | ❌ | **sim** |
    /// | `G`/`R`/`S` (`mode_key`) | ❌ | **sim** (o aplicado FICA) |
    /// | `Esc` (`Cancel`) | ❌ | **não** — net zero, e por isso está certo |
    ///
    /// ⇒ acabar uma transformação com `Enter` não registava passo NENHUM: ela colava-se à ação
    /// seguinte do artista. *Uma lei escrita em três sítios ainda não é uma lei.*
    #[test]
    fn every_exit_from_a_gizmo_gesture_marks_the_frame_except_the_one_that_undoes_it() {
        use crate::field3d_gizmo::Mode;
        use crate::field3d_input::{mode_key, typed_key};
        use crate::field3d_typed::Stroke;

        // 1. Largar o botão.
        armed(|s| s.drag = Some(Drag::Gizmo(Handle::Axis(1))));
        let (took, authored) = armed(crate::field3d_input::finish);
        assert!(took && authored, "largar o botão autora a cena");

        // 2. `Enter` — a saída que o report pagou.
        armed(|s| s.drag = Some(Drag::Gizmo(Handle::Axis(1))));
        assert_eq!(
            armed(|s| typed_key(s, Stroke::Commit)),
            (true, true),
            "fechar com Enter autora tanto quanto largar o botão"
        );

        // 3. Trocar de verbo a meio — o que já foi aplicado FICA, logo é autoria.
        armed(|s| s.drag = Some(Drag::Gizmo(Handle::Axis(1))));
        assert_eq!(
            armed(|s| mode_key(s, Mode::Rotate)),
            (true, true),
            "trocar de verbo confirma o que se fez e tem de registar passo"
        );

        // ⛔ E o CONTROLE que impede isto de virar «marque sempre»: sem arrasto nenhum, trocar de
        // verbo não autora nada — senão cada tecla `G` viraria um passo de undo vazio.
        armed(|s| s.drag = None);
        assert_eq!(
            armed(|s| mode_key(s, Mode::Move)),
            (true, false),
            "trocar de verbo PARADO não é autoria"
        );

        // ⛔ E o `Cancel` repõe a pose, logo o quadro NÃO é autorado.
        armed(|s| s.drag = Some(Drag::Gizmo(Handle::Axis(1))));
        assert_eq!(
            armed(|s| typed_key(s, Stroke::Cancel)),
            (true, false),
            "cancelar devolve a peça ao sítio — um passo ali seria um passo vazio"
        );

        // ⛔ E um dígito NÃO fecha o gesto: ele continua aberto e o passo espera.
        armed(|s| s.drag = Some(Drag::Gizmo(Handle::Axis(1))));
        assert_eq!(
            armed(|s| typed_key(s, Stroke::Digit(b'5'))),
            (true, false),
            "um dígito não acaba o gesto"
        );
    }

    /// ⭐⭐⭐ **UMA MUDANÇA SERVIDA NUM QUADRO SEM EVENTO DECLARA-SE** (W115).
    ///
    /// # ⛔⛔ O report, com o log do próprio app na mão (Enio, 2026-09-03)
    ///
    /// ```text
    /// [undo] ⛔ o documento MUDOU e o passo foi SUPRIMIDO — motivo: sem entrada neste quadro
    /// ```
    ///
    /// A forma que a paleta escolhe e a escultura que o diálogo carrega chegam por **pedido servido
    /// noutro quadro** — o pick é consumido pelo modal, e o **mundo** só é escrito no quadro
    /// seguinte, que já não tem evento nenhum. ⇒ elas nasciam **sem passo próprio** e fundiam-se na
    /// acção seguinte do artista: dois gestos, um `Ctrl+Z`.
    ///
    /// ⚠️ **É um EVENTO e não um estado** — a segunda leitura tem de dar `false`, senão a próxima
    /// supressão legítima registaria um passo que já foi registado.
    #[test]
    fn an_authored_change_served_on_an_eventless_frame_declares_itself() {
        use crate::field3d_smoke::{mark_authored_change, take_authored_change};
        // Um quadro qualquer não autora nada.
        let _ = take_authored_change();
        assert!(!take_authored_change(), "sem marca, o quadro é mudo");
        mark_authored_change();
        assert!(take_authored_change(), "a marca chega ao undo");
        assert!(
            !take_authored_change(),
            "e vale UMA vez — deixá-la pousada registaria um passo que já foi registado"
        );
    }

    /// ⭐⭐ **E a PALETA marca** — a costura, pelo caminho real (`sync_scene_and_birth`).
    ///
    /// ⚠️ A lei pura acima não prova que alguém a chama; este gate arma o pedido exactamente como o
    /// modal o arma e corre a ponte que serve o mundo.
    #[test]
    fn a_shape_born_from_the_palette_marks_the_frame_as_authored() {
        use crate::field3d_smoke::{ask_shape, take_authored_change};
        let _ = take_authored_change();
        let mut sim = ph2d_ecs::SimWorld::new();
        // Uma peça qualquer: o pedido precisa de uma RAIZ para pendurar a forma nova.
        let doc = ph2d_field::FieldDoc::new(
            vec![ph2d_field::Node::new(
                ph2d_field::Xform::IDENTITY,
                ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere { radius: 0.4 }),
            )],
            ph2d_field::NodeId(0),
        )
        .expect("documento válido");
        crate::field3d_scene::sync_scene(&mut sim, Some(&doc), 0.0);
        let _ = take_authored_change();
        ask_shape(0);
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            take_authored_change(),
            "a forma da paleta nasceu num quadro que não se declarou — ela funde-se na acção \
             seguinte, e o artista vê o undo pular uma etapa"
        );
    }

    /// ⚠️ **E o `App` tem de LER as duas metades** — a costura que os gates acima não alcançam.
    ///
    /// A lei pura devolve `autorou`; se quem a chama descartar essa metade, tudo volta ao estado do
    /// report. É o mesmo modo de falha (e o mesmo remédio) do
    /// [`the_undo_pass_asks_whether_this_module_is_mid_gesture`].
    #[test]
    fn the_shell_feeds_every_gesture_exit_into_the_undo_input_mark() {
        let src = include_str!("field3d_input.rs");
        let marcas = src
            .matches("self.any_input_this_frame |= authored;")
            .count();
        assert!(
            marcas >= 4,
            "só {marcas} saídas alimentam a marca de entrada do undo — as quatro são mover, \
             soltar, `Enter` e a tecla de verbo"
        );
    }
}

/// Os gates das **teclas de verbo** — `G` / `R` / `S`.
mod mode_keys {
    use crate::field3d_gizmo::Mode;
    use crate::field3d_input::mode_for_key;
    use winit::keyboard::{KeyCode, ModifiersState};

    /// As três letras do Blender, sem modificador, nomeiam os três verbos.
    #[test]
    fn the_three_blender_letters_name_the_three_verbs() {
        let none = ModifiersState::empty();
        assert_eq!(mode_for_key(KeyCode::KeyG, none), Some(Mode::Move));
        assert_eq!(mode_for_key(KeyCode::KeyR, none), Some(Mode::Rotate));
        assert_eq!(mode_for_key(KeyCode::KeyS, none), Some(Mode::Scale));
        assert_eq!(mode_for_key(KeyCode::KeyX, none), None);
    }

    /// ⭐ **`Ctrl+S` não é um atalho de gizmo — é o SALVAR do app.**
    ///
    /// ⚠️ A guarda de *"ponteiro sobre a janela 3D"* protege os campos de texto; ela **não** protege
    /// os atalhos GLOBAIS, que valem em qualquer sítio da janela. Sem esta segunda guarda, guardar o
    /// projeto com o rato em cima da peça trocava o gizmo para *Size* e **não salvava nada**, em
    /// silêncio — e o artista descobriria ao fechar o app.
    #[test]
    fn a_modifier_makes_it_someone_elses_shortcut() {
        for m in [
            ModifiersState::CONTROL,
            ModifiersState::ALT,
            ModifiersState::SUPER,
        ] {
            for code in [KeyCode::KeyG, KeyCode::KeyR, KeyCode::KeyS] {
                assert_eq!(
                    mode_for_key(code, m),
                    None,
                    "{code:?} com {m:?} tem de passar adiante — é atalho de outra pessoa"
                );
            }
        }
    }

    /// ⚠️ **O `Shift` fica de fora da proibição, de propósito:** `Shift+S` continua a ser um `S`, e
    /// nenhum atalho global da casa o usa. Proibi-lo custaria o atalho a quem escreve com Caps.
    #[test]
    fn shift_alone_is_still_the_letter() {
        assert_eq!(
            mode_for_key(KeyCode::KeyR, ModifiersState::SHIFT),
            Some(Mode::Rotate)
        );
    }
}

/// ⭐ **`Numpad5` alterna a lente, e volta.**
///
/// ⚠️ A lei é pura de propósito ([`law::other_lens`]): a porta da tecla não pode ser a única forma
/// de a exercer, senão a troca só se prova abrindo uma janela. E a volta importa — uma troca que só
/// funcionasse num sentido deixaria o artista preso na lente que ele escolheu para comparar.
#[test]
fn the_lens_key_toggles_and_comes_back() {
    use ph2d_field_render::Lens;
    let start = Lens::Perspective {
        half_fov: ph2d_field_render::DEFAULT_HALF_FOV,
    };
    let flipped = law::other_lens(start);
    assert_eq!(flipped, Lens::Ortho, "a convergente troca para a paralela");
    assert_eq!(
        law::other_lens(flipped),
        start,
        "e volta com a abertura da REFERÊNCIA, não com uma lembrada"
    );
}

/// ⭐ **A câmera nasce CONVERGENTE** — é o que um modelador espera, e é a escolha declarada.
///
/// ⚠️ A nota que estava na câmera dizia que a perspectiva *"merece a sua própria comparação lado a
/// lado, não uma troca silenciosa"*. A comparação é a tecla; este gate prende o **default**, que é
/// a metade que uma tecla não prova.
#[test]
fn the_camera_is_born_converging() {
    use ph2d_field_render::{Lens, Orbit};
    assert!(
        matches!(Orbit::default().lens, Lens::Perspective { .. }),
        "o default é a lente convergente"
    );
    assert!(
        Orbit::default().eye_distance().is_some(),
        "e por isso ela tem olho"
    );
    // ⚠️ E a paralela **não** tem — o `None` é o que impede a conta da convergência de correr lá.
    let flat = Orbit {
        lens: Lens::Ortho,
        ..Orbit::default()
    };
    assert!(flat.eye_distance().is_none());
}
