//! ⭐⭐ **O RELEVO de cada camada do Composite Brush** — a metade que a pilha perdia (fila 44,
//! item 8).
//!
//! # O defeito, medido
//!
//! A pilha que ACUMULA ([`super::composite_acumulado`]) deposita cada camada num PLANO de medição, e
//! força ali `Draw To = Color`: um plano é RGBA, e o relevo é da TELA. A frase estava certa e a
//! consequência não tinha dono — **nenhuma camada chegava ao depósito de altura**. Sonda
//! `diag_o_relevo_da_pilha.rs` (ao lado), Impasto, o mesmo traço:
//!
//! | pilha | corpo depositado | com `Draw To = Depth`, tinta |
//! |---|---|---|
//! | nenhuma | `974,24` | `0` |
//! | 1 Brush (a rota de sempre) | `974,24` | `0` |
//! | **2 Brush** | **`0,00`** | **`393 834`** |
//!
//! ⇒ com duas camadas ou mais **o Impasto pintava chapado**, e um pincel de só-relevo pintava COR. E a
//! camada `Erase` não mordia o relevo que a borracha avulsa morde (`0,840 → 0,633` no centro de um
//! dab a força `0,5`). O mesmo vale para o FILME do papel no Digital, que passa pela mesma porta.
//!
//! # A lei
//!
//! **O relevo é da TELA e a cor é do PLANO, e cada um tem uma porta.** Os planos ficam com o
//! `Draw To` do ARTISTA — é ele que corta o pigmento de um pincel que deposita corpo num FILME, e
//! forçá-lo a `Color` pintava a camada com a tinta cheia do Digital (pior `178` contra o avulso), e
//! com `Depth` punha cor onde não devia haver nenhuma. O que impede o corpo de ser depositado dentro
//! de um plano é o `acumulando_no_plano`, lido pelo despacho ANTES do depósito de altura.
//!
//! O relevo corre à parte: cada camada viva chama o **depósito de altura de sempre**
//! ([`PainterTool::stamp_dabs_height`]) sobre a lista dela, com a força e a dureza dela —
//! directamente, e **não** pelas rotas de cor (o ramo «só relevo» do despacho cala a cor no
//! Impasto, mas no Digital com filme de papel ele a pintaria).
//!
//! **Resultado, medido:** cor e relevo da pilha IDÊNTICOS AO BIT aos do pincel avulso, no Digital e
//! no Impasto, para o Brush e para a borracha (gates em `composite_relevo_tests.rs`).
//!
//! * **Brush** deposita o corpo no envelope do traço, que só assenta na camada ao soltar. O envelope
//!   é um MÁXIMO (uma passagem deixa uma espessura), logo a ordem entre camadas não o muda.
//! * **Erase de escopo `Tudo`** morde o relevo ASSENTE, pela lei da borracha avulsa.
//! * **Erase de escopo `Traco`** não o toca: ela devolve o `pre`, e o `pre` do relevo assente é ele
//!   próprio.
//! * **Blur** não mexe no relevo, como o Blur avulso (medido: `974,24` antes e depois).
//! * **Smear** já o arrasta pela sessão de deformação, como o avulso (medido: `929,24` nos dois).
//!
//! # ⭐ A borracha POR CIMA de um Brush, no mesmo traço (fila 44, 8b)
//!
//! A Erase de cima apagava a COR que o Brush de baixo pôs neste traço e **não** o CORPO dele: o corpo
//! vive no envelope do traço, que é UM plano partilhado e só assenta ao soltar. Medido (sonda
//! `diag_o_corpo_fantasma`, borracha dura e maior que o Brush, escopo `Traco`): tinta **`0`** e
//! relevo **`974,24`** — exactamente o de um Brush sozinho. *Um corpo sem tinta nenhuma, que a luz
//! sombreia.*
//!
//! ⇒ quando há uma Erase viva por cima de um Brush vivo, **cada Brush guarda o SEU envelope**
//! ([`PlanosDoCorpo`], trocado à volta do depósito como a cadeia) e o envelope do traço é
//! RECOMPOSTO por evento, de baixo para cima, pela mesma lei dos planos de cor:
//!
//! ```text
//!     tinta ← max(tinta, tinta_b)          uma camada Brush (o envelope é um máximo)
//!     tinta ← tinta · (α_e / 255)          uma camada Erase (o que o escudo dela deixou)
//! ```
//!
//! ⭐ **A borracha tira TINTA, e o corpo DERIVA-SE do que sobra** — nunca `altura × k`. O corpo que
//! assenta ao soltar é re-derivado da TINTA do traço ([`PainterTool::commit_stroke_height`]: *«os
//! ingredientes são a verdade»*), logo multiplicar só a altura seria desfeito no commit e ressuscitado
//! por cada toque no `Depth`. O filme (a cobertura que a luz pesa) é multiplicado pelo mesmo `k`,
//! como a borracha avulsa multiplica a `cover`.
//!
//! ⚠️ **DIVERGÊNCIA DECLARADA contra «Brush, e depois uma borracha avulsa por cima»:** a avulsa
//! morde a ALTURA assente (`h ← h·(1−c)`), e aqui o corpo é `derive(tinta·k)`. As duas leis
//! coincidem onde a borracha não toca (`k = 1`) e onde apaga tudo (`k = 0`), e divergem na orla
//! parcial: pela curva do `Body`, meia tinta ainda pode estar no planalto. É a lei que sobrevive a um
//! ajuste do `Depth` depois do traço, e é por isso que é esta.
//!
//! ⚠️ **Só corre quando é preciso** ([`PainterTool::corpo_por_camada`]): sem uma Erase por cima de um
//! Brush nada muda de caminho, e o traço é **byte-idêntico** ao de antes (o envelope partilhado já é
//! o máximo das camadas). O deslocamento do `Push` continua partilhado: a borracha não o desfaz.
//!
//! # E o TILING, que a mesma régua apanhou
//!
//! A região da pilha era medida com os dabs SEM embrulhar ([`super::region::caixa_das_camadas`]):
//! um lote com o cursor já para lá da borda dava `None` e a pilha saía cedo, e as cópias do outro
//! lado da costura nunca chegavam à tela — `0` px de cor contra `738` do avulso, no Digital
//! também. E o depósito da pilha não publicava os GRUPOS do Tiling, logo cada cópia embrulhada
//! tirava a sua própria moldura aleatória (régua: o traço deslocado de uma largura inteira tem de
//! pintar a mesma imagem, porque a tela é um toro).
//!
//! # O estado por traço é POR CAMADA
//!
//! O depósito de altura lembra onde o lote anterior acabou (`last_height_center`, para o corpo de um
//! dab se ligar ao anterior em vez de ficar uma conta de rosário) e a onda do Push. Partilhados, a
//! 1.ª camada de um lote ligar-se-ia à ÚLTIMA camada do lote anterior — outro tamanho, outra força —,
//! logo cada camada guarda os seus e troca-os à volta da passagem, como o `rng_camada` e a máscara.

