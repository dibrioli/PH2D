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

## EMENDA DO E — 2026-09-16 (resposta aos 3 achados do R-pré)

⭐ **A tese da wave sobrevive e ficou MAIS FORTE: a lei do alvo continua a cair exactamente no
plano dele, a nossa não, e no sulco o buraco real é MAIOR do que a 1.ª redacção publicava.** O que
mudou foi **de onde saem os números**.

### Achado 1 — a lei NOSSA que a espec media já não existe

Re-derivada do código vivo (`ph2d-sculpt3d`): o estimador que os quatro verbos de plano chamam
pesa pela **MÁSCARA** — pegada sem máscara ⇒ **média aritmética simples** — e amostra a **pegada
inteira** do pincel (a consulta devolve exactamente o raio do dab nesses quatro verbos; os
factores de consulta próprios que existem são de outros verbos). A ponderação pelo *falloff* foi
**removida em 2026-08-11**.

**Re-medido sobre as fixtures publicadas** (plano do alvo recuperado por ajuste, o método da §4.1
da espec), altura do nosso centro contra o plano dele, pincel de raio `0,4` salvo indicação:

| célula | a NOSSA lei real | (a 1.ª redacção publicava) |
|---|---|---|
| sulco | **`+0,03795`** (`+9,5 %` do raio) | `−0,0229` — ⛔ **sinal oposto** |
| rampa | `−0,01603` (`−4,0 %`) | `−0,0160` |
| degrau | `+0,00489` (`+1,2 %`) | `−0,0073` |
| bossas | `0,00000` | `0,0000` |
| raio de amostragem do alvo apertado (`0,25`) | **`+0,06853`** (`+17,1 %`) | — |
| raio de amostragem do alvo a `2R` | `−0,05012` (`−12,5 %`) | — |
| sulco, raio `0,6` | `+0,04186` (`+7,0 %`) | — |

⭐ **E a NORMAL tem o mesmo problema, que a 1.ª redacção também subestimava:** o desvio angular da
nossa normal contra a do alvo vai de **`0,00°`** (rampa — superfície plana, toda média concorda:
é o controlo) a **`16,9°`** no sulco, **`23,5°`** com o raio da normal apertado e **`31,2°`** na
superfície curva nos dois eixos. A candidata do alvo lê `0,0000°` nas **11** células.

⛔ **A citação do nosso registo de perf SAIU da espec, nomeada como EPITÁFIO:** aquela medição
(*«um erro de centro de `5,8 %` do raio move o `Flatten` `0,54×`»*) é de **2026-08-11** e foi a que
**causou** a remoção da lei ponderada. Usá-la para dimensionar o buraco de hoje mede uma lei que
já não existe. *Um registo de perf que documenta uma remoção não é prova de um buraco vivo.*

### Achado 2 — a barra do gate discriminante

Recalculada sobre **todas** as células. Três **não discriminam**, e cada uma tem o mecanismo
nomeado na espec §2.2: duas porque o conjunto de amostragem do alvo tem **`1` vértice** (com uma
amostra as três leis coincidem) e uma porque a superfície é **antissimétrica em torno da linha do
traço** (toda altura média é zero por construção — ⭐ *o mesmo mecanismo do controlo mudo do
estabilizador do centro*). Sobre as **`8`** que discriminam, o piso medido é **`0,00269`** ⇒ a
barra passa de `5e-3` (impossível: a própria tabela publicava `0,00269`) para **`1e-3`**, que é
`2,7×` abaixo do piso e `1000×` acima da barra de aceitação. O gate ganha **piso de população**:
menos de `8` células discriminantes **reprova**.

### Achado 3 — o quadro de fixtures

**Derivado do directório** (`ls <pasta> | grep -c '\.gz$'`), com a regra escrita ao lado:
`lei 14 · lados 14 · inversao 3 · firmeza 12 · inercias 5 · amostragem 7 · corte 26 ·
superficies 11 · cadeia 8` = **100**. A 1.ª redacção somava `99` e errava a família do corte.

### Depois da emenda

- **Filtragem §4.3 re-executada** sobre o texto novo em 2026-09-16 (texto novo é texto por
  auditar, mesmo quando a emenda é factual): toda frase acrescentada descreve **o que o programa
  faz** ou **o que o nosso código faz**, e todo número novo é medição datada de 2026-09-16 sobre
  as fixtures publicadas e o nosso próprio código.
- **Sweep verde** sobre a espec, este ledger, o `README.md` da pasta e as 100 fixtures, com o
  **controlo positivo** a continuar a disparar sobre a redacção reprovada.
