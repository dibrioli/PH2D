# 122 — CICLO 13: os cinco abertos do módulo, num ciclo

> **Ordem do dono, 2026-10-07:** *«resolver tudo que está em aberto em um único ciclo»*. A lista é a do
> [handoff de continuação de 07/10](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-07_ABERTOS_DO_MODULO.md):
> o Bug #14, o empate batida-tique do `pulse.beat`, o preparo de `value.attribute` + `value.reduce` na
> placa, o tecto de `32 768` a remedir e a submissão da leitura de volta. Cada item foi auditado contra o
> CÓDIGO do `main` das seis linhas (`2a925859d`) antes de construir — e dois dos cinco não eram o que a
> lista dizia (§1 e §5).

## §0 — O estado medido, o desenho e o kill-criterion de cada item (escritos ANTES)

| # | o que a lista dizia | o que o código e a medição disseram | desenho | kill-criterion |
|---|---|---|---|---|
| 1 | as estrelas da SAÍDA de uma zona perdem-se entre o cozimento e o ecrã | **não há perda**: as duas rotas entregam as estrelas em todo quadro, pela porta do produto e pela foto | a estrela legível + a sonda que mentiu, curada | a `=27` com estrelas na foto; um gate pela porta do desenho que reprova o defeito |
| 2 | o `pulse.beat` diverge ±1 tique entre CPU (`f64`) e placa (`f32`) nos empates | confirmado; e a divisão `f32` da WGSL nem é exacta (2,5 ULP) | o compasso decide-se no hospedeiro | paridade AO BIT nos `91` tiques da fixtura, empates incluídos |
| 3 | preparar `value.attribute` + `value.reduce` custa `~0,09 ms` de CPU por quadro | `+0,070 ms` sobre o condutor constante: `+0,029` na etapa da redução, `+0,032` no `submit` | ligações, uniforms e rascunhos GUARDADOS entre quadros | `≤ 0,05 ms` sobre o constante, intercalado no mesmo processo |
| 4 | o tecto de `32 768` foi medido com o carimbo na CPU | confirmado — o carimbo foi à placa no ciclo 12 e a tabela nunca foi refeita | remedir a escada `=17` até ao penhasco, nas duas placas | a tabela nova, com o recurso que limita; **o número é do dono** |
| 5 | a submissão da leitura de volta custa `0,34–0,42 ms` na NVIDIA | **não reproduz**: `0,045–0,058 ms` (NVIDIA) e `0,026–0,037` (iGPU), dos quais o `submit` próprio é `0,019` / `0,016` | a leitura no MESMO `submit` do cozimento + recursos guardados | um `submit` por quadro; a leitura ao bit igual à síncrona |

## §1 — Bug #14: não era perda — era o TAMANHO, e a sonda que o «confirmou» era cega

**A medição, pela porta do produto** (`coze_o_quadro` e a MESMA pergunta que o host faz em
`fase_vector_bands.rs`): as duas rotas entregam as `3 → 6 → 11` estrelas em todos os `180` quadros —
nos buffers da placa (`read_formas`, as `16` palavras válidas) e nas `vector_instances` da CPU —, parado
no `0`, a `3` tiques por quadro e com as tomadas dos gizmos ligadas. A camada da placa de formas pinta
`508` pixels. No app, instrumentado, `decide_do_dispositivo` e `desenha` correm em TODO quadro com a
população a crescer.

**A causa:** a estrela tinha `size = 0,07` em mundo. Na câmera do produto (`height_world = 10` sobre os
`~480 px` do recorte da cena, `48 px` por unidade) isso é um raio de `~3 px` — e a foto ampliada mostra-os,
três pontos brancos. A cruz de posição some quando o ponto tem forma (por desenho), logo a cena «não
mostrava nada». ⚠️ E a cascata sai do enquadramento: o `force.curl` puxa-a para baixo nos primeiros
`2 s` e depois espalha-a a `|x| > 10` aos `3 s` (medido, `probe` da zona) — uma foto aos `10 s` não tem
nada, com qualquer tamanho.

