# SPEC — o GESTO DE CORTE (clean-room, comportamento observado)

```
Alvo: Blender 5.2.1 LTS (binário /usr/bin/blender, pacote 17:5.2.1-2) · fonte lido: tag v5.2.0
  · Licença do alvo: GPL-2.0-or-later
⭐ Degrau: **T2 para a LEI DO GESTO (este documento) · T0 para o MOTOR DE BOOLEANA**
  A triagem tem DUAS metades e elas dão degraus diferentes (ledger §2). O solucionador que o alvo
  usa POR OMISSÃO no corte é uma biblioteca EXTERNA sob Apache-2.0, que o binário dele LIGA
  dinamicamente (medido por `ldd`); há ligações e portes em Rust sob MIT/Apache-2.0 com
  proveniência verificada. ⇒ ⛔ **NÃO escreva um motor de booleana a partir desta espec** — ele
  porta-se, com atribuição. Esta espec é só a LEI DO GESTO.
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-trim.md, 2026-09-15 (ANTES da 1ª leitura de
  conteúdo do fonte)
Patente (§8.1): buscado em 2026-09-15 — 4 patentes examinadas, todas EXPIRADAS ou lapsadas, e a
  única deste século tem reivindicação presa a um sistema específico que não é o nosso. ⇒ nenhuma
  patente VIVA alcança o método. Termos e tabela no ledger.
Filtragem §4.3: executada em 2026-09-15 · Sweep: ✅ VERDE — vassoura de **565** entradas, contra
  **as 7 vassouras vivas da casa**, sobre a espec + o ledger + as fixturas.
  ⚠️ A vassoura cobre INGLÊS e PORTUGUÊS e as formas SEM ACENTOS — esta espec escreve-se em
  português, e traduzir prosa do alvo é a forma de fuga que já passou três vezes nesta casa.
  ⚠️⚠️ **O sweep VERDE inicial era do INSTRUMENTO, não do texto** — e foi preciso curá-lo DUAS
  vezes para ele dizer a verdade: (a) a vassoura não cobria as descrições do manual traduzidas,
  e ao cobri-las acusou **5 linhas desta espec**, que foram reescritas a partir da MEDIÇÃO em vez
  da prosa do alvo; (b) o `cleanroom-sweep.sh` era **cego a toda frase que o parágrafo quebra**
  (`grep -F` casa dentro de UMA linha), e ao curá-lo ele apanhou uma citação traduzida que
  sobrevivia partida entre duas linhas. Ensaio com controlo e a cura: ledger §«Achado de
  instrumento». ⭐ As 5 especs que já estavam no repo continuam verdes sob o instrumento curado.
Auditoria §4.2 (R-pré): ⛔⛔ **PENDENTE — a janela NÃO implementa a partir desta versão.**
  Quem a escreveu foi o subagente-E, e autofiltragem não é auditoria (SKILL §3.R). É preciso um
  subagente R-PRÉ NOVO (≠ este E) que leia os dois lados e ateste aqui. ⚠️ Peça-lhe atenção
  especial aos itens com selo **L** (§0): são os que descrevem comportamento lido no fonte e
  ainda NÃO confirmado na saída do oráculo — §4.2 (o enchimento da profundidade), §6.2 (as quatro
  combinações do ponto médio) e §10 (as três suposições).
Mapa de leitura da literatura: ⭐ **nenhum paper é necessário para a LEI DO GESTO** — ela é
  geometria elementar (desprojecção, prisma, triangulação de polígono 2D). Para o MOTOR, que é
  T0 e se porta, a literatura pública relevante é a do algoritmo de booleana robusta sobre malhas
  (arranjos de malha / números de enrolamento) — ⛔ mas ler literatura de motor é DESNECESSÁRIO:
  a biblioteca permissiva existe e está medida.
Denylist de URLs (⛔ o Implementador NUNCA abre): projects.blender.org/blender/blender (fonte,
  commits, PRs, issues), qualquer espelho do fonte (github.com/blender/blender, git.blender.org),
  developer.blender.org, e qualquer code-search que devolva o fonte do alvo (grep.app, searchcode,
  sourcegraph). ✅ **O manual público (docs.blender.org) é LEGÍTIMO para o I** (SKILL §3.I: toda a
  prosa pública) — ⛔ desde que ele não transcreva o wording.
"Este documento descreve comportamento; não contém expressão do alvo."
```

---

## §0 — O que esta espec é, e como se lê

Descreve **o que a família de gestos de corte faz**, fase a fase, com cada número acompanhado da
sua proveniência:

| selo | significa |
|---|---|
| **F** | derivado de fórmula / aritmética sobre um objecto que esta espec nomeia |
| **M** | **medido** na saída do oráculo, com a fixtura ao lado |
| **D** | **documentado** pelos autores em fonte pública **de fora do código** (manual público, notas de versão, rastreador) |
| **L** | **lido no fonte do alvo** como **facto de comportamento** (SKILL §4.1.3 — constante, default ou regra de decisão). ⚠️ É o selo mais fraco: um facto lido não foi **confirmado** na saída |
| **N** | decisão **nossa** |

⭐ **Onde um item tem `L` e `M`, o `M` manda** — e esta espec diz os dois de propósito, para que se
veja **o que foi confirmado na saída e o que ainda não foi**. ⚠️ **Todo `L` sozinho é um convite à
auditoria**: ele descreve comportamento que ninguém ainda viu acontecer.

⛔⛔ **O selo `D` NUNCA cobre nada que se leia DENTRO da árvore do alvo — ficheiro de teste
incluído.** Um ficheiro de teste é código-fonte como outro qualquer. O que se pode usar é o
**facto**, e um facto entra aqui por **M** ou por **F** — ⛔ nunca por `D`.

⚠️ **A decomposição em fases abaixo é NOSSA**, escolhida para descrever comportamento. Não
reproduz a organização do alvo, e o Implementador é livre de a repartir como quiser.

