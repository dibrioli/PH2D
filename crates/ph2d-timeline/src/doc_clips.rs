//! ⭐⭐⭐ **A AUTORIA DE CLIPS** — irmão de [`super::doc`] pelo teto de 700 LOC da workspace, e o
//! corte é por RESPONSABILIDADE: ali vive *o que o documento É* (as bindings, as chaves, o
//! serializar); aqui, *como o artista cria, nomeia, copia e apaga uma animação*.
//!
//! ⚠️ **A lei dos NOMES vive toda aqui**, e é uma porta só ([`TimelineDoc::unique_clip_name`]): as
//! quatro entradas — criar, renomear, copiar e o nome de partida — passam por ela. Dois clips a
//! partilhar um nome fazem o dropdown mentir, a renomeação ficar ambígua, e — desde que um osso
//! inteligente referencia um clip **pelo nome** — o controlo percorrer **o outro**, calado.

use super::*;

impl TimelineDoc {
    /// Append a clip named `name` and return its index. Refuses past
    /// [`MAX_CLIPS`] (the selector's option ids are a fixed array) and returns
    /// the active index unchanged.
    ///
    /// The new clip is **empty**, and that is the whole model: BINDINGS are
    /// document-wide (a binding maps an entity's property to a stable target id),
    /// so every clip animates the same objects and only the KEYS differ. A second
    /// clip therefore costs a name and nothing else — "walk" and "run" are two sets
    /// of curves over one rig, which is how After Effects and Unity both read it.
    ///
    /// ⚠️ **The name is COERCED unique** ([`Self::unique_clip_name`]) — a Smart Bone references a
    /// clip by name, so two clips sharing one make that control run the wrong animation, silently.
    pub fn add_clip(&mut self, name: String) -> usize {
        if self.clips.len() >= MAX_CLIPS {
            return self.active_clip;
        }
        let name = self.unique_clip_name(&name, None);
        self.clips.push(NamedClip {
            name,
            clip: Clip::new(RationalTime::from_seconds(0.0)),
            loop_range: None,
            loop_ping_pong: false,
            keys_loop_range: None,
            keys_loop_ping_pong: false,
            length_override: None,
            expr: BTreeMap::new(),
            // Em BRANCO: um clip novo não herda trajetória nenhuma (ver o campo).
            paths: BTreeMap::new(),
        });
        self.clips.len() - 1
    }

    /// Rename clip `index` (out of range: no-op).
    ///
    /// ⚠️ **Coerced unique against the OTHERS** ([`Self::unique_clip_name`]) — renaming a clip to
    /// the name it already has is a no-op, never `"Walk 2"`.
    pub fn rename_clip(&mut self, index: usize, name: String) {
        if index >= self.clips.len() {
            return;
        }
        let name = self.unique_clip_name(&name, Some(index));
        if let Some(c) = self.clips.get_mut(index) {
            c.name = name;
        }
    }

    /// **Set (or clear) the per-clip expression** for `target` in clip `index`
    /// (ADR-0151). A blank formula clears it. Out of range: no-op. The formula lives
    /// in the clip like a track, so a strip that plays this clip windows it.
    pub fn set_clip_expr(&mut self, index: usize, target: AnimTarget, expr: Option<String>) {
        if let Some(c) = self.clips.get_mut(index) {
            match expr.filter(|s| !s.trim().is_empty()) {
                Some(s) => {
                    c.expr.insert(target, s);
                }
                None => {
                    c.expr.remove(&target);
                }
            }
        }
    }

    /// **Copy clip `index`** — curves, loop and all — as a new clip at the end, and
    /// return its index. Refuses past [`MAX_CLIPS`], or on an index out of range.
    ///
    /// This is the "start from what I have" button, and it is the one thing
    /// [`Self::add_clip`] cannot be: bindings are document-wide, so a *new* clip is
    /// always empty — a variation ("walk" → "walk, tired") means copying the curves
    /// and editing them, and hand-copying every key is not an authoring workflow.
    ///
    /// The copy is **deep and independent**: the keys carry fresh [`ph2d_anim::KeyId`]s
    /// via [`Clip`]'s own clone, so editing the copy never reaches back into the
    /// original. Its loop travels too — a loop is a property of the animation it
    /// brackets, so a copy of the animation has the same one.
    pub fn duplicate_clip(&mut self, index: usize) -> Option<usize> {
        if self.clips.len() >= MAX_CLIPS {
            return None;
        }
        let src = self.clips.get(index)?;
        let copy = NamedClip {
            name: self.fresh_copy_name(&src.name),
            clip: src.clip.clone(),
            loop_range: src.loop_range,
            loop_ping_pong: src.loop_ping_pong,
            // Both loops travel — a copy of the animation has the same brackets in
            // both views.
            keys_loop_range: src.keys_loop_range,
            keys_loop_ping_pong: src.keys_loop_ping_pong,
            // The explicit duration travels too: it is part of what the clip IS.
            length_override: src.length_override,
            // The per-clip formulas travel — a variation animates the same targets
            // (ADR-0151). A copy of "walk" carries walk's expressions.
            expr: src.expr.clone(),
            // A trajetória viaja: uma cópia da animação percorre a mesma jornada.
            paths: src.paths.clone(),
        };
        self.clips.push(copy);
        Some(self.clips.len() - 1)
    }

