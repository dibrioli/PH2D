# ADR-0153 — O auto layout é o `taffy` atrás de uma crate-folha, e a pose que ele produz é DERIVADA

- **Estado:** proposto (⚠️ **número PROVISÓRIO** — o valor se conta contra o `main` do dia; seis
  renumerações já aconteceram neste repo)
- **Data:** 2026-08-02
- **Linha:** `line/Vector` · plano [`docs/Vector Module/Estudos/PLANO_UI_UX_padrao_figma.md`](../../Vector%20Module/Estudos/PLANO_UI_UX_padrao_figma.md) §4 W2
- **Supersede:** nada. **Relacionado:** ADR-0111 (a pose é publicada, não autorada), ADR-0121 (fonte
  autorada ≠ cozida), ADR-0148 (um assador serve preview e apply), ADR-0109 (a cerca das deps).

## Contexto

A moldura (W0) existe e tem tamanho autorado. Falta o que o Figma chama *Auto Layout* e o Rive §7
chama flexbox: **uma moldura que empilha os filhos** (direção, vão, recuo, alinhamento,
crescimento) e os recompõe quando ela ou eles mudam de tamanho. Sem isso, "responsivo" não tem o que
responder — e a barra de ferramentas que o smoke desenha é composta à mão, um retângulo de cada vez.

Duas decisões precisam de registo: **de quem é o motor** e **onde pousa o resultado**.

## Decisão 1 — o motor é o `taffy`, atrás de uma crate-folha

`taffy = "0.12"`, `default-features = false`, features `std` + `taffy_tree` + `flexbox`, isolado na
crate nova **`ph2d-vec-layout`** — que não conhece ECS, nem documento vetorial, nem tema. É a mesma
contenção que confinou o `realfft` na `ph2d-audio-spectral` e o `tract` na `ph2d-audio-ml`, e corta
para os dois lados: nada pesado entra, e **nenhuma UI entra**.

**Por que não escrever o nosso.** Flexbox é uma spec, não um algoritmo: ordem de resolução,
`flex-basis` contra `width`, `min-content`/`max-content`, wrap, `align`/`justify` em nove
combinações. Reproduzi-la é gastar semanas para reencontrar os casos de borda que uma crate madura
já passa — e o `taffy` é o motor do Dioxus, do Zed, do Bevy e do Blitz.

### O que a medição decidiu (M7)

| feature set | deps que entram | cold debug | cold release |
|---|---|---|---|
| `std` + `taffy_tree` + `flexbox` | `arrayvec` | **0,20 s** | 0,17 s |
| \+ `grid` | `arrayvec`, `grid` | **0,63 s** | — |

⇒ **grid NÃO entra.** Triplica o custo de build por um `dir = grid` que nada honraria hoje — o `dir`
que o artista escolhe e que não muda um pixel, que é o controlo morto que a política de UI deste
repo existe para impedir. Ele nasce com a UI que o consumir.

⚠️ **A dep é `deny`-limpa** (crates.io, MIT/Apache-2.0, sem build script, sem lib de sistema) e o
`machete` é honrado: ela entra no commit que a usa.

## Decisão 2 — o resultado é uma POSE DERIVADA, e nunca o `Transform` autorado

> **O passe de layout publica onde as coisas ficam. Ele não escreve onde elas estão.**

⚠️ **Isto não é preferência de estilo, e o preço de errar está medido noutro sistema deste mesmo
repo:** o undo deste editor é **por DIFF do mundo ECS** (`shells/desktop/src/undo.rs`,
`canonicalize`). Um passe que escrevesse `Transform`:

1. faria **cada frame de um redimensionamento virar um passo de undo** — o mesmo defeito que o
   `canonicalize` existe para matar (a ordenação por bits de entidade, 2026-07-09);
2. faria o layout **brigar com o arrasto do artista** dentro do mesmo frame (dois autores de um
   facto, e o de trás ganha em silêncio — a lição do W4 da física).

