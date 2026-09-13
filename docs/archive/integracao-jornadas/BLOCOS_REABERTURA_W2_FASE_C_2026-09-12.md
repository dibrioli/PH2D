# BLOCOS DE REABERTURA — W2 **FASE C** (3.ª rodada), 2026-09-12

> ⛔ **CORRECÇÃO de 2026-09-12 (integrador):** a regra deste bloco que diz que o `nextest-impacted`
> *«filtra»* / *«NÃO alcança»* `shells/desktop/tests/it/` está **refutada por medição** — ele alcança
> (`rdeps(ph2d-app-vec)` → 793 testes). Ver [`ESTADO_W2_2026-09-12.md`](ESTADO_W2_2026-09-12.md) §4
> lei 4. O bloco fica como registo do que se instruiu; não o siga nesse ponto.

> Um bloco colável por linha. **O §1 é comum e vale para as três** — cada bloco cita-o em vez de o
> repetir. Os números são os da árvore combinada ao fim da 2.ª volta (`main` = shell **373 937** /
> 1 498 ficheiros), **medidos hoje**, não copiados de um handoff.
>
> Estado completo da obra: [`ESTADO_W2_2026-09-12.md`](ESTADO_W2_2026-09-12.md).
> Molde do corte: [`HOWTO_partir_uma_familia_da_shell.md`](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md).

---

## §1 — AS NOVE REGRAS COMUNS (leia antes do seu bloco)

1. ⛔⛔ **Não toque no `TETO_LOC`** (`crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`,
   hoje `377_937`). É um número que **soma entre linhas**, logo CONTA-SE e é do **integrador**, sobre
   a árvore combinada, **depois** do `cargo fmt --all`. Com três linhas a escrevê-lo, o merge fica
   com um deles e nenhum está certo — em silêncio. ⚠️ Ele **vai** ficar vermelho na sua worktree
   quando o seu corte passar de ~4 000 linhas: **é esperado, é a metade de obsolescência, e não é
   seu.** Diga-o no handoff e siga.
2. ⚠️⚠️ **Corra `cargo test -p ph2d-host-desktop --test it` À PARTE, antes de dizer que fechou.** O
   `nextest-impacted` do portão da sua linha **filtra** `shells/desktop/tests/it/`, e foi ali que a
   `flip` reprovou no gate da árvore combinada com o `cargo check` verde (ESTADO §4 lei 4). São ~812
   testes e é rápido.
3. ⛔ **Uma agulha de gate ancora na LEI, nunca na VISIBILIDADE.** Atravessar a fronteira obriga
   `pub(crate) fn` a virar `pub fn`, e um gate que nomeia o modificador reprova **sem que a lei mude
   uma linha**. Três linhas já pagaram isto, uma delas ao *corrigir* a armadilha que citava
   (HOWTO §2.13).
4. ⛔⛔ **Um SEXTO método no `AppHost` = PARE e reporte.** As 5 portas
   (`pointer`, `mods`, `pointer_over_chrome`, `modal_takes_the_pointer`, `note_authored_change`)
   chegaram para **cinco** famílias. Antes de pedir porta, **escreva em TIPOS o que a função
   precisa** — a `physics`, a `sculpt3d` e a `flip` fizeram-no e a resposta foi sempre *assinatura*
   ou um `SceneCtx` que a shell preenche. ⛔ Nenhum método devolve um handle.
5. ⛔ **Tudo DENTRO da SUA worktree.** `pwd` na dúvida: o mesmo caminho relativo existe no primário,
   e editar a árvore errada **compila e commita sem erro nenhum**.
6. ⛔ **Commits locais, `--no-verify`. NUNCA `push`, `--force`, `git add -A` ou `git stash` nu** (a
   pilha de stash é partilhada entre worktrees).
