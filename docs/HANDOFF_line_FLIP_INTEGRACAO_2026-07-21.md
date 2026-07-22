# Handoff de INTEGRAÇÃO — `line/FLIP` → `main` (2026-07-21)

> **Para o agente INTEGRADOR.** A linha está **fechada e smokada** — todos os smokes desta
> jornada foram aprovados pelo Enio (o último: *"smoke ok"*, sobre o ajuste ao vivo
> assíncrono). O implementador parou aqui (CLAUDE.md §0.7).
>
> **Base:** `Worktrees/line-FLIP`, branch `line/FLIP`, **31 commits à frente** do `main`.
> **`main` NÃO andou** desde o fork (`git rev-list --count HEAD..main` = **0**) ⇒
> **fast-forward limpo**, sem merge e sem resolução de conflito.
>
> ⚠️ Estes 31 commits **incluem** os do
> [`HANDOFF_line_FLIP_CONTINUACAO_2026-07-20.md`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-20.md),
> que era de *continuação*, não de integração. Este documento é o de INTEGRAÇÃO da jornada
> inteira do **Colorize**.

---

## 1. O comando (o caminho feliz)

```bash
cd /home/enio/Documentos/Projetos/PH2D           # a árvore PRIMÁRIA, não a worktree
git status --short                                # tem de estar limpa
git merge --ff-only line/FLIP
```

Se o `--ff-only` recusar, **PARE**: a `main` andou depois desta escrita. Vale a DIRETRIZ
§1.5.5 — resolva pelos **ESTÁGIOS do índice** (`:1` base, `:2` ours, `:3` theirs), nunca pelos
marcadores, e rode `cargo check --workspace` depois
([[feedback_clean_text_merge_can_be_semantically_broken]]).

**Depois do merge, rode o ship COMPLETO** (`./scripts/ship.sh`), não o `nextest-impacted`:
esta rodada mexe em **crate foundational** (`ph2d-flip-fill`) e o impacted já teve false-green
em RAM baixa.

⚠️ **Um `✗` do ship pode ser o AMBIENTE** e não o código
([[feedback_a_ship_x_can_be_the_environment_not_the_code]]) — confira o ESTADO antes de
"corrigir".

---

## 2. O que este delta entrega

**A wave COLORIZE fechou inteira** — as três fatias, todas smokadas:

