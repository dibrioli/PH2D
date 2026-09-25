# 120 — CICLO 12: os tectos confortáveis (2026-09-24)

> Ordem do dono: *«siga para ciclo 12. lembre-se que o runtime irá rodar em mobile»*. A fila
> ([doc 103](103_dinamica_dos_ciclos.md) §5) diz o que o ciclo entrega: **uma tabela**, e o tecto é
> **decisão do Enio** tirada dela. O §5.1 do mesmo doc diz porque ele é o último: *o tecto confortável
> só se escreve depois de o `10` decidir quem vai ao dispositivo, senão ele é o número da CPU outra
> vez* (`CLAUDE.md` §0.0).
>
> ⚠️ Este ciclo **não muda produto nenhum**: ele mede e devolve a decisão. A única linha nova de
> produto é a cena de medição (`PH2D_MOTION_OBJ_SMOKE=17`), que fica porque é o instrumento que o
> dono corre na máquina dele.

## §1 — O que «confortável» quer dizer aqui

**Confortável = o quadro fecha em `16,7 ms` (60 fps) com folga, na configuração que o artista usa.**
Três coisas entram na definição e são medidas separadas, porque cada uma tem um recurso diferente:

| parte | o que custa | recurso | quem a mede |
|---|---|---|---|
| **a simulação** (emissor + integrador + forças) | posições por partícula por tique | CPU *ou* placa, conforme a rota | `MOTION (cozer + separar)` do perfilador |
| **o carimbo** (`motion.duplicator` veste cada partícula com a forma) | uma instância por partícula | **CPU** — é a fronteira do planeador | idem |
| **o desenho** | um quad do atlas por objecto, ou uma curva vectorial por objecto | placa (+ CPU para codificar as curvas) | `cpu-encode(raw)` e `gpu-busy(span)` |

⛔ **E o que NÃO entra:** o relógio de parede de uma máquina ocupada. Toda leitura abaixo traz o
`load` ao lado, e nenhuma acima de `load ~5` foi usada (`CLAUDE.md` §5.0).

## §2 — O instrumento: a cena `=17`, A ESCADA DOS TECTOS

[`motion_object_smoke_escada.rs`](../../crates/ph2d-app-motion/src/motion_object_smoke_escada.rs) —
o uso real do dono dentro, por regra (`CLAUDE.md` §0.8): uma **FORMA** e uma **SIMULAÇÃO com campos**.

- `motion.emitter` (taxa `n / VIDA`, vida `4 s`, rectângulo `18 × 6 m`) → `motion.integrate` ←
  `force.vortex` → `force.curl` → `force.drag` (o laço `pre` de sempre);
- `motion.duplicator` veste cada partícula com o quadradinho `Particle` (`source.object`) ou, com
  `PH2D_TECTO_FORMA=1`, uma **estrela vectorial viva** (`source.shape`);
- `motion.move` → `motion.output`.
- `PH2D_TECTO_N=<n>` escolhe a população (omissão `16 384`); ela **enche em `4 s`** e depois fica.

⭐ **As DUAS formas vão pela MESMA rota — a HÍBRIDA** (gate
`a_escada_mede_a_rota_hibrida_com_as_duas_formas`): a simulação e o carimbo correm na CPU e só o
`move` vai à placa. ⚠️ A 1.ª redacção deste gate afirmava que a estrela caía inteira na CPU e
**reprovou** — o que a estrela muda é o **desenho**, não a rota do cozimento. *Sem o gate a tabela
podia estar a medir uma rota que a cena já não toma.*

**O proxy de TELEMÓVEL** é a placa integrada desta máquina (AMD Radeon, **2 CU RDNA2**), escolhida por
`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json`. ⚠️ Ela é um proxy da **PLACA** do telemóvel
(ordem de grandeza de um A14 / Adreno 660 — os mínimos do SKILL §4), **não** da CPU dele: o CPU desta
máquina continua a ser o do desktop. Ver §6.

## §3 — A ESCADA, dentro do tecto de hoje (`32 768` por nó) — medida calma

Release, `1930 × 1040`, `load 1,3`–`3,4`, a média das três últimas janelas de 120 quadros, depois de
a população encher. Todos os números em ms; o quadro é o do vsync (`16,67` = 60 fps).

