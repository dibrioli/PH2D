//! ⭐⭐⭐⭐ **A PILHA DE CAMADAS DA PEÇA** (`docs/3D/30` §2 — etapa 4, W1).
//!
//! A peça usa a MESMA pilha do Painter 2D: o [`LayerStack`] é metadado puro
//! (modo, opacidade, máscara, recorte, grupos, ajustes) e os píxeis vivem à
//! parte. Aqui os «píxeis» de uma camada são as amostras do plano de tinta
//! fina — uma lista —, e o compositor do Painter lê-a como uma imagem
//! DOBRADA em linhas de [`LARGURA_DA_DOBRA`]: a composição ponto a ponto não
//! depende da forma da imagem, e a dobra é o que deixa o compositor repartir
//! as linhas pelos núcleos (a peça inteira `N×1` corria numa thread só:
//! `346 ms` contra `33,5 ms` a `64x`, doc 30 §6).
//!
//! ⛔ **Uma segunda pilha de camadas seria a segunda resposta a «o que é uma
//! camada»** — por isso nenhuma lei de composição mora aqui: a cor é a do
//! [`composite_region`], ao bit (o gate compara com a MESMA imagem 2D).
//!
//! ⛔⛔ **Toda operação que cria ou apaga uma camada passa por ESTA porta**,
//! que muda a pilha e os planos juntos: uma camada sem plano é um traço que
//! cai em lado nenhum. [`PilhaDaPeca::sincronizada`] é o invariante, e os
//! gates o leem depois de cada operação.
//!
//! ⭐ O painel de Layers (W3) chama a porta pelos pedidos do Painter
//! (`pilha_da_peca_porta`); os `define_*` soltos ficam para os gates — o
//! produto muda o metadado de uma vez ([`PilhaDaPeca::troca_metadado`]).

use std::collections::BTreeMap;

use ph2d_mesh_colors::Tinta;
use ph2d_tool_painter::{AdjustmentKind, SpatialUnits, rescale_spatial_params};
#[cfg(test)]
use ph2d_tool_painter::{AdjustmentParams, BlendMode};
use ph2d_tool_painter::{
    LayerId, LayerKind, LayerPixelSource, LayerStack, Region, composite_region,
};

/// A largura da dobra: o plano de `N` amostras lê-se como uma imagem
/// `1 024 × ⌈N/1 024⌉`. ⚠️ É a granularidade do paralelismo do compositor
/// (uma linha = uma unidade de trabalho), não um recurso: a faixa de `1 024`
/// amostras compõe em `0,114 ms` (doc 30 §6). É a largura MÍNIMA — ver
/// [`ALTURA_MAX_DA_DOBRA`].
pub(crate) const LARGURA_DA_DOBRA: u32 = 1024;

/// ⛔ **A altura máxima da dobra** — o lado de textura 2D que TODA placa
/// garante (o mínimo do WebGPU para `max_texture_dimension_2d`): o compositor de
/// GPU lê a dobra como uma textura. Acima dela a dobra ALARGA (potências de 2):
/// a `128x`/`256x` a peça da lição tem `12 M`/`48 M` amostras, e a `1 024` de
/// largura a placa recusava-a — a peça caía na CPU, `153`/`646 ms` por passo do
/// arrasto (doc 30 §13.2).
pub(crate) const ALTURA_MAX_DA_DOBRA: u32 = 8192;

/// O nome da camada em que um plano anterior às camadas abre — o mesmo que o
/// Painter 2D dá à camada de um sprite novo, porque é o mesmo painel que o
/// mostra. ⚠️ É DADO do documento depois de nascer (o artista renomeia-o); a
/// identidade da camada é o `LayerId`.
pub(crate) fn nome_da_base() -> &'static str {
    ph2d_i18n::tr("app.sculpt3d.pilha_da_peca.camada_de_base")
}

/// As dimensões da dobra de um plano de `n` amostras.
#[must_use]
pub(crate) fn dobra(n: usize) -> (u32, u32) {
    let (n, teto) = (n as u64, u64::from(ALTURA_MAX_DA_DOBRA));
    let mut largura = u64::from(LARGURA_DA_DOBRA);
    while n.div_ceil(largura) > teto && largura < teto {
        largura *= 2;
    }
    let altura = n.div_ceil(largura).max(1);
    (largura as u32, u32::try_from(altura).unwrap_or(u32::MAX))
}

