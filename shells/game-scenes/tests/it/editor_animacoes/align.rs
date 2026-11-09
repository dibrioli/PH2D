//! ⭐ **A prévia Align** (GEF 28, onda B): o botão troca a folha pela prévia; o arrasto move a folha, o canto muda-lhe o
//! tamanho, Alt + arrasto e as setas acertam só o quadro aberto; o clique direito num quadro da tira fixa a referência.
//! Cada gesto é conferido na edição que a aba emite e, depois do `frame`, no mundo.

use super::{Mesa, centre};
use ph2d_ecs::{CellOffsets, Entity, ExtraSheet, FrameRef, SpriteAnimFrames, SpriteSheets};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput, PointerInput};
use ph2d_editor_core::{AnimFrameRef, ImageUseEdit};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;

/// O herói tem células de 192 px num quad de 1 m: um pixel da imagem são `1/192` m.
const MPP: f32 = 1.0 / 192.0; // LITERAL-PX-OK: a fixtura (1536 / 8 px por 1 m)

fn sprite(m: &Mesa) -> ph2d_render::Sprite {
    *m.sim
        .world()
        .get::<ph2d_render::Sprite>(Entity::from_bits(m.e))
        .expect("o herói")
}

fn offsets(m: &Mesa) -> Option<CellOffsets> {
    m.sim
        .world()
        .get::<CellOffsets>(Entity::from_bits(m.e))
        .cloned()
}

impl Mesa {
    /// Liga a prévia pelo botão «Align» e pinta-a.
    fn align_on(&mut self) {
        let t = self.doc().layout.align.toggle;
        assert!(t.w > 0.0, "o botão Align no título da folha");
        self.click(centre(t));
        self.frame();
        assert!(self.doc().align.on);
    }

    fn pointer_with(
        &mut self,
        kind: PointerKind,
        button: PointerButton,
        at: (f32, f32),
        mods: Modifiers,
    ) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let input = PointerInput {
            text: &mut ts,
            mods,
            now_ns: 0,
        };
        self.docs().pointer(e, input, kind, button, at.0, at.1);
        self.ts = ts;
    }

    /// Um arrasto inteiro de `a` a `b` com `mods`, em passos (o último é o Up).
    fn drag(&mut self, a: (f32, f32), b: (f32, f32), mods: Modifiers) {
        let p = PointerButton::Primary;
        self.pointer_with(PointerKind::Down, p, a, mods);
        for i in 1..=4 {
            let t = i as f32 / 4.0; // LITERAL-PX-OK: passos do arrasto
            let at = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            self.pointer_with(PointerKind::Move, p, at, mods);
        }
        self.pointer_with(PointerKind::Up, p, b, mods);
    }

    fn arrow(&mut self, key: Key, shift: bool) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let mods = Modifiers {
            shift,
            ..Modifiers::default()
        };
        self.docs().key(
            e,
            &mut ts,
            KeyInput {
                key,
                mods,
                pressed: true,
                text: None,
                paste: None,
            },
        );
        self.ts = ts;
    }

    fn sheet_edits(&mut self) -> Vec<ImageUseEdit> {
        self.doc().sheet_ui.pending.clone()
    }
}

