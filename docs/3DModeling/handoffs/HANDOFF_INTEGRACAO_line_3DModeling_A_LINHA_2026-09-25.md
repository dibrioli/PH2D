# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, a LINHA INTEIRA (2026-09-20 → 2026-09-25)

> **Leitor:** o agente INTEGRADOR (DIRETRIZ §1.5.3). ⛔ **NÃO integrado, NÃO enviado** (§0.7) — a
> linha fecha aqui e espera a ordem do dono.
>
> ⚠️ **Este documento SUPERSEDE, como documento de integração, o
> [`A_CURVA` de 21/09](HANDOFF_INTEGRACAO_line_3DModeling_A_CURVA_2026-09-21.md)** — aquele foi um
> handoff de WAVE a meio da linha, nunca integrado, e os commits dele estão dentro destes.

---

## §1 — Identidade (DIRETRIZ §1.5.9 item 1)

| | |
|---|---|
| branch | `line/3DModeling` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |
| base (merge-base) | **`20a630f1b`** = o `main` de 2026-09-25 — **a linha já está REBASEADA** sobre ele |
| commits | **105** de trabalho + o commit deste handoff |
| ficheiros | `308` (ver §3) |
| integração esperada | `--ff-only` limpo **se o `main` não andar**; se andar, rebase e re-conte os contadores do §3 |

⚠️ O rebase de 25/09 foi limpo: o único commit que o `main` tinha a mais tocava **um** ficheiro de
`project-memory/` que esta linha não toca.

---

## §2 — O que a linha trouxe, por obra (a fila em vigor é a do [`15` §5](../../Render3d/15_as_metas.md))

| obra | o quê | onde se lê o mecanismo | smoke do dono |
|---|---|---|---|
| **1.ª — a lei que acende o sprite** (`F4`) | o sprite assado passa a ser aceso pelo **OpenPBR** (crate-folha nova [`ph2d-form-pbr`](../../../crates/ph2d-form-pbr/)), no dispositivo, com o céu, a oclusão e a indirecta da casa; `PH2D_FORM_PBR=0` volta à lei de tinta | [`15` §7–§8](../../Render3d/15_as_metas.md) | ✅ **aprovado 21/09** |
| **«o que se vê é o que se assa»** | as QUATRO causas do *«o bake não é idêntico ao 3D»*: o modo do visor, a lei do bake, a **matéria** e a **curva sRGB** (`Rgba8Unorm → Rgba8UnormSrgb`); a lei **por objecto** gravada; a **lente**; o **recorte**; a **matéria da peça** | [`16`](../../Render3d/16_o_que_se_ve_e_o_que_se_assa.md) · [`../Render/01`](../../Render/01_o_assado_e_identico_ao_que_se_ve.md) · [`A_CURVA`](HANDOFF_INTEGRACAO_line_3DModeling_A_CURVA_2026-09-21.md) | ✅ a 1.ª metade (21/09) |
| **2.ª — a rota B, o catavento** (`F3`) | o componente **`Mesh3D`** (a malha viva que um sprite mantém, com a pose 3D e o `spin`), a fase `fase_cataventos`, a cena **`=52`**, a secção **Live Mesh** do Inspector | [`17`](../../Render3d/17_a_rota_b_o_catavento.md) | ⏳ reports de 21/09 curados; sem aprovação final |
| **3.ª — a luz sobrevive ao movimento** (absorve a `W9`) | fita inerte · cache do chão · o **gate vermelho herdado resolvido** · o vaso por FÓRMULA · o modo de omissão (matcap) na placa · armazéns CONTADOS · o recorte pela caixa da marcha · matcap em kernel magro · borda compacta · luz em kernel próprio · a luz encostada sem anéis · o chão sem rectângulos · **sondas guardadas na placa** · o assentar que espera o que custa · **a oclusão a passo `2` no quadro de movimento** | [`03` §W9](../../Render3d/03_o_plano.md) (da secção «O PRIMEIRO ACTO» até «A OCLUSÃO A PASSO») | ✅ recorte (24/09) · ⏳ o resto (§6) |
| `W7d` (profundidade de campo) | **FORA, por decisão do dono** (22/09) | [`12` §W7d](../../Render3d/12_o_acabamento.md) | — |

