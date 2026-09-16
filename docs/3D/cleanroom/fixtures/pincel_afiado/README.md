# Fixtures — o PINCEL DE TRAÇO AFIADO, do ORÁCULO, sobre malhas NOSSAS

⭐ Estes ficheiros são os vectores de teste da [`SPEC_pincel_afiado.md`](../../SPEC_pincel_afiado.md)
§11 — **80** traços sobre malhas geradas por nós, com as posições e normais de repouso, as posições
de saída, e (conforme a família) os estados intermédios dab a dab ou passagem a passagem.

⚠️ **A contagem sai do directório, nunca desta prosa:**
`find docs/3D/cleanroom/fixtures/pincel_afiado -name '*.txt.gz' | wc -l`.

| família | ficheiros | o que ela fixa |
|---|---|---|
| [`lei/`](lei/) | **17** | a lei de **um** dab, um knob de cada vez (curva, força, dureza, raio da normal, máscara, faces de frente, pegada projectada, plano do pen-down, controlo do desenho comum) |
| [`cadeia/`](cadeia/) | **10** | **cadeias** de dabs dados por script, com o estado depois de CADA dab (`d<j>`): de onde se mede a distância, de onde vem a normal, o que o acumular muda, a auto-limitação |
| [`produto/`](produto/) | **35** | o **produto**: traços arrastados com o rato, os valores de fábrica, o pincel tal como nasce no catálogo, oito passagens contínuas e separadas, três densidades, superfície curva, malha em triângulos, a ablação e o motor do alvo com os valores da nossa casa |
| [`detector/`](detector/) | **15** | o **passo** do traço: um salto curto do rato e quantos dabs ele deposita |
| [`artefacto_caixa/`](artefacto_caixa/) | **3** | um **artefacto do alvo que NÃO se copia** (ver §4 abaixo) |

