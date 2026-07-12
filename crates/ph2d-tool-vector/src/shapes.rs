//! O catálogo de **apresentação** das formas — o que o painel precisa saber para montar
//! a UI de qualquer forma sem uma linha de código por forma.
//!
//! Par do `ph2d_vec_scene::kind` (que sabe DESENHAR): aqui mora o rótulo de cada forma,
//! a família a que ela pertence, e a descrição de cada parâmetro (rótulo, faixa, passo,
//! **unidade**). O painel itera isto e desenha os campos; a tool itera isto e clampa os
//! valores. Adicionar uma forma = uma linha de tabela aqui + um braço no `cook` lá.
//!
//! **Os defaults NÃO estão aqui** — vêm de `ShapeKind::defaults()`, para não haver duas
//! verdades sobre quanto vale um raio.
//!
//! ## A fronteira de unidade
//!
//! O documento guarda tudo em **mundo** (a geometria é mundo). A UI, porém, fala a
//! unidade em que o usuário pensa — e para um raio de canto isso é **pixel** (a unidade
//! de mundo é pequena: a viewport inteira tem ~10 unidades, então um raio útil seria
//! `0.3`, ilegível numa caixa). [`FieldUnit`] marca quais campos cruzam essa fronteira,
//! e [`to_world`] / [`to_ui`] fazem a travessia — um lugar só, em vez de espalhado.

use ph2d_vec_scene::{ShapeKind, ShapeValues};

/// A unidade em que um parâmetro é AUTORADO (o que o usuário vê e digita).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FieldUnit {
    /// Contagem inteira (lados, pontas, voltas). O campo arredonda.
    Count,
    /// Fração `0..1` (razão interna da estrela, …).
    Ratio,
    /// Ângulo em graus.
    Degrees,
    /// **Pixels de tela** — cruza a fronteira: o documento guarda `px × px_to_world`.
    Px,
}

/// Um parâmetro de forma, do ponto de vista da UI.
#[derive(Copy, Clone, Debug)]
pub struct FieldDesc {
    /// Rótulo do campo no painel (inglês — é UI).
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    /// Passo do arrasto / das setas.
    pub step: f64,
    pub unit: FieldUnit,
}

/// A família da forma — agrupa o seletor do painel (com 25+ formas, um grid plano
/// seria ilegível).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShapeGroup {
    Basic,
}

impl ShapeGroup {
    /// Rótulo da família no seletor.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ShapeGroup::Basic => "Basic",
        }
    }
}

/// Todas as famílias, na ordem do seletor.
pub const ALL_GROUPS: &[ShapeGroup] = &[ShapeGroup::Basic];

/// A forma, do ponto de vista da UI: rótulo, família e os parâmetros dela.
#[derive(Copy, Clone, Debug)]
pub struct ShapeDesc {
    pub kind: ShapeKind,
    pub label: &'static str,
    pub group: ShapeGroup,
    pub fields: &'static [FieldDesc],
}

