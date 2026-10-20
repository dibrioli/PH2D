# 16 — Bloco 4 da W23: velocidade e memória (B6 · B7 · B10 · B11) — 2026-10-20

> Leitor: a próxima LLM. Lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](handoffs/HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) (B6, B7,
> B10, B11). Ordem do bloco: **medir ANTES de construir**. Régua única, um processo, variantes intercaladas, vale o MÍNIMO
> ([MEDIR_VELOCIDADE.md](../DevOps/MEDIR_VELOCIDADE.md)): `crates/ph2d-board-render/tests/it/measure_bloco4.rs`.
> Saída gravada: [ferramentas/bloco4/saida_2026-10-20.txt](ferramentas/bloco4/saida_2026-10-20.txt).
>
> ```text
> bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --profile smoke --test it measure_bloco4 -- --ignored --nocapture
> ```
>
> `PH2D_BLOCO4_PNG=<pasta>` grava as imagens da visibilidade (PPM). `PH2D_BLOCO4_SO_OLHAR=1` corre só a visibilidade.
> ⚠️ Medido com a máquina carregada por outras linhas (`loadavg` 23–42): os números absolutos estão inflados ~2–4× em relação
> aos da W5 (`load 4`); a comparação lado a lado é a que vale.

| item | veredito | o quê |
|---|---|---|
| B6 | ✅ FEITO | abaixo de **11 px** no ecrã uma forma em rascunho desenha-se FINAL com o contorno **×2,5** |
| B7 | ✅ FEITO | a ordem de desenho ordena o PREFIXO inteiro da chave de z (8 bytes) e só no empate vai ao texto |
| B10 | ✅ já feito na W16 | a lista estava desactualizada — o export SVG embebe os bytes sem descodificar |
| B11 | ✅ já feito na W16 | idem — a leitura do Flip é por ladrilhos; a memória não cresce com as camadas |

## 1. B6 — o rascunho de quadro grande

**A pergunta.** A W5 mediu 10 mil formas em rascunho a ~8 px: 11,1 ms de CPU contra 6,0 no final. O candidato era «abaixo de
N px o traço à mão é invisível ⇒ desenhar o final». A régua mede custo (CPU e placa) E se a troca SE VÊ.

**A régua da visibilidade.** 400 formas da mistura `Sketch` (rectângulo, elipse, losango; cantos redondos alternados; uma em
cinco rodada), renderizadas pela placa a 640×480 a cada tamanho. Métrica: |Δ luma| médio entre o rascunho e a candidata.
**Controlo:** a mesma imagem final deslocada MEIO píxel — uma diferença que ninguém distingue. Razão ≤ 1 ⇒ não se vê.

**O 1.º achado (as imagens, não o número):** o que distingue o rascunho de longe NÃO é o tremido, é o CONTORNO. O rough.js
passa a linha duas vezes; a 1 unidade de espessura, de longe o final fica com meio píxel de contorno e quase some, e o
rascunho mostra um contorno claro e forte. Trocar para o final simples (o candidato da W5) muda o aspecto do quadro inteiro.

Visibilidade (controlo = o final simples ½ px, o mais exigente; 1.ª rodada):

| forma no ecrã | final simples | final contorno ×1,5 | ×2 | ×2,5 | ×3 |
|---|---|---|---|---|---|
| 5 px | 1,53 | 1,20 | 0,90 | **0,62** | 0,45 |
| 6 px | 1,87 | 1,48 | 1,02 | **0,70** | 0,54 |
| 8 px | 1,97 | 1,53 | 1,16 | **0,90** | 0,78 |
| 10 px | 2,19 | 1,67 | 1,21 | **0,92** | 0,93 |
| 12 px | 2,30 | 1,74 | 1,29 | 1,08 | 1,11 |
| 16 px | 2,19 | 1,60 | 1,18 | 1,08 | 1,24 |
| 24 px | 1,90 | 1,45 | 1,06 | 1,36 | 1,87 |
| 32 px | 1,94 | 1,50 | 1,52 | 1,95 | 2,56 |