⚠️ **Nomes PÚBLICOS conservados de propósito** (SKILL §4.1.13 — são a interface que o oráculo
aceita, e portanto a chave para o re-correr): os valores `DIFFERENCE` · `UNION` · `JOIN` do modo,
`VIEW` · `SURFACE` da orientação, `FIXED` · `PROJECT` da extrusão, `EXACT` · `FLOAT` · `MANIFOLD`
do solucionador, e os nomes de propriedade que o oráculo lê. ⛔ Nenhum nome interno do alvo
aparece neste documento.

---

## §1 — ⭐⭐⭐ A DECISÃO QUE ESTA ESPEC EXISTE PARA INFORMAR

O corte pode ser construído por **booleana de malha** ou pela **rota por campo/voxel** que esta
casa já tem em produção (`ph2d_sdf::voxelize → flood_fill → surface nets`). ⚠️ **Não são dois
caminhos para o mesmo sítio: eles produzem malhas de naturezas diferentes**, e a diferença é
medível numa grandeza só.

### §1.1 — A régua que separa: quantos vértices da ENTRADA sobrevivem BIT-A-BIT

| caminho | peça | V antes → depois | ⭐ vértices de entrada preservados **ao bit** | relógio |
|---|---|---|---|---|
| **booleana** (alvo, `MANIFOLD`) | esfera 482 V | 482 → 371 | **317** (`66 %`) — **M** | — |
| **booleana** (alvo, `MANIFOLD`) | esfera 98 306 V | 98 306 → 58 019 | **57 489** (`58 %`) — **M** | — |
| **voxel** (nossa, `res=64`) | esfera 482 V | 482 → 19 142 | **0** — **M** | 14,1 ms |
| **voxel** (nossa, `res=128`) | esfera 482 V | 482 → 76 606 | **0** — **M** | 72,9 ms |
| **voxel** (nossa, `res=256`) | esfera 482 V | 482 → 306 670 | **0** — **M** | 473,7 ms |
| **voxel** (nossa, `res=64`) | esfera 98 306 V | 98 306 → **19 862** | **0** — **M** | 108,4 ms |
| **voxel** (nossa, `res=128`) | esfera 98 306 V | 98 306 → 79 142 | **0** — **M** | 171,7 ms |
| **voxel** (nossa, `res=256`) | esfera 98 306 V | 98 306 → 316 742 | **0** — **M** | 655,1 ms |

*(booleana: fixtura `uv_sphere_16x32` e `sculpt_sphere`, corte em caixa a `x ≥ 0,2`, `DIFFERENCE`,
`VIEW`, `FIXED`, sem profundidade de cursor. Voxel: as MESMAS malhas, `ph2d_sdf::remesh`,
`--release`.)*

### §1.2 — E a medição LOCAL, que é a que decide

⭐⭐ **Longe do corte, a booleana não toca em NADA.** Contando só os vértices do lado oposto
(`x < −0,5`, a `0,7` de distância do plano de corte):

| peça | vértices longe, antes | depois | ⭐ **iguais ao bit** |
|---|---|---|---|
| esfera 482 V | 97 | 97 | **97 / 97** — **M** |
| esfera 98 306 V | 26 533 | 26 533 | **26 533 / 26 533** — **M** |
| toro 512 V | 162 | 162 | **162 / 162** — **M** |

⇒ **a booleana re-tessela SÓ a vizinhança do corte.** A rota por voxel re-tessela **tudo, sempre**
(`0` sobreviventes em todas as seis células medidas), e a contagem de saída dela é ditada pela
**resolução da grelha**, não pela entrada: a mesma grelha que **destrói** uma peça densa
(`98 306 → 19 862`, `−80 %`) **explode** uma peça grosseira (`482 → 306 670`, `636×`).

### §1.3 — O que isso custa ao artista, dito sem jargão

- Uma escultura tem densidade **autorada**: fino onde o artista trabalhou, grosso onde não. A
  booleana devolve essa densidade intacta fora do corte; a rota por voxel devolve densidade
  **uniforme** em toda a peça.
- ⇒ Um corte por voxel **não é um corte**: é um corte **mais um remalhamento da peça inteira**.
- ⚠️ **E o alvo trata os dois como passos SEPARADOS e sequenciais** — o manual público diz que a
  ferramenta é *«especialmente útil»* para esboçar uma malha base para depois remalhar por voxel
  (**D**). *Colapsar os dois num só retira do artista a possibilidade de cortar **sem** remalhar,
  que é a razão de o passo existir.*
- ⚠️ E há um tecto declarado do outro lado: o alvo **desaconselha** a ferramenta acima de
  **100 k vértices** quando o modo é `DIFFERENCE`/`UNION` **com o solucionador `EXACT`**, e manda
  usar outra ferramenta nesse caso (**D**).

### §1.4 — ⭐⭐⭐ E o motor da booleana NÃO precisa de ser escrito: a porta está ABERTA

⚠️ Esta secção é **triagem de licença**, não desenho — está aqui porque muda o preço da decisão
acima em uma ordem de grandeza. O detalhe e a medição vivem no ledger.

- O alvo **não implementa** o solucionador que ele próprio usa por omissão: ele **liga-se** a uma
  biblioteca externa de geometria, sob licença **permissiva (Apache-2.0)**, e o ficheiro dele é só
  o adaptador entre a representação de malha dele e a da biblioteca — que é exactamente a parte
  que teríamos de escrever de qualquer modo.
- Medido nesta máquina: o binário do alvo **liga dinamicamente** essa biblioteca (`ldd`), o pacote
  dela declara `Apache-2.0`, e o **valor de omissão** do selector de solucionador do gesto de corte
  é essa biblioteca (**M** no fonte e na interface).
- Existe **porte e ligação em Rust sob licença permissiva**, com proveniência verificada
  (descende da biblioteca Apache-2.0, **não** do alvo). Nomes públicos e degrau: ledger §«Metade (a)».
