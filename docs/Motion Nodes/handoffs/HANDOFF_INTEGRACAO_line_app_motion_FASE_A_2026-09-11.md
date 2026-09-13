# HANDOFF DE INTEGRAÇÃO — `line/app-motion`, **Fase A** (2026-09-11)

> W2/L1 do [briefing de partir a shell](../../archive/integracao-jornadas/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md)
> (§0–§3 e §5) · auditoria de velocidade [§4-C2](../../DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md)
> · DIRETRIZ §6.7.
>
> **Fase A fechada. A Fase B espera a `line/app-host` integrar** (ordem do Enio), e o §9 deste
> doc diz o que ela tem de cobrir — que é **menos** substrato do que o briefing supunha.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/app-motion` |
| último commit de CÓDIGO | `dd7255f04` (os seguintes são só docs) |
| merge-base com `main` | `8fa4f115b` |
| commits | **9** |
| ficheiros tocados | 433 |

---

## §2 — Foundational / partilhado tocado, e porquê

| ficheiro | o quê | porquê |
|---|---|---|
| `shells/desktop/src/main.rs` | 27 `mod motion_*;` → **1** `mod motion;` | A3 (agrupar) |
| `shells/desktop/src/app_state.rs` | **−4 campos, +1** (`motion_shell`) | A2b — os 4 eram lidos só pela família |
| `shells/desktop/src/input_dispatch.rs` | 2 campos renomeados (`motion_shell.*`) | consumidor do A2b |
| `shells/desktop/src/render_loop/mod.rs` | 7 chamadas do prólogo + renames | consumidor do A2a/A3 |
| `shells/desktop/src/render_loop/motion_*.rs` | renames de caminho | consumidores do A3 |
| `shells/desktop/Cargo.toml` | **+1 dep** (`ph2d-app-motion`) | A4 |
| `Cargo.lock` | +1 pacote **interno** | A4 |

⛔ **Nenhum contrato congelado encostado** (§6 do CLAUDE.md): `ph2d-nodegraph/src/node.rs` e
`ph2d-editor-core/src/tool.rs` **intocados**.

⭐ **Zero edição central para a crate nova** — o `members` da workspace é `crates/*` por glob.

---

## §3 — Símbolos que podem COLIDIR (saída do `collision-surface.sh`, 11/09)

```
SUPERFÍCIE DE COLISÃO — line/app-motion contra main
  merge-base 8fa4f115b   ·   6 commit(s)   ·   433 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6)          ambos intocados
▸ ADR       esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock    1 pacote novo: "ph2d-app-motion"  (INTERNO)
▸ MARCADORES DE CONFLITO           nenhum
▸ TETOS DE LOC                     nenhum arquivo da linha passa do teto
```

⭐⭐ **NENHUM contador partilhado se move**, e é por construção: esta linha **move código**, e
mover código não muda serialização. Se numa re-leitura algum deles divergir, alguém fez mais do
que a tarefa.

⚠️ **A tabela acima mede contra o `main` de 11/09.** O integrador **re-roda**
`collision-surface.sh` nesta worktree imediatamente antes de fundir (DIRETRIZ §1.5.3) — esta
serve para saber o que a linha *achava* que estava a tocar.

**Símbolos novos desta linha:**

- crate `ph2d-app-motion` (nome novo no workspace)
- `crate::motion` (módulo novo na shell) e `crate::motion::motion_shell_state::MotionShellState`
- campo `App::motion_shell`
- ⛔ **nenhum id / const / variant / token numerado** — nada que some entre linhas.

---

## §4 — Contratos congelados encostados

**Nenhum.**

---

## §5 — O que só o `ship.sh` apanha

- `cargo machete` — a crate nova declara **7** deps e usa as 7; não corrido aqui.
- `cargo deny` / `audit` — **zero** pacotes externos novos (o único `+name` do `Cargo.lock` é
  interno), logo o risco é nulo por construção.
- `typos` — **corrido**, limpo.
- `fmt` / `clippy --all-targets` — **corridos**, limpos (§7).

---

## §6 — A PROVA (§3 do briefing), com os números

### (a) Nenhum teste se perde — ⭐ `ONLY-A = 0`

```
antes: 22635 testes (22011 chaves) | depois: 22635 (22011)
MOVED (mesma chave, outro pacote/binário): 4
   a_changed_atlas_throws_the_stale_pixels_away_and_an_unchanged_one_does_not
   a_crop_returns_the_region_it_was_asked_for
   a_degenerate_region_returns_nothing
   the_end_of_the_frame_drops_the_atlas_and_keeps_the_crops
      ['ph2d-host-desktop::bin/ph2d-host-desktop'] -> ['ph2d-app-motion']
ONLY-A (perdidos): 0
ONLY-B (novos): 0
```

Os 4 `MOVED` são exactamente os testes do `motion_leaf_images`, que foi com a crate. **Contagem
idêntica** — e note-se que a poda de cenas do A1 **não custou um teste**: os 3 modos apagados não
tinham testes próprios, e o construtor que ficou órfão (`build_driven_offset_graph`) passou a
`#[cfg(test)]` em vez de ser apagado, porque o **gate** dele mede uma propriedade real.

### (b) Roteadores e níveis idênticos

- Inventário de env vars `PH2D_*_SMOKE|DEMO` em `shells/desktop/src`: **`diff` VAZIO** contra a
  base (109 dos dois lados).
- Níveis dos dois roteadores de cena (`motion_state_demo_router` + `_ciclos`): **idênticos literal
  a literal**; `MAX_DEMO_LEVEL` continua **114**.
- `PH2D_MOTION_OBJ_SMOKE`: **9** modos (12 − os 3 apagados de propósito, §8).
- Os gates `no_two_*_scenes_claim_the_same_level` continuam a correr (mudaram de pasta, não
  morreram).

### (c) A shell encolheu — ⚠️ **e é pouco POR CONSTRUÇÃO**

| | ficheiros | LOC |
|---|---:|---:|
| `shells/desktop/src` antes | 1 784 | **493 252** |
| `shells/desktop/src` depois | 1 783 | **492 726** |
| delta | −1 | **−526** (−0,11 %) |
| `crates/ph2d-app-motion/src` | 7 | 364 |

⚠️⚠️ **A Fase A NÃO PODE encolher a unidade de compilação, e isto não é um resultado fraco — é a
forma da tarefa.** Os 269 ficheiros do A3 mudaram-se **dentro** de `src/`: continuam na MESMA
crate, logo o `bin (check-test)` vê exactamente o mesmo código. Os −526 LOC são só o que de facto
**saiu**: A1 (−193, cenas apagadas) + A4 (−330, o módulo que foi para a crate) + os 2 ficheiros
novos. ⇒ *o número que o briefing quer ver mexer é da **Fase B**, e o §9 diz o que a destranca.*

**Leitura a frio da unidade da shell** (`cargo check -p ph2d-host-desktop --tests --timings -j 32`,
target novo):

| | `ph2d-host-desktop bin (check-test)` | total do workspace | `load` |
|---|---:|---:|---|
| antes (base + A1/A2) | **16,69 s** | 51,38 s | 7,12 → 10,30 |

⚠️ **A leitura «depois» NÃO foi tirada, e a razão é honesta:** as seis linhas da W2 compilaram a
máquina o dia inteiro (`load` entre **25** e **120**), e o CLAUDE.md §5.0 diz que *nenhuma leitura
de relógio desta workstation vale nada acima de `load ~5`*. A de cima já foi tirada a `load 7–10`
e só é citável porque **bate com a auditoria de 10/09 (16,8 s)** — as duas concordam a 0,7 %.
⇒ **a leitura par tira-se numa máquina calma**, e com −0,11 % de LOC a previsão é *sem diferença
mensurável*. Uma segunda medição sob carga provaria menos do que esta frase.

### (d) Gate de fecho

| gate | resultado |
|---|---|
| `cargo check -p ph2d-host-desktop --all-targets` | **0 erros, 0 avisos** |
| `cargo check -p ph2d-app-motion` | **0 erros, 0 avisos** |
| `cargo clippy -p ph2d-host-desktop -p ph2d-app-motion --all-targets -- -D warnings` | **limpo** |
| `cargo fmt --all -- --check` | **limpo** |
| `typos` | **limpo** |
| `scripts/doc-index.sh --check` | **✓ 19 índices em dia** |
| `scripts/nextest-impacted.sh` | **5 678 testes, 5 678 passaram, 0 falharam** (19 226 saltados) |

⚠️ A 1.ª corrida do `nextest-impacted` deu **2 vermelhos**, os dois causados por esta linha e os
dois da classe que o briefing nomeia — estão no §7, com a cura. A corrida acima é depois dela.

### (e) O binário do smoke fica COMPILADO (DIRETRIZ §1.5.9 item 9)

```
$ cargo build -p ph2d-host-desktop --profile smoke     # 1.ª
    Finished `smoke` profile [optimized] target(s) in 1m 51s
$ cargo build -p ph2d-host-desktop --profile smoke     # 2.ª — A PROVA
    Finished `smoke` profile [optimized] target(s) in 0.19s
    (linhas "Compiling": 0)
```

Feito **depois** do último commit e **depois** do `rm -rf target/*/incremental` (4,1 GB
devolvidos). É o perfil `smoke`, não `release` — o dono não paga build.

### (f) Smoke

O comportamento **não muda** — a extracção não toca produto. As cenas são as mesmas, com os
mesmos números (menos as 3 apagadas do §8). Comando no §11.

---

## §7 — O que o portão apanhou (e o `check` não via) — **QUATRO coisas**

1. ⭐ **`clippy::empty_line_after_doc_comments`** — apagar o módulo `osc` no A1 deixou um `///`
   seguido de linha vazia antes do `#[path]` do irmão. O `cargo check` passava **verde**. *É
   exactamente a razão de o gate batched existir* (DIRETIVA §3: verde-de-compilação vale ZERO no
   audit).
