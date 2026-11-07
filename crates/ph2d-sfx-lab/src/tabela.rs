//! ⭐ **A tabela única dos campos** (`04_W3_plano.md`): o que não está aqui não existe em lugar nenhum.
//!
//! `ler` devolve `None` onde o campo não se aplica, e nesse caso ele nem se pinta. Daí sai a regra: nenhum controle morto
//! na tela (o ruído branco não tem Hz; a curva sem Attack, Decay e Close não finge que os tem).
//!
//! **As faixas** são as do motor (`ph2d-sfx-recipe/src/sanitize.rs`, com o recurso de cada uma), em unidade de tela.
//! Medição em `04_W3_plano.md` §6: gate `os_extremos_de_cada_campo_soam_e_sao_finitos`.

use ph2d_a11y::NodeId;
use ph2d_det_math::DetMath;
use ph2d_sfx_recipe::{
    Body, EnvPoint, Envelope, Layer, MAX_HZ, MAX_SECONDS, MAX_SEMITONES, MIN_BODY_HZ, MIN_HZ,
    ModeLaw, Noise, NoiseFilter, Recipe, Recording, SAMPLE_RATE, Source, Tone, Wave,
};

#[path = "tabela_campos.rs"]
mod campos;
#[path = "tabela_opcoes.rs"]
mod opcoes;
pub use campos::CAMPOS;
pub use opcoes::{Grupo, OPCOES, Opcao, opcao};

const MS_MAX: f64 = MAX_SECONDS as f64 * 1000.0;
const HZ: (f64, f64) = (MIN_HZ as f64, MAX_HZ as f64);
const MS_POR_S: f64 = 1000.0;
/// Abaixo disto a camada é silêncio: −60 dB é a ponta de baixo do campo.
const DB_MIN: f64 = -60.0;
/// O ganho do motor vai a 1e6, mas a escuta limita: 24 dB já satura qualquer som e deixa a mão na faixa útil.
const DB_MAX: f64 = 24.0;
/// O corpo não leva mais que isto: 20 Hz até a banda dá 1 080 modos harmônicos (`03_W2_medicao.md`).
const MODOS_MAX: f64 = 1080.0;
const CONTATO_MIN_MS: f64 = 1000.0 / SAMPLE_RATE as f64;
const CEM: f64 = 100.0;

/// De que cartão um campo é.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cartao {
    Som,
    Camada,
    Fonte,
    Forma,
}

/// Onde o valor mora.
#[derive(Clone, Copy)]
pub enum Onde {
    Som {
        ler: fn(&Recipe) -> f64,
        escrever: fn(&mut Recipe, f64),
    },
    Camada {
        ler: fn(&Layer) -> Option<f64>,
        escrever: fn(&mut Layer, f64),
    },
}

#[derive(Clone, Copy)]
pub struct Campo {
    pub id: NodeId,
    /// A chave do rótulo, com a unidade.
    pub chave: &'static str,
    pub cartao: Cartao,
    pub faixa: (f64, f64),
    /// A seta.
    pub passo: f64,
    /// O arrasto, por px.
    pub scrub: f64,
    /// O duplo clique na pega.
    pub fabrica: f64,
    pub onde: Onde,
}

impl Campo {
    /// O valor na unidade da tela; `None` = não se aplica a este estado.
    #[must_use]
    pub fn le(&self, r: &Recipe, l: Option<&Layer>) -> Option<f64> {
        match self.onde {
            Onde::Som { ler, .. } => l.is_none().then(|| ler(r)),
            Onde::Camada { ler, .. } => l.and_then(ler),
        }
    }

    pub fn escreve(&self, r: &mut Recipe, l: Option<ph2d_sfx_recipe::LayerId>, v: f64) {
        let v = v.clamp(self.faixa.0, self.faixa.1); // CLAMP-OK: as faixas da tabela têm lo < hi
        match self.onde {
            Onde::Som { escrever, .. } => escrever(r, v),
            Onde::Camada { escrever, .. } => {
                if let Some(c) = l.and_then(|id| r.layer_mut(id)) {
                    escrever(c, v);
                }
            }
        }
    }
}

#[must_use]
pub fn campo(id: NodeId) -> Option<&'static Campo> {
    CAMPOS.iter().find(|c| c.id == id)
}

/// O valor em f32, com a unidade tirada.
fn ms(v: f64) -> f32 {
    (v / MS_POR_S) as f32
}

fn em_ms(s: f32) -> f64 {
    redondo(f64::from(s) * MS_POR_S, 1000.0)
}

/// Sem o resto do f32: 0,65 × 100 não pode aparecer 65,000001.
fn redondo(v: f64, por: f64) -> f64 {
    (v * por).round() / por
}

fn ruido(l: &Layer) -> Option<&Noise> {
    match &l.source {
        Source::Noise(n) => Some(n),
        _ => None,
    }
}

