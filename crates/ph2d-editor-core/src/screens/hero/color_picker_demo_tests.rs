//! ⭐⭐ O seletor de cor AO LADO do Inspector, onde o artista o largou, e o PINO (dono 24/10).
//! Tudo pelo quadro real (`paint_hero_screen`) e pelo despacho real (`dispatch_pointer`).

use super::*;
use crate::interaction::{dispatch_pointer, restore_section_prefs_text, section_prefs_text};
use crate::screens::hero::{HeroScreen, paint_hero_screen};
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

/// O alvo de referência dos tokens (iPad Pro 12,9").
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};
const ALVO: NodeId = NodeId(0x0C0_1012);

fn pinta(hero: &mut HeroScreen) -> HeroLayout {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(hero, JANELA, &mut scene, &mut text);
    crate::screens::hero::frame_layout::frame_layout(hero, JANELA, &mut text)
}

fn aberto() -> HeroScreen {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.set_picker_target(Some(ALVO));
    hero
}

fn rect_publicado(hero: &HeroScreen) -> Rect {
    hero.store
        .panel_rect(ids::INSP_BLENDER_PICKER)
        .expect("o seletor aberto publica o rect")
}

fn ev(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn manda(hero: &mut HeroScreen, kind: PointerKind, x: f32, y: f32) {
    let arena = Bump::new();
    let _ = dispatch_pointer(&mut hero.store, &hero.hit_index, ev(kind, x, y), &arena);
}

fn clica(hero: &mut HeroScreen, x: f32, y: f32) {
    manda(hero, PointerKind::Down, x, y);
    manda(hero, PointerKind::Up, x, y);
}

fn meio(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn cruza(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn dentro(a: Rect, janela: Rect) -> bool {
    a.x >= janela.x
        && a.y >= janela.y
        && a.x + a.w <= janela.x + janela.w
        && a.y + a.h <= janela.y + janela.h
}

/// A lei pura, nos dois lados: coluna à direita ⇒ à esquerda dela; sob espelho ⇒ à direita.
#[test]
fn o_encaixe_fica_ao_lado_da_coluna_nos_dois_lados() {
    let direita = Rect::new(1066.0, 40.0, 300.0, 984.0);
    let (x, y) = picker_anchor_beside(JANELA, direita, PICKER_W, PICKER_H);
    let r = Rect::new(x, y, PICKER_W, PICKER_H);
    assert!(!cruza(r, direita), "por cima da coluna da direita: {r:?}");
    assert!(dentro(r, JANELA), "fora da janela: {r:?}");
    assert!(x + PICKER_W <= direita.x, "não está à esquerda dela");
    assert!(y >= direita.y, "acima do topo da coluna");

    let esquerda = Rect::new(48.0, 40.0, 300.0, 984.0);
    let (x, _) = picker_anchor_beside(JANELA, esquerda, PICKER_W, PICKER_H);
    let r = Rect::new(x, y, PICKER_W, PICKER_H);
    assert!(!cruza(r, esquerda), "por cima da coluna da esquerda: {r:?}");
    assert!(x >= esquerda.x + esquerda.w, "não está à direita dela");
}

/// ⭐ RED no encaixe antigo (o canto de baixo-direito do canvas = DENTRO da coluna do Inspector).
#[test]
fn o_seletor_abre_ao_lado_do_inspector_e_nao_por_cima() {
    let mut hero = aberto();
    let layout = pinta(&mut hero);
    let r = rect_publicado(&hero);
    assert!(
        !cruza(r, layout.inspector),
        "o seletor {r:?} tapa o Inspector {:?}",
        layout.inspector
    );
    assert!(dentro(r, JANELA), "o seletor {r:?} sai da janela");
}

/// O arrasto real pela pega: fechar e reabrir volta ao sítio onde o artista o largou, e a posição
/// marca o que se grava.
#[test]
fn reabre_onde_o_artista_o_largou() {
    let mut hero = aberto();
    pinta(&mut hero);
    let antes = rect_publicado(&hero);
    let pega = hero
        .hit_index
        .rect_for(ids::BLENDER_DRAG_HANDLE)
        .expect("a pega está publicada");
    let (px, py) = meio(pega);
    let _ = hero.store.take_section_prefs_dirty();
    manda(&mut hero, PointerKind::Down, px, py);
    manda(&mut hero, PointerKind::Move, px - 200.0, py + 100.0);
    manda(&mut hero, PointerKind::Up, px - 200.0, py + 100.0);
    assert!(
        hero.store.take_section_prefs_dirty(),
        "largar o seletor não marcou a posição para gravar"
    );
    let x = hero.hit_index.rect_for(ids::BLENDER_CLOSE).expect("o ×");
    let (cx, cy) = meio(x);
    pinta(&mut hero);
    let x = hero.hit_index.rect_for(ids::BLENDER_CLOSE).expect("o ×");
    assert!(
        cx != meio(x).0 || cy != meio(x).1,
        "o arrasto não moveu o ×"
    );
    let (cx, cy) = meio(x);
    clica(&mut hero, cx, cy);
    assert_eq!(hero.store.picker_target(), None, "o × não fechou");
    hero.store.set_picker_target(Some(NodeId(0x0C0_1013)));
    pinta(&mut hero);
    let depois = rect_publicado(&hero);
    assert_eq!((depois.x, depois.y), (antes.x - 200.0, antes.y + 100.0));
}

/// A posição sobrevive ao fecho do app (`sections.txt`); um ficheiro antigo, sem a chave, lê-se.
#[test]
fn a_posicao_viaja_pelo_ficheiro_e_o_ficheiro_antigo_ainda_le() {
    let mut hero = aberto();
    hero.store
        .set_blender_picker_offset(ids::INSP_BLENDER_PICKER, -150.0, 80.5);
    pinta(&mut hero);
    let antes = rect_publicado(&hero);
    let texto = section_prefs_text(&hero.store);
    assert!(texto.contains("picker_offset="), "não grava: {texto}");

    let mut novo = aberto();
    restore_section_prefs_text(&mut novo.store, &texto);
    pinta(&mut novo);
    assert_eq!(
        rect_publicado(&novo),
        antes,
        "o arranque não repôs a posição"
    );

    // Ficheiro de antes desta onda: sem a chave, e com um valor ilegível noutra.
    let mut velho = aberto();
    restore_section_prefs_text(&mut velho.store, "order=1,2\npicker_offset=lixo\n");
    assert_eq!(velho.store.section_order(), &[NodeId(1), NodeId(2)]);
    assert_eq!(
        velho.store.blender_picker_offset(ids::INSP_BLENDER_PICKER),
        (0.0, 0.0)
    );
    assert!(!section_prefs_text(&velho.store).contains("picker_offset"));
}

/// ⭐⭐ O pino por um clique REAL no rect publicado; fixado ⇒ um Down fora não fecha. Desafixado ⇒
/// fecha (o CONTROLO). O × fecha e desafixa.
#[test]
fn fixado_um_clique_fora_nao_fecha_e_desafixado_fecha() {
    let fora = (JANELA.w * 0.25, JANELA.h * 0.5);
    // CONTROLO: sem pino, o clique fora fecha.
    let mut hero = aberto();
    pinta(&mut hero);
    assert!(!rect_publicado(&hero).contains(fora.0, fora.1));
    clica(&mut hero, fora.0, fora.1);
    assert_eq!(
        hero.store.picker_target(),
        None,
        "CONTROLO: fora não fechou"
    );

    let mut hero = aberto();
    pinta(&mut hero);
    let pino = hero.hit_index.rect_for(ids::BLENDER_PIN).expect("o pino");
    let (px, py) = meio(pino);
    clica(&mut hero, px, py);
    assert!(hero.store.picker_pinned(), "o clique no pino não o fixou");
    assert_eq!(hero.store.picker_target(), Some(ALVO));
    pinta(&mut hero);
    clica(&mut hero, fora.0, fora.1);
    assert_eq!(hero.store.picker_target(), Some(ALVO), "fixado e fechou");
    // Outro dono reabre: só muda o alvo.
    hero.store.set_picker_target(Some(NodeId(0x0C0_1014)));
    assert!(hero.store.picker_pinned());
    pinta(&mut hero);
    let pino = hero.hit_index.rect_for(ids::BLENDER_PIN).expect("o pino");
    let (px, py) = meio(pino);
    clica(&mut hero, px, py);
    assert!(
        !hero.store.picker_pinned(),
        "o segundo clique não desafixou"
    );
    hero.store.set_picker_pinned(true);
    pinta(&mut hero);
    let x = hero.hit_index.rect_for(ids::BLENDER_CLOSE).expect("o ×");
    let (cx, cy) = meio(x);
    clica(&mut hero, cx, cy);
    assert_eq!(hero.store.picker_target(), None, "o × não fecha o fixado");
    assert!(!hero.store.picker_pinned(), "o × não desafixou");
}

/// ⭐ **Todo fecho desafixa** (auditoria 24/10): o pino vale para ESTA abertura. Fechado por outra porta que não o
/// «×» (a amostra clicada outra vez, um painel que fecha, o Inspector que troca de entidade — todas chamam
/// `set_picker_target(None)`), a abertura seguinte nasce solta e um clique fora volta a fechá-la. Vermelho antes:
/// só o «×» desafixava.
#[test]
fn fechado_por_qualquer_porta_reabre_solto() {
    let fora = (JANELA.w * 0.25, JANELA.h * 0.5);
    let mut hero = aberto();
    hero.store.set_picker_pinned(true);
    hero.store.set_picker_target(None);
    assert!(!hero.store.picker_pinned(), "fechou e ficou fixado");
    hero.store.set_picker_target(Some(NodeId(0x0C0_1014)));
    assert!(!hero.store.picker_pinned(), "reabriu fixado");
    pinta(&mut hero);
    clica(&mut hero, fora.0, fora.1);
    assert_eq!(
        hero.store.picker_target(),
        None,
        "reaberto, o clique fora não fecha"
    );
}

/// A porta é ÚNICA: só `set_picker_target` escreve o alvo — senão um fecho podia saltá-la e o pino ficar.
#[test]
fn so_a_porta_escreve_o_alvo_do_seletor() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    // Partida em dois, para este ficheiro não se contar a si próprio.
    let agulha = [".picker_target", " = "].concat();
    let mut escritas = Vec::new();
    let mut pilha = vec![std::path::PathBuf::from(dir)];
    while let Some(p) = pilha.pop() {
        for e in std::fs::read_dir(&p).expect("src legível").flatten() {
            let f = e.path();
            if f.is_dir() {
                pilha.push(f);
            } else if f.extension().is_some_and(|x| x == "rs") {
                let txt = std::fs::read_to_string(&f).expect("rs legível");
                let n = txt.matches(agulha.as_str()).count();
                if n > 0 {
                    escritas.push((f.display().to_string(), n));
                }
            }
        }
    }
    assert_eq!(
        escritas.len(),
        1,
        "escritas do alvo fora da porta: {escritas:?}"
    );
    assert!(escritas[0].0.ends_with("chrome_ops.rs") && escritas[0].1 == 1);
}
