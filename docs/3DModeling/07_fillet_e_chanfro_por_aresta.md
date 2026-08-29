# Fillet e chanfro **por aresta** e **por vértice** — avaliação medida

> **Pergunta do Enio, 2026-08-28:** *"avalie a possibilidade de chamfer e fillet por edge e por
> vertex, em vez do objeto todo. relate"*
>
> Sonda executável: [`spike_per_edge_radius.rs`](../../crates/ph2d-field-eval/tests/spike_per_edge_radius.rs)
> — 3 gates + 1 medição, `0,43 s`. Tudo o que este doc afirma sai dela ou de um doc já medido.

---

## §1 — O veredicto, em três linhas

| Pergunta | Resposta | Onde |
|---|---|---|
| Um raio por **grupo de arestas** de uma primitiva (as 4 verticais de uma caixa) | ⭐ **SIM**, e é barato e exacto | §3 |
| Um raio por **aresta individual** de uma primitiva (as 12 de uma caixa) | ⭐ **SIM** — `1,21×` no relógio, `‖∇f‖ = 1`, zero continuidade a defender. **O preço agora é a INTERFACE** | §4 |
| Um raio por **aresta do RESULTADO** de uma booleana (o fluxo do CAD) | ⛔ **NÃO** sem mudar de representação — e traria uma doença junto | §5 |
| Um raio por **vértice** | ⚠️ **Não é um item à parte: é a consequência do de aresta** | §6 |

---

## §2 — ⚠️ Nós e o modelador de referência estamos em pontas OPOSTAS

O estudo do modelador que este plano ia portar já tinha medido isto
([`00_plano_port.md` §1.3](00_plano_port.md)):

> | Fillet em **todas** as arestas de uma taça | **falhou** | Fillet é sempre **por aresta
> selecionada**; sem botão "arredondar tudo" |

⇒ **Ele só faz por aresta e não tem "arredondar tudo". Nós só fazemos "tudo" e não temos por
aresta.** A pergunta do Enio é, literalmente, a distância entre as duas.

⚠️ E a razão não é preguiça: é a **família** que o [ADR-0161](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)
escolheu, com o preço escrito na altura.

| | B-Rep (Parasolid, `monstertruck`, `brepkit`) | Implícito (nós, nTop, Womp) |
|---|---|---|
| O que é uma **aresta** | uma **entidade** com identidade — a curva de intersecção de duas faces | **não existe**; é uma descontinuidade do gradiente, emergente |
| Fillet por aresta | ✅ nativo (`fillet_edges`, `RadiusSpec` por aresta) | ⛔ não há em que pegar |
| Booleana que nunca falha | ❌ falha em geometria difícil | ✅ é `min`/`max` de dois números |
| «Arredondar tudo» | ❌ **não existe** (medido no referencial) | ✅ é o que temos |

---

## §3 — ⭐ O que É barato: um raio por GRUPO de arestas, MEDIDO

Uma caixa tem 12 arestas em **3 grupos de 4** (as paralelas a X, a Y, a Z). Um raio por grupo
constrói-se como a **intersecção de três barras de secção arredondada** — a barra em X limita
`|y|≤hy, |z|≤hz` com os cantos arredondados em `rx`, e arredonda exactamente as 4 arestas paralelas
a X.

**Medido na sonda:**

| | valor | veredicto |
|---|---|---|
| `‖∇f‖` pior | **`1,0000`** | ⭐ continua a ser uma **distância** — a marcha não abranda |
| Custo, em nós de árvore | `30 → 58` (**`1,93×`** a caixa arredondada) | e a caixa **viva** custa 28 |
| Só o grupo que pediu é arredondado | Δ `0,083` na aresta que pediu · **`< 1e-6`** na que não | ⭐ zero vazamento |

⭐⭐ **E o caso uniforme pode ficar BYTE-IDÊNTICO:** com os três raios iguais, o construtor continua a
ser o `sd_box` de sempre (30 nós); a construção de três barras só entra quando eles **diferem**. ⇒
nenhuma peça existente muda, e ninguém paga os `1,93×` por uma caixa que não pediu nada.

### §3.1 — ⚠️ Mas ela NÃO é a caixa arredondada de hoje, e o sítio onde difere é o achado

| direcção | `sd_box` | por grupo | Δ |
|---|---|---|---|
| **face** `(1,0,0)` | `0,50000` | `0,50000` | `+0,00000` |
| **aresta** `(1,1,0)` | `0,64497` | `0,64497` | `+0,00000` |
| **vértice** `(1,1,1)` | `0,75622` | `0,78993` | **`+0,03371`** |

O `sd_box` arredonda por **deslocamento** — a soma de Minkowski com uma **bola** —, então os 8 cantos
saem **esféricos**. Três barras arredondadas dão **cilindros** nas arestas e um canto de
**Steinmetz**. As duas coincidem na face e na aresta, e **divergem no vértice**.

