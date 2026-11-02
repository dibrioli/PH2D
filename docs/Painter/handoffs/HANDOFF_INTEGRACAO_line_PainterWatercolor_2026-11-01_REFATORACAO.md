# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, refatoração do Painter (Wet Paint), janelas 2–11, fechada 01/11

> Ramo `line/PainterWatercolor`, merge-base `422fec2027`: **286 commits** (251 no `--first-parent`; 3 junções internas `5a48fc7fc6` r3, `8c792f0330` r1, `8c1f5881cd` r2), HEAD de código `093fef0d7b` (+ o commit dos handoffs que o coordenador faz depois). `git diff --stat 422fec2027..HEAD`: **738 ficheiros, +56 066/−2 036** (338 em `docs/Painter`, 40 PNG de miniaturas). Nada integrado, sem push. Arquitetura: [ADR-0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md) · índice da obra: [`Arquitetura/README.md`](../Arquitetura/README.md) · handoffs por janela em `../Arquitetura/HANDOFF_31_10_noite.md`, `_janela2/3.md`, `HANDOFF_01_11_janela4..11.md`. Anterior desta linha: [PINCEIS2](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-29_PINCEIS2.md).

## 1. O que mudou (uma linha por tema; a narrativa vive no doc de cada etapa)

| tema | commit principal | doc |
|---|---|---|
| Crate nova `ph2d-paint-tinta`: lei única de tinta (mistura Oklab pesada pelo volume, camada Kubelka-Munk, conservação; oráculo `lei_unica.py` + fixture `tests/fixtures/lei_unica.tsv`) | `0eaf6fa1ed` | [F4_1](../Arquitetura/F4_1_tinta.md) |
| E4 receita × quantidade (Pigment+Water = só a diluição; Volume = só a quantidade) + migração dos presets | `7a4bc540b7` `b7f54356d6` | [E4](../Arquitetura/E4_receita_e_quantidade.md) |
| Bingham no escoamento do óleo grosso (não escorre fora do pincel) + piso do 1.º toque | `c5e6daa0a9` `43dd8acc3d` | [RELEVO](../Arquitetura/RELEVO_oleo_ondas_e_crateras.md) |
| `Sim::tinta` (Velha/Pigmento), `CelulaTinta`, E5 ecrã, E6 depósito/pick-up, E7 escoamento, E8 tempo, lei seca (cor escolhida = masstone seco) | `c5e6daa0a9` … `723e67e39b` | [E5](../Arquitetura/E5_ecra.md) [E6a](../Arquitetura/E6a_deposito.md) [E6b](../Arquitetura/E6b_pickup_e_ferramentas.md) [E7](../Arquitetura/E7_escoamento.md) [E8](../Arquitetura/E8_tempo.md) |
| Meio na célula (`Soma::m`), ecrã em sRGB, régua plana lê seco | `8d6ec43322` | [E9](../Arquitetura/E9_portao_mae.md) |
| Ponta do Thick Impasto: ruído de saturação por cerda com média zero | `787663c0e3` | [E9b](../Arquitetura/E9b_ponta_thick_impasto.md) |
| **Pigment mix** (slider 0..1, lado espectral `Espectro`, `Grid::pigmento` preguiçoso; `p=0` ao bit) | `28c5db8bd0` `be475f3a46` | [E10b](../Arquitetura/E10b_pigment_mix.md) |
| **Painel E10**: Dilution, Volume (µm), Medium, Body, Hardness, Pigment mix nas duas leis; camada fina presa ao matiz; Absorbency por tela | `e8f35b5710` `4e725aff41` | [E10a1](../Arquitetura/E10a1_receita.md) [E10b2](../Arquitetura/E10b2_painel.md) [E10d1](../Arquitetura/E10d1_camada_fina.md) [E10d2](../Arquitetura/E10d2_absorbency.md) |
| **Real Oil na célula** (`pincel_de_oleo`, 28 B), K-M no invólucro, `seca` da GPU lê `retencao` | `04944ad5a5` | [E10e1](../Arquitetura/E10e1_oleo_na_celula.md) [E10e2](../Arquitetura/E10e2_km_e_remolha.md) |
| **Óleo diluído escorre numa lei só** (diluente sempre líquido; pigmento prende 1/9) | `f8a006c60b` | [E10g](../Arquitetura/E10g_oleo_diluido.md) |
| **Paint type: Water · Oil** (slider Medium vira seletor; Acrylic = Water com Re-wet 0) | `92a2d55e59` | [E10f](../Arquitetura/E10f_paint_type.md) |
| Pequenos (i18n órfã, `wet_opacidade` morto, parede do shader binding 18) e ferrugem do assentar da tinta opaca | `a91a7e33b1` `19ef5bdcac` | [E10h](../Arquitetura/E10h_pequenos.md) [E10i](../Arquitetura/E10i_ferrugem_velha.md) |
| J7 — óleo pouco diluído pousa como PASTA (`meio::pasta`, `pousa_pasta`); miniaturas regeradas 1× (31 + Fan Brush) | `ce1b739ad8` `ff250d8847` | HANDOFF_01_11_janela7 |
| J8 — réguas curadas (Water do Smear provado pela conta; régua da água sem o Rebelle; tecto da guache 13 → 21 pelo Body) | `f20ed68601` `4fb0c114b1` | [J8](../Arquitetura/J8_vermelhos_antigos.md) |
| J9 — o meio anda com o líquido (`solver/meio_transporte.rs`); a «água que sobe a ladeira» era a pegada do pincel; composição só paga a óptica do Real Oil com tinta de Real Oil (Wet Drip Oil d0,9 7,3 → 0,5 s) | `73ce4379d6` `2d7e44b483` `364e5e1457` | [J9](../Arquitetura/J9_meio_no_liquido.md) |
| J10 — **o chão como FORÇA** (`solver/chao_forca.rs`, `Grid::chao`; `v += G·∇(h+chão)`, G 0,125) | `38b497468f` | [J10](../Arquitetura/J10_chao_como_forca.md) |
| J11 — **atrito do leito** (frente no corpo do óleo 99,8 → 90,1 contra 91,5 no papel) + **água parada dissolve** `Re-wet × Solubility` (`solver/remolha_parada.rs`, orla 0,803 → 0,071 com Re-wet 1) + `Grid::sett_do_traco`; contador de sujo `painter/conta_sujo.rs` | `093fef0d7b` | [J11](../Arquitetura/J11_agua_no_oleo.md) |
| Velocidade/GPU: crate nova `ph2d-wet-gpu` (advecção/passo em WGSL, paridade), 3 junções r1/r2/r3 (Wet Wash/Liner, `Knob::EdgeSoftMass`; Splatter e Thick Impasto na fábrica; a tela do Wet compõe-se 1× por quadro) | `5a48fc7fc6` `8c792f0330` `8c1f5881cd` | [`Arquitetura/README.md`](../Arquitetura/README.md) |

