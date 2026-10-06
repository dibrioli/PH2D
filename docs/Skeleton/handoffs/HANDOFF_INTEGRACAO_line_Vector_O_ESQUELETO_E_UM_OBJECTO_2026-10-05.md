# HANDOFF DE INTEGRAÇÃO — `line/Vector`: o ESQUELETO é um OBJECTO (A14, 2026-10-05)

> Leitor: o agente integrador (e a próxima LLM da linha). Smoke = do dono, e ele pediu-o para
> DEPOIS desta obra. Nada aqui foi integrado nem enviado. Plano:
> [`05_plano_o_esqueleto_e_um_objecto.md`](../05_plano_o_esqueleto_e_um_objecto.md); detalhe por
> commit: fila [`01_a_fila.md`](../01_a_fila.md) §F64.

## 0. Onde está e o que fazer

- Worktree `Worktrees/line-Vector`, ramo `line/Vector`. Commits desta onda: `ed0d4852f` … `HEAD`
  (`git log --oneline 0584aa495..line/Vector`), sobre a onda anterior (handoff
  [`…_A_PONTA_DO_VINCO_E_AS_PASSAGENS_2026-10-05.md`](HANDOFF_INTEGRACAO_line_Vector_A_PONTA_DO_VINCO_E_AS_PASSAGENS_2026-10-05.md),
  ainda por integrar — as duas entram juntas).
- ⚠️ **EMPILHADA sobre a `line/UIUX`** (rebase de 05/10 sobre `71056d29f`): a `line/UIUX` entra no
  `main` ANTES desta; os commits dela saem do rebase desta por igualdade. ⚠️ A `line/UIUX` tem uma
  continuação aberta (`850ac97d6`, «os furos do Vector») que vai mexer em
  `despacho_clique_vetor_premido.rs` e `vector_mode.rs` — esta onda TIROU o bloco do osso do primeiro
  (ver §1); se a UIUX integrar primeiro com mudanças ali, o conflito é desse bloco.
- Integrar: `/pd-integracao line/Vector` (DIRETRIZ §1.5.3). Só por ordem do dono.

## 1. Superfície de colisão

