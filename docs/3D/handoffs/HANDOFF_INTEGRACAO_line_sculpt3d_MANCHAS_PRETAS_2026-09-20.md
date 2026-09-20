# As manchas pretas são `NaN`, e a guarda existia numa das duas curvas

**Linha:** `line/sculpt3d` · **Data:** 2026-09-20 · **Merge-base:** `395da6a55`

> ⚠️ Este handoff **não supersede** o [da linha inteira](HANDOFF_INTEGRACAO_line_sculpt3d_A_LINHA_2026-09-20.md).
> Aquele é o documento do integrador (superfície de colisão, contadores, prova de fecho);
> este é a jornada que o report de 20/09 abriu, **depois** de a linha ter sido integrada.

## §1 — O report, e por que a pista estava errada

Dono, 20/09, com foto: pintar com a **topologia dinâmica armada** deixa **faces
INTEIRAS a preto, de aresta dura**, dentro da zona pintada, nos três verbos de cor.

O roteador deixou escrito que o refino estava *«gateado e provavelmente ilibado»* e
que a primeira pergunta era *«quem permuta o plano de cor quando um colapso renumera
a malha»*. **As duas metades dessa nota eram falsas**, e a tarefa mandava construir a
régua primeiro — foi ela que o disse.

### ⛔ A nota herdou a promessa de um NOME

`ph2d-mesh :: the_new_vertices_carry_colour_and_mask` promete a cor no nome e
media-a pelo **comprimento**:

```rust
let colors = m.colors().expect("a cor também");
assert_eq!(colors.len(), m.vert_count());
```

A máscara, ao lado, tinha asserção de **valor**. E a fixtura de cor era um xadrez por
índice par — *um campo sem valor esperado em ponto nenhum*, construído de um jeito que
torna a asserção de valor impossível de escrever. Um refino que fizesse todo vértice
novo nascer preto deixava aquele gate **verde**.

### ⛔ E o colapso MENCIONA a cor — na porta que ele chama

`collapse.rs` de facto não diz `colors`. A lei vive em `Mesh::shrink_topology`
(`crates/ph2d-mesh/src/mesh_shrink.rs`), a porta que ele chama, e o doc dela
**declara-se dona** por escrito, nas duas metades (a média do merge e a permutação do
`Remap`). *Uma ausência afirmada sobre o ficheiro do operador é um palpite sobre a
porta que ele usa.*

A metade dos **testes** estava certa: a palavra não aparecia uma vez em
`collapse_tests.rs`.

## §2 — A régua

`crates/ph2d-app-sculpt3d/src/manchas_pretas_tests.rs`.

**A cor uniforme é ponto fixo do passe.** Peça inteira em `C`, pincel a depositar `C`:
refino `(C+C)/2`, colapso `(C+C)/2`, permutação move `C`, depósito `C(1−f) + Cf`.
⇒ todo vértice lê `C` no fim, **seja qual for a topologia** — e é isso que a torna imune
à cadeia de renumeração: *uma permutação de valores iguais é invisível, logo o que ela
mede é só o que INVENTA um valor.*

Duas metades, porque a primeira é cega a uma permutação: a do **ponto fixo** e a do
**envelope** sobre um campo de duas cores. Mais o **piso de população** (a topologia tem
de ter mudado) e o **controlo sem passe** — que passou, e é o que atribui o defeito ao
passe em vez de à lei de cor.

⚠️ Ela percorre a **rota** (`passe_nos_motores` → `shrink_with` → `grow_with` → `dab`),
não as funções soltas. O arnês do censo dos knobs chama `begin` entre os dabs, o que
recongela o `base_color` a cada carimbo — aqui isso apagaria exactamente o estado que o
defeito poderia corromper.

### ⛔ A minha própria régua nasceu cega ao que procurava

`d > pior.0` com `d = NaN` é **sempre falso** ⇒ um máximo por comparação **salta todo
`NaN`** e devolve `0,0` sobre uma peça inteiramente envenenada. A metade uniforme passou
**por vácuo** na primeira corrida. *Uma régua escrita para achar um valor inventado não
pode usar a ordem dos `f32` para o achar, porque o valor que ela procura não está nessa
ordem.*

## §3 — A causa, em forma fechada

A bissecção (posições · normais · `base_color` do traço · cor da malha, a cada passo):

| dab | pos | nrm | cor | base | verts | |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 | 2 868 | `done` |
| 1 | 0 | 0 | 0 | 0 | 8 275 | `done` |
| **2** | 0 | 0 | **3** | 0 | 7 806 | **`cut`** + `done` |

