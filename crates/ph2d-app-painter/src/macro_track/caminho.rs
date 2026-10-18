//! `PH2D_MACRO_SMOKE=6` — **o caminho do dono, sozinho** (`docs/Macros/handoffs/
//! HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md` passo 1): a cena `=5`, e o app faz o que o dono
//! fez — Gravar, duas pinceladas de Wet Paint pelo Painter do editor, Parar, ⏮ e ▷ na aba Keys — pelas
//! MESMAS portas da shell (o painel MACROS, o ponteiro da tela, o transporte da Timeline). O roteiro é
//! no relógio da cena (o do dono é o de parede); a ordem dos gestos é fixa. A cada décimo de segundo a
//! consola diz o instante do clipe, o palco, a marca, o editor e o que a subida anterior MOSTROU
//! ([`PalcosDoPainter::diagnostico`]). A `=5` escreve o mesmo registo, sem gestos.
//!
//! Texto de consola e do ficheiro de registo, lido pela LLM: isento do HR-15 por nome
//! (`tests/it/every_word_this_family_shows_comes_from_the_string_table.rs`).
//!
//! A variante `arrasta` é o smoke de 18/10 (buracos retangulares ao arrastar a cabeça da Keys): em
//! Digital, como a `=5` do dono, saltos para trás e para a frente e um arrasto contínuo; o diagnóstico
//! conta os texels do palco que diferem da tela inteira dele (`inteira=`) e, com a cabeça parada, de
//! uma reprodução À PARTE da mesma macro no mesmo instante (`fresca=`).

use super::PalcosDoPainter;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;

/// Um gesto do dono, como a shell o recebe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesto {
    /// O Record do painel MACROS.
    Grava,
    /// O botão do rato desce na tela, em fracções da imagem `(u, v)`.
    Desce([f32; 2]),
    Move([f32; 2]),
    Sobe,
    /// O Stop do painel MACROS.
    Para,
    /// O ⏮ da aba Keys.
    Rewind,
    /// O ▷ da aba Keys.
    Play,
    /// O Wet Paint no seletor de meio do Painter.
    MeioWet,
    /// O Play da barra de cima (corre a cena: só os clipes em Autoplay tocam).
    PlayDeCima,
    /// A cabeça da Keys arrastada até este instante do clipe (segundos).
    Arrasta(f32),
}

impl Gesto {
    /// O instante para onde o gesto leva a cabeça da Keys (o ⏮ é o arrasto até `0`).
    #[must_use]
    pub fn segundos(self) -> f64 {
        match self {
            Self::Arrasta(s) => f64::from(s),
            _ => 0.0,
        }
    }
}

/// Como o dono pode ter feito o caminho (`PH2D_MACRO_CAMINHO`): as variantes correm na mesma rodada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variante {
    /// Gravar, 2 pinceladas, Parar, ⏮, ▷ — depressa.
    #[default]
    Base,
    /// O mesmo ao ritmo de uma mão: pausas de segundos, a água a secar entre os gestos.
    Humano,
    /// O Wet Paint escolhido no seletor DEPOIS de Gravar (a cena abre em Digital).
    Meio,
    /// O Splash tocado e pausado antes de Gravar.
    Splash,
    /// Parar e ▷, sem ⏮.
    SemRewind,
    /// O caminho do smoke do dono de 18/10: Parar e o Play da barra de cima.
    Topo,
    /// O 2.º smoke de 18/10: em Digital, Parar e a cabeça da Keys arrastada para trás e para a frente.
    Arrasta,
    /// A base, com uma linha POR QUADRO no segundo e meio depois do Parar e, em cada uma, a diferença
    /// entre o que a ranhura mostrou e uma reprodução à parte (`fresca=`).
    Parar,
}

/// A variante desta corrida.
#[must_use]
pub fn variante() -> Variante {
    match std::env::var("PH2D_MACRO_CAMINHO").as_deref() {
        Ok("humano") => Variante::Humano,
        Ok("meio") => Variante::Meio,
        Ok("splash") => Variante::Splash,
        Ok("semrewind") => Variante::SemRewind,
        Ok("topo") => Variante::Topo,
        Ok("arrasta") => Variante::Arrasta,
        Ok("parar") => Variante::Parar,
        _ => Variante::Base,
    }
}

