# HANDOFF DE INTEGRAÇÃO — `line/anim` (2026-07-12, **final**)

> **Para:** o **agente integrador** (e o Enio, para a decisão de ordem em §6).
> **De:** o agente da linha `line/anim`.
> **Estado:** linha **FECHADA**. Smoke do Enio **OK**. Não integrei e não fiz ship (CLAUDE.md §0.7).
>
> Substitui a versão anterior deste arquivo (escrita em `a762aebf`, antes de toda a composição de
> clips). O único item que exige decisão humana antes de integrar está em **§6** — leia-o primeiro.

---

## §1 — Cabeçalho

| | |
|---|---|
| **Branch** | `line/anim` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim/` |
| **Base** | `3805f650` (= `main` local) — **fast-forward limpo**; `main` não andou desde então |
| **HEAD** | `26e26725` — **27 commits** |
| **Diff** | 89 arquivos, +11 120 / −1 388 |
| **Gate** | `nextest-impacted` **3752/3752** · workspace inteiro verde · clippy paridade-CI (`--all-targets --features ph2d-spike/bevy_ecs -D warnings`) **0** · `fmt --check` (rustup **1.95**) 0 · `typos` 0 · todos os arch/LOC gates |
| **Auditoria** | **2 lentes adversariais** no fechamento → **7 defeitos, 2 CRÍTICOS**, todos corrigidos em `26e26725` com repro executável (§5) |
| **Contratos congelados (CLAUDE.md §6)** | **NENHUM tocado** — `Tool`/`NodeOp`/`OpResolver`/`NodeManifest`/`PanelEvent`/vector-doc intactos |
| **`DOC_VERSION`** | **3 → 4** (§4 — quebra de save da timeline, deliberada) |
| **⚠️ Colisão com `line/motion-value` (VIVA)** | **§6. A árvore combinada NÃO COMPILA — e `merge-tree` passa.** |

---

## §2 — O que a linha entrega

### 2.1 — Composição de clips ([ADR-0115](architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md))

O "NLA": faixas de **instâncias de clip** (strips), empilhadas, com **crossfade por sobreposição**.

**Portar o strip-stack do Blender foi DESCARTADO pela pesquisa** (5 frentes) — e o ADR registra o
porquê, porque a decisão vai ser questionada de novo:

- o **próprio Blender** está movendo blend/influence do strip para a **CAMADA** (projeto Baklava),
  cortou 5 modos de blend para 2 e quer matar o tweak mode. Veredito **deles**: *"not a pleasure to
  work with"*;
- os sequenciadores (Unity/Unreal/Maya/MotionBuilder) **convergiram** no gesto **sobrepôs-cruzou** —
  exatamente o que falta ao Blender;
- e em **2D** "empilhar e blendar" **não é o idioma**: Animate/Harmony/AE têm **zero** blend de
  animação; a Cavalry escolheu camadas por-atributo; o Moho resolve overlap por canais disjuntos. O
  idioma 2D é **nesting** — e nós temos zero. **É o ADR seguinte** (§7).

O modelo que ficou, em uma frase: **a sobreposição É o crossfade** (ninguém digita uma duração, e não
existem dois números para manter de acordo), os canais são **esparsos** (o que o clip keya É a máscara
— sem Avatar Mask), e blend/peso vivem na **FAIXA**, não no strip.

Três propriedades que caíram de graça e valem por si:

- **Pesos complementares.** `smoothstep` é antissimétrica, então dois strips sobrepostos somam
  **exatamente 1** — e peso complementar **não precisa de valor-base**. Logo o crossfade é imune ao
  "afundar rumo à pose default" que a Unity combate com um `AnimationOutputWeightProcessor` inteiro.
  Num `TranslationX` (posição **absoluta**) isso jogaria o sprite na origem do pai.
- **A inversão afim.** Toda operação da pilha é afim no valor do clip sondado, então
  `out(v) = A·v + B`, e **duas avaliações fixam a reta**. Keyar sob a pilha é **exato**, não iterativo.
  Onde `A ≈ 0` a key é **RECUSADA** (R9) — nunca escrita movendo o objeto por baixo.
- **Escala multiplica, não soma.** Somar dois clips de escala 1.0 dá 2.0 — o bug (Blender T47035) que
  forçou o COMBINE a existir. `PropKind::algebra()` separa `Sum` de `Ratio`.

**E a recusa FALA** — uma vez, não 60×/s: `KeyRefusal` nomeia as 3 causas (`NotPlaying` · `PlaysTwice`
· `Overridden`) e o `autokey_pass` levanta o toast na **borda de subida**. Uma recusa calada é
indistinguível de um bug: o animador arrasta, o objeto volta, e nada explica.

### 2.2 — Relógio único (W4.T7) — **o `MotionTransport` MORREU**

Eram **três** cópias do tempo. O transporte do Motion foi **removido**; o tick que o cook renderiza é
**derivado** do `Playhead`. Não há o que divergir — **por construção, não por disciplina.**
`motion_bridge::ticks_owed()` isola o determinismo: play = **todo** tick à frente (a sim é sequencial);
scrub/jump = **uma** chamada, sem replay.

**É esta entrega que colide com a linha Motion. §6.**

### 2.3 — Raiz do gizmo + semântica de canal no fit do record

`Transform.rotation` vinha de um **par** de `atan2` (ambos em `(-π, π]`), então saltava **2π num frame**
no corte de ramo. Na tela era invisível (rotação é mod 2π) — por isso ninguém viu. Um giro de 2 voltas
gravava com span de **0.00 rad**: o giro **sumia**. A contagem de voltas virou **estado do arrasto**
(`GizmoDragState.turns`), consumida **dentro** do `compute_gizmo_transform`, para que nenhum chamador
futuro possa esquecê-la.

**Mudança de UX autorizada pelo Enio:** o Inspector mostra **graus acumulados** (430°, não 70°) — é o
que Blender e AE fazem e o que animação exige. A **view do gizmo não muda**.

---

## §3 — Superfície nova

**Crates novas:** nenhuma. **Crates removidas:** nenhuma.

`ph2d-timeline` ganhou 6 módulos (`stack`, `stack_eval`, `stack_edit`, `clock`, `refusal`,
`intent_apply`) e exporta:

```
ClipLane · ClipStrip · StripId · StripLoop · LaneMode · MAX_LANES
LaneView · StripView              (no TimelineViewSnapshot)
KeyRefusal · key_home             (a recusa, com o motivo)
TimelineDoc::prime_stack          (§5.3 — é uma armadilha, leia)
```

**11 variantes novas** em `TimelineIntent` (`AddLane`…`SetStripSpeed`) — enum **apendado**, nunca
reordenado.

**Foundational tocado** (Modo L permite — ADR-0107; todos por **extensão**, nenhuma assinatura
existente quebrada):

| arquivo | o quê |
|---|---|
| `ids/chrome/timeline.rs` | `TIMELINE_ADD_LANE`, `LANE_MUTE[8]`, `LANE_ADD_STRIP[8]`, `LANE_WEIGHT[8]`, `LANE_ROW[8]`, `timeline_strip_hit_id()` |
| `ids/menus_timeline.rs` | **arquivo novo** — as 5 tabelas de menu da timeline saíram de `menus.rs` (que cruzou os 700 LOC) |
| `interaction/types.rs` | `TimelineHitKind::{Strip, LaneHeader}` · `ContextMenuKind::{TimelineStrip, TimelineLane}` |
| `pointer_down_menus.rs` · `context_menu_overlay.rs` · `pre_populate.rs` | 1 braço/linha cada, para os 2 menus |
| `ph2d-i18n` | `panel.timeline.add_lane` |

> ⚠️ **`TimelineHitKind::Lane` já existia** — é a linha **vazia** do dope-sheet (onde nasce o marquee).
> A minha é `LaneHeader`. Duas coisas chamadas "lane" é obra da própria timeline (uma track row e uma
> faixa da pilha); se algum merge tentar **unificá-las, está errado**.

---

## §4 — `DOC_VERSION` 3 → 4 (quebra de save, deliberada)

`TimelineDoc.stack` (a pilha) e `TargetBinding.rest` (a pose de repouso capturada) são campos
**apendados**. Postcard é **posicional** → um save v3 é **rejeitado**, não mal-lido. Um documento com
pilha **vazia** se comporta **byte-por-byte** como v3 — é o gate
`an_empty_stack_is_the_single_clip_path_value_for_value`.

O `rest` existe porque `TranslationX` é posição **absoluta**: sem ele, uma faixa com peso < 1
"blendaria para o default" e jogaria o sprite na origem do pai. É o *Capture Base State* do Rive /
*Base Pose* do Unreal.

**Nada mais no repo serializa timeline** — nenhum outro `SCHEMA_VERSION` muda.

---

## §5 — Armadilhas que a auditoria expôs (leia antes de mexer neste código)

A auditoria final (2 lentes adversariais — uma delas montou uma **crate-sonda contra o código real**)
achou **7 defeitos, 2 CRÍTICOS**, todos corrigidos em `26e26725` com repro executável. Não listo os
fixes (estão no commit). Listo o que **sobrevive como conhecimento**:

### 5.1 — A faixa **não é uma escada**, e a janela de blend não pode supor que é

`blend_out(i)` perguntava a `strips[i+1]` — o vizinho na **ordem do vetor**. É o strip certo **apenas**
se os strips formarem uma escada, e **nada os obriga a isso** (o arrasto do corpo não tem clamp contra
vizinhos). Solte um strip curto **dentro** de um longo e o longo passa a desvanecer contra um vizinho
que já acabou: a cobertura desaba e **o sprite rasteja de volta ao rest no meio de um clip que não se
move** (500 → 104, medido). Agora a janela vem de **todo** strip vivo naquela borda.

### 5.2 — Uma key pode mover a **referência** contra a qual ela é medida

Numa faixa **Additive** o delta é medido contra o valor do próprio clip em `src_in`. Keye no primeiro
frame do strip — **onde o animador começa a posar** — e a key que você escreve **É** a referência: o
delta sai zero, a pose é descartada, e todo *outro* frame da faixa translada pelo valor que você acabou
de inventar. A sonda mantinha a referência **fixa**, então o solve reportava influência cheia onde a
verdade é nenhuma. Agora a sonda modela a **escrita** (`Probe{clip, value, t_key}`).

### 5.3 — `prime_stack` antes de qualquer pergunta à pilha

O `scratch` (strips vivos + relógios) é reconstruído **dentro do apply**. Quem pergunta "onde a key
cai?" ou "essa pose é alcançável?" pede *agora* e era respondido *quando o apply rodou por último*. Em
produção coincidem — **e é exatamente isso que torna o acoplamento invisível**. É a classe de bug que
já quebrou este módulo **quatro vezes** ([[feedback_derived_coordinate_seed_must_match_sample]]).

> **Regra:** todo caminho que autora key chama `doc.prime_stack(t)` **primeiro**. `key_home` tem
> `debug_assert` conferindo a promessa. Não é opcional.

### 5.4 — A inversão **verifica** a afinidade; não acredita nela

Dois pontos passam uma reta por **quaisquer** duas amostras — não dizem o que houve **entre** elas. Uma
3ª sonda confere a reta e **recusa** todo caso em que o mapa não é afim (o mesmo clip numa faixa
`Override` **e** numa `Ratio` ao mesmo tempo é **quadrático** em `v`).

### 5.5 — Ordem de registro de hit é **load-bearing**

O hit index resolve para o **último** id registrado sobre o ponto, e os strips de uma faixa **podem se
sobrepor** — isso **É** o crossfade. Registrar cada strip inteiro (corpo, depois suas bordas) punha o
**corpo** do strip da direita **em cima da borda direita** do strip da esquerda — justamente a grip que
se usa para ajustar o crossfade recém-criado. **Dois passes: todos os corpos, depois todas as bordas.**
A ordem virou função **pura** (`hit_plan`) com gate — nenhum teste de `apply_event` a alcançaria.

---

## §6 — ⚠️ COLISÃO COM `line/motion-value` (VIVA) — decisão do Enio, **antes** de integrar

**A árvore combinada NÃO COMPILA. E `git merge-tree` passa** — o conflito **não é textual**.

| | |
|---|---|
| `merge-tree HEAD line/motion-value` | conflito **só** em `CLAUDE.md` (§5, ambas editam). Todo o Rust: **auto-merge limpo** |
| **Mas** | `line/anim` **removeu** `MotionState.transport`; `line/motion-value` tem **13 usos** de `state.transport` (7 em `motion_bridge.rs`, 5 nos testes, 1 no `motion_state.rs`) |
| **Resultado** | o merge textual "funciona" e a **compilação quebra** — ou, pior, um resolvedor apressado **restaura o campo** e a divergência de relógio volta **em silêncio** |

**Arquivos que as duas linhas tocam:** `CLAUDE.md` · `crates/ph2d-eval-motion/src/lib.rs` ·
`shells/desktop/src/motion_state.rs` · `motion_state_tests.rs` · `render_loop/motion_bridge.rs`.

**Os 7 usos vivos** (`line/motion-value:motion_bridge.rs`): `transport.play()` · `transport.toggle()` ·
`transport.advance(1)` · `transport.tick` (×2) · `transport.playhead(fixed_dt)` (×2).
Todos têm equivalente **exato** no `Playhead` — a tradução é **mecânica**, não é redesenho.

### Recomendação (a ordem é sua; o custo é assimétrico)

**Integre `line/anim` PRIMEIRO.** Depois a linha Motion rebasa em cima e **re-aponta os 7 usos para o
`Playhead`** — que é a direção para a qual o W4.T7 existe.

O inverso (Motion primeiro) faz o `MotionTransport` **ressuscitar** dentro da integração do anim, e o
integrador teria que **refazer o W4.T7 no meio de um merge** — o pior lugar possível para uma decisão
arquitetural.

> **Não negociei nada com o agente da linha Motion** (CLAUDE.md §0.2). Estou reportando ao Enio.

**`CLAUDE.md`** é o único conflito textual: ambas editam o §5. É conflito de **parágrafo**, não de
semântica — as duas entradas (Timeline e Motion) coexistem. Resolva mantendo **as duas**.

---

## §7 — O que fica ABERTO (escopo, não dívida escondida)

| item | onde |
|---|---|
| **Nesting** — o idioma 2D de verdade (Precomp/Symbol), que nós **não temos** | **próximo ADR**. Nomeado, não varrido para debaixo do tapete |
| **W4.T4** — dock da timeline no `motion_timeline_slot` | **BLOQUEADO** até a linha Motion fechar |
| Ajustes de UX da pilha ("faremos ajustes depois" — Enio, pós-smoke) | próxima sessão |
| `ph2d_motion_doc::MotionTransport` (o **tipo**) ainda existe, órfão | remoção é da linha Motion (§6) |
| W4.T6/B5 (save cena+timeline) · markers→signals · export | fila da timeline |
| Varredura do `set_dropdown_popover` órfão em outros painéis | pré-existente, fora do escopo |

---

## §8 — Protocolo

Segui **CLAUDE.md §0.7** e [[feedback_integration_only_enio_command_end_of_all_lines]]:
**não** integrei · **não** pushei · **não** rodei `ship.sh`. A linha **fecha aqui** e **PARA**.

O integrador é um **agente dedicado**, por **ordem explícita do Enio**, munido deste handoff.

**O que a memória do projeto já pagou caro para saber** (vale ouro na integração):

- [[feedback_pipe_masks_script_exit_code]] — `| grep` faz `$?` virar o do `grep`. O
  `foundational-integrate.sh` **falha e você lê 0**. Confira o **ESTADO**, não o exit code.
- [[feedback_sweep_conflict_markers_every_commit]] — varra `<<<<<<<` em **cada** commit. Árvore limpa
  não prova o histórico.
- [[project_integrator_ship_catches_latents_budget_iterations]] — o gate por-linha **não** roda
  fmt/clippy-all/machete/deny. **Só o ship completo vermelha.** Orce 2–4 iterações.
- [[project_integration_prefork_lines_ship_drift]] — `foundational-integrate.sh` **não** roda
  fmt/typos.

---

**Em uma linha:** a linha está fechada, verde e com smoke aprovado; o único bloqueio é a **ordem de
integração vs. `line/motion-value`** (§6) — porque aquele merge é **verde por fora e quebrado por
dentro**.
