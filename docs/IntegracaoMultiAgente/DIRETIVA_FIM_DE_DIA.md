# DIRETIVA DE FIM DE DIA — arrumar e liberar disco COM SEGURANÇA TOTAL

> **O que é:** o runbook do agente que, ao fim de um dia de trabalho, libera o disco que a
> jornada Modo L encheu (`target/` das worktrees somam centenas de GB) **sem nunca arriscar
> fonte, git, ou o cache que deixa o dia seguinte rápido**.
>
> **A regra-mãe:** *só se apaga o que se regenera sozinho.* Um `target/` é 100% descartável — nunca
> contém fonte nem objetos git (a worktree os guarda no `.git`). Tudo o mais é **intocável**.
>
> **Refuse-by-default:** ao MENOR sinal inesperado (build rodando, worktree com trabalho
> não-commitado, um `target` que é symlink em vez de dir), **NÃO apague nada** — pare e reporte.
> Segurança parcial não é segurança.

---

## §1 — Os 3 portões (TODOS passam, ou não apague)

1. **Nenhum build rodando.** `pgrep cargo|rustc|mold|cc1|ld` vazio. Apagar o `target/` de um build
   em curso o quebra. Portão **global e duro** no modo padrão: um build ativo aborta a limpeza
   inteira. ⚠️ **Exceção nomeada — o MODO PARCIAL (§1-bis):** quando há agentes trabalhando e o Enio
   manda limpar mesmo assim, este portão vira **por-worktree**. É a única flexibilização permitida, e
   ela tem regras próprias.
2. **A worktree está limpa de FONTE.** `git status` sem `M`/`A`/`D` (ignore o `?? target`). Worktree
   suja = alguém deixou trabalho não-commitado ali → **pule essa worktree** e sinalize; não nuke o
   target quente de quem está no meio de algo.
3. **O alvo é um dir REAL sob `Worktrees/*/target`, não symlink.** O `target/` do **primário** é um
   symlink pra tmpfs (`/dev/shm`) — apagar *através* dele é outra coisa. Nunca `rm -rf` num symlink.

---

## §1-bis — MODO PARCIAL: limpar com agentes vivos *(emenda 2026-07-30)*

> **Quando:** o Enio manda limpar no meio de uma jornada — *"temos vários agentes trabalhando, limpe
> apenas o que não atrapalha"*. O modo padrão abortaria tudo por causa de UM build ativo, e no dia em
> que esta emenda nasceu isso teria deixado **464 GB** no chão com o disco a **85%**.

### A troca, e só ela

O portão 1 deixa de ser global e passa a ser **por-worktree**: para cada candidata, resolva o **`cwd`
de cada processo de build vivo** e pule a worktree se algum estiver dentro dela.

⚠️ **E pergunte também se alguém está EXECUTANDO de dentro do target** — ver "o portão que enumera
construtores" logo abaixo.

```bash
ativo=""
# (a) quem CONSTRÓI: cwd dentro da worktree.
for p in cargo rustc mold cc1 ld rustdoc; do
  for pid in $(pgrep -x "$p" 2>/dev/null); do
    cwd=$(readlink -f "/proc/$pid/cwd" 2>/dev/null) || continue
    case "$cwd" in "$wt"|"$wt"/*) ativo="build $p($pid)";; esac
  done
done
# (b) quem EXECUTA: o binário vivo mora dentro do target que eu ia apagar.
#     Genérico de propósito — pergunta pelo exe, nunca por uma lista de nomes.
for pid in $(ls /proc 2>/dev/null | grep -E '^[0-9]+$'); do
  exe=$(readlink -f "/proc/$pid/exe" 2>/dev/null) || continue
  case "$exe" in "$t"/*) ativo="APP RODANDO pid=$pid ($exe)";; esac
done
[ -n "$ativo" ] && { echo "· $(basename "$wt"): OCUPADA — $ativo — PULADO"; continue; }
```

### ⚠️ O portão que enumera CONSTRUTORES é cego a quem EXECUTA

