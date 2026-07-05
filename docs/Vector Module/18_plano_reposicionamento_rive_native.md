# 18 — Plano de reposicionamento: Vector nativo, referenciado no Rive (editor-first)

> **Canônico a partir de 2026-07-05.** Supersede a ambição de [17_plano_de_implementacao.md](17_plano_de_implementacao.md)
> (plano de 20 waves — parkeado como histórico). Decisão: [ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md).
> Regra-mãe (DIRETIVA §5): **verde-de-compilação é velocidade; DoD é seam-test verde + smoke do Enio.**

## Norte (uma frase)

Um **editor vetorial direto** — desenhar/editar path + boolean edit-time + rig de bones com deformação por
skinning — **nativo em ECS + kurbo + Vello**, GPU-renderizado e **apto a animação** (timeline vem depois), usando o
**runtime Rive (MIT) como fonte da verdade e dos algoritmos** (portados com atribuição), **sem** herdar seu runtime
OOP nem o formato `.riv`.

## Princípios inegociáveis

1. **Rive = blueprint provado + fonte de algoritmos (MIT, atribuído).** Reimplementar o *modelo*, nativo; nunca
   vendorizar o runtime OOP nem depender de `rive-rs` nem adotar `.riv` (ADR-0108 §2/§6).
2. **Editor-first:** data model mutável + undo-ável; playback lê a mesma cena. Nada de runtime playback-only.
3. **GPU-first esclarecido:** render GPU-residente (Vello) + operações CPU baratas (skinning/boolean) + **dirty-tracking**
   no re-encode como alavanca de escala. Skinning/boolean em compute-shader = futuro medido, não aposta inicial.
4. **Boolean só edit-time** (linesweeper / fallback `path-bool`); o resultado anima. **Sem** boolean em runtime.
5. **Skinning preserva o path Bézier editável** (deforma âncora + handles); mesh triangulada só para imagem/textura.
6. **Norte ECS (ADR-0075):** components + events/resources; tools/nós = drop-crates; UI canônica (tokens/i18n, HR-15).
7. **Kill-criteria antes do build** (DIRETIVA §5); medir a escala **antes** de prometer.

## Aproveitamento vs descarte (das 34 crates atuais)

| Descartar (arquitetura antiga) | Salvar como building-block provado (lift verbatim) |
|---|---|
| `VectorNetwork` AoS/SmallVec, grafo por env-flag, SDF-hybrid, diffusion-fill, LLM/font/CRDT/dormant, 34-crate fan-out | Cubic-fit de Levien · Hobby spline · wrapper boolean sobre linesweeper (`boolean_paths`) · schema versionado depth-bounded (`postcard_schema`) |

## Arquitetura (split inicial — refinar na Fase 0)

- `ph2d-vec-scene` — data model editor-first (ECS): documento, camadas, **paths** (backed `kurbo::BezPath`), **rig**
  (bones = hierarquia `Transform`), **skin weights** por vértice; mutação + undo estrutural.
- `ph2d-vec-skin` — **port do LBS** do `rive-runtime` (`src/bones/`, MIT + `NOTICE`): pura, `(bones, weights, control_points) → BezPath` deformado.
- `ph2d-vec-boolean` — boolean edit-time (linesweeper; fallback `path-bool`), pura.
- `ph2d-vec-render` — construtor da cena Vello + **dirty-tracking** (só re-encoda sub-árvore suja).
- `ph2d-tool-vec-{pen,edit,shape,select,rig}` — drop-crates de ferramenta (desenhar/editar/riggar).
- `ph2d-panel-vec-{inspector,rig}` — UI (inspector de path/fill; painel de rig/bones).

**Fluxo por frame:** cena (ECS) → [rig sujo?] LBS na CPU → `BezPath` deformado → `ph2d-vec-render` re-encoda só o
sujo → Vello (GPU) rasteriza.

## Fases

