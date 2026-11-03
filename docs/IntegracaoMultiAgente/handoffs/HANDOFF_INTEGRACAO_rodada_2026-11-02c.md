# HANDOFF DE INTEGRAÇÃO — rodada 2026-11-02c (UIUX → GEF → Pixel → Painter)

> RASCUNHO COMPLETO (não comitado). Ramo `integ/2026-11-02c`, base `3df5894489`, main de partida `995303365b`. TIP à
> escrita `b1747dbbe5` (527 commits desde a base); cadeia verde em `292110c7af`. Números dos relatórios `cura-*.md`
> (fora do repo, `~/.ph2d-integ-2026-11-02c/`); SHAs de `git log 3df5894489..HEAD`.
> O ramo `cura/2026-11-02c/grao` JÁ está no TIP por cherry-pick: os SHAs «→ grao» do §3 são os do ramo; no TIP têm a
> mesma mensagem (`git log --grep`). Os SHAs das curas da janela 2 em diante (§3b) são os do TIP.

## 1. Linhas e ordem
UIUX (tip `903262e48e`) → GEF (`59da279031`; schema à mão) → Pixel (rebase de 87, só `MEMORY.md` conflitou; `722aef0e9e`) →
Painter (rebase de 349, só o hook conflitou). `line/TrelloUITeste` FORA (teste). Porquê: UIUX e GEF sobem ambos o schema
213→214; o degrau da UIUX está amarrado a `VEC_SCENE_SCHEMA_VERSION` 27→28 e a uma fixture v27, logo fica com 214 e o GEF
passa a 214→215; a UIUX encolhe a shell (−166) e abre folga ao GEF e à Pixel; a Painter, a maior e quase disjunta, vai por
último (vermelho inesperado, as três primeiras já estão sãs).

## 2. Conflitos e recontagens
- `PROJECT_SCHEMA` = **215**, tripla **(215, 13, 28)** (`python3 scripts/schema-recount.py`): UIUX 214 traço/filtros, GEF
  214→215 sons, `PROJECT_SCHEMA_SEM_SONS = 214`, geometria v194 `0xD7`+5 zeros, teste `a_v214_opens_without_sounds`.
- Degrau 167→168: os dois moveram o mesmo 166→167; escada viva 606/600 ⇒ 167→168 movido verbatim para
  `project_schema_history_v160.rs` (v160..v168), 590 linhas.
- Hook `prazo-do-agente.sh` = versão da Pixel. `MEMORY.md` = a da Pixel. `ph2d-i18n` (`lib.rs`, `chrome_panes.rs`):
  as DUAS tabelas. `Cargo.lock` regenerado (`ph2d-app-text`, `ph2d-pixel-text`).
- CLAUDE.md §5.1: Painter 707→677 B (§5 14 017→13 988/14 000), «Último:» aponta ESTE handoff (`43340afd20`).
- Cadeia: censo HR-15 GEF (`NOT_LANGUAGE` «Sound»; `Err` de `add_by_component`) e Painter (`Canvas`; `wetpaint_fps.rs`) + índice ADR.

## 3. Suíte 1 (tip `292110c7af`): 42 vermelhos
Causa · cura · medição · SHA (cura → TIP).

**GEF (6)** `879f3a491e`·`e0de88724c`·`1e5bb6dc0c` → `fbe2bb445d`·`72288b0b56`·`ac828abf2c`
- registry-init ×2: `ph2d-app-text` varrida sem `FAMILY`/`default`. `FAMILY{key:"text"}`, blocos por `ph2d-app-sync`;
  `FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA += (text, PH2D_GAME_SMOKE)`.
- shell 170/169: `svg_import_smoke_done` → `VecState`; `fn_loc_caps` `main.rs::new` 202→201.
- `the_drag_promises_what_the_drop_does` ×3: veredito movido para `asset_drop::verdict`, rustfmt partiu a cadeia; o gate
  segue a lei, mutações 4/4.