A lista `cargo rustc mold cc1 ld rustdoc` responde *"alguém está compilando aqui?"*. Ela **não**
responde *"alguém está usando isto?"* — e o gesto que fecha toda wave do Modo L é o **smoke**, que
não compila nada: ele **roda o binário que mora dentro do `target/`**.

> **Medido em 2026-08-04:** `line-Vector` passou os **três** portões — sem `cargo`/`rustc`, worktree
> **limpa** (`git status --porcelain` vazio), target dir real — enquanto o pid 176476 executava
> `…/line-Vector/target/release/ph2d-host-desktop`. Seguir o runbook ao pé da letra teria apagado
> **197 GB debaixo de um smoke em andamento**, no exato momento em que aquela linha era julgada.

Um `rm -rf` sob um processo vivo não o mata na hora (o binário já está mapeado), e é isso que torna a
falha **pior**: o smoke segue rodando e falha adiante, ao carregar um asset ou ao re-executar — longe
da causa. A pergunta certa é sobre o **exe**, não sobre um nome, senão a lista apodrece no dia em que
nasce o segundo binário ([[feedback_a_condition_that_enumerates_its_readers_rots]]).

**Os portões 2 e 3 não mudam** — e o 2 (worktree suja) faz o trabalho pesado aqui: um agente no meio
de uma tarefa quase sempre tem alteração não-commitada, então *sujo* já é um bom detector de *ocupado*.

⚠️ **Re-cheque os portões IMEDIATAMENTE antes de cada `rm`**, dentro do laço — nunca uma vez no
começo. Entre a inspeção e a remoção há minutos, e agentes começam builds nesse vão.

### ⚠️ O sinal de atividade que MENTE

**A mtime do diretório `target/` NÃO é sinal de atividade.** Ela só muda quando entradas são criadas ou
removidas **diretamente** em `target/` — escrita em subpasta não a move.

> **Medido no dia da emenda:** `line-Painter` marcava mtime de **2 dias antes** enquanto um
> `cargo test --release` rodava **naquele instante**. Quem confiasse nela apagaria o target de um build
> em curso — exatamente o que o portão 1 existe para impedir.

O sinal que **funciona** é o arquivo mais recente *dentro* do target, e ele é barato por causa do
`-quit` (para no primeiro achado). ⚠️ **Data ABSOLUTA, e falha da sonda conta como QUENTE:**

```bash
ref=$(date -d '-24 hours' '+%Y-%m-%d %H:%M:%S')   # absoluta: nem todo `find` aceita relativa
if ! recente=$(find "$t" -type f -newermt "$ref" -print -quit 2>&1); then
  echo "· $(basename "$wt"): SONDA FALHOU ($recente) — tratando como QUENTE, PULADO"; continue
fi
[ -n "$recente" ] && { echo "· $(basename "$wt"): escrita <24h — PULADO"; continue; }
```

Vazio **com exit 0** = fria. Use-o como **terceira confirmação**, nunca como substituto dos portões.

⚠️ **A versão anterior desta sonda estava MORTA nesta máquina, e em silêncio.** O `find` aqui é
**`bfs` 4.1.1**, não o findutils da GNU, e ele **recusa** `-newermt '-24 hours'` (string relativa):
escreve o erro em **stderr** e devolve **exit 1 com stdout VAZIO**. Capturada por `$(...)` sem checar
o exit, a sonda respondia **"fria" para toda worktree, sempre** — inclusive, medido no mesmo minuto,
para duas que estavam **compilando naquele instante** (`line-motion-value` e `line-physics`).

> *Zero não é o mesmo que não-medido.* Uma "terceira confirmação" que só sabe dizer *fria* não
> confirma nada — ela **tranquiliza**, que é o pior modo de falha de um instrumento
> ([[feedback_a_silenced_instrument_reads_as_a_result]]). Por isso a sonda agora **falha alto** e a
> falha é interpretada do lado seguro (quente).

### ⚠️ E no MODO PADRÃO ela responde QUENTE para tudo — por construção