O `NaN` entra no **carimbo** (nunca no passe nem na costura), na **cor**, e só a partir
do **primeiro colapso**. O `Draw` corre os mesmos colapsos e fica com **zero** `NaN` nas
posições ⇒ o `accum` está limpo pelo caminho dele.

Instrumentado o `apply_color` e depois o `w`:

```
[DIAG-T] t=1.0005066 gate=1.0 dist=0.45022795 inv_r=2.2222223
         paint_hardness=0.75 softness=0.5 | a geometria no mesmo t leria 0.0
```

- `channel_weight(t, h) = (1 − t)^{2(1 − h)}`
- de fábrica `paint_hardness = 0,75` ⇒ expoente **`0,5`**, uma raiz quadrada
- `t` medido **`1,0005` a `1,0108`** ⇒ base **negativa** ⇒ `powf` = **`NaN`** (IEEE-754)

Um `NaN` no canal **contamina a interpolação da FACE inteira** — que é a mancha de aresta
dura do report. E a guarda do dab (`if w <= 0.0 { return; }`) é **cega a `NaN`** pela
mesma aritmética que cegou a minha régua.

### Porque `t` passa de 1

A consulta (`verts_in_sphere`) usa as posições **VIVAS**; no envelope
(`from_live = false`, a lei do `Paint`) o peso sai da **CONGELADA** no pen-down. O colapso
pousa o sobrevivente no ponto médio (`positions[keep] = m.at`) e **não toca no `base_pos`**
⇒ as duas discordam. *Não é arredondamento: é a distância entre duas posições do mesmo
vértice.*

O `stroke_dab_core` escreve meia frase disto — *«a pegada já sai das posições vivas nos
dois casos, então no revezamento pegada e peso concordam»* — verdadeira no revezamento e
falsa no envelope, que é onde ninguém foi tirar a consequência.

⛔ **Hipótese alternativa REFUTADA:** um traço com `auto_smooth = 1,0` e **sem** passe
deixa a posição viva `0,0959` longe da congelada (`21 %` do raio) e produz **zero** `NaN`
— quem se move não é quem a pegada admite. *A deriva sozinha não basta; é preciso que a
malha ganhe ou perca vértices.* ⇒ o *«só com topologia dinâmica»* do report fica
confirmado por medição.

## §4 — A cura

`ph2d_sculpt3d::fora_da_pegada` (`crates/ph2d-sculpt3d/src/falloff.rs`) — uma **porta com
dois chamadores**, porque o defeito foi uma linha **AUSENTE** numa das duas cópias de uma
lei, e a outra (`Falloff::weight`) **documentava por escrito o mecanismo que a ausência
provocaria**:

```rust
// O NaN é peneirado explicitamente: sem isso ele escorre pelas fórmulas
// e sai como peso NaN num vértice, que é como uma malha inteira vira `NaN`…
```

*Uma lei escrita num sítio ainda não é uma lei; só uma porta é.* Um ramo que se esqueça
dela fica visível por **ausência de chamada**; duas cópias divergem outra vez em silêncio.

### ⭐ A referência SEMPRE teve esta guarda

O porte fiel do `Masking.js` vive nesta crate (`ref_mask::mask`) e escreve-a à letra —
`if dist > 1.0 { dist = 1.0; }` **antes** do `powf`. Ela perdeu-se quando a fórmula foi
reescrita como uma curva de `t`, **com a nota que declarava o ramo inalcançável a ocupar o
lugar dela**. ⇒ *esta cura é o regresso ao que o alvo faz, não uma divergência.*

### ⛔ Divergência DECLARADA, de um ramo que o alvo nunca executa

Ele clampa a **distância**, nós clampamos o **peso**, e as duas leis só discordam num
ponto: com `hardness = 1` exacto o expoente é `0` e **`0⁰ = 1`** em IEEE-754 — o original
devolveria peso **cheio** a um vértice fora do pincel. Fica a nossa: *um vértice fora da
pegada não é do dab*, e ali o alvo não tem lado aprovado (a consulta dele é `d² < r²`
estrita, logo `dist > 1` é inalcançável **no programa dele**).

### A premissa que morreu

O doc de `mask_weight` afirmava que o ramo era **inalcançável** e que *«a guarda mora onde
o consumo mora»*. As duas falsas: o consumo não tem guarda nenhuma, e o `t` medido no
produto vale `1,0005`–`1,0108`. Reescrito com a morte à vista no diff.

