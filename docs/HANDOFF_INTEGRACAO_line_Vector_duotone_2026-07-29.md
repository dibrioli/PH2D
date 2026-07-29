# Handoff de integração — `line/Vector`: DUOTONE + LUMA TO ALPHA (plano 24 W9)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `78d770370`
**Estado:** fechada, **pendente de smoke**

⚠️ **Esta é a QUINTA wave da mesma abertura de linha,** e as cinco **compartilham o bump de schema**
(`PROJECT_SCHEMA` 38). Integre-as juntas — o `main` nunca viu nenhuma:

1. [`…_fx_blend_…`](HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md) — a lei de mistura
2. [`…_turbulence_…`](HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md) — a turbulência
3. [`…_morphology_…`](HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md) — Grow / Shrink
4. [`…_colour_adjust_…`](HANDOFF_INTEGRACAO_line_Vector_colour_adjust_2026-07-28.md) — Color Adjust
5. **esta** — Duotone + Luma to Alpha

⚠️ **O handoff da W8 diz que *"com esta, o catálogo do plano 24 FECHA"*.** Era verdade para as
quatro famílias que o `apply_op` da W2 nomeou; estes dois vieram **por pedido do Enio**, fora
daquela lista. A frase fica corrigida aqui em vez de apagada lá.

---

## O que o artista ganha

Dois degraus novos, os dois **pontuais** (um dispatch, margem zero, cobertura preservada — salvo o
segundo, que é o único pontual que a MOVE de propósito):

| tipo | controles | o que faz |
|---|---|---|
| **Duotone** | **Shadows** + **Highlights** (duas swatches) · Blend · Opacity | a luminância de cada texel escolhe um ponto entre as duas cores — a *Gradient Map* de dois stops |
| **Luma to Alpha** | **Opacity, e nada mais** | o brilho vira cobertura: claro fica opaco, escuro fica transparente |

O Duotone **preserva a modelagem** (sombreado, volume, degradê) e troca só a paleta — é isso que o
separa do Color Overlay, que achata. Lado a lado, medido: excursão de **188 níveis** no verde contra
**0**.

---

## A PRIORIDADE do pedido responde antes do desenho

O Enio pediu os dois com o eixo nomeado: *"quero boa qualidade, mas quero principalmente
**performance em tempo real em runtime para games**"*. Os dois caem na pista PONTUAL, e o número é
medido (RTX, `the_pointwise_op_costs_much_less_than_a_blur`):

| | custo |
|---|---|
| a moldura (pilha vazia) | 0,058 ms |
| **6 degraus pontuais** | **+0,022 ms** — ≈ **0,004 ms cada** |
| 6 borrões | +0,575 ms — ≈ 0,096 ms cada |

**0,02 % de um quadro de 60 fps por degrau.** A prioridade não teve de ser negociada contra a
qualidade: a operação que o pedido descreve já é a barata da pilha.

---

## As três decisões que carregam a wave

### 1. A RÉGUA é o `L` do OKLab, não o `lum()` das leis de mistura

O repo tem **duas** funções que parecem responder a *"quão claro é este texel?"*, e elas não são
intercambiáveis: o `lum()` do `blend_modes.wgsl` é a **luminosidade do W3C**, definida para os modos
`Color`/`Luminosity`, sobre luz LINEAR; o `L` do OKLab é a **lightness perceptual**, que é
literalmente a definição da pergunta que a rampa faz.

**Medido** (`measure_the_two_candidate_rulers_for_the_ramp`), cinza sRGB 128: `lum` = **0,216** ·
`L` = **0,600**. Com o `lum` o meio-tom cairia a **um quinto** do caminho e a arte inteira se
empilharia na ponta escura. Os coeficientes do `L` **somam 1** ⇒ as duas pontas são EXATAS.

### 2. A divergência do SVG no Luma to Alpha é DELIBERADA, e é o que faz o efeito compor

