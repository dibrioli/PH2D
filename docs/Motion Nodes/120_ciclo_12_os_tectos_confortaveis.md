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
