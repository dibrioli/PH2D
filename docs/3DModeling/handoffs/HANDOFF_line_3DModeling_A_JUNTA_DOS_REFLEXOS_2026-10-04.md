# Handoff — `line/3DModeling`: A JUNTA DOS REFLEXOS (2026-10-04)

> **Continuação, não integração.** Report 3 do dono (§11 do
> [AS_CAPTURAS_DE_REFLEXO](HANDOFF_line_3DModeling_AS_CAPTURAS_DE_REFLEXO_2026-10-04.md)): *«quando dois reflexos
> estão próximos mesmo quando os objetos estão separados … é como se o reflexo fizesse operação booleana»*.
> Base `1ad60a1ce` (o main não andou). Commits: `5e4e6e1e6` a sonda move peças · `48a600fb1` red-first (oráculo
> `junta`, a régua da faixa, o gémeo da busca) · `0962d8470`/`7af09f5ef` WIP · `8875f7a2c` a cura · `46135d865` a guarda que a
> mutação mostrou redundante · `ea72de353` docs · **report 4** (§9): `48a600fb1..81292c496` (WIP `861f5ddab`,
> `7a22b2f4c`, …) e `81292c496` a cura · o commit desta atualização. **Smoke do dono: PENDENTE** (o do report 3
> voltou com o report 4).

## §1 — O que mudou

- **A captura guarda o 2.º momento da distância.** `fs_sonda` escreve `(d, 1, d², 1)` na textura da distância
  (o canal `b` estava a `0`). Ele atravessa TODA média — o resolve do MSAA das faces, as 4 amostras de `fs_octa`,
  `fs_desce`, o filtro linear da leitura — sem se perder, e normalizado pela cobertura (`b/g`) a borda contra o
  FUNDO não tem variância; só a borda entre DUAS vizinhas a distâncias diferentes a tem (o truque das sombras
  por variância).
- **`sonda_aresta(g)`** (`sonda_le.wgsl`): desvio `> ARESTA · média` (`ARESTA 0,1`, `gpu_sondas.rs`) ⇒ a
  distância lida é FANTASMA. Na busca (`sonda_marcha`), o cruzamento cujo lado de lá é aresta não é aceite nem
  dá franja (o raio continua).
- **`PASSAGEM 0,5`**: a busca passa ao ponto fixo só nos últimos `0,5` níveis abaixo de `LOD_BUSCA`
  (rugosidade `0,1 → 0,2`); abaixo disso (o cromo da cena 42, `0,05`) é só a busca, e o que ela falha é nada.
- **Sonda do app**: `PH2D_SONDA_MOVE=<folha>:x,y,z;…` (folha na ordem do documento) — a pose de um report.
- **Testes**: `tests_reflexo_junta` (gate) e `tests_sonda_junta` (sonda: a busca portada para a CPU sobre a
  captura lida de volta, por leitor e por limiar); `tests_sonda_cpu::texel` (o texel mais próximo);
  `tests_reflexo_perto::{Vista.alvo/meia, JUNTA, desenha_ate, vizinha_refletida}`.
- **Oráculo**: `oraculo_reflexo_perto_blender.py -- … junta` → `fixtures/oraculo_reflexo_junta.csv.gz`
  (`1,2 MB`): a caixa azul LONGE meio atrás da verde PERTO vistas do centro do cromo, com fundo entre elas visto
  de cada ponto do cromo; a vista amplia o par refletido (`meia 0,14`). E `… sobreposta` →
  `oraculo_reflexo_sobreposta.csv.gz` (a pose do report 4, a câmara do app de lado: o reflexo junto ao limbo).
- **Report 4** (§9): `ESPESSURA`/`FRANJA` `0,05`/`0,1` → `0,01`/`0,01`; na aresta, `sonda_fora_na_aresta` (a
  silhueta da vizinha da frente a meio da mistura, a de trás pelos momentos, só com o texel todo das duas:
  `DUAS 0,95`) e a cor lida na da frente pura adiante (`ARESTA_PASSOS 4`); a cor do ACERTO opaca
  (`rgb / max(a, 0,5)`). Gate novo `tests_reflexo_junta::o_reflexo_nao_alarga_alem_da_vizinha` (a ORLA, de fora e
  de dentro — esta só onde o centro do cromo VÊ o ponto, `tests_reflexo_perto::visto_do_centro`); sondas
  `tests_sonda_junta::{sonda_da_tira, sonda_da_orla_de_dentro}` (a busca traçada cruzamento a cruzamento).

