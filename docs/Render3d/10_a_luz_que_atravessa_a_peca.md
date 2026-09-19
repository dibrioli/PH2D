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

## §11 — ⛔⛔⛔ O report de 2026-09-18: a LINHA DURA, e as DUAS curas que a foto refutou

> *«Bom resultado! Avalie apenas uma coisa: em `Thin Walled: Solid` não há transição suave entre a
> área iluminada e a área sombreada da esfera, mas uma linha dura. Veja se é correto.»*
> … e depois: *«não vi mudanças. O resultado em Thin Walled é melhor (mais suave a transição).»*

**Não é correto.** E ⚠️⚠️ **nada nesta secção foi achado por uma régua: foi achado por uma FOTO.**

### §11.1 — A resposta, em uma linha

⭐⭐⭐ **A linha que ele aponta é a borda da SOMBRA QUE A PLACA LANÇA SOBRE A BOLA.** Não é o
terminador, não é a lei da subsuperfície e não é a quadratura.

O experimento que o decide é de uma linha (`PH2D_TERM_SO_A_BOLA=1` na
[`sonda_fotografa_o_terminador`]): a MESMA câmera, a MESMA luz, o MESMO material, **sem a placa**.
| | o que se vê |
|---|---|
| a cena `=33` | a linha está lá |
| a bola sozinha | **a bola é perfeitamente lisa** |

⇒ ela é dura porque a luz é um **PONTO**, e um ponto lança sombra com borda em degrau. E ela só
incomoda no `Solid` porque ali a resposta tem contraste através da borda; na parede fina metade da
energia vem do lóbulo de trás e a mesma borda lê-se lavada.

⛔ **É correcto como geometria e ERRADO como produto:** num jade a luz que entra fora da sombra
espalha-se por baixo da superfície PARA DENTRO dela, e a borda amolece. A nossa subsuperfície
multiplica uma visibilidade **dura, por pixel** — *a difusão está na lei do `N·L` e não está na lei
da sombra.*

### §11.2 — ⛔ A fixtura mentiu QUATRO vezes antes de conter o fenómeno

1. Uma bola **sozinha na origem**: o perfil atravessou `N·L = 0` liso e o pior salto ficou no
   **brilho especular**, que lê o mesmo no opaco. *Uma régua que procura o máximo global mede o
   realce.*
2. `radiance_at_one: [intensity; 3]` em vez da porta do produto (`cor × intensidade × π`): a
   lâmpada saiu **`π×` fraca e sem cor** e o céu dominou. *Uma sonda que reescreve a conversão do
   produto mede outro programa.*
3. O enquadramento de omissão em vez do **zoom** que ele usou.
4. ⭐⭐⭐ E a que custou a jornada inteira: **nunca perguntei se a linha era a sombra da placa.**
   As duas curas foram desenhadas sem essa resposta.

### §11.3 — ⛔⛔ CURA A, construída inteira e REVERTIDA pela foto

O passe de sombra escreve `vis = 1,0` para todo ponto de costas para a luz, o que **trunca** no
terminador a sombra de um vizinho — defeito real, e o comentário desse filtro **previa-o por
escrito** (*«até ao dia em que alguém ler este canal para outra coisa — a OpenPBR tem termos que
recebem luz com `N·L < 0`»*). Construiu-se a cura: o raio parte na mesma e só conta o que estiver
**depois de sair do próprio corpo**, com o `t` do estimador de penumbra recontado a partir da saída.

Ela tinha tudo: gémeo em WGSL, **6 de 6** paridades CPU↔dispositivo verdes, **4 de 4** mutações a
sangrar, a folha intacta (`83,7`), a esfera convexa com zero acne, a banda do terminador de
`p99 9,21` para `3,71`.

⛔ **E a foto reprovou-a:** a borda alargou **e ganhou um FIO escuro SERRILHADO** por cima. O
serrilhado é a assinatura — **um `if` por pixel** (`N·L <= 0`) escolhia entre duas maneiras de
calcular a mesma grandeza, e *a fronteira entre elas desenha-se*.

⛔ **E apagar o ramo é PIOR:** um caminho só, todo raio a partir do ponto, devolve **acne** (riscos
claros ao longo do terminador; a banda vai a `p99 13,06`), porque o ergue pela normal deixa de lá
estar.

⇒ *a truncagem é INVISÍVEL nesta cena e o fio é VISÍVEL* ⇒ **shipa a truncagem**, e os dois gates
que a cura teria partido ficam, porque são onde a segunda tentativa vai bater:
[`a_folha_com_a_luz_atras_nao_se_apaga`] · [`um_corpo_convexo_nao_se_tapa_a_si_proprio`].

### §11.4 — ⭐⭐ CURA B, que FICA: a quadratura deixa de pôr vincos na lei

Medida a lei **sozinha** (sem cena, `4 001` amostras de `N·L`), o maciço tinha o pior salto da 2.ª
derivada em **`N·L = −0,0980` = `sin(π/32)`** — o `x` da 1.ª amostra da quadratura — e **não se
movia** com o `Subsurface Radius` (`0,1` · `1,0` · `4,0` dão todos o mesmo ponto). *Uma feição cuja
posição não depende de nenhum parâmetro físico é da discretização.*

A causa: o `max(cos(θ+x), 0)` amostrado no **MEIO** da célula põe uma quina em cada nó, e o perfil
de Burley é **singular em `x = 0`** ⇒ as duas células vizinhas do zero levam quase todo o peso.
⇒ o cosseno passa a ser integrado **exactamente dentro da célula** (`(sin b − sin a)/largura`, com
os extremos cortados ao domínio onde ele é positivo), que é **C¹ em θ** — nos cortes a derivada é
`cos(±π/2) = 0`.

| pior 2.ª derivada em `\|N·L\| <= 0,97` | |
|---|---|
| opaco (o terminador de Lambert, quina legítima) | `276` |
| parede fina — **o lado que o dono APROVOU** | `137` |
| maciço, ponto médio (o oráculo) | `67`–`95`, em `N·L = −0,098` |
| **maciço, hoje** | **`0,4`–`0,5`** |

⛔⛔ **Divergência DECLARADA:** a mediana contra o oráculo vai de `5,0e-4` para `2,9e-3`. ⚠️ **Toda
cura possível diverge daqui, e é aritmético:** o integral verdadeiro é um só, e é o ponto médio a
`N = 32` que está a `~3e-3` dele — somar mais amostras converge para o mesmo sítio. *O que diverge
do oráculo é ele próprio do seu limite.* As barras subiram com a tabela ao lado, e ⭐ a divergência é
**load-bearing**: quem a reverter para recuperar a mediana reprova no
[`o_macico_nao_e_mais_duro_que_o_lado_que_o_dono_aprovou`], cuja barra é calibrada **no lado que ele
aprovou**. **3 de 3** mutações sangram.

⚠️ **E ela NÃO apaga a linha da foto** — ela apaga um vinco *da lei*, que é outro defeito.

### §11.5 — ⏳ A cura que falta, agora com endereço

**A visibilidade que um termo TRANSMISSIVO lê tem de ser BORRADA pela distância de espalhamento.**
É isso que faz a sombra num jade ter a borda mole, e nenhuma das duas referências o escreve (o
MaterialX põe `occlusion = 1` e foge do assunto).

⭐ A maquinaria já existe nesta crate: o borrão com guarda de normal que o céu e o ricochete usam
(`OCCLUSION_BLUR_COS`, com os gates `a_suavizacao_apaga_o_ruido_dentro_de_uma_superficie` e
`a_suavizacao_nao_atravessa_uma_quina`). O que falta é um **segundo canal de visibilidade**, borrado
com o raio `mfp/κ` que o `integrate_burley` já usa, lido **só** pela closure de subsuperfície —
⚠️ e isso muda a fronteira do `ph2d-material` (hoje há **uma** radiância por lâmpada para todas as
closures), mais o gémeo em WGSL. **É wave própria, e fica nomeada.**

---

## §12 — ⭐⭐⭐ A SOMBRA COM A BORDA MOLE (ordem do dono, 2026-09-18: *«sim. faça»*)

A §11 fechou com o diagnóstico: a linha é a borda da sombra que a placa lança, dura porque a luz é
um ponto, e **certa como geometria e errada como produto**. Esta secção é a cura.

### §12.1 — A lei

> **A visibilidade que uma closure TRANSLÚCIDA lê é a MÉDIA da vizinhança, sobre a distância de
> espalhamento do material.**

Num jade a luz que entra **fora** da sombra espalha-se por baixo da superfície **para dentro** dela.
A nossa subsuperfície já tinha a difusão na lei do `N·L` (o `integrate_burley` envolve a luz à volta
do terminador) e **não a tinha na lei da SOMBRA** — a visibilidade entrava dura, por pixel.

⭐ **O comprimento da média é o `subsurface_radius` por canal** ([`Surface::scatter_distance`]), em
unidades do MUNDO, convertido a píxeis pela câmera ([`sss_shadow::raio_em_pixeis`]). *É por ser por
canal que a borda fica avermelhada — o vermelho viaja mais e entra mais fundo na sombra, que é a
assinatura de toda pele e de toda cera.*

⚠️ **O raio é do MUNDO e não do ecrã**: um raio escrito em píxeis seria uma borda que encolhe quando
o artista se aproxima.

### §12.2 — Onde ela entra, e porque NÃO precisou de partir o `compose`

