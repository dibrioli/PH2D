# ADR-0144 — Expressões na timeline: IR congelado, num passe pós-composição SEPARADO

- **Status:** aceito (provisório na `line/anim`; o número do ADR e o `DOC_VERSION` **renumeram na integração** se colidirem — [[feedback_numbers_that_sum_across_lines_count_dont_pick]])
- **Data:** 2026-07-26
- **Linha:** `line/anim` (Wave C do plano [`docs/Timeline/07_plano_joias_da_coroa.md`](../../Timeline/07_plano_joias_da_coroa.md) §7)
- **Contexto:** Enio mandou seguir a Wave C. O plano exige ADR ANTES de construir — subsistema novo, alternativas reais.

## O problema e a força que obriga a decidir agora

Uma propriedade animada hoje só pode ser dirigida por **keyframes**. A feature mais amada
do After Effects é a **expressão**: uma propriedade dirigida por uma **fórmula** de tempo
e/ou de OUTRAS propriedades — `time*100`, `wiggle(3, 20)`, "linka a `Sprite.position.x`".
**Não existe mecanismo nenhum hoje** (confirmado: `ph2d-timeline`/`ph2d-anim` não têm
expressão nem driver).

A força que obriga a decidir **agora**, antes de escrever qualquer linha: a timeline carrega
o **sistema precioso de Clips/Strips/Containers/Fade** (ADR-0115/0133), cujo avaliador
(`stack_eval`) é guardado por um *fingerprint* que **não pode mover um byte**. Um subsistema
novo que compõe valores tem exatamente um jeito de contaminar esse avaliador, e a decisão
de ONDE ele roda é irreversível depois de shipada. Então ela vem primeiro.

## A peça que já existe (e o que falta)

- **`ph2d-expr`** (ADR-0033) é um **IR + evaluator, SEM parser de texto**: `Expr`
  (`Const`/`Attr`/`Param`/`Unary`/`Binary`/`Call`/`Select`), `eval(&Expr, &dyn Bindings) -> f32`
  **total** (arg faltante lê `0.0`, nunca entra em pânico), a trait `Bindings { attr(name), param(name) }`,
  `Func` (`Sin`/`Cos`/`Abs`/`Sqrt`/`Floor`/`Fract`/`Min`/`Max`/`Mix`/`Noise`), e
  **`noise1(x)`** — value-noise determinístico em `[0,1)`, hash inteiro, semeado, **bit-idêntico
  CPU/GPU**. Também `Func::is_deterministic` (falso p/ `Sin`/`Cos`/`Sqrt`).
- **O parser VEX-lite existe** mas é **`pub(crate)`** em `ph2d-node-motion-expression::parse`
  (o `motion.expression` dos Motion Nodes, OUTRO sistema). ⚠️ A descoberta que decide a
  escolha de parser: **ele já é agnóstico de semântica** — todo identificador vira
  `Expr::Attr(name)` e a resolução é DELEGADA à `Bindings` do consumidor. Ele não hard-codeia
  nada do domínio de nós.

## A decisão

**Uma expressão de timeline é um texto por-binding, parseado ao IR congelado `ph2d-expr`
por um parser COMPARTILHADO, e avaliado num passe pós-composição SEPARADO que lê as
propriedades JÁ COMPOSTAS e escreve as propriedades DIRIGIDAS — nunca tocando `stack_eval`
nem o blend.**

Sete sub-decisões:

### 1. IR congelado, NUNCA um interpretador geral

A fórmula parseia ao `ph2d-expr::Expr` e só isso roda. **Não há `eval` de JS/Python/Lua**,
nem reflexão, nem acesso a arquivo — o único que uma expressão pode LER são nomes declarados,
resolvidos pela `Bindings` do timeline (tempo, o valor composto de outra prop, o seed do
wiggle). É a lição paga de graça (ver "Alternativas rejeitadas" §A). `wiggle` **não vira um
`Func` novo** (§4): o contrato do `ph2d-expr` fica intocado.

### 2. Passe pós-composição SEPARADO — a espinha que protege o fade

O passe roda em `apply.rs` **DEPOIS** que `apply_active_clip`/`apply_from_doc` compôs todas
as props keyadas no mundo. Ele:

1. Lê os valores JÁ COMPOSTOS via `read_prop(world, entity, prop)` (apply.rs:260) — os mesmos
   que o stack acabou de escrever.
2. Avalia cada expressão contra esses valores + o relógio.
3. Escreve as props dirigidas via `write_prop` (apply.rs:409).

