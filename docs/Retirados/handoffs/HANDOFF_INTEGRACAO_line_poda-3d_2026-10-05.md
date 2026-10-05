# HANDOFF DE INTEGRAÇÃO — `line/poda-3d` (2026-10-05) — o 3D sai do PH2D

> Ordem EXPLÍCITA do dono (05/10): retirar todo o 3D — 3D Modeling (modelador SDF + modo Render),
> 3D Sculpt (escultura, retopologia, tecido, tinta fina, Painter na peça, forma doada/assada,
> catavento) e Render3d — e LIMPAR o código. É uma PODA: o código saiu, nada ficou desligado.
> Decisão: [ADR-0179](../../architecture/decisions/0179-o-3d-sai-do-ph2d.md). História: `b1a6f9b07`
> (último `main` com 3D) e `~/Documentos/Backups/PH2D/ph2d-main-2026-10-05.bundle`.

## 1. Identidade

- Branch `line/poda-3d` · HEAD = o commit que traz este handoff (o último da branch) · merge-base `b1a6f9b07` (= `main` no fecho) · 7 commits.
- `git diff --shortstat b1a6f9b07..HEAD`: **3 806 ficheiros, +2 241 / −784 102**.
- ⚠️ O 1.º commit (`83795820b`) leva as remoções da shell sem as costuras: a shell só compila a
  partir do 2.º (`d82d1629d`). Integre a linha inteira, nunca o 1.º sozinho.

## 2. Medido — antes / depois

| | antes (`b1a6f9b07`) | depois | Δ |
|---|---|---|---|
| pastas em `crates/` | 390 | 347 | −43 |
| membros do workspace | 403 | 360 | −43 |
| linhas `.rs` em crates+shells | 2 702 458 (10 120 ficheiros) | 2 151 875 (8 309) | **−550 583 (−20,4 %)** |
| shell `src/` · shell inteira | 153 696 · 196 867 | 149 372 · 185 076 | −4 324 · −11 791 |
| `#[test]` no fonte | 31 432 | 26 363 | −5 069 |
| nextest `--workspace` (ci-test) | 28 143 (ship de 04/10, dado do dono) | **24 186 passaram / 24 186** | −3 957 |
| `cargo check --workspace --all-targets` a frio, sem sccache | 62 s · 837 unidades · load 1,30 | **60 s** · 781 unidades · load 1,87 (1 min) | |
| `Cargo.lock` | — | −75 pacotes (43 nossos + 32 de terceiros), **zero versões novas** | |
| docs | `docs/3D` 66 MB · `3DModeling` 2,1 MB · `Render3d` 964 KB · `Render` 20 KB | apagados | |

## 3. Foundational / partilhado tocado (tudo SUBTRACTIVO)

- **ph2d-ecs**: `BakedForm`, `Sculpt3dPieceRef`, `Mesh3D` saem do registo (**109 → 106**).
- **ph2d-component-desc**: `ObjectKind::Model3D/Sculpt3D` (o enum é DERIVADO, nunca gravado — e eram
  as duas últimas variantes), `ComponentCategory::Model3D`, `ObjectKinds::MODEL3D`, o catálogo `field`;
  pontes de identidade 5 → 3.
- **ph2d-editor-core**: `ObjectMode::Sculpt` (ALL 5→4), `AddGroup::ThreeD` (4→3), `TaskLayout::Modeling3d`
  (6→5), `ComponentEdit::Mesh3d`, ids `model3d/sculpt3d`, `mesh3d_edits`, `LIVE_SECTIONS` 47→46. Ids de
  barra de rolagem 840/843 ficam RETIRADOS (o livro-razão nunca reusa).
- **ph2d-i18n**: 10 catálogos e **961 chaves** 3D; **+2** chaves novas
  (`shell.project_load.project_migrated_from{,_128}_without_sculpture`).
- **ph2d-render / ph2d-script** (espelhos do registo): **110 → 107**. **ph2d-light**: perde só o céu/barro
  e o serde do objecto assado; o rig do impasto fica. **ph2d-gpu**: deixa de pedir `PRIMITIVE_INDEX` e
  de subir `max_vertex_buffers` (só a malha 3D os usava; as pipelines 2D usam ≤ 2).
- **ph2d-wet-paint**: `reproject_grid` (só a peça 3D o chamava). **ph2d-bloom**: fica só `BloomParams`.
- **Registos gerados** regenerados pelos geradores (`ph2d-app-sync` 8 famílias, `ph2d-panel-sync` 25
  painéis, `ph2d-tool-sync`, `ph2d-node-sync`).
