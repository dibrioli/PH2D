# Handoff — `line/3DModeling`: a OCLUSÃO NO TEMPO, e o ZOOM que não era tempo real (2026-09-30)

> Ordem do dono (2026-09-29): *«o render ainda não está em tempo real. lembre-se: somos uma game
> engine»*, e depois *«siga»*. É o passo `3` da [`F1`](../../Render3d/14_a_ordem_de_superar.md)
> (*«o quadro anterior é informação; hoje é deitado fora»*). ⚠️ **Conte o DELTA: `PROJECT_SCHEMA` 0,
> registos 0, `FIELD_DOC_VERSION` 0, zero contrato, zero ADR, zero pacote externo.** Toca
> `ph2d-field-render` (a lei do alcance da oclusão) — ⚠️ **muda a imagem assente** em `5` de `22`
> cenas no enquadramento de fábrica (§2).

## §1 — O que existe

[`ph2d_field_gpu::ceu_tempo`](../../../crates/ph2d-field-gpu/src/ceu_tempo.rs): as SOMAS dos cones da
oclusão (`Σ c·vis`, `Σ c`) guardadas NO MUNDO, numa tabela de hash de células de `1`–`2` píxeis
(`16` baldes de normal), à maneira da SHaRC. O quadro ASSENTE grava a MÉDIA dos pixels de cada célula
(`ceu_tempo_zera` + `ceu_tempo_grava`); o quadro de MOVIMENTO lê, e as células novas reclamam as
fatias em falta numa lista de trabalho (`pede` → `args` → despacho indirecto `marcha` → `le`). Uma
célula vazia herda das vizinhas só se DUAS concordarem (`CONCORDANCIA`). Uma edição da peça muda a
[`ChaveDoCeu`] e o quadro desce a «gravar» (`trace_marcha_com.rs`). O produto liga-a por omissão
(`Sonda::default().ceu_no_tempo`, com `PH2D_FIELD_CEU_TEMPO` a bissectar).

⛔ **Recusada e medida:** o histórico no ECRÃ com reprojecção — `9 212` canais acima de `8` níveis
depois de UM quadro de meio grau, e o erro ACUMULA (doc do módulo).

## §2 — ⭐⭐⭐⭐ O ZOOM não era tempo real, e o gate não o podia ver

O gate de antes só GIRAVA à distância fixa. Medido com o gesto de aproximar: **cada quadro
recomeçava a tabela** e pagava a oclusão inteira. Três causas, as três curadas:

1. **O alcance da oclusão vinha da câmara** (`OCCLUSION_REACH × half_extent`, escrito em NOVE
   sítios) e entrava na chave. ⇒ porta única `ph2d_field_render::occlusion_reach(raio_da_bola)` =
   o diâmetro da bola, que com a cerca da bola é a **visibilidade do céu exacta** (fora da bola não
   há peça). O estado da arte (Workbench/Eevee, SHaRC) guarda o alcance em MUNDO. Medido no quadro
   assente das `22` cenas vivas contra a lei de antes: afastado (`half_extent 1,6`) `0` cenas mudam
   mais de `8` níveis; no enquadramento de fábrica (`0,8`) **`5` de `22`, pior `13`**, a escurecer no
   fundo das fendas; aproximado (`0,4`) as `22`, pior `59` — ⇒ *a lei de antes clareava as sombras
   de contacto ao aproximar.* A `ground_sky` deixou de receber a câmara (a assinatura é a prova).
2. **Os limiares de pixel (`hit_eps`, `normal_eps`) estavam na chave.** Saíram: deslocam a origem
   do cone na ordem de um pixel, o tamanho da própria célula (e nesta faixa de zoom são constantes:
   `min(2e-4, pixel/4)`).
3. **A tabela ENCHIA.** Era `2²¹` fixos; a `1920×1080` um pixel sem entrada marca UMA fatia (`6`
   cones) só para si — o SAL nas fendas, e o zoom põe um nível inteiro de células ao lado do de
   antes. ⇒ [`ENTRADAS_POR_PIXEL`] `= 4` com o tecto do dispositivo ([`entradas_para`]: o maior
   buffer que a placa deixa ligar), e o tamanho entra na chave (mudar de janela recomeça pela mesma
   porta que uma edição). Varredura (canais acima de `8` contra a exacta, `12` quadros):

| entradas | por pixel | girar nó · rosca | aproximar nó · rosca | afastar nó · rosca |
|---:|---:|---|---|---|
| `2²¹` | `1` | `64`–`131` · `480`–`548` | `1 604`–`1 744` · `1 251`–`1 361` | `55`–`83` · `144`–`187` |
| `2²²` | `2` | `11`–`32` · `181`–`239` | `326`–`411` · `464`–`516` | `1`–`34` · `57`–`75` |
| **`2²³`** | **`4`** | `11`–`32` · `208`–`214` | `240`–`267` · `417`–`445` | `1` · `51`–`69` |
| `2²⁴` | `8` | `11`–`32` · `183`–`192` | `213`–`240` · `346`–`373` | `1` · `51` |

⚠️ **Recurso: MEMÓRIA** — `80 B` por pixel, `166 MB` a `1080p`, `664 MB` a 4K se a placa deixar
ligar; no piso da `wgpu` (`128 MiB`) a `1080p` fica com `6,7 M` (entre `2²²` e `2²³`).

## §3 — O relógio

`diag_o_ceu_no_tempo` (release, `1920×1080`, mínimo das medianas de `3` corridas, `0` reinícios;
carga `8`–`12` com a placa ociosa `86`–`93 %`; a coluna SEM CÉU foi medida a carga `42`–`48` e é
INDICATIVA — `PH2D_SONDA_ZOOM=0.97|1.03` troca o giro pelo zoom):