---

## §3 — A superfície de colisão (saída de `collision-surface.sh`, colada — itens 2 e 3)

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 20a630f1b   ·   105 commit(s)   ·   308 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        165   (base: 160)
  ⚠   └ tripla do gate               (165, 13, 22)   (base: (160, 13, 22))
    VEC_SCENE_SCHEMA 22 · FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23 — todos = base
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-ecs                              104   (base: 103)
  ⚠ ph2d-render (espelho)                 105   (base: 104)
  ⚠ ph2d-script (espelho)                 105   (base: 104)
▸ CONTRATO CONGELADO (§6)   node.rs intocado · tool.rs intocado
▸ ADR                       esta linha não cria ADR
▸ Cargo.lock                1 pacote '+name' novo: "ph2d-form-pbr" (crate INTERNA nova)
▸ MARCADORES DE CONFLITO    nenhum
▸ TETOS DE LOC              nenhum arquivo da linha passa do teto
```

⛔ **A tabela é REFERÊNCIA, não evidência** (§1.5.9 item 3): leia o valor do `main` **no
ficheiro** no dia de integrar e aplique os DELTAS abaixo.

### §3.1 — Os contadores, como DELTA

| contador | delta | os degraus (cada um com o parágrafo na escada de [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs)) |
|---|---|---|
| `PROJECT_SCHEMA` | **+5** | `161` a lei que acende um objecto assado VIAJA no ficheiro (`BakedForm::lei`) · `162` o `Mesh3D` (a malha viva do catavento, com a pose 3D) · `163` o `Mesh3D::spin` · `164` `BakedFormDocument::recorte` · `165` `BakedFormDocument::materia_da_forma` |
| registo do `ph2d-ecs` | **+1** | `ph2d::ecs::Mesh3D` (`register_default`) |
| espelhos `ph2d-render` · `ph2d-script` | **+1** cada | o mesmo componente |
| `LIVE_SECTIONS` (`ph2d-editor-core`) | **38 → 39** | `(INSP_LIVE_MESH3D_SECTION, INSP_LIVE_MESH3D_COLOR)` |
| `ComponentEdit` (`action_bus_component.rs`) | **+1 variante**, append-only | `Mesh3d(Mesh3dFieldEdit)` |
| catálogo de componentes (`ph2d-component-desc`) | **+1**, e as `PORTAS` do núcleo **11 → 12** | `"ph2d::ecs::Mesh3D"` |
| `FIELD_DOC_VERSION` | **0** | — |

⚠️ **Sem degrau de migração** (decisão do dono, 26/08, escrita na escada): um ficheiro de uma versão
anterior é **recusado em voz alta**. Se outra linha subir o `PROJECT_SCHEMA` na mesma rodada, os
cinco degraus desta são **renumerados como bloco** a seguir aos dela — conte com
`python3 scripts/schema-recount.py`.

### §3.2 — Ficheiros PARTILHADOS tocados (fora das crates do módulo), e porquê

| onde | o quê | natureza |
|---|---|---|
| `crates/ph2d-ecs/` (`lib.rs`, `mesh3d.rs`, `scene/registry*.rs`) | o componente `Mesh3D` | **aditivo** (módulo novo + 1 linha de registo) |
| `crates/ph2d-editor-core/` (`action_bus_component.rs`, `mesh3d_edits.rs`, `ids/live_sections.rs`, `ids/inspector_camera.rs`, `lib.rs` + 3 gates em `tests/it/`) | a edição do `Mesh3D` pelo Inspector | **aditivo** (append-only) |
| `crates/ph2d-component-desc/src/catalog/core.rs` | a entrada do `Mesh3D` | aditivo |
| `crates/ph2d-i18n/` (4 ficheiros, `+78` linhas) | chaves da secção Live Mesh, do bake e do sculpt | aditivo |
| `crates/ph2d-material/` | o encolhimento do lóbulo muda-se para a crate que o nomeava · `prefilter` · o gate da cor do texel | a lei não muda de valor |
| `crates/ph2d-mesh-render/` (25 ficheiros) | o visor acende com a lei que assa (`Lighting::Pbr`), a lente, o albedo `…Srgb`, o G-buffer para o catavento | ⚠️ **partilhado com a `line/sculpt3d`** — ver §4 |
| `shells/desktop/` (30 ficheiros) | `project_schema*.rs` (a escada) · `project_baked_form.rs` · as fases `fase_cataventos` (**nova**, `#[cfg(feature = "sculpt3d")]`), `fase_sculpt3d_bake`, `fase_inspector_commits*`, `snapshots*`, e 9 gates em `tests/it/` | a shell é composição: as fases **chamam** as crates |
| `Cargo.lock` | a crate interna `ph2d-form-pbr` + arestas | sem pacote EXTERNO novo |

