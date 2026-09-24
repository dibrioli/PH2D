# 04 — Pesquisa: ossos sobre desenho VETORIAL — o estado da arte, e o que ele diz da nossa lei

> **Ordem do dono (2026-09-23):** *«vamos fazer auditoria e pesquisa profunda para descobrir o estado
> da arte de como usar desenho vetorial com Bones e obter resultados perfeitos. estude grandes apps
> como after effects e outros»* — e, no mesmo dia, *«consumindo muitos créditos. vamos encerrar e
> tentar trabalhar com o que foi encontrado»*.
>
> ⚠️ **A pesquisa foi INTERROMPIDA de propósito e isso tem de estar à vista.** Correu num fan-out de
> agentes; ficaram prontas **41** respostas — o chão (5 de 6 auditorias internas), os **6** oráculos
> desta máquina, **8** apps/varreduras, **5** frentes de ciência, a triagem e **12** das 24
> verificações adversariais. ⛔ **NÃO correram:** o painel de propostas (4 ângulos × 3 júris) nem o
> crítico de completude. As propostas da §6 são **minhas**, sobre o corpus, e não foram julgadas.
>
> 📦 **O corpus bruto** (as 41 respostas, JSON, 1,2 MB) está **FORA do repo** —
> `~/Referencias/vector-bones-2026-09-23/journal.jsonl`, uma linha `{"type":"result"}` por agente.
> ⚠️ Ele fica fora porque contém leituras de alvos com parede (corridos, nunca lidos) que ninguém
> triou contra a vassoura; e é só DESTA máquina. O que importa está resumido aqui.
>
> Esta pesquisa foi feita na `line/Vector` com o HEAD em `7e9d3c0e8` (o `bind` já sem subdivisão).

## 1. Os quatro factos que decidem tudo (medidos, com a verificação ao lado)

### 1.1 ⭐⭐⭐ A recusa do BAKE foi medida contra uma base que o dono APAGOU no commit seguinte

`6cbd1fbae` recusou o [`refit_pelo_bake`](../../crates/ph2d-vec-skin/src/curva_segundo_corpo.rs) por
*«não substituir a subdivisão do bind»*; `7e9d3c0e8`, doze horas depois, **retirou a subdivisão**.
As duas linhas que decidem nunca tinham sido postas lado a lado — estão as duas no doc-comment do
próprio ficheiro (conferido por mim em 2026-09-23):

| sobre a fonte de **8** nós (o que shipa HOJE) | nós desenhados | µs/forma | ouro p90 |
|---|---:|---:|---:|
| a lei de hoje | `8` | `53` | `0,22281` |
| bake, `32` amostras/seg | `42` | `245` | `0,00426` |
| **bake, `64` amostras/seg** | `45` | **`303`** | **`0,00165`** |
| bake, `128` amostras/seg | `59` | `421` | `0,00121` (máx `0,02191` ⛔) |

⇒ **`~135×` mais fiel por `5,7×` o relógio** (`0,3 ms` por forma). ⚠️ O «não compensa» do dono
(2026-09-20) foi dito sobre um ganho de `~1,3×` (bake contra `bind + lei`); **o ganho de hoje é
outro número** — é o §0.0 à letra: *quem move o número que tornava algo inalcançável reconfere a
nota*. ⚠️ Os `45` nós são do DESENHO (a camada avaliada), nunca do documento: o documento fica com
os `8` do artista. ⛔ E amostrar mais **piora** o máximo (`128`): o alisamento do bake é o que o
protege dos bicos que a malha linear do campo deixa em cada aresta — existe uma amostragem óptima.

### 1.2 ⭐⭐⭐ Com 8 nós, 97,8 % do erro é o MODELO — nenhum procedimento o cura

`diag_b_atribuicao` (réguas, 2026-09-23): desvio ao padrão-ouro `0,73357` da espessura da barra
contra um **chão teórico** de `0,70977` — o melhor ajuste cúbico possível por segmento com as pontas
presas no ouro. ⇒ **97,8 % é MODELO, 2,2 % é procedimento** (a 150° o produto fica *abaixo* do chão
de pontas presas). ⇒ **mata com número** toda proposta de «melhor fitter», «impor G¹», «conciliar
tangentes», «afinar a correcção de alças». O que falta são **graus de liberdade**.

