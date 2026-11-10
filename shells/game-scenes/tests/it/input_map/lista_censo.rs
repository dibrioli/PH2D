//! A prova «0 números mortos, 0 textos cortados» (plano §3): o mapa de fábrica e o de 11 acções a 1600×1000 e a
//! 1280×720 — nenhum texto elidido, nenhum campo numérico numa acção só de teclas, e 2 no move_left com manche.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_editor_core::zones::Rect;

const SIZES: [(f32, f32); 2] = [(1600.0, 1000.0), (1280.0, 720.0)]; // LITERAL-PX-OK: as janelas do plano §8

#[test]
fn a_lista_nao_corta_texto_nem_mostra_numero_morto() {
    for (label, map) in [
        ("fábrica", ph2d_input::InputMap::with_player_defaults()),
        ("onze", onze()),
    ] {
        for (w, h) in SIZES {
            let mut m = Mesa::aberta(View::List);
            m.hero.input_map = map.clone();
            m.frame();
            m.paint_in(Rect::new(0.0, 0.0, w, h));
            let c = &m.tab().doc.census;
            assert!(!c.texts.is_empty(), "{label} {w}×{h}: a lista pintou texto");
            assert_eq!(
                c.elided(),
                Vec::<&str>::new(),
                "{label} {w}×{h}: texto cortado"
            );
            for a in map.actions() {
                let want = if a.is_analog() { 2 } else { 0 };
                assert_eq!(
                    c.numbers_of(a.id),
                    want,
                    "{label} {w}×{h}: {} tem {} números",
                    a.name,
                    c.numbers_of(a.id)
                );
            }
            let names: Vec<&str> = c.texts.iter().map(|t| t.1.as_str()).collect();
            if label == "fábrica" {
                assert!(
                    names.contains(&"8 actions · 14 keys."),
                    "{w}×{h}: o resumo {names:?}"
                );
                assert!(names.contains(
                    &"Down Arrow, Up Arrow and S are in two actions: a Platform Player and a Top-Down Player never run on the same object."
                ) || names.iter().any(|n| n.ends_with("are in two actions: a Platform Player and a Top-Down Player never run on the same object.")),
                    "{w}×{h}: a tecla repetida explicada {names:?}");
            }
        }
    }
}
