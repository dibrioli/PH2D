//! D11 — o fundo do Kanban pela costura do hero (report do dono 28/10: «o fundo da Board deveria ser
//! personalizável com cores e padrões de desenho»): o menu do quadro abre a gaveta «Change
//! background», cada escolha é UM passo de desfazer, o seletor da cor livre grava UMA vez ao largar,
//! o «Upload…» pela porta do mundo, e cada fundo pinta coisa diferente.

use crate::verso_anexos::{Mundo, ficheiro};
use crate::{AREA, Mesa, centro};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_kanban_model::{Background, Hue, KanbanId, Pattern, PatternKind, Tint};
use ph2d_vector::VectorScene;

/// Os alvos da gaveta aberta.
pub(crate) fn alvos(m: &mut Mesa, id: KanbanId) -> Vec<(String, Rect)> {
    m.docs().probe_background_rows(id.0)
}

fn alvo(m: &mut Mesa, id: KanbanId, nome: &str) -> Rect {
    alvos(m, id)
        .into_iter()
        .find(|(n, _)| n == nome)
        .unwrap_or_else(|| panic!("{nome} não está na gaveta"))
        .1
}

/// O «…» do cabeçalho → «Change background».
pub(crate) fn abre(m: &mut Mesa, id: KanbanId) {
    let menu = m.docs().probe_rect(id.0, "menu").expect("o «…»");
    m.clica(centro(menu));
    let linha = m
        .docs()
        .probe_menu_rows(id.0)
        .into_iter()
        .find(|(n, _)| n == "Background")
        .expect("a linha «Change background» no menu do quadro")
        .1;
    m.clica(centro(linha));
    assert!(!alvos(m, id).is_empty(), "a gaveta abriu");
}

/// Toca `nome`, rolando a gaveta até ele se ver (a roda sobre ela).
pub(crate) fn toca(m: &mut Mesa, id: KanbanId, nome: &str) {
    let mut r = alvo(m, id, nome);
    for _ in 0..40 {
        if r.y + r.h <= AREA.y + AREA.h {
            break;
        }
        m.roda(r.x + r.w / 2.0, AREA.y + AREA.h / 2.0, -1.0);
        r = alvo(m, id, nome);
    }
    m.clica(centro(r));
}

fn fundo(m: &Mesa, id: KanbanId) -> Background {
    m.kanban(id).header.background
}

#[test]
fn d11_the_board_menu_opens_the_drawer_and_each_choice_is_one_undo_step() {
    let (mut m, id) = Mesa::cena1();
    let antes = fundo(&m, id);
    abre(&mut m, id);
    let nomes: Vec<String> = alvos(&mut m, id).into_iter().map(|(n, _)| n).collect();
    for n in ["Upload", "Color(Green)", "Gradient(8)", "Custom(Solid)", "Pattern(Dots)"] {
        assert!(nomes.iter().any(|x| x == n), "{n} em {nomes:?}");
    }
    assert!(
        !nomes.iter().any(|x| x.starts_with("Tint(")),
        "as cores do padrão só com um padrão"
    );
    toca(&mut m, id, "Color(Green)");
    assert_eq!(fundo(&m, id), Background::Color(Hue::Green));
    assert!(!alvos(&mut m, id).is_empty(), "a gaveta fica aberta (o Trello)");
    m.desfaz();
    assert_eq!(fundo(&m, id), antes, "um passo de desfazer");
    // Um padrão, e depois a cor da base dele.
    toca(&mut m, id, "Pattern(Waves)");
    assert_eq!(
        fundo(&m, id),
        Background::Pattern(Pattern::new(PatternKind::Waves))
    );
    toca(&mut m, id, "Tint(Base, Red)");
    let Background::Pattern(p) = fundo(&m, id) else {
        panic!("padrão");
    };
    assert_eq!((p.kind, p.base), (PatternKind::Waves, Tint::Hue(Hue::Red)));
    toca(&mut m, id, "Pattern(Grid)");
    let Background::Pattern(p) = fundo(&m, id) else {
        panic!("padrão");
    };
    assert_eq!(
        (p.kind, p.base),
        (PatternKind::Grid, Tint::Hue(Hue::Red)),
        "trocar o desenho guarda as cores"
    );
    m.desfaz();
    m.desfaz();
    assert_eq!(
        fundo(&m, id),
        Background::Pattern(Pattern::new(PatternKind::Waves))
    );
}