| placa | forma | objectos | quadro | CPU (codificar) | Motion (cozer) | placa |
|---|---|---:|---:|---:|---:|---:|
| RTX | quadrado | 4 096 | 16,69 | 4,41 | 1,95 | 0,68 |
| RTX | quadrado | 16 384 | 16,66 | 6,03 | 3,42 | 0,73 |
| RTX | quadrado | 32 768 | 16,70 | 7,12 | 4,56 | 0,74 |
| RTX | estrela | 4 096 | 16,64 | 4,24 | 1,28 | 0,93 |
| RTX | estrela | 16 384 | 16,64 | 6,89 | 2,96 | 1,70 |
| RTX | estrela | 32 768 | 16,65 | 10,02 | 4,60 | 3,13 |
| **iGPU** | quadrado | 4 096 | 16,66 | 6,68 | 4,28 | 4,53 |
| **iGPU** | quadrado | 16 384 | 16,63 | 7,92 | 5,54 | 4,74 |
| **iGPU** | quadrado | 32 768 | 16,68 | 10,14 | 7,30 | 5,05 |
| **iGPU** | estrela | 4 096 | 16,71 | 3,81 | 1,35 | 5,98 |
| **iGPU** | estrela | 16 384 | 16,66 | 7,30 | 3,14 | 11,30 |
| **iGPU** | estrela | 32 768 | **17,96** | 10,84 | 5,06 | **17,95** |

**Leitura:**

1. **Tudo fecha a 60 fps até ao tecto de hoje, menos uma célula** — a estrela no proxy de telemóvel a
   `32 768` passa o quadro (`17,96 ms`, ~56 fps), e é **a PLACA** que o passa (`17,95 ms`), não a CPU.
2. **O quadrado quase não pesa na placa** (`0,68 → 0,74 ms` na RTX, `4,53 → 5,05` no iGPU, ×8 de
   objectos): um quad do atlas é barato. **A estrela pesa linear** (`5,98 → 17,95` no iGPU): cada uma é
   uma curva que a placa rasteriza.
3. **O que cresce com o número é a CPU** — o cozimento (`1,95 → 4,56 ms` na RTX) e a codificação
   (`4,41 → 7,12`). É ela que decide o tecto visível num telemóvel, onde a CPU é o recurso escasso (§6).

## §4 — ACIMA do tecto (compilação LOCAL, nunca commitada)

**Como:** `MAX_INSTANCIAS_POR_NO` subido a `1 048 576` e `LADO_MAX_DE_GRELHA` a `1024` **só no disco**,
com as três cercas de compilação que dependem do tecto (a guarda WGSL do `motion.grid`, as duas da
cena `=126`, a da `=107`) neutralizadas na mesma compilação e marcadas `LOCAL-MEDICAO-NAO-COMITAR`;
tudo revertido por `git checkout --` antes do commit, com `grep -rn LOCAL-MEDICAO` a ler `0`.

⚠️ **A máquina foi partilhada a jornada inteira** (duas outras linhas a compilar e a correr suítes, e
um app de outra linha na RTX). A escada passou a **esperar `load < 4` antes de cada célula** e a
imprimir a carga no início e no fim; as corridas que apanharam carga foram **deitadas fora e
repetidas** (a primeira passagem acima do tecto correu a `load 26`–`80` e lia `3 fps` a `131 072` —
número de máquina, não de produto). A única célula abaixo com o fim a `load > 5` está marcada.

| placa | forma | objectos | quadro | CPU (codificar) | Motion (cozer) | placa | carga |
|---|---|---:|---:|---:|---:|---:|---|
| RTX | quadrado | **98 304** | **16,66** | 13,92 | 11,18 | 0,78 | 2,4→2,0 |
| RTX | quadrado | 131 072 | 23,07 | 23,11 | 19,83 | 0,79 | 2,0→4,5 |
| RTX | estrela | 65 536 | 26,02 | 24,87 | 18,51 | 6,28 | 3,6→**8,8** |
| RTX | estrela | 131 072 | 77,72 | 77,40 | 54,20 | 9,58 | 3,9→3,1 |
| **iGPU** | quadrado | **49 152** | **16,71** | 12,32 | 9,23 | 5,30 | 3,8→4,2 |
| **iGPU** | quadrado | 65 536 | 16,60 | 16,30 | 12,61 | 5,61 | 3,4 |
| **iGPU** | quadrado | 98 304 | 29,32 | 29,01 | 24,78 | 6,19 | 2,6 |
| **iGPU** | quadrado | 131 072 | 59,53 | 59,01 | 57,73 | 6,78 | 3,9→2,9 |
| **iGPU** | estrela | **24 576** | **16,66** | 8,31 | 3,69 | **15,09** | 3,1→1,9 |
| **iGPU** | estrela | 65 536 | 39,97 | 33,01 | 16,49 | 31,97 | 3,9→6,3 |

