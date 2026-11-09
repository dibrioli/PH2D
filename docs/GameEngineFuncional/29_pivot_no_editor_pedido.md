# 29 — O Pivot da Animated Sprite mora no editor de animações (pedido do dono, 09/11)

Pedido (verbatim, resumido): *«uma solução inteligente para o PIVOT das Animated Sprites, com os ajustes no Editor de
Animações. Dentro das opções do Align, um botão para ajuste manual e um widget de pivot como o do Inspector, só que com
25 pontos em vez de 10. Assim aproveitamos a sobreposição e a correção do Align e pomos o pivot alinhado relativamente a
todos os quadros. Para Animated Sprites a opção de ajustar o pivot no canvas deixa de existir; se o usuário tentar, uma
mensagem diz que deve ser feito no editor de animações.»* Referência visual: o widget Pivot do Inspector (3 × 3 pontos +
o central em acento).

## Decisões do dono (09/11, por pergunta)

1. **Os 25 pontos contam sobre TODOS os quadros de TODAS as animações**, já alinhados (a união das caixas opacas,
   com os acertos de folha e de célula): «embaixo, meio» = os pés em qualquer animação; o Pivot não salta ao trocar.
2. **O campo Pivot do Inspector também vai ao editor**: numa Animated Sprite vira um botão que abre o editor de
   animações com o Align ligado. Um só lugar.
3. Canvas: a ferramenta Pivot numa Animated Sprite não age; uma mensagem manda ao editor de animações.

## O que existe (para a próxima LLM)

- O Pivot de 9 pontos e o encaixe: `crates/ph2d-sprite-edit/src/pivot.rs` (`fracao_opaca`, `set_pivot`, `o_resto_fica`:
  mudar o pivô não move o desenho; a posição compensa). O Pivot é POR FOLHA (`anchor` do `Sprite` / `ExtraSheet`).
- A prévia Align: `crates/ph2d-anim-editor/src/align.rs` (cruz do Pivot = origem local `(0, 0)`, `align_point`,
  `drawing_box`, `silhouette_of`), moldura em `layout.rs` / `paint_align.rs`.
- ⚠️ Com várias folhas, «pôr o pivô» é mover a ORIGEM do objeto em relação ao desenho alinhado: todas as folhas andam o
  mesmo delta (o `anchor` de cada uma) e a posição compensa — os acertos de célula ficam.

Plano por escrever (`/pd-feature`), com maquete da moldura antes do código.
