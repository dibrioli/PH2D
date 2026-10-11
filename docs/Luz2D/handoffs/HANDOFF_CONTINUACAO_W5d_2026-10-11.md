# Handoff de continuação — Luz 2D, W5d: a precisão junto à borda, a cena 6, o entorno e o K9 (2026-10-11)

> Linha `line/Luz2D` (worktree `Worktrees/line-Luz2D`), base = `main` em `8f20c60de` (nada a rebasear ao
> assumir). Antes desta onda, na linha e por integrar: a cena 5 «Volume» (`baa38fb51`) e a `Fabrica`
> (`a147378cb`, BUGS §1). Commits da W5d: `249789cc1` · `8baac89fb` · `fca6e3a0b` · `b097ac629` ·
> `7c0210c72` · `f7c29fbaa` · `8768a7155` · `b63c0cd3e` · `cda917031` (+ os de docs no fecho). Nada integrado; push e integração só por ordem do dono.

## O que fechou

| | o que o dono via | a causa MEDIDA | a cura |
|---|---|---|---|
| A0 | o disco «cone em fatias» com a aresta da sombra serrilhada; degraus na elipse | três: (1) o iso-½ da bilinear desvia-se da borda até ~0,08 texel e o raio rasante que sai da passagem luz/sombra reentrava no contorno; (2) com o rebote, as sondas da cascata COLADAS a uma superfície que reflete variam ±15 % de uma para a outra e a lei do interior projetava o ruído até ao centro; (3) num sprite a cobertura capturada era a MÉDIA do alfa (a rampa do disco do atlas não é área) | a borda EXATA no interior (o AAEDT), a cobertura do sprite pelo MEIO, a c0 do ponto de fronteira pela MEDIANA ao longo da face |
| A1 | ~0,1 de fração nas diagonais entre faces | o desvio do iso-½ (o mesmo da A0-1) | a borda exata (o pior da caixa 0,142 → 0,063) |
| A2 | um texel imperfeito num canto da caixa | idem | 1 → 0 (o teto do `o_interior_do_opaco` 3 → 1) |
| A3 | «não temos desenhos esféricos para testar» | — | a cena 6 «Formas desenhadas» (`PH2D_LIGHT2D_SMOKE=6`) |
| B1 | com zoom forte some o brilho à esquerda do «L» | o «fora da grade» — CONFIRMADO: a mesma vista com a grade esticada sobre a sala, o mesmo texel, −63,6 % → −0,5 %. Com a parede DENTRO da grade o erro era o mesmo (−58,8 %): o chão e o teto ficavam fora | o ENTORNO |
| B2 | (K9) o rebote leva 11–12 quadros a 1 % | um rebote por quadro | dois por quadro no Baixo |

⚠️ **O cone e a aresta da elipse são a LEI** (o interior é o exterior mais próximo): o gabarito com a série
do rebote também sai em cunhas. O que saiu foi o ruído e a escada em cima dela.

## O desenho

**A borda exata** (`c3_borda.wgsl`, o AAEDT de Gustavson & Strand, 2011): num passe por quadro (no
carimbo do `C3 preenche`), cada texel da captura que a C3 lê vira `(cobertura, normal para dentro,
recuo)` — a reta que deixa dentro a área = a cobertura, com a normal pelo Sobel de pesos √2 (sem
gradiente: o iso-½). O `meio` decide «dentro» por ela onde `borda_aqui` — no INTERIOR dos opacos (a lei
F3 e a visibilidade lida na fronteira) e no preenchimento e na fonte do rebote dos texels de dentro; a
luz direta dos texels de FORA marcha no iso-½ (a mesma foto, ~0,4 ms a menos na cena 4). A C3 liga a
textura nova no lugar da cobertura (o `textureGather` lê o vermelho): a C2 e a holográfica não mudam.
`Config::borda_exata` (ligada).

**A cobertura pelo meio** (`c0_sprite.wgsl`, `COBERTURA_PELO_MEIO`, com a borda exata): a cobertura de um
sprite é a fração das 8×8 subamostras com alfa ≥ ½ — a área da forma alfa ≥ ½; a borda «dentro» fica
onde estava, a cor segue ponderada pelo alfa. A camada do VETOR (lida exata, um píxel por texel) já traz
a área no alfa: `FormaSprite::alfa_e_area` mantém-lhe a média (com o limiar a cobertura dela era 0/1 —
riscos nos desenhos; a foto da cena 6 mostrou-os).

