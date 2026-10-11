# Handoff de continuação — Luz 2D, W5e: os tetos do dispositivo, Flip e Motion na luz, S1–S8 contra o Godot (2026-10-11)

> Linha `line/Luz2D` (worktree `Worktrees/line-Luz2D`), base = `main` em `8f20c60de`. A W5d continua por
> integrar (13 commits antes desta onda). Commits da W5e: `615f42784` (tetos do dispositivo) · `0f6228ea0`
> (governador célula + rodada S) · `f0a4d6b33` (Flip bloqueia, cena 7) · `050110889` (Flip recebe) ·
> `edc4e44d0` (Motion bloqueia, cena 8) · `a14d3b157` (G7 do rebote) · `de258bd36` (oráculo Godot, fotos,
> sonda do quadro do app) · `114f3573d` · `ed146400a` (o portão, 1.ª e 2.ª corridas) · `6f6187537` (a
> mutação vira portões) · `4bd6480e1` (a rodada final de medição). Nada integrado; push e integração só
> por ordem do dono.

## O que fechou

| | o que o dono via / pediu | a causa MEDIDA | a cura |
|---|---|---|---|
| A | «a 4K a luz apaga sem aviso» | a captura fina (2× a fluência) a 2160p passa o `max_texture_dimension_2d` 8 192 do dispositivo da casa (10 240 / 9 984): erro de validação em `luz2d/tex/cobertura_fina` nos três níveis, a luz a calcular nada. O solver só conferia a fluência (BUGS §4) | `Config::cabe_na_placa` em toda textura da `PassList`; a fina que cede (`teto::fina_que_cabe`); o nascimento sob os escopos OutOfMemory + Validation; o governador desce pela FINA e pela CÉLULA antes do nível |
| B1 | o traço do Flip não projetava sombra nem recebia a luz | a camada do Flip não entrava na captura (`CenaDeLuz.outros` não existia) | contrato `CamadaDeOclusao` + `CamadaDeOutros`; `OclusaoDoFlip`; o `fs_blit` do `FlipCompose` com o `receber.wgsl` |
| B2 | as formas do Motion idem | idem | `OclusaoDoMotion` (as cópias da placa num passe irmão no formato da luz); a cena 8 |
| B3 | (E da W5d) o vetor que reflete: sem portão de paridade do REBOTE | — | G7 do rebote (cena 6, anel a 1,25× fora de cada forma, por canal) |
| C | S1–S8 «contra o Godot por script» | — | o oráculo corrido (`docs/Luz2D/oraculo_godot/`) + a rodada nossa nos mesmos cenários |

## O desenho

**Os tetos do dispositivo** (`teto.rs`, `solver_nascer.rs`). O que o wgpu 29 expõe (A2): o ADAPTADOR dá
`max_texture_dimension_2d` 32 768, mas o dispositivo da casa pede o padrão do wgpu (8 192); **não há
orçamento de VRAM**: só o relatório do alocador (o que JÁ se alocou), `InternalCounters` e a RECUSA — do
driver, ou de `MemoryBudgetThresholds::for_resource_creation` (Vulkan: `heap_usage + size ≥
heap_budget/100·t`; com limiar 1 % o dispositivo nem nasce, a 50/100 % cria). Por isso a defesa é em
dois tempos: (1) `Config::cabe_na_placa` diz, ANTES de alocar, se alguma textura da `PassList` passa do
lado; (2) `Solver::novo_com` aloca sob os escopos OutOfMemory + Validation e devolve
`ErroSolver::SemMemoria{bytes}`; uma validação SEM recusa é defeito do código → `ConfigInvalida` com o
texto do erro (não se cala). `teto::fina_que_cabe`: a fina 2^k o mínimo que cabe no lado E abaixo dos
bytes recusados NESTA janela; manual recusa com aviso, Automático desce o nível; se nada cabe abaixo da
recusa o solver NÃO realoca a cada quadro (outra janela/outro nível tentam de novo). Avisos novos:
`SemMemoria` («All light is off — the GPU refused its memory…») e `SombraEngrossada` («…shadow edges are
drawn N× coarser»).

