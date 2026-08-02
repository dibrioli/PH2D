# 31 — Handoff: avaliar e otimizar o modo **Watercolor**

> ⚠️ **CUMPRIDO em 2026-08-02 — leia a §5.71 do [doc 28](28_otimizacoes_o_que_funcionou.md) e o
> [handoff de integração](../HANDOFF_INTEGRACAO_line_Painter_watercolor_cadence_2026-08-02.md).**
> Este documento continua **válido e correto no que mede** (a tabela de ablação, o piso de ruído, as
> três coisas que ele proíbe). O que ele não sabia:
>
> - **A §7 dizia *"comece rodando a sonda da §2.2"*, e a sonda mede um raio só (100).** O custo é
>   **LINEAR no raio** — 11,87 ms/move a r=400 — e o Enio pinta a 300. A conclusão *"3,1 ms não é um
>   problema"* valia exatamente onde foi medida.
> - **A aquarela cobrava por EVENTO de ponteiro, não por dab** (o Digital, no mesmo teste, é plano):
>   o mesmo desenho custava **2,56×** mais num dispositivo de 960 Hz. Curado — a reconstrução agora é
>   uma por QUADRO, byte-idêntica e com latência zero.
> - **O pen-down (~75-112 ms) não aparece em nenhuma tabela daqui**, porque o `move_ms` da sonda o
>   descarta de propósito. Era 268 MB alocados para reproduzir uma cor chapada. Curado.
>
> A §3 (o que **não** fazer) **segue inteiramente de pé**: os 9 taps do warp continuam intocáveis, a
> fatoração dos dois eixos continua sem render produto, e aproximar o warp continua exigindo ordem do
> Enio. O warp segue sendo 56% do que a aquarela cobra sobre o Digital.

> Escrito em 2026-08-02 para o agente que assume a `line/Painter` com esta tarefa.
> Leia isto **antes** de medir qualquer coisa. Ele existe para você não gastar a sessão
> redescobrindo o que já está medido, nem construindo o que já foi reprovado.

---

## ⚠️ 1. Leia isto primeiro: **Watercolor não é Wet Paint**

Este app tem **quatro meios de pintura**, e dois deles têm nomes parecidos:

| Meio | O que é | Onde mora |
|---|---|---|
| **Watercolor** ← **a sua tarefa** | Um pincel com *warp*, granulação, escurecimento de borda e mistura de pigmento. **Não há simulação de fluido.** | `tool/paint/watercolor_*.rs` |
| **Wet Paint** | Uma **simulação de fluido** completa, com solver, thread própria e grade configurável. | crate `ph2d-wet-paint` |

⛔ **O Wet Paint já teve doze waves de performance** (doc 28, §5.31 a §5.57) e a frente de CPU
dele está **fechada por medição**. Se você começar a ler sobre `advect`, `drying_pass`, `Grid
Size` ou `Flow Grid`, **você está no módulo errado**.

O jeito mais rápido de confirmar que está no lugar certo: o seu alvo é `PaintMedia::Watercolor`
(valor `1` em `tool/paint/media.rs`), não `PaintMedia::WetPaint` (valor `3`).

---

## 2. O que já está medido (não re-meça)

Tudo abaixo saiu **da porta do produto** (`on_canvas_pointer`), com a máquina calma.

### 2.1 O custo de um movimento de mouse, nos quatro meios

| Meio | 2048² | 4096² |
|---|---|---|
| Digital | 1,17 ms | 1,21 ms |
| **Watercolor** | **3,07 ms** | **3,12 ms** |
| Impasto | 2,00 ms | 1,93 ms |
| Wet Paint | 2,32 ms | 1,82 ms |

✅ **O Watercolor tem a forma CERTA:** o custo não sobe com o tamanho da tela. Isso significa
que ele é limitado pela **pegada do pincel**, que é o correto. Não há varredura de tela inteira
escondida ali — essa classe de defeito já foi procurada e não existe.

### 2.2 De que é feito o custo dele

A decomposição foi feita **desligando knobs do painel**, um a um — nunca instrumentando o laço
por dentro (uma sonda que refaz o laço fica cega à porta e passa a medir outra coisa).

| Ablação | Custo do move |
|---|---|
| Como shipa | 3,082 ms |
| **Sem o Warp** | **2,012 ms** |
| Tudo desligado | 2,011 ms |

⇒ **O warp é 1,071 ms, ou 56% de tudo que o Watercolor cobra a mais que o Digital.** E como
"tudo desligado" dá o mesmo que "sem warp", ele é **praticamente todo o custo que dá para
atacar**.

⚠️ **A tabela trouxe o próprio controle, e ele importa:** dois knobs (`wet_smudge`,
`wet_rewet`) **já valem 0 por padrão**, então as linhas deles medem **nada** — e mediram
−0,100 e −0,061 ms. Esse é o **piso de ruído da sonda**. Qualquer efeito menor que ~0,13 ms
nessa tabela é indistinguível de zero: Granulation (−0,085), Edge (−0,066) e Pigment mixing
(−0,126) estão todos aí. *Seria fácil escrever três "otimizações" em cima de ruído.*

### 2.3 Por que o warp custa

