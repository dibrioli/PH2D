# SPEC — as RESTRIÇÕES LINEARES entram por ELIMINAÇÃO (a costura e as linhas de feição)

```
Alvo funcional: fechar a casca e encostar a grade aos vincos · Degrau: T2-por-papers
Alvo NOMEADO · Licença: ⛔ NENHUM fonte é insumo desta espec. O insumo é a literatura
  pública do mapa de leitura, mais a MEDIÇÃO do nosso próprio código. As implementações
  existentes ficam FORA da árvore e servem só de ORÁCULO (ADR-0164), e o §3.I conta
  porte/fork "em qualquer linguagem e sob qualquer licença" como código do alvo.
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md
Papel E desta espec: a janela 49c94a84-e903-48a9-bd7f-b14685d71061 (2026-08-24), que foi o
  R-pós da obra anterior. ⚠️ Ela É contaminada — e é por isso que pode ser E (§3.E:
  "contaminado por definição, e tudo bem"). ⛔ Ela NÃO pode ser I nesta obra.
Patente (§8.1): o checkpoint do LEDGER cobre este caminho (campo cruzado, mapa de grade
  inteira, extracção) e não achou patente viva que o alcance. ⚠️ O alinhamento a feições
  cai dentro do mesmo caminho e do mesmo checkpoint; ⛔ se a obra crescer para
  RECONSTRUÇÃO de feições (algo que esta espec NÃO pede), refaça a busca.
Denylist de URLs (⛔ NÃO abrir): qualquer hospedagem de código, issue tracker, PR ou
  code-search de `libQEx`, `CoMISo`, `vcglib`, `xfield_tracer`, `quadretopology`,
  `quadwild`, `quadwild-bimdf`, `blossom5`, `Directional`, `libigl`.
Denylist de CAMINHOS (⛔ as duas implementações estão NESTE disco): `~/Referencias/**` ·
  `ph2d-quadbench/oracle/**` (⚠️ irmão de `ph2d-quadbench/corpus/`, que é NOSSO e lícito).
  ⇒ O Passo 0 do BLOCO-I transforma isto em permissão do harness.
  ⚠️⚠️ E acrescente um `deny` de **Bash**, não só de `Read`: medido em 2026-08-24, a janela
  I anterior fez 177 chamadas `Bash` e ZERO `Read` — um `deny` só de `Read` não é parede
  nenhuma sob um agente que trabalha por shell (LEDGER, "Papel I").
Mapa de leitura da literatura (⭐ os dois são PÚBLICOS e lícitos a TODOS os papéis):
  · Bommes, Zimmer, Kobbelt — "Mixed-Integer Quadrangulation", SIGGRAPH 2009.
    §2 (o solver guloso, e a lei da eliminação no fim dele) · §3 (direcções salientes:
    a detecção de feição) · §5 + §5.2 (as condições de compatibilidade da costura, e o
    alinhamento a feições e a bordo). Cópia local: `~/Literatura/papers/miq2009.pdf|.txt`
  · Ebke, Bommes, Campen, Kobbelt — "QEx: Robust Quad Mesh Extraction", SIGGRAPH Asia
    2013. Contexto da extracção. Cópia local: `~/Literatura/papers/qex2013.pdf|.txt`
  ⛔ Apêndice com listing compilável de autores do alvo: **PULE** (é código do alvo).
  ⚠️ ⛔ NÃO os procure em `~/Referencias/papers/` — essa árvore está NEGADA ao I inteira,
  de propósito. `~/Literatura/` existe exactamente porque uma fonte lícita guardada dentro
  de uma pasta negada é uma fonte inalcançável (R-pré, 2026-08-24).
Filtragem §4.3: executada pelo E em 2026-08-24 · Sweep: verde em 2026-08-24 (E) e
  RE-CORRIDO verde em 2026-08-24 pelo R-pré sobre uma vassoura ALARGADA de 21 para 56
  entradas — a de 21 não continha um único identificador da implementação que o E leu.
Auditoria §4.2 (R-pré): ⭐ **VERDE — auditada contra §4.2 por R (janela
  6ce7cd70-b800-48d7-91c7-b18f17bc7bc1) em 2026-08-24.** Nenhum texto de código, nome
  interno, comentário do alvo, tabela verbatim, organização transcrita nem pseudo-código
  espelhado. A contra-medida que o E declarou — espec sem receita de montagem — foi
  CONFERIDA e sustenta-se: §1 e §2.3 não nomeiam estrutura, decomposição, factorização,
  permutação nem ordem de eliminação, e a lei que eles afirmam está PUBLICADA no *paper*
  de 2009 (fim do §2 dele).
⛔⛔ DUAS EMENDAS DEVIDAS PELO E — leia-as antes de construir (R-pré, 2026-08-24):
  1. §5, gate nº1: a barra escrita (`3,5e-15`) é MENOR que o valor medido nos próprios
     mapas de referência de que ela diz descender (`3,553e-15`, nas duas peças, pelo
     verificador de `fixtures/`) ⇒ como está, ela REPROVA a referência. E o §1 promete
     "zero, não uma tolerância" enquanto o gate é uma tolerância: falta dizer que o
     resíduo que sobra é o erro de AVALIAÇÃO da própria substituição, não uma folga.
     ⇒ **não fixe a barra por conta própria; devolva a pergunta pelo Enio.**
  2. §3.1 (a detecção de feição, OBRA B): a espec manda medir "os quatro coeficientes" e
     descreve o papel de três. O quarto — a MEIA-LARGURA da janela de estabilidade à
     volta de cada raio — desapareceu, e com ele a condição de que os dois limiares
     valham em TODA a janela; a espec lê-se como se o desvio de direcção fosse medido
     sobre a faixa inteira, que é outra regra. ⇒ **a OBRA B fica à espera da emenda.**
     ⭐ A OBRA A (a costura) NÃO depende disto e é a primeira pelo §6 da própria espec.
"Este documento descreve comportamento; não contém expressão do alvo."
```

> ⛔⛔ **AVISO AO R-PRÉ, e ele é o item nº1 da auditoria desta espec.** A janela que a
> escreveu **leu**, em 2026-08-24, o laço de arredondamento **e a montagem de restrições**
> de uma implementação de referência (registado no [`LEDGER §R-pós.3`](LEDGER_quadwild.md)).
> ⇒ **O risco desta espec não é o de sempre; é convergência de expressão vinda do E.**
> Ela foi escrita de propósito **sem receita de montagem** — diz o que tem de ser
> **verdade** e qual é a **lei publicada**, e ⛔ **não** diz por que estrutura de dados,
> por que decomposição nem em que ordem se monta o sistema. *Se em algum ponto esta espec
> parecer um roteiro de implementação em vez de um requisito, é aí que o R-pré tem de
> morder.*

---

## §0 — Por que esta espec existe, com os números

⭐ **O botão `Quad Retopology` no caminho novo já entrega quads ao nível da referência de
produção**, medido nas duas peças que o artista de facto olhou (2026-08-24):

| peça (a da cena de smoke) | aspecto p50 | enviesamento p50 | `>60°` | ⛔ células más | ⛔ bordo |
|---|---|---|---|---|---|
| enrugada (`=35`) | ⭐ `1,15` | ⭐ `5,7°` | ⭐ `4` | **19 de 2 041** (`0,9 %`) | `46` |
| orelha (`=36`) | ⭐ `1,12` | ⭐ `7,1°` | `7` | **33 de 2 071** (`1,6 %`) | `50` |
| — a barra do **oráculo** — | `1,08`–`1,22` | `4,8°`–`7,1°` | `0`–`4` | | |

**Veredito do dono do produto**, sobre estas fotos: *«resultado razoável … mas é o melhor
resultado conseguido até agora»* · *«obedece razoavelmente o relevo»* · e as duas queixas:
**«vários buracos»** e **«não é perfeitamente fiel à curvatura da topologia»**.

⭐⭐⭐ **E as duas queixas são o MESMO mecanismo em falta.** É o achado que dimensiona esta
obra: o *paper* de 2009 resolve as duas com **uma** frase — *restrições lineares entram
eliminando uma variável por restrição independente*. A costura é uma restrição linear; a
aresta de feição é outra. ⇒ **uma obra, dois pagamentos.**

---

## §1 — ⛔ A LEI, e ela é a espinha de tudo o que vem abaixo

> **Uma restrição linear entra ELIMINANDO uma variável. Nunca como termo de energia.**

⚠️ **É a diferença entre *o sistema é penalizado quando desobedece* e *o sistema não
consegue desobedecer*.** Um termo de energia com peso `w` produz sempre um compromisso:
existe um `w` alto que fecha a restrição e estraga o resto, e um `w` baixo que preserva o
resto e deixa a restrição aberta. ⛔ **Não existe `w` que faça as duas** — e isto está
**medido no nosso código**, não suposto:

| `SEAM_WEIGHT` | ângulo da grade | resíduo da costura |
|---|---|---|
| `8` (o que shipa) | ⭐ `2,9°` | ⛔ `0,23` de célula |
| `64` | `12,3°` | `0,004` |
| `512` | ⛔ `13,0°`–`16,8°` | ⭐ `0,0006` |

⇒ *Fechar a costura custa o alinhamento, e a curva não tem joelho.* **Uma restrição a
fingir-se de termo de energia tem esta assinatura exacta**, e reconhecê-la é o critério
para saber que a cura é a eliminação e não um número melhor.

⚠️ **O que a eliminação COMPRA, dito como requisito e não como método:** depois dela, o
resíduo da restrição é **zero por construção** — não «pequeno», não «abaixo de uma
tolerância». Não há caminho de execução que o torne diferente de zero, porque a grandeza
que o mediria deixou de ser uma variável livre.

---

## §2 — OBRA A: a COSTURA (é o que fecha a casca)

### §2.1 — O estado de hoje, medido

⚠️ **Duas grandezas estão a ser lidas como uma, e é isso que esconde o defeito:**

| grandeza | o que mede | hoje |
|---|---|---|
| a **translação** da costura é inteira? | `x == x.round()` | ⭐ **sim, exactamente**, nas três peças medidas (o G5 entrega-o, e há gate) |
| os **dois lados** da costura coincidem? | `‖ z_b − R(k)·z_a − t ‖`, em células | ⛔ **`1,00` a `1,41`** — uma célula inteira de desacordo, nas três peças, **e não há gate nenhum** |

⇒ ⭐⭐ **O arredondamento torna a costura INTEIRA; ele não a torna FECHADA.** Só a segunda
é a propriedade de que a extracção depende. *Um gate que mede a condição necessária, num
sítio onde se lê a suficiente, fica verde sobre um mapa que não serve.*

⛔⛔ **E o defeito é INVARIANTE à afinação** — a sonda que já existe varre as duas
constantes da escada do arredondamento em toda a gama:

| tolerância × tecto | fracção no degrau barato | visitas | ⛔ resíduo da costura |
|---|---|---|---|
| `1e-2` × `2 000`…`200 000` | `98,6 %`…`100 %` | `18 282`…`18 588` | `1,0834` · `1,0849` · `1,0849` |
| `1e-3` × `2 000`…`200 000` | `42,9 %`…`100 %` | `112 585`…`481 365` | `1,0369` · `1,0371` · `1,0369` |
| `1e-4` × `2 000`…`200 000` | `1,4 %`…`91,4 %` | `139 908`…`6 464 031` | `1,0897` · `1,0370` · `1,0369` |

⇒ *A fracção varia de `1,4 %` a `100 %`, as visitas variam 350×, e o rasgo fica onde
estava.* **Um defeito que não se move quando os dois botões do subsistema varrem toda a
gama não é afinação daquele subsistema.**

### §2.2 — O mecanismo do rasgo, para não se curar o sintoma

O guloso do arredondamento é forçado a pregar variáveis a **`0,49` de célula** (medido:
`0,4913` · `0,4922` · `0,4955` · `0,4994` nas peças). ⚠️ **Meia célula tem de ir para
algum lado.** Com a costura a ser uma penalização, o sistema paga um pouco de energia e
**abre a costura**; com ela eliminada, o deslocamento só pode ser absorvido pelo interior,
que é onde ele é barato.

⭐ **A assinatura, e ela é verificável:** na esfera fina o resíduo da costura vai de
`0,2348` **para `1,0000`** ao longo do arredondamento. *O arredondamento não herdou o
rasgo: ele criou-o.*

### §2.3 — O requisito

1. ⛔ **Os dois lados de uma costura deixam de ser duas variáveis livres.** Um deles é
   **derivado** do outro pela transição `z_b = R(k)·z_a + t`, e a transição já é um facto
   conhecido do campo (o salto de período por aresta).
2. ⇒ **O resíduo da costura passa a ser zero por construção**, e não por peso.
3. ⚠️ **A translação `t` continua a ser a variável inteira**, e o arredondamento
   uma-a-uma continua a ser sobre ela — ⭐ **e a resposta a *quais* translações já existe
   nesta casa**: o [`gauge`](../../../crates/ph2d-gridmap/src/gauge.rs) provou que a
   translação de uma costura é grandeza de **calibre**, que numa árvore de expansão elas
   vão todas a zero de graça, e que as que restam são as **`E − V + componentes` que fecham
   ciclo**. ⛔ *Não reconstrua isso.*
4. ⚠️ **O peso da costura deixa de existir como knob** quando a eliminação estiver de pé.
   ⛔ Mantê-lo ao lado da eliminação seria a mesma lei escrita duas vezes, e a segunda
   ganharia em silêncio.

⛔⛔ **O que esta espec NÃO diz, de propósito:** por que estrutura o sistema reduzido é
representado, se a eliminação acontece na montagem ou na varredura, e em que ordem.
*Essas são decisões de implementação, e há uma implementação alheia neste disco que as
tomou de uma maneira particular — a nossa tem de nascer da lei, não da forma dela.*

---

## §3 — OBRA B: as LINHAS DE FEIÇÃO (é o que encosta a grade ao vinco)

⭐ **A mesma lei, outro consumidor.** Uma aresta de feição impõe que uma das duas
coordenadas seja **constante e inteira** ao longo dela — o que garante que ela cai sobre
uma isolinha inteira e **sobrevive** à extracção. É uma restrição linear entre dois
vértices ⇒ **elimina uma variável**, exactamente como a costura.

⚠️ **São TRÊS consumidores, e falhar um deles anula os outros dois:**

| fase | o que a feição impõe ali | o que acontece se faltar |
|---|---|---|
| **campo** (F2) | as arestas de feição entram como **restrições de orientação** nos dois triângulos vizinhos | a cruz não fica paralela ao vinco, e o resto não tem como o preservar |
| **mapa** (G3) | uma coordenada **constante e inteira** ao longo da aresta ⇒ uma variável eliminada, e o valor entra no conjunto dos inteiros do guloso | a grade atravessa o vinco em diagonal |
| **extracção** | ⭐ **já está especificado** — [`SPEC_extracao_de_malha_quad.md` §2.5](SPEC_extracao_de_malha_quad.md): *encoste as arestas de feição à isolinha inteira mais próxima, **aqui** — não depois* | a isolinha passa perto do vinco e não sobre ele |

⭐⭐ **E há um pagamento de bónus, publicado:** a mesma maquinaria de alinhamento aplicada
às arestas de **bordo** preserva o bordo na malha de saída e evita bordo serrilhado. ⇒ o
nosso caso de bordo (hoje resolvido **sem oráculo**) melhora sem código próprio.

### §3.1 — O que PRODUZ as arestas de feição

⚠️ **Hoje não existe nada:** `grep` por detecção de feição sobre `ph2d-crossfield`,
`ph2d-trace`, `ph2d-gridmap`, `ph2d-quadextract` e `ph2d-remesh-iso` devolve **zero
ficheiros** (medido 2026-08-24).

**A lei, do *paper* de 2009 (§3 dele, público):** as direcções que interessam são as das
regiões **parabólicas** — onde as duas curvaturas principais são muito diferentes —,
porque só elas têm orientação bem definida. A grandeza é a **anisotropia relativa** das
curvaturas principais, normalizada para `[0, 1]`, e uma região quase plana é excluída por
um piso de curvatura média. A estimativa é feita sobre uma **vizinhança geodésica**, e
como ela depende do raio, mede-se numa **faixa** de raios e escolhe-se a leitura **estável**
— a de menor variação de direcção dentro da faixa.

⛔⛔ **O *paper* dá números concretos para os quatro coeficientes. Esta espec NÃO os copia,
e a recusa é deliberada** ([`project-memory`](../../../project-memory/feedback_a_clean_spec_is_less_specific_than_the_paper_it_descends_from.md)):
*quem traduz código herda as constantes; quem descreve herda a lei.* ⇒ **MEÇA-OS no nosso
corpus**, e escreva ao lado a tabela que a medição deu (`CLAUDE.md` §0.0). ⭐ Os quatro têm
sentido intuitivo e dois deles são **relativos a grandezas que já temos** (o passo alvo da
grade `h`, o raio da caixa da peça) — o que torna a varredura barata e o resultado
defensável.

⚠️ **Uma cerca que o próprio *paper* declara:** as restrições devem ser **esparsas e
conservadoras**. ⛔ *Marcar feição a mais é pior que marcar a menos* — cada restrição
força singularidades, e um campo cheio de restrições duvidosas produz aglomerados que
nenhum alisamento remove. ⇒ **a régua desta obra não é «quantas feições achámos», é «a
peça ficou melhor»** (§5).

---

## §4 — ⛔ O que JÁ EXISTE do nosso lado (não reconstrua)

| a peça | onde vive | estado |
|---|---|---|
| campo cruzado com decisão inteira global | `ph2d-crossfield` | ⭐ ilibado por resultado — bate o campo do oráculo **na malha dele** |
| alinhamento do campo ao **relevo** | `ph2d_crossfield::ALIGN_WEIGHT` (`0,03`) | ✅ **VIVO nos dois caminhos do botão** ⚠️ e o doc de módulo dizia o contrário até 2026-08-24 |
| a continuação do alinhamento sob arredondamento | `ph2d-crossfield/continuation.rs` | ✅ — é o ponto de extensão onde uma **restrição de orientação** nova encaixa |
| índice/valência por vértice | `ph2d_crossfield::vertex_index` | ✅ |
| salto de período por aresta | `ph2d-crossfield` / `ph2d-gridmap` | ✅ — **é a transição** de que o §2.3 precisa |
| ⭐ **quais translações são de facto livres** | `ph2d_gridmap::gauge` | ✅ — `E − V + componentes`; ⛔ **não re-derive** |
| arredondamento uma-a-uma + escada | `ph2d_gridmap::round` (G5) | ✅ — a escada fica; o que muda é o sistema que ela relaxa |
| a extracção | `ph2d-quadextract` | ✅ — e o gancho de feição do §2.5 dela **já está escrito** |
| a régua por-face | `ph2d_quadfill::QuadShape` | ✅ — **é a barra** |
| o instrumento ponta-a-ponta | `cargo run --release -p ph2d-quadextract --example chain_info -- <peça\|ficheiro.obj>` | ✅ — ⭐ aceita um `.obj` do corpus desde 2026-08-24 |

---

## §5 — Os GATES, e a barra de cada um (derivada, nunca de conforto)

| # | o gate | a barra, e de onde ela vem |
|---|---|---|
| 1 | ⭐⭐ **o resíduo da costura é ZERO** | ⛔ **é o gate que não existe hoje**, e é o coração da obra. A barra é a dos mapas de referência: `3,5e-15`, **não** «pequeno». ⚠️ Prove por **mutação** que desligar a eliminação fica vermelho |
| 2 | a translação continua inteira | `x == x.round()`, exacto — o gate que já existe, e ele **fica** |
| 3 | ⭐ **a casca fecha** | `χ` da saída = `χ` da entrada, **e zero arestas de bordo** numa peça fechada. Hoje: `−8` e `46` na enrugada, `−6` e `50` na orelha |
| 4 | ⭐ **a forma NÃO regride** | a barra do oráculo, medida pelo mesmo código: aspecto p50 `1,08`–`1,22` · enviesamento p50 `4,8°`–`7,1°` · `>60°` = `0`–`4`. ⚠️ **A enrugada já está lá dentro** ⇒ este gate é de **não-regressão**, e é o que impede a cura de comprar topologia com geometria |
| 5 | uma aresta de feição **sobrevive** | ela cai sobre uma isolinha inteira, e existe uma fileira de arestas da saída sobre ela. ⛔ Sem este gate, «alinhado» é uma opinião |
| 6 | ⚠️ **o campo obedece à feição** | o ângulo entre a cruz e a aresta de feição, nos triângulos vizinhos. ⛔ Meça-o **antes** do mapa: um alinhamento que falha aqui não pode ser diagnosticado no fim |
| 7 | ⛔ **as restrições ficam ESPARSAS** | a contagem de arestas marcadas contra a contagem de arestas da peça, e a **contagem de singularidades** ao lado. *Marcar feição a mais aparece como singularidades a mais, nunca como uma feição feia* |
| 8 | ⭐ **o bordo é preservado** | a peça do corpus com bordo; as arestas de bordo da saída caem sobre as da entrada |
| 9 | ⛔ **o caminho antigo continua byte-idêntico** com o interruptor desligado | a lei desta casa: tudo o que é novo shipa **desligado** com a tabela ao lado |

⚠️ **A comparação fase a fase está disponível e é mais forte que comparar o fim** — o
oráculo grava o **campo** dele e a **decomposição** dele em `ph2d-quadbench/ref/<peça>/`
([`LEDGER`](LEDGER_quadwild.md)), e ler saída **não** é obra derivada.
⛔⛔ **Correr o arnês do oráculo NÃO é acto do Implementador** (o mecanismo está na
[`SPEC_extracao_de_malha_quad.md` §9](SPEC_extracao_de_malha_quad.md): ele é um consumidor
*header-only*, e um erro de compilação despeja a implementação alheia no terminal).
⇒ **I consome dumps prontos.** Falta um? Peça-o pelo Enio, como emenda.

---

## §6 — A ORDEM, e por que ela não é livre

1. ⭐ **A costura primeiro.** Ela é o que o artista vê (`~1 %` de células rasgadas), é o
   que impede a feature de shipar ligada, e ⚠️ **é pré-requisito da outra**: uma restrição
   de feição é a mesma maquinaria, e construí-la sobre um sistema que ainda penaliza
   costuras faria a feição herdar o rasgo.
2. **As feições depois**, e nessa ordem os três consumidores do §3.
3. ⛔ **Não misture as duas numa wave só.** Se a forma regredir (gate nº4), com as duas
   dentro não há como saber qual delas a moveu.

---

## §7 — ⛔ Recusas MEDIDAS

| recusa | mecanismo | onde |
|---|---|---|
| ⛔ **Não afinar o `SEAM_WEIGHT`** | a curva não tem joelho: `8` dá `2,9°`/`0,23`, `512` dá `13–17°`/`0,0006`. Fechar a costura custa o alinhamento | §1 |
| ⛔ **Não afinar as duas constantes da escada do arredondamento** | o rasgo é **invariante** nas 9 combinações, com a fracção do degrau barato a variar de `1,4 %` a `100 %` | §2.1 |
| ⛔ **Não procurar a causa nas DOBRAS do mapa** | na esfera fina e nas duas peças do smoke o mapa tem **`0,0 %` de dobras** e o rasgo é o mesmo. *A atribuição «é a montante, são as dobras» foi escrita, e refutada por medição* | §2.1 |
| ⛔ **Não ler `shift_frac_max` como se fosse o fecho da costura** | ele é `0` exacto nas três peças **enquanto** o resíduo é uma célula inteira | §2.1 |
| ⛔ **Não acusar o campo de não obedecer ao relevo** | o `ALIGN_WEIGHT` shipa a `0,03` e está vivo nos dois caminhos; o artista confirmou-o (*«obedece razoavelmente o relevo»*). ⚠️ O doc de módulo dizia «INERTE» e estava velho | §4 |
| ⛔ **Não marcar feição a mais** | cada restrição força singularidades; um campo com restrições duvidosas produz aglomerados que nenhum alisamento remove. O próprio *paper* pede **esparso e conservador** | §3.1 |
| ⛔ **Não copiar as constantes do *paper*** | uma espec limpa é **menos** específica que o *paper* de que descende: quem traduz código herda as constantes, quem descreve herda a lei. **Meça-as** | §3.1 |
| ⛔ **Não construir a detecção de feição antes da costura** | a feição é a mesma maquinaria e herdaria o rasgo; e com as duas na mesma wave uma regressão de forma fica sem dono | §6 |
