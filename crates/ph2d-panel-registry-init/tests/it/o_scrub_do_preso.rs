//! ⭐ **O scrub num cartão PRESO** (`docs/CartoesUI/10`, smoke 3): a pega de um campo, por gesto real.
//! Entre um Move e o seguinte correm quadros inteiros: a seleção publica o número DELA por cima da
//! caixa, e o gesto do preso tem de sobreviver. Censo: todo campo de todo cartão preso.

use super::*;
use ph2d_editor_core::interaction::dispatch_key;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_editor_core::zones::Rect;

/// Quem corre os quadros de um cartão preso, na ordem da app.
pub(in super::super) trait MesaDoPreso {
    fn hero(&mut self) -> &mut HeroScreen;
    fn quadros(&mut self, n: usize);
}

impl MesaDoPreso for Mesa {
    fn hero(&mut self) -> &mut HeroScreen {
        &mut self.h
    }
    fn quadros(&mut self, n: usize) {
        Mesa::quadros(self, n);
    }
}

/// As pegas à vista no solto `k`: `(id, centro da pega)`.
fn pegas(h: &HeroScreen, k: NodeId) -> Vec<(NodeId, (f32, f32))> {
    let s = h.store.panel_rect(loose_rect_of(k)).expect("o solto");
    let mut achados: Vec<(NodeId, Rect)> = Vec::new();
    let mut y = s.y;
    while y < s.y + s.h {
        let mut x = s.x;
        while x < s.x + s.w {
            if let Some((id, r)) = h.hit_index.hit_with_rect(x, y)
                && h.store.card_field(id).is_some()
                && h.store.tool_row_slot(id) != Some(r)
                && !achados.iter().any(|(i, _)| *i == id)
            {
                achados.push((id, r));
            }
            x += 3.0;
        }
        y += 3.0;
    }
    achados
        .into_iter()
        .map(|(id, r)| (id, centro(handle_rect(r))))
        .filter(|(_, c)| s.contains(c.0, c.1))
        .collect()
}

fn valor_de(h: &HeroScreen, id: NodeId) -> f64 {
    h.store.number_value(id).unwrap_or(f64::NAN)
}

fn esc(h: &mut HeroScreen) {
    let arena = bumpalo::Bump::new();
    let ev = ph2d_host::KeyEvent {
        keycode: 0x1B,
        modifiers: ph2d_host::Modifiers::default(),
        kind: ph2d_host::KeyKind::Down,
        timestamp_ns: 0,
    };
    for e in dispatch_key(&mut h.store, ev, &arena).to_vec() {
        h.apply_event(e);
    }
}

/// O que `passos` Moves fazem a `id`: devolve os problemas medidos.
fn um_campo<M: MesaDoPreso>(
    m: &mut M,
    k: NodeId,
    id: NodeId,
    p: (f32, f32),
    nome: &str,
) -> Vec<String> {
    let mut erros = Vec::new();
    let _ = m.hero().store.record_pointer_down(Some(NodeId(1)), 0);
    mao(m.hero(), PointerKind::Down, p);
    let partida = valor_de(m.hero(), id);
    if m.hero().store.number_input_drag().map(|d| d.id) != Some(id) {
        return vec![format!("{nome} {id:?}: a pega não agarrou")];
    }
    m.quadros(1);
    // Para o lado que tem curso.
    let sobe = m
        .hero()
        .store
        .number_scrub_interval(id)
        .is_none_or(|(_, hi)| partida < hi);
    let sinal = if sobe { 1.0 } else { -1.0 };
    let mut x = p.0;
    let mut ultimo = partida;
    for i in 0..8 {
        x += sinal * 3.0;
        mao(m.hero(), PointerKind::Move, (x, p.1));
        let no_move = valor_de(m.hero(), id);
        m.quadros(1);
        let no_quadro = valor_de(m.hero(), id);
        if (no_move - no_quadro).abs() > 1e-9 {
            erros.push(format!("{nome} {id:?}: passo {i}: o quadro levou a caixa de {no_move} a {no_quadro} (partida {partida})"));
            break;
        }
        if i > 1 && (no_quadro - ultimo) * f64::from(sinal) < 0.0 {
            erros.push(format!(
                "{nome} {id:?}: passo {i}: andou para trás, {ultimo} → {no_quadro}"
            ));
            break;
        }
        ultimo = no_quadro;
    }
    if erros.is_empty() && ultimo == partida {
        erros.push(format!("{nome} {id:?}: o número não mexeu ({partida})"));
    }
    esc(m.hero());
    let reposto = valor_de(m.hero(), id);
    if m.hero().store.number_input_drag().is_some() {
        erros.push(format!("{nome} {id:?}: o Esc não largou"));
    }
    if (reposto - partida).abs() > 1e-9 {
        erros.push(format!(
            "{nome} {id:?}: o Esc deu {reposto}, partida {partida}"
        ));
    }
    mao(m.hero(), PointerKind::Up, (x, p.1));
    m.quadros(2);
    let _ = k;
    erros
}

