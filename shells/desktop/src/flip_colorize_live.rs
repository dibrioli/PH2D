//! **A SESSÃO VIVA do Colorize** — o Trap/Bleed ajustável depois do Apply, e o worker que o
//! tira da thread de UI (`09 §7.2`). Irmão do `flip_colorize.rs` pelo teto de LOC do shell
//! (HR-18, 600).

use super::{
    ColorRegion, DrawingId, FlipObjectId, FlipStroke, Job, Scribble, Vec2, colorize_regions,
    install_regions, precision_and_trap,
};

/// Um quadro que o Apply escreveu — a base congelada dele e quantos strokes saíram.
///
/// ⚠️ **É por QUADRO, não do gesto**: o Apply multiframe (C3) escreve em N desenhos, e a
/// linha de cada quadro é diferente, então o número de regiões também é. Uma contagem só,
/// do gesto, tornaria o guard de segurança inútil em todos menos um.
pub(super) struct LiveFrame {
    pub(super) did: DrawingId,
    /// As fronteiras da BASE deste quadro, congeladas no Apply. Durante um ajuste ao vivo a
    /// base não muda por definição — e é isto que permite ao corte rodar **fora da thread de
    /// UI**: o worker recebe geometria clonada e nunca vê o `FlipDoc`.
    pub(super) lines: Vec<(Vec<Vec2>, Vec<f32>, bool)>,
    /// Os strokes do desenho ANTES da 1ª inserção — a base congelada.
    pub(super) base: Vec<FlipStroke>,
    /// Quantos strokes a última (re)inserção produziu neste quadro. O **guard de
    /// segurança**: se o desenho não é mais `base.len() + produced`, o artista editou outra
    /// coisa (nova linha, balde, borracha) e a sessão MORRE — re-Aplicar apagaria a edição.
    pub(super) produced: usize,
}

/// O que o worker do ajuste ao vivo devolve: as regiões de CADA quadro, mais os parâmetros
/// que as produziram.
///
/// ⚠️ Os parâmetros voltam JUNTO em vez de serem lidos do painel na chegada: entre a saída e
/// a volta o artista pode ter movido o slider de novo, e gravar o valor de agora marcaria
/// como honrado um pedido que ninguém computou — a sessão pararia de recalcular, presa num
/// resultado velho que se declara atual (a mesma lei do readout do ADR-0125).
pub(super) struct LiveResult {
    pub(super) regions: Vec<Vec<ColorRegion>>,
    pub(super) trap: f64,
    pub(super) bleed: f64,
}

pub(super) struct LiveApply {
    /// Rótulo → cor (a paleta que o Apply montou dos rabiscos).
    pub(super) palette: Vec<[u8; 4]>,
    /// As sementes já em LOCAL do desenho, congeladas no Apply — uma função pura de
    /// `(sementes, trap, bleed)`, então o re-Apply é determinístico.
    ///
    /// ⚠️ **UMA lista serve todos os quadros, e não é atalho: é o que o onion fill É.** O
    /// rabisco é autorado em MUNDO por cima das poses empilhadas, e a `w2l` é do OBJETO (não
    /// do desenho) ⇒ o LOCAL é o mesmo em todo quadro da camada. O que muda entre eles é a
    /// LINHA, e é por isso que cada um resolve por conta própria.
    pub(super) seeds: Vec<Scribble>,
    /// O objeto onde as regiões foram escritas.
    pub(super) oid: FlipObjectId,
    /// Os quadros tocados — o ativo é o **primeiro** (a âncora do `targets`).
    pub(super) frames: Vec<LiveFrame>,
    /// Os últimos `(trap, bleed)` **aplicados** — o re-Apply dispara quando o painel diverge.
    pub(super) trap: f64,
    pub(super) bleed: f64,
    /// O corte em voo (no máximo UM). Enquanto ele existe, mexer o slider só muda o alvo;
    /// quando ele volta, se o alvo mudou, sai outro. É o rate-limiter inteiro — sem timer,
    /// sem constante a calibrar.
    pub(super) job: Option<Job<LiveResult>>,
}

