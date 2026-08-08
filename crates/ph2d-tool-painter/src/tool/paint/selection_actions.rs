//! Selection **actions** (ADR-0103 Wave 5) — the panel's action buttons that operate on the active
//! selection: **Select layer contents** (mask ← the layer's opaque texels), **Color Fill** (flood the
//! selected region with the brush colour), and **Copy / Paste** (an in-memory clipboard of the selected
//! pixels). Every mutation that changes pixels or the mask records ONE structural undo entry, so it joins
//! the single interleaved queue. Split from `selection` for the LOC cap.

use super::selection_shapes::{SelectionEntry, SelectionShape};
use super::{PainterTool, Region};
use ph2d_painter_brush::material::MaterialBytes;
use std::sync::Arc;

/// An in-memory clip of copied selection pixels: the source bounding box + its straight-RGBA texels
/// (already coverage-premultiplied against the selection, so Paste composites cleanly).
#[derive(Clone, Debug)]
pub(crate) struct SelectionClip {
    pub rect: Region,
    pub rgba: Vec<u8>,
    /// **O CORPO da tinta copiada** — `None` quando a camada de origem não tem relevo nenhum.
    ///
    /// Ele viaja junto com a cor e não ao lado dela (Enio, 2026-08-07: *"o Copy/Paste não levou o relevo
    /// do impasto, apenas a cor"*): sob impasto uma pincelada é **espessura + cobertura + material** tanto
    /// quanto pigmento, e um clipboard que leva um quarto do fato cola uma decalcomania chapada de algo que
    /// o artista esculpiu. A lei que o doc do MATERIAL já tinha escrito, do outro lado — *ao adicionar um
    /// plano, adicione-o ao snapshot no MESMO commit* — vale igual para o clipboard.
    pub relief: Option<ReliefClip>,
}

/// Os três planos que fazem da tinta uma substância (`docs/Painter/15..17`), recortados do mesmo `rect` do
/// [`SelectionClip`]: espessura, quanta tinta há ali, e de que material ela é.
///
/// Eles andam **juntos ou nenhum**: a luz pesa a altura pela cobertura e a colore pelo material, então
/// carregar um subconjunto é a doença de *duas coisas que devem concordar sobre a mesma tinta,
/// discordando* — a mesma que custou a ESCADA da silhueta e o buraco do `mats` no `ModelSnapshot`.
#[derive(Clone, Debug)]
pub(crate) struct ReliefClip {
    /// A espessura, em cargas de tinta.
    pub heights: Vec<f32>,
    /// Quanta tinta há ali — **já multiplicada pela cobertura da seleção**, o espelho exato do que o
    /// alfa do RGBA faz: uma borda meio-selecionada leva meia tinta.
    pub covers: Vec<u8>,
    /// Rugosidade / metálico / cera + a cor da cera.
    pub mats: Vec<MaterialBytes>,
}

