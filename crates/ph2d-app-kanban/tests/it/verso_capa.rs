//! O popover da CAPA (report do dono 28/10: «não vejo como escolher imagens para Cover») pela
//! costura do hero: o «Upload a cover image» (o diálogo respondido pelo mundo do teste, pela MESMA
//! função do diálogo real), as miniaturas só de IMAGENS, o aviso sem imagens, largar uma imagem
//! sobre o popover e o tamanho dele no ecrã.

use crate::verso::{abre, alvo, toca};
use crate::verso_anexos::{Mundo, anexos, capa, ficheiro, png};
use crate::{AREA, Mesa, centro};
use ph2d_editor_core::screens::hero::surface_input;
use ph2d_kanban_model::{CardId, CoverFill, Hue, KanbanId, Link};

fn pintados(m: &mut Mesa, id: KanbanId) -> Vec<String> {
    m.docs()
        .probe_back_hits(id.0)
        .into_iter()
        .map(|(n, _)| n)
        .collect()
}

/// O popover inteiro dentro da área da aba.
fn cabe(m: &mut Mesa, id: KanbanId) {
    let r = alvo(m, id, "Pop(Inside)");
    assert!(
        r.y >= AREA.y && r.y + r.h <= AREA.y + AREA.h && r.x >= AREA.x && r.x + r.w <= AREA.x + AREA.w,
        "o popover da capa sai do ecrã: {r:?} em {AREA:?}"
    );
}

fn cheio(m: &Mesa, id: KanbanId, c: CardId) -> bool {
    m.kanban(id).cards[&c].cover.is_some_and(|c| c.full)
}

#[test]
fn capa_mostra_o_enviar_imagem_e_o_aviso_sem_imagens() {
    let (mut m, id, _) = abre("Write the logline");
    // Um PDF anexado NÃO é miniatura de capa.
    toca(&mut m, id, "Side(Attachment)");
    toca(&mut m, id, "Pop(PickFile)");
    let mut w = Mundo {
        escolha: Some(vec![ficheiro("capa-aviso", "brief.pdf", b"%PDF-1.4")]),
        ..Mundo::default()
    };
    ph2d_app_kanban::service_with(&mut m.hero, &mut w);
    m.quadro();
    toca(&mut m, id, "Side(Cover)");
    let hits = pintados(&mut m, id);
    assert!(
        hits.iter().any(|h| h == "Pop(CoverUpload)"),
        "o botão «Upload a cover image»: {hits:?}"
    );
    assert!(
        !hits.iter().any(|h| h.starts_with("Pop(CoverImage(")),
        "um PDF não se oferece como capa: {hits:?}"
    );
    cabe(&mut m, id);
}

#[test]
fn capa_enviar_imagem_anexa_e_poe_capa_num_so_passo() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "Side(Cover)");
    toca(&mut m, id, "Pop(CoverHue(3))");
    toca(&mut m, id, "Pop(CoverSize(true))");
    toca(&mut m, id, "Pop(CoverUpload)");
    assert!(m.docs().probe_back_outbox(id.0).1, "o diálogo foi pedido");
    let mut w = Mundo {
        escolha: Some(vec![ficheiro("capa-enviar", "poster.png", &png(64, 32))]),
        ..Mundo::default()
    };
    ph2d_app_kanban::service_with(&mut m.hero, &mut w);
    m.quadro();
    let v = anexos(&m, id, c);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].name, "poster.png");
    let Link::File(f) = v[0].link else {
        panic!("um anexo de ficheiro: {:?}", v[0].link)
    };
    assert_eq!(capa(&m, id, c), Some(CoverFill::Image(f)), "a capa trocou");
    assert!(cheio(&m, id, c), "o tamanho escolhido fica");
    assert_eq!(
        m.docs().probe_back_state(id.0).0.as_deref(),
        Some("Cover"),
        "o popover continua aberto"
    );
    let hits = pintados(&mut m, id);
    assert!(
        hits.contains(&format!("Pop(CoverImage({:?}))", v[0].id)),
        "a miniatura da imagem nova: {hits:?}"
    );
    // UM passo de desfazer: o anexo sai e a capa volta à cor.
    m.desfaz();
    assert!(anexos(&m, id, c).is_empty());
    assert_eq!(capa(&m, id, c), Some(CoverFill::Color(Hue::ALL[3])));
}