**A sonda que mentiu:** o `PH2D_PAN_DIAG` lia só as `vector_instances` da bomba, que na rota do
DISPOSITIVO ficam vazias de propósito (as formas vivem no buffer do cozimento). Ela imprimia `de 0` e
foi lida como *«o app não tem forma para desenhar»*. ⇒ ela passa a dizer a rota: `PLACA: <s> sprites e
<f> formas no buffer do cozimento (mundo não lido)` (`ph2d_pan_diag::note_device`, chamada pela ponte no
ramo que devolve `Handled`).

**A cura:** `ESTRELA = 0,22` (`~10 px`), uma constante com o porquê medido. **Gate**
`the_sparks_draw_legible_stars_on_both_routes` — o quadro pelo `coze_o_quadro`, a placa decidida como o
host decide e desenhada a `48 px`/unidade: `≥ 80` pixels por estrela nas duas rotas; **vermelho a `0,07`**
(`32` px por estrela, provado). **O censo da família** (`census_stamps_downstream_of_a_zone`, pelo
registo das cenas, todo nível): **uma** cena tem carimbo a jusante de uma zona — a `=27` —, e ela desenha
nas duas rotas, `10,6 px` a menor.

⛔ **Recusado por medição:** curar «a perda entre o cozimento e o desenho» — ela não existe.

## §2 — O compasso do `pulse.beat` decide-se no HOSPEDEIRO

**O defeito:** `k = ⌊(t − offset − i·stagger)/period⌋` em `f64` na CPU e em `f32` na placa. Nos
empates batida-tique (a fixtura decimal `0,0537`/`0,0031`: tique `36`, linha `3`, e tique `72`, linha `6`;
o período redondo `0,05` a cada três tiques) as rotas arredondam para lados opostos e discordam por um
tique inteiro. ⚠️ Trocar a CPU para `f32` não curava: a divisão da WGSL tem `2,5 ULP` de erro permitido.

**A lei nova** (`Compasso`, em `ph2d-node-pulse-beat`): o hospedeiro parte `u = (t − offset)/period`,
em `f64`, em `k₀ = ⌊u⌋` e `frac = u − k₀`, e calcula `sc = stagger/period`; os três viajam como
`DerivedUniform` (`bt_k0`, `bt_frac`, `bt_sc`) e a CPU lê-os da MESMA função. Por linha:
`k = k₀ + ⌊frac − i·sc⌋` — só `×`, `−` e `⌊⌋`, exactos ou correctamente arredondados nos dois lados.

⚠️ **O FMA:** a placa pode fundir `frac − i·sc` num FMA e arredondar uma vez só. ⇒ `sc` leva **7 bits
significativos** (`SC_BITS`): `i·sc` é EXACTO para `i < 2^17`, e um produto exacto arredonda igual
fundido ou não. O preço é a fase por linha quantizada a `≤ 0,4 %`. Um `const assert` prende-o ao
`MAX_INSTANCIAS_POR_NO` (quem subir o tecto acima de `131 072` tem de baixar o `SC_BITS`). Com
`phase_stagger = 0` (toda cena do produto) a lei é `⌊u⌋` em `f64`, ao bit a de antes.

**Gates:** `parity_pulse_beat` corre os `91` tiques (era `6`); `parity_pulse_beat_on_the_tick` (período
`0,05`, empate a cada três); a paridade do nascimento por pulso (`gpu_stream_ops_disparo`) deixa de
SALTAR tiques (eram `4` de `91`) e exige o mesmo pulso em todos. **Mutação:** a lei `f32` antiga no WGSL
sangra nas duas fixturas (volta `1`, linha `52`; volta `3`, linha `0` — o empate redondo). ⚠️ **A
quantização do `sc` SOBREVIVE à mutação** (sem ela os gates continuam verdes): nesta placa o produto
não fundido cai do mesmo lado. A imunidade a FMA é uma lei de construção SEM régua nesta máquina —
nomeada, não provada.

