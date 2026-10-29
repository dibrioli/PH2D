//! The wet COMPOSITE half (child of [`super`] — workspace file-LOC cap):
//! pigment (with the doc-22 visual terms) OVER the frozen base, the
//! Selection/protection/alpha-lock gates, and the show-wet veil.

use super::*;
// The row fan-out and the engine's per-row door live HERE and nowhere else, so the imports do too: the
// parent was three lines over the workspace LOC cap carrying names only this file uses.
#[cfg(test)]
use super::composite_contagem as contagem;
use ph2d_wet_paint::render::{RenderLayer, render_pigment_row_visual};
use rayon::prelude::*;

impl PainterTool {
    /// Composite the engine's dirty rect: pigment (straight alpha) OVER the
    /// frozen base, written into `canvas_rgba`, preview marked.
    ///
    /// Selection / protection / alpha-lock (W2.5) are enforced HERE, at the
    /// single place the module writes the canvas: the fluid flows wherever
    /// the SIM takes it, and the gated texels keep-lerp back to the frozen
    /// base (`splat_keep` — the watercolor composite's exact law, a hard
    /// guarantee independent of where the water reached). Alpha-lock pins
    /// the layer's α to the frozen base (colour deposits into existing
    /// paint; the silhouette never moves). All gates off = byte-identical.
    pub(super) fn wetpaint_composite(&mut self) {
        self.wetpaint_composite_veiled(self.paint.wetpaint.show_wet);
    }

