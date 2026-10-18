//! `PH2D_MACRO_SMOKE=6` — **o caminho do dono, sozinho** (`docs/Macros/handoffs/
//! HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md` passo 1): a cena `=5`, e o app faz o que o dono
//! fez — Gravar, duas pinceladas de Wet Paint pelo Painter do editor, Parar, ⏮ e ▷ na aba Keys — pelas
//! MESMAS portas da shell (o painel MACROS, o ponteiro da tela, o transporte da Timeline). O roteiro é
//! no relógio da cena (o do dono é o de parede); a ordem dos gestos é fixa. A cada décimo de segundo a
//! consola diz o instante do clipe, se o palco troca a textura da sprite e quantos texels pintados têm a
//! ranhura e o editor ([`PalcosDoPainter::diagnostico`]).

use super::PalcosDoPainter;
use ph2d_tool_painter::PainterTool;

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
}

/// Como o dono pode ter feito o caminho (`PH2D_MACRO_CAMINHO`): as variantes correm na mesma rodada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variante {
    /// Gravar, 2 pinceladas, Parar, ⏮, ▷ — depressa.
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

/// O condutor: os gestos já dados e a última linha da consola.
#[derive(Default)]
pub struct Caminho {
    dados: usize,
    ultimo_diag_ms: Option<u64>,
    /// Quadros desde a última linha de diagnóstico.
    quadros: u32,
}

impl Caminho {
    /// Os gestos que vencem até `ms` (por ordem, cada um uma vez).
    pub fn gestos_ate(&mut self, ms: u64) -> Vec<Gesto> {
        let r = roteiro(variante());
        let novos: Vec<Gesto> = r
            .iter()
            .skip(self.dados)
            .take_while(|(t, _)| *t <= ms)
            .map(|(_, g)| *g)
            .collect();
        self.dados += novos.len();
        novos
    }

    /// A cada 100 ms do relógio da cena, a hora de uma linha de diagnóstico: os quadros desde a
    /// anterior (a velocidade do app, de que um defeito pode depender).
    pub fn hora_do_diagnostico(&mut self, ms: u64) -> Option<u32> {
        self.quadros += 1;
        let devida = self.ultimo_diag_ms.is_none_or(|u| ms >= u + 100);
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

impl PalcosDoPainter {
    /// **A linha de diagnóstico da sprite `bits`** (a cena `=6`): o palco toca, a época, se a textura da
    /// sprite troca neste quadro (como [`Self::sobe`] decide, com `cede`), os texels pintados no palco,
    /// na marca e no que a ranhura mostra (o palco, e a imagem atual onde há marca), e os do editor.
    #[must_use]
    pub fn diagnostico(
        &self,
        bits: u64,
        cede: Option<u64>,
        painter: Option<&PainterTool>,
    ) -> String {
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
        let troca = p.toca && p.sprite.is_some() && p.ranhura.is_some() && p.sprite != cede;
        let marca = p.chave.and_then(|chave| {
            painter
                .filter(|t| p.sprite.is_some() && t.bound_document() == p.sprite)
                .and_then(PainterTool::marca_do_artista)
                .filter(|m| m.epoca() == chave.1)
                .or_else(|| self.marcas.get(&chave).map(|g| g.vale()))
        });
        let atual = p.atual.as_ref().and_then(|a| a.img.as_ref());
        let ranhura = match (marca, atual) {
            (Some(m), Some(a)) if a.rgba.len() == p.espelho.len() => p
                .espelho
                .as_chunks::<4>()
                .0
                .iter()
                .zip(a.rgba.as_chunks::<4>().0)
                .enumerate()
                .filter(|(k, (e, a))| pintados(if m.marcado(*k) { &a[..] } else { &e[..] }) > 0)
                .count(),
            _ => pintados(&p.espelho),
        };
        format!(
            "toca={} epoca={:04x} troca={troca} palco={} marca={} ranhura={ranhura} editor={editor}",
            p.toca,
            p.epoca & 0xffff,
            pintados(&p.espelho),
            marca.map_or_else(|| "-".to_owned(), |m| m.quantos().to_string()),
        )
    }
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
            assert!(
                matches!(r.last().unwrap().1, Gesto::Play | Gesto::PlayDeCima),
                "{v:?}: o Play fecha o caminho"
            );
            let sem = matches!(v, Variante::SemRewind | Variante::Topo);
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
        let mut c = Caminho::default();
        let todos = roteiro(Variante::Base).len();
        let mut n = c.gestos_ate(0).len();
        for ms in (0..20_000).step_by(37) {
            n += c.gestos_ate(ms).len();
        }
        assert_eq!(n, todos);
        assert!(c.gestos_ate(u64::MAX).is_empty());
    }
}
