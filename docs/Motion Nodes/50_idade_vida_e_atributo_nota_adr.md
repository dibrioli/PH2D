# 50 — **Idade, vida e o nó `value.attribute`** — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **O4** (parte 3 — fecha o sistema de partículas)
**Status:** implementado, testado (mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. A morte que faltava

A zona sabia matar **por LUGAR** (`motion.falloff` + `motion.cull`: saiu do círculo, morreu). Mas a morte de que
todo sistema de partículas vive é **por IDADE**: a faísca apaga, o floco derrete, a fumaça se dissipa — e nenhum
deles numa coordenada específica.

- **`sim.step` passa a criar a `age`.** Ele é dono do relógio da sim, então é dono do envelhecimento: a idade cresce
  pelo **mesmo `dt`** que o movimento. Nenhum outro nó poderia fazer isso honestamente (teria de **adivinhar o
  frame rate**).
- **`sim.lifetime`** mata quem passou da própria vida e escreve **`life`**: quão longe da morte cada sobrevivente
  está — **0 ao nascer, 1 no fim**.

**A `life` é o ponto, não um subproduto.** É o número que faz um sistema de partículas *parecer* um: colorir por
ela, encolher por ela, desvanecer por ela. Um nó que só matasse teria **jogado fora** essa informação, e todo
artista teria de reconstruí-la a partir de uma idade e de um parâmetro que ele não consegue ler.

**Variância por IDENTIDADE:** a vida de cada elemento é espalhada por um `hash(seed, id)` — sem isso, tudo que
nasce num tick morre num tick e a população **pisca** em vez de respirar. Hash, não sorteio: o scrub reproduz as
mesmas mortes.

## 2. `value.attribute` — a cola que faltava na biblioteca inteira

O stream carrega uma dúzia de colunas por-elemento (`age`, `life`, `id`, `size`, `vel`, `inv_mass`…) e **nada podia
lê-las de volta**. O `value.lfo` cunha um global; o `value.instance_field` cunha um campo a partir da **identidade**
(índice / rampa / hash) — e param aí. **Um número que o elemento JÁ CARREGA era inalcançável para o grafo de
valores.**

Ou seja: *"colore as faíscas pela idade delas"* — a frase mais ordinária do motion graphics — **não tinha caminho
nenhum** nesta biblioteca, com 84 nós.

Um nó resolve, e resolve para **toda coluna de uma vez** (é o *Named Attribute* do Blender, o `@age`/`@speed` do
Houdini). Por isso é um atributo **nomeado**, não um enum das colunas que existem hoje.

- **O nome é um param de TEXTO** — o canal do doc 32 (`set_text_param`/`text_param`), porque `NodeManifest.params`
  é `f32`-only e **congelado**. Nenhum contrato foi bumpado para pôr uma string dentro de um nó.
- **Modo `Length`**: lê a magnitude de uma coluna Vec2 — então `vel` lê como **speed**, que é o que o artista quer
  dizer ao pedir "velocidade".
- **Coluna inexistente = ZEROS, no comprimento certo.** Nem erro (um typo derrubaria o grafo inteiro), nem campo
  **vazio** — e o vazio é o pior dos três: um campo de comprimento 1 é **broadcast global** nesta biblioteca, então
  um `ag` em vez de `age` produziria um preto uniforme **que parece um grafo funcionando**.

## 3. Demo (doc de boot): a neve **envelhece**

`sim.step` → `sim.lifetime` → (`value.attribute("life")` → `motion.color_ramp.t`) → `falloff` → `cull`.

O floco **desvanece pela rampa Ice conforme envelhece** e morre de velhice no caminho; o desgarrado que sai do
círculo é ceifado na borda. **Duas mortes, e um sistema de partículas tem as duas.** É um **diamante** no grafo (a
rampa lê o mesmo stream duas vezes — uma como geometria, outra como valor) e o cook memoiza o ramo compartilhado:
custa **uma** avaliação, não duas.

Guarda de produto (no doc de boot real): velhos e recém-nascidos coexistem · `life ∈ [0,1]` · e **as cores se
espalham** — mutante: um `value.attribute` lendo coluna inexistente daria `t = 0` para todos e a nevasca inteira
sairia de **uma cor só**, parecendo perfeitamente saudável.

## 4. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| crates novas | **`ph2d-node-sim-lifetime`** (`sim.lifetime`) · **`ph2d-node-value-attribute`** (`value.attribute`) → **85 crates-nó** |
| coluna nova | **`age`** (escrita por `sim.step`) · **`life`** (escrita por `sim.lifetime`, 0→1) |
| text param | `value.attribute` usa a chave `attr` (canal do doc 32) |
| shell | a neve do doc de boot agora envelhece, colore por idade e morre de velhice |

## 5. A lição

**Uma biblioteca com 84 nós não conseguia dizer "colore pela idade".** Não faltava um nó de cor, nem de idade —
faltava a **leitura**: o caminho de volta da coluna para o grafo de valores. Vocabulário grande não é o mesmo que
vocabulário **fechado**, e o buraco só apareceu quando uma cena real tentou atravessá-lo.
