# HANDOFF de integração — `line/Vector` → `main` (2026-07-12)

> DIRETRIZ §1.5.9. Escrito **pela linha**, para o **agente integrador**. A linha está FECHADA:
> não integrei, não pushei, não fiz ship. Aguardo ordem explícita do Enio.

## 0. Cartão de identificação

| | |
|---|---|
| **Branch** | `line/Vector` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| **Base (merge-base com main)** | `3805f650` |
| **Head** | `833c6594` |
| **Commits** | 27 |
| **Diff** | 121 arquivos, +24 679 / −1 607 |
| **Smoke do Enio** | **OK** (3 rodadas: catálogo · conectores · rótulos) |

**Estado do gate, rodado no HEAD, agora:**

```
cargo nextest run --workspace --no-fail-fast → 5835 passed, 0 failed, 77 skipped
cargo clippy --workspace --all-targets       → 0 warnings, 0 errors
rustup run 1.95 cargo fmt --all -- --check   → sem diff
typos                                        → 0 erros
```

**Aviso honesto (memória `project_integrator_ship_catches_latents_budget_iterations`):** o gate
acima **não é o `ship.sh`**. Ele não roda `machete`, `deny`, `audit` nem o perfil `ci-test`.
Orce 2–4 iterações de ship — é o esperado, não sinal de problema.

---

## 1. O que entrou (em uma frase cada)

1. **Catálogo de formas data-driven** — 47 formas paramétricas VIVAS (editáveis por parâmetros
   depois de desenhadas): círculo unificado (elipse/arco/pizza/rosquinha/segmento), setas em
   bloco, fluxograma ANSI/ISO 5807, balões, símbolos, sólidos isométricos.
2. **Conectores de diagrama, completos** — roteador A\* sobre grafo de visibilidade ortogonal
   (Wybrow), desvio de obstáculo, rotas reta/ortogonal/curva, waypoints manuais, alças de ponta,
   pontas de seta, painel.
3. **Rótulos ancorados** — texto que pertence à forma ou ao conector e os segue.
4. **Pontas de traço** (arrowheads), **raio por-canto + squircle**, e o **espaço de autoria**
   que matou uma classe inteira de bugs de espelhamento.

Documentação viva: [`docs/Vector Module/BUGS_vector.md`](Vector%20Module/BUGS_vector.md) (7 bugs
com sintoma / causa real / gate) e
[`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`](Vector%20Module/20_pesquisa_ferramentas_de_artista.md).

---

## 2. **FOUNDATIONAL TOCADO** — leia isto antes de integrar

Esta é a seção que decide se o merge dói. Toquei foundational **de propósito** (Modo L permite,
ADR-0107), e projetei para isolamento onde deu. Onde **não** deu, está listado abaixo com o
símbolo exato.

### 2.1 Crate NOVA (zero risco de colisão)

- **`crates/ph2d-vec-connect/`** — o roteador. **Puro**: sem ECS, sem kurbo, sem documento.
  Entra uma descrição de rota, sai uma polilinha. Ninguém mais no repo a referencia.

### 2.2 `ph2d-ecs` — **3 pontos de colisão de mesmo-símbolo**

| Arquivo | O que fiz | Risco |
|---|---|---|
| `src/vec_connector.rs` | **NOVO** (módulo irmão) | Nenhum |
| `src/vec_label.rs` | **NOVO** (módulo irmão) | Nenhum |
| `src/lib.rs` | +2 `mod` +2 `pub use` | **Baixo** — linhas apendadas |
| `src/scene/registry.rs` | **`assert_eq!(reg.len(), 28)`** (era 26) | **ALTO** ⚠ |
| `src/vec_shape.rs` | campos novos em `VecShape` (formas do catálogo) | **Médio** |

> ⚠ **A CONTAGEM DO REGISTRY TEM ASSERÇÕES GÊMEAS.** Registrei 2 componentes
> (`VecConnector`, `VecLabel`), então a contagem foi **26 → 28**. E há **três** lugares que a
> afirmam:
>
> - `crates/ph2d-ecs/src/scene/registry.rs` → `28`
> - `crates/ph2d-render/src/registry.rs` → `29` (conta +1 próprio)
> - `crates/ph2d-script/src/registry.rs` → `29` (idem)
>
> **Se outra linha registrou componentes, os três números somam.** Ajuste os TRÊS ou o workspace
> fica vermelho. (Isto já me pegou uma vez nesta linha: subi só o do ECS e li um "verde" que
> vinha do meu próprio `echo`.)
>
> E o motivo de o registro existir: **um componente que não passa pelo `ComponentRegistry` é
> silenciosamente DESCARTADO pelo snapshot** — undo e save o perdem, sem erro nenhum.

