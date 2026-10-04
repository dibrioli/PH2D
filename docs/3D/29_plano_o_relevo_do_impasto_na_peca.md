# 29 — Plano: o relevo do IMPASTO na peça (etapa 3b do Painter na peça)

> **Ordem:** o dono, 30/09 (*«siga com o que está em aberto»*), sobre a decisão
> de produto dele de 24/09: **relevo de LUZ** — a tinta ganha espessura que pega
> luz e sombra, **a forma da peça não muda**, desfaz-se com `Ctrl+Z`, **não entra
> na silhueta nem no ficheiro exportado**, e *«a luz usada é a luz do cenário 3d»*.
> Contexto: [25](25_avaliacao_o_painter_na_malha.md) · etapa 2 no
> [handoff](handoffs/HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_ETAPA_2_2026-09-24.md).

## 1. O que existe hoje (medido pelas três leituras de 30/09)

- **No Painter** a altura do impasto é um `f32` por píxel e POR CAMADA
  (`PainterTool::heights`, `+` levanta, `−` cava; `h = 1` é um traço de
  profundidade cheia, e a luz converte-a a píxeis com `DEPTH_UNIT_PX = 16`). O
  traço escreve-a num plano próprio (`relief.stroke_height`) e o `close_stroke`
  soma-a na camada. ⚠️ **Na tela da vista ela é apagada a cada traço** — o
  `set_source`, por onde passam o `clear_screen_canvas` e o `seed_screen_canvas`,
  limpa `heights`/`covers`/`mats`.
- ⛔ **A drenagem sai ACESA**: o `take_screen_canvas` devolve o composto já com
  o `apply_impasto_light` (a luz 2D do Painter) por cima, e não há acessor da cor
  SEM luz. Pousar isso na peça pintaria a sombra 2D como COR, e a luz 3D por
  cima dela contaria a luz duas vezes.
- **Na peça** não há canal de relevo: o shader não perturba a normal em sítio
  nenhum (nem `dpdx`/`dpdy`). A normal de sombreado é `canvas_normal(n_view)`, lida
  UMA vez no `fs_core` e usada pelos modos `Rig`, `Pbr` e `Matcap`; o `Flat` não a
  usa (e não deve: ele é a cor sem luz).

## 2. As decisões

### D1 — A altura mora no PLANO DE TINTA FINA, ao lado da cor

`Tinta` ganha `alturas: Option<Vec<f32>>`, uma por amostra, endereçada e
interpolada pelos MESMOS pesos da cor (`leitura_tri`/`leitura_quad` + `indice`).
⭐ Com isso o empréstimo ao traço, a ranhura por degrau, a cerca de identidade do
plano e a exportação que o ignora vêm **de graça** — tudo o que já transporta o
plano transporta a altura.

- ⚠️ **`Option` de propósito:** a `256x` um plano tem dezenas de MB, e uma peça
  que nunca levou impasto não paga um byte.
- ⛔ **Sem tinta fina armada (`Paint Detail = Mesh`) não há relevo**, e isso é
  DITO no pen-down (a mesma família de vozes da tinta fina). A alternativa — uma
  altura POR VÉRTICE — daria a derivada constante por triângulo (relevo
  facetado à resolução da malha) e custaria um 10.º buffer de vértice, o
  `MeshData` partilhado e o formato do `ph2d-mesh`. *O relevo de uma pincelada é
  mais fino que a malha, que é exactamente o que a tinta fina existe para ser.*

### D2 — A unidade é o MUNDO: a espessura que a tinta tinha no ECRÃ

A altura na peça é medida em unidades de objecto, ao longo da normal:
`altura = h · DEPTH_UNIT_PX · (mundo por píxel no ponto, na vista do traço)`.
⇒ o relevo é **geométrico**: rodar a vista e aproximar não o mudam, e um traço dado
de perto fica tão espesso, em mundo, quanto a tinta parecia no ecrã. O factor sai
da `Vista` da sessão (a mesma que projecta a amostra).

