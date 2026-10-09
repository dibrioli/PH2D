# HANDOFF DE INTEGRAÇÃO — `line/ObjectsAndComponents`, onda 2 (08/10): a Animação por objecto

> Leitor: o agente integrador (e a próxima janela desta linha). Plano e leis:
> [`01_objeto_e_componente.md`](../01_objeto_e_componente.md) — o §12 diz o que entrou e as
> premissas que a medição derrubou. A onda 1 (Hierarquia nova) tem o seu handoff:
> [onda 1](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md). **As duas ondas vão juntas
> na mesma branch** — integre-as num só passo.

## 1. Identidade

- Branch `line/ObjectsAndComponents` · worktree `Worktrees/line-ObjectsAndComponents` · base (merge-base)
  `2a925859d` (= o `main` de 08/10; o `main` não andou, a linha não precisou de rebase).
- Commits da onda 2 (depois de `b8418ba63`): `f3cb5d7eb` (fixtura v18) · `ff6f6bdc6` (dono do clip, v19,
  migração) · `3c0b095f1` (a Timeline segue a selecção, chip «Animation: X ▾») · `7ea12b13c` (Hierarquia,
  paleta, cópia) · `f14b9f320` (Cargo.lock) · `420a8cbe8` (dope sheet filtrado + smoke) · `76ab44cb7`
  (selector de acções do osso) · `596975c89` (curas do portão) · `5dbfa1540` (teste M8 + doc 01 §12)
  · `b128b38e9` (a cópia de um PREFAB leva clips próprios — pedido do dono no smoke) + os commits deste handoff.
- Diff da onda 2: 76 arquivos, `+3004 −159` (testes e docs incluídos). **Shell: `+197 −3`** (o smoke
  `render_loop/animacao_smoke.rs` ~100 linhas + plumbing); `the_shell_only_shrinks` verde.

## 2. O que entrou (F4 do doc 01 §6, escolha 9.1 = A)

Escolhas do DONO (08/10): (1) a chave vai para a animação do CABEÇALHO; (2) duplicar dá à cópia clips PRÓPRIOS.

- **Desenho:** UM `TimelineDoc`; `NamedClip.owner: Option<ClipOwner>` (`None` = Cena). `ClipOwner{wire_id,
  entity(skip), missing(skip)}` tem a vida da binding: carimbo no save (`stamp_wire_ids`),
  `resolve_entities` no load, `refresh_and_heal_bindings` por quadro; liveness e purga em
  `ph2d-timeline-persist::upkeep` / `purge_the_dead`, num passo do undo da timeline; o reset só depois
  de purgar uma BINDING e sem donos vivos.
- **Módulos novos:** `ph2d-timeline/src/doc_owner.rs` (`clip_belongs_to`, `clips_of`, `has_animation`,
  `add_clip_for`, `remove_animation`, `purge_dead_owners`, `copy_animation`) · `doc_v18.rs` (imagem
  congelada v18 → v19, todo clip na Cena) · `animation_view.rs` (`AnimationView{owner, selected, pin}`,
  `follow`, `AnimationIntent{Show,Add,Remove}`, `AnimationHeader`, `fill_clips`, `row_belongs`) ·
  `ph2d-timeline-persist/src/animation.rs` (`follow_selection`, `publish_animation_header`,
  `copy_animations`) · `ph2d-panel-timeline/src/transport_animation.rs` (chip + popover + evento) ·
  `ph2d-panel-timeline/src/hier.rs` (`clips_content`, `declare_in_row`) · `ids/timeline_animation.rs`.
- **Comportamento:** a Timeline segue a selecção (a animação do objecto, ou a do 1.º ancestral que a
  tenha, ou a da Cena); o chip «Animation: X ▾» muda a vista e fixa (`pin`); a Hierarquia mostra o
  grupo «Clips (n)» (Cena e objectos); o 🎞 de um objecto com clips abre a Timeline; a paleta *Add
  Component* oferece «Animation»; Duplicate copia a animação com clips próprios.
- **Dope sheet:** lista só o que a animação em vista anima (+ bindings sem chave nenhuma) — achado
  na foto do smoke.
