# LEDGER de proveniência — clean-room do PINCEL QUE APARA ESFREGANDO (alvo `blender-trim-pincel`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-trim-pincel.md` (append cego).

⏱️ **Aberto em 2026-09-15, ANTES da primeira leitura de CONTEÚDO do fonte.** Antes desta abertura
o E fez apenas:

1. **metadados de pacote** desta máquina (`pacman -Qi blender`, `blender --version`) — o primeiro
   acto obrigatório da triagem (§2: *ler a licença real*);
2. uma **listagem de NOMES** de ficheiros do directório do modo escultura e do subdirectório dos
   pincéis (nenhum conteúdo aberto) — para localizar a obra;
3. a leitura dos **cabeçalhos SPDX de 3 linhas** de um ficheiro de pincel (é licença, não obra);
4. leitura de artefactos **NOSSOS** do repo (a SKILL, `_ComoInvestigarApps`, o precedente
   `LEDGER_blender-trim.md`, e o código da nossa crate de escultura);
5. **prosa pública** (manual do alvo, fórum público de desenvolvimento, imprensa) e a busca de
   patente — que é o checkpoint §8.1, obrigatório *antes* de E começar.

---

## ⚠️ ESTE ALVO NÃO É O `blender-trim` — e a distinção é a razão de ele ter ledger próprio

| | `blender-trim` (obra ANTERIOR, 2026-09-15) | **`blender-trim-pincel`** (esta obra) |
|---|---|---|
| o que é | um **GESTO**: desenha-se uma forma no ecrã **uma vez** e o pen-up corta a peça com uma booleana de volume | um **PINCEL**: **esfrega-se** sobre a superfície e ela vai sendo aparada/achatada contra um plano local, dab a dab |
| o que o move | booleana de malha (biblioteca externa Apache-2.0) | deslocamento de vértices por dab, com falloff |
| a espec dele | `SPEC_trim_gesture.md` | esta obra |
| verbo nosso | `Verb::BoxTrim` | `Verb::Flatten` / `Fill` / `Scrape` / `Clay` / `ClayThumb` / `MultiplaneScrape` |