**UIUX (4)** `fbd68d1ccf`·`edbad33913`·`f47e003a4f` → `a637511724`·`f62dd5b57c`·`ec59967848`
- isenção `VECTOR_BOOL_` sem objecto (F50) removida; `the_atlas_clips_every_cell` ×2: âncora nova `draw_path_for_effects(`.
- `no_row_label_carries_its_own_unit` → `a_unidade_aparece_uma_vez_so`: lei do dono 01/11 (unidade no rótulo), UMA vez por
  linha, sem excepções; seis rótulos «(px)/(/s)»; PDF do tutorial 02 regerado. Tentativa «Distance» revertida (`18623b609f`/`661db21712`).
- Os 3 do tool-painter que a bisseção atribuiu à UIUX são os lentos (abaixo).

**Pixel (5)** `42cb9000ab`·`ebe8d3adda` → `053672a175`·`4e86c7dc3a`
- galeria mostra `paint_range_slider`/`paint_curve_editor`; `BENIGN_SET_MODE += set_pixel_filter_hue_mode`
  (reconcilia em `filtros::acompanha`); 3 sítios `safe_clamp` (o de `range_handle_rect` era pânico real, `track.w<0`);
  `a_interface_tem_losango`: o grupo da paleta só perde o cabeçalho sem rótulo E sem key mark; censo do pixel-core desce às subpastas.

**Painter chrome** `26f7f8e8a5`·`f816097d1a`·`2a0ce5ddb5` → `d64f3071fd`·`dca2e264c5`·`a2e30c8972`
- `COLOR-RAW-OK` em `papel_texturizado.rs` (tipar `SrgbRgba` recusado: contrato `RasterEditTool`, 2 cópias por documento);
  catracas do wet_tuning 2→1 e 14→13 ao medido; `o_censo_dos_controlos_so_encolhe` (75,5–76,8 s) → família de carga.

**Os 4 «conhecidos» do Painter** (ramo grao)
1. Cycles `e7e49f465b`: o lóbulo da lâmpada do Real Oil era ~3,2× o físico (sem `G/(4·n·v)`); escondido no opaco, aparece
   com Transmission. Hipótese do handoff (reflexão interna) REFUTADA no Cycles (Tr1÷opaca 0,38 / 0,35 / 0,40). Cura:
   microfacet, `brilho_max` 1,6→1,0. 0,5 L·sat 0,152→**0,125·0,766** (Cycles 0,124·0,801); 1 L 0,115→**0,088·0,708**
   (0,076·0,705); Σ|Δ| 1,43→0,19. Alvos re-medidos no Cycles de hoje; hashes do Clear glass e dos neutros regravados.
2. Grão `d27da04c02`+`6e58eca761`: âncora 0,67 era a tela do motor velho. Oráculo (Blender 5.2.3,
   `docs/Painter/49_realshader/oraculo/relevo_papel_cycles.py`) miolo÷nu 1,708. `papel_na_tinta` 1,5→**7,6** (1,718),
   âncora 0,67→**1,71**, faixa 1,40..2,04. Pode mexer hashes com Paper Relief>0.
3. Splash `f5925c756b`: bisseção `0170428784` (K-M de fábrica, dono 30/10). A régua «R>B+20» era do modo Plain; nova:
   cada pincelada puxa a tela para a SUA cor, limiar 5,0 (K-M on 27,6/12,4/13,0/11,7; off 29,0/32,6/23,9/29,8); mutação sangra.
4. Largura `0595b17d96`+`095629d585`: `de_tabela` largava o patamar após um pico (Smear Wet r7 17,0 para 14). Gerador grava a
   `envolvente_de_baixo`; `raio_do_motor_da_tabela` alinhado; `largura_do_rasto` ≥ 3 colunas (`COLUNAS_MIN_DO_RASTO`).
   r7 17,01→14,05. Water Brush, Wet Drip, Wet Dry Brush, Gouache Opaque também tinham tabelas erradas sob o motor de hoje
   (igual na base) ⇒ 9 presets regerados.