### D3 — A lei na peça: a altura SOMA-SE à de antes do traço

A tela da vista começa cada traço com altura zero (o `set_source` limpa-a), logo o
que ela entrega é o relevo DESTE traço: `nova = antes + altura(p)`. Pintar por cima
empilha tinta; cavar (`−`) desce. A janela de desfazer guarda a altura de antes,
num canal ao lado da cor.

### D4 — A cor que se pousa é a SEM luz

Porta nova no Painter: a drenagem da tela da vista entrega a cor **sem** o
`apply_impasto_light` e, ao lado, a altura da região mudada (camada activa, com o
traço vivo). A luz passa a ser só a da cena 3D.

### D5 — O sombreado: *bump mapping* sem parametrização (Mikkelsen, 2010)

Referência publicada e portada: M. Mikkelsen, *«Bump Mapping Unparametrized
Surfaces on the GPU»* (2010) — a normal perturbada sai das derivadas de ecrã da
posição e da altura, **sem tangentes e sem UV**, que é o que uma malha esculpida
não tem:

```
σs = ∂p/∂x   σt = ∂p/∂y        (posição em vista, derivadas de ecrã)
R1 = σt × n  R2 = n × σs       det = σs · R1
∇  = sign(det) · (∂H/∂x · R1 + ∂H/∂y · R2)
n' = normalize(|det| · n − ∇)
```

Ela substitui o `nc` no `fs_core` (para `Rig`, `Pbr`, `Matcap`), e as derivadas
chamam-se **fora** de ramos divergentes (o `tinta_no_ponto` ramifica por
`topo`). Com a altura toda a zero, `∇ = 0` e `n' = n` — ⚠️ mas **não ao bit** (o
`normalize(|det|·n)` arredonda), logo o caminho SEM relevo armado **não passa por
ela**: ele é byte-idêntico ao de hoje por construção, com gate.

- ⛔ **Fora:** o G-buffer (a normal doada ao 2D e ao *bake*) fica SEM relevo nesta
  onda — a doação é GEOMETRIA, e *«não entra na silhueta»*; se o dono quiser o
  relevo no sprite assado, é pergunta dele e onda própria.

## 3. As ondas

| Onda | O quê | Gate que a fecha |
|---|---|---|
| **W1** | `Tinta::alturas` + leitura `altura_tri/quad` + `footprint` + a janela de desfazer (`TintaDoTraco` com a altura de antes, `JanelaFina` com o canal) + o documento `v4` (corridas de `f32` por bits, com migração do `v3`) | ida-e-volta do documento ao bit; `Ctrl+Z` devolve a altura; uma peça sem relevo grava o MESMO número de bytes de hoje |
| **W2** | o device: `@binding(7)` com as alturas, subida inteira e incremental, o `armado` a dizer se há relevo, e o *bump* no `fs_core` | paridade da leitura (compute) contra `altura_tri/quad`; sem relevo o quadro é byte-idêntico; com um degrau de altura numa chapa a luz muda **só** na borda dele e no sentido que a luz pede |
| **W3** | o Painter: a drenagem sem luz + a altura da região; a pousada escreve `antes + h·16·mundo_por_px` | censo de fiação; um traço de impasto na peça sobe a altura das amostras que cobriu, no valor da conta D2; a cor pousada é a sem luz |
| **W4** | a cena e o roteiro (`=52` ganha o passo, ou cena própria), a voz do pen-down sem tinta fina, e o smoke | a foto: a pincelada lê-se em relevo com a luz da cena, e rodar a luz move as sombras dela |

## 4. Critério de desistência (antes de construir)

- Se a derivada de ecrã da altura, com o plano a `8x`, der **facetas visíveis por
  sub-célula** que nenhuma interpolação dentro do `tinta_no_ponto` cure, a forma
  D5 pára e a pergunta passa a ser a normal lida por diferenças CENTRAIS no
  plano (o que o Painter faz em 2D) — com o custo medido antes.