| sítio | o quê | natureza |
|---|---|---|
| `crates/ph2d-tool-bone/` | crate NOVA (ferramenta de osso, sabor 2) | drop-crate; `ph2d-tool-registry-init` regenerado pelo `ph2d-tool-sync` |
| `crates/ph2d-tool-vector/` | SAEM `DrawMode::Bone` (17 → 16), `BoneAction`, `WeightDirection`, `WeightMode`, constantes e ids do pincel, campos do `VectorDrawConfig`/`VectorStyleSnapshot`, `tool_weight_tests.rs` | remoção (movidos para `ph2d-tool-bone`, strings dos hashes iguais) |
| `crates/ph2d-editor-core/src/object_mode.rs`, `ids/chrome/rail.rs` | `ObjectMode::Pose` ANEXADO, `OBJECT_MODE_POSE` | foundational da `line/UIUX`, só anexo |
| `crates/ph2d-component-desc/src/lib.rs` | `ObjectKind::Skeleton` ANEXADO; `ObjectKinds::ANY` derivado de `ALL` (o literal `0b1_1111` deixava a variante nova de fora); `DRAWABLE` exclui o esqueleto | anexo + cura |
| `crates/ph2d-skeleton-ecs/` | marcador `Skeleton`, `skeleton_of`, `bones_of`; registo do esqueleto `7 → 8` | anexo |
| `crates/ph2d-ecs/src/visibility.rs` | `is_hidden_in_tree` (nova) | anexo |
| `crates/ph2d-app-skeleton/` | `skeleton_mode` (família), `object_add`, `bone_bridge`, `loose`; `bone_pick::{object_at, bind_seed}`; SAI `grabbable_outside_bone_mode` | família |
| `crates/ph2d-skeleton-live/` | `visible_bone_polylines`; `skeleton_of` de um osso com objecto = os ossos do objecto; recusa conta objectos | lei |
| `crates/ph2d-app-vec/` | `VecOverlayPlan::bones` SAI; cena `smoke_bone_modos.rs` (`NIVEIS 6 → 7`); `vector_bridge::{set_bone_action, arm_bone}` SAEM | família |
| `crates/ph2d-panel-skeleton/`, `ph2d-panel-vector/` | ids do verbo vêm da `ph2d-tool-bone`; `Bind` só em Object | painel |
| `crates/ph2d-tool-flip/src/params.rs` | braço `ObjectMode::Pose` vazio no `tools_of` | 1 linha |
| `crates/ph2d-app-registry-init/tests/it/every_object_mode_has_a_composed_family.rs` | famílias `3 → 4`, pares D6 `4 → 6` | número que SOMA — recontar se outra linha mexer |
| `shells/desktop/src/input_dispatch/` | `despacho_clique_osso.rs` NOVO (o press/release do osso, movidos de `despacho_clique_vetor_premido.rs`/`_solto.rs`); `ramo_ferramenta_osso` no `input_dispatch.rs`; sai o pick de alças do `despacho_clique_select.rs`; `Ctrl+P` | shell |
| `shells/desktop/src/render_loop/` | `fase_object_mode` (família + porta das raízes soltas), `fase_object_add`, `fase_tool_mirrors`, `fase_bus_tool_panel`, `fase_snapshots_publish` (`"bone"` na condição da caixa), `fase_vector_bone_overlay`, `fase_skeleton_verbs` | shell |
| **`shells/desktop/src/project_schema.rs` + `project_schema_tests.rs`** | **`PROJECT_SCHEMA 184 → 185`**, tripla `(185, 13, 22)` | ⚠️ número que SOMA: `python3 scripts/schema-recount.py` |

- Contrato congelado (§6): **nenhum** tocado — a `ph2d-tool-bone` IMPLEMENTA o `Tool`, não o muda.
- Shell: a catraca `the_shell_only_shrinks` ficou verde (corpos movidos, ramos do osso saíram do vetor).

## 2. O que mudou para o artista (números e decisões na fila §F64)

- **C0** os ossos desenham-se seja qual for a ferramenta (o 2.º report do dono); o olho da Hierarquia
  esconde-os, e o que não se vê não se agarra.
