# ARQUIVO — parte_4.md (história, 1007 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_4.md`](parte_4.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-352` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §9.17 — UM BLOCO: fechar o aberto do passe de formas e a próxima alavanca do tracejado (2026-10-05, escrito ANTES de construir)

Ordem do dono (`CLAUDE.md` §0.10): (a) as mutações dos pedaços do §9.15 · (b) os `override` dos pedaços: ficam ou
dobram-se · (c) o app (`=127` densa contínua e tracejada, as duas placas, e a memória das arestas) · (d) a próxima
alavanca da emissão tracejada · (e) os registos do `base` e do `F`. Base: `014c93238`. Uma construção, UMA rodada
intercalada (sonda + app no mesmo binário), as decisões, a dobra, o gate batched e as mutações sobre o código final.

**(e) os registos, JÁ medidos sem relógio** (iGPU, `RADV_DEBUG=shaderstats`, a sonda intercalada com uma variante — o
roteiro `registos_dos_shaders.sh` lê a sonda antiga; os números são os mesmos shaders): só dois mudam de `base` a `F`.

| shader (iGPU) | `base` | `F` |
|---|---:|---:|
| `cs_escreve` COMPLETA | `128` VGPRs · `8` ondas · **`30 140` B** · `12` SGPRs derramados | `128` · `8` · **`47 112` B** · **`28`** derramados |
| `cs_conta` COMPLETA | `40` · `24` · `3 964` B | `40` · `24` · `4 384` B |
| `cs_soma` | `1 328` B | `1 156` B |
| os outros `13` | iguais | iguais (nada em scratch em nenhum) |

**O inventário da emissão (d), lido no CÓDIGO DE MÁQUINA** (`RADV_DEBUG=shaders`, o `cs_escreve` completo do `F`): o
laço dos pedaços tem `2 517` instruções de código, com `68` escritas de aresta e, antes de CADA uma, `2` leituras de
derrame (`v_readlane`) e a recarga do descritor do buffer (`s_load_dwordx4` + espera) — `136` leituras de derrame no
laço. ⚠️ A ablação `E3` do §9.15 (sem gravar arestas) tirou só `0,01` ms: as recargas NÃO são o custo, são o sintoma da
pressão. O que dobrou o código é o **A1b**: o 1.º traço do fechado, emitido no FIM, é um SEGUNDO sítio de chamada do
`emite_pedaco`, e o compilador inline-o inteiro duas vezes (`30 → 47` KB, `12 → 28` derramados). As cópias da sonda só
diferem na posição (a onda não diverge): o custo é o COMPRIMENTO do caminho de cada pedaço — `~1,5 µs` por pedaço na
iGPU (`0,19` ms ÷ `~126` pedaços por cópia), muito acima da aritmética do `emite_pedaco`.

**O plano, item a item:**

| item | o que se constrói | régua | kill-criterion (escrito antes) |
|---|---|---|---|
| (b) os pedaços do §9.15 | nada novo: a sonda passa a medir **deixar UM de fora** sobre o `F` (`F−A1a` · `F−A1b` · `F−B1` · `F−B2`), além do `base` | soma por cena, iGPU e RTX | um pedaço DOBRA-SE (`true`, os ramos `false` apagados) se tirá-lo do `F` não melhora nenhuma cena mais de `2 %` na iGPU; se melhora, SAI com o código dele. O `D` e o `c2` são portas de EXECUÇÃO (o dispositivo sem subgrupos; os gates do 1.º quadro) e ficam. **Prova de custo zero:** os registos do `F` dobrado iguais aos do `F` por `override` |
| (d) **L** — o adiado no laço | o 1.º traço do fechado emite-se pelo MESMO sítio de chamada (uma volta a mais do laço dos troços, `k == n`), um `emite_pedaco` só | `escreve` e os registos | `cs_escreve` completo `≤ 35` KB e `≤ 19` SGPRs derramados, **e** `escreve` das esticadas tracejadas da iGPU `−0,02` ms; senão sai |
| (d) **H** — a geometria do troço uma vez | os cantos `a`, `b`, a direcção, a normal e as duas bissectrizes (com o recuo GEOMÉTRICO; a comparação com o recuo de cada pedaço fica no pedaço) calculados por troço, fora do laço dos pedaços; a junta do troço (no máximo uma, §9.15 B2) emitida DEPOIS do laço | `escreve` | `−0,02` ms nas esticadas tracejadas da iGPU; senão sai |
| (d) **P** — as quatro arestas de uma peça de uma vez | um teste de capacidade por peça e a caixa da cópia pelos `4` cantos (os mesmos pontos: `min`/`max` são exactos) em vez de por aresta | `escreve` | `−0,02` ms nas esticadas tracejadas da iGPU; senão sai |
| (d) **G32** — grupos de `32` no `cs_conta` e no `cs_escreve` | `@workgroup_size` por `override` (e o despacho do Rust pela mesma constante): a iGPU corre ondas de `64` em duas metades, e uma onda com metade das pistas vazias pode saltar a metade | `conta + escreve` | `−0,02` ms nas esticadas tracejadas da iGPU e nenhuma cena (densas incluídas: `1 225` cópias) pior que `+5 %`; senão sai. ⚠️ Previsão: se a placa não salta a metade vazia, zero — e sai |
| (d) **E1F** (ablação, fora de commit) | o `F` sem emitir pedaços | `escreve` | sem critério: a fatia da emissão no `F` (no §9.15 era `0,19` dos `0,40` sobre o `base`) |
| (c) o app | `[formas] arestas: capacidade {cap} ({MB}) para {reservadas} reservadas por {n} copias` no `garante` (que MUDA-SE para `contorno_capacidade.rs`: `contorno.rs` está em `695`/`700`); uma porta de medição `PH2D_FORMAS_CONSTANTES=NOME=v,…` (lida ao criar a placa no app, só nomes da lista exportada pela crate) para o A/B no MESMO binário; `mede_formas_na_placa.sh` SEM a espera de calma (a carga vai ao lado, `CLAUDE.md` §2) | `[frame]` (as três últimas janelas de `120` quadros), `[motion-route]`, `[formas]` | sem critério de velocidade (é a confirmação no produto); a memória das arestas da `=127` densa tracejada `base → F` medida e escrita, e a do `F` com os aceites do (d) |
| (a) as mutações | `mutacao_o_bloco_do_9_15_2026-10-05.py`, no padrão do de 04/10 (pré-voo, corrida LIMPA verde nas duas placas, restauro + `touch`), SOZINHO na árvore depois do gate | os `14` gates de `ph2d-shape-gpu --test it --ignored` | cada mutação SANGRA; uma que sobrevive é um gate por escrever, nesta sessão. As previstas: `m1` o ajuste da contagem a `1` · `m2` o traço adiado nunca emitido · `m3` o prefixo das escritas deslocado de um bloco · `m4` a reserva sem as pontas (`pecas · 4 + junta`) · `m5` o c2 desligado e a cena nova sem `medir_ja` · `m6` a correção da fronteira da célula no subgrupo (só sangra com subgrupos de `64`: iGPU) — mais uma por pedaço aceite do (d) |

