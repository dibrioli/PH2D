//! Gates da porta única de comprimento (plano 25 §9, a W6 — o rótulo de distância).
//!
//! O gate que carrega a wave é o primeiro: ele não conhece a implementação de
//! nenhuma das duas superfícies, só exige que elas digam **o mesmo número para a
//! mesma distância**.

use super::*;
use crate::project::{DEFAULT_PIXELS_PER_METER, DisplayUnit, ProjectSettings};
use crate::ruler::label_text;

/// A régua do artista por default: 100 px por metro, lendo em PIXELS.
fn shipping() -> LengthDisplay {
    LengthDisplay::of(&ProjectSettings::default())
}

/// **As duas superfícies dizem o mesmo número.**
///
/// O painel de Grid Snap converte por `display_unit.from_meters` (é literalmente
/// o que o `NumberInput` do passo mostra) e a régua imprime `label_text`. Antes
/// desta wave, para a MESMA distância de mundo, o painel dizia **150** e a régua
/// **2** — não por arredondamento, mas porque `paint_rulers` não recebia as
/// settings e portanto não *conseguia* converter.
///
/// Mutação que tem de sangrar: `label_text` voltar a formatar o valor CRU.
#[test]
fn the_ruler_and_the_panel_say_the_same_number_for_the_same_distance() {
    let d = shipping();
    assert_eq!(d.unit, DisplayUnit::Pixels, "o default do projeto");
    for world in [0.5_f64, 1.0, 1.5, 12.0] {
        let panel = f64::from(d.unit.from_meters(world as f32, DEFAULT_PIXELS_PER_METER));
        // O passo em MUNDO que a régua escolheria num zoom de 1 px por unidade
        // de display; o que importa aqui é o VALOR, e ele não depende do passo.
        let printed: f64 = label_text(world, 1.0, d)
            .parse()
            .expect("a régua imprime um número");
        assert!(
            (printed - panel).abs() < 0.5,
            "world {world}: régua {printed} contra painel {panel} — as duas \
             superfícies estão a descrever a mesma distância"
        );
    }
}

/// **Um projeto em METROS é byte-idêntico ao que já shipava** — o CONTROLE.
///
/// `from_meters` é a identidade nessa unidade, então a conversão não pode mover
/// um caractere. Sem este gate, a wave inteira poderia estar a mudar o mundo
/// para todo mundo em vez de só para quem escolheu pixels.
#[test]
fn reading_in_metres_prints_exactly_what_the_old_ruler_printed() {
    let d = LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: DEFAULT_PIXELS_PER_METER,
    };
    // Os mesmos pares que o `ruler_tests` pina desde a W6.2.
    assert_eq!(label_text(0.2, 0.2, d), "0.2");
    assert_eq!(label_text(1.0, 1.0, d), "1");
    assert_eq!(label_text(0.05, 0.05, d), "0.05");
    assert_eq!(
        label_text(-0.0, 1.0, d),
        "0",
        "o zero negativo lê como erro"
    );
}

/// **As casas vêm da resolução CONVERTIDA, não da resolução de mundo.**
///
/// Meio metro é `0,5` em metros (uma casa) e `50` em pixels (nenhuma).
/// Converter só o valor imprimiria `150.0` — uma casa decimal que o número não
/// tem resolução para honrar.
///
/// Mutação que tem de sangrar: `decimals_for(resolution_world)` em vez do
/// convertido.
#[test]
fn the_decimals_come_from_the_step_the_artist_reads() {
    let px = shipping();
    assert_eq!(px.text(1.5, 0.5), "150", "meio metro = 50 px: sem casas");
    let m = LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: DEFAULT_PIXELS_PER_METER,
    };
    assert_eq!(m.text(1.5, 0.5), "1.5", "meio metro em metros: uma casa");
}

/// **Uma leitura em metros carrega os CENTÍMETROS** — o repro do smoke.
///
/// ⚠️ Reprovado pelo Enio: *"em metros, só mede metros inteiros, mas deveria ser
/// metros e cm"*. E era pior que grosseiro — no zoom de trabalho a ficha
/// imprimia **`2`** para uma distância de **1,5 m**, porque emprestava a cadência
/// de rótulos da RÉGUA, que ali vale 1 m inteiro (`MIN_LABEL_PX / 100`, arredondado
/// para cima na escada 1/2/5).
///
/// Um pixel de tela nesse zoom vale 1 cm, e é essa a resolução que o número tem.
///
/// Mutação que tem de sangrar: `text_at_zoom` voltar a `ruler::label_step`.
#[test]
fn a_metre_reading_carries_its_centimetres() {
    let m = LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: DEFAULT_PIXELS_PER_METER,
    };
    // O zoom de trabalho: 100 px por metro de mundo.
    assert_eq!(
        m.text_at_zoom(1.5, 100.0),
        "1.50",
        "um pixel vale 1 cm neste zoom — a distância não pode ser arredondada para o metro"
    );
    // E não é um caso isolado do 1,5: qualquer distância traz os centímetros.
    assert_eq!(m.text_at_zoom(2.37, 100.0), "2.37");
    assert_eq!(m.text_at_zoom(0.04, 100.0), "0.04");
}

