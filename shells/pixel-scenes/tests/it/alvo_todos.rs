//! **O alvo «All» dos filtros** (rodada 3 no Trial, `oracle_trial_round3.json` chave `C_targets`):
//! o botão do diálogo alterna «Selected» → «All»; com «All» o filtro age em todos os quadros × todas
//! as camadas que se pintam, como UM passo; o Preview mostra só o cel activo. A 2.ª leitura do
//! Trial («reaberto, só a camada de baixo do quadro 1») é o alvo «Selected» lembrado com a camada de
//! baixo activa — nada mais a reproduzir.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{PixelEditAction as E, PixelRequest};
use ph2d_pixel_core::{CelKey, Sprite};
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

/// A cena 9 com 2 quadros (o 2.º copiado) × 2 camadas (a nova pintada em cada quadro), sem selecção
/// nem história — o `multi.ase` da rodada 3.
fn multi() -> Palco {
    use ph2d_app_pixel::Tool;
    use ph2d_app_pixel::compose::Mods;
    use ph2d_editor_core::pixel_panel::{PixelFrameAction as F, PixelLayerAction as L};
    let mut p = Palco::limpo();
    p.pede(PixelRequest::FrameAction(F::NewFrame));
    p.pede(PixelRequest::LayerAction(L::NewLayer));
    p.ferramenta(Tool::RectMarquee);
    for (cor, quadro) in [
        ([160, 160, 160, 255], F::Next),
        ([200, 200, 200, 255], F::First),
    ] {
        p.pede(PixelRequest::FrameAction(quadro));
        p.app.fg = cor;
        p.arrasta((0, 0), (6, 14), Mods::default());
        p.tecla(KeyCode::KeyF, false, false);
    }
    p.pede(PixelRequest::Select(
        ph2d_editor_core::pixel_panel::PixelSelectAction::Deselect,
    ));
    p
}

fn carrega(p: &mut Palco, id: ph2d_editor_core::NodeId) {
    p.hero.apply_event(WidgetEvent::Click(id));
    p.quadro();
}

/// Os cels que «All» deve tocar: todos os quadros × as camadas de imagem visíveis e destrancadas.
fn todos(s: &Sprite) -> Vec<CelKey> {
    s.layers()
        .iter()
        .filter(|l| !l.is_group() && !l.locked && s.is_shown(l.id))
        .flat_map(|l| s.frames().iter().map(move |f| (l.id, f.id)))
        .collect()
}

fn invertido(c: [u8; 4]) -> [u8; 4] {
    [255 - c[0], 255 - c[1], 255 - c[2], c[3]]
}

#[test]
fn invert_with_target_all_changes_every_frame_and_layer_in_one_step() {
    let mut p = multi();
    let antes = p.sprite().clone();
    let keys = todos(&antes);
    let frames = antes.frames().len();
    assert!(
        frames >= 2 && keys.len() >= 2 * frames,
        "a cena tem ≥ 2 quadros × ≥ 2 camadas"
    );
    let n = p.passos();
    p.pede(PixelRequest::Edit(E::Invert));
    assert!(
        !p.hero.store.pixel_edit_dialog().filter.all,
        "omissão: «Selected»"
    );
    carrega(&mut p, ids::PIXEL_FILTER_TARGET);
    assert!(
        p.hero.store.pixel_edit_dialog().filter.all,
        "um clique: «All»"
    );
    assert_eq!(*p.sprite(), antes, "o Preview não grava");
    carrega(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(p.passos(), n + 1, "UM passo para todos os cels");
    let (w, h) = (antes.width() as i32, antes.height() as i32);
    let mut mudaram = 0;
    let mut donos: Vec<CelKey> = keys.iter().map(|&k| antes.owner(k)).collect();
    donos.sort_unstable();
    donos.dedup();
    for &k in &donos {
        let mut mudou = false;
        for y in 0..h {
            for x in 0..w {
                let c = antes.rgba_of_value(antes.value_at(k, x, y));
                let got = p.sprite().rgba_of_value(p.sprite().value_at(k, x, y));
                assert_eq!(got, invertido(c), "cel {k:?} ({x},{y})");
                mudou |= c[3] > 0;
            }
        }
        mudaram += usize::from(mudou);
    }
    // As cels ligadas (o New Frame pode ligar) contam uma vez: a camada nova tem duas próprias.
    assert!(mudaram >= 3, "só {mudaram} cels com tinta mudaram");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes, "um Ctrl+Z desfaz todos os cels");
    // Reaberto: o alvo fica lembrado na sessão.
    p.pede(PixelRequest::Edit(E::Invert));
    assert!(
        p.hero.store.pixel_edit_dialog().filter.all,
        "«All» lembrado"
    );
}

