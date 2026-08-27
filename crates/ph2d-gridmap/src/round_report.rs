//! ⭐ **O RELATÓRIO DO ARREDONDAMENTO** — o que o G5 mediu de si próprio.
//!
//! ⚠️ **Ele vive num irmão e não no [`crate::round`] por causa do tecto de LOC**, e o
//! corte é por RESPONSABILIDADE: aqui está a superfície de **medição**, lá o algoritmo.
//! *Cada coluna deste relatório existe porque uma pergunta se perdeu sem ela* — os
//! doc-comments são o registo dessas perguntas, e é de propósito que são longos.

use crate::solve::SolveReport;

/// O que o arredondamento mediu de si próprio.
#[derive(Debug, Clone, Default)]
pub struct RoundReport {
    /// Quantas componentes inteiras foram pregadas.
    pub pinned: usize,
    /// ⭐⭐⭐ **Quantas fecharam no degrau 1** (Gauss–Seidel local). *É a régua que
    /// separa a escada adaptativa de um re-solve disfarçado.*
    pub level1: usize,
    /// Quantas precisaram do degrau 2 (varreduras globais).
    ///
    /// ⏳ **O degrau 3 (factorização directa) não está construído**, e a razão é esta
    /// coluna: enquanto ela for zero, ele não teria consumidor nenhum e nada o
    /// mediria. *Construir o degrau caro antes de o barato falhar é construir o que
    /// nenhuma medição pede.*
    pub level2: usize,
    /// Visitas de vértice gastas no degrau 1, somadas.
    pub visits: usize,
    /// ⭐ O maior `|x − round(x)|` que foi preciso pagar, em células.
    pub worst_step: f32,
    /// A soma dos passos pagos — o custo total do arredondamento.
    pub sum_step: f32,
    /// Costuras na árvore de calibre (translação levada a zero, de graça).
    pub tree_seams: usize,
    /// ⭐ Costuras que **fecham ciclo** — as que carregam inteiros de verdade.
    pub cycle_seams: usize,
    /// ⭐ Vértices singulares cuja imagem foi pregada num ponto inteiro.
    pub singular_pinned: usize,
    /// ⛔⛔ **Cópias que o transporte RECUSOU mover** por estarem a mais de meia
    /// célula do valor transportado.
    ///
    /// ⚠️ **Cada uma nomeia uma costura cuja translação era ambígua** — ela ficou a
    /// meio caminho entre dois inteiros, e o arredondamento teve de escolher um lado.
    /// É o sintoma de que o solver contínuo ainda não fecha aquela costura, e **não**
    /// um defeito do arredondamento: ver o resíduo de costura em
    /// [`Self::seam_before`].
    pub ambiguous_seams: usize,
    /// ⭐ **Cópias** de vértices singulares levadas ao valor transportado.
    ///
    /// ⚠️ **Pregar UMA cópia não chega, e a medição di-lo:** na esfera fina saíam
    /// `6` nós de vértice para `8` singularidades pregadas, porque a extracção lê a
    /// imagem na carta do canto que encontrar primeiro — e as outras cópias ficavam
    /// onde a costura mole as tinha deixado, a um resto de inteiro.
    pub singular_copies: usize,
    /// ⛔⛔⛔ **Vértices singulares que a malha CORTADA não contém** — pedidos e não
    /// achados, logo **não pregados**.
    ///
    /// ⚠️ **A ausência deste contador escondia a causa dos furos.** O relatório dizia
    /// *«17 singularidades»* e o campo tinha plantado **25**: os oito que faltavam não
    /// eram uma escolha, era o `cut.origin` não os ter — e um vértice singular sem imagem
    /// inteira deixa as transições à volta dele **fraccionárias**, que o extractor depois
    /// arredonda para células inteiras. *Uma mentira de meia célula que passa em todos os
    /// gates de integralidade e manda o traçado dois triângulos ao lado.*
    ///
    /// ⭐ Medido em 2026-08-25 no corpus: as peças com `0` aqui têm **zero** órfãs e zero
    /// arestas de bordo; a peça do artista tem `8` aqui, `11` órfãs e `14` de bordo.
    pub singular_absent: usize,
    /// ⭐⭐⭐ **Vértices singulares SOLTOS que passaram a ser pregados** — os que o corte
    /// não duplicou, logo sem fecho a representá-los. Ver [`RoundOptions::pin_lone_singularities`].
    pub singular_loose_pinned: usize,
    /// ⭐⭐ Triângulos virados depois da 1.ª resolução do contínuo.
    pub folded_before: usize,
    /// ⭐⭐⭐ Triângulos virados no fim do contínuo — ver
    /// [`ph2d_gridmap::STIFFEN_PASSES`].
    pub folded_after: usize,
    /// Quantas passagens de endurecimento local correram.
    pub stiffen_passes: usize,
    /// ⭐⭐ Triângulos virados **antes** de a escada gulosa pregar o primeiro inteiro.
    pub folded_before_rounding: usize,
    /// ⭐⭐⭐ Triângulos virados **depois** de a escada acabar.
    pub folded_after_rounding: usize,
    /// ⭐⭐⭐ **Quantos PREGOS aumentaram a contagem de dobras.**
    ///
    /// ⚠️ *É a coluna que separa «um punhado de pregos maus» de «o custo espalhado de
    /// todos» — e as duas pedem curas diferentes.*
    pub pins_that_folded: usize,
    /// ⭐ Quantas vezes a 2.ª tentativa (o inteiro do outro lado) correu.
    pub second_tries: usize,
    /// ⭐⭐⭐ Quantas vezes ela **ganhou** — dobrou menos que o inteiro mais próximo.
    pub second_tries_won: usize,
    /// ⚠️ **O CASO DE CANTO, e ele tem nome:** as singularidades esgotaram-se e ainda
    /// sobravam costuras por arredondar — acontece em peças com **alça**, e o nosso
    /// corpus tem um toro. ⛔ Terminar ali deixaria o mapa *quase* inteiro, que é pior
    /// que contínuo.
    pub switched_to_seams: bool,
    /// ⛔ **A pior distância a inteiro DEPOIS**. Tem de ser exactamente `0` no caminho
    /// penalizado.
    ///
    /// ⚠️ **No caminho soldado ela é medida ANTES do encaixe final** e é o resíduo de
    /// `f32` da substituição — a barra ali é derivada, não `0` exacto. *Ler as duas
    /// como a mesma grandeza faria uma delas mentir.*
    pub shift_frac_max: f32,
    /// O resíduo de costura antes e depois — o **preço** do arredondamento.
    pub seam_before: (f32, f32),
    /// O resíduo de costura depois, `(p50, max)`.
    pub seam_after: (f32, f32),
    /// As réguas do mapa final.
    pub solve: SolveReport,
    /// ⭐ A estrutura da soldadura — **só o caminho soldado a preenche**.
    pub weld: crate::weld::WeldReport,
    /// ⭐⭐⭐ Grupos de escalares **amarrados pelos arcos** que entraram de facto.
    ///
    /// ⚠️ *Sem esta coluna, «a wave não mudou nada» e «a wave não correu» leem igual* —
    /// e a 1.ª versão dela custou duas corridas a distinguir.
    pub tie_groups: usize,
    /// ⛔⛔ Grupos recusados por algum membro já ter dono.
    pub tie_refused: usize,
    /// ⭐ A razão: `[dependente, livre do sistema, pregada]`.
    pub tie_refused_why: [usize; 3],
    /// ⭐ `H / H_fingida` das amarras — ver [`crate::weld_solve_driver::WeldSolveReport::tie_gain_p50`].
    pub tie_gain: (f32, f32),
    /// ⭐⭐ Grupos com RAIZ de classe simples.
    pub tie_plain_roots: usize,
    /// ⛔⛔⛔ Eixos de incógnitas livres que a escada SALTOU por estarem congelados por
    /// uma amarra — ou seja, escalares que nunca chegam a ser inteiros.
    pub tie_axes_skipped: usize,
    /// ⭐ A ronda em que o contínuo estourou (`0` = nunca), e o movimento dela.
    pub nonfinite_round: (usize, f32),
    /// ⭐⭐ Qual escritor estourou — ver `WeldSolveReport::nonfinite_who`.
    pub nonfinite_who: usize,
    /// ⛔ Coordenadas não-finitas no fim do contínuo, e logo após a 1.ª ronda.
    pub nonfinite: (usize, usize),
    /// ⛔⛔⛔ Pregos da escada cujo passo saiu **não-finito** — o valor já estava
    /// `NaN` quando o prego chegou.
    pub nonfinite_pins: usize,
    /// ⭐⭐⭐ Equações de CICLO de arco que entraram — o A3.
    pub arc_cycles: usize,
    /// ⭐ O resíduo da costura separado por espécie — **só o caminho soldado**.
    ///
    /// ⚠️ Ele responde ao que [`Self::seam_after`] não distingue: aquele mistura as
    /// ligações eliminadas (onde o resíduo é o chão da representação) com as que fecham
    /// ciclo (onde é um facto do mapa). *Uma coluna que soma as duas lê-se como se a
    /// eliminação não tivesse acontecido.*
    pub seam: crate::weld::SeamResidual,
}