use super::Region;
use super::composite::{CompositeOp, EscopoDaBorracha, N_CAMADAS};
use super::relief_state::WaveTip;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

/// O estado do depósito de altura que é de UMA camada ao longo do traço.
#[derive(Default)]
pub(super) struct RelevoDaCamada {
    cadeia: Vec<Option<([f32; 2], f32)>>,
    onda: Vec<(f32, Option<WaveTip>)>,
    /// O envelope PRÓPRIO desta camada — vazio fora de [`PainterTool::corpo_por_camada`].
    planos: PlanosDoCorpo,
}

/// ⭐ **O envelope de UMA camada Brush** — os mesmos cinco planos do envelope do traço
/// ([`super::relief_state::ReliefState`]), trocados com eles à volta do depósito dela.
#[derive(Default)]
pub(super) struct PlanosDoCorpo {
    height: Vec<f32>,
    paint: Vec<f32>,
    grain: Vec<u8>,
    film: Vec<u8>,
    radius: Vec<f32>,
}

impl PlanosDoCorpo {
    /// Trocar com o envelope do traço — a mesma porta nos dois sentidos.
    fn troca(&mut self, r: &mut super::relief_state::ReliefState) {
        std::mem::swap(&mut self.height, &mut r.stroke_height);
        std::mem::swap(&mut self.paint, &mut r.stroke_paint);
        std::mem::swap(&mut self.grain, &mut r.stroke_grain);
        std::mem::swap(&mut self.film, &mut r.stroke_film);
        std::mem::swap(&mut self.radius, &mut r.stroke_radius);
    }
}

/// Que relevo a camada escreve — a porta ÚNICA da pergunta.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum RelevoDaOp {
    /// Deposita o corpo no envelope do traço.
    Corpo,
    /// Morde o relevo assente (a borracha avulsa).
    Morde,
    /// Não o toca.
    Nada,
}

