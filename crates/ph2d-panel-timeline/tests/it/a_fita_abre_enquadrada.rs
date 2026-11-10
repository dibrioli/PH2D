//! ⭐ Doc 15 onda 2: nos cartões a fita ABRE enquadrada no clipe (o `F` sozinho), e a reprodução que
//! passou do fim do clipe não a arrasta para a zona morta. A cena do `PH2D_ANIMACAO_SMOKE=1` é
//! exactamente esta: chaves em 1–1,5 s (como o «andar»), clipe de 2 s, relógio a tocar em 9 s.

use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_timeline::{
    AnimTarget, Extrap, Interp, KeyId, KeyView, PropKind, TimelineViewSnapshot, TrackView,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn cena(t: f64) -> TimelineViewSnapshot {
    let chave = |n: u64, t: f64| KeyView {
        id: KeyId::new(n),
        t_seconds: t,
        value: 0.0,
        interp: Interp::Linear,
        selected: false,
        roving: false,
    };
    TimelineViewSnapshot {
        fps: 24.0,
        time_seconds: t,
        playing: true,
        view_length_seconds: 2.0,
        keys_mode: true,
        tracks: vec![TrackView {
            target: AnimTarget::new(1),
            prop: PropKind::TranslationX,
            entity: 7,
            missing: false,
            keys: vec![chave(1, 1.0), chave(2, 1.5)],
            buffer_ghost: None,
            pre: Extrap::Hold,
            post: Extrap::Hold,
            expr: None,
        }],
        ..TimelineViewSnapshot::default()
    }
}

/// A vista mostra o instante `t`?
fn mostra(st: &TimelinePanelState, t: f64) -> bool {
    t >= st.view_start_s && t <= st.view_start_s + st.view_span_s
}

#[test]
fn nos_cartoes_a_fita_abre_no_clipe_mesmo_a_tocar_depois_do_fim() {
    for cartoes in [true, false] {
        force_cards_on_this_thread(Some(cartoes));
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState {
            view_start_s: 8.0,
            ..TimelinePanelState::default()
        };
        set_current_timeline(Some(cena(9.0)));
        host.paint::<TimelinePanel>(&mut st, VIEWPORT);
        host.paint::<TimelinePanel>(&mut st, VIEWPORT);
        set_current_timeline(None);
        force_cards_on_this_thread(None);
        if cartoes {
            assert!(
                mostra(&st, 0.0) && mostra(&st, 2.0),
                "o clipe inteiro, 0 → 2 s, à vista: {} + {}",
                st.view_start_s,
                st.view_span_s
            );
        } else {
            // O clássico continua a seguir o cursor como sempre (a saída de referência).
            assert!(
                mostra(&st, 9.0),
                "clássico segue o cursor: {}",
                st.view_start_s
            );
        }
    }
}

/// ⭐ Dono 09/11 («losangos muito pequenos»): nos cartões a chave pinta-se do tamanho da maquete com
/// contorno, e o alvo de clique REGISTADO acompanha — cobre o losango e o contorno inteiros,
/// centrado na chave; no clássico, o de sempre.
#[test]
fn o_alvo_da_chave_cobre_o_losango_com_contorno() {
    let id = ph2d_panel_timeline::ids::timeline_key_hit_id(1, 2);
    for cartoes in [true, false] {
        force_cards_on_this_thread(Some(cartoes));
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut st = TimelinePanelState::default();
        set_current_timeline(Some(cena(1.2)));
        host.paint::<TimelinePanel>(&mut st, VIEWPORT);
        let regs = host.paint::<TimelinePanel>(&mut st, VIEWPORT);
        set_current_timeline(None);
        force_cards_on_this_thread(None);
        let r = regs
            .iter()
            .find(|(w, _)| *w == id)
            .map(|(_, r)| *r)
            .expect("a chave em 1,5 s pintou o alvo dela");
        if cartoes {
            let losango = ph2d_tokens::TIMELINE_KEY_HALF_PX + ph2d_tokens::TIMELINE_KEY_RING_PX;
            assert!(
                r.w * 0.5 >= losango,
                "alvo {} < losango com contorno {losango}",
                r.w * 0.5
            );
            assert_eq!(r.w, ph2d_tokens::TIMELINE_KEY_HIT_HALF_PX * 2.0);
        } else {
            assert_eq!(r.w, 14.0, "o clássico fica com o alvo de sempre");
        }
    }
}

/// ⭐ Dono 10/11 (onda 3f): a fita abre com o FIM do clipe e um pouco de «além do fim» à vista, mesmo
/// quando a área de tempo muda de largura nos primeiros quadros (o cartão flutuante a assentar). O
/// clássico não enquadra ao abrir.
#[test]
fn a_fita_abre_com_o_fim_e_a_folga_a_vista_mesmo_se_a_area_muda() {
    force_cards_on_this_thread(Some(true));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();
    let mut snap = cena(0.0);
    (
        snap.playing,
        snap.view_length_seconds,
        snap.view_length_explicit,
    ) = (false, 6.0, true);
    set_current_timeline(Some(snap));
    host.paint::<TimelinePanel>(&mut st, VIEWPORT);
    let estreito = Rect::new(0.0, 0.0, 1290.0, 900.0);
    host.paint::<TimelinePanel>(&mut st, estreito);
    host.paint::<TimelinePanel>(&mut st, estreito);
    set_current_timeline(None);
    force_cards_on_this_thread(None);
    let folga = 6.0 * f64::from(ph2d_tokens::chrome::TIMELINE_ENQUADRE_MARGEM);
    let fim_da_vista = st.view_start_s + st.view_span_s;
    assert!(
        st.view_start_s <= 0.0 && fim_da_vista >= 6.0 + folga,
        "0 → 6 s e a folga de {folga} à vista: {} → {fim_da_vista}",
        st.view_start_s
    );
}
