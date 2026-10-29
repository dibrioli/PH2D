# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, Óleo Real W3b + Missão Pincéis 2 (J6), fechada 29/10

> Ramo `line/PainterWatercolor`, 288 commits desde o merge-base `c3662cd8e4` (`git log --oneline c3662cd8e4..HEAD`); 1134 ficheiros, +402 124/−18 337 (a maioria PNG/JSON/docs). Nada integrado, sem push. SHAs por item: [`ESTADO.md`](../Pinceis/ESTADO.md) · [`HANDOFF_JANELA_6.md`](../Pinceis/HANDOFF_JANELA_6.md) · [`HANDOFF_W3b.md`](../49_realshader/HANDOFF_W3b.md) · método [`REGRAS_DE_CUSTO.md`](../Pinceis/REGRAS_DE_CUSTO.md).

## 1. O que mudou

**Knobs de motor** (cada um com gate; mutação no commit): `wetOpenEdge`+`strokeFilmDome` `8394f62eed` (pena `5f6ae47051`; px pela razão da grade `8884c33c1d`, gate `o_domo_do_filme_e_o_mesmo_em_px_a_toda_a_razao`) · `strokeFilmEnds` `5cd611289a` · `pontas_rectas`/`pontas_alem` `10eee50b0d` `fd73b349ca` · `pontas_esbate` `5b8c03e2ca` · `dryerLevel` `48da05baa5` · `blendReach` `eac8182beb` · `wetCover` `600152c535` · `smear_fio` `3f6377fcec` · `DabJitter::subpixel_area` `f9fbbe7c26` · `bristleLaneSizeExp` `a7cd157000`.
- Controlos: **Paper Press** 0..1 (Tuning ▸ Material; 1 = omissão = ao bit) `b17f8d64c3`; **Same as Paper** no Grain do Digital/Impasto/Wet `1d0ccb1524` (segue o papel `bfa866b7ea`, clique `f7cd85fd8f`, só ferramentas com grão `4860ad535c`, 4 meios `eeb4efd96d`).
- Gate `a_lamina_de_cada_preset_e_a_do_oraculo` (lâmina fixa no eixo). Óleo Real W3b (27–28/10): Transmission, Paper Relief, Smooth Edges, Depth/Gloss/Real Oil por traço (`65d6a6a16d` … `6cab0a4026`; mecanismo em HANDOFF_W3b).

**Curas:** `borda_suave` não apaga o grão de 1 px `9fba4165d8` (gate `o_grao_de_um_px_sobrevive_ao_oleo`) · spray do Wet uma gota por lane `3cc04a5d93` · dual compõe na ponta achatada `9dfb735b24` `095a508bfa` · domo/pena/Paper Press em px `8884c33c1d` `820566c218` · fecho das pontas sem a mão parar `8884c33c1d` · grão de profundidade por dab `ba68f2166f`.

**Oil/\* com Real Oil** (dono 28/10): fábrica `e314e58cec`; 10 regerados `b00ee7cbdc` `3eea4188a8`. Exame 10× mais rápido `eabfc63ccd` (largura sem Real Oil; luz do óleo só na foto: gate `a_luz_do_oleo_so_na_foto_e_a_de_cada_passo`).

**Aprovados:** Soft Pastel e Wax Crayon entram (cega); os 5 óleos Filbert · Knife Textured/Dry/Smooth 3/Oily MANTÊM-SE com Real Oil (cega de 9, Rebelle achado 11/27, acaso 9). Total **28**; fábrica **66 PNG** (WetPainter 45 + Digital 20 + Watercolor 1; `the_factory_presets_are_counted`).

**Régua da largura** (`91de0982f4` `7a9f7fa2ee` `2a39e3f2d8` `213d512cd2` `3ff82ff892`): largura equivalente = área ÷ pico por coluna, mediana de 3 sementes (≤ 4 % da de 9), cena do misturador cresce com o raio, largura gravada pelo menor texto do f32. Regerados (`f623fa0ecb` … `29e11dd166`), px nos raios 13,5 / 52,8 / 122 / 281,9, antes→depois: Blender Soft 13→12·93→61·212→134·496→339 · Hard 13→13·94→62·217→137·507→348 · Textured 16→14·119→75·250→170·607→418 · Dots 12→12·53→62·121→139·298→349 · Smear Soft 22→19·88→86·203→198·469→451 · Spikey idem · Flat 20→18·87→84·201→193·464→440 · Shattered 28→24·123→114·290→239·664→533 · Wet 11→14·91→89·218→213·497→471 (promessa a 2·r ≤ 4 % nas 9).

**Auditorias de 2 lentes.** UI `c4e9cf99e9` `4a17cb76e9` `8a19f601f8`: censo `o_censo_dos_controlos_so_encolhe` reprova clique morto (mutação: tirar `PAINTER_BRUSH_GRAIN_SAME_AS_PAPER` do envio ⇒ 3 vermelhos), Tuning só mostra o knob da ferramenta na mão (`WetKnobs::dona_do_knob`), Depth/Gloss/Paper Press esmaecem sem Real Oil (`Dependente::MaterialSemOleo`). Motor `07b41f2a5a` `8884c33c1d` `820566c218`: 7 achados curados, cada um com gate.

