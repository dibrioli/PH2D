# Handoff de continuação — Luz 2D, W5c: o rebote e a cascata holográfica (2026-10-10)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · rebase sobre `main` `782f90b69` no início da onda (76/76
  limpo; três conflitos SEMÂNTICOS curados — abaixo).
- **HEAD:** o commit deste doc. Commits da onda: `d361db9ec` · `ea0ecc7dc` (o rebase) · `5272ec4fa` (a lei no plano;
  o HRC descrito do artigo) · `f4072d3ba` (o rebote no gabarito) · `ec6a010af` (o rebote na placa; a fusão junto à
  face; o G4) · `d1c8966a5` (a C2 sai do `solver.rs`) · `1a131e4dc` (a holográfica) · `1697d6d41` · `e157fb1e3` (a
  holográfica em todo nível; o Ultra sai) · `ba46b8245` · `aa35e0064` · `fe8768edf` · `01e5527a5` · (fecho).
- **Estado:** W5c FECHADA (onda). Próxima: **W5d — janela NOVA** (governador acima de 1080p — agora com a MEMÓRIA da
  holográfica —, Flip receber/bloquear, formas do Motion bloquearem, a rodada desktop S1–S8 com o Godot, o pior caso
  da A3 da W5b, e os ABERTOS abaixo — o 1.º é o rebote de fora da grade).
- **Delta da shell: −5** (a publicação da luz no fim do quadro foi para a família — o portão). Fundação (ADR-0107, acréscimo): `ph2d-editor-core/hier_rows.rs` (`ComponentCategory::Light
  ⇒ IconId::Light`) e `settings_luz_tests.rs` (o rádio conta `ALL.len()`), `ph2d-i18n/component_keywords.rs`
  (`component.bloqueia_luz.keywords`, `component.nao_recebe_luz.keywords`), `ph2d-panel-inspector/reveal.rs`
  (`SECTION_COMPONENT` 46 → **47**: `INSP_LIVE_LUZ_SECTION → ph2d::light2d::Luz`). Partilhados: `.config/nextest.toml`
  (a faixa dos flakes +1: `g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes`) e `FLAKES_DE_CARGA.md` (+1
  linha). Contratos congelados: nenhum. `PROJECT_SCHEMA`: sem degrau (nenhum componente mudou de forma: o albedo
  vem do tint do sprite e da cor do caminho).

## O rebase

Três conflitos SEMÂNTICOS do `main` (ObjectsAndComponents F4/onda 6), curados por acréscimo (acima): o `match` de
`component_icon` sem a `Light`, as palavras de intenção da paleta e a tabela ícone→cartão do Inspector.

## O que fechou

