# 10 — A LUZ QUE ATRAVESSA A PEÇA (a subsuperfície)

> **Ordem do dono, 2026-09-17.** Posto perante os três ingredientes que faltavam ao
> [`01`](01_o_alvo_decomposto.md) — a translucidez (`6`), o pós (`7`) e o estilo (`8`) —, ele
> escolheu **«a luz atravessa a peça»**: *folha, jade, cera, mármore fino*.

⚠️ **Esta wave está NO PRODUTO.** A lei, a curvatura, o painel, os dois motores e a cena `=33`.

---

## §1 — ⭐⭐⭐ São DOIS caminhos e não um, e a diferença é a PEÇA

O `open_pbr_surface` escolhe entre eles pelo `geometry_thin_walled`, e eles respondem a perguntas
diferentes:

| | a peça | a lei | o que se vê |
|---|---|---|---|
| **parede fina** | folha, pétala, papel, abajur | oren-nayar à frente **+ `translucent` por trás**, meio a meio, com a fase a pesar os dois lados | com a luz **ATRÁS**, ela acende inteira |
| **maciça** | jade, cera, mármore fino, leite | o perfil de difusão de **Burley** integrado sobre a **curvatura local** | a luz **contorna** a quina, e o terminador amacia |

⭐ **A wave inteira da parede fina cabe numa linha** — o `mx_translucent_bsdf` faz `N = -N` e mais
nada, logo o `max(0)` dele é sobre `−N·L`. *É essa negação, e só ela, que faz uma folha acender com
o sol atrás.*

⚠️ **A parede fina não sabe nada sobre a forma da peça** e a maciça precisa de **uma** grandeza
geométrica. É essa diferença que faz a primeira custar zero e a segunda custar uma amostra de campo.

---

## §2 — ⭐⭐⭐ A lei maciça é a que ESTA CASA já tinha escrita

O [`ph2d_mesh_render::sss`](../../crates/ph2d-mesh-render/src/sss.rs) — a tabela pré-integrada de
Penner & Borshukov, portada em 2026-08 para o **esculpir** — integra **o mesmo integral**:

```text
D(θ, r) = ∫ clamp(cos(θ + x), 0, 1) · R(2r·sin(x/2)) dx  /  ∫ R(2r·sin(x/2)) dx
```

A mesma **corda** `2r·sin(x/2)`, a mesma normalização. O que muda:

