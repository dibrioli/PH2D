# A TINTA FINA SOBREVIVE AO TRAÇO SEGUINTE — e o `16x`

> `line/sculpt3d` · 2026-09-21 · dois reports do dono numa mensagem só.
> Documento para a próxima LLM e para o integrador. O §6 tem o que uma leitura
> rápida do diff entende ao contrário; o §7 as premissas que a medição derrubou.

---

## §1 — O que o dono disse

> *«Muito bom! MAs ainda inconcistente: traços posteriores estão reduzindo a
> resolução dos traços em alta resolução anteriores. como se voltasse para o
> modo mesh. Corrija.*
> *E acrencente a opção de 16x»*

Duas coisas — e a primeira tinha **DUAS causas**: uma PREMISSA que a wave
anterior deixou de pé (§2) **e** um plano emprestado que um gesto abandonado
nunca devolvia (§2-bis). ⚠️ **A primeira sozinha não curava o report**, e quem
o disse foi o gate de produto, não uma leitura.

⚠️ **Conte o DELTA:** `PROJECT_SCHEMA` **0** · os três registos **0** · zero
contrato · zero ADR · zero pacote externo. A fileira ganha **um chip** e o
`NIVEL_MAX` do produto sobe de `3` para `4`.

---

## §2 — «Volta ao modo Mesh» é LITERAL, e a causa é uma frase escrita em 19/09

O plano de tinta fina é **paramétrico nas FACES**: o endereço de uma amostra é
`(face, sítio)`. Ele sobrevive a qualquer pincel que só mova vértices e **não
sobrevive** a um que parta ou funda uma face — e a [`concorda_com`] é lida em
todo quadro, com a discordância a reconstruir o plano **semeado da cor por
vértice**, que é exactamente *a tinta com a resolução da malha*.

⛔⛔ **E o `Verb::Paint` mudava topologia.** Ele cai no `_ =>` do
`refina_no_dyntopo`/`colapsa_no_dyntopo`, logo com a topologia dinâmica armada
**cada traço de cor partia faces** ⇒ o plano do traço anterior era deitado fora
e re-semeado grosso. Carregar no `P` sozinho fazia o mesmo, por outra porta: o
`toggle_dyntopo` **TRIANGULA** a peça.

⭐⭐⭐ **A ordem do dono de 19/09 — *«permita que o dynamic topology funcione
para os 3 pincéis»* — foi justificada por escrito com *«a cor por vértice é uma
IMAGEM e a resolução dela É a da malha»*.** Com o plano armado **essa premissa é
FALSA**: a resolução da tinta deixou de ser a da malha. ⇒ refinar debaixo de um
pincel de cor **compra ZERO** e custa o plano inteiro.

*É a §0.0 aplicada a uma premissa em vez de a um número: quem move o facto que
tornava uma regra necessária tem de reconferir a regra.*

### A cura é uma PORTA, e ela não é um `match` novo

```rust
// crates/ph2d-app-sculpt3d/src/tinta_da_peca.rs
pub(crate) fn o_gesto_muda_a_topologia(verbo: Verb, tinta_fina_armada: bool) -> bool {
    if verbo.paints_color() && tinta_fina_armada {
        return false;          // ⭐ a lei nova, e a ÚNICA linha dela
    }
    verbo.refina_no_dyntopo() || verbo.colapsa_no_dyntopo()   // a tabela de sempre
}
```

Três consumidores, e **nenhum é opcional**:

| consumidor | sem ele |
|---|---|
| `dyntopo::refine_for_dab` | o traço parte faces e o plano morre a meio |
| `history_dyntopo::open_dyntopo_stroke` | a foto do desfazer + a **triangulação** do pen-down |
| `dyntopo::toggle_dyntopo` | **carregar `P` sozinho** destrói o plano |

⚠️ **O terceiro é o que quase escapou:** ele não tem verbo nenhum em mãos — a
guarda ali é `if self.tinta_fina_armada() { return (true, 0); }`, e sem ela a
tecla que o roteiro da `=52` manda carregar no passo (6) apagava a lição.

⚠️ **`tinta_fina_armada()` pergunta aos DOIS sítios** (`self.stroke.tinta_fina`
**ou** `self.objects[active].tinta`) porque o plano é **EMPRESTADO** ao traço: a
meio de um gesto o `Option` da peça está VAZIO, e uma pergunta só à peça leria
*«não há plano»* exactamente quando há um a ser pintado.

---

## §2-bis — E a cura da premissa era NECESSÁRIA e NÃO SUFICIENTE

