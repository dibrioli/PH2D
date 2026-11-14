//! **O mapa de cobertura do traço** (o tecto do Accumulate-OFF) que sabe ONDE foi escrito (B26.A1).
//!
//! Era um `Vec<u8>` do tamanho da tela zerado por `clear` + `resize`: 8,3 MB escritos a 4K na 1.ª
//! pincelada de cada traço e em CADA re-carimbo dos métodos de preenchimento (Line/Arc/Ellipse/
//! Polygon/Free Hand). Agora o pen-down devolve as páginas ao SO (a próxima alocação é `alloc_zeroed`)
//! e o re-carimbo zera, linha a linha, só o trecho que as footprints dos dabs anteriores cobriram.
//!
//! ⚠️ O registo só é exacto para quem escreve pela porta do lote ([`MascaraDoTraco::prepara`] +
//! [`MascaraDoTraco::fatia_do_lote`]). Qualquer outro acesso mutável (o `DerefMut`) marca o mapa
//! inteiro como sujo: o re-carimbo seguinte zera tudo, que é a lei antiga.

use super::super::Region;
use ph2d_painter_brush::{StrokeMethod, TexelLargo};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Default, PartialEq)]
enum Sujo {
    #[default]
    Limpo,
    /// Por linha da tela, o trecho `[x0, x1)` escrito (`x0 >= x1` = linha limpa).
    Linhas(Vec<[u32; 2]>),
    Todo,
}

/// `T` = o texel do tecto ([`ph2d_painter_brush::Tecto`], B29.1): `u8` = a lei que shipou.
#[derive(Clone, Debug, Default)]
pub(crate) struct MascaraDoTraco<T = u8> {
    texels: Vec<T>,
    sujo: Sujo,
    /// Bytes que ESTE mapa zerou à mão (a régua do B26.A1; o `alloc_zeroed` não conta: ninguém escreve).
    zerados: u64,
}

impl<T> Deref for MascaraDoTraco<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.texels
    }
}

impl<T> DerefMut for MascaraDoTraco<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        self.sujo = Sujo::Todo;
        &mut self.texels
    }
}

impl<T: Copy + Default + PartialEq + Send + Sync + bytemuck::Zeroable> MascaraDoTraco<T> {
    /// O pen-down: o mapa do traço anterior volta ao SO; o 1.º lote recebe páginas zeradas.
    pub(crate) fn clear(&mut self) {
        self.texels = Vec::new();
        self.sujo = Sujo::Limpo;
    }