**Fica de fora (não mudou):** a E10c/E11 (GPU com meio e chão), E12 (a lei Velha ainda é o produto), E13, os outros 3 modos — ver §6. **Produto hoje pinta com a lei `Velha`**; a Pigmento vive atrás do interruptor.

## 2. Superfície de colisão (cole do `collision-surface.sh` no worktree; contagens aqui são da `git diff --name-only`)

Ficheiros por crate: `ph2d-tool-painter` 132 · `ph2d-wet-paint` 129 · `ph2d-panel-painter-layers` 23 · `ph2d-wet-gpu` 16 (NOVA) · `ph2d-paint-tinta` 16 (NOVA) · `ph2d-painter-brush` 10 · `ph2d-panel-wet-tuning` 6 · `ph2d-i18n` 5 · `ph2d-editor-core` 4 · `ph2d-app-painter` 4 · `shells/desktop` 3 · `ph2d-pigment` / `ph2d-label-census` / `ph2d-image-import` 1 cada · `scripts/amostra_cpu.sh` · `project-memory/MEMORY.md` + 1 arquivo.

| ponto | o que é | risco no merge |
|---|---|---|
| `Cargo.toml` (+6) · `Cargo.lock` (+19) · `crates/ph2d-{app-painter,tool-painter,wet-paint}/Cargo.toml` · 2 crates novas | membros do workspace + deps | `Cargo.lock`: regenerar, nunca resolver à mão; censos por prefixo de crate (HOWTO §2) |
| `crates/ph2d-wet-paint/**`, `ph2d-tool-painter/**`, `ph2d-panel-painter-layers/**`, `ph2d-paint-tinta/**`, `ph2d-wet-gpu/**` | dono desta linha | só colide se outra linha tocou o Painter |
| `ph2d-i18n` (`painter_wet_cartao.rs` novo, `painter_layers.rs`, `wet_tuning_avancado.rs`, `chrome_panes.rs`, `lib.rs`) | chaves do painel E10 | união de chaves se outra linha somou; censo de chaves (`ph2d-label-census/src/keys.rs`) |
| `ph2d-editor-core` (`ids/menus.rs`, `screens/hero/context_menu_dialogs.rs`, 2 testes `tests/it/*`) · `shells/desktop` (`fill_drag.rs`, `fase_new_image_modal.rs`, 1 teste) | pequenos cortes de contagem/rótulo, 30 linhas | aditivo; conferir `menus.rs` se outra linha somou ids |
| `ph2d-pigment/src/lib.rs` (+44), `ph2d-image-import/src/lib.rs` (+9) | helpers de cor | baixo |
| Ficheiros perto do tecto de 700 linhas | `ph2d-wet-paint/src/painter.rs` **696** · `solver/advect_jacobi.rs` **673** · `assenta.rs`/`trail/transfer.rs` em 700 exactos (J7) | quem somar linhas nelas estoura `architecture_workspace_file_loc_cap`: a cura é MOVER |
| `assets/brushes/**` | 40 PNG (WetPainter 38: Oil 14, Smear 5, Blend 4, Watercolor 6, Ink 3, Pastel 2, Pencil 2, Acrylic/Marker/Water 1; Digital/Splatter 1) | binário: em conflito fica o da linha; **miniaturas só se regeram pela janela principal** (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` é o gate) |
| `project-memory/MEMORY.md` | +1 linha | união; tecto de bytes com gate |
| `CLAUDE.md` §5.1 | linha do Painter (§7) | união; cabe em 700 B |

**ADR:** +1 — [0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md) (+1 linha em `decisions/README.md`). O integrador reconta o próximo livre (`ls docs/architecture/decisions | sort | tail`); se outra linha também criou o 0197, renumera-se uma por `git mv` + links.
**`PROJECT_SCHEMA`: não tocado** (zero linhas +/− com a palavra em `crates/`) ⇒ sem recontagem de degrau.
**Contratos §6:** `git diff --stat 422fec2027..HEAD -- crates/ph2d-editor-core crates/ph2d-nodegraph crates/ph2d-vec*` ⇒ só `ph2d-editor-core` (4 ficheiros, +30/−3: 2 rótulos de menu em `ids/menus.rs`/`context_menu_dialogs.rs`, 2 testes); **zero** em `ph2d-nodegraph` e `ph2d-vec*`; nenhuma linha +/− com `NodeOp`/`NodeManifest`/`OpResolver`/`VectorOp`/`trait Tool` em `crates/` ⇒ **contrato congelado intacto** (gates `architecture_contract_surface`, `architecture_tool_contract_surface`, `architecture_vector_contract_surface` não devem mexer).

## 3. O que um merge pode partir

1. **Censos que varrem por prefixo de crate** — duas crates novas (`ph2d-paint-tinta`, `ph2d-wet-gpu`): `architecture_*` de membros do workspace/LOC/`#[cfg]` órfão; reconte antes de acreditar numa contagem verde de «zero ficheiros».
2. **`Cargo.lock`**: regenere com `cargo update -w`/check, não escolha lados.
3. **Chaves i18n e censo de controlos** (`o_censo_dos_controlos_so_encolhe`, `every_key_of_this_panel_exists_on_both_sides`): outra linha que somou chaves ao painel do Painter.
4. **Hashes gravados do motor** (`advect_jacobi_bits_*`, `flow_bit`, `assenta_bit`): regravados na J4 pela E4 de propósito; uma linha que mexa no depósito/solver parte-os.
5. **Tecto de 700 linhas** (tabela §2) e o gate `scripts/nextest-filtros-casam.py` se algum `test(=…)` do `.config/nextest.toml` foi renomeado.

## 4. ⚠️ Vermelhos CONHECIDOS (9 testes em `ph2d-tool-painter --lib`, módulo `tool::paint::wetpaint_preset::`)

**NÃO são regressões desta onda (J10/J11):** já estavam vermelhos antes da janela 10 (achados na J9 §4) e foram bisseccionados na J10 — mecanismo e pontos medidos em [`J10_bissecao_vermelhos.md`](../Arquitetura/J10_bissecao_vermelhos.md). Os 7 que já existiam estão verdes no merge-base `422fec2027`; **o culpado está dentro da linha (E10/E10-painel/Real Oil), não na J10 nem J11.** A cura está na FILA da linha (§6). O integrador não deve afrouxar tectos nem regravar alvos para os fechar — a régua é que está errada (N, A) ou ainda não foi isolada (P, S).

| # | teste | ficheiro | família | culpado (1.º vermelho) |
|---|---|---|---|---|
| 1 | `a_transmissao_tinge_o_papel_creme_do_documento` | `medida_transparencia.rs:356` | **N** — miolo vazio, `NaN × 0.456` | `e8f35b5710` (E10 no painel) |
| 2 | `a_transparencia_deixa_ver_o_grao_e_a_cor_do_papel` | `medida_transparencia.rs` | **N** (`NaN × 6.02`) | ≤ HEAD, por isolar (pela mensagem) |
| 3 | `o_dente_do_papel_nao_atravessa_a_tinta_grossa` | `medida_transparencia.rs` | **N** (`NaN × papel nu 6.02`) | idem |
| 4 | `a_transmissao_e_o_relevo_do_papel_mostram_o_grao` | `medida_transparencia.rs` | **P** (relevo, em (124,166]) + N | por isolar |
| 5 | `o_papel_nos_vaos_entre_lascas_nao_fica_preto` | `medida_oleo_real_fenda.rs` | **P** (`Classico 1,46 % < 70`, tecto 0 %) + ? | por isolar (suspeitos `611591d423`, `a6796fd570`, `3570226c1e`) |
| 6 | `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` | `medida_transparencia.rs:443` | **A** (+ N no HEAD, `L NaN`) — modo Blender | `f70ea44cd9` (ladeira fotografada no início do traço) |
| 7 | `a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono` | `medida_oleo_real_fenda.rs:86` | **A′** (curado em `a6796fd570`, volta a 68 > tecto 70 em (166,247]) | provável `f70ea44cd9` + E10, não conferido |
| 8 | `sonda_oleo_furos::o_oleo_sobre_o_oleo_assentado_nao_encaroca` | `sonda_oleo_furos.rs:211` | **S** — sondas do óleo (`p99 24,1 → 110,9`) | por isolar (suspeitos `c5e6daa0a9`, `7a4bc540b7`, `e8f35b5710`) |
| 9 | `sonda_oleo_grumos::o_oleo_sobre_oleo_molhado_satura_o_relevo` | `sonda_oleo_grumos.rs:307` | **S** (`p99 1,49 → 10,95`) | por isolar |

Família N: `e8f35b5710` trocou a receita (`Knob::Volume` em µm × `M_UM`, `phi = 1 − Dilution`); o limiar `sett + susp > 3000` de `cena_papel_texturizado` (`medida_oleo_real_fenda.rs:252`) está na unidade antiga ⇒ miolo vazio ⇒ `NaN`. Cura: limiar relativo + assert de miolo não-vazio. Família A: o alvo `Cycles 0,205/0,744` foi tirado do relevo antigo ⇒ correr o oráculo de novo sobre a cena de hoje. Os nomes exactos e o intervalo de cada um: [bisseção](../Arquitetura/J10_bissecao_vermelhos.md) (a tabela acima é a dela). Se o integrador vir MAIS do que estes 9 em `ph2d-tool-painter`/`ph2d-wet-paint`, esses sim são novos: compare com o pai de cada commit da J10/J11 (A/B em worktree de árvore inteira).

## 5. Prova de fecho (o que correu e o que NÃO correu)

- **A suíte completa NÃO correu na linha** (regra do dono: só na integração). Fecho por nome.
- Verdes por nome na janela 11 (`ph2d-tool-painter --lib`): `agua_no_oleo_regua_tests` (3, mutação M1–M3), `chao_forca_regua_tests` (4), `a_queda_cresce_com_a_diluicao`, `a_agua_nao_sobe_a_ladeira_do_oleo`, `remolha_tests`, `agua_sobre_oleo_tests`, `queda_na_ladeira_tests`, `oleo_sem_real_oil_tests`.
- Verdes em `ph2d-wet-paint` (ao bit da água): `flow_bit`, `advect_jacobi`, `oleo_sobre_oleo`, `assenta`, `meio`, `advect_bit`, `fundida`, `bingham`.
- Anteriores (J4–J10, ver cada handoff): `tinta` (67), `bristles` (22), `o_invariante_soma_massa_vale_no_motor`, `ph2d-paint-tinta` inteira, 7 `pigmento_*`, 4 `pigment_mix_motor_*`, `portao_mae_f0_lei_pigmento_minimo`; grelha E9 inteira (3 240 células) em J5 e J7 ([E9 §8](../Arquitetura/E9_portao_mae.md), [J7](../Arquitetura/J7_pasta_do_oleo.md)).
- **Smokes do dono aprovados:** J7 (Paint type, óleo diluído escorre, amarelo × azul em verde) · J9 (óleo diluído escorre) · J10 («empoçando corretamente») · **J11** («Realmente melhorou muito! Parabéns.», foto [`fotos/j11/smoke_dono_1.png`](../Arquitetura/fotos/j11/smoke_dono_1.png)). Ordem do dono (J11): integrar no main antes de seguir.
- Fmt/clippy `-D warnings --all-targets` em `ph2d-tool-painter`/`ph2d-wet-paint`/`ph2d-paint-tinta`/`ph2d-wet-gpu`/`ph2d-panel-painter-layers`: **do integrador** (1× na rodada).
- Gates do `ship.sh` e `nextest-impacted`: do integrador.

## 6. ABERTO depois da integração

Fonte: [`HANDOFF_01_11_janela11.md` §3](../Arquitetura/HANDOFF_01_11_janela11.md).

1. **FPS com o Real Oil**: o único marcador de sujo é o passo (`painter/stage.rs:48/53`); o custo está no TRAÇO (+2,8 ms por evento do rato, tela 1200×800) e na água que corre por cima do óleo (caixa inteira recomposta a cada passo). Medir no app antes; atacar a composição por passo (ritmos relevo 24×/s e luz 12×/s do dono; adormecer o invisível; marcador discreto ao lado do Dry — UI nova, mostrar no app).
2. O rectângulo velho do smoke J10 (foto 2): o motor não o reproduz (0 px); se voltar, a subida da tela na shell.
3. **Os 9 vermelhos** (§4): N, A, P, S — cura na fila da linha.
4. E9 §8 ⚑ furos (a régua lê o relevo nivelado como furo) · chip rápido de Dilution · `assenta.rs`/`trail/transfer.rs`/`painter.rs`/`advect_jacobi.rs` perto do tecto (partir antes de mexer) · E10c · **E11 GPU com o meio e o chão** (a placa não lê o meio nem a pasta nem o chão) · E12 (a lei Velha sai) · E13 · os outros 3 modos (Digital, Watercolor, Impasto) · ajuda ilustrada (F5) · sondas para crate dev-only (~39 k linhas) · crateras na costura entre pinceladas.

## 7. Smoke do dono (depois da integração, no main)

1. `cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --profile smoke` (máquina calma).
2. No Painter abra o modo **Wet Paint**; pincel **Dabber** (Oil) azul, desenhe um V com o mouse.
3. Troque para **Wet Drip** amarelo e solte uma gota por cima do V: a água deve **empoçar no fundo do V**, misturar-se com a orla (verde) e **não entrar** no corpo do azul.
4. No painel, **Paint type**: troque Water ↔ Oil; com Oil e **Dilution** alta o traço escorre devagar e fica molhado.
5. Deu errado se: a água atravessa o azul como se fosse papel, o painel perde o seletor Paint type, ou o FPS cai muito com um traço de óleo (isso último já é conhecido, §6.1).

## 8. Linha do §5 (para o CLAUDE.md, módulo Painter — ≤ 700 bytes; NÃO editada aqui)

```
- **Painter** — Layers + Efeitos e o motor de pintura clean-room (Digital · Watercolor · Impasto · Wet Paint), crate `ph2d-app-painter`; Wet Paint = UMA lei de pigmento, o meio é um ponto dela (Paint type Water·Oil, [ADR-0197](docs/architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md)); o Rebelle já não é a referência. Smoke `PH2D_COMPOSITE_SMOKE=1`. Último: [refatoração J2–J11, no main 01/11](docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-01_REFATORACAO.md) · [docs](docs/Painter/00_INDEX.md) · [BUGS](docs/Painter/BUGS_painter.md) · [história](docs/archive/estado-2026-10-02/painter.md)
```