### Fase 0 — Fundação + spike de medição (fecha a escala ANTES de prometer)
- Scaffold `ph2d-vec-scene` + `ph2d-vec-render`: desenhar um `BezPath` estático da cena via Vello no canvas real
  (prova o seam ponta-a-ponta).
- Aposentar as crates antigas + retirar o gate `architecture_vector_contract_surface` (ADR-0108 §4).
- **Spike de escala:** medir quantos objetos riggados animando sustentam 60 FPS @ resolução-alvo — **naive
  re-encode vs dirty-tracked** — e **fixar N** (kill-criterion). Registrar o número (memória: medir a escala do
  sintoma antes da causa).
- **DoD:** path aparece no canvas pela pipeline nova + N fixado + gate antigo retirado com workspace verde.

### Fase 1 — MVP: editor + skinning + boolean (o coração)
- **Desenhar/editar:** pen (criar), edit (mover âncora/handle), shape (retângulo/elipse/polígono), select. Seam
  completo (7 sites, DIRETIVA §2) com seam-test `ph2d-ui-testkit` por controle.
- **Boolean edit-time:** union/subtract/intersect como **ação destrutiva na UI real** (não env-flag, não node-graph);
  resultado vira path editável.
- **Rig + skinning (a estrela da referência Rive):** criar bones, bind de pesos (≤4/vértice), **arrastar bone →
  path deforma interativo**, path permanece editável. LBS portado (MIT) com **paridade numérica testada** contra a
  referência.
- **Dirty-tracking** ligado (só re-encoda o que mudou).
- **DoD:** seam-tests verdes + **smoke do Enio** (desenha, aplica boolean, riga, arrasta bone e vê deformar). Compile-verde
  não é "pronto" (DIRETIVA §5).

### Fase 2 — Timeline / animação (futuro; arquitetura já apta)
- Modelo de keyframe **referenciado no Lottie** (property tracks + easing) dirigindo os params do rig; playback via
  `sample(t)`. Curve editor / onion-skin conforme prioridade.

### Fase 3 — Interatividade (futuro)
- **State machine** (lógica portada do runtime Rive, MIT) + **constraints** (IK/translation/rotation) para reação a
  input em tempo real.

## Kill-criteria + gates

- **Escala (two-strikes, ADR-0108 §5):** N fixado na Fase 0; falhou 60 FPS após a 2ª arquitetura de dirty-tracking →
  PARA e prova antes da 3ª.
- **Gate por fase:** `nextest-impacted` + clippy `--all-targets` + auditoria ≥2 lentes (DIRETIVA §3) + **seam-test
  comportamental** (`architecture_interactive_crate_has_behavioral_test`) + smoke do Enio.
- **Skinning:** teste de paridade numérica LBS↔Rive é entregável, não "parece certo".
- **Integração (Modo L):** fechamento = `scripts/foundational-integrate.sh` (gate da árvore combinada).

## Referências

- **Rive runtime (MIT, source da verdade):** [rive-runtime](https://github.com/rive-app/rive-runtime)
  (`src/bones/`: `skin.cpp`/`weight.cpp`/`tendon.cpp`/`skinnable.cpp`; `src/shapes/cubic_vertex.cpp`) ·
  [rive-rs](https://github.com/rive-app/rive-rs) · LICENSE MIT ([C++](https://github.com/rive-app/rive-runtime/blob/main/LICENSE)/[Rust](https://github.com/rive-app/rive-rs/blob/main/LICENSE)).
- **Stack (deps permissivas):** kurbo (geometria) · vello/peniko (render GPU) · linesweeper (boolean; fallback
  `path-bool` do Graphite, Apache-2.0).
- **Referência de arquitetura (Apache-2.0):** [Graphite](https://github.com/GraphiteEditor/Graphite) (editor vetorial
  Rust node-based). **Referência de modelo de animação:** Lottie/velato. **Só comportamento (GPL, nunca porte):**
  Inkscape, Blender Grease Pencil.
- **Decisão:** [ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md).