### §4.1 — ⛔⛔ Acima do quadro NÃO há rampa: há um PENHASCO, e a causa está no código

O `Motion (cozer)` do iGPU vai de `12,61 ms` a `65 536` para `24,78` a `98 304` — **o dobro por `1,5×`
os objectos** —, e a RTX faz o mesmo entre `98 304` (`11,18`) e `131 072` (`19,83`). ⇒ não é o
carimbo a ficar super-linear: é a lei que o [`ph2d-eval-motion`](../../crates/ph2d-eval-motion/src/lib.rs)
já escreve por extenso — ***a shell cozinha UM QUADRO POR TIQUE EM DÍVIDA*** (`ticks_owed`) e só o
último é desenhado. Um quadro que passa os `16,7 ms` deixa um tique em atraso, o seguinte coze dois, e
fica mais lento ainda. *O preço realimenta.*

⇒ **o tecto confortável não se lê como «onde o quadro chega a 16,7», lê-se como «onde ainda sobra
folga»** — porque o passo seguinte a esse não custa 16,8 ms, custa o dobro. É a razão de a coluna
confortável da §7 estar **abaixo** da última célula que fecha.

### §4.2 — A placa integrada custa CPU também, não só placa

A mesma população com a mesma CPU coze mais devagar com o iGPU (`7,30` contra `4,56 ms` a `32 768`;
`24,78` contra `11,18` a `98 304`, onde o iGPU já está em dívida): na rota híbrida o `move` corre na
placa e o cozimento **espera** por ele. ⇒ *o proxy de telemóvel está a medir a espera pela placa lenta
dentro da coluna «Motion», e é por isso que ele é mais honesto do que a RTX para a pergunta do dono.*

### §4.3 — A estrela: dois tectos diferentes conforme a placa

- **No iGPU a estrela é presa pela PLACA** (`15,09 ms` de `16,67` a `24 576`) — a rasterização das
  curvas, uma por objecto.
- **Na RTX ela é presa pela CPU** (`24,87 ms` de codificar a `65 536`, placa em `6,28`) — cada estrela é
  um caminho vectorial construído e codificado a cada quadro, a *segunda metade* do item `10`
  ([doc 103 §5.1](103_dinamica_dos_ciclos.md)), que continua por fazer.

## §5 — O que já estava medido e continua a valer: a simulação SÓ NA PLACA

O `emitter_sim_ceiling_probe` ([`gpu_cpu_parity_sim.rs`](../../crates/ph2d-gpu-cook/tests/it/gpu_cpu_parity_sim.rs)),
que corre o emissor e o integrador **sem carimbo e sem desenho** — o tecto da SIMULAÇÃO em si:

| onde | 1 M | 4 M |
|---|---:|---:|
| RTX | — | 4,55 ms |
| iGPU (proxy de telemóvel) | 15,6 ms | 86 ms |
| CPU (referência) | — | 179–191 ms |

⇒ **a simulação pura aguenta milhões na placa**; o que a escada do §3 mostra é que, com uma FORMA a
vestir cada partícula, **a rota passa a híbrida** e o tecto passa a ser o da CPU (carimbo) e o do
desenho — duas ordens de grandeza abaixo.

## §6 — O telemóvel: o que o proxy diz e o que NÃO diz

**O que ele diz, com número:** a PLACA de um telemóvel da faixa mínima aguenta o quadrado até ao
tecto de hoje com folga (`5,05 ms` a `32 768`, `30 %` do quadro) e a estrela até `~16 384` com folga
(`11,30 ms`); a `32 768` estrelas a placa sozinha já passa o quadro. ⇒ *para formas VECTORIAIS o
tecto de telemóvel é a PLACA, e está entre `16 384` e `32 768`.*

