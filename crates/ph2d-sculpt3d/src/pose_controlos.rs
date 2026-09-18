//! Os controlos **próprios** do pincel de pose, e a porta que os junta aos
//! partilhados para formar a lei.
//!
//! ⚠️⚠️ **A divisão entre «próprio» e «partilhado» é load-bearing, e é a espec
//! que a fixa:** o raio, a força, a curva de atenuação, a máscara, a simetria e
//! o *Connected Only* **já existem no pincel** e este verbo lê-os como os outros
//! — inventar um segundo raio aqui daria ao artista dois números que fazem a
//! mesma coisa e discordam. O que é próprio são os **seis** que nenhum outro
//! verbo tem.

use crate::Brush;

/// Os seis controlos que só o [`crate::Verb::Pose`] lê.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseControlos {
    /// **QUAL DOS CINCO GESTOS** — a escolha do artista, por ordem do dono
    /// (2026-09-15: *«não devem ser ativados com CTRL mas checando o botão no
    /// painel»*).
    ///
    /// ⚠️⚠️ **Ela substituiu o `modo`, que era um de TRÊS, e a diferença é de
    /// alcance:** no alvo cada modo tem uma segunda metade escondida atrás do
    /// modificador de inversão, e o artista tem de **descobrir** que ela existe.
    /// Aqui as cinco estão à vista. ⛔ A LEI não mudou — quem a lê continua a
    /// receber `(modo, invertido)` pela [`ph2d_pose::Deformacao::modo_e_inversao`],
    /// que é a inversa exacta e tem gate de ida-e-volta.
    ///
    /// ⛔⛔ **E é por isso que o `Brush::invert` deixou de chegar aqui:** com as
    /// cinco alcançáveis, o `Ctrl` seria a **segunda** maneira de dizer a mesma
    /// coisa — e uma que **compõe** com a primeira, logo escolher `Twist` no
    /// painel e carregar `Ctrl` voltaria a `Rotate`. É exactamente o argumento
    /// que o [`crate::Verb::honours_invert`] já escreve para os dois
    /// [`crate::Grip::Turn`].
    pub deformacao: ph2d_pose::Deformacao,
    /// `1..20` — quantos segmentos a cadeia tem. Com mais de um ela dobra como
    /// um braço.
    pub segmentos: u32,
    /// `0..2` — afasta o pivô do cursor, em múltiplos do raio.
    pub desvio_da_origem: f32,
    /// **A LARGURA DA TRANSIÇÃO, em múltiplos do raio do pincel** —
    /// `0..`[`PoseControlos::TRANSICAO_MAX`].
    ///
    /// ⚠️ Ela substituiu o `Weight Smoothing` (um número de iterações de
    /// difusão) em 2026-09-17 — ver [`Self::TRANSICAO_MAX`] para a medição que
    /// a decidiu.
    pub transicao: f32,
    /// Prende a extremidade distante da cadeia.
    pub ancorado: bool,
    /// No modo de escala, escala **sem rodar**.
    pub trava_rotacao: bool,
    /// O arrasto do ponteiro em **pixels de ecrã**, no eixo horizontal, desde o
    /// pen-down.
    ///
    /// ⚠️⚠️ **É o único número deste pincel que vem do ECRÃ, e por isso o único
    /// que mora aqui apesar de não ser um controlo do artista.** O modo de
    /// torção lê pixels (espec §5.2) ⇒ a saída dele depende da **resolução e do
    /// zoom**, e a conta que os produz precisa da câmera, que vive na app. ⛔ Um
    /// `Dab` não carrega nada de ecrã, e alargá-lo tocaria todos os 27 verbos
    /// para servir um.
    ///
    /// ⭐ **E isto é um candidato NOMEADO a superar o alvo:** *o mesmo gesto dá
    /// torções diferentes conforme o zoom*, que é a família de defeitos «o
    /// resultado depende de algo em que o artista não está a pensar». ⛔ Trocá-lo
    /// é um MODO com gate próprio, nunca uma correcção silenciosa — as fixturas
    /// de torção medem esta lei.
    pub arrasto_x_pixels: f32,
}