**A rodada (uma só):** a sonda intercalada (`mede_intercalado.sh`, as duas placas) com `base` · `F` · `F−A1a` · `F−A1b`
· `F−B1` · `F−B2` · `E1F` · `L` · `H` · `P` · `G32` · `LHPG` (os quatro), a passar a imprimir `conta` e `escreve` À
PARTE e as arestas reservadas · escritas de cada variante; e o app (`mede_formas_na_placa.sh`, `release`): `=127` densa
contínua e tracejada × `base` · `F` · `LHPG` × as duas placas. Depois da tabela: os pedaços recusados saem com o código
deles, os aceites dobram-se, e o `variantes()` da sonda fica com o que sobrar de porta de medição. Imagens iguais: os
`14` gates nas duas placas, os do produto (`motion_shape_placa`) e o arnês do tracejado contra o Vello.

**Série justificada (§0.10), escrita ANTES da 2.ª medição:** na rodada (`3609f28d2`) o **G32** falhou o critério dele
nas tracejadas (`conta + escreve` `−0,016` ms, pedia `−0,02`) e SAI daí; mas nas esticadas CONTÍNUAS da iGPU a soma
desceu `0,858 → 0,817` (`−4,8 %`, mínimo e mediana juntos), que não era o alvo dele. ⇒ uma confirmação curta, só `F` e
`G32`, `11` rodadas, as `6` cenas nas duas placas. **Critério:** o `G32` fica (o grupo passa a `32` no `cs_conta` e
no `cs_escreve`) se a soma das esticadas contínuas da iGPU descer `≥ 3 %` OUTRA VEZ, e nenhuma cena piorar mais que
`+2 %` na iGPU nem `+5 %` na RTX; senão sai com as outras.

**O resultado da rodada (05/10, `3609f28d2`, `mede_intercalado.sh`: `13` variantes × `6` cenas, `7` rodadas de blocos
de `20` quadros; iGPU `18,6` s, RTX `73` s; carga `8`–`13`, sem espera).** Soma mínima, ms; `conta` e `escreve` À PARTE
(os dois passes da escrita); `=F` ⇒ a camada IGUAL à do `F` byte a byte (o controlo: as ablações diferem).