- ⇒ **Só a LEI DO GESTO (este documento) é T2.** O motor é **T0** e porta-se.

---

## §2 — As fases (a nossa decomposição)

1. **Captura** — o gesto e o estado do cursor no instante em que ele começa (§3).
2. **Polígono de ecrã** — a forma desenhada vira um anel fechado de pontos 2D (§4).
3. **Plano da forma** — origem + normal, que dão o eixo do varrimento (§5).
4. **Profundidade** — onde o volume começa e acaba ao longo desse eixo (§6).
5. **Prisma** — o polígono de ecrã varre-se nesse eixo e vira uma malha fechada (§7).
6. **Normais** — a malha varrida é reorientada, e é isso que torna o sentido do desenho
   irrelevante (§8).
7. **Passagens de simetria** — o prisma é espelhado e a operação repete-se (§9).
8. **Booleana** — a malha da peça e o prisma combinam-se (§10).

---

## §3 — Captura: quatro variantes, três formas

**D** + **M**. A família expõe **quatro** ferramentas — caixa, laço, linha, polilinha — mas
internamente há **três** formas, porque **a polilinha é tratada como um laço** (a diferença está só
em como o utilizador a desenha).

| variante | o que o gesto entrega | modos que oferece |
|---|---|---|
| caixa | 4 cantos do rectângulo de ecrã | os três |
| laço | o caminho desenhado, ponto a ponto | os três |
| polilinha | os vértices clicados | os três |
| linha | **2** pontos | ⚠️ **só `DIFFERENCE`** (**D** e **M**) |

⚠️ **A linha é a mesma máquina, não outra**: os 2 pontos viram um quadrilátero (§4.2), e o modo é
**forçado** a `DIFFERENCE` — a interface dela nem mostra o selector de modo (**M**).

No instante em que o gesto começa, grava-se, **da posição do ponteiro**:

- se havia superfície sob o cursor (**acerto**) — e, havendo, a **posição 3D** e a **normal da
  superfície** nesse ponto;
- ⚠️ **se NÃO houve acerto, a orientação é FORÇADA a `VIEW`** — **L** + ⭐ **M, com o controlo
  emparelhado**:

  | onde o gesto COMEÇA | `VIEW` (V / F / sobrev. / volume) | `SURFACE` (V / F / sobrev. / volume) | |
  |---|---|---|---|
  | **fora** da peça | `371` / `374` / `317` / `2,66194` | `371` / `374` / `317` / `2,66194` | ⭐ **idênticos** |
  | **sobre** a peça | `420` / `419` / `369` / `3,09146` | `453` / `436` / `387` / `3,25716` | **diferentes** |

  ⚠️ **As duas linhas são obrigatórias.** Só a de cima mostraria um knob morto; só a de baixo
  mostraria um knob vivo. *Juntas elas dizem a lei: o knob é vivo, e é silenciosamente anulado
  quando não há superfície de onde tirar uma normal.*

  *É a única correcção silenciosa de um parâmetro autorado em toda a família — o resto dos
  vereditos é recusa em voz alta.* **N:** um módulo nosso pode **dizê-lo** em vez de o fazer calado.

---

## §4 — O polígono de ecrã

### §4.1 — Caixa, laço e polilinha

Os pontos são usados como vêm. A caixa entrega os 4 cantos numa ordem fixa. ⚠️ **O anel pode ser
CÔNCAVO** (um laço em forma de C é forma válida) e **pode ser desenhado em qualquer sentido** — as
duas coisas são resolvidas mais à frente (§7.3 e §8).

### §4.2 — A linha vira um quadrilátero  — **L** + **F**, com a consequência **M**

Dois pontos de ecrã `p0`, `p1` não delimitam área nenhuma. A lei que os transforma num anel:

1. **Um factor de alcance** (**L**), derivado da peça, não escolhido: projecta-se a caixa envolvente do
   objecto no ecrã, e o factor é o **comprimento da diagonal** desse rectângulo **× 2**.
   ⚠️ O papel dele é **um só**: pôr os pontos criados longe o bastante para envolverem a peça
   inteira. ⛔ Não é constante de afinação — **qualquer** valor que envolva a peça dá a mesma saída,
   e é por isso que o `× 2` não precisa de ser reproduzido.
2. **Prolongamento**: se *não* se está a limitar ao segmento, `p1` avança e `p0` recua ao longo da
   direcção da linha por esse factor. ⇒ a linha passa a atravessar a peça de lado a lado.
3. **Lado**: toma-se a perpendicular da direcção — a rotação de `−90°`, isto é `(dy, −dx)` — e
   **inverte-se o sinal dela se o gesto estiver invertido**; multiplica-se pelo factor.
4. O anel final é `[ p0 , p1 , p1+perp , p0+perp ]`.

⇒ **a linha é um meio-plano** aproximado por um rectângulo grande o bastante. **M:** a mesma linha
com o sinal invertido corta **o outro lado** — `preservados_ao_bit` do lado oposto passa de
`97/97` para `0/97`.

⚠️ **Limitar ao segmento NÃO é um corte menor do mesmo corte:** com o limite ligado a peça
**ganha** vértices em vez de perder (`482 → 493`, contra `482 → 371` sem o limite — **M**), porque
o volume deixa de atravessar a peça e passa a abrir um **entalhe**.

⚠️ Há também um termo que depende de a vista ser ou não em perspectiva a entrar no sinal do lado —
a consequência observável é que **o lado que a linha come é o mesmo em vista ortográfica e em
perspectiva**, que é o que o artista espera.

---

## §5 — O plano da forma: origem e normal

Duas orientações, e elas diferem **só na normal**:

| orientação | origem | normal do varrimento |
|---|---|---|
| `VIEW` | o ponto 3D onde o gesto começou, em mundo | a **direcção da vista**, invertida |
| `SURFACE` | idem | a **normal da superfície** no ponto onde o gesto começou, levada a mundo |

