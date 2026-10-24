# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, biblioteca + FERRAMENTAS do Painter (2026-10-24)

> Leitor: o integrador e a próxima janela da linha. Anterior:
> [PINCÉIS, no main 23/10](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md). Munição (por ordem):
> [BIBLIOTECA](HANDOFF_line_PainterWatercolor_2026-10-23_BIBLIOTECA.md) ·
> [FERRAMENTAS](HANDOFF_line_PainterWatercolor_2026-10-23_FERRAMENTAS.md) ·
> [FERRAMENTAS_2](HANDOFF_line_PainterWatercolor_2026-10-24_FERRAMENTAS_2.md) (§4 e §5). Smoke do dono: o das
> ondas passou (23/10 e 24/10, até à 7.ª volta das orlas); **a integração em si não é aprovação**.

## §1 — Coordenadas

- Linha única nesta rodada: nada a ordenar. Merge-base `972f411a6f`; o `main` andou **169 commits** (UIUX F27, MiroClone
  W24–W25, Macros W17–W19, Luz 2D, CodeEditor) e a linha trazia **35**; rebase limpo para `5af68b7cbe`.
- Sobreposição medida (ficheiros tocados pelos dois lados): **11** — `CLAUDE.md`, `.config/nextest.toml`,
  `docs/DevOps/FLAKES_DE_CARGA.md`, `docs/Components/ferramentas/fotografa_cena.sh`, `ph2d-app-painter/src/lib.rs`,
  `ph2d-editor-core` (`interaction/state/{mod,store_core}.rs`, `screens/hero/paint.rs`), `ph2d-tool-painter`
  (`dab_spec.rs`, `traco_plano.rs`, `watercolor_render/window.rs`). A única sobreposição semântica real: a W18 do Macros
  trocou `brush.radius_px` por `raio_do_pincel()` (o do painel × a escala da macro) — a linha reescrevia as mesmas linhas.
- Schemas todos iguais (`PROJECT_SCHEMA` 193, tripla `(193, 13, 23)`, vec 23, flip 13, timeline 22); registos
  108/109/109; contratos §6 intocados; ADR: nenhum; `Cargo.lock`: nada externo.

## §2 — O que entra no main

- **Biblioteca de pincéis** (23/10): a ordem não muda sozinha, ordem manual por arrasto, Delete com janela de
  confirmação, Export/Import com as imagens de todos os slots, Stars sem degraus, Jitter com o Unit no topo.
