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
| **B** — 31 compatíveis | 4 | **4** | ✅ **fechado** | LLM | 2026-08-29 |
| **C** — GPU e texto | 22 | **22** | ✅ **fechado** (4 eram inexistentes — §11) · falta o smoke do Enio | LLM | 2026-08-29 |
| **D** — bevy_ecs | 14 | **14** | ✅ **fechado** (9 eram irrelevantes — §10) | LLM | 2026-08-29 |
| **E** — rapier2d | 14 | **14** | ✅ **fechado** em 2 etapas (0.31 §13 · 0.35 §14) · falta o smoke do Enio | LLM | 2026-08-29 |
| **F** — a cauda | 19 | **15** | 🟡 4 abertas (F2·F4·F8·F16) + F5 devolvida · **F1 RECUSADA com medição (§15)** | LLM | 2026-08-29 |
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

### Bloco B — fechado 2026-08-29

**Resultado:** `20 041 / 20 041` outra vez, clippy verde, `fmt` limpo, `physics_ecs_c9` no mesmo
hash, `deny` e `audit` ok. **196 pacotes moveram**, todos compatíveis.

⛔ **A GUARDA do plano disparou e a SONDA é que estava errada.** A verificação «nenhum crate dos
blocos C/D/E pode mexer» acusou `kurbo`, `skrifa`, `peniko`, `glam`, `accesskit`, `miniz_oxide` —
mas ela indexava o lock por **nome**, e um dicionário `nome → versão` **colapsa duplicados**. Esta
árvore tem várias crates em duas versões de propósito. Refeita como **multiconjunto**, a resposta é
outra: `wgpu` 28, `vello` 0.8, `bevy_ecs` 0.18, `rapier2d` 0.28, `pollster` 0.4, `ndarray` 0.15 —
**todos parados**, como o plano exige.

⚠️ **E o mesmo defeito produziu um alarme falso sobre `glam`:** o lock lista **19 versões**
(0.14 … 0.33). Elas são dependências **opcionais e desligadas** do `nalgebra`, que oferece conversões
para cada glam já lançada — e o `Cargo.lock` é **agnóstico a features**, resolve o que *poderia* ser
activado. Compilada há **uma**: `glam v0.30.10`. *A nossa continua em 0.30 — o trabalho do bloco F é
real.*

⭐ **Três supressões mortas apagadas.** O `cargo update` levou o `quick-xml` de 0.39.4 a **0.41.0**,
que corrige as **duas** advisories de DoS cujo `ignore` no `deny.toml` dizia *«não há versão corrigida
alcançável sem bumpar a cadeia winit/Wayland»*. Havia. Verificado retirando ambas: `advisories ok`.
⛔ *Uma supressão cujo motivo é «não há saída» tem de ser re-testada sempre que o grafo se mexe.*

**Três duplicatas NOVAS**, e as duas que importam são transitórias: `skrifa 0.44` + `read-fonts 0.41`
vêm do `swash 0.2.10` ← `parley 0.6` — **o bloco C colapsa-as** ao levar a nossa `skrifa` a 0.44.
`miniz_oxide 0.9.1` é o tecto já documentado (folha de compressão; duas cópias é benigno).

### ⛔ ENOSPC no meio do bloco B — e NÃO era o disco

O clippy morreu com `No space left on device` e quatro `could not compile`. **Nenhum era erro real.**

| | |
|---|---|
| disco do projecto | 34 GB de 1,9 TB · 1 827 GiB não-alocados · `btrfs-health` **VERDE, e certo** |
| `/mnt/ramtarget` (tmpfs, 48 GB) | **100 % cheio** |

`target/debug` e `target/rust-analyzer` são **links** para essa tmpfs (setup deliberado e medido:
73 % menos escrita no SSD). Ocupavam **23 GB de `debug/incremental`** e **16 GB de cache do RA**.

⭐ **A causa não é «a tmpfs é pequena»: é que esta jornada NÃO TEM inner loop.** Toda corrida é um
build em LOTE da workspace inteira (`clippy --workspace`, `nextest --workspace`), e o §2 do
`CLAUDE.md` já diz que em lote a compilação incremental «não colhe nada e paga 11 GB». Aqui pagou 48.
Medido depois de libertar: **100 % → 26 %**, e o clippy da workspace inteira **sem** incremental
custou **~1 GB** (26 % → 28 %).

⇒ **Os blocos C, D e E correm com `CARGO_INCREMENTAL=0` em TODA corrida**, não só no nextest.

⚠️ E isto qualifica o **T1**: o `nodatacow` valeu para `ci-test` e `release` (disco); `debug` e
`rust-analyzer` vivem em tmpfs, onde ele não significa nada.

---

### Bloco F — **14** de 19 fechadas em 2026-08-29 (5 abertas + a F5 devolvida)

> ⛔ **CORREÇÃO DE CONTAGEM (2026-08-29, mais tarde no mesmo dia).** Esta secção dizia **18 de 19**,
> e o número saiu de contar a **unidade errada**: somei *9 subidas + 9 declarações de teto* = 18
> «coisas», quando as 9 declarações de teto pertencem a **4** tarefas (F3, F7, F10, F19). Medido
> contra a árvore, com `grep` nos manifestos:
>
> | | tarefas | quais |
> |---|---|---|
> | subidas feitas | **9** | F6 · F9 · F11 · F12 · F13 · F14 · F15 · F17 · F18 |
> | tetos documentados | **4** | F3 `pollster` · F7 `miniz_oxide` · F10 `ndarray` · F19 `core-graphics` |
> | recusa medida | **1** | F5 `linesweeper` |
> | ⛔ **por fazer** | **5** | **F1 `glam` 0.30** · **F2 `rfd` 0.15** · **F4 `mlua` 0.10** · **F8 `cpal` 0.15** · **F16 `usvg` 0.43** |
>
> ⚠️ **Nenhuma das cinco tinha motivo registado** — não foram adiadas, ficaram por fazer. O que as
> escondeu foi o placar dizer «18/19»: um número que fecha faz ninguém reabrir a lista.
> *Um placar conta tarefas; contar declarações dá um número maior e igualmente verdadeiro sobre
> outra pergunta.*
> ⭐ E a **F1 (`glam`)** deixou de ser tarefa de cauda: ela está **acoplada ao bloco E** pelo
> `glamx` (§6) e a ordem entre as duas tem de ser **medida**, não escolhida.

**Resultado:** `20 041 / 20 041`, clippy verde, `fmt` limpo.

**Nove subidas limpas, sem tocar numa linha de código:** `roxmltree` 0.20→0.21 · `jxl-oxide`
0.10→0.12 · **`zip` 2→8 (seis majors)** · `taffy` 0.12→0.14 · `ctt` 0.4→0.5 · `criterion` 0.7→0.8 ·
`toml` 0.9→1.1 · `wasmtime` 47→48.