⛔⛔⛔⛔ **O report tinha DUAS causas, e a segunda é maior.** Com o §2 no sítio
e a suíte inteira verde, o gate de PRODUTO
(`dois_tracos_de_cor_com_dyntopo_nao_perdem_o_detalhe_do_primeiro`) **reprovou**
— *«o plano foi RECONSTRUÍDO entre os dois traços», `47 106` contra `0`*. A
topologia estava intacta (o assert acima dele passou ⇒ o §2 funciona) e **o
plano tinha desaparecido**.

A sonda (`diag_onde_mora_o_plano_entre_dois_tracos`) diz onde ele estava:

```
inicio               peca=Some(47106)  traco=None
A: pos-sync          peca=Some(47106)  traco=None      ← devolvido
B: pos-sync          peca=None         traco=Some(0)   ← PRESO no traço, e pintou ZERO
```

⭐⭐⭐⭐ **O MECANISMO:** o pen-down **empresta o plano por um `take`** (linha
`318`) **antes** de saber se o gesto vai pegar — e tem de ser antes, porque o
primeiro dab precisa dele. A decisão vem `92` linhas depois (`410`): se o raio
**erra o modelo**, o gesto vira `Drag::Orbit`. E o `close_stroke` — o único
sítio que devolve — corre **só** no pen-up de um `Drag::Sculpt`/`Filter`
([`input.rs:225`/`243`], os dois únicos chamadores de produto) ⇒ **o plano morre
dentro do traço abandonado**, a peça fica sem ele, e o `garante` do quadro
seguinte reconstrói-o **semeado da cor por vértice**: *«voltou para o modo
mesh»*, à letra.

⛔ **E isto NÃO precisa da topologia dinâmica: basta UM clique fora da peça** —
o gesto que o comentário do próprio `input_down` chama de *«o mais comum do
mundo — arrastar no vazio»*. É a explicação que faltava para *«traços
posteriores»* sem que o dono tenha de armar nada.

⚠️ **E não é só o plano:** quando se chega àquele braço, o pen-down já tirou a
fotografia da superfície, já clonou a **malha INTEIRA** para o `dyn_before` e já
abriu a superfície de referência (linhas `364`–`376`). O gesto abandonado
segurava as quatro coisas.

⭐ **A cura é a PORTA, não uma devolução escrita ali:** o doc do `empresta` já
promete que *«o `close_stroke` a chama sempre, inclusive no caminho de
recusa»* — **o que faltava era este caminho de recusa CHAMAR o `close_stroke`**,
que devolve as quatro. Escrever um `devolve` local seria a segunda resposta à
mesma pergunta e deixaria as outras três abertas.

### ⚠️⚠️ E a FIXTURA do gate estava errada — foi ela que expôs o defeito

O traço B batia em `x = 250`, e a peça ocupa **`x ∈ [280, 610]`** a `y = 350`
(medido pela sonda; raio `50 px`) ⇒ **B ERRAVA A PEÇA**, e o gate que existia
para reproduzir o report media dois traços de que **só um existia**. Hoje B bate
em `560`, que acerta e **não sobrepõe** A (`[350, 498]` contra `[510, 658]`) —
senão o `mudou == 0` reprovaria sobre produto certo.

⭐⭐ *A ironia é o achado: a fixtura errada percorreu exactamente o caminho do
defeito real. Se eu a tivesse escrito bem à primeira, o gate teria passado e o
report do dono continuaria vivo.*

---

## §3 — O TERCEIRO defeito era VIVO, MUDO, e foi achado a LER números de linha

O pen-down avisa quando um gesto vai custar o detalhe fino
(`recusa::Entradas`). Medido: **ele nunca soou desde que existe**.

```
input_down.rs:318   empresta(&mut self.objects[i].tinta, &mut self.stroke)  ← o plano SAI da peça
input_down.rs:381   Entradas { tinta_fina_armada: o.tinta.is_some(), .. }   ← e aqui já é None
```

⇒ o aviso lia `false` **sempre**, e um smoke nunca o mostraria. A cura é a mesma
porta (`self.tinta_fina_armada()`), e a ORDEM das duas linhas entra no gate —
*uma lei verificada nas duas pontas ainda pode ser contrariada no meio*.

⭐ E a voz ganhou a **metade SIMÉTRICA**, que é a que o dono vai ler agora que
os pincéis de cor deixaram de mexer na malha:

> `app.sculpt3d.recusa.a_tinta_fina_dispensa_a_topologia` —
> *«Dynamic Topology is on, and {nome} will NOT densify the mesh — Paint Detail
> is armed for this piece…»*

