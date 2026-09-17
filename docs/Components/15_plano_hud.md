# TOP-20 #20 — O HUD MÍNIMO: `UiCanvas` · `UiAnchor` · `UiLabel` · `UiButton`

> *«placar, vida e menu fecham o loop de demo — o botão publica Signal para a tabela do item 5»*
> ([levantamento](00_levantamento_componentes.md) §341). Ordem do dono, 2026-09-17: *«escolha um e
> implemente»*, sobre os três abertos que restavam.

## §1 — O que a MEDIÇÃO mudou no plano, antes da 1.ª linha de código

⭐⭐⭐ **A §5.0 do `CLAUDE.md` mandou medir se a composição já exprime o item, e ela exprime quase
tudo.** O levantamento dizia «parcial»; medido, o substrato é este — e cada peça foi **verificada no
código**, nunca acreditada a partir do doc:

| Peça que o plano ia construir | O que JÁ existe | Veredito |
|---|---|---|
| `UiAnchor` (prender aos cantos) | [`VecAnchors`](../../crates/ph2d-ecs/src/vec_anchors.rs) + o passe vivo [`layout_live_anchors`](../../shells/desktop/src/layout_live_anchors.rs) | ⛔ **NÃO construir.** O doc do passe **nomeia o HUD como o caso de uso dele**: *«É o HUD: a vida no canto de cima, a pontuação colada no canto oposto, a barra que estica no meio.»* |
| Empilhar filhos (hotbar, menu) | `VecLayout` + taffy ([ADR-0153](../architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)) | ⛔ NÃO construir |
| Texto | `VecShape::Text` + `VecTextParams` + `text_to_compound_path` | ⭐ reusar |
| Geometria DERIVADA por quadro | `LiveGeometry = BTreeMap<VecPathId, Vec<VecPath>>` | ⭐ a porta da substituição — **o documento não se toca** |
| Estado visual de um botão | `ph2d-ui-state` (+ Smart Animate) | ⭐ reusar |
| A vista do jogo, em metros | `fase_game_camera` **já a devolve** (o `DestroyOutside` do #12 lê-a) | ⭐ reusar |
| Pose conduzida por um motor sem sujar o undo | `ph2d-preview-drive` (`Driver`/`Driven`) | ⭐ reusar |
| Publicar um sinal | `SignalOutbox::publish` | ⭐ reusar |

⇒ **A wave não é «construir um sistema de UI». É fazer a UI que já existe responder ao JOGO**, e são
quatro coisas: a raiz que se cola à vista · o clique que publica · o texto que muda · e um número
para mostrar.

## §2 — O ORÁCULO (Godot 4.7.2, MIT, corrido SEM INTERFACE)

Sonda versionada: [`ferramentas/godot_hud_probe.gd`](ferramentas/godot_hud_probe.gd).
⛔⛔ **A 1.ª redacção mediu o NADA em dois dos três blocos e foram os CONTROLOS que o disseram** — o
cabeçalho da sonda guarda as três cegueiras (a câmera que não enquadrava · a janela que não obedece
em headless ⇒ *inverter a experiência* · o MOUSE MOTION em falta, sem o qual o botão mede outro
programa). *Um oráculo sem controlo positivo do próprio sujeito lê-se como uma medição.*

### L1 — a imunidade à câmera (com o controlo ao lado)

| câmera | transformação do VIEWPORT (o CONTROLO) | a do `CanvasLayer` |
|---|---|---|
| `(0, 0)` | `(360,0 · 225,0)` | `(0,0 · 0,0)` |
| `(200, 0)` | `(160,0 · 225,0)` | `(0,0 · 0,0)` |
| `(200, 120)` | `(160,0 · 105,0)` | `(0,0 · 0,0)` |

⭐ E a lei apareceu **pelo avesso** na 3.ª passagem: com o botão FORA do canvas o bloco L3 deu **zero
disparos**, porque a câmera desloca o canvas do viewport e o clique caía ao lado.

### L2 — a escala por resolução (janela fixa `720×450`; a referência é que varia)

| `aspect` | a lei | conferido |
|---|---|---|
| `ignore` | escala **por eixo**, `(J/R)`, deslocamento `0` | `ref(500,400)` → `(1,4400 · 1,1250)` |
| `keep` | uniforme `min(Jx/Rx, Jy/Ry)`, **centrado** | `ref(320,480)` → `0,9375`, desloc. `x = (720−300)/2 = 210` |
| `expand` | uniforme `min(…)`, **sem centrar** (canto) | `ref(1280,360)` → `0,5625`, desloc. `0` |
| `keep_width` / `keep_height` | ⛔ **FORA**: eles redimensionam o VIEWPORT, e `get_final_transform` não o mostra (nas referências mais altas que a janela degeneram no `keep`) | medido e **não portado** |

⛔ **Divergência DECLARADA:** o alvo **arredonda o deslocamento a pixel inteiro e encolhe a escala
para caber** (`keep`, `ref(1280,360)`: escala `y = 0,561111` e desloc. `124`, em vez de `0,5625` e
`123,75`). O nosso canvas vive em **metros**, não em pixels — guardamos o valor exacto, e o gate
afirma-o.

### L3 — quando um botão dispara

| caso | o alvo |
|---|---|
| carregar DENTRO, largar DENTRO | ⭐ dispara **uma vez, ao LARGAR** |
| carregar DENTRO, **mover para fora**, largar FORA | **não** dispara |
| carregar FORA, largar DENTRO | **não** dispara |
| desactivado | **nunca** |

## §3 — O desenho

1. **`ph2d-hud`** — crate-folha, zero dependências: `Fit ∈ {Stretch, Keep, Expand}` e a porta única
   `place(canvas, vista) -> Placement { escala, deslocamento }`, com as três leis da tabela L2.
2. **`UiCanvas { ref_w, ref_h, fit }`** — o `Transform` da entidade é **conduzido** por quadro para
   que a caixa de referência caia sobre a vista do jogo. Vai pelo **ledger** (`Driver::CanvasPose`),
   logo não entra no ficheiro nem no `Ctrl+Z` — a lei do `preview_drive`, que o #16 já usou.
   ⇒ os filhos herdam por `Transform`, e as âncoras e o fluxo que já existem re-fluem de graça.
3. **`UiLabel { fonte }`** — a string é DERIVADA (contador · relógio · contagem de uma tag · literal)
   e os glyphs re-cozidos vão para a `LiveGeometry`. ⛔ **O documento não é reescrito** — é a lei do
   ADR-0153 (*o passe publica o que a coisa MOSTRA; ele não escreve o que ela É*).
4. **`UiButton { sinal }`** — durante a **corrida** (`playhead.is_playing()`, a lei da fábrica), um
   clique dentro dele publica o sinal, com as quatro células do L3.
5. **`Counter` + o verbo `AddToCounter`** na tabela do #5 (append-only) — sem ele o placar não tem o
   que mostrar: *«bateu na moeda → soma 1 → o placar muda»* fecha com o `SignalOnHit` que já existe.

## §4 — Onde encosta

`PROJECT_SCHEMA` **+1** · registo do `ph2d-ecs` **+4** e os dois espelhos **+4** · `SignalVerb` e
`Driver`/`Driven` **append-only** · `LIVE_SECTIONS` **+4**. ⛔ **Zero contrato congelado** (§6): o
`Tool=12`, o `NodeOp` e a superfície do vector-doc não são tocados.
