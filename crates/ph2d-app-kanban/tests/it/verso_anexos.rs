//! O verso e o MUNDO (W3, lote D3) pela costura do hero: anexar ficheiros pelo diálogo (o resultado
//! injectado pela MESMA função que o diálogo chama — `service_with`) e largados do sistema
//! (`surface_input::drop_files`), a capa de imagem real, abrir links e anexos fora, o «Copy link»,
//! o duplo/triplo clique e o Shift no editor rico, e as datas do verso na frente.

use crate::verso::{abre, alvo, cartao, sem, toca};
use crate::{AREA, Mesa, centro};
use ph2d_app_kanban::{Outside, World};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::screens::hero::surface_input;
use ph2d_editor_core::widget::{CalendarLayout, DatePickerLayout};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_kanban_model::{Attachment, CardId, CoverFill, KanbanId, Link};
use std::path::PathBuf;

/// O mundo de um teste: o que o diálogo «escolhe» e o que se pediu para abrir.
#[derive(Default)]
struct Mundo {
    escolha: Option<Vec<PathBuf>>,
    aberto: Vec<String>,
}

impl World for Mundo {
    fn pick_files(&mut self) -> Option<Vec<PathBuf>> {
        self.escolha.take()
    }

    fn open(&mut self, target: &str) -> bool {
        self.aberto.push(target.to_owned());
        true
    }
}

/// Um PNG de `w × h` (liso).
fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([200, 120, 40, 255]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .expect("png");
    out.into_inner()
}

/// Escreve `bytes` em `<tmp>/ph2d-kanban-it-<pid>-<caso>/<nome>`.
fn ficheiro(caso: &str, nome: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ph2d-kanban-it-{}-{caso}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("pasta");
    let p = dir.join(nome);
    std::fs::write(&p, bytes).expect("escrever");
    p
}

fn anexos(m: &Mesa, id: KanbanId, c: CardId) -> Vec<Attachment> {
    let mut v: Vec<Attachment> = m
        .kanban(id)
        .attachments
        .values()
        .filter(|a| a.card == c && !a.meta.deleted)
        .cloned()
        .collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos));
    v
}

fn capa(m: &Mesa, id: KanbanId, c: CardId) -> Option<CoverFill> {
    m.kanban(id).cards[&c].cover.map(|c| c.fill)
}

#[test]
fn verso_anexa_ficheiros_pelo_dialogo_com_capa_real_e_abre_do_temporario() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "Side(Attachment)");
    toca(&mut m, id, "Pop(PickFile)");
    assert!(m.docs().probe_back_outbox(id.0).1, "o diálogo foi pedido");
    let imagem = png(40, 30);
    let fox = ficheiro("dialogo", "fox.png", &imagem);
    let notas = ficheiro("dialogo", "notes.txt", b"shot notes");
    let mut w = Mundo {
        escolha: Some(vec![fox, notas]),
        ..Mundo::default()
    };
    ph2d_app_kanban::service_with(&mut m.hero, &mut w);
    m.quadro();
    assert!(!m.docs().probe_back_outbox(id.0).1, "o pedido serviu-se");
    let v = anexos(&m, id, c);
    let nomes: Vec<&str> = v.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(nomes, ["fox.png", "notes.txt"]);
    let Link::File(f) = v[0].link else {
        panic!("a imagem é um anexo de ficheiro: {:?}", v[0].link)
    };
    {
        let set = m.hero.documents.kanbans();
        let k = set.file(f).expect("os bytes no conjunto");
        assert_eq!(k.size, Some([40, 30]), "o tamanho natural pelo cabeçalho");
        assert_eq!(&k.bytes.0[..], &imagem[..]);
    }
    assert_eq!(
        capa(&m, id, c),
        Some(CoverFill::Image(f)),
        "a 1.ª imagem passa a capa"
    );
    // «Remove cover» / «Make cover» com a imagem real.
    toca(&mut m, id, &format!("AttCover({:?})", v[0].id));
    assert_eq!(capa(&m, id, c), None);
    toca(&mut m, id, &format!("AttCover({:?})", v[0].id));
    assert_eq!(capa(&m, id, c), Some(CoverFill::Image(f)));
    // Abrir o anexo: uma cópia no temporário, entregue ao programa do sistema.
    toca(&mut m, id, &format!("AttOpen({:?})", v[1].id));
    assert_eq!(
        m.docs().probe_back_outbox(id.0).0,
        vec![Outside::File(
            match v[1].link {
                Link::File(t) => t,
                _ => panic!("ficheiro"),
            },
            "notes.txt".into()
        )]
    );
    ph2d_app_kanban::service_with(&mut m.hero, &mut w);
    let aberto = w.aberto.last().expect("abriu-se");
    assert!(aberto.ends_with("notes.txt"), "{aberto}");
    assert_eq!(std::fs::read(aberto).expect("cópia"), b"shot notes");
    // Cada ficheiro foi UM passo de desfazer; o da imagem leva a capa com ele.
    m.desfaz();
    m.desfaz();
    m.desfaz();
    m.desfaz();
    assert_eq!(anexos(&m, id, c).len(), 0);
    assert_eq!(capa(&m, id, c), None);
}

