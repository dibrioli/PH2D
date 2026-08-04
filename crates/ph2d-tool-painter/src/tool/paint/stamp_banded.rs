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
//! # O CAP entrou, e o que o deixava de fora era uma PREMISSA que envelheceu
//!
//! Até 2026-08-04 este módulo servia o pincel de **falloff puro** — sem Shape, sem Grain, **sem cap de
//! Accumulate** —, e a razão escrita era que o cap tem *"estado compartilhado (a máscara canvas-shaped)"*.
//! ⚠️ **Compartilhado entre DABS, não entre LINHAS.** A máscara é lida e escrita **por-texel**, no mesmo
//! índice do pixel que o dab acabou de compor; bandas são linhas **disjuntas**, então nenhuma banda lê
//! um byte que outra escreve — exatamente o invariante do ADR-0109 que o `buf` já satisfazia. Uma
//! fatia paralela (`stride` para a tinta, `stride / 4` para a máscara) e a rota vale para os dois.
//!
//! ⚠️ **E o alcance não é o impasto:** `stroke_cover_wanted` dispara em **`strength < 1`**, que é ajuste
//! comum de pincel digital, *e* no AA do filme de todo pincel de impasto. Com um shape editor (Line ·
//! Curve · Ellipse · Polygon · Free Hand) o lote são centenas de dabs pequenos — nenhum perto do piso
//! do kernel — e o lote inteiro rodava **num núcleo de trinta e dois**.
//!
//! # O que este módulo NÃO faz, de propósito
//!
//! Ele segue **sem Shape e sem Grain**: essas rotas têm estado **por-dab** (o stream de RNG das bases
//! de textura), que é sequencial por semântica — a banda não pode avançá-lo sem decidir qual banda o
//! avança. Isso é uma pergunta própria, e **entra quando for medida**, não por simetria.

use ph2d_painter_brush::{
    BrushSpec, Dab, DirtyRect, dab_write_bounds, stamp_dab_textured_masked,
    stamp_dab_textured_masked_with,
};

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
/// O TRABALHO do lote: a soma das pegadas, em visitas de texel.
///
/// ⚠️ **É esta a grandeza que se publica, nunca o raio.** O custo de um lote é quadrático no raio
/// do pincel, então um log que traz `dabs` e omite o raio convida a uma aritmética infundada — eu
/// derivei um `ns/visita` de um raio ASSUMIDO antes de escrever isto. A soma das pegadas não
/// assume nada: ela É o trabalho.
pub(super) fn batch_work(dabs: &[Dab], w: u32, h: u32) -> usize {
    dabs.iter()
        .filter_map(|d| dab_write_bounds(d.center, d.radius_px, w, h))
        .map(|b| (b.w as usize) * (b.h as usize))
        .sum()
}

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
    batch_work(dabs, w, h) >= min_area
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
    mask: Option<&mut [u8]>,
) -> Option<DirtyRect> {
    stamp_plain_dabs_banded_with(buf, w, h, dabs, brush, alpha_locked, mask, BATCH_MIN_AREA)
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
    mask: Option<&mut [u8]>,
    min_area: usize,
) -> Option<DirtyRect> {
    // ⚠️ **O balde é fechado DEPOIS do trabalho, não antes.** Ele era carimbado aqui em cima com os
    // dabs e as visitas e mais nada, então o `ns/visita` do log tinha de buscar um numerador noutro
    // lugar — e buscava no RE-STAMP, um evento diferente. Medir onde se conta é o que torna a razão
    // uma razão.
    let bands = wants_bands(dabs, w, h, min_area);
    let work = batch_work(dabs, w, h);
    let t0 = std::time::Instant::now();
    let out = stamp_plain_dabs_banded_run(buf, w, h, dabs, brush, alpha_locked, mask, bands);
    #[allow(clippy::cast_possible_truncation)]
    diag::note(bands, dabs.len(), work, t0.elapsed().as_micros() as u64);
    out
}