| fatia | o que o artista ganha | smoke |
|---|---|---|
| **C1 — Trap** | *(já estava no `main`)* trapped-ball no balde | — |
| **C2 — LazyBrush** | **rabiscar cores** num line-art em vez de clicar região a região (a feature que só o TVPaint entrega): rabisco → **Apply** → regiões coloridas, com overlay ao vivo, paleta própria e sliders **Trap**/**Bleed** | ✅ |
| **C3 — onion fill** | com chaves marcadas na tira, **um Apply colore todas** — o rabisco atravessa as poses empilhadas | ✅ |

Mais **três defeitos de produto** fechados no caminho, dois deles em código que já estava no
`main`:

| defeito | onde vivia | registro |
|---|---|---|
| **A cor escapava por uma quina FECHADA na tela** | Colorize | [BUGS #23](Flip/BUGS_flip.md) |
| **O BALDE recusava (`Leaked`) numa caixa de quinas fechadas** — e *subir a Precision* quebrava o balde | `ph2d-flip-fill` (**já no `main`**) | [BUGS #23](Flip/BUGS_flip.md) |
| **O produto PANICAVA em build de debug** ao colorir; **2/31 testes falhavam sem `--release`** — que é o perfil do `ship.sh` | `ph2d-flip-colorize` | [09 §Estado 2026-07-21](Flip/09_colorize.md) |

E o **kill-criterion do §7.2, honrado**: o ajuste Trap/Bleed ao vivo saiu da thread de UI
(**304 ms/tique** medidos, 19× o orçamento; 1,45 s com zoom).

---

## 3. ⚠️ O que o integrador precisa saber ANTES de mesclar

### 3.1 `ph2d-flip-fill` é FOUNDATIONAL e a superfície pública CRESCEU

Três exports novos, todos **aditivos** (nada foi removido nem teve assinatura trocada):

```rust
pub use weld::{Weld, welds};                    // NOVO módulo — a solda das juntas
pub use dilate::{nearest_on_axis_indexed, ...}; // NOVO — a busca indexada (irmã da que já havia)
pub use dilate::outward_normals;                // NOVO
pub use raster::{BOUNDARY, FILLED, Grid, INK};  // BOUNDARY/FILLED/INK: antes só `Grid` era público
```

**Consumidores da crate no workspace:** `ph2d-flip-render`, `ph2d-flip-colorize`,
`shells/desktop`. Nenhum outro módulo do repo. **Risco de colisão de mesmo-símbolo: baixo** —
`welds` é nome novo no namespace da crate, e o `path_ops.rs` do vetor tem uma variável local
homônima que **não** conflita (namespaces distintos).

### 3.2 Uma MUDANÇA DE COMPORTAMENTO no balde, que já estava no `main`

O `fill_at` agora **solda as juntas** que a tinta do artista cobre. Isso muda o resultado de
uma arte que antes recusava — e é o fix. **Não é opt-in e não deve ser**: o default era a
recusa.

**A disjunção com o Gap Closure é por construção e está gateada** (a solda só dispara onde a
tinta já cobre; um vão deliberado tem as pontas longe e ela nunca o toca). O gate do C aberto
que já existia no shell **continuou verde sem ser tocado** — é a melhor evidência de não-regressão que esta linha tem.

### 3.3 Crate NOVA no workspace

`ph2d-flip-colorize` — o motor do LazyBrush, headless. **Sem dependência externa nova**
(`Cargo.lock` só ganhou a própria crate). `#![forbid(unsafe_code)]`.

### 3.4 O que NÃO mudou (e alguém vai perguntar)

- **Nenhum SCHEMA foi bumpado.** O Colorize não persiste nada próprio: os rabiscos são
  **sementes transientes** (fora do `ProjectState`) e o resultado são `FlipStroke`s comuns,
  que o formato já carrega. `PROJECT_SCHEMA`, `DOC_VERSION` e `VEC_SCENE_SCHEMA_VERSION`
  **intactos**.
- **Nenhum contrato congelado** (CLAUDE.md §6) foi tocado.
- **Nenhum ADR novo.** Se o integrador achar que a solda merece um, ela está inteira no
  BUGS #23 com a medição — mas ela não muda arquitetura, muda um raster.

### 3.5 Arquivos COMPARTILHADOS tocados (onde um merge futuro morde)

| arquivo | o que a linha fez | risco |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/flip.rs` | **+6 ids** (`FLIP_MODE_COLORIZE`, `FLIP_COLORIZE_*`) | **só ADIÇÃO** — a lista é compartilhada e ADICIONAR é seguro ([[feedback_a_shared_list_is_merged_against_todays_main]]) |
| `ph2d-editor-core/tests/node_id_collisions.rs` + `architecture_panel_wiring_parity.rs` | os ids novos entram nas listas | idem |
| `shells/desktop/src/undo.rs` | `post_frame_undo` pergunta `flip_colorize.live_busy(...)` ao lado do `held_button` | **1 linha lógica** num ponto quente do shell — conferir se outra linha mexeu no mesmo `if` |
| `shells/desktop/src/undo_route.rs` | `UndoOwner::Colorize` (rabisco pendente é dono do Ctrl+Z) | idem |
| `shells/desktop/src/render_loop/mod.rs` | drain do Apply/Clear + `flip_colorize_live_adjust()` no prólogo | adição no prólogo |
| `shells/desktop/src/{app_state,input_dispatch,main,render_loop/present,render_loop/flip_*}.rs` | costura do modo (estado, ponteiro, overlay, cursor) | adições |
| `ph2d-panel-flip/src/ids.rs` | re-export dos ids novos | **reordenação do `pub use`** pelo `fmt` — se conflitar, é textual puro |

---

## 4. Como VERIFICAR (o que rodei, e o que dá para reproduzir)

```bash
cd /home/enio/Documentos/Projetos/PH2D                     # DEPOIS do merge
cargo test -p ph2d-flip-colorize -p ph2d-flip-fill \
           -p ph2d-panel-flip -p ph2d-tool-flip \
           -p ph2d-host-desktop -p ph2d-editor-core --release
```

**Resultado na worktree: 1893 passaram, 0 falharam.** Detalhe:

| suíte | verdes |
|---|---|
| `ph2d-flip-colorize` | 33 (**+ em DEBUG também** — ver §5.1) |
| `ph2d-flip-fill` | 71 |
| `ph2d-host-desktop` | 903 |
| `ph2d-panel-flip` (seam) | 23 |
| `ph2d-tool-flip` | 16 |

Arch-gates verdes: `node_id_collisions` · `architecture_panel_wiring_parity` ·
`no_magic_numeric_in_widget_or_screens` · `no_tofu_glyphs_in_ui_strings` ·
`shell_files_respect_hr18_loc_cap` · `panel_files_under_loc_cap` ·
`the_colorize_scribble_crosses_the_selected_frames` (novo, 6 testes).

`cargo clippy --all-targets` limpo nas crates tocadas · `cargo fmt --all --check` limpo **no
pin 1.95** ([[feedback_ci_direct_lint_gates_and_fmt_skew]]).

### 4.1 As RÉGUAS (`#[ignore]`, rodam sob demanda)

```bash
cargo test -p ph2d-flip-colorize --release --test probe_live_cost -- --ignored --nocapture
cargo test -p ph2d-flip-colorize --release probes:: -- --ignored --nocapture
```

A primeira é a que **decidiu** o desenho assíncrono — e refuta a hipótese intuitiva de que
cachear o raster resolveria (o split é solve 76% · vetorização 18% · **raster 4%**).

---

## 5. As lições que este delta paga (leia antes de mexer no que ele tocou)

### 5.1 ⚠️ Rodar a suíte SÓ com `--release` esconde pânico

O `voronoi.rs` construía o array de vizinhos **eager**, e em `y == 0` o
`p.wrapping_sub(w) + 1` estourava. **O produto panicava ao colorir em build de debug** — o
caso normal — e **2/31 testes falhavam sem `--release`**, que é o perfil do `ship.sh`
(`ci-test`). Ficou invisível por 3 commits porque o brief da wave dizia para rodar com
`--release` (por causa do custo do corte). **Rode as duas.**

### 5.2 A parede é o EIXO, a arte é o CORPO (BUGS #23)

O raster das fronteiras é o eixo do traço (raio 0 — a âncora zoom-proof do BUGS #14), e o
preço nunca tinha sido nomeado: **dois traços cujos corpos pintados se sobrepõem folgado podem
ter eixos que não se tocam**. Medido numa caixa de mão: vão de 0,0045–0,0404 doc entre eixos,
contra **0,26 de tinta**. O comentário do `fill_at` **já descrevia o mecanismo e o declarava
aceitável**, roteando o artista para o Gap Closure — o mecanismo estava certo, o veredito não,
e ninguém o reconferiu quando a Precision subiu.

### 5.3 Um gate de FONTE pina forma, não comportamento

O arch-gate do fan-out (padrão do `Join` da física) **passou** por uma mutação `.take(0)` que
preserva o texto e neutraliza o laço. Por isso o laço foi **extraído** para uma função
dirigível sobre um `FlipDoc` headless. E procurar o *nome* de uma função deixava passar
`let _ = f(…)`: o gate pina a **frase inteira**, que é a ligação.

### 5.4 Um oráculo se re-MEDE, não se afrouxa (BUGS #24)

Um gate do 5º smoke media *espalhamento* do desvio e ficou 1,76 depois da solda. Medido: o
espalhamento **nunca separou** (são 1,76 × doente 2,06) e a correlação de forma separa **menos
ainda** (0,897 × 0,879). Quem separa por 3× é a **mediana**. ⚠️ Eu havia escrito no comentário
os números que *esperava* (0,98) antes de medir — o medido inverteu a conclusão.

### 5.5 Uma cena de smoke tem de ARMAR o gesto

A C3 só age com 2+ chaves marcadas, e marcar é Shift/Ctrl+clique na tira. A cena montava as
três chaves e **não marcava nenhuma** ⇒ o Apply coloria um quadro só, **indistinguível da
feature quebrada** (foi a dúvida literal do Enio). A cena agora arma **e imprime o que
montou**. Memória atualizada: [[feedback_ready_to_smoke_example]].

### 5.6 Um recálculo assíncrono pendente É um gesto em andamento

O `post_frame_undo` suprime enquanto o `held_button` está preso; um worker continua **depois**
de soltar. Sem a correção, um arrasto de slider viraria **dois** Ctrl+Z, e o primeiro
devolveria um estado que o artista nunca viu.

---

## 6. O que fica ABERTO (nomeado, não escondido)

| item | onde | gatilho |
|---|---|---|
| **Pré-segmentação por regiões** (perf a 4K) | [09 §7.1](Flip/09_colorize.md) | arte de 4K num Apply |
| **`trap_px` não sobrevive ao clamp de `MAX_SIDE`** | 09 §Estado | zoom ~10× com Trap alto (bola de 21,6 doc) |
| **O `reach` do Gap Closure precisa de 4× o vão** — e o slider é rotulado pelo *alcance* | [BUGS #23](Flip/BUGS_flip.md) | medido escrevendo o gate da disjunção; é ergonomia, wave própria |
| **A barra de progresso do ajuste ao vivo não é pintada** | 09 §Estado | deliberado (sub-segundo no caso comum; barra piscando a cada nudge disputaria a coluna com toasts) |
| **A exceção `rayon` para a EDT** | [09 §7.1](Flip/09_colorize.md) | as alavancas single-thread estão esgotadas e a tabela está pronta — **decisão do Enio, com ADR** |

**Backlog da linha** (cada item com spec pronta): [`01_plano_waves.md` §Deferidos](Flip/01_plano_waves.md).

---

## 7. Os smokes desta jornada (todos APROVADOS)

```bash
cd /home/enio/Documentos/Projetos/PH2D && \
  env PH2D_FLIP_COLORIZE_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena imprime `cena montada: 3 chave(s) em [0, 4, 8], 3 marcada(s)` — **se essa linha não
aparecer, pare**: o resto do smoke não significa nada.

| # | o que foi aprovado |
|---|---|
| 1-5 | o modo Colorize clicável, o overlay, o undo do rabisco, a borda que segue a linha |
| 6 | Trap/Bleed em tempo real · `Bleed 0` **sela** o vão · o selo não extrapola · o selo é degrau |
| 7 | a **solda das quinas** (a cor para na caixa no default) |
| 8 | **C3** (um Apply colore as três chaves) |
| 9 | o **balde** (arte de quinas fechadas volta a preencher, com Gap Closure e Trap em zero) |
| 10 | o ajuste ao vivo **assíncrono** (arrasto fluido; **um** Ctrl+Z desfaz o arrasto inteiro) |

---

## 8. Depois da integração

1. `./scripts/ship.sh` **completo**, e corrija todo `✗` antes de qualquer push.
2. **Push só por ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7) — a linha não pusha, o integrador
   não pusha por conta própria.
3. **Atualize a §5 do `CLAUDE.md`** com a entrada do Flip/Colorize: hoje ela não menciona a
   wave, e uma §5 que não descreve o que está no `main` faz a próxima LLM reconstruir o que
   existe (a lição literal que o módulo de áudio pagou).
4. Se houver outra linha na fila, **a ordem de integração se MEDE** pela sobreposição
   par-a-par ([[feedback_integration_order_comes_from_measured_overlap]]) — esta toca
   `ph2d-flip-*`, `ph2d-editor-core/ids/chrome/flip.rs` e a costura do shell (§3.5).