| iGPU | `base` | `F` | `F−A1a` | `F−A1b` | `F−B1` | `F−B2` | `E1F` | `E4F` | `L` | `H` | `P` | `G32` | `LHPG` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| esticadas contínuas | `0,907` | `0,858` | `0,855` | `0,857` | `0,879` | `0,858` | `0,859` | `0,856` | `0,860` | `0,861` | `0,859` | `0,817` | `0,846` |
| esticadas TRACEJADAS | `1,198` | **`0,964`** | `0,991` | `0,996` | `1,015` | `0,966` | `0,634` | `0,693` | `0,959` | `0,970` | `0,958` | `0,952` | `0,956` |
| — `escreve` | `0,310` | **`0,244`** | `0,293` | `0,277` | `0,229` | `0,244` | **`0,084`** | **`0,150`** | `0,239` | `0,254` | `0,238` | `0,230` | `0,233` |
| conformes contínuas | `0,626` | `0,627` | `0,622` | `0,627` | `0,613` | `0,625` | `0,620` | `0,623` | `0,621` | `0,622` | `0,627` | `0,610` | `0,610` |
| conformes tracejadas ⚠️ | `0,588` | `0,594` | `0,579` | `0,594` | `0,545` | `0,576` | `0,583` | `0,564` | `0,580` | `0,582` | `0,562` | `0,587` | `0,559` |
| densas contínuas | `1,028` | `0,873` | `0,872` | `0,874` | `0,967` | `0,874` | `0,884` | `0,872` | `0,872` | `0,867` | `0,882` | `0,875` | `0,880` |
| densas TRACEJADAS | `1,331` | **`0,963`** | `0,978` | `0,970` | `1,119` | `0,990` | `0,576` | `0,750` | `0,964` | `0,965` | `0,961` | `0,965` | `0,965` |

| RTX | `base` | `F` | `F−A1a` | `F−A1b` | `F−B1` | `F−B2` | `L` | `H` | `P` | `G32` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| esticadas contínuas | `0,231` | `0,225` | `0,224` | `0,223` | `0,222` | `0,223` | `0,223` | `0,223` | `0,223` | `0,218` |
| esticadas TRACEJADAS | `0,370` | **`0,302`** | `0,337` | `0,340` | `0,304` | `0,302` | `0,309` | `0,316` | `0,301` | `0,299` |
| conformes contínuas · tracejadas | `0,088` · `0,083` | `0,084` · `0,080` | `0,083` · `0,080` | `0,083` · `0,079` | `0,081` · `0,076` | `0,083` · `0,078` | `0,084` · `0,080` | `0,084` · `0,078` | `0,084` · `0,080` | `0,081` · `0,075` |
| densas contínuas · TRACEJADAS | `0,098` · `0,120` | `0,085` · `0,096` | `0,084` · `0,098` | `0,085` · `0,098` | `0,085` · `0,099` | `0,085` · `0,097` | `0,085` · `0,097` | `0,084` · `0,097` | `0,085` · `0,096` | `0,086` · `0,096` |

⚠️ As conformes tracejadas da iGPU NÃO decidem nada: nenhuma cópia delas usa a emissão (`0` arestas do contorno) e
o mínimo varia `±0,03` entre variantes que lhes são idênticas, com a mediana a `+0,1` do mínimo. Arestas reservadas ·
escritas (iguais nas duas placas): esticadas tracejadas `561 600` (`base`, `F−B2`) → `285 120` · `36 288`; densas
tracejadas `607 600` → **`343 000`** · `127 400`.

**Os veredictos, contra os critérios escritos antes:**

- **(b) os pedaços do §9.15 DOBRAM-SE, os quatro.** Tirar o `A1a` piora as esticadas tracejadas `+2,8 %` na iGPU e
  `+11,6 %` na RTX; o `A1b` `+3,3 %` e `+12,6 %`; o `B2` as densas tracejadas `+2,8 %` (e a reserva volta a
  `561 600`/`607 600`). ⚠️ **O critério escrito estava mal posto para o `B1`:** à letra ele o tirava — sem ele as
  conformes contínuas descem `2,2 %` (`0,627 → 0,613`: o `cs_soma_escritas` de um grupo, `+0,015` ms num passe onde
  reserva = escritas). Mas sem ele as densas tracejadas sobem `+16 %`, as densas contínuas `+11 %`, as esticadas
  tracejadas `+5 %`: o critério era cego à TROCA entre cenas. Fica o do §9.15 que o aceitou (nenhum arranjo pior que
  `+5 %`), e a troca fica escrita. O `D` e o `c2` são portas de execução e ficam. A dobra apaga os ramos `false` (código
  velho vivo) e as constantes saem da lista da porta (`CONSTANTES_DO_CONTORNO`), que fica para o próximo candidato.
- ⚠️ **Achado (sem acção): o `A1b` muda `183` B da camada nas densas tracejadas da RTX** (o `base` e o `F−A1b` diferem
  do `F` pelos mesmos `183` B; na iGPU é igual). O total de um fechado somado no percurso e somado à parte é a MESMA
  soma termo a termo, mas o compilador da NVIDIA funde-a de outra maneira: numa estrela onde o total cai no limite da
  emenda (`a_fim < tot < a_fim + tr`) a decisão vira — um traço de `~46` px. As duas leituras estão no limite do
  arredondamento; os gates contra o Vello passam nas duas.
