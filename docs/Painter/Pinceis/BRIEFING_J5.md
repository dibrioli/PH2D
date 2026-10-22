# BRIEFING comum da JANELA 5 (missão Pincéis, 2026-10-21) — vale para TODO agente da J5

> Leia por esta ordem: este ficheiro · [`REGRAS_DE_CUSTO.md`](REGRAS_DE_CUSTO.md) (ordem do dono: «muito gasto e
> pouco resultado») · [`BRIEFING_FAMILIA.md`](BRIEFING_FAMILIA.md) (o método, as formas proibidas, a bancada, o
> EXAME, o que devolver) · o relatório da SUA família em `exames/<família>.md` (grep/offset; não o leia inteiro se
> passar de 10 KB). Nada mais sem precisar.

## §0 — O que mudou da J4 para a J5

- **Custo manda.** No máximo DUAS rondas de cura por pincel; um pincel que já teve as duas só ganha mais UMA se a
  janela lhe der um recurso de motor NOVO feito para ele (diz-se no seu prompt). Revisor cego = UM por ronda,
  lançado pela ferramenta Agent com `model: "sonnet"`, resposta ≤ 15 linhas. No máximo 4 variantes por cura.
- **Dono por ficheiro.** O seu prompt diz que ficheiros são seus. Motor (`crates/*/src`) só o agente a quem o prompt
  o dá. Os outros: só valores de receita, presets e portões da sua família.
- **Formas PROIBIDAS** (param a missão num pedido de permissão): `cd X && …` (caminhos absolutos, `git -C`),
  `bash -c`/`sh -c` (escreva um `.sh` em `<scratchpad>/<família>/` e corra `bash <ficheiro>`), `rm -rf` com
  variável/glob (`rm -r -- /caminho/literal`), `kill` de PID que não gravou, escrever em `.git/`, `.claude/`,
  `~/.bashrc`. Nunca `git push`, `sudo`, outras worktrees, `AskUserQuestion`.
- **Pesados** pela vez (`ferramentas/vez_pesada.sh`, 2 vagas na linha); Rebelle/ArtRage/Krita pela
  `ferramentas/vez_do_app.sh`. ⚠️ Load alto (outros agentes): o custo `ms/quadro` medido agora não vale — a
  coordenação re-mede calma no fim; escreva o `load` ao lado e não repita por causa dele.
- **Veredito sem «aberto»**: aprovado · curado-não-aprovado (com a diferença que o revisor viu, escrita) ·
  reprovado/sai (com o porquê medido) · fundido com X. O relatório da família fica ≤ 10 KB (corte o histórico para
  o git: «o detalhe está em `<sha>`»).
- **Commits** `feat(Pincéis J5 <família>): …` + `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, por
  caminho, `--no-verify`, os novos no `add`. A cada pincel que fecha.
- **Devolva** ≤ 25 linhas: aprovados / curados-não-aprovados / saídos (uma linha cada, porquê), SHAs, motor que
  mudou (com o portão e a mutação), e a fila que sobra com a medição.