- ⚠️ **Nota de higiene do INC-R1 aceite pelo E:** nesta obra o único ficheiro com a vassoura em
  claro viveu em `~/Referencias/blender-trim-pincel/notes/`, **dentro** da zona contaminada;
  verificado que nada foi escrito no scratchpad nem em `/tmp`.

⏳ **R-pré, 2.ª passagem: pendente. A janela I continua FECHADA.**

---

## R-PRÉ — 2.ª PASSAGEM (2026-09-16) — veredito: ⚠️ **NÃO atestado.** A parede do §4.2 continua LIMPA; ficam **6 achados**, `4` que bloqueiam

> Subagente **novo**, contexto independente do E **e** do R-pré da 1.ª passagem (§3.R: *auditar a
> própria filtragem é o que falha, e re-auditar pelo mesmo contexto herda os pontos cegos dele*).

### 1. As três curas da 1.ª passagem — CONFERIDAS contra as redacções reprovadas (`git show 41cd08eff`)

| cura | conferida como | veredito |
|---|---|---|
| **1 — a lei NOSSA re-derivada** | li o código vivo em vez de aceitar a emenda: o estimador dos quatro verbos de plano pesa pelo valor livre da **máscara** (`1 − máscara`, logo pegada sem máscara ⇒ pesos todos `1`) e o divisor é a **soma dos pesos**, que aí é a contagem ⇒ **média aritmética simples**, tal como a emenda diz; a normal é a soma dos vectores dos vértices pesada igual e depois normalizada. A **extensão** é a consulta do dab, e ela devolve **exactamente o raio** para os quatro — os dois factores próprios que existem pertencem mesmo a outros verbos (a faixa e o campo elástico), como a emenda escreve. A data **2026-08-11** é a do commit `88966645d`, que é onde o estimador passou a chamar os kernels portados | ✅ **exacta** |
| **2 — a barra do gate discriminante** | recalculei o piso sobre **todas** as células publicadas: as `18` leituras das três candidatas erradas nas **6** colunas discriminantes da tabela são todas `≥ 0,00269`, e o piso é mesmo `0,00269` (célula do raio de amostragem a `2R`). A barra `1e-3` fica `2,69×` abaixo dele e `1000×` acima da de aceitação ⇒ **o gate PODE passar**, ao contrário do que a 1.ª redacção publicava | ✅ o **número** está certo · ⛔ a **população declarada** não (achado 3) |
| **3 — o quadro de fixtures** | derivado outra vez do directório: `lei 14 · lados 14 · inversao 3 · firmeza 12 · inercias 5 · amostragem 7 · corte 26 · superficies 11 · cadeia 8` = **100**, e `find -name '*.gz' \| wc -l` = **100**. Bate família a família | ✅ **exacta** |

⛔ **E não parei nelas** (a lição desta pasta: *uma emenda cura o que o auditor NOMEOU, e um achado
substancial sobrevive por ninguém lhe apontar o dedo*). O documento foi varrido inteiro pela
pergunta da §4.3.1, por leitura **e** por varredura textual — e **3 dos 4 bloqueadores abaixo estão
em texto que a emenda NÃO tocou**, que é exactamente a forma prevista.

### 2. A parede (§4.2) — LIMPA, item a item, re-auditada do zero

| item do §4.2 | resultado desta passagem |
|---|---|
| texto de código, trechos, diffs | **nenhum** |
| nomes internos | **nenhum** — extraí **todos** os *code spans* do documento e classifiquei um a um: são identificadores **nossos** (crate, ficheiro, verbo, predicado), nomes de **fixtura nossa** (em português), nomes de **gate propostos por nós**, variáveis **locais desta espec** (os dois raios, os dois tectos, a altura com sinal, a curva suave), ou factos de interface pública (a tecla modificadora). **Zero** identificadores do alvo |
| comentários do original | **nenhum** |
| wording de manual verbatim ou quase | **nenhum**. Varri o documento por vocabulário de estrutura de código (laço · função · variável · ficheiro · sinalizador · chamada · retorno · ramo) e os únicos acertos são usos **nossos** da palavra (método no sentido matemático, assinatura no sentido de impressão digital) |
| tabela verbatim / LUT | **nenhuma** — toda tabela do documento é medição nossa ou catálogo de fixtures nossas |
| organização transcrita | **nenhuma** — a espec descreve por fases cuja ordem é **forçada por dependência de dados** |
| pseudo-código espelhado | **não** — os **11** blocos cercados são 1 a 8 linhas de **fórmula** em português, sem controlo de fluxo, sem declaração e sem nome do alvo; cabem no §4.1.10. ⭐ E num deles a espec apresenta a forma condicional e **manda escrevê-la incondicional**, o oposto de transcrever |

