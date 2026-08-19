# 12 — O MOTOR DE TRAÇO NOVO: baseline medido, pesquisa, e a comparação dos candidatos

**Data:** 2026-07-28 · **Linha:** `line/FLIP` · **HEAD:** `9b2e72ee4` · **Status:** PESQUISA —
**nenhuma linha de motor escrita**, por ordem do handoff
([`HANDOFF_line_FLIP_NOVO_MOTOR_DE_TRACO_2026-07-28.md`](handoffs/HANDOFF_line_FLIP_NOVO_MOTOR_DE_TRACO_2026-07-28.md) §11.4).

> A ordem do Enio: *"encontrar um modo completamente novo de renderização do stroke e descartar
> completamente o atual … Ele deve pesquisar o estado da arte, o padrão ouro."*

Este documento entrega os passos 2 e 3 do §11 do handoff: **de que baseline eu parto** (medido, hoje)
e **o mapa dos candidatos** com o que cada um quebra das três propriedades do §3. A decisão de qual
construir é do Enio.

> ⚠️ **Recorte de 2026-08-18 — o que sobrou aqui, e por quê.**
> A pesquisa (§1-§3), a derivação da lei (§5-§15), a fiação (§18) e a auditoria por grep (§19) já
> viraram produto: elas foram movidas **verbatim** para
> [`docs/archive/docs-2026-08-18/Flip/12_novo_motor_pesquisa.md`](../archive/docs-2026-08-18/Flip/12_novo_motor_pesquisa.md).
> Ficou o que ainda responde alguma coisa **hoje**: a **§4 recomendação** (o desenho que shipou), a
> **⛔ §13.3** (uma conclusão minha que a REFERÊNCIA refutou), a **§16** e a **§20** (o que sobra e as
> três divergências nomeadas), o **§18.5 Rodar** (o A/B do smoke), a **§21.6** (*o que NÃO é alavanca,
> medido, para ninguém re-derivar*) e o **§22 inteiro** — o padrão-ouro, a fila aprovada (§22.4) e as
> **três decisões que são do Enio**: o resíduo de quina, os joins & caps (⛔ premissa REFUTADA) e a
> terceira lei do Krita.
> ⛔ Nada foi resumido — as duas metades remontam o original byte-a-byte (sha256).

---

## 4. RECOMENDAÇÃO

**Construir o C4** (integral de arco acumulada por blending aditivo), com o C1 como plano B
declarado.

Porquê, em três linhas:

1. É o **único candidato que quebra as três** propriedades do §3, e quebra (B) e (C) *pela
   representação*, não por um remendo — a aditividade **apaga o caso especial**, que é o padrão que
   este repo já premiou várias vezes.
2. Entrega a lei da tinta que o Enio pediu (o depósito do Painter) **sem** herdar o parâmetro de
   spacing nem o dilema de lei do §2.4 — e transforma o `self_overlap` de *defeito aberto de 43/255*
   em comportamento natural.
3. É **mais barato** que o C1 em fragmentos, e os dois pagam exatamente o mesmo pedágio de
   arquitetura (o alvo por traço), então o pedágio não é argumento a favor do C1.

**A ordem de execução que eu proponho, se aprovado** — ⚠️ os passos 0 e 1 **já rodaram** (§5, §6) e
mudaram o resto da tabela:

| # | passo | estado |
|---|---|---|
| 0 | **Medir o pedágio** do alvo por-traço | ✅ **FEITO** (§6). A bbox morreu (67 telas/frame); a granularidade é **TILE**, e com ela o alvo por-traço **deixa de existir** |
| 1 | Quantificar **integral × soma finita** contra `painter_deposit_sized` | ✅ **FEITO** (§5). Corpo a **±2/255**, cruzamento incluso; densidade exatamente constante; `sub = 4` |
| 2 | O **binning por tile** + o walk por-tile (o esqueleto, sem lei) | o esqueleto que o §6.3 desenhou; é onde o `neighbors.rs` morre |
| 3 | O kernel `τ` + resolve dentro do walk, com `hardness = 1` byte-idêntico | o CONTROLE de todos os smokes (§8 do handoff) |
| 4 | Reconstruir a bateria do §6 do handoff contra o motor novo | os oráculos são de COMPORTAMENTO e sobrevivem |
| 5 | **Caps e joins como primitivo** (§5.5) — deixou de ser risco e virou escopo | + tips; o `self_overlap` some sozinho |

**O que eu NÃO recomendo:** portar o Ciallo esperando a cura (§2.1 — ele declara a limitação),
continuar na linhagem GP (§2.2 — segue quebrada no 5.0), ou adotar o `Soft` do Drawpile sem medir o
beading (§2.4 — o ponto fixo dele é o `max` que já foi reprovado na tela).

---

### 13.3 ⛔ A CORREÇÃO: a §9.5 concluiu errado, e a referência a refutou

A §9.5 dizia *"qualquer primitivo de cap tem de ser invariante à partição do caminho"*. Fui medir a
referência antes de contorcer o desenho para satisfazê-la — **o depósito do Painter não é
invariante**: um caminho contra cinco pernas compostas por `over` difere em **−59/255 em 178 px**
(dureza 0,4), **−102 em 123** (0,7) e **−255 em 17** (1,0), **sempre nos CANTOS** (cada perna abre
com um dab em `pts[0]`).

A identidade que a §9.1 mediu em ZERO era artefato de o motor **não ter cap**. Ela vale onde o
caminho é o MESMO — os CRUZAMENTOS, que é o que o oráculo do Enio pergunta — e é ali que o gate
agora a afirma (as pontas de perna saem, como saem no depósito de referência).

---

## 16. O QUE FALTA PARA ISTO SER PRODUTO (não é só o passo 5)

1. ~~**ANTI-ALIASING**~~ — **FECHADO no §11.**
2. ~~**As features do §8**~~ — **AUDITADAS no §12.4**: um item de projeto (o cap) e quatro
   mecânicos; nada pede outra arquitetura.
3. ~~**O cap da ponta**~~ — **FECHADO no §13** (um termo de fronteira, não uma geometria).
4. ~~**O port para compute**~~ — **FECHADO no §14: 2,16 ms, paridade 4e-6.**

**Não sobra item de PROJETO.** O que resta é integração: os quatro mecânicos do §12.4 (self_overlap
ON · airbrush · tip Dots/Squares · fade sub-pixel), trocar a saída para textura, ligar o passe no
`flip_pass.rs` no lugar do `flip.wgsl`, e **o smoke do Enio** — que é quem decide.

---

### 18.5 Rodar

⚠️ **O default INVERTEU no §22** — o percurso é o motor, e o interruptor é a escape:

```
cargo build -p ph2d-host-desktop --release
env PH2D_FLIP_DEMO=1                        cargo run -p ph2d-host-desktop --release   # o PERCURSO (default)
env PH2D_FLIP_DEMO=1 PH2D_FLIP_NEW_ENGINE=0 cargo run -p ph2d-host-desktop --release   # o CONTROLE (raster)
```

