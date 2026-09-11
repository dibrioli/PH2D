//! **A ARTE DE UM QUAD DO PASSE VECTORIAL, em CPU** — a memória que a terceira média precisa.
//!
//! ⛔⛔ **Por que ela existe** (report do Enio, 2026-08-30, três vezes): a casa desenha os
//! sprites no passe 1 (alvo HDR) e o vector no passe 3 (a cena Vello), então **todo vector fica
//! por cima de todo sprite** — e uma folha que é uma IMAGEM nunca podia ficar à frente de um
//! galho. A cura é desenhar essa imagem **na mesma cena** em que a planta vive.
//!
//! ⚠️ **E o Vello 0.10 só aceita uma imagem em CPU** — verificado no fonte da versão instalada:
//! o `ExternalResource` dele, que ligaria uma textura de GPU, é interno ao crate.
//!
//! ⚠️ **E não custa COR**: o tonemap desta casa é passagem pura para conteúdo de 8 bits
//! (`tonemap.wgsl`, com gate a medi-la byte-exacta), logo mover uma sprite do passe HDR para a
//! camada LDR **não muda um pixel** dela. *É isso que separa isto de um remendo.*
//!
//! ⚠️ **Toda leitura PARA a GPU**, então cada arte é resolvida **uma vez** e memoizada por
//! `(textura, região)`. O caminho quente não lê nada.

use std::collections::BTreeMap;
use std::sync::Arc;

/// A arte já recortada de um quad: `(largura, altura, RGBA premultiplicado ou não — como veio)`.
pub(crate) type Art = (u32, u32, Arc<Vec<u8>>);

/// A chave: a textura e a região dela, com a região quantizada para o `f32` não a fragmentar.
type Key = (u32, [i32; 4]);

#[derive(Default)]
pub(crate) struct LeafImages {
    /// O atlas partilhado, lido inteiro (`texture_id == 0` amostra dele).
    ///
    /// ⛔⛔ **Ele já foi guardado pela VIDA DO PROCESSO, e eram `268 MB`** — achado §2.5 da
    /// auditoria de seis lentes. Hoje é largado no fim de cada quadro
    /// ([`LeafImages::end_frame`]), então a retenção é de **um quadro** e não da sessão.
    atlas: Option<(u32, Arc<Vec<u8>>)>,
    recortes: BTreeMap<Key, Option<Art>>,
    /// O [`ph2d_render::TextureAtlas::epoch`] com que os `recortes` foram tirados.
    ///
    /// ⚠️ **A outra metade do mesmo achado: nada era INVALIDADO** — *pintar na folha servia
    /// pixels velhos para sempre*. Um cache de bytes de GPU não se invalida por tempo nem por
    /// tamanho; invalida-se por **mudança**, e só o dono da textura sabe quando ela muda.
    epoch: u64,
}

impl LeafImages {
    /// **O FIM DO QUADRO larga o atlas** — a metade da §2.5 que é sobre MEMÓRIA.
    ///
    /// ⚠️ **Os recortes FICAM**, e é isso que a torna barata: eles são a resposta memoizada por
    /// `(textura, região)`, e são pequenos (a arte de uma folha). O que sai é a cópia INTEIRA do
    /// atlas — `8192² × 4 = 268 MB` —, que só serve para RESOLVER recortes novos. Uma folha nova
    /// paga uma leitura; nenhuma folha nova paga zero.
    pub(crate) fn end_frame(&mut self) {
        self.atlas = None;
    }

    /// **O CACHE CONCORDA COM A TEXTURA?** — a metade que é sobre CORRECÇÃO, separada de
    /// propósito.
    ///
    /// ⚠️ **Ela não toca na GPU, e é por isso que existe sozinha.** O `resolve` recebe
    /// `gpu`/`atlas`/`individual` e não é alcançável de um teste — *uma decisão enterrada num
    /// método que precisa de um contexto de GPU não tem gate possível*, que é a mesma frase que
    /// o doc do [`crop`] já dizia sobre a aritmética.
    pub(crate) fn sync_to(&mut self, epoch: u64) {
        if self.epoch != epoch {
            self.epoch = epoch;
            self.atlas = None;
            self.recortes.clear();
        }
    }

