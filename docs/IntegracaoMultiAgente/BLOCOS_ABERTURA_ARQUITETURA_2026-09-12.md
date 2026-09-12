# `line/editor-core` + `line/render-loop` — as duas linhas que a auditoria de arquitectura deixou

> **De onde vêm:** a [auditoria de 12/09](AUDITORIA_ARQUITETURA_2026-09-12.md) §6 fechou oito dos
> onze achados na própria jornada e deixou **quatro obras medidas** para a seguinte: a A5b (os ids
> descem para o painel dono), a A10 (os módulos da fundação em ciclo), a A9 (os campos `vec_*` soltos
> na `App`) e o `run_render_frame` (13 685 linhas numa função). O dono delegou as decisões técnicas
> ao padrão-ouro e pediu *«tudo no melhor estado antes de enviar»* — nada é enviado por estas linhas.
>
> ⛔ **São DUAS linhas, não quatro, e a razão é medida:** as quatro obras formam duas duplas que
> editam os mesmos ficheiros, e as duplas quase não se tocam (§1). Duas linhas dentro da mesma dupla
> colidiriam em código movido, que nenhum merge sintáctico resolve.

---

## §1 — O que se sobrepõe, medido no `main` de 12/09 (`646b2523c`)

| par | onde se tocam | número | ⇒ |
|---|---|---:|---|
| A9 × `run_render_frame` | acessos `self.vec_*` dentro de `render_loop/mod.rs` | **581** | a MESMA linha, A9 primeiro |
| A5b × A10 | ficheiros de `widget/` · `interaction/` · `screens/` que citam `ids::` | **97** | a MESMA linha, A5b primeiro |
| A5b × A10 | referências `interaction → ids` / `ids → interaction` | **90 / 10** | idem |
| editor-core × render-loop | ficheiros da shell que citam `ids::` (dos quais em `render_loop/`) | **44** (19) | ⚠️ a única zona de contacto — **cercada** (§2) |

⚠️ **Nenhuma linha aberta está por integrar** (`git branch --no-merged main` não devolve nenhuma
`line/*`): as duas nascem sem vizinhos.

## §2 — ⛔⛔ A CERCA entre as duas linhas (é LEI nas duas)

