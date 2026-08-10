# HANDOFF — `line/Painter`: Shape **FLOW** (o padrão segue o traço) — 2026-07-19

> Continuação da mesma linha (lag do Rake + remoção do Random Angle). Um commit novo.
> **Pendente de smoke do Enio.** A linha NÃO integra nem pusha sozinha (§0.7 / §0.2).

## 0. Estado

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| Ahead of `origin/main` | **5 commits** (takeover doc + rake-lag + random-angle + handoff + **flow**) |
| Commit desta entrega | `60fe2e5f feat(painter): Shape FLOW` (28 arquivos) |
| Árvore | limpa · `cargo check --workspace --all-targets` 0 · clippy 0 · LOC cap verde · arch gates verdes |

Suítes das 4 crates tocadas (brush / tool / panel / editor-core): **verdes, 0 falhas.**

## 1. O pedido do Enio (2026-07-19, com screenshot)

> *"os traços [com textura] não conseguem o ângulo em direção ao traço (para que as linhas
> permanecessem paralelas mesmo nas curvas). … O rake que estamos falando é o de Shape.
> Analise o melhor modo de resolver sem perder funcionalidades e combinações entre shape, grain e paper."*

**Diagnóstico (confirmado no código + render-and-look):** o Rake gira cada **carimbo** para a tangente,
mas a textura do Shape está ancorada ao **próprio dab** (mapeamento ViewPlane; Grain/Paper têm os
mapeamentos View/Tiled/Stencil, o Shape não tem — é o slot da silhueta, sempre dab-local). Numa curva os
carimbos vizinhos **reiniciam a fase** do padrão e se entrelaçam → nunca linhas contínuas. Isso é limite do
modelo dab-local, **não** precisão do Rake (o lag já estava mínimo). É **só do Shape**.

**A cura (aprovada — "vamos lá"):** um 3º modo de "seguir" no Shape, o **FLOW**: deitar o padrão no
referencial do **traço** — a coordenada *ao-longo* vira o **arc-length** do dab + a projeção do pixel na
tangente; a *atravessada* fica dab-local. A fase casa de dab a dab ⇒ linhas contínuas, paralelas na curva.

## 2. O que mudou (commit `60fe2e5f`)

### Motor (`ph2d-painter-brush`)
- **`Dab::arc_len`** — distância cumulativa ao longo do traço (px), monotônica, carimbada em cada dab.
  Acumulada em `walk_space` (`base_arc + traveled`, e `arc_len += seg` no fim), nos métodos por-evento
  (Dots/DragDot/Airbrush: `+= dist`), e no `dab_at` de `begin`/`fill_segment`/`tick`. Resetada em `begin` e
  nos `fill_*` dos shape editors (curve/ellipse/polygon). Anchored = 0 (carimbo único).
- **`TextureSettings::flow`** (bool, Shape-only, default `false`). `is_cacheable`/`is_canvas_cacheable`
  passam a exigir `!flow` (Flow é sempre per-pixel). Warm-up (`begin`) inclui `shape.flow`.
- **Sampler:** ramo `s.flow` em `texture::sample` (procedural) — `along = (arc_len + (p−c)·u)/r · sx`,
  `across = (p−c)·v/r · sy` (a soma do arc-length telescopa com o incremento de spacing entre dabs, então a
  fase é contínua); e em `texture/shape::sample_shape` (imagem) — **stream do bico**: tile ao-longo
  (período-2 wrap), clamp atravessado (fora do bico → 0). O `arc_len` viaja por
  **`TexDabBasis::with_arc_len`** (só o caminho per-pixel do Shape o seta; Grain/Paper e os caches deixam 0).

### UI — o checkbox "Rake" virou um seletor de **3 estados** "Follow" (Off / Rake / Flow)
Espelha o dropdown "Texture" (Shape Kind) que já existia ao lado. Removeu `PAINTER_SHAPE_RAKE` +
`toggle_brush_shape_rake`; criou `PAINTER_SHAPE_FOLLOW` + `painter_shape_follow_option_id` +
`PAINTER_SHAPE_FOLLOW_MODES` + `set_brush_shape_follow(u8)` (Off/Rake/Flow **mutuamente exclusivos por UMA
porta** — `shape.rake`/`shape.flow` nunca divergem). Wiring: pending-dd state (`state.rs`/`state_dropdowns.rs`),
popover (`paint_brush.rs`), row (`paint_shape.rs`), `decode_shape_follow_option` + linha no `option_route`,
`populate` (registrado como **Dropdown**, não Button), snapshot `shape_follow:u8`, handler
`SelectOption(PAINTER_SHAPE_FOLLOW)` no `jitter_settings.rs`. **Grain/Paper e todas as combinações
Shape×Grain×Paper intactas** — Flow é só do Shape.