A sonda de 24 h pergunta *"alguém escreveu aqui hoje?"*, que é a pergunta do **modo parcial** (limpar no
meio de uma jornada). No **fim de dia** a resposta é **sim por construção**: a jornada que encheu os
targets terminou há poucas horas, então toda worktree candidata tem escrita recente.

> **Medido em 2026-08-08:** as **cinco** worktrees com target responderam QUENTE, e o arquivo mais
> recente de cada uma era da madrugada do mesmo dia (00:35 a 01:48) — com a limpeza rodando às 08:37,
> **zero** processo vivo e as cinco com `git status --porcelain` vazio. Limpar era o veredito certo, e a
> sonda dizia o contrário sobre todas.

Promovê-la a portão no modo padrão faria o fim-de-dia **nunca limpar nada**. Quem responde *"está
ocupada?"* são os portões **1** (processo vivo — construtor **e** executor) e **2** (fonte
não-commitada); a idade de um arquivo não é evidência de ocupação, é evidência de que a jornada
existiu. Antes de aceitar o veredito dela, **compare a mtime com o relógio**: madrugada do mesmo dia é
a assinatura de uma jornada que acabou, não de um agente trabalhando.

### O risco residual, que é real e aceito

Uma worktree limpa e fria **pode** receber um build segundos depois da remoção. O custo é o documentado
(build frio, reversível, com o sccache quente amortecendo) — **não há corrupção**, porque o `rm`
terminou antes do build começar.

> **Aconteceu em 2026-07-30:** o agente do `line-anim` iniciou um build ~30 s depois de o target dele
> ser limpo. Nada quebrou; aquela linha pagou um rebuild frio. **Reporte isso** — é perturbação real de
> um agente, e omiti-la faria o relatório mentir.

---

## §2 — O que LIMPAR (e só isto)

- **`Worktrees/<linha>/target`** de cada worktree que passou os 3 portões. É o ganho: as 6 linhas de
  uma jornada somam ~800 GB de artefato. Depois de limpar, **recrie o dir vazio com `chattr +C`**
  (nodatacow) — no btrfs isso tira os builds futuros do caminho CoW+zstd (artefato descartável não
  deve ser comprimido nem versionado por CoW), e o dia seguinte escreve mais rápido.

O `target/` do **primário** vive em tmpfs (RAM) e **evapora no reboot** — deixe-o. Só limpe se ele
tiver crescido demais **e** o primário estiver ocioso (é o checkout de dev ativo).

---

## §3 — O que NUNCA tocar (o "liberar memória" que SABOTA o dia seguinte)

| Nunca | Por quê |
|---|---|
| **`~/.cache/sccache`** | É o **cache quente** que serve os deps entre worktrees (hit de ~78% num target FRIO). Apagá-lo torna o 1º build de amanhã lento — é o oposto de arrumar. O instinto "limpar caches pra liberar memória" está **errado** aqui. |
| **Fonte, `.git`, trabalho não-commitado** | Óbvio, e é o que a §1.2 protege. |
| **`git clean -fdx` · `git reset --hard` · `git checkout .`** | "Limpar a árvore" com esses APAGA trabalho não-commitado em silêncio, e o gate "passa" ([[feedback_mutation_undo_with_cp_never_git_checkout]], [[feedback_destructive_git_outside_pasta]]). Para limpar artefato, use `rm -rf <target>` — cirúrgico, nunca um comando git destrutivo. |
| **`git worktree remove` / `git branch -d`** | Uma linha integrada segue viva pra próxima wave; removê-la sem **ordem explícita do Enio** perde a worktree e qualquer commit não-pushado dela. Limpar o `target/` já libera o disco **sem** encostar na linha. |
| **`git push` / `git commit`** | Fim-de-dia é arrumação, não entrega. Só por ordem explícita (CLAUDE.md §0.7). |

---

## §4 — O procedimento (guardado; seguro de colar)

Roda da **raiz do repo**. Não apaga nada até os portões passarem; pula worktree suja; reporta tudo.