| cena | girar `3°` antes → agora | aproximar `3 %` | afastar `3 %` | sem céu (girar · aproximar) |
|---|---|---|---|---|
| nó (`28`) | `28,5 → 18,3` | `66,3 → 29,5` | `27,2 → 16,0` | `12,4` · `21,0` |
| `5` · `11` · `30` | `8,0`–`9,2 → 5,8`–`7,2` | `12,3`–`15,7 → 8,1`–`10,6` | `7,8`–`9,8 → 6,6`–`7,3` | `4,4` · `5,9` (a `30`) |

⛔⛔ **O critério de paragem da `F1` disparou para o nó a aproximar, e é por isso que a próxima wave
não é a oclusão:** com o céu DE GRAÇA o nó a aproximar custa `21,0 ms` — o problema muda de classe
(a marcha da peça: ela cresce no ecrã e cada pixel marcha o toro inteiro). A girar ele fica a
`18,3` contra `12,4` sem céu: a oclusão em movimento ainda custa `~5 ms` ali.

## §4 — Os gates e a prova

[`preview_device_w9_ceu_tempo_tests.rs`](../../../crates/ph2d-app-field3d/src/preview_device_w9_ceu_tempo_tests.rs):
`parado_o_movimento_le_a_oclusao_do_assente` (a barra é o quadro de passo `2` que a cache
substitui) · **`os_gestos_da_camara_ficam_perto_da_exacta`** (os três gestos × duas cenas, `0`
reinícios, e as `BARRAS` do vale — tabela no doc delas) · `uma_edicao_da_peca_nao_herda_nada` ·
`o_produto_liga_a_oclusao_no_tempo` · `a_tabela_segue_a_vista_e_o_tecto_do_dispositivo`.

⭐ **Prova de mutação, duas rondas.** A 1.ª (a lei de antes) deu `5 de 7`: sobreviviam a
CONCORDÂNCIA (o sal da tabela cheia escondia-a) e o **recurso ao nível VIZINHO** da célula «a encher»,
que era **inerte por construção** — o `pede` reclama todas as fatias em falta no MESMO quadro, antes
do `le` — e **saiu**. A 2.ª, sobre a lei nova: **`10 de 10` sangram** (os dois `atomicStore` do
assente · o `zera` fora · `quantas < 1` · concordância `10` · `herda` sempre verdadeiro · a tabela
de volta a `2²¹` · o alcance da câmara no dispositivo (apanhado pelos `0` reinícios) · o alcance da
câmara na CPU (apanhado pela paridade do G-buffer) · o chão com `2` amostras). ⚠️ Esta última foi
apanhada pelo gate de AJUSTE do chão, não pelo dos anéis.

⭐ **O gate dos anéis do chão tinha a barra no CONTROLO** (`lei × 4 ≤ 48 cones`): com o alcance
novo a lei ficou onde estava (`212 → 210` extremos) e os anéis dos `48` cones caíram (`904 → 654`).
Reescrito contra a referência CONVERGIDA (`1 024` cones: `394 → 264`) — a lei fica abaixo dela nos
dois alcances — com o controlo a exigir que os `48` cones tenham mais do DOBRO.

**Portão:** as três crates com GPU `710` de `712` — os dois vermelhos curados (o censo
`os_tres_caminhos_de_um_quadro_leem_a_mesma_porta` lia só o `gpu_frame.rs` depois do corte que
mudou a `Sonda` para [`gpu_frame_sonda.rs`](../../../crates/ph2d-app-field3d/src/gpu_frame_sonda.rs);
e um SIGSEGV) · clippy `-D warnings` zero · `fmt` · censos da árvore COMBINADA `127/127` · as `10`
vassouras limpas sobre os `26` ficheiros.

## §5 — ⚠️ O que uma leitura rápida entende ao contrário

1. **O SIGSEGV de `com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` NÃO é deste diff**
   — o teste imprime `test result: ok` e o processo morre DEPOIS, na desmontagem do driver (`1` em
   `3` corridas sozinho, a mesma assinatura das sondas desta wave). E o gate está **VERDE**:
   `21` de `22` cenas nítidas em movimento, contra os `10` de `22` com que a linha fechou em 20/09.
2. **A imagem ASSENTE mudou de propósito** (§2.1) — um golden que falhe no fundo de uma fenda à
   escala de fábrica é a lei nova, e nenhum dos `712` o fez.
3. **`mede_o_preco_de_uma_aresta_de_perfil` estourou o prazo de `180 s`** numa corrida: é uma sonda
   `#[ignore = "medição"]` num ficheiro que esta wave não tocou.
4. A sonda de relógio ganhou o botão `Sonda::sem_ceu` (só sondas o ligam) e a coluna SEM CÉU.

## §7 — ⭐⭐⭐⭐ *«a luz indirecta ainda desliga e a resolução ainda cai»* (report do dono, foto)

⚠️ **Conte o DELTA: `PROJECT_SCHEMA` 0, registos 0, `FIELD_DOC_VERSION` 0, zero contrato, zero ADR,
zero pacote externo.** Ficheiros novos: [`cronometro.rs`](../../../crates/ph2d-field-gpu/src/cronometro.rs),
[`ceu_tempo_wgsl.rs`](../../../crates/ph2d-field-gpu/src/ceu_tempo_wgsl.rs) (o WGSL saiu do
`ceu_tempo.rs` por tecto de LOC), [`preview_medicoes_tests.rs`](../../../crates/ph2d-app-field3d/src/preview_medicoes_tests.rs).

