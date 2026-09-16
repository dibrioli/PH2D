# LEDGER de proveniência — clean-room do PINCEL QUE APARA ESFREGANDO (alvo `blender-trim-pincel`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-trim-pincel.md` (append cego).

> ⛔⛔ **REGRA DE INSTRUMENTO, NO TOPO DE PROPÓSITO — leia-a antes de tocar na vassoura:**
> **a vassoura descodifica-se em MEMÓRIA, por cano** (`sed -n 'Np' … | base64 -d`, ou o próprio
> `cleanroom-sweep.sh`, que já a descodifica em memória). ⛔ **Nunca para ficheiro — nem em
> `/tmp`, nem no scratchpad, nem em `/dev/shm`.** E um **relatório de sweep** cita os termos que
> acusaram **em claro**: ele é material do alvo e vive em `~/Referencias/<alvo>/`, nunca no
> scratchpad da janela-mãe.
>
> ⚠️ **Esta regra está aqui em cima porque estava no FIM (INC-R1) e a 3.ª passagem do R-pré
> repetiu o incidente (INC-R2) — ela leu a cura DEPOIS de a corrida começar.** *Inspeccionar a
> vassoura é das primeiras coisas que um R faz; uma cura escrita no fim de um documento é lida
> depois de já não servir.*

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

R-PRÉ: **três passagens**, cada uma por um subagente **novo e independente do E** (§3.R: *auditar
a própria filtragem é o que falha, e re-auditar pelo mesmo contexto herda os pontos cegos dele*) —
1.ª, 2.ª e 3.ª em **2026-09-16**, todas despachadas pela janela
`9f820704-0d7e-4d96-847e-9cd720cbf178`. **Nenhuma atestou.** · R-PÓS: (pendente)

⚠️ **A parede do §4.2 saiu LIMPA nas três**, por três contextos independentes; o que bloqueou foi
sempre **exactidão**, e em cada passagem o achado mais pesado estava em texto que a emenda anterior
**não tinha tocado**.

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

## 2.ª EMENDA DO E — 2026-09-16 (resposta aos 6 achados da 2.ª passagem do R-pré)

⭐ **Confirmado pelo auditor novo e a não mexer:** as três curas da 1.ª passagem estão de pé (ele
**re-derivou duas** em vez de as aceitar), a **parede do §4.2 está limpa** re-auditada do zero, e
o sweep saiu verde com controlo positivo em **sete** canais.

⛔⛔ **A LEI QUE ESTA RODADA PAGOU: três dos quatro bloqueadores tinham a MESMA doença — a 1.ª
emenda mexeu num sítio e deixou o GÉMEO intacto.** É a lei da casa *«uma lei escrita em dois
sítios ainda não é uma lei»*, aplicada a um documento em vez de a código: a barra do gate mudou na
secção dos gates e não na de proveniência; a população do gate mudou no texto e não na tabela que
a fundamenta; o número do ULP mudou na derivação e não nas linhas que o citavam. ⇒ **ao curar um
número numa espec, procure o outro leitor dele antes de fechar.**

| # | achado | cura |
|---|---|---|
| 1 | a tabela de proveniência publicava a barra **retirada** do gate discriminante como se fosse derivada de vales medidos, enquanto a secção dos gates já fixava a nova | a linha passa a carregar a **barra em vigor** (`1e-3`, com a derivação), e a retirada ganha a forma **«o que SAIU»** da linha de baixo, com o motivo: era **impossível** sobre o corpus publicado |
| 2 | **BLOQUEADOR de conteúdo**, o mais pesado: a §2.6 atribuía à nossa casa um ramo que os quatro verbos de plano **não têm**. Eles lêem a superfície **viva incondicionalmente** e nunca consultam o interruptor de acumular; quem o consulta são **três outros verbos**, cuja referência é este mesmo alvo. A espec agrupava pelo eixo *«ajusta um plano?»* em vez de *«de que referência veio?»*, e mandava o pincel novo entrar «como os outros de plano» — o braço onde o interruptor fica **INERTE**, defeito que a nossa casa já pagou e curou | §2.6 reescrita com os **dois braços** em tabela, o verbo novo no que **CONSULTA** o interruptor, os quatro verbos de plano nomeados como **CONTRA-EXEMPLO**, e o preço do braço errado com os números que a nossa casa mediu (`1,04×` / `0,99×` / `1,00×` contra `1,74×`) |
| 3 | a população declarada do gate discriminante não era recuperável da página: **11** anunciadas, **3** nomeadas não-discriminantes, **8** concluídas — mas a tabela tinha **8 colunas das quais 2 eram das nomeadas** ⇒ só **6** na página. As duas em falta eram as que variam a extensão de amostragem da **normal**, e as fixtures delas já existiam | a tabela da §2.2 está **TRANSPOSTA**: uma linha por célula, as **11** à vista, mais a coluna **`min \|errada\|`** e a coluna **`discrimina?`**. O piso (`0,00269`) e a contagem (`8`) passam a ser **lidos da página**, e o gate aponta para as duas colunas |
| 4 | a barra do gate que afirma que o plano segue o cursor era a **única sem proveniência**: `1` raio, justificada com o defeito público do alvo — que entrega uma **propriedade**, não um número | **medida** sobre as `11` células: `0,0000 … 0,4947` raios, mediana `0,1649`. A barra passa a **`0,75 R + \|deslocamento\| · R`**, com o segundo termo **exacto** (§2.4) e o `0,75` declarado em voz alta como **decisão nossa** (`1,5×` o máximo medido, porque um corpus de um dab efectivo não limita um traço longo) |
| 5 | o ruído de `f32` aparecia com **dois** valores, e a barra era derivada do menor ⇒ a folga lia-se `40 ULP` quando é `34` na escala do raio e **`8,4`** na maior coordenada das fixtures; e uma secção dizia `3e-8` onde outra publicava `8,0e-08` para as **mesmas** 24 configurações | derivação da §12 vira **tabela** com as duas escalas e o pior resíduo medido; a §1 e o G-1 passam a citar `8,0e-08` e `8,4 ULP` |
| 6 | o mecanismo de duas células não-discriminantes era **refutado pela linha delas**: dizia que com uma amostra as candidatas *«dão o mesmo ponto»*, e a linha imprimia duas num valor e a terceira noutro | corrigido: as duas de `R_c` **colapsam sobre a própria amostra** e a do alvo fica **entre** a amostra e o cursor ⇒ o vão **COLAPSA para `0,00003`**, três ordens abaixo de qualquer barra usável, e **não desaparece**. A conclusão (excluir a célula) **mantém-se** |

### Depois da 2.ª emenda

- **Filtragem §4.3 re-executada** sobre o texto novo em 2026-09-16: toda frase acrescentada
  descreve **o que o programa faz**, **o que o nosso código faz** ou **de onde sai um número**;
  os números novos (`0,4947 R`, `0,1649 R`, `1,1921e-07`, `2,9802e-08`, `8,4 ULP`) são medições
  datadas desta jornada sobre as fixtures publicadas e sobre o nosso próprio código.
- **Sweep verde** com o instrumento de hoje, e o **controlo positivo** a continuar a disparar
  sobre a redacção reprovada.
- ⚠️ **Higiene aceite nas duas metades:** a vassoura descodifica-se **em memória** e um relatório
  de sweep — que cita **em claro** os termos que acusaram — nunca é gravado no scratchpad da
  janela-mãe. Nesta obra nenhum relatório foi gravado em ficheiro; a saída do sweep viveu só no
  terminal, e o único ficheiro com a vassoura em claro vive dentro de `~/Referencias/`.

⏳ **R-pré, 3.ª passagem: pendente. A janela I continua FECHADA.**

---

## R-PRÉ — 3.ª PASSAGEM (2026-09-16) — veredito: ⛔ **NÃO atestado.** A parede do §4.2 continua LIMPA; ficam **2 achados que BLOQUEIAM** e **9 erratas**

> Subagente **novo**, contexto independente do E e das duas passagens anteriores. Corrido sob a
> regra nova desta rodada: **a PAREDE e a EXACTIDÃO separam-se, e só a parede — mais o achado de
> exactidão que faz nascer produto errado ou gate impossível — bloqueia.**

### 1. As seis curas da 2.ª emenda — CONFERIDAS, e **quatro RE-DERIVADAS do zero**

| cura | como a conferi | veredito |
|---|---|---|
| **1 — a §13 passa a carregar a barra em vigor** | li as duas linhas: a `1e-3` com a derivação, e a retirada marcada como **«o que SAIU»** com o motivo | ✅ exacta |
| **2 — a §2.6 reescrita com os braços** | li o **código vivo** em vez de aceitar a emenda. O `match` põe os quatro verbos de plano no braço que lê o **vivo** e nunca consulta o interruptor; os três cuja referência é este alvo consultam-no. As duas linhas impressas estão certas | ✅ o conteúdo · ⚠️ a **contagem** não (errata E3) |
| **3 — a tabela da §2.2 transposta** | ⭐⭐ **re-derivei a tabela INTEIRA das fixtures publicadas** — as `11` células, as quatro candidatas, por ajuste do plano aos vértices tocados (o método da §4.1). **Bate a todos os dígitos impressos, célula a célula**, incluindo o piso `0,00269` e as `8` que discriminam. O `#` de amostras confirma o mecanismo das três mudas: `1` vértice nas duas de `R_c`, e as quatro candidatas a coincidir em `0,00000` nas bossas | ✅ **exacta** |
| **4 — a barra do G-6 medida** | ⭐⭐ **re-derivei-a**: `0,0000 … 0,4947` raios, **mediana `0,1649`**, e o máximo é mesmo na célula de menor raio. A parte que é decisão nossa está declarada | ✅ **exacta** |
| **5 — o ULP** | refiz a aritmética: `ULP(1,0) = 1,1921e-07` e `ULP(0,4) = 2,9802e-08`; `1e-6` são `8,39` e `33,6` ULP; `8,0e-08` são `0,67` ULP; `1e-6/8,0e-08 = 12,5` | ✅ exacta |
| **6 — o mecanismo das células de uma amostra** | medido: com `1` vértice as duas candidatas de `R_c` colapsam sobre a amostra e a do alvo fica entre ela e o cursor ⇒ o vão **colapsa** para `0,00003`, não desaparece | ✅ exacta |

