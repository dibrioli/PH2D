# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, «o Wet Painter» (2026-10-07 → 2026-10-11)

> **Para o AGENTE INTEGRADOR** (integra por ordem EXPLÍCITA do dono, dada em 11/10). Cobre os **68
> commits** desde o merge-base `09a24afe1` (a rodada de 07/10, [handoff anterior](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-07.md),
> já está no `main`). O *mecanismo*, as tabelas antes·depois·Rebelle, as mutações e as recusas de cada
> onda vivem em **um sítio**: [`05_plano_da_linha_e_o_antes.md` §4b](../NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md)
> — aqui só o que decide a integração. Aberto: §8.

---

## §1 — Coordenadas

| | |
|---|---|
| ramo · worktree | `line/PainterWatercolor` · `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| base | merge-base **`09a24afe1`**; HEAD de código/memória `f4fcc9c6b` (+ o commit deste handoff) |
| `main` hoje | `78a8d90db`, **72 commits** à frente do fork ⇒ **há rebase** |
| diff | **432** ficheiros, `+22 053 / −463` (a maior parte é `docs/` e pranchas; código: ~132 ficheiros em 10 crates) |
| crate nova | **`ph2d-brush-presets`** (formato + biblioteca de presets `*.ph2d-brush.png`; no lock só ela; usa `crc32fast`, `miniz_oxide`, `serde`, `serde_json`) |
| ADR novo · contrato congelado | **nenhum · nenhum** |
| `PROJECT_SCHEMA` | **NÃO muda** (a linha não toca `shells/desktop/src/project_schema*.rs`). O degrau 187→188 das gotas foi **desfeito** com elas (§3). |

## §2 — Superfície de colisão (colada de `collision-surface.sh`, 11/10)

```
SUPERFÍCIE DE COLISÃO — line/PainterWatercolor contra main
  merge-base 09a24afe1   ·   68 commit(s)   ·   432 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        187   (base: 187)
      └ tripla do gate               (187, 13, 22)   (base: (187, 13, 22))
    VEC_SCENE_SCHEMA 22 (22) · FLIP_SCHEMA 13 (13) · DOC_VERSION (timeline) 20 (20)
▸ REGISTRO DE COMPONENTES   ph2d-ecs 106 · render 107 · script 107   (iguais à base)
▸ CONTRATO CONGELADO   node.rs intocado · tool.rs intocado
▸ ADR — último 0181, próximo livre 0182; a linha não cria ADR
▸ Cargo.lock — 1 pacote '+name' novo: "ph2d-brush-presets"
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
      660 / 700   crates/ph2d-app-painter/src/painter_bridge.rs  (marcador/allowlist — confira)
