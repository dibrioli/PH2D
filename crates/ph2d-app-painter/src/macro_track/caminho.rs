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
    /// O botão do rato desce no **C&F** do rail do Painter (W11).
    CeFDesce,
    /// O arrasto do C&F, em fracções da imagem `(u, v)` — fora de `[0, 1]` é fora da tela.
    CeFMove([f32; 2]),
    CeFSobe,
    /// A cor escolhida no seletor que o C&F abriu (o valor do próprio seletor, sRGB).
    EscolheCor([u8; 3]),
    /// O seletor fecha (o clique fora dele).
    FechaSeletor,
    /// O Done do modal «Fill adjust».
    FillPronto,
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
    /// O smoke de 19/10: depois do Parar, uma pincelada vertical POR CIMA das duas gravadas; ⏮ e ▷ — a
    /// vertical fica inteira enquanto as gravadas nascem (sem faixas no miolo delas).
    Cruza,
    /// A `cruza` em Wet Paint (W12): a vertical pinta por cima da gravação (a água dela pára no 1.º toque)
    /// e, no ▷, fica limpa onde cruza uma gravada por nascer — sem o «nó».
    CruzaMolhada,
    /// O smoke de 19/10 depois da W10: a gravar, a cor trocada no C&F (premido, arrastado um pouco sem
    /// entrar na tela, solto ⇒ o seletor) entre as duas pinceladas; ▷ toca as duas, a 2.ª na cor nova.
    Cor,
    /// O ColorDrop gravado: a cor escolhida no C&F, depois arrastada do C&F para a tela e solta (o
    /// preenchimento) e o Done do modal; uma pincelada a seguir. ▷ toca o preenchimento e a pincelada.
    ColorDrop,
    /// W14: Parar e a cabeça da Keys parada no fim do clipe — a água continua até à imagem do artista; o
    /// diagnóstico conta os texels da ranhura que diferem do editor (`≠editor=`).
    Fim,
}

/// A variante desta corrida.
#[must_use]
pub fn variante() -> Variante {
    let pedido = std::env::var("PH2D_MACRO_CAMINHO").unwrap_or_default();
    NOMES
        .iter()
        .find(|(n, _)| *n == pedido)
        .map_or(Variante::Base, |(_, v)| *v)
}

/// Os valores de `PH2D_MACRO_CAMINHO`. Um valor fora desta tabela corre a base — e a consola di-lo
/// ([`caminho_desconhecido`]): o dono correu `…=PH2D_MACRO_CAMINHO=colordrop` e viu a base calada (20/10).
const NOMES: &[(&str, Variante)] = &[
    ("humano", Variante::Humano),
    ("meio", Variante::Meio),
    ("splash", Variante::Splash),
    ("semrewind", Variante::SemRewind),
    ("topo", Variante::Topo),
    ("arrasta", Variante::Arrasta),
    ("parar", Variante::Parar),
    ("cruza", Variante::Cruza),
    ("cruzamolhada", Variante::CruzaMolhada),
    ("cor", Variante::Cor),
    ("colordrop", Variante::ColorDrop),
    ("fim", Variante::Fim),
];

