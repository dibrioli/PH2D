//! **Os gates de Split by rows, do ◆ e da tira que rola** (E3 da [ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)).
//! Cada gesto pelas portas reais da aba. Filhos de [`super`] (a `Mesa`).

use super::*;
use ph2d_ecs::SpriteAnimMarkers;
use ph2d_editor_core::SheetImage;
use std::sync::Arc;

/// A folha de 4 × 3 células de 2 px: a 1.ª fileira cheia, a 2.ª transparente, a 3.ª só com as células 9 e 10.
fn folha_com_fileira_vazia() -> SheetImage {
    let (w, h) = (8u32, 6u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (c, r) = (x / 2, y / 2);
            if r == 0 || (r == 2 && (c == 1 || c == 2)) {
                rgba[((y * w + x) * 4 + 3) as usize] = 255;
            }
        }
    }
    SheetImage::new(Arc::new(rgba), w, h).expect("a folha")
}

/// A sprite com essa folha na grelha 4 × 3 e as animações `lib`: a aba lê-a como a lê da cache.
fn com_a_folha(m: &mut Mesa, lib: SpriteAnimations) {
    let e = Entity::from_bits(m.e);
    let grade = SpriteGrid {
        hframes: 4,
        vframes: 3,
        frame: 0,
        sheet: 0,
    };
    m.sim.world_mut().entity_mut(e).insert((grade, lib));
    m.frame();
    pinta_a_folha(m);
}

fn pinta_a_folha(m: &mut Mesa) {
    m.doc().snap.sheet = Some(folha_com_fileira_vazia());
    m.paint();
}

fn nomes(m: &Mesa) -> Vec<(String, u32, u32)> {
    m.sim
        .world()
        .get::<SpriteAnimations>(Entity::from_bits(m.e))
        .expect("biblioteca")
        .iter()
        .map(|t| (t.name.clone(), t.from, t.to))
        .collect()
}

