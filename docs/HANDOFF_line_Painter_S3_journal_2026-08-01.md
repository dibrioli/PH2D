# HANDOFF — `line/Painter` · S3: o journal vira a fonte do `before` do RELEVO

> **Para o agente que assume a linha.** Antes de ler qualquer código, execute a **FASE 0** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> — o bloco pronto para esta linha está no §0 abaixo.
>
> **Estado:** `line/Painter` @ `b882f5cc7`, árvore limpa, 18 commits nesta jornada, tudo verde
> (tool-painter 936/0 · wet-paint fingerprint 3/3 · painter-brush 280/0 · clippy limpo).
> **Nada pendente de smoke** — as duas coisas desta jornada que tocam o produto são uma fatoração
> bit-exata e um conserto de contador `cfg(test)`.

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

## §4 — O DESENHO (§5.58.2), e as três perguntas

O `cursor` e o `stroke_undo` respondiam três perguntas. Cada uma tem um consumidor com sítio exato:

| pergunta | consumidor | hoje | depois |
|---|---|---|---|
| qual é o lado `before` do delta? | `UndoEntry::split` (`undo_record.rs:33/66/71`) | `stroke_undo.relief` | **os tiles do journal** |
| alguém escreveu no meio? | `absorb_foreign_writes` (`undo_absorb.rs:44`) | `split(cursor, before)` | o **estado** do journal |
| de que estado o undo parte? | `UndoEntry::materialize` (`undo.rs:432`) | `cursor.relief` | o **plano VIVO** |

**O relevo ganha um caminho de delta PRÓPRIO** — é isso que torna a wave tratável. O canvas continua
pelo `split(before, after)` de dois snapshots (não há o que ganhar ali).

* lado **`before`** = os tiles do journal — **bytes que já existem**, sem cópia nova;
* lado **`after`** = a janela correspondente do plano VIVO;
* **janela** = a união dos tiles tomados (o journal já a conhece: `taken`);
* **materialização** aplica o patch ao plano VIVO — e o chamador **já constrói o vivo uma linha
  acima** (`absorb_foreign_writes_now` faz `snapshot_model()`), então a mudança de assinatura de
  `undo()`/`redo()` não acrescenta trabalho.

⚠️ **O guard de proveniência é o que mantém *lento nunca, errado jamais*:** se
`journal_describes_step_at(before.writes)` for falso, o relevo cai no caminho de hoje — que só existe
enquanto os snapshots carregarem os planos. ⇒ **a elisão é do caminho do TRAÇO**, nunca de
`snapshot_model()` em geral: uma edição de camada continua carregando os planos, é user-paced, e é o
**fallback VIVO** em vez de um ramo morto.

---

## §5 — A ORDEM de landing (a chave é o degrau 2)

| # | o quê | ganho | oráculo |
|---|---|---|---|
| 1 | ~~identificar o 4º dono~~ | — | ✅ feito (§3) |
| **2** | **`split_from_journal`** + a pergunta de escrita estrangeira pelo journal, **atrás do guard**, com os snapshots ainda carregando tudo | **zero** | **gate de IGUALDADE** contra o `split` de dois snapshots |
| 3 | a materialização parte do vivo (`undo`/`redo` recebem o vivo) | zero | os gates de undo/redo existentes |
| 4 | o journal sai do `cfg(debug)` **JUNTO** com o `cursor` e o `stroke_undo` largarem o relevo | **~9,6 ms/traço** | a sonda de donos vai a **1**; o fold cai |

⚠️ **O degrau 2 é a chave de tudo, e ele é byte-idêntico POR CONSTRUÇÃO:** o delta que ele produz tem
de ser **igual** ao que o `split` de dois snapshots produz. Isso é um **gate de igualdade**, não uma
promessa — e com ele verde o degrau 4 vira mecânico.

⚠️ **O degrau 4 não pode ser partido:** promover o journal sozinho paga **captura + fork** até o fork
morrer (regressão pura), **e** deixa metade da API do journal com o AUDIT como único consumidor ⇒
**dead-code que só aparece em `--release`** — a armadilha exata que a §5.25 já pagou (4 warnings por
3 commits, porque `cargo clippy -p` roda em **debug**).

⚠️ **O AUDIT fica gateado.** Ele é a rede que confere a troca, e *uma rede de verificação não pode
viver no relógio do que ela observa* (§5.23). A promoção é do **journal**, nunca do audit.

---

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