| peça | o desenho | portões |
|---|---|---|
| **A lei do rebote** (plano §2.4) | reflete o corpo OPACO que RECEBE luz; `L_r(x) = min(a, A_MAX)·luz_em(x⁺)` — o que a câmera mostra ali é o que volta; `A_MAX = 0,9` (`ph2d_light2d_ecs::ALBEDO_MAX`: o recurso é a convergência) | — |
| **O rebote no gabarito** (`ph2d-light2d-ref/rebote.rs`) | elementos de fronteira de radiância constante; os extremos são ângulos-evento ⇒ cada linha (direto + pesos ângulo × transmissão) é exata; a série é matriz × vetor | `g9_o_rebote.rs` (7): a fórmula FECHADA do meio céu (`L_p = a·L/(2 − a)`, ≤ 1e-6), a sala vermelha/verde, o passo `h`, a série e o teto, G10, o que não reflete |
| **Quem reflete** (a coleta) | o albedo `tint∘self_tint` no `Oclusor`/`SpriteDeLuz`; `VetorDeLuz::reflete` (a cor do caminho vem da camada do Vello, sRGB → linear) | `quem_reflete.rs` (2) |
| **O rebote na placa** | a C0 escreve o ALBEDO (4.º alvo); o passe `C0 rebote` soma `albedo × S` à emissão, com a fonte `S` do quadro ANTERIOR lida no MUNDO (`escala`/`desloc`); a `c3_rebote` mede `S` = a luz na face (`x⁺` pela bissecção, as cascatas + a luz direta) numa faixa à volta das bordas; sem nenhum corpo que reflita os dois passes saltam | `g9_rebote.rs` (G9 por degrau; o pan 0,00000; a oitava 0,059) |
| **A fusão junto a uma face** (o ABERTO 1, no RC clássico) | `MascaraDaFusao::PeloFim` (sem os raios de uma sonda de cima DENTRO de um opaco cujo fim discorda do fim do verdadeiro) + `fronteira_na_face` (o zero inteiro só com a anterior E o início dentro; o raio de baixo amostra o próprio fim junto à face) | `a_luz_junto_a_uma_face_e_a_do_gabarito` (vermelho com a fusão antiga: 0,43); G3 apertado |
| **A cascata holográfica** (HRC, `c2_hrc.wgsl` · `solver_hrc.rs` · `hrc.rs`; do artigo, `pesquisa/research_notes/hrc_para_implementar.md`) | 4 quadrantes por rotação; T0–T2 traçados na lei do meio (entradas em (A, B]); fusões Eq. 14–20; o céu como o R do topo; o filtro em cruz (Eq. 21); a MESMA c0 das cascatas. Nossas leituras, medidas: partir de dentro de um translúcido conta como entrada; o «+1» da Fig. 12 só quando a sonda adiante não cai num opaco; a sonda dentro de um opaco guarda a luz que a rodeia; as sondas além da grade não se guardam | `holografica.rs` (7): bate a clássica na penumbra, junto à face, G9, pan, enxame denso, K3; G5 e G10 com ela |
| **A decisão** (o «Ultra») | a holográfica é a cascata do PRODUTO em todo nível (`DEGRAU_PADRAO.holografica`); o RC clássico fica (`holografica: None`) como referência e alternativa; o `NivelDaLuz` continua de três | a rodada (abaixo) |
| **G4 (K3)** | a causa medida: a entrada num emissor relia a emissão e calculava a queda de alcance infinito (vale 1) | 4,5–5,1 % → 0,9–1,1 % (o RC); 1,9 % (a holográfica) |
| **A cena 3 «Rebote»** | sala fechada, albedos de sala real (0,75), parede vermelha e verde, fundo que só recebe luz; `PH2D_LIGHT2D_REBOTE=0` desliga | `a_cena_3_ensina_o_rebote` (gabarito: o meio 0,175 → 0,474; R/G 1,46 · 0,80) |

## Antes → depois (a rodada `medida_luz2d_w5c`)

Rodada `medida_luz2d_w5c` (RTX 5060 Ti, `smoke`, 1920×1080, vista de fábrica, `load` 0,6; o MÍNIMO de 7 rodadas
intercaladas; variantes como configuração em execução):

**A. Tempo de placa (ms; cenas 1 · 2 · 3 · 4) e memória**

| nível | W5b (RC antigo, sem rebote) | RC da W5c + rebote | holográfica + rebote (o produto) | MB (W5b → produto) |
|---|---|---|---|---|
| Baixo | 2,61 · 2,74 · 2,35 · 3,76 | 2,78 · 2,87 · 2,73 · 3,91 | **2,52 · 1,83 · 2,08 · 3,15** | 105,7 → 147,5 |
| Médio | 6,27 · 8,48 · 7,33 · 7,99 | 6,97 · 9,08 · 8,20 · 8,60 | **5,76 · 5,14 · 5,33 · 6,22** | 133,7 → 308,6 |
| Alto | 19,61 · 28,18 · 23,79 · 24,65 | 22,38 · 31,03 · 26,81 · 27,45 | **24,78 · 24,52 · 25,32 · 26,57** | 200,2 → 893,6 |

O rebote custa 0,13–0,19 ms (a fonte, só com refletor; a soma ≤ 0,02). A fusão nova do RC custou +5 % (Baixo) a
+13 % (Alto) — a holográfica tira-a do caminho do produto. K3 (G4): RC 4,5–5,1 % → 0,9–1,1 %; holográfica 1,9 %
(as cascatas 1,10 ms contra 4,6 do RC no Alto de 512²).

**B. Convergência do rebote (cena 3):** 5 % no quadro 8 · 1 % no quadro 11–12 (os três níveis).

**C. Erro contra o gabarito (o pior ponto, relativo à luz local)**

