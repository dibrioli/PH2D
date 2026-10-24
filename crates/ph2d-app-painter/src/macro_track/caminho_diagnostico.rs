//! **O diagnóstico por quadro** das cenas `PH2D_MACRO_SMOKE=5|6` (filho `#[path]` do `caminho.rs`):
//! a [`Espia`] do que a subida mostrou e [`PalcosDoPainter::diagnostico`], a linha do registo.

use super::super::{Caixa, PalcoDaPintura, PalcosDoPainter, TrocaDaSprite};
use super::Variante;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;

/// Texels com tinta numa imagem RGBA8 (premultiplicada ou não: o branco opaco é o papel).
fn pintados(rgba: &[u8]) -> usize {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0 && p[0].min(p[1]).min(p[2]) < 235)
        .count()
}

/// **O que a cena mostrou** (ligada na 1.ª linha de diagnóstico): uma cópia de cada ranhura, refeita a cada subida de
/// [`PalcosDoPainter::sobe`], e as trocas dessa subida — o que o extract do quadro usou, não uma conta à
/// parte do que a ranhura devia ter.
#[derive(Default)]
pub(in super::super) struct Espia {
    /// Por ranhura: (largura, rgba premultiplicado).
    ranhuras: std::collections::BTreeMap<u32, (u32, Vec<u8>)>,
    pub(in super::super) trocas: Vec<TrocaDaSprite>,
}

impl Espia {
    /// A subida de `rgba` à ranhura `id` (de largura `w`): inteira (`caixa = None`) ou só a caixa.
    pub(in super::super) fn copia(&mut self, id: u32, w: u32, caixa: Option<Caixa>, rgba: &[u8]) {
        let Some((x, y, cw, _)) = caixa else {
            self.ranhuras.insert(id, (w, rgba.to_vec()));
            return;
        };
        // Uma ranhura que nasceu antes da espia não se conhece: fica de fora até subir inteira.
        let Some((lw, t)) = self.ranhuras.get_mut(&id) else {
            return;
        };
        let (lw, x, cw) = (*lw as usize, x as usize, cw as usize);
        for (j, linha) in rgba.chunks_exact(cw * 4).enumerate() {
            let a = ((y as usize + j) * lw + x) * 4;
            t[a..a + cw * 4].copy_from_slice(linha);
        }
    }
}

impl PalcosDoPainter {
    /// **A linha de diagnóstico da sprite `bits`** (as cenas `=5`/`=6`), escrita no INÍCIO do quadro: o
    /// palco (toca, época, texels pintados nele e na marca dessa época), os do editor e o que a subida
    /// ANTERIOR mostrou de facto ([`Espia`], ligada na 1.ª linha). ⛔ A troca nunca se recalcula aqui: no
    /// quadro depois do Parar a cedência já saiu e o palco ainda é o do anterior (handoff de integração
    /// da W9, «o piscar»).
    #[must_use]
    pub fn diagnostico(
        &mut self,
        bits: u64,
        painter: Option<&PainterTool>,
        mundo: &ph2d_ecs::World,
    ) -> String {
        let espia = self.espia.get_or_insert_default();
        let editor = painter
            .filter(|t| t.bound_document() == Some(bits))
            .and_then(|t| {
                let (w, h) = t.canvas_size();
                t.imagem_aplicada_na_caixa((0, 0, w, h))
            })
            .map_or_else(|| "-".to_owned(), |i| pintados(&i).to_string());
        let Some(l) = self.lidos.get(&(bits, 0)) else {
            return format!("palco=nenhum editor={editor}");
        };
        let p = match &l.palco {
            Err(e) => return format!("palco=erro({e:?}) editor={editor}"),
            Ok(None) => return format!("palco=vazio editor={editor}"),
            Ok(Some(p)) => p,
        };
        let marca = p.chave.and_then(|chave| {
            painter
                .filter(|t| p.sprite.is_some() && t.bound_document() == p.sprite)
                .and_then(PainterTool::marca_do_artista)
                .filter(|m| m.epoca() == chave.1)
                .or_else(|| self.marcas.get(&chave).map(|g| g.vale()))
        });
        let doc = mundo
            .get::<ph2d_ecs::Macro>(ph2d_ecs::Entity::from_bits(bits))
            .map(|m| &m.doc[..]);
        let mut extra = mostrado(p, espia, bits, doc, super::variante() == Variante::Parar);
        if super::variante() == Variante::Arrasta {
            extra.push_str(&confere(p, doc));
        }
        if super::variante() == Variante::Fim {
            extra.push_str(&ao_editor(espia, bits, painter));
        }
        format!(
            "toca={} epoca={:04x} palco={} marca={} editor={editor}{extra}",
            p.toca,
            p.epoca & 0xffff,
            pintados(&p.espelho),
            marca.map_or_else(|| "-".to_owned(), |m| m.quantos().to_string()),
        )
    }
}

