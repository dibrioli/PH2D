# BLOCOS DE REABERTURA — W2 **FASE D** (4.ª rodada), 2026-09-12

> Um bloco colável por linha. **O §1 é comum e vale para as três.** Os números são medidos na árvore
> de hoje (`main` = shell **225 394** / 917 ficheiros), não copiados de handoff.
>
> Estado da obra: [`ESTADO_W2_2026-09-12.md`](ESTADO_W2_2026-09-12.md) ·
> molde: [`HOWTO_partir_uma_familia_da_shell.md`](HOWTO_partir_uma_familia_da_shell.md) ·
> rodada anterior: [`BLOCOS_REABERTURA_W2_FASE_C_2026-09-12.md`](BLOCOS_REABERTURA_W2_FASE_C_2026-09-12.md).

## ⭐ O que mudou desde a Fase C (leia antes de tudo)

1. **A shell perdeu 57,2 %** (526 809 → 225 394) e **cinco das sete famílias chegaram ao padrão de
   chegada** — o que sobra delas são invólucros de costura, que ficam por desenho.
2. ⭐⭐ **A régua do FECHO está no `main`**: `python3 scripts/fecho-da-familia.py <familia>`, com
   `--autoteste` (6/6 controlos). ⛔ **Corra o autoteste primeiro** — ela mentiu quatro vezes antes de
   ser instrumento, três delas *a favor* de mover demasiado.