**Nunca chama `stack_eval`, nunca entra no blend, nunca vira input de crossfade.** Um
documento SEM nenhuma expressão **não executa o passe** (early-out) e é **byte-idêntico** ao
de hoje. Isto é o que mantém o `fade_fingerprint` verde por construção, e vira um arch-gate
(§Gates).

### 3. O texto mora no `TargetBinding` (document-wide), append por postcard

`TargetBinding` ganha `expr: Option<String>` (apêndice posicional ⇒ **`DOC_VERSION 14→15`**,
v14 recusado no load — a política de todo bump deste documento desde o ADR-0133). Escolha do
`TargetBinding` e não do `Track`/clip porque **o binding já É "a prop desta entidade é
dirigida"** — é document-wide como o binding, mora FORA do clip/stack/track, e uma prop pode
ser expression-driven **sem ter track nenhuma** (`bind` cria o binding sem criar track). Expr
por-clip é um follow-up nomeado (§Preço), não v1.

### 4. `wiggle` LOWERA para `Noise + time + seed` no parser (zero mudança de IR)

`wiggle(freq, amp)` não é função do IR; o parser o reescreve para
`amp * 2 * (noise((time + seed) * freq) - 0.5)`, usando o `Func::Noise` que já existe. O
`seed` é `Attr("__seed")`, resolvido pela `Bindings` do timeline ao **hash estável do
`wire_id` do binding** — então dois `wiggle` em props diferentes divergem, e o mesmo binding
é **reproduzível** (o gate "wiggle determinístico por seed"). v1 é **single-octave** (o AE
tem octaves/amp_mult; nomeado como follow-up).

### 5. Parser COMPARTILHADO, extraído para `ph2d-expr::parse`