## §5 — O preço

**Zero.** A suíte da `ph2d-sculpt3d` fecha verde (`488 + 129 + 1`, zero falhas) — a cura
não custou uma paridade, o que é o que confirma que **nenhum corpus de oráculo continha um
`t` fora da pegada** (eles correm as leis directamente, com fixturas em que a pegada não
se move).

## §6 — Os gates

| onde | o quê |
|---|---|
| `ph2d-sculpt3d :: fora_da_pegada_tests` | a porta separa · **nenhuma curva devolve `NaN`** (com a família da geometria como controlo) · dentro da pegada a curva é a de sempre, **ao bit** contra a lei reconstruída à mão · o **censo** (toda curva pergunta à porta, população nomeada e piso) |
| `ph2d-app-sculpt3d :: manchas_pretas` | ponto fixo · envelope, os dois pela rota do produto |
| `ph2d-mesh :: the_new_vertices_carry_colour_and_mask` | a cor passa a ser medida por **valor**, com a lei lida dos `Birth` e composta **por gerações** |
| `ph2d-mesh :: collapse::cor_tests` | o colapso carrega cor e máscara — campo constante (ao bit) + envelope |

**Mutação: 8 a sangrar + 1 nomeada.** M1 apagar a guarda do canal · M2 `>=` → `>` · M3
apagar a peneira do não-finito · M4 apagar a guarda do `Falloff` · M5 tirar uma fonte do
censo · M6 o vértice novo nasce preto · M8 a média do merge · (M7, apagar a **permutação**,
**sobrevive** — ver §7).

### ⛔ E a M4 apanhou o meu próprio censo a ler a prosa

Com a chamada apagada ele ficou **verde**, porque encontrou o texto `fora_da_pegada` no
**comentário que explica a chamada**. Os comentários saem antes da varredura agora.
*Uma régua textual que lê a prosa ao lado do código mede a prosa.*

## §7 — Aberto, com o mecanismo

- ⚠️ **A permutação da cor no colapso continua sem régua.** A mutação que apaga
  `c[to] = c[from]` **sobrevive** às duas metades do gate novo: trocar `C` por `C` é
  invisível, e permutar para dentro do envelope também. A régua que a veria aplica o
  `Remap` à mão sobre a tabela de antes e conta quantos divergem contra o número de
  colapsos. **O doc do gate dizia que a 2.ª metade a via, e a mutação provou que não** —
  está reescrito com a medição.
- ⚠️ **A `mask_weight` está exposta ao mesmo mecanismo e nunca foi observada.** A cura
  cobre-a pela porta; o que não existe é uma fixtura em que a máscara alcance `t > 1`
  (o `Verb::Mask` não refina desde 13/09, logo nada move vértices durante um traço dela).
- ⚠️ **O `Falloff::weight` ganhou um segundo chamador da porta**, e a ordem das guardas
  nele é a de sempre (byte-idêntica).

## §8 — O que uma leitura rápida do diff entende ao contrário

1. **A cura não muda a forma da curva.** Para `t ∈ [0,1)` o caminho é o de sempre, e em
   `t = 1` exacto as duas leis já concordavam (`0^s = 0` para `s > 0`).
2. **`fora_da_pegada` não é arrumação.** Ela existe porque a lei estava escrita numa das
   duas cópias — a porta é a cura, não o efeito colateral dela.
3. **O gate do refino não foi «apertado»: ele passou a medir a cor.** Ele já media a
   máscara por valor; a metade da cor contava bytes.
4. **A régua incremental por gerações não é conforto.** A 1.ª redacção estourou
   (`len 86, índice 102`) porque o refino é iterativo — a lei da cadeia, do lado do refino.
5. **O `collapse_cor_tests.rs` não é um ficheiro «de sobra».** Ele é o corte que o tecto de
   LOC obrigou, e o assunto dele (os canais) não é o do irmão (a topologia).
6. **`pub(super)` nos três helpers é deliberado**, com a razão no comentário: partir um
   ficheiro de teste não deve partir a fixtura.
7. **A divergência com `hardness = 1` não é um defeito que ficou por curar.** Ela é a nossa
   escolha, num ramo que o alvo não consegue executar.

## §9 — Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=51 cargo run -p ph2d-host-desktop --profile smoke
```

O roteiro da cena já conduz; o passo **(4)** é o do report, e ganhou a linha do que
procurar: *o traço tem de sair inteiro na cor escolhida, sem uma face preta que seja.*
