//! R7c: o painel compacto — caixas do tamanho da tecla, o pé sem corte em nenhum modo de texto, «Press a key» no
//! amarelo do Input Map (plano InputMap §4).

use ph2d_editor_core::text_elide::elisao;
use ph2d_editor_core::zones::Rect;
use ph2d_input::{InputMap, Key, PlayerKeys};
use ph2d_text::TextSystem;
use ph2d_tokens::{TextRendering, Theme};
use ph2d_vector::VectorScene;

use crate::{ControlsScreen, Layout, Outcome, rebind, rows, rows_marked};

const F1: Key = Key(0xF704);
const ENTER: Key = Key(0x0D);
const LEFT: Key = Key(0xF702);

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

/// As janelas do plano §8 e a tela da prévia C4 (16:9 de 448 px, o céu sem a faixa do chão).
const AREAS: [(f32, f32); 3] = [(1600.0, 1000.0), (1280.0, 720.0), (448.0, 224.0)]; // LITERAL-PX-OK: janelas medidas

/// ⛔ Em `CrispHeavy` a dica «Up and Down choose…» saía cortada a 304 px (foto P1). O pintor não corta NADA do
/// painel — fechado, à escuta, sem acções, com o aviso da tecla repetida — em nenhum dos três modos de texto.
#[test]
fn o_painel_do_jogador_nao_corta_texto_em_nenhum_modo() {
    let g = game();
    let mut k = PlayerKeys::new();
    assert_eq!(rebind(&g, &mut k, "jump", 1, LEFT), Outcome::Changed);
    let repeated = rows_marked(&k.apply_over(&g), &g);
    for modo in [
        TextRendering::Default,
        TextRendering::CrispHeavy,
        TextRendering::CrispHeavyPlus,
    ] {
        ph2d_text::set_active_text_rendering(modo);
        let mut ts = TextSystem::new();
        for (w, h) in AREAS {
            let area = Rect::new(0.0, 0.0, w, h);
            let (mut closed, mut listening) =
                (ControlsScreen::default(), ControlsScreen::default());
            let mut kk = PlayerKeys::new();
            listening.handle_key(F1, true, &g, &mut kk);
            listening.handle_key(ENTER, true, &g, &mut kk);
            assert!(listening.listening(), "Enter escuta");
            closed.open();
            for (at, r, s) in [
                ("fechado", rows(&g), &closed),
                ("à escuta", rows(&g), &listening),
                ("sem acções", Vec::new(), &closed),
                ("tecla repetida", repeated.clone(), &closed),
            ] {
                let mut scene = VectorScene::new();
                let (lay, medidos) = elisao::medindo(|| {
                    crate::paint(area, &r, s, &mut scene, &mut ts, Theme::default())
                });
                let cortados: Vec<String> = (medidos.iter())
                    .filter(|x| !x.coube())
                    .map(|x| format!("{:?} em {:.2} px ({})", x.texto, x.largura, x.onde))
                    .collect();
                assert_eq!(
                    cortados,
                    Vec::<String>::new(),
                    "{modo:?} {w}×{h} {at}: o pintor cortou"
                );
                assert!(!medidos.is_empty(), "{modo:?} {at}: o censo viu o pintor");
                assert!(
                    lay.panel.w <= w && lay.panel.h <= h,
                    "{modo:?} {w}×{h} {at}: cabe"
                );
            }
        }
    }
    ph2d_text::set_active_text_rendering(TextRendering::Default);
}

