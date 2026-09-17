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
    /// `0..`[`PoseControlos::SUAVIZACOES_MAX`].
    pub suavizacoes_do_peso: u32,
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
            suavizacoes_do_peso: lei.suavizacoes_do_peso,
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
    /// # ⭐⭐⭐ Porque o tecto é `300`, e não «três vezes o que era»
    ///
    /// Faces viradas por (arrasto, `N`), pelo caminho do produto, na esfera de
    /// `97 922` vértices — a densidade da peça de fábrica, que tem `98 306`
    /// (`sonda_das_viradas`):
    ///
    /// | arrasto | `N=0` | `4` (fábrica) | `25` | `100` (tecto antigo) | `200` | **`300`** | `600` |
    /// |---|---|---|---|---|---|---|---|
    /// | `0,10` | 193 | 498 | 89 | **0** | 0 | **0** | 0 |
    /// | `0,20` | 200 | 724 | 842 | **0** | 0 | **0** | 0 |
    /// | `0,40` | 201 | 832 | 1 222 | 537 | **0** | **0** | 0 |
    /// | `0,60` | 203 | 878 | 1 336 | 801 | 10 | **0** | 0 |
    /// | `0,90` | 203 | 896 | 1 385 | 863 | 56 | **0** | 0 |
    /// | `1,20` | 203 | 931 | 1 386 | 820 | 107 | **0** | 0 |
    ///
    /// ⭐ **`300` é a primeira coluna que lê `0` em TODA a linha** — até a um
    /// arrasto de `1,20`, que é **um raio e meio** de pincel. O tecto antigo
    /// deixava `801` faces viradas no arrasto que o dono fotografou, que é o
    /// report à letra.
    ///
    /// ⛔ **E acima de `300` não há regime novo:** a `600` as viradas já eram
    /// zero, e o que se compra é só banda mais larga por amplitude perdida
    /// (`−10 %` de `100` para `300`, `−22 %` até `900`) e relógio linear.
    /// *O tecto é de PRODUTO — é onde o defeito que ele existe para curar
    /// desaparece —, e o recurso está na linha seguinte.*
    ///
    /// # ⚠️ O que ele custa, e onde
    ///
    /// `0,133 ms` por iteração por `98 k` vértices, **no pen-down e uma vez por
    /// traço** (nunca por dab): medido em `--release`, `12,83 ms` no tecto
    /// antigo e **`39,85 ms`** no novo — `2,4` quadros. ⚠️ É `O(V·N)` **por
    /// segmento**, logo numa peça de `1,5 M` vértices o topo do slider custa da
    /// ordem do meio segundo. *O mesmo já era verdade no tecto antigo, em ponto
    /// mais baixo.*
    ///
    /// ⏳ **ABERTO e nomeado:** que o knob conte anéis é o defeito de fundo, e a
    /// cura seria a banda medir-se em **raios de pincel** com a contagem
    /// derivada da densidade — a mesma forma do `Detail` do `Density`, que
    /// passou a pedir uma CONTAGEM ancorada na área. ⛔ **Ela não é afordável
    /// com esta lei**: `N ∝ (banda/aresta)²` e o custo é `O(V·N)` ⇒ `O(V²)` a
    /// banda constante, e na peça de fábrica uma banda de **um** raio pede
    /// `~1 200` iterações. *A cura de fundo é outra lei de peso — e essa é
    /// decisão do dono, porque o corpus do oráculo mede esta.*
    pub const SUAVIZACOES_MAX: u32 = 300;

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
            suavizacoes_do_peso: self.suavizacoes_do_peso,
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