**O governador abaixo do Baixo.** Dois eixos novos além do nível: a FINA (fixa, custo fixo) e a CÉLULA
(por nível, `CELULA_MAX` 2: a 8 px a 4K é no mundo a de 4 px a 1080p). Desce pelo eixo que pesa mais;
devolve fina → célula → nível, cada um previsto pela lei 1/célula² da grade dele.

**`CamadaDeOclusao` + `CamadaDeOutros`** (`ph2d-light2d-gpu`, `ph2d-app-light2d`). Contrato: a família
desenha numa camada da grade na resolução da CAPTURA, formato **`Rgba8UnormSrgb` premultiplicado**; a C0
lê **EXATA** (a lei do vetor: um píxel por texel, a média do alfa é a área). A coleta
`CenaDeLuz.outros` = entidades `BloqueiaLuz` sem Sprite, sem caminho nem Luz — perguntado por ENTIDADE
(um `try_query` com `Without<VecPathRef>` devolve NADA num mundo sem caminhos). `CamadaDeOutros::gravar_com(familias)`
no `LuzDoQuadro`; a shell só entrega as camadas (`shells/desktop/src/render_loop/present_luz.rs`) e o
documento do Flip ao codificar.

- **Flip bloqueia:** `OclusaoDoFlip` (`ph2d-app-flip`) — a MESMA tesselação do passe, o rasterizador do
  preview num `FlipRenderer` próprio. Nada para quem não pediu.
- **Flip recebe:** o `fs_blit` do `FlipCompose` aplica o `receber.wgsl` (`GrupoDeLuz` público na
  `ph2d-render`). ⚠️ O blit compõe TODOS os objetos Flip (ABERTO 4).
- **Motion bloqueia:** `OclusaoDoMotion` (`ph2d-app-motion`) — as cópias da placa (do dispositivo ou da
  CPU) num passe irmão no formato da luz. Nada sem placa, nada sem objeto pedido. A cena 8 marca os
  objetos Motion quando nascem.
- **A cena 7 e a 8:** a 7 «Traço do Flip» (discos exatos na captura); a 8 «Formas do Motion» = o smoke do
  Motion com a lâmpada e o chão por cima. O censo do roteador (`max_level 8`, «não há cena 9») mora no
  teste dela.
- **G7 do rebote** (`o_rebote_do_desenho.rs`, `os_outros_devolvem_a_luz_como_a_imagem`): a camada dos
  outros devolve a luz como a imagem.

**A faixa `luz2d-placa`** (`.config/nextest.toml`, `max-threads = 2`, filtro `ph2d-app-light2d |
ph2d-light2d-gpu`): o fan-out do nextest dava 9 recusas de memória da placa (verdes sozinhos, com ~7 GB já
tomados por outros programas a `load` 8–34). Membro da família de flakes de carga — `FLAKES_DE_CARGA.md`
§ «A espécie da memória da placa».

## Antes → depois

**A0 — o declarado (sem placa, `PassList`, vista de fábrica; MB vista + entorno)**

| | Baixa | Média | Alta |
|---|---|---|---|
| 1080p | 275 + 8 | 434 + 34 | 1011 + 128 |
| 1440p | 472 + 16 | 732 + 60 | 1643 + 250 |
| 2160p | 1109 + 34 | 1663 + 128 | 4162 + 528 |

A memória mora na grade FINA: 1080p Baixa → borda 126 · `tau_dentro_fina` 31 · `cobertura_fina` 31 ·
fluência 31 de 275 (80 %). A janela é 19–47 % da grade (vista de fábrica): encolher a computação às
janelas é alavanca de TEMPO, não de memória (a alocação é para o pior zoom da oitava, sem realocar no
zoom — BUGS §1).

**A1 — a placa, rodada final** (máquina calma, `load` 1,4 → 0,7, RTX 5060 Ti, `smoke`, o MÍNIMO; cena 3,
fábrica; alocado MB · ms por nível + fixo = total):

