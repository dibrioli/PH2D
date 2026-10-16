# CONTINUAÇÃO — `line/motion-value`: a LÍNGUA das cenas (15/10) · 2.ª rodada: nenhuma ficha tapa a figura · 3.ª rodada (16/10): as cenas abrem enquadradas, formas em todas as cenas de ciclo, o crash da =118 e os marcadores na placa (do main), a rota anunciada com gate

> Continua o [ABERTOS 132](HANDOFF_CONTINUACAO_line_motion_value_2026-10-14_ABERTOS_132.md) (válido para o que diz: o
> `Scene` com imagens no dispositivo, a sonda, as recusas). Plano, leis e recusas: [doc 132](../132_zerar_as_seis_pendencias.md).
> **Este ficheiro SUBSTITUI o ABERTOS_132 como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, HEAD `98fe9bb150` (+ o commit de docs deste handoff), base = `main` `0aab2c1cb6` (não andou; rebase vazio). **35 commits por integrar** medidos (`git rev-list --count main..HEAD`), 36 com o de docs. Só integra por ordem do Enio.
- `MAX_DEMO_LEVEL = 142`. Diff da 3.ª rodada (`cbec0eb27c..HEAD`): 123 ficheiros, +4 657 −926; 67 em `ph2d-app-motion/src` (cenas, gates), 2 em `shells/desktop/src/render_loop`, 2 em `ph2d-gpu-cook/src` + 1 `tests/it`, 1 em `ph2d-eval-motion/src`, tutoriais 03–14 (HTML, figuras, PDFs), `fotografa_cena.sh`.
- **Superfície de colisão (rodadas 1–2):** `ph2d-label-census` (`portuguese_tokens`, `keys::declared_pairs_in`, corpus `portugues_cenas_2026_10_14.txt`), ~50 ficheiros de cena (texto, ficha movida), `motion_demo_legend_lingua_tests.rs`, `motion_demo_legend_tapa_tests.rs`, `fotografa_cena.sh`. **3.ª rodada:** `motion_demo_enquadra.rs` + `motion_demo_enquadra_tabela.rs` + `motion_demo_enquadra_tests.rs` (novos); `fase_motion_bridge` na shell (+9 −5, `abre_inteira`); `ph2d-gpu-cook/src/tap.rs` e `attr.rs`; `ph2d-eval-motion/src/taps.rs`; `motion_bridge_gpu.rs` 748 → 669 (+ `motion_bridge_gpu_tomadas.rs`); `jogo_demo::carimbo`.
- **Um merge pode partir:** cena nova com legenda/título PT (censo de língua) · pares do leitor partilhado (ph2d-i18n/src 8 018 → 8 012) · ficheiro de exemplo das cenas =109/=119 (`month,sales,time,level`) · quem mova uma ficha, mude a câmara de arranque ou acrescente uma cena de ciclo: os gates `nenhuma_ficha_tapa…`, `motion_demo_enquadra_tests`, `toda_cena_de_ciclo_desenha_formas…` re-medem; **cena de ciclo nova tem de ter FORMAS, entrada na tabela das extensões (sonda `a_tabela_das_extensoes`) e frase de rota que confira**; quem toque `Stream::set`/`le_tap` ou os gizmos do Motion (leitura pela placa).

## 3.ª rodada (16/10) — `cbec0eb27c..98fe9bb150`

Smoke do dono: **=142 OK (15/10)**, **=87 OK (16/10)** (legendas por fora das figuras).