impl CompositeOp {
    /// O relevo que esta operação escreve, dado o escopo da borracha.
    pub(super) fn relevo(self, escopo: EscopoDaBorracha) -> RelevoDaOp {
        match self {
            Self::Brush => RelevoDaOp::Corpo,
            Self::Erase if matches!(escopo, EscopoDaBorracha::Tudo) => RelevoDaOp::Morde,
            Self::Erase | Self::Blur | Self::Smear => RelevoDaOp::Nada,
        }
    }
}

impl PainterTool {
    /// **O relevo da camada `pos`** sobre os dabs novos dela. Corre ANTES da acumulação da cor: o
    /// depósito de altura lê uma CÓPIA do fluxo aleatório, e os dois têm de partir do mesmo ponto.
    pub(super) fn relevo_da_camada(&mut self, pos: usize, dabs: &[Dab]) {
        let layer = self.paint.composite[pos];
        let morde = match layer.op.relevo(layer.erase_scope) {
            RelevoDaOp::Corpo => false,
            RelevoDaOp::Morde => true,
            RelevoDaOp::Nada => return,
        };
        // A lista com o Tiling, e os GRUPOS dela: uma cópia embrulhada liga-se ao antecessor do
        // ORIGINAL dela, senão o corpo seria uma barra atravessada na tela.
        let tiling = self.paint.tiling;
        let (lista, grupos) = if tiling[0] || tiling[1] {
            super::tiling::tiled_dabs_grouped(dabs, self.source_size, tiling)
        } else {
            (dabs.to_vec(), Vec::new())
        };
        let grupos_de_antes = std::mem::replace(&mut self.paint.dab_groups, grupos);
        let saved_strength = self.paint.brush.strength;
        let saved_hardness = self.paint.brush.hardness;
        let saved_eraser = self.paint.eraser;
        let saved_rng = self.paint.tex_rng;
        self.paint.brush.strength = layer.strength;
        self.paint.brush.hardness = layer.hardness.unwrap_or(saved_hardness);
        self.paint.eraser = morde;
        self.paint.tex_rng = self.paint.rng_camada[pos];
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        // O envelope é da CAMADA só quando uma borracha acima o vai apagar (8b); a mordida (`morde`)
        // escreve o relevo ASSENTE e nunca o envelope.
        let proprio = !morde && self.corpo_por_camada();
        let estado = &mut self.paint.pilha.relevo[pos];
        std::mem::swap(
            &mut self.paint.relief.last_height_center,
            &mut estado.cadeia,
        );
        std::mem::swap(&mut self.paint.relief.stroke_wave, &mut estado.onda);
        if proprio {
            estado.planos.troca(&mut self.paint.relief);
        }

        let spec = self.stroke_spec();
        self.stamp_dabs_height(&lista, &spec);

        let estado = &mut self.paint.pilha.relevo[pos];
        if proprio {
            estado.planos.troca(&mut self.paint.relief);
        }
        std::mem::swap(
            &mut self.paint.relief.last_height_center,
            &mut estado.cadeia,
        );
        std::mem::swap(&mut self.paint.relief.stroke_wave, &mut estado.onda);
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        self.paint.tex_rng = saved_rng;
        self.paint.eraser = saved_eraser;
        self.paint.brush.hardness = saved_hardness;
        self.paint.brush.strength = saved_strength;
        self.paint.dab_groups = grupos_de_antes;
    }

    /// **Há uma Erase — ou um Smear que leva corpo — viva por CIMA de um Brush vivo?** — a porta
    /// ÚNICA da pergunta, lida pelo depósito (o envelope passa a ser da camada) e pela recomposição
    /// (quem o volta a juntar). Duas respostas deixariam um envelope próprio sem ninguém que o juntasse.
    pub(super) fn corpo_por_camada(&self) -> bool {
        let e_viva = |pos: usize, op: CompositeOp| {
            self.camada_viva(pos) && self.paint.composite[pos].op == op
        };
        let leva = self.o_esfregao_leva_corpo();
        // A posição 0 é o TOPO: uma camada em `e` está por cima de todo `b > e`.
        (0..N_CAMADAS).any(|e| {
            (e_viva(e, CompositeOp::Erase) || (leva && e_viva(e, CompositeOp::Smear)))
                && (e + 1..N_CAMADAS).any(|b| e_viva(b, CompositeOp::Brush))
        })
    }