## §3 — O preparo da placa: as ligações GUARDADAS entre quadros

**Onde estava o custo** (cronómetros por etapa na ponte e no `cook_many`, `600` quadros, `release`,
NVIDIA, sem esperar pela placa): o condutor caro `0,094 ms` contra o constante `0,024`. Do excesso de
`0,070`: a etapa `value.reduce` `0,029` (um `BindGroup` + um uniform + um resultado + níveis CRIADOS a
cada quadro) e o `submit` `+0,032` (o rastreio desses recursos novos); `+0,0045` no prólogo (largá-los).
O plano, os condutores e a bomba: `< 0,002`.

**A cura — uma família, um bloco** ([`ligacoes.rs`](../../crates/ph2d-gpu-cook/src/ligacoes.rs)):
uniforms por CONTEÚDO (o mesmo buffer enquanto os bytes forem os mesmos), ligações por `(pipeline,
buffers)` (o mesmo `BindGroup` com os mesmos buffers), rascunhos `STORAGE` por classe de potência de
dois e por nível; tudo com IDADE (`8` viragens, uma por `cook`). Aplicada na etapa genérica de kernel,
no mapa e nos níveis da redução, no scan, nos passes estruturais (`Project`, `Compact`…) e na leitura
dos cartões. O resultado de 4 bytes da redução sai do pool (as mesmas identidades quadro a quadro).

**Medido, intercalado no mesmo processo** (`prof14tmp_ab` → a sonda do relógio do ciclo 6; «frio» =
`esquece_as_ligacoes()` antes de cada quadro, o custo antigo pelo MESMO código):

| placa | condutor caro quente | caro frio | constante quente | **caro − constante** |
|---|---:|---:|---:|---:|
| RTX | 0,0614 | 0,0776 | 0,0212 | **0,040** |
| iGPU | 0,0577 | 0,0727 | 0,0211 | **0,037** |

⏳ A rodada final (§6) repete-a ao lado da escada.

## §4 — O tecto de `32 768` com o carimbo na placa

**O instrumento é o do doc 120 §2** (a cena `=17`, A ESCADA DOS TECTOS: `motion.emitter` → laço de
forças → `motion.duplicator` com a imagem `Particle` ou a estrela viva → `motion.move`), pelo roteiro
[`mede_formas_na_placa.sh`](ferramentas/mede_formas_na_placa.sh) (ganhou `TECTO_FORMA`: `0` = imagem),
`release`, `1930 × 1040`, a média das três últimas janelas de `120` quadros, a carga antes e depois ao
lado (sem espera de calma — `CLAUDE.md` §2). As duas formas vão à placa inteiras: `[formas] … pela
PLACA (do dispositivo)` em toda célula da estrela.

### §4.1 — Dentro do tecto de hoje

| placa | forma | objectos | quadro | CPU (codificar) | Motion (cozer) | placa | carga |
|---|---|---:|---:|---:|---:|---:|---|
| RTX | imagem | 4 096 | 16,17 | 3,26 | 0,79 | 0,78 | 4,4→2,6 |
| RTX | imagem | 16 384 | 16,12 | 3,50 | 0,80 | 0,81 | 2,6→2,2 |
| RTX | imagem | 32 768 | 16,10 | 3,05 | 0,75 | 0,77 | 2,2→1,8 |
| RTX | estrela | 4 096 | 16,08 | 3,26 | 0,67 | 0,86 | 0,9→0,6 |
| RTX | estrela | 16 384 | 16,12 | 3,43 | 0,72 | 0,91 | 0,6→3,2 |
| RTX | estrela | 32 768 | 16,11 | 3,17 | 0,65 | 1,07 | 3,2→2,2 |
| **iGPU** | imagem | 4 096 | 16,13 | 2,95 | 0,68 | 4,97 | 1,8→1,2 |
| **iGPU** | imagem | 16 384 | 16,18 | 2,91 | 0,67 | 5,21 | 1,2→0,8 |
| **iGPU** | imagem | 32 768 | 16,12 | 2,97 | 0,64 | 5,54 | 0,8→0,9 |
| **iGPU** | estrela | 4 096 | 16,21 | 3,40 | 0,67 | 6,08 | 2,2→3,5 |
| **iGPU** | estrela | 16 384 | 16,38 | 4,00 | 0,77 | 8,41 | 3,5→4,7 |
| **iGPU** | estrela | 32 768 | 16,15 | 4,58 | 0,90 | 11,76 | 4,7→4,3 |

