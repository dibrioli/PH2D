//! Os BLOCOS dos campos das formas — os `FieldDesc` partilhados (lados, pontas, ângulos, frações,
//! raios, …) com que o catálogo [`crate::shapes::SHAPES`] se escreve. Irmão de `shapes` pelo tecto
//! de 700 LOC (HR-18): o catálogo fica com a TABELA, aqui os tijolos.

use crate::shapes::{FieldDesc, FieldUnit};

pub(crate) const SIDES: FieldDesc = FieldDesc {
    label: "Sides",
    min: 3.0,
    max: 60.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
pub(crate) const POINTS: FieldDesc = FieldDesc {
    label: "Points",
    min: 3.0,
    max: 60.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
pub(crate) const INNER_RATIO: FieldDesc = FieldDesc {
    label: "Inner",
    min: 0.05,
    max: 0.95,
    step: 0.01,
    unit: FieldUnit::Ratio,
};
pub(crate) const TURNS: FieldDesc = FieldDesc {
    label: "Turns",
    min: 1.0,
    max: 8.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
/// Quanto o corte ABRE (graus). É o parâmetro que leva a elipse a pizza / anel parcial.
pub(crate) const SWEEP: FieldDesc = FieldDesc {
    label: "Sweep",
    min: 1.0,
    max: 360.0,
    step: 1.0,
    unit: FieldUnit::Degrees,
};
/// ONDE o corte começa (graus, 0 = 3h) — gira a fatia sem girar a caixa. O teto é 359:
/// 360° é o MESMO ângulo que 0°, e um campo cujos extremos dão a mesma forma é um campo
/// que não faz nada (o gate do catálogo pega isso).
pub(crate) const START: FieldDesc = FieldDesc {
    label: "Start",
    min: 0.0,
    max: 359.0,
    step: 1.0,
    unit: FieldUnit::Degrees,
};
/// Quantas bolhas tem a nuvem. A nuvem do ECMA-376 tem 11 arcos — daí o default; abaixo de
/// 5 ela deixa de ler como nuvem. Compartilhada pela nuvem e pelo balão de pensamento (que
/// é a mesma nuvem + a corrente de bolhas).
pub(crate) const BUMPS: FieldDesc = FieldDesc {
    label: "Bumps",
    min: 5.0,
    max: 24.0,
    step: 1.0,
    unit: FieldUnit::Count,
};
/// Quanto do miolo é FURO (fração do raio). `0` = cheio; `>0` = rosquinha / anel.
pub(crate) const HOLE: FieldDesc = FieldDesc {
    label: "Hole",
    min: 0.0,
    max: 0.95,
    step: 0.01,
    unit: FieldUnit::Ratio,
};
/// Uma fração da caixa (`0..1`) — as medidas das setas são relativas, não absolutas.
pub(crate) const fn frac(label: &'static str, min: f64, max: f64) -> FieldDesc {
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
pub(crate) const VIEWED: FieldDesc = FieldDesc {
    label: "Viewed",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    unit: FieldUnit::Choice(&["From above", "From below"]),
};

/// Raio de canto (px). A faixa vai a 500 porque o teto antigo (40) não alcançava formas
/// grandes; o arredondamento satura na geometria, então pedir demais achata, nunca inverte.
pub(crate) const fn radius(label: &'static str) -> FieldDesc {
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
pub(crate) const fn corner_offset(label: &'static str) -> FieldDesc {
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
pub(crate) const SMOOTHING: FieldDesc = FieldDesc {
    label: "Smoothing",
    min: 0.0,
    max: 1.0,
    step: 0.01,
    unit: FieldUnit::Ratio,
};

// ── F10 (11/10): os parâmetros que os apps de referência têm e o catálogo não tinha. Todos
// NEUTROS em zero (um save anterior lê zero e cozinha igual). Doc:
// `docs/UI_New_and_Simple/vetor_nova_ui/15_mais_parametros_das_formas.md`.

/// **O tipo do canto** — as quatro do Affinity (*Rounded · Straight · Concave · Cutout*); o índice
/// é o `ph2d_vec_scene::CornerStyle`. Arredondado = `0`, o de sempre.
pub(crate) const CORNER_STYLE: FieldDesc = FieldDesc {
    label: "Corner",
    min: 0.0,
    max: 3.0,
    step: 1.0,
    unit: FieldUnit::Choice(&["Rounded", "Straight", "Concave", "Cutout"]),
};
/// **A envergadura das arestas** (o *Curve* do polígono e o *Curved edges* da estrela do Affinity):
/// `> 0` para fora, `< 0` para dentro; `1` = o meio da aresta a meia corda dela.
pub(crate) const CURVE: FieldDesc = signed("Curve", 1.0);
/// **O liso dos vértices** (o *Rounded* do Inkscape, corrido): alças tangentes de `smooth × corda`.
/// Só para fora (F12): o meio negativo do Inkscape é um laço por definição (doc 19 §3).
pub(crate) const SMOOTH_VERTS: FieldDesc = frac("Smooth", 0.0, 1.0);
/// **A torção** (o `arg2` do Inkscape): os graus que os vales rodam em relação às pontas.
pub(crate) const TWIST: FieldDesc = FieldDesc {
    label: "Twist",
    min: -90.0,
    max: 90.0,
    step: 1.0,
    unit: FieldUnit::Degrees,
};
/// **A divergência da espiral** (o *Divergence* do Inkscape, o *Decay* do Affinity e do
/// Illustrator): `r = t^(1 + expansion)` — `0` = voltas iguais, `> 0` abrem para fora.
pub(crate) const EXPANSION: FieldDesc = FieldDesc {
    label: "Expansion",
    min: -0.9,
    max: 9.0,
    step: 0.05,
    unit: FieldUnit::Ratio,
};

/// Uma fração COM SINAL, simétrica (`-max..max`).
const fn signed(label: &'static str, max: f64) -> FieldDesc {
    frac(label, -max, max)
}
