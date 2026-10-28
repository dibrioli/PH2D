# MISSÃO AUTÓNOMA — line/TrelloClone, do W2b ao W11, SEM o dono

> Leitor: a LLM que abre a janela nova. Você é o **COORDENADOR**. O dono (Enio) **não está presente**
> e **não vai responder**. Ordem dele, 28/10: *«o agente principal coordena os outros agentes de
> modo a implementarem todo o módulo num loop de implementação sem a minha supervisão. Smoke será
> apenas no final de tudo.»*

## 0 — A lei desta missão (leia duas vezes)

1. **NÃO PARE até ao fim.** O fim é: todas as ondas W2b, W3…W11 de `docs/TrelloClone/01_plano.md §6`
   implementadas, com testes, fotografadas, e o smoke final escrito. Nenhuma mensagem sua ao dono
   antes disso. Nunca termine um turno com uma pergunta, um «quer que eu continue?», um plano à
   espera de aprovação ou um resumo intermédio — **cada turno acaba a lançar ou a receber trabalho.**
2. **Não há a quem perguntar.** Dúvida TÉCNICA → padrão-ouro, decida você (memória
   `feedback_architecture_decisions_are_delegated_to_the_gold_standard`). Dúvida de PRODUTO → na
   ordem: (a) as decisões do dono em `01_plano.md §8` e nas memórias `project_kanban_trello_clone_*`;
   (b) a maquete aprovada `docs/TrelloClone/maquete/`; (c) **o que o Trello faz** (§1 do plano).
   Registe cada decisão de produto tomada sem ele em `DECISOES_SEM_O_DONO.md` (§3) — o dono revê no
   smoke.
3. **Bloqueio NÃO é paragem.** Se um item exige contrato congelado (CLAUDE.md §6), dado que só o
   dono tem (ex.: o JSON real do Trello dele na W10), máquina externa (furo de NAT real na W5) ou
   rebase a conflitar fora dos seus ficheiros: faça o máximo honesto (fixture sintética fiel ao
   formato, loopback, base actual), registe em `BLOQUEIOS.md` com o que falta e **siga para o
   item seguinte**.
4. **Missão autónoma nunca pára num pedido de permissão** (memória
   `feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt`): ponha no briefing de
   cada agente as formas que os guardas recusam (§5) para eles não as tentarem.
5. **Proibido, sempre:** `git push`, integrar no `main`, `git add -A`/`.`, `git stash`, `spectacle`
   (fotografa o ecrã REAL do dono — use `import -window` no `kwin --virtual`), matar o PAI de um
   órfão (é o `systemd --user` = logout), suíte completa (é do integrador), clippy a meio da onda.