- **Ferramentas do Painter** (23–24/10): na coluna do Vector (Máscara e Seleção em dois botões), inspector por ferramenta
  (UMA tabela `ferramenta_linhas.rs`), Dual nas que mexem na tinta, preview só no Pincel e na Borracha, meio e presets
  **por ferramenta** (Borracha com slot e mão próprios, Hard Eraser), Flow à vista, conta-gotas com cursor, seletor de cor
  ancorado/fixável. Orlas do traço (bug #58, 7 voltas: Wet do Digital = o aro da aguada), Export/Bake/Board/Merge levam a
  tinta por assar (bug #53), Aquarela com os números do painel do dono.
- Portões da fábrica passam o tempo de UMA vez (`sonda_gestos::Relogio::DeUmaVez`, ao byte nas 283 cenas).

## §3 — Conflitos e como se resolveram (estágios `:1/:2/:3`, nunca marcadores)

| Ficheiro | Conflito | Resolução |
|---|---|---|
| `.config/nextest.toml` (2×) | as duas linhas acrescentaram nomes aos mesmos dois `filter =` | união; faixa isolada **sem** os dois `macro_estado_inicial` (o main já os pôs na pesada e a tabela dele usa `::`, que o gate não lê como membro) |
| `FLAKES_DE_CARGA.md` | idem, na tabela | a linha combinada do main + a linha de `a_separacao_das_pocas…`; evidência da linha junta à do main |
| `fotografa_cena.sh` | as duas curaram o MESMO falso positivo (`wayland-0` fixo) | fica o corpo do main (compara com as constantes **e** com quem chama) |
| `traco_plano.rs` | o main trocou a fonte do raio numa linha que a linha reescreveu | a da linha (`largura_px`, sem raio) |
| `watercolor_render/window.rs` | idem: `raio_do_aro(spread, …)` × `raio_do_pincel()` | os dois: `raio_do_aro(spread, self.raio_do_pincel())` + a 2.ª ocorrência do main |
| *(sem conflito)* `traco_aguada.rs` | a linha lia `brush.radius_px` | `raio_do_pincel()` — a lei da W18 também vale para o aro |

Prova de que o rebase sem conflito não perdeu nada: **cada um dos 35 commits rebaseados comparado com o original**
(multiconjunto `+/-` por ficheiro): 31 idênticos; os 4 que diferem são exactamente os 4 conflitos acima; varredura de
marcadores (`<<<<<<<`, `=======`, `>>>>>>>`, `|||||||`) em todos os commits: 0. Uma perda minha, apanhada na comparação
e curada (`fixup` para o commit certo): ao resolver pelos estágios usei o `:2` inteiro, que perde os hunks sem conflito do
commit — o comentário de 3 linhas do `nextest.toml`.

## §4 — Vermelhos da árvore combinada (ship.sh sobre o tip) e a cura de cada um

Nenhum era `main` verde a tornar-se vermelho por flake; todos reproduzem sozinhos, salvo o último.

| # | Portão | Causa | Cura |
|---|---|---|---|
| 1 | `the_shell_only_shrinks` | a soma: +178 linhas da linha sobre um main que já estava a 36 do tecto (189 183 > 189 041) | **corte por responsabilidade de 165 linhas**: a sonda `measure_painter_bridge_phases` (só `PainterTool`, `#[ignore]`) vai da shell para `ph2d-app-painter/tests/it` → 189 018. Tecto intacto. **O main fica a 23 linhas do tecto**: a próxima linha que toque a shell corta primeiro |
| 2 | `workspace_src_files_under_loc_cap` | `interaction/state/mod.rs` 703 (o main já estava em 700) | a doc de `picker_target` (7 → 2 linhas): 699 |
| 3 | `the_foundation_modules_form_a_dag` | `widget → interaction` 44 contra 43: o pino do seletor trazia um `use` novo | `paint_pin` já não importa `interaction` (devolve se pintou; o registo no `HitIndex` fica em `paint.rs`, que já importa) |
| 4 | `the_corner_of_a_control_is_never_chosen…` | o cursor do conta-gotas escolhia `Radius::Sm` à mão (gate novo do UIUX) | `paint::frame_radius(hero.theme, Radius::Sm.px())` |
| 5 | `todo_leitor_de_pixels_de_sprite_tem_veredito` | `fase_macros.rs` (Macros W19) é um leitor novo de pixels de sprite | veredito em `LEITORES`: o «aplicar» lê o documento do Painter antes da sprite (`documento_da_imagem`); a gravação do «mundo» lê a sprite ao fechar — ver §5 |
| 6 | `a_altura_de_abertura…` | `painter_layers` 1545 → 1570 = a fileira Flow (25 px). **Medido**: com `le_o_flow()` forçado a falso volta a 1545 | número 1570, com a nota ⬆️ no gate (não é secção nascida aberta) |
| 7 | `architecture_toda_flake_de_carga…` | os dois `macro_estado_inicial` na faixa isolada sem linha própria na tabela | saem da isolada (ficam na pesada, como no main) |
| 8 | `instructional_docs_only_cite_paths…` | o CLAUDE.md citava `project-memory/feedback_full_suite…md`, que está **por rastrear** no checkout principal | o link sai (a regra fica no texto). ⚠️ o ficheiro de memória continua por rastrear |
| 9 | `duas_pinceladas_que_se_cruzam…` (Macros W17) | **bisect**: nasce no commit «a Aquarela nasce com os números do painel do dono». 3 de 5 454 texels da Aquarela a 1–2 níveis do papel (a cauda do aro), tecto 0 | a folga da água (1 %) que o Wet já tinha passa à Aquarela; um retângulo apagado são centenas |
| 10 | `a_cena_4_conduzida_passo_a_passo` (CodeEditor) | espera de 30 s pela lista do `luau-lsp` sob `load 119`; zero linhas de `code-scenes` no diff; sozinho 1,0 s, 12/12 em paralelo; com 120 laços de CPU (`load 13 → 38`) 6–16 s, 4/4 | **promovido a flake de carga** (tabela + faixa isolada, as duas escritas) |
| 11 | `a_cena_3_conduzida_passo_a_passo` (CodeEditor) | a irmã da cena 4, na 2.ª corrida do ship (a cena 4 passou nela): a mesma espera de 30 s pelo `luau-lsp` sob a suíte; sozinha 0,5 s | flake de carga, as duas escritas (tabela + faixa isolada) |
| 12 | TIMEOUT `censo_dos_controlos::a_linha_esmaecida_e_a_que_nao_age` e `seam_estado_escondido::cada_interruptor_…` | faixa pesada de 6 × 60 s: sozinhos 51,1 · 70,3 s; sob a suíte 327,0 · 357,4 s (passaram a 91–99 % do prazo) e na corrida seguinte 360 s (mortos) = 5,1–6,4× | a faixa pesada passa a **10 × 60 s** (8,5× o mais lento sozinho). É o sintoma do §5 (a população de cenas), não o fim dele |

Vermelhos que já vinham do `main` (ship.sh, linhas de outras): `typos` (`Mch` em `autokey_smoke_componentes.rs`, UIUX F27) e
`cargo-machete` (`hashbrown` sem uso em `spikes/regua-registos`, Luz 2D) — curados aqui (comentário; dependência e o `Cargo.lock`
da spike, que é workspace própria).

## §5 — ABERTO

- **Decisão do dono (velocidade, FERRAMENTAS_2 §4):** a água custa o que custa. Medido sozinho no `ci-test`, fatia de 16
  núcleos, mínimo de 3 intercalado: `a_largura_e_a_prometida` **63,8 (18/10) → 135,2 → 101,0 s** com o `DeUmaVez`;
  `cada_preset_parece_a_sua_midia` **26,8 → 51,9 → 36,9 s**; `seam_estado_escondido` 70,3 s (nasceu a 71). O resto é
  **população** (fábrica 18 → 29 presets do Wet, 23 → 36 cenas; o Blender Soft é ⅓ da largura): por preset está abaixo
  do de 18/10. O chão só desce com **um passo de água mais rápido** (trabalho de motor) ou **menos cenas** (decisão de produto).
- Macros × tinta por assar: a gravação da família «mundo» lê a sprite (sem a tinta do Painter ainda não assada). Aplicar já
  vai pelo documento do Painter. Se o dono pintar e gravar o mundo sem sair da sprite, a imagem embutida é a de antes.
- Vindos de FERRAMENTAS_2 §4: (a) **produto** — a Borracha na Aquarela apaga igual ao Digital: tirar do menu, ou construir
  «apaga a aguada mantendo a sessão»? · (b) preview da Borracha no Wet ainda pinta o S · (c) pegada ~1,9× da Borracha Wet ·
  (d) Borracha Wet não levanta tinta seca depois de «Wet canvas» · (e) Texture Eraser em Strength 1 · (f) Flow só no Pincel
  Digital · (h) pin de 14 px; (i) conta-gotas só smokado em foto · (j) presets de utilizador com Accumulate off e Flow < 1
  pintam diferente · (k) Máscara/Fill/Inpaint sem gaveta · `docs/Painter/handoffs/README.md` desactualizado.
- Orlas (§5 de FERRAMENTAS_2): no Glaze Round de fábrica o Wet 0,8 muda só ~13/255; o aro sobe ~2 px menos que o da aguada;
  custo do Wet medido só sob `load 70–90` (W 48 ≈ 36 ms/quadro) — medir em calma.
- O `.config/nextest.toml` e o `FLAKES_DE_CARGA.md` somam entre linhas: quem integrar a seguir reconta as duas escritas.

## §6 — Gate da árvore combinada (esta integração)

- Censos da árvore combinada: 120/121 antes das curas (`the_shell_only_shrinks`, §4 nº 1), verdes depois.
- `ship.sh` sobre o tip (`PH2D_SUITE=fecho`, `NO_FAIL_FAST=1`), 1.ª corrida: fmt ✓ · clippy `-D warnings --all-targets` ✓ ·
  deny ✓ · audit ✓ · nextest **28 699** testes, 28 690 verdes, 9 vermelhos (§4); curados os 9 + typos + machete.
- 2.ª corrida (as curas 1–10): fmt ✓ · clippy ✓ · deny ✓ · audit ✓ · machete ✓ · typos ✓ · índices ✓ · nextest **28 699**, 28 696 verdes,
  1 FAIL (cena 3, nº 11) e 2 TIMEOUT (nº 12). Curados 11 e 12 (só `.config/nextest.toml` e `FLAKES_DE_CARGA.md`); a pesagem final
  é a do `foundational-integrate.sh` (gate da árvore combinada, `PH2D_SUITE=fecho`), cujo resultado está no commit que entra no main.
