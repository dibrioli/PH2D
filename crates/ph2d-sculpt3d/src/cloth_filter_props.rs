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
    /// ⭐⭐⭐ ***Stretch Limit*** — quanto o pano pode esticar antes de TRANCAR,
    /// em razão do comprimento de repouso (`1,10` = `10 %`).
    ///
    /// ⛔⛔ **É o report do dono de 08/09, e ele estava certo:** *«o Cloth não age
    /// como pano real, mas como um elástico que estica indefinidamente»*. Uma
    /// restrição de distância com rigidez `0,6` é uma **mola**, e o desvio de
    /// equilíbrio de uma mola cresce com a carga sem limite — sob gravidade
    /// sustentada com pontos presos, uma esfera de raio `1` chega a
    /// **`18,6×`** o comprimento de repouso numa aresta.
    ///
    /// ⚠️ **O alvo não tem este número**, e por isso [`Solver::estica_max`] nasce
    /// em `∞`: as `86` fixtures do pincel e as `17` do filtro correm sobre a
    /// omissão da LEI e não se mexem. O que muda é a omissão do **PRODUTO**, e é
    /// uma **divergência declarada**.
    pub stretch_max: f32,
    /// ⭐⭐⭐ ***Bend Stiffness*** — o tamanho das rugas.
    ///
    /// ⛔⛔ **É o report de 2026-09-09:** *«nunca consigo uma configuração onde o
    /// pano passa a ter ondulação maiores como se fosse um pano duro ou um
    /// couro. Sempre as ondulações são finas»*. E ele estava certo por
    /// construção: **não havia modelo de dobra nenhum** — a única coisa que
    /// resistia a uma prega era a rede de restrições de distância, cujo alcance é
    /// UMA aresta, logo o comprimento de onda da flambagem era o tamanho do
    /// triângulo. *Nenhuma combinação dos outros números podia mudar isso, porque
    /// nenhum deles fala de curvatura.*
    ///
    /// ⚠️ `0` = o pano de sempre (a lei do alvo, que também não tem dobra) e é a
    /// omissão; subir engrossa a onda.
    pub bend: f32,
    /// ⭐⭐⭐ ***Preserve Volume*** — quanto do volume de repouso a peça mantém
    /// (`0` desliga, `1` = todo).
    ///
    /// ⛔ **Só tem sentido numa peça FECHADA**, e o filtro só a liga quando a
    /// malha o é ([`ph2d_mesh::Mesh::is_closed`]): o volume com sinal de uma
    /// superfície aberta não é o volume de nada.
    pub volume: f32,
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
    /// Faixa do tecto de esticão — `1,00` (não estica nada) a `2,00` (o dobro).
    ///
    /// ⛔⛔ **Não há «desligado» no PRODUTO, e a 1.ª redacção tinha-o.** Ela punha
    /// o topo em `10,0` e fazia esse valor virar `∞`, *«para o artista alcançar o
    /// comportamento antigo sem um segundo controlo»* — duas coisas erradas: o
    /// comportamento antigo é o **defeito que o dono reportou** (ninguém o quer),
    /// e uma faixa de `1` a `10` põe todo o intervalo útil (`1,00`–`1,50`) nos
    /// primeiros `5 %` do cursor. ⇒ *um sentinela escondido no topo de uma faixa
    /// que ninguém consegue usar não é um controlo, é uma armadilha.*
    ///
    /// ⚠️ **O `∞` continua a existir na LEI** ([`ph2d_cloth::verlet::Solver`]), e
    /// é ele que as `103` fixtures do oráculo correm.
    pub const STRETCH: (f32, f32) = (1.0, 2.0);
    /// Faixa da conservação de volume. `0` = desligada.
    pub const VOLUME: (f32, f32) = (0.0, 1.0);
    /// Faixa da rigidez de dobra. `0` = sem modelo de dobra.
    pub const BEND: (f32, f32) = (0.0, 1.0);

    /// ⚠️ **Preso na PORTA**, e não em quem lê: o device não tem opinião, e uma
    /// massa negativa ou zero varreduras seriam uma divisão por zero dentro do
    /// laço. É a mesma lei do `SsaoParams::pack` desta casa.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            mass: self.mass.clamp(Self::MASS.0, Self::MASS.1),
            damping: self.damping.clamp(Self::DAMPING.0, Self::DAMPING.1),
            plasticity: self
                .plasticity
                .clamp(Self::PLASTICITY.0, Self::PLASTICITY.1),
            sweeps: self.sweeps.clamp(Self::SWEEPS.0, Self::SWEEPS.1),
            collisions: self.collisions,
            stretch_max: self.stretch_max.clamp(Self::STRETCH.0, Self::STRETCH.1),
            volume: self.volume.clamp(Self::VOLUME.0, Self::VOLUME.1),
            bend: self.bend.clamp(Self::BEND.0, Self::BEND.1),
        }
    }

    /// ⭐ **A porta que traduz os números do artista nos da LEI** — e o único
    /// sítio onde o topo da faixa vira `∞`.
    ///
    /// ⚠️ **A tradução mora aqui e não em quem constrói o [`Pincel`]** pelo mesmo
    /// motivo do `clamped`: escrita nos dois sítios, esconde de qual dos dois o
    /// número saiu.
    #[must_use]
    pub fn solver(self) -> ph2d_cloth::verlet::Solver {
        let p = self.clamped();
        ph2d_cloth::verlet::Solver {
            massa: f64::from(p.mass),
            amortecimento: f64::from(p.damping),
            plasticidade: f64::from(p.plasticity),
            varreduras: p.sweeps,
            estica_max: f64::from(p.stretch_max),
            volume: f64::from(p.volume),
            dobra: f64::from(p.bend),
            passagens_limite: ph2d_cloth::verlet::PASSAGENS_LIMITE,
        }
    }
}

