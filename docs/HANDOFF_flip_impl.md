# HANDOFF de integração — linha `line/FLIP`, Wave W0 (dados)

> Entregável §1.5.9 (DIRETRIZ). A linha está **fechada e PARADA** — não integrei
> nem pushei. Este doc é o que o Enio passa ao agente integrador.

## 1. Identidade

- **Branch:** `line/FLIP`
- **HEAD:** `66c76e64`
- **Base (merge-base com `main`):** `1c7c9a22`
- **Commits na linha:** 5 (todos `--no-verify`, fast mode)

```
66c76e64 fix(flip): W0 gate — count ecs 25 em render/script + PROJECT_SCHEMA v2
dbeb0991 style(flip): derive Default (clippy derivable_impls)
0d01ff1b feat(flip): W0 T0.9-T0.11 — ponte ECS + undo + save (shell)
1fb2e593 feat(flip): W0 T0.8 — componente ECS FlipObjectRef (ph2d-ecs)
a491f400 feat(flip): W0 T0.1-T0.7,T0.12 — modelo ph2d-flip (clean-room GP 5.2)
```

## 2. Foundational / compartilhado tocado (+ por quê)

Tudo **aditivo** (nada reescrito). Foundational-não-contrato = editável pela linha
sob o gate testado (ADR-0107).

| Arquivo | Mudança | Por quê |
|---|---|---|
| `crates/ph2d-flip/**` (crate NOVA, 10 arquivos) | modelo de documento puro | drop-crate isolada; entra no workspace pelo glob `crates/*` (sem editar `members`) |
| `crates/ph2d-ecs/src/flip_object_ref.rs` (NOVO) | componente `FlipObjectRef(u64)` | ponte objeto↔entidade (espelha `vec_path_ref.rs`) |
| `crates/ph2d-ecs/src/lib.rs` | `pub mod flip_object_ref;` + `pub use FlipObjectRef` | export do componente |
| `crates/ph2d-ecs/src/scene/registry.rs` | registro + `reg.len()` **24→25** | save/undo precisa do componente registrado |
| `crates/ph2d-render/src/registry.rs` · `crates/ph2d-script/src/registry.rs` | count ecs+próprio **25→26** | somam `register_ecs_components`; SÓ a batched gate pega (não o `check -p`) |
| `shells/desktop/src/flip_entities.rs` (NOVO) | ponte `sync`/`rebuild_map` | espelha `vec_entities.rs` |
| `shells/desktop/src/render_loop/mod.rs` | destructure `flip` + 1 chamada `flip_entities::sync` (ao lado do vetor, todo frame) | reconciliação doc↔entidades; **no-op no W0** (sem tool que crie objetos) |
| `shells/desktop/src/app_state.rs` | `AppGfx.flip: FlipDoc` + `App.flip_entities: FlipEntityMap` | estado vivo |
| `shells/desktop/src/init.rs` · `main.rs` | init dos 2 campos + `mod flip_entities;` | boot |
| `shells/desktop/src/undo.rs` | `ProjectState` ganha 3º campo `flip: FlipDoc`; capture/restore/apply cobrem o Flip | undo global |
| `shells/desktop/src/project.rs` | `PROJECT_SCHEMA` **1→2** (o `flip` mudou o formato do save) | HR-14 |
| `shells/desktop/Cargo.toml` · `Cargo.lock` | dep `ph2d-flip` | — |

**Nenhum ponto de extensão central foi editado de forma não-append-only.** O
componente é um arquivo irmão novo; o registro é append no fim de
`register_ecs_components`.

## 3. Símbolos novos que podem COLIDIR com outra linha (grep-áveis)

- **Componente ECS** (nome canônico, string estável): `"ph2d::ecs::FlipObjectRef"`.
  → `ComponentRegistry` len: **25** (era 24). Downstream: render/script tests = **26**.
- **Crate nova:** `ph2d-flip` (nome de pacote).
- **Const nova:** `ph2d_flip::FLIP_SCHEMA_VERSION = 1`.
- **Const alterada:** `PROJECT_SCHEMA = 2` (era 1) em `shells/desktop/src/project.rs`.
- **Campos novos:** `AppGfx.flip`, `App.flip_entities`, `ProjectState.flip` (3º campo).
- **Módulo shell novo:** `mod flip_entities;` em `main.rs`.
- **Sem** `IconId`/`NodeId`/`ColorToken`/token novo (isso é W1/W2).

> Colisão mais provável com outra linha: se **outra linha também bumpou
> `register_ecs_components`** (novo componente), o `reg.len()` esperado soma —
> reconcilie o número (não é 25, é 24 + Σ dos componentes novos das linhas) nos 3
> sites: `ph2d-ecs`, `ph2d-render`, `ph2d-script`. Mergiraf funde as duas linhas
> de `register(...)`; o número do `assert_eq!` é o resíduo semântico a acertar.

## 4. Contratos congelados encostados (§4)

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/
`CanvasPaintTool`/`PanelEvent`, `Vector`(`ph2d-vector-doc/-traits`) — **intactos**.
O `ComponentRegistry` do ECS **não é** contrato congelado (é ponto de extensão
append-only). Não exige ADR.

## 5. O que SÓ o `ship.sh` pega (a gate de integração não roda)

- **fmt pré-fork:** rodei `rustup run 1.95 cargo fmt` nas crates tocadas → limpo no pin.
- **machete (deps não-usadas):** `ph2d-flip` usa todas as 4 deps (ph2d-core,
  ph2d-painter-effects, serde, postcard); a dep `ph2d-flip` no shell é usada. Sem
  dep órfã — mas o ship confirma.
