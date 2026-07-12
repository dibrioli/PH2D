# HANDOFF de integração — `line/Painter` (2026-07-12)

> **Para o agente integrador.** A linha está **FECHADA**. Não integrei, não pushei, não rodei ship —
> isso é ordem explícita do Enio (CLAUDE.md §0.7 · DIRETRIZ §1.5.9).
> Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` · branch `line/Painter`
> **33 commits** sobre `main`, 74 arquivos, +9855/−2550.

---

## 0. ADENDO (mesmo dia, dono novo) — Fase 4: o CORPO (`482c0696` · `2c3492ab`)

Após o veredito do Enio (*"não sei se melhorou ou piorou; ficou mais difícil de ajustar"*), a linha
trocou de dono, fez **pesquisa nova** (5 varreduras de fontes primárias —
[`docs/Painter/17_impasto_deposito_pesquisa2.md`](Painter/17_impasto_deposito_pesquisa2.md)) e
**redesenhou o modelo de depósito** (decisões + divergências medidas: plano 16 §10/§10.1):

- **Curva de corpo** no kernel (`body_profile`, `height.rs`): platô + parede DENTRO da tinta
  pigmentada (o bevel *inner* do PS); o véu translúcido não carrega relevo. O domo morreu.
- **Inclinação física** (`DEPTH_UNIT_PX = 16`): morrem `SLOPE_GAIN` e o mute quadrático por
  cobertura; o glint ganhou curva própria (`gloss_body` — specular só no filme).
- **O knob `Amount` (canvas) MORREU** — era o gêmeo acoplado do Depth. Superfície final: brush
  `Enable/Depth/Smoothing/Source/Draw To` · canvas `Show/Angle/Elevation/Shine`. O id
  `PAINTER_IMPASTO_LIGHT_AMOUNT` saiu de `PAINTER_IMPASTO_FIELDS` (6→5) — **desmonte, não append**:
  se outra linha referenciar esse id, é colisão real (nenhuma deveria).
- **Teto de vidro** no commit de traços (`H_CEIL = 2.0`, "pressed against glass" do Painter).
- **Dívida herdada fechada:** o gate de LOC do workspace já estava **vermelho no HEAD desta linha**
  (`paint.rs` 712/700) — `union_region` movido para o irmão `region.rs` (697/700).
- Gates: **584** `ph2d-tool-painter` (2 novas com RED por mutação: body-with-an-edge + glass
  ceiling; halo/does-not-shade/corrugação re-derivadas — corduroy ViewPlane 1.0→0.70, atenuado) +
  **239** brush; clippy `--all-targets` 0; perf **1.79 ms/move** (alvo ≤4).
- **Fase 4.1 (pós-smoke, mesmo dia):** o Enio aprovou o corpo mas apontou a perda do arredondado —
  a curva esmagava todo falloff. Nasceu o dial **`Body`** (`impasto_body`, 0..1, default 1.0 = look
  da Fase 4; **0 = o relevo obedece a silhueta por inteiro**, o domo redondo sob a luz nova; meio =
  família mesa). É o `Technique` do PS como contínuo. `PAINTER_IMPASTO_BODY` entrou em
  `PAINTER_IMPASTO_FIELDS` (5→6, append). Gate com RED por mutação:
  `impasto_body_zero_obeys_the_falloff`. Perf 1.87 ms/move. Plano §10.2.
- **Fase 4.2 (2ª ordem do smoke): TODO parâmetro do relevo é VIVO.** O traço deixou de guardar a
  ALTURA e passou a guardar os **ingredientes** (`stroke_paint` f32 + `stroke_grain` u8); o relevo é
  sempre `derive_height(spec, paint, grain)` — uma função pura, a MESMA no depósito e na edição. Logo
  **Depth · Body · Depth Source · Smoothing** editam o último traço ao vivo, sem caso especial.
  Envelope passa a ser na TINTA (grandeza que nenhum setting muda) e o perfil roda na tinta
  (`w × dinâmica`), alinhando geometria e luz. `Draw To` segue não-vivo **de propósito** (é roteamento
  de canal, não propriedade da tinta — a metade da cor é irreversível). Gate
  `impasto_every_body_knob_edits_the_last_stroke_live` (RED em 2 mutações) + o unit de pureza no
  kernel. `height.rs` 761→511 (testes p/ `height_tests.rs` — **arquivo NOVO**); `paint.rs` 700/700.
  Perf 1.66 ms/move. Plano §10.3.
- **Fase 4.3 — `Shine` estava MORTO** (3ª rodada do smoke). Causa **minha, da Fase 4**: o relevo só tem
  declive na faixa `W_TAIL..W_SOLID` (a parede) e eu tinha gateado o glint **acima** de `W_SOLID` — só
  no platô, que é plano. Medido: 94% dos pixels com declive ficavam fora do gate; Shine 0→1 movia
  **1 nível**. Fix duplo: o glint usa a **mesma curva de corpo do difuso** (1 → **160 níveis**) e o
  highlight vira **screen** (`lit + add·(1−lit)`) em vez de aditivo puro — o aditivo plano ressuscitava
  o halo pelo canal saturado (gate do halo vermelha em 19%). Gate novo com **3 claims, 3 vermelhos por
  mutação**: `impasto_shine_glints_on_the_wall_without_bleaching_the_rim`. Perf 1.67 ms/move. Plano §10.4.
- **Defaults do artista + BUG do relevo por documento (Fase 4.4).** Defaults (ordem do Enio): brush
  `Depth 1` · `Body 0` · `Smoothing 1`; canvas `Angle 230°` · `Elevation 30°` · `Shine 0.7` — nos 4
  sites espelhados; o smoke passou a NÃO re-armar Depth/Smoothing (mostra o pincel default de verdade).
  **Bug meu, da Fase 1, achado no caminho:** `StashedDoc` não levava `heights`/`covers`, e os
  `RtLayerId` colidem entre documentos ⇒ trocar de sprite **perdia** a escultura, e voltar a um sprite
  cacheado fazia o relevo do anterior **iluminar** o novo (espécie do #13.c). Fix + gate
  `relief_travels_with_its_document_and_is_never_lent_to_another` (duas barreiras, defesa em
  profundidade — provado por mutação). **Gate do halo reescrita** para a propriedade indefensável
  (tinta sem corpo não recebe luz **nenhuma**, em qualquer setting) — a métrica de croma antiga não
  distinguia luz honesta de halo sob `Body 0`; e o screen ganhou **gate algébrico exato**
  (`screen(R)−screen(G) = (R−G)(1−add)`), sem limiar de imagem. Perf 1.94 ms/move. Plano §10.5.
- **PERSISTÊNCIA do documento pintado (Fase 5) — a pintura sobrevive ao `Ctrl+S`.** Antes: sprite
  pintado = `Individual{texture_id}` (id de runtime da GPU) ⇒ pintar+salvar+reabrir devolvia o quadro
  **em branco**. Agora: componente **NOVA `ph2d_ecs::PaintedDoc(u32)`** (módulo irmão `painted_doc.rs`,
  registrada no `ComponentRegistry` 26→**27**) dá identidade estável; `PaintedDocument`
  (`ph2d-tool-painter/src/tool/persist.rs`) leva **camadas + pixels + relevo + cobertura** para o
  arquivo (`LayerImage` ganhou serde); o load re-instala, compõe **pelo caminho normal do preview**
  (sem segundo bake) e re-materializa a textura individual. `PROJECT_SCHEMA` 2→**3**. Shell:
  `project_painter.rs` (NOVO) + 3 linhas em `project.rs` + `next_painted_doc` no `AppGfx`. Gate
  `a_painted_document_survives_the_disk_with_its_relief` (postcard REAL, entidade com bits diferentes).
  **⚠️ Foundational tocado:** `ph2d-ecs` (componente nova, append; `registry.rs` +1 linha e o gate de
  contagem 26→27) — anotado para o integrador. Plano §10.6.
- **Smoke do Enio: PENDENTE** (validar: os defaults novos · **Shine** acende a crista · girar
  Depth/Body/Source/Smoothing DEPOIS do traço e ver o relevo mudar ao vivo · pintar relevo, trocar de
  sprite e voltar — a escultura tem de estar lá · **pintar, Ctrl+S, fechar, reabrir, Ctrl+O — a pintura
  E o relevo têm de voltar, ainda editáveis**). Comando do §5 inalterado; card Lighting 4 linhas,
  card Body 5.

Os §§ 1–8 abaixo descrevem as entregas anteriores da linha e continuam válidos; números de gates
ficam superseded pelos desta seção.

---

## 1. O que a linha entregou, em uma frase cada

1. **Varredura do Painter** (9 achados, 8 corrigidos) — nasceu de um SIGSEGV e virou a descoberta de que o
   bug era uma **espécie**, não um caso: todo *guard* de reúso perguntava *"já inicializei?"* em vez de
   *"este dado ainda pertence às entradas que o produziram?"*.
2. **Impasto (#16)** — traço com aspecto 3D: canal de altura + passe de luz, integrado a **tudo** que o
   Enio listou (Shape, Grain, ramps, Stroke, shapes dinâmicas, Tiling, Mirror, Jitter, Per-Layer Color) e
   **escondido** onde não se aplica. **Watercolor não foi tocado.**

---

## 2. Estado dos gates (rodados no fechamento, worktree limpa)

| Gate | Resultado |
|---|---|
| `ph2d-painter-brush` | **239** passed |
| `ph2d-tool-painter` | **575** passed (+17 ignored: GPU/perf) |
| `ph2d-panel-painter-layers` | **40** + **20** seam |
| `ph2d-editor-core` (33 suítes de arquitetura) | verde — incl. `no_magic_numeric`, LOC caps |
| `cargo clippy --all-targets` (3 crates + shell) | **0 warnings** |
| `cargo fmt --all --check` | limpo |
| `cargo build --release -p ph2d-host-desktop` | ok |

**NÃO rodei `ship.sh`.** A memória [`project_integrator_ship_catches_latents_budget_iterations`] avisa: o
gate por-linha **não roda** fmt-workspace / clippy-all / machete / deny / typos — **orce 2-4 iterações** de
ship vermelha. Os prováveis: `machete` (a shell ganhou uso novo de `ph2d-tool-painter`), `typos` (os
commits e docs têm pt-BR).

---

## 3. Superfície tocada (para prever conflito de merge)

**Foundational tocado** (Modo L permite; anotado conforme
[`feedback_foundational_editable_design_for_isolation`]):

- **`ph2d-editor-core`** — **só append**: `ids/chrome/painter_impasto.rs` (arquivo NOVO) + 2 linhas em
  `ids/chrome/mod.rs`. Nenhum id existente mudou. Colisão só se outra linha inserir no mesmo ponto do
  `mod.rs` — resíduo textual trivial.
- **`ph2d-painter-brush`** — `height.rs` (NOVO) · `spec.rs` (+campos `impasto_*`, **append**; testes inline
  extraídos p/ `spec_tests.rs` pelo teto de LOC) · `dab.rs` (silhueta extraída p/ `silhouette_at`, chamada
  pelo kernel de cor — **comportamento byte-idêntico**, 239 testes provam) · `texture.rs`
  (`rotate_by_degrees` virou `pub`).
- **`shells/desktop`** — `impasto_smoke.rs` (NOVO) + 1 guard em `painter_gpu_preview.rs` + 3 linhas em
  `painter_bridge.rs` + 1 campo em `app_state.rs`.

**Contratos congelados (§6): NENHUM tocado.** `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`
intactos — o Impasto anda pelo `PanelEvent` existente.

---

## 4. ⚠️ O único ponto que exige olho no merge

`crates/ph2d-panel-painter-layers/src/paint_watercolor.rs` teve `card_frame`/`card_row` **extraídos** para
`card.rs` (**movimento puro**: −92/+1, só remoções + um import). Se outra linha editou esse arquivo, o
Mergiraf pode se confundir. **A óptica da aquarela não foi tocada** — nenhuma linha de física/render.
Confira com: `git diff main -- crates/ph2d-panel-painter-layers/src/paint_watercolor.rs`.

---

## 5. O que o Enio ainda não viu (smoke pendente)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
Canvas branco 1024² **já selecionado**, pincel **já armado** (Depth 0.7, source Grain). **Pegue o Painter e
arraste.** Card **Lighting** move a luz ao vivo; **Depth negativo CAVA**.

Também pendentes de smoke (da varredura): os 8 fixes — em especial **Rake de Shape com Per-Layer Color +
traço freehand**, que era o SIGSEGV original.

---

## 6. Aberto e DELIBERADO (não é esquecimento)

| Item | Por quê |
|---|---|
| **Watercolor OFF→ON no meio do traço** apaga tinta | **Não consegui construir um RED** — o dab plano não chega a pintar no harness. Escrevi o fix e **revertí**: sem vermelho refutável não se mexe, menos ainda na aquarela que o Enio mandou não ferir. BUGS #13. |
| Gates de paridade banda-vs-serial são **dependentes de máquina** | Num runner de 1 core comparam serial com serial. |
| `Plow` (Smear arrasta relevo) · Composite Depth por camada · luz na GPU · relevo do PAPEL · persistência do `h` | Fora do 1º corte, **nomeados** no plano §6 + §9.10. O relevo do papel **acopla impasto↔aquarela ⇒ exige ordem nova do Enio**. |

---

## 7. Docs vivos

- [`docs/Painter/16_impasto_plano_implementacao.md`](Painter/16_impasto_plano_implementacao.md) — plano +
  **§9: onde a implementação divergiu do plano, e por quê** (5 decisões, cada uma com gate).
- [`docs/Painter/15_impasto_pesquisa_e_design.md`](Painter/15_impasto_pesquisa_e_design.md) — a pesquisa.
- [`docs/Painter/BUGS_painter.md`](Painter/BUGS_painter.md) — **#12** (o SIGSEGV) e **#13** (a varredura).
- [`docs/Painter/13_fila_integracao_watercolor_secoes.md`](Painter/13_fila_integracao_watercolor_secoes.md)
  — **#19** (semântica da aquarela fixada).
- Memória nova: [`feedback_stale_comment_and_dead_code_lie`](../project-memory/feedback_stale_comment_and_dead_code_lie.md).

---

## 8. A lição que vale além desta linha

**Cinco vezes** um teste meu ficou **verde pelo motivo errado**, e sempre do mesmo jeito: a fixture não
continha o fenômeno. Cache testado em dois tools (frio nas duas vezes → re-assa sempre). Opacidade com a
camada de baixo cheia (o over-composite satura → a de cima não podia mudar nada). Smear dentro de região
uniforme (girar a elipse não muda nada). Disco duro chamado de "crista" (platô de parede vertical não tem
flanco). Papel branco usado como "tinta plana". **Em todos, foi DESLIGAR o fix que expôs o teste falso** —
e num deles o próprio assert de sanidade me pegou antes.

É por isso que cada gate desta linha tem **vermelho verificado por mutação**, não só verde.
