# A TINTA FINA SOBREVIVE AO TRAÇO SEGUINTE — e o `16x`

> `line/sculpt3d` · 2026-09-21 · dois reports do dono numa mensagem só, e um
> TERCEIRO a seguir (§10) que devolveu a wave com *«não corrigiu»*.
> Documento para a próxima LLM e para o integrador. O §6 tem o que uma leitura
> rápida do diff entende ao contrário; o §7 as premissas que a medição derrubou.
>
> ⛔⛔ **Leia o §10 antes de acreditar no §2-bis como fecho:** ele está CERTO e
> é **metade** da resposta — ele impede o plano de MORRER num gesto abandonado,
> e não faz o traço EXISTIR.

---

## §1 — O que o dono disse

> *«Muito bom! MAs ainda inconcistente: traços posteriores estão reduzindo a
> resolução dos traços em alta resolução anteriores. como se voltasse para o
> modo mesh. Corrija.*
> *E acrencente a opção de 16x»*

Duas coisas — e a primeira tinha **TRÊS causas**: uma PREMISSA que a wave
anterior deixou de pé (§2), um plano emprestado que um gesto abandonado nunca
devolvia (§2-bis) **e** um traço de cor que, começando fora da peça, nunca
chegava a abrir (§10). ⚠️ **Nenhuma das duas primeiras sozinha curava o
report**, e quem o disse foi o gate de produto — e depois o dono.

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
- ⚠️ **E o §10.7 acrescenta três**, incluindo uma DECISÃO do dono — esta
  lista fechou antes de o report seguinte chegar.

---

## §10 — «NÃO CORRIGIU»: o 2.º report do mesmo dia, a auditoria e a ordem

> *«não corrigiu. parece que se começar a pintar sem tocar um vertex acontece
> mais vezes de sumir a pintura. permita pintar mesmo se tocar em vertex.
> auditoria»* — o dono, 2026-09-21, depois do §2-bis já commitado.

⚠️ **LEITURA DECLARADA, e ela tem de ser confirmada pelo dono:** li *«permita
pintar mesmo se **[não]** tocar em vertex»* — a frase anterior é *«se começar a
pintar **sem** tocar um vertex»*, e a leitura literal (*«mesmo se TOCAR»*) pede
uma capacidade que já existe. **Se a leitura estiver errada, a cura desta secção
está errada inteira.**

### §10.1 — A auditoria reproduziu o defeito ANTES de eu tocar em código

A régua é a sonda versionada `diag_o_gesto_que_comeca_fora_da_peca`
(`#[ignore]` + placa), que conta **amostras pintadas** — nunca o tamanho do
plano, que foi a régua que enganou a wave anterior:

```
[diag] o plano nasceu com 47106 amostras; a peca ocupa x em [280, 610]
[diag] inicio                    plano=47106  traco=None  pintadas=0
[diag] A: dentro da peca         plano=47106  traco=None  pintadas=960
[diag] B: comecou FORA, entrou   plano=47106  traco=None  pintadas=960   ← ZERO
[diag] C: dentro outra vez       plano=47106  traco=None  pintadas=1925
```

⭐⭐⭐⭐ **O plano NÃO morre mais — a cura do §2-bis funciona, e o que o dono vê
não é a tinta velha a desaparecer: é a tinta NOVA a nunca acontecer.** O
pen-down pica a superfície uma vez (`sculpt_at`), o raio **erra**, o gesto
inteiro vira `Drag::Orbit`, e o dedo entra na peça **sem traço nenhum aberto**.
Do lado do artista as duas coisas leem-se com a mesma frase.

⛔⛔ **E as duas metades do §2-bis são complementares, não redundantes:** a
primeira impede que o gesto abandonado **leve o plano com ele** (o vazamento), e
esta faz o gesto **existir**. Curar só a primeira deixa o report vivo com outra
cara — que é exactamente o que aconteceu.

### §10.2 — Porque os gates existentes estavam VERDES

- `um_pen_down_que_erra_a_peca_nao_fica_com_o_plano` — ele mede o **PLANO** (que
  sobrevive) e nunca pergunta se o traço PINTOU. *Uma régua que mede o recurso
  não vê o gesto que não aconteceu.*
- `dois_tracos_de_cor_com_dyntopo_nao_perdem_o_detalhe_do_primeiro` — os **dois**
  traços dele começam DENTRO da peça ⇒ **a fixtura não contém o fenómeno**.

⚠️ É a mesma forma que o §2-bis já pagou, com os papéis trocados: lá uma fixtura
**errada** percorreu o caminho do defeito por acaso; aqui duas fixturas
**certas** não o percorrem nunca.

### §10.3 — A cura (F1): um pincel de COR não precisa de acertar no pen-down

[`input_down.rs`](../../../crates/ph2d-app-sculpt3d/src/input_down.rs), no braço
da decisão:

```rust
if took || scene.brush.verb.paints_color() {
    scene.drag = Some(Drag::Sculpt);
} else {
    scene.brush.verb = verb;
    scene.close_stroke();
    scene.drag = Some(Drag::Orbit);
}
```

⭐ **É seguro porque o `stroke_anchor` já é escrito ANTES da decisão** (linha
`384`): o traço tem âncora mesmo sem o 1.º dab ter pegado, logo os dabs
seguintes entram pelo caminho normal.

⛔ **TROCA DECLARADA, e é de produto:** com um pincel de cor na mão, arrastar no
vazio **deixa de orbitar** — o gesto que o próprio ficheiro chama de *«o mais
comum do mundo»*. O botão direito continua a orbitar (linha `445`, intocada), e
é isso que torna a troca pagável. **Se o dono preferir o contrário, a cura é uma
linha.**

⚠️ **O predicado é `paints_color()` e não o grip nem o verbo:** a pergunta é
*«este pincel deposita no canal de COR?»*, que é exactamente a população para
quem o pen-down errado é inofensivo — um verbo de FORMA que erra não tem o que
mover, e tirar-lhe a órbita seria retirar uma afordância sem comprar nada.

### §10.4 — O gate que faltava

`um_traco_de_cor_que_comeca_fora_da_peca_pinta`
([`tinta_no_produto_tests.rs`](../../../crates/ph2d-app-sculpt3d/src/tinta_no_produto_tests.rs)),
`#[ignore]` + placa, com o **CONTROLO dentro**: a seguir ao traço de cor ele
troca para `Verb::Draw` e exige que o mesmo gesto **não mova um vértice** —
*sem essa metade, a cura podia ter comido a órbita de toda a gente e o gate
ficava verde*. Corrida: **`6 tests run: 6 passed, 421 skipped`**.

⛔⛔ **E um gate que só vive em `#[ignore]` de GPU não é gateável por este
repo** — nem o arnês de mutação (`--lib`, sem `--ignored`) nem o CI o correm,
que é a armadilha que as M19–M22 desta mesma jornada pagaram. ⇒ a cura ganha o
**6.º ELO** no censo de texto (`tinta_fiacao_tests.rs`, `M26`), com as duas
metades de sempre: a agulha no código **e** ausente da prosa.

### §10.5 — O 2.º achado da auditoria: LATENTE, medido, NÃO curado

⚠️ **O empréstimo do plano é por PEÇA ACTIVA, e nada prende as duas pontas à
mesma peça.** O pen-down faz
`empresta(&mut scene.objects[scene.active].tinta)` e o `close_stroke` devolve a
`scene.objects[scene.active]` — se o índice `active` mudar **entre** o pen-down
e o pen-up, o plano da peça A aterra na peça B e a A fica sem ele (⇒ o `garante`
reconstrói-a grosso: **o mesmo sintoma do report, por outra porta**).

⭐ **Hoje é inalcançável por gesto**, e é por isso que fica declarado em vez de
curado: o `a_stroke_belongs_to_the_piece_it_started_on` já proíbe que um
consumidor de `pick` mova a peça activa a meio de uma pincelada, e a Hierarquia
só a troca **na mudança de selecção**. ⛔ *Mas a cerca que o protege é de OUTRO
assunto* — ela existe contra um pânico de índice, não contra este. A cura
honesta é o empréstimo carregar **quem o emprestou** (um índice dentro do
`SculptStroke`), e é uma wave pequena com gate próprio.

### §10.6 — Lente 2 (costura de UI): LIMPA

Varridos os controlos que esta jornada tocou (a fileira `Paint Detail`, os
quatro/cinco chips e a caixa de cor): **pintados, hit-indexados, no `populate`
e com braço no despacho** — o censo derivado `populate_censo_tests` fecha os
dois sentidos e nenhuma das duas metades acusa. *Nada a reportar é um
resultado, e escrevê-lo é o que impede a próxima janela de o re-varrer.*

### §10.8 — ⛔⛔ E a cura TIROU o fenómeno da fixtura do gate do §2-bis

`um_pen_down_que_erra_a_peca_nao_fica_com_o_plano` mede o **vazamento** do
plano num gesto abandonado — e a cena `=52` tem o **pincel de COR** na mão.
Depois do §10.3 um pincel de cor **já não abandona** nada ⇒ *aquela metade do
gate deixou de percorrer o braço da recusa*, e ela ficaria **VERDE a afirmar
nada**: a mutação `M25` (apagar o `close_stroke` daquele braço) passaria a ser
invisível ao comportamento, sobrando só o elo textual.

⭐ **A cura é a FIXTURA e não a barra:** a metade do vazamento troca para
`Verb::Draw` antes de errar, porque *a população que ainda orbita é a dos
verbos de FORMA* — o controlo (um traço que acerta, com o pincel de cor da
cena) fica onde estava.

⚠️⚠️ **É a lei que a `=45` já pagou, escrita no §29 daquela jornada:** *uma
cena corrigida deixa de conter o fenómeno, e um gate cuja fixtura deixou de o
conter não afirma nada.* Aqui quem tirou o fenómeno não foi uma cena — foi a
**cura de outro defeito no mesmo ficheiro**, no mesmo dia.

### §10.8-bis — ⛔⛔ E o PORTÃO apanhou um SEGUNDO gate com a premissa morta,
### na SHELL

`the_left_button_sculpts_where_it_hits_and_orbits_where_it_misses`
([`shells/desktop/tests/it/the_sculpt_gesture_is_wired.rs`](../../../shells/desktop/tests/it/the_sculpt_gesture_is_wired.rs))
lê o CORPO do `pointer_down` por texto e procurava a agulha **`if took {`** —
que a cura reescreveu. ⇒ `1` vermelho em `18 560`, e ele **só aparece no
portão**: vive em `shells/desktop/tests/it/`, que o laço interno desta linha
nunca corre (a cegueira que o §5.0 do roteador nomeia, aqui pela enésima vez).

⭐ **A lei dele continua INTEIRA — para os verbos de FORMA**, que são a
população onde *«arrastar no vazio = órbita»* é a afordância; o que mudou foi a
agulha, reescrita **com a morte da premissa visível no diff** e com uma metade
NOVA: `Drag::Sculpt` tem de aparecer **uma vez só**, senão a cura podia ter sido
um segundo ramo com ordem própria e a régua da ordem não o veria.

⚠️⚠️ **O próprio ficheiro já registava duas expirações anteriores da mesma
agulha** (*«a terceira vez nesta sessão que um proxy expirou»*, escrito quando
o `Grab` trouxe a segunda porta de pick). *Um gate que lê o corpo de uma função
por texto envelhece com cada linha que essa função ganha — e esse é o preço
que se paga por ele alcançar o que um teste não alcança.*

### §10.9 — A prova de mutação

O arnês de [`muta_a_metade_visivel.sh`](../ferramentas/muta_a_metade_visivel.sh)
passou de `25` para **`27`** casos: **`26` SANGRAM** e a única que sobrevive é
o **CONTROLO** (`M15`, uma linha em branco, que não pode sangrar).

- **`M26`** (`if took || …paints_color()` → `if took {`) **SANGRA** — e sangra
  pelo **6.º ELO** do censo de texto, não pelo gate de produto, porque este é
  `#[ignore]` + placa.
- **`M25`** continua a sangrar; ⚠️ e desde o §10.8 ela volta a ser observável
  também pelo **comportamento**, que é o que a emenda da fixtura comprou.

⚠️ **E o arnês teve de ser corrido SOZINHO.** Encadear os dois (`A; B`) numa
invocação só põe os **dois** debaixo do MESMO prazo de 30 min do cgroup, e a
`A` sozinha mede **~25 min** (`27` casos × ~60 s) ⇒ a `B` seria morta **a meio
de uma mutação**, que é exactamente o incidente do §8-bis. *Um prazo é por
INVOCAÇÃO, e encadear dois trabalhos longos não soma prazos — divide um.*
⭐ Quando a corrida encadeada foi interrompida, o `trap restore EXIT` **correu**
e a árvore ficou byte-idêntica ao backup (conferido com `diff -r`); mas ⚠️ o
subshell do `$(corrida)` **sobreviveu ao pai** e continuou a correr o `nextest`
— a lei do reparentamento, aqui com a forma mais barata.

### §10.10 — O portão de fecho, e o ACHADO de vassoura (NOMEADO, não curado)

| régua | resultado |
|---|---|
| `nextest-impacted.sh` | **18 560 / 18 560** (1.ª corrida: `1` vermelho, o §10.8-bis) |
| censos da árvore COMBINADA | **127 / 127**, com o controlo do filtro `12 de 12` |
| clippy `-D warnings` (4 crates) | **zero** |
| `cargo fmt --all --check` | limpo |
| mutação (visível) | **26 de 27** (a 27.ª é o CONTROLO) |
| mutação (upload) | **6 de 7** (a 7.ª é a `U3`, NOMEADA) |
| gates de GPU da tinta fina | **6 / 6** com adaptador |

⚠️ **VASSOURAS: nove das dez ficam IDÊNTICAS ao merge-base; UMA diverge, e a
divergência é o NOME DE UM GATE NOSSO.** Medido ficheiro a ficheiro contra o
`HEAD` (`base` contra `agora`, a mesma vassoura): a `blender-trim` passa de `0`
para `3` linhas, e as três são o token **`sculpt_gesture`** — que é o nome do
ficheiro `shells/desktop/tests/it/the_sculpt_gesture_is_wired.rs`, **presente
no merge-base** (`git cat-file -e` confirma) e criado numa wave de
infra-estrutura de testes.

⛔ *Os hits novos são o handoff a CITAR esse ficheiro.* Não há cura que não seja
renomear um gate nosso por causa de uma coincidência de token, e **a triagem de
uma vassoura é do R, nunca da janela I** — fica NOMEADO, com a medição, como o
`tip_roundness` de 14/09.

### §10.7 — ABERTO desta secção

- ⛔ **A leitura de *«mesmo se [não] tocar»* precisa do veredito do dono** — e
  com ela a troca do §10.3 (arrastar no vazio com um pincel de cor deixa de
  orbitar).
- ⏳ O empréstimo por `active` (§10.5), latente e nomeado.
- ⏳ O **SMOKE**: a `=52` passa a ter um passo que começa o traço FORA da bola.

---

## §11 — «A TINTA SÓ É DEPOSITADA SE O PINCEL ESTÁ SOBRE UM VERTEX»

> *«o problema da tinta sumindo já foi resolvido, mas a tinta só é depositada
> se o pincel está sobre um vertex»* — o dono, 2026-09-21, o **terceiro** report
> do dia, depois do §10.

⭐ **A primeira metade do report é o veredito do §10:** o plano deixou de morrer.
A segunda é um defeito **separado**, que o §10 tornou visível ao devolver o
traço a quem começava fora da peça.

### §11.1 — A reprodução, e a régua é o MAPA e não a quantidade

A sonda `diag_a_tinta_so_cai_onde_ha_vertice` varre o ecrã de 5 em 5 píxeis com
**um clique em cada sítio** e imprime `#` onde alguma amostra foi pintada:

```
a peça tem 738 vertices e 768 faces; aresta média 0,1304
raio  6 px  pintou em  3/61  ...#....................................................##...
raio 12 px  pintou em 23/61  ..###..###...........###....####....###...........###..###..#
raio 24 px  pintou em 61/61  #############################################################
raio 48 px  pintou em 61/61  #############################################################
```

⛔⛔ **A régua tinha de ser a FRACÇÃO DE SÍTIOS e não a contagem de amostras:**
o defeito é um mapa em **ILHAS**, e somar as amostras da varredura inteira
esconde-o atrás dos sítios que funcionavam. *O mapa das posições que pintam é a
rede de vértices, e é isso que o dono descreve.*

### §11.2 — ⛔⛔⛔ A minha primeira atribuição estava ERRADA, e a medição disse-o

Escrevi a cura no `if self.moved.is_empty()` do fim do dab — a cerca que conta
os vértices que o carimbo MOVEU — e a re-medição devolveu **o mapa idêntico,
carácter a carácter**. *Uma cura que não move a régua não é a cura*, e a
segunda tentativa (a cerca da PEGADA, no princípio do dab) deu exactamente o
mesmo.

⭐⭐ **Quem resolveu foi o RASTO, não o raciocínio:** um `eprintln!` atrás de
`PH2D_DAB_TRACE` em cada cerca mostrou o dab a passar as duas e a morrer numa
**terceira** — a da máscara de alcance, que corre depois delas:

```
[trace] dab_core: r=0,0315 footprint=0 so_amostras=true fina=true
[trace] pos-mascara: footprint=0 (surface_only=true)     ← 96 das corridas
[trace] chegou ao ramo da COR fina                       ← 148
```

⚠️ **São TRÊS cercas de vértices em fila** (`pegada` · `máscara` · `movidos`), e
cada uma sozinha lê-se como *a* causa. *Uma cadeia de guardas só se ataca com
um instrumento que diz em qual delas se morre.*

### §11.3 — A máscara decide quando tem PROVA

A cerca da máscara fica — mas com a pergunta certa: se ela **cortou** vértices e
não sobrou nenhum, ela decidiu, e o dab morre para todos. Se a pegada **já
estava vazia**, ela não julgou nada — não havia um único vértice sobre que
julgar — e a unidade que sobra é a **amostra**.

### §11.4 — ⛔⛔ E abrir essa cerca obrigou a uma segunda cura, porque a máscara
### já era INERTE para a cor fina

O A/B (`diag_a_mascara_ainda_decide_na_tinta_fina`) sobre a barbatana de `0,06`
de espessura, o MESMO traço com a máscara armada e desarmada:

| raio | armada | desarmada |
|---|---|---|
| 64 px | `66 179` | `66 179` |
| 24 px | `13 035` | `13 035` |
| 10 px | `2 536` | `2 536` |

⛔⛔⛔ **Iguais aos três raios: a `Connected Only` — que o dono mandou shipar
LIGADA — não cortava uma única amostra de cor fina.** A cura do report de 19/09
(*«Snake Hook … deformando a face POSTERIOR»*) protege o VÉRTICE, e a tinta fina
escreve AMOSTRAS: *o pincel já pintava as costas de uma parede fina, e ninguém
tinha medido*.

⇒ a lei da folha passa a valer para a amostra. Depois da cura:

| raio | armada | desarmada |
|---|---|---|
| 64 px | **`33 841`** | `66 179` |
| 24 px | **`7 603`** | `13 035` |
| 10 px | `2 536` | `2 536` ← **o CONTROLO** |

⭐ **A linha do raio pequeno é o controlo que dá direito às outras duas:** ali a
esfera do dab não alcança as costas, logo não há o que cortar — e as duas
colunas TÊM de ler igual.

⚠️ **Das três condições daquela máscara só a NORMAL se transplanta:** a
conectividade e a razão `superfície/ar` são um passeio por ARESTAS, e uma
amostra não tem vizinhos no grafo.

### §11.5 — ⛔⛔ E a transplantação custou DUAS correcções, as duas por gates vermelhos

1. **O ARMAR.** A máscara só corta *«se alguma coisa na pegada estiver virada ao
   artista»*, e a minha 1.ª redacção largou essa metade ⇒ **seis** gates de lei
   reprovaram com *«o traço não tocou amostra nenhuma»*: as fixturas daquele
   ficheiro constroem o dab com `Dab::at(c, r, c)` — **o olho é RADIAL** —, e
   sem o armar toda a gente lê *virada ao contrário*.