- **(d) os quatro candidatos SAEM:** `escreve` das esticadas tracejadas da iGPU `F 0,244` → `L 0,239` (`−0,005`) ·
  `H 0,254` (`+0,010`) · `P 0,238` (`−0,006`) · `G32 0,230` (`−0,014`) · os quatro `0,233` — nenhum chega aos `−0,02`.
  As ablações partem a emissão (`0,160` ms dos `0,244`): **andar o laço dos pedaços** (`E4F − E1F`) `0,066`, **a
  geometria e as arestas** `0,094`; na RTX o laço não custa nada (`0,088` contra `0,086`) e a geometria `0,072`.
  **Os registos não são o custo:** o `L` leva o `cs_escreve` completo de `47 112` a `32 776` B (`26` derramados), os
  quatro juntos a `112` VGPRs, **zero** derrames e `28 916` B — e o tempo fica onde estava: com `72` cópias em DUAS
  ondas não há ocupação a ganhar, e o caminho de cada pedaço já não encurta por arrumação. ⇒ a alavanca que resta é o
  PARALELISMO dentro de uma cópia — a topologia recusada (o GRUPO por cópia, §9.14) —; esta onda não a reconstrói.
- **G32 na série justificada (`353156e55`, só `F` e `G32`, `11` rodadas):** esticadas contínuas da iGPU `0,883 →
  0,862` (**`−2,4 %`**, o critério pedia `≥ 3 %`), RTX `−1,6 %`; nenhuma cena pior. ⇒ **SAI.** A diferença real e
  repetida é a do `escreve` (`−0,014`–`−0,016` ms na iGPU — compatível com a placa a saltar a metade vazia de uma onda
  de `64`, hipótese NÃO medida), pequena demais para o critério.
- **(e) registos ✅** (a tabela do início desta secção); `registos_dos_shaders.sh` não muda.
- **(c) o app ✅** (`mede_formas_na_placa.sh`, `release` de `3609f28d2`, sem espera de calma, carga `2`–`5`; a
  `=127` densa: `16 384` cópias «pela PLACA (do dispositivo)»; `sem915` = os quatro pedaços do §9.15 desligados pela
  porta `PH2D_FORMAS_CONSTANTES`, o MESMO binário):

  | `=127` densa | arestas reservadas | capacidade das arestas | células | parede (3 janelas de `120`) |
  |---|---:|---:|---:|---:|
  | contínua, `F` e `sem915` (as duas placas) | `1 235 968` | `2 097 152` (**`32` MB**) | `43` MB | `16,6`–`16,8` ms (`60` fps) |
  | TRACEJADA, `sem915` (as duas placas) | `4 507 648` | `8 388 608` (**`128` MB**) | `43` MB | `16,6`–`16,7` ms |
  | TRACEJADA, `F` (as duas placas) | **`2 544 640`** | **`4 194 304` (`64` MB)** | `43` MB | `16,6`–`16,8` ms |

  ⇒ o `B2` corta a memória das arestas da cena tracejada do app a METADE (`128 → 64` MB). A parede está presa ao
  ecrã (`60` fps em todas as células): no app ela não separa as variantes — a régua delas é a sonda.
- ⛔ **Na RTX a sonda com `13` variantes voltou a morrer com SIGSEGV DEPOIS do `test result: ok`**, mesmo com
  `__GL_SHADER_DISK_CACHE=0` (com `2` variantes saiu limpa). Os dados estão completos (a tabela sai antes); o roteiro
  passa a aceitar a saída quando o `test result: ok` está lá e a dizê-lo alto.

**A dobra (`df46623f5`):** os quatro pedaços `true` com os ramos `false` apagados; os candidatos e as ablações saem
com o código deles (o `shape.wgsl` volta ao de `64b08daab`); sai também o recurso `ajuste == 0 ⇒ calcula-o aqui` do
`percorre`, que a contagem tornou morto e nenhum `override` apagava. **Registos do `F` dobrado contra o `F` por
`override` (iGPU): `16` dos `17` shaders IGUAIS** (a porta não custava nada); o `cs_escreve` completo `47 112 → 46 216`
B, `28 → 26` SGPRs derramados — o recurso morto. Sonda intercalada sobre o dobrado (`F`, `F-D`, `F-c2`): todas as
imagens iguais ao `F`; esticadas tracejadas `0,940` ms iGPU · `0,294` RTX. `contorno.rs` `695 → 624` linhas (o
`garante` mudou-se para `contorno_capacidade.rs`, `73 → 144`). A sonda fica com `F` · `F-D` · `F-c2`; a lista da
porta (`CONSTANTES_DO_CONTORNO`) fica vazia, à espera do próximo candidato.

**(a) as mutações ✅ `7` de `7` SANGRARAM** ([`mutacao_o_bloco_do_9_15_2026-10-05.py`](../../ferramentas/mutacao_o_bloco_do_9_15_2026-10-05.py),
sozinho na árvore depois do gate; pré-voo `7/7`; corrida limpa `14/14` nas duas placas):