⇒ **Os dois chamam-se «trim» e não têm uma linha em comum.** O pedido do dono
(*«faz o trim esfregando o pincel como massinha, modelando»*) é **inequivocamente o segundo**: o
verbo dele é **esfregar**, e o gesto anterior não se esfrega — desenha-se e larga-se.

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — a **família de pincéis que ajustam um PLANO local à pegada e movem os vértices em direcção a ele** (o pincel unificado de plano que a versão nova entrega, os seus modos e estabilizadores, e o corte por distância ao plano que toda a família partilha, incluindo os pincéis de barro e a lâmina em V) |
| Versão / commit do fonte lido | tag **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` — checkout **esparso e grafted**, o MESMO das obras `blender-cloth`, `-pose`, `-boundary`, `-pull`, `-unblocked`, `-trim` |
| Onde vive o fonte | `/home/enio/Documentos/Recursos/BlenderSculpt/` — fora de qualquer árvore do PH2D |
| Zona fora da árvore (notas, rascunhos, oráculo, fixtures cruas) | `~/Referencias/blender-trim-pincel/` (`notes/` · `draft/` · `oracle/` · `fixtures/`) |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ na denylist do I) |
| Licença do ALVO | **GPL-2.0-or-later** — `COPYING` remete a `doc/license/GPL-license.txt` (GPLv2, junho 1991); os ficheiros da obra levam `SPDX-License-Identifier: GPL-2.0-or-later` |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.1 LTS**, pacote `blender 17:5.2.1-2` (build 2026-09-01, commit de 2026-08-24) |

### A concessão relevante (GPLv2 §0), transcrita do ficheiro do checkout

> *"Activities other than copying, distribution and modification are not covered by this
> License; they are outside its scope. The act of running the Program is not restricted,
> and the output from the Program is covered only if its contents constitute a work based
> on the Program (independent of having been made by running the Program)."* (§0)

⇒ Ler, correr e instrumentar **em privado** é licenciado. A **saída** (posições de vértices de
malhas NOSSAS) é dado. Nenhum acto deste ledger envolve distribuição. Não é AGPL.
⚠️ GPLv2-only-style (§2(a)): a leitura de que modificação privada não exige release é **consensual,
não textual** — logo, todo ficheiro do alvo que esta obra instrumentar leva uma linha de aviso de
mudança no topo. **Nesta obra nenhum ficheiro do alvo foi modificado** (o oráculo corre por script
Python sobre o binário do pacote, sem patch) ⇒ o §2(a) não chega a ser accionado.

---

## §2 — Triagem: a escada de portas (percorrida NA ORDEM, 2026-09-15)

⚠️ A unidade da triagem é o **ARTEFACTO/biblioteca**, nunca o nome do projecto (lição do `mypaint`).

| degrau | veredito | por quê (medido) |
|---|---|---|
| **T0** (permissivo) | ⛔ | ⚠️ **Verificado por LIGAÇÃO DINÂMICA, não por reputação:** ao contrário do gesto de corte — cujo motor de booleana se revelou uma biblioteca externa **Apache-2.0** (`ldd` ⇒ `libmanifold.so.3`, `pacman -Qi manifold` ⇒ `Apache-2.0`) —, **este pincel não delega a biblioteca externa nenhuma.** Ele é aritmética de vértices dentro do alvo: uma média ponderada, um produto interno e uma interpolação. Não há `.so` de terceiros a que atribuir a lei. ⇒ a porta do gesto **não se repete aqui** |
| **T0½** (copyleft por-ficheiro) | ⛔ | os ficheiros da obra levam `GPL-2.0-or-later` no SPDX; nenhum é MPL/LGPL |
| **T1 (a)** código dos autores | ⛔ | não há paper nem repositório de referência paralelo — a lei nasceu dentro do alvo, discutida no fórum público de desenvolvimento dele |
| **T1 (b)** versões antigas | ⛔ | a família nasceu já sob GPL; a unificação recente também |
| **T1 (c)** reimplementações permissivas | ⚠️ **MEIA porta, e ela já está paga** | ⭐ O **SculptGL (MIT)** tem a metade antiga da família — o achatamento contra um plano de área, com o deslocamento do plano e a escolha de lado. **Esta casa já o portou** (`ref_kernels`, `stroke_plane`, `stroke_target`) e ele é a base dos nossos `Flatten`/`Fill`/`Scrape`/`Clay`. ⛔ **O que ele NÃO tem é exactamente o que o dono pediu**: o corte por distância ao plano, os dois limites por lado, os estabilizadores do plano, a amostragem de raio próprio e os modos de inversão. Busca de 2026-09-15: nenhuma implementação permissiva desses. ⚠️ E o autor do SculptGL **arquivou** o repositório em Jan/2026 e trabalha hoje num produto **proprietário** — não há versão nova permissiva a caminho |
| **T1 (d)** e-mail aos autores | n/a | a licença do alvo é institucional (fundação); dual-license não se coloca |
| **T2** | ✅ **este é o degrau** | copyleft com fonte — o pipeline da SKILL |

⇒ ⭐ **A obra parte-se em duas, e a metade barata JÁ ESTÁ NA CASA.** O estimador de plano e a
projecção são porte MIT que já shipa; o que esta espec tem de descrever é a **camada que o alvo
acrescentou por cima** — e é essa, e só essa, que paga clean-room.

---

## Patente (§8.1) — checkpoint incondicional (2026-09-15)

- **Termos:** `patent sculpting brush flatten plane trim clay scrape 3D digital sculpting method
  plane offset` · `patent digital sculpting brush flatten plane averaging vertices normal offset
  polygon mesh deformation claims granted`.
- **Achados e veredito, um a um:**

| patente | assunto | estado | alcança-nos? |
|---|---|---|---|
| **US 9 830 743** | pincel de suavização **com preservação de volume** — estima a mudança de volume pelo comprimento médio dos vectores laplacianos da região do pincel | concedida 2017 | ⛔ **não.** Assunto **diferente** (suavização laplaciana com compensação de volume); esta obra não suaviza, não usa laplaciano e não estima volume. ⚠️ **Registada aqui de propósito** — ela é vizinha do nosso `Verb::Smooth`/`SurfaceSmooth` e **quem mexer NAQUELES tem de a reler** |
| **US 6 724 393** · **US 7 031 790** · **US 7 034 818** | esculpir modelos digitais (volume varrido; corpos-folha; dados de alcance) | prioridade ~2001 ⇒ **EXPIRADAS** | ⛔ não |

- **Prior art esmagadora e pública** para o que esta espec descreve: achatar uma malha contra um
  plano ajustado por mínimos quadrados/média de área é literatura e produto desde os anos 1990
  (as ferramentas de achatar/polir dos esculpidores comerciais da época), e a metade antiga
  distribui-se hoje sob **MIT**.
- ⇒ **Veredito: nenhuma patente VIVA alcança o método.** Não há gatilho de §8.1 para o Enio.

---

## §8.5 — Válvula de escalonamento: NÃO accionada

Dono do alvo historicamente litigioso? **Não** (fundação de software livre; o histórico dela é
exigir o *copyleft*, e esta obra não distribui obra derivada). AGPL/rede? **Não** (GPLv2). Mercado
fora do analisado? **Não** (Brasil/UE/EUA).

---

## Cobertura da travessia (§3.E) — 2026-09-15

⚠️ **Honesto sobre o que é INTEGRAL e o que é PARCIAL.**

| área do assunto | linhas | lido |
|---|---|---|
| o pincel unificado de plano — **o assunto** | 521 | ✅ **INTEGRAL** |
| a lâmina em V (o irmão que partilha o corte) | 830 | ⚠️ **~75 %** — integral no ponto de entrada, na amostragem das duas metades, nos filtros de lado, nas distâncias e nas translações; saltados os três corpos por tipo de topologia, que repetem a mesma lei |
| o pincel de tiras de barro (o outro consumidor VIVO do corte) | 534 | ⚠️ **~60 %** — integral no corte, no factor do eixo local, no quadro local e no ponto de entrada |
| o pincel de barro | 245 | ✅ **INTEGRAL** |
| o polegar de barro | 270 | ✅ **INTEGRAL** |
| a maquinaria partilhada de pincéis de malha (cabeçalho) | 535 | ✅ **INTEGRAL** no que toca a planos, distâncias, factores e translações |
| o núcleo do modo escultura — **só as áreas do assunto** | (de ~8 400) | ✅ **INTEGRAL** em: construção do plano do pincel · o estabilizador · a amostragem de normal e centro de área (as três variantes de topologia) · os dois raios de amostragem · o filtro de corte · os filtros de lado do plano · o deslocamento do plano · a inicialização do quadrado do limiar. ⛔ **Não integral** no resto do ficheiro, e não precisa de ser |
| declarações de tipos de pincel e sinalizadores | — | ✅ integral na região do assunto — ⭐ é aqui que se mede que **três** verbos desta família estão marcados obsoletos e **nada os despacha** |
| a camada de propriedades públicas (faixas, valores de fábrica, textos de painel) | — | ✅ **INTEGRAL** para os 12 controlos do assunto |
| o painel (o que é mostrado a que verbo) | — | ✅ **INTEGRAL** na secção do assunto — é onde se mede que **o painel mostra o corte em três verbos e o esconde no quarto**, ao contrário do que o motor faz |
| as funções de capacidade (que verbo oferece que controlo) | — | ✅ integral nas 6 do assunto |

⛔ **História: NÃO disponível no checkout.** Ele é `grafted` com profundidade **1** (uma única
revisão), logo não há mensagens de commit nem `git log` por ficheiro. A mineração de §4.1.12 foi
feita na **prosa pública** (manual do alvo + fórum público de desenvolvimento dele), e está
destilada na §10 da espec, em palavras nossas.

---

## Oráculo (§5) — o que foi corrido, e o que ele NÃO tocou

| | |
|---|---|
| binário | `/usr/bin/blender` 5.2.1 LTS, do pacote — ⛔ **sem patch, sem build próprio** ⇒ o §2(a) da GPLv2 não chega a ser accionado |
| como | janela dentro de um compositor **virtual** (`kwin_wayland --virtual`), `--factory-startup`, script Python sobre a API pública |
| entradas | ⭐ **100 % nossas** — grelhas de quadriláteros com campo de altura analítico e uma esfera por projecção de cubo. ⛔ Nenhum asset do alvo |
| ⭐ **pincéis** | o harness **cria o pincel dele próprio** e marca-o como recurso local; ⛔ **nenhum ficheiro de pincéis do alvo é aberto em nenhuma corrida** (o precedente `blender-unblocked` importava um; este não precisa) |
| corridas | **149** em duas rodadas (79 + 71, mais 2 de smoke); **1** falhou por nome de opção errado e foi refeita |
| publicadas como fixtures | **100**, em `docs/3D/cleanroom/fixtures/pincel_de_plano/` (2,5 MB) |
| reprodução independente da lei | `oracle/reproduz.py` — 24 de 25 configurações a `≤ 8,0e-08`, 1 a `3,2e-05` com a causa **nomeada e medida** (espec §4.1) |

---

## ⛔⛔ ACHADO DE INSTRUMENTO — a vassoura era cega à TERCEIRA fonte de prosa (2026-09-15)

**Medido com controlo dos dois lados, e curado no mesmo turno.**

A vassoura desta obra nasceu com **264** entradas: identificadores internos, **comentários** do
fonte e **dicas de painel**. O controlo positivo foi corrido reconstruindo a redacção **anterior**
da espec — aquela que a filtragem §4.3 tinha acabado de limpar — e varrendo-a.

| redacção | trechos que a filtragem removeu | hits da vassoura de 264 | hits da vassoura de 321 |
|---|---|---|---|
| anterior (suja) | 3 | **1** | **2** |
| esta (filtrada) | — | 0 | **0** |

⇒ **a vassoura não carregava as frases da DISCUSSÃO PÚBLICA DE DESENHO do alvo**, que é a terceira
fonte de prosa que o §4.1.12 manda destilar (as outras duas são o comentário e o manual). Uma
espec que mina o fórum de desenvolvimento e cuja vassoura só conhece comentários **não é vigiada
na fonte de que mais bebe**. Acrescentadas **57** entradas dessa fonte, nas duas línguas e com e
sem acentos.

⚠️ **E o terceiro trecho continua a não ser apanhável, por construção:** ele não era uma frase do
alvo — era uma observação sobre a **FORMA** do código dele (a existência de um caso especial).
*Nenhuma vassoura apanha forma; é para isso que o R-pré existe.*

---

## Papel E — Especificador

| campo | valor |
|---|---|
| quem | **subagente-E (alvo `blender-trim-pincel`)** despachado pela janela I da `line/sculpt3d` |
| data | 2026-09-15 |
| ⚠️ transcript | **zona contaminada** — a janela I ⛔ não o lê (§3.I) |

## Corrente I

(a janela I declara-se no `INBOX_blender-trim-pincel.md`)

## Papel R

R-PRÉ: subagente **independente do E**, corrido em **2026-09-16** (janela
`9f820704-0d7e-4d96-847e-9cd720cbf178`) · R-PÓS: (pendente)

### R-PRÉ — veredito: ⚠️ **NÃO atestado.** A parede do §4.2 está LIMPA; ficam **3 achados de EXACTIDÃO**

#### 1. A parede (§4.2) — limpa, item a item

| item do §4.2 | resultado |
|---|---|
| texto de código, trechos, diffs | **nenhum** |
| nomes internos (função, variável, ficheiro, struct) | **nenhum** — varridos todos os `snake_case` e todos os *code spans* do documento: cada identificador é **nosso** (`brush_verb.rs`, `stroke_plane.rs`, `fit_plane`, `front_only`, `RefMode`, `Verb::BoxTrim`, os sete verbos do nosso catálogo), nome de fixtura **nossa**, ou variável local da própria espec |
| comentários do original | **nenhum** — a prosa do alvo está re-dita; o sweep cobre as duas línguas com e sem acentos e não acusa |
| wording de manual/doc-comment verbatim ou quase | **nenhum**. O caso mais próximo é a frase de UMA oração que declara a limitação da inversão (§5/§10.4): é um facto funcional com essencialmente uma forma de o dizer, está **atribuído** aos autores e é curto — cabe no direito de citação do §4.1.12, e mesmo assim está re-escrito |
| tabela verbatim / LUT no lugar da fórmula | **nenhuma** — todas as tabelas do documento são **medições nossas**; a única fórmula fechada (a curva suave) é a que a casa já usa |
| organização arquivo-a-arquivo / função-a-função transcrita | **nenhuma** — a espec descreve por FASES, e a ordem delas é **forçada por dependência de dados** (sem plano não há quadro local; sem quadro local não há factor; sem factor não há translação), logo não é a «organização arbitrária» que o §4.2 proíbe. ⭐ Em dois pontos ela chega a mandar **divergir** da forma do alvo (o tratamento dos dois casos extremos dos tectos, §3.3) — o oposto de transcrever |
| pseudo-código espelhado linha a linha | **não**. Os blocos das §§2–4 e §6 são 3–7 linhas de fórmula em vocabulário do domínio, contra dezenas de linhas do alvo com nomes próprios; nenhum nome do alvo sobrevive e a ordem é a da dependência |

⇒ ⭐ **A emenda que os 3 achados pedem é FACTUAL, não de filtragem** — ela não reabre a §4.3
nem obriga a re-filtrar o documento.

#### 2. O sweep, e o CONTROLO POSITIVO (corrido, cinco canais)

`bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_blender-trim-pincel.txt
docs/3D/cleanroom/SPEC_pincel_de_plano.md docs/3D/cleanroom/fixtures/pincel_de_plano/`
⇒ **✓ limpo, 321 entradas, `exit 0`.**

⛔ **E o instrumento foi APONTADO A ALGUMA COISA antes de o verde contar.** Canários plantados
**fora da árvore** (scratchpad da sessão, nunca no repo) e apagados a seguir — o sweep foi a
`exit 1` e **nomeou os cinco**:

| canal | canário | acusado? |
|---|---|---|
| NOME de ficheiro | identificador interno no nome | ✅ |
| conteúdo de texto | identificador interno numa linha | ✅ |
| passagem com as quebras de linha desfeitas | prosa traduzida **sem acentos**, partida a meio por uma quebra de linha | ✅ |
| idem, com ênfase | prosa traduzida **com acentos**, partida por `**…**` e `` `…` `` | ✅ |
| **dentro de um `.gz`** | linha de cabeçalho com prosa do alvo, em inglês **e** em português acentuado | ✅ (o ramo que descomprime em memória foi mesmo exercido, com o rótulo a nomear a fixtura) |

⚠️ **ACHADO DE INSTRUMENTO, medido aqui, com controlo dos dois lados:** o ramo do `.gz` **não
tem a passagem que desfaz as quebras de linha** que o ramo de texto tem. Medido: a MESMA frase
da vassoura dentro de um `.gz` é **acusada** quando cabe numa linha do cabeçalho e passa
**LIMPA** quando está partida em duas linhas. ⇒ As 100 fixtures desta obra estão **cobertas**
(o formato é `# chave: valor`, uma linha por grandeza, e o caso de uma linha é o que dispara);
o ponto cego morde no dia em que uma fixtura levar um comentário **dobrado**. Fica registado
para o dono do script — a cura é a mesma normalização que o ramo de texto já faz.