**O que ele NÃO diz, e é o que mais pesa:** a CPU. Na rota híbrida o cozimento e o carimbo correm na
CPU e crescem com o número (`4,56 ms` na RTX a `32 768`), e a CPU de um telemóvel é **mais lenta por
núcleo e tem menos núcleos** que a desta máquina. ⚠️ Não há proxy honesto aqui: restringir núcleos
(`taskset`) mede **quantos** núcleos, não **quão rápido** cada um é — e é a velocidade por núcleo que
o laço do carimbo paga. ⇒ **a coluna «Motion (cozer)» é o número a multiplicar por um factor que só
o aparelho real dá.** A forma honesta de o ter é o dono correr a cena `=17` num Mac/iPad (a mesma
máquina que ele usa para testes, `CLAUDE.md` §4) e ler a barra de baixo.

⛔ **E a consequência de arquitectura, que esta escada torna visível:** enquanto uma cadeia com
FORMA for híbrida, o tecto do telemóvel é o da **CPU dele**, e não o da placa — o erro exacto que o
§0.0 nomeia (*o caminho lento a definir o tecto do rápido*), agora do lado do aparelho. A simulação
pura já vive na placa (§5); o que falta é o **carimbo** ir para lá também (o item `10`, primeira
metade: a contagem derivada no planeador — [doc 103 §5.1](103_dinamica_dos_ciclos.md)).

## §7 — A decisão do dono

### §7.1 — A tabela de decisão (tudo com FORMA e SIMULAÇÃO — o uso real)

**Confortável** = 60 fps **com folga** (o recurso que aperta abaixo de ~`90 %` do quadro), lido das
§3–§4. **Limite** = a última célula que ainda fecha o quadro, sem folga.

| forma | desktop (RTX) confortável | desktop limite | telemóvel (proxy) confortável | telemóvel limite |
|---|---:|---:|---:|---:|
| imagem (`Particle`) | **≥ 65 536** | ~98 304 | **~49 152** | ~65 536 |
| estrela vectorial | **32 768** | < 65 536 | **~16 384** | ~24 576 |

⚠️ **A coluna do telemóvel é um tecto SUPERIOR:** o proxy acerta a placa e usa a CPU do desktop, e a
CPU de um telemóvel é mais lenta (§6). O número real só o aparelho real dá.

### §7.2 — As três saídas e o preço de cada uma

| saída | o que o artista ganha | o que ele paga |
|---|---|---|
| **A. ficar em `32 768`** | toda célula abaixo do tecto a 60 fps no desktop, com as duas formas; no telemóvel as imagens com folga | a estrela no telemóvel já passa o quadro no tecto (`17,96 ms`, ~56 fps) |
| **B. subir a `65 536`** | o dobro de imagens no desktop, ainda com folga | no telemóvel as imagens ficam **no limite, sem folga** (e a CPU real é mais lenta); a estrela a `65 536` custa `26 ms` no desktop e `40 ms` no telemóvel — o tecto deixa de proteger quem usa formas vectoriais |
| **C. descer a `16 384`** | a estrela confortável no telemóvel | metade do que o desktop faz com folga; desfaz a subida de 22/09 |

⭐ **Recomendação técnica: A — ficar em `32 768`.** É o único número em que **toda** célula medida
(as duas formas, as duas placas) está a 60 fps ou a um passo disso, e acima dele não há degradação
suave (§4.1): o passo seguinte ao limite custa o dobro. O que sobe o tecto **a sério** não é mudar o
número — é o **carimbo ir para a placa** (o item `10`, primeira metade: W1(b) + W2 do
[doc 116](116_ciclo_10_o_carimbo_no_dispositivo.md) §5.6), porque a simulação pura já faz **1 M na
placa integrada em `15,6 ms`** (§5). ⚠️ **Nesse dia esta tabela morre**, e quem subir o tecto tem de
a medir outra vez (`CLAUDE.md` §0.0 — *quem move o número que tornava algo inalcançável tem de
reconferir a nota*).

⏳ **A decisão é do dono.** A medição que falta — a CPU real de um telemóvel — ele pode fazê-la
correndo a cena `=17` no Mac/iPad dele.

> **2026-09-24, resposta do dono: *«continue»*** — sem escolher uma das três. ⇒ o tecto **fica em
> `32 768`** (é o estado de hoje e a saída A), e a jornada segue para a alavanca que a §7.2 nomeia.

## §8 — O que a escada REABRE: a W1(b) do ciclo 10, com a população que faltava