**A c0 ao longo da face** (`c0_ao_longo_da_face`, `Config::c0_ao_longo_da_face` = 2 texels das
cascatas, `Config::c0_ao_longo_mediana`): o texel do interior lê a c0 no ponto de fronteira pela MEDIANA
(por canal) de até cinco pontos ao longo da tangente — só os que continuam sobre a MESMA face (fora do
corpo, com ele um texel atrás). A mediana e não a média: junto a um canto a cascata já vaza a luz do outro
lado por ~2 texels, e a média espalhava-o mais 2 (a prova de mutação M7 mostrou o buraco). Só no que a
câmera mostra: a fonte do rebote lê no ponto.

**A cena 6** («Formas desenhadas»): um círculo e uma elipse VETORIAIS em cima e, no espelho deles em
relação à lâmpada, as IMAGENS gêmeas com as mesmas cores. A costura sem a shell crescer (zero linhas):
`LuzDoQuadro::codificar` recebe o documento editável (a mesma chamada da shell compila — reempréstimo
implícito); no 1.º quadro `smoke::semear_documento` põe os caminhos; o editor sincroniza a entidade de
cada um no fim do quadro e `smoke::marcar_semeadas` dá-lhe o nome e o «bloqueia a luz» (os que saem do
documento — um projeto aberto por cima — esquecem-se). O chão com `RootOrder(0)`: sem ordem uma raiz
desenha na FRENTE (`root_key` = `u32::MAX`) e cobria os desenhos.

**O entorno** (`Degrau::entorno` = 8, `MARGEM_DO_ENTORNO` 1,5; `PH2D_LIGHT2D_ENTORNO=0` compara): a
`LuzDoQuadro` mantém um 2.º `Solver` 8× mais grosso e 4× mais largo, sem céu, com a emissão do MIOLO (a
grade da vista) apagada (`Solver::mascarar_miolo` → o `tracar` da holográfica não a recolhe). A c0 dele é
a luz que vem de FORA (os refletores de fora recebem as lâmpadas pela luz direta DELE) e soma-se à c0 da
vista na saída da holográfica (`Solver::somar_entorno`, grupo 2 da `hrc_saida`) — chega à cena, ao
interior dos corpos e à fonte do rebote. Gravado antes, no mesmo codificador; o custo dele entra no
custo mostrado e no fixo do governador. Só com a holográfica. Sem entorno a saída liga a emissão (o
`entorno.w = 0` não a lê): uma textura vazia a mais mudava o enchimento do alocador nas vizinhas (G5).

**Dois rebotes por quadro** (`Config::rebotes_por_quadro`; o `Degrau`: 2 no Baixo, 1 no Médio e no Alto):
o 2.º repete no mesmo quadro a fonte (a C3 só do rebote, com a borda exata antes), a emissão de BASE
reposta por cópia (`luz2d/tex/emissao_base`), a soma com a fonte nova na MESMA grade (o uniforme
`luz2d/buf/rebote_mesma`) e a cascata (no carimbo da cascata 1, vazio com a holográfica). O campo de
distância não se repete.

## Antes → depois (rodada `medida_luz2d_w5d` e `medida_luz2d_w5d_rebote`, RTX 5060 Ti, perfil smoke)

**A — os polígonos (96 lados) da cena 5 contra o gabarito**, 2 zooms × 2 posições × 4 anéis (0,35 · 0,65 ·
0,9 do raio; +2,5 texels fora), sem rebote: o pior excesso de variação +1,89 → +0,31; cruzamentos a mais
numa passagem luz/sombra 4 → 0; o maior desvio dela 5,3 → 3,6 texels; pontos errados (> 10 %) 1 → 0.

**A — o disco de IMAGEM** (o sprite do atlas, pela captura do produto), zoom 1: variação +1,30 → +0,66,
cruzamentos 2 → 0, erros 2 → 0; zoom 2,5: cruzamentos 4 → 2, variação +2,38 → +3,56 (ABERTO 1).

**A — o interior com o rebote** (60 quadros, zoom 2,5): variação do anel de dentro / a de fora (disco ·
elipse): sem nada 1,014 · 1,109 · a média de 2 texels 0,979 · 0,981 · a média de 1 0,994 · 1,018 · **a
mediana de 2 (o produto) 0,987 · 1,033**.

