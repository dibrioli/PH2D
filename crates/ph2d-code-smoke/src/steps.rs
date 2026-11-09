//! **Os roteiros das cenas 3, 4, 5 e 10** — cada `STEPS*[k]` é o passo `k + 1`. Sairam do `ph2d-app-code/src/smoke_drive.rs`
//! por tecto de LOC (fecho da Onda 2, 20/10); a porta que os conduz continua lá.

use crate::{Gesture, Until};
use ph2d_code_viewmodel::Aim;
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

pub const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};
pub const ALT: Modifiers = Modifiers { alt: true, ..NONE };
pub const CTRL: Modifiers = Modifiers { ctrl: true, ..NONE };
pub const CTRL_SHIFT: Modifiers = Modifiers {
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
    // 3 — o Ctrl largado, o rato num comentário; a lista já aberta pelo erro do Play (Onda A: um
    // erro de runtime abre a faixa nos Problems — um Ctrl+Shift+M aqui FECHÁ-LA-IA).
    &[
        Gesture::Ctrl(false),
        Gesture::Point {
            needle: "-- 2) O erro abriu",
            ahead: 3,
        },
        Gesture::Wait(Until::ProblemRows),
    ],
    // 4 — o clique na linha do erro: o culpado seleccionado no texto.
    &[Gesture::ClickProblem(0)],
    // 5 — o clique no 1.º aviso (mais acima no texto). ⚠️ Não o F8: o erro PAROU o jogo (Pause on
    // Error, Onda D, ligado de fábrica) e parado o F8 é o Continue (`commands::row_state`).
    &[Gesture::ClickProblem(1)],
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

/// ⭐ O roteiro da cena 10 (o script do Kanban): a agulha é o `ph2d_kanban_scenes::script::NEEDLE`.
pub const STEPS_10: &[&[Gesture]] = &[
    // 1 — uma linha nova depois do «1)», `card:`: a lista abre com os métodos do cartão.
    &[
        Gesture::Wait(Until::Checked),
        Gesture::Go {
            needle: "-- 1) escreva aqui",
            ahead: 0,
        },
        Gesture::Key(Key::End, NONE),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Type("card:"),
        Gesture::Wait(Until::ListHas("move")),
    ],
    // 2 — Esc fecha a lista, o `card:` sai, Ctrl+S: o script corre; o separador Console mostra-o.
    &[
        Gesture::Key(Key::Escape, NONE),
        Gesture::Key(Key::Backspace, NONE),
        Gesture::Key(Key::Backspace, NONE),
        Gesture::Key(Key::Backspace, NONE),
        Gesture::Key(Key::Backspace, NONE),
        Gesture::Key(Key::Backspace, NONE),
        SAVE,
        Gesture::Wait(Until::Saved),
        Gesture::ClickBandTab(0),
    ],
];

/// O roteiro da cena 11 (13 §4 itens 11–13b): o cursor dentro de `update`, o menu Edit aberto com o
/// rato em «Toggle Comment»; depois o cartão «Script options» pela engrenagem.
pub const STEPS_11: &[&[Gesture]] = &[
    // 1 — o cursor em `update` (o caminho diz `update`) e o menu Edit aberto.
    &[
        Gesture::Go {
            needle: "self.t = self.t + dt",
            ahead: 0,
        },
        Gesture::Frame,
        Gesture::ClickTabMenu(1),
        Gesture::Frame,
        Gesture::PointTabMenuRow(2),
    ],
    // 2 — o menu fecha (Esc) e a engrenagem abre o cartão.
    &[
        Gesture::TabKey(Key::Escape, NONE),
        Gesture::Frame,
        Gesture::ClickGear,
    ],
    // 3 — (item 15) o cartão fecha e um objecto largado sem tecla pergunta o que escrever.
    &[
        Gesture::Key(Key::Escape, NONE),
        Gesture::Frame,
        Gesture::DropObject {
            needle: "self.t * self.sped",
            name: "Door",
        },
        Gesture::Frame,
    ],
    // 4 — (item 16) Esc não escreve nada; Alt+Enter sobre o `3` oferece «Make it a property».
    &[
        Gesture::Key(Key::Escape, NONE),
        Gesture::Frame,
        Gesture::Go {
            needle: "> 3 then",
            ahead: 2,
        },
        Gesture::Key(Key::Enter, ALT),
        Gesture::Frame,
        Gesture::Frame,
    ],
    // 5 — (item 14) o botão direito sobre `self.sped`: o conserto primeiro, a 1.ª linha acesa.
    &[
        Gesture::Key(Key::Escape, NONE),
        Gesture::Frame,
        Gesture::RightClick {
            needle: "self.sped",
            ahead: 7,
        },
        Gesture::Frame,
        Gesture::TabKey(Key::Down, NONE),
        Gesture::Frame,
    ],
];

/// A linha do ponto de parada da cena 12 (a mesma agulha do `ph2d_app_components::code_smoke`).
pub const DEPURA_BREAK: &str = "local swing = ";

/// ⭐ **Cena 12 — o depurador** (Onda D): o jogo pára no ponto que a cena traz; Continue passa ao
/// objecto seguinte; tirado o ponto, os valores ao vivo aparecem.
pub const STEPS_12: &[&[Gesture]] = &[
    // 1 — parado no Circle: a seta, a faixa e o cartão «Stopped» com as variáveis.
    &[Gesture::Wait(Until::Stopped("Circle"))],
    // 2 — Continue: o mesmo ponto, agora na Star.
    &[
        Gesture::ClickDebug(0),
        Gesture::Frame,
        Gesture::Wait(Until::Stopped("Star")),
    ],
    // 3 — o ponto sai (clique na marca) e Continue: o jogo corre e os valores aparecem.
    &[
        Gesture::ClickMark {
            needle: DEPURA_BREAK,
        },
        Gesture::Frame,
        Gesture::ClickDebug(0),
        Gesture::Frame,
        Gesture::Wait(Until::Live),
    ],
];

/// ⭐ **Cena 13 — a Bancada** (Onda B): o balão «em quem» de `speed`, a célula alterada do Hero 2,
/// a célula do Hero escrita à mão (a MESMA edição do Inspector), e o espiar da cena.
pub const STEPS_13: &[&[Gesture]] = &[
    // 1 — o rato na marca ◆ de `speed` (linha 1): o balão diz Hero 220 · default, Hero 2 180 · changed.
    &[
        Gesture::Wait(Until::Script),
        Gesture::Frame,
        Gesture::PointAim(Aim::Mark(0)),
        Gesture::Frame,
    ],
    // 2 — o rato na célula do Hero 2 em `speed`: a linha e a coluna acesas, e a frase diz o que é.
    &[Gesture::PointAim(Aim::Cell(1, 0)), Gesture::Frame],
    // 3 — um clique na célula do Hero, `300` e Enter: a grelha (e o Inspector) dizem 300.
    &[
        Gesture::ClickAim(Aim::Cell(0, 0)),
        Gesture::Frame,
        Gesture::TabKey(Key::Char('a'), CTRL),
        Gesture::TabType("300"),
        Gesture::TabKey(Key::Enter, NONE),
        Gesture::Wait(Until::CellIs(0, 0, "300")),
        Gesture::PointAim(Aim::Cell(0, 0)),
        Gesture::Frame,
    ],
    // 4 — o rato parado no nome «Hero 2» da grelha: a cena aparece com ele contornado na cor dele.
    &[
        Gesture::PointAim(Aim::Holder(1)),
        Gesture::Wait(Until::Peeking),
        Gesture::Frame,
    ],
    // 5 — o rato volta à grelha e o Ctrl+Z da JANELA: o speed do Hero volta a 220.
    &[
        Gesture::PointAim(Aim::Cell(0, 0)),
        Gesture::Frame,
        Gesture::WindowUndo,
        Gesture::Wait(Until::CellIs(0, 0, "220")),
        Gesture::Frame,
    ],
];

/// Cena 14: o ponto de parada no `DEPURA_LUAU` (os passos: Sheet, `self.t` e `self.pos.x` em âmbar
/// pelo Next line, Cards, Strip, e o menu das colunas).
pub const QUADROS_BREAK: &str = "self.t = self.t + dt";

/// ⭐ **Cena 14 — os quadros do depurador** (plano 14).
pub const STEPS_14: &[&[Gesture]] = &[
    &[Gesture::Wait(Until::Stopped("Circle")), Gesture::Frame],
    &[
        Gesture::ClickDebug(1),
        Gesture::Frame,
        Gesture::Wait(Until::Changed("self.t")),
    ],
    &[
        Gesture::ClickDebug(1),
        Gesture::Frame,
        Gesture::ClickDebug(1),
        Gesture::Frame,
        Gesture::Wait(Until::Changed("self.pos.x")),
    ],
    &[
        Gesture::ClickTabMenu(4),
        Gesture::Frame,
        Gesture::ClickTabMenuRow(3),
    ],
    &[
        Gesture::ClickTabMenu(4),
        Gesture::Frame,
        Gesture::ClickTabMenuRow(4),
    ],
    &[
        Gesture::ClickTabMenu(4),
        Gesture::Frame,
        Gesture::ClickTabMenuRow(2),
        Gesture::Frame,
        Gesture::RightClickDebugHeader,
    ],
];
