# 12 — O material entre gestos, e o relógio que não era o arrasto

> Report do dono, 2026-09-09: *«quando uso inflate e faço mais de uma simulação o objeto
> **desinfla a cada início de simulação**!»* — mais o pedido que abriu a outra metade desta
> jornada: *«porque vc não faz uma série de testes abrindo o blender como vc fez para
> implementar nosso belo Cloth tool e tenta descobrir a melhor implementação do Filter Mesh
> Cloth?»*

---

## §1 — O report, reproduzido antes de tocar em código

Três gestos de *Inflate* seguidos sobre uma esfera UV `32×64`, volume normalizado ao repouso, com
os *defaults* do produto:

| gesto | k0 | k1 | k2 | k3 | k4 | k5 | … | fim |
|---|---:|---:|---:|---:|---:|---:|---|---:|
| 1 | `1,000` | `1,001` | `1,002` | `1,004` | `1,006` | `1,008` | | `1,168` |
| 2 | `1,103` | `1,030` | `0,964` | `0,916` | `0,890` | **`0,887`** | | `1,166` |
| 3 | `1,101` | `1,028` | `0,963` | `0,915` | `0,890` | **`0,886`** | | `1,166` |

⇒ **duas coisas, e a segunda é a que ele vê:** (a) a inflação **não acumula** — os três gestos
acabam no mesmo sítio; (b) o início de cada gesto novo **afunda a peça abaixo do repouso** antes de
a voltar a levantar.

### A causa é NOSSA, e é a cura do report anterior

A wave de 08/09 fez o material atravessar os gestos — a cura de *«se eu fizer mais de uma simulação
o objeto continua esticando»*. Ela é **incondicional**: no início de cada gesto o comprimento de
repouso de cada aresta volta a sair do material original. Num tipo que INFLA isso é exactamente
«desfazer o gesto anterior antes de começar o seguinte».

---

## §2 — A lei: um tipo ou CARREGA a peça, ou muda o TAMANHO dela

⭐ **A pergunta é sobre a intenção do tipo**, e ela parte os cinco em dois
([`ClothFilterKind::muda_o_material`](../../../crates/ph2d-sculpt3d/src/cloth_filter_kind.rs)):

| tipo | o que faz à peça | material |
|---|---|---|
| **Gravity · Pinch** | uma CARGA externa — o pano é o mesmo, o que muda é o que puxa por ele | **persiste** |
| **Inflate · Expand · Scale** | mudam o TAMANHO da peça — a forma nova é o que o artista quer guardar | **segue a malha** |

Depois da cura:

| três gestos | fim de cada | **fundo** de cada |
|---|---|---|
| Inflate | `1,166 · 1,353 · 1,563` | `1,000 · 1,166 · 1,353` |
| Scale | `1,216 · 1,467 · 1,764` | `1,000 · 1,216 · 1,467` |

⭐ **O fundo de cada gesto é o fim do anterior** — a peça nunca desce.

⚠️ **O censo da família achou um SEGUNDO membro que o report não nomeia:** a **Scale** tinha o mesmo
defeito (`fundo 1,001 · 0,890 · 0,871`) pelo mesmo mecanismo — a âncora dela sai da pose de agora e
o tecto mede-a contra o material velho. *O exemplo que o dono aponta pode ser a excepção da família;
o censo corre antes do veredito.*

### ⭐⭐⭐ E o ORÁCULO foi corrido para decidir isto — o alvo tem o MESMO defeito

*«porque vc não faz uma série de testes abrindo o blender … e tenta descobrir a melhor
implementação do Filter Mesh Cloth?»* — dez corridas novas, [espec §10.18](../cleanroom/SPEC_cloth_brush.md).