O [doc 116 §5.6](116_ciclo_10_o_carimbo_no_dispositivo.md) pôs a W1(b) (o carimbo na placa) em ⏸️
com um número: *`36` de `36` cartões do catálogo trazem uma estrela VIVA*, logo a ponte recusava-os
uma camada acima e um kernel no carimbo **mudava zero cartões**. ⚠️ **Era verdade sobre o CATÁLOGO e
é falso sobre o RUNTIME** — o dono disse *«o runtime irá rodar em mobile»*, e num jogo o que se
carimba são **imagens** (`source.object`), que a escada mede aqui.

### §8.1 — ⭐⭐⭐ Onde a escada gasta: o carimbo custa pouco; o que ele custa é ARRASTAR a simulação

Sonda `sonda_onde_a_escada_gasta` (pump da CPU, população cheia, `--release`, ⚠️ `load 6`–`8` ⇒ leia
as **proporções**, não os milissegundos), com o CONTROLO a ser a MESMA simulação sem o carimbo:

| pedidos | variante | melhor tique | fronteira do planeador |
|---:|---|---:|---|
| 16 384 | só a simulação | 1,318 ms | **—** (a cadeia inteira na placa) |
| 16 384 | sim + carimbo | 1,715 ms | `motion.duplicator` |
| 32 768 | só a simulação | 2,523 ms | **—** |
| 32 768 | sim + carimbo | 3,109 ms | `motion.duplicator` |
| 65 536 | só a simulação | 6,340 ms | **—** |
| 65 536 | sim + carimbo | 7,804 ms | `motion.duplicator` (carimba `32 768`, o tecto) |

⇒ **o carimbo é ~20 % do cozimento; a simulação é ~80 %** — e ela só corre na CPU **porque o carimbo
é a fronteira**. Sem ele o planeador põe a cadeia inteira na placa, onde a mesma simulação faz
**`1 M` em `15,6 ms` no proxy de telemóvel** (§5). *Um nó CPU-only no meio de uma cadeia não custa o
que ele custa: custa o dispositivo inteiro* — a frase do doc 103 §5.1, agora com o número da cadeia
que o runtime usa.

### §8.2 — As DUAS cercas, e porque a segunda cai para a escada

1. **O carimbo não tem kernel** (`lowerings: &[Cpu]`). O molde existe — o `motion.clone` da W1(a) —, e
   o desenho do doc 116 §5.3 já o escreve: `SourceRows` na porta `shape` com `cp_rows = i / np`, o `P`
   e o `rot` somados das DUAS portas (o codegen já nomeia `read_<porta>_<coluna>` quando uma coluna é
   lida de duas — o `motion.integrate` faz `read_rest_vel`/`read_forces_vel`), e `Index`/`Count`
   renumerados **sempre** (a lei da CPU escreve-os incondicionalmente, ao contrário do clone).
2. **A ponte recusa um grafo de objecto cujo sufixo na placa mude a contagem**
   (`graph_has_object_source && suffix_changes_count`), porque a partição por textura
   (`texture_runs_from_boundary`) lê o `texture_id` da FRONTEIRA e alinha-o posição a posição com o
   sink. ⭐ **Mas a partição tem um caso em que a posição não importa:** quando **toda** textura do
   objecto é `0` (o átlas partilhado — o `Particle` da escada, e todo sprite do átlas), ela já é
   **vazia por construção** (*«all-atlas object graph → the legacy path already draws it»*), qualquer
   que seja a contagem. ⇒ a cerca pode passar a perguntar pelo CONTEÚDO, como a irmã de cima já faz
   com a geometria viva (`cook_publishes_live_geometry`).

### §8.3 — As waves

| wave | o quê | prova |
|---|---|---|
| **W1(b).1** | o kernel do carimbo, modo `Off` + `Shape Wins` + `point_scale = 0` (o de fábrica); os outros modos são `applicable = false` com a população ao lado | paridade CPU × placa ao bit na POSIÇÃO e na renumeração, com o controlo `np = 1` |
| **W1(b).2** | a cerca da ponte passa a perguntar pelo CONTEÚDO: um objecto todo no átlas não se recusa | gate de rota: a escada vai `FullyGpu`; o CONTROLO (um objecto com textura própria) continua recusado |
| **W1(b).3** | a escada outra vez, com a máquina calma | a tabela do §3/§4 com a cadeia inteira na placa |

### §8.4 — ✅ W1(b).1 e W1(b).2 FECHADAS (2026-09-24)

