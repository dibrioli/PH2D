# ARQUIVO — parte_2.md (história, 1789 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_2.md`](parte_2.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-505` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §9.10 — A VARIANTE ENXUTA: o tracejado inline dobrara os registos de TODA a cena (2026-10-03)

Commit `8cb0ab9e1`.

**O achado.** O handoff de 02/10 pedia re-medir a linha de base porque o `EixoItem` crescera `56 → 72 B`
no §9.9. A sonda intercalada (o binário de `7a58e7aaf`, antes do §9.9, e o de HEAD na MESMA janela calma,
`load < 4`, [`mede_sonda_das_estrelas.sh`](../../ferramentas/mede_sonda_das_estrelas.sh) com várias cópias)
achou uma regressão de **`+33 %` a `+41 %` na iGPU nos TRÊS arranjos** — e as conformes nem lêem o eixo.
⇒ não era o eixo maior.

**O mecanismo — medido sem relógio** ([`registos_dos_shaders.sh`](../../ferramentas/registos_dos_shaders.sh):
`RADV_DEBUG=shaderstats,nocache`, a mesma sonda). O ramo do tracejado (`tracejado_px`, `emite_tracejado`,
`ajuste_do_tracejado`) é código grande atrás de um `if` de dados; o compilador aloca os registos do PIOR
caminho para o shader inteiro:

| shader (iGPU, RADV) | antes do §9.9 | HEAD (§9.9) | enxuta | completa |
|---|---|---|---|---|
| fragmento do desenho | `56` VGPRs · `18` ondas/SIMD · `16 620 B` | **`128` · `8`** · `35 704 B` | `56` · `18` · `16 620 B` | `128` · `8` |
| `cs_escreve` (um fio por cópia) | `64` · `16` | **`128` · `8`** | `64` · `16` | `128` · `8` |
| `cs_conta` | `40` · `24` | `48` · `20` | `40` · `24` | `48` · `20` |

Nada em scratch; os outros kernels nossos e os do Vello idênticos. Ocupação `18 → 8` ondas no fragmento: a
iGPU deixa de esconder a latência das leituras encadeadas (cópia → registo → lista).

**A cura.** `override TRACEJADO: bool` no `shape.wgsl`, dentro do predicado `tracejado(it)` (os três
chamadores: o pixel a pixel, o `percorre` e o `limite_de_arestas`). O passe compila as duas variantes do
desenho, do `cs_conta` e do `cs_escreve` na criação (`contorno::Variantes`) e escolhe pelo eixo CARREGADO
(`EixoItem::tracejado`, o mesmo predicado, no mesmo buffer que o shader lê). A enxuta é byte a byte o
fragmento de antes (`16 620 B`). Uma cena com tracejado continua a pagar a completa.

**Gate:** `so_um_eixo_tracejado_pede_a_variante_completa` (com CONTROLO: a geometria contínua TEM eixo).
Os gates de pixel não vêem «completa sempre» — a mesma imagem, mais devagar; vêem «enxuta sempre» (a cena
tracejada perde os traços). ⚠️ «O `override` fora do predicado» não muda um pixel nem a escolha: só o
`registos_dos_shaders.sh` o vê — corra-o depois de mexer em qualquer ramo raro de um shader quente.

⭐ *Um ramo raro inline num shader quente cobra o seu preço a quem nunca o toma*: o relógio do §9.9 foi
tirado na RTX (que tem registos de sobra: `0,33 → 0,36 ms`), e a iGPU — o proxy de telemóvel — pagou `+41 %`.

**Mutação `8` de `9`** ([arnês](../../ferramentas/mutacao_a_variante_enxuta_2026-10-03.py), pré-voo `9/9`, corrida
LIMPA `16` verdes): escolhe sempre a completa (só o gate novo a mata) · sempre a enxuta · `Variantes::de`
trocada · opções de compilação trocadas · contagem / escrita / desenho sempre na enxuta · o predicado do
Rust aceita todo troço (só o gate novo) — e a **V9** (o `override` fora do predicado do WGSL)
**SOBREVIVE, como declarado**: só o `registos_dos_shaders.sh` a vê.

⛔ **Recusa MEDIDA — a origem das células como interpolante `flat`** (o vértice já lê a cópia; o fragmento
saltaria uma leitura encadeada antes da do registo). Decomposição iGPU, esticadas, `PH2D_FLUID_PROFILE=1`,
`load < 4`: desenho `0,81 → 0,80 ms` (critério escrito ANTES: `≥ 10 %`). A cadeia cópia → registo → lista
não é o que custa no desenho.

### §9.11 — A ABLAÇÃO DO DESENHO: `84 %` é o laço das listas (2026-10-03) — e o desenho seguinte

**A tabela calma de base** (cura = `8cb0ab9e1`; `load < 4` antes e depois de cada corrida, todas abaixo de
`1`; [`mede_sonda_das_estrelas.sh`](../../ferramentas/mede_sonda_das_estrelas.sh) com os binários intercalados;
placa · Vello, ms, média de `2`):

| arranjo | iGPU antes do §9.9 | iGPU cura | RTX antes do §9.9 | RTX cura |
|---|---:|---:|---:|---:|
| `72` grandes esticadas | `1,74` · `1,00` | **`1,74`** · `1,01` | `0,35` · `0,37` | **`0,33`** · `0,36` |
| `72` grandes conformes | `0,93` · `0,94` | **`0,93`** · `0,92` | `0,19` · `0,37` | **`0,17`** · `0,34` |
| `1225` densas da `=127` | `1,69` · `2,69` | **`1,69`** · `2,70` | `0,22` · `0,49` | **`0,20`** · `0,49` |

