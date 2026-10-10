# 14 — O Smooth Edges do Wet Paint: a borda suave por raio (W1g, 10/10)

> **O dono, no 3.º smoke (10/10):** *«Smooth edges ficou maravilhoso. Melhor até documentar a solução.»*
> Este doc é a solução e o caminho até ela — o que foi medido e recusado não se reconstrói (CLAUDE.md §5.0).
> Código: [`wetpaint/smooth_edges.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/smooth_edges.rs)
> (`Suave`), chamado do composite ([`wetpaint/composite.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/composite.rs)).

## §1 — O que o artista vê

Uma linha **Smooth Edges** `0..1` na secção Wet Paint (antes uma caixa). `0` de fábrica = o composite de
sempre, ao byte. A intensidade é a **largura da borda suave**: a ¼ some a escadinha do contorno escuro, a
½ a borda amacia, a 1 a borda fica bem suave. Só a orla muda; o miolo e o papel aberto ficam iguais; a
tinta total conserva-se. É aparência (como o Paper): nenhuma água nem pigmento do motor se move.

## §2 — A lei

Num pixel da região que o composite escreve:

1. **A mancha** `M = [pa ≥ SS0]` (`SS0 = 0,12`, o limiar da cobertura endurecida da Aquarela) — 0/1 por pixel.
2. **A faixa da orla** `B = tenda_r ∗ M` — a mancha desfocada por uma **tenda** de meia largura
   `r = slider × RAIO_MAX` (`RAIO_MAX = 6 px`), feita de duas caixas de meia largura fracionária `r/2`,
   separáveis (as pontas `⌊h⌋+1` pesam `h − ⌊h⌋`, para o slider ser contínuo). `B ∈ {0, 1}` longe da orla,
   `B ∈ (0, 1)` numa faixa de largura ~`2r` à volta dela.
3. **A tinta desfocada** `P̄ = tenda_r ∗ P` — a tinta PREMULTIPLICADA (`pc·pa`, e o próprio `pa`) desfocada
   pela MESMA tenda (2D, `(4⌊h⌋+5)²` toques).
4. **A mistura** `P' = P + (P̄ − P) · 4·B·(1 − B)` — peso inteiro na orla (`B = ½`), zero no miolo e no papel
   aberto. `peso < 10⁻³` ⇒ o pixel sai pelo caminho de antes, sem conta nenhuma.

`r = 0` ⇒ a tenda é a identidade ⇒ `B = M ∈ {0,1}` ⇒ peso 0 ⇒ **o desligado, ao byte** (e o composite nem
monta a `Suave`). O retângulo sujo cresce por `alcance(r) = ⌈r⌉ + 1` (um pixel a essa distância de uma
mudança também muda); a região lida, por `2 × alcance`.

**Porque premultiplicada:** a mistura é de COBERTURA — exacta também sobre uma camada transparente (misturar
a cor straight com a base escurecia a orla; a mesma lição do Smooth Edges da Aquarela).
**Porque `4·B·(1−B)`:** é o único peso simples que vale 1 exactamente na orla e 0 dos dois lados, contínuo.

## §3 — O caminho até lá (medido, por ordem)

| tentativa | a lei | o que se viu | veredicto |
|---|---|---|---|
| 1 (09/10) | o 3×3 do Watercolor (`aa_coverage`): anti-serrilhado de 1 px da silhueta | no máximo «não vi benefício» (dono) | ⛔ fraca demais |
| 2 (10/10) | o slider a misturar desligado ↔ 1.ª lei | só enfraquece o que já não se via | ⛔ |
| 3 (10/10) | o raio a alisar a FORMA da mancha (`B ≥ ½` com 1 px de AA; saliências e pontinhos < r somem, recortes enchem-se) | contorno −55 %, **e quase nada se vê no S**: a silhueta já é lisa | ⛔ |
| **4** | **a tinta desfocada na faixa da orla** (§2) | ¼ tira a escadinha, ½ amacia, 1 suave | ✅ «maravilhoso» |

⭐ **A lição:** o que o olho lê como «borda dura» no Wet Paint não é a SILHUETA da mancha — é o
**contorno escuro POR DENTRO dela** (o escurecimento da orla, `EdgeDarkening`), com escadinha de pixel. Uma
lei que só mexe na silhueta (AA, forma) não alcança o que se vê; a que funciona amacia a TINTA numa faixa à
volta da orla, contorno escuro incluído. Prancha das tentativas 3 e 4 no S do dono: o 05 §4b «W1g».

## §4 — Os portões

| portão | o que prova |
|---|---|
| `smooth_edges_no_zero_e_o_desligado_de_antes_ao_byte` | `0` = o composite sem Smooth Edges, FNV da tela na razão 1 e 2 (a identidade e o amostrador) |
| `smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes` | no máximo, todo byte que muda tem mancha E papel a menos de `alcance` px; voltar a 0 = 0 bytes de diferença |
| `mais_smooth_edges_e_uma_borda_mais_suave` | a **aspereza** (soma das diferenças de escuro entre vizinhos) cai a cada degrau: −27 % · −37 % · −45 % a ¼ · ½ · 1; a tinta total dentro de 2,3 %. Mutação «raio fixo» (o slider vira interruptor) ⇒ VERMELHO |
| `o_smooth_edges_e_um_slider_que_chega_a_ferramenta` (painel) | o clique real: viva sob o ponteiro, a edição vira `SetValue`, presa a `0..1` |
| `a_chave_muda_com_cada_parametro_e_o_traco_tambem` | o preview do painel vê o Smooth Edges |

## §5 — O custo

`medida_do_smooth_edges` (perfil smoke, no máximo, o mínimo de 7, no mesmo processo):

| versão | tela 4096² inteira (mexer no slider recompõe tudo) | retângulo de 300² (o quadro normal) |
|---|---|---|
| desligado | 9,5 ms | 0,13 ms |
| 1.ª versão da lei 4 (load 42) | 82 ms | 3,3 ms |
| + as passadas saltam as linhas sem tinta a `alcance` (lá o desfocado é EXACTAMENTE zero) | 45 ms | 3,0 ms |
| **+ o pigmento da região lido uma vez num plano** (só nas linhas a `2 × alcance` da tinta) | **31 ms** | **1,1 ms** |

O que sobra é a convolução 2D na faixa da orla (`17²` toques a `r = 6`). ⚠️ Se for preciso mais: a tinta
desfocada também é separável — uma passada horizontal nas linhas perto da tinta e a vertical só nos pixels
da faixa. Não medido; não prometa sem medir.

## §6 — Ligações

O id, a i18n, o painel (`paint_display_toggles`), o preset (não guarda — é aparência, como o Paper), o Reset
(volta a 0) e o preview (`CenaDoPreview` / `ChaveDoPreview`): 05 §4b «W1g». O Smooth Edges da **Aquarela**
é outro controlo e não mudou (a cerca).