2. **A POPULAÇÃO do armar.** Com o armar escrito mas a varrer **todas** as
   amostras colhidas, os mesmos seis continuaram vermelhos: a consulta de
   amostras é por **CAIXA** (`faces_in_sphere`) e traz faces do outro lado da
   peça, enquanto a pegada de vértices é uma **ESFERA**. Uma amostra da face
   oposta armava a cerca e as do cursor eram todas cortadas. ⇒ o armar conta só
   quem está **dentro** do raio.

⭐⭐⭐ *Uma lei transplantada tem de trazer a POPULAÇÃO dela junto — e as duas
correcções foram achadas por gates que já existiam, não por leitura.*

### §11.6 — ⛔ E uma coisa que eu MEDI e NÃO curei: o `Ctrl+Z`

`diag_o_ctrl_z_desfaz_a_tinta_fina`, pelo caminho do produto (a tecla, pela
porta que a shell chama):

```
antes do traço:  0 amostras pintadas
depois do traço: 1010
Ctrl+Z consumido=true; depois do desfazer: 1010
```

⛔⛔ **Com o plano armado o desfazer não devolve uma única amostra.** A entrada
de desfazer é a janela de **vértices tocados** (`touched` + `base_colors`), e a
cor fina não escreve no canal por-vértice — o `close_stroke` até sai cedo quando
essa janela está vazia.

⚠️⚠️ **E o roteiro da `=52` dizia o contrário** (*«a tinta volta atrás, passo a
passo, **com plano ou sem ele**»*) — a espécie que o §5.0 do roteador chama de
**pior que uma cena ausente**. O passo (7) foi corrigido para dizer o que
acontece e para o dono **não** o reportar como regressão.

⭐ **O material da cura já existe e está pago:** o `TintaDoTraco` guarda
`tocadas` (as amostras do traço) e `base` (a cor delas antes) — é a janela, com
a mesma forma da que o canal por-vértice usa. Falta a variante de `StrokeUndo` e
o `swap` involutivo que serve desfazer e refazer com um buffer só. **Wave
própria, nomeada, com o material identificado.**

---

## §12 — O `Ctrl+Z` DESFAZ A TINTA FINA — o QUARTO canal

A wave que o §11.6 deixou **nomeada com o material identificado**, feita a
seguir ao smoke aprovado. Ela é o único item da lista de abertos desta jornada
que era **medido e não curado**.

### §12.1 — O que a medição dizia, e o que ela diz hoje

A sonda `diag_o_ctrl_z_desfaz_a_tinta_fina` (versionada desde ontem, pelo
caminho do produto — a tecla, pela porta que a shell chama):

| | antes do traço | depois | depois do `Ctrl+Z` |
|---|---|---|---|
| **21/09, de manhã** | `0` | `1010` | **`1010`** |
| **21/09, depois desta wave** | `0` | `1010` | **`0`** |

⭐ **O MESMO instrumento dos dois lados da cura.** *É por isto que uma sonda se
versiona em vez de se apagar* — a próxima janela que duvide do número corre-a
em vez de o re-derivar.

### §12.2 — A causa, e porque ela não era um esquecimento

A entrada de desfazer de um traço é `StrokeUndo::Stroke`, e os três canais dela
(`positions` · `masks` · `colors`) são indexados pelo **MESMO** `verts`, que é a
janela de **vértices tocados**. A tinta fina escreve **AMOSTRAS**, cuja unidade
de endereço é `(face, sítio)` e cuja população não é a dos vértices — depois da
cura de 21/09 um dab fino escreve amostras **sem um único vértice na pegada**.

⇒ o quarto canal **não cabia** nos três: ele carrega os **próprios índices**.

E havia um segundo elo, que é o mesmo portão que esta jornada já corrigiu três
vezes noutro ficheiro:

```rust
if self.stroke.touched().is_empty() {
    return;                       // ← o QUARTO portão a contar VÉRTICES
}
```

Um traço de cor fina entre dois vértices saía daqui **sem entrada nenhuma**, e o
`Ctrl+Z` gastava o passo **ANTERIOR** — pior do que não fazer nada.

### §12.3 — A cerca, e porque ela é exactamente tão forte quanto a do produto

O plano é paramétrico nas faces. Escrever a cor de antes num plano
**reconstruído** põe a tinta de uma face na face vizinha — o defeito mudo que o
cabeçalho da [`tinta_da_peca`] narra inteiro. ⇒ a janela carrega a identidade do
plano em que foi escrita ([`IdDoPlano`]: `verts` · `faces` · `nivel`).

⚠️ **Ela é deliberadamente tão forte quanto a [`concorda_com`]** — a régua com
que o PRODUTO decide se o plano vivo sobrevive ao quadro seguinte. Uma cerca
mais apertada aqui seria uma **segunda resposta** à mesma pergunta; uma mais
frouxa escreveria onde o produto já não escreve.

⛔⛔ **E há uma SEGUNDA cerca, que responde a outra pergunta: «os meus índices
cabem?»** A [`swap_window`] indexa **sem cerca nenhuma**, e um `panic` num
`Ctrl+Z` é o pior desfecho possível de um canal de desfazer.

⚠️⚠️ **A 1.ª redacção metia a contagem de amostras DENTRO da identidade, e uma
MUTAÇÃO SOBREVIVENTE mostrou porque isso não se pode:** dado
`(verts, faces, nivel)`, a contagem é **derivada** — se as três batem, a
`concorda_com` aceita o plano e ele **não é reconstruído** ⇒ *um campo
redundante numa igualdade é um campo que nenhuma fixtura consegue pôr a
decidir*. E a fixtura que eu escrevera para o testar **sobrescrevia o próprio
campo** que media, logo a mutação que o apagava passava verde.

⇒ hoje são duas cercas separadas, e a dos índices tem gate que a alcança **por
construção** (identidade a bater, índice fora). Ela é **inalcançável pelo
produto** e fica na mesma, porque *a alternativa é uma afirmação sobre a malha
com um `panic` à espera*.

### §12.4 — ⛔ LARGAR é a resposta certa, e carregar seria um defeito de DIRECÇÃO

Quando a cerca recusa, a janela é **largada** e a inversa não a leva.

A alternativa óbvia — carregá-la inalterada para a fila oposta — é **errada** e
o mecanismo é aritmético: uma entrada carrega o estado de **ANTES**; quem a
aplica devolve o de **DEPOIS**, e é esse que o refazer instala. Uma janela que
não se pôde aplicar **não tem o «depois»** ⇒ devolvê-la a ela própria poria o
`Ctrl+Shift+Z` a instalar as cores de antes **outra vez**, ou seja a desfazer
duas vezes — e só no dia em que o artista voltasse a armar o mesmo degrau.

*Um payload que sobrevive à recusa é pior que a recusa.*

### §12.5 — O que NÃO foi preciso construir

⭐ **O canal por-vértice já estava coberto, e por construção.** A
[`tinta_da_peca::devolve`] corre no topo do `close_stroke` e reescreve
`Mesh::colors` a partir do plano; só **depois** disso é que o
`color_window_changed()` é avaliado. Logo o terceiro canal já grava o que a
tinta fina mudou na cor grossa.

⚠️ E a população fecha: um vértice cujo *corner sample* mudou está, por
construção, dentro do raio do dab — logo está na pegada. Com a pegada vazia,
nenhuma cor por vértice mudou.

### §12.6 — A lei vive FORA da cena, e isso é a decisão

[`history_tinta_fina.rs`] não conhece a `Sculpt3dScene`: ela toca numa
[`Tinta`] e em mais nada. É isso que dá **quatro gates que correm sem
adaptador** — e a razão é a mesma que a §33 desta linha já pagou: *quando um
gate precisa de um device para medir uma decisão que não tem pixel nenhum, a lei
está no sítio errado.*

Os gates de PRODUTO (`#[ignore]` + placa) são dois, e o segundo é o que prende a
cura do portão:

- `o_ctrl_z_desfaz_a_tinta_fina` — desfazer devolve o plano de antes **e**
  refazer devolve o de depois, **ao bit**. ⚠️ As duas metades: sem o refazer,
  um desfazer que pintasse tudo de branco passaria.
- `um_traco_de_cor_sem_vertice_debaixo_do_pincel_deixa_desfazer` — a fixtura sai
  de uma **medição** (`diag_onde_a_pegada_de_vertices_fica_vazia`: a `raio 6 px`
  a pegada de vértices lê `0` em `30` dos `31` sítios varridos), e a 1.ª
  asserção é o **CONTROLO** de que ela ainda contém o fenómeno — *um raio maior
  torna este gate verde a medir outra coisa*.

### §12.7 — Os TRÊS elos de texto, e porque eles são precisos

Os gates puros chamam a porta **DIRECTAMENTE**: eles ficam verdes sobre um
produto que nunca a chama. A prova de comportamento até à tecla é `#[ignore]` +
placa, que **nem o arnês de mutação nem o CI correm**.

⇒ o censo da fiação passa de **NOVE para DOZE** elos (`M30` a colheita ·
`M31` o portão · `M32` a aplicação). *É a armadilha que as M19–M22 desta mesma
jornada já tinham pago, e agora ela é prevista em vez de descoberta.*

⚠️ E a colheita é feita **DENTRO** do `if let` que consome o plano, e não numa
linha própria antes do `take`: escrita antes, ela compila e sobrevive a um
reordenamento que a deixe **depois** — e depois do `take` o `Option` está vazio,
logo a janela viria sempre vazia, **em silêncio**.

### §12.8 — O roteiro da `=52` foi corrigido OUTRA VEZ, no sentido inverso

O passo (7) tinha sido corrigido ontem para dizer que o `Ctrl+Z` **não** desfaz
a tinta fina. Hoje ele diz o que acontece, e **acrescenta a fronteira medida**:

> ⚠️ Se entre o traço e o `Ctrl+Z` você TROCAR a fileira `Paint Detail`, aquele
> traço deixa de poder voltar: o plano de amostras foi refeito e o de antes já
> não existe.

⚠️ **E o censo derivado do roteiro apanhou-me:** `Ctrl+Shift+Z` não é um rótulo
que o painel pinte, e ele reprovou na primeira corrida. A cura é a entrada na
lista `TECLAS` — que tem a metade que a impede de ser licença (uma tecla que
seja **também** um rótulo pintado reprova).

### §12.8-bis — ⛔ O ARNÊS DE MUTAÇÃO JÁ NÃO CABE NO PRAZO DA FATIA

Com as seis mutações novas a rede tem **35**, e a corrida de `nextest` custa
`~50 s` ⇒ **~30 min**, que é exactamente o prazo do `ph2d-run.sh`. A corrida que
as levou todas foi **MORTA na M25 com a árvore inteira** — o risco de mutação
CONGELADA que o §8-bis desta jornada já registou.

⭐ **A árvore foi conferida e está limpa** (zero substitutos de mutação vivos,
`git diff` das outras duas crates a zero), e a cura é `MUTA_FILTRO=<regex>`:

```
MUTA_FILTRO='^M3[0-5] ' bash docs/3D/ferramentas/muta_a_metade_visivel.sh
```

⚠️ **E o sumário DIZ o filtro**, com a contagem a descrever a população que de
facto correu — *um placar parcial lido como completo é a forma mais barata de
um arnês mentir*, e este ficheiro já tem quatro registos de arneses a mentir.

### §12.8-ter — O placar, em três fatias

| fatia | filtro | placar |
|---|---|---|
| a rede de ontem, 1.ª metade | `^(M[1-9] \|M1[0-4])` | **15 de 15** |
| a rede de ontem, 2.ª metade | `^M(1[6-9]\|2[0-9]) ` | **14 de 14** |
| **as seis do quarto canal** + o controlo | `^(M15 \|M3[0-5] )` | **6 de 7** |
| **as duas do empréstimo por dono** + o controlo | `^(M15 \|M3[6-7] )` | **2 de 3** |

⇒ **37 de 38 sangram**, e a 38.ª é o `M15` — o CONTROLO, que é uma linha em
branco e **não pode** sangrar.

⚠️ A rede de ontem foi **re-corrida inteira** de propósito: este diff toca no
`history.rs`, no `undo.rs` e no censo da fiação (que passou de nove para doze
elos), e *um placar herdado é um placar sobre outra árvore*.

### §12.9 — ABERTO

- ⏳ A **permutação** da janela quando um colapso renumera a malha: a entrada
  guarda índices de AMOSTRA, e a cerca da identidade recusa qualquer plano
  reconstruído ⇒ o caso não é alcançável hoje. *Se um dia o plano sobreviver a
  uma mudança de topologia, esta janela precisa da cadeia de renumeração* — que
  é a lei do §27, e que já mordeu duas vezes nesta linha.
- ⏳ O plano continua a **não viajar no `.ph2dproj`** (dívida herdada).

---

## §13 — O EMPRÉSTIMO PASSA A CARREGAR QUEM O EMPRESTOU

O item **LATENTE** que o §10.5 deixou medido e declarado, e que o handoff
prescreveu por escrito: *«a cura honesta é o empréstimo carregar quem o
emprestou»*.

### §13.1 — O defeito, e porque ele era latente e não inofensivo

O empréstimo tinha **duas pontas** e ambas liam `objects[self.active]`:

```
pen-down   →  empresta(&mut scene.objects[scene.active].tinta)
close      →  devolve( … objects[self.active] … )
```

Nada as prendia à MESMA peça. Com o índice a mudar entre as duas, **o plano da
peça A aterra na B**, a A fica sem ele, e a `garante` reconstrói-a semeada da
cor por vértice ⇒ *a tinta a voltar à resolução da malha* — **o mesmo sintoma
do report do dono, por outra porta**.

⛔ **A cerca que o protegia é de OUTRO assunto:** o
`a_stroke_belongs_to_the_piece_it_started_on` existe contra um **pânico de
índice**, não contra isto. *Um invariante mantido por uma cerca escrita para
outra pergunta é um invariante a prazo.*

### §13.2 — A forma: DENTRO do empréstimo, nunca ao lado dele

O `dono` é um `u32` **opaco** dentro do [`TintaDoTraco`], e esta crate nunca o
interpreta — do lado da família ele é o [`ObjectId`].

⛔⛔ **A alternativa óbvia — um `Option<ObjectId>` na cena ao lado do
`Option<TintaDoTraco>` no traço — são DOIS campos que têm de concordar**, e
esta casa já pagou essa forma meia dúzia de vezes. Aqui a pergunta *«de quem é
este plano?»* tem **uma** resposta, e ela **morre com o plano**.

⭐ E a volta é uma PORTA ([`tinta_da_peca::devolve_ao_dono`]) que acha a peça
pelo id. ⚠️ **Se ela já não existe, o plano morre com ela, e isso é a resposta
certa** — um plano é paramétrico nas faces de UMA malha, logo não há segunda
peça a que pudesse pertencer; ⛔ e a porta **devolve um booleano**, porque
*largar um plano sem ninguém saber é exactamente como ele se perdia antes*.

### §13.2-bis — ⚠️ PROMOÇÃO PEDIDA à lista de flakes do §5.0

