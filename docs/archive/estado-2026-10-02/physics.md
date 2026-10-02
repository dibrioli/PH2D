# ARQUIVO — CLAUDE.md (história, 65 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-735,801-812` do original.
>
> ⚠️ **A única alteração ao corpo:** 16 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Física** — *runtime-truth* + bake opcional sobre o `rapier2d` que já existia; a linha escreve **integração e
  autoria, não solver** ([ADR-0131](../../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)).
  Crate-ponte `ph2d-physics-ecs` (components de **CONFIG**, nunca estado vivo de solver — o undo ordena por bytes),
  ring de checkpoints GGPO, painel global de mundo (tecla `W`), camadas de colisão, **joints como ENTIDADES**
  (7 tipos + polia/talha/tambor + pino de mundo, [ADR-0149](../../architecture/decisions/0149-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md)),
  a família das **zonas** (força/arrasto/empuxo/torque/falloff/frame/espelho), contatos e sinais, e o
  **player de plataforma** (`ph2d-platformer`, lei pura) em três modos.
  ⚠️ **`BTreeMap`, nunca `HashMap`** — é a espinha do determinismo (lint estrutural), e o hash `physics_ecs_c9` roda na
  matriz 3-OS provando **o nosso** código, não o wrapper.
  ⚠️ **Sem porta de escala:** o `Transform` já é metros; a única px→m é `ProjectSettings.pixels_per_meter`, do projeto.
  **Aberto:** o campo de **atração** não alcança um player de pose própria (é **sustentado**, pede canal por-tique) ·
  `bXYOverride` do Unreal, quando houver quem peça · a trava de beirada não tem gesto de canvas · **quatro ❌** na
  [auditoria 09](../../Physics/09_auditoria_engines.md) e ⚠️ **nenhum é trabalho pendente** (dois foram **recusados por
  medição**, um é arquitetura com *não agora* escrito, um está fora da fila) — *um ❌ «recusado com motivo» e um ❌
  «ninguém fez» leem igual numa tabela* · o buraco real contra o referencial é *obstacle actions: climbing* (plano 08 §4.8) ·
  ⭐⭐ **A família SAIU da shell pela metade (W2/L2 Fase A, 11/09):** as cenas e as leis vivem em
  [`ph2d-app-physics`](../../../crates/ph2d-app-physics) (112 ficheiros, 22,5 k LOC) e o que precisa da `App`
  agrupou-se em [`shells/desktop/src/physics/`](../../../shells/desktop/src/physics) — `impl App` de **88 para 13**
  blocos, 7 campos de `App` num `PhysicsState`, e a shell de **493 k para 471 k** linhas, com **zero**
  contador partilhado a mexer-se. ⭐⭐ **E a Fase B CORTOU** (11/09, [handoff](../../Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_B_2026-09-11.md)):
  a shell de **465 105 para 451 079** linhas, prova `nextest-list-diff` **exacta nos dois sentidos**
  (ONLY-A **0**, 130 `MOVED`, e os 4 `ONLY-B` são os gates novos do roteador), e a resposta que a **batedora** deve às
  outras quatro linhas — ⭐ **as 5 portas do `AppHost` CHEGARAM, zero sextos métodos**: o que bloqueia
  não é a `App`, são **três FOLHAS partilhadas da shell** (`inspector_ordering` · `preview_drive` ·
  `name_unique`, com 11/38/14 consumidores de famílias diferentes), que são alvo de uma linha própria.
  ⭐⭐⭐ **E O FIM DA LINHA FOI ALCANÇADO**: o `match` de 117 níveis e o `PAUSED_SCENES` vivem em
  [`ph2d_app_physics::smoke`](../../../crates/ph2d-app-physics/src/smoke.rs), a env é lida lá, o `const FAMILY`
  declara-a com `max_level: smoke::CENAS` (**contado** do `match`, com gate nas duas pontas e prova de
  mutação) e **`"physics"` SAIU** de `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`. ⭐ **O que destravou foi uma DECISÃO, não um
  refactor:** os três últimos braços autoram uma track de timeline, e a família passou a poder depender
  da `ph2d-timeline` — *uma crate-motor irmã não é a shell* (ADR-0075). ⚠️ **O prólogo FICA na shell**
  (rebobinar, armar o toggle, abrir a timeline, play/pause): *o que sai são os CORPOS; o que decide a
  ordem do quadro fica.*
  ⚠️ E a §5 do handoff tem as **oito** armadilhas que esta fase pagou, entre elas que uma régua textual
  a varrer `\bApp\b` **lê o doc-comment que EXPLICA a cura** e acusa 93 ficheiros de 133, e que o
  ÓRFÃO e o DUPLICADO são as duas metades do MESMO audit (13 e 3 achados), os dois **mudos**. O
  §6 do [handoff da Fase A](../../Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_A_2026-09-11.md) tem a lista
  **NOMEADA** do que só o substrato resolve (timeline · playhead · readout · inspector da roldana · ponteiro) —
  ⚠️ e o §9 as **nove** armadilhas medidas, entre elas que apagar um `mod` re-liga o `#[cfg(test)]` dele ao
  **vizinho**, em silêncio, e que o censo de cenas do briefing apagaria **28 cenas VIVAS** (elas são citadas
  pelo atalho `` `=N` ``, não pela env).
  ⭐⭐⭐ **E a FASE C FECHOU O CORTE (12/09):** os 48 ficheiros que sobravam em `shells/desktop/src/physics/`
  vivem em [`ph2d-app-physics`](../../../crates/ph2d-app-physics) — mais o `render_loop/point_gizmo{,_tests}.rs`,
  que era **física com nome genérico** (ele já importava `ph2d_app_physics::overlay::joint_glyphs`) —, e a
  shell desce de **373 937 para 360 599** linhas com `ONLY-A = 0`, `ONLY-B = 0` e 169 `MOVED`.
  ⭐ **As 15 625 linhas estavam presas por SEIS SÍMBOLOS**, e cinco só eram tocados por testes: duas
  curaram-se (o `GroupDragSnapshot` foi para o `ph2d-editor-core`, onde o `TransformSnapshot` dele já vivia)
  e **quatro ficam por DESENHO** — o `build_component_registry` regista os componentes de cinco crates
  irmãs, logo é **composição**, e os gates que atravessam essa porta moram com o que exercitam (HOWTO §2.6).
  ⛔ **Zero sextos métodos no `AppHost`**: escritos em tipos, os quatro `impl App` pediam coisas que a `App`
  por acaso segurava, e ⭐⭐ **o `std::mem::take` do `resolve_player_input` DESAPARECEU** — *o truque não era
  lei do domínio, era o preço de a função estar na struct errada*. ⚠️⚠️ **E o `cargo check --all-targets`
  estava VERDE com 15 gates VERMELHOS**: eles vivem em `shells/desktop/tests/it/`, que um `check` não
  corre (⛔ o `nextest-impacted` ALCANÇA-os por `rdeps`) — entre eles o gémeo mudo do §2.6 que **sobreviveu dentro da crate** (curado com
  `include_str!`, que falha a compilar) e a **quarta** agulha deste repo a nomear a **visibilidade** em vez
  da lei. [Handoff da Fase C](../../Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_C_2026-09-12.md)
  (⚠️ o §5 tem as **oito** armadilhas e o §6 as **quatro** premissas minhas que a medição derrubou — entre
  elas uma acusação de cegueira à régua do fecho que **era falsa**).
  **Smokes:** `PH2D_PHYSICS_SMOKE=<n>` (⚠️ **`=84` não existe, de propósito**; ⚠️ **a `=15` tem as
  paredes CINEMÁTICAS desde 30/08 e isso é load-bearing** — com paredes estáticas as duas bolas
  param no mesmo sítio desde a `rapier` 0.35, e a cena passa a ensinar o contrário do que diz).
  **Ler:** [`docs/Physics/`](../../Physics) · tracker [`HANDOFF_line_physics.md`](../../Physics/handoffs/HANDOFF_line_physics.md) ·
  [`00_plano_waves.md`](../../Physics/00_plano_waves.md) · [`BUGS_physics.md`](../../Physics/BUGS_physics.md) ·
  [handoffs](../../Physics/handoffs/README.md) · [história](../estado-2026-08-18/physics.md)