| | Baixo | Médio | Alto |
|---|---|---|---|
| cena 3 (rebote) · RC da W5c | 8,7 % | 5,0 % | 15,9 % |
| cena 3 (rebote) · holográfica | **1,5 %** | **1,1 %** | **0,8 %** |
| vaga-lumes (cena 2) · RC · holográfica | −2,8 · −2,7 % | −2,8 · −0,9 % | −1,0 · −0,8 % |
| sala do G9 (placa, 512²) · RC · holográfica | 13 · 0,7 % | 7,4 · 1,0 % | 4,4 · 0,9 % |
| junto a uma face (pior de 6 alinhamentos; longe · próprio) · RC W5b → W5c → holográfica | 0,33 · 0,22 → 0,05 · 0,22 → **0,011 · 0,009** | 0,37 · 0,18 → 0,03 · 0,15 → **0,007 · 0,005** | 0,43 · 0,21 → 0,02 · 0,14 → **0,003 · 0,002** |

G3: céu e vidro 0,0121 · 0,0198 · 0,0374 → 0,0074 · 0,0087 · 0,0117 (RC W5c); a penumbra pelas cascatas no Alto
RC 0,1175 × holográfica **0,0757** (a Fig. 8 do artigo: 0,0354 × 0,0184).

**D. O rebote de fora da grade:** numa vista de 2 m de altura da cena 3, o ponto junto à borda esquerda sai
**−55 a −64 %** do gabarito (o chão, o teto e as paredes fora da grade não refletem) — ABERTO 1.

Fotos (`fotos_W5c/`): `cena{1,2,3,4}_{low,medium,high}.png` e `cena3_{low,high}_sem_rebote.png` (o antes × depois do
rebote); as cenas 1, 2 e 4 contra as da W5b: iguais a olho (as paredes delas não recebem luz — nada reflete).

## ⛔ Recusas MEDIDAS (W5c)

| tentativa | número | por quê |
|---|---|---|
| a máscara da fusão PELA SONDA (só as sondas de cima de fora) | a luz de longe junto à face +7 a +11 %; o termo próprio de uma face que brilha −8 a −19 % | tira os raios que acertam a face e recolhem a radiância dela |
| a máscara PELO FIM sem a restrição à sonda de dentro | o anel do G3 0,0393 → 0,0485 | tira a mistura de acertos e falhas na silhueta de uma luz pequena (a correção bilinear) |
| a fronteira no fim INTEIRA (todo raio amostra o seu fim) | o anel 0,049 | reforça o padrão de uma luz pequena (o viés quase não muda: −0,5 → −0,2 %) |
| o zero pelo INÍCIO (sem a anterior) | o termo próprio −49 % (Baixo) | perde a entrada na fronteira |
| o passo ½ nas cascatas para o rebote no Baixo | 0,121 × 0,122 | o erro não é do passo |
| o RECUO da fonte do rebote (1 · 2 · 3 texels para fora) | alinhada −5 → −12 %; desalinhada −27 → −11 % | converge a −11 % em toda face; saiu |
| o «+1» do artigo cego | o texel colado à face −43 % | a sonda adiante cai dentro do opaco |
| sem o «+1» | cruzes nas diagonais de uma luz pequena +17 a +30 % (a Fig. 12) | ⇒ o «+1» guardado |
| sem o filtro em cruz | a penumbra 0,0757 → 0,1447 | o xadrez das sondas de v par e ímpar |
| o passo 1 nos traços curtos | a mesma régua a −6 % de custo | fica ½ pela fresta fina |
| o Ultra como NÍVEL próprio | a holográfica é mais exata em todo nível e mais barata no Baixo e no Médio | ⇒ em todo nível |
| a cena 3 com albedos 0,9 e uma caixa | a sala satura a branco (×8); a caixa em cunhas pretas (o eixo medial) | ⇒ 0,75, sem caixa |

## Prova de mutação

Agente `mutacao`: pré-voo das 27 âncoras (1× cada), filtros povoados, restauro `checkout` + `touch shaders.rs`,
árvore = HEAD no fim. **24 de 27 sangram** à primeira; os 2 controlos equivalentes (o passo do traço 0,5 → 0,45;
`FORA` 1e-9 → 2e-9) ficam verdes. Sobreviventes: **R2** (o lado do `x⁺` no gabarito) é EQUIVALENTE — o
`fora_dos_opacos` leva o ponto de dentro ao mesmo exterior; **P5** (o translúcido com albedo na placa), **P9** (a
queda de alcance finito nas cascatas — lacuna anterior à onda) e **P17** (a contagem de sondas numa grade que não é
potência de 2) ganharam portão (`o_translucido_nao_reflete_na_placa`, `g3_a_queda_de_alcance_finito_nas_cascatas`,
`a_holografica_numa_grade_que_nao_e_potencia_de_dois`), cada um visto a sangrar. A cura das pintas pretas viu o
vermelho à mão (0,21 contra 0,40).

