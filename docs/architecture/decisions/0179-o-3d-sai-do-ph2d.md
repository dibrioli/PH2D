# ADR-0179 — O 3D sai do PH2D

- **Status:** Aceite (2026-10-05).
- **Data:** 2026-10-05.
- **Decisor:** Enio (dono). Ordem: foco no 2D — *«acabar esse app antes que a morte chegue»*.
- **Supersede:** toda a linha do **3D** —
  [ADR-0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md) (escultura = malha que doa sombreamento),
  [ADR-0156](0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md) (`rayon` na `ph2d-sdf`),
  [ADR-0159](0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md) (`rayon` na `ph2d-sculpt3d`),
  [ADR-0160](0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md) e
  [ADR-0162](0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) (quad remesh),
  [ADR-0161](0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) (modelagem = árvore de campo implícito),
  [ADR-0167](0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md) (extração quad),
  [ADR-0170](0170-o-corte-e-uma-booleana-de-malha-e-o-motor-permissivo-ENTRA-em-vez-de-ficar-oraculo.md) (corte = booleana de malha) e
  [ADR-0176](0176-o-render-do-modelador-e-uma-malha-de-jogo.md) (modo Render por malha). **MANTÉM como história** (os ADR não se apagam).
- **NÃO revoga:** [ADR-0177](0177-as-camadas-do-painter-juntam-se-em-tons-de-ecra.md) — a lei (camadas do Painter em tons de ecrã) é 2D e vive; só a parte «peça 3D» do texto passa a histórica.
- **Conjuga com:** [ADR-0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) e [ADR-0099](0099-remove-painting-brush-engine-preserve-layers-effects.md) (as outras podas) e [ADR-0114](0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md) (Flip, «sem viewport 3D»).

## 1. Contexto

O dono decidiu concentrar o produto no 2D. O 3D (modelador SDF, escultura, Quad Retopology, tecido,
tinta fina, Painter na peça, forma doada/assada) era a maior fatia de código e de docs, e a que mais
custava em build, gates e contexto. A ordem é PODA: o código sai, não fica desligado.

## 2. O que sai

**Famílias:** 3D Modeling (modelador SDF editável + modo Render por malha) · 3D Sculpt (escultura, Quad
Retopology, tecido, tinta fina, Painter na peça, forma doada/assada, «catavento» `Mesh3D`) · Render3d.

**43 crates apagadas**, por grupo:
- Apps/painéis: `ph2d-app-field3d` · `ph2d-app-sculpt3d` · `ph2d-panel-model3d` · `ph2d-panel-sculpt3d` · `ph2d-sculpt3d` · `ph2d-viewport3d`
- Campo implícito: `ph2d-sdf` · `ph2d-field` · `ph2d-field-ecs` · `ph2d-field-eval` · `ph2d-field-gpu` · `ph2d-field-mesh` · `ph2d-field-profile` · `ph2d-field-render`
- Malha: `ph2d-mesh` · `ph2d-mesh-bool` · `ph2d-mesh-colors` · `ph2d-mesh-forward` · `ph2d-mesh-render` · `ph2d-uv-atlas` · `ph2d-untangle` · `ph2d-trim` · `ph2d-boundary`
- Quad remesh: `ph2d-crossfield` · `ph2d-gridmap` · `ph2d-quadchain` · `ph2d-quadextract` · `ph2d-quadfill` · `ph2d-quadflow` · `ph2d-remesh-iso`
- Sombreamento / forma doada: `ph2d-form-donation` · `ph2d-form-pbr` · `ph2d-material` · `ph2d-triplanar` · `ph2d-sky` · `ph2d-style` · `ph2d-view-transform`
- Outras: `ph2d-cloth` · `ph2d-contacto` · `ph2d-pose` · `ph2d-quantize` · `ph2d-rake` · `ph2d-trace`

**Dentro de crates 2D e da shell:** `ObjectKind::Model3D` / `ObjectKind::Sculpt3D`; os componentes
registados `FieldObject`, `FieldNode`, `FieldPose`, `FieldMods`, `FieldMaterial`, `FieldVerb`,
`FieldProfileSource`, `FieldLight`, `FieldTexture`, `BakedForm`, `Sculpt3dPieceRef`, `Mesh3D`; os blobs do
documento da escultura e do campo; o menu *Add ▸ Model*; o pill MODEL; os modos Sculpt, Paint-na-peça e
Edit-do-Model; os painéis Model3D e Sculpt3D; as fases e ids 3D.

**Docs:** `docs/3D`, `docs/3DModeling`, `docs/Render3d`, `docs/Render`; as entradas «3D / Sculpt» e
«3D Modeling» do `CLAUDE.md` §5.1.

## 3. O que FICA (não é 3D) e porquê

- `ph2d-light` — rig de luz do IMPASTO do Painter 2D; perde só o que servia à forma doada/assada.
- `ph2d-bloom` — `motion_fx` (2D).
- `ph2d-imageio-exr` e os outros leitores de imagem — import 2D.
- `ph2d-node-field-*` e `ph2d-node-value-instance-field` — campos 2D do Motion.
- `ph2d-navmesh` — navegação 2D (ADR-0178 vale).
- ADR-0177 — lei 2D; ver acima.

## 4. Consequências

- **Projetos `.ph2dproj` salvos antes NÃO abrem:** `PROJECT_SCHEMA` 182 → 183, **sem degrau de migração**
  (política da casa: o degrau só existe quando o dono pede). Nenhum dado 2D é migrado.
- Os gates e censos que varriam as crates 3D saem com elas ou perdem a entrada 3D (nunca afrouxados; os que ficariam a medir uma lista vazia foram reescritos com casos 2D); ver o [handoff da poda](../../Retirados/handoffs/HANDOFF_INTEGRACAO_line_poda-3d_2026-10-05.md).
- Quem quiser o 3D de volta lê a história, não reconstrói às cegas.

## 5. Onde está a história

- Último commit do `main` com 3D: `b1a6f9b07` (`b1a6f9b07fea12e6adcebf2bd98d644217022ff2`).
- Backup completo (história inteira, verificado): `~/Documentos/Backups/PH2D/ph2d-main-2026-10-05.bundle`.
- Qualquer doc ou fonte apagado: `git show b1a6f9b07:docs/3D/README.md`, `git show b1a6f9b07:crates/ph2d-sdf/src/lib.rs`, etc.
- O arquivo `docs/archive/` guarda verbatim o que já citava o 3D.

## 6. Medidas

Medido na `line/poda-3d` (2026-10-05), `b1a6f9b07` → fecho da linha:

| | antes | depois |
|---|---|---|
| crates (`crates/`) · membros do workspace | 390 · 403 | 347 · 360 |
| linhas `.rs` em crates+shells | 2 702 458 | 2 151 875 (−20,4 %) |
| shell inteira | 196 867 | 185 076 |
| testes do nextest `--workspace` | 28 143 (ship de 04/10) | 24 186 |
| `Cargo.lock` | — | −75 pacotes, zero versões novas |
| `cargo check --workspace --all-targets` a frio, sem sccache | 62 s · 837 unidades | 60 s · 781 unidades |

⚠️ O relógio do check quase não mexe: ele corre em paralelo e é limitado pelo caminho crítico das
crates de base, não pela soma. O que a poda corta é o **volume** (linhas, testes, disco, leitura).

A luz do impasto — o único caminho 2D cuja aritmética a poda simplificou — foi provada idêntica AO
BIT contra o próprio PH2D de antes da poda (gate `a_luz_do_impasto_com_relevo_e_a_de_antes_da_poda`).