7. ⛔ **Fechar = gate batched 1× + handoff (DIRETRIZ §1.5.9) + PARAR.** Você **não** integra e
   **não** faz ship — isso é ordem explícita do Enio, por um integrador dedicado (`CLAUDE.md` §0.7).
   E reclame o disco no fim: `rm -rf target/*/incremental`.
8. ⛔⛔ **O TERRITÓRIO É NOMEADO, E É DISJUNTO — esta é a lei 7 do ESTADO, paga nesta wave.** Na 2.ª
   volta a `motion` pediu 7 ficheiros a duas linhas pela prosa do handoff dela: a `flip` levou os
   cinco, a `vec` classificou os dois dela como *«não é trabalho desta linha»* e deixou-os. **Nenhuma
   das duas errou** — o pedido não estava no briefing de quem o recebeu. Resultado medido: **632
   linhas não movidas prendem 99 845.** ⇒ cada bloco abaixo diz **o que é seu** *e* **o que é de
   outra linha esta rodada**. Se precisar de algo fora do seu território, **não o mova**: escreva-o
   no handoff com o nome do ficheiro e o nome da linha dona.
9. ⚠️ **As TRÊS vão tocar os mesmos cinco ficheiros de costura** — `shells/desktop/src/app_state.rs`,
   `main.rs`, `input_dispatch.rs`, `render_loop/mod.rs` (14 046 linhas) e `shells/desktop/Cargo.toml`,
   mais o `Cargo.lock`. Um conflito de rebase **ali, em regiões diferentes**, é o caso legítimo e
   resolve-se; no **mesmo símbolo** é violação de isolamento ⇒ **PARE e reporte** (DIRETRIZ §1.5.5).
   ⚠️ `Cargo.lock` e `*-registry-init/` **nunca** à mão: aceite um lado e regenere.

### A régua do FECHO (o passo 1 dos três blocos)

⛔⛔ **Contar citações mede menos do que o fecho e erra A FAVOR** — a `motion` pagou-a a **30×**
(mediu 46 757 linhas movíveis onde a verdade era 1 573). A pergunta certa é *«a partir dos ficheiros
que quero mover, que raízes da SHELL continuam alcançáveis?»*, e ela tem de seguir **campos**
(`app.gfx.motion`, `app.flip_state`), que nenhuma varredura por nome de módulo vê.

A `line/app-motion` construiu o instrumento:

```
python3 scripts/fecho-da-familia.py <familia>        # se já estiver no main
python3 scripts/fecho-da-familia.py --autoteste      # ele tem autoteste; corra-o primeiro
```

⚠️⚠️ **Se o ficheiro não existir no `main`, ele vive SÓ na worktree da motion** (a linha dela nunca
integrou). Chame-o pelo caminho absoluto **de DENTRO da sua worktree**:

```
python3 /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-motion/scripts/fecho-da-familia.py <familia>
```

⭐ Isso é seguro porque ele resolve `shells/desktop/src` **relativamente ao directório de onde é
chamado** — ele mede a árvore em que você está, não a árvore onde ele mora. ⛔ Mas confirme com
`pwd` antes, senão mede a árvore da motion.

---

## §2 — `line/app-motion` — **99 845 linhas** (a maior de todas, e agora desbloqueada)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE C   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-motion.  Ela JÁ EXISTE e tem 5 commits
seus por integrar (docs + a ferramenta do fecho); está 16 commits atrás do
main.

⭐⭐ O BLOQUEIO CAIU, E QUASE TODO. Na 2.ª volta você mediu 7 âncoras: 4 suas,
   5 ficheiros da `line/app-flip` e 2 da `line/app-vec`. A flip LEVOU os cinco
   dela (verificado ficheiro a ficheiro pelo integrador: vivem em
   `ph2d-app-flip`). A vec NÃO levou os dois dela.
   ⇒ Sobra UM cluster de 632 linhas, e ELE É SEU nesta rodada.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-motion
   pwd && git branch --show-current     # DEVE dizer line/app-motion