`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
([`ph2d-tool-painter`](crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs))
reprovou no meio de um fan-out de **18 566** testes e passa **3 de 3 sozinho a
`load 41,6` · `30,0` · `30,0`** — *mais carga do que teria tido isolado na
corrida que o reprovou* —, com **zero linhas** do diff desta linha naquela
crate.

⛔ **É o SÉTIMO deste repo cujo doc-comment se declara imune por escrito**, e
aqui a declaração é a mais explícita de todas: ela narra que uma redacção
anterior flakou e foi curada medindo os dois lados **no mesmo instante**
(*«um gate que flaka é pior que ausente»*) — ⚠️ **verdade sobre a DERIVA da
máquina e falsa sobre o FAN-OUT**, que é exactamente a distinção que aquela
lista existe para guardar. *Dividir dois relógios não deixa de ser um relógio
por a razão ser adimensional.*

### §13.2-ter — ⛔ QUATRO vermelhos de RETOPOLOGIA, MEDIDOS como pré-existentes

A bateria larga (`--run-ignored all`, `441` testes) devolve **11** vermelhos, e
nenhum é desta wave — mas *atribuir um vermelho por inferência é o que esta
casa proíbe*, e a nota que o diz está escrita em `project-memory`.

- **7 são `TIMEOUT` a `180 s`**: SONDAS da bancada de retopologia (o
  `what_does_proving_the_optimum_cost`, o `how_fine_can_the_global_chain_go`…),
  pesadas por construção e mortas pelo tecto de tempo da casa quando corridas
  num fan-out de 441.
- **4 são gates de FORMA do quad remesh**, e falham **sozinhos a `load 7,11`**
  ⇒ deterministas, não flakes. Um deles é o `the_quads_are_as_square_as_the_oracles`,
  que o §5 do roteador **já documenta como vermelho com esse endereço**.

⭐ **A medição que decide:** os quatro foram corridos na árvore **de ANTES desta
wave** (`git checkout HEAD~1 -- crates/`, a `load 3,95`) e reprovam **os
quatro, com os mesmos números** (`ORELHA: enviesamento mediano 27°, barra 10°`).
⇒ **pré-existentes**, e o `git checkout HEAD -- crates/` devolveu a árvore.

⚠️ **E a população certa desta wave é OUTRA:** os gates de GPU da tinta fina
correm **41 de 41 verdes**, com os dois novos lá dentro. *Correr
`--run-ignored all` numa crate que hospeda uma bancada de oráculo mede a
bancada, não a wave.*

### §13.3 — A régua, e porque ela precisa de ser um ELO

Os dois gates da lei são **puros** (`SceneObject::new` não pede device) e o
segundo deles é o CONTROLO — *sem a asserção de que a peça activa fica **sem**
plano, um `devolve_ao_dono` que o desse aos dois passaria*.

⛔ **Mas eles chamam a porta DIRECTAMENTE**, e o `close_stroke` pode voltar a
`objects[self.active]` sem que nenhum gate de produto o veja — *porque hoje
nenhum gesto troca a peça activa a meio de um traço*. **É a definição de um
defeito latente: a régua que o apanha tem de ser o ELO.** ⇒ o censo da fiação
passa de **doze para CATORZE**, com `M36` (a devolução) e `M37` (o empréstimo a
deixar de carregar o dono).

### §13.4 — O PORTÃO desta corrida

| régua | resultado |
|---|---|
| `nextest-impacted` | **18 566 / 18 566** |
| `nextest` das duas crates do motor+família (`--lib`) | **766 / 766** |
| gates de GPU da tinta fina, com adaptador | **41 / 41** |
| censos da árvore COMBINADA | **127 / 127** · controlo do filtro `12 de 12` |
| `cargo fmt --all --check` | limpo (⚠️ e **todas** as agulhas do censo e do arnês conferidas depois dele) |
| clippy `--all-targets -D warnings` nas três crates | zero |
| mutação | **37 de 38** (a 38.ª é o CONTROLO) |
| as 10 vassouras sobre os 18 ficheiros do commit | **zero achados NOVOS** (`tip_roundness` e `sculpt_gesture` são linhas de ontem, as duas já nomeadas no §10.10) |
| tectos de LOC | o maior ficheiro tocado é o `history.rs` a **668** de `700` |

⚠️ **A 1.ª corrida da varredura impactada devolveu DUAS vermelhas**, e a
segunda é a promoção do §13.2-bis: o
`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` passa **3 de 3
sozinho** e a re-corrida do fan-out fecha **18 566 / 18 566** — *o conjunto de
reprovadas mudou entre duas corridas da mesma árvore, que é a assinatura que
separa um recurso partilhado de um defeito de lógica*.

⚠️ **E o índice da memória reprovou** (`22 150` contra `22 000`) — a cura é a
que o próprio gate prescreve: a entrada desceu para o
`reference_topic_code_pattern_gotchas`, e o índice fecha a `21 944`.

---

## §14 — O PÂNICO: `index out of bounds` no registo da tinta, e a cerca que faltava

### §14.1 — O report, e o que o NÚMERO já dizia

```
PH2D PANIC frame=10216 thread="main"
location="crates/ph2d-mesh-colors/src/topo.rs:239"
message="index out of bounds: the len is 196608 but the index is 196608"
```

A linha `239` é `self.lado_da_face[4 * f + s]`, e o vector tem **`4` entradas por
face**. ⇒ `196 608 = 4 × 49 152` e o índice é `4 × 49 152 + 0`, ou seja **`f`
valia exactamente a contagem de faces da topologia**: a lista de faces que
chegou ao registo tinha **mais faces do que o plano descreve**.

⭐ *O número dizia o mecanismo antes de qualquer leitura de código* — a única
maneira de `4f + s` cair no primeiro índice fora do vector é `f == faces()` com
`s == 0`.

### §14.2 — Reproduzido, e não inferido

Sonda pura, sem device: uma topologia nascida de **dois quads** e o payload
alimentado com os **quatro triângulos** da mesma malha triangulada.

| perfil | o que sai |
|---|---|
| `dev` | `debug_assert_eq!` dispara: *«o payload recebeu outra face»*, `left: 3, right: 4` |
| `release` | `topo.rs:239:38 — index out of bounds: the len is 8 but the index is 8` |

⇒ **a mesma linha, a mesma coluna, a mesma forma** (`len = 4 × faces`,
`index = 4 × faces`), com `2` faces em vez de `49 152`.

⛔⛔ **E ele só existe em RELEASE.** A única régua daquela linha era um
`debug_assert`, e o perfil `smoke` — que é o que o dono corre — não o compila.
*Uma promessa escrita num `debug_assert` é uma promessa que o produto não faz.*

### §14.3 — O caminho do PRODUTO, com os endereços

1. [`input_down.rs:322`] — o pen-down **empresta** o plano ao traço
   (`tinta_da_peca::empresta`). A topologia dele descreve a malha de AGORA.
2. [`input_down.rs:378`] — `open_dyntopo_stroke()`, **56 linhas depois**, chama
   `mesh_mut().triangulate()`: os dois motores de topologia recusam quads, e
   desde 2026-09-20 é o pen-down que herda esse trabalho (o `toggle_dyntopo`
   deixou de o fazer com o plano armado, para não apagar o detalhe fino ao
   LIGAR um interruptor). ⇒ **a contagem de faces dobra e o plano fica a
   descrever a malha de antes.**
3. [`slots.rs:170-180`] — o quadro reconcilia com a `garante`, mas **só na rota
   `Rota::DaPeca`**. Com o plano emprestado a rota é `Emprestado`, e o doc dela
   di-lo por escrito: *«não se reconcilia nada»*.
4. [`slots.rs:297`] — `upload_tinta_at(mesh de AGORA, plano de ANTES)` →
   `payload` → **pânico**.

⚠️ **E o `triangulate` do pen-down é só a PRIMEIRA porta:** o `refine_for_dab`
parte triângulos em triângulos **a cada dab**, e ali a contagem de faces sobe
com todos os cantos a `3`. *O pen-down explica o primeiro quadro; os dabs
explicam o quadro `10 216`.*

⛔⛔ **O cabeçalho da porta já escrevia o perigo e nada o media:** *«um plano da
malha de antes é tinta no vértice errado, e nenhuma contagem o vê»*. É a família
que esta casa chama de *promessa num doc-comment* — a 2.ª desta jornada.

### §14.4 — A cura, em três camadas e UMA lei

| camada | onde | o quê |
|---|---|---|
| **LEI** | `ph2d_mesh_colors::Topologia::descreve` | *«esta topologia descreve aquela malha?»*, pelas duas contagens, `O(1)` |
| **PORTA** | `Topologia::payload` | devolve **`bool`** e RECUSA, face a face, deixando `out` **vazio** |
| **DEVICE** | `MeshRenderer::upload_tinta_at` | **DESARMA** (`armado = 0`) em vez de construir |

⭐⭐⭐ **A lei mudou de casa, e essa é a decisão da wave.** A conta vivia no
`tinta_da_peca::concorda_com`, na crate da FAMÍLIA — e o pânico provou que ela
tem um **segundo leitor que aquela crate não alcança**: a porta do device, em
`ph2d-mesh-render`. *Uma lei escrita em dois sítios ainda não é uma lei; só uma
PORTA é.* Hoje o `concorda_com` **delega**, e o que sobra nele é a tradução de
`Tinta` + `Mesh` para as duas contagens.

⚠️⚠️ **E na porta do device são as DUAS perguntas, porque nenhuma cobre a
outra:** a `descreve` é a única que vê os **VÉRTICES** (um registo é feito de
faces, e o payload nunca os olha), e o payload é o veredito **forte**, o único
que separa duas malhas com as mesmas duas contagens. A `descreve` vem à frente
pela ordem barata — ela corta sem percorrer as faces todas, que é o caso comum
durante um traço de forma com o plano armado.

⚠️ **Desarmar é a resposta CERTA e não um remendo.** O plano já não descreve
esta malha, logo não há tinta fina que se possa desenhar; o device passa a
mostrar a **cor por vértice**, que é exactamente o que a `garante` reconstrói
assim que o traço larga o plano. *Meio quadro com a cor de baixa resolução é o
que já ia acontecer de qualquer maneira; um pânico é a sessão inteira.*

### §14.5 — As TRÊS recusas, e porque são três

| recusa | o que apanha | quem a produz no produto |
|---|---|---|
| `f >= self.faces()` | faces a **MAIS** com os mesmos cantos | o `refine_for_dab` de cada dab |
| `n != self.cantos_de(f)` | a mesma face com **outros cantos** | a triangulação do pen-down |
| `vistas != self.faces()` | faces a **MENOS** | um colapso |

⛔⛔ **A terceira é a que nunca estourou, e é a pior.** Com menos faces o laço
acaba sozinho, nenhum índice sai de alcance, e o registo fica **truncado**: o
shader lê o bloco de interior de uma face que já não está lá e pinta **tinta
válida no sítio errado, em silêncio**. É por isso que a régua é a IGUALDADE e
não um tecto.

### §14.6 — ⛔⛔⛔ A MUTAÇÃO ACHOU A FIXTURA A MASCARAR-SE

A 1.ª rede deu **5 de 8** com DOIS sobreviventes que não podiam sobreviver:
`N1` (apagar a recusa das faces a mais — o pânico) e `N4` (a recusa deixar o
registo meio escrito).

⭐ **A causa é uma só e é da FIXTURA:** ela media o caso do dono — quads
triangulados —, e ali a **primeira** face já é um triângulo onde o plano espera
um quad ⇒ a recusa dispara em `f = 0`, **pela cerca dos CANTOS**. Com ela a
disparar primeiro, apagar a cerca da CONTAGEM não muda um bit, e o `out` nunca
chega a ter nada dentro para o `clear` importar.

⇒ *duas cercas que se tapam uma à outra leem-se como uma cerca a funcionar.*

A cura é uma fixtura que **isola** cada cerca: uma topologia de `2` triângulos
alimentada com `4` triângulos. Ali as duas primeiras faces passam (o `out` fica
com `2 × PAYLOAD_STRIDE` palavras), a terceira sai de alcance, e as duas
mutações voltam a sangrar. ⚠️ **E o caso é REAL**, não um exercício: é o
`refine_for_dab`, que parte triângulos em triângulos.

⭐ O caso do dono fica com gate próprio
(`o_panico_do_dono_reproduzido_uma_peca_de_quads_triangulada`) e o doc dele diz
**qual** das três cercas ali dispara — *escrever só este caso deixaria a cerca
da contagem sem régua nenhuma*.

### §14.7 — E o arnês irmão perdeu duas âncoras, em voz alta

A delegação do `concorda_com` apagou o texto que as mutações `M3`/`M4` do
[`muta_a_metade_visivel.sh`] mutavam ⇒ as duas passaram a casar **zero** vezes.
⭐ **O controlo de âncora do arnês foi quem o disse** — *uma rede sem esse
controlo teria lido as duas como SOBREVIVENTES*, que é o placar perfeito e
fabricado que o §8-bis já registou noutra forma.

Elas mudaram de agulha e passaram a medir a **tradução** (`descreve(verts,
faces)` com um dos dois argumentos preso ao próprio plano): **2 de 2 sangram**,
sobre uma corrida limpa de `306` testes.

### §14.8 — O que NÃO foi construído, com o número ao lado

⛔ **Não borrar o plano quando o gesto vai mexer na topologia** — construído em
raciocínio e recusado por CUSTO: com o plano na peça, a `garante` do quadro
reconstrói-o **a cada dab que refina** (`72 MB` no degrau `8×` da peça de
fábrica, `288 MB` no `16×`). *A cura trocaria um pânico por uma paragem.*

⛔ **Uma cerca dentro da lei de cor** (`tinta_fina::aplica`, que indexa
`amostras_mut()[a.idx]`) — a mesma família de pânico é **inalcançável pela
rota**: com o plano armado um verbo de COR não mexe na topologia (a cura de
20/09), e entre dois traços há sempre um quadro em que a `garante` corre.
Fronteira **declarada**, não coberta.

### §14.9 — ABERTO desta secção

- ✅ **A lente estreita da VOZ** foi achada a medir esta cura e fechou no mesmo
  dia — §15.
- ⏳ A cerca dentro da lei de cor (§14.8), declarada e não coberta.

---

## §15 — E a VOZ tinha a lente mais estreita que o consumidor, nos DOIS sentidos

### §15.1 — O achado, e ele saiu de medir a cura do §14

A cura do pânico obriga a perguntar *quem muda a topologia debaixo do plano
emprestado?* A resposta é o [`open_dyntopo_stroke`], e ao lê-lo apareceu que a
**VOZ** que avisa o artista lê **outra coisa**:

| | o CONSUMIDOR (`open_dyntopo_stroke`) | a VOZ (`recusa.rs`) |
|---|---|---|
| interruptor | `dyntopo.armed \|\| verbo.corre_sem_o_interruptor()` | `dyntopo_armado` |
| pilha | `level_count() == 1` | *(não pergunta)* |
| o gesto mexe | `o_gesto_muda_a_topologia(…)` | idem |

⇒ **duas células erradas, com sinais OPOSTOS, no mesmo `if`:**

| configuração | o que acontece | o que a voz dizia |
|---|---|---|
| interruptor **OFF** + `Density` + plano armado | o plano é **refeito** | **calada** |
| interruptor **ON** + pilha de multiresolução | o passe **não corre** | **avisa** |

⚠️ *Um falso negativo e um falso positivo na mesma condição* — e nenhum deles é
visível a quem lê só um dos dois sítios. É a forma que o §5.0 desta casa chama
de **a lente do painel mais larga que a do consumidor**, aqui com os papéis
trocados numa metade e não na outra.

⭐ **E o `Density` não é um caso de canto:** ele é o único verbo que
`corre_sem_o_interruptor()`, por ordem do dono de 14/09 (*«Dynamic topology é
para os outros pincéis»*) — ou seja, **a configuração de fábrica** dele é
exactamente a célula muda.

### §15.2 — A cura é UMA PORTA com dois leitores

[`tinta_da_peca::o_passe_corre_no_pen_down(verbo, dyntopo_armado, niveis,
tinta_fina_armada)`] — pura, com as três metades, lida pelo
[`open_dyntopo_stroke`] e pela [`recusa::Entradas::recusa`].

⚠️ **O `dyntopo_armado` FICA na `Entradas`** e não foi substituído: a SEGUNDA
voz (*«a tinta fina dispensa a topologia»*) quer mesmo o **interruptor** — ela
é sobre o gesto que o artista acabou de fazer, não sobre o que o passe vai
fazer. *Duas perguntas parecidas com respostas diferentes continuam a ser duas
perguntas.*

⚠️ **A `Entradas` ganhou `niveis`**, e é a metade que faltava para a voz poder
calar-se com a pilha montada.

### §15.3 — As réguas

- `o_passe_do_pen_down_tem_as_tres_metades` — a porta **pura**, com as duas
  células do erro e **quatro** de CONTROLO (incluindo *«sem o plano armado um
  verbo de cor volta a mexer»*, sem a qual esta porta apagava a cura de 20/09).
- `o_pen_down_diz_o_preco_da_tinta_fina` — a voz pelo **produto**, com as duas
  células novas ao lado dos três controlos que já lá estavam.
- O censo textual passa a **DEZASSEIS** elos: o do pen-down mudou de agulha e o
  da voz **nasceu** (ela não tinha elo nenhum).

⛔⛔ **E o censo apanhou a mudança sozinho:** ao apontar o pen-down para a porta,
o `a_cura_da_tinta_fina_esta_ligada_nos_*` reprovou em voz alta com a agulha
antiga impressa. *Uma agulha que deixa de casar é o instrumento a funcionar —
foi assim que as âncoras `M3`/`M4` do arnês também apareceram.*

---

## §16 — E o CAMINHO RÁPIDO salta a porta: a cerca `!mexeu` não diz o que promete

### §16.1 — O achado, e ele também saiu de medir a cura do §14

O upload da tinta tem um atalho — *subir só as amostras que o traço escreveu* —
com três cercas, e o comentário delas nomeia o que cada uma assume:

> *«a topologia e as posições não mexeram (`!mexeu`), ninguém pediu o plano
> inteiro (`!tinta_suja`), e o device tem EXACTAMENTE este plano»*

⛔⛔ **A primeira é FALSA, e a prova está no próprio módulo:** `mexeu` é
`!dirty.is_empty()`, e o `mesh_rebuilt()` — que é quem TODA mudança de
topologia chama — faz `dirty.clear()` e `uploaded = false`. ⇒ *a linha que
regista «a topologia mudou» é a mesma que apaga a evidência de que alguma coisa
mudou.*

### §16.2 — Reachable, e por um gesto comum (medido no código, não inferido)

O pen-down de um verbo com **ÂNCORA** (`Grab` · `Snake Hook` · `Twist` ·
`Local Scale` · `Cloth`) **não carimba**: ele PEGA
([`input_down.rs:409`], *«o primeiro toque escolhe o ponto e não move nada»*).
⇒ com o plano armado e o passe a correr:

| passo | estado |
|---|---|
| `empresta` | o traço leva o plano da malha de ANTES |
| `open_dyntopo_stroke` → `triangulate` | a malha dobra de faces · `dirty` **vazio** · `uploaded = false` |
| `take_hold` | **não escreve um vértice** ⇒ `dirty` continua vazio |
| o quadro | `job = Full`, `mexeu = false`, `emprestado`, `!tinta_suja` ⇒ **o atalho dispara** |

E o atalho devolve `true` (o device tem `armado` da última subida e a contagem
de amostras **não mudou** — quem mudou foi a malha) ⇒ **`upload_tinta_at` nunca
é chamada, e a cerca do §14 nunca corre.**

⛔⛔⛔ **E o que se vê não é nada de bom:** a entrada de fragmento da tinta lê
`@builtin(primitive_index)` — o índice do triângulo da geometria **PRINCIPAL**,
que o `upload_at` acabou de renovar — e resolve-o contra o `origem`/`topo` da
tinta, que ficaram os de antes. *Duas tabelas a descrever malhas diferentes,
indexadas pelo mesmo número.*

### §16.3 — A cura: a cerca passa a dizer o que assume

O atalho ganha a quarta cerca — **o device tem de já ter ESTA malha**
(`!matches!(line.job, SlotJob::Full)`) —, e a decisão sai para uma função
**PURA** ([`tinta_da_peca::so_as_amostras_bastam`]), pela razão que este módulo
já pagou duas vezes: *quando um gate precisa de um `wgpu::Device` para medir
uma decisão que não tem pixel nenhum, a lei está no sítio errado.*

### §16.4 — As réguas, e o CORTE que elas obrigaram

- `o_atalho_do_upload_tem_quatro_cercas` — a porta pura, **uma entrada de cada
  vez** com as outras no valor que faz o atalho disparar. ⚠️ Com duas a mexer
  ao mesmo tempo, uma cerca apagada passa despercebida atrás da outra — que é
  exactamente a forma que a fixtura do payload pagou no §14.6, **no mesmo dia**.
- `o_laco_de_upload_pergunta_a_porta_se_bastam_as_amostras` — o ELO, com a
  prosa cortada antes de se medir, e a metade que exige que a quarta entrada
  venha do **JOB do slot**: *passar `false` ali deixaria a porta certa e a
  chamada errada.*

⛔ **E o `tinta_da_peca_tests.rs` estourou o tecto (`706` contra `700`)** — curado
por **CORTE e nunca por uma entrada no `FILE_OVERAGE_OK`**. O corte é por
ASSUNTO e ficou melhor do que o ficheiro era: lá **o PLANO** (como nasce, como
volta à peça, o que pesa, onde está o tecto), aqui **as PORTAS que decidem** —
e o cabeçalho do irmão novo diz porque elas são puras.

### §16.5 — ⛔⛔⛔ E a mutação achou a MINHA agulha a medir um fragmento

A `M41` (apagar a quarta cerca da porta) sangrou à primeira. A **`M42`** — *a
cerca fica certa e a CHAMADA passa-lhe sempre `false`* — **SOBREVIVEU**.

⭐ **A causa é a agulha:** o elo perguntava por `matches!(line.job,
SlotJob::Full)`, e esse texto aparece **DUAS vezes** no `slots.rs` (a outra é a
condição que decide se vale a pena subir alguma coisa) ⇒ trocar o ARGUMENTO por
`false` deixava a **outra ocorrência** a satisfazer o censo.

⇒ *uma agulha que é um FRAGMENTO mede a presença do fragmento, não a da
chamada* — a mesma lei que este repo já escreve para as âncoras de mutação
(*«âncora = expressão inteira»*), agora do lado do CENSO, onde ela morde mais:
**um censo é escrito precisamente onde não há comportamento para medir.**

Hoje a agulha são as **seis linhas** da chamada, e o gate imprime-as na falha.

### §16.6 — E os arneses ganharam PRÉ-VOO (`MUTA_SO_ANCORAS=1`)

⛔⛔ **`cargo fmt` reescreve a indentação de uma âncora, ela passa a casar
ZERO, e isso lê-se exactamente como uma mutação que SOBREVIVEU** — ao preço de
uma corrida inteira para descobrir. O pré-voo salta a corrida limpa e o
`corrida()`, conta cada âncora e sai `1` se alguma não casar **exactamente uma
vez**: `43 de 43` e `9 de 9`, em segundos, e corre-se **depois de todo `fmt`**.

⚠️ **E o sumário dele DIZ que zero testes correram** — *um pré-voo que
imprimisse «N de N sangram» seria um instrumento a descrever-se mal, que é o
defeito que o arnês inteiro existe para não ter.*

---

## §17 — O PORTÃO desta corrida

| régua | resultado |
|---|---|
| `nextest-impacted` | **18 575 / 18 575** |
| gates da tinta fina, **com adaptador** (a suíte de GPU incluída) | **56 / 56** |
| censos da árvore COMBINADA | **127 / 127** · controlo do filtro `12 de 12` |
| `cargo fmt --all --check` | limpo — ⚠️ e o **pré-voo das âncoras** a seguir (`43 de 43` · `9 de 9`) |
| clippy `--all-targets -D warnings` nas quatro crates | zero |
| as 10 vassouras sobre os 23 ficheiros do diff | **zero achados NOVOS** |
| tectos de LOC | maior ficheiro tocado a **495** de `700` |
| mutação — a cerca do plano (`muta_a_cerca_do_plano.sh`) | **8 de 9** (a 9.ª é o CONTROLO) |
| mutação — a metade visível (`muta_a_metade_visivel.sh`) | **42 de 43** em duas fatias (a 43.ª é o CONTROLO) |

⚠️ **As vassouras: os dois achados são PRÉ-EXISTENTES e isso foi MEDIDO, não
inferido.** O mesmo handoff no `HEAD` já os tem (`3` e `5` linhas), as linhas
acusadas são a `523`, `578`, `579`, `586` e `1031`, e **o §14 começa na `1047`**
— ou seja, nenhuma delas está numa linha que esta corrida escreveu. Os dois
tokens estão nomeados no §10.10, com a triagem de cada um.

⛔⛔ **E a 1.ª redacção DESTE parágrafo repetia os dois nomes — e a vassoura
acusou-a.** *A prosa que explica uma isenção herda a isenção que ela explica*,
e a saída barata (isentar mais uma linha) é como um ledger cresce até não medir
nada. ⇒ o parágrafo APONTA para o §10.10 em vez de repetir, e o *«zero achados
novos»* volta a ser literalmente verdade.

⚠️ **E a rede da metade visível foi RE-CORRIDA INTEIRA, em fatias**: este diff
toca no `tinta_da_peca.rs`, no `slots.rs`, no `recusa.rs`, no
`history_dyntopo.rs` e nos censos — que são exactamente os ficheiros que ela
ataca — e *um placar herdado é um placar sobre outra árvore*.

---

## §18 — O PLANO VIAJA NO `.ph2dproj` — e a nota que mandava construí-lo estava ERRADA pela metade

### §18.1 — A auditoria da nota, ANTES da primeira linha

A §9 e o `CLAUDE.md` §5 diziam: *«o plano não viaja no `.ph2dproj` (dívida
herdada da wave anterior, **ao lado da cor por vértice**)»*.

⛔ **A segunda metade é FALSA, e o código dizia-o:** o `MeshData` tem
`colors: Option<Vec<[f32; 3]>>`, o `Mesh::to_data`/`from_data` carregam-no, e
uma sonda pelo caminho real (`encode_doc` → `decode`) mede **`SIM, AO BIT`**.

⇒ *auditar a lista contra o CÓDIGO antes de pegar um item dela* — a lei que
este §5 escreve sobre si mesmo — poupou metade da wave, e a nota foi corrigida.

### §18.2 — O que o plano custa CRU, medido

Peça de fábrica, `98 306` vértices:

| degrau | amostras | postcard | escrever |
|---|---:|---:|---:|
| `2x` | `393 218` | `4,7 MB` | `1,4 ms` |
| `8x` | `6 291 458` | **`75,5 MB`** | `19,1 ms` |
| `16x` | `25 165 826` | **`302 MB`** | `69,4 ms` |

⇒ *armar o `8x`, dar um traço e gravar custaria `75 MB`*, e o `.ph2dproj` **não
comprime** (medido: nenhum `flate2`/`zstd` no caminho).

### §18.3 — ⛔⛔⛔ E a MEDIÇÃO derrubou o meu desenho, uma vez

A 1.ª redacção guardava **só corridas** (`(quantas, cor)`), com uma tabela que
saíra de uma sonda sobre `Tinta::nova` — onde um plano é **um** `COR_DE_NINGUEM`
repetido e a resposta é **uma corrida**. Eu escrevi, por extenso, que *«`8 %` num
caso que a pintura real não produz não paga uma segunda forma»*.

A pintura real produz **exactamente** esse caso. Medido sobre a
`Tinta::semeada`, que é **como um plano nasce numa peça já pintada**:

| plano ao `8x` | corridas | em corridas | contra cru |
|---|---:|---:|---:|
| `nova` (peça nunca pintada) | `1` | `~0` | **`0,000×`** |
| `semeada` de cor CHAPADA | `3 145 729` | `40,9 MB` | `0,542×` |
| `semeada` de cor VARIADA | `6 291 458` | `81,8 MB` | **`1,083×`** |

⭐⭐ **A semente INTERPOLA, e interpolar entre três cores iguais não devolve a
cor em `f32`** — os pesos baricêntricos somam `1` com erro de último bit. *Um
plano «chapado» não é chapado nos BITS.*

⇒ **duas formas, e o escritor escolhe a MENOR** por uma conta exacta e barata
(uma passagem sobre as corridas, sem serializar as duas — aos `16x` cada
serialização são `300 MB`). O pior caso passa a ser **`+1 byte`** e o melhor
continua a ser **`75 MB → 0`**.

⚠️ **E quem apanhou o erro foi um GATE VERMELHO**, não uma releitura: o
`as_duas_formas_das_amostras_fazem_o_que_prometem` nasceu com a barra do
desenho antigo (`< 1/50`) e reprovou sobre a fixtura semeada.

### §18.4 — ⛔⛔⛔ A igualdade é por BITS, nunca por `==`

`-0.0 == 0.0` é **verdade** em `f32` e os bits são diferentes ⇒ uma corrida
fechada por `==` juntaria os dois e devolveria os bits errados na leitura.
*Uma compressão que se diz sem perda e que troca um sinal de zero é pior do que
uma que se diz com perda*, porque ninguém vai procurar ali. Com `to_bits` o
`NaN` também se comporta: por `==` ele nunca é igual a si próprio, e uma corrida
de mil `NaN` viraria mil corridas.

### §18.5 — O que é DERIVADO, e a migração

⭐ **O documento guarda o NÍVEL e as AMOSTRAS, e mais nada.** A `Topologia` é
reconstruída das faces da malha que viaja ao lado — a mesma lei que já faz esta
porta re-derivar normais, adjacência e octree.

⚠️ **O plano NÃO é redundante com a cor por vértice**, que viaja dentro do
`stack`: aquela é a PROJECÇÃO deste plano nos vértices (o `devolve` escreve-a no
fim de cada traço), e re-semear a partir dela devolve um plano exacto nos
vértices e **interpolado no resto** — que é literalmente a tinta a voltar à
resolução da malha.

⭐⭐ **`SCULPT_DOC_VERSION` 1 → 2, COM migração.** Um v1 abre e as peças vêm sem
plano. ⛔ *Subir a versão de um formato sem degrau é apagar o trabalho de quem
já o usou* — e o `decode` recusa o load inteiro por versão, logo todo
`.ph2dproj` que o dono já gravou deixaria de abrir. A versão é lida **sozinha e
primeiro** (`take_from_bytes::<u32>`): tentar a forma nova e cair para a velha
no erro seria apostar que um v1 falha a parsar como v2, e o postcard é
POSICIONAL — *ele devolve lixo bem-formado*.

⚠️ **O golden do tamanho subiu `1538 → 1539`, e a conta FECHA à mão:** o campo
`tinta` é um `Option` e um `None` custa exactamente o discriminante. A VERSÃO
não muda nada — `1` e `2` são ambos um varint de um byte.

### §18.6 — E o SAVE era o terceiro consumidor de uma porta que não existia

Durante um traço o plano não está na peça: o pen-down **empresta-o**. ⇒ um
`Ctrl+S` a meio de uma pincelada gravava a peça **sem o detalhe fino**.

⭐ ⇒ [`tinta_da_peca::plano_de`] — *onde está o plano desta peça agora?* —, com
**três** consumidores (o upload · a voz · o save) e a pergunta pelo **DONO** do
empréstimo, nunca pelo índice `active`.

⚠️ **Ela recebe as PARTES e não o `&self`**, e não é arrumação: o laço de upload
precisa de `&mut self.renderer` ao mesmo tempo. *A assinatura que o compilador
aceita é a mesma que um gate consegue montar sem uma cena* — e uma cena pede um
`wgpu::Device`.

### §18.7 — As réguas

- `as_corridas_devolvem_as_amostras_ao_bit` · `um_plano_chapado_cabe_numa_corrida_so`
  (sem a segunda, um encoder que nunca juntasse nada passava a ida-e-volta)
- `o_zero_negativo_nao_se_junta_ao_positivo` — ⚠️ a asserção é sobre os **BITS**:
  um `assert_eq!` em `f32` diria que os dois são iguais e o gate ficava VÁCUO
- `uma_corrida_de_nan_e_uma_corrida` · `corridas_que_nao_somam_o_que_a_malha_pede_sao_recusadas`
- `o_plano_de_tinta_fina_atravessa_o_ficheiro_ao_bit` — ⚠️ com o CONTROLO de que
  a fixtura DIFERE da semente, *senão um `decode` que simplesmente re-semeasse
  passava*
- `um_documento_da_versao_anterior_abre_e_vem_sem_plano`
- `um_plano_que_nao_descreve_a_malha_recusa_o_load` (e a recusa NOMEIA a peça)
- `as_duas_formas_das_amostras_fazem_o_que_prometem` (as duas metades medidas)
- `a_porta_do_plano_pergunta_pelo_dono_e_nao_pelo_indice` (três células)
- o censo da fiação vai a **DEZOITO** elos: o save lê a porta, e o load instala
  o que leu — *o `decode` pode estar certo e o `install_doc` deitar o plano
  fora, e aí o ficheiro tem o detalhe lá dentro e o artista nunca o vê*

### §18.8 — O PORTÃO desta wave

| régua | resultado |
|---|---|
| `nextest-impacted` | **18 585 / 18 585** |
| gates da tinta fina, **com adaptador** | **74 / 74** |
| censos da árvore COMBINADA | **127 / 127** · controlo do filtro `12 de 12` |
| `cargo fmt --all --check` | limpo — com o **pré-voo das âncoras** a seguir (`43` · `9` · `12`) |
| clippy `--all-targets -D warnings` | zero |
| as 10 vassouras sobre os 12 ficheiros | **zero achados NOVOS** |
| tectos de LOC | maior ficheiro tocado a **494** de `700` |
| mutação — o plano no ficheiro (`muta_o_plano_no_ficheiro.sh`) | **11 de 12** (a 12.ª é o CONTROLO) |
| mutação — a metade visível, RE-CORRIDA | **42 de 43** em duas fatias |

⚠️ **A rede da CERCA DO PLANO (`muta_a_cerca_do_plano.sh`, `8 de 9`) NÃO foi
re-corrida, e a razão é medida:** as nove mutações dela vivem na
`ph2d-mesh-colors` e na `ph2d-mesh-render`, e a população de teste dela são
essas duas crates — **este diff não toca em nenhuma**. *Um placar herdado é um
placar sobre outra árvore; este é sobre a MESMA.*

## §19 — «SOBREVIVEU MAS SEM OS DETALHES 8x» — o ficheiro estava certo e o PRIMEIRO QUADRO deitava o plano fora

### §19.1 — O que o dono disse

> «sobreviveu mas sem os detalhes 8x»

A tinta volta do ficheiro. A **resolução** dela não.

### §19.2 — Onde acaba o alcance dos gates da §18

Os gates da wave anterior medem o **FICHEIRO**: `encode` → bytes → `decode` →
amostras **ao bit**, com o controlo de que a fixtura difere da semente. Estavam
**todos verdes** e continuam certos.

⛔ **O produto tem mais um elo, e ele corre DEPOIS do `decode`:** cada quadro
reconcilia o plano de cada peça contra o degrau que a fileira `Paint Detail`
pede — a [`rota`] com o `tinta_nivel` da CENA. Num app acabado de abrir esse
campo é `None`, logo a rota devolve `DaPeca { pedir: None }` e a [`garante`]
faz `tinta.take()`: **o plano que o ficheiro acabou de trazer é deitado fora no
primeiro quadro.**

⭐ **E a cor por VÉRTICE sobrevive**, porque ela viaja dentro da malha — é por
isso que o artista vê a tinta lá **com a grossura errada** em vez de a ver
desaparecer. *Os dois sintomas leem-se com frases muito parecidas, e o report
descreve exactamente o primeiro.*

⚠️⚠️ *Uma ida-e-volta medida a montante do consumidor não afirma nada sobre o
consumidor* — a MESMA forma que esta linha pagou na ponte da curva do pincel de
pose, onde o corpus corria a crate directamente com a convenção dela.

### §19.3 — A cura: o degrau é um FACTO DO DOCUMENTO

O `tinta_nivel` é o estado da fileira, e até aqui só um gesto o escrevia. Ele
passa a ser escrito também por quem instala um documento — *abrir um projecto é
aprender o que ele diz*, e o degrau está lá dentro, no plano de cada peça.

⭐ [`tinta_da_peca::degrau_do_documento`] — porta **pura**, ao lado das irmãs
deste ficheiro: o laço do `install_doc` já tem `&mut self`, e um gate que
precisasse de uma cena pediria um `wgpu::Device` e nasceria `#[ignore]`.

