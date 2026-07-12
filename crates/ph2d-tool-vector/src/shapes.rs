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

use ph2d_vec_scene::ShapeKind;

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
    /// **Escolha** entre N opções nomeadas (o valor é o índice, `0..N−1`).
    ///
    /// O painel pinta um botão que CICLA pelos nomes — não uma caixa de número. "Ponto de
    /// vista: 0" não é uma UI; "Viewed: From above" é. O valor continua um `f64` no
    /// documento (o vetor de parâmetros é de tamanho fixo e homogêneo), mas a UI nunca o
    /// mostra cru.
    Choice(&'static [&'static str]),
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
    /// A família do círculo (pizza, rosquinha, segmento) — a elipse fica em `Basic`
    /// porque é uma primitiva, mas os atalhos dela vivem aqui.
    Round,
    /// Setas em BLOCO (formas preenchíveis). As pontas de traço são outra coisa.
    Arrows,
    /// O vocabulário de FLUXOGRAMA (ANSI/ISO): cada silhueta tem um significado fixo.
    Flow,
    /// Balões de fala / pensamento / grito + a chave de anotação.
    Bubbles,
    /// Símbolos (coração, nuvem, raio, escudo, engrenagem, cruz…).
    Symbols,
    /// Isométricas 2.5D (cubo, cone, pirâmide) — polígonos em perspectiva, não 3D.
    Iso,
}

impl ShapeGroup {
    /// Rótulo da família no seletor.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ShapeGroup::Basic => "Basic",
            ShapeGroup::Round => "Round",
            ShapeGroup::Arrows => "Arrows",
            ShapeGroup::Flow => "Flow",
            ShapeGroup::Bubbles => "Bubbles",
            ShapeGroup::Symbols => "Symbols",
            ShapeGroup::Iso => "3D",
        }
    }
}