**Molhada** `a_tinta_molhada_nao_alarga_o_traco` (real, 14/12) `1eecae9e4b`+`b176807c40`. Não é o piso de `032cda64a5`
(14 antes e depois). A E10 trocou a `parede_do_traco` pela cedência de Bingham, mas o termo do CORPO dependia só do `susp`;
a faixa molhada (Oil Bristle, corpo 0,16, meio 0) tem `susp≈0` (a tinta pousa no `sett`) ⇒ τ=0 e o nivelamento escoava.
Lei perdida na E10. Cura (`ladeira_do_oleo::cede`): o corpo conta **`susp+sett`**; vale em Jacobi e serial
(`build_flow_field`), não no alisamento. Lente 14/12×3→12/13·12/12·12/12; `sonda_oleo_furos` Thick Impasto p99
24,13/39,73/43,91 e `_grumos` Filbert 3,16 iguais à base. Muda 17 miniaturas.

**Esmaecido em 9 irmãos** `eefde032a6` → `ce129ceca0`. `a_linha_esmaecida…` 620,7 s e `cada_ferramenta_da_agua…` >750 s
(23–24/10: 43–51 s). Idêntico na linha sozinha (572,0 s) e combinada (570,3 s). Causa: a PINTURA do painel (~0,25 s × 5–8 por
ensaio) com população crescida (77 estados × ~50 gestos; água ~220 × 5) e laço em série; o relógio fixo do Wet NÃO é a causa.
Cura sem tocar prazo: `em_paralelo_largo` + trabalho achatado em (estado|ferramenta, gesto) com vagas que reproduzem o
`vistos`; irmãos `_no_digital _na_aquarela _no_impasto _no_wet_paint` e `_smear _blend _water _dry _blow`; faixa isolada
nomeia os 9. 9/9 verdes, 0 falhas, 2 972 alvos / 1 379 gestos; `_no_wet_paint` ~142 s e `_water` ~163 s a load 52–66.

**Motor A** `8d23d9059b`·`14bac5964d`·`bbfa9cfca3` → grao `c4745f06b8`·`9fb89571cd`·`b8687b9e3f`: a cena do FPS liga o Real
Oil por `Click(PAINTER_WETPAINT_OLEO_REAL)` (porta interna); `the_worker_reports_what_a_step_costs` (J13b P7 ligou a óptica,
o tique não traz o motor: `bring_home()` por volta); `a_agua_pelo_compositor…` → família de carga (4 imagens, piso 5).

**Lentos** `7cbb8c6d6b` → grao `011db407a9` (180 s mesmo calmos). Fenda: 137 de 140 s eram a luz do Óleo Real a cada tique
que o gate não lê ⇒ `set_wet_oleo_adiado(true)` na cena (tela e 4 renders com hash igual A×B; 137,7→29,3 s). Corpo da água
(1 505 ensaios) em 5 irmãos de 301. Portão-mãe (4 células em série) por pincel. Depois: 50,5 s · 66–72 s · 27,3/78,7 s.

**Carga (CALMA, `calma.log`)**: `rota_que_paga` **5,3 s** calmo (1,28× era a load 68); `cada_knob…` 170 s,
`cada_interruptor…` 232 s, `pular_a_pincelada…` 233 s (faixa lenta). Regra: passa sozinho em calma ⇒ família de carga;
sozinho já excede o prazo ⇒ regresso de custo, corta-se o custo, nunca o prazo.

## 3b. Curas das janelas 2 e 3 (SHAs do TIP)

**`sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`** `3bf498f1ca` (cura-semglaze.md). Defeito ANTERIOR à rodada:
mudavam as 20 cenas já em `6b4e4f075f` e antes de `0170428784` (vermelho dias na linha Painter; o gate é lento e só corre na suíte).
Não se regravou às cegas: bisseção por cena (`atrib2.py`, 2 worktrees `-bis` com `target/` por reflink), cada ponta de mudança
com **2 leituras iguais**, `rev-parse HEAD` conferido e `touch` no teste (relink). Causadores, todos leis legítimas: `0170428784`
K-M de fábrica (18 cenas), `02ff1d0f9a`/`167ba3db16` (11 cenas pré-0170), `99d1d5806b`/`0046ce2414` borda molhada, `7c8e08d16b` e
`1f9e115c7e` (E10), óleos (`0275929512`, `0f32fb6e75`, `c77e40484c`, `5acb712ec6`, `b5dc9a8dec`, `032cda64a5`), `dc66557b6f` Graphite,
`e7e49f465b` (rodada), `1eecae9e4b`+`b176807c40` cedência (Wet Wash, Wet Liner, Oil Bristle, Liner obra), `92ab6911f0`
(width_table). Zero leituras divergentes; uma leitura-fantasma do Ink Bullet (1 em 5) apanhada pela regra das 2 leituras e
desfeita. Verde por nome no TIP.