| commit | o quê |
|---|---|
| `13c68a62d4` + `74da46bb1c` (docs) | **As cenas abrem ENQUADRADAS.** Lei em `motion_demo_enquadra.rs`; tabela medida `motion_demo_enquadra_tabela.rs` pela sonda `a_tabela_das_extensoes` (re-medida a 1 mm pelo gate): figura ao longo de 2·CICLO + fichas no tamanho real (ponto fixo). Fica a vista de 10 m se cabe com a margem do Frame All (44 px), senão `camera_frame::frame_within`, nunca mais perto; cenas-campo abrem na vista por omissão. A shell aplica por `abre_inteira` (`fase_motion_bridge`). FORA-DA-VISTA **53 → 0**; 76 fichas re-medidas com a câmara real (22 cenas); =137 sai das isenções, =95 fica com 6. Gate `motion_demo_enquadra_tests.rs`; =11 não medida (60,8 s/tique), =15/=16 não cabem (181/271 m). 20 fotos dos tutoriais 10–14 + 5 PDFs |
| `ee37a51e6f` | **O corte «voa × cresce no lugar».** Nenhum discriminador de NATUREZA separa (7 candidatos medidos nas 131 cenas, tabela no commit); fica a razão de zoom `razao_da_fuga`, corte 1,55 → **1,51** (meio geométrico do intervalo real 1,454 (=128) … 1,581 (=138), ~4 % de folga de cada lado). Gate com `FOGEM` (11 cenas) em 4 janelas (1930×1040, 2560×1080 do dono, 2560×1440, 1366×768) e tripwire `PERTO_DO_CORTE` 3 %. O doc antigo usava outra razão (=10 e =102 já abriam no ciclo) |
| `f5674426f1` | **Crash da =118 pela placa — não era o Trail** (hipótese refutada). `readout::take_tap` → `GpuCook::tap_sem_espera` → `le_tap` (`ph2d-gpu-cook/src/tap.rs`) → `Stream::set` (`attr.rs:456`): o anel do Slit Scan `ss_ring0..3` é `Dim::Mat4` (`438bd1a49f`) e o `_ => Vec4` do `le_tap` (`3608e67a47`) empurrava 36×16/4 = 144 vec4 contra 36. **ORIGEM: o main** (medido numa worktree temporária no `0aab2c1cb6`, vermelho igual). Cura `cabe_numa_coluna` + `le_tap` exaustivo. Gate `toda_cena_de_ciclo_coze_pela_placa` (`placa:`, 23 cenas × 60 quadros, `o_quadro` **+ a leitura como o dispatch** — a 1.ª versão só com `o_quadro` passava verde) + loop da paridade do slit scan na crate |
| `de58233ef3` (+ `964392b621` docs) | **Marcadores PARADOS pela placa num grafo sem formas.** Os gizmos (pontos, warp, colisor, pivô) liam `pump.tap_streams()` de `cook_taps_only` (`ph2d-eval-motion/src/taps.rs:110`) sobre um estado que nunca marchou (Trail: 72 pontos na CPU × 9 na placa). Origem: o main (08/09 + 19/09). Cura: tomadas da placa pedidas por nome no mesmo envio (`TECTO_DA_TOMADA` = `MAX_PONTOS` 4 096), um quadro atrás; cartão/colunas/valor dirigido passam a preferir a placa ao memo velho da CPU. Custo medido a 32 768 linhas: **0,229 ms** (cura) × 0,193 (sem gizmo) × 0,257 (CPU antigo) por quadro. Gate `os_marcadores_andam_na_placa_como_na_cpu` (`placa:`) + 2 sem placa, mutações 3/3 |
| `c6187829cb` + `81dbf63a46` (docs) | **=118 sem FORMAS desde `72cdfa6f71`** (19/09, a lei «só com uma forma»): agora Grid → Move → Duplicator com Shape; tutorial 07 (prosa + figuras) e PDF. Gates `toda_cena_de_ciclo_que_pinta_aparencia_desenha_formas`, `todo_anuncio_que_promete_cor_tem_formas_em_todo_pano` |
| `4737080011` + `41282c0123` (docs) | **Formas em =111, =112, =113** (o carimbo antes da simulação), **=116** (vão 0,012 → 0,03; cook placa 0,067 → 0,185 ms, HÍBRIDO com o Shape na CPU), **=119** (o emissor ganhou Integrate + gravidade — antes nascia e morria num ponto). Ajudante `jogo_demo::carimbo`; gate `toda_cena_de_ciclo_desenha_formas_em_todo_sink_visivel` (≥ 23 cenas, 56 sinks, controlo só-pontos no teste); 7 testes passam a montar pela trava da legenda. Tutoriais 03–06, 08 |
| `77b03e6b6b` | `fotografa_cena.sh`: o guarda de frescura comparava com TODO `.rs` (recusava por `*_tests.rs`; um agente fez `touch` no binário para passar = silenciar). Agora lê o dep-info do cargo (`target/smoke/ph2d-host-desktop.d`, 5 286 deps, cobre também 56 `.wgsl`/`include_str`); `.d` ausente/velho ⇒ recusa «sem régua». Controlos (a)(b)(c) no commit |
| `ba3a04aab6` | **15 frases FALSAS de rota** (7 anúncios, 1 smoke de formas, 7 tutoriais: =113 «tudo no dispositivo» era CPU inteira; =111/=112 «só o quadradinho sai do processador» era o Duplicator; =56, =51, =137, tutoriais 02/03/04/10…) curadas com a rota MEDIDA pela porta do produto (`o_quadro` com a placa), não a sonda do plano (`sonda_rota_das_cenas_de_ciclo` lia o plano e errava =113/120/134/139). Gates `toda_frase_de_rota_tem_classe_e_cena` (suíte) e `toda_frase_de_rota_confere_com_a_rota_medida` (`placa:`); 3 mutações RED. PDFs 02,03,04,07,09,10,12 |
| `98fe9bb150` | Os 6 vermelhos do fecho: o teste da cena de foco mede a cena COM formas (6 400 peças com `publish_all`; fronteira única = o carimbo; controlo sem carimbo `is_fully_gpu`, `arranca_carimbos`); censo dos `#[ignore]`; `motion_bridge_gpu.rs` 748 → 669 (família das tomadas → `motion_bridge_gpu_tomadas.rs`); `abre_inteira` recebe a janela da cena (`scene_mapping::janela(center_split, size)`, bit-idêntico); doc partido reunido; let-chain |

