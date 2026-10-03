# Handoff — `line/3DModeling`: O CHÃO QUE TAPA (2026-10-03)

> **Wave, não integração.** Candidato (b) do §8 do [O_CONTACTO](HANDOFF_line_3DModeling_O_CONTACTO_2026-10-03.md):
> a parte de baixo de uma peça pousada via o céu de BAIXO do estúdio. Base da linha `1ad60a1ce` (main não
> andou). Commits: `f74e00a2b` a lei, o oráculo, os gates, a cena 41, a sonda · `7820979ce` a memória por
> pixel · `19c32bd00` as direcções no laço · `e7260f990` o verniz no oráculo + o gate da memória
> (`tests_custo_chao_tapa.rs → tests_passe_chao_tapa.rs`) · `58c65a45d` clippy. **Smoke do dono: pendente.**

## ⛔ A premissa que a medição derrubou

*«O chão tapa as peças» como paridade com o traçado* — o Render traçado **também** nunca deixou o chão
tapar peças (`ground_bounce.rs`: «o chão é invisível e não entra na marcha»). É física NOVA, não paridade:
a régua é o **Cycles**. E o céu do chão da onda anterior tinha um defeito que só esta cena achou:
`ceu_chao.wgsl` fundia TODAS as peças de uma fatia de azimute num ÚNICO intervalo de elevação — uma peça a
flutuar à frente de uma pousada enchia o vão de céu entre as duas e o chão ficava escuro demais ali. Achado
porque a barriga da esfera pousada lia `−0,052` contra o Cycles na cena de 3 peças, enquanto cada variante
de 2 peças lia `−0,019` / `+0,015` e a esfera sozinha `+0,005`. Cura: até **DOIS** intervalos disjuntos por
fatia (`junta`/`toca`/`uniao`/`largura` em `ceu_chao.wgsl`; um 3.º disjunto funde-se no mais próximo):
viés da barriga `−0,052 → +0,0075`. Os gates do chão (`tests_chao`) não se mexeram: linha `0,0062/0,022` ·
coluna_esfera `0,0018/0,011` · coluna_caixa `0,0055/0,030` · diagonal `0,0036/0,024` (médio/máx).

## §1 — O que existe

**`ph2d-mesh-forward`, `src/chao_tapa.rs` (NOVO, `pub mod`)** — o chão é o plano infinito `y = chão`.
- **Difuso:** `D(p,n) = ∫_{d.y<0} (1 − V(x(d)))·max(n·d,0) dω/π`. O total abaixo do horizonte é forma
  fechada `(1−n.y)/2`; `RAIOS_CHAO = 16` direcções FIXAS (espiral de Fibonacci no hemisfério de baixo, sem
  referencial tangente ⇒ sem costura) só dão a MÉDIA ponderada pelo cosseno de `1−V` (exacta para `V`
  constante). `V` lê-se no céu do chão onde o raio bate no plano.
- **Reflexo** `reflexo(p, r, α)`: o lóbulo GGX em torno da direcção reflectida em **DOIS anéis** nos quantis
  `1/4` e `3/4` da distribuição do half-vector (`tan²θh = α²q/(1−q)`, reflectido a `2θh`), `TAPS_ANEL = 4`
  por anel, pesos cosseno a `r` (o pré-filtro do céu com `n=v=r`); tap descendente lê o chão onde bate,
  ascendente é céu; `α→0` colapsa no espelho exacto.
- **Gémeo WGSL** gerado por `chao_tapa::wgsl()` para o slot `{CHAO_TAPA}` de `forward.wgsl`:
  `ceu_do_chao(p)` (a ÚNICA porta da textura do céu do chão; `visibilidade_do_chao` chama-a),
  `chao_tapa`, `chao_reflexo` (laços com as direcções pela MESMA fórmula da CPU) e a memória por pixel
  `chao_tapa_no_pixel` / `chao_reflexo_no_pixel` (o material pede o lóbulo dieléctrico e o metálico com a
  mesma dir e `α`).
- **`forward.wgsl`:** `ponto` privado, `chao_tapa_ligado` (`quadro.chao.y > 0.5 && quadro.sombra.w > 0.5`,
  posto em `luz_de_cena`); `env_irradiance_da_cena` subtrai `ceu_de_baixo()·D` (`ceu_de_baixo = ceu_irr_sem(−y)`);
  `env_radiance_da_cena` multiplica a parte do céu SEM caixa por `(1 − reflexo)`. A parte da caixa/sol não
  se mexe. Precedente: *Lower Hemisphere Is Solid Color* do Unreal SkyLight — aqui a cor de baixo é o chão
  NAQUELE ponto.

