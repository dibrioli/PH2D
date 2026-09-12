//! Gates da PILHA DE APARÊNCIA do lado da shell (v20) — a vista publicada, os verbos, e a
//! resolução de um clique num id de runtime.

use super::{
    StackVerb, apply, published, set_blend, set_offset, set_opacity, set_width, stack_verb_for_id,
};
use ph2d_vec_scene::{
    MAX_PAINT_LAYERS, Paint, PaintEntry, PaintKind, Rgba8, StrokeSpec, VecPath, VecScene, VecVertex,
};

fn cena() -> (VecScene, Vec<u64>) {
    let mut scene = VecScene::default();
    let ids: Vec<u64> = (0..2)
        .map(|_| {
            scene.push_path(VecPath {
                verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
                    .map(VecVertex::corner)
                    .to_vec(),
                closed: true,
                ..VecPath::default()
            })
        })
        .collect();
    (scene, ids)
}

/// ⭐⭐⭐ **UM CLIQUE NUM ID DE RUNTIME RESOLVE NO VERBO E NO ÍNDICE CERTOS** — e um id que não é da
/// pilha não resolve em nada.
///
/// ⚠️ **A varredura é do espaço FIXO** (`MAX_PAINT_LAYERS`): perguntar «que índice é este?» a uma
/// lista do tamanho de hoje faria o mesmo clique resolver diferente conforme a forma seleccionada.
#[test]
fn a_click_on_a_runtime_id_resolves_to_the_right_verb_and_index() {
    use ph2d_editor::ids;
    assert_eq!(
        stack_verb_for_id(ids::VECTOR_PAINT_ADD_FILL),
        Some(StackVerb::AddFill)
    );
    assert_eq!(
        stack_verb_for_id(ids::VECTOR_PAINT_ADD_STROKE),
        Some(StackVerb::AddStroke)
    );
    for i in [0usize, 1, MAX_PAINT_LAYERS - 1] {
        assert_eq!(
            stack_verb_for_id(ids::vector_paint_eye_id(i)),
            Some(StackVerb::Eye(i))
        );
        assert_eq!(
            stack_verb_for_id(ids::vector_paint_up_id(i)),
            Some(StackVerb::Up(i))
        );
        assert_eq!(
            stack_verb_for_id(ids::vector_paint_del_id(i)),
            Some(StackVerb::Del(i))
        );
    }
    assert_eq!(
        stack_verb_for_id(ids::VECTOR_OBJ_OPACITY),
        None,
        "um id que nao e' da pilha nao pode resolver nela"
    );
}

/// ⭐⭐ **UMA CAMADA NOVA NASCE VISÍVEL E NO TOPO.**
///
/// ⚠️ *Visível* não é enfeite: uma camada nova transparente faria o artista carregar no botão, ver
/// nada mudar, e concluir que ele está morto — o defeito que o `CLAUDE.md` §5.0 nomeia como o mais
/// caro de todos, porque um controlo morto e um sem efeito visível leem-se igual.
#[test]
fn a_new_layer_is_born_visible_and_on_top() {
    let (mut scene, ids) = cena();
    assert!(apply(&mut scene, &ids, StackVerb::AddFill));
    assert!(apply(&mut scene, &ids, StackVerb::AddStroke));
    for id in &ids {
        let p = scene.path(*id).expect("a forma");
        assert_eq!(p.paints.len(), 2, "as DUAS formas da seleccao receberam");
        assert!(matches!(p.paints[0].kind, PaintKind::Fill(_)));
        assert!(
            matches!(p.paints[1].kind, PaintKind::Stroke(_)),
            "a ultima acrescentada fica no TOPO (o fim do vector)"
        );
        assert!(p.paints.iter().all(|e| e.enabled));
        assert!(
            p.paints[0].swatch_color().a == 255,
            "e ela e' opaca — uma camada nova invisivel le-se como um botao morto"
        );
    }
}

/// **Subir move para o TOPO** — e a inversão do painel não entra aqui: o índice que chega é o do
/// documento.
#[test]
fn moving_a_layer_up_moves_it_towards_the_top() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddFill);
    apply(&mut scene, &ids, StackVerb::AddStroke);
    assert!(apply(&mut scene, &ids, StackVerb::Up(0)));
    let p = scene.path(ids[0]).expect("a forma");
    assert!(
        matches!(p.paints[1].kind, PaintKind::Fill(_)),
        "o preenchimento subiu para o topo"
    );
    assert!(
        !apply(&mut scene, &ids, StackVerb::Up(1)),
        "no topo, subir nao faz nada — e nao estoura"
    );
    assert!(
        !apply(&mut scene, &ids, StackVerb::Down(0)),
        "no chao, descer idem"
    );
}

/// **O OLHO desarma sem perder a tinta**, e apagar remove a camada CERTA.
#[test]
fn the_eye_disarms_without_losing_the_paint_and_delete_removes_the_right_one() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddFill);
    apply(&mut scene, &ids, StackVerb::AddStroke);
    assert!(apply(&mut scene, &ids, StackVerb::Eye(0)));
    let p = scene.path(ids[0]).expect("a forma");
    assert!(!p.paints[0].enabled);
    assert!(
        p.paints[0].swatch_color().a == 255,
        "desarmar nao pode custar os parametros"
    );
    assert!(apply(&mut scene, &ids, StackVerb::Del(0)));
    let p = scene.path(ids[0]).expect("a forma");
    assert_eq!(p.paints.len(), 1);
    assert!(
        matches!(p.paints[0].kind, PaintKind::Stroke(_)),
        "sobrou o contorno, que era o de cima"
    );
}