| | Baixa | Média | Alta |
|---|---|---|---|
| 1080p | 285 · 2,71 + 1,54 = **4,25** | 470 · **7,12** | 1151 · **30,99** |
| 1440p | 489 · **7,43** | 797 · **12,75** | 1900 · **59,79** |
| 2160p (fina ×2) | 478 · 9,29 + 2,22 = **11,51** | 1151 · **30,93** | 4105 · **141,83** |

Alocado ≈ declarado (285/283, 470/468, 1151/1139 MB a 1080p). Orçamento 5,56 ms: 1080p Baixa cabe; a
1440p a fina (fixo 2,93 → /4) cabe; a 2160p a célula (9,29 → /4). Antes: 2160p dava erro de validação
nos três níveis. A rodada com `load ~20` (vale a ordem) dera 4,36 · 7,40 · 31,2 / 7,7 · 13,0 · 59,8 /
12,3 · 31,0 · 142,3 ms.

**O quadro do app** (Alta + Automático, cena 3, 1930×1012): Alta 26–29 ms de placa → Média ao 5.º
quadro; fica em Média 5,4–6,4 ms (oscila à volta do orçamento 5,56 sem 3 caras seguidas); a troca com as
pipelines já compiladas 24 ms. Na 1.ª sonda (antes da rodada final) desceu a Baixa em 11 quadros, ~6 ms de
parede. ⚠️ A 1.ª troca de cada nível, a FRIO, compila a variante: 2,0–2,7 s (ABERTO 1).

**S — nós × Godot** (Godot 4.7.2 por script, GPU ms; nós = Baixa, vista de fábrica, total ms, rodada final)

| cenário | Godot (sem / com sombra) | nós |
|---|---|---|
| S1 uma luz e um corpo, 1080p | 0,038 / 0,052 | 5,71 |
| S1 720p · 1440p · 2160p | S7 (sem sombra): 0,097 · 0,367 · 0,819 | 4,44 · 9,11 · 11,85 |
| S2 sprites emissivos N = 1·10·100·1000·10000 | 0,033 · 0,145 · 0,209 · 0,217 · 0,520 | 2,00 · 2,02 · 2,04 · 2,52 · 6,04 (1080p; c2 plano 1,08) |
| S2 com sombra (acima de 256 luzes o Godot cala e dá milhões de linhas de erro: INVÁLIDO) | 0,036 · 0,156 · 0,293 · 0,416 · 0,888 | — |
| S2L N lâmpadas (o S2 do Godot) 1·10·100·1000·10000 | (mesmos Godot) | 1080p: 1,90 · 4,48 · 67,2 · 379 · 374 · 1440p: 3,14 · 7,51 · 188 · 1 679 · 2 834 |
| S3 K = 0·10·100·1000 oclusores, com sombra | 0,067 · 0,069 · 0,071 · 0,123 (mediana K=1000: 0,30) | 1080p: 1,90 · 6,51 · 28,2 · 43,8 · 1440p: 3,2 · 11,7 · 61,4 · 119,3 |
| S6 pan N = 100 | 0,202 | = o G11 (ancorada 0, solta 4,9) |
| S4/S5 · S8 | — | = o G3 · = o G9/K9 |

Limites do Godot medidos (não assumidos): 15 luzes por item e 256 por quadro, calados. A comparação é de
ORDEM DE GRANDEZA: o Godot desenha sombras de lâmpada por oclusor (sem rebote, sem difusa), nós calculamos
um campo; o nosso custo é plano em N de emissores (S2) e o dele cresce — onde o nosso é pior é a luz DIRETA
com geometria densa. **A maior distância ao piso**: S3 e S2L — a luz direta (7 lâmpadas) marcha pela grade
fina através das lâmpadas opacas (`c3_fina` 0,05 → 347 ms). É o próximo alvo de tempo.

**Flip e Motion — a paridade com a imagem** (contratos de camada; ponta a ponta antes de o portão sair da
família):