const PASSO_MS: u64 = 16;

/// Uma pincelada horizontal na linha `v`, de `t0` (ms) durante `dura` ms.
fn pincelada(r: &mut Vec<(u64, Gesto)>, t0: u64, v: f32, dura: u64) {
    let passos = dura / PASSO_MS;
    for i in 0..=passos {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let u = 0.2 + 0.6 * (i as f32 / passos as f32);
        let g = if i == 0 {
            Gesto::Desce([u, v])
        } else {
            Gesto::Move([u, v])
        };
        r.push((t0 + i * PASSO_MS, g));
    }
    r.push((t0 + dura + PASSO_MS, Gesto::Sobe));
}

/// **O roteiro da variante `v`**: cada gesto e o instante (ms do relógio da cena), por ordem.
#[must_use]
pub fn roteiro(v: Variante) -> Vec<(u64, Gesto)> {
    let mut r = Vec::new();
    // O Splash antes: ▷ e, dois segundos depois, ▷ outra vez (pausa); o resto anda 3 s.
    let d = if v == Variante::Splash {
        r.extend([(500, Gesto::Play), (2_500, Gesto::Play)]);
        3_000
    } else {
        0
    };
    let (dura, t1, t2, para, rewind, play) = if v == Variante::Humano {
        (1_500, 3_000, 7_000, 11_000, 13_000, 14_500)
    } else {
        (600, 2_000, 3_600, 5_200, 5_700, 6_200)
    };
    r.push((d + 1_000, Gesto::Grava));
    if v == Variante::Meio {
        r.push((d + 1_500, Gesto::MeioWet));
    }
    pincelada(&mut r, d + t1, 0.30, dura);
    pincelada(&mut r, d + t2, 0.50, dura);
    r.push((d + para, Gesto::Para));
    if v == Variante::Arrasta {
        arrastos(&mut r, para + 800);
        return r;
    }
    if !matches!(v, Variante::SemRewind | Variante::Topo) {
        r.push((d + rewind, Gesto::Rewind));
    }
    let play_g = if v == Variante::Topo {
        Gesto::PlayDeCima
    } else {
        Gesto::Play
    };
    r.push((d + play, play_g));
    r
}

/// Os saltos do dono a cada 1,5 s (para trás e para a frente, dois a meio de uma pincelada), depois um
/// arrasto contínuo da mão para trás e outra vez para a frente, a partir de `t0` (ms).
fn arrastos(r: &mut Vec<(u64, Gesto)>, t0: u64) {
    let saltos = [3.0, 1.4, 2.8, 0.9, 3.4, 2.0];
    let mut t = t0;
    for s in saltos {
        r.push((t, Gesto::Arrasta(s)));
        t += 1_500;
    }
    #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
    let mao = |k: u64| 3.8 - 3.2 * (k as f32 / 50.0);
    r.extend((0..=50).map(|k| (t + k * 32, Gesto::Arrasta(mao(k)))));
    t += 50 * 32 + 1_500;
    r.extend((0..=50).map(|k| (t + k * 32, Gesto::Arrasta(mao(50 - k)))));
}

/// O condutor: os gestos já dados e a última linha da consola.
#[derive(Default)]
pub struct Caminho {
    variante: Variante,
    dados: usize,
    /// Até quando (ms) há uma linha por quadro (a variante `parar`, depois do Parar).
    por_quadro_ate: Option<u64>,
    ultimo_diag_ms: Option<u64>,
    /// Quadros desde a última linha de diagnóstico.
    quadros: u32,
}

thread_local! {
    /// O relógio da cena e o condutor do roteiro (nasce no 1.º quadro da cena).
    static CAMINHO: std::cell::RefCell<Option<(std::time::Instant, Caminho)>> =
        const { std::cell::RefCell::new(None) };
}

/// **O passo deste quadro** da cena `=5`/`=6`: o relógio da cena (ms), os gestos que vencem (só com
/// `conduz`, a `=6`) e, quando devida, a hora de uma linha de diagnóstico (os quadros desde a anterior).
/// O 1.º passo anuncia a cena.
pub fn passo(conduz: bool) -> (u64, Vec<Gesto>, Option<u32>) {
    CAMINHO.with_borrow_mut(|c| {
        let (t0, cam) = c.get_or_insert_with(|| {
            regista(&anuncio(conduz));
            (std::time::Instant::now(), Caminho::de(variante()))
        });
        let ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
        let gestos = if conduz {
            cam.gestos_ate(ms)
        } else {
            Vec::new()
        };
        (ms, gestos, cam.hora_do_diagnostico(ms))
    })
}