⭐ **`symphonia` 0.5 → 0.6 foi REDESENHO, não renome** — `SampleBuffer`, `DecoderOptions` e
`CODEC_TYPE_NULL` **deixaram de existir**. Migrado lendo o fonte real da 0.6 (`SampleBuffer` →
`copy_to_vec_interleaved` num `Vec` nosso · `codecs::audio::AudioDecoderOptions` ·
`make_audio_decoder` · `Probe::format`→`probe` devolvendo o leitor directo · `packet.track_id`
virou campo · `AudioSpec` deixou de ser `Copy`). Verificado a jusante: `audio-decode` 2,
`audio-stream` 4, `audio-encode` 28, `audio-edit` 235 — todos verdes.
⭐⭐ **Duas mudanças MELHORARAM o nosso código:** o fim de ficheiro deixou de ser um
`IoError(UnexpectedEof)` disfarçado e passou a ser `Ok(None)` explícito (o ramo antigo fica como
rede, para um ficheiro cortado a meio); e a escolha de faixa passou de *«o codec não é NULO»* para
*«os parâmetros existem **e** são de áudio»* — mais estrito, de graça, porque o enum novo distingue
áudio de vídeo e legenda.

**Nove tectos documentados no sítio onde alguém os vai querer subir** (`pollster` ×4,
`miniz_oxide` ×3, `core-graphics`, `ndarray`), cada um com o dono, o motivo e o gatilho de
reabertura. ⚠️ O `ndarray` está marcado pelo que é: **dívida NOSSA** — quem o prende é o
`deep_filter` que nós vendorizámos, e o `stack-audit.sh` nem o vê (não varre `vendor/`).

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
| ~~**rapier «migrou para glam»**~~ | ⛔⛔ **ESTA RECUSA ESTAVA ERRADA E FOI RETIRADA** — ver a linha abaixo | 2026-08-29 | — |
| ~~**`linesweeper` 0.4**~~ | ⭐ **RESOLVIDO no mesmo dia — a causa era NOSSA.** Ver abaixo | 2026-08-29 | `crates/ph2d-vec-boolean/src/expand.rs`, doc de `Region::of` |
| | | | |

### ⭐⭐⭐ A recusa nº 3 (`linesweeper`) foi RESOLVIDA — e o defeito era do nosso lado

**A recusa dizia:** *«a 0.4 mudou duas convenções ao mesmo tempo — saída aproximada por omissão e
direção de winding invertida — e parte o power stroke; ela declara-se early beta.»* Ela nomeava a
biblioteca como culpada e listava três hipóteses já eliminadas.

⛔ **Nenhuma das duas convenções era a causa.** Medido no motor cru, sobre uma fita de 128
quadriláteros de largura variável:

| chamada | 0.3.0 | 0.4.0 |
|---|---|---|
| `binary_op(a, **a**, NonZero, Union)` — o que fazíamos | 1 grupo | **128 grupos** |
| `binary_op(a, **∅**, NonZero, Union)` | 1 grupo | **1 grupo** |
| `contours_correct` (o modo «antigo») sobre a auto-união | — | **128** |

**A causa:** `Region::of` regularizava por `A ∪ A = A` e passava o **mesmo** caminho como os dois
operandos, pondo **cada aresta na varredura com multiplicidade 2**. A partir da 0.4, uma
multiplicidade **par** faz o motor deixar de dissolver a aresta interna que dois quadriláteros
vizinhos partilham — e cada quad sai como contorno próprio.

⚠️ **É paridade, e mede-se como paridade:** `k` cópias coincidentes dão **1** peça para `k` ímpar e
**uma peça por face** para `k` par. A mesma região desenhada como um hexágono (sem aresta interna)
dá 1 para todo `k`. E é **livre de escala** — a obliquidade varrida por **12 ordens de grandeza**
reprova nas nove sob auto-união e acerta em todas sob `A ∪ ∅`. ⇒ **não era tolerância.**

⭐ **A cura é uma linha**, e o doc-comment que estava por cima dela **declarava a armadilha**: ele
dizia, em texto, que a identidade `A ∪ A = A` era a razão do desenho.

⚠️ **A lição:** a recusa foi escrita a olhar para o que a **biblioteca** mudou, porque foi a
biblioteca que se moveu. Mas uma quebra é o encontro de duas coisas, e o lado que não se moveu
também é suspeito — *o nosso lado dependia de uma propriedade que nunca foi prometida, e só se
tornou visível quando a outra metade mudou.*

### ⛔⛔⛔ A recusa nº 2 estava ERRADA — e o modo de falha é o mais caro que existe

**O que a recusa dizia:** *«o `CHANGELOG` do `master` da rapier anuncia a migração `nalgebra` → `glam`
na 0.32; **nenhuma versão publicada faz isso**; continuamos em `nalgebra`.»*

**O facto, medido em 2026-08-29 (recon do bloco E):** a rapier 0.32.0 em diante **migrou mesmo**.
A dependência não se chama `glam` — chama-se **`glamx`** (um invólucro que a rapier publica por cima
do `glam`). A busca que produziu a recusa procurou a string `glam` nos manifestos e não a achou.

**A prova que fecha:** `parry2d 0.30.2` (a dependência da rapier 0.35.3) exporta
`Vector = glam::Vec2`, `Pose = Pose2`, `Rotation = Rot2`, e **`Point` / `Isometry` / `Translation`
deixaram de existir** — `grep` devolve zero linhas em `parry2d-0.30.2/src/math/mod.rs`.

⇒ **O bloco E não é um bump de versão: é uma migração de biblioteca de matemática em 47 ficheiros**,
e nenhuma das 14 tarefas do plano a menciona. O custo do bloco muda de categoria.

⚠️ **A lição, que vale para além desta jornada:** uma recusa medida é escrita para **impedir que
alguém volte a perguntar**. Quando ela está errada, ela não falha como um teste falha — ela
**apaga a pergunta**. Esta teria custado a alguém a descoberta no meio da execução, com o bloco
já meio migrado. ⇒ **uma ausência só se declara pelo nome do símbolo que se procurou**, e a nota
tem de dizer *qual* string foi procurada, para a próxima pessoa poder ver que a busca era estreita.

## §15 — ⛔⛔ A tarefa **F1** (`glam` 0.30 → 0.33) é uma RECUSA MEDIDA (2026-08-29)

O plano mandava unificar o `glam`. **Não deve ser feito**, e a razão só apareceu depois do bloco E.

### §15.1 — O mecanismo, lido no grafo de features (não deduzido)

```
rapier2d/enhanced-determinism → parry2d/enhanced-determinism
                              → glamx/scalar-math → glam/scalar-math
```

Hoje a árvore tem **duas** cópias do `glam`: a **0.33.6** que a física trouxe (via `glamx`) e a
**0.30.10** que as nossas oito crates de desenho usam. O Cargo unifica features **por versão**, e é
exactamente por isso que a cópia da física corre em `scalar-math` (SIMD desligado, exigência do
determinismo entre sistemas — HR-5) enquanto a nossa mantém o SIMD ligado.

