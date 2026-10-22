//! **O preview VIVO do pincel do Wet Painter** (a pedido do dono, 08/10): a cada mudança de qualquer
//! parâmetro que o traço mostra — o pincel, os knobs, o papel, a cor, a obra — o preview repinta-se
//! numa thread própria e anima o traço UMA vez.
//!
//! Porquê numa thread: pintar o S custa `33`–`80` ms no perfil `smoke` (sonda
//! `presets_de_fabrica::sonda_custo_da_miniatura`, 08/10), e o quadro do ecrã tem `16,7` ms. Porquê
//! gravado e depois TOCADO: a simulação inteira cabe nesses ~80 ms, e uma animação de 80 ms não se vê —
//! os quadros tocam-se ao ritmo [`QUADRO_DA_ANIMACAO`].
//!
//! A lei da apresentação ([`Apresentacao`]): uma mudança ISOLADA anima assim que os quadros chegam;
//! mudanças seguidas (um arrasto de slider, menos de [`SOSSEGO`] entre elas) mostram o quadro mais
//! recente de cada uma, ao vivo, e animam uma vez quando o arrasto pára — nunca à espera da água parar.

use ph2d_panel_painter_layers::set_current_wet_preview;
use ph2d_tool_painter::{
    ALTURA_DO_PREVIEW, CenaDoPreview, ChaveDoPreview, LARGURA_DO_PREVIEW, PainterTool,
    pintar_preview,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// O intervalo entre dois quadros tocados: ~30 por segundo — o S (24 quadros) e um quadro por segundo
/// de água até ela parar (Wet Round ~33, Wet Wash ~60): ~2–3 s de animação.
pub(crate) const QUADRO_DA_ANIMACAO: Duration = Duration::from_millis(33);
/// Duas mudanças mais próximas que isto são UM gesto (o arrasto de um slider dá uma a cada quadro).
pub(crate) const SOSSEGO: Duration = Duration::from_millis(180);

/// Um quadro do preview (`LARGURA_DO_PREVIEW × ALTURA_DO_PREVIEW`, a faixa 2:1 do S e dos fios).
pub(crate) type Quadro = Arc<Vec<u8>>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Modo {
    /// Toca os quadros desde `inicio` (posto quando o primeiro quadro chega).
    Animar { inicio: Option<Instant> },
    /// Mudanças seguidas: mostra o quadro MAIS RECENTE da última cena até o gesto sossegar.
    Arrasto,
}

/// **O que o painel mostra em cada instante** — pura (o relógio vem de fora), para se testar sem esperar.
pub(crate) struct Apresentacao {
    geracao: u64,
    ultima_mudanca: Option<Instant>,
    modo: Modo,
    quadros: Vec<Quadro>,
    completo: bool,
    mostrado: Option<Quadro>,
}

impl Apresentacao {
    pub(crate) fn nova() -> Self {
        Self {
            geracao: 0,
            ultima_mudanca: None,
            modo: Modo::Animar { inicio: None },
            quadros: Vec::new(),
            completo: false,
            mostrado: None,
        }
    }

    /// A cena mudou: uma geração nova (os quadros da anterior deixam de valer).
    pub(crate) fn mudou(&mut self, agora: Instant) -> u64 {
        let seguida = self
            .ultima_mudanca
            .is_some_and(|t| agora.saturating_duration_since(t) < SOSSEGO);
        self.modo = if seguida {
            Modo::Arrasto
        } else {
            Modo::Animar { inicio: None }
        };
        self.ultima_mudanca = Some(agora);
        self.geracao += 1;
        self.quadros.clear();
        self.completo = false;
        self.geracao
    }

    /// Um quadro da thread; os de uma geração velha deitam-se fora.
    pub(crate) fn chegou(&mut self, geracao: u64, quadro: Quadro) {
        if geracao == self.geracao {
            self.quadros.push(quadro);
        }
    }

    /// A thread acabou a geração `geracao`.
    pub(crate) fn acabou(&mut self, geracao: u64) {
        if geracao == self.geracao {
            self.completo = true;
        }
    }

    /// O quadro a mostrar `agora`. Enquanto a cena nova não tem quadros, fica o que estava.
    pub(crate) fn quadro(&mut self, agora: Instant) -> Option<Quadro> {
        // ⚠️ Nem o arrasto nem a animação esperam o FIM da cena (dono 10/10: «demora até 10 s»): a água
        // do preview exato leva 4–8 s até parar, o traço sai em 0,2–0,4 s (`medida_o_relogio_do_preview`).
        if let Modo::Arrasto = self.modo
            && self
                .ultima_mudanca
                .is_some_and(|t| agora.saturating_duration_since(t) >= SOSSEGO)
        {
            // O gesto sossegou: a animação, uma vez, a seguir os quadros à medida que chegam.
            self.modo = Modo::Animar { inicio: None };
        }
        let escolhido = match &mut self.modo {
            Modo::Arrasto => self.quadros.last().cloned(),
            Modo::Animar { inicio } if !self.quadros.is_empty() => {
                let t0 = *inicio.get_or_insert(agora);
                let i = (agora.saturating_duration_since(t0).as_millis()
                    / QUADRO_DA_ANIMACAO.as_millis().max(1)) as usize;
                self.quadros.get(i.min(self.quadros.len() - 1)).cloned()
            }
            Modo::Animar { .. } => None,
        };
        if let Some(q) = escolhido {
            self.mostrado = Some(q);
        }
        self.mostrado.clone()
    }
}

enum Mensagem {
    Quadro(u64, Quadro),
    Fim(u64),
}

/// O pedido que espera a thread: só o MAIS RECENTE conta (um arrasto não enfileira cenas velhas).
type Pedido = Arc<(Mutex<Option<(u64, CenaDoPreview)>>, Condvar)>;

struct PreviewVivo {
    pedido: Pedido,
    viva: Arc<AtomicU64>,
    recebe: Receiver<Mensagem>,
    chave: Option<ChaveDoPreview>,
    apresentacao: Apresentacao,
    publicado: Option<Quadro>,
}

fn trabalhador(pedido: Pedido, viva: Arc<AtomicU64>, envia: Sender<Mensagem>) {
    loop {
        let (geracao, cena) = {
            let (m, cv) = &*pedido;
            let mut g = m.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            loop {
                if let Some(p) = g.take() {
                    break p;
                }
                g = cv
                    .wait(g)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };
        let parar = || viva.load(Ordering::Relaxed) != geracao;
        let completo = pintar_preview(&cena, &parar, &mut |px, _w, _h| {
            let _ = envia.send(Mensagem::Quadro(geracao, Arc::new(px.to_vec())));
        });
        if completo && envia.send(Mensagem::Fim(geracao)).is_err() {
            return;
        }
    }
}

impl PreviewVivo {
    fn novo() -> Self {
        let pedido: Pedido = Arc::new((Mutex::new(None), Condvar::new()));
        let viva = Arc::new(AtomicU64::new(0));
        let (envia, recebe) = channel();
        let (p, v) = (Arc::clone(&pedido), Arc::clone(&viva));
        // Sem a thread não há preview vivo, e o painel continua a funcionar: o erro diz-se uma vez.
        if let Err(e) = std::thread::Builder::new()
            .name("ph2d-preview-pincel".into())
            .spawn(move || trabalhador(p, v, envia))
        {
            eprintln!("[preview] a thread do preview não arrancou: {e}");
        }
        Self {
            pedido,
            viva,
            recebe,
            chave: None,
            apresentacao: Apresentacao::nova(),
            publicado: None,
        }
    }

    fn quadro(&mut self, painter: &PainterTool, agora: Instant) {
        let chave = painter.wet_preview_chave();
        if self.chave.as_ref() != Some(&chave) {
            self.chave = Some(chave);
            let g = self.apresentacao.mudou(agora);
            self.viva.store(g, Ordering::Relaxed);
            let (m, cv) = &*self.pedido;
            *m.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some((g, painter.wet_preview_cena()));
            cv.notify_one();
        }
        while let Ok(msg) = self.recebe.try_recv() {
            match msg {
                Mensagem::Quadro(g, q) => self.apresentacao.chegou(g, q),
                Mensagem::Fim(g) => self.apresentacao.acabou(g),
            }
        }
        let q = self.apresentacao.quadro(agora);
        let mudou = match (&q, &self.publicado) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (a, b) => a.is_some() != b.is_some(),
        };
        if mudou {
            set_current_wet_preview(
                q.clone()
                    .map(|px| (px, LARGURA_DO_PREVIEW, ALTURA_DO_PREVIEW)),
            );
            self.publicado = q;
        }
    }
}

thread_local! {
    static PREVIEW: RefCell<Option<PreviewVivo>> = const { RefCell::new(None) };
}

/// A cada quadro com o Wet Paint armado: repinta o preview se a cena mudou e publica o quadro da vez.
pub(crate) fn drive(painter: &PainterTool) {
    PREVIEW.with(|c| {
        c.borrow_mut()
            .get_or_insert_with(PreviewVivo::novo)
            .quadro(painter, Instant::now());
    });
}

#[cfg(test)]
#[path = "painter_bridge_preview_foto_tests.rs"]
mod foto_tests;
#[cfg(test)]
#[path = "painter_bridge_preview_tests.rs"]
mod tests;
