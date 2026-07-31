# ADR-0151 — A expressão é POR-CLIP, então um strip a janela

**Data:** 2026-07-27 · **Status:** aceito (Enio 2026-07-27) · **Numerado na integração
(2026-07-31):** nasceu **0145** e virou **0151** — a `line/Painter` levou 0145·0146·0147 na mesma
janela, a `line/Vector` o 0148, a `line/physics` o 0149 e a `line/sculpt3d` o 0150; como os NOMES de
arquivo diferem, o git **nunca conflitou**, e quem pega é o gate `architecture_adr_numbers_are_unique`.
*Um número escolhido numa linha paralela é PROVISÓRIO: ele se CONTA a partir do `main` do dia.*
**Segue** e emenda o [ADR-0144](0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md)
(§F já nomeava este follow-up: *"`expr` por-track (por-clip) … Per-clip é follow-up"*).

## Contexto — a força que obriga a decidir

O ADR-0144 pôs a expressão no **binding** (document-wide). Smoke do Enio (2026-07-26): uma
expressão **PURA** (Time/Slider, `time*1.2`, **sem keyframes**) **extrapola o strip** — toca
além dele no container e no Arrange. A causa é estrutural, não um bug: uma prop pura **não tem
track**, logo **nenhum strip a referencia**, logo **não há janela a obedecer**. A cura para uma
prop COM keys já fechou (a cobertura da composição, `512c19f9`); a pura precisa de um **vínculo
explícito** a um clip.

O Enio escolheu, entre três desenhos: **por-clip (como os keyframes)** — o modelo *precomp* do AE.

## Decisão

**A fórmula de uma expressão vive no CLIP** (`NamedClip.expr: BTreeMap<AnimTarget, String>`),
exatamente como os keyframes vivem no track do clip. **Um strip que toca esse clip a janela**, e a
expressão é avaliada no **tempo LOCAL do strip** (a mesma `sole_strip_of` + `strip_source_time`
que o seed do K já usa). Fora do strip, ela fica **QUIETA junto com os keys**. Na vista de um
clip só (sem stack), o clip ativo toca por inteiro (cortado por `clip_cut`), então a expressão
toca por inteiro.

Uma frase que sobrevive fora de contexto: **a expressão é animação do clip; ela vai aonde o clip
vai, e só enquanto o clip toca.**

### Como

1. **Storage:** `NamedClip.expr` (map `AnimTarget -> String`), apêndice posicional ⇒ **`DOC_VERSION`
   15→16**, v15 recusado no load (a política deste documento desde o ADR-0133). Mora no
   `NamedClip` (o invólucro de metadados por-clip: loops, `length_override`) e **não** no
   `ph2d_anim::Clip` (a crate de anim não sabe o que é uma expressão da timeline).
2. **Autoria:** o `SetBindingExpr` (o menu "Expression…" da track) escreve no **clip ATIVO** — é
   onde se autora a animação por-clip (a vista Keys). O `binding.expr` (document-wide) do ADR-0144
   **fica** como o driver **GLOBAL** (o "Arrange" da escolha do Enio): toca a cena inteira,
   janelado só por `cut_scene`. Em v1 o menu da track escreve por-clip; a autoria explícita de um
   global do Arrange é follow-up nomeado.
3. **Avaliação** (`expr_pass`, ainda o passe SEPARADO — **nunca entra no BLEND**
   `sample_stack`/`eval_frame`, o `fade_fingerprint` segue intacto; usa só as leituras
   READ-ONLY do layout de strips, `sole_strip_of`/`strip_source_time`, que dizem ONDE um strip
   toca, nunca um peso de fade): recebe o `scratch` (o layout de strips). Para cada expr
   por-clip de `(entity, target)`, resolve a janela+tempo pela porta única
   `clip_expr_time(scratch, clip, entity, t)`:
   - **stacked:** `sole_strip_of(scratch, clip)` → o strip; `strip_source_time` → o tempo local.
     Zero strips ⇒ quieta; o clip tocando 2× (`PlaysTwice`) ⇒ quieta (edge honesto, igual ao K).
   - **não-stacked:** só o clip ATIVO toca, por inteiro, em `solo_source_time` (cortado por `clip_cut`).
   O `value` (pré-expressão) segue vindo do `composed` (a cobertura da composição, ADR-0144).
   O `binding.expr` global segue avaliado em `cut_scene(t)`.

## Alternativas REJEITADAS (as três da pergunta ao Enio)

- **Apontar a expressão para um strip (Pick), scope no binding.** Explícito e sem ambiguidade,
  mas é um **gesto novo** e mantém a expressão document-wide com um `Option<strip>` ao lado — um
  2º modelo de "onde esta animação vive" concorrendo com o clip, que já é a resposta para os keys.
  Rejeitado: o clip já É o escopo da animação; um scope paralelo divergiria dele.
- **Só as expressões SOBRE keys obedecem; as puras ficam globais.** Zero mudança, zero schema — mas
  **não conserta o Slider** (o caso reportado), e força o artista a keyar uma prop só para janelar
  a fórmula. Rejeitado pelo Enio.
- **`expr` por-track (não por-clip).** Igual em efeito ao por-clip, mas uma prop pura não tem track
  — precisaria criar um track vazio só para carregar a fórmula, e um track sem keys não produz
  cobertura. O map no clip evita o track-fantasma.

## O preço (explícito)

- **`DOC_VERSION` 15→16** (recusa saves v15 — provisório até integrar).
- **Duas fontes de fórmula** (clip = janelada · binding = global). A precedência é: onde o strip do
  clip cobre, a por-clip; senão a global; senão os keys. Um prop com as duas é o artista pedindo as
  duas (raro; documentado, não proibido).
- **Multi-strip do MESMO clip** (o clip tocando 2× no mesmo instante) ⇒ a expressão fica **quieta**
  (`sole_strip_of` recusa) — o mesmo veredito honesto que o K dá para "keyar aqui tem 2 respostas".
- **A autoria de um global do Arrange** (o `binding.expr` pela UI) é follow-up; em v1 o menu da track
  autora por-clip (o caso reportado).
- **A FASE de um termo `time` puro** dentro do strip agora é o tempo LOCAL do strip (correto); o
  refino residual da fase do wiggle citado no plano 07 §11 é subsumido por isto.

## O que fica GATEADO

- **`documento sem expr é byte-idêntico`** + **`fade_fingerprint` intacto** (o passe segue com
  early-out; o arch-gate vira *"o passe NUNCA entra no BLEND"* — `sample_stack`/`eval_frame` —,
  refinado do *"nunca chama stack_eval"* do ADR-0144, porque as leituras de layout de strips não
  computam fade).
- **`uma expressão pura por-clip fica QUIETA fora do strip`** (o bug reportado) — a MESMA cena com o
  strip movido/cortado janela a expressão; mutação: tirar a `clip_expr_time` ⇒ toca em todo lugar.
- **`a expressão pura roda no tempo LOCAL do strip`** — mover o strip de `[0,2]` para `[5,7]` desloca
  a fase; mutação: usar `t` cru ⇒ a fase não desloca.
- **`o clip ativo (não-stacked) toca por inteiro`** (a vista Keys não regride).

## Consequência para a próxima LLM

A expressão deixou de ser "uma prop do documento" e virou "uma prop DO CLIP", como os keys. Quem
mexer no fade/stack segue sem precisar saber que expressões existem (o passe roda depois, o
arch-gate garante). Quem adicionar um 3º escopo de expressão herda a precedência clip>global>keys.
