//! **O DESVIO DO `Rough`** — o campo que faz a linha VAGUEAR (plano 38 W6, `rough.js`/Excalidraw).
//!
//! ⚠️ **A frase inteira deste módulo é: isto NÃO é o jitter.** O motor já espalha cada dab
//! independentemente ([`crate::jitter`]), e um desvio pseudo-aleatório POR DAB **seria** aquilo — a
//! segunda porta para a mesma pergunta, que esta casa já removeu duas vezes (o *Random Angle*
//! por-slot de Shape+Grain, o `sews_threads` do enum). O que o `rough.js` faz e o jitter não é
//! **vaguear**: o desvio é COERENTE ao longo do traço, e é por isso que o resultado lê como mão e
//! não como poeira. Logo o campo é **ruído de valor 1-D no ARCO**, de baixa frequência, avaliado na
//! PERPENDICULAR do heading.
//!
//! ⚠️ **DUAS oitavas, porque a referência tem dois knobs e eles são amplitudes de escalas
//! diferentes:** o `roughness` do `rough.js` desloca os pontos de controle (o tremor curto) e o
//! `bowing` desloca o meio do segmento (o arco longo). Uma oitava só colapsaria os dois num slider
//! e tornaria o arqueamento inexprimível.
//!
//! ⚠️ **A SEMENTE É O ARCO, e é isso que torna o tipo compatível com os shape editors.** Eles
//! re-carimbam a figura INTEIRA a cada quadro enquanto o artista a ajusta, então um desvio semeado
//! num contador por-dab faria a figura **FERVER enquanto ele só olha** — a doença que este módulo
//! nomeia desde o sculpt. Sendo [`offset_at`] função **pura** de `(arco, passada, spec)`, o mesmo
//! desenho dá os mesmos dabs **ao bit**.
//!
//! ⚠️ **E ele move a TINTA, nunca o CAMINHO** — a mesma lei do [`super::speed`], pelo mesmo motivo:
//! `last_pos`, o `accum` do espaçamento e o `arc_len` continuam sendo o que a mão fez.
//!
//! Clean-room: o `rough.js` é MIT e o Excalidraw é MIT, mas nada aqui é port de linha nenhuma — o
//! que este módulo sabe deles é **comportamento** (o desvio coerente, as duas amplitudes, o traço
//! duplo), lido da documentação pública.

use super::Stroke;
use crate::BrushSpec;

/// O hash de rede, o `smoothstep` e o `lerp` vêm do vizinho de padrões — eles são helpers ESCALARES
/// genéricos (*hash um ponto de rede*, *interpole suave*), não geometria de textura, e escrever um
/// segundo `smoothstep` nesta crate seria a segunda resposta que este módulo passa o doc inteiro a
/// recusar.
use crate::texture::patterns::math::{hash2, ifloor, lerp, smoothstep};

/// Ruído de valor **1-D** em `[-1, 1]`: interpola suavemente entre hashes de células inteiras.
///
/// ⚠️ **`seed` entra como a SEGUNDA coordenada da rede**, então duas passadas são dois campos
/// independentes sem um segundo gerador — é o mesmo `hash2` que a família de padrões usa, com o eixo
/// que ela deixa livre.
fn noise1(x: f32, seed: i32) -> f32 {
    let i = ifloor(x);
    let t = smoothstep(x - i as f32);
    // `hash2` devolve `[0, 1)`; o campo é centrado para que o desvio seja simétrico e a amplitude
    // signifique *"até tanto para cada lado"* em vez de *"até tanto para um lado só"*.
    let a = hash2(i, seed) * 2.0 - 1.0;
    let b = hash2(i + 1, seed) * 2.0 - 1.0;
    lerp(a, b, t)
}