⚠️⚠️ **O recurso à PRIMEIRA peça com plano não é conforto, é a lei:** o degrau
é **UM para a cena inteira** e os planos são **por peça**. Com a peça activa sem
detalhe fino, ler só a activa deixaria a fileira desarmada e o quadro seguinte
deitaria fora o plano de **TODAS as outras** — *um documento com dez peças
pintadas finas perdia as dez porque a activa não estava*.

⚠️ E o terceiro braço — **sem plano nenhum a resposta é `None`** — é o que
impede a cura de ARMAR a fileira num documento que nunca teve tinta fina.

### §19.4 — As réguas

- `o_primeiro_quadro_nao_deita_fora_o_plano_que_o_documento_trouxe` — o gate do
  report, e ele percorre o **QUADRO** e não a porta. ⭐ **O CONTROLO está dentro
  dele e é ele que reproduz o defeito:** com a fileira desarmada o plano é
  deitado fora; com o degrau do documento ele sobrevive **ao nível em que foi
  gravado**.
- `o_degrau_do_documento_sai_da_peca_activa_e_recorre_as_outras` — as três
  células (nada · só uma peça distante · a activa manda).
- o censo da fiação vai a **DEZANOVE** elos: *o `decode` pode estar certo, o
  `install_doc` pode instalar, e o degrau não voltar à fileira — e aí o ficheiro
  tem o detalhe lá dentro, a peça recebe-o, e o **quadro seguinte** apaga-o*.
  Nenhum dos dezoito elos anteriores atravessa esse ponto.

### §19.5 — A prova de mutação, e o que o PRÉ-VOO apanhou

A rede do plano no ficheiro passa a **catorze** mutações e fecha em
**`13 de 14` a sangrar** (a `P12` é o CONTROLO e não pode) — as duas novas são
a cura inteira e a metade do recurso:

| mutação | o que ela tira |
|---|---|
| `P13` | o degrau não volta para a fileira ⇒ o 1.º quadro deita o plano fora |
| `P14` | o recurso às OUTRAS peças desaparece |

⚠️ **E o pré-voo (`MUTA_SO_ANCORAS=1`) fez exactamente o trabalho dele:** a
âncora da `P14` casou **zero** vezes, porque o `cargo fmt` tinha reescrito a
expressão dela em cinco linhas. *Uma âncora que casa zero lê-se, num placar,
exactamente como uma mutação que sobreviveu* — e aqui ela foi apanhada em
segundos, sem correr um teste.

### §19.6 — O PORTÃO desta wave

| régua | resultado |
|---|---|
| `nextest-impacted` | **18 587 / 18 587** |
| gates do DOCUMENTO e do PLANO, **com adaptador** | **70 / 70** (os dois novos e o censo lá dentro) |
| censos da árvore COMBINADA | **127 / 127** · controlo do filtro `12 de 12` |
| `cargo fmt --all --check` | limpo — e os **três** pré-voos de âncora a seguir (`9` · `43` · `14`) |
| clippy `--all-targets -D warnings` | zero — ⚠️ **VERMELHO à primeira**, ver abaixo |
| as 10 vassouras sobre os 6 ficheiros | **zero achados NOVOS** (as 5 linhas acusadas são `≤ 1031` e a §19 começa na `1541`) |
| tectos de LOC | maior ficheiro tocado a **524** de `700` |
| mutação — o plano no ficheiro | **13 de 14** (a 14.ª é o CONTROLO) |

⚠️ **O clippy reprovou sobre esta wave e apanhou-a bem:** um `let mut quadro`
no gate novo, cujo fecho não muta nada do que captura. *É o portão da LINHA a
fazer o trabalho que de outra forma o `ship.sh` faria dias depois.*

⛔⛔ **E a 1.ª corrida dele MENTIU-ME, pela armadilha que o `CLAUDE.md` §2
escreve por extenso:** eu canalizei o clippy por `| grep -E '^(error|warning)' |
head`, e um `head` **destrói o código de saída** ⇒ a corrida imprimiu
`error: could not compile` e o processo saiu **`0`**, com a notificação a
dizer *«completed (exit code 0)»*. Só a LEITURA das linhas o apanhou. ⇒ a
re-corrida escreve para um ficheiro e imprime o `rc` REAL. *Um portão cujo
veredito passa por um `head` não é um portão.*

⚠️ **E uma segunda mentira de arnês, de graça:** a 1.ª varredura das dez
vassouras devolveu `rc=2` nas **dez** e eu quase a li como *«dez com achado»* —
`2` é **uso errado**, e a causa é que neste terminal `$ALVOS` não se parte em
palavras, logo o script recebeu os seis caminhos como **um**. ⭐ Ele **falhou
alto** (`✗ path não existe`), que é o desenho dele; refeita sob `bash -c`, a
resposta é `2 de 10` com achados, **todos** pré-existentes.

### §19.7 — E a CENA não continha o fenómeno: o roteiro ganha o passo (8)

A `=52` ensinava a tinta fina inteira e **não tinha um passo de GRAVAR**. As
duas waves da persistência (§18 e esta) vivem do ciclo *gravar → reabrir*, e o
dono só o exercitou porque foi procurá-lo. ⇒ passo **(8)**, com a linha do
*«como saber que deu errado»* a acompanhá-lo.

⚠️⚠️ **E ele manda FECHAR o app, com a razão escrita dentro:** reabrir o
ficheiro na MESMA sessão deixa a fileira onde o artista a pôs, logo o
`tinta_nivel` já vale `8x` e a reconciliação concorda — *o passo passaria com
o defeito vivo*. **A cena só contém o fenómeno com um `tinta_nivel` virgem.**

⚠️ **Os dois atalhos foram MEDIDOS antes de escritos**, não assumidos: o
`if ctrl` do teclado da escultura é um catch-all que devolve `false` a todo
`Ctrl+` que não seja o desfazer ⇒ o `Ctrl+Shift+S` e o `Ctrl+O` **caem para a
shell** e chegam ao `project_save_gesture`/abrir. Os dois entram na lista de
TECLAS do censo do roteiro, que **afirma que nenhum deles é também um rótulo
pintado** — *a excepção tem de continuar a ser uma excepção*.

Réguas da cena e do painel: **411 / 411** · clippy `-D warnings` zero · as 10
vassouras sobre os dois ficheiros **zero achados** · LOC `227` e `267`.

## §20 — A SAÍDA: exportar uma peça pintada a `8x` DIZ o que fica para trás

> Ordem do dono: *«siga conforme sua própria orientação»*, e a orientação era a **P5 — a SAÍDA**,
> começando pela metade barata: **o app passar a avisar**. A segunda metade (assar o detalhe numa
> imagem que acompanha a peça) fica para a wave seguinte, e o §20.7 diz o que ela precisa.

### §20.1 — O defeito: uma perda SILENCIOSA, e ela não estava em lado nenhum

`Ctrl+Shift+E` escreve a cena num `.obj`/`.ply`/`.stl`. Os três guardam cor **POR VÉRTICE**, e o
escritor recebe a **projecção do plano nos vértices** — *a tinta de volta à resolução da malha*,
que é exactamente a grandeza que esta linha inteira existiu para separar.

⛔ **E o app não dizia nada.** O [`export.rs`](../../../crates/ph2d-app-sculpt3d/src/export.rs) nunca
toca em `tinta`, e o toast que ele escreve — *«not carried: …»* — nomeava `mask`, `colour` e
`pieces merged` e **nunca** a tinta fina.

⚠️ É a espécie que este repo já nomeia por escrito: *um importador que ignora em silêncio é pior
que um que recusa*, aqui do lado da **saída**. O artista arma o `8x`, pinta o detalhe que só existe
por causa dele, exporta, e descobre a perda **no outro programa**.