| ficheiros | dono nesta rodada | a outra linha |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/**` · `crates/ph2d-panel-*/src/ids*.rs` · `crates/ph2d-editor-core/tests/it/node_id_collisions.rs` | `line/editor-core` | não edita, não cria ids |
| `shells/desktop/src/render_loop/**` · `shells/desktop/src/app_state.rs` · `shells/desktop/src/vec_*.rs` · `crates/ph2d-app-vec/src/state.rs` | `line/render-loop` | não edita |

⇒ **Um id que desce e é citado num ficheiro da `line/render-loop` FICA nesta rodada**; a
`line/editor-core` lista-os no handoff e o integrador desce-os **depois** das duas fusões, com a
árvore combinada à frente (§5).

⛔ **`TETO_LOC` de `architecture_the_shell_only_shrinks` não é de nenhuma das duas** — soma entre
linhas, reconta-o o integrador. Os tectos **por ficheiro** dos ficheiros que cada linha encolhe descem
no mesmo commit (a metade *«o tecto ficou para trás»* obriga).

---

## §3 — BLOCO `line/editor-core` (cole isto numa janela nova)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA — Modo L · ARQUITECTURA A5b + A10   (DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/editor-core.  Alvo: a FUNDAÇÃO deixa de
ser o sítio onde todas as famílias se encontram. Duas obras, NESTA ordem:
  1. A5b — os ids de widget que só UM painel lê descem para esse painel,
     e o censo de colisões passa a ser DERIVADO da workspace.
  2. A10 — os módulos da ph2d-editor-core deixam de depender uns dos
     outros nos dois sentidos (um DAG, com gate).
⭐ POR QUE: em 30 dias, 221 dos 437 commits à editor-core/src tocaram
   ids/ — cada um recompila as 43 crates que dependem da fundação, e é o
   ficheiro onde as linhas paralelas colidem. E partir a crate é
   impossível enquanto os módulos formam ciclos.
⛔ ESTA LINHA NÃO MUDA PRODUTO: nenhum pixel, texto ou comportamento. Se
   uma cura pedir mudança visível, PARE e reporte ao Enio.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh          # tem de dizer `workstation`
2. cd /home/enio/Documentos/Projetos/PH2D && git status -sb
      → RAIZ, em main. M/?? em project-memory/ são ALHEIOS: não toque.
      ⛔ NÃO faça pull: o main local está À FRENTE do origin (nada foi
        enviado) e é ELE a base.
3. mkdir -p Worktrees
   git worktree add -b line/editor-core Worktrees/line-editor-core main
4. cd Worktrees/line-editor-core
   pwd && git branch --show-current    # DEVE dizer line/editor-core
5. cargo check -p ph2d-editor-core     # warm-up; o 1º build é frio
6. bash scripts/mergiraf-setup.sh      # idempotente; ✗ não é bloqueio
7. A BASE DA PROVA, antes de editar uma linha:
   mkdir -p target/prova
   cargo nextest list --workspace --cargo-profile ci-test > target/prova/antes.txt
   wc -l target/prova/antes.txt        # ⚠️ vazio = a prova mede nada
8. LEIA INTEIRO (dentro da worktree):
   a) docs/IntegracaoMultiAgente/BLOCOS_ABERTURA_ARQUITETURA_2026-09-12.md
      — §1 o que se sobrepõe; §2 a CERCA com a line/render-loop (LEI).
   b) docs/IntegracaoMultiAgente/AUDITORIA_ARQUITETURA_2026-09-12.md
      §2 (A1, A5, A10) e §6 — as três espécies de cura e o que já foi feito.
   c) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
      §2 (20 armadilhas; as ⛔ são MUDAS) e §3 (a prova).
   d) docs/IntegracaoMultiAgente/DIRETRIZ.md §0, §1.5, §6.7
   e) docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — tudo, e
      releia a cada passo.
9. Reporte "linha editor-core pronta" e SIGA (não pare).

OBRA 1 — A5b: os ids descem para o painel dono
  MEDIDO no main de 12/09 — RE-MEÇA na base antes de mover (a régua
  envelhece a cada integração; a 1.ª tabela do seu handoff é esta):
    2 533 ids em crates/ph2d-editor-core/src/ids/
       754  só têm leitor no painel dono (e em famílias/shell que
            dependem dele)                     → DESCEM
       948  lidos pela própria editor-core     → FICAM (a fundação não
            pode depender de um painel: gate
            architecture_no_dependency_climbs_a_layer)
       474  citados dentro de ids/ (tabelas, fábricas indexadas)
                                               → ficam, ou descem JUNTO
                                                 com quem os cita
       179  lidos por vários painéis           → FICAM
    ⚠️ a régua conta CÓDIGO: tire comentários antes (HOWTO §2.12) e
       resolva `use … as`, `super::` e grupos `{…}` — um ids::X lido por
       alias é um leitor.

  A CURA (o critério da A1, que já é lei neste repo):
    · a DEFINIÇÃO passa a morar no painel; a linha
      `pub use ph2d_editor_core::ids::X` do ids.rs dele morre.
      ⛔ ZERO fachadas: nenhum `pub use` na editor-core a apontar para
      um painel, nenhum no painel a apontar para a editor-core.
    · quem fora do painel lê o id nomeia `ph2d_panel_<x>::ids::X` — as
      camadas deixam (família, composição e registo podem depender de
      um painel).
    · ⚠️ tecto de painel: 600 L por FICHEIRO e 200 por função
      (architecture_panel_loc_cap). Um ids.rs que passe parte-se por
      ASSUNTO em ficheiros irmãos — ⛔ nunca uma entrada nova no
      FILE_OVERAGE_OK.
    · ⚠️ o `hash_node_id` vive em crates/ph2d-tool-registry/src/node_id.rs
      e só 4 dos 28 painéis dependem dessa crate. Duas saídas — MEÇA as
      duas (arestas novas, sítios a reescrever, tempo de check) e escolha:
      (a) o painel passa a depender do contrato;
      (b) a LEI desce para junto do tipo (o NodeId vem da ph2d-a11y, que
          os painéis-alvo já têm).
      ⛔ Nunca fachada. O gate architecture_tool_contract_surface fica
      verde. A tabela vai no handoff.

  AS CERCAS (estão escritas no código — leia-as antes de mover):
    · cabeçalho de ids/mod.rs: os ids são HASH de slug (FNV-1a) para
      acabar com a alocação por faixas; e as linhas HIER_PLAYER..
      HIER_MAIN_CAMERA (400..411) ficam NUMÉRICAS de propósito (a conta
      dos bits EYE_TOGGLE_BIT / EXPAND_TOGGLE_BIT). Não as hashe.
    · o doc de cada crates/ph2d-panel-*/src/ids.rs diz porque a definição
      ficou na editor-core: «layout + z-order walk + node_id_collisions
      referenciam-nas». Os dois primeiros são os 948 que FICAM; o
      terceiro é o censo que você reescreve. Depois da mudança há UMA
      definição e zero re-exports — o «fork da verdade» que o doc temia
      deixa de existir. Reescreva o doc para dizer ISSO.
    · o WidgetStore é pré-populado na construção (ids/mod.rs, topo):
      confira que nenhum id que desce é lido lá.

  O CENSO DE COLISÕES passa a ser DERIVADO:
    · hoje crates/ph2d-editor-core/tests/it/node_id_collisions.rs lista
      À MÃO 619 consts (CHROME_IDS) contra 3 463 literais
      `hash_node_id("…")` na workspace, mais 37 chamadas não-literais
      (parte em prosa; ids/chrome/painter.rs:625 e
      ids/chrome/timeline.rs:523 hasheiam um slug CALCULADO).
    · o novo lê os literais da workspace inteira e hasheia-os com a
      MESMA função. Reprova (1) dois slugs diferentes com o mesmo hash e
      (2) o MESMO slug em dois sítios (o erro de copiar-colar para que o
      ficheiro existe). Tem PISO de população. Nomeia as formas
      não-literais e cobre-as, ou lista-as uma a uma. Prova de mutação
      nas duas metades (/pd-mutacao). A lista à mão MORRE — duas
      respostas à mesma pergunta divergem no dia seguinte.
  ⛔ OS GATES QUE LEEM ids/ PELO CAMINHO podem ficar verdes a varrer
     MENOS (HOWTO §2.7, mudo). Confira cada um depois de mover, e ponha
     piso de população onde faltar:
       editor-core/tests/it: architecture_panel_wiring_parity ·
         every_menu_row_reaches_a_handler · hr12_widgets_a11y ·
         the_menu_bar_relocates_the_verbs_it_shows ·
         the_painted_control_reaches_a_consumer
       panel-vector/tests/it: section_headers_are_collapsible
       shells/desktop/tests/it: the_draw_pass_asks_the_door_that_has_the_gates