    /// [`Self::wetpaint_composite`] with the show-wet veil explicit — the
    /// end-session door forces `false` so the veil never bakes.
    pub(super) fn wetpaint_composite_veiled(&mut self, veil: bool) {
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
        let (gate_sel, gate_prot, gate_alock) = self.wet_splat_gates();
        let camada = self.layers.active().filter(|&id| !self.layers.is_mask(id));
        #[cfg(test)]
        let camada = camada.filter(|_| !self.sem_vidro);
        let visual = ph2d_wet_paint::render::PigmentVisual {
            // A Paint Texture imprime o PAPEL: sem papel escolhido não há o que imprimir (dono 10/10). É do TRAÇO (dono
            // 28/10): QUEM imprime é a célula (`grid::aparencia::textura`), nunca a caixa de agora.
            paper: self.paint.brush.paper.is_active(),
            // O relevo da tinta é da tinta (dono 11/10): o knob Emboss decide, com ou sem papel.
            relevo: true,
        };
        // O raio do Smooth Edges (o slider `0..1` × `RAIO_MAX`, px); `0` = desligado. É da TELA (D1 W3b).
        let raio = self.paint.wetpaint.smooth_edges * super::smooth_edges::RAIO_MAX;
        let suave = raio > 0.0;
        // As linhas que o Smooth Edges procura dentro da tinta: as da GRAVIDADE (o eixo mais perto dela).
        let wp = &self.paint.wetpaint;
        let linhas = (wp.tilt_on && wp.tilt_ring > 0).then(|| {
            let [gx, gy] = ph2d_wet_paint::sim::tilt_dir_for_spoke(wp.tilt_spoke);
            gy.abs() >= gx.abs()
        });
        // A água do artista (W12): a cobertura de cada texel escrito é o `pa` — exacta quando a tinta é
        // composta «por cima» da tela congelada (Mix, a camada única, sem vidro nem alfa preso); fora disso
        // o texel pintado fica cheio.
        let marca = wp.session.as_ref().and_then(|s| s.marca);
        let marca = marca.filter(|_| self.undo.marca.molhada_viva(marca));
        let exata = marca.is_some()
            && wp.session.as_ref().map(|s| s.blend) == Some(ph2d_painter_brush::BrushBlend::Mix)
            && gate_alock.is_none()
            && camada.is_none_or(|c| !self.vidros.contains_key(&c))
            && self.preview_layer_stack_is_trivial()
            && !self.papel_atravessa_vidro();
        let oleo = self.oleo_do_composite(raio);
        // W3b: o dente do papel que a Transmission assa na tinta ([`super::grao_no_filme`]); sem papel escolhido, o
        // papel de fábrica (U2).
        let grao = oleo.and_then(|_| {
            super::grao_no_filme::GraoNoFilme::de(
                super::grao_no_filme::papel_do_filme(
                    self.paint.brush.paper,
                    self.paint.substrate_depth,
                    self.paint.substrate_rough,
                    self.paint.paper_image.as_ref().map(|i| i.as_mask()),
                ),
                &self.paint.impasto_rig,
                (w, h),
            )
        });
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        sess.bring_home();
        // A tela fica a mostrar a água NESTE estado (W14: onde ela pára se a sessão acabar já).
        self.paint.wetpaint.composta = Some(fita::MarcaDaAgua::de(&sess.engine));
        // ⚠️ **Células e pixels deixaram de ser a mesma coisa** ([`grid_map`]):
        // a grade tem `ratio` pixels de canvas por célula. O dirty do motor é
        // em CÉLULAS e é clampado à GRADE; o retângulo que este composite
        // escreve — e que o `mark_dirty` publica — é em PIXELS.
        let ratio = sess.ratio;
        let (gw, gh) = sess.grid;
        let (cx0, cy0, cx1, cy1) = match sess.engine.take_dirty() {
            Dirty::Clean => return,
            Dirty::Full => (1, 1, gw, gh),
            Dirty::Rect { x0, y0, x1, y1 } => (
                (x0.max(1) as usize).min(gw),
                (y0.max(1) as usize).min(gh),
                (x1.max(1) as usize).min(gw),
                (y1.max(1) as usize).min(gh),
            ),
        };
        #[cfg(test)]
        let area_motor = (cx1 + 1).saturating_sub(cx0) * (cy1 + 1).saturating_sub(cy0);
        // Óleo Real: a fenda e a sombra de uma célula leem a tinta à volta — a região cresce pelo alcance.
        let (cx0, cy0, cx1, cy1) = match &oleo {
            Some(o) => {
                let grades: Vec<_> = sess
                    .engine
                    .layers
                    .iter()
                    .map(|l| (&l.grid, l.visible))
                    .collect();
                let a = ph2d_wet_paint::oleo_real::alcance_na(
                    o,
                    &grades,
                    usize::from(ratio),
                    (cx0, cy0, cx1, cy1),
                );
                (
                    cx0.saturating_sub(a).max(1),
                    cy0.saturating_sub(a).max(1),
                    (cx1 + a).min(gw),
                    (cy1 + a).min(gh),
                )
            }
            None => (cx0, cy0, cx1, cy1),
        };
        if cx1 < cx0 || cy1 < cy0 || sess.base.len() != w * h * 4 {
            return;
        }
        #[cfg(test)]
        contagem::nota(area_motor, (cx1 + 1 - cx0) * (cy1 + 1 - cy0));
        let (px0, py0, px1, py1) = grid_map::cell_rect_to_px((cx0, cy0, cx1, cy1), ratio, w, h);
        if px1 <= px0 || py1 <= py0 {
            return;
        }
        // Smooth Edges: um pixel até ao alcance do raio também muda, logo o retângulo cresce por ele.
        let (px0, py0, px1, py1) = if suave {
            let a = super::smooth_edges::alcance(raio);
            (
                px0.saturating_sub(a),
                py0.saturating_sub(a),
                (px1 + a).min(w),
                (py1 + a).min(h),
            )
        } else {
            (px0, py0, px1, py1)
        };
        // A marca do artista (W9): o que a água muda aqui não é do artista — guarda-se a caixa como está.
        let agua_antes: Option<Vec<u8>> = self.undo.marca.ativa().then(|| {
            let mut v = Vec::with_capacity((px1 - px0) * (py1 - py0) * 4);
            for y in py0..py1 {
                v.extend_from_slice(&self.canvas_rgba[(y * w + px0) * 4..(y * w + px1) * 4]);
            }
            v
        });
        let layers: Vec<RenderLayer<'_>> = sess
            .engine
            .layers
            .iter()
            .map(|l| RenderLayer {
                grid: &l.grid,
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        // The visual terms (doc 22): the paper printed into the pigment and
        // the K–M glaze stacking ride the SAME single render body; both off
        // is the historical pigment render, byte for byte (engine-gated).
        //
        // ROW-PARALLEL ([ADR-0109](../../../../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md),
        // amendment 2): the same shape of work the ADR already sanctioned for
        // the watercolor optical composite — a pure per-pixel map whose rows
        // are independent, in the crate the ADR already opened. The ENGINE
        // stays dependency-free and spawns nothing; it exposes one row
        // (`render_pigment_row_visual`) and the fan-out lives here.
        //
        // Byte-identical by construction: every row computes exactly what the
        // serial loop computed, and rows never read each other's output.
        let params = sess.engine.sim.gather_params(&sess.engine.tuning);
        #[cfg(test)]
        let t_engine = std::time::Instant::now();
        let stride = w * 4;
        // O plano de pigmento é da GRADE (uma linha por linha de célula), e é
        // por isso que uma razão maior corta o render de pigmento por `ratio²`
        // — é o mesmo trabalho por célula, e há `ratio²` vezes menos delas.
        let g_stride = gw * 4;
        #[cfg(test)]
        let t_campo = std::time::Instant::now();
        // As duas curas de custo (27/10, bit a bit): o campo em faixas pelo rayon e a sombra no papel das células
        // PINTADAS tirada dos termos que a cor delas já calculou ([`super::oleo_plano`]).
        #[cfg(test)]
        let (sombra_antiga, campo_em_serie) = super::oleo_plano::curas_desligadas::get();
        #[cfg(not(test))]
        let (sombra_antiga, campo_em_serie) = (false, false);
        // O campo guardado por DEPENDÊNCIA (`oleo_real::CacheCampo`): a mesma chave e as mesmas grades reusam-no.
        let revisao = sess.engine.revisao();
        let chave = oleo.map(|o| {
            let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
            ph2d_wet_paint::oleo_real::ChaveCampo::de(&o, &grades, usize::from(ratio))
        });
        #[cfg(test)]
        let sem_cache = super::oleo_plano::sem_cache_do_campo::get();
        #[cfg(not(test))]
        let sem_cache = false;
        let guardado = match (&oleo, &chave) {
            (Some(o), Some(k)) if !sem_cache => ph2d_wet_paint::oleo_real::CacheCampo::serve(
                sess.campo_cache.as_ref(),
                k,
                revisao,
                o,
                (cy0, cy1),
            ),
            _ => None,
        };
        let proprio = oleo.filter(|_| guardado.is_none()).map(|o| {
            let grades: Vec<_> = layers.iter().map(|l| (l.grid, l.visible)).collect();
            let janela = (cx0, cy0, cx1, cy1);
            if campo_em_serie {
                ph2d_wet_paint::oleo_real::CampoAltura::de_com(
                    &ph2d_wet_paint::oleo_real::EmSerie,
                    &grades,
                    &o,
                    usize::from(ratio),
                    janela,
                )
            } else {
                ph2d_wet_paint::oleo_real::CampoAltura::de_com(
                    &super::oleo_plano::EmFaixas,
                    &grades,
                    &o,
                    usize::from(ratio),
                    janela,
                )
            }
        });
        #[cfg(test)]
        if proprio.is_some() {
            super::oleo_plano::sem_cache_do_campo::conta();
        }
        let campo = oleo.zip(guardado.or(proprio.as_ref()));
        #[cfg(test)]
        contagem::campo(t_campo.elapsed());
        if campo.is_some() && sess.sombra.len() != gw * gh {
            sess.sombra = vec![0u8; gw * gh];
        }
        // `feita[k·gw + cx − 1]`: a sombra da célula já veio da linha da tinta.
        let mut feita = if campo.is_some() && !sombra_antiga {
            vec![false; gw * (cy1 + 1 - cy0)]
        } else {
            Vec::new()
        };
        #[cfg(test)]
        let t_pig = std::time::Instant::now();
        let pig_rows = sess.pigment[(cy0 - 1) * g_stride..cy1 * g_stride].par_chunks_mut(g_stride);
        if let (Some((o, c)), false) = (&campo, feita.is_empty()) {
            pig_rows
                .zip(sess.sombra[(cy0 - 1) * gw..cy1 * gw].par_chunks_mut(gw))
                .zip(feita.par_chunks_mut(gw))
                .enumerate()
                .for_each(|(k, ((row, srow), frow))| {
                    ph2d_wet_paint::render::render_pigment_row_oleo_com_sombra(
                        Some(&params),
                        &layers,
                        visual,
                        o,
                        c,
                        cy0 + k,
                        cx0,
                        cx1,
                        row,
                        srow,
                        frow,
                    );
                });
        } else {
            pig_rows.enumerate().for_each(|(k, row)| match &campo {
                Some((o, c)) => ph2d_wet_paint::render::render_pigment_row_oleo(
                    Some(&params),
                    &layers,
                    visual,
                    o,
                    c,
                    cy0 + k,
                    cx0,
                    cx1,
                    row,
                ),
                None => render_pigment_row_visual(
                    Some(&params),
                    &layers,
                    visual,
                    cy0 + k,
                    cx0,
                    cx1,
                    row,
                ),
            });
        }
        drop(layers);
        #[cfg(test)]
        let t_pig = t_pig.elapsed();
        #[cfg(test)]
        let t_som = std::time::Instant::now();
        let caixa = (cx0, cy0, cx1, cy1);
        super::composite_oleo::sombra_do_papel(&mut sess.sombra, &campo, &feita, caixa, (gw, gh));
        super::composite_oleo::grao_das_celulas(&mut sess.grao, &campo, &grao, caixa, (gw, gh));
        let sem_oleo = super::composite_oleo::tinta_sem_oleo(
            &sess.engine,
            &campo,
            (suave, raio),
            ratio,
            caixa,
            (gw, gh),
        );
        if oleo.is_none() {
            // Sem tinta com óleo na tela o campo guardado só ocupava memória (~28 B por célula, 230 MB a 4K).
            sess.campo_cache = None;
        }
        if let (Some(c), Some(k)) = (proprio.as_ref(), chave) {
            ph2d_wet_paint::oleo_real::CacheCampo::guarda(
                &mut sess.campo_cache,
                k,
                revisao,
                c,
                (gw, gh),
                (cx0, cy0, cx1, cy1),
            );
        }
        #[cfg(test)]
        contagem::tempos(t_pig, t_som.elapsed());
        // ⚠️ **A FRONTEIRA DO MOTOR.** Tudo acima lê `sess.engine` (as camadas, o
        // `params`, o dirty); tudo abaixo lê `sess.pigment`, `sess.base` e o
        // `canvas` — **e mais nada do motor**, exceto o véu no fim.
        //
        // ⛔ **MEDIDO E REJEITADO — não refaça: devolver o motor ao worker AQUI.**
        // O log do Enio (2026-08-01, janela assistindo) mostra `worker: busy 67 %
        // **away 24 %** sleep 9 %` com a água a 34,0 Hz contra 40 nominais, e o
        // `split` abaixo diz que **55,8 % do composite não toca o motor** na razão
        // 1. A conclusão óbvia — *o worker está parado à toa, devolva antes* — foi
        // construída (uma porta `wetpaint_composite_releasing` para o tick, com o
        // véu vetando) e **o relógio a negou**: A/B costas-com-costas em máquina
        // calma, três amostras cada, `heavy_puddle` a 4096²:
        //
        // | | água | tick p50 | tick max |
        // |---|---|---|---|
        // | sem | 35,6 / 36,2 / 36,1 Hz | 4,1-5,5 | **12,9 / 15,9 / 14,3 ms** |
        // | com | 36,6 / 36,6 / 36,9 Hz | 4,1-5,8 | **20,5 / 16,7 / 27,5 ms** |
        //
        // **+0,6 Hz de água (2 %) ao preço de +50 % no PIOR TICK** — um hitch que
        // o artista vê trocado por uma taxa que ele não distingue.
        //
        // ⚠️ **O mecanismo, e é ele que vale mais que o número:** o `away` **não é
        // núcleo ocioso**. As duas metades já são row-parallel (ADR-0109 no
        // composite, ADR-0145/0147 no solver) e saturam os 32 núcleos, então
        // sobrepô-las não cria capacidade — só move a contenção para DENTRO do
        // frame. *Um balde de espera só é oportunidade quando o recurso que ele
        // espera está parado.*
        #[cfg(test)]
        split::note_engine(t_engine.elapsed());
        #[cfg(test)]
        let t_pixel = std::time::Instant::now();
        // Straight-alpha OVER the frozen base, cell (cx,cy) → pixel (cx-1,cy-1).
        let gsel: Option<&[u8]> = gate_sel.as_deref().map(Vec::as_slice);
        let gprot: Option<&[u8]> = gate_prot.as_deref().map(Vec::as_slice);
        let gate_on = gsel.is_some() || gprot.is_some();
        let alock_on = gate_alock.is_some();
        // O VIDRO de uma aguada por baixo (doc 48): o fluido cobre-a por `pa` e cada canal atenua-se
        // ([`crate::compositor::vidro::cobre`]), a partir do vidro da base congelada da sessão. Só numa
        // camada que JÁ tem vidro: sem aguada o alfa único é exacto, e o plano não nasce aqui.
        let camada = camada.filter(|c| self.vidros.contains_key(c));
        let vidro = camada.and_then(|c| {
            super::watercolor_vidro::abre_o_vidro(
                &mut self.vidros,
                &mut self.vidro_congelado.molhada,
                c,
                w * h,
                &sess.base,
            )
            .map(|(p, b)| (c, p, b))
        });
        let (vcamada, mut vplano, vbase) = match vidro {
            Some((c, p, b)) => (Some(c), Some(p), Some(b)),
            None => (None, None, None),
        };
        let vbase: Option<&[crate::compositor::vidro::Vidro]> = vbase.as_deref().map(Vec::as_slice);
        let vlinhas: Vec<Option<&mut [crate::compositor::vidro::Vidro]>> = match vplano.as_mut() {
            Some(p) => Arc::get_mut(p).expect("o plano é só deste quadro")[py0 * w..py1 * w]
                .chunks_mut(w)
                .map(Some)
                .collect(),
            None => (py0..py1).map(|_| None).collect(),
        };
        let canvas = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        // ROW-PARALLEL, pelo MESMO ADR-0109 e pela mesma razão do render acima: um map puro por-pixel
        // cujas linhas são independentes — cada uma lê `pigment`/`base` no seu próprio offset e escreve
        // só o seu, e nenhuma lê a saída de outra. **Byte-idêntico por construção**: muda qual thread
        // avalia qual linha, nunca o que a linha diz.
        //
        // ⚠️ **Ele era o que sobrava serial no tick**, e o tick é O(ÁREA MOLHADA) — medido a 4096²,
        // traço de 3600 px: **sim 10,06 ms + composite 6,11**, com 3% da tela molhada. Um frame de
        // 60 fps inteiro. O `WET_MAX_STEPS` fechou o LAÇO (a contagem); isto ataca o custo por passo.
        //
        // A metade da SIM continua serial e assim fica: o engine é port 1:1 com fingerprint pinado e o
        // ADR-0134 declara o solver serial POR SEMÂNTICA — o ADR-0109 é inaplicável lá, de propósito.
        let (pigment, base, sombra) = (&sess.pigment, &sess.base, &sess.sombra);
        let grao_cel = &sess.grao;
        let rr = usize::from(ratio);
        let modo = sess.blend;
        let base_da_marca = marca.map(|_| Arc::clone(&sess.base));
        let cw = px1 - px0;
        let mut cob: Vec<u16> = if marca.is_some() {
            vec![0; cw * (py1 - py0)]
        } else {
            Vec::new()
        };
        let cob_linhas: Vec<Option<&mut [u16]>> = if marca.is_some() {
            cob.chunks_mut(cw).map(Some).collect()
        } else {
            (py0..py1).map(|_| None).collect()
        };
        let vazio = sess.vazio; // L7: o modo de cor sobre o vazio pousa a cor (W3C geral)
        // O amostrador do upsample. ⚠️ A pergunta *"um pixel É uma célula?"* é
        // feita UMA vez, fora do laço: na razão 1 o corpo abaixo lê os quatro
        // bytes no MESMO offset que sempre leu, com a MESMA aritmética de
        // `f32`, então aquele caminho é byte-idêntico ao que shipava — não por
        // sorte numérica, por ser literalmente a mesma expressão.
        let sampler = grid_map::SampleU::new(ratio, gw, gh);
        let one_cell_per_pixel = sampler.is_identity();
        let orla = oleo.is_none();
        #[cfg(test)]
        let orla = orla || super::smooth_edges::so_na_cor::get();
        #[cfg(test)]
        let suave = suave && !super::smooth_edges::sem_suave::get();
        let suavizada = suave.then(|| {
            super::smooth_edges::Suave::nova(
                sampler,
                (pigment, stride),
                (w, h),
                (px0, py0, px1, py1),
                raio,
                linhas,
                orla,
            )
        });
        canvas[py0 * stride..py1 * stride]
            .par_chunks_mut(stride)
            .zip(vlinhas.into_par_iter())
            .zip(cob_linhas.into_par_iter())
            .enumerate()
            .for_each(|(k, ((row, mut vrow), mut crow))| {
                let py = py0 + k;
                let gb = py * stride;
                // A metade-`y` do amostrador, resolvida UMA vez por linha.
                let srow = sampler.row(py);
                for px in px0..px1 {
                    let lo = px * 4; // dentro da linha
                    let o = gb + lo; // global, para os planos e os gates
                    // O pigmento, em premultiplicado normalizado (`pc * pa`) —
                    // a forma em que o `over` abaixo já o consumia.
                    let dele = if one_cell_per_pixel {
                        super::smooth_edges::premul_da_celula(pigment, o)
                    } else {
                        super::smooth_edges::premul_amostrado(srow.at(pigment, px))
                    };
                    // Smooth Edges: o pigmento que o pixel mostra, com a cobertura da borda alisada (a
                    // fração na COBERTURA premultiplicada — exacta também sobre uma camada transparente).
                    let (premul, pa) = match (&suavizada, &sem_oleo) {
                        (Some(sv), Some((sx, sy, sw, so))) => {
                            let (cx, cy) = ((px / rr).min(gw - 1) + 1, (py / rr).min(gh - 1) + 1);
                            let sem = cx >= *sx
                                && cy >= *sy
                                && so.get((cy - sy) * sw + cx - sx).copied().unwrap_or(false);
                            sv.aplica_com(px, py, dele, orla || sem)
                        }
                        (Some(sv), None) => sv.aplica(px, py, dele),
                        (None, _) => dele,
                    };
                    // W3b: o dente do papel na tinta (Relevo do papel, Transmission); `d = 0` ao bit.
                    let premul = match &grao {
                        Some(gf) if pa > 0.0 && !grao_cel.is_empty() => {
                            let d = grao_cel[(py / rr).min(gh - 1) * gw + (px / rr).min(gw - 1)];
                            if d == 0 {
                                premul
                            } else {
                                let k = pa * 255.0;
                                gf.sombreia(px, py, f32::from(d) / 255.0, premul.map(|v| v / k))
                                    .map(|v| v * k)
                            }
                        }
                        _ => premul,
                    };
                    let escuro = if !sombra.is_empty() && vrow.is_none() {
                        // §BUG-bordas-W3b: sem o furo de uma célula (o pontinho escuro da orla).
                        ph2d_wet_paint::oleo_real::sombra_sem_furo(
                            sombra,
                            (gw, gh),
                            ((px / rr).min(gw - 1), (py / rr).min(gh - 1)),
                        )
                    } else {
                        0
                    };
                    // A sombra do Óleo Real sobre o que está por baixo: preto por cima, com a fracção
                    // `escuro` — pelas mesmas gates que a tinta (selecção/protecção, alfa preso). Sob a orla
                    // da tinta ela é o papel sobre o qual o pigmento se compõe.
                    let sombreada: [u8; 4];
                    let base_px: &[u8] = if escuro > 0 {
                        let mut s = f32::from(escuro) / 255.0;
                        if gate_on {
                            s *= super::watercolor_accum::splat_keep(gsel, gprot, None, o / 4);
                        }
                        let ba = f32::from(base[o + 3]) / 255.0;
                        let oa = if alock_on { ba } else { s + ba * (1.0 - s) };
                        let mut b = [0u8; 4];
                        for ch in 0..3 {
                            let bc = f32::from(base[o + ch]);
                            b[ch] = if oa > 0.0 {
                                (bc * ba * (1.0 - s) / oa).round().clamp(0.0, 255.0) as u8
                            } else {
                                0
                            };
                        }
                        b[3] = (oa * 255.0).round() as u8;
                        if pa <= 0.0 {
                            row[lo..lo + 4].copy_from_slice(&b);
                            continue;
                        }
                        sombreada = b;
                        &sombreada
                    } else {
                        &base[o..o + 4]
                    };
                    if pa <= 0.0 {
                        // Base verbatim — the gates keep-lerp TOWARD the base, so
                        // they are exact no-ops on this branch.
                        row[lo..lo + 4].copy_from_slice(&base[o..o + 4]);
                        if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase) {
                            v[px] = b[o / 4];
                        }
                        continue;
                    }
                    let mut cobertura = pa;
                    if modo == ph2d_painter_brush::BrushBlend::Mix {
                        let ba = base_px[3] as f32 / 255.0;
                        let oa = pa + ba * (1.0 - pa);
                        for ch in 0..3 {
                            let bc = base_px[ch] as f32;
                            row[lo + ch] = ((premul[ch] + bc * ba * (1.0 - pa)) / oa).round() as u8;
                        }
                        row[lo + 3] = (oa * 255.0).round() as u8;
                    } else {
                        // O modo da SESSÃO pela lei do traço digital (`blend_over`, sRGB straight):
                        // a cor straight do pigmento sobre a base, pela cobertura `pa`.
                        let inv = 1.0 / (pa * 255.0);
                        let cor = [premul[0] * inv, premul[1] * inv, premul[2] * inv];
                        let dst = [
                            f32::from(base_px[0]) / 255.0,
                            f32::from(base_px[1]) / 255.0,
                            f32::from(base_px[2]) / 255.0,
                            f32::from(base_px[3]) / 255.0,
                        ];
                        let out = ph2d_painter_brush::blend_over_modo(modo, dst, cor, pa, vazio);
                        for c in 0..4 {
                            row[lo + c] = (out[c] * 255.0).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                    if gate_on {
                        let keep = super::watercolor_accum::splat_keep(gsel, gprot, None, o / 4);
                        if keep < 1.0 {
                            cobertura *= keep;
                            for c in 0..4 {
                                let painted = f32::from(row[lo + c]);
                                let orig = f32::from(base[o + c]);
                                row[lo + c] = (painted * keep + orig * (1.0 - keep))
                                    .round()
                                    .clamp(0.0, 255.0)
                                    as u8;
                            }
                        }
                    }
                    if alock_on {
                        // Alpha-lock: the α reference IS `sess.base` (the gate's
                        // buffer, served by `wet_splat_gates`' wet-session arm).
                        row[lo + 3] = base[o + 3];
                    }
                    if let Some(c) = crow.as_deref_mut() {
                        c[px - px0] = if exata {
                            (cobertura.clamp(0.0, 1.0) * f32::from(u16::MAX)).round() as u16
                        } else {
                            u16::MAX
                        };
                    }
                    if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase)
                        && alock_on
                    {
                        // Com o alfa preso à base a cobertura não muda: o texel novo, os alfas da base.
                        let px4 = [row[lo], row[lo + 1], row[lo + 2], row[lo + 3]];
                        v[px] = crate::compositor::vidro::sela(
                            px4,
                            crate::compositor::vidro::alfas(&base[o..o + 4], &b[o / 4]),
                        );
                    } else if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase) {
                        let antes = crate::compositor::vidro::alfas(&base[o..o + 4], &b[o / 4]);
                        let px4 = [row[lo], row[lo + 1], row[lo + 2], row[lo + 3]];
                        v[px] = crate::compositor::vidro::sela(
                            px4,
                            crate::compositor::vidro::cobre(antes, cobertura),
                        );
                    }
                }
            });
        if let (Some(c), Some(p)) = (vcamada, vplano) {
            self.vidros.insert(c, p);
        }
        #[cfg(test)]
        split::note_pixel(t_pixel.elapsed());
        if veil {
            let g = sess.engine.active_grid();
            let caixa = (px0, py0, px1, py1);
            super::veu::pinta_o_veu(g, canvas, (w, usize::from(ratio)), (gw, gh), caixa);
        }
        // Re-arm the guard with the allocation our own make_mut may have
        // re-seated (weak — [`WetSession::canvas`]).
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        // A região é a de PIXELS que este composite de fato escreveu (o dirty
        // do motor é em células; publicar aquele retângulo verbatim sob razão
        // > 1 declararia sujo `ratio²` menos pixels do que mudaram).
        let region = Region {
            x: px0 as u32,
            y: py0 as u32,
            w: (px1 - px0) as u32,
            h: (py1 - py0) as u32,
        };
        // ⭐ **E a mesma região DECLARA a escrita** (`crate::undo::window`): o `fork_canvas` acima abriu
        // um acesso, e as duas passadas que escrevem — o composite e o véu — só tocam `px0..px1 ×
        // py0..py1`. Sem esta linha o acesso ficava aberto e o histórico varria a tela inteira no
        // commit do traço E na absorção do escorrido (medido 2026-09-24: `16–26` acessos por traço
        // sem declaração, todos daqui). Em DEBUG o `split` confere que a janela verdadeira cabe nesta.
        self.declare_wrote(Some(region));
        self.mark_dirty(region);
        if let Some(antes) = agua_antes {
            self.undo
                .marca
                .agua_mudou(&antes, &self.canvas_rgba, w, (px0, py0, px1, py1));
        }
        if let Some(b) = base_da_marca {
            let caixa = (px0 as u32, py0 as u32, px1 as u32, py1 as u32);
            self.undo.marca.agua_cobre(marca, caixa, &cob, &b);
        }
    }
}

/// **O SPLIT DO COMPOSITE, medido no código que SHIPA** (só sob `cfg(test)`).
///
/// A pergunta que ele responde é de PRODUTO, não de curiosidade: o tick segura o
/// motor durante o composite inteiro e só o devolve no `hand_off_sim` do fim, então
/// **a metade que não precisa do motor é tempo em que o worker fica parado à toa** —
/// o balde `away` do log do Enio (24 % da janela dele, 2026-08-01).
///
/// ⚠️ **Isto é instrumentação do PRODUTO, não uma sonda com laço próprio** — a
/// distinção que a §5.11 e a §5.44 pagaram: um laço meu ficaria cego à porta, e
/// atribuir a uma LINHA dentro de um bloco é inferência de segunda ordem. Aqui os
/// dois blocos já são separados pela fronteira do motor, e o relógio só diz de que
/// lado dela o tempo caiu.
#[cfg(test)]
pub(in crate::tool::paint) mod split;
