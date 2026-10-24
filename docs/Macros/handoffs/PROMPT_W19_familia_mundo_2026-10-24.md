# PROMPT W19 — a família «mundo»: tudo pode ser macro (2026-10-24)

> Rota «linha reaberta» (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`):
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
> A W18 está FECHADA e o smoke `=7` foi **aprovado pelo dono (24/10)** — [handoff W18](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W18.md)
> (§2 o que existe, §4 o aberto). W17+W18 ainda NÃO integradas: integrar só por ordem do Enio.
> Protocolo: `/pd-feature` — **plano primeiro** (`docs/Macros/17_W19_plano.md`), produto final, UI incluída.
> Leia antes: [plano W18](../16_W18_plano.md) §2.3 e [ADR-0190](../../architecture/decisions/0190-a-macro-e-um-objeto-e-leva-tudo-o-que-usa.md).

## O pedido do dono (verbatim, 23/10 — decisão 7)

«a macro não será só imagem. Tudo na game engine pode criar macros. Por exemplo: posso começar com uma pintura de
um player e depois acrescentar um collider e depois simular a física. tudo pode ser macro e a macro pode conter
qualquer coisa. Logo a macro é um objeto completo e independente»

## O que já existe (W18 — não reconstruir)

- O objecto Macro (`Transform`+`Name`+`Macro`, Add ▸ Macro), a sessão que arma o objecto e abre a trilha de cada
  FAMÍLIA dentro dele com a ficha `Track::nasce` (`ph2d-app-macros/src/sessao.rs`); hoje só Painter e Motion,
  **uma de cada** por objecto Macro (recusa `one_per_family`: o palco do Painter toca a 1.ª trilha da família).
- O ficheiro `.ph2dmacro`, a biblioteca, aplicar novo/por cima (`aplicar.rs`), o undo como UM passo
  (`motion_objeto_atados`, `instala_nascido`), o reatar por `StableId` no undo (`ph2d_ecs::ReataPorIdentidade`).
- A captura do mundo é por objecto e por componente, chaveada pelo `StableId` (`WorldSnapshot`, `component_blob`,
  `set_component_blob`, `parent` por id) — a matéria natural da família «mundo».

## O que entregar

1. **A família «mundo»**: cada passo do undo DENTRO da sub-árvore do objecto Macro (componente posto/tirado/mudado,
   objecto nascido/apagado, pai) gravado como diferença de linhas do `WorldSnapshot`, ids locais à macro, e
   recursos referenciados (texturas por `AssetId`) EMBUTIDOS no ficheiro. Portas declaradas (`Recordability`):
   o que não grava recusa-se em voz alta.
2. **A reprodução** — o nó do plano, a MEDIR antes de escolher: um mundo-palco (o objecto que ainda não nasceu
   não existe no documento) vs. escrita conduzida pela porta `ph2d-preview-drive` (preview ≠ documento: nada no
   undo nem no save enquanto toca). Tabela de medição ao lado da escolha (§0 do CLAUDE.md).
3. **Simular a física como passo**: «Simular» gravado como um passo cuja reprodução corre o `rapier` determinístico
   (ADR-0131: runtime-truth) — não gravar poses por quadro.
4. **Várias imagens e vários Motion por objecto Macro** (tirar o `one_per_family`: palco do Painter por
   `(dono, trilha)`), porque «a macro pode conter qualquer coisa».
5. Cena de smoke `PH2D_MACRO_SMOKE=8` com o exemplo do dono: pintar um player, pôr um collider, simular a física
   — gravar, guardar, aplicar noutro projecto e ver tudo nascer e cair. Fotografada antes de ir ao dono.
6. Handoff no fim; integrar só por ordem do Enio.
