# HANDOFF — `line/Painter` · S3: o journal vira a fonte do `before` do RELEVO

> **Para o agente que assume a linha.** Antes de ler qualquer código, execute a **FASE 0** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> — o bloco pronto para esta linha está no §0 abaixo.
>
> **Estado:** `line/Painter`, árvore limpa, tudo verde (tool-painter 939/0 debug e release ·
> wet-paint fingerprint 3/3 · LOC workspace 2/2 · LOC shell 2/2 · contrato congelado 4/4 · clippy
> `--release --all-targets` limpo). **Nada pendente de smoke** — tudo o que landou é byte-idêntico.
>
> ✅ **DEGRAUS 2 E 3 ESTÃO CONSTRUÍDOS E GATEADOS** (doc 28 §5.59). Eles não compram um milissegundo:
> é o desenho deles. O que sobra é o **degrau 4**, e ele **não é mecânico** — a construção achou três
> coisas que o §5.58.2 não previa, e as três estão medidas na §5.59. **Comece pela §4 deste doc.**

---

## §0 — O bloco da FASE 0, já preenchido

```
Sua linha: line/Painter
cd Worktrees/line-Painter && pwd && git branch --show-current
   → tem de terminar em /Worktrees/line-Painter, branch line/Painter
git log --oneline -5 && git status -sb
git rebase main            (obrigatório no início da jornada)
cargo check -p ph2d-tool-painter
```

⚠️ **A cwd do Bash VOLTA ao primário entre chamadas.** Esta sessão perdeu uma medição inteira por
isso (o `cargo` foi tentar `/home/enio/Documentos/Projetos/PH2D/target`, que é um symlink para um
tmpfs evaporado). **Prefixe TODO comando com o `cd` da worktree** — não é conselho, é o desenho da
bancada.

---

## §0.5 — O QUE JÁ ESTÁ FEITO (não reconstrua)

| degrau | estado | onde |
|---|---|---|
| 1 · identificar o 4º dono | ✅ | §5.58.1 — são **três** (tool · `stroke_undo` · `cursor`) |
| **2 · `split_from_journal`** | ✅ **construído, 5 gates, 7 mutações, 7 sangram** | `undo_delta_journal.rs` (filho novo) · `journal_delta_tests.rs` |
| **3 · a materialização parte do VIVO** | ✅ **construído, net + mutação** | `PlaneDeltas::side` ganhou duas bases |
| 4 · a elisão + a promoção | ⛔ **redesenhado** — ver §4 | — |

**Duas correções ao plano, as duas achadas por gate vermelho:**

1. ⚠️ **`UndoEntry::materialize` tem DOIS chamadores com adjacências diferentes** (a tabela do §5.58.2
   tratava-o como um): `undo`/`redo` são adjacentes ao AGORA (base = o vivo); `absorb_foreign_writes` e
   a extensão de run coalescido reconstroem contra o cursor ANTIGO (base = o cursor) — e no `absorb` o
   vivo difere do cursor **por construção**.
2. ⚠️ **A caixa de tiles do journal é 128-alinhada**, e sozinha ela levou o passo típico de **2,51 para
   8,23 MB a 1024²** com os endpoints materializados idênticos. Ela é cruzada com a janela DECLARADA
   (dois superconjuntos). *Conteúdo e memória são perguntas separadas.*

---

## §1 — A TAREFA, numa frase

**Fazer o journal por tile ser a fonte do lado `before` do delta dos três planos de RELEVO**, para
que o `cursor` e o `paint.stroke_undo` possam largá-los — e com isso o fold do pen-up
(`commit_stroke_height`) pare de forkar 64 + 112 + 16 MB a cada traço.

**O prêmio, medido hoje na máquina calma (4096², impasto):**

| | ms |
|---|---|
| fold (`commit_stroke_height`) | **11,92** — dos quais **9,61** são o fork dos três planos |
| `record_structural` (o commit) | 10,95 — dos quais **9,23** são `PlaneDeltas::split` |
| pen-up TOTAL | **23,96** |

⚠️ **O alvo desta wave é o fork (9,61).** O `split` (9,23) é a outra metade do pen-up e **não** cai
com ela — fica nomeado, não prometido.

---