/// As caixas medem a tecla: «J» é mais estreita que «Left Arrow», e a que escuta alarga para «Press a key».
#[test]
fn as_caixas_tem_o_tamanho_da_tecla() {
    let g = game();
    let r = rows(&g);
    let mut ts = TextSystem::new();
    let area = Rect::new(0.0, 0.0, 1280.0, 720.0);
    let lay = Layout::of(area, &r, &mut ts);
    let at = |name: &str| r.iter().position(|x| x.action == name).expect("acção");
    let left = at("move_left");
    let (wl, wa) = (lay.rows[left].slots[0].w, lay.rows[left].slots[1].w);
    assert!(wa < wl, "«A» ({wa}) mais estreita que «Left Arrow» ({wl})");
    let mut s = ControlsScreen::default();
    let mut k = PlayerKeys::new();
    s.handle_key(F1, true, &g, &mut k);
    for _ in 0..left {
        s.handle_key(Key(0xF701), true, &g, &mut k);
    }
    s.handle_key(Key(0xF703), true, &g, &mut k);
    s.handle_key(ENTER, true, &g, &mut k);
    assert!(s.listening() && s.selected() == left && s.slot() == 1);
    let l2 = Layout::scrolled(area, &r, &s, &mut ts);
    assert!(
        l2.rows[left].slots[1].w > wa,
        "a caixa à escuta mede «Press a key»"
    );
    assert!(
        lay.reset.y > lay.rows.last().expect("linhas").band.y,
        "o pé fica abaixo das linhas"
    );
    assert!(lay.rule.y < lay.reset.y, "o fio separa as linhas do pé");
}

/// ⭐ Onda 4 (foto do dono-coordenador: 228 → 272 px ao entrar em «Press a key»): o painel tem a MESMA largura em
/// repouso e à escuta, numa 2.ª caixa e numa «Add». Controlo: a caixa que escuta cresce de facto (o estado mudou).
/// E as linhas, as caixas e o Reset ficam no MESMO y (a dica de 2 linhas em repouso e 1 à escuta não as empurra);
/// controlo: a dica muda mesmo de altura. Mutação: voltar a somar as larguras do estado actual ⇒ o painel alarga; voltar
/// o `lift` ao título ⇒ o conteúdo salta.
#[test]
fn o_painel_nao_muda_de_largura_ao_escutar() {
    let g = game();
    let r = rows(&g);
    let mut ts = TextSystem::new();
    for area in [
        Rect::new(0.0, 0.0, 1600.0, 1000.0),
        Rect::new(0.0, 0.0, 1280.0, 720.0),
    ] {
        let repouso = Layout::of(area, &r, &mut ts);
        let mut dica_mudou = false;
        for (i, row) in r.iter().enumerate() {
            for slot in 1..row.slots() {
                let mut s = ControlsScreen::default();
                let mut k = PlayerKeys::new();
                s.handle_key(F1, true, &g, &mut k);
                for _ in 0..i {
                    s.handle_key(Key(0xF701), true, &g, &mut k);
                }
                for _ in 0..slot {
                    s.handle_key(Key(0xF703), true, &g, &mut k);
                }
                s.handle_key(ENTER, true, &g, &mut k);
                assert!(s.listening() && s.selected() == i && s.slot() == slot);
                let escuta = Layout::scrolled(area, &r, &s, &mut ts);
                let (a, b) = (&repouso.rows[i].slots[slot], &escuta.rows[i].slots[slot]);
                assert!(
                    b.w > a.w,
                    "{}: a caixa à escuta mede «Press a key»",
                    row.action
                );
                let ys = |l: &Layout| {
                    let mut v: Vec<f32> = l
                        .rows
                        .iter()
                        .flat_map(|r| r.slots.iter().map(|c| c.y))
                        .collect();
                    v.extend(l.rows.iter().map(|r| r.band.y));
                    v.extend([l.title.y, l.reset.y, l.panel.y, l.panel.h]);
                    v
                };
                assert_eq!(
                    ys(&escuta),
                    ys(&repouso),
                    "{} slot {slot}: o conteúdo saltou",
                    row.action
                );
                dica_mudou |= (escuta.hint.h - repouso.hint.h).abs() > 0.01;
                assert!(
                    (escuta.panel.w - repouso.panel.w).abs() < 0.01,
                    "{} slot {slot}: {} → {}",
                    row.action,
                    repouso.panel.w,
                    escuta.panel.w
                );
            }
        }
        assert!(
            dica_mudou,
            "controlo: a dica muda de linhas ao escutar (o estado mudou)"
        );
    }
}