**A — junto a um canto** (uma parede 0,2 × 1 m, o corpo que brilha em cima à esquerda; a face direita na
sombra, o gabarito 0; o valor lido a 1 · 1,5 · 2 · 3 · 4 texels das cascatas do canto, a luz acima dele
1,31): sem nada 0,66 · 0,30 · 0,15 · 0,02 · 0 · a média de 2 0,39 · 0,34 · 0,30 · 0,17 · 0,03 · a média de 1
0,60 · 0,44 · 0,24 · 0,04 · 0 · **a mediana de 2 0,26 · 0,18 · 0,11 · 0,02 · 0**.

**A — a cena 6**: as faces 46 × e 47 ×; desenho × imagem na face acesa −0,33 % · +0,06 %; a variação nos
anéis, desenho × imagem −0,3 % · +3,9 %.

**A — o custo da C3** (preenche + visibilidade + fina, 1080p, vista de fábrica, o mínimo de 7): cena 1
1,43 → 1,63 · cena 2 0,55 → 0,87 · cena 3 0,74 → 1,02 · cena 4 2,26 → 2,55 · cena 5 4,27 → 4,65 ms (a
borda exata por chamada, antes da pré-conta, custava até 11 ms na cena 5).

**B1 — o vermelho junto à borda esquerda da vista de 2 m da cena 3 contra a série** (nível Médio):

| câmera em x | sem entorno | entorno 4× | entorno 8× (o produto) |
|---|---|---|---|
| −2 (a parede dentro, o chão e o teto fora) | −58,8 % | −10,5 % | −11,3 % |
| 0 | −63,6 % | +0,5 % | −0,3 % |
| +2 | −53,7 % | −0,4 % | −0,9 % |

O preço a 1080p (vista + entorno, cena 3 · cena 5): Low 2,59 + 0,38 · 6,84 + 0,46 ms, 273 + 8 MB ·
Medium 5,83 + 0,91 · 8,92 + 0,96 ms, 434 + 34 MB · High 27,15 + 2,78 · 25,00 + 2,86 ms, 1011 + 128 MB.

**B2 — rebotes por quadro** (a cena 3, vista de fábrica, o pior ponto a 1 % do convergido; o convergido
igual a 0,00 %): Baixo 11 → 6 quadros por 2,75 → 3,72 ms · Médio 12 → 6 por 5,82 → 10,11 ms · Alto 12 → 6
por 24,2 → 73,8 ms ⇒ dois só no Baixo.

**C — a linha de base** (sem mudança): a luz pequena na holográfica −20,0 % no raio 1,5 células, +5,0 % no
3 (a clássica −4,9 · +0,9 %); a queda de alcance finito RMSE 4,21 (a clássica 0,36).

## ⛔ Recusas MEDIDAS (W5d)

| tentativa | número | por quê |
|---|---|---|
| a c0 CONTÍNUA ao longo da borda (a bilinear das de fora completada pela tenda de raio 2) | a variação dos anéis igual (+1,89 → +1,89) | o suspeito (i) do handoff W5c não era a causa |
| a TENDA de raio 2 das sondas de fora no ponto de fronteira | o G9 da holográfica caía (Médio 0,0966 > 0,082) | as sondas a 1–2 texels veem menos o próprio corpo: −4 a −9 % na face com o rebote |
| a média ao longo da face SEM o filtro «a mesma face» | o G9 da holográfica no Baixo 0,036 (teto 0,008) | nos cantos da sala a tangente passa da ponta da face e lê o outro lado |
| a MÉDIA ao longo da face (com o filtro) | junto a um canto, a 3 texels: 0,02 → 0,17 da luz do outro lado | espalha o vazamento da cascata no canto; a mediana não (0,02) |
| a borda exata em TODA a C3 (a luz direta dos texels de fora também) | a mesma foto; cena 4 2,54 → 2,95 ms | custo sem ganho |
| a borda exata por CHAMADA (o Sobel em cada `meio`) | vaga-lumes 0,37 → 3,15 ms, cena 5 4,4 → 11 ms | a pré-conta por texel dá a mesma qualidade a 0,87 · 4,65 |
| o PÉ da perpendicular em todo texel do interior (re-medido sobre a borda exata) | variação +0,32 → +0,52 | a W5c recusou-o sobre o iso-½; continua pior |
| o RECUO da visibilidade para fora da fronteira (0,1 · 0,25 · 0,5 texel) | 0,25 limpa o zoom 2,5 do disco de imagem e cria 14 erros no zoom 1 | uma distância fixa troca um zoom pelo outro |
| o SALTO na face (o raio que entra vale 0; o que sai salta o próprio corpo 1–4 texels) | os polígonos +0,32 → +2,11, 5 erros, a passagem a 11 texels | a normal estimada erra mais que a penumbra da lâmpada (1,7°) |
| a borda exata com a cobertura = a MÉDIA do alfa num sprite | o disco de imagem no zoom 2,5: variação +2,4 → +47 | a rampa do disco do atlas ampliada não é área |
| o limiar ½ também na camada do vetor | riscos escuros nos desenhos da cena 6 | lida exata, a cobertura fica 0/1 |
| a sobre-relaxação da fonte do rebote (de graça) | — (análise) | duas paredes frente a frente têm o modo que alterna (autovalor −k): com albedo 0,9 só ω < 1,05 converge |
| dois rebotes no Médio e no Alto | +4,3 e +50 ms | ganham ~0,1 s de assentamento |
| C1: o passo do traço curto 0,25 · sem o filtro em cruz | −0,1999 → −0,1999 · −0,2072 | o viés da luz pequena não vem daí (vem da fusão) |

