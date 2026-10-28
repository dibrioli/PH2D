//! **A recomposição que o PAINEL pede corre no QUADRO** (smoke do dono 27/10: arrastar um número do Real Oil
//! «calculando mil vezes por segundo»). O arrasto manda um `SetValue` por evento do rato e cada um recompunha a
//! tela inteira ali mesmo. Agora o valor só fica guardado e a tela marcada; o tick varre-a em FAIXAS de linhas
//! com o último valor, dentro de [`ORCAMENTO_MS`] por quadro, e a varredura que um valor novo reinicia continua
//! de onde ia. Quando ela fecha, a tela é a da recomposição inteira, ao byte (gate em `oleo_arrasto_tests`).
//! Medição: `docs/Painter/49_realshader/ESTADO_MISSAO_W3.md`, linha «arrasto do cartão».

use super::*;

/// O tempo de óleo que um quadro paga durante o arrasto. Recurso: o quadro de 60 Hz (16,7 ms) — metade fica
/// para o resto do quadro (painel, envio da tela à placa).
const ORCAMENTO_MS: f64 = 8.0;
/// A primeira faixa de uma tela cujo custo ainda não se mediu.
const FAIXA_INICIAL: usize = 32;
/// A faixa mais fina.
const FAIXA_MIN: usize = 8;
/// A faixa nunca é mais fina que `ALCANCES ×` o alcance do óleo: a região de cada faixa cresce por ele dos dois
/// lados, e a varredura inteira custa `1 + 2/ALCANCES` recomposições inteiras (medido: faixas de ~8 linhas com o
/// alcance de ~32 custavam 11× uma recomposição inteira a 4K).
const ALCANCES: usize = 4;

/// A varredura pendente (linhas de CÉLULA, `1..=gh`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Varredura {
    /// A próxima linha a recompor.
    cursor: usize,
    /// Quantas linhas faltam para a tela inteira mostrar o último valor (`0` = nada pendente).
    faltam: usize,
    /// O custo medido da última faixa, por linha PINTADA (`0` = por medir).
    ms_por_linha: f64,
}

#[cfg(test)]
thread_local! {
    /// `Some(n)`: faixas de `n` linhas fixas (gates deterministas).
    static FAIXA_FIXA: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    /// `true` = o caminho de antes: o painel recompõe a tela inteira no próprio evento (sondas e mutação).
    static NO_EVENTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::tool::paint) fn faixa_fixa(n: Option<usize>) {
    FAIXA_FIXA.with(|c| c.set(n));
}

#[cfg(test)]
pub(in crate::tool::paint) fn no_evento(sim: bool) {
    NO_EVENTO.with(|c| c.set(sim));
}

impl PainterTool {
    /// A porta do PAINEL: a óptica mudou (Real Oil, o Smooth Edges da tela) — a tela inteira recompõe-se nos próximos
    /// quadros, com o último valor que houver então.
    pub(in crate::tool::paint) fn wet_recompoe_no_quadro(&mut self) {
        #[cfg(test)]
        if NO_EVENTO.with(std::cell::Cell::get) {
            self.wet_recomposite_full();
            return;
        }
        let Some(gh) = self.paint.wetpaint.session.as_ref().map(|s| s.grid.1) else {
            return;
        };
        let v = &mut self.paint.wetpaint.varredura;
        if v.faltam == 0 || v.cursor == 0 || v.cursor > gh {
            v.cursor = 1;
        }
        v.faltam = gh;
    }

    /// Há tela por recompor (a varredura não fechou)?
    #[cfg(test)]
    #[must_use]
    pub(in crate::tool::paint) fn oleo_por_compor(&self) -> bool {
        self.paint.wetpaint.varredura.faltam > 0 && self.paint.wetpaint.session.is_some()
    }

    /// A recomposição inteira acabou de correr: nada fica pendente.
    pub(in crate::tool::paint) fn wet_varredura_feita(&mut self) {
        self.paint.wetpaint.varredura.faltam = 0;
    }

    /// O passo do TICK: uma faixa da varredura, do tamanho que cabe em [`ORCAMENTO_MS`]. Sem o motor em casa
    /// (o worker a meio de um passo) não espera — o quadro seguinte tenta.
    pub(in crate::tool::paint) fn wet_varre_oleo(&mut self) {
        let v = self.paint.wetpaint.varredura;
        if v.faltam == 0 {
            return;
        }
        let facts = self.wet_facts();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            self.paint.wetpaint.varredura.faltam = 0;
            return;
        };
        if !sess.try_bring_home() {
            return;
        }
        sess.reconcile_facts(facts);
        let (gw, gh) = sess.grid;
        let cursor = v.cursor.clamp(1, gh.max(1));
        // O alcance com que o composite alarga a faixa (o mesmo `alcance_na` que ele chama).
        let suave_px = f64::from(self.paint.wetpaint.smooth_edges * super::smooth_edges::RAIO_MAX);
        let alcance = self.paint.wetpaint.oleo_real.map_or(0, |mut o| {
            o.suave_px = suave_px;
            let grades: Vec<_> = sess
                .engine
                .layers
                .iter()
                .map(|l| (&l.grid, l.visible))
                .collect();
            let fim = (cursor + FAIXA_INICIAL).min(gh);
            ph2d_wet_paint::oleo_real::alcance_na(
                &o,
                &grades,
                usize::from(sess.ratio),
                (1, cursor, gw, fim),
            )
        });
        let cabe = if v.ms_por_linha > 0.0 {
            ((ORCAMENTO_MS / v.ms_por_linha) as usize).saturating_sub(2 * alcance)
        } else {
            FAIXA_INICIAL
        };
        let cabe = cabe.max(ALCANCES * alcance).max(FAIXA_MIN);
        #[cfg(test)]
        let cabe = FAIXA_FIXA.with(std::cell::Cell::get).unwrap_or(cabe);
        let n = cabe.min(v.faltam).min(gh + 1 - cursor);
        let y1 = cursor + n - 1;
        // As linhas que o composite de facto pinta: a faixa e o alcance dos dois lados, presos à grade.
        let pintadas =
            (y1 + 1 + alcance).min(gh + 1) - cursor.saturating_sub(1 + alcance).max(1) + 1;
        // `merge_dirty` alarga 1 célula por lado; a faixa vizinha volta a escrever essa orla com o valor novo.
        // Só a ÓPTICA mudou: as grades (e o campo guardado, `oleo_real::CacheCampo`) ficam.
        sess.engine
            .merge_dirty_otica(1, cursor as i32, gw as i32, y1 as i32);
        let t0 = std::time::Instant::now();
        self.wetpaint_composite();
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let v = &mut self.paint.wetpaint.varredura;
        v.ms_por_linha = ms / pintadas as f64;
        v.faltam -= n;
        v.cursor = if y1 >= gh { 1 } else { y1 + 1 };
    }
}