⭐ **E os identificadores PÚBLICOS do cabeçalho das fixtures foram CONFERIDOS um a um contra a
declaração de API do alvo:** as **15** cadeias em maiúsculas que aparecem nos 100 cabeçalhos são
todas identificadores de enumeração **públicos** — incluindo um cujo nome público **difere** do
nome interno correspondente, e o cabeçalho traz o **público**, que é o que regenera. ⇒ a
autorização do §4.1.13 escrita no cabeçalho da espec está **aplicada correctamente**.

#### 3. Os 3 achados (entregues à janela como instruções funcionais de reescrita)

1. ⛔⛔ **A espec atribui à NOSSA casa uma lei de centro de plano que ela já não tem — e é a lei
   de que os dois números que dimensionam o buraco (B) são leitura.** O nosso estimador foi
   mudado em **2026-08-11**: ele deixou de pesar pelo *falloff* e passou a pesar pela **máscara**
   (numa pegada sem máscara ⇒ **média aritmética simples**), e amostra a pegada **inteira** do
   pincel, não uma fracção dela. ⚠️ **O documento contradiz-se sobre isto**: a célula que compara
   a ponderação da NORMAL já diz que a nossa é a máscara, enquanto a tabela de candidatas rotula
   a média ponderada pelo *falloff* como «o que nós fazemos» — e as duas leituras saem do **mesmo
   fecho de leitura**, logo não podem divergir.
   ⭐ **Medido aqui, sobre as fixtures publicadas** (plano recuperado por ajuste dos vértices
   tocados, o método que a própria §4.1 usa): a média aritmética reproduz a **primeira** linha da
   tabela a **todos** os dígitos impressos (`−0,01392` · `−0,01603` · `−0,00731` · `−0,01613` ·
   `−0,00269`), e o nosso estimador **real** — média simples sobre o raio **inteiro** — lê
   **`+0,03795`** no sulco (`9,5 %` do raio, e de **sinal oposto** ao número que a espec publica),
   `−0,01603` na rampa, `+0,00489` no degrau e `0,00000` nas bossas simétricas.
   ⚠️ E a citação do nosso registo de perf (`5,8 %` do raio ⇒ `0,54×` / `1,74×`) é o **epitáfio**
   daquela lei: é a medição que **causou** a remoção dela, citada como prova de um buraco de hoje.
   ⭐⭐ **A conclusão da wave SOBREVIVE e fica mais forte** — a lei do alvo continua a cair
   exactamente no plano dele e a nossa não, e no sulco o buraco real é **maior**.
