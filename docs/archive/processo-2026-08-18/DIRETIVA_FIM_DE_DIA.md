# ARQUIVO — DIRETIVA_FIM_DE_DIA.md (história, 32 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`DIRETIVA_FIM_DE_DIA.md`](../../IntegracaoMultiAgente/DIRETIVA_FIM_DE_DIA.md) em 2026-08-18, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-395` do original.

---

- **Emenda 2026-07-30 (§1-bis) — a 2ª vez que rodar o runbook o corrigiu.** Jornada com 8 worktrees e
  agentes vivos, disco a **85%** (798 G de 950 G). O portão 1 global teria abortado tudo por causa de
  um `cargo test` no `line-Painter`; o modo por-worktree liberou **464 GB** (→ 36%) limpando
  `line-anim` (221 G), `line-physics` (203 G) e `line-FLIP` (42 G), e pulando as três worktrees com
  trabalho não-commitado. Dois achados vieram de *exercitar*, não de revisar: a **mtime de diretório
  mente** (o `line-Painter` marcava 2 dias enquanto compilava) e a **corrida é real** (o `line-anim`
  começou um build 30 s depois da limpeza — sem dano, mas com rebuild frio a reportar). Mesma lição do
  achado do `grep -vq` acima: *um runbook de segurança tem de ser EXERCITADO*
  ([[feedback_render_and_look_when_a_green_gate_is_contradicted]]).
- **Emenda 2026-08-04 (§1-bis) — a 3ª vez, e a 1ª em que o runbook teria DESTRUÍDO trabalho.** Jornada
  com 8 worktrees e disco a **69%** (651 G de 950 G — sem pressão). **Nada foi apagado, e esse é o
  resultado certo:** das 5 worktrees com target não-vazio, 4 tinham fonte não-commitada (`line-physics`
  217 G/23 arquivos · `line-Painter` 42 G/4 · `line-motion-value` 31 G/5 · `line-sculpt3d` 23 G/8) e a
  5ª (`line-Vector`, 197 G) estava **limpa e passou os três portões** — com um smoke rodando de dentro
  do próprio target. Dois defeitos independentes, os dois achados **exercitando**, nunca revisando:
  o portão 1 **enumera construtores** e é cego a quem *executa*; e a sonda de frescor estava **morta em
  silêncio** (`bfs` recusa a data relativa → exit 1, stdout vazio → *"fria"* para todas, sempre).
  Sozinho, cada um é um buraco; **juntos, eles compõem exatamente o pior caso** — a única worktree que
  os portões liberaram foi a única com um processo vivo dentro do alvo, e o instrumento que existia
  para pegar isso respondeu *fria*. Corolário para a próxima emenda: **quando um portão e sua rede de
  segurança discordam do mundo na MESMA worktree, desconfie do instrumento antes de confiar no portão.**
- **Emenda 2026-08-08 (§1-bis) — a 4ª vez, e o modo de falha SIMÉTRICO ao da anterior.** Fim de dia com
  8 worktrees, disco a **72%** (672 G de 950 G), **nenhum** agente vivo. As cinco worktrees com target
  passaram os três portões (todas com `git status --porcelain` vazio; nenhum construtor, nenhum `exe`,
  FD ou mapa de memória dentro dos alvos) e a limpeza liberou **518 GB** (→ 17%): `line-motion-value`
  172 G · `line-Vector` 136 G · `line-physics` 105 G · `line-sculpt3d` 61 G · `line-Painter` 46 G. **A
  sonda de frescor — a rede de segurança que a emenda anterior acabara de consertar — disse QUENTE para
  as cinco**, porque a jornada terminara às 01:48 do mesmo dia. Onde 04/08 ensinou *desconfie do
  instrumento antes de confiar no portão*, esta rodada ensina o inverso exato: **um instrumento que só
  sabe dizer *quente* paralisa o runbook tão certamente quanto um que só sabia dizer *fria*** — e o
  segundo modo de falha é mais fácil de aceitar, porque parece prudência. Nos dois casos quem decide é o
  que os portões MEDEM sobre o mundo agora, nunca a idade de um arquivo.