**`ph2d-app-field3d`:** `malha_render_quadro.rs` ganha `SEM_CHAO` (atómico, só em teste, aplicado ANTES da
assinatura — senão a cache do quadro devolve «Mesmo»); a sonda `malha_render_sondas.rs` ganha
`PH2D_SONDA_PITCH=<rad>` e `PH2D_SONDA_SEM_CHAO=1`. **Cena 41 «O CHÃO QUE TAPA»** (`smoke_scenes_chao.rs`):
bola branca mate `r 0,3` pousada `z −0,7` · bola cromada `r 0,22` pousada `z −0,05` · bola pequena `r 0,15`
a flutuar `4 cm` (`y 0,19`) `z 0,45` · caixa cinzenta meia-aresta `0,15` pousada `z 0,95`; `materiais_41`.

### Ids / consts (colisão na integração!)

| o quê | valor |
|---|---|
| consts | `RAIOS_CHAO = 16` · `TAPS_ANEL = 4` · `QUANTIS = [0,25; 0,75]` |
| `smoke::scenes::CENAS` | `40 → 41` (o integrador **reconta**); notas `1..41` em `smoke.rs` e `lib.rs`; `QUEM_SEMEIA` `(41, …)` |
| bindings / pipelines | **nenhum novo** (o céu do chão já está na `15`); `Quadro` igual; `Objeto` igual em tamanho (`80 B`) — `extra.yzw` passa a levar a PEGADA da peça (`chao_tapa::pegada`: centro `x, z` e raio) |
| `PROJECT_SCHEMA` · shell | intacto · `0` linhas |
| fixtura | `ph2d-mesh-forward/fixtures/oraculo_chao_tapa.csv` (`13 924` linhas, `1 817 920` B ≈ `1,8 MB`) |
| instrumento | `docs/3DModeling/ferramentas/oraculo_chao_tapa_blender.py` · `instrumento_custo_chao_tapa` |

**Oráculo (Cycles, Blender 5.2.2):** céu uniforme `1`, chão difuso branco infinito em `y = 0` invisível à
câmara, peças BRANCAS ao raio de câmara e PRETAS nos ressaltos (*Is Camera Ray*: tapam mas não se
inter-reflectem — a nossa lei não tem inter-reflexo entre peças), `1` ressalto difuso, câmara ortográfica
BAIXA em `(0,6; 0,15; 1,0)` para ver as barrigas. Cena: caixa de aresta `0,4` em `(0,4; 0,2; −0,05)`, esfera
`r 0,3` pousada em `(−0,45; 0,3; 0)`, esfera `r 0,15` a `6 cm` em `(0,05; 0,21; 0,5)`. Corridas: `sem` (chão
escondido) · `com` · `preto` (albedo do chão `0` = controlo de forma fechada: pontos livres vêem exactamente
`(1+n.y)/2`, medido `0,0024`) · `espelho`/`áspero` (metal branco GGX `r 0`/`0,5`) sem/com · `verniz`
(Principled metal `0,5` + coat `r 0` ior `1,5`) sem/com. Sanidade: esfera `r 0,3` sozinha pousada, vista de
baixo através do chão, Cycles contra a lei com o chão `(r/D)³` exacto `= 0,001` por faixa (a lei é exacta;
o resíduo da cena de 3 peças vem dos vizinhos).

## §2 — Medições

