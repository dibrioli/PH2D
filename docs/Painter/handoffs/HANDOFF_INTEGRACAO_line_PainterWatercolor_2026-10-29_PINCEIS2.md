# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, Óleo Real W3b + Missão Pincéis 2 (J6), fechada 29/10

> Ramo `line/PainterWatercolor`, 288 commits desde o merge-base `c3662cd8e4` (`git log --oneline c3662cd8e4..HEAD`); 1134 ficheiros, +402 124/−18 337 (a maioria PNG/JSON/docs). Nada integrado, sem push. Estado vivo e SHAs por item: [`ESTADO.md`](../Pinceis/ESTADO.md) · [`HANDOFF_JANELA_6.md`](../Pinceis/HANDOFF_JANELA_6.md) · [`HANDOFF_W3b.md`](../49_realshader/HANDOFF_W3b.md). Método da missão: [`REGRAS_DE_CUSTO.md`](../Pinceis/REGRAS_DE_CUSTO.md) §METODOLOGIA.

## 1. O que mudou

**Knobs de motor** (cada um com gate; prova de mutação no commit):
- `wetOpenEdge` + `strokeFilmDome` `8394f62eed` (pena recua metade `5f6ae47051`; domo e pena em px pela razão da grade `8884c33c1d`, gate `o_domo_do_filme_e_o_mesmo_em_px_a_toda_a_razao`, mutação `r_px = r` vermelha) · `strokeFilmEnds` `5cd611289a` · `pontas_rectas`/`pontas_alem` `10eee50b0d` `fd73b349ca` · `pontas_esbate` `5b8c03e2ca` · `dryerLevel` `48da05baa5` · `blendReach` `eac8182beb` · `wetCover` `600152c535` · `smear_fio` `3f6377fcec` · `DabJitter::subpixel_area` `f9fbbe7c26` · `bristleLaneSizeExp` `a7cd157000`.
- Controlos: **Paper Press** 0..1 (Tuning ▸ Material; 1 = omissão = ao bit) `b17f8d64c3`; **Same as Paper** no Grain do Digital/Impasto/Wet `1d0ccb1524` (+ segue o papel visível `bfa866b7ea`, clique chega à ferramenta `f7cd85fd8f`, só nas ferramentas que recebem o grão `4860ad535c`, nos 4 meios `eeb4efd96d`).
- Gate novo `a_lamina_de_cada_preset_e_a_do_oraculo` (a lâmina é fixa no eixo: o Rebelle não gira).
- Óleo Real W3b (27–28/10): Transmission, Paper Relief, Smooth Edges por traço, Depth/Gloss/Real Oil por traço (`65d6a6a16d` … `6cab0a4026`); mecanismo em HANDOFF_W3b.

**Curas:** `borda_suave` já não apaga o grão de 1 px (`9fba4165d8`, gate `o_grao_de_um_px_sobrevive_ao_oleo`) · spray do Wet: uma gota por lane, sem rasto por marca (`3cc04a5d93`) · dual compõe dentro da ponta achatada (`9dfb735b24`, `095a508bfa`) · domo/pena/Paper Press em px (`8884c33c1d` `820566c218`) · fecho das pontas independe de a mão parar (`8884c33c1d`) · grão de profundidade por dab (`ba68f2166f`).

**Oil/\* com Real Oil** (ordem do dono 28/10): fábrica com o óleo ligado `e314e58cec`; os 10 regerados `b00ee7cbdc` `3eea4188a8`. Exame/gerador do óleo 10× mais rápido `eabfc63ccd` (a largura mede sem o Real Oil; a luz do óleo só na foto: gate `a_luz_do_oleo_so_na_foto_e_a_de_cada_passo`, mutação sem a recomposição = vermelho).