⚠️ **O A/B é o ponto** — a mesma cena, a mesma mão, os dois builds. O que o §11.3 diz que muda:
o cruzamento com `hardness` (a queixa original), a ponta convexa, e nada mais.

## 20. ⭐ A LISTA FECHOU — e o que sobra

Com o §19.6 o percurso lê **todas as sete** entradas de `Stroke`/flags que o rasterizador lê. O §19
começou porque *"armado, o motor novo apagava CINCO features em silêncio"*; nenhuma continua apagada.

**O que a comparação com o raster deixou NOMEADO em vez de alinhado** (as três divergências, cada uma
com número e sonda):

| divergência | raster | percurso | onde |
|---|---|---|---|
| conta sub-pixel (0,40 px) | apaga a fileira | pontilhado fraco (76 px, pico 57) | §19.3 |
| borda de tampa chata fora de fronteira de pixel | passo duro (perde a posição sub-pixel) | AA exato (102/255) | §19.5 |
| ombro de cruzamento em opacidade 1 com a flag | +63 em 16 px | +31 em 12 px | §19.6 |

Nas três o percurso é o mais correto pelas próprias regras que o raster afirma nos comentários dele, e
nas três **o smoke decide** se isso é o produto.

**A fronteira volta a ser a PERF — e o §21 mediu que ela NÃO era onde este doc dizia.**

### 21.6 O que NÃO é alavanca (medido, para ninguém re-derivar)

- **o ladrilho** (§21.3: o percurso é insensível num fator 8 de área);
- **o piso de dispatch** (1 traço = 0,08 ms de 2,73 ⇒ pular ladrilho vazio compra ~nada);
- **o `log`** (3%);
- **`SUB = 2`** (−30% e reprova o gate de controle);
- **portar o binner** (ele é 26% do total, não 45% — o §14 estava sobre uma amostra ruidosa).

**A decisão de DEFAULT era do Enio, e ela foi tomada no §22** — que também mostra por que este §21
estava com a lente errada.

---

## 22. ⭐⭐⭐ O PADRÃO-OURO — a decisão do Enio, e por que o §21 media a coisa errada

> *"vamos ao padrão ouro"* (Enio, 2026-07-29), depois de *"qual é o estado da arte, o padrão ouro **sem
> olhar custos**?"*

⚠️ **O §21 inteiro comparou o percurso contra o RASTER e concluiu que 26% de um quadro é demais.**
Isso é o §0.0 do `CLAUDE.md` de cabeça para baixo: *nunca deixe o fallback definir o produto* — eu
deixei o caminho mais barato definir o teto do mais correto, e a recomendação que saiu daí (rotear por
tile) era um contorno para um custo que, medido no lugar certo, é de **ARQUITETURA**.

### 22.1 A hierarquia das leis — o percurso não é um candidato, é o LIMITE

```
Beer-Lambert sobre densidade varrida   ← A FÍSICA
   ↓ integral exata
τ = ∫ f(dn) ds ,  α = 1 − exp(−τ)      ← O PERCURSO (construído, gateado)
   ↓ soma finita a 0,1·diâmetro
buffer de dabs                          ← GIMP · Krita · Procreate · o NOSSO Painter
   ↓ (fora da família)
união global + eleição por depth        ← o raster que shipava
```

A pesquisa do §2 disse que **ninguém publicou a resposta** (Ciallo declara o nosso defeito como
limitação própria; o GP segue quebrado no 5.0.1; o Vello é *"outlines only — not soft/feathered"*), e a
hierarquia explica por quê: o percurso é o **limite contínuo** que os dab buffers da indústria
aproximam por soma finita. É por isso que ele bate o **próprio depósito do Painter** como oráculo nas
três divergências do §20 — o Painter é uma soma finita do mesmo limite.

⚠️ **Corolário que fecha o pedido original:** o C1 (buffer de dabs, o que o Enio pediu ao pé da letra)
é **estritamente pior** que o que está construído — nele a lei volta a depender do *spacing*, a doença
que matou o motor atual e que a `sampling_invariance` proíbe. **O percurso é o C1 sem o defeito do
C1.**

### 22.2 A decisão: o percurso é o DEFAULT (`aa14e9366` invertido)

`PH2D_FLIP_NEW_ENGINE=0` passa a ser a **escape** para o raster. A política mora numa função **PURA**
(`walk_from_env`) e não no `OnceLock`, porque um `OnceLock` sobre variável de processo responde uma vez
por binário ⇒ **o default não era afirmável**, e default não-afirmável é default que a próxima edição
inverte em silêncio. Gate `the_walk_is_the_default_and_only_an_explicit_zero_escapes_to_the_raster`
(mutação: voltar ao opt-in sangra na 1ª asserção). Só o desligamento **explícito** volta ao raster —
um `=flase` falha **para o default**, nunca para um terceiro comportamento.

### 22.3 ⚠️ O custo era ARQUITETURA, e o número de hoje mede o Pass A errado

Conferido no `flip_pass.rs`: **não existe dirty check no Pass A.** O cache é da *tesselação* (CPU,
cache-hit em pan/zoom); a rasterização na GPU roda **por camada, por frame, sempre** — panhando,
parado, em playback. Nenhum app de pintura faz isso; **o nosso Painter não faz** (canvas + preview +
dirty-rect); nenhum app vetorial faz (Illustrator/Figma cacheiam tiles, invalidados por edição e por
nível de zoom).

E o Flip **multiplica camadas por conta própria**: cada fantasma do onion é uma camada com o seu
`stage_layer`. ⚠️ **E a cobertura de um fantasma é IDÊNTICA à do desenho** — o `with_ghost_tint` põe o
tint na CÂMERA, só rgb/alpha mudam ⇒ hoje o onion re-rasteriza N vezes a mesma cobertura.

| | hoje | com cache de cobertura |
|---|---|---|
| arte commitada | re-rasteriza todo frame | **0/frame** |
| fantasmas do onion | N× a MESMA cobertura, todo frame | **0/frame** (uma cobertura serve o desenho e todos os fantasmas dele) |
| traço VIVO | — | **0,166 ms = 1% de um quadro** (medido, §21.2) |

⇒ **os 14,8× são o preço de um re-render COMPLETO**, que sob a arquitetura certa acontece na **edição**
e na **troca de zoom** — não 60 vezes por segundo. E o item conserta o **raster também**: não é gastar
para bancar o percurso, é uma dívida do módulo que o percurso apenas **expôs**.

### 22.4 O que falta para padrão-ouro (a fila aprovada)

