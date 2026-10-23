//! **Os roteiros das cenas 3, 4 e 5** — cada `STEPS*[k]` é o passo `k + 1`. Sairam do `smoke_drive.rs`
//! por tecto de LOC (fecho da Onda 2, 20/10); a porta que os conduz continua lá.

use super::{Gesture, Until};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

pub(super) const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};
pub(super) const ALT: Modifiers = Modifiers { alt: true, ..NONE };
pub(super) const CTRL: Modifiers = Modifiers { ctrl: true, ..NONE };
pub(super) const CTRL_SHIFT: Modifiers = Modifiers {
    shift: true,
    ..CTRL
};
const SAVE: Gesture = Gesture::Key(Key::Char('s'), CTRL);
const TAB: Gesture = Gesture::Key(Key::Tab, NONE);

/// ⭐ O roteiro: `STEPS[k]` é o passo `k + 1` do smoke do dono, gesto a gesto.
pub const STEPS: &[&[Gesture]] = &[
    // 1 — o cursor no «ampiltude» sublinhado, Alt+Enter.
    &[
        Gesture::Wait(Until::Problem("ampiltude")),
        Gesture::Go {
            needle: "self.ampiltude",
            ahead: 8,
        },
        Gesture::Key(Key::Enter, ALT),
        Gesture::Wait(Until::Actions),
    ],
    // 2 — Enter aplica, Ctrl+S: as formas mexem-se.
    &[
        Gesture::Key(Key::Enter, NONE),
        SAVE,
        Gesture::Wait(Until::Saved),
    ],
    // 3 — uma linha nova depois do «2)», `ph2d.`: a lista abre com a API.
    &[
        Gesture::Go {
            needle: "-- 2) escreva aqui",
            ahead: 0,
        },
        Gesture::Key(Key::End, NONE),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Type("ph2d."),
        Gesture::Wait(Until::ListHas("set")),
    ],
    // 4 — `se` deixa `set` em cima; Enter escolhe: o esqueleto com buracos.
    &[
        Gesture::Type("se"),
        Gesture::Wait(Until::ListTop("set")),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Wait(Until::Snippet),
    ],
    // 5 — os três buracos, Tab entre eles, Ctrl+S: as formas giram.
    &[
        Gesture::Type("self.id"),
        Gesture::Key(Key::Tab, NONE),
        Gesture::Type("rotation"),
        Gesture::Key(Key::Tab, NONE),
        Gesture::Type("self.t"),
        Gesture::Key(Key::Tab, NONE),
        SAVE,
        Gesture::Wait(Until::Saved),
    ],
    // 6 — F2 sobre o nome da propriedade: a caixinha diz quantos objectos guardam um valor.
    &[
        Gesture::Go {
            needle: "\"amplitude\"",
            ahead: 3,
        },
        Gesture::Key(Key::F(2), NONE),
        Gesture::Wait(Until::RenameBox),
    ],
    // 7 — o nome novo, Enter: o script e o valor da estrela mudam juntos.
    &[
        Gesture::Type("height"),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Wait(Until::Saved),
    ],
];

/// ⭐ O roteiro da cena 4 (o acabamento): `STEPS_4[k]` é o passo `k + 1`.
pub const STEPS_4: &[&[Gesture]] = &[
    // 1 — Ctrl com o rato sobre o `set` do `ph2d.set`: o nome sublinha na cor do link.
    &[
        Gesture::Wait(Until::Problems(3)),
        Gesture::Point {
            needle: "ph2d.set(self.id, \"y\"",
            ahead: 6,
        },
        Gesture::Ctrl(true),
        Gesture::Wait(Until::Reach(true)),
    ],
    // 2 — o controlo: o rato sobre `ampiltude` (um campo que não existe) não sublinha.
    &[
        Gesture::Point {
            needle: "self.ampiltude",
            ahead: 8,
        },
        Gesture::Wait(Until::Reach(false)),
    ],
    // 3 — o Ctrl largado, o rato num comentário; Ctrl+Shift+M: a lista, o erro primeiro.
    &[
        Gesture::Ctrl(false),
        Gesture::Point {
            needle: "-- 2) Ctrl+Shift+M",
            ahead: 3,
        },
        Gesture::Key(Key::Char('m'), CTRL_SHIFT),
        Gesture::Wait(Until::ProblemRows),
    ],
    // 4 — o clique na linha do erro: o culpado seleccionado no texto.
    &[Gesture::ClickProblem(0)],
    // 5 — F8: o problema seguinte da lista (o 1.º aviso, mais acima no texto).
    &[Gesture::Key(Key::F(8), NONE)],
    // 6 — `ph2d.set` numa linha nova e, dentro do `entity`, outra vez: os buracos fundem-se; dois
    //     Tabs andam pelos de dentro.
    &[
        Gesture::Go {
            needle: "-- 3) escreva aqui",
            ahead: 0,
        },
        Gesture::Key(Key::End, NONE),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Type("ph2d."),
        Gesture::Wait(Until::ListHas("set")),
        Gesture::Type("se"),
        Gesture::Wait(Until::ListTop("set")),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Wait(Until::Snippet),
        Gesture::Type("ph2d."),
        Gesture::Wait(Until::ListHas("set")),
        Gesture::Type("se"),
        Gesture::Wait(Until::ListTop("set")),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Wait(Until::Has("ph2d.set(ph2d.set(entity")),
        Gesture::Frame,
        TAB,
        Gesture::Frame,
        TAB,
    ],
    // 7 — os Tabs que faltam: o fim da chamada de dentro, os de fora, e o fim.
    &[
        TAB,
        Gesture::Frame,
        TAB,
        Gesture::Frame,
        TAB,
        Gesture::Frame,
        TAB,
    ],
];