2. **`rustfmt` reexpandiu 15 ficheiros** — o rename `crate::motion_X` → `crate::motion::motion_X`
   alonga cada caminho e cadeias que cabiam numa linha deixaram de caber. Os 15 são todos
   ficheiros desta linha; nenhum alheio foi reformatado.
3. ⭐⭐⭐ **`every_demo_scene_ends_in_an_output_node`** — a classe que o briefing nomeia (*«gates que
   varrem `shells/desktop/src` por nome de família»*). Ele faz `read_dir("src")` à procura de
   `motion_state_conferencia_demos*.rs`; com a família em `src/motion/` **achava zero**. ⭐⭐ **E
   foi o CONTROLE POSITIVO dele que o disse**, com a mensagem já escrita para este dia: *«a
   varredura achou 0 cenas de conferencia — a familia mudou de nome e o gate está a medir nada»*.
   Sem aquele `assert!(scanned.len() >= 10)` ele teria ficado **VERDE a medir zero, para sempre**.
   ⇒ *é a única coisa que torna seguro um gate que varre por caminho — e é o molde para as outras
   cinco linhas.*
4. **`the_dispatch_wires_press_move_and_release`** — casa **texto** no `input_dispatch.rs`, e o A2b
   renomeou `self.motion_path_drag` → `self.motion_shell.path_drag`. *Um gate de texto segue o
   NOME, nunca a propriedade.*