/// O censo de um cartão preso: cada pega anda e o quadro não a devolve. Devolve quantas mediu.
pub(in super::super) fn censo<M: MesaDoPreso>(
    m: &mut M,
    k: NodeId,
    nome: &str,
) -> (usize, Vec<String>) {
    let achadas = pegas(m.hero(), k);
    let mut erros = Vec::new();
    for (id, p) in &achadas {
        erros.extend(um_campo(m, k, *id, *p, nome));
    }
    (achadas.len(), erros)
}

impl Mesa {
    fn largura_px(&self, i: usize) -> f64 {
        DocPx::of(&self.h.project).of_world(self.forma(i).base_stroke().expect("o traço").width)
    }

    fn pega(&self, k: NodeId, id: NodeId) -> (f32, f32) {
        pegas(&self.h, k)
            .into_iter()
            .find(|(i, _)| *i == id)
            .expect("a pega")
            .1
    }

    /// Agarra a pega e anda `n` Moves de `dx`, com quadros entre eles. Devolve o x onde parou.
    fn anda(&mut self, p: (f32, f32), dx: f32, n: usize) -> f32 {
        let _ = self.h.store.record_pointer_down(Some(NodeId(1)), 0);
        mao(&mut self.h, PointerKind::Down, p);
        self.quadro();
        let mut x = p.0;
        for _ in 0..n {
            x += dx;
            mao(&mut self.h, PointerKind::Move, (x, p.1));
            self.quadro();
        }
        x
    }
}

