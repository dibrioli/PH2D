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

## §0 — ANTES dos portões: o disco está doente? *(2026-08-22)*

```bash
bash scripts/btrfs-health.sh        # sem root, 1 s — exit 1 = vermelho
```

⚠️ **«Disco cheio» com 500 GB livres NÃO se cura apagando `target/`.** Medido em 2026-08-22: o btrfs
tinha 937,85 GiB dos 950 alocados a blocos de **dados** (410 usados), **zero** não-alocado, e a
**metadata** (6 GiB, 5,46 usados) sem ter de onde crescer — `ENOSPC` a meio do build, `.o` truncado,
`mold` em SIGBUS, e o `df` a dizer 45%. Apagar targets liberta espaço *dentro* dos blocos; só um
`btrfs balance` (root) devolve blocos ao não-alocado. O script mede isso, o swap (o target do
primário em tmpfs **vive no zram**) e a corrupção de checksum, e imprime o comando de cada cura —
runbook: [`docs/DevOps/BTRFS_METADATA_E_SWAP.md`](../DevOps/BTRFS_METADATA_E_SWAP.md). Se sair
vermelho em «não-alocado»/«metadata», **limpe na mesma** (os portões do §1 continuam a mandar), mas
o relatório do §5 leva as linhas ✗ **e diz ao Enio que a cura é o balance dele** — a limpeza sozinha
não devolve espaço à metadata. O bloco do §4 já corre o script **duas vezes**: antes de apagar e no
fim, e a segunda saída vai inteira no relatório (§5, item 5).

## §1 — Os 3 portões (TODOS passam, ou não apague)

1. **Ninguém está usando a worktree** — e isso são **duas** perguntas, nunca uma:
   - **(1a) alguém CONSTRÓI aqui?** `pgrep -x` sobre a lista **`$BUILDERS`**, que é declarada
     **uma vez só, no topo do §4**. ⚠️ **Não redigite a lista** — três variantes dela já
     conviveram neste doc, e a mais curta (`cargo rustc mold cc1`, sem `ld` nem `rustdoc`) era
     justamente a do bloco colável, isto é, a única que alguém de facto executa. Apagar o
     `target/` de um build em curso o quebra. Portão **global e duro** no modo padrão: um build
     ativo aborta a limpeza inteira. ⚠️ **Exceção nomeada — o MODO PARCIAL (§1-bis):** quando há
     agentes trabalhando e o Enio manda limpar mesmo assim, este portão vira **por-worktree**. É
     a única flexibilização permitida, e ela tem regras próprias.
   - **(1b) alguém EXECUTA de dentro dele?** Algum processo vivo cujo `/proc/<pid>/exe` esteja
     **dentro** do target. Portão **incondicional — modo padrão E modo parcial**, e está no
     script do §4. O (1a) enumera **construtores** e é cego a quem *roda*; o gesto que fecha toda
     wave do Modo L é o **smoke**, que não compila nada — ele executa o binário que mora dentro
     do `target/`. Medido em 2026-08-04: sem este portão o runbook teria apagado **197 GB debaixo
     de um smoke em andamento** (§1-bis).
2. **A worktree está limpa de FONTE.** `git status` sem `M`/`A`/`D` (ignore o `?? target`). Worktree
   suja = alguém deixou trabalho não-commitado ali → **pule essa worktree** e sinalize; não nuke o
   target quente de quem está no meio de algo.
3. **O alvo é um dir REAL sob `Worktrees/*/target`, não symlink.** O `target/` do **primário** é um
   symlink pra tmpfs (`/dev/shm`) — apagar *através* dele é outra coisa. Nunca `rm -rf` num symlink.

### ⚠️ As duas leis do INSTRUMENTO (pagas em 04/08 e 08/08, e simétricas)

Os portões medem o mundo; as **sondas** que os confirmam são instrumentos — e um instrumento
avariado não avisa que está avariado, ele apenas **responde**. Duas rodadas deste runbook
produziram as duas metades da mesma lei, e elas apontam em direções opostas de propósito:

1. **Quando um portão e a sua rede de segurança discordam do mundo na MESMA worktree, desconfie
   do instrumento antes de confiar no portão.** *(04/08: a única worktree que os três portões
   liberaram era a única com um processo vivo dentro do alvo — e a sonda de frescor, que existia
   exatamente para pegar isso, dizia «fria» por estar morta em silêncio.)*
2. **Um instrumento que só sabe dizer «quente» paralisa o runbook tão certamente quanto um que só
   sabia dizer «fria»** — e este segundo modo de falha é mais fácil de aceitar, porque parece
   prudência. *(08/08: a sonda recém-consertada respondeu QUENTE para as cinco worktrees porque a
   jornada terminara às 01:48 do mesmo dia; limpar era o veredito certo.)*

⛔ **Corolário operacional:** quem decide é **o que os portões MEDEM sobre o mundo agora** — processo
vivo (1a/1b) e fonte não-commitada (2) —, **nunca a idade de um arquivo**. E um runbook de segurança
se valida **EXERCITANDO-o**, não revisando-o no olho: os quatro defeitos que este doc já corrigiu
(o `grep -vq` que pulava worktrees limpas, a mtime de diretório que mente, o portão cego a quem
executa, a sonda morta em silêncio) apareceram **todos** ao rodar, **nenhum** ao ler.

### ⚠️ A terceira lei, de outra espécie: **um laço que nunca itera é um portão que sempre passa** *(2026-08-19)*

As duas leis acima são sobre uma sonda que **lê errado**. Esta é sobre um portão que **nunca chega
a perguntar** — e por isso é mais silenciosa que ambas: não há resposta errada a inspecionar, há
uma pergunta que não foi feita.

> **Medido ao rodar este runbook em 2026-08-19:** o portão 1a — o portão **duro e global**, o que
> aborta a limpeza inteira se alguém estiver a construir — executou **uma** iteração em vez de seis,
> com `p` igual à string inteira `cargo rustc mold cc1 ld rustdoc` (31 caracteres). O `pgrep -x`
> recusa padrões acima de 15 caracteres, escreveu um aviso em **stderr** e devolveu **zero
> correspondências**. ⇒ **O portão 1a passava sempre, e passava por avaria.**

**O mecanismo, e ele generaliza:** `BUILDERS="a b c"` + `for p in $BUILDERS` é idioma **bash**, que
faz *word splitting* em expansão não-citada. O bloco do §4 é, por desenho, **colado no shell** — e o
shell desta máquina é **zsh**, que **não** faz word splitting (é a diferença clássica entre os dois).
Um script com shebang `#!/usr/bin/env bash` corre sob bash e está a salvo; **o bloco colável não**.
*O defeito mora exatamente no único formato que alguém de facto executa* — a mesma família do achado
anterior deste doc, em que a variante mais fraca da lista era a colável.

**A cura tem duas metades, e a segunda é a que impede a reincidência:**

1. **`BUILDERS=(cargo rustc mold cc1 ld rustdoc)`** — um array expande citado (`"${BUILDERS[@]}"`) e é
   imune ao IFS **e** ao shell.
2. **Um CONTROLE POSITIVO no topo do §4:** o script pergunta ao `pgrep` se ele consegue ver **o
   próprio shell que o executa**, e aborta se não conseguir. Um portão que enumera não pode provar a
   sua própria negativa; só um controle positivo distingue *"ninguém está a construir"* de
   *"eu não consigo ver ninguém"*.

⚠️ **E a limpeza daquele dia foi segura por ACIDENTE de método, não pelo portão:** os construtores
tinham sido medidos **num comando à parte**, um turno antes, e por isso o veredito estava certo. *Um
resultado correto obtido com um instrumento morto não valida o instrumento* — é a mesma frase do §1.1,
vista do outro lado.