| o quê | resultado |
|---|---|
| Lei CPU contra o integral denso (`a_lei_do_chao_que_tapa_e_o_integral`, barriga de esfera pousada sobre o chão `(r/D)³`) | `4/8/16/32` direcções, médio `0,0180/0,0101/0,0047/0,0022` (máx a `16`: `0,0135`) |
| Reflexo CPU contra o lóbulo GGX denso (`o_reflexo_e_o_lobo`, `660` lóbulos, `α 0,0625/0,25/0,5`) | `4` por anel médio `0,0159` máx `0,111` (pior: reflexo rasante `r.y −0,31` perto do contacto, `α 0,5`); `8` por anel `0,0120/0,093` |
| **Placa contra Cycles, difuso** (`o_chao_tapa_as_pecas_como_no_cycles`, `11 783` px) | médio `0,0098` · onde o chão escurece (`6 027` px) `0,0130` · máx `0,074` · **SEM a lei** `0,0721 / 0,1189` · contacto contra `sem` `0,0074` · viés da barriga `+0,0075` (`335` px). Esfera sozinha na placa: médio `0,0049`, máx `0,033` |
| Placa contra Cycles, reflexo (razão `com/sem`, `o_reflexo_do_chao_e_o_do_cycles`) | espelho `0,0030` / onde escurece `0,0082` (sem a lei `0,0801`) · áspero `0,5` `0,0155` / `0,0268` (sem `0,0883`) · verniz `0,0147` / `0,0265` (sem `0,0881`) |
| Custo, passe em ecrã cheio 1080p (RTX 5060 Ti, `instrumento_custo_chao_tapa`) | difuso `+0,137–0,148 ms` · reflexo (`4`/anel) `+0,075–0,084` · ambos `+0,24–0,25` |
| Custo, quadro real (cena 41, câmara baixa `pitch 0,1`, `1920×1080`, sonda do quadro a girar, mediana, load `3,2–3,7`) | SEM a lei `1,43–1,48 ms` · COM `1,77–1,79` ⇒ **`+0,30–0,34 ms` (`~23 %`)**. Desenrolado × laço: igual (laço `1,71–1,79`, desenrolado `1,63–1,73`, load `~4,2`). Ganho da memória: `3/3` corridas, `~0,07 ms` (`1,83/1,88/1,90` contra `1,97/1,94/1,92`). Celular (regra ×13) `~4 ms` em ecrã cheio — **NÃO medido num aparelho** |
| Costura do app (`o_chao_que_tapa_chega_ao_quadro_do_render`, cena 40) | `2 590` de `24 286` px de peça escurecem `> 5 %`, o mais escuro `× 0,474`, `0` clareiam. A costura do contacto segue verde (`1 751` px `> 10 %`, `× 0,258`) |

- O máx `0,074` é a esfera pousada de frente para a flutuante: o contacto E o chão escuro atrás do vizinho
  contam-se duas vezes (produto multiplicativo de oclusões, a mesma família do contacto).
- O gate difuso usa material SÓ difuso (`specular_weight 0`, = o do oráculo): o cinzento por omissão, com
  especular, lia a barriga `−0,05` porque em rasante o especular pesa e lê o chão escuro — física que o
  Cycles difuso não tem.

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| Uma UNIÃO por fatia no céu do chão | a barriga lia `−0,052` na cena de 3 peças (um flutuante enche o vão da pousada) |
| Misturar nítido/largo por `α` para o reflexo áspero | era palpite; o oráculo recusou: `0,060` onde o chão escurece, com os dois sinais |
| `8` direcções no difuso | contra o Cycles `0,0098 → 0,0149` (CPU `0,0101`) |
| `8` taps por anel | `0,0233` contra `0,0268` ao DOBRO do custo (`+0,166` contra `+0,075 ms`) |
| 3.º anel nos quantis `1/6, 1/2, 5/6` | mais perto do lóbulo denso na CPU, NADA contra o Cycles na placa (`0,0236 → 0,0235`): o resíduo é o pré-filtro `n=v=r`, não a amostragem |
| Direcções desenroladas | mesmo custo do laço, shader maior |
| O cinzento por omissão no gate difuso | o especular dele lê o chão escuro; o Cycles do oráculo é difuso |
| ⛔⛔ Texels inválidos (DENTRO de uma peça pousada) lidos como `1` pela lei | **REVOGADO pelo report do dono**: o reflexo do cromo via a base das vizinhas ACESA e a base de cada peça (que só vê o chão debaixo dela) não escurecia — vista de baixo, base `0,3701` (máx `1,0`) contra o Cycles. A lei lê agora `ceu_do_chao_tapado` (`V × validade` cru: dentro = `0`, às escuras); a sombra do chão continua a ler `1` dentro (`ceu_do_chao`). Base `0,0107` |
| O reflexo a ler o chão TODO (as sombras das vizinhas incluídas) | **report 2 do dono**: *«reflete a sombra do objeto mas não o objeto, e o objeto está sobre a sombra»* — o reflexo sabe céu e chão, não as outras peças. Agora `sombra_propria`: só a zona da peça (inteira até `1,5` raios, a nada aos `3`). Fora da zona `0` de `4 012` px escurecem (sem a máscara `949`) |
| A grelha do contacto da PRÓPRIA peça como sombra dela no chão (em vez da distância) | imprecisa rente à peça, onde o chão está: base de baixo `0,073` contra `0,0081`, e `334` sombras órfãs fora da zona |
| A régua ABSOLUTA (`com/solo`) a afirmar o reflexo de cima | dominada pelo contacto suave, que não faz reflexos nítidos das vizinhas (áspero `0,060` contra `0,021` na razão); imprime-se, não afirma |
| Não pintar o chão visto de baixo (guarda em `escuro_do_chao`) | código morto: o chão desenha-se PRIMEIRO sem escrever profundidade e as peças pintam por cima — a mutação sobreviveu; saiu |