- Se a subida incremental das alturas custar mais de `1 ms` por quadro durante um
  traço na peça de fábrica, a W2 não fecha nessa forma.

## 5. O estado (30/09)

| onda | commit | o que ficou | gates · mutação |
|---|---|---|---|
| **W1** | `sculpt3d(relevo W1)` | como planeado | 5 + 3 + 4 · `muta_o_relevo.sh` 8/8 |
| **W2** | `c5fbff7ec` | o `binding 7`, a leitura `vec4` (cor e altura pelos MESMOS pesos), o *bump* de Mikkelsen num `fs_core_n` que recebe a normal — o `fs_core` delega nele com `in.n_view`, logo sem relevo o ramo nem é tomado | a paridade lê `.w` · `o_relevo_inclina_a_luz_so_onde_ha_degrau` (pixel, com adaptador) · 3/3 + controlo |
| **W3** | `82a057060` | a luz 2D desligada NA TELA DA VISTA (`impasto_visible`, uma porta para os dois caminhos do Painter) · a espessura numa JANELA à volta do rectângulo mudado, no mesmo quadro que a cor · `Vista::mundo_por_pixel` PARALELO À IMAGEM · `nova = antes + h_px·k·píxel` | 4 + 5 · `muta_o_relevo_na_peca.sh` 9/9 + controlo |
| **W4** | este | a VOZ do pen-down: um traço que molda relevo (`stroke_shapes_relief`) sobre uma peça sem tinta fina diz que só a cor fica | as 4 células + o elo no pen-down |

⚠️ **Duas correcções que a construção impôs ao plano:**

- **O píxel mede-se paralelo à IMAGEM, não perpendicular ao RAIO** (D2). A 1.ª
  redacção seguia o raio e o gate leu `0,5 %` de erro no canto da tela — fora do
  eixo as duas direcções diferem, e a espessura do Painter é medida no plano do
  ecrã. A linha `w` da matriz é o «em frente»; sem perspectiva, a linha `z`.
- **A espessura A MEIO do traço** vive no envelope do traço aberto e não no
  comprometido — uma janela que lesse só o comprometido entregava zero a
  pincelada inteira, e o gate que media depois do pen-up **não o via** (a
  mutação P3 sobreviveu até existir o gate a meio).

⏳ **Por medir no smoke do dono:** o relógio do upload incremental com relevo
(o critério de desistência do §4 fala de `> 1 ms`; a escrita é por corrida de
amostras sujas, a `4` bytes cada, ao lado da cor) e a leitura a olho do relevo
sob a luz de fábrica (`Pbr`).

## 6. O CORPO e a SEMENTE do relevo (01/10, dois reports do dono)

Smoke da 3b aprovado, com dois reports: *«o traço tem um relevo indesejado na
borda»* (foto: um anel cinzento em degraus à volta de um traço vermelho) e
*«smooth, knife e outras tools não funcionam no relevo»*.

### 6.1 A causa, medida antes da cura

- **O anel é a ESPESSURA SEM TINTA.** O assentamento do impasto alisa a altura e
  ela espalha-se `~6 px` para fora da cor (medido: cor `22 px` de largura, altura
  `33 px`, e na borda da cor a altura ainda vale `~5` de um pico de `15,5`). No
  Painter 2D a luz pesa o relevo pela cobertura (`impasto_light::paint_body`, a
  cura do halo de 2026-07-12) e o halo não acende; a tela da vista levava a
  altura **sem** a cobertura, e a peça acendia o barro nu. Sonda: `1 894` píxeis
  com altura e sem cor, pico `4,94 px`.