## §2 — A causa (medida, não suposta)

| pergunta | medição |
|---|---|
| reproduz pelo produto? | sim — cena 42, `PH2D_SONDA_MOVE='4:0.7,0.15,1.7' PH2D_SONDA_YAW=1.45 PH2D_SONDA_MEIA=0.33 PH2D_SONDA_ALVO=0,0.3,0`: uma ponte azulada translúcida liga o reflexo da verde ao da azul (as fotos do dono: `images/7.png`, `8.png` da sessão `13764661…`) |
| a régua vê? | a FAIXA entre os dois reflexos (px cujo raio, pela geometria, não acerta nada, a > 2 px de quem acerta, com uma vizinha DIFERENTE de cada lado na linha): `1 533 px`, `|Δ|` médio **`0,597`**, **`1 268`** grosseiros (nítido); `0,550 / 1 265` a `0,05` |
| é a interpolação da distância? (gémeo da CPU, a busca do produto sobre a captura lida) | com a azul, a busca ACEITA `1 233` px da faixa (bilinear); com o TEXEL mais próximo ainda `330` — a distância já vem misturada nos DADOS (MSAA + amostras do octaedro), não só na leitura; sem a azul, `0` |
| e o 0,05? | só a busca (temporário `busca = 1`): `0,0185 / 28`. O PONTO FIXO pesava `~37 %` no quase nítido (`(1 − busca) + busca · lod0` a `lod0 0,25`) e cai, pela imprecisão própria (níveis largos), em cima das vizinhas — não numa aresta (marcar o ponto fixo pela aresta mexeu `0,172 → 0,170`: recusado) |

## §3 — Números (A/B na mesma máquina, antes → depois)

| régua | antes | depois |
|---|---|---|
| junta nítido (faixa: `|Δ|` / grosseiros) | `0,597 / 1 268` | **`0,0142 / 29`** |
| junta `0,05` | `0,550 / 1 265` | **`0,0177 / 26`** |
| perto nítido (miolo / grosseiros) | `0,0126 / 919` | `0,0126 / 925` |
| perto `0,05` (miolo / contorno / grosseiros) | `0,0226 / 0,0938 / 820` | `0,0223 / 0,0982 / 858` — o contorno a `0,05` ficou tão nítido como o espelho (o ponto fixo borrava-o) |
| par nítido e `0,3` | `0,0069 · 0,0417` | iguais |
| gémeo: dos `42 711` px que DEVEM acertar, aceites | `41 875` | `41 155` (`−1,7 %`: 1–2 px da borda de uma vizinha junto da OUTRA) |
| **custo**, 1080p, release, A/B alternado `antes·depois·antes·depois`, load `4,1–4,8` | girar 16: `+0,46 / +0,41 ms` | `+0,47 / +0,40 ms` — nada além do ruído; arrastar 16: `+9,71 / +10,04` × `+10,03 / +9,77 ms` |

Suíte com placa da crate (report 3): **42/42** (`tests_reflexo`, `tests_reflexo_perto`, `tests_reflexo_junta`,
`tests_sonda_cpu`, `tests_sondas` — o cache ao byte e girar perto —, `tests_chao_tapa`, `tests_contacto`, …).
clippy `--all-targets` das duas crates limpo; fmt limpo.

## §4 — ⛔ Recusas MEDIDAS (não reconstrua)

| recusado | por quê |
|---|---|
| ler o texel MAIS PRÓXIMO na busca (sem filtro) | ainda `330` de `1 533` aceites: a mistura está nos dados |
| `ARESTA 0,2` / `0,05` | os três dão `0` aceites na faixa; `0,2` não marca pares de profundidade próxima (`d₂/d₁ < ~1,5`: desvio máximo `< 0,2`), `0,05` perde mais acertos legítimos (`41 069`); `0,1` marca `d₂/d₁ ≥ ~1,2` |
| na aresta, ler a distância «pura» uns texels adiante (`3` passos) | recupera `+278` acertos (`0,65 %`), a foto não muda e nenhum gate o veria — retirado |
| marcar o PONTO FIXO pela aresta | `0,172 → 0,170`: a ponte a `0,05` não vinha dali |
| o controlo «sem a azul» com a barra da média | lê `0,09`: é a FRANJA (abaixo), não ponte; o controlo «sem capturas» lê `0,107` (a máscara da zona da lei antiga escurece a faixa) ⇒ o controlo afirma só os GROSSEIROS (`35 ≤ 50`; a ponte dá `1 268`) |