/// **A precisão é a que UM PIXEL distingue** — e o CONTROLE é que em pixels nada
/// muda.
///
/// Este par é o que separa a lei nova da velha. Em PIXELS, no zoom de trabalho,
/// um pixel de tela É um pixel de display, então o número segue inteiro — que é
/// exatamente por que o defeito era invisível na unidade default e sobreviveu à
/// wave inteira até o smoke.
///
/// Mutação que tem de sangrar: `world_per_pixel` devolver `label_step`.
#[test]
fn the_precision_is_what_one_pixel_distinguishes() {
    let px = shipping();
    let m = LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: DEFAULT_PIXELS_PER_METER,
    };
    // O CONTROLE: em pixels, no zoom de trabalho, um pixel de tela é um pixel de
    // display ⇒ zero casas, como sempre.
    assert_eq!(px.text_at_zoom(1.5, 100.0), "150");
    // E a precisão SEGUE o zoom, nos dois sentidos: afastar a câmera tira casas,
    // aproximar acrescenta. Um readout que não degrada ao afastar promete uma
    // exatidão que a tela não tem.
    assert_eq!(
        m.text_at_zoom(1.5, 1.0),
        "2",
        "1 px vale 1 m: não há cm a ver"
    );
    assert_eq!(m.text_at_zoom(1.5, 10.0), "1.5", "1 px vale 10 cm");
    assert_eq!(m.text_at_zoom(1.5, 100.0), "1.50", "1 px vale 1 cm");
    assert_eq!(m.text_at_zoom(1.5, 1000.0), "1.500", "1 px vale 1 mm");
}

/// **Um zoom degenerado ainda imprime um número.**
///
/// O mesmo fallback do [`crate::ruler::label_step`], e pelo mesmo motivo: sem
/// escala não há resolução a afirmar. ⚠️ Ele importa porque a câmera passa por
/// zoom zero durante um reset, e um `1/0` viraria `inf` casas — um `panic` no
/// `format!` a partir de um estado transitório.
#[test]
fn a_degenerate_zoom_still_prints_a_number() {
    let d = shipping();
    for bad in [0.0_f64, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(crate::length::world_per_pixel(bad), 1.0, "zoom {bad}");
        assert_eq!(d.text_at_zoom(1.5, bad), "150");
    }
}

/// **A conversão é feita em `f64`.**
///
/// Uma coordenada de régua longe da origem, em pixels, passa da resolução do
/// `f32`: `1e6 m × 100 = 1e8`, e o `f32` só carrega ~7 dígitos, então o rótulo
/// imprimiria um número redondo que não é o do traço.
///
/// Mutação que tem de sangrar: `from_meters_f64` delegar à versão `f32`.
#[test]
fn a_far_coordinate_survives_the_conversion() {
    let d = shipping();
    assert_eq!(d.text(1_000_000.0, 1.0), "100000000");
    assert_eq!(d.text(1_000_001.0, 1.0), "100000100");
}

/// **As duas larguras são a MESMA regra** — a estreita delega.
#[test]
fn the_narrow_conversion_agrees_with_the_wide_one() {
    for unit in [DisplayUnit::Meters, DisplayUnit::Pixels] {
        for m in [0.0_f32, 1.0, -2.5, 37.75] {
            let narrow = unit.from_meters(m, DEFAULT_PIXELS_PER_METER);
            let wide = unit.from_meters_f64(f64::from(m), DEFAULT_PIXELS_PER_METER);
            assert!(
                (f64::from(narrow) - wide).abs() < 1e-6,
                "{unit:?} {m}: {narrow} contra {wide}"
            );
        }
    }
}

/// **O default da régua é o default do projeto** — para que uma fixture que não
/// fala de unidade meça o que o app mede ao abrir.
#[test]
fn the_default_ruler_is_the_projects_ruler() {
    assert_eq!(LengthDisplay::default(), shipping());
}

/// **A porta tem DOIS sentidos, e eles fecham** — `to_world(value(w)) == w`.
///
/// É a lei que impede o defeito mais caro que um campo numérico pode ter: MOSTRAR por uma porta
/// e LER por outra. O artista digitaria de volta o mesmo `150` que o campo lhe mostrou e a forma
/// saltaria cem vezes de tamanho — e nada no compilador diria uma palavra, porque os dois lados
/// são `f64`.
///
/// Mutação que tem de sangrar: `to_world` devolver o valor cru.
#[test]
fn the_door_closes_in_both_directions() {
    for d in [
        shipping(),
        LengthDisplay {
            unit: DisplayUnit::Meters,
            pixels_per_meter: DEFAULT_PIXELS_PER_METER,
        },
        LengthDisplay {
            unit: DisplayUnit::Pixels,
            pixels_per_meter: 37.5,
        },
    ] {
        for world in [0.0_f64, 0.16, 1.5, -2.75, 4096.0] {
            let round = d.to_world(d.value(world));
            assert!(
                (round - world).abs() < 1e-9,
                "{d:?} @ {world}: voltou {round}"
            );
        }
    }
}

/// **Uma escala degenerada não devolve `inf` ao documento.**
///
/// A UI tem piso (`MIN_PIXELS_PER_METER`), mas um arquivo de outra máquina não é obrigado a
/// honrá-lo — e `x / 0.0` é `inf`, que envenena um `Transform` inteiro em silêncio.
#[test]
fn a_degenerate_scale_does_not_poison_the_document() {
    let d = LengthDisplay {
        unit: DisplayUnit::Pixels,
        pixels_per_meter: 0.0,
    };
    assert!(d.to_world(150.0).is_finite());
}
