//! O estado da tela e o que cada tecla e clique fazem (plano InputMap §4).

use ph2d_input::{Binding, InputMap, Key, PlayerKeys};

/// Colunas de tecla por linha (a primária e a segunda, como no molde): uma acção com mais teclas mostra todas.
pub const SLOTS: usize = 2;

const KEY_UP: Key = Key(0xF700);
const KEY_DOWN: Key = Key(0xF701);
const KEY_LEFT: Key = Key(0xF702);
const KEY_RIGHT: Key = Key(0xF703);
const KEY_SPACE: Key = Key(0x20);
const KEY_ESC: Key = Key(0x1B);

/// Uma acção na tela: o nome que o jogo lê, o que o jogador lê, e as teclas de teclado que valem agora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub action: String,
    pub label: String,
    pub keys: Vec<Key>,
    /// A tecla REPETIDA que o jogador criou: `(caixa, nome legível da outra acção)` — «J is also Fire.».
    pub also: Option<(usize, String)>,
}

impl Row {
    /// Caixas de tecla desta linha.
    #[must_use]
    pub fn slots(&self) -> usize {
        self.keys.len().max(SLOTS)
    }

    /// O aviso da linha, em palavras: «J is also Fire.».
    #[must_use]
    pub fn note(&self) -> Option<String> {
        let (slot, other) = self.also.as_ref()?;
        let key = self.keys.get(*slot)?;
        Some(
            ph2d_i18n::tr("player_controls.also")
                .replace("{key}", &key.label())
                .replace("{action}", other),
        )
    }
}

/// `move_left` → «Move left».
#[must_use]
pub fn readable_name(name: &str) -> String {
    let spaced = name.replace('_', " ");
    let mut chars = spaced.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().chain(chars).collect::<String>()
    })
}

/// As linhas da tela, do mapa EFECTIVO (o do jogo com as trocas do jogador), sem as acções escondidas.
#[must_use]
pub fn rows(effective: &InputMap) -> Vec<Row> {
    let c = effective.controls();
    effective
        .actions()
        .iter()
        .filter(|a| c.shown_in_game(a.id))
        .map(|a| Row {
            action: a.name.clone(),
            label: readable_name(&a.name),
            keys: a
                .bindings
                .iter()
                .filter_map(|b| match b {
                    Binding::Key(k) => Some(*k),
                    _ => None,
                })
                .collect(),
            also: None,
        })
        .collect()
}

/// ⭐ As linhas com o AVISO da tecla repetida. Escolha (estado da arte, XAG 107 / The Last of Us II / a demo de
/// remapeamento do Godot): AVISAR na linha, nunca trocar sozinho — aqui uma tecla em duas acções é autoria legítima
/// (a fábrica põe ↑ em `jump` e `move_up`), e trocar tiraria em silêncio a tecla da outra acção; a troca com
/// confirmação pedia um passo modal a mais. Só avisa o que o JOGADOR criou: a partilha que já vem do jogo cala-se.
#[must_use]
pub fn rows_marked(effective: &InputMap, game: &InputMap) -> Vec<Row> {
    let mut out = rows(effective);
    let theirs = rows(game);
    let in_game = |action: &str, k: Key| {
        theirs
            .iter()
            .any(|r| r.action == action && r.keys.contains(&k))
    };
    for i in 0..out.len() {
        let me = &out[i];
        let also = me.keys.iter().enumerate().find_map(|(slot, &k)| {
            out.iter()
                .enumerate()
                .find(|(j, o)| {
                    *j != i
                        && o.keys.contains(&k)
                        && !(in_game(&me.action, k) && in_game(&o.action, k))
                })
                .map(|(_, o)| (slot, o.label.clone()))
        });
        out[i].also = also;
    }
    out
}

/// O que um gesto fez — a shell decide o resto (gravar, pausar, retomar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Não era para a tela (fechada e não é a tecla de abrir).
    Ignored,
    /// A tela tomou-o e nada mudou fora dela.
    Consumed,
    Opened,
    Closed,
    /// As teclas do jogador mudaram: grave o ficheiro e refaça o mapa efectivo.
    Changed,
}