---

## §4 — ⭐⭐⭐ Por aresta INDIVIDUAL numa primitiva: **MEDIDO (28/08), e é mais barato do que parecia**

*(o Enio pediu esta medição em 28/08; era o degrau que estava por medir)*

### §4.1 — A construção não tem SELECÇÃO nenhuma, e é isso que apaga o problema

A forma canónica (`sdRoundedBox` 2D) escolhe o raio pelo **quadrante** e aplica uma fórmula só —
isso pede um `select` na álgebra da árvore, e um `select` à mão pede uma constante de inclinação
**inventada**, que o cabeçalho do `ops.rs` proíbe.

⭐ **Em vez disso, cada canto é um termo exacto por si, e o rectângulo é a intersecção dos quatro:**
`canto(c, r) = ‖max(p − c, 0)‖ + min(max(dᵤ, dᵥ), 0) − r`. Ele vale três coisas de uma vez — no
quadrante dele é o arco; ao longo de cada face **degenera exactamente na distância àquela face**; e
no lado oposto é uma constante que o `max` ignora.

⇒ **não há região de transição, logo não há continuidade a defender** — a pergunta que a §4 anterior
tinha em aberto **dissolveu-se com a construção**, em vez de precisar de resposta.

⚠️ **A 1.ª redacção esquecia o termo interior** (`min(max(dᵤ,dᵥ), 0)`): sem ele o campo vale `−r`
dentro da peça em vez da distância, e com `r = 0` vale **zero**. O oráculo leu `0,300` de erro. É o
mesmo termo que o `cylinder_raw` desta crate já escreve, pela mesma razão.

### §4.2 — ⭐⭐⭐ A contagem de nós era a RÉGUA ERRADA

| construção | nós | × | **ns/ponto** | **× no relógio** |
|---|---|---|---|---|
| caixa viva | 28 | — | — | — |
| raio uniforme (**hoje**) | 30 | `1,00×` | `17,38` | `1,00×` |
| 3 raios (**por grupo**) | 58 | `1,93×` | `17,96` | **`1,03×`** |
| 12 raios (**por aresta**) | 210 | `7,00×` | `21,03` | **`1,21×`** |

> ⭐⭐⭐ **Sete vezes os nós é 1,21× o relógio.** A fita corre com JIT em oito faixas de SIMD, e o que
> custa é o **caminho crítico**, não a contagem: os quatro termos de canto são **independentes** e
> enchem faixas que estavam paradas. *Uma contagem de nós é um limite superior grosseiro do relógio,
> e aqui ele erra por 6×.*

⚠️ Estável em **3 corridas** (`1,21` · `1,21` · `1,23`, absolutos a `0,2 %`), a `load ~5–8`. É uma
**razão** medida no mesmo processo, back-to-back, com mediana de 7 — a forma mais robusta disponível
nesta máquina. ⚠️ E é o custo da **primitiva sozinha**: numa peça real ela é um nó entre vários, e a
marcha paga o caminho inteiro ⇒ na cena o efeito é **menor** que isto.

### §4.3 — E a capacidade funciona, com o controlo que importa

| afirmação | medido |
|---|---|
| `‖∇f‖` com os 12 raios **todos diferentes** | **`1,0000`** — continua a ser uma distância |
| 12 raios iguais ≡ a caixa por grupo | `< 2e-6` (o oráculo do degrau) |
| arredondar **UMA** aresta | Δ `> 0,02` nela · **`< 1e-6`** nas outras **onze** |

⭐ O controlo é a metade que importa: um gate que só medisse *«a aresta pedida mudou»* passaria com
«arredondar tudo».

### §4.4 — ⚠️ Então o obstáculo mudou de sítio: agora é a INTERFACE

Com `1,21×` na primitiva e zero problema de continuidade, **o motor deixou de ser o preço**. O que
sobra é como o artista escolhe:

- ⛔ **12 sliders numa caixa é inusável** — e um `Extrude` com um contorno de 40 pontos teria 40+
  arestas verticais, o que torna a lista impossível por construção.
- ⇒ o gesto que isto pede é *«aponto a aresta no canvas e escrevo o número»*, que precisa de
  **apanhar aresta na vista 3D**. Para uma **primitiva** isso é construível (as arestas têm nome
  estrutural e posição derivável da forma); é trabalho de UI, não de campo.
- ⭐ E há uma pista já paga: as arestas verticais de um `Extrude` **são as quinas do contorno 2D**,
  que já têm raio por vértice no editor vetorial (*Live Corners*). Aquele caminho pode já responder
  metade da pergunta para as formas de perfil — **não medido**.

⛔ Tudo isto vale para **primitivas**, onde a aresta tem nome estrutural. Não vale para o resultado
de uma booleana — §5.

---

## §5 — ⛔ Por aresta do RESULTADO: não, e o motivo é maior que o custo