2. ⛔ **A barra do gate DISCRIMINANTE está derivada de um piso que a própria tabela da espec
   refuta.** Ele exige que as candidatas erradas reprovem por `≥ 5e-3`, citando como menor desvio
   `0,00731`; a tabela duas secções acima imprime **`0,00269`** para uma candidata errada na
   configuração de amostragem mais larga, e eu reproduzi `0,00269`. ⇒ com a barra onde está, o
   gate **não pode passar** na configuração que a espec publica.
3. ⚠️ **O quadro de fixtures da §11 não fecha:** as famílias listadas somam **99** contra **100**
   ficheiros em disco, e a família do corte tem **26** contra os `25` impressos.

#### 4. O item herdado — o controlo de **arredondamento da ponta** (triagem pedida a este R-pré)

**Veredito: a VASSOURA é que está larga; a autorização do cabeçalho desta espec NÃO precisa de
revisão.** Medições:

- O identificador é **simultaneamente** propriedade **pública** da API do alvo (declarada como
  propriedade de tipo `FLOAT` e lida como tal pelo próprio painel dele) **e** nome de campo
  interno, escritos igual. É exactamente a classe que o §4.1.13 admite.
- ⭐ **As duas vassouras da mesma casa aplicam regras opostas ao mesmo caso, e esta é a que
  segue a regra:** o E desta obra **retirou 16** identificadores públicos da vassoura por serem
  a chave de regeneração das fixtures (conferido entrada a entrada, e eu reconferi 15 deles nos
  cabeçalhos); a vassoura `blender-pull` **manteve** um da mesma classe.