*A narrativa jornada-a-jornada que produziu estas leis está arquivada em
[`docs/archive/processo-2026-08-18/DIRETIVA_FIM_DE_DIA.md`](../archive/processo-2026-08-18/DIRETIVA_FIM_DE_DIA.md).*

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
#     $BUILDERS vem do topo do §4 — FONTE ÚNICA, nunca uma cópia desta lista.
#     ⚠️ É ARRAY e expande CITADO — `for p in $BUILDERS` morre em zsh (§4).
for p in "${BUILDERS[@]}"; do
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

A lista `$BUILDERS` (§4) responde *"alguém está compilando aqui?"*. Ela **não**
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

O `target/` do **primário** nunca entra nesta limpeza — é o checkout de dev ativo. ⚠️ Se ele ainda
for um symlink para tmpfs (`/dev/shm`, o desenho **retirado** em 2026-08-22: 33 GB lá viravam 30 GB
de zram e swap a 100%), a ação certa não é limpar, é migrá-lo para disco com
`bash scripts/target-to-disk.sh` (preserva o build; portões próprios) — §0 e o runbook.


---

## §2-bis — DE QUE o target é feito, e as duas regras que atacam o PICO *(emenda 2026-08-16)*

> **A pergunta do Enio, no fim da jornada de cinco linhas:** *"por que o HD enche tão rápido? Quase
> 1 TB num dia de trabalho no target."* A limpeza desta diretiva ataca o **depois**; estas duas regras
> atacam o **durante**, que é onde o pico mora.

### A decomposição, MEDIDA (não estimada)

Amostra: o `target/` do primário no fim da jornada de 2026-08-16, **46 GB** em dois perfis
(`debug` + `ci-test`). ⚠️ É o primário, não uma worktree em jornada — os targets das worktrees já
tinham sido limpos quando a pergunta foi feita, e o tmpfs do primário **evaporou no reboot** logo
depois, então esta amostra não é re-medível sem um build completo.

| Componente | Tamanho | Fração |
|---|---:|---|
| **`incremental/`** (14 GB `debug` + 11 GB `ci-test`) | **25 GB** | **54%** |
| 40 binários de teste (`--all-targets`, `debug = true`) | 8,3 GB | 18% — **208 MB cada** |
| `rlib`/`rmeta` | 3,3 GB | 7% — dos quais 2,6 GB de terceiros |
| resto (`.o`, `.d`, build scripts) | ~1,7 GB | 4% |

⚠️ **DUAS hipóteses intuitivas morreram na medição, e ficam escritas para ninguém as re-propor:**

- **"É lixo acumulado."** O `cargo` de facto **nunca** coleta `deps/` — cada rebuild com fingerprint
  novo escreve um arquivo novo e o antigo fica para sempre, e a assinatura é visível (`libseam` com
  **78** cópias, várias crates com 10-13). Mas medido, guardar só a cópia mais nova de cada artefato
  poupa **1,5 de 12,3 GB = 12%**. O mecanismo é real; o termo é pequeno.
- **"São os terceiros duplicados entre as worktrees."** Os `rlib`/`rmeta` de terceiros somam
  **2,6 GB** por target ⇒ **13 GB nas cinco linhas**, não os ~65 que a intuição sugere. O `sccache`
  já partilha a COMPILAÇÃO; o que não se partilha é o armazenamento, e ele é pequeno.

**O achado estrutural:** um build não é grande — ele é **~46 GB**. O que enche o disco é *cinco
worktrees manterem um build multi-perfil vivo ao mesmo tempo, sem nada reclamar até o fim do dia*.

### Regra 1 — `CARGO_INCREMENTAL=0` no gate de fechamento

O perfil `ci-test` existe para **uma** coisa: o `cargo nextest run --workspace --cargo-profile ci-test`
do fechamento e do `ship.sh`. Ele roda em **BATCH**, sobre a workspace inteira, uma ou duas vezes por
jornada. Compilação incremental existe para tornar a *próxima edição* barata — um gate que varre tudo
de uma vez não colhe quase nada dela e paga **11 GB**.