### §7.1 — A LUZ: a lei W73 mudou

O `assente` passa a governar SÓ a oclusão do céu: o **ricochete** e a **cor do chão** correm em
todo quadro do dispositivo (`gpu_frame.rs`). ⚠️ Com uma peça nova os pipelines do ricochete ainda
não estão compilados — [`PaintSetup::ricochete_sem_esperar`] (= `!assente`) faz esse quadro de
movimento ir sem ricochete em vez de esperar o compilador (`1,3`–`2,8 s` de imagem parada), e o
assente compila. Gate `render_bounce_gpu_tests::o_quadro_de_movimento_leva_o_ricochete` (o
movimento é o assente AO BIT; sem ricochete difere `7` níveis — o controlo). ⚠️ A metade «peça
nunca vista» usa um **traçador PRÓPRIO**: na suíte inteira outro teste já tinha compilado a cena
`2` no partilhado e o gate reprovava sobre produto certo (`left: 1`), verde sozinho.

### §7.2 — O INSTRUMENTO: o relógio por passe NA PLACA

[`ph2d_field_gpu::cronometro`] (`TIMESTAMP_QUERY`, só com `PH2D_GPU_CRONOMETRO=1` — o produto
continua a pedir `Features::empty()`): cada passe marca um rótulo, os troços de CPU (`cpu-*`,
`espera-*`) e **contadores do dispositivo** (`n-*`: fatias pedidas, itens de lâmpada, transbordo,
desfechos da herança) saem no relatório da `diag_o_ceu_no_tempo`, com o MÍNIMO por passe entre
repetições (a placa é partilhada com o ecrã: `±20 %` entre voltas iguais). ⭐ *A escada de ablação
media cada parte e o que ela arrastava; o relógio por passe responde quem come o quadro.*

Primeira leitura (nó, giro `3°`, `1920×1080`): placa `~15 ms` — `luz 4,7` · `ceu-marcha 4,0` ·
`centro 2,7` · `bordas-marcha 1,8` · `pinta 0,6` · o resto `< 0,5`; mais `~4,6 ms` de CPU e duas
idas-e-voltas. ⚠️ **E a `luz` NÃO era a sombra da peça** (`~1 ms`, ablação): era a sombra das
lâmpadas no CHÃO (`~3,5 ms`) — a placa mostrou-o, a intuição não.

### §7.3 — As curas, cada uma com o número

| cura | onde | medida |
|---|---|---|
| **envio único**: a pintura continua no encoder da marcha; o passe da borda lê a contagem NA PLACA (`min(tecto, conta)`) e despacha pelo tecto | `trace_marcha_com.rs`, `paint.rs`, `paint_wgsl_sondas.rs` | giro `14,3 → 13,9 ms`; a contagem volta depois da imagem |
| **a sombra e o céu do CHÃO na tabela do mundo** (independentes da câmara), com a célula do tamanho da PEGADA do pixel no chão | `ceu_tempo_wgsl.rs` (`Alvo`, `chave_do_alvo`, `nivel_do_chao`), `trace_wgsl.rs` | passe da luz `5,3 → 1,4 ms` no giro |
| **o chão PERTO da peça marcha por pixel** (`CHAO_PERTO = 32` pegadas, pelo campo): a penumbra ali é mais fina do que a célula | `chao_perto_da_peca` | rosca a afastar `207`–`249 → 30`–`54` canais `> 8`; `+0,4 ms` (`8`/`16`/`32` medidos) |
| **a impressão da chave é um 2.º hash** (era `mistura(h)` do MESMO `h` de `32` bits ⇒ células fundidas, `N²/2³³` pares) | `chave_no_nivel` | pontos claros de `77` níveis → `0` |
| **cópia entre NÍVEIS** (céu e lâmpadas do chão), só sem vizinhas no nível; uma cópia não é fonte de outra (`COPIA_*`) | `herda_do_nivel`, `lampadas_do_nivel` | céu no zoom `8,9 → 3,1 ms`; as «escamas» do zoom (cópia de cópia) curadas |
| **fatias por quadro**: as `4` primeiras (de `8`) só nos `QUADROS_COM_TECTO = 2` depois do assente, o resto completa no seguinte | `FATIAS_POR_QUADRO`, `tab.por_quadro` | corta o PICO do 1.º quadro |
| **procurar antes de reclamar** + marca de leitura só quando muda | `ceu_tempo_pede` | `pede` `0,78 → 0,69 ms` |
| **o divisor decide pela mais BARATA das duas últimas medições** | [`preview::Medicoes`] | um pico isolado (o 1.º quadro de um gesto) não baixa a resolução; dois seguidos baixam |

### §7.4 — ⛔ Recusas MEDIDAS (não reconstrua)

- **A sombra das lâmpadas da PEÇA na tabela**: `luz 5,3 → 2,2 ms`, e a penumbra de um tubo sobre
  outro saía em DEGRAUS do tamanho da célula (nó a aproximar `2 273` canais `> 8`, pior `123`,
  contra `175`/`39`). A oclusão aguenta a célula porque é suave; uma sombra não.
- **A célula de pegada na PEÇA** (só no chão ficou): `2 → 11 752` canais `> 8` no nó.
- **O tecto de 4 fatias PERMANENTE** (até ao assente): `~2 ms` a menos num giro e um viés largo
  nos sulcos da rosca (`9 024` canais `> 8` contra a barra `260`), que saltava ao parar.
