//! **O GANHO do relevo** — a sombra do impasto como uma grandeza que não é um pixel.
//!
//! Irmão do [`super::impasto_light`], e um arquivo separado porque a pergunta é outra. Aquele
//! responde *"como estes pixels ficam sob o relevo?"*; este responde *"quanto o relevo escurece ou
//! ilumina aqui?"* — um número, sem cor, que outras coisas podem multiplicar.
//!
//! Existe porque o relevo precisava alcançar a **silhueta** do slot Shape (report do Enio,
//! 2026-08-09), e uma silhueta é COBERTURA: alpha, que o passe de luz por construção nunca escreve.
//! Sem separar o ganho da cor, levar o relevo ao pincel era inexprimível.

use super::Region;
use crate::tool::PainterTool;

/// O albedo neutro que [`PainterTool::relief_shade_gain`] ilumina para ler o ganho do relevo, e o
/// valor que ele devolve onde a superfície é PLANA.
///
/// ⚠️ **A primeira versão deste comentário afirmava que 128 era ARITMÉTICA — e era falso**, o que a
/// mutação `FLAT_PROBE = 127` provou ao sobreviver a tudo: os pesos Rec.601 deste repo somam 256, e
/// por isso **todo** cinza neutro lumina para si mesmo exatamente. A identidade não depende do valor.
///
/// O que o valor compra é **FAIXA nos dois sentidos**, e isso está medido: sobre a crista de teste o
/// passe leva o ganho a `45..206` — ele **escurece o flanco** a 0,35× e **soma especular na crista**
/// a 1,61×. Um albedo alto (250) faria o lado claro pedir 402 e bater no teto de 255, entregando
/// 1,02× onde o relevo brilha; um albedo baixo quantizaria o lado escuro a nada. O meio da faixa é o
/// único ponto que resolve os dois, e o gate `the_gain_resolves_both_sides_of_the_relief` o afirma
/// pela PROPRIEDADE (o ganho tem de sair dos dois lados do plano), nunca pela constante — um oráculo
/// escrito em cima de `FLAT_PROBE` se move junto com ela e não pode falhar.
pub(super) const FLAT_PROBE: u8 = 128;

impl PainterTool {
    /// O **GANHO** que o relevo impõe à aparência, como um campo de `[0,255]` do tamanho da tela onde
    /// [`FLAT_PROBE`] é a resposta plana — ou `None` quando não há relevo nenhum.
    ///
    /// Existe porque o relevo precisa alcançar coisas que **não são pixels**: a silhueta que o slot
    /// Shape captura é COBERTURA, e a cobertura é alpha, que o shade nunca escreve (ele multiplica
    /// COR). Sem uma grandeza separada, "levar o relevo para o pincel" seria inexprimível — e foi
    /// exatamente isso que o report do Enio (2026-08-09) encontrou.
    ///
    /// ⚠️ **A construção não é um segundo modelo de luz: é o passe canônico rodado sobre um albedo
    /// NEUTRO.** É isso que o torna exato no caso que decide tudo — o passe é *relativo* (a resposta
    /// de um pixel é dividida pela de uma superfície plana), então sem relevo ele multiplica por
    /// exatamente 1, o cinza volta ele mesmo, e o ganho é a identidade **ao byte**. Nenhuma silhueta de
    /// documento sem escultura se move.
    ///
    /// ⚠️ **A aproximação, nomeada:** para tinta METÁLICA a cor do próprio pixel é uma ENTRADA do
    /// shade (o realce de um metal *é* a cor dele), então um albedo neutro devolve o brilho de um
    /// metal cinzento. Isto é um ganho de LUMINÂNCIA para uma máscara, não a cor do pixel — e uma
    /// máscara não tem cor a preservar.
    #[must_use]
    pub fn relief_shade_gain(&self) -> Option<Vec<u8>> {
        if !self.impasto_visible() {
            return None;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let n = (w as usize) * (h as usize);
        // Cinza médio com alpha CHEIO: o passe pesa a sombra pela COBERTURA de tinta, e um alpha
        // parcial mediria "quanta tinta há", que é a pergunta da própria silhueta — pesá-la duas
        // vezes deixaria a borda do traço mais escura por ela ser borda, não por ter relevo.
        let mut probe = vec![255u8; n * 4];
        for px in probe.chunks_exact_mut(4) {
            px[0] = FLAT_PROBE;
            px[1] = FLAT_PROBE;
            px[2] = FLAT_PROBE;
        }
        self.apply_impasto_light(&mut probe, Region { x: 0, y: 0, w, h });
        Some(
            probe
                .chunks_exact(4)
                .map(|p| {
                    ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8)
                        as u8
                })
                .collect(),
        )
    }
}