⭐⭐⭐ **As duas cópias não são um resíduo: são o mecanismo que deixa a física ser determinística e o
renderizador ser rápido ao mesmo tempo.** Unificar o `glam` numa versão só imporia a política da
física a `ph2d-core`, `ph2d-mesh-render`, `ph2d-vector`, `ph2d-anim`, `ph2d-vec-edit`,
`ph2d-vector-font`, `ph2d-vector-doc` e `ph2d-vector-traits`.

### §15.2 — O outro lado da balança, contado

O que a 0.31→0.33 traz, e quantas vezes o nosso código o alcança:

| o que a versão nova dá | usos nossos |
|---|---|
| `Affine3` (afim não-SIMD) e tudo o que toca `Vec3A` | **0** |
| `ISizeVec2/3/4` | **0** |
| a correcção de `escalar / matriz` (0.32.1) | **0** |
| `USES_WASM_SIMD` (renome) | **0** |
| tipos opcionais (tempo de compilação) | ganho pequeno, não medido |
| `try_inverse`, `mul_diagonal_scale`, `mul_transpose_vecn` | conveniências, nada bloqueado hoje |
| ⛔ `Vec2::angle_between` **removido** | **6 sítios a reescrever** — é *custo*, não ganho |

⇒ **O lado do benefício é zero medido, e o lado do custo tem mecanismo provado.** Uma troca cujo
ganho é zero perde para qualquer custo maior que zero, e por isso **esta recusa não precisou de um
número de desempenho** — precisou de contar os usos.

### §15.3 — ⚠️ Uma hipótese minha, refutada por medição

Escrevi primeiro que o perigo era o `Vec3A` encolher de 16 para 12 bytes e desalinhar buffers de
GPU. **Falso: há zero usos de `Vec3A` no repositório.** O risco de layout é nulo, e o argumento
verdadeiro é só o SIMD. *Uma hipótese plausível sobre um tipo que ninguém usa mede zero.*

### §15.4 — Quando reconferir

Esta recusa responde **uma** pergunta: *«vale a pena unificar hoje?»*. Ela muda se:
- alguma crate nossa passar a precisar de `Affine3`/`Vec3A`, **ou**
- o `glamx` deixar de reencaminhar `scalar-math` para o `glam`, **ou**
- medir-se que `scalar-math` custa pouco no nosso caminho de desenho — e aí o argumento passa a ser
  o tempo de compilação, que ainda não foi medido.

## §14 — Bloco E, etapa B — `rapier2d` 0.31 → **0.35.3** (+ `parry2d` 0.30.2, `glamx` 0.3) (2026-08-29)

**A maior mudança de superfície da jornada inteira**, e a única cujo risco não era de compilação.

### §14.1 — O que a plataforma fez

A `rapier` trocou a matemática de `nalgebra` para `glam` (pelo invólucro `glamx`). O vocabulário
inteiro mudou de nome — `Vector`, `Pose`, `Rotation` — e três tipos foram **apagados**: `Point`,
`Isometry`, `Translation`. Tudo isso é erro de compilação, logo seguro.

⛔⛔ **Uma coisa não é.** No `nalgebra`, `Point2` e `Vector2` são tipos **distintos de propósito**:
um ponto é um lugar, um vetor é um deslocamento, e `Isometry2 * Vector2` **só roda** (ignora a
translação) enquanto `Isometry2 * Point2` roda **e** translada. No `glam` os dois são o **mesmo
tipo**, e por isso `Pose2 * Vec2` é **sempre** `transform_point`.

⇒ *Todo sítio que multiplicava uma isometria por um vetor de direcção passou a ganhar uma
translação que antes não existia — e **compila**.* Era a única classe de defeito silencioso da
migração, e o custo dela é sempre o mesmo: uma direcção que anda com o corpo.

### §14.2 — Como ela foi medida, e por que a resposta foi zero

⚠️ **A varredura foi feita POR OPERAÇÃO, não por ficheiro** — 119 ficheiros, cada multiplicação de
uma pose por um vetor classificada como *era-ponto* ou *era-direcção*, **com um controlo**: se a
varredura não achasse nem um único caso de cada lado, ela estaria partida e não limpa.

**Resultado: exposição ZERO.** E a causa não foi sorte — foi a disciplina antiga desta crate.
Enquanto o `nalgebra` distinguia os dois tipos, o nosso código **era obrigado** a escolher `Point2`
ou `Vector2` em cada sítio, e escolheu certo em todos. A fusão dos dois tipos herdou essa escolha.
⭐ *Um tipo que force uma decisão hoje paga-se quando a plataforma deixar de a forçar amanhã.*

### §14.3 — O trabalho

**116 erros de compilação → 0**, em 25 ficheiros e ~340 pontos, feitos por **quatro agentes em
paralelo** sobre conjuntos disjuntos, com o núcleo (`rmath`, `world`) por mim.
**11 testes vermelhos → 0**, e nenhum baixando a barra sem prova:

| quantos | o que era |
|---|---|
| 4 | constantes do solver que mudaram o **tato** — fixadas com o número escrito ao lado |
| 2 | a **mesma melhoria** (balas já não atravessam cenário fixo) — viraram dois testes distintos |
| 1 | media a **direcção de um torque** em vez da consequência física |
| 1 | media a altura de marcha, que a física real já roçava — barra alargada **com a medição ao lado** |
| 1 | confiava em **exactidão de bits** que dissolveu, e ameaçava apagar animação do artista |
| 2 | degradações reais, pequenas, **medidas e registadas** |

### §14.4 — ⛔ A recusa medida

Existe um campo novo que curaria um dos testes ao preço de trazer de volta **pilhas altas a tombar**.
**Não foi ligado.** O mecanismo e o número estão em `crates/ph2d-physics/src/world.rs`, ao lado da
constante — não aqui, porque uma recusa longe do código que ela governa não é lida por ninguém.

### §14.5 — ⭐⭐ A escada da caixa que cai, e a cerca no SINAL

O último vermelho foi `the_drop_survives_exactly_where_the_resting_box_still_overlaps_the_plank`,
que afirmava um **bicondicional** célula a célula. A tabela das dez células está no próprio teste.

⚠️ **A lei real tem DUAS cláusulas** — *«já passei»* (as caixas envolventes) **e** *«a prancha parou
de me pegar»* (o cone do gancho) — e o teste modelava só a primeira. Nove de dez células
concordavam; numa, a caixa já passou e o cone ainda pega. ⚠️ **A margem sozinha não explica qual
falha**: outra célula com a *mesma* margem de 5 cm concorda.

⇒ **A cerca mudou-se para o SINAL, que é onde a segurança mora.** As duas direcções não custam o
mesmo: aposentar **cedo** torna a prancha sólida com o personagem a cair através dela (o arremesso
que esta wave curou); sobreviver **de mais** apenas o deixa continuar a cair. Por isso o
bicondicional fica **intacto no lado perigoso**, e o lado seguro ganha a faixa de cruzamento.
⭐ *Um teste que pesa as duas direcções por igual está a defender o defeito barato com o mesmo rigor
com que defende o caro.*

