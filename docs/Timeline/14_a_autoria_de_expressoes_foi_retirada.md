# A AUTORIA de expressões foi RETIRADA — o MOTOR ficou

> **Ordem do Enio, 2026-07-30**, depois da avaliação medida de
> *"consegue retirar o feature de expressões sem nenhum prejuízo para a timeline?"*:
> *"faça como vc sugere. mas faça um trabalho bem feito. deixe as coisas organizadas e
> limpas, sem resíduos."*
>
> A sugestão foi cortar entre **o motor** e **a autoria**, não remover tudo. Este doc é o
> registro do que saiu, do que ficou, do porquê, e do caminho de volta.

---

## 1. Por que a autoria e não o motor

Medido antes de decidir, não depois:

| metade | estado | veredito |
|---|---|---|
| **o motor** — a expressão per-clip como **fonte de lane** (ADR-0152), o transform global (ADR-0144/0145) | fechado, gateado por fingerprint, smokado | **fica** |
| **a autoria** — o card, o catálogo de receitas, os 2 smokes de card | inacabada; o smoke de 29/07 nomeou o motivo | **sai** |

⛔ **O que decidiu:** a **folha é write-once**. O card não reconstrói linhas a partir do
texto (decisão declarada da crate: *"um reconhecedor de fragmentos canônicos começa a MENTIR
no dia em que alguém edita um caractere"*), então toda fórmula reaberta volta como uma linha
`Custom Formula` com texto cru — e o `Custom Formula` acabava sendo a receita mais usada do
catálogo sem ninguém a ter escolhido. Foi isso que o smoke do G1 mostrou
(*"todos aparecem como custom"*).

## 2. O que SAIU

**11.125 linhas, 42 arquivos**, em quatro blocos:

| bloco | o quê |
|---|---|
| **o catálogo** | a crate inteira `ph2d-expr-recipes` (leaf, dep-free, zero consumidores fora do card) |
| **o card** | `expr_modal{,_columns,_gallery,_paint,_preview}.rs` + os 6 testes dele |
| **a superfície foundational** | os 6 ids `EXPR_MODAL_*` + os 7 helpers `expr_*_id` (`ids/chrome/expr_modal.rs`) · a row **"Expression…"** das TRÊS tabelas de menu de track · os 2 `TimelineHitKind` (`ExprModalHandle`/`ExprModalScrim`) · as 7 chaves i18n `panel.timeline.expr*` |
| **o preview vivo** | `expr_live.rs` inteiro, o clock de wall-clock do shell, a instalação por-frame no `render_loop`, a supressão `previewed` do `stack_eval` e a guarda `is_previewing()` do undo |

Mais os 2 smokes de card (`PH2D_EXPR_SMOKE`, `PH2D_EXPR_GROUP_SMOKE`) e os 3 gates que os
vigiavam.

⚠️ **E três resíduos que só apareceram na varredura**, todos *write-only* depois do corte:

- **`TimelineViewSnapshot::selected_entity`** — o shell o preenchia todo frame e **ninguém o
  lia** (o card era o único leitor). Saiu com o arch-gate que o vigiava.
- **`TimelineWheel::anchor_y`** — existia para a guarda de moldura do card recusar a roda
  dentro dela; sem leitor. Saiu do tipo, do `add_timeline_wheel` e do `dispatch/scroll`.
- **doc-comments que passaram a MENTIR** — `seed_of_target`, `frame_solve::any_formula`,
  `snapshot::object_names` e o `view.rs` citavam o card como razão de existirem. Reescritos
  com a razão verdadeira — nenhum foi apagado deixando o vão.

## 3. O que FICOU, e por quê

### 3.1 O documento — `DOC_VERSION` **16**, intocado

⚠️ **Remover a FEATURE não é remover o SCHEMA.** `TargetBinding.expr` (v15) e
`NamedClip.expr` (v16) **continuam serializados**. Postcard é posicional: apagá-los moveria
o layout ⇒ bump ⇒ **todo projeto já salvo recusado no load**. Jogar fora trabalho real do
Enio para deletar dois campos é o trade errado — a mesma aritmética que manteve o
`PROJECT_SCHEMA` parado sete vezes na linha de física.

`PROJECT_SCHEMA` fica em **37**.

### 3.2 O motor inteiro

- **A expressão per-clip é fonte de lane** (`stack_eval::clip_anim_source` →
  `eval_frame`/`solo_source_value`): ela **fadeia com o strip, cruza e soma no aditivo**.
- **O transform global** (`expr_pass`), aplicado a FULL onde a composição cobre.
- **A porta de autoria** — `TimelineIntent::SetBindingExpr` → `TimelineDoc::set_clip_expr` —
  segue viva. É a API; o que saiu foi a UI que a chamava.
- **`ph2d-expr` (FROZEN, ADR-0039) e `ph2d-expr-parse` NÃO são da timeline**: a
  `motion.expression` dos Motion Nodes **delega** ao parser único (gate
  `the_motion_node_delegates_to_the_one_parser`). Matá-los quebraria o Motion.

### 3.3 O livro-razão das poses devidas — `expr_live` → **`expr_owed`**

⚠️ O módulo tinha **duas** metades e só uma era do card:

| metade | quem a produzia | destino |
|---|---|---|
| o **preview vivo** (`LiveExpr`/`set_live_expr`/`is_previewing`) | só o card | **saiu** |
| o **livro-razão** (`remember`/`drain_owed`/`has_pending_restore`/`forget_owed_poses`) | todo driver de fórmula | **ficou** |

A segunda é a metade que a auditoria de 29/07 (§4 D-I) provou ser a MAIOR — sem ela, uma
binding pelada dirigida por `value + 250` fica em **250 para sempre** depois de DELETE —, tem
gate próprio (`clearing_a_formula_hands_the_pose_back`) e é alcançável pela porta de autoria,
que continua viva.

**O arquivo mudou de nome porque um `expr_live` sem preview seria um nome que mente.**

### 3.4 Os gates e o smoke do motor

Sobreviveram intactos: `expressions.rs` · `expr_in_blend.rs` · `pure_expression_window.rs` ·
`clearing_a_formula_hands_the_pose_back.rs` · `no_expression_link_frame_alloc.rs` ·
`one_door_authors_an_expression.rs` · `the_noise_seed_is_stable.rs` — e o smoke do motor
**`PH2D_EXPR_BLEND_SMOKE`**, que prova a expressão fadeando com o strip.

## 4. A prova de que a timeline não sofreu

⚠️ *"Sem prejuízo"* aqui **não é opinião** — o instrumento já existia. `fade_fingerprint.rs`
e `fade_fingerprint_channels.rs` pinam o sistema de Clips/Strips/Containers/Fade num **hash
literal exato** (CPU-only, sem transcendental, sem FMA), e são o *"o sistema de fade não pode
ser afetado"* do Enio feito executável.

**Rodados antes e depois da cirurgia: verdes, o mesmo hash.** Qualquer coisa que tivesse
tocado o fade falharia ali.

## 5. O caminho de volta

Se a autoria voltar um dia, **não reconstrua o que morreu por um motivo**:

1. ⛔ **Não refaça a folha write-once.** O erro não foi o parser recusado — foi **jogar a
   folha fora**. Guardá-la (no estado do painel, chaveada por `target` = **zero schema**; ou
   no documento = `DOC_VERSION` + ADR) resolve sem reconhecedor nenhum.
2. O **documento já aceita** expressões (§3.1) e o **motor já as roda** (§3.2): uma UI nova
   escreve pela porta que já existe e não precisa tocar schema.
3. A pesquisa, o plano e a auditoria continuam em
   [`09`](09_pesquisa_editor_de_expressoes.md) · [`10`](10_plano_editor_de_expressoes.md) ·
   [`11`](11_HANDOFF_AUDITORIA_EXPRESSOES.md) · [`12`](12_plano_reescrita_expressoes.md) ·
   [`13`](13_RESULTADO_AUDITORIA_EXPRESSOES.md) — **históricos a partir desta data**, e o
   que eles medem sobre o catálogo (as fusões, as faixas, os defeitos D1..D14) segue válido.
4. O código saiu por remoção limpa: `git log` na branch tem o commit inteiro, então o card e
   as receitas são recuperáveis verbatim se alguém quiser partir deles.