## 1 — Fase 0 (antes de ler qualquer coisa)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloClone && pwd && git branch --show-current
```
Tem de dar `line/TrelloClone`. Depois `git fetch` + rebase sobre o `main`; conflito fora dos seus
ficheiros → `git rebase --abort`, siga na base actual e registe em `BLOQUEIOS.md`.

Leia (só isto, e por esta ordem):
- `docs/TrelloClone/handoffs/HANDOFF_CONTINUACAO_W2_para_W3_2026-10-28.md` — o estado, as duas
  observações do dono e as pendências.
- `docs/TrelloClone/01_plano.md` §3, §6, §7, §8 — as ondas, os oráculos, as decisões do dono.
- ADR-0195 (o Kanban é um documento do projeto; aba e canvas são VISTAS).
- `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`.
- Memórias: `feedback_brush_mission_cost_budget`, `feedback_full_suite_only_before_integration`,
  `feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt`.

**Não leia código você mesmo.** Quem lê é o `explorador`; você recebe a conclusão com `ficheiro:linha`.

## 2 — A ordem do trabalho

| Onda | O quê | Nota |
|---|---|---|
| **W2b** | As duas observações do dono no smoke da W2: **(a)** título das listas MAIOR e mais destacado (MAIÚSCULAS na pintura, nunca no texto guardado; a largura de elisão mede-se com a MESMA fonte/peso/caixa que se pinta); **(b)** arrastar uma lista **não a faz sumir**: ela voa com inclinação e sombra de voo como o cartão, as vizinhas abrem o espaço com mola, os cartões dela vão junto | primeiro, sozinha — é o que o dono pediu |
| W3 | Verso do cartão completo (+ datas, capas de imagem, campos — adiados da W2) | widgets novos no `editor-core` com showcase |
| W4 | O mesmo Kanban no canvas do Quadro, vivo, com zoom (+ «Show on board») | |
| W6 | Filtro, pesquisa, modelos, espelhos, copiar/mover, envelhecimento, `E`, todos os atalhos | pode correr em paralelo com a W4 (crate própria `ph2d-kanban-filter`) |
| W5 | Ao vivo com colegas (+ Share/avatares no cabeçalho) | depois da W4; prove em loopback com dois processos |
| W7 | Vistas Tabela, Calendário, Linha do tempo, Painel | Painel: skill `dataviz` |
| W8 | Automação + `ph2d.kanban.*` no Luau | crate `ph2d-kanban-auto` |
| W9 | Calendário do projecto, Inbox, Planner, Os meus cartões, notificações | |
| W10 | Importar do Trello / exportar CSV e markdown | fixture sintética no formato real; o JSON do dono → `BLOQUEIOS.md` |
| W11 | «Power-Ups» do PH2D: cartão ↔ objecto/script/macro/trecho da timeline; nota adesiva ↔ cartão | |

Série só onde uma onda usa a anterior (W2b→W3→W4→W5); o resto o coordenador encaixa em paralelo.

## 3 — Memória da missão em DISCO (para sobreviver à compactação)

A janela vai compactar. A verdade vive em ficheiros, não na sua cabeça. Crie em
`docs/TrelloClone/loop/` e mantenha a cada passo:
- `ESTADO.md` — **≤ 4 KB**: onda actual, itens `[ ]/[~]/[x]`, agentes em voo (o que cada um faz, que
  caminhos possui), último commit verde. **Depois de cada compactação, a primeira coisa é relê-lo.**
- `DECISOES_SEM_O_DONO.md` — uma linha por decisão de produto: o quê, porquê, como desfazer.
- `BLOQUEIOS.md` — o que ficou por fazer e o que falta para destravar.
- `MEDICOES.md` — cada tabela do §0.0 (nenhum `MAX_*` sem a tabela ao lado).

## 4 — Como coordenar SEM encher a janela

- **Você decide e desenha; não implementa nem lê ficheiros grandes.** Cada onda: (1) um `explorador`
  devolve o mapa do que já existe (≤ 40 linhas); (2) você escreve o plano da onda em
  `docs/TrelloClone/loop/W<n>_plano.md` com a partição em lotes; (3) lança os implementadores;
  (4) um `verificador` confere; (5) um revisor independente; (6) fecha a onda.
- **Partição por CAMINHO, nunca por assunto.** Agentes em paralelo trabalham na MESMA worktree
  (worktrees à parte = um `target/` por agente; partilhar `CARGO_TARGET_DIR` corrompe os `.rlib`).
  Cada lote possui ficheiros/crates disjuntos e comita **só os seus caminhos**
  (`git commit --no-verify -m "…" -- <paths>`). Os ficheiros partilhados (registos em `lib.rs`,
  `ph2d-i18n`, `ph2d-tokens`/`tokens.json`, shell, `Cargo.toml` do workspace) são **seus**: o agente
  devolve o trecho a acrescentar e você aplica, ou um único agente os possui nessa onda.
- **≤ 3 agentes em paralelo** e **≤ 2 rondas de revisão por item** (dono 21/10: «muito gasto»).
  Revisor em `model: "sonnet"`; implementadores com o modelo por omissão.
- **Todo briefing de agente leva:** o caminho da worktree + `pwd`/branch; os caminhos que ele possui
  (e que nenhum outro toca); o critério de pronto (testes por nome que têm de passar, foto a tirar);
  as formas proibidas do §5; e **«devolva no máximo 25 linhas: o que fez, commits, o que ficou, onde
  — nada de diffs nem de logs»**.
- **Nunca leia o ficheiro de saída de um agente em fundo** (é o transcrito inteiro). Espere a
  notificação.
- Se o aviso de conversa grande (300 mil por passo) aparecer: delegue ainda mais — o trabalho
  continua; o `ESTADO.md` é o que garante a retomada.

## 5 — Regras de máquina e de guarda (copie para cada briefing)

- Máquina PARTILHADA: comando pesado só por `bash scripts/ph2d-run.sh <cmd>`; nada de vigia de fundo
  sem `timeout`; antes de acusar um processo de pendurado, leia `utime+stime` de `/proc/<pid>/stat`
  DUAS vezes; espere por PID, nunca `until ! pgrep -f`.
- Inner loop `bash scripts/cargo-check-narrow.sh <crate>`; teste só por nome
  `bash scripts/cargo-test-narrow.sh <crate> <nome>`; crate inteira só com
  `PH2D_SUITE="motivo: crate NOVA …"` (ou formato gravado, deps de `Cargo.toml`, código movido).
- Smoke e sondas em `--profile smoke`; `--release` só com `PH2D_RELEASE="mede velocidade"`.
- Editar com a ferramenta `Edit`; `sed -i` só com `grep -c` na mesma linha; script que grava com `assert`.
- UI canónica: zero hex, zero `f32` literal de UI, zero texto fixo — tokens `kanban-*` (contraste
  medido pelo gate) e chaves em `crates/ph2d-i18n/src/kanban.rs`.
- Código de família em `crates/ph2d-app-kanban` (ou drop-crate nova `ph2d-kanban-*`); a shell só
  compõe — nunca suba o número da catraca `the_shell_only_shrinks`, mova.
- Tetos de ficheiro (700 linhas `.rs`, 600 nos testes): parta antes de chegar lá.

## 6 — Fecho de CADA onda (o `/pd-onda-fechar`, sem a parte do dono)

1. `cargo fmt --all` · `PH2D_SUITE=onda bash scripts/ph2d-run.sh cargo clippy -p <crates da onda> --all-targets -- -D warnings`.
2. Testes da onda por nome (seam tests pelo hero, como os de `tests/it/`); nextest-impacted só se a
   cadência deixar (exit 3 = recusa esperada, não é ✗).
3. Cena nova no roteador `PH2D_KANBAN_SMOKE=<n>` (conte o número no `smoke.rs`, não escolha);
   **fotografe-a** (claro e escuro; ao lado da maquete quando a maquete cobre a peça) em
   `docs/TrelloClone/smoke/W<n>/`. Olhe a foto você mesmo: cena que ensina o contrário do que
   acontece é pior que nenhuma.
4. Medição do §0.0 onde a onda criou algo que escala (cartões no canvas, vistas, filtro, réplica).
5. Handoff `HANDOFF_CONTINUACAO_W<n>_para_W<m>` (≤ 15 KB) pelo `documentador`; `ESTADO.md`
   actualizado; commit por caminho. **E siga para a onda seguinte no mesmo turno.**

## 7 — O FIM (só aqui você fala com o dono)

1. Fecho da LINHA (`/pd-linha-fechar`) **sem a suíte completa** (é do integrador, dono 23/10):
   clippy `--all-targets` das crates tocadas, auditoria por lentes independentes, testes por nome,
   `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.
2. Handoff de INTEGRAÇÃO da linha (para o integrador: catraca da shell, recontagens, tokens novos,
   `PROTOCOL`/formato se mudaram) — **não integre, não faça push.**
3. **Smoke final ao dono**, em PT-BR, curto, sem jargão, em PASSOS NUMERADOS por onda: (1) o comando
   completo com o `cd`; (2) onde clicar, com o nome que aparece na tela; (3) o que tem de acontecer;
   (4) como saber que deu errado. Cada passo com a foto ao lado. Se ficar longo, faça um tutorial em
   PDF como os dos outros módulos e dê-lhe o caminho.
4. No fim da mensagem, três listas curtas: o que ficou de fora (`BLOQUEIOS.md`), as decisões que
   tomou por ele (`DECISOES_SEM_O_DONO.md`), e o que precisa do «sim» dele para integrar.
