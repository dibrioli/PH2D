# W2 — os três blocos de REABERTURA (motion · vec · flip), 2026-09-12

> ⛔ **CORRECÇÃO de 2026-09-12 (integrador):** a regra deste bloco que diz que o `nextest-impacted`
> *«filtra»* / *«NÃO alcança»* `shells/desktop/tests/it/` está **refutada por medição** — ele alcança
> (`rdeps(ph2d-app-vec)` → 793 testes). Ver [`ESTADO_W2_2026-09-12.md`](ESTADO_W2_2026-09-12.md) §4
> lei 4. O bloco fica como registo do que se instruiu; não o siga nesse ponto.

> **O estado completo está em [`ESTADO_W2_2026-09-12.md`](ESTADO_W2_2026-09-12.md)** — os blocos
> abaixo mandam lê-lo, e ele é a primeira coisa que uma janela nova tem de abrir.
>
> ⭐ **A estrada está livre.** As sete folhas partilhadas que as três famílias nomearam
> independentemente saíram da shell em 12/09 (`line/shell-folhas`). O que cada uma disse que a
> prendia **deixou de estar lá**.
>
> ⚠️⚠️ **E isso NÃO é uma promessa de que tudo sai agora.** A régua de um bloqueio é o **FECHO**,
> nunca a contagem de citações — a `motion` mediu `46 757` onde a verdade era `1 573`. O primeiro
> passo de cada uma das três é **re-medir o fecho**, e o número que sair é o plano.

| linha | na shell | pasta própria | `render_loop/` |
|---|---:|---|---|
| `line/app-motion` | **99 915** | 268 f / 60 852 | 143 f / 39 063 |
| `line/app-vec` | **28 204** | 92 f / 25 989 | 6 f / 2 215 |
| `line/app-flip` | **18 845** | 64 f / 16 030 | 10 f / 2 815 |

⚠️ **Uma indicação, e só isso:** contando em CÓDIGO (sem comentários), quantos ficheiros de cada
família ainda mencionam `App`/`gfx` — `motion` **14 de 268** · `vec` **15 de 92** · `flip` **36 de
64**. ⛔ **Isto NÃO é o fecho** (é a régua da lei nº 1 do estado, a que erra a favor); serve para
dizer *onde começar a olhar*, nunca *quanto sai*.

---

## §1 — AS REGRAS que valem para as três

1. ⛔⛔ **NÃO toque no `TETO_LOC`** do `the_shell_only_shrinks`. É um número que soma entre linhas:
   com três a escrevê-lo, o merge fica com um e nenhum está certo. **O gate VAI reprovar na sua
   worktree** assim que tirar código — *isso é o seu marcador de progresso*, não um defeito. Escreva
   o número medido no handoff.
2. ⚠️ **Corra `cargo test -p ph2d-host-desktop --test it` À PARTE, sempre.** O `nextest-impacted`
   **não** alcança `shells/desktop/tests/it/`, e foi ali que a `flip` reprovou no gate da árvore
   combinada com o `cargo check` verde.
3. ⛔ **Uma agulha de gate ancora na LEI, nunca em quem pode chamá-la.** Ao publicar a API de uma
   folha, `pub(crate) fn` vira `pub fn` — um gate ancorado no modificador reprova sem que a lei mude
   (HOWTO §2.13, pago por duas linhas no mesmo dia).
4. ⛔ **Se precisar de um SEXTO método no `AppHost` — PARE e reporte com a lista.** A batedora
   (`physics`) correu a família que mais toca a `App` (126 membros) e **não** precisou. Antes de
   pedir porta, escreva o que a função precisa **em tipos**: se a resposta é *«três coisas que a
   `App` segura»*, não é porta — **é assinatura**.
5. ⛔ **Não edite a árvore de outra linha.** As três abrem ao mesmo tempo. Módulo seu consumido por
   `<outra>_*` ⇒ **alias de uma linha com data de validade escrita** (HOWTO §4).
6. **A PROVA:** `cargo nextest list --workspace --cargo-profile ci-test` antes (na base) e depois,
   por `python3 scripts/nextest-list-diff.py antes.txt depois.txt`. `ONLY-A` vazio, ou não fechou.
7. ⭐ **O FIM DA LINHA é gateado para a `motion` e a `vec`:** a família sai da catraca
   `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` porque o `const FAMILY` passou a declarar os roteadores
   que ela **de facto lê** (`max_level` **contado** no `match`, nunca de memória). ⚠️ A `flip` já
   está fora dela — declara 18.

---

## §2 — `line/app-motion` (99 915 — a maior, e a que tem o plano mais antigo)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE B (2.ª volta)   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-motion.  Ela JÁ EXISTE: a Fase A dela
integrou em 11/09, e a Fase B dela foi BLOQUEADA e parou por ordem do
integrador — sem mover produto nenhum.

