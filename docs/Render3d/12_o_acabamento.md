# 12 — O ACABAMENTO (a `W7`): o brilho, o anti-serrilhado e a profundidade de campo

> **Ordem do dono, 2026-09-19:** *«8 e depois do smoke o 7»* — e o smoke da `W8` foi aprovado no
> mesmo dia. Este é o **último ingrediente** dos oito do [`01`](01_o_alvo_decomposto.md).
>
> ⚠️ **Este documento é o PLANO, escrito antes da primeira linha de produto** (`/pd-feature`). O que
> ele contém de medido está medido; o que ele propõe está marcado como proposta.

## §1 — ⭐ A triagem de licença, e ela pára na PRIMEIRA porta aberta

`CLAUDE.md` §0.9 manda a triagem primeiro, e **pelo ARTEFACTO instalado, nunca pelo nome do
projecto**. Medido em 2026-09-19, perguntando ao próprio binário:

```
$ godot --version
4.7.2.stable.arch_linux.ed1daf0bf
$ Engine.get_license_text()   # 21 linhas
Copyright (c) 2014-present Godot Engine contributors … Permission is hereby granted, free of
charge, to any person obtaining a copy … without restriction …
```

⇒ **MIT**. ⭐ **Porta aberta: porta-se, com atribuição — sem parede, sem subagentes E/R, sem
vassoura** (é o que o [arsenal](../_ComoInvestigarApps/01_o_arsenal.md) já declara para este alvo).
⚠️ E o binário **não vem do `pacman`** (`/usr/bin/godot`, instalado à mão), que é exactamente porque
a licença se leu **no artefacto** e não num gestor de pacotes.

## §2 — O que o modo Render tem hoje: NADA, e o pipeline fecha POR PIXEL

Medido contra o código (`grep` sobre `ph2d-field-render`, `ph2d-field-gpu`, `ph2d-view-transform`):
**zero** ocorrências de bloom, DOF ou AA temporal. O que existe chama-se `antialias` e é a média
geométrica de borda do marchador ([`edges.rs`](../../crates/ph2d-field-render/src/edges.rs)), que é
outra coisa.

⛔⛔ **E o achado que decide o preço da wave:** os dois motores fecham o quadro **pixel a pixel** —

```rust
// shade_render.rs, o fim da função
let cena = pres.style.apply(…);      // a camada de estilo (W8)
pres.look.apply(cena)                 // a exposição + a vista (W1)
…
px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);   // e vira BYTE aqui
```

*Nada guarda a imagem em HDR.* O brilho tem de ler a imagem **antes** da exposição e do byte, e
hoje esse valor existe durante uma expressão e morre. ⇒ **a primeira peça da wave não é um efeito,
é um buffer** — e ela não muda um pixel do que se vê.

## §3 — O ORÁCULO, corrido sem interface

### §3.1 A porta, medida

`--headless` do Godot força o driver de render **dummy** (⇒ sem brilho), e esta máquina **não tem
Xvfb**. A porta que funciona é a **sessão KWin virtual** que a casa já paga para fotografar cenas
([`fotografa_cena.sh`](../Components/ferramentas/fotografa_cena.sh)):

```
kwin_wayland --virtual --xwayland --exit-with-session <sessao.sh>
  └─ godot --path <proj> --display-driver x11 --rendering-driver vulkan \
           --write-movie <saida.png> --quit-after <n>
```

Medido: `Vulkan 1.4.351 — Forward+ — NVIDIA GeForce RTX 5060 Ti`, `0,03 ms/quadro` de GPU, PNG
gravado. ⚠️ **Uma sessão virtual serve N corridas** — a placa é cara de segurar (`CLAUDE.md` §2: a
regra ali é *exclusão + prazo*), então a varredura abre a sessão **uma vez** e corre as células
dentro dela.

### §3.2 A superfície de botões, despejada da API (`--doctool`)

`Environment` expõe **18** botões de brilho, com estes valores de fábrica:

| botão | fábrica | | botão | fábrica |
|---|---:|---|---|---:|
| `glow_enabled` | `false` | | `glow_hdr_threshold` | `1.0` |
| `glow_intensity` | `0.3` | | `glow_hdr_scale` | `2.0` |
| `glow_strength` | `1.0` | | `glow_hdr_luminance_cap` | `12.0` |
| `glow_mix` | `0.05` | | `glow_bloom` | `0.0` |
| `glow_blend_mode` | `1` (*Screen*) | | `glow_normalized` | `false` |
| `glow_levels/1..7` | `0 · 0.8 · 0.4 · 0.1 · 0 · 0 · 0` | | `glow_map_strength` | `0.8` |

Os cinco modos de mistura: `0` Additive · `1` **Screen** · `2` Softlight · `3` Replace · `4` Mix.

⚠️ O `--doctool` despeja a **superfície** (nomes, tipos, omissões, enums) e **não** a prosa — e isso
é o lado certo: §0.9 diz que o alvo se **corre**, e a prosa responderia *como* onde um gate precisa
de *o quê*.

### §3.3 ⭐ A LEI DO LIMIAR — medida, com controlo

Um quadrado não iluminado de cor `(i,i,i)` sobre fundo preto, tonemapper **Linear**, exposição `1`,
tudo o resto de fábrica. O halo é a soma em **linear** de tudo o que está **fora** do quadrado:

| entrada | halo (soma linear) | pico do halo | raio até `1/255` |
|---:|---:|---:|---:|
| `0,5` | **`0,00`** | `0,00000` | `0` |
| `0,75` | **`0,00`** | `0,00000` | `0` |
| `1,0` | **`0,00`** | `0,00000` | `0` |
| `1,5` | `1 408,43` | `0,29177` | `89` |
| `2,0` | `4 028,09` | `0,79910` | `98` |
| `4,0` | `9 856,80` | `1,00000` | `106` |
| `8,0` | `10 655,88` | `1,00000` | `107` |
| **`4,0` com o brilho DESLIGADO** (controlo) | **`0,00`** | `0,00000` | `0` |

**Três leis saem desta tabela:**

1. ⚠️⚠️ **A `1,0` o halo é ZERO, não «pouco»** — e a 1.ª redacção desta linha dizia *«o limiar é
   DURO e ele é exactamente o `glow_hdr_threshold`»*. **A atribuição estava errada** e a §3.5
   desmente-a: esta varredura correu com o limiar no valor de FÁBRICA, logo ela mede a ENTRADA e
   não o botão. *Uma varredura de uma variável com as outras no default mede a variável, nunca a
   causa.*
2. ⭐ **O RAIO quase não depende da intensidade** (`89 → 107 px` sobre `5,3×` de entrada): a
   extensão do halo é uma propriedade da **cadeia de níveis**, não do brilho.
3. ⛔ **E a minha régua SATURA:** o pico do halo lê `1,00000` a partir de `4,0` porque a saída é de
   **8 bits** — a coluna da soma continua a subir e a do pico já não diz nada. *Uma régua que mede
   a saída do produto herda o tecto da saída do produto*; a coluna honesta para a intensidade é o
   **raio**, e para a forma é o perfil longe do quadrado.

### §3.4 ⭐⭐ A GEOMETRIA DA CADEIA — um nível de cada vez

O mesmo quadrado a `4,0`, com **um** `glow_levels/k` a `1,0` e os outros seis a `0`:

| nível | raio até `1/255` | **raio a meia altura** |
|---:|---:|---:|
| 1 | `7` | `3` |
| 2 | `25` | `9` |
| 3 | `58` | `20` |
| 4 | `119` | `38` |
| 5 | `192` ⚠️ | `55` |
| 6 | `192` ⚠️ | `151` |
| 7 | `192` ⚠️ | `192` |

⭐⭐⭐ **O raio a meia altura DUPLICA por nível** (`3 · 9 · 20 · 38 · 55`), que é a assinatura de uma
**cadeia de mip**: cada nível desce a resolução a metade e borra com o mesmo núcleo, logo o borrão
em pixéis de ecrã dobra. ⚠️ **Os níveis `5`, `6` e `7` batem no bordo de um quadro de `512`** — a
`192` a medição é a moldura e não o nível —, e é por isso que os valores de fábrica dele acendem
**`2`, `3` e `4`** e deixam os outros a zero: *a fábrica é a faixa que cabe no ecrã*.