⇒ ⭐ **Os 6 achados são de EXACTIDÃO e PROVENIÊNCIA; nenhum reabre a §4.3 nem obriga a re-filtrar.**

### 3. O sweep, e o CONTROLO POSITIVO em **SETE** canais (dois novos)

`bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_blender-trim-pincel.txt` sobre a espec
+ as **100** fixtures + o `README.md` da pasta + este ledger + o INBOX ⇒ **✓ limpo, 321 entradas,
`exit 0`**. E `--git-history` sobre `docs/3D/cleanroom/` ⇒ **✓ limpo**.

⛔ **Canários plantados em `~/Referencias/<alvo>/`** — ⚠️ **não no scratchpad**, que é o que o
INC-R1 registou; apagados no fecho. Cada um provado a disparar:

| canal | acusado? |
|---|---|
| NOME de ficheiro | ✅ `exit 1` |
| conteúdo de texto, numa linha | ✅ `exit 1` |
| texto com as quebras de linha desfeitas | ✅ `exit 1` |
| idem, com ênfase (`**…**` e `` `…` ``) | ✅ `exit 1` |
| **dentro de um `.gz`**, numa linha | ✅ `exit 1` |
| ⭐ **dentro de um `.gz`, DOBRADO** (o canal que a 1.ª passagem achou cego) | ✅ `exit 1` — **a cura de `fc7014427` está VERIFICADA** |
| ⭐ **controlo NEGATIVO da mesma forma em texto** (o par que decide de quem é o resto) | — ver abaixo |

⭐⭐ **E o controlo dos dois lados nomeia o resto com precisão.** Uma frase da vassoura dobrada
sobre duas linhas em que a segunda **repete um marcador de comentário** passa **limpa** — e passa
**igual no ramo de texto e no ramo do `.gz`** (`exit 0` nos dois). ⇒ *não é resíduo da cura do
`.gz`: é uma propriedade simétrica do normalizador*, que apaga três marcadores e não esse. A
distinção importa porque a 1.ª passagem só podia ver um lado. ⚠️ **E ela não morde este corpus:**
conferido que **todas** as linhas de comentário das **100** fixtures são `# chave: valor` numa
linha só — não há comentário dobrado em nenhuma.

⭐ Os identificadores em maiúsculas dos cabeçalhos das fixtures foram recenseados outra vez
(`45` cadeias distintas): as que têm forma de API são valores de enumeração **públicos** — a chave
que regenera —, e o resto é vocabulário **nosso** em português. A autorização do §4.1.13 continua
aplicada correctamente.

### 4. ⚠️ HIGIENE — o INC-R1 tem uma SEGUNDA metade, do lado da SAÍDA (registado, não bloqueia)

O *scratchpad* desta janela guarda artefactos de corridas anteriores **desta linha** (não minhas), e
**quatro** das oito vassouras da pasta disparam sobre ele. Triado ficheiro a ficheiro:

- **relatórios de sweep gravados em ficheiro** (`novo.out`, `velho.out`) — por construção citam em
  claro os termos que acusaram;
- **um censo de identificadores** e **cópias de `SPEC_*`** e de fonte **nosso**;
- os **registos do portão** (`portao2.txt`, `portao3.txt`), que disparam sobre **nomes de testes
  NOSSOS** — a classe da entrada larga, não uma fuga.

⇒ **Nenhum fonte do alvo está lá.** Mas a lição do INC-R1 (*o scratchpad é alcançado pela
janela-mãe*) vale para a **SAÍDA** do instrumento tanto como para a entrada dele: a cura de 2026-09-16
fechou o lado da vassoura e o lado do relatório ficou aberto. ⇒ **relatório de sweep vai para
`~/Referencias/<alvo>/`, nunca para o scratchpad.** Não apaguei nada: os registos do portão podem
ser da obra da janela.

### 5. Os 6 achados (entregues como instruções funcionais de reescrita)

#### ⛔ Bloqueiam

1. **A tabela de proveniência ainda publica a barra RETIRADA do gate discriminante.** A §13
   declara-se *"a proveniência de **cada** número"* e continua a listar `5e-3` como barra dele,
   *"derivada de vales MEDIDOS"* — enquanto a §12 a fixa em `1e-3` e diz, na mesma linha, que
   `5e-3` era **impossível**. A emenda mexeu na §12 e deixou a §13 intacta. *Quem tirar a barra da
   tabela que existe para dizer de onde vêm os números escreve a barra que o documento refuta.*
   ⇒ a linha da §13 passa a carregar a barra que a §12 fixa, e a retirada fica **marcada como
   retirada** — a linha imediatamente abaixo já tem essa forma, escrita pela própria emenda.