    /// O esfregão arrasta o CORPO? — o interruptor `Affect Relief` e o `Plow` do pincel, as duas
    /// perguntas que a sessão dele já faz ao relevo ASSENTE (`warp_render_relief`).
    fn o_esfregao_leva_corpo(&self) -> bool {
        #[cfg(test)]
        if ESFREGAO_SEM_CORPO.with(std::cell::Cell::get) {
            return false;
        }
        self.paint.warp.affect_relief && self.paint.brush.effective_impasto_plow() > 0.0
    }

    /// ⭐ **O envelope do traço, recomposto de baixo para cima** sobre a região da composição
    /// (8b, e o report de 2026-09-30).
    ///
    /// Corre DEPOIS da composição da cor: é nela que o esfregão acumula o deslocamento do lote. A
    /// região é a da composição (que já inclui o que o esfregão tocou) crescida pelo MAIOR raio do
    /// lote: o corpo de um dab é varrido até ao centro do anterior, logo pode escrever até um raio
    /// para lá da pegada.
    ///
    /// ⭐ **Um Smear por cima de Brushes arrasta o CORPO deles pelo MESMO deslocamento da cor**
    /// (report do dono, 2026-09-30, com foto: *«smear puxa a cor e não puxa o relevo»*): o envelope
    /// das camadas de BAIXO é escrito numa base (o slot de planos do próprio Smear, que não deposita
    /// nada) e lido em `p − Plow·disp(p)`, exactamente como a cor lê `base(p − disp(p))` e o relevo
    /// assente lê `pre_h`. Onde `disp = 0` a amostra bilinear devolve o valor do próprio texel, ao
    /// bit — o caminho sem esfregão não muda.
    pub(super) fn compoe_o_corpo(&mut self, caixa: Region, camadas: &[Vec<Dab>; N_CAMADAS]) {
        if !self.corpo_por_camada() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let raio = camadas
            .iter()
            .flatten()
            .map(|d| d.radius_px)
            .fold(0.0_f32, f32::max);
        let Some(r) = super::region::grow_region(caixa, raio.ceil() as u32 + 2, w, h) else {
            return;
        };
        // De baixo (posição N−1) para cima (0): o que cada uma é, e se entra.
        let ordem: Vec<(usize, CompositeOp)> = (0..N_CAMADAS)
            .rev()
            .filter(|&p| self.camada_viva(p))
            .map(|p| (p, self.paint.composite[p].op))
            .collect();
        let algum = ordem.iter().any(|&(p, op)| {
            op == CompositeOp::Brush && self.paint.pilha.relevo[p].planos.paint.len() == n
        });
        if !algum {
            return;
        }
        // O esfregão que leva corpo, e o deslocamento dele — `None` = nenhum (a dobra é uma só).
        let ds = self.paint.warp.relief_disp_scale;
        let disp = std::sync::Arc::clone(&self.paint.warp.disp);
        let esfregao =
            (self.o_esfregao_leva_corpo() && self.paint.warp.active && ds > 0.0 && disp.len() == n)
                .then(|| ordem.iter().position(|&(_, op)| op == CompositeOp::Smear))
                .flatten();
        let spec = self.stroke_spec();
        let push = spec.effective_impasto_push();
        {
            let relief = &mut self.paint.relief;
            for (v, zero) in [
                (&mut relief.stroke_height, 0.0),
                (&mut relief.stroke_paint, 0.0),
                (&mut relief.stroke_radius, 0.0),
            ] {
                if v.len() != n {
                    *v = vec![zero; n];
                }
            }
            for v in [&mut relief.stroke_grain, &mut relief.stroke_film] {
                if v.len() != n {
                    *v = vec![0u8; n];
                }
            }
        }
        // A BASE do esfregão: o envelope das camadas de baixo dele, escrito sobre a região.
        let (acima, base) = match esfregao {
            Some(k) => {
                let pos = ordem[k].0;
                let mut base = std::mem::take(&mut self.paint.pilha.relevo[pos].planos);
                base.garante(n);
                for py in r.y..r.y + r.h {
                    for px in r.x..r.x + r.w {
                        let i = py as usize * w as usize + px as usize;
                        let e = dobra(Envelope::default(), &ordem[..k], i, &self.paint.pilha, n);
                        base.escreve(i, e);
                    }
                }
                (&ordem[k + 1..], Some((pos, base)))
            }
            None => (&ordem[..], None),
        };
        let pilha = &self.paint.pilha;
        let relief = &mut self.paint.relief;
        let com_push = push > 0.0 && relief.stroke_push.len() == n;
        let mut spec_i = spec;
        for py in r.y..r.y + r.h {
            let linha = py as usize * w as usize;
            for px in r.x..r.x + r.w {
                let i = linha + px as usize;
                let inicio = match &base {
                    Some((_, b)) => {
                        let d = disp[i];
                        b.amostra(w, h, px as f32 - d[0] * ds, py as f32 - d[1] * ds)
                    }
                    None => Envelope::default(),
                };
                let e = dobra(inicio, acima, i, pilha, n);
                spec_i.radius_px = e.raio;
                let mut altura = ph2d_painter_brush::height::derive_height(
                    &spec_i,
                    e.tinta,
                    f32::from(e.grao) / 255.0,
                );
                if com_push {
                    altura += push * relief.stroke_push[i];
                }
                relief.stroke_paint[i] = e.tinta;
                relief.stroke_grain[i] = e.grao;
                relief.stroke_film[i] = e.filme;
                relief.stroke_radius[i] = e.raio;
                relief.stroke_height[i] = altura;
            }
        }
        if let Some((pos, b)) = base {
            self.paint.pilha.relevo[pos].planos = b;
        }
        self.mark_dirty(r);
    }
}