⚠️ **Nenhum dos quatro era alcançável por `cargo check`**, e os dois últimos só apareceram na
corrida de testes — é o argumento inteiro do gate batched, numa linha que não mudou produto nenhum.

---

## §8 — As cenas apagadas (A1), com o roteador e o nível

| roteador | nível | o que era | porque pôde sair |
|---|---|---|---|
| `PH2D_MOTION_OBJ_SMOKE` | `=5` | vetor vivo CRISP (ADR-0154) | gate `a_live_vector_object_lowers_to_a_vector_instance_not_a_quad` |
| `PH2D_MOTION_OBJ_SMOKE` | `=6` | o freeze das 160k (report do dono, 05/08) | gates `the_lod_partition_cost_at_scale` + `the_lod_tile_lands_exactly_where_the_crisp_vector_would` |
| `PH2D_MOTION_OBJ_SMOKE` | `=10` | o gémeo da `=7` com o offset por FIO | a `=7` fica (mesma cena, offset autorado) |

**193 linhas de 100 352 da família — `0,19 %`, não as ~1 k que a ordem de 10/09 antecipava.**

⭐⭐ **O CENSO mediu o oposto do que a tarefa supunha: nesta família não há poda a fazer.**

- `PH2D_GPU_COOK_DEMO`: **114 de 114** níveis citados por um doc vivo ⇒ **zero cenas órfãs**.
- **0** ficheiros `motion_*.rs` órfãos (todos alcançáveis por declaração de módulo).
- ⚠️ **DUAS das três apagadas encodam um report/decisão do dono.** Saem porque a propriedade de
  cada uma tem **gate** que a mede — não porque não interessem. `git revert da5a9783d` devolve-as.

