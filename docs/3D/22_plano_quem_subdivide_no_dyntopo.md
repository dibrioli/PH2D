---
titulo: "Quem subdivide no Dynamic Topology — o plano do estudo e da cura"
tags: [modulo/3d, tipo/plano, status/aberto]
status: aberto
modulo: 3D
atualizado: 2026-09-14
resumo: "A METADE DA MÁSCARA ESTÁ CURADA (14/09): a porta passou a perguntar ao VERBO e a máscara deixou de mudar a topologia — medido, ela levava a peça de 830 para 1 331 vértices. O resto da tabela continua aberto. O refino do dyntopo estava preso ao GESTO e não ao VERBO: 19 verbos refinam (o Smooth e a MÁSCARA incluídos) e 8 nunca refinam (Grab, Snake Hook, Cloth, Pose…). O que cada verbo DEVE fazer é uma pergunta de oráculo, com meia resposta livre (SculptGL é MIT) e meia atrás da parede (Blender é GPL ⇒ corre-se, não se lê)."
---

# 22 — Quem subdivide no Dynamic Topology

> **Report do dono, 2026-09-14:** *«algumas tools que não deveriam fazer a
> subdivisão de polígonos no modo Dynamic Topology estão fazendo (como smooth)
> enquanto algumas que deveriam criar subdivisões com Dynamic Topology não estão
> criando. é preciso um estudo no blender ou no SculptGL para descobrir que deve
> ou não subdividir faces no Dynamic Topology.»*

⚠️ **Este doc é o PLANO e o CENSO.** O que ele afirma sobre **nós** está medido;
o que ele afirma sobre os alvos é a **pergunta** e o **instrumento** que a
responde — nunca uma resposta que eu não medi.

> ## ✅ ACTUALIZAÇÃO 2026-09-14 — a metade da MÁSCARA está CURADA
>
> A porta (`refine_for_dab`) passou a **receber o verbo** e a ler **duas
> colunas** ([`Verb::refina_no_dyntopo`] · [`Verb::colapsa_no_dyntopo`]), e a
> máscara responde `false` às duas. **Medido na cena, com o detalhe no extremo
> fino:** um dab de máscara levava a peça de **`830` para `1 331` vértices** —
> `+60 %` de topologia num gesto que **não move um único vértice**.
>
> ⛔ **E MAIS NADA MUDOU, com gate a afirmá-lo**
> (`o_mask_e_a_unica_correccao_que_esta_tabela_faz`): todos os outros verbos leem
> exactamente o que já liam. *O resto da tabela continua a ser a pergunta de
> oráculo que o §4 descreve*, e o §5 já não é um esboço — a porta existe, e o
> estudo preenche células em vez de re-fiar.
>
> ⚠️ **O `false` dos 8 verbos com âncora é o valor CONSERVADOR, não uma
> resposta:** hoje eles nem chegam à porta, e assim ligá-la aos gestos ancorados
> antes do estudo não muda nada em silêncio. Há gate
> (`nenhum_verbo_com_ancora_refina_ou_colapsa_hoje`).
>
> ⚠️ **As duas colunas coincidem hoje em TODOS os verbos**, e isso é um facto
> sobre o produto de hoje — não uma lei. O gate
> `as_duas_colunas_ainda_coincidem_e_isso_nao_e_uma_lei` **reprova** no dia em
> que o estudo as separar, que é quando se quer que ele fale.

---

## §1 — ⭐⭐ A causa é UMA LINHA, e ela não menciona verbo nenhum

Medido nesta árvore em 2026-09-14: o refino do dyntopo tem **exactamente um
chamador de produto**.

```
crates/ph2d-app-sculpt3d/src/input.rs:121    self.refine_for_dab(hit.point, brush.radius);
```

Esse chamador é o `sculpt_at` — o braço do **CARIMBO**. ⇒ a pergunta que o
produto responde hoje não é *«este verbo cria superfície nova?»* e sim **«este
gesto passou pelo caminho do carimbo?»**.