**A ablação** (iGPU, `PH2D_FLUID_PROFILE=1`, o relógio do passe `render.formas`, ms; os binários
mutilados não vão a commit — A1: o fragmento devolve uma cor constante; A2: lê o registo da célula e
devolve só o FUNDO, sem o laço das listas):

| arranjo | desenho inteiro | A2 (registo + fundo) | A1 (piso: vértice, rasterização, mistura) |
|---|---:|---:|---:|
| esticadas | `0,81` | `0,13` | `0,09` |
| conformes | `0,47` | `0,13` | `0,09` |
| densas | `0,88` | `0,11` | `0,07` |

⇒ **o laço das listas é `0,68` dos `0,81 ms`** (e `0,77` dos `0,88` nas densas): cada pixel de uma célula
de `32 px` percorre a lista INTEIRA da célula, e a aresta que só cruza um pixel é lida e avaliada pelos
`32`. O piso do hardware é `0,09`; o registo, `0,04`. O resto do quadro é o cálculo: células `0,43` ·
escrita `0,17` · contagem `0,07`.

**O desenho seguinte (✅ construído e medido no §9.12 — passou à 1.ª tentativa).** Não o rasterizador fino inteiro no cálculo que o
handoff de 02/10 previa: esse pedia a ORDEM entre cópias, porque a mistura passava ao cálculo. Basta tirar
do fragmento a SOMA, e deixar-lhe a mistura (o hardware já a faz na ordem certa):

- **o buffer de ACUMULAÇÃO por pixel** da grelha de células de cada cópia (o `accumulation buffer` dos
  rasterizadores de linhas de varrimento): cada aresta, na fileira, deposita a área do pixel que cruza e o
  RESTO (`dy − área`) no pixel seguinte — por atómicos em ponto fixo (`ESCALA_FIXA`), logo a ordem dos
  fios não muda um bit, como hoje. Uma aresta paga os pixels que CRUZA, não `32` por célula;
- **a varredura por célula** (um fio por célula): o fundo da célula (o prefixo que o `cs_lugar_das_listas`
  já faz) mais o prefixo dos depósitos dela dá a cobertura de cada pixel; as três famílias acabadas
  (`af` com a regra, `as_` com a soma do contorno e das marcas) gravam-se numa palavra;
- **o fragmento**: uma leitura, e a mistura de sempre; a fileira que não coube continua a refazer-se pelo
  caminho de sempre (o `SEM_LISTA` de agora, a mesma capacidade medida dois quadros depois).

⚠️ A lista por célula sai do caminho das células (as `68 276` entradas das esticadas viram depósitos);
recontar a capacidade e as guardas do §9.8 (a cena que muda, as fileiras que não cabem) na mesma
jornada.

**Kill-criterion (DIRETIVA §5, escrito ANTES):** na iGPU, `PH2D_FLUID_PROFILE=1`, `load < 4`, as `72`
esticadas com o quadro da placa (soma dos passes) **≤ `1,0 ms`** (o Vello inteiro) depois da 2.ª
tentativa, e nenhuma das outras duas pior que hoje (`0,76` · `1,50` de soma dos passes); na RTX nenhum
arranjo mais de `10 %` pior. Falhou ⇒ fica a lista, e o desenho entra aqui como recusa medida. É a 3.ª
topologia do caminho das células (§9.6 máscaras → §9.8 listas → acumulação): a prova do modelo é esta
ablação — o fragmento só com o registo custa `0,13` — e o que falta provar é o PREÇO do cálculo novo.

### §9.12 — O BUFFER DE ACUMULAÇÃO: o plano (2026-10-03, escrito ANTES de construir)

Comparação: o binário da sonda de `8cb0ab9e1` (a cura), compilado e copiado antes de mexer no shader.

**A lei que se preserva.** Hoje um pixel `x` de uma célula `k` soma `fundo(k) + Σ fixo(c(e, x))` sobre a
lista da célula (`c` = `contribuicao`). Para cada aresta numa fileira, `c(x)` é `0` à esquerda do
pedaço recortado à fileira `[xa, xb]`, `dy` a partir de `ceil(xb)`, e só varia nos pixels que ele
CRUZA. ⇒ cada aresta deposita, por pixel `x ∈ [floor(xa), ceil(xb))`, `fixo(c(x)) − fixo(c(x − 1))`, e
`fixo(dy) − fixo(c(ceil(xb) − 1))` em `ceil(xb)`; o prefixo dos depósitos dá `Σ fixo(c(e, x))`. Os
depósitos param na célula `kb` (a primeira cujo início `≥ xb`): dali para a frente a aresta é FUNDO,
como hoje. O prefixo RECOMEÇA em cada célula (o primeiro pixel de uma célula recebe `fixo(c(x))`
inteiro, não a diferença), porque o fundo da célula só conta as arestas que acabam ANTES dela. Em
inteiros a soma não depende da ordem dos fios: determinístico, como hoje. ⚠️ O pedaço é recortado à
fileira (uma divisão por aresta, não por fileira): uma aresta paga os pixels que cruza, não `32` por
célula. O resto `~1e-6` da `contribuicao` à esquerda do pedaço recortado (o do Vello) deixa de entrar
— `≤ 1` unidade de `2¹⁶`, dentro do `ALFA_MAX = 2`.

