# 04 — Registro de execução

> **Preencher DURANTE, não depois.** Uma medição que não foi anotada na hora vira memória, e memória
> vira palpite.
> Marque cada tarefa: `[ ]` por fazer · `[x]` feita · `[-]` **recusada com motivo** (o motivo é
> obrigatório, e vale tanto quanto um `[x]`).

## §1 — A fotografia do ANTES (tarefa T3)

| | valor |
|---|---|
| data | **2026-08-29 17:18 −03** |
| `rustc -V` | `1.95.0 (59807616e 2026-04-14)` · `cargo 1.95.0` |
| `git rev-parse HEAD` | `bb7e01ddc838f1d45deef4bdc4a2b3c70e656067` |
| branch de trabalho · rede | `chore/stack-upgrade-2026-08` · tag `stack-upgrade-base` |
| tier da máquina | `workstation` (123 GiB, 32 cores, mold) |
| `target/` no início → fim | **12 KB → 8,8 GB** |
| build + suíte `ci-test` (relógio de parede) | **3 min 50 s** ⚠️ **e este número NÃO é uma build fria** — ver abaixo |
| execução dos testes só (nextest) | 98,4 s |
| **testes: passaram / falharam / total** | **20 041 / 0 / 20 041** ✅ |
| **vermelhos pré-existentes, pelo nome** | **NENHUM.** A árvore parte 100% verde. |
| ignorados (`#[ignore]`, não correram) | **1 990** |
| `_antes_audit.txt` gravado? | ✅ 76 linhas |

### ⛔ O relógio de 3 min 50 s mede a coisa errada — e a causa muda o custo dos blocos C/D/E

O `target/` estava mesmo vazio, mas **o `~/.cargo/config.toml` global tem
`rustc-wrapper = "sccache"`**, e o cache dele vive em `~/.cache/sccache`, ou seja em `/home`
(`nvme1n1p2`) — **que não foi trocado**. A corrida devolveu:

```
Compile requests            3030
Compile requests executed   1311
Cache hits                  1302   (Rust 1003 · C/C++ 284 · Assembler 15)
Cache misses                   2
```

⇒ *`target/` vazio não é o mesmo que build fria.* A compilação foi quase toda servida do disco.

⚠️ **E isto corrige uma afirmação que este plano fez ao Enio:** *«o `target/` está vazio, então o
maior custo escondido da atualização — jogar fora a cache de build — já foi pago pela troca de
disco»*. **Falso.** O que a troca de disco apagou foi a metade barata (o `target/`, que o sccache
reconstrói em minutos); a metade cara — o **conteúdo** do sccache — sobreviveu, e é justamente ela
que os blocos **C**, **D** e **E** vão invalidar de verdade: `wgpu 29`, `vello 0.10`, `bevy_ecs 0.19`
e `rapier2d 0.35` **nunca estiveram naquele cache**, então serão faltas genuínas.

⇒ **Meça o relógio da primeira build de cada um desses três blocos** — é lá que o custo real aparece,
não aqui. O `3 min 50 s` serve como baseline apenas para comparar corridas **com o mesmo grafo de
dependências** (por exemplo, antes × depois do bloco A).

⚠️ **A 1.ª tentativa do T3 saiu 127 e não rodou nada:** o `/usr/bin/time -v` **não existe** nesta
máquina (o CachyOS não traz o GNU time). O comando externo ainda assim devolveu **0**, porque quem
fechava o pipe era um `tail` — a família «pipe mascara exit code» da memória, encontrada aqui pelo
`echo "exit=$?"` que estava no script. ⛔ *Uma fotografia do «antes» que não rodou é pior que
nenhuma: ela vira o baseline contra o qual tudo depois é comparado.* O relógio passou a ser de
parede, medido pelo próprio shell.

## §2 — Placar

| bloco | tarefas | feitas | estado | quem fechou | data |
|---|---:|---:|---|---|---|
| **T** — terreno | 5 | **5** | ✅ **fechado** | LLM | 2026-08-29 |
| **A** — Rust 1.98 | 11 | **11** | ✅ **fechado** | LLM | 2026-08-29 |
| **B** — 31 compatíveis | 4 | 0 | por fazer | | |
| **C** — GPU e texto | 22 | 0 | por fazer | | |
| **D** — bevy_ecs | 14 | 0 | por fazer | | |
| **E** — rapier2d | 14 | 0 | por fazer | | |
| **F** — a cauda | 19 | 0 | por fazer | | |
| **G** — fecho | 6 | 0 | por fazer | | |

### Bloco T, tarefa a tarefa