fn marcas(m: &Mesa) -> Vec<(String, u32, String)> {
    m.sim
        .world()
        .get::<SpriteAnimMarkers>(Entity::from_bits(m.e))
        .map(|k| {
            k.0.iter()
                .map(|x| (x.tag.clone(), x.step, x.signal.clone()))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_transparencia_decide_quais_fileiras_tem_desenho() {
    let s = folha_com_fileira_vazia();
    assert_eq!(
        ph2d_app_sprite_anim::editor::split::drawn_by_row(&s.rgba, s.w, s.h, 4, 3),
        vec![vec![0, 1, 2, 3], vec![9, 10]]
    );
}

/// ⭐ Um clique: uma animação por fileira com desenho, sem a transparente, numa só edição (um passo de desfazer). O
/// «loop» da folha inteira não cobre fileira nenhuma, e o nome em uso é saltado.
/// *Mutação: tirar o `seq.iter().all` de `covered` ⇒ o loop cobre tudo e nada nasce.*
#[test]
fn split_by_rows_cria_uma_por_fileira_ocupada_num_so_gesto() {
    let mut m = Mesa::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("loop", 0, 11)).unwrap();
    lib.insert(AnimationTag::new("row 1", 4, 5)).unwrap();
    com_a_folha(&mut m, lib);
    let botao = m.doc().layout.split;
    m.click(centre(botao));
    assert_eq!(m.pending().len(), 1, "uma edição, um passo de desfazer");
    m.frame();
    assert_eq!(
        nomes(&m),
        [
            ("loop".to_owned(), 0, 11),
            ("row 1".to_owned(), 4, 5),
            ("row 2".to_owned(), 0, 3),
            ("row 3".to_owned(), 9, 10),
        ]
    );
    assert!(m.toasts.is_empty());
}

/// Fileira já coberta não se duplica; sem nada a fazer, o aviso diz porquê e a lista fica.
#[test]
fn fileira_ja_coberta_nao_se_duplica() {
    let mut m = Mesa::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("walk", 0, 3)).unwrap();
    com_a_folha(&mut m, lib);
    let botao = m.doc().layout.split;
    m.click(centre(botao));
    m.frame();
    assert_eq!(
        nomes(&m),
        [("walk".to_owned(), 0, 3), ("row 1".to_owned(), 9, 10)]
    );
    assert!(m.toasts.is_empty());
    pinta_a_folha(&mut m);
    let botao = m.doc().layout.split;
    m.click(centre(botao));
    m.frame();
    assert_eq!(nomes(&m).len(), 2, "a 2.ª vez não junta nada");
    assert!(!m.toasts.is_empty(), "e diz que não há o que fazer");
}

fn escolhe_o_quadro(m: &mut Mesa, i: usize) {
    let r = m.doc().layout.frames[i].0;
    m.click(centre(r));
    m.frame();
}

fn carrega_no_marcador(m: &mut Mesa) {
    let r = m.doc().layout.mark_toggle;
    assert!(r.w > 0.0, "o botão só existe com um quadro escolhido");
    m.click(centre(r));
    m.frame();
}

/// ⭐ Escolher o quadro, pôr o ◆, escrever o nome: o marcador grava-se no modelo de sempre e pinta-se no quadro; o
/// mesmo botão tira-o, e o último leva o componente.
#[test]
fn o_marcador_poe_nomeia_pinta_e_tira_no_quadro_escolhido() {
    let mut m = Mesa::new();
    assert_eq!(m.doc().layout.mark_toggle.w, 0.0, "sem quadro não há botão");
    escolhe_o_quadro(&mut m, 2);
    carrega_no_marcador(&mut m);
    assert_eq!(marcas(&m), [("idle".to_owned(), 2, String::new())]);
    assert_eq!(
        m.doc().editing,
        Some((doc::Field::Signal(2), String::new())),
        "o campo do nome abre-se"
    );
    assert_eq!(m.doc().painted_marks, [2], "o ◆ pinta-se nesse quadro");
    m.key(Key::Char('s'), Some("step"));
    m.key(Key::Enter, None);
    m.frame();
    assert_eq!(marcas(&m), [("idle".to_owned(), 2, "step".to_owned())]);
    assert_eq!(m.doc().marker_signal(2), "step");
    carrega_no_marcador(&mut m);
    assert!(marcas(&m).is_empty(), "o mesmo botão tira");
    assert!(m.doc().painted_marks.is_empty());
}

/// O nome reabre no campo e a edição fica só nesse marcador.
#[test]
fn clicar_no_campo_renomeia_o_sinal() {
    let mut m = Mesa::new();
    for (quadro, nome) in [(1, "a"), (3, "b")] {
        escolhe_o_quadro(&mut m, quadro);
        carrega_no_marcador(&mut m);
        m.key(Key::Char('n'), Some(nome));
        m.key(Key::Enter, None);
        m.frame();
    }
    escolhe_o_quadro(&mut m, 1);
    let campo = m.doc().layout.mark_signal;
    m.click(centre(campo));
    m.key(Key::Char('c'), Some("c"));
    m.key(Key::Enter, None);
    m.frame();
    assert_eq!(
        marcas(&m),
        [
            ("idle".to_owned(), 1, "ac".to_owned()),
            ("idle".to_owned(), 3, "b".to_owned())
        ]
    );
}

/// ⭐ O ◆ pertence ao desenho: reordenar leva-o junto; apagar o quadro apaga-o; apagar outro antes dele desloca-o.
/// *Mutação: tirar `markers_follow_move` do `release` ⇒ o ◆ fica no lugar e passa a marcar outro desenho.*
#[test]
fn o_marcador_anda_com_o_quadro() {
    let mut m = Mesa::new();
    escolhe_o_quadro(&mut m, 3);
    carrega_no_marcador(&mut m);
    m.key(Key::Enter, None);
    m.frame();
    let l = m.doc().layout.clone();
    let y = centre(l.frames[3].0).1;
    m.pointer(PointerKind::Down, centre(l.frames[3].0));
    m.drag_to((l.frames[0].0.x + 1.0, y));
    m.pointer(PointerKind::Up, (l.frames[0].0.x + 1.0, y));
    m.frame();
    assert_eq!(
        marcas(&m)[0].1,
        0,
        "o quadro foi para o começo, o ◆ com ele"
    );
    let fechar = m.doc().layout.frames[0].1;
    m.click(centre(fechar));
    m.frame();
    assert!(marcas(&m).is_empty(), "o quadro saiu, o ◆ com ele");

    escolhe_o_quadro(&mut m, 2);
    carrega_no_marcador(&mut m);
    m.key(Key::Enter, None);
    m.frame();
    let fechar = m.doc().layout.frames[0].1;
    m.click(centre(fechar));
    m.frame();
    assert_eq!(marcas(&m)[0].1, 1, "um quadro antes dele saiu: desce um");
}

/// O marcador que passou do fim não some: mostra-se no último quadro, diz-se mudo e sai pelo mesmo botão.
#[test]
fn um_marcador_passado_do_fim_mostra_se_no_ultimo_e_tira_se() {
    let mut m = Mesa::new();
    let e = Entity::from_bits(m.e);
    m.sim
        .world_mut()
        .entity_mut(e)
        .insert(SpriteAnimMarkers(vec![ph2d_ecs::FrameMarker {
            tag: "idle".into(),
            step: 40,
            signal: "x".into(),
        }]));
    m.frame();
    let ultimo = m.doc().authored_frames().len() - 1;
    assert_eq!(m.doc().painted_marks, [ultimo]);
    assert_eq!(m.doc().marker_past_end(ultimo), Some(40));
    escolhe_o_quadro(&mut m, ultimo);
    carrega_no_marcador(&mut m);
    assert!(marcas(&m).is_empty());
}

/// ⭐ Com a mão parada na borda a tira rola a cada pintura, e a largada cai no quadro que está sob a mão; no meio não
/// rola; na borda esquerda volta.
/// *Mutação: tirar `scroll_at_edge` de `lay_out` ⇒ o `strip_scroll` fica em zero.*
#[test]
fn a_tira_rola_com_a_mao_na_borda() {
    let mut m = Mesa::new();
    m.doc().pending.push(AnimFieldEdit::Frames(
        0,
        fr(&(0..32).chain(0..32).collect::<Vec<u32>>()),
    ));
    m.frame();
    let l = m.doc().layout.clone();
    let y = centre(l.frames[0].0).1;
    let meio = l.strip.x + l.strip.w * 0.5;
    let direita = l.strip.x + l.strip.w - 1.0;
    m.pointer(PointerKind::Down, centre(l.frames[0].0));
    m.drag_to((meio, y));
    m.paint();
    assert_eq!(m.doc().strip_scroll, 0.0, "no meio não rola");
    m.drag_to((direita, y));
    for _ in 0..200 {
        m.paint();
    }
    let rolou = m.doc().strip_scroll;
    assert!(rolou > 0.0, "a mão parada na borda rola a tira");
    let gap = m.doc().frame_drag().expect("o arrasto").gap;
    assert_eq!(
        gap,
        m.doc().drop_gap(direita),
        "o intervalo segue a rolagem"
    );
    m.drag_to((l.strip.x + 1.0, y));
    m.paint();
    assert!(m.doc().strip_scroll < rolou, "a borda esquerda volta");
    let solta = l.strip.x + l.strip.w - 1.0;
    m.drag_to((solta, y));
    m.paint();
    let gap = m.doc().frame_drag().expect("o arrasto").gap;
    m.pointer(PointerKind::Up, (solta, y));
    let to = doc::gap_to_index(0, gap);
    assert!(to > 0, "largou além do que se via sem rolar");
    let Some(AnimFieldEdit::Frames(_, f)) = m.pending().first().cloned() else {
        panic!("a largada grava os quadros");
    };
    assert_eq!(
        f[to],
        AnimFrameRef::cell(0),
        "o quadro pegado está onde a mão largou"
    );
}