impl Caminho {
    /// O condutor do roteiro da variante `v`.
    #[must_use]
    pub fn de(v: Variante) -> Self {
        Self {
            variante: v,
            ..Self::default()
        }
    }

    /// Os gestos que vencem até `ms` (por ordem, cada um uma vez).
    pub fn gestos_ate(&mut self, ms: u64) -> Vec<Gesto> {
        let r = roteiro(self.variante);
        let novos: Vec<Gesto> = r
            .iter()
            .skip(self.dados)
            .take_while(|(t, _)| *t <= ms)
            .map(|(_, g)| *g)
            .collect();
        self.dados += novos.len();
        if self.variante == Variante::Parar && novos.contains(&Gesto::Para) {
            self.por_quadro_ate = Some(ms + 1_500);
        }
        novos
    }

    /// A cada 100 ms do relógio da cena, a hora de uma linha de diagnóstico: os quadros desde a
    /// anterior (a velocidade do app, de que um defeito pode depender).
    pub fn hora_do_diagnostico(&mut self, ms: u64) -> Option<u32> {
        self.quadros += 1;
        let devida = self.ultimo_diag_ms.is_none_or(|u| ms >= u + 100)
            || self.por_quadro_ate.is_some_and(|f| ms <= f);
        devida.then(|| {
            self.ultimo_diag_ms = Some(ms);
            std::mem::take(&mut self.quadros)
        })
    }
}

/// O que a cena diz ao abrir: a `=6` (`conduz`) faz o caminho sozinha; a `=5` só o regista.
#[must_use]
pub fn anuncio(conduz: bool) -> String {
    let f = ficheiro_do_registo();
    if !conduz {
        return format!(
            "this session is logged every 0.1 s (Record, Stop, the Timeline clock, the canvas) to {}",
            f.display()
        );
    }
    if variante() == Variante::Arrasta {
        return format!(
            "the app does the path by itself — Record, two Digital strokes, Stop, then it drags the \
             Timeline's playhead back and forth (six jumps, 1.5 s apart, then a slow drag back and \
             forward). Do not touch the mouse for 25 s. The strokes must never show rectangular holes. \
             This log is also written to {}",
            f.display()
        );
    }
    format!(
        "the app does the path by itself — Record, two Wet Paint strokes, Stop, Rewind and Play in the \
         Timeline's Keys tab ({:?}). Do not touch the mouse for 15 s. Watch the canvas after Play: the \
         top stroke must appear first and the lower one about 1.5 s later. This log is also written to {}",
        variante(),
        f.display()
    )
}

/// O ficheiro onde as cenas `=5` e `=6` deixam o que a consola diz (o dono corre-as; a LLM lê-o depois).
#[must_use]
pub fn ficheiro_do_registo() -> std::path::PathBuf {
    std::env::temp_dir().join("ph2d-macro-smoke.log")
}

/// **Uma linha do registo**: na consola e no [`ficheiro_do_registo`] (recomeçado na 1.ª linha).
pub fn regista(linha: &str) {
    use std::io::Write as _;
    static ABERTO: std::sync::Once = std::sync::Once::new();
    let caminho = ficheiro_do_registo();
    ABERTO.call_once(|| {
        let _ = std::fs::write(&caminho, "");
    });
    eprintln!("[macro smoke] {linha}");
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&caminho) {
        let _ = writeln!(f, "{linha}");
    }
}

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
pub(super) struct Espia {
    /// Por ranhura: (largura, rgba premultiplicado).
    ranhuras: std::collections::BTreeMap<u32, (u32, Vec<u8>)>,
    pub(super) trocas: Vec<super::TrocaDaSprite>,
}