Dessas duas grandezas forma-se um **plano** (ponto + normal), e esse plano é o referencial de
**todas** as profundidades da §6.

⚠️ **Divergência declarada do alvo, e ela é um FACTO de comportamento**: a normal é levada a mundo
por uma transformação que **não** compensa escala não-uniforme. Consequência observável: numa peça
com escala não-uniforme, a orientação `SURFACE` aponta para um sítio ligeiramente errado — o alvo
trata essa configuração como fora do uso previsto do modo de escultura. **N:** se o nosso módulo permitir
escala não-uniforme, esta é uma escolha **nossa** a fazer conscientemente, e não um detalhe a herdar.

---

## §6 — A profundidade: onde o volume começa e acaba

⭐ Há **dois regimes**, e eles não são variações um do outro.

### §6.1 — Regime de omissão: a profundidade TOTAL da peça

**D** (o manual diz que sem a opção do cursor a ferramenta usa *a profundidade total do objecto*)
e **F**:

1. Percorre-se **todo** vértice da peça (em mundo) e mede-se a **distância com sinal ao plano** da
   §5. `frente = mínimo`, `trás = máximo`.
2. **Enchimento** (**L** — os dois números são lidos no fonte e ⏳ **não** foram isolados na saída):
   `pad = (trás − frente) × 0,01 + 0,001`; depois `frente −= pad` e `trás += pad`.
   ⚠️ A razão é declarada e é **numérica, não estética**: afastar as tampas do prisma das faces da
   peça **evita faces coplanares** na booleana, que é a configuração em que um solucionador exacto
   é mais frágil. ⛔ Os dois termos são necessários: o relativo (`1 %`) escala com a peça, e o
   absoluto (`0,001`) cobre a peça **degenerada** cuja extensão ao longo do eixo é zero.

⇒ **Neste regime o volume ATRAVESSA sempre a peça, por construção** — a extensão vem da própria
peça. *Não existe aqui o caso «não atravessou».*

### §6.2 — Regime do cursor: uma fatia de espessura igual ao diâmetro do pincel

Com a opção ligada, a profundidade deixa de vir da peça e passa a vir do **cursor**:

1. **Um ponto médio** (**L** nas quatro combinações; ⭐ **M** no efeito agregado, tabela abaixo):
   - orientação `VIEW`: se o gesto começou **sobre** a peça, a distância com sinal desse ponto ao
     plano; se não, o **meio** de `(frente, trás)` da §6.1.
   - orientação `SURFACE`: se o gesto começou sobre a peça, **exactamente zero** — ⚠️ e a
     consequência é declarada: a forma fica **metade dentro** da superfície. Se não, o meio de
     `(frente, trás)`.
2. **Um raio**: o raio do cursor em unidades de cena. ⚠️ **Com uma fronteira nomeada:** o raio da
   sessão só é válido se o gesto começou **sobre** a peça; começando fora, ele é **recalculado** a
   partir dos ajustes do pincel na posição inicial. *Sem essa segunda rota o raio lê-se `0` e o
   volume tem espessura nula* — o alvo registou isto como defeito e corrigiu-o.
3. `frente = médio − raio`, `trás = médio + raio`.
4. ⛔ **Não há enchimento neste regime** — a razão declarada é que o enchimento alteraria a
   profundidade que o cursor acabou de definir.

⇒ ⭐⭐ **É AQUI que o volume pode NÃO atravessar a peça**, e a resposta é que **nada de especial
acontece**: a booleana corre à mesma e o resultado é um **bolso** (uma cova) em vez de um corte
passante. Não há recusa, não há aviso, não há caso especial.

**M — a varredura do raio, no MESMO gesto e na MESMA peça** (esfera de 482 V, caixa a `x ≥ 0,2`):

| raio do cursor | V depois | F depois | sobreviventes | volume com sinal | bordo | o que é |
|---|---|---|---|---|---|---|
| `0,15` | **503** | 516 | 469 | `3,9475` | `0` | ⭐ **bolso raso** — a peça **GANHA** vértices |
| `0,35` | **503** | 516 | 469 | `3,7195` | `0` | bolso mais fundo: **mesma topologia, outra geometria** |
| `0,80` | 459 | 468 | 417 | `3,2482` | `0` | já come mais do que acrescenta |
| `3,00` | **371** | **374** | **317** | **`2,6619`** | `0` | ⭐ **idêntico ao regime da §6.1** — a fatia é mais grossa que a peça |
| *(§6.1, sem cursor)* | 371 | 374 | 317 | `2,6619` | `0` | a profundidade total |

⭐⭐⭐ **Três leituras que só a tabela inteira dá:**
1. o volume com sinal **decresce monotonamente** com o raio — a fatia come progressivamente mais;
2. ⚠️ **contagem igual não é geometria igual**: `0,15` e `0,35` dão exactamente os mesmos `503`
   vértices e `516` faces e volumes **diferentes**. *Um gate que compare só contagens não vê a
   profundidade do bolso;*
3. ⭐ com raio suficientemente grande o regime do cursor **converge ao bit** para o regime de
   omissão — o que confirma que a única coisa que os separa é de **onde vem a distância**.

⚠️ **E o bolso FECHA**: `0` arestas de bordo em todas as linhas. A peça continua a encerrar volume.

⚠️ **E este é o único knob da família cujo efeito depende de onde o gesto COMEÇOU** — as quatro
combinações (`VIEW`/`SURFACE` × acertou/não acertou) dão quatro leis diferentes de ponto médio.

---

## §7 — O prisma: o polígono de ecrã varrido

### §7.1 — Contagens — **F**

Para um anel de `n` pontos de ecrã:

- **vértices**: `2n` — o anel da frente, depois o anel de trás, **na mesma ordem**;
- **faces**: `2(n−2) + 2n`, **todas triângulos**;
  - tampa da frente: `n−2` triângulos;
  - tampa de trás: `n−2` triângulos, os mesmos índices somados de `n`;
  - lateral: `2n` triângulos (dois por aresta do anel).