```bash
CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test
```

⚠️ **Não é o mesmo que desligar incremental no `Cargo.toml`**, e a diferença é o inner loop: o
`cargo check -p <crate>` da DIRETRIZ §2 é exactamente o caso em que incremental paga, e ele corre no
perfil `dev`. A regra é estreita de propósito — ela nomeia o perfil de BATCH e deixa o de EDIÇÃO em paz.

### Regra 2 — a linha reclama o próprio `incremental/` ao FECHAR

Depois do gate batched e do handoff (DIRETRIZ §1.5.9), antes de a linha parar:

```bash
rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental
```

**25 GB por worktree**, risco **zero** (o cargo recria), e **sem ship** — não é mudança de config.
Cinco linhas fechando assim tiram **~125 GB** do pico sem tocar em nada que alguém vá ler.

⚠️ **Reclamar no FIM, nunca desligar no COMEÇO:** durante a jornada o `incremental/` do `dev` é o que
faz o inner loop voar. O que ele não pode é sobreviver à linha que o criou.

### Regra 3 — `split-debuginfo = "unpacked"` no `[profile.dev]` — **2,5×, MEDIDO**

Cada um dos 40 binários de teste liga estaticamente a workspace inteira **com DWARF completo**, e por
isso pesa 208 MB. O `unpacked` mantém a informação de depuração nos `.o`/`.dwo` em vez de a **COPIAR**
para dentro de cada artefato que a consome.

**A/B sobre a mesma árvore** (`cargo build --tests -p ph2d-anim -p ph2d-timeline`, targets separados,
2026-08-16):

| | total | `deps/` | `.dwo` |
|---|---:|---:|---:|
| `off` (o que shipa hoje) | **11 GB** | 9,4 GB | 0 |
| `unpacked` | **4,4 GB** | 3,6 GB | 6.479 |

**60% a menos, 2,5×** — e os 6.479 `.dwo` **já estão contados** nos 4,4 GB.

⚠️ **O PREÇO foi medido, e é pequeno:** a seção **`.debug_line` SOBREVIVE no binário nos dois braços**
(conferido por `readelf -S`), e ela é a que dá `file:line` a um backtrace; a mensagem de pânico nem
depende de DWARF (vem do `core::panic::Location`, que é dado estático). O binário em si encolhe apenas
26% (12 → 8,9 MB) — *o ganho não está no binário, está em não duplicar o DWARF por toda a árvore*. O
que passa a exigir os `.dwo` ao lado é a depuração de nível `gdb`, e eles ficam onde o cargo os escreve.

⚠️ **A amostra é uma SUB-ÁRVORE, não a workspace** (duas crates e as suas dependências). A fração pode
diferir no todo; o que a medição estabelece é a ordem de grandeza e o sinal.

⚠️ **E a primeira tentativa de medir isto foi INCONCLUSIVA por defeito da FERRAMENTA, não do produto:**
`objdump -h` num `.rlib` não soma as seções de todos os membros (um `.rlib` é um archive `ar`), e
reportou **`0,0%` de DWARF** para dependências que certamente o carregam. *Um instrumento que responde
com confiança à pergunta errada é pior que nenhum* — a medição honesta foi o **A/B**, que constrói o
mesmo alvo dos dois modos e compara a pegada real.

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

