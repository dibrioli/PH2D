# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, a linha INTEIRA (2026-09-20 → 2026-09-25)

> **Este documento é para o AGENTE INTEGRADOR** (que só funde por ordem explícita do dono). Ele diz o
> que a linha toca, onde um merge pode doer, os números que somam entre linhas, a prova de fecho e os
> smokes. O *mecanismo* de cada wave está no diário:
> [`HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md`](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md)
> (§39–§43, o Wet Paint) e, arquivadas verbatim, as §1–§38 em
> [`docs/archive/docs-2026-09-24/painter/`](../../archive/docs-2026-09-24/painter/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md).

---

## §1 — Coordenadas

| | |
|---|---|
| ramo | `line/PainterWatercolor` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| base | **rebaseada sobre o `main` de 2026-09-25** (`20a630f1b`), sem conflitos |
| commits | **140** de trabalho + os do fecho (este handoff e a devolução do `CLAUDE.md`) |
| diff | `205` ficheiros, `+33 476 / −997` |
| crates novas | **uma**: `ph2d-pigment` (folha; depende só do `libm`, que já estava na árvore) |
| pacotes EXTERNOS novos | **nenhum** |

**O que a linha fez, numa frase por assunto** (os números estão no §6):
- **Aquarela:** a costura dura do retorno sobre o próprio traço (a reserva passou a ser um nível disputado);
- **A lei da tinta:** o Kubelka–Munk virou a folha `ph2d-pigment`, e o Digital passou a misturar por ela;
- **Composite Brush:** a pilha de camadas cresceu de 3 para 7, montada à mão, com ordem por traço e acumulação por camada;
- **Blur da pilha:** passou a ter núcleo de caixa em fatias;
- **Performance da aquarela** (ADR-0172/0173);
- **x86-64-v2:** o produto compila para esse nível de processador (ADR-0174, decisão do dono);
- **Wet Paint** (ADR-0175): depósito e bico por linhas, a absorção barata do escorrido, a região declarada do composite, o nascimento da sessão e o papel por linhas.

---

## §2 — Superfície de colisão (colada do `collision-surface.sh`, 2026-09-25)

**Contadores partilhados — todos com delta `0`:**

| contador | delta |
|---|---|
| `PROJECT_SCHEMA` (e a tripla `(160, 13, 22)`) | 0 |
| `VEC_SCENE_SCHEMA` | 0 |
| `FLIP_SCHEMA` | 0 |
| `DOC_VERSION` | 0 |
| `FIELD_DOC_VERSION` | 0 |
| os três registos de componentes (`103` / `104` / `104`) | 0 |

**Contratos congelados (§6): intocados** — `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs`.

**ADRs — ⚠️ os números SOMAM entre linhas e são PROVISÓRIOS:** a linha cria **0171 · 0172 · 0173 · 0174 ·
0175**. O `main` acaba em **0170**, e em 2026-09-25 **nenhuma outra worktree viva** tem um ADR acima de
0170 (conferido nas seis). Se alguém chegar primeiro, reconte — as citações a renomear:

| ADR | ficheiros que o citam |
|---|---|
| 0171 (o borrão de caixa da pilha em fatias) | 7 |
| 0172 (passagens por pixel da pilha em linhas) | 10 |
| 0173 (passagens da aquarela na equipa de threads) | 16 |
| 0174 (**o produto compila para `x86-64-v2`**) | 5 |
| 0175 (o depósito do dab do Wet Paint em linhas, + §3-bis o papel) | 14 |

`git grep -l "ADR-017[1-5]"` dá a lista; o índice `docs/architecture/decisions/README.md` é **derivado**
(`bash scripts/adr-index.sh`).

---

## §3 — Global e foundational tocados (e porque cada um é aditivo)