fn ruido_mut(l: &mut Layer) -> Option<&mut Noise> {
    match &mut l.source {
        Source::Noise(n) => Some(n),
        _ => None,
    }
}

/// O ruído branco não tem filtro: o Hz e o Q dele não mudam o som.
fn filtrado(l: &Layer) -> Option<&Noise> {
    ruido(l).filter(|n| n.filter != NoiseFilter::White)
}

fn tom(l: &Layer) -> Option<&Tone> {
    match &l.source {
        Source::Tone(t) => Some(t),
        _ => None,
    }
}

fn tom_mut(l: &mut Layer) -> Option<&mut Tone> {
    match &mut l.source {
        Source::Tone(t) => Some(t),
        _ => None,
    }
}

fn corpo(l: &Layer) -> Option<&Body> {
    match &l.source {
        Source::Body(b) => Some(b),
        _ => None,
    }
}

fn corpo_mut(l: &mut Layer) -> Option<&mut Body> {
    match &mut l.source {
        Source::Body(b) => Some(b),
        _ => None,
    }
}

fn gravacao(l: &Layer) -> Option<&Recording> {
    match &l.source {
        Source::Recording(r) => Some(r),
        _ => None,
    }
}

fn gravacao_mut(l: &mut Layer) -> Option<&mut Recording> {
    match &mut l.source {
        Source::Recording(r) => Some(r),
        _ => None,
    }
}

fn fm(l: &Layer) -> Option<(f32, f32, f32)> {
    match tom(l)?.wave {
        Wave::Fm {
            ratio,
            index,
            index_end,
        } => Some((ratio, index, index_end)),
        _ => None,
    }
}

fn fm_mut(l: &mut Layer) -> Option<(&mut f32, &mut f32, &mut f32)> {
    match &mut tom_mut(l)?.wave {
        Wave::Fm {
            ratio,
            index,
            index_end,
        } => Some((ratio, index, index_end)),
        _ => None,
    }
}

fn esticada(l: &Layer) -> Option<f32> {
    match corpo(l)?.law {
        ModeLaw::Stretched { stretch } => Some(stretch),
        _ => None,
    }
}

fn db_de(g: f32) -> f64 {
    if g <= 0.0 {
        DB_MIN
    } else {
        redondo((20.0 * f64::from(g).det_log10()).max(DB_MIN), CEM)
    }
}

fn ganho_de(db: f64) -> f32 {
    if db <= DB_MIN {
        0.0
    } else {
        10f64.det_powf(db / 20.0) as f32
    }
}

/// Os três tempos, em s, de uma curva que saiu de [`Envelope::hit`]. Tolera o resto do f32 da soma.
#[must_use]
pub fn hit_de(e: &Envelope) -> Option<(f32, f32, f32)> {
    let [a, b, c, d] = e.points.as_slice() else {
        return None;
    };
    let (ataque, queda, fecho) = (b.t, c.t - b.t, d.t - c.t);
    if a.t != 0.0 || ataque < 0.0 || queda < 0.0 || fecho < 0.0 {
        return None;
    }
    let refeito = Envelope::hit(ataque, queda, fecho);
    let igual = |p: &EnvPoint, q: &EnvPoint| {
        (p.t - q.t).abs() <= TOLERANCIA_S && p.v == q.v && p.exp == q.exp
    };
    let mesma = refeito
        .points
        .iter()
        .zip(&e.points)
        .all(|(p, q)| igual(p, q));
    mesma.then_some((ataque, queda, fecho))
}

/// Menos de 1 amostra a 48 kHz: é resto de soma, não outra curva.
const TOLERANCIA_S: f32 = 1.0e-5;

/// Reescreve um dos três tempos. Fora de uma curva de `hit` não faz nada.
fn hit_escreve(l: &mut Layer, f: impl FnOnce(&mut (f32, f32, f32))) {
    if let Some(mut t) = hit_de(&l.envelope) {
        f(&mut t);
        // A curva inteira cabe em MAX_SECONDS, senão a lei do motor a corta ao abrir e ela deixa de ser um `hit`.
        t.0 = t.0.min(MAX_SECONDS);
        t.1 = t.1.min(MAX_SECONDS - t.0);
        t.2 = t.2.min(MAX_SECONDS - (t.0 + t.1));
        l.envelope = Envelope::hit(t.0, t.1, t.2);
    }
}

/// O número da semente na tela: os 32 bits de baixo, porque o campo é `f64` e a semente tem 64.
const SEMENTE_BAIXA: u64 = 0xFFFF_FFFF;

/// A curva nova do «Use Attack, Decay and Close»: dura o que falta do som a partir do começo da camada.
#[must_use]
pub fn forma_nova(segundos: f32, start: f32) -> Envelope {
    const ATAQUE: f32 = 0.001;
    const FECHO: f32 = 0.01;
    let cabe = (segundos - start - ATAQUE - FECHO).max(ATAQUE);
    Envelope::hit(ATAQUE, cabe, FECHO)
}