**Tecto de 700:** `0ce0b04e0d` `341c504211` `72c67fb9b4` `f2dbd1642c` `a06cbaa9cf` `65ceebba40` (só mover; `pub use`). **Gate novo** `scripts/nextest-filtros-casam.py` (em `scripts/ship.sh`): todo `test(=…)` dos overrides de `.config/nextest.toml` casa um teste.

**Cura do vermelho** `cada_preset_da_agua_cai_na_foto_do_seu_rebelle`: curado em `c0c7e17b15` — a escala do exame do Smear deriva da largura do rasto do Rebelle pela régua nova (`mistura.rs:458` `ESCALA_DO_EXAME`: Soft/Spikey 0,59, Flat 0,76, Shattered 0,5, Wet 0,59); Fundo Soft/Spikey 92 → 104 (alvo 119 ± 25, tolerância intacta); gate novo: o rasto cai a ±0,5 + 10 % da foto.

## 2. Superfície de colisão (foundational, aditivo)

Fora de `ph2d-*painter*`/`ph2d-wet-paint`/`ph2d-tool-painter`: `ph2d-editor-core` (`property_row/inativa.rs` novo, `casas.rs`; `left_rail/tools.rs` + tabelas partidas; `interaction/state/*` partido; `number_input`; `icons`) · `ph2d-i18n` (chaves + partição) · `ph2d-panel-wet-tuning` (`plano.rs`) · `ph2d-panel-registry-init/tests` (contagens) · `ph2d-app-macros/src/mundo/*` + `tests/it/*` (`ea39947224`, `gate7`) · `ph2d-render/examples/mede_oleo_real.rs` · `.config/nextest.toml` · `scripts/ship.sh` · `scripts/nextest-filtros-casam.py` · `project-memory/*`. Aditivo; sem mudar assinatura de trait.
Pode partir num merge: `MEMORY.md`, `CLAUDE.md` §5.1 (linha do Painter), `nextest.toml` — união; `ph2d-i18n`/`left_rail` se outra linha somou chaves/linhas.
**Contratos §6** (`git diff c3662cd8e4..HEAD -- crates`): nenhuma linha +/− com `NodeOp`, `NodeManifest`, `OpResolver`, `VectorOp`, `trait Tool`; zero ficheiros em `ph2d-nodegraph`, `ph2d-tool-api`, `ph2d-vec*`, `ph2d-project*`, `shells`. **`PROJECT_SCHEMA` não tocado** ⇒ sem recontagem (main 209, Pixel, não colide).

## 3. Números MEDIDOS

Óleo por cena (Knife Oily) 15,84 → 1,60 s (9,9×), fnv `0xd8885bb7515c2019` igual · sonda da largura 3,6–3,7× · regerar Knife Oily 17–27 min → 224 s · Real Oil 1,65–1,85×/quadro ao bit (`d94e19a57e`) · fecho das pontas 2 451 dabs/51,3 s → 32 · spray do Wet 645 lanes → 1, tabela 24 G+ → ≤ 4,1 GiB, p99 317 → 40 px · `a_largura_e_a_prometida` 795 s (579 sem a prova da semente) · censo das ferramentas da água 167,8 s · fio do Smear linhas vazias 35/118 → 0; misturador 52 → 36 px (Rebelle 30).
**`nextest-impacted` 19 506 testes em 1 418 s** (medido no fecho, `c20048c2de`); 9 timeouts eram carga (passaram sozinhos a load 1–13).
Fecho: fmt `f88f55426d`, clippy `-D warnings --all-targets` em `ph2d-tool-painter` `21e0dd183e`. Estado do gate: tudo verde menos `a_cena_3_conduzida_passo_a_passo` (CodeEditor, pré-existente no main, [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)). Suíte completa: do integrador.

## 4. ABERTO

- Wax Crayon mudou de Spacing depois da cega (0,25 → 0,1 + Continuous, `eeb4efd96d`): o dono olha no smoke. Misturadores pequenos pintam ~metade da largura de antes (= o que o painel diz).
- Motor (ESTADO): óleo — luz cresce com o pincel, grumos do misturador, poros da Granulation; Gouache Flat com `wetCover`; lâmina que gira; partículas; mip das pontas; CAP-19 cerdas; azul sobre vermelho molhado. Curados não aprovados e Mop reprovado: HANDOFF_JANELA_6 §1.
- Brecha `arch_color_space_typed`; Ink Brush p95 12,54 ms a vigiar.

## 5. Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D && env PH2D_COMPOSITE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke` (depois da integração, no main).
2. No Painter, pincel **Soft Pastel** (Pastel ▸ Dry) e **Wax Crayon** (Pastel ▸ Oil): giz com pó e giz de cera, sem pontas redondas.
3. Qualquer **Oil/\*** (Knife, Filbert, Dabber): o brilho do óleo já vem ligado.
4. **Tuning ▸ Material**: mova **Paper Press** (0 segue o papel, 1 amassa); sem Real Oil fica esmaecido.
5. **Grain** (Digital, Impasto, Wet Paint, Blur/Smear/Clone): marque **Same as Paper**; o grão segue o papel.
6. Abra `docs/Painter/Pinceis/tutorial/J6_folha_do_dono.html`, marque serve / quase / não serve.
Deu errado se: um controlo mexe e a tela não muda, o óleo não brilha, ou o Wax Crayon sai grosso e picotado.