- **As ferramentas trabalhavam numa tela LISA.** Cada pincelada começa por
  semear a tela com o retrato da cor, e essa semente passa pelo `set_source`,
  que apaga o relevo das camadas. Sonda: sem semente o alisar não devolve um
  quadro e a faca devolve `21` sem espessura; com a espessura semeada os dois a
  mudam (`9,7` e `2,8 px`).

### 6.2 A lei que fica

> ⚠️ **Superseded em 04/10 ([doc 30 §19](30_plano_camadas_e_efeitos_na_peca.md)):** o corpo pesa a LUZ
> (a superfície lisa e a com relevo inteiro, misturadas pelo corpo — a lei do 2D), não a inclinação.

- **O relevo é um PAR `[altura, corpo]`** por amostra (`ph2d-mesh-colors`), lido
  pelos mesmos pesos da cor. O corpo é a cobertura da tinta (`0..1`), e o
  *bump* do shader é **escalado por ele** (`clamp(corpo, 0, 1) × gradiente`):
  com corpo `1` o desenho é o de antes **ao bit**; onde a espessura transborda a
  tinta o corpo é `0` e a luz não a lê. ⇒ o halo deixa de acender sem tocar na
  altura.
- **A tela é SEMEADA com o relevo da peça** (`tela_semente_relevo`, a mesma
  rasterização do retrato de cor), em píxeis e com o corpo como cobertura, no
  `entrega` e **só** para um traço que molda relevo.
- **`nova = antes + k·(tela − semente)`**, nos dois canais, com o corpo preso a
  `0..1`. Com a tela intocada a diferença é ZERO e o plano sai ao bit; o alisar
  que BAIXA a tela baixa a peça. ⚠️ A semente que a escultura guarda é a que a
  tela **DEVOLVE** depois de semeada, nunca a enviada: a ida-e-volta pela
  profundidade da camada custa um ULP, e um ULP em cada amostra não tocada
  viraria espessura.
- **O documento sobe a `v5`, com degrau:** um `v4` lê-se com o corpo DERIVADO
  (`1` onde há altura, `0` onde não há) — antes do corpo uma altura não nula só
  nascia debaixo de tinta.

### 6.3 Gates e mutação

| onde | gate |
|---|---|
| Painter | `o_corpo_e_zero_onde_a_espessura_transborda_a_tinta` (a régua é o que a LUZ lê: `altura × corpo` no halo `≤ 5 %` do cru, medido `1,2 %`; e `≥ 99 %` do halo sem corpo, medido `1 890` de `1 894`) · `a_semente_do_relevo_volta_pela_janela` · `sobre_o_relevo_semeado_o_alisar_e_a_faca_trabalham` (com o CONTROLO sem semente) |
| peça | `com_a_tela_semeada_o_que_nao_mudou_nao_mexe` (CONTROLO: sem semente a tela soma outra vez) · `uma_ferramenta_que_baixa_a_tela_baixa_a_peca` · `a_semente_de_relevo_e_o_relevo_da_peca_em_pixeis` |
| placa | `tinta_relevo_no_device` parte (3): a mesma rampa com corpo `0` desenha o liso |
| ficheiro | `um_documento_v4_abre_com_o_corpo_derivado_da_altura` |
| fiação | `R1`/`R2` no censo da tinta fina (a semente pedida no `entrega` · a semente LIDA) |

⚠️ **Uma barra que mudou de forma, com a razão:** a 1.ª redacção do gate do
halo exigia corpo **exactamente zero** e reprovou com `3/255` em `4` píxeis — a
cobertura e o alfa de 8 bits não arredondam no mesmo sítio. A régua passou a ser
a grandeza que o produto LÊ (o relevo aceso), não um sucedâneo dela.

## 7. A normal do relevo deixa de depender da VISTA (01/10, report com foto)

*«Olhando exatamente de cima parece bom! MAS quando se vê inclinado, aparece
artefato de relevo»* — na foto, estilhaços cor-de-rosa claros soltos ao longo da
borda do fim do traço, junto da silhueta.