**M:** em modo `JOIN`, que junta o prisma sem o cortar, uma caixa (`n = 4`) acrescenta exactamente
`+8` vértices e `+12` faces à peça — `2·4 = 8` e `2(4−2)+2·4 = 12`. *A fórmula confere na saída.*

### §7.2 — Onde cada anel é pousado

O eixo é a normal da §5; as distâncias são as da §6.

- **Anel da frente:** desprojecta-se cada ponto de ecrã para 3D —
  - com `VIEW`, à profundidade da frente ao longo da vista;
  - com `SURFACE`, sobre o **plano da forma**, e depois desloca-se pela normal até à frente.
- **Anel de trás:** aqui entram os **dois modos de extrusão**, e é a única coisa que os separa:
  - `PROJECT` — desprojecta-se **outra vez** cada ponto de ecrã, agora à profundidade de trás. ⇒ o
    prisma segue o **cone de visão**: em perspectiva ele ALARGA com a distância, e a forma sai
    **cónica**.
  - `FIXED` — toma-se o vértice **da frente** e desloca-se ao longo da normal até ao plano de trás.
    ⇒ as paredes ficam **paralelas** e os ângulos rectos.

⭐⭐ **M — e é o facto mais útil dos dois modos: a divergência CRESCE com a força da perspectiva.**

| vista | `FIXED` (V / sobrev.) | `PROJECT` (V / sobrev.) | diferença |
|---|---|---|---|
| **ortográfica** | `371` / `317` | `371` / `317` | ⭐ **nenhuma — saída idêntica** |
| perspectiva, câmara a `5,0` | `371` / `317` | `372` / `312` | pequena |
| perspectiva, câmara a `2,6` | `371` / `317` | **`364` / `304`** | maior |

⇒ *a conicidade não é um efeito do modo: é o modo a deixar a perspectiva passar.* ⚠️ E repare-se
que a coluna `FIXED` **não se mexe** nas três linhas: ela é, por construção, independente da vista. Um módulo que desenhe sempre em ortográfica tem os
dois modos a custo zero de diferença — e um gate que os compare em ortográfica **não afirma nada**.

### §7.3 — A tampa de um anel CÔNCAVO

As tampas são trianguladas por uma **triangulação de polígono 2D feita sobre os pontos de ECRÃ**,
não sobre os pontos 3D. ⇒ um laço côncavo (um C) é tampado correctamente, e a mesma lista de
triângulos serve às duas tampas.

⚠️ **É por isto que a triangulação é 2D e não 3D:** no ecrã o anel é, por construção, um polígono
simples e plano; em 3D ele pode não ser plano nenhum (uma vista em perspectiva com `PROJECT` põe os
pontos em profundidades diferentes).

---

## §8 — ⭐ As normais do prisma são RECALCULADAS, e é isso que torna o sentido do desenho irrelevante

Depois de montado, o prisma passa por uma **reorientação global de faces** (**L**) — a operação de
«tornar as normais consistentes» aplicada à malha inteira dele.

⇒ **O sentido em que o artista desenhou o laço não importa.** Um laço no sentido horário e o mesmo
laço no sentido anti-horário produzem anéis com enrolamento oposto, e portanto tampas e laterais
com normais opostas; a reorientação corrige os dois para fora.

⚠️⚠️ **Esta é a armadilha mais cara desta espec para quem implementar.** Sem este passo, metade dos
gestos do artista entrega ao solucionador um volume **com o dentro e o fora trocados** — e uma
booleana de diferença com o operando invertido não falha: ela devolve **o complemento**, isto é,
apaga tudo *menos* o que se queria apagar. *O defeito não é um erro visível de geometria; é a
ferramenta a fazer o contrário do pedido, de forma intermitente, conforme o sentido do gesto.*

**M — medido, e com o laço CÔNCAVO que também exercita a §7.3:** o mesmo C desenhado no sentido
anti-horário e no horário dá saída **idêntica em todas as colunas** — `482 → 510` vértices,
`512 → 492` faces, `410` sobreviventes, volume com sinal `3,5942`, `0` arestas de bordo, nos dois.

**N:** um implementador que garanta o enrolamento **por construção** (ordenando o anel de ecrã por
área com sinal antes de varrer) obtém a mesma propriedade mais barato. A reorientação global é uma
escolha do alvo, não uma necessidade da lei.

⚠️ Note-se de passagem que este laço côncavo **acrescenta** vértices à peça (`482 → 510`): um C
corta duas vezes e deixa duas tampas, e a geometria criada supera a removida.

---

## §9 — Simetria: N booleanas, uma por passagem

A maquinaria partilhada de gestos itera as **passagens de simetria** activas na peça (as
combinações dos eixos ligados). Para **cada** passagem válida:

1. os vértices do prisma são repostos a partir de uma **cópia guardada** (**L**) das coordenadas
   originais e **espelhados** nos eixos da passagem;
2. as normais do prisma são recalculadas **outra vez** (§8 — o espelhamento inverte o enrolamento);
3. **a booleana corre de novo**, sobre o resultado da passagem anterior.

⇒ ⚠️ **Simetria custa uma booleana por passagem, não uma booleana sobre um prisma simétrico.**
Com os três eixos ligados são **oito** combinações e portanto oito booleanas encadeadas (**F** —
`2³`; ⏳ **só o caso de UM eixo foi medido**, e um gate sobre os oito fica por escrever).

**M:** com simetria em `X`, o mesmo gesto que deixava `97/97` vértices intactos do lado oposto
deixa `0/97` — os dois lados são cortados; a peça vai de `482` para `260` vértices.

⚠️ **A cópia guardada é load-bearing:** é ela que impede a passagem `k+1` de espelhar o prisma que a
passagem `k` já espelhou. *Sem ela as passagens compõem-se e o espelho vai ao sítio errado.*