/// ⭐ O defeito do dono: o Weight do Stroke solto preso a A, com outra forma escolhida. Medido antes
/// da cura: A = 3, B = 1: o número ficava entre 1,8 e 2,6 e, ao largar, A tomava o 1 de B.
/// *Mutação: tirar a guarda de `set_number_value` ⇒ vermelho.*
#[test]
fn scrubbing_the_weight_of_a_pinned_stroke_moves_its_object_only() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_STROKE, 0, sel);
        let (a0, b0, c0) = (m.a(), m.forma(1), m.forma(2));
        let w0 = m.largura_px(0);
        let t0 = m.tool().stroke_width_px;
        let p = m.pega(k, v::VECTOR_WIDTH_NUM);
        let x = m.anda(p, 2.0, 10);
        let meio =
            m.h.store
                .number_value(v::VECTOR_WIDTH_NUM)
                .expect("a caixa");
        assert!(
            meio > w0 + 1.0,
            "{sel:?}: o número ficou em {meio}, partiu de {w0}"
        );
        assert!(
            (m.largura_px(0) - meio).abs() < 1e-3,
            "{sel:?}: o traço não seguiu: {} × {meio}",
            m.largura_px(0)
        );
        mao(&mut m.h, PointerKind::Up, (x, p.1));
        m.quadros(3);
        assert!(
            (m.largura_px(0) - meio).abs() < 1e-3,
            "{sel:?}: ao largar, A ficou com {}",
            m.largura_px(0)
        );
        assert_eq!(m.forma(1), b0, "{sel:?}: B recebeu");
        assert_eq!(m.forma(2), c0, "{sel:?}: C recebeu");
        if sel[0] != 0 {
            assert_eq!(
                m.tool().stroke_width_px,
                t0,
                "{sel:?}: o gesto ficou na tool"
            );
            assert_eq!(
                m.h.store.number_value(v::VECTOR_WIDTH_NUM),
                Some(t0),
                "{sel:?}: a caixa da seleção não voltou"
            );
        }
        let preso =
            m.h.store
                .pinned_states(k)
                .and_then(|e| e.estado(v::VECTOR_WIDTH_NUM));
        assert!(
            matches!(preso, Some(ph2d_editor_core::interaction::InteractiveState::NumberInput { value, .. }) if (*value - meio).abs() < 1e-3),
            "{sel:?}: o preso mostra {preso:?}, A tem {meio}"
        );
        // O Esc a meio repõe A; nada fica em B nem em C. *Mutação: `note_card_key` sem o scrub ⇒ a
        // reversão vai à seleção.*
        let x = m.anda(p, 2.0, 6);
        esc(&mut m.h);
        m.quadros(2);
        assert!(
            (m.largura_px(0) - meio).abs() < 1e-3,
            "{sel:?}: o Esc deixou {}",
            m.largura_px(0)
        );
        mao(&mut m.h, PointerKind::Up, (x, p.1));
        m.quadros(2);
        assert_ne!(m.a(), a0);
        assert_eq!(m.forma(1), b0);
    }
}

/// A pega da opacidade de Fill, de Stroke e de uma camada: ao largar, A fica com o número da mão, não
/// com a barra que a seleção semeou (o painel só ouve a barra). *Mutação: tirar o espelho do Up ⇒ vermelho.*
#[test]
fn scrubbing_an_opacity_of_a_pinned_card_keeps_the_number_on_release() {
    let mut m = Mesa::nova();
    let f = m.prende(CARD_FILL, 0, &[1]);
    let s = m.prende(CARD_STROKE, 0, &[1]);
    let casos: [(NodeId, NodeId, fn(&VecPath) -> f64); 3] = [
        (f, v::VECTOR_FILL_OPACITY_NUM, |p| {
            f64::from(cor_de_fill(p).a) * 100.0 / 255.0
        }),
        (s, v::VECTOR_STROKE_OPACITY_NUM, |p| {
            f64::from(p.base_stroke().expect("o traço").color().a) * 100.0 / 255.0
        }),
        (f, v::vector_paint_row_opacity_id(0), |p| {
            f64::from(p.paints[0].opacity.get()) * 100.0
        }),
    ];
    let b0 = m.forma(1);
    for (k, id, le) in casos {
        let antes = le(&m.a());
        let p = m.pega(k, id);
        // Para baixo: A começa cheia ou a meio, e nunca no piso.
        let x = m.anda(p, -2.0, 8);
        let meio = m.h.store.number_value(id).expect("a caixa");
        assert!(meio < antes - 1.0, "{id:?}: {antes} → {meio}");
        mao(&mut m.h, PointerKind::Up, (x, p.1));
        m.quadros(3);
        assert!(
            (le(&m.a()) - meio).abs() < 1.0,
            "{id:?}: A ficou com {}, a mão largou {meio}",
            le(&m.a())
        );
        assert_eq!(m.forma(1), b0, "{id:?}: B recebeu");
    }
}