### 7.1 O mecanismo, reproduzido antes da cura

A D5 tirava o gradiente da altura por **diferenças de ECRÃ** (`dpdx`/`dpdy`,
Mikkelsen 2010). O hardware calcula-as por **bloco de `2×2` píxeis**: de cima a
encosta do impasto ocupa muitos píxeis e o erro não se vê; inclinada ela cabe em
um ou dois, um bloco atravessa a borda (ou uma aresta de triângulo) e a normal
desse bloco aponta para onde calhar — a luz acende-se em **estilhaços**.

Sonda pelo caminho do produto (`diag_o_relevo_visto_inclinado`, um traço de
impasto a `8x` na cena `=52`, de frente e inclinado; `PH2D_SONDA_DIR`): com a lei
antiga a borda inclinada tem fragmentos claros soltos — a foto do dono — e de
cima aparecem riscos ao longo das ARESTAS da malha (blocos que atravessam dois
triângulos); com a lei nova os dois somem.

### 7.2 A lei que fica

- **O gradiente é EXACTO** (`TintaLida::g`): dentro de cada célula da retícula a
  altura é linear (triângulos) ou bilinear (quads), logo as leituras devolvem a
  derivada nos parâmetros delas e o `tinta_no_ponto4` passa-a para o OBJECTO pelos
  gradientes das baricêntricas (`∇βa = n × (pc − pb)/|n|²`) — a mesma geometria
  de que já recuperava as baricêntricas.
- **A luz lê-o pelo gradiente de SUPERFÍCIE** (Mikkelsen 2020):
  `n' = normalize(n − corpo·(g − n(n·g)))`, em espaço de vista (o declive é
  adimensional, só roda). ⛔ **Nenhuma derivada de ecrã** na leitura do relevo.

### 7.3 Gates e mutação

| gate | o que afirma |
|---|---|
| `a_lei_da_reticula_le_o_mesmo_na_placa_e_na_cpu` (3.ª palavra) | o gradiente da placa é o da LEI: diferenças centrais da `altura_*` na CPU, longe das fronteiras de célula, barra relativa `2e-3` |
| `a_normal_do_relevo_nao_tira_derivadas_de_ecra` (sem placa) | o IR do `naga` não tem `Derivative` em nenhuma função `tinta_*`, com o CONTROLO de que a régua acha um `dpdx` |
| `o_relevo_acende_como_a_geometria_que_ele_finge` | a rampa pintada como relevo muda a luz para o MESMO lado e com a mesma ordem de força que a malha LEVANTADA pela mesma rampa — sem ele, um sinal trocado passava no gate de «onde» |

Arnês: `muta_a_normal_do_relevo.sh` (com a placa).

⛔ **Uma régua RECUSADA, com a razão:** *contar píxeis soltos na diferença
relevo − liso* lia a lei nova PIOR na vista rasante sintética (`68` contra `25`) —
ela contava como «solto» o traço contínuo de um píxel que uma encosta virada para
a câmara desenha a `69°`. Um recorte à mão mostrou estilhaços na antiga e um traço
contínuo na nova: *uma régua cujo «defeito» inclui o desenho certo não decide*.

⏳ **Fica nomeado:** a altura é contínua e o gradiente não (por célula), logo numa
encosta muito inclinada vêem-se as facetas da retícula como um traço em degraus —
o mesmo detalhe que a cor tem a esse degrau. Curá-lo é interpolar gradientes por
amostra (o análogo das normais por vértice), com o custo por medir.

## 8. O HORIZONTE da normal inclinada (01/10, 2.ª volta, duas fotos)

Report do dono: *«mesma ponta vista de frente e inclinada»* — de frente a ponta
do traço é redonda e limpa; com a peça rodada até a ponta ficar junto ao
contorno da bola, aparece uma meia-lua dura (escura por dentro, um fio claro por
fora) exactamente na encosta da ponta.