## Prova de mutação

Agente `mutacao`, 17 mutações com pré-voo das âncoras (todas casaram 1×), o controlo verde, restauro
`git checkout` + `touch` (e o `shaders.rs` nos `.wgsl`), árvore limpa no fim: **14 sangram** à primeira.
Sobreviventes: **M4** (a borda exata em toda a C3 em vez de só no interior) — EQUIVALENTE declarada: é
uma escolha de custo, medida com a mesma foto; **M7** (o filtro «a mesma face» da média ao longo da
face) e **M9** (a máscara do miolo no entorno) ganharam portão — `junto_a_um_canto_a_face_le_a_sua_luz`
(que mostrou que a MÉDIA espalhava o vazamento do canto ⇒ a mediana) e
`o_entorno_nao_conta_duas_vezes_o_que_esta_na_vista` (o corpo de 1 m: com 0,3 m o entorno 8× grosso nem o
capturava e a M9 passava) — re-provados à mão: M7 e M9 sangram, e a **M18** (a média no lugar da mediana)
sangra no do canto.

| sangram | o quê |
|---|---|
| M1 M2 M3 | a borda exata desligada · o recuo de sinal trocado · sem reta (o iso-½) |
| M5 M6 | o limiar na camada do vetor (+70,7 % de variação) · a média do alfa no sprite |
| M7 M8 M18 | sem o filtro «a mesma face» · sem a média · a média no lugar da mediana |
| M9 M10 M16 | sem o miolo · sem a soma do entorno · o deslocamento do entorno trocado (−63,6 %) |
| M11 | o entorno COM céu (a cena 5: a cruz sem faces — o céu contado duas vezes) |
| M12 M13 M17 | sem repor a emissão de base (outro ponto fixo) · sem a fonte do 1.º rebote (11 quadros) · o Baixo com um rebote |
| M14 M15 | as formas semeadas sem «bloqueia a luz» · o chão sem `RootOrder(0)` |

## Portão de fecho

Agente `verificador`, 1× sobre o diff desde o merge-base `8f20c60de` (a shell intacta: `git diff
8f20c60de -- shells/` vazio). 1.ª corrida: fmt ✓ · clippy `-D warnings --all-targets` (`ph2d-light2d-gpu`,
`ph2d-app-light2d`, `ph2d-light2d-ref`, `ph2d-light2d-ecs`) ✗ 2 na sonda (`type_complexity`,
`needless_range_loop`) · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ · `nextest-impacted`
2112 / 2113 — `workspace_src_files_under_loc_cap`: o `solver.rs` a 732 LOC (teto 700). Curado MOVENDO as
portas do entorno para o irmão `solver_entorno.rs` (676 + 66) e a sonda corrigida (`cda917031`). 2.ª
corrida: fmt ✓ · clippy ✓ · deny-check ✓ · `nextest-impacted` **2113 / 2113** ✓ (`load` 7). Nenhum `HashMap`
no diff. As suítes da família (placa 34 + 19 ignorados · família 59 + 20 · gabarito 27 + 2 · ECS 14 · lib
6) verdes; o G4 da clássica cai SÓ com as suítes em paralelo na placa (1,36 % sozinho, `load` 93).

Binário do smoke (depois do `rm -rf target/*/incremental` só no fim), 2.ª saída: `Finished \`smoke\`
profile [optimized] target(s) in 0.21s`. Máquina limpa (nenhum processo nosso, nada na placa).