## §2 — ⛔ O que NÃO refazer (medido e fechado; re-derivar custa uma sessão cada)

Tudo abaixo está em [`docs/Painter/28_otimizacoes_o_que_funcionou.md`](Painter/28_otimizacoes_o_que_funcionou.md).
**Leia §5.20 · §5.22–§5.29 · §5.58** antes de escrever uma linha.

1. **Os pré-requisitos estão TODOS construídos e gateados.** O `TileJournal` com captura paralela e
   caminho contíguo (§5.25), a proveniência `journal_since` (§5.26), o oráculo `absent` vs
   `incomplete` (§5.29), a absorção nos DOIS consumidores (§5.24) e a identidade do cursor medida em
   233/233 (§5.28). **Não reconstrua nada disso.**
2. ⛔ **`mark_dirty` não serve como janela declarada** (§5.17): ele diz onde a IMAGEM mudou, não onde
   BYTES foram escritos. Foi construído e revertido, com a rede disparando na 1ª rodada.
3. ⛔ **O guard com `Drop` não compila** (§5.18): `Drop` estende o empréstimo ⇒ 14 `E0499`. A resposta
   é um CONTADOR (`declare_wrote`), que já existe.
4. ⛔ **Materializar o cursor a partir do journal é circular** (§5.58.2, atalho 1): produz o `Vec` que
   a wave existe para não pagar, em todo `record_*` **e** todo undo/redo.
5. ⛔ **"Materializar só quando o journal não está vazio" é falso na cena que importa** (§5.58.2,
   atalho 2): durante um traço o journal está SEMPRE cheio — o fold escreveu.
6. ⛔ **Paralelizar a extração da janela não é o gargalo** (§5.16): o `split` é limitado por LARGURA
   DE BANDA lendo os dois endpoints, não pelo `Window::extract`.

---

## §3 — O MAPA dos donos (fechado nesta sessão, §5.58.1)

Sonda: `who_holds_the_planes_when_a_stroke_begins`
(`cargo test -p ph2d-tool-painter --release --lib -- --ignored --nocapture who_holds`).

```text
REGIME (2 traços commitados, nenhum gesto)   canvas 2 · heights 2 · covers 2 · mats 2
DENTRO do gesto (logo após o pen-down)       canvas 1 · heights 4 · covers 4 · mats 4
  - sem o snapshot de pen-down               canvas 1 · heights 3 · covers 3 · mats 3
  - …e sem o histórico                       canvas 1 · heights 1 · covers 1 · mats 1
```

| dono | onde nasce | como sai |
|---|---|---|
| **o tool** | os campos `heights`/`covers`/`mats` | irredutível — é o produto |
| **`paint.stroke_undo`** | `capture_shape_model()` no pen-down (`tool/paint.rs:215`) | elidir o relevo **no caminho do traço** |
| **`cursor`** | `set_cursor(after.clone())` **ANTES** do `split` (`undo_record.rs:32/63/70`) | elidir o relevo |
| a entrada do **1º traço** | `StoredPlane::Whole` (o 1º traço CRIA o relevo ⇒ não há `before`) | **não precisa sair** — transiente por camada |

⚠️ **O canvas NÃO é o problema dentro do gesto** (já está em 1 dono: o pen-down forkou e o tool ficou
sozinho). **Só o relevo.** É por isso que a wave é relevo-first.

⚠️ **É tudo-ou-nada** (§5.14): `make_mut` copia com qualquer coisa acima de um ⇒ remover UM dos dois
donos removíveis **não compra milissegundo nenhum**. Os dois saem juntos, e os três planos juntos.

---

## §4 — ⛔ O DEGRAU 4 NÃO É MECÂNICO — a elisão joga fora a ENTRADA do fallback

A política que sustenta todo o S3 é *lento nunca, errado jamais*: o guard recusa e o commit **cai no
caminho de sempre**. ⚠️ **Ela não sobrevive à elisão**, e o motivo é de ordem no tempo:

* o guard é perguntado no **COMMIT**;
* a elisão acontece no **PEN-DOWN** (é ela que tira o dono — é ela que É a wave);
* o caminho de sempre precisa de `before.relief`, **que a elisão acabou de descartar**.