**`a_lei_de_omissao_e_a_do_wet_round`** `f119bf17a5`: `cca667c3ec` fez `de_tabela` guardar o FIM de cada patamar; a
`LARGURA_DO_WET_ROUND` ficara com 54 pontos, agora é a tabela limpa do ficheiro (64). PNG do Wet Round inalterado.
**`os_exemplos_sao_os_que_o_motor_grava`** `20d0b93b18`: já reprovava na base; pixels iguais, só `color_mode` das rampas
Rgb→Oklch (`32cdcee7f6`); Sketch Pencil regerado.

**Water Brush (nasceu 31/10; não existe no main)** — três camadas:
1. Gerador `52209c208a`+`9bd0b4ec8a`: a lei da largura corre em paralelo PELA MEMÓRIA, contando o cgroup real (o glibc guarda as
   páginas na arena do fio; contar só o pedido subestima). Water Brush deu OOM a 64 G. `167b0bff41`: a promessa fora da lei
   MEDIDA não se pinta (o refino pedia uma cena de 92 GB). `8f84d533a6`: PNG regerado (pico 33,5 G, 48 G).
2. Motor, `d8e5d237e4`: `apply_wet_push` (só a Water Brush tem `waterPush`) parava a frente no 1.º poro seco ⇒ **biestável** (~110 px ou
   ~22–44 px sobre base Wet Round). Cura: a frente não pára no poro seco. Descartados: dente contínuo, molhado forçado, `has_fluid`, `sealed`.
3. Régua, `01028009c7`: `largura_da_agua` lê só `cy±1,5·m` nos modos de base Wet Round (= só Water Brush); a cova a r≈298 era a
   orla de LÁ da base (monotonia 0,38→0,007). `ddfb0b36ae`: o refino mede até `MAX_BRUSH_RADIUS_PX` 4 096, não até `RAIO_MAX` 1 100
   (a lei diz 1 141,7 a r300). `c7f9cd81ff`: PNG final, fnv `0x43002037db28a835`, pico 27,9 G; o escalonador falha alto se uma medição não cabe.
