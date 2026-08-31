//! ⭐⭐ **A ARRUMAÇÃO DO ARTISTA sobrevive ao fecho do app** — `~/.ph2d/layout.txt`.
//!
//! > *«⭐ E torna o layout **serializável de forma trivial**: um layout é `{encaixe → [painéis],
//! > posição das divisórias}`.»* — `00_DECISOES_DO_ENIO.md`, D4
//!
//! É essa frase, ao pé da letra: as **excepções de encaixe** (quem o artista moveu) e as **duas
//! larguras de coluna** (a divisória que ele arrastou).
//!
//! # Por que um ficheiro PRÓPRIO, e não uma chave no `prefs.txt`
//!
//! O [`crate::prefs`] guarda um `Prefs` **`Copy`** — três escalares — e o espelho que decide *«isto
//! mudou?»* é um `Cell<Option<Prefs>>`. Uma arrumação é uma **colecção** de tamanho variável: pô-la
//! ali obrigaria o tipo a deixar de ser `Copy` e o espelho a mudar de forma.
//!
//! ⇒ o irmão certo é o [`crate::palette_persist`], que já resolve exactamente esta classe: mesma
//! pasta, mesmo estilo (texto, std-only, best-effort, sem serde), e o espelho é um **hash FNV** da
//! projecção. ⚠️ *Não é uma casa nova* — é a terceira gaveta da que já existe.
//!
//! # ⚠️ Sem número de schema, pela razão do `prefs.txt`
//!
//! Num `chave=valor` a compatibilidade é grátis nos dois sentidos: um build antigo lê as chaves que
//! conhece e ignora as outras. E uma linha `slot.<painel>` de um painel que **já não existe** é
//! saltada por construção — quem a lê procura o painel no registry.
//!
//! ⛔ **Um encaixe que o painel já não PERMITE também é saltado.** O ficheiro é do artista, mas o
//! `ALLOWED_SLOTS` é do produto: se uma wave estreitar o que um painel aceita, a arrumação gravada
//! não pode ressuscitar um sítio onde ele deixou de caber. *A validação vive na leitura, não na
//! escrita — o ficheiro pode ser mais velho que a regra.*

use ph2d_editor::screens::slot::Slot;
use std::path::PathBuf;

/// A arrumação que viaja: as excepções de encaixe (por `Panel::ID`) e as duas larguras.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    /// `(Panel::ID, encaixe)`, ordenado pelo id — a ordem é o que torna o hash estável.
    pub slots: Vec<(String, Slot)>,
    pub dock_w_left: Option<f32>,
    pub dock_w_right: Option<f32>,
}

/// `~/.ph2d/layout.txt`, ou `None` com `$HOME` por definir (a persistência é então saltada).
fn layout_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".ph2d").join("layout.txt"))
}

/// Serializa para `chave=valor`. Inverso de [`parse`].
#[must_use]
pub fn serialize(l: &Layout) -> String {
    use std::fmt::Write as _;
    let mut s = String::from("# PH2D layout\n");
    if let Some(w) = l.dock_w_left {
        let _ = writeln!(s, "dock_w_left={w}");
    }
    if let Some(w) = l.dock_w_right {
        let _ = writeln!(s, "dock_w_right={w}");
    }
    for (id, slot) in &l.slots {
        let _ = writeln!(s, "slot.{id}={}", slot.wire());
    }
    s
}

/// Lê o formato `chave=valor`. **Toda linha que não se entende é saltada** — comentário, lixo,
/// chave de um build mais novo, encaixe desconhecido. É esta tolerância que dispensa a versão.
#[must_use]
pub fn parse(text: &str) -> Layout {
    let mut l = Layout::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "dock_w_left" => l.dock_w_left = value.parse().ok(),
            "dock_w_right" => l.dock_w_right = value.parse().ok(),
            _ => {
                if let Some(id) = key.strip_prefix("slot.")
                    && let Some(slot) = Slot::from_wire(value)
                {
                    l.slots.push((id.to_string(), slot));
                }
            }
        }
    }
    l.slots.sort();
    l
}

/// Serializa + escreve. Um erro de IO é registado, nunca fatal.
pub fn save(l: &Layout) {
    let Some(path) = layout_file() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, serialize(l)) {
        eprintln!("[ph2d] layout save: {e}");
    }
}