/// ⭐ O roteiro da cena 5 (a página de ajuda): `STEPS_5[k]` é o passo `k + 1`. O histórico é o do
/// Godot 4.7.2 (`crate::nav`): o link na MESMA página só rola, logo o Alt+← do passo 4 volta ao
/// SCRIPT (no cursor do Ctrl+clique), não ao `sin`.
pub const STEPS_5: &[&[Gesture]] = &[
    // 1 — Ctrl com o rato sobre o `sin` do `math.sin`: o nome sublinha (e o ponteiro é a mão). Só
    //     depois de o assistente verificar o script: na shell, o Ctrl a ~0,4 s do arranque deixou a
    //     janela parada nas fotos (as da cena 4 esperavam pelos três problemas).
    &[
        Gesture::Wait(Until::Script),
        Gesture::Wait(Until::Checked),
        Gesture::Point {
            needle: "math.sin(",
            ahead: 6,
        },
        Gesture::Ctrl(true),
        Gesture::Wait(Until::Reach(true)),
    ],
    // 2 — o clique com o Ctrl: a página `math` numa aba, rolada ao `sin`; o Ctrl largado.
    &[
        Gesture::Click {
            needle: "math.sin(",
            ahead: 6,
            mods: CTRL,
        },
        Gesture::Ctrl(false),
        Gesture::Wait(Until::Page {
            page: "math",
            member: "sin",
        }),
    ],
    // 3 — o link `math.noise` no texto do `sin`: a MESMA aba rola até ele.
    &[
        Gesture::Link {
            page: "math",
            to: "math",
            member: "noise",
        },
        Gesture::Wait(Until::Page {
            page: "math",
            member: "noise",
        }),
    ],
    // 4 — Alt+←: o link na mesma página não entrou no histórico, logo volta ao SCRIPT.
    &[
        Gesture::TabKey(Key::Left, ALT),
        Gesture::Wait(Until::Script),
    ],
    // 5 — o cursor no `set` do `ph2d.set`, F12: a página `ph2d` no `set`.
    &[
        Gesture::Go {
            needle: "ph2d.set(self.id",
            ahead: 6,
        },
        Gesture::Key(Key::F(12), NONE),
        Gesture::Wait(Until::Page {
            page: "ph2d",
            member: "set",
        }),
    ],
    // 6 — o `PropertyName` da assinatura do `set`: a página `types` (empilha).
    &[
        Gesture::Link {
            page: "ph2d",
            to: "types",
            member: "PropertyName",
        },
        Gesture::Wait(Until::Page {
            page: "types",
            member: "PropertyName",
        }),
    ],
    // 7 — Alt+←: de volta à `ph2d`, no `set`.
    &[
        Gesture::TabKey(Key::Left, ALT),
        Gesture::Wait(Until::Page {
            page: "ph2d",
            member: "set",
        }),
    ],
    // 8 — Ctrl+F e `entity`: a barra da busca conta.
    &[
        Gesture::TabKey(Key::Char('f'), CTRL),
        Gesture::TabType("entity"),
        Gesture::Wait(Until::Found),
    ],
    // 9 — Ctrl+W: a página fecha e o script volta.
    &[
        Gesture::TabKey(Key::Char('w'), CTRL),
        Gesture::Wait(Until::Script),
    ],
    // 10 — o controlo: Ctrl sobre `local` (palavra da linguagem) não sublinha nem dá a mão.
    &[
        Gesture::Point {
            needle: "local m = math",
            ahead: 2,
        },
        Gesture::Ctrl(true),
        Gesture::Frame,
        Gesture::Wait(Until::Plain {
            needle: "local m = math",
            ahead: 2,
        }),
    ],
];
