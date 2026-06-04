═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador / Design · 3 tokens cromáticos p/ tinta de canal das Curves
Autor: Implementador Painter (sessão 2026-06-04) · W4 §3 editor de curva completo
═══════════════════════════════════════════════════════════════════

## Pedido (1 coisa)
Adicionar **3 tokens de cor** — `curve-r` / `curve-g` / `curve-b` — pro editor de
curva tingir a curva (e o anel do handle) pela aba ativa: R = vermelho, G = verde,
B = azul; master (RGB) continua `accent`. Hoje a curva é `accent` em todas as
abas; falta o feedback cromático "qual canal estou editando".

**Por que tokens novos (e não os existentes):** o `ColorToken` só tem cores
SEMÂNTICAS. `Danger`/`Success`/`Info` são estados (erro/ok/info), não garantidos
R/G/B em todos os 4 temas, e usá-los pra "canal" é abuso semântico. Hex/literal
quebra HR-15. Então preciso de tokens cromáticos dedicados — design-system, fora
do meu escopo de painel.

## O que mexer (3 lugares — o mesmo padrão de `grid-line`/`grid-axis`)
1. **`docs/design/tokens.json`** — em CADA um dos 4 blocos de tema, 3 entradas
   `oklch(L C H / α)`. Mantenha o canal RECONHECÍVEL (vermelho/verde/azul) e
   harmônico com o tema; afine L p/ contraste sobre `bg-2`. Ponto de partida
   (design finaliza por tema):
   - temas ESCUROS (curva sobre bg escuro): L ~0.65
     - `curve-r`: `oklch(0.640 0.200 27)`
     - `curve-g`: `oklch(0.720 0.170 145)`
     - `curve-b`: `oklch(0.640 0.150 258)`
   - temas CLAROS (curva sobre bg claro): L ~0.55 (mais escuro p/ contraste)
     - `curve-r`: `oklch(0.560 0.200 27)`
     - `curve-g`: `oklch(0.600 0.170 145)`
     - `curve-b`: `oklch(0.540 0.160 258)`
   (Chroma moderada — não os primários sRGB crus, que ficam berrantes; mesma
   filosofia perceptual do resto do `tokens.json`.)
2. **`crates/ph2d-tokens/src/color.rs`** — 3 variantes no enum `ColorToken`
   (`CurveR`/`CurveG`/`CurveB`) + os slugs no `match self` (`"curve-r"` etc.),
   espelho exato de `GridLine => "grid-line"` (color.rs ~328/375).
3. **Sync/regen** se houver codegen de tokens.json → tabela de tema (mesmo passo
   que `grid-line`/`grid-axis` exigiram). ⚠️ Cheguei a procurar gate de CONTAGEM
   de `ColorToken` e não achei, mas confirme (token/widget-sync) — 3 variantes
   aditivas não devem ripplar nada além do sync.

## Como eu ligo (assim que os tokens existirem — 2-3 linhas, MEU)
`crates/ph2d-panel-painter-layers/src/paint_adjust.rs::paint_curve_editor`:
```rust
let curve_color = resolve(match channel {
    1 => ColorToken::CurveR, 2 => ColorToken::CurveG, 3 => ColorToken::CurveB,
    _ => ColorToken::Accent,           // master = accent
}, theme);
// usar curve_color no stroke_polyline da curva + no `ring` do handle
```
Não é scaffold morto: o editor já está completo e funcional (`121e294`); isto é
só trocar a cor `Accent` fixa por `curve_color`. Me avise quando landar que eu
fecho em 1 commit.

## Estado (commits locais, não pushados)
W4 §3 editor de curva COMPLETO: drag 2D-livre (`fe9969e`), abas RGB/R/G/B +
add/remover ponto (`0faec14`+`e1767fe`), grade + diagonal identidade (`121e294`).
Falta só esta tinta cromática (bloqueada nos tokens). Isolamento: fiquei em
painel/tool/ids-aditivos; tokens são design/Coord.
═══════════════════════════════════════════════════════════════════