/// As amostras de UMA camada.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlanoDaCamada {
    /// RGBA8 sRGB direito (a precisão das camadas do Painter), já DOBRADO:
    /// `largura · altura · 4` bytes, e a cauda depois da amostra `N` é
    /// transparente — é o que deixa o compositor ler a camada sem cópia.
    rgba8: Vec<u8>,
    /// O RELEVO da camada, `[altura, corpo]` por amostra (`N`), ou `None` se
    /// ela nunca levou impasto.
    relevo: Option<Vec<[f32; 2]>>,
    /// O que a placa ainda não tem destes píxeis (`composto_na_placa`).
    pub(crate) na_placa: NaPlaca,
    /// A largura da dobra deste plano ([`dobra`]).
    largura: u32,
}

/// ⭐⭐ **A versão dos píxeis de uma camada e as linhas da dobra que mudaram
/// desde a última subida à placa** — o compositor de GPU só volta a subir uma
/// camada cuja versão mudou, e só as linhas sujas (`LayerPixels::dirty`).
///
/// ⛔ Todo escritor de `rgba8` chama [`NaPlaca::mudou`]: um que não chame
/// deixa a placa a compor a camada de antes (gate
/// `composto_na_placa_tests::cada_escritor_da_camada_muda_a_versao`). É estado
/// da SESSÃO: duas camadas com os mesmos píxeis são iguais (`PartialEq`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct NaPlaca {
    pub(crate) versao: u64,
    /// `None` = nada por subir; `Some((a, b))` = as linhas `a..=b`.
    pub(crate) linhas: Option<(u32, u32)>,
}