| | contrato | ponta a ponta | cena |
|---|---|---|---|
| Flip bloqueia | área +0,078 %, pior píxel 0,146 | traço × imagem −0,55 · +0,19 %; anéis +3,3 · +2,0 % (grade fina pelo meio) | 7 (discos exatos): paridade −0,38 · +0,38 %, anéis +2,0 · −1,4 %, faces 45 × e 38 ×; sem a família: sem faces |
| Flip recebe | pior erro 1,34e-3 em 1 680 canais (sprite 1,21e-3) | controlo apagado byte a byte | — |
| Motion bloqueia | área −0,184 % (polígono inscrito a 0,25 px), pior píxel 0,118 | forma × imagem −0,49 · +0,06 %; anéis +6,0 · +22,3 % (grade fina pelo meio) | 8 |

Na captura EXATA (a lei da camada) os anéis do Motion ficam em 4,5 · 8,5 %. O que sobra de anel é a FASE
da grade (= o F5 do plano).

**G7 do rebote** (cena 6, por canal): círculo −0,37 · −0,25 · −0,20 %, elipse −0,12 · −0,19 · −0,21 %;
controlo sem reflexo +1,8 · −6,7 · −8,5 % e −9,0 · −3,1 · −0,9 %.

## ⛔ Recusas MEDIDAS (W5e)

| tentativa | número | por quê |
|---|---|---|
| subir `max_texture_dimension_2d` na `ph2d-gpu` (o adaptador dá 32 768) | — (análise) | a exportação PNG do Quadro usa o limite como lado do ladrilho: 32 768² Rgba8 = 4 GB; a cura é a luz caber no limite, não o limite subir |
| `MemoryBudgetThresholds` na instância da casa (a recusa por orçamento do alocador) | muda TODAS as famílias | decisão do integrador/dono (ABERTO 8) |
| a camada dos outros na grade fina lida pelo MEIO (bilinear ≥ ½, como o sprite) | anéis +13 / +18 % | a camada é exata (área no alfa); com o limiar ficam riscos — a lei do vetor (W5d) |
| a tolerância do nível de detalhe da placa a 0,125 e 0,01 px (formas do Motion) | anéis 19,2 e 22,1 % | não são as facetas; revertida |
| deslocar a camada do Motion ±½ px | fase ±½ px: 4,4–12,5 / 13,3–22,7 % — piora a face | a fase da grade é o F5; não se compensa na família |
| o 2.º rebote só enquanto a fonte muda, no Médio/Alto (K9) | +4,3 / +50 ms durante o arrasto | análise, NÃO implementado (ABERTO 9) |
| dev-dependency família → família para os portões | o censo `no_dependency_climbs_a_layer` reprova | a regra das camadas; o contrato mora no portão da luz e a família prova o seu lado |

## Prova de mutação

Agente `mutacao`, 20 mutações: **15 sangraram à primeira**; sobreviventes curados em `6f6187537`.

| sangram | o quê |
|---|---|
| M1 M2 M4 M5 M8 M9 M13–M20 | (as da família: tetos, governador, Flip, Motion, G7) |
| M6 M7 (curadas) | M6 a recusa não lembrada · M7 o «nada cabe» a montar de novo — o portão da memória agora exige o RECUO e, apagada a recusa, zero montagens novas (`LuzDoQuadro::montagens`; uma corrida a 2160p que TEM de apagar) |
| M11 (curada) | a camada dos outros lida bilinear — `captura_dos_outros`: RMSE 0,0033 contra o polígono de 256 lados; bilinear 0,0066 (tol 0,0037); sem a família 0,92 |
| M12 (curada) | lida não premultiplicada — G7 dos outros: −0,27 % (tol 0,30 %); não premultiplicada −0,39 %; sem reflexo −8,6/−14,2 % |
| M3 (EQUIVALENTE) | validação sem OOM: nenhuma configuração conhecida a provoca; o ramo existe para dizer o defeito em vez de o calar |
| M10 | só sangra na cena 7 (a 8 não cobre a exclusão) |

## Portão de fecho