2. git rebase main                      # 16 commits; os seus 5 são docs+script
3. cargo check -p ph2d-app-motion       # warm-up
4. LEIA INTEIRO, nesta ordem:
   a) docs/archive/integracao-jornadas/ESTADO_W2_2026-09-12.md — o §3 tem o seu
      cluster com o mecanismo, e o §4 as SETE leis (a 1.ª e a 7.ª são suas)
   b) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md — a LEI
   c) o §1 de BLOCOS_REABERTURA_W2_FASE_C_2026-09-12.md (as nove regras)
   d) o seu handoff: docs/Motion Nodes/handoffs/
      HANDOFF_line_app_motion_FASE_B2_O_FECHO_REMEDIDO_2026-09-12.md
      — o §3 tem as 7 âncoras, o §4 o MotionSceneCtx já desenhado em tipos
5. Reporte "motion pronta" e SIGA.

A TAREFA:
  PASSO 0 — OS 632 QUE FALTAM, e são seus:
      shells/desktop/src/brush_live.rs            223 L
      shells/desktop/src/texture_pattern_live.rs  409 L
    ⭐ Medidos pelo integrador na árvore combinada: são um CLUSTER-FOLHA. A
    única aresta para fora é `brush_live` → `texture_pattern_live`, o irmão;
    tudo o mais que eles nomeiam são crates irmãs (`ph2d_vec_render`,
    `ph2d_vec_scene`, `ph2d_asset`, `ph2d_vector`).
    ⛔ NÃO os ponha em `ph2d-app-vec` — a `line/app-vec` está a reestruturar
    essa crate AGORA. Faça uma crate-FOLHA de assunto próprio (a arte das
    estampas e dos pincéis do vetor). Uma folha por ASSUNTO, nunca um saco.
    ⚠️ Eles NÃO são uma folha de funções puras: a shell tem os campos
    `self.brush_live` e `self.texture_pattern_live`, com 13 consumidores
    (`app_state.rs`, `main.rs`, `render_loop/mod.rs`, `fx_live.rs`,
    `input_dispatch.rs`, `vec_stroke_paint.rs`, `texture_pattern_pick.rs`,
    `render_loop/vector_bridge_publish.rs`, e os testes) ⇒ o TIPO vai para a
    crate, o CAMPO fica na shell.
    ⚠️ Porque isto destrava 99 845 com 632: o `motion/motion_object_bake.rs`
    chama `crate::brush_live::resolve` DENTRO de um laço, e daí a cadeia sobe
    ao `motion_state.rs`, que 106 ficheiros nomeiam e de que 42 são filhos
    `#[path]`. Um campo de struct é aresta tão dura como um `#[path]`.

  PASSO 1 — RE-MEDIR O FECHO (a régua do §1). ⛔ Não mova produto antes.
    Corra o `--autoteste` primeiro. O número que sair é o plano e vai no
    handoff. ⚠️ A sua própria lista de 7 âncoras é de ontem: 5 dissolveram.

  PASSO 2 — O CORTE, pelo HOWTO. O alvo medido HOJE:
      shells/desktop/src/motion/            267 f /  60 782 L
      shells/desktop/src/render_loop/motion* 143 f /  39 063 L
      ───────────────────────────────────────────────────────
                                            410 f /  99 845 L
    + as suas 3 âncoras próprias: picker_smoke.rs (187) · field_gizmo.rs (583)
      · thumbnail.rs (115, folha PURA ⇒ crate própria)
    + o `MotionSceneCtx` que você já desenhou em tipos (§4 do seu handoff):
      o único campo que era âncora de shell — `app.flip_state` — hoje é
      `ph2d_app_flip::state::FlipState`, uma CRATE. Reconfira-o.
    ⚠️ O LAÇO fica na shell (ele garante a ordem dos 48 símbolos); saem os
    CORPOS, e a shell chama `ph2d_app_motion::<mod>::<fn>` no ponto certo.
    ⛔ NÃO abstraia o laço.
    ⚠️ VÁ POR FATIAS QUE COMPILAM, uma por commit. 410 ficheiros num commit é
    um passo que não se bissecta, e esta é a maior família da wave.

  O FIM DA LINHA (gateado) — você é a ÚLTIMA na catraca:
  · os roteadores que a shell lê pela motion passam para a crate — CONTE-OS
    (forma `PH2D_*_SMOKE`; `PH2D_GPU_COOK`, `PH2D_LADO`, `PH2D_LAYOUT_LEVEL`,
    `PH2D_DROPS_SCAN_MAX` são DIAGNÓSTICO e não entram no `FAMILY`);
  · o `const FAMILY` declara-os, com cada `max_level` CONTADO no `match`;
  · "motion" SAI de FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL — ⭐ e ela fica
    VAZIA, logo o bloco e a metade `if` do gate desaparecem com ela
    (está escrito no doc-comment dele: leia-o antes de apagar);
  · `cargo test -p ph2d-app-registry-init` verde.

  ⭐ E PONHA A SUA FERRAMENTA NO CAMINHO DE TODOS: o
  `scripts/fecho-da-familia.py` só existe na sua worktree, e as outras duas
  linhas precisam dela. Ela entra no main com a sua integração — mas
  ACRESCENTE-A ao §1 do HOWTO pelo NOME, no passo 1 do molde. Uma ferramenta
  que nenhum passo escrito chama pelo nome morre (CLAUDE.md §2: o
  `cargo-check-narrow.sh` foi invocado 5 vezes em 101 sessões).

