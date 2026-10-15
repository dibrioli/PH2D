# CONTINUAÇÃO — `line/motion-value`: a LÍNGUA das cenas (15/10) — tudo o que uma cena pinta no canvas é inglês · 2.ª rodada: nenhuma ficha tapa a figura, fotos pretas curadas

> Continua o [ABERTOS 132](HANDOFF_CONTINUACAO_line_motion_value_2026-10-14_ABERTOS_132.md) (continua válido para o que
> diz: o `Scene` com imagens no dispositivo, a sonda, as recusas). Plano, leis e recusas: [doc 132](../132_zerar_as_seis_pendencias.md).
> **Este ficheiro SUBSTITUI o ABERTOS_132 como fila viva** e herda-lhe os abertos que restam. Regras R1–R9 do
> `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, HEAD `4ce527459b` (+ o commit de docs deste
  handoff), base = `main` `0aab2c1cb6` (não andou; rebase vazio). **19 commits por integrar** medidos (`git rev-list --count main..HEAD`), 20 com o de docs deste handoff. Só integra por ordem do Enio.
- `MAX_DEMO_LEVEL = 142` (inalterado desde o ABERTOS_132). Superfície de colisão desta jornada: `ph2d-label-census`
  (`portuguese_tokens`, `keys::declared_pairs_in`, corpus `portugues_cenas_2026_10_14.txt`), ~50 ficheiros de cena de
  `ph2d-app-motion` (só texto), o censo novo `motion_demo_legend_lingua_tests.rs`, `fotografa_cena.sh`, tutoriais
  HTML/PDF 03–05 e 07–14; 2.ª rodada: `motion_demo_legend_tapa_tests.rs` (gate novo), ~26 ficheiros de cena (a ficha movida), `fotografa_cena.sh` (cache de shaders próprio), fotos/PDFs 12 e 14. Um merge pode partir: quem acrescente uma cena com legenda ou título de cartão em português
  (o censo novo reprova); quem dependa do número de pares do leitor partilhado (ph2d-i18n/src: 8 018 → 8 012); quem
  leia o ficheiro de exemplo das cenas =109/=119 (colunas agora `month,sales,time,level`); quem mova a ficha de uma cena ou mude a câmara de arranque (10 m ⇒ 57,2 px/m) — o gate `nenhuma_ficha_tapa_a_figura_que_explica` re-mede e reprova.

## O que fechou (15/10)

| commit | o quê |
|---|---|
| `27c251357e` | `fotografa_cena.sh`: o comentário do tecto do cache de shaders vivia DENTRO do heredoc `<<EOF` sem aspas — as crases executavam («[env]: comando não encontrado»). O guarda antigo lia o ficheiro GERADO, onde a substituição já correu: cego à espécie que nomeava. Guarda novo lê o FONTE do heredoc (`awk` sobre `$0` entre o `cat >` e o `EOF`), recusa crase e `$(` sem barra; RED na versão antiga (aponta a linha 116), verde na corrigida. Fecha o aberto 4 do ABERTOS_132 |
| `d0fc08d9da` | `ph2d-label-census`: `portuguese_tokens` acusa letra acentuada (sem `é`: «Bézier» está na tabela inglesa), o apóstrofo no lugar do acento (`so'`, `e'`; palavra ≤ 4, fora de aspa aberta; a contracção `don't` não abre aspa), lista FECHADA +19 e MEDIDAS +48 (de fora as que o inglês também tem: tremor, forte, vista, mar, magnitude, cantos, giro, viva, alfa, age, leva, cor). Controlo MEDIDO: `crates/ph2d-label-census/src/portugues_cenas_2026_10_14.txt` (195 textos PT que as cenas pintavam, lidos do produto) — **90/195 antes, 184/195 agora**; os 11 que fogem estão NOMEADOS no teste `ela_acusa_o_portugues_que_as_cenas_pintavam`. E o leitor partilhado `keys::declared_pairs_in`: `meio.contains("=>")` emparelhava `"xx" => Idioma::Teste,` com o `eprintln!` 3 linhas abaixo; agora só `=>`/`=> {` entre chave e texto; sobre ph2d-i18n/src 8 018 → 8 012 pares, os 6 a menos todos falsos. Teste `um_braco_sem_texto_nao_se_emparelha_com_o_literal_seguinte` |
| `4606706e9b` | toda palavra que uma cena pinta no canvas é inglês (ordem do dono 19/09: as cenas são isentas da TABELA por desenho, não da LÍNGUA). Censo `every_word_a_scene_paints_on_the_canvas_is_english` (`motion_demo_legend_lingua_tests.rs`): monta cada nível `1..=MAX_DEMO_LEVEL` pela porta `motion_demo_legend::monta` e lê legendas + títulos de cartão (`snapshot_from` + `stamp_card_params`); pisos 42 cenas com legenda / 192 legendas / 3 884 cartões. Medido antes: 94 acusados pela régua velha, ~150 legendas + 44 cartões PT de facto (=82..=142). Traduzidos em ~50 ficheiros (3 agentes por faixa, glossário comum); anúncios do terminal continuam PT, só o nome citado muda; tutoriais HTML 03·04·05·07·08·09 citam os títulos novos; o ficheiro de exemplo das cenas =109/=119 passa a `month,sales,time,level` (a cena reescreve-o sempre). Lidos à mão, depois, os 192 + 44 textos. Testes por nome: 732 da lib + 5 de `tests/it` verdes. **DECISÃO (fecha o aberto 3 do ABERTOS_132):** as legendas NÃO passam a chaves i18n — a isenção das cenas no gate da família (`CENAS`, cabeçalho de `every_word_this_family_shows_comes_from_the_string_table`, 16/09) é deliberada; a língua guarda-se pelo censo novo |
| `cae382d240` | cena `=142`: a legenda a −3,2 (herdada da =140, duas galáxias menores) TAPAVA a ponta de baixo do losango (lida na foto como «um quadrado solto»). Galáxia simulada ⇒ borda de baixo medida em 2·CICLO (481 quadros pela bomba) com a meia-diagonal do quadrado: −3,677 m; ficha a −4,2 (meia ficha 0,35 m). Gate `a_legenda_da_cena_142_pousa_abaixo_da_galaxia_em_todos_os_quadros`: RED a −3,2, verde a −4,2. Refotografada |
| `b9280796ca` | docs: PDFs 03–05 e 07–14 regenerados (`scripts/tutorial-pdf.sh`) e 15 fotos dos tutoriais 10–14 refeitas no mesmo tamanho/passo (`PH2D_ACABAMENTO_PASSO`, `PH2D_MOTION_INPUTS`, `PH2D_MOTION_FITA_*`). Diferem também no ângulo da galáxia e na hora da Timeline (o arranque varia); a =132 tem hoje outro enquadramento (grelha 55,6 px contra 47,4 — anterior a esta linha). As fotos que não mostram legenda/cartão mudado ficaram |
| `f07db1723d` | docs: smoke do dono da =142 OK |
| `104a87b7c3` | `fotografa_cena.sh`: fotos pretas **9/12 → 0/12** (+0/2 na =135). Mecanismo provado: o HOME isolado levava o cache de shaders do driver (`$HOME/.cache/nvidia`) VAZIO a cada foto; o app compilava os pipelines (fio principal em `_nv002nvvm`, visto com `eu-stack`) e o `import -window` antes do 1.º quadro devolve zeros. REFUTADOS: janela errada (cliente único, `_NET_WM_PID` = pid do app); cache do dono (aparado por outros: frio 3/4); `XDG_CACHE_HOME` (o driver não o lê: 4/4 frio); contenção só alonga. Cura: cache próprio persistente (`__GL_SHADER_DISK_CACHE_PATH`/`MESA_SHADER_CACHE_DIR` = `FOTO_CACHE_SHADERS`, omissão `~/.cache/ph2d-fotografa`); janela pelo `_NET_WM_PID`; a espera conta a partir do 1.º quadro aceso (sonda até pixel não nulo; recusa alta em `FOTO_PRAZO_QUADRO`=90 s ou app morto); foto toda a zeros nunca entregue; saída antiga apagada antes; `FOTO_GUARDA_TMP=1` guarda a sessão. Controlos: cache vazio + espera 3 ⇒ foto boa aos 12,6 s; `FOTO_PRAZO_QUADRO=2` ⇒ recusa com razão. Ponta a ponta (`4ce527459b`): 3/3 fotos à 1.ª, 1.º quadro 716–774 ms. Fecha o aberto 5 |
| `540919f6d7` | gate geral `nenhuma_ficha_tapa_a_figura_que_explica` (`crates/ph2d-app-motion/src/motion_demo_legend_tapa_tests.rs`) + sonda `#[ignore]` `a_tabela_das_fichas_que_tapam`. Régua = o que a bomba entrega ao ecrã em TODOS os tiques de 2·CICLO: quads de sprite `P+basis·(anchor+[±½]·size)`, formas por `path_visual_bounds` sob a `instance_pose` do encode, quads de imagem e o gizmo de posições dos sinks sem aparência (sem ele as =82..=97 liam zero peças). Não contam o fundo que atravessa a vista nem o que está fora do canvas visto (px [332,96,1622,566] na foto 1930×1040). Ficha: largura `readout::chip_width` (Inter embutida), altura `CHIP_H_PX` 16 px, câmara de arranque 10 m ⇒ 57,2 px/m (conferido na foto). Viés achado: ler a cada 2 tiques perdia gatilhos (=139: 3,5 m de raio contra 4,1 na foto) ⇒ todos os tiques. Antes: ≥ 95 de 192 fichas tapavam; curadas movendo a ficha em =82·83·85·86·87·88·89·90·92·93·94·96·97·111·112·113·115·117·119·134·136·137(dir)·138·139·140·141 (o porquê numa linha em cada cena). **Isentas por nome, re-medidas a cada corrida** (reprovam se surgir lugar ou se deixarem de tapar): =95 as 4 de VORTEX/SEA (plumas até ±37 m) e =137 a da esquerda (discos até ±5 m). Controlos: =142 a −3,2 RED / −4,2 verde; ficha à mão no centro da galáxia RED; desfazer a cura da =141 RED e nomeia-a. Fecha o aberto 3 |
| `4ce527459b` | docs: fotos `jogo_cena`, `jogo_entradas` (=134) e `fita_cena` (=139) refeitas no mesmo recorte; PDFs 12 e 14 |

## Mutações

- Censo de língua (`ph2d-label-census`): **7/7 RED**. A M6 (a contracção) SOBREVIVIA → curada com o caso
  `don't go, e' red`.
- Guarda do heredoc de `fotografa_cena.sh`: RED na versão antiga (linha 116), verde na corrigida.
- Gate da legenda da =142: RED a −3,2, verde a −4,2.
- Gate geral da ficha (`540919f6d7`): RED com a ficha à mão no centro da galáxia, RED desfazendo a cura da =141 (e nomeia-a).
- `fotografa_cena.sh` (`104a87b7c3`): cache vazio + espera 3 ⇒ foto boa; `FOTO_PRAZO_QUADRO=2` ⇒ recusa com razão.

## Aberto (ordem)

1. **A CÂMARA NÃO ENQUADRA a cena** (novo): 46 fichas e as figuras das mesmas linhas ficam fora do canvas visto (por baixo da barra de ferramentas) nas =82..=98, =110, =118, =119 — a sonda `a_tabela_das_fichas_que_tapam` marca `FORA-DA-VISTA`; a foto da =82 mostra as linhas 1–2 cortadas no topo. É a câmara de arranque (10 m de altura) contra cenas mais altas, não a ficha. E a ficha da =139 encosta à borda esquerda do canvas visto sem margem (`fita_cena.png`). Próxima onda: janela nova.
2. O detalhe do aviso «Vector-path shapes filter as one» — o dono decide (dica ou tutorial).
3. A corrida «GPU não-sonda» do repo inteiro: medido 15/10 — ~2 800 `#[ignore]` no repo, só ~750 com motivo no formato do
   Motion (`placa:`/`sonda:`/`gerador:`), 1 496 de texto livre + 352 sem motivo ⇒ classificar ~1 850 testes nas crates de
   todas as linhas: onda própria, não desta linha.
4. Flakes de outras linhas: `ph2d-physics-ecs::measure_player_budget::the_cost_of_a_player_is_linear_in_their_number`
   reprovou no fecho a loadavg 75,5 («10x os players tem de custar ~7x, nao 12.35x»), 4/4 sozinho — candidato à família de
   [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md), NÃO promovido; mais os do ZERAR_132.

FECHADOS: ~~smoke do dono da =142~~ OK do dono (`f07db1723d`; log `[formas] 513 copias de 2 geometrias pela PLACA (do dispositivo)` e `[motion-route] device: HIBRIDO`) · ~~nenhum gate geral pergunta se uma legenda tapa a figura~~ FECHADO `540919f6d7` · ~~fotos pretas do `fotografa_cena.sh`~~ FECHADO `104a87b7c3` · do ABERTOS_132: aberto 3 (legendas em PT → decisão acima) e aberto 4 (crases no heredoc).

## Smoke ao dono (cena =142, foto `cena142d.png`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && PH2D_GPU_COOK_DEMO=142 PH2D_MOTION_ROUTE_LOG=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Na tela: quadrados laranja (imagens) com uma estrela azul INTEIRA por cima de cada um; a galáxia simulada ao fundo e a
   legenda em INGLÊS, ABAIXO dela, sem tapar nenhuma ponta do losango.
3. No cartão **Output**, depois do **Combine**, mude **Blend With Scene** ↔ **Copies**: em Copies a estrela clareia e não tapa.
4. O terminal diz `device:` (HIBRIDO é o certo) e `[formas] 513 copias de 2 geometrias pela PLACA (do dispositivo)`.

Deu errado se: a legenda está em português ou tapa a figura; o laranja tapa as estrelas; faltam quadrados ou estrelas; o
terminal mostra `[motion-route] CPU:`.

## Fecho

**1.ª rodada (HEAD `b9280796ca`):** `PH2D_SUITE=fecho` nextest-impacted **17 447/17 448** (o 1 = o flake de física do aberto 4, 4/4 sozinho) · clippy na forma
do `ship.sh` ✓ · fmt ✓ · check warnings=deny ✓ · gates de i18n/label-census das 9 crates consumidoras 326/326 ·
`motion-gate-placa` NÃO re-corrido (nenhum código de placa mudou desde o fecho de 14/10, 59/59).
Perfil (`agent-loop-profile.sh`): paralelismo 1,16/passo ✗ (alvo 1,5) · test:check 2,7× ✗ · contexto 279 mil/passo ✗ ·
edições sem conferir 1,0 % ✓ · respostas/sessão 134 ✓.

**2.ª rodada (`104a87b7c3`..`4ce527459b`):** verificada por testes POR NOME (447 + os da 1.ª rodada) e **NÃO** pelo
`nextest-impacted` de fecho — a suíte tem cadência; **o integrador a corre** (`PH2D_SUITE=fecho`). A linha tem 19 commits sobre o `main` (20 com o de docs deste handoff).