Agente `verificador`, 1× sobre o diff desde `8f20c60de`. **1.ª corrida:** 6 defeitos (clippy 2 · HR-15 2 ·
camadas família→família 1 · `fn_loc` 1) + 9 recusas de memória no fan-out (a faixa `luz2d-placa`).
Curados em `114f3573d` (a camada dentro da lei do vetor, os contratos nas famílias, a luz da shell num
irmão — `present_luz.rs`). **2.ª corrida:** fmt ✓ · `check --workspace` deny-warnings ✓ · `nextest-impacted`
**17 509 / 17 509** · censos ✓ (`the_shell_only_shrinks` · `no_dependency_climbs_a_layer` ·
`toda_flake_de_carga` · `panel_wiring_parity` · `fn_loc_caps` · HR-15 luz e motion · `seam_luz` 7/7) ·
clippy 2 (`chunks_exact`) e o tecto de 700 (`medida_luz2d_w5d.rs` com 796 LOC, herdado da W5d, agora
cortado) curados em `ed146400a`. Depois da mutação, +2 portões (`captura_dos_outros`,
`os_outros_devolvem_a_luz_como_a_imagem`) e o reforço de `a_luz_recua_quando_a_placa_recusa_a_memoria`
(`6f6187537`), clippy limpo. Gates novos da onda: `os_tetos_da_placa` (6: fina só onde precisa — 1080p,
1440p e 2560×1080 k=0, 2160p o menor k · memória recusada · governador abaixo do Baixo · 2160p acesa sem
erro, com controlo: o degrau da W5d recusa limpo · a placa a recusar de verdade, limiar mínimo em que nasce
e aloca 64 MB · a luz a recuar 434 → 260 → 221 MB, com aviso e sem validação).