**Os passes** (os quatro despachos indirectos ficam quatro; o `cs_conta`, o `cs_soma` e o `cs_escreve`
ficam, com a unidade das células em CÉLULAS e não em palavras):

| passe | fio por | faz |
|---|---|---|
| `cs_zera` | PIXEL de célula | apaga os três acumuladores do pixel; os fios `0..3` da célula apagam o fundo |
| `cs_deposita` | ARESTA | por fileira: o fundo da célula `kb` (atómico, como hoje) e os depósitos `[floor(xa), ceil(xb)]` nas células `< kb` |
| `cs_fundo` | FILEIRA | o prefixo dos fundos ao longo da fileira (a metade de cima do `cs_lugar_das_listas`) |
| `cs_varre` | PIXEL (grupos de `64` = duas células) | prefixo SEGMENTADO por célula em memória de grupo (`5` passos), mais o fundo, as duas regras (`af` com a da cópia, `as_` com a soma das marcas e do contorno) e grava `pack2x16unorm(af, as_)` |

O fragmento: `(c1.x + r · células) · 32 + (x − x0)`, UMA leitura e `unpack2x16unorm`; as regras saem
dele para esta via (o caminho de sempre continua a aplicá-las).

**Os buffers** (por célula, indexados pelo número GLOBAL da célula — o prefixo das células de cada
cópia já é contíguo): `celulas` `4` palavras (os três fundos, `atomic`) · `acumula` `96` palavras (`3`
famílias × `32` pixels, família a família, para a leitura do `cs_varre` ser coalescida) · `cobertura`
`32` palavras. Saem as `listas` e o `lista_total`. A regra da cópia (`even_odd`) passa ao `cs_escreve`
(`ccopias[3·ii + 2].z`, livre). O `despacho` ganha `[6, 9)`: um fio por pixel de célula.

**A capacidade:** uma só, a das células (contada no `cs_conta`, lida dois quadros depois, como as
arestas). O tecto do recurso é o `acumula` (`max_storage_buffer_binding_size / 384 B`). Uma cópia que
não cabe vai INTEIRA pelo caminho de sempre (o `cs_escreve` já a recusa). ⇒ **não há fileira que não
caiba**: o `SEM_LISTA` e o «dois quadros depois das arestas» deixam de existir, e o regime do produto
volta a medir-se (o `QUADROS_DO_PRODUTO = 5` era o das listas).

**As guardas do §9.8, recontadas:**

- `as_fileiras_que_nao_cabem_nas_listas_desenham_o_mesmo` → **`as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo`**
  (tecto em metade das células pedidas: as que cabem pelas células, as outras pelo caminho de sempre,
  a mesma imagem; controlo: o tecto morde E alguma cópia continua nas células). Porta
  `ShapePass::limita_as_celulas` (sai `limita_as_listas`); instrumento `celulas_do_ultimo_quadro`.
- `uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior` fica — e passa a guardar também o
  `cs_zera` (um acumulador que traz o quadro anterior).
- A mutação `14/14` das listas morre com o código dela: um arnês novo
  (`mutacao_o_buffer_de_acumulacao_2026-10-03.py`) com os análogos — fundo na `kb − 1`, o 1.º pixel
  cruzado perdido, o degrau final perdido, o prefixo que não recomeça na célula, depósitos em `≥ kb`,
  `cs_fundo` sem prefixo, `cs_zera` mudo, varredura não segmentada, `even_odd` ignorado, escala
  dobrada, `min/max` do recorte trocados, sem a verificação da capacidade, passo da fileira errado no
  fragmento, preenchimento e contorno trocados.

**As duas tentativas** (o kill-criterion do §9.11 conta-se depois da 2.ª):

1. a DENSA acima (zera e varre todo pixel de célula);
2. se a 1.ª falhar: a ESPARSA — uma marca por célula TOCADA (o `cs_deposita` a põe); o `cs_varre` só
   corre nas tocadas e devolve-lhes os acumuladores a zero (sai o `cs_zera`), e uma célula sem
   depósitos é constante — o fragmento lê-a do registo. Paga uma leitura encadeada a mais nas bordas.

**Réguas:** `registos_dos_shaders.sh` (o fragmento tem de descer dos `56` VGPRs) · os `10` gates GPU da
crate e os `5` do produto · a sonda intercalada com `PERFIL=1 PLACAS=igpu` (e `rtx`) · a mutação nova
e, no fecho, a `21/21` do tracejado.

**✅ O resultado — a 1.ª tentativa (a DENSA) passa o kill-criterion** (commit `cd3ec059e`; o binário de
`8cb0ab9e1` intercalado na MESMA janela, `PH2D_FLUID_PROFILE=1`, `2` corridas por célula, a soma dos
passes do perfilador, ms):

| arranjo | iGPU cura | iGPU acumulação | critério | RTX cura | RTX acumulação |
|---|---:|---:|---:|---:|---:|
| `72` grandes esticadas | `1,50` · `1,52` | **`0,88`** · `0,98`¹ | `≤ 1,0` ✅ | `0,25` · `0,26` | **`0,22`** · `0,22` |
| `72` grandes conformes | `0,76` · `0,76` | **`0,59`** · `0,59` | `≤ 0,76` ✅ | `0,10` · `0,10` | **`0,09`** · `0,10` |
| `1225` densas da `=127` | `1,50` · `1,50` | **`1,00`** · `1,00` | `≤ 1,50` ✅ | `0,11` · `0,11` | **`0,10`** · `0,10` |

