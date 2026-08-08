# Handoff de integração — `line/Vector` · W7h, **a HIERARQUIA**

> **Data:** 2026-08-08 · **Branch:** `line/Vector` · **Wave:** W7h (o item aberto do W7)
> **Estado:** fechada, gates verdes, **PENDENTE DE SMOKE** e de ordem de integração do Enio.

---

## 1. O que esta wave entrega, numa frase

**O menu não fecha quando o cursor desce para um item dele.**

O §10 do handoff da W7c nomeou o que faltava: *"a HIERARQUIA (`Machine` é PLANA) — um menu que
abre com sub-estados é ela"*.

⚠️ **Não era imperfeição, era inusável:** o `point` mandava *o hospedeiro ANTERIOR* para o
`Default`, e um **ancestral** do novo alvo contava como anterior. Autorar um menu com itens
próprios produzia um menu que fechava no instante em que o cursor entrava num item.

---

## 2. A lei, numa frase — e os dois defeitos que ela fecha

> **O mais INTERNO ganha, e os ancestrais ficam acesos.**

### 2.1 `host_under` devolve uma CADEIA, não um hospedeiro

Do mais interno para fora, e `point` a recebe. ⚠️ **Esquecer os ancestrais passa a ser impossível
por TIPO** — não há como passar um id só.

⚠️ **A cadeia é ordenada por PERTENÇA**, e não por uma profundidade contada à parte: contar seria
uma segunda travessia da árvore, com a chance de discordar da primeira.

### 2.2 O segundo defeito, que ninguém tinha visto: o vencedor saía do `BTreeMap`

A versão antiga varria `states.hosts()` e parava no primeiro que contivesse o pick. Com dois
hospedeiros **aninhados**, o pick pertence aos dois ⇒ o vencedor era decidido por **qual
`VecPathId` era menor**.

⚠️ O doc dizia *"o primeiro que pertence a um hospedeiro ganha — o de cima é o que o artista
vê"*, o que é **verdade entre PICKS** (que vêm em ordem de Z) e **falso entre HOSPEDEIROS** para
um pick só. Gate: `the_innermost_host_wins_not_the_smaller_id`.

### 2.3 Só o mais interno responde ao APERTO

Um `Pressed` que subisse a cadeia acenderia o menu inteiro ao clicar num item. O ancestral segura
o `Hover`, que é o que ele de facto **é** — o cursor está dentro dele.

---

## 3. ⚠️ DUAS CAMADAS, e a primeira mutação NÃO sangrou

É o achado da wave, e ele mudou o que eu ia escrever.

O conserto tem duas metades: o **filtro de quem se deixa** (olhar a cadeia inteira em vez do
hospedeiro anterior) e o **`role_for`** (dar `Hover` a quem está na cadeia). Mutei a primeira e
**os gates de comportamento ficaram verdes** — porque o segundo laço do `point` re-pede o papel
do ancestral no mesmo quadro, e a pose visível acaba igual.

⇒ cada camada quer o seu gate [[feedback_layered_defenses_need_per_layer_gates]].

⚠️ **E a primeira camada É load-bearing — por CUSTO, não pela pose.** `Machine::go_to` constrói
uma `Transition` a cada chamada, e o doc da `ph2d-ui-state` mede o casamento em **0,64 ms por par
com geometria** (*"20 objetos numa troca só-de-cor pagariam 12,79 ms — 77% de um quadro"*). Pedir
`Default` e logo `Hover` constrói duas e joga a primeira fora.

O gate lê a **propriedade observável**: `an_ancestor_that_stays_lit_is_not_re_animated`.

---

## 4. ⚠️ Nenhum knob novo, de propósito

**O comportamento é a entrega.** Autorar um hospedeiro aninhado **já era alcançável** (`host` é
simplesmente a forma selecionada, sem restrição de aninhamento) e o readout *"Showing:"* já é
verdadeiro por construção — ele lê a mesma máquina que escreve o mundo, e a máquina do menu está
em `Hover`.

Inventar um controle aqui seria a **lei do knob-morto ao contrário**: um botão que não responde a
pergunta nenhuma do artista.

---

## 5. ⚠️ Uma nota que SOBREVIVEU AO FATO, corrigida

O doc da `ph2d-ui-state` dizia que os dois regimes onde a mola morde eram *"INALCANÇÁVEIS hoje,
porque o seletor de curva não existe"*. **A W7c construiu-o** (2026-08-08).