- **Um orçamento de fatias CONTADAS por quadro**: contar fatias não mede custo (rosca `364 884`
  fatias por `1,5 ms`, nó `54 846` por `5,7 ms`).
- **As 26 vizinhas** (cubo `3×3×3`) na herança: 1.º quadro `54 846 → 39 559` fatias e o erro
  espalhado (`252 → 1 573` canais `> 8`).
- **Medir o território novo no nível de CIMA**: `16,1 → 15,3 ms` e o erro a explodir (`36 → 588`,
  pior `18 → 129`).

### §7.5 — O relógio de agora

`diag_o_ceu_no_tempo` (release, `1920×1080`, mín das medianas de `3`; ⚠️ carga `40`–`57`, a placa
`47`–`91 %` ociosa — INDICATIVO; a sequência quadro a quadro está na sonda):

| cena | girar `3°` | aproximar `3 %` | afastar `3 %` |
|---|---|---|---|
| nó (`28`) | `18,3 → 16,9` | `29,5 → 19,4` | `16,0 → 12,4` |
| rosca `29` · `5` · `11` · `30` | `6,5`–`7,6` | `7,6`–`8,3` | `5,9`–`6,9` |

⚠️ **O nó ainda não cabe em TODO quadro**: a girar os quadros `1`–`8` de um gesto custam `16`–`19`
e os seguintes `13`–`15`; a aproximar o custo sobe com a peça a encher o ecrã (`17 → 21`). O
divisor com histerese segura a resolução no pico isolado e baixa-a se o custo FICAR acima.

### §7.6 — Gates e prova

- `os_gestos_da_camara_ficam_perto_da_exacta`: **barras re-medidas pelo vale** com a lei nova (tabela
  no doc de [`BARRAS`]); ⛔ a **concordância** deixou de ser visível nestes gestos (a mutação que a
  desliga SOBREVIVE — escrito no doc em vez de uma barra que fingisse vê-la); a asserção dos pontos
  claros passou a contar só os NOVOS (claros e `> 8` níveis acima da exacta no mesmo pixel) — a
  contagem crua media o limiar (`129` contra `128` com o vizinho a `116`).
- `preview_medicoes_tests` (`4`): pico isolado · dois seguidos · a 1.ª medição sozinha · por pixel.
- Os censos de texto acompanharam a refactoração (`SABIDAS` ganhou `chao_perto_da_peca`, com a
  entrada na tabela; a agulha das duas passagens moles segue o laço rotulado).
- **Mutação `6 de 7`**: chão-perto desligado · borda pelo tecto (`12` paridades) · `le` sem as
  lâmpadas do chão (`37 512`) · `le` sem o céu do chão · cópia como fonte · o divisor a ler só a
  última — sangram. ⚠️ **NOMEADA**: tirar o tecto das fatias por quadro torna a imagem MELHOR; é uma
  lei de CUSTO, e só um relógio a veria (um gate de relógio é da família das flakes).
- ⚠️ O SIGSEGV na SAÍDA do processo da sonda (depois de `test result: ok`, `3` em `~45` corridas,
  `0` em `6` a reproduzir) é o mesmo da §5.1 — desmontagem do driver.

## §8 — ⭐⭐⭐⭐ *«quase perfeito! se aproximar do objeto ainda fica lento e perde resolução»* (report do dono, 2026-10-01)

⚠️ **Conte o DELTA: `PROJECT_SCHEMA` 0, registos 0, `FIELD_DOC_VERSION` 0, zero contrato, zero ADR,
zero pacote externo.** Ficheiros novos: [`amplia.rs`](../../../crates/ph2d-field-gpu/src/amplia.rs)
(+ `amplia_tests.rs`), [`amplia_gpu_tests.rs`](../../../crates/ph2d-app-field3d/src/amplia_gpu_tests.rs),
[`chao_sem_camara.rs`](../../../crates/ph2d-field-render/src/tests/chao_sem_camara.rs).

### §8.1 — LENTO: a câmara estava em TRÊS chaves de caches do MUNDO

Pelo relógio da placa (§7.2), o nó a `40 %` do enquadramento custava **`285 ms`** por quadro de zoom:
`assa-sondas` `162,7 ms` + `~88 ms` de CPU a re-assar o chão. As duas assaduras são grelhas no
MUNDO, e as chaves delas levavam a precisão de acerto da CÂMARA (`pixel/4`), que de perto muda a
cada quadro de zoom. ⇒ três curas, a mesma lei:

| chave | o que saiu | e a assadura passa a usar |
|---|---|---|
| `ChaveDoCeu` (W9, `abe1a7838`) | `hit_eps`/`normal_eps` | — (o céu é por pixel e temporal) |
| `ChaveDasSondas` (`sondas_na_placa.rs`) | `hit_eps`/`normal_eps` | a precisão do MUNDO, nos DOIS lados: `paint::Alvos::setup_mundo` (um 2.º uniforme só para o passe `assa_sondas`) e `probes::bake_probes` |
| `ChaveDoChao` (`gpu_frame_chao.rs`) | o `Sharpness` inteiro | `Sharpness::do_mundo()` dentro de `ground_bounce::assa` |

⛔⛔ **A metade que tira só a chave é um defeito novo**: guardadas sem a câmara e assadas com ela, a
MESMA vista saía diferente conforme o zoom em que o armazém nasceu (medido na placa: `pior 2` níveis
— a mutação `M6`). ⚠️ **O preço é `≤ 0,91` byte no zoom mais apertado** (a tabela de 2026-09-21,
mudada para o doc do `chao_sem_camara`): a precisão do mundo é a de todo enquadramento de fábrica.
Depois: zoom a `0,4` `285 → 33,6 ms`.