# ══ FONTE ÚNICA da lista de construtores ═══════════════════════════════════
# O §1 e o §1-bis referem-se a ESTA variável. NUNCA redigite a lista noutro
# lugar: três variantes dela já conviveram neste doc e a mais curta era a
# colável — quem executava o runbook rodava o portão mais fraco dos três.
#
# ⚠️⚠️ É UM ARRAY, e isso é LOAD-BEARING (medido 2026-08-19, ver §1).
# `BUILDERS="a b c"` + `for p in $BUILDERS` é idioma BASH, e este bloco é
# COLADO no shell interativo — que aqui é **zsh**, e o zsh **não faz word
# splitting** em expansão não-citada. O laço corria UMA vez com o padrão
# inteiro (31 chars), o `pgrep -x` recusava-o por passar de 15 e devolvia
# zero — o portão 1a passava SEMPRE, por avaria. Um array é imune ao IFS e
# ao shell.
BUILDERS=(cargo rustc mold cc1 ld rustdoc)

# ⚠️ CONTROLE POSITIVO do instrumento — sem ele o portão 1a não pode afirmar
# NADA. Se o `pgrep -x` não consegue ver nem o processo que o executa, ele
# também não veria um `cargo`, e "ninguém constrói" seria silêncio, não
# medição. É a lei do §1 aplicada ao próprio script: *zero não é o mesmo que
# não-medido.*
_self=$(basename "$(readlink -f /proc/$$/exe 2>/dev/null)" 2>/dev/null)
if [ -z "$_self" ] || ! pgrep -x "$_self" >/dev/null 2>&1; then
  echo "✗ CONTROLE POSITIVO FALHOU: o pgrep não enxerga o próprio shell ('$_self')."
  echo "  O portão 1a seria cego. ABORTADO, nada apagado."
  exit 1
fi

# Portão 1a (duro, GLOBAL): build ativo aborta a limpeza INTEIRA.
for p in "${BUILDERS[@]}"; do
  pgrep -x "$p" >/dev/null && { echo "✗ '$p' rodando — ABORTADO, nada apagado."; exit 1; }
done

df -h / | awk 'NR==2{printf "disco antes: %s de %s (%s)\n",$3,$2,$5}'