¹ a 2.ª corrida das esticadas caiu numa janela com a média de 5 min a `8,5` (outra linha); a 1.ª a `1,8`.

A decomposição iGPU das esticadas: desenho `0,83 → 0,14` (o fragmento só lê — o piso da ablação era
`0,13`) · células `0,43 → 0,50` · escrita `0,17` e contagem `0,07` iguais. ⇒ **o preço do cálculo novo
é `+0,07 ms` para tirar `0,69` ao desenho.** No relógio de parede da sonda as esticadas ficam em
`0,98` contra `0,94` do Vello (antes `1,64` contra `1,13`): o último arranjo em que o proxy de telemóvel
perdia para o Vello passa a empatar. As conformes ficam `0,70` contra `0,87`; as densas `1,13` contra
`2,65` (`2,3×`).

⚠️ **Onde a densa paga:** nas densas as células sobem `0,51 → 0,78 ms` — a `cs_zera` e a `cs_varre`
correm em TODO pixel de célula, e `1225` cópias sobrepostas são muitos pixels de grelha por pixel de
ecrã. A 2.ª tentativa (a ESPARSA, acima) é exactamente esta alavanca: fica como item medido, não como
pendência do critério. A memória: as esticadas pedem `10 362` células (`~5,5 MB`, `528 B` por célula);
o gate das estrelas sobrepostas a `512²`, `77 502` (`~41 MB`) — o tecto do recurso continua a ser o
`max_storage_buffer_binding_size`, e quem não cabe vai por cópia.

**Registos** (`registos_dos_shaders.sh`, iGPU): o fragmento continua a `56` VGPRs · `18` ondas (o caminho
de sempre está no MESMO shader e manda nos registos), código `16 620 → 15 336 B`; os quatro kernels novos
`≤ 32` VGPRs, nada em scratch.

**Gates** (RTX): `ph2d-shape-gpu` **10/10** — todos os quadros de todas as fixturas com `alfa ≤ 1` e `0`
pixels `> 1` contra o eixo, inclusive o 1.º (a mistura com o caminho de sempre) e a metade das células
(`as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo`: `21/40`, `2/6` e `26/60` cópias nas células, o
resto por cópia, a mesma imagem); o tracejado contra o Vello no mesmo `71` do §9.9 · produto **5/5** +
sonda.

**Mutação `17` de `17`** ([arnês](../../ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py), pré-voo `17/17`,
corrida LIMPA `10` verdes, nenhuma por shader inválido): fundo uma célula depois da `kb` · o 1.º pixel
cruzado perdido · o degrau final até `dy` perdido · o prefixo que não recomeça na célula · `cs_fundo` sem
prefixo · `cs_zera` sem apagar a acumulação · sem apagar o fundo · varredura não segmentada · a regra
par-ímpar ignorada no `cs_varre` · e no `cs_escreve` · escala dobrada · o recorte com o mínimo trocado ·
sem a verificação da capacidade · o passo da fileira no fragmento · preenchimento e contorno trocados ·
a `kb` por `floor` · o depósito sem a diferença. As duas do `cs_zera` morrem no gate da cena que muda
(o análogo da M10 do §9.8) e no da metade das células. O arnês das listas (`14/14`) foi aposentado com
o código que mutava.

**A memória no APP — medida, e o defeito que a medição achou** (W5, `mede_formas_na_placa.sh`, o relato
`[formas] celulas:` sob `PH2D_FLUID_PROFILE=1`, janela `1930 × 1040`). ⛔ A 1.ª corrida pediu `132 MB`
na `=127` densa e `264 MB` na escada de `32 768` — e as «pedidas» eram EXACTAMENTE `16 ×` as cópias: era
o palpite de fábrica por cópia (`CELULAS_POR_COPIA_INICIAL = 16`), que entrava num `max` e nunca mais
saía (o comentário dizia «a leitura do total substitui-o»; o código guardava o maior dos dois). Com as
listas o mesmo palpite custava `~384 B` por cópia; com a acumulação, `8,4 KB`. ✅ **Cura:** as células
só têm a capacidade MEDIDA (os dois primeiros quadros de uma cena nova vão pelo caminho de sempre — a
mesma imagem, os gates `10/10` e `5/5` re-corridos). Re-medido:

| cena (app) | células pedidas | capacidade | quadro iGPU com · sem placa | CPU (encode) com · sem |
|---|---:|---:|---:|---:|
| escada, `4 096` · `16 384` · `32 768` | `0` (as conformes pequenas não vão às células) | — | `60 fps` nas três | `~3,1` · `4,1`–`17,8` |
| `=127` densa, `16 384` | `107 520` (`6,6` por cópia) | `131 072` · **`66 MB`** | **`16,6`** · `20,8` ms | `3,1` · `8,2` ms |

⇒ no proxy de telemóvel a `=127` densa passa a `60 fps` (as listas davam `17,6 ms`). Os `66 MB` são
`528 B` por célula de `32 px` (`16,5 B` por pixel de grelha) e a potência de dois.

⛔ **RECUSA MEDIDA — estreitar a célula para a acumulação** (a hipótese era que a grelha de `32` de uma
estrela de `~14 px` fosse mais de metade fora da forma). Sonda intercalada, `PH2D_FLUID_PROFILE=1`, `2`
corridas, soma dos passes · células · pixels de grelha:

| largura | iGPU esticadas | iGPU conformes | iGPU densas | RTX (os três) | grelha esticadas · densas |
|---|---:|---:|---:|---:|---:|
| **`32`** | **`0,88`** · `0,50` | **`0,59`** · `0,37` | `1,00` · `0,78` | `0,22` · `0,09` · `0,10` | `331 k` · `470 k` px |
| `16` | `0,87` · `0,49` | `0,60` · `0,38` | `1,00` · `0,78` | `0,22` · `0,09` · `0,09` | `301 k` · `470 k` |
| `8` | `0,97`–`1,06` · `0,59`–`0,67` | `0,73` · `0,52` | `0,94` · `0,71` | `0,22` · `0,10` · `0,09` | `294 k` · `353 k` |

⇒ `16` empata (a grelha das densas nem muda: a caixa estimada com a folga já cabe em `16`); `8` só ganha
`0,06 ms` nas densas e perde `0,09`–`0,18` nas esticadas. Fica `32`, como no §9.8. A memória não se
cura pela largura: as alavancas que ficam são a cobertura gravada NO LUGAR do 1.º acumulador
(`−4 B` por pixel, `−24 %`) e um arredondamento da capacidade mais fino que a potência de dois.

✅ **A MEMÓRIA DAS CÉLULAS: `66 → 43 MB` na `=127` densa** (commit `9d1058a40`; kill-criterion escrito
antes: capacidade `≤ 46 MB` no app, e nenhum arranjo pior que `+5 %` na iGPU nem `+10 %` na RTX na soma
dos passes, contra o binário de `fab8999a8` intercalado na MESMA janela). As duas alavancas acima:

1. **A cobertura NO LUGAR do 1.º acumulador.** O `cs_varre` grava o `pack2x16unorm` na palavra
   `acumula[cel·ACUMULA + p]` (o depósito do preenchimento do PRÓPRIO pixel), que ele já leu antes das
   barreiras: nenhum outro fio a lê (o prefixo vive na memória de grupo) e o `cs_zera` do quadro
   seguinte apaga-a com as outras. Sai o buffer `cobertura` (ligação `9` do grupo `2`); a ligação `2`
   do grupo `1` do desenho passa a ser a acumulação, `read` no passe de desenho e `read_write` no de
   cálculo — passes separados, o regime que a `cobertura` já tinha. `528 → 400 B` por célula.
   ⚠️ **O fragmento parte a coluna:** `x` corre a FILEIRA inteira, e com a cobertura contígua
   `primeira·32 + x` caía sozinho na célula `primeira + x/32`; com o passo `ACUMULA` é
   `(primeira + x/32)·ACUMULA + x%32`. O 1.º rascunho (`primeira·ACUMULA + x`) pôs **`7` dos `10`**
   gates vermelhos — virou a mutação A19.
2. **A capacidade ao OITAVO do degrau** (`ao_oitavo_do_degrau` em `contorno.rs`): o múltiplo seguinte
   de `2^(⌊log₂ n⌋ − 3)`, só para as células (crescem só por medição). `107 520 → 114 688` em vez de
   `131 072`. ⚠️ **O preço, medido:** numa cena que cresce UMA célula de cada vez até `107 520` (o pior
   caso: cada medição passa a capacidade) os buffers recriam-se `118` vezes contra `18` — no máximo
   `8` por oitava, com gate (`numa_cena_que_cresce_as_celulas_recriam_se_no_maximo_oito_vezes_por_oitava`).
   Na `=127` do app a contagem não cresce: UMA criação, como antes.

| medida | antes (`fab8999a8`) | depois |
|---|---:|---:|
| app `=127` densa (iGPU e RTX): capacidade | `131 072` · **`66 MB`** | `114 688` · **`43 MB`** (`45,9 × 10⁶ B`) |
| app `=127` densa: quadro iGPU · RTX | `16,6 ms` · `16,7 ms` | `16,6`–`16,8 ms` · `16,6 ms` (`60 fps`) |
| sonda: memória das esticadas · conformes · densas | `8` · `8` · `8 MB` (`16 384`) | `4` · `3` · `5 MB` |
| iGPU esticadas (soma dos passes, ms) | `0,88` · `0,89` | `0,89` · `0,90` (`+1 %`) ✅ |
| iGPU conformes | `0,61` · `0,61` | `0,59` · `0,59` ✅ |
| iGPU densas | `1,01` · `1,01` | `1,01` · `1,01` ✅ |
| RTX esticadas | `0,22` ×6 | `0,22` ×6 ✅ |
| RTX conformes | `0,09` ×5 · `0,10` ×1 | `0,09` ×5 · `0,10` ×1 ✅¹ |
| RTX densas | `0,10` ×6 | `0,10` ×6 ✅ |

¹ o `0,10` aparece UMA vez em cada binário (o de depois na 1.ª janela, o de antes na repetição): é o degrau de `0,01 ms` do perfilador, `11 %` de `0,09`; o `gpu-busy` dá `0,10` igual nos
dois. Cargas: a 1.ª janela com a média de 5 min entre `3,5` e `8,8` (duas outras linhas a testar); a
repetição da RTX (`CORRIDAS=4`) entre `0,4` e `2,4`.

**Registos** (`registos_dos_shaders.sh`, iGPU, os dois binários): o fragmento continua `56` VGPRs · `18`
ondas · `15 336 B` (a partição da coluna em célula e pixel não custa registo); o `cs_varre`
`1 008 → 1 000 B`; nada em scratch. **Gates** (RTX): `ph2d-shape-gpu` `10/10` + `3` unitários novos da
capacidade, produto `5/5` + sonda, `ph2d-gpu-cook` formas `2/2`. **Mutação `19` de `19`** (o
[arnês](../../ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py) com a A13 re-ancorada e duas novas: A18 a
cobertura gravada na palavra das marcas · A19 a coluna sem a partição; pré-voo `19/19`, corrida limpa `10`
verdes, nenhuma por shader inválido) e a do tracejado `21/21`.