### §8.2 — PERDE RESOLUÇÃO: a escala era em três degraus e esticada em bilinear

Mesmo sem céu, de perto o nó a ecrã cheio pede `23`–`35 ms`: não cabe nativo. ⇒ **resolução
dinâmica contínua + ampliação na placa**:

- [`preview::preview_size`]: escala contínua em passos de `1/32` (`ESCALA_PASSOS`), piso `1/3`, com
  **histerese** (subir só com `SUBIR_COM_FOLGA = 0,85` do orçamento) — sem ela o laço escorregava
  degraus (o gate `the_loop_settles_inside_the_budget`, cujo modelo de custo passou a interpolar
  linearmente em píxeis entre as linhas medidas).
- [`ph2d_field_gpu::amplia`]: **Catmull-Rom 16 amostras com ANTI-ANEL** (cada canal preso ao
  mín/máx dos QUATRO texels mais perto) em bytes sRGB, os quatro canais; gémeo na CPU
  (`amplia_cpu`). A imagem sai do dispositivo à área CHEIA e o ecrã desenha-a `1:1`.
  ⛔ Sem a trava, uma aresta `64|192` toca abaixo de `63,5` (o controlo do gate do halo).
- A medição do custo usa os píxeis TRAÇADOS (`Ready::tracado_px`), não os entregues.
- ⭐ **E o MATCAP também** (o modo de omissão do modelador): `MatcapSetup::entrega` e a ampliação no
  mesmo encoder do `matcap::pinta`. O tamanho que a thread diz ao ecrã sai de UMA porta com dois
  chamadores (`smoke_draw_thread::tamanho_entregue`).

### §8.3 — ⛔ E a resolução dinâmica RECOMEÇAVA a tabela do céu

O tamanho traçado entrava na `ChaveDoCeu` (dimensiona a tabela) ⇒ cada passo da escala recomeçava-a:
no laço do produto `4`–`8` recomeços em `16` quadros e picos de `40`–`50 ms`. ⇒ a tabela é
dimensionada pela área ENTREGUE (`tracado.max(entrega)` em `trace_marcha_com.rs`). Depois: **`0`
recomeços** em todas as células.

### §8.4 — O relógio do laço do produto (`PH2D_SONDA_LACO=1`, `--release`, `1920×1080`, carga `~2–3`)

| perto | gesto | larguras do laço | ms por quadro (depois do 3.º) |
|---|---|---|---|
| `1,0` | girar | `1920 → 1620 → 1920` | `12,5`–`15,7` |
| `1,0` | aproximar `3 %` | `1920 → 1560` | `16,2`–`18,1` |
| `0,4` | girar | `1140 → 900 → 1140` | `11,8`–`15,0` |
| `0,4` | aproximar | `1260 → 1080 → 1140` | `12,6`–`15,5` |
| `0,25` | girar | `1080 → 780 → 960` | `11,4`–`14,8` (cauda `17,9`–`25,8`) |
| `0,25` | aproximar | `1200 → 960 → 1020` | `12,0`–`14,8` |

⚠️ Os três primeiros quadros de um gesto ainda custam `23`–`52 ms` (o laço parte da largura cheia sem
medição); no app a `Medicoes` herda a do gesto anterior.

### §8.5 — ⛔ Gates com a PREMISSA MORTA (reescritos com a morte à vista)

- `chao_ricochete::a_tolerancia_de_acerto_entra_na_chave_da_cache_do_chao` — **apagado**, com
  lápide e a tabela dele levada para o `chao_sem_camara`.
- `gpu_frame_tests::a_chave_do_chao_leva_a_precisao_inteira` → `…_nao_leva_a_camara_porque_a_assadura_nao_a_le`.
- `preview_device_w9_chao_tests` (metade do zoom): *faltar* → **acertar**.
- ⛔⛔ **E um gate media a ORDEM**: `parado_o_movimento_le_a_oclusao_do_assente` lia `35 169` canais
  `> 8` SOZINHO e `27` na suíte — num processo novo os quadros de movimento saltam o ricochete até o
  assente o compilar (§7.1), e a fixtura comparava quadros com e sem ele. Aquecido: `0` acima de `8`,
  pior `2` (contra `24`/`13` do quadro que ele substitui). Defeito do arnês, anterior a esta wave.

### §8.6 — Gates novos e prova

- `amplia_tests` (`3`, CPU): identidade · sem halo junto da aresta (com o controlo da bicúbica crua) · rampa.
- `amplia_gpu_tests::a_placa_amplia_o_quadro_pequeno_pela_lei_da_cpu`: tamanho, `≤ 1` nível contra a
  CPU, determinismo como controlo.
- `chao_sem_camara` (`2`): o campo do chão e as sondas IGUAIS AO BIT a `1,6` e `0,05`.
- `preview_device_w9_sondas_tests::aproximar_nao_reassa_as_sondas_nem_muda_a_imagem`: `0` assaduras
  e igualdade AO BIT contra assar de fresco ali (a mesma rotação ⇒ os mesmos bits).
- `preview_device_w9_ceu_tempo_tests::a_resolucao_dinamica_nao_recomeca_a_tabela`: `≤ 1` recomeço
  com a área fixa; CONTROLO sem ela: um por tamanho.
- `amplia_gpu_tests::a_placa_amplia_o_matcap_pela_lei_da_cpu` (o irmão do matcap).
- `preview_medicoes_tests::subir_de_resolucao_pede_folga_e_descer_nao` — a LEI da histerese.
- `render_bounce_seam_tests::os_dois_caminhos_da_placa_pedem_e_entregam_a_area` — a costura da
  thread (textual: ela não é alcançável de um teste).

