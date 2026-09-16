# Fixtures — o PINCEL DE PLANO, do ORÁCULO, sobre malhas NOSSAS

⭐ Estes ficheiros são os vectores de teste da [`SPEC_pincel_de_plano.md`](../../SPEC_pincel_de_plano.md)
§11 — **100** traços scriptados sobre malhas geradas por nós, com as posições e normais de
repouso, as posições de saída, e os pontos do cursor pela ordem.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness: grelhas de quadriláteros `48×48` de lado `2,0` com campo de altura analítico (plano · sulco `sin(7x)` · bossas `sin(9x)·sin(9y)` · rampa · degrau) e uma esfera de quadriláteros por projecção de um cubo `24×24`. ⛔ Nenhum asset do alvo |
| **Quem calculou** | o binário Blender **5.2.1 LTS**, corrido pelo subagente-E **fora da árvore** (`~/Referencias/blender-trim-pincel/oracle/`, ⛔ negado ao Implementador) por traços scriptados sobre a API pública, sem interface visível (janela dentro de um compositor virtual). ⭐ **Nenhum ficheiro de pincéis do alvo é aberto**: o harness cria o pincel dele próprio e só lhe escreve o tipo — **todos** os parâmetros que decidem o resultado são escritos explicitamente e vão no cabeçalho |
| **Estatuto legal** | ⭐ **dados** — GPLv2 §0: a saída só é coberta se o conteúdo dela for obra baseada no programa, e posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do Implementador (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-15 |

## O formato

Texto comprimido (`gzip`, `mtime` fixo a `0` ⇒ reprodutível byte a byte). Cabeçalho de linhas
`# chave: valor`, depois um bloco por linha:

| prefixo | o que é |
|---|---|
| `r x y z` | posição de **repouso** (antes do traço) |
| `n x y z` | **normal** do vértice no repouso |
| `s x y z` | posição de **saída** (depois do traço) |
| `c x y z` | os pontos do **cursor**, pela ordem |

⚠️⚠️ **O cabeçalho é a fonte; esta prosa nunca é.** Toda grandeza que enquadra o traço está lá,
uma por linha — e as chaves do cabeçalho usam vocabulário do **domínio** à esquerda; onde o
identificador **público** da API do alvo aparece, é porque é ele que torna a fixture
**regenerável** (SKILL_Cleanroom §4.1.13).

⚠️ **Nem toda chave do cabeçalho é lida por toda família.** O ângulo e o modo da lâmina só
significam alguma coisa na família `corte/corte_laminav_*`; a altura, a profundidade, o modo de
inversão e as duas firmezas só significam alguma coisa no pincel de plano. Nas outras elas são
despejadas por uniformidade e são **inertes**. *Uma coluna que só uma família lê, mas que aparece
em todas, é exactamente como alguém inventa uma dependência que não existe.*

## Os traços

Salvo indicação em contrário: malha de `2 401` vértices, pincel de raio `0,4`, força `1,0`, curva
`SMOOTH`, dureza `0`, acumular desligado, traço de **8** dabs ao longo de `x` com comprimento
`0,8`, vista ortográfica de topo.

As famílias `lei/` e `lados/` usam **dois** dabs e curva **constante** de propósito: o primeiro
dab não move nada (espec §1), logo o traço tem **um dab efectivo**, e com a curva constante e
força `1` os vértices tocados aterram **exactamente** no plano — é isso que torna o plano
**recuperável** da fixtura por ajuste, e é sobre essa recuperação que a espec §4.1 reproduz a lei.

## ⚠️ As três fixtures que existem para NÃO se mexerem

| fixtura | o que ela prova |
|---|---|
| `lei/lei_primeiro_dab` | um traço de **um** dab move `0` de `2 401` vértices |
| `lados/traco_inerte` | com os dois tectos a zero, o pincel é alcançável e **inerte** |
| `superficies/superficie_lisa` | sobre uma superfície já plana o pincel **pára sozinho** |

## ⚠️ E as DUAS que existem para ser MUDAS

`firmeza/firmeza_centro_bossas_00` e `_10` diferem apenas por ruído de `f32` — **de propósito**.
Elas são o controlo da espec §6.2: numa superfície simétrica aquele knob é mudo **por mecanismo**,
e a primeira medição desta obra quase o arquivou como morto. *Uma barra calibrada sem o lado vivo
mede outra coisa.*

## ⛔ E as que fixam knobs MORTOS no alvo

`corte/corte_barro_*`, `corte/corte_polegar_*` e `corte/corte_laminav_desloc*` fixam pares de
controlos que o alvo **mostra no painel** (ou, num caso, esconde) e que **não alteram a saída**.
Elas estão aqui para que a nossa versão não herde a doença por simetria — ver a espec §7.3 e o
gate **G-13**.