⭐ **E re-derivei, ao dígito, muito para além das seis:** a tabela de resíduos e as populações
tocadas da §4 (`265`/`274`/`271`/`287`/`62`/`632`) · a força ao quadrado (mediana `0,250000`, `min
0,249999`, `max 0,250001`) · os tectos da §3.1 (`0,0460`/`0,1316`/`0,2278`/`0,2278`) · o
deslocamento da §2.4 (`±0,08000`, razão `0,2000`, normais a `0,0000°`) · as duas leis do `Ctrl`
(`0,000e+00` e `7,041e-02`) · a queda da §2.3 (`0,000e+00` e `5,595e-02`) · as três inércias
(`0,000e+00`) · os quatro pares mortos da §7 e as duas famílias vivas · os desvios angulares da
§0.2(C) (`0,00°`/`16,9°`/`23,5°`/`31,2°`) · e o `0,00575°` da §4.1.

⛔ **E não parei nelas.** O documento foi varrido inteiro pela pergunta da §4.3.1, por leitura e por
varredura textual independente — **os dois bloqueadores estão em texto que NENHUMA das duas emendas
tocou**, que é a forma que esta pasta já previu duas vezes.

### 2. A parede (§4.2) — LIMPA, item a item, re-auditada do zero

| item do §4.2 | resultado desta passagem |
|---|---|
| texto de código, trechos, diffs | **nenhum** |
| nomes internos | **nenhum** — extraí os **139** *code spans* não-numéricos e classifiquei-os um a um: crates/ficheiros/verbos/predicados **nossos**, nomes de **fixtura nossa** em português, nomes de **gate propostos por nós**, variáveis **locais desta espec** (os dois raios, os dois tectos, a altura com sinal, a curva suave) e um facto de interface pública (a tecla modificadora). **Zero** do alvo |
| comentários do original | **nenhum** |
| wording de manual verbatim ou quase | **nenhum** — varri por vocabulário de estrutura de código com uma rede de 30 termos; os **5** acertos fora do cabeçalho são usos **nossos** (a palavra «declara» sobre um gate nosso, «assinatura» no sentido de impressão digital, «método» no sentido legal do §4.1.12, «chamam» sobre os nossos verbos, e a nota da §3.3 que manda escrever **sem** caso especial) |
| tabela verbatim / LUT | **nenhuma** — toda tabela é medição nossa ou inventário de fixtures nossas, e **re-derivei a maior delas do corpus** |
| organização transcrita | **nenhuma** — a ordem das fases é forçada por dependência de dados |
| pseudo-código espelhado | **não** — os **12** blocos cercados têm 1 a 10 linhas de **fórmula** em português, sem controlo de fluxo, sem declaração, sem nome do alvo |

⇒ ⭐ **A parede está limpa pela terceira vez, e por três contextos independentes.**

### 3. O sweep, e o CONTROLO POSITIVO em **NOVE** canais

`cleanroom-sweep.sh` sobre a espec + as **100** fixtures + o `README.md` da pasta + o INBOX + este
ledger ⇒ **✓ limpo, 321 entradas, `exit 0`**; e `--git-history` sobre `docs/3D/cleanroom/` ⇒ **✓
limpo, `exit 0`**.

⛔ **Canários plantados em `~/Referencias/<alvo>/`** (nunca no repo, nunca no scratchpad — a lição
do INC-R1 nas duas metades), apagados no fecho:

| canal | acusado? |
|---|---|
| NOME de ficheiro | ✅ `exit 1` |
| conteúdo de texto, numa linha | ✅ `exit 1` |
| texto com as quebras de linha desfeitas | ✅ `exit 1` |
| idem, com ênfase | ✅ `exit 1` |
| dentro de um `.gz`, numa linha | ✅ `exit 1` |
| ⭐ **dentro de um `.gz`, DOBRADO** | ✅ `exit 1` — **a cura de `fc7014427` está VERIFICADA** |
| ⭐ dentro de um `.gz`, dobrado **com ênfase** (canal novo) | ✅ `exit 1` |
| ⭐ **gémeo em TEXTO da mesma forma** (canal novo — o par que prova a simetria dos dois ramos) | ✅ `exit 1` |
| controlo NEGATIVO já triado (2.ª linha repete o marcador de comentário) | limpo nos dois ramos, como a 2.ª passagem registou |

⚠️ **E o `exit 2` foi discriminado do `exit 1` em TRÊS formas** (sem paths · vassoura inexistente ·
path inexistente), todas `exit 2`. *A 2.ª passagem perdeu tempo com um laço que os colapsava.*

⛔⛔ **ACHADO DE INSTRUMENTO, e ele quase me fez registar uma regressão que não existe: o meu
PRIMEIRO canário do canal do `.gz` dobrado leu LIMPO — porque eu o escrevi na forma óbvia, uma
linha de comentário de fixtura, e a 2.ª metade começava com o marcador `#` repetido.** Essa é
exactamente a forma que a 2.ª passagem já triou como **isenção simétrica do normalizador** (ele
apaga três marcadores e não esse). ⇒ *um canário para o canal dobrado que repita o marcador de
comentário na 2.ª linha não testa a cura — testa a isenção, e devolve verde*. Reconstruído sem o
marcador, o canal dispara, e o gémeo em texto dispara igual. **Fica escrito para a 4.ª passagem.**

### 4. ⛔⛔ Os DOIS achados que BLOQUEIAM (instruções funcionais de reescrita)

#### B1 — **a barra do G-1 não pode passar sobre a população que o próprio G-1 declara**

O gate exige `max|Δ| ≤ 1e-6` sobre *«as **25** fixtures de `lei/` e `lados/`»*, e a §4.1 publica,
para essas **mesmas** 25, que **1 delas lê `3,2e-05`** — **`32×` a barra**. O §12 chega a dizê-lo na
frase da derivação (*«fica duas ordens de grandeza abaixo do único desvio que a espec regista como
divergência»*), e o quadro do gate não o absorveu. ⇒ **quem construir o G-1 como está escrito
reprova numa fixtura que a espec já sabe que diverge.**

⚠️ **E a população não é recuperável da página:** as duas pastas carregam **28** ficheiros (`14` +
`14`, contados), dos quais **2 não movem nada** — e esses dois são justamente as fixtures do **G-2**
e do **G-8** — e **26** movem. A página nunca diz quais são as 25. É a **terceira** aparição da
mesma espécie nesta obra (a barra impossível da 1.ª passagem, a população irrecuperável da 2.ª),
desta vez num gate que nenhuma emenda tocou.

⭐ **A causa da divergência confirmei-a eu:** o ângulo entre a candidata de normal do alvo e o plano
recuperado da própria saída dele é `0,006°` naquela fixtura e `0,000°` nas outras dez — que é
exactamente o `0,00575°` que a §4.1 publica, e é a ambiguidade da normal num quadrilátero empenado
que a espec já explica e que **a nossa malha de triângulos não produz**.

⇒ **Cura funcional:** o G-1 declara a população que a página sustenta — nomeia as duas que não
movem nada (e diz que são as fixtures do G-2 e do G-8), nomeia a que diverge e diz porquê, e **ou**
a exclui pelo nome **ou** a carrega com a barra própria dela. E ganha **piso de população**, como o
G-5 já tem, para que encolher o corpus em silêncio não o branqueie.

#### B2 — **o cabeçalho de proveniência das fixtures não carrega todas as grandezas que enquadram o traço, e a §11 afirma que carrega**

A §11 escreve: *«cabeçalho de proveniência (**todas as grandezas que enquadram o traço**, uma por
linha)»*. Censo sobre as **100**, agrupando por (as **35** chaves do cabeçalho + o bloco de repouso
+ o bloco dos cursores): existe **um grupo com enquadramento publicado byte-idêntico e TRÊS saídas
diferentes** —

- quatro fixtures de três famílias diferentes partilham a saída (e uma delas é o controlo de
  determinismo, `0,000e+00`);
- a do modo **afastar** difere delas por **`7,041e-02`**;
- a da **máscara** difere por **`4,673e-02`** (move `327` em vez de `560`).

⇒ **duas grandezas não viajam:** (a) **se o modificador estava carregado** — o cabeçalho até tem a
chave do sentido do gesto, e ela traz **o mesmo valor** na fixtura invertida e na base; (b) **a
máscara** — não há chave nenhuma nem bloco que a carregue, logo *que metade foi mascarada* não está
no ficheiro. A segunda fixtura do modo de inversão sofre do mesmo (o cabeçalho dela declara o par de
tectos **não trocado** e a saída é a do par trocado).

⚠️ **Porque isto faz nascer produto errado:** a única forma sã de correr 100 fixtures é um leitor
guiado pelo cabeçalho. Esse leitor reproduz a fixtura do modo afastar **sem inversão** e a da
máscara **sem máscara**, e devolve desacordos de `7,0e-02` e `4,7e-02` contra uma lei que está
**certa** — e o Implementador, a quem a §11 garantiu que o cabeçalho é completo, não tem como ver o
buraco: ele persegue um fantasma e mexe numa lei correcta. O **G-9** é declarado sobre essa família
e exige **igualdade exacta**.

⇒ **Cura funcional:** o cabeçalho passa a carregar o estado do modificador e a máscara (esta como
bloco próprio, ao lado dos que já existem) — **ou** a §11 diz, com todas as letras, que o cabeçalho
**não** é completo, nomeia as **três** fixtures cujo enquadramento vive só na prosa, e diz onde cada
grandeza em falta está escrita.

### 5. ⚠️ As NOVE erratas (gaveta B — não bloqueiam; a janela I corrige-as enquanto implementa)