OBRA 2 — A10: os módulos da fundação formam um DAG (só depois da 1)
  MEDIDO 12/09 (referências de código entre módulos, nos dois sentidos):
    widget ↔ interaction     73 / 165
    screens ↔ interaction   162 / 22
    interaction ↔ ids        90 / 10
  RE-MEÇA depois da OBRA 1 (ela muda a última linha) — a tabela inteira,
  todos os pares de módulos de topo de src/, não só estes três.

  A CURA, aresta a aresta, por ASSUNTO: um TIPO desce para o módulo dono
  do conceito; uma LEI desce para quem a corre; uma TABELA é injectada
  por quem chama. ⛔ Nunca um re-export para esconder a aresta.
  O GATE: nasce um gate em editor-core/tests/it que lê as arestas entre
  os módulos de topo e reprova um ciclo — catraca NUMERADA das arestas
  toleradas que só encolhe, com a metade «a entrada já não descreve
  nada» e prova de mutação. Alvo: catraca VAZIA.
  A MEDIÇÃO que decide partir a crate — ⛔ NÃO parta nesta linha:
    edite UMA linha real em editor-core/src (⛔ `touch` não mede uma
    edição: o incremental vê o hash igual) e meça
    `cargo test --no-run --workspace --profile ci-test --timings`
    com a máquina calma e o /proc/loadavg AO LADO (acima de load ~5 o
    relógio não vale nada). O handoff leva a tabela e a proposta de
    corte: que módulo vira que crate, e quantas das 43 dependentes
    deixam de recompilar.

