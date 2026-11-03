# 06 — Como a linha de implementação usa o que a linha de TESTE construiu

> Leitor: o agente que abre a linha REAL da UI em cartões a partir do `main`. Lei da UI: [05](05_ESPECIFICACAO_UI_COMPLETA.md).
> Este doc responde a outra pergunta: **o que fazer com o CÓDIGO** de `line/TrelloUITeste` (estudo de 02/11/2026, medido).

## 1 — O veredicto

**Portar o estado FINAL do código, por FAMÍLIA e em ordem de dependência. Não reescrever do zero, não fazer merge da branch e não fazer cherry-pick dos 89 commits.**

| alternativa | porque não |
|---|---|
| Reescrever a partir do 05 | Deita fora ~18 mil linhas que o dono aprovou no smoke, com testes de clique real e provas de mutação (T61 4/4, T63 7/7…). O 05 descreve a lei, mas o detalhe (custos medidos, ordem de registo no hit-index, cache dos retratos) só existe no código. |
| `git merge line/TrelloUITeste` | A branch nasceu de `4e648ec2b4`, um commit da linha do Kanban (TrelloClone) que **não está no `main`**. Os 34 commits de baixo são TrelloClone: 32 já entraram no `main` noutra forma, 2 não, e não são desta UI. O merge traria esse histórico duplicado e daria conflito em ~25 ficheiros. |
| cherry-pick dos 89 commits dos cartões | Muitos são cura-sobre-cura e há 1 revert. Cada um reaplicado contra os 202 commits que o `main` ganhou desde 29/10 repete o mesmo conflito várias vezes. |
| **diff final por família** (`git diff 4e648ec2b4 line/TrelloUITeste -- <paths> \| git apply -3`) | ✅ Uma resolução de conflito por ficheiro, contra o `main` de hoje, e família a família, com o teste dela verde antes de passar à seguinte. A proveniência fica no commit: «portado de `line/TrelloUITeste` `fb4d8ed237`, família X». |

## 2 — Medições (02/11, `main` = `3df5894489`)

- Commits dos cartões: de `1cd10968e4` (1.º, docs) até `fb4d8ed237` = **89**. Base de comparação: `4e648ec2b4` (o pai do 1.º).
- Tamanho: 309 ficheiros, +19 836 / −1 464; **sem `docs/`: 224 ficheiros, +17 891 / −1 464**; 72 ficheiros novos em `crates/` + `shells/`.
- Por crate (ficheiros): `ph2d-editor-core` 79 · `ph2d-panel-inspector` 33 · `shells/desktop` 24 · `ph2d-panel-registry-init` 18 (testes) · `ph2d-app-vec` 12 · `ph2d-panel-skeleton` 10 · `ph2d-vec-scene` 7 · `ph2d-app-kanban` 6 · `ph2d-tokens` 4 · `ph2d-i18n` 4 · `ph2d-card-stack` 4 · `ph2d-card-rope` 4 · `ph2d-tool-vector`/`ph2d-panel-vector`/`ph2d-node-motion-verlet-rope` 3 · outros ≤ 2.
- **Crates NOVAS** (não existem no `main`): `ph2d-card-stack` (a mola, o vão e as métricas do cartão) e `ph2d-card-rope` (a corda do alfinete).
- **Conflitos medidos** (`git merge-tree --merge-base=4e648ec2b4 main line/TrelloUITeste`): 18 ficheiros, a saber `Cargo.lock`; `ph2d-app-kanban/src/{metrics,paint}.rs`; `ph2d-app-vec/src/{smoke_bone,state}.rs`; `ph2d-editor-core/src/{icons,icons_tests}.rs`, `interaction/dispatch/key.rs`; `ph2d-panel-skeleton/src/paint.rs`; `ph2d-panel-vector/src/state_appearance.rs`; `ph2d-vec-scene/src/{lib,schema}.rs`; `shells/desktop/src/{project_schema,project_schema_tests,project_load_read,project_geometria_v194_tests}.rs`, `render_loop/{fase_selection_mirror_bone_focus,fase_skeleton_verbs}.rs`.
- **O esquema mexe:** `VEC_SCENE_SCHEMA_VERSION` 25 → 26 (os estilos de cor ligados, D30). Número que soma entre linhas **conta-se** (`python3 scripts/schema-recount.py`), nunca se copia o 26.
- **A shell CRESCEU nesta linha:** `shells/desktop` +583 / −434 = **+149 linhas**. Na linha real a catraca `the_shell_only_shrinks` reprova isto: o código de família vai para `ph2d-app-*`/crates de painel ([HOWTO](../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md)), e a shell só compõe.
- **Interruptor:** o modo cartões já nasce ligado com `PH2D_CARDS=0` a voltar ao Inspector de hoje (`ph2d-panel-inspector/src/cards.rs` `active()`, e o mesmo no Bones). Isto deixa portar sem mudar o app de quem desliga. **O que fica ligado por omissão no `main` é decisão do DONO**: pergunte numa linha.

## 3 — A ordem das famílias (cada uma: aplicar → `cargo-check-narrow` → testes dela por nome → commit)