## §5 — Gate e prova de mutação

`tests_reflexo_junta::entre_dois_reflexos_o_cromo_mostra_o_fundo` (nítido e `0,05`; barras `|Δ| < 0,03`,
grosseiros `≤ 50`; CONTROLO sem capturas: grosseiros `≤ 50`; foto `nossa | Cycles | faixa` com
`PH2D_REFLEXO_FOTOS`). **Mutações (agente `mutacao`, controlo verde `0,0142·29 / 0,0177·26`): 5 de 6
VERMELHAS** — sem o 2.º momento, `sonda_aresta → false`, `ARESTA 10`, `if (true)` na busca (todas `0,597·1 268 /
0,564·1 265`); `PASSAGEM 1` (só o `0,05`: `0,133·482`). **Sobreviveu** o `g.y > 0.5 &&` dentro de `sonda_aresta`:
o lado de lá de um cruzamento é coberto por construção ⇒ a guarda saiu (`46135d865`).

## §6 — ABERTO

- **O lado escondido da vizinha** (a face que só o ponto do cromo vê: a lateral e o topo da caixa, que o Cycles
  mostra) continua — o limite de UMA captura; o candidato medido-a-fazer é guardar também o FUNDO (a distância
  das faces de trás: `λ ∈ [frente, fundo]` é «dentro» — exacto para vizinhas convexas), recusado antes para
  outro uso (fiapos no contorno, §9 do AS_CAPTURAS) e com o custo de mais um passe de faces.
- **O limbo do cromo**: na borda de uma vizinha refletida em raspão extremo a busca não acha o cruzamento
  (px `168–198` da `sobreposta`, já antes de hoje) — o resto da orla de dentro.
- A borda dura tem o serrilhado de `1 px` (o Cycles do oráculo é pontual; o nosso não filtra o conteúdo do
  reflexo).
- ⛔ **Arrastar 16 peças de METAL**, re-medido CALMO (load `< 5`): `+9,7–10,0 ms`, quadro `12,7 ms` a 1080p (o
  handoff anterior dizia `~+8 ms` com load `~15`). Alavancas medidas lá: amostras `64 → 16`, capturas só das
  peças brilhantes, LOD.
- Herdados: §5 e §9–§11 do [AS_CAPTURAS_DE_REFLEXO](HANDOFF_line_3DModeling_AS_CAPTURAS_DE_REFLEXO_2026-10-04.md).

## §7 — Para o integrador

`ph2d-mesh-forward`: consts novas `ARESTA`, `ARESTA_PASSOS`, `DUAS`, `PASSAGEM` (WGSL `SONDA_*`); mudadas
`ESPESSURA 0,05 → 0,01`, `FRANJA 0,1 → 0,01`; a textura da distância das capturas ganha o canal `b = d²·a`
(mesmo formato `Rgba16Float`, nenhuma ligação nova). Fixturas novas `~1,2 MB` + `~1,0 MB`. `ph2d-app-field3d`: só a sonda `#[ignore]` (`PH2D_SONDA_MOVE`). `PROJECT_SCHEMA` intacto; shell
`0` linhas; contratos congelados: nenhum; cenas: nenhuma nova (a próxima continua a 43).

## §8 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**. Arraste a caixa AZUL para perto da câmara, como nas suas fotos.
3. Deve acontecer: na bola de cromo, o reflexo da caixa verde e o da azul ficam SEPARADOS, com o fundo entre
   eles — nenhuma ponte nem cunha a colá-los; e a borda da verde é recta e igual de cima a baixo, também onde a
   azul fica atrás dela — sem buraco nem degrau. A girar a câmara e a arrastar a azul.