| mutação | placa | reprovou |
|---|---|---|
| `m1` o ajuste da contagem a `1` | RTX | `3`: os dois do tracejado contra o Vello e o pixel, a escolha da variante |
| `m2` o traço adiado do fechado nunca emitido | RTX | os mesmos `3` |
| `m3` o prefixo das escritas deslocado de um bloco | RTX | `7` (paridade com o Vello, contorno calculado, tracejado) |
| `m4` a reserva sem as pontas | RTX | `5` (o tracejado e a cena nova: cópias recusadas por falta de reserva) |
| `m5a` o c2 desligado | RTX | `a_cena_nova_mede_a_capacidade_antes_do_primeiro_quadro` |
| `m5b` a cena nova sem `medir_ja` | RTX | o mesmo |
| `m6` o subgrupo sem a correcção da fronteira da célula | **iGPU** (subgrupo `64`) | `9` |

**Fecho:** gate batched sobre `df46623f5` (base `5d596eaaf`) verde — `nextest-impacted` `15 713/15 713`, `check
--workspace --all-targets` com `-D warnings`, clippy `--all-targets --all-features` das três crates, `fmt`, `machete`,
`check-standalone-optional`, `check-workflow-packages`, `ph2d-shape-gpu --test it --ignored` `14/14` nas duas placas,
`motion_shape_placa` `5/5` nas duas, `ph2d-gpu-cook formas` `2/2` nas duas, o tecto de LOC, censos `114/114`.

### §9.18 — O QUE O §9.17 DEIXOU: a emissão por PEÇA, a mordida e o SIGSEGV da RTX (2026-10-05, escrito ANTES de construir)

Ordem do dono (05/10, depois do smoke do §9.17): *«precisamos resolver tudo»*.

**(E) A emissão tracejada por PEÇA, em paralelo — ⚠️ é a 3.ª topologia da escrita** (um fio por cópia, §9.5; o
GRUPO por cópia, §9.14, recusado) ⇒ **regra two-strikes (DIRETIVA §5): PROVAR O MODELO antes de construir.**
O modelo: hoje o `escreve` das esticadas tracejadas (iGPU, `0,244` ms) é o passeio em série de cada cópia
(`E1F`: `0,084`) mais a emissão em série (`0,160`: `0,066` só de andar o laço dos pedaços, `0,094` de geometria).
O GRUPO perdia por emitir DUAS vezes, pela fase em série do fio `0` e por `13` KB de memória de grupo — nenhuma das
três existe aqui: (1) o passeio de cada cópia anda só os TROÇOS (o `arco`, o `s0`, `n0..n1`) e escreve uma linha
por troço numa tabela; (2) um prefixo das peças por troço; (3) um passe com UM FIO POR PEÇA (`~9 000` nas
esticadas, `~32 000` nas densas) acha o seu troço por busca binária no prefixo (o desenho do `aresta_de`) e emite
nos sítios da reserva da peça. **Prova antes de construir (sem relógio de produto, ablação):** o passeio por troço
+ a tabela + o passe por peça a fazer só o `pedaco` (sem arestas) ≤ `0,13` ms no `escreve` das esticadas
tracejadas da iGPU; acima disso o modelo cai e (E) fecha por recusa medida. **Kill-criterion do produto:** `escreve`
`≤ 0,15` (de `0,244`) e soma `≤ 0,86` (de `0,94`) nas esticadas tracejadas da iGPU, nenhuma cena pior que `+5 %`
iGPU / `+10 %` RTX, a imagem IGUAL à do `F` byte a byte (o controlo da sonda) e os `14` gates, a reserva igual.

**(C) A mordida do traço rente — a PLACA é a lei** (a união verdadeira); quem morde é o traçador do Vello
(`kurbo::stroke`: a junta interior passa pelo pivô e o pedaço curto cruza-se, enrolamento `0`). A cura é na rota
Vello do Motion (`motion_shape_gen::encode`): o contorno TRACEJADO expande-se pela MESMA lei da placa — peças,
juntas e pontas como sub-caminhos com o mesmo sentido, preenchidos `nonzero` — em vez do `kurbo::stroke`. O
traçador do módulo Vector NÃO muda (outra linha; o defeito lá é do `kurbo`). **Kill-criterion:** uma família
nova no arnês do tracejado contra o Vello com pedaços rentes depois de uma quina e PONTA REDONDA (hoje evitada com
ponta quadrada): rota Vello do Motion = placa (alfa `≤ 1`); as `7` famílias de hoje não pioram; o `encode` da
`=127` tracejada não fica mais de `10 %` mais caro na CPU.

**(D) O contacto dos colisores na placa — a pergunta JÁ foi medida e RECUSADA, com prazo de validade.** O doc 115
§9.5/§11 recusou o contacto no dispositivo À POPULAÇÃO DO DONO (centenas de objectos: a separação na CPU custa
`12,4 %` de um quadro a `500` objectos e `8` varreduras) e escreveu quando a recusa EXPIRA: *«um duplicador que
multiplique um objecto com colisor em milhares»*. Este doc pôs as formas aos milhares na placa (a `=127`: `16 384`)
— e uma forma com `Collide` ainda derruba o cozimento INTEIRO para a CPU (`motion_bridge_gpu_colisor.rs`; o kernel
de dispositivo de hoje só separa DISCOS, a CPU honra a CAIXA). ⇒ **o 1.º passo é MEDIR se a recusa expirou, não
construir:** a `=127` com `Collide` ligado na forma (caixa), `1 024` · `4 096` · `16 384` cópias, as duas placas,
o quadro inteiro pela régua do app (`mede_formas_na_placa.sh`) e o custo do cozimento da CPU (`[frame] MOTION`).
**Critério:** se a `4 096` o quadro passa de `16,7` ms (`60` fps) na iGPU, a recusa expirou e abre-se a wave do
contacto da CAIXA no dispositivo (o solver de `ph2d-contact` em WGSL, paridade por passo contra a CPU, as cercas do
`motion_bridge_gpu_colisor.rs` a cair só para o que o kernel honra); senão a recusa renova-se com o número novo.