## Rodadas 1–2 (15/10), comprimidas

| commit | o quê |
|---|---|
| `27c251357e` | `fotografa_cena.sh`: crases de um comentário DENTRO do `<<EOF` sem aspas executavam; o guarda lia o ficheiro GERADO (cego). Guarda novo lê o FONTE (`awk` entre `cat >` e `EOF`, recusa crase e `$(` sem barra); RED na antiga (linha 116) |
| `d0fc08d9da` | `portuguese_tokens` acusa letra acentuada, apóstrofo no lugar do acento, lista FECHADA +19 e MEDIDAS +48. Corpus `portugues_cenas_2026_10_14.txt` (195 textos PT lidos do produto): **90/195 → 184/195**; os 11 que fogem estão NOMEADOS em `ela_acusa_o_portugues_que_as_cenas_pintavam`. `keys::declared_pairs_in` deixa de emparelhar `"xx" => …` com o `eprintln!` seguinte (8 018 → 8 012, os 6 a menos falsos) |
| `4606706e9b` | Toda palavra que uma cena pinta é inglês: censo `every_word_a_scene_paints_on_the_canvas_is_english` (monta cada nível `1..=MAX_DEMO_LEVEL`, lê legendas + títulos de cartão; pisos 42 cenas / 192 legendas / 3 884 cartões). ~150 legendas + 44 cartões PT traduzidos em ~50 ficheiros. **DECISÃO:** legendas NÃO viram chaves i18n (isenção `CENAS` deliberada, 16/09); a língua guarda-se pelo censo |
| `cae382d240` | =142: legenda a −3,2 tapava a ponta do losango; borda de baixo −3,677 m em 2·CICLO, ficha a −4,2. Gate `a_legenda_da_cena_142_pousa_abaixo_da_galaxia_em_todos_os_quadros` (RED −3,2 / verde −4,2) |
| `b9280796ca`, `4ce527459b`, `f07db1723d` | PDFs 03–05, 07–14 e fotos dos tutoriais 10–14 refeitos; smoke do dono da =142 OK |
| `104a87b7c3` | Fotos pretas 9/12 → 0/12: o HOME isolado levava o cache de shaders do driver VAZIO; app compilava pipelines e `import -window` antes do 1.º quadro devolve zeros. REFUTADOS: janela errada, cache do dono, `XDG_CACHE_HOME`. Cura: cache próprio (`FOTO_CACHE_SHADERS`, omissão `~/.cache/ph2d-fotografa`), janela pelo `_NET_WM_PID`, espera conta do 1.º quadro aceso (`FOTO_PRAZO_QUADRO`=90 s), foto toda a zeros nunca entregue |
| `540919f6d7` | Gate `nenhuma_ficha_tapa_a_figura_que_explica` (`motion_demo_legend_tapa_tests.rs`) + sonda `a_tabela_das_fichas_que_tapam`. Régua = o que a bomba entrega em TODOS os tiques de 2·CICLO (quads de sprite, formas por `path_visual_bounds`, imagens, gizmo de posições). Ficha: `readout::chip_width`, `CHIP_H_PX` 16 px, câmara 10 m ⇒ 57,2 px/m. Antes ≥ 95/192 tapavam; ~26 cenas curadas. Isentas por nome (re-medidas): =95 (4 de VORTEX/SEA; 6 após a rodada 3) e =137 (saiu das isenções na rodada 3) |

## Mutações

- Rodadas 1–2: censo de língua **7/7 RED** (a M6, a contracção, sobrevivia → caso `don't go, e' red`); guarda do heredoc RED na antiga; gate da =142 RED −3,2; gate da ficha RED (ficha no centro da galáxia; desfazer a cura da =141); `fotografa_cena.sh` cache vazio / `FOTO_PRAZO_QUADRO=2`.
- Rodada 3: marcadores na placa **3/3**; frases de rota **3 RED**; `toda_cena_de_ciclo_coze_pela_placa` e `…desenha_formas…` com controlo só-pontos; guarda de frescura controlos (a)(b)(c); enquadramento: corte 1,51 com tripwire 3 % em 4 janelas.