**Contra o doc 120 §3, à mesma população (`32 768`):** o Motion caiu de `4,56`/`4,60` (RTX) e
`7,30`/`5,06` (iGPU) para `0,65`–`0,90` ms e ficou **PLANO** com a população; a codificação caiu de
`7,1`–`10,8` para `3,0`–`4,6`; e a estrela no proxy de telemóvel, a única célula que passava o quadro
(`17,96`), fecha a `16,15` com a placa a `11,76` (era `17,95`). ⇒ *a nota de 24/09 morreu como ela
própria anunciou: o que limitava era a CPU a carimbar, e o carimbo está na placa.*

### §4.2 — Acima do tecto (compilação LOCAL, nunca commitada)

**Como:** `MAX_INSTANCIAS_POR_NO = 1 048 576` e `LADO_MAX_DE_GRELHA = 1024` só no disco, com as cercas
de compilação que dependem do tecto neutralizadas por `#[cfg(any())]` e marcadas
`LOCAL-MEDICAO-NAO-COMITAR` (a do WGSL do `motion.grid`, as duas da cena do carimbo, a da `=107` e a
NOVA do `pulse.beat`, §2); revertido por `git checkout --` com `grep -rn LOCAL-MEDICAO crates shells` a
ler `0`. Cada série SOBE e pára quando o quadro passa de `40 ms`. ⚠️ **A máquina estava partilhada**
(outras linhas a compilar, carga até `90`): uma célula que FECHA o quadro sob carga fecha-o calma (a
carga só abranda); as que passaram do quadro foram corridas DUAS vezes e fica a menor.

| placa | forma | objectos | quadro | CPU (codificar) | Motion (cozer) | placa | carga |
|---|---|---:|---:|---:|---:|---:|---|
| RTX | imagem | 65 536 | 16,26 | 4,99 | 1,13 | 0,95 | 21→5,6 |
| RTX | imagem | 131 072 | 16,17 | 3,09 | 0,76 | 0,70 | 5,6→3,1 |
| RTX | imagem | 262 144 | 16,16 | 3,16 | 0,77 | 0,85 | 3,1→12 |
| RTX | imagem | 524 288 | 16,12 | 3,24 | 1,07 | 1,30 | 12→21 |
| RTX | imagem | **1 048 576** | **16,18** | 4,46 | 1,29 | 2,00 | 21→19 |
| RTX | estrela | 65 536 | 16,28 | 5,62 | 1,36 | 1,70 | 19→24 |
| RTX | estrela | 131 072 | 16,10 | 3,41 | 0,86 | 2,54 | 24→18 |
| RTX | estrela | 262 144 | 16,10 | 2,88 | 0,74 | 4,71 | 18→12 |
| RTX | estrela | **524 288** | **16,14** | 3,39 | 0,79 | 8,79 | 12→20 |
| RTX | estrela | 1 048 576 | 20,80 | 3,30 | 0,78 | **17,55** | 20→12 |
| **iGPU** | imagem | 65 536 | 16,16 | 5,53 | 1,15 | 6,47 | 12→7,3 |
| **iGPU** | imagem | 131 072 | 17,36 | 5,14 | 1,19 | 7,42 | 7,3→8,6 |
| **iGPU** | imagem | 262 144 | 18,76 | 3,10 | 0,65 | 9,03 | 8,6→4,8 |
| **iGPU** | imagem | 393 216 | 98,71 | 14,82 | 6,70 | 12,41 | 33→69 |
| **iGPU** | imagem | 524 288 | 111,96 | **74,13** | **54,19** | 12,20 | 69→50 |
| **iGPU** | estrela | **49 152** | 17,64 | 4,13 | 0,64 | **13,70** | 40→33 |
| **iGPU** | estrela | 65 536 | 24,09 | 7,75 | 1,52 | **18,40** | 33→33 |
| **iGPU** | estrela | 131 072 | 49,79 | 5,27 | 1,37 | 31,36 | 75→43 |