4. Deu errado se: aparecer uma ponte (mesmo translúcida) entre dois reflexos, um reflexo perder um pedaço junto
   do outro, ou pontos claros a piscar na borda de um reflexo.

## §9 — ⛔ REPORT 4 DO DONO (04/10): o BURACO

*«agora fica um espaço vazio onde os objetos refletidos deveriam estar sobrepostos»* — 1 foto: a borda direita
da verde refletida com um recorte junto à azul.

**A medição derrubou a leitura óbvia.** Reproduzido (`PH2D_SONDA_MOVE='4:0.6,0.15,1.8'`, a mesma câmara) e contra
o Cycles na pose (`sobreposta`): o troço com a azul atrás estava na posição CERTA; o resto da borda é que
ALARGAVA (a espessura `5 %` e a franja `10 %` aceitavam raios que passam por trás da silhueta, vista do centro) —
ao lado do alargado, o certo parecia roído. E por dentro a borda lia MEIA cor (a cobertura `0,5` do texel da
silhueta, onde o cruzamento cai), uma tira mole de vários px de raspão.

| régua (sobreposta, nítido; Cycles) | antes do report 4 | depois |
|---|---|---|
| orla de FORA (px sem vizinha a `2–16 px` dela) | `0,0255 / 775` | `0,0169 / 177` |
| orla de DENTRO (acertam, o centro vê o ponto) | `0,0423 / 822` | `0,0234 / 396` (o resto: o limbo, `1 px` de contorno) |
| orla de dentro, `0,05` | `0,0647 / 896` | `0,0265 / 394` |
| junta (faixa, nítido · `0,05`) | `0,0065 / 0` · `0,0110 / 0` | `0,0076 / 2` · `0,0121 / 2` |
| perto, miolo nítido · `0,05` | `0,0126` · `0,0223` (manhã) | `0,0113` · `0,0116` |
| contorno do par (grosseiros) | `465` | `280` |

As curas, cada uma medida isolada: (1) `ESPESSURA`/`FRANJA` `0,01` (a varredura satura aí; tabela na doc de
`gpu_sondas::ESPESSURA`); (2) numa aresta a mistura empurra o cruzamento para DENTRO da da frente (o raio lá já
está `1,5–3,6 %` atrás): mede-se o atrás na SILHUETA dela, onde a distância lida é a média das duas
(`sonda_fora_na_aresta`); a de trás sai dos momentos do texel (`d_trás = média + variância/(média − d_frente)`),
porque às vezes ela é só uma lasca sem leitura pura; com céu no texel (três superfícies) a conta não vale (`DUAS`:
faixa `12 → 2`); (3) a cor do acerto, opaca: `rgb / max(a, 0,5)` no próprio cruzamento.

⛔ **Recusas medidas desta resposta:**

| recusado | por quê |
|---|---|
| a régua «a de trás não muda a da frente» (com × sem a azul) | a vista SEM a azul é a errada (mole e larga): a régua castigava a certa |
| o atrás medido no INÍCIO da zona misturada | reabre a junta (faixa `0,0065 → 0,066`, `111` grosseiros), dentro quase igual |
| a de trás só por leitura PURA para trás | a azul vista do centro é uma lasca toda misturada: a conta desistia (`1e9`) e o buraco ficava |
| ler a cor uns texels ADIANTE, onde a vizinha cobre o texel | o friso claro do bisel da caixa lido aos saltos: os pontos claros do report 1 (foto) |
| o oráculo com a azul TODA escondida atrás da verde vista do centro | nenhuma captura única a mostra (limite); trocado pela pose do app |

**Mutações (agente `mutacao`, controlo verde): 8 de 8 VERMELHAS** — `ESPESSURA 0,05` (junta `76` grosseiros),
`FRANJA 0,1` (orla de fora `1 172`), a busca sem `sonda_fora_na_aresta` (junta `1 039`), `meio` sem a variância
(dentro `606`), a cor sem tirar a mistura (dentro `634`, perto `0,05` miolo `0,0228`), sem o passo para a da
frente na cor (dentro `621`), `if (true)` no ramo `m ≤ d_frente` (dentro `587`); `DUAS 0` **sobrevivia** às barras
largas (junta `2 → 12`, orla de fora da junta `8 → 26`) ⇒ as barras da vista da junta passaram à medição
(`BARRA 6`, `BARRA_ORLA_JUNTA 15`; o controlo sem capturas tem a sua, `50`) e ela sangra nos dois testes.
Suíte com placa: **45/45**; clippy `--all-targets` e fmt limpos.