⚠️⚠️ **E o instrumento do censo apanhou TRÊS defeitos em si próprio antes de responder** — leia
isto antes de escrever o censo da sua família:

1. **A forma dominante de citação NÃO é `PH2D_GPU_COOK_DEMO=87`, é `` Cena **`=87`** ``.** Um
   censo que só procure a primeira **apaga cenas que o `CLAUDE.md` cita na linha seguinte**.
2. **Um `=NN` nu casa com coisas que não são cenas:** `TranslationX=0..Opacity=5` fabricou os
   níveis `1..114` de uma vez, e `` `=1..=56` `` leu-se como intervalo **aberto**. ⇒ o `=NN` só
   conta quando não vem colado a um identificador, e o intervalo aberto tem de excluir crase.
3. **Um `=NN` nu é AMBÍGUO entre roteadores da MESMA família**, não só entre módulos: o `=12` da
   doc 96 §1.4 é `MOTION_OBJ_SMOKE=12`, e a 1.ª leitura atribuiu-o ao roteador principal.

⚠️ **E uma leitura minha estava errada e foi corrigida:** eu ia registar que o `CLAUDE.md` promete
`PH2D_AUTOFIX_SMOKE=1..8` e o roteador só oferece `1..6`. **O `CLAUDE.md` está certo** — o braço
`_` delega os modos `7` e `8` a ficheiros irmãos. *Contar os níveis de um roteador pelos braços do
`match` subconta quando um `_` delega.*

---

## §9 — ⭐⭐⭐ O QUE A FASE B PRECISA (e é MENOS substrato do que o briefing supunha)

**Medido:** a família tem **`86 222` LOC que não tocam `App` nem `gfx`** — 229 de 269 ficheiros em
`src/motion/`, **134 de 143** em `src/render_loop/`. Mesmo assim só **um** módulo (`330` LOC) pôde
sair na Fase A. A razão **não é acoplamento à shell**; é a FORMA do grafo de módulos:

1. **Tudo pende de `MotionState`** — a subárvore dela tem **225 ficheiros / 50 775 LOC**, e **46**
   deles fazem `use super::*` sobre o namespace dela.
2. **`MotionState` GUARDA quatro tipos que vivem em `src/render_loop/`:** `VecPathStore`
   (`motion_shape_store.rs`), `PlantMemo` (`motion_lsystem_gen.rs`), `BandCache`
   (`motion_audio_gen.rs`), `TableCache` (`motion_table_gen.rs`). ⭐ **Os quatro módulos são PUROS
   — nenhum toca `App` nem `gfx`.** O que os prende é o **SÍTIO**, não a dependência.

⇒ ⭐⭐⭐ **O bloqueador da Fase B não é o trait de host: é mover `MotionState` e as quatro caches
JUNTAS.** Três daqueles módulos contêm **também** a função de publicação que precisa de `gfx`
(`publish`), então o corte parte cada um em **TIPO** (vem) e **PONTE** (fica).

**O que é genuinamente da shell nesta família** (a lista que a `ph2d-app-host` tem de cobrir) — e é
curta, porque o A2 já a reduziu:

| o que | quantos | nota |
|---|---|---|
| `App.gfx` | 54 usos | o renderer, de todos — é o host |
| `App.vec_entities` · `App.flip_entities` | 6 · 1 | de **outras** famílias (L4/L5) |
| `App.timeline` · `App.playhead` | 4 · 2 | partilhados |
| os 9 ficheiros-ponte de `render_loop/motion_*` | `3 205` LOC | `crate::audio`, `vec_entities`, `flip_entities`, `field_gizmo`, `vec_font`, `picker_smoke`, `pan_diag`, `modal` |

⛔ **Esta linha NÃO desenhou nenhuma porta nova** (regra do briefing: a extensão do substrato é
decisão do integrador). O acima é **medição**, não proposta.