| # | item | estado |
|---|---|---|
| 1 | **percurso como default** | ✅ **feito** (§22.2) |
| 2 | **o Pass A pergunta antes de rasterizar** | ✅ **feito** (§22.5) — arte commitada e fantasma de onion custam **zero** enquanto ninguém mexe neles |
| 2b | o cache em tiles de MUNDO (sobreviver ao pan, não só ao parado) | desenho; o (2) já entrega o caso que domina |
| 3 | **a integral de ÁREA do pixel** | ⚠️ a premissa estava ERRADA (§22.6) — mas a medição achou **três** defeitos, e **dois já fecharam** (§22.7): a QUINA (63,75/255) e o ÂNGULO (9,72 a 45°, que ninguém tinha visto). |
| 3a | ⚠️ **NÃO era saturação — o percurso DERRUBAVA tinta em tampa e junta** | ✅ **CURADO** (§22.10): a grade resolve a JANELA, não o segmento. Tampa e junta vão a **zero px derrubados**, e o miolo **não se move** (0 px acima de 1/255 em h=0,4 e 0,7). |
| 3c | o resíduo de QUINA que a lei de área expôs | **FECHADO por medição (2026-07-31)** — segue pinado, e **não é regressão** (§22.10). Três curas construídas/avaliadas; a melhor (amostrar no CENTROIDE da parte coberta) foi **reprovada pelo oráculo supersampleado**: empate nos flancos, pior nas junções (96,38 contra 61,86/255). E o achado que fecha: o pior erro da lei que SHIPA contra a verdade é **22-62/255** e o resíduo vale **≤ 14,94** — *o artefato é menor que o erro da aproximação que o curaria*. A cura real ataca a aproximação (supersamplear o perfil / 2º tap): outra wave. |
| 4 | ~~**joins & caps**~~ | ⛔ **A PREMISSA FOI REFUTADA POR MEDIÇÃO (§22.9).** O `−64` saía do RASTERIZADOR (a escape hatch, não o default desde `9a4bdd07b`), e no percurso ele decompõe em duas causas conhecidas — o termo de fronteira do `end_dab` (correto: cinco traços têm cinco começos) e `over` contra união exata. **Nenhuma é geometria de junta.** Sobra uma pergunta de PRODUTO, não de correção. |
| 5 | **a terceira lei** (`Soft` do Krita, §2.4) | ⚠️ **MEDIDA (§22.11): funciona perfeitamente, e a ressalva do §2.4 NÃO alcança o percurso** — sem dabs não há beading. Mas ela muda a borda de UMA passada em +69%, então é **decisão de LOOK, do Enio**, não de engenharia. **⚠️ E há uma SEGUNDA metade do preço, medida em 2026-07-31 (§22.11):** ela capa toda acumulação dentro do traço, então o **SELF OVERLAP** (shipado em 2026-07-27) fica INERTE — o cruzamento vai de **1,50× para 1,00×** o braço, em toda dureza. Os dois são mutuamente exclusivos, como o One-Way e a zona de força da física. |

A **antiderivada** (§21.5) cai para o fim: ela é *perf*, e sob (2) deixa de ser necessária.

### 22.5 ⭐ PASSO 2 — o Pass A PERGUNTA antes de rasterizar

O skip é correto quando **duas** coisas valem, e cada uma responde o que a outra não pode:

1. **a impressão digital bate** — o que este frame produziria é o que já foi produzido (o nosso
   `StageMemo`);
2. **o compositor AINDA tem a fatia** — `LayerCompositor::has_slice`, **a palavra do dono**: ela pode
   ter sido despejada pelo LRU do `alloc_slice` ou limpa por um rebuild do array (resize, op-list
   nova), e nenhum memo nosso sabe disso.

⚠️ **Sem a (2) o modo de falha é arte VELHA na tela, e nada parece quebrado.** Com ela o pior caso é
*fazer o trabalho* — o que o produto fazia antes desta wave. É a lição do ADR-0124 no nível da fatia:
pergunte ao DONO, nunca ao seu próprio memo.

⚠️ **A `version` do inject fica em `0`, e a armadilha estava documentada no código que eu ia mudar.**
Meu 1º desenho punha a impressão digital ali — mas o `DummyProvider` reporta versão **0** para
qualquer chave, e é essa igualdade que faz o `ensure_slice` achar a fatia "limpa" e **não subir o
dummy transparente por cima da arte**. Um número diferente de 0 apagaria a camada. A frescura mora no
nosso memo por causa disso.

**A impressão digital toma a câmera como RESULTADO** (`CameraRaw` inteiro, POD de 96 bytes via
`bytemuck`) e não como ingredientes: paralaxe multiplano, fold do `model` e tint de fantasma já estão
dobrados lá dentro ⇒ uma porta nova que mexa na câmera é coberta sem ninguém se lembrar da função. O
mesmo vale para os pontos do preview (`GpuPoint` é POD). ⚠️ **Esquecer uma entrada é o único bug grave
da wave, e ele é silencioso** — a camada congela mostrando o estado anterior.

**O que a impressão CUSTA** (`measure_what_the_fingerprint_costs`, 12 corridas, 1ª descartada,
mínimo):

| preview | custo por camada |
|---|---|
| **0 pontos** — arte commitada e TODO fantasma do onion | **0,0001 ms** |
| 200 pontos | 0,0047 |
| 2 000 | 0,0467 |
| 20 000 | 0,4475 |

⇒ o caso que domina (as camadas que serão puladas) custa **100 ns** contra **4,33 ms** economizados. O
`O(n)` só morde no traço VIVO, cuja faixa real depois do RDP + reamostragem é 200–2000 pontos.
⛔ **Duas alternativas O(1) rejeitadas:** *contagem + último ponto* colide em princípio (dois previews
distintos com a mesma contagem e a mesma ponta), e *identidade de ponteiro do buffer* é o ABA que o
ADR-0124 e a §5.12 do Painter já pagaram.

**Gates:** 5 de unidade (impressão estável · **cada** entrada a move · o traço vivo a move e a mão
parada não · a lei do skip com as duas metades · as estatísticas) + **3 de arch-gate** sobre o
`flip_pass.rs` (pergunta ANTES do raster **e** honra a resposta com `continue` · o 3º argumento é
`has_slice` e **não** o literal `true` · a impressão usa `&layer_cam` e `l.preview`).

⚠️ **4 mutações, 4 sangram — e cada uma só na camada que a possui**, o que é a prova de que os dois
níveis não são redundantes:

| mutação | unidade | arch |
|---|---|---|
| o compositor "sempre tem" a fatia (`true`) | verde | **RED** |
| a impressão usa a câmera do FRAME | verde | **RED** |
| o preview sai da impressão | verde | **RED** |
| o skip ignora a impressão digital | **RED** | verde |

Instrumento: **`PH2D_FLIP_STATS=1`** passou a imprimir `pass A: N rasterizada(s), M pulada(s)` por
frame — sem ele *"o cache está funcionando?"* é opinião. ⚠️ **O ganho fim-a-fim é PROJEÇÃO até o
smoke:** o mecanismo está gateado e o custo medido, mas quem confirma os zeros é aquela linha com
onion ligado e a mão parada. **(SMOKE do default APROVADO pelo Enio, 2026-07-29.)**