### §20.2 — A cláusula entra na LEI PARTILHADA, nunca no wrapper da escultura

A pergunta *«este formato carrega a tinta fina?»* é uma propriedade **do FORMATO**, e o formato tem
**dois** consumidores: a escultura e a modelação 3D. ⇒ a cláusula vive em
[`ph2d_mesh::lost_by`](../../../crates/ph2d-mesh/src/read.rs), ao lado das outras três, e a tabela
ganha [`MeshFormat::keeps_fine_paint`](../../../crates/ph2d-mesh/src/export.rs) — `false` nos três,
com o mecanismo escrito ao lado.

⭐ **É isso que faz os dois consumidores pararem de avisar JUNTOS** no dia em que um formato passar
a carregá-la, sem ninguém se lembrar de ir apagar a segunda cópia. Escrevê-la no wrapper da
escultura seria a segunda lista que diverge — a forma que o próprio doc daquele wrapper já condena
por extenso (*«duas listas divergem — a que fica errada diz "cor preservada" sobre um STL com a
confiança da certa»*).

### §20.3 — A porta pergunta pelo DONO do empréstimo, e essa é a lição do §18 outra vez

A cláusula precisa de saber se **esta cena** carrega algum plano. ⛔ E a resposta ingénua
(`objects.iter().any(|o| o.tinta.is_some())`) está **errada a meio de uma pincelada**: o plano é
EMPRESTADO ao traço por um `take`, logo durante um traço o `Option` da peça está **VAZIO** e o
aviso **calava-se exactamente enquanto o artista pinta**.

⇒ [`tinta_da_peca::alguma_peca_tem_plano`](../../../crates/ph2d-app-sculpt3d/src/tinta_da_peca.rs),
construída sobre a `plano_da_peca` que já pergunta ao **dono** do empréstimo (§13), com a
`Sculpt3dScene::alguma_peca_tem_tinta_fina` a ser a fachada de cena.

⚠️ **É o mesmo mecanismo que o SAVE pagou no §18** — ali o 3.º consumidor de uma porta que não
existia, aqui o 4.º. *Todo consumidor que leia o `Option` da peça em vez da porta inventa o seu
próprio defeito, e todos eles são invisíveis fora de um traço.*

### §20.4 — A modelação 3D passa `false` POR MEDIÇÃO, e uma nota envelhecida foi corrigida

O [`ph2d-app-field3d/src/export.rs`](../../../crates/ph2d-app-field3d/src/export.rs) chama
`ph2d_mesh::lost_by(fmt, false)`: uma peça de campo implícito **não tem plano de amostras**, e o
`false` está escrito com essa razão ao lado — não é um valor de conforto.

⛔⛔ **E o doc do wrapper da escultura dizia uma coisa FALSA:** *«`pub(crate)` porque a modelação 3D
o CHAMA ([`crate::field3d_export`])»* — esse módulo **não existe nesta crate** desde que a família
saiu da shell (W2), e a modelação chama `ph2d_mesh::lost_by` **directamente**. *Uma nota que
justifica uma visibilidade por um chamador que mudou de casa lê-se como medição e é um palpite* — a
frase ficou, com a correcção à vista.

⚠️⚠️ **E a MESMA prosa envelhecida vivia em TRÊS sítios, não dois:** o doc do wrapper, o doc do
`pub use` no `lib.rs` da escultura (*«partilhado com a modelagem 3D ([`crate::field3d_export`])»*)
e o cabeçalho do `export.rs` da própria modelação, que dizia que o aviso vem *«do
`sculpt3d::lost_by`»*. ⇒ *três módulos a nomear um chamador que mudou de casa, e nenhum dos três a
saltar por onde a nota dizia* — os três foram corrigidos nesta wave. ⭐ **E é a mesma lei que o
repo já escreve sobre o CÓDIGO, aplicada à PROSA:** uma afirmação repetida em três sítios viaja
para os que alguém se lembrar de emendar, e *só uma PORTA é uma lei* — aqui a porta é o
`ph2d_mesh::lost_by`, que já existia; o que faltava era as notas concordarem com ela.

### §20.5 — Os gates, e o CONTROLO de cada um

| gate | onde | o que afirma | CONTROLO |
|---|---|---|---|
| `the_warning_names_fine_paint_only_when_the_scene_carries_some` | `ph2d-mesh/src/export_tests.rs` | sobre `MeshFormat::ALL`: sem tinta a frase é **byte a byte** a de antes da wave (reconstruída à mão no gate); com tinta a cláusula aparece; e `!keeps_fine_paint()` nos três | a frase pré-wave, montada no gate a partir das outras três cláusulas — *sem ela, uma cláusula que soasse sempre passava* |
| `a_saida_pergunta_pela_porta_e_ve_a_tinta_emprestada_ao_traco` | `ph2d-app-sculpt3d/src/tinta_da_peca_portas_tests.rs` | com o plano EMPRESTADO, a porta responde `true` | `pecas.iter().all(|p| p.tinta.is_none())` **primeiro** — sem ele o gate passaria com a resposta ingénua |
| elo nº **20** do censo da fiação | `ph2d-app-sculpt3d/src/tinta_fiacao_tests.rs` | o `export.rs` da ESCULTURA chama `alguma_peca_tem_tinta_fina` | a prosa é cortada antes de medir; o inverso exige a agulha **ausente** da metade comentada |
| elo nº **21** do mesmo censo | idem, por `include_str!` relativo | o `export.rs` da MODELAÇÃO 3D passa `false` | o mesmo par corte-de-prosa / controlo |

⚠️ O censo passou de `dezanove_sitios` para **`vinte_e_um_sitios`**, e a razão de ele existir é a
mesma das M19–M22 do §12: *a prova de comportamento do elo vive num gate `#[ignore]` de GPU, que
nem o arnês nem o CI correm* — sem o elo textual, apagar a chamada no `export.rs` seria silencioso
em todo o sítio onde alguém a fosse procurar.

⛔⛔ **E os dois elos da saída falham ao CONTRÁRIO um do outro, que é porque são dois:** se o da
escultura morre, o artista perde o detalhe **em silêncio**; se o da modelação morre (um `true` no
lugar do `false`), ela **avisa de uma perda que não acontece** — e *um aviso errado é pior que
aviso nenhum, porque o artista confia nele*.

### §20.6 — O ARNÊS, e as duas coisas que ele obrigou a corrigir

[`muta_a_saida_da_tinta.sh`](../ferramentas/muta_a_saida_da_tinta.sh), 7 mutações + pré-voo.

⚠️ **A população são DUAS crates** (`ph2d-mesh` tem a lei, `ph2d-app-sculpt3d` tem a porta): *uma
corrida só de uma delas leria VERDE sobre a mutação da outra, e isso é um placar fabricado.*

⚠️ **Ele corre por `cargo nextest` e não por `--lib`**, pela razão que o §18 deste handoff já mediu:
o `cargo test --lib` desta crate morre em `SIGSEGV` de forma intermitente, e um binário que morre
leva o `test result:` com ele — *um aborto mudo lê-se exactamente como uma mutação que não entrou*.

⛔⛔ **E DUAS mutações minhas tinham de ser reescritas antes de correr.** A `S2` apagava o bloco
inteiro e a `S3` apagava o `has_fine_paint &&` — as duas deixam o parâmetro **sem uso**, logo o
veredito delas passa a depender da política de warnings da árvore: com `-D warnings` elas **não
compilam** e o arnês aborta. ⇒ hoje a `S2` **esvazia o corpo** do `if` e a `S3` troca `&&` por
`||`, e cada uma isola **uma metade** da lei com o parâmetro lido:

| | o que morre | o que se vê |
|---|---|---|
| **S1** | a tabela mente (`keeps_fine_paint → true`) | a cláusula nunca soa |
| **S2** | a cláusula cala-se | a perda volta a ser silenciosa |
| **S3** | a metade da CENA deixa de decidir | o aviso soa **sempre**, até sem tinta |
| **S4** | a porta lê o `Option` da peça | o aviso cala-se a meio de um traço |
| **S5** | o elo crava `false` | o aviso nunca soa |
| **S6** | a modelação 3D passa `true` | ela avisa de uma perda que não acontece |
| **S7** | CONTROLO (uma linha em branco) | **não pode sangrar** |

*Uma mutação imune à configuração de lint mede a LEI; a outra mede o ambiente.*

⛔⛔⛔ **E a 1.ª corrida deu `5 de 7` com o `S6` a SOBREVIVER — um sobrevivente FABRICADO pelo
próprio arnês.** Ele muta a `ph2d-app-field3d`, e a população era `-p ph2d-mesh
-p ph2d-app-sculpt3d`: **a crate mutada nem é compilada por aquela corrida**, logo a mutação não
podia sangrar de maneira nenhuma. ⚠️ *O cabeçalho que eu tinha acabado de escrever no arnês já
condenava isto por extenso* — eu escrevi a lei para DUAS crates e mutei uma TERCEIRA.

⭐⭐ **A cura não foi acrescentar a crate à população** (isso paga uma suíte inteira em cada uma
das sete corridas): foi escrever o **elo dela no censo da fiação**, que vive na
`ph2d-app-sculpt3d` e a alcança por `include_str!` relativo — o mesmo caminho pelo qual ele já
alcança o MOTOR e a PLACA, com a mesma razão declarada. ⇒ quem **OBSERVA** a mutação passou a
estar na população, e ela sangra sem uma crate nova no arnês.

⇒ **A população de um arnês é de quem OBSERVA a mutação, nunca de quem a CONTÉM.** Um `-p` por
crate mutada é a leitura ingénua e a mais cara.

⚠️ **E a rede foi re-corrida INTEIRA depois da cura**, não emendada no lugar da `S6`: *um placar
herdado é um placar sobre outra árvore* (§12).

**Placar depois da cura: `6 de 7` sangram** — a 7.ª é o `S7`, o CONTROLO, e ela **não pode**.
*(A 1.ª corrida, antes do elo do field3d, deu `5 de 7` com o `S6` a sobreviver.)*


### §20.7 — E o PORTÃO abriu um buraco MAIOR que a wave: metade dos arneses desta linha estava CEGA, e DOIS estavam MORTOS

O passo 2 do portão desta wave corre o **pré-voo de todas as âncoras** — e ele devolveu isto:

| arnês | pré-voo | veredito |
|---|---|---|
| `muta_a_cerca_do_plano` · `muta_a_metade_visivel` · `muta_o_plano_no_ficheiro` · `muta_a_saida_da_tinta` | tinha | verdes |
| **`muta_a_origem_do_triangulo`** | **não tinha** | ⛔ **`0 de 3` âncoras — MORTO** |
| **`muta_a_lei_da_reticula`** | **não tinha** | ⛔ **`15 de 16` — a `M16` morta** |
| `muta_o_gemeo_em_wgsl` · `muta_o_upload_da_tinta` | não tinha | âncoras vivas |

⛔⛔⛔ **O `muta_a_origem_do_triangulo` não media NADA.** As três âncoras dele apontam a
`mesh.rs`, e a lei mudou-se para **`mesh_indices.rs`** quando um **corte de tecto de LOC** partiu
aquele ficheiro (`723 → 691`) numa wave posterior desta mesma linha. ⚠️ É a lei que o repo já
escreve — *mover código parte gates em DUAS espécies e só UMA avisa* — aqui na espécie **MUDA**:
um arnês com âncora morta imprime `0 de 3` e **um placar lê-se como uma corrida**.

⛔⛔ **E a `M16` do `muta_a_lei_da_reticula` morreu no `7d446fde6` — um commit desta jornada.** A
forma que ela ancorava (`out.push(if s < n { cantos[s] } else { TRI });`) saiu do `topo.rs` quando
o laço foi reescrito para o idioma que o clippy aceita, na wave do **gémeo em WGSL**. Durante
quatro waves aquele arnês leu `15 de 16` e ninguém re-leu o log.

⭐⭐ **A cura tem TRÊS metades, e nenhuma basta sozinha:**

1. **re-ancorar** — `mesh.rs` → `mesh_indices.rs` (3 âncoras) e a `M16` sobre o laço novo
   (`cantos.iter().take(n)` → `.rev()`, que é a MESMA intenção: *os cantos viajam invertidos*);
2. **o pré-voo nos quatro** que não o tinham — senão o mesmo corte cega-os outra vez;
3. **a guarda da corrida limpa** — ⚠️ a 1.ª redacção da cura pôs o pré-voo e **deixou a corrida
   limpa correr**, logo ele imprimia `VERDE antes: 579 testes correram` e a seguir *«ZERO testes
   corridos»*. *Um sumário que diz «zero» depois de correr a suíte é o instrumento a mentir sobre
   si mesmo*, e a coluna `corridas-limpas` do portão é o que o prova.

⛔ **E um quarto achado, no `muta_o_upload_da_tinta`: ele NÃO TINHA TECTO NENHUM.** O ficheiro
acabava num `echo`, logo o código de saída era o do `echo` — **zero**, com ou sem sobrevivente.
*Um arnês sem tecto não reprova; ele RELATA* — e um laço de portão que leia só o `rc` lê-o verde
para sempre. Hoje o tecto é `total - 1`, com o número tirado de uma corrida (`6 de 7`, a `U3`
NOMEADA) e nunca de um palpite.

**A prova de que os mortos voltaram a viver** (corrida completa, não pré-voo):

| arnês | antes | depois |
|---|---|---|
| `muta_a_origem_do_triangulo` | `0 de 3` (três ABORTOS) | **`3 de 3` sangram** |
| `muta_a_lei_da_reticula` | `15 de 16` | **`16 de 16` sangram** |
| `muta_o_upload_da_tinta` | `6 de 7`, sem tecto | `6 de 7`, **com tecto medido** |

⇒ os oito arneses desta linha somam hoje **104 âncoras, todas a casar exactamente uma vez, com
`corridas-limpas = 0`**.

⚠️⚠️ **E o laço do meu portão tinha o mesmo defeito de família:** ele passou `MUTA_SO_ANCORAS=1` a
todos, e os quatro que o ignoram **correram as mutações todas** — um passo de segundos virou um
passo de dezenas de minutos, e o sinal era o mesmo `rc=0`. *Um laço que assume uma capacidade não
medida não falha: ele fica caro e cala-se.* ⇒ a cura foi dar a capacidade aos quatro; a alternativa
(derivar o suporte por `grep` no próprio laço) fica registada como a segunda saída.

### §20.8 — ABERTO: a segunda metade da P5

⏳ **A tinta fina ainda não SAI.** A cura de fundo é **assar o plano numa textura UV** que acompanhe
a peça — e o substrato existe pela metade: a [`ph2d-uv-atlas`](../../../crates/ph2d-uv-atlas/) já
vive no repo e **o único consumidor dela hoje é um `example`** (`ph2d-quadchain/examples/atlas_probe`),
ou seja ela nunca correu no caminho de um produto.

⚠️ E o aviso desta wave é o que torna essa segunda metade **honesta de adiar**: enquanto ela não
existir, o artista sabe o que perde **antes** de abrir o ficheiro noutro programa.

## §21 — «PASSO 4 NÃO MOSTRA MENSAGEM NENHUMA»: a exportação estava certa, e o aviso morria antes de ser pintado

> Report do dono, 22/09, sobre o smoke da §20: ***«passo 4 não mostra mensagem nenhuma no app.
> Onde deveria aparecer?»***

⚠️ **Conte o DELTA:** `PROJECT_SCHEMA` 0, os três registos 0, `SCULPT_DOC_VERSION` 0, zero
contrato, zero ADR. ⛔ **Mas ela TOCA FOUNDATIONAL:** `ph2d-app-host/src/modal.rs` ganha uma porta
**append-only** (`pick_files`) — ver §21.6.

### §21.1 — O que estava certo

O ficheiro é escrito, o `lost_by` devolve a frase com a cláusula, e o `import::toast` empurra-a
para o `ToastQueue`. Os dois ramos do quadro pintam a fila
(`fase_hero_chrome_tail` · `fase_legacy_chrome`), no **topo ao centro**
([`progress::column_row`]), com TTL de **3 s**.

⇒ *nada na wave da §20 estava errado.* O elo partido é o SEGUINTE.

### §21.2 — O mecanismo: um relógio de PAREDE contra um diálogo que CONGELA

O `ToastQueue::tick` anda **segundos de parede**. Um `rfd::FileDialog` bloqueia o laço **dentro**
do quadro, logo o quadro seguinte traz um `wall_dt` do tamanho do tempo que o artista passou a
escolher o nome — e o primeiro `tick` depois do diálogo põe `age_s ≈ 10 s` num toast de `3 s`.

⇒ **ele é removido antes de alguém o desenhar.** Do lado do artista: *nenhuma mensagem*.

### §21.3 — ⛔⛔⛔ E a cura já existia, com as palavras do PRÓPRIO dono, de 2026-08-22

O [`fase_chrome_clock.rs`](../../../shells/desktop/src/render_loop/fase_chrome_clock.rs) tem isto
escrito, verbatim, há um mês:

> ⭐ **O tempo em que um DIÁLOGO MODAL congelou o loop não é tempo de animação.** […] O sintoma,
> com as palavras do Enio (2026-08-22): *"não vejo em nenhum lugar a mensagem"* — o toast escrito
> logo depois do diálogo era pintado UM quadro e morria no `tick` seguinte.

A cura é o `ui_dt = modal::chrome_dt(wall_dt, modal::take_stall())`, e a cláusula que a torna
condicional está na linha seguinte: *«a parte parada **nomeada por quem a causou**»*.

⛔⛔ **⇒ ela só protege quem DECLARA.** E a exportação da escultura abria o diálogo à mão:

```rust
let Some(path) = dialog.set_file_name(...).save_file() else { return; };
```

⚠️⚠️ ***Uma cura escrita para UM chamador não é uma lei — só uma PORTA é.*** A mesma frase que
esta linha já pagou no `stroke_uniform`, no `compact_for_faces` e no `fora_da_pegada`, aqui uma
camada acima: **a porta estava construída, tinha um gate a exigi-la, e a população desse gate era
UMA crate.**

### §21.4 — E o gate que devia tê-lo apanhado tinha DUAS cegueiras

O [`ph2d-app-field3d/tests/it/modal_door.rs`](../../../crates/ph2d-app-field3d/tests/it/modal_door.rs)
existe exactamente para isto, com a mensagem *«diálogo modal aberto sem declarar o
congelamento»*. Ele não podia ver este defeito por duas razões independentes:

1. **A população é a crate dele.** O cabeçalho di-lo por escrito — *«o gate viajou com o sujeito
   dele»* —, e o sujeito é a modelação 3D. A escultura nunca esteve ao alcance.
2. ⛔ **A agulha era cega ao PLURAL.** A lista é `[".save_file()", ".pick_file()"]`, e
   `.pick_files()` **não contém** `.pick_file()` — o parêntesis fecha antes do `s`. A importação
   de malha desta família usa exactamente o plural, logo ela teria passado por aquele gate **sem
   uma palavra**, mesmo que a população o alcançasse.

⭐ *Uma agulha que fecha o parêntesis mede o verbo exacto e é cega ao irmão dele.*

### §21.5 — A cura, e o que ela FECHA

| peça | o quê |
|---|---|
| `ph2d_app_host::modal::pick_files` | a porta **plural**, que faltava — e a ausência dela é o que tinha empurrado a importação para fora (*uma porta que cobre metade dos verbos empurra a outra metade para fora dela*) |
| `export.rs` · `import.rs` da escultura | passam pela porta |
| `tests/it/modal_door.rs` (novo, desta família) | o gate irmão, com as **três** agulhas e piso de população de `150` ficheiros |
| `modal_door.rs` do vizinho | ganha `.pick_files()` na lista |
| `a_agulha_do_plural_nao_e_apanhada_pela_do_singular` | o CONTROLO da terceira agulha — sem ele, alguém que voltasse à lista de duas não reprovava |

⭐⭐ **A corrente está gateada nos TRÊS elos, e nenhum deles a cobre sozinho:**

