//! Provas de mutação da onda 3 do Input Map: o que sobreviveu à primeira corrida e o que o mata. Cada teste afirma a
//! lei pela porta pública, sem repetir a conta do código (a que o mutante deixava intacta).

use std::cell::RefCell;
use std::rc::Rc;

use super::Mesa;
use super::lista::onze;
use ph2d_a11y::NodeId;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::keys_fit::OUTLINE_REACH;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput, SurfaceTab,
};
use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_input::{Binding, GamepadButton, InputMap};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

fn pinta(view: View, sub: Sub, map: InputMap, a: Rect) -> Mesa {
    let mut m = Mesa::aberta(view);
    m.hero.input_map = map;
    m.frame();
    if view == View::Keyboard {
        m.click(m.at(KbHit::Sub(sub).id()));
    }
    m.paint_in(a);
    m
}

/// Os bytes RGBA de um token na tinta da lista, como o vello os põe nos dados de desenho.
fn bytes_de(t: ColorToken) -> u32 {
    let c = ph2d_editor_core::widget::card_ink::CardInk::of_list(Theme::default())
        .color(t)
        .to_rgba8();
    u32::from_le_bytes([c.r, c.g, c.b, c.a])
}

/// Quantas vezes a pintura da aba activa usou o token `t` (a aresta fina da plaquinha é um traço em `Border`).
fn usos_de(m: &mut Mesa, t: ColorToken) -> usize {
    let mut scene = VectorScene::new();
    let ts = &mut m.ts;
    if let Some((f, id)) = m.hero.documents.active_surface_mut() {
        f.paint(id, super::AREA, &mut scene, ts, &Theme::default());
    }
    let alvo = bytes_de(t);
    (scene.inner().encoding().draw_data.iter())
        .filter(|c| **c == alvo)
        .count()
}

fn sem_ligacoes() -> InputMap {
    let mut map = onze();
    let ids: Vec<_> = map.actions().iter().map(|a| a.id).collect();
    for id in ids {
        map.get_mut(id).expect("acção").bindings.clear();
    }
    map
}

/// ⭐ Onda 3 D2, no COMANDO: sem acção a plaquinha não se pinta. Num comando sem nenhuma ligação nenhuma parte tem
/// aresta de plaquinha; ligar UMA parte acrescenta exactamente a dela (um mutante que pintasse a de todas as partes
/// sem acção daria o mesmo número nos dois e o comando todo ladeado de arestas).
#[test]
fn o_comando_sem_accao_nao_pinta_plaquinha() {
    let a = Rect::new(308.0, 28.0, 988.0, 890.0); // LITERAL-PX-OK: a área central a 1600×1000
    let mut vazio = pinta(View::Keyboard, Sub::Gamepad, sem_ligacoes(), a);
    let n0 = usos_de(&mut vazio, ColorToken::Border);
    let mut map = sem_ligacoes();
    let id = map.actions()[0].id;
    map.get_mut(id)
        .expect("acção")
        .bindings
        .push(Binding::PadButton(GamepadButton::South));
    let mut um = pinta(View::Keyboard, Sub::Gamepad, map, a);
    let n1 = usos_de(&mut um, ColorToken::Border);
    assert_eq!(
        n1,
        n0 + 1,
        "só a parte ligada ganha plaquinha: {n0} -> {n1}"
    );
}

/// ⭐ Onda 3 R6: as barras de uma SETA (braço da cruz, seta da manche) começam por dentro do contorno da acção aberta
/// (`OUTLINE_REACH` da aresta), ao contrário do botão redondo: o contorno nunca as come, e isto vê-se nas barras
/// pintadas, não num pior caso calculado.
#[test]
fn as_barras_da_seta_comecam_por_dentro_do_contorno() {
    let a = Rect::new(308.0, 28.0, 988.0, 890.0); // LITERAL-PX-OK: a área central a 1600×1000
    let setas = [
        Binding::PadButton(GamepadButton::DPadUp),
        Binding::PadButton(GamepadButton::DPadLeft),
        Binding::PadAxis {
            axis: ph2d_input::GamepadAxis::LeftStickX,
            positive: false,
        },
    ];
    let mut map = onze();
    let id = map.actions()[0].id;
    for b in setas {
        map.get_mut(id).expect("acção").bindings.push(b);
    }
    let m = pinta(View::Keyboard, Sub::Gamepad, map, a);
    let doc = &m.tab().doc;
    for b in setas {
        let r = doc.rect_of(KbHit::Pad(b).id()).expect("a parte");
        let bars: Vec<Rect> = (doc.census.bars.iter())
            .filter(|(bar, _)| r.contains(bar.x + bar.w * 0.5, bar.y + bar.h * 0.5))
            .map(|b| b.0)
            .collect();
        assert!(!bars.is_empty(), "{b:?}: a seta tem barras");
        for bar in bars {
            assert!(
                bar.x >= r.x + OUTLINE_REACH - 0.01
                    && bar.x + bar.w <= r.x + r.w - OUTLINE_REACH + 0.01,
                "{b:?}: a barra {bar:?} entra no contorno de {r:?}"
            );
        }
    }
}

