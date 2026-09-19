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

1. ⭐⭐ **O limiar é DURO em `1,0`** — exactamente o `glow_hdr_threshold`, e a `1,0` o halo é
   **zero**, não «pouco». *O que não passa do limiar não brilha, ponto.*
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

## ⛔ Recusas MEDIDAS

| o que foi recusado | porquê, com o número |
|---|---|
| pôr o brilho na `ph2d-style` | ele lê os VIZINHOS ⇒ é um passe; a crate deixaria de ser a lei partilhada (§4, a mesma recusa da [`11` §11](11_a_camada_de_estilo.md)) |
| ler a lei do brilho no fonte do Godot | é legal (MIT) e é o método pior: o oráculo **corre-se**, e a §3.3/§3.4 saíram de `15` corridas (§0.9) |
| `--headless` para colher o oráculo | força o driver **dummy**: a saída é preta e lê-se como «não há brilho» |
| medir a intensidade pelo PICO do halo | satura em `1,00000` a partir de `entrada 4,0` — 8 bits (§3.3) |
| fixar a cadeia em `7` níveis | a `512` px os níveis `5`–`7` são maiores que o quadro e medem a moldura (§3.4) |
| a profundidade de campo LIGADA por omissão | num modelador ela esconde a peça que se está a modelar — proposta, decisão do dono (§5) |