| sangram | o quê |
|---|---|
| R1 R3 R4 R5 | o teto do albedo · a transmissão até o refletor · o núcleo 1/2π · os extremos dos elementos (gabarito) |
| E1 E2 E3 | o translúcido · o `NaoRecebeLuz` · o vetor (a coleta) |
| P1 P16 | a fonte lida no índice · o desloc invertido (o pan: 0,86) |
| P2 P3 P4 P15 | o teto na placa · a fonte no centro do texel · sem a luz direta · os passes sempre saltados (G9) |
| P6 P7 P8 | a máscara pelo fim · o zero pelos dois · o fim inclusivo (junto à face) |
| P10–P14 | o «+1» cego · o fator de partida · o filtro em cruz · a média dos caminhos tortos · o arco (holográfica) |

## Portão de fecho

1.ª corrida (sobre `d657ccdd9`): fmt ✓ · clippy `-D warnings` (`ph2d-light2d-gpu`, `ph2d-app-light2d`,
`ph2d-light2d-ref`, `ph2d-light2d-ecs`, `ph2d-editor-core`, `ph2d-i18n`, `ph2d-panel-inspector`, `ph2d-host-desktop`) ✓ ·
machete ✓ · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ · `nextest-impacted` **17 133 / 17 134** —
`fn_loc_caps::shell_functions_respect_their_ceiling`: a `fase_snapshots_publish` da shell a 202 LOC (teto 200) — a
soma da W4 desta linha com o `main` (veio no rebase). Curado MOVENDO a publicação da luz para a família
(`LuzDoQuadro::publicar`, `inspector.rs`): a shell **−5** (`54eafad84`). 2.ª corrida, sobre `54eafad84`: fmt ✓ ·
clippy ✓ · machete ✓ · deny-check ✓ · `nextest-impacted` **17 134 / 17 134** ✓ (`load` ~1). Suítes da família:
placa 34 + 19 ignorados (sondas) · lib 6 · família 47 + 16 · gabarito 27 + 2 · ECS 14.

`agent-loop-profile.sh` (20 sessões): paralelismo 1,11/passo ✗ · respostas por sessão 192 ✓ · test:check 3,8 ✗ ·
`Edit` 31 % ✗ · contexto relido 432 mil ✗ · contexto no início 62 mil ✓.

Smoke recompilado (regra I, depois do `rm -rf target/*/incremental`), 2.ª saída: `Finished \`smoke\` profile [optimized] target(s) in 0.20s`.

## Depois do smoke do dono (10/10): o interior em escada

**A queixa** (três fotos): a parede verde da cena 3 esticada e arrastada — as faces do interior (a lei F3: o ponto de
dentro recebe a luz do exterior mais próximo, cortada no eixo medial) mudam de desenho com a posição: a diagonal entre
duas faces em escada, e num sítio um risco vertical a mais.

**A causa, medida** (`o_interior_do_opaco.rs`): a luz do interior está CERTA ponto a ponto (contra o gabarito nos
centros dos texels, longe do eixo, ≤ 3 % em 8 deslocamentos × 2 zooms × 3 níveis, caminho do polígono = caminho do
sprite). O defeito é de AMOSTRAGEM: o texel fino guardava a lei no CENTRO; um corte seco em diagonal amostrado numa
grade de 1–2 px é uma escada que anda com a posição, e com o eixo sobre uma coluna de centros a coluna inteira empata e
fica com a cor de uma face só (o risco).

**O desenho** (`c3_direta.wgsl`, `Config::media_no_eixo`, por omissão ligado): o texel que o eixo corta guarda a MÉDIA
da lei sobre a área dele — a face mais perto e a outra, na fração do texel além do eixo (`area_alem`: a área exata
de um quadrado além de uma reta; o eixo é onde as distâncias se igualam e o gradiente da diferença é `n₁ − n₂`). As
duas distâncias medem-se pela PERPENDICULAR (`pe_na_face`: a tangente por dois cruzamentos paralelos; o ponto de
fronteira cruzava a face na oblíqua e a bissecção curta errava décimos de texel — a fração vem da diferença das
duas); a luz de cada face continua no ponto de fronteira dela. Outra face = direção a > 60° E normal a > 60° (um
ponto da mesma face visto de lado afasta-se em direção, não em normal); refino que falha = a mesma face. Longe do
eixo o texel é o de sempre. Junto à borda, se a janela 5×5 não alcança uma segunda face, entram as sementes do JFA
de OUTRA face ao alcance (os cantos).