impl Default for ClothFilterProps {
    /// ⭐⭐ **Os CINCO números do alvo nascem na omissão dele, byte a byte** — é
    /// isso que faz aqueles controlos serem uma ADIÇÃO e não uma quebra de
    /// paridade.
    ///
    /// ⛔⛔ **E os DOIS de 2026-09-08 NÃO nascem, porque o alvo não os tem.** O
    /// `stretch_max` nasce em `1,10`, que é uma **divergência declarada de
    /// produto**: sem ele o pano é o elástico que o dono reportou (esticão máximo
    /// `18,6×` numa esfera com três pontos presos).
    ///
    /// ⚠️⚠️ **A paridade não se perde, e a razão é onde ela vive:** as `86`
    /// fixtures do pincel e as `17` do filtro montam o
    /// [`ph2d_cloth::verlet::Solver`] **directamente**, e a omissão DELE continua
    /// neutra (`estica_max = ∞`, `volume = 0`). *A lei fica a ser a do alvo; o
    /// produto é que escolhe outra.*
    fn default() -> Self {
        Self {
            mass: 1.0,
            damping: 0.0,
            plasticity: 0.0,
            sweeps: ph2d_cloth::verlet::VARREDURAS,
            collisions: false,
            // ⚠️⚠️ **AQUI a omissão DEIXA de ser a do alvo, e é deliberado.** O
            // alvo não tem estes três números; nós temos, e o report de 08/09
            // mediu que sem eles o pano rasga. A paridade não se perde: ela vive
            // na omissão de [`ph2d_cloth::verlet::Solver`], que continua neutra, e
            // é ela que as 103 fixtures correm.
            stretch_max: 1.10,
            // ⚠️ A dobra nasce em `0` — o alvo não a tem, e é ela que decide o
            // TAMANHO da ruga, não se ela existe.
            bend: 0.0,
            // ⚠️ O volume nasce DESLIGADO — o dono pediu *«a possibilidade de
            // manter volume»*, que é uma opção, não uma lei.
            volume: 0.0,
        }
    }
}

#[cfg(test)]
#[path = "cloth_filter_props_tests.rs"]
mod tests;