    /// ⭐⭐⭐ **A ÚNICA PORTA PARA A ARTE, e ela obriga a sincronizar** — a costura que a
    /// auditoria dizia não existir (*«`resolve` recebe `gpu`/`atlas`/`individual`
    /// directamente, sem costura»*).
    ///
    /// ⛔⛔ **Ela nasceu de uma mutação SOBREVIVENTE.** Enquanto o `art` chamava o
    /// [`Self::sync_to`] por dentro, arrancar essa linha deixava os dois gates de invalidação
    /// **verdes** — eles medem o `sync_to` sozinho, e nada media que alguém o CHAMASSE.
    /// *Dois gates sobre as duas metades de uma lei não cobrem o fio entre elas.*
    ///
    /// ⇒ o `art` mudou-se para o [`Synced`], que só se obtém daqui. Hoje, esquecer a
    /// sincronização é **erro de compilação** — que é a única forma de gate que uma mutação não
    /// atravessa.
    pub(crate) fn synced(&mut self, epoch: u64) -> Synced<'_> {
        self.sync_to(epoch);
        Synced { cache: self }
    }

    /// Quantas artes o cache guarda — `(tem o atlas inteiro?, quantos recortes)`.
    #[cfg(test)]
    pub(crate) fn cached(&self) -> (bool, usize) {
        (self.atlas.is_some(), self.recortes.len())
    }

    /// **Semeia o cache sem GPU** — a porta que torna os dois gates possíveis.
    ///
    /// ⚠️ Ela existe porque o único caminho que enche estes mapas é o [`Self::resolve`], que
    /// precisa de um `GpuContext`; sem ela, a lei de invalidação só seria demonstrável com
    /// adapter, e os gates de GPU desta casa são `#[ignore]` — *skip gracioso não é verde*.
    #[cfg(test)]
    pub(crate) fn seed_for_tests(&mut self, epoch: u64, side: u32, recortes: usize) {
        self.epoch = epoch;
        self.atlas = Some((side, Arc::new(vec![0u8; 4])));
        self.recortes.clear();
        for i in 0..recortes {
            self.recortes.insert((i as u32, [0; 4]), None);
        }
    }
    /// A resolução propriamente dita — privada, e só alcançável pelo [`Synced`].
    fn art_synced(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        atlas: &ph2d_render::TextureAtlas,
        individual: &ph2d_render::IndividualTextureStore,
        texture_id: u32,
        uv: [f32; 4],
    ) -> Option<Art> {
        let key: Key = (texture_id, uv.map(|v| (v * 4096.0).round() as i32));
        if let Some(hit) = self.recortes.get(&key) {
            return hit.clone();
        }
        let art = self.resolve(gpu, atlas, individual, texture_id, uv);
        self.recortes.insert(key, art.clone());
        art
    }

    fn resolve(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        atlas: &ph2d_render::TextureAtlas,
        individual: &ph2d_render::IndividualTextureStore,
        texture_id: u32,
        uv: [f32; 4],
    ) -> Option<Art> {
        let (w, h, px) = if texture_id == 0 {
            // ⚠️ **O atlas lê-se INTEIRO, e uma vez POR QUADRO** — ele é partilhado por toda a
            // cena, e uma leitura por folha seria uma paragem de GPU por folha. ⛔ *«Uma vez
            // só»* era o que esta nota dizia, e custava `268 MB` retidos pela vida do processo
            // (§2.5): quem larga é o [`LeafImages::end_frame`].
            let (side, bytes) = match &self.atlas {
                Some(hit) => (hit.0, Arc::clone(&hit.1)),
                None => {
                    let (side, _, bytes) = atlas.readback_mip(gpu, 0);
                    let bytes = Arc::new(bytes);
                    self.atlas = Some((side, Arc::clone(&bytes)));
                    (side, bytes)
                }
            };
            (side, side, bytes)
        } else {
            let (w, h, bytes) = individual.readback_rgba8(gpu, texture_id).ok()?;
            (w, h, Arc::new(bytes))
        };
        crop(w, h, &px, uv)
    }
}

/// **O CACHE JÁ SINCRONIZADO** — ver [`LeafImages::synced`], que é a única forma de o obter.
pub(crate) struct Synced<'c> {
    cache: &'c mut LeafImages,
}

impl Synced<'_> {
    /// **A arte de um quad**, memoizada. `None` = esta textura não se resolve (formato que a
    /// porta de leitura recusa, id que já não existe) — e a linha simplesmente não desenha, que
    /// é o mesmo que a membrana faz com um nome que ninguém publicou.
    pub(crate) fn art(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        atlas: &ph2d_render::TextureAtlas,
        individual: &ph2d_render::IndividualTextureStore,
        texture_id: u32,
        uv: [f32; 4],
    ) -> Option<Art> {
        self.cache
            .art_synced(gpu, atlas, individual, texture_id, uv)
    }
}

/// **O recorte de uma região `uv` de uma imagem RGBA8.**
///
/// ⚠️ **Função com nome e sem GPU de propósito** — é a única aritmética aqui, e uma
/// aritmética enterrada num método que precisa de um contexto de GPU não tem gate possível.
pub(crate) fn crop(w: u32, h: u32, px: &[u8], uv: [f32; 4]) -> Option<Art> {
    if w == 0 || h == 0 || px.len() < (w as usize * h as usize * 4) {
        return None;
    }
    let clampf = |v: f32| v.clamp(0.0, 1.0);
    let x0 = (clampf(uv[0].min(uv[2])) * w as f32).floor() as u32;
    let y0 = (clampf(uv[1].min(uv[3])) * h as f32).floor() as u32;
    let x1 = (clampf(uv[0].max(uv[2])) * w as f32).ceil() as u32;
    let y1 = (clampf(uv[1].max(uv[3])) * h as f32).ceil() as u32;
    let (cw, ch) = (x1.saturating_sub(x0).min(w), y1.saturating_sub(y0).min(h));
    if cw == 0 || ch == 0 {
        return None;
    }
    // ⚠️ A região INTEIRA é o caso comum (uma textura individual): devolvê-la sem copiar linha
    // a linha poupa o recorte, e o `Arc` do chamador já a partilha.
    let mut out = Vec::with_capacity(cw as usize * ch as usize * 4);
    for y in y0..y0 + ch {
        let ini = ((y * w + x0) * 4) as usize;
        out.extend_from_slice(&px[ini..ini + cw as usize * 4]);
    }
    Some((cw, ch, Arc::new(out)))
}

#[cfg(test)]
#[path = "motion_leaf_images_tests.rs"]
mod tests;