⚠️⚠️ **É a mesma família de defeito que este módulo já pagou três vezes ao
contrário** (`fill_hc_disp`, a recusa do `begin_filter`, o canal do undo do
filtro): *inferir uma propriedade do verbo a partir do caminho que o gesto
tomou*. Aqui o preço é simétrico e o dono vê os dois lados.

---

## §2 — O CENSO de hoje: `19` refinam, `8` nunca refinam

O discriminador é [`Verb::anchors`] (`Grip::Hold | Hook | Turn | Simulate`):
quem tem âncora entra pelo `take_hold`/`grab_at`/`hook_step`/`cloth_step`, e
**nenhum deles** chama o refino.

| | verbos | o que acontece hoje |
|---|---|---|
| **Refinam** (`19`) | Draw · Inflate · **Smooth** · Sharpen · Flatten · Fill · Scrape · Clay · Pinch · Magnify · Crease · Blob · **Mask** · Clay Strips · Clay Thumb · Multiplane Scrape · Slide Relax · Surface Smooth · Layer | cada dab refina **e colapsa** na esfera do pincel |
| **Nunca refinam** (`8`) | Move/Grab · Snake Hook · Twist · Local Scale · Cloth · Thumb · Nudge · **Pose** | a malha estica e a densidade fica a do repouso |

⛔ **Dois casos são indefensáveis em qualquer leitura, e não precisam do estudo
para serem nomeados:**

1. **`Mask`** — ele pinta um canal por-vértice e **não move um único vértice**.
   Refinar debaixo dele muda a topologia da peça num gesto que não toca na
   geometria, e o custo é pago por um artista que só queria proteger uma zona.
2. **`Pose` / `Cloth` / `Move`** — são os verbos que mais **esticam superfície**
   (a pose roda um membro inteiro; a espec §13 diz que *nenhuma parte da malha é
   excluída pelo raio*), logo são os que mais produzem aresta longa — e são
   exactamente os que **nunca** ganham um vértice novo.

⚠️ **O resto da tabela é que precisa do estudo.** *Não escrevo aqui um palpite
sobre o `Smooth`*: ele move vértices (tangencialmente), e um refino sob ele tem
um argumento legítimo (manter densidade ao relaxar) e um ilegítimo (crescer a
malha sem detalhe novo). ⛔ **Qual dos dois é o comportamento que o artista
conhece é uma pergunta de ORÁCULO, não de raciocínio.**

---

## §3 — ⚠️ E há uma SEGUNDA metade que o report não separa: o COLAPSO

O `refine_for_dab` faz **duas** coisas — `collapse_in_sphere` e depois
`refine_in_sphere` —, e elas são leis independentes com consumidores diferentes.
Um verbo pode legitimamente querer **colapsar e não refinar** (relaxar densidade
sem criar detalhe) ou o contrário.

⇒ o estudo tem de responder **duas** colunas por verbo, nunca uma. *Uma tabela
com uma coluna só obriga quem a lê a escolher por ele — e a escolha desaparece.*

---

## §4 — Quem pode responder o quê (a triagem de licença vem primeiro, §0.9)

| perna | licença | o que ela pode dar | quem a corre |
|---|---|---|---|
| **SculptGL** | **MIT** | ⭐ **tudo, e sem parede**: o código lê-se, porta-se e **já corre nesta casa como oráculo** (o Node roda-o sobre as nossas malhas). A resposta para os verbos do `s-mode` sai daqui, com atribuição | **qualquer janela**, incluindo a que implementa |
| **Blender** | **GPL** | só **comportamento observado**: correr sem interface sobre uma malha **nossa**, com dyntopo armado, um traço por verbo, e gravar a **contagem de vértices antes e depois** | ⛔ **uma janela E**, nunca a que implementa (SKILL clean-room) |

