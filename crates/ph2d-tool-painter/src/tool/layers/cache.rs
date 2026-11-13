//! Composite-CACHE invalidation — the single structural chokepoint that drops
//! the CPU composite + dirty-rect + adjustment cut-cache and bumps the publish
//! revision. `impl PainterTool` (one of several blocks in this crate). Split out
//! of the former `tool/layers.rs` god-file (pure move).

use super::super::*;
use crate::undo_delta::confine::PlaneReach;

/// Um gesto de estrutura medido ANTES dele ([`PainterTool::mede_a_estrutura`]).
pub(crate) struct MedidaDaEstrutura {
    /// A camada do gesto.
    id: RtLayerId,
    /// Onde a figura pode mudar (`None` = não provado).
    alcance: Option<PlaneReach>,
    /// A lista achatada da placa antes do gesto (N7 onda 5).
    lista: Option<Vec<ph2d_layer_ops::LayerOp>>,
    /// A activa com a tinta viva do Wet antes do gesto (N7 onda 5).
    saida: Option<RtLayerId>,
}

#[cfg(test)]
thread_local! {
    /// A sonda A/B do B18.3 no mesmo processo: `true` = a troca recompõe sempre, como antes.
    pub(crate) static TROCA_RECOMPOE_SEMPRE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// N7 (A×B em execução): `true` = a troca com relevo aceso ou na pista da placa recompõe, como antes.
/// `PH2D_TROCA_RECOMPOE=1` no arranque; a sonda vira-a no mesmo processo.
pub static TROCA_RECOMPOE_COMO_ANTES: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(2);

fn troca_recompoe_como_antes() -> bool {
    use std::sync::atomic::Ordering::Relaxed;
    match TROCA_RECOMPOE_COMO_ANTES.load(Relaxed) {
        2 => {
            let v = std::env::var_os("PH2D_TROCA_RECOMPOE").is_some_and(|v| v == "1");
            TROCA_RECOMPOE_COMO_ANTES.store(u8::from(v), Relaxed);
            v
        }
        v => v == 1,
    }
}

impl PainterTool {
    /// Bump the panel's publish revision — and **nothing else**.
    ///
    /// The layers panel re-clones the stack only when this changes ([`Self::layers_revision`]). Most
    /// callers want [`Self::invalidate_composite`], which bumps it as part of dropping every cache. This
    /// is for the one case that is NOT a composite change: a metadata bit the PANEL must learn about
    /// while the pixels stay exactly as they are (`Layer::has_relief`). Reaching for the big hammer there
    /// cost a full canvas recompose — 225 ms at 4096² — to publish a boolean.
    pub(crate) fn bump_layers_revision(&mut self) {
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// **A pista da CPU volta a ser a dona da prévia** (a placa largou o documento, B16.12): o próximo
    /// [`Self::take_preview_arc`] compõe a tela INTEIRA já neste quadro, e não no próximo sujo (o toque).
    pub fn pede_a_composicao_inteira(&mut self) {
        self.invalidate_composite();
        self.preview_dirty = true;
    }

    /// Mark the composite stale so the next `current_preview` recomputes.
    ///
    /// ⚠️ **`PH2D_INVALIDATE_TRACE=1` diz QUEM derrubou o cache** (só em teste, e só com a env var).
    /// Ele existe porque *"o cache foi derrubado"* é um fato sem autor: o `restore_model` chama uma
    /// dúzia de restauradores, e um deles invalidava incondicionalmente — o que manteve o quadro
    /// pós-Ctrl+Z em 381 ms **depois** de o undo já publicar o retângulo certo (doc 28 §5.63). Uma
    /// varredura de callees respondeu ERRADO (casou com a função de nome parecido, noutro arquivo); um
    /// backtrace respondeu em uma corrida.
    pub(crate) fn invalidate_composite(&mut self) {
        #[cfg(test)]
        if std::env::var_os("PH2D_INVALIDATE_TRACE").is_some() {
            eprintln!(
                "INVALIDATE:\n{}",
                std::backtrace::Backtrace::force_capture()
            );
        }
        self.composited = None;
        self.estrutura_confinada = None;
        self.larga_a_luz_guardada();
        // Drop any accumulated dirty-rect: a structural edit (opacity / blend /
        // visibility / reorder / add / select) can change the composite OUTSIDE
        // the stamped region, so the next drain must do a FULL recompose.
        self.dirty_rect = None;
        // …e a pista GPU também: um `mark_dirty` antes da drenagem (o 1.º dab do traço que virou a marca do
        // Wet) re-encheria o `dirty_rect` e ela dobraria só a janela dele (B10c).
        self.gpu_lane_stale = true;
        self.preview_dirty = true;
        self.edited_since_bind = true; // structural/metadata edit → unbaked composite change
        // W5: a structural edit (add/remove/reorder/visibility/opacity/blend/
        // select) changes the composite below some adjustment → every cut is
        // potentially stale. Conservative-correct: drop them all (they cold-
        // refill on the next slider-drag). The id is ignored by `invalidate_from`.
        self.compositor_cache
            .invalidate_from(RtLayerId(0), &self.layers);
        // B.5: every structural/metadata edit funnels through here, so this is
        // the single chokepoint that bumps the publish revision (set_source is
        // the only structural reset that bypasses it — bumped there too).
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// **A troca da activa deixa a figura composta igual?** (B18.3) — só a pergunta da pista CPU, e
    /// conservadora: `false` salvo provado. O composite é função dos bytes de cada camada e dos metadados
    /// da pilha, nunca da activa; o que lê a activa fica de fora: o filme da proteção, a vista cinza da
    /// máscara, o relevo aceso (salvo o que não lê a activa), a tinta viva do Wet (adormecer escreve a última
    /// imagem na tela), uma forma aberta (fecha-se na tela) e um traço aberto. Pergunta-se antes E depois.
    pub(crate) fn a_troca_mantem_a_figura(&self, alvo: RtLayerId) -> bool {
        self.a_troca_mantem_com(alvo, true)
    }

    /// N7 onda 3: a mesma pergunta, salvo a tinta viva do Wet — quem adormece ou acorda na troca marca só a janela
    /// da tinta dela ([`Self::a_tinta_que_adormeceu_muda`]), como no «+». Com a chave de antes, nunca.
    pub(crate) fn a_troca_mantem_salvo_a_tinta(&self, alvo: RtLayerId) -> bool {
        !troca_recompoe_como_antes() && self.a_troca_mantem_com(alvo, false)
    }

    fn a_troca_mantem_com(&self, alvo: RtLayerId, tinta_conta: bool) -> bool {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize) * 4;
        #[cfg(test)]
        if TROCA_RECOMPOE_SEMPRE.with(std::cell::Cell::get) {
            return false;
        }
        let antes = troca_recompoe_como_antes();
        // N7: a pista da placa drena com `composited = None`; sem sujo nem pista velha, o slot dela é a figura.
        let placa_em_dia = !antes && self.composited.is_none() && !self.gpu_lane_stale;
        (self.composited.as_ref().is_some_and(|c| c.len() == n) || placa_em_dia)
            // N7 onda 4: o sujo CONFINADO pendente (a simulação viva do Wet marca-o a ~40 imagens/s) fica para a
            // drenagem seguinte, que o recompõe de qualquer modo; fora dele o cache é a figura.
            && (!self.preview_dirty || (!antes && self.dirty_rect.is_some()))
            && self.canvas_rgba.len() == n
            && (self.layers.active() == Some(alvo)
                || self
                    .images
                    .get(&alvo)
                    .is_some_and(|i| i.width == w && i.height == h && i.rgba8.len() == n))
            && !self.is_editing_shape()
            && !self.has_parked_shapes()
            && !(if tinta_conta { self.traco_ou_tinta_viva() } else { self.traco_por_fechar() })
            && !self.mask_scratch_active()
            && self.mask_view_grayscale.is_none()
            && (!self.impasto_visible() || (!antes && self.o_relevo_nao_le_a_activa()))
            // N7: a composição pelo vidro lê os bytes e o vidro de cada camada, nunca a activa (o traço aberto, que
            // o segue, já ficou de fora).
            && (!antes || !self.papel_atravessa_vidro())
    }

    /// **A camada nova TRANSPARENTE deixa a figura igual?** (B18.4, o «+») — α = 0 não compõe nada nem cobre o
    /// dente do papel (`ReliefFields::tinta` lê o α), então, ao contrário da troca, o relevo aceso e a tinta viva
    /// (cada uma marca o próprio sujo) ficam dentro; só um traço aberto e o que lê a activa ficam de fora. O sujo
    /// pendente (cache vazio, pista GPU velha) fica como estava: nada se acrescenta, nada se perde.
    pub(crate) fn a_nova_transparente_mantem_a_figura(&self) -> bool {
        #[cfg(test)]
        if TROCA_RECOMPOE_SEMPRE.with(std::cell::Cell::get) {
            return false;
        }
        !self.traco_por_fechar()
            && !self.is_editing_shape()
            && !self.has_parked_shapes()
            && !self.mask_scratch_active()
            && self.mask_view_grayscale.is_none()
            && !self.papel_atravessa_vidro()
    }

    /// **Onde a figura pode mudar quando a camada raster `id` sai, ganha uma cópia por cima ou entra num grupo**
    /// (N7): só onde ela tem α > 0 — fora dela não compõe nada, em qualquer modo, opacidade, ordem ou grupo. Medida
    /// ANTES do gesto. `None` = não provado (o caminho de antes recompõe tudo): o que espalha para fora do α ou
    /// lê a vizinha (ajuste, recorte, relevo/material próprio) fica de fora. A tinta viva do Wet entra (onda 5): a
    /// que adormece ou acorda marca a janela dela em [`Self::a_estrutura_muda_so_em`].
    pub(crate) fn alcance_da_tinta_de(&self, id: RtLayerId) -> Option<PlaneReach> {
        use crate::layers::LayerKind;
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize) * 4;
        if troca_recompoe_como_antes()
            || !self.a_nova_transparente_mantem_a_figura()
            || !matches!(
                self.layers.get(id)?.kind,
                LayerKind::Raster(_) | LayerKind::Texture(_)
            )
            || self.heights.contains_key(&id)
            || self.covers.contains_key(&id)
            || self.mats.contains_key(&id)
            || self.layers.all_ids().any(|l| {
                self.layers
                    .get(l)
                    .is_some_and(|l| l.clipping || matches!(l.kind, LayerKind::Adjustment(_)))
            })
        {
            return None;
        }
        let px: &[u8] = if self.layers.active() == Some(id) {
            &self.canvas_rgba
        } else {
            &self.images.get(&id)?.rgba8
        };
        (px.len() == n).then(|| crate::undo_delta::confine::janela_da_tinta(px, w as usize))
    }

    /// [`Self::alcance_da_tinta_de`] e, quando provado, a lista achatada da placa ANTES do gesto (N7 onda 5).
    pub(crate) fn mede_a_estrutura(&self, id: RtLayerId) -> MedidaDaEstrutura {
        let alcance = self.alcance_da_tinta_de(id);
        let lista = alcance
            .is_some()
            .then(|| crate::flatten_for_gpu(&self.layers).map(|(ops, _)| ops))
            .flatten();
        let saida = self.layers.active().filter(|_| self.tinta_viva());
        MedidaDaEstrutura {
            id,
            alcance,
            lista,
            saida,
        }
    }

    /// O fim do gesto medido por [`Self::mede_a_estrutura`]: a figura recompõe-se só na janela da tinta, e a
    /// placa recebe `(antes, depois)` para compor só o sujo com a lista nova. A tinta viva do Wet que adormeceu
    /// (de outra camada) ou acordou (na nova activa) marca a janela da tinta dela, como na troca (onda 3).
    pub(crate) fn a_estrutura_muda_so_em(&mut self, m: MedidaDaEstrutura) {
        let MedidaDaEstrutura {
            id,
            alcance,
            lista: antes,
            saida,
        } = m;
        let (w, _) = self.source_size;
        let confinada = match alcance {
            Some(PlaneReach::Untouched) => {
                self.troca_sem_mudar_a_figura();
                true
            }
            Some(PlaneReach::Window(j)) => match j.to_region(w) {
                Some(r) => {
                    self.troca_sem_mudar_a_figura();
                    self.mark_dirty(r);
                    true
                }
                None => false,
            },
            _ => false,
        };
        if !confinada {
            self.invalidate_composite();
            return;
        }
        let depois = crate::flatten_for_gpu(&self.layers).map(|(ops, _)| ops);
        self.estrutura_confinada = antes.zip(depois);
        // Depois da lista: um `invalidate_composite` aqui larga-a.
        let activa = self.layers.active();
        if let Some(s) = saida.filter(|&s| s != id && Some(s) != activa) {
            self.a_tinta_que_adormeceu_muda(s);
        }
        if self.tinta_viva() && activa != saida {
            self.a_tinta_que_acordou_muda();
        }
    }

    /// A tinta viva da camada `id` adormeceu no «+» (B18.4): a figura muda só na janela da tinta dela (a ótica viva
    /// → a assentada); fora dela nada muda.
    pub(crate) fn a_tinta_que_adormeceu_muda(&mut self, id: RtLayerId) {
        use crate::undo_delta::confine::{PlaneReach, janela_da_tinta};
        let (w, h) = self.source_size;
        let alcance = match self.images.get(&id) {
            Some(i) if i.rgba8.len() == (w as usize) * (h as usize) * 4 => {
                janela_da_tinta(&i.rgba8, w as usize)
            }
            _ => PlaneReach::Whole,
        };
        match alcance {
            PlaneReach::Untouched => {}
            PlaneReach::Window(j) => match j.to_region(w) {
                Some(r) => self.mark_dirty(r),
                None => self.invalidate_composite(),
            },
            PlaneReach::Whole => self.invalidate_composite(),
        }
    }

    /// A tinta da activa acordou na troca (N7 onda 3): a figura muda só na janela da tinta da tela dela (a assentada →
    /// a ótica viva), o inverso de [`Self::a_tinta_que_adormeceu_muda`].
    pub(crate) fn a_tinta_que_acordou_muda(&mut self) {
        use crate::undo_delta::confine::{PlaneReach, janela_da_tinta};
        let (w, h) = self.source_size;
        let alcance = if self.canvas_rgba.len() == (w as usize) * (h as usize) * 4 {
            janela_da_tinta(&self.canvas_rgba, w as usize)
        } else {
            PlaneReach::Whole
        };
        match alcance {
            PlaneReach::Untouched => {}
            PlaneReach::Window(j) => match j.to_region(w) {
                Some(r) => self.mark_dirty(r),
                None => self.invalidate_composite(),
            },
            PlaneReach::Whole => self.invalidate_composite(),
        }
    }

    /// A troca da activa com a figura provada igual ([`Self::a_troca_mantem_a_figura`]): o painel publica e
    /// os cortes dos ajustes caem como no [`Self::invalidate_composite`], mas o `composited`, o sujo e a
    /// pista GPU ficam — o quadro seguinte não recompõe a pilha (37 ms a 4K com 3 camadas).
    pub(crate) fn troca_sem_mudar_a_figura(&mut self) {
        self.larga_a_luz_guardada();
        self.edited_since_bind = true;
        self.compositor_cache
            .invalidate_from(RtLayerId(0), &self.layers);
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Force the NEXT preview drain to FULLY recompose + FULLY upload (drop the dirty-rect fast lane), so
    /// a new SHAPE session (Curve / Free Hand / Ellipse / Polygon "no session yet" creation block) starts
    /// from a byte-correct preview base instead of patching a possibly-stale `composited` cache — the rare
    /// early-session artifact where a sliver of the shape appears then vanishes, only on the first few uses
    /// (`HANDOFF_per_layer_color_perf_artifacts` §1.R FOLLOW-UP). Unlike [`Self::invalidate_composite`] it
    /// does NOT flag an edit (`edited_since_bind`) nor drop the adjustment cut-caches (no pixels changed,
    /// it only re-seeds the preview). Cheap: one full recompose + upload, once per shape-session start.
    pub(crate) fn reseed_preview_base(&mut self) {
        self.composited = None; // → `take_preview_arc` takes the full-recompose branch next drain
        self.dirty_rect = None; // → `preview_upload_bbox = None` → the bridge does a FULL texture upload
        self.preview_dirty = true; // ensure that drain runs even though no pixels changed yet
    }
}