    /// **Play clip `index` backwards**: every track of it mirrored inside
    /// `[0, clip_end_seconds(index)]`. Returns `false` on an index out of range.
    ///
    /// The pivot is the clip's **effective end** — the same door the strip sizing and
    /// go-to-end read ([`Self::clip_end_seconds`]), never `duration()`. A hand-keyed
    /// clip has an authored duration of `0`, so mirroring about *that* would fold every
    /// key onto the negative side of zero and the animation would vanish from the
    /// panel. (This is the two-doors bug that already shipped once here, as a 5-second
    /// clip in a 1-second strip.)
    ///
    /// Reversal is a **mirror, not a re-typing of key times**: each segment's shape
    /// travels with it ([`ph2d_anim::Track::reverse_about`]), so an ease-out stays an
    /// ease-out when read backwards.
    pub fn reverse_clip(&mut self, index: usize) -> bool {
        let span = self.clip_end_seconds(index);
        let Some(c) = self.clips.get_mut(index) else {
            return false;
        };
        c.clip.reverse_about(span);
        true
    }

    /// `"Walk copy"`, then `"Walk copy 2"`… — a name no clip is using yet.
    fn fresh_copy_name(&self, of: &str) -> String {
        self.unique_clip_name(&format!("{of} copy"), None)
    }

    /// ⭐⭐⭐ **A NAME NO OTHER CLIP IS USING** — `"Walk"`, then `"Walk 2"`, `"Walk 3"`…
    ///
    /// `except` is the clip being renamed, which must not collide with itself.
    ///
    /// # ⛔⛔ Why this is coerced at the DOOR and not merely warned about
    ///
    /// This file already wrote the law twice — *"two clips sharing a label make the dropdown
    /// unreadable and the rename ambiguous"* — and honoured it in the two doors that INVENT a
    /// name ([`Self::fresh_clip_name`], [`Self::fresh_copy_name`]) while [`Self::add_clip`] and
    /// [`Self::rename_clip`], which take one from the caller, accepted anything.
    ///
    /// ⚠️ A third reader then arrived and made the ambiguity **silent instead of merely ugly**: a
    /// Smart Bone stores the clip's NAME (the house's durable reference), so two clips called
    /// `"Wave"` make the control run **the first one**, with no way to tell from the outside —
    /// the artist configures the second and reads the app as broken. *An ambiguity that only
    /// looked bad in a dropdown becomes a wrong answer the moment something references by name.*
    ///
    /// ⛔ Warning instead would leave the artist holding a document he cannot repair by renaming
    /// (both names are equally valid), which is the shape of a refusal with extra steps.
    #[must_use]
    fn unique_clip_name(&self, wanted: &str, except: Option<usize>) -> String {
        let taken = |name: &str| {
            self.clips
                .iter()
                .enumerate()
                .any(|(i, c)| Some(i) != except && c.name == name)
        };
        if !taken(wanted) {
            return wanted.to_string();
        }
        // ⚠️ The ceiling is [`MAX_CLIPS`] + 1 because that is how many names can be taken at once
        // — the resource is the document's own clip cap, not a number picked here.
        for n in 2..=MAX_CLIPS + 1 {
            let candidate = format!("{wanted} {n}");
            if !taken(&candidate) {
                return candidate;
            }
        }
        wanted.to_string()
    }

    /// Delete clip `index`, returning `true` if it went.
    ///
    /// **The last clip never goes** — a document must always have one to edit, and
    /// an empty `clips` would make `active_clip()` panic on the very next frame.
    /// The active index follows the deletion (it shifts down with the clips above
    /// it, and clamps if it WAS the deleted one), so the caller never has to
    /// repair it.
    pub fn remove_clip(&mut self, index: usize) -> bool {
        if self.clips.len() <= 1 || index >= self.clips.len() {
            return false;
        }
        self.clips.remove(index);
        // A strip names its clip by INDEX, and removing one slides every later
        // clip down: without this, every strip above the hole would quietly start
        // playing its neighbour. Strips of the deleted clip go with it.
        self.repoint_strips_after_clip_removal(index);
        if self.active_clip >= index {
            self.active_clip = self.active_clip.saturating_sub(1);
        }
        true
    }

    /// A name no clip is using yet — `"Clip 2"`, then `"Clip 3"`… Seeds the
    /// selector's "New Clip" so two clips never share a label (which would make
    /// the dropdown unreadable and the rename ambiguous).
    #[must_use]
    pub fn fresh_clip_name(&self) -> String {
        for n in 2..=MAX_CLIPS + 1 {
            let candidate = format!("Clip {n}");
            if !self.clips.iter().any(|c| c.name == candidate) {
                return candidate;
            }
        }
        "Clip".to_string()
    }
}