- `[x]` **T1** — `nodatacow` nos 7 `target/`. Confirmado por `lsattr -d`: os 7 mostram `C`.
  ⚠️ Feito com todos vazios (12 KB) — `chattr +C` só vale para arquivos criados **depois**, então
  esta janela não volta a existir sem apagar tudo de novo.
- `[x]` **T2** — o vermelho falso do `btrfs-health.sh`. A metadata passa a exigir **as duas
  metades** (folga < 1 GiB **E** não-alocado < 8 GiB). **Provado por mutação:** no disco real dá
  `VERDE`; com `unalloc=0` injetado volta a `✗ metadata livre 0.89 GiB … E não-alocado 0.00 GiB`.
  Ficheiro restaurado byte-a-byte depois da mutação.
- `[x]` **T3** — a fotografia. **20 041 / 20 041 verdes, zero vermelhos pré-existentes.** ⚠️ Ver o
  §1: o relógio dela não é de build fria (sccache), e a 1.ª tentativa não correu de todo.
- `[x]` **T4** — tag `stack-upgrade-base` + branch `chore/stack-upgrade-2026-08`.
- `[x]` **T5** — uma linha no §1 do `CLAUDE.md` apontando `scripts/stack-audit.sh --tetos`.
  ⚠️ Escrita como **pergunta que o agente faz** («dá para atualizar X?»), não como nome de
  ferramenta: medido em 2026-08-18, ponteiro não é adoção — o que faz um script viver é um passo
  que o **nomeia no caminho de quem executa**.

### Bloco A — fechado 2026-08-29

**Resultado:** `20 041 testes, 20 041 passaram, 0 falharam` — **idêntico à fotografia do antes**.
Clippy `-D warnings` verde sobre a workspace inteira, `fmt` limpo, e o `physics_ecs_c9` estável
entre duas corridas (`6ebe2cb6…`). ⭐ **Zero supressões novas em código**, verificado por um portão
explícito no diff.

**O que o 1.98 desenterrou — 373 pontos, nenhum deles estilo:**

| regra | n | por que importava |
|---|---:|---|
| ambiguidade de literal `f32` | 22 | ⚠️ **ia deixar de compilar** numa versão futura do Rust |
| `chunks_exact` → `as_chunks` | 136 | blocos de tamanho fixo em compilação, sem verificação de limites, em código de pixel |
| `manual_slice_fill` | 236 | um laço de 3 linhas que é um `.fill()` |
| `unneeded_wildcard_pattern` · `use` morto | 2 | — |

⛔⛔ **A REGRA `manual_slice_fill` DO CLIPPY 1.98 EMITE SUGESTÃO PARTIDA — e o `--fix` aplica-a.**
Ela colapsa `for slot in &mut EXPR { *slot = v }` para `&mut EXPR.fill(v);`, deixando o `&mut` do
laço para trás: o resultado é um `&mut ()` descartado que reprova `unnecessary_operation` **e**
`unused_must_use`. Medido: **236 de 236** — não é caso de borda, é a forma que ela emite sempre que
o laço percorre um `&mut expr` explícito.

⚠️ **A 1.ª resposta a isto foi um `#![allow(...)]` na crate, e estava ERRADA** (corrigido por ordem
do Enio no mesmo dia: *«não quero nada armengado»*). *A transformação é boa; só o caminho automático
até ela é que está partido.* A cura certa: deixar o `--fix` escrever a forma partida, **remover o
`&mut ` inicial** dos 236 (assert exato), e deixar o compilador julgar — onde tirar fosse errado,
não compila. Registado em [[feedback-perfection-no-deferrals]] §SUPRESSÃO.

**A migração `as_chunks` precisou de intervenção humana em 2 de 136.** O item passa de fatia
(`&[T]`) para array (`&[T; N]`); onde o código o usava como fatia, o compilador reprovou
(`ph2d-field-render/src/tests.rs`, `*px` → `**px`). ⚠️ É por isso que o clippy marca esta sugestão
como *pode-estar-incorreta* e o `--fix` a recusa — e por isso ela é o caso legítimo de script
mecânico **com o compilador como oráculo**.

⭐ **Duas dívidas de LOC foram EXTINTAS, não ajustadas.** A conversão faz o `fmt` partir cada linha
em três, e dois ficheiros que estavam **exactamente no seu tecto** passaram por cima
(`imageio-tiff` 905→919, `imageio-apng` 768→774). Em vez de subir a entrada, os testes saíram para
`src/tests.rs` (o padrão de 63 crates): os ficheiros caíram para **496** e **410**, sob o tecto
**simples** de 700, e as duas entradas da `FILE_OVERAGE_OK` foram **deletadas**. Contagem de testes
conferida antes/depois (12→12, 11→11) — mover testes de ficheiro é o gesto que os faz evaporar.