impl Espia {
    /// A subida de `rgba` à ranhura `id` (de largura `w`): inteira (`caixa = None`) ou só a caixa.
    pub(super) fn copia(&mut self, id: u32, w: u32, caixa: Option<super::Caixa>, rgba: &[u8]) {
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
        let Some(l) = self.lidos.get(&bits) else {
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
        let mut extra = mostrado(p, espia, bits, doc, variante() == Variante::Parar);
        if variante() == Variante::Arrasta {
            extra.push_str(&confere(p, doc));
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
fn mostrado(
    p: &super::PalcoDaPintura,
    e: &Espia,
    bits: u64,
    doc: Option<&[u8]>,
    fresca: bool,
) -> String {
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

/// ` inteira=N fresca=M` da variante `arrasta`: os texels do palco que diferem da tela inteira do
/// tocador dele (uma cópia, lida inteira) e — com a cabeça parada desde a linha anterior, uma vez por
/// instante — de uma reprodução À PARTE de `doc` no mesmo instante.
fn confere(p: &super::PalcoDaPintura, doc: Option<&[u8]>) -> String {
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

#[cfg(test)]
mod tests {
    use super::{Caminho, Gesto, Variante, roteiro};

    /// Cada variante grava, pinta duas pinceladas fechadas na tela, pára e só então toca — por ordem.
    #[test]
    fn o_roteiro_de_cada_variante_e_o_caminho_do_dono_por_ordem() {
        for v in [
            Variante::Base,
            Variante::Humano,
            Variante::Meio,
            Variante::Splash,
            Variante::SemRewind,
            Variante::Topo,
            Variante::Arrasta,
            Variante::Parar,
        ] {
            let r = roteiro(v);
            assert!(
                r.windows(2).all(|w| w[0].0 <= w[1].0),
                "{v:?}: fora de ordem"
            );
            let pos = |g: Gesto| r.iter().position(|(_, x)| *x == g);
            let (grava, para) = (pos(Gesto::Grava).unwrap(), pos(Gesto::Para).unwrap());
            let desce: Vec<usize> = (0..r.len())
                .filter(|&i| matches!(r[i].1, Gesto::Desce(_)))
                .collect();
            let sobe: Vec<usize> = (0..r.len()).filter(|&i| r[i].1 == Gesto::Sobe).collect();
            assert_eq!((desce.len(), sobe.len()), (2, 2), "{v:?}: duas pinceladas");
            assert!(grava < desce[0] && desce[0] < sobe[0] && sobe[0] < desce[1]);
            assert!(sobe[1] < para, "{v:?}: Parar depois da 2.ª pincelada");
            let arrasta = r.iter().filter(|(_, g)| matches!(g, Gesto::Arrasta(_)));
            let fim = if v == Variante::Arrasta {
                // Saltos para trás depois do Parar, todos dentro do clipe.
                assert!(
                    arrasta
                        .clone()
                        .all(|(t, g)| *t > r[para].0 && (0.0..=4.2).contains(&g.segundos()))
                );
                let s: Vec<f64> = arrasta.map(|(_, g)| g.segundos()).collect();
                assert!(
                    s.windows(2).filter(|w| w[1] < w[0]).count() > 10,
                    "{v:?}: arrasta para trás"
                );
                matches!(r.last().unwrap().1, Gesto::Arrasta(_))
            } else {
                assert_eq!(arrasta.count(), 0);
                matches!(r.last().unwrap().1, Gesto::Play | Gesto::PlayDeCima)
            };
            assert!(fim, "{v:?}: o Play (ou o arrasto) fecha o caminho");
            let sem = matches!(v, Variante::SemRewind | Variante::Topo | Variante::Arrasta);
            assert_eq!(pos(Gesto::Rewind).is_some(), !sem);
            assert!(r.iter().all(|(_, g)| match g {
                Gesto::Desce([u, v]) | Gesto::Move([u, v]) =>
                    (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v),
                _ => true,
            }));
        }
    }

    /// O condutor dá cada gesto uma vez, os vencidos de uma vez.
    #[test]
    fn o_condutor_da_cada_gesto_uma_vez() {
        let mut c = Caminho::de(Variante::Base);
        let todos = roteiro(Variante::Base).len();
        let mut n = c.gestos_ate(0).len();
        for ms in (0..20_000).step_by(37) {
            n += c.gestos_ate(ms).len();
        }
        assert_eq!(n, todos);
        assert!(c.gestos_ate(u64::MAX).is_empty());
    }
}
