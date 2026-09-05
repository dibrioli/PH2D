//! ⭐ **O UNDO DE UM ARRASTO na janela 3D** — a lei do shell («um gesto em andamento espera o fim»)
//! aplicada ao gizmo.
//!
//! # Por que um arquivo irmão
//!
//! O [`super`] responde *«o modelo segue a minha mão?»*; este responde *«e quantos passos de undo
//! isso deixa?»*. O irmão passou as `600` linhas do gate de LOC do shell — ⛔ *split, nunca a marca
//! de isenção.*

//!
//! ⚠️ A lei do shell («um gesto em andamento espera o fim») lê o `held_button`, e o gancho deste
//! módulo consome o `Down` e volta **antes** da linha que o escreve. A lei estava certa e **não
//! alcançava este gesto** — arrastar uma seta registava um passo de undo por quadro.

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
    // ⚠️ **Pela CABEÇA da guarda, e não pelo primeiro motivo** (2026-09-04): a ordem dos
    // cinco inverteu-se — um FACTO do app (gesto em curso) passou à frente de uma AUSÊNCIA
    // (sem eventos), porque com a ordem antiga o log dizia *«sem entrada neste quadro»* sobre
    // quadros que eram um arrasto, e mandou uma jornada inteira caçar um fantasma. *Um gate
    // que se agarra ao PRIMEIRO ramo de uma cadeia proíbe reordená-la.*
    let guarda = src
        .find("let motivo = if ")
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
