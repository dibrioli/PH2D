//! **A LENTE do Motion no quadro** (ciclo 20, [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md)
//! §3.2) — o `fx.lens` do grafo, a LUT do disco, e o passe da placa sobre o acumulador do mundo.
//!
//! Três tempos, três portas:
//! 1. [`MotionLente::prepara`] — no passo do Motion (`coze_o_quadro`), que tem o grafo e o relógio:
//!    lê o nó, o TIQUE (o grão animado anda com o relógio do Motion, logo o scrub devolve o mesmo
//!    grão) e o `.cube`, relido só quando o caminho ou a data do ficheiro mudam;
//! 2. [`MotionLente::liga`] — a shell, ao planear o quadro: a lente corre? (liga o modo em faixas,
//!    sem o qual o acumulador não existe);
//! 3. [`MotionLente::aplica`] — a shell, depois da última escrita do mundo (a placa das formas) e
//!    antes do vidro e do chrome.
//!
//! ⭐ A recusa de um `.cube` diz-se NO CARTÃO ([`MotionLente::queixa`], na linha de leitura do nó)
//! — nunca só no terminal.

use ph2d_lens::{Cubo, ErroCubo, Lente};
use ph2d_nodegraph::graph::{Graph, NodeId};

/// Porque a LUT escolhida não entrou.
#[derive(Clone, Debug, PartialEq)]
pub enum ErroLut {
    /// O ficheiro não se lê (não existe, sem permissão, não é texto).
    Ilegivel,
    /// O texto não é uma LUT 3D válida.
    Cubo(ErroCubo),
}

struct LutLida {
    caminho: String,
    data: Option<std::time::SystemTime>,
    cubo: Result<Cubo, ErroLut>,
    /// Sobe a cada leitura: a placa recarrega a textura só quando muda.
    geracao: u64,
}

/// O estado da lente entre quadros.
#[derive(Default)]
pub struct MotionLente {
    lido: Option<ph2d_node_fx_lens::NoDaLente>,
    tique: u32,
    lut: Option<LutLida>,
    geracoes: u64,
    ligada: bool,
    passe: Option<ph2d_render::LensPass>,
    geracao_na_placa: Option<u64>,
}

impl MotionLente {
    /// **Por quadro, no passo do Motion**: o nó, o tique e a LUT.
    pub fn prepara(&mut self, graph: &Graph, tique: u64) {
        self.lido = ph2d_node_fx_lens::from_graph(graph);
        // O grão quer 32 bits; o relógio dá 2⁶⁴ — o tique dá a volta, a lei continua a mesma.
        #[expect(clippy::cast_possible_truncation, reason = "a semente dá a volta")]
        {
            self.tique = tique as u32;
        }
        let caminho = self.lido.as_ref().and_then(|l| l.lut.clone());
        self.sincroniza_lut(caminho.as_deref());
    }

    fn sincroniza_lut(&mut self, caminho: Option<&str>) {
        let Some(caminho) = caminho else {
            self.lut = None;
            return;
        };
        let data = std::fs::metadata(caminho).and_then(|m| m.modified()).ok();
        if self
            .lut
            .as_ref()
            .is_some_and(|l| l.caminho == caminho && l.data == data)
        {
            return;
        }
        let cubo = std::fs::read_to_string(caminho)
            .map_err(|_| ErroLut::Ilegivel)
            .and_then(|t| Cubo::ler(&t).map_err(ErroLut::Cubo));
        self.geracoes += 1;
        self.lut = Some(LutLida {
            caminho: caminho.to_string(),
            data,
            cubo,
            geracao: self.geracoes,
        });
    }

    /// A LUT carregada, se o ficheiro entrou.
    #[must_use]
    pub fn cubo(&self) -> Option<&Cubo> {
        self.lut.as_ref().and_then(|l| l.cubo.as_ref().ok())
    }

    /// A lente do quadro (`None` sem `fx.lens`).
    #[must_use]
    pub fn lente(&self) -> Option<&Lente> {
        self.lido.as_ref().map(|l| &l.lente)
    }

    /// **A shell, ao planear o quadro**: a lente corre neste quadro? `desenha` = o Motion está na
    /// cena (a ferramenta na mão ou um objecto Motion visível). Lente neutra ⇒ `false` e o quadro
    /// fica o de sempre.
    pub fn liga(&mut self, desenha: bool) -> bool {
        let tem_lut = self.cubo().is_some();
        self.ligada = desenha && self.lente().is_some_and(|l| !l.neutra(tem_lut));
        self.ligada
    }

    /// **A lente sobre o acumulador do mundo**, no sub-rect da cena (`None` = a janela inteira).
    pub fn aplica(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        mundo: &ph2d_render::WorldRt,
        scene_viewport: Option<[f32; 4]>,
    ) {
        if !self.ligada {
            return;
        }
        let Some(lente) = self.lido.as_ref().map(|l| l.lente) else {
            return;
        };
        let passe = self
            .passe
            .get_or_insert_with(|| ph2d_render::LensPass::new(gpu, ph2d_render::WorldRt::FORMAT));
        let geracao = self
            .lut
            .as_ref()
            .filter(|l| l.cubo.is_ok())
            .map(|l| l.geracao);
        if geracao != self.geracao_na_placa {
            let cubo = self.lut.as_ref().and_then(|l| l.cubo.as_ref().ok());
            passe.carrega_lut(gpu, cubo);
            self.geracao_na_placa = geracao;
        }
        let (w, h) = mundo.size();
        let rect = scene_viewport.map_or([0, 0, w, h], rect_em_pixels);
        passe.aplica(
            gpu,
            mundo.texture(),
            mundo.blend_view(),
            rect,
            &lente,
            self.tique,
        );
    }

    /// **O que o cartão do nó `no` diz da LUT** — a frase da recusa, ou nada.
    #[must_use]
    pub fn queixa(&self, no: NodeId) -> Option<String> {
        if self.lido.as_ref()?.no != no {
            return None;
        }
        let Err(e) = &self.lut.as_ref()?.cubo else {
            return None;
        };
        Some(frase(e))
    }
}

/// O viewport da cena (pixels do alvo, já inteiros pelo `floor` do split) como sub-rect.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "o split dá pixels inteiros e não negativos"
)]
pub(crate) fn rect_em_pixels(v: [f32; 4]) -> [u32; 4] {
    v.map(|c| c.max(0.0) as u32)
}

/// A frase que o artista lê.
#[must_use]
pub fn frase(e: &ErroLut) -> String {
    use ph2d_i18n::{tr, tr_with};
    match e {
        ErroLut::Ilegivel => tr("app.motion.motion_lens.lut_unreadable").to_string(),
        ErroLut::Cubo(ErroCubo::So1d) => tr("app.motion.motion_lens.lut_1d").to_string(),
        ErroLut::Cubo(ErroCubo::SemTamanho) => tr("app.motion.motion_lens.lut_no_size").to_string(),
        ErroLut::Cubo(ErroCubo::TamanhoInvalido(n)) => {
            tr_with("app.motion.motion_lens.lut_bad_size", &[("n", n)])
        }
        ErroLut::Cubo(ErroCubo::LinhaInvalida(linha)) => {
            tr_with("app.motion.motion_lens.lut_bad_line", &[("linha", linha)])
        }
        ErroLut::Cubo(ErroCubo::Contagem { esperado, lido }) => tr_with(
            "app.motion.motion_lens.lut_count",
            &[("lido", lido), ("esperado", esperado)],
        ),
    }
}

#[cfg(test)]
#[path = "motion_lens_tests.rs"]
mod tests;