**O kernel** ([`ph2d-node-motion-duplicator/src/kernel.rs`](../../crates/ph2d-node-motion-duplicator/src/kernel.rs)):
o molde do `motion.clone` com as duas portas — `cp_rows = i / np` na forma, `read_points_P(i % np)`
no ponto, o `np` do ORÇAMENTO por `DerivedUniform` (a mesma chamada do `eval`), `Index`/`Count`
escritos sempre (a CPU cunha-os). A `rot` é a peça que não tinha verbo: a CPU escreve-a se
QUALQUER lado a traz, e nenhum `ColumnAccess` diz «escreve se uma das duas portas a tem» ⇒ a da
forma é `SourceReadWriteExisting` e a dos pontos é `RefuseIfPresent` (vai à CPU **só** nesse caso).

**A bancada** ([`gpu_cpu_parity_duplicator.rs`](../../crates/ph2d-gpu-cook/tests/it/gpu_cpu_parity_duplicator.rs)),
na placa, com a cadeia inteira reclamada:

| caso | instâncias | pior \|Δpos\| | pior \|Δtint\| |
|---|---:|---:|---:|
| 3 formas × 24² · a forma certa | 1 728 | **`0`** | `0` |
| 3 formas × 24² · a renumeração | 1 728 | **`0`** | `5,88e-3` |
| 4 formas rodadas × 17² · a forma certa | 1 156 | **`0`** | `5,88e-3` |
| 3 formas × 150² · **o orçamento** | 32 766 | **`0`** | `5,88e-3` |

⭐ **A posição é exacta AO BIT nos quatro**, e o `5,88e-3` é o piso da LUT da rampa (`1,5/255`) que
a bancada do `motion.clone` já mediu com o controlo em passagem — não é do carimbo. ⛔ A 1.ª
redacção comparava a BASE ao bit e reprovou por **um ULP** do `cos` (`0,92050487` contra
`0,9205048`): a trigonometria é do DESENHO de cada rota, a jusante do kernel, e a barra passou a ser
a da casa (`1e-4`), com o controlo de que a fixtura roda de facto.

**A cerca** (`cook_publishes_only_atlas_objects`, na ponte): a da contagem só recusa um objecto com
textura PRÓPRIA; um todo no átlas passa. Gate da pergunta + gate da fiação (a rota da ponte pede um
`GpuContext`, logo a chamada prova-se por texto, que é o que a casa faz para essa costura).

**O gate da cena mudou de premissa, à vista:** `a_escada_mede_a_rota_hibrida_com_as_duas_formas`
afirmava o carimbo como fronteira; hoje `a_escada_poe_a_simulacao_na_placa_e_so_a_fonte_fica_na_cpu`
afirma que a única fronteira é a FONTE (`source.object` / `source.shape`).

**Mutação: 10 de 10 sangram** — o gather na forma errada · o ponto lido em `i / np` · o `Index` por
renumerar · o `np` cru sem orçamento · a `rot` da forma perdida · as duas recusas apagadas · o kernel
por registar · a pergunta de conteúdo sempre verdadeira · a cerca a deixar de a chamar.

### §8.5 — ✅ W1(b).3: o carimbo foi para a placa e o Motion NÃO ficou mais barato — até se acharem DUAS contas escondidas (2026-09-24)

Com o carimbo na placa (§8.4), a escada no app media o **mesmo** Motion de antes (RTX `32 768`:
`~5,2 ms`). A leitura por relógios de medição **locais** (nunca commitados) partiu o quadro em dois
cozimentos na CPU que ninguém tinha pedido:

| conta | o quê | na RTX a `32 768` |
|---|---|---:|
| **1. a marcha do prefixo** | na rota HÍBRIDA a bomba da CPU avançava **toda** fonte de `pre` do grafo — o `motion.integrate` que a placa já tinha reclamado era simulado aqui também, e deitado fora | `4,0 ms` |
| **2. a tomada do gizmo das posições** | o `ponto_gizmo::taps_for` pedia **todos** os sinks, e numa rota de dispositivo uma tomada recozinha a cadeia inteira na CPU — para um gizmo que **não se desenha** (o sink tem aparência) | `2,4 ms` |

**As curas:**

1. **A marcha restrita ao CONE** ([`cook_advance_within.rs`](../../crates/ph2d-nodegraph/src/cook_advance_within.rs),
   aditiva em `ph2d-nodegraph`): `Cook::advance_tick_fanned_within` avança só as fontes de `pre` no cone
   a montante das FRONTEIRAS (seguindo também as arestas de `pre` — ⚠️ ao contrário do
   `cook_substep::upstream_cone`, que responde a outra pergunta). A bomba usa-a no braço
   `CookTarget::Boundaries`; o braço `Sinks` (a rota da CPU) fica **byte-idêntico**.