⇒ Num passo em que o guard recuse, o `split` clássico vê `before` sem relevo e `after` com, devolve
`OnlyAfter`, e **desfazer aquele passo REMOVE o relevo** em vez de o restaurar. Em silêncio. Mesma
classe do buraco que o `mats` fora do `ModelSnapshot` custou em 2026-07-13.

**Logo: o guard tem de ser decidível no pen-down, ou o commit tem de RECUSAR** (descartar o histórico,
como o `side` já faz com um cursor incoerente). Não há terceira saída — um `Whole` com `before = after`
é a mesma perda vestida de delta.

### 📊 O CENSO que redesenha o guard (`PH2D_UNDO_AUDIT=1`, a suíte `--lib` inteira)

| estado do journal de relevo | passos |
|---|---|
| **DESCREVE** (`relevo/PASSO`) | **307** |
| **SEM-RELEVO** | **467** |
| MISTURADO | **0** |
| INCOMPLETO | **0** |

1. **Nenhum passo da suíte é indescritível** ⇒ a política de *recusar no commit* é barata de verdade.
2. ⚠️ **A MAIORIA não escreve relevo, e hoje o guard os RECUSA.** O `speaks_for` exige
   `layer == Some(l)`, então um passo que não tocou relevo cai no fallback — que, com o `before`
   elidido, é o caminho que perde a edição. *"Este passo não escreveu relevo"* é uma descrição boa e o
   journal já a tem: **o guard tem de aceitar `layer.is_none()` e responder `Unchanged` em toda
   chave.** Sem isso a elisão quebra em **467 dos 774** passos.
3. O `absorb` ganha a metade dele de graça: os journals estão **intactos** quando ele roda (o
   `begin_undo_step` só os zera *depois* dele), então *"escreveram relevo no intervalo?"* é
   `relief journals vazios?`.

---

## §5 — A ORDEM do degrau 4, revisada

| # | o quê | por quê |
|---|---|---|
| 1 | o guard aceita **"nada escrito"** (`layer.is_none()` ⇒ `Unchanged` em toda chave) | 467 dos 774 passos; **gate próprio** |
| 2 | `ModelSnapshot::without_relief` + os dois sítios de elisão (`stroke_undo` no pen-down · o `cursor`) | é a wave |
| 3 | o `absorb` pergunta o relevo ao **journal** e, no re-split, **adota o delta de relevo da entrada velha** | o escorrido é do CANVAS; o relevo do topo não mudou — **conferido**, não assumido |
| 4 | a mesma cirurgia na extensão de run coalescido | mesmo chamador de `materialize` |
| 5 | o commit **RECUSA** quando o guard falha sobre um `before` elidido | não há fallback a que cair |
| 6 | o journal sai do `cfg(debug)` | **junto**, nunca antes (§5.58.1) — e o `expect(dead_code)` do `ReliefSource` vira erro e obriga a removê-lo |
| 7 | os gates do §6 | a sonda de donos vira gate |

⚠️ **Tudo-ou-nada segue valendo** (§5.14): `heights/covers/mats` estão a **3** donos dentro do gesto
(medido hoje), e só as duas elisões juntas levam a **1**.

⚠️ **O prêmio segue lá, e o número é do handoff, não desta sessão:** o `what_the_two_halves` mediu o
fold em **12,33 ms a 4096²** contra os 11,92 do §5.58 — mesma ordem, mas com `load average 22`, e
**nenhum relógio desta máquina vale nada acima de ~5** (§5.49). Re-meça com a máquina calma antes de
citar qualquer ganho.

## §6 — Os gates que a wave deve deixar (red-first, com mutação)

1. **igualdade** — `split_from_journal(journal, live)` == `split(before, after)`, plano a plano, com
   fixture que atravessa o `PAR_MIN` e cobre os três tipos (`f32`, `u8`, `[u8;7]`);
   *mutação*: trocar a origem da janela ⇒ RED.
2. **proveniência** — journal ancorado num passo mais VELHO ⇒ cai no caminho de hoje;
   *mutação*: ignorar `journal_describes_step_at` ⇒ o undo devolve pixels de outro passo.
3. **donos** — depois do degrau 4, `heights/covers/mats` a **1** dentro do gesto (a sonda do §3 vira
   gate); *mutação*: devolver a elisão ⇒ volta a 3.