#[test]
fn verso_recebe_ficheiros_largados_do_sistema_no_cartao_e_no_verso() {
    let (mut m, id) = Mesa::cena1();
    let musica = cartao(&m, id, "Pick the music");
    let (x, y) = centro(m.rect_cartao(id, musica));
    let mix = ficheiro("largar", "mix.png", &png(8, 8));
    assert!(surface_input::drop_files(
        &mut m.hero,
        x,
        y,
        std::slice::from_ref(&mix)
    ));
    m.quadro();
    let v = anexos(&m, id, musica);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].name, "mix.png");
    assert!(matches!(capa(&m, id, musica), Some(CoverFill::Image(_))));
    // Fora de um cartão, a aba não toma o que se largou (a shell segue o caminho dela).
    assert!(!surface_input::drop_files(
        &mut m.hero,
        AREA.x + AREA.w - 2.0,
        AREA.y + AREA.h - 2.0,
        std::slice::from_ref(&mix)
    ));
    // Com o verso aberto, o que se larga em qualquer sítio é do cartão dele.
    let logline = cartao(&m, id, "Write the logline");
    let r = m.rect_cartao(id, logline);
    m.clica(crate::pega(r));
    assert_eq!(m.docs().probe_back(id.0), Some(logline));
    let doc = ficheiro("largar", "brief.pdf", b"%PDF-1.4");
    let falta = std::env::temp_dir().join("ph2d-kanban-it-nao-existe.bin");
    assert!(surface_input::drop_files(
        &mut m.hero,
        AREA.x + 2.0,
        AREA.y + AREA.h - 2.0,
        &[doc, falta]
    ));
    m.quadro();
    let v = anexos(&m, id, logline);
    assert_eq!(v.len(), 1, "o que não se lê não entra");
    assert_eq!(v[0].name, "brief.pdf");
    assert_eq!(capa(&m, id, logline), None, "um PDF não é capa");
    let served = m.docs().serve(&mut Mundo::default());
    assert_eq!(served.notices.len(), 1, "{:?}", served.notices);
    assert!(served.notices[0].contains("ph2d-kanban-it-nao-existe.bin"));
}

#[test]
fn verso_abre_links_no_navegador_e_copia_o_link_do_cartao() {
    let (mut m, id, c) = abre("Collect references for the forest");
    let v = anexos(&m, id, c);
    let Link::Url(u) = v[0].link.clone() else {
        panic!("a cena tem links")
    };
    toca(&mut m, id, &format!("AttOpen({:?})", v[0].id));
    let mut w = Mundo::default();
    ph2d_app_kanban::service_with(&mut m.hero, &mut w);
    assert_eq!(w.aberto, vec![u], "o link vai ao navegador do sistema");
    assert!(m.docs().probe_back_outbox(id.0).0.is_empty(), "e sai da fila");
    toca(&mut m, id, "Side(Share)");
    toca(&mut m, id, "Pop(CopyLink)");
    ph2d_app_kanban::service_with(&mut m.hero, &mut w);
    assert_eq!(
        m.hero.store.take_clipboard_copy(),
        Some(format!("ph2d://kanban/{}/card/{}", id.0, c.0)),
        "o botão copia sem tecla"
    );
}