**Leitura:**

1. **Na RTX nada chega ao penhasco até ao maior número medido:** a imagem fecha o quadro com
   `1 048 576` (a placa a `2,0 ms`, a CPU a `4,5`); a estrela fecha até `524 288` e passa-o a
   `1 048 576` pela PLACA (`17,55 ms` — a rasterização das curvas, linear: `~17 µs` por mil estrelas).
2. **No proxy de telemóvel a estrela é presa pela PLACA**, como no doc 120 §4.3, mas o dobro mais
   longe: `11,76 ms` a `32 768` (confortável), `13,70` a `49 152` (o quadro escorrega para `17,6`),
   `18,40` a `65 536`.
3. **A imagem no proxy fecha até `~262 144` e depois cai do PENHASCO** (`99`–`112 ms`): a coluna que
   explode é o Motion (`54 ms` a `524 288`) — a dívida de tiques do doc 120 §4.1, a mesma lei, agora
   `8×` mais acima. ⚠️ Essas duas células correram a carga `33`–`90`; o penhasco é real (a placa está a
   `12 ms` e o Motion não), o NÚMERO exacto não é.

### §4.3 — A tabela de decisão (com FORMA e SIMULAÇÃO, o uso real) — ⛔ o número é do DONO

**Confortável** = o recurso que aperta abaixo de `~90 %` do quadro; **limite** = a última célula que
ainda fecha o quadro.

| forma | desktop (RTX) confortável | desktop limite | telemóvel (proxy) confortável | telemóvel limite |
|---|---:|---:|---:|---:|
| imagem (`Particle`) | **≥ 1 048 576** (o maior medido) | > 1 048 576 | **~131 072** | ~262 144 |
| estrela vectorial | **524 288** | ~524 288–1 048 576 | **32 768** | ~49 152 |
| *(doc 120, 24/09, para comparar)* | *≥ 65 536 / 32 768* | | *~49 152 / ~16 384* | |

| saída | o artista ganha | o artista paga |
|---|---|---|
| **A. ficar em `32 768`** | toda célula medida, as duas formas nas duas placas, a 60 fps com folga | o desktop faz `16×` (estrela) a `32×` (imagem) mais do que o tecto deixa pedir |
| **B. subir a `65 536`** | o dobro no desktop e nas imagens do telemóvel, tudo com folga | a estrela no telemóvel a `65 536` passa o quadro (`24 ms`, ~40 fps) |
| **C. subir a `131 072`** | `4×` no desktop, as duas formas com folga; imagens no telemóvel no limite | a estrela no telemóvel a `131 072` cai para `~20 fps` (`50 ms`) |

⭐ **Recomendação técnica: B — `65 536`.** É o maior número em que o desktop tem folga com as duas
formas E o telemóvel tem folga com imagens; o único custo é a estrela em massa no telemóvel, que já é o
caso mais caro hoje (`11,76 ms` de placa no tecto actual). ⚠️ **O tecto POR NÓ é o mesmo número para
todos os dispositivos**, e é isso que obriga a escolher entre o desktop e a estrela no telemóvel: a cura
a sério é um tecto por DISPOSITIVO (o recurso que limita, medido pela placa que corre) — nomeado aqui,
não feito. ⚠️ O `pulse.beat` prende o tecto a `≤ 131 072` (§2): subir além disso pede baixar o
`SC_BITS` no mesmo passo.

