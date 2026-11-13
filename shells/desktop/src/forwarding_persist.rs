//! Persistência cross-session do que é do ARTISTA, dirigida pelo hook de ponteiro: o facto tem
//! um dono vivo, o ficheiro é uma projecção dele, a mudança é DETECTADA (nunca anunciada: uma
//! porta nova nasce coberta) e escrever é best-effort (falhar loga, nunca mata o frame).

/// Save the picker's named palettes to `~/.ph2d/palettes.txt` when they changed since the last call.
pub(super) fn palettes_if_changed(hero: &ph2d_editor_core::HeroScreen) {
    let Some(set) = hero
        .store
        .blender_palette_set(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
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
/// ⚠️ DERIVAÇÃO, não um canal (o dono é o `HeroScreen`). O espelho anda mesmo quando a escrita
/// falha: senão um disco cheio seria uma escrita por evento de ponteiro, para sempre.
pub(super) fn prefs_if_changed(hero: &ph2d_editor_core::HeroScreen) {
    let now = crate::prefs::Prefs {
        character: hero.motion.character(),
        reduced_motion: hero.motion.reduced_motion(),
        ui_sound: hero.ui_sound,
        text: hero.text_style,
        ui_scale: hero.ui_scale,
        left_handed: hero.view.ui_mirrored,
        luz2d: hero.view.luz2d,
        recovery: ph2d_recovery::Period::from_secs(ph2d_editor_core::recovery_pref::secs()),
        input_map_view: ph2d_app_input_map::view(hero),
        wet_guess: ph2d_tool_painter::palpite_ligado(),
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