1. *o chamador passa pela porta* — os dois gates `modal_door` (prova de mutação abaixo);
2. *a porta declara a paragem* — `the_door_times_what_goes_through_it`, que já existia;
3. *um laço congelado não envelhece a mensagem* — `a_frozen_loop_does_not_age_the_message_it_was_about_to_show`, que já existia.

**Prova de mutação: `2 de 2` sangram**, e as duas são o defeito REAL e não um sucedâneo — a `D1`
devolve o `export.rs` à forma exacta que o dono reportou, e a `D2` devolve o `import.rs` ao plural
que o gate do vizinho não veria.

### §21.6 — ⚠️ PARA O INTEGRADOR: isto toca FOUNDATIONAL

`crates/ph2d-app-host/src/modal.rs` ganha **uma função nova no fim** (`pick_files`), sem tocar em
assinatura nenhuma das duas que já existem — o ponto de extensão **append-only** que o ADR-0107
prescreve. ⚠️ É uma crate que as seis famílias consomem: se outra linha lhe tiver tocado na mesma
rodada, o conflito é textual e resolve-se por **união** (as três portas são independentes).

### §21.7 — ⏳ ABERTO, com o número: a classe tem ~20 membros e NÃO é desta wave

Medido nesta árvore: **~25 sítios abrem `rfd::FileDialog` directamente** contra **5** que passam
pela porta — e o doc da própria porta já dizia *«`rfd::FileDialog` em 12 arquivos do shell»*.

Toda mensagem escrita **logo a seguir** a um desses diálogos tem o mesmo destino: gravar projecto,
exportar imagem, exportar SVG, os nove do editor de áudio, o importador de imagem, os tokens.

⛔ **Não foram tocados nesta wave, de propósito:** são de outras famílias e de outra linha, o
número está medido, e a decisão de quando é do dono. ⭐ **O que fica no sítio certo é a FORMA da
cura:** a porta existe, e o que falta a cada família é um gate com a população dela — que é
precisamente o que este §21 acrescentou à escultura.

## §22 — «AS MENSAGENS ESTÃO CORTADAS COM …»: o aviso nunca coube no balão, e isso é PRÉ-EXISTENTE

> Report do dono, 22/09, logo a seguir à cura do §21: ***«as mensgens estão cortadas com … não
> consigo ler tudo»***.

⚠️ **Conte o DELTA:** `PROJECT_SCHEMA` 0, os três registos 0, `SCULPT_DOC_VERSION` 0, zero
contrato, zero ADR. ⛔ **Toca FOUNDATIONAL** (`ph2d-editor-core`: duas portas novas) e a **lei
partilhada** da `ph2d-mesh` muda de TEXTO — ver §22.7.

### §22.1 — A medição primeiro, e ela reatribui o defeito

O balão vive numa coluna de largura fixa (`360 px`); o que sobra para o texto depois da faixa de
severidade, do ícone e dos dois recuos é **`300 px`, e a `13 px` de corpo isso são ~`48`
caracteres** ([`ph2d_editor_core::toast::text_budget_px`], porta nova).

| frase | escrita | lida |
|---|---|---|
| com a cláusula da tinta fina (a do §20) | `95` | cortada aos `48` |
| **sem ela — a de ANTES do §20** | **`60`** | **cortada aos `49`** |
| em STL, o pior caso | `118` | cortada aos `50` |

⛔⛔ ⇒ ***o aviso do que um formato não carrega NUNCA foi legível.*** A cláusula do §20 piorou-o
(`60 → 95`); ela **não** o criou. *Uma wave que torna visível um defeito antigo é acusada de o ter
causado, e a única defesa é a medição da linha do meio.*

### §22.2 — ⭐⭐ A lei da partição, e ela sai de uma medição

Uma frase única com as duas metades **cabe** com `teste.obj` (`48`) e **estoura com um nome de
ficheiro real** — `retrato-da-personagem-v3.obj` ⇒ `62`. ⚠️ E a elisão corta o **FIM**, que é
exactamente onde o aviso está.

⇒ ***A metade que TEM de ser lida não pode ter parte variável.***

A saída passa a escrever **dois** balões:

1. a **confirmação**, que leva o nome do ficheiro — pode elidir, e o artista acabou de o escrever
   (o balão do `text_elide` mostra-o ao passar o rato);
2. o **aviso**, **sem uma única parte variável**, medido a caber no pior caso.

### §22.3 — ⛔ E o GATE apanhou um erro MEU a meio da cura

Eu medi o candidato curto (`Lost: mask, colour, pieces merged, fine paint`, `45` ✓) e depois
troquei **só o prefixo** — a cláusula continuava `fine paint (mesh resolution only)`, e o gate
reprovou com `68` no STL e `60` no PLY.

⇒ a **EXPLICAÇÃO** sai e as **LETRAS** ficam, que é a lei que a `line/UIUX` já escreveu em 20/09:
*um nome perde a explicação antes de perder letras*. Pior caso: **`68 → 45`**.

⚠️ **E não é `Paint Detail`, que cabia (`47`) e nomearia a fileira que o artista vê:** o `lost_by`
devolve texto **CRU** e o rótulo daquela fileira vem da tabela de traduções — nomear um controlo a
partir de uma frase não traduzida parte-se no dia em que alguém traduzir a fileira e não esta
linha.

### §22.4 — As duas PORTAS, e porque não são `const` públicas

| porta | o quê |
|---|---|
| `ph2d_editor_core::progress::toast_column_w()` | a largura da coluna |
| `ph2d_editor_core::toast::text_budget_px()` | o que sobra para o TEXTO — *a mesma conta que o pintor faz* |

⚠️ **UMA régua, DOIS consumidores** (o pintor e quem PERGUNTA se a frase cabe). Escrita duas
vezes, a resposta do gate e a do produto divergem no dia em que um recuo mudar de token — e a que
o artista vê é a errada. ⭐ O pintor leva um `debug_assert` que compara as duas contas, e é ele que
a mutação `B3` faz sangrar.

⛔ *Uma constante pública é um número que alguém copia; uma porta é um número que alguém pergunta.*

### §22.5 — O gate, e o CONTROLO dentro dele

`o_aviso_cabe_no_balao` (na família da escultura, a única que alcança a `ph2d-mesh` **e** a
`ph2d-editor-core`) mede **os três formatos × as duas colunas** pela régua do produto.

⚠️ **A barra é o PIOR CASO e não o comum:** o `.obj` guarda cor e peças, logo o aviso dele é curto
— uma barra medida ali deixaria passar exactamente a linha que o dono não consegue ler.

⭐ **O CONTROLO vem primeiro:** se o orçamento viesse a zero (uma porta partida, um token
renomeado) tudo seria elidido e o gate reprovaria **por um motivo que não é o dele**. A mutação
`B4` encolhe a coluna e é esse controlo que dispara.

**Prova de mutação: `4 de 5` sangram** (`muta_o_balao_do_aviso.sh`), com o `B5` a ser o CONTROLO
inerte. ⚠️ A população são **três** crates e o arnês corre as duas que **OBSERVAM** — a lei do
§20.6, aplicada à primeira.

### §22.6 — ⚠️ A SONDA fica versionada

`toast_orcamento_tests.rs` (`diag_o_que_cabe_no_balao`, `-- --nocapture`) imprime a frase elidida
ao lado da inteira: o estado **antes** da cura, as **quatro medições que decidiram a partição**, e
o que a saída escreve hoje. ⛔ As frases são literais ali de propósito — a `ph2d-editor-core`
**não alcança a `ph2d-mesh`** (medido antes de escrever, e teria sido um erro de compilação).

### §22.7 — ⚠️ PARA O INTEGRADOR

* `ph2d-editor-core` ganha **duas portas** (`toast::text_budget_px`, `progress::toast_column_w`),
  as duas **aditivas**; o `toast.rs` ganha um `debug_assert` no pintor.
* `ph2d-mesh::lost_by` **muda o TEXTO que devolve** (`not carried: …` → `Lost: …`, e a cláusula da
  tinta fina perde o parêntesis). ⚠️ Ela é partilhada com a modelação 3D, que a lê **directamente**
  — o texto dela muda também, e para melhor.
* `ph2d-i18n`: a chave `app.sculpt3d.export.exported_piece_s_kb` **larga o `{fmt}`**.
* `ph2d-app-sculpt3d/Cargo.toml` ganha duas **dev-dependencies** (`ph2d-text`, `ph2d-tokens`), só
  para a régua do gate.

### §22.8 — ⏳ ABERTO, e não é meu para decidir

⛔ **A coluna de `360 px` não nomeia recurso nenhum.** Ela é uma `const` com `LITERAL-PX-OK` e sem
medição ao lado, numa janela de ~`1900 px` — e é partilhada com as barras de trabalho. Alargá-la
tornaria legível toda a família de mensagens longas deste app, e é **decisão do dono / da linha da
UI**, não desta.

⏳ **E a modelação 3D continua a meter o aviso DENTRO da frase da confirmação** (o `{fmt}` do
template dela), logo a partição do §22.2 não a alcança. O prefixo mais curto ajudou-a; a partição
é da família dela.

## §23 — A TINTA FINA **SAI** NO FICHEIRO: um ladrilho por face, sem solver de UV

> A 2.ª metade da **P5** do [doc 27 §9](../27_o_estado_da_arte_de_onde_a_tinta_mora.md).
> A §20 ensinou a saída a **dizer** que a tinta fina não era carregada; esta ensina-a a
> **carregá-la**. Ordem do dono, 22/09: *«garanta que vai implementar de forma mais ágil e
> depois siga implementando»*.

⚠️ **Conte o DELTA:** `PROJECT_SCHEMA` 0, os três registos 0, `SCULPT_DOC_VERSION` 0, zero
contrato, zero ADR, zero pacote externo. ⛔ **A `ph2d-app-sculpt3d` ganha UMA dependência**
(`image`, só com a feature `png` — ver §23.8).

### §23.1 — ⭐⭐⭐⭐ Porque não há solver de UV nenhum, e isso é um TEOREMA

O [doc 27 §5](../27_o_estado_da_arte_de_onde_a_tinta_mora.md) já tinha medido a razão, e ela
decide a arquitectura inteira desta wave:

| esquema | o que ele achata | distorção de área |
|---|---|---|
| atlas | um pedaço **CURVO** da superfície | ⛔ **inevitável** (Egregium) — só o valor muda com o solver |
| **um ladrilho por face** | **uma face plana de cada vez** | ⭐ **ZERO**, por construção |

⇒ *o parametrizador é a **disposição***: cada face recebe um ladrilho seu, e o endereço
`(face, i, j, k)` que a [`ph2d_mesh_colors::indice`] já resolve **é** o texel.

⛔⛔ **E é por isso que isto NÃO exige a retopologia.** A família do atlas
([doc 26](../26_a_parametrizacao_como_atlas.md), cinco waves) precisa de um `GridMap`, que só
existe depois do botão; aqui não há desenrolamento para correr, logo **uma escultura crua sai
tão bem como um quad limpo**. *O caminho que parecia ser o pagamento das cinco waves de atlas
acabou por não precisar de nenhuma delas* — e elas continuam a ser a resposta certa para a
outra pergunta (uma textura com **poucas ilhas**, que um artista abre no Substance).

### §23.2 — ⚠️ O que o assado PERDE, e é o formato que obriga

A fronteira de uma face é **PARTILHADA** na retícula (é a diferença de espécie para o Ptex) e
**uma textura não sabe partilhar**: cada ladrilho leva a **cópia** da borda dele. ⭐ Os dois
lados escrevem o MESMO valor — a amostra é uma só —, logo a costura é **invisível em cor** e o
que ela custa é **filtragem**, que é o que a `FOLGA_EM_TEXELS = 2` paga (um texel para a
bilinear, o segundo para um nível de mip; ⛔ **não** são os `8` do `VAO_EM_TEXELS` do atlas, que
separa ILHAS de uma peça inteira).

⚠️⚠️ **E ele NÃO iguala densidades entre faces, de propósito.** O ladrilho tem o tamanho do
`lado` da retícula, que hoje é **UM** para a peça toda ⇒ uma face grande e uma pequena recebem o
mesmo número de texels. *Isso não é distorção do assado — é a retícula que ele copia
fielmente*, e curá-la é a **P2** do plano (o `R` por face). **Medido agora**, sobre as peças do
próprio dono (`p1`/`p99` da área das faces, raiz quadrada = densidade linear):

| peça | faces | dispersão da densidade |
|---|---:|---:|
| `Sculpt_Blender` | 8 291 | **4,89×** |
| `_base_sculpt` | 18 432 | **6,74×** |
| `_remesh_sculpt` | 5 445 | 4,06× |
| `nossa_com_calota` | 21 914 | 3,12× |
| `sculpt_antes` | 13 824 | **18,26×** |

⇒ *a P2 tem defeito real e medido*, e a §5 do doc 27 dá o chão dela: com o `R` por face
quantizado a potências de dois o pior caso é **`√2 = 1,41×`**.

### §23.3 — ⛔⛔ A cura não é um número: é a INVERSÃO do eixo `v` a viver num sítio só

Uma textura conta as linhas **de cima para baixo** e um `.obj` conta `v` **de baixo para
cima**. A inversão vive em [`assar::coord`] e **em mais lado nenhum** — escrevê-la no escritor
do ficheiro poria a mesma lei em dois sítios, e *o dia em que nascesse o segundo formato ela
viajaria para um só*. A mutação `A2` faz o gate do canto sangrar.

### §23.4 — ⭐ A DILATAÇÃO não é acabamento

Sem ela o assado tem um defeito **na primeira olhada**: um `uv` sobre a borda de uma face cai
**ENTRE** dois centros de texel, logo a bilinear lê um bloco `2×2` que inclui um texel de fora —
e a metade vazia do ladrilho de um triângulo está *dentro* desse bloco ao longo da hipotenusa.
⇒ a borda de **toda** face sairia com uma linha escura.

⚠️ **A média dos vizinhos COBERTOS, nunca o primeiro que aparece:** com o primeiro, o resultado
depende da ordem em que os oito são visitados e dois texels simétricos da mesma borda ficam de
cores diferentes.

⛔⛔ **E o GATE dela teve de ser reescrito, porque a 1.ª redacção media a GRANDEZA ERRADA.** Ela
exigia que todo vizinho de um texel **opaco** fosse opaco — e um texel **dilatado** também é
opaco, logo a condição cascateava para fora do ladrilho e reprovava sobre produto correcto. A
régua honesta entra pela **porta pública**: varre o interior e a borda de cada face **em
coordenadas da própria face**, interpola os `uv` dos cantos (*que é o que um motor faz*) e
confere o bloco `2×2` que a bilinear leria. *Uma régua que recalculasse o ladrilho afirmaria
sobre a cópia dela, nunca sobre o assado.*

### §23.5 — ⭐⭐ O ESCRITOR: dois acumuladores, e o segundo é a lei

`write_obj` **delega** em `write_obj_com_uv(pieces, &[], "")` ⇒ o caminho sem textura é
**byte a byte** o de sempre (gate `sem_textura_o_obj_e_byte_a_byte_o_de_sempre`, e é ele que faz
toda a família de gates que já media o OBJ passar a medir o caminho novo).

⛔⛔ **Os índices de `vt` contam-se num acumulador PRÓPRIO**, porque uma peça sem textura
acrescenta **vértices** e **nenhum** `vt`: somar os dois no mesmo contador desloca a tinta de
todas as peças a seguir à primeira sem textura — e *o ficheiro abre sem queixa, com a tinta no
sítio errado*. A fixtura do gate é exactamente esse arranjo (peça 0 sem textura, peça 1 com), e
a mutação `A6` sangra nele.

⚠️ **`Kd 1 1 1` e `map_Kd` sem pasta** são as duas metades que um visualizador obriga, e as duas
são **silenciosas** quando erradas: a primeira escurece a tinta que o artista pintou, a segunda
não resolve no computador de quem abrir.

### §23.6 — ⚠️ Ter tinta fina e PERDÊ-LA são perguntas diferentes

A tabela do formato passa a dizer que o **OBJ carrega** (`keeps_fine_paint` = `Obj`), e é isso
que cala o `fine paint` no aviso da §20. ⛔ **Mas o `keeps_fine_paint` sozinho MENTE numa
célula:** uma peça **recusada por tamanho** tem tinta fina e ela **fica para trás** ⇒
`perdeu_tinta_fina(fmt, tem, assados)` = `tem && (!fmt.keeps_fine_paint() || !assados.alguma())`,
com a tabela-verdade das quatro células num gate e a mutação `A10` a sangrar na que custa.

⭐⭐ **E o gate da §20 previu a própria morte por escrito.** A redacção dele acabava em
`assert!(!fmt.keeps_fine_paint())` com a frase *«se isso é verdade, a metade de cima deste gate
deixou de descrever o produto»* — e passou a ser verdade hoje. Ele foi reescrito com a **morte
da premissa visível no diff**, e a população passa a partir-se **pela tabela** (quem carrega não
avisa, quem não carrega avisa sempre) com um piso em cada metade. *Uma lista escrita à mão ali
divergiria no dia do quarto formato, que é exactamente o que o `lost_by` existe para impedir.*

### §23.7 — ⛔ Três ficheiros e não um, e uma textura POR PEÇA

Um `.obj` não embute imagem: ele aponta para um `.mtl`, que aponta para o `.png`. Os três saem
**lado a lado**, com o nome derivado do que o artista escreveu (`teste.obj` ⇒ `teste.mtl` +
`teste_0.png`), e a metade que os nomeia é **separada de quem grava**, porque *esta metade é
testável* — o corte que o `sheet_export` da shell já fazia pela mesma razão.

⚠️ **Uma textura por PEÇA e não uma para a cena:** cada peça tem o plano dela, com a topologia
dela, e um assado único obrigaria a re-endereçar as amostras de todas num espaço comum — *que é
exactamente o trabalho que a família do atlas faz e que esta não precisa de fazer*.

⛔ **E a ORDEM é load-bearing:** as texturas e o material são gravados **ANTES** do `.obj`,
porque é a existência deles que decide se o `.obj` pode apontar para lá. *Um `.obj` a apontar
para um material que não existe abre PRETO no destino e ninguém sabe porquê* — em erro a função
devolve `None`, diz QUAL ficheiro falhou, e o `.obj` sai **sem** textura, que é um ficheiro
coerente.

### §23.8 — ⚠️ PARA O INTEGRADOR

* `ph2d-mesh-colors` ganha `assar.rs` (**aditivo**, e a crate continua com **zero**
  dependências) e re-exporta `Assado`/`Recusa`/`Relatorio`/`assar`.
* `ph2d-mesh` ganha `UvDaPeca`, `write_obj_com_uv` e `write_mtl`, os três **aditivos**; o
  `write_obj` **delega** e a saída dele é byte-idêntica. ⚠️ `MeshFormat::keeps_fine_paint`
  **muda de resposta** para o `Obj` — a modelação 3D lê a mesma tabela e passa a beneficiar dela
  no dia em que assar (hoje ela passa `false` por medição, logo nada muda lá).
* `ph2d-app-sculpt3d/Cargo.toml` ganha **`image` com `default-features = false, features =
  ["png"]`** — a mesma declaração que a `shells/desktop` já tem para o exportador da folha de
  sprites. ⚠️ É a **primeira** dependência externa nova desta linha inteira.
* `ph2d-i18n`: **três chaves novas** em `app_sculpt3d.rs`.
* O censo da fiação vai de **21 para 23** elos (`S7` o assado corre · `S8` o obj passa pelo
  escritor com uv), e o nome do teste muda com ele.
* ⚠️ **Uma isenção NOMEADA no censo do HR-15** (`export_assado.rs`): o nome do material
  (`ph2d_<n>`) é um token **dentro de um par de ficheiros** — o `usemtl` e o `newmtl` têm de
  casar letra a letra e quem os lê é outro programa. *Traduzi-lo faria os dois discordarem no
  dia em que alguém mudasse de língua, e o destino abriria a peça sem textura sem uma queixa* —
  a mesma razão pela qual o aviso da §22 não nomeia a fileira `Paint Detail`.