⭐⭐ **E é a simetria que explica um modo AUSENTE.** A operação de **interseção** existe no motor e
**não é oferecida na interface** — a razão declarada é que ela **não funciona com simetria**: a
primeira passagem apaga tudo o que está fora do primeiro prisma, e isso inclui a metade que a
segunda passagem ia tratar. ⇒ *num desenho encadeado por passagens, a interseção não é uma
operação repetível.* **N:** se o nosso corte não encadear passagens desta maneira, a interseção
deixa de estar bloqueada — a recusa é do **encadeamento**, não da operação.

---

## §10 — A booleana e os modos

⚠️ **Descritos pelo que a SAÍDA mostra (M), não pela prosa do alvo** — cada linha é conferível na
fixtura nomeada ao lado:

| modo | o que a saída mostra | conferível em |
|---|---|---|
| `DIFFERENCE` | o que caía dentro do prisma desaparece, e a peça **continua a encerrar volume**: `0` arestas de bordo e volume com sinal ainda positivo (`4,1219 → 2,6619`) | `A1` |
| `UNION` | a peça **cresce** (`482 → 379 V` aqui, porque o prisma atravessa e funde), e a fronteira entre os dois corpos deixa de existir | `A2` |
| `JOIN` | ⛔ **nenhum solucionador corre**: o prisma é anexado inteiro e a peça fica intacta — os `482` vértices sobrevivem todos, e a contagem sobe pela fórmula da §7.1 | `A3` |
| *(interseção)* | ⛔ **existe no motor e a interface não a oferece** — a razão é a §9 | — |

As suposições que o gesto declara ao solucionador são **três** (**L**), e valem a pena porque são
o que permite ao solucionador saltar trabalho:

- sem auto-intersecções em nenhum dos operandos;
- sem componentes **aninhados** (uma caixa dentro de outra);
- ⚠️ **NÃO estanque** — o gesto **não** promete ao solucionador um volume fechado. *É a suposição
  que ele deliberadamente não faz, e é ela que permite cortar uma peça com bordo.*

⚠️ **`JOIN` não é `UNION` com outro nome** (**M**): em `JOIN` **todos** os `482` vértices da peça
sobrevivem e o prisma aparece inteiro (`+8 V`, `+12 F`); em `UNION` a peça vai a `379` vértices e
só `317` sobrevivem. *Um deles toca na peça, o outro não lhe toca de todo.*

### §10.1 — Os três solucionadores

⚠️ **A coluna que interessa é a TERCEIRA — o que cada um EXIGE da entrada** (é ela que decide se a
ferramenta corre), e ela está **medida** na §11.1, não citada:

| valor | o que o alvo lhe atribui (**D**) | ⭐ o que ele EXIGE da peça (**M**, §11.1) |
|---|---|---|
| `EXACT` | tolera operandos que se cruzam a si mesmos e entre si; em troca, **é o mais caro dos três** | corre sobre peça **com bordo** |
| `FLOAT` | o mais elementar; **não** aguenta os mesmos cruzamentos | corre sobre peça **com bordo**, e ⚠️ dá **outro** resultado que o anterior |
| `MANIFOLD` | apresentado como o de melhor relação custo/resultado | ⛔ **RECUSA** peça com bordo, em voz alta e sem lhe tocar. ⭐ **É o valor de OMISSÃO** |

⚠️ **O alvo declara uma excepção à exigência** do terceiro — o caso de uma diferença contra um
plano —, que ⏳ **não** foi exercitada por este oráculo. Fica como dívida nomeada.

**M:** nas fixturas **fechadas** os três dão saída **idêntica**, e não só na peça pequena — na
esfera de **98 306** vértices os três dão `98 306 → 58 019`, `98 304 → 57 754` faces, `57 489`
sobreviventes e o mesmo volume com sinal (`2,8373`).

⇒ ⭐ *a escolha de solucionador **não é uma escolha de resultado no caso fácil**; ela paga-se nos
casos DIFÍCEIS* — e a §11 mostra exactamente onde: numa malha com bordo os três deixam de
concordar, e um deles recusa-se a correr.

⏳ **Dívida nomeada:** o **relógio** dos três não foi isolado. As corridas deste oráculo medem o
processo inteiro (arranque do binário incluído) e ficaram todas em `0,1`–`0,3 s`; ⛔ **não afirme
daqui nenhuma razão de custo entre solucionadores.** A nota pública dos 100 k vértices (§14) é a
única afirmação de custo que esta espec faz, e ela é **D**, não **M**.

---

## §11 — As recusas, e a diferença entre recusar e não fazer nada

⚠️ **Há TRÊS comportamentos distintos, e dois deles leem-se iguais de fora.**

| situação | o que acontece | selo |
|---|---|---|
| peça em **multirresolução** | ⛔ **recusa em voz alta**, com mensagem que **nomeia o modo** | **M** |
| peça em **topologia dinâmica** | ⛔ **recusa em voz alta**, com mensagem que **nomeia o modo** | **M** |
| peça **sem faces nenhumas** | cancela **em silêncio** (sem mensagem) | **M** no fonte |
| objecto **não visível** | cancela em silêncio | **M** no fonte |
| laço com **≤ 1 ponto** | cancela em silêncio | **M** no fonte |
| ⭐ gesto **inteiramente fora** da peça | ✅ **«concluído»**, peça **byte-idêntica** (`482 → 482`) | **M** |
| ⭐ gesto **degenerado** (área zero) | ✅ **«concluído»**, peça **byte-idêntica** (`482 → 482`) | **M** |
| malha **aberta / não-manifold** com o solucionador que o exige | ⛔ **erro reportado e a peça fica INTACTA** — ver §11.1 | **M** |
| resultado **grande demais** para o solucionador | ⛔ erro reportado; a peça fica como estava | **M** no fonte |
| solucionador **ausente** da build | ⛔ erro reportado | **M** no fonte |

