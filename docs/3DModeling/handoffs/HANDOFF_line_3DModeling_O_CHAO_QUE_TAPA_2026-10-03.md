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
| bindings / pipelines | **nenhum novo** (o céu do chão já está na `15`); `Objeto`/`Quadro` uniformes iguais |
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
| Texels inválidos do céu do chão lidos como `0` | ficam `1` (como a leitura do próprio chão); sob uma peça pousada essas direcções atravessam a peça, e o contacto já as conta |

## §4 — Gates e mutações

Todos verdes com `PH2D_GPU=1`. `ph2d-mesh-forward`: `a_lei_do_chao_que_tapa_e_o_integral` (CPU) ·
`o_reflexo_e_o_lobo` (CPU, `--release` recomendado) · `o_chao_tapa_as_pecas_como_no_cycles` ·
`o_reflexo_do_chao_e_o_do_cycles` (3 materiais) · `a_memoria_do_pixel_nao_troca_respostas`
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

## §5 — ABERTO

- **Contagem dupla** onde um vizinho está sobre o seu próprio chão escuro (máx `0,074`).
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

- pendente (smoke enviado ao dono 03/10).

## §8 — A PRÓXIMA ONDA

Candidatos sem ordem: (a) retirar o Render traçado (decisão de produto — perguntar ao dono) · (c) mobile
real: o quadro, o passe do céu do chão, o contacto e agora o chão que tapa (`+0,30 ms` aqui) · (d) dentes
de 2–3 px na quina côncava (§5-b do [O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md)) ·
(e) LOD. O candidato (b) fecha com esta onda.