⚠️ Sem ela o artista liga o `P`, pinta, **e o arame não adensa**: o produto
certo lê-se como o interruptor partido.

---

## §4 — O `16x`: medido ANTES de escrito, e a medição obrigou duas curas

O doc do `NIVEL_MAX` exigia por escrito que um degrau novo fosse medido antes de
existir. Medido (`--release`, peça de fábrica, `98 306` vértices):

| `k` | lado | amostras | plano | construir | **empacotar** |
|---|---|---|---|---|---|
| `2` | `4` | `1,57 M` | `18 MB` | `32,9 ms` | `7,2 ms` |
| `3` | `8` | `6,29 M` | `72 MB` | `46,1 ms` | **`21,9 ms`** |
| **`4`** | **`16`** | **`25,2 M`** | **`288 MB`** | **`94,8 ms`** | **`81,7 ms`** |

⛔⛔ **A coluna «empacotar» era paga POR QUADRO enquanto o traço durava** — já
era `21,9 ms` contra um quadro de `16,7` no `8x` que shipava, e o `16x` seria
`81,7`. *O degrau novo não podia entrar sozinho; ele tornou visível uma parede
que já existia um degrau abaixo.*

Duas curas, as duas medidas:

1. **O empacotamento deixou de existir.** Os três `collect()` que achatavam
   `&[[f32; 3]]` e `&[[u32; 3]]` eram **cópia pura** — aquilo já é o bloco de
   bytes que a placa quer ⇒ `bytemuck::cast_slice`, vista sem cópia.
2. **O upload passou a subir só o que o traço ESCREVEU.** A `Tinta` ganhou uma
   janela `suja: Vec<bool>` **paralela ao `tocadas`** (indexada por SLOT, nunca
   por vértice — ⚠️ o `dirty` que já existia é uma janela de VÉRTICES e não
   serve), drenada por `drena_sujas`, e o `ph2d-mesh-render` coalesce-a em
   corridas de bytes (`corridas_das_sujas`, ordena · dedup · funde contíguas).
   ⇒ o custo por quadro passa a ser `O(pegada)` e não `O(plano)`.

⚠️ **A rota incremental tem TRÊS guardas e todas recusam para o caminho
cheio** (slot ausente · `!armado` · `n_amostras` diferente do plano). A última é
a que impede o defeito caro: *um plano que mudou de tamanho não pode receber uma
escrita parcial nos offsets antigos.*

---

## §5 — A fileira

`DetalheDaTinta::{Malha, Dois, Quatro, Oito, Dezasseis}` — `ALL: [Self; 5]`,
`nivel()` → `Some(4)`, id `sculpt3d.tinta_detalhe.4` **apendado** (a posição no
array **é** a tag do segmentado), chave `panel.sculpt3d.tinta_detalhe.dezasseis`
→ `"16x"`.

⚠️ **O roteiro da `=52` deixou de dizer *«o último»*.** Ele agora nomeia o `8x`
por uma `const DEGRAU_DA_LICAO` (`#[cfg(test)]`, dois consumidores: o gate da
densidade da peça e o do texto), e ganhou um passo **(4-bis)** para o `16x`.
*Até 20/09 «o degrau da lição» e «o topo da fileira» eram a mesma coisa por
ACIDENTE, e o dia em que deixaram de o ser foi o seguinte.*

---

## §6 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. **O `Verb::Paint` não deixou de poder adensar.** Ele deixa de o fazer **só
   quando o plano está armado**; sem plano a ordem do dono de 19/09 continua
   inteira, e há CONTROLO no gate a medi-lo.
2. **A guarda do `toggle_dyntopo` não é a mesma lei da do dab.** Ali não há
   verbo — o que se protege é a TRIANGULAÇÃO, e uma leitura que a veja como
   redundante apaga o passo (6) do roteiro.
3. **`tinta_fina_armada()` não é `o.tinta.is_some()`.** Durante um traço o plano
   está no `SculptStroke`, e foi essa diferença que manteve a voz muda.
4. **A janela `suja` não é o `dirty`.** Uma é por SLOT de amostra, a outra por
   VÉRTICE; usá-las uma pela outra sobe o bloco errado.
5. **O `n_amostras` do slot não é derivável do `cap_tri`.** Foi a derivação que
   pôs o `wgpu` a recusar um upload na wave anterior; ele é gravado ao lado do
   buffer que descreve.