### 8.1 O mecanismo, lido no código antes da cura

O `canvas_normal` do `mesh.wgsl` (`if (n.z < 0.0) { n = -n; }`) existe para uma
casca vista por TRÁS acender como frente. Perto do contorno a normal de BASE já
está quase de lado (`n.z` pequeno), e a encosta da ponta inclina-a para FORA da
peça — passa o horizonte. O `canvas_normal` vira-a INTEIRA: o `z` volta positivo
**e o `xy` troca de sinal**, ou seja a normal passa a apontar para o lado oposto
no ecrã. O matcap e o rig lêem a luz do lado contrário numa linha só: é a
meia-lua da foto. De frente a base tem `n.z ≈ 1` e nenhuma inclinação razoável
chega ao horizonte — daí *«de frente bom, inclinado não»*. ⚠️ A cura da §7 tirou
o ruído de ecrã e **deixou isto à vista**: os estilhaços de antes cobriam-no.

### 8.2 A lei que fica

> ⚠️ 04/10 ([doc 30 §19](30_plano_camadas_e_efeitos_na_peca.md)): o `corpo` saiu desta lei; onde se lê
> «`corpo = 0`», leia-se «sem declive».

`tinta_inclina` (WGSL, pura) = gradiente de superfície da §7 + `tinta_horizonte`,
com o dono da lei em CPU em `ph2d_mesh_render::relevo_normal`:

- o limiar sai da BASE: `t = min(|n.z|, T)`, `T = 0,25` (a ~14° do horizonte);
- acima dele a normal inclinada passa INTACTA (o caminho da §7 ao bit);
- abaixo, o `z` desce por `zc = zmin + (t − zmin)·exp((z − t)/(t − zmin))`, com
  `zmin = K·t`, `K = 0,1` — toca o limiar com valor e declive iguais (sem vinco na
  luz) e nunca atravessa `zmin > 0`; o `xy` reescala para comprimento 1 e
  **mantém a direcção** no ecrã;
- `corpo = 0` devolve a base inteira (sem o `nb/|nb|`, que erra um ULP e cairia
  um ULP abaixo do limiar).

⚠️ `T` e `K` não nomeiam recurso: são a FORMA da compressão, e dizê-lo é a única
forma honesta (§0.0 do roteador).

### 8.3 Gates e mutação

| gate | o que afirma |
|---|---|
| `a_luz_nao_salta_na_ponta_vista_de_lado` | a régua do report: a encosta a crescer para fora sobre uma base quase de lado (`z = 0,05 · 0,15 · 0,3`), passos de `1e-3`; o maior salto da normal DEPOIS do `canvas_normal` fica `< 0,01`. CONTROLO: a lei da §7 salta `> 0,5` ali |
| `a_normal_inclinada_nunca_passa_o_horizonte` | `200 000` entradas aleatórias, a saída fica do lado da base e unitária; CONTROLO: a lei da §7 cruza em mais de `2 %` delas |
| `sem_corpo_ou_acima_do_limiar_nada_muda` | `corpo = 0` devolve a base AO BIT; acima do limiar a saída é a da §7 |
| `a_compressao_toca_o_limiar_sem_vinco` | valor e declive contínuos em `z = t` |
| `a_normal_do_relevo_passa_pelo_horizonte` (IR do `naga`) | o ELO: `tinta_relevo_n → tinta_inclina → tinta_horizonte`, com o CONTROLO de que a régua acha `fs_core_n → canvas_normal` |
| `o_horizonte_le_o_mesmo_na_placa_e_na_cpu` (placa) | o gémeo em WGSL contra a CPU em `4 096` entradas, com o CONTROLO de que mais de `200` caem na compressão |

Mutação: `6 de 6` sangram nos gates sem placa (o elo da entrada · o elo da
compressão · a compressão desligada · o `corpo = 0` sem atalho · o declive do
joelho · o chão negativo) e o CONTROLO sobrevive; a 7.ª (o `K` da placa diferente
do da CPU) só a placa vê.