#[test]
fn capa_enviar_o_que_nao_e_imagem_avisa_e_nao_muda_nada() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "Side(Cover)");
    toca(&mut m, id, "Pop(CoverHue(2))");
    toca(&mut m, id, "Pop(CoverUpload)");
    let mut w = Mundo {
        escolha: Some(vec![ficheiro("capa-texto", "notes.txt", b"shot notes")]),
        ..Mundo::default()
    };
    let served = m.docs().serve(&mut w);
    assert_eq!(served.notices.len(), 1, "{:?}", served.notices);
    assert!(served.notices[0].contains("notes.txt"));
    assert!(anexos(&m, id, c).is_empty(), "não se anexa");
    assert_eq!(capa(&m, id, c), Some(CoverFill::Color(Hue::ALL[2])));
}

#[test]
fn capa_largar_imagem_sobre_o_popover_troca_a_capa() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "Side(Cover)");
    let a = ficheiro("capa-largar", "a.png", &png(8, 8));
    let pop = alvo(&mut m, id, "Pop(Inside)");
    assert!(surface_input::drop_files(
        &mut m.hero,
        centro(pop).0,
        centro(pop).1,
        std::slice::from_ref(&a)
    ));
    m.quadro();
    let fa = match anexos(&m, id, c)[0].link {
        Link::File(f) => f,
        ref l => panic!("{l:?}"),
    };
    assert_eq!(capa(&m, id, c), Some(CoverFill::Image(fa)));
    // Fora do popover (no verso) anexa, mas a capa que já há fica.
    let b = ficheiro("capa-largar", "b.png", &png(9, 9));
    assert!(surface_input::drop_files(
        &mut m.hero,
        AREA.x + 2.0,
        AREA.y + AREA.h - 2.0,
        std::slice::from_ref(&b)
    ));
    m.quadro();
    assert_eq!(anexos(&m, id, c).len(), 2);
    assert_eq!(capa(&m, id, c), Some(CoverFill::Image(fa)), "fora não é capa");
    // Sobre o popover, a nova passa a capa; um texto avisa e não entra.
    let pop = alvo(&mut m, id, "Pop(Inside)");
    let t = ficheiro("capa-largar", "t.txt", b"x");
    assert!(surface_input::drop_files(
        &mut m.hero,
        centro(pop).0,
        centro(pop).1,
        &[t]
    ));
    m.quadro();
    assert_eq!(anexos(&m, id, c).len(), 2, "o texto não entra");
    for i in 0..7u32 {
        let p = ficheiro("capa-largar", &format!("n{i}.png"), &png(10 + i, 6));
        let pop = alvo(&mut m, id, "Pop(Inside)");
        assert!(surface_input::drop_files(
            &mut m.hero,
            centro(pop).0,
            centro(pop).1,
            &[p]
        ));
        m.quadro();
    }
    let v = anexos(&m, id, c);
    assert_eq!(v.len(), 9);
    let Link::File(last) = v[8].link else {
        panic!("ficheiro")
    };
    assert_eq!(capa(&m, id, c), Some(CoverFill::Image(last)));
    // Nove miniaturas (três filas) e o popover ainda cabe no ecrã.
    let n = pintados(&mut m, id)
        .iter()
        .filter(|h| h.starts_with("Pop(CoverImage("))
        .count();
    assert_eq!(n, 9);
    cabe(&mut m, id);
}

/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --profile smoke --test it d10_foto_capa -- --ignored`
#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/D10/"]
fn d10_foto_capa_enviar_imagem() {
    use ph2d_tokens::Theme;
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let (mut m, id, _) = abre("Write the logline");
        m.hero.theme = theme;
        m.ts = ph2d_text::TextSystem::new();
        m.quadro();
        toca(&mut m, id, "Side(Cover)");
        m.quadro();
        let mut s = ph2d_vector::VectorScene::new();
        m.pinta(&mut s);
        crate::j2_fotos::grava_em("D10", &format!("capa_sem_imagens_{tag}"), &s);
        for (i, (w, h)) in [(64u32, 32u32), (40, 40), (30, 50)].into_iter().enumerate() {
            let p = ficheiro("capa-foto", &format!("img{i}.png"), &png(w, h));
            let pop = alvo(&mut m, id, "Pop(Inside)");
            surface_input::drop_files(&mut m.hero, centro(pop).0, centro(pop).1, &[p]);
            m.quadros(3);
        }
        m.quadros(10);
        let mut s = ph2d_vector::VectorScene::new();
        m.pinta(&mut s);
        crate::j2_fotos::grava_em("D10", &format!("capa_com_imagens_{tag}"), &s);
    }
}
