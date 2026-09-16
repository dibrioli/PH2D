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

---

## ⭐ 2.ª missão (2026-09-16): `fabrica_e_traco/` e `cursor_na_superficie/`

Os vectores da [espec §14](../../SPEC_pincel_de_plano.md) — **o pincel tal como nasce** e **o traço
que a mão faz**, que as famílias acima não dizem (cada valor delas foi escrito à mão, e os dabs
foram dados por script).

⚠️ **O inventário deriva-se do directório** (`ls <pasta> | grep -c '\.gz$'`): `fabrica_e_traco/`
**44** + a tabela `valores_de_fabrica.txt` · `cursor_na_superficie/` **11** · a pasta inteira
**155** traços.

### Proveniência — o que muda em relação às famílias acima

| | |
|---|---|
| **Malha** | ⭐ **nossa**: grelha `96×96` de quadriláteros, lado `2,0`, relevo `0,02 · cos(kx) · cos(ky)` com comprimento de onda `0,5` (cristas **e** vales) — `9 409` vértices. As `cursor_na_superficie/` usam as malhas das células homónimas de `lei/` |
| **Traço arrastado** (`fab_*`, `copia_*`, `ctl_*_traco_arrastado_*`, `ablacao_*`, `passo_salto_*`) | eventos de **rato simulados** dentro de uma janela do alvo num compositor **virtual** (nunca o ecrã do dono), 1 px por evento, `200` px por unidade ⇒ o raio de fábrica (`50` px) vale **`0,25`**. O espaçamento e a atenuação **do alvo** correm |
| **Pincel de fábrica** | carregado pela porta de activação do catálogo, como o artista o carrega; o cabeçalho diz **qual** perfil (`pincel_de_origem`, o nome público dele — é a chave de regeneração). ⛔ O ficheiro de pincéis do alvo **nunca** foi lido como fonte nem copiado: os valores vêm das **propriedades** do pincel carregado |
| **Traço por script** (`ctl_*_traco_do_corpus_*`, `passo_script_*`) | a mesma porta das famílias acima; o cabeçalho diz **onde está o cursor** (`cursor:`) — nos pontos dados (`z = 0`) ou **na superfície** |
| **Valores de fábrica** (`valores_de_fabrica.txt`) | os **cinco** perfis do pincel de plano, lidos das propriedades — *facto de interface* |

### ⚠️⚠️ O cursor das famílias da 1.ª missão está FORA da superfície

Nas famílias acima, o cursor está nos pontos do bloco `c` **tal como dados**, em `z = 0` — num
relevo isso é **até `0,54 R`** fora da superfície. **O cabeçalho e o bloco `c` enquadram-nas por
inteiro** e a lei reproduz-se sobre elas; mas o **tamanho** de um efeito lido nelas não é o que um
artista vê. ⛔ **As células de rampa e de degrau movem `274` e `271` vértices com o cursor fora — e
`0` com ele na superfície** (`cursor_na_superficie/sup_rampa`, `sup_degrau`).

### ⚠️ O traço arrastado NÃO é determinístico ao bit

A mesma entrada duas vezes difere **`1,49e-08`** (`fab_aparar_continuo_4` contra `_repete`) — um
passo de `f32`. O caminho por script era byte-idêntico. ⇒ *uma barra sobre estas fixturas não pode
ser `0`.*

### ⭐ A prova de que os valores de fábrica lidos estão COMPLETOS

`copia_aparar_continuo_4` e `_8` são um pincel **nosso** com os valores da tabela escritos: diferem
do perfil de fábrica **`7,45e-09`** — abaixo do ruído de determinismo. ⇒ *nenhum valor que decide o
resultado ficou por ler.*

### A régua da planura

Resíduo (RMS e máximo) ao plano de mínimos quadrados dos vértices com **`|x| ≤ 0,5`** e
**`|y| ≤ 0,1`** (`441` vértices) — a faixa que o traço varre por inteiro —, dividido pelo do
repouso (`0,01153`). O cabeçalho de cada fixtura arrastada repete-a (`regua:`).