impl Default for PoseControlos {
    fn default() -> Self {
        let lei = ph2d_pose::Controlos::default();
        PoseControlos {
            // ⚠️ **CONTADA da lei, nunca escrita à mão:** o neutro dela é o par
            // `(modo, invertido)` de omissão, e derivá-lo aqui é o que impede as
            // duas omissões de divergirem.
            deformacao: lei.deformacao(),
            segmentos: lei.segmentos,
            desvio_da_origem: lei.desvio_da_origem,
            transicao: Self::TRANSICAO_DE_FABRICA,
            // ⭐ Os dois defaults vêm da crate da LEI, onde a recomendação está
            // declarada **como nossa** — ⛔ e não como observação do alvo, cuja
            // proveniência era circular (o cabeçalho da fixtura é entrada do
            // harness, não uma leitura do que o artista encontra).
            ancorado: lei.ancorado,
            trava_rotacao: lei.trava_rotacao,
            arrasto_x_pixels: 0.0,
        }
    }
}

impl PoseControlos {
    /// ⭐⭐⭐ **O TECTO DAS SUAVIZAÇÕES, e ele é MEDIDO — report do dono de
    /// 2026-09-17** (*«por que essas reentrâncias com pose? por que não é mais
    /// regular a borda da deformação? … mesmo com Weight Smoothing no máximo
    /// não consigo uma transição mais suave»*, com foto).
    ///
    /// # ⛔⛔ As reentrâncias são FACES VIRADAS DO AVESSO, e vivem TODAS na banda
    ///
    /// A região da pose nasce **binária**: a varredura do §2.2 escreve `1` em
    /// quem alcança e `0` no resto, e o único alisador é a difusão de Jacobi do
    /// §4. Quem tem peso `1` roda inteiro com a cadeia e quem tem `0` fica
    /// parado, logo a banda entre os dois tem de **absorver a rotação toda**.
    /// Estreita de mais para o arrasto, a superfície **dobra sobre si mesma** —
    /// e o entalhe escuro que o artista vê é uma normal invertida.
    ///
    /// ⭐ **Medido pelo caminho do produto** (`sonda_de_onde_vivem_as_viradas`,
    /// esfera de `97 922` vértices, arrasto `0,6`): das faces viradas,
    /// **`203` de `203`, `878` de `878`, `1 336` de `1 336`, `801` de `801` e
    /// `10` de `10`** têm peso estritamente entre `0` e `1`. **Zero** no miolo,
    /// **zero** fora da região. *Não é «a borda está feia»: é a banda a dobrar.*
    ///
    /// # ⛔⛔ E o knob conta ANÉIS DA MALHA, não raios do pincel
    ///
    /// A largura que `N` iterações compram, medida em quatro densidades de
    /// esfera (`1 490` · `6 050` · `24 386` · `97 922` vértices, raio `0,8`,
    /// `sonda_da_banda`):
    ///
    /// | `N` | banda, em **arestas da malha** |
    /// |---|---|
    /// | `4` | `3,84` · `4,06` · `4,29` · `4,17` |
    /// | `25` | `10,81` · `10,04` · `10,05` · `10,07` |
    /// | `100` | — · `23,97` · `20,11` · `20,04` |
    /// | `300` | — · — · `37,46` · `34,81` |
    ///
    /// ⇒ **banda ≈ `2,0 · √N` arestas** (`banda/√N` lê `2,00`–`2,16` nas dez
    /// células em que o núcleo não diluiu). ⚠️ **Duas leituras, e as duas são
    /// resposta ao dono:** a unidade é a **MALHA**, logo o mesmo ponto do slider
    /// dá transição larga numa peça grossa e estreita numa fina — na densidade
    /// de fábrica o tecto antigo comprava `0,285` do raio do pincel e o novo
    /// compra `0,495`; e a **raiz quadrada** quer dizer que *triplicar o número
    /// dá `√3 ≈ 1,73×` de suavidade, nunca `3×`*.
    ///
    /// ⚠️ Os traços da tabela são as células em que a difusão **come o próprio
    /// miolo**: sem condição de fronteira, com banda maior que a região o peso
    /// do núcleo cai (a `1 490` vértices ele vai de `1,0000` a `0,5498` em
    /// `N = 300`) e o que há deixa de ser uma fronteira — é um pincel mais
    /// fraco. Na peça de fábrica o núcleo fica em `1,0000` até `300`.
    ///
    /// # ⭐⭐⭐ E a cura de fundo CHEGOU: a faixa é uma DISTÂNCIA no barro
    ///
    /// O parágrafo que estava aqui dizia que ancorar a faixa no raio *«não é
    /// afordável com esta lei»* — `N ∝ (banda/aresta)²` sobre um custo `O(V·N)`
    /// dá `O(V²)`, e uma faixa de um raio pedia `~1 200` iterações. **Está
    /// certo, e a saída era trocar a LEI**: a [`ph2d_pose::pesos::por_distancia`]
    /// calcula a distância de cada vértice à fronteira do anel **andando pelas
    /// arestas** (Dijkstra, `O(V log V)` **uma vez**) e tira o peso dela. O
    /// preço deixa de depender da largura pedida.
    ///
    /// **A faixa medida pelo caminho do PRODUTO, sobre quatro densidades**
    /// (`1 490` · `6 050` · `24 386` · `97 922` vértices, raio `0,8`,
    /// `sonda_da_banda` — a largura em que a média por concha cai de `0,9` a
    /// `0,1`, em **raios de pincel**):
    ///
    /// | lei | faixa em RAIOS, por densidade |
    /// |---|---|
    /// | difusão `N=4` (o de fábrica antigo) | `0,417` · `0,211` · `0,108` · `0,054` |
    /// | distância `t = 0,6` | `0,342` · `0,310` · `0,304` · `0,302` |
    /// | **distância `t = 1,0`** | **`0,507` · `0,501` · `0,501` · `0,502`** |
    /// | distância `t = 2,0` | `0,994` · `1,018` · `1,007` · `1,015` |
    ///
    /// ⇒ **a difusão parte ao meio cada vez que a malha dobra; a distância fica
    /// constante a `±0,6 %` sobre uma faixa de `8×` de aresta.** Em **arestas**
    /// as duas colunas trocam de lado: a `t = 1` a faixa lê `4,4` · `8,8` ·
    /// `17,6` · `35,3` — *dobra com a densidade, que é o que uma distância faz.*
    ///
    /// # ⭐ Porque o valor de fábrica é `1,0`
    ///
    /// É o **joelho medido pelo caminho do produto**, na esfera de `97 922`
    /// vértices (`sonda_das_viradas`), faces viradas por (arrasto, `t`):
    ///
    /// | arrasto | `0,6` | `0,7` | `0,8` | `0,9` | **`1,0`** | `1,2` |
    /// |---|---|---|---|---|---|---|
    /// | `0,10` | 0 | 0 | 0 | 0 | **0** | 0 |
    /// | `0,20` | 0 | 0 | 0 | 0 | **0** | 0 |
    /// | `0,40` | 398 | 160 | 0 | 0 | **0** | 0 |
    /// | `0,60` | 606 | 380 | 119 | 0 | **0** | 0 |
    /// | `0,90` | 695 | 507 | 257 | 2 | **0** | 0 |
    /// | `1,20` | 715 | 552 | 310 | 19 | **0** | 0 |
    ///
    /// ⇒ `1,0` é a **primeira coluna que lê `0` em toda a linha**, até a um
    /// arrasto de `1,20` — **um raio e meio** de pincel.
    ///
    /// ⛔⛔ **E a primeira medição disto deu `0,6`, sobre OUTRO PROGRAMA.** Ela
    /// correu numa sonda com [`ph2d_pose::Controlos::default()`], cuja lei de
    /// arrasto é a da espec (**projectada no osso**); o produto crava o arrasto
    /// **INTEIRO** por veredito do dono, logo deforma mais e precisa de faixa
    /// mais larga. *A régua é o PRODUTO* — a `0,6` o gate reprovou com `606`
    /// faces viradas, que é o número da tabela.
    /// # ⚠️ O tecto, e de que recurso ele é
    ///
    /// **`2,0`**, e o recurso é o **NÚCLEO**: a faixa é centrada na fronteira do
    /// anel, logo uma larga de mais come o miolo da região. Medido a `97 922`
    /// vértices, o peso do vértice sob o cursor é `1,0000` até `2,0·R` e cai
    /// para **`0,9394`** a `3,0·R` — *acima daqui o pincel deixa de mover
    /// inteiro o que está debaixo do dedo, que é outro produto.*
    ///
    /// # ⚠️ O que custa
    ///
    /// **`2,9`–`3,6 ms`** a `97 922` vértices, **no pen-down e uma vez por
    /// traço** (medido a `load 19`, contra um pen-down sem banda de `12,5`; a
    /// `24 386` vértices custa `1,1`). A difusão no tecto que isto substitui
    /// custava `47,6 ms` para a mesma peça ⇒ **`16×` mais barato**.
    ///
    /// ⛔⛔ **E ele NÃO é plano na largura pedida — esta linha dizia que era, e
    /// deixou de ser em 2026-09-17.** A marcha PÁRA na meia-banda (o resto está
    /// cortado em `0`/`1` por construção), logo o preço segue a ÁREA da faixa:
    /// no tecto (`2,0`) custa **`8,7 ms`**, `3×` o da largura de fábrica.
    /// *A lei por arestas era plana porque varria a malha inteira em qualquer
    /// largura* — a de hoje é mais barata onde o artista vive e mais cara no
    /// extremo, e continua `5,4×` abaixo da difusão que as duas substituíram.
    ///
    /// # ⛔ A DIVERGÊNCIA, declarada
    ///
    /// A lei do alvo é a difusão, e é ela que os `69` traços do oráculo medem —
    /// a [`ph2d_pose::Controlos::banda_do_peso`] nasce em `None` por isso, e a
    /// bancada pede-a **campo a campo**. O produto ship a distância porque ela
    /// ganha em todas as colunas medidas; *o oráculo continua vivo e a medir a
    /// dele*, que é o que separa uma divergência de um desvio.
    pub const TRANSICAO_MAX: f32 = 2.0;