O parser sobe de `ph2d-node-motion-expression` (privado) para **`ph2d-expr::parse`** (a casa
natural: o IR mora ali, e o próprio doc-comment do parser já diz "texto → o IR congelado
`ph2d_expr::Expr`"). O Motion node passa a **delegar** (`ph2d_expr::parse`) — uma linha. É a
regra da **porta única**: dois parsers para o mesmo IR divergiriam em silêncio. O parser é
estendido com o lowering de `wiggle` (§4) e o lexer passa a aceitar `.` dentro de
identificadores (`Sprite.x` vira um `Attr("Sprite.x")` que a `Bindings` resolve).
⚠️ **Custo de isolamento nomeado:** isto toca a crate do Motion node (foundational editável
sob Modo L; `NodeOp`/`OpResolver`/`NodeManifest` **intactos** — o parser não é superfície de
contrato).

### 6. Ciclos: uma varredura Gauss-Jacobi sobre um SNAPSHOT do início do passe

O passe tira um **snapshot** dos valores no início e faz UMA varredura: cada expressão lê do
snapshot, escreve no mundo. Consequências:

- `A = B.x` onde B é **keyado** (não dirigido): lê o valor de B que o apply JÁ escreveu ⇒
  **sem lag**, exato. Este é o caso comum (linkar de tempo/wiggle/prop keyada).
- `A ↔ B` (ciclo): cada lado lê o valor do OUTRO do início do passe (= o valor do frame
  ANTERIOR para o que ainda não foi reescrito) ⇒ **não explode, nunca** (é o modelo
  AE/Blender de dependência circular: lê o último valor). Termina em O(nº de expressões).
- `A = f(B)`, `B = g(C)`, ambos **dirigidos** (cadeia driven→driven): 1-frame de lag por elo
  não-ordenado. Aceito em v1; a ordem topológica que o elimina é follow-up nomeado (§Preço).

Sem ordenação topológica em v1 **de propósito**: uma varredura sobre snapshot é
*provadamente terminante* e a escolha de menor risco para um subsistema que roda ao lado do
fade precioso.

### 7. Erro de parse = fallback para o valor keyado, visível

Texto malformado **não derruba nada**: o passe pula a prop (ela fica com o valor composto
pelos keyframes) e a UI mostra o erro no campo. Um `eval` já é total (arg faltante → 0.0);
o parser devolve `Result` e o erro é do editor, não do runtime.

## Alternativas rejeitadas (com o motivo, não "achamos pior")

**A. Interpretador geral (JavaScript / Python / Lua) — o caminho que os grandes ABANDONARAM.**
O AE usa JS e o Blender usa Python em drivers — e o Blender **abandonou o `eval` irrestrito**:
pydrivers viraram buraco de segurança, então "Auto Run Python Scripts" (`--enable-autoexec`)
é **OFF por default** e drivers correm num **namespace restrito** (só funções whitelisted).
Um interpretador geral traz três problemas que não queremos: **segurança** (um `.postcard` de
projeto passaria a carregar código executável), **portabilidade/determinismo** (um runtime JS
por plataforma) e **peso**. Nosso `ph2d-expr` é essa lição **pré-paga**: um IR pequeno e
congelado, só matemática + leituras declaradas, já determinístico. Rejeitado por ser um
custo que já sabemos que vira dívida.

**B. Um parser NOVO, timeline-local.** Evitaria tocar a crate do Motion node — mas deixaria
**dois parsers para o mesmo IR** (`ph2d-expr::Expr`), a exata "duas portas" que este projeto
combate; eles divergiriam em silêncio (o Motion aceitaria `mix` mas a timeline não, ou os
dois discordariam sobre precedência). Rejeitado: o parser do Motion **já é agnóstico** (emite
`Attr`, delega à `Bindings`), então extrair custa uma linha de delegação e paga o resto para
sempre.

**C. Avaliar DENTRO de `stack_eval` / como input do blend.** Seria a arquitetura "natural"
de um sistema greenfield — mas quebraria a **byte-identidade** (todo documento passaria pelo
caminho novo) e alcançaria o **fade** (o crossfade lê o que o stack compõe). Rejeitado pelo
requisito de isolamento — é a razão de este ADR existir antes do código.

**D. `wiggle` como `Func` novo no IR.** Mudaria o contrato do `ph2d-expr` (ADR-0033) e o
emissor WGSL, para uma função que **já se expressa** com `Noise + time + seed`. Rejeitado:
um lowering no parser (§4) entrega a mesma coisa com zero churn de contrato.

**E. Ordem topológica + detecção de ciclo em v1.** Elimina o 1-frame de lag em cadeias
driven→driven, mas é mais código (grafo de dependências, sort, fallback) rodando ao lado do
fade no primeiro corte. Rejeitado para v1 (a varredura-snapshot é mais simples e
provadamente terminante); nomeado como refinamento.

**F. `expr` por-track (por-clip).** Deixaria cada clip com sua fórmula (o precomp do AE). Mas
o binding é document-wide no nosso modelo (todo clip anima os mesmos objetos, só as keys
mudam — ADR-0115), então v1 põe a expr no binding. Per-clip é follow-up.

## O preço (explícito)

- **1-frame de lag** em cadeias driven→driven (§6) até a ordem topológica chegar.
- **`Sin`/`Cos`/`Sqrt` não são bit-determinísticos** (libm/GPU) — a apresentação é
  HR-5-exempt, mas `Func::is_deterministic` já os marca, e um dia um lowering de *gameplay*
  (Luau/fixed-point) terá de recusá-los. Nomeado, não resolvido aqui.
- **`DOC_VERSION 14→15`** recusa saves v14 (a política do documento; provisório até integrar).
- **wiggle single-octave** em v1 (o AE tem octaves/amp_mult).
- **Toca a crate do Motion node** (delegação do parser) — foundational sob Modo L, contrato
  de nós intacto.
- A expr por-binding **não distingue clips** (§F) até o follow-up.

## O que fica GATEADO (para ninguém re-litigar por prosa)

- **`documento sem expr é byte-idêntico`** + **`fade_fingerprint` intacto** (o passe faz
  early-out sem nenhuma expr).
- **arch-gate: o passe de expressão NUNCA chama `stack_eval`** (varredura do fonte, como o
  gate irmão do onion).
- **`time*10` = rampa** (o link ao tempo).
- **`wiggle(f,a)` determinístico por seed** (mesmo binding reproduz; bindings diferentes
  divergem) — apoiado no `noise1` já bit-determinístico.
- **link A→B** (B keyado ⇒ A segue sem lag).
- **ciclo A↔B lê o snapshot e NÃO explode** (termina, valor do frame anterior no elo).
- **parse inválido = fallback ao valor keyado** (não derruba o frame).
- **um só parser** (o Motion node delega a `ph2d_expr::parse` — gate de que a porta é única).

## Consequência para a próxima LLM

O passe é a única coisa nova no caminho quente do apply; ele é `if sem_expr { return }` no
topo. Quem for mexer no fade/stack **não precisa saber que expressões existem** — elas rodam
depois, sobre o resultado, e o arch-gate garante que continuem lá.
