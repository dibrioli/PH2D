---
name: feedback-try-to-build-the-harness-before-declaring-it-impossible
description: "'Exige janela/GPU, não dá pra gatear headless' — TENTE construir o objeto antes de escrever isso; o App do PH2D nasce sem janela"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 55388e1a-541d-4d65-8237-d22637f7df4f
---

Um gate foi **deletado com nota** ("o `App` exige janela, não há teste headless que o construa") e a
propriedade ficou sem cobertura por uma jornada inteira. **A premissa era falsa.** No winit 0.30 a
janela nasce no `resumed`, então `App::new()` devolve `window`/`host`/`gfx` em **`None`** — o app roda
o primeiro frame assim. E todo passo do `project_load` que depende de `gfx` **já degradava para no-op
de propósito**. O harness existia; ninguém tentou.

**Why:** "não é testável" é uma afirmação sobre o CÓDIGO, e como toda afirmação sobre o código ela se
verifica lendo/rodando — não se deduz do tipo. O custo de checar é um `cargo test`; o custo de errar é
uma propriedade viva sem gate, que foi exatamente o que aconteceu (o relógio do load), e ainda
*escondeu 3 bugs confirmados* que só apareceram quando o gate finalmente rodou.

**How to apply:** antes de escrever "exige harness que não existe" (ou de pedir um), **construa o
objeto**: `let x = X::new();` num `#[test]` e veja o que quebra. Em binários (`shells/*`), o teste
unitário mora DENTRO da crate e enxerga item privado de módulo ancestral — `App::new()` é acessível de
`crate::project::tests`. Só depois de ver o erro real é que "precisa de janela/GPU" vira um fato.
Correlato: [[feedback_painted_is_not_populated_paint_gate]] — um gate que não roda o caminho real não
prova nada, e "não deu pra rodar o caminho real" merece uma tentativa antes de virar nota.