/// Todas as famílias, na ordem do seletor.
pub const ALL_GROUPS: &[ShapeGroup] = &[
    ShapeGroup::Basic,
    ShapeGroup::Round,
    ShapeGroup::Arrows,
    ShapeGroup::Flow,
    ShapeGroup::Bubbles,
    ShapeGroup::Symbols,
    ShapeGroup::Iso,
];

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
/// Quanto o corte ABRE (graus). É o parâmetro que leva a elipse a pizza / anel parcial.
const SWEEP: FieldDesc = FieldDesc {
    label: "Sweep",
    min: 1.0,
    max: 360.0,
    step: 1.0,
    unit: FieldUnit::Degrees,
};
/// ONDE o corte começa (graus, 0 = 3h) — gira a fatia sem girar a caixa. O teto é 359:
/// 360° é o MESMO ângulo que 0°, e um campo cujos extremos dão a mesma forma é um campo
/// que não faz nada (o gate do catálogo pega isso).
const START: FieldDesc = FieldDesc {
    label: "Start",
    min: 0.0,
    max: 359.0,
    step: 1.0,
    unit: FieldUnit::Degrees,
};
/// Quantas bolhas tem a nuvem. A nuvem do ECMA-376 tem 11 arcos — daí o default; abaixo de
/// 5 ela deixa de ler como nuvem. Compartilhada pela nuvem e pelo balão de pensamento (que
/// é a mesma nuvem + a corrente de bolhas).
const BUMPS: FieldDesc = FieldDesc {
    label: "Bumps",
    min: 5.0,
    max: 24.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
/// Quanto do miolo é FURO (fração do raio). `0` = cheio; `>0` = rosquinha / anel.
const HOLE: FieldDesc = FieldDesc {
    label: "Hole",
    min: 0.0,
    max: 0.95,
    step: 0.01,
    unit: FieldUnit::Ratio,
};
/// Uma fração da caixa (`0..1`) — as medidas das setas são relativas, não absolutas.
const fn frac(label: &'static str, min: f64, max: f64) -> FieldDesc {
    FieldDesc {
        label,
        min,
        max,
        step: 0.01,
        unit: FieldUnit::Ratio,
    }
}

/// **O ponto de vista** de um sólido isométrico — e ele MUDA a geometria, não é cosmético.
///
/// A câmera acima ou abaixo do plano da base decide **quais arestas existem**:
///
/// - o cone visto de CIMA tem o corpo tapando a borda de trás da base (ela some); visto de
///   BAIXO, vê-se a face inferior do disco, que fica *na frente* do cone — e a borda de trás
///   aparece cruzando o corpo;
/// - o cubo é a **ambiguidade de Necker**: o mesmo hexágono lê dos dois jeitos, e o que
///   decide é para onde apontam as três arestas internas — para o vértice `(skew, rise)` ou
///   para o OPOSTO, `(1 − skew, 1 − rise)`. (Nos defaults `0,5/0,5` os dois coincidem no
///   centro; é exatamente por isso que um cubo em traço puro é ambíguo.)
const VIEWED: FieldDesc = FieldDesc {
    label: "Viewed",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    unit: FieldUnit::Choice(&["From above", "From below"]),
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

/// **Desvio** do raio de um canto do round-rect (px, COM SINAL) — `0` = o canto segue o raio
/// base. Não é raio absoluto de propósito: o vetor de parâmetros é posicional e campo ausente
/// lê ZERO, então um campo apendado só é seguro se zero for o seu NEUTRO — e o neutro de "o
/// raio deste canto" é "o mesmo dos outros", não "canto vivo" (senão todo save anterior
/// passaria a desenhar três cantos vivos). Ver `ph2d_vec_scene::round_rect_radii`. A faixa
/// espelha a do raio, então qualquer quádrupla de cantos é alcançável.
const fn corner_offset(label: &'static str) -> FieldDesc {
    FieldDesc {
        label,
        min: -500.0,
        max: 500.0,
        step: 1.0,
        unit: FieldUnit::Px,
    }
}

/// **Suavização de canto** (o *corner smoothing* do Figma / o canto contínuo da Apple): `0` =
/// o arco de círculo de sempre (a curvatura SALTA da reta para o arco); `1` = o squircle
/// cheio (o arco some e duas asas de Bézier fazem a curvatura subir em rampa). É neutro em
/// zero — obrigatório, para um campo apendado.
const SMOOTHING: FieldDesc = FieldDesc {
    label: "Smoothing",
    min: 0.0,
    max: 1.0,
    step: 0.01,
    unit: FieldUnit::Ratio,
};

/// **O catálogo.** Uma linha por forma; a ordem dos `fields` é a ordem dos valores.
pub const SHAPES: &[ShapeDesc] = &[
    ShapeDesc {
        kind: ShapeKind::Rectangle,
        label: "Rect",
        group: ShapeGroup::Basic,
        fields: &[],
    },
    // O raio é o MESTRE (move os quatro cantos); cada canto pode se afastar dele por um
    // desvio, e `Smoothing` troca o arco de círculo pelo squircle. Os quatro campos foram
    // APENDADOS depois do raio, e todos são neutros em zero — o round-rect de um save antigo
    // (só `Radius`) cozinha exatamente como cozinhava.
    ShapeDesc {
        kind: ShapeKind::RoundRect,
        label: "Round",
        group: ShapeGroup::Basic,
        fields: &[
            radius("Radius"),
            corner_offset("TR offset"),
            corner_offset("BR offset"),
            corner_offset("BL offset"),
            SMOOTHING,
        ],
    },
    // A elipse carrega a FAMÍLIA inteira: fechar o `Sweep` faz a pizza, abrir o `Hole`
    // faz a rosquinha, e os dois juntos fazem o anel parcial (ver `ph2d_vec_scene::round`).
    ShapeDesc {
        kind: ShapeKind::Ellipse,
        label: "Oval",
        group: ShapeGroup::Basic,
        fields: &[SWEEP, START, HOLE],
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
        fields: &[SWEEP, START],
    },
    // Atalhos da mesma família (defaults já abertos) — quem quer a fatia não devia ter
    // de descobrir que ela mora dentro da elipse.
    ShapeDesc {
        kind: ShapeKind::Pie,
        label: "Pie",
        group: ShapeGroup::Round,
        fields: &[SWEEP, START, HOLE],
    },
    ShapeDesc {
        kind: ShapeKind::Segment,
        label: "Segment",
        group: ShapeGroup::Round,
        fields: &[SWEEP, START],
    },
    // ── Setas ────────────────────────────────────────────────────────────────
    ShapeDesc {
        kind: ShapeKind::ArrowRight,
        label: "Arrow",
        group: ShapeGroup::Arrows,
        fields: &[
            frac("Tail", 0.02, 1.0),
            frac("Head len", 0.02, 1.0),
            frac("Head width", 0.05, 1.0),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::ArrowDouble,
        label: "Double",
        group: ShapeGroup::Arrows,
        fields: &[
            frac("Tail", 0.02, 1.0),
            frac("Head len", 0.02, 0.5),
            frac("Head width", 0.05, 1.0),
        ],
    },
    // `Corner` é o raio EXTERNO do cotovelo; o interno acompanha (arcos concêntricos, que
    // é o que mantém a espessura constante na curva). `0` = a quina quadrada do OOXML.
    ShapeDesc {
        kind: ShapeKind::ArrowBent,
        label: "Bent",
        group: ShapeGroup::Arrows,
        fields: &[
            frac("Tail", 0.02, 0.9),
            frac("Head len", 0.05, 0.6),
            frac("Head width", 0.05, 1.0),
            frac("Corner", 0.0, 1.0),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Chevron,
        label: "Chevron",
        group: ShapeGroup::Arrows,
        fields: &[frac("Point", 0.02, 0.9), frac("Notch", 0.0, 0.9)],
    },
    // ── Fluxograma ───────────────────────────────────────────────────────────
    ShapeDesc {
        kind: ShapeKind::Diamond,
        label: "Decision",
        group: ShapeGroup::Flow,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::Pill,
        label: "Terminal",
        group: ShapeGroup::Flow,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::Parallelogram,
        label: "Data",
        group: ShapeGroup::Flow,
        fields: &[frac("Slant", 0.0, 0.9)],
    },
    ShapeDesc {
        kind: ShapeKind::Trapezoid,
        label: "Manual in",
        group: ShapeGroup::Flow,
        fields: &[frac("Slant", 0.0, 0.49)],
    },
    ShapeDesc {
        kind: ShapeKind::TrapezoidFlip,
        label: "Manual op",
        group: ShapeGroup::Flow,
        fields: &[frac("Slant", 0.0, 0.49)],
    },
    ShapeDesc {
        kind: ShapeKind::HexagonFlat,
        label: "Prepare",
        group: ShapeGroup::Flow,
        fields: &[frac("Cut", 0.02, 0.49)],
    },
    ShapeDesc {
        kind: ShapeKind::Cylinder,
        label: "Database",
        group: ShapeGroup::Flow,
        fields: &[frac("Lip", 0.05, 0.9)],
    },
    ShapeDesc {
        kind: ShapeKind::Document,
        label: "Document",
        group: ShapeGroup::Flow,
        fields: &[frac("Wave", 0.02, 0.5)],
    },
    ShapeDesc {
        kind: ShapeKind::Delay,
        label: "Delay",
        group: ShapeGroup::Flow,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::Display,
        label: "Display",
        group: ShapeGroup::Flow,
        fields: &[frac("Nose", 0.02, 0.45)],
    },
    ShapeDesc {
        kind: ShapeKind::PredefinedProcess,
        label: "Subroutine",
        group: ShapeGroup::Flow,
        fields: &[frac("Bars", 0.02, 0.4)],
    },
    ShapeDesc {
        kind: ShapeKind::OffPage,
        label: "Off-page",
        group: ShapeGroup::Flow,
        fields: &[frac("Tip", 0.05, 0.9)],
    },
    ShapeDesc {
        kind: ShapeKind::Junction,
        label: "Junction",
        group: ShapeGroup::Flow,
        fields: &[],
    },
    ShapeDesc {
        kind: ShapeKind::NoteBracket,
        label: "Note",
        group: ShapeGroup::Flow,
        fields: &[frac("Lip", 0.05, 1.0)],
    },
    // ── Balões + símbolos ────────────────────────────────────────────────────
    // O BICO dos balões é um ponto 2D livre na caixa (`Tip x`/`Tip y`) — o setor dele
    // escolhe por qual lado o rabo sai. `Room` é a faixa que o corpo cede ao rabo: ele vive
    // dentro dela, então nunca vaza a caixa (e o gizmo não mente).
    ShapeDesc {
        kind: ShapeKind::SpeechRect,
        label: "Speech",
        group: ShapeGroup::Bubbles,
        fields: &[
            radius("Radius"),
            frac("Tip x", 0.0, 1.0),
            frac("Tip y", 0.0, 1.0),
            frac("Base", 0.05, 0.6),
            frac("Room", 0.05, 0.6),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::SpeechOval,
        label: "Oval say",
        group: ShapeGroup::Bubbles,
        fields: &[
            frac("Tip x", 0.0, 1.0),
            frac("Tip y", 0.0, 1.0),
            FieldDesc {
                label: "Wedge",
                min: 2.0,
                max: 45.0,
                step: 1.0,
                unit: FieldUnit::Degrees,
            },
            frac("Room", 0.05, 0.6),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Thought,
        label: "Thought",
        group: ShapeGroup::Bubbles,
        fields: &[
            frac("Tip x", 0.0, 1.0),
            frac("Tip y", 0.0, 1.0),
            FieldDesc {
                label: "Bubbles",
                min: 1.0,
                max: 6.0,
                step: 1.0,
                unit: FieldUnit::Count,
            },
            BUMPS,
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Burst,
        label: "Burst",
        group: ShapeGroup::Bubbles,
        fields: &[
            FieldDesc {
                label: "Spikes",
                min: 5.0,
                max: 30.0,
                step: 1.0,
                unit: FieldUnit::Count,
            },
            frac("Inner", 0.2, 0.9),
            frac("Radial", 0.0, 0.6),
            frac("Angular", 0.0, 0.9),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Brace,
        label: "Brace",
        group: ShapeGroup::Bubbles,
        fields: &[
            frac("Arm", 0.01, 0.25),
            frac("Pinch", 0.1, 0.9),
            frac("Stem", 0.15, 0.85),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Heart,
        label: "Heart",
        group: ShapeGroup::Symbols,
        fields: &[frac("Cleft", 0.0, 0.5)],
    },
    // `Puff` é o raio da bolha em frações do passo da espinha — **abaixo de 0,5 as bolhas
    // vizinhas não se sobrepõem** e o join (a interseção círculo-círculo) deixa de existir;
    // daí o piso da faixa. `Smooth` é o raio do filete do vale: 0 = a cúspide escalopada do
    // OOXML, > 0 = um contorno G1 de verdade.
    ShapeDesc {
        kind: ShapeKind::Cloud,
        label: "Cloud",
        group: ShapeGroup::Symbols,
        fields: &[
            BUMPS,
            frac("Puff", 0.52, 0.9),
            FieldDesc {
                label: "Smooth",
                min: 0.0,
                max: 0.06,
                step: 0.005,
                unit: FieldUnit::Ratio,
            },
            frac("Irregular", 0.0, 1.5),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Bolt,
        label: "Bolt",
        group: ShapeGroup::Symbols,
        fields: &[frac("Waist", 0.05, 0.8)],
    },
    ShapeDesc {
        kind: ShapeKind::Moon,
        label: "Moon",
        group: ShapeGroup::Symbols,
        fields: &[frac("Phase", 0.05, 0.95)],
    },
    ShapeDesc {
        kind: ShapeKind::Drop,
        label: "Drop",
        group: ShapeGroup::Symbols,
        fields: &[frac("Tip", 0.1, 0.9)],
    },
    ShapeDesc {
        kind: ShapeKind::Shield,
        label: "Shield",
        group: ShapeGroup::Symbols,
        fields: &[frac("Curve", 0.0, 1.0)],
    },
    ShapeDesc {
        kind: ShapeKind::Tag,
        label: "Tag",
        group: ShapeGroup::Symbols,
        fields: &[frac("Point", 0.05, 0.6), frac("Hole", 0.0, 0.4)],
    },
    ShapeDesc {
        kind: ShapeKind::Gear,
        label: "Gear",
        group: ShapeGroup::Symbols,
        fields: &[
            FieldDesc {
                label: "Teeth",
                min: 4.0,
                max: 30.0,
                step: 1.0,
                unit: FieldUnit::Count,
            },
            frac("Depth", 0.05, 0.5),
            frac("Hole", 0.0, 0.8),
        ],
    },
    ShapeDesc {
        kind: ShapeKind::Cross,
        label: "Cross",
        group: ShapeGroup::Symbols,
        fields: &[frac("Arm", 0.05, 0.95)],
    },
    ShapeDesc {
        kind: ShapeKind::Check,
        label: "Check",
        group: ShapeGroup::Symbols,
        fields: &[frac("Weight", 0.05, 0.6)],
    },
    ShapeDesc {
        kind: ShapeKind::Banner,
        label: "Banner",
        group: ShapeGroup::Symbols,
        fields: &[frac("Notch", 0.02, 0.4)],
    },
    // ── Isométricas ──────────────────────────────────────────────────────────
    // `Rise` inclina os eixos da projeção (é a altura do vértice interno) e `Skew` reparte
    // largura e profundidade sem mexer no ângulo — os dois governam cubo e pirâmide.
    ShapeDesc {
        kind: ShapeKind::IsoCube,
        label: "Cube",
        group: ShapeGroup::Iso,
        fields: &[frac("Rise", 0.1, 0.9), frac("Skew", 0.1, 0.9), VIEWED],
    },
    ShapeDesc {
        kind: ShapeKind::IsoCone,
        label: "Cone",
        group: ShapeGroup::Iso,
        fields: &[frac("Lip", 0.05, 0.45), VIEWED],
    },
    ShapeDesc {
        kind: ShapeKind::IsoPyramid,
        label: "Pyramid",
        group: ShapeGroup::Iso,
        fields: &[frac("Rise", 0.1, 0.9), frac("Skew", 0.1, 0.9), VIEWED],
    },
];

// A travessia (descritor · unidade · clamp · escolhas) mora no módulo irmão, mas a porta é
// aqui: `shapes::to_world`, `shapes::clamp`, … seguem existindo para quem consome.
pub use crate::shapes_units::{
    choice_label, clamp, clamp_to, desc, next_choice, shapes_in, to_ui, to_world,
};

#[cfg(test)]
#[path = "shapes_tests.rs"]
mod tests;