/// Carregar e largar em `p` com `mods`, no instante `agora` da mesa (sem pintar no meio).
fn clique(m: &mut Mesa, p: (f32, f32), mods: Modifiers) {
    for kind in [PointerKind::Down, PointerKind::Up] {
        let input = Input {
            text: &mut m.ts,
            mods,
            now_ns: m.agora,
        };
        board_view::pointer(
            &mut m.hero,
            input,
            kind,
            PointerButton::Primary,
            p.0,
            p.1,
            true,
        );
    }
}

#[test]
fn verso_duplo_e_triplo_clique_e_shift_estendem_a_seleccao_da_descricao() {
    let (mut m, id, _) = abre("Write the logline");
    toca(&mut m, id, "DescEdit");
    let r = alvo(&mut m, id, "DescArea");
    let (md, xs) = (
        ph2d_tokens::Spacing::Md.px(),
        ph2d_tokens::Spacing::Xs.px(),
    );
    let inicio = (r.x + md + 3.0, r.y + xs + ph2d_tokens::ROW_H_PX + md + 6.0);
    let sel = |m: &mut Mesa| m.docs().probe_back_selection(id.0, true).unwrap_or_default();
    clique(&mut m, inicio, sem());
    assert_eq!(sel(&mut m), "");
    clique(&mut m, inicio, sem());
    assert_eq!(sel(&mut m), "One", "o duplo clique escolhe a palavra");
    clique(&mut m, inicio, sem());
    assert_eq!(sel(&mut m), "One sentence.", "o triplo clique, o bloco");
    // Passada a janela do duplo clique: um clique no fim da linha e o Shift+clique no início.
    m.agora += 1_000_000_000;
    clique(&mut m, (r.x + r.w - md - 2.0, inicio.1), sem());
    assert_eq!(sel(&mut m), "");
    m.agora += 1_000_000_000;
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    clique(&mut m, inicio, shift);
    let s = sel(&mut m);
    assert!(
        s.ends_with("sentence.") && s.len() >= "ne sentence.".len(),
        "o Shift estende do caret ao clique: {s:?}"
    );
}

#[test]
fn verso_as_datas_mudadas_no_verso_chegam_a_frente() {
    let (mut m, id, c) = abre("Write the logline");
    m.docs().probe_arm_fronts();
    m.quadro();
    let antes = m.docs().probe_painted_front("Write the logline").expect("pintada");
    assert_eq!(antes.3, None, "sem datas");
    assert!(m.tecla(BoardKey::Char('d'), sem(), Some("d")));
    m.quadro();
    let picker = alvo(&mut m, id, "Pop(Picker)");
    let cal = DatePickerLayout::of(picker).calendar;
    let dia = CalendarLayout::of(cal).cell(20);
    m.clica((dia.x + dia.w / 2.0, dia.y + dia.h / 2.0));
    let save = DatePickerLayout::of(alvo(&mut m, id, "Pop(Picker)")).save;
    m.clica((save.x + save.w / 2.0, save.y + save.h / 2.0));
    assert!(m.kanban(id).cards[&c].due.is_some(), "o documento mudou");
    let depois = m.docs().probe_painted_front("Write the logline").expect("pintada");
    assert!(depois.3.is_some(), "a frente releu a entrega no quadro seguinte");
    m.tecla(BoardKey::Escape, sem(), None);
    m.quadro();
    let fechado = m.docs().probe_painted_front("Write the logline").expect("pintada");
    assert_eq!(fechado.3, depois.3, "e fica com o verso fechado");
}