Não é a função ser cara — é ela ser chamada muitas vezes: **`warp_offset` roda 10× por texel**
(o centro + 9 amostras de anti-aliasing). Reduzir de 9 para 1 amostra levaria o warp a 0,511 ms.

---

## 3. ⛔ O que **não** fazer (já foi decidido ou medido)

### 3.1 Não corte as 9 amostras

Passar as 9 amostras pelo warp foi **a correção que curou a borda serrilhada** (com warp 48:
226 degraus de borda → zero). Cortá-las devolve um bug visível. **Isto não é opinião nem
trade — é uma cura anterior.**

### 3.2 A fatoração exata dos dois eixos já shipou, e **não deu ganho de produto**

Os dois eixos do ruído pedem a mesma oitava na mesma posição e só o `seed` difere, então a
aritmética de grade era feita duas vezes. Isso foi fatorado (`value_noise_pair`), é
**byte-exato**, e está gateado.

Resultado honesto: a **função** ficou 1,20× mais rápida (153,4 → 127,9 ms em 4 M avaliações),
mas no **produto** o ganho foi de 0,12–0,17 ms — **dentro do piso de ruído de ±0,13**. Ficou no
código por ser estritamente menos trabalho, não por ser um resultado.

⇒ **Não conte com micro-fatoração para mover este número.** Ela já foi feita.

### 3.3 Aproximar o warp dentro do texel **precisa de ordem do Enio**

É o que sobra como alavanca de CPU: em vez de avaliar o warp nas 9 amostras, aproximá-lo (por
exemplo, interpolando dentro do texel).

⚠️ Essa classe já foi **medida e rejeitada duas vezes** nesta mesma linha, no anti-aliasing do
impasto: o erro vinha das **quinas**, e tentar casar mais um momento da função **piorou**.

Se você for por aí, precisa de duas coisas antes de escrever código: um **oráculo de
aparência** (não um número de erro — uma comparação de imagem que saiba dizer se a borda
degradou) e a **ordem explícita do Enio**, porque muda o desenho do produto.

---

## 4. Onde as coisas estão

- **A sonda:** `crates/ph2d-tool-painter/src/tool/paint/measure_watercolor_cost.rs`
  (`measure_what_a_watercolor_move_is_made_of`). Rode com
  `cargo test -p ph2d-tool-painter --release <nome> -- --ignored --nocapture --test-threads=1`.
- **O motor:** `tool/paint/watercolor_*.rs` — o warp está em `watercolor_noise.rs`
  (com o oráculo congelado `warp_axis` sob `cfg(test)`, que é o código que shipava antes da
  fatoração: **não o chame no produto**, ele existe para o gate comparar).
- **O histórico:** doc 28, linhas **T** e **U** da tabela do topo, e as seções que elas citam.
- **Os planos do módulo:** docs 08, 10, 11, 12 e 13 desta pasta.

---

## 5. ⚠️ Regras da máquina (custaram tempo real nesta sessão)

1. **Nenhuma medição de tempo vale nada com `load average` acima de ~5.** Confira com `uptime`
   antes e depois. Hoje o mesmo teste, mesmo código, mediu 14 ms e 47 ms conforme a carga.
2. **Sondas de tempo rodam com `--test-threads=1`.** Duas sondas em paralelo disputam o mesmo
   pool de threads e passam a medir uma à outra.
3. **Todo comando do shell começa com o `cd` da worktree.** A pasta volta sozinha para a árvore
   primária, e o mesmo caminho existe nas duas — editar a errada compila e commita sem erro.

---

## 6. Estado da linha quando você a recebe

- **52 commits** locais, **nada enviado**. Suíte do Painter: **954 verdes em release, 953 em
  debug**. Clippy limpo. Árvore limpa.
- O trabalho recente foi no **motor de undo** (o pen-up de um traço grande caiu de 380 para
  179 ms a 4096²) — doc 28, §5.66 a §5.70. **Não toca no Watercolor**, mas está na mesma linha,
  então o seu trabalho vai junto na mesma integração.
- ⛔ **Você não integra e não faz push.** A linha fecha, escreve o handoff e para. Integração e
  ship são **ordem explícita do Enio**, sempre.
- A linha ainda **espera um smoke** do trabalho de undo (pintar, desfazer, refazer: a tinta e o
  relevo têm de voltar iguais). Isso é do Enio, não seu.

---

## 7. O primeiro passo que eu daria

1. Rode a sonda da §2.2 e confirme os números na sua máquina. Se eles não baterem, **pare e
   descubra por quê antes de otimizar** — a tabela é a base de tudo aqui.
2. Só então decida se o alvo é o warp (e aí leia a §3.3 inteira antes de escrever uma linha) ou
   se você encontrou algo que esta tabela não mostra.

⚠️ E o alvo honesto tem de ser dito em voz alta: **o Watercolor custa 3,1 ms por movimento, e o
orçamento de um quadro de 60 fps é 16,6 ms.** Ele não é um problema de performance hoje. Se a
sua avaliação concluir isso, **esse é um resultado legítimo** — e é melhor entregá-lo do que
inventar uma otimização dentro do ruído.