## 3. Gates (red-first / mutação)

| Gate | Crate | Prova |
|---|---|---|
| `arc_len_is_the_cumulative_path_length` | brush | o motor produz arc-length correto (monotônico, = x num traço reto) |
| `flow_gives_adjacent_dabs_a_continuous_phase` | brush | FLOW: dabs vizinhos concordam na fase (`<1e-3`); **control OFF** = mutante embutido (`>0.2`) |
| `with_arc_len_moves_the_flow_phase_and_is_inert_without_flow` | brush | o builder seta E o sampler lê o arc-length; **inerte sem flow** (`==0`) |
| `shape_flow_reaches_the_paint_and_differs_from_rake` | tool | procedural end-to-end (motor→tool→sampler→canvas), difere de Rake e do estático |
| `shape_flow_streams_an_image_tip_along_the_stroke` | tool | o ramo de imagem (`sample_shape` flow) é vivo, não código morto |
| `every_shape_follow_option_id_round_trips` | panel | decode do option-id ↔ valor (o risco de drift silencioso); disjunto do Shape Kind |
| `shape_follow_dropdown_selects_off_rake_flow_mutually_exclusively` | tool | o clique no dropdown seta os flags do MOTOR, exclusivos |

**Lacuna honesta (gateada por review + smoke, não end-to-end):** o literal `.with_arc_len(d.arc_len)` no
`stamp_cache.rs` passa `d.arc_len` (vs uma constante). Isolar esse link **end-to-end** é intratável: no
traço reto o arc-length está entrelaçado com a POSIÇÃO, e re-traçar a mesma linha para variar o arc-length
acumula COBERTURA (opacidade) — os dois confundem qualquer comparação de canvas. O link está coberto por:
o gate do motor (arc-length correto) + o gate builder↔sampler (with_arc_len move a fase) + o end-to-end
(flow difere de rake) + o **smoke do Enio** (a continuidade visual é exatamente o que ele julga). O mutante
`with_arc_len(0.0)` sobrevive aos gates atuais — documentado aqui de propósito.

## 4. Notas de integração (DIRETRIZ §1.5.9)

- **`Dab` ganhou `arc_len: f32`** (não é contrato congelado — ADR-0099 removeu o painter contract). Uma
  linha paralela que construa `Dab {…}` por literal conflita textualmente; resolver adicionando `arc_len`.
- **`TextureSettings` ganhou `flow: bool`** — idem para literais de `TextureSettings`/Default.
- **`TexDabBasis` ganhou `arc_len` privado + `with_arc_len`** — a forma pública do `Dab`/`TexDabBasis` não
  quebra nenhum contrato; nenhum `PROJECT_SCHEMA`/gate de contrato bumpou. `BrushSpec` não é serde ⇒ nenhum
  save antigo carrega o campo.
- **Ids removidos:** `PAINTER_SHAPE_RAKE`. **Novos:** `PAINTER_SHAPE_FOLLOW` (+ option ids). Sem colisão.
- **`option_route` array 16→17.** **Docs históricos** que citam a checkbox "Rake" do Shape são snapshots.

## 5. Pendente de smoke do Enio (veredito CONDICIONAL)

Rodar `cargo run --release -p ph2d-host-desktop` (do worktree) e, no Painter:
1. Seção **Shape** → escolha uma textura direcional (procedural **Stripes**/**Dots**, ou uma imagem de bico).
2. No dropdown **Follow**, escolha **Flow**.
3. Pinte uma **curva** com pincel grande: as linhas do padrão devem **seguir a curva, contínuas e paralelas**
   (não o entrelaçado do screenshot). Compare com **Rake** (gira por carimbo, entrelaça na curva) e **Off**
   (ângulo fixo).
4. Confirme que **Grain** e **Paper** seguem iguais (Flow é só do Shape) e as combinações Shape×Grain funcionam.

## 6. ⛔ NÃO integrei nem pushei (protocolo §0.7 / §0.2)

Fechei, escrevi este handoff, **PAREI**. Integração e ship só por ordem explícita do Enio.