## 9. A INCLINAÇÃO POR AMOSTRA — os degraus da retícula na encosta (02/10)

O aberto da §7 (*«a altura é contínua e o gradiente não»*), conferido antes de
mexer: o código ainda lia a derivada EXACTA por célula e nenhum commit lhe tocara
depois de 01/10. A sonda `diag_o_relevo_visto_inclinado`, com o canal vermelho
esticado, mostrava blocos do tamanho da célula ao longo do traço — de lado e
também de cima (o brilho do tubo em escadinha).

### 9.1 A lei que fica

A cura nomeada na §7, a das normais por vértice:

- cada célula dá o seu gradiente EXACTO (o da §7: constante no sub-triângulo,
  a derivada da bilinear no CENTRO do quad) aos cantos, pesado pela ÁREA dela
  no objecto; o gradiente de uma amostra é a média — `ph2d_mesh_colors::Inclinacoes`;
- o fragmento interpola os gradientes das amostras pelos MESMOS pesos da cor
  (`Tinta::inclinacao_tri`/`_quad`; no WGSL o `tinta_soma` acumula
  `tinta_inclinacao(i)`, `@binding(8)`, três `f32` por amostra);
- as amostras da fronteira são partilhadas, logo a inclinação é contínua
  ATRAVÉS das arestas da malha — o que a lei por célula não era nem dentro da face.

⚠️ É derivada: não entra no documento nem na fila de desfazer. Precisa das
POSIÇÕES (é um vector do objecto), logo esculpir muda-a sem tocar numa altura.

⚠️ A suavização espalha o declive UMA célula para fora da rampa (a amostra da
beira dá metade à vizinha). Com o CORPO a zero fora da tinta isso não acende
nada; o gate `o_relevo_inclina_a_luz_so_onde_ha_degrau` passou a medir a rampa
mais uma coluna, e além dela nada.

### 9.2 Como se mantém em dia

| quando | o que corre |
|---|---|
| o traço (subida incremental) | `atualiza(sujas)`: as faces das amostras sujas são refeitas inteiras; o anel delas pelos vértices só na BORDA (as únicas células que dão às amostras partilhadas) |
| a subida inteira, mesma topologia (esculpir) | a FOTO (`TintaGpu::inc_foto`: o registo das faces, as posições, as alturas) diz o que mudou — os vértices movidos entram como sujas (o índice da amostra de um vértice é o dele) |
| a subida inteira, topologia nova | `Inclinacoes::nova`: adjacência + só as faces com altura (o 1.º toque não paga o plano) |

Cada face soma num rascunho próprio e os rascunhos juntam-se pela ORDEM das
faces; acima de `32 768` células as faces correm em paralelo (`std::thread::scope`,
a crate continua sem dependências) e o resultado é o mesmo AO BIT com qualquer
número de threads (gate).

### 9.3 Medições (perfil `smoke`, `load < 4`, peça da cena `=52`)

O critério da §4 (*«a subida incremental das alturas não pode custar mais de
`1 ms` por quadro durante um traço na peça de fábrica»*), nunca medido até hoje:

| traço de impasto a `8x` (59 quadros) | escrever na fila (CPU) | `submit` + espera |
|---|---|---|
| ANTES da §9 (só alturas) | mediana `0,039` · pior `0,064 ms` | p95 `0,079` · pior `0,190 ms` |
| com as inclinações | mediana `0,10` · pior `0,20 ms` | p95 `0,16–0,29` · pior `0,6–0,9 ms` |

⇒ a W2 fecha nessa forma, e a §9 também: o quadro paga `~0,1 ms` de CPU.