`agent-loop-profile.sh` (20 sessões): paralelismo 1,13/passo ✗ · respostas por sessão 220 ✓ · test:check
2,8 ✗ · `Edit` 30 % ✗ · contexto relido 403 mil ✗ · contexto no início 62 mil ✓.

## ⚠️ ABERTO (W5d → W5e)

0. **Junto a um canto a cascata vaza a luz do outro lado** por ~2 texels (0,66 · 0,30 · 0,15 da luz de
   cima a 1 · 1,5 · 2 texels, o gabarito 0): da cascata, não da lei. A mediana não o piora.
1. **O disco de imagem ampliado** (zoom 2,5): 2 cruzamentos a mais e variação +3,56 (W5c +2,38). A textura
   do disco (256 px) ampliada: o iso-½ da rampa bilinear tem quinas a cada texel dela. E no app, zoom 1,
   ainda um grão fino de 1–2 texels na passagem luz/sombra de baixo (a foto `fotos_W5d/cena5_volume.png`).
   Candidatas: a captura do sprite pela MESMA lei da forma (o polígono do contorno alfa = ½ da textura,
   cozinhado uma vez por textura) ou amostrar a textura com um filtro que preserve a borda.
2. **O K9 no Médio e no Alto** (12 quadros): o 2.º rebote custaria +4,3 e +50 ms. Candidata: o 2.º rebote só
   enquanto a fonte muda (depois de mexer numa lâmpada ou num corpo).
3. **C1 — a luz pequena na holográfica** (−20 % no raio 1,5 células; o passo e o filtro não a mudam — a
   fusão dos níveis).
4. **C2 — a queda de alcance finito na holográfica** (RMSE 4,21): a fusão perde a distância ao recetor. Só
   toca lâmpadas de alcance finito além das 7 diretas.
5. **O entorno não vê os VETORES fora da grade da vista** (a camada do vetor está alinhada à grade dela) — o
   rebote de um desenho fora da vista não chega. E o céu da vista conta por inteiro mesmo onde um corpo de
   fora o taparia (o entorno não tem céu; com luz ambiente fraca não se mede).
6. **B1 com a parede dentro e o chão/teto fora** (câmera em −2): −11,3 % — o resto do entorno grosso.
7. **D — a memória** (Low 273 · Medium 434 · High 1011 MB + o entorno 8 · 34 · 128 MB a 1080p) e o
   governador acima de 1080p (lê só o tempo).
8. **E — o vetor que reflete**: o portão de paridade sprite × vetor do rebote (G7). A cena 6 mede a paridade
   da luz recebida (−0,33 %), não a do rebote que o desenho devolve.
9. **F — os da W5b**: a A3 (muitos objetos fora da grade sem nenhum a tapar), dois translúcidos sobrepostos
   fora da grade, o `a_nitidez_da_luz` sob carga. E o G4 da clássica sob carga (a família de flakes: 1,36 %
   sozinho com load 93; cai com as suítes em paralelo na placa).

## O que a W5e precisa saber

- `Config` novos (em execução, para as rodadas): `borda_exata`, `c0_ao_longo_da_face`,
  `rebotes_por_quadro`; o `Degrau` novo: `entorno`, `rebotes_por_quadro`. As portas novas do `Solver`:
  `mascarar_miolo`, `somar_entorno`, `c0()`. A `sprites_da_cena` passou a pública (para medir pelo caminho
  do produto).
- ⛔ Armadilhas desta onda: (1) a régua dos anéis precisa da vista CENTRADA no corpo (fora da janela a grade
  fina não se calcula e lê zero); (2) a imagem gêmea da cena 6 é o ESPELHO (`y → −y`), não a translação;
  (3) os passes sem carimbo não entram no custo (o 2.º rebote leu 5,82 → 5,74 ms até ganhar o carimbo);
  (4) a série do gabarito com o rebote tem degraus de 2 cm junto à superfície — não é régua de lisura ali;
  (5) uma grade esticada a ~2000 texels passa do tamanho de textura da captura fina (erro de validação, lê
  zero).

## Smoke do dono

`PH2D_LIGHT2D_SMOKE=5` (o volume) e `=6` (as formas desenhadas); `PH2D_LIGHT2D_ENTORNO=0` desliga o
entorno para comparar com zoom forte. Fotos: `fotos_W5d/cena5_volume.png` · `cena6_formas_desenhadas.png`
(e o antes em `fotos_W5c/cena5_volume.png`). Os passos estão no relatório ao dono.
