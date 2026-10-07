//! **Os instrumentos que leem a lista dos planos** ([`super::PlaneDeltas`]) — o relatório de variante,
//! o de alcance e as divergências de dois snapshots. Só de teste e debug; filho de `undo_planes` para
//! ver os campos, e cortado de lá pelo tecto de LOC, POR ASSUNTO.

use super::PlaneDeltas;

impl PlaneDeltas {
    /// **A VARIANTE de cada plano canvas-shaped, por nome** — o instrumento da §5.66 §4.
    ///
    /// O irmão [`Self::confine_report`] responde *"quem recusou o confinamento?"* e para isso colapsa
    /// `Whole`/`OnlyBefore`/`OnlyAfter` num tag só. Aqui a pergunta é outra — **por qual porta este
    /// plano virou `Whole`?** —, e ela decide onde a cura da posse mora.
    #[cfg(test)]
    pub(crate) fn variant_report(&self) -> String {
        format!(
            "canvas {} · heights [{}] · covers [{}] · mats [{}]",
            self.canvas_rgba.variant(),
            self.heights.variant_tags(),
            self.covers.variant_tags(),
            self.mats.variant_tags(),
        )
    }

    /// **O alcance de CADA plano, por nome** — o instrumento que separa *"o passo não é confinado"* de
    /// *"qual plano recusou"*. Sem ele o `None` é mudo, e um veredito mudo manda adivinhar.
    #[cfg(test)]
    pub(crate) fn confine_report(&self) -> String {
        use crate::undo_delta::StoredEntry;
        use crate::undo_delta::confine::PlaneReach;
        let tag = |r: PlaneReach| match r {
            PlaneReach::Untouched => "-",
            PlaneReach::Window(_) => "win",
            PlaneReach::Whole => "WHOLE",
        };
        let map = |v: Vec<PlaneReach>| {
            if v.is_empty() {
                "-".to_string()
            } else {
                v.into_iter().map(tag).collect::<Vec<_>>().join(",")
            }
        };
        // ⚠️ Destructure EXAUSTIVO, como o `confined_region`: um relatório que enumera à mão esconde
        // justamente o plano que ninguém lembrou de listar — que é o plano que está souring o veredito.
        let Self {
            canvas_rgba,
            images,
            heights,
            covers,
            mats,
            vidros,
            mask_scratch,
            selection_mask,
            selection_crisp,
            deform_disp,
            deform_pre,
            deform_pre_h,
            deform_pre_cover,
            deform_pre_mats,
            sculpt_pre,
            sculpt_amount,
            sculpt_plane_sum,
            sculpt_pre_cover,
            sculpt_pre_mats,
            sculpt_pre_rgba,
            relief_indescribable,
        } = self;
        let mut out = vec![
            format!("canvas={}", tag(canvas_rgba.reach())),
            format!("images={}", map(images.reaches())),
            format!(
                "h={}",
                map(heights.entries().map(StoredEntry::reach).collect())
            ),
            format!(
                "c={}",
                map(covers.entries().map(StoredEntry::reach).collect())
            ),
            format!(
                "m={}",
                map(mats.entries().map(StoredEntry::reach).collect())
            ),
            format!(
                "v={}",
                map(vidros.entries().map(StoredEntry::reach).collect())
            ),
        ];
        for (n, r) in [
            ("mask", mask_scratch.reach()),
            ("selmask", selection_mask.reach()),
            ("selcrisp", selection_crisp.reach()),
            ("dsp", deform_disp.reach()),
            ("dpre", deform_pre.reach()),
            ("dh", deform_pre_h.reach()),
            ("dc", deform_pre_cover.reach()),
            ("dm", deform_pre_mats.reach()),
            ("spre", sculpt_pre.reach()),
            ("samt", sculpt_amount.reach()),
            ("ssum", sculpt_plane_sum.reach()),
            ("sc", sculpt_pre_cover.reach()),
            ("sm", sculpt_pre_mats.reach()),
            ("srgba", sculpt_pre_rgba.reach()),
        ] {
            if !matches!(r, PlaneReach::Untouched) {
                out.push(format!("{n}={}", tag(r)));
            }
        }
        out.push(format!("indescr={relief_indescribable}"));
        out.join(" ")
    }