⚠️ **O índice das fases** (`shells/desktop/src/render_loop/mod.rs`) ganhou **uma** fase
(`fase_cataventos`) e perdeu um parágrafo de narrativa arquivada, pelo tecto de LOC dele. ⛔ **A
ORDEM é load-bearing:** o catavento tem de correr **antes** da fase que desenha a irmã assada ([`17`
§7.3](../../Render3d/17_a_rota_b_o_catavento.md)); se outra linha inserir uma fase no mesmo sítio, a
ordem decide-se pelo `frame_text` e não pelo número da linha.

---

## §4 — Onde um merge textual pode colidir — MEDIDO contra as linhas vivas de 25/09

Ficheiros desta linha que outra linha viva também toca (`git diff` de cada `merge-base main..line/X`
cruzado com o desta; `project-memory/` e `Cargo.lock` à parte):

| linha | à frente do `main` | ficheiros em comum | onde está o atrito |
|---|---:|---:|---|
| **`line/components`** | 53 | **37** | ⚠️⚠️ **a mais pesada:** a escada do `PROJECT_SCHEMA` (`project_schema.rs` + `_tests.rs`) · o registo do `ph2d-ecs` e os **dois espelhos** · `ids/live_sections.rs` (array com **tamanho no tipo**) · `action_bus_component.rs` · `ids/inspector_camera.rs` · o `ph2d-panel-inspector` (12 ficheiros: `lib`, `state*`, `sections/mod`, `paint_optional*`, `sync_sections`, `populate_waves`, `event`, `ids`) · e as fases do quadro (`fase_bus_*`, `fase_inspector_commits*`, `fase_hero_commits`, `fase_snapshots_publish`, `snapshots*`, `frame_gfx`, `render_loop/mod.rs`) |
| **`line/sculpt3d`** | **74** (o `main` de hoje é a base dela: trabalho NÃO integrado) | **28** | a `ph2d-mesh-render` (`pipeline.rs`, `pipeline_build.rs`, `shaders/mesh.wgsl`, `lib.rs`, gates) · a `ph2d-app-sculpt3d` (`bake_light`, `birth`, `cena`, `panel`, `scenes*`, `scripts`) · o `ph2d-panel-sculpt3d` · as chaves `app_sculpt3d`/`sculpt3d` do `ph2d-i18n` |
| **`line/UIUX`** | 72 | **21** | o Inspector (`lib`, `state`, `sections/mod`, `paint_optional`) · o `ph2d-panel-sculpt3d` · `app_state_gfx`/`init`/`frame_gfx`/`snapshots_inspector` da shell · os dois `o_*_armado.rs` |
| `line/motion-value` | 86 | 2 | só os `tests/it/main.rs` (listas de `mod`) |
| `line/PainterWatercolor` · `line/Vector` | — | 0 | — |

⇒ **As regras de fusão para os três pesados:**

1. **Escada do `PROJECT_SCHEMA`** — os cinco degraus desta linha (`161`–`165`) são um **bloco**:
   se a `line/components` aterrar primeiro com degraus próprios, estes renumeram-se **a seguir aos
   dela** e a constante fica colada à ponta; conte com `python3 scripts/schema-recount.py`.