### §3.5 — ⛔⛔⛔ E o CORTE do alvo não é uma lei limpa: três botões que se pisam

Variando **o botão** em vez da entrada (quadrado a `4,0`, halo lido a `60 px` da borda):

| botão | valores varridos | o que o halo faz |
|---|---|---|
| `glow_hdr_threshold` | `0 · 0,25 · 0,5 · 1 · 2 · 3` | **`0,0160 → 0,0152`** — `5 %`, que é a quantização de 8 bits |
| `glow_hdr_scale` | `0,5 · 1 · 2 · 4` | **nada**: as quatro células lêem o mesmo |
| `glow_hdr_luminance_cap` | `1 · 3 · 12` | `0,100 · 0,289 · 1,000` — **o único que manda** |

E a `entrada 2,0`, onde um limiar teria mais força, ele move `27 %` sobre uma faixa de `0 → 3`:
`0,1095 · 0,1046 · 0,0953 · 0,0802`. ⛔ **Com limiar `3` e entrada `2` ainda brilha `0,0802`** —
um corte a sério daria **zero**. E o `glow_bloom` faz brilhar uma entrada de `0,5`, que está
**abaixo** de qualquer limiar (`0 · 0,00273 · 0,00518`, linear nele).

⇒ ⭐⭐⭐ **A GEOMETRIA dele é limpa e porta-se; o CORTE dele não é uma lei — é três botões que se
pisam, um deles quase inerte e um limiar que não gateia.** Portar esse comportamento seria herdar
um painel em que *«Threshold»* não faz o que o nome diz.

**Decisão:** portamos a **cadeia** (§3.4, medida e limpa) e **escrevemos o nosso corte, declarado** —
um limiar que de facto gateia, com o gate a prová-lo. É o mesmo movimento que o pincel de POSE fez
ao bater o modelo de referência da própria espec: *a referência responde o que outro programa FAZ;
ela não obriga a repetir o que ele faz MAL.*

## §4 — ⛔⛔ A DECISÃO DE ARQUITECTURA: o brilho é um PASSE, e isso já estava escrito

A [`11` §11](11_a_camada_de_estilo.md) recusou pôr o contorno desenhado na `ph2d-style` com este
mecanismo, palavra por palavra:

> ⛔ **O CONTORNO desenhado fica de fora:** ele lê os VIZINHOS no ecrã, logo é um **passe** e não uma
> multiplicação — pô-lo na crate da lei obrigaria-a a receber um G-buffer e ela deixaria de ser a
> lei que os dois motores partilham.

**O brilho lê os vizinhos.** ⇒ ele **não** entra na `ph2d-style` nem na `ph2d-view-transform`; ele é
uma crate-folha nova de PASSE, com a lei em Rust e o gémeo em WGSL, como todo o resto desta pilha.

⚠️ **E a ordem no quadro é load-bearing:** `sombreamento → ESTILO → [brilho] → exposição/vista →
byte`. O brilho lê **depois** do estilo (senão um contorno pintado não brilha) e **antes** da
exposição (senão ele mede bytes já comprimidos, que é o defeito que o `01` §2 nomeia como *«sem o
`1`, o bloom mente»*).

## §5 — As waves propostas

