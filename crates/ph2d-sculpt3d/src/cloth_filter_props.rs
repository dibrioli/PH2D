//! ⭐⭐⭐ **AS PROPRIEDADES DO FILTRO DE TECIDO** — dele, e não do pincel.
//!
//! # ⛔⛔ O defeito que este ficheiro cura, com a pergunta do dono
//!
//! *«cloth filter tem as propriedades do tecido? Já foram implementadas para
//! cloth filter?»* (Enio, 2026-09-08). Medido: **em parte, e por EMPRÉSTIMO**.
//! O filtro lia `brush.cloth_mass`, `brush.cloth_damping` e
//! `brush.cloth_plasticity` — os números do **pincel** — e isso está errado em
//! três sítios de uma vez:
//!
//! | | no pincel (espec §8.1) | no filtro (espec §7) | o que se passava |
//! |---|---|---|---|
//! | amortecimento | omissão `0,01`, faixa `0,01..1` | omissão **`0`**, faixa `0..1` | ⛔ o filtro **nunca conseguia** o valor de omissão dele |
//! | plasticidade | do artista | o alvo **fixa `0`** | passávamos o do pincel, em silêncio |
//! | massa | `1,0` | `1,0` | ✅ |
//!
//! ⚠️ **E os três eram INALCANÇÁVEIS**: a linha do painel pergunta *«o pincel de
//! tecido está na mão?»*, e o filtro corre com **qualquer** verbo (a W9b
//! desacoplou a lei do verbo). Com o Draw na mão eles mexiam na simulação sem
//! nada na tela os mostrar — a espécie *vivo e inalcançável*, que é o espelho do
//! knob morto.
//!
//! ⚠️⚠️ **E a bancada não podia ver nada disto:** ela monta o `Pincel` da lei
//! **directamente** do cabeçalho de cada fixture e nunca passa pelo mapeamento
//! do produto; além disso a plasticidade nasce em `0`, que é exactamente o valor
//! da espec ⇒ *uma fixtura no ponto neutro de um knob não testa esse knob*.
//!
//! # ⭐⭐⭐ E a cura estrutural: o filtro deixou de RECEBER um pincel
//!
//! O [`crate::SculptStroke::cloth_filter_begin`] já não tem um `&Brush` na
//! assinatura. *Não é possível ler por engano um campo que não chega* — e é isso
//! que faz esta lei valer para o próximo campo de tecido que alguém acrescentar
//! ao pincel, sem ninguém se lembrar dela.