### §14.6 — Dois defeitos que o portão de FECHO achou, e que não eram da subida

1. ⚠️ **A direcção do `NaN`.** O `clippy` apontou `if !(hi - lo > PISO)` em `bake.rs` como estilo.
   Não era estilo: com uma amostra `NaN` a comparação é falsa, o `!` torna-a verdadeira, e o canal
   é declarado **constante e descartado** — que é exactamente a falha que aquele piso existe para
   curar. Escrito na forma positiva, um canal envenenado **sobrevive** para quem o consome o ver.
   ⭐ *O lado em que se põe o `!` decide se um dado partido é apagado em silêncio ou entregue.*
2. **O teto de 700 LOC, pela quarta vez nesta jornada**, e sempre pelos meus próprios comentários.
   Curado como as outras três: **corte por responsabilidade**, nunca subindo o teto. A porta das
   camadas (`groups_for`) mudou-se para o módulo `layers`, que é o dono do assunto — os quatro
   consumidores não mudaram uma linha, porque o nome ficou re-exportado.

### §14.7 — O que ficou verde

`physics_ecs_c9` **estável em 2 de 2 corridas**, hash `20e3e7a8…`. Suíte da física **981/981**.
⚠️ `nalgebra` **continua na árvore e não é resíduo**: entra pelo `fidget` (campo implícito) e pelo
próprio `glamx` (álgebra densa), e o `machete` confirma zero dependências mortas.

## §13 — Bloco E, etapa A — `rapier2d` 0.28 → **0.31.0** (2026-08-29)

**A paragem intermédia que o plano não tinha.** A 0.31.0 é a **última versão em `nalgebra` puro**
(o `glamx` entra na 0.32), e parar aqui entrega o solver reescrito da 0.29, o `contact_softness`,
o `InteractionTestMode` e o `CoefficientCombineRule` **sem tocar num único `Vector2`**.

⇒ É isso que faz o `physics_ecs_c9` **isolar** a mudança de solver: um hash que se mexe aqui tem
uma causa só. Misturar as duas etapas tornaria qualquer diferença impossível de atribuir.

### §13.1 — As seis mudanças de API, e duas são MELHORIAS

| o quê | nota |
|---|---|
| `num_solver_iterations` deixou de ser `NonZeroUsize` | ⚠️ **a rede mudou de dono** — a guarda contra o zero era do TIPO e passa a ser nossa. Escrita. |
| `contact_damping_ratio` + `contact_natural_frequency` → `contact_softness` | ⭐ os dois sempre foram os **dois parâmetros de UMA mola**; como campos soltos dava para escrever um e esquecer o outro |
| `effective_world_inv_inertia_sqrt` → `effective_world_inv_inertia` | ⭐ a rapier deixou de guardar a **raiz**, e o nosso código elevava-a ao quadrado para desfazer isso. Menos uma ida-e-volta. |
| `InteractionGroups::new` ganhou um 3.º argumento | ⚠️ escrevemos `And` **à mão** em vez de herdar o `default` — ver §13.2 |

### §13.2 — ⚠️ Por que o `InteractionTestMode::And` vai escrito

A porta das camadas de colisão é **única** (`groups_for`), e a regra que ela implementa é
*«uma camada só interage com outra se as duas linhas da matriz concordarem»* — que é **exactamente**
o `And`. Herdá-lo de um `Default` significaria que, no dia em que o upstream mudasse esse default,
**a matriz que o artista desenhou passaria a querer dizer outra coisa sem uma linha nossa mudar**.

### §13.3 — ⭐⭐ O achado: um corpo com «não gira» voltou a girar

Dois gates caíram, e o defeito é do pior tipo: **a marca continua no inspector e o corpo deixa de
obedecer**. Medido: um corpo com `lock_rotation` e `angvel = 5` girava **2,5 rad em 0,5 s**.

⭐ **E a resposta já estava escrita no ficheiro.** O comentário do eixo de *translação* dizia, com
todas as letras: *«a rapier trata só a ROTAÇÃO como caso especial»* — até à 0.28 o solver anulava
sozinho a velocidade angular de um corpo travado, e por isso só a translação precisava da nossa
projecção. **O solver reescrito da 0.29 não o faz mais.**

⚠️ *A assimetria nunca foi nossa: era compensação de uma assimetria deles. Quando ela caiu, a nossa
lei ficou meio escrita.* A cura é a simétrica — um eixo congelado não carrega velocidade, nos dois
eixos —, que é também o que o Unity e o Godot fazem, e o que o nosso próprio comentário já defendia.

### §13.4 — ⭐⭐ E um gate que passava por SORTE

`a_joint_made_mid_swing_survives_a_reset_unchanged` afirma *«o pino não pode andar ao longo do
corpo depois de um Reset»* — intenção certa, **régua errada**. Ele comparava *onde a prancha estava
pendurada* depois de 140 tiques, vivo contra repetido, com uma barra de `0,05 m`.

⛔ **As duas corridas não são a mesma simulação:** a viva cai 40 tiques em queda livre antes de ser
presa; a repetida nasce presa. Energias diferentes ⇒ amplitudes diferentes ⇒ **dois pêndulos sem
amortecimento que nunca convergem**. Medido, variando os tiques:

| tiques | 140 | 300 | 600 | 1200 |
|---|---|---|---|---|
| \|diferença\| | 0,133 | 0,035 | 0,055 | **0,289** |

A diferença **oscila** — ela é a fase de dois pêndulos fora de sincronia. Que ela ficasse abaixo de
`0,05` aos 140 tiques era **coincidência**. O solver novo mudou a fase, não o pino.

⇒ A régua passa a ser **a âncora** (`local_a`/`local_b`), que é o que o teste sempre disse querer
medir e que é um **facto discreto**: ou são os mesmos bytes dos dois lados do Reset, ou não são.
⚠️ *Uma régua que aceita um INTERVALO pode ser satisfeita por acaso; um oráculo não.*

## §12 — A cauda do bloco F que tinha ficado por fazer (2026-08-29)

As **cinco** que o placar escondia (§«Bloco F», correcção de contagem). A **F16** (`usvg`) entrou
no bloco C porque é ela que solta o `kurbo`; a **F1** (`glam`) fica para depois do bloco E, porque
as duas estão acopladas (§6).

### F2 — `rfd` 0.15 → 0.17 · **zero mudanças de código**
As 8 assinaturas que usamos são idênticas. ⭐ **E a subida corta 41 pacotes do grafo:** a 0.17.0
trocou o backend do diálogo de ficheiro de `ashpd`(zbus) por `libdbus` via `dlopen`, e o `zbus`
inteiro sai. ⛔ **Teto confirmado:** é este crate que prende o `pollster` em `^0.4`.
⚠️ O transporte D-Bus passa a ser uma `.so` do sistema aberta em runtime — **um humano tem de abrir
`Save`, `Save As…`, `Open Project…` e `Import…` uma vez cada**; nenhum teste alcança um diálogo.