## §4 — Gates e mutações

Todos verdes com `PH2D_GPU=1`. `ph2d-mesh-forward`: `a_lei_do_chao_que_tapa_e_o_integral` (CPU) ·
`o_reflexo_e_o_lobo` (CPU, `--release` recomendado) · `o_chao_tapa_as_pecas_como_no_cycles` ·
`o_reflexo_do_chao_e_o_do_cycles` (3 materiais; a razão afirma, a absoluta imprime-se) ·
`o_chao_visto_de_baixo_e_o_do_cycles` (`tests_chao_tapa_baixo.rs`, fixtura `oraculo_chao_tapa_baixo.csv`) ·
`a_memoria_do_pixel_nao_troca_respostas`
(`tests_passe_chao_tapa.rs`, `65 536` px) · instrumento `instrumento_custo_chao_tapa`. `ph2d-app-field3d`:
`o_chao_que_tapa_chega_ao_quadro_do_render`.

**Mutações: `11` (agente `mutacao`), `10` VERMELHAS** — a lei desligada na irradiância · o reflexo desligado ·
`chao_tapa_ligado = false` (também vermelha na costura do app) · o ponto de bater no chão `p − d` · taps
ascendentes sem peso · anel `2` com ângulo `0` · união de intervalo único em `junta` (a asserção da barriga)
· o factor `baixo` da CPU removido · o ângulo do anel `1·atan` · app com `chao: None`. ⛔ **Sobreviveu a #11:**
a memória do reflexo nunca invalidada — contra o Cycles a linha do verniz não a viu (`0,0147 → 0,0154`).
Escreveu-se o gate dedicado `a_memoria_do_pixel_nao_troca_respostas`: memória do reflexo partida ⇒ a 2.ª
resposta acerta `0/65 536` ⇒ VERMELHO; memória do difuso partida ⇒ `0/65 536` ⇒ VERMELHO.

**Suítes** (verificador, load `3,8–8,3`): `ph2d-mesh-forward` `32/32` com placa · `ph2d-contacto` `2/2` ·
`field3d` família Render `38/39` em série (o `1` é `sonda_oclusao_da_malha`, que pede `PH2D_SONDA_DIR` —
sonda, não gate) · `field3d` lib `513` + `6` que só falham em paralelo e passam sozinhos: **cinco** da costura
do Render (desenhista GLOBAL; o handoff anterior dizia três) e
`layout::every_still_viewport_settles_not_only_the_active_one` (relógio de `3 s` sob load `8`) · clippy
`-D warnings` limpo · fmt limpo.

**Depois do report do dono** (`ceu_do_chao_tapado`; load `~2`): de baixo difusa `0,0133` (máx `0,098`), na
base `0,0107` (máx `0,074`; sem a lei `0,6748`), espelho `0,0027` / base `0,0055` — com o chão debaixo das
peças lido aceso `0,1306` / base `0,3701` (VERMELHO). De cima: difusa `0,0095` / `0,0125` / máx `0,066` (era
`0,0098` / `0,0130` / `0,074`), viés da barriga `+0,0064`; reflexo (razão) nítido `0,0030` / `0,0083`, áspero
`0,0206` / `0,0304` (era `0,0155` / `0,0268`: a direcção que vai à base de uma vizinha conta duas vezes na
nossa razão e uma no Cycles), verniz `0,0195` / `0,0298`. Suíte `ph2d-mesh-forward` `33/33` com placa;
costura do app `3 460` px `> 5 %`, `× 0,472`, nenhum clareia.

**Depois do report 2** (a zona da peça no reflexo): de cima, a comparação com o Cycles só na zona (fora
dela o Cycles mostra as vizinhas e as sombras delas): nítido `0,0182` / onde o chão escurece `0,0550` (sem
a lei `0,1045`), áspero `0,0323` / `0,0652` (sem `0,0883`), verniz `0,0320` / `0,0657`; de baixo espelho
`0,0097`, base `0,0081`. Difusa inalterada. Suíte `33/33`; costura do app `3 223` px `> 5 %`, `× 0,472`.

## §5 — ABERTO

- **Contagem dupla** onde um vizinho está sobre o seu próprio chão escuro (máx `0,066`).
- ⭐ **O cromo não reflete as OUTRAS peças** (nem, desde o report 2, as sombras delas): só céu, chão e a
  sombra da própria peça. O PRÓXIMO item recomendado ao dono: **capturas de reflexo** por peça brilhante
  (o cubo 360° que o Fortnite móvel usa — 6 faces pequenas com o mesmo desenhista, refeitas só quando a
  chave muda, pré-filtradas por rugosidade; cabe no WebGL2). Elas trazem as vizinhas E as sombras delas
  de uma vez, e a máscara `sombra_propria` sai.