    /// A largura de fábrica da transição — ver [`Self::TRANSICAO_MAX`].
    pub const TRANSICAO_DE_FABRICA: f32 = 1.0;

    /// Junta os próprios aos partilhados e devolve a lei.
    ///
    /// ⚠️ **A força entra LINEARMENTE** neste pincel (espec §1.2) — ⛔ nunca ao
    /// quadrado, que é a curva do modo de referência dos outros verbos. *Este
    /// repo já pagou essa confusão uma vez, num corpus inteiro a força cheia
    /// onde `s`, `s²` e `s⁴` coincidem.*
    #[must_use]
    pub fn lei(&self, brush: &Brush) -> ph2d_pose::Controlos {
        // ⛔⛔ **O par sai da ESCOLHA e não do `Ctrl`** — ver
        // [`Self::deformacao`]. O `brush.invert` **não é lido por este verbo**, e
        // há gate (`o_ctrl_nao_troca_a_deformacao_da_pose`): com as cinco à vista
        // no painel, o modificador seria um segundo caminho que COMPÕE com o
        // primeiro.
        let (modo, invertido) = self.deformacao.modo_e_inversao();
        ph2d_pose::Controlos {
            modo,
            segmentos: self.segmentos,
            desvio_da_origem: self.desvio_da_origem,
            // ⛔ A difusão fica **desligada** neste caminho: quem esbate é a
            // distância, e deixar as duas ligadas aplicaria as duas leis.
            suavizacoes_do_peso: 0,
            banda_do_peso: Some(brush.radius * self.transicao),
            ancorado: self.ancorado,
            trava_rotacao: self.trava_rotacao,
            // ⛔⛔⛔ **CRAVADA no arrasto INTEIRO — veredito do dono,
            // 2026-09-17:** *«Full drag parece ser o único necessário»*. Ele
            // nasceu na véspera como um par de chips, por ordem dele (*«cada
            // modo com opção, com um botão para mudar o modo»*); ele testou-o e
            // a escolha ficou sendo uma só.
            //
            // ⚠️⚠️ **É uma DIVERGÊNCIA DECLARADA da espec**, que manda projectar
            // o arrasto no osso (§5.4/§5.5) — e ela é de PRODUTO: puxar de lado
            // deixa de ser deitado fora. ⛔ **A LEI não mudou e o oráculo
            // continua a medir a da espec:** o `Controlos::default()` da crate
            // da lei nasce em [`ph2d_pose::Arrasto::AoLongoDoOsso`], que é o que
            // as `69` fixturas alimentam. *Quem cravar a outra ali re-baseia o
            // corpus inteiro em silêncio* — há gate nas duas pontas, em
            // [`arrasto_do_produto_tests`].
            lei_do_arrasto: crate::PoseArrasto::Completo,
            raio: brush.radius,
            forca: brush.strength,
            // ⚠️ **Da ESCOLHA, não do `Ctrl`** — o `brush.invert` não entra aqui
            // desde 2026-09-15 (ver [`Self::deformacao`]).
            invertido,
            simetria: [false; 3],
            // ⭐ O *Connected Only* do pincel **é** o «só conectado» da espec:
            // a mesma pergunta (a travessia atravessa peças desligadas?), e o
            // artista já o conhece pelo carimbo. Um segundo controlo com o
            // mesmo sentido seria a dupla fonte de verdade de sempre.
            so_conectado: brush.surface_only,
            distancia_max_entre_pecas: ph2d_pose::Controlos::default().distancia_max_entre_pecas,
        }
    }
}