⭐⭐⭐ A composição do OpenPBR é **linear na resposta das closures** — o `mix`, o `add` e o `layer`
(que multiplica pelo *throughput*, e esse não depende da radiância). ⇒ compor a superfície **com** e
**sem** o peso de subsuperfície e ficar com a diferença dá **exactamente** a parcela dela através de
toda a pilha. A porta é o [`Surface::direct_sss`], que recebe **duas** radiâncias.

⚠️ **Com as duas iguais ele é o [`Surface::direct`] AO BIT**, pelo braço curto — e é esse o caminho
de todo material sem subsuperfície, que não paga nada.

### §12.3 — A medição, e a FOTO

| | a borda | a quebra na banda `\|N·L\| <= 0,15` |
|---|---|---|
| antes | **uma linha** | p99 `9,21` |
| hoje | **mole, e a sombra continua lá** | p99 `1,36` |
| o mesmo material sem subsuperfície | dura (correcto) | **byte a byte o de sempre** |

⭐ E a foto da cena `=33` com o enquadramento do dono mostra a linha **desaparecida**, com a região
sombreada ainda visivelmente mais escura — *a sombra não foi apagada, foi amaciada*.

### §12.4 — Os portões, e as duas mutações que SOBREVIVERAM primeiro

Três metades, porque nenhuma chega sozinha
([`a_borda_da_sombra_num_jade_e_mole_e_a_do_opaco_continua_dura`]): **o jade amacia** · **o opaco não
muda UM BYTE** (sem isto, borrar a visibilidade de toda a gente passava e apagava a sombra do app
inteiro) · **o jade continua a TER sombra** (sem isto, `vis = 1` em todo o lado passava na primeira).

⛔⛔ **E duas mutações sobreviveram à primeira redacção, cada uma a nomear uma régua em falta:**
- *a separação deixa de ser exacta* — a cena de jade tem `subsurface_weight = 1`, logo o difuso já
  saiu da mistura e **trocar quem lê o quê no resto da pilha não movia um byte lá**. ⇒
  [`zerar_a_radiancia_da_subsuperficie_tira_so_a_parcela_dela`], com os dois lóbulos a valer.
- *a guarda de normal cai* — a bola é lisa, e **uma cena sem quina nenhuma não testa a guarda da
  quina**. ⇒ [`a_media_da_borda_mole_nao_atravessa_uma_quina`], sobre uma tira sintética.
  ⚠️ E ela sobreviveu **outra vez**: a 1.ª asserção media as PONTAS da tira, que a `r = 8` ficam fora
  do alcance da quina e leem o mesmo com a guarda apagada. *Uma régua colada ao fenómeno, não ao
  extremo.* **4 de 4 sangram** hoje.

### §12.5 — ⏳ O que FALTA, e é declarado

⛔⛔ **O DISPOSITIVO ainda não tem o gémeo.** Ele calcula a visibilidade **dentro** da pintura, por
pixel, logo dar-lhe a borda mole pede uma passagem que a escreva num buffer, duas de borrão separável
e a leitura — o mesmo desenho que o ricochete lá já tem. ⇒ **hoje a cura vê-se no caminho de
REFERÊNCIA** (`PH2D_FIELD_GPU=0`), e as paridades CPU↔dispositivo continuam verdes porque nenhuma
delas assa o canal.

⚠️⚠️ **E em 18/09 essa dívida declarada custou um report** — ver a **§21**: ela era uma NOTA, e uma
nota não se mede.

⏳ E fica também: a média é **separável** (duas passagens de uma dimensão) e a guarda de normal não é
separável em rigor — a divergência é declarada no cabeçalho do [`sss_shadow`], e vale o preço
(`O(r)` contra `O(r²)`; a `r = 24` isso são `2 401` toques por pixel e por canal).

---

## §13 — ⏳ O PADRÃO-OURO: a pergunta do dono, o que já está montado, e um DESVIO DE PROTOCOLO meu

> *«Não sei se temos o padrão ouro em qualidade. Temos a Unreal instalada aqui. Quer comparar e ver
> se podemos melhorar? Ou mesmo superar a unreal?»* — 2026-09-18.

### §13.1 — ⭐⭐⭐ A reformulação que muda o trabalho: a Unreal NÃO é a verdade

A subsuperfície em tempo real da Unreal é **também uma aproximação** (ecrã, perfil de Burley), com
artefactos próprios. ⇒ *comparar só com ela responde «somos como a indústria», nunca «estamos
certos».*

⭐ **O padrão-ouro é um TRAÇADO DE CAMINHOS CONVERGIDO**, e isso dá a *«superar a Unreal»* uma
definição medível e crispa:

> **superar = ficar mais perto da verdade do que a resposta de tempo real dela.**

⭐⭐ E ela própria traz um traçador de caminhos, logo os três podem correr a MESMA cena: nós · a
Unreal em tempo real · a verdade.

### §13.2 — O que está MEDIDO nesta máquina (2026-09-18)

| | |
|---|---|
| **Unreal Engine** | **5.8.2** instalada em `~/Documentos/Projetos/UnrealEngine`, `73 GB`, build promovida |
| licença | **EULA proprietária** ⇒ **PAREDE**: corre-se, nunca se lê o fonte |
| porta sem interface | ⭐ **`Engine/Binaries/Linux/UnrealEditor-Cmd` existe** |
| **Blender** | `5.2.2 LTS` (GPL ⇒ parede), `blender -b --factory-startup -P <script>` |
| placa | RTX 5060 Ti, 16 GB |
| ⭐ **a verdade, corrida** | um traçado convergido da cena `=33` a `2 048` amostras: **`7 s`** |

⭐⭐ **E o enquadramento BATE**: a bola sai no mesmo sítio, do mesmo tamanho, com o brilho no mesmo
canto — porque os números da câmera saem das **portas do produto**
([`sonda_os_numeros_da_cena`]) e não de um script que os re-deriva.

### §13.3 — ⛔ Os números ainda NÃO são veredito, e está declarado

O desvio de forma lê `96 %`–`98 %`, grande demais para uma comparação de forma ⇒ *as duas imagens
ainda não são comparáveis*. O que falta está nomeado: o **céu** do oráculo é cinzento uniforme e o
nosso é a `StudioSky`; as **unidades da lâmpada** não estão casadas (o ajuste de exposição foge
`+7` stops, que é a assinatura disso); a **janela** do perfil ainda apanha a silhueta.
*Publicar este número como resposta seria a quinta régua mal calibrada do dia.*

### §13.4 — ⭐⭐⭐ O achado que já saiu, e que explica o dia inteiro

A sombra da placa sobre a bola é uma **penumbra que nunca chega ao preto** (`vis` de `~0,52` a
`1,000`), e **a borda dela corre quase na HORIZONTAL**. ⇒ toda régua desta jornada que varria em
**linha** andava **paralela à feição** e lia outra coisa — a silhueta (`x = 317`), o realce
especular (`N·L = 0,95`), o vinco da quadratura. *Uma régua paralela à feição não a vê*, e o
instrumento passa a varrer em **coluna**.

### §13.5 — ⛔⛔ DESVIO DE PROTOCOLO (INC-R1), registado sem desculpa

O [`método §6`](../_ComoInvestigarApps/00_o_metodo.md) diz: *«o harness que corre o alvo vive FORA
da árvore. Ele é acto do **E**; o implementador pede uma emenda, nunca o corre.»*

**Eu fiz as duas coisas erradas:** escrevi o harness do Blender e **corri-o eu**, e cheguei a
copiá-lo para dentro do repo (retirado no mesmo commit).

⭐ **Contaminação realizada: ZERO** — o que foi lido foi a IMAGEM de uma cena nossa, que é livre
(GPLv2 §0), e nenhuma linha de fonte do alvo. Mas *o valor da parede é o protocolo, não a sorte de
desta vez não ter lido nada*. ⇒ **daqui para a frente as corridas do oráculo são pedidas a uma
janela E**, e o harness fica fora da árvore; o que entra no repo é a **fixtura com cabeçalho**.

### §13.6 — ⏳ O preço do que falta

| | |
|---|---|
| casar o céu e as unidades de luz, e fechar a janela do perfil | **meia jornada** — e é o que torna o número um veredito |
| a Unreal como terceiro contendor (projecto, compilação de shaders, cena casada, render sem interface) | **uma jornada** |

---

## §14 — ⭐⭐⭐ NÓS CONTRA A VERDADE: o primeiro veredito com número

O dono perguntou se temos o padrão-ouro. A §13 montou o oráculo; esta secção é a medição.

### §14.1 — ⛔⛔ Antes do número, DOIS defeitos MEUS que o agente E apanhou

O relatório dele mediu as duas convenções do PFM, e as duas partiam o meu leitor **em silêncio**:

1. o ficheiro é **BIG-endian** (escala `> 0`) e eu lia `from_le_bytes`;
2. o meu laço de cabeçalho parava aos **três** campos ⇒ **a linha da escala nunca era lida** e o
   offset dos dados ficava errado.

⇒ **o «desvio de forma de `96 %`» que esta sonda imprimiu era isso.** ⭐ *O leitor passou a
RECUSAR em voz alta* (magia, contagem de campos, tamanho que tem de fechar, valores finitos): um
leitor que devolve lixo plausível é pior que um que falha.

⚠️ E o E entregou o oráculo com **controlo interno**: cada média e cada máximo é **exactamente ×2 a
cada duplicação da energia**, nos dois modos ⇒ a saída é genuinamente linear, sem tonemap e sem ceifa.

### §14.2 — ⭐⭐ O CONTROLO valida a montagem, e é ele que dá direito ao resto

