//! **O VIDRO da aguada** (doc 48, BUGS #46) — o que o núcleo da aguada precisa para escrever, a cada
//! quadro, a transparência POR CANAL de cada texel ([`crate::compositor::vidro`]): o plano da camada
//! activa, o vidro da BASE (congelado com a base da tela: o núcleo repõe a base fora da mancha, e a
//! base guarda o seu próprio vidro) e o chão PRETO (as camadas de baixo sobre o preto; `None` quando
//! não há nenhuma — o preto puro). O núcleo avalia a óptica sobre o chão de sempre e sobre o preto; a
//! diferença, por canal, é o que o chão atravessa.

use super::*;
use crate::compositor::vidro::Vidro;
use std::sync::Weak;

/// A base congelada (e a camada) e o vidro dela, pelo journal da sessão (ADR-0204 D3).
pub(super) type BaseCongelada = (Weak<Vec<u8>>, RtLayerId, SessaoDoVidro);
/// O plano do vidro aberto para escrita e o vidro da base na janela do quadro.
pub(super) type PlanoEBase = (Arc<Vec<Vidro>>, JanelaDoVidro);

/// **O vidro do início da sessão molhada** (ADR-0204 D3): os tiles que a sessão escreveu, tomados na 1.ª
/// escrita de cada um; o resto é o vivo (§5.28). Atravessa traços: não zera com o passo.
#[derive(Clone, Default)]
pub(super) struct SessaoDoVidro {
    tiles: crate::undo::journal::TileJournal<Vidro>,
    /// O plano inteiro na congelação, só com [`GUARDA_O_INICIO_DA_SESSAO`] (o oráculo dos portões).
    #[cfg(test)]
    pub(super) inicio: Option<Arc<Vec<Vidro>>>,
}

impl SessaoDoVidro {
    /// Toma os tiles de `caixa` (`x0, x1, y0, y1`, em texels) de `vivo` ANTES de alguém os escrever.
    pub(super) fn captura(&mut self, vivo: &[Vidro], fw: usize, (x0, x1, y0, y1): Caixa) {
        self.tiles.capture(vivo, fw, Some((x0, y0, x1, y1)));
    }

    /// O vidro do início da sessão em `caixa`: o journal da sessão com o vivo por baixo.
    fn janela(&self, vivo: &[Vidro], fw: usize, (x0, x1, y0, y1): Caixa) -> JanelaDoVidro {
        let w = x1 - x0;
        let mut dados = Vec::with_capacity(w * (y1 - y0));
        for y in y0..y1 {
            if self.tiles.is_empty() {
                dados.extend_from_slice(&vivo[y * fw + x0..y * fw + x1]);
            } else {
                self.tiles.read_row_into(y, x0, w, vivo, &mut dados);
            }
        }
        JanelaDoVidro {
            dados,
            x0,
            y0,
            w,
            fw,
        }
    }

    /// Os bytes que a sessão retém (os tiles tomados), para o tecto e o instantâneo (D5).
    pub(super) fn bytes(&self) -> usize {
        self.tiles.heap_bytes()
    }
}

/// `(x0, x1, y0, y1)` meio-aberta, em texels.
pub(super) type Caixa = (usize, usize, usize, usize);

/// **O `vbase` de um quadro**: o vidro do início da sessão, denso, só na caixa que o quadro escreve. Indexa-se
/// pelo índice GLOBAL do texel (`gy * fw + gx`), como o plano; fora da caixa, pânico.
#[derive(Clone, Debug)]
pub(crate) struct JanelaDoVidro {
    pub(super) dados: Vec<Vidro>,
    pub(super) x0: usize,
    pub(super) y0: usize,
    pub(super) w: usize,
    fw: usize,
}

impl std::ops::Index<usize> for JanelaDoVidro {
    type Output = Vidro;
    #[inline]
    fn index(&self, i: usize) -> &Vidro {
        let (y, x) = (i / self.fw, i % self.fw);
        debug_assert!(
            x >= self.x0 && x < self.x0 + self.w && y >= self.y0,
            "fora da janela do vbase"
        );
        &self.dados[(y - self.y0) * self.w + (x - self.x0)]
    }
}
/// O chão (o `Weak` dele) e o chão preto que lhe corresponde (`None` = o preto puro).
type ChaoCongelado = (Weak<Vec<u8>>, Option<Arc<Vec<u8>>>);