### §23.9 — ⛔ O PRÉ-VOO apanhou DUAS âncoras que esta wave matou

O `muta_a_saida_da_tinta.sh` (o arnês da §20) leu **`5 de 7`**: a `S1` ancorava em
`keeps_fine_paint` a devolver `false` e a `S5` na chamada crua do `lost_by`, e **as duas linhas
mudaram hoje**. ⭐ *Uma âncora que casa zero lê-se, num placar, exactamente como uma mutação que
sobreviveu* — e o pré-voo apanhou-as **em segundos e sem correr um teste**.

⚠️ **A `S1` foi re-ancorada ao contrário, e isso é a lei a mudar:** a mutação de ontem era
*«o formato mente e diz que CARREGA»*; hoje o OBJ carrega de verdade, logo o que há a mutar é
**os três** passarem a dizer que sim — e aí o `.ply` e o `.stl` calam-se sobre uma perda que
acontece. *A pergunta do arnês é a mesma; o que se inverteu foi o produto.*

### §23.10 — O placar

* **Mutação: `12 de 13` sangram** (`muta_a_tinta_que_sai.sh`; o `A13` é o CONTROLO inerte), com
  a população a ser as **TRÊS** crates que OBSERVAM — a lei do §20.6 aplicada à primeira.
* Pré-voo dos **dez** arneses: **120 âncoras, todas a casar uma vez**.
* Gates novos: `5` no assado (o canto · a fronteira partilhada · a dilatação · a recusa · o
  nível base), `3` no escritor, `3` no app.

### §23.11 — ⏳ O que fica ABERTO, com o mecanismo

* **A P2 — o `R` por FACE.** A dispersão está medida acima (`3,1×` a `18,3×` nas peças do dono),
  e o chão teórico é `1,41×`. ⛔ Ela **não** é uma afinação do assado: ela muda o endereçamento
  da retícula (as arestas são **partilhadas**, logo uma aresta entre duas faces de resolução
  diferente precisa de uma resolução própria — o `max` das duas, que com potências de dois faz a
  face grossa ler um **subconjunto exacto** da fina).
* **O tecto é `8192` e o recurso é a PLACA de quem abrir** (`max_texture_dimension_2d`, o mesmo
  que a folha de sprites já usa). ⛔ Ele **não** limita o peso do ficheiro, que é outra grandeza
  e não foi medida.
* **O relógio do assado não foi varrido.** Ele corre **uma vez por exportação** e não por
  quadro, logo não compete com o carimbo — mas o número não existe.
* **A modelação 3D continua a não assar** (ela passa `false` ao `lost_by`): a porta está aberta
  do lado da tabela, e quem a usar herda os três ficheiros de graça.

## §24 — *«o Blender não consegue importar e não dá nenhuma mensagem»* — o ficheiro IMPORTA, e a investigação achou DOIS defeitos meus que não são esse

> Report do dono, 22/09, sobre o `.obj` que a §23 produz.

⚠️ **Conte o DELTA: tudo a 0** (`PROJECT_SCHEMA`, os três registos, `SCULPT_DOC_VERSION`), zero
contrato, zero ADR.

### §24.1 — ⭐⭐⭐ O ALVO foi CORRIDO sobre um ficheiro NOSSO, e ele importa

O Blender **5.2.2 LTS está instalado nesta máquina**, e o §0.9 diz o que fazer com ele: *um alvo é
um oráculo que se CORRE, nunca um fonte que se lê* — e **ler o que ele diz sobre um ficheiro
NOSSO é livre** (a saída não é obra baseada no programa).

⭐ **A sonda produz o ficheiro pelo caminho do produto** ([`export_assado_sonda`], versionada,
`PH2D_SONDA_SAIDA=<pasta>`): as MESMAS chamadas que o `export` faz, sobre a peça da `=52` com o
plano ao `8x`. Depois:

```text
blender --background --factory-startup --python <importa e conta>
```

**O veredito, duas vezes:**

| ficheiro | resultado | objectos | vértices | polígonos | UV | material |
|---|---|---:|---:|---:|---|---|
| `teste.obj` | `{'FINISHED'}` | `1` | `738` | `768` | `UVMap` | `ph2d_0` |
| o mesmo com cor por vértice (`v x y z r g b`) | `{'FINISHED'}` | `1` | `738` | `768` | — | — |

⇒ ***o ficheiro que o nosso escritor produz importa***, com as coordenadas e o material. E a
segunda linha existe porque a minha primeira hipótese era essa: o dono **pintou**, logo a malha
dele tem cor por vértice e o `v` leva **seis** números em vez de três — *e o oráculo refutou-a*.

### §24.2 — ⭐ O RELÓGIO também está ilibado, com número

Esta casa já pagou uma vez o defeito *«exportar congela a janela e o KDE dá-a por morta»* (o
modelador 3D, `8 min 17 s → 6,4 s`), e *uma janela dada por morta é indistinguível, para quem a
usa, de um ficheiro que não saiu*. ⇒ medido ([`export_assado_relogio`], versionada, `--release`):

| peça | faces | degrau | textura | assar | png | **total** |
|---|---:|---:|---:|---:|---:|---:|
| a cena `=52` | `768` | `8×` | `364 px` | `0,00 s` | `0,00 s` | **`0,00 s`** |
| média | `12 288` | `8×` | `1 443 px` | `0,02` | `0,00` | **`0,03 s`** |
| de fábrica | `65 712` | `8×` | `3 341 px` | `0,18` | `0,00` | **`0,18 s`** |

⇒ *a saída não congela nada*, e o pior caso medido é `1 %` do limiar em que o KDE começa a
duvidar da janela.

### §24.3 — ⛔⛔⛔ E a investigação achou um defeito MEU, da classe que esta linha já tinha curado