6. **`NIVEL_MAX = 4` não é «mais um degrau»:** é `4×` o plano do `8x`, e o que
   o torna afordável são as duas curas do §4 — revertê-las com o `16x` ligado põe
   `81,7 ms` por quadro num orçamento de `16,7`.
7. **A voz nova não bloqueia.** Suprimir o passe de topologia com o plano armado
   continua a ser **decisão do dono** (§9).
8. **O `close_stroke()` no braço da ÓRBITA não é arrumação.** Ele é o que
   devolve o plano — e a foto da superfície, o `dyn_before` e a referência — de
   um gesto que errou a peça; sem ele o report de 21/09 continua vivo **sem a
   topologia dinâmica sequer armada**.

---

## §7 — As premissas que a medição derrubou

- *«a cor por vértice é uma IMAGEM e a resolução dela É a da malha»* (19/09) —
  **verdade nessa data e falsa desde a wave do plano**. Era a justificação
  escrita da ordem que este report contradiz.
- *«a colheita do upload é modesta»* — o empacotamento era **`21,9 ms` por
  quadro** no degrau que já shipava, e nenhuma régua desta linha o media.
- *«o aviso do pen-down cobre esta fronteira»* — ele existia, estava ligado, e
  **lia sempre `false`**.
- *«curada a premissa, o report fecha»* — **falso, e foi o gate de produto que o
  disse**: faltava o plano que um gesto abandonado nunca devolvia (§2-bis).
- *«a fixtura do gate reproduz o report»* — ela **errava a peça** (`x = 250`
  contra `[280, 610]`), e media dois traços de que só um existia.

---

## §8 — A prova de mutação, e o que ela achou

⛔⛔⛔⛔ **A cura desta wave nasceu SEM RÉGUA NENHUMA, e foi a prova de mutação
que o disse: `20` de `25` sangravam, e as QUATRO sobreviventes reais eram,
exactamente, as quatro metades escritas hoje.**

| mutação | o que ela apaga | sangrava? |
|---|---|---|
| **M19** | a tinta **EMPRESTADA** deixa de contar na porta | ⛔ sobrevivia |
| **M20** | o `refine_for_dab` volta a perguntar **só ao verbo** | ⛔ sobrevivia |
| **M21** | ligar o `P` volta a **triangular** com o plano armado | ⛔ sobrevivia |
| **M22** | o pen-down volta a fotografar/triangular para quem não mexe | ⛔ sobrevivia |

⭐⭐⭐ **E a causa é ESTRUTURAL, não distracção: a única prova de comportamento
daqueles três consumidores vive em gates `#[ignore]` de GPU.** O
`tinta_no_produto_tests.rs` mede-os de verdade — e o arnês corre
`nextest run -p … --lib` **sem `--ignored`**, e *o CI nunca corre um
`#[ignore]`* (§5). ⇒ **uma regressão na cura desta wave seria silenciosa em
toda a parte onde alguém a fosse encontrar.**

⛔ **E não havia como escrever um gate de comportamento em CPU:** os três
consumidores são métodos de `Sculpt3dScene`, e `Sculpt3dScene::new` pede um
`wgpu::Device` — os quatro testes do `dyntopo_tests.rs` são **todos**
`#[ignore]` por isso. *Quando a decisão não tem pixel nenhum e o gate precisa de
uma placa, a lei está no sítio errado* — e aqui ela já estava no sítio certo (a
porta é pura e a M17/M18 sangram nela). **O que faltava era o ELO.**

⇒ [`tinta_fiacao_tests.rs`](../../../crates/ph2d-app-sculpt3d/src/tinta_fiacao_tests.rs),
um censo de TEXTO dos quatro elos, com **duas** metades que o tornam honesto:

- a **prosa é cortada antes de se medir** (`sem_prosa`), senão o doc-comment que
  EXPLICA a cura contém o nome da porta e satisfaz a agulha — a armadilha que a
  régua do fio da Fase B da física já pagou;
- e o **CONTROLO** é o inverso (`so_a_prosa`): cada agulha tem de estar
  **ausente** da metade comentada, senão o corte não está a medir nada.

⚠️ **Dois gates e não um:** a M20 **substitui** a chamada pela pergunta antiga
em vez de a apagar ⇒ um vê a **ausência** da agulha certa, o outro a **presença**
da errada. *Um ficheiro pode conter as duas*, e foi assim que a lei do projectar
sobreviveu numa segunda cópia em 15/09.

**Depois do censo, e com a `M25` da §2-bis já dentro: `25` de `26` sangram**, e
o único sobrevivente é a **M15**, que é o CONTROLO — uma mutação inerte (uma
linha em branco) que **não pode** sangrar.