| sítio | o quê | nota |
|---|---|---|
| ⚠️⚠️ **`.cargo/config.toml`** | `[target.'cfg(target_arch = "x86_64")'] rustflags = ["-C", "target-cpu=x86-64-v2"]` | **ADR-0174, decisão do dono (23/09).** Vale para TODO build x86-64 do repo (Linux e Windows, e o CI); o macOS aarch64 fica fora. O resultado numérico é o mesmo byte (o `v2` não traz FMA). ⚠️ Uma variável `RUSTFLAGS` definida no ambiente **substitui** este nível em silêncio. |
| `crates/ph2d-pigment` (**nova**) | a lei do pigmento Kubelka–Munk, extraída do `ph2d-wet-paint` | o motor de fluido **delega** (byte-idêntico); o `painter-brush` e o `tool-painter` passam a depender dela |
| `crates/ph2d-i18n/src/painter_layers.rs` | `+11` chaves (fileiras novas do painel) | apendado |
| `crates/ph2d-editor-core/tests/it/hr12_widgets_a11y.rs` · `every_button_wears_the_live_hover.rs` | a cadeia de portas verificadas do HR-12 ganhou o **2.º salto** (§18 do diário) | gates, não produto |
| `shells/desktop/src/render_loop/fase_painter_brush_smokes.rs` | `+30`: o `PH2D_COMPOSITE_SMOKE` | a catraca `the_shell_only_shrinks` passou no fecho |
| `project-memory/` | 1 memória nova, 1 família estendida, 1 linha no `MEMORY.md` | ⚠️ ver §4 |
| `CLAUDE.md` | **nenhuma mudança** — ver abaixo | |

⛔ **O `CLAUDE.md` foi DEVOLVIDO ao estado do `main` no fecho.** Durante a jornada esta linha escreveu no
§5 do Painter, e o saldo era **uma linha de `~40 KB`** acrescentada ao parágrafo. Isso viola a regra (a
linha não edita o `CLAUDE.md`; só a integração) e colidia com **quatro** das seis linhas vivas, que
também mexem nele. A narrativa não se perde: está nas secções do diário, arquivadas verbatim. **A UMA
linha que esta linha propõe para o §5 está no §8.**

---

## §4 — Onde um merge textual pode doer (interseção medida com as linhas vivas, 2026-09-25)

| linha viva | ficheiros em comum com esta |
|---|---|
| ⚠️⚠️ **`line/UIUX`** (71 commits) | `crates/ph2d-panel-painter-layers/src/{brush_fallback,lib,paint_brush_rows,paint_watercolor}.rs` · `crates/ph2d-i18n/src/painter_layers.rs` · `crates/ph2d-editor-core/tests/it/hr12_widgets_a11y.rs` · `project-memory/{MEMORY.md,reference_topic_gate_discipline.md}` · `Cargo.lock` |
| ⚠️ **`line/sculpt3d`** (72 commits) | `crates/ph2d-tool-painter/src/{lib.rs,tool/mod.rs,tool/runtime.rs,tool/layers/preview.rs,tool/paint/watercolor_backdrop.rs}` · `project-memory/MEMORY.md` · `Cargo.lock` |
| `line/components` | `hr12_widgets_a11y.rs` · `project-memory/{MEMORY.md,reference_topic_gate_discipline.md}` · `Cargo.lock` |
| `line/3DModeling` · `line/motion-value` | só `project-memory/` e `Cargo.lock` |
| `line/Vector` | só `Cargo.lock` |

⚠️ **O painel do pincel é o atrito real com a `line/UIUX`.** Esta linha acrescentou fileiras e cartões:
- a pilha do Composite Brush, com o `+` e o dropdown, os `x` por camada e a quota;
- o cartão `Mixing`, com o `Pigment` a mudar de sítio;
- a caixa do escopo da borracha.

A `line/UIUX` reescreve a disposição e os rótulos de todo painel do app. Depois de fundir:
- corra os **censos de texto** (HR-15) e a **varredura de elisões** — um rótulo desta linha escrito contra a lei de espaçamento antiga aparece aí;
- corra o gate **`o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta`** (censo do fonte do painel): um cartão que perca uma fileira na fusão tem de continuar a declarar o `n_rows` certo.

⚠️ **O `runtime.rs` do Painter** tem aqui a porta `begin_undo_step` e o `absorb_foreign_writes_now`
intocados. Esta linha **não** os editou no fim (os marcadores de medição do §40 foram retirados antes do
commit); a colisão com a `line/sculpt3d` nesse ficheiro é de outras linhas do mesmo.

⚠️ **`project-memory/MEMORY.md`**: o primário tem hoje edições **não comitadas** nesse ficheiro. A linha de
índice desta linha é **uma**; junte-a à mão.

---

## §5 — Prova de fecho (2026-09-25, depois do rebase)