## §1 Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness: grelhas de quadriláteros de lado `2,0` (`48×48`, `96×96`, `192×192`) com campo de altura analítico (plano · bossas `0,05·sin(6x)·sin(6y)` · cilindro de raio `1,2`), e a mesma grelha partida em triângulos. ⛔ Nenhum asset do alvo |
| **Quem calculou** | o binário Blender **5.2.1 LTS**, corrido pelo subagente-E **fora da árvore** (zona negada ao Implementador), numa janela dentro de um compositor **virtual** — nunca no ecrã do dono. Dois caminhos: dabs dados por **script** (famílias `lei/` e `cadeia/`) e um **arrasto de rato simulado** evento a evento (as outras três) |
| **O pincel** | ⭐ em 78 fixturas o harness **cria** o pincel e **escreve** todos os valores que decidem o resultado (estão no cabeçalho). Nas **2** de catálogo (`produto/*_de_catalogo_*`) o pincel é o de **fábrica**, carregado pelo **próprio programa** num arranque de fábrica, e os valores do cabeçalho foram **lidos** dele — nenhum ficheiro de pincéis foi aberto pelo harness. ⭐ A de catálogo do pincel afiado coincide com a de valores escritos a `1,2e-7` (ruído de `f32`) ⇒ **os valores escritos são o pincel de fábrica inteiro** |
| **Estatuto legal** | ⭐ **dados** — GPLv2 §0: a saída só é coberta se o conteúdo for obra baseada no programa, e posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do Implementador (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-16 |

## §2 O formato

Texto comprimido (`gzip`, `mtime` fixo a `0` ⇒ reprodutível byte a byte). Cabeçalho de linhas
`# chave: valor`, depois um bloco por linha:

| prefixo | o que é |
|---|---|
| `r x y z` | posição de **repouso**, uma linha por vértice, pela ordem dos índices |
| `n x y z` | **normal** do vértice no repouso (a que o próprio alvo calculou) |
| `m v` | **máscara** por vértice (`1` = travado) — só em `lei/mascara_metade` |
| `c x y z` | nas famílias por script: o cursor de **cada** dab, pela ordem. Nas de arrasto: só o início e o fim da linha do traço, na superfície de repouso |
| `d<j> i x y z` | posição do vértice `i` **depois do dab `j`** (só os que diferem do repouso) — cada `j` é uma corrida **própria** com os `j` primeiros pontos `c` |
| `p<k> i x y z` | posição do vértice `i` **depois da passagem `k`** de um traço arrastado (só os que diferem do repouso) |
| `s x y z` · `s i x y z` | a **saída** final: completa (uma linha por vértice) nas famílias por script de `2 401` vértices; esparsa (com o índice) nas outras |

⚠️⚠️ **O cabeçalho é a fonte; esta prosa nunca é.** Toda grandeza que enquadra o traço está lá,
uma por linha: superfície (com a caixa envolvente), vista, píxeis por unidade, o píxel do
pen-down e o passo de um píxel no mundo, o raio efectivo, o caminho do traço (passagens,
contínuo ou separado, saltos), o cursor (dado ou vivo), as normais entre dabs, pressão,
**modificador** (Ctrl), máscara, simetria, auto-máscara, textura, e os valores do pincel.

⚠️ **As chaves do cabeçalho estão em vocabulário do DOMÍNIO; os VALORES de enumeração são os
públicos da API do alvo**, porque são eles que tornam a fixtura regenerável (SKILL_Cleanroom
§4.1.13). A tradução deles para o nosso vocabulário:

| valor no cabeçalho | o que quer dizer nesta casa |
|---|---|
| `pincel: DRAW_SHARP` / `DRAW` | o pincel **afiado** / o **desenho comum** do alvo |
| `curva: POW4` · `SMOOTH` · `SHARP` · `CONSTANT` | `(1−u)⁴` ([`Falloff::Sharper`]) · `3t²−2t³` com `t = 1−u` ([`Falloff::Smooth`]) · `(1−u)²` · `1` |
| `direccao: SUBTRACT` / `ADD` | afunda ao longo da normal / levanta |
| `metodo_do_traco: SPACE` / `DOTS` | dabs a distância fixa no ecrã / um dab por evento (inerte no caminho por script) |
| `espacamento_medido_em: VIEW` | o passo mede-se em píxeis de **ecrã** |
| `referencial_da_direccao: AREA` | a direcção é a **normal da área** (espec §2.3) |
| `forma_da_pegada: SPHERE` / `PROJECTED` | distância em 3D / distância no plano da vista |
| `unidade_do_diametro_proprio` · `cena_unidade_do_diametro: VIEW` | o diâmetro mede-se em píxeis |

⚠️ **Nem toda chave é lida por toda família.** O espaçamento, a atenuação e o método do traço são
**inertes** no caminho por script (o cabeçalho di-lo na linha `caminho_do_traco`); o diâmetro
próprio do pincel é inerte quando o da cena é unificado. *Uma coluna que só uma família lê, mas
que aparece em todas, é exactamente como alguém inventa uma dependência que não existe.*

## §3 Os dois caminhos do alvo, e o que cada um NÃO faz

1. ⚠️⚠️ **O caminho por script NÃO refresca as normais de vértice entre dabs.** Um dab lê as
   normais que a malha tinha no início da corrida (o bloco `n`). Medido: com as normais do
   repouso a lei reproduz as cadeias a `≤ 2,4e-8` (plano) e `≤ 1,9e-6` (bossas); com normais
   recalculadas a cada dab o erro sobe a `1,3e-2`–`6,5e-2`. ⇒ **a bancada destas duas famílias
   usa o bloco `n` para TODOS os dabs**. Não é a lei do produto — é o modo como este caminho do
   alvo foi medido.
2. ⚠️ **No caminho por script o cursor é IMPOSTO** (o bloco `c`): nas cadeias «vivas» ele foi
   posto pelo harness no acerto de um raio vertical sobre a superfície **viva**, que é o que o
   produto faz com o acumular desligado. A metade «com acumular ligado o cursor é o do
   pen-down» não se mede por aqui — está em `produto/ablacao_acumular`.
3. ⭐ **O caminho arrastado refresca as normais** (uma vez por evento de rato) — e é por isso que
   as famílias de produto têm de ser comparadas com normais **vivas**. As fotos `p<k>` são tiradas
   depois de uma **pausa** no fim de cada passagem; o relógio do desenho do alvo nessa pausa
   move o resultado em `≤ 1,8e-4` contra uma corrida que acaba naquela passagem. Esse é o chão
   de uma comparação passagem a passagem.
4. ⚠️ **Nas famílias de arrasto o cursor é o do PRODUTO**: o alvo pica a superfície ele próprio,
   a cada dab. As linhas `c` não são impostas.

## §4 ⛔ O artefacto que NÃO se copia (família `artefacto_caixa/`)

No arrasto numa vista **ortográfica**, o alvo recorta o raio do cursor à **caixa envolvente** da
malha antes de a picar. Num plano perfeito essa caixa tem **espessura zero** na direcção da vista
(mais uma folga de `1e-3`), e um vinco que desça mais do que isso fica **fora** dela: o acerto
seguinte falha e **o dab perde-se em silêncio**.

- `caixa_fina_salto_14px` — com um salto de `14` px o alvo deposita **só** o dab do pen-down
  (`446` vértices movidos, fundo `0,0148`), contra os três dabs e `507` vértices (`0,0323`) da
  mesma corrida numa superfície com caixa espessa (`detector/salto_14px`).
- `caixa_funda_somar` · `caixa_fina_ctrl` — com a caixa espessa só **para baixo** (ou nenhuma), um
  traço que **levanta** sai dela: o resultado difere `1,5e-2` e `2,4e-2` das corridas com caixa
  espessa para os dois lados.

⇒ **Toda fixtura das famílias `produto/` e `detector/` corre numa superfície com um vértice de
canto a `z = −1` e outro a `z = +1`** (as de triângulos, que só afundam, só com o de baixo; o
cilindro tem espessura própria), para a caixa engrossar; o cabeçalho di-lo. *A nossa casa não recorta o raio pela caixa, e não o deve
passar a fazer.*

## §5 Regenerar

⛔ Acto do E. O harness, o escritor e a bancada de reprodução vivem na zona do alvo; o que o I
precisa de saber para LER estas fixturas está no cabeçalho de cada uma e na espec.