A física disto é conhecida: sob LBS com pesos lineares por triângulo, a imagem de uma cúbica é
**exactamente uma sêxtica** (medido a `9,2e-14`; composição de Bézier, DeRose 1988) — e a nossa
mistura em círculo nem polinomial é. O Godot, com uma lei totalmente diferente, mede `46,6×` entre 4
e 64 pontos de contorno; nós `184×` entre 8 e 54. **A penalização de poucos pontos é a física do
regime, não da nossa lei.**

### 1.3 ⭐⭐⭐ O vinco do cotovelo é uma DOBRA DO MAPA, com forma fechada — nem pesos, nem «a mistura falhar»

Verificação `V29`/`V30` (corrigindo a triagem e o próprio briefing da pesquisa): a nossa lei roda
cada ponto em torno da junta partilhada (`p' = R(θ̄)·(p − c) + Σ wᵢMᵢ(c)`), logo **preserva
`|p − J|` por construção** (`1,0000` em todo ângulo — ⚠️ isso NÃO a prova padrão-ouro, não
discrimina lei nenhuma, V25). O vinco é o determinante do mapa:

> **`det J = |1 − θ̄′·r|`** — `r` a distância ao eixo (a meia-espessura, vem da ARTE) e `θ̄′` a
> derivada do ângulo MISTURADO ao longo do repouso (vem da LEI). Bico em `θ̄′·r = 1` (≈ `93°` na
> barra da cena).

⇒ o vinco é o **produto** dos dois; nenhum sozinho o produz. ⇒ as alavancas são **baixar `θ̄′`**
(o ângulo a atravessar por unidade de comprimento) ou **autorar a forma** na faixa em que nenhum
`θ̄′` serve. ⛔ Alargar a transição de pesos está **medido como pior** (pescoço `0,2796 → 0,0092`,
F6-c do arquivo) — ⚠️ o que a forma fechada sugere é **partir o ÂNGULO**, não esbater o PESO (§3.2).

### 1.4 ⭐⭐ O ORÇAMENTO dobra de graça: o índice do campo é reconstruído POR QUADRO

[`curva.rs:397`](../../crates/ph2d-vec-skin/src/curva.rs) — `IndiceDoCampo::novo(&c.malha)` dentro de
`aplica_pela_curva_com`, chamada por quadro de
[`skin_live.rs:393`](../../crates/ph2d-skeleton-live/src/skin_live.rs), sobre uma malha que por
construção **não muda** (é o domínio do repouso). Medido: `58,8` de `126,6 µs` do recook = **`46,4 %`**
(⚠️ a `load 93–98` ⇒ o absoluto é tecto, a razão entre colunas da mesma corrida é robusta). O
`curva_segundo_corpo.rs:116,233` repete o mesmo. **Memoizar no bind é a obra mais barata do corpus.**

## 2. O que a indústria faz — e a pergunta do dono («poucos pontos, deformação perfeita»)