/// A régua: o mesmo gesto no cartão não preso anda igual, e na forma escolhida.
#[test]
fn the_same_scrub_on_an_unpinned_card_moves_the_same_distance() {
    let anda = |pino: bool| {
        let mut m = Mesa::nova();
        let k = if pino {
            m.prende(CARD_STROKE, 0, &[1])
        } else {
            solta_com_recuo(&mut m.h, CARD_STROKE, 0.0);
            m.quadros(5);
            CARD_STROKE
        };
        let p = m.pega(k, v::VECTOR_WIDTH_NUM);
        let w0 = m.largura_px(0);
        let x = m.anda(p, 2.0, 8);
        mao(&mut m.h, PointerKind::Up, (x, p.1));
        m.quadros(3);
        m.largura_px(0) - w0
    };
    let (preso, solto) = (anda(true), anda(false));
    assert!(solto > 1.0, "o não preso andou {solto}");
    assert!(
        (preso - solto).abs() < 1e-3,
        "preso {preso} × não preso {solto}"
    );
}

/// O clique na ponta direita do campo não dá um passo: o cartão não pinta setas.
#[test]
fn a_click_on_the_right_end_of_a_card_field_does_not_step() {
    let mut m = Mesa::nova();
    let k = m.prende(CARD_STROKE, 0, &[1]);
    let w0 = m.largura_px(0);
    let c = m.no_solto(k, v::VECTOR_WIDTH_NUM);
    let (_, r) = m.h.hit_index.hit_with_rect(c.0, c.1).expect("o campo");
    for y in [r.y + 2.0, r.y + r.h - 2.0] {
        let _ = m.h.store.record_pointer_down(Some(NodeId(1)), 0);
        mao(&mut m.h, PointerKind::Down, (r.x + r.w - 2.0, y));
        mao(&mut m.h, PointerKind::Up, (r.x + r.w - 2.0, y));
        m.quadros(2);
        assert_eq!(m.largura_px(0), w0, "o clique mudou A");
    }
}

/// Uma tecla (o Shift) a meio do scrub continua a ser do preso: os Moves seguintes não vão à seleção.
#[test]
fn a_key_in_the_middle_of_a_pinned_scrub_keeps_it_on_its_object() {
    let mut m = Mesa::nova();
    let k = m.prende(CARD_STROKE, 0, &[1]);
    let b0 = m.forma(1);
    let p = m.pega(k, v::VECTOR_WIDTH_NUM);
    let x = m.anda(p, 2.0, 4);
    let arena = bumpalo::Bump::new();
    let shift = ph2d_host::KeyEvent {
        keycode: 0x10,
        modifiers: ph2d_host::Modifiers {
            shift: true,
            ..Default::default()
        },
        kind: ph2d_host::KeyKind::Down,
        timestamp_ns: 0,
    };
    let _ = dispatch_key(&mut m.h.store, shift, &arena);
    assert_eq!(m.h.store.card_press_subject(), Some(m.bits[0]));
    mao(&mut m.h, PointerKind::Move, (x + 4.0, p.1));
    m.quadro();
    let n =
        m.h.store
            .number_value(v::VECTOR_WIDTH_NUM)
            .expect("a caixa");
    assert!(
        (m.largura_px(0) - n).abs() < 1e-3,
        "A = {}, caixa = {n}",
        m.largura_px(0)
    );
    assert_eq!(m.forma(1), b0, "B recebeu");
    mao(&mut m.h, PointerKind::Up, (x + 4.0, p.1));
}

/// O Shift é ×10 e o Alt ×0,1, como no cartão da lista (e o preso não os perde).
#[test]
fn shift_and_alt_on_a_pinned_scrub_change_the_step() {
    let mut anda = |shift: bool, alt: bool| {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_STROKE, 0, &[1]);
        m.h.store.set_shift_held(shift);
        m.h.store.set_alt_held(alt);
        let p = m.pega(k, v::VECTOR_WIDTH_NUM);
        let w0 = m.largura_px(0);
        let x = m.anda(p, 1.0, 9);
        let d =
            m.h.store
                .number_value(v::VECTOR_WIDTH_NUM)
                .expect("a caixa")
                - w0;
        mao(&mut m.h, PointerKind::Up, (x, p.1));
        d
    };
    let (normal, shift, alt) = (anda(false, false), anda(true, false), anda(false, true));
    assert!(normal > 0.0);
    assert!(
        (shift / normal - 10.0).abs() < 0.01,
        "Shift: {shift} × {normal}"
    );
    assert!((alt / normal - 0.1).abs() < 0.01, "Alt: {alt} × {normal}");
}