⛔⛔ A CERCA COM A line/render-loop (corre ao mesmo tempo que você):
  NÃO edite shells/desktop/src/render_loop/** · shells/desktop/src/app_state.rs
  · shells/desktop/src/vec_*.rs · crates/ph2d-app-vec/src/state.rs.
  Um id que desce e é citado nesses ficheiros FICA nesta rodada — conte-os
  e liste-os no handoff; o integrador desce-os depois das duas fusões.

O FIM DA LINHA (a prova, toda ela):
  · cargo nextest list --workspace --cargo-profile ci-test > target/prova/depois.txt
    python3 scripts/nextest-list-diff.py target/prova/antes.txt target/prova/depois.txt
    → ONLY-A = 0 (nada perdido); ONLY-B só os gates novos, nomeados.
  · cargo test -p ph2d-host-desktop --test it   À PARTE (o
    nextest-impacted não alcança shells/desktop/tests/it).
  · o lint EXACTO do ship.sh, nunca por -p (HOWTO §2.17):
    cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
    (mordeu no próprio dia desta auditoria: items_after_test_module e
    type_complexity, os dois invisíveis ao `check`)
  · cargo check --workspace --all-targets sem aviso (um -p muda a
    unificação de features e mostra outro programa)
  · cargo machete · typos --force-exclude <ficheiros tocados>
  · auditoria ≥ 2 lentes sobre o diff acumulado (/pd-auditoria)

REGRAS PERMANENTES:
A. Tudo DENTRO de Worktrees/line-editor-core/. O mesmo path relativo
   existe na raiz: editar lá é editar a árvore ERRADA. `pwd` na dúvida.
B. Sondas em bash (`bash -c '…'` ou `bash <<'X'`): o shell das
   ferramentas é zsh, que não parte palavras nem tem PIPESTATUS — uma
   sonda que devolve zero pode ser o shell, não a árvore.
C. Edite pela ferramenta Edit. Script só para renomear em N ficheiros, e
   SEMPRE com assert de contagem. ⚠️ Renomear por NOME destrói a prosa
   que cita o símbolo — o porquê deste repo vive em doc-comments.
D. Commits locais frequentes, `git commit --no-verify`. NUNCA push,
   NUNCA --force, NUNCA `git add -A`. `git rebase main` no início de cada
   jornada. Cargo.lock ou registry-init em conflito: regenere, nunca à mão.
E. ⛔ NÃO toque no TETO_LOC de architecture_the_shell_only_shrinks — soma
   entre linhas, quem o reconta é o integrador. Os tectos POR FICHEIRO
   dos ficheiros que você encolhe descem no MESMO commit.
F. Fechar = PARE. Você NÃO integra e NÃO roda foundational-integrate.sh
   nem ship.sh — é do integrador, e só por ordem do Enio.
G. PARE e reporte ao Enio SÓ se: contrato congelado (CLAUDE.md §6), rebase
   a conflitar fora dos seus ficheiros, ou uma cura que muda produto.
H. HANDOFF (DIRETRIZ §1.5.9) em
   docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_editor_core_<data>.md
   — a saída de `bash scripts/collision-surface.sh` colada; as tabelas
   medidas (as duas da base e as do fim); a lista dos ids que ficaram pela
   cerca; o que foi construído, medido e REVERTIDO; as premissas DESTE
   bloco que a medição derrubou; e o que o dono deve exercitar no smoke.
I. No fim: rm -rf target/*/incremental ; depois DEIXE O SMOKE COMPILADO,
   2 corridas, a 2ª colada no handoff (Finished em segundos, ZERO Compiling):
   cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §4 — BLOCO `line/render-loop` (cole isto noutra janela nova)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA — Modo L · ARQUITECTURA A9 + O QUADRO   (DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/render-loop.  Alvo: o quadro da shell
deixa de ser UMA função de 13 685 linhas. Duas obras, NESTA ordem:
  1. A9 — os 54 campos `vec_*` soltos na `App` agrupam-se por ASSUNTO.
  2. O QUADRO — `run_render_frame` parte-se em FASES com um contexto de
     quadro explícito, chamadas pela MESMA ordem.
⭐ POR QUE: é a maior função do repo, e o bloco do ecrã principal sozinho
   tem 10 855 linhas e 241 sub-blocos. Nenhuma revisão lê isso, e toda
   linha que toca o quadro soma nele. A A9 vem primeiro porque o
   contexto de quadro vai levar o GRUPO, não 54 campos soltos.
⛔ ESTA LINHA NÃO MUDA PRODUTO: nenhum pixel, texto ou comportamento, e a
   ORDEM do quadro é o contrato. Se uma cura pedir mudança visível ou de
   ordem, PARE e reporte ao Enio.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh          # tem de dizer `workstation`
2. cd /home/enio/Documentos/Projetos/PH2D && git status -sb
      → RAIZ, em main. M/?? em project-memory/ são ALHEIOS: não toque.
      ⛔ NÃO faça pull: o main local está À FRENTE do origin (nada foi
        enviado) e é ELE a base.
3. mkdir -p Worktrees
   git worktree add -b line/render-loop Worktrees/line-render-loop main
4. cd Worktrees/line-render-loop
   pwd && git branch --show-current    # DEVE dizer line/render-loop
5. cargo check -p ph2d-host-desktop    # warm-up; o 1º build é frio
6. bash scripts/mergiraf-setup.sh      # idempotente; ✗ não é bloqueio
7. A BASE DA PROVA, antes de editar uma linha:
   mkdir -p target/prova
   cargo nextest list --workspace --cargo-profile ci-test > target/prova/antes.txt
   wc -l target/prova/antes.txt        # ⚠️ vazio = a prova mede nada
8. LEIA INTEIRO (dentro da worktree):
   a) docs/IntegracaoMultiAgente/BLOCOS_ABERTURA_ARQUITETURA_2026-09-12.md
      — §1 o que se sobrepõe; §2 a CERCA com a line/editor-core (LEI).
   b) docs/IntegracaoMultiAgente/AUDITORIA_ARQUITETURA_2026-09-12.md
      §2 (A1, A9) e §6.
   c) docs/IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md §3 — «o que fica
      na shell por desenho»: o render_loop é o LAÇO, e o porquê.
   d) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
      §2 (as ⛔ são MUDAS), §3 (a prova) e §4 (o que NÃO se faz).
   e) docs/IntegracaoMultiAgente/DIRETRIZ.md §0, §1.5, §6.7
   f) docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — tudo, e
      releia a cada passo.
