//! Os gates da etiqueta de moldura — a GEOMETRIA dela, que é a metade que pode estar errada em
//! silêncio (o desenho é uma chamada só, e o smoke o julga).

use super::*;
use crate::zones::Rect;

/// A mesma vista dos gates da régua — o zoom mora no `camera_height_world` (menor = mais perto).
fn view() -> GridView {
    GridView {
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
    }
}

/// **A etiqueta fica ACIMA da moldura, nunca dentro dela.**
///
/// ⚠️ São dois sinais que se cancelam se um for trocado: o topo do MUNDO é `max_y` (Y-up) e subir
/// na TELA é subtrair (`+y` desce). Errar um põe o nome sobre a arte; errar os dois o põe
/// exatamente onde deveria — e é por isso que o gate mede a etiqueta contra a BORDA projetada, e
/// não contra o canto de mundo.
#[test]
fn the_label_sits_above_the_frames_top_edge() {
    let v = view();
    let (bounds, _) = world_bounds(&v);
    let top_left = [2.0_f64, 3.0_f64];

    let (lx, ly) = label_origin(&v, top_left);
    let edge_y = world_to_screen_y(top_left[1] as f32, &bounds, &v);
    let edge_x = world_to_screen_x(top_left[0] as f32, &bounds, &v);

    assert!(
        (lx - edge_x).abs() < 1e-3,
        "a etiqueta alinha pela ESQUERDA da moldura ({lx} vs {edge_x})"
    );
    assert!(
        ly < edge_y,
        "a etiqueta tem de ficar ACIMA da borda ({ly} não está acima de {edge_y}) — abaixo ela \
         cairia sobre a arte que a moldura contém"
    );
    assert!(
        (edge_y - ly - (LABEL_GAP_PX + LABEL_PX)).abs() < 1e-3,
        "a folga é `LABEL_GAP_PX + LABEL_PX` (o texto ancora pelo TOPO); sem a altura da fonte a \
         base da letra encostaria na moldura"
    );
}

/// **O tamanho não segue o zoom.** O canto ANDA com a câmara (é a moldura que a etiqueta nomeia),
/// mas a altura da fonte é de TELA — uma etiqueta que escalasse seria ilegível em todo zoom menos
/// um.
#[test]
fn the_label_travels_with_the_camera_but_never_scales() {
    let near = view();
    let far = GridView {
        camera_height_world: 40.0,
        ..view()
    };
    let a = label_origin(&near, [2.0, 3.0]);
    let b = label_origin(&far, [2.0, 3.0]);
    assert_ne!(
        a, b,
        "o canto tem de acompanhar o zoom — senão não nomeia nada"
    );

    // A folga é a MESMA nos dois: ela é medida em px de tela, e é isso que a mantém legível.
    for v in [&near, &far] {
        let (bounds, _) = world_bounds(v);
        let edge_y = world_to_screen_y(3.0, &bounds, v);
        let (_, ly) = label_origin(v, [2.0, 3.0]);
        assert!((edge_y - ly - (LABEL_GAP_PX + LABEL_PX)).abs() < 1e-3);
    }
}

/// A etiqueta de uma moldura fora de quadro **não é desenhada** — colada na borda ela apontaria
/// para nada. O gate mede a decisão pela porta que o pintor usa (`canvas.contains`).
#[test]
fn a_frame_panned_off_screen_has_no_label_on_the_edge() {
    let v = view();
    let (x, y) = label_origin(&v, [10_000.0, 10_000.0]);
    assert!(
        !v.canvas.contains(x, y),
        "um canto absurdamente longe caiu DENTRO do canvas ({x},{y}) — a fixture não contém o \
         fenómeno e o gate do recorte seria vácuo"
    );
}

/// **Arch-gate: o passe de paint do hero DESENHA as etiquetas.**
///
/// ⚠️ Os três gates acima medem a geometria e ficariam verdes com a chamada de desenho deletada —
/// eles não alcançam o `paint_hero`, que exige um `HeroScreen` vivo. Sem esta asserção a etiqueta
/// seria calculada todo frame e nunca apareceria, que é a metade silenciosa de toda feature de
/// chrome.
#[test]
fn the_hero_paint_pass_draws_the_frame_labels() {
    const PAINT: &str = include_str!("screens/hero/paint.rs");
    assert!(
        PAINT.contains("frame_label::paint_frame_labels("),
        "o passe de paint do hero deixou de desenhar as etiquetas de moldura — elas seriam \
         publicadas todo frame e nunca chegariam à tela"
    );
    assert!(
        PAINT.contains("hero.gizmo.frame_labels"),
        "o pintor deixou de ler a lista PUBLICADA — desenhar de outra fonte seria a segunda \
         resposta a *quais molduras existem?*"
    );
}