2. **A §2.6 atribui ao nosso código um ramo que os nossos quatro verbos de plano NÃO têm.** Medido
   no código vivo: a escolha entre ler a superfície congelada e a viva é um `match` de três braços;
   os quatro verbos que ajustam plano caem no braço que lê a **viva incondicionalmente** e nunca
   consultam o interruptor. Quem o consulta são **três outros verbos** — e o comentário do nosso
   próprio ficheiro diz porquê, por escrito: *a regra é da REFERÊNCIA, não do verbo*. A §2.6 agrupa
   pelo eixo errado (a família «de plano») e manda o pincel novo entrar *"como os outros de plano"*
   — que é precisamente o braço onde o interruptor **não é lido**. ⚠️ **É load-bearing:** seguido à
   letra, o pincel novo nasce com esse controlo **inerte**, que é o defeito que o nosso código
   regista ter pago e curado. ⇒ dizer que ele entra no braço que **consulta** o interruptor, ao lado
   dos verbos cuja referência é **este** alvo, e nomear os quatro verbos de plano como o
   **contra-exemplo**, não como o modelo.
3. **A população declarada do gate discriminante não é recuperável da §2.2, e o piso dele reprova
   sobre o que está publicado.** A secção anuncia **11** configurações, nomeia **3** como não
   discriminantes e conclui **8** discriminantes; mas a tabela impressa tem **8 colunas**, das quais
   **2** são das nomeadas não-discriminantes ⇒ só **6** células discriminantes estão na página. O
   gate reprova com menos de `8`. *Quem o construir a partir da §2.2 junta 6 e cai no piso do próprio
   gate* — a mesma forma do achado 2 da 1.ª passagem (uma barra que não pode passar), sobrevivida à
   cura dela. ⭐ As duas discriminantes que faltam são as que variam a **extensão de amostragem da
   normal**, e as fixtures delas **existem**. ⇒ publicar as duas colunas, ou declarar o piso com a
   contagem que a tabela de facto carrega — e, nos dois casos, recalcular o piso com **todas** as
   discriminantes à vista, porque hoje o `0,00269` só é conferível sobre `6`.
4. **Uma barra de gate não tem proveniência nenhuma, e a §13 diz que carrega todas.** O gate que
   afirma que o plano acompanha o cursor limita o afastamento a **um raio do dab** — e a
   justificação escrita ao lado é *"a barra é o defeito público do alvo"*. Mas aquele defeito
   público entrega uma **propriedade** (o plano deixou de seguir o cursor), **não um número**; a
   §13 não lista origem para ele; e é a **única** barra do documento sem uma medição impressa por
   perto de onde a derivar — todas as outras têm. ⚠️ A frase que a acompanha (*"um gate cuja barra
   é a de um defeito real vale mais do que um número escolhido"*) afirma exactamente o que não é
   verdade dela. ⇒ medir a grandeza sobre as fixtures do traço passo a passo, que **já existem**, e
   escrever o número que a medição der com a tabela ao lado — ou declará-la decisão nossa e dizê-lo.

#### ⚠️ Não bloqueiam (exactidão barata)

5. **O ruído de `f32` na escala das fixtures aparece com DOIS valores em duas secções, e a barra de
   paridade é derivada do menor.** A §12 escreve um ULP naquela escala como `2,4e-08` e conclui que
   a barra `1e-6` é `≈ 40 ULP`; na magnitude em causa o ULP é **`2,98e-08`**, o que faz da mesma
   barra `≈ 34 ULP`. A §1, por sua vez, diz que a reprodução bate a saída do alvo *"a `3e-8`"*
   enquanto a §4.1 publica, para as **mesmas** 24 configurações, um pior caso medido de `8,0e-08`.
   A folga da barra sobrevive às duas leituras; o que precisa de cura é o documento imprimir dois
   números para uma grandeza e a derivação citar o menor.
6. **O mecanismo escrito para uma das células não-discriminantes é refutado pela própria linha
   dela.** A §2.2 diz que, com **uma** amostra, as três candidatas *"dão o mesmo ponto"*; a linha ao
   lado imprime duas delas em `−0,00003` e a terceira em `0,00000`. A conclusão funcional (excluir a
   célula) está certa; o mecanismo não — com uma amostra a lei que puxa para o cursor continua a
   diferir pelo peso naquele vértice, e o que torna a célula inútil é o vão **colapsar muito abaixo
   de qualquer barra usável**, não desaparecer.

⏳ **R-pré, 2.ª passagem: NÃO atesta. A janela I continua FECHADA.** ⇒ emenda por subagente-E sobre
os `4` bloqueadores (os `2` restantes são baratos e cabem na mesma emenda), depois **3.ª corrida**
do R-pré sobre o texto novo.

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
