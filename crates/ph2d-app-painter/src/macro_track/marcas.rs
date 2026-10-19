//! **As marcas do artista por imagem** (W9, `docs/Macros/07_plano_W9.md`): a marca vive FORA do mundo,
//! ao lado dos píxeis — no Painter enquanto ele tem a imagem (cada passo marca, o desfazer desmarca), e
//! aqui quando não tem. Sem o Painter não há água: o que mudou na sprite desde que ele a largou é do
//! artista (uma ferramenta de imagem; desfazê-la desmarca sozinho, porque a régua é a diferença para a
//! base). Gravar de novo é outra época: outra marca; a antiga fica para o desfazer da gravação.

use super::{Atual, ChaveDaMarca, Fonte, LeSprite, PalcoDaPintura, PalcosDoPainter, fim};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::marca::{self, MarcaDoArtista};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A marca de uma imagem numa gravação.
pub(super) struct Guardada {
    /// A marca que o Painter deixou (ou a que se abriu do projeto).
    pub(super) marca: MarcaDoArtista,
    /// Os píxeis da sprite (straight) quando o Painter a largou; `None` = a ler na próxima leitura.
    base: Option<Vec<u8>>,
    /// `marca` ∪ o que mudou na sprite desde a `base` (a última leitura).
    pub(super) efetiva: Option<MarcaDoArtista>,
}

impl Guardada {
    fn nova((w, h): (u32, u32), epoca: u64) -> Self {
        Self::de(MarcaDoArtista::nova(w, h, epoca))
    }

    fn de(marca: MarcaDoArtista) -> Self {
        Self {
            marca,
            base: None,
            efetiva: None,
        }
    }

    /// A marca que vale agora (a efetiva, se a sprite já se leu).
    pub(super) fn vale(&self) -> &MarcaDoArtista {
        self.efetiva.as_ref().unwrap_or(&self.marca)
    }

    /// Um escritor que não é o artista mexeu na sprite (a reescrita da macro): o que valia fica a marca
    /// e a base relê-se.
    pub(super) fn rebase(&mut self) {
        if let Some(e) = self.efetiva.take() {
            self.marca = e;
        }
        self.base = None;
    }
}

/// Os texels onde `a` e `b` (RGBA, do mesmo tamanho) diferem.
fn diferem(a: &[u8], b: &[u8], n: usize) -> Vec<u64> {
    let mut bits = vec![0u64; n.div_ceil(64)];
    for (k, (p, q)) in a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .enumerate()
    {
        if p != q {
            marca::liga(&mut bits, k);
        }
    }
    bits
}

/// **Lê a sprite de `p`** (sem o Painter ligado) quando a leitura que há não é dela: a imagem atual para a
/// mistura, e a marca efetiva = a guardada ∪ o que mudou desde a base (a 1.ª leitura depois de o Painter
/// a largar É a base).
pub(super) fn le_a_sprite<R: ?Sized>(
    p: &mut PalcoDaPintura,
    guardadas: &mut BTreeMap<ChaveDaMarca, Guardada>,
    le: Option<&mut LeSprite<'_, R>>,
    ranhuras: &mut R,
) {
    if p.atual.as_ref().is_some_and(|a| a.fonte == Fonte::Sprite) {
        return;
    }
    let (Some(sprite), Some(chave), Some(le)) = (p.sprite, p.chave, le) else {
        return;
    };
    let lida = le(sprite, ranhuras);
    p.versao_da_atual += 1;
    p.atual = Some(Atual {
        fonte: Fonte::Sprite,
        img: lida.as_ref().map(|(rgba, w, h)| {
            let mut rgba = rgba.clone();
            ph2d_render::premul::premultiply_rgba8(&mut rgba);
            Arc::new(fim::Imagem { rgba, w: *w, h: *h })
        }),
    });
    let Some((rgba, w, h)) = lida else {
        return;
    };
    let g = guardadas
        .entry(chave)
        .or_insert_with(|| Guardada::nova((w, h), chave.1));
    // Outro tamanho (um Trim, um Padding): a marca fica guardada e não se mistura (a gravação é de
    // outro tamanho); o desfazer da ferramenta devolve o tamanho e a marca volta a valer.
    if g.marca.tamanho() != (w, h) {
        g.efetiva = None;
        return;
    }
    let mut efetiva = g.marca.clone();
    match &g.base {
        Some(base) if base.len() == rgba.len() => {
            efetiva.satura_bits(&diferem(base, &rgba, (w * h) as usize));
        }
        _ => g.base = Some(rgba),
    }
    g.efetiva = Some(efetiva);
}

