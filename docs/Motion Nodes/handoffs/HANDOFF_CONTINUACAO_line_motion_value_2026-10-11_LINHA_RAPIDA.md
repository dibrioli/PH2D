# CONTINUAÇÃO — `line/motion-value`, 1.ª onda: tornar a linha RÁPIDA (11/10)

## As regras (R1–R9, de `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` §2 — valem para toda janela da linha)

- **R1 — Janela curta.** Uma onda = uma janela. Passou de ~150 chamadas ou fechou a onda: commit,
  handoff de continuação ≤ 8 KB, PARE.
- **R2 — Edite pela ferramenta `Edit`.** Novo = `Write`; existente = `Edit`. Script só para a MESMA
  troca em N arquivos, com `assert` de contagem.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`;
  só então teste.
- **R4 — Teste dirigido por NOME no laço:** `bash scripts/cargo-test-narrow.sh <crate> <filtro>`. Suíte
  inteira e `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- **R5 — Nada de `until … sleep`.** Comando > 2 min: `run_in_background` e espere o aviso.
- **R6 — Chamadas independentes na MESMA mensagem.**
- **R7 — Doc > 40 KB nunca se lê inteiro:** `grep -n` + `Read` com `offset`/`limit`.
- **R8 — Escreva menos** (comentário 1–3 linhas; resposta ao Enio curta; prompt de subagente: objetivo,
  arquivos, formato da resposta).
- **R9 — Nenhum teste da suíte normal passa de 10 s em `dev`.** Cura: (a) `opt-level = 2` na crate
  quente (§2.1 do prompt); (b) fixture menor que mantém a lei, provada por mutação; (c) só o que não
  afirma nada (`diag_*`, `sonda_*`, `measure_*`) vai a `#[ignore]`. ⛔ Lei afirmada nunca a `#[ignore]`.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, HEAD `10a08f355` (+ o commit
  deste handoff). Base `8f20c60de`. **Nada integrado nem enviado.**
- **Ciclo 20 FECHADO** — handoff de integração:
  [`HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20.md`](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20.md)
  (o §9 dele tem os dois commits pós-fecho: a cena que não salta quando a rota cai da placa,
  `e30104881`, e o `Collide` automático, `10a08f355`). Smoke do `Collide` (`=122`) **APROVADO pelo
  dono** (11/10). Detalhe: doc 129 §5.10, doc 115 §33.
- Abertos nomeados: o §7 do doc 129 e o §9 do handoff de integração (o engasgo da troca de rota,
  `0,115 ms`/tique, sem readback do estado do dispositivo).

## A tarefa desta onda (o §3.Motion do prompt — confirme sobre ESTA base antes de agir)

Prompt completo (só no `main`, ainda não nesta worktree):
`/home/enio/Documentos/Projetos/PH2D/docs/IntegracaoMultiAgente/PROMPT_LINHAS_RAPIDAS_2026-10-11.md`
— leia §1, §2 e §3.Motion, não as outras secções.

1. `bash scripts/agent-loop-profile.sh 10` → os números de ANTES.
2. `cargo nextest run -p ph2d-app-motion --status-level pass` (via `bash scripts/ph2d-run.sh`, em
   fundo) → os 10 testes mais lentos.
3. `motion_tecto_de_instancias::nenhum_no_emite_mais_do_que_o_tecto`: 113 s de uma suíte de 122 s; com
   `profile.dev.opt-level=2` deu 12,6 s (9×) e remontar ficou igual. Descubra QUAL crate leva o tempo
   (`ph2d-app-motion`? os nós / `ph2d-eval-motion`?) e ligue só essa(s) pelo §2.1 do prompt (A/B
   intercalado com `CARGO_TARGET_DIR` próprio na scratchpad; bloco no FIM de `[profile.dev.package.*]`
   do `Cargo.toml` da raiz, com a tabela num comentário de 3 linhas). Depois o R9 sobre o resto:
   `pilha_demo` 11,7 s · `carimbo_demo` 11 s ·
   `nenhum_criador_passa_o_tecto_com_a_contagem_no_extremo`.
   ⚠️ Esta linha acrescentou gates de placa (`#[ignore]`, correm com `--include-ignored`) — não
   contam para o R9 da suíte normal.
4. Partir com `python3 scripts/doc-split.py`: `121_as_formas_na_placa.md` (248 KB),
   `BUGS_motion_nodes.md` (97 KB), `110_ciclo_6_valor_e_pulso.md` (75 KB) → ≤ 40 KB por pedaço + um
   índice. (O `129_ciclo_20_o_acabamento.md` está em 49 KB — parta-o também.)
5. Meça de novo; handoff com a tabela ANTES × DEPOIS e este bloco de regras no topo. **Sem push.**

Hábitos desta linha que mais pesam (medido): 98 `until/sleep`, 1 545 edições por script, 997
corridas de teste na janela principal.

## Depois desta onda

A fila do Motion continua pelo §5.1 do `CLAUDE.md` e pelos abertos acima. Integrar e enviar só por
ordem do Enio.
