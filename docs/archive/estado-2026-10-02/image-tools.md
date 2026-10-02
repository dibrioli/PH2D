# ARQUIVO — CLAUDE.md (história, 15 linhas)

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
> Recorte: linhas fora de `1-1828,1844-1855` do original.
>
> ⚠️ **A única alteração ao corpo:** 8 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Image Tools — os utilitários de bitmap** (⚠️ **~30 k LOC que esta seção nunca mencionou**, achado
  da auditoria de 2026-08-18): `ph2d-tool-color-equalization` (10.291) · `ph2d-tool-bgremoval` (8.377) ·
  `-upscale` (1.788) · `-equalize-sizes` · `-rasterize` · `-padding` · `-make-square` · `-real-size` ·
  `-trim-transparency`, mais os painéis irmãos. Cada uma é **drop-crate** sob o contrato `Tool=12` (§6),
  e três implementam `RasterEditTool` — ⚠️ **quatro**, contando o Painter: quem escrever a 5ª herda essa conta.
  Vizinhos sem entrada própria: `ph2d-inpaint` (2.140) · `ph2d-grid` + `ph2d-panel-grid-snap` (7.012) ·
  `ph2d-tokens` (5.141, o design system do §7) · `docs/Deform/` · `docs/Pixel Art/`.
  **Ler:** [`Image Tools Bugs`](../../Image%20Tools%20Bugs/README.md) · **Inpaint** = PatchMatch multiescala CPU+GPU
  ([ADR-0102](../../architecture/decisions/0102-inpaint-multiscale-patchmatch-cpu-gpu.md), [plano](../../Inpaint/01_pesquisa_design_plano.md)) ·
  **Deform** = transformação/deformação do Painter, com [tracker único](../../Deform/00_README.md) e [índice](../../Deform/README.md)
- ⚠️ **As duas maiores crates do repo não eram nomeadas em lugar nenhum deste arquivo:**
  [`ph2d-tool-painter`](../../../crates/ph2d-tool-painter) (**136.093 LOC** — é onde o módulo Painter de facto
  vive; o §5 nomeava só `ph2d-paint-gpu`) e [`ph2d-editor-core`](../../../crates/ph2d-editor-core) (**97 385** em `src/`, medido 13/09 depois da refatoração final —
  widgets, interaction e os ids que ela própria lê; ⭐ **1 944 ids desceram para as crates que os lêem** ([A5b](../integracao-jornadas/HANDOFF_INTEGRACAO_line_editor_core_2026-09-12.md)) e os módulos de topo formam um DAG com catraca de 7 arestas; **27** gates `architecture_*` entre os **113** ficheiros de `tests/it/`). *Um módulo que o roteador não
  nomeia é procurado por `grep`, não alcançado por link.*