| opaco (sem subsuperfície) | NÓS | VERDADE |
|---|---|---|
| largura da transição, 10–90 % | **`43 px`** | **`43 px`** |
| cor da região iluminada, `R/B` | **`1,33`** | **`1,32`** |

⇒ *a cena, a câmera, a lâmpada, a sombra e a cor batem.* **Sem este controlo, nenhum número do
jade valeria nada** — seria mais uma régua mal calibrada.

### §14.3 — O veredito

| jade (`Subsurface Radius = 1,0 × (1 · 0,5 · 0,25)`) | NÓS | VERDADE |
|---|---|---|
| largura da transição, 10–90 % | `70 px` | `80 px` |
| **cor da região iluminada, `R/B`** | **`1,61`** | **`0,87`** |

⭐ **A LARGURA está perto** — `12,5 %` mais apertada que a verdade. A wave da §12 pôs a borda no
regime certo.

⛔⛔⛔ **A COR MOVE-SE NO SENTIDO OPOSTO, e é este o buraco para o padrão-ouro.** Partindo de
`R/B = 1,33` (a cor base), **nós vamos para `1,61`** (mais vermelho) e **a verdade vai para `0,87`**
(azulado). O mecanismo é lisível nas duas leis:

- a nossa (`mx_subsurface_bsdf`, o porte do MaterialX) lê `shape = 1/mfp` ⇒ **o canal de mfp maior
  tem o perfil mais LARGO** ⇒ o vermelho envolve mais ⇒ a peça avermelha;
- o traçado de caminhos transporta **fotões**: um mfp de `1,0` numa esfera de raio `0,42` quer dizer
  que **o vermelho ATRAVESSA e não volta** ⇒ o que regressa ao olho é azul.

⚠️⚠️ **Isto não é um defeito do nosso porte — é da APROXIMAÇÃO que a indústria publica.** Nós
reproduzimos o `mx_subsurface_bsdf` fielmente (a §11.4 mede a paridade contra ele). ⇒ *fechar este
buraco é SUPERAR a aproximação de referência, não alcançá-la* — que é exactamente a pergunta que o
dono fez.

### §14.4 — ⛔⛔ A VARREDURA CORRIGE A §14.3: não são «sentidos opostos» — é uma lei SURDA

> ⚠️⚠️ **NOTA de 2026-09-18 (§16.3): os números desta secção são de ESPAÇO DE ECRÃ, e não se
> comparam com os da §16.3, que são LINEARES.** A §16 mediu que `R/B` em bytes **não é invariante à
> exposição** — e que a régua de bytes chega a **inverter a ordem** de duas colunas que a linear põe
> ao contrário. ⭐ **O veredito desta secção sobrevive inteiro**, porque ele está ancorado na ÁLGEBRA
> da lei (com o `mfp` partilhado, o `integrate_burley` devolve o mesmo nos três canais) e não na
> régua; o que não sobrevive é comparar as MAGNITUDES daqui com as de lá.

⚠️ **A §14.3 leu UM ponto com máscaras diferentes dos dois lados e escreveu *«a cor move-se no
sentido oposto»*. Com três profundidades e a MESMA máscara, esse veredito está CORRIGIDO** — e o que
o substitui é pior para nós, não melhor.

`R/B` da região iluminada, os dois lados no **nosso** olhar, com a população da máscara casada:

| `Subsurface Radius` | família | **NÓS** | **VERDADE** |
|---:|---|---:|---:|
| `0,10` | por canal (`1 : 0,5 : 0,25`) | `1,42` | **`3,19`** |
| `0,30` | por canal | `1,73` | `1,55` |
| `1,00` | por canal | `1,61` | **`1,00`** |
| `0,10` | **IGUAIS** nos três | `1,41` | `1,64` |
| `0,30` | **IGUAIS** | `1,41` | `1,02` |
| `1,00` | **IGUAIS** | `1,41` | `1,00` |

⭐⭐⭐ **A verdade balança `3,2×` com a profundidade; nós balançamos `1,2×`** — e não é sequer
monótono. O botão que o artista tem quase não muda a cor no nosso, e muda-a enormemente na verdade.

⭐⭐⭐ **E o controlo dos RAIOS IGUAIS é o que fecha o diagnóstico: com ele nós lemos `1,41` nas TRÊS
profundidades — exactamente o mesmo número.** A nossa lei **não tem cor em função da profundidade**.
E o mecanismo não é uma medição feliz, é a ESTRUTURA da lei: o `thick` faz
`sss = subsurface_color × integrate_burley(…)`, e com os três canais a partilharem o `mfp` o
`integrate_burley` devolve **o mesmo valor nos três** ⇒ a cor que sai **é** o `subsurface_color`,
seja qual for a profundidade. *A nossa matiz é um multiplicador; a da verdade é transporte.*

⚠️ A verdade dessaturar com a profundidade (`1,64 → 1,00`) mesmo com raios iguais tem mecanismo: um
caminho livre médio da ordem da peça faz a luz **atravessar e não voltar**, logo o que chega ao olho
é dominado por caminhos curtos, menos filtrados pela cor.

### §14.5 — ⚠️ O piso de ruído do oráculo, medido pelo E

O Cycles em CPU **não é bit-reprodutível entre invocações** (a mesma cena, o mesmo `seed`, duas
corridas: `max|d| = 1,03e-04`). ⛔ **Nenhum gate contra estas fixturas pode afirmar abaixo de
`~1e-4` absoluto** (`≈0,2 %` da média) — abaixo disso mede-se o jitter do Cycles.
⭐ O efeito que medimos (`3,19` contra `1,42`) está **ordens de grandeza** acima desse piso.
⛔ E o `sha256` de um EXR **não serve** para comparar píxeis: o ficheiro embute `Date` e `RenderTime`.

### §14.6 — ⏳ O que falta para o veredito ficar fechado

✅ **O segundo ponto CHEGOU e está na §14.4** — ele transformou um ponto numa lei, e corrigiu a
leitura do primeiro.

⏳ **Fica a Unreal** como terceiro contendor: ela usa a mesma família de aproximação que nós, logo a
previsão é que **balance tão pouco quanto nós**. Se isso se confirmar, *«superar a Unreal»* passa a
ser: **ser o renderizador de tempo real cuja cor segue a profundidade.**

⚠️ E fica nomeado que o desvio de FORMA lê `25 %` **no próprio controlo opaco** — ele é o **piso do
método** (a janela apanha o realce ceifado e o ajuste de exposição é um compromisso), e não uma
propriedade do jade, cujo `30 %` está a cinco pontos desse piso.

---

## §15 — ⭐⭐⭐ A UNREAL COMO TERCEIRO CONTENDOR: a triagem, a parede, e o que ela cura

Ordem do dono, 2026-09-18: *«avance para a Unreal»*.

### §15.1 — A triagem (passo 1, sempre)

| | |
|---|---|
| artefacto | **Unreal Engine 5.8.2**, `Build.version` com `IsPromotedBuild: 1` · `IsLicenseeVersion: 0` |
| onde | `~/Documentos/Projetos/UnrealEngine/` — build binária oficial para Linux, **fora** da árvore do repo |
| licença | **EULA proprietária** ⇒ **PAREDE**: corre-se, nunca se lê |
| o que vem dentro | ⚠️ `Engine/Source/**` e `Engine/Shaders/**` — *o fonte está instalado, não é preciso ir buscá-lo* |
| porta sem interface | `Engine/Binaries/Linux/UnrealEditor-Cmd` ✅ |
| placa | RTX 5060 Ti, `VK_KHR_ray_tracing_pipeline` presente ⇒ **o path tracer dela corre** |

⭐⭐⭐ **E é isso que a torna o contendor mais valioso de todos: ela traz DOIS motores.**
O de **tempo real** é o nosso PAR (a mesma família de aproximação em espaço de ecrã), e o **path
tracer** é uma **segunda verdade independente** — que serve para confirmar ou desmentir o oráculo
Cycles da §14. *Um alvo que responde dos dois lados da mesma mesa vale mais que dois alvos.*

### §15.2 — ⛔⛔ O ACHADO QUE PAROU A JORNADA: a parede era uma promessa a dizer-se propriedade

O [`00_o_metodo`](../_ComoInvestigarApps/00_o_metodo.md) §0 afirmava, por escrito, que ninguém
*podia* ler o fonte de um alvo restrito — *«o `.claude/settings.local.json` nega os caminhos»*.

**Medido antes de abrir a janela do oráculo: não existia lista `deny` NENHUMA**, em ficheiro nenhum
(`.claude/settings.local.json` do repo · `~/.claude/settings.json` · `~/.claude/settings.local.json`).
⇒ *a família que este repo mais paga — um doc que declara a lei que o código não implementa lê-se
como auditado.* **Seis** afirmações em dois docs dependiam dela.

⭐ **A cura tem DUAS metades porque o defeito tem duas portas**, e uma sozinha é teatro:

| metade | cobre | onde |
|---|---|---|
| `permissions.deny` | a ferramenta `Read` | [`.claude/settings.json`](../../.claude/settings.json), **versionado** |
| regra **R3** | o `Bash` — por onde um `cat` passa **ao lado** do `deny` | [`tecto-de-recursos.sh`](../../.claude/hooks/tecto-de-recursos.sh) |

**Prova de mutação `3 de 3`**, com a metade positiva verde na mesma corrida (CORRER a Unreal ·
CORRER o Blender · ler o `config.ocio` do Blender · o nosso `oraculo_de_cor.py`) — *um guarda de
parede que bloqueasse correr o alvo teria matado o método em vez da fuga*.