### F4 — `mlua` 0.10 → 0.12 · **duas linhas de `use`**
`ThreadStatus` mudou-se para `mlua::thread` e o `Compiler` para `mlua::chunk`. Os ~70 outros usos
têm assinatura idêntica.
⚠️ **Uma quebra que o reconhecimento não viu:** `gc_step_kbytes(kb)` deixou de existir. A substituta
(`gc_step()`) **não recebe orçamento** — ele mudou de sítio, do ponto de chamada para o **modo do
coletor**. Isso é a forma certa (*um orçamento por-chamada era uma resposta por-quadro a uma
pergunta de configuração*), e a sonda de spike que o media passa a imprimir números **não
comparáveis** com os de antes. É sonda, não gate; ninguém depende do valor.
⭐ **E um comentário cuja RAZÃO morreu sem a decisão morrer:** o texto dizia *«o `Integer` do Luau é
`i32`, estreito demais para o `to_bits()`»*; na 0.12 ele é `i64`. O que segura a escolha é o outro
argumento, que também estava escrito ali. *Um motivo que morre não torna a decisão errada — torna-a
uma decisão que precisa do motivo que sobreviveu.*

### F8 — `cpal` 0.15 → 0.18 (**três majors**) · 7 edições, 1 ficheiro
`Device::name()` removido (→ `Display`) · `SampleRate` deixou de ser newtype (→ `u32`) ·
`StreamConfig` virou `Copy` (os `.clone()` viram erro de clippy) · o descritor passa **por valor** ·
`BuildStreamError` unificou-se em `cpal::Error`.

⭐ **A subida CURA uma violação do HR-3 que existia.** O comentário do `scratch` diz *«sized once …
no allocation in the warm hot path»*; sob a 0.15 o bloco por callback era **variável**
(1881/4410) e ele redimensionava **26 vezes em 32 callbacks**. A 0.18 entrega **512 constante** ⇒
redimensiona **uma**. ⚠️ Em troca, o orçamento por callback cai de ~85 ms para **~10,7 ms** — **8×
menos folga** para o jitter do mixer de 42 efeitos.

⭐⭐ **E fechou-se um caminho que levava ao silêncio sem sinal.** O código perguntava *«o formato
POR OMISSÃO serve?»* e, se não, desligava o som com uma linha em `stderr`. A 0.18 torna isso
provável: a heurística do formato por omissão passou a ordenar **todos** os formatos, então
hardware que devolvia `I16` pode passar a devolver `I32`/`I24`. ⇒ a pergunta passa a ser **«o
dispositivo tem ALGUM que sirva?»** (`pick_writable_config`, preferência `F32` > `I16` > `U16`,
mantendo a taxa que o dispositivo escolheria). *Um app mudo com um aviso que ninguém lê é, para o
artista, indistinguível de um app partido.*

⚠️ **A verificação honesta do áudio não é um log.** A cerca da casa (*«elos verdes = elo FORA»*)
diz que um canal pode estar mudo no PipeWire, fora do processo. O passo que prova é gravar o
monitor do sink e medir a frequência.

## §11 — Bloco C — GPU e texto (2026-08-29)

**19 linhas de manifesto**, e as seis crates subiram **juntas** como o bloco exigia:
`wgpu` 28 → **29.0.4** · `vello` 0.8 → **0.10.0** · `naga` 28 → **29.0.4** ·
`skrifa` 0.40 → **0.44.0** · `parley` 0.6 → **0.11.1** · `fontique` 0.6 → **0.11.1** ·
`accesskit` fixado em **0.24.1**. Mais o `usvg` 0.43 → **0.48** (a F16, que entrou aqui porque é
ela que solta o `kurbo`).

### §11.1 — ⭐ O que a subida rendeu de graça: cinco duplicações mortas

| dependência | antes | depois |
|---|---:|---:|
| `skrifa` | **3 cópias** (0.37 · 0.40 · 0.44) | **1** (0.44.0) |
| `read-fonts` | **3 cópias** (0.35 · 0.37 · 0.41) | **1** (0.41.0) |
| `swash` | 1 cópia | **0 — saiu da árvore** |
| `kurbo` | 2 cópias (0.11.3 · 0.13.1) | **1** (0.13.1) |
| `wgpu-types` | 2 cópias | **1** (29.0.4) |

⚠️ O `wgpu-types` colapsou no **bloco D**, não neste — a 2.ª cópia vinha do `bevy_reflect 0.18`.
O `kurbo` colapsou com o `usvg`, cuja cópia de `svgtypes` era a **única** consumidora da 0.11.3.
*Um teto pode cair por causa de um bloco que ninguém ligou a ele.*

### §11.2 — ⛔ As quatro tarefas do plano que mandavam trabalho INEXISTENTE

| tarefa | o plano mandava | medido |
|---|---|---|
| **C10** | 34 edições de `VertexState::buffers` | **0** — a linha do wgpu 29 é byte a byte igual à do 28 |
| **C13** | pôr `@interpolate(flat)` em 3 varyings | **0** — a regra já existia no naga 28, e os 3 casamentos são **atributos de vértice**, onde o atributo nem se aplica |
| **C16** | renomes de `Alignment` | **0** — o enum é idêntico desde antes da versão que usávamos |
| **C17** | `peniko::Font` → `FontData` | **0** — já tinha acontecido antes da 0.6 |

### §11.3 — ⚠️ E a contagem errou nas DUAS receitas que sobraram

| receita | plano | medido |
|---|---:|---:|
| `bind_group_layouts` → `Option` no `ph2d-render` | 9 | **13** |
| idem nas crates-ferramenta | 8 (ou 9 — o plano discordava de si próprio) | **10** |

Os agentes **não confiaram no número**: varreram o crate inteiro e reportaram a diferença. Depois
disso uma varredura da árvore toda (`bind_group_layouts` sem `Some(`, `depth_*` sem `Some(`,
`SurfaceError`, `.suboptimal`) devolveu **zero**. *Um número numa receita é uma expectativa, não
um alvo — quem executa conta.*

### §11.4 — ⭐⭐ Dois achados que valem mais que a subida

**1. Um caminho nosso que era INALCANÇÁVEL passou a existir.** O `AcquireError::Occluded` está
declarado no `ph2d-gpu` e o shell já o trata (salta o quadro) — mas o `SurfaceError` do wgpu 28
**não tinha** essa situação, então nenhum produtor podia emiti-lo. O wgpu 29 tem `Occluded`, e o
comentário do ficheiro revela porquê: *«esta árvore já esteve no wgpu 29 e recuou»*. ⇒ o C8
**restaura**, não inventa.

**2. Uma quebra que o reconhecimento declarou INEXISTENTE, e cuja cura é melhor que o original.**
Ele afirmou ter conferido os oito símbolos do `fontique` e que as assinaturas eram idênticas. O
`Script` mudou de dono (fontique → `parlance`) e **perdeu o `From<&str>`**. ⭐ A conversão antiga
era `as_bytes().try_into().unwrap_or_default()`: **um código de escrita com tamanho errado virava
quatro zeros em silêncio**, e o sintoma seria uma cascata de fontes errada para aquele idioma. O
substituto (`Script::from_bytes([u8; 4])`) torna isso **erro de compilação**.
⚠️ *O reconhecimento errou por olhar assinaturas de FUNÇÃO e não a origem do TIPO.*