⚠️⚠️ **«Concluído com a peça intacta» é a resposta a um gesto que não apanhou nada** — ⛔ não é um
erro, e transformá-lo em erro seria pior: o artista que risca ao lado da peça não fez nada de
errado. ⚠️ Mas note-se que ele é **indistinguível, de fora, de um corte que falhou em silêncio** —
*esta é a fronteira onde o nosso módulo pode ser melhor que o alvo, dizendo qual dos dois foi.*

### §11.1 — ⭐⭐ A malha ABERTA é onde os três solucionadores DIVERGEM

**M**, sobre três fixturas **nossas** com bordo (um tubo aberto, um disco, e uma peça degenerada
de uma face):

| solucionador | tubo aberto (18 V, bordo 12) | disco (19 V, bordo 12) | peça de 1 face (bordo 3) |
|---|---|---|---|
| `MANIFOLD` | ⛔ **RECUSA** | ⛔ **RECUSA** | ⛔ **RECUSA** |
| `EXACT` | `→ 17 V`, bordo **`10`** | `→ 21 V`, bordo **`17`** | `→ 4 V`, bordo `4` |
| `FLOAT` | `→ 17 V`, bordo **`14`** | `→ 21 V`, bordo **`17`** | `→ 4 V`, bordo `4` |

⭐ **Três factos que só esta tabela dá:**
1. o solucionador de omissão **recusa** malha aberta, em voz alta e sem tocar na peça — ⇒ numa
   escultura que ainda não fechou, a ferramenta **não funciona** na configuração de fábrica;
2. os outros dois **prosseguem**, e ⚠️ **o corte NÃO fecha o que abriu**: o disco sai com **mais**
   bordo do que entrou (`12 → 17`);
3. ⭐⭐ os dois que prosseguem **discordam entre si** — `10` contra `14` arestas de bordo no mesmo
   tubo, com a mesma contagem de vértices. *É aqui que a escolha de solucionador deixa de ser
   indiferente, e é por isso que ela é um knob e não uma constante.*

⚠️ **Contraste com a §13, e ele é o ponto:** a garantia «o corte fecha o que abriu» vale para peças
**fechadas**. Numa peça com bordo ela **não vale**, e o alvo não avisa.

---

⚠️ **Uma recusa da booleana aborta a passagem, mas as passagens de simetria anteriores JÁ
ESCREVERAM** (§9). ⇒ numa peça com simetria, um erro no meio deixa a peça **meio cortada**. *É um
estado que o nosso undo tem de saber tratar como um passo só.*

---

## §12 — O que o corte NÃO faz (e um knob que não faz nada)

- ⛔ **Não respeita a MÁSCARA.** **M:** com a peça inteira mascarada a `1,0`, a saída é **idêntica
  em todas as colunas** à saída sem máscara (`482 → 371`, `317` sobreviventes). *A máscara é um peso
  por vértice e a booleana não tem onde a ler.*
- ⛔ **Não respeita geometria escondida nem conjuntos de faces** como filtro de selecção.
- ⛔ **Não usa a estrutura de aceleração para limitar o trabalho**: a maquinaria partilhada calcula
  que nós da árvore o gesto toca, e o corte **ignora** esse resultado — ele entrega a peça inteira
  ao solucionador.
- ⛔⛔ **O knob «só faces viradas para a vista» é MORTO neste gesto.** Ele é **registado** nas quatro
  variantes pela maquinaria partilhada — logo **aparece na interface** — e o corte **nunca o lê**
  (medido no fonte: zero ocorrências; e medido na saída: ligado e desligado dão a mesma malha).
  **M:** ligado e desligado, a saída é **idêntica em todas as colunas** (`482 → 371`, `374` faces,
  `317` sobreviventes, volume `2,6619`). ⚠️ *Ele é vivo noutros gestos que partilham a mesma
  maquinaria* — é o caso clássico desta casa: **a lente do painel é mais larga que a do
  consumidor**. **N:** ⛔ não o reproduza.

### §12.1 — ⭐ O gesto IRMÃO prova que nada disto é uma limitação da maquinaria

O mesmo gesto de linha alimenta uma **segunda** ferramenta — a que o manual recomenda no lugar
desta para malhas de alta resolução (§14). Ela é uma **deformação**, não uma booleana: empurra os
vértices para o plano da linha. E, com a **mesma** maquinaria partilhada, ela:

- ✅ **respeita a máscara e a geometria escondida** (o peso por vértice entra no cálculo dela);
- ✅ **usa** a lista de nós que o gesto tocou, logo o custo é o da **região**, não o da peça;
- ✅ funciona em **multirresolução e topologia dinâmica** — os três tipos de topologia;
- ✅ regista undo **de posições**, não de geometria.

⇒ ⭐⭐ **As ausências da §12 não são da maquinaria de gestos: são do CORTE.** *Uma implementação
nossa que leia a máscara não está a inventar uma capacidade — está a usar a que o irmão já usa.*

---

## §13 — A borda nova e as normais

**M**, em **todas** as corridas bem sucedidas de `DIFFERENCE` e `UNION` sobre peças fechadas
(esfera 482, esfera 98 306, toro, cilindro, cubo):

- **arestas de bordo: `0`** — o corte **fecha** o que abriu;
- **arestas não-manifold: `0`**;
- **volume com sinal continua POSITIVO** (`4,1219 → 2,6619` na esfera) ⇒ ⭐ **nada fica com a normal
  virada**, e a peça continua a encerrar volume.

⇒ **A borda nova é uma tampa plana**, triangulada, coplanar com a parede do prisma que a criou.
⚠️ **O manual público afirma o mesmo pelo lado do produto** (**D**): o modo de remoção é descrito
como deixando a superfície tapada, e não aberta. *Aqui o facto basta e o wording dele não importa —
a medição acima é mais forte que a frase.*