⛔ NÃO É SEU esta rodada: shells/desktop/src/vec_*.rs · os outros 16 *_live.rs
   · bone_*.rs / skeleton_*.rs (todos da `line/app-vec`) ·
   shells/desktop/src/physics/ (da `line/app-physics`).

REGRAS: as nove do §1. Em especial ⛔ TETO_LOC · ⚠️ `--test it` à parte ·
⛔ 6º método = PARE · ⛔ tudo na SUA worktree · ⛔ fechar e PARAR.
═══════════════════════════════════════════════════════════════════
```

---

## §3 — `line/app-vec` — **22 104 linhas** + duas famílias por desalojar

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE C   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-vec.  A sua 2.ª volta INTEGROU em 12/09
(shell 398 037 → 390 646, e depois a flip levou-a a 373 937). O seu ramo está
IGUAL ao main: zero commits à frente, zero atrás. ⭐ Você alcançou o fim da
linha gateado e "vec" já SAIU da catraca das famílias.

⇒ O seu critério nesta rodada é o CORTE, e o seu próprio §11 escreveu a fila.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec
   pwd && git branch --show-current     # DEVE dizer line/app-vec
2. git rebase main                      # deve ser no-op (já está em main)
3. cargo check -p ph2d-app-vec          # warm-up
4. LEIA INTEIRO: (a) ESTADO_W2_2026-09-12.md — o §4 tem as SETE leis, e a
   7.ª nasceu DESTA linha (ver abaixo); (b) o HOWTO; (c) o §1 do
   BLOCOS_REABERTURA_W2_FASE_C; (d) o seu handoff
   docs/Vector Module/handoffs/HANDOFF_INTEGRACAO_line_app_vec_FASE_B2_2026-09-12.md
   — o §11 é a fila desta rodada, por ordem de valor.
5. Reporte "vec pronta" e SIGA.

⚠️⚠️ LEIA ISTO SOBRE SI MESMA, porque virou a 7.ª lei da wave: a
`line/app-motion` pediu-lhe DOIS ficheiros (`brush_live.rs` e
`texture_pattern_live.rs`) no §6 do handoff dela; você classificou-os no seu
§11 como *«3.ª rodada de famílias — não é trabalho desta linha»* e deixou-os.
⭐ VOCÊ NÃO ERROU: o pedido viajou na prosa de um handoff que não era o seu
briefing. Mas o preço foi medido — 632 linhas não movidas prendem 99 845, e a
motion parou uma segunda vez. ⇒ nesta rodada o território está NOMEADO, e
esses dois ficheiros são da MOTION: ⛔ não lhes toque.

A TAREFA (o seu §11, com os números re-medidos hoje):
  PASSO 1 — RE-MEDIR O FECHO (a régua do §1). O seu §11 diz «15 ficheiros /
  3 404 LOC movem HOJE sem curar nada» — confirme-o com a ferramenta, não com
  a memória. ⛔ A sua sonda corrigiu-se TRÊS vezes na volta passada, as três a
  favor de mover demasiado (5.ª, 6.ª e 7.ª ocorrência nesta família).

  PASSO 2 — O CORTE, em três frentes que são três assuntos:

  (a) A SUA FAMÍLIA — 76 f / 22 104 L em `shells/desktop/src/vec_*.rs`
      + 6 f em `render_loop/vec*` + 6 f em `render_loop/vector_bridge*`.
      ⭐ O corte que a volta passada fez em seis ficheiros é o molde:
      *lei pura ↔ ponte `impl App`*. O `vec_bucket` e o `vec_svg_export`
      pedem-no pelo nome.
      ⚠️ `#[path]` é aresta DURA nos DOIS sentidos (o `vec_gizmo_view`
      DECLARA `#[path = "vec_gizmo_pick.rs"]`) — releia o doc-comment do
      `lib.rs` da sua crate antes de planear.
      ⚠️ E os 76 continuam SOLTOS no topo de `src/`: agrupá-los em
      `src/vec/` é o que torna o corte mecânico, e quatro famílias já o
      fizeram na Fase A.

  (b) AS LEIS VIVAS — 16 f / 6 267 L, os `*_live.rs` do topo de `src/`:
      align · blend · bool · connector · contour · envelope · fx · label ·
      layout · morph · offset · pattern · profile · skeleton · symmetry ·
      widget.
      ⛔⛔ EXCEPTO `brush_live.rs` e `texture_pattern_live.rs` — esses dois
      são da `line/app-motion` esta rodada (ver acima). NÃO os mova.
      ⚠️ O seu §11 já mediu quem prende quem: `bool_live` tem 4 pedintes,
      `offset_live`/`profile_live`/`envelope_live` 2 cada. Meça o fecho do
      GRUPO, não de um ficheiro: eles são o pipeline de geometria viva do
      vetor e provavelmente saem juntos ou não saem.

  (c) ⭐ O ESQUELETO É FAMÍLIA PRÓPRIA — 5 f / 1 673 L:
      bone_pose.rs (144) · bone_limit.rs (227) · bone_pick.rs (418) ·
      bone_gesture.rs (407) · skeleton_goal.rs (477).
      O seu §11 item 3 chamou-lhe *«uma wave coerente e com dono próprio»*, e
      tem razão: o Esqueleto é MÓDULO desde o ADR-0169, com crates e painel
      próprios (`ph2d-skeleton`, `-ecs`, `-render`, `ph2d-panel-skeleton`), e
      os nomes canónicos dele são SEM «vec».
      ⇒ crie `crates/ph2d-app-skeleton/` como família, com `const FAMILY`.
      ⚠️ A `ph2d-skeleton-live` que você criou na volta passada é a LEI PURA;
      isto é a metade de AUTORIA (o gesto, o pick, o limite de ângulo).
      ⛔ Não pode depender da `ph2d-skeleton-ecs` de forma a fechar ciclo —
      você já mediu esse ciclo na volta passada. Confirme antes de ligar.

  O FIM DA LINHA: já está alcançado (108 = 108 roteadores, "vec" fora da
  catraca) ⇒ o critério é o CORTE, com `nextest-list-diff` exacto
  (ONLY-A = 0) e a suíte `--test it` verde à parte.
  ⚠️ Se criar a família do esqueleto, ela ENTRA na catraca ou declara
  roteador: `cargo test -p ph2d-app-registry-init` diz qual, e a entrada é
  uma ausência DECLARADA, nunca muda.

