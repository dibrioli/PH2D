# HANDOFF DE INTEGRAÇÃO — `line/FLIP`: o cruzamento e a curva (2026-07-28)

> **Missão do Enio** (com 5 screenshots): *"Nosso Flip desenha horrivelmente seus traços. Cria
> horrivelmente seus vértices. Nosso traço com hardness < 1 — veja onde se cruzam. Muito ruim."*
> Mais: ir ao fonte do Blender 5.2 e apps GP-like, e estudar os 13 pincéis para trazê-los.
>
> **5 commits, todos medidos, todos com mutação.** Estado: `line/FLIP`, tip `c0589697a`.
>
> ⚠️ **LEIA A §9 ANTES DE QUALQUER COISA.** O smoke do Enio REPROVOU esta wave (*"não vejo
> nenhuma diferença nesses smokes"*) e a causa era outra, achada com foto lado a lado do
> Painter. A §9 é a wave que de fato responde ao que ele fotografou.

---

## 1. O que estava errado, e o que a medição disse

A pesquisa paralela (5 agentes: fonte do GP 5.2 · os 13 pincéis · apps GP-like · o estado da
arte fora do Blender · auditoria do nosso código) produziu **duas** causas, e nenhuma delas era
a que o comentário do nosso próprio shader apontava.

### ⛔ A hipótese que o shader me vendeu, e que a medição REFUTOU

O cabeçalho do `flip.wgsl` declarava *"teto conhecido: sobreposição com vizinhos i±2 e
auto-cruzamento NÃO-adjacente seguem first-wins (semântica do GP, **pinada em teste**)"*. Ela
mandou a investigação direto para "o cruzamento volta ao first-wins".

**As três partes eram falsas.** A união alcança i±k E o auto-cruzamento não-adjacente (é para
isso que existe `seg_extras`, o 4º binding do próprio shader), e **o teste citado nunca foi
escrito**. Medido: um laço macio que se auto-cruza sai com **0 pixels divergentes de 57.699**
contra a união analítica — a união já resolvia aquilo.

*Um comentário que contradiz o código shipado é pior que comentário nenhum.* Corrigido no
commit `2c953cff4`.

### ✅ A causa real nº 1 — a seleção de vizinhos era ESTRUTURALMENTE incapaz

```
alcance / passo  =  3r / 0,8r  =  3,75      ⇒  i±1 … i±4 SEMPRE na lista
```

As duas grandezas são proporcionais ao raio, então a razão é **constante** — o pincel some da
conta. Os ~6 vizinhos da própria fita entram **antes de existir cruzamento**, e — ordenados por
distância, que neles é ~0 — **ganham os 16 slots**. O segmento que de fato cruza cai fora,
aquele pixel volta ao first-wins, e a GPU pinta a cauda macia de uma passagem sobre o **núcleo**
de outra. É a foto do Enio.

⚠️ **A reamostragem que shipou em 27/07 criou essa razão.** Não é bug antigo: é a interação de
duas waves.

**Cura:** partição com **dois orçamentos**, e a definição se escreve sozinha — *vizinho da mesma
passagem = alcançável andando pela polilinha dentro do alcance*. É comprimento de **arco**, não
um `k` escolhido. (`5396f3ae5`)

### ✅ A causa real nº 2 — a parametrização da reamostragem cuspidava

`centered_tangent` era Catmull-Rom **uniforme**, com um comentário afirmando que *"uniforme
basta porque as quinas são tratadas à parte"*. O regime perigoso não é o giro **agudo**: é o
**espaçamento desigual com giro gentil** — e espaçamento desigual é o que o **RDP produz**. O
par RDP→reamostragem **fabricava** o caso ruim. (`4652c980b`)

---

## 2. Os números (todos de sonda, todos reprodutíveis)

| Wave | Antes | Depois |
|---|---|---|
| **W1a/b** hachura densificada vs união | **FAILED, desvio 178** (pintou 31, pede 209) | ok |
| **W1d** tinta do grampo sub-pixel `r=0,20` | `0,170` do esperado `0,352` | `0,352` |
| **W1d** idem `r=0,45` | `0,813` de `0,973` | `0,973` |
| **W2a** giro máx. da saída, espaçamento 50:1 | **158,5°** (cúspide) | **29,6°** |
| **W2a** desvio à polilinha, 50:1 | **5,55** (span de corda 2,83) | 1,51 |
| **W2a** inflação do arco, 50:1 | 1,081 | **1,002** |
| **W2b** pontos mudos numa reta | 3 entram, **49 saem, 46 mudos** | **0** (saem 3) |
| **W2b** arco de 90° (controle) | 33 | **33, intacto** |
| **self_overlap** erro vs lei de tinta | **43/255 (17%)** | ⛔ **ABERTO** |

---

## 3. Commits

| # | O quê |
|---|---|
| `5396f3ae5` | **W1a+b** — a fita local ganha orçamento PRÓPRIO (arco); cap **16**, MEDIDO |
| `89f928239` | **W1d** — o piso de largura mínima chega ao laço de extras |
| `4652c980b` | **W2a** — Catmull-Rom **CENTRÍPETA** (α=½) |
| `2c953cff4` | **docs** — o cabeçalho do shader para de mentir; o `self_overlap` ganha número |
| `c0589697a` | **W2b** — span já reto ganha ZERO pontos |

---

## 4. Superfície tocada

- **Nenhum schema** (`PROJECT_SCHEMA` 37 · `FLIP_SCHEMA` 12 — intocados).
- **Nenhum contrato congelado** (§6): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/
  `PanelEvent=4` intactos, conferido por grep.
- **Nenhum ADR novo, nenhuma crate nova, nenhuma dep nova, nenhum `Cargo.toml` tocado.**
- **Nenhum id/token/i18n.**
- ⚠️ **Uma assinatura interna mudou:** `flip_smooth::resample_smooth` 3→4 args (`flat_tol`).
  `pub(crate)`, **um único chamador** (`flip_draw::stroke_from_samples`).
- **LOC:** `flip_smooth.rs` cruzaria 605 > 600 ⇒ `mod resample_measurement` foi para o irmão
  `flip_smooth_resample_tests.rs` por `#[path]` (segue FILHO, `use super::*` alcança privados).

### Mudanças de comportamento (o smoke decide)

1. **Toda curva reamostrada muda de forma** onde o espaçamento é desigual — é o fim da cúspide,
   mas é mudança visível. Espaçamento par: matematicamente idêntico (não *bit*-idêntico).
2. **Retas saem com menos vértices.** Subtrativo por construção (a tolerância é a do próprio RDP).
3. **Cruzamentos de pincel macio pintam certo** — o alvo da missão.

---

## 5. ⚠️ ABERTO, com o preço medido

### W1c — `self_overlap` conta DUAS vezes (**43/255, 17%**)

Cada face que passa o depth computa a **união GLOBAL**, então as `N` faces sobrepostas compõem
`1−(1−u)^N` com o MESMO `u` — a cobertura da passagem mais **próxima**, creditada `N` vezes.
`N` é o número de **QUADS**, não de **PASSAGENS**. A assinatura é exata: todo pixel sai em
`1−(1−OFF)²`, inclusive onde a 2ª passagem está a `dn = 0,82` e deveria contribuir ~nada.

⚠️ **O default é `self_overlap: false`** ⇒ isto **não** é o artefato que o Enio fotografou.

**Desenho da cura** (não construído): a lista de extras **particionada por passagem** — a fita
local já é separada desde o orçamento próprio; falta agrupar os extras de GRADE em runs de
índice consecutivo (marcador no bit alto de `seg_extras[].x`) e expor o `ribbon_count` — mais
**depth de volta a por-TRAÇO**, o que mata junto a colisão de `f32` do degrau por-segmento em
`sid` alto. Sonda pronta: `measure_the_self_overlap_double_count`.

### W3 — joins & caps (miter/bevel/round + butt/round/square)

Avaliado e deixado para **wave dedicada**: mexe na cobertura-união, a joia que custou uma semana
de bugs. O desenho que a pesquisa achou: **três tipos de quina = três métricas de distância** no
fragment (é o que o GP 5.2 faz), e o nosso `min` de cápsulas já é a estrutura certa para hospedá-las.

### W4 — os 13 pincéis (**a metade da missão que NÃO foi construída**)

A pesquisa está feita e não deve ser refeita. Achados que decidem o desenho:

- Os 13 são **assets** no 5.2, re-autorados à mão — **versionar não traduz**.
- `unprojected_radius` é o autoritativo (não `size`).
- **Marker Chisel não é uma elipse:** é pressão modulada pelo ALINHAMENTO com um eixo de 35°
  (`fac = 1 − |cos|`, `interpf(..., 0.3)`).
- **3 dos 13 usam `GP_MATERIAL_MODE_DOT`** — que nós já temos como `FlipStroke::tip`.
- O que falta de motor: **curva de pressão** (reusar `ph2d-curve`, já existe) e
  **pressão→opacidade** (hoje só largura).

### Menores

- `MAX_SUB_PER_SPAN = 24` segue sendo um cap de contagem, não de tolerância — com a regra de
  planeza ele quase nunca morde, mas não foi re-medido.
- A reamostragem roda a cada frame do drag (como o RDP); custo por-frame não medido.

---

## 6. Como rodar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP

# gates de unidade
cargo test -p ph2d-flip-render --release
cargo test -p ph2d-host-desktop --release --bins flip_smooth

# ⚠️ os gates GPU sao #[ignore] e PRECISAM de adapter (sem ele fazem skip, que nao e verde)
cargo test -p ph2d-flip-render --release --test gpu_render -- --ignored

# as sondas (todas imprimem tabela)
cargo test -p ph2d-flip-render --release --test gpu_render measure_ -- --ignored --nocapture
cargo test -p ph2d-host-desktop --release --bins measure_ -- --nocapture
```

### Smoke — o que o Enio tem de olhar

1. **O cruzamento** (a foto da missão): pincel **macio** (hardness < 1), traço que cruza a si
   mesmo e **hachura** que volta sobre si a meio raio. O núcleo de uma passagem não pode ser
   pintado pela cauda macia da outra.
2. **A curva** — desenhe devagar e depois rápido, com curvas fechadas e trechos retos no MESMO
   traço (é o que produz espaçamento desigual). Nenhuma barriga/bico onde a mão não fez.
3. **Zoom OUT bem longe** com hachura densa: a densidade da tinta não pode cair no encontro das
   passagens (W1d).
4. Retas continuam retas e nada ficou facetado (W2b não pode ter tirado densidade de curva).

---

## 7. Lições que valem além desta linha

- **Uma razão entre duas grandezas proporcionais ao mesmo parâmetro é CONSTANTE** — foi assim que
  o alcance/passo tornou os cruzamentos estruturalmente incapazes de caber. Procure essa forma.
- **Uma fixture pode não conter o fenômeno mesmo exercitando o código certo:** no W1d o **X**
  percorre o laço de extras e fica byte-idêntico; só o **quase-paralelo** enxerga o piso que
  falta. O X virou o CONTROLE justamente por isso.
- **Um `debug_assert` sobre uma constante não pode falhar** (clippy pegou o meu), e **uma
  constante que ninguém lê é um knob que não gira** — as duas saíram.
- **Um gate pode pinar o desperdício como contrato:** `rp.len() > 3` sobre uma RETA exigia os 46
  pontos mudos. A metade real dele sobreviveu; a asserção incidental morreu.
- **Meça no lugar certo:** eu quase reportei "sem ganho" por medir a razão de tinta num par de
  telas onde a cópia é ruído.

---

## 9. ⬛ SEGUNDA ONDA — a LEI DA DUREZA (o que a foto de fato mostrava)

**O smoke do Enio reprovou as §1-§8:** *"Não entendi nada. Vc mudou alguma coisa? Não vejo
nenhuma diferença nesses smokes."* — com uma foto nova, empilhando o MESMO cruzamento nos dois
módulos: *"o cruzamento de cima é o FLIP, o de baixo é do Painter. O correto é o aspecto do
cruzamento de baixo e o flip deveria ser idêntico"*.

Ele estava certo, e as §1-§8 não estão erradas: elas curaram *quais segmentos competem pelo pixel*
e *onde os vértices caem* — coisas que aparecem em hachura densa e em zoom-out sub-pixel, e que
um X largo não exercita. **O perfil da tinta eu não tinha tocado**, e era ele a foto.

### 9.1 A causa — duas leis com o mesmo nome

| | `t < h` (miolo) | como cai |
|---|---|---|
| **Painter** `falloff_weight` | **1.0 — platô sólido** | curva `Smooth` só em `[h, 1]` |
| **Flip** `hardness_mask` | *nada* — sem platô | `smoothstep(0,1, (1−dn)^(10·(1−h)))` desde o centro |

O `dn` onde a tinta cruza meia-tinta — a metade **VISÍVEL** da largura pedida:

| hardness | Flip (era) | Painter |
|---|---|---|
| 0,9 | 0,500 | 0,951 |
| 0,7 | 0,207 | 0,850 |
| 0,5 | **0,130** | 0,751 |
| 0,3 | 0,095 | 0,651 |

Em hardness 0,5 a largura visível era **13% da pedida**; o resto era névoa, e é isso que lia como
um filete brilhante dentro de um borrão. ⚠️ **O Flip estava FIEL ao Grease Pencil** (é o
`gpencil_stroke_round_cap_mask` ao pé da letra) — a ordem do Enio **sobrepõe a fidelidade ao GP**,
e a razão técnica concorda: a mesma palavra "Hardness" governando duas leis em dois módulos do
mesmo app é falha de duas-portas, silenciosa porque nenhum número aparece na tela.

**A lei agora é a do Painter** (`BrushSpec::falloff_weight` + `Falloff::Smooth`), escrita com as
mesmas operações na mesma ordem. ⚠️ **`hardness ≥ 1` é byte-idêntico nas duas** e
`DEFAULT_HARDNESS = 1.0` ⇒ **o traço padrão do Flip não se move**.

### 9.2 ⚠️ ESTA SEÇÃO ESTAVA ERRADA — ver §11

Ela dizia que casar o **DEPÓSITO** do Painter fora *"considerado e REJEITADO por medição"*, porque
o acumulado depende do spacing. O argumento é verdadeiro e a conclusão era errada: o residual que
ela mandou o smoke decidir valia **−112 de 255** contra o depósito real, e o smoke o reprovou duas
vezes. A correção completa está em **§11**.

### 9.3 A cadeia de prova, e o elo que faltava

A lei vive em **dois idiomas** (WGSL no device · Rust no Painter) e o oráculo de união dos testes
tinha uma **terceira** cópia (`cpu_mask`). A cadeia é `shader ≡ cpu_mask ≡ Painter`:

1. Os **9 gates de união** provam (1) — e todos os 9 caíram de uma vez quando só o shader mudou,
   que é como essa dívida se cobra.
2. **`the_union_oracle_is_the_painters_law`** (gate NOVO, sem adapter) prova (2).

⚠️ **Sem (2) os dois lados podiam derivar JUNTOS e tudo ficava verde** — foi literalmente o estado
até esta wave. **Provado por mutação:** revertendo os DOIS ao GP, **os 9 gates de união passam** e
só (2) + o gate do airbrush sangram.

### 9.4 O gate do airbrush foi RE-DERIVADO (não afrouxado)

Ele afirmava *"o padrão DESABA no meio-raio"* — **falso por projeto** com o platô — e comparava o
EIXO, onde a asserção `air_axis > std_axis + 15` **inverteu** (medido: padrão **255**, airbrush
**252**). O discriminante **mudou de lugar, do centro para o ARO**, e **não encolheu**: `dn≈0.8`
**55 vs 231** · `dn≈0.9` **7 vs 192** (delta máximo 0,76 sobre `dn`). Mutação: a flag ignorada ⇒
os dois perfis viram o mesmo ⇒ RED.

### 9.5 Superfície

- ⚠️ **Um `Cargo.toml` tocado** (a §4 dizia zero): `ph2d-painter-brush` entra em
  **`[dev-dependencies]`** da `ph2d-flip-render`. Crate **FOLHA** (`[dependencies]` vazio), o
  `src/` não a toca ⇒ **machete-safe** (conferido: `cargo machete` limpo) — o precedente do gate
  de paridade CPU×GPU da `line/gpu-nodes`.
- **Nenhum schema** (`PROJECT_SCHEMA` 37 · `FLIP_SCHEMA` 12), **nenhum contrato congelado**,
  **nenhum ADR**, **nenhum id/token/i18n**.
- Arquivo novo: `shells/desktop/src/flip_hardness_smoke.rs` (+ `mod` no `main.rs` e a chamada no
  prólogo, ao lado dos outros smokes).
- **SEIS notas corrigidas** que descreviam o airbrush como *"o oposto do pico do `pow`"* — o
  default mudou, então a frase virou falsa; os números do doc 03 §8 e do smoke do airbrush foram
  **refeitos com medição**, não reescritos por analogia.

### 9.6 Smoke

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

Três cruzamentos, hardness **1.0 (o CONTROLE, byte-idêntico) · 0.7 · 0.4**. O que olhar: cada
traço tem **miolo sólido com borda macia** (nunca um filete dentro de um borrão); o cruzamento
funde liso; e **o mesmo gesto no Painter, na mesma hardness, tem de ler igual**. A cena imprime a
tabela medida e o residual conhecido da §9.2.

⚠️ Re-rodar também **`PH2D_FLIP_AIRBRUSH_SMOKE=1`** — o contraste dele mudou de lugar e a
mensagem foi reescrita; é o smoke que confirma que o airbrush segue distinto.

---

## 10. ⬛ TERCEIRA ONDA — o auto-cruzamento COMPÕE (§8.7 do doc 03)

**2º smoke do Enio, com foto:** *"funcionou muito bem se são dois traços cruzados. Mas se o mesmo
traço cruza a si mesmo então temos o mesmo aspecto indesejado"*.

⚠️ **O relato separa duas ROTAS DE CÓDIGO — é o diagnóstico inteiro.** Traços distintos têm depth
diferente ⇒ o mais novo pinta por cima ⇒ **composição `over`** (lisa). Um traço cruzando a si mesmo
tem o MESMO depth ⇒ o `GREATER` estrito descarta o 2º quad ⇒ **união** (`min`), que tem **VINCO** na
bissetriz. Hardness 1 esconde (máscara binária); pincel macio mostra.

**Medido** (o MESMO X como um traço e como dois, disco de raio 18 no cruzamento): hardness 0,7
**35/255 → 1/255**; hardness 0,4 **48/255 → 1/255**, zero pixel fora de 8.

**Cura:** a lista de vizinhos vem **particionada por passagem** (`neighbors::SegExtras`) e o
fragment faz `1 − (1−mask_própria)(1−mask_estranha)`. **Compõe-se a COBERTURA, nunca o ALFA** ⇒ um
traço a opacity 0,5 segue sem escurecer sobre si mesmo (a regra do GP, gateada). **Sem cruzamento é
byte-idêntico por construção.**

### ⚠️ Três coisas que a medição derrubou, e que valem mais que o fix

1. ⛔ **Cortar a passagem por ARCO (a v1) está ERRADO — não re-derive.** Curva fechada volta a
   ficar perto com arco grande ⇒ a fita compunha consigo mesma (196 onde a união pede 184).
2. ⚠️ **O transbordo do teto da fita tem de degradar para UNIÃO, nunca para composição** — com o
   `return` no cap o teto passava a *adicionar* tinta.
3. ⚠️ **O teste espacial só é honesto sobre polilinha REAMOSTRADA** (segmentos enormes têm
   distância medida pelas pontas) ⇒ as fixtures dos gates novos são **densas**.

### Gates

- `a_stroke_crossing_itself_paints_what_two_crossing_strokes_paint` — oráculo = o desenho
  aprovado. ⚠️ O irmão antigo usa `hardness = 1.0` e sob a mutação fica **VERDE**.
- `a_dense_soft_ribbon_that_never_crosses_itself_is_exactly_the_union` — a garantia de segurança;
  **nasceu de uma mutação que sobreviveu** (os 9 gates de união usam raio 4, onde a faixa de AA
  engole o ombro e eles viram teste binário).

**3 mutações, 3 sangram.** ⚠️ **Resíduo NOMEADO:** em `hardness = 1.0` o traço único fica ~11
níveis mais cheio que dois traços na franja de AA mais externa da bissetriz (240 × 229) — fora do
disco medido, e na direção de MAIS tinta. É conflação de cobertura correlacionada, inerente a
compor AA; não perseguido.

### Superfície

`neighbors::extras_for_stroke` passa a devolver `Vec<SegExtras>` (interno à crate) e a 2ª palavra
de `seg_extra_range` carrega `count | (ribbon << 16)` — **sem campo novo, sem mudar a BGL, sem
varying novo**. Nenhum schema, contrato congelado, ADR, id ou token.

### Smoke

**`PH2D_FLIP_HARDNESS_SMOKE=1`** — a MESMA cena serve: os três cruzamentos são de **um traço só**.
O que olhar agora: o cruzamento **funde liso**, sem costura na bissetriz, e desenhar o mesmo X com
**dois traços separados** tem de dar o mesmo aspecto.

### 10.1 — A QUINA: o transbordo do teto tem DOIS destinos errados (3º smoke)

*"Melhor mas não completamente; principalmente na quina do traço o problema aparece"* — setas
vermelhas sobre **cunhas ESCURAS** mordendo a tinta.

⚠️ **Nenhuma sonda anterior podia ver isto** (a de quina não cruzava; a de cruzamento tinha a quina
fora do quadro): o defeito vive na **INTERAÇÃO**. A quina afiada alonga a caminhada da fita, o teto
`MAX_RIBBON_EXTRAS` satura, e **o destino do transbordo decide tudo**. Medido no "4":

| destino do transbordo | falta | sobra | px fora de 8 |
|---|---|---|---|
| carimbado (some das 2 listas) — **a §10 shipou isto** | **−252** buraco | +4 | 167 |
| não carimbado (vira estranho) | −1 | **+63** | 109 |
| ao grid, **MARCADO** como própria | **−1** | **+4** | **0** |

**DOIS carimbos:** `walked` (*a caminhada chegou aqui* ⇒ mesma passagem) e `stamp` (*já está na
lista* ⇒ o grid não duplica). O transbordo perde só o `stamp`, então o grid o recolhe e a partição
por `walked` o devolve à própria passagem.

⚠️ **Faltar e sobrar não são o mesmo erro:** sobrar sobre a união é a lei nova; **faltar é sempre
defeito**. Um gate para cada (`a_sharp_corner_that_crosses_itself_never_loses_ink` ·
`a_dense_soft_ribbon_that_never_crosses_itself_is_exactly_the_union`); **2 mutações, cada uma
derruba só o SEU gate**. Corolário: o teto do laço do fragment é a SOMA dos dois orçamentos (32).


---

## 11. ⬛ QUARTA ONDA — o perfil é o do **TRAÇO** do Painter, não o de um **DAB** dele

**Ordem do Enio** (4ª rodada, com foto anotada de um rabisco em estrela): *"É assustador como vc
não consegue resolver. Tudo que quero é que tenha o aspecto do traço do nosso próprio módulo
painter digital"* — setas vermelhas sobre cunhas ESCURAS nas quinas.

### 11.1 O que a medição disse (a sonda que faltava)

⚠️ **Toda sonda desta linha comparava o Flip com a UNIÃO** (`expected_union_alpha`) — e a união
era exatamente a coisa sob suspeita. A sonda nova (`tests/painter_look.rs`) compara com o
**DEPÓSITO DO PAINTER**: dabs a `spacing × diâmetro` de arco compostos por `over`, com o `Falloff`
REAL da crate dele, na figura da foto (a estrela de UM traço: quina de 36° em cada ponta, cinco
auto-cruzamentos).

| perfil do Flip | falta de tinta | px fora de 16 |
|---|---|---|
| a queda de UM DAB (ondas 2-3) | **−112 de 255** | 613 |
| o perfil de TRAÇO (esta) | **−4** | 166, **todos de SOBRA** |

A cura cabe numa frase: **o Flip desenha um TRAÇO, então o perfil dele é o perfil de TRAÇO do
Painter, nunca o de um DAB dele.** Em hardness 0,4 e `dn = 0,70` um dab pesa **0,500** e o traço
pesa **0,916** — era essa a distância entre as duas fotos.

### 11.2 Por que "assar o spacing do Painter" está certo aqui

A regra que esta linha pagou quatro vezes é *a lei é função do CAMINHO, nunca de quão fino o MOTOR
amostrou o caminho*, e ela segue honrada: a máscara continua função **pura** da distância ao
caminho. O `spacing` é propriedade do **pincel que estamos igualando**, e o pedido nomeia esse
pincel. Preço NOMEADO: `DEPOSIT_STEP` é espelho de `spec_default.rs:29`.

### 11.3 Três coisas que a medição entregou de brinde

- **A composição por passagem deixou de ser heurística** — o produto sobre todos os dabs **fatora
  por passagem**, então o `1 − (1−P₁)(1−P₂)` da §10 virou a fatoração exata.
- **Num traço RETO o Flip É o depósito do Painter, ao ±1 de 255.**
- **A fase da grade de dabs é irrelevante, e foi MEDIDA:** meio passo move o perfil em **0,003**.

### 11.4 Resíduo NOMEADO (o único)

No canto **CONVEXO** os dabs do Painter **recuam** em vez de correr paralelos e o perfil de traço
os superestima: a ponta do Flip fica mais cheia (**+122/255** no vértice de 36°; +13 em hardness
0,8, **zero** em 0,9). É a direção **oposta** à queixa. Medido em toda a faixa de hardness:
**ZERO** pixel com menos tinta que o Painter. Fechá-lo exige o produto de dabs por posição de arco
no fragment — wave própria.

### 11.5 Gates (5 elos) e mutações

1. `shader ≡ cpu_mask` — os 9 gates de união.
2. `cpu_mask ≡ o DEPÓSITO de ph2d_painter_brush` — `the_union_oracle_is_the_painters_law`.
3. `o depósito ≠ um dab, e este é o número` — **`the_stroke_profile_is_fuller_than_a_single_dab`**.
4. `o laço ±4 é a fileira INTEIRA` — **`the_shaders_dab_row_is_the_whole_row`**, que **parseia as
   constantes do `flip.wgsl`** em vez de repeti-las.
5. `o produto pinta o que o Painter deposita` — **`the_flip_paints_what_the_painters_digital_brush_deposits`**,
   na figura da FOTO.

**Mutações:** (M1) o shader volta à queda de um dab ⇒ 3 gates de união sangram e (5) acusa **796 px
com menos tinta, pior −122**. (M2) apertar o laço para ±2 nos DOIS lados da paridade ⇒ a paridade
GPU fica **VERDE** e só (4) sangra — a prova de que (4) não é redundante.

### 11.6 ⚠️ Duas armadilhas de ORÁCULO, as duas minhas

1. **Superamostrar o oráculo 4×4 foi tentado e REVERTIDO.** Ele mede uma verdade que *nenhum dos
   dois produtos calcula* — o Painter também avalia a queda no centro do texel. Com 4×4, em
   hardness 0,8 a faixa de queda mede 1,4 px e a média de área discorda da amostra pontual por
   **−67/255**, penalizando o Flip por algo que o Painter faz igual. *O oráculo tem de amostrar
   como o produto amostra.*
2. **A franja da silhueta é pulada:** o Flip fecha a borda com AA, o depósito não tem termo de AA
   nenhum. Comparar ali mede convenção de borda, não a lei.

### 11.7 Superfície

`flip.wgsl` (`hardness_mask` + 2 consts) · `tests/gpu_render.rs` (`cpu_mask` + 2 gates) ·
`tests/hardness_law.rs` (header corrigido + 1 gate) · `tests/painter_look.rs` (NOVO) ·
`flip_hardness_smoke.rs` (a cena passou a encenar **a foto**: a estrela de um traço) ·
`docs/Flip/03` §8.6. **Nenhum schema, nenhum contrato congelado, nenhum ADR, nenhum id/token.**

### 11.8 Smoke

```bash
env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena tem **três** grupos: X duro (o CONTROLE, byte-idêntico), X macio, e **a ESTRELA de UM
traço** — a foto encenada. ⚠️ As ondas anteriores só encenavam X de DOIS traços, e dois traços
cruzados **nunca** tiveram o defeito (o depth difere e o mais novo já compõe): por isso ele só
aparecia quando o Enio desenhava à mão.