### 22.6 ⚠️ PASSO 3 — a premissa do item estava ERRADA, e a medição achou outros dois defeitos

**O que eu escrevi no §22.4 e está falso:** *"a cobertura é amostrada no CENTRO do pixel, sem AA"*,
inferido de `sample_count: 1` + `no_msaa()` em todo o pipeline. Os dois fatos são verdade e
**irrelevantes** — o AA aqui é **analítico**, não MSAA. O `stroke_deposit` já computa
`edge = clamp(0.5 − sd, 0, 1)` (o filtro-caixa da silhueta em PIXELS), com o `min` sobre as passagens
**EXATO**, e o perfil amostrado no **ponto médio da parte coberta** (`u* = (sd − ½)/2`) — cuja
derivação está no próprio código, junto com a medição que reprovou empurrar meio pixel inteiro
(24,19/255). ⚠️ **Inferir a ausência de um mecanismo a partir de um proxy, em vez de grepar o
mecanismo**, é exatamente a armadilha que a memória do repo nomeia — e eu caí nela num item que
promovi a *"o maior buraco visível que sobrou"*.

**O que de fato sobra**, medido em `aa_tests.rs::measure_what_the_box_filter_owes_the_pixel_area`
(oráculo: a ÁREA de verdade por supersampling 16×16 do teste dentro/fora, em `hardness = 1`, onde a
cobertura é pura área):

| cena | `edge` vs área | `cover` vs área | saturação |
|---|---|---|---|
| borda reta longe das tampas (**CONTROLE**) | **0,00** | **0,00** | 0,00 |
| a mesma cena **incluindo a tampa redonda** | 8,74 | 24,90 | **29,99** |
| **PONTA** aguda | 10,97 | 18,93 | **21,55** |
| **QUINA** externa (duas cápsulas cruzando) | **63,75** | **63,75** | 0,00 |

⚠️ **O controle precisou de uma janela, e a 1ª versão não tinha:** o pior pixel da cena "reta" caía em
`(87,5; 41,5)` — 3,5 px DEPOIS do fim do traço, ou seja **na tampa redonda**. A fixture continha a
curvatura que ela existia para excluir, e o "controle" media 24,90. Com `x ∈ [20, 76]` ele mede
**0,00**, e é isso que torna as outras linhas fenômeno em vez de ruído de instrumento.

**São DOIS defeitos independentes, e a decomposição é o que impede consertar a metade errada:**

- **3a — a SATURAÇÃO (22-30/255), e ela NÃO é o AA.** Em curvatura (tampa, ponta) o filtro-caixa erra
  só 9-11 e o produto erra 19-25; o resto é `1 − exp(−τ)` não chegar a 1. Mecanismo: em
  `hardness = 1` o `f_of` devolve `F_MAX = 16` e um pixel raso perto da borda pega **fração de um
  dab** ⇒ `τ = 4` ⇒ `1 − e⁻⁴ = 0,9817`. O `F_MAX` é o que substitui o infinito da lei dura, e é ele
  que limita a saturação ali.
- **3b — a QUINA (63,75/255 = ¼ da cobertura de um pixel), 100% geométrica** (saturação 0,00 no pior
  pixel): a união das duas cápsulas cobre **¾** do pixel e o `min` do SDF diz que o centro está EM
  CIMA da fronteira (`edge = 0,5`). É a limitação clássica de todo AA por SDF.

⚠️ **E a sonda `dump_the_crossing_pixel` pegou um erro meu de LEITURA antes de eu publicar a
conclusão:** eu li o pior pixel como *"uma borda reta vertical"* e ele é a **quina externa de um L** —
porque **o Y do `point_px` é invertido** (mundo (16,16) → px (16,80)), a cicatriz que o §18.4b já
registrou. `63,75` ser exatamente `0,25 × 255` foi o que me fez abrir o pixel em vez de concluir: *um
número redondo é ou geometria exata ou fixture, e os dois são indistinguíveis sem olhar as partes.*

**Nada foi construído neste passo** — o item mudou de forma, e as duas correções são waves próprias
(uma toca o `F_MAX`, que tem racional próprio; a outra é AA ciente de quina). O que fica é o
**instrumento**: um oráculo de área supersampleado, com controle em zero, que qualquer tentativa
futura tem de bater.

## 17. Fontes