⇒ **×2,5 fica abaixo do controlo de 5 a 10 px e acima a 12** ⇒ `SKETCH_FINAL_BELOW_PX = 11`, `SKETCH_FAR_STROKE = 2,5`. Com o
controlo do próprio caminho de longe (2.ª rodada, mais folgado) a razão fica em 0,44–0,62 até 11 px e só passa de 1 aos 24 px —
o limiar segue o controlo mais exigente. Fotos ampliadas 4× conferidas: a 6 e 10 px o de longe lê-se como o rascunho; a 12 px
o tremido do rascunho já aparece.

Custo (10 mil formas, 1920×1080, mínimo de 7×20; «de longe» = o caminho novo em todos os tamanhos, 2.ª rodada, `load 30`):

| forma no ecrã | CPU à mão | CPU de longe | placa à mão | placa de longe |
|---|---|---|---|---|
| 5 px | 23,95 | 15,83 | 6,40 | 3,11 |
| 8 px | 23,96 | **16,10** (−33 %) | 6,61 | **3,01** (−55 %) |
| 10 px | 20,78 | 13,62 | 5,98 | 2,48 |
| 11 px | 19,35 | 12,54 | 4,89 | 2,39 |
| 12 px (à mão no produto) | 17,25 | 10,76 | 5,22 | 2,17 |

**A lei no produto** (`ph2d-board-render/src/lib.rs::paint_shape`): `far = sketch ∧ preenchimento sólido ∧ lado maior × zoom <
lod(Detail::sketch_final_below_px)`; `far` ⇒ o caminho final com o contorno `stroke_width × 2,5` (e a folga da camada de
opacidade com ele). O export vectorial (`lod` = 0) continua sempre à mão. `Detail::sketch_final_below_px = 0` devolve o desenho
de antes (é o interruptor das réguas). Gate: `sketch_tests::a_far_sketched_shape_is_drawn_final_and_a_near_one_by_hand` (nenhum
traço à mão gerado, contorno ×2,5; acima do limiar à mão; limiar a zero = antes).

## 2. B7 — o chão das cenas de 100 mil

**A pergunta.** Cada imagem ordena TODOS os vivos por `(z, id)`. A chave de z é um `FracKey(String)`: cada comparação vai a
dois textos no heap. O índice de z mantido pelas ops já está recusado (plano §6: 7,8 ms contra 6,1); cortar pela vista antes de
ordenar também (W5: +1,4–3 ms).

Ordem de desenho (ms por imagem, mínimo de 7×20, mediana entre parênteses; 2.ª rodada, `load 30`; as variantes conferidas
IGUAIS à de antes antes de cronometradas):

| caso | o de antes (estável ×2) | instável + partição | **o produto (prefixo u64 + partição)** | chão: passagem + `collect` |
|---|---|---|---|---|
| 10 mil, z na ordem dos ids | 0,207 (0,272) | 0,205 (0,243) | 0,253 (0,278) | 0,154 |
| 100 mil, z na ordem dos ids | 8,01 (9,66) | 8,68 (9,65) | **7,41** (8,29) | 2,31 |
| 100 mil, z BARALHADO | 21,68 (22,97) | 20,22 (23,39) | **9,29** (10,27) | 3,51 |

⇒ com o z baralhado (o quadro depois de muitos «para a frente/trás», colagens, colaboração) **2,3× mais depressa**; na ordem
natural, igual (a 10 mil a diferença de 0,05 ms está dentro da mediana). **A lei** (`ph2d-board-model`): `FracKey::prefix()` =
os 8 primeiros bytes como `u64` big-endian, zero no resto — a base 62 não tem o byte 0, logo `a.prefix() < b.prefix() ⇒ a < b`;
`live_in_z_order` ordena `(prefixo, índice)` com `sort_unstable` e desempata por `(z, id)`; `live_in_paint_order` põe os vídeos
por cima com uma PARTIÇÃO (era uma 2.ª ordenação). Gates: `frac_tests::the_prefix_orders_like_the_key` (3 041 chaves de inserções reais,
caudas de colaboração, chaves além dos 8 bytes) e `board_tests::the_paint_order_is_the_key_order_on_a_shuffled_board` (z
baralhado com empates, chaves longas, vídeos).