| pergunta | resposta medida |
|---|---|
| o **filtro** lê a base persistente? | ⭐ **sim** — e o interruptor é a opção do **pincel activo**; o painel do filtro **não a mostra**. O controlo decisivo é a opção ligada **sem** base gravada: byte a byte igual a desligada em `34` blocos ⇒ *quem muda a lei é a base, não o interruptor* |
| **sem** base, três invocações compõem? | **sim, e nunca recuam** — *Gravity* `0,1087 → 0,2165 → 0,3233`; *Inflate* na esfera cresce em volume nos **24** passos, nas quatro realizações; *Expand* `+98,9 %`/`+194,9 %`; *Scale* `+116,1 %`/`+250,8 %` (o único superlinear) |
| **com** a base gravada? | ⛔⛔ **o alvo TEM o afundamento do report**: a 3.ª invocação acaba **abaixo** da 2.ª (`0,1806` contra `0,2125`), com mínimo a `−19,8 %` (gravidade) e `−20,3 %` (inflar) dentro dela |
| há tecto de esticão? | **não** — `36` passos levam o volume a **`4,735×`** e a maior aresta a **`21,55×`** |
| o painel do filtro tem volume, dobra ou limite? | **não** — são `8` controlos e nenhum deles; e o *Repeat* que a família de filtros regista é **morto** aqui (`5` contra `1`: byte a byte igual) |

⇒ ⭐⭐ **o lado limpo tinha tomado como LEI o que no alvo é um interruptor escondido — e o
interruptor produz exactamente o defeito que o dono reportou.** A nossa resposta é melhor que as
duas do alvo: em vez de escolher entre *compor sempre* e *recuar sempre*, cada tipo faz o que a
intenção dele pede, sem knob nenhum.

⚠️ **Armadilha de régua que a corrida longa devolveu:** o máximo de deslocamento tem **pico no passo
33** e desce `1,01 %` até ao 36 **enquanto o volume sobe**. *A grandeza que responde a «a peça
cresceu?» é integral (volume, área); o máximo de um deslocamento responde a outra pergunta.*

### Os gates, e a metade que NÃO vive no mesmo ficheiro

- `um_gesto_que_muda_o_tamanho_nunca_desinfla_o_anterior` — mutação `muda_o_material ≡ false`:
  **reprova**.
- a outra metade é o `o_pano_nao_cresce_a_cada_gesto` do ficheiro irmão — mutação
  `muda_o_material ≡ true`: **reprova**.

⛔⛔ **A 1.ª redacção tinha um gate próprio para a segunda metade e ele era VÁCUO:** sem máscara, a
gravidade sobre uma peça solta é uma **translação rígida**, e uma translação rígida preserva o
volume tenha o material sido re-semeado ou não (`1,0000` nos três gestos, com e sem a mutação).
*Um gate que mede a grandeza que o gesto não muda passa sempre.*

---

## §3 — E o relógio da simulação era a TAXA DE QUADROS

O censo devolveu, de lado, um defeito de outra espécie e maior que o report.

O `s` que a lei recebe é a **distância acumulada ao ponto de pressão**, e o shell junta os
movimentos do rato **por quadro** (`tecido.pending`) ⇒ o relógio da simulação era a taxa de quadros.
O MESMO arrasto (`s` de `0` a `1`) sobre uma esfera, entregue em `n` eventos:

| amostras | 8 | 15 | 30 | 60 | 120 | 240 |
|---|---:|---:|---:|---:|---:|---:|
| **Gravity** — raio máximo | `1,15` | `1,45` | `2,65` | `7,30` | `25,60` | **`98,20`** |
| **Expand** — volume | `1,97` | `2,93` | `3,50` | `5,49` | `19,51` | **`55,84`** |
| Inflate — volume | `1,19` | `1,17` | `1,17` | `1,17` | `1,17` | `1,17` |
| Scale — volume | `1,29` | `1,22` | `1,22` | `1,22` | `1,22` | `1,22` |

⭐⭐ **`85×` na gravidade, e a peça nem se deforma** — volume `1,000` e esticão `1,00` nas seis
colunas: ela **voa**, como uma translação rígida. ⚠️ *Era esta a metade de «um elástico que estica
indefinidamente» que nenhum tecto podia curar — o tecto mede DEFORMAÇÃO, e isto não é deformação.*

⚠️ **Inflate e Scale são invariantes** porque escrevem uma força ou uma âncora e o equilíbrio contra
a rede é fixado pela MAGNITUDE de `S`, que chega a `1` seja qual for o número de passos. Os dois que
se movem são o que **acumula** (o `τ` do Expand) e o que **integra sem parar** (a velocidade da
gravidade numa peça sem âncora).

### A cura, e os DOIS números que não são o mesmo

Um passo da lei passa a valer um **passo de arrasto**, e o adaptador corre tantos quantos o arrasto
pedir — **nenhum se o dedo não andou o suficiente**. *É a lei do espaçamento dos dabs, que o traço
já tem: parametrizar pelo CAMINHO, nunca pela amostragem.*

