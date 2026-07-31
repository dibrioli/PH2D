//! **O perfil de largura** de um traço — o que separa um desenho de um diagrama.
//!
//! Hoje um traço tem uma largura só. Um artista quer que ela VARIE ao longo do caminho: a
//! linha que afina na ponta, o traço de nanquim que engrossa na curva, a caligrafia. É o
//! *Power Stroke* do Inkscape e o *Width Tool* do Illustrator.
//!
//! # Duas faces, UMA representação (ADR-0148)
//!
//! O que o documento guarda é a **lista de paradas** ([`WidthStops`]) — `(posição de arco,
//! multiplicador)`. O que o artista escolhe por nome é o **preset** ([`WidthProfile`]): quatro
//! números (`início`, `meio`, `fim`, e onde o meio senta), que é o que a tabela de parâmetros de
//! um efeito já sabe desenhar, e o que cobre 90% dos casos (*afina-no-fim*, *afina-nos-dois*,
//! *engrossa-no-meio*).
//!
//! ⚠️ **O preset CONSTRÓI a lista** ([`WidthProfile::to_stops`]) e nada lê os dois tentando
//! reconciliá-los. Duas representações do mesmo fato divergem em silêncio, e aqui a divergência
//! apareceria como *"o traço mudou de forma quando eu arrastei a alça"*.
//!
//! ⚠️ E a redução é **EXATA, não aproximada**: as paradas interpolam pelo MESMO `smoothstep`, e
//! um preset vira três paradas cujo `at` devolve o número que o `WidthProfile::at` devolvia,
//! bit a bit — inclusive nos dois casos degenerados (meio colado numa ponta). É isso que permitiu
//! trocar a representação do motor sem mover um pixel do que já shipava; há gate.
//!
//! # Multiplicadores, nunca medidas absolutas
//!
//! O artista já escolheu a largura no slider de Width, e o perfil diz o que acontece com **ELA**
//! ao longo do caminho. `1.0` em toda parte é o traço uniforme de sempre — o que torna
//! [`WidthProfile::UNIFORM`] (e a lista vazia) um ponto neutro **de verdade**, e não um valor que
//! por acaso não faz nada.
//!
//! # A interpolação é SUAVE, e sem transcendental
//!
//! Ligar as paradas com retas deixa um vinco em cada uma: a largura tem derivada descontínua e a
//! silhueta ganha uma quina que ninguém desenhou. O `smoothstep` (`u²(3−2u)`) chega em cada
//! ponto de controle com derivada zero, então os trechos se encontram lisos — e é polinomial, o
//! que mantém o HR-5 (nada de `sin`/`exp` em geometria de documento).

use serde::{Deserialize, Serialize};

/// Largura mínima que ainda é tinta. Abaixo disto o traço não tem o que desenhar.
pub const MIN_WIDTH_FACTOR: f64 = 1e-9;

/// Uma parada do perfil: *neste ponto do arco, a largura é ESTE múltiplo da largura do traço*.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidthStop {
    /// Onde, em fração do comprimento de **ARCO** (`0` = começo, `1` = fim).
    ///
    /// Por arco e não por segmento: é a mesma lei que o Zig Zag e o Blend já seguem — duas
    /// formas que se veem iguais têm de se comportar igual, e picar uma aresta em vinte
    /// pedaços não pode mover o ponto mais grosso do traço.
    pub pos: f64,
    /// O multiplicador da largura ali.
    pub mult: f64,
}

/// **O perfil que o documento guarda** — a lista de paradas, ordenada por posição.
///
/// Vazia = o traço uniforme de sempre. É o neutro que permite ao componente ECS existir só
/// quando há de facto um perfil: um documento não acumula relações invisíveis que não desenham
/// nada (a mesma lei do `VecOffset` com `d = 0`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WidthStops {
    stops: Vec<WidthStop>,
}