### §11.5 — ⚠️ O que MUDA PIXEL, e compila verde

Esta é a lista que decide o smoke do dono. Nenhum destes itens dá erro.

| # | o quê | onde se vê |
|---|---|---|
| **1** | **Toda imagem do vello desloca-se meio pixel.** Correcção de defeito do upstream (*"blurry image rendering due to incorrect half-pixel offset"*, vello 0.9): o `fine.wgsl` passou a amostrar no **centro** do pixel. | Os três modos de qualidade, 46 sítios de `draw_image`. **Mais visível no modo de arte de pixel**: meio pixel muda qual texel o vizinho-mais-próximo escolhe ⇒ uma pré-visualização pode aparecer deslocada **uma coluna inteira**. |
| **2** | **`ImageQuality::High` era bilinear disfarçado e passa a bicúbico de verdade** (Mitchell, 16 amostras). | O filtro **«Smooth»** fica mais nítido, com o halo fino que um Mitchell produz numa borda de contraste. **2 sítios de produto.** |
| **3** | **O `y_offset` do glifo inverteu de sinal** (parley 0.8). ⭐ **E isso CURA um defeito nosso** — a 0.6 injectava uma grandeza Y-up num campo Y-down. | Acentos posicionados por *mark-attachment* subiam ao contrário. Para latino precomposto, `y_offset` é 0 e **nada muda**. |
| **4** | **O motor de moldagem saltou `harfrust` 0.3 → 0.12.** | Kerning e largura de cluster podem mover texto **um pixel** em qualquer painel. Não estava na lista de risco do plano. |

⛔ **E uma que o plano mandava procurar e NÃO PODE ACONTECER:** o gradiente do selector de cor.
Verificado no gerador da rampa do vello — o modo novo é opcional, o campo que o escolhe já existia
na versão que usávamos, nenhum sítio nosso o escreve, e o caminho por omissão é a **mesma chamada**.
O gradiente é **byte-idêntico**. *Mandar o dono procurar o que não pode acontecer gasta a única
coisa que ele tem de escasso.*

### §11.5-bis — ⭐⭐⭐ O achado do bloco: **48 gates caíram por um valor INERTE**

A fotografia do «depois» dos gates de GPU passou de **12** vermelhos para **65**, e quase toda a
suíte do `ph2d-mesh-render` estava lá. A causa é **uma linha**:

```
In Device::create_render_pipeline, label = 'ph2d-mesh wire'
  Depth bias is not compatible with non-triangle topology LineList
```

O passe de arestas declarava `DepthBiasState { constant: -4, slope_scale: -1.0 }`, com um
comentário longo a explicar que *«o viés negativo é o que faz a aresta ganhar da face»*. O
raciocínio está certo **para triângulos**; a topologia é `LineList`. O WebGPU exige viés **zero**
fora de topologia de triângulos e o Vulkan aplica-o só a polígonos ⇒ **aquele valor nunca foi
aplicado por backend nenhum**.

⭐⭐ **E esta casa já o tinha medido.** A sonda `probe_wire_continuity` carrega, escrito:

> *«não há um gate afirmando "o viés do pipeline não alcança uma linha", que é o achado mais caro
> desta investigação — porque eu não consigo fazê-lo falhar pelo motivo que ele alegaria»*

com a varredura ao lado: `constant` de **0 a −4096**, tinta **idêntica ao pixel**. A cura
verdadeira foi construída no shader (`WIRE_DEPTH_NUDGE` + o empurrão lateral, cada um a atacar
metade do problema, cada um com a sua tabela). **O campo do pipeline ficou para trás porque nada o
obrigava a sair.**

⇒ **O `wgpu` 29 foi o que o obrigou:** ele valida o que o 28 ignorava calado.

⚠️ **A lição, e ela é geral:** *um valor inerte não custa nada até ao dia em que a plataforma deixa
de o tolerar — e nesse dia custa a suíte inteira.* Pior: enquanto lá está, **quem o lê pela
primeira vez acredita no comentário**. O gate que faltava não era sobre o wireframe; era sobre o
campo estar morto — e a investigação que o descobriu disse, com todas as letras, que não sabia como
o escrever.

⭐ **Curado, e a prova é a que o próprio módulo já queria:** com o viés a zero, os gates de
continuidade do arame (*quanto de cada aresta chega à tela*, *as arestas rasantes são comidas pela
própria superfície?*) passam — **60 de 62**, e os 2 que sobram são os **mesmos dois** que já
estavam vermelhos na fotografia do antes.

### §11.5-ter — O veredito dos gates de GPU: **zero regressões**

| | antes | depois |
|---|---:|---:|
| corridos | 535 | 535 |
| passaram | 523 | **525** |
| **falharam** | **12** | **10** |

Diferença nominal: **1 novo**, **3 curados**. E os quatro são **sondas de custo/relógio**:
`emitter_sim_ceiling_probe` (o novo) re-corre **3 de 3 verde** sozinho, com a máquina a carga
**0,36**; os três «curados» (`how_far_does_the_packing_scale`, `bounded_readback_cost_probe`,
`two_seam_hybrid_timing`) são da mesma espécie e caíram no «antes» porque **aquela** corrida
apanhou a máquina a carga 20.

⇒ **9 vermelhos estáveis, os mesmos nos dois lados. O bloco C não introduziu nenhuma regressão de
GPU** — o que ele fez foi expor o viés morto (§11.5-bis) e curá-lo.

### §11.6-bis — ⚠️ Um MEMBRO NOVO da família de falso-positivos de carga

`ph2d-field-render tests::an_abandoned_march_returns_nothing_and_returns_fast` reprovou **uma vez**
no portão desta jornada (`14,24 ms` contra `17,94 ms`, uma razão de dois relógios) e passou
**3 de 3** ao correr sozinho — com a máquina ainda a **carga 20**.

Ele tem a forma exacta que o `CLAUDE.md` §5.0 descreve: *mede um recurso partilhado dividindo dois
relógios*. E o crate não tem relação nenhuma com este bloco — o traçador de campo é CPU, não toca
`wgpu`, `vello` nem `parley`. ⇒ acrescentar à lista de membros confirmados no fecho da jornada.

### §11.6 — ⚠️ Uma decisão que passa por omissão se ninguém a nomear

O `parley 0.11.1` tem uma feature nova **`complex-scripts` que NÃO é default**. Sem ela, a
segmentação de linha e de palavra degrada para **tailandês, khmer, lao e birmanês** (o parley cai
nos segmentadores *não-complexos*). Nenhum idioma do produto hoje ⇒ **não ligámos**, mas fica
nomeado: é uma escolha, não um esquecimento.

## §10 — Bloco D — `bevy_ecs` 0.18.1 → 0.19.1 (2026-08-29)