- Ele vive em `VASSOURA_blender-pull.txt` (⛔ não nesta) e acusa **14** linhas de `crates/`, com
  **14** no merge-base ⇒ **zero** adições desta linha; entrou pela wave da faixa, muito antes.
- A **isenção nomeada já existe**, com cinco argumentos medidos, no `LEDGER_blender-cloth.md`
  (secção de 2026-09-13), que regista também que o vermelho é **sabido**.

⇒ **Cura, e ela é do dono daquela vassoura, nunca do nosso produto:** retirar a entrada (ou
marcá-la como aposentada, com o motivo ao lado). ⛔ Renomear custaria 16 sítios em 3 crates,
mudaria **dois `NodeId` hasheados da string** e **o que o artista lê na tela**, para não comprar
parede nenhuma. ⚠️ **E o preço de a deixar está medido no próprio cabeçalho do `cleanroom-sweep.sh`:**
*uma entrada que dispara sobre uso lícito treina quem corre o sweep a ignorar achados* — hoje um
vermelho por isenção registada e um vermelho por dívida **leem-se iguais** numa corrida.
⇒ **Isto NÃO bloqueia esta obra:** nenhuma linha de `crates/` entra no caminho varrido desta
espec, e o sweep desta vassoura sobre a espec e as 100 fixtures está **verde**.