- **Shell**: fases 3D, `fase_cataventos`; `fase_gizmo_suppression` (ex-`…_and_field3d_frame`) fica só com a
  metade 2D; App 178 → 174 campos; `CHROME_BACKDROPS` (só o gizmo 3D o lia).

## 4. Superfície de colisão (colada de `scripts/collision-surface.sh`)

```
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        183   (base: 182)
  ⚠   └ tripla do gate               (183, 13, 22)   (base: (182, 13, 22))
  ⚠ FIELD_DOC_VERSION                     —   (base: 23)
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-ecs                              106   (base: 109)
  ⚠ ph2d-render (espelho)                 107   (base: 110)
  ⚠ ph2d-script (espelho)                 107   (base: 110)
▸ CONTRATO CONGELADO (§6): node.rs intocado · tool.rs intocado
▸ ADR: esta linha cria 0179 — reconte contra o main do dia
▸ Cargo.lock: os 7 «+name» são linhas reordenadas (o `smallvec` deixou de precisar de versão no
  nome); conferido: zero (nome, versão) novos.
▸ TETOS DE LOC: app_state.rs 931 e main.rs 1061 — caps próprios em file_loc_caps, BAIXADOS (976, 1118).
```

- **Degrau `182 → 183`** (escada + tripla): saem do `ProjectFile` os documentos da escultura e do campo
  e as formas assadas; ECS −3, espelhos −3, o registo do campo inteiro. **Sem degrau de migração** ⇒
  um projecto salvo entre v129 e v182 é **recusado** em voz alta. Os v95/v128 (migração própria)
  abrem: a escultura fica para trás **e o toast di-lo**; com objectos assados são recusados.
- ⚠️ Se outra linha subir o schema entretanto: `python3 scripts/schema-recount.py` e conte o DELTA
  contra o `main` no ficheiro; os contadores do registo idem (ECS −3, espelhos −3).
- **Linhas vivas que tocam o 3D** (decisão do integrador/Enio, NÃO TOCADAS): as worktrees
  `line-3DModeling` e `line-sculpt3d` e as branches `line/3DModeling`, `line/sculpt3d`, `line/app-sculpt3d`
  (0 commits fora do main, limpas); a branch `backup/line-sculpt3d-tecido-2026-09-06` (84 commits fora do
  main — backup); a pasta externa `~/Documentos/Projetos/ph2d-quadbench` (852 MB). Integrá-las depois
  desta linha reintroduziria código que não compila — apagá-las é a cura.

## 5. Gates — o que mudou e porquê (nenhum afrouxado)

Cada número novo é MEDIDO e leva a linha de porquê (ADR-0179). Os principais: `the_shell_only_shrinks`
TETO_LOC **196 990 → 189 041** (shell medida 185 041 + folga 4 000 — ⚠️ o doc do gate diz que só o
integrador reconta: reconte na árvore combinada) · `the_app_only_sheds_fields` 178 → 174 ·
`file_loc_caps` app_state 976→931, main 1118→1061, `main.rs::new` 224→213 ·
`every_object_mode_has_a_composed_family` 5→3 famílias · `nenhum_rotulo_do_app_pinta_nada` 27→25 painéis ·
`cada_motor_da_fronteira` piso 20→8 (16 eram do motor da escultura) · `no_untracked_writes_in_the_sim_crates`
5→4 · 9 flakes de carga das crates apagadas saem das DUAS escritas (o 9.º, `measure_normals_parallel_speedup`,
faltava no mapa). Gates que ficariam a medir uma lista VAZIA foram reescritos com casos 2D
(`the_scene_asks_the_one_chrome_door` agora exige a porta única a `painter_canvas_down`,
`try_eyedropper_sample` e `try_add_area_click` — mais estrito que antes).

⭐ **Gate NOVO — o oráculo da luz do impasto**: `a_luz_do_impasto_com_relevo_e_a_de_antes_da_poda`
(ph2d-tool-painter). O mesmo ficheiro correu numa worktree em `b1a6f9b07` e nesta árvore: 4 materiais,
relevo cruzado, hash dos bytes iluminados — **iguais ao bit**. Mutação (normal z 1.0→1.001) sangra.
A paridade CPU×placa (`impasto_light_gpu`, `--run-ignored`, GPU real) passa 4/4. Os ficheiros que o
agente do Painter REPÔS inteiros (pré-3D) foram conferidos commit a commit: entre a versão reposta e
`b1a6f9b07` só passaram commits `sculpt3d`/`escultura`.

## 6. Prova de fecho

