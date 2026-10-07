//! **O VIDRO da aguada** (doc 48, BUGS #46) — o que o núcleo da aguada precisa para escrever, a cada
//! quadro, a transparência POR CANAL de cada texel ([`crate::compositor::vidro`]): o plano da camada
//! activa, o vidro da BASE (congelado com a base da tela: o núcleo repõe a base fora da mancha, e a
//! base guarda o seu próprio vidro) e o chão PRETO (as camadas de baixo sobre o preto; `None` quando
//! não há nenhuma — o preto puro). O núcleo avalia a óptica sobre o chão de sempre e sobre o preto; a
//! diferença, por canal, é o que o chão atravessa.

use super::*;
use crate::compositor::vidro::Vidro;
use std::sync::Weak;

/// A base congelada e o vidro dela.
type BaseCongelada = (Weak<Vec<u8>>, Arc<Vec<Vidro>>);
/// O plano do vidro aberto para escrita e o vidro congelado da base.
pub(super) type PlanoEBase = (Arc<Vec<Vidro>>, Arc<Vec<Vidro>>);
/// O chão (o `Weak` dele) e o chão preto que lhe corresponde (`None` = o preto puro).
type ChaoCongelado = (Weak<Vec<u8>>, Option<Arc<Vec<u8>>>);

/// O que se congela uma vez por base: ligado a ela por `Weak` (o endereço não se reutiliza enquanto o
/// seguramos), então uma base nova congela de novo e a mesma base de uma sessão molhada reaproveita.
#[derive(Default)]
pub(crate) struct VidroCongelado {
    base: Option<BaseCongelada>,
    chao_preto: Option<ChaoCongelado>,
}

/// O vidro de um quadro do núcleo. O plano sai do mapa e volta com [`PainterTool::devolve_o_vidro`].
pub(super) struct VidroDoQuadro {
    pub camada: RtLayerId,
    pub plano: Arc<Vec<Vidro>>,
    pub base: Arc<Vec<Vidro>>,
    pub chao_preto: Option<Arc<Vec<u8>>>,
}

impl PainterTool {
    /// O vidro deste quadro, ou `None` (a pintar uma máscara, ou sem tela).
    pub(super) fn vidro_do_quadro(
        &mut self,
        base: &Arc<Vec<u8>>,
        backdrop: &Arc<Vec<u8>>,
    ) -> Option<VidroDoQuadro> {
        #[cfg(test)]
        if self.sem_vidro {
            return None;
        }
        let camada = self
            .layers
            .active()
            .filter(|&id| !self.layers.is_mask(id))?;
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        let (plano, vbase) =
            abre_o_vidro(&mut self.vidros, &mut self.vidro_congelado, camada, n, base)?;
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
        let tecto = (self.paint.stroke_mask.len() == n).then_some(&self.paint.stroke_mask);
        let cor = self.paint.brush.color;
        let antes = self.paint.stroke_undo.as_ref()?;
        let (vantes, pantes) = (antes.vidros.get(&camada)?, &antes.canvas_rgba);
        if vantes.len() != n || pantes.len() != n * 4 {
            return None;
        }
        let (vantes, pantes) = (Arc::clone(vantes), Arc::clone(pantes));
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
        let plano = self.vidros.get_mut(&camada)?;
        if Arc::strong_count(plano) > 1 {
            *plano = Arc::new(crate::plane_copy::par_clone(plano));
        }
        let plano = Arc::get_mut(plano).expect("o plano é só nosso");
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
                    let a = crate::compositor::vidro::alfas(o, &vantes[i]);
                    *slot = crate::compositor::vidro::sela(
                        [px[0], px[1], px[2], px[3]],
                        crate::compositor::vidro::cobre(a, s),
                    );
                    c = Some(c.map_or((x, x), |(a, b)| (a.min(x), b.max(x))));
                }
                c.map(|(a, b)| (a, b, y))
            })
            .filter_map(|c| c)
            .reduce_with(|p, q| (p.0.min(q.0), p.1.max(q.1), p.2.max(q.2)));
        let lo = (y0..y1).find(|&y| {
            (x0..x1).any(|x| {
                pantes[(y * w + x) * 4..(y * w + x) * 4 + 4]
                    != tela[(y * w + x) * 4..(y * w + x) * 4 + 4]
            })
        })?;
        caixa.map(|(a, b, hi)| Region {
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
    congelado: &mut VidroCongelado,
    camada: RtLayerId,
    n: usize,
    base: &Arc<Vec<u8>>,
) -> Option<PlanoEBase> {
    if n == 0 || base.len() != n * 4 {
        return None;
    }
    let mut plano = vidros
        .remove(&camada)
        .filter(|p| p.len() == n)
        .unwrap_or_else(|| Arc::new(vec![[0u8; 7]; n]));
    let vbase = match &congelado.base {
        Some((w, v)) if Weak::ptr_eq(w, &Arc::downgrade(base)) && v.len() == n => Arc::clone(v),
        _ => {
            congelado.base = Some((Arc::downgrade(base), Arc::clone(&plano)));
            Arc::clone(&plano)
        }
    };
    // O plano é escrito no lugar: quem o partilha (o desfazer, a base congelada) fica com a cópia.
    if Arc::strong_count(&plano) > 1 {
        plano = Arc::new(crate::plane_copy::par_clone(&plano));
    }
    Some((plano, vbase))
}