impl crate::App {
    /// **Trap/Bleed em tempo real depois do Apply** (6º smoke, pedido do Enio: *"trap e bleed
    /// não estão em tempo real após apply. faça ficar em tempo real para ajustes"*) —
    /// **fora da thread de UI** (`09 §7.2`).
    ///
    /// Roda no prólogo do frame (ao lado do drain do Apply, com `self` livre). Quando o Trap
    /// ou o Bleed mudou desde a última rodada, o corte é re-executado com os parâmetros novos
    /// e as regiões substituem as anteriores sobre a base congelada — o *"ajustar a última
    /// operação"* do Blender.
    ///
    /// # Por que assíncrono, e por que NÃO foi uma escolha
    ///
    /// O `§7.2` declarou o kill-criterion **antes** do build: *"se o solve de um quadro ficar
    /// acima de um frame de 60 fps (16 ms), o Colorize não é síncrono — ele copia o padrão
    /// `progress`"*. Medido na escala do PRODUTO (câmera default = 10 unidades de mundo em
    /// 1080p ⇒ precisão 172,8) e com os 3 quadros que a C3 introduziu:
    ///
    /// | precisão | grade | 1 quadro | 3 quadros (C3) | × o orçamento |
    /// |---|---|---|---|---|
    /// | 86,4 | ~731 px | 26 ms | 72 ms | 2× / 5× |
    /// | **172,8** | ~1422 px | **104 ms** | **304 ms** | **7× / 19×** |
    /// | 345,6 | ~2804 px | 486 ms | 1449 ms | 30× / **90×** |
    ///
    /// ⚠️ **E nenhum cache resolve** — medido o split: `solve` **76%**, vetorização 18%,
    /// raster+setup **4%**. O que é constante entre tiques (o raster) é justamente a fatia
    /// desprezível; o que custa depende de `trap`/`squeeze`, que é o que o slider move. Ou
    /// seja: a medição diz *muda o invólucro, não o kernel* — exatamente o que o §7.2
    /// pré-decidiu.
    ///
    /// # O rate-limiter não tem constante
    ///
    /// **No máximo UM worker em voo, e o pedido mais recente é coalescido.** Enquanto ele
    /// roda, mexer o slider só reescreve `want`; quando termina, se `want` mudou, sai outro.
    /// Isso se auto-pace na taxa do próprio solve, sem `SETTLE` para calibrar — e é a
    /// diferença deliberada para o [`OffThread`] do áudio (ADR-0125), que precisa de debounce
    /// porque lá o trabalho é **automático e invisível**; aqui ele **é** o feedback visual, e
    /// esperar um timer seria a tela deixar de responder de propósito.
    ///
    /// **Undo continua UM passo por gesto:** durante o arrasto o `held_button` suprime, e
    /// depois dele o [`FlipColorize::live_busy`] assume o mesmo papel — um recálculo pendente
    /// **é** o gesto não ter terminado. Só a instalação que zera a fila registra o passo.
    pub(crate) fn flip_colorize_live_adjust(&mut self) {
        if self.flip_colorize.live.is_none() {
            return;
        }
        // Sair do modo Colorize encerra a adjustabilidade (o painel some, a base congelada
        // deixa de descrever o que está na tela).
        let Some(style) = self.flip_style.filter(|_| self.flip_wants_colorize()) else {
            self.flip_colorize.end_live();
            return;
        };
        let oid = self.flip_colorize.live.as_ref().expect("live").oid;
        let w2l = self.flip_active_world_to_local();
        let obj_scale = w2l.mean_scale() as f32;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let win = gfx.surface.size();
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;

        // ⚠️ **O guard é perguntado a TODOS os quadros ANTES de escrever em qualquer um** — o
        // ajuste é UMA operação, e re-rodar metade dela deixaria a tira com dois Traps. Se
        // um único quadro não é mais `base + as MINHAS regiões` (o artista desenhou, encheu
        // com o balde, apagou), a sessão MORRE inteira e o Trap novo simplesmente não
        // retro-aplica — o artista clica Apply de novo.
        let live = self.flip_colorize.live.as_ref().expect("live is Some");
        let intact = live.frames.iter().all(|f| {
            gfx.flip
                .object(oid)
                .and_then(|o| o.drawing(f.did))
                .is_some_and(|d| d.strokes.len() == f.base.len() + f.produced)
        });
        if !intact {
            self.flip_colorize.end_live(); // desenho editado ou sumido (undo/delete)
            return;
        }

        // ── 1. Colhe o resultado pronto, se houver. ──
        let live = self.flip_colorize.live.as_mut().expect("live is Some");
        if let Some(done) = live.job.as_mut().and_then(Job::try_take) {
            live.job = None;
            for (f, regions) in live.frames.iter_mut().zip(done.regions) {
                let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(f.did))
                else {
                    continue; // o `intact` acima já provou que existe; defensivo
                };
                drawing.strokes.clone_from(&f.base);
                f.produced = install_regions(drawing, &f.lines, &live.palette, regions);
            }
            live.trap = done.trap;
            live.bleed = done.bleed;
            // Sem isto o `post_frame_undo` pularia o diff num frame sem outro input, e o
            // ajuste ficaria fora do passo.
            self.any_input_this_frame = true;
            self.title_dirty = true;
        }

        // ── 2. O pedido mais recente ainda não foi honrado? Sai um worker (um só). ──
        let live = self.flip_colorize.live.as_mut().expect("live is Some");
        if live.job.is_some() || (style.trap == live.trap && style.colorize_bleed == live.bleed) {
            return;
        }
        let (precision, trap_px) = precision_and_trap(&style, px_to_world, obj_scale);
        let squeeze = ph2d_flip_colorize::squeeze_from_bleed(style.colorize_bleed as f32);
        // O worker recebe geometria CLONADA e **nunca vê o `FlipDoc`** — é o que torna a
        // porta segura, e o que obrigou as `lines` a serem congeladas no Apply.
        let lines: Vec<_> = live.frames.iter().map(|f| f.lines.clone()).collect();
        let seeds = live.seeds.clone();
        let (trap, bleed) = (style.trap, style.colorize_bleed);
        live.job = Some(Job::spawn("colorize", move |_| LiveResult {
            regions: lines
                .iter()
                .map(|l| colorize_regions(l, &seeds, precision, trap_px, squeeze))
                .collect(),
            trap,
            bleed,
        }));
    }
}