2. **Registo + espelhos + `LIVE_SECTIONS` + `ComponentEdit`** — as duas linhas apendam no mesmo
   sítio: **as duas entradas ficam**, os tamanhos (`104/105` e `39`) somam os DELTAS das duas, e a
   ordem no array de secções é indiferente.
3. **`ph2d-mesh-render`** com a `line/sculpt3d` — esta linha mudou a **LUZ** do visor
   (`Lighting::Pbr`, o albedo `…Srgb`, a lente, o G-buffer do catavento); a `line/sculpt3d` mexe na
   **COR por vértice** e nos buffers. São ortogonais em LEI ⇒ a resolução é **juntar as duas**, nunca
   escolher um lado; confirme depois com os gates de shader (`wgsl_gate_tests`: dois dos três validam o
   `mesh.wgsl` pelo `naga`, SEM adaptador; o terceiro é `#[ignore]`) e os `--ignored` da crate.
4. **Fases do quadro** — `fase_cataventos` corre **antes** da fase que desenha a irmã assada ([`17`
   §7.3](../../Render3d/17_a_rota_b_o_catavento.md)); ⛔ a ordem decide-se pelo `frame_text`, nunca pelo
   número da linha no ficheiro.
5. **`project-memory/`** — `MEMORY.md` e sete `reference_topic_*` ganharam linhas (mais 21
   ficheiros novos), e **quatro** linhas vivas tocam os mesmos tópicos. O índice tem um tecto em
   BYTES (`≤ 22 KB`) e um gate conta as famílias: juntar as linhas das duas partes e recontar.

---

## §5 — O portão desta linha, corrido depois do rebase (itens 5 e 5-bis)

| régua | resultado |
|---|---|
| `scripts/nextest-impacted.sh` | **17 811 / 17 811** |
| `scripts/censos-da-arvore-combinada.sh` | **verde**, `127/127`, `12 de 12` censos correram |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `cargo clippy -p ph2d-app-field3d -p ph2d-field-gpu -p ph2d-field-render --all-targets -D warnings` | limpo |
| `cargo machete` | nenhuma dependência órfã (as novas — `bytemuck`, `serde`, `half`, `naga` — são usadas; `half` e `naga` só em `[dev-dependencies]`) |
| `scripts/check-standalone-optional.sh` · `scripts/check-workflow-packages.sh` | verdes |
| `#[cfg(target_os` novo | **nenhum** ⇒ a classe que só o CI de macOS/Windows vê não se aplica |
| gates de GPU (`--ignored`) de `ph2d-field-gpu` | `4/4` |
| gates de GPU de `ph2d-app-field3d` (sem as sondas `diag_*`/`mede_*`/`measure_*`) | `89/89` depois de uma cura (§7.2) |

⚠️ **O CI não corre os gates de GPU** (são `#[ignore]`), logo a linha correu-os à mão. ⚠️ Um binário
de testes de GPU pode sair com **`SIGSEGV` DEPOIS de `test result: ok`** — é a classe conhecida da
saída do adaptador, não um gate: leia a linha do `test result`, não o código de saída.

---

## §6 — O que o dono NÃO smokou (item 6)

- ⏳ **O movimento no Render.** Foto de 25/09 do dono sobre a cena `=28` (o nó de toro), a girar:
  *«ainda não ficou bom, mas continuaremos outro dia»*. A foto mostra **duas coisas**:
  1. a imagem continua **grossa** a mexer — o nó custa `55,5 ms` a `1920×1080` depois da oclusão a
     passo (era `109`), acima dos `16,7` do orçamento, logo o laço do movimento ainda encolhe a tela;
  2. um **contorno PONTILHADO** claro ao longo das silhuetas dos tubos. ⚠️ **Hipótese, NÃO
     medida:** o período de `2` píxeis da tela encolhida coincide com o `ceu_passo = 2` (a oclusão
     reconstruída na silhueta, onde os representantes caem no fundo); o 1.º passo é bissectar com
     `PH2D_FIELD_CEU_PASSO=1` no mesmo enquadramento. A outra candidata é a **borda re-amostrada** na
     tela encolhida, que se bissecta com `PH2D_FIELD_BORDA=0`.