⚠️ **A tampa herda a densidade do PRISMA, não da peça** — ela é a triangulação de um polígono de
`n` pontos, e `n` é quantos pontos o gesto tinha. Um corte em caixa deixa uma tampa de `2`
triângulos, por maior que seja a peça. *Isto é visível: a região cortada fica com faces enormes ao
lado de faces finas.* **N:** é exactamente por isso que o fluxo documentado manda remalhar a
seguir — e é a nossa oportunidade de fazer melhor, sem remalhar a peça inteira.

---

## §14 — Custo

- **D:** o alvo **desaconselha** a ferramenta acima de **100 k vértices** quando o modo é
  `DIFFERENCE`/`UNION` **e** o solucionador é `EXACT`, atribuindo o custo ao facto de ser uma
  booleana, e encaminha o artista para outras ferramentas nas malhas de alta resolução.
- **F:** o custo tem duas partes que escalam de maneiras diferentes — a construção do prisma é
  `O(n)` nos pontos do gesto (trivial), e a booleana é do tamanho da **peça inteira**, não do corte.
- ⚠️ **Multiplicado pelas passagens de simetria** (§9): três eixos ligados = **oito** booleanas.

---

## §15 — A metade EXCLUÍDA, nomeada

⛔ **Os CONJUNTOS DE FACES estão fora desta espec, por exclusão permanente do dono.** O que fica de
fora, dito para que ninguém o reconstrua por engano:

- antes do corte, o alvo garante que a peça tem a camada de conjuntos de faces criada;
- depois do corte, ele procura o **próximo identificador livre** e atribui-o às faces que a
  booleana criou e que não pertencem a conjunto nenhum. O manual descreve o efeito: a geometria
  nova recebe um conjunto novo, e ao remover geometria é a **geometria interior** ao longo da
  selecção que o recebe (**D**).

⭐ **A separação é limpa e isso é um facto útil:** este passo é o **último acto** do gesto, não
altera **nenhuma** posição de vértice nem nenhuma face, e é a única parte do corte que toca
conjuntos de faces. ⇒ **removê-lo não muda a geometria em nada** — todas as medições desta espec
continuam válidas sem ele. *Não é uma metade entrelaçada; é um apêndice.*

⚠️ **O que se PERDE ao não o ter** é nomeado para ser uma decisão e não um esquecimento: sem ele, a
superfície cortada não é seleccionável como um grupo, e o artista não consegue apanhar «a face que
o corte criou» com um clique.

---

## §16 — Dívidas NOMEADAS (o que esta espec não cobre)

1. ⏳ **Os ícones e a apresentação das quatro ferramentas** dependem de assets do alvo e ficam
   **fora** (§1.5.3 da SKILL: asset é obra plena). A nossa entrada na barra é desenho nosso.
2. ⏳ **O desenho do cursor durante o gesto** (o traço do laço, o rectângulo elástico, a
   pré-visualização do volume) não foi medido — é superfície de UI, e a lei dela é da casa.
3. ⏳ **O comportamento com escala não-uniforme** (§5) é declarado não-suportado pelo alvo; não há
   lado aprovado para calibrar uma barra, então ⛔ **não há gate possível** sobre ele hoje.
4. ⏳ **O relógio comparativo entre os três solucionadores** e contra a nossa rota por voxel está
   medido só para a nossa rota e para o caso fácil do alvo — a varredura por tamanho fica por fazer.
5. ⏳ **O caso de peça com BORDO** (malha aberta) por solucionador está medido em fixturas nossas e
   o resultado vive no ledger; ⚠️ esta espec só afirma o que o fonte declara (§11).

---

## §17 — As fixturas

Vivem em `docs/3D/cleanroom/fixtures/trim/`, com README de proveniência. ⭐ **A entrada é NOSSA** —
as malhas saem de `ph2d_mesh::shapes` e `ph2d_mesh::shapes_open`, exportadas por um gerador do
repo; ⛔ **nenhum asset do alvo entra**, nem como entrada.

---

## §18 — Gates propostos (o que se pode cobrar de uma implementação)

1. **A fórmula do prisma**: para um anel de `n` pontos, a malha varrida tem `2n` vértices e
   `2(n−2)+2n` faces triangulares. *(prova por construção; controlo: `n = 4` dá `8` e `12`)*
2. **O corte é LOCAL**: numa esfera de 98 306 vértices cortada por uma caixa, **todos** os vértices
   a mais de `0,7` do plano de corte saem **bit-a-bit iguais** aos de entrada. ⚠️ **Este é o gate
   que a rota por voxel não pode passar** — e é por isso que ele é o gate certo.
3. **O resultado fecha**: `0` arestas de bordo, `0` arestas não-manifold e volume com sinal
   **positivo**, sobre as cinco fixturas fechadas.
4. **O sentido do gesto é irrelevante**: o mesmo laço côncavo desenhado nos dois sentidos dá a
   **mesma** malha. *(o gate que apanha a armadilha da §8)*
5. **Ortográfica não distingue as extrusões**: em vista ortográfica, `FIXED` e `PROJECT` dão saída
   idêntica; em perspectiva, **não**. ⚠️ **As duas metades são obrigatórias** — só a primeira seria
   satisfeita por uma implementação que ignorasse o modo.
6. **`JOIN` não toca na peça**: todos os vértices de entrada sobrevivem, e a contagem sobe
   exactamente pela fórmula do gate 1.
7. **O limite ao segmento abre um entalhe**: a contagem de vértices **sobe**, não desce.
8. **Um gesto fora da peça é um no-op byte-idêntico** — ⛔ e não um erro.
9. **A simetria multiplica as operações**: com `X` ligado, o lado oposto deixa de estar intacto.
10. ⛔ **O knob morto não renasce**: um censo que prove que nenhum parâmetro registado no painel do
    corte deixa de ter consumidor. *(§12 — o gate que o alvo não tem)*