| # | sítio | o que está errado, e o que fica de pé |
|---|---|---|
| **E1** | **§0.2, linha (G)** | *«o nosso `invert` só troca o sinal»* é falso nas duas direcções para esta família: o predicado que decide quem honra o modificador **não lista** três dos quatro verbos de plano (o modificador nunca lhes chega), e no quarto ele **não é um sinal** — baixa o plano **e** inverte o lado, com as duas metades a viajar no mesmo factor. A conclusão da célula (não sabemos exprimir *trocar os tectos*) **fica**. ⭐ E a correcção **paga**: o nosso código carrega três recusas escritas para *«fazer o invert funcionar no achatar»*, e uma delas morre em *«ele tem UM utensílio com um interruptor e nós temos DOIS verbos com dois chips»* — premissa que **dissolve** para este pincel, que é precisamente um utensílio com um interruptor |
| **E2** | **§0.2, linha (E)** | *«o nosso `accumulate` decide de que superfície o plano é lido»* é o **gémeo** do bloqueador que a 2.ª emenda curou na §2.6, deixado intacto — para os quatro verbos de plano o interruptor **não** decide isso. A conclusão (o nosso acumular não é um knob de memória) **fica**. *A lei que a própria 2.ª emenda escreveu — «ao curar um número numa espec, procure o outro leitor dele» — não foi aplicada a esta célula* |
| **E3** | **§2.6** | *«o nosso estimador tem **dois** braços»* — o `match` vivo tem **três**: o terceiro lê a superfície **congelada incondicionalmente** e tem um verbo lá dentro. As duas linhas impressas estão certas e a instrução (o pincel novo entra no braço que consulta o interruptor) não muda |
| **E4** | **§9, 1.ª linha** | as amostras: `1 131` na fracção `2,0` está **certo**, mas o `61` com que é comparado é a contagem na fracção **de fábrica `0,5`**, não em `0,25` — em `0,25` a contagem é **`1`**, que é exactamente o que a §2.2 publica para essa célula duas secções acima. E as duas contagens estão em razão de **`18,5×`**, não *«`4×` a área»*: o `4×` é a razão das fracções, ou seja dos **raios** |
| **E5** | **§13 (o tecto `20`) e §12 G-14** | o `20` é facto de comportamento lido do alvo (legítimo), mas *«confirmado pela saturação medida»* **não tem corpus**: o traço mais longo de todo o corpus publicado tem **8 dabs** (`74` fixtures a 8, uma a 6, duas a 4, uma a 3, `21` a 2, uma a 1), e um tecto de 20 é inobservável abaixo de 20. O G-14 conta o nosso próprio anel, logo passa; o que falta é a testemunha do oráculo |
| **E6** | **§12 G-13 e §13** | a barra sai de *«a menor mudança medida num knob VIVO do alvo foi `1,1e-02`»*, que é o deslocamento no pincel de tiras — e **não existe fixtura publicada** para esse par (a pasta do corte tem `26` ficheiros e nenhum deles). A barra `1e-4` **sobrevive** no que está publicado: a menor mudança viva que reproduzi é `7,3e-03` |
| **E7** | **§7.3, coluna «prova»** | *«seis valores de limiar (`desligado`, `0,05`, `0,1`, `0,3`, `0,5`, `1,0`)»* para o barro — a pasta publica **cinco** (não há o `0,3`); e *«idem, as seis byte-idênticas»* para o polegar, que publica **duas**. Todas as publicadas são byte-idênticas, logo a conclusão (os dois mortos) **fica** |
| **E8** | **§6.2, tabela** | ela publica **nove** células; o corpus carrega **seis** — a linha do degrau tem as quatro fixtures, e as linhas da rampa e das bossas só têm o `0` e o `1,0`. As duas metades do G-12 estão inteiramente cobertas, logo o gate não é afectado |
| **E9** | **§8, última linha** | a linha da pegada projectada dá o **mecanismo** e cala o **resultado medido**: a fixtura publicada move **`0` de `2 401`** vértices. É um **segundo** caminho para o pincel ficar inerte, ao lado dos dois tectos a zero da §3.1, e o único que a §8 não nomeia |

### 6. Higiene

Nenhum relatório de sweep foi gravado no scratchpad da janela-mãe: todos os ficheiros de trabalho
desta passagem (sondas, saídas do sweep, canários) viveram em `~/Referencias/<alvo>/rpre3/`, e os
canários foram apagados no fecho. A vassoura foi descodificada **só em memória, por cano**, para
construir os canários — nunca para ficheiro (a cura do INC-R1, aplicada).

⏳ **R-pré, 3.ª passagem: NÃO ATESTA. A janela I continua FECHADA.** ⇒ 3.ª emenda por subagente-E
sobre os **2** bloqueadores (as **9** erratas cabem na mesma emenda), depois **4.ª corrida** do
R-pré.

## 3.ª EMENDA DO E — 2026-09-16 (2 bloqueadores + 9 erratas da 3.ª passagem do R-pré)

⭐ **Confirmado pelo terceiro auditor independente:** a **parede do §4.2 está limpa pela TERCEIRA
vez**, sweep verde em todos os canais, e as **seis** curas da 2.ª emenda de pé — **quatro**
re-derivadas do zero em vez de aceites.

⛔⛔ **A LEI, PELA TERCEIRA VEZ: os dois bloqueadores estavam em texto que NENHUMA das duas
emendas tinha tocado.** *Uma emenda cura o que o auditor nomeou.* E duas das nove erratas eram
**gémeas de bloqueadores já curados**, deixadas intactas na **§0.2** — a secção de abertura, que
ninguém relê. ⇒ **a varredura pelo gémeo tem de incluir o resumo, não só a secção técnica.**

### B1 — o gate de paridade não passava sobre a população que ele próprio declarava

Ele exigia `max|Δ| ≤ 1e-6` sobre *«as 25 fixtures»* de `lei/` + `lados/`, e a §4.1 publica, para
essas mesmas, uma que lê **`3,2e-05` — `32×` a barra**. A §12 até o dizia na frase da derivação e
**o quadro do gate não o absorveu**.

**Contado do directório**, e agora na página: as duas pastas carregam **`28`** ficheiros.

| grupo | contagem | o que é |
|---|---|---|
| não movem nada | **`2`** | `lei/lei_primeiro_dab` e `lados/traco_inerte` — os sujeitos de **G-2** e **G-8** |
| traços inteiros de 8 dabs | **`7`** | `lados/traco_*` menos o inerte — a reprodução é do **primeiro dab efectivo** e não os alcança |
| **um dab efectivo** | **`19`** | `13` de `lei/` + `6` de `lados/lados_*` — **a população do G-1** |
| ⇒ na barra comum | **`18`** | pior resíduo medido `7,96e-08` |
| ⇒ com barra PRÓPRIA | **`1`** | `lei/lei_bossas`, `1e-4` (`3,1×` o `3,2e-05` medido), com a razão: ali a malha tem **quadriláteros empenados** e a normal de um vértice é escolha **a montante** desta lei. ⭐ *a nossa malha é de triângulos e não produz a ambiguidade* |

**+ piso de população (`19`)**, como o G-5 já tinha, e os `7` traços inteiros ficam como **dívida
nomeada** num gate próprio que nasce com quem os cobrar (**G-1b**).

### B2 — o cabeçalho das fixtures não era completo, e a §11 dizia que era

Censo sobre as 100 agrupando pelo enquadramento publicado: **um grupo com enquadramento
byte-idêntico e TRÊS saídas diferentes**. Não viajavam: o **estado do modificador** (a chave do
sentido do pincel traz o **mesmo** valor na invertida e na base — *são coisas diferentes*) e a
**máscara**, que não tinha chave nem bloco. ⚠️ O E achou ainda uma **terceira**: a **direcção da
vista**, que a lei consome **duas** vezes (o filtro de faces viradas ao observador e a escolha da
metade da pegada, §2.5).

⇒ ⭐ **cura pela via forte: as 100 fixtures foram REGERADAS** (o gerador vive fora da árvore; o
enquadramento em falta estava nas configurações das corridas, que também vivem lá).

| chave nova | quantas fixtures enquadra |
|---|---|
| `modificador_carregado` | **`5`** — as `3` de `inversao/` e as `2` de `corte/corte_laminav_din_inv_*` |
| `mascara` + bloco `m` por vértice | **`1`** — `superficies/superficie_mascara` (`1 176` travados de `2 401`) |
| `direccao_da_vista` | **todas as 100** |

⚠️ **Sem isto, o leitor guiado pelo cabeçalho — a única forma sã de correr 100 fixtures —
reproduzia duas sem inversão e uma sem máscara, e lia desacordos de `7,0e-02` e `4,7e-02` contra
uma lei CERTA.** *Uma fixtura cujo enquadramento vive na prosa é uma fixtura que o gate seguinte
vai ler ao contrário.*

### As nove erratas, cada uma com o que ficou escrito

| # | errata | o que passou a dizer |
|---|---|---|
| 1 | §0.2(G): *«o nosso `invert` só troca o sinal»* — ⛔ **gémeo de um bloqueador já curado** | o predicado vivo **não lista** três dos quatro verbos de plano (nos três o modificador é **inerte**) e no quarto ele **baixa o plano E inverte o lado** |
| 2 | §0.2(E): repetia o erro da §2.6 que a 2.ª emenda curou noutro sítio — ⛔ **o outro gémeo** | nos quatro verbos de plano o interruptor de acumular **nem é consultado** |
| 3 | a §2.6 dizia *«dois braços»* | o `match` vivo tem **três** — o terceiro lê o congelado **incondicionalmente**, e o interruptor deixou de lhe ser oferecido de propósito |
| 4 | a §9 rotulava `61` amostras como a contagem de `0,25` e chamava **razão de área** a uma razão de raios | a fábrica é `0,5` e amostra `61`; a `2,0` amostra `1 131`; o **raio** é `4×` e a **contagem** `18,5×`; a `0,25` a contagem é **`1`** |
| 5 | o tecto `20` dizia-se *«confirmado pela saturação medida»* | ⏳ **não está**: o traço mais longo do corpus tem **`8`** dabs — facto lido do alvo, **dívida nomeada** |
| 6 | a barra do **G-13** vinha de `1,1e-02`, **sem fixtura publicada** | passa a vir de **`7,3e-03`**, medido entre **fixtures publicadas** (`corte/corte_tiras_05` contra `_desligado`) |
| 7 | a §7.3 prometia **seis** limiares onde a pasta publica cinco e dois | diz agora quantos foram **medidos** e quantos têm **fixtura** |
| 8 | a §6.2 publicava **nove** células sem dizer quantas o corpus sustenta | **`5` de `9`** são recuperáveis, e a tabela diz **quais** |
| 9 | a §8 dava o mecanismo da pegada projectada e calava que a fixtura move **`0` de `2 401`** | diz-o, com o mecanismo — e nomeia-a como a **quarta** fixtura inerte do corpus e a **única cuja inércia não é a lei a funcionar** |

### Depois da 3.ª emenda

- **Filtragem §4.3 re-executada** sobre o texto novo em 2026-09-16.
- **Sweep verde** sobre a espec e as 100 fixtures **regeradas**; controlo positivo re-corrido.
  ⭐ **Sobre a armadilha do canal «dobrado dentro de um `.gz`» que o R nomeou:** com o instrumento
  de hoje, os **três** canários disparam — o **dobrado sem o marcador na 2.ª linha** (o que testa
  o canal), o **simétrico** (o que testava a isenção) e o de texto simples. ⇒ *o buraco que a
  passagem anterior mediu está fechado neste instrumento.*
- ⚠️ **Higiene:** nenhum relatório de sweep foi gravado em ficheiro (a saída viveu só no terminal),
  e a vassoura descodifica-se em memória.

⏳ **R-pré, 4.ª passagem: pendente. A janela I continua FECHADA.**