---

## §10 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **«269 ficheiros moveram ⇒ a shell encolheu 269 ficheiros»** — **não**: eles foram para
   `src/motion/`, que **é** `shells/desktop/src`. A unidade de compilação é a mesma. Quem saiu da
   crate foram **2** ficheiros (A4) e **1** apagado (A1).
2. **«a família tem 0 `impl App`» (o censo do briefing)** — tem **10**. O censo contou `^impl App`
   e esta família escreve `impl crate::App`, que no repo é **3× mais comum** (212 contra 67). Os
   números corrigidos de **todas** as famílias estão em `2af357f5a`.
3. **«59 membros de `self.`»** — esse número **não mede acoplamento a `App`**: conta `self.` de
   qualquer struct do ficheiro (`self.cache`, `self.tiles`, `self.by_handle` são structs da própria
   família). Contados **dentro** dos blocos `impl App`, são **6 campos**.
4. **«os 243 `#[path]` tiveram de ser reescritos»** — **zero** foram tocados. Um `#[path]` resolve
   relativo ao directório do ficheiro que o contém, então mover o conjunto **inteiro** preserva-os
   por construção. Foi medido **antes** de mover.
5. **«`pub(crate) mod` nas 27 raízes alarga a visibilidade»** — **restaura-a**. Na raiz da crate um
   `mod` privado já é visível a toda a crate; um nível abaixo não é. Sem `pub(crate)`: 198 erros.
6. **«`motion_leaf_images` foi escolhido por ser pequeno»** — foi o **único FECHADO**. Ver §9.
7. **«a crate `ph2d-app-motion` é a família»** — hoje é `364` LOC de `100 352`. Ela é o **lugar**
   que a Fase B enche.

---

## §11 — Premissas minhas que a MEDIÇÃO derrubou

1. *«A poda das cenas rende ~1 k linhas»* → rende **193** (0,19 %). Nesta família a alavanca não é
   a poda.
2. *«O roteador oferece `AUTOFIX 1..6` e o `CLAUDE.md` mente»* → o `CLAUDE.md` está **certo**; o
   braço `_` delega `7` e `8`.
3. *«`motion_demo_legend` e `motion_object_bake_dims` são movíveis»* → **não**: o 1.º chama
   `motion_state::demo_router::build_level`, o 2.º importa de `motion_object_bake`, os dois a
   ficar. Chegaram a ser movidos e foram **devolvidos**. ⚠️ A régua perguntava *«só referencia
   módulos da FAMÍLIA?»* quando a pergunta é *«só referencia módulos DO CONJUNTO QUE MOVE?»* —
   *um conjunto movível tem de ser FECHADO sob o que referencia.*
4. *«A Fase A vai encolher a unidade da shell»* → **não pode**, por construção (§6c).
5. *«O substrato da L0 é o que destranca esta família»* → o que a destranca é mover `MotionState`
   **com** as quatro caches; o trait de host cobre a lista curta do §9.

---

## §12 — Ordem dos commits

```
da5a9783d  A1  as 3 cenas que nenhum doc cita
2af357f5a  A2a os 10 `impl crate::App` -> funções livres   (+ o censo corrigido das 6 famílias)
859a424a9  A2b os 4 campos de `App` -> uma struct da família
08178d50e  A3  a família agrupa-se em src/motion/
7b7554e99  A4  nasce a crate ph2d-app-motion
1fe1caab5  A3-docs  57 citações de caminho reapontadas por script
363d67c18  fmt + o doc-comment órfão (clippy -D warnings)
dd7255f04  os 2 gates que varrem por CAMINHO e por NOME, reapontados
a708282a9  o handoff (+ a prova do smoke)
```

Sem dependências cruzadas fora desta ordem.

---

## §13 — O que smoke-testar

Nada mudou de comportamento — a extracção não toca produto. O smoke é de **não-regressão**, sobre
as MESMAS cenas de antes:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-motion && env PH2D_GPU_COOK_DEMO=111 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **Não smokado por mim:** as 114 cenas uma a uma (o dono aprova por amostragem) e a cena `=114`,
que **já estava por smokar antes desta linha** (CLAUDE.md §5: o `motion.collide` dentro de uma
simulação).
