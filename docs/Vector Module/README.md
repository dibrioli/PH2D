---
name: vector-module
description: Roteador do módulo Vector do PH2D — o motor `ph2d-vec-*` (GPU-first, editor-first, referenciado no runtime MIT do Rive, ADR-0108). O que o módulo é hoje, o que está ABERTO, e onde ler.
status: ROTEADOR (não é spec). A spec anterior (pré-ADR-0108) está arquivada — ver §5.
data: 2026-08-18
---

# Vector Module — roteador

> **Este arquivo é um PONTEIRO, não uma especificação.** Ele responde três coisas: *o que o módulo
> É hoje*, *o que está ABERTO*, e *onde ler*. Detalhe de mecanismo vive no ADR e no handoff da wave
> que o construiu — nunca aqui.

## 1. O que o módulo é HOJE

Um motor vetorial **GPU-first, editor-first**, **referenciado** no runtime MIT do Rive: é um
reimplemento nativo sobre `kurbo`/`vello`, e ⛔ **não vendoriza `rive-rs`**
([ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)).
As crates são `ph2d-vec-*`.

As leis que atravessam o módulo — quebre uma e algo cai longe do sítio da edição:

- **Todo path é uma ENTIDADE ECS, e a pose mora no `Transform`**
  ([ADR-0110](../architecture/decisions/) / [0111](../architecture/decisions/)).
- ⚠️ **Regra-mãe do pen:** *o que se **vê**, se **aponta** e se **encaixa** é MUNDO; o que o
  documento guarda é LOCAL.* Quase todo bug de "a alça escorrega" é esta fronteira atravessada
  no sítio errado.
- ⚠️ **A lei do auto layout** ([ADR-0153](../architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)):
  *o passe publica **onde** as coisas ficam; ele não escreve **onde** elas estão.* Nada no auto
  layout toca `Transform` — senão cada quadro de um resize vira um passo de undo.
- **fonte ≠ cozido**: a costura que destravou os **Live Path Effects**
  ([ADR-0132](../architecture/decisions/)); o documento guarda a AUTORIA, o consumidor cozinha.

O que existe: **13 modos** (Select · Node · Pen · Build · Width · Tesoura · …,
[ADR-0112](../architecture/decisions/)) · **Live Corners** ([ADR-0121](../architecture/decisions/)) ·
**blend** ([ADR-0128](../architecture/decisions/)) · **largura viva**
([ADR-0148](../architecture/decisions/)) · **auto layout** via `taffy` atrás de uma crate-folha
([ADR-0153](../architecture/decisions/)) · guias e régua · **simetria como modo** · **booleana viva
com um verbo POR FORMA** ([27](27_um_verbo_por_forma.md)) ·
moldura · **tokens no documento** · **estados de UI + Smart Animate** · a **árvore autorada como
painel vivo** (o app escreve o código do painel) · e a pilha de **FX raster**
([`24_plano_fx_raster.md`](24_plano_fx_raster.md)).

## 2. O que está ABERTO

> A lista canônica e datada é o **[`CLAUDE.md` §5](../../CLAUDE.md)**; o que segue é o mesmo
> conteúdo com o ponteiro para onde ler cada um.

- ⏸️ **O `n`/folga do *tether* e o `DRAG_RATE_X = 50`** são números de **FEEL sem medição atrás**
  — e a lei irmã do repo diz `rate = step`, **50× menos**, em **141 campos**. É **decisão do
  Enio**, com o número na mão.
- ⏸️ **Abrir/fechar painel nunca foi animado** — é **ausência, não regressão**, e ⛔ **não** é o
  gêmeo da dobra.
- **O hit-test só recebe o produtor de OFFSET**: os outros seis produtores de `LiveGeometry` não
  chegam ao pick. A cura é o pick ler o mapa **fundido** — wave própria.
  ⚠️ Uma superfície `Plain` **nova** que leia `hover_live` sem estar no mapa **nasce muda**; e
  ⛔ **não alargue o censo a todo `Plain`** — isso revive a cerca do estudo §6.2, e há gate.
- A cascata da **F5** · o **menu radial (E4)** · o **realce de proveniência (C2)** · **som de UI
  (D1)**, ⛔ nunca ligado por omissão · **partículas (D2)**.
