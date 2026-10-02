# ARQUIVO — CLAUDE.md (história, 58 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-548,607-618` do original.
>
> ⚠️ **A única alteração ao corpo:** 17 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Painter** — host de Layers + Efeitos ([ADR-0099](../../architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md))
  **mais** um motor de pintura clean-room do Blender Texture Paint (ref vendorizada, GPL ⇒ só comportamento). Quatro
  **meios** num dropdown (`Digital` é o default) — Digital · Watercolor · **Impasto** (relevo com material por-pixel e
  luz na GPU) · **Wet Paint** (sim de fluido, [ADR-0134](../../architecture/decisions/0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md),
  solver independente de ordem [ADR-0147](../../architecture/decisions/0147-wet-paint-order-invariant-solver.md), row-parallel [ADR-0145](../../architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md)).
  Mais **sculpt do relevo** (8 verbos), **liquify** ([ADR-0157](../../architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md)),
  **substrato/papel**, **taper**, **grid stamp**, seleção com caneta, e o carimbo no device (`ph2d-paint-gpu`).
  ⚠️ **A lei que este módulo pagou seis vezes:** *o traço é fato do **CAMINHO**, nunca de quão fino o motor amostrou o
  caminho* — um produto por-dab depende do Spacing e do polling; a forma certa é envelope ou integral de arco.
  ⚠️ **`DEPTH_UNIT_PX = 16.0`**: toda grandeza **geométrica** sobre `h` cruza essa conversão na entrada.
  **Aberto:** ⛔ a composição da cobertura da aquarela muda **toda** cruz/laço/hachura já pintada — **produto** ·
  ⛔ pré-agrupar por banda as arestas do `fill_coverage` (o pré-filtro ingênuo é **mais caro que o passe inteiro**) ·
  ⛔ o caráter do Speed sem rampa **reabre um look que o Enio já recusou** · o **endurecimento da borda da máscara**
  (as duas leis de acúmulo já foram tentadas, cada uma com artefato — a próxima hipótese tem de estar noutro lugar) ·
  a cauda do taper no impasto (o próximo passo **não é código**) · `undo` de knobs de painel **não existe em nenhuma
  ferramenta do app** — decisão do Enio · dois `watercolor_app_params_incremental_*` seguem `#[ignore]` com **diagnóstico
  novo** (é raio de invalidação, e ⛔ `pad += 2·raio` **não** é a cura: vira canvas inteiro por quadro) · dois gates de
  razão de `plane_copy`/`undo_delta` vermelhos porque **a premissa da calibração dissolveu** (o serial deixou de ser
  fault-bound) — pede varredura por tamanho com alocação **fria**, ⛔ não baixar a barra.
  ⭐⭐⭐ **E A METADE-SHELL SAIU** (12/09, W2 Fase D): a família vive em
  [`ph2d-app-painter`](../../../crates/ph2d-app-painter) (48 f / 10 547 L) e a shell desce de **225 394 para
  215 394** linhas, com `ONLY-A = 0`, `ONLY-B = 3` e 79 `MOVED`. ⭐ **Os 40 ficheiros estavam presos
  por SEIS SÍMBOLOS**, e nenhum pediu porta nova no `AppHost`: duas âncoras eram **fachadas** (o
  `image_import` de 6 linhas; o `PainterPreview`, que já era alias de uma crate), uma era a porta #5
  que já existia, e três caíram escritas em TIPOS — um `bool`, um **fecho de leitura** (o idioma que
  a `ph2d-tool-runtime` já declarava por escrito) e um contador. ⭐⭐ Duas folhas novas:
  [`ph2d-sprite-screen`](../../../crates/ph2d-sprite-screen) (o afim `imagem-px → ecrã-px`, partilhado por
  **quatro** assuntos) e [`ph2d-preview-slot`](../../../crates/ph2d-preview-slot) (a ranhura de GPU que o
  `app_state.rs` guardava por inércia — ⛔ **não** foi para a `ph2d-tool-runtime`, onde o gémeo de
  CPU vive, porque o teto de LOC dela se declara *«the discipline mechanism»* e os 17 de folga só
  chegariam apagando prosa **medida**). ⛔⛔ **E mover um ficheiro TROCA O REGIME DE TETO que o
  governa:** o `painter_bridge.rs` atravessou a fronteira a 1093 linhas com um `// ph2d-loc-cap:`
  que é **inerte** em `crates/`, e partiu-se em três por responsabilidade (`1093 → 691`).
  ⚠️⚠️ **E a régua do fecho tem um furo que erra A FAVOR — ela não resolve `super::`**: leu `2`
  ficheiros onde havia `11` e perdeu uma âncora inteira. O que fica na shell são **961 L em 6
  ficheiros** (os gestos de canvas e o Apply) — a costura, por desenho
  ([handoff](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_app_painter_2026-09-12.md): o §5 tem as
  **oito** armadilhas, entre elas cinco `#[cfg(test)]` órfãos que se colaram ao módulo vizinho **em
  silêncio**, e o §6 as **cinco** premissas minhas que a medição derrubou).
  ⭐⭐⭐ **E a `line/PainterWatercolor` fechou em 25/09 (140 commits, smoke do dono aprovado em cada wave):** a costura
  do retorno na aquarela · a lei da tinta K–M na folha `ph2d-pigment` (o Digital mistura por ela; a aquarela não,
  recusa medida) · o **Composite Brush** de até 7 camadas, montado à mão, com ordem por traço, e o Blur da pilha em
  caixa · o produto em **`x86-64-v2`** (ADR-0174) · o **Wet Paint** a `~8 ms` por quadro a raio 250 (era `~31`) ·
  ADRs **0171–0175**. ⚠️ **Na integração (com a UIUX) o cartão *Mixing* subiu a altura de abertura do
  `painter_layers` de `1 529` para `1 596`** — é a SOMA das duas linhas, com a conta fechada no gate
  (`+67` = uma fileira com moldura). [Handoff da linha](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md)
  · [diário](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md).
  ⭐ **E reabriu (26/09–01/10): a fila 44 fechou inteira (12/12) e a pilha do Composite a Size 0.5 vai de `~40` para `60` fps** (acumular por quadro · o Blur em linhas · o campo do esfregão em faixas e AO LADO do acúmulo · o motor do pincel num só pedaço no `smoke`). [Handoff](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-01.md) · smoke `PH2D_COMPOSITE_RABISCO=<Size>` (risca sozinho e diz o período).
  **Smokes:** `PH2D_IMPASTO_SMOKE=1|2` · `PH2D_COMPOSITE_SMOKE=1` · `PH2D_WETPAINT_SMOKE` (+ `PH2D_FLUID_PROFILE=1`) · `PH2D_MASK_SMOKE` ·
  `PH2D_TAPER_SMOKE` · `PH2D_LINE_SMOKE` · `PH2D_SUBSTRATE_SMOKE`. Diagnóstico: `PH2D_PAINT_PERF=1` ·
  `PH2D_PREVIEW_DIAG` · `PH2D_PREVIEW_DUMP=<dir>`.
  ⚠️ **Rode a suíte do Painter em DEBUG também** (precedente registrado), e os `--ignored` com **`--test-threads=1`**
  e a máquina calma.
  **Ler:** [`docs/Painter/`](../../Painter) · [`BUGS_painter.md`](../../Painter/BUGS_painter.md) ·
  [`28_otimizacoes_o_que_funcionou.md`](../../Painter/28_otimizacoes_o_que_funcionou.md) (o log de perf, com o que foi
  **rejeitado por medição**) · [handoffs](../../Painter/handoffs/README.md) ·
  [história](../estado-2026-08-18/painter.md)