⭐ O BLOQUEIO CAIU. Você mediu que 13 folhas partilhadas prendiam os 26
   bloqueadores, e que a maior (`vec_entities`) prendia 7 sozinha. Em
   12/09 a `line/shell-folhas` tirou-as todas da shell. A estrada está
   livre — e você é a maior das três: 99 915 linhas.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-motion
   pwd && git branch --show-current     # DEVE dizer line/app-motion
2. git rebase main                      # o main andou MUITO (130 commits)
      → Cargo.lock ou registry-init em conflito: NUNCA na mão — regenere.
      → conflito em código FORA dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-motion       # warm-up; o 1º build é frio
4. LEIA INTEIRO, nesta ordem:
   a) docs/IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md
      — o estado completo. A §4 tem as SEIS leis destes dois dias, e a
        nº 1 é literalmente a sua: contar citações erra A FAVOR, e você
        pagou-a a 30×.
   b) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
      — a LEI da wave; a §2 tem 13 armadilhas, cinco MUDAS.
   c) o seu próprio handoff do bloqueio:
      docs/Motion Nodes/handoffs/HANDOFF_line_app_motion_FASE_B_O_BLOQUEIO_MEDIDO_2026-09-11.md
      ⚠️ Ele existe SÓ na sua worktree — a sua linha parou e nunca foi
      integrada, logo ele não está no main. Depois do rebase do passo 2
      ele continua lá (é um dos seus 3 commits). Se não o achar, você
      está na árvore errada: `pwd`.
   d) o §1 de docs/IntegracaoMultiAgente/BLOCOS_REABERTURA_W2_FASE_B2_2026-09-12.md
5. Reporte "motion pronta, a re-medir o fecho" e SIGA.

A TAREFA:
  PASSO 1 — RE-MEDIR O FECHO. ⛔ Não mova nada antes. A sua lista de 26
  bloqueadores foi escrita ANTES de as folhas saírem; ela descreve uma
  árvore que já não existe. A pergunta é a sua: «a partir dos ficheiros
  que quero mover, que raízes da shell continuam alcançáveis?» — e ela
  tem de seguir CAMPOS (`app.gfx.motion`), não só caminhos de módulo.
  O número que sair é o plano, e vai no handoff.

  PASSO 2 — o corte, pelo HOWTO. O alvo medido em 12/09:
    shells/desktop/src/motion/       268 f /  60 852 L
    shells/desktop/src/render_loop/  143 f /  39 063 L   (o MAIOR das três)
    ──────────────────────────────────────────────────
                                     411 f /  99 915 L
  Para o render_loop: o LAÇO fica na shell — é ele que garante a ordem
  dos 48 símbolos — e saem os CORPOS; a shell chama
  `ph2d_app_motion::<mod>::<fn>` no ponto certo. ⛔ NÃO abstraia o laço.
  ⚠️ Vá por FATIAS que compilam, uma por commit: 411 ficheiros num
  commit é um passo que não se bissecta.

  O FIM DA LINHA (gateado):
  · os roteadores que a shell lê pela sua família passam para a crate —
    CONTE-OS você (a forma é `PH2D_*_SMOKE`; `PH2D_GPU_COOK`, `PH2D_LADO`,
    `PH2D_LAYOUT_LEVEL` e `PH2D_DROPS_SCAN_MAX` são DIAGNÓSTICO e não
    entram no `FAMILY`);
  · o `const FAMILY` declara-os, com cada `max_level` CONTADO no `match`;
  · "motion" SAI de FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL (só a sua);
  · `cargo test -p ph2d-app-registry-init` verde.

REGRAS: §1 do BLOCOS_REABERTURA (as sete). Em especial: ⛔ não toque no
TETO_LOC · ⚠️ corra `cargo test -p ph2d-host-desktop --test it` à parte ·
⛔ 6º método no AppHost = PARE e reporte.
A. Tudo DENTRO da SUA worktree (`pwd` na dúvida — o mesmo path existe na
   raiz, e editar lá compila e commita sem erro).
C. Commits locais, `--no-verify`. NUNCA push/--force/`git add -A`.
E. Fechar = gate batched 1× e PARE. Você NÃO integra.
H. Handoff obrigatório (DIRETRIZ §1.5.9), com o fecho re-medido.
I. Smoke COMPILADO na sua worktree, 2 corridas, a 2ª no handoff:
   cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §3 — `line/app-vec` (28 204 — a que nomeou a raiz)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE B (2.ª volta)   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-vec.  Ela JÁ EXISTE: a Fase A integrou