**Gate** `o_interior_do_opaco_nao_depende_da_posicao_na_grade` (a caixa livre na sala, 4 deslocamentos × zoom 1 e
2,5, a configuração do produto; o gabarito é a média sobre o texel, 12×12 junto ao eixo): texels errados > 10 % — o
produto 0–1 por caso (um canto), o controlo sem a média 35–537 (teto 3; o controlo tem de passar 30). Visto vermelho
antes da cura (pior 0,23–0,48). Fotos: `fotos_W5c/interior_antes_depois.png`.

**A régua do F3 mudou** (`c3_grade_fina.rs`): o gabarito do interior passa a ser a média da lei sobre a parte de DENTRO
do texel (`media_no_texel`: 8×8 onde a média dos cantos se afasta do centro — um corte; a parte de fora é a borda do
receptor, que o interior nunca guardou). Com a régua de ponto a média no eixo lia-se como erro. Os tetos re-assentes no
medido + 10 % e o gate exige também vencer o controlo sem a média: fundo 0,001281 · 0,000043 · 0,000337 (sem 0,002545 ·
0,000311 · 0,000536); faixa 0,3207 · 0,1136 · 0,0063 (sem 0,3664 · 0,2370 · 0,0522). A faísca na faixa é o canto do
quadrado girado (a régua de ponto dava 0,0032 porque o centro É a lei do centro).

**Custo** (`sonda_custo_da_media_no_eixo`, `c3_fina`, mínimo de 7 intercaladas, 1080p, três corridas iguais): cena 1
0,74 → 0,75 ms · cena 2 0,26 → 0,37 · cena 3 0,33 → 0,37 · cena 4 1,33 → 1,40; a caixa no zoom 2,5 3,06 → 3,10 no
melhor (as corridas deram até 4,17 com a placa a 96 % de uso por outro processo — releia na W5d com a placa livre). O que custa é a luz na outra
face (a visibilidade marcha por lâmpada) e, nos vaga-lumes, que todo o interior de um corpo minúsculo está junto ao
eixo.

⛔ **Recusas medidas:** guardar o centro (a escada; o controlo); mistura linear em vez da área exata (45°: erra 1/8 da
diferença das faces); a distância pelo ponto de fronteira oblíquo (pior 6–22 %); a luz no PÉ da perpendicular e uma
TERCEIRA face (o encontro de três) — a 1.ª prova de mutação mostrou-as mudas em todo gate e custavam: cena 3 0,57 →
0,37 ms ao tirá-las; sem a condição da normal a faixa da penumbra ia a 0,43 (pior que o controlo); refinar também
longe do eixo (custo sem ganho); a régua do gate com 6×6 (degraus de 1/6 = 5 % da diferença das faces); o refino que
falha a devolver a direção oblíqua como normal (um ponto da MESMA face lia-se como outra: −0,0002 junto às faces).

**Prova de mutação** (agente `mutacao`, duas rodadas; os 4 gates: `o_interior_do_opaco`, F3 de `c3_grade_fina`,
`junto_a_face` e a das faces da holográfica): a 1.ª, sobre o desenho com três faces e a luz no pé, deu 8/12 — as
sobreviventes eram a 3.ª face, a luz no pé e duas guardas redundantes: saíram (acima). A 2.ª, sobre o código final:
**11/12** — `media_no_eixo` falso, as duas fórmulas da área, `COS_OUTRA_FACE`, a condição da normal, o JFA dos
cantos, a distância perpendicular, a escolha da face mais perto, a luz da outra face, o passo lateral do refino e o
filtro das sementes sangram. ⚖️ Equivalente: M6, o refino que falha devolver a distância em vez de −1 — com a
condição da normal o par só passa se as direções já se afastam, e então mistura dois pontos da mesma face de luz
quase igual; nenhum gate o distingue.

## 2.º smoke do dono (10/10): «até o zoom muda o render»

**A queixa** (duas fotos, a mesma caixa verde junto à lâmpada em dois zooms): a face da caixa na sombra e a sombra ao
lado clareavam ao afastar o zoom.