/// Lê + parseia. Vazio com o ficheiro ausente / ilegível / malformado.
#[must_use]
pub fn load() -> Layout {
    layout_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| parse(&t))
        .unwrap_or_default()
}

/// FNV-1a da projecção, para o host escrever só numa mudança real.
#[must_use]
pub fn hash(l: &Layout) -> u64 {
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |b: u8| {
        h ^= u64::from(b);
        h = h.wrapping_mul(PRIME);
    };
    for (id, slot) in &l.slots {
        for &b in id.as_bytes() {
            feed(b);
        }
        feed(0);
        for &b in slot.wire().as_bytes() {
            feed(b);
        }
        feed(0);
    }
    // ⚠️ As larguras entram pelos BITS do `f32`, não por um arredondamento: arrastar a divisória
    // meio pixel é uma mudança que o artista fez, e um hash que a ignorasse gravaria com atraso.
    for w in [l.dock_w_left, l.dock_w_right] {
        for &b in &w.unwrap_or(f32::NAN).to_bits().to_le_bytes() {
            feed(b);
        }
    }
    h
}

/// ⭐ **Grava?** — a decisão, pura, para ser gateada sem tocar no disco.
///
/// ⛔⛔ **A PRIMEIRA observação de uma sessão NUNCA grava**, e a razão não é economia: nesse
/// instante o estado é exactamente o que acabou de ser LIDO do ficheiro, então escrever seria
/// reescrever o ficheiro no arranque de toda sessão — inclusive de uma em que o artista não tocou
/// em nada. ⚠️ E com o disco cheio isso viraria um erro por sessão sobre um facto que ninguém mudou.
///
/// ⚠️ **Eu escrevi isto ao contrário na primeira versão** — o comentário prometia *«o primeiro
/// quadro não grava»* e o código gravava (`None != Some(h)` é verdade). É por isso que a decisão é
/// uma função com nome em vez de uma condição dentro do hook.
#[must_use]
pub fn should_save(previous: Option<u64>, now: u64) -> bool {
    previous.is_some_and(|p| p != now)
}

/// ⭐ **O que o app tem AGORA**, lido do dono vivo — a projecção que o ficheiro espelha.
///
/// ⚠️ **Só as EXCEPÇÕES.** Um painel que nunca foi movido não aparece, e por isso um painel que
/// nasce amanhã vai para onde ele próprio declara sem uma linha de migração.
#[must_use]
pub fn current(hero: &ph2d_editor::HeroScreen) -> Layout {
    use ph2d_editor::screens::layout::DockSide;
    let mut slots: Vec<(String, Slot)> = Vec::new();
    ph2d_editor::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            if let Some(s) = hero.store.panel_slot(p.manifest.panel_node_id) {
                slots.push((p.manifest.id.to_string(), s));
            }
        }
    });
    slots.sort();
    Layout {
        slots,
        // ⚠️ `dock_width` devolve sempre um número (o default quando ninguém arrastou), então
        // gravá-lo directamente escreveria o default como se fosse uma escolha. O que se grava é a
        // escolha — e ela existe só quando difere do que o layout daria sozinho.
        dock_w_left: hero.store.dock_width_choice(DockSide::Left),
        dock_w_right: hero.store.dock_width_choice(DockSide::Right),
    }
}

/// ⭐ **Instala uma arrumação gravada.** Chamado ANTES do primeiro quadro.
///
/// ⛔ Um encaixe que o painel já não permite é **saltado**, e o painel fica onde ele declara — ver
/// o cabeçalho do módulo.
pub fn install(hero: &mut ph2d_editor::HeroScreen, l: &Layout) {
    use ph2d_editor::screens::layout::DockSide;
    ph2d_editor::panel::with_registry_opt(|reg| {
        for (id, slot) in &l.slots {
            let Some(p) = reg.panels().iter().find(|p| p.manifest.id == id.as_str()) else {
                continue; // um painel que já não existe nesta build
            };
            if !p.manifest.allowed_slots.contains(*slot) {
                continue; // o produto estreitou o que ele aceita
            }
            hero.store.set_panel_slot(p.manifest.panel_node_id, *slot);
        }
    });
    if let Some(w) = l.dock_w_left {
        hero.store.set_dock_width(DockSide::Left, w);
    }
    if let Some(w) = l.dock_w_right {
        hero.store.set_dock_width(DockSide::Right, w);
    }
}

#[cfg(test)]
#[path = "layout_persist_tests.rs"]
mod tests;