É a disciplina do **ADR-0111** (a pose de um path é publicada por frame em `VecXforms`) e do
`LiveGeometry` (a geometria consumida é derivada), aplicada a um terceiro facto. Os dois canais já
existem, e é por eles que o layout fala:

- **posição** → o afim publicado (`VecXforms`), que propaga para a sub-árvore inteira do filho;
- **tamanho** (quando `grow`/`shrink` mudam o filho) → a geometria derivada (`LiveGeometry`), que já
  é o canal de *"o que esta forma desenha ≠ o que ela guarda"*.

### Corolário: arrastar dentro de um fluxo é REORDENAR

Se a posição é derivada, um arrasto não tem onde pousar — escrever `Transform` seria escrever num
número que o próximo frame recalcula. Então o gesto muda de significado dentro de uma moldura com
fluxo: ele **reordena** (é o que o Figma faz, e é a única leitura coerente). Isso é gesto, não
desenho, e tem gate próprio.

## Decisão 3 — a árvore do motor é reconstruída, não memoizada

Medido (M2, release, `--ignored` em `crates/ph2d-vec-layout/tests/measure_layout.rs`):

| árvore | nós | ms/passe |
|---|---|---|
| linha achatada, 10 filhos | 11 | 0,0015 |
| linha achatada, 100 filhos | 101 | 0,0117 |
| linha achatada, **1000 filhos** | 1001 | **0,1153** |
| molduras aninhadas, prof. 4 | 17 | 0,0032 |
| molduras aninhadas, prof. 16 | 65 | 0,0126 |
| molduras aninhadas, prof. 64 | 257 | 0,0555 |

**120 ns por nó, constante de 100 a 1000** (o piso de um nó é 0,0001 ms). Mil nós custam **0,7% de
um quadro de 60 fps**, com a árvore **reconstruída a cada chamada**.

⇒ **Não há memoização, e não há TETO de nós.** O plano previa que a medição desse um teto; ela deu o
contrário: o recurso não é escasso. Para o passe comer um quadro seriam ~140 000 nós numa moldura, e
o documento morre muito antes disso por outros motivos. Um teto escrito aqui seria um palpite
esperando um smoke — a §0 do `CLAUDE.md` ao contrário.

⚠️ **O que RE-ABRE esta decisão** é a medição mudar de forma, não crescer: se um passe futuro deixar
de ser linear (uma measure function de texto que reflui, W2a, é o candidato óbvio), a tabela é
re-corrida e o memo volta à mesa com o número novo ao lado.

## Consequências

- ✅ Uma crate-folha nova (`ph2d-vec-layout`), 9 gates, oráculos **aritméticos** (a posição que a
  régua e o vão obrigam), não "o que o motor devolveu".
- ✅ Nenhum contrato do §6 é tocado; nenhum bump de schema (os componentes da wave são **novos**, e
  componente novo cunha `stable_type_id` próprio).
- ⚠️ **A convenção da crate é a do CSS** (`y` para baixo, origem no canto superior-esquerdo da raiz)
  e o documento é **Y-up**. A conversão é UMA, na shell, e está escrita lá — fazê-la dentro do motor
  esconderia metade de uma troca de eixo num sítio que não conhece o eixo do documento.
- ⚠️ **Um pai que não flui é RECUSADO** (`LayoutError::ParentDoesNotFlow`), e o gate existe porque o
  contrário passou: o default do `taffy` é `Display::Flex`, então um nó sem estilo de moldura
  **dispõe os filhos na mesma** — em silêncio, com o artista a ver as formas dele saltarem para um
  canto. Quem monta a fatia recolhe só sub-árvores que fluem.
- ⚠️ **O texto ainda não sabe REFLUIR.** Ele entra como folha com a bbox que já tem (os glifos são
  contornos assados), o que é correto e é o que a W2 entrega; quebrar linha a uma largura é a W2a, e
  ela traz uma measure function — e, com ela, a M2 é re-corrida.