A matriz do `feColorMatrix` escreve `A' = luma(cor RETA)` **ignorando o alfa**. Num pipeline
premultiplicado isso endurece a orla anti-aliased — **medido** com a lei literal instalada: um texel
com **4/255** de cobertura salta para **180/255**.

A nossa lei **escala** (`A' = A · luma`) e **preserva a cor**. O argumento decisivo é de composição:
**encadear recupera o SVG, e o contrário é impossível** — `Luma to Alpha` → `Color Adjust
(Brightness −1)` dá o matte preto exacto da matriz, e nenhuma ordem devolve a cor já apagada.
*A lei que guarda informação é a que compõe.* Há gate para as duas metades.

### 3. A segunda swatch é a PRIMEIRA outra vez

`filter_color_swatch` passou a receber `(id, cor, rótulo, y)` em vez de derivar tudo da linha — o
que faz a ponta clara reusar a MESMA função de pintura, o MESMO registro de picker e a MESMA
estética. Um caminho de desenho paralelo divergiria na primeira mudança visual.

Quem responde *"qual ponta o artista abriu?"* é **`fx_live::colour_target(id) -> (linha,
é_a_segunda)`**, e a resposta vem do **id do alvo**, nunca do `kind` do degrau.

---

## O que a wave encontrou de errado no que já estava lá

1. ⚠️ **Uma família de acessores `pub` sem UM chamador.** `reads_noise` / `reads_grow` /
   `reads_adjust` (das W6b/W7/W8), cada um com um doc-comment a afirmar *"porta única com dois
   consumidores: o painel a consulta para OFERECER, o produtor da GPU para HONRAR"*. **A frase era
   falsa nos dois lados** — o painel não alcança o `ph2d-ecs` (lê o `FilterKindView` publicado) e o
   produtor copia os campos incondicionalmente; quem HONRA é o ramo por `kind` no shader. Os três
   foram **removidos** (o quarto, o meu, não chegou a existir), com o porquê gravado no `spec()`.
   **Achado por uma mutação que SOBREVIVEU.**
2. ⚠️ **O `node_id_collisions` estava cego a metade da seção** — ele enumera os ids de linha à mão, e
   as três waves anteriores acrescentaram **catorze** sem entrar na lista. Agora são **32 por
   linha** + os modos + as opções de mistura.
3. **Um doc-comment órfão a mentir um número:** *"64 bytes de propósito"*, pendurado num `use`, com o
   struct em 112. Foi para o campo de padding que ele descreve, com o número certo.
4. ⚠️ **A varredura de tipos não podia conter o fenômeno.** A fixture dela é uma CHAPA branca, e
   sobre branco puro o Luma to Alpha é a **identidade** — ela reportaria *"não desenha nada"* sobre
   um produto correto, que é a própria falha que ela existe para produzir, com o sinal trocado. A
   varredura ganhou fixture própria (um DEGRADÊ); as outras ficaram na chapa, porque os comentários
   delas estão calibrados nela.

---

## Superfície tocada

| | |
|---|---|
| **`PROJECT_SCHEMA`** | **fica em 38** — a política que o próprio 38 declara (*uma linha, um bump*): ele já carrega turbulência + morfologia + ajuste, e um save v37 já é recusado. ⚠️ **O valor se CONTA a partir do `main` do dia** — se a integração achar outro dono para o 38, este é o que anda |
| **Contrato congelado** | **intacto** — gates verdes (`architecture_contract_surface` 3 · `_tool_` 4 · `_vector_` 11) |
| **ADR** | nenhum |
| **`Cargo.toml`** | **zero** — nenhuma dep nova, nenhuma crate nova |
| **`ph2d-ecs`** | `FxOp` +`color_b`, +`DUOTONE`(12)/+`LUMA_TO_ALPHA`(13), `KINDS` 12→**14**; `FxKindSpec` +`color_b_label`; −4 acessores mortos |
| **`ph2d-editor-core`** | `MAX_FILTER_KINDS` 12→**14**; +`filter_color_b_id` |
| **`ph2d-render`** | `FxOpGpu` +`tint_b`; `Globals` 112→**128 bytes** (pin atualizado); 2 ramos no `cs_op_point`; 2 consts de tipo |
| **`ph2d-panel-vector`** | +`color_b_label`/`color_b` nas views; a 2ª swatch; `filter_color_swatch` recebe o id |
| **`shells/desktop`** | `FilterHit::ColorB`; `colour_target`/`colour_bytes`; o readback do picker ramifica; cena de smoke **38** |
| **LOC** | `fx_live.rs` bateu 607 > 600 ⇒ split por responsabilidade em **`fx_live_hit.rs`** (*o que um ID quer dizer* × *o que uma pilha É e o que a shell FAZ com ela*), com re-export para não mover caminho de chamador |