3. ⛔⛔ **A catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` MORREU.** Uma família registada **declara
   roteador**. A única ausência aceite é a da família cuja CENA vive numa crate irmã
   (`FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA`), e ela é **verificada**, não tolerada.
   ⭐ **As duas famílias novas desta rodada TÊM roteadores próprios** ⇒ nenhuma precisa da excepção.
4. ⚠️ **O que sobra na shell já não é só «famílias».** Metade do peso está em cinco ficheiros de
   **composição** que ficam: `render_loop/mod.rs` (14 029 — o LAÇO), `input_dispatch.rs` (7 263),
   `app_state.rs` (1 902), `main.rs` (1 451), `render_loop/snapshots.rs` (1 398).

---

## §1 — AS DEZ REGRAS COMUNS

1. ⛔⛔ **Não toque no `TETO_LOC`** (hoje `229_394`, em
   `crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`). É do **integrador**,
   sobre a árvore combinada, **depois** do `cargo fmt --all`. ⚠️ Ele **vai** ficar vermelho na sua
   worktree pela metade de obsolescência: **é esperado, diga-o no handoff e siga.**
2. ⚠️ **Corra `cargo test -p ph2d-host-desktop --test it` À PARTE antes de dizer que fechou** — é
   barato (~800 testes) e é cinto e suspensórios. ⛔⛔ **Mas a razão que esta regra dava estava
   ERRADA** (medido pelo integrador em 12/09, ESTADO §4 lei 4): o `nextest-impacted` **alcança** a
   suíte `it` da shell — `rdeps(<família>)` selecciona os 793. Os vermelhos que as linhas acharam ali
   são reais; a atribuição ao script não era.
3. ⛔ **Uma agulha de gate ancora na LEI, nunca na VISIBILIDADE.** Atravessar a fronteira obriga
   `pub(crate) fn` a virar `pub fn`. Já mordeu **quatro** vezes (HOWTO §2.13).
4. ⛔⛔ **Um SEXTO método no `AppHost` = PARE e reporte.** As 5 portas serviram **sete** famílias em
   toda a W2, e a metade que mais parecia precisar de porta nova — o gesto de canvas e o inspector da
   `physics` — era **assinatura**. Antes de pedir porta, escreva em **TIPOS** o que a função precisa.
5. ⛔⛔⛔ **O PREFIXO NÃO É A FAMÍLIA — medido HOJE, e é a lei nova desta rodada.** O prefixo
   `layout_*` junta **dois assuntos diferentes**: `layout_live*` + `layout_reorder*` +
   `layout_scroll_gesture*` + `layout_smoke` são o **auto layout do Vetor** (ADR-0153), e
   `layout_persist*` é a **arrumação dos painéis do artista** (`~/.ph2d/layout.txt`), que é da shell e
   **não** sai. ⇒ *abra o cabeçalho de cada ficheiro antes de o mover;* um censo por prefixo levaria
   a arrumação de painéis para dentro do Vetor, e a falha seria **muda** (HOWTO §2.7).
6. ⛔ **Tudo DENTRO da SUA worktree.** `pwd` na dúvida — o mesmo caminho relativo existe no primário, e
   editar a árvore errada **compila e commita sem erro**.
7. ⛔ **Commits locais, `--no-verify`. NUNCA `push`, `--force`, `git add -A`, nem `git stash` nu** (a
   pilha é partilhada entre worktrees).
8. ⛔ **Fechar = gate batched 1× + handoff (DIRETRIZ §1.5.9) + PARAR.** Você não integra nem shipa.
   E reclame o disco: `rm -rf target/*/incremental`.
9. ⛔⛔ **O TERRITÓRIO É NOMEADO E DISJUNTO** (lei 7 do ESTADO, paga a 632 linhas que prenderam
   99 845). Cada bloco diz **o que é seu** *e* **o que é de outra linha**. Se precisar de algo fora do
   seu território, ⛔ **não o mova**: escreva-o no handoff com o nome do ficheiro e da linha dona.
10. ⚠️ **As três vão tocar os mesmos ficheiros de costura** — `app_state.rs`, `main.rs`,
    `input_dispatch.rs`, `render_loop/mod.rs`, `shells/desktop/Cargo.toml`, `Cargo.lock` e **uma linha
    do `CLAUDE.md`**. Conflito **em regiões diferentes** é o caso legítimo; no **mesmo símbolo** é
    violação de isolamento ⇒ **PARE e reporte**.
    ⚠️⚠️ **E uma LISTA em conflito não se resolve escolhendo um lado** — medido na Fase C: duas linhas
    escreveram duas listas com o mesmo aspecto e significados **opostos** no mesmo ficheiro, e a
    resposta certa não estava em nenhum dos dois lados (ESTADO §4 lei 8).

### ⛔ O que NÃO é de ninguém nesta rodada

| fica | porquê |
|---|---|
| `render_loop/mod.rs` · `input_dispatch*` · `app_state.rs` · `main.rs` · `snapshots.rs` | a **raiz de composição** e o LAÇO — ⛔ não abstraia o laço (HOWTO §4) |
| `project*` · `undo*` | persistência e a fila de undo, uma por desenho |
| `layout_persist*` | a arrumação dos painéis — regra 5 |
| `render_loop/inspector*` (24 f / 7 287 L) | ⚠️ é **chrome PARTILHADO**, não família. Sai numa wave com dono próprio, e o dono natural é a `ph2d-panel-inspector` |
| `timeline_*` · `render_loop/timeline*` · `autokey_*` (~7 866 L) | a **Timeline** — família própria, 5.ª rodada |
| `sheet_*` · `render_loop/sprite*` · `render_loop/anchor*` | o **Sprite** — 5.ª rodada |
| `shells/desktop/src/flip/` (36 f) · `src/physics/` (13 f) | ⭐ o **padrão de chegada**: são invólucros, e ficam |

---

## §2 — `line/app-vec` — **36 199 linhas** (a maior que sobra)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE D   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-vec.  Ela integrou TRÊS vezes (Fase A,
B, B2 e C) e o ramo está IGUAL ao main. ⭐ Você já alcançou o fim da linha
gateado — "vec" saiu da catraca, e a catraca em si já morreu.

⇒ O seu critério é o CORTE, e você é a ÚNICA família que ainda tem um número
  grande na shell: 36 199 linhas em 125 ficheiros.

⚠️⚠️ E ele é MAIOR do que o seu §11 dizia. O seu handoff media o prefixo
   `vec_*` (16 296) e a família tem SEIS prefixos a mais — a medição é do
   integrador, hoje:
       vec_*                  59 f   16 296 L
       fx_*                   16 f    4 095 L   (a pilha de aparência, o Trim, o Knot)
       morph_*                10 f    3 428 L   (a máquina de estados do morph)
       envelope_*              8 f    3 041 L
       bool_*                 10 f    2 458 L
       layout_*  (ver ⛔)      13 f    3 496 L   (o auto layout, ADR-0153)
       label_*                 3 f    1 170 L
       render_loop/vector*     6 f    2 215 L
       ─────────────────────────────────────
                             125 f   36 199 L

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec
   pwd && git branch --show-current     # DEVE dizer line/app-vec
2. git rebase main                      # deve ser no-op
3. cargo check -p ph2d-app-vec          # warm-up
4. LEIA: (a) ESTADO_W2_2026-09-12.md — as OITO leis, e a 8.ª nasceu de um
   conflito ENTRE a sua linha e a motion, no ficheiro do registo; (b) o HOWTO;
   (c) o §1 deste documento (as DEZ regras — a 5.ª é nova e é sobre VOCÊ);
   (d) o seu handoff da Fase C.
5. Reporte "vec pronta" e SIGA.

⛔⛔ A REGRA 5 DO §1 É SOBRE O SEU TERRITÓRIO, E A FALHA SERIA MUDA: dos 15
   ficheiros `layout_*`, DOIS não são seus — o `layout_persist.rs` e o
   `layout_persist_tests.rs` (883 L) são a ARRUMAÇÃO DOS PAINÉIS do artista
   (`~/.ph2d/layout.txt`), que é da shell. Os outros 13 são o auto layout
   (ADR-0153). ⇒ abra o cabeçalho de cada um antes de mover.

A TAREFA:
  PASSO 1 — RE-MEDIR O FECHO com a régua que agora está no main:
      python3 scripts/fecho-da-familia.py --autoteste
      python3 scripts/fecho-da-familia.py vec --extra 'fx_' --extra 'morph_' \
          --extra 'envelope_' --extra 'bool_' --extra 'label_' --extra 'layout_live'
    ⛔ Não mova produto antes. O número que sair é o plano.
    ⚠️ A sua sonda corrigiu-se TRÊS vezes na Fase B2 e QUATRO na Fase A, sempre
    a favor de mover demasiado. Use a régua, não a memória.

  PASSO 2 — o corte, pelo HOWTO, em fatias que compilam (uma por commit).
    ⭐ O molde é o que VOCÊ fez na Fase C: *lei pura ↔ ponte `impl App`*.
    ⚠️ `#[path]` é aresta DURA nos DOIS sentidos (o `vec_gizmo_view` DECLARA
    `#[path = "vec_gizmo_pick.rs"]`).
    ⚠️ E os 59 `vec_*` continuam SOLTOS no topo de `src/`: agrupá-los em
    `src/vec/` é o que torna o corte mecânico.
    ⚠️ Os SEIS `*_live.rs` que a Fase C deixou (`bool_live` 504 · `envelope_live`
    487 · `fx_live` 530 · `label_live` 455 · `layout_live` 575 · `skeleton_live`
    16) estão todos dentro dos prefixos acima — meça o fecho do GRUPO, não de um
    ficheiro: eles são o pipeline de geometria viva e provavelmente saem juntos
    ou não saem.

  O FIM DA LINHA: já alcançado ⇒ o critério é o CORTE, com `nextest-list-diff`
  exacto (ONLY-A = 0 e ONLY-B = 0) e a suíte `--test it` verde à parte.

⛔ NÃO É SEU: layout_persist* · instance_* / component_* (da `line/components`)
   · render_loop/painter* / paint* (da `line/app-painter`) · e tudo o que o §1
   lista como «de ninguém».

REGRAS: as dez do §1.
═══════════════════════════════════════════════════════════════════
```

---

## §3 — `line/components` — **~16 400 linhas** (as instâncias)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE D   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/components.  A worktree JÁ EXISTE.

⚠️⚠️ LEIA ISTO PRIMEIRO: o seu ramo está **355 commits atrás do main** e tem
   **ZERO commits próprios**. ⭐ Isso torna o rebase TRIVIAL (não há nada seu
   para reaplicar — é um fast-forward), mas significa que **tudo o que você
   sabe sobre a árvore está errado**: a W2 tirou 301 415 linhas da shell em dois
   dias, sete famílias saíram, e o seu próprio módulo ganhou o Top-20 (#2 Timer,
   #4 Áudio 2D, #5 SignalActions, #7 GameCamera) e a F4.6c/F8.
   ⇒ ⛔ **Não cite nenhuma nota de memória sobre ficheiros sem a conferir no
   código.**

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components
   pwd && git branch --show-current     # DEVE dizer line/components
2. git rebase main                      # 355 commits, ZERO seus ⇒ fast-forward
3. cargo check -p ph2d-host-desktop      # ⚠️ o 1º build desta worktree é FRIO
4. LEIA INTEIRO, nesta ordem:
   a) docs/IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md — a obra toda, e as
      OITO leis que ela pagou. ⛔ Sem isto você não sabe em que árvore está.
   b) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md — a LEI
   c) o §1 deste documento (as DEZ regras comuns)
   d) o handoff mais recente do seu módulo:
      docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_2026-09-10.md
5. Reporte "components pronta, N commits de rebase" e SIGA.

A TAREFA: **criar `crates/ph2d-app-components` e levar a família para lá.**
  O alvo medido HOJE:
      instance_*.rs              45 f   13 413 L
      component_*.rs              8 f    1 754 L
      + os roteadores de cena     ~7 f   ~1 240 L
        (timer_smoke · signal_smoke · signal_action_smoke · signal_table_smoke
         · audio_2d_smoke · camera_2d_smoke, com os `_tests` irmãos)
      ────────────────────────────────────────
                                ~60 f  ~16 400 L

  PASSO 1 — RE-MEDIR O FECHO (a régua está no main):
      python3 scripts/fecho-da-familia.py --autoteste
      python3 scripts/fecho-da-familia.py components --extra 'instance_' \
          --extra 'component_'
    ⛔ Não mova nada antes. ⚠️ E ela tem de seguir CAMPOS: o acoplamento que
    viaja por um campo de `App` não tem nome de módulo nenhum, e foi assim que a
    `motion` errou 30×.

  PASSO 2 — o corte, pelo HOWTO, em fatias que compilam.
    ⚠️⚠️ **A sua família tem uma armadilha que as outras seis não tinham: ela
    mexe no SNAPSHOT e no UNDO.** O `ProjectState::capture` e o
    `deep_copy_subtree` são o coração do seu módulo, e o `undo.rs`/`project.rs`
    ficam na shell por desenho (§1). ⇒ o corte aqui é *lei pura ↔ ponte*, com a
    LEI na crate e a CAPTURA na shell — e se a fronteira parecer exigir o
    contrário, **PARE e reporte com a tabela dos tipos** (regra 4).
    ⚠️ E o `MasterPiece` é **DERIVADO, nunca gravado** — um passe com duas
    metades obrigatórias (marcar E desmarcar); saltar a segunda deixa uma peça
    arrastada para fora do mestre INVISÍVEL ao solver, em silêncio.

  O FIM DA LINHA (gateado):
  · os roteadores que a shell lê pela sua família passam para a crate —
    CONTE-OS (forma `PH2D_*_SMOKE`);
  · o `const FAMILY` declara-os, com cada `max_level` CONTADO;
  · ⛔⛔ **a catraca das famílias sem roteador MORREU** — uma família que se
    registe sem declarar roteador REPROVA. A sua tem roteadores: declare-os.
  · `cargo test -p ph2d-app-registry-init` verde.

⛔ NÃO É SEU: vec_* / fx_* / morph_* / envelope_* / bool_* / label_* / layout_*
   (da `line/app-vec`) · render_loop/painter* / paint* (da `line/app-painter`) ·
   e tudo o que o §1 lista como «de ninguém» — em especial o `project*` e o
   `undo*`, que são a costura que a sua família mais toca.

REGRAS: as dez do §1.
═══════════════════════════════════════════════════════════════════
```

---

## §4 — `line/app-painter` — **~9 300 linhas** (LINHA NOVA)

```
═══════════════════════════════════════════════════════════════════
ABERTURA — Modo L · W2 FASE D   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-painter.  Ela é NOVA: a worktree foi criada
pelo operador em 2026-09-12 e está em `main`, limpa, sem commits próprios.

⭐ Por que ela existe: o Painter é o módulo com a MAIOR crate-motor do repo
   (`ph2d-tool-painter`, 136 093 LOC) e ainda assim tem 9 mil linhas da metade
   de composição dentro da `shells/desktop`. É a última família grande sem
   `crates/ph2d-app-<nome>`.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-painter
   pwd && git branch --show-current     # DEVE dizer line/app-painter
2. git status --porcelain                # deve ser vazio; já está em main
3. cargo check -p ph2d-host-desktop      # ⚠️ o 1º build desta worktree é FRIO
4. LEIA INTEIRO, nesta ordem:
   a) docs/IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md — a obra, e as OITO
      leis. ⛔ Você é a primeira linha desta wave que não as pagou: leia-as.
   b) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md — a LEI,
      e a §2 tem as armadilhas, metade delas com modo de falha MUDO
   c) o §1 deste documento (as DEZ regras comuns)
   d) o §5 do CLAUDE.md, módulo **Painter** — em especial a lei que aquele
      módulo pagou seis vezes (*o traço é fato do CAMINHO, nunca de quão fino o
      motor amostrou o caminho*) e o `DEPTH_UNIT_PX = 16.0`
5. Reporte "painter pronta" e SIGA.

⭐ VOCÊ TEM UM MOLDE PRONTO, USE-O: a `line/app-physics` fez exactamente este
  corte (Fase A o endereço, Fase B/C o corte) e o handoff dela nomeia os sete
  prendedores e como cada um caiu:
  docs/Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_C_2026-09-12.md
  O substrato é `crates/ph2d-app-host` (o trait `AppHost`, 5 métodos) e o registo
  gerado `crates/ph2d-app-registry-init` (`cargo run -p ph2d-app-sync`).

A TAREFA: **criar `crates/ph2d-app-painter` e levar a metade-shell para lá.**
  O alvo medido HOJE:
      shells/desktop/src/render_loop/painter*.rs   25 f   7 215 L
        (painter_bridge + 15 irmãos · painter_gpu_* · painter_preview_* ·
         painter_stamp_device)
      shells/desktop/src/paint*.rs                  3 f     672 L
      + os roteadores de cena                     ~12 f  ~2 086 L
        (impasto · wetpaint · mask · substrate · taper · line · brush_smoke ·
         brush_corner_smoke · paint_opacity_smoke, com os `_tests` irmãos)
      ──────────────────────────────────────────────────────
                                                  ~37 f  ~9 300 L

  PASSO 1 — RE-MEDIR O FECHO. ⛔ Não mova nada antes:
      python3 scripts/fecho-da-familia.py --autoteste
      python3 scripts/fecho-da-familia.py painter --extra 'paint' \
          --extra 'impasto_' --extra 'wetpaint_' --extra 'brush_' --extra 'mask_'
    ⚠️⚠️ **A pergunta é o FECHO, não a contagem de citações** — contar citações
    erra A FAVOR, e a `motion` pagou-a a **30×** (mediu 46 757 linhas movíveis
    onde a verdade era 1 573). A régua tem de seguir **CAMPOS** (`app.gfx.*`),
    que nenhuma varredura por nome de módulo vê.

  PASSO 2 — o corte, pelo HOWTO, em fatias que compilam (uma por commit).
    ⚠️ O LAÇO fica na shell — o `render_loop/mod.rs` garante a ordem do quadro;
    saem os CORPOS, e a shell chama `ph2d_app_painter::<mod>::<fn>` no ponto
    certo. ⛔ NÃO abstraia o laço.
    ⚠️ E esta família tem **GPU**: o `painter_gpu_*` e o `painter_stamp_device`
    tocam o device. Gates de GPU são `#[ignore]` e precisam de adapter — ⛔
    *skip gracioso não é verde*, e diga no handoff quais não correram.

  O FIM DA LINHA (gateado):
  · os roteadores passam para a crate — CONTE-OS na forma `PH2D_*_SMOKE`;
    ⛔ `PH2D_PAINT_PERF`, `PH2D_PREVIEW_DIAG` e `PH2D_PREVIEW_DUMP` são
    DIAGNÓSTICO e **não** entram no `FAMILY` (um roteador declarado diz ao dono
    que ele tem uma cena para ver);
  · o `const FAMILY` declara-os com cada `max_level` CONTADO no `match`;
  · ⛔⛔ **a catraca das famílias sem roteador MORREU** — registar-se sem
    declarar roteador REPROVA;
  · `cargo test -p ph2d-app-registry-init` verde.

⚠️ E duas coisas do módulo que o seu corte vai atravessar, nomeadas para não as
   descobrir a meio: a suíte do Painter **corre em DEBUG também** (precedente
   registado), e os `--ignored` querem `--test-threads=1` com a máquina calma.

⛔ NÃO É SEU: vec_* / fx_* / morph_* / envelope_* / bool_* / label_* / layout_*
   (da `line/app-vec`) · instance_* / component_* (da `line/components`) · e
   tudo o que o §1 lista como «de ninguém» — em especial o
   `render_loop/inspector*`, que é chrome partilhado.

REGRAS: as dez do §1. Em especial ⛔ TETO_LOC · ⚠️ `--test it` à parte ·
⛔ 6º método = PARE e reporte · ⛔ fechar e PARAR (você não integra).
═══════════════════════════════════════════════════════════════════
```

---

## §5 — Notas para o integrador

1. **Ordem medida** (churn de costura decrescente): **vec → components → painter.** A `vec` reescreve
   125 ficheiros e é a que mais toca costura; a `painter` vive quase toda no `render_loop/`.
2. ⛔ **O `TETO_LOC` vai reprovar depois de cada fusão.** É a metade de obsolescência, é esperado, e a
   ordem é **integrar → `cargo fmt --all` → medir → escrever**. Nenhuma linha lhe toca.
3. ⚠️ **Duas famílias NOVAS entram no registo** (`components`, `painter`) ⇒ o `ph2d-app-sync` corre no
   passo 2 do `foundational-integrate.sh`, e as duas **têm** roteadores próprios: se alguma se
   registar com `routers: &[]`, o gate reprova e a cura é declarar, não tolerar.
4. ⚠️ **Conflito numa LISTA não se resolve escolhendo um lado** (ESTADO §4 lei 8) — e depois de
   resolver, **releia a prosa em volta**: a fusão da Fase C deixou três frases a mentir que nenhum
   gate apanha.
5. **Ainda nada foi pushado** (`git rev-list --count origin/main..main`), e sobra **um** aviso de
   clippy a travar o `ship.sh`: `crates/ph2d-app-sculpt3d/src/keys.rs:33` (8 argumentos), cuja cura é
   mudança de desenho e cujo `#[allow]` está proibido por escrito.