**A causa, medida** (`o_rebote_nao_depende_da_oitava`, a caixa logo à direita e abaixo da lâmpada da cena 3): sem rebote
a luz é a mesma em toda vista; com ele, na oitava mais grossa (texel 0,031 m) a face 1,653 × gabarito 0,269 (+514 %),
a sombra +204 %, o fundo +61 %; na fina (0,016 m) ≤ 1 %. Com a caixa sem albedo as duas oitavas batem — o excesso é a
luz que a própria caixa devolve. A cadeia: a borda direita caía DENTRO de uma célula das cascatas (cobertura 0,50); o
ponto logo fora da borda pela captura fina (o `meio`) tinha as QUATRO sondas vizinhas dentro do opaco (a borda da
captura e a das sondas diferem por centésimos de célula, conforme a posição — o mesmo desvio do ABERTO 5); o `c0_em`,
sem nenhuma sonda de fora, voltava às de DENTRO (o lixo do interior: 2,107 = 2π × 0,335) — a fonte do rebote da face
inteira 0,335 em vez de 0,06, a emissão da face 6×, e o interior remapeado lia o mesmo (as listras verticais).

**A cura** (`c0_em`, `c3_direta.wgsl`): sem sonda de fora nas quatro, as de fora do anel seguinte (4×4, tenda de raio 2);
só sem nenhuma o bilinear cru. Gate: 4 zooms em duas passagens de oitava, o pior 2,9 % (teto 3,5 %), visto vermelho
(+514 %). Mutações (2/2 sangram): sem o anel (+514 %); o anel sem a máscara de fora (+424 %). Foto:
`fotos_W5c/oitava_antes_depois.png` (a luz em volta da caixa; em cima antes — grossa · fina —, em baixo depois).

**A mesma família:** a fusão das cascatas CLÁSSICAS (`pesos_da_fusao`, `c2_rc.wgsl`) também volta aos pesos sem máscara
quando as quatro sondas de cima estão mascaradas — não é o caminho do produto (a holográfica) e a clássica erra o
rebote desta cena por outro lado (a face 0,071, −74 %); fica anotado no item 7 abaixo.

## ⚠️ ABERTO (W5c → W5d)

1. **O rebote de FORA da grade (o maior):** com zoom forte o rebote enfraquece — na vista de 2 m da cena 3, −55 a
   −64 % junto à borda, porque os refletores fora da grade não refletem (o mundo fora da grade da W5b só tapa a luz
   direta). Viola «o mesmo em qualquer vista» para salas maiores que a vista. Candidatas: um 2.º nível de grade, mais
   largo e grosso, só para o rebote (as cascatas holográficas são baratas no grosso: 1,8 ms no Baixo), ou um ambiente
   por direção à volta da grade (o R do topo da holográfica aceita-o — nota 1 do artigo; mas sem paralaxe).
1b. **K9 (≤ 10 quadros):** 5 % no quadro 8, 1 % no 11–12 na cena 3 (sala de albedo 0,75); numa sala de albedo 0,9 o
   resíduo cai a ~0,8 por rebote (5 % só no ~14). Candidata: dois rebotes por quadro (+1 C2).
2. **A luz pequena na holográfica:** raio < ~3 células: −20 % de viés no raio 1,5, +5 % no 3 (`sonda_hrc_pelo_tamanho_da_luz`);
   os vaga-lumes da cena 2 (~2 células no Baixo) saem a −2,7 %. Sprites que brilham menores ficam mais escuros.
3. **A memória da holográfica** a 1080p: 148 · 309 · 894 MB (Baixo · Médio · Alto); acima de 1080p cresce com a
   área — o governador da W5d tem de a ler (hoje só lê o tempo).
4. **A queda de alcance finito na holográfica:** a queda conta-se do início de cada segmento fundido (a composição
   dos traços não carrega a distância ao recetor) — um corpo que brilha com 0,8 m de alcance: RMSE 4,21 (o RC 0,36).
   No produto só toca lâmpadas de alcance finito além das 7 diretas (os sprites que brilham têm alcance infinito).
5. **O interior de uma parede que RECEBE luz** (ABERTO 2 da W5a) segue: a metade de fora das paredes da cena 3 fica
   escura (a face mais próxima, o corte no eixo medial). O corte agora é antisserrilhado (secção acima); sobram ~0,1
   de fração numa diagonal em certas posições — a borda do `meio` (cobertura bilinear ≥ ½) desvia-se até ~0,08 texel
   da captura conforme a posição dentro do texel — e um texel num canto.