9. Reporte "linha render-loop pronta" e SIGA (não pare).

OBRA 1 — A9: a App deixa de ter 54 campos vec_* soltos
  MEDIDO 12/09 — RE-MEÇA na base:
    struct App em shells/desktop/src/app_state.rs:496 — 247 campos.
    55 começam por vec_: o vec_state (ph2d_app_vec::VecState, em
    crates/ph2d-app-vec/src/state.rs:46) e mais 54 soltos (24 de tipos de
    crate, 30 de tipos da shell). 1 171 acessos por `self.`, 86 por
    `app.`, 5 desmontagens; 89 ficheiros da shell, 581 acessos só em
    render_loop/mod.rs. Os maiores: vec_entities 317 · vec_pen 230 ·
    vec_history 77 · vec_draw_config 73.
  O MOLDE: as famílias no padrão de chegada agrupam o estado num tipo da
    crate (VecState; Sculpt3dShellState em
    crates/ph2d-app-sculpt3d/src/shell_state.rs; MasterEcho). A vec tem o
    tipo e deixou 54 campos ao lado dele.
  A CURA, campo a campo (a tabela campo → destino → porquê vai no handoff):
    · MORTO sai — ⚠️ o CLAUDE.md §5 «Editor / shell» chama o vec_history
      de morto, «subsumido pela captura» do undo. MEÇA quem o LÊ e se o
      valor chega a um consumidor: apagar vence agrupar.
    · assunto da família + tipo de crate → VecState.
    · tipo da shell cujo assunto é da família → o TIPO desce para a crate
      primeiro (A1: um tipo desce para a folha do domínio), depois o campo.
    · o que é composição → um grupo na shell com nome de ASSUNTO
      (⛔ o teste do nome: se só o consegue descrever com «comum»,
      «misc», «shared» ou «utils», o agrupamento está errado).
  ⚠️ A ARMADILHA é o borrow checker: `self.vec.pen` e `self.vec.entities`
     continuam disjuntos por caminho de campo, mas uma função `&mut self`
     sobre o grupo prende-os todos. ⛔ Nenhum `.clone()` para calar o
     compilador — é o caminho do quadro.
  O GATE: tecto NUMERADO do número de campos da App (hoje 247), com as
    duas metades (cresceu · o tecto ficou para trás), no molde de
    shells/desktop/tests/it/file_loc_caps.rs, e prova de mutação.

