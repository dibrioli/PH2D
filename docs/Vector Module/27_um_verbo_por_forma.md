# UM VERBO POR FORMA — a booleana viva ganha uma receita

> `line/Vector`, 2026-08-22. **Desenho do Enio**, implementado no mesmo dia em que o
> [grafo foi retirado](26_plano_grafo_booleano_vivo.md).
>
> É a mesma capacidade que o grafo perseguia — *"somo com esta, subtraio aquela"* — por um caminho
> que não precisa de janela, de gesto, nem de posições guardadas.

## 1. O pedido

> *"O usuário escolhe um shape base e coloca todas as outras como filhas. O modo do boolean é
> escolhido por shape e na ordem em que aparece na hierarquia atua sobre o resultante das operações
> pregressas. […] Quem aparece por último na hierarquia sempre atua sobre o resultante dos
> anteriores."* (Enio, 2026-08-22)

## 2. A lei

> **As formas de um grupo booleano combinam-se na ordem da hierarquia, e cada uma traz o verbo com
> que dobra sobre o resultado das anteriores.**

`((base op₀ p₀) op₁ p₁) op₂ p₂ …` — a dobra à esquerda que o motor **já fazia**, com o verbo a
variar em vez de ser um só.

⚠️ **É por isso que a mudança é pequena.** O `apply_many_checked` sempre foi um fold binário da
esquerda para a direita; o que estava fixo era o verbo. A generalização é
[`apply_chain_checked`](../../crates/ph2d-vec-boolean/src/lib.rs), e a porta N-ária passou a
delegar nela — **um corpo de fold só**, porque dois divergiriam no dia em que alguém corrigisse a
regra de preenchimento num deles.

### 2.1 A ordem já era a da hierarquia

O que fechou o desenho foi uma medição, não uma opinião: **nesta app a lista da hierarquia É a
pilha de z** (a lei de Godot — o pai desenha antes dos filhos, e a pilha é o DFS *na ordem*). Logo:

- *"quem está mais abaixo atua sobre o resultante"* = *"quem vem depois em z dobra sobre o
  acumulado"* — que é **literalmente** a ordem em que o `bool_live` já colhia os operandos;
- *"a base é o pai"* sai de graça: um pai é o **primeiro item do próprio ramo**, e o primeiro item
  é a base.

A intuição estrutural do pedido coincidia com o que a projeção de z já fazia. Nada teve de ser
inventado para a acomodar.

## 3. O padrão-ouro

| referência | como faz | adotámos? |
|---|---|---|
| **Illustrator — compound shape vivo** | cada componente guarda o seu *Shape Mode* (Add · Subtract · Intersect · Exclude), resolvido de baixo para cima; o modo do componente de baixo é **inerte** | **sim** — é este |
| **Blender — pilha de modificadores booleanos** | um verbo por cortador, aplicados em ordem de pilha, sobre o objeto-base | sim, mesma forma |
| **Figma — boolean group** | a operação é do **GRUPO**; para misturar, aninha-se | ⛔ não — é a limitação que isto remove |

## 4. As três decisões, e o que cada uma evita

1. ⛔ **Só as QUATRO operações de conjunto cabem numa forma.** `MinusBack`/`Trim`/`Crop`/`Merge`
   são afirmações sobre a **pilha inteira** (*"cada forma menos a união do que está acima dela"*
   não é uma relação entre duas). Elas continuam a ser verbos do grupo, e com uma delas em vigor a
   fileira por forma **não é oferecida** — há gate dos dois lados (o cozimento ignora, a UI não
   oferece).
2. **Ausência é HERANÇA, não *"sem verbo"*.** Sem o componente, a forma dobra com o `op` do grupo.
   Isto compra duas coisas: todo documento anterior desenha **byte-idêntico**, e os oito botões do
   grupo **não morrem** — eles passam a ser o *padrão* de quem não se pronunciou. Sem esta escolha
   o seletor do grupo ficaria inerte, que é o defeito *"parâmetro que não muda nada"*.