**8 declarações subidas. Zero erros de compilação.** E foi exactamente isso que tornou o bloco
perigoso: tudo o que ele muda é **comportamento silencioso**.

### §10.1 — ⛔ A decisão que eu tomei ERRADA, e o portão que me corrigiu

O sítio quente é `shells/desktop/src/render_loop/sim_extract.rs`, que descarta o mundo de
apresentação por quadro. Na 0.19 recursos **são entidades**, e `World::clear_entities()` contorna os
hooks: destrói as entidades-recurso e deixa o índice interno a apontar para elas.

**Eu decidi manter `clear_entities()`**, com este argumento: *«o modo de falha é ALTO — quem violar
descobre na hora, então basta uma cerca.»* Escrevi o portão que afirmava isso.

⛔ **O portão ficou vermelho: não há pânico.** Medido na 0.19.1, o resultado depende da ORDEM:

| ordem | o que acontece |
|---|---|
| descartar → inserir recurso | **PÂNICO** (`ResourceCache … ValidButNotSpawned`) |
| descartar → **criar entidades** → inserir recurso | ⛔ **silêncio, e corrompe** |
| descartar → ler recurso | devolve `None`, sem aviso |

⚠️ **A do meio é exactamente a ordem do laço do quadro.** Medido: das 5 entidades criadas, uma
recebe a marca `IsResource` soldada por cima e **desaparece de toda consulta filtrada** — a
contagem lê 4, as 5 continuam lá, e nada falha.

⇒ *A premissa da minha decisão era falsa, e só um teste escrito para a afirmar a derrubou.*
**Um gate que só confirma o que já se acredita não paga o custo de existir.**

### §10.2 — As quatro saídas, medidas

| caminho | veredito |
|---|---|
| `clear_entities()` | ⛔ corrompe em silêncio nesta ordem |
| despachar **tudo**, recursos incluídos | ⛔ **PÂNICO** — a entidade-recurso morre em cascata e é revisitada |
| despachar só o que **não** é recurso | ✅ **adoptado** — 300 quadros, recursos intactos, inserção posterior funciona |
| `*mundo = World::new()` | ✅ correcto, mas aloca um mundo por quadro |

⚠️ **E o portão de zero alocação apanhou a primeira implementação da cura:** construir a consulta
por quadro media **107 blocos / 10 quadros** contra um orçamento de **64**. Guardar o `QueryState`
entre quadros pô-lo de volta em regime. *A cura correcta partia outra regra, e foi outro portão
— não uma revisão — que o disse.*

### §10.3 — ⭐⭐ O hash de determinismo derivou, e a causa está PROVADA

`cross_os_golden_hash_pinned` ficou vermelho. A mensagem dele perguntava *«libm? glam?»* — as duas
respostas erradas, e carimbar o valor novo teria sido o caminho fácil.

**A/B medido**, com a 0.18.1 corrida numa **árvore separada** no commit anterior:

| | bevy 0.18.1 | bevy 0.19.1 |
|---|---|---|
| hash **com ids** | `d2a3ca34…` | `987aa255…` |
| hash **só das matrizes** | `0308874d…` | **`0308874d…`** (idêntico) |
| 1.º `to_bits` | `4294967196` | `4294967195` |

⇒ A deriva é **rotulagem**: a 0.19 gasta o índice 0 do alocador numa entidade sua no arranque e
desloca todos os 100 ids em **exactamente 1**. A matemática não mexeu **um bit**, e o
bit-idêntico entre sistemas operativos está intacto.

⭐ **E o instrumento ficou melhor do que estava.** O portão passa a ter **dois** valores fixados —
`EXPECTED_MATRICES_HASH` (só os floats) e `EXPECTED_GLOBALS_HASH` (floats + ids) — e a mensagem de
cada um diz o que o vermelho dele significa. ⚠️ *Antes, uma regressão de determinismo real e uma
re-rotulagem inofensiva produziam o MESMO sintoma e a MESMA mensagem; quem tivesse pressa
recapturava as duas e apagava a distinção para sempre.*

### §10.4 — O resto

- **`entity_count()` filtrado** (`SimWorld` e `PresentWorld`): na 0.19 um mundo recém-construído já
  tem 1 entidade, então a conta crua devolveria `1` para um mundo vazio. As **6** asserções de
  contagem de `sim_present_flow.rs` ficaram intactas por causa disso.
- **`count_simulatable`** (a régua que diz espelhar a ponte da física) passou a excluir recursos —
  ela media **4 onde há 1**. ⚠️ A ponte **não** é afectada: todas as consultas dela exigem um
  componente positivo (`&RigidBody`, `&Collider`, …), e por isso nunca alcançaram entidades-recurso.
- **`save_tests.rs`** despawnava tudo e passaria a entrar em pânico. ⚠️ O undo **de verdade** nunca
  teve o defeito — ele filtra por `With<Transform>`. *O teste imitava o undo pela FORMA e não pela
  GARANTIA, e foi só aí que a diferença apareceu.*
- **20 comentários** com o número de versão envelhecido, actualizados com contagem verificada.
  ⚠️ O que afirma que `to_bits` **inverte** a ordem de criação **não** foi reescrito às cegas: o
  gate que o mede foi corrido na 0.19 e continua verde ⇒ a nota envelheceu no **número**, não no
  facto, e passa a dizer *«medido na 0.18, reconferido na 0.19»*.

## §9 — A FOTOGRAFIA DO ANTES (tarefa C1, refeita) — **12 gates de GPU já vermelhos**

⛔ **A tarefa C1 do plano não é executável como está escrita.** Ela manda `cargo run` do app com
duas variáveis de despejo — mas o app é uma **janela**, ele não termina sozinho, e ninguém está
sentado à frente dela. E manda contar **61** ficheiros de golden: existem **3**.

**O que de facto serve de «antes»** é outra coisa, e ela existe: os **gates de GPU**, que são
`#[ignore]` e que *o CI nunca correu*. ⭐ Esta máquina tem GPU real (RTX 5060 Ti + RADV), então
eles **rodam** — e é a única forma de comparar antes/depois de um bloco que muda pixel sem depender
só do olho do dono.

```
cargo nextest run --cargo-profile ci-test --run-ignored ignored-only --no-fail-fast \
  -p ph2d-gpu -p ph2d-gpu-cook -p ph2d-render -p ph2d-flip-render -p ph2d-mesh-render \
  -p ph2d-paint-gpu -p ph2d-inpaint -p ph2d-vec-render -p ph2d-text -p ph2d-vector \
  -p ph2d-vector-font -p ph2d-system-fonts -p ph2d-a11y
```

**Resultado em `b812f8dc4`, antes de tocar em C/D/E:**
`535 testes · 523 passaram · **12 falharam** · 781 saltados · 781,9 s`

