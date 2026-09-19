# A geodésica — o carimbo deixa de atravessar uma parede fina

> **Estado: SHIPADO** (2026-09-19). A máscara de alcance do carimbo ganhou a lei
> da **RAZÃO** — `superfície / ar > 3,5` —, e com ela o pincel deixa de
> atravessar uma parede fina. **Zero regressão de oráculo e custo zero.**
>
> ⛔⛔ **O título desta pasta é um ARTEFACTO da wave e fica à vista de propósito:**
> ela abriu para construir uma geodésica a sério, construiu-a, e a medição disse
> que ela **não é precisa para esta lei**. O que shipa é o passeio por arestas de
> sempre com uma PERGUNTA nova. *O que faltava não era precisão — era a pergunta.*

## As figuras

| ficheiro | o que é |
|---|---|
| [`parede_fina_costas.png`](parede_fina_costas.png) | **a foto.** A barbatana vista POR TRÁS — o lado em que o artista não tocou. ANTES: uma mancha. AGORA: só a orla junto à beira. |
| [`parede_fina_corte.png`](parede_fina_corte.png) | o **mecanismo**: o corte de lado, com a face de baixo a acompanhar a de cima. |

⚠️ **Os dois painéis são PRODUTO contra PRODUTO** — o ANTES é a lei que shipava
(tecto `2,00 × R` sobre a distância), não um ideal desenhado à mão. *Uma figura
que mostra um ideal que o código não implementa é uma promessa, não uma medição.*

## O que mudou, em números

Barbatana `2,0 × 2,0` com `0,06` de espessura, pincel `R = 0,40` a força `1,00`,
carimbo a `0,30` da beira — *o gesto de quem esculpe uma orelha*.

| | antes | agora |
|---|---|---|
| **em frente do pincel**, onde a peça deslizava | `99 %` do que a frente andou | **`0 %`** |
| do carimbo inteiro, o que cai nas costas | `42,6 %` | **`16,7 %`** |
| o total que as costas andaram | `9,477` | **`2,561`** (`27 %` do que era) |
| peso do carimbo que a máscara corta | `9,86 %` | **`34,82 %`** |
| ⭐ CONTROLO — vértices da FRENTE que a cura tira | — | **`0`** |
| ⭐ **preço** | — | **zero** (é o mesmo passeio de sempre) |

E a **fronteira**: a cura alcança **todas** as distâncias da beira medidas
(`0,10` a `0,70`). Numa chapa a lei escreve-se `(2d + t)/t > 3,5` ⇒ `d > 1,25 t`
— ⭐ **ela não depende do raio do pincel; só a ESPESSURA da peça decide.**

## ⛔⛔⛔ O tecto ABSOLUTO foi construído, medido e RECUSADO

A cura óbvia era baixar o `ALCANCE_TECTO` de `2,00` para `1,50 × R`, e a
varredura até dava um planalto limpo. Ela **rebentou a paridade do `Scene
Project`**: placar de oráculo `13 → 5` fixturas dentro da barra.

⭐ **O diagnóstico é a medição, e ela aponta o dedo:** instrumentado, o corte a
`1,50` leva **só vértices da ORLA da pegada** (`ar/R` entre `0,80` e `1,00`),
onde a queda já é ~zero. Eles não movem barro nenhum — **mas alimentam o ajuste
do plano** daquele verbo, logo a saída inteira desloca-se.

⇒ *um tecto absoluto mede a mesma grandeza que o raio do pincel, logo apertá-lo
come sempre a ORLA antes de chegar ao defeito.*

## ⛔⛔⛔⛔ E a MARCHA GEODÉSICA também foi construída, medida e RECUSADA

A wave começou por portar a marcha de Kimmel–Sethian para a `ph2d-mesh`
(`Geodesica`, `atravessa`), com gates próprios contra um oráculo **exacto** (numa
chapa a geodésica **é** a distância euclidiana) e um gate de concordância ao bit
contra a cópia que a `ph2d-pose` já tinha. Ela funciona: razão `medido/exacto` de
`1,05`–`1,09` na faixa da decisão, contra `1,41` do passeio por arestas.

**E não se paga.** Medido na lei que ficou:

| | marcha | passeio por arestas |
|---|---|---|
| todas as peças APROVADAS | `0,00 %` cortado | `0,00 %` — **`0,00 pp` de diferença** |
| a BARBATANA (o defeito) | `31,9 %` | **`38,9 %`** — o passeio corta MAIS |
| custo, `523 k` vértices, `R = 0,40` | `8,04 ms` | **`1,71 ms`** |
| um dab inteiro, ali | `7,49 ms` de um tecto de `8` (**`94 %`**) | `~2,6 ms` |

⚠️ **O mecanismo é que a régua da lei é GROSSEIRA ao lado do erro do
instrumento:** `3,5` contra um viés de `1,41`. E o viés empurra para o lado
CERTO — ele infla `sup`, logo infla `sup/ar`, logo corta mais do defeito.

⭐ *O que faltava não era precisão. Era a PERGUNTA.* Eu construí a máquina antes
de medir se ela era precisa, que é exactamente o que o `CLAUDE.md` §5.0 manda
fazer ao contrário.