/// ⭐⭐⭐ **AS PROPRIEDADES DO FILTRO DE TECIDO.**
///
/// ⚠️ **GLOBAIS, e não por-verbo**, como a [`crate::ClothFilterOrientation`] ao
/// lado e pela mesma razão: o `slots` do painel guarda o pincel de cada
/// ferramenta porque *afinar a força do Smooth não é afinar a do Clay* — e um
/// filtro não pertence a ferramenta nenhuma. Três dos cinco tipos não têm verbo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClothFilterProps {
    /// ***Mass*** — o ganho inverso sobre um `dt` fixo (espec §5.3).
    /// Omissão `1,0`, faixa `0..2` — a do alvo.
    pub mass: f32,
    /// ***Damping*** — a fracção de velocidade PERDIDA por passo (espec §5.3).
    ///
    /// ⚠️ **Omissão `0`, e a faixa começa em `0`** — *o filtro nasce **sem**
    /// perda de velocidade nenhuma* (espec §7). ⛔ O pincel tem outra omissão
    /// (`0,01`) e outra faixa (`0,01..1`), e era a dele que chegava aqui: o
    /// filtro **não conseguia** o próprio valor de omissão.
    pub damping: f32,
    /// ***Plasticity*** `ρ` — a memória de forma (espec §5.2).
    ///
    /// ⚠️⚠️ **DIVERGÊNCIA DECLARADA, e ela ACRESCENTA:** a espec §7 diz que o
    /// alvo cria a simulação do filtro com `ρ = 0` fixo (*«sem memória de
    /// forma»*), logo lá isto **não é um controlo**. Aqui é — e a omissão `0`
    /// mantém o comportamento do alvo byte a byte, que é o que faz a divergência
    /// ser uma adição e não uma quebra de paridade.
    pub plasticity: f32,
    /// ⭐⭐⭐ ***Quality*** — quantas varreduras de relaxação por passo.
    ///
    /// ⚠️⚠️ **O ALVO FIXA ISTO EM `5`** ([`ph2d_cloth::verlet::VARREDURAS`]) e
    /// não o oferece. É o maior controlo que este pano ganha sobre o dele, e o
    /// que ele compra está MEDIDO — o esticão máximo de um aperto sobre uma
    /// grelha `21×21`, doze passos:
    ///
    /// | varreduras | esticão máx | `p95` | ms |
    /// |---:|---:|---:|---:|
    /// | `1` | `1,677` | `0,893` | `0,6` |
    /// | **`5`** (o alvo) | **`0,188`** | `0,100` | `1,9` |
    /// | `8` | `0,066` | `0,051` | `3,0` |
    /// | `16` | `0,050` | `0,038` | `6,3` |
    /// | **`32`** | **`0,039`** | `0,028` | `11,4` |
    /// | `64` | `0,090` | `0,023` | `22,5` |
    ///
    /// ⇒ **de `5` para `32` o pior esticão cai `4,9×`**, que é exactamente o
    /// regime em que o pano rasga.
    ///
    /// ⛔⛔ **E o teto de `32` é MEDIDO, não escolhido:** a `64` o pior caso
    /// **piora** (`0,039 → 0,090`) enquanto o `p95` mal se move — passado o
    /// joelho, o que sobra é tempo. ⚠️ **O recurso é TEMPO e ele cresce com a
    /// MALHA:** na malha do smoke (`98 306` vértices) cada varredura vale
    /// `~8,7 ms`, então `5` já custa `47,5 ms` contra um quadro de `16,7`. *Quem
    /// quiser as 32 reduz a malha primeiro — o botão de retopologia existe.*
    ///
    /// ⚠️ **E ele NÃO é «mais é melhor»:** na Escala o ótimo medido é o `5` do
    /// alvo (`0,058`) e `8..64` pioram (`~0,076`). Por isso o rótulo é
    /// *Quality* e não *Stiffness*, e o default é o do alvo.
    pub sweeps: u32,
    /// ⭐⭐ ***Collisions*** — o pano bate nas OUTRAS peças da cena (espec §5.6).
    ///
    /// ⛔⛔ **Ela existia na espec do filtro e não existia no produto:** *«idem
    /// §5.6, opção nasce desligada»* (§7), e nós passávamos-lhe uma lista
    /// **vazia**. A construção dos colisores era ~40 linhas inline no traço; hoje
    /// é uma porta que os dois partilham
    /// (`stroke_cloth_ref::caixas_de`).
    ///
    /// ⚠️⚠️ **NASCE DESLIGADA, e o preço é a razão** — medido no traço: `2,6×` a
    /// `6,1×` o custo de um dab, e **no filtro a peça inteira é o pior caso**
    /// (não há banda a limitar quem colide). A `16 641` vértices um obstáculo
    /// põe uma pincelada em `16,6 ms` contra um quadro de `16,7`.
    ///
    /// ⛔ **Divergência declarada, a mesma do traço:** o alvo dá ao raio da
    /// colisão uma **espessura** (`0,3`) e o nosso lança raio fino — num colisor
    /// fino visto de raspão o nosso passa. Não existe amostra do alvo com
    /// obstáculo, logo esta parte **não tem lado aprovado**.
    pub collisions: bool,
}

/// A faixa de cada número, na ordem em que o painel os mostra.
impl ClothFilterProps {
    /// Faixa da massa — a do alvo (espec §7).
    pub const MASS: (f32, f32) = (0.0, 2.0);
    /// Faixa do amortecimento — ⚠️ começa em `0`, ao contrário da do pincel.
    pub const DAMPING: (f32, f32) = (0.0, 1.0);
    /// Faixa da plasticidade.
    pub const PLASTICITY: (f32, f32) = (0.0, 1.0);
    /// Faixa das varreduras — ver o doc de [`Self::sweeps`] para o teto medido.
    pub const SWEEPS: (u32, u32) = (1, 32);

    /// ⚠️ **Preso na PORTA**, e não em quem lê: o device não tem opinião, e uma
    /// massa negativa ou zero varreduras seriam uma divisão por zero dentro do
    /// laço. É a mesma lei do `SsaoParams::pack` desta casa.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            mass: self.mass.clamp(Self::MASS.0, Self::MASS.1),
            damping: self.damping.clamp(Self::DAMPING.0, Self::DAMPING.1),
            plasticity: self.plasticity.clamp(Self::PLASTICITY.0, Self::PLASTICITY.1),
            sweeps: self.sweeps.clamp(Self::SWEEPS.0, Self::SWEEPS.1),
            collisions: self.collisions,
        }
    }
}

impl Default for ClothFilterProps {
    /// ⭐⭐ **A omissão é a do ALVO, byte a byte** — é isso que faz os quatro
    /// controlos serem uma ADIÇÃO e não uma quebra de paridade: com eles
    /// intocados, os 17 traços da bancada do filtro dão exactamente o que davam.
    fn default() -> Self {
        Self {
            mass: 1.0,
            damping: 0.0,
            plasticity: 0.0,
            sweeps: ph2d_cloth::verlet::VARREDURAS,
            collisions: false,
        }
    }
}

#[cfg(test)]
#[path = "cloth_filter_props_tests.rs"]
mod tests;
