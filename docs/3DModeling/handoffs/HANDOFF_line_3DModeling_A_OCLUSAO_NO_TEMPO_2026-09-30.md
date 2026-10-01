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

## §6 — Aberto

- ⏳ **O nó em TODO quadro** (§7.5): o que sobra é a marcha primária (`centro`, `2,7`–`4,8 ms`), a
  re-amostragem das bordas (`1,8`–`2,2`), a sombra da peça por pixel e as `48` direcções das
  células novas — cada corte medido ou muda a imagem ou a paridade com a CPU.
- ⏳ O relógio com a máquina CALMA (`load < 5`) para as tabelas do §3 e do §7.5.
- ⏳ **Smoke do dono** (`PH2D_FIELD_SMOKE=28`, Render, girar e fazer zoom).