- `./scripts/ship.sh` COMPLETO: fmt · clippy `--all-targets` · machete · deny · audit · typos ·
  opcionais sozinhas · pacotes dos workflows · índices de ADR/docs/arquivo · **nextest 24 186/24 186**
  (load 3,35 no início) — verde na 2.ª corrida (a 1.ª acusou `the_painted_control_reaches_a_consumer` e o índice do arquivo `docs-2026-08-18`, curados em `d93c53bef`; o `fmt` do oráculo, em `df6f027d1`).
- `bash scripts/censos-da-arvore-combinada.sh`: 12/12 censos, 114/114.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets`: verde.
- Auditoria (2 lentes independentes, só leitura): correção do 2D e completude/honestidade dos gates —
  zero defeitos. Varredura mecânica de órfãos: 1 `pub` órfão pela poda em 1 447 (`stroke_shapes_relief`,
  apagado). Achado da 1.ª corrida do ship, curado: `MENUBAR_BACKDROP` volta a declarado por AUSÊNCIA.
- Varredura final `grep sculpt|field3d|model3d|…`: só prosa histórica sem link (degraus antigos do
  schema, livros-razão de contadores, medições datadas) e vocabulário 2D (o modo Sculpt do impasto, o
  reshape «sculpt» do Flip, «barro»). O único `#[cfg(feature = "sculpt3d")]` está num doc-comment de
  `project_schema_history.rs`.

## 7. Aberto (decisões, não dívida da linha)

1. **Produto (Enio):** os chips MOVER/RODAR/ESCALAR/ESPAÇO do trilho esquerdo continuam pintados e o
   único leitor era o 3D; estão `DeadOnPurpose` em `the_rail_names_a_consumer_for_every_chip`. Apagar ou
   ligar ao gizmo 2D.
2. `set_area_commands`/`menu_contrib` (D2) ficaram sem produtor — mecanismo genérico, mantido, com o
   gate reescrito sobre uma fixtura.
3. Memória `reference_topic_impasto_physics` mistura impasto 2D e escultura: ficou inteira (o índice
   aponta o 3D para o HISTÓRICO).
4. Palavras do `.typos.toml` órfãs ANTES da poda (11, ex. `factorys`, `rquickjs`): fora do escopo.
5. 5 deps da shell sem uso em `src/` (`ph2d-gpu-cook`, `ph2d-shake`, `ph2d-node-registry{,-init}`,
   `ph2d-audio-encode`) — já assim no merge-base; o machete aceita-as (usadas em testes).

## 8. Smoke (o que NÃO foi smokado à mão: tudo — é do Enio)

Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-poda-3d && cargo run -p ph2d-host-desktop --profile smoke`
(prova do binário quente, depois do último commit de código e do `rm -rf target/*/incremental`:
2.ª corrida de `cargo build -p ph2d-host-desktop --profile smoke` → `Finished … in 0.18s`, zero `Compiling`).

Fotografado antes de ir ao dono (`docs/Components/ferramentas/fotografa_cena.sh`, tela virtual, 1930×1040):

| cena | o que a foto mostra |
|---|---|
| `PH2D_OBJECT_ADD_SMOKE=1` | *Add Object* com **2D · 3** (Image…, Vector Object, Flip Drawing) · **Game · 4** · **Empty · 1** — 8 itens, sem grupo 3D; abas do topo Draw · Vector · Flip · Animate · Nodes (sem *Model*) |
| `PH2D_OBJECT_MODE_SMOKE=1` | imagem *Canvas*, seletor de modo Object / Paint |
| `PH2D_OBJECT_MODE_SMOKE=4` | *Flip Drawing* em Draw, seletor Object / Draw / Edit, tira de Frames |
| `PH2D_OBJECT_MODE_SMOKE=6` | *Vector* nasce em Edit (Object / Edit), duas formas com os pontos |
| `PH2D_COMPOSITE_SMOKE=1` | o Painter compõe as camadas (quatro discos sobre o gradiente e a grelha) |

As cenas `PH2D_OBJECT_MODE_SMOKE=2, 3, 5` eram 3D e saíram (o CLAUDE.md §5.1 UI/UX diz agora `1|4|6`).
NÃO smokado à mão: salvar e reabrir um projecto novo (o clique sintético não chega à tela virtual) —
é o passo 5 do smoke do dono.

## 9. Perfil do laço (`scripts/agent-loop-profile.sh`, 20 sessões)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5
  ✓ respostas por sessao (mediana)                250   alvo: <= 800
  ✗ cargo test : cargo check                711 : 219   alvo: <= 1,0
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%
  ✗ contexto relido por passo (media)         469 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
Nesta linha a maior parte das edições foi por script com `assert` de contagem (cortes multi-linha
exactos em centenas de ficheiros) e por cinco agentes em paralelo, por camada.