# §0 — a saúde do disco ANTES de apagar (não-alocado · metadata · swap · checksum). Não
# aborta: limpar continua certo; mas «não-alocado»/«metadata» vermelhos dizem que o df
# MENTE e que a cura é o balance (Enio, root), não este rm -rf — o relatório §5 leva isto.
bash scripts/btrfs-health.sh || echo "! disco VERMELHO antes da limpeza — leia as linhas ✗ acima e o §0"

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

  # Portão 1b (INCONDICIONAL — vale no modo padrão tanto quanto no parcial):
  # alguém EXECUTA de dentro deste target? O portão 1a enumera CONSTRUTORES e é
  # cego a quem RODA, e o gesto que fecha toda wave é o smoke, que não compila
  # nada. Pergunta pelo `exe`, nunca por uma lista de nomes — senão a lista
  # apodrece no dia em que nascer o segundo binário. (§1-bis, medido 2026-08-04.)
  ocupado=""
  for pid in $(ls /proc 2>/dev/null | grep -E '^[0-9]+$'); do
    exe=$(readlink -f "/proc/$pid/exe" 2>/dev/null) || continue
    case "$exe" in "$t"/*) ocupado="pid=$pid ($exe)"; break;; esac
  done
  [ -n "$ocupado" ] && { echo "· $(basename "$wt"): APP RODANDO — $ocupado — PULADO (§1b)"; continue; }

  sz=$(du -sh "$t" 2>/dev/null | cut -f1)
  rm -rf "$t" && mkdir -p "$t" && chattr +C "$t" 2>/dev/null
  echo "✓ $(basename "$wt"): liberado $sz, target recriado nocow"
done

df -h / | awk 'NR==2{printf "disco depois: %s de %s (%s)\n",$3,$2,$5}'

# ══ A SAÚDE DO DISCO NO FIM DO DIA — obrigatória, e a saída vai INTEIRA no relatório ═══
# É a medição diária que o Enio pediu (2026-08-22): «não-alocado» e «metadata» dizem se
# o balance semanal está a dar conta; «checksum» é o A/B do kernel (0 no LTS = a hipótese
# kernel 7.2.0 aguenta; >0 reabre a RAM → memtest86+); «swap» apanha um tmpfs que alguém
# recriou. VERDE fecha o dia. Qualquer ✗ é para o Enio, não para consertar aqui.
echo
bash scripts/btrfs-health.sh; disco_rc=$?
echo "— saúde do disco: $( [ "$disco_rc" = 0 ] && echo VERDE || echo 'VERMELHO — cole as linhas ✗ no relatório e sinalize ao Enio' )"

# Reporte, NÃO aja: commits locais não-pushados.
echo "— não-pushado em HEAD: $(git rev-list --count @{u}..HEAD 2>/dev/null || echo '?') commit(s)"
```

O `~/.cache/sccache` **não aparece** no script — de propósito. Ele fica.

> **Com agentes vivos, este script aborta no portão 1a** (global). Para o **modo parcial**, a troca
> é **uma só**: apague o `for p in $BUILDERS … exit 1` do topo e ponha o teste **por-cwd** de
> [§1-bis](#1-bis--modo-parcial-limpar-com-agentes-vivos-emenda-2026-07-30) **dentro** do laço,
> logo acima do portão 1b. O portão **1b não muda** — ele já está no laço, e é o mesmo nos dois
> modos. A sonda `find -newermt` entra como **terceira confirmação** só no modo parcial (no modo
> padrão ela responde QUENTE para tudo, por construção — §1-bis).

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
5. **A saída INTEIRA de `bash scripts/btrfs-health.sh`** (as 4 linhas + o veredito, não um resumo).
   É o único item que o Enio lê **todo dia** *(pedido dele, 2026-08-22)*:
   - **VERDE** → o dia fechou.
   - ✗ em **«não-alocado»/«metadata»** → o `df` está a mentir e a limpeza acima não cura; a cura
     é `balance` (root) — diga-lhe isso em uma linha, com o comando que o próprio script imprime.
   - ✗ em **«checksum»** → **destaque no topo do relatório.** É o A/B do kernel: um `csum failed`
     no `linux-cachyos-lts` reaponta a suspeita para a RAM (memtest86+), e **nenhum balance pode
     correr nesse boot** (o timer já se recusa sozinho). Mecanismo e números:
     [`docs/DevOps/BTRFS_METADATA_E_SWAP.md`](../DevOps/BTRFS_METADATA_E_SWAP.md) §3.
   - ✗ em **«swap»/«target em tmpfs»** → alguém religou o `target/` em RAM (retirado em 22/08);
     `bash scripts/target-to-disk.sh` desfaz, com os portões dele.

---

## Notas

- **Idempotente:** rodar de novo não faz mal — dir já vazio, nada a apagar.
- **Reversível:** o único "custo" de limpar um `target/` é o próximo build ser frio — e, com o
  sccache quente + o nocow, frio já é rápido (78% de hit medido). Nenhuma outra ação é reversível
  com a mesma segurança, e é por isso que a limpeza se limita a `target/`.
- **Diagnóstico que originou esta diretiva:**
  [`project-memory/project_modo_l_speed_hole_worktree_targets_slow_path.md`](../../project-memory/project_modo_l_speed_hole_worktree_targets_slow_path.md)
  — a lentidão do Modo L não era RAM, era disco + recompilação 6×; o cache quente é o herói, não o vilão.
- **As 3 emendas narradas jornada-a-jornada (30/07 · 04/08 · 08/08)** foram arquivadas **verbatim** em
  [`docs/archive/processo-2026-08-18/DIRETIVA_FIM_DE_DIA.md`](../archive/processo-2026-08-18/DIRETIVA_FIM_DE_DIA.md).
  ⚠️ **O que elas ENSINARAM não foi arquivado:** as duas leis do instrumento subiram para o §1 e o
  portão 1b entrou no script do §4. Leia a narrativa para responder *"por que isto ficou assim?"* —
  nunca para decidir a próxima ação.