/// O botão troca a folha pela prévia: a folha deixa de aceitar cliques, e a prévia pinta o quadro aberto, a
/// referência, a cruz do Pivot, a linha do chão e a pega do canto. Desligar volta à folha.
/// *Mutação: a prévia sem a cruz (`painted.pivot = None`) ou a folha a continuar clicável ⇒ vermelho.*
#[test]
fn o_botao_align_troca_a_folha_pela_previa() {
    let mut m = Mesa::new();
    assert!(!m.doc().align.on);
    m.align_on();
    let d = m.doc();
    assert_eq!(d.layout.sheet.w, 0.0, "a folha cede o lugar");
    assert_eq!(
        d.cell_at(
            d.layout.align.preview.x + 4.0,
            d.layout.align.preview.y + 40.0
        ),
        None
    );
    let p = d.align.painted;
    let cur = p.current.expect("o quadro aberto pintado");
    assert!(p.reference.is_some() && p.previous.is_some(), "{p:?}");
    let (px, py) = p.pivot.expect("a cruz do Pivot");
    assert!(
        (px - (cur.x + cur.w / 2.0)).abs() < 1.0 && (py - (cur.y + cur.h / 2.0)).abs() < 1.0,
        "o herói tem o Pivot no centro do quad"
    );
    let floor = p.floor.expect("a linha do chão");
    assert!(
        floor > py && floor <= cur.y + cur.h + 1.0,
        "os pés ficam abaixo do Pivot, dentro do quad"
    );
    assert!(p.handle.is_some());
    let caixa = p
        .reference_box
        .expect("o contorno do desenho da referência");
    assert!(
        (caixa.y + caixa.h - floor).abs() < 1.0,
        "a linha do chão é a base do desenho da referência"
    );
    let t = d.layout.align.toggle;
    m.click(centre(t));
    m.frame();
    assert!(
        !m.doc().align.on && m.doc().layout.sheet.w > 0.0,
        "desligado, a folha volta"
    );
}

/// Arrastar na prévia move a folha inteira em pixels inteiros da imagem; o mundo recebe o Pivot andado.
/// *Mutação: o sinal do Y trocado em `px` (o ecrã desce, o local sobe) ⇒ o `anchor[1]` sai com o sinal errado.*
#[test]
fn arrastar_na_previa_move_a_folha_em_pixels_inteiros() {
    let mut m = Mesa::new();
    m.align_on();
    let v = m.doc().align.view.expect("a vista");
    let p = m.doc().layout.align.preview;
    let a = (p.x + p.w / 4.0, p.y + p.h / 4.0);
    // 12 px da imagem para a direita e 5 para CIMA, no ecrã.
    let b = (a.0 + 12.0 * MPP * v.k, a.1 - 5.0 * MPP * v.k);
    let antes = sprite(&m).anchor;
    m.drag(a, b, Modifiers::default());
    assert!(
        m.sheet_edits()
            .iter()
            .all(|e| matches!(e, ImageUseEdit::MoveSheet(_)))
    );
    m.frame();
    let depois = sprite(&m).anchor;
    let anda = [(depois[0] - antes[0]) / MPP, (depois[1] - antes[1]) / MPP];
    assert!(
        (anda[0] - 12.0).abs() < 1e-3 && (anda[1] - 5.0).abs() < 1e-3,
        "{anda:?}"
    );
}

/// O canto muda o tamanho da folha em volta do Pivot: puxar para o dobro da distância dobra o quad.
/// *Mutação: `ScaleSheet(total)` em vez do passo `total / applied` ⇒ a escala compõe-se e passa do dobro.*
#[test]
fn o_canto_da_previa_muda_o_tamanho_da_folha() {
    let mut m = Mesa::new();
    m.align_on();
    let p = m.doc().align.painted;
    let h = centre(p.handle.expect("a pega"));
    let (ox, oy) = p.pivot.unwrap();
    let longe = (ox + (h.0 - ox) * 1.5, oy + (h.1 - oy) * 1.5);
    m.drag(h, longe, Modifiers::default());
    let n = m.sheet_edits().len();
    assert!(n >= 2, "um passo por movimento ({n})");
    m.frame();
    let s = sprite(&m).size;
    assert!(
        (s[0] - 1.5).abs() < 1e-3 && (s[1] - 1.5).abs() < 1e-3,
        "{s:?}"
    );
}