2. **A tomada pergunta à PLACA** ([`ponto_gizmo::taps_for`](../../crates/ph2d-app-motion/src/ponto_gizmo.rs)):
   quando o dispositivo desenhou o quadro anterior, o registo dele (`GpuCook::shape()`) diz que colunas
   cada sink levou, e o **mesmo** `veredito_do_dispositivo` que decidiu desenhar decide que o gizmo se
   cala ⇒ a tomada não se pede. ⚠️ Um quadro de atraso, nomeado. ⛔⛔ **E o gizmo do PIVÔ lia essa
   tomada de boleia** (a nota dele dizia *«o irmão já pede TODOS os sinks»*): hoje ele pede o sink ele
   próprio, só durante o arrasto — sem isso o alvo do pivô sumia **em silêncio**.

**A/B na mesma sessão** (RTX, `32 768`, imagens): Motion **`3,19 → 1,12 ms`** · o cozimento na placa
`2,32 → 0,23` · o prefixo `2,08 → 0,006`.

**A escada outra vez** (release, `1930 × 1040`, a média das três últimas janelas, `load 1,8`–`3,7` por
célula — ⚠️ com o app de smoke de OUTRA linha aberto a `~35 %` de um núcleo durante toda a corrida,
que não é meu e não se fecha):

| placa | objectos | Motion antes (§3) | **Motion agora** | CPU antes | **CPU agora** | placa |
|---|---:|---:|---:|---:|---:|---:|
| RTX | 4 096 | 1,95 | **1,17** | 4,41 | **3,55** | 0,70 |
| RTX | 16 384 | 3,42 | **1,15** | 6,03 | **3,42** | 0,67 |
| RTX | 32 768 | 4,56 | **1,18** | 7,12 | **3,59** | 0,69 |
| iGPU | 4 096 | 4,28 | **3,34** | 6,68 | **5,98** | 4,50 |
| iGPU | 16 384 | 5,54 | **3,48** | 7,92 | **6,84** | 4,78 |
| iGPU | 32 768 | 7,30 | **3,98** | 10,14 | **6,62** | 5,05 |

⭐⭐⭐ **O custo das IMAGENS deixou de crescer com o número**: de `4 096` a `32 768` o Motion na RTX
fica em `~1,2 ms` e no proxy de telemóvel em `3,3`–`4,0` — a simulação e o carimbo vivem na placa, e
o que a CPU paga já não é por objecto. ⇒ **a tabela de decisão do §7.1 ficou CONSERVADORA para
imagens**; a medição acima do tecto (compilação local) fica por refazer, e o tecto continua a
`32 768` porque é a decisão do dono.

⚠️ **A estrela não mudou de caminho** (ela é recusada à CPU **antes** de planear, pela forma
vectorial viva — ADR-0154) e leu `~0,7 ms` acima do §3 nesta sessão; o código dela é o mesmo, logo a
diferença é do ambiente (o app aberto acima).

✅ **E a leitura dos cartões, que era a próxima alavanca, FECHOU no §8.6.**

⛔ **Dois vermelhos PRÉ-EXISTENTES que a corrida de fecho apanhou** (os dois `#[ignore]`, logo o CI
nunca os corre): `write_the_rig_figures` (a cena `=120` tem hoje `19` pontos na corda contra os `20`
que o gerador de figuras afirma) e `measure_the_source_group` (é anterior à lei do dono *«só com
forma»*: um emissor sem forma não desenha, e a sonda conta instâncias — passa com
`PH2D_MOTION_SO_COM_FORMA=0`). Nenhum dos dois toca no que esta wave mudou.

**Gates:** a marcha restrita conta AVALIAÇÕES (`a_marcha_restrita_so_avanca_o_laco_que_a_fronteira_le`,
com o CONTROLO da marcha completa) · a bomba usa-a na rota de fronteiras e não na de sinks
(`a_rota_de_fronteiras_nao_simula_o_laco_que_elas_nao_leem`) · a lei do veredito e a filtragem, puras ·
e a costura inteira **na placa real** (`a_placa_que_desenhou_o_sink_nao_pede_a_tomada`, com o CONTROLO
de uma grelha sem aparência, que continua a pedir) · o pivô pede o sink dele. **Mutação 6 de 6.**