**(D) — o resultado: a recusa RENOVA-SE até aos milhares, com o ponto de expiração novo.** A separação na CPU
(`ph2d-contact`, `custo_da_separacao_aos_milhares`, perfil `smoke`, carga `2,2`, caixas orientadas na densidade de
uma cena, mínimo de `5`):

| peças | `8` varreduras (o PRODUTO, `sim.step`) | % de um quadro | `32` varreduras | % |
|---|---:|---:|---:|---:|
| `1 024` | `0,505` ms | `3,0 %` | `2,162` ms | `13,0 %` |
| `4 096` | `1,548` ms | **`9,3 %`** | `5,524` ms | `33,1 %` |
| `16 384` | `5,778` ms | **`34,7 %`** | `20,078` ms | `120,4 %` |

⇒ a `4 096` peças com colisor o contacto na CPU cabe com folga num quadro: a recusa do doc 115 vale até aos
milhares; o ponto em que ela expira passa a ser **`16 384`** (um terço do quadro nesta CPU de secretária — e o dobro,
`8` varreduras de `32`, se uma cena pedir a convergência da corda). ⛔⛔ **E a tabela do doc 115 §9.3 lia `~8×` a
mais** (`1 000` peças, `32` varreduras: `18,058` ms lá, `2,162` aqui): foi tirada com a máquina a `load 45`–`91` (o
cabeçalho da sonda di-lo) — *o mínimo sob carga continua a ser um tecto, nunca o custo*. A medição do quadro INTEIRO
no app (com Play: a `=114` ganhou `PH2D_PILHA_LADO`/`PH2D_PILHA_COLIDE`) fica por correr — o roteiro de foto não carrega
em Play.

**(F) O SIGSEGV/SIGABRT da RTX ao sair — a causa MEDIDA e a cura.** As pilhas (`coredumpctl`, `3` despejos): uma
thread do compilador da NVIDIA (`libnvidia-gpucomp`, `_nv002nvvm`) ainda compila quando a principal está no `exit`, e
o destrutor do `libGLX_nvidia` desmonta o driver por baixo dela. Depende da carga (`3/3` limpas com a máquina calma).
O nosso papel: o perfilador (`pass_profiler`, um `static OnceLock` que emprestava `&'static QuerySet`) segurava o
dispositivo até ao `exit`, e o `vkDestroyDevice` nunca corria antes do destrutor do driver. **O experimento (a mesma
sonda, `13` variantes, RTX, carga `8`–`67`):** com o perfilador `4/6` falhas (incluindo `1/3` com as `13` variantes
IGUAIS do controlo A/A, `PH2D_SONDA_AA`), sem ele **`0/9`**. **A cura:** o perfilador vive num
`Mutex<Option<Arc<…>>>`, entrega a cada passe uma cópia DONA do `QuerySet` (`PassTimestamps`, `.compute()` /
`.render()`) em vez do empréstimo `'static`, e ganha `shutdown()` — que as sondas chamam no fim (o gate
`pass_profiler_gpu` prova que ele solta). Os `15` sítios instrumentados (`13` em `ph2d-render`, `2` em
`ph2d-shape-gpu`) passam a `compute_writes(..).as_ref().map(PassTimestamps::compute)`. ⚠️ A shell chama o `init`
com `PH2D_FLUID_PROFILE=1` e não o `shutdown` (a catraca da shell só desce; as corridas de perfil do app acabam
mortas pelo roteiro, não pelo `exit`).

⚠️ **A prova DIRECTA da cura sob carga está por fazer, e digo porquê.** Com a máquina calma as duas versões passam
(`6/6` a cura, `3/3` a antiga); a carga sintética dentro da fatia da linha (`≤ 50 %`, load `7`–`11`) não chegou à
condição; e a disputa presa aos núcleos `30`–`31` (a sonda e `4` laços nos mesmos núcleos até ela sair) deu `0/4`
falhas MESMO sem a cura — as threads do compilador do driver não ficam presas à afinidade, ou a carga que conta é a
da máquina inteira. A evidência que fica é o experimento `com × sem perfilador` (`4/6` contra `0/9`, carga natural),
e a cura retira exactamente o que ele isolou. A rede de segurança continua: o `mede_intercalado.sh` aceita e DIZ uma
saída `≠ 0` depois do `test result: ok`. Quem vir a próxima falha com a cura no binário reabre isto com o despejo.