## Aberto (ordem)

1. ⭐ **TECTO POR SUBIR (próxima onda, janela nova):** o Duplicator com `point_scale > 0` não tem kernel na placa (recusa declarada em `ph2d-node-motion-duplicator/src/kernel.rs`; doc 120 §8.3 W1(b).1 fez só `point_scale = 0`) ⇒ tudo antes do carimbo vai para a CPU, incluindo as simulações inteiras das galáxias (=132, =136, =137, =139; ~80 % do cozimento pelo cabeçalho do kernel); =120/=139 inteiras no processador; =111, =112, =119, =125, =141, =62 também. Outras causas NÃO confirmadas: =131 (fronteira sem `point_scale`), =98 (`transfer = 1`), =134 (Time Remap ⇒ tudo CPU), =51 (Boids da banda 3).
2. =116 HÍBRIDO: o Shape do carimbo na CPU custa ~0,12 ms/quadro (estático, recozido?) — por medir se se cacheia.
3. Fotos: a sessão às vezes pára o relógio ~0,3 s (a pilha da =113 e o repuxo da =119 não aparecem); notificações de arranque tapam o topo do canvas em algumas.
4. Frases de rota não conferidas: Look At no tutorial 03 («digitar um Target X tira-o de lá»), frases de capacidade «X roda na placa», os smokes de objecto =13/=16 (`PH2D_MOTION_OBJ_SMOKE`).
5. Enquadramento: o gate mede a janela da foto (1930×1040, canvas [332,96,1622,566]); o app usa a área real; o corte 1,51 depende da janela (gate em 4 janelas).
6. =138 «leitura nunca chegou» reprovou 1× a load ~15 (3/3 depois) — candidato a [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md), não promovido; idem `ph2d-physics-ecs::…the_cost_of_a_player_is_linear_in_their_number` (load 75,5, 4/4 sozinho) e o da Luz2D (`os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria`); mais os do ZERAR_132.
7. O aviso «Vector-path shapes filter as one» — o dono decide.
8. A corrida «GPU não-sonda» do repo inteiro: ~2 800 `#[ignore]`, só ~750 com motivo no formato (`placa:`/`sonda:`/`gerador:`) ⇒ ~1 850 por classificar em todas as linhas: onda própria.

FECHADOS nesta jornada: enquadramento da câmara (ex-aberto 1) · crash da =118 · marcadores parados · formas em todas as cenas de ciclo · rotas anunciadas · fichas que tapam · fotos pretas · língua das cenas · smokes do dono =142 e =87.

## Smoke ao dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && PH2D_GPU_COOK_DEMO=118 PH2D_MOTION_ROUTE_LOG=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Na tela: a cena abre INTEIRA (nenhuma linha cortada pela barra de ferramentas); seis panos de FORMAS (quadrados) — laranja no Tint, arco-íris na Color Ramp, um rasto que se lê no Trail; legendas em inglês por fora das figuras.
3. Clique num cartão do Trail: os pontinhos do gizmo andam com o rasto (não ficam parados).
4. Repita com `PH2D_GPU_COOK_DEMO=111` e `=119`: formas visíveis, a legenda diz a rota certa.

Deu errado se: o app fecha (crash) ao abrir a =118; o rasto ou os pontos do gizmo ficam parados; a cena abre cortada; falta cor; o terminal mostra `[motion-route] CPU:` onde a legenda diz «placa».

## Fecho

**1.ª rodada (`b9280796ca`):** `PH2D_SUITE=fecho` nextest-impacted 17 447/17 448 (1 = flake de física) · clippy ✓ · fmt ✓ · check warnings=deny ✓ · label-census 326/326.
**2.ª rodada (`4ce527459b`, + fmt `d973c94c7b`):** 17 450/17 451 (1 = flake de placa da Luz2D, 3/3 sozinho) · clippy ✓ · fmt ✓.
**3.ª rodada (HEAD `98fe9bb150`):** nextest-impacted `PH2D_SUITE=fecho` **17 475/17 475** · `motion-gate-placa` **63/63** · clippy na forma do `ship.sh` ✓ · fmt ✓ · check warnings=deny ✓ (sem mudança de avisos desde a rodada anterior) · `ph2d-gpu-cook` tap 5/5 e slit scan 2/2 com `--test-threads=1`.