/// **O desvio LATERAL de uma passada, em pixels** — função PURA de `(arco, passada, spec)`.
///
/// ⚠️ **A pureza é o contrato, não um detalhe de estilo:** é ela que dá a idempotência sob
/// re-carimbo de que os shape editors dependem, e é ela que o gate afirma. Um `&mut self` aqui
/// (para puxar do RNG do traço) seria exactamente o jitter, e a figura ferveria.
///
/// ⚠️ **A passada entra como semente CRUA, e isso foi MEDIDO depois de eu afirmar o contrário.** A
/// primeira versão multiplicava por um primo (`pass * 977`) com o argumento de que sementes vizinhas
/// *"partilham metade dos argumentos do hash"* — a mutação que tirava o primo **sobreviveu**, e a
/// medição diz por quê: a avalanche do `hash2` já descorrelaciona, e sobre 4 000 células a
/// correlação é **−0,0067 com a semente crua contra +0,0163 com o primo** — os dois indistinguíveis
/// de zero, e o primo ligeiramente PIOR. Um número mágico cuja razão declarada é falsa é a segunda
/// resposta à espera de alguém a chamar, então ele saiu. *A mutação sobrevivente acusou a
/// afirmação, não um buraco de gate* — a terceira vez nesta linha, depois da comutatividade do
/// IEEE-754 no warp e do `any` da zona de física.
///
/// ⚠️ **A guarda do topo é DEFESA EM CAMADAS, medida e não gateada:** trocá-la pelo `line_kind` cru
/// **sobrevive** à suíte, porque as duas oitavas já são gateadas em `> 0.0` e no neutro devolvem
/// zero de qualquer maneira. A camada que de facto carrega é a
/// [`BrushSpec::rough_pass_count`] — mutá-la faz o traço emitir **84 dabs contra 42**, que é a
/// duplicação a acontecer sobre amplitude zero. Escrever um gate para a camada redundante seria um
/// gate que não pode falhar pelo motivo que alega (o precedente do CAS do ADR-0145).
#[must_use]
pub fn offset_at(spec: &BrushSpec, arc: f32, pass: u32) -> f32 {
    if !spec.rough_active() {
        return 0.0;
    }
    let d = 2.0 * spec.clamped_radius();
    let seed = pass as i32;
    let short = if spec.rough_amount > 0.0 {
        let w = crate::line_kind::ROUGH_WAVELENGTH_SHORT_D * d;
        noise1(arc / w, seed) * spec.rough_amount_px()
    } else {
        0.0
    };
    let long = if spec.rough_bowing > 0.0 {
        let w = crate::line_kind::ROUGH_WAVELENGTH_LONG_D * d;
        // A oitava longa lê a MESMA rede num eixo deslocado — um campo próprio (outro `seed`) faria
        // as duas amplitudes serem dois ruídos sem relação, e o `rough.js` as tem sobre o mesmo
        // segmento. O deslocamento evita que as duas oitavas se somem em fase na origem.
        noise1(arc / w + 31.0, seed) * spec.rough_bowing_px()
    } else {
        0.0
    };
    short + long
}

impl Stroke {
    /// Onde a tinta desta passada cai — `pos` deslocado na PERPENDICULAR do heading.
    ///
    /// ⚠️ **Perpendicular, nunca radial:** um desvio ao longo do próprio caminho só re-espaçaria os
    /// dabs (e o espaçamento já tem dono), enquanto o que faz uma linha parecer desenhada à mão é
    /// ela sair do lugar **de lado**. É a mesma normal que o Chisel e o aro do bow wave usam.
    ///
    /// ⚠️ **Sem heading não há perpendicular** (o primeiro dab de um traço), e aí a tinta fica onde o
    /// dedo a pôs — o mesmo degenerado honesto que o [`Stroke::throw`] já trata.
    pub(super) fn roughen(&self, pos: [f32; 2], arc: f32, pass: u32) -> [f32; 2] {
        let d = self.heading;
        if d[0] == 0.0 && d[1] == 0.0 {
            return pos;
        }
        let off = offset_at(&self.spec, arc, pass);
        if off == 0.0 {
            return pos;
        }
        // A perpendicular do heading (giro de +90°), que já vem normalizado do `heading::advance`.
        [pos[0] - d[1] * off, pos[1] + d[0] * off]
    }
}
