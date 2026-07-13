# 08 — Edit Mode: a seleção de traço (W6)

> **O que é:** o "select do traço" — clicar um traço, vê-lo realçado, e ter o Sculpt e o
> painel agindo **só nele**. É o Edit Mode do Grease Pencil, e ele aposenta o "alvo vivo"
> (`flip_live.rs`, o alvo provisório que era *"a última coisa que você fez"*).
>
> Referência: [`02_referencia §11`](02_referencia_algoritmos_blender_5.2.md) (o atributo
> `.selection` do GP). Landou 2026-07-13.

---

## §1 — As duas decisões que vieram antes do código

### 1.1 — A seleção é um ATRIBUTO do traço, não uma lista de índices

`FlipStroke.selected: bool` — o atributo `.selection` do GP, no domínio **Curve**.

A tentação é guardar `Vec<usize>` no shell. Ela morre em um parágrafo:

> A identidade de um traço é a **posição dele na `Vec`**. E o **balde insere no meio da
> lista** (`flip_fill::fill_click` → `strokes.insert(at, …)`: a cor entra POR BAIXO do
> line-art), a borracha **reescreve a lista inteira** (ela corta traços em pedaços), e o
> undo **a restaura**. Contra qualquer uma das três, um índice guardado apodrece — **em
> silêncio**. O sintoma não seria um crash: seria o painel recolorindo o traço errado.

O atributo **viaja com o traço**. Gate: `the_selection_survives_the_bucket_inserting_a_stroke_beneath_it`.

**Custo:** `FLIP_SCHEMA_VERSION` 3→4 e, porque o `FlipDoc` vive dentro do `ProjectState`,
`PROJECT_SCHEMA` 7→8. (Postcard é posicional e **não avisa** — o gate do par pegou.)

**Corolário — os três pontos de estrangulamento.** Todo campo novo do `FlipStroke` tem de
passar por `FlipStroke::clone_attrs`, `flip_erase::new_like` e o `cleanup_soft`. A W4
esqueceu o segundo e a borracha passou a devolver fills sem furo. Aqui: o `new_like`
**carrega** o `selected` (cortar um traço não o desmarca — os pedaços continuam
selecionados), e o `clone_attrs` **não** (o tween produz um traço DERIVADO; herdar o
realce pintaria os quadros gerados de selecionado).

### 1.2 — Edit é um MODO próprio, não uma sobrecarga do Select

O `Select` é a arbitragem do [ADR-0112](../architecture/decisions/0112-vector-select-node-pen-are-three-tools.md):
ali quem manda é o **gizmo**, que move/gira/escala o objeto Flip inteiro. Se o clique do
Select passasse a pegar traço, o usuário **perderia o gizmo**.

É a mesma separação **Object Mode × Edit Mode** do Grease Pencil. E sai de graça: o shell
só publica `GizmoView` quando `mode == Select`, então o Edit já nasce sem gizmo.

---

## §2 — O que a seleção FAZ

Uma seleção que não faz nada é enfeite. Ela tem quatro consumidores:

| Consumidor | Comportamento |
|---|---|
| **Sculpt** (`Session::begin`) | havendo seleção, o gesto toca **só os selecionados**. **Seleção vazia = tudo** — é essa metade da regra que torna a feature **aditiva**: quem nunca abriu o Edit Mode não vê diferença |
| **Painel** (Size / Hardness / Opacity / Color) | reescrevem os traços selecionados — é o que aposenta o alvo vivo |
| **Delete** (tecla + botão) | apaga os traços selecionados |
| **Realce** (overlay Vello) | o contorno âmbar sobre cada traço selecionado |

**O Sculpt mascarado é a promessa central:** num desenho cheio, alisar uma linha não pode
alisar o que passa perto dela.

**O Delete CONSOME a tecla.** Sem o `return`, o caminho genérico apagaria também o
**objeto Flip inteiro** (ele continua selecionado como entidade): uma tecla, dois efeitos,
e o segundo é catastrófico. O bloco vetorial consome pelo mesmo motivo.

---

## §3 — As duas armadilhas do painel-sobre-a-seleção

São as que destroem arte, e por isso são as que têm gate.

### 3.1 — SÓ A MUDANÇA age

Se o passe reaplicasse o estilo do painel a cada frame, um traço **vermelho** selecionado,
com o painel em **azul**, viraria azul **no ato do clique**. O usuário perderia a arte *só
por olhar para ela*.

O passe guarda o estilo do **frame anterior** (`App.flip_edit_style`) e escreve **só os
campos que mudaram**. A 1ª volta com seleção apenas memoriza.
Gate: `selecting_a_stroke_does_not_repaint_it_with_the_panel_colour`.

### 3.2 — ZERO geometria

O passe **nunca** reescreve posições a partir de uma cópia "pristina" do traço. Se o
fizesse, esculpir um traço e **depois** mexer no slider de cor **desfaria a escultura** —
o slider de cor apagaria o trabalho do Sculpt.

A espessura é o único campo com forma, e ela vira **`size × perfil`**, com o perfil
(`w_i / w_max`) lido do traço **agora**. Escalar preserva razões ⇒ o resultado é
**idempotente**, o slider é **reversível**, e o desenho de pressão da caneta sobrevive —
que um `w_i := size` chapado destruiria em silêncio.
Gate: `resizing_preserves_the_pressure_profile_and_is_reversible`.

