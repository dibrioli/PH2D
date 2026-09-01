# As variações saem do NOME e viram DADO — plano

> **Ordem do Enio, 2026-09-01:** *«nós realmente não conseguimos nos entender e precisamos mudar o
> modo de criar Variações. Não vamos mais usar as chaves no nome. Vamos usar o Card com botões
> específicos para cada função. Ao criar e modificar uma instância surge no card um botão do tipo
> "Salvar Variação". Daí o fluxo acontece da forma mais inteligente possível, com o momento de
> colocar o nome que vai gerar o botão seletor da variação.»*
>
> ⚠️ **A decisão anterior — *«o nome é a única verdade»* — é REVOGADA por esta.** Ela nasceu de uma
> pergunta dele (*«por que não funciona mudando o nome entre as chaves?»*) e custou seis reports com
> foto. Não a reconstrua sem ler o §1.

---

## §1 — Pesquisa: quem faz isto, como, e o que foi ABANDONADO

| Produto | Modelo | O momento de nomear | Nível |
|---|---|---|---|
| **Figma** | *Component set* + **propriedades** (`Property` → `Value`) guardadas como DADO no painel | *Create component set*, depois nomear a propriedade | **Dois** (propriedade × valor) |
| **Adobe XD** (descontinuado 2023) | *Component States* — lista **plana** de estados nomeados | `+` no painel de estados → escreve o nome → o estado passa a ser selecionável | **Um** (só o nome) |
| **Unity** | *Prefab Variant* — derivação, **sem** propriedades | o diálogo de gravar o asset (o nome do ficheiro) | zero (só a árvore de derivação) |
| **Unreal** | Blueprint filho / data-only BP | criar o asset | zero |
| **Blender** | *Library Override* de uma coleção; catálogos no Asset Browser | criar o override / o catálogo | zero |
| **Rive** | máquina de estados guia a arte; não há eixo de variante | — | zero |
| **After Effects** | *Master Properties* numa pré-comp = **overrides**, não versões | — | zero |

**O que foi tentado e abandonado — as duas lições que decidem este desenho:**

1. ⛔ **O nome como fonte foi tentado e demovido.** O Figma reconhece a sintaxe `Propriedade=Valor`
   no nome da camada, mas **só para SEMEAR** um conjunto: a fonte autoritativa é a tabela de
   propriedades do painel. A razão é a que nós pagámos em 2026-08-31: renomear uma camada passa a
   ser uma operação **estrutural**, e um nome que não obedece à gramática produz propriedades
   fantasma sem erro nenhum. *Nós construímos exactamente a versão que a indústria demoveu.*
2. ⚠️ **A lista PLANA do XD não compõe.** Estados nomeados (`Hover`, `Pressed`) são o gesto mais
   simples e é literalmente o que o Enio descreve — mas duas perguntas independentes
   (tamanho × estado) viram `n × m` nomes soltos, e o artista fica a manter a matriz à mão. É por
   isso que o Figma tem **dois** níveis, e é por isso que o Enio pediu, a 2026-08-31,
   *«Variant deveria ser Size. Nos botões deveríamos ter Small e Big»*.

⇒ **O desenho é o do Figma (dois níveis, dado explícito) com o GESTO do XD (modificar → gravar →
nomear).** Nenhum dos dois faz isso sozinho: o Figma obriga a montar o conjunto antes, e o XD não
tem propriedade nenhuma.

---

## §2 — O desenho, e a PORTA ÚNICA de cada pergunta

### §2.1 — O dado

```rust
/// O que esta RECEITA declara — propriedade → valor. Vazio = a receita não participa de eixo nenhum.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantValues {
    pub values: BTreeMap<String, String>,   // BTreeMap: determinismo (HR-5)
}
```

Vive na **raiz da receita** (`MasterRoot`). ⚠️ **O elo de família continua a ser o `InstanceOf`** que
já existe: uma variante é `MasterRoot` **e** `InstanceOf { master: base }`. Não se inventa um segundo
ponteiro — as duas cercas que o P0 de 31/08 pagou (`follow` sobre raiz-receita · `ItselfAsMaster` na
porta do swap) continuam a valer e continuam gateadas.

### §2.2 — Uma porta por pergunta

| Pergunta | A porta ÚNICA | Hoje |
|---|---|---|
| que versões existem nesta família? | `variant_family::members(sim, base_id)` | derivado de nomes |
| que valor esta receita declara? | `VariantValues` na raiz dela | `parse_combo(name)` |
| que versão esta cópia segue? | `InstanceOf.master` na raiz da cópia | idem (fica) |
| como se cria uma versão? | `variant_save::save_variation(sim, instance, prop, value)` | `make_master` + escrever chaves |
| como se troca de versão? | `instance_variant::swap` | idem (fica) |
| como se renomeia um valor? | `variant_save::rename_value(sim, recipe, prop, value)` | `with_value` no nome |