impl PalcosDoPainter {
    /// **Por quadro, antes de [`Self::reescreve`] e de [`Self::sobe`]**: o Painter ligado a uma imagem com
    /// macro recebe a marca dela (quando se liga) e devolve-a a cada passo; cada sprite sem o Painter lê-se
    /// quando os píxeis dela mudam (a marca efetiva e a base, que nasce quando o Painter a larga).
    pub fn liga_a_marca<R: ?Sized>(
        &mut self,
        painter: Option<&mut PainterTool>,
        le: &mut LeSprite<'_, R>,
        ranhuras: &mut R,
    ) {
        let ligada = painter.as_deref().and_then(PainterTool::bound_document);
        let mut alvo = None;
        for l in self.lidos.values_mut() {
            let Ok(Some(p)) = &mut l.palco else {
                continue;
            };
            let (Some(sprite), Some(chave)) = (p.sprite, p.chave) else {
                continue;
            };
            if Some(sprite) == ligada {
                alvo = Some((sprite, chave));
                // A leitura da sprite deixou de ser a imagem: quando o Painter a largar, relê-se.
                if p.atual.as_ref().is_some_and(|a| a.fonte == Fonte::Sprite) {
                    p.atual = None;
                }
                continue;
            }
            le_a_sprite(p, &mut self.marcas, Some(&mut *le), ranhuras);
        }
        let Some(t) = painter else {
            return;
        };
        let Some((_, chave)) = alvo else {
            if t.marca_do_artista().is_some() {
                t.poe_a_marca(None);
            }
            return;
        };
        let dims = t.canvas_size();
        let g = self
            .marcas
            .entry(chave)
            .or_insert_with(|| Guardada::nova(dims, chave.1));
        // O Painter com a imagem noutro tamanho: nada a marcar, e a marca guardada fica.
        if g.vale().tamanho() != dims {
            if t.marca_do_artista().is_some() {
                t.poe_a_marca(None);
            }
            return;
        }
        // Ligar outra imagem limpa a marca do Painter (`bind_document`): sem marca, ou de outra
        // gravação, ele recebe a desta.
        let fresca = t
            .marca_do_artista()
            .is_none_or(|m| m.epoca() != chave.1 || m.tamanho() != dims);
        if fresca {
            let m = g.vale().clone();
            t.poe_a_marca(Some(m.clone()));
            g.marca = m;
        } else if let Some(m) = t.marca_do_artista()
            && m.versao() != g.marca.versao()
        {
            g.marca = m.clone();
        }
        // O Painter tem a imagem: a base nasce quando ele a largar.
        g.base = None;
        g.efetiva = None;
    }

    /// **As marcas para gravar no projeto**: a de cada imagem com macro na gravação que ela toca agora (a
    /// do Painter ligado, viva). Vazio = nenhuma marca.
    #[must_use]
    pub fn marcas_a_gravar(&self, painter: Option<&PainterTool>) -> Vec<u8> {
        let mut vivas: BTreeMap<ChaveDaMarca, &MarcaDoArtista> = BTreeMap::new();
        for l in self.lidos.values() {
            let Ok(Some(p)) = &l.palco else {
                continue;
            };
            let Some(chave) = p.chave else {
                continue;
            };
            let do_painter = painter
                .filter(|t| t.bound_document().is_some() && t.bound_document() == p.sprite)
                .and_then(PainterTool::marca_do_artista)
                .filter(|m| m.epoca() == chave.1);
            if let Some(m) = do_painter.or_else(|| self.marcas.get(&chave).map(Guardada::vale))
                && !m.vazia()
            {
                vivas.insert(chave, m);
            }
        }
        if vivas.is_empty() {
            return Vec::new();
        }
        let mut out = vec![FORMATO];
        out.extend_from_slice(&(vivas.len() as u32).to_le_bytes());
        for ((sid, _), m) in vivas {
            let b = m.codifica();
            out.extend_from_slice(&sid.to_le_bytes());
            out.extend_from_slice(&(b.len() as u32).to_le_bytes());
            out.extend_from_slice(&b);
        }
        out
    }

    /// **As marcas de um projeto aberto** ([`Self::marcas_a_gravar`]); `false` = bytes de outro formato
    /// (nada se restaura).
    pub fn restaura_marcas(&mut self, b: &[u8]) -> bool {
        self.marcas.clear();
        if b.is_empty() {
            return true;
        }
        let Some(lidas) = le_marcas(b) else {
            return false;
        };
        for (sid, m) in lidas {
            self.marcas.insert((sid, m.epoca()), Guardada::de(m));
        }
        true
    }
}

/// A versão do formato de [`PalcosDoPainter::marcas_a_gravar`].
const FORMATO: u8 = 1;

fn le_marcas(b: &[u8]) -> Option<Vec<(u64, MarcaDoArtista)>> {
    let (&FORMATO, mut b) = b.split_first()? else {
        return None;
    };
    let n = u32::from_le_bytes(*b.split_first_chunk::<4>()?.0);
    b = &b[4..];
    let mut out = Vec::new();
    for _ in 0..n {
        let (sid, r) = b.split_first_chunk::<8>()?;
        let (len, r) = r.split_first_chunk::<4>()?;
        let len = u32::from_le_bytes(*len) as usize;
        let m = MarcaDoArtista::descodifica(r.get(..len)?)?;
        out.push((u64::from_le_bytes(*sid), m));
        b = &r[len..];
    }
    b.is_empty().then_some(out)
}