**Aprovados:** Soft Pastel (cega r2) e Wax Crayon (cega; o Rebelle nunca achado) entram; os 5 óleos Filbert · Knife Textured · Knife Dry · Knife Smooth 3 · Knife Oily MANTÊM-SE com Real Oil (cega de 9, Rebelle achado 11/27, acaso 9). Total **28** (26 + 2); fábrica **66 PNG** (WetPainter 45 + Digital 20 + Watercolor 1; contagem oficial: `the_factory_presets_are_counted`).

**Régua da largura nova** (`91de0982f4` `7a9f7fa2ee` `2a39e3f2d8` `213d512cd2` `3ff82ff892`): largura EQUIVALENTE = área ÷ pico por coluna (contínua no raio), mediana de 3 sementes pela sensibilidade medida (uma semente varia ±10 %; a mediana de 3 fica ≤ 4 % da de 9), cena do misturador que cresce com o raio (a lei crua do Blender Soft deixa de cair de 248 px a 0 entre raios 723 e 990), largura gravada pelo menor texto do f32. Presets regerados: Blender Soft/Textured/Dots/Hard, Smear Soft/Spikey/Flat/Shattered/Wet (`f623fa0ecb` … `29e11dd166`). Antes→depois por ferramenta: ⏳ tabela do coordenador (não está nos commits).

**Auditorias de 2 lentes.** UI `c4e9cf99e9` `4a17cb76e9`: censo `o_censo_dos_controlos_so_encolhe` reprova clique morto (mutação: tirar `PAINTER_BRUSH_GRAIN_SAME_AS_PAPER` do envio ⇒ 3 linhas vermelhas), Tuning só mostra o knob da ferramenta na mão (`WetKnobs::dona_do_knob`), Depth/Gloss/Paper Press esmaecem sem Real Oil (`Dependente::MaterialSemOleo`), `pela_porta_do_rato`; `8a19f601f8` a linha esmaecida pergunta ao grão com que o motor pinta. Motor `07b41f2a5a` `8884c33c1d` `820566c218`: gota do Blend, material perto, índice da mangueira, óleo adiado, Same as Paper do lote, domo/pena em px, fecho — 7 achados curados, cada um com gate.

**Tecto de 700:** `0ce0b04e0d` `341c504211` `72c67fb9b4` `f2dbd1642c` `a06cbaa9cf` `65ceebba40` (só mover; `pub use` mantém os caminhos).

**Gate novo:** `scripts/nextest-filtros-casam.py` (lido por `scripts/ship.sh` sobre `cargo nextest list`): todo `test(=…)` dos overrides de `.config/nextest.toml` tem de casar um teste (o termo `=presets_de_fabrica::a_largura_e_a_prometida` deixou de casar quando o ficheiro se partiu para `portoes::`).

## 2. Superfície de colisão (foundational, aditivo)

Fora de `ph2d-*painter*`/`ph2d-wet-paint`/`ph2d-tool-painter`: `ph2d-editor-core` (`property_row/inativa.rs` novo = o alfa da linha inativa, uma lei para dois painéis; `property_row/casas.rs`; `screens/hero/left_rail/tools.rs` + tabelas partidas; `interaction/state/*` partido; `widget/number_input`; `icons`) · `ph2d-i18n` (chaves novas + partição) · `ph2d-panel-wet-tuning` (linhas, `plano.rs`) · `ph2d-panel-registry-init/tests` (contagens) · `ph2d-app-macros/src/mundo/*` + `tests/it/*` (a macro «mundo» embute a tinta por assar, `ea39947224`; gate `gate7`) · `ph2d-render/examples/mede_oleo_real.rs` · `.config/nextest.toml` · `scripts/ship.sh` · `scripts/nextest-filtros-casam.py` · `project-memory/*`. Aditivo: chaves/linhas/ficheiros novos e divisões em filhos; sem mudar assinatura de trait.
Pode partir num merge: `MEMORY.md` e `CLAUDE.md` §5.1 (linha do Painter) e `nextest.toml` (overrides de outras linhas) — resolver por união; `ph2d-i18n`/`left_rail` se outra linha somou chaves/linhas.

