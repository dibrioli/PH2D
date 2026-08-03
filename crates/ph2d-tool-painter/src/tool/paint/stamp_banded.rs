//! **O LOTE de dabs em bandas de linhas** — a granularidade que faltava no depósito de pigmento.
//!
//! # O defeito que este módulo corrige
//!
//! O kernel de um dab já se divide entre os núcleos ([`ph2d_painter_brush`], `parallel_band_stamp`), e o
//! piso dele — `PARALLEL_MIN_AREA`, ~131 k px — está calibrado **para um dab**: *"small brush dabs stay
//! serial; large Anchored stamps parallelise"*. Um dab de pincel comum (r = 24) cobre **2,3 k px** e
//! nunca o alcança, **corretamente** — abrir threads para isso perde.
//!
//! Só que os métodos de **re-stamp** (Line · Curve · Ellipse · Polygon · Free Hand) não carimbam um dab:
//! eles re-carimbam a FIGURA INTEIRA a cada quadro. Medido em 2026-08-03 (`measure_shape_cost`), uma
//! elipse de raio 400 são **525 dabs** sobre uma união de ~722 k px — **5,5× o piso como lote**, e
//! nenhum deles perto dele sozinho. O resultado é que o depósito roda **em um núcleo de trinta e dois**,
//! e um move custa **30 ms no Digital, 67 no Impasto, 119 no Wet Paint**.
//!
//! ⚠️ **E o depósito não é ineficiente — ele é REPETIDO.** O mesmo comprimento de caminho custa
//! **1,02×** à mão livre e por re-stamp; o que difere é que a mão livre carimba o pedaço novo e o
//! re-stamp refaz a figura toda. Por isso a cura é *dividir o lote*, não *acelerar o dab*.
//!
//! # A banda é uma TELA VIRTUAL, e é isso que dispensa um kernel novo
//!
//! Cada banda recebe a fatia `[by0, by1)` do canvas, a **altura da banda** como altura da tela e o
//! centro do dab **deslocado por `by0`**. O kernel então computa `v = (py + 0,5 − cy) · inv_radius` com
//! `py` e `cy` deslocados pelo mesmo tanto — **o mesmo número** — e o recorte às linhas da banda cai do
//! `clamp` que ele já faz contra a altura da tela. Nenhuma segunda resposta a *"que forma tem este
//! dab?"*: é o kernel do produto, com outra moldura.
//!
//! # Por que é byte-idêntico, e não "quase"
//!
//! As bandas são linhas **disjuntas** e cada banda percorre **TODOS os dabs na ordem da lista**, então
//! um pixel qualquer é composto pelos mesmos dabs, na mesma ordem, com a mesma aritmética — muda quem
//! AVALIA a linha, nunca o que a linha diz (o invariante do ADR-0109). O kernel lê e escreve só o
//! próprio pixel, então não há leitura de vizinho para atravessar uma fronteira de banda.
//!
//! # O que este módulo NÃO faz, de propósito
//!
//! Ele serve o pincel de **falloff puro** — sem Shape, sem Grain, sem cap de Accumulate. Não é um
//! recorte de conveniência: é exatamente o pincel que o artista abre por default, e é a rota que o
//! próprio dispatch nomeia (*"a bare falloff brush stays on the per-pixel path"*). As outras rotas têm
//! estado por-dab (o stream de RNG das bases de textura) ou compartilhado (a máscara canvas-shaped do
//! cap), e cada uma é uma pergunta própria — **entram quando forem medidas**, não por simetria.

use ph2d_painter_brush::{BrushSpec, Dab, DirtyRect, dab_write_bounds, stamp_dab_textured_masked};

/// Área da união abaixo da qual o lote não vale uma divisão — o irmão do `PARALLEL_MIN_AREA` do kernel,
/// um nível acima.
///
/// ⚠️ **É o mesmo número, e isso é deliberado:** a pergunta *"vale abrir threads para esta quantidade de
/// trabalho?"* não muda por quem a faz, e dois pisos diferentes seriam duas respostas para ela. O que
/// muda é **sobre o que** ela é feita — lá, um dab; aqui, o lote.
pub(super) const BATCH_MIN_AREA: usize = 1 << 17;

/// A união dos retângulos de escrita declarados do lote — `None` se nenhum dab escreve.
///
/// ⚠️ Usa [`dab_write_bounds`], o **superconjunto declarado** que o journal já consome, e não a janela
/// exata do kernel. Para a rejeição por banda a direção segura é o superconjunto: rejeitar de menos
/// custa uma chamada que não desenha nada; rejeitar de mais **some com tinta em silêncio**.
fn batch_bounds(dabs: &[Dab], w: u32, h: u32) -> Option<DirtyRect> {
    dabs.iter()
        .filter_map(|d| dab_write_bounds(d.center, d.radius_px, w, h))
        .fold(None, |acc: Option<DirtyRect>, r| {
            Some(acc.map_or(r, |a| union(a, r)))
        })
}

/// A união de dois retângulos sujos — uma porta, quatro consumidores.
fn union(a: DirtyRect, b: DirtyRect) -> DirtyRect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    DirtyRect {
        x,
        y,
        w: x1 - x,
        h: y1 - y,
    }
}