## Vassoura (§7.1)

`docs/3D/cleanroom/VASSOURA_blender-trim-pincel.txt` — **321** entradas em base64, uma por linha:
**104** identificadores internos (nomes de função, de tipo, de sinalizador e de campo interno) e o
resto prosa do alvo — comentários do fonte, dicas de painel e frases da discussão pública de
desenho —, cada uma também em **tradução** e em **variante sem acentos** (a lição do `unblocked`:
*uma vassoura monolingue não vigia uma espec traduzida*, e o sweep casa por texto exacto).

⚠️ **16 identificadores PÚBLICOS da API do alvo foram deliberadamente RETIRADOS** da vassoura:
eles são a chave que torna as fixtures **regeneráveis** (§4.1.13) e viajam no cabeçalho de cada
uma, exactamente como no precedente `blender-unblocked` (conferido entrada a entrada antes de
decidir).

## Espec

`docs/3D/cleanroom/SPEC_pincel_de_plano.md` — entregue em **2026-09-15**, num commit ÚNICO
pós-filtragem (§3.E: `git log -p` retém para sempre o que um rascunho contaminado carregasse),
scoped, `docs-only`, `--no-verify`, com `git add -- <paths>`. Junto dele: o ledger, a vassoura,
o INBOX, a entrada no `README.md` da pasta e as **100** fixtures.