#[test]
fn d11_the_custom_colour_previews_while_dragging_and_saves_once_on_release() {
    let (mut m, id) = Mesa::cena1();
    abre(&mut m, id);
    toca(&mut m, id, "Gradient(2)");
    toca(&mut m, id, "Custom(Solid)");
    let sv = alvo(&mut m, id, "Sv");
    let hue = alvo(&mut m, id, "HueStrip");
    assert!(hue.y + hue.h <= AREA.y + AREA.h, "o seletor abre à vista: {hue:?}");
    let de = (sv.x + 4.0, sv.y + 4.0);
    let para = (sv.x + sv.w * 0.8, sv.y + sv.h * 0.3);
    m.arrasta(de, para, 6);
    assert_eq!(
        fundo(&m, id),
        Background::Gradient(2),
        "a arrastar só se mostra"
    );
    assert!(m.larga(para.0, para.1));
    m.quadro();
    let Background::Custom(rgb) = fundo(&m, id) else {
        panic!("a cor livre gravou: {:?}", fundo(&m, id));
    };
    let [r, g, b] = rgb.0;
    assert!(
        b > r && b > g,
        "o seletor parte do matiz do fundo actual (o gradiente azul): {rgb:?}"
    );
    // A faixa do matiz move só o matiz.
    m.clica((hue.x + 1.0, hue.y + hue.h / 2.0));
    let Background::Custom(rgb2) = fundo(&m, id) else {
        panic!("cor livre");
    };
    assert!(rgb2.0[0] > rgb2.0[2], "o vermelho do início da faixa: {rgb2:?}");
    m.desfaz();
    assert_eq!(fundo(&m, id), Background::Custom(rgb));
    m.desfaz();
    assert_eq!(
        fundo(&m, id),
        Background::Gradient(2),
        "o arrasto inteiro foi UM passo"
    );
}

#[test]
fn d11_escape_and_a_click_outside_close_the_drawer() {
    let (mut m, id) = Mesa::cena1();
    abre(&mut m, id);
    assert!(m.tecla(BoardKey::Escape, Modifiers::default(), None));
    m.quadro();
    assert!(alvos(&mut m, id).is_empty());
    abre(&mut m, id);
    let antes = fundo(&m, id);
    m.clica((AREA.x + 40.0, AREA.y + AREA.h - 40.0));
    assert!(alvos(&mut m, id).is_empty(), "fora fecha");
    assert_eq!(fundo(&m, id), antes, "e não muda nada");
}

/// Uma «fotografia»: céu em gradiente com um sol (`w × h`).
pub(crate) fn foto(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(w, h, |x, y| {
        let t = y as f32 / h as f32;
        let (dx, dy) = (x as f32 - w as f32 * 0.7, y as f32 - h as f32 * 0.35);
        if dx.hypot(dy) < h as f32 * 0.12 {
            return image::Rgba([255, 214, 120, 255]);
        }
        let hill = (h as f32) * (0.72 + 0.06 * (x as f32 / w as f32 * 6.0).sin());
        if (y as f32) > hill {
            return image::Rgba([40, 90 + (30.0 * t) as u8, 60, 255]);
        }
        image::Rgba([
            (40.0 + 180.0 * t) as u8,
            (90.0 + 110.0 * t) as u8,
            (200.0 - 30.0 * t) as u8,
            255,
        ])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).expect("png");
    out.into_inner()
}

/// O «Upload…» da gaveta servido por `mundo` (a mesma porta do diálogo).
pub(crate) fn sobe(m: &mut Mesa, id: KanbanId, caminho: std::path::PathBuf) -> Vec<String> {
    toca(m, id, "Upload");
    let mut mundo = Mundo {
        escolha: Some(vec![caminho]),
        ..Mundo::default()
    };
    let avisos = m.docs().serve_background(&mut mundo);
    m.quadro();
    avisos
}

#[test]
fn d11_upload_makes_the_picked_image_the_background_and_refuses_a_non_image() {
    let (mut m, id) = Mesa::cena1();
    abre(&mut m, id);
    let png = foto(64, 40);
    let avisos = sobe(&mut m, id, ficheiro("d11", "ceu.png", &png));
    assert!(avisos.is_empty(), "{avisos:?}");
    let Background::Image(f) = fundo(&m, id) else {
        panic!("imagem: {:?}", fundo(&m, id));
    };
    assert_eq!(f, ph2d_kanban_model::FileId::of(&png));
    assert!(
        m.hero.documents.kanbans().files_in_use().contains(&f),
        "a imagem vai para o disco com o projecto"
    );
    assert!(
        alvos(&mut m, id).iter().any(|(n, _)| n == "Photo"),
        "a fotografia actual na gaveta"
    );
    let avisos = sobe(&mut m, id, ficheiro("d11", "notas.txt", b"nada"));
    assert_eq!(avisos.len(), 1);
    assert_eq!(fundo(&m, id), Background::Image(f), "e o fundo fica");
    m.desfaz();
    assert_ne!(fundo(&m, id), Background::Image(f), "UM passo de desfazer");
}

/// A pintura inteira da aba, gravada.
fn pintura(m: &mut Mesa) -> String {
    let mut s = VectorScene::new();
    s.record();
    m.pinta(&mut s);
    format!("{:?}", s.take_record())
}

#[test]
fn d11_each_background_kind_paints_the_tab_differently() {
    let (mut m, id) = Mesa::cena1();
    abre(&mut m, id);
    let mut vistas = Vec::new();
    for nome in [
        "Color(Green)",
        "Gradient(1)",
        "Pattern(Dots)",
        "Pattern(Diagonal)",
        "Tint(Ink, Yellow)",
    ] {
        toca(&mut m, id, nome);
        // A gaveta fecha-se para comparar só o quadro.
        assert!(m.tecla(BoardKey::Escape, Modifiers::default(), None));
        m.quadro();
        vistas.push(pintura(&mut m));
        abre(&mut m, id);
    }
    let n = vistas.len();
    vistas.sort();
    vistas.dedup();
    assert_eq!(vistas.len(), n, "cada fundo pinta coisa diferente");
}