3. **O verbo aparece na LINHA da hierarquia**, não só no painel lateral. É a metade que faz o
   desenho funcionar: com o verbo só no inspector, entender uma booleana de cinco formas custa
   cinco cliques e memória — que é exactamente a queixa que matou o grafo. A hierarquia já mostra a
   **ordem**; o selo acrescenta o **verbo**; ordem + verbo **são** a receita.
   Selos: `UNI` · `SUB` · `INT` · `EXC` · `BSE` (a base, que não tem verbo) · `RCP` (o grupo está
   numa receita).

### 4.1 O verbo SOBREVIVE ao Ungroup — e isso é decisão

O **Apply** consolida e mata o grupo inteiro com os descendentes, então nada fica para trás. O
**Ungroup** não: as formas saem soltas e **continuam a carregar o verbo**, inerte.

⚠️ Parece a armadilha clássica (*estado invisível que reaparece meses depois*), e seria — se não
fosse o selo. **No instante em que aquela forma volta a ser operando de uma booleana viva, a linha
dela diz `SUB`.** O verbo nunca está em vigor sem estar escrito, e é por isso que apagá-lo no
Ungroup seria destruir a escolha do artista para resolver um problema que o selo já resolve.

## 5. Onde eu discordei do pedido, e o que ficou

O pedido dizia que **mesmo através de vários níveis** o mais abaixo atua sobre o resultante de
todos — ou seja, aninhar não criaria sub-resultado, só contribuiria para uma fila única.

⛔ **Isso achata a árvore e apaga os parênteses.** Alguns casos não sofrem — `X − A − B` já dá o
mesmo que `X − (A ∪ B)`, porque subtrair em cadeia é subtrair a união. Mas isto deixa de ser
exprimível:

> **X ∩ (A ∪ B)** — *"o que X tem em comum com a junção de A e B"*.

Numa fila achatada não há onde pôr o parêntese, e `(X ∩ A) ∪ B` é outra coisa.

**O que ficou:** a regra do pedido vale **dentro de cada nível**, e aninhar continua a significar
*"isto conta como uma forma só"* — o parêntese. O modelo mental do Enio não muda (em cada nível a
leitura é a que ele descreveu) e a expressividade que já existia não se perde.

## 6. O custo

⚠️ **Não há custo novo, e isto não é estimativa.** A cadeia com verbo variável faz **exatamente o
mesmo número de operações binárias** que a cadeia de verbo fixo — é o mesmo fold. Os números de
[26 §4](26_plano_grafo_booleano_vivo.md) continuam a valer: uma booleana de 10 formas custa
**1,9 ms** contra os **16,6 ms** de um quadro a 60 fps, e o memo só paga quando a entrada muda.

## 7. A prova

- **Motor** — 5 gates em
  [`the_chain_folds_with_a_verb_per_step.rs`](../../crates/ph2d-vec-boolean/tests/the_chain_folds_with_a_verb_per_step.rs):
  o verbo uniforme é a booleana de sempre · verbos diferentes desenham diferente · a dobra é sobre
  o **acumulado** e não sobre a base · a ordem decide · a base sozinha não é operação.
- **Cozimento** — 8 gates em `bool_live_tests.rs`, em pares CAPACIDADE/HERANÇA.
- **Triagem** — 8 gates em
  [`vec_bool_shape_tests.rs`](../../shells/desktop/src/vec_bool_shape_tests.rs): quem recebe o
  seletor e quem não recebe.
- **Fixture do smoke** — a barra do trio tem de morder as duas irmãs, senão o passo 3c pediria ao
  Enio que visse uma diferença que a geometria não produz.

### 7.1 ⚠️ Três harnesses de mutação MENTIRAM antes de um resultado valer

Vale mais que os gates, porque se repete:

| o que mentiu | por quê | o sinal |
|---|---|---|
| restaurar com `shutil.copy2` | repõe o **mtime original**, o cargo salta a reconstrução e a mutação **sobrevive** nas corridas seguintes | 7 mutantes com sangramento **idêntico** |
| filtro `bool_live_tests` | o módulo chama-se `bool_live::tests` — **zero** gates correram | 4 mutantes "sobreviveram" de uma vez |
| `finally` sozinho | **não corre em SIGTERM**, e um timeout mata por SIGTERM — a árvore ficou mutada | o `grep` de conferência, por sorte |

**As três curas, e elas são obrigatórias:** restaurar por `write_text` · um **controlo positivo**
que exige um mínimo de testes de facto executados · um **handler de sinal** que restaura. Um
harness que não prova que rodou gates não mede mutação — mede a própria linha de comando.

## 8. ⚠️ O DEFEITO QUE ESCAPOU — e o gate que não existia

**Report (Enio, 2026-08-22):** *"Os botões não funcionaram."*

### 8.1 A causa

Eu escrevi a regra de elegibilidade como ***"exactamente UMA forma selecionada"*** — e **nenhum
clique deste editor pode satisfazê-la**. Tocar um filho **seleciona o grupo inteiro**; é lei
deliberada, e está escrita no `input_dispatch`:

> *"Tocar um filho seleciona o GRUPO (a árvore é a Hierarquia)."*

⇒ a fileira **nunca aparecia**. Os quatro chips não "deixaram de responder": eles não estavam na
tela. ⚠️ *Um controlo que nunca é pintado e um controlo que ignora o clique são indistinguíveis do
lado de fora* — e foi por isso que o report não bastou para localizar a causa.

### 8.2 A cura, e por que ela já estava lá

O sujeito é o **PRIMÁRIO**, não "a seleção". `set_object_selection` **preserva o primário** quando
ele está na lista (`ph2d-vec-edit/selection.rs`), então depois do clique `selected()` continua a
ser *a forma que o dedo apontou*, com os irmãos todos acesos à volta. A porta existia desde sempre;
eu é que perguntei outra coisa.

E como o grupo inteiro fica aceso no canvas, a fileira **nomeia o próprio sujeito** (o `Name` da
forma) em vez de dizer um genérico: sem isso o artista escolheria o verbo sem saber de qual das
formas o painel fala.

### 8.3 ⚠️ E havia um SEGUNDO defeito, independente — o report veio duas vezes

A cura de 8.2 fez a fileira **aparecer**. O Enio voltou: *"os botões não funcionam!"*

Os quatro chips estavam **pintados, hit-indexados e MORTOS sob o ponteiro** — faltavam no
[`populate_ops.rs`](../../crates/ph2d-panel-vector/src/populate_ops.rs). Sem `InteractiveState` no
store, `is_focusable()` é `false`: o Down nunca torna o chip ativo, o Up nunca emite `Click`, e
nada mais na suíte afirma essa paridade.

⚠️ **É por isso que os meus dois gates anteriores não o viram:** ambos usam `Click` **sintético** —
um prova a allowlist, o outro o mapeamento `id ⟶ código`, e os dois **pulam a checagem de
focabilidade**. *Só o gesto real (Down+Up sobre o retângulo que o painel pintou) mede esta
costura*, e o arnês para isso já existia (`ph2d-ui-testkit` + `tests/seam_bool.rs`) — eu é que não
o usei.

⛔ **Do lado de fora, os dois defeitos são o mesmo sintoma.** Um controlo nunca pintado e um
controlo morto sob o dedo produzem exactamente o mesmo report — e é por isso que *"não funciona"*
não localiza causa nenhuma, por mais vezes que se repita.

**A conta desta classe neste repo:** as 36 células da matriz de física · os 10 chips do Painter ·
as 4 pills PENCIL/SHAPE/SELECT/DIRECT (CI-verde-e-mortas por sessões) · e agora estes 4. O gate
novo [`bool_registration_parity.rs`](../../crates/ph2d-panel-vector/tests/bool_registration_parity.rs)
apanha **o quinto**, no dia em que alguém o acrescentar — que é a diferença entre consertar e
prevenir.