/// Texels de `a` (premultiplicada) que diferem de `b` (straight, premultiplicada aqui).
fn diferentes(a: &[u8], b: &[u8]) -> usize {
    let mut b = b.to_vec();
    ph2d_render::premul::premultiply_rgba8(&mut b);
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

/// ` | mostrou: troca=B t=S tinta=N [fresca=M]` — o que a SUBIDA anterior pôs no ecrã (a espia): se
/// trocou a sprite, o instante, os texels com tinta da ranhura e, com `fresca` (a variante `parar`),
/// quantos diferem de uma reprodução À PARTE do `doc` atual nesse instante.
fn mostrado(p: &PalcoDaPintura, e: &Espia, bits: u64, doc: Option<&[u8]>, fresca: bool) -> String {
    let troca = e.trocas.iter().find(|t| t.entity_bits == bits);
    let Some((_, rgba)) = troca.and_then(|t| e.ranhuras.get(&t.texture_id)) else {
        return format!(" | mostrou: troca={}", troca.is_some());
    };
    #[expect(clippy::cast_precision_loss, reason = "µs de um clipe")]
    let s = p.em_cena.tempo().0 as f64 / 1e6;
    let mut linha = format!(" | mostrou: troca=true t={s:.3} tinta={}", pintados(rgba));
    if fresca {
        let n = doc
            .and_then(|d| MacroEmCena::de(d).ok().flatten())
            .and_then(|mut m| m.segue(s).ok().and_then(|()| m.toma_a_tela()))
            .filter(|(a, _, _)| a.len() == rgba.len())
            .map_or_else(
                || "-".to_owned(),
                |(a, _, _)| diferentes(rgba, &a).to_string(),
            );
        let _ = std::fmt::Write::write_fmt(&mut linha, format_args!(" fresca={n}"));
    }
    linha
}

/// ` ≠editor=N` da variante `fim`: os texels que a subida anterior mostrou e diferem da imagem do editor.
fn ao_editor(e: &Espia, bits: u64, painter: Option<&PainterTool>) -> String {
    let troca = e.trocas.iter().find(|t| t.entity_bits == bits);
    let rgba = troca
        .and_then(|t| e.ranhuras.get(&t.texture_id))
        .map(|r| &r.1);
    let editor = painter
        .filter(|t| t.bound_document() == Some(bits))
        .and_then(PainterTool::imagem_aplicada)
        .map(|i| i.0);
    match (rgba, editor) {
        (Some(a), Some(b)) if a.len() == b.len() => format!(" ≠editor={}", diferentes(a, &b)),
        _ => " ≠editor=-".to_owned(),
    }
}

/// ` inteira=N fresca=M` da variante `arrasta`: os texels do palco que diferem da tela inteira do
/// tocador dele (uma cópia, lida inteira) e — com a cabeça parada desde a linha anterior, uma vez por
/// instante — de uma reprodução À PARTE de `doc` no mesmo instante.
fn confere(p: &PalcoDaPintura, doc: Option<&[u8]>) -> String {
    static PARADO: std::sync::Mutex<(u64, bool)> = std::sync::Mutex::new((u64::MAX, false));
    let inteira = p
        .em_cena
        .tocador()
        .instantaneo()
        .ok()
        .and_then(|mut c| c.toma_a_tela())
        .filter(|(a, _, _)| a.len() == p.espelho.len())
        .map_or_else(
            || "-".to_owned(),
            |(a, _, _)| diferentes(&p.espelho, &a).to_string(),
        );
    let t = p.em_cena.tempo().0;
    let mut parado = PARADO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let fresca = if parado.0 != t {
        *parado = (t, false);
        None
    } else if std::mem::replace(&mut parado.1, true) {
        None
    } else {
        #[expect(clippy::cast_precision_loss, reason = "µs de um clipe")]
        let s = t as f64 / 1e6;
        doc.and_then(|d| MacroEmCena::de(d).ok().flatten())
            .and_then(|mut m| m.segue(s).ok().and_then(|()| m.toma_a_tela()))
            .filter(|(a, _, _)| a.len() == p.espelho.len())
            .map(|(a, _, _)| diferentes(&p.espelho, &a))
    };
    format!(
        " inteira={inteira} fresca={}",
        fresca.map_or_else(|| "-".to_owned(), |n| n.to_string())
    )
}