É este o fluxo do CAD: *«clico nesta aresta do sólido e arredondo-a com 2 mm»*. Ele precisa de duas
coisas que a representação implícita não tem:

**1. A aresta não existe como coisa.** As arestas de `A − B` são as curvas onde a superfície de `A`
encontra a de `B`. O campo não as enumera — ele responde `f(p)` e mais nada. Dar raios diferentes a
duas delas exige um raio que **varia com a posição**, `r(p)`; e aí o campo deixa de ser uma distância
onde `r` varia, o que custa a promessa central do módulo (*«o raio pedido é o raio entregue, 0,00 %
de erro»*) e obriga a marcha a abrandar.

**2. ⭐⭐⭐ E traria junto o PROBLEMA DO NOME PERSISTENTE.** Para o raio sobreviver a uma edição, o
documento tem de **nomear** a aresta de forma durável. Em B-Rep isto tem nome próprio — *persistent
naming* — e é **o problema não resolvido** de todo kernel de CAD: é a razão de um modelo de histórico
"explodir" quando se muda uma cota lá atrás, e cada kernel tem a sua heurística para adiar o
sintoma.

> ⚠️ **A nossa representação não tem essa doença precisamente por não ter identidade de aresta.**
> Pôr selecção por aresta importaria a doença junto com a feature.

⛔ Há uma terceira saída que **não** recomendo esconder: um raio por junção **já existe** desde a W98
— o artista consegue raios diferentes decompondo a peça nas partes cujas junções são as arestas que
lhe interessam. É o fluxo nativo do SDF (é o que se faz no MagicaCSG e no Womp), e é honesto dizer
que ele pede outra maneira de modelar, não a do CAD.

---

## §6 — ⭐⭐⭐ «Por vértice» não é um item à parte: é a CONSEQUÊNCIA de «por aresta»

Isto saiu da medição, não de uma opinião. Na tabela da §3.1, face e aresta coincidem e **só o vértice
diverge** — porque num canto de caixa encontram-se **três** arestas, e assim que elas têm raios
independentes o canto deixa de ter resposta óbvia.

É exactamente por isso que os kernels de CAD tratam o *vertex blend* como **operação separada**, com
opções próprias (rolling-ball, setback, patch de N lados).

⇒ **A ordem é forçada:** por vértice não se pode desenhar antes de por aresta existir, porque ele é a
pergunta que o por-aresta cria.

---

## §7 — Recomendação

1. ⭐ **FAZER: um raio por grupo de arestas nas primitivas.** Três números numa caixa, dois num
   cilindro (o aro de cima e o de baixo). É exacto, mantém `‖∇f‖ = 1`, custa `1,93×` **só** quando os
   raios diferem, e o caso uniforme fica byte-idêntico. ⭐ E o painel **não muda**: as linhas de
   número saem do `params_of`, como o `Joint` da W98.
2. ⭐ **MEDIDO (28/08) — e o motor está liberado: por aresta individual custa `1,21×`** na primitiva
   e `‖∇f‖ = 1`. ⚠️ **O que ficou por resolver mudou de sítio:** não é o campo, é **apanhar a aresta
   na vista 3D** — 12 sliders numa caixa é inusável, e um perfil de 40 pontos torna a lista
   impossível. ⇒ o degrau seguinte é de **UI**, não de matemática.
3. ⛔ **RECUSAR: por aresta do resultado de uma booleana.** Não é preço, é representação — e traz o
   problema do nome persistente, que é a doença que não temos.
4. ⏸️ **ADIAR: por vértice.** Ele é a consequência do (2), e desenhá-lo antes seria responder a uma
   pergunta que ainda não existe.

---

## ⛔ Recusas MEDIDAS

| Recusa | Motivo medido |
|---|---|
| Fillet por aresta selecionada no **resultado** de uma booleana | a aresta não existe na representação, e nomeá-la de forma durável é o *persistent naming*, não resolvido em CAD nenhum (§5) |
| Substituir o `sd_box` pela construção de três barras | ela diverge no **vértice** (`+0,03371`): canto de Steinmetz contra esférico ⇒ mudaria toda peça já feita. A cura é usá-la **só** quando os raios diferem (§3) |
| Raio que varia com a posição (`r(p)`) para arredondar uma aresta e não a vizinha | mata a exactidão do raio entregue e obriga a marcha a abrandar em toda peça (§5) |
| Escolher o raio por **quadrante** com um `select` (a forma canónica do `sdRoundedBox`) | pede uma constante de inclinação **inventada**, e cria uma região de transição em que o campo muda de lei. A intersecção de quatro termos de canto dá o mesmo sem nenhuma das duas (§4.1) |
| Ler o preço na **contagem de nós** | `7,00×` os nós são `1,21×` o relógio — o SIMD enche faixas paradas com os termos independentes. *Errou por 6×* (§4.2) |