### §8.7 — Prova de mutação (`muta_perto.py`, pré-voo `14/14` âncoras, `1` cada)

**`12` de `13` sangram + `1` NOMEADA + o CONTROLO sobrevive**: anti-anel na CPU (`M1`) e na placa
(`M2`) · a entrega ignorada (`M3`) · o chão (`M4`) e as sondas da CPU (`M5`) com a precisão da câmara
· as sondas da placa com o uniforme do quadro (`M6`, `pior 2`) · a tolerância de volta à chave das
sondas (`M7`) · sem histerese (`M8`) · a tabela pelo tamanho traçado (`M9`) · o matcap a ignorar a
entrega (`M11`) · a thread a devolver o tamanho traçado (`M12`) · o matcap sem pedir a área (`M13`).

⛔⛔ **Duas SOBREVIVERAM na 1.ª corrida, e as duas eram buracos de régua:** a histerese (`M8`) era
INVISÍVEL ao `the_loop_settles_inside_the_budget` — o modelo de custo dele interpola em píxeis e o
laço converge com ou sem ela ⇒ gate da LEI; e a costura da thread (`M12`) casava a agulha `p.cheio`
na CONDIÇÃO ⇒ os braços passam a ser lidos linha a linha. ⚠️ **NOMEADA (`M10`)**: o chão com a
precisão da câmara visto pelo gate de CONTAGEM do dispositivo sobrevive por construção — a cache
acerta pela CHAVE, que não lê a precisão; quem vê a mutação é o gate de VALOR (`M4`).

### §8.8 — ⚠️ Um aviso para quem corre a suíte desta família

O `tests/it` da `ph2d-field-render` só é verde **um processo por teste** (`cargo nextest`, `144/144`):
com `cargo test` em threads, `7` gates de CONTAGEM (fitas compiladas, acertos de cache, amostras de
marcha) somam os contadores globais uns dos outros e reprovam — pré-existente, nenhum toca no que
esta wave mexeu. E o `scripts/cargo-test-narrow.sh` parava no 1.º binário vermelho (a flake
`an_abandoned_march_returns_nothing_and_returns_fast`, `3/3` verde sozinha) e nunca chegava ao `it`.

## §9 — ⭐⭐⭐⭐ *«Ruído ao rotacionar a view»* (report do dono, 3 fotos, 2026-10-01)

Pontos CLAROS do tamanho de uma célula nas faces de baixo (escuras) dos tubos, a aparecer só
depois de girar um bocado. Cena `=28`, Render, de perto.

### §9.1 — O mecanismo (medido, não lido)

A tabela do céu no tempo (`ceu_tempo`) tem `8` posições de sonda linear por chave. Uma célula que
SAIU de vista guardava o lugar dela durante `VELHA = 120` quadros (`lida` = o último quadro em que
alguém a leu). Num giro LONGO a vista varre células novas mais depressa do que as velhas expiram ⇒
a tabela ENCHE, e um pixel sem lugar caía num ramo de recurso que marchava **`1` fatia** (`6` dos `48`
cones) — uma estimativa da oclusão feita com um oitavo da hemisfera, que numa face virada para
baixo tanto pode acertar o chão como o céu ⇒ **um ponto claro do tamanho da célula**.

A régua que o viu é a sonda `device_probes_w9_ceu_tempo` com **pontos = píxeis com luminância acima
da exacta `+16`** (⛔ a 1.ª régua contava só máximos locais estritos sobre os `8` vizinhos e lia
`0`–`4` sobre milhares de pontos: um ponto de `2` píxeis não é máximo estrito de nenhum vizinho).
Pontos contra o número de quadros girados (`3°` cada):

| quadros | 4 | 12 | 24 | 40 | 80 |
|---|---|---|---|---|---|
| pontos | 16 | 29 | 363 | 4 106 | 23 300 |

⇒ **ele cresce com o ângulo varrido**, que é a assinatura de uma tabela a encher e não de uma lei.
⚠️ **O gate que existia (`a_tabela_segue_a_vista…`) gira `12` quadros** — exactamente onde o
fenómeno ainda não existe (`29`). *Uma fixtura que pára antes da fronteira mede o planalto.*

### §9.2 — Hipóteses REFUTADAS antes da certa (a `40` quadros, linha de base `4 106`)

| hipótese | pontos |
|---|---|
| uma entrada recém-reivindicada podia ser roubada | 4 056 |
| sem herança da célula vizinha | 4 083 |
| baldes de normal mais finos | 4 290 |

Nenhuma mexe — a contagem de **`n-sem-lugar`** (contador novo, nono, do `ceu_tempo`) é que acusa.

### §9.3 — A cura: três metades

1. **`VELHA` `120 → 16`.** Um lugar não lido há `16` quadros é de uma célula fora de vista; com `120`
   a tabela fica cheia de mortos. A `80` quadros:

   | `VELHA` | sem-lugar | pontos | canais `> 8` |
   |---|---|---|---|
   | 120 | 15 362 | 23 298 | 150 989 |
   | **16** | **238** | **54** | **1 007** |
   | 4 | 25 | 100 | 839 |

   As fatias marchadas por quadro **não sobem** (o custo fica). `8` e `4` dão contagens parecidas mas
   pior extremo (`max 78` contra `42`) ⇒ fica `16`. ⚠️ A nota antiga do `ENTRADAS_POR_PIXEL` dizia
   *«`VELHA=16` não ajuda»* — medida sobre gestos de `12` quadros, onde nada enche; corrigida à vista.
   ⛔ **Sondar `32` posições em vez de `8` foi medido e RECUSADO** (`653` sem-lugar, `780` pontos).