| alvo | como foi estudado | o que deforma | o que ensina |
|---|---|---|---|
| **Moho** | doc pública (não instalado) | **pontos de Bézier** do artista | ⭐ a **alça não é guardada**: é DERIVADA das âncoras vizinhas a cada quadro (direcção = corda `P[i+1]−P[i−1]`; comprimento = distância ao vizinho × `curvature` × `weight`; rodada por um `offset` angular) ⇒ exacta sob rotação e escala uniforme, `O(1)`, sem ponto novo. Um ponto guarda **um inteiro** (o osso), não um vector de pesos. Cotovelo: **osso auxiliar a metade do ângulo** e **Smart Bones** (correctivo autorado indexado pelo ÂNGULO). Densidade é do artista, ensinada pela app |
| **After Effects** | doc pública (não instalado) | ⛔ **não tem ossos para caminho vetorial** | o *Puppet* traceja o **alfa**, triangula e resolve ARAP; `Triangles` é um knob de densidade; *Starch* endurece regiões. ⚠️ «o AE tem ossos em shape layers» é **falso** — conflação de ToonSquid + Limber + AE numa pesquisa web |
| **Illustrator Puppet Warp** | doc + patentes | ⭐ o **único** que deforma Bézier e devolve Bézier | ⛔ **patente Adobe** — US 10 217 262 B2 (malha adaptativa guiada por BBW, viva até 2035-10-05) e US 9 865 073 B2 / US 10 078 910 B2 (BBW + ARAP em forma fechada, até ~2036) |
| **OpenToonz Plastic** | ⭐ FONTE LIDO (BSD-3) | malha do desenho | **ARAP (Igarashi 2005) sem UM peso por osso** ⇒ a LEI DO MEIO não o mata. Três passos **pré-factorizados** (a factorização cai no bind; o quadro paga só substituição). Handles livres na superfície (baricêntricas + Lagrange). **Rigidez** pintada por vértice (`1e4` contra `1`) |
| **Blender GP** | CORRIDO (GPL, oráculo) | polilinha de EIXO + raio | o documento **não ganha pontos**; um modificador de subdivisão ACIMA da armadura densifica só a camada avaliada (9 guardados → 65 avaliados). ⚠️ V36: essa densificação **interpola os pesos entre as âncoras** e congela a deformação na contagem do artista — não é grátis |
| **Synfig** | CORRIDO (GPL) | ⭐ âncora + alças (o único outro a fazê-lo) | prender o **vértice inteiro** torna o rígido exacto (nós já o fazemos); mas a mistura das alças é **cartesiana** e perde `42` pontos de área a 90° com pesos perfeitos |
| **Spine / Harmony / Animate / Live2D** | doc pública | malha / disco de junta / conjunto / retícula | ⭐ **ninguém procura a mistura perfeita**: duas zonas rígidas + uma banda estreita (Spine), disco de articulação com raio e inclinação animáveis (Harmony), ligação como CONJUNTO editado à mão (Animate), cortar a malha e re-colar (Live2D) |
| **Godot** | CORRIDO + fonte (MIT) | vértices de polígono | LBS de matriz, 4 influências, **zero pesos automáticos**; acrescentar um ponto a um polígono preso faz a arte **desaparecer** (tabela indexada por posição) |
| **Rive** | fonte (MIT) | afim misturado a âncora+alças | barato e `8,6×` longe do ouro — o **controlo**, nunca o alvo |
| **Inkscape / Krita** | CORRIDO | — | ausência: `48` Live Path Effects e zero deformador por osso |

⇒ **A resposta à pergunta do dono** (*«como o Arc/Moho fica perfeito com tão poucos pontos»*):
ninguém deforma 8 pontos e fica perfeito por uma lei melhor. Ou **acrescenta pontos numa camada que
não é o documento** (AE, Blender, o nosso bake, o próprio `Effects: Arc`), ou **deriva as alças das
âncoras** e aceita o que isso dá (Moho), ou **autora a pose difícil** (Moho Smart Bones, Live2D).

## 3. A ciência

### 3.1 ⭐⭐⭐ O artigo que É esta pergunta: Liu, Jacobson & Gingold, SIGGRAPH Asia 2014

*«Skinning Cubic Bézier Splines and Catmull-Clark Subdivision Surfaces»* (ACM TOG 33(6)). Minimiza
`E(C′) = ∫ ‖G_{C′}(p) − Σᵢ wᵢ(G_C(p))·Tᵢ·G_C(p)‖² dp` sobre os pontos de controlo — **é, letra por
letra, a nossa [`aplica_pela_curva`](../../crates/ph2d-vec-skin/src/curva.rs)** (nós aproximamos o
integral com amostras por quadro). Três diferenças que importam:

1. **Eq. (4): sob LBS o integral sai da pose** — `C′ = Σᵢ Tᵢ·Ŵᵢ·Â⁻¹`, com `Ŵᵢ` pré-calculado no BIND
   ⇒ zero amostragem por quadro. ⛔ **A nossa mistura em círculo não é linear em `Tᵢ`** e a
   factorização quebra; o `blend_linear` existe ao lado e é. Trocar é trocar o vinco (LBS colapsa:
   `0,4704` a 150°) por relógio.