    /// Dimensiona o mapa para `len` texels (`largura` por linha) antes de um lote cujos dabs escrevem
    /// dentro de `footprints`. Os métodos de preenchimento re-carimbam o traço INTEIRO a cada chamada,
    /// logo o tecto recomeça do zero (senão uma mudança de Strength apagava o traço); Space/Dots/
    /// Airbrush acumulam entre lotes.
    pub(crate) fn prepara(
        &mut self,
        len: usize,
        largura: usize,
        method: StrokeMethod,
        footprints: impl IntoIterator<Item = Region>,
    ) {
        let incremental = matches!(
            method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        if self.texels.len() != len {
            if incremental && !self.texels.is_empty() {
                let antes = self.texels.len();
                self.texels.resize(len, T::default()); // a tela mudou de tamanho a meio: o conteúdo manda
                self.zerados += len.saturating_sub(antes) as u64;
                self.sujo = Sujo::Todo;
            } else {
                // `zeroed_vec` = `alloc_zeroed` para QUALQUER texel (o `vec![T::default(); n]` só o faz no `u8`).
                self.texels = bytemuck::zeroed_vec(len);
                self.sujo = Sujo::Limpo;
            }
        } else if !incremental {
            self.zera_o_sujo(largura);
        }
        self.regista(largura, footprints);
    }

    /// A fatia que o lote preparado escreve — SÓ dentro das footprints dadas a [`Self::prepara`].
    pub(crate) fn fatia_do_lote(&mut self) -> &mut [T] {
        &mut self.texels
    }

    fn regista(&mut self, largura: usize, footprints: impl IntoIterator<Item = Region>) {
        if matches!(self.sujo, Sujo::Todo) || largura == 0 {
            return;
        }
        let altura = self.texels.len() / largura;
        for r in footprints {
            if matches!(self.sujo, Sujo::Limpo) {
                self.sujo = Sujo::Linhas(vec![[u32::MAX, 0]; altura]);
            }
            let Sujo::Linhas(linhas) = &mut self.sujo else {
                return;
            };
            let y1 = ((r.y + r.h) as usize).min(altura);
            for t in &mut linhas[(r.y as usize).min(y1)..y1] {
                t[0] = t[0].min(r.x);
                t[1] = t[1].max((r.x + r.w).min(largura as u32));
            }
        }
    }

    fn zera_o_sujo(&mut self, largura: usize) {
        match std::mem::take(&mut self.sujo) {
            Sujo::Limpo => {}
            Sujo::Todo => {
                self.texels.fill(T::default());
                self.zerados += self.texels.len() as u64;
            }
            Sujo::Linhas(linhas) => {
                for (y, &[x0, x1]) in linhas.iter().enumerate() {
                    if x0 < x1 {
                        let i = y * largura;
                        self.texels[i + x0 as usize..i + x1 as usize].fill(T::default());
                        self.zerados += u64::from(x1 - x0);
                    }
                }
            }
        }
        debug_assert!(
            self.texels.iter().all(|&b| b == T::default()),
            "B26.A1: alguém escreveu o tecto fora das footprints do lote"
        );
    }

    #[cfg(test)]
    pub(crate) fn zerados(&self) -> u64 {
        self.zerados
    }
}

/// B29.1: o tecto em [`TexelLargo`] (cobertura `u16` + pré-traço; a tela recompõe), escolhido UMA vez por traço
/// no 1.º lote do carimbo por dab ([`super::super::PainterTool::quer_o_tecto_largo`]). Com `Largo` o `stroke_mask`
/// `u8` fica vazio o traço inteiro: só se escolhe onde nenhum leitor dele corre.
#[derive(Clone, Debug, Default)]
pub(crate) struct TectoDoTraco {
    /// `None` = por escolher; `Some(true)` = `Largo`; `Some(false)` = o `u8` que shipou.
    pub(crate) escolha: Option<bool>,
    pub(crate) largo: MascaraDoTraco<TexelLargo>,
    /// O tecto largo de CADA camada do Composite, trocado para dentro de `largo` à volta do passe dela — como o
    /// `composite_mask` com o `u8`. Só o Composite de UMA camada escolhe o largo (com ≥ 2 a tela sob cada camada
    /// já não é o pré-traço: `recompoe` não vale).
    pub(crate) por_camada: [MascaraDoTraco<TexelLargo>; super::super::composite::N_CAMADAS],
    /// Veto de TESTE: o mesmo traço no `u8` (o red do gate, no mesmo processo).
    #[cfg(test)]
    pub(crate) so_byte: bool,
    /// TESTE: a lei do largo medida fora do Wet Paint (a porta do meio é só do produto, BUGS_painter #115); o traço
    /// do Wet não passa pelo carimbo por dab, e sem isto a lei ficava sem prova até ser adotada noutro meio.
    #[cfg(test)]
    pub(crate) sem_a_porta_do_meio: bool,
}

impl TectoDoTraco {
    /// O pen-down.
    pub(crate) fn clear(&mut self) {
        self.escolha = None;
        self.largo.clear();
        for m in &mut self.por_camada {
            m.clear();
        }
    }

    /// O passe da camada `pos` do Composite (ida e volta: a mesma chamada desfaz a troca).
    pub(crate) fn troca_com_a_camada(&mut self, pos: usize) {
        std::mem::swap(&mut self.largo, &mut self.por_camada[pos]);
    }
}

#[cfg(test)]
#[path = "mascara_tests.rs"]
mod tests;