⚠️ **A R3 recusa a MENÇÃO e não só a leitura, e é deliberado:** separar *«este caminho é um
operando»* de *«este caminho está dentro de um padrão de busca»* não se faz com um `grep` honesto —
o gate irmão `grep que MENCIONA` da prova existe porque a distinção é real. Aqui escolhe-se errar a
**FECHAR**: *um guarda de parede que erra a favor do alvo não é um guarda.* O custo é nomear o
caminho por outra ferramenta, e a recusa diz qual.

⛔ **O `datafiles/` do Blender fica FORA de propósito** — o `oraculo_de_cor.py` lê o `config.ocio`
(OpenColorIO, BSD-3), que é **dado** e não implementação: a mesma distinção que faz a SAÍDA de um
alvo ser livre.

### §15.3 — ⛔⛔⛔ E UM GUARDA ESCRITO NUMA WORKTREE NÃO GUARDA ESSA WORKTREE

Armada a R3, o guarda **não** recusou uma leitura de fonte da Unreal. O mecanismo, medido:

| | |
|---|---|
| o hook regista-se por | `${CLAUDE_PROJECT_DIR}/.claude/hooks/tecto-de-recursos.sh` |
| essa raiz resolve para | o **PRIMÁRIO**, nunca para a worktree da linha |
| os dois ficheiros | **inodes diferentes** (`37992926` · `42438351`) |
| a regra nova | `1` ocorrência na worktree · **`0` no primário** |

⚠️⚠️ **A assinatura é cruel: a regra R1 continua a recusar em voz alta** (o guarda ESTÁ activo — é
a cópia do primário que corre) ⇒ *a linha vê um guarda a funcionar e conclui que o dela está
armado.* ⛔ **Uma regra nova de parede só passa a guardar no dia da INTEGRAÇÃO**, e até lá a única
cerca é a disciplina de quem a escreveu.

⚠️ É a família da nota do `collision-surface.sh` (CLAUDE.md §1 — *«um script novo só existe nas
árvores que nasceram depois dele»*), **no sentido inverso e pior**: lá a ferramenta falta e falha
alto; aqui ela existe, corre, e mede **outra árvore**.

⛔ **A cura NÃO é escrever no primário** — isso é acto do integrador (§0.2/§0.7), e uma edição não
commitada noutra árvore é invisível a quem a for fundir. O que fica é o ficheiro versionado + esta
nota + a linha do handoff.

### §15.4 — ⛔ INC-R2, registado sem desculpa

Ao verificar a R3 eu corri, de propósito, o comando que ela devia recusar — e ele **não** foi
recusado (§15.3). Voltaram **três linhas** de um cabeçalho da Unreal: a linha de direitos de autor e
um `#pragma once`. **Contaminação realizada: zero de implementação** — mas o valor da parede é o
protocolo, não a sorte de desta vez não ter voltado nada.

⭐ **A lei que fica, e é a mesma do INC-R1 um nível abaixo:** *não se verifica um guarda de parede
tentando o acto proibido.* A verificação é a **prova de mutação sobre o guarda** (que existe, `3 de
3`) e a comparação do ficheiro que corre com o ficheiro que se editou — as duas sem tocar no alvo.

### §15.5 — ⏳ O que a janela E está a correr

Etapa 0 (reconhecimento) + etapa **1: o CONTROLO OPACO nos dois motores**, e ela **para aí**. A
barra é a da §14.2 — `43 px` de transição e `R/B 1,33` — e *sem ela nenhum número do jade vale*,
porque a conversão de mão (a Unreal é levógira, X-para-a-frente, em centímetros) produz uma imagem
espelhada que passa despercebida a olho. A etapa 2 são as seis células de jade × dois motores.

---

## §16 — ⭐⭐⭐ O VEREDITO A QUATRO COLUNAS: nós somos os ÚNICOS completamente surdos

### §16.1 — O portão, e a barra que reprovava o lado APROVADO

⛔⛔ **A 1.ª redacção do portão punha a barra em `0,05`, tirada do `1,32` que a §14.2 publica — e
reprovou o Cycles**, que é o lado aprovado. A causa é de método e vale para toda comparação futura:
aquele `1,32` foi medido por **outra régua** (a §14.2 casa a exposição pelo **perfil de bytes de uma
coluna**; a §14.4 e esta secção casam-na pela **população iluminada**), e ⚠️ **`R/B` em bytes NÃO é
invariante à exposição**. *Dois critérios de casamento produzem dois números que nunca mediram a
mesma coisa.*

⇒ a barra passa a ser **derivada na própria corrida**: o desvio que o lado aprovado produz, mais
meia folga. Sem o aprovado presente o portão **não arma** e sela o jade.

| controlo opaco, a MESMA régua nos quatro | `R/B` | Δ contra nós |
|---|---:|---:|
| **NÓS** | `1,332` | — |
| **VERDADE (Cycles)** — o aprovado | `1,165` | **`0,167`** ⇐ é esta a barra |
| UNREAL tempo real | `1,190` | `0,143` ✓ |
| UNREAL traçado | `1,190` | `0,142` ✓ |

⭐ **A montagem da Unreal está mais perto da nossa do que a do Cycles está** — e ela foi construída
por uma janela E que nunca viu a nossa, a partir dos números que as portas do produto imprimem.

### §16.2 — ⛔⛔ O TECTO do alvo, e porque a varredura publicada não servia

A janela mediu que o parâmetro de espalhamento da Unreal **satura**: acima do campo `≈ 60`–`80` a
imagem congela **ao bit** e os canais G e B **colapsam um sobre o outro**. A célula de `1,00 m` cai
lá dentro **nas duas escalas de unidade** ⇒ *um balanço que a inclua apoia-se num ponto que não é
uma medição.*

⇒ a varredura desce uma casa, para **`0,03 · 0,10 · 0,30`**, e ⚠️ **as QUATRO colunas descem juntas**
— descer só a do alvo seria comparar profundidades diferentes em colunas diferentes.

⭐ **E o mapeamento ficou ancorado, não escolhido:** com a escala de unidade de FÁBRICA, o ponto
raso do traçado lê `3,0963` contra `3,19` do Cycles **na mesma célula** (`3 %`). A previsão que o
fixou era falsificável e está registada: *«a curva desloca-se uma década»* — bateu em `0,1 %`,
`6,7 %` e `5,7 %` em três pontos, e falhou no quarto **porque o tecto não se desloca com a escala**,
que foi o que revelou o tecto.

### §16.3 — ⭐⭐⭐ O veredito, em LINEAR — e porque é ali que ele se lê

⛔⛔ **As duas réguas DISCORDAM sobre qual das duas verdades balança mais**, e isso é um defeito de
método que a §14.4 tem sem o saber: em bytes o traçado lê `1,27×` contra `1,92×` do Cycles, e em
linear lê `1,86×` contra `1,58×` — **a ordem inverte-se**.

⭐ **A causa é estrutural e a cura é escolher o espaço certo:** `R/B` em **linear é invariante à
exposição** (os dois canais escalam juntos), logo ali a pergunta da exposição **desaparece** — e é
só por medirmos em bytes que a maquinaria de casar exposição existe. Em bytes, a exposição é
ajustada **célula a célula**, e a curva de exibição entra na resposta junto com a distribuição de
brilho. ⇒ *para a pergunta «a matiz responde à profundidade?», o espaço é o LINEAR.*

⭐⭐ **E a nossa coluna em linear não se mede — ela sai da ÁLGEBRA:** o `thick` faz
`sss = subsurface_color × integrate_burley(…)`, e com o `mfp` partilhado pelos três canais o
`integrate_burley` devolve **o mesmo escalar nos três**. Com `subsurface_weight = 1` e o especular a
zero, todo pixel iluminado tem radiância `∝ (0,75 · 0,35 · 0,35)`:

> ⭐⭐⭐ **`R/B = 2,143` a QUALQUER profundidade — que é, ao bit, o `R/B` da bola OPACA.**
> *Com as três cores a viajar por igual, o nosso jade tem exactamente a cor de uma pedra opaca.*

| raios IGUAIS · linear · `0,03 · 0,10 · 0,30` | os três valores | **balanço** |
|---|---|---:|
| **NÓS** | `2,143 · 2,143 · 2,143` | **`1,00×`** (exacto) |
| UNREAL **tempo real** | `2,2687 · 2,1311 · 1,9351` | **`1,17×`** |
| UNREAL **traçado** | `2,5617 · 1,5865 · 1,3765` | **`1,86×`** |
| **VERDADE (Cycles)** | `2,2564 · 1,9130 · 1,4249` | **`1,58×`** |

⭐⭐⭐ **A leitura, e ela responde à pergunta do dono:**

1. **As duas VERDADES concordam** (`1,86×` e `1,58×`, a mesma classe) — e são motores independentes,
   de projectos independentes. ⇒ *o oráculo Cycles da §14 fica CONFIRMADO por um segundo traçado.*
2. **O tempo real da Unreal é quase surdo** (`1,17×`), que era a previsão do dono.
3. ⛔ **E nós somos os ÚNICOS completamente surdos** (`1,00×`, exacto e estrutural). A Unreal, no
   modo de jogo, ainda faz **parte** do caminho; nós não fazemos nenhum.

⚠️ ⇒ *«superar a Unreal»* deixa de ser ambição e passa a ter definição medível: **ser o motor de
tempo real cuja matiz segue a profundidade.** Ninguém lá está — e há duas verdades concordantes a
dizer para onde é.

### §16.4 — ⛔ O que fica ABERTO, nomeado