⛔ NÃO É SEU esta rodada: shells/desktop/src/motion/ · render_loop/motion* ·
   brush_live.rs · texture_pattern_live.rs (todos da `line/app-motion`) ·
   shells/desktop/src/physics/ (da `line/app-physics`).

REGRAS: as nove do §1. Em especial ⛔ TETO_LOC · ⚠️ `--test it` à parte ·
⛔ 6º método = PARE · ⛔ fechar e PARAR.
═══════════════════════════════════════════════════════════════════
```

---

## §4 — `line/app-physics` — **15 625 linhas** (a metade de AUTORIA)

```
═══════════════════════════════════════════════════════════════════
REABERTURA — Modo L · W2 FASE C   (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-physics — A BATEDORA desta wave. Você fez
a Fase A e a Fase B em 11/09, alcançou o fim da linha gateado (as 118 cenas,
o `const FAMILY` com `max_level: smoke::CENAS` contado, "physics" FORA da
catraca) e provou às outras quatro que as 5 portas do `AppHost` bastavam.
O seu ramo está 60 commits atrás do main e não tem nada à frente.

⇒ O seu critério é o CORTE: sobram 60 ficheiros / 15 625 linhas.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-physics
   pwd && git branch --show-current     # DEVE dizer line/app-physics
2. git rebase main                      # 60 commits — o main andou muito
      → Cargo.lock / *-registry-init: NUNCA à mão, aceite um lado e regenere
      → conflito em código FORA dos seus ficheiros: PARE e reporte
3. cargo check -p ph2d-app-physics      # warm-up; o 1º build é frio
4. LEIA INTEIRO: (a) ESTADO_W2_2026-09-12.md — as SETE leis; (b) o HOWTO;
   (c) o §1 do BLOCOS_REABERTURA_W2_FASE_C; (d) os SEUS dois handoffs:
   docs/Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_A_2026-09-11.md
   (o §6 é a lista NOMEADA do que só o substrato resolvia, e o §9 as nove
   armadilhas) e ..._FASE_B_2026-09-11.md (o §5 tem as oito da Fase B).
5. Reporte "physics pronta" e SIGA.

⭐⭐ DUAS COISAS MUDARAM DESDE QUE VOCÊ ESCREVEU AQUELE §6, e as duas a seu favor:
 · A TIMELINE: você decidiu que *«uma crate-motor irmã não é a shell»* e
   dependeu da `ph2d-timeline` — a decisão está tomada e shipada; não a
   re-litigue, USE-A.
 · As FOLHAS PARTILHADAS que você nomeou (`inspector_ordering`,
   `preview_drive`, `name_unique`) SAÍRAM da shell em 12/09 pela
   `line/shell-folhas` — são crates hoje (`ph2d-inspector-ordering`,
   `ph2d-preview-drive`, `ph2d-unique-name`), com 17 a 41 consumidores.
   ⇒ as três âncoras que você nomeou como *«só o substrato resolve»*
   dissolveram. Reconfira o §6 contra o código antes de o citar.

A TAREFA:
  PASSO 1 — RE-MEDIR O FECHO (a régua do §1). ⛔ Não mova nada antes. O seu §6
  descreve uma árvore de 11/09; o main já não é essa árvore.
  ⚠️ E ele tem de seguir CAMPOS: você criou o `PhysicsState` com 7 campos de
  `App`, e o acoplamento que viaja por um campo não tem nome de módulo nenhum.

  PASSO 2 — O CORTE. O alvo medido HOJE, em `shells/desktop/src/physics/`
  (60 f / 15 625 L), e ele parte-se em DOIS assuntos com forma diferente:

  (a) AS JUNTAS — a autoria: `joint.rs` · `joint_create.rs` · `joint_draw.rs`
      · `joint_rig.rs` · `joint_rig_drag.rs` · `joint_anchor_drag*.rs` (4 f)
      + os testes irmãos. É GESTO DE CANVAS: arrastar uma âncora, desenhar
      uma junta, agarrar uma roldana.
  (b) O INSPECTOR — `inspector_body.rs` + os 7 `inspector_player_*_tests.rs`.
      ⚠️ O seu §6 diz que os gates deles dirigem
      `render_loop::inspector_joint_wheel` e `inspector_physics_tests::apply`
      — confirme onde essas funções vivem HOJE antes de planear.
  (c) `bake.rs` · `bake_curve_tests.rs` · `bridge.rs` · `bridge_tests.rs`.

  ⚠️ Os `render_loop/physics_*` JÁ NÃO EXISTEM (a sua Fase B levou-os) — o que
  resta no `render_loop` com nome `inspector_*` são 24 ficheiros de inspector
  GENÉRICO, que são chrome partilhado e ⛔ NÃO são seus.

  ⛔⛔ É AQUI QUE UM SEXTO MÉTODO PARECE NECESSÁRIO, E NÃO É. Gesto de canvas
  e inspector são precisamente a metade que toca a `App`. A regra 4 do §1 vale
  em cheio: escreva o que cada função precisa em TIPOS, e a resposta será
  assinatura ou um `SceneCtx` que a shell preenche. ⭐ Você é a linha que
  PROVOU isto para as outras quatro — o molde é o seu próprio
  `ph2d_app_physics::SceneCtx`. Se ainda assim faltar porta: PARE e reporte,
  com a tabela dos tipos.

  O FIM DA LINHA: já alcançado ⇒ o critério é o CORTE, com prova
  `nextest-list-diff` (ONLY-A = 0) e a suíte `--test it` verde à parte.

⛔ NÃO É SEU esta rodada: shells/desktop/src/motion/ · render_loop/motion* ·
   brush_live.rs · texture_pattern_live.rs (da `line/app-motion`) ·
   vec_*.rs · os 16 *_live.rs · bone_*.rs · skeleton_*.rs (da `line/app-vec`).

REGRAS: as nove do §1. Em especial ⛔ TETO_LOC · ⚠️ `--test it` à parte ·
⛔ 6º método = PARE e reporte · ⛔ fechar e PARAR.
═══════════════════════════════════════════════════════════════════
```

---

## §5 — Notas para o integrador

1. **Ordem medida** (churn de costura decrescente, o critério que funcionou nas duas voltas):
   **motion → vec → physics.** A motion reescreve `render_loop/mod.rs` em 143 ficheiros; a physics
   é a que toca menos costura.
2. ⚠️ **A `motion` traz a ferramenta `scripts/fecho-da-familia.py` no ramo dela.** Se as outras duas
   linhas correrem antes dela, elas usam o atalho do §1 (caminho absoluto, medindo a árvore de onde
   é chamado). **Prefira integrar a motion primeiro** — o instrumento fica no `main` para todos.
3. ⛔ **O `TETO_LOC` vai reprovar depois de cada fusão desta rodada.** É a metade de obsolescência, é
   esperado, e a ordem é: integrar → `cargo fmt --all` → medir → escrever. Nenhuma linha lhe toca.
4. ⚠️ Se a `vec` criar `crates/ph2d-app-skeleton/`, o `ph2d-app-sync` tem de correr (passo 2 do
   `foundational-integrate.sh` já o faz), e a família nova ou declara roteador ou entra na catraca —
   uma ausência **declarada**, nunca muda.
5. **Nada foi pushado.** Conte os commits (`git rev-list --count origin/main..main`), e os **quatro
   avisos de clippy** em `crates/ph2d-app-sculpt3d/{keys,bake,requests}.rs` +
   `shells/desktop/src/sculpt_source/mod.rs` continuam a travar o `ship.sh` (`-D warnings`).