```bash
cd "$(git rev-parse --show-toplevel)"
primary="$(git rev-parse --show-toplevel)"

# Portão 1 (duro): build ativo aborta a limpeza INTEIRA.
for p in cargo rustc mold cc1; do
  pgrep -x "$p" >/dev/null && { echo "✗ '$p' rodando — ABORTADO, nada apagado."; exit 1; }
done

df -h / | awk 'NR==2{printf "disco antes: %s de %s (%s)\n",$3,$2,$5}'

# Percorre SÓ as worktrees (pula o primário). --porcelain é a fonte da verdade.
git worktree list --porcelain | awk '/^worktree /{print $2}' | while read -r wt; do
  [ "$wt" = "$primary" ] && continue                              # pula primário (target em tmpfs)
  t="$wt/target"
  [ -e "$t" ] || continue
  [ -L "$t" ] && { echo "· $(basename "$wt"): target é symlink — PULADO (§1.3)"; continue; }
  # Portão 2: QUALQUER mudança não-commitada? (target é gitignored → não aparece aqui.)
  # Teste por string vazia, NÃO por exit-code de `grep -v` (que sobre entrada vazia mente).
  if [ -n "$(git -C "$wt" status --porcelain)" ]; then
    echo "· $(basename "$wt"): TRABALHO NÃO-COMMITADO — PULADO (§1.2)"; continue
  fi
  sz=$(du -sh "$t" 2>/dev/null | cut -f1)
  rm -rf "$t" && mkdir -p "$t" && chattr +C "$t" 2>/dev/null
  echo "✓ $(basename "$wt"): liberado $sz, target recriado nocow"
done

df -h / | awk 'NR==2{printf "disco depois: %s de %s (%s)\n",$3,$2,$5}'

# Reporte, NÃO aja: commits locais não-pushados.
echo "— não-pushado em HEAD: $(git rev-list --count @{u}..HEAD 2>/dev/null || echo '?') commit(s)"
```

O `~/.cache/sccache` **não aparece** no script — de propósito. Ele fica.

> **Com agentes vivos, este script aborta na 1ª linha** (portão 1 global). Use o **modo parcial**:
> troque o `for p in … exit 1` do topo pelo teste **por-worktree** de [§1-bis](#1-bis--modo-parcial-limpar-com-agentes-vivos-emenda-2026-07-30),
> movido para **dentro** do laço, e acrescente a sonda `find -newermt` como terceira confirmação.

> **Por que o teste é `[ -n "$(git status --porcelain)" ]` e não `grep -vq`:** `grep -v` sobre
> entrada vazia devolve exit-code ambíguo (depende do shell) — a 1ª versão desta diretiva usava
> `grep -vq` e pulava worktrees LIMPAS em silêncio. String vazia = limpa é inequívoco. *(Achado
> rodando o próprio script — [[feedback_render_and_look_when_a_green_gate_is_contradicted]]: um
> runbook de segurança tem de ser EXERCITADO, não revisado no olho.)*

---

## §5 — O relatório (o que o agente devolve)

1. **Disco antes → depois** (o número é a prova).
2. **Quais worktrees foram puladas e por quê** (build ativo / trabalho não-commitado / symlink).
3. **O que foi preservado**: `~/.cache/sccache` (o cache quente), o `target/` do primário (tmpfs),
   toda fonte e todo git.
4. **Sinalize, sem agir**: commits locais não-pushados, branches à frente do main. É informação pro
   Enio decidir — o fim-de-dia não pusha.

---

## Notas

- **Idempotente:** rodar de novo não faz mal — dir já vazio, nada a apagar.
- **Reversível:** o único "custo" de limpar um `target/` é o próximo build ser frio — e, com o
  sccache quente + o nocow, frio já é rápido (78% de hit medido). Nenhuma outra ação é reversível
  com a mesma segurança, e é por isso que a limpeza se limita a `target/`.
- **Diagnóstico que originou esta diretiva:**
  [`project-memory/project_modo_l_speed_hole_worktree_targets_slow_path.md`](../../project-memory/project_modo_l_speed_hole_worktree_targets_slow_path.md)
  — a lentidão do Modo L não era RAM, era disco + recompilação 6×; o cache quente é o herói, não o vilão.
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