#[cfg(test)]
mod arrasto_do_produto_tests {
    use super::PoseControlos;

    /// ⭐⭐⭐ **O PRODUTO lê o arrasto INTEIRO, e o ORÁCULO continua na projecção.**
    ///
    /// ⚠️⚠️ **Este gate substitui o `o_pincel_nasce_com_a_projeccao_no_osso`, e
    /// a premissa dele morreu à vista no diff.** Ele afirmava que o pincel
    /// nascia com a lei da espec — verdade de 2026-09-17 de manhã, quando o
    /// arrasto era uma escolha de painel com a projecção de fábrica, e **falsa**
    /// desde o veredito do dono da tarde (*«Full drag parece ser o único
    /// necessário»*). *Um gate cuja premissa muda reescreve-se com a morte
    /// visível; apagá-lo levaria a régua junto.*
    #[test]
    fn o_produto_le_o_arrasto_inteiro_e_o_oraculo_fica_na_projeccao() {
        // (1) — o que SHIPA: a lei que sai da ponte para o motor.
        let lei = PoseControlos::default().lei(&crate::Brush::default());
        assert_eq!(
            lei.lei_do_arrasto,
            ph2d_pose::Arrasto::Completo,
            "o pincel deixou de ler o arrasto inteiro — e' o veredito do dono \
             de 17/09 que esta' a ser desfeito, nao uma afinacao"
        );

        // (2) — e o que o CORPUS mede: a lei nua continua a nascer na projecção.
        // ⛔ Sem esta metade, cravar `Completo` no `Controlos::default()`
        // re-baseava as `69` fixturas do oráculo **em silêncio**.
        assert_eq!(
            ph2d_pose::Controlos::default().lei_do_arrasto,
            ph2d_pose::Arrasto::AoLongoDoOsso,
            "a LEI passou a nascer no arrasto inteiro — as 69 fixturas do \
             oraculo passam a medir outro pincel, e a paridade que elas \
             afirmam deixa de ser sobre a especificacao"
        );

        // (3) — e a divergência **existe**: as duas leis discordam num arrasto
        // que não é ao longo do osso. ⚠️ Sem ela, (1) e (2) seriam compatíveis
        // com um `Completo` que por acaso faz o mesmo — *uma divergência
        // declarada que não se mede é uma nota, não uma decisão*.
        let deslocamento = [0.6, 0.8, 0.0];
        let normal = [1.0, 0.0, 0.0];
        let projectada = ph2d_pose::Arrasto::AoLongoDoOsso.alavanca(deslocamento, normal);
        let inteira = ph2d_pose::Arrasto::Completo.alavanca(deslocamento, normal);
        assert!(
            (inteira - projectada).abs() > 0.3,
            "as duas leis de arrasto leem {projectada} e {inteira} no mesmo \
             gesto transversal — se elas deixaram de discordar, a divergencia \
             declarada em (1) nao descreve nada"
        );
    }
}