/// **Vale dividir este lote?** — a porta ÚNICA, perguntada pelo produto para DECIDIR e pelo gate para
/// AFIRMAR.
///
/// ⚠️ **A régua é a SOMA DAS PEGADAS, não a área do bbox** — e a diferença é medida, não estética. O
/// trabalho de um lote é `Σ área(dab)`, e os dabs de um traço se SOBREPÕEM fortemente (espaçamento ~10%
/// do diâmetro), então a união é muito menor que o trabalho: numa elipse de raio 100 são 65 dabs =
/// 156 k visitas de pixel sobre um bbox de 62 k. Perguntar pelo bbox mandava exatamente essa janela —
/// 9,1 ms de trabalho real — para a rota serial, porque a caixa dela é pequena.
///
/// ⚠️ E ela existe como FUNÇÃO, não como aritmética inline, porque um gate que recomputa a regra por
/// conta própria afirma a própria conta e fica verde com o produto decidindo outra coisa — foi o que a
/// primeira versão deste gate fazia.
pub(super) fn wants_bands(dabs: &[Dab], w: u32, h: u32, min_area: usize) -> bool {
    if dabs.len() < 2 {
        return false;
    }
    let Some(bounds) = batch_bounds(dabs, w, h) else {
        return false;
    };
    if (bounds.h as usize) <= 1 {
        return false;
    }
    let work: usize = dabs
        .iter()
        .filter_map(|d| dab_write_bounds(d.center, d.radius_px, w, h))
        .map(|b| (b.w as usize) * (b.h as usize))
        .sum();
    work >= min_area
}

/// Carimba o lote **inteiro** de dabs de falloff puro, dividindo as LINHAS entre os núcleos.
///
/// Devolve o retângulo tocado — a união dos spans dos dabs que de fato escreveram, a mesma resposta do
/// laço serial —, ou `None` se o lote não vale a divisão (aí o chamador segue pelo caminho de sempre).
pub(super) fn stamp_plain_dabs_banded(
    buf: &mut [u8],
    w: u32,
    h: u32,
    dabs: &[Dab],
    brush: &BrushSpec,
    alpha_locked: bool,
) -> Option<DirtyRect> {
    stamp_plain_dabs_banded_with(buf, w, h, dabs, brush, alpha_locked, BATCH_MIN_AREA)
}

/// [`stamp_plain_dabs_banded`] com o piso **explícito** — a rota de ablação.
///
/// ⚠️ Com `min_area = usize::MAX` o braço serial é literalmente o laço `for d in dabs` que o
/// `stamp_dabs_per_pixel` rodava antes desta função existir, chamando o mesmo kernel com os mesmos
/// argumentos. É isso que torna o gate de identidade uma comparação contra **o produto**, e não contra
/// uma segunda implementação escrita para o teste.
#[allow(clippy::too_many_arguments)]
pub(super) fn stamp_plain_dabs_banded_with(
    buf: &mut [u8],
    w: u32,
    h: u32,
    dabs: &[Dab],
    brush: &BrushSpec,
    alpha_locked: bool,
    min_area: usize,
) -> Option<DirtyRect> {
    let serial = |buf: &mut [u8]| -> Option<DirtyRect> {
        let mut touched: Option<DirtyRect> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            if let Some(r) = stamp_dab_textured_masked(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                None,
                None,
                None,
                None,
                spec.dab_rotor(d),
            ) {
                touched = Some(touched.map_or(r, |a| union(a, r)));
            }
        }
        touched
    };
    if dabs.len() < 2 {
        return serial(buf);
    }
    let bounds = batch_bounds(dabs, w, h)?;
    let rows = bounds.h as usize;
    if !wants_bands(dabs, w, h, min_area) {
        return serial(buf);
    }
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, rows);
    if threads < 2 {
        return serial(buf);
    }
    let stride = (w as usize) * 4;
    let rows_per_band = rows.div_ceil(threads);
    let y_top = bounds.y as usize;
    let region = &mut buf[y_top * stride..(y_top + rows) * stride];

    // Uma banda percorre TODOS os dabs, recortando cada um às suas próprias linhas. O dab que não
    // alcança a banda é rejeitado por duas comparações — `dabs × bandas` testes, ruído contra o blit.
    let band_work = |chunk: &mut [u8], band_y0: u32| -> Option<DirtyRect> {
        let band_h = (chunk.len() / stride) as u32;
        let band_y1 = band_y0 + band_h;
        let mut touched: Option<DirtyRect> = None;
        for d in dabs {
            let Some(b) = dab_write_bounds(d.center, d.radius_px, w, h) else {
                continue;
            };
            if b.y + b.h <= band_y0 || b.y >= band_y1 {
                continue;
            }
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            // A banda é a TELA: altura da banda, centro deslocado. O `v` do kernel sai idêntico e o
            // recorte é o `clamp` que ele já faz.
            let r = stamp_dab_textured_masked(
                chunk,
                w,
                band_h,
                [d.center[0], d.center[1] - band_y0 as f32],
                &spec,
                d.coverage,
                alpha_locked,
                None,
                None,
                None,
                None,
                spec.dab_rotor(d),
            );
            if let Some(r) = r {
                // De volta às coordenadas do canvas.
                let r = DirtyRect {
                    x: r.x,
                    y: r.y + band_y0,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(r, |a| union(a, r)));
            }
        }
        touched
    };
    let band_work = &band_work;
    #[allow(clippy::cast_possible_truncation)]
    std::thread::scope(|s| {
        region
            .chunks_mut(rows_per_band * stride)
            .enumerate()
            .map(|(bi, chunk)| {
                let band_y0 = (y_top + bi * rows_per_band) as u32;
                s.spawn(move || band_work(chunk, band_y0))
            })
            // Colhe primeiro para que TODA banda seja juntada (sem short-circuit deixando thread solta).
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|h| h.join().unwrap_or(None))
            .fold(None, |acc: Option<DirtyRect>, r| {
                Some(acc.map_or(r, |a| union(a, r)))
            })
    })
}