Pela regra do §0 — *quem move o número reconfere a nota* — o veredito **mudou de NATUREZA**: a
mola deixou de ser dispensável por *ausência de regime* e passou a ser **decisão de produto**.
`Cubic InOut` interrompido **para e recomeça**, e está a um clique.

⛔ **E o doc agora diz explicitamente para não a construir por conta própria:** uma mola não tem
*duração* nem *curva* (tem rigidez e amortecimento), então o slider de duração **e o próprio
seletor** deixariam de significar o que significam. Wave própria, e é do Enio.

---

## 6. A tabela de colisão

| Eixo | Valor | Nota |
|---|---|---|
| `PROJECT_SCHEMA` | **61**, intocado | esta wave não serializa nada |
| `VEC_SCENE_SCHEMA_VERSION` | **14**, intocado | |
| `FLIP_SCHEMA_VERSION` | **13**, intocado | |
| Registro do `ph2d-ecs` | **intocado** | |
| Contrato congelado | **intacto** | |
| ADR | **nenhum** | ⇒ fora de toda disputa de número |
| `Cargo.toml` | **zero** | nenhuma dep, nenhuma crate nova |
| Ids novos | **zero** | |
| Cena de smoke | **`=64`** | próximo livre: **65** |

### 6.1 O ponto de merge sensível

**`host_under` e `UiPreview::point` mudaram de assinatura** (`Option<VecPathId>` → cadeia), e o
mesmo vale para `App::ui_preview_host_at`. São **três** sítios, todos na shell, e o compilador
pega qualquer chamador que fique para trás — não há caminho silencioso aqui.

---

## 7. Gates e mutações

**5 gates novos** em `render_loop/ui_preview_tests.rs`:

| gate | o que ele afirma |
|---|---|
| `an_ancestor_stays_lit_while_the_cursor_is_in_its_descendant` | ⭐ a lei |
| `leaving_the_whole_tree_returns_every_ancestor_to_default` | a outra metade — um menu que nunca fecha é pior que um que fecha cedo |
| `the_innermost_host_wins_not_the_smaller_id` | §2.2 |
| `an_ancestor_that_stays_lit_is_not_re_animated` | §3, a camada do custo |
| *(a fixture `nested`)* | aninhamento ECS de verdade, não dois ids num mapa |

| # | Mutação | Sangra |
|---|---|---|
| M11 | *quem se deixa* volta a ser o hospedeiro anterior | **só** `an_ancestor_that_stays_lit_is_not_re_animated` — os de comportamento ficam verdes (§3) |
| M12 | `role_for` só responde ao mais interno | `an_ancestor_stays_lit_while_the_cursor_is_in_its_descendant` |

⚠️ **E a minha fixture nasceu sem o fenômeno:** `VecScene::push_path` **REESCREVE o id** (ele é
quem os cunha), então os ids que ela declarava nunca chegavam à cena — `members` devolvia vazio e
**o aninhamento não existia**. Quem pegou foi o gate que exigia o ancestral na cadeia; sem ele,
dois gates teriam passado sobre uma fixture plana [[reference_topic_fixture_discipline]].

---

## 8. O smoke

```
env PH2D_BUILD_SMOKE=64 cargo run -p ph2d-host-desktop --release
```

Um menu com dois itens, e os **três** são hospedeiros com Default/Hover gravados.

⚠️ **A cena imprime o número que a torna válida** — quantos hospedeiros autorou e quantos caminhos
o menu governa. Se disser menos de 3, ou se aparecer `!! a cena NAO contem o fenomeno`, **PARE**.

O passo que decide: **ligue a Preview, passe o cursor pelo fundo do menu (ele clareia) e desça
para um item.** O item acende e **o menu tem de continuar claro**. Se ele escurecer no instante
em que o cursor entra no item, é o defeito que esta wave existe para fechar.

---

## 9. Aberto, nomeado

- **A MOLA** — §5. Decisão de produto, com os números na mesa (W7c §6).
- **W8a** — ⛔ bloqueado por **ausência**: `ph2d-runtime` não existe. Não é adiamento, é
  pré-requisito.
- **O `Disabled` continua sem gatilho** — é um fato do DOCUMENTO, não do rato, e por isso não sai
  de `role_for`. (Herdado da W7r, não desta wave.)
- **Irmãos na mesma cadeia** mantêm uma ordem estável mas arbitrária. Não é observável hoje: o
  `point` acende a cadeia inteira em `Hover` menos o primeiro, e dois hospedeiros irmãos não se
  contêm. Se um dia a ordem entre irmãos importar, ela quer o Z — e um gate.