Verdes por nome no TIP: `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `cada_preset_de_fabrica_volta_ao_byte`,
`a_tinta_molhada_nao_alarga_o_traco`, `sem_glaze…`, `os_exemplos…`, `a_lei_de_omissao…` (`nomes-tip2.log`).

**`a_largura_e_a_prometida` TIMEOUT 1 200 s** (ramo `cura/2026-11-02c/largura2`, `6da46454b0`, relatório cura-largura2.md).
NÃO é regressão do portão de 12×: já eram 795 s sozinho em 29/10; o conjunto cresceu de 45 para 55 presets do Wet. Escalonador
antigo = um preset por vaga, esperando o raio mais caro e a prova da semente (OnceLock) com os fios parados (Blender Hard 370 s,
r2 já 108 s). Cura: fase 1 a tolerância de todos; fase 2 todos os (preset, raio, crua) numa fila só pela memória. Portão partido
em 8 irmãos `a_largura_e_a_prometida_{blend_hard_soft, blend_dots_textured, smear_flat_shattered_soft, smear_spikey_wet, agua, oleo,
lapis, tinta}` (448 casos antes = depois; `as_familias_da_largura_cobrem_o_wet`; filtro da faixa em `nextest.toml` vira regex).
Sozinhos: lapis 11 s · oleo 60 s · tinta 85 s · blend_hard_soft 310 s (2 presets; custo das Blend por medir, §6).
Vermelhos de TABELA velha (Wet Dry Brush r7 17 px por 14; Smear Wet r3,5 9,77 por 7) ⇒ regeração, ver Fecho.

**Janela 3, suite2 (TIP `c7f9cd81ff`: 31 437 ok, 4 FAIL, 5 TIMEOUT, load 25–55)** — `b1747dbbe5`:
- `workspace_src_files_under_loc_cap`: `macro_track/roteiro.rs` 717 e `ph2d-wet-paint/src/oleo_real.rs` 702 > 700. Cura por corte, sem
  isenção: testes do roteiro → `roteiro_tests.rs`; o `mod exato` → `oleo_exato.rs`.
- `the_vertical_step_of_a_stack_asks_the_door`: `showcase/slider.rs` (passo da galeria) passa a `ph2d_tokens::control_gap_px()`.
- `a_largura_e_a_prometida` → ramo largura2 (acima). Os 6 restantes: ver Fecho.

## 4. Escolhas pelo padrão-ouro
Unidade no rótulo sem excepções (dono 01/11) · gate moldado à lei só onde o dono a mudou (K-M, unidade) · âncoras
re-medidas contra o oráculo corrido, nunca a olho · cedência no alisamento recusada (conservador, réguas iguais à base) ·
`SrgbRgba` em `papel_texturizado` recusado · lentos partidos/achatados, nunca prazo, tecto ou `#[ignore]` mexidos ·
`sem_glaze`: 20 hashes regravados só depois de cada causador ter 2 leituras (regravar 16 sem atribuição seria silenciar) ·
Water Brush: curada no motor (frente contínua) e na régua (junto ao traço), não por tolerância · gerador com concorrência
pela memória REAL, nunca por contagem de núcleos · `largura`: partir em irmãos e fila única, nunca `terminate-after`.

## 5. Notas
- Cena `PH2D_MACRO_SMOKE=1` acaba em cinzas/verdes (K-M por omissão); 4 cores puras seriam mudança de cena (produto).
- **Miniaturas regeradas** (24: 14 Oil, Wet Liner, Wet Wash, Gouache Opaque, Smear×5, Wet Drip, Wet Dry Brush + Water Brush):
  18 juntas deram **OOM a 64 G** (cada uma ~32–33 GB) ⇒ regerar em LOTES MENORES, Water Brush sozinha com `PH2D_MEM_MAX` maior.
- `.config/testes-lentos.tsv` (gerado) ainda tem os nomes antigos; regera-se na próxima corrida.
- A mensagem de `599d9be3db` citava 105,7 como recusa da cedência no alisamento: era contaminação entre sondas (furos+grumos
  no mesmo processo 105,53; sozinha 39,73). Corrigida no grao; não citar.
- `sonda_oleo_furos` e `sonda_oleo_grumos` NÃO correm no mesmo processo (a segunda deixa interruptores atómicos).
- **PRODUTO, não mexido:** o candidato `W2_Bristle_candidatos/Oil Bristle` tem group «Bristle» sem `library_folder`; na E10
  (`7c8e08d16b`) migra para Medium 0 / Body 0,16 = tratado como tinta de ÁGUA (antes Bingham usava m = Real Oil 1,0). A medida
  «óleos iguais ao bit» foi nos óleos de controlo com medium 1. É a lei da E10; decidir se um óleo classificado como água é aceitável é do dono.
- Lições: gerador com 18 presets juntos = OOM; ramo de cura feito sobre a linha original mede diferente do TIP (meça no TIP);
  `cargo-test-narrow.sh` corre na cwd do agente, noutra worktree use `env -C <wt>`; subagente não pode `PH2D_REGERA_*`.

## 6. ABERTO
- Medir a `sonda_oleo_furos` SOZINHA com a cedência também no alisamento (variante de `56426c88f8`).
- Especular do AMBIENTE ~1,4× o do Cycles; custo por pintura do painel (i18n `tr` em debug, 14 % em `str::eq`).
- `o_dente_do_papel_nao_atravessa_a_tinta_grossa` (129 s) e `o_papel_nos_vaos_entre_lascas_nao_fica_preto` (73 s): mesma
  família dos lentos, não medidos. Tempo SOZINHO em calma de 7 dos 9 irmãos e dos 3 lentos. clippy das curas não corrido.