impl WidthStops {
    /// Constrói a lista **ordenando por posição** e clampando cada posição em `[0,1]`.
    ///
    /// A ordenação mora aqui, e não na cabeça de quem escreve: a alça do Width Tool (W2) vai
    /// inserir paradas onde o artista clicar, e uma lista fora de ordem faria [`Self::at`]
    /// devolver lixo em silêncio — o modo de falha mais caro que existe numa geometria.
    #[must_use]
    pub fn new(mut stops: Vec<WidthStop>) -> Self {
        for s in &mut stops {
            s.pos = s.pos.clamp(0.0, 1.0);
        }
        // `sort_by` estável: paradas na MESMA posição preservam a ordem em que foram escritas, e
        // é dela que sai a redução exata do preset degenerado (o meio colado numa ponta).
        stops.sort_by(|a, b| {
            a.pos
                .partial_cmp(&b.pos)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self { stops }
    }

    /// As paradas, na ordem.
    #[must_use]
    pub fn as_slice(&self) -> &[WidthStop] {
        &self.stops
    }

    /// Não há perfil nenhum — o traço é o uniforme de sempre.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// O multiplicador em `t`, a fração de ARCO percorrida (`0` = começo, `1` = fim).
    ///
    /// Fora de `[0,1]` o valor é o da ponta — um chamador que amostre ligeiramente além da borda
    /// por erro de arredondamento recebe a ponta, não uma extrapolação.
    #[must_use]
    pub fn at(&self, t: f64) -> f64 {
        let n = self.stops.len();
        if n == 0 {
            return 1.0;
        }
        if t <= self.stops[0].pos {
            return self.stops[0].mult;
        }
        // A ÚLTIMA parada com `pos <= t`. A busca é linear de propósito: as listas têm unidades
        // de paradas (o preset tem três) e o `power_stroke` a chama 128 vezes por contorno —
        // uma busca binária aqui seria complexidade paga sem número que a peça.
        let mut i = 0;
        while i + 1 < n && self.stops[i + 1].pos <= t {
            i += 1;
        }
        if i + 1 >= n {
            return self.stops[n - 1].mult;
        }
        let (a, b) = (self.stops[i], self.stops[i + 1]);
        let span = b.pos - a.pos;
        if span <= 0.0 {
            return b.mult;
        }
        a.mult + (b.mult - a.mult) * smoothstep((t - a.pos) / span)
    }

    /// Este perfil é o traço uniforme? Então quem o consome pode tomar o caminho barato — e,
    /// mais importante, um comando pode recusar-se a existir em vez de produzir a mesma coisa
    /// com outro nome, e a shell pode REMOVER o componente em vez de guardar um perfil inerte.
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        self.stops
            .iter()
            .all(|s| (s.mult - 1.0).abs() < MIN_WIDTH_FACTOR)
    }

    /// O maior multiplicador — quanto o traço pode engrossar no pior ponto.
    #[must_use]
    pub fn peak(&self) -> f64 {
        self.stops.iter().fold(1.0, |m, s| m.max(s.mult))
    }
}

/// O perfil como PRESET — os quatro números que o artista arrasta.
///
/// ⚠️ Esta é a **face**, não o que o documento guarda: [`Self::to_stops`] a converte na lista, e
/// é a lista que viaja no save. Ver o cabeçalho do módulo.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidthProfile {
    /// Multiplicador no começo do caminho.
    pub start: f64,
    /// Multiplicador no ponto de controle do meio.
    pub mid: f64,
    /// Multiplicador no fim.
    pub end: f64,
    /// Onde o meio senta, em fração do comprimento de ARCO (`0.5` = na metade).
    pub position: f64,
}

impl Default for WidthProfile {
    fn default() -> Self {
        Self::UNIFORM
    }
}

impl WidthProfile {
    /// O traço uniforme — a largura que o `StrokeSpec` já diz, do começo ao fim.
    pub const UNIFORM: Self = Self {
        start: 1.0,
        mid: 1.0,
        end: 1.0,
        position: 0.5,
    };

    /// O multiplicador em `t`, a fração de ARCO percorrida (`0` = começo, `1` = fim).
    ///
    /// Fora de `[0,1]` o valor é clampado nas pontas — um chamador que amostre ligeiramente
    /// além da borda por erro de arredondamento recebe a ponta, não uma extrapolação.
    #[must_use]
    pub fn at(&self, t: f64) -> f64 {
        // ⚠️ Sem `t.clamp` aqui, de propósito: o [`smoothstep`] já clampa o `u` que ele
        // recebe, e um `t` fora do domínio cai no ramo cuja ponta é a resposta certa de
        // qualquer forma. Um clamp a mais seria uma SEGUNDA defesa da mesma propriedade — e
        // duas defesas com um gate só significam que mutar uma delas não sangra, que foi
        // exatamente o que aconteceu. [[feedback_layered_defenses_need_per_layer_gates]]
        let p = self.position.clamp(0.0, 1.0);
        // Nas bordas degeneradas (meio colado numa ponta) o trecho vazio não existe: a
        // resposta é o outro trecho inteiro, sem divisão por zero.
        let (a, b, u) = if t <= p {
            if p <= 0.0 {
                return self.mid;
            }
            (self.start, self.mid, t / p)
        } else {
            if p >= 1.0 {
                return self.mid;
            }
            (self.mid, self.end, (t - p) / (1.0 - p))
        };
        a + (b - a) * smoothstep(u)
    }