/// O que se congela uma vez por base: ligado a ela por `Weak` (o endereço não se reutiliza enquanto o
/// seguramos), então uma base nova congela de novo e a mesma base de uma sessão molhada reaproveita.
#[derive(Clone, Default)]
pub(crate) struct VidroCongelado {
    /// A base da aguada e a do Wet Paint, cada uma no seu lugar: os dois meios podem ter sessões vivas
    /// ao mesmo tempo, e um lugar só voltava a congelar a partir de um plano já escrito.
    pub(super) aguada: Option<BaseCongelada>,
    pub(super) molhada: Option<BaseCongelada>,
    chao_preto: Option<ChaoCongelado>,
}

thread_local! {
    /// `false` = o caminho de antes do B18.4 (WB): o borrão deixa o `Weak` do congelado morrer (o oráculo A×B,
    /// por fio: o borrão corre no fio do traço).
    #[doc(hidden)]
    pub static VIDRO_SEGUE_A_BASE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

#[cfg(test)]
thread_local! {
    /// As cópias do plano do vidro feitas pelo [`abre_o_vidro`] neste fio (a régua dos portões).
    pub(super) static COPIAS_DO_VIDRO: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// As congelações da sessão do vidro neste fio (uma por sessão; o caminho sem B18.4 recongela a cada quadro).
    pub(super) static CONGELACOES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Liga o oráculo do D3: a congelação guarda o plano inteiro e cada [`abre_o_vidro`] grava a janela que deu.
    pub(super) static GUARDA_O_INICIO_DA_SESSAO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// As janelas do `vbase` que [`abre_o_vidro`] deu, com o início da sessão de cada uma.
    pub(super) static JANELAS_DO_VBASE: std::cell::RefCell<Vec<(JanelaDoVidro, Arc<Vec<Vidro>>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl VidroCongelado {
    /// **O que as duas sessões do vidro retêm** (ADR-0204 D5): os tiles tomados desde a congelação.
    pub(crate) fn bytes(&self) -> usize {
        [&self.aguada, &self.molhada]
            .into_iter()
            .flatten()
            .map(|c| c.2.bytes())
            .sum()
    }

    /// Larga a sessão de cada meio cuja camada não está em `vivas` (`None` = todas): o restauro troca o plano
    /// inteiro e o id de uma camada apagada reaproveita-se.
    pub(crate) fn larga_as_sessoes(
        &mut self,
        vivas: Option<&std::collections::BTreeSet<RtLayerId>>,
    ) {
        for s in [&mut self.aguada, &mut self.molhada] {
            if s.as_ref()
                .is_some_and(|c| vivas.is_none_or(|v| !v.contains(&c.1)))
            {
                *s = None;
            }
        }
    }

    /// **A base mudou de `Arc`, não de dados** (B18.4, Watercolor Basic a 4K): o borrão escreve a base no lugar e,
    /// com o `Weak` daqui vivo, o `make_mut` MOVE-a para um `Arc` novo — o `Weak` morria, o vidro recongelava a
    /// partir do plano JÁ escrito e o copiava inteiro em todo evento (58 MB/quadro a 3840×2160). Religa só o
    /// `Weak` que apontava exactamente o `Arc` movido (`antes`, vivo um instante antes): sem endereço reaproveitado.
    pub(super) fn a_base_mudou_de_arc(&mut self, antes: *const Vec<u8>, agora: &Arc<Vec<u8>>) {
        if std::ptr::eq(antes, Arc::as_ptr(agora)) || !VIDRO_SEGUE_A_BASE.get() {
            return;
        }
        for c in [&mut self.aguada, &mut self.molhada].into_iter().flatten() {
            if std::ptr::eq(c.0.as_ptr(), antes) && c.0.strong_count() == 0 {
                c.0 = Arc::downgrade(agora);
            }
        }
    }
}

/// O vidro de um quadro do núcleo. O plano sai do mapa e volta com [`PainterTool::devolve_o_vidro`].
pub(super) struct VidroDoQuadro {
    pub camada: RtLayerId,
    pub plano: Arc<Vec<Vidro>>,
    pub base: JanelaDoVidro,
    pub chao_preto: Option<Arc<Vec<u8>>>,
}

thread_local! {
    /// `false` = o caminho de antes do n.º 7 item 3e: o fundo de baixo e o chão preto compõem-se no 1.º pen-down da
    /// camada que passou a activa (10–12 ms a 4K) em vez de na troca. Só a sonda e o gate o desligam.
    #[doc(hidden)]
    pub static FUNDO_AQUECIDO_NA_TROCA: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

impl PainterTool {
    /// **O fundo do próximo toque compõe-se na TROCA de camada** (n.º 7 item 3e): com a aguada armada e nenhum traço
    /// aberto, o fundo de baixo ([`Self::fundo_de_baixo`], na cache dele) e o chão preto do vidro (chaveado pelo `Arc`
    /// desse fundo) ficam prontos, e o pen-down acerta as duas caches. As mesmas funções com as mesmas chaves: o
    /// mesmo bit; o que muda entre a troca e o toque só as invalida, e o toque compõe como antes.
    pub(in crate::tool) fn aquece_o_fundo_do_toque(&mut self) {
        if !FUNDO_AQUECIDO_NA_TROCA.get()
            || self.paint.stroke.is_some()
            || !self.paint.brush.watercolor
            || !matches!(self.paint.paint_mode, PaintMode::Paint)
            || self.paint.eraser
            || self.source_size.0 == 0
            || self.source_size.1 == 0
        {
            return;
        }
        let Some(camada) = self.layers.active().filter(|&id| !self.layers.is_mask(id)) else {
            return;
        };
        if self.aquece_em_fundo(camada) {
            return;
        }
        let fundo = self.fundo_de_baixo();
        let fresca = matches!(&self.vidro_congelado.chao_preto,
            Some((w, _)) if Weak::ptr_eq(w, &Arc::downgrade(&fundo)));
        if !fresca {
            let abaixo = self.layers.z_order_bottom_up().first() != Some(&camada);
            let c = abaixo.then(|| Arc::new(self.build_wet_backdrop_sobre([0, 0, 0])));
            self.vidro_congelado.chao_preto = Some((Arc::downgrade(&fundo), c));
        }
    }

    /// Guarda `chao` como o chão preto de `fundo` (o aquecimento em fundo, colhido no pen-down).
    pub(super) fn guarda_o_chao_preto(&mut self, fundo: &Arc<Vec<u8>>, chao: Option<Arc<Vec<u8>>>) {
        self.vidro_congelado.chao_preto = Some((Arc::downgrade(fundo), chao));
    }

    /// O chão preto congelado é o de `fundo`.
    pub(super) fn chao_preto_fresco(&self, fundo: &Arc<Vec<u8>>) -> bool {
        matches!(&self.vidro_congelado.chao_preto,
            Some((w, _)) if Weak::ptr_eq(w, &Arc::downgrade(fundo)))
    }

    /// Os bytes do chão preto congelado (`[2]` = o preto puro, `[3]` = nenhum), para o gate.
    #[cfg(test)]
    pub(super) fn chao_preto_congelado(&self) -> Vec<u8> {
        match &self.vidro_congelado.chao_preto {
            Some((_, Some(c))) => c.to_vec(),
            Some((_, None)) => vec![2],
            None => vec![3],
        }
    }

    /// O chão preto congelado é o de `fundo` (o portão do aquecimento).
    #[cfg(test)]
    pub(super) fn chao_preto_e_de(&self, fundo: &Arc<Vec<u8>>) -> bool {
        matches!(&self.vidro_congelado.chao_preto,
            Some((w, _)) if Weak::ptr_eq(w, &Arc::downgrade(fundo)))
    }

    /// O vidro deste quadro, ou `None` (a pintar uma máscara, ou sem tela).
    pub(super) fn vidro_do_quadro(
        &mut self,
        base: &Arc<Vec<u8>>,
        backdrop: &Arc<Vec<u8>>,
        area: Option<Region>,
    ) -> Option<VidroDoQuadro> {
        #[cfg(test)]
        if self.sem_vidro {
            return None;
        }
        let camada = self
            .layers
            .active()
            .filter(|&id| !self.layers.is_mask(id))?;
        let (plano, vbase) = abre_o_vidro(
            &mut self.vidros,
            &mut self.vidro_congelado.aguada,
            &mut self.vidro_congelado.molhada,
            camada,
            self.source_size,
            base,
            &self.undo.write_state,
            area,
        )?;
        let fresca = |w: &Weak<Vec<u8>>, a: &Arc<Vec<u8>>| Weak::ptr_eq(w, &Arc::downgrade(a));
        let chao_preto = match &self.vidro_congelado.chao_preto {
            Some((w, c)) if fresca(w, backdrop) => c.clone(),
            _ => {
                let abaixo = self.layers.z_order_bottom_up().first() != Some(&camada);
                let c = abaixo.then(|| Arc::new(self.build_wet_backdrop_sobre([0, 0, 0])));
                self.vidro_congelado.chao_preto = Some((Arc::downgrade(backdrop), c.clone()));
                c
            }
        };
        Some(VidroDoQuadro {
            camada,
            plano,
            base: vbase,
            chao_preto,
        })
    }

    /// **O traço de outro meio CONTINUA o vidro** (Digital, Impasto, o Solid): cada canal atenua-se
    /// por `1 − s` a partir do vidro de ANTES do traço (o que o desfazer segura), com `s` a cobertura do
    /// traço no texel — o tecto `stroke_mask` quando armado (o texel é `antes·(1 − m) + cor·m`), senão
    /// a do `over` resolvida do píxel de antes, do de agora e da cor do pincel ([`cobertura_do_over`]; o
    /// de fábrica, Strength 1, não arma o tecto). O que o traço APAGOU (o alfa desceu) fica para a
    /// leitura ([`crate::compositor::vidro::alfas`]). Na `regiao`, ou na tela inteira (o pen-up);
    /// devolve a caixa do que mudou. Sem vidro antes do traço, ou numa aguada: nada.
    pub(crate) fn continua_o_vidro_do_traco(&mut self, regiao: Option<Region>) -> Option<Region> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = w * h;
        let camada = self.layers.active()?;
        // A aguada e o Wet Paint escrevem o vidro no próprio composite: aqui só os outros meios.
        if self.watercolor_render_active()
            || self.paint_media() == super::media::PaintMedia::WetPaint
            || self.canvas_rgba.len() != n * 4
        {
            return None;
        }
        // O vidro de antes do traço: o que o `before` segura (o oráculo) ou, descrito (ADR-0204 D2), o journal
        // sobre o vivo, materializado na janela que este passe escreve ANTES do `fork_vidro`.
        let antes = self.paint.stroke_undo.as_ref()?;
        let segurado = antes.vidros.get(&camada).map(Arc::clone);
        if segurado.is_none() && !antes.relief_elided.vidros.contains_key(&camada) {
            return None;
        }
        let pantes = self.tela_de_antes_do_traco(true)?;
        if pantes.len() != n * 4 {
            return None;
        }
        let tecto = (self.paint.stroke_mask.len() == n).then_some(&self.paint.stroke_mask);
        let cor = self.paint.brush.color;
        // Sem região pedida, a janela que o traço declarou ao desfazer; sem ela, a tela inteira.
        let r = regiao
            .or_else(|| self.undo.write_state.get().declarada())
            .unwrap_or(Region {
                x: 0,
                y: 0,
                w: w as u32,
                h: h as u32,
            });
        let (x0, y0) = (r.x as usize, r.y as usize);
        let (x1, y1) = ((x0 + r.w as usize).min(w), (y0 + r.h as usize).min(h));
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        let vivo = self.vidros.get(&camada)?;
        if vivo.len() != n {
            return None;
        }
        // `vantes(i)`: o antes do texel `i` da janela.
        let (vantes, vw): (Arc<Vec<_>>, usize) = match segurado {
            Some(v) if v.len() == n => (v, 0),
            Some(_) => return None,
            None => (
                Arc::new(
                    self.undo
                        .write_state
                        .vidro_antes_na_janela(vivo, w, (x0, x1, y0, y1)),
                ),
                x1 - x0,
            ),
        };
        let vi = move |i: usize| {
            if vw == 0 {
                i
            } else {
                (i / w - y0) * vw + (i % w - x0)
            }
        };
        // A sessão molhada desta camada vê o vidro de antes desta escrita (D3).
        for s in [
            &mut self.vidro_congelado.aguada,
            &mut self.vidro_congelado.molhada,
        ]
        .into_iter()
        .flatten()
        .filter(|s| s.1 == camada)
        {
            s.2.captura(vivo, w, (x0, x1, y0, y1));
        }
        let plano = super::plane_fork::fork_vidro(
            self.vidros.get_mut(&camada)?,
            &self.undo.write_state,
            camada,
            self.source_size,
            Some(r),
        );
        let tela = &self.canvas_rgba;
        let tecto = tecto.map(|t| t.as_slice());
        use rayon::prelude::*;
        let caixa = plano[y0 * w..y1 * w]
            .par_chunks_mut(w)
            .enumerate()
            .map(|(k, linha)| {
                let y = y0 + k;
                let mut c: Option<(usize, usize)> = None;
                for (x, slot) in linha.iter_mut().enumerate().take(x1).skip(x0) {
                    let i = y * w + x;
                    let (o, px) = (&pantes[i * 4..i * 4 + 4], &tela[i * 4..i * 4 + 4]);
                    if o == px || px[3] < o[3] {
                        continue; // intocado, ou apagado (a leitura escala os alfas)
                    }
                    let s = match tecto.map(|t| t[i]) {
                        Some(m) if m > 0 => f32::from(m) / 255.0,
                        _ => cobertura_do_over(o, px, cor),
                    };
                    let a = crate::compositor::vidro::alfas(o, &vantes[vi(i)]);
                    *slot = crate::compositor::vidro::sela(
                        [px[0], px[1], px[2], px[3]],
                        crate::compositor::vidro::cobre(a, s),
                    );
                    c = Some(c.map_or((x, x), |(a, b)| (a.min(x), b.max(x))));
                }
                c.map(|(a, b)| (a, b, y, y))
            })
            .filter_map(|c| c)
            .reduce_with(|p, q| (p.0.min(q.0), p.1.max(q.1), p.2.min(q.2), p.3.max(q.3)));
        caixa.map(|(a, b, lo, hi)| Region {
            x: a as u32,
            y: lo as u32,
            w: (b - a + 1) as u32,
            h: (hi - lo + 1) as u32,
        })
    }

    /// Antes de um quadro sobre o papel de cor, com um traço aberto: o vidro segue o que o traço já
    /// cobriu na região suja ([`Self::continua_o_vidro_do_traco`]).
    pub(crate) fn segue_o_vidro_no_quadro(&mut self) {
        if self.paint.stroke_undo.is_some() && self.papel_atravessa_vidro() {
            let _ = self.continua_o_vidro_do_traco(self.dirty_rect);
        }
    }

    /// Devolve o plano escrito ao mapa.
    pub(super) fn devolve_o_vidro(&mut self, camada: RtLayerId, plano: Arc<Vec<Vidro>>) {
        self.vidros.insert(camada, plano);
    }
}

/// **Os alfas por canal de um texel da aguada**, da óptica sobre os dois chãos: `branco`/`preto` são as
/// aparências (bytes) sobre o chão de sempre e sobre o preto, `chao`/`chao_preto` os dois chãos. Onde
/// os dois chãos coincidem (uma camada opaca por baixo) o chão não se vê, e o texel fica com o alfa
/// único `a`.
#[inline]
pub(super) fn alfas_do_vidro(
    branco: [u8; 3],
    preto: [u8; 3],
    chao: [u8; 3],
    chao_preto: [u8; 3],
    a: u8,
) -> [u8; 3] {
    core::array::from_fn(|c| {
        let d = f32::from(chao[c]) - f32::from(chao_preto[c]);
        if d < 1.0 {
            return a;
        }
        let t = ((f32::from(branco[c]) - f32::from(preto[c])) / d).clamp(0.0, 1.0);
        ((1.0 - t) * 255.0).round() as u8
    })
}

/// **A cobertura `s` de um `over` com a cor `cor`** que levou o píxel `o` a `px` (RGBA): em
/// pré-multiplicado o `over` é `px = cor·s + o·(1 − s)` nos quatro canais (a cor com alfa 1), e `s` é a
/// projecção por mínimos quadrados — exacta para um traço de cor única; com o alfa de `o` < 1 ela
/// também o lê, então só uma tinta da MESMA cor sobre uma camada opaca a deixa sem resposta (`0`).
fn cobertura_do_over(o: &[u8], px: &[u8], cor: [f32; 3]) -> f32 {
    let pm = |p: &[u8]| {
        let a = f32::from(p[3]) / 255.0;
        [
            f32::from(p[0]) / 255.0 * a,
            f32::from(p[1]) / 255.0 * a,
            f32::from(p[2]) / 255.0 * a,
            a,
        ]
    };
    let (po, pn) = (pm(o), pm(px));
    let d = [cor[0], cor[1], cor[2], 1.0];
    let (mut num, mut den) = (0.0f32, 0.0f32);
    for c in 0..4 {
        let v = d[c] - po[c];
        num += (pn[c] - po[c]) * v;
        den += v * v;
    }
    if den <= 1e-6 {
        0.0
    } else {
        (num / den).clamp(0.0, 1.0)
    }
}

/// **Abre o vidro da `camada` para o escrever**: o plano sai do mapa (volta com
/// [`PainterTool::devolve_o_vidro`]) e o vidro da `base` congela-se com ela (ligado por `Weak`). Por
/// campos e não por `&mut self`: o composite do Wet Paint chama-o com a sessão emprestada.
pub(super) fn abre_o_vidro(
    vidros: &mut crate::compositor::vidro::Vidros,
    congelado: &mut Option<BaseCongelada>,
    outra: &mut Option<BaseCongelada>,
    camada: RtLayerId,
    size: (u32, u32),
    base: &Arc<Vec<u8>>,
    win: &crate::undo::window::WriteState,
    area: Option<Region>,
) -> Option<PlanoEBase> {
    let n = (size.0 as usize) * (size.1 as usize);
    if n == 0 || base.len() != n * 4 {
        return None;
    }
    let guardado = vidros.remove(&camada).filter(|p| p.len() == n);
    let nasce = guardado.is_none();
    let mut plano = guardado.unwrap_or_else(|| Arc::new(vec![[0u8; 7]; n]));
    let fresca = matches!(congelado, Some((w, c, _)) if *c == camada && Weak::ptr_eq(w, &Arc::downgrade(base)));
    if !fresca {
        let sessao = SessaoDoVidro {
            #[cfg(test)]
            inicio: GUARDA_O_INICIO_DA_SESSAO
                .get()
                .then(|| Arc::new(plano.as_ref().clone())),
            ..Default::default()
        };
        #[cfg(test)]
        CONGELACOES.with(|c| c.set(c.get() + 1));
        *congelado = Some((Arc::downgrade(base), camada, sessao));
    }
    let sessao = &mut congelado.as_mut()?.2;
    let (fw, fh) = (size.0 as usize, size.1 as usize);
    let caixa = area.map_or((0, fw, 0, fh), |r| {
        let (x0, y0) = ((r.x as usize).min(fw), (r.y as usize).min(fh));
        (
            x0,
            (x0 + r.w as usize).min(fw),
            y0,
            (y0 + r.h as usize).min(fh),
        )
    });
    // Antes da escrita deste quadro: a sessão toma os tiles que ele vai escrever, e a janela lê-os. A sessão
    // do OUTRO meio na mesma camada também (as duas podem estar vivas: trocar de meio não seca a aguada).
    sessao.captura(&plano, fw, caixa);
    if let Some(o) = outra.as_mut().filter(|o| o.1 == camada) {
        o.2.captura(&plano, fw, caixa);
    }
    let vbase = sessao.janela(&plano, fw, caixa);
    #[cfg(test)]
    if let Some(i) = &sessao.inicio {
        JANELAS_DO_VBASE.with(|j| j.borrow_mut().push((vbase.clone(), Arc::clone(i))));
    }
    // O plano é escrito no lugar: quem o partilha (o desfazer, a base congelada) fica com a cópia. O que
    // nasceu AGORA é zeros: a cópia é outro plano zerado, cujas páginas só existem onde o traço escreve
    // (B18.4 WB: a 4K, 58 MB copiados no 1.º toque de cada camada nova).
    if nasce {
        win.note_absent_vidro(camada);
        if Arc::strong_count(&plano) > 1 {
            plano = Arc::new(vec![[0u8; 7]; n]);
        }
    } else {
        #[cfg(test)]
        if Arc::strong_count(&plano) > 1 {
            COPIAS_DO_VIDRO.with(|c| c.set(c.get() + 1));
        }
        super::plane_fork::fork_vidro(&mut plano, win, camada, size, area);
    }
    Some((plano, vbase))
}