Fixtures: `docs/3D/cleanroom/fixtures/pincel_de_plano/` (2,5 MB, 100 traços em 9 famílias) —
entradas **nossas**, saída do binário do pacote, `gzip` com `mtime = 0` ⇒ reprodutível byte a byte.

## Incidentes

### INC-R1 (2026-09-16) — **HIGIENE DE INSTRUMENTO do próprio R-pré. Sem exposição; registado porque um evento escondido é a acusação pronta**

- **O que:** para auditar a cobertura da vassoura, o R-pré **descodificou-a para um ficheiro** no
  *scratchpad* da sessão. ⛔ O §3.E manda que material do alvo viva **só** em `~/Referencias/`, e
  nomeia o scratchpad entre os sítios proibidos **exactamente porque a janela-mãe o alcança** — e
  este é o scratchpad da janela-mãe (o UUID é o dela).
- **Régua do §6.2:** **não é substancial e não é sequer um relance** — nada foi *lido* por quem
  escreve produto; foi o R a escrever e a apagar um ficheiro derivado, dentro da própria corrida.
  A vassoura descodificada é uma **lista** de identificadores e de fragmentos de prosa, não um
  corpo de função.
- **Janela de risco:** a corrida do R-pré. O ficheiro foi **apagado** no fecho, junto com os
  canários do controlo positivo (que viveram na mesma pasta e nunca no repo).
- **Estado depois:** sweep **verde** com as **oito** vassouras da pasta sobre a espec, este ledger
  e as 100 fixtures; e **verde** em `--git-history` sobre `docs/3D/cleanroom/`.
- ⭐ **Cura, e ela apaga a classe:** o `cleanroom-sweep.sh` **já descodifica em memória** — não há
  razão nenhuma para materializar a vassoura. Quem precisar de a inspeccionar por outro motivo
  descodifica **por cano**, sem passar pelo disco, ou fá-lo dentro de `~/Referencias/<alvo>/`.
  *O instrumento que evita isto já existia; o que faltou foi usá-lo.*
