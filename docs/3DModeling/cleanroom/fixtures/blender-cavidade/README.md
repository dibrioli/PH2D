# O oráculo da TINTA POR CURVATURA — Blender 5.2.2 LTS, CORRIDO

> ⚠️ **O alvo é GPL-2.0-or-later ⇒ PAREDE.** Nada do fonte dele foi lido. O que está aqui é a
> **SAÍDA** do programa a correr sobre uma entrada **NOSSA** — números de posições e de cores sobre
> uma caixa que nós especificámos. GPLv2 §0: a saída não é obra baseada no programa.
>
> ⚠️ **Os identificadores de propriedade citados abaixo (`cavity_ridge_factor`, …) são API PÚBLICA**,
> obtidos por **introspecção RNA do programa a correr** (`bpy.types.X.bl_rna.properties`) — a mesma
> classe que o `--doctool` do Godot, e a mesma que os seis nomes já registados no `CLAUDE.md §5`.
> Nenhum nome interno, nenhum símbolo de implementação.

## Proveniência

| | |
|---|---|
| binário | `blender` 5.2.2 LTS, hash `d13f752e3b9c`, build 2026-09-15 |
| pacote | `blender 17:5.2.2-1`, licença `GPL-2.0-or-later` (`pacman -Qi blender`) |
| porta | `blender -b --factory-startup -noaudio --python <harness> -- <args>` |
| harness | `~/Referencias/blender/oracle/` — **fora da árvore** (arsenal §3) |
| entrada | **nossa**: caixa de meia-extensão `b = 1,0` com as 12 arestas filetadas em `r = 0,2` |
| data | 2026-09-19 |

⚠️ **`--factory-startup` é obrigatório** — sem ele o oráculo herda as preferências desta máquina.

## Os dois ficheiros

| ficheiro | rota | o que mede |
|---|---|---|
| `sdf_curvatura_por_escala.tsv.gz` | **SDF** (o nosso substrato) | `Mesh to SDF Grid` → `SDF Grid Mean(Width,Iterations)` / `Grid Laplacian`, nos dois braços (escala no CAMPO · escala na MEDIDA). `Grid Laplacian` = *«divergence of the gradient»*; num SDF `∇²f = 2H` |
| `malha_cavidade_por_escala.tsv.gz` | **malha** | `bpy.ops.paint.vertex_color_dirt` (*Dirty Vertex Colors*) — a tinta de cavidade assada em cores de vértice, CPU, sem GL |

Os dois carregam **cabeçalho com todas as grandezas que enquadram a corrida** (molde do arsenal §4).

## A régua, e de onde a barra sai

⭐ **A barra sai de um vale que inclui o lado APROVADO**: a verdade analítica desta peça é conhecida
em forma fechada — `H = 0` na face plana e `H = 1/(2r) = 2,5` no filete (um quarto de cilindro) —,
logo o oráculo é julgado contra a **geometria**, nunca contra a nossa saída.

⭐⭐ **O CONTROLO que torna esta fixtura honesta é a coluna `d0`**: o SDF amostrado nos pontos de
sonda. Ele tem de ser `≈ 0`, senão a sonda não está na superfície e tudo o resto mede outra coisa.
⛔ **Ele apanhou a primeira fixtura desta jornada a ser falsa:** a caixa filetada estava a ser gerada
pela soma de Minkowski parametrizada pela direcção da esfera, que **degenera em 8 esferas de canto**
(para toda direcção com as três componentes não-nulas o núcleo colapsa no mesmo vértice). A sonda
lia `H = 49,95 ≈ 1/voxel` numa face **plana** e a tabela parecia plausível. Medido depois da cura:
`d0 = 0,00026`, que é **1,3 % de um voxel**.

## Grandezas medidas (resumo — a tabela inteira está nos `.tsv.gz`)

| | |
|---|---|
| erro do estimador do oráculo no pico | `H = 2,47`–`2,64` contra `2,500` ⇒ **1,2 %–5,6 %** |
| `H` na face plana | `0,000` em todas as corridas |
| largura da rampa 10–90 % da borda | `0,0327` → `0,1963` em unidades de mundo = **10 % → 62 %** do arco do filete |
| o que a move | **`Width` × `Iterations`** (a escala), **não** a intensidade |
| amplitude ao longo dessa varredura | `H·R` de `4,57` a `4,28` — **praticamente parada** |
| escala no mínimo (rota de malha, `blur_iterations = 0`) | rampa **`0,0000`** — uma escada perfeita |
| preço do braço A | o filtro de campo **MOVE a superfície** (`dshift` até `0,0167` = `0,84` voxel); o braço B não move nada |

## O catálogo de controlos do alvo (introspecção RNA, `bl_rna.properties`)

⭐⭐ **SETE controlos, em duas famílias que ele ship JUNTAS** — e a partição diz porquê:
`cavity_type` oferece `SCREEN` (*«curvature-based … making fine details more visible»*), `WORLD`
(*«computed in world space, useful for larger-scale occlusion»*) e **`BOTH`**. *Nenhuma das duas
cobre a faixa sozinha.*

| controlo | tipo | fábrica | faixa (soft / hard) | papel |
|---|---|---:|---|---|
| `show_cavity` | bool | `false` | — | liga |
| `cavity_type` | enum | `SCREEN` | `WORLD`·`SCREEN`·`BOTH` | **que escala** |
| `curvature_ridge_factor` | float | `1,0` | `0..2` / `0..2` | intensidade da **ARESTA**, rota de ecrã |
| `curvature_valley_factor` | float | `1,0` | `0..2` / `0..2` | intensidade da **COVA**, rota de ecrã |
| `cavity_ridge_factor` | float | `1,0` | `0..2,5` / `0..250` | intensidade da **ARESTA**, rota de mundo |
| `cavity_valley_factor` | float | `1,0` | `0..2,5` / `0..250` | intensidade da **COVA**, rota de mundo |
| `matcap_ssao_distance` | float | `0,2` | `0..100` / `0..100000`, **subtype `DISTANCE`, unit `LENGTH`** | ⭐⭐⭐ **a ESCALA ESPACIAL** — *«distance of object that contribute to the cavity/edge effect»* |
| `matcap_ssao_attenuation` | float | `1,0` | `0..100` | queda com a distância |
| `matcap_ssao_samples` | int | `16` | `1..500` | amostras do integral de vizinhança |

⭐ E na rota de assar (`paint.vertex_color_dirt`): `blur_strength` (`0,01..1`), **`blur_iterations`
(`0..40` — a escala)**, `dirt_angle`/`clean_angle` (a janela tonal, em `ROTATION`), `dirt_only`
(*«don't calculate cleans for convex areas»* — o lado da cova sozinho) e `normalize`.

⚠️ **Medido:** `dirt_angle`/`clean_angle` **não mordem** nesta peça (as três janelas testadas dão a
mesma coluna, ao milésimo) — a janela tonal só separa quando o corpus tem ângulos que a alcançam.
⚠️ **`normalize = false` devolve `0,503` CHAPADO em toda a peça** ⇒ a resposta daquela rota é
**normalizada sobre o objecto**, não absoluta.

## ⛔ O que NÃO foi medido, e porquê

| | |
|---|---|
| a **curva de resposta** do *cavity* de viewport (como `ridge`/`valley` entram no sombreamento) | é um passe de **ecrã** e exige um contexto GL; a placa está em **exclusividade de outra linha** nesta jornada |
| a rota `WORLD` a valer (SSAO com `matcap_ssao_distance`) | idem — só o catálogo dela é dado aqui |
| o *anti-aliasing* da aresta viva | idem: é propriedade do passe de ecrã |
