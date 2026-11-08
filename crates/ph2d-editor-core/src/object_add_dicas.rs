//! As ilustrações das duas entradas da fundação no menu Add (Paint a new image…, Empty) — desenhos em
//! DADOS na moldura 2:1 ([`crate::widget::illustration`]). As das famílias moram em `ph2d-game-dicas`.

use crate::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

/// Uma tela em branco e o traço do pincel a começar.
pub const IMAGE: &[Mark] = &[
    Mark::Rect {
        at: [0.3, 0.1],
        size: [0.4, 0.8],
        fill: C::AccentFg,
        round: 0.03,
    },
    Mark::Line {
        a: [0.38, 0.68],
        b: [0.5, 0.42],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Line {
        a: [0.5, 0.42],
        b: [0.6, 0.56],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Disc {
        c: [0.6, 0.56],
        r: 0.06,
        fill: C::Accent,
    },
];

/// Um ponto na cena e nada mais: a cruz da origem e o contorno de onde as coisas vão morar.
pub const EMPTY: &[Mark] = &[
    Mark::Dashed {
        at: [0.36, 0.2],
        size: [0.28, 0.6],
        stroke: C::Text3,
    },
    Mark::Line {
        a: [0.5, 0.3],
        b: [0.5, 0.7],
        stroke: C::Text1,
        arrow: false,
    },
    Mark::Line {
        a: [0.42, 0.5],
        b: [0.58, 0.5],
        stroke: C::Text1,
        arrow: false,
    },
];