- Os abertos por-wave dos FX raster estão em [`24_plano_fx_raster.md`](24_plano_fx_raster.md), e os
  das ferramentas de desenho em
  [`25_plano_ferramentas_de_desenho.md`](25_plano_ferramentas_de_desenho.md).

## 3. Smoke

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_BUILD_SMOKE=<n> cargo run -p ph2d-host-desktop --release
```

- `PH2D_BUILD_SMOKE=<n>` — ⚠️ **várias cenas IMPRIMEM o que montaram**: *se a linha não aparecer,
  PARE* — a cena não montou, e o que está na tela é outra coisa.
- `PH2D_UI_MOTION_SMOKE=1..3` — o motion da UI.
- Diagnóstico: `PH2D_BUILD_LOG=1`.

⚠️ **Preferência de utilizador FORA do repo:** `~/.ph2d/prefs.txt` (`motion_character`,
`reduced_motion`). Um `reduced_motion=1` esquecido **reprova smokes sobre produto correto**.

## 4. Contratos

- **Congelado** ([ADR-0056..0068](../architecture/decisions/)): `VectorOp ≤ 16` · `Vertex` SmallVec32
  · `Segment` 64 · `Region.segments` 16 · `AnimValue` enum · `sample(t: f64)` ·
  `MAX_SPIRAL_TURNS = 64` · `MAX_POLYGON_SIDES = 128` · `MAX_VERTICES_PER_LLM_GEN = 1000`.
  Gate `architecture_vector_contract_surface` (escaneia só `ph2d-vector-doc` + `-traits`).
  **PERMANECE congelado** mesmo depois do ADR-0108 — o cutover mexeu em crates satélite, não na
  superfície do doc.
- **O motor novo (`ph2d-vec-*`) tem contrato PRÓPRIO, e ele ainda NÃO está congelado.** Re-congelar
  é follow-up. Mexer no congelado = **Coord-only + ADR** (`CLAUDE.md` §6).

## 5. Onde ler

| quero | vou a |
|---|---|
| **por que o módulo é assim** | [ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md) e o [`18_plano_reposicionamento_rive_native.md`](18_plano_reposicionamento_rive_native.md) |
| **o que já quebrou, e o padrão que se repete** | [`BUGS_vector.md`](BUGS_vector.md) — os 26 estão fechados; o que vale são as **recusas ⛔**, os **padrões** e o índice com o mecanismo |
| **os FX raster** | [`24_plano_fx_raster.md`](24_plano_fx_raster.md) |
| **as ferramentas de desenho** | [`25_plano_ferramentas_de_desenho.md`](25_plano_ferramentas_de_desenho.md) |
| **um VERBO POR FORMA na booleana viva** | [`27_um_verbo_por_forma.md`](27_um_verbo_por_forma.md) — a lei, o padrão-ouro (compound shape vivo do Illustrator), as três decisões, e ⚠️ **os três harnesses de mutação que mentiram** |
| ⛔ **o GRAFO da booleana viva** — CONSTRUÍDO E RETIRADO | [`26_plano_grafo_booleano_vivo.md`](26_plano_grafo_booleano_vivo.md) — o registo da recusa (*"confuso de usar"*), o custo medido, as leis derivadas, e o defeito pré-existente que ele expôs (**✅ curado à parte**: *a tinta do GRUPO é a porta dos operandos dele*, §3) |
| **a booleana viva ANIMA nos estados de UI** | [`28_plano_booleana_viva_nos_estados.md`](28_plano_booleana_viva_nos_estados.md) — os dois canais da pose, a auditoria de 2 lentes (7 achados, os 2 graves invisíveis à suíte) e a cena `=74`. ⚠️ **Ficou fora deste índice ao ser escrito** — recuperado em 24/08 |
| ⭐ **A FILA e a ORDEM** | [`29_fila_morph_state_machine_e_texture_pattern.md`](29_fila_morph_state_machine_e_texture_pattern.md) — **(1)** Input Map · **(2)** máquina de estados do Morph · **(3)** Texture pattern, com o porquê da ordem |
| ⭐⭐ **PLANO: o INPUT MAP** (a começar) | [`30_plano_input_map.md`](30_plano_input_map.md) — Godot + os **contextos com prioridade** do Unreal; ⛔ a **lei nº 1**: *a fita determinística grava a **ação resolvida**, nunca a tecla* (senão um remap reescreve o passado e parte o replay-hash do CI) |
| ⭐⭐ **PESQUISA: máquinas de estado** | [`31_pesquisa_maquinas_de_estado.md`](31_pesquisa_maquinas_de_estado.md) — Rive · Unity Animator (o caso de **ódio**) · Unreal State Tree · statecharts de Harel/XState · Godot. A lei copiável: *várias condições numa seta = **E**; várias setas = **OU*** — o desenho **é** a expressão |
| ⭐⭐ **PLANO: a MÁQUINA DE ESTADOS DO MORPH** — ✅ **FECHADO** | [`32_plano_maquina_de_estados_do_morph.md`](32_plano_maquina_de_estados_do_morph.md) — W1-W11j. ⚠️ **Estava FORA deste índice** (como o 28 esteve): o §5 é a fonte do desenho de 25/08 (⛔ as setas no canvas e o arrasto forma-a-forma foram **retirados**), e o §16 traz a lição das seis waves — *uma fixtura que não contém o fenómeno aprova a cura errada* |
| ⭐⭐ **PLANO: o TEXTURE PATTERN** (o último da fila) | [`33_plano_texture_pattern.md`](33_plano_texture_pattern.md) — o Vello **ladrilha nativamente** (provado ao nível do bit) ⇒ a lei de ladrilho (grid/brick/half-drop/hex, gap, overlap) resolve-se **ao ASSAR** e o quadro custa **um** `fill()`. ⛔ E as três premissas da folha 29 que envelheceram, medidas no §0 |
| ⭐ **PLANO: o CONTORNO que uma forma não consegue ganhar** | [`34_plano_o_contorno_que_falta.md`](34_plano_o_contorno_que_falta.md) — nasceu do report do Enio de 27/08. *Todo* referencial (SVG · Illustrator · Inkscape · Figma) trata **«sem traço» como um VALOR**; o nosso `Option<StrokeSpec>` trata-o como um **buraco**, e um buraco não tem widget. ⛔ E a cura **não** é no `restyle_selected_strokes`: ele corre **por quadro** |
| ⭐⭐ **PLANO: o PADRÃO no TRAÇO** (ordem do Enio, 27/08) | [`35_plano_padrao_no_traco.md`](35_plano_padrao_no_traco.md) — o *"as a fill or stroke"* do Figma. ⭐ O **tamanho decide a representação**, medido: em linha o `VecPath` engorda **54 %** e em `Box` **4 %** ⇒ `Box`, e o preço é o `Copy` do `StrokeSpec` — que o compilador contou em **~15–30 sítios mecânicos**, não nas 287 menções que a nota anterior citava. ⛔ E reusar o `Paint` do preenchimento está RECUSADO: ele representa gradientes que o traço não desenha |
| **texto em caminho · pattern along path · envelope/warp** | [`22_…`](22_plano_texto_em_caminho.md) · [`23_…`](23_plano_pattern_along_path.md) · [`21_…`](21_pesquisa_envelope_warp.md) |
| **o estado por wave** | [`handoffs/README.md`](handoffs/README.md) |
| **a história até 2026-08-18** | [`docs/archive/estado-2026-08-18/vector.md`](../archive/estado-2026-08-18/vector.md) |

⚠️ **Os arquivos `01_…` a `17_…` deste diretório são a spec PRÉ-ADR-0108** — eles descrevem as 30
crates `ph2d-vector-*` que foram **RETIRADAS**. Leia-os como história, nunca como estado.

⛔ **Este README era essa spec.** Ele estava com `status: W0 RATIFIED 2026-05-29`, descrevia as 30
crates retiradas, não mencionava `ph2d-vec-*` uma única vez, e não era tocado desde 2026-06-01 —
**104 KB de estado que já não existia**. O texto integral está, **verbatim**, em
[`docs/archive/docs-2026-08-18/Vector Module/README.md`](../archive/docs-2026-08-18/Vector%20Module/README.md):
as três iterações de crítica adversarial, os cinco eixos, as oito inovações e as 20 waves
planeadas continuam lá, e várias delas **shiparam por outro caminho**. Vá lá para responder *"o que
se pensou em 2026-05"*; ⛔ nunca para decidir a próxima ação.