```

**Ficheiros tocados pelos DOIS lados desde o merge-base** (`comm` das duas listas): `Cargo.lock` ·
`crates/ph2d-i18n/src/lib.rs` · `project-memory/MEMORY.md` · `project-memory/reference_topic_code_pattern_gotchas.md`.
- `Cargo.lock`: hunks de pacotes diferentes ⇒ deve fundir; se não, **regenere** (`cargo metadata`) em vez de resolver à mão.
- `ph2d-i18n/src/lib.rs`: a linha **apaga** 7 braços do painel Wet Tuning (grupo `paper`, 6 knobs de papel); o `main` **acrescenta** `mod inspector_motion_input;` e `mod node_options_value;`. Regiões distintas — confira que nenhum braço apagado reapareceu.
- Memória: a linha trouxe de volta as entradas do primário (fusão a 3 vias, `f4fcc9c6b`) e recontou os gotchas de código (40). **Releia a contagem contra o `main`** (lição da família «recontagem» do `MEMORY.md`) e mantenha **um ponteiro por ficheiro**.
- `CLAUDE.md` §5.1: o `main` mexeu na árvore; a linha **só** troca a entrada **Painter**. Se conflitar, mantenha as dos outros módulos do `main` e a Painter daqui (≤ 700 bytes, gate).

## §3 — O que a linha entrega, por onda (condensado; detalhe no 05 §4b)

| onda (data) | entrega | portão principal | recusa/decisão MEDIDA principal |
|---|---|---|---|
| **Cerca + W0** (07/10) | a cerca `os_outros_meios_nao_mudam` (FNV da sonda a 28; Digital, Watercolor, Impasto ao byte) + `ferramentas/cerca.sh`; a lei raio→largura medida | a cerca; 3 mutações sangram, o controlo não | degrau 12,5→16 px de raio (larguras 11–15 inalcançáveis) → a **W2** |
| **W1** presets em ficheiro (07/10) | crate `ph2d-brush-presets` (PNG com `iTXt`, `tEXt`, `phTx`; zlib por `miniz_oxide`, não `flate2`); `wetpaint_preset.rs` (`CAMPOS_DO_PINCEL`/`CAMPOS_FORA`); navegador de presets no painel; 4 pincéis de fábrica em `assets/brushes/WetPainter/…` | `o_pincel_guardado_pinta_o_mesmo_traco` (visto vermelho por defeito real, `paper_depth`); 13 mutações | `flate2` recusado (bytes variam com as features) |
| **Preview vivo** (08/10) | a faixa do painel pinta o pincel de AGORA sobre a obra de AGORA (papel incluído) | `a_chave_muda_com_cada_parametro_e_o_traco_tambem` | — |
| **W1b/W1c gotas e borda** (08/10) | gotas pela lei do Rebelle (tela, papel, água), Easy Drips, 5 rondas de smoke, auditoria 2 lentes | vários | ⏪ **DESFEITO em 09/10** (`c9bbfd688`, ordem do dono: *«parece que o traço perdeu qualidade»*): o código voltou a `69b4f1a75`; **não há** `drips*`, Create/Easy Drips, `DripLaw`, nem o degrau 188 (⚠️ um projecto gravado a 188 durante os testes não abre). Fica a **tabela de recusas** (a faixa de transição inunda a folha: 106–127 mil px fora da mancha) e os docs 06–10 (o prompt 10 está **OBSOLETO**) |
| **Smooth Edges no Wet Paint** (09/10) | a lei do Watercolor (`aa_coverage`) sobre a mancha; no painel como **intensidade** (`wet_smooth_edges`, **0 = desligado de fábrica**); a W1g deu-lhe raio e fez amaciar tinta sobre tinta e as linhas da Heavy Edge | `smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes` | — |
| **W1e** água, papel, Tilt, mistura (09/10) | F6 a mistura é a do Rebelle (velatura sobre o seco, sangria sobre o molhado); F2/F3 **um** papel, na secção Paper (o grupo PAPER do Wet Tuning sai: 6 knobs, `wet_tuning` 17→14 comandos); F1/F4/F5 gravidade igual em todos os pincéis, o preset guarda o Tilt | γ seco 0,17→**0,47** (Rebelle 0,42–0,49); `azul_sobre_vermelho…` | a porta WebSocket do Rebelle não mede o papel (memória) |
| **W1f** achatamento (10/10) | o escorrimento deixa de sair em rectângulos: a lei da gota **sai do produto**; a água só corre depois do Up (4096²: 14–45 ms/quadro com ela) | `o_escorrimento_do_s_nao_se_corta_em_rectangulos` (+ controlo dentro) | ~40 variantes da trava na entrada do seco, 6 rodadas |
| **W1g** opções (10/10) | **Live Water** e **Heavy Edge** (desligadas = o traço de ontem AO BYTE); preview exacto; «Paper: None» sem desenho de papel | `com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte` | **O3 «fios finos e compridos» NÃO entregue** (14 variantes, nenhuma passou) |
| **Mistura (prompt 15)** (10/10) | amarelo sobre azul seco já não dá **preto** e cobre; o pincel sujo só apanha tinta molhada | `sonda_camadas` (Rebelle 8.3.4 como oráculo) | a causa do preto era a K–M ligada; recusas: tecto da velatura no mínimo, média geométrica, zerar a re-molha |
| **Assentar não muda o que se vê** (10/10) | as transferências VERTICAIS (assentar, Dry canvas, re-molhar) resolvem a cor pela inversa da lei do ecrã (`assenta::cor`, a porta única do render) — pior Δ 126 níveis antes; valores depois no 05 §4b | `duas_etapas`, `a_mesma_tinta_nao_muda` | ver 05 §4b |
| **A claridade é o PIGMENTO** (10/10, **lei Cobre**) | a Opacity do Wet Paint é a **quantidade de pigmento** (`Sliders::pigmento`); a cor acumula até à escolhida; a camada de cima tapa (`Ecra::Cobre`); tinta molhada parece mais forte e clareia ao secar; `estado.json` ganha `versao`=1 | `com_a_opacity_baixa_a_cor_acumula_ate_a_escolhida` (+7 mutações vermelhas) | ⛔ **a diluição** (recusada pelo dono) e **escalar o carimbo** (medido: o portão do papel come a tinta clara) |
| **Mistura II** (11/10, prompt 16, Fases A–F) | o Wet Drip com as caixas K–M já **não clareia**; **«Glaze layering» = a lei Cobre com camadas transparentes**; o re-molhar fica na caixa das tintas; re-molhar tinta seca leva a cor do assentado | `sonda_mistura_wet_drip` (fora do cone 9 001 → 0); transferências verticais (M7b sangra nas 4 âncoras ⇒ **13/13**) | densidade óptica como representação única · `CAMADA_CONTINUA` · média da densidade na K–M · `CAMADA_FISICO/GAMA/RESOLVE` · a outra raiz da quadrática (tabela H do 05) |

## §4 — Crates e ficheiros tocados (fora de `docs/` e `assets/`)

| crate | ficheiros | natureza |
|---|---|---|
| **`ph2d-brush-presets`** (NOVA) | 10 (`png.rs`, `biblioteca.rs`, `estado.rs`, `modelo.rs`, 3 `tests/it`) | formato de preset; não conhece o motor |
| `ph2d-wet-paint` | 22 (`assenta.rs`, `solver/…`, sondas) | o motor: lei Cobre, assentar/re-molhar, sedimentação da Heavy Edge, mistura |
| `ph2d-tool-painter` | 58 (`tool/…`, `ids/`, 6 `tests/it`) | preset do Wet Painter, composite, Smooth Edges, preview, portões |
| `ph2d-panel-painter-layers` | 21 | navegador de presets, Opacity/Live Water/Heavy Edge/Smooth Edges |
| `ph2d-panel-wet-tuning` | 7 | grupo PAPER sai (W1e F2); 17→14 comandos |
| `ph2d-app-painter` | 8 (`painter_bridge_presets*.rs`, `painter_bridge_preview*.rs`) | ponte dos presets e do preview vivo |
| `ph2d-i18n` | 3 (`app_painter.rs`, `painter_layers.rs`, `lib.rs`) | textos das opções; **7 braços apagados** do Wet Tuning |
| `ph2d-editor-core` | **1, só testes** (`the_painted_control_reaches_a_consumer.rs`, −1 linha) | censo do painel; **`tool.rs` intocado** |
| `ph2d-panel-registry-init` | **1, só teste** (`quantas_entradas_tem_cada_painel.rs`: `wet_tuning` 17→14) | contador |
| `Cargo.lock` | +13 linhas | só a crate nova |

**Foundational:** nenhum registo, nenhum `PROJECT_SCHEMA`, nenhum ADR, nenhum contrato; i18n e dois testes de contagem são os únicos toques fora da família (mais 1 ficheiro em `ph2d-painter-brush`).

## §5 — Portões novos, cercas e o que um merge pode partir

- **Cercas que viajam com o código:** `os_outros_meios_nao_mudam` (Digital/Watercolor/Impasto ao byte) · `opcoes_tests` (opções desligadas = traço de hoje) · `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (as 4 miniaturas de fábrica; **a W2 muda a lei ⇒ regerar com `PH2D_REGERA_PRESETS=1`**).
- **Portões novos de lei** (nomes no 05 §4b): `o_pincel_guardado_pinta_o_mesmo_traco`, `o_escorrimento_do_s_nao_se_corta_em_rectangulos`, `com_a_opacity_baixa_a_cor_acumula_ate_a_escolhida`, `a_tinta_molhada_parece_mais_forte_e_clareia_ao_secar`, `o_traco_nao_muda_de_tom_ao_secar`, `duas_etapas`, `sonda_mistura_wet_drip`, o portão das transferências verticais com o «Glaze layering».
- **Um merge pode partir:**
  1. **Contagens** — `wet_tuning` 14 (se o `main` regerou `quantas_entradas_tem_cada_painel.rs` noutra linha, reconte, não escolha).
  2. **Impressões ao byte** regravadas COM razão escrita (`opcoes_tests`, `grid_ratio_tests`, as miniaturas): qualquer linha que mexa no motor do Wet Paint vermelha aqui é esperada; **reconte contra o `main`, não copie o hash**.
  3. **Censo dos controlos** — a Heavy Edge lê «viva e inerte» com o Shape **Noise** (#47) ⇒ o censo arma o **Checker** no Wet Paint; a pega do ângulo do carimbo no Wet Paint passou a **inerte**.
  4. **Projecto gravado a 188** durante os testes das gotas (só na máquina do dono): não abre; não é um defeito da integração.
  5. **`estado.json` do dono** ganha `versao`; migra-se no próximo arranque, sem tocar no ficheiro.
- **Flake de carga:** `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)); confirme com o `loadavg`.

## §6 — O que só o `ship.sh` pega (NÃO corrido neste handoff)

O gate batched do fecho **não foi repetido** neste handoff; os números por onda estão em cada entrada do 05 §4b. O integrador corre 1× na árvore combinada: `foundational-integrate.sh` → `censos-da-arvore-combinada.sh` → `./scripts/ship.sh` (só por ordem do dono). ⚠️ Crate nova ⇒ `cargo machete`/`cargo deny` hão de vê-la.

## §7 — Smokes (o dono smokou várias ondas sobre binários desta árvore, ver 05 §4b; o smoke do fecho é o do `main`)

Compilar (a 2.ª corrida sem «Compiling»):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop
```

1. `Ctrl+N` → fundo **White** → **Create** → módulo **Painter** → pincel **Wet Paint**. No topo do painel escolha um dos quatro pincéis (**Wet Round · Wet Wash · Wet Dry Brush · Wet Drip**) — cada um aparece **uma vez** no menu e a faixa de pré-visualização muda com cada controlo.
2. **A claridade é o pigmento.** Baixe **Opacity** (≈ 0,3) e passe o mesmo traço 6 vezes por cima: a cor **vai ficando mais forte até à que escolheu**; a tinta molhada parece mais escura e **clareia ao secar**. Errado: parar numa cor mais clara que a escolhida, ou mudar de tom ao secar.
3. **Mistura.** Azul, deixe secar (**Dry canvas**), amarelo por cima: sai verde-creme que **cobre** o azul, nunca preto. Repita com as caixas **Pigment mixing (K-M)** e **Glaze layering (K-M)** ligadas: o **Wet Drip** azul-escuro sobre vermelho **não** clareia para ciano nas pontas dos fios.
4. **Escorrimento.** Ligue a inclinação (**Tilt**): o escorrimento é redondo, sem rectângulos. **Live Water** e **Heavy Edge** (desligadas = como antes) só ligam por clique; **Smooth Edges** (intensidade) amacia a orla.

## §8 — ABERTO

1. **Decisão do dono (pendente): o Staining com K–M.** Com a K–M ligada a tinta MOLHADA sobre o azul seco re-dissolve ~14 % dele (miolo verde-amarelado, 0,35–0,49, nunca abaixo do azul); **Staining 1** faz cobrir (0,98). Subir o Staining de fábrica quando a K–M liga seria lei nova — não pedida, não construída (05 §4b, «Mistura — amarelo sobre azul seco»).
2. **BUGS_painter [#47](../BUGS_painter.md)** — com o Shape **Noise** o traço de Wet Paint quase não aparece (6 contra 93 818 do `None`; **já era assim antes**). Mecanismo provável, lido e não ablado: o depósito é `stamp − (1 − tooth)·gate`, subtracção ABSOLUTA. A cura muda o traço redondo de hoje ⇒ decisão de desenho.
3. **W2 — a largura prometida** (05 §5): o raio do motor sai da `width_table` do preset (hoje 22–30 px a 40; Rebelle 32–41); gate «largura = 2·raio ± 2 px» de 2 a 300; regerar a fábrica no mesmo commit.
4. **Fio único, fino e longo do Rebelle** (O3 recusada; os fios do Drip/Round inclinados são grossos e curtos: 13 · 71 px contra 350).
5. **Mistura II, abertos (a)–(d)** (05 §4b, fim): o filme lilás `[231,186,222]` ao re-molhar azul-escuro sob quase branco (0,05 % dos pixels) pede portão de unidade sobre `assenta`/`levanta`; a inversa Cobre com tintas diferentes muda o ecrã seco > 1,5 níveis em 36/2 560 estados; o salto do Glaze no Dry canvas (20,8 contra 13,2).
6. **Calibração:** sangria no molhado 0,06–0,10 contra 0,27 do Rebelle · o Wash sem a linha escura ao secar (orla 1,03 contra 1,41) · o custo da Live Water · Smooth Edges no máximo amacia o grão da tinta clara molhada (−10 %).
7. **As gotas** (Create/Easy Drips) ficam recusadas pelo dono; quem voltar a elas lê a tabela de recusas W1b/W1c e compara com o dono (05 §4b, «DESFEITO»).

---

## Integração

**Integrada no `main` a 11/10** (ordem do dono, depois da `line/motion-value`), por `scripts/foundational-integrate.sh`:
ponta `8b84cfdb8`, base `78a8d90db`.

- **Rebase** sobre `78a8d90db` (72 commits do `main`; a sobreposição de ficheiros era só `Cargo.lock`, `ph2d-i18n/src/lib.rs` e
  dois de `project-memory/`). Conflitos, pelos estágios: `project_schema.rs` / `project_schema_tests.rs` em `e5078c578` (W1b,
  as gotas `187 → 188`) e `c9bbfd688` (o revert das gotas) — o `main` tem agora `188` = o `Motion Input` da motion-value; como
  a linha desfez as gotas e a ponta não muda o schema, os dois commits intermédios ficaram com o lado do `main` (`:2`) ⇒ **os
  commits entre a W1b e o revert não levam o degrau das gotas** (a ponta é a testada; não bisecte por ali). `MEMORY.md` e
  `reference_topic_code_pattern_gotchas.md` por união; a família dos gotchas recontada a **42** (39 do `main` + 3 da linha),
  um ponteiro só (`65c7ecbab`). `range-diff`: só mudaram os commits com conflito resolvido e o do handoff (o contexto do
  CLAUDE.md). Nenhum marcador em nenhum commit.
- **Primário:** tinha por rastrear os rascunhos de 07/10 de `docs/Painter/NewFeaturesWetPaint/` e
  `crates/ph2d-tool-painter/tests/sonda_tracos_por_meio.rs` (já na linha desde `5895c3f76`, evoluídos) — guardados em
  `~/.local/share/ph2d-integracao-backup/2026-10-11/primario/` e removidos. As entradas vivas da memória do primário
  (PT-BR 3.ª vez, o Rebelle pela porta WebSocket, «só o Wet Paint evolui») entraram pela linha, fundidas a 3 vias (`f4fcc9c6b`).
- **1.ª corrida do portão: VERMELHA** — `n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` passou o tecto de 180 s
  sob a suíte inteira (140 s sozinho no `ci-test`). Cura (`8b84cfdb8`): os três portões de cena caros correm uma cena por
  thread (camadas 140 → 69 s, `uma_mistura…` 65 → 23 s, `o_traco_nao_muda_de_tom_ao_secar` 43 → 18 s; asserções iguais) e
  as camadas + `sem_celulas_soltas` ganham um override de 6 × 60 s em `.config/nextest.toml` com a medida escrita.
- **2.ª corrida: VERDE** — sync dos registos, staleness, `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets`,
  `nextest-impacted` **18 107 / 18 107** (11 lentos) — e `merge --ff-only` no `main`.
