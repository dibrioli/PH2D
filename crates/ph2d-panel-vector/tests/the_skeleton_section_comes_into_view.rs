//! ⭐⭐⭐ **A SECÇÃO SKELETON VEM À VISTA quando um osso entra em foco.**
//!
//! ⛔⛔ **Report do dono (2026-09-08): *«selecionar o bone nem sempre abre a secção de skeleton no
//! painel»*.** A causa não é o painel esquecer-se dela — é ela **nunca caber na tela**: medido
//! neste mesmo arnês, sobre um viewport de `900 px`, o cabeçalho cai em `y = 1316 px` com só um
//! osso escolhido e em `1978 px` com uma forma de traço na selecção. O *«nem sempre»* é o painel já
//! estar rolado até lá por outra razão.
//!
//! ⚠️ **O oráculo é o PIXEL, não o pedido:** um gate que perguntasse *«o pedido foi consumido?»*
//! ficaria verde com a secção exactamente onde estava. O que se mede aqui é o retângulo que o
//! cabeçalho ocupa **depois** do quadro seguinte, contra a faixa do painel.
//!
//! ⚠️ **O CONTROLO é a metade que dá sentido à medição** — sem o pedido, o mesmo estado deixa o
//! cabeçalho abaixo da dobra. Sem ele o gate passaria sobre um painel que já nascesse rolado.

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// A cena tem esqueleto e há um osso em foco — o estado do report.
fn publica_um_osso() {
    state::set_current_has_skeleton(true);
    state::set_current_skinned(true);
    state::set_current_bone(Some((20.0, 1.0)));
}

/// Onde o cabeçalho da secção ficou, e onde está a faixa do painel.
fn cabecalho_e_painel(host: &mut MockPanelHost, st: &mut VectorPanelState) -> (Rect, Rect) {
    let h = host
        .painted_rect::<VectorPanel>(st, VIEWPORT, ids::VECTOR_SECTION_BONE)
        .expect("o cabecalho da seccao SKELETON nao foi pintado");
    let p = host
        .store()
        .panel_rect(ids::VECTOR_PANEL)
        .expect("o painel nao publicou o proprio rect");
    (h, p)
}

/// ⭐⭐⭐ **Com um osso em foco, o cabeçalho fica DENTRO da faixa do painel** — e o controlo mostra
/// que, sem o pedido, ele fica abaixo dela.
///
/// ⚠️ São **dois** hosts, e não dois quadros do mesmo: a rolagem é estado do `WidgetStore`, então
/// medir o controlo depois da cura leria o painel já rolado.
#[test]
fn a_bone_in_focus_brings_the_skeleton_section_into_view() {
    publica_um_osso();

    // ── CONTROLO: sem pedido, a secção nasce fora da dobra.
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let (antes, painel) = cabecalho_e_painel(&mut host, &mut st);
    assert!(
        antes.y > painel.y + painel.h,
        "o CONTROLO desta medicao caiu: o cabecalho ja' nasce dentro da faixa (y={}, faixa \
         {}..{}), logo o gate abaixo passaria sobre um painel que nunca precisou de rolar",
        antes.y,
        painel.y,
        painel.y + painel.h
    );

    // ── A CURA: a shell pede, o painel rola, e o quadro seguinte mostra-a.
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    state::set_reveal_bone_section(true);
    let _ = cabecalho_e_painel(&mut host, &mut st);
    let (depois, painel) = cabecalho_e_painel(&mut host, &mut st);
    assert!(
        depois.y >= painel.y && depois.y + depois.h <= painel.y + painel.h,
        "a seccao SKELETON continua fora da faixa depois do pedido: cabecalho {}..{}, painel \
         {}..{} — e' exactamente o report do dono («selecionar o bone nem sempre abre a seccao»)",
        depois.y,
        depois.y + depois.h,
        painel.y,
        painel.y + painel.h
    );
}

/// ⭐ **Um cabeçalho JÁ à vista não se mexe.** ⛔ Sem esta metade a cura seria um salto por clique:
/// o artista escolhe outro osso e o painel arranca-lhe o sítio onde ele estava a ler.
///
/// ⚠️⚠️ **A posição de partida tem de ser visível E longe das DUAS bordas do que a rolagem
/// alcança** — a 1.ª redacção deste gate punha-a no fundo do documento, onde a correcção que ele
/// proíbe é engolida pelo `clamp` e vale exactamente zero. *Uma fixtura que não produz o fenómeno
/// deixa o gate verde sobre a mutação que o devia matar.*
#[test]
fn a_section_already_in_view_is_left_where_it_is() {
    publica_um_osso();
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let (cru, painel) = cabecalho_e_painel(&mut host, &mut st);
    let conteudo = host
        .store()
        .panel_content_h(ids::VECTOR_PANEL)
        .expect("o painel nao publicou a altura do conteudo");
    let visivel = host
        .store()
        .panel_visible_h(ids::VECTOR_PANEL)
        .expect("o painel nao publicou a altura visivel");
    let maximo = conteudo - visivel;

    // Põe o cabeçalho perto do FUNDO da faixa: dentro dela, e com espaço de rolagem de sobra dos
    // dois lados — é aí, e só aí, que «não mexer» é uma afirmação com conteúdo.
    const FOLGA_PX: f32 = 120.0;
    let alvo_y = painel.y + painel.h - FOLGA_PX;
    let rolagem = cru.y - alvo_y;
    assert!(
        rolagem > 0.0 && rolagem < maximo,
        "a fixtura nao produz o fenomeno: rolagem {rolagem} fora de 0..{maximo}"
    );
    host.store_mut()
        .set_panel_scroll(ids::VECTOR_PANEL, rolagem);
    let (assente, painel) = cabecalho_e_painel(&mut host, &mut st);
    assert!(
        (assente.y - alvo_y).abs() < 1.0
            && assente.y >= painel.y
            && assente.y + assente.h <= painel.y + painel.h,
        "a fixtura nao ficou onde devia: {} contra {alvo_y}",
        assente.y
    );

    state::set_reveal_bone_section(true);
    let _ = cabecalho_e_painel(&mut host, &mut st);
    let (depois, _) = cabecalho_e_painel(&mut host, &mut st);
    assert!(
        (depois.y - assente.y).abs() < 0.5,
        "o pedido mexeu um cabecalho que ja' estava a' vista ({} -> {}) — isto arranca ao artista \
         o sitio onde ele estava a ler",
        assente.y,
        depois.y
    );
}