## R-PRÉ — 4.ª PASSAGEM (2026-09-16) — veredito: ✅ **ATESTADA.** A parede do §4.2 está LIMPA (4.ª vez, contexto independente) e **ZERO achados BLOQUEIAM**. Ficam **6 erratas**, nomeadas abaixo

⚠️ **A regra do §3.R aplicada como está escrita:** a parede é o que o atestado afirma, e ela está
limpa; a exactidão vai para duas gavetas e **nada caiu na que bloqueia**. As três passagens
anteriores tiveram parede limpa e não atestaram, e cada volta custou uma re-emenda de E mais uma
re-auditoria inteira — *esta passagem atesta, e as erratas seguem para a janela I, que as corrige
enquanto implementa.*

### 1. ⛔⛔ PRIORIDADE Nº 1 — a REGENERAÇÃO das 100 fixtures é **ESTRITAMENTE ADITIVA**, provada contra o `git`

O risco mais caro desta obra era uma regeneração mudar valores em silêncio. **Não mudou.** Censo
sobre as **100**, comparando `git show 13b057c64:<path>` (antes) com `1195dad93:<path>` (depois),
descomprimindo as duas e separando cabeçalho de blocos de dados:

| régua | medido |
|---|---|
| ficheiros na pasta, antes e depois | **`100` → `100`**, `100` `M`, **zero** `A`, **zero** `D` |
| **saídas medidas idênticas ao BIT** (blocos `r`, `n`, `s`, `c`, na mesma ordem) | ⭐ **`100` de `100`** |
| chaves do cabeçalho antigo **perdidas** | **`0`** |
| chaves do cabeçalho antigo com **valor mudado** | **`0`** |
| chaves **novas** | `3` — `modificador_carregado`, `mascara`, `direccao_da_vista`, cada uma em **`100` de `100`** |
| blocos de dados **novos** | `1` — o bloco `m` (`2 401` linhas) em **uma** fixtura, `superficies/superficie_mascara` |

⇒ **a diferença é *só* as grandezas novas.** A prova é a forte: filtrando da versão NOVA os
prefixos de bloco que a ANTIGA não tinha, a sequência de linhas de dados é **idêntica**, fixtura a
fixtura.

⭐ **E o censo do enquadramento confirma que a cura B2 fez o que prometeu**, re-derivado do zero:
agrupando as 100 por (cabeçalho de enquadramento + malha de repouso + normais + pontos do cursor),

| | grupos ambíguos | fixturas envolvidas | saídas distintas no grupo |
|---|---|---|---|
| **antes** (`13b057c64`) | **`1`** | `6` | **`3`** |
| **depois** (`1195dad93`) | ⭐ **`0`** | `0` | — |

e os **dois** desacordos que um leitor guiado pelo cabeçalho antigo lia reproduzem-se ao dígito
publicado: **`7,0410e-02`** (§11 publica `7,0e-02`) e **`4,6734e-02`** (publica `4,7e-02`).
Os `90` enquadramentos distintos passaram a `92` — as três chaves novas partem exactamente o grupo
de 6 em três.

⚠️ **E as contagens da tabela da §11 batem com o directório:** `5` com o modificador carregado,
`1` com máscara, `100` com direcção da vista.

### 2. A parede (§4.2) — LIMPA, item a item, re-auditada do zero

| item do §4.2 | como foi auditado | resultado |
|---|---|---|
| texto de código / trechos / diffs | os **`12`** blocos cercados extraídos e lidos um a um | o 1.º é o cabeçalho de proveniência; os outros **`11`** são `1`–`9` linhas de **fórmula em português**, sem sintaxe de linguagem nenhuma, sem laço e sem estrutura de controlo que espelhe fonte. ✅ |
| nomes internos do alvo | **todos** os *code spans* extraídos (`192` não-numéricos), e os `57` com forma de identificador classificados **um a um** | ✅ **zero**: são verbos e ficheiros **NOSSOS** (`brush_verb.rs`, `stroke_plane.rs`, `fit_plane`, `RefMode`, `Verb::BoxTrim`, os 7 verbos da vizinhança), campos **nossos** (`accumulate`, `invert`), nomes de **fixtura nossa**, nomes dos **gates propostos**, ou variáveis locais desta espec (`R`, `R_c`, `R_n`, `LIMITE`, os prefixos de bloco) |
| comentários do original | — | ✅ zero |
| wording de manual/README/discussão, verbatim ou quase | as **`17`** citações entre `«»` extraídas e classificadas | ✅ **todas NOSSAS**: redacções anteriores desta espec citadas como retiradas, a frase do dono, o achado do 3.º auditor, ou paráfrase rectórica em português. **Nenhuma** é texto do alvo |
| tabela verbatim / LUT | todas as tabelas | ✅ são **medições nossas**, cada uma com a fixtura ao lado |
| organização arquivo-a-arquivo / função-a-função | a estrutura §2→§3→§4 | ✅ **fases funcionais** cuja ordem é forçada por dependência de dados (a normal antes do centro, o centro antes do quadro), e em **dois** pontos a espec manda **DIVERGIR** da forma do alvo (§2.6 o braço do interruptor; §3.3 a multiplicação incondicional) |
| pseudo-código espelhado | os 11 blocos | ✅ nível de **paper** (§4.1.10/11): equações e definições por ramos, que é a própria matemática |