1. **Docs**: `git checkout line/TrelloUITeste -- docs/CartoesUI` (o 05 manda; o 01 tem as decisões com a palavra do dono; o 03 é o mapa do código).
2. **Fundação nova**: `crates/ph2d-card-stack`, `crates/ph2d-card-rope` (entram no workspace e no `Cargo.lock` regenerado, nunca resolvido à mão); `ph2d-tokens` (tokens `Card*`/`Kanban*`, `derive::card_tone`); `ph2d-i18n` (`src/cartoes.rs`, só inglês). Crate nova = a suíte dela inteira (motivo imprescindível).
3. **Kanban com a mola extraída** (`ph2d-app-kanban` 6 ficheiros, D19): o `main` mudou o Kanban depois de 29/10 ⇒ **refaça a extracção contra o Kanban do `main`**, não aplique o diff às cegas. Smoke de controlo: `PH2D_KANBAN_SMOKE=1` igual a antes.
4. **Núcleo `ph2d-editor-core`**: `widget/card_ink.rs`, `widget/card_field.rs`, `panel/cards*.rs` (lista, corpo, solto, rola, estilos, rótulos, dicas), `interaction/state/card_*_ops.rs` (lista, solto, instâncias/face T63, alfinete, retrato/ensaio T61), `interaction/dispatch/*` (arrasto, menus), as portas que ramificam no corpo do cartão (`property_row` + `property_row/{escolha,marca}.rs`, `widget/dropdown`, `rows`), `icons`. ⚠️ `panel/cards.rs` tem 1 397 linhas: na linha real **parte-se por responsabilidade** antes de crescer (os tectos de LOC já reprovam ficheiros acima de 700).
5. **Bones (piloto)**: `ph2d-panel-skeleton` (`cards.rs`, `paint.rs`, `publica_presos`) + `ph2d-app-skeleton`/`ph2d-skeleton-live` (2 cada).
6. **Inspector em cartões**: `ph2d-panel-inspector` (`cards.rs`, `cards_componentes.rs`, `retrato.rs`, `sections/rows*.rs`, `event.rs` → `com_retrato`). ⚠️ `cards.rs` 791 linhas e a fn `cards` 305: partir.
7. **Estilos de cor do Vector** (D28–D32): `ph2d-vec-scene` (esquema, recontado), `ph2d-app-vec`, `ph2d-panel-vector`, `ph2d-tool-vector`; os testes de projecto da shell (`project_schema*`).
8. **A cola na shell** (`render_loop/snapshots_inspector.rs`, fases de retrato/alfinete): mova para uma crate o que for lógica; o saldo da shell tem de ser ≤ 0.
9. **Cenas de smoke** (`ph2d-app-vec/src/smoke_cartoes_inspector.rs`, `PH2D_VEC_CARTOES_SMOKE=1..7`, `PH2D_CARTAO_CORDA_SMOKE`): o número da próxima cena conta-se no roteador da família.
10. **Testes** (`ph2d-panel-registry-init/tests/it/o_*`, `os_*`, `toda_dica_dos_cartoes`, `a_lista_de_cartoes_flutua_no_hero`): entram com a família que testam (passo 4–8), não no fim.

Em cada passo use `git diff 4e648ec2b4 fb4d8ed237 -- <paths da família> > p && git apply -3 p`. Os conflitos resolvem-se por estágio do índice (`git checkout --theirs/--ours` só com a razão escrita) e, onde o `main` mudou o mesmo código, à mão com o teste da família como juiz. Delegue o mecânico ao `integrador`, e as baterias ao `verificador`.

## 4 — Réguas antes de começar

- **Os vermelhos já existentes medem-se no `main`, não nesta linha.** Aqui ficaram vermelhos que não eram dela (loc caps de `scroll_list.rs` 615, `panel/cards.rs` 1 419, `state/mod.rs` 726…, o ciclo `interaction ↔ object_mode`, `seam_*` do Inspector 134–139, `node_id_collisions::every_non_literal_hash_is_named`). Corra os mesmos gates no `main` ANTES do 1.º porte e escreva a lista; depois, cada família não pode acrescentar nenhum.
- **`CARGO_TARGET_DIR` próprio** para a worktree (partilhar troca os `.rlib`).
- **A foto «antes»** de uma cura tira-se com o patch reverso (`git diff > p; git apply -R p; foto; git apply p`), e a cena tem de mostrar o cartão aberto e acima da dobra (a foto não clica — semeie um solto em `.ph2d/sections.txt` com `FOTO_HOME_SEMENTE`).

## 5 — Depois do porte: o que se constrói de novo

O ABERTO do [05](05_ESPECIFICACAO_UI_COMPLETA.md) §9, a começar pelo **cartão Camera** (smoke do dono 02/11: sem título, mal organizado, cores fora do padrão, ponto de animação mal posicionado): é o retrato da família «chamadores que montam a linha à mão» (`property_label_row` 11, `Checkbox::new` direto 6, `paint_text` direto, popover das listas). Widget que ainda não existe segue a regra de referência do 05 §0: **nós (smoke OK) → Figma → Rive → Inkscape/Godot/Unity**.