// `true` = o esfregão não leva o corpo do traço (o código de antes de 2026-09-30) — o CONTROLO.
#[cfg(test)]
thread_local! {
    pub(super) static ESFREGAO_SEM_CORPO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O envelope num texel — os quatro ingredientes que a dobra carrega.
#[derive(Clone, Copy, Default)]
struct Envelope {
    tinta: f32,
    grao: u8,
    filme: u8,
    raio: f32,
}

/// **A dobra de baixo para cima** das camadas `ops` num texel, a partir de `e`: um Brush entra pelo
/// MÁXIMO (o envelope de uma passagem é um máximo), uma Erase multiplica pelo que o escudo dela
/// deixou. O Blur não tem corpo e o Smear é tratado por quem chama (ele é uma amostra, não uma dobra).
fn dobra(
    mut e: Envelope,
    ops: &[(usize, CompositeOp)],
    i: usize,
    pilha: &super::composite_pilha::PilhaDoTraco,
    n: usize,
) -> Envelope {
    for &(p, op) in ops {
        match op {
            CompositeOp::Brush => {
                let c = &pilha.relevo[p].planos;
                if c.paint.len() != n {
                    continue;
                }
                if c.paint[i] > e.tinta {
                    (e.tinta, e.grao, e.raio) = (c.paint[i], c.grain[i], c.radius[i]);
                }
                e.filme = e.filme.max(c.film[i]);
            }
            CompositeOp::Erase => {
                let escudo = &pilha.planos[p];
                if escudo.len() != n * 4 {
                    continue;
                }
                let k = f32::from(escudo[i * 4 + 3]) / 255.0;
                e.tinta *= k;
                // A mesma arredondação da `cover` na borracha avulsa.
                e.filme = (f32::from(e.filme) * k) as u8;
            }
            CompositeOp::Blur | CompositeOp::Smear => {}
        }
    }
    e
}

impl PlanosDoCorpo {
    fn garante(&mut self, n: usize) {
        if self.paint.len() != n {
            self.paint = vec![0.0; n];
            self.radius = vec![0.0; n];
            self.grain = vec![0u8; n];
            self.film = vec![0u8; n];
        }
    }

    fn escreve(&mut self, i: usize, e: Envelope) {
        self.paint[i] = e.tinta;
        self.grain[i] = e.grao;
        self.film[i] = e.filme;
        self.radius[i] = e.raio;
    }

    /// A amostra bilinear em `(x, y)` — os MESMOS amostradores com que o relevo assente segue a cor.
    fn amostra(&self, w: u32, h: u32, x: f32, y: f32) -> Envelope {
        use super::warp::relief::{bilinear_f32, bilinear_u8};
        Envelope {
            tinta: bilinear_f32(&self.paint, w, h, x, y),
            grao: bilinear_u8(&self.grain, w, h, x, y),
            filme: bilinear_u8(&self.film, w, h, x, y),
            raio: bilinear_f32(&self.radius, w, h, x, y),
        }
    }
}
