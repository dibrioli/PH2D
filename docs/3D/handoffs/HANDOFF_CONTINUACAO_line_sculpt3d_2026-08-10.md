# HANDOFF DE CONTINUAÇÃO — `line/sculpt3d` (2026-08-10)

**Status:** FECHADO 2026-08-10 · no `main` em `470da287b` (o commit que trouxe este arquivo).

> **Para o agente que assume a linha.** Ele **supersede** o
> [`HANDOFF_CONTINUACAO_..._2026-08-06`](HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-06.md)
> como *"comece aqui"* — aquele descreve um mundo em que a W10.7 ainda não tinha
> integrado, e três dos itens que ele lista como abertos **já shipam**.
>
> ⚠️ **Isto não é o estado do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../../CLAUDE.md)**;
> este documento diz **onde a LINHA está** e **o que não está feito**, com o preço
> medido ao lado de cada item.

---

## 1. Antes de ler qualquer código

Rode a **FASE 0** do [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
— `cd` + `pwd` + `git branch --show-current`, **antes** de abrir um arquivo. A janela
abre na raiz (que está em `main`) e o mesmo path relativo existe nas duas árvores:
editar `crates/…` da raiz **compila e commita sem um único erro**, e ninguém descobre
até a integração.

As **regras permanentes da sessão (A–H)** vivem no
[`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) e
valem iguais para você. **Não estão copiadas aqui de propósito** — duas cópias da mesma
regra divergem.

---

## 2. A linha, hoje

| | |
|---|---|
| Worktree | `Worktrees/line-sculpt3d/` — **criada em 2026-08-10**, não é a antiga |
| Branch | `line/sculpt3d`, a partir de `main` **`76788440a`** |
| Commits próprios | **ZERO** — a linha está limpa, no ponto de partida |
| Tier | `workstation` (123 GiB, 32 cores, `mold`, RA full) ⇒ **Modo L** |

⚠️ **A linha anterior foi INTEGRADA e ENCERRADA** (branch e worktree removidas pelo
integrador em 2026-08-10). Esta é abertura nova — não há trabalho não-commitado a
resgatar, e **não** existe rebase pendente: `main` **é** a sua base.

⚠️ **A integração de ontem chegou por REBASE**, então o SHA da linha antiga
(`dff370122`) **não é ancestral do `main`**. Se você for procurar o trabalho da W14→W17
por SHA, não vai achar — ele está lá **por conteúdo** (conferido: `height_per_depth` 3×
em `crates/ph2d-sculpt3d/src/alpha_frame.rs`, `stencil_for|stencil_of` 5× em
`shells/desktop/src/sculpt3d_space.rs`, e o gate `the_stamp_does_not_swim_with_depth`).

⚠️ **`main` está `ahead 5` de `origin/main`** — a integração de ontem é **local e não
pushada**. Não pushe: ship é ordem explícita do Enio, feita pelo integrador
(CLAUDE.md §0.7).

---

## 3. O estado do módulo — leia nesta ordem

1. **[`CLAUDE.md §5`](../../../CLAUDE.md)**, a entrada `3D / SCULPT` — o estado vivo, e a
   única fonte que é atualizada.
2. **[`06.1-Waves-riscos-e-alvos.md`](../06-Plano/06.1-Waves-riscos-e-alvos.md)** — o
   roteiro. **W1..W18 estão ✅**, cada wave com o mecanismo e as medições ao lado. É o
   documento que responde *"o que vem agora"*.
3. Os dois MESTRE, para o **mecanismo** das últimas jornadas:
   [`MESTRE_2026-08-09`](HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-09.md) (W14→W17,
   o pill, as três rodadas do carimbo) e
   [`MESTRE_2026-08-08`](HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-08.md) (até a
   W10.7 — ⚠️ o de 09 **não copiou** o detalhe dele).
4. [`00-INDEX.md`](../00-INDEX.md) — o cofre, uma linha por nota.

**As seis crates do módulo:** `ph2d-mesh` · `ph2d-mesh-render` · `ph2d-sculpt3d` ·
`ph2d-sdf` · `ph2d-light` · `ph2d-panel-sculpt3d`. Mais a metade de shell, que é grande
(~40 arquivos `shells/desktop/src/sculpt3d_*.rs`): a navegação orbital e o ciclo de
frame moram no **shell** de propósito (ADR-0150) — é essa decisão que mantém o contrato
congelado `Tool=12` fora do caminho.

---

## 4. ⚠️ A lista aberta que o último handoff repetiu está DESATUALIZADA

O `MESTRE_2026-08-09` §10 fecha com *"o resto da lista aberta do módulo (import/export,
objeto misto, merge/isolate, marching cubes) não foi tocado por esta jornada"*. A frase
é verdadeira sobre **aquela jornada** e falsa como **lista de pendências** — ela foi
herdada do handoff da W4-W8 e não reconferida. Medido por grep hoje, no `main`:

| Item da lista velha | Medição de hoje |
|---|---|
| import / export | **FEITO** — `sculpt3d_import.rs` (W8.4, o corredor do `import_obj`) e `sculpt3d_export.rs` (W8.5, `Ctrl+Shift+E`, formato pela EXTENSÃO) |
| merge / isolate | **FEITO** — `SceneObjects::merge_visible` e `::toggle_isolate` (W8.8, cena `=13`) |
| objeto misto (O2) | **FEITO pela rota assada** — W8.6, smoke OK 2026-08-04 |
| marching cubes | **ABERTO** — o que shipou foi **Surface Nets**, e por escolha (um vértice por célula, valência 4 quase em toda parte: a topologia que subdivide bem) |

*Quem lê a lista velha reconstrói o que existe.* É o §0 do CLAUDE.md mordendo numa nota
em vez de num número: **quem move o fato reconfere a nota**.

---

## 5. O que está DE FATO aberto, com o preço ao lado

**Do último smoke (MESTRE 08-09 §10):**

* **O preview no barro recalcula por quadro de órbita** enquanto um carimbo está armado —
  **0,36–0,46 ms** a 13,7k vértices, **7,9–10,1 ms** a 426k. ⚠️ **É inerente, não é
  dívida:** um estêncil muda de lugar no barro a cada movimento da câmera e **nenhum
  vértice se move**, então um bake por-vértice é invalidado pelo quadro seguinte, por
  definição. Os nove procedurais **não** respondem à câmera, e há gate de CONTROLE
  afirmando isso.
* **K1/K2 do ADR-0150** seguem como o MESTRE de 08/08 os deixou. ⚠️ O K2 aponta para o
  **lugar errado** (manda migrar as normais, que são 1,66 dos 13,1 ms; o custo real é
  **DESCOBRIR A VIZINHANÇA**, 88% do refresh) — e o K1 só dispara com pincel cobrindo
  **19% da malha inteira**. Isto é decisão de PRODUTO, não conserto pendente.

**Do roteiro (06.1), nomeados com motivo:**

* **W9.3 — o COLAPSO** (a outra metade do dyntopo) está ⏳ **pendente de smoke**, sem
  rodada registrada. O código existe; falta o Enio olhar.
* **Marching cubes** (o *manifold*) — W7.
* **O remesh RECUSA com a pilha de multires montada**, e a recusa é nomeada no log. ⚠️
  A alternativa — achatar a pilha **em silêncio** — é destruir trabalho autorado sem
  dizer. Falta um verbo de **achatar explícito**, e ele é decisão de produto.
* **A resolução do remesh não é autorável** (o botão usa o default 150). Um slider é UI,
  e a tabela do §W7 é o que ele precisaria mostrar.
* **Fundir NÃO solda** — a costura é do remesh (`V`). Um *merge by distance* seria outra
  operação, com tolerância própria.
* **O campo do remesh não carrega cor, material nem a MÁSCARA.**

---

## 6. As armadilhas deste módulo (as que custam tempo)

* ⚠️ **Os gates de GPU são `#[ignore]` e precisam de adapter.** Sem ele fazem *skip
  gracioso*, **que não é verde**. Rode-os: `cargo test -p ph2d-mesh-render --release -- --ignored`.
  A última medição na árvore combinada deu **54 verdes** (o MESTRE 08-09 dizia 45 — o
  número dele contava menos alvos). **Meça, não cite.**
* ⚠️ **Rode a suíte em DEBUG também.** Precedente do repo: o `ph2d-flip-colorize` panicava
  só em debug (um `wrapping_sub`), e a nota sobreviveu ao fato por três integrações.
* ⚠️ **Cenas de smoke `1..25` estão TODAS usadas** ⇒ a **próxima livre é `26`**. O roteador
  é uma lista de comparações e **o primeiro vence**: duas cenas no mesmo número deixam a
  segunda inalcançável **em silêncio** (foi assim que a `line/Vector` perdeu a cena dos
  tokens).
* ⚠️ **Três cenas imprimem o número que as torna válidas** (arestas de beira · maior
  aresta · peças abertas). **Se a linha não aparecer, o resto do smoke não diz nada.**
  E **rode uma vez SEM a env var** — é a metade que prova a inércia do frame 2D.
* ⚠️ **Ids novos são `hash_node_id` (hash de string)** ⇒ ficam fora de todo gate de
  contagem; são cobertos pelo `node_id_collisions`. O painel já tem o **scrollbar id 840**.
* ⚠️ **A posição do pill SCULPT na topbar é load-bearing, não gosto:** os **sete
  primeiros** clusters são o grupo da ESQUERDA (o `split` do `paint_top_bar`); ele entra
  **depois do FLIP**, e um merge que o mova quebra o layout **sem nenhum gate reclamar**.
* ⚠️ **O `ph2d-i18n/src/lib.rs` foi PARTIDO** (as chaves `panel.sculpt3d.*` moram no irmão
  `sculpt3d.rs`, as `panel.vector.*` em `vector.rs`), e os irmãos são consultados **em
  CADEIA** antes do vazamento. **Um irmão novo entra nessa cadeia, nunca num segundo
  `match`** — ficar com um lado apaga a família inteira do outro painel, que passa a
  pintar os próprios identificadores na tela com a suíte verde.
* ⚠️ **Se você bumpar `PROJECT_SCHEMA`** (hoje **70**, tripla `(70, 13, 14)`): o valor se
  **CONTA** contra o `main` do dia, e a conferência é nos **DOIS** arquivos (`project.rs`
  **e** `project_schema_tests.rs`) — esta colisão passa **MUDA** quando duas linhas
  escrevem o mesmo literal, porque o git não sabe o que o número significa. E **escreva o
  degrau na escada** do `project.rs`: quem conta o próximo lê a escada, não o literal.
* ⚠️ **O registro do `ph2d-ecs` tem TRÊS casas** (o registro + os espelhos em
  `ph2d-render` e `ph2d-script`), cada uma rodando só na suíte da própria crate. Este
  módulo não o toca hoje; um componente novo move as três.

---

## 7. O que você NÃO faz

Fecha a wave, roda o gate batched (DIRETRIZ §6.6.A.2 + DIRETIVA §3-§5), escreve o
**handoff de integração** (DIRETRIZ §1.5.9, **nesta pasta**) e **PARA**. Você **não**
roda `foundational-integrate.sh`, **não** integra e **não** pusha — integração e ship são
do Enio, por ordem explícita, via agente integrador dedicado.

---

## 8. Ao começar a trabalhar

O primeiro output é a **TRIAGEM** (DIRETRIZ §2), e a cada passo da implementação a
[`DIRETIVA_IMPLEMENTACAO.md`](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) é
lida antes — ela é o antídoto das quatro causas conhecidas (costura não-testada ·
"audit" = compilar · isolamento órfão · alvo irrefutável). Inner loop = **só
`cargo check -p`**; teste, clippy e auditoria **uma vez**, no fechamento.