const SIDES: FieldDesc = FieldDesc {
    label: "Sides",
    min: 3.0,
    max: 60.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
const POINTS: FieldDesc = FieldDesc {
    label: "Points",
    min: 3.0,
    max: 60.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
const INNER_RATIO: FieldDesc = FieldDesc {
    label: "Inner",
    min: 0.05,
    max: 0.95,
    step: 0.01,
    unit: FieldUnit::Ratio,
};
const TURNS: FieldDesc = FieldDesc {
    label: "Turns",
    min: 1.0,
    max: 8.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
const DEGREES: FieldDesc = FieldDesc {
    label: "Degrees",
    min: 1.0,
    max: 360.0,
    step: 1.0,
    unit: FieldUnit::Degrees,
};
/// Raio de canto (px). A faixa vai a 500 porque o teto antigo (40) não alcançava formas
/// grandes; o arredondamento satura na geometria, então pedir demais achata, nunca inverte.
const fn radius(label: &'static str) -> FieldDesc {
    FieldDesc {
        label,
        min: 0.0,
        max: 500.0,
        step: 1.0,
        unit: FieldUnit::Px,
    }
}

/// **O catálogo.** Uma linha por forma; a ordem dos `fields` é a ordem dos valores.
pub const SHAPES: &[ShapeDesc] = &[
    ShapeDesc {
        kind: ShapeKind::Rectangle,
        label: "Rect",
        group: ShapeGroup::Basic,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::RoundRect,
        label: "Round",
        group: ShapeGroup::Basic,
        fields: &[radius("Radius")],
    },
    ShapeDesc {
        kind: ShapeKind::Ellipse,
        label: "Oval",
        group: ShapeGroup::Basic,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::Polygon,
        label: "Poly",
        group: ShapeGroup::Basic,
        fields: &[SIDES, radius("Radius")],
    },
    ShapeDesc {
        kind: ShapeKind::Star,
        label: "Star",
        group: ShapeGroup::Basic,
        fields: &[
            POINTS,
            INNER_RATIO,
            radius("Tip round"),
            radius("Notch round"),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Spiral,
        label: "Spiral",
        group: ShapeGroup::Basic,
        fields: &[TURNS],
    },
    ShapeDesc {
        kind: ShapeKind::Line,
        label: "Line",
        group: ShapeGroup::Basic,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::Arc,
        label: "Arc",
        group: ShapeGroup::Basic,
        fields: &[DEGREES],
    },
];

/// O descritor de `kind` (todo `ShapeKind` tem um — o gate abaixo garante).
#[must_use]
pub fn desc(kind: ShapeKind) -> &'static ShapeDesc {
    SHAPES.iter().find(|d| d.kind == kind).unwrap_or(&SHAPES[0])
}

/// As formas de uma família, na ordem do catálogo.
pub fn shapes_in(group: ShapeGroup) -> impl Iterator<Item = &'static ShapeDesc> {
    SHAPES.iter().filter(move |d| d.group == group)
}

/// Valores autorados (UI) → valores de DOCUMENTO (mundo). Só os campos `Px` viajam;
/// contagens, razões e ângulos são os mesmos dos dois lados.
#[must_use]
pub fn to_world(kind: ShapeKind, ui: &ShapeValues, px_to_world: f64) -> ShapeValues {
    let mut out = *ui;
    for (i, f) in desc(kind).fields.iter().enumerate() {
        if f.unit == FieldUnit::Px {
            out[i] = ui[i] * px_to_world;
        }
    }
    out
}

/// Valores de DOCUMENTO (mundo) → valores autorados (UI) — o inverso exato de
/// [`to_world`]. `px_to_world` degenerado devolve os campos `Px` zerados (em vez de
/// infinito).
#[must_use]
pub fn to_ui(kind: ShapeKind, world: &ShapeValues, px_to_world: f64) -> ShapeValues {
    let mut out = *world;
    for (i, f) in desc(kind).fields.iter().enumerate() {
        if f.unit == FieldUnit::Px {
            out[i] = if px_to_world > 0.0 {
                world[i] / px_to_world
            } else {
                0.0
            };
        }
    }
    out
}

/// Clampa cada campo à faixa dele (e arredonda as contagens). Aplicado a toda autoria,
/// então nem digitação nem save corrompido produzem forma inválida.
pub fn clamp(kind: ShapeKind, v: &mut ShapeValues) {
    for (i, f) in desc(kind).fields.iter().enumerate() {
        let mut x = v[i].clamp(f.min, f.max);
        if f.unit == FieldUnit::Count {
            x = x.round();
        }
        v[i] = x;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::{ALL_SHAPES, MAX_SHAPE_FIELDS};

    /// **Gate anti-forma-sem-UI:** toda forma que o `cook` desenha tem descritor aqui —
    /// senão ela seria desenhável e invisível no painel (ou vice-versa: um descritor sem
    /// forma). E nenhuma passa do teto de campos.
    #[test]
    fn every_cookable_shape_has_a_ui_descriptor_and_fits_the_field_cap() {
        for &k in ALL_SHAPES {
            let d = SHAPES.iter().find(|d| d.kind == k);
            assert!(d.is_some(), "{k:?} cozinha mas não tem descritor de UI");
            let d = d.unwrap();
            assert!(
                d.fields.len() <= MAX_SHAPE_FIELDS,
                "{k:?}: {} campos > teto {MAX_SHAPE_FIELDS}",
                d.fields.len()
            );
        }
        assert_eq!(
            SHAPES.len(),
            ALL_SHAPES.len(),
            "descritor órfão no catálogo"
        );
    }

    /// O default de toda forma CABE na faixa declarada dos campos dela — senão a forma
    /// nasceria já clampada, e o número do painel discordaria da geometria no 1º frame.
    #[test]
    fn the_geometry_defaults_sit_inside_the_ui_ranges() {
        for &k in ALL_SHAPES {
            let d = desc(k);
            let defs = k.defaults();
            for (i, f) in d.fields.iter().enumerate() {
                // O raio de canto é o caso especial: o default é MUNDO, a faixa é PX.
                if f.unit == FieldUnit::Px {
                    continue;
                }
                assert!(
                    defs[i] >= f.min && defs[i] <= f.max,
                    "{k:?}.{}: default {} fora de [{}, {}]",
                    f.label,
                    defs[i],
                    f.min,
                    f.max
                );
            }
        }
    }

    /// A fronteira de unidade fecha: px → mundo → px devolve o mesmo número. É o que
    /// impede o raio de saltar de escala a cada clique.
    #[test]
    fn the_px_fields_round_trip_across_the_unit_boundary() {
        const PTW: f64 = 0.01;
        let mut ui: ShapeValues = [0.0; MAX_SHAPE_FIELDS];
        ui[0] = 5.0; // Sides (Count — não viaja)
        ui[1] = 30.0; // Radius (Px — viaja)
        let world = to_world(ShapeKind::Polygon, &ui, PTW);
        assert!((world[0] - 5.0).abs() < 1e-9, "contagem não vira mundo");
        assert!((world[1] - 0.3).abs() < 1e-9, "30 px x 0.01 = 0.3 de mundo");
        let back = to_ui(ShapeKind::Polygon, &world, PTW);
        assert!((back[1] - 30.0).abs() < 1e-9, "voltou a 30 px");
    }

    /// O clamp respeita a faixa e arredonda as contagens (um polígono de 4.7 lados não
    /// existe).
    #[test]
    fn clamp_bounds_the_fields_and_rounds_the_counts() {
        let mut v: ShapeValues = [0.0; MAX_SHAPE_FIELDS];
        v[0] = 4.7; // Sides
        v[1] = 9_999.0; // Radius (px)
        clamp(ShapeKind::Polygon, &mut v);
        assert!((v[0] - 5.0).abs() < 1e-9, "lados arredondam");
        assert!((v[1] - 500.0).abs() < 1e-9, "raio clampa no teto");
    }
}