**Custo** (1080p, release, A/B alternado, load `~1,2`): o quadro a GIRAR 16 peças de metal `2,47–2,53 → 2,82 ms`
(`+0,3 ms`), 8 peças `+0,1 ms`; a ARRASTAR 16 `12,7–12,8 → 13,6 ms`. Parte aparece com as capturas DESLIGADAS
(`2,02 → 2,30 ms`): o código da busca maior pesa no passe inteiro (registos), não só nas arestas — alavanca
não medida: tirar `sonda_fora_na_aresta` para um laço que não se desenrola.

## §10 — ⛔ DECISÃO DO DONO (04/10) e a PRÓXIMA ETAPA

*«não melhorou»* (sem foto) depois do report 4. Perguntado se a técnica é a das grandes engines: **não** — as
capturas (Unreal reflection captures, Unity reflection probes, Godot) só servem de reflexo de FUNDO/borrado em
todas; a busca dentro da captura (Szirmay-Kalos 2005) é académica e tem um tecto que não se fura (UM ponto de
vista por peça: lateral escondida, uma vizinha atrás da outra — cada cura abriu outro defeito: ponte → buraco →
borda). Proposto o raio de verdade na placa (ray query): ⛔ **RECUSADO pelo dono — «quero que tudo funcione em
qualquer hardware»** (WebGL2/celular incluídos). Não proponha ray tracing por hardware outra vez.

**Próxima etapa (para a janela nova):** o reflexo PELA TELA (screen-space reflections) POR CIMA das capturas —
o padrão das engines que corre em qualquer placa (um passe de ecrã que marcha o raio refletido sobre a
profundidade do quadro): nítido e exacto para o que está à vista, e onde o raio sai da tela ou passa por trás
do que se vê, cai na captura de hoje. Antes de escrever: (1) pedir ao dono a FOTO do que ainda vê errado (é a
régua — as minhas não viam o que ele vê); (2) oráculo = os mesmos Cycles (`junta`, `sobreposta`, `perto`, `par`)
com a câmara do app, e réguas da faixa/orla já existentes; (3) custo medido inclusive no caminho GLES/WebGL2
(o tecto do celular), com o passe em meia resolução como alavanca. Candidato a medir depois, também sem placa
especial: marcha contra o CAMPO de distância da cena (o que o Lumen faz em software) — o módulo já é um campo.

## §11 — REPORT 5 DO DONO (04/10): o GAP é a vizinha escondida do centro

2 fotos (sessão `0bbd8ca2…`, `images/2.png`, `3.png`): o ALUMÍNIO (cena 42, folha 5) com a rugosidade a `0` pelo
dono; a azul arrastada para trás da verde. No reflexo: a verde, um VAZIO, e só depois um pedaço da azul.
*«Curiosamente para a sombra da caixa azul o gap não acontece.»*

Leitura (coerente com toda a medição desta janela, não reproduzida nesta pose): o vazio é a parte da azul que o
ponto do metal vê e o CENTRO da captura não vê (tapada pela verde) — o limite de UMA captura, que a ponte de
antes «enchia» por acaso. A sombra não tem o vazio porque não vem da captura (é a lei do chão, da luz). Duas
curas, ambas em qualquer placa: (a) o reflexo PELA TELA (§10) — a azul está à vista nas fotos do dono, logo
fica exacta; (b) uma 2.ª CAMADA na captura (a próxima superfície VIRADA para o centro atrás da primeira:
faces de novo com a profundidade da 1.ª como teste, as de trás descartadas — Shade et al. 1998, *layered depth
images*), que cobre também o que está fora da tela; custo: um passe de faces a mais só quando a cena muda e
`~+50 %` de memória por captura. Ordem proposta: (a) primeiro (o caso do dono), (b) medido depois.

Também na 2.ª foto (sem seta do dono): DENTES no contorno onde a azul entra no cromo grande — é o item (d) já
aberto do O_CHAO_QUE_TAPA (*«dentes de 2–3 px na quina côncava»*), não as capturas.