> ⚠️⚠️ **As `W7a` e `W7b` FUNDEM-SE, e a razão foi medida:** um buffer de HDR sem consumidor é
> **código morto** — a lei da casa é que *cada canal só é assado se o consumidor DELE estiver vivo*
> ([`11` §11](11_a_camada_de_estilo.md)). Elas shipam juntas, e a prova de que o substrato é inerte
> continua a ser a mesma: **com o brilho desligado, a imagem é a de hoje ao bit.**
>
> ⭐ E a assinatura **não** muda nos `50` chamadores: `shade_render` passa a **delegar** numa porta
> que devolve o quadro inteiro (o molde do `move_character`/`move_character_from` do TOP-20 #13).
> Dos `50`, **um** é produto — o resto são gates e sondas.

### `W7a` — O QUADRO EM HDR (substrato, zero mudança visível)

O sombreamento passa a escrever um `Vec<[f32; 3]>` linear, e a conversão para byte passa a ser um
**passo separado**. ⭐ A prova de que não mudou nada é **byte a byte**: a imagem de hoje e a de
depois têm de ser idênticas nos dois motores.

⚠️ No dispositivo isto é um alvo de render em `rgba16float` em vez de escrever `u32` — e o tecto
dele é **memória de GPU**, a medir (`1920×1080×8 B ≈ 16,6 MB`).

### `W7b` — O BRILHO (o efeito que o dono nomeou)

A crate-folha nova (proposta: `ph2d-bloom`, zero dependências, o molde da `ph2d-style`), com a
cadeia de mip, o limiar duro e os pesos por nível. Botões no painel, na secção do Render.

⛔ **Os valores de fábrica saem do oráculo, não de gosto:** limiar `1,0`, `intensidade 0,3`,
`níveis 0 · 0,8 · 0,4 · 0,1 · 0 · 0 · 0`, mistura *Screen*.
⚠️ **E o número de níveis é DERIVADO da resolução**, não fixo em `7`: a `512` só cinco cabem
(§3.4). *Um nível cujo borrão é maior que o quadro mede a moldura.*

### `W7c` — O ANTI-SERRILHADO ao mexer

O que o dono vê é a peça a «ferver» na borda enquanto orbita. ⏳ A forma ainda não está medida — a
escolha é entre acumular quadros (temporal) e amostrar melhor a borda no próprio marchador, e a
segunda é mais barata aqui porque **a borda deste renderer é analítica** (o `edges.rs` já sabe onde
ela está). *Medir antes de escolher.*

### `W7d` — A PROFUNDIDADE DE CAMPO

⚠️ **Recomendação: FICA DE FORA por omissão**, e a razão não é preço — é que num **modelador** ela
esconde a peça que se está a modelar. Ela é uma feature de **apresentação**, e o sítio dela é junto
do botão de exportar imagem, não no viewport de trabalho. ⛔ **Decisão do dono** (ele já disse que
a ouve: *«se preferir que a profundidade de campo fique de fora, diga»*).

## §6 — A UI

A secção **Bloom** no painel do Render, a seguir à **Style**, com as mesmas leis que a `W8` pagou:

- ⛔ **Toda linha que o modo não lê fica à vista, apagada, com a razão ao lado** (`ParamRow::inert`,
  decisão do dono de 18/09) — e aqui há quatro candidatas por construção: com o brilho desligado,
  as outras não fazem nada.
- ⛔ **Cada cor/número é um controlo SEU** — o defeito das cinco cores num controlo
  ([`11` §9](11_a_camada_de_estilo.md)) custou um espaço de nomes próprio, e este painel herda-o.
- ⚠️ **O tecto de cada slider vem de uma medição**, com a tabela ao lado (`CLAUDE.md` §0.0) — os
  três tectos que a auditoria da `W8` apanhou como palpite ([`11` §10.8](11_a_camada_de_estilo.md))
  são o precedente.

## §7 — Os gates que esta wave deve

| gate | o que afirma |
|---|---|
| `o_hdr_nao_muda_um_byte` | a `W7a` é byte-idêntica nos dois motores (a prova de que o substrato é inerte) |
| `o_que_nao_passa_do_limiar_nao_brilha` | a `§3.3` no caminho do produto: a `1,0` o halo é **zero** |
| `o_raio_de_cada_nivel_dobra` | a `§3.4`, medida no nosso motor contra a tabela do oráculo |
| `a_fabrica_e_a_identidade` | com `glow_enabled = false` a imagem é a de hoje **ao bit** |
| `o_brilho_le_depois_do_estilo` | um contorno pintado pelo estilo **brilha** (a ordem do §4) |
| `os_niveis_cabem_no_quadro` | o número de níveis é derivado da resolução, com piso de população |
| paridade CPU↔dispositivo | a mesma barra dos outros passes, e ⛔ **não afrouxada** |

## §8 — O smoke (para o dono)

A cena **`=36`** (a seguir à do estilo): a mesma peça, com uma **fonte de luz forte** e uma zona
escura ao lado — *sem as duas, o brilho não tem o que provar*. O roteiro nomeia os controlos pelo
nome que aparece no ecrã, e o gate `o_roteiro_da_cena_nomeia_controlos_que_existem` da `W8` já o
cobra automaticamente.

## §9 — ⚠️ Armadilhas MEDIDAS neste plano

- ⛔⛔ **Sete células do oráculo leram EXACTAMENTE o mesmo número, e eu imprimi uma tabela elegante
  sobre uma imagem em branco.** A cena do oráculo nunca foi construída (um erro de tipo no
  `_ready`), o quadro era a cor de fundo (`76` em todos os pixéis), e a minha régua somou-a com
  três casas decimais. ⇒ *toda varredura leva um **CONTROLO que tem de diferir*** (aqui,
  `glow_enabled = false`), e a primeira coisa a olhar num corpus é `min == max`.
- ⛔ **O `--headless` do Godot desliga o render** — ele é para script e servidor. Quem quiser a
  imagem corre na sessão virtual.
- ⛔ **A régua da intensidade satura a 8 bits** (§3.3): a coluna certa é o raio.
- ⚠️ **`kwin_wayland` precisa de `--exit-with-session`**; sem ele o comando não corre e não há log
  nenhum para ler — o modo de falha é **mudo**.

## §10 — ⛔⛔⛔ O report de 2026-09-19: o halo era CALCULADO, SOMADO, e NUNCA chegava ao ecrã

> *«Talvez devido a total falta de atmosfera não se possa perceber o efeito ao redor das esferas»*
> — o dono, com a foto da `=36` em Render, as três bolas brancas e **nenhum halo**.

### §10.1 — O que a medição disse, pelo caminho do produto

Com o app a correr (`=36`, `PH2D_FIELD_GPU=0`, Bloom ligado, `1898×916`):

| grandeza | medido |
|---|---:|
| pico da cena em linear (`campo_de_cena`) | `11,8381` |
| pico do halo (`ph2d_bloom::halo`) | `44,8896` |
| canais que o `soma_halo` MUDOU | **`2 738 165` de `5 215 704`** |
| maior salto de um canal | `255` bytes |
| o que se via na tela | **nada** |

⭐ O buffer que a thread de traçado produz foi despejado e **tem o halo**, magnífico. ⇒ *a lei
estava certa e o quadro estava certo; o que falhava era o que acontece DEPOIS dele.*

### §10.2 — A causa: luz com cobertura ZERO

O modelador traça com `BACKGROUND = [0, 0, 0, 0]` — **transparente de propósito**, para o canvas
do app (o cinzento, a grelha) aparecer por baixo; a decisão está escrita no próprio
`ph2d-app-field3d/src/smoke.rs` desde um smoke do dono de 19/08. E **um halo mora, por definição,
onde a peça não está** — ou seja, exactamente onde a cobertura é zero.

O `soma_halo` somava a luz e deixava o alfa como estava, com a frase *«luz acrescentada com alfa
inalterado é o que um compositor lê como luz»* ao lado. Essa frase é a álgebra do pré-multiplicado
e **é uma afirmação sobre o CONSUMIDOR** — e este consumidor compõe o quadro sobre o canvas e
multiplica a cor pela cobertura. *Luz com cobertura zero é luz multiplicada por nada.*

⭐ **A cura:** o halo é uma CAMADA de luz, e uma camada tem cobertura —
`c = max(r, g, b)` da luz que ela põe, composta como toda camada: `c + (1 − c)·a`. É **a mesma
conta** que a sombra do chão já fazia em `ground_shade::shadowed_background` (`(1 − f) + a·f`).
*A sombra já sabia que tapar o fundo custa cobertura; a luz é que não sabia.*

⚠️ **E o arredondamento é para CIMA.** A cor vai em **sRGB** e a cobertura em **linear**, e as duas
quantizam a ritmos muito diferentes: com arredondamento ao mais perto, `12 778` de `52 167` píxeis
acesos da cena ficavam com cobertura `0` (um linear de `0,0005` sai byte `6` na cor e `0` no alfa).
*Um pixel que recebeu luz nunca fica com cobertura nenhuma*, e o preço máximo é `1/255` de véu.

⚠️ **E a guarda `if c > 0.0` que a 1.ª redacção da cura tinha era provadamente MORTA** — a ida e
volta `byte → f32 → byte` é exacta nos **256** valores (medido). Ela saiu, e a propriedade que
alegava proteger virou gate (`o_alfa_atravessa_a_lei_sem_perder_um_byte`).

### §10.3 — ⛔⛔ Porque NENHUMA régua desta wave o via — são DUAS cegueiras somadas

1. **A fixtura dos gates do passe pintava sobre fundo OPACO** (`FUNDO = [0, 0, 0, 255]`). Ali o
   alfa é `255` em todo o quadro e **nunca chega a ser a grandeza que decide** — os sete gates
   mediam bytes de COR e passavam todos. *Uma fixtura com fundo opaco não pode conter este
   fenómeno.*
2. **A única coisa que alguém OLHOU foi a sonda `despeja_o_halo` — e ela escreve PPM**, um formato
   **sem canal alfa**. Foi com ela que eu verifiquei o halo antes de o entregar ao dono.
   ⭐⭐⭐ *Um despejo que deita fora um canal não pode auditar esse canal* — e o defeito vivia
   exactamente nesse canal.

⇒ Os gates novos pintam sobre **os dois** fundos, e o da crate do app corre a **cena a sério** com
o `BACKGROUND` **lido** do módulo (não repetido): se alguém o tornar opaco, é esse gate que diz que
a lei mudou de sujeito. Prova de mutação: **4 de 4 sangram** — e a 4.ª (`max` → `soma`) só passou a
sangrar quando ganhou uma fixtura com halo **TINGIDO**, porque *um corpus de uma cor só não testa
uma lei que fala de canais*.

### §10.4 — ⏳ O VIZINHO, nomeado e NÃO curado

`ground_shade::mais_luz` — a cor que a peça devolve ao chão (`docs/Render3d/09`) — tem a mesma
forma: soma luz e deixa o alfa. Pelo mesmo mecanismo, ela só chega ao ecrã onde a **sombra** já deu
cobertura (`a = 1 − f`), e evapora onde o chão está limpo. Ali isso é quase sempre invisível (a luz
devolvida é forte justamente dentro da sombra) e mudá-lo mexe na imagem de **toda** cena com chão
⇒ fica nomeado aqui, com o mecanismo, para quem lhe pegar não redescobrir a causa.

## ⛔ Recusas MEDIDAS

| o que foi recusado | porquê, com o número |
|---|---|
| pôr o brilho na `ph2d-style` | ele lê os VIZINHOS ⇒ é um passe; a crate deixaria de ser a lei partilhada (§4, a mesma recusa da [`11` §11](11_a_camada_de_estilo.md)) |
| ler a lei do brilho no fonte do Godot | é legal (MIT) e é o método pior: o oráculo **corre-se**, e a §3.3/§3.4 saíram de `15` corridas (§0.9) |
| `--headless` para colher o oráculo | força o driver **dummy**: a saída é preta e lê-se como «não há brilho» |
| medir a intensidade pelo PICO do halo | satura em `1,00000` a partir de `entrada 4,0` — 8 bits (§3.3) |
| fixar a cadeia em `7` níveis | a `512` px os níveis `5`–`7` são maiores que o quadro e medem a moldura (§3.4) |
| portar o CORTE do alvo (os três botões de HDR) | dois estão inertes e o limiar não gateia: entrada `2` com limiar `3` ainda brilha `0,0802` (§3.5) |
| mudar a assinatura do `shade_render` | `50` chamadores, `1` de produto — a porta nova **delega** e o churn é zero (§5) |
| uma wave só de substrato (o buffer sem o brilho) | um canal sem consumidor é código morto, que é a lei que a `11` §11 já escreve (§5) |
| o `if c > 0.0` à volta da cobertura | a ida e volta `byte → f32 → byte` é exacta nos **256** valores ⇒ guarda provadamente morta (§10.2) |
| `soma` dos canais como cobertura | over-cobre um halo tingido: `[0,4 · 0,3 · 0,3]` sairia **opaco** e taparia a grelha; `max` é a MENOR cobertura válida (§10.3) |
| a profundidade de campo LIGADA por omissão | num modelador ela esconde a peça que se está a modelar — proposta, decisão do dono (§5) |
