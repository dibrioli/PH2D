# HANDOFF DE CONTINUAÇÃO — `line/sculpt3d`

> **Para o próximo agente que assumir esta linha.** Rota: `/pd-linha-assumir`
> com este arquivo, e o **`cd` + `pwd` + `git branch --show-current` ANTES de
> abrir qualquer arquivo** (a janela abre na raiz, que é `main`, e o mesmo path
> relativo existe nas duas árvores).
>
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d`
> **Branch:** `line/sculpt3d`, **reaberta a partir do `main`** em `ee1432203`.
> Árvore **limpa**, zero commits próprios, gate do `main` herdado.

---

## §1 — O ESTADO: a W9 integrou, e a linha nasce com dois reports abertos

A **W9 «Mesh Filter»** fechou e está no `main` (os 9 tipos de filtro + o picker
+ as curas do smoke + os 6 achados da auditoria multiagentica). O roteador
(`CLAUDE.md` §5) já a dá por fechada.

⚠️ **Mas ela integrou com DUAS perguntas do Enio em aberto**, e uma delas eu
**diagnostiquei até à linha** nesta sessão sem ter contexto para a curar. **É por
onde você começa.**

---

## §2 — ⭐ TAREFA 1: o undo do filtro — DIAGNOSTICADO, por curar

### O report

Enio, 2026-08-18: ***"não temos undo para Filter"***.

### O que já foi descartado (não repita)

- A fiação **existe**: `sculpt3d_pointer_up` fecha o `Drag::Filter` pela porta do
  traço (`close_stroke`), o `close_stroke` grava `StrokeUndo::Stroke`, e o
  `sculpt3d_key` roteia `Ctrl+Z → undo_stroke` (a chamada está em
  `input_dispatch/keyboard.rs:78`, **multi-linha** — um `grep "sculpt3d_key("`
  sobre `src/*.rs` **não a acha**, porque aquele arquivo está num SUBDIRETÓRIO;
  eu caí nisso).
- Existe gate a prová-la: `the_whole_drag_is_one_undo_step`
  (`shells/desktop/src/sculpt3d_filter_tests.rs:249`) — e ele **passa**.
- ⛔ A guarda `sculpt3d_keys_live()` (que inclui `!a_tool_owns_the_bare_keys()`)
  **NÃO é o defeito** — eu suspeitei dela e **recuei**: se o Motion ou o Vector
  estão em mãos, o `Ctrl+Z` deve desfazer o grafo deles, não a escultura. Mover o
  acorde para antes dela seria uma **regressão**.

### ⭐ A CAUSA, achada por leitura e confirmada no fonte

`shells/desktop/src/sculpt3d_undo.rs:351-385`, o braço `StrokeUndo::Stroke`, é um
**`if/else`**:

```rust
if let Some(masks) = masks {
    swap_window(… .masks_mut(), &verts, &masks);        // ⚠️ só as MÁSCARAS
    // ⚠️ e NEM `rebuild()` NEM `mesh_rebuilt()`
} else {
    swap_window(… .positions_mut(), &verts, &positions); // as POSIÇÕES
    self.piece_mut().stack.mesh_mut().rebuild();
    self.mesh_rebuilt();
}
```

E quem decide qual ramo corre é o `close_stroke`
(`shells/desktop/src/sculpt3d_history.rs:~530`):

```rust
masks: self.brush.verb.paints_mask().then(|| self.stroke.base_masks().to_vec()),
```

⚠️ **Ele pergunta ao VERBO.** E `paints_mask()` é `matches!(self, Self::Mask)`
(`crates/ph2d-sculpt3d/src/brush_verb.rs:410`) — medido, não suposto.

⇒ **Com o `Verb::Mask` em mãos, um gesto de FILTRO grava a entrada como se fosse
um gesto de máscara.** O `Ctrl+Z` troca as máscaras, a **geometria fica onde
estava**, e — porque o ramo das máscaras não chama `rebuild()`/`mesh_rebuilt()` —
**a tela nem atualiza**. Do lado do artista isso é exactamente *«não tem undo»*.

### ⚠️ E é a TERCEIRA vez que o mesmo mecanismo morde

O **picker da W9b** desacoplou a LEI do VERBO em mãos, e todo sítio que ainda
infere *«que espécie de gesto foi este?»* a partir do verbo passou a estar
errado. Os três, em ordem:

1. o `fill_hc_disp` (o **panic** do Surface Smooth) — curado;
2. a mensagem de recusa do `begin_filter`, que ainda nomeia quatro verbos —
   ⚠️ **e o ramo que a imprime é INALCANÇÁVEL** (dentro de
   `if scene.filter_arm()`, o `begin_filter` só devolve `false` se
   `!filter_arm()`); código morto **e** obsoleto, `sculpt3d_input.rs:~113`;
3. **este.**

*Uma condição que enumera os seus leitores apodrece no dia em que o segundo
nasce* — e aqui o segundo nasceu uma wave antes, sem ninguém reconferir.

### A cura prescrita

A pergunta certa não é *«o verbo pinta máscara?»* e sim ***«este gesto mudou
máscaras?»*** — que é um **fato**, não uma inferência. Duas formas, e a segunda
é a que eu recomendo:

- **(a) barata e estreita:** o `close_stroke` chamado a partir de `Drag::Filter`
  nunca grava máscaras (um filtro só escreve posições — o `filter()` não toca o
  canal). Custa um parâmetro na porta.
- **(b) correta:** o `if/else` vira **duas trocas independentes** (o gesto pode
  ter mudado os dois canais), e o `rebuild()`/`mesh_rebuilt()` corre sempre que
  as POSIÇÕES mudarem. Isto também cura um defeito irmão que ninguém reportou: um
  gesto que mude máscara **e** geometria hoje desfaz metade.

⚠️ **O gate que falta é o que NENHUM teste desta família tem: a rota do
PONTEIRO.** Todos chamam `arm_filter`/`begin_filter`/`filter_at`/`close_stroke`
**direto**; o produto passa por `sculpt3d_pointer_down → move → up`. Escreva-o
red-first com o `Verb::Mask` em mãos — é a fixture que contém o fenómeno, e é
por isso que ele escapou.

⚠️ **E o CONTROLE:** o mesmo gesto com um verbo que **não** pinta máscara tem de
continuar a desfazer a geometria, byte a byte, como hoje.

---

## §3 — ⏸️ TAREFA 2: as duas decisões do Enio (NÃO são trabalho)

Já devolvidas com a tabela. **Não construa nada aqui sem ordem dele.**

**(a) O Sharpen é o afiador que se quer?** O porte é **fiel** (conferido termo a
termo contra o `sculpt_filter_mesh.cc` três vezes). A lei, como a referência a
escreve, **alisa detalhe fino** (degrau `0,528× → 0,279×`) e **mal toca feição
grande** (`0,990× → 1,398×` de força 1 a 64). ⚠️ **Duas hipóteses minhas foram
REFUTADAS por medição** e estão registadas para ninguém as repetir: o
`sharpen_intensify_detail_strength` compra **2%** a oito vezes o default dele
(doc do const `INTENSIFY`), e a teoria de que ela afiaria detalhe fino saiu **ao
contrário**.

**(b) O teto (`SHARPEN_MAX = 4,0`).** Subir compra excursão real (a lei **não**
satura) e **paga em milissegundos por evento de ponteiro**:

| fatias | 1 | 4 | **8 (o teto)** | 16 | 32 |
|---|---|---|---|---|---|
| tempo | 7,72 ms | 11,47 | **17,17** | 27,93 | 49,16 |

⚠️ **Um quadro de 60 fps são 16,7 ms** ⇒ o teto de hoje já está no limite. A cura
do outro lado — cortar o custo por fatia — é **wave própria**, e é o item de perf
mais importante deste módulo.

---

## §4 — A FILA, depois da tarefa 1

Do [plano 21](../21_plano_modos_e_ferramentas.md) §7 (a tabela é a fonte; **não
confie nesta lista sem a reconferir** — seis células desta conferência já
envelheceram antes de alguém voltar a elas):

| wave | conteúdo | nota |
|---|---|---|
| **W11** | **Handles** — Pose · Boundary · Nudge · **Thumb** | ⚠️ o `Thumb` é um verbo **distinto** do `Clay Thumb` já feito |
| **W10** | **Cloth** (XPBD) + **Cloth Filter** (5 tipos) | pesada |
| **W12** | a **GEODÉSICA** — Heat Method na pegada → `l-mode` de falloff para a família inteira | |
| — | o **marching cubes** | o único item que sobra da lista aberta de 02/08 |

⚠️ **A primeira coisa de toda wave deste plano é MEDIR se a composição já
exprime o item** — o Elastic Deform foi respondido **sem verbo novo** exactamente
assim, e o alvo caiu de 14 para 13 pincéis.

---

## §5 — O que a auditoria multiagentica NÃO alcançou

Ela correu em 18/08 (8 lentes → 8 céticos; **58 candidatos, 8 verificados, 8
sobreviveram, todos curados**). ⚠️ **50 achados NÃO foram verificados** — cap de
8 por severidade. Eles estão no journal do run **`wf_76127d6f-aa1`**
(`~/.claude/projects/…/subagents/workflows/`), e **ninguém os leu**.

Os buracos que ela própria nomeou:

- o **custo por evento de ponteiro** (o filtro percorre a malha inteira por
  evento) — é o §3(b);
- **nada de GPU** foi exercitado (o filtro invalida a malha toda quadro; o custo
  de re-upload não foi medido);
- **multires, simetria e máscara em composição** com o filtro;
- os **rótulos i18n** dos 9 chips (li `k.label()`, não os tokens);
- e a **fidelidade das citações `sculpt_filter_mesh.cc:NNNN`** espalhadas pelos
  doc-comments — duas foram conferidas, **as dezenas restantes não**, e o modo de
  falha já se materializou uma vez.

---

## §6 — ⚠️ LIÇÕES DE PROCESSO desta sessão (leia antes de repetir)

1. ⛔ **NÃO rode uma auditoria multiagentica na árvore em que você está a
   trabalhar.** Os agentes céticos **mutam arquivos, rodam testes e restauram** —
   uma medição minha deu um resultado **aritmeticamente impossível** porque eu li
   a mutação de outro agente, e o meu trabalho em curso foi **apagado** por um
   restore. Use `isolation: "worktree"`, ou termine antes de mexer.
2. **`0 passed` não é verde, é *nada rodou*.** `cargo test --workspace <nome>`
   devolveu isso nos quatro arch-gates; rode **por target** e confira a contagem.
3. **Um `grep` sobre `src/*.rs` não é recursivo** — o `input_dispatch/keyboard.rs`
   escapou e eu quase reportei um chamador inexistente. Toda busca negativa
   precisa de **controle positivo**.
4. **Um oráculo escrito à mão a partir do mesmo fonte herda a mesma leitura
   errada.** O `-abs` do EnhanceDetails escapou porque a sonda de paridade da
   wave **também** o omitia — *ela concordava com o produto em toda parte*.
5. **Uma comparação entre duas malhas diferentes não mede o que as separa**, mede
   que elas são diferentes (o `zip` trunca em silêncio). Um gate meu sobreviveu a
   uma mutação por isso.

---

## §7 — Os smokes que existem hoje

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=34 cargo run -p ph2d-host-desktop --release
```

A **`=34`** é a do filtro (a W9 inteira) e imprime o roteiro de 6 passos —
⚠️ **se a lista não aparecer, PARE**. As cenas **`=1..=33`** têm de continuar
iguais. ⚠️ **Próxima cena livre: `35`**, e o número se **CONTA** lendo os arquivos
`sculpt3d_scenes*.rs` (há gate: `no_two_sculpt3d_scenes_claim_the_same_level`),
**nunca uma nota** — a desta §5 já envelheceu uma vez.

⚠️ **Rode a suíte do módulo também em DEBUG** (precedente registado: o
`ph2d-flip-colorize` panicava só ali).