| constante | de onde vem | o que fixa |
|---|---|---|
| `QUANTUM_DE_ARRASTO` = `0,09` | do **cabeçalho das fixtures do alvo** (`avanco_por_passo_px 90`) | a unidade em que a lei do `τ` do *Expand* foi calibrada |
| `PASSO_DE_ARRASTO` = `0,01` (`10 px`) | **calibração de PRODUTO** | quanto o dedo tem de andar para a simulação avançar um passo |

⚠️⚠️ **Com o quantum da LEI o filtro ficava `9×` mais fraco** — um arrasto de ecrã inteiro dava `11`
passos contra os `~120` de antes — e **três gates reprovaram na PRECONDIÇÃO** (*«a fixtura tem de
produzir o defeito»*), que é o instrumento a dizer que a peça já não chega onde chegava. *Um número
do lado aprovado responde à pergunta dele, não à nossa.* O `0,01` sai de **preservar o que o dono
aprovou**: até 09/09 o ritmo de facto era `1` passo por quadro, e um arrasto de trabalho anda
`~10 px` por quadro.

### O tecto de passos por chamada é DERIVADO, e o recurso é o quadro

| vértices | por passo | passos num quadro de `16,7 ms` |
|---:|---:|---:|
| `1 986` | `1,146 ms` | `14,6` |
| `8 066` | `5,178 ms` | `3,2` |
| `18 242` | `11,229 ms` | `1,5` |

⇒ **`0,615 µs` por vértice**, e o tecto é o orçamento de dois quadros a dividir por ele:
**`54 000 / n`**, limitado a `[1, 32]`. ⛔ Um tecto CONSTANTE seria de um recurso que não existe —
`16` passos numa peça de `18 242` vértices são `180 ms` num quadro. O arrasto que não coube **fica
por consumir** e o quadro seguinte continua de onde este parou.

---

## §4 — O `τ` do Expand contava EVENTOS

Mesmo mecanismo, um nível abaixo: o *Expand* soma `τ += 0,01 · f` **por passo**, e no filtro um
passo era um movimento do rato. ⇒ o incremento passa a ser pesado por `|Δs| / QUANTUM_DE_ARRASTO`.

⭐ **As duas fixtures do Expand ficam BYTE-IDÊNTICAS por construção** (nelas o avanço é uniforme —
`0,09` por passo, `−0,09` na negativa — logo o peso vale exactamente `1` nos oito passos), e as
`86 + 17` corridas do oráculo correm verdes.
⚠️ **O valor absoluto é load-bearing:** sem ele a fixture negativa inverteria o sinal de `τ` e o
*Expand* para trás faria a peça **crescer**.

Gate `o_expand_mede_o_arrasto_e_nao_conta_eventos`, com a barra **derivada**: a soma é uma soma de
Riemann à direita de `∫₀¹ s ds`, vale `(n+1)/2n` contra `1/2`, logo o desvio é **`1/n`**. ⚠️ A 1.ª
redacção escreveu `1/(2n)` e **reprovou sobre produto correcto** — *quem escreve uma barra derivada
tem de derivar a conta certa.*

---

## §5 — Aberto

- ⏳ **As dez corridas novas ainda não são todas gates** — o corpus do filtro passou de `17` para
  `27` e o arnês (`oraculo_do_filtro.rs`) ganhou as invocações repetidas e a base persistente; o que
  cada traço novo mede e com que barra é trabalho da wave seguinte.
- **O `Expand` continua a ser o mais violento dos cinco** mesmo com o `τ` parametrizado pelo
  arrasto: um arrasto de ecrã inteiro leva o volume a `2,4×`. Ele é o único tipo que mexe no
  repouso, logo o **tecto de esticão não o mede** — o tecto compara contra um comprimento que a
  própria lei cresce.
- **A resposta do filtro é quadrática no arrasto** (`Σ_j (k−j+1)·S_j·Δt`, espec §7): dobrar o
  arrasto quadruplica o efeito. É a lei do alvo, e ela torna o fim da faixa difícil de dosear.
- **O tamanho da ruga continua a seguir a densidade da malha** (doc [11](11_o_material_a_ruga_e_a_memoria.md) §5).
