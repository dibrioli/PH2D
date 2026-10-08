//! Os gates do **MODELO do radial** — a secção que ele toma, e o que acontece ao que não cabe.

use super::*;

fn items(n: usize) -> Vec<RadialItem> {
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            RadialItem::leaf(format!("T{i}"), NodeId(1000 + i as u64))
        })
        .collect()
}

/// **O que cabe passa intacto.**
#[test]
fn a_list_that_fits_travels_untouched() {
    for n in 0..=MAX_SECTORS {
        assert_eq!(fit(items(n)), items(n), "n={n} foi mexido sem precisar");
    }
}

/// ⛔ **O QUE NÃO CABE NÃO É TRUNCADO EM SILÊNCIO: o último sector é a PORTA.**
///
/// ⚠️ É o gate que impede a linha mais fácil de escrever errada — um `truncate(8)` mudo. O modo de
/// falha dele é o pior: o artista procura uma ferramenta que o menu decidiu não mostrar, e não há
/// nada na tela a dizer que ela existe.
#[test]
fn what_does_not_fit_gets_a_door_never_a_silent_cut() {
    for n in [MAX_SECTORS + 1, MAX_SECTORS + 5, 40] {
        let out = fit(items(n));
        assert_eq!(
            out.len(),
            MAX_SECTORS,
            "n={n}: o radial passou dos sectores"
        );
        let last = out.last().expect("o radial não pode sair vazio");
        assert_eq!(
            last.id,
            RADIAL_MORE,
            "n={n}: {} itens foram cortados EM SILÊNCIO — o último sector tem de ser a porta \
             para a paleta",
            n - MAX_SECTORS + 1
        );
        assert_eq!(last.label, MORE_LABEL.tr());
        // E os sete primeiros são os sete primeiros — o transbordo não reordena nada.
        assert_eq!(&out[..MAX_SECTORS - 1], &items(n)[..MAX_SECTORS - 1]);
    }
}

/// **A porta NÃO nasce quando a lista cabe.**
///
/// ⚠️ O controle do gate acima: sem ele, um `fit` que acrescentasse *"More…"* sempre passaria nos
/// dois — e o artista teria um sector morto no menu de quatro ferramentas.
#[test]
fn the_door_does_not_appear_when_everything_fits() {
    for n in 0..=MAX_SECTORS {
        assert!(
            !fit(items(n)).iter().any(|i| i.id == RADIAL_MORE),
            "n={n}: a porta apareceu num radial que já mostrava tudo"
        );
    }
}

/// ⭐ **O MODELO TOMA A SECÇÃO DO MEIO DO RAIL — as FERRAMENTAS.**
///
/// ⚠️ Ele não pode tomar a lista inteira: o rail começa com os interruptores de painel e acaba com
/// espaço/vista/undo/redo. Um radial que oferecesse *"Show Inspector"* e *"Undo"* sob a caneta
/// gastaria duas das oito direcções em coisas que já têm tecla.
///
/// Medido em 2026-08-23: **4** no modo normal, **13** no Painter.
#[test]
fn the_model_takes_the_rails_tool_section() {
    let hero = HeroScreen::new(NodeId(1));
    let model = build_radial_model(&hero);
    assert!(!model.is_empty(), "o radial saiu vazio no modo por omissão");
    for banned in [
        "Show Inspector",
        "Show Hierarchy",
        "Undo",
        "Redo",
        "Frame view",
    ] {
        assert!(
            !model.iter().any(|i| i.label == banned),
            "o radial ofereceu `{banned}` — ele gasta uma das oito direcções numa coisa que não é \
             ferramenta (e que já tem tecla)"
        );
    }
    // O CONTROLE: as ferramentas de facto estão lá.
    assert!(
        model.iter().any(|i| i.label == "Translate"),
        "a secção de ferramentas não chegou ao radial: {:?}",
        model.iter().map(|i| &i.label).collect::<Vec<_>>()
    );
}

/// ⭐ **A COLUNA DE UM MÓDULO ENTRA INTEIRA, A DOIS GESTOS** (F4) — um sector por grupo, a família
/// abre em leque, as amostras ficam de fora, e o último sector é a porta para a paleta.
///
/// ⚠️ A forma da coluna é a do vetor (2 · 4 · 4 · 4 · 3 · cor), 17 ferramentas: com «as 7 mais
/// usadas + mais» dez delas iriam para trás da porta; com as famílias, nenhuma.
#[test]
fn a_module_column_enters_whole_two_strokes_deep() {
    use crate::interaction::{BarButton, BarFace, BarFamily};
    let b = |id: u64, family| BarButton {
        id: NodeId(id),
        face: BarFace::Icon(crate::icons::IconId::Select),
        label: format!("B{id}"),
        on: false,
        family,
    };
    let sw = |id: u64| BarButton {
        face: BarFace::Swatch {
            rgba: None,
            ring: false,
        },
        ..b(id, BarFamily::Colour)
    };
    let fams = [
        (BarFamily::Neutral, 2),
        (BarFamily::Draw, 4),
        (BarFamily::Build, 4),
        (BarFamily::Carve, 4),
        (BarFamily::Stroke, 3),
    ];
    let mut next = 1u64;
    let mut column: Vec<Vec<BarButton>> = fams
        .iter()
        .map(|&(f, k)| {
            (0..k)
                .map(|_| {
                    next += 1;
                    b(next, f)
                })
                .collect()
        })
        .collect();
    column.push(vec![
        sw(90),
        sw(91),
        b(92, BarFamily::Colour),
        b(93, BarFamily::Colour),
        b(94, BarFamily::Colour),
    ]);
    let model = from_column(&column);
    assert_eq!(model.len(), 7, "seis grupos e a porta");
    assert_eq!(
        model.last().map(|i| i.id),
        Some(RADIAL_MORE),
        "a porta não é a última"
    );
    let leaves: Vec<NodeId> = model
        .iter()
        .flat_map(|i| {
            if i.children.is_empty() {
                vec![i.id]
            } else {
                i.children.iter().map(|c| c.id).collect()
            }
        })
        .collect();
    for g in &column {
        for btn in g {
            let n = leaves.iter().filter(|&&id| id == btn.id).count();
            let want = usize::from(!matches!(btn.face, BarFace::Swatch { .. }));
            assert_eq!(n, want, "{} aparece {n}× (esperado {want})", btn.label);
        }
    }
    assert_eq!(model[0].label, ph2d_i18n::tr(BarFamily::Neutral.name_key()));
    assert_eq!(model[5].label, ph2d_i18n::tr(BarFamily::Colour.name_key()));
}