| portão | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **`18 337 / 18 337`** verdes (`10 595` saltados por não serem impactados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `bash scripts/check-standalone-optional.sh` · `bash scripts/check-workflow-packages.sh` | limpos |
| `cargo clippy` nas 9 crates tocadas, `--all-targets --all-features -D warnings` | limpo |
| `cargo machete` nas crates tocadas | nada morto |
| `bash scripts/censos-da-arvore-combinada.sh` | **`127 / 127`**, controlo do filtro `12 de 12` |
| `bash scripts/adr-index.sh --check` · `bash scripts/doc-index.sh --check` | em dia |
| `#[cfg(target_os` escrito ou movido | **nenhum** (conferido no diff) |
| `unsafe` novo | **nenhum** (as três ocorrências no diff são comentários e um `forbid(unsafe_code)`) |
| binário do smoke, depois do `rm -rf target/*/incremental` | 1.ª `cargo build -p ph2d-host-desktop --release` `1m 19s`; 2.ª **`Finished release … in 0.82s`, `rc=0`** |

⚠️ **Flakes de recurso a promover à lista do §5.0** (acto do integrador), as duas gates de RAZÃO do
Painter, que reprovaram no meio do fan-out desta jornada e passam sozinhas:

| gate | evidência |
|---|---|
| `tool::paint::mask::mask_gate_tests::the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` | 3/3 verde sozinha a `load ~7`, zero linhas do diff no módulo dela |
| `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` | §31.3 do diário arquivado (pedido de promoção já escrito lá) |

(O `the_mask_stroke_cost_does_not_follow_the_canvas` já está na lista.)

---

## §6 — Os números medidos (o que o dono sente)

Tela 4096², `release`. As medições de relógio desta máquina oscilam com a carga; cada linha diz em
que carga foi tirada, no diário.

| assunto | antes | depois | onde |
|---|---|---|---|
| Wet Paint, quadro a pintar, raio 100 / 250 / 400 | `12,5 / 31 / 45 ms` | **`3,8 / 8 / 10,5 ms`**, ao bit | §39 |
| Wet Paint, começo de pincelada sobre água a correr | `27–35 ms` (sob carga) | **`~9 ms`** | §40–§41 |
| Wet Paint, soltar o pincel (o commit) | `~6,5 ms` | **`~4 ms`** | §41 |
| Wet Paint, 1.º traço da sessão, sem papel | `~40 ms` | **`~18 ms`** | §42–§43 |
| Wet Paint, 1.º traço com papel do artista (tipos 1/2/5) | `309 / 230 / 126 ms` | **`43 / 35 / 28 ms`** | §43 |
| Composite Brush, a pilha da foto do dono (`1024²`, 6 camadas) | **`240`–`250 ms` por traço** (*«FPS cai para 1»*) | wave 1: `86 ms` por traço · wave 2: **`11,2`–`12,9 ms` por quadro** (*«mais que o dobro»*) | [doc 43](../43_a_pilha_do_composite_brush_ficou_rapida.md) §0 |

⚠️ **A memória do desfazer**:
- O §42 corrigiu uma regressão que o §41 tinha trazido: o 1.º traço de uma sessão guardava o canvas `Whole`, dois planos. Hoje há gate: guarda menos de ¼ de plano.
- A partir do §41 os commits do Wet Paint guardam a janela **DECLARADA** (a região do composite), não a exacta: é a política de todo commit declarado desta crate. A rede de DEBUG confere que a declarada contém a verdadeira, e a mutação `1 × 1` reprova seis gates do `wetpaint`.

---

## §7 — Leituras que o diff inverte

1. **`split_exact_within` NÃO é o `split` com dica.** O detector da absorção varre só a janela
   declarada mas devolve a janela **exacta**. O `split` com dica guardaria a declarada tal como veio, e
   uma declaração sobre bytes iguais faria a absorção disparar onde o caminho de sempre não dispara
   (há gate para isso).
2. **A absorção barata guarda a MESMA entrada que a cara**, campo a campo: o gate compara a pilha de
   undo inteira, o cursor e o livro de bytes das duas rotas, e prova que a barata correu. Não é uma
   aproximação.
3. **O `take_dirty` no nascimento da sessão não salta pintura nenhuma.** A premissa está gateada: o
   composite da folha inteira sobre uma grade virgem não muda um byte, com e sem o véu. E o descarte
   mora no 1.º lote do traço, **não** no nascimento, porque o `reconcile_facts` re-coze o papel depois
   dele.
4. **O tile do papel do motor (`~12 ms`) fica em série DE PROPÓSITO:** a caminhada das fibras consome o
   gerador aleatório em ordem, e as somas `f64` são a impressão digital do motor.
5. **O `x86-64-v2` não muda um resultado** (sem FMA). Muda a velocidade de `round`/`floor` em todo o
   código x86-64.
6. **O `CLAUDE.md` sem mudança é deliberado** (§3), não um esquecimento.
7. **A pilha do Composite Brush compõe por CAMADA acumulada, não por lote.**
   - `PH2D_COMPOSITE_REPLAY=1` volta à rota de replay (bissecção num campo, não no ambiente).
   - O Blur da pilha diverge **por desenho** (pior `114` de 255): uma passagem de raio `P·k` em vez de `P` passagens de raio `k`.
   - Brush, Erase e Smear saem iguais ao bit.

---

## §8 — A UMA linha proposta para o §5 do `CLAUDE.md` (módulo **Painter**)

> ⭐⭐⭐ **E a `line/PainterWatercolor` fechou em 25/09 (140 commits, smoke do dono aprovado em cada
> wave):**
> - a costura do retorno na aquarela;
> - a lei da tinta K–M na folha `ph2d-pigment` (o Digital mistura por ela; a aquarela não, recusa medida);
> - o Composite Brush de até 7 camadas, montado à mão, com ordem por traço, e o Blur da pilha em caixa;
> - o produto em **`x86-64-v2`** (ADR-0174);
> - o **Wet Paint** a `~8 ms` por quadro a raio 250 (era `~31`), com o começo da pincelada a `~9 ms` e a 1.ª da sessão a `~18 ms` (`126–309 ms` com papel → `28–43`);
> - ADRs 0171–0175, a recontar.
>
> [Handoff da linha](docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md)
> · [diário](docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md).

---

## §9 — Smokes (o dono já aprovou todos; ficam para quem integrar conferir a árvore fundida)

⚠️ O caminho é o da **worktree**. Depois de fundir, troque pelo do primário. `--release` porque são
smokes de performance.

1. **Wet Paint** (o começo de pincelada, o soltar, a 1.ª pincelada da sessão, com e sem papel):
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && env PH2D_WETPAINT_SMOKE=1 cargo run -p ph2d-host-desktop --release
   ```
   Pinceladas curtas seguidas sem esperar secar; Ctrl+Z várias vezes (cada pincelada some com o
   escorrido dela até a tela ficar em branco); Ctrl+Shift+Z traz de volta. Deu errado se sobrar
   mancha, se uma pincelada voltar pela metade, ou se a 1.ª pincelada com um papel escolhido travar.
2. **Composite Brush** (a pilha da foto do dono, já com o Painter na mão):
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && env PH2D_COMPOSITE_SMOKE=1 cargo run -p ph2d-host-desktop --release
   ```

**Régua de performance do Wet Paint, sem janela** (é o que produziu os números do §6):
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_o_wet_paint -- 4096 100 pousos 24 [papel]
```

---

## §10 — Aberto, e de quem é

**Decisão do dono** (medidas, com o preço no diário):
- os passos 3–4 do transfer do Wet Paint (Gauss-Seidel e soma `f64`; mudá-los muda a tinta);
- unificar a mistura seca (RYB) e molhada (K–M) na aquarela (a pista do *glaze*, §17.3);
- o `Mixing` no Impasto;
- o `Pigment` que mudou de sítio na aquarela;
- a pilha cheia numa tela grande (o **raio** é a alavanca, não a tela).

**Trabalho nomeado**:
- o **fork do canvas** no 1.º toque depois de soltar (`~9 ms` do começo da pincelada) — é o preço do undo com a tela inteira, e curá-lo pede canvas em ladrilhos (arquitectura);
- o tile do papel do motor (`~12 ms`, em série por impressão digital);
- o relevo fora da recomposição da pilha;
- o resíduo Blur+Smear (`12/255`);
- metade dos bytes dos intermédios da pilha.

**Do integrador**: recontar os ADRs, fundir a linha do `MEMORY.md`, promover as duas flakes, e aplicar a
linha do §8.