⛔ **O NOME deixa de responder a qualquer uma delas.** Renomear passa a ser inerte — que é o pedido.

### §2.3 — O gesto, passo a passo

1. O artista escolhe uma **cópia** e muda-lhe alguma coisa ⇒ ela ganha `ObjectInstance.overrides`.
2. O cartão *Properties* mostra **`Salvar Variação…`** — ⚠️ **só quando há overrides**: sem
   modificação não há o que gravar, e um botão que não faz nada é a doença que a caça aos knobs
   mortos nomeia.
3. Clique → o cartão abre um **formulário em linha** (o mesmo `TextInput` do campo de valor que já
   existe; ⛔ nada de diálogo modal):
   - **primeira variação da família:** dois campos — *Propriedade* (ex.: `Size`) e *Nome da versão*
     (ex.: `Big`);
   - **as seguintes:** um campo só — *Nome da versão* (a propriedade já é conhecida).
4. `Enter` ⇒ a app: cria a receita nova a partir da cópia modificada, liga-a à base
   (`MasterRoot` + `InstanceOf{base}`), escreve `VariantValues{Size:"Big"}`, **religa a cópia à
   receita nova** e **limpa os overrides dela** (foram absorvidos).
5. ⚠️ **A base ganha valor no mesmo gesto**, senão a fileira nasce com um botão só: recebe
   `Size = <nome da receita base>`. Se o artista lhe chamou `Casa`, a fileira lê `Size: [Casa] [Big]`
   — e o rótulo é **renomeável por clique**, que é a maquinaria de `ValueEdit` que já existe.
6. O cartão passa a mostrar `Size` com os dois botões, `Big` aceso, e o `Salvar Variação…`
   **desaparece** (não há overrides pendentes).
7. Modificar uma cópia que já segue `Big` mostra **dois** botões: **`Atualizar "Big"`**
   (= o verbo `Apply`, que já existe) e **`Salvar Variação…`**. *Sem os dois, gravar uma correcção
   obriga a criar uma versão a mais.*

### §2.3-bis — E a SEGUNDA propriedade? (pergunta do Enio, 2026-09-01)

*«e se o usuário quiser criar outros tipos de propriedades depois?»*

⭐⭐⭐ **A resposta que unifica: criar a PRIMEIRA propriedade e criar a SEGUNDA são o MESMO
gesto.** Os dois precisam exactamente das mesmas três respostas — *como se chama a propriedade* ·
*como se chama o que já existe* · *como se chama isto que acabei de fazer*. ⇒ **uma porta só**, e o
passo 3 do §2.3 deixa de ser um caso especial da primeira vez.

O formulário do `Salvar Variação…` tem **um selector e um campo**:

```
Propriedade:  [ Size ▾ ]        ← as que a família já tem + «Nova propriedade…»
Nome:         [ Big         ]
```

- **Valor novo numa propriedade existente** (o caso comum): escolhe `Size`, escreve `Big`. Um campo.
- **Propriedade nova** (`Nova propriedade…` no selector): o formulário cresce para três campos —
  *Propriedade* (`Color`), *Como se chama o que já existe* (`Normal`), *Nome desta versão* (`Red`).
  ⚠️ **A pergunta do meio é obrigatória e é a que torna a coisa honesta:** nascer uma propriedade
  significa que **toda receita da família** passa a declarar um valor nela, e o valor delas é o que
  já existia — sem lhe dar nome, a fileira nova nasceria com um botão em branco.

⛔⛔ **Uma propriedade NUNCA nasce sozinha, e isso é uma decisão, não uma falta.** Uma propriedade
com um valor só é uma fileira com um botão — um controlo que não escolhe nada, que é a espécie que
a caça aos knobs mortos nomeia. ⇒ **não há gesto de «criar propriedade vazia»**, e por isso também
não há gesto de apagar: a fileira é **DERIVADA** — mostra-se quando a família declara **dois ou mais
valores distintos** naquela chave, e desaparece sozinha quando os valores voltam a concordar.
*Uma fileira derivada não pode ficar morta.*

#### A MATRIZ, e a combinação que não existe

Com duas propriedades a família vira uma grelha `n × m`, e o artista quase nunca a enche. Uma cópia
está em `Size=Big, Color=Normal`; ele carrega em `Red`; a receita `{Big, Red}` **pode não existir**.

⛔ **As três saídas foram pesadas:**

