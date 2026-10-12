# HANDOFF de INTEGRAÇÃO — `line/Vector`, prompt 49 (lições do VectorCraft) — 2026-10-11

> Tarefa: [`49_PROMPT_…`](../49_PROMPT_linha_vector_licoes_do_vectorcraft.md). Estudo:
> [`ESTUDO_vectorcraft_comparado_2026-10-11.md`](../Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md).
> Continuações intermédias (o porquê de cada número da W0–W2):
> [`HANDOFF_CONTINUACAO_…_prompt49_W0_W2.md`](HANDOFF_CONTINUACAO_line_Vector_2026-10-11_prompt49_W0_W2.md).
> A linha **não integra nem pusha** (CLAUDE.md §0.7).

## 1. Identidade

- Branch `line/Vector`, worktree `Worktrees/line-Vector`, HEAD `49c21ed5a` (+ o commit deste
  handoff), merge-base com `main` = `8f20c60de` (a linha fez `reset --keep main` no início: os
  commits velhos eram patches EMENDADOS pelo integrador de 07/10). **15 commits**, todos
  `--no-verify`, nenhum push.

| commit | o quê |
|---|---|
| `92cc9281d` | docs: o estudo, o prompt 49, as sondas em `ferramentas/` |
| `4e71193d7` | **cerca**: `the_offset_sweep_matches_the_table` (600 combinações do Offset na estrela vs verdade analítica, atada ao `resvg` por fixture) |
| `6d6dbae87` | **W1**: `linesweeper` 0.4 → 0.5 (pânicos 104 → 0; Round fora 87 → 100/100) |
| `5b830149f` | **W2** (BUGS #38): banda Bevel/Miter por peças de área positiva (`expand_band.rs`) |
| `68455bb47` | **W0**: o VectorCraft como oráculo corrido por script, 592 passos |
| `665d959c2` | **#39 #40**: a lente de duas âncoras; a junção Bevel/Miter pela tangente |
| `a97a42576` | **W3**: kerning atrás da chave `PH2D_TEXT_KERNING=1` |
| `7b0850c61` | W3: as três cláusulas que a mutação achou sem gate passam a sangrar |
| `99c48a2e4` | **W4**: Shape Builder vs mapa planar do VectorCraft — medido, a onda NÃO abre |
| `2cb93b7b4` | W2: controlo do contador de pânicos |
| `b198270b8` | W3: a sonda mede a linha do Motion por quadro |
| `49c21ed5a` | memória: duas palavras que o `typos` do CI reprovava |
| `9b85f02ca` | **BUGS #41** (smoke do dono): os chips de Corner/Side pré-visualizam sem esperar o slider |
| `3b9247948` | **o kerning passa a ser o PADRÃO** (aceite do dono: «kerning é melhor») |
| `d911e5525` | **BUGS #41, 2.ª metade**: o slider de Offset mostra a verdade da seleção |
| `784aa51c5`, `76f9cbf41` | o handoff de continuação (um ficheiro, duas janelas) — e este |

## 2. Foundational / compartilhado tocado (tudo aditivo)

| ficheiro | porquê | aditivo? |
|---|---|---|
| `crates/ph2d-vector-font/Cargo.toml` | `harfrust = "0.12.0"` (`default-features = false, features = ["std"]`) — **já na árvore** pelo `parley 0.11.1`, mesma exigência | sim: nenhum pacote externo novo no `Cargo.lock` |
| `crates/ph2d-vector-font/src/kerning.rs` (novo) | `VariableFont::kerning` + `KernAdjust` (re-exportado no `lib.rs`) | sim |
| `crates/ph2d-vector-font/src/skrifa_bridge.rs` | `font()` passa a `pub(crate)`; campo privado `shaping: OnceLock<ShaperData>` | sim (campo privado; `VariableFont` não é `Clone`) |
| `crates/ph2d-vec-text/src/glyph.rs` | **campo público novo `TextLayout.kerning: bool`** + `kerning_by_default()`; `line_glyphs` é a régua única de `line_advance` e `walk_glyphs` | ⚠️ **quebra literais**: todo `TextLayout { … }` sem `..` tem de ganhar o campo. Nesta árvore: 5 de produção + os testes |
| `crates/ph2d-app-motion/src/motion_text_gen.rs` | +1 campo no literal (`kerning_by_default()`) | sim |
| `shells/desktop/src/{vec_text.rs, vec_text_object.rs, text_path_smoke.rs}` | +1 linha por literal (4 linhas na shell) | sim; `the_shell_only_shrinks` verde |
| `Cargo.lock` | `linesweeper 0.4.0 → 0.5.0` (mesmas deps); aresta `ph2d-vector-font → harfrust` | sim |
| `project-memory/*` | lições da linha + duas palavras corrigidas para o `typos` | — |

⚠️ **Para o integrador:** se outra linha criou um `TextLayout { … }` novo (grep `TextLayout {` no
`main`), ele não compila sem `kerning: …`. A cura é `kerning: crate::vec_glyph::kerning_by_default()`
(shell) ou `ph2d_vec_text::glyph::kerning_by_default()` (crates) — **nunca** `false` à mão em
produção, que deixaria aquele consumidor fora da chave.

## 3. Superfície de colisão (`bash scripts/collision-surface.sh`, colado)

```
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 8f20c60de2   ·   13 commit(s)   ·   51 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        188   (base: 188)
      └ tripla do gate               (188, 13, 22)   (base: (188, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 20   (base: 20)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 107 (base 107) · ph2d-render 108 (base 108) · ph2d-script 108 (base 108)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs   intocado
    crates/ph2d-editor-core/src/tool.rs intocado
▸ ADR — esta linha não cria ADR (último no disco 0181)
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

(Corrido com 13 commits; os seguintes — `49c21ed5a`, memória, e este handoff — não mudam nenhuma linha da tabela.)
⚠️ Referência, não evidência: o integrador lê o `main` no ficheiro (§1.5.3).

- ⚠️ A `line/UIUX` mexeu em `penned_outline` (cortes do `miter_clip`). Esta linha **não** tocou em
  `penned_outline` nem em `miter_clip`, mas o **Offset Bevel/Miter deixou de passar por
  `penned_outline`** (só o Round e o Outline Stroke passam). Se a UIUX mudou a SEMÂNTICA do Miter
  (cortar em vez de cair em bevel), o Offset Miter (peças: bevel além do `miter_limit`, a lei do SVG)
  e o Outline Stroke divergem. Essa decisão é do integrador.
- Ficheiros novos: `ph2d-vec-boolean/src/{expand_band.rs, expand_band_tests.rs, expand_phantom.rs}`,
  `tests/it/{raster.rs, the_offset_sweep_matches_the_table.rs, oracle_vectorcraft.rs,
  oracle_vectorcraft_build.rs}` + 3 fixtures; `ph2d-vector-font/src/kerning.rs`,
  `tests/fixtures/kern_xplacement.ttf`; `ph2d-vec-text/src/glyph_kerning_tests.rs`,
  `tests/fixtures/vectorcraft_kerning.tsv`. `expand.rs` 699 → 610 LOC.

## 4. Contratos congelados

**Nenhum.** `collision-surface.sh`: `node.rs` e `tool.rs` intocados. O vector congelado
(`ph2d-vector-doc`, ADR-0056..0068) não está no diff: `git diff --name-only 8f20c60de..HEAD | grep
ph2d-vector-doc` dá vazio. O `ph2d-vector-font` não é contrato congelado.

## 5. O que fechou — números MEDIDOS

### W0–W2, #39/#40 (janelas anteriores; detalhe na continuação)

| `the_offset_sweep_matches_the_table` (600) | merge-base | W1 | W2 |
|---|---|---|---|
| Round fora | 87/100 | 100/100 | 100/100 |
| Bevel fora | 35/100 | 40/100 | **100/100** |
| Bevel dentro | 94/100 | 94/100 | **100/100** |
| pânicos engolidos | 104 | 0 | 0 |

Oráculo W0: 582/592 passos concordam; as 10 discordâncias estão em `DISAGREE` com o lado certo.

### W3 — kerning (**LIGADO por omissão** desde o aceite do dono, `3b9247948`)

- **Lei:** `VariableFont::kerning` molda a linha duas vezes com o `harfrust` (com e sem `kern`;
  `liga`/`clig`/`calt` desligadas) e devolve a DIFERENÇA. Um glifo por `char`; `None` se a moldagem
  trocar glifos (a linha fica nominal). `line_glyphs` é a régua única do quebrador (`wrapped_lines`),
  do caret, do Fixed (`unwrapped_block_width`) e do desenhador (`walk_glyphs`).
- **Chave:** `kerning_by_default()`, lida uma vez por processo — `true` salvo `PH2D_TEXT_KERNING=0`.
  Os 5 construtores de produção (sessão de texto, objecto de texto, Motion `source.text`, cena 22 e
  a sua sonda) leem-na. ⚠️ **Muda a aparência de todo texto já autorado** — aceite do dono no smoke
  de 11/10, depois do antes/depois (ordem do coordenador cumprida).
- **Oráculo:** `oraculo_vc_kerning.py`. O `vectorcraft-cli convert --outline-text` desenha cada
  prefixo de 12 palavras (`AV To Wa fi Ty LT Yo AVATAR WAVE Kerning minimum Typography`) com a
  MESMA `InterVariable.ttf` (o «A» dele cai ao ponto sobre o nosso) →
  `ph2d-vec-text/tests/fixtures/vectorcraft_kerning.tsv`. **12/12 palavras, todo pen a ≤ 0,5 u**.
  Controlo: sem kerning, a maioria das palavras discorda. ⛔ Só wght 400: a Inter do VC é estática,
  e a 700 ele a substitui pela SemiBold (`status: substitute`), por isso recusada como régua.
- **Gates** (`glyph_kerning_tests.rs`, `kerning.rs`): pens = oráculo · texto sem pares
  **byte-idêntico** (400 e 650, 3 alinhamentos, com e sem caixa, reto e em caminho) · quebra na
  largura moldada e caret na ponta do último glifo · texto em caminho carrega o kerning · o par segue
  o `wght` · `liga` (Noto Sans, que liga `fi`) · `calt` (a seta `->` da Inter) · XPlacement (fonte
  mínima gerada, `fonte_kern_xplacement.py`, 816 B: nenhuma fonte do repo o usa).
- **Custo** (release, intercalado, o MÍNIMO; sonda `measure_the_kerning_against_advance_only`):

| caso | advance-only | kerning |
|---|---|---|
| re-cook de 216 chars com caixa de 12 em (uma tecla) | 0,333 ms | 0,702 ms (2,1×) — 0,964 ms antes de guardar o `ShaperData` na fonte |
| linha do Motion, 17 chars (publicada a CADA quadro) | 3,4 µs | 10,9 µs (load 12) |

- **Antes/depois:** `target/fotos-w3/kerning_antes_depois.png`, desenhado pelo mesmo cozedor (sonda
  `draw_the_before_and_after_for_the_owner`): em cima sem, embaixo com. Fotos do app, cenas 21 e 22,
  com `PH2D_TEXT_KERNING=0|1` em `target/fotos-w3/cena{21,22}_kern{0,1}.png`. Na 21 a palavra «PATH»
  passa de **403,42 → 377,91** de largura no painel Transform.
- **Premissa derrubada:** o doc dizia «moldado − nominal deixaria um resto de 1/128». Medido: o
  `harfrust` e o `skrifa` leem o MESMO avanço (Inter a 400/650/733, os dois arredondam o HVAR) ⇒ a
  mutação «diferença → moldado − nominal» é EQUIVALENTE nas nossas fontes. A diferença fica porque
  não depende dessa coincidência; o doc foi corrigido.
- Fora: `ph2d-app-vec/src/font_preview.rs` (o nome da família desenhado no menu de fontes) tem o
  seu próprio laço advance-only. É um RÓTULO de UI, não texto autorado, e tem outra lei (0,25 em
  para `char` sem glifo); fica sem kerning de propósito.

### W4 — Shape Builder (medido; **a onda não abre**)

- `oraculo_vc_build.py` corre `shapeBuilder.regions` sobre os 49 pares do W0 e 3 casos do Build: a
  cena do BUGS #28 escalada à caixa do corpus, um círculo com uma linha ABERTA e três círculos em
  fila → `fixtures/vectorcraft_build_regions.tsv`. As nossas faces são as que o cursor acha
  (`face_at` em grelha 160² de frações irracionais), casadas por pertinência e área (1 %); lascas
  abaixo de 0,2 % ficam de fora.
- **51/52 casos com as mesmas faces.** O 52.º (`IsoCube`) vai para a `DISAGREE`, e o PH2D é que está
  certo: as arestas internas do cubo são contornos ABERTOS, que o renderer e o hit-test não
  preenchem; o SVG do corpus fecha-os no `fill`, e o VC vê ali um buraco de 0,4625.
- A linha aberta **não corta em nenhum dos dois**: o VC devolve uma face só, com `lines: []`, apesar
  de o catálogo dele o anunciar. BUGS #12 é de FLUXO de UI (o clique caía no «selecionar tudo»), já
  gateado na shell. O motor dele é o `face_at`, que este gate cobre.
- Prompt 49: «abrir só se o nosso perder». O nosso não perde ⇒ fica registado, sem onda.

### BUGS #41 — smoke do dono (cena 17), corrigido depois do 1.º fecho

- *«os botões de Side e Corner só tiveram efeito após mover offset»*: o chip chamava `retune`, que
  só ajusta offsets já armados, com o painel a MOSTRAR 25 % sobre uma forma sem offset. Cura:
  `offset_live::choose_knobs` (retuna os vivos, arma as nuas com o `d` do slider; zero ⇒ nada).
  O gate antigo `retuning_a_shape_without_a_live_offset_arms_nothing` EXIGIA o defeito e foi
  substituído (3 gates, red-first, mutação sangra). O roteiro auto-dirigido dos níveis 18/19 nunca
  clicava um chip (parava de rolar no slider; `chip ROUND fora do hit-index` sem falhar) — rola
  agora até o Apply; no caminho real `off=0 → 1, d=0,300, live=152` logo após o clique.
- 2.º smoke: *«o slider mostra 25 mas o efeito não aparece até mudar uma opção»* — o slider guardava
  a fração de nascença sobre uma forma nua. Cura (`d911e5525`): `offset_live::shown_frac` (seleção
  sem offset vivo ⇒ slider em ZERO) e `chip_frac` (o chip sobre o zero arma os 25 % de nascença e o
  slider mostra-os). Nível 19 real: `off=0 slider=0.00` → clique → `off=1 d=0,300 slider=0.25`;
  ablado, `off=0 slider=0.25` (o sintoma). `nextest-impacted` do diff: 2 881/2 881.
- *«negativo fica sempre pontuda»*: **não é defeito** — a ponta convexa encolhe em ponta; o Corner
  muda os vales (d = −0,3: Miter 0,232 · Round 0,260 · Bevel 0,307). Imagem:
  `target/fotos-w3/offset_negativo.png`.
- Shell tocada a mais: `render_loop/fase_live_offset_and_width.rs` (+4 linhas) e
  `build_smoke_expand.rs` (o roteiro).

## 6. Provas de mutação (fecho)

| cláusula | resultado |
|---|---|
| W2 `loop_region` por peças (`round = true`) | SANGRA `the_offset_sweep_matches_the_table` |
| W2 `crosses_out` → `false` | SANGRA `the_side_filter_never_changes_the_answer` |
| W2 `push_positive` sem inverter | SANGRA idem + `every_piece_is_positive_whatever_the_loop_direction` |
| W2 `bends` colinear → `false` | SOBREVIVE (só custo, esperado) |
| W2 `bends` do lado útil → `false` | SANGRA `the_side_filter…`; sobrevive no sweep |
| W2 `into_paths` sem o filtro `SLIVER_REL` | SANGRA `the_ink_is_one_connected_piece_with_no_loops` |
| W2 hook de pânicos não incrementa | era NÃO PROVADO ⇒ controlo novo `the_panic_counter_counts_a_swallowed_panic`; agora SANGRA |
| W3 `line_advance` nominal | SANGRA `the_wrapper_breaks_at_the_kerned_width` |
| W3 pen sem `kern` | SANGRA o oráculo + caminho + quebra |
| W3 sem `offset` | era SOBREVIVE ⇒ fonte mínima; agora SANGRA `a_placement_moves_the_glyph_and_not_the_pen` |
| W3 `plain` sem `kern` | SANGRA (as duas crates) |
| W3 `liga` / `calt` ligadas | eram vácuas ⇒ Noto Sans / `->`; agora SANGRAM |
| W3 diferença → moldado − nominal | EQUIVALENTE (ver §5 W3) |
| W3 `kerning_by_default = true` | sobrevive na crate (esperado: a chave é dos construtores) |
| W4 região sem subtrair as de fora | SANGRA `the_build_faces_agree_with_the_vectorcraft_oracle` |

## 7. Gate batched de fecho (1× sobre o diff acumulado, 11/10)

| passo | resultado |
|---|---|
| `BASE=8f20c60de bash scripts/nextest-impacted.sh` (load 3,8) | **16 952/16 952**; os `rdeps` entraram: `ph2d-app-vec` 863 · `ph2d-app-motion` 1 461 · `ph2d-host-desktop` 1 992 · `ph2d-vec-boolean` 160 · `ph2d-vector-font` 25 · `ph2d-vec-text` 35 |
| clippy `--all-targets --all-features -D warnings` (6 crates tocadas/consumidoras) | limpo |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `file_loc_caps` (shell), `arch_safe_clamp_only`, `the_shell_only_shrinks`, `no_downcast…` | 10/10 verdes; os maiores tocados: `vec_text_object.rs` 534, `vec_text.rs` 471 (tecto 600) |
| `censos-da-arvore-combinada.sh` | 117/117 |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | OK · OK |
| `cargo machete` (4 crates tocadas) | nada sem uso |
| `cargo fmt --all --check` | limpo |
| `typos` sobre o diff | ✗ 8 em `reference_topic_measurement_discipline.md` (escritos por `665d959c2`) ⇒ reescritos em `49c21ed5a`; re-corrido: limpo |

- **Re-gate depois do #41 e do kerning por omissão** (`9b85f02ca`, `3b9247948`): `nextest-impacted`
  **16 953/16 954** (load 33–58); o vermelho é `ph2d-tool-painter …mask_tests::the_mask_stroke_cost_
  does_not_follow_the_canvas` (relógio: `0,41 ms @1024² vs 1,53 ms @2048²`), crate sem diff nesta
  linha, verde sozinho — família de [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md). Nenhum teste
  de texto mudou com o kerning ligado. clippy `-D warnings`, `CARGO_BUILD_WARNINGS=deny cargo check
  --workspace --all-targets`, `fmt` e `typos`: limpos.
- O `nextest-impacted` avisa que `docs/Vector Module/ferramentas/vectorcraft_2026-10-11/sonda_offset/`
  (uma sonda Rust da W0) não é membro do workspace. É de propósito (regra J4: sonda em
  `ferramentas/`, saída em `target/`).
- **O que só o `ship.sh` ou o CI pegam:** nenhum `#[cfg(target_os …)]` escrito ou movido; nenhuma
  dependência nova além do `harfrust`, que já estava no lock.

## 8. Auditoria (2 lentes)

```
LENTE: costura (wiring)
CLAIM: todo texto autorado — editor, objecto reaberto, HUD, Motion — mede e desenha com a MESMA régua, e a chave chega a todos.
TRAÇO: TextLayout só nasce em 5 sítios de produção (o campo novo obriga o compilador a mostrá-los):
  shells/desktop/src/vec_text.rs:layout_of · vec_text_object.rs:layout_of_params (→ hud_label_live.rs:101, vec_text_reopen.rs:43)
  · text_path_smoke.rs:layout · ph2d-app-motion/src/motion_text_gen.rs:build_stream — todos `kerning_by_default()`.
  Medidores: wrapped_lines / caret_x_offset / unwrapped_block_width → line_advance → line_glyphs; desenho: walk_glyphs → line_glyphs.
ASSERÇÃO-VERMELHA: the_wrapper_breaks_at_the_kerned_width (mutação 1 sangra); a_placement_moves_the_glyph_and_not_the_pen.
NÃO-CHECADO-PELA-COMPILAÇÃO: que a chave LIGADA no app dá o desenho do gate — visto nas fotos 21/22 (403,42 → 377,91).
LOC LIDAS: ~900 (glyph.rs, kerning.rs, os 5 construtores, motion_text_gen.rs publish).

LENTE: custo
CLAIM: a chave ligada não pesa numa tecla nem num quadro do Motion.
TRAÇO: motion_text_gen.rs:153 publish → build_stream → walk_glyphs a CADA quadro; sessão de texto → regen_into → text_to_compound_path a cada tecla.
ASSERÇÃO-VERMELHA: sonda measure_the_kerning_against_advance_only (números na §5); não é gate de tecto — 10,9 µs contra 16 ms não pede um.
NÃO-CHECADO: textos muito longos com caixa (quebrador O(palavras × linha)): não medido acima de 216 chars.
LOC LIDAS: ~200.
```

## 9. Smoke para o Enio

Binário já compilado na worktree (2.ª corrida: `Finished smoke profile [optimized] target(s) in
0.37s` depois das duas metades do #41 e do kerning por omissão, zero `Compiling`).

**A. Os chips de Corner/Side pré-visualizam já no clique, e a estrela não some** (re-smoke do #41).
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && PH2D_BUILD_SMOKE=17 ./target/smoke/ph2d-host-desktop`
(1) clique na estrela azul de borda amarela; (2) no painel da direita, desça até **Expand** e, SEM
tocar no slider, clique **Round** em **Corner**; (3) antes do clique o slider mostra 0 (a
estrela não tem offset); no clique ela cresce na hora, com as pontas arredondadas, e o slider
passa a 25; clique **Bevel** e **Miter**, e cada um muda o desenho na hora;
depois arraste **Offset** de ponta a ponta — ela não pisca nem some no meio; (4) errado = o clique
não muda nada até mexer no slider, ou a forma some no meio do arrasto. No lado NEGATIVO as pontas
ficam afiadas em todos os cantos (é a geometria); o que muda são os vales entre elas
(`target/fotos-w3/offset_negativo.png`).

**B. Kerning — já é o padrão** (aceite do dono). Comparar: `… && PH2D_TEXT_KERNING=0
./target/smoke/ph2d-host-desktop` volta ao espaçamento antigo. Imagem:
`target/fotos-w3/kerning_antes_depois.png`.

## 10. Aberto (nada in-scope)

- Textos longos com caixa e kerning: o quebrador molda cada linha candidata (O(palavras × linha));
  medido até 216 chars (0,7 ms). A sonda vira tecto se o dono pedir textos longos.
- Linha aberta a cortar faces no Shape Builder (como o Illustrator faz): nem o VC nem o PH2D o
  fazem. É feature nova, decisão de produto, e não defeito.

## 11. Integração (11/10, por ordem do dono)

- `main` estava no merge-base (`8f20c60de`): rebase vazio, `--ff-only` para `361c9afc8`.
- `foundational-integrate.sh` (pela `ph2d-run.sh`, load ~16–25 com a suíte do Painter a correr ao
  lado): sync dos registos sem diff · staleness verde · `CARGO_BUILD_WARNINGS=deny cargo check
  --workspace --all-targets` limpo · `nextest-impacted` **16 956/16 956** (o flake do
  `mask_tests` passou). O `--ff-only` do script recusou só por `project-memory/MEMORY.md` sujo no
  primário (sessões vivas); o diff local foi guardado, o ficheiro reposto, o ff feito à mão sobre o
  MESMO tip testado e o diff reaplicado (`git apply --3way`, hunks disjuntos).
- Rascunhos não versionados do primário que a linha substituiu (`49_PROMPT…`, `ESTUDO…`,
  `Estudos/vectorcraft_2026-10-11/`, versões anteriores às correções de caminho da linha) saíram da
  árvore; cópia em `target/integracao-vector-2026-10-11-backup/`.
- **Medido para as linhas ainda abertas** (`git merge-tree` de cada uma contra o novo `main`):
  Luz2D, MiroClone e UIUX fundem sem conflito; PainterWatercolor conflita só no `CLAUDE.md` (§5,
  entradas vizinhas) e motion-value só em `MEMORY.md` + `reference_topic_measurement_discipline.md`
  (contagens de família: reconte, não escolha). Nenhuma das cinco cria `TextLayout { … }`.
- ⚠️ **Para o integrador da line/UIUX:** o `24c7f2ea7` dela põe o bico CORTADO (miter-clip SVG 2)
  em `penned_outline`; o Offset Bevel/Miter desta linha já não passa por ali e continua a chanfrar
  além do `miter_limit` (lei do Offset Path do Illustrator). Fusão textual limpa em `expand.rs`; a
  divergência Offset × Outline Stroke no Miter é de lei, não de build — confirme
  `the_offset_sweep_matches_the_table` na árvore combinada.