**(E) — o resultado: a PROVA do modelo cai, e o (E) fecha por recusa medida.** O código medido é o `afa0cc014`
(`override POR_PECA`: `1` a prova · `2` o produto com a reserva atómica por aresta · `3` o produto contado e
reservado de uma vez · `4`–`6` as ablações da prova), retirado em `ce2fab7ad`. A rodada intercalada
(`mede_intercalado.sh`, `7` variantes com o A/A `F1` × `6` cenas, `7` rodadas, carga `20`–`33`, sem espera) e a das
ablações (`11` rodadas, `3` cenas). `escreve` mínimo, ms, as esticadas TRACEJADAS:

| variante | iGPU | RTX |
|---|---:|---:|
| `F` (o produto) | `0,239` | `0,158` |
| `P1` **a prova**: o passeio por troço + a tabela + o `pedaco` por peça, SEM arestas | **`0,183`** (critério `≤ 0,13`) | `0,110` |
| `P2` o produto, uma reserva atómica por aresta | `0,297` | `0,115` |
| `P3` o produto, contado e reservado de uma vez por peça | `0,275` | `0,112` |

O produto, na soma: iGPU esticadas tracejadas `+7,2 %` (`P2`) · `+3,0 %` (`P3`), densas tracejadas `+20,2 %` · `+14,9 %`
(`escreve` `0,364` · `0,317` contra `0,147`); RTX esticadas tracejadas `−14 %` · `−15 %`, densas tracejadas `+12 %` ·
`+10 %` — e a imagem das densas da RTX DIFERE do `F` em `183 B` (a mesma fusão do compilador da NVIDIA do achado `A1b`,
§9.17); nas outras células `P2`/`P3` dão a imagem `= F` byte a byte. As conformes tracejadas pagam `+0,014` ms: os dois
despachos que correm vazios.

**A decomposição da prova** (iGPU, esticadas tracejadas, `escreve` mínimo de `11` rodadas; densas entre parênteses):

| ablação | `escreve` | o que acrescenta |
|---|---:|---|
| `P6` só o passeio por troço, sem escrever as linhas (+ os dois despachos vazios: `0,014`) | `0,144` (`0,083`) | o passeio ≈ **`0,130`** |
| `P4` + as linhas da tabela; o fio da peça sai logo | `0,165` (`0,099`) | `+0,021` |
| `P5` + as duas buscas binárias e as leituras | `0,184` (`0,130`) | `+0,019` |
| `P1` + o `pedaco` | `0,186` (`0,146`) | `+0,002` |

⇒ **o chão é o PASSEIO em série de cada cópia pelos troços** (o arco de cada um somado por ordem, o `n0`/`n1`, o
prefixo das peças): sozinho ele gasta o limite inteiro do modelo; o `pedaco` por peça custa `0,002`. ⚠️ A premissa do
plano — *«o passeio é o `E1F`, `0,084`»* — não se confirmou: o passeio que GUARDA o que anda mede `0,130`; o `E1F` não
guardava nada (hipótese não medida: o compilador cortou a aritmética que ninguém lia). **Quem voltar:** a única
alavanca que resta é tirar o passeio da série — um fio por troço com o arco e um prefixo SEGMENTADO por sub-caminho (a
4.ª topologia, two-strikes outra vez). ⚠️ Um prefixo paralelo soma noutra ordem: a imagem deixa de ser a do `F` ao bit
(as decisões de limite e de emenda podem virar, §9.17 `A1b`), e o critério da imagem teria de ser outro (alfa `≤ 1`
contra o `F`, escrito antes); e a troca RTX (`−15 %` nas esticadas, `+10 %` nas densas) tem de vir escrita antes.

**(C) — o resultado: CURADA, e o defeito não estava onde o plano o punha.** ⛔ O traçador da CASA (`kurbo`,
`expand_stroke` — o das marcas de uma cópia CONFORME na placa) **não morde**: varrido no CPU (`contorno_cpu_tests.rs`,
um L com o 1.º traço a acabar depois da quina, quina de `30°` a `170°` × `0,05`–`0,8` larguras além dela × pontas
redonda, rente e quadrada) dá os MESMOS pontos pintados que a lei da placa. Quem morde é o traçador do **Vello** (o
traço expandido na placa gráfica), e só na rota Vello. ⇒ as marcas conformes da placa ficam como estavam, e a cura é
toda na rota Vello do Motion, que passa a PREENCHER (`nonzero`, com o pincel do traço) os polígonos que a placa desenha
para a MESMA cópia, no MESMO nível de aplanamento que ela escolhe:

- **a porta CPU da lei** (`ph2d-shape-gpu/src/contorno_cpu.rs`): o `percorre` do `contorno.wgsl` função a função em
  `f32` — o quadrilátero com a faixa na bissectriz, a junta de quem chega, os leques, as pontas, o tracejado andado
  pelo arco no ecrã com o ajuste e a emenda — escrito como POLÍGONOS fechados com o sentido de cada peça da placa (as
  arestas que a placa não escreve cancelam-se aos pares); e o `copia_de` (o nível e a conformidade, `nivel_da_copia`;
  a caneta, `caneta_de`). O espaço de saída é o ECRÃ (as constantes em pixels da placa: a flecha `0,25` dos leques, a
  folga `0,1` da faixa) — uma cópia conforme não passa por ela, usa as marcas;
- **as portas da geometria** (`geometry.rs`): `tolerancia`, `extensao`, `eixo_do_nivel`, `contorno_conforme` — o
  `prepare` passa a usá-las (as mesmas chamadas pela mesma ordem: os `14` gates da crate dão o mesmo);
- **o gancho** (`ph2d-vec-render`, `draw_shared_instances_com_traco` + `TracoProprio`): a porta de lote do Motion com
  «quem desenha o traço»; as portas de sempre passam `None`, byte a byte;
- **o traço pela lei** (`ph2d-app-motion/src/motion_shape_traco.rs`, `TracoDaPlaca`, uma por `encode`): uma forma com UM
  traço, sem marcadores, com o tracejado que o eixo exprime → cópia conforme: o `contorno_conforme` do nível (as marcas
  da placa) pelo afim da cópia; cópia esticada: o `contorno_do_eixo` no ecrã. O resto continua no traçador da casa.

O arnês do tracejado contra a placa (`motion_shape_placa_gpu_tracejado_tests.rs`), as DUAS placas iguais:

| | rota Vello × placa: alfa máx. · px `> 16` |
|---|---|
| **família nova** (`a_rota_vello_traceja_o_pedaco_rente_como_a_placa`: estrelas de arestas rectas com traços de `2` larguras e vão `1`, ponta REDONDA e ponta rente; uma estrela ARREDONDADA — o nível de aplanamento entra — e uma engrenagem com FURO — a emenda do contorno mais curto entra —, as duas de preenchimento transparente; `160` cópias, `54` conformes) — a rota de ANTES (o controlo) | `173` · `6 333` |
| a mesma família — **pela lei** | **`1`** · `0` |
| as `4` famílias de hoje (`a_rota_da_placa_traceja_o_esticado_como_a_casa`) — antes | `85` · `2 142` |
| as mesmas — pela lei | `46` · `165` (o resto é o aplanamento das curvas do PREENCHIMENTO, §6) |

O custo, a sonda intercalada no mesmo processo (`custo_do_encode_tracejado`, a rota pela lei contra a de antes, mínimo
de `15`/`5` rodadas, carga `16`–`46`):

| a `=127` tracejada | `encode` (CPU) | parede do Vello RTX | parede do Vello iGPU |
|---|---:|---:|---:|
| densa `35 × 35` (o arranjo da sonda) | `10,86 → 1,85` ms (**`−83 %`**) | `0,58 → 0,62` (`+7 %`) | `3,33 → 4,17` (`+25 %`) |
| `16 384` cópias | `146,5 → 25,3` ms (**`−83 %`**) | `4,39 → 6,33` (`+44 %`) | `15,5 → 23,3` (`+50 %`) |

⇒ o critério do `encode` (`≤ +10 %`) passa com `−83 %`. ⚠️ **A troca, escrita:** o Vello preenche mais caminhos do que
expandia traços, e a parede dele sobe; o quadro da rota Vello (`encode` + parede) fica `58`–`79 %` mais curto (iGPU
densa: `14,2 → 6,0` ms; `16 384` cópias: `161,9 → 48,5` ms). A rota Vello é a da CPU sem a placa
(`PH2D_FORMAS_NA_PLACA=0`) e a de um quadro que a placa recusa inteiro.

**(C) — as mutações ✅ `12` de `12` SANGRARAM** ([`mutacao_a_mordida_do_traco_rente_2026-10-05.py`](../../ferramentas/mutacao_a_mordida_do_traco_rente_2026-10-05.py),
sozinho na árvore, pré-voo `12/12`, corrida limpa verde nas duas placas; reprovam a família nova — alfa entre parênteses):
`m1` a cura desligada (`192`) · `m2` a conforme pelo eixo no ecrã (`87`) · `m3` a esticada pelas marcas (`249`) · `m4` o
nível fixo no mais grosso (`240`) · `m5` o ajuste a `1` (`251`) · `m6` a emenda nunca (`62`) · `m7` o sentido do
quadrilátero trocado (`241`, e o teste da porta CPU) · `m8` a faixa nunca serve (`101`) · `m9` o leque grosso (`200`) ·
`m10` o traço sem a ponta de início (`249`) · `m11` o arco pela corda (`237`) · `m12` a casa traça por cima da lei (`92`).
⛔ Na 1.ª corrida a `m6` SOBREVIVEU: o ajuste fecha o sub-caminho mais LONGO dentro do último vão (a `FOLGA_DO_AJUSTE`),
e a emenda só acontece num mais curto — as formas da família tinham um contorno só. A engrenagem com furo entrou na
família e a `m6` sangra.