### 2.3 `ph2d-editor-core` — **IDs e a11y (colisão provável com qualquer linha de UI)**

| Arquivo | O que fiz |
|---|---|
| `src/ids/chrome/vector.rs` | **+~40 `NodeId`** (seções, campos do conector, marcadores, catálogo) |
| `tests/node_id_collisions.rs` | a lista de ids cresceu junto |
| `tests/hr12_widgets_a11y.rs` | +9 linhas |

**Como resolver o conflito:** os ids são `hash_node_id("vector.…")` — **namespaced**, então uma
colisão de *hash* é improvável. O conflito será **textual** (duas linhas apendando no mesmo
bloco). Mergiraf resolve; depois **rode `cargo test -p ph2d-editor-core`** — o gate
`node_id_collisions` é justamente o que pega um id duplicado.

### 2.4 `ph2d-i18n` — +52 linhas (chaves `panel.vector.*`)

Apêndice puro no `match`. Conflito textual trivial.

### 2.5 `ph2d-vec-edit` — **`shape.rs` foi reescrito em grande parte** (461 linhas mexidas)

O `ShapeTool` passou a ser data-driven (o catálogo manda) em vez de um `match` gigante. Se outra
linha tocou `shape.rs`, **este é o pior ponto do merge**. `shape_constraint.rs` é módulo NOVO
(Shift/Alt).

`pen_support.rs`: `PenStyle` ganhou `marker_start`/`marker_end`/`marker_scale`/`marker_round`.

### 2.6 `.typos.toml` — +48 linhas

Palavras pt-BR de comentário. Apêndice puro; conflito textual trivial.

---

## 3. **SCHEMA / SAVE** — o que muda no arquivo do usuário

| Struct | Mudança | Compatibilidade |
|---|---|---|
| `StrokeSpec` | +`marker_start`, `marker_end`, `marker_scale`, `marker_round` | **Apendados por ÚLTIMO** + `#[serde(default)]`. Postcard é posicional ⇒ **saves antigos seguem legíveis**. `marker_scale` usa `default = "unit_scale"` (o default de `f64` é ZERO, e cabeça de tamanho zero é cabeça invisível) |
| `VecConnector` | componente NOVO | Não existia; nada a quebrar |
| `VecLabel` | componente NOVO | Idem |
| `ShapeKind` | **discriminante 14 virou um BURACO** (a `ArrowCurved` foi removida) | Os seguintes **NÃO recuaram**, de propósito: recuá-los quebraria todo save que já tem um losango dentro |

**Não subi `SCHEMA_VERSION`/`DOC_VERSION`** — e é deliberado: toda mudança é apêndice
compatível. Se o integrador discordar, é uma conversa antes do ship, não depois.

---

## 4. Contratos congelados (CLAUDE.md §6)

**Nenhum foi tocado.** Explicitamente:

- `Tool = 12` / `RasterEditTool = 5` / `CanvasPaintTool = 1` / `PanelEvent = 4` — **intactos**.
- `NodeOp` / `OpResolver` / `NodeManifest` — **não encostei**.
- O gate `architecture_vector_contract_surface` (que escaneia `ph2d-vector-doc` + `-traits`) —
  **verde**; não toquei nessas crates.

O motor novo (`ph2d-vec-*`) tem contrato **próprio, ainda NÃO congelado** — e cresceu bastante
nesta linha. Re-congelá-lo continua sendo follow-up (era antes, e é agora).

---

## 5. Ordem de frame — **carga, não detalhe**

O `render_loop/mod.rs` ganhou passes novos, e **a ordem entre eles é load-bearing**. Se o merge
os reordenar, três gates ficam vermelhos (e é isso que eles existem para fazer):

```
vec_entities::sync
  → connector_live::upkeep        (pendura o VecConnector)
  → label_live::upkeep_pending    (pendura o VecLabel)      ← DEPOIS do recook, ver abaixo
  → vec_transform::settle_origins (PULA conectores e Live Shapes)
  → vec_transform::build
  → connector_live::recook        (a rota — e quem monta as PAREDES)
  → label_live::upkeep            (a pose do rótulo)
  → render
```

Duas armadilhas que já me custaram sangue, documentadas nos gates:

- **`settle_origins` só pula o que ENXERGA.** Com o `upkeep` depois dele, um conector recém-criado
  era assentado como path comum e renderizava **deslocado em dobro**.
  Gate: `a_fresh_connector_never_enters_the_xform_map`.