- **A nossa coluna em linear, na família POR CANAL**, não foi medida: o `quadro` desta sonda devolve
  **bytes** e a família de raios iguais dispensou-a por álgebra. ⇒ a sonda precisa de um quadro
  linear nosso para fechar a outra metade da tabela no espaço certo.
- ⚠️ **A §14.4 fica com os números em espaço de ECRÃ, e isso passa a estar escrito ali.** O veredito
  dela (*«a nossa cor não responde à profundidade»*) **sobrevive** — ele está ancorado na álgebra, não
  na régua —, mas as magnitudes das outras colunas são de bytes e não se comparam com as desta secção.
- **As duas famílias «por canal» têm o pico em profundidades diferentes** (a verdade em `0,10`, o
  traçado do alvo em `0,03`) — as curvas têm a mesma forma, deslocadas ~meia década. Resíduo do
  mapeamento ou lei: **não medido**.
- **O tecto do alvo** está cercado entre o campo `60` e `80`, com assinatura dupla (congela ao bit ·
  G e B colapsam). É informação sobre a ferramenta, e não bloqueia nada.

---

## §17 — ⭐⭐⭐ ATACAR A COR: o diagnóstico, e a nossa lei é o LIMITE RASO da verdade

Ordem do dono, 2026-09-18, depois do veredito da §16: *«atacamos agora a cor»*.
⛔ **Esta secção MEDE e não cura.** Uma lei escrita antes de as duas verdades concordarem sobre a
curva seria um ajuste a três pontos com cara de mecanismo.

### §17.1 — ⛔⛔ O mecanismo da nossa surdez está numa DIVISÃO

O `integrate_burley` devolve `Σ(R·w) / Σ R` — e o perfil `R` aparece **em cima e em baixo**. Tudo o
que ele sabe sobre a profundidade (a forma `1/mfp`, as duas exponenciais, a escala) **cancela-se na
divisão**, e o que sobra é uma média direccional pura. A cor sai depois por
`sss = subsurface_color × isso`.

⇒ *a informação da profundidade não se perde por aproximação — ela é **DIVIDIDA FORA por
construção**.* Com o `mfp` partilhado pelos três canais o quociente é o mesmo nos três, e a matiz é a
que o artista escreveu.

### §17.2 — ⭐⭐⭐ A surdez deixou de ser argumento e passou a ser uma CORRIDA

Medida pela **porta do material** (`Surface::direct` com luz branca ⇒ radiância linear, e o
quociente é a matiz da lei) — ⭐ *não precisa de um quadro linear, e por isso não herda a exposição*:

| `mfp` | `N·L = 0,9` | `0,4` | `0,0` | **`−0,3`** |
|---:|---:|---:|---:|---:|
| `0,03` | `2,14286` | `2,14286` | `2,14286` | `2,14286` |
| `0,10` | `2,14286` | `2,14286` | `2,14286` | `2,14286` |
| `0,30` | `2,14286` | `2,14286` | `2,14286` | `2,14286` |
| `1,00` | `2,14286` | `2,14286` | `2,14286` | `2,14286` |

**Dezasseis células, cinco casas decimais, um só número** — e `0,75/0,35 = 2,142857` é a cor
autorada **ao dígito**. ⚠️ Inclusive em `N·L = −0,3`, o lado **sombreado**, onde a luz só chega por
dentro da peça: *mesmo ali a matiz é a do painel.* Balanço `1,0000×`, `p = 1,0000`.

### §17.3 — A curva da VERDADE, e as duas verdades concordam onde ambas vivem

`R/B` **linear**, família de raios IGUAIS, **máscara geométrica fixa** (a silhueta iluminada do
controlo opaco). O expoente é `p` em `R/B = (0,75/0,35)^p`: `p = 1` é *«a cor autorada»*, `p = 0` é
*«branco»*.

| `mfp/raio` | CYCLES `R/B` | `p` | UNREAL-PT `R/B` | `p` |
|---:|---:|---:|---:|---:|
| `0,071` | `2,1092` | **`0,979`** | `2,1362` | **`0,996`** |
| `0,238` | `1,7007` | `0,697` | `1,7480` | `0,733` |
| `0,714` | `1,3467` | `0,391` | `1,9474` | `0,874` ⛔ |
| `2,381` | `1,1931` | `0,232` | `1,9474` | `0,874` ⛔ saturado |

⭐ **Nos dois pontos em que o alvo está vivo, as duas verdades concordam a `~5 %`** (`0,979` contra
`0,996`; `0,697` contra `0,733`) — dois motores independentes, de projectos independentes. ⛔ Nos
outros dois o alvo está dentro do tecto dele (§16.2) e a coluna não é uma medição.

### §17.4 — ⭐⭐⭐ O ACHADO: a nossa lei é o LIMITE RASO da verdade, truncado

**No extremo raso a verdade converge para NÓS** (`p → 0,979`, e a tendência é para `1`). ⇒ *a nossa
lei não está errada — ela está INCOMPLETA*: é a assímptota de `mfp ≪ peça`, publicada como se
valesse em todo o lado.

⭐⭐ **E o mecanismo é nomeável, com física e não com ajuste.** Num passeio aleatório cada evento de
espalhamento multiplica a luz pelo albedo do canal:

| regime | o que acontece | a matiz que volta |
|---|---|---|
| `mfp ≪ peça` (espesso) | **muitos** eventos antes de sair | a **reflectância difusa** — a cor autorada, `2,14` |
| `mfp ≳ peça` (fino) | **poucos** eventos; a luz atravessa e não volta | tende para o **albedo CRU**, muito mais perto de `1` |

⚠️ ⇒ o limite fino **não é branco** — é o quociente dos albedos, e ele é bem mais próximo de `1` que
o das reflectâncias porque a reflectância satura com o albedo. Os `1,19` medidos a `mfp/raio =
2,381` são consistentes com isso.

⇒ **a grandeza que falta à nossa lei é a ESPESSURA ÓPTICA da peça** (`2·raio / mfp`) — e ⭐ num campo
de distância ela **mede-se**, que é a vantagem estrutural que a §15 já nomeava como candidata a
superar. ⛔ Nenhuma das duas referências de tempo real a lê.

### §17.5 — ⏳ O que falta, e a ordem

1. ⏳ **Fechar a curva:** o quarto ponto do alvo é inalcançável, logo a curva é do Cycles, com o
   alvo a confirmar dois pontos. Mais profundidades **rasas** (onde os dois vivem) apertariam a
   concordância.
2. ⏳ **Derivar a forma fechada** do limite fino ao espesso — ⚠️ **derivar, não ajustar**: um ajuste
   de duas constantes a três pontos passa por mecanismo e não é.
3. ⏳ **Só então** o produto, atrás de porta, com estas oito células como corpus de gates.

### §17.6 — ⭐⭐⭐ O alvo APERTOU: a MAGNITUDE funciona, e só a MATIZ é surda

| ângulo | a magnitude sobre `33×` de profundidade |
|---|---:|
| `N·L = 0,9` (a pino) | `1,209×` |
| `N·L = 0,4` | **`1,031×`** |
| `N·L = 0,0` (o terminador) | **`1,757×`** |
| `N·L = −0,3` (o lado escuro) | **`3,503×`** |

⭐ **A lei responde com força exactamente onde tem trabalho.** ⇒ *não há nada a mudar na forma dela —
só na COR que a multiplica*, que é a cirurgia mais pequena possível para o defeito medido.

⛔⛔⛔ **E uma armadilha de RÉGUA paga neste mesmo passo:** a 1.ª redacção desta sonda imprimia a
magnitude **só em `N·L = 0,4`**, leu `1,03×` e eu quase publiquei *«o botão está quase morto»*. O
`0,4` é o **PIVÔ** da redistribuição — o único ângulo onde esta lei, por construção, quase não se
mexe —, e a razão de ela existir é o terminador, que era onde eu não estava a olhar.
*Uma régua que amostra um ângulo só mede o sítio onde o fenómeno não está.*

### §17.7 — ⛔⛔ E um SEGUNDO defeito, medido de caminho: `Subsurface Radius < 0,1` é INERTE

`mfp = 0,03` e `mfp = 0,10` devolvem valores **idênticos nos quatro ângulos**. A causa é o
`max(mfp, 0.1)` do `mx_integrate_burley_diffusion`, que é um piso em **unidades ABSOLUTAS de mundo**
— e a peça desta cena tem raio `0,42`. ⇒ *um quarto do raio da peça é o chão do botão do artista, e
abaixo dele o botão não faz nada.*

⚠️ **O recurso que aquele piso guarda é NUMÉRICO** (o perfil diverge em `mfp → 0`), não físico — e
§0.0: *um limite legítimo diz de que recurso ele é*. Um guarda numérico deveria ser **relativo à
peça**, não absoluto. ⛔ Mudá-lo é **divergência declarada** da referência e move os gates da §4.1:
fica **nomeado e não curado** nesta wave.

---

## §18 — ⭐⭐⭐ A CURA: a matiz passa a seguir a profundidade, e o neutro é BYTE-IDÊNTICO

[`ph2d_material::subsurface::cor_na_profundidade`], atrás de
[`OpenPbr::subsurface_depth_hue`] — **nasce em `0`**.

### §18.1 — A lei, e ela é a cirurgia mais pequena possível

A §17.6 mediu que **a magnitude já funciona** (`1,76×` no terminador, `3,50×` do lado escuro) e que
**só a matiz é surda**. ⇒ *não se toca na forma da lei — só na COR que a multiplica*:

```text
C_k = A_k · (α_k / A_k)^((1 − f) · peso)        f = 1 / (1 + (mfp_k·κ / X0)^N)
```