| degrau | amostras | refazer TODAS (série → paralelo) | dab de pincel (11 vért.) | dab grande (40) | a mesma subida SEM relevo |
|---|---|---|---|---|---|
| `8x` | 47 k | `1,0 → 0,6 ms` | `0,23 ms` | `0,39 ms` | `0,06 ms` |
| `16x` | 188 k | `4,7 → 1,6 ms` | `0,6 ms` | `1,2 ms` | `0,18 ms` |
| `32x` | 754 k | `17 → 6,8 ms` | `2,1 ms` | `3,3 ms` | `0,7 ms` |
| `64x` | 3,0 M | `68 → 27 ms` | `8,1 ms` | `12,2 ms` | `6,1 ms` |

A coluna da direita é o que o quadro de esculpir já pagava: **a subida inteira
corre em todo quadro em que a peça muda de forma com plano**, e a `64x` ela são
`6 ms` de cópia do plano. ⏳ Esse é o próximo ganho do caminho de esculpir (subir só
as faces dos vértices movidos), e é anterior a esta secção.

`LIMIAR_PARALELO = 32 768` células: a `12 ns` por célula numa thread são `~0,4 ms`,
a ordem do que custa lançar as threads; a `94 k` células (o `8x` inteiro) o
paralelo já ganha (`1,03 → 0,6 ms`).

### 9.4 Gates

| gate | o que afirma |
|---|---|
| `a_inclinacao_lida_e_continua_e_a_lei_por_celula_reprova_a_mesma_regua` | o maior salto entre passos vizinhos numa linha de `4 000` passos: `1,1e-4` por amostra contra `4,2e-2` por célula (o CONTROLO, `380×`) |
| `a_inclinacao_por_amostra_e_a_da_superficie` | contra a inclinação VERDADEIRA de uma altura lisa numa grelha NÃO alinhada aos eixos: `1,6 %` no nível 3, `0,4 %` no 4 — ordem dois |
| `os_dois_lados_de_uma_aresta_leem_a_mesma_inclinacao` | quad\|quad e quad\|triângulo |
| `a_atualizacao_por_pedacos_da_a_inteira_e_diz_o_que_mudou` · `mover_vertices_e_atualizar_por_eles_da_a_inteira` · `um_plano_com_pouca_altura_nasce_igual_ao_inteiro` | as três portas incrementais contra a inteira |
| `as_faces_em_paralelo_dao_o_mesmo_ao_bit` | `1` contra `8` threads, inteira e atualização |
| `a_lei_da_reticula_le_o_mesmo_na_placa_e_na_cpu` (3.ª palavra) | a placa contra `inclinacao_*` em TODO ponto (a lei antiga só se media longe das fronteiras de célula), barra `1e-4` |
| `esculpir_com_relevo_desenha_o_que_uma_subida_do_zero_desenha` (placa) | a foto: a peça esculpida desenha igual a um renderizador novo |
| `o_relevo_subido_por_pedacos_desenha_o_que_uma_subida_do_zero_desenha` (placa) | o incremental do traço sobe as inclinações VIZINHAS das sujas |

Arnês: `muta_a_normal_do_relevo.sh`, re-escrito (as N1–N4 de 01/10 mutavam a lei
por célula e o pré-voo apanhou-as mortas).

### 9.5 ⛔ Recusas medidas

| o que | porquê |
|---|---|
| curar só DENTRO da face (diferenças centrais na retícula da face, sem adjacência) | deixa um vinco ao longo de cada aresta da malha — a família dos «riscos ao longo das arestas» que a §7.1 já tinha fotografado |
| deixar a lista de vértices movidos ao CHAMADOR | o `dirty` é limpo antes de o plano subir, e um desfazer troca as alturas sem mover nada: a foto é a única régua que não depende de o chamador acertar sempre |
| um 9.º buffer de armazenamento na placa | o piso do WebGPU é `8` por estágio; o fragmento usa `7` (o arnês de paridade, que juntava `2`, passou a um buffer só de entrada e saída) |