6. **O vetor que reflete:** a cor da camada do Vello entra como albedo (sRGB → linear), mas o gabarito não vê vetores
   e não há ainda um portão de paridade sprite × vetor do rebote (o G7 do rebote).
7. **O RC clássico** (fica como alternativa) depende do alinhamento das faces junto a elas: o rebote da cena 3 5,5 ·
   24 · 39 % — não o religue sem ler isto.
   E a fusão dele volta aos pesos SEM máscara quando as quatro sondas de cima estão mascaradas (a mesma família da
   cura do `c0_em` do 2.º smoke); na caixa junto à lâmpada a face sai 0,071 × 0,269.
8. Os da W5b que continuam: a A3 (muitos objetos fora da grade sem nenhum a tapar), dois translúcidos sobrepostos
   fora da grade, o `a_nitidez_da_luz` sob carga.

## O que a W5d precisa saber

- `Config::holografica: Option<Holografica>` (o `DEGRAU_PADRAO` liga-a); `Config::rebote`, `mascara_da_fusao`,
  `fronteira_na_face` são configuração em execução — a rodada compara-os no mesmo processo.
- A holográfica escreve a c0 num passe só (os carimbos da cascata 0; os outros vazios): `TemposMs::c2_total()` é
  ela. Os buffers T/R vivem na `PassList` (`hrc::buffers`), o G5 conta-os.
- O rebote: `Solver::fonte_do_rebote()` expõe a fonte; o primeiro quadro não tem rebote; a fonte lê-se no mundo
  (pan e oitava); sem refletor os passes saltam e a fonte esquece-se.
- ⛔ Duas armadilhas da onda: (1) o `Edit`/script que «não casa» depois de um `cargo fmt` — duas vezes a rodada
  correu a versão antiga; (2) uma sonda que não recebe a configuração que diz medir (a das faces sem a
  `fronteira_na_face`) — leu −13 % onde era −1 %.

## Smoke do dono

`PH2D_LIGHT2D_SMOKE=3` (a sala do rebote; `PH2D_LIGHT2D_REBOTE=0` para comparar) e as cenas 1, 2, 4 em Low · Medium ·
High — os passos estão no relatório ao dono. Os dois smokes seguintes (o interior em escada; o rebote com o zoom):
OK do dono, 10/10.

## Integração no main (10/10, por ordem do dono: «Pode integrar»)

- **Rebase** dos 96 commits sobre o main `b77911b67` (o main andara 246 commits desde o merge-base `782f90b69`);
  ramo de segurança `line/Luz2D-pre-rebase-2026-10-10` = `29473c3e2`. 20 arquivos tocados dos dois lados; quatro
  commits pararam, resolvidos pelos estágios do índice: `prefs.rs`/`init.rs`/`prefs_tests.rs`/`forwarding_persist.rs`
  — o `left_handed` do main e o `luz2d` da linha juntos (o formato `…left_handed={}\nlight_quality={}\nlight_auto={}`);
  `hero.rs` — o `ViewState::default()` do main (o `Default` já traz o `luz2d`); `reference_topic_code_gotchas.md` — as
  lições dos dois lados; `MEMORY.md` — a família recontada no ficheiro dela: 33. `range-diff`: 93/96 idênticos, os 3
  diferentes só na resolução. Marcadores (inclusive `|||||||`): zero em cada commit. `Cargo.lock`: só as 4 crates
  novas.
- **Números que somam:** a linha não toca nenhum — PROJECT_SCHEMA 187 (187, 13, 22), registros 106/107/107, ADR
  nenhum, contrato congelado intocado, tetos de LOC sem estouro.
- **Portão da árvore combinada** (tip `24a2967fe`): os três geradores sem diferença, staleness ✓, fmt ✓, o check do
  workspace com avisos como erro ✓, clippy `-D warnings` nas 7 crates ✓, `nextest-impacted` BASE=main **17 333 /
  17 333** (loadavg 12,5). O `foundational-integrate.sh` recusava o primário (dois ficheiros por rastrear de outra
  frente do Painter, fora de `project-memory/`): os mesmos passos correram à mão, na ordem do script, e o
  `merge --ff-only` levou o `MEMORY.md` sujo do primário por patch (hunks disjuntos dos da linha), reaplicado depois.
- CLAUDE.md §5.1 ganhou a entrada da Luz 2D (532 bytes).