/// O laço `for d in dabs` de sempre — **o código que este módulo existe para NÃO mudar**.
///
/// ⚠️ Ele era uma closure `&dyn Fn(&mut [u8])`, e a máscara o obrigou a virar função: um `dyn Fn` não
/// pode segurar um `&mut [u8]` emprestado do chamador, e capturá-lo por valor tiraria a máscara de quem
/// precisa dela no braço em banda. Uma `fn` com a máscara no argumento diz a mesma coisa e deixa o
/// borrow checker escolher o braço.
fn stamp_plain_serial(
    buf: &mut [u8],
    w: u32,
    h: u32,
    dabs: &[Dab],
    brush: &BrushSpec,
    alpha_locked: bool,
    mut mask: Option<&mut [u8]>,
) -> Option<DirtyRect> {
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
            mask.as_deref_mut(),
            spec.dab_rotor(d),
        ) {
            touched = Some(touched.map_or(r, |a| union(a, r)));
        }
    }
    touched
}

/// A execução, separada da contagem — o `?` e os recuos dela ficam intactos.
#[allow(clippy::too_many_arguments)]
fn stamp_plain_dabs_banded_run(
    buf: &mut [u8],
    w: u32,
    h: u32,
    dabs: &[Dab],
    brush: &BrushSpec,
    alpha_locked: bool,
    mask: Option<&mut [u8]>,
    bands: bool,
) -> Option<DirtyRect> {
    if dabs.len() < 2 {
        return stamp_plain_serial(buf, w, h, dabs, brush, alpha_locked, mask);
    }
    let bounds = batch_bounds(dabs, w, h)?;
    let rows = bounds.h as usize;
    if !bands {
        return stamp_plain_serial(buf, w, h, dabs, brush, alpha_locked, mask);
    }
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, rows);
    if threads < 2 {
        return stamp_plain_serial(buf, w, h, dabs, brush, alpha_locked, mask);
    }
    let stride = (w as usize) * 4;
    // ⚠️ **Uma régua, DOIS buffers.** A tinta anda `stride`, a máscara `stride / 4` — e as duas fatias
    // têm de concordar sobre onde uma banda começa, senão o cap é lido no lugar errado e a lei de
    // cobertura sai deslocada por linhas ([[feedback_derived_coordinate_seed_must_match_sample]]).
    let mrow = w as usize;
    let rows_per_band = rows.div_ceil(threads);
    let y_top = bounds.y as usize;
    let region = &mut buf[y_top * stride..(y_top + rows) * stride];
    let mask_region = mask.map(|m| &mut m[y_top * mrow..(y_top + rows) * mrow]);

    // Uma banda percorre TODOS os dabs, recortando cada um às suas próprias linhas. O dab que não
    // alcança a banda é rejeitado por duas comparações — `dabs × bandas` testes, ruído contra o blit.
    let band_work =
        |chunk: &mut [u8], mut mchunk: Option<&mut [u8]>, band_y0: u32| -> Option<DirtyRect> {
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
                // recorte é o `clamp` que ele já faz — e a máscara da banda é indexada nas MESMAS
                // coordenadas locais, então ela cai no texel certo sem tradução nenhuma.
                //
                // ⚠️ **`usize::MAX`: o paralelismo é do LOTE, não do dab.** O kernel dividiria de novo
                // dentro de cada banda (alcançável a 4096² com o pincel máximo — a nota da porta traz o
                // número), e trinta e duas bandas abrindo trinta e duas threads cada é contenção, não
                // capacidade. No braço serial acima ele SEGUE livre para dividir: lá não há quem o faça.
                let r = stamp_dab_textured_masked_with(
                    chunk,
                    w,
                    band_h,
                    [d.center[0], d.center[1] - band_y0 as f32],
                    &spec,
                    d.coverage,
                    alpha_locked,
                    mchunk.as_deref_mut(),
                    spec.dab_rotor(d),
                    usize::MAX,
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
    // ⚠️ Os dois iteradores avançam JUNTOS (`zip` não serve: a máscara é opcional), e o `next` só é
    // chamado uma vez por banda — é isso que mantém a n-ésima fatia de tinta emparelhada com a
    // n-ésima de cobertura.
    let mut mask_bands = mask_region.map(|m| m.chunks_mut(rows_per_band * mrow));
    #[allow(clippy::cast_possible_truncation)]
    std::thread::scope(|s| {
        region
            .chunks_mut(rows_per_band * stride)
            .enumerate()
            .map(|(bi, chunk)| {
                let mchunk = mask_bands.as_mut().and_then(Iterator::next);
                let band_y0 = (y_top + bi * rows_per_band) as u32;
                s.spawn(move || band_work(chunk, mchunk, band_y0))
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

/// **Qual rota o carimbo tomou** — o instrumento que o report de 2026-08-03 exigiu.
///
/// ⚠️ Ele existe porque eu media o EVENTO, celebrei 10× e o produto não sentiu: sem saber se o ramo em
/// banda sequer DISPARA no documento do artista, *"não melhorou"* admite duas leituras opostas — a rota
/// não é tomada, ou é tomada e o tempo está noutro lugar. Um número no log separa as duas.
///
/// ⚠️ **UM leitor só:** `take` ZERA os contadores, então dois leitores publicariam pedaços do mesmo
/// quadro como se fossem quadros — a mesma lei do `wash_diag`. O leitor é a linha `[paint-perf]`.
pub mod diag {
    use std::cell::Cell;

    // ⚠️ **POR THREAD, não atômicos globais.** Estes baldes são lidos por um gate que afirma
    // `banda > 0`, e poluição só SOMA — um teste vizinho carimbando em paralelo tornaria essa
    // afirmação verdadeira mesmo com a fixture tomando a rota serial: **falso VERDE**, a metade
    // exata que a flake do `ph2d-painter-brush` (2026-08-01) provou ser a vulnerável. Contador
    // por thread torna a poluição estruturalmente impossível e não deixa lista de quem precisa
    // se isolar.
    //
    // ⚠️ **O invariante que isto assume:** quem CARIMBA e quem LÊ são a mesma thread. No app são
    // as duas o laço do quadro (o flush coalescido roda antes do `cpu_start`, e o `[frame]` o lê
    // no fim). `note` é chamado ANTES do `thread::scope` do lote, então as bandas não contam.
    // Se um dia o carimbo sair da thread do quadro, este balde **emudece** — e um balde mudo
    // lê-se como resultado; quem mover o carimbo move este contador junto.
    thread_local! {
        static BANDED: Cell<u32> = const { Cell::new(0) };
        static SERIAL: Cell<u32> = const { Cell::new(0) };
        static DABS: Cell<u32> = const { Cell::new(0) };
        static VISITS: Cell<u64> = const { Cell::new(0) };
        static CPU_US: Cell<u64> = const { Cell::new(0) };
        static DELIVERIES: Cell<u32> = const { Cell::new(0) };
        static DEVICE: Cell<u32> = const { Cell::new(0) };
        static DEV_DABS: Cell<u32> = const { Cell::new(0) };
        static DEV_VISITS: Cell<u64> = const { Cell::new(0) };
        static DEV_US: Cell<u64> = const { Cell::new(0) };
        static RESTORE_US: Cell<u64> = const { Cell::new(0) };
        static RELIEF_US: Cell<u64> = const { Cell::new(0) };
        static SAVE_US: Cell<u64> = const { Cell::new(0) };
        static STAMP_US: Cell<u64> = const { Cell::new(0) };
    }

    /// **O lote foi para o DISPOSITIVO?** — o instrumento que a wave da GPU exige pela mesma razão
    /// que o `banded`/`serial` existe: sem ele, *"não melhorou"* admite duas leituras opostas (a
    /// rota não é tomada · é tomada e o tempo está noutro lugar), e as curas são opostas.
    /// ⚠️ **O trabalho do device é contado AQUI, e por isso a linha descreve o TRAÇO.** Quando ele
    /// aceita o lote, o `stamp_plain_dabs_banded` nem é chamado — então até 2026-08-04 os `dabs` e as
    /// `visitas` do log eram os da **metade que ficou na CPU**, com o log dizendo `775 dabs` para um
    /// traço que carimbou mais. Um contador que descreve um subconjunto sem dizer qual é um número
    /// que se lê como o todo.
    pub(in crate::tool::paint) fn note_device(dabs: usize, work: usize, us: u64) {
        DEVICE.with(|c| c.set(c.get().saturating_add(1)));
        DEV_DABS.with(|c| {
            c.set(
                c.get()
                    .saturating_add(u32::try_from(dabs).unwrap_or(u32::MAX)),
            )
        });
        DEV_VISITS.with(|c| c.set(c.get().saturating_add(work as u64)));
        DEV_US.with(|c| c.set(c.get().saturating_add(us)));
    }

    pub(super) fn note(banded: bool, dabs: usize, work: usize, us: u64) {
        let bucket = if banded { &BANDED } else { &SERIAL };
        bucket.with(|c| c.set(c.get().saturating_add(1)));
        DABS.with(|c| {
            c.set(
                c.get()
                    .saturating_add(u32::try_from(dabs).unwrap_or(u32::MAX)),
            )
        });
        VISITS.with(|c| c.set(c.get().saturating_add(work as u64)));
        CPU_US.with(|c| c.set(c.get().saturating_add(us)));
    }

    /// As QUATRO fases de um quadro de re-stamp, em µs.
    ///
    /// ⚠️ **Elas dividem o dreno com os baldes do lote de propósito.** Descrevem o MESMO evento (uma
    /// entrega de ponteiro que re-carimba a figura), e dois drenos publicariam janelas diferentes
    /// como se fossem a mesma — a lei do `wash_diag`, que esta sessão já pagou duas vezes.
    pub(in crate::tool::paint) fn note_restamp(
        restore_us: u64,
        relief_us: u64,
        save_us: u64,
        stamp_us: u64,
    ) {
        DELIVERIES.with(|c| c.set(c.get().saturating_add(1)));
        RESTORE_US.with(|c| c.set(c.get().saturating_add(restore_us)));
        RELIEF_US.with(|c| c.set(c.get().saturating_add(relief_us)));
        SAVE_US.with(|c| c.set(c.get().saturating_add(save_us)));
        STAMP_US.with(|c| c.set(c.get().saturating_add(stamp_us)));
    }

    /// O que o depósito fez desde a última chamada — e ZERA.
    ///
    /// Como ele ZERA, há **um leitor só** por thread (a lei do `wash_diag`): dois leitores
    /// publicariam pedaços do mesmo quadro como se fossem quadros.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DepositDiag {
        /// Lotes que tomaram a rota em BANDA.
        pub banded: u32,
        /// Lotes carimbados pelo DISPOSITIVO (subconjunto dos que entram no ramo de falloff puro).
        pub device: u32,
        /// Lotes que ficaram seriais (pequenos demais para dividir).
        pub serial: u32,
        /// Dabs somados sobre os lotes que ficaram na **CPU** (banda + serial).
        pub dabs: u32,
        /// **Visitas de texel** da CPU — a soma das pegadas. É o TRABALHO, e não assume raio nenhum.
        pub visits: u64,
        /// µs gastos carimbando na CPU. ⚠️ Numerador e denominador do `ns/visita` da CPU vêm do
        /// **mesmo** evento — a razão anterior dividia os µs do RE-STAMP pelas visitas do DEPÓSITO,
        /// duas populações diferentes, e por isso imprimia `0.0` em toda sessão de mão livre.
        pub cpu_us: u64,
        /// O mesmo trio, para os lotes que o DISPOSITIVO aceitou.
        pub dev_dabs: u32,
        pub dev_visits: u64,
        pub dev_us: u64,
        /// Quadros de re-stamp (entregas que passaram pelo `stamp_drag_preview`).
        pub deliveries: u32,
        /// µs somados em cada fase de um quadro de re-stamp.
        pub restore_us: u64,
        pub relief_us: u64,
        pub save_us: u64,
        pub stamp_us: u64,
    }

    #[must_use]
    pub fn take() -> DepositDiag {
        DepositDiag {
            banded: BANDED.with(Cell::take),
            serial: SERIAL.with(Cell::take),
            device: DEVICE.with(Cell::take),
            dabs: DABS.with(Cell::take),
            visits: VISITS.with(Cell::take),
            cpu_us: CPU_US.with(Cell::take),
            dev_dabs: DEV_DABS.with(Cell::take),
            dev_visits: DEV_VISITS.with(Cell::take),
            dev_us: DEV_US.with(Cell::take),
            deliveries: DELIVERIES.with(Cell::take),
            restore_us: RESTORE_US.with(Cell::take),
            relief_us: RELIEF_US.with(Cell::take),
            save_us: SAVE_US.with(Cell::take),
            stamp_us: STAMP_US.with(Cell::take),
        }
    }
}
