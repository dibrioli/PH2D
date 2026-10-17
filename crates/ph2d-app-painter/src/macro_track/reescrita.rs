//! **Editar uma etapa reescreve a imagem do artista** (W8 item 2b, `docs/Macros/05_plano_W8.md` §2b): um
//! `Macro.doc` novo com a MESMA gravação crua e outra trilha que toca guarda o fim antigo; com o fim novo
//! pronto, cada texel da imagem que ainda é o do fim antigo passa ao do fim novo — no Painter ligado à
//! sprite (um passo dele, por camada) ou nos píxeis da sprite (o passo global). Os dois passos levam o
//! mesmo número de transação: um Ctrl+Z desfaz a edição e a imagem.

use super::{PalcosDoPainter, fim};
use ph2d_tool_painter::PainterTool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// ⭐ **Quanto a reescrita espera que a água do editor pare** antes de comparar (a água a correr não é
/// nem o fim antigo nem o novo). Medido (W7, `QUADROS_ATE_ASSENTAR`): a água das pinceladas de teste pára
/// em 2 645–3 562 quadros (≤ 59 s); o dobro. Enquanto espera, o desfazer global não regista — o tecto é
/// do artista, que não pode ficar sem passos de undo.
const ESPERA_PELA_AGUA: Duration = Duration::from_secs(120);

/// As transações das reescritas (o `0` não se usa).
static TRANSACOES: AtomicU64 = AtomicU64::new(1);

/// O fim da trilha que a imagem ainda tem, à espera do novo.
pub(super) struct PorReescrever {
    pub(super) de: fim::FimDaGravacao,
    desde: Instant,
}

impl PorReescrever {
    pub(super) fn de(de: fim::FimDaGravacao) -> Self {
        Self {
            de,
            desde: Instant::now(),
        }
    }
}

/// Lê a sprite (bits) em RGBA8 straight.
pub type LeSprite<'a, R> = dyn FnMut(u64, &mut R) -> Option<(Vec<u8>, u32, u32)> + 'a;
/// Escreve os píxeis (RGBA8 straight, `w`×`h`) na sprite (bits) — a porta de edição de imagem da shell.
pub type EscreveSprite<'a, R> = dyn FnMut(u64, Vec<u8>, u32, u32, &mut R) + 'a;

/// `rgba` onde ainda é `de` passa a `para` (straight, do mesmo tamanho). Quantos texels mudaram.
fn reescreve_a_imagem(rgba: &mut [u8], de: &[u8], para: &[u8]) -> usize {
    let mut n = 0;
    for ((a, d), p) in rgba
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(de.as_chunks::<4>().0)
        .zip(para.as_chunks::<4>().0)
    {
        if a == d && d != p {
            *a = *p;
            n += 1;
        }
    }
    n
}

impl PalcosDoPainter {
    /// **Uma reescrita espera** o fim novo (ou a água do editor): o desfazer global não regista enquanto
    /// isto for `true` (a edição e a imagem entram no MESMO passo).
    #[must_use]
    pub fn a_reescrever(&self) -> bool {
        self.lidos
            .values()
            .any(|l| matches!(&l.palco, Ok(Some(p)) if p.por_reescrever.is_some()))
    }

    /// A transação da última reescrita, até o passo global a levar ([`Self::toma_a_transacao`]).
    #[must_use]
    pub fn transacao(&self) -> Option<u64> {
        self.transacao
    }

    /// O passo global regista-se: leva a transação.
    pub fn toma_a_transacao(&mut self) -> Option<u64> {
        self.transacao.take()
    }

    /// **Por quadro, antes de [`Self::sobe`]** (nunca a meio de um gesto): aplica as reescritas cujos
    /// fins (o antigo e o novo) já chegaram. `painter` = o Painter do editor (o documento dele, se está
    /// ligado à sprite); senão a sprite lê-se por `le` e escreve-se por `escreve`. Devolve a transação,
    /// se escreveu; ela fica também à espera do passo global ([`Self::toma_a_transacao`]).
    pub fn reescreve<R: ?Sized>(
        &mut self,
        mut painter: Option<&mut PainterTool>,
        le: &mut LeSprite<'_, R>,
        escreve: &mut EscreveSprite<'_, R>,
        ranhuras: &mut R,
    ) -> Option<u64> {
        #[cfg(test)]
        let calculo = self.calcula_o_fim;
        #[cfg(not(test))]
        let calculo = None;
        // Uma transação ainda por registar continua: o passo global leva as duas.
        let mut transacao = self.transacao;
        let mut escreveu = false;
        for l in self.lidos.values_mut() {
            let Ok(Some(p)) = &mut l.palco else {
                continue;
            };
            let Some(pr) = &mut p.por_reescrever else {
                continue;
            };
            let (de, para) = match calculo {
                Some(f) => (pr.de.colhe_com(f).cloned(), p.fim.colhe_com(f).cloned()),
                None => (pr.de.colhe().cloned(), p.fim.colhe().cloned()),
            };
            if pr.de.erro().is_some() || p.fim.erro().is_some() || p.sprite.is_none() {
                p.por_reescrever = None;
                continue;
            }
            let (Some(de), Some(para), Some(sprite)) = (de, para, p.sprite) else {
                continue;
            };
            let ligado = painter
                .as_deref_mut()
                .filter(|t| t.bound_document() == Some(sprite));
            let prazo = pr.desde.elapsed() >= ESPERA_PELA_AGUA;
            let n = if let Some(t) = ligado {
                if !t.agua_parada() && !prazo {
                    continue;
                }
                let t_id =
                    transacao.unwrap_or_else(|| TRANSACOES.fetch_add(1, Ordering::Relaxed));
                let n = t.reescreve_a_gravacao(&de.modelo, &para.modelo, t_id);
                if n > 0 {
                    transacao = Some(t_id);
                }
                n
            } else {
                match le(sprite, ranhuras) {
                    Some((mut rgba, w, h))
                        if (w, h) == (de.fim.w, de.fim.h)
                            && rgba.len() == de.fim_reto.len()
                            && rgba.len() == para.fim_reto.len() =>
                    {
                        let n = reescreve_a_imagem(&mut rgba, &de.fim_reto, &para.fim_reto);
                        if n > 0 {
                            transacao.get_or_insert_with(|| {
                                TRANSACOES.fetch_add(1, Ordering::Relaxed)
                            });
                            escreve(sprite, rgba, w, h, ranhuras);
                        }
                        n
                    }
                    // Um quadro sem leitura tenta no seguinte, até ao prazo; outro tamanho não casa.
                    None if !prazo => continue,
                    _ => 0,
                }
            };
            p.por_reescrever = None;
            // A imagem mudou: a mistura relê-a.
            if n > 0 {
                p.atual = None;
                escreveu = true;
            }
        }
        self.transacao = transacao;
        transacao.filter(|_| escreveu)
    }
}