### §9.13 — A VARIANTE COMPLETA ENCOLHIDA e a variante ESPARSA medida (2026-10-03, escrito ANTES de construir)

Ordem do dono (03/10, antes de integrar): os itens 3 e 4 do §6 do
[handoff de 03/10](../../handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md). Comparação: o binário da
sonda de `ed23a77b1` (`antes`), com a sonda nova `PH2D_SONDA_TRACEJADO=1` (o `Dash 2` · `Dash Gap 1,5` da
`=127` tracejada, no contorno de cada arranjo) e o instrumento `celulas_tocadas_do_ultimo_quadro`.

**A régua reproduzida, sem relógio** (`registos_dos_shaders.sh`, iGPU, `antes`): fragmento enxuta `56`
VGPRs · `18` ondas · `15 336 B`, completa **`128` · `8`** · `34 428 B`; `cs_escreve` enxuta `64` · `16`,
completa **`128` · `8`** · `28 136 B`; `cs_conta` `40` · `24` e `48` · `20`. Nada em scratch.

**O relógio de partida** (iGPU, `PH2D_FLUID_PROFILE=1`, o regime — a 2.ª janela de `120` quadros; a 1.ª
inclui os dois quadros pixel a pixel de antes da capacidade medida; ms):

| arranjo | conta | escreve | células | desenho | soma | Vello (parede) |
|---|---:|---:|---:|---:|---:|---:|
| `72` esticadas, TRACEJADAS | `0,17` | `0,32` | `0,63` | `0,22` | `1,34` | `0,95` |
| `72` esticadas, contínuas (§9.12) | `0,07` | `0,17` | `0,50` | `0,14` | `0,88` | `0,94` |

⇒ o tracejado não cobra só no desenho: a contagem e a escrita correm UM fio por cópia, e com `72`
cópias são duas ondas na placa inteira — ali a ocupação não decide nada, decide o trabalho EM SÉRIE de
cada fio. A contagem percorre o eixo três vezes (o `ajuste_do_tracejado` com o seu laço interior e o
`arco` de cada troço) e a escrita quatro (o ajuste outra vez, o `sub_tracejado` de cada fechado, o
percurso). O desenho (`0,22` contra `0,14`) é o único dos quatro que é a ocupação: no regime nenhuma
cópia passa pelo pixel a pixel, e mesmo assim o fragmento corre com os registos dele.

**(3a) O fragmento — a placa escolhe a variante por quadro.** O `cs_escreve` é o único que sabe se uma
cópia TRACEJADA vai pelo pixel a pixel (os dois primeiros quadros, a capacidade, a escrita recusada). O
desenho passa a dois `draw_indirect` com os mesmos argumentos — o da ENXUTA e o da COMPLETA — e só um
deles tem cópias: o `cs_soma` põe `n` cópias na enxuta e `0` na completa; uma cópia tracejada, VISÍVEL,
que fica sem células troca-os (atómicos, o mesmo valor de todos os fios: a ordem não importa). Uma só
chamada desenha todas as cópias, logo a ordem da mistura é a de sempre. Os argumentos vivem no buffer do
`despacho` (palavras `[9, 17)`), que já é `INDIRECT`. Sem tracejado carregado, o desenho de hoje.

**Kill-criterion (3a):** (i) sem relógio — no regime da cena tracejada o fragmento que corre é o da
enxuta (`56` · `18`); um instrumento lê os argumentos e um gate o prova, com CONTROLO (o 1.º quadro, sem
capacidade, desenha pela completa); (ii) a imagem igual: os `10` gates GPU da crate, os `5` do produto e
as mutações do tracejado (`21/21`) e da acumulação (`19/19`); (iii) o relógio — sonda intercalada,
`load < 4`, iGPU e RTX: o desenho das tracejadas a `≤ +10 %` do das contínuas do mesmo arranjo, e
nenhum arranjo pior que `+5 %` (iGPU) / `+10 %` (RTX) na soma dos passes. Falhou ⇒ recusa medida aqui.

**(3b) A contagem e a escrita — menos voltas ao eixo por fio.** Dois pedaços, medidos um a um:

1. **o limite da contagem sem o ajuste.** O `limite_de_arestas` só precisa de um TECTO de peças por
   troço, e o ajuste nunca encurta o período mais que meia peça por troço: com `L` o sub-caminho mais
   longo (o que o ajuste fecha), `per` o período da caneta e `len ≤ L` o arco do troço, o período
   ajustado é `per · L / d` com `d ≤ L + per/2` (o arredondamento de `L/per`, fechado ou aberto) — ou,
   quando o arredondamento dá zero, `≥ L ≥ len` (uma peça). ⇒ `len / per_ajustado ≤ len/per + 1/2`,
   e o tecto passa a `⌈len/per + 1/2⌉ + 2` sem o ajuste: a contagem percorre o eixo UMA vez (era
   três). A escrita continua a usar o ajuste exacto; o tecto só reserva;
2. **o ajuste numa volta só.** O `ajuste_do_tracejado` andava cada sub-caminho a partir do seu troço de
   início (o laço interior com o `proximo_troco`), por cima do laço de todos os itens: cada troço lido
   duas vezes. Os troços de um sub-caminho são contíguos (só cabeçalhos de bloco no meio), logo uma
   volta acumula o arco e fecha o sub-caminho quando chega o seguinte.

