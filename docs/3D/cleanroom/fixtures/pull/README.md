# Fixtures — os gestos de PUXAR do ORÁCULO, sobre malhas NOSSAS

⭐ **Estes arquivos são os vectores de teste da [`SPEC_pull_brushes.md`](../../SPEC_pull_brushes.md) §11**
— um traço scriptado por lei, com as posições de repouso e as posições depois do traço.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **nossas**, geradas pelo próprio harness: grelha plana `64×64` de lado `3,0` (4 225 vértices) · grelha plana `8×8` de lado `3,0` (81 vértices) · esfera UV `96×64` de raio `1` (6 082 vértices) · dois «telhados» `128×128` de lado `3,0` (16 641 vértices), com a cumeeira ao longo de `Y` em `x = ∓1` e abas a `30°`. ⛔ Nenhum asset do alvo |
| **Quem calculou** | o binário do alvo (5.2.1 LTS, hash de build `9e2066aef7ef`), corrido pelo E **fora da árvore** (`~/Referencias/blender-pull/oracle/`, ⛔ negado ao I) com um traço scriptado; o preset do binário é activado **só para existir um pincel do tipo certo** (a API não deixa criar+activar um de raiz), e **todos** os parâmetros que decidem o resultado são reescritos e gravados no cabeçalho de cada fixture |
| **Estatuto legal** | ⭐ **dados** — «the output from the Program is covered only if its contents constitute a work based on the Program» (GPLv2 §0): posições de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do I (o harness vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-13 |

## Formato

- `<superficie>.repouso.txt.gz` — as posições de repouso, **uma por superfície**, partilhadas:
  cabeçalho `vertices N`, depois `N` linhas `r x y z`.
- `<tag>.deformado.txt.gz` — cabeçalho de `20` chaves, depois `caminho N` + `N` linhas `c x y z`
  (o percurso do cursor, em espaço de objecto), depois `vertices M` + `M` linhas `d x y z`
  (as posições **depois** do traço). Os índices de vértice batem com os do `.repouso` da mesma
  superfície.
- Todas as chaves e etiquetas estão em **vocabulário do domínio** — ⛔ nenhum nome interno do alvo
  entra aqui (SKILL §5).

⚠️ **Leia sempre o cabeçalho da fixture, nunca esta prosa** — o cabeçalho é a fonte.
As omissões comuns a quase todas: raio `0,35` em espaço de objecto · força `1,0` · curva *suave* ·
dureza `0` · alcance em *esfera* · fator do raio da normal `0,5` · peso da normal `0` ·
aperto `0,5` (o neutro) · alvo de deformação *geometria* · sem espelho · vista de topo ·
pressão `1` · sem textura · sem auto-máscara · sem acumulação · sem face frontal.

### Excepções ao parágrafo acima, com a régua ao lado

A régua é: comparar o cabeçalho de cada `.deformado.txt.gz` com as **nove** grandezas que o
parágrafo fixa sem ressalva (`raio · forca · curva · dureza · forma_do_alcance ·
fator_raio_da_normal · peso_da_normal · aperto · alvo_da_deformacao`), mais `vista`,
`espelho_x`, `vertice_activo`, `silhueta` e `modo_de_deslize`.

| grandeza | quantas | quais |
|---|---|---|
| **forca** ≠ `1,0` | `5` | `polegar_plano_forca05` · `empurrao_plano_forca05` (`0,5`) · `agarrar_grelha8_vertativo_sim_forca04` (`0,4`) · `deslizar_plano_arrastar_forca05` (`0,5`) · os `deslizar_*_origem` (`0,25`) |
| **fator_raio_da_normal** ≠ `0,5` | `2` | `polegar_esfera_topo_raionormal03` · `empurrao_esfera_topo_raionormal03` (`0,3`) |
| **vista** ≠ topo | `2` | `polegar_esfera_frente` · `empurrao_esfera_frente` |
| **espelho_x** ligado | `2` | `polegar_plano_espelhox` · `empurrao_plano_espelhox` |
| **alvo_da_deformacao** ≠ geometria | `5` | os cinco `*_alvo_pano` (um por gesto) — ⭐ **load-bearing: os cinco movem ZERO vértices** (§9.2 da espec) |
| **silhueta** ligada | `5` | `agarrar_plano_silhueta_sim` · os quatro `agarrar_telhado_*_silhueta_sim*` — ⭐ **load-bearing: os cinco movem ZERO vértices**, e a causa é do HARNESS, não da lei (§12.1) |
| **vertice_activo** ligado | `2` | `agarrar_grelha8_vertativo_sim` · `…_sim_forca04` |
| **modo_de_deslize** ≠ arrastar | `4` | `deslizar_plano_apertar_origem` · `deslizar_plano_expandir_origem` · `deslizar_esfera_topo_apertar` · `deslizar_esfera_topo_expandir` |

## O que cada grupo prende

- **`polegar_*` (o gesto ancorado).** A pegada e a normal congelam no pen-down e o gesto que chega
  é o deslocamento **TOTAL**. ⭐ `polegar_plano_origem` e `polegar_plano_passos24` dão
  `max_deslocamento` **idêntico** (`0,600000`) com `12` e `24` eventos: é a prova de que a lei não
  depende da taxa de amostragem. Os `_k02…_k11` truncam o percurso e prendem que só o total conta.
- **`empurrao_*` (o gesto que viaja).** ⭐ O contraste: `empurrao_plano_origem` (`0,599054`) e
  `empurrao_plano_passos24` (`0,595226`) **diferem** — esta lei é uma integral ao longo do
  caminho e depende de quantos eventos houve. `_parado` e `_ida_volta` prendem que ela transporta
  matéria (voltar não devolve o barro).
- **`agarrar_*_vertativo_*`.** A âncora em cima de um VÉRTICE em vez do ponto sob o cursor: numa
  grelha grossa (`8×8`) isso muda o pico de `0,446338` para `0,500001` e move um vértice a mais.
- **`deslizar_*`.** As três direcções (arrastar/apertar/expandir) sobre o mesmo caminho e a mesma
  malha; `_parado` e `_passos24` prendem a dependência do caminho.
- **`*_alvo_geometria` / `*_alvo_pano`.** O par por gesto que mede o alvo da deformação. ⛔ O lado
  `pano` move **zero** nos cinco.

⚠️ **Os cinco `*_alvo_pano` e os cinco `*_silhueta_sim` são fixtures de valor ZERO, e estão aqui de
propósito:** elas são a prova medida de que o oráculo scriptado **não exibiu** aqueles dois
comportamentos — a espec §9.2 e §12.1 dizem o que isso significa e o que falta para os fechar.
*Uma fixture que mede zero não é uma fixture inútil; é a que impede alguém de afirmar que mediu.*

## Verificar

As posições são `%.6f` em espaço de objecto. O parser é trivial (linhas `r`/`c`/`d`); o gate do
produto compara o nosso resultado com a coluna `d` da fixture, sobre a malha de `.repouso`,
com a barra que a espec §10 deriva — ⛔ **nunca um epsilon de conforto**.