- Ciao, S. & Wei, L.-Y. — *Ciallo: GPU-Accelerated Rendering of Vector Brush Strokes*, SIGGRAPH 2024.
  [ACM](https://dl.acm.org/doi/10.1145/3641519.3657418) ·
  [CIS Lab](https://cislab.hkust-gz.edu.cn/publications/ciallo-gpu-accelerated-rendering-of-vector-brush-strokes/) ·
  [tutorial do autor](https://shenciao.github.io/brush-rendering-tutorial/)
- Levien, R. et al. — *GPU-friendly Stroke Expansion*, [arXiv:2405.00127](https://arxiv.org/html/2405.00127v1)
- Blender — [#154433](https://projects.blender.org/blender/blender/issues/154433) (opacity/hardness,
  5.0.1 e 4.2.18 LTS) · [corner overlap artifacts](https://devtalk.blender.org/t/grease-pencil-corner-overlap-artifacts/3032)
- Krita — [Opacity and Flow](https://docs.krita.org/en/reference_manual/brushes/brush_settings/opacity_and_flow.html) ·
  [Soft painting mode](https://krita-artists.org/t/feedback-wanted-soft-painting-mode/167535) (Drawpile)
- In-repo, **não re-derivar**: [`docs/Painter/25 §13.9–§13.13`](../Painter/25_avaliacao_gpu.md) ·
  [`docs/Flip/03 §8.6–§8.7.2`](03_traco_rasterizacao.md)

### 22.7 ⭐ PASSO 3 — a cobertura é a ÁREA do pixel, e o mesmo mecanismo fecha DOIS defeitos

A §22.6 mediu a quina em **63,75/255** e a chamou de "o único 100% geométrico". Indo consertá-la,
uma dúvida sobre a minha própria atribuição derrubou metade da tabela e achou um **terceiro**
defeito, maior em alcance que a quina.

#### O que a dúvida era

Eu havia escrito que na cena da PONTA *"o erro é a saturação, não o AA"*, com `|edge−área| = 10,97`.
Mas o filtro-caixa da lei antiga é **1-D ao longo da normal**, e a área de um quadrado unitário
cortado por um semi-plano **depende do ÂNGULO da borda**: a rampa `0,5 − sd` só é exata quando a
borda é paralela a um eixo. Uma conta de meia linha diz que a 45° isso já deve ~10,9/255 — todo o
número atribuído à saturação, numa cena sem quina nenhuma.

⚠️ **E a 1ª medição da varredura de ângulo deu 15,11 — ACIMA do teto teórico.** Um número que passa
do próprio limite é ou fixture ou premissa errada, e aqui era o **ORÁCULO**: a estimativa por
sub-amostras erra na proporção de quantas sub-células a fronteira atravessa, e isso é **máximo a
45°**, exatamente o ângulo em questão. *Um oráculo cujo erro é função do parâmetro que a sonda varia
não é um oráculo* — a mesma lição que a antiderivada (§21.5b) pagou nesta jornada. Com `N = 96` a
área amostrada bate a analítica em quatro dígitos e o número do produto aparece:

| ângulo | RAMPA (era) | ÁREA (é) |
|---|---|---|
| 0° | 0,00 | 0,00 |
| 15° | 5,76 | **0,03** |
| 30° | 8,54 | **0,03** |
| 45° | **9,72** | **0,07** |
| 60° | 8,54 | **0,03** |
| 75° | 5,76 | **0,03** |
| 90° | 0,00 | 0,00 |

**Este defeito é o mais pervasivo dos três** — não precisa de quina, de ponta nem de tampa: basta
uma borda que não seja horizontal nem vertical, ou seja quase todo traço de todo desenho.

#### A cura, e por que ela apaga os dois casos de uma vez

O conjunto **NÃO** coberto do pixel é a interseção dos semi-planos de FORA de cada passagem.
Interseção de semi-planos com um quadrado é um **polígono convexo** ⇒ recorte de Sutherland-Hodgman
+ fórmula do sapateiro (`pixel_area.rs`), exato, sem caso especial e sem transcendental (HR-5). É a
mesma decomposição que o empuxo da `line/physics` usa para *"quanto deste corpo está dentro da
água"*.

- com **um** plano ele é a área exata de um semi-plano em qualquer ângulo ⇒ o defeito novo;
- com **dois** ele é a quina ⇒ o defeito de 63,75;
- **e reduz à rampa EXATAMENTE onde ela estava certa** (gate `an_axis_aligned_edge_is_exactly_the_old_ramp`, borda paralela a um eixo) ⇒ o traço horizontal/vertical não se move um bit.

| cena | RAMPA (era) | ÁREA (é) |
|---|---|---|
| borda reta longe das tampas (CONTROLE) | 0,00 | 0,00 |
| a mesma com a TAMPA redonda | 10,18 | **2,59** |
| PONTA aguda | 10,65 | **4,50** |
| **QUINA externa** | **63,75** | **3,20** |

O resíduo de 2,6-4,5 é a **CURVATURA** (cada passagem entra como o plano TANGENTE, e uma tampa de
raio 7 px não é reta dentro do pixel) — declarado deliberado no `pixel_area.rs`, e o controle em
0,00 é o que prova que ele não é o instrumento.

#### ⚠️ O port para o device, e o gate que o exigiu

O que **shipa** é o compute (`walk.wgsl`), não a referência em Rust — então mudar só a CPU deixaria
o produto na lei antiga com a referência na nova. O `walk_gpu_parity` **nasceu VERMELHO** e é ele
que provou o port: pior `|Δ|` = **4,883e-4**, que **é** o quantum da meia precisão do alvo
(`rgba16float`, 2⁻¹¹) — os dois lados concordam até o limite do formato.

#### ⚠️ O teto de planos é MEDIDO, e as duas metades estão no código (§0.0)

*Quantos o desenho usa:* a quina mede **2**, e o pior pixel de um zigue-zague de passo sub-pixel — a
única figura em que consegui pôr três bordas perto do mesmo pixel — também mede **2**. *Quanto cada
vaga custa*, no device a 200 traços/1080p, ladrilho 16:

| vagas | frame do percurso |
|---|---|
| a lei antiga | **2,72 ms** |
| 2 | 3,46 |
| **3 (o que shipa)** | **3,65** |
| 4 | 4,71 |

O degrau de 3→4 é o array de recorte deixando de caber em registrador. **Três fica acima da maior
contagem que consegui produzir e abaixo do degrau**; truncar erra sempre para MENOS cobertura, e
ainda assim para mais que a lei antiga, que enxergava **um** plano só.

⚠️ **E os dois atalhos exatos do `coverage()` quase não pagam no device** (4,91 → 4,71, 4%): pixel de
fronteira é espalhado, então quase todo warp tem um, e um atalho divergente não economiza tempo se
alguém no warp toma o caminho longo. Ficam por serem exatos e de graça, **não** por serem a
otimização — a nota que eu havia escrito ali (*"1,39 → 2,32"*) era um número que eu não tinha
medido, e foi corrigida.

#### O que NÃO mudou, de propósito

- o **`p_eval`** (o empurrão que amostra o perfil) segue lendo `sd`, **não** o `edge`. Enquanto a
  cobertura era a rampa os dois eram a mesma expressão e é tentador "unificar" — são perguntas
  diferentes: *onde ao longo da NORMAL amostrar o perfil* (1-D) contra *quanto do PIXEL a silhueta
  cobre* (2-D). Os dois lados carregam a nota.
- o **rasterizador** (`flip.wgsl`) fica na rampa por-PASSAGEM: ele precisa do `fwidth` de um `min`,
  que salta na costura. Ele é a escape hatch, não o produto.

**Gates:** 6 em `pixel_area` (incluindo o oráculo de FORÇA BRUTA, que não sabe de recorte nem de
sapateiro) + 1 no PRODUTO (`at_a_corner_the_deposit_passes_what_the_nearest_edge_alone_allows`,
cujo oráculo é aritmético: a rampa não passa de `0,5` num pixel de `sd ≥ 0`, então nenhum ajuste de
constante alcança o outro lado) + o `walk_gpu_parity`. **6 mutações, 5 sangram**; a 6ª
(`PLANE_REACH` gigante) **sobrevive por projeto** — o `offer` já ordena por `sd`, então um plano fora
de alcance é despejado antes de importar: o alcance é **custo**, não correção, e está escrito assim.

### 22.8 ⚠️ O defeito "3a" NÃO era saturação — o percurso DERRUBA tinta em tampa e junta

A §22.6 mediu uma coluna que chamei de *"saturação, 22-30/255"* e escrevi que ela vinha de
`1 − exp(−τ)` não chegar a 1, encostando no `F_MAX`. **Fui abrir o pixel antes de tocar no `F_MAX`,
e o rótulo estava errado.**

#### O que a sonda achou

| | sd | área real | `edge` | **τ** | `cover` |
|---|---|---|---|---|---|
| flanco reto (pior pixel) | +0,382 | 0,1028 | 0,1041 | **0,000** | **0,0000** |
| PONTA aguda | +0,415 | 0,0796 | 0,0829 | **0,000** | **0,0000** |
| QUINA externa | +0,450 | 0,0654 | 0,0662 | **0,000** | **0,0000** |

O `τ` não é *pequeno*: é **zero**, e o `stroke_tau` devolve `None`. A coluna que eu chamara de
saturação era `edge − 0` — o depósito **não depositando nada** num pixel com 6-10% de cobertura
real. Multiplicar 0,1041 por 255 dá exatamente os 26,56 que a tabela antiga atribuía ao `F_MAX`.

#### O CONTROLE, que é o que separa as duas explicações

No **flanco**, com o MESMO `sd`, o produto acerta:

| sd | área | `cover` |
|---|---|---|
| +0,401 | 0,0938 | **0,0938** |
| +0,431 | 0,0759 | **0,0758** |
| +0,309 | 0,1592 | **0,1591** |
| +0,339 | 0,1355 | **0,1354** |

⚠️ **E o controle precisou ser INCLINADO:** num traço horizontal o `sd` do flanco cai exatamente em
±0,5, então nunca existe pixel meio-coberto ali — é por isso que os três piores pixels das sondas
caíram todos numa TAMPA, e é o que fez a 1ª versão do gate falhar medindo o vazio.

#### O mecanismo, aberto no pixel

Perto de uma tampa ou de uma junta o **pico do integrando cai EM CIMA da fronteira do domínio de
integração** (o extremo do segmento), e o suporte encolhe. No pixel `(87,5; 41,5)`: o ponto
empurrado pelo `p_eval` fica a 6,942 do extremo com `r = 7` ⇒ os centros de dab que o alcançam vivem
num trecho de **0,121 px**, contra um passo de quadratura de `pitch/SUB = 1,4/4 = 0,35`. **A regra
do ponto médio não pega uma amostra sequer**, e a integral inteira dá 0.

Não é termo de fronteira faltando (o `end_dab` existe e a assimetria dele é medida contra o
Painter): é o **`SUB` sendo um piso sobre o SEGMENTO quando deveria ser um piso sobre a JANELA**.

#### Por que a cura é wave própria, com o preço de adiá-la medido

Resolver a janela em vez do segmento move as posições de amostra **em todo pixel**, logo move todo
número de tinta do motor — e com eles o port WGSL e os gates de look que o Enio já aprovou. O que se
paga por adiar:

| cena | px com tinta | px DERRUBADOS | pior queda | soma perdida |
|---|---|---|---|---|
| traço reto (2 tampas) | 1180 | **4** | 26,21/255 | 0,41 px |
| L (2 tampas + 2 juntas) | 1824 | **4** | 16,68/255 | 0,26 px |
| zigue-zague (24 juntas) | 1115 | **13** | 14,94/255 | 0,70 px |

≈ **um pixel por tampa ou junta**, na banda mais externa, num pixel que estaria ≤10% coberto. É
pequeno — e é um número, não uma impressão.

**Pinado em gate executável:** `the_walk_drops_a_sliver_of_ink_at_caps_and_joints_and_this_is_its_number`
afirma as DUAS metades (o flanco exato · a tampa em zero) e **falha quando o defeito for
corrigido**, obrigando a atualizar esta nota — o padrão do
`the_documented_hardening_is_still_there_and_this_is_its_number` do Painter. Sem ele o diagnóstico
volta a ser re-derivado do zero, que é literalmente o que aconteceu com o rótulo "saturação".

### 22.9 ⛔ O item 4 (joins & caps) foi REFUTADO — o número dele era de outro motor, e a causa é outra

A fila trazia *"o resíduo MEDIDO que sobrevive a `hardness = 1,0` (−64, 58 px) ⇒ provadamente não é
a lei da tinta"*, e concluía **geometria de junta e tampa**. Fui construir e a medição derrubou as
duas metades.

#### (a) O número descrevia o RASTERIZADOR

`measure_the_star_one_stroke_against_separate_strokes` renderiza pelo `FlipRenderer` — o
**rasterizador**, que desde `9a4bdd07b` (§22.2) é a *escape hatch* e não o default. A §1 inteira
deste doc é a baseline DELE. Ler dali que *"o percurso não conserta"* é uma conclusão sobre um motor
que o artista não usa mais.

⚠️ **E a minha 1ª tentativa de testar isso foi INVÁLIDA:** desliguei o `end_dab` em Rust e os
números não mudaram — o que eu quase reportei como "hipótese refutada". Eles não mudaram porque a
medição roda no **device**: a mutação caiu fora do caminho medido. *Uma busca negativa precisa de
controle positivo* ([[feedback_a_negative_search_needs_a_positive_control]]), e aqui o controle era
trivial — perguntar por qual pipeline o harness renderiza.

#### (b) No PERCURSO o resíduo existe, e decompõe em duas causas conhecidas

Sonda nova `measure_the_star_residual_on_the_walk_not_the_raster` (CPU, `walk_pixel`, byte-paridade
com o device):

| | com `end_dab` | **sem** `end_dab` |
|---|---|---|
| h=0,4 | −37 (138 px) | **−2 (0 px)** |
| h=0,7 | −55 (111 px) | **−7 (0 px)** |
| h=1,0 | −71 (87 px) | −62 (68 px) |

- **em dureza macia o resíduo INTEIRO é o termo de fronteira** — cinco traços têm cinco começos,
  logo cinco meios-dabs; um traço tem um. Isso é o motor **reproduzindo fielmente** que cinco
  traços não são um traço, exatamente como o Painter carimba um dab no começo de cada traço. Não é
  defeito: é a resposta certa para entradas diferentes;
- **em dureza 1 o que sobra é `over` contra UNIÃO EXATA.** No pior pixel, `um = 0,4577` e
  `cinco = 0,6997` — e `a + b − ab` com `a = b = 0,4577` dá **0,7060**, a composição
  probabilística. Um traço computa a área exata da união (o que a §22.7 estabeleceu, com gate);
  cinco traços independentes compõem `over`, que é o certo para camadas de tinta separadas. As duas
  respostas são corretas para as duas entradas, e **têm** de divergir onde os traços se sobrepõem.

⚠️ **Corolário:** *um traço vs cinco traços* **não é um oráculo**. Ele compara dois desenhos
diferentes por duas leis corretas, e usá-lo como evidência de defeito foi o erro.

#### (c) E o pedaço de h=1,0 que a §22.8 já tinha nomeado

No pior pixel de h=1,0 **com** `end_dab`, `um = 0,0000` — o traço único deposita **zero**. É o
defeito **3a** (§22.8) outra vez, agora numa estrela: a junta da ponta colapsa o suporte do
integrando e a quadratura não pega amostra. O `cinco` é não-nulo porque cada traço tem um começo, e
o `end_dab` dispara nele.

#### O que sobra de joins & caps, honestamente

Uma pergunta de **produto**, não de correção — e com um contra-argumento estrutural:

- **round join e round/flat cap já existem** (a união de cápsulas *é* o join redondo; o `cap_sd` +
  `FLAG_*_FLAT` dão o butt);
- **miter e bevel contradizem o modelo** que a §22.1 estabelece como o padrão-ouro: a silhueta é a
  união dos dabs, e um pincel de dabs **não mitra**. Adicioná-los quebraria a hierarquia inteira
  (*o percurso é o limite contínuo do dab buffer*) para ganhar um estilo de traço VETORIAL;
- **square/projecting cap** é o único que cabe barato e sem tocar a união: é o mesmo semi-plano do
  flat, deslocado de `r`. Fica disponível se o produto pedir.

⇒ **Nada a construir aqui sem uma ordem de produto.** A fila real que sobra é o **3b** (a cura da
quadratura, §22.8) e o **5** (a terceira lei).

### 22.10 ⭐ O 3a CURADO — a grade resolve a JANELA, e o miolo não se move

A §22.8 disse *"a cura é wave própria porque move todo número de tinta do motor"*. **Era estimativa
minha, e a medição a derrubou.**

#### A mudança, em duas linhas

A quadratura ancorava a grade no **SEGMENTO** (`n = ceil(len/ds)` células, amostras nos centros) e
depois escolhia as células que caem na janela `[t0, t1]`. Agora ela ancora na **JANELA**
(`n = ceil(win/ds)`, amostras nos centros dela), com a **mesma densidade `ds`**. O integrando é zero
fora da janela por construção — a janela é exatamente onde o dab alcança —, então é a **mesma
integral**; o que muda é onde as amostras pousam.

#### O raio de explosão, MEDIDO (`dump_the_walk_alpha_field`, A/B do campo inteiro)

| dureza | px com tinta | px que movem >1/255 | >16/255 | pior |
|---|---|---|---|---|
| 0,4 | 2208 | **0** | 0 | 0,10 |
| 0,7 | 2234 | **0** | 0 | 0,37 |
| 1,0 | 2324 | 16 | 8 | 69,11 |

⚠️ **O miolo não se move**, e o motivo é estrutural: onde a janela é larga, re-ancorar a grade vale
`O(passo²)`. Os 16 pixels de dureza 1 **são o defeito sendo corrigido**. O medo de "mover todo
número de tinta" era meu; o número é do produto.

#### O que fechou

| cena | px derrubados ANTES | DEPOIS |
|---|---|---|
| traço reto (2 tampas) | 4 | **0** |
| L (2 tampas + 2 juntas) | 4 | **0** |
| zigue-zague (24 juntas) | 13 | 13 (outra causa — abaixo) |

E a estrela no percurso melhorou em dureza 1: **−71 → −63**, com a sobra indo a 0.

**Custo:** device a 200 traços/1080p, ladrilho 16: **3,65 → 3,70 ms** (+1,4%). O port para o
`walk.wgsl` foi junto e o `walk_gpu_parity` fecha no mesmo **4,883e-4** — o quantum da meia
precisão do alvo. ⚠️ O port esbarrou numa colisão de nome (`win` já existia como o retorno do
`seg_window`) — o validador de shader pegou, não o olho.

#### ⚠️ O resíduo que sobrou é da MINHA lei de área, e não é regressão

Os 13 do zigue-zague são outra coisa. O 1º ofensor tem **`sd = +0,5355`**: o centro do pixel está a
mais de meio pixel da silhueta, e a área só é não-nula porque uma **QUINA** do pixel entra (a lei de
área enxerga até `√2/2`). Mas o `p_eval` — que decide onde amostrar o PERFIL — é **1-D ao longo da
normal**, e a derivação dele (*a parte coberta é `v ∈ [sd − ½, 0]`*) tem intervalo **vazio** quando
`sd > ½`: ele pousa FORA, `τ` sai 0, o depósito devolve `None`.

⚠️ **Com a lei antiga esses pixels tinham `edge = 0,5 − sd ≤ 0` e morriam no early-out** — eram
derrubados do mesmo jeito, só que sem ninguém saber. A lei de área não criou o buraco: ela o **tornou
visível**, que é o que uma lei mais exata faz.

Medido: **13 px de 1115**, área ≤ 2,9% (14,94/255), pinado em
`the_area_law_can_claim_a_corner_the_profile_cannot_sample_and_this_is_its_number`.

#### ⛔ 2026-07-31 — a pergunta FECHOU por medição, e a resposta é *não vale a pena por esta rota*

As duas curas que esta seção nomeou ganharam uma **terceira, melhor que as duas**: amostrar o perfil
no **CENTROIDE da parte coberta**, derivado do MESMO polígono que a lei de área já recorta
(`c_cob = −c_desc·A_desc/A_cob`, um momento acumulado no laço do sapateiro). Ela fecha o buraco **por
construção** (a região coberta é não-vazia exatamente quando a área é não-nula), não capa o alcance
dos planos, e substitui um modelo de FATIA que erra duas vezes — a extensão do quadrado na normal é
`|nx|+|ny|` (até `√2`), não 1, e a densidade ao longo dela é um **TRAPÉZIO**, não uma constante.

Foi construída, e o **oráculo supersampleado** (`measure_which_profile_sample_point_is_closer_to_the_truth`,
a média da tinta sobre o pixel a 24×24 — irmã do `true_area`) a reprovou:

| cena / dureza | erro médio FATIA | CENTROIDE | pior FATIA | pior CENTROIDE |
|---|---|---|---|---|
| flanco 0° / 0,8 | 0,89 | 0,87 | 23,07 | 23,07 |
| flanco 45° / 0,8 | **2,77** | 2,87 | 28,19 | 28,19 |
| zigue-zague / 0,8 | **3,39** | 4,45 | **61,86** | **96,38** |

Empate nos flancos, **pior nas junções** — que é exatamente onde o resíduo vive. O mecanismo: numa
junção a região coberta é uma **UNIÃO** de passagens, possivelmente não-convexa, e o centroide dela
não representa ninguém; a fatia, ancorada na normal da passagem MAIS PRÓXIMA, ao menos amostra onde a
passagem dominante manda. (A variante intermediária — manter a fatia e corrigir só o SUPORTE, `r` no
lugar do ½ — é pior que as duas: **65,77/255** de movimento no zigue-zague, **21,48** até num flanco a
0°, porque herda a densidade uniforme.)

⚠️ **E o que a medição estabeleceu vale mais que o veredito de uma cura:** o pior erro da lei que
SHIPA contra a verdade é **22 a 62/255**, e este resíduo vale **≤ 14,94**. *O artefato é menor que o
erro da aproximação que o curaria.* Enquanto o ponto de amostra do perfil for **UM ponto**, mexer nele
troca um artefato pequeno e conhecido por um maior e difuso. A cura de verdade é atacar a aproximação
inteira — supersamplear o perfil, ou um segundo tap — e isso é outra wave, com preço de device
próprio. **O item 3c sai da fila de decisões e entra como "medido e não vale a pena por esta rota".**

#### Gates

O pin do defeito virou o **guard da cura** (`the_walk_no_longer_drops_the_ink_at_a_cap`), com as
duas metades de sempre: o flanco INCLINADO exato (o controle) e a tampa depositando. **2 mutações:**
voltar a grade para o segmento **sangra**; trocar `ceil` por `floor` na contagem de amostras
**sobrevive — e é honesto**: ela muda a resolução em ≤1 amostra, que é exatamente o que o gate
`the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled` prova que não muda a resposta (e
ele passa sob a mutação, o que é evidência, não buraco).

### 22.11 A TERCEIRA LEI, medida no percurso — ela funciona, e o preço é a borda de UMA passada

O item 5 pedia *"medir o ponto fixo antes de decidir"*. Medido, e ele responde três coisas.

#### (1) O percurso TEM a doença

O §2.4 mediu o endurecimento no Painter, cujo depósito é um **produto por-dab**. O percurso não tem
dabs — mas a álgebra é da mesma família (`α = 1 − exp(−τ)` é uma **taxa rumo a um teto**), e dobrar
`τ` a cada passada encolhe a faixa em que `α` sobe de 10% a 90%. Vai-e-volta sobre a mesma linha,
dentro de **um** traço, pincel de dureza 0:

| passadas | banda 10-90% |
|---|---|
| 1 | **3,248 px** |
| 3 | 1,962 |
| 15 | **1,436 px** |

**2,26× de endurecimento** — o Painter mediu 2,56× (3,53 → 1,38) pelo outro mecanismo.

#### (2) ⚠️ A RESSALVA DO §2.4 NÃO ALCANÇA O PERCURSO

O §2.4 avisou: *"o ponto fixo dessa recorrência é `max_k(w_k)` — o mesmo `max` que o Painter já
reprovou por beading"*. **Isso era verdade num buffer de dabs e é falso aqui.** O beading era
*estrutura por-dab ficando à vista* — uma fileira de discos. No contínuo, `max_s w(s)` sobre um
caminho suave **é o perfil avaliado na distância ao caminho**: um campo LISO, sem período, sem
discos. Não há o que ficar à vista.

⇒ o risco principal que mantinha o item 5 em espera **não existe neste motor**.

#### (3) E ela funciona — perfeitamente, o que é o problema

Computando o teto na sonda (`min(cover, w(dn_min)·edge)`, com `dn = dist/r` e `r = dist − sd`, os
dois já disponíveis na silhueta):

| dureza | banda HOJE (1 · 3 · 15) | banda com a 3ª LEI (1 · 3 · 15) |
|---|---|---|
| 0,0 | 3,248 · 1,962 · **1,436** | **5,485 · 5,485 · 5,485** |
| 0,2 | 2,630 · 1,738 · **1,536** | **4,481 · 4,481 · 4,481** |
| 0,5 | 1,785 · 1,369 · **1,583** | **2,916 · 2,916 · 2,916** |

**A banda fica IDÊNTICA em 1, 3 e 15 passadas** — não *quase*: o mesmo número. É literalmente o que
o Krita promete (*"soft edges remain soft during painting"*), e no contínuo ele entrega exato.

⚠️ **E é a mesma coluna que traz o preço: a borda de UMA passada vai de 3,248 para 5,485 px, +69%.**
A terceira lei não corrige só a sobreposição — ela redefine a borda de **todo traço macio**, porque
diz que a borda é o perfil do bico, não o resultado da varredura.

#### A decisão é de LOOK, e é do Enio

- **ganho:** acúmulo dentro do traço desaparece por completo, e o beading que assombrava a ideia
  **não pode acontecer aqui**;
- **preço:** todo traço macio fica com a borda ~69% mais larga — a aparência aprovada muda;
- **não é conserto, é MODO:** acumular ao passar por cima é o que tinta faz, e é o que o build-up do
  GIMP entrega. O Krita ship as duas justamente porque é gosto.

**Nada construído** — um flag de lei sem alguém que o autore seria botão morto, e a escolha entre
duas aparências corretas não é minha. O número do endurecimento fica pinado em
`a_soft_edge_hardens_when_the_stroke_crosses_itself_and_this_is_its_number` (descrição, não
veredito) e a receita da 3ª lei está na sonda irmã, pronta para virar produto no dia em que a
resposta vier.

#### ⚠️ 2026-07-31 — a decisão tem uma SEGUNDA metade, e ela não estava aqui

A 3ª lei limita cada pixel pela cobertura do **próprio bico** ali. Isso não capa só o endurecimento
da borda: capa **toda acumulação dentro do traço** — e o Flip shipou o **SELF OVERLAP** em
2026-07-27 (*"um traço que cruza a si mesmo fica mais escuro no cruzamento"*), que é exatamente uma
acumulação dentro do traço. Medido
(`measure_whether_the_third_law_also_switches_off_the_self_overlap`, X de um traço, opacidade 0,5,
`FLAG_SELF_OVERLAP` ligado):

| dureza | braço HOJE | cruzamento HOJE | ganho | cruzamento 3ª LEI | ganho |
|---|---|---|---|---|---|
| 1,0 | 0,5000 | 0,7500 | **1,50×** | 0,5000 | **1,00×** |
| 0,5 | 0,5000 | 0,7500 | **1,50×** | 0,5000 | **1,00×** |
| 0,0 | 0,5000 | 0,7500 | **1,50×** | 0,5000 | **1,00×** |

⇒ **A 3ª lei capa o cruzamento no valor exato do braço**, em toda dureza: o toggle de Self Overlap
fica **inerte dentro do traço**. Ela não é *um modo ao lado do que existe* — ela é
**mutuamente exclusiva** com uma feature shipada, como o One-Way e a zona de força da física (cada
controle é morto no modo do outro, e o painel tem de dizer isso em vez de deixar o artista
descobrir). O preço do item 5 passa a ter duas metades: a borda de uma passada **+69 %** *e* o
cruzamento **1,50× → 1,00×**.

⚠️ **E a sonda custou TRÊS fixtures erradas, todas minhas — a lição é a de sempre.** (1) Ela lia o
`cover` (geometria × tinta) para perguntar *"escureceu?"*, quando o `opacity` entra **depois** dele,
no alfa da cor (a regra do GP que o `tau.rs` documenta): mediu 1,0000 em tudo. (2) Não ligava o
`FLAG_SELF_OVERLAP` — o `art` não o liga —, então media o toggle DESLIGADO fazendo o que promete.
(3) O X era desenhado *vai, volta, sobe*, então ele cruzava num **VÉRTICE**, onde as passagens são
contíguas e a partição não vê duas. Nas três a sonda reportou **1,00× sobre um motor que funciona**,
e a única coisa que a salvou foi o gate `the_self_overlap_composes_only_where_the_stroke_crosses_itself`
já ter uma fixture que contém o fenômeno.