### §8.6 — ✅ A leitura dos cartões deixou de esperar pela placa (2026-09-24)

⚠️ **O que o dono aprovou no smoke da §8.5** (as imagens a `1,23 ms` de Motion; as cruzinhas das
posições intactas) é o ponto de partida desta wave, que ele mandou seguir.

**O defeito:** o `GpuCook::tap` fazia `poll(wait_indefinitely)`. O cabeçalho dele mediu `+0,075 ms`
num arnês sem janela — e ali a fila está vazia. No app a fila tem o quadro anterior inteiro, e a CPU
ficava parada à espera de TUDO: `0,68 ms` por quadro na RTX, `60 %` do Motion que sobrava.

**A cura** ([`tap_voo.rs`](../../crates/ph2d-gpu-cook/src/tap_voo.rs)): `GpuCook::tap_sem_espera`
encomenda a leitura num quadro e recolhe-a no seguinte (`poll(Poll)`, que não bloqueia), com **um**
pedido em voo de cada vez. Os cartões ficam um quadro mais atrás (já eram um). ⚠️ **Um quadro que
não é da placa DESCARTA a leitura** — sem isso, voltar à placa mostraria números de há minutos. O
`tap` síncrono **fica** para os gates e as sondas, e as duas rotas partilham o gather e a leitura
(`encomenda_tap` · `le_tap`), logo leem as **mesmas** amostras — e há gate a afirmá-lo ao bit.

**E a encomenda ficou mais barata:** ela criava um buffer de parâmetros **por coluna e por quadro**;
hoje é **um** buffer persistente escrito de uma vez, com uma fatia alinhada por coluna.

**Medido (imagens, `32 768`):**

| placa | Motion (§8.5) | **Motion agora** | CPU (§8.5) | **CPU agora** | load |
|---|---:|---:|---:|---:|---|
| **iGPU** (proxy de telemóvel) | 3,98 | **0,72–0,75** | 6,62 | **3,65** | `4,3 → 4,1` |
| RTX | 1,18 | ~1,1–1,3 | 3,59 | — | ⚠️ `4,5 → 5,8`, 48 processos de outras linhas |
| **RTX — o smoke do DONO** (máquina dele, calma) | 1,18 | **0,56** | 3,59 | **2,65** | — |

⭐⭐ **No proxy de telemóvel o Motion caiu `5×`** — o custo era esperar por uma placa LENTA, que é
exactamente o regime do telemóvel. ⚠️⚠️ **E a minha linha da RTX estava ERRADA por metade, e quem a corrigiu foi o smoke do dono:**
com a máquina calma ela lê **`0,56 ms`** de Motion e `2,65` de CPU, contra os `~1,1` que eu medi a
`load 4,5`–`5,8` com 48 processos de outras linhas vivos — *a lei do `load ~5` outra vez, e uma
medição de relógio numa máquina partilhada não conclui nada que o dono não possa reproduzir*. ⚠️ A
frase seguinte fica pela metade que continua verdade (o empacotamento não moveu a encomenda na
NVIDIA), e a conclusão *«quase não mexe»* **caiu**: ali o
que custa é a ENCOMENDA (`0,34`–`0,42 ms`, relógios locais), e o empacotamento dos parâmetros **não
a moveu** — o custo na NVIDIA está na submissão em si. ⏳ **Nomeado e não perseguido:** a cura seria
encomendar a leitura no MESMO submit do cozimento; no computador de secretária já é 60 fps com folga,
e o alvo do ciclo é o telemóvel.

**Gates:** a leitura encomenda num quadro e recolhe no seguinte, **igual ao bit** à síncrona, e
descartar esquece as duas coisas (com o controlo) · a leitura **nunca espera** (régua de TEXTO sobre
o corpo — um relógio seria mais uma flake de carga, e esperar dá a mesma resposta mais tarde) · na
ponte, dois quadros até haver números e um quadro da CPU a descartar. **Mutação 5 de 5** — esperar
pela placa · descartar só metade · a ponte sem descarte · a ponte de volta ao síncrono · todas as
colunas a lerem a fatia da primeira. ⚠️ **NOMEADA e sem régua determinística:** *«um pedido em voo de
cada vez»* — encomendar por cima de um em voo só se vê com uma placa mais lenta que o quadro, o que
nenhum gate sem relógio reproduz; a guarda é o `if em_voo.is_none()`.