| Saída | Porquê não |
|---|---|
| não fazer nada | é o **chip morto sob o dedo** — o report chega igual ao de um botão nunca pintado |
| aproximação («o mais parecido») | o botão acende num valor e a arte mostra outro: o app **mente**, que é a doença de 31/08 outra vez |
| recusar com aviso | honesto, mas manda o artista fazer à mão o que a app sabe fazer |

⭐ **A escolhida: o clique CRIA a combinação em falta** — a partir da versão em que a cópia está,
com o valor novo aplicado — e a voz diz o que nasceu (*«criei Big / Red»*). É **um passo de undo**
como qualquer outro. ⚠️ O chip em falta pinta-se **esmaecido com `+`**, para que o gesto não seja
uma surpresa: *acender é escolher; esmaecido com `+` é criar*.

⚠️ **Tectos, e eles são MEDIDOS (§0.0):** hoje `MAX_INSTANCE_AXES = 4` e `MAX_INSTANCE_AXIS_VALUES`
são cercas da **tabela de ids**, sem medição nenhuma por baixo. Com composição o número que
interessa deixa de ser a contagem de fileiras e passa a ser o **produto** — quem os declarar mede o
custo de pintar a grelha cheia e escreve a tabela ao lado.

### §2.4 — O que MORRE

`variant_axes::{parse_combo, display_name, hidden_count, row_label, with_value, variant_name,
chip_label, declared_axes}` · o módulo `instance_declared_value` **inteiro** (`follow`, `apply`,
`mirror_onto_copy`, `mirror_onto_copies_of`, `family_declares`, `write_combo`) e os seus dois
ficheiros de teste · o gate `the_braces_law_is_wired_into_every_door.rs` · a acção
`InspectorNameCommitted` · `App::followed_selection` · o gancho no `hierarchy_rename` · o dreno
`rename_variant_value` na rota do nome · o selo `*²` na Hierarquia.
⚠️ **`ValueEdit` e o campo em linha SOBREVIVEM** — mudam de sujeito (renomear o rótulo do botão),
que é para o que foram construídos.

---

## §3 — Contrato congelado e schema