### E o arnês do UPLOAD achou uma régua VÁCUA minha

`5` de `7` na primeira corrida, com duas sobreviventes — e as duas são animais
diferentes, o que só se soube depois de contar os chamadores:

⛔⛔ **A `U1` era um buraco REAL, e no gate que eu tinha escrito de propósito
para aquela lei.** A metade (3) do `a_janela_das_amostras_sujas_e_o_que_o_traco_escreveu`
promete *«uma amostra RE-ESCRITA volta à janela»* e afirmava
`!sujas.is_empty()` — só que **o 2.º dab ANDOU**, logo toca amostras NOVAS, que
nascem sujas ⇒ a janela nunca vinha vazia **mesmo com a marca por escrita
apagada**. *Uma régua que conta QUANTOS nunca vê QUAIS* — o `edge_max` e o `χ`
outra vez, agora dentro de um gate cujo próprio doc-comment nomeava a
propriedade certa. ⇒ ela mede agora a **INTERSECÇÃO** com o conjunto do 1.º
dab, que é a única população onde a lei é observável.

⚠️ **A `U3` NÃO pode sangrar, e isso é MEDIDO — não é um gate em falta.** O
`slot_de` tem **UM** chamador, e esse chamador marca a amostra suja
**incondicionalmente** no fim da escrita ⇒ *nascer suja é o cinto das
suspensórias da `U1`*, e não existe percurso no produto em que um slot nasça
sem ser escrito a seguir. ⛔ **Ela fica na mesma** (apagá-la para comprar um
ponto no placar seria optimizar a métrica), **contada como NOMEADA** e com a
medição escrita no próprio arnês — um segundo chamador de `slot_de` que não
escreva torna-a viva no mesmo dia.

### E o próprio ARNÊS tinha dois defeitos, os dois achados a usá-lo

⛔⛔ **Faltava-lhe o 4.º controlo: a corrida LIMPA tem de estar VERDE.** Ele
media `verde=$(corrida | populacao)` — e **o `|` deita fora o código de
saída** —, logo só verificava que *algum* teste correu. Com a árvore já
vermelha antes de mutar, **toda** mutação se lê como `SANGRA` e o placar sai
perfeito. ⚠️ *O erro é para o lado que ninguém investiga: um placar cheio não
faz ninguém olhar duas vezes.* Os dois arneses abortam agora com o `rc != 0` e
imprimem as linhas de falha.

⚠️ **E a M25 estava DEPOIS do `echo` do sumário** ⇒ ela corria, imprimia o
veredito dela **abaixo do total**, e o total dizia `24 de 25` sem a contar —
*um total impresso antes do último caso mede outra população*.

---

## §8-bis — O INCIDENTE do restauro

⛔⛔⛔ **Um arnês de mutação congelou uma mutação na árvore, e só a SUÍTE a
viu.** O `muta_a_metade_visivel.sh` fotografa `$APP`/`$PAN` e restaura a cada
caso; a árvore foi copiada **enquanto ele corria** e a cópia levou a **M1**
(`Some(c) => Tinta::semeada(c, faces(), k)` reescrito para
`Tinta::nova(mesh.vert_count(), faces(), k)`, os dois braços do `match`
idênticos). O passe de compilação fechou **VERDE** — os dois braços compilam —,
e quem a apanhou foi a suíte: `883` testes, `882` verdes, **`1` vermelho em
`0,004 s`** com a mensagem do gate (*«a amostra 0 nasceu [1.0,1.0,1.0] e não a
cor da peça»*).

⚠️ *A mutação viva é, por construção, uma edição que COMPILA — o laço interno é
cego a ela, e ela é indistinguível de um defeito próprio.* Lei registada na
memória; a regra que fica é **nunca copiar nem editar `$APP`/`$PAN` com o arnês
a correr**, e **nunca deixar uma tarefa de GPU em fila enquanto ele corre** (ela
pode ganhar a placa a meio de uma mutação e construir a árvore mutada).

---

## §9 — ABERTO

- ⏳ **O SMOKE do dono** desta wave (cena `=52`, passos (4-bis) e (6)).
- ⏳ **Suprimir o passe de topologia** com o plano armado — hoje o pen-down
  **FALA e não bloqueia**; a decisão é de produto.
- ⏳ O plano **não viaja no `.ph2dproj`** (dívida herdada da wave anterior, ao
  lado da cor por vértice).
- ⏳ Voltar a `Mesh` **perde** o detalhe fino, declarado no roteiro.