- **deny/audit (RUSTSEC):** zero crate externa nova (só path-deps + serde/postcard
  já no workspace). Sem superfície nova de advisory.
- **typos:** comentários em pt-BR (mesma convenção de `vec_entities.rs`, que passa
  CI). Baixo risco; ship confirma.

## 6. Ordem / dependências + o que smoke-testar

- **Ordem dos commits:** linear, sem interdependência frágil. `a491f400` (crate) →
  `1fb2e593` (ecs) → `0d01ff1b` (shell) → fixes. Integrar como um bloco.
- **Smoke (W0 é headless — não há UI ainda):** o gate É os testes. Nada visual
  para o Enio clicar nesta wave (a tool/painel/render vêm em W1/W2). Se quiser
  confirmar que o app **ainda sobe**: `cd Worktrees/line-FLIP && cargo run -p
  ph2d-host-desktop` — deve abrir normal (o Flip é no-op: nenhuma tool cria objetos).
- **NÃO smokado (por não existir ainda):** desenhar, renderizar traço, criar objeto
  Flip pela UI. É W1/W2.

## Gate W0 — resultado (rodado nesta linha, 1× sobre o diff acumulado)

- `cargo test -p ph2d-flip` → **28 verdes** (tabela GP `{0:d0,5:d1,10:end,12:d2}`,
  refcount+remap, ops de frame, round-trip serde, amostragem por playhead).
- `cargo test -p ph2d-ecs` (registry + full) → verde; render/script registry → verde.
- Shell: `flip_entities` (3) + `undo` (7, inclui o flip novo) + `project` (1) → verde.
- `bash scripts/nextest-impacted.sh` → **957 passed, 0 failed** (40 GPU-skipped).
- `cargo clippy --all-targets -D warnings` em ph2d-flip / ph2d-ecs / ph2d-host-desktop → limpo.
- `cargo fmt --check` (pin 1.95) → limpo. LOC: maior arquivo 455 (cap 700). HR-5:
  zero transcendental. Sem hex / f32-UI / tofu em string literal.

## Auditoria (DIRETIVA §3 — 2 lentes, ASSERÇÃO-VERMELHA real, não "compila OK")

**LENTE: correção (port clean-room das ops de frame).**
CLAIM: `drawing_at`/`add_frame`/`remove_frame` reproduzem a semântica de hold +
end-sentinel do GP 5.2.
TRAÇO: `layer.rs:92 drawing_at` = `range(..=frame).next_back().and_then(|f| f.drawing)`
← lido contra `grease_pencil.cc:1617` (`upper_bound`+recua); `add_frame` (`layer.rs:151`)
← `grease_pencil.cc:1535` (overwrite-end, remove-leading-ends, sentinela em key+dur);
`remove_frame` (`layer.rs:194`) ← `grease_pencil.cc:1565` (replace-with-end quando o
anterior é fixo).
ASSERÇÃO-VERMELHA: `layer::tests::drawing_at_follows_hold_and_end_sentinel` dirige a
tabela canônica do GP e quebraria se o hold/sentinela regredisse;
`remove_frame_with_fixed_prev_becomes_end` prova o branch replace-with-end.
NÃO-CHECADO-PELA-COMPILAÇÃO: a IGUALDADE numérica com o GP (a compilação não sabe
que d1 aparece 5..9) — coberta pelos testes tabelados.
LOC LIDAS: `grease_pencil.cc` 1505-1610 + 3207-3530 (fonte) + os 413 de `layer.rs`.

**LENTE: wiring (undo/save — o risco "compila mas está morto").**
CLAIM: capturar→restaurar de fato round-trips o `FlipDoc` E reconstrói a ponte
objeto↔entidade (senão o `sync` seguinte duplicaria objetos).
TRAÇO: `undo.rs:capture` grava `flip.clone()` → `restore` despawna Transform,
respawna do snapshot (que carrega `FlipObjectRef` **porque o registrei**), chama
`flip_entities::rebuild_map` → `apply_project` atribui `gfx.flip` + `self.flip_entities`.
ASSERÇÃO-VERMELHA: `undo::tests::flip_survives_capture_restore_and_rebuilds_bridge` —
muda o flip, restaura, e afirma (a) o doc voltou ao capturado, (b) `fmap` tem o
objeto apontando uma entidade VIVA, (c) capturar 2× é idêntico (sem passo espúrio).
Sem o registro do componente, (b) falharia (o snapshot descartaria `FlipObjectRef`).
NÃO-CHECADO-PELA-COMPILAÇÃO: que o componente está REGISTRADO (compila sem ele; só o
teste de restore-com-entidade-viva pega) e que o diff não regista passo fantasma.
LOC LIDAS: `undo.rs` inteiro (475) + `vec_entities.rs` (519, o espelho) + `registry.rs`.

## Aberto (fora do W0, por design)

- W1 (render GPU), W2 (tool+painel), W3+ — o `flip_entities::sync` já está wirado no
  render loop e vira ativo assim que a tool criar objetos.
- Persistir `flip` cross-sessão já funciona (entra no `ProjectState`); a UI real de
  Save/Open continua stub (herança do estado atual da persistência).
- **Docs de planejamento** (`docs/Flip/`, `docs/architecture/decisions/0113-*.md`,
  `project-memory/project_flip_module_grease_pencil_2d.md`) estão **untracked na
  árvore primária** — NÃO os commitei nesta linha (senão o `merge --ff-only` da
  integração quebra com "untracked working tree files would be overwritten"). O Enio
  deve commitá-los ao `main` por fora, antes ou depois da integração.