2. **Sem lugar ⇒ marcha os `48` cones (`TODAS = 0xfd`), nunca `1` fatia.** O pixel sem lugar paga
   a estimativa inteira no próprio quadro (`ceu_por_cones`), que é a mesma lei da referência ⇒ ele
   pode ficar MAIS CARO, nunca ERRADO. A `80` quadros com as três metades: **`39` pontos**, `488`
   canais `> 8`, máximo `42`.
3. **`ENTRADAS_MAX = 2²⁴ − 2`** (defeito LATENTE achado a medir a §9.3.1). A lista de trabalho codifica
   a célula em `24` bits (`cel << 8 | fatia`, com `0xffffff` = sem célula); uma tabela com mais de
   `2²⁴` entradas **corrompe as somas de outras células em silêncio**. Pedir `16` entradas por pixel
   deu **`499 623` pontos** — e a `4K` a `4`/px a tabela já pedia `33 M`. ⛔⛔ **E o gate
   `a_tabela_segue_a_vista_e_o_tecto_do_dispositivo` DEFENDIA o defeito:** ele afirmava o tamanho
   pedido a `4K` acima de `2²⁴`. Hoje afirma `ENTRADAS_MAX` ali, com o CONTROLO de que o tecto não
   morde a `1920×1080`. O limite nomeia o recurso: **a largura da codificação da lista**, não a
   memória.

### §9.4 — Gates e prova

- `preview_device_w9_ceu_tempo_tests::um_giro_longo_nao_enche_a_tabela` (GPU): cena `28`, `80`
  quadros de `3°`, contra a exacta. CONTROLO: a cena tem de ter `> 1 000` píxeis de peça. Barra
  `≤ 400` pontos; mede **`10`**.
- `a_tabela_segue_a_vista_e_o_tecto_do_dispositivo`: a metade de `4K` reescrita (ver §9.3.3).
- Mutação (`muta_perto.py`, pré-voo `17/17` âncoras): **`M14`** `VELHA` de volta a `120` ·
  **`M15`** sem `ENTRADAS_MAX` · **`M16`** `TODAS → 0` (sem lugar volta a marchar uma fatia) —
  **as três sangram**. ⚠️ A âncora da `M15` mudou com o `clamp` que o clippy pediu
  (`quer.min(cabe).clamp(1 << 16, ENTRADAS_MAX)`) e foi re-corrida.
- GPU `62 + 3`; `nextest` das três crates `642/642`; clippy `-D warnings` zero.

## §10 — ⭐⭐⭐⭐ *«o ruído persiste se a view é rotacionada várias vezes»* + o material e o arrasto (reports do dono, 2026-10-01)

### §10.1 — O ruído: uma célula «cheia» de soma ZERO lida como céu aberto

A §9 curou o giro LONGO e não este: a sonda ganhou `PH2D_SONDA_GESTOS=<g>` (e `PH2D_SONDA_VAIVEM=1`),
que parte os quadros em `g` gestos com um quadro ASSENTE entre eles — o dono a largar o rato. No nó
(`=28`, `0,4` do enquadramento, `1920×1080`, `240` quadros de `3°`):

| | um gesto | 12 gestos | 12 de 2 quadros |
|---|---|---|---|
| pontos claros | `14` | **`15 463`** | `65` |

⇒ quem o fabrica é o ASSENTE no meio de uma sessão longa. ⛔ **A ablação dos atalhos da tabela não
mexeu** (`base` `15 445` · herança sem cópias `15 482` · sem herança `15 651` · sem a do nível
`15 651` · sem o tecto de fatias `15 702`) — o defeito não era uma aproximação.

O contador que separou (`n-le-peso-zero`, novo: o `le` a encontrar uma célula cheia com `peso = 0`):
`1` por quadro num gesto, **`991`** em doze. O `le` lia `0/0` como `1,0` — **céu aberto**, o ponto
claro. O mecanismo: os píxeis de uma célula ROUBAM a mesma vaga velha ao mesmo tempo, e o perdedor do
`compareExchange` — cujo `old_value` já era a SUA chave — seguia para a vaga seguinte e reclamava uma
**segunda** célula com a mesma chave. No assente o `zera` dava as duas por cheias e o `grava` só achava
a primeira (a procura pára nela) ⇒ a duplicada ficava cheia de soma zero; muito depois, quando a
primeira era roubada por outra chave, a procura caía na duplicada.

Cura em duas metades:
1. **`entrada_de`**: quem perde o roubo para a MESMA chave fica com a célula; e a troca da impressão é
   forte (`troca_forte` — a `Weak` pode falhar espuriamente e deixar uma vaga VAZIA para trás da
   chave). Contadores a `240` quadros/`12` gestos: cheias-vazias `991 → 5`, sem-lugar `1 165 → 83`
   (as duplicadas ENCHIAM a tabela), mediana do quadro `42 → 29 ms`; pontos `15 463 → 100`.
2. **`ceu_tempo_le`**: uma cheia de peso zero nunca é céu aberto — o pixel paga os `48` cones e a
   célula volta a VAZIA para o pedido seguinte. Pontos `100 → 0` (num gesto e em doze), `7` em
   vaivém (era `4 048`), `19` em `480` quadros/`24` gestos.

⛔ **Medido e REVERTIDO:** marcar a vaga como LIDA no instante da reclamação (para outra chave não a
roubar no mesmo passe) — consistentemente PIOR (`13`–`15` cheias-vazias, `163`–`240` pontos em três
corridas).