### 8.4 A lição, que é maior que o bug

⚠️ **Eu gatei o modelo, o cozimento e a triagem — e ZERO gates no caminho que o clique percorre.**
É a primeira das quatro causas da `DIRETIVA_IMPLEMENTACAO` (*costura não-testada*), e a razão de ela
ter passado é estrutural: o mapeamento `id ⟶ código` vivia **dentro do `render_loop`**, que exige
janela e GPU e portanto não é alcançável por teste nenhum.

A cura da causa (não do sintoma) foi **mudar o código de sítio**: `vec_bool_shape::shape_op_for_id`
é agora uma porta pura, irmã do `layout_edit_for_id` e do `classify_click`, com gate — e o outro
lado da costura (o id atravessar o barramento do painel) ganhou o gate dele **dentro da crate onde
o predicado vive**, e não num teste que precisasse de tornar o predicado público.

⛔ *Uma triagem cuja pré-condição nenhum gesto do produto consegue produzir passa em todos os
testes de unidade e falha em 100% dos smokes.* O gate que a apanha tem de encenar a seleção que o
CANVAS de facto produz — não a que o autor da regra imaginou.

## 9. O que mudou, em código

`ph2d_vec_boolean::apply_chain_checked` (e a porta N-ária a delegar nela) · o componente
`ph2d_ecs::VecBoolOp` (registo 58 → 59, `PROJECT_SCHEMA` 86 → **87**) · o cozimento e o memo do
`bool_live` (os verbos entram na **chave**, senão o clique não faz nada na tela) ·
`vec_bool_shape` (a porta única do *papel*, partilhada pelo painel e pela hierarquia) · a fileira
*This Shape* no painel · o selo na linha da hierarquia · a cena `PH2D_BUILD_SMOKE=48`, cujo rig 1
passou a **trio**.

---

## ⛔ Recusas MEDIDAS

| O quê | Por quê | Onde |
|---|---|---|
| Achatar o aninhamento numa fila única | Apaga os parênteses: `X ∩ (A ∪ B)` deixa de ser exprimível | §5 |
| Receita (`Trim`/`Crop`/`Merge`/`MinusBack`) por forma | É afirmação sobre a pilha inteira, não relação entre duas | §4.1 |
| Ausência do componente == *"sem verbo"* | Mataria os oito botões do grupo e mudaria todo documento já salvo | §4.2 |
| O verbo só no painel lateral | Cinco formas = cinco cliques e memória — a queixa que matou o grafo | §4.3 |
| Um seletor por forma sobre um grupo em receita | Controlo inerte pintado como vivo | §4.1 |
| Oferecer o seletor na linha da BASE | Ela não dobra sobre nada: o verbo dela é inerte por construção | §4.3 |
| Apagar o verbo no Ungroup | O selo já o torna legível no instante em que volta a valer; apagar destruiria a escolha do artista | §4.1 |
| Dois folds separados (uniforme e por-passo) | Divergiriam na regra de preenchimento | §2 |
| `shutil.copy2` para restaurar mutação | Repõe o mtime e a mutação sobrevive à corrida seguinte | §7.1 |
| Exigir *"exactamente uma forma selecionada"* | Nenhum clique a satisfaz — tocar um filho seleciona o GRUPO; a fileira nunca aparecia | §8.1 |
| Rótulo genérico na fileira | Com o grupo todo aceso, não diz de QUAL forma fala | §8.2 |
| Chip pintado sem `button(store, id)` no `populate_ops` | Morto sob o ponteiro, com a suíte verde — a 4.ª vez desta classe no repo | §8.3 |
| Provar a costura do clique com `Click` SINTÉTICO | Pula a checagem de focabilidade: passa com o chip morto | §8.3 |
| O mapeamento `id -> código` dentro do `render_loop` | Não é alcançável por teste (exige janela e GPU) — foi a causa-raiz de o defeito shipar | §8.3 |