    /// **Onde dois snapshots DIFEREM, plano a plano** — o TERCEIRO consumidor desta lista, e ele existe
    /// para conferir a premissa do S3 (doc 28 §7): *o estado VIVO do tool serve de base para o delta?*
    ///
    /// O `undo_delta` afirma que não (*"`restore_shape_overlay` RE-CARIMBA a figura, então o vivo depois
    /// de um undo não é byte-a-byte o snapshot instalado"*), e essa frase decide se o `cursor` — hoje um
    /// segundo dono PERMANENTE dos quatro planos canvas-shaped — pode largá-los. Uma afirmação sobre o
    /// produto não se cita: mede-se. A suíte desta crate roda em debug em ~4 s e exercita fill, seleção,
    /// warp, sculpt, máscara, inpaint, clone, aquarela, Wet Paint e os shape editors, então chamar isto
    /// no instante de todo undo/redo pergunta a **todo gesto que algum teste encena**.
    ///
    /// Mora aqui pela mesma razão que o `split` e o `side`: é a lista dos dezenove, e um quarto
    /// consumidor que a enumerasse de novo nasceria com dezoito.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn divergences(
        a: &crate::undo::ModelSnapshot,
        b: &crate::undo::ModelSnapshot,
    ) -> Vec<&'static str> {
        fn plane<T: PartialEq>(
            out: &mut Vec<&'static str>,
            name: &'static str,
            x: &std::sync::Arc<Vec<T>>,
            y: &std::sync::Arc<Vec<T>>,
        ) {
            if !std::sync::Arc::ptr_eq(x, y) && **x != **y {
                out.push(name);
            }
        }
        fn map<T: PartialEq>(
            out: &mut Vec<&'static str>,
            name: &'static str,
            x: &std::collections::BTreeMap<super::RtLayerId, std::sync::Arc<Vec<T>>>,
            y: &std::collections::BTreeMap<super::RtLayerId, std::sync::Arc<Vec<T>>>,
        ) {
            if x.len() != y.len() || x.keys().ne(y.keys()) {
                out.push(name);
                return;
            }
            for (k, xv) in x {
                let yv = &y[k];
                if !std::sync::Arc::ptr_eq(xv, yv) && **xv != **yv {
                    out.push(name);
                    return;
                }
            }
        }
        let mut d = Vec::new();
        plane(&mut d, "canvas_rgba", &a.canvas_rgba, &b.canvas_rgba);
        map(&mut d, "heights", &a.heights, &b.heights);
        map(&mut d, "covers", &a.covers, &b.covers);
        map(&mut d, "mats", &a.mats, &b.mats);
        map(&mut d, "vidros", &a.vidros, &b.vidros);
        plane(&mut d, "mask_scratch", &a.mask_scratch, &b.mask_scratch);
        plane(
            &mut d,
            "selection_mask",
            &a.selection_mask,
            &b.selection_mask,
        );
        plane(
            &mut d,
            "selection_crisp",
            &a.selection_crisp,
            &b.selection_crisp,
        );
        plane(&mut d, "deform.disp", &a.deform.disp, &b.deform.disp);
        plane(&mut d, "deform.pre", &a.deform.pre, &b.deform.pre);
        plane(&mut d, "deform.pre_h", &a.deform.pre_h, &b.deform.pre_h);
        plane(
            &mut d,
            "deform.pre_cover",
            &a.deform.pre_cover,
            &b.deform.pre_cover,
        );
        plane(
            &mut d,
            "deform.pre_mats",
            &a.deform.pre_mats,
            &b.deform.pre_mats,
        );
        plane(&mut d, "sculpt.pre", &a.sculpt.pre, &b.sculpt.pre);
        plane(&mut d, "sculpt.amount", &a.sculpt.amount, &b.sculpt.amount);
        plane(
            &mut d,
            "sculpt.plane_sum",
            &a.sculpt.plane_sum,
            &b.sculpt.plane_sum,
        );
        plane(
            &mut d,
            "sculpt.pre_cover",
            &a.sculpt.pre_cover,
            &b.sculpt.pre_cover,
        );
        plane(
            &mut d,
            "sculpt.pre_mats",
            &a.sculpt.pre_mats,
            &b.sculpt.pre_mats,
        );
        plane(
            &mut d,
            "sculpt.pre_rgba",
            &a.sculpt.pre_rgba,
            &b.sculpt.pre_rgba,
        );
        let images_differ = a.images.len() != b.images.len()
            || a.images.keys().ne(b.images.keys())
            || a.images.iter().any(|(k, x)| {
                let y = &b.images[k];
                !std::sync::Arc::ptr_eq(x, y) && (x.width != y.width || x.rgba8 != y.rgba8)
            });
        if images_differ {
            d.push("images");
        }
        d
    }
}