### §10.2 — *«ao mudar o material dos objetos, o render só atualiza ao arrastar»*

Duas metades no `materials::sync`, cada uma suficiente:
- **o dono que não existia**: N folhas com o MESMO material nascem SEM lei do dono (a regra de
  2026-09-22, `300×`), e a re-tradução dos números deixava-a assim — o pintor lia `all[0]` em toda a
  peça. ⇒ `Table::renovada` constrói quando `precisa_de_donos` VIRA (a regra é uma função só, lida por
  `build` e `renovada`).
- **a tabela emprestada**: a troca era no sítio (`Arc::get_mut`), `None` enquanto um traçado em voo a
  segura — e no `Render` o refinamento segura-a dezenas de passagens. ⇒ a tabela nova nasce ao lado
  (`Clone` barato: a geometria compilada passou a viver num `Arc`), e o refinamento em voo é largado
  (`Smoke::larga_os_refinamentos`; um traçado de MOVIMENTO nunca).

### §10.3 — *«arrastar objetos tem um delay absurdo»* — uma das causas

O mesmo `sync` chamava `forget_requests` em TODA mudança do documento — a cada quadro de um arrasto
de gizmo. Sem pedido guardado o `next_trace` pede o quadro ASSENTE inteiro, e o arrasto nunca tinha
quadro de MOVIMENTO. O laço já vê o documento novo sozinho (o pedido guardado leva-o) ⇒ o `forget`
ficou só no ramo dos NÚMEROS. ⚠️ **A outra causa conhecida NÃO está curada:** com materiais
distintos, cada quadro de arrasto reconstrói a tabela e COMPILA uma fita de dono por folha (a sonda
`measure_what_building_the_table_costs_per_frame`); e acrescentar uma forma com lei do dono custa
`2 310 ms` (a regra de 2026-09-22).

### §10.4 — Gates e prova

- `girar_varias_vezes_nao_deixa_pontos` (GPU): `12` gestos de `20` quadros com assente entre eles,
  barra `50`, mede `9`. Mutação: sem a leitura curada `126` (sangra) · sem as duas `15 353` (sangra)
  · ⚠️ **sem a reclamação curada SOBREVIVE, NOMEADA**: a leitura curada esconde-a no pixel; o efeito
  dela é de custo e está na sonda (os contadores acima).
- `materials::tests::a_cor_nova_de_uma_forma_constroi_o_dono` — sangra sem o ramo do
  `precisa_de_donos`.
- `reach_tests::shading::o_documento_mantem_o_pedido_e_a_cor_larga_o` — duas metades, as duas
  sangram (o `forget` de volta no ramo do documento · tirado do ramo dos números).
- `cargo nextest` das três crates `644/644`; GPU do céu no tempo `7/7`; clippy `-D warnings` zero.

### §10.5 — ⚠️ Os gates de GPU desta família correm por `cargo test`, não por `nextest`

Sob `nextest` (um processo por teste) **todo** gate de GPU deste ficheiro morre com `SIGSEGV`
DEPOIS do `ok` — na saída do processo, ao largar a placa —, e o `os_gestos_da_camara…`, anterior a
esta wave, morre igual ⇒ pré-existente. Corra-os com
`cargo test --release -p ph2d-app-field3d --lib ceu_tempo -- --ignored --test-threads=1`.

## §6 — Aberto

- ⏳ **O nó em TODO quadro** (§7.5): o que sobra é a marcha primária (`centro`, `2,7`–`4,8 ms`), a
  re-amostragem das bordas (`1,8`–`2,2`), a sombra da peça por pixel e as `48` direcções das
  células novas — cada corte medido ou muda a imagem ou a paridade com a CPU.
- ⏳ O relógio com a máquina CALMA (`load < 5`) para as tabelas do §3 e do §7.5.
- ⏳ **Smoke do dono** (`PH2D_FIELD_SMOKE=28`, Render, girar e fazer zoom — e AGORA aproximar muito,
  e girar MUITO de seguida: os pontos claros do §9 só nasciam depois de ~`24` quadros de giro).
- ⏳ O custo do pixel sem lugar (`48` cones no próprio quadro, §9.3.2) não foi varrido com a máquina
  calma — a `80` quadros são `~238` píxeis por quadro, e o relógio fica para a tabela do §3.
- ⏳ Os três primeiros quadros de um gesto de perto (`23`–`52 ms`, §8.4): o laço parte sem medição.
- ⏳ A cauda a girar a `0,25` de perto (`17,9`–`25,8 ms` nos últimos quadros, §8.4) — por medir se é
  a peça a encher o ecrã ou o laço a subir de resolução cedo demais.
- ⏳ **Reports do dono de 2026-10-01 ainda abertos:** *«ao colocar em render, o primeiro movimento de
  rotação ainda apresenta um delay»* (suspeita: os passes do MOVIMENTO só compilam no 1.º quadro de
  movimento — a sonda aquece-os com DOIS quadros `com` por isso) · *«ao acrescentar novos objetos o
  render fica lento»* e *«com 3 objetos rotacionar faz cair a resolução»* (o custo do quadro sobe por
  objecto; com materiais distintos a lei do dono compila uma fita por folha — §10.3) · a outra metade
  do arrasto lento (§10.3).
- ⏳ **Pergunta de RUMO devolvida ao dono** (*«tem certeza que esse sistema de modelagem pode
  funcionar em uma game engine?»*): o campo como FONTE de edição, e o jogo/editor a desenhar uma
  MALHA gerada dele — a recomendação está escrita na conversa; a decisão é dele.