Rodada final: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_GPU=1 bash
scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it medida_luz2d_w5e -- --ignored
--nocapture --test-threads=1` — saída inteira em [`medida_luz2d_w5e_rodada_final.txt`](medida_luz2d_w5e_rodada_final.txt).

## O oráculo Godot

`docs/Luz2D/oraculo_godot/` (`corre.sh`, `oraculo.gd`, `project.godot`, fixture `medida_godot_w5e.txt`,
saída crua, `fotos/` com S1–S3·S6·S7 em PNG). CORRIDO por script, nunca lido (CLAUDE.md §0.9); fixture
medida a `load` 19–28 (vale a ordem). **Triagem de licença por artefacto:** `godot` 4.7.2 MIT;
bibliotecas MIT/BSD/Apache/Zlib/libpng/LGPL-de-sistema; o driver NVIDIA é proprietário e **não se
distribui**. Comando: `bash docs/Luz2D/oraculo_godot/corre.sh` (passa pela porta da placa; exit 75 = espere).

## ⚠️ ABERTO (W5e → W5f)

1. **A compilação a FRIO de cada nível, 2–3 s** na 1.ª troca do processo (q0 2,0 s; q6 2,7 s) — o BUGS §1
   fala em 2–10 ms nas revisitas (a `Fabrica`); o Automático que desce de Alta paga-a. Candidata:
   pré-compilar o nível vizinho em segundo plano.
2. **A luz direta com geometria densa (S2L, S3)**: o maior desvio do piso Godot, o próximo alvo de TEMPO
   (380 ms com 1 000 lâmpadas; `c3_fina` 0,05 → 347 ms).
3. **Fotos da tela virtual com a luz**: 5 qps (cena 7) / 2 qps (cena 8) na barra; a W5d no mesmo ambiente
   7 qps (133 ms) e a cena 6 hoje 5 qps (172 ms) — já existia; a luz em si ~6 ms no mesmo processo.
   MEDIR NO MONITOR DO DONO.
4. **`NaoRecebeLuz` por objeto Flip**: o blit compõe todos os objetos.
5. **A placa não sabe de que objeto é cada cópia**: um objeto Motion que bloqueia faz bloquear TODAS as
   formas; a placa sem dono por cópia.
6. **As cópias de SPRITE do Motion** (`source.object`, o Swarm) não entram na captura; a rota Vello
   (`PH2D_FORMAS_NA_PLACA=0`) não bloqueia.
7. **O «distantes» (F4) não vê Flip/Motion fora da grade** (como os vetores, ABERTO 5 da W5d).
8. **Decisões do integrador/dono:** o limiar `MemoryBudgetThresholds` na instância da casa (muda todas as
   famílias) e o limite `max_texture_dimension_2d` 32 768 (o ladrilho PNG do Quadro).
9. **D — o K9 no Médio e no Alto** (12 quadros): análise do 2.º rebote só enquanto a fonte muda —
   +4,3 / +50 ms durante o arrasto; não implementado.
10. **M3 equivalente** (validação sem OOM): declarada, sem portão por construção.
11. **A fila, na ordem do briefing:** E = C1 (a luz pequena na holográfica, −20 %) e C2 (a queda de alcance
    finito, RMSE 4,21) — limites da fusão; F = os da W5b (a A3, dois translúcidos fora da grade, o
    `a_nitidez_da_luz` sob carga); G = o F5 do plano (a passagem luz/sombra nos relevos) — **ÚLTIMO, ordem
    do dono (11/10)**. E os ABERTOS 0, 1, 5 e 6 da W5d continuam.
12. O `medida_luz2d_w5d.rs` da W5d estava a 796 LOC (tecto 700) e foi partido em `ed146400a` (796 → 507 + 312):
    quem procurar uma régua dessa onda confira os dois ficheiros.

## O que a W5f precisa saber

- ⛔ Armadilhas desta onda: (1) o `try_query` com `Without<VecPathRef>` devolve NADA num mundo sem
  caminhos — pergunte por entidade; (2) uma textura grande que falha dá ERRO DE VALIDAÇÃO e a luz lê zero:
  toda régua nova a 4K precisa do `cabe_na_placa`; (3) a rodada com `load` > 5 só vale a ordem — a
  final foi a `load` 1,4 → 0,7; (4) o fan-out do nextest com a placa partilhada reprova por memória:
  a faixa `luz2d-placa` (2 fios) é a cura, não retentar; (5) a camada dos outros é lida EXATA — com
  limiar ou bilinear os anéis sobem 13–18 %; (6) o censo do roteador das cenas (`max_level 8`) mora no
  teste da cena 8: a próxima cena é a 9.
- Superfície de colisão: `ph2d-light2d-gpu` (`CamadaDeOclusao`, `solver_nascer.rs`, `Config::cabe_na_placa`),
  `ph2d-app-light2d` (`teto.rs`, `CamadaDeOutros`), `ph2d-app-flip` (`OclusaoDoFlip`), `ph2d-app-motion`
  (`OclusaoDoMotion`), `ph2d-render` (`GrupoDeLuz` público), `shells/desktop/src/render_loop/present_luz.rs`
  (a luz da shell num irmão, para a shell não crescer), `.config/nextest.toml` (faixa `luz2d-placa`),
  `docs/DevOps/FLAKES_DE_CARGA.md`. Sem degrau de `PROJECT_SCHEMA`; nenhum contrato congelado tocado.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
   — **cena 7, o traço do Flip**: tem de aparecer a sombra do traço no chão, com uma face acesa do lado
   da lâmpada e a de trás escura. Errado: o traço sem sombra nenhuma.
2. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_MOTION_OBJETO_SMOKE=1 PH2D_LIGHT2D_SMOKE=8 cargo run -p ph2d-host-desktop --profile smoke`
   — **cena 8, as formas do Motion** a cair sob a lâmpada: cada forma projeta sombra no chão (o «recebe» das
   formas do Motion não foi medido nesta onda). Errado: formas sem sombra ou a luz apagada.

Fotos: [`fotos_W5e/cena7_traco_do_flip.png`](fotos_W5e/cena7_traco_do_flip.png) ·
[`cena8_formas_do_motion.png`](fotos_W5e/cena8_formas_do_motion.png) ·
[`cena8_sem_luz_controlo.png`](fotos_W5e/cena8_sem_luz_controlo.png) (a mesma cena sem a luz). ⚠️ Tela
virtual, perfil `smoke`: a barra mostra 5 e 2 qps COM a luz (ABERTO 3). Oráculo e licenças: secção acima.