/// O valor de `PH2D_MACRO_CAMINHO` quando ele existe e não é nenhum caminho.
#[must_use]
pub fn caminho_desconhecido() -> Option<String> {
    let pedido = std::env::var("PH2D_MACRO_CAMINHO").ok()?;
    (!pedido.is_empty() && !NOMES.iter().any(|(n, _)| *n == pedido)).then_some(pedido)
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
    if matches!(v, Variante::Cor | Variante::ColorDrop) {
        return roteiro_do_cef(v);
    }
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
    if v == Variante::Fim {
        // Para lá do fim do clipe (4,2 s): a cabeça fica no último instante.
        r.push((para + 500, Gesto::Arrasta(4.5)));
        return r;
    }
    let (rewind, play) = if matches!(v, Variante::Cruza | Variante::CruzaMolhada) {
        vertical(&mut r, d + para + 500, 0.5, 600);
        (rewind + 1_200, play + 1_200)
    } else {
        (rewind, play)
    };
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

/// O vermelho que a `cor` escolhe; o azul que o `colordrop` derrama.
pub const COR_NOVA: [u8; 3] = [200, 30, 30];
pub const COR_DO_DROP: [u8; 3] = [40, 90, 210];

/// O C&F premido, arrastado por `pontos` (fracções da imagem) e solto, a partir de `t0` (ms).
fn cef(r: &mut Vec<(u64, Gesto)>, t0: u64, pontos: &[[f32; 2]]) -> u64 {
    r.push((t0, Gesto::CeFDesce));
    let mut t = t0;
    for &p in pontos {
        t += 50;
        r.push((t, Gesto::CeFMove(p)));
    }
    r.push((t + 100, Gesto::CeFSobe));
    t + 100
}

/// As variantes do C&F (W11), em Digital.
fn roteiro_do_cef(v: Variante) -> Vec<(u64, Gesto)> {
    let mut r = vec![(1_000, Gesto::Grava)];
    pincelada(&mut r, 2_000, 0.30, 600);
    // Um pouco de arrasto que fica à ESQUERDA da tela: só o seletor abre.
    let fora = [[-0.30, 0.40], [-0.25, 0.42], [-0.20, 0.45]];
    let cor = if v == Variante::Cor {
        COR_NOVA
    } else {
        COR_DO_DROP
    };
    let t = cef(&mut r, 3_000, &fora);
    r.extend([
        (t + 250, Gesto::EscolheCor(cor)),
        (t + 550, Gesto::FechaSeletor),
    ]);
    let t = if v == Variante::ColorDrop {
        // De fora para dentro da tela, e solto lá: o preenchimento; depois o Done do modal, e o
        // vermelho no C&F para a pincelada se ver sobre o azul.
        let t = cef(
            &mut r,
            t + 900,
            &[[-0.20, 0.75], [0.50, 0.75], [0.55, 0.78]],
        );
        r.push((t + 450, Gesto::FillPronto));
        let t = cef(&mut r, t + 750, &[]);
        r.extend([
            (t + 250, Gesto::EscolheCor(COR_NOVA)),
            (t + 550, Gesto::FechaSeletor),
        ]);
        t + 900
    } else {
        t + 900
    };
    pincelada(&mut r, t, 0.50, 600);
    let para = t + 1_600;
    r.extend([
        (para, Gesto::Para),
        (para + 500, Gesto::Rewind),
        (para + 1_000, Gesto::Play),
    ]);
    r
}

/// Uma pincelada vertical na coluna `u`, de `v = 0,2` a `0,6` (cruza as duas gravadas), de `t0` (ms).
fn vertical(r: &mut Vec<(u64, Gesto)>, t0: u64, u: f32, dura: u64) {
    let passos = dura / PASSO_MS;
    for i in 0..=passos {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let v = 0.2 + 0.4 * (i as f32 / passos as f32);
        r.push((
            t0 + i * PASSO_MS,
            if i == 0 {
                Gesto::Desce([u, v])
            } else {
                Gesto::Move([u, v])
            },
        ));
    }
    r.push((t0 + dura + PASSO_MS, Gesto::Sobe));
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
        let gestos = if conduz && caminho_desconhecido().is_none() {
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
    if let Some(x) = caminho_desconhecido() {
        let nomes: Vec<&str> = NOMES.iter().map(|(n, _)| *n).collect();
        return format!(
            "PH2D_MACRO_CAMINHO=«{x}» is not a path this scene knows ({}), so the app does NOTHING by \
             itself. Close it and run the command again with one of those names.",
            nomes.join(", ")
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
    match variante() {
        Variante::Cor => {
            return format!(
                "the app does the path by itself — Record, one Digital stroke, a press on C&F dragged a \
                 little to the left of the canvas and released (the colour picker opens), red picked, a \
                 second stroke, Stop, Rewind and Play in the Timeline's Keys tab. Do not touch the mouse \
                 for 12 s. The MACROS panel must list no «Refused» step, and Play must draw both strokes, \
                 the second one red. This log is also written to {}",
                f.display()
            );
        }
        Variante::ColorDrop => {
            return format!(
                "the app does the path by itself — Record, one Digital stroke, blue picked on C&F, then \
                 C&F dragged onto the canvas and released (the fill), Done in «Fill adjust», a second \
                 stroke, Stop, Rewind and Play in the Timeline's Keys tab. Do not touch the mouse for \
                 14 s. The MACROS panel must list no «Refused» step, and Play must redo the blue fill \
                 and then the second stroke. This log is also written to {}",
                f.display()
            );
        }
        Variante::Fim => {
            return format!(
                "the app does the path by itself — Record, two Wet Paint strokes, Stop, then it drags \
                 the Timeline's playhead past the end of the clip and leaves it there. Do not touch the \
                 mouse. At the end of the clip the paint must keep spreading for up to a minute and stop \
                 on the editor's image: «≠editor» in this log reaches 0, and nothing jumps. This log is \
                 also written to {}",
                f.display()
            );
        }
        _ => {}
    }
    let fim = match variante() {
        Variante::Topo => "Stop, then the Play at the top of the window (no Rewind)",
        Variante::SemRewind => "Stop, then Play in the Timeline's Keys tab (no Rewind)",
        _ => "Stop, Rewind and Play in the Timeline's Keys tab",
    };
    // O meio em que a cena pinta (`smoke::arm_brush_once`): Digital só no `arrasta` e no `cruza`.
    let meio = if matches!(variante(), Variante::Arrasta | Variante::Cruza) {
        "Digital"
    } else {
        "Wet Paint"
    };
    let fim = if matches!(variante(), Variante::Cruza | Variante::CruzaMolhada) {
        "Stop, one vertical stroke over both, then Rewind and Play in the Timeline's Keys tab. While it \
         plays the vertical stroke must stay whole and clean — no white gaps and no dark knots where it \
         crosses a recorded stroke that has not appeared yet"
    } else {
        fim
    };
    format!(
        "the app does the path by itself — Record, two {meio} strokes, {fim} ({:?}). Do not touch \
         the mouse for 15 s. Watch the canvas after Play: the top stroke must appear first and the lower \
         one about 1.5 s later. This log is also written to {}",
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
        if variante() == Variante::Fim {
            extra.push_str(&ao_editor(espia, bits, painter));
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

/// ` ≠editor=N` da variante `fim`: os texels que a subida anterior mostrou e diferem da imagem do editor.
fn ao_editor(e: &Espia, bits: u64, painter: Option<&PainterTool>) -> String {
    let troca = e.trocas.iter().find(|t| t.entity_bits == bits);
    let rgba = troca.and_then(|t| e.ranhuras.get(&t.texture_id)).map(|r| &r.1);
    let editor = painter
        .filter(|t| t.bound_document() == Some(bits))
        .and_then(PainterTool::imagem_aplicada)
        .map(|i| i.0);
    match (rgba, editor) {
        (Some(a), Some(b)) if a.len() == b.len() => format!(" ≠editor={}", diferentes(a, &b)),
        _ => " ≠editor=-".to_owned(),
    }
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
#[path = "caminho_tests.rs"]
mod tests;