---

## Gates

**9 no arquivo novo** `crates/ph2d-render/tests/fx_stack_duotone_gpu.rs` (⚠️ `#[ignore]`, precisam de
adapter — rodados na RTX: **9/9**), mais a varredura de tipos (13/13) e os gates de seam/decode.

O que carrega a wave é o **oráculo em CPU independente** sobre a conversão OKLab do `ph2d-color` —
escrita por outra wave, para outro consumidor: **pior delta 1 NÍVEL DE BYTE**.

**10 mutações, 9 sangram.** A sobrevivente produziu a correção do item 1 acima.

| # | mutação | resultado |
|---|---|---|
| M1 | a régua vira o `lum` do W3C | RED (`cinza 4: [27,32,89] vs [96,84,99]`) |
| M2 | as duas pontas trocadas | RED (3 gates) |
| M3 | a força por-ponta vira `1.0` | RED |
| M4 | o `rgb` não acompanha o alfa | RED (a cor reta muda: 28 contra 4) |
| M5 | a lei LITERAL do SVG | RED (2 gates: orla 4→**180**) |
| M6 | `reads_color_b` → `false` | **SOBREVIVEU** ⇒ os 4 acessores eram mortos |
| M7 | o readback ignora QUAL swatch | RED |
| M8 | a 2ª swatch nunca é pintada | RED (2 gates) |
| M9 | o `tint_b` não chega ao device | RED (2 gates) |
| M10 | a 2ª swatch não é alvo de picker | RED |

---

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=38 cargo run -p ph2d-host-desktop --release
```

Quatro pares. ⚠️ **Toda estrela leva um Bevel antes** — uma chapa de cor sólida não tem variação de
brilho, e sem variação estas duas leis não têm o que ler.

**FILEIRA 1:** (1) controle, só o Bevel · (2) + Duotone frio→quente: **a mesma modelagem, outra
paleta** · (3) outra paleta · (4) a mesma rampa invertida.
**FILEIRA 2:** (5) Color Overlay: a estrela vira **chapa** · (6) Duotone com a mesma cor: o volume
**fica** — a resposta lado a lado à objeção óbvia · (7) controle com halo azul · (8) + Luma to
Alpha: o escuro fica transparente e o **halo aparece por trás**.

**No painel (é o que fecha o smoke):** selecione a estrela 2, abra FILTERS, e o card *Duotone* tem
**duas** swatches — clique a de cima e escolha uma cor: ela tem de pousar na ponta ESCURA; depois a
de baixo, na CLARA. E o card *Luma to Alpha* da estrela 8 tem **Opacity e mais nada**, de propósito.

---

## Aberto, nomeado

- **Uma rampa de N stops** (o Gradient Map completo) — o pedido foi de **duas pontas**, que é o caso
  que o artista usa; N stops é outro widget (um editor de gradiente por degrau), não um terceiro
  campo.
- **O `Amount to Tint` do AE** não existe como knob próprio: a Opacity do degrau **é** ele, e um
  slider a mais seria a segunda porta para o mesmo número.