**Kill-criterion (3b):** (i) a imagem igual — os mesmos gates e mutações do (3a); (ii) o relógio, sonda
intercalada contra o binário do (3a), iGPU, `load < 4`: nas `72` esticadas tracejadas `conta + escreve`
`≤ 0,40 ms` (de `0,49`, `−18 %`), e nenhum arranjo pior que `+5 %` (iGPU) / `+10 %` (RTX) na soma dos
passes; (iii) sem relógio — o `cs_conta` da completa não sobe dos `48` VGPRs nem o `cs_escreve` dos
`128`. ⚠️ Os registos do `cs_escreve` NÃO são alvo deste item: com `72` cópias ele é DUAS ondas na
placa inteira, com `1 225` são `20` para `24` SIMDs — a ocupação não tem o que esconder, e o que conta é
a cadeia em série de cada fio. Pedaço que não mexe o relógio sai (recusa medida aqui).

**✅ O resultado do (3) — os dois kill-criteria PASSAM** (commit `b723b02d1`; os quatro binários
intercalados na MESMA janela — `antes` · `3a` · `3b1` · `3b2` —, `PH2D_FLUID_PROFILE=1`, `2` corridas por
célula, o regime, a soma dos passes, ms; as duas corridas de cada célula iguais ao `0,01` salvo nota):

| arranjo TRACEJADO | iGPU antes | 3a | 3b1 | **3b2** | RTX antes | **3b2** |
|---|---:|---:|---:|---:|---:|---:|
| `72` esticadas | `1,33` | `1,27`–`1,30` | `1,20` | **`1,17`** (`−12 %`) | `0,42`–`0,43` | **`0,37`** (`−13 %`) |
| `72` conformes | `0,61` | `0,55` | `0,55`¹ | **`0,55`** (`−10 %`) | `0,09` | **`0,09`** |
| `1 225` densas | `1,45` | `1,42` | `1,33` | **`1,32`** (`−9 %`) | `0,12` | **`0,12`** |

¹ a 2.ª corrida leu `0,63` numa janela com a média de 1 min a `1,3` e a de 5 a subir (outra linha).

- **(3a)** o desenho das tracejadas `0,22 → 0,15` (esticadas) · `0,21 → 0,15` (conformes — o eixo delas é
  tracejado mesmo sem o usarem, e pagavam a completa) · `0,15 → 0,11` (densas); as MESMAS cenas sem
  tracejado, na mesma janela: `0,15` · `0,15` · `0,11` ⇒ o critério «`≤ +10 %` das contínuas» passa a
  zero. O 1.º e o 2.º quadro de uma cena nova desenham pela completa (o gate), o resto pela enxuta.
- **(3b)** `conta + escreve` nas esticadas `0,49 → 0,40` (o critério: `≤ 0,40`); o `cs_conta` da completa
  `48 · 20 → 40 · 24` (`4 716 → 3 940 B`). ⚠️ **Os relógios POR passe mentem na fronteira:** nas densas a
  «escrita» caiu `0,31 → 0,17` com o (3b1), que só mexeu na contagem — e o `cs_escreve` dos dois
  binários é o MESMO (`128` · `8`, `30 204 B`). O carimbo de início de um passe sai antes da barreira
  que o separa do anterior, e a cauda da contagem caía na conta da escrita. ⇒ a régua é a SOMA; os
  relógios de dois passes vizinhos só valem somados.
- **Sem tracejado** (o controlo, `antes` contra `3b2`, iGPU): `0,89 → 0,88` · `0,59 = 0,59` · `1,01 = 1,01`.
- **Parede da sonda (iGPU)** — as tracejadas esticadas continuam atrás do Vello (`1,57` contra `0,90`):
  as `0,62 ms` de células são as arestas dos traços (as mesmas estrelas contínuas pedem `0,49`) e a
  escrita é UM fio por cópia (`72` fios). O que fica é a topologia da escrita (um fio por troço, com o
  prefixo do arco), não os registos — item próprio, não deste fecho. As densas `1,48` contra `3,25`
  (`2,2×`) e as conformes `0,66` contra `0,85` ganham.
- **No PRODUTO, a cena do report** (`=127` densa tracejada, `16 384` cópias, `1930 × 1040`, rota
  `HIBRIDO` lida no `[motion-route]`, `mede_formas_na_placa.sh` com `PH2D_TRACO_ESTICADO_TRACEJADO=1`,
  binário de `40a2cbc1a`): com a placa **`16,6 ms` (`60 fps`) nas duas placas**; sem ela (o Vello, que
  corta o tracejado na CPU) `80`–`84 ms` (`12 fps`). Células `114 688` = **`43 MB`**, a mesma criação de
  antes. ⇒ a «perda para o Vello» da sonda é contra o Vello com o tracejado JÁ cortado e fora do relógio;
  no produto a alternativa é `5×` mais lenta.