- Custo das Blend na largura (~150 s por preset; `blend_hard_soft` 310 s): medir por raio com a sonda de `sondas.rs:173`
  (`PH2D_SONDA_PRESET="Blender Hard"`) no merge-base `3df5894489` × TIP; suspeita = motor da frente da água (`bordas.rs`) ou a cena
  da base das ferramentas. Legítimo ⇒ um preset Blend por irmão; regressão ⇒ curar a causa. Alvo < 120 s calmo; nunca subir prazo.
- `.config/testes-lentos.tsv` (gerado) e o «94,8 s» antigo da largura estão velhos; regerar na próxima corrida.
- `ph2d-tool-painter`: `sem_glaze` é lento (~25–50 s) e só corre na suíte; vermelho dias na linha sem ninguém ver.

## Fecho (janela 4, 03/11) — main LOCAL, sem push, com vermelhos conhecidos
⛔ **Ordem do dono 03/11:** integrar as 4 linhas (UIUX, Game Engine, Pixel Art, Painter) no main LOCAL já, **SEM push e SEM CI**
nesta rodada; o push fica para a próxima, quando tudo estiver verde. O `ff` foi feito à mão (o `integrar-rodada.sh ff` exige a
árvore no registo verde, que não existe). Sem `ship`, sem run de CI. TIP da rodada: `62e7fc8505`.

**Últimas curas**
- Régua da largura pela média sobre a fase (`e0595e8489`).
- 5 presets regerados (`6cbcce804c`) ⇒ `_agua`/`_oleo`/`_smear_flat_shattered_soft` verdes.
- **Causa do Smear Wet no MOTOR:** `ph2d-wet-paint` `trail/deposit.rs:221` + `tools/smear.rs:258` escreviam o byte de humidade
  tudo-ou-nada na orla (meia célula da orla da base cx=80+2m ⇒ zona larga de 3 vs 4 colunas conforme a fase). Cura
  `round(byte·cobertura)` + régua com orla por cobertura + cena do Smear com `cy` inteiro (`62e7fc8505`). Teste novo
  `o_rasto_do_smear_wet_e_continuo_e_nao_depende_da_fase` verde (salto máx. 0,76 px entre motores, 0,79 entre fases).
- **Escolha pelo padrão-ouro:** curar no motor (defeito real de discretização sub-píxel), apesar de mudar a borda de ~24
  presets — o dono olha no smoke.

**VERMELHOS CONHECIDOS que entram no main local (curar na próxima rodada)**
- Regeração ainda em curso na worktree `cura-02c-largura2` (`regera-l5.sh` no scratchpad da janela 4; log
  `/home/enio/.ph2d-integ-2026-11-02c/regera-l5.log`; feitos: Smear Wet + Fan Brush, Blender Soft, Blender Textured):
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (24 presets) · `sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`
  (13 casos) · `a_largura_e_a_prometida_smear_spikey_wet` e `_smear_flat_shattered_soft` até regerar.
- Os 7 testes da família de CARGA do ESTADO (passam sozinhos/calmos; lista na secção acima) e
  `a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio` (por confirmar calmo).
- Os campos `largura2` / carga / `b2` / `ff` / `ship` / `CI` do rascunho anterior ficam SEM preencher de propósito: não houve
  b2 verde, ship nem CI nesta rodada.

**Aberto (não vermelho):** a cena do misturador na régua da largura tem o mesmo `cy=h·0,5` com `h=(6m)+80` (meia fase
escondida); por medir.

**Decisões do dono 03/11 para a próxima onda:** protocolo MANUAL do dono para os pincéis (comparar o gizmo com o traço), com
proposta de alarme automático barato; pesquisa sobre desenvolvimento rápido em
`/home/enio/PESQUISA_DESENVOLVIMENTO_RAPIDO_2026-11-03.md` (em curso).

**CLAUDE.md §5.1:** conferido no TIP — Painter, UI/UX, Game Engine e Pixel Art já apontam para handoffs desta rodada; sem mudança.
