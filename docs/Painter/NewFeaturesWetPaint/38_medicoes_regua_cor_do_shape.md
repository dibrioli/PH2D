# ARQUIVO — HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P38_COR_DO_SHAPE_EM_CAMADAS.md (história, 16 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P38_COR_DO_SHAPE_EM_CAMADAS.md`](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P38_COR_DO_SHAPE_EM_CAMADAS.md) em 2026-10-15, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-75,92-122` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## §4 — Medições (régua `tests/rampa_do_shape.rs::regua_a_rampa_do_shape_segue_a_textura`)

Um carimbo, rampa vermelho → azul, matiz `t = b/(r+b)`; «cobertura» = α médio ÷ o da queda sozinha.

| meio / forma / queda | antes (= Falloff Color hoje) corr padrão · queda · cob. | Texture Color | Grain Color (ref.) |
|---|---|---|---|
| Digital Checker macia | 0,141 · 0,932 · 0,500 | 1,000 · 0,000 · 0,999 | 1,000 · 0,000 · 0,999 |
| Digital Noise macia | 0,459 · 0,239 · 0,281 | 0,982 · −0,568 · 0,999 | 1,000 · −0,568 · 0,999 |
| Digital Checker dura | 0,974 · — · 0,500 | 1,000 · — · 1,000 | 1,000 · — · 1,000 |
| Impasto Checker macia | 0,455 · 0,790 · 0,500 | 1,000 · 0,000 · 1,000 | — |
| Watercolor Noise dura | 0,215 · 0,911 · 0,637 | 0,925 · 0,011 · 0,637 | — |
| Watercolor Checker dura | −0,104 · 0,819 · 0,680 | 0,634 · −0,360 · 0,680 | — |

A Falloff Color reproduz a coluna «antes». Na aguada o xadrez fica em 0,63 (espalha a cor entre
quadrados vizinhos; o Noise dá 0,92).