em 11/09 e a Fase B em 12/09, mas parou cedo — tirou 4 975 das 35 671 que
tinha pela frente, e NÃO alcançou o fim gateado.

⭐ E VOCÊ ACERTOU NA CAUSA: escreveu *«o bloqueador desta família não é a
   `App` — é o `name_unique`»*, e desenhou o grafo com a raiz nele. Em
   12/09 a `line/shell-folhas` tirou o `name_unique`, o `morph_set`, o
   `vec_entities`, o `vec_transform` e o `off_canvas` da shell — o seu
   grafo perdeu a raiz E o ciclo.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec
   pwd && git branch --show-current     # DEVE dizer line/app-vec
2. git rebase main                      # o main andou MUITO (130 commits)
      → Cargo.lock ou registry-init em conflito: NUNCA na mão — regenere.
      → conflito em código FORA dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-vec          # warm-up; o 1º build é frio
4. LEIA INTEIRO: (a) ESTADO_W2_2026-09-12.md — a §4 tem as seis leis;
   (b) o HOWTO (a LEI da wave, §2 com 13 armadilhas, cinco MUDAS);
   (c) o seu handoff HANDOFF_INTEGRACAO_line_app_vec_FASE_B_2026-09-11.md
   — o §3 é o grafo que acabou de mudar, e o §6 diz por que o fim não
   foi alcançado; (d) o §1 do BLOCOS_REABERTURA_W2_FASE_B2_2026-09-12.md
5. Reporte "vec pronta, a re-medir o fecho" e SIGA.

A TAREFA:
  PASSO 1 — RE-MEDIR O FECHO. O seu §3 media quatro cenários sobre um
  grafo cuja RAIZ já não está na shell. ⛔ Não mova nada antes de o
  refazer; o número novo é o plano.
  ⚠️ E a sua crate já consome as folhas: `ph2d_vec_entities` tem 188
  consumidores no repo, `ph2d_unique_name` 17.

  PASSO 2 — o corte, pelo HOWTO. O alvo medido em 12/09:
    shells/desktop/src/vec_*          92 f /  25 989 L
    shells/desktop/src/render_loop/    6 f /   2 215 L
    ─────────────────────────────────────────────────
                                      98 f /  28 204 L
  ⚠️ A sua Fase A mediu que `#[path]` é aresta DURA nos DOIS sentidos
  (`vec_gizmo_view` DECLARA `#[path = "vec_gizmo_pick.rs"]`) — releia o
  doc-comment do `lib.rs` da sua crate antes de planear. ⚠️ E os seus 92
  ficheiros continuam SOLTOS no topo de `src/`: agrupá-los em `src/vec/`
  é o passo que torna o corte mecânico, e as outras quatro fizeram-no na
  Fase A.

  O FIM DA LINHA (gateado):
  · os roteadores passam para a crate — PH2D_VEC_BONE_SMOKE ·
    PH2D_VEC_STACK_SMOKE · PH2D_VEC_APPEARANCE_SMOKE ·
    PH2D_VEC_FADE_SMOKE (+ PH2D_BUILD_SMOKE e PH2D_UI_MOTION_SMOKE se
    forem seus — CONTE, não copie; PH2D_BLEND_LOG e PH2D_TEXT_LOG são
    DIAGNÓSTICO e não entram);
  · o `const FAMILY` declara-os com `max_level` CONTADO, e o doc-comment
    dele (que hoje explica o `routers: &[]`) é REESCRITO no mesmo commit
    — uma dívida cumprida e não apagada lê-se como dívida aberta;
  · "vec" SAI de FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL (só a sua);
  · `cargo test -p ph2d-app-registry-init` verde.