OBRA 2 — O QUADRO em fases (só depois da 1)
  MEDIDO 12/09 em shells/desktop/src/render_loop/mod.rs (14 038 L; tecto
  NUMERADO 14 038 em shells/desktop/tests/it/file_loc_caps.rs):
    `pub(super) fn run_render_frame(&mut self)`   @352 — 13 685 linhas
    `let AppGfx { … } = gfx;`   @970 — desmontagem exaustiva (~70 campos)
    bloco do ecrã principal     @2915 — 10 855 L, 241 sub-blocos
    `for action in hero.bus.drain()`  @3725 — 1 699 L
    43 variáveis locais atravessam as fases
  A CURA: um CONTEXTO de quadro explícito — struct(s) com os locais que
    atravessam fases e os empréstimos DISJUNTOS de App/AppGfx — passado a
    funções de FASE, chamadas pela MESMA ordem. A run_render_frame fica o
    ÍNDICE do quadro: as chamadas de fase, lidas de uma vez.
  ⛔⛔ NUNCA ganchos genéricos: nada de `Vec<Box<dyn Phase>>`, trait de
    fase ou registo de callbacks. ESTADO_W2 §3: «o render_loop fica como
    LAÇO: ele garante a ordem de símbolos heterogéneos; o que sai são os
    CORPOS … ⛔ Não abstraia o laço.» Um corpo cujo ASSUNTO é de uma
    família vai para a crate dela, e a shell chama
    ph2d_app_<fam>::<mod>::<fn> no ponto certo (HOWTO §4).
  ⛔ A ORDEM é o contrato, e parte dela tem gate: o pick de hover no topo
    (the_highlight_has_one_source) · os sinais entre produtores e dreno
    (the_signal_table_is_wired_into_the_frame) · as máquinas de UI e o
    undo (the_ui_state_machines_run_and_undo_waits) · o dreno da
    hierarquia antes da projecção de z antes da captura do undo
    (a_hierarchy_drag_leaves_the_capture_a_fixed_point).
  ⛔ Gates TEXTUAIS que leem o corpo PELO NOME ficam MUDOS quando o corpo
    sai (HOWTO §2.9 e §2.12):
      a_baked_object_outlives_the_3d_module lê
        function_body(source("render_loop/mod.rs"), "run_render_frame");
      the_global_palette_is_wired procura `self.run_render_frame();`;
      the_ui_state_machines_run_and_undo_waits exige "fn run_render_frame".
    Reaponte cada um ao endereço novo e prove que ele AINDA reprova
    (mutação) — senão fica verde a ler um corpo vazio. Varra os outros:
      bash -c 'grep -rn "render_loop/mod.rs\|run_render_frame" shells/desktop/tests crates/*/tests'
  A PROVA DE QUE NADA MUDOU:
    · UM commit por fase; cada um compila e passa — bissectável.
    · o corpo MOVE-SE, não se reescreve: cada diff lê-se com
      git diff --color-moved=zebra --color-moved-ws=allow-indentation-change
      e só as assinaturas e o contexto são linhas novas.
    · zero alocações ou clones novos por quadro (CLAUDE.md §0.0: é o laço
      quente — se a tentação aparecer, o contexto está mal cortado).
    · o tecto do mod.rs desce a cada fase que sai (a metade «ficou para
      trás» obriga: FOLGA_MAXIMA = 20).
    · nasce o tecto NUMERADO por FUNÇÃO na shell (molde de
      architecture_panel_loc_cap: FN_OVERAGE_OK + censo de
      obsolescência), com a run_render_frame e cada fase grande numeradas
      e a descer.
    · ⚠️ a shell tem ~3 270 linhas de folga até ao TETO_LOC (187 356
      contra 190 629). Assinaturas e contexto somam; se a folga acabar, é
      sinal de que um corpo é de FAMÍLIA e sai para a crate dela — ⛔
      nunca suba o número.
  ⚠️ Sem janela, a run_render_frame devolve no primeiro `let Some(gfx)` —
    NENHUM teste corre o quadro. O smoke do dono é a única prova de
    comportamento: o handoff leva a lista de gestos por módulo que ele
    deve repetir (abrir cada módulo, desenhar, mexer, desfazer).
  ⛔ FORA desta linha: shells/desktop/src/input_dispatch.rs (7 290 L) —
    nomeie no handoff, não toque.

⛔⛔ A CERCA COM A line/editor-core (corre ao mesmo tempo que você):
  NÃO edite crates/ph2d-editor-core/src/ids/** · crates/ph2d-panel-*/src/ids*.rs
  · crates/ph2d-editor-core/tests/it/node_id_collisions.rs, e não crie
  ids (um refactor sem mudança de produto não precisa de nenhum).