- **Osso inteligente:** `ph2d-app-skeleton/src/smart.rs::actions_for/action_at` recebem o OSSO e listam
  a animação do 1.º ancestral com Animação (ou a da Cena); cabe no pool `MAX_SMART_CLIPS` por construção;
  projecto antigo (tudo na Cena) não muda.

## 3. Versões e contadores (DELTA)

- `DOC_VERSION` (blob da timeline) **18 → 19**. `PROJECT_SCHEMA` **NÃO sobe** (187; `project_schema.rs`
  sem diff). A v18 **MIGRA** — 1.º degrau dessa escada que não recusa; as outras versões antigas
  continuam recusadas. Fixturas `crates/ph2d-timeline/tests/fixtures/timeline_doc_v18.{bin,json}`
  gravadas em `f3cb5d7eb`, ANTES da v19 (README lá).
- `MAX_CLIPS` (16) passou a ser **POR animação** (o recurso é o seletor, que mostra uma); o documento
  pára no `u16` do strip.
- Contratos congelados: nenhum tocado (`node.rs`/`tool.rs` intocados).

## 4. Superfície de colisão — o que um merge pode partir

⚠️ Assinaturas e formas que mudaram (outra linha que as use reprova a compilação da árvore combinada):

| sítio | mudança |
|---|---|
| `ph2d-app-skeleton/src/smart.rs` | `actions_for` / `action_at` ganham `osso: Entity` |
| `TimelineViewSnapshot` | `clips` = só a animação em vista; campos NOVOS `clip_index: Vec<usize>` (índice do DOCUMENTO por linha) e `animation: AnimationHeader`. Quem construa `TimelineViewSnapshot { clips: vec![..], .. }` à mão precisa de `clip_index` (duas fixturas curadas aqui: `transport_clips.rs` tests, `view_tabs_seam.rs`). `active_clip` continua índice do DOCUMENTO (novo `active_row()`, `clip_name(i)`) |
| `TimelineState` | campo novo `view: AnimationView` |
| `TimelineIntent` | variante NOVA `Animation(AnimationIntent)` — `match` exaustivo noutra linha reprova. `intent_apply::intent_loop_sync` passou a `pub(crate)` |
| `ph2d-editor-core/src/hier_rows.rs` | `ContentOpens{Mode(ObjectMode), Panel(&'static str)}`; `ContentGroup.mode` → `opens`; `RowRole::ContentItem.mode` → `opens` (construir com `opens: ContentOpens::Mode(..)`) |
| `action_bus_hier.rs` | variante NOVA `HierRequest::OpenContentInPanel{owner, panel, key}` |
| `widget/dropdown/mod.rs` + `widget/mod.rs` | `popover_width_for(text_w, has_icon)` (a lista do ▾ toma a largura do rótulo mais largo) |
| `ph2d-ecs/src/subtree_copies.rs` (**novo**, irmão) | recurso `SubtreeCopies` + `announce_copy` / `take_copies`: os pares `(original, cópia)` que o EDITOR fez; a Timeline drena-os no `upkeep` (`copy_animations`). Anunciam `instantiate_master` (Make, Instantiate, colocar um prefab) e `duplicate_subtree`; **não** anuncia `instantiate_master_many` (a Fábrica — cópias de jogo) |
| `ph2d-app-components` | `instantiate_master`/`_many` passam por um corpo privado `instantiate_copies(.., announce)` (assinaturas públicas INALTERADAS) · `component_palette::ANIMATION_ITEM` + `offer_animation` · `component_attach::open_palette_if_asked` / `refresh_palette_on_toggle` ganham `has_animation: &dyn Fn(u64)->bool` · `take_animation_pick` |
| `ph2d-panel-hierarchy` | `scene_entry()` público; a Cena recebe conteúdo pela entrada `HIER_SCENE_ROW`; o 🎞 lê as SUB-LINHAS (o `entry.content` já foi expandido) |
| `Cargo.lock` | arestas internas `ph2d-panel-timeline → ph2d-component-desc` (folha sem deps) e `ph2d-timeline-persist → ph2d-core` (de dev a `[dependencies]`); nenhum pacote externo |