- **C1** o tipo `Skeleton`. **C3** a ferramenta de osso própria. **C5** os modos: *Add ▸ Skeleton*
  (um osso, entra no Edit), Edit = *Create*, Pose = *Transform*/*Weight*, `Tab`/seletor *Mode*, os
  ossos são partes (o cadeado não tropeça), um osso seleccionado responde pelo esqueleto.
- **C6** as alças do osso são do Edit e do Pose; em Object os ossos aparecem sem elas.
- **C7** em Object: clicar num osso selecciona o esqueleto, o gizmo move o esqueleto inteiro com a
  forma presa, *Bind to Skeleton* e `Ctrl+P` prendem a forma/imagem escolhida ao esqueleto
  escolhido.
- **C8** o *Bind* só em Object; a fileira dos verbos fica (é a porta do 1.º osso e a troca de modo).
- **C9** projectos antigos e cenas: cada raiz solta ganha um esqueleto no 1.º quadro, pose ao bit.
- **C10** as cenas abrem em Object; cena nova `=7` (os três modos).
- Desvios do plano, com a razão na fila: C2 junto ao C5; C4 junto ao C5; o esqueleto adoptado chama-se
  `Skeleton` (não o nome da raiz); o painel não esconde os verbos em Object.

## 3. Prova de fecho

Gate batched 1× (agente `verificador`, BASE `0584aa495`, loadavg ao fim `58,6`):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | ❌ 16 607 / 16 610 — os 3 vermelhos eram TECTOS de tamanho desta onda (`section.rs` do painel `612/600`, `smoke_bone.rs` `702/700`, a função `fase_skeleton_verbs` `205/200`) → curados por CORTE em `b82fc6cab` (testes inline para `section_tests.rs`; a `=7` por `return f()`; as formas do Bind para `bone_pick::bind_paths`) e re-medidos verdes |
| `cargo clippy --workspace --all-targets -D warnings` | ✅ |
| `cargo fmt --all --check` | ✅ |
| `file_loc_caps` · `architecture` da shell e do editor-core | ✅ (depois do corte) |
| `censos-da-arvore-combinada.sh` | ✅ 114/114, 12 de 12 censos |
| censos do `ph2d-panel-registry-init` (só valem com `--workspace`) | ✅ cobertos pelo `nextest-impacted` |

- **Mutação** (agente `mutacao`, 22 mutações nas leis novas, controlo verde com população `> 0` em
  cada filtro): **20 sangraram**; sobreviveram **M6** (o `follow` sem largar a ferramenta quando o
  modo acaba sem `leave`) e **M10** (a raiz adoptada guardava a `RootOrder`) — os dois NÃO
  equivalentes ⇒ ganharam gate em `363096b7e` (`deleting_the_skeleton_in_pose_releases_the_bone_tool`
  e a asserção no gate da porta) e foram re-aplicados: sangram.
- **Fotos** (`fotografa_cena.sh`, `kwin --virtual`, 1930×1040) das cenas `=1..7`: os ossos à vista em
  Object em todas, cada raiz com o seu esqueleto na Hierarquia, a pele intacta na `=6`; a `=7` abre
  com o esqueleto de dois ossos, a barra solta e o painel Bones (*Create* · *Transform* · *Weight* ·
  *Bind to Skeleton*). ⚠️ Visto na foto: os esqueletos ADOPTADOS nascem na identidade ⇒ o anel de
  objecto deles fica na ORIGEM (um círculo solto no centro das cenas antigas) — é o preço da pose ao
  bit; um esqueleto do *Add* nasce com o 1.º osso na origem dele, onde o anel cai bem.
- Binário de smoke (`--profile smoke`) compilado 2×: a 2.ª `Finished … in 0.45s`.
  `rm -rf target/*/incremental` feito.

## 4. ABERTO

- **A5-a**, **A13** — fora desta obra (lista viva).
- O precedente divergente do olho (sprites por entidade × esqueleto e Flip pela árvore) — registado
  no doc de `ph2d_ecs::is_hidden_in_tree`; o gizmo do Flip pode passar a chamá-la (outra linha).

## 5. A linha do `CLAUDE.md` §5 (para o integrador)

O «Último» do módulo Vector + Esqueleto passa a apontar para ESTE handoff; a frase do módulo ganha
«o esqueleto é um objecto (Object · Edit · Pose)» e o smoke `PH2D_VEC_BONE_SMOKE=7`.

## 6. Smoke (o dono — DEPOIS desta obra, como ele pediu)

Para o dono (ele pediu-o para depois da obra):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Na tela há uma barra cor de laranja com dois ossos azuis por cima. Clique num osso azul: na
   Hierarquia acende **Skeleton**. Com `Shift` carregado, clique na barra; depois `Ctrl+P`.
3. Tem de acontecer: aparece «presa» no terminal. Carregue em `Tab`: o painel **Bones** acende
   **Create**; arraste a partir da ponta da direita do último osso e nasce um osso novo. Abra o menu
   **Mode** (o 1.º do topo da área) e escolha **Pose Mode**: arraste um osso e a barra DOBRA com ele.
   `Tab` de novo volta a Object: arraste o círculo do esqueleto e tudo (ossos e barra) anda junto.
4. Deu errado se: os ossos desaparecem fora do Edit; clicar no osso não selecciona o Skeleton; o
   `Ctrl+P` não faz nada; em Pose arrastar cria osso em vez de dobrar; ou em Object a barra fica para
   trás quando o esqueleto anda.
