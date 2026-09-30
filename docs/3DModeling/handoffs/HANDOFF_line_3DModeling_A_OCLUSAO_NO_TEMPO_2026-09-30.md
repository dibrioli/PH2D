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

## §6 — Aberto

- ⏳ **A marcha do nó a aproximar** (`21 ms` sem céu) — o alvo seguinte do tempo real.
- ⏳ A oclusão em movimento ainda pesa `~5 ms` no nó a girar e `2`–`5 ms` nas outras — medir onde
  (pede · marcha · le) antes de mexer.
- ⏳ O relógio com a máquina CALMA (`load < 5`) para a tabela do §3.
- ⏳ **Smoke do dono** (`PH2D_FIELD_SMOKE=28`, Render, girar e fazer zoom).