**Sem Smoothing no Edit.** O alisamento é uma op de GEOMETRIA sobre as **amostras cruas** da
caneta, que um traço já desenhado não guarda. Um slider que não pode agir é o controle
morto que a doutrina modal do painel proíbe.

---

## §4 — O pick

- **O traço de CIMA primeiro** (a ordem da lista é a ordem de z — a varredura é `.rev()`).
- **Uma REGIÃO pega pelo INTERIOR.** Ela não tem linha (`hide_stroke`): exigir proximidade
  da borda tornaria a cor do balde inselecionável. O **buraco** não pega — clicar no furo
  de um "O" é clicar no que está atrás dele.
- **O raio de pick acompanha o ZOOM** (`px_to_world × mean_scale`): a espessura é absoluta
  em px de TELA e a geometria é documento. Aproximar a câmera não pode exigir mira mais
  fina. (É a MESMA armadilha de unidade que matou o balde três vezes — BUGS #11/#14/#16.)
- **Selecionar NUNCA cria um quadro.** O resto dos gestos passa pelo
  `flip_autokey::target_drawing`, que materializa a chave. Selecionar lê o que está na
  tela: por isso o `flip_select::visible_drawing` recebe um **`&FlipDoc` imutável** — a
  proibição é do TIPO, não de um comentário que a próxima wave esquece.

Gestos: **clique** substitui · **Shift+clique** alterna · **clique no vazio** desmarca (com
Shift, o vazio não faz nada: um shift-clique que errou o traço por 2 px não pode apagar a
seleção que o usuário levou meia dúzia de cliques para montar).

---

## §5 — A lição de GATE que esta wave rendeu

> **Os gates do painel enumeravam os modos À MÃO — e o modo novo passou por eles sem um
> pio**, apesar de mostrar controles que dois deles afirmavam não existir fora dos seus
> modos.

Um gate que não OBSERVA não dispara. Agora existe **`FlipMode::ALL`**, e os dois gates
(`each_mode_shows_only_its_own_attributes` e `size_is_shared_by_…`) **afirmam que a tabela
deles cobre `ALL` inteira**. O próximo modo quebra o teste **no dia em que nascer** — que é
o único momento em que arrumar é barato.

---

## §5b — Os gestos (W6.1)

| Gesto | O que faz |
|---|---|
| arrastar no **vazio** | **marquee**: pega tudo que a caixa TOCA (Shift soma) |
| arrastar um **traço** | **move a seleção inteira** — e se o traço não estava selecionado, ele é selecionado primeiro e vai junto |
| Shift+arrasto num traço | alterna (o arrasto não pega: o gesto seria ambíguo) |

**O marquee não é "algum ponto dentro".** Uma reta longa pode **atravessar** a caixa sem
ter um único vértice nela — e quem desenhou a caixa em cima dela espera pegá-la. O teste é
**ponto-dentro OU segmento-cruza** (orientação; transcendental-free, HR-5).

**Mover translada os pontos E OS BURACOS.** Um preenchimento carrega os furos em anéis
próprios (o "O"): mover só os pontos deixaria os furos para trás e quebraria a forma — e
isso só apareceria no desenho do usuário meses depois. É a MESMA regra que o Sculpt já
obedece (a lição do Suzanne: *a cor anda com a linha*).

**Arrastar um traço não-selecionado já o seleciona e o move** (Illustrator, Blender Edit
Mode). Exigir clicar-soltar-clicar-de-novo é a ergonomia que faz o usuário concluir que a
ferramenta não responde.

A caixa é **desenhada** (overlay, px de tela) — um marquee invisível é um arrasto que não
faz nada. E um **slop de 3 px** separa o clique trêmulo do gesto: sem ele, um clique no
vazio desenharia uma caixa de 2 px e a seleção piscaria.

---

## §6 — Aberto (declarado, não esquecido)

- **Transformar a seleção (girar / escalar)** — mover já existe (W6.1). O caminho do PH2D
  é o **gizmo de sprite** (ADR-0111: quem move/gira/escala é ele), mas o gizmo é
  **por-entidade** — ele escreve num `Transform` do ECS, e um traço não é entidade. Fazê-lo
  agir sobre a seleção exige um consumidor NOVO: bbox da seleção → `GizmoView` → o delta é
  **assado nos pontos**. É um pacote próprio, e é o próximo passo natural deste modo.
- **Domínio Point** (meio-traço selecionado) — o `02 §11` o especifica (vetor paralelo aos
  pontos + conversão de domínio explícita). É o que destrava o *segment mode* e o
  auto-masking por PONTO no Sculpt.
- **O painel não espelha a seleção** — selecionar um traço vermelho não move o swatch do
  painel para vermelho (os controles são "aplique este valor", não um espelho). O
  write-back exigiria o shell empurrar valores para dentro da tool; é um pacote próprio.
- **Transformar a seleção** (mover/girar/escalar os traços selecionados) — o `02 §11` diz
  que no GP isso é uma op comum consumindo a mesma lista, não um transform de GP.