    /// **O preset vira a lista** — a porta única do ADR-0148.
    ///
    /// ⚠️ Os dois casos degenerados têm ramo PRÓPRIO, e não é higiene: com o meio colado numa
    /// ponta o [`Self::at`] responde pelo trecho que sobrou (duas paradas), e emitir três com
    /// posições repetidas daria outra resposta na borda — `at(1.0)` com `position = 1` devolve
    /// `mid`, não `end`. A redução exata é o que torna esta a MESMA função, e não uma segunda
    /// parecida.
    #[must_use]
    pub fn to_stops(&self) -> WidthStops {
        let p = self.position.clamp(0.0, 1.0);
        let stops = if p <= 0.0 {
            vec![stop(0.0, self.mid), stop(1.0, self.end)]
        } else if p >= 1.0 {
            vec![stop(0.0, self.start), stop(1.0, self.mid)]
        } else {
            vec![
                stop(0.0, self.start),
                stop(p, self.mid),
                stop(1.0, self.end),
            ]
        };
        WidthStops::new(stops)
    }

    /// Este perfil é o traço uniforme?
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        (self.start - 1.0).abs() < MIN_WIDTH_FACTOR
            && (self.mid - 1.0).abs() < MIN_WIDTH_FACTOR
            && (self.end - 1.0).abs() < MIN_WIDTH_FACTOR
    }

    /// O maior multiplicador do perfil — quanto o traço pode engrossar no pior ponto.
    #[must_use]
    pub fn peak(&self) -> f64 {
        self.start.max(self.mid).max(self.end)
    }
}

/// **Um perfil com NOME** — uma entrada do catálogo que o painel oferece (W2b).
///
/// A `key` é de **i18n**, não o rótulo: esta crate não sabe em que língua o app está, e um nome
/// literal aqui seria a segunda cópia de um texto que o catálogo de traduções já possui.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidthPreset {
    /// A chave i18n do rótulo (`panel.vector.width.preset.…`).
    pub key: &'static str,
    /// A forma que a entrada carrega.
    pub profile: WidthProfile,
}

/// **O catálogo** — as formas que se escolhem por nome, na ordem em que o painel as pinta.
///
/// É o *Width Profile* do Illustrator reduzido ao que se distingue por PALAVRA: lá a lista é de
/// miniaturas (sete desenhos sem nome), e um nome só serve se descrever a curva. Estas quatro
/// descrevem, e foram **medidas** antes de escritas
/// (`ph2d-vec-boolean/tests/measure_width_presets.rs`, o multiplicador em cinco pontos do arco):
///
/// | perfil  | t=0   | 0.25  | 0.50  | 0.75  | 1.00  | fita (área a 8 px × 100) |
/// |---------|-------|-------|-------|-------|-------|--------------------------|
/// | Uniform | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | — (nada a assar)         |
/// | Taper   | 1.000 | 0.775 | 0.550 | 0.275 | 0.000 | 420.0                    |
/// | Both    | 0.000 | 0.500 | 1.000 | 0.500 | 0.000 | 400.0                    |
/// | Bulge   | 1.000 | 1.400 | 1.800 | 1.400 | 1.000 | 1120.0                   |
///
/// ⚠️ **O `mid` do Taper é a MÉDIA das pontas de propósito** — é o que faz os cinco pontos caírem
/// em passos iguais (0.225), ou seja o que faz a palavra *taper* descrever a curva. Um `mid`
/// arbitrário produziria um degrau no meio do afinamento, e o nome mentiria.
///
/// ⚠️ **A ponta de largura ZERO é o pincel de ponta fina, e foi MEDIDA** (era o único valor da
/// tabela a tocar a borda do domínio): a fita fecha em ponta com área 400-420 e 252-253 âncoras,
/// todas finitas — a mesma ordem do Bulge. Não há caso degenerado a tratar.
///
/// ⚠️ **`Uniform` é o primeiro e não assa nada**: [`WidthStops::is_uniform`] faz o Power Stroke
/// devolver vazio (o comando ali é o *Outline Stroke*), então esta entrada é a **REMOÇÃO** do
/// perfil — a única porta de volta ao traço de largura única, e é por isso que ela existe.
pub const PRESETS: [WidthPreset; 4] = [
    WidthPreset {
        key: "panel.vector.width.preset.uniform",
        profile: WidthProfile::UNIFORM,
    },
    WidthPreset {
        key: "panel.vector.width.preset.taper",
        profile: WidthProfile {
            start: 1.0,
            mid: 0.55,
            end: 0.0,
            position: 0.5,
        },
    },
    WidthPreset {
        key: "panel.vector.width.preset.both",
        profile: WidthProfile {
            start: 0.0,
            mid: 1.0,
            end: 0.0,
            position: 0.5,
        },
    },
    WidthPreset {
        key: "panel.vector.width.preset.bulge",
        profile: WidthProfile {
            start: 1.0,
            mid: 1.8,
            end: 1.0,
            position: 0.5,
        },
    },
];

/// Uma parada, sem cerimónia.
#[must_use]
fn stop(pos: f64, mult: f64) -> WidthStop {
    WidthStop { pos, mult }
}

/// `u²(3−2u)` em `[0,1]`: sobe de 0 a 1 chegando com derivada ZERO nas duas pontas.
#[must_use]
fn smoothstep(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