⚠️ **A marcha FICA, atrás da `test-support`**, porque ela é o **instrumento** da
recusa: a sonda `a_razao_precisa_da_marcha_ou_o_passeio_chega` é quem a corre, e
apagá-la levaria a medição junto. O `cfg` é o que impede um consumidor de produto
de aparecer por distracção.

## De onde sai o `RAZAO_MAXIMA = 3,5`

Do planalto medido, **com a rugosidade da peça varrida até deixar de importar** —
a borda move-se com ela, e uma constante posta na borda de UMA rugosidade mede
essa rugosidade:

| peça aprovada | factor em que lê `0,00 %` |
|---|---|
| esfera lisa · tubo · `sculpt_sphere` · cratera `0,25` e `0,50` | `≤ 2,00` |
| esfera rugosa `amp 0,08` | `2,00` |
| esfera rugosa `amp 0,12` | `2,50` |
| esfera rugosa `amp 0,16` | `3,00` |
| esfera rugosa `amp 0,20` | **`3,50`** |
| esfera rugosa `amp 0,24` | **`3,50`** ← pára de se mexer |

⛔ **E o PISO que não existe:** a 1.ª redacção trazia um piso (*«só perguntar a
razão a quem está a mais de `0,15 × R` do cursor»*). Varrida com piso `0,15` e
com piso `0`, a tabela sai **idêntica célula a célula** ⇒ ele é inerte por
geometria. *Um knob que nenhuma fixtura pode acordar é peso morto.*

## ⛔ A duplicação com a `ph2d-pose` é deliberada

A marcha existia desde 2026-09-17 na [`ph2d_pose::pesos`], para a transição do
pincel de pose. A [`ph2d_mesh::Geodesica`] **não a chama**: aquela crate declara
zero dependências porque *não sabe o que é uma `Mesh`*, e é isso que a mantém do
lado de lá da parede clean-room (o mesmo precedente da `ph2d-boundary`).

⚠️ O que torna a duplicação honesta é o **gate de concordância**
(`as_duas_marchas_concordam`): as duas `atravessa` sobre o mesmo corpus de
`20 000` triângulos mais `7` degenerados, **igualdade AO BIT**.

## ⛔ Duas outras coisas construídas, medidas e recusadas

* **Semear no PONTO** em vez do vértice mais próximo (tira um erro de meia
  aresta): não move o planalto **e** custa paridade (`13 → 12`).
* **Baixar o tecto absoluto** — acima.

## As armadilhas que esta wave pagou

1. ⛔⛔ **O tecto cortava só na SAÍDA da fila, não na ENTRADA** — quem já fora
   relaxado ficava marcado, e a marca é o que a máscara lê ⇒ **um anel inteiro
   além do tecto contava como alcançado**. Quatro gates de oráculo reprovaram.
   *A cerca de uma marcha mora onde ela ESCREVE, não onde ela pára.*
2. ⛔ **A primeira cratera era 14× rasa demais** (`0,035` contra os `0,5` do
   corpus) e lia `0,00 %`, dizendo que a cratera não era o problema enquanto o
   gate reprovava. *Uma fixtura que não contém o fenómeno responde que ele não
   existe.*
3. ⛔ **O arnês da mutação somava com `bc`**, que esta máquina não tem, e o
   `|| echo 0` fazia-o reportar «zero testes» em TODOS os casos.
4. ⛔ **O gate do tecto tinha uma banda cega** (`1,3 × tecto`) e uma mutação
   sobrevivente mostrou-o — a implicação certa é exacta: *alcançado ⇒ exacto ≤
   tecto*.
5. ⛔⛔⛔ **Eu construí a marcha ANTES de medir se ela era precisa para esta
   lei** — §5.0 manda o contrário, e a medição custou a wave inteira em tempo.
6. ⛔ **O leque da semente não tinha régua** — os gates mediam a *faixa da
   decisão*, e o leque age na primeira coroa.

## ⏳ O que fica ABERTO, com o mecanismo

**`17 %` do carimbo ainda cai nas costas**, na orla junto à beira. Aqueles pontos
estão a `1,1`–`2,0 × R` **pela superfície** — um pincel honesto de alcance `R`
cortá-los-ia — e a razão deles é baixa (`~1,4`) porque estão deslocados DE LADO,
logo o ar também é grande.

⭐ **A causa de não se poder apertar já está diagnosticada**, e é a de cima: a
máscara trima a **pegada**, e a pegada alimenta o **ajuste de plano** de
verbos como o `Scene Project`. ⇒ a cura é separar as duas perguntas — *quem se
MOVE* (a máscara) de *qual é a superfície local* (o ajuste) —, que é wave própria
e toca em como a pegada flui para a normal de área.

## Como correr

```text
bash scripts/ph2d-run.sh cargo test -p ph2d-sculpt3d --test it \
  sonda_da_parede_fina -- --ignored --nocapture --test-threads=1

bash docs/3D/geodesica/mutacao_2026-09-19.sh     # 11 de 11 sangram
```