| peça | `f` | a matiz | de onde vem |
|---|---|---|---|
| espessa (`mfp·κ → 0`) | `1` | a **reflectância autorada** | **DERIVADO** — é a lei de hoje |
| fina (`mfp·κ → ∞`) | `0` | o **albedo cru** | **DERIVADO** — inversão publicada de Christensen & Burley |
| a transição | — | — | `X0 = 0,306` · `N = 2,25`, **calibrados** no oráculo |

⭐⭐ **Os dois extremos são derivados e só a transição é calibrada** — e há gate a afirmá-lo: se
alguém trocar a inversão publicada, o `os_dois_extremos_da_matiz_sao_derivados_e_nao_calibrados`
reprova.

⭐ **E ela não precisa de entrada nova:** `mfp·κ` é exactamente o adimensional que a §2.1 já tinha
medido como a **única** grandeza de que esta lei depende.

### §18.2 — ⭐⭐⭐ O que ela compra

| `mfp/raio` | VERDADE | NÓS hoje | erro | NÓS com a cura | erro |
|---:|---:|---:|---:|---:|---:|
| `0,0714` | `2,1092` | `2,1429` | `+1,6 %` | `2,0992` | **`−0,5 %`** |
| `0,2381` | `1,7007` | `2,1429` | `+26,0 %` | `1,7467` | **`+2,7 %`** |
| `0,7143` | `1,3467` | `2,1429` | `+59,1 %` | `1,3114` | **`−2,6 %`** |
| `2,3810` | `1,1931` | `2,1429` | `+79,6 %` | `1,2259` | **`+2,8 %`** |

⇒ **pior erro de matiz `79,6 % → 2,8 %`, `28,9×` melhor.**

### §18.3 — ⚠️ O NEUTRO é byte-idêntico por ÁLGEBRA, não por uma cerca

A lei escreve-se `A · (α/A)^((1−f)·peso)`. Com `peso = 0` o expoente é **exactamente** `0`, e `x^0` é
`1` ao bit para todo `x` finito; a renormalização de luminância divide `lum(cor)` por si próprio, e
`y/y` é `1,0` exacto em IEEE-754.

⭐⭐ **E isso está PROVADO por mutação**: apagar o `if peso <= 0` deixa os quatro gates **VERDES** ⇒
*aquele `if` é um atalho de desempenho e não uma cerca de correcção*. As outras **seis** mutações
sangram (expoente invertido · a lei como no-op · a renormalização fora · `X0` · `N` · a inversão do
albedo).

⇒ as quatro paridades contra o renderizador de referência (§4.1) correm **verdes e intactas**, e o
produto de hoje não muda um bit.

### §18.4 — ⚠️ Ela preserva a LUMINÂNCIA, e isso é uma decisão MEDIDA

`α ≥ A` sempre (o espalhamento múltiplo perde energia), logo a correcção crua **clarearia** a peça
inteira. ⛔ O defeito medido é **só de matiz** ⇒ curar o que não está partido seria trocar um defeito
medido por um não medido. A correcção é renormalizada, e ⭐ **reescalar por um escalar não muda
`R/B`** — a curva que a calibração mediu fica intacta.

### §18.5 — ⛔ O que fica por fazer, e porquê nesta ordem

| | |
|---|---|
| ⏳ **a decisão do dono** | ligá-la muda **toda peça translúcida de toda cena** e move a paridade da §4.1 — *superar a referência e alcançá-la são duas coisas, e só uma se liga sem ele saber* |
| ⏳ o campo na cena + o painel | custa um degrau de `PROJECT_SCHEMA`; ⭐ o `OpenPbr` **não é serializado**, e é por isso que a lei nasceu onde nasceu — **zero** contadores partilhados nesta wave |
| ⏳ o gémeo em **WGSL** | a paridade de `100,000 %` exige-o; ⛔ mas construí-lo para uma lei ainda **não aprovada** seria a ordem errada, e com o botão a `0` o dispositivo e a CPU concordam por construção |
| ⏳ o piso `max(mfp, 0.1)` | a §17.7 mediu-o inerte abaixo de um quarto do raio da peça — **nomeado e não curado** |

---

## §19 — ⭐⭐⭐ «SS Anisotropy está morto?» — não: são CINCO, e é uma PARTIÇÃO

Pergunta do dono, 2026-09-18, depois de aprovar o smoke da `=34`.

⛔ **Responder a UM botão por leitura de código deixaria os outros trinta e dois por perguntar** — e
a caça de 2026-08-30 achou `34` controlos mortos sobre ~`504`. ⇒ a resposta é um **censo**
([`censo_dos_knobs_do_material_tests`](../../crates/ph2d-app-field3d/src/censo_dos_knobs_do_material_tests.rs)):
para cada campo do material, o mesmo material com **duas** posições do botão, comparado **ao bit**,
nos **dois** caminhos.

### §19.1 — A tabela

| botão | `Solid` | `Thin Walled` | porquê |
|---|---|---|---|
| **`subsurface_anisotropy`** | ⛔ **morto** | vivo | a fase só entra nos dois factores da parede fina |
| **`subsurface_radius`** | vivo | ⛔ **morto** | a parede fina **não sabe nada sobre a forma da peça** |
| **`subsurface_scale_r/g/b`** | vivo | ⛔ **morto** | idem — sem profundidade, não há distância por canal |
| base · especular · verniz (`0`–`18`) | vivo | vivo | |
| `subsurface_weight` · a cor (`23`–`26`) | vivo | vivo | |
| emissão (`19`–`22`) | — | — | ⚠️ **fora do alcance deste censo** (ele mede a luz de uma LÂMPADA; a emissão sai por porta própria) |

⭐ **As duas metades são o PORTE FIEL:** a parede fina é a lambertiana do lado de lá — ela não tem
profundidade, logo um caminho livre médio não lhe diz nada; e a maciça integra um perfil
**isotrópico**, logo uma fase não lhe diz nada. ⇒ o gate **FIXA** a partição em vez de a curar.

### §19.2 — ⚠️⚠️ Mas as cinco SÃO controlos mortos, e a cura é de PRODUTO

*«o painel escreve onde · quem lê · o leitor DECIDE, ou entrega a alguém que descarta?»*
(`CLAUDE.md` §5.0). **O painel mostra a UNIÃO e a lei lê uma PARTIÇÃO** ⇒ em qualquer dos dois modos
há fileiras que o barro não sente. É a mesma forma do `Strength` do `Density` na família do
esculpir, e a cura é a mesma: **esconder**, ou **pintar desactivado com a razão à vista**.

✅ **DECIDIDO pelo dono em 2026-09-18: *«deixá-las à vista, apagadas»*** — a segunda saída, com a
razão ao lado. Ver a **§20**.

### §19.3 — ⭐ O que o censo tem para não mentir

| cerca | o defeito que ela impede |
|---|---|
| material de base com **tudo armado** | *um botão cujo dono está desligado mede-se morto sem o ser* — a forma mais barata de fabricar dívida |
| `subsurface_weight` a **meio** e não a `1` | com `1` a difusa desaparece e metade dos botões da base lê-se morta |
| varrer `0,25`/`0,75` e não `0`/`1` | *um par degenerado mede a AUSÊNCIA do ramo em vez do botão* |
| **piso** (`vivos·2 > total`) | um arnês partido faria a tabela inteira ler `MORTO`, e isso leria-se como um achado enorme |
| **controlo positivo nomeado** | a cor da base tem de acusar nos dois caminhos |
| o que fica **de fora**, nomeado | a emissão e o céu saem por portas próprias — *um morto nesta tabela pode viver numa delas* |
| os índices com o **nome conferido** | se a tabela do documento se mexer, o gate reprova a dizer isso, em vez de medir outro campo |


## §20 — ✅ AS FILEIRAS APAGADAS DIZEM PORQUÊ — e não eram cinco, eram SEIS FAMÍLIAS

### §20.1 — ⛔⛔ O mecanismo já existia; o que faltava era a RAZÃO

Uma fileira que o modo em mãos não lê **já** era pintada como facto, sem slider, sem campo e sem
entrada no índice de acerto — é a [`ph2d_field::Span::Locked`], que existe desde 14/09 por ordem do
dono (*«não devem desaparecer, mas apenas serem inativados, mas sempre visíveis»*).

⚠️⚠️ **E ela era MUDA — em todas as famílias.** O censo do travamento:

| posições | inertes quando | a razão que o artista passa a ler |
|---|---|---|
| `4`, `11` | `metalness == 1` | *um metal cheio não tem camada difusa* |
| `13`–`18` | `coat == 0` | *o verniz está desligado* |
| `20`–`22` | `emission == 0` | *o objecto não emite luz* |
| `24`–`32` | `subsurface_weight == 0` | *a subsuperfície está desligada* |
| **`27`–`30`** | **`thin_walled == 1`** | *uma parede fina não tem profundidade* |
| **`31`** | **`thin_walled == 0`** | *um sólido espalha por igual em todas as direcções* |
| o 3.º ângulo | trava de cardan | *mexa no ângulo do meio para separar os eixos* |

⭐ *Um controlo travado e sem razão à vista lê-se exactamente como um controlo morto* — é a conclusão
que o dono já tirou três vezes noutra família (*«não vejo efeito com density»*). ⇒ **curar só as duas
famílias que ele perguntou deixaria as outras cinco a mentir do mesmo modo.**

### §20.2 — ⛔ Porque a razão viaja DENTRO da faixa, e não num campo ao lado