- **Ids novos:** `TIMELINE_ANIM_DD = hash_node_id("timeline.anim_dd")` · `TIMELINE_ANIM_OPT[16] =
  "timeline.anim_opt_0..15"` (num irmão `ids/timeline_animation.rs`: o `ids/timeline.rs` estava no tecto
  de 600 do painel) · paleta `ANIMATION_ITEM = item_id("timeline.animation")`.
- **i18n novas:** `panel.timeline.animation_of` «Animation: {name}» · `panel.timeline.animation_add`
  «Add animation to {name}» · `panel.timeline.animation_remove` «Remove animation from {name}» ·
  `hier_rows.content.clips` «Clips ({n})» · `app.components.component_palette.animation` «Animation».
- **Shell (composição):** `fase_timeline_drain` (+`follow_selection` antes dos pedidos, +`publish_animation_header`) ·
  `fase_bus_hierarchy` (`OpenContentInPanel` → `AnimationIntent::Show` + `SetActiveClip`) ·
  `snapshots` / `snapshots_hierarchy` (+parâmetro timeline; `declare_in_row`; conteúdo da Cena) ·
  `fase_component_palette` (Animation → `AnimationIntent::Add`) · `fase_selection_mirror_bone_focus` (osso no
  `actions_for`) · `render_loop/animacao_smoke.rs` (módulo do roteador `fase_app_scene_smokes`; trava
  `static AtomicBool` — a `App` estava no tecto de 174 campos).
- `the_shell_only_shrinks` soma entre linhas: esta onda põe **+194 líquidos** além dos +35 da onda 1.

## 5. Portão de fecho (batched, sobre o diff inteiro, 2.ª corrida depois das curas)

- `nextest-impacted` **16791/16791** (3.ª corrida, depois de `b128b38e9`; a 2.ª deu 16788/16788) · censos de rótulo `--workspace` **14/14** ·
  `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ · `clippy --all-targets -D warnings` ✓ ·
  `fmt` ✓ · `machete` ✓ · `check-standalone-optional` ✓ · `check-workflow-packages` ✓.
- A 1.ª corrida reprovou 4 (16783/16787): tectos da shell (`app_state` 933>931, `main` 1063>1061,
  `App::new` 214>213, `App` 175>174 campos — a trava do smoke) e o 🎞 que lia `entry.content` já
  expandido; curados em `596975c89`.

## 6. Provas de mutação (8; controlo verde antes, cada âncora casou 1×, árvore restaurada com `touch`)

| # | mutação | veredito |
|---|---|---|
| M1 | migração v18 com dono | SANGROU |
| M2 | `row_belongs = true` | SANGROU |
| M3 | sem `purge_dead_owners` | SANGROU |
| M4 | `pin` ignorado | SANGROU |
| M5 | cópia não reaponta | SANGROU |
| M6 | `animation_of = SCENE` | SANGROU |
| M7 | 🎞 sem painel | SANGROU |
| M8 | `AddClip` na Cena | **SOBREVIVIA** (a suíte toda corria com a vista na Cena) ⇒ teste novo `animation_chip_seam::o_mais_cria_o_clip_na_animacao_em_vista` (`5dbfa1540`); re-mutado: SANGROU |
| M9 | `instantiate_master` não anuncia | SANGROU (`o_editor_anuncia_a_copia_e_a_fabrica_nao`) |
| M10 | a Fábrica anuncia | SANGROU (idem) |
| M11 | o `upkeep` não drena | SANGROU (`a_copia_que_o_editor_anuncia_leva_a_animacao_no_quadro_seguinte`) |

## 7. Testes novos

- `ph2d-timeline` `clip_owner` (7: migração v18 contra a fixtura em JSON campo a campo, dono, tecto por
  animação, tirar, último da Cena, cópia própria, dope sheet).
- Cópia de prefab (`b128b38e9`): `ph2d-app-components` `instantiate::tests::o_editor_anuncia_a_copia_e_a_fabrica_nao`
  · `ph2d-timeline-persist` `owner_tests::a_copia_que_o_editor_anuncia_leva_a_animacao_no_quadro_seguinte`.
- `ph2d-timeline-persist` `owner_tests` (3: atravessa o ficheiro, apagar+undo, duplicar num passo).
- `ph2d-panel-timeline` `animation_chip_seam` (5, cliques reais: segue / menu volta à Cena e o `pin`,
  clip pelo índice do documento, oferta+chave vai para a do cabeçalho, remove+undo, o «+»).
- `ph2d-panel-hierarchy` `the_clips_of_an_animation` (3, cliques reais).
- `ph2d-app-components` `a_animacao_e_oferecida…` (paleta) · `ph2d-app-skeleton`
  `o_osso_ve_as_accoes_da_animacao_onde_vive`.

## 8. Premissas do briefing que a medição derrubou

1. «Documento por `StableId` como o `painted`» — um documento só, e o clip tem dono.
2. «`PROJECT_SCHEMA` sobe» — não sobe (blob com versão própria); sobe o `DOC_VERSION` e a v18 migra.
3. «`ModeFamily::content` serve à Timeline» — não (`family()` acha pelo MODO) ⇒ `ContentOpens::Panel`.
4. «Componente Animação = cartão no Inspector» — é componente DERIVADO; o cartão É a Timeline; o
   catálogo exige tipo registado.
5. «`MAX_CLIPS` é o tecto do documento» — é por animação, e o selector de acções do osso teve de seguir.

## 9. Smokes (binário `smoke` compilado na worktree)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-ObjectsAndComponents
PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop   # Herói seleccionado
PH2D_ANIMACAO_SMOKE=2 ./target/smoke/ph2d-host-desktop   # Inimigo seleccionado
```