⭐⭐ **A saída é livre e a pergunta é binária**, que é a melhor forma que uma
pergunta de oráculo pode ter: *com dyntopo armado e o mesmo traço, a contagem de
vértices muda?* — e a contagem de vértices de uma malha **nossa** não é obra
derivada (GPLv2 §0). Não é preciso ler uma linha do alvo para responder.

⚠️ **O corpus tem de trazer o par**: um traço que **estica** (para ver o refino)
e um que **relaxa** (para ver o colapso), porque um verbo pode disparar um e não
o outro. E tem de correr com o detalhe em **dois** pontos do slider — um refino
que só aparece no extremo fino lê-se como *«não refina»* num corpus grosso.

---

## §5 — A cura, quando a tabela existir

⛔ **A cura NÃO é mover a chamada de sítio.** Ela é dar ao verbo a pergunta que
hoje ele não tem:

```rust
// esboço — a forma, não o valor
impl Verb {
    pub fn refina_no_dyntopo(self) -> bool { … }
    pub fn colapsa_no_dyntopo(self) -> bool { … }
}
```

…lidas **no `refine_for_dab`**, que é a porta única, e chamadas também pelos
gestos com âncora (`grab_at`, `hook_step`, `cloth_step`) para os `8` que hoje
nunca a alcançam.

⚠️ **Três armadilhas que a implementação vai encontrar, nomeadas agora:**

1. ⛔ **Um verbo com âncora que passe a refinar muda a malha DEBAIXO do gesto em
   voo**, e o traço tem estado por-índice (`slot`, `stamp`, `touched`, e no caso
   da pose a cadeia **e** o `p0` da malha inteira). O `grow_with`/`shrink_with`
   já existe para o carimbo — ⚠️ **mas a `PoseSessao` guarda um `p0` do tamanho
   da malha e uma cadeia com pesos por vértice, e nenhum dos dois sabe crescer.**
   *Refinar sob a pose sem responder a isso indexa a cadeia velha com índices
   novos: mudo enquanto a malha só cresce, pânico no dia do colapso.*
2. ⚠️ **O `Verb::Cloth` e o `Verb::Pose` têm corpus de paridade** (86 e 69
   traços). Refinar sob eles muda a contagem de vértices ⇒ **as fixturas deixam
   de ser comparáveis**. A cura tem de nascer atrás do ARM do dyntopo (que as
   bancadas nunca ligam), e há de haver gate a afirmá-lo.
3. ✅ **O `Mask` era o caso mais barato e o mais visível, e está CURADO**
   (14/09) — a resposta não dependia de alvo nenhum: um gesto que não escreve
   posição não tem porque mudar a topologia. ⚠️ **A régua precisou das duas
   metades:** sem o controlo positivo (um `Draw` no mesmo arranjo, que tem de
   refinar) um `assert_eq!` de contagem ficaria verde sobre um dyntopo **inerte**
   — o detalhe grosso, a esfera já fina, o raio errado. *Uma régua que não vê o
   fenómeno acontecer não prova que ele não aconteceu.* E há uma segunda
   asserção a impedir a cura barata: **a máscara continua a mascarar** (*curar um
   defeito desligando o verbo é a forma mais barata de o esconder*).

---

## §6 — O que fica MEDIDO por este doc

| afirmação | como foi obtida |
|---|---|
| `refine_for_dab` tem **um** chamador de produto (`input.rs:121`, no `sculpt_at`) | varredura da árvore |
| `19` verbos refinam e `8` não | o `match` de [`Verb::grip`] cruzado com `anchors()` e com o único chamador |
| o refino e o colapso são **duas** leis no mesmo sítio | corpo do `refine_for_dab` |
| o `Mask` refina | ele é `Grip::Paint`, que **não** é âncora ⇒ entra pelo carimbo |

⛔ **O que este doc NÃO afirma:** o que o Blender ou o SculptGL fazem. Nenhuma
linha acima foi lida de um alvo.