⏳ **A decisão é do dono** (ele mandou manter o `32 768` em 24/09). Nada foi mudado.

## §5 — A leitura de volta: `0,34–0,42 ms` não reproduz; o `submit` próprio sai

**Auditado contra o código de hoje:** o custo de `0,34 ms` do doc 120 era o da encomenda com UM buffer
`mapped_at_creation` por coluna — curado no próprio ciclo 12 (um buffer de parâmetros persistente). O
que restava: a leitura codificava num encoder SEU e fazia o SEGUNDO `submit` do quadro, e recriava o
destino, a leitura e uma ligação por coluna. Medido (`release`, sem esperar): `0,045–0,058 ms` (NVIDIA) e
`0,026–0,037` (iGPU) por quadro, dos quais o `submit` próprio `0,019` / `0,016`.

**A cura:** o PEDIDO fica registado e o cozimento seguinte codifica a leitura no SEU encoder, no MESMO
`submit` (`tap_no_cozimento` / `tap_depois_do_submit`); só o ARRANQUE — um cozimento que aconteceu antes
de haver pedido — encomenda pelo próprio `submit`. Destino, leitura `MAP_READ` e ligações guardados
(recriados só quando o tamanho muda). A leitura fica um quadro MAIS FRESCA (lê os streams do cozimento
que a codificou, não os do anterior).

**Gates:** `o_cozimento_leva_a_leitura_no_seu_submit` (depois do arranque, o cozimento deixa a leitura
em voo sem nova chamada, e ela é ao bit a síncrona); o terceiro passo de
`sem_nada_a_amostrar_a_leitura_antiga_e_esquecida` passou a afirmar a intenção (a leitura depois do
vazio ou está em voo ou é NOVA — com o protocolo novo ela pode já estar pronta).

## §6 — A rodada de medição (uma, em série só onde o resultado mudava o passo seguinte)

1. **No processo** (`sonda_as_ligacoes_guardadas`, `release`, `PH2D_SONDA_SEM_ESPERA=1`, `9` rodadas ×
   `40` quadros, blocos rodados, o MÍNIMO) — carga `3,2 → 5,1`:

   | placa | caro quente | caro frio | const quente | const frio | **caro − const (quente)** |
   |---|---:|---:|---:|---:|---:|
   | RTX | 0,0595 | 0,0758 | 0,0199 | 0,0206 | **0,040** ✅ ≤ 0,05 |
   | iGPU | 0,0592 | 0,0760 | 0,0207 | 0,0218 | **0,039** ✅ ≤ 0,05 |

   ⚠️ O «frio» esquece a memória mas o resultado da redução já sai do pool — ele é um tecto INFERIOR
   do custo antigo; a partida medida antes do código (cronómetros, o mesmo dia) era `+0,070`.
   A memória fica em `8` ligações no regime (`ligacoes_vivas`).
2. **A escada `=17`, dentro do tecto** (o binário do produto, 12 células) e **acima** (compilação LOCAL,
   séries a subir até `40 ms`, as bordas repetidas) — §4. ⚠️ Série, e digo porquê: o tecto da compilação
   local dependia de a imagem ainda estar PLANA a `32 768` (estava — §4.1 —, por isso a compilação foi a
   `1 048 576` e não a `131 072`).

⛔ **O SIGSEGV da RTX ao SAIR do binário de testes `it` do `ph2d-gpu-cook`** (`3/3` corridas, dentro de
`libnvidia-glcore`, DEPOIS dos testes) **NÃO é deste ciclo** — ablação: o binário da base `2a925859d`,
compilado numa worktree temporária com target próprio, morre igual (`3/3`, intercalado, carga
`2`–`6`). É a família do doc 121 §9.18 (F): o `OnceLock` partilhado do teste segura o dispositivo até
ao `exit`. O vermelho intermitente na mesma suíte é o flake de carga já registado
(`crossing_the_reach_boundary_does_not_step_the_cost`, `2/2` verde sozinho a `load 3`).