- **Um rótulo nasce vazio** ⇒ sem geometria ⇒ sem path ⇒ sem entidade ⇒ **sem `VecLabel`**. No
  frame em que a 1ª letra o materializa, o roteador pergunta "isto é um rótulo?" e ouve **não**.
  Resultado: a linha desviava do próprio rótulo, o rótulo se re-centrava, e a coisa **oscilava
  para sempre**. Gate: `a_label_sitting_on_the_route_never_pushes_the_line_aside` — que roda a
  sequência EXATA do frame, afirma em TODO frame e exige **período zero**.

---

## 6. Roteiro de integração sugerido

1. `git rebase main` (ou o `scripts/foundational-integrate.sh`, que é o protocolo do ADR-0107).
2. **Conflitos esperados, em ordem de dor:**
   - `ph2d-vec-edit/src/shape.rs` (reescrita grande) — o pior.
   - `ph2d-editor-core/src/ids/chrome/vector.rs` + os dois testes de id.
   - `ph2d-ecs/src/scene/registry.rs` — **some as contagens**, não escolha um lado.
   - `ph2d-i18n`, `.typos.toml`, `Cargo.lock` — apêndices, triviais.
   - `shells/desktop/src/{main,app_state,input_dispatch,render_loop/mod}.rs` — apêndices
     (declarações de `mod`, campos de `App`, arms de dispatch), mas **confira a ordem do §5**.
3. **Varra marcadores de conflito em CADA commit** (memória
   `feedback_sweep_conflict_markers_every_commit`): `git grep -n '^<<<<<<< '`. Uma árvore limpa
   no fim **não prova** que o histórico compila.
4. Rode, nesta ordem: `cargo check --workspace` → `cargo nextest run --workspace --no-fail-fast`
   → `cargo clippy --workspace --all-targets` → `ship.sh`.
5. **Cuidado com o pipe** (memória `feedback_pipe_masks_script_exit_code`): `./ship.sh | grep …`
   faz o `$?` virar o do `grep`. Verifique o ESTADO, não o código de saída de um pipe.

---

## 7. O que fica ABERTO (não é dívida escondida; é escopo)

**Do Vector:**
- **Export/import SVG** — um editor vetorial que não exporta SVG não fecha o ciclo.
- **Faces isométricas sombreáveis** — exige o `cook` emitir **1 entidade por face** (hoje é
  1 forma = 1 path = 1 entidade). É **mudança de arquitetura**, e vale decidir **antes** que mais
  formas dependam do modelo 1-para-1.
- **`vec_save` não serializa pose/nome/parentesco** — gap pré-existente, herdado, não meu.
- **Portas fixas** (âncoras azuis) do conector: o motor as suporta (`Anchor::Port`), falta a UI.
- **Re-congelar o contrato do motor novo** (`ph2d-vec-*`) — follow-up antigo, e a superfície
  cresceu.

**Do Painter (herdado, intocado):** `docs/HANDOFF_per_layer_color_perf_artifacts.md`.

**A pesquisa de próximos passos** está em
[`20_pesquisa_ferramentas_de_artista.md`](Vector%20Module/20_pesquisa_ferramentas_de_artista.md).
A tese, em uma linha: **os ~50 Live Path Effects do Inkscape são um sistema de nós — e nós já
temos um.** A próxima grande alavanca é a espinha, não mais uma ferramenta.

---

## 8. Riscos que eu declaro (o que eu NÃO provei)

Sou obrigado a ser honesto sobre os limites do que os testes cobrem:

1. **Nenhum teste roda com GPU/janela.** Tudo é headless. O smoke do Enio (3 rodadas) é a única
   prova de que a coisa aparece na tela — e ele passou nas três.
2. **A oscilação linha↔rótulo tinha um gate VERDE enquanto o bug estava vivo.** O harness
   reproduzia o *mecanismo* e não o *contexto* (faltava um passe do frame). Reescrevi o gate para
   rodar a sequência real. **Se outro bug de laço aparecer, desconfie do harness antes do
   relato.**
3. **O campo `Curve` do painel nasceu MORTO** — pintado, clicável, e sem registro no `populate.rs`.
   A suíte inteira ficou verde. Matei a classe (tabela única de campos + gate que a varre), mas
   é a prova de que **um widget pintado não é um widget vivo**, e o gate de seam é o único que
   sabe a diferença.
4. **Não medi performance com um diagrama grande.** O roteador custa 1–6 µs no caso realista e
   48 µs com 51 obstáculos (medido). Mas **não** rodei 500 conectores numa cena.
