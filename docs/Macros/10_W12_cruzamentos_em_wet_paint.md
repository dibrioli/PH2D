# W12 — os cruzamentos em Wet Paint sem «nó» (a marca do artista como cobertura no meio molhado)

> Pedido: [PROMPT_W12](handoffs/PROMPT_W12_cruzamentos_em_wet_paint_2026-10-20.md). Antes: [plano W10](08_plano_W10_cobertura.md)
> (a marca é cobertura `c` + o píxel por baixo `B`; o Play mostra `atual + (1 − c)·(palco − B)`) — só no Digital.
> Handoff: [W12](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md).

## 1 — Medição antes de desenhar (§0.10: a família numa rodada)

O gate da W10 (`palcos_cobertura_tests::a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio`) gravado e
pintado em cada meio, desvio máximo em níveis contra a régua (a mesma vertical num tool novo sobre a reprodução
daquele instante), a `0,1 s` / `0,6 s` / fim:

| Pincel | Digital | Wet Paint | Watercolor | Impasto |
|---|---|---|---|---|
| macio | 2 / 2 / 0 | 113 / 113 / 113 | 255 / 255 / 145 | 217 / 217 / 138 |
| duro esfera | 1 / 1 / 0 | 109 / 106 / 99 | 255 / 255 / 145 | 217 / 217 / 138 |
| duro disco | 0 / 0 / 0 | 71 / 102 / 71 | 255 / 255 / 145 | 132 / 132 / 41 |
| grão (segue o pincel) | 3 / 3 / 0 | 116 / 116 / 116 | 255 / 255 / 145 | 249 / 249 / 133 |
| grão fixo à tela | 5 / 3 / 0 | 81 / 113 / 80 | 255 / 255 / 145 | 249 / 249 / 183 |
| força ½ | 3 / 3 / 0 | 113 / 113 / 113 | 255 / 255 / 145 | 212 / 212 / 62 |
| contínuo esparso | 1 / 1 / 0 | 116 / 116 / 110 | 254 / 254 / 145 | 255 / 255 / 158 |

Nos três meios o desvio fica alto **até no fim do clipe** (onde o Play já é a imagem atual): a régua «a vertical
sobre a imagem, num tool novo» não descreve o que o tool da gravação faz. Uma sonda de coluna (a vertical no tool
da gravação contra a mesma num tool novo sobre a imagem gravada, linha a linha; os números ficam aqui, a sonda saiu)
e as de `tool-painter` `sonda_w12_tests` (ignoradas, ficam) mostraram porquê:

- **Wet Paint — FUSÃO.** Depois do Parar a sessão molhada da gravação continua viva (`macro_para` não a encerra) e a
  vertical entra nela: o pigmento dela e o das gravadas viram um só pool por célula (`sett`/`susp`, as cores
  misturadas por massa, `ph2d-wet-paint/src/render.rs`), a água arrasta tinta gravada (bordas mais escuras nas
  pontas das horizontais, cruzamento mais claro que «por cima»). Não há `c` que separe o que é do artista.
- **Watercolor — a aguada funde-se e não é «por cima».** Na mesma lavagem (`wet_session_base` de antes das gravadas)
  o cruzamento passa a ser o valor da vertical sozinha (atual 135 onde a gravada era 3; num tool novo, 9), e a lei é
  Beer–Lambert em luz linear por canal (`watercolor_render.rs`): em bytes sRGB nenhum `c` escalar a exprime.
- **Impasto — a luz.** A cor compõe-se por cima, mas a imagem é `lit(over(…))` com o relevo de TODAS as pinceladas
  (Lambert + Blinn-Phong, `impasto_light.rs`): até 138 níveis entre o mesmo traço com e sem o relevo das gravadas.

### Os candidatos a cobertura com a fusão (Wet Paint, pincel macio, coluna da vertical, palco branco)

| `c` | pior desvio contra a vertical sozinha |
|---|---|
| cheia (W10) — o «nó» escuro | 112 |
| a opacidade da vertical sozinha (a sombra ideal, medida num tool à parte) | 53 |
| a diferença de massa por célula desde o 1.º toque, desenhada pela porta do render | 63 |
| o ajuste a dois pontos com essa diferença | 64 |

A fusão deixa um resíduo de dezenas de níveis (mancha clara no cruzamento, borda escura nas pontas) com qualquer
`c`; a única forma exacta mantendo-a seria seguir a ORIGEM da massa dentro do motor da água (um segundo pool de
pigmento advectado com o primeiro) — no motor que a `line/PainterWatercolor` está a mudar (21 commits por integrar
em `ph2d-wet-paint`).

### Assar no 1.º toque (medido antes de propor)