**Mutação `6` de `8`** ([arnês](../../ferramentas/mutacao_a_variante_da_placa_2026-10-03.py), pré-voo `8/8`,
corrida LIMPA `17` verdes): nenhuma cópia pede a completa · a completa sempre (só o gate novo a vê) · o
teste de fora do ecrã invertido · os dois desenhos trocados · o ajuste que não recomeça a soma em cada
sub-caminho (o gate contra o Vello; a estrela com FURO é a única fixtura com dois sub-caminhos) · o
ajuste com um troço a menos —
SANGRAM. ⚠️ **Sobrevivem, e é medido porquê:** o tecto da contagem sem a meia peça (W6) e com METADE das
peças (W7). O orçamento de arestas por peça (`4 + 2 · ponta + junta`) é tão folgado que nem metade das
peças falta em fixtura nenhuma: o tecto exacto fica guardado pela PROVA acima, e um tecto curto não pinta
mal — a escrita recusa a cópia, ela vai pixel a pixel e a placa desenha pela completa (a mesma imagem;
o gate novo veria a completa no regime). A V7 do arnês da variante enxuta (§9.10) foi re-ancorada no
desenho novo.

**O ajuste numa volta é o MESMO ao bit:** as imagens do passe nas `7` famílias do gate do tracejado contra
o Vello, com o `shape.wgsl` de antes e o de depois (o resto igual), são iguais byte a byte.

**(4) A variante ESPARSA — a fracção de células TOCADAS, medida** (instrumento
`celulas_tocadas_do_ultimo_quadro`, o binário do (3b), o último quadro da sonda; cota por baixo):

| arranjo | células em uso | tocadas, contínuo | tocadas, tracejado |
|---|---:|---:|---:|
| `72` grandes esticadas | `10 362` | `6 243` (**`60 %`**) | `5 962` (`58 %`) |
| `72` grandes conformes | `8 916` | `8 640` (`97 %`) | `8 629` (`97 %`) |
| `1 225` densas da `=127` | `14 700` | `8 820` (**`60 %`**) | `8 330` (`57 %`) |
| escada de `32 768` (app, `=17`) | `0` — as conformes pequenas não vão às células (§9.12) | — | — |

⛔ **A premissa do handoff caiu:** nas densas NÃO é «quase toda célula tocada» — `40 %` das células de uma
estrela de `~14 px` numa grelha de `32 px` só têm fundo (a caixa com a folga e a borda da estrela não
enchem a célula). A alavanca existe nas densas e nas esticadas; nas conformes grandes não.

**Kill-criterion (4), escrito ANTES de construir.** A esparsa poupa, no MÁXIMO, o `cs_zera` inteiro e o
`cs_varre` das células não tocadas — `teto = t_zera + (1 − f) · t_varre` — e paga as marcas no
`cs_deposita` (um atómico por célula cruzada), a lista compacta das tocadas, o apagar das tocadas do
quadro ANTERIOR (a cobertura vive no lugar do 1.º acumulador) e, no fragmento, uma leitura encadeada a
mais nas tocadas e as regras aplicadas ao fundo nas outras. ⇒ (i) **sem construir:** mede-se `t_zera` e
`t_varre` por ablação (binários mutilados, fora de commit, como no §9.11 — a soma dos passes com e sem o
despacho; os relógios POR passe não servem, ver abaixo); se o teto nas densas da iGPU for `< 10 %` da
soma dos passes, a variante é RECUSADA aqui com a tabela; (ii) se construída: densas iGPU `−10 %` na
soma dos passes, nenhum arranjo pior que `+5 %` (iGPU) / `+10 %` (RTX), a memória da `=127` densa no app
`≤ 43 MB`, a imagem igual (os gates e as duas mutações) e uma mutação nova a sangrar.

⛔ **RECUSA MEDIDA — a variante ESPARSA das células** (2026-10-04; binários mutilados fora de commit, o
de `b723b02d1` como base, intercalados na MESMA janela, iGPU, `PH2D_FLUID_PROFILE=1`, `2` corridas
iguais ao `0,01`, cargas `0,0`–`2,8`; a soma dos passes, ms):

| arranjo (contínuo) | base | sem `cs_zera` | sem `cs_varre` | `f` tocadas | teto `t_z + (1−f)·t_v` | + marcas e lista | + fragmento marca/fundo | líquido esperado |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `1 225` densas | `1,01` | `0,91` | `0,77` | `60 %` | `0,20` (`19 %`) | `+0,05` | `+0,02` | **`~0,10` (`≤ 10 %`)** |
| `72` esticadas | `0,88` | `0,82` | `0,71` | `60 %` | `0,13` (`15 %`) | `+0,04` | `+0,02` | `~0,05` (`6 %`) |
| `72` conformes | `0,59` | `0,54` | `0,44` | `97 %` | `0,05` (`9 %`) | `+0,05` | `+0,01` | **`−0,02` (pior)** |

O teto passa o filtro (i), por isso os custos fixos foram medidos antes de construir: «+ marcas e lista» =
um `atomicOr` na palavra da regra por célula que a aresta cruza e um `atomicAdd` de compactação na 1.ª
marca; «+ fragmento» = a leitura encadeada da marca e, nas não tocadas, as três palavras do fundo com as
regras. O líquido esperado conta só `77 %` do `cs_zera` (o apagar não sai inteiro: a cobertura do quadro
anterior vive no 1.º acumulador das tocadas e o registo de toda célula continua a apagar-se) e não conta
o despacho que lê o total da lista. ⇒ no MELHOR caso as densas ficam na fronteira dos `10 %`, as
esticadas abaixo e as conformes PIORAM — e a lista das tocadas custa `4 B` por célula (`43 → 44 MB` na
`=127` densa do app), o que viola o critério (ii) por construção. **Fica a densa.** Para quem voltar a
isto: a alavanca que sobra nas células é o `cs_varre` (`0,24 ms` nas densas, um fio por pixel com os cinco
passos do prefixo em memória de grupo), não o apagar.

