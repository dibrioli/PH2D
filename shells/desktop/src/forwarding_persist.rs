//! Persistência cross-session do que é do ARTISTA, dirigida pelo hook de ponteiro.
//!
//! Extraído do [`super`] quando o segundo inquilino chegou (as preferências de UI) e o arquivo
//! cruzou o teto de 600 LOC do HR-18. **O corte é por RESPONSABILIDADE, não por tamanho:** o pai
//! *encaminha eventos para o hero*; isto *repara que algo que o artista possui mudou, e grava*.
//!
//! Os dois inquilinos partilham a mesma forma, e ela é a razão de viverem juntos:
//!
//! 1. o facto tem **um dono vivo** (o `WidgetStore` do picker · o `HeroScreen.motion`);
//! 2. o ficheiro é uma **projecção** desse dono, nunca uma segunda cópia autorada;
//! 3. a mudança é **detectada**, nunca anunciada — quem muda o estado não emite intent nenhum, e
//!    é isso que faz uma porta NOVA (um atalho, um `--flag`, um smoke) nascer coberta em vez de
//!    ter de se lembrar de avisar;
//! 4. escrever é **best-effort**: falhar grava um log e nunca mata o frame.

/// Save the picker's named palettes to `~/.ph2d/palettes.txt` when they changed since the last call.
pub(super) fn palettes_if_changed(hero: &ph2d_editor::HeroScreen) {
    let Some(set) = hero
        .store
        .blender_palette_set(ph2d_editor::ids::INSP_BLENDER_PICKER)
    else {
        return;
    };
    let data: Vec<(String, Vec<[u8; 4]>)> = set
        .iter()
        .map(|p| (p.name.clone(), p.swatches.iter().map(|c| c.rgba).collect()))
        .collect();
    let h = crate::palette_persist::hash(&data);
    let changed = LAST_PALETTE_HASH.with(|c| {
        let changed = c.get() != h;
        c.set(h);
        changed
    });
    if changed {
        crate::palette_persist::save(&data);
    }
}

/// Persiste as preferências de UI (`~/.ph2d/prefs.txt`) quando — e só quando — elas mudam.
///
/// ⚠️ **DERIVAÇÃO, não um canal.** O ficheiro é uma projecção de `HeroScreen.motion`, que é o dono
/// único do facto, então o handler do menu não emite intent e não há um segundo sítio a manter em
/// dia. O preço de um intent apareceria no dia em que uma SEGUNDA porta mudasse o carácter: ela
/// teria de se lembrar de emitir, e esquecer-se seria silencioso.
///
/// ⚠️ **O espelho anda mesmo quando a escrita falha.** Andasse só no sucesso, um disco cheio faria
/// disto uma tentativa de escrita por evento de ponteiro, para sempre.
pub(super) fn prefs_if_changed(hero: &ph2d_editor::HeroScreen) {
    let now = crate::prefs::Prefs {
        character: hero.motion.character(),
        reduced_motion: hero.motion.reduced_motion(),
    };
    let previous = LAST_PREFS.with(|c| c.replace(Some(now)));
    if crate::prefs::should_save(previous, now) {
        crate::prefs::save(&now);
    }
}

thread_local! {
    /// Last-saved FNV hash of the named-palette set, so [`palettes_if_changed`] writes only on
    /// a real change instead of every pointer event.
    static LAST_PALETTE_HASH: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Últimas preferências de UI observadas, para [`prefs_if_changed`] escrever só numa mudança
    /// real. `None` = ainda não observadas nesta sessão.
    static LAST_PREFS: std::cell::Cell<Option<crate::prefs::Prefs>> =
        const { std::cell::Cell::new(None) };
}