O [`ph2d_mesh_colors::assar`] chamava `indice_tri`/`indice_quad` **sem a guarda
[`Topologia::descreve`]** — *a lei que a [§14](#) desta mesma linha estabeleceu depois de o dono
levar um `panic` na cara* (`topo.rs:239 — index out of bounds: the len is 196608 but the index is
196608`).

⚠️ **A `Topologia` guarda `4` entradas por face**, logo uma lista de faces **mais longa** do que a
que a construiu indexa fora de alcance — `index out of bounds` **no meio de uma exportação**, e o
`.obj` nunca chega a ser escrito. ⛔ **E a metade CURTA é a pior**, como lá: com menos faces nada
sai de alcance, o laço acaba sozinho, e a textura fica com tinta **válida no sítio errado**, em
silêncio.

⭐⭐ *Um assado nasceu uma wave depois daquela cura e sem nenhuma das duas metades dela.* ⇒
`Recusa::NaoDescreve { faces, plano }`, com a guarda **antes de qualquer indexação** e o gate a
exigir as **duas** metades (mais faces · menos faces) mais o CONTROLO de que a malha certa assa.
⭐ E a exaustividade do `match` no gate da recusa **é o censo**: uma recusa nova **não compila**
até alguém dizer o que ela significa.

### §24.4 — ⭐ E um segundo, que o oráculo mediu: a COBERTURA saía no ficheiro

O assado guarda `rgba`, e o alfa dele **não é transparência — é a COBERTURA** (a marca de quem
recebeu uma amostra ou uma dilatação), que é por onde os gates distinguem *um texel preto* de *um
texel vazio*. Eu escrevia esse canal no `.png`.

⚠️ *Escrever a cobertura entrega ao destino uma grandeza interna com cara de alfa.* ⇒ a porta
[`Assado::rgb`] deixa a cobertura em casa e o ficheiro leva **três** canais, com o gate a afirmar
as duas metades (a cor não muda · o `rgba` **mantém** o alfa, senão alguém «simplifica» o assado
e apaga a régua de toda a família de graça). ⭐ De graça, o `.png` pesa menos.

⛔ **E o oráculo diz que esta NÃO era a causa:** o Blender põe o material em
`blend_method: HASHED` **com e sem** o canal, e a socket `Alpha` fica por ligar nos dois casos.
*Uma cura medida que não explica o report continua a ser uma cura; o que não se pode é dizer que
ela fecha o assunto.*

### §24.5 — ⏳ O que NÃO foi reproduzido, e é honesto dizê-lo

Com o formato, a cor por vértice e o relógio **eliminados por medição**, e com o ficheiro a
importar no alvo, **não consegui reproduzir a falha do dono nesta máquina**. O que sobra são
causas de ambiente que um handoff não pode adivinhar — onde o diálogo do sistema pousou os três
ficheiros, se os três ficaram lado a lado, qual Blender ele usou.

⇒ **a cura para isso é um INSTRUMENTO e não uma hipótese:** a saída passa a dizer, no terminal, o
caminho do `.obj`, o tamanho dele, quantas texturas escreveu e de que lado
([`export_assado::diz_o_que_escreveu`]). ⚠️ **O balão não serve** — ele tem `48` caracteres
(§22) e não cabe um caminho —, e o terminal é *a única superfície onde o artista e eu olhamos
para os MESMOS números*. ⛔ Ela corre **depois** de o `.obj` estar em disco: escrita antes, ela
mede o ficheiro que lá estava de uma exportação anterior, e *um instrumento que mede o ficheiro
errado é pior que nenhum — ele CONFIRMA* (a lição que a `line/components` pagou em 19/09 com o
`fotografa_cena.sh`).

### §24.6 — O placar

* **Mutação: `14 de 15` sangram** (`muta_a_tinta_que_sai.sh`; o `A16` é o CONTROLO), com as duas
  novas a cobrir a guarda e a ordem dos canais.
* Pré-voo dos dez arneses: **122 âncoras**, todas a casar uma vez.
* Duas sondas **versionadas** que não se apagam: a que produz o ficheiro e a que mede o relógio.
  *As duas leituras — a de hoje e a do dia em que isto voltar — valem uma pela outra.*
* Portão: clippy `-D warnings` **zero** · censos da árvore COMBINADA **127/127** (controlo do
  filtro `12 de 12`) · `nextest-impacted` **`18 606` de `18 607`** na 1.ª corrida.
* ⭐⭐ **E o único ✗ fechou com as TRÊS assinaturas da família de flakes de fan-out** (§5.0), a
  última delas a mais forte que esta casa conhece: o
  `the_cost_of_a_player_is_linear_in_their_number` (`ph2d-physics-ecs`) é **membro já NOMEADO**
  da lista · o diff desta wave tem **zero** linhas naquela crate · ele passa **`3` de `3`
  sozinho a `load 140`–`145`**, ou seja *dez vezes* a carga em que reprovou · e **uma SEGUNDA
  corrida da MESMA árvore leu `18 607` de `18 607`**. ⚠️ *Um defeito de lógica reprova o mesmo
  caso sempre; só um recurso partilhado troca de vítima entre corridas* — e é por isso que a
  re-corrida sozinha vem ANTES de olhar para o próprio commit.

### §24.7 — ⚠️ *«importa se arrastar, não importa pelo diálogo»* — as DUAS portas do alvo, medidas

> Report do dono, 23/09, a fechar o §24.

⭐ **O alvo foi corrido pelas DUAS portas sobre o MESMO ficheiro nosso** — porque o diálogo *File ▸
Import* e o ARRASTO chamam o mesmo operador com argumentos **diferentes**: o arrasto passa só o
`filepath`, e o navegador de ficheiros passa `directory` + a colecção `files`, que é como ele
importa vários de uma vez.

| porta | como o alvo é chamado | resultado |
|---|---|---|
| arrasto | `filepath` | `FINISHED` — `1` objecto, `738` v, `768` f, UV, material |
| diálogo | `filepath` + `directory` + `files` | **idem** |
| só `directory` + `files` | sem `filepath` | **idem** |
| diálogo + `filter_glob` | como o navegador o passa | **idem** |

⇒ **as quatro leem o mesmo**, logo *a diferença que o dono vê não está no ficheiro nem no operador*.
⛔ E a hipótese de ambiente mais forte fica **fechada com medição**: o Blender desta máquina é
pacote nativo (`pacman -Qo` ⇒ `blender 17:5.2.2-1`), **não** Flatpak nem Snap — logo não há caixa de
areia a deixar passar um ficheiro largado e a esconder o mesmo ficheiro ao navegador de dentro.

⚠️⚠️ **O que sobra é do lado do alvo e eu não o posso medir daqui:** o navegador de ficheiros do
Blender **lembra as opções do operador entre invocações** e o arrasto não as lê. *Uma opção de
importação trocada numa tentativa anterior sobrevive no diálogo e é invisível no arrasto* — e é
exactamente essa a forma do report. ⇒ o passo que o desempata é **um**: abrir o diálogo e carregar
em *Restore Operator Defaults* (ou conferir a barra lateral dele) antes de importar.

⛔ **Nada disto muda uma linha do produto**, e é por isso que fica aqui e não numa cura: *uma cura
que não explica o report continua a ser uma cura; o que não se pode é inventar uma que não tem
mecanismo medido*.

---

## §25 — ⭐⭐⭐⭐ A P2: a retícula deixa de ter UM lado — o nível é da FACE

> Ordem do dono, 23/09: *«pode seguir implementando»*. O item aberto que a §23.11 nomeia.

⚠️ **Conte o DELTA: tudo a 0** — `PROJECT_SCHEMA`, os três registos, `SCULPT_DOC_VERSION`; zero
contrato, zero ADR, zero pacote externo. **E zero mudança de produto:** nada no app produz hoje um
plano graduado, e um plano uniforme sai **byte a byte** o de antes.

### §25.1 — ⭐⭐⭐ A medição veio ANTES da primeira linha, e REFUTOU o número do handoff

O [`examples/mede_o_r_por_face.rs`](../../../crates/ph2d-mesh-colors/examples/mede_o_r_por_face.rs)
corre sobre o corpus do dono e mede a **densidade linear de amostras** (`lado / √área`) face a face:

| peça | faces | hoje (uniforme) | com o `R` por face | amostras |
|---|---:|---:|---:|---|
| `nossa_com_calota` | 21 914 | `3,12×` | **`1,90×`** | `1,40 M → 1,22 M` (**−13 %**) |
| `Sculpt_Blender` | 8 291 | `4,88×` | **`1,97×`** | `0,53 M → 0,55 M` (`+4 %`) |
| `_base_sculpt` | 18 432 | `6,74×` | **`1,95×`** | `1,08 M → 1,43 M` (`+32 %`) |
| `sculpt_antes` | 13 824 | **`18,26×`** | **`1,92×`** | `0,88 M → 1,03 M` (`+18 %`) |

⛔⛔ **E a §23.11 escreveu o chão errado.** Ela diz *«com o `R` por face quantizado a potências de
dois o pior caso é `√2 = 1,41×`»* — **as duas afirmações não são a mesma grandeza**: o `√2` é o
desvio ao alvo de **UMA** face (meia escada) e a dispersão é uma razão entre **DUAS**, logo
`√2 × √2 = 2`. *A sonda lê `1,90`–`1,97`, e o `2,37` da `sculpt_antes` a `k` baixo é o **CHÃO DA
ESCADA** a morder* — uma face menor que `1/alvo` pede um nível NEGATIVO e o corte em `0` deixa-a
mais fina do que o alvo. Isso é o fim da escada, não um defeito da lei, e está escrito no doc da
[`niveis_por_area`].

### §25.2 — ⭐⭐⭐⭐ A lei: a aresta leva o MÁXIMO, e a face grossa lê um SUBCONJUNTO EXACTO

A fronteira é **partilhada** (é a diferença de espécie para o Ptex), logo ela só pode ter UMA
resolução ⇒ `nivel_da_aresta = max(vizinhos)`. A face grossa conta `t` na retícula DELA e a aresta
guarda as amostras na DELA, e o passo `le / lf` é **inteiro** porque as duas são potências de dois
⇒ a amostra da face cai **em cima** de uma da aresta, sem arredondar e com as duas pontas
preservadas.

⛔ **Tomar o MÍNIMO apagaria detalhe que o artista pintou do lado fino**, e a média não é potência
de dois. ⭐ **A multiplicação vive no [`enderecos::indice`] e em mais lado nenhum** — *uma segunda
cópia dela é como metade de uma peça fica com a tinta da vizinha*.

### §25.3 — ⚠️ O que NÃO muda, e é a metade que dá direito ao resto

Com um nível uniforme os dois prefixos voltam a ser **produtos** — `off_aresta[id] = id·(L−1)` e
`off_interior[f] = f·interior(L)` —, que é **exactamente** a aritmética que o shader ainda faz.
⇒ *o caminho da placa fica correcto sem uma linha de WGSL nova*, e a suíte da crate passou de
`36/36` para `42/42` **sem um gate antigo se mexer**. O gate que o afirma leva as duas metades
(o uniforme é um produto · `Topologia::nova(k)` dá a MESMA topologia que `regraduada(&[k; n])`).

### §25.4 — ⛔⛔ Quem ainda assume um lado só RECUSA, e não adivinha

| consumidor | com um plano graduado | porquê |
|---|---|---|
| o **assado** | ~~`Recusa::Graduado`~~ — **ASSA**, desde a §26 | a recusa durou **um commit**: o empacotador aprendeu o trabalho dela |
| o **device** | **desarma** (`armado = 0`) | entrega a cor por VÉRTICE, que é o caso base desta família e está certo |
| o **pincel** | **não recusa** | ele passou a ler `lado_da_face(fi)` DENTRO do laço, e fica correcto de graça |

⭐ *A resposta errada com a confiança da certa é exactamente o que estas duas guardas existem para
não entregar*, e a exaustividade do `match` no gate da recusa é o censo: **uma recusa nova não
compila** até alguém dizer o que ela significa.

### §25.5 — ⛔⛔ O portão apanhou DUAS coisas minhas, e a segunda estava debaixo da primeira

**(a) O clippy acusou uma asserção VÁCUA:** `com[5] >= NIVEL_MAX - 5` num `u8` é
`com[5] >= 0` — **sempre verdade**. *Uma asserção que não pode falhar é comentário com sintaxe de
código.*

**(b) E por trás dela a fixtura NÃO era uma corrente:** ela emitia um triângulo por coluna, e dois
deles partilham só um **VÉRTICE**. A cerca do salto corre por **ARESTA** ⇒ ela não propagava nada, e
o gate ficava verde sobre uma corrente que não existia. Hoje é uma tira a sério (`A_i`/`B_i`
alternados) e a escada é afirmada **EXACTA**: `5 · 4 · 3 · 2 · 1 · 0`.

⚠️ *Sem o lint eu tinha shipado um gate que mede o nada sobre uma fixtura que não contém o
fenómeno* — as duas metades da mesma família, uma a esconder a outra.

### §25.6 — ⭐⭐ E o PRÉ-VOO apanhou QUATRO âncoras que ESTA wave matou

O `muta_a_lei_da_reticula.sh` leu **`12` de `16`** em segundos e **sem correr um teste**: o
`indice` perdeu o argumento `lado` (ele é propriedade da FACE) e o `total` deixou de ser um produto,
logo as âncoras `M3`, `M5`, `M7` e `M8` casavam **zero** vezes. ⚠️ E a re-ancoragem do `M5` falhou à
primeira porque o `cargo fmt` colapsou o `total` numa linha só — *a armadilha que o pré-voo existe
para apanhar, apanhada pelo pré-voo*. Depois de re-ancorado: **`16` de `16` sangram**.

⚠️⚠️ *É a lei do §20.7 a cobrar-se de quem a escreveu: mover ou reescrever código parte arneses em
duas espécies, e a que fica MUDA é a que se leva para o `main`.*

### §25.7 — O placar

* **Mutação: `11 de 12` sangram** ([`muta_o_r_por_face.sh`](../ferramentas/muta_o_r_por_face.sh); o
  `P12` é o CONTROLO e não pode), com a população a ser as **três** crates que OBSERVAM
  (`ph2d-mesh-colors` · `ph2d-mesh-render` · `ph2d-app-sculpt3d`).
* O `muta_a_lei_da_reticula.sh` re-ancorado: **`16 de 16`**.
* Pré-voo dos **onze** arneses: **`143` âncoras**, todas a casar uma vez.
* `nextest-impacted` **`18 613` de `18 613`** · clippy `-D warnings` **zero** · `fmt` limpo ·
  censos da árvore COMBINADA **`127/127`** (controlo do filtro `12 de 12`).
* Censo da fiação da tinta fina: **`23` → `25` elos** (o pincel a ler o lado da FACE · o device a
  desarmar) — *as duas leis não têm prova de comportamento alcançável, porque nenhum gesto produz
  hoje um plano graduado e a do device vive atrás de um adaptador*.

### §25.8 — ⏳ O que fica ABERTO, com o mecanismo

* **O EMPACOTADOR do assado.** A cura da recusa `Graduado` é ladrilhos de tamanhos diferentes numa
  textura só — e aí a P2 chega ao FICHEIRO, que é onde o dono a vê.
* **O device.** O registo do shader descreve a retícula com um `lado` e o bloco das arestas com
  `id × (lado − 1)`; com níveis por face as duas contas pedem um **offset por aresta**, que é buffer
  novo no bind group.
* **Quem ESCOLHE os níveis no produto.** A lei existe (`niveis_por_area`) e precisa das ÁREAS, que
  esta crate não conhece — o chamador é a família, e a pista `Paint Detail` passa a pedir uma
  densidade em vez de um degrau.
* **A PERSISTÊNCIA de um plano graduado.** O formato guarda um `nivel` só; ou ele passa a guardar a
  lista, ou a graduação é **re-derivada** no load a partir das áreas — e aí uma mudança na lei
  relayouta ficheiros gravados **em silêncio**, o que pede um degrau.

---

## §26 — ⭐⭐⭐⭐ O EMPACOTADOR: a P2 chega ao FICHEIRO

> Mesma ordem, a seguir à §25. **A recusa `Graduado` da §25.4 durou UM commit** — ela era a
> fronteira honesta enquanto o empacotador não existia, e existe agora.

⚠️ **Conte o DELTA: tudo a 0** (`PROJECT_SCHEMA`, os três registos, `SCULPT_DOC_VERSION`), zero
contrato, zero ADR. **E zero mudança de produto:** nada no app produz hoje um plano graduado.

### §26.1 — ⭐⭐⭐ O que ele compra no ficheiro, medido nas peças do dono

Ao degrau que o artista escolhe (`8x`), pelo caminho do produto
([`examples/mede_o_r_por_face.rs`](../../../crates/ph2d-mesh-colors/examples/mede_o_r_por_face.rs)):

| peça | textura uniforme | textura por face | dispersão |
|---|---|---|---|
| `Sculpt_Blender` | `1196²` @ `46,9 %` | `1196²` @ **`47,9 %`** | `4,88× → 1,97×` |
| `_base_sculpt` | `1768²` @ `44,2 %` | `1854²` @ **`50,2 %`** (`+4,9 %` de lado) | `6,74× → 1,95×` |
| `sculpt_antes` | `1534²` @ `47,1 %` | `1578²` @ **`50,5 %`** (`+2,9 %`) | **`18,26× → 1,92×`** |
| `nossa_com_calota` | `1937²` @ `47,3 %` | **`1845²`** @ `45,7 %` (**`−4,8 %`**) | `3,12× → 1,90×` |

⇒ *o lado da textura mexe-se entre `−4,8 %` e `+4,9 %` e o aproveitamento **sobe** em três das
quatro*, enquanto a dispersão cai `1,6×` a `9,5×`. **Não há troca a declarar.**

### §26.2 — ⚠️ Prateleiras, com as MAIORES primeiro

As peças entram por lado decrescente (com o índice a desempatar, para a saída ser **determinista**);
cada prateleira começa com a mais alta que ainda cabe, logo **a altura dela é a da primeira** e
nenhuma peça a faz crescer depois. ⛔ *Ordenar ao contrário obriga toda prateleira a crescer para a
última peça, que é onde um empacotador ingénuo desperdiça metade da textura.*

⭐⭐⭐ **E com um plano UNIFORME ele devolve a grelha de antes AO TEXEL** — por aritmética e não por
acaso: com `n` peças iguais de lado `s`, o menor `W` que fecha é `ceil(√n)·s`, que é o
`cols × ladrilho` que o ficheiro calculava à mão. *É isso que permite haver **UM** caminho em vez de
dois*, e o gate afirma-o nos três níveis.

⚠️ A busca do `W` sobe **de um em um** durante `s_max` tentativas e só depois cresce por fracção: é
esse trecho fino que garante o óptimo do caso uniforme (ele está a menos de `s` acima do palpite da
área), e a fracção é o que impede uma peça enorme de varrer o tecto texel a texel.

### §26.3 — ⛔⛔ A prova de mutação achou CÓDIGO A MAIS

O `P15` mutava um `if y + s > w { return None }` **por peça** e **SOBREVIVEU** — porque a última
prateleira é a mais funda por construção (o `y` só cresce) e o teste final já respondia por todas.
⭐ **A cura não foi um gate novo: foi APAGAR a linha.** *Uma linha que a mutação não consegue matar
não é lei, é comentário com sintaxe de código* — e o preço dela era um ramo por peça no laço.

### §26.4 — ⛔⛔⛔ E o gate que faltava é a NÃO-SOBREPOSIÇÃO, que a COR não acusa

Duas faces empilhadas no mesmo sítio dão `uv` **perfeitamente válidas** e uma textura em que a
segunda escreve por cima da primeira — e *o canto de cada uma continua a ler a cor do vértice dela*
se elas partilharem o vértice, logo **todos os gates de COR passam**. O que as separa é a
**GEOMETRIA da disposição** ⇒ `nenhum_ladrilho_pisa_outro`, que lê o rectângulo de cada face dos
`uv` que saem, **crescido da folga**, e exige que eles sejam disjuntos e caibam na textura.

⭐ Ele é o **único** que mata o `P14` (a prateleira a encolher para a última peça), e leva o
CONTROLO dentro: *a fixtura tem de ter ladrilhos de pelo menos três tamanhos, senão isto mede uma
grelha e não um empacotador*.

### §26.5 — O placar

* **Mutação: `13 de 14` sangram** (o `P12` é o CONTROLO), com as três novas do empacotador.
* Pré-voo dos **onze** arneses: **`142`** âncoras, todas a casar uma vez.
  ⛔⛔ **CORRIGIDO em 23/09, e a correcção é o achado:** esta linha era FALSA. Medido contra
  o `HEAD` desta wave, o `muta_a_tinta_que_sai.sh` tinha **DUAS** âncoras mortas (`A4` e `A5`,
  as duas apontadas ao `assar.rs` que o empacotador reescreveu **nesta mesma wave**) e ele
  **tem** pré-voo. ⇒ *o pré-voo só vale o CONJUNTO em que foi corrido*, e um sumário que diz
  «os onze» depois de correr menos que onze é o instrumento a mentir sobre si mesmo — a mesma
  forma que este ficheiro já registou quando o pré-voo imprimia «ZERO testes corridos» depois
  de ter corrido a suíte inteira.
* `nextest-impacted` **`18 615` de `18 615`** · clippy `-D warnings` **zero** · `fmt` limpo ·
  censos da árvore COMBINADA **`127/127`**.
* A suíte da crate: `36` → **`44`** testes, e **nenhum gate antigo se mexeu**.

### §26.6 — ⏳ O que fica ABERTO (a §25.8 menos o empacotador)

* **O device.** O registo do shader descreve a retícula com um `lado` e o bloco das arestas com
  `id × (lado − 1)`; com níveis por face as duas contas pedem um **offset por aresta** — buffer novo
  no bind group. Até lá ele **desarma**, e o artista vê a cor por vértice.
* **Quem ESCOLHE os níveis no produto.** A lei existe (`niveis_por_area`) e precisa das ÁREAS, que a
  crate não conhece ⇒ o chamador é a família, e a pista `Paint Detail` passa a pedir uma
  **densidade** em vez de um degrau.
* **A PERSISTÊNCIA de um plano graduado.** O formato guarda um `nivel` só; ou passa a guardar a
  lista, ou a graduação é **re-derivada** no load a partir das áreas — e aí uma mudança na lei
  relayouta ficheiros gravados **em silêncio**, o que pede um degrau.
* ⚠️ **E a ordem entre os três é load-bearing:** sem o device o artista não VÊ a P2, e sem quem
  escolhe os níveis nada a produz. *O empacotador veio primeiro porque, sem ele, o dia em que
  alguma coisa graduasse seria o dia em que a exportação deixava de levar a tinta fina.*

---

## §27 — ⭐⭐⭐⭐ O DEVICE: a placa desenha um plano GRADUADO

> **Ordem do dono:** *«Pode seguir implementando»* (21/09). O §26.6 deixou três itens abertos e
> disse por escrito qual vinha primeiro: *«sem o device o artista não VÊ a P2»*.

### §27.1 — O que mudou, numa frase

O registo achatado que o shader lê passou de **`10` para `19`** palavras por face, e com as nove
palavras novas o gémeo em WGSL resolve um endereço sem nunca perguntar por um `lado` global.
⇒ o `cfg_de` **deixou de desarmar** um plano graduado.

| palavra | o quê | quem a lê |
|---|---|---|
| `0..4` | os cantos (sentinela no `3` de um triângulo) | canto |
| `4..8` | `id << 1 \| virada` de cada lado | aresta |
| `8` | o início do bloco de interior desta face | interior |
| `9` | quantos cantos | o despacho tri/quad |
| **`10`** | **o LADO desta face (`2^k`)** | **tudo** |
| **`11..15`** | **o início do bloco de cada aresta** | **aresta** |
| **`15..19`** | **o LADO de cada aresta (o MÁXIMO dos vizinhos)** | **aresta** |

A conta da aresta era `verts + id × (lado − 1) + (t − 1)` e passou a ser
`verts + off[s] + (t × (le / lf) − 1)`, com a **virada a contar contra `le`** e **depois** do passo.
A do interior era `verts + arestas × (lado − 1) + …` e passou a `verts + arestas_amostras + …`.

### §27.2 — ⛔⛔ Porque não foi um buffer por aresta no bind group

A leitura natural seria um `storage` novo com o par `(início, lado)` de cada aresta. Ele fica de
fora **com o número**: o grupo 1 já tem **cinco** `storage` mais um `uniform`, e o doc daquele bloco
já escreve porque não há um `@group(4)` (o `max_bind_groups` de omissão é `4`).

⇒ o par viaja **no registo da face**, duplicado nas duas faces que tocam cada aresta. Custo MEDIDO:
`19 × 4` bytes por face contra `10 × 4` — a `100 k` faces, **`7,6 MB` contra `4,0`**, ao lado de um
plano que a `8x` mede **dezenas de MB** na mesma peça. *A duplicação é barata porque a unidade com
que ela compete não é o registo, é o plano.*

### §27.3 — ⛔⛔⛔ O GATE DE PARIDADE MONTAVA O UNIFORME À MÃO, e por isso reprovou sobre a lei CERTA

Assim que a `TintaCfg` perdeu o `lado` global, o `tinta_paridade` reprovou na **primeira** fixtura,
a UNIFORME, com `7,02e-1` de divergência — e a lei estava certa. O arnês dele construía as quatro
palavras do uniforme ele próprio:

```rust
let cfg: [u32; 4] = [ t.lado_uniforme()…, verts, arestas, 1 ];   // a arrumação de ONTEM
```

⇒ ele alimentava o shader novo com a arrumação antiga. *Um arnês que CONSTRÓI o uniforme em vez de o
PEDIR mede outro programa* — e é a **mesma família** que esta crate já pagou em 20/09, quando quatro
cópias do `device()` pediam o piso do WebGPU enquanto o produto pedia o do adaptador.

⭐ A cura é a porta: `cfg_de` passou a ser **pública** (`ph2d_mesh_render::tinta_cfg`) e o arnês
chama-a. Depois disso, **`4` de `4`** verdes, as seis fixturas incluídas.

### §27.4 — ⭐⭐⭐ As fixturas GRADUADAS, e porque um plano uniforme não mede nada disto

O `tinta_paridade` corria quatro fixturas e as quatro eram uniformes. Com um nível só:

* a palavra `10` é **constante** em toda a peça;
* `le / lf` vale **`1`** em toda aresta, logo o passo do subconjunto é invisível;
* o prefixo `off[s]` volta a ser **`id × (lado − 1)`** por acidente aritmético.

⇒ *uma fixtura uniforme não distingue a lei nova da antiga.* As duas fixturas novas são graduadas
com saltos de **três degraus** entre faces vizinhas (`le / lf = 8`) — de propósito: com um degrau só
a razão é `2`, e ali uma multiplicação trocada por uma soma daria o mesmo número.

O mesmo vale para o gate **puro** que é a implementação de referência do gémeo
(`o_payload_resolve_o_mesmo_endereco_que_a_lei`): ele resolve os endereços **só** com o registo
achatado e os dois globais, e passou a correr dois planos graduados por fixtura.

### §27.5 — ⛔⛔ DUAS premissas mortas, as duas em prosa que se lia como cerca a funcionar

O cabeçalho da `ph2d-mesh-colors` dizia: *«quem ainda assume um lado só **recusa** um plano graduado
em voz alta: o assado por `assar::Recusa::Graduado`, o device por `Tinta::lado_uniforme`»*.

* O `Recusa::Graduado` **já não existia** — ele morreu na §26, quando o empacotador passou a dispor
  um ladrilho por face.
* O device **deixou de recusar** nesta wave.

⇒ as duas recusas que aquele parágrafo nomeava estão mortas, e *uma nota que nomeia uma recusa por
um endereço que já não existe lê-se como uma cerca a funcionar*. O `lado_uniforme` **fica** e o doc
dele foi reescrito: ele hoje é uma **PERGUNTA** (*«esta peça tem um lado só?»*) que só as fixturas
fazem — **nenhum consumidor de produto o chama**.

### §27.6 — O censo da fiação vai a `27` elos

As duas leis novas do gémeo não têm prova de comportamento alcançável sem placa: a única régua delas
é o `tinta_paridade`, que é `#[ignore]` **e** pede adaptador — logo nem o CI nem o arnês de mutação
(`--lib`) lhe chegam. ⇒ o censo de TEXTO ganhou o `tinta.wgsl` como fonte (`sem_prosa` corta por
`//`, que é comentário nas duas linguagens) e dois elos: o **passo do subconjunto** e a **leitura do
lado da face**.

⚠️ E a agulha do segundo leva a **assinatura da função** junto, porque `let l = tinta_topo[base +
10u];` aparece nas duas leituras (tri e quad) — *uma agulha que casa duas vezes sobrevive a uma
mutação numa delas*.

### §27.7 — ⭐ E o PRÉ-VOO fez exactamente o trabalho dele

A âncora `P11` do arnês da P2 citava `let Some(lado) = t.lado_uniforme() else {`, que esta wave
apagou. O pré-voo (`MUTA_SO_ANCORAS=1`) leu **`13` de `14`** em segundos e **sem correr um teste** —
e um arnês com uma âncora morta imprime um placar que se lê, num log, como uma corrida.

⚠️ Ele apanhou ainda uma âncora **ambígua** (a do `let l = …`, que casava duas vezes) e um erro de
aspa meu numa mensagem. *As três teriam sido lidas como «a mutação sobreviveu».*

⚠️ E o `restore` do arnês tocava `-name '*.rs'`: com mutações no `.wgsl`, o ficheiro voltava com o
mtime ANTIGO e o `include_str!` do censo ficava com o conteúdo mutado no build seguinte.

### §27.8 — ⛔⛔ E o PRÉ-VOO achou MAIS TRÊS âncoras mortas, duas delas mortas há um dia

Corrido sobre **os onze** arneses (e não sobre os que a wave tocou), ele acusou:

| arnês | âncora | quem a matou |
|---|---|---|
| `muta_o_r_por_face.sh` | `P11` (a guarda `lado_uniforme` do device) | **esta** wave |
| `muta_a_cerca_do_plano.sh` | `N8` (o CONTROLO inerte, `PAYLOAD_STRIDE = 10`) | **esta** wave |
| `muta_a_tinta_que_sai.sh` | `A4` e `A5` (o ladrilho e o tecto, no `assar.rs`) | a wave do **EMPACOTADOR** (§26) |

⛔⛔⛔ **As duas últimas estavam mortas quando a §26 fechou, e aquele §26.5 dizia por escrito
*«pré-voo dos onze arneses: `142` âncoras, todas a casar uma vez»*.** Medido contra o `HEAD` dela,
as duas agulhas **não existem** no `assar.rs` — e aquele arnês **tem** pré-voo.

⇒ *o pré-voo só vale o CONJUNTO em que foi corrido.* Um sumário que diz «os onze» depois de correr
menos que onze é o instrumento a mentir sobre si mesmo, e é exactamente a mesma forma que este
ficheiro já registou na §14 (o pré-voo a imprimir *«ZERO testes corridos»* depois de ter corrido a
suíte inteira). ⭐ A correcção ficou escrita **no §26.5**, com a morte à vista.

⚠️ E o `N8` é o mais instrutivo dos três: ele é o **CONTROLO** do arnês dele — a mutação inerte que
NÃO pode sangrar. *Um controlo cuja âncora casa zero também não sangra*, logo ele continuava a
somar `1` ao placar por não fazer nada, que é precisamente o que ele finge medir.

### §27.9 — ⚠️ E uma etiqueta REPETIDA num placar

As três mutações novas do gémeo nasceram `W5`–`W7`, e o `W5` já era o nome da mutação **nomeada**
daquele arnês. O placar imprimiu **dois `W5`**, um a sangrar e outro a sobreviver de propósito.
⛔ Nada no arnês o acusa: *o pré-voo mede a ÂNCORA e nunca o NOME*. Renumeradas para `W6`–`W8`.

### §27.10 — O placar

* **Mutação da P2 (`muta_o_r_por_face.sh`): `18 de 19` sangram** — o `P12` é o CONTROLO inerte.
  As quatro novas cobrem o uniforme (`P11`), as três palavras novas do registo (`P16`–`P18`) e as
  duas leis do gémeo pelo censo de texto (`P19`, `P20`).
* **Mutação do gémeo (`muta_o_gemeo_em_wgsl.sh`, COM A PLACA): `7 de 8` sangram** — o `W5` é a
  sobrevivente NOMEADA (o corte do piso do quad, que nenhuma régua de COR pode matar).
* **Pré-voo dos ONZE arneses: `146` âncoras, todas a casar uma vez.**
* `nextest-impacted` **`18 615` de `18 615`**.
* `tinta_paridade` na placa: **`4 de 4`**, com as duas fixturas graduadas novas.
* `clippy -D warnings` **zero** · `fmt` limpo.

### §27.11 — ⏳ O que fica ABERTO (a §26.6 menos o device)

* **Quem ESCOLHE os níveis no produto.** A lei existe (`niveis_por_area`) e precisa das ÁREAS, que a
  crate não conhece ⇒ o chamador é a família, e a pista `Paint Detail` passa a pedir uma
  **densidade** em vez de um degrau. ⚠️ **Enquanto isto não existir, nenhum gesto produz um plano
  graduado** — todo o caminho que esta wave abriu é, no produto, inalcançável.
* **A PERSISTÊNCIA de um plano graduado.** O formato guarda um `nivel` só (`SCULPT_DOC_VERSION`);
  ou passa a guardar a lista, ou a graduação é **re-derivada** no load a partir das áreas — e aí uma
  mudança na lei relayouta ficheiros gravados **em silêncio**, o que pede um degrau.
* ⚠️ **E a ORDEM entre os dois continua a ser load-bearing**, agora ao contrário: *quem escolhe*
  vem primeiro porque é ele que torna a P2 alcançável; a persistência só tem sujeito depois disso.