⭐ **As FIXTURES são texto NOVO e a parede delas foi auditada pela primeira vez:** as **`38`**
chaves de cabeçalho distintas são todas vocabulário de **domínio em português**; dos valores, os
**`15`** tokens em maiúsculas são **valores de enumeração da API PÚBLICA** do alvo — a chave de
regeneração —, que o **§4.1.13** cobre expressamente (*"nome de API pública … quando ler/escrever o
formato ou ser compatível EXIGE o nome exato, o nome é o próprio fato funcional"*) e que o
cabeçalho da espec **já declara** como exclusão consciente da vassoura. ✅ **Nenhum** tem forma de
nome interno (função, variável, ficheiro de fonte).

### 3. O sweep, e o CONTROLO POSITIVO em **DEZ** canais

⭐ **`bash scripts/cleanroom-sweep-controlo.sh` corrido PRIMEIRO**, como o §3.R agora manda: os
**dez** canais acusam (`exit 1`) — nome de ficheiro · texto numa linha · binário · dentro de um
`.gz` · dobrado com ênfase pelo meio · **dobrado entre linhas de comentário** · o mesmo comprimido ·
dobrado no `.gz` sem marcador · marcador de outra família · e o controlo **negativo** limpo
(`exit 0`) com o **`exit 2`** do uso errado discriminado. ⇒ *nenhum canal está cego; um sweep verde
aqui é prova de filtragem.* ⚠️ A cura do **marcador de comentário no início da linha**
(`f81e377ee`, encomendada pela 3.ª passagem) está **confirmada pelo controlo**, não à mão.

| alvo do sweep | resultado |
|---|---|
| a espec | ✅ `exit 0` |
| as **100** fixtures regeradas | ✅ `exit 0` |
| este ledger + o INBOX + o README da pasta | ✅ `exit 0` |
| `--git-history -- docs/3D/cleanroom/` | ✅ `exit 0` |
| os **9** commits desta obra, mensagem + patch, um a um | ✅ **`0` acertos em cada** |

### 4. As curas da 3.ª emenda — CONFERIDAS, e as que têm número **RE-DERIVADAS do zero**

**B1** ✅ — e a população do G-1 é **recuperável do directório**, contada por mim: as duas pastas
carregam **`28`** ficheiros = **`1`** de um dab + **`19`** de dois dabs + **`8`** de oito dabs.
⇒ um dab **efectivo** = **`19`** (`13` de `lei/` + `6` de `lados/lados_*`), traços inteiros menos o
inerte = **`7`**, não movem nada = **`2`** (verificado: `lei/lei_primeiro_dab` e `lados/traco_inerte`
têm saída **byte-idêntica ao repouso**). **`28 = 2 + 7 + 19`** fecha. A aritmética das barras fecha
também: `1e-6 / 1,1921e-07 = 8,4` ULP · `1e-6 / 2,9802e-08 = 34` · `1e-6 / 7,96e-08 = 12,6` ·
`8,0e-08 / 1,1921e-07 = 0,67` ULP · `1e-4 / 3,2e-05 = 3,1`.

**B2** ✅ — provado na §1 desta passagem, pela via mais forte que havia (aditividade contra o `git`).

**As nove erratas** ✅ todas curadas, e **oito re-derivadas**:

| # | conferência |
|---|---|
| 1 | §0.2(G) lida no **código vivo**: o predicado de inversão lista `Blob, Boundary, Clay, Cloth, Crease, Draw, Inflate, Layer, Mask, MultiplaneScrape, Pose` — **`Flatten`/`Fill`/`Scrape` ausentes** (`0` acertos). E no `Clay` a lei é `d = distância_com_sinal − sinal · raio · fracção`, com a guarda `d · sinal < 0`: com o sinal negativo o plano **baixa** *e* o lado **inverte**. ⭐ A espec descreve-o exactamente |
| 2 e 3 | §0.2(E) e §2.6 lidas no **código vivo**: o `match` tem **três** braços — um verbo lê o congelado **incondicionalmente**, **três** consultam o interruptor, e o `_ =>` (onde caem os **quatro** verbos de plano) lê o **vivo** sem o consultar. ⭐ A tabela de três braços da §2.6 está certa nas três linhas, e a instrução (*o pincel novo vai para o braço que CONSULTA*) segue dela |
| 4 | §9 re-medida na família `lei/lei_area*`: fracção `0,25` → **`1`** vértice · `0,5` → **`61`** · `1,0` → **`279`** · `2,0` → **`1 131`**. Razão de contagens `1131/61 = 18,54` (**`18,5×`**), razão de raios `4×`. ⭐ E o «`61`–`279`» que a §2.2 atribui à nossa lei nas duas células degeneradas bate: ali a nossa extensão é `R` |
| 5 | distribuição de dabs contada no corpus: **`74`** a 8, `1` a 6, `2` a 4, `1` a 3, **`21`** a 2, `1` a 1 = `100`. ⇒ o tecto `20` é de facto **inobservável** no corpus, e a dívida está nomeada |
| 6 | a barra do **G-13** re-medida no par que a página nomeia: `corte/corte_tiras_05` contra `_desligado` = **`7,3026e-03`** ⇒ `73×` acima de `1e-4`. ⭐ E é de facto a **menor** mudança viva entre fixturas publicadas (as outras: `1,4e-02`, `1,4e-02`, `1,4e-02`, e na lâmina `1,0e-01`…`5,0e-02`) |
| 7 | contado na pasta: o barro publica **`5`** limiares, o polegar **`2`**. E os dois estão **mortos ao bit** (`0,0000e+00` em todos os pares) |
| 8 | §6.2 re-medida: degrau `1,7315e-01` / `2,3906e-01` / `1,8821e-01`; rampa `1,0` → `2,4666e-02`; bossas `1,0` → **`3,0000e-08`** (o controlo mudo). `5` de `9` recuperáveis ✅ |
| 9 | ✅ a fixtura move `0` de `2 401` — mas **a cura introduziu uma errata nova**, a R2 abaixo |

⭐ **Reproduzidas ainda, ao dígito, de texto que nenhuma emenda tocou:** a tabela inteira da §2.2 na
linha da **rampa** (`0,00000` do alvo contra `−0,01603` das outras três — ⭐ e o **mecanismo**
confirma-se: a rampa é **exactamente plana** (resíduo `7,8e-09`) e o cursor está **`0,0693` ACIMA**
dela, então toda média de pontos da superfície fica na superfície e só a candidata que **puxa para o
cursor** sai dela) · as `8` de `11` que discriminam com piso `0,00269` · as `6` populações tocadas da
§4 (`265`, `274`, `271`, `287`, `62`, `632`) · os três tectos da §3.1 (`151`, `114`,
`265 = 151+114`) · a queda da §2.3 (`0,0000e+00` e `5,5954e-02`) · as três inércias da §2.7 e o
corte da §7.1 (`0,0000e+00` nas quatro) · as duas leis do `Ctrl` (`0,0000e+00` nas duas byte-idênticas,
`7,0410e-02` na terceira) · a máscara da §8 (`327` contra `560`) · e os `8` pares vivos/mortos da §7.3.

### 5. ⚠️ As SEIS erratas (gaveta B — **nenhuma bloqueia**; a janela I corrige-as enquanto implementa)

| # | sítio | o que está errado, e o que fica de pé |
|---|---|---|
| **R1** ⭐ | **§4.1 (l. 484, 487, 489), §1 (l. 210) e a tabela de derivação da §12 (l. 789)** | ⛔⛔ **O GÉMEO DE B1, pela TERCEIRA repetição da mesma doença — e está na secção que FUNDA a barra.** A 3.ª emenda curou o quadro do **G-1** (l. 798) para **`19` = `18` + `1`** e deixou **cinco** sítios a dizer **`25`** / *«24 de 25»* sobre a **mesma** reprodução e as **mesmas** duas pastas. ⭐ **A cura é trocar o número, não re-medir:** o `19` é o correcto e **eu contei-o do directório** (`13` de `lei/` + `6` de `lados/lados_*`), e o `25` não é derivável de partição nenhuma da pasta. O intervalo de resíduos, o `8,0e-08`, o `3,2e-05` e a barra `1e-6` **não se mexem** — só a contagem. ⇒ passa a ser **`18` de `19`** e **`1` de `19``. ⚠️ **Não bloqueia porque o G-1 é auto-consistente e passa:** quem correr as `19` obtém `18` no intervalo publicado mais a `lei/lei_bossas` a `3,2e-05`, que é exactamente o que o gate declara |
| **R2** | **§8, última linha (l. 690)** | a censura de fixturas inertes diz **«a QUARTA fixtura inerte do corpus»** e enumera quatro; o corpus tem **CINCO**. A quinta é **`corte/corte_laminav_din_inv_desloc02`** (`0` de `2 401`, saída byte-idêntica ao repouso). ⚠️ **Foi a cura da errata E9 que introduziu esta** — *um censo escrito para nomear uma ausência ganhou o dever de estar completo, e ninguém o correu sobre as 100* |
| **R3** ⭐ | **§7.3, tabela da célula única (l. 667)** | *«modo dinâmico + invertido → `3,2e-02` ⇒ ele vive»* está **certo como observabilidade e incompleto como mecanismo**: a fixtura com deslocamento `+0,2` move **`0` de `2 401`** vértices, e o `3,2347e-02` é inteiramente o deslocamento máximo da fixtura de deslocamento `0` (que move `338`). ⇒ *o que o deslocamento sobrevivente faz naquele modo é tornar o pincel **INERTE**, não deslocar o plano por `3,2e-02`.* A conclusão da secção (o valor chega ao consumidor num caminho e é projectado fora nos outros) **fica**, e a instrução *«não porte o par por simetria»* **fica**. ⚠️ Mas a espec chama-lhe *«a assinatura mais limpa desta espec»*, e uma assinatura cujo efeito observável é *«o pincel para de funcionar»* tem de o dizer |
| **R4** | **§7.3 (l. 674) e §12 G-13 (l. 816)** | *«três das oito células medidas estão mortas»* contra **quatro** marcas `⛔` nas duas tabelas. O discriminador é *morto em **todas** as células* (barro/corte, polegar/corte, polegar/deslocamento) contra *morto em `2` de `3` modos* (lâmina/deslocamento) — e a espec não o diz, então quem contar as marcas lê `4`. A conclusão **fica** |
| **R5** | **§7.3, 2.ª tabela (a do deslocamento)** | ela dá **`3,4e-02` / `1,1e-02`** para as tiras e **não há fixtura publicada** para esse par (a pasta do corte tem `26` ficheiros e nenhum é tiras+deslocamento). É **a mesma espécie da errata E6**, que a 3.ª emenda curou na tabela imediatamente acima — e a §6.2 já mostra o remédio: dizer, na própria tabela, **quais** células o corpus sustenta |
| **R6** ⚠️ **INSTRUMENTO** | **`VASSOURA_blender-trim-pincel.txt`, linhas `68`, `70`, `84`, `85`, `87`, `88`** | ⛔ **Seis entradas são fragmentos curtos (`8`–`11` caracteres, quatro delas num bloco contíguo com forma de truncagem) que casam DENTRO de identificadores NOSSOS**, e produzem **`41`** acertos **falsos** na nossa própria árvore rastreada — entre eles um *setter* de defaults do Painter e dois nomes de teste do `ph2d-editor-core`. ⇒ **o sweep `--git-history` SEM pathspec e o sweep da árvore viva sobre `crates/` e `docs/` devolvem `exit 1` sobre falsos positivos**, logo a barra do **§7.2** (*«zero hits sobre a árvore inteira é satisfazível — e é a barra»*) **não é satisfazível como está**. ⚠️ **Não toca nesta espec** (espec, 100 fixtures, ledger e `--git-history` da pasta: todos verdes) e **não é contaminação** — verifiquei cada acerto até ao identificador nosso que o causa. ⛔ **Mas cure-a ANTES do R-PÓS**, e por alongamento (nunca por remoção cega): uma vassoura que grita sobre código intocado ensina o auditor a ignorar acertos, e **a especificidade é o único lado que a truncagem estraga** — um fragmento casa *mais*, nunca menos, logo a detecção nunca esteve enfraquecida. ⭐ *Sintoma medido: o sweep sobre o scratchpad da janela-mãe acusa `exit 1`, e as `4` linhas acusadas são registos de corrida de gates NOSSOS* |

### 6. Higiene

- Nenhum relatório de sweep foi gravado em ficheiro: toda a saída viveu no terminal.
- ⚠️ **INC-R2 aberto** (abaixo): a vassoura foi descodificada **para ficheiro** uma vez, em
  `/dev/shm`, antes de eu ter lido o INC-R1 — mesma classe, cura já escrita, e não aplicada porque
  o ledger se lê **depois** de a corrida começar. Apagada dentro da corrida; verificado que
  `/dev/shm` não retém nada.
- ⭐ **Verificado, e é um resultado, não uma ausência:** o scratchpad da janela-mãe **não contém
  material do alvo** — os `4` acertos que lá aparecem são a errata **R6** a morder nos registos de
  gate dela.

✅ **R-pré, 4.ª passagem: ATESTA.** A parede do §4.2 está limpa, a regeneração é aditiva ao bit, e
**zero** achados bloqueiam. ⇒ **a janela I está ABERTA.** As `6` erratas seguem com ela, e a **R6**
tem de estar curada antes do R-PÓS.

## R6 — ALONGAMENTO da vassoura (2026-09-16, pedido pela 4.ª passagem do R-pré)

⭐ **A espec ATESTOU na 4.ª passagem e a janela I abriu.** Esta tarefa é do **instrumento**, não da
espec: entradas **curtas** da vassoura casavam **dentro de identificadores NOSSOS** e produziam
acertos falsos na árvore rastreada. ⛔ O problema não é detecção — é **especificidade**: a barra do
§7.2 (*«zero hits sobre a árvore inteira»*) deixava de ser satisfazível, e *uma entrada que dispara
sobre uso lícito treina quem corre o sweep a ignorar achados*.

### A cura: ALONGAMENTO, nunca remoção cega

Cada entrada curta foi substituída por formas **mais longas** que uma transcrição carregaria
igualmente (o nome do campo com o dono, o índice, a chamada com o 1.º argumento, a frase inteira em
vez do fragmento). ⇒ **a detecção nunca esteve enfraquecida e continua a não estar; o que se
recuperou foi a especificidade.**

**`11` entradas alongadas, substituídas por `22`** (vassoura **`321` → `332`**). A quatro delas
formavam um bloco contíguo com forma de truncagem; as outras sete são da mesma família e foram
curadas no mesmo passo, porque a barra é *zero*, não *«zero entre as seis nomeadas»*:

| classe do choque | com que nosso identificador colidia |
|---|---|
| **truncagem de nome de campo** (4) | nomes nossos que **começam pelo mesmo prefixo** — um de área com *custo*, três contadores nossos de coisas que começam por `no…`/`co…` |
| **nome genérico igual ao nosso** (4) | um centro anterior, um raio ao quadrado, uma amostragem de superfície e uma força — **todos existem na nossa casa com o mesmo nome e outro significado** |
| **fragmento de prosa** (3) | uma abertura de frase de três palavras que quatro painéis nossos usam em doc-comment, e uma tradução minha que colidia com a **paráfrase atribuída** que o nosso próprio ficheiro já fazia do mesmo facto |

### A contagem, ANTES e DEPOIS (para o R-PÓS não a redescobrir)

Método: `git ls-files -z | xargs -0 bash scripts/cleanroom-sweep.sh <vassoura>`.

| grandeza | antes | depois |
|---|---|---|
| ficheiros acusados, árvore inteira | **66** | **30** |
| linhas de conteúdo acusadas | **105** | **35** |
| ⭐ **ficheiros acusados em `crates/` (código NOSSO)** | **28** | **`1`** |
| ⭐ **linhas de conteúdo acusadas em `crates/`** | **57** | **`0`** |
| termos distintos a acusar | 29 | 19 |

⇒ **os acertos falsos dentro do nosso código foram a ZERO.** Os `29` ficheiros que sobram são
todos `docs/**` — a dívida de citações **pré-existente** que o `CLAUDE.md` §5 declara **fora do
censo por construção**, e que esta obra não criou nem tocou.

### ⚠️ E o ÚNICO acerto que sobra em `crates/` NÃO é um falso positivo — é um achado para o R-PÓS

`crates/ph2d-sculpt3d/src/verb_scrape_tests.rs` traz, num **doc-comment de teste**, uma **citação
verbatim curta da prosa do alvo, em inglês, entre aspas e ATRIBUÍDA** («a referência escreve o
porquê no próprio comentário — …»). Ele está na árvore desde **antes desta obra** (o ficheiro é
anterior a 2026-09-09, e migrou de sítio na W2, o que faz o `git log -S` por caminho actual não o
alcançar).

⛔ **Deliberadamente NÃO alongado.** Alongar a entrada para o calar seria exactamente *afrouxar a
tolerância* que o cabeçalho do `cleanroom-sweep.sh` proíbe. A pergunta — **citação curta de prosa
atribuída (direito de citação, §4.1.12) ou comentário do original a re-dizer (§4.2)?** — é do
**R-PÓS**, e fica aqui com o endereço, sem reproduzir o trecho.

### O instrumento, re-provado

`bash scripts/cleanroom-sweep-controlo.sh docs/3D/cleanroom/VASSOURA_blender-trim-pincel.txt` →
**os dez canais discriminam com a vassoura alongada**: os oito que têm de acusar acusam (nome de
ficheiro · texto numa linha · binário · dentro de `.gz` · dobrado com ênfase pelo meio · dobrado
entre comentários · esse comprimido · **dobrado no `.gz` sem marcador** · marcador de outra
família), o controlo **negativo** fica limpo, e o uso errado distingue-se do achado (`exit 2`).
Corre em **0,08 s**. ⇒ *alongar não abriu canal cego nenhum.*

## 2.ª MISSÃO DO E — «o pincel de fábrica e o traço» (2026-09-16)

**Pedido:** os dois relatos do smoke do dono — *o raio máximo é pouco* e *achata pior que o alvo*.
**Premissa dada e confirmada:** a lei por dab está fechada; a diferença mora nos valores de fábrica
e no traço, que o corpus da 1.ª missão fixava à mão.

### Actos sobre o alvo nesta missão (todos privados, §1.1)

| acto | como | o que tocou |
|---|---|---|
| ler os valores de fábrica | o binário do pacote **sem janela**, carregando o catálogo essencial pela API normal e lendo **propriedades** | ⚠️ o catálogo é um **asset** do alvo (§1.5.3): foi **carregado no processo do oráculo** e **nada dele foi copiado** — só os **números** das propriedades, como facto de interface (§4.1.3). A curva personalizada embutida em cada perfil foi **amostrada** para verificar que é inerte e **não** foi publicada |
| arrastar o pincel | janela do alvo num compositor **virtual**, eventos de rato **simulados** | nenhum ficheiro do alvo alterado; sem patch, sem build próprio |
| ler fonte novo | por shell, dentro da zona | ver a cobertura abaixo |

### Cobertura da travessia desta missão

| área | lido |
|---|---|
| o sistema de traço do modo de pintura — o espaçamento, a atenuação por sobreposição, o passo por evento, a execução por script e o passo real | ✅ **integral** nessas funções (as duas vias de espaçamento, a variável por pressão, a atenuação, o laço do espaço, o passo real e a via por script) |
| a força por dab do modo escultura | ✅ integral na função (todos os casos, o do plano incluído) |
| a actualização por passo do estado do traço (o cursor, a pressão, o raio inicial) | ✅ integral |
| o raio e o tamanho do pincel, e a curva de queda por preset | ✅ integral nessas funções |
| ⚠️ o sistema de eventos da janela e o seu teste próprio de simulação | ⚠️ **parcial, e só para o HARNESS** — lido para descobrir porque nenhum evento simulado pegava (era o ecrã de boas-vindas). **Nada disto entra na espec** além da condição funcional |

### O que se mediu (os números vivem na espec §14)

- Os valores de fábrica dos **cinco** perfis do pincel de plano, e a faixa do raio (é um
  **diâmetro**: pista `1`–`1 000` px, digitável `1`–`10 000`; de fábrica `100` px, unificado na
  cena). Contra o nosso tecto de `1/8` da altura da janela.
- **Completude:** um pincel nosso com os valores lidos reproduz o perfil de fábrica a `7,45e-09`.
- **Determinismo do arrasto:** `1,49e-08` (não ao bit).
- **O passo:** `0,14 R`, **exacto** pelo detector de saltos (`6`/`7`/`13`/`14`/`20`/`21` px ⇒
  `0`/`1`/`1`/`2`/`2`/`3` dabs efectivos). **A pressão do rato:** `1` (tirá-la é byte-idêntico).
- **A atenuação:** lei `0,570`, medido `0,563`.
- **A planura** passagem a passagem (1 a 8), contínua e com a caneta levantada; o perfil de achatar;
  a matriz 2×2 valores × traço; **os nossos valores no motor do alvo**; e a ablação de um valor de
  cada vez — ⭐ **a firmeza da normal é a alavanca dominante** (`0,029` → `1,380`).
- ⭐⭐ **Achado de método:** o cursor do corpus da 1.ª missão estava **fora** da superfície (até
  `0,54 R`). A lei do alvo não muda com isso; o **tamanho** dos efeitos sim — e numa rampa e num
  degrau, com o cursor na superfície, **nada** se move (as fixturas antigas movem `274` e `271`).
  O buraco da §2.2 **sobrevive** e no sulco fica **maior** (`+11,7 %`).
- ⛔ **Uma emulação por script do traço arrastado NÃO bateu** (cursor fora da superfície + atenuação
  ausente na via por script); com o cursor na superfície o dab único bate a menos da posição
  sub-píxel. Registado para quem tentar a emulação de novo: *a via por script usa o ponto do cursor
  que lhe dão*.

### Corridas

`65` novas (`18` + `2` + `9` do arrasto, `8` do traço do corpus, `9` + `1` do detector, `3` dabs por
script, `2` de emulação, `12` com o cursor na superfície, `1` de ensaio); **`55`** publicadas.

### Vassoura

`332 → 432 → 434` (o último passo é o R-bis abaixo): os identificadores internos do fonte lido nesta missão e as frases dos comentários
dele, nas duas línguas, com e sem acentos. Os identificadores **públicos** novos (os do simulador de
eventos, as propriedades de pressão e de atenuação) ficam **fora**, pela regra de §4.1.13.
Controlo dos **dez** canais **verde antes** do sweep, e um controlo positivo com duas entradas
**novas** disparou.

### R6-bis — as entradas NOVAS contra o nosso código (antes do commit)

Varrido `crates/`, `shells/` e `scripts/` com a vassoura de `432`: acenderam **20** ficheiros.
Classificado **cada** acerto até ao identificador que o causa:

| classe | entradas | cura |
|---|---|---|
| **falso positivo** — pedaço de um nome NOSSO mais longo (um factor de sobreposição do painter, a marca de «primeiro dab feito», um teste de redimensionar painéis) | `3` | ⭐ **ALONGADAS** pela regra do R6 (o nome com o dono, a expressão da lei) ⇒ vassoura **`432 → 434`** e **zero** acertos destas três |
| ⛔ **citação REAL pré-existente** — noutra obra | ver abaixo | **NÃO alongadas** |

⛔⛔ **O que sobra são 17 ficheiros, e NENHUM é falso positivo:**

- **16 são do PAINTER** — que é outra obra clean-room sobre o **mesmo** sistema de traço do alvo:
  **cinco** identificadores internos dele aparecem citados pelo nome em doc-comments de
  `ph2d-painter-brush` (`falloff.rs`, `lib.rs`, `sampler.rs`, `spec.rs`, `spec/queries.rs`,
  `stroke.rs`), e **um** deles vive ainda **embutido no nome de uma constante NOSSA** de id de
  controlo, usada em `ph2d-tool-painter` (`ids/painter.rs`, `tool/trait_impls.rs`,
  `tool/paint/accumulate_probe.rs`, `tool/paint/accumulate_tests.rs`,
  `tool/paint/tests/brush_panel.rs`) e em `ph2d-panel-painter-layers` (`event.rs`,
  `paint_stroke.rs`, `paint_stroke/tests.rs`, `populate.rs`, `tests/it/seam.rs`).
  ⚠️ É a espécie que o `CLAUDE.md` §5 já declara **não medida** pelo gate de citações: *os nomes
  de SÍMBOLO internos são §4.2 e o gate não os mede*. **Não é desta obra, e não se cura aqui** —
  vai ao coordenador com os endereços.
- **1 é o de sempre** (`ph2d-sculpt3d/src/verb_scrape_tests.rs`), já entregue ao R-PÓS no R6.

⇒ **A barra do §7.2 para ESTA obra continua a ser zero acertos nos ficheiros que ela criou** —
e é o que o sweep sobre a espec, o ledger, o README e as 155 fixturas mede (verde). A árvore
inteira não pode ser zero enquanto a dívida do Painter viver lá.

### ⚠️ Para o R-pré desta missão — o ponto mais perto da FORMA

A **§14.4** publica a lei da atenuação como uma soma em forma fechada, com os dois parâmetros de
discretização (o passo e o número de dabs sobrepostos) e a amostragem da fase em dez pontos. É o
mínimo que reproduz o número (`0,140` depende da amostragem), e está escrito como matemática; é,
ainda assim, o parágrafo desta missão mais perto da forma do alvo, e fica **nomeado** para a
auditoria o ler primeiro.

✅ **R-pré sobre a 2.ª missão: ATESTADA na 5.ª passagem** (secção seguinte).

## R-PRÉ — 5.ª PASSAGEM (2026-09-16) — veredito: ✅ **ATESTADA.** A parede do §4.2 está LIMPA (5.ª vez, contexto independente, documento INTEIRO varrido pela forma) e **ZERO achados BLOQUEIAM**. Ficam **9 erratas** (Q1–Q9) e **uma nota de proveniência de ENTRADA**

⚠️ **Higiene, escrita PRIMEIRO de propósito (INC-R1, INC-R2):** a vassoura foi descodificada
**só por cano** (`<(…)`), nunca para disco; nenhum ficheiro foi escrito fora dos dois que este
registo toca (a espec e este ledger), nem em `/tmp`, nem em `/dev/shm`, nem no *scratchpad* da
janela-mãe. As re-derivações correram em processos sem ficheiro (programa por `stdin`), a ler as
fixturas publicadas.

### 1. A parede (§4.2) — LIMPA, item a item, sobre o documento INTEIRO

⚠️ **A varredura foi pela FORMA** (a pergunta da §4.3.1), e sobre **toda** linha nova desde o
atestado da 4.ª passagem (`915cdd45d..HEAD`: `349` inserções — a abertura, os gémeos na §0.2/§0.3,
§2.1, §2.2, §4.1, §7.3, §8, §11, §12, §13, e a §14 inteira), não só sobre a §14.

| item do §4.2 | como foi auditado | resultado |
|---|---|---|
| texto de código / trechos | os **`14`** blocos cercados extraídos (`2` novos, ambos na §14.4) | ✅ fórmula em português, sem sintaxe de linguagem e sem laço. O da **atenuação** é o ponto mais perto da forma (o E nomeou-o): foi lido **contra o fonte** e é a soma de sobreposição de um núcleo periódico em forma fechada — um máximo sobre uma fase amostrada, uma soma sobre os vizinhos, o inverso — que é a matemática do método e as escolhas de discretização dele (§4.1.2, §4.1.11); a notação é genérica e não há estrutura de controlo transcrita |
| nomes internos | os *code spans* com forma de identificador, classificados um a um | ✅ os novos são **gates propostos** (`G-15..G-20`), **fixturas** e **ficheiros** nossos; zero nome interno. Um censo de `23` identificadores idiossincráticos do fonte lido nesta missão: **`0`** na espec (também com `_` trocado por espaço), e **`19`** deles cobertos pela vassoura — os `4` de fora não aparecem em lado nenhum da obra; `1` é propriedade de API pública e os outros `3` são internos, a acrescentar na próxima extensão da vassoura (acto de E) |
| comentários do original | as três frases de comentário que o R viu no fonte desta missão, procuradas também **traduzidas** (termos-chave em português) | ✅ zero |
| wording de manual / discussão | as citações `«»` novas; varredura de palavras inglesas na §14 | ✅ as `«»` novas são a **frase do dono**; a única palavra inglesa da §14 está **dentro** dessa frase |
| tabela verbatim / LUT | as tabelas novas, e a `valores_de_fabrica.txt` | ✅ as tabelas da §14 são **medições nossas** (re-derivadas abaixo). A tabela de fábrica são **defaults como factos de comportamento** (§4.1.3), lidos das **propriedades** do alvo a correr (caminho (b) do §4.2) e com a **prova de completude** ao lado (`copia_*`); cinco perfis, não uma LUT afinada |
| organização transcrita | a estrutura da §14 | ✅ organizada por **pergunta de medição** (valores · raio · passo · planura · matriz · ablação · cursor), nada que siga o arranjo do fonte |
| pseudo-código espelhado | os dois blocos novos | ✅ nível de paper |
| **cabeçalhos das fixturas novas** (texto novo) | as chaves e os tokens de valor dos `55` | ✅ chaves = domínio em português (`62` chaves distintas na pasta); tokens em maiúsculas = valores de enumeração da **API pública** (`17` na pasta, **dois** novos: o método de traço por espaço e a unidade de vista) e **cinco nomes públicos de perfil do catálogo** — os dois, chave de regeneração (§4.1.13) |

### 2. O sweep, e o controlo positivo — PRIMEIRO

| corrida | resultado |
|---|---|
| `cleanroom-sweep-controlo.sh` (antes de tudo) | ✅ **os dez canais** acusam, o negativo fica limpo, o `exit 2` discrimina-se |
| `cleanroom-sweep.sh` sobre a espec + a pasta das `155` fixturas | ✅ `exit 0`, `434` entradas |
| idem sobre este ledger, o INBOX e o README da pasta | ✅ `exit 0` |
| `--git-history` sobre a espec + a pasta das fixturas, e sobre `docs/3D/cleanroom/` | ✅ `exit 0` |
| ⭐ **sem distinção de caixa** (em memória, `grep -i`) sobre a espec, o README das fixturas, a tabela de fábrica, os `55` cabeçalhos novos e a mensagem do `b1d27f3b4` | ✅ **`0`** em todos — a cegueira de caixa que o `CLAUDE.md` nomeia como não curada não esconde nada aqui |

### 3. A EXACTIDÃO — re-derivada, não aceite

#### 3.1 Os números da §14, a partir das fixturas publicadas (tudo ✅)

| afirmação | re-derivado |
|---|---|
| faixa da planura: `441` vértices, repouso RMS `0,01153`, máximo `0,02032`, altura `+0,0003` | `441` · `0,011529` · `0,020321` · `+0,000321` — ⚠️ **com a faixa escolhida pela posição de REPOUSO** (Q5) |
| *aparar*, 1·2·3·4·6·8 passagens: `0,418` · `0,226` · `0,139` · `0,092` · `0,048` · `0,029`; altura `−0,0177` | idênticos ao dígito; `−0,0177` |
| caneta levantada 2·4·8: `0,239` · `0,180` · `0,119`, altura `−0,043` | idênticos; `−0,0434` |
| *achatar* 1·2·4·8: `0,730` · `0,534` · `0,284` · `0,066`, altura `+0,0001` | idênticos |
| matriz 2×2: `0,029` · `0,354` · `1,934` · `1,996` | idênticos |
| os nossos valores 1·2·4·8: `1,010` · `1,027` · `1,061` · `1,114` | idênticos |
| ablação: `1,380`/`−0,090` · `0,287`/`−0,010` · `0,122`/`−0,015` · `0,096`/`−0,015` · `0,020`/`−0,018`; inclinação `1,24°` contra `0,32°` | `1,380`/`−0,0904` · `0,287`/`−0,0098` · `0,122`/`−0,0146` · `0,096`/`−0,0150` · `0,020`/`−0,0182`; `1,24°`/`0,32°` (plano de mínimos quadrados ortogonal) |
| «vala de `4,5×` a amplitude» | `0,0904 / 0,02 = 4,5` |
| determinismo `1,49e-08` · completude `7,45e-09` | `1,5e-08` e `8,0e-09` no texto decimal — os passos de `f32` que a espec cita |
| detector: `6 px` → `0`, `7`/`13` → `1`, `14` → `2`, `21` → `3`; sem pressão = byte-idêntico | `6 px` move `0`; **`7 px` e `13 px` são idênticos AO BIT**; `7` e `14` diferem, `14` e `21` diferem; amplitudes `3,29e-3 · 6,60e-3 · 9,96e-3` (≈ `1 : 2 : 3`); sem pressão = `0,0` |
| atenuação: `0,140` · `0,200` · `0,071` e `(1+a)/2` | `0,1400` · `0,2000` · `0,0714` pela fórmula da §14.4 (também em `f32`) — e a força por dab da §14.4 confere com o fonte, nos dois ramos |
| razão medida `0,563` | `0,5627` (razão das amplitudes máximas) |
| «o traço toca até `|y| = 0,25`» | o maior `|y|` movido é `0,25` |
| a tabela da §14.8, toda | ✅ **ao dígito**: desvios do cursor `0,271`·`0,543`·`0,181`·`0,271`×3·`0,271`×2·`0,000`·`0,179`·`0,375`; contagens `265→259`·`62→65`·`632→614`·`262→252`·`274→268`·`276→272`·`265→259`×2·`268→268`·`274→0`·`271→0`; o nosso centro `+11,7`·`+7,5`·`+12,2`·`+14,7`·`+2,6`·`−8,1`·`+11,7`·`+11,6`·`0` %; a nossa normal `8,7`·`4,6`·`12,7`·`8,7`·`8,7`·`8,7`·`11,7`·`3,8`·`31,2°`; e a candidata do alvo cai no plano do alvo (`≤ 8,7e-08`) também com o cursor na superfície |
| advertência do G-5: `R_c = 0,25 R` lê `0,00057` com o cursor na superfície | `0,00057` (e a célula do sulco de raio `0,2` lê `0,00060`, também abaixo de `1e-3` — mesma advertência) |
| G-19: rampa `0`, degrau `2,98e-08` | `0,0` e `3,0e-08` |
| inventário `155` = `14+14+3+12+5+7+26+11+8+44+11`; `65` corridas, `55` publicadas | ✅ do directório |

#### 3.2 ⭐⭐⭐ O G-15 REPRODUZIDO pela lei da página (a barra é alcançável)

A lei das §§2–4 + §6.1 + a dureza + a força por dab da §14.4 (`a = 1` na via por script), escrita a
partir da **página**, sobre `fabrica_e_traco/passo_script_1dab_na_superficie`:

| variante | `max |Δ|` contra o oráculo |
|---|---|
| cursor lançado na superfície; memória da normal **semeada no primeiro dab** (o que não move nada) | ⭐ **`1,88e-08`** |
| idem, mas a memória semeada no primeiro dab **efectivo** | `2,98e-03` |

⇒ a barra `1e-6` do G-15 é **alcançável** e o gate **discrimina** o estabilizador (o ângulo entre as
duas normais em jogo é `4,3°`). ⚠️ E é daqui que sai a **Q6**.

#### 3.3 A coluna «nosso» da §14.2, lida no código vivo (✅, com a Q3)

Força `0,5` (o verbo não tem perfil `s` e cai no valor de fábrica genérico) · dureza `0` · curva
suave · acumular desligado · **nenhum** controlo de firmeza (normal ou centro) · extensões `0,5`/`0,5`
· tectos `1`/`0` · inversão *afastar* · raio de fábrica `50` px · deslocamento `0` · auto-alisamento
`0` · só-frente desligado. **Espaçamento:** `0,15 R` para **todo** verbo, **sem** atenuação (Q3).
**Tecto do raio** em `b1d27f3b4`: `1/8` da altura do **viewport activo** (Q1). E a casa já calcula o
plano **antes** do teste de direcção do traço, que é onde o estabilizador tem de viver (Q6).

### 4. ⛔ Achados que BLOQUEIAM: **nenhum**

Nenhuma lei atribuída à nossa casa que ela não tenha, nenhuma barra impossível contra a tabela que a
fundamenta (o G-15 foi reproduzido; o G-16..G-19 têm a população publicada e re-derivada), nenhuma
população de gate que a página não sustente. O G-20 é **condicionado ao dono** pela própria página, e
o conflito dele vai como Q2.

### 5. ⚠️ As NOVE erratas (gaveta B — a janela I corrige-as enquanto implementa)

| # | sítio | o que está impreciso | instrução funcional |
|---|---|---|---|
| **Q1** | §14.3, linha «tecto», coluna «nós», e a frase «numa janela de 1080 linhas o nosso raio pára em 135 px»; a linha «pista» da coluna «nós» (`—`); este ledger, §2.ª missão («contra o nosso tecto de 1/8 da altura da janela») | em `b1d27f3b4` o tecto era `1/8` da altura do **viewport ACTIVO** (menor que a janela com os painéis, e metade numa vista quádrupla), e o painel tinha uma **pista de `200` px** de raio que volta ao tecto real; ⚠️ e **durante esta passagem** o `b5423a9df` trocou o tecto pela **diagonal da vista** (pista de `5 000` px, presa ao tecto real) | ler a linha como **instantâneo datado** do estado anterior (o `3,7×`/`37×` é um **piso** desse instantâneo), **nunca** como lei da casa; quando o tecto mudar, a linha diz a data, o commit e o recurso (o viewport) |
| **Q2** | §12, G-20 | a barra não nomeia **em que tamanho de vista** se mede, e a metade «digitável ≥ `5 000` px de raio» é **inalcançável** sob um tecto derivado da diagonal da vista em qualquer vista de diagonal `< 5 000` px (`2 203` px a 1920×1080, `4 406` a 3840×2160); a metade «pista ≥ `500`» é satisfeita por esse tecto em qualquer vista de diagonal `≥ 500` | se o dono quiser o G-20, ele nasce com a **população nomeada** (tamanhos de vista) e contra o **recurso** que o tecto da casa declara; a metade digitável é **decisão do dono** contra um tecto limitado ao ecrã, não um número a copiar |
| **Q3** | §14.2, linha «espaçamento» (`≈`) | a casa espaça a `0,15 R` para **todo** verbo (uma fracção global) e **não tem atenuação por espaçamento**; o G-16 exige `0,14 R` exactos (com `0,15 R` os saltos de `14` e `21` px dão `1` e `2` dabs efectivos, não `2` e `3`) e o G-17 exige o factor dos `7 %` (`0,570`; a `7,5 %` seria `0,575`) | o verbo de plano precisa de **espaçamento próprio** (`7 %` do diâmetro no perfil *aparar*) **e** do factor de atenuação; a linha deve marcar a atenuação como ⛔ |
| **Q4** | §12, G-16, célula de `7 px` | a célula é um salto de **exactamente um passo**; o alvo pousa um dab quando o comprimento percorrido **atinge** o passo (`7 px` e `13 px` saem idênticos ao bit), e a fronteira do passo da **casa** recusa um salto igual a um passo — de propósito, com gate a declará-lo deliberado. Com um só salto e a caneta levantada a seguir, a casa deposita `0` dabs onde a fixtura mostra `1` | **mudar a fronteira** (e re-raciocinar o gate da casa que a declara deliberada — a própria nota dele diz que as duas fronteiras dão a mesma lista, menos o passo pendente) **ou** declarar a célula como **divergência nomeada**; nunca a calar |
| **Q5** | §14.5, «A régua» | a faixa de `441` só existe escolhida pela posição de **REPOUSO**; escolhida pela de saída ela tem `423`–`441` vértices e os números mudam (ablação `1,246` em vez de `1,380`; os nossos valores `0,984`·`0,998`·`1,029`·`1,076` em vez de `1,010`·`1,027`·`1,061`·`1,114`; *achatar* `0,706`·`0,509`·`0,260`·`0,048` em vez de `0,730`·`0,534`·`0,284`·`0,066`) | a régua escolhe os índices no **repouso** e mede-os na **saída**. O G-18 passa nas duas leituras, mas qualquer gate que imprima estes números usa a do repouso |
| **Q6** | §6.1 × §1 | a página diz as duas coisas mas não a **ordem**: o primeiro dab da passagem, **que não move nada**, calcula o plano **e semeia** a memória da normal; com firmeza `1` toda a passagem usa a normal **desse** dab | o estabilizador vive **dentro** do cálculo do plano, **antes** do teste «o traço já tem direcção?» (a casa já calcula o plano antes desse teste). Medido: `1,9e-08` na ordem certa, `3,0e-03` semeando no primeiro dab efectivo (§3.2) |
| **Q7** | §14.6, coluna «traço do corpus» | ela muda **duas** coisas ao mesmo tempo — a via por script **sem atenuação** e o **cursor em `z = 0`, fora do relevo** (os cabeçalhos dos `ctl_*_traco_do_corpus_*` dizem-no) —, logo «o TRAÇO decide ATÉ ONDE» está confundido com o efeito da §14.8 | nomear a confusão; nenhum gate usa esta coluna, e a conclusão sobre os **valores** (a linha de baixo, nas duas colunas) fica de pé |
| **Q8** | cabeçalhos de `fabrica_e_traco/`; §14.4; README das fixturas | `25` de `44` com `o_que_ela_fixa` **vazio**; as arrastadas **sem** a chave `cursor` (o cursor é o raio do rato sobre a superfície **viva** — implícito); `passo_script_1dab_na_superficie` diz «ida e volta» com **dois** pontos num só sentido, e não diz (como a família `cursor_na_superficie/` diz) que o bloco `c` são os pontos **PEDIDOS** em `z = 0` e que o dab cai onde a vertical por eles corta a superfície; as `5` da via por script **sem** a chave `pincel_de_origem`; `sup_sulco*` corresponde a `lei_crista*` (o README diz «homónimas»); a célula de `20 px` da tabela da §14.4 **sem fixtura publicada** (publicadas: `6`, `7`, `13`, `14`, `21` — a espécie da R5) | o I lê o cursor destas fixturas como **raio na superfície** e o bloco `c` como **pedido**; a próxima regeneração (acto de E) completa os cabeçalhos |
| **Q9** | §4 (tabela de resíduos), §2.2/§11 (nota do cursor), §12 (G-1b) | a bossas lê `287` na §4 (qualquer mudança) e `268` na §14.8 (limiar `1e-7`) — `19` vértices mexem só por ruído de `f32`; as notas «o cursor das `11` está fora do relevo» valem para `10` — a célula de **bossas** tem o cursor **NA** superfície (`0,000 R`, a própria §14.8 o diz); a linha do G-1b ainda diz «dívida», e a abertura da §14 e a bancada da casa já reproduzem os traços inteiros | um gate de contagem **nomeia o limiar**; as duas notas dizem «`10` de `11`»; a linha do G-1b acompanha a implementação |

### 6. ⚠️ Nota de proveniência de ENTRADA (§5 e §1.5.3) — **não bloqueia, com a razão**

**`14`** fixturas foram corridas com o **perfil do catálogo do alvo carregado como pincel**
(`fab_aparar_*` ×`10`, `fab_achatar_*` ×`4`); as outras `41` da pasta nova declaram pincel **nosso** ou
são da via por script do harness. A letra do §5 proíbe assets do alvo como **entrada** de fixture,
porque a saída herdaria a expressão da entrada.

⇒ **A substância da regra está cumprida para as `10` do *aparar*, e está medida:** o par
`copia_aparar_continuo_4`/`_8` — pincel **NOSSO** com os valores do cabeçalho escritos — bate as
`fab_*` homónimas a **`8e-09`**, abaixo do não-determinismo do próprio arrasto (`1,5e-08`). A saída é
função **só** dos números publicados (factos, §4.1.3); o único elemento não-numérico do perfil (a curva
personalizada) é **inerte** sob a curva suave (confirmado no fonte e pela igualdade do par), e nada
dele foi publicado. **As `4` do *achatar* não têm par**, e nenhum gate as usa.

**Instruções:** (a) a janela I ancora os gates nas fixturas cujo cabeçalho declara pincel **NOSSO**
(o número `0,029` do G-18 tem sujeito em `copia_aparar_continuo_8`); (b) a próxima regeneração (acto de
E) troca as `14` por corridas de pincel nosso, e dá ao *achatar* o seu par de completude.

### 7. O que o R leu do alvo nesta passagem (por shell, na zona)

Para conferir a §14 contra o **comportamento**, e só isso: a atenuação por sobreposição e o sítio onde
ela é calculada no passo do traço; a força por dab do modo escultura (o caso do plano); a entrada do
pincel de plano (a ordem entre o cálculo do plano e a saída do primeiro dab); a curva de queda por
preset; e a definição e o *setter* do tamanho do pincel na API. **Nada** disto foi escrito na espec ou
neste ledger além dos **factos** já publicados pela §14.

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

### INC-R2 (2026-09-16) — **a MESMA classe do INC-R1, no R-pré da 4.ª passagem. Sem exposição; registado porque a cura do INC-R1 existia e não me alcançou**

- **O que:** para auditar a **cobertura e a especificidade** da vassoura (o achado **R6**), descodifiquei-a
  **para um ficheiro**, em `/dev/shm`. O §3.E nomeia `/tmp` entre os sítios proibidos e `/dev/shm` é
  da mesma classe (tmpfs alcançável pela janela-mãe).
- **Régua do §6.2:** **não é substancial e não é um relance** — nada foi lido por quem escreve
  produto; foi o R a escrever, a consultar e a apagar um ficheiro derivado dentro da própria
  corrida. O conteúdo é uma **lista** de identificadores e fragmentos de prosa, não um corpo de
  função.
- **Janela de risco:** duas invocações de shell da corrida do R-pré. **Apagado** a seguir
  (`rm -f /dev/shm/vb.*`), com a pasta verificada depois. O resto da passagem descodificou **por
  cano** (`<(dec)`), sem disco.
- ⛔⛔ **A LIÇÃO, e ela é sobre o MÉTODO, não sobre mim:** o INC-R1 escreveu a cura certa — *«o
  `cleanroom-sweep.sh` já descodifica em memória; quem precisar de inspeccionar descodifica por
  cano»* — e ela **não impediu a reincidência**, porque um incidente registado no **fim** do ledger
  é lido **depois** de a corrida começar, e a inspecção da vassoura é das **primeiras** coisas que
  um R-pré faz. ⇒ *uma cura que vive só no registo do incidente protege a passagem seguinte apenas
  se ela ler o registo antes de agir.* A cura que apaga a classe é **instrumental**: um
  `scripts/cleanroom-vassoura-inspecciona.sh` que só escreve em `stdout` (ou uma nota no **topo**
  do ledger e no BLOCO da MISSÃO-R-PRÉ), para que a forma certa seja a forma **fácil**.
- **Estado depois:** sweep **verde** sobre a espec, as 100 fixtures, este ledger, o INBOX e o README
  da pasta, e **verde** em `--git-history -- docs/3D/cleanroom/`. O scratchpad da janela-mãe foi
  varrido e **não contém material do alvo** (os acertos que lá aparecem são a errata **R6**).
