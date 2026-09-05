//! Gates do readout de valor da row — a lei da largura e a lei da precisão.

use super::{VALUE_W, fits, text};

/// **O NÚMERO NUNCA É CORTADO** — a razão de a precisão ceder antes da largura.
///
/// ⚠️ Mutação que tem de sangrar: `format!("{v:.2}")` para todas as bandas. `123456.78` sairia com
/// 9 caracteres numa fatia de 6, e o corte come os algarismos do FIM — o artista leria `12345…`
/// sobre um valor que é dez vezes maior.
#[test]
fn the_readout_never_outgrows_its_slot() {
    for v in [
        0.0_f32,
        0.5,
        -std::f32::consts::PI,
        99.994,
        100.4,
        -1234.56,
        99_999.9,
    ] {
        let s = text(v);
        assert!(
            s.chars().count() <= 7,
            "`{v}` saiu como `{s}` — nao cabe na fatia de {VALUE_W} px"
        );
    }
}

/// **A BANDA DE BAIXO TEM AS MESMAS DUAS CASAS DO GRÁFICO** — o editor de curvas rotula
/// `{v:.2}`, e é nessa banda que vivem opacidade, escala e radianos.
///
/// *Duas superfícies que mostram a mesma grandeza com precisões diferentes leem-se como dois
/// valores.*
#[test]
fn the_common_band_matches_the_graph_editors_precision() {
    assert_eq!(text(0.47), "0.47");
    assert_eq!(text(-std::f32::consts::PI), "-3.14");
}

/// **NUMA COLUNA ESPREMIDA O NÚMERO SAI DE CENA** — quem identifica a row é o nome.
///
/// ⚠️ O piso da coluna (56 px, menos o twirl) deixa ~32 px para o nome: ali um número roubaria
/// mais de metade do que sobra.
#[test]
fn a_squeezed_column_drops_the_number_instead_of_the_name() {
    // Piso da coluna (56) menos o twirl (24) e o respiro: ~28 px de nome.
    assert!(!fits(28.0), "no piso da coluna o nome fica sem nada");
    // Largura de fabrica (176) menos os mesmos: ~148 px.
    assert!(fits(148.0), "na largura de fabrica ele cabe");
}

/// ⭐⭐⭐ **O NÚMERO CHEGA A TINTA** — a metade que nenhum gate de dados alcança.
///
/// ⛔⛔ Os gates acima provam a lei da largura e a da precisão; a shell prova que o valor é
/// publicado. **Nenhum dos três prova que ele é PINTADO** — e este repo tem a lição escrita: um
/// `if` no braço errado, um `right` fora da coluna ou um `Text3` sobre o próprio fundo deixam-nos
/// todos verdes e o artista continua a ver o painel parado, que é o report de origem.
///
/// A régua é o número de GLIFOS que o painel emite: ⚠️ **não** a altura nem um retângulo — espaço
/// reservado não emite glifo nenhum (achado §4.2 da auditoria do `source.lsystem`).
#[test]
fn the_number_reaches_the_glyphs() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_timeline::{
        AnimTarget, Extrap, PropKind, TimelineViewSnapshot, TrackValues, TrackView,
    };

    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let glyphs = |com_valor: bool| {
        let tracks = vec![TrackView {
            target: AnimTarget::new(11),
            prop: PropKind::Opacity,
            entity: 7,
            missing: false,
            keys: Vec::new(),
            buffer_ghost: None,
            pre: Extrap::default(),
            post: Extrap::default(),
            expr: None,
        }];
        let mut values = TrackValues::default();
        if com_valor {
            values.publish(&tracks, |_, _| Some(0.47));
        }
        let snap = TimelineViewSnapshot {
            tracks,
            values,
            ..TimelineViewSnapshot::default()
        };
        crate::set_current_timeline(Some(snap));
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
        let mut state = crate::state::TimelinePanelState::default();
        host.paint_and_count_geometry::<crate::TimelinePanel>(&mut state, viewport)
            .0
    };
    let sem = glyphs(false);
    let com = glyphs(true);
    assert!(
        com > sem,
        "a row publicou um valor e o painel emitiu os MESMOS {sem} glifos — o numero nao chega a \
         tinta, e o painel continua parado enquanto a animacao corre"
    );
}