REGRAS: §1 do BLOCOS_REABERTURA (as sete). ⛔ não toque no TETO_LOC ·
⚠️ `cargo test -p ph2d-host-desktop --test it` à parte · ⛔ 6º método no
AppHost = PARE e reporte · A (tudo na SUA worktree) · C (commits locais,
nunca push) · E (fechar e PARAR, você não integra) · H (handoff com o
fecho re-medido) · I (smoke compilado, 2 corridas, a 2ª no handoff).
═══════════════════════════════════════════════════════════════════
```

---

## §4 — `line/app-flip` (18 845 — a mais acoplada em proporção)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE B (2.ª volta)   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-flip.  Ela JÁ EXISTE: a Fase A integrou
em 11/09 e a Fase B em 12/09. ⭐ Você ALCANÇOU o fim gateado (18 roteadores
declarados, e a `flip` não está na catraca) — mas tirou 2 838 linhas de
21 679, porque duas funções de outra família prendiam 84 % de si.

⭐ ESSAS DUAS SAÍRAM. Em 12/09 a `line/shell-folhas` levou o
   `vec_transform` e o `name_unique` para folhas próprias
   (`ph2d-vec-entities` e `ph2d-unique-name`). O seu §3 media a cascata a
   partir delas — refaça-o.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip
   pwd && git branch --show-current     # DEVE dizer line/app-flip
2. git rebase main                      # o main andou MUITO (130 commits)
      → Cargo.lock ou registry-init em conflito: NUNCA na mão — regenere.
      → conflito em código FORA dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-flip         # warm-up; o 1º build é frio
4. LEIA INTEIRO: (a) ESTADO_W2_2026-09-12.md — a §4 tem as seis leis, e
   a nº 4 nasceu de SI (ver abaixo); (b) o HOWTO, §2 com 13 armadilhas;
   (c) o seu handoff HANDOFF_INTEGRACAO_line_app-flip_FASE_B_2026-09-11.md
   — o §3 é a cascata que acabou de mudar; (d) o §1 do BLOCOS_REABERTURA.
5. Reporte "flip pronta, a re-medir a cascata" e SIGA.

⚠️⚠️ E LEIA ISTO SOBRE SI MESMA, porque virou lei da wave: você fechou
   VERDE e reprovou no gate da ÁRVORE COMBINADA, num teste que a sua
   worktree não podia ver — o `nextest-impacted` NÃO alcança
   `shells/desktop/tests/it/`. O gate partido era
   `the_flip_preview_bakes_through_the_same_door`, e ele tinha DUAS
   pontas erradas: um `read_to_string` de caminho fixo a apontar para a
   shell quando a lei tinha ido para a crate (HOWTO §2.6), e uma agulha
   `"pub(crate) fn stroke_from_samples("` que se partiu porque atravessar
   a fronteira obriga `pub(crate)` a virar `pub` (HOWTO §2.13, espécie
   nova). ⇒ **corra `cargo test -p ph2d-host-desktop --test it` À PARTE
   antes de dizer que fechou.**

A TAREFA:
  PASSO 1 — RE-MEDIR A CASCATA, agora que as duas funções saíram.
  PASSO 2 — o corte, pelo HOWTO. O alvo medido em 12/09:
    shells/desktop/src/flip/          64 f /  16 030 L
    shells/desktop/src/render_loop/   10 f /   2 815 L
    ─────────────────────────────────────────────────
                                      74 f /  18 845 L
  ⚠️ Você é a mais acoplada em PROPORÇÃO: 36 dos 64 ficheiros da sua
  pasta ainda mencionam `App`/`gfx` em código (contra 14 de 268 da
  motion). ⛔ Isso é uma INDICAÇÃO de onde olhar, não o fecho — a lei
  nº 1 do estado diz por que a contagem erra a favor.

  O FIM DA LINHA: você já está fora da catraca, então o seu critério é
  o CORTE — as 18 845 linhas fora da shell, com a prova do
  nextest-list-diff exacta e a suíte `--test it` verde.
  ⚠️ E o `undo.rs` da shell depende de `crate::flip::entities`: quando
  essa peça sair, DIGA-O no handoff — ela é o que hoje mantém o `undo`
  preso à shell, e destravá-lo é a wave seguinte de outra pessoa.

REGRAS: §1 do BLOCOS_REABERTURA (as sete). ⛔ não toque no TETO_LOC ·
⚠️ `--test it` à parte · ⛔ 6º método no AppHost = PARE e reporte ·
A (tudo na SUA worktree) · C (commits locais, nunca push) · E (fechar e
PARAR) · H (handoff) · I (smoke compilado, 2 corridas, a 2ª no handoff).
═══════════════════════════════════════════════════════════════════
```

---

## §5 — Para o integrador, no dia da fusão

1. **Ordem:** mede-se com `collision-surface.sh` em cada worktree imediatamente antes. Na Fase A o
   critério que funcionou foi **churn de costura decrescente**.
2. ⚠️ **O `render_loop/mod.rs` é a costura desta volta** — as três tiram corpos de lá. Espere
   conflitos e resolva por **UNIÃO**: na Fase A os 8 conflitos de código foram todos duas linhas a
   melhorar o mesmo sítio por razões diferentes.
3. **O `TETO_LOC` conta-se UMA vez, no fim, na ordem `integrar → cargo fmt --all → medir → escrever`.**
4. **Depois da última fusão:** `cargo build -p ph2d-host-desktop --profile smoke` no primário (duas
   corridas, a 2.ª com zero `Compiling`) — é a árvore que o Enio abre.
5. ⚠️ **Os quatro avisos de clippy pré-existentes** (§6 do estado) continuam a travar o `ship.sh`.