**Contratos §6, provados por grep** (`git diff c3662cd8e4..HEAD -- crates`): nenhuma linha +/− contém `NodeOp`, `NodeManifest`, `OpResolver`, `VectorOp`, `trait Tool`; zero ficheiros em `ph2d-nodegraph`, `ph2d-tool-api`, `ph2d-vec*`; `--stat` sobre `crates/ph2d-nodegraph crates/ph2d-tool-api crates/ph2d-vec* crates/ph2d-project* shells` vazio. **`PROJECT_SCHEMA`: não tocado** (zero linhas +/− com o nome) ⇒ sem recontagem; o main (209, Pixel) não colide.

## 3. Números MEDIDOS

Óleo por cena (Knife Oily, gestos+foto) 15,84 → 1,60 s (9,9×), fnv igual `0xd8885bb7515c2019` (`eabfc63ccd`); sonda da largura 3,6–3,7× · regerar Knife Oily 17–27 min → 224 s · Real Oil 1,65–1,85×/quadro ao bit (`d94e19a57e`) · fecho das pontas (último dab parado) 2 451 dabs/51,3 s → 32 dabs · spray do Wet 645 lanes → 1, tabela 24 G+ → ≤ 4,1 GiB, p99 da gota 317 → 40 px · `nextest-impacted`: 19 506 testes, 1 418 s (relato do coordenador) · `a_largura_e_a_prometida` sozinha 795 s (579 sem a prova da semente) · censo das ferramentas da água 167,8 s sozinho · fio do Smear linhas vazias 35/118 → 0; misturador 52 → 36 px (Rebelle 30).

Fecho: fmt `f88f55426d`, clippy `-D warnings --all-targets` em `ph2d-tool-painter` `21e0dd183e`. Suíte completa: do integrador.

## 4. ABERTO

- ⏳ **Vermelho** `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` (Smear Soft/Spikey; escala do exame): em cura — **⏳ SHA da cura** (coordenador preenche).
- `a_cena_3_conduzida_passo_a_passo` (CodeEditor): pré-existente, família [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md).
- Wax Crayon mudou de Spacing depois da cega (0,25 → 0,1 + Continuous, `eeb4efd96d`): o dono olha no smoke.
- Misturadores pequenos pintam ~metade da largura de antes (agora = o que o painel diz).
- Motor (ESTADO): óleo — luz cresce com o pincel (carris 10,1 × 3,32), grumos do misturador, poros da Granulation; Gouache Flat com `wetCover` (largura do fino); lâmina que gira; partículas; mip das pontas; CAP-19 cerdas; azul sobre vermelho molhado. Curados não aprovados e Mop reprovado: lista em HANDOFF_JANELA_6 §1.
- Brecha `arch_color_space_typed`; Ink Brush p95 12,54 ms a vigiar.

## 5. Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D && env PH2D_COMPOSITE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke` (depois da integração é no main).
2. No Painter, escolha o pincel **Soft Pastel** (Pastel ▸ Dry) e **Wax Crayon** (Pastel ▸ Oil) e risque: giz com pó e giz de cera, sem pontas redondas.
3. Escolha qualquer **Oil/\*** (Knife, Filbert, Dabber): o brilho do óleo (Real Oil) já vem ligado.
4. Em **Tuning ▸ Material**: mova **Paper Press** (0 segue o papel, 1 amassa); sem Real Oil ele fica esmaecido.
5. Em **Grain** (Digital, Impasto, Wet Paint e ferramentas Blur/Smear/Clone): marque **Same as Paper** e veja o grão seguir o papel.
6. Abra `docs/Painter/Pinceis/tutorial/J6_folha_do_dono.html`, marque serve / quase / não serve.
Deu errado se: um controlo mexe e a tela não muda, o óleo não brilha, ou o Wax Crayon sai grosso e picotado.