A [`Span::Locked`] passou a ser `Locked(&'static str)` — a **chave i18n** da razão — e a
[`ParamRow::live: bool`] passou a ser `inert: Option<&'static str>`.

⛔ **Um `live: bool` com um `reason` ao lado seriam duas respostas à mesma pergunta**, e a combinação
`apagado sem razão` — que é precisamente o defeito de hoje — continuaria exprimível. Num `Option`
ela **não é**: *não há como travar uma fileira sem dizer porquê.*

⚠️ **Uma CHAVE e não um rótulo** (HR-15), e é a convenção que a [`Span::Choice`] vizinha já usa: este
documento **já** carrega o vocabulário dos params. ⛔ É o oposto do `ph2d_sculpt3d::CurvaInerte`,
onde o MOTOR devolve um **enum** por não saber o vocabulário da interface — *a fronteira é de quem
carrega os nomes, e este carrega*.

### §20.3 — ⭐⭐ A razão é dita UMA VEZ por corrida

Com `Thin Walled` ligado são **duas** fileiras seguidas com a mesma razão (o *Subsurface Radius* e a
amostra do *Radius Scale*); com a subsuperfície desligada são **nove posições**. Escrever a frase por
baixo de cada uma é o defeito que a família do esculpir já nomeou por escrito: *um pincel que se
queixa sempre é ruído que o artista aprende a ignorar, exactamente quando a queixa passar a ser
verdade.*

⭐ A corrida é **derivada** (`razao_a_pintar`) e quebra sozinha quando a razão muda, quando aparece
uma fileira viva, e quando o orquestrador pinta um cabeçalho de secção — *nesse caso ele separa as
duas à vista, e a razão de cima deixa de estar ao lado da de baixo*.

### §20.4 — ⭐⭐⭐ O gate que torna isto honesto: o painel apaga o que a MEDIÇÃO diz

A cura vive numa lista de índices e a verdade vive na `ph2d_material::Surface::direct`. *Uma segunda
lista escrita à mão ao lado de uma medição é a que envelhece*, e o modo de falha é mudo **nos dois
sentidos**: uma fileira apagada que o barro SENTE é um controlo roubado, e uma viva que ele não sente
é o defeito que o dono encontrou.

⇒ `o_painel_apaga_exactamente_o_que_a_medicao_diz_estar_morto` compara os dois conjuntos com um
`assert_eq!` que falha nos dois sentidos, **sobre o mesmo material**, escrito no documento posição a
posição pela porta do produto.

⛔⛔ **E a 1.ª redacção dele reprovou sobre produto CORRECTO**, o que mudou a régua: ela comparava
*fileiras* e leu `[27, 28]` contra `[27, 28, 29, 30]` — porque o `subsurface_radius_scale` **é uma
cor** (âncora em `28`), e os canais seguidores não têm fileira própria: eles viajam na amostra.
⇒ a pergunta não é *«esta fileira está apagada?»* mas ***«o artista consegue mexer neste número?»***,
e cada fileira **cobre** os campos dela. ⭐ A metade estrutural veio com ela: *todo campo da família
é alcançável* — um campo sem fileira e sem amostra não está nem vivo nem apagado, está **AUSENTE**,
que é a saída que o dono recusou.

| gate | o que ele afirma |
|---|---|
| `o_painel_apaga_exactamente_o_que_a_medicao_diz_estar_morto` | o conjunto apagado **é** o medido ao bit, nos dois caminhos · todo campo é alcançável · cada caminho apaga alguma coisa |
| `nenhuma_fileira_apagada_fica_muda` | não existe travamento mudo, e a razão chega **traduzida** (uma chave que o `ph2d-i18n` não conheça sai pintada em cru) |
| `a_razao_de_uma_fileira_apagada_chega_a_pixel` | a frase entra na cena — medido em **glifos**, porque o Vello encaminha texto por `draw_glyphs` e *nenhum glifo entra na contagem de caminhos* |
| `a_mesma_razao_seguida_e_dita_uma_vez_so` | a corrida colapsa, e **só** quando as razões são iguais |
| `cada_caminho_tranca_a_metade_que_nao_le` | a partição, com o controlo dos que vivem nos dois |
| `com_a_subsuperficie_desligada_a_razao_e_essa_e_nao_a_do_caminho` | **o bloqueio mais externo fala primeiro** |
| `toda_razao_e_uma_chave_do_vocabulario_desta_familia` | nenhum braço tranca com uma chave de fora, com piso de população |

⚠️ **A ordem dos braços é a LEI, e não arrumação:** com o peso a zero **e** a parede fina ligada, as
posições `27`–`30` têm duas razões verdadeiras — e dizer *«uma parede fina não tem profundidade»* a
quem também tem a subsuperfície desligada é mandá-lo resolver a metade que **não** o destranca. É a
mesma lei da `recusa::Entradas::recusa` na família do esculpir.

**9 mutações, 9 sangram.**

### §20.5 — ⚠️ E o tecto de LOC estava VERMELHO desde antes desta wave

O `subsuperficie_terminador_tests.rs` estava a **`1 455`** linhas contra o tecto de `700` — no `HEAD`,
**antes** desta wave. ⚠️ É a sexta ocorrência da cegueira que o `CLAUDE.md` §5.0 nomeia: *aqueles
gates vivem em `ph2d-editor-core/tests/it/` e um portão que só corre as crates editadas não os vê.*

⛔ Curado por **CORTE POR RESPONSABILIDADE**, nunca por uma entrada no `FILE_OVERAGE_OK`:

| ficheiro | o que ele responde | LOC |
|---|---|---|
| `subsuperficie_terminador_tests.rs` | a moldura (`quadro`) e as **três leis** que reprovam | `430` |
| `subsuperficie_sondas_tests.rs` | as quatro **sondas** que decompõem o report (todas `#[ignore]`) | `594` |
| `subsuperficie_oraculo_tests.rs` | a **bancada do oráculo externo** (PFM, exposição casada, `R/B`) | `502` |

⭐ *Um ficheiro em que uma sonda e uma lei se leem iguais é onde uma lei passa a `#[ignore]` sem
ninguém dar por isso* — e a bancada do oráculo é outra pergunta: *«comparada com quê, medida como?»*.


## §21 — ⛔⛔ «Por que a linha dura voltou em Solid? A luz está diferente?»

Report do dono, 2026-09-18, com a foto do ecrã dele: a bola de jade em `Thin Walled: Solid`, **com a
linha dura de volta** — e, no mesmo dia, o achado dele: ***«descobri que a presença da placa faz a
linha dura aparecer»***.

### §21.1 — ⭐ Ele tem razão, e confirmou a §11 por si

Sem a lâmina na cena, no mesmo enquadramento e com a mesma luz, a bola sai **lisa** ⇒ a linha é a
borda da **sombra que a placa lança**. É exactamente o que a §11 mediu com `PH2D_TERM_SO_A_BOLA=1`.

### §21.2 — ⭐⭐⭐ A luz NÃO está diferente: MEDIDO, na banda do terminador

| caminho | quebra `p99` | contraste através da banda |
|---|---|---|
| **DISPOSITIVO** (o que ele fotografou) | **`9,21`** | `61,2` |
| **REFERÊNCIA** (`PH2D_FIELD_GPU=0`) | **`1,00`** | `61,1` |

⭐⭐ **O contraste é o mesmo e a quebra é `9,2×`** — *a sombra está lá, com a mesma força, nos dois; o
que muda é só a BORDA dela.* ⇒ a resposta à pergunta é **não**: a luz é a mesma, e o que falta no
ecrã dele é a passagem da §12.

### §21.3 — ⛔⛔ A causa é a dívida da §12, e ela era uma NOTA em vez de um número

A cura da §12 foi assada no traçado de **CPU** e o **dispositivo não tem o gémeo**. A §12 declarou-o
por escrito — e **nada media a diferença**: as paridades CPU↔dispositivo ficam verdes porque
*nenhuma delas assa este canal*. Elas comparam as metades em que os dois concordam, e a metade em
que discordam não entra em nenhuma.

⚠️⚠️ **E o roteiro de smoke que eu escrevi não trazia o interruptor** (`PH2D_FIELD_GPU` ausente ⇒ o
dispositivo). *Um roteiro que manda olhar para onde a cura não corre ensina que ela não existe* — a
espécie que o `CLAUDE.md` §5.0 chama de pior que uma cena ausente, e desta vez fui eu que a escrevi.

### §21.4 — ⭐ A dívida passa a ter NÚMERO e data de fim

`Quadro::mole` desenha o quadro **como o dispositivo o desenha**, e dois gates leem-no:

| gate | o que ele afirma |
|---|---|
| `o_dispositivo_ainda_desenha_a_borda_dura_e_a_referencia_nao` | a diferença **existe** (`≥ 3×`) · o **contraste não muda** (a luz é a mesma) · os dois **continuam a ter sombra** |
| `a_regua_da_banda_le_quase_zero_num_gradiente_sem_degrau` | a régua separa um **degrau** de uma **rampa** — ver abaixo |

⭐⭐ **O primeiro REPROVA no dia em que o gémeo chegar, e isso é o desenho:** quem o curar tem de
apagar a dívida da §12 e o gate com ela. *Uma diferença declarada e não medida é uma nota que
envelhece; uma com gate é uma propriedade com data de fim.*

### §21.5 — ⚠️ E uma mutação SOBREVIVEU a DOIS gates, com a cura no experimento do DONO

Trocar a **segunda** diferença (`a − 2b + c`) pela **primeira** (`a − c`) na régua da banda deixava
verdes o gate da borda mole (barra ABSOLUTA) **e** o dos dois caminhos (uma RAZÃO — invariante ao
operador **por construção**, já que compara dois renders com a mesma régua).

