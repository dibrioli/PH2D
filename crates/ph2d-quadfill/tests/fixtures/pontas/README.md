# Fixtures — as PONTAS que o dono julgou, dos DOIS lados

⭐⭐⭐ **Estes cinco ficheiros são o discriminador que o handoff de 2026-09-01 (§0) exigiu:**
*«uma régua candidata tem de separar `Sculpt_Blender.obj` das nossas. Se não separa, não é a
régua.»* O portão que os lê é [`../../pontas_do_dono.rs`](../../pontas_do_dono.rs).

## Os ficheiros

| ficheiro | o que é | vértices · faces | veredito do dono |
|---|---|---|---|
| `sculpt_antes.obj.gz` | a escultura de entrada nº 1 (exportada pelo PH2D, 29/08 10:42) | 13 682 · 13 824 | — |
| `Sculpt_Blender.obj.gz` | a retopologia dela pelo **QRemeshify** (add-on do Blender, 29/08 10:47) | 8 293 · 8 291 quads | ✅ *«preserva as pontas»* |
| `_base_sculpt.obj.gz` | a escultura de entrada nº 2 (exportada pelo PH2D, 30/08 20:52) | 16 898 · 18 432 | — |
| `_remesh_sculpt.obj.gz` | a **nossa** saída sobre ela, `Detail 0,75` (31/08 18:45) | 5 447 · 5 445 quads | ⛔ *«amputa uma ponta»* |
| `sculpt_Depois.obj.gz` | a **nossa** saída sobre ela (01/09 16:56, antes da cura da cerca de viagem) | 18 324 · 18 322 quads | ⛔ *«não é bom»* (foto) |

⚠️ **A entrada da aprovada NÃO é a entrada das reprovadas** — são duas esculturas. A caixa de
`Sculpt_Blender.obj` é a de `sculpt_antes.obj` (`2,970 × 2,241 × 2,664`), não a de
`_base_sculpt.obj`. O handoff anterior emparelhava-as como se fossem a mesma peça; uma régua
que compare as duas saídas **em unidades de mundo** compara peças diferentes. ⇒ toda régua
deste portão é normalizada pela própria malha (unidade = aresta mediana da saída), e um
portão com **duas** peças prova a régua, não uma constante.

## O referencial (⛔ a armadilha que mordeu quatro vezes em dois dias)

O importador (`sculpt3d_import::place`) faz `Mesh::recenter()` — subtrai o **centro da caixa**
— e guarda escala e posição numa `Pose` que só desenha e exporta. Um `.obj` exportado traz a
pose **assada**: `p_exportado = p_cena · s + âncora`, com `s = IMPORT_SPAN / span = 2 / 3,424240
= 0,5840711` e `âncora = (2, 0, 0)` para estas duas saídas.

⇒ `_remesh_sculpt.obj.gz` e `sculpt_Depois.obj.gz` estão **já no referencial de
`_base_sculpt.obj`**: `p = (p_exportado − âncora) / s + centro_da_caixa(_base_sculpt)`, com
`centro = (−0,413688, −0,095009, −0,580249)`. O alinhamento foi verificado: `p50` da
distância saída→superfície da entrada `0,000 h`, `máx 0,34`–`0,38 h`.
`Sculpt_Blender.obj.gz` e `sculpt_antes.obj.gz` já partilham o referencial (o Blender exporta
no referencial em que importou): `p50 0,024 h`, `máx 0,32 h`.

## Proveniência e licença

| | |
|---|---|
| as duas esculturas | do **dono do produto** (Enio), feitas no Sculpt do PH2D |
| as duas saídas nossas | o botão `Quad Retopology` deste repo |
| `Sculpt_Blender.obj` | **saída** do QRemeshify sobre a escultura dele. O QRemeshify empacota o `quadwild-bimdf` (GPL); *a saída de um programa não é coberta pela licença do programa* — é o mesmo estatuto de `docs/3D/cleanroom/fixtures/*.mapa.gz` e de `ph2d-quadbench/ref/`. ⛔ Nenhum código, nome interno ou string do alvo entra aqui: `bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_quadwild.txt <estes ficheiros>` ⇒ `✓ sweep limpo (94 entradas)` em 2026-09-02 |
| compressão | `gzip -9` com cabeçalho determinista (`mtime 0`, sem nome) — o `sha256` do `.gz` é reprodutível |

## O que a régua lê neles (2026-09-02, unidade = aresta mediana da saída, `apices` com piso `0,25` e cone `≤ 1,0`)

O que o portão imprime (`cargo test -p ph2d-quadfill --test pontas_do_dono -- --nocapture`):

| par | espinhos | pior `gap` do ápice | pior grade a `3 h` | amputadas (`gap > 0,5`) | grade `> 1,0` | tempo |
|---|---|---|---|---|---|---|
| ✅ `sculpt_antes` → `Sculpt_Blender` | 5 | **`0,19`** | **`0,79`** | 0 | 0 | `0,12 s` |
| ⛔ `_base_sculpt` → `sculpt_Depois` | 5 | `4,08` (⚠️ a `9663`, comida por inteiro, lê o piso `3,0` = *«mais longe do que a régua olha»*) | `4,50` | 3 | 3 | `0,18 s` |
| ⛔ `_base_sculpt` → `_remesh_sculpt` | 6 | `3,17` | `1,51` | 1 | 2 | `0,50 s` |

*As duas barras (`TIP_GAP_MAX = 0,5` · `TIP_DENSITY_MAX = 1,0`) vivem nos vazios `0,31…1,02` e
`0,88…1,10` — o portão `as_barras_vivem_no_vazio_entre_o_aprovado_e_o_reprovado` exige a
margem. ⚠️ Os tempos são em DEBUG; a 1.ª versão da bola de caminho (Dijkstra por pilha) levava
`71 s` na terceira linha.*

## ⭐⭐⭐ `nossa_com_calota.obj.gz` — a NOSSA saída depois da cura (2026-09-03)

| | |
|---|---|
| o que é | a saída do botão `Quad Retopology` sobre `_base_sculpt`, na realização **do próprio dono** |
| como foi produzida | `PH2D_PIECE=_base_sculpt.obj PH2D_RECENTER=1 PH2D_DETAIL=1.0 PH2D_ADAPT=1.0` na sonda `the_artists_piece_through_the_button`, com a **calota** da fase zero (`ph2d_remesh_iso::Cap`, `TIP_CAP_STEP = 1,0`) e o **desembaraçador** de gravatas (`ph2d_quadfill::untangle_bowties`) — plano §105 |
| o que ela vale | `21 928` quads · `χ = 2` · `0` bordo · `0` não-manifold · **`0` de `5`** pontas amputadas (pior gap **`0,18`**) · grade no bico **`0,74`** |
| ⭐ a fenda SAIU | `21 914` faces (o disco da aba trocado por um leque): as faces do avesso vão de `6` num grupo para **`1` isolada** — um vinco real da escultura, que a lei deixa em paz de propósito (plano §107) |
| ⚠️ a entrada é RECENTRADA no teste | por [`ph2d_mesh::Mesh::recenter`], a **porta do importador** — sem isso as duas malhas vivem em espaços diferentes e a régua lê `5 de 5` com o gap saturado (*uma medição entre dois referenciais mede a translação*) |

⛔ **É o gate que impede a cura de se desfazer em silêncio:** a MESMA peça, no MESMO ponto do
slider, saía como `sculpt_Depois` — a ponta maior cortada sete células abaixo do bico.