4. **o fold** — razão entre duas telas (o fold é limitado pela PEGADA depois da troca) **e** kill de
   wall-clock; ⚠️ **afirme a RAZÃO primeiro** — a ordem é load-bearing (a lição do `warp_perf`).
5. **comportamento** — pinte, desfaça, refaça: a tinta **e o RELEVO** voltam iguais, em 1 e 2 camadas
   (a 2ª derrota o `doc_is_disposable`).

⚠️ **A cadeia só é observável a partir do 2º passo** (§5.14): um delta sozinho está sempre certo; o
que pode estar errado é a BASE. Toda fixture de undo tem de ter **dois** passos, em lugares
**diferentes**.

---

## §7 — Como medir (as sondas já existem)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo test -p ph2d-tool-painter --release --lib -- --ignored --nocapture who_holds          # os donos
cargo test -p ph2d-tool-painter --release --lib -- --ignored --nocapture what_the_two_halves # fold vs commit
cargo test -p ph2d-tool-painter --release --lib -- --ignored --nocapture what_a_plane_fork   # o fork por plano
cargo test -p ph2d-tool-painter --release --lib -- --ignored --nocapture what_the_record_structural
PH2D_UNDO_AUDIT=1 cargo test -p ph2d-tool-painter -- --nocapture 2>&1 | grep S3-AUDIT        # a rede
```

⚠️ **Nenhum número desta máquina significa nada com `load average` acima de ~5** (§5.49). Confira
`cat /proc/loadavg` antes de acreditar num relógio, e prefira **A/B costas-com-costas dentro da mesma
corrida** — esta worktree divide 32 núcleos com outras linhas, e o MESMO passo já foi de 14,5 a 30,2
ms sem uma linha de código mudar.

---

## §8 — Fechamento

* `cargo test -p ph2d-tool-painter` em **debug E release** (um gate desta linha já reprovou só em
  debug — um bar de relógio mede o PERFIL do build);
* `cargo clippy -p ph2d-tool-painter --release --all-targets` (⚠️ `-p` sozinho roda em **debug** e
  esconde dead-code de release);
* `cargo test -p ph2d-wet-paint --release --test fingerprint` (3/3 — a troca não pode movê-lo);
* o gate de LOC da shell (`shells/desktop/tests/file_loc_caps.rs`) e o
  `architecture_workspace_file_loc_cap` **isolados** — eles não correm no `cargo test -p` filtrado, e
  esta linha já shipou dívida vermelho-latente por isso;
* **NÃO integre e NÃO faça ship** (CLAUDE.md §0.7): feche, escreva o handoff de integração
  (DIRETRIZ §1.5.9) e PARE.

**Contrato congelado e schema:** esta wave **não deve tocar nenhum dos dois** (`PROJECT_SCHEMA` no
`main` de hoje é **48**; `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`). Se você
achar que precisa, **PARE e reporte ao Enio** — é ADR.

---

## §9 — O que mais está aberto nesta linha (não é da wave; não misture)

* **Semear os planos da luz no `prewarm`** — vale **12,7 ms** medidos no 1º traço com impasto, ao
  preço de **VRAM canvas-sized em TODO bind**, inclusive de quem nunca liga o impasto. **Decisão de
  produto do Enio**, com o número na mesa (doc 28 §4.8.2).
* **A borda da MÁSCARA que endurece** (doc 25 §13.10.4) — as **duas** leis de acúmulo possíveis já
  foram tentadas e cada uma tem artefato (produto = endurece · envelope = contas). A próxima hipótese
  tem de estar noutro lugar: o overlay, os defaults do pincel de máscara, ou aceitar.
* **A fatoração da COLUNA no upsample do composite** — ~1,2× sobre ~9 ms nas razões > 1, **zero na
  razão 1** (o default toma o caminho de identidade). **Nomeada e não construída de propósito**: é
  estritamente menos trabalho e não compra nada que o artista veja (doc 28 §5.56).
* **A memória da entrada do 1º traço** — ela retém o plano INTEIRO (`Whole.after`): a 4096² são
  **192 MB numa única entrada**, contra 2,36 MB de um passo típico. Não custa milissegundo; custa o
  cap de bytes do histórico (§5.58.1).