O FIM DA LINHA (a prova, toda ela):
  · cargo nextest list --workspace --cargo-profile ci-test > target/prova/depois.txt
    python3 scripts/nextest-list-diff.py target/prova/antes.txt target/prova/depois.txt
    → ONLY-A = 0 (nada perdido); ONLY-B só os gates novos, nomeados.
  · cargo test -p ph2d-host-desktop --test it   À PARTE (o
    nextest-impacted não alcança shells/desktop/tests/it).
  · o lint EXACTO do ship.sh, nunca por -p (HOWTO §2.17):
    cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
    (mordeu no próprio dia desta auditoria: items_after_test_module e
    type_complexity, os dois invisíveis ao `check`)
  · cargo check --workspace --all-targets sem aviso (um -p muda a
    unificação de features e mostra outro programa)
  · cargo machete · typos --force-exclude <ficheiros tocados>
  · auditoria ≥ 2 lentes sobre o diff acumulado (/pd-auditoria)

REGRAS PERMANENTES:
A. Tudo DENTRO de Worktrees/line-render-loop/. O mesmo path relativo
   existe na raiz: editar lá é editar a árvore ERRADA. `pwd` na dúvida.
B. Sondas em bash (`bash -c '…'` ou `bash <<'X'`): o shell das
   ferramentas é zsh, que não parte palavras nem tem PIPESTATUS — uma
   sonda que devolve zero pode ser o shell, não a árvore.
C. Edite pela ferramenta Edit. Script só para renomear em N ficheiros, e
   SEMPRE com assert de contagem. ⚠️ Renomear por NOME destrói a prosa
   que cita o símbolo — o porquê deste repo vive em doc-comments.
D. Commits locais frequentes, `git commit --no-verify`. NUNCA push,
   NUNCA --force, NUNCA `git add -A`. `git rebase main` no início de cada
   jornada. Cargo.lock ou registry-init em conflito: regenere, nunca à mão.
E. ⛔ NÃO toque no TETO_LOC de architecture_the_shell_only_shrinks — soma
   entre linhas, quem o reconta é o integrador. Os tectos POR FICHEIRO
   dos ficheiros que você encolhe descem no MESMO commit.
F. Fechar = PARE. Você NÃO integra e NÃO roda foundational-integrate.sh
   nem ship.sh — é do integrador, e só por ordem do Enio.
G. PARE e reporte ao Enio SÓ se: contrato congelado (CLAUDE.md §6), rebase
   a conflitar fora dos seus ficheiros, ou uma cura que muda produto ou
   a ordem do quadro.
H. HANDOFF (DIRETRIZ §1.5.9) em
   docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_render_loop_<data>.md
   — a saída de `bash scripts/collision-surface.sh` colada; a tabela
   campo → destino da A9; a lista de fases com as linhas de cada uma; o
   que foi construído, medido e REVERTIDO; as premissas DESTE bloco que a
   medição derrubou; e os gestos que o dono deve repetir no smoke.
I. No fim: rm -rf target/*/incremental ; depois DEIXE O SMOKE COMPILADO,
   2 corridas, a 2ª colada no handoff (Finished em segundos, ZERO Compiling):
   cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §5 — Para o integrador, no dia da fusão

1. **Os ids que ficaram pela cerca** (§2): a `line/editor-core` entrega a lista; descem numa passada
   própria **depois** das duas fusões, com o censo novo a confirmar.
2. **O `TETO_LOC`** reconta-se sobre a árvore combinada, **depois** do `cargo fmt --all`.
3. **Tectos por ficheiro:** as duas linhas baixam entradas diferentes das mesmas tabelas
   (`file_loc_caps.rs` e `architecture_workspace_file_loc_cap.rs`); linhas vizinhas numa tabela
   conflituam no texto — o valor certo é o **medido** na árvore combinada, nunca um dos lados.
4. **O gate `the_draw_pass_asks_the_door_that_has_the_gates`** (shell) lê `ids/` pelo caminho: a
   `line/editor-core` pode repontá-lo e a `line/render-loop` pode repontá-lo pelo `render_loop/` —
   confira que a versão fundida lê **os dois** endereços novos.
5. **Fica com o integrador, pequeno demais para linha:** o realce âmbar que é o MESMO literal em
   `ph2d-app-flip` (`HALO_RGBA`) e `ph2d-app-motion` (`PATH_RGBA`) — pede um token (auditoria §6, item 4).