impl PainterTool {
    /// **Select layer contents**: replace the selection with the active layer's opaque region (alpha > 0).
    /// Collapses the shape list to one `Raster` entry (the alpha silhouette is non-parametric). One undo entry.
    pub fn selection_from_layer_contents(&mut self) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let before = self.snapshot_model();
        let mut crisp = vec![0u8; w * h];
        for (i, c) in crisp.iter_mut().enumerate() {
            if self.canvas_rgba[i * 4 + 3] > 0 {
                *c = 255;
            }
        }
        self.paint.selection_shapes = vec![SelectionEntry {
            shape: SelectionShape::Raster {
                crisp: Arc::new(crisp.clone()),
            },
            op: 0,
        }];
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// **Color Fill**: paint the brush colour into the selected region of the active layer, blended by the
    /// selection coverage (feathered edges partial). No-op without a live selection. One undo entry.
    pub fn selection_color_fill(&mut self) {
        if !self.selection_restricts_paint() {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let before = self.snapshot_model();
        let color = [
            (self.paint.brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            255,
        ];
        // ⚠️ **A cobertura de um FILL não é a máscara crua no Impasto.** A porta única
        // (`fill_selection_keep`) devolve a máscara verbatim no digital — byte-idêntico, o mesmo `Arc` —
        // e, com corpo em mãos, a borda da seleção com o perfil do Falloff que o artista escolheu.
        let mask = self.fill_selection_keep();
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        let n = mask.len().min(buf.len() / 4);
        for i in 0..n {
            let cov = f32::from(mask[i]) / 255.0;
            if cov <= 0.0 {
                continue;
            }
            let b = i * 4;
            for c in 0..4 {
                let dst = f32::from(buf[b + c]);
                let src = f32::from(color[c]);
                buf[b + c] = (src * cov + dst * (1.0 - cov)).round().clamp(0.0, 255.0) as u8;
            }
        }
        self.mark_dirty(Region {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        });
        // …e o CORPO, pela MESMA cobertura que pintou a cor: as duas metades da mesma tinta não podem
        // discordar sobre onde ela está (a doença que o Accumulate do relevo já custou uma wave).
        self.deposit_fill_body(&mask);
        self.commit_structural_edit(before);
    }

    /// **Select All**: the whole canvas becomes the selection. One undo entry.
    ///
    /// ⚠️ **Substitui a lista de formas por UM `Raster` cheio**, e não por um retângulo paramétrico: um
    /// marquee do tamanho da tela carregaria um gizmo de transformação nas bordas do documento, e
    /// arrastá-lo por acidente encolheria a seleção "de tudo" sem que ninguém tivesse pedido.
    pub fn selection_select_all(&mut self) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        let crisp = vec![255u8; w * h];
        self.paint.selection_shapes = vec![SelectionEntry {
            shape: SelectionShape::Raster {
                crisp: Arc::new(crisp.clone()),
            },
            op: 0,
        }];
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// **Cut**: copiar os pixels selecionados e LIMPÁ-LOS, em UM passo de undo.
    ///
    /// ⚠️ **A metade que apaga tem de honrar a COBERTURA, não a máscara binarizada** — uma borda com
    /// feather 0,5 fica meio apagada, exatamente como o Copy a levou meio opaca. As duas metades leem a
    /// MESMA `selection_mask`, então o que sai é precisamente o que fica faltando.
    ///
    /// ⚠️ E o Copy **não** grava undo (é leitura); é por isso que o snapshot é tirado aqui, entre a
    /// cópia e a limpeza — um `selection_copy()` seguido de um `selection_erase()` daria a MESMA imagem
    /// e DOIS passos de undo, e o artista desfaria um Cut pela metade.
    pub fn selection_cut(&mut self) {
        if !self.selection_restricts_paint() {
            return;
        }
        self.selection_copy();
        if self.paint.selection_clipboard.is_none() {
            return; // nada coberto — o Copy recusou, e não há o que apagar
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let before = self.snapshot_model();
        let mask = Arc::clone(&self.paint.selection_mask);
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        let n = mask.len().min(buf.len() / 4);
        for i in 0..n {
            let cov = f32::from(mask[i]) / 255.0;
            if cov <= 0.0 {
                continue;
            }
            let b = i * 4;
            // Straight-alpha erase: a cobertura RETIRA alfa, e a cor fica onde está (o que sobra de
            // um texel meio cortado é a metade que não foi levada, com a cor dela).
            let a = f32::from(buf[b + 3]);
            buf[b + 3] = (a * (1.0 - cov)).round().clamp(0.0, 255.0) as u8;
        }
        self.mark_dirty(Region {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        });
        self.erase_relief(&mask);
        self.commit_structural_edit(before);
    }

    /// A metade do Cut que retira o **CORPO** — a cobertura recua pela mesma cobertura de seleção que
    /// retirou o alfa.
    ///
    /// ⚠️ **Sem isto o Cut deixa tinta INVISÍVEL com espessura** (medido: alfa 0 e o par
    /// `(altura 0,80, cobertura 255)` intacto), e a luz — que pesa a altura pela cobertura — desenha um
    /// sulco fantasma onde não há mais pigmento nenhum. É o mesmo fato do report do Copy/Paste, visto do
    /// outro lado: o corpo é metade da tinta, então quem leva a cor tem de levar o corpo.
    ///
    /// ⚠️ **A COBERTURA recua e a ALTURA fica**, a mesma assimetria de [`Self::copy_relief`] e do alfa:
    /// cobertura é *quanta tinta há ali* — a grandeza que a luz integra e que zera o relevo quando some —,
    /// e altura é *quão grossa é a que restou*. Afinar a metade não-cortada seria uma segunda coisa que
    /// ninguém pediu.
    fn erase_relief(&mut self, mask: &[u8]) {
        let (Some(layer), (w, h)) = (self.layers.active(), self.source_size) else {
            return;
        };
        let n = (w as usize) * (h as usize);
        let Some(entry) = self.covers.get_mut(&layer).filter(|p| p.len() == n) else {
            return; // sem corpo nesta camada não há corpo a retirar
        };
        let cov =
            super::plane_fork::fork_covers(entry, &self.undo.write_state, layer, (w, h), None);
        for (i, c) in cov.iter_mut().enumerate().take(n.min(mask.len())) {
            let k = f32::from(mask[i]) / 255.0;
            if k <= 0.0 {
                continue;
            }
            *c = (f32::from(*c) * (1.0 - k)).round().clamp(0.0, 255.0) as u8;
        }
        self.sync_relief_flags();
    }

    /// **Copy**: capture the selected pixels (coverage-premultiplied) into the in-memory clipboard. No undo
    /// entry (a read-only capture). No-op without a live selection.
    pub fn selection_copy(&mut self) {
        if !self.selection_restricts_paint() {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let mask = &self.paint.selection_mask;
        // Tight bbox of the covered texels.
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                if mask[y * w + x] > 0 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let (cw, ch) = (x1 - x0, y1 - y0);
        let mut rgba = vec![0u8; cw * ch * 4];
        for y in 0..ch {
            for x in 0..cw {
                let src = (y0 + y) * w + (x0 + x);
                let cov = f32::from(mask[src]) / 255.0;
                let s = src * 4;
                let d = (y * cw + x) * 4;
                for c in 0..3 {
                    rgba[d + c] = self.canvas_rgba[s + c];
                }
                // Premultiply the source alpha by coverage so Paste blends the selected shape only.
                rgba[d + 3] = (f32::from(self.canvas_rgba[s + 3]) * cov)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
        }
        let rect = Region {
            x: x0 as u32,
            y: y0 as u32,
            w: cw as u32,
            h: ch as u32,
        };
        let relief = self.copy_relief(rect, mask);
        self.paint.selection_clipboard = Some(SelectionClip { rect, rgba, relief });
    }

    /// **O CORPO da tinta copiada** — os três planos do impasto recortados no mesmo `rect`, ou `None`
    /// quando a camada não tem relevo (os planos são *lazy*: 12 B/px alocados só quando alguém pinta).
    ///
    /// ⚠️ **A cobertura é pré-multiplicada pela seleção e a ALTURA não**, e a assimetria é a mesma do
    /// RGBA: lá a cor fica verbatim e o ALFA é escalado. Cobertura é *quanta tinta há ali* — a grandeza
    /// pela qual a luz pesa —, então uma borda meio-selecionada leva meia tinta e a peça feather-a como a
    /// cor. A altura é *quão grossa é a tinta que há ali*: escalá-la faria a borda da peça AFINAR além de
    /// desbotar, que é uma segunda coisa que ninguém pediu.
    ///
    /// Os três são recortados juntos ou nenhum: ver [`ReliefClip`].
    fn copy_relief(&self, rect: Region, mask: &[u8]) -> Option<ReliefClip> {
        let layer = self.layers.active()?;
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        let src_h = self.heights.get(&layer).filter(|p| p.len() == n)?;
        let src_c = self.covers.get(&layer).filter(|p| p.len() == n)?;
        let src_m = self.mats.get(&layer).filter(|p| p.len() == n)?;
        let (cw, ch) = (rect.w as usize, rect.h as usize);
        let mut heights = vec![0.0f32; cw * ch];
        let mut covers = vec![0u8; cw * ch];
        let mut mats = vec![[0u8; 7]; cw * ch];
        for y in 0..ch {
            for x in 0..cw {
                let s = (rect.y as usize + y) * w + rect.x as usize + x;
                let d = y * cw + x;
                let cov = f32::from(mask[s]) / 255.0;
                heights[d] = src_h[s];
                covers[d] = (f32::from(src_c[s]) * cov).round().clamp(0.0, 255.0) as u8;
                mats[d] = src_m[s];
            }
        }
        // Uma peça sem tinta nenhuma não é relevo, é um retângulo de zeros que o composite teria de
        // aprender a ignorar — a recusa aqui é o que mantém o caminho digital byte-idêntico.
        covers.iter().any(|&c| c > 0).then_some(ReliefClip {
            heights,
            covers,
            mats,
        })
    }

    /// **Paste**: arma a peça FLUTUANTE do clipboard sobre a camada ativa, no lugar de origem, com o
    /// gizmo de transformação vivo. **Enter aplica** ([`PainterTool::paste_commit`]), **Esc descarta**
    /// ([`PainterTool::paste_cancel`]). No-op com o clipboard vazio.
    ///
    /// ⚠️ **MUDANÇA DE COMPORTAMENTO (Enio, 2026-08-07):** antes ele compositava na hora e gravava undo
    /// ali mesmo — colar e querer mover custava um Ctrl+Z. Agora nada é commitado até o Enter, e é por
    /// isso que o Esc não precisa de undo nenhum: não há o que desfazer.
    pub fn selection_paste(&mut self) {
        self.arm_paste_patch();
    }
}