/// Onde caiu um clique dentro do painel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Slot {
        row: usize,
        slot: usize,
    },
    Reset,
    /// No painel, fora de um alvo.
    Panel,
}

/// A tela: aberta, a linha e a caixa escolhidas, e se está à escuta. A linha `rows.len()` é «Reset to defaults».
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlsScreen {
    open: bool,
    selected: usize,
    slot: usize,
    listening: bool,
    /// A primeira linha à vista (a lista rola quando não cabe; a pintura acerta-a).
    top: usize,
    /// O alvo sob o ponteiro (D32 do dono: acende no quadro, sem fade); `Hit::Panel` nunca fica aqui.
    hover: Option<Hover>,
    /// O Down caiu no «Reset to defaults»: o Up DENTRO dispara, fora cancela (o idioma do `interact` da aba).
    reset_down: bool,
}

/// O que está sob o ponteiro: uma linha (a faixa inteira, caixas incluídas) ou o botão «Reset to defaults».
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hover {
    Row(usize),
    Reset,
}

impl ControlsScreen {
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }
    #[must_use]
    pub fn slot(&self) -> usize {
        self.slot
    }
    #[must_use]
    pub fn listening(&self) -> bool {
        self.open && self.listening
    }
    #[must_use]
    pub fn top(&self) -> usize {
        self.top
    }

    #[must_use]
    pub fn hovered(&self) -> Option<Hover> {
        self.hover.filter(|_| self.open)
    }
    /// O «Reset to defaults» premido: só enquanto o ponteiro continua nele (sair dele mostra o cancelamento).
    #[must_use]
    pub fn reset_pressed(&self) -> bool {
        self.open && self.reset_down && self.hover == Some(Hover::Reset)
    }

    /// O ponteiro mexeu: guarda o alvo sob ele. `true` = mudou (a pintura tem de refazer o quadro).
    pub fn pointer_move(&mut self, hover: Option<Hover>) -> bool {
        let before = self.hover;
        self.hover = hover.filter(|_| self.open);
        before != self.hover
    }

    /// O Up do botão: só o Up DENTRO do «Reset to defaults» premido dispara; fora cancela.
    pub fn release(&mut self, hit: Option<Hit>, keys: &mut PlayerKeys) -> Outcome {
        if !self.open {
            return Outcome::Ignored;
        }
        match (std::mem::take(&mut self.reset_down), hit) {
            (true, Some(Hit::Reset)) => reset(keys),
            _ => Outcome::Consumed,
        }
    }

    /// A primeira linha à vista, como a pintura a deixou (a escolhida sempre à vista).
    pub fn scroll_to(&mut self, top: usize) {
        self.top = top;
    }

    /// As linhas mudaram por fora (o autor editou o mapa no Play): a escolha não passa de «Reset to defaults».
    pub fn fit(&mut self, rows: &[Row]) {
        self.selected = self.selected.min(rows.len());
        self.clamp_slot(rows);
    }

    /// Abre na primeira linha, sem escutar.
    pub fn open(&mut self) {
        *self = Self {
            open: true,
            ..Self::default()
        };
    }

    pub fn close(&mut self) {
        *self = Self::default();
    }

    /// Uma tecla física. `game` = o mapa do PROJECTO (nunca o efectivo: é sobre ele que as trocas se medem).
    pub fn handle_key(
        &mut self,
        key: Key,
        pressed: bool,
        game: &InputMap,
        keys: &mut PlayerKeys,
    ) -> Outcome {
        let c = game.controls();
        if !self.open {
            if pressed && c.players_can_change_keys && key == c.open_key {
                self.open();
                return Outcome::Opened;
            }
            return Outcome::Ignored;
        }
        // O soltar passa sempre: uma tecla que o jogo tinha em baixo ao abrir não fica presa.
        if !pressed {
            return Outcome::Ignored;
        }
        let rows = rows(&keys.apply_over(game));
        if self.listening {
            self.listening = false;
            if key == KEY_ESC || key == c.open_key {
                return Outcome::Consumed;
            }
            let Some(row) = rows.get(self.selected) else {
                return Outcome::Consumed;
            };
            return rebind(game, keys, &row.action, self.slot, key);
        }
        match key {
            KEY_ESC => {
                self.close();
                Outcome::Closed
            }
            k if k == c.open_key => {
                self.close();
                Outcome::Closed
            }
            KEY_UP => {
                self.selected = self.selected.saturating_sub(1);
                self.clamp_slot(&rows);
                Outcome::Consumed
            }
            KEY_DOWN => {
                self.selected = (self.selected + 1).min(rows.len());
                self.clamp_slot(&rows);
                Outcome::Consumed
            }
            KEY_LEFT => {
                self.slot = self.slot.saturating_sub(1);
                Outcome::Consumed
            }
            KEY_RIGHT => {
                self.slot += 1;
                self.clamp_slot(&rows);
                Outcome::Consumed
            }
            // Os dois Enter (fileira e numérico) confirmam: `Key::is_enter_like`.
            k if k.is_enter_like() || k == KEY_SPACE => self.activate(&rows, keys),
            _ => Outcome::Consumed,
        }
    }

    /// O Down no painel (o `Hit` vem do [`crate::layout::Layout::hit`]); `None` = fora do painel. Numa caixa escuta já;
    /// no «Reset to defaults» só o captura ([`Self::release`] dispara).
    pub fn click(&mut self, hit: Option<Hit>) -> Outcome {
        if !self.open {
            return Outcome::Ignored;
        }
        self.listening = false;
        match hit {
            Some(Hit::Slot { row, slot }) => {
                self.selected = row;
                self.slot = slot;
                self.listening = true;
                Outcome::Consumed
            }
            Some(Hit::Reset) => {
                self.reset_down = true;
                Outcome::Consumed
            }
            Some(Hit::Panel) | None => Outcome::Consumed,
        }
    }

    fn activate(&mut self, rows: &[Row], keys: &mut PlayerKeys) -> Outcome {
        if self.selected >= rows.len() {
            return reset(keys);
        }
        self.listening = true;
        Outcome::Consumed
    }

    fn clamp_slot(&mut self, rows: &[Row]) {
        let max = rows.get(self.selected).map_or(0, |r| r.slots() - 1);
        self.slot = self.slot.min(max);
    }
}

