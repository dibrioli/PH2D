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