- A zona `1,5–3` raios é medida pela forma fechada da esfera pousada (`(1/√10)³ ≈ 3 %` aos `3`), não por
  oráculo de produto; numa cena compacta a sombra de uma vizinha DENTRO da zona ainda aparece.
- O resíduo do reflexo áspero é o pré-filtro `n=v=r` (o alongamento rasante não está modelado).
- A direcção de `ceu_de_baixo` é INVISÍVEL ao oráculo de céu uniforme (`L_baixo = L` em todo o lado) —
  declarado; um oráculo de céu em rampa a gateava.
- A radiância livre do chão no estúdio é a metade de baixo do estúdio (autorada) modulada por `V`; sob o
  HDRI fotográfico a metade de baixo é o chão fotografado e só a parte do CÉU da sombra do chão entra (a
  sombra do sol sobre o chão não está em `V`).
- Custo `+0,30 ms` no desktop; celular sem medida.
- Fixtura `~1,8 MB`.
- Mais de DOIS intervalos disjuntos por fatia fundem-se no mais próximo.
- As `5` falhas da costura que só aparecem em paralelo.
- Herdados: §5 do [O_CONTACTO](HANDOFF_line_3DModeling_O_CONTACTO_2026-10-03.md#5--aberto) (menos o «o chão não tapa as peças»).

## §6 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=41 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**. A câmara abre de LADO, rente ao chão (o chão fica de perfil e
   não se vê): arraste um pouco para **cima**, uns 5 a 10 graus.
3. Deve acontecer: a bola branca grande escurece por baixo, mais forte onde encosta no chão; a bola de
   cromo mostra no reflexo uma mancha escura debaixo dela (o chão à sombra dela); a bola pequena que flutua
   escurece por baixo menos que a pousada; a caixa escurece ao pé das faces.
4. Deu errado se: alguma peça fica mais **clara** por baixo do que estava, aparece um anel ou degrau na
   barriga das bolas, ou a mancha no reflexo do cromo fica num sítio que não é debaixo dela.

Foto do app (`fotografa_cena.sh "PH2D_FIELD_SMOKE=41 PH2D_FIELD_SHADING=render" 1930 1040`): a câmara
de fábrica é LATERAL (o chão de perfil) — o efeito não se vê sem girar; ⛔ a 1.ª redacção do roteiro dizia
«gire para BAIXO», corrigida. Sonda (o antes/depois): `PH2D_GPU=1 PH2D_SONDA_CENAS=41 PH2D_SONDA_PITCH=0.1
[PH2D_SONDA_SEM_CHAO=1] … sonda_do_render_por_malha` — com o chão o cromo reflete o chão escuro debaixo
dele e das vizinhas, e a bola grande fecha escura onde encosta.

## §7 — Reports do dono

- ⛔ **Report 1 (03/10)**, 3 fotos de cima e de baixo: *«de forma muito bizarra o reflexo da esfera reflete a
  base oculta dos outros objetos (a parte colada ao chão)»* — manchas cinzentas no cromo, uma cruz clara na base
  da bola branca vista de baixo, um retângulo claro na base da caixa. Mecanismo: a lei lia o chão DENTRO da
  pegada de uma peça pousada como aceso (§3). Gate red-first `o_chao_visto_de_baixo_e_o_do_cycles` (base
  `0,3701 → 0,0107`). Fotos da sonda de cima (`PITCH 0.25`) e de baixo (`−0.35`): base da caixa preta inteira,
  sem cruz; no cromo a mancha virou a sombra escura da bola pequena (§5).
- ⛔ **Report 2 (03/10)**, 1 foto: *«esses reflexos não fazem sentido. Reflete a sombra do objeto mas não o
  objeto. Contudo, o objeto está sobre a sombra»* — as manchas escuras no cromo eram as sombras das vizinhas
  sem as vizinhas. Cura: a zona da peça (§3). Foto da sonda (`PITCH 0.25`): o cromo sem manchas, com o
  escurecimento suave do contacto e a sombra dele em baixo.
- pendente: o smoke do dono depois das duas curas.

## §8 — A PRÓXIMA ONDA

Candidatos sem ordem: (a) retirar o Render traçado (decisão de produto — perguntar ao dono) · (c) mobile
real: o quadro, o passe do céu do chão, o contacto e agora o chão que tapa (`+0,30 ms` aqui) · (d) dentes
de 2–3 px na quina côncava (§5-b do [O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md)) ·
(e) LOD. O candidato (b) fecha com esta onda.
