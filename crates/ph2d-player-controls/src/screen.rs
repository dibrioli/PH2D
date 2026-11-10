//! O estado da tela e o que cada tecla e clique fazem (plano InputMap §4).

use ph2d_input::{Binding, InputMap, Key, PlayerKeys};

/// Colunas de tecla por linha (a primária e a segunda, como no molde): uma acção com mais teclas mostra todas.
pub const SLOTS: usize = 2;

const KEY_UP: Key = Key(0xF700);
const KEY_DOWN: Key = Key(0xF701);
const KEY_LEFT: Key = Key(0xF702);
const KEY_RIGHT: Key = Key(0xF703);
const KEY_ENTER: Key = Key(0x0D);
const KEY_SPACE: Key = Key(0x20);
const KEY_ESC: Key = Key(0x1B);

/// Uma acção na tela: o nome que o jogo lê, o que o jogador lê, e as teclas de teclado que valem agora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub action: String,
    pub label: String,
    pub keys: Vec<Key>,
}

impl Row {
    /// Caixas de tecla desta linha.
    #[must_use]
    pub fn slots(&self) -> usize {
        self.keys.len().max(SLOTS)
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
        })
        .collect()
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
    Slot { row: usize, slot: usize },
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
        if !pressed {
            return Outcome::Consumed;
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
            KEY_ENTER | KEY_SPACE => self.activate(&rows, keys),
            _ => Outcome::Consumed,
        }
    }

    /// Um clique no painel (o `Hit` vem do [`crate::layout::Layout::hit`]); `None` = fora do painel.
    pub fn click(&mut self, hit: Option<Hit>, keys: &mut PlayerKeys) -> Outcome {
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
            Some(Hit::Reset) => reset(keys),
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