/// ⭐ O censo: toda pega do Fill e do Stroke presos anda e o quadro não a devolve.
#[test]
fn every_scrub_handle_of_the_pinned_fill_and_stroke_survives_the_frames() {
    let mut m = Mesa::nova();
    let f = m.prende(CARD_FILL, 0, &[1]);
    let s = m.prende(CARD_STROKE, 0, &[1]);
    let (nf, mut erros) = censo(&mut m, f, "Fill");
    let (ns, e) = censo(&mut m, s, "Stroke");
    erros.extend(e);
    eprintln!("CENSO Fill {nf} Stroke {ns}");
    assert!(nf >= 1 && ns >= 2, "Fill {nf}, Stroke {ns}");
    assert!(erros.is_empty(), "{erros:#?}");
}

/// As setas no campo em foco do preso andam do número de A, não do da seleção (o quadro não o troca).
#[test]
fn the_arrows_on_a_focused_pinned_field_step_from_its_object() {
    let mut m = Mesa::nova();
    let k = m.prende(CARD_STROKE, 0, &[1]);
    let w0 = m.largura_px(0);
    let c = m.no_solto(k, v::VECTOR_WIDTH_NUM);
    mao(&mut m.h, PointerKind::Down, c);
    mao(&mut m.h, PointerKind::Up, c);
    assert_eq!(m.h.store.focus_id(), Some(v::VECTOR_WIDTH_NUM));
    m.quadros(3);
    assert_eq!(
        m.h.store.number_value(v::VECTOR_WIDTH_NUM),
        Some(w0),
        "o quadro trocou o número em foco"
    );
    let arena = bumpalo::Bump::new();
    let seta = ph2d_host::KeyEvent {
        keycode: 0xF700,
        modifiers: ph2d_host::Modifiers::default(),
        kind: ph2d_host::KeyKind::Down,
        timestamp_ns: 0,
    };
    // Duas vezes: a 1.ª escrita devolve a tool à seleção, e a 2.ª partia do número dela.
    for _ in 0..2 {
        for e in dispatch_key(&mut m.h.store, seta, &arena).to_vec() {
            m.h.apply_event(e);
        }
        m.quadros(2);
    }
    assert!(
        (m.largura_px(0) - (w0 + 2.0)).abs() < 1e-3,
        "A ficou com {}, partiu de {w0}",
        m.largura_px(0)
    );
    assert_eq!(
        m.forma(1).base_stroke().map(|s| s.width),
        Some(0.01),
        "B recebeu"
    );
}

/// A dica do campo nasce ao lado do campo sob a mão, mesmo com o mesmo id noutra cópia ou na barra.
/// *Mutação: `alvo_da_dica` só com `rect_for` ⇒ a dica de uma das cópias aparece longe dela.*
#[test]
fn the_field_hint_is_anchored_to_the_copy_under_the_pointer() {
    let mut m = Mesa::nova();
    let a = m.prende(CARD_STROKE, 0, &[2]);
    let b = m.prende(CARD_STROKE, 1, &[2]);
    for k in [a, b] {
        let p = m.pega(k, v::VECTOR_WIDTH_NUM);
        mao(&mut m.h, PointerKind::Move, p);
        m.quadro();
        let r = ph2d_editor_core::screens::hero::topbar::alvo_da_dica(
            &m.h.hit_index,
            &m.h.store,
            v::VECTOR_WIDTH_NUM,
        )
        .expect("o alvo");
        assert!(
            r.contains(p.0, p.1),
            "{k:?}: a dica em {r:?}, a mão em {p:?}"
        );
    }
}