| | o esculpir (2026-08) | o modelador (esta wave) |
|---|---|---|
| perfil `R` | soma de **seis gaussianas** (d'Eon & Luebke) | **Burley**, duas exponenciais (Pixar) |
| avaliação | **pré-integrada** numa tabela | `32` termos por amostra, como o oráculo |

⇒ *não é um sistema novo: é o mesmo fenómeno com o perfil que o oráculo desta linha usa.*

### §2.1 — ⭐⭐ E ela depende só do ÂNGULO e do quociente `mfp/raio` — medido

Escalar o raio e o caminho livre médio pelo mesmo factor deixa `shape·dist` invariante e `R` escala
por `1/k` **uniformemente**, logo `ΣD/ΣR` não se mexe. Medido sobre `16×`:

| `θ` | `r = 0,25 · mfp = 0,25` | `r = 1 · mfp = 1` | `r = 4 · mfp = 4` | desvio |
|---:|---:|---:|---:|---:|
| `0°` | `0,82041167` | `0,82041167` | `0,82041167` | `0,00e+00` |
| `90°` | `0,16056012` | `0,16056012` | `0,16056012` | `0,00e+00` |
| `180°` | `0,05095574` | `0,05095574` | `0,05095574` | `0,00e+00` |

⭐ A grandeza é o adimensional `mfp·κ` — **exactamente o eixo `t = scatter·|κ|` que o cabeçalho da
tabela do esculpir declara**.

---

## §3 — ⭐⭐⭐ A CURVATURA sai do CAMPO, e não de derivadas de ecrã

O renderizador de referência estima-a por `length(fwidth(N)) / length(fwidth(P))`.

⛔⛔ **Aqui isso era inexprimível sem partir a paridade de `100,000 %` desta linha:** o `fwidth` da
placa é por **quad de `2×2`** e a diferença do traçador de CPU seria **por pixel**.

### §3.1 — A lei, e porque ela quase não custa amostras

Num campo de distância `|∇f| = 1`, logo `H = ∇²f / 2`. E o Laplaciano sai da **mesma soma** que a
normal já percorre:

```text
Σⱼ f(p + ε·dⱼ) = n·f(p) + ε·∇f·Σdⱼ + (ε²/2)·Σ dⱼᵀ H dⱼ + O(ε³)
```

Nos dois estêncis `Σdⱼ = 0` e `Σ dⱼdⱼᵀ = c·I`:

| estêncil | `c` | `∇²f` |
|---|---:|---|
| `Central6` (`\|d\| = 1`) | `2` | `(Σfⱼ − 6·f(p)) / ε²` |
| `Tetra4` (`\|d\| = √3`) | `4` | `(Σfⱼ − 4·f(p)) / (2ε²)` |

⇒ falta **uma** amostra: a do centro. ⛔ E o estêncil desta lei é **fixo** no de quatro, mesmo
quando a normal usa seis — o traçador do dispositivo lê sempre por aquele, e seguir a escolha do
quadro faria os dois motores divergirem no dia em que ele mudasse.

### §3.2 — ⛔⛔ A premissa que a medição derrubou: o PASSO não é o da normal

A 1.ª redacção dizia *«`eps` é o mesmo que a normal usou; dois passos diferentes dariam duas
respostas para a mesma pergunta»*. **Falso, e o gate reprovou em voz alta:** com o passo da normal
uma face plana lia curvatura **`1,49`** e a esfera errava `109 %`.

*Uma primeira diferença divide por `ε` e uma segunda por `ε²`* ⇒ o cancelamento em `f32` entra
`1/ε` vezes mais cedo, e o óptimo da segunda é `~ulp^{1/4}` contra `^{1/3}` da primeira.

| `ε / raio` | `R = 0,4` | `R = 1,0` | `R = 2,5` |
|---:|---:|---:|---:|
| `0,0004` | `0,534` | `0,397` | `0,404` |
| `0,0016` | `0,033` | `0,022` | `0,025` |
| **`0,0064`** | **`0,0041`** | **`0,0041`** | **`0,0047`** |
| `0,0256` | `0,027` | `0,013` | `0,013` |
| `0,1024` | `0,054` | `0,054` | `0,054` |

⭐ **O vale está no MESMO sítio nos três**, e é isso que faz dele uma lei: abaixo manda o
cancelamento, acima manda a truncagem. ⚠️ E a escala é a da **PEÇA**, nunca a da vista — um passo
que seguisse o zoom daria duas curvaturas para o mesmo ponto.

### §3.3 — ⚠️ A divergência declarada

A referência devolve um **comprimento**, logo não distingue bossa de cova; esta porta devolve `|H|`
pela mesma razão. A diferença real é outra e é **a favor**: a dela é a curvatura normal na direcção
do **ECRÃ** (muda ao rodar a câmera) e a nossa é a **média** (não muda).

---

## §4 — ⭐⭐⭐ O oráculo, e porque ele ganhou uma LUZ DE TRÁS

O corpus é o `GlslRenderer` do MaterialX 1.39.5 corrido sem interface
([`ferramentas/fixture_openpbr.py`](ferramentas/fixture_openpbr.py)). Ele ganhou **4 materiais** e
**uma luz que viaja para `+z`**, isto é, de detrás da esfera contra a câmera.

⚠️ **Sem ela a fixture não contém o fenómeno que a parede fina existe para produzir:** nas três
luzes de sempre a transmissão lê `max(−N·L, 0) = 0` em quase todo pixel visível. *Um corpus sem o
fenómeno não afirma nada sobre a lei que o produz.* `945 → 1980` amostras directas, `315 → 495`
indirectas.

### §4.1 — A paridade, e as duas barras

| caminho | pior | barra | leitura |
|---|---:|---:|---|
| **parede fina** (materiais `7`, `8`) | `9,6e-6` | `1e-4` | a MESMA ordem do resto da fixture |
| resto do corpus (`0`..`6`) | `9,8e-6` | `1e-4` | inalterado |
| luz do céu (os `11`) | `1,6e-6` | `1e-4` | inalterado |

### §4.2 — ⛔⛔ E o caminho MACIÇO tem gate PRÓPRIO, com o porquê medido

A curvatura implícita de cada amostra do oráculo, sobre uma esfera cuja curvatura verdadeira é `1`,
espalha-se de **`0,02` a `54`**. ⚠️ **E não é a tesselação:** a mesma medição sobre uma esfera
`256×128` **nossa** devolve a mesma dispersão. *A grandeza é do ECRÃ, não da malha.*

⇒ um **máximo** sobre essa população mede o estimador **dele**. O gate afirma a mediana, os
quartis, e — a metade que de facto afirma a lei — que o mínimo em `κ = 1` é **AGUDO**:

| | `κ = 0,5` | **`κ = 1` (a verdade)** | `κ = 2` |
|---|---:|---:|---:|
| material `9` · `p50` | `0,0769` | **`0,0005`** | `0,0582` |
| material `9` · `p75` | `0,7307` | **`0,0073`** | `0,9589` |
| material `10` · `p50` | `0,0449` | **`0,0004`** | `0,0392` |

*Uma lei que não fosse a dele não teria vale nenhum na curvatura verdadeira.*

### §4.3 — ⛔ A divergência do `acos`

O `mx_acos` do MaterialX é o `acos` do GLSL **sem corte** (um `#define` directo em
`stdlib/genglsl/lib/mx_math.glsl`), e `dot` de dois unitários lê `1,0000001` em `f32` ⇒ o oráculo
devolve **`NaN`** ali. Esta porta corta a `[-1, 1]` — no-op dentro do domínio, e *um pixel `NaN` e
um pixel legitimamente preto leem-se iguais*.

---

## §5 — ⚠️ O `energy_compensation` é um ARGUMENTO, e o motivo é um valor de omissão

A nodedef `ND_oren_nayar_diffuse_bsdf` dá `energy_compensation = false`, e a **reflexão da parede
fina da subsuperfície** é a **única** closure do `open_pbr_surface` que o deixa por escrever — a
base escreve `true`. ⛔ Passar-lhe a compensada mudaria o número que o oráculo mede.

⛔ E o `stinv` do clássico é **`0` quando `s ≤ 0`**, onde o compensado guarda o `s` negativo: são
duas leis publicadas diferentes e não uma simplificação.

---

## §6 — ⭐⭐⭐ O que o artista ganha, e as três leis que a wave herdou

Seis linhas novas na secção do material: **Subsurface** (o peso) · **Subsurface Color** (amostra) ·
**Subsurface Radius** · **Subsurface Radius Scale** (amostra — *quanto mais fundo cada canal
viaja*, e é o que põe o vermelho à frente numa orelha) · **Subsurface Anisotropy** · **Thin
Walled**.

O painel é **derivado da tabela**, e foi isso que tornou a wave barata: nem uma linha de pintura
nova. As três leis herdadas:

1. **as seis ficam TRAVADAS com o peso a zero** — a lei do verniz (§21) numa quinta família, e a
   parede fina está entre elas porque com o peso a zero o caminho que ela escolhe nem é avaliado;
2. **a cor e a escala são AMOSTRAS**, pela tabela `CORES` que a ordem do dono de 14/09 criou;
3. **a escrita não se estreita** — o `set_param` continua a aceitar as `33` posições.

### §6.1 — ⛔ O `Thin Walled` é um booleano guardado como NÚMERO

A tabela de `FieldMaterial::get` é de `f32` e o painel é derivado dela; uma variante nova de linha
custaria a tabela inteira. Ele viaja como `Span::Choice` de dois (`Solid` · `Thin Walled`), logo o
arrasto é inteiro. ⏳ **Que ele se pinte como uma CAIXA fica NOMEADO e por fazer.**

---

## §7 — ⏱️ O degrau, o tecto e o que eles custaram

**`PROJECT_SCHEMA` `144 → 145`** — dez `f32` apendados ao `FieldMaterial`, o mesmo mecanismo dos
degraus `140`..`143` (`92` bytes onde o binário pede `132`).

⚠️ **APENDADOS e não na posição da NODEDEF**, e a troca é declarada: a subsuperfície vem **antes**
do verniz lá, logo re-numerar mexeria em `11` posições já gravadas — a quebra de layout que o
`142 → 143` pagou uma vez.

**`MAX_ROWS` do painel `79 → 85`**, pela lei que o doc dele já escrevia: a alternativa era baixar o
`MAX_POLYGON_VERTICES` de `27` para `24`, e *um tecto de REGISTO cujo recurso é memória a mandar
num tecto de FORMA é o caminho lento a definir o rápido* (§0.0). Preço medido: `6` widgets por
linha, `36` uma vez no arranque.

---

## §8 — ⭐ Os portões

| gate | o que afirma |
|---|---|
| `the_direct_light_is_the_oracles_on_every_sphere` | `9,8e-6` sobre `9` materiais, com a parede fina lá dentro |
| `o_caminho_macico_bate_o_oraculo_na_mediana` | `p50 5e-4` · `p75` · e o **mínimo AGUDO** em `κ = 1` |
| `uma_esfera_de_raio_r_le_curvatura_um_sobre_r` | `1/R` nos três raios **e** que a régua separa raios |
| `um_plano_le_curvatura_zero` | `0,000023` na face de uma caixa |
| `a_guarda_devolve_zeros_e_nunca_nan` | «não sei» é plano, nunca `NaN` |
| `every_number_a_material_has_reaches_the_law` | as `33` posições movem a radiância em pelo menos um dos dois caminhos |
| `a_luz_que_atravessa_a_peca_e_a_mesma_nos_dois_motores` | **`100,000 %`** nos dois, e cada um **move** a imagem |

---

## §9 — ⚠️ As premissas que a medição derrubou

1. *«o material FECHOU: são as `15` entradas do OpenPBR e não há mais nenhuma para apender»* —
   escrita em **três** sítios (a escada do schema, o `FieldMaterial::get`, o `MAX_ROWS`). Era
   verdade sobre a **fatia** de 14/09 e falsa sobre o modelo, que tem `41` entradas — e a
   `ph2d-material` declarava por escrito, na mesma semana, que cinco famílias ficavam *«nesta
   fatia»*. ⇒ *uma lista fecha-se contra o que se construiu, nunca contra o que existe.*
2. *«este tecto deixa de crescer por material»* (o `MAX_ROWS`).
3. *«o passo da curvatura é o da normal»* (§3.2).
4. *«a tesselação explica a dispersão do `fwidth`»* — a esfera `256×128` nossa dá a mesma.
5. *«o desvio do caminho maciço é a curvatura, e existe um valor que o fecha»* — a varredura de
   `0,05` a `27` fechou a porta: o melhor é `0,81`. *Se nenhum valor do parâmetro livre serve, o que
   está errado é a régua ou a lei* — e era a **régua** (o pior sobre a população).

---

## §10 — ⏳ O que fica

- **A ESPESSURA como entrada.** Nenhum dos dois caminhos da referência a lê — a maciça aproxima-a
  pela curvatura, e num campo de distância a espessura **mede-se** (uma marcha para dentro até o
  campo voltar a ser positivo). ⭐ É a mesma vantagem estrutural do [`02` §5.1](02_o_estado_da_arte.md),
  um nível abaixo, e é o candidato natural a **superar** a referência. ⛔ Fica aberta de propósito:
  a paridade desta wave é contra a lei da curvatura, e trocar a entrada sem oráculo é inventar.
- **A cena `=33` não põe a luz atrás sozinha** nem autora os materiais — um `FieldDoc` não carrega
  nenhum dos dois e o gancho não existe. O artista faz os dois gestos, que existem.
- **O `Thin Walled` como caixa** (§6.1).
- **O relógio desta wave não foi medido** — a máquina esteve entre `load 13` e `27` toda a jornada,
  e *nenhuma leitura acima de `load ~5` vale nada*. ⇒ vai para a **`W9`** ([`03`](03_o_plano.md)),
  que é onde o dono a pôs.
- **`transmission_*` · `fuzz_*` · `thin_film_*` · `geometry_opacity`** continuam fora, agora com a
  lição do §9 ao lado: a lista fecha contra o construído.

---

## §11 — ⛔⛔⛔ O report de 2026-09-18: a LINHA DURA no terminador, e o filtro que a previu por escrito

> *«Bom resultado! Avalie apenas uma coisa: em `Thin Walled: Solid` não há transição suave entre a
> área iluminada e a área sombreada da esfera, mas uma linha dura. Veja se é correto.»* — duas fotos
> da cena `=33`, a seta em cima do vinco.

**Não é correto**, e o mecanismo não está na lei da subsuperfície.

### §11.1 — A fixtura mentiu duas vezes antes de conter o fenómeno

A 1.ª sonda montou uma bola **sozinha na origem** e binou a luminância por `N·L`: o perfil
atravessou `N·L = 0` **liso** e o pior salto ficou em `0,78` — o **brilho especular**, que lê o mesmo
no material opaco. *Uma régua que procura o máximo global mede o realce, não o terminador.*

A 2.ª escreveu `radiance_at_one: [intensity; 3]` em vez de passar pela porta do produto
([`lights::radiance_at_one`], que é `cor × intensidade × π`): a lâmpada saiu **`π×` fraca e sem
cor**, o céu dominou, e o terminador leu-se como uma rampa de `99` a `108`. *Uma sonda que reescreve
a conversão do produto mede outro programa* — a quinta vez que esta casa o paga.

### §11.2 — A causa, com o canal isolado

Na `=33` a **LÂMINA projecta sombra sobre a ESFERA**. Ao longo de uma linha que atravessa o
terminador, a visibilidade da lâmpada lia:

| `N·L` | `0,054` | `0,038` | `0,020` | `0,002` | `−0,018` |
|---|---|---|---|---|---|
| `vis` | `0,984` | `0,849` | `0,712` | **`0,555`** | **`1,000`** |

⇒ **um degrau de `0,445` num pixel**, exactamente em `N·L = 0`, porque o passe de sombra escrevia
`vis = 1,0` para todo ponto de costas para a luz. ⭐⭐⭐ **E o comentário desse filtro previa o dia:**

> *«`1,0` e não `0,0`, e a diferença NÃO é visível hoje: o `N·L ≤ 0` já anula a contribuição da
> lâmpada … até ao dia em que alguém ler este canal para outra coisa (**a OpenPBR tem termos que
> recebem luz com `N·L < 0`**)»*

A subsuperfície **maciça** é o primeiro consumidor desta casa que lê luz do lado escuro. ⇒ a sombra
do vizinho acabava a meio, num degrau de **um pixel**, que é a linha que ele fotografou.

⚠️ **Não é acne:** uma esfera **sozinha** lê `0` de `12 924` pixels com `vis < 0,99` — o ergue pela
normal já a cura. *Uma sombra falsa e uma sombra truncada leem-se iguais numa foto.*

### §11.3 — Duas curas construídas, MEDIDAS e refutadas

| | banda do terminador, 2.ª dif p99 | salto no pixel | folha com a luz ATRÁS |
|---|---|---|---|
| **o defeito** | **`9,21`** | **`+4,4`** | `83,7` (intacta) |
| marchar o raio de costas como os outros | `2,14` | `−1,4` | ⛔ **`73,8`**, `vis` mín `0,000` |
| ⛔ o mesmo, sem recontar o `t` na saída | `7,42` | `−4,1` | ⛔ `74,0` |
| **hoje** | **`3,71`** | `+1,8` | ✅ **`83,7`** |
| a mesma cena sem lâmpada (o controlo liso) | `1,00` | `−0,9` | — |

⛔ **A cura óbvia mata a outra metade da wave:** o raio de um ponto de costas atravessa o **próprio
corpo**, lê-o como obstáculo, e a folha com o sol atrás — a razão de ser do caminho de parede fina —
**apaga-se**. *A pergunta certa não é «a minha peça está no caminho?» (está sempre, por construção),
é «há mais ALGUMA COISA no caminho?».*

### §11.4 — A lei que fica

O raio de um ponto **de costas** parte do ponto (⚠️ **sem erguer** — erguido, um raio que roça o
terminador pode nunca tocar no corpo, nunca sair e nunca acusar), anda **sem acusar** enquanto não
tiver estado dentro e voltado a sair, e **a partir da saída o estimador de penumbra volta a contar
do zero**.

⭐⭐⭐ **O `t` reconta-se porque `dureza·d/t` é o tamanho ANGULAR do obstáculo visto da origem do
raio** — com o `t` a incluir a corda andada lá dentro, um raio que sai e depois roça a própria peça
lê `d/t` minúsculo e **inventa** uma penumbra: o lado escuro lia `0,105` onde o lado iluminado, a um
pixel, lia `0,555`. Com a recontagem a rampa fica `0,555 · 0,400 · 0,582 · 0,781 · 1,000` — contínua.

⚠️ **`precisa_sair` vazio é a marcha de sempre, ao bit**, e é isso que deixa o ricochete, o cone da
oclusão e os testes intactos. O gémeo do dispositivo é o `visivel_saindo` do `trace_wgsl`, linha a
linha — e **as 6 provas de paridade CPU↔dispositivo continuam verdes**.

### §11.5 — Os portões e o preço

Três metades, e cada uma é um defeito medido:
[`a_sombra_de_um_vizinho_nao_e_truncada_no_terminador`] (barra `6,0`, do **vale** entre `3,71` e
`9,21`, **com o controlo liso dentro** — sem ele uma mutação que apagasse a sombra toda lia `1,00` e
passava) · [`a_folha_com_a_luz_atras_nao_se_apaga`] · [`um_corpo_convexo_nao_se_tapa_a_si_proprio`],
que é o piso que as outras duas não vêem (*uma «cura» que escurecesse a metade escura do mundo
passaria nas duas primeiras*). **4 de 4 mutações sangram.**

⏱️ **Preço:** o passe de sombra sobe **`+7 %` a `+9 %`** (debug, `load 1,1`: `1,91 → 2,08 ms` a
320 px, `72,4 → 79,2 ms` a 1920). ⚠️ O doc do `march_shadow_to` já media que filtrar os raios de
costas cortava **`55 %` da população e `6 %` do relógio** — *o filtro era uma optimização de `6 %`
paga com uma linha dura na tela*.

### §11.6 — ⏳ O que fica aberto

O p99 da banda é `3,71` contra `1,00` do controlo liso: sobra um **V** logo a seguir ao terminador
(`0,555 → 0,400 → 0,582`), porque um raio que sai muito perto da saída tem `t − base` curto e lê uma
penumbra mais dura. ⛔ A cura de fundo é a que a física manda e que nenhuma das duas referências
escreve: **a visibilidade que um termo TRANSMISSIVO lê é a do sítio por onde a luz ENTRA**, não a do
ponto sombreado — e o `thick` já supõe uma esfera local de raio `1/κ`, logo o ponto de entrada é
construtível com a curvatura que esta wave já calcula, **sem premissa nova**. Fica nomeado.