A sessão da gravação encerrada (o fim da sessão É o bake) antes da vertical: a dela nasce sobre a imagem, e em Mix
`atual = P + (1 − pa)·B` vale a **1,4 níveis**; o Play antes de as gravadas nascerem fica a **1,7** da régua. O
documento muda até **53** níveis no cruzamento (por cima em vez de fundido).

## 2 — Decisão do dono (20/10)

Perguntado em linguagem de produto (fundir e ficar com um resíduo, fundir e reformar o motor, ou por cima): **«Por
cima»** — a tinta pintada depois de Parar fica POR CIMA da gravada, como sobre tinta seca; a água da gravação que
ainda escorra pára no 1.º toque do artista. Memória: `project_wet_paint_after_stop_paints_over_the_recording_2026_10_20`.

## 3 — A lei

- **A sessão molhada tem dono.** `WetSession::marca` = o id que `MarcaViva::nasce_a_molhada` lhe deu quando nasceu
  com a marca a escrever (sem marca, ou suspensa: `None`). `stamp_dabs` → `wet_da_marca`: com a marca a escrever e os
  dabs a irem para a água, uma sessão que não é a viva do artista **assa** (`wetpaint_end_session`) e a dele nasce
  ali, ANTES de qualquer marca. A borracha molhada fica de fora (apaga o fluido que houver; lei da W9).
- **A cobertura é o `pa` do composite**, por píxel, depois do Smooth Edges e × o `keep` da selecção/proteção:
  `composite.rs` devolve-a num plano `u16` da caixa que escreveu, e `MarcaViva::agua_cobre` →
  `MarcaDoArtista::recompoe_da_agua` recompõe cada texel a partir da marca do NASCIMENTO da sessão:
  `c = 1 − (1 − c₀)(1 − pa)`, `B = B₀` onde `c₀ > 0`, senão a tela congelada da sessão (sobre o papel). A água anda
  depois do passo: cada composite volta a escrever o que a tinta tem agora; onde ela sai, `c` volta a `c₀`.
- **Exacta** em Mix, camada única, sem vidro de aguada, sem alfa preso, sem papel que atravesse vidro; fora disso o
  texel pintado fica cheio (o `pa > 0` marca `CHEIA`, a lei da W9 sobre a tinta).
- **Os discos provisórios, o miolo ≥ ½ e a cobertura do Solid na água** deixam de marcar na água do artista (o
  composite cobre-os). A água da gravação continua a não marcar (W9 A1).
- **O histórico.** O passo vai da marca em que começou (`comeca_o_passo`) à do registo; a água que corre depois estica
  o delta do passo quando o histórico a absorve (`undo_absorb` → `estende_pela_agua`, a mesma regra do canvas: o
  escorrido é do passo que o causou). O que o passo muda FORA do composite compõe-se também no nascimento.
- **Watercolor, Impasto e os outros modos de mistura ficam com `c` cheia** (§1: a aguada funde-se e é linear-light;
  o Impasto acende o relevo depois de compor). Memória `project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07`.

## 4 — Medições depois

Gate dos cruzamentos (`MEIOS = Digital, Wet Paint`; Wet também 20 quadros depois do Up, com a água a correr):

| Pincel | Wet Paint, água a correr | Wet Paint, assentada | Digital |
|---|---|---|---|
| todos os 7 | 1–2 | 1–2 | 0–5 (igual à W10) |

Custo (`load` ~35, máquina partilhada; mesmo processo, blocos intercalados):

| Sonda | sem a marca | com a marca |
|---|---|---|
| `tool-painter sonda_w12_o_custo_da_agua_marcada`, 1024², `on_tick` com a água no worker | mediana 2,57 · média 2,54 · máx 10,3 ms | mediana 2,36 · média 2,60 · máx 9,6 ms |
| `app-painter sonda_o_custo_da_marca` (A3), `liga+sobe` | água 0,032 ms | água + traço Wet marcado 0,760 ms (W10 0,747) · Play Wet mediana 0,95 / máx 56,2 (1 subida inteira) |

## 5 — ⛔ Recusas MEDIDAS

| Proposta | Medida | Porque não |
|---|---|---|
| `c` = diferença de massa por célula (sem mexer no motor) | 63 níveis de resíduo | a água move tinta gravada; a massa nova não é só do artista |
| `c` = opacidade da vertical sozinha (uma sombra) | 53 | a fusão: `atual` não contém «a vertical sobre `B`» |
| `c` a partir de `atual` e `B` (o «keying» pela cor do pincel) | — | indeterminado onde a cor do pincel = `B` (preto sobre preto: o caso do dono) |
| a vertical numa 2.ª camada do motor (`Engine::add_layer`) | — | só a camada ativa simula: a água da gravação pararia igual, com mais custo |
| fundir e seguir a origem da massa no motor | — | o dono escolheu «por cima»; colide com a `line/PainterWatercolor` |