- ⏳ O catavento (`=52`) e a Live Mesh — curados sobre reports de 21/09, sem aprovação final.
- ✅ Aprovados: a lei que acende o sprite (21/09) · o recorte pela caixa da marcha (24/09).

**Smoke para o dono, quando houver ordem:**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=28 cargo run -p ph2d-host-desktop --profile smoke
```

---

## §7 — Coisas que uma leitura rápida do diff entende AO CONTRÁRIO

1. **A oclusão a passo NÃO é uma perda de qualidade do quadro final.** Só o quadro de MOVIMENTO a
   usa; o ASSENTE é byte-idêntico ao de antes, com gate (`o_quadro_assente_ignora_o_passo`).
2. **O `o_quadro_de_movimento_nao_paga_o_ricochete` foi EDITADO e não afrouxado.** Ele compara o
   movimento do dispositivo com a CPU e passou a ler `13` níveis — era a oclusão a passo (divergência
   declarada, com gate e tecto próprios) e não o ricochete. O passo vai a `1` NESSE gate; a mutação
   que põe o ricochete no movimento continua a reprová-lo (`7` níveis).
3. **O gate vermelho herdado (`com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`) está
   VERDE** — mas passou a `load ~90`; um gate de relógio só erra para o vermelho sob carga, logo o
   verde vale, e a medição calma é a do [`03` §W9](../../Render3d/03_o_plano.md).
4. **Os representantes da oclusão correm num despacho PRÓPRIO (`ceu_meia`)** — marchá-los no
   `luz_so` só nos píxeis que representam foi construído e **não poupava nada** (o warp espera pelos
   `48` cones de quem os marcha: nó `107 → 90`). Quem «simplificar» juntando os dois despachos
   devolve o custo inteiro.
5. **O `ceu_passo` mora no ENCHIMENTO do `vec3` do `ball_center`** no uniforme — não é um campo a
   mais no fim; zero bytes e zero ligações novas no grupo `0` (a contagem de armazéns não se move).
6. **As sondas da placa são guardadas entre quadros por IGUALDADE de chave, nunca por hash** — uma
   colisão entregaria o ricochete de outra peça sem erro; e uma peça com ESCULTURA **não** é guardada.
7. **`device_probes_w9_torno.rs` foi PARTIDO** (`767 → 434 + 337`, o irmão novo é
   `device_probes_w9_omissao.rs`) — era um tecto de LOC herdado **já vermelho no HEAD anterior**; a
   lista de testes prova que as três sondas mudaram de endereço e nenhuma evaporou.

---

## §8 — O que fica ABERTO, com endereço

| item | onde |
|---|---|
| a foto de 25/09 (grosso a mexer + contorno pontilhado) | §6 acima |
| o 1.º assente depois de mexer na PEÇA ou na LUZ paga a assadura inteira das sondas (`~170 ms` no nó) | [`03` §W9](../../Render3d/03_o_plano.md), «os travões ao girar» |
| os píxeis que recorrem aos cones no `ceu_sobe` ainda divergem (`~3 ms` no nó); compactá-los numa lista está nomeado e não construído | [`03` §W9](../../Render3d/03_o_plano.md), «a oclusão a passo» |
| o passo `3` da `F1`: **reprojecção temporal** | [`14` §6](../../Render3d/14_a_ordem_de_superar.md) |
| o catavento com **N** instâncias não foi varrido | [`15` §5](../../Render3d/15_as_metas.md) |
| a cobertura do G-buffer da forma é binária (silhueta dura no zoom) | [`17` §11.7-ter](../../Render3d/17_a_rota_b_o_catavento.md) |
| a cauda da 1.ª obra (o destaque satura · oclusão especular · não re-enviar a forma quando só o rig mudou) | [`15` §7](../../Render3d/15_as_metas.md) |

---

## §9 — Reclamado

`rm -rf target/*/incremental` corrido nesta worktree depois do portão (item 7).