⛔ E a troca **não é inofensiva**: um terminador tem um gradiente legítimo, e uma primeira diferença
acusa-o inteiro — os números desta página passariam a medir a **inclinação** da banda em vez do
**degrau** nela.

⭐⭐⭐ **O discriminador é o experimento do dono, virado do avesso:** na bola **sem a placa** a banda é
um gradiente puro, e ali a segunda diferença lê `~0` enquanto a primeira lê a inclinação toda.
*Uma régua de degrau que acusa uma rampa não é uma régua de degrau* — e a `so_a_bola` deixou de ser
uma variável de ambiente numa sonda para ser uma **porta**: o experimento que decidiu o diagnóstico
é o que um gate tem de poder repetir.

**4 mutações, 4 sangram.**

## §22 — ⛔⛔ «A chapa ainda influencia o SSS da esfera. Por que isso? Não faz sentido.»

Report do dono, 2026-09-18, a seguir ao da linha dura. **São TRÊS caminhos, e só um é o que ele vê.**

| caminho | legítimo? | medido |
|---|---|---|
| **(A) a SOMBRA** que a chapa lança | ⭐ **sim** | é o que se vê |
| **(B) o PASSO DA CURVATURA** sai da bola do DOCUMENTO | ⛔ não | `2,54×` no `ε` · **`1` byte** na imagem |
| **(C) o RAIO DO BORRÃO** é o MÁXIMO da cena | ⛔ não | `18×` no raio · **dormente** na `=33` |

### §22.1 — ⭐⭐⭐ (A) faz sentido, e é a resposta à pergunta

O termo de subsuperfície é *a luz que entrou PERTO e saiu aqui*. Se a chapa impede a luz de entrar
perto, **sai menos** — logo uma sombra sobre uma peça translúcida **tem** de a escurecer. ⚠️ É
exactamente por isso que a cura da §12 **borra** a visibilidade que aquela closure lê, em vez de a
remover: tirá-la seria fazer a peça ignorar a sombra, que é o defeito oposto.

⭐ **E a medição isola-o:** com a sombra **desligada**, a chapa move a esfera **`1` byte de `255`**
sobre `18 315` píxeis. *Tudo o resto que a chapa faz à esfera é invisível.*

### §22.2 — ⛔ (B) O passo da curvatura de uma peça sai da bola do DOCUMENTO

A [`eps_para`] diz de si mesma, por escrito, que `escala` é *«o tamanho da PEÇA (o raio da bola que
a envolve)»* — e quem a chama passa `bounding_ball(doc)`, que é a **cena**. ⇒ pôr uma chapa ao lado
leva o `ε` da esfera de `0,00269` a `0,00683` (**`2,54×`**), e a curvatura é o que o caminho maciço
da subsuperfície lê.

⛔ **Não curado**, e o motivo é o preço: um `ε` por peça muda toda imagem que já ship e move a
paridade com o dispositivo, que deriva o dele da mesma bola. ⇒ **decisão do dono.**

⚠️⚠️ **E o tecto do gate (`1` byte) é sobre ESTA FIXTURA, não sobre o defeito** — medido: tornar o
`eps_para` quadrático na escala **não** move aquele byte, porque a curvatura de uma **esfera** é
robusta ao passo. *Numa peça com detalhe fino um `ε` `2,54×` mais grosso apagaria feição.*

### §22.3 — ✅ (C) O raio do borrão era o MÁXIMO da cena — **CURADO em 19/09** (ver §23)

O código declarava que era *«uma diferença que só se vê onde as duas peças se tocam»*. **Não é** — o
raio é a **largura** com que toda borda de sombra da imagem é amaciada, e a sombra que uma peça
lança **sobre outra** é precisamente onde o raio da primeira aparece. ⇒ *uma divergência declarada
com o mecanismo errado é pior que uma não declarada: ela convence quem a lê a não a medir.*

⭐ **Medido:** uma esfera de raio `0,05` ao lado de uma chapa de `0,90` desenha a borda dela com
`0,90` — **`18×`**. ⭐⭐ **E ele DORME na `=33` porque a lâmina está OPACA:** o que a mantém fora do
máximo é o **PESO** da subsuperfície, não o raio dela (ela tem um número no slider).

### §22.4 — ⭐⭐⭐ «SSRadius tira a linha dura» — e o botão faz o CONTRÁRIO no dispositivo

| `Subsurface Radius` | DISPOSITIVO | REFERÊNCIA |
|---|---|---|
| `0,05` | `6,14` | `1,43` |
| `0,30` | `7,35` | `1,14` |
| `0,76` | `8,92` | `1,00` |
| `1,00` | **`9,21`** | **`1,00`** |

⛔⛔ **No dispositivo, SUBIR o raio piora a linha (`+50 %`)** — lá não há borrão nenhum, e o botão só
muda o **perfil** de Burley: ele escala o degrau em vez de o alisar. *Baixá-lo reduz a quebra um
terço, que é provavelmente o que o dono viu — e não a cura.*

⭐ **Na referência a linha já não existe em NENHUMA posição do botão** (`1,00` chapado): ali a cura
não depende do knob, que é como uma lei deve ser.

### §22.5 — ⛔ E a sonda pisou um ESTOURO que ninguém procurava

`blur_por_canal` percorre o **gbuffer** enquanto indexa o **canal**: com uma lâmpada que o passe não
escreveu, `index out of bounds: the len is 0 but the index is 9983`. ⭐ Hoje um canal que não cobre o
gbuffer devolve **vazio**, e o `soft_at` cai na visibilidade **dura** — que é exactamente *«esta
lâmpada não tem borda mole»*.

⚠️ *Um porte que estoira sobre uma entrada vazia é uma armadilha para o SEGUNDO chamador* — o
primeiro só não a pisou porque percorre as lâmpadas que existem.

### §22.6 — ⚠️ E DUAS mutações sobreviveram, cada uma a nomear uma fixtura vazia

- **A vizinha «opaca» tinha raio ZERO** ⇒ tirar o filtro de `reads_curvature` era invisível. *Uma
  fixtura cujo valor «mau» é zero não testa o filtro que o deita fora.* ⭐ E ao curá-la apareceu o
  que aquele filtro compra **sozinho**: não é o número (a `scatter_distance` tem a mesma guarda lá
  dentro) — é o **`Some` contra o `None`**, que é o que faz um quadro sem subsuperfície não pagar
  nada.
- **O tecto de (B) não tinha a metade da ENTRADA** ⇒ um `eps_para` que ignorasse a escala passaria
  trivialmente, com o gate a afirmar que não há vazamento nenhum.

**5 mutações, 5 sangram.**

## §23 — ✅ O RAIO DO BORRÃO É DE CADA MATERIAL (ordem do dono, 2026-09-19: *«cure»*)

### §23.1 — ⭐⭐ A lei, e porque o preço é por VALOR DISTINTO e não por peça

O borrão é uma passagem sobre a imagem inteira ⇒ o custo é o número de passagens. ⚠️ Mas dois
materiais com o **mesmo** espalhamento pedem a **mesma** passagem ⇒ o preço é o número de **valores
distintos**, que numa cena real é `1` ou `2` e não o número de peças
([`espalhamentos_distintos`], uma porta com **dois** leitores: o borrão e o gate do preço).

⭐⭐⭐ **E com UM valor distinto a saída é BYTE-IDÊNTICA à que ship, sem sequer perguntar a quem é
cada pixel** — *paga-se a correcção exactamente quando se usa a capacidade*, e uma cena de um
material só, que é toda cena de hoje, não paga nada.

⚠️ **Um pixel cujo material não espalha leva a visibilidade DURA**, e não o borrão de um vizinho:
ele não tem termo de subsuperfície para a ler, e emprestar-lhe um raio seria inventar espalhamento
onde o artista pôs zero.

### §23.2 — ⛔ E o `maior_espalhamento` foi APAGADO

Ele ficou sem chamador de produto, e *um método que ninguém chama é lixo* — a lei que o
`Surfaces::of` deste repo já escreve. ⚠️ **O gate que media o vazamento morreu com ele** e foi
reescrito do lado da cura: hoje afirma o **preço** (uma passagem por valor distinto), e a lei em si
é gateada na crate do borrão, **ao bit**, contra o que cada peça teria sozinha.

### §23.3 — ⚠️⚠️ TRÊS mutações sobreviveram, e cada uma nomeou uma cegueira diferente

1. **Um pixel opaco levava o borrão de um vizinho** — a fixtura tinha as **duas** peças
   translúcidas, logo o braço do opaco nunca corria. ⭐ E ao curá-la apareceu um defeito **na
   própria cura**: com *«translúcida + opaca»* há **um** valor distinto ⇒ cai no caminho rápido,
   onde a pergunta *«de quem é este pixel?»* nem é feita. ⇒ a fixtura passou a ter **três** peças
   (dois raios translúcidos **mais** o opaco), que é o mínimo em que o caminho por-pixel corre.
2. **A dedução dos valores distintos estava escrita DUAS vezes** — no produto e no gate do preço —,
   e apagar a do produto **não movia a imagem**: todas as passagens dão o mesmo resultado, só o
   **custo** dobra. ⇒ *um gate que reimplementa o que mede não mede nada*, e a dedução virou porta.
3. **O gate da saída é cego ao preço** — e o do preço é cego à saída. *Duas grandezas, dois gates.*

**5 mutações, 5 sangram.** Portão: `2 221` testes verdes, clippy `-D warnings` a zero.