/// Com «All» o Preview mostra o cel activo invertido e os outros cels do documento ficam.
#[test]
fn target_all_previews_only_the_active_cel() {
    let mut p = multi();
    let antes = p.sprite().clone();
    let (layer, frame) = {
        let d = p.app.doc(p.key).expect("doc");
        (d.layer, d.frame)
    };
    let (w, h) = (antes.width() as i32, antes.height() as i32);
    let cell = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .find(|&(x, y)| antes.rgba_of_value(antes.value_at((layer, frame), x, y))[3] > 0)
        .expect("o cel activo tem tinta");
    p.pede(PixelRequest::Edit(E::Invert));
    carrega(&mut p, ids::PIXEL_FILTER_TARGET);
    let key = p.key;
    let shown = p.app.shown_rgba(key, cell.0, cell.1).expect("doc");
    let c = antes.rgba_of_value(antes.value_at((layer, frame), cell.0, cell.1));
    assert_eq!(shown, invertido(c), "o Preview mostra o cel activo");
    assert_eq!(*p.sprite(), antes, "nada gravado");
}

fn cel_igual(a: &Sprite, b: &Sprite, k: CelKey) -> bool {
    (0..a.height() as i32)
        .all(|y| (0..a.width() as i32).all(|x| a.value_at(k, x, y) == b.value_at(k, x, y)))
}

/// Invert + OK com o alvo escolhido; devolve o sprite de antes.
fn inverte(p: &mut Palco, all: bool) -> Sprite {
    let antes = p.sprite().clone();
    p.pede(PixelRequest::Edit(E::Invert));
    if all {
        carrega(p, ids::PIXEL_FILTER_TARGET);
    }
    carrega(p, ids::PIXEL_EDIT_OK);
    antes
}

/// Com o alvo «Selected» (a omissão) só o cel activo muda: os outros quadros e camadas ficam.
#[test]
fn target_selected_changes_only_the_active_cel() {
    let mut p = multi();
    let (layer, frame) = {
        let d = p.app.doc(p.key).expect("doc");
        (d.layer, d.frame)
    };
    let antes = inverte(&mut p, false);
    let keys = todos(&antes);
    assert!(keys.len() >= 4, "a cena tem ≥ 2 × 2 cels");
    let ativo = antes.owner((layer, frame));
    assert!(!cel_igual(&antes, p.sprite(), ativo), "o activo mudou");
    for k in keys.into_iter().filter(|&k| antes.owner(k) != ativo) {
        assert!(cel_igual(&antes, p.sprite(), k), "{k:?} ficou");
    }
}

/// «All» não toca nas camadas trancadas, escondidas, nem cria cel num grupo.
#[test]
fn target_all_skips_locked_and_hidden_layers_and_groups() {
    use ph2d_editor_core::pixel_panel::PixelLayerAction as L;
    for caso in ["locked", "hidden"] {
        let mut p = multi();
        let ativa = p.app.doc(p.key).expect("doc").layer;
        let outra = p
            .sprite()
            .layers()
            .iter()
            .map(|l| l.id)
            .find(|&l| l != ativa)
            .expect("2 camadas");
        p.pede(if caso == "locked" {
            PixelRequest::LayerLocked(outra.0)
        } else {
            PixelRequest::LayerVisible(outra.0)
        });
        let antes = inverte(&mut p, true);
        let frames: Vec<_> = antes.frames().iter().map(|f| f.id).collect();
        for &f in &frames {
            assert!(
                cel_igual(&antes, p.sprite(), (outra, f)),
                "{caso}: {outra:?} ficou"
            );
        }
        assert!(
            frames
                .iter()
                .any(|&f| !cel_igual(&antes, p.sprite(), (ativa, f))),
            "{caso}: a outra camada mudou"
        );
    }
    let mut p = multi();
    p.pede(PixelRequest::LayerAction(L::NewGroup));
    let grupo = p
        .sprite()
        .layers()
        .iter()
        .find(|l| l.is_group())
        .map(|l| l.id)
        .expect("um grupo");
    let n = p.passos();
    inverte(&mut p, true);
    assert_eq!(p.passos(), n + 1, "um passo");
    let s = p.sprite();
    assert!(
        s.frames().iter().all(|f| s.cel((grupo, f.id)).is_none()),
        "o grupo não ganha cel"
    );
}
