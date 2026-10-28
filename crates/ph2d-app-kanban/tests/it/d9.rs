//! D9 — defeitos vistos nas fotos do tutorial, cada um pela porta real (como os testes de costura) e,
//! onde o defeito é de PIXEL, lido na placa (o `VelloPass` do tutorial).

use crate::verso::{cartao, sem};
use crate::w6::{foca, tecla, toca};
use crate::{AREA, Mesa};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::KanbanId;
use ph2d_vector::VectorScene;

/// A aba pintada, lida da placa (RGBA, `AREA`).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn pixeis(m: &mut Mesa) -> Vec<u8> {
    let mut s = VectorScene::new();
    m.pinta(&mut s);
    let (w, h) = (AREA.w as u32, AREA.h as u32);
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    pass.render_and_readback(&gpu, s.inner(), (w, h))
        .expect("readback")
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn px(img: &[u8], x: f32, y: f32) -> [u8; 4] {
    let i = ((y as usize) * (AREA.w as usize) + x as usize) * 4;
    [img[i], img[i + 1], img[i + 2], img[i + 3]]
}

fn alvo(m: &mut Mesa, id: KanbanId, nome: &str) -> Rect {
    m.docs()
        .probe_w6_hits(id.0)
        .into_iter()
        .rev()
        .find(|(n, _)| n == nome)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"))
        .1
}

/// 4.3 — os botões da edição rápida (`E`) são opacos: os cartões da lista por baixo nunca se vêem
/// através deles. A ponta direita de «Open card» (sobre a lista DOING) tem a MESMA cor que a de
/// «Archive» (sobre o fundo do quadro).
#[test]
fn d9_os_botoes_da_edicao_rapida_tapam_as_listas_por_baixo() {
    let mut m = Mesa::new();
    m.ts = ph2d_text::TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, 5).expect("cena 5");
    m.quadros(2);
    toca(&mut m, id, "ClearAll");
    m.tecla(BoardKey::Escape, sem(), None);
    m.quadro();
    let c = cartao(&m, id, "Design the fox");
    foca(&mut m, id, c);
    tecla(&mut m, 'e');
    m.quadros(4);
    let abrir = alvo(&mut m, id, "Quick(Open)");
    let arquivar = alvo(&mut m, id, "Quick(Archive)");
    let doing = m.lista(id, "Doing");
    let lr = m
        .docs()
        .probe_list_rect(id.0, doing)
        .expect("a lista DOING");
    assert!(
        abrir.x + abrir.w > lr.x && abrir.y < lr.y + lr.h,
        "o «Open card» está sobre a DOING: {abrir:?} × {lr:?}"
    );
    let img = pixeis(&mut m);
    let ponta = |r: Rect| px(&img, r.x + r.w - 3.0, r.y + r.h / 2.0);
    assert_eq!(
        ponta(abrir),
        ponta(arquivar),
        "a lista vê-se através do botão"
    );
}

/// 4.2 — a pesquisa tem UMA caixa de edição: a do cabeçalho; o painel dos resultados só os mostra
/// (o texto não aparece duas vezes).
#[test]
fn d9_a_pesquisa_tem_uma_so_caixa_de_edicao() {
    let (mut m, id) = Mesa::cena1();
    tecla(&mut m, '/');
    m.escreve("#Animation");
    m.quadros(2);
    let caixas: Vec<Rect> = m
        .docs()
        .probe_w6_hits(id.0)
        .into_iter()
        .filter(|(n, _)| n == "SearchBox")
        .map(|(_, r)| r)
        .collect();
    assert_eq!(caixas.len(), 1, "uma só caixa: {caixas:?}");
    let topo = m
        .docs()
        .probe_rect(id.0, "menu")
        .expect("o «…» do cabeçalho");
    assert!((caixas[0].y - topo.y).abs() < 1.0, "e é a do cabeçalho");
}

/// Os textos cortados (com reticências) numa pintura da aba que são PEDAÇOS de `frases`.
fn cortadas(m: &mut Mesa, frases: &[&str]) -> Vec<String> {
    use ph2d_editor_core::text_elide::elisao;
    let ((), medidos) = elisao::medindo(|| {
        let mut s = VectorScene::new();
        m.pinta(&mut s);
    });
    medidos
        .into_iter()
        .filter(|x| !x.coube() && frases.iter().any(|f| f.contains(x.texto.as_str())))
        .map(|x| x.texto)
        .collect()
}

fn mesa_com_fontes(n: u8) -> (Mesa, KanbanId) {
    let mut m = Mesa::new();
    m.ts = ph2d_text::TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, n).expect("cena");
    m.quadros(2);
    (m, id)
}

/// 4 — o texto que EXPLICA nunca se elide: «Mirror card» e «Card aging» quebram em linhas e a
/// janela cresce (as escolhas longas do envelhecimento também).
#[test]
fn d9_as_explicacoes_das_janelas_quebram_em_linhas() {
    use ph2d_i18n::tr;
    let (mut m, id) = mesa_com_fontes(5);
    toca(&mut m, id, "ClearAll");
    m.tecla(BoardKey::Escape, sem(), None);
    m.quadro();
    let c = cartao(&m, id, "Design the fox");
    foca(&mut m, id, c);
    tecla(&mut m, 'e');
    toca(&mut m, id, "Quick(Mirror)");
    let mirror = [tr("kanban.w6.mirror_note")];
    assert_eq!(
        cortadas(&mut m, &mirror),
        Vec::<String>::new(),
        "Mirror card"
    );

    let (mut m, id) = mesa_com_fontes(6);
    let menu = m.docs().probe_rect(id.0, "menu").expect("menu");
    m.clica(crate::centro(menu));
    let linha = m
        .docs()
        .probe_menu_rows(id.0)
        .into_iter()
        .find(|(n, _)| n == "Aging")
        .expect("Card aging…")
        .1;
    m.clica(crate::centro(linha));
    let aging = [
        tr("kanban.w6.aging_note"),
        tr("kanban.w6.aging.off"),
        tr("kanban.w6.aging.regular"),
        tr("kanban.w6.aging.pirate"),
    ];
    assert_eq!(cortadas(&mut m, &aging), Vec::<String>::new(), "Card aging");
}

/// 4 — a coluna lateral do verso alarga para o rótulo mais largo: «Convert to sticky note» inteiro.
#[test]
fn d9_a_coluna_do_verso_mede_os_rotulos() {
    use ph2d_editor_core::text_elide::elisao;
    let (mut d, id, _) = crate::duas_janelas::Duas::new();
    d.ts = ph2d_text::TextSystem::new();
    let c = d
        .hero
        .documents
        .kanbans()
        .get(id)
        .unwrap()
        .cards
        .values()
        .find(|c| c.title == "Write the logline")
        .unwrap()
        .id;
    d.docs().open(id);
    d.quadros(2);
    d.docs().open_back(id.0, c);
    d.quadros(2);
    assert!(
        d.docs()
            .probe_back_hits(id.0)
            .iter()
            .any(|(n, _)| n == "Side(ToNote)"),
        "o botão está no verso"
    );
    let rotulo = ph2d_i18n::tr("kanban.links.to_note");
    let ((), medidos) = elisao::medindo(|| d.quadro());
    let cortado: Vec<_> = medidos
        .iter()
        .filter(|x| !x.coube() && rotulo.contains(x.texto.as_str()))
        .collect();
    assert!(cortado.is_empty(), "{cortado:?}");
    assert!(
        medidos.iter().any(|x| x.texto == rotulo),
        "o rótulo foi medido"
    );
}
