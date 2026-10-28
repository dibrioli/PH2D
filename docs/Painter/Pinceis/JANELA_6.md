# JANELA 6 — Missão Pincéis 2: os presets do Rebelle que ainda não temos (autónoma, até ao fim)

> Ordem do dono (28/10): «quando acabar as tarefas em andamento [Óleo Real W3b], volte automaticamente às tarefas dos
> presets que faltam … a nova janela deverá seguir sozinha até completar a tarefa de finalizar todos os presets
> Rebelle que ainda não temos.»

## Nascer em espera

Responda **«JANELA 6 pronta — aguardo o sinal»** e PARE. O sinal chega de duas formas (a que vier primeiro):
- um `<cross-session-message>` da janela W3b: `SINAL: JANELA 6, comece. …`;
- o ficheiro `docs/Painter/Pinceis/SINAL_JANELA_6.ok` (com o SHA) — se o dono te escrever «começa» ou «verifica o
  sinal», confira-o com `ls` e, se existir, comece.
Não faça nada antes do sinal (a W3b ainda escreve nos mesmos ficheiros do motor). Sem polling em loop.

## Ao receber o sinal — FASE 0

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
(tem de dar `line/PainterWatercolor`; `git status --short` limpo — se não, é da W3b: leia o handoff dela antes de tocar).
Leia, nesta ordem e só o necessário (grep/offset nos grandes): CLAUDE.md §0 e §2 · `project-memory/MEMORY.md` ·
`docs/Painter/49_realshader/HANDOFF_W3b.md` (o que a W3b deixou: leis novas do material/óleo, o que é «do traço» e o que
é «da tela») · este ficheiro · `00_MISSAO.md` (contrato: §1 autonomia, §3 o EXAME, §4 recursos, §6 cumprida) ·
`REGRAS_DE_CUSTO.md` · `ESTADO.md` · `01_CATALOGO.md` §3d (a fila de MOTOR medida) e §3e–§3g · `BRIEFING_FAMILIA.md`.

## A tarefa

Completar a biblioteca contra o Rebelle: **cada preset do catálogo (112 itens) que ainda não está aprovado** ou sai
aprovado pelo EXAME (§3 do contrato: prancha lado a lado com o Rebelle + revisão cega), ou sai do catálogo com o porquê
MEDIDO. Hoje: 26 aprovados, fábrica 52 (Wet 36, Digital 16); candidatos com receita em `EM_ESTUDO_W2` (Knife Smooth ·
Smooth 2 · Soft · Dry 2; Dabber 2 · 3 · Fan; Metallic Round; Metallic Splats; Oil Bristle) — confira os números no
`ESTADO.md` e no código (`the_factory_presets_are_counted`), não aqui.

1. **Censo primeiro (um `explorador`)**: a lista dos presets do Rebelle (bateria de fotos, `01_CATALOGO.md`) × a nossa
   fábrica × candidatos → a FILA ordenada por «pincéis destravados por unidade de trabalho». Escreva-a no `ESTADO.md`.
2. **O que trava é MOTOR** (J5, medido: depois da 2.ª ronda o rendimento era zero). Por isso a fila de motor do §3d
   (borda da mancha molhada · pontas cortadas a direito · traço de 200 px do filme · mistura em luz linear [decisão do
   dono — PERGUNTE-LHE só esta, numa linha, se ele estiver presente; se não, fica fora] · orla do secador · misturador
   com grão · azul sobre vermelho molhado · fio do arrasto · lâmina que gira · partículas/ondulação/mip das pontas) e
   os abertos do Óleo Real W3 (`ESTADO_MISSAO_W3.md` «Fila para a próxima onda») entram na fila ANTES dos pincéis que
   destravam. Cada lacuna de motor: um agente, padrão-ouro, neutro ao bit para os pincéis aprovados (gate), mutação
   vista vermelha, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` verde.
3. **Depois, pincel a pincel** pelo método da JANELA 3 (`BRIEFING_FAMILIA.md`): família por agente, o EXAME, receita
   no ficheiro de preset, prancha em `pranchas/`, linha no catálogo.
4. **Lei de hoje para controlos novos** (dono 27–28/10): controlo do PINCEL só vale para os traços seguintes (guardado
   por célula; gate `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` + `um_controlo_do_pincel_nao_recompoe_a_tela`);
   controlo da TELA no cartão (Paper, Real Oil: luz/sombra/ambiente). Todo knob novo entra nesses censos.
5. **A biblioteca como um todo** no fim (como a J5): matriz de distinção, pastas/nomes i18n, ordem, custo a 60 Hz,
   memória, revisão cega final por categoria.
6. **O fecho** (`/pd-linha-fechar`, DIRETRIZ §1.5.9): o gate batched 1× (`PH2D_SUITE=fecho` nextest-impacted + clippy
   `--all-targets` + fmt, pelo `verificador`) — ⛔ a suíte COMPLETA não (é do integrador); handoff de integração
   `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_<data>_PINCEIS2.md` (≤ 8 KB), a linha do Painter
   no CLAUDE.md §5 (≤ 700 B), `agent-loop-profile.sh`, `rm -rf target/*/incremental`.
7. **O smoke do dono**: binário `--profile smoke`; tutorial PDF em `docs/Painter/Pinceis/tutorial/` actualizado (uma
   página por categoria, foto nossa | Rebelle, passos numerados sem jargão — CLAUDE.md §0.8); a mensagem final curta.

## Regras da missão (inegociáveis)

- **Autonomia** (`00_MISSAO.md` §1): nunca `AskUserQuestion` salvo a pergunta única do item 2; nunca parar à espera;
  nenhuma forma de comando que dispare pedido de permissão (`cd X && …` → caminhos absolutos; `bash -c` → `.sh`;
  `rm -rf` com variável; `kill` de PID que não gravou; escrever em `.git/`/`.claude/`) — ponha esta linha no briefing
  de TODO agente.
- **Custo** (`REGRAS_DE_CUSTO.md`, memória `feedback_brush_mission_cost_budget`): ≤ 2 rondas por pincel; revisor cego
  `model: "sonnet"`, ≤ 15 linhas, um por ronda; ≤ 3 agentes em paralelo com ficheiros disjuntos; pesados ≤ 2 ao mesmo
  tempo por `scripts/ph2d-run.sh`, `--profile smoke`; teste POR NOME; clippy só no fecho; o Rebelle é UMA instância
  (UM agente de fotos de cada vez; nunca o mate — receita em `reference_rebelle_trial_installed_under_wine.md`).
- **Contexto pequeno**: esta janela DECIDE e INTEGRA; todo o mecânico vai para agentes (`explorador`, `verificador`,
  `mutacao`, `documentador`, `general-purpose`). Commit por item fechado (`git commit --no-verify -- <paths>`).
  Ao aviso de 300 mil tokens: actualize `ESTADO.md` + `HANDOFF_JANELA_6.md` e CONTINUE delegando mais.
- ⛔ Sem `git push`, sem integrar no `main`, sem tocar noutras worktrees, ⛔ NÃO desligar o PC (dono 27/10).
- `ESTADO.md` actualizado a cada item fechado (é o que sobrevive se a janela cair).

## Saída

A missão cumprida (`00_MISSAO.md` §6) ou, item a item, o porquê medido de cada um que saiu; `ESTADO.md` final
(«MISSÃO 2 CUMPRIDA» + SHAs); árvore limpa; nada nosso a correr; a mensagem ao dono com o comando do smoke.