Medidos com foto (`fotografa_cena.sh`):

- `=1`: Hierarquia «Scene ▸ Clips (1)», «Herói ▸ Clips (2)» + Espada; Timeline «Animation: Herói ▾ · andar»;
  só as linhas Herói/Espada. Errado se aparecerem linhas do Inimigo ou o chip disser «Scene».
- `=2`: «Animation: Scene ▾» aberto com «Add animation to Inimigo». Errado se o texto vier cortado.
- A 1.ª foto achou dois defeitos, curados: o Inimigo nas linhas do Herói (`row_belongs`) e «Add animation
  t…» cortado (`popover_width_for`, chip 200 px).

## 10. O que fica ABERTO

- (a) ~~Instanciar um PREFAB não copia a Animação~~ — **feito** (`b128b38e9`, dono no smoke de 08/10:
  «Prefab também deve levar clipes próprios»). As cópias de JOGO (Fábrica) continuam sem clips, de
  propósito: e tocar a animação de um objecto no relógio dele durante o Play é outra peça (hoje toca
  o clip activo da vista ou o Arrange da Cena, como antes do F4).
- (b) Clip copiado ganha nome único «andar 2» (nomes de clip são únicos no documento porque o osso
  inteligente referencia pelo nome): um osso da cópia que diga «andar» toca o do original.
- (c) O undo de apagar um objecto com Animação são DOIS Ctrl+Z (o global devolve o objecto, o da
  timeline devolve os clips) — a mesma lei das tracks de sempre.
- (d) Add Clip acima do tecto por animação é recusado em silêncio (como antes do F4).
- (e) O seletor de clips do Arrange só oferece a animação em vista (para pôr o clip do Herói na Cena,
  mude o ▾ para o Herói).
- **Onda 3** = F5 (Motion como objecto-gerador, doc 01 §8, escolha 9.3) + F6 (busca por intenção no
  Add Component + filtro da Hierarquia por componente).

## 11. Fecho

- `rm -rf target/*/incremental` e o build `smoke`, 2.ª corrida (ainda por fazer):

`rm -rf target/*/incremental` feito; `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

- `bash scripts/agent-loop-profile.sh` (20 sessões):

```text
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5
  ✓ respostas por sessao (mediana)                234   alvo: <= 800
  ✗ cargo test : cargo check                420 : 264   alvo: <= 1,0  razao 1.6x
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%
  ✗ contexto relido por passo (media)         487 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