2. **As âncoras também são incógnitas** (sistema 4×4); nós prendemos `P₀`/`P₃` na posição misturada.
3. **A continuidade é POR NÓ e sai do tipo autorado**: G¹ num nó liso (restrição **linear**, Eq. 5),
   ângulo fixo numa quina de 90° (Eq. 13 — que nós não temos). Medido: o alvo é G¹ nos nós lisos a
   `0,0000°`; o ajuste livre parte-o até `13,6°`.

⛔⛔ **E a limitação declarada do artigo é, à letra, a ordem do dono** (§7.1): *«When there are
insufficient control points … our optimization may produce undesirable undulations. These can be
eliminated with the insertion of additional control points.»* ⇒ o estado da arte de 2014 já disse
que, sem pontos novos, o ajuste ondula.

### 3.2 A junta: partir o ÂNGULO (inferência, NÃO medida)

A forma fechada da §1.3 diz que o bico nasce em `θ̄′·r = 1`. O truque documentado do Moho — um osso
auxiliar conduzido a **metade** do ângulo da junta — divide o ângulo que cada mistura atravessa;
⚠️ **inferido, não medido**: se `θ̄′` cair para metade, o bico sobe de `~93°` para lá de `180°`.
É a primeira hipótese a medir antes de qualquer lei nova, e não custa relógio.

### 3.3 Deformar sem pesos

- **ARAP** (Igarashi 2005; Sorkine–Alexa 2007) — o que o OpenToonz e o AE usam. Não decide por pesos
  ⇒ **a única família do corpus que a LEI DO MEIO não exclui**. ⚠️ Ninguém o correu sobre a nossa
  malha: qualidade e custo **desconhecidos** (lacuna nº 1).
- **Coordenadas de Cauchy/Green** (Weber–Ben-Chen–Gotsman 2009; Lipman 2008), com a extensão a
  gaiolas de Bézier de **Liu–Liu–Fu 2024 (arXiv:2408.06831)**. ⚠️ A patente do método de Cauchy
  (US 8 400 472 B2, Technion) está **EXPIRADA** por falta de pagamento (efectivo 2025-03-19); uma
  petição de revivescência é possível até ~2027-03. Green (2D) é equivalente e não é coberto.
- **Suavidade dos pesos muda a ORDEM de convergência**, não o desenho a nós fixos: C⁰ → `h¹`,
  C¹ → `h²`, C² → `h⁴`. ⇒ o campo C¹ ([`pesos_suave`](../../crates/ph2d-vec-skin/src/pesos_suave.rs))
  só paga *junto com* mais amostras — e a combinação **C¹ + amostras** nunca foi medida junta.

## 4. O que a verificação DERRUBOU (não citar sem a correcção)

| afirmação da triagem | veredito | a forma certa |
|---|---|---|
| «a nossa mistura JÁ É padrão-ouro: `|p−J|` = 1,0000 como o DQ do Blender» (V25) | parcial | é `1,0000` **por construção** — não discrimina lei nenhuma |
| «o vinco é do CONTORNO, não da mistura» (V29/V30) | parcial | é uma **dobra do mapa**, produto de `r` (arte) por `θ̄′` (lei) |
| «não se prende osso a um contorno gerado; o gerador destrói os pesos» (V27/V28/V31) | refutada | os pesos **sobrevivem** como atributos nomeados; o deformador é que não os lê naquela ordem — observação estreita do Blender, não lei |
| «o GP não guarda silhueta ⇒ o vinco é cosmético lá» (V32/V34/V35) | parcial | vale para TRAÇO; um **preenchimento** guarda o contorno. E não transfere: onde se aplica já temos a camada avaliada (`LiveGeometry`) |
| «o contorno auto-intersecta a partir de ~120°» (V33) | parcial | ~`112°` a raio `0,30`, e o joelho é função do **raio** (zero travessias a `r = 0,05`) |
| «densificar a camada avaliada resolve as duas recusas do dono» (V36) | parcial | a barata **congela** a deformação na contagem do artista; a que não congela é o bake — e é aí que o dono julga o **preço** |

