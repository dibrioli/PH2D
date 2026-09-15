//! §7 — aplicar aos vértices.
//!
//! ```text
//! deslocamento(v) = Σ_i  peso_i(v) · [ X_{i,a(v)}(p₀(v))  −  p₀(v) ]
//! ```
//!
//! com `p₀(v)` a posição do vértice **no início do traço** e `a(v)` o octante de
//! espelho dele. Depois multiplica-se pelo factor da máscara (§9).

use crate::Controlos;
use crate::cadeia::Cadeia;
use crate::mapas::{Mapa, octante};
use crate::vetor::{V3, add, escalar, sub};

/// O que atenua o deslocamento final, por vértice (§9).
///
/// ⚠️ **A máscara escala o DESLOCAMENTO; não muda os pesos nem o pivô** — e está
/// medido: as fixturas com máscara `0,5` e `1` têm o **mesmo pivô** que a sem
/// máscara, e deslocamento máximo `0,192` e `0,108` contra `0,385`.
#[derive(Default, Clone, Copy)]
pub struct Fatores<'a> {
    /// A máscara de escultura: o factor é `1 − máscara(v)`.
    pub mascara: Option<&'a [f32]>,
    /// O produto das auto-máscaras activas.
    pub auto_mascara: Option<&'a [f32]>,
    /// Vértices ocultos: factor `0`.
    pub escondido: Option<&'a [bool]>,
}

impl Fatores<'_> {
    /// **O factor deste vértice** — a resposta ÚNICA a *«quanto deste
    /// deslocamento chega»*.
    ///
    /// ⚠️ Ela é pública porque tem um **segundo** consumidor fora do §7: a
    /// auto-suavização do pincel, que tem de atenuar pelo mesmo factor. *Uma
    /// segunda cópia desta aritmética divergiria no dia em que um canal novo
    /// entrasse no produto* — e a máscara já entrou depois das auto-máscaras.
    #[must_use]
    pub fn de(&self, v: usize) -> f32 {
        if self
            .escondido
            .is_some_and(|e| e.get(v).copied().unwrap_or(false))
        {
            return 0.0;
        }
        let m = self
            .mascara
            .map_or(1.0, |m| 1.0 - m.get(v).copied().unwrap_or(0.0));
        let a = self
            .auto_mascara
            .map_or(1.0, |a| a.get(v).copied().unwrap_or(1.0));
        m * a
    }
}

/// Escreve em `saida` a posição de **cada vértice depois deste evento**.
///
/// ⭐⭐ **Esta assinatura É a lei do §7.2, e por construção.** A lei diz: *depois
/// de `N` eventos, a posição de cada vértice é a posição do **início do traço**
/// mais o deslocamento que o `G` acumulado pede — **nunca** uma soma de
/// passos*. Ao devolver a posição final a partir de `p0`, uma implementação que
/// acumulasse passos deixa de ser **exprimível** nesta API.
///
/// ⛔ **O mecanismo é livre** — rebasear, ou repor a malha ao estado do início e
/// aplicar por cima, dão a **mesma** lei. ⭐ A razão de preferirmos esta forma é
/// **de paridade e é um número**: as duas são iguais em aritmética exacta e
/// diferem no arredondamento de `f32` (repor-e-reaplicar faz uma escrita extra
/// por vértice e por evento), e a barra deste documento vive a `~1e-7` — *na
/// mesma ordem do ruído que essa escrita introduz*.
pub fn posicoes_finais(
    cadeia: &Cadeia,
    ctrl: &Controlos,
    mapas: &[Mapa],
    p0: &[V3],
    fatores: Fatores<'_>,
    saida: &mut Vec<V3>,
) {
    let n_seg = cadeia.segmentos.len();
    saida.clear();
    saida.reserve(p0.len());
    for (v, &p) in p0.iter().enumerate() {
        let a = octante(p, ctrl.simetria);
        let base = a * n_seg;
        let mut deslocamento = [0.0f32; 3];
        for i in 0..n_seg {
            let peso = cadeia.peso(i, v);
            if peso == 0.0 {
                continue;
            }
            let movido = mapas[base + i].aplicar(p);
            deslocamento = add(deslocamento, escalar(sub(movido, p), peso));
        }
        let f = fatores.de(v);
        saida.push(add(p, escalar(deslocamento, f)));
    }
}