/// ⭐ Onda 3 A6: o deslize de lado não rola o teclado numa vista que não o tem, nem com o rasto da pintura anterior
/// (o campo do teclado ainda registado quando a vista muda e o deslize chega antes do quadro seguinte).
#[test]
fn na_lista_o_deslize_de_lado_ignora_o_campo_que_o_teclado_deixou() {
    let a = Rect::new(297.0, 28.0, 360.0, 604.0); // LITERAL-PX-OK: estreita, o teclado rola
    let mut m = pinta(View::Keyboard, Sub::Keyboard, onze(), a);
    let clip = m
        .tab()
        .doc
        .kb
        .hbar
        .expect("o teclado rola na estreita")
        .clip;
    m.tab_mut().doc.set_view(View::List);
    let dentro = (clip.x + clip.w * 0.5, clip.y + clip.h * 0.5);
    assert!(
        !m.tab_mut().wheel_sideways(0, dentro.0, dentro.1, -40.0), // LITERAL-PX-OK: um deslize
        "na lista o deslize de lado não é tomado"
    );
    assert_eq!(m.tab().doc.kb.hscroll, 0.0, "o teclado não rolou por baixo");
}

/// Uma família que PEDE o lado: conta o que a roda e o deslize lhe trazem, `(dy, shift)` e `dx`.
struct Lado(Rc<RefCell<(Vec<(f32, bool)>, Vec<f32>)>>);

impl DocumentSurface for Lado {
    fn tabs(&self) -> Vec<DocTab> {
        vec![DocTab {
            id: 1,
            title: String::new(),
            dirty: false,
        }]
    }
    fn paint(&mut self, _: u64, _: Rect, _: &mut VectorScene, _: &mut TextSystem, _: &Theme) {}
    fn pointer(
        &mut self,
        _: u64,
        _: PointerInput<'_>,
        _: PointerKind,
        _: PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        false
    }
    fn pointer_move(&mut self, _: u64, _: PointerInput<'_>, _: f32, _: f32) -> bool {
        false
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, dy: f32, shift: bool) -> bool {
        self.0.borrow_mut().0.push((dy, shift));
        true
    }
    fn wheel_sideways(&mut self, _: u64, _: f32, _: f32, dx: f32) -> bool {
        self.0.borrow_mut().1.push(dx);
        true
    }
    fn key(&mut self, _: u64, _: &mut TextSystem, _: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// ⭐ Onda 3 item 7: cada eixo vai UMA vez e só à porta dele — a roda (`dy`) à roda, o `dx` à `wheel_sideways`; a
/// família que pede o lado nunca é perguntada por um deslize de lado que não existe (`dx` = 0).
#[test]
fn a_familia_que_pede_o_lado_so_ouve_o_dx_quando_ha_dx() {
    let mut hero = HeroScreen::new(NodeId(1));
    let ouvido = Rc::new(RefCell::new((Vec::new(), Vec::new())));
    hero.documents
        .install_surface(Box::new(Lado(ouvido.clone())));
    hero.documents
        .activate_surface(SurfaceTab { family: 0, id: 1 });
    let passo = |hero: &mut HeroScreen, s| {
        scroll(hero, 500.0, 300.0, s); // LITERAL-PX-OK: um ponto qualquer da área
        std::mem::take(&mut *ouvido.borrow_mut())
    };
    let (rodas, lados) = passo(&mut hero, BoardScroll::Pan([0.0, -20.0]));
    assert_eq!(
        (rodas, lados),
        (vec![(-20.0, false)], vec![]),
        "vertical puro"
    );
    let (rodas, lados) = passo(&mut hero, BoardScroll::Wheel(-16.0));
    assert_eq!(
        (rodas, lados),
        (vec![(-16.0, false)], vec![]),
        "a roda do rato"
    );
    let (rodas, lados) = passo(&mut hero, BoardScroll::Pan([30.0, -20.0]));
    assert_eq!(
        (rodas, lados),
        (vec![(-20.0, false)], vec![30.0]),
        "diagonal: cada eixo uma vez"
    );
    let (rodas, lados) = passo(&mut hero, BoardScroll::Pan([30.0, 0.0]));
    assert_eq!(
        (rodas, lados),
        (vec![(0.0, false)], vec![30.0]),
        "de lado puro"
    );
}

/// ⭐ Onda 3 item 1: a legenda de TEXTO cabe também em ALTURA (a fila F em meia altura) — em toda largura de aba, nos
/// dois modos de texto, o texto pintado de cada tecla (a de duas linhas e a pontuação têm a sua conta) mais a folga da
/// plaquinha (2 × `SEG_PAD`) cabe na tecla. Sem a cláusula de altura em `choose`, um degrau mais alto do que a tecla
/// aguenta é escolhido assim que a largura o deixa.
#[test]
fn a_legenda_de_texto_cabe_em_altura_em_toda_a_largura() {
    let tight = ph2d_tokens::LineHeight::Tight.ratio();
    super::em_cada_modo_de_texto(|| {
        let mut w = 360.0; // LITERAL-PX-OK: da estreita (rola) à larga
        while w <= 1700.0 {
            for view in [View::Keyboard, View::TryIt] {
                let a = Rect::new(297.0, 28.0, w, 604.0);
                let m = pinta(view, Sub::Keyboard, onze(), a);
                for l in &m.tab().doc.census.keys {
                    let Some((s, px)) = &l.text else { continue };
                    if l.key.cap_lines().is_some()
                        || ph2d_app_input_map::view_keyboard::keys_fit::is_punct(s)
                    {
                        continue;
                    }
                    assert!(
                        px * tight + 4.0 <= l.rect.h + 0.01, // LITERAL-PX-OK: 2 × SEG_PAD
                        "{view:?} {w}: «{s}» a {px} px não cabe em {} de altura",
                        l.rect.h
                    );
                }
            }
            w += 50.0; // LITERAL-PX-OK: passo da varredura
        }
    });
}