**O chão que fica:** a passagem pelo `BTreeMap` + `collect` (2,3–3,5 ms a 100 mil sob carga; ~0,5 ms na W5 a `load 4`). Tirá-lo
é mudar o contentor dos elementos — fora deste bloco, e o índice mantido já perdeu uma vez.

## 3. B10 e B11 — já feitos na W16 (a lista-mãe herdou-os de um handoff anterior à W16)

- **B10** — [integração W16 §1.3](handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md): `ExportTarget::Svg` grava
  `DrawOp::Embed` com os bytes originais (`ph2d-board-render/src/image.rs`, `embed` → `level = None`); nenhuma imagem se
  descodifica. Medido (`measure_svg_export_images`): fotografia 4000×3000 + SVG de 2 000 círculos **161,7 → 1,4 ms**, o SVG
  escrito igual byte a byte. Conferido no código a 20/10.
- **B11** — [integração W16 §1.5](handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md): a leitura do Flip vai por
  ladrilhos (`ph2d-app-flip/src/pass_raster.rs::tile_for`). [saida_2026-10-11_depois_ladrilhos.txt](ferramentas/memoria_flip/saida_2026-10-11_depois_ladrilhos.txt),
  linhas `CF#0` (o caminho do produto): a 8192², pico RSS **+410 / +418 / +411 MiB com 1 / 4 / 8 camadas** — não cresce com as
  camadas; alocador **+0 MiB** depois. Os 256 MiB que restam são a própria imagem 8192² RGBA que vai para o quadro. ⚠️ As linhas
  `PICO_DECOMPOSTO` do mesmo ficheiro são a variante ANTIGA medida ao lado (2 982 MiB a 4 camadas) — não as cite como o produto.

## 4. Prova de mutação (20/10)

| mutação | gate | resultado |
|---|---|---|
| M1 `hand = sketch ∨ ¬sólido` (sem o caminho de longe) | B6 | sangrou |
| M2 `SKETCH_FAR_STROKE = 1,0` | B6 | sobreviveu à 1.ª (o gate lia a própria constante) → o gate fixa o **2,5 medido** → sangra |
| M3 o contorno final com `stroke_width` (sem o ×2,5) | B6 | sangrou |
| M4 `sketch_final_below_px` de nascença 0 | B6 | sangrou |
| M5 `from_le_bytes` no prefixo | prefixo · ordem | sangrou nos dois |
| M6 sem o desempate por `id` | ordem | sangrou |
| M7 só o `id` no empate do prefixo | ordem | sobreviveu à 1.ª (nenhum par partilhava os 8 bytes) → o gate ganhou 20 chaves de cauda longa com o mesmo prefixo e ids ao contrário → sangra |
| M8 a partição dos vídeos ao contrário | ordem | sangrou |
| M9 prefixo de 4 bytes em vez de 8 | — | sobrevive: EQUIVALENTE (o empate cai no texto) |

## ⛔ Recusas MEDIDAS

| proposta | porquê não |
|---|---|
| desenhar o FINAL simples abaixo de N px (o candidato da W5) | perde o contorno: razão 1,5–2,3 contra o controlo ½ px em TODOS os tamanhos de 5 a 32 px — vê-se |
| limiar acima de 11 px | ×2,5 passa o controlo exigente a 12 px (1,08) e o tremido aparece nas fotos |
| levar o caminho de longe às formas com preenchimento às riscas (`fill_style` ≠ sólido) | NÃO medido (a régua só tem o sólido): o final pinta-as sólidas, o que muda o tom — ficam à mão até uma régua as medir |
| `sort_unstable` sobre as referências (sem prefixo) | igual ao de antes (20,2 contra 21,7 ms baralhado): o custo é ir ao texto, não a estabilidade |