/// ⛔ **O TECTO é honrado no sítio que ESCREVE**, e não só no que pinta.
///
/// ⚠️ O painel esconde os botões quando a pilha está cheia — mas esconder é metade: um gesto que
/// chegue por outra via (um atalho, um teste, um bug de ordem) tem de ser recusado aqui.
#[test]
fn the_ceiling_is_honoured_where_the_write_happens() {
    let (mut scene, ids) = cena();
    for _ in 0..MAX_PAINT_LAYERS {
        assert!(apply(&mut scene, &ids, StackVerb::AddFill));
    }
    assert_eq!(
        scene.path(ids[0]).expect("a forma").paints.len(),
        MAX_PAINT_LAYERS
    );
    assert!(
        !apply(&mut scene, &ids, StackVerb::AddFill),
        "no tecto, acrescentar nao muda nada"
    );
}

/// ⭐ **A VISTA publica o PRIMÁRIO, e a escrita alcança TODOS** — a lei desta janela, agora com a
/// pilha dentro.
#[test]
fn the_view_reads_the_primary_and_the_write_reaches_them_all() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddStroke);
    scene
        .path_mut(ids[0])
        .expect("a")
        .paints
        .push(PaintEntry::fill(Paint::Solid(Rgba8::new(1, 2, 3, 255))));

    let v = published(&scene, &ids).expect("ha' seleccao");
    assert_eq!(
        v.layers.len(),
        2,
        "a vista e' a do PRIMARIO: {:?}",
        v.layers
    );
    assert!(!v.layers[0].is_fill && v.layers[1].is_fill);

    assert!(set_width(&mut scene, &ids, 0, 5.0));
    for id in &ids {
        let PaintKind::Stroke(s) = &scene.path(*id).expect("a forma").paints[0].kind else {
            panic!("a camada 0 e' um contorno nas duas");
        };
        assert!((s.width - 5.0).abs() < 1e-9, "a escrita alcanca a seleccao");
    }
    assert!(
        !set_width(&mut scene, &ids, 9, 5.0),
        "um indice que nao existe e' SALTADO, nunca criado"
    );
}

/// **Escrever o MESMO valor devolve `false`** — é o que impede um passo de undo por quadro.
#[test]
fn writing_the_same_value_reports_no_change() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddFill);
    assert!(set_opacity(&mut scene, &ids, 0, 0.5));
    assert!(!set_opacity(&mut scene, &ids, 0, 0.5));
    assert!(set_blend(&mut scene, &ids, 0, 1));
    assert!(!set_blend(&mut scene, &ids, 0, 1));
}

/// ⛔ **Abrir uma linha NÃO é uma mudança de documento** — senão cada clique de UI viraria um passo
/// de undo, que é o defeito que a captura por DIFF desta casa existe para não ter.
#[test]
fn opening_a_row_is_a_view_change_and_never_a_document_one() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddFill);
    let antes = scene.clone();
    assert!(!apply(&mut scene, &ids, StackVerb::Open(0)));
    assert_eq!(scene, antes, "o documento nao se mexeu");
}

/// **Um contorno novo tem largura**, e uma camada de preenchimento não aceita largura nenhuma.
#[test]
fn a_new_stroke_has_a_width_and_a_fill_layer_refuses_one() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddStroke);
    apply(&mut scene, &ids, StackVerb::AddFill);
    let PaintKind::Stroke(s) = &scene.path(ids[0]).expect("a").paints[0].kind else {
        panic!("a 0 e' contorno");
    };
    assert!(
        s.width > 0.0,
        "um contorno de largura zero nao desenha nada"
    );
    assert!(
        !set_width(&mut scene, &ids, 1, 4.0),
        "a camada de preenchimento nao tem largura para escrever"
    );
    let _ = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0);
}

/// ⭐⭐⭐ **O DESLOCAMENTO ESCREVE-SE EM TODA A SELECÇÃO, E É IDEMPOTENTE.**
///
/// ⚠️ **As duas metades, e a segunda é a que importa para o undo:** escrever o MESMO deslocamento
/// devolve `false`, senão um arrasto de campo registaria um passo por quadro. É a lei que o
/// `set_opacity` e o `set_blend` ao lado já pagam.
#[test]
fn a_layers_offset_is_written_to_the_whole_selection_and_is_idempotent() {
    let (mut scene, ids) = cena();
    apply(&mut scene, &ids, StackVerb::AddFill);
    assert!(set_offset(&mut scene, &ids, 0, [3.0, -2.0]));
    assert!(
        !set_offset(&mut scene, &ids, 0, [3.0, -2.0]),
        "reescrever o mesmo deslocamento contou como mudanca de documento"
    );
    for id in &ids {
        assert_eq!(
            scene.path(*id).expect("forma").paints[0].offset,
            [3.0, -2.0],
            "a forma {id:?} nao recebeu o deslocamento"
        );
    }
    // ⛔ Um índice que não existe é SALTADO, nunca criado — a lei do cabeçalho deste módulo.
    assert!(!set_offset(&mut scene, &ids, 7, [1.0, 1.0]));
}