impl PartialEq for NaPlaca {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// A próxima versão de píxeis — única no processo, logo uma camada que volta
/// do desfazer com a sua versão antiga nunca se confunde com a que a placa tem.
fn proxima_versao() -> u64 {
    static VERSAO: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    VERSAO.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl NaPlaca {
    fn nova(altura: u32) -> Self {
        Self {
            versao: proxima_versao(),
            linhas: Some((0, altura.saturating_sub(1))),
        }
    }

    /// Os píxeis das linhas `a..=b` mudaram.
    pub(crate) fn mudou(&mut self, a: u32, b: u32) {
        self.versao = proxima_versao();
        self.linhas = Some(match self.linhas {
            Some((x, y)) => (x.min(a), y.max(b)),
            None => (a, b),
        });
    }
}

impl PlanoDaCamada {
    pub(crate) fn transparente(n: usize) -> Self {
        let (l, h) = dobra(n);
        Self {
            rgba8: vec![0; l as usize * h as usize * 4],
            relevo: None,
            na_placa: NaPlaca::nova(h),
            largura: l,
        }
    }

    /// A camada inteira mudou.
    pub(crate) fn mudou_toda(&mut self) {
        let h = self.rgba8.len() / (self.largura as usize * 4);
        self.na_placa.mudou(0, (h as u32).saturating_sub(1));
    }

    /// A amostra `i` mudou.
    pub(crate) fn mudou_amostra(&mut self, i: usize) {
        let y = (i / self.largura as usize) as u32;
        self.na_placa.mudou(y, y);
    }

    /// Os píxeis da dobra inteira (com a cauda) — o que a placa sobe.
    pub(crate) fn dobrado(&self) -> &[u8] {
        &self.rgba8
    }

    /// A largura da dobra deste plano.
    pub(crate) fn largura(&self) -> u32 {
        self.largura
    }

    /// Escreve as `px.len()` primeiras amostras e o relevo (a leitura do
    /// documento).
    pub(crate) fn escreve(&mut self, px: &[[u8; 4]], relevo: Option<Vec<[f32; 2]>>) {
        for (d, s) in self.rgba8.as_chunks_mut::<4>().0.iter_mut().zip(px) {
            d.copy_from_slice(s);
        }
        self.relevo = relevo;
        self.mudou_toda();
    }

    /// As amostras, sem a cauda da dobra.
    pub(crate) fn rgba8(&self, n: usize) -> &[u8] {
        &self.rgba8[..n * 4]
    }

    pub(crate) fn relevo(&self) -> Option<&[[f32; 2]]> {
        self.relevo.as_deref()
    }
}

/// Por que a porta recusou uma operação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecusaDaPilha {
    /// O tecto de camadas do Painter (`HARD_CAP_LAYERS`).
    Tecto,
    /// A camada não existe (ou não é do tipo que a operação pede).
    Desconhecida,
    /// ⛔ Um ajuste que lê o PLANO da imagem (uma direcção, um centro, uma
    /// trama de pontos — `reads_the_image_plane`): a superfície não tem nenhum
    /// dos três. Os de vizinhança borram na retícula (`docs/3D/30` §14).
    LeOPlanoDaImagem(AdjustmentKind),
    /// ⛔ A camada de BASE não se apaga nem sai do fundo — a lei do Painter 2D
    /// (`delete_layer`: a base é permanente). Desde a W4 o relevo é de cada
    /// camada; a base já não é a dona dele.
    ABase,
    /// ⛔ Um efeito de vizinhança acima de `64x`
    /// (`vizinhanca_da_peca::NIVEL_MAX_DA_VIZINHANCA`): um passo levaria segundos.
    DegrauAlto,
    /// Um traço está a pintar a pilha (a cópia de trabalho é de uma camada
    /// dela): mudar a estrutura por baixo dele perderia o traço.
    TracoAberto,
    /// Um pedido de METADADO que muda a estrutura (camadas, tipos, máscaras) —
    /// isso só pelas operações da porta, que levam os planos.
    MudaAEstrutura,
    /// A camada activa não é de pintura (um ajuste, uma máscara): o traço não
    /// tem onde pousar.
    ActivaNaoPinta,
}

/// ⭐⭐⭐⭐ **A pilha da peça: o metadado do Painter + o plano de cada camada.**
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PilhaDaPeca {
    pilha: LayerStack,
    planos: BTreeMap<LayerId, PlanoDaCamada>,
    /// `N`, o número de amostras do plano de tinta fina a que a pilha serve.
    amostras: usize,
    /// A camada que o traço em curso pinta (ver `pilha_da_peca_traco`) —
    /// estado da sessão, nunca do documento.
    em_traco: Option<LayerId>,
    /// ⭐⭐ O FUNDO: a cor do barro por baixo da pilha, POR VÉRTICE, fixada
    /// quando a pilha nasce (`pilha_da_peca_fundo`).
    fundo: Vec<[f32; 3]>,
    /// O plano de tinta da CPU ficou atrás da pilha (a placa compôs) — sessão.
    cpu: fundo::Atraso,
    /// A retícula como vizinhança dos efeitos de vizinhança — sessão.
    vizinhanca: crate::vizinhanca_da_peca::NaPilha,
    /// A forma da dobra que está no relevo da peça — sessão (`relevo`).
    relevo_dobrado: relevo::Dobrado,
}

impl LayerPixelSource for PilhaDaPeca {
    fn layer_rgba(&self, id: LayerId) -> Option<&[u8]> {
        self.planos.get(&id).map(|p| p.rgba8.as_slice())
    }
}

/// `true` para os tipos que têm plano: o raster e a máscara.
fn tem_plano(kind: &LayerKind) -> bool {
    matches!(kind, LayerKind::Raster(_) | LayerKind::Mask(_))
}

/// O byte sRGB8 de uma cor do plano de tinta (`byte / 255`, ver
/// `ph2d_sculpt3d::tela_na_malha::Tela`) — o inverso exacto da leitura.
fn byte_de(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl PilhaDaPeca {
    /// ⭐⭐ **A pilha de um plano anterior às camadas: UMA camada opaca** com a
    /// cor e o relevo dele — a migração do documento v5 e o nascimento da pilha
    /// sobre um plano que acabou de ser semeado.
    ///
    /// ⚠️ A cor passa a `sRGB8`: cada amostra fica a no máximo meio degrau do
    /// que era (o gate `um_v5_abre_igual_e_regrava_a_meio_degrau`).
    #[must_use]
    pub(crate) fn de_tinta(t: &Tinta) -> Self {
        let n = t.amostras().len();
        let (l, h) = dobra(n);
        let mut pilha = LayerStack::new();
        let base = pilha
            .add_raster(nome_da_base(), l, h)
            .expect("uma pilha vazia está abaixo do tecto");
        let mut plano = PlanoDaCamada::transparente(n);
        for (px, c) in plano
            .rgba8
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(t.amostras())
        {
            px.copy_from_slice(&[byte_de(c[0]), byte_de(c[1]), byte_de(c[2]), 255]);
        }
        plano.relevo = t.relevo().map(<[_]>::to_vec);
        if plano.relevo.is_some()
            && let Some(camada) = pilha.get_mut(base)
        {
            camada.has_relief = true;
        }
        Self {
            pilha,
            planos: BTreeMap::from([(base, plano)]),
            amostras: n,
            em_traco: None,
            fundo: t.plano_por_vertice().to_vec(),
            cpu: fundo::Atraso::default(),
            vizinhanca: Default::default(),
            relevo_dobrado: Default::default(),
        }
    }

    /// A pilha montada de partes já lidas (o documento) — quem chama confere a
    /// [`Self::sincronizada`] antes de a usar.
    pub(crate) fn de_partes(
        pilha: LayerStack,
        planos: BTreeMap<LayerId, PlanoDaCamada>,
        amostras: usize,
        fundo: Vec<[f32; 3]>,
    ) -> Self {
        let mut p = Self {
            pilha,
            planos,
            amostras,
            em_traco: None,
            fundo,
            cpu: fundo::Atraso::default(),
            vizinhanca: Default::default(),
            relevo_dobrado: Default::default(),
        };
        let ids: Vec<LayerId> = p.planos.keys().copied().collect();
        for id in ids {
            p.marca_relevo(id);
        }
        p
    }

    /// ⭐ **As camadas `ids` subiram à placa** — as linhas sujas delas ficam
    /// limpas (`composto_na_placa`); a versão fica, é ela que a placa guarda.
    pub(crate) fn subiu_a_placa(&mut self, ids: impl IntoIterator<Item = LayerId>) {
        for id in ids {
            if let Some(p) = self.planos.get_mut(&id) {
                p.na_placa.linhas = None;
            }
        }
    }

    /// O plano de uma camada, para os gates escreverem a fixtura.
    #[cfg(test)]
    pub(crate) fn plano_mut(&mut self, id: LayerId) -> Option<&mut PlanoDaCamada> {
        self.planos.get_mut(&id)
    }

    /// A pilha (metadado), para ler — quem escreve passa pela porta.
    #[must_use]
    pub(crate) fn pilha(&self) -> &LayerStack {
        &self.pilha
    }

    /// O plano de uma camada.
    #[must_use]
    pub(crate) fn plano(&self, id: LayerId) -> Option<&PlanoDaCamada> {
        self.planos.get(&id)
    }

    /// Quantos bytes a pilha segura — os planos e os relevos (o metadado é
    /// desprezável) — para o orçamento do desfazer.
    #[must_use]
    pub(crate) fn footprint_bytes(&self) -> usize {
        self.planos
            .values()
            .map(|p| {
                p.rgba8.capacity()
                    + p.relevo
                        .as_ref()
                        .map_or(0, |r| r.capacity() * size_of::<[f32; 2]>())
            })
            .sum()
    }

    /// `N`.
    #[must_use]
    pub(crate) fn amostras(&self) -> usize {
        self.amostras
    }

    /// ⭐⭐⭐ **O INVARIANTE**: todo raster e toda máscara da pilha têm plano,
    /// todo plano tem a sua camada, cada plano tem o tamanho da dobra e cada
    /// relevo `N` amostras — e nenhuma camada lê o plano da imagem.
    #[must_use]
    pub(crate) fn sincronizada(&self) -> bool {
        let (l, h) = dobra(self.amostras);
        let bytes = l as usize * h as usize * 4;
        let com_plano: Vec<LayerId> = self
            .pilha
            .all_ids()
            .filter(|&id| self.pilha.get(id).is_some_and(|c| tem_plano(&c.kind)))
            .collect();
        let dims_certas =
            self.pilha
                .all_ids()
                .all(|id| match self.pilha.get(id).map(|c| &c.kind) {
                    Some(LayerKind::Raster(r)) => (r.width, r.height) == (l, h),
                    Some(LayerKind::Mask(m)) => (m.width, m.height) == (l, h),
                    Some(LayerKind::Texture(_)) => false,
                    Some(LayerKind::Adjustment(a)) => !a.kind.reads_the_image_plane(),
                    _ => true,
                });
        dims_certas
            && com_plano.len() == self.planos.len()
            && com_plano.iter().all(|id| self.planos.contains_key(id))
            && self.planos.values().all(|p| {
                p.rgba8.len() == bytes && p.relevo.as_ref().is_none_or(|r| r.len() == self.amostras)
            })
    }

    /// ⭐ **Uma camada nova, transparente, no topo** — e activa, como no 2D.
    pub(crate) fn nova_camada(&mut self, nome: &str) -> Result<LayerId, RecusaDaPilha> {
        self.livre()?;
        let (l, h) = dobra(self.amostras);
        let id = self
            .pilha
            .add_raster(nome, l, h)
            .ok_or(RecusaDaPilha::Tecto)?;
        self.planos
            .insert(id, PlanoDaCamada::transparente(self.amostras));
        Ok(id)
    }

    /// ⭐ **Uma máscara para a camada `dono`** — nasce BRANCA (tudo visível),
    /// que é o que torna ligá-la uma operação que não muda a peça. A activa
    /// continua a dona (pintar a máscara na peça ainda não existe).
    pub(crate) fn nova_mascara(&mut self, dono: LayerId) -> Result<LayerId, RecusaDaPilha> {
        self.livre()?;
        let id = self
            .pilha
            .add_mask(dono)
            .ok_or(RecusaDaPilha::Desconhecida)?;
        let mut plano = PlanoDaCamada::transparente(self.amostras);
        plano.rgba8[..self.amostras * 4].fill(255);
        plano.mudou_toda();
        self.planos.insert(id, plano);
        Ok(id)
    }

    /// ⭐ **Um ajuste novo no topo** — recusado se ele lê o plano da imagem. Nasce
    /// como o do 2D (`seed_user_adjustment`, os raios na unidade da peça com o
    /// slider no mesmo sítio), e a activa continua a camada de pintura que era:
    /// um ajuste não se pinta.
    pub(crate) fn novo_ajuste(
        &mut self,
        kind: AdjustmentKind,
        unidades: SpatialUnits,
    ) -> Result<LayerId, RecusaDaPilha> {
        self.livre()?;
        if kind.reads_the_image_plane() {
            return Err(RecusaDaPilha::LeOPlanoDaImagem(kind));
        }
        let antes = self.pilha.active();
        let id = self
            .pilha
            .add_adjustment(kind)
            .ok_or(RecusaDaPilha::Tecto)?;
        if let Some(a) = self.pilha.adjustment_mut(id) {
            ph2d_tool_painter::seed_user_adjustment(&mut a.params);
            rescale_spatial_params(&mut a.params, SpatialUnits::Pixels, unidades);
        }
        if let Some(a) = antes {
            self.pilha.set_active(a);
        }
        Ok(id)
    }

    /// ⭐ **Duplica uma camada de pintura** logo acima dela, com uma CÓPIA do
    /// plano — a semântica do `LayerStack::duplicate` do 2D: a máscara não vem
    /// junto. A cópia leva o RELEVO, com a profundidade e o modo (W4).
    pub(crate) fn duplica(&mut self, id: LayerId) -> Result<LayerId, RecusaDaPilha> {
        self.livre()?;
        if !matches!(
            self.pilha.get(id).map(|c| &c.kind),
            Some(LayerKind::Raster(_))
        ) {
            return Err(RecusaDaPilha::Desconhecida);
        }
        let copia = self
            .pilha
            .duplicate(id)
            .ok_or(RecusaDaPilha::Desconhecida)?;
        if let Some(p) = self.planos.get(&id).cloned() {
            self.planos.insert(copia, p);
        }
        self.marca_relevo(copia);
        Ok(copia)
    }

    /// ⭐ **Apaga uma camada** (com a máscara e, num grupo, os filhos) e
    /// devolve os planos dela — o desfazer leva-os. A BASE não se apaga.
    pub(crate) fn apaga(
        &mut self,
        id: LayerId,
    ) -> Result<BTreeMap<LayerId, PlanoDaCamada>, RecusaDaPilha> {
        self.livre()?;
        if self.pilha.get(id).is_none() {
            return Err(RecusaDaPilha::Desconhecida);
        }
        if self.base() == Some(id) {
            return Err(RecusaDaPilha::ABase);
        }
        self.pilha.remove(id);
        let vivas: Vec<LayerId> = self.pilha.all_ids().collect();
        let (vivos, mortos) = std::mem::take(&mut self.planos)
            .into_iter()
            .partition(|(k, _)| vivas.contains(k));
        self.planos = vivos;
        Ok(mortos)
    }

    /// A camada activa (a que o traço pinta).
    #[cfg(test)]
    pub(crate) fn define_activa(&mut self, id: LayerId) {
        self.pilha.set_active(id);
    }

    /// O modo de mistura de uma camada.
    #[cfg(test)]
    pub(crate) fn define_modo(&mut self, id: LayerId, modo: BlendMode) {
        self.pilha.set_blend_mode(id, modo);
    }

    /// A opacidade de uma camada.
    #[cfg(test)]
    pub(crate) fn define_opacidade(&mut self, id: LayerId, opacidade: f32) {
        self.pilha.set_opacity(id, opacidade);
    }

    /// Mostra ou esconde uma camada.
    #[cfg(test)]
    pub(crate) fn define_visivel(&mut self, id: LayerId, visivel: bool) {
        self.pilha.set_visible(id, visivel);
    }

    /// Recorta (ou não) uma camada à de baixo.
    #[cfg(test)]
    pub(crate) fn define_recorte(&mut self, id: LayerId, recorte: bool) {
        self.pilha.set_clipping(id, recorte);
    }

    /// ⭐ **Os parâmetros de um ajuste** — recusados se são de outro tipo: o
    /// tipo de um ajuste é decidido na porta ([`Self::novo_ajuste`]).
    #[cfg(test)]
    pub(crate) fn define_parametros(
        &mut self,
        id: LayerId,
        params: AdjustmentParams,
    ) -> Result<(), RecusaDaPilha> {
        let ajuste = self
            .pilha
            .adjustment_mut(id)
            .ok_or(RecusaDaPilha::Desconhecida)?;
        if params.kind() != ajuste.kind {
            return Err(RecusaDaPilha::Desconhecida);
        }
        ajuste.params = params;
        Ok(())
    }

    /// ⭐⭐⭐⭐ **A COR DA PEÇA: a pilha inteira composta** — RGBA8 sRGB
    /// direito, `N · 4` bytes, pelo compositor do Painter sobre a dobra.
    #[must_use]
    pub(crate) fn compor(&self) -> Vec<u8> {
        self.compor_faixa(0, self.amostras)
    }

    /// ⭐⭐⭐ **A cor das amostras `inicio..fim`** — ao bit igual ao mesmo
    /// pedaço de [`Self::compor`] (a composição é ponto a ponto). Corta a
    /// faixa nas linhas da dobra: cabeça parcial, linhas inteiras, cauda.
    /// Com um efeito de vizinhança a peça compõe-se INTEIRA (`vizinhanca`).
    #[must_use]
    pub(crate) fn compor_faixa(&self, inicio: usize, fim: usize) -> Vec<u8> {
        let fim = fim.min(self.amostras);
        if inicio >= fim {
            return Vec::new();
        }
        if self.le_a_vizinhanca() {
            return self.faixa_da_superficie(inicio, fim);
        }
        let (l, h) = dobra(self.amostras);
        let lu = l as usize;
        let mut out = Vec::with_capacity((fim - inicio) * 4);
        let mut i = inicio;
        while i < fim {
            let (y, x) = (i / lu, i % lu);
            let regiao = if x == 0 && fim - i >= lu {
                let linhas = (fim - i) / lu;
                Region {
                    x: 0,
                    y: y as u32,
                    w: l,
                    h: linhas as u32,
                }
            } else {
                let w = (lu - x).min(fim - i);
                Region {
                    x: x as u32,
                    y: y as u32,
                    w: w as u32,
                    h: 1,
                }
            };
            out.extend(composite_region(&self.pilha, self, l, h, regiao));
            i += regiao.w as usize * regiao.h as usize;
        }
        out
    }

    /// ⭐⭐⭐⭐ **Escreve a peça composta no plano de tinta** — o que a placa,
    /// o shader, o bake e a doação já leem, sem mudança nenhuma.
    ///
    /// `fundo()` devolve a cor por baixo da pilha, amostra a amostra (a
    /// semente): só é chamado se alguma amostra do composto não é opaca, e
    /// uma amostra que ele não cubra lê-se BRANCA. Ver [`achata`].
    pub(crate) fn pinta_tinta(&self, tinta: &mut Tinta, fundo: impl FnOnce() -> Vec<[f32; 3]>) {
        debug_assert_eq!(
            tinta.amostras().len(),
            self.amostras,
            "a pilha é deste plano"
        );
        let composto = self.compor();
        let fundo = if precisa_de_fundo(&composto) {
            fundo()
        } else {
            Vec::new()
        };
        achata(
            &composto,
            |i| fundo.get(i).copied().unwrap_or(ph2d_mesh_colors::BRANCO),
            tinta.amostras_mut(),
        );
        tinta.com_relevo(self.relevo_composto());
    }
}

/// `true` se alguma amostra do composto não é opaca (e o fundo se vê).
#[must_use]
pub(crate) fn precisa_de_fundo(composto: &[u8]) -> bool {
    composto.as_chunks::<4>().0.iter().any(|px| px[3] < 255)
}

/// ⭐⭐⭐ **O composto sobre o FUNDO, na unidade do plano de tinta** (o byte
/// sRGB sobre `255`).
///
/// ⚠️ Onde o composto é opaco a cor é **exactamente** `byte / 255` — a mesma
/// leitura que a tela do Painter faz — e o fundo nem é lido. Onde não é, a
/// mistura é em LUZ (linear), como o compositor mistura as camadas.
pub(crate) fn achata(composto: &[u8], fundo: impl Fn(usize) -> [f32; 3], destino: &mut [[f32; 3]]) {
    use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_byte, srgb_to_linear_unit};
    for (i, (px, out)) in composto
        .as_chunks::<4>()
        .0
        .iter()
        .zip(destino.iter_mut())
        .enumerate()
    {
        if px[3] == 255 {
            *out = [px[0], px[1], px[2]].map(|b| f32::from(b) / 255.0);
            continue;
        }
        let a = f32::from(px[3]) / 255.0;
        let f = fundo(i);
        for c in 0..3 {
            let luz = srgb_to_linear_byte(px[c]) * a + srgb_to_linear_unit(f[c]) * (1.0 - a);
            out[c] = linear_to_srgb_unit(luz);
        }
    }
}

/// ⭐ **O traço sobre a pilha** (W2) — a cópia de trabalho, a descida ao
/// byte, a recomposição incremental e as trocas do desfazer.
#[path = "pilha_da_peca_traco.rs"]
mod traco;
pub(crate) use traco::para_bytes;

/// ⭐ **O fundo da pilha e o plano da CPU atrasado** (W1b) — a cor por baixo
/// da pilha fixada quando ela nasce, e a composição na placa.
#[path = "pilha_da_peca_fundo.rs"]
mod fundo;

/// ⭐ **O painel sobre a pilha** (W3) — o metadado de uma vez e a troca
/// estrutural do desfazer.
#[path = "pilha_da_peca_porta.rs"]
mod porta;
pub(crate) use porta::TrocaDaPilha;

/// ⭐ **Os efeitos de vizinhança na peça** (W6) — a retícula como vizinhança.
#[path = "pilha_da_peca_vizinhanca.rs"]
mod vizinhanca;

/// ⭐ **O relevo por camada** (W4) — a dobra do 2D sobre a pilha.
#[path = "pilha_da_peca_relevo.rs"]
mod relevo;

#[cfg(test)]
#[path = "pilha_da_peca_tests.rs"]
mod tests;