| gate vermelho **antes** | crate |
|---|---|
| `w2/w3/w4/w5_smoke_scene_loads_without_panic_and_matches_goldens` (4) | `ph2d-render` |
| `the_colour_loop_closes_the_same_way_on_the_device` | `ph2d-gpu-cook` |
| `value_slope_kernel_matches_the_cpu_on_the_device` | `ph2d-gpu-cook` |
| `two_seam_hybrid_timing` | `ph2d-gpu-cook` |
| `bounded_readback_cost_probe` | `ph2d-gpu-cook` |
| `how_far_does_the_packing_scale` | `ph2d-gpu-cook` |
| `the_field_pass_is_linear_in_the_segment_count_and_the_cap_fits_a_frame` | `ph2d-render` |
| `the_mesh_appears_on_screen_at_the_size_the_framing_promised` | `ph2d-mesh-render` |
| `the_pose_scale_grows_the_silhouette_without_tilting_the_light` | `ph2d-mesh-render` |

⚠️ **Esta tabela é o produto todo desta tarefa.** Sem ela, o bloco C entrega doze vermelhos e
ninguém consegue dizer quais são dele. *Um «antes» que não foi tirado transforma toda regressão
pré-existente em regressão nova.*

⛔ **Uma primeira corrida foi ABANDONADA de propósito, e o motivo vale mais do que o resultado:**
ela pedia os ignorados da workspace **inteira**, e caiu dentro dos testes de **medição** — um deles
tem 27 min medidos numa nota do `CLAUDE.md`. Além de não acabar, ele mede **relógio**, e a máquina
estava sob seis agentes: seria ruído gravado como linha de base. *A fotografia certa é a das crates
que o bloco toca.*

⚠️ **E ela caiu antes disso, por ENOSPC** — não no disco (709 GB livres) mas no `target/` que vive
em **RAM** (48 GB, cheio, com 39 GB só em `deps`). ⇒ **lei operacional desta jornada:** o `target/`
em RAM não cabe um build de todos os alvos deste monorepo; o laço interno (`cargo check -p`) fica
nele, e **toda corrida em lote vai para o disco** (`--cargo-profile ci-test`). Libertar o cache
devolveu 41 GB de RAM.

## §8 — O RECONHECIMENTO (2026-08-29) — e por que ele mudou quase todos os blocos

> Antes de executar C, D e E, cada bloco foi reconhecido contra a **fonte real** (os `.crate`
> baixados e diffados lado a lado, versão actual × versão alvo), não contra changelogs.
> ⚠️ **O resultado justifica o passo:** o plano foi escrito a partir de changelogs e errou em
> **onze** afirmações verificáveis. Duas delas mandavam fazer trabalho que não existe; uma
> descrevia o perigo **ao contrário**; uma apagava o bloco mais caro da jornada.

### §8.1 — O placar da conferência

| bloco | tarefas do plano | confirmadas | **irrelevantes** (0 sítios) | **erradas** | quebras que o plano NÃO viu |
|---|---:|---:|---:|---:|---:|
| **C** — GPU e texto | 22 | 11 | 5 | **4** | **5** |
| **D** — bevy_ecs | 14 | 2 | **9** | **2** | **5** |
| **E** — rapier2d | 14 | 8 | 4 | **2** | **18** |

### §8.2 — Os quatro achados que mudam decisões

**1. ⛔ O bloco E não é um bump — é uma migração de matemática** (§6). A rapier 0.32+ trocou
`nalgebra` por `glam`, através de um invólucro chamado **`glamx`**. 47 ficheiros.
⭐ Mitigação medida: a `glamx` foi desenhada com a forma do `nalgebra` — `rot.angle()` é idêntico
(14 sítios intactos), e muitos casos só perdem um `&`.
⭐ E existe uma **paragem intermédia**: `rapier2d 0.31.0` é a última em `nalgebra` puro e entrega as
tarefas E9–E13 inteiras **sem tocar num `Vector2`**. ⇒ o bloco parte em dois commits.

**2. ⛔ O bloco D estava classificado ao contrário.** O plano diz *«grande e mecânico: 185 ficheiros,
o trabalho é volume, não risco»*. Medido: **zero renomes nos atingem** e tudo compila sem uma
alteração. O que sobra é **100 % comportamental e silencioso** — 5 sítios, nenhum com erro de
compilação. *Um bloco descrito como volume era risco puro.*

**3. ⛔ Três tarefas do bloco C mandam editar código que não precisa de mudar.**
- **C10** (`VertexState::buffers` viraria `Option`) — a linha do wgpu 29 é **byte a byte igual** à
  do 28. **34 edições que o plano pedia: zero.**
- **C13** (varyings inteiros precisariam de `@interpolate(flat)`) — a regra **já existia no naga 28**,
  e os 3 casamentos do grep são **atributos de vértice**, onde o atributo nem se aplica. Os varyings
  inteiros reais **já têm `flat`**.
- **C16/C17** (renomes de alinhamento e `peniko::Font`) — já tinham acontecido **antes** da versão
  que usamos hoje.

**4. ⛔⛔ O plano manda o dono procurar uma mudança que NÃO PODE ACONTECER.** Ele diz que o gradiente
do selector de cor muda de espaço de mistura. Verificado no gerador da rampa do vello: o modo novo é
**opcional**, o campo que o escolhe **já existia** na versão que usamos, nenhum sítio nosso o escreve,
e o caminho por omissão é **a mesma chamada**. O gradiente é **byte-idêntico**.
⚠️ *Mandar o dono do produto procurar o que não pode acontecer gasta a única coisa que ele tem de
escasso — a atenção dele — e ensina-o a não confiar na lista.*

### §8.3 — As cinco quebras de texto que o plano perdeu (bloco C)

Todas em `crates/ph2d-text/src/system.rs`, todas erro de compilação (logo, seguras):
`parley` deixou de re-exportar `swash` (usamos `swash::tag_from_bytes`) · `FontStack` foi **apagado**
(→ `FontFamily`) · `FontSettings<T>` partiu-se em dois tipos · `FontVariation.tag` mudou de `u32`
para um tipo próprio · `Layout::align` perdeu um argumento.
⚠️ E uma que **não** é erro de compilação: `Limits.max_*_buffer_binding_size` mudou de `u32` para
`u64`, o que faz o nosso `clippy -D warnings` reprovar em 2 sítios.

### §8.4 — ⭐ A regra que este passo comprova

**Um plano escrito a partir de changelogs descreve o que os autores acharam digno de anunciar —
não o que o NOSSO código encosta.** As quatro classes de erro que ele produziu são estáveis:
*(a)* mudança anunciada que **não** foi publicada (C10) · *(b)* regra antiga anunciada como nova
(C13) · *(c)* mudança real cujo alcance no nosso código é **zero** (C16, C17, 9 das 14 de D) ·
*(d)* a quebra que **ninguém anuncia** porque não é da biblioteca, é do encontro dela com o nosso
código (as 5 do texto, as 18 da física, e a causa do `linesweeper`).
⇒ *o custo de um bloco não se lê no changelog; lê-se no diff da superfície contra os nossos greps.*

## §7 — Diário

| data | bloco/tarefa | o que aconteceu |
|---|---|---|
| 2026-08-29 | — | plano escrito; `scripts/stack-audit.sh` criado; nada executado |
