//! A janela em cartões (P12 S): o ponteiro em píxeis físicos chega ao botão certo sob a escala do
//! chrome, o resto do ecrã é comido, o `Esc` = `Later`; a linha de base depois de abrir; o fecho.

use std::path::PathBuf;
use std::time::Instant;

use ph2d_editor_core::ui_scale::UiScaleMap;
use ph2d_editor_core::zones::Rect;
use ph2d_recovery::card::{self, Hit};
use ph2d_recovery::{Choice, Orphan, Quit, Recovery};
use ph2d_tokens::UiScale;

fn orphan(name: &str) -> Orphan {
    Orphan {
        dir: PathBuf::new(),
        copy: PathBuf::new(),
        meta: super::meta(0),
        size: 3 << 20,
    }
    .renamed(name)
}

trait Renamed {
    fn renamed(self, n: &str) -> Self;
}
impl Renamed for Orphan {
    fn renamed(mut self, n: &str) -> Self {
        self.meta.name = n.into();
        self
    }
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// O clique real: o centro FÍSICO de cada botão (o lógico × a escala do chrome, a 150 % num ecrã
/// HiDPI 2×) dá a resposta daquele cartão; o véu e o corpo do cartão são comidos; `Later` fecha.
#[test]
fn o_ponteiro_responde_ao_cartao_certo_sob_a_escala_e_o_resto_e_comido() {
    let mut r = Recovery::off();
    r.prompt.entries = vec![orphan("A.ph2dproj"), orphan("B.ph2dproj")];
    let map = UiScaleMap::no_ecra(UiScale::P150, 2.0);
    let screen = Rect::new(0.0, 0.0, 2560.0, 1600.0);
    r.screen = Some((screen, map));
    let l = card::layout(map.rect_to_logical(screen), 2);
    assert_eq!(l.rows.len(), 2, "os dois cartões cabem");
    let phys = |rect: Rect| centre(map.rect_to_physical(rect));
    let (x, y) = phys(l.rows[1].restore);
    assert_eq!(
        card::click(&mut r, x, y),
        Some(Hit::Answer(1, Choice::Restore))
    );
    let (x, y) = phys(l.rows[0].discard);
    assert_eq!(
        card::click(&mut r, x, y),
        Some(Hit::Answer(0, Choice::Discard))
    );
    let texto = Rect {
        w: l.rows[0].discard.x - l.rows[0].card.x,
        ..l.rows[0].card
    };
    let (x, y) = phys(texto);
    assert_eq!(
        card::click(&mut r, x, y),
        Some(Hit::Swallowed),
        "o texto do cartão"
    );
    assert_eq!(card::click(&mut r, 1.0, 1.0), Some(Hit::Swallowed), "o véu");
    assert!(r.prompt.is_open());
    let (x, y) = phys(l.later);
    assert_eq!(card::click(&mut r, x, y), Some(Hit::Later));
    assert!(!r.prompt.is_open(), "Later fecha a janela");
    assert_eq!(
        card::click(&mut r, x, y),
        None,
        "fechada, o clique é do editor"
    );
}

/// `Esc` = `Later` (as cópias ficam no disco: o `later_all` não apaga nada) e `Cancel` no fecho.
#[test]
fn esc_e_later_no_arranque_e_cancel_no_fecho() {
    let mut r = Recovery::off();
    r.prompt.entries = vec![orphan("A.ph2dproj")];
    assert!(r.is_modal());
    card::key_escape(&mut r);
    assert!(!r.is_modal());
    r.quit = Quit::Asking("A".into());
    assert!(r.is_modal());
    card::key_escape(&mut r);
    assert_eq!(r.quit, Quit::Idle);
}

/// A pergunta do fecho: os três botões pelo ponteiro.
#[test]
fn o_fecho_pergunta_save_dont_save_cancel_pelo_ponteiro() {
    let mut r = Recovery::off();
    r.quit = Quit::Asking("A".into());
    let map = UiScaleMap::new(UiScale::P100);
    let screen = Rect::new(0.0, 0.0, 1280.0, 800.0);
    r.screen = Some((screen, map));
    let q = card::quit_layout(screen);
    let (x, y) = centre(q.save);
    assert_eq!(card::click(&mut r, x, y), Some(Hit::Save));
    let (x, y) = centre(q.dont_save);
    assert_eq!(card::click(&mut r, x, y), Some(Hit::DontSave));
    let (x, y) = centre(q.cancel);
    assert_eq!(card::click(&mut r, x, y), Some(Hit::Cancel));
    assert_eq!(r.quit, Quit::Idle, "Cancel fecha a pergunta");
}

/// «Today 14:32» · «Yesterday» · a data, em inglês; e a linha do cartão leva o tamanho.
#[test]
fn o_cartao_diz_hoje_ontem_ou_a_data_e_o_tamanho() {
    let utc = jiff::tz::TimeZone::UTC;
    // 2026-11-08 14:32 UTC = 1 794 148 320.
    let t = 1_794_148_320;
    assert_eq!(card::when_in(t, t + 60, &utc), "Today 14:32");
    assert_eq!(card::when_in(t, t + 86_400, &utc), "Yesterday 14:32");
    assert_eq!(card::when_in(t, t + 5 * 86_400, &utc), "Nov 8, 14:32");
    let d = card::detail(&orphan("A"), ph2d_recovery::now_secs());
    assert!(d.ends_with("\u{b7} 3 MB"), "{d}");
}

/// ⭐ A linha de base: depois de abrir, a primeira hora com os MESMOS bytes não escreve; um Open
/// marca o disco do artista (nada por gravar), um `Restore` fica por gravar.
#[test]
fn abrir_poe_a_linha_de_base_e_restore_fica_por_gravar() {
    let root = super::tmpdir("baseline");
    let mut r = Recovery::start(Some(&root), ph2d_recovery::Period::default());
    let now = Instant::now();
    r.opened(Some(b"aberto"), false);
    let later = now + r.autosave.period.duration() + std::time::Duration::from_secs(1);
    assert!(!r.due(later), "sem entrada do artista a hora não serializa");
    r.touched = true;
    assert!(r.due(later));
    assert!(!r.unsaved(b"aberto"));
    assert!(r.unsaved(b"mudou"));
    r.offer(b"aberto".to_vec(), None, later);
    assert_eq!(r.autosave.finish(), Some(ph2d_recovery::Offer::Unchanged));
    r.opened(Some(b"restaurado"), true);
    assert!(r.unsaved(b"restaurado"), "restaurado = por gravar");
    r.offer(b"restaurado".to_vec(), None, later);
    assert_eq!(
        r.autosave.finish(),
        Some(ph2d_recovery::Offer::Unchanged),
        "mas a cópia dele já está no disco"
    );
    r.close();
}