/// Alt + arrasto acerta só o quadro aberto; as setas andam 1 px e, com Shift, 10. A folha não se move.
/// *Mutação: o Shift ignorado (`n = 1`) ⇒ o acerto final sai 3 px em vez de 12.*
#[test]
fn alt_arrasto_e_setas_acertam_so_o_quadro_aberto() {
    let mut m = Mesa::new();
    m.align_on();
    let (_, f) = m.doc().align_current().expect("o quadro aberto");
    let v = m.doc().align.view.unwrap();
    let p = m.doc().layout.align.preview;
    let a = (p.x + p.w / 4.0, p.y + p.h / 4.0);
    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    m.drag(a, (a.0 + 2.0 * MPP * v.k, a.1), alt);
    m.arrow(Key::Right, false);
    m.arrow(Key::Right, true);
    m.arrow(Key::Down, false);
    let antes = sprite(&m).anchor;
    m.frame();
    assert_eq!(sprite(&m).anchor, antes, "a folha fica");
    let o = offsets(&m).expect("o acerto do quadro");
    let d = o.of([8, 4], f.cell);
    assert!(
        (d[0] / MPP - 13.0).abs() < 1e-3 && (d[1] / MPP + 1.0).abs() < 1e-3,
        "{:?}",
        [d[0] / MPP, d[1] / MPP]
    );
    assert_eq!(o.cells.len(), 1, "só o quadro aberto");
}

/// Sem a prévia, as setas não são dela (a cena fica com elas).
#[test]
fn sem_a_previa_as_setas_nao_acertam_nada() {
    let mut m = Mesa::new();
    m.arrow(Key::Left, false);
    assert!(m.sheet_edits().is_empty());
}

/// O clique direito num quadro da tira abre «Use as reference»; escolhê-lo fixa esse quadro como referência.
/// *Mutação: o menu a fixar o quadro aberto em vez do clicado ⇒ a referência sai a célula 0.*
#[test]
fn o_clique_direito_num_quadro_fixa_a_referencia() {
    let mut m = Mesa::new();
    m.align_on();
    assert_eq!(
        m.doc().align_reference(),
        Some(AnimFrameRef::cell(0)),
        "a 1.ª da animação de entrada"
    );
    let q = centre(m.doc().layout.frames[3].0);
    m.pointer_with(
        PointerKind::Down,
        PointerButton::Secondary,
        q,
        Modifiers::default(),
    );
    m.paint();
    let item = m.doc().align.painted.menu_item.expect("o menu aberto");
    m.click(centre(item));
    m.frame();
    assert_eq!(m.doc().align_reference(), Some(AnimFrameRef::cell(3)));
    assert!(m.doc().align.menu.is_none());
}

/// O quadro aberto vem de OUTRA folha: o gesto põe-na visível primeiro, e o verbo age nela (não na folha 0).
/// *Mutação: sem o `ShowSheet` ⇒ o `MoveSheet` anda o Pivot da folha 0.*
#[test]
fn o_gesto_age_na_folha_do_quadro_aberto() {
    let mut m = Mesa::new();
    let run = m.db.insert_image_rgba8(64, 32, vec![255; 64 * 32 * 4]);
    let e = Entity::from_bits(m.e);
    let extra = ExtraSheet {
        asset: run,
        size: [1.0, 1.0],
        anchor: [0.0, 0.0],
        hframes: 2,
        vframes: 1,
        offsets: Default::default(),
    };
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "idle",
        vec![
            FrameRef::Cell { sheet: 1, cell: 0 },
            FrameRef::Cell { sheet: 1, cell: 1 },
        ],
    );
    m.sim
        .world_mut()
        .entity_mut(e)
        .insert((SpriteSheets(std::iter::once(extra).collect()), frames));
    m.frame();
    m.align_on();
    let (_, f) = m.doc().align_current().unwrap();
    assert_eq!(f.sheet, 1);
    let antes0 = sprite(&m).anchor;
    let p = m.doc().layout.align.preview;
    let v = m.doc().align.view.unwrap();
    let a = (p.x + p.w / 4.0, p.y + p.h / 4.0);
    // A folha 1 tem células de 32 px num quad de 1 m.
    m.drag(a, (a.0 + 4.0 / 32.0 * v.k, a.1), Modifiers::default());
    assert_eq!(m.sheet_edits().first(), Some(&ImageUseEdit::ShowSheet(1)));
    m.frame();
    assert_eq!(sprite(&m).anchor, antes0, "a folha 0 fica");
    let s = m.sim.world().get::<SpriteSheets>(e).unwrap().0[0].anchor;
    assert!((s[0] * 32.0 - 4.0).abs() < 1e-3, "{s:?}");
}
