# Fixtures — os quatro pincéis desbloqueados, do ORÁCULO, sobre malhas NOSSAS

⭐ **Estes ficheiros são os vectores de teste da [`SPEC_unblocked_brushes.md`](../../SPEC_unblocked_brushes.md) §7**
— traços scriptados sobre malhas geradas por nós, com as posições de repouso, as de saída e (nas
famílias de multirresolução) as da **superfície de referência**.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness: um plano triangulado `40×40` de lado `2,0` com jitter determinístico nos vértices interiores, um plano `4×4` de quads que recebe multirresolução, e esferas de quads. ⛔ Nenhum asset do alvo |
| **Quem calculou** | o binário Blender **5.2.1 LTS**, corrido pelo subagente-E **fora da árvore** (`~/Referencias/blender-unblocked/oracle/`, ⛔ negado ao Implementador) por um traço scriptado sobre a API pública. Um pincel do alvo é importado **só para existir um pincel do tipo certo** (a API não deixa criar+activar um de raiz); **todos** os parâmetros que decidem o resultado são reescritos e vão no cabeçalho |
| **Estatuto legal** | ⭐ **dados** — GPLv2 §0: a saída só é coberta se o conteúdo dela for obra baseada no programa, e posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do Implementador (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-13 |

## O formato

Texto comprimido (`gzip`, `mtime` fixo a `0` ⇒ o ficheiro é reprodutível byte a byte). Cabeçalho de
linhas `# chave: valor`, depois um bloco por linha:

| prefixo | o que é | onde existe |
|---|---|---|
| `r x y z` | posição de **repouso** (antes do traço) | todas |
| `l x y z` | posição na **superfície de referência** (a superfície-limite da subdivisão — espec §2) | `apagador`, `esfregao` |
| `s x y z` | posição de **saída** (depois do traço) | todas |
| `c x y z` | os pontos do **cursor**, pela ordem | todas |
| `fr i j k …` | **faces antes** | `densidade` |
| `fs i j k …` | **faces depois** | `densidade` |

⭐ **O bloco `l` é o que torna as duas famílias de multirresolução utilizáveis HOJE**, antes de
existir o nosso avaliador de ponto-limite (espec §2.3): ele deixa cobrar a **lei** do pincel
isoladamente de quem calcula a referência.

⚠️ **As chaves `detalhe_*` do cabeçalho só significam alguma coisa na família `densidade`.** Nas
outras três elas são despejadas pelo harness por uniformidade e são **inertes** — a projecção e os
dois de multirresolução não correm passe de topologia nenhum. *Uma coluna que só é lida por uma
família, mas aparece em todas, é exactamente como alguém inventa uma dependência que não existe.*

## O traço

Vista ortográfica; o cursor anda em linha recta ao longo de `+X`, comprimento `0,6`, em `6` passos
iguais (`*_1passo` usa **um**; `*_parado*` mantém o cursor no sítio). O raio do pincel em espaço de
objecto e todos os ajustes estão **no cabeçalho de cada fixture** — ⛔ leia-o lá, nunca desta página.

### As excepções ao parágrafo, DERIVADAS (não escritas à mão)

A régua: para cada família, o valor **mais comum** de cada chave do cabeçalho é a base, e o que
diverge está aqui. ⚠️ Ela é re-derivável de `zcat */*.txt.gz | grep '^#'` — *uma lista de excepções
sem a régua ao lado não é auditável, e quem acrescenta uma fixture herda a régua que não vê.*

| família | base (o que o parágrafo acima descreve) | excepções |
|---|---|---|
| **`densidade`** (14) | raio `0,3` · força `1,0` · curva *Smooth* · dureza `0` · detalhe **constante**, resolução `3,0`, refino *Subdivide Collapse* · sem auto-alisamento | **refino**: `…_grossa_so_colapsa` = *Collapse* · `…_refino_so_subdivide` = *Subdivide* — **tipo de detalhe**: `…_detalhe_manual` = *Manual* · `…_detalhe_pincel25` = *Brush* — **resolução**: `…_resolucao6` = `6,0` — **raio**: `…_raio05_1passo` = `0,5` — **auto-alisamento**: `…_autosuave05` = `0,5` |
| **`apagador`** (10) | raio `0,35` · força `1,0` · curva *Smooth* · dureza `0` | **curva**: `…_forca1_constante_1passo`, `…_forca05_constante_1passo` = *Constant* — **dureza**: `…_dureza05` = `0,5` — **força**: `…_forca05`, `…_forca05_constante_1passo`, `…_parado12` = `0,5` |
| **`esfregao`** (12) | raio `0,35` · força `1,0` · curva *Smooth* · modo *Drag* · três níveis | **curva**: os três `…_constante_1passo` = *Constant* — **modo**: `…_pinch*` = *Pinch* · `…_expand*` = *Expand* — **força**: `…_drag_forca05` = `0,5` |
| **`projectar`** (24) | raio `0,35` · força `1,0` · curva *Smooth* · dureza `0` · raio de vista · folga `0` · um sentido · *Add* | **curva**: os dois `…_constante_1passo` = *Constant* — **direcção do raio**: `…_normal_plano_x`, `…_normal_plano_area` = *Plane Normal* — **folga**: `…_mindist01`, `…_mindist01_acima_bidir` = `0,1` · `…_mindist06` = `0,6` — **dureza**: `…_dureza05` = `0,5` — **força**: `…_forca05`, `…_forca05_constante_1passo` = `0,5` — **dois sentidos**: `…_acima_bidir`, `…_dois_lados_bidir`, `…_mindist01_acima_bidir` = ligado — **sentido**: `…_subtrair` = *Subtract* |

⚠️ **`…_invertido` não aparece em excepção nenhuma de cabeçalho**, e isso é load-bearing: a inversão
dele veio pelo **gesto** (segurar Ctrl durante o traço), que não é uma propriedade do pincel e por
isso não viaja no cabeçalho. O par `projectar_invertido` (gesto) × `projectar_subtrair` (propriedade)
mede **`0,0` de diferença** — espec §1.2.

## As três coisas que uma leitura rápida entende ao contrário

1. ⛔ **Os blocos `s` de `apagador` e `esfregao` NÃO são goldens da nossa saída** sobre a mesma
   entrada. A referência do alvo é outra superfície que a nossa (espec §2.2), logo a resposta certa
   é outra. Eles são goldens **da lei** quando alimentada com o bloco `l` da própria fixture.
2. ⚠️ **`*_repete` não é uma fixture a mais: é o CONTROLO da barra de paridade.** `apagador_base` e
   `apagador_base_repete` são a MESMA corrida duas vezes, e elas diferem `1,788e-07` — ⇒ **nenhuma
   barra destas duas famílias pode ser mais apertada que isso** (espec §8.1).
3. ⛔⛔ **`densidade_regular` e `densidade_regular_repete` NÃO concordam** — `1 360` contra `1 367`
   vértices. Elas estão aqui **como prova disso**, não como golden: o detalhe *relativo* depende do
   raio em pixels e o arnês não prega o estado da vista. *Um par que discorda é uma medição sobre a
   régua, não sobre o produto* — o detalhe **constante** (`densidade_base` × `…_repete`) é
   **byte-idêntico**, e é sobre ele que se mede.