⚠️ **Uma reprovada foi flake de carga, não regressão:**
`a_round_live_offset_costs_like_the_other_joins` (o membro canónico do §5.0 do `CLAUDE.md`) — 3 de 3
passam sozinho.

---

## §3 — As decisões que este plano exige explicitamente

Cada uma **tem** de ser respondida — deixar acontecer por omissão é o defeito.

| # | decisão | tarefa | escolhido | por quê |
|---|---|---|---|---|
| 1 | `linker_messages`: corrigir ou silenciar? | A7 | **nenhum dos dois** | O `mold` é **mudo**: `0` mensagens numa build forçada do shell. Não há defeito a corrigir nem aviso a silenciar. ⚠️ A 1.ª verificação disto estava malformada (`grep … \| head … \|\| echo` — o `\|\|` nunca dispara depois de um `head`, que sai `0` sem entrada); refeita com `grep -c`. |
| 2 | adotar `build.warnings` no lugar do `RUSTFLAGS`? | A8 | **ADOPTADO** | **Medido** em `ph2d-color` a seguir a uma build normal: `CARGO_BUILD_WARNINGS=deny` = **105 ms** (reaproveita a cache) contra `RUSTFLAGS="-D warnings"` = **2 399 ms** (recompila — entra no fingerprint). **23×**, e o `RUSTFLAGS` aplicava-se à workspace inteira. Trocado no `spike.yml` **e** no `ship.sh` (paridade: sem isso o ship fica verde num aviso que o CI reprova). |
| 3 | contatos do rapier: aceitar o padrão novo ou fixar o antigo? | E5 | | |
| 4 | adormecer do rapier: aceitar 0,05 / 0,5 s? | E6 | | |
| 5 | teto de velocidade de 400 u/s: aceitar ou levantar? | E7 | | |
| 6 | `criterion` resistiu — subir ou adiar? | F11 | | |

## §4 — Os tetos, e quando reconferir

⚠️ **Um teto não é permanente — é uma data.** Quem sobe a dependência que segura tem de voltar aqui.

| teto | dono | cai quando | reconferido em |
|---|---|---|---|
| wgpu **29.0.4** | `vello 0.10` | o vello suportar wgpu 30 | |
| skrifa **0.44** | `vello`, `parley`, `usvg` | os três passarem de 0.44 | |
| accesskit **0.24.1** | `parley 0.11.1` | o parley passar de 0.24 | |
| pollster **0.4** | `rfd 0.17.2` | o rfd passar de 0.4 | |
| core-graphics **0.23.2** | `winit 0.30.13` | **o `winit 0.31` sair de beta** | |
| ndarray **0.15** | `deep_filter` vendorizado | trocarmos ou atualizarmos o vendor | |
| miniz_oxide *(2 cópias)* | `ctt`/`exr`/`png` vs `flate2` | convergirem | |
| thiserror *(2 cópias)* | `psd 0.3.5` | o psd passar para 2 | |

**Comando:** `bash scripts/stack-audit.sh --tetos`

## §5 — Depois: as 6 worktrees

As linhas ficaram num `main` anterior e o `target/` de cada uma está frio.

| linha | rebase feito? | primeira build (relógio) | notas |
|---|---|---|---|
| `line/3DModeling` | | | |
| `line/Sprite` | | | |
| `line/Vector` | | | |
| `line/components` | | | |
| `line/motion-value` | | | |
| `line/quadextract` | | | |

⛔ **Não rebase worktree alheia sem ordem do Enio** (§0.2 do `CLAUDE.md`).

## §6 — O que foi MEDIDO E REJEITADO nesta jornada

> ⚠️ Esta tabela é a mais valiosa do documento a longo prazo. Uma recusa medida é o que impede
> refazer trabalho já pago — e o §5.0 do `CLAUDE.md` conta **126** delas no repo.

| o quê | por quê não | medido em | onde está a medição |
|---|---|---|---|
| **wgpu 30** | `vello 0.10` pede `^29.0.3`; forçar dá duas cópias e o vello recusa o nosso `Device` | 2026-08-29 | `01_inventario.md` §3 |
| **rapier «migrou para glam»** | nenhuma versão **publicada** faz; é o `master` não lançado | 2026-08-29 | `01_inventario.md` §7 |
| | | | |

## §7 — Diário

| data | bloco/tarefa | o que aconteceu |
|---|---|---|
| 2026-08-29 | — | plano escrito; `scripts/stack-audit.sh` criado; nada executado |