- **§6 não é tocado.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4`,
  `NodeOp=2` / `OpResolver=1` / `NodeManifest=8`, e a superfície do `ph2d-vector-doc` ficam
  intactas — esta wave mexe em `ph2d-ecs` (componente novo), `ph2d-editor-core` (modelo do cartão),
  `ph2d-panel-inspector` (pintura) e `shells/desktop` (verbos). **Prova por grep antes do commit:**
  `git diff --name-only | grep -E 'ph2d-vector-doc|ph2d-vector-traits|nodegraph'` tem de vir vazio.
- ⛔⛔⛔ **O `PROJECT_SCHEMA` NÃO se move — esta linha do plano estava ERRADA, e a medição corrigiu-a**
  (2026-09-01). O plano mandava subir de `105` *«porque o postcard é posicional»*. **O precedente v86
  já respondia**: um componente é **name-keyed** e só o BLOB dele é posicional — o `type_id` é
  `stable_type_id(canonical_name)`, o hash do nome. ⇒ `VariantValues` estreia um id que **nenhum
  ficheiro antigo contém**, o restauro é por chave, e um projeto v105 simplesmente não traz o
  componente. ⚠️ E a **ausência já tem o significado certo**: sem declaração, modo plano.
  *Um bump aqui teria recusado todo ficheiro gravado, em nome de uma incompatibilidade que não
  existe* — a diferença entre «acrescentar um tipo» e «mudar o layout de um tipo», que é a lei que
  os degraus v6/v7/v8 e v86 escreveram.

---

## §4 — A UI, pelas QUATRO condições independentes

| Condição | O que a satisfaz |
|---|---|
| **existe** | `INSP_INSTANCE_SAVE_VARIATION`, `INSP_INSTANCE_UPDATE_VERSION`, `INSP_INSTANCE_PROP_EDIT` na tabela de ids |
| **é pintado e registado** | pintados no `paint_properties_card`, com `hit_index.register` **depois** da pergunta *«isto vira campo?»* (a cura do chip fantasma de 31/08) |
| **o clique chega ao barramento** | `EditorAction::InspectorSaveVariation { root_bits, property, value }` e `InspectorUpdateVersion { root_bits }` |
| **a sequência leva a algum lugar** | o dreno chama `save_variation` / o verbo `Apply`, e o cartão do quadro seguinte mostra a fileira nova com o botão aceso |

⚠️ **A 4.ª condição é a que os gates unitários não vêem** — foi ela que deixou passar as quatro
mutações de fiação de 31/08. O gate dela é **textual** (o molde do swap-door) mais um `seam_*` que
carrega num pixel.
⚠️ **Tectos:** `MAX_INSTANCE_AXES = 4` e `MAX_INSTANCE_AXIS_VALUES` deixam de ser cercas da
gramática e passam a ser cercas da **tabela de ids** — ⛔ §0.0: quem os declarar mede-os e escreve a
tabela ao lado; hoje eles não têm medição nenhuma.

---

## §5 — Gates, red-first, e a fixtura que contém o fenómeno

1. `renaming_anything_never_changes_a_variant` — ⭐ **o gate que dá nome à wave**: renomear a base,
   a variante e a cópia, com chaves e tudo, e o elo + `VariantValues` + a fileira ficam **byte a
   byte iguais**. *Fixtura com o fenómeno: nomes que ainda contêm `{Size=Big}`* — se ela usar nomes
   limpos, prova nada.
2. `saving_a_variation_absorbs_the_overrides_and_relinks_the_copy` — overrides não-vazios antes,
   **vazios** depois, `InstanceOf` a apontar para a receita nova, e as outras cópias **intactas**.
3. `the_first_variation_gives_the_base_a_value_too` — senão a fileira nasce com um botão só.
4. `the_button_only_appears_when_there_is_something_to_save` — sem overrides, sem botão (o oráculo é
   o modelo do cartão, não a função).
5. `updating_a_version_does_not_create_a_second_one` — o `Atualizar "Big"` usa o `Apply`.
6. `two_versions_never_declare_the_same_combination` — a porta recusa e **fala**.
7. `no_code_reads_braces_any_more` — **textual**, sobre `crates/` e `shells/`: nenhum
   `parse_combo`/`with_value`/`{`-em-nome sobrevive. É o censo que impede a lei velha de voltar por
   uma porta esquecida.
8. `a_new_property_names_what_already_existed` — nascer `Color` escreve `Color=Normal` em **todas**
   as receitas da família; sem isso a fileira nova nasce com um botão em branco.
9. `a_row_with_a_single_value_is_never_painted` — a fileira é derivada: colapsar os valores
   faz a fileira **desaparecer**, e nenhuma fica com um botão só.
10. `clicking_a_missing_combination_creates_it_and_says_so` — o oráculo é o MUNDO (a receita nova
    existe e a cópia segue-a) **e** a voz; ⛔ e um gate irmão prova que ela nasce da versão em que a
    cópia estava, não da base.
11. `seam_save_variation.rs` — gesto REAL (clique no pixel do botão → barramento → mundo).
   ⛔ Um `Click` sintético passa com o botão morto sob o ponteiro (lição do §8 do doc 27 do Vector).

⚠️ **Cada cerca com o SEU gate** — duas cercas juntas escondem-se uma à outra, e a mutação que
apaga só uma sobrevive (medido a 31/08: a 3.ª cerca era código morto e treze gates não o disseram).

---

## §6 — A cena de smoke

Reescrever a **`=80`** (`variant_flow_smoke.rs`) de ponta a ponta, **sem uma chaveta**:
nasce `Casa` → *Make Component* → *Instantiate* → muda a cópia → **`Salvar Variação…`** →
`Size` / `Big` → a fileira nasce com `[Casa] [Big]` → troca por clique → **renomeia tudo** e prova
que nada se mexe. Cada passo imprime o que fez, e o passo do salvar diz **quantos overrides
absorveu** e **para que receita a cópia passou**.
⛔ Os números da mensagem final saem da sonda headless corrida ANTES de escrever a mensagem.

---

## §7 — Ordem das waves

| Wave | O que entra | Estado ao fim |
|---|---|---|
| **W1** | `VariantValues` + `variant_family` + o cartão a ler DADO + a lei das chaves **apagada** + o `Make Component` sobre cópia a escrever dado | variantes funcionam; criam-se pelo verbo que já existe |
| **W2** | o botão **`Salvar Variação…`** + o formulário em linha + o valor da base + a absorção dos overrides | o fluxo que o Enio descreveu |
| **W3** | **`Atualizar "Big"`** + renomear valor e propriedade pelo cartão + `+ Propriedade` (2.º eixo) | dois níveis completos |
| **W4** | varredura: o gate textual, o `*²` fora da Hierarquia, o smoke `=80` reescrito, tectos MEDIDOS | nada no repo lê chaves |

⚠️ **A W1 apaga a lei velha no MESMO passo em que a nova lê dado.** Deixar as duas vivas por uma
wave seria ter **duas fontes para a mesma pergunta** — que é exactamente a doença que esta wave
existe para curar.
