//! §8 — **o peso de cada vértice**, que é o produto de quatro coisas.
//!
//! ```text
//! peso(v) = queda_de_profundidade(anel(v), K)      §8.1
//!         × (1 − máscara(v))                        §8.2
//!         × automáscara(v)                          §8.2
//!         × queda_no_contorno(patrono(v))           §8.3
//! ```
//!
//! Um vértice **sem** entrada de propagação tem peso `0` sempre.

use crate::estrutura::{Estrutura, SEM_ANEL};
use crate::{Curva, QuedaNoContorno};

/// §8.1 — a queda de profundidade: a **curva do pincel**, avaliada com
/// `distância = anel` e `comprimento = K`.
///
/// ⚠️⚠️ **A cláusula `d ≥ L ⇒ 0` vem ANTES da escolha da curva, e isso inclui a
/// curva CONSTANTE.** *Um implementador que ponha o `1,0` da constante antes do
/// corte produz um anel a mais e um degrau na borda da deformação* — e o corpus
/// mede-o: a coluna do anel `K` é `0` nas quatro curvas medidas.
pub fn queda_de_profundidade(anel: u32, alcance: u32, curva: Curva<'_>) -> f32 {
    if anel == SEM_ANEL || alcance == 0 || anel >= alcance {
        return 0.0;
    }
    let p = 1.0 - (anel as f32) / (alcance as f32);
    curva(p)
}

/// §8.3 — a queda **ao longo do contorno**, com o seu sinal.
///
/// `t` é a distância de cadeia do patrono e `raio` o **inicial** — ⚠️ nunca o de
/// propagação (§8.4): o deslocamento da origem alonga a PROPAGAÇÃO e **não** o
/// troço de borda afectado.
///
/// ⚠️⚠️ **A coluna do artista é ISENTA**: todo vértice cujo patrono é a própria
/// âncora salta esta queda por completo (factor `1`, sinal `+1`) **nos quatro
/// modos**. ⇒ sob `RADIUS`, o ponto onde a mão carregou recebe sempre a
/// deformação cheia, mesmo que a curva já estivesse a esmorecer. *Quem a
/// esquecer vê o pico do traço cair para metade do valor medido.*
pub fn queda_no_contorno(modo: QuedaNoContorno, t: f32, raio: f32, curva: Curva<'_>) -> (f32, f32) {
    if raio <= 0.0 {
        return (1.0, 1.0);
    }
    match modo {
        QuedaNoContorno::Constante => (1.0, 1.0),
        QuedaNoContorno::Raio => (avaliar(curva, t, raio), 1.0),
        QuedaNoContorno::Laco => (avaliar(curva, dobrada(t, raio), raio), 1.0),
        QuedaNoContorno::LacoInvertido => {
            let lobulo = (t / raio).floor();
            // ⭐ **«dois SIM, dois NÃO», deslocada de um:** o lóbulo `0` não
            // inverte, os lóbulos `1` e `2` invertem, `3` e `4` não, `5` e `6`
            // invertem… Medido lóbulo a lóbulo no corpus: `+ − − + +`.
            //
            // ⚠️ A conta é `((⌊t/R⌋ + 3) mod 4) < 2`, e ela é mais fácil de
            // verificar do que de adivinhar: para `0,1,2,3,4` dá
            // `3,0,1,2,3 mod 4` ⇒ `não, sim, sim, não, não`.
            let k = lobulo.max(0.0) as i64;
            let inverte = (k + 3).rem_euclid(4) < 2;
            (
                avaliar(curva, dobrada(t, raio), raio),
                if inverte { -1.0 } else { 1.0 },
            )
        }
    }
}

/// A onda TRIANGULAR do laço: `t mod R` quando `⌊t/R⌋` é par, e `R − (t mod R)`
/// quando é ímpar.
fn dobrada(t: f32, raio: f32) -> f32 {
    let lobulo = (t / raio).floor();
    let resto = t - lobulo * raio;
    if (lobulo as i64).rem_euclid(2) == 0 {
        resto
    } else {
        raio - resto
    }
}

/// A curva do pincel avaliada com `distância = t` e `comprimento = raio` — a
/// mesma forma da §8.1, e é por isso que ela é uma função e não duas.
fn avaliar(curva: Curva<'_>, t: f32, raio: f32) -> f32 {
    if t >= raio {
        return 0.0;
    }
    curva(1.0 - t / raio)
}

/// O que atenua o peso por-vértice, e que vem de fora desta lei (§8.2).
#[derive(Clone, Copy, Default)]
pub struct Fatores<'a> {
    /// A máscara do editor — multiplica por `1 − máscara`.
    pub mascara: Option<&'a [f32]>,
    /// ⚠️ **A automáscara tem de ser aplicada AQUI, explicitamente**: este
    /// pincel tem queda **própria**, logo ela não lhe chega pelo caminho comum
    /// dos outros pincéis. Os autores do alvo corrigiram exactamente esta
    /// omissão. ⛔ **Não medida** — o harness do oráculo desliga-a em todas as
    /// corridas; quem a ligar reconfere.
    pub auto_mascara: Option<&'a [f32]>,
    /// Vértices que não se movem.
    pub escondido: Option<&'a [bool]>,
}

impl Fatores<'_> {
    fn de(&self, v: usize) -> f32 {
        if self
            .escondido
            .is_some_and(|e| e.get(v).copied().unwrap_or(false))
        {
            return 0.0;
        }
        let m = self
            .mascara
            .map_or(0.0, |m| m.get(v).copied().unwrap_or(0.0));
        let a = self
            .auto_mascara
            .map_or(1.0, |a| a.get(v).copied().unwrap_or(1.0));
        (1.0 - m) * a
    }
}

/// §8 inteira: o peso **com sinal** de cada vértice, na ordem dos vértices.
pub fn construir(
    e: &Estrutura,
    modo: QuedaNoContorno,
    raio_inicial: f32,
    curva: Curva<'_>,
    fatores: Fatores<'_>,
    saida: &mut Vec<f32>,
) {
    saida.clear();
    saida.resize(e.anel.len(), 0.0);
    for (v, peso) in saida.iter_mut().enumerate() {
        let anel = e.anel[v];
        if anel == SEM_ANEL {
            continue;
        }
        let profundidade = queda_de_profundidade(anel, e.alcance, curva);
        if profundidade == 0.0 {
            continue;
        }
        let patrono = e.patrono[v];
        let (contorno, sinal) = if patrono == e.ancora {
            (1.0, 1.0)
        } else {
            let t = e
                .distancia_de_cadeia
                .get(patrono as usize)
                .copied()
                .unwrap_or(0.0);
            queda_no_contorno(modo, t, raio_inicial, curva)
        };
        *peso = profundidade * fatores.de(v) * contorno * sinal;
    }
}