fn reset(keys: &mut PlayerKeys) -> Outcome {
    if keys.is_empty() {
        return Outcome::Consumed;
    }
    keys.clear_all();
    Outcome::Changed
}

/// A tecla nova troca a ligação de teclado `slot` de `action` (ou junta-se, se a caixa estava vazia). Igual ao do
/// jogo ⇒ a troca esquece-se (o ficheiro guarda só o que o jogador mudou).
pub fn rebind(
    game: &InputMap,
    keys: &mut PlayerKeys,
    action: &str,
    slot: usize,
    new: Key,
) -> Outcome {
    let Some(a) = game.actions().iter().find(|a| a.name == action) else {
        return Outcome::Consumed;
    };
    let before: Vec<Binding> = keys
        .get(action)
        .map_or_else(|| a.bindings.clone(), <[Binding]>::to_vec);
    let mut b = before.clone();
    let kb: Vec<usize> = (0..b.len())
        .filter(|i| matches!(b[*i], Binding::Key(_)))
        .collect();
    match kb.get(slot) {
        Some(&i) => b[i] = Binding::Key(new),
        None => b.push(Binding::Key(new)),
    }
    let mut seen = Vec::with_capacity(b.len());
    b.retain(|x| {
        let fresh = !seen.contains(x);
        seen.push(*x);
        fresh
    });
    if b == before {
        return Outcome::Consumed;
    }
    if b == a.bindings {
        keys.clear(action);
    } else {
        keys.set(action, b);
    }
    Outcome::Changed
}