## 5. As dívidas da própria casa que o corpus achou

1. ⛔⛔⛔ **As réguas medem a peça errada:** `46` de `53` chamadas de `b_palco(..)` na
   `ph2d-skeleton-live` pedem `b_palco(true)` (contado por mim em 2026-09-23; o agente leu `40` de `46`) — a barra de `54` nós que o `bind` deixou de produzir. A porta honesta
   (`barra_da_cena_do_produto()`) tem **um** chamador. Todo veredito verde dessa família é sobre um
   produto que não existe.
2. ⛔⛔ **Dois gates VERDES a afirmar o contrário:** `main` exige `quina < 1e-9`
   (`dobrar_a_barra_nao_crava_uma_quina_em_no_nenhum`) e a `line/Vector` exige `quina > 1e-6`
   (`o_ajuste_compra_fidelidade_e_paga_tangente`). A integração tem de escolher com a medição.
3. ⛔⛔ **O `01_a_fila.md` prescreve a `reconcilia`** (`:448`, *«a cura é um quarto passe»*) que a
   `line/Vector` mediu como sendo ELA a serpentina (`9,3×`) e removeu.
4. ⛔ **Custos publicados em desacordo:** o BBW lê `31,9 ms` nos doc-comments e `20 ms` na mensagem
   do commit que os introduziu, e nenhuma sonda produz qualquer um; o campo C¹ publica
   `0,363 ms/forma` e mede-se `122,6 µs`.
5. ⛔ **Ninguém mediu o QUADRO** com N formas presas — todo orçamento é soma de bancadas, a forma de
   erro que este módulo já pagou por `4,5×` (17/09). O perfilador existe (`PH2D_FLUID_PROFILE=1`) e
   nunca correu numa cena de esqueleto.

## 6. As rotas, com o preço (minhas — o painel que as julgaria NÃO correu)

| rota | o que compra | preço conhecido | o que falta medir |
|---|---|---|---|
| **A. Memoizar o índice do campo** (§1.4) | `~46 %` do recook, sem mudar um pixel | nenhum | o quadro inteiro, pela porta do produto |
| **B. Religar o bake no desenho** (§1.1) | `~135×` de fidelidade com os `8` pontos do artista intactos | `~0,3 ms`/forma (`5,7×`); com A por baixo, menos | o quadro inteiro; a amostragem óptima (`32`–`64`) na cena do dono |
| **C. Partir o ângulo da junta** (§3.2) | empurra o bico do cotovelo | ~zero de relógio | tudo — é inferência |
| **D. ARAP portado do OpenToonz** (§3.3) | a única família que a LEI DO MEIO não mata | factorização no bind, substituição por quadro | qualidade e relógio sobre a nossa malha (lacuna nº 1) |
| **E. Autoria na pose difícil** (Smart Bones, já fechado em F3) | perfeição onde nenhuma lei chega | trabalho do artista | nada — é produto |

⛔ **Recusadas por medição e que uma pesquisa externa vai propor primeiro** (ver
[`01_a_fila.md`](01_a_fila.md) «Recusas MEDIDAS»): pesos por difusão de calor · centros de rotação
optimizados (Le & Hodgins 2016) · dual quaternion / log de `se(2)` · pesos harmónicos · portar os
pesos do Godot · o refit adaptativo que chama a lei de dentro do fitter (`~45×` o orçamento).

## 7. Lacunas (o que esta pesquisa NÃO apurou)

- Nenhum alvo proprietário foi corrido (Moho, AE, Harmony, Spine, Live2D, Animate) — tudo deles é
  documentação pública.
- O OpenToonz tem porta de consola (`tcomposer` com `QT_QPA_PLATFORM=offscreen`) e **não foi usada**:
  não há um número do Plastic sobre arte nossa.
- O Rive não está instalado e nunca foi corrido.
- O painel de propostas e o crítico de completude não correram (encerrado por ordem do dono).
