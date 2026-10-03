# Handoff — `line/3DModeling`: O CONTACTO (2026-10-03)

> **Wave, não integração.** Item 5 da ordem aprovada pelo dono (contacto entre peças + chão em
> paridade). Continua [AS_TEXTURAS](HANDOFF_line_3DModeling_AS_TEXTURAS_2026-10-03.md). Base da linha
> `1ad60a1ce` (main não andou). Commits: `f6c128d26` o céu do chão · `886033b8d` os gates dele ·
> `83866949c` a crate `ph2d-contacto` · `71c0d2be6` o contacto na placa · `14490aa71` a oclusão própria
> (cones, depois trocada) · `6fc671bd4` o app assa do campo DO OBJETO · `2526730db` o custo por pixel ·
> `b8b47730b` a cena 40 · `628928ecd` a costura · `4f5409497` raios no campo cru. **Smoke do dono:
> pendente.**

## ⛔ A premissa do briefing que a medição derrubou

*«Gate de paridade contra o catcher do Render traçado (`ph2d_field_render::ground`, a lei já
aprovada)»* — o traçado **não** bate com a física: o Cycles (céu uniforme, esfera e caixa pousadas,
shadow catcher) dá a forma fechada `(r/D)³` da esfera a `1 %`, e o `ground_sky` do traçado escurece
**demais e largo demais**, e depende do tamanho da cena (alcance = `2 ×` a bola do documento): a
`0,4` do contacto da esfera Cycles `0,215` · traçado `0,43` (só a esfera) / `0,53` (a cena) · os cones
de referência dele (`2 048`) `0,454`; a `0,8` da esfera, na cauda, Cycles `0,043` · traçado `0,30`.
⇒ **a régua dos dois itens é o Cycles**, corrido sobre entradas nossas.

## §1 — O que existe

**(a) O céu do chão — `ph2d-mesh-forward`**

- A cobertura passa a ser vista também **de baixo** (`fs_cobertura_baixo`, pipeline `desenha_baixo`,
  canal `b` = o fundo das peças). Por fatia de azimute o chão vê um **intervalo** de elevações tapado
  (do fundo ao topo), não só um horizonte: o campo de alturas só de cima enchia o vão debaixo de uma
  esfera (`0,378` contra `0,318` exacto a `d = 0,321`).
- `gpu_ceu_chao.rs` + `ceu_chao.wgsl`: o céu do chão num passe próprio de `512²` no enquadramento da
  cobertura, **não depende da câmara** — `4` fatias × `~33` passos (razão `1,25`, pegada `0,03`), o
  azimute e o 1.º passo **entrelaçados** numa grelha `4×4` e um borrão `5×5` no MESMO quadro (nada
  acumula); texels DENTRO de uma peça pousada (fundo a `< 3/255` do chão) não entram; o chão lê com
  uma leitura filtrada pré-multiplicada pela validade. **Só se refaz quando a chave muda** (uniforme,
  poses, geração das malhas): girar a câmara custa zero.
- `COBERTURA_LADO 512 → 1024` e margem `MARGEM_CEU = 4 ×` a altura (a cauda), a cair a zero nos `15 %`
  da borda. ⚠️ O ramo «sem bloqueador da luz-chave» devolvia `vec2(1)` e apagava o céu longe das
  peças — corrigido (`a_cauda_do_ceu_do_chao_nao_tem_degrau`).

**(b) O contacto — crate NOVA `ph2d-contacto` (CPU + gémeo WGSL, sem wgpu)**

- `Volume`: o campo de UMA peça amostrado (`64³` sobre a caixa da malha), trilinear; fora da caixa, um
  limite inferior. **Cru** — dividir por `|∇f|` piorava os tubos (§3).
- `Grade` (`16³`, cubo de meia-aresta `raio·(1 + MARGEM=1)`): em cada ponto, `64` raios marcham a peça e
  a visibilidade dela projecta-se em **harmónicos esféricos de ordem 2** (9 coeficientes); pontos de
  DENTRO empurrados para a superfície. `oclusao(p, n)`: trilinear + convolução com o cosseno
  (Ramamoorthi–Hanrahan); fora do cubo, a da borda × `1/s²`; além de `ALCANCE = 8` meias-arestas, zero.
  `em_f16()` = o que a placa guarda. É a *neighborhood transfer* do PRT (Sloan 2002) / *AO fields*
  (Kontkanen–Laine 2005).
- `visibilidade_propria(vol, pos, nrm, alcance)`: **`128` raios BINÁRIOS** distribuídos pelo cosseno,
  conjunto fixo (sem ruído entre vértices); o raio pára ao sair da caixa (convexa).

**`ph2d-mesh-forward` (b)**: `gpu_contacto.rs` — atlas de três texturas `3D` `Rgba16Float` (uma telha
`16³` por malha, dobra quando enche), a tabela do quadro (por instância com grelha: a afim mundo →
grelha e a telha; `MAX_CONTACTO = 32`); `forward.wgsl::contacto(p, n)` multiplica o peso do CÉU (a
caixa tem o mapa de sombra); a própria instância salta (`Objeto.extra.x`). `Forward::sobe_contacto`;
subir outra malha no mesmo id esquece a grelha velha. `gpu_ligacoes.rs`: o grupo `0` saiu do `gpu.rs`
(`697 → 632` linhas).

**`ph2d-app-field3d`**: `malha_render_contacto::assa` — por objeto do Render, o campo das **unidades
dele** (não o grupo) → `Volume` → oclusão própria em `malha.ao` + a `Grade`, que o quadro sobe
(`sobe_contacto`); a placa lê-a com a pose de AGORA: mover não refaz nada. Sai a oclusão de Quilez
(`AO_PASSOS`, a escala `|∇f|` dos cantos). **Cena 40 «O CONTACTO»** (`smoke_scenes_contacto.rs`):
caixa azul, bola amarela `1 cm` acima, vermelha `1 cm` ao lado, toro verde deitado, L laranja fundido;
a fila ao longo de Z. `CENAS = 40`.

### Ids / variantes / consts novos (colisão na integração!)

| o quê | valor |
|---|---|
| ligações do `g0` do desenhista | `15` (céu do chão), `16` (tabela do contacto, uniforme), `17–19` (atlas `3D`) — `14` texturas amostradas de `16` |
| pipelines compilados | `7 → 10` (`12 → 15` com o brilho): cobertura de baixo, céu do chão, borrão |
| `Objeto` (uniforme do grupo `1`) | `64 → 80` bytes (`extra.x` = índice da instância), visível no FRAGMENTO |
| consts | `CEU_CHAO_LADO = 512` · `COBERTURA_LADO = 1024` · `MARGEM_CEU = 4.0` · `MAX_CONTACTO = 32` · `ph2d_contacto::{LADO 16, MARGEM 1.0, COEFS 9, RAIOS 64, ALCANCE 8.0, RAIOS_PROPRIOS 128}` · `LADO_VOLUME = 64` |
| `smoke::scenes::CENAS` | `39 → 40` (o integrador **reconta**); notas `1..40` em `smoke.rs` e `lib.rs`; `QUEM_SEMEIA` `(40, …)` |
| crate nova | `ph2d-contacto` (deps `half`, `rayon`); dependem dela `ph2d-mesh-forward` e `ph2d-app-field3d` |
| `PROJECT_SCHEMA` | intacto · shell (`shells/desktop`): `0` linhas |
| fixturas | `ph2d-mesh-forward/fixtures/oraculo_ceu_do_chao.csv` (`1 349`) · `ph2d-contacto/fixtures/oraculo_contacto.csv` (`11 247`, `764 KB`) · `…/oraculo_oclusao_propria.csv` (`14 717`, `1,0 MB`) |
| instrumentos | `docs/3DModeling/ferramentas/oraculo_{ceu_do_chao,contacto,oclusao_propria,oclusao_malha}_blender.py` · `tests_custo_chao::instrumento_custo_ceu_do_chao` · `tests_custo_contacto::instrumento_custo_contacto` · `malha_render_contacto::tests::sonda_oclusao_da_malha` |

## §2 — Medições

**(a) Chão contra o Cycles** (`o_ceu_do_chao_e_o_do_cycles`, 4 cortes): antes (horizonte só do topo, por
pixel, 8 × 5) `|Δ|` médio `0,029–0,063`, máx `0,10–0,16`; agora médio `≤ 0,0062`, máx `≤ 0,030`,
cauda (Cycles `< 0,1`) `≤ 0,016`. Varrido no gate: `256²` dá `0,072` junto à face da caixa (um texel de
`2,8 cm`), `512²` `0,030`; cobertura `512` dá médio até `0,009`, `1024` até `0,007`.
**Custo** do passe (RTX 5060 Ti, load 12, `+` sobre a base de `0,03 ms`): `256²`/4 fatias `0,18 ms` ·
`512²`/4 `0,25` · `512²`/8 `0,44` — e zero quando a chave não muda.

**(b) Contacto entre peças** (CPU, `a_oclusao_entre_pecas_e_a_do_cycles`, `11 242` pontos): médio
`0,0114`, onde o Cycles `< 0,9` `0,016`, máx `0,175` (o mesmo de `512` raios exactos: silhueta); sem a lei
`0,218` / `0,370`. **Placa** (`o_contacto_na_placa_e_o_do_cycles`, esferas RODADAS): contra o Cycles
`0,0091` / `0,0132` / máx `0,063`; contra a CPU `0,0031` / máx `0,010`. **Custo por pixel** (load 4,
ecrã cheio a 1080p): `1/2/4/8/16` vizinhas `+0,015 / 0,047 / 0,109 / 0,233 / 0,526 ms` (`~0,03 ms` por
vizinha; só paga quem está ao alcance). Celular (×13, a regra das texturas) `~0,4 ms` por vizinha em
ecrã cheio — não medido num aparelho.

**Oclusão própria** (médio / onde o Cycles `< 0,9`): peças fundidas (`a_oclusao_propria_e_a_do_cycles`)
`0,004` / `0,006–0,010`; malhas REAIS dos tubos das cenas `28` e `37` (Cycles sobre o `.obj`
extraído, `sonda_oclusao_da_malha`) `0,013–0,016` / `0,017–0,019`. A de Quilez que a casa assava:
`0,05–0,12` / `0,14–0,22` e `0,11–0,15` / `0,09–0,12`.

**App**: entrar no Render `0,35 s` (cena 37) e `0,39–0,48 s` (28) — o volume + a própria + a grelha
`~8–10 ms` por objeto; quadro a girar `1,35–1,6 ms` de mediana (load 2–4). Costura
(`o_contacto_chega_ao_quadro_do_render`, cena 40): `1 928` px escurecem `> 10 %`, o mais escuro
`× 0,275`, nenhum clareia. Foto do app (cena 40): ver §6; o chão escurece `~6 %` junto à base sobre o
cinzento do editor — é a física de um chão que só recebe sobre um fundo médio.

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| O `ground_sky` do traçado como régua | escurece demais e largo demais, depende do tamanho da cena (cabeçalho) |
| Horizonte só do TOPO (campo de alturas) | enche o vão debaixo de peças redondas/a flutuar: `+0,06` junto à esfera |
| O céu do chão por pixel (`32` fatias × `80` passos) | exacto (`0,007`) mas `2 560` leituras por pixel do chão; o passe de `512²` dá o mesmo com `~130` por texel |
| Fatias sem entrelaçar | `16` fatias × `18` passos: médio `0,019`, degraus de azimute; `desvio = 0,5` sangra o gate |
| `SOLIDO = 1 mm` | a profundidade de `8` bits erra `±1,2 mm`: texels de dentro da caixa entravam (`0,58` junto à face) |
| Grelha do contacto de ordem `1` | `0,028`; a de ordem `2` exacta no ponto `0,0061` |
| `9` cones moles por pixel (DFAO) | `0,018` a `~90` leituras por pixel; a grelha lê `3` |
| Grelha `24³`/`32³`, margem `2–3` | não melhoram (`0,012–0,019`); a cauda `1/s²` fora do cubo é que curou a margem |
| Oclusão de Quilez (`5` passos na normal) | §2; e lia o campo do GRUPO — contava as vizinhas, presas à pose da extracção |
| `48` cones moles para a própria | fundidas `0,006–0,011`, tubos `0,044` (escuros: leem distância e o campo por fórmula não é uma) |
| Raios no volume dividido por `|∇f|` | tubos `0,016–0,020`, viés `+0,014`; cru `0,013–0,016`, viés `±0,003` |
| Raios rodados por vértice | ruído `±0,04` entre vértices sem ganho medido; o conjunto fixo dá degraus de `1/128` |
| Limite inferior de FORA da caixa nos raios da própria | escurecia os que saem rasantes (`−0,025`) |

## §4 — Gates

| Crate | Gates |
|---|---|
| `ph2d-contacto` | `a_oclusao_entre_pecas_e_a_do_cycles` · `a_oclusao_propria_e_a_do_cycles` |
| `ph2d-mesh-forward` | `o_ceu_do_chao_e_o_do_cycles` · `o_ceu_do_chao_segue_as_pecas` (pose, forma, meia volta) · `a_cauda_do_ceu_do_chao_nao_tem_degrau` · `o_contacto_na_placa_e_o_do_cycles` · `o_contacto_anda_com_a_peca` (`0,188 → 0,989`) · `gpu_contacto::a_inversa_desfaz_a_pose` · os de antes verdes (`26/26` com a placa) |
| `ph2d-app-field3d` | `uma_peca_convexa_nao_se_tapa` · `a_oclusao_propria_nao_conta_a_vizinha` · `o_contacto_chega_ao_quadro_do_render` (placa) |

**Mutações (a): 9, 9 VERMELHAS** — o intervalo sem o fundo, o céu não lido, o retorno cedo sem o céu,
a chave sem a matriz, sem `malha_mudou` no `sobe`, `SOLIDO = 0`, o borrão sem a validade, `MARGEM_CEU
= 0`, sem entrelaçar. ⛔ Duas sobreviveram à 1.ª corrida (a chave sem a matriz: mover a caixa também
mudava o quadro; o retorno cedo: o gate não ia longe o bastante) — os gates ganharam a meia volta e a
cauda. **Mutações (b): 11, 10 VERMELHAS + 1 equivalente** — sem saltar a própria instância, a normal
sem ir ao referencial da peça, a cauda sem o `1/s²` (placa e CPU), a translação da afim com o sinal
trocado, sem subir a grelha no app, a grelha velha que não se esquece ao subir outra malha, a própria
lida do campo do GRUPO. ⛔ Sobreviveram à 1.ª corrida: o campo do grupo (as duas bolas do gate nem
caíam no mesmo grupo — trocadas por caixas, cujas bolas de bordo se sobrepõem: agora vermelha, mínimo
`0`); empurrar os pontos de dentro da grelha para a superfície (não mudava nenhuma resposta: SAIU do
código); parar o raio da própria à saída da caixa (equivalente — só poupa marcha: a caixa é convexa e
o limite de fora nunca chega a `0`).

**Suítes** (verificador, load 1–8): `ph2d-contacto` 2/2 · `ph2d-mesh-forward` 26/26 com a placa ·
`field3d` lib `514/518` — os 4 passam sozinhos: `dragging_a_colour_compiles_no_tape_at_all` lê o
`POINT_TAPES` GLOBAL (o doc dele diz: só sob `nextest`, um processo por teste) e três da costura do
Render esperam o desenhista GLOBAL sob `cargo test` paralelo · clippy `-D warnings` limpo · fmt limpo.

## §5 — ABERTO

- **O chão sobre o fundo escuro do editor** escurece pouco à vista (`~6 %`): é a física do *shadow
  catcher*; num céu fotográfico claro vê-se mais.
- **O Render TRAÇADO** continua com o `ground_sky` antigo (escuro e largo, §cabeçalho): sai quando o
  dono aprovar o Render por malha; se ficar, reajustar contra o oráculo do chão.
- `MARGEM_CEU = 4 ×` a altura: um objeto alto e fino alarga o quadro (texel mais grosso) — não medido.
- O intervalo é UM por coluna: peças empilhadas com vãos entre elas aproximam-se pela união.
- `MAX_CONTACTO = 32` instâncias (a 33.ª não tapa ninguém); o CHÃO não tapa as peças (a parte de baixo
  delas vê o céu de baixo do estúdio); peças que partilham unidades (corte, espelho: `movel = false`)
  contam o gémeo na própria E na grelha.
- A própria tem degraus de `1/128` (`~0,8 %`) em superfícies lisas; viés dos tubos `±0,003`.
- Custo no celular real (o passe do chão `~0,25 ms` aqui quando refeito; o contacto `~0,03 ms` por
  vizinha em ecrã cheio) — medir num aparelho.
- As fotos da sonda herdam a luz girada da cena anterior (a ordem das cenas muda a foto).
- Herdados: §5 do [AS_TEXTURAS](HANDOFF_line_3DModeling_AS_TEXTURAS_2026-10-03.md#5--aberto).

## §6 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=40 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**.
3. Deve acontecer: a bola amarela por cima da caixa azul fica mais escura por baixo, e o topo da caixa
   escurece debaixo dela; a bola vermelha e a face da caixa escurecem onde se olham; o toro verde
   escurece por dentro do anel; o L laranja escurece no canto de dentro; o chão escurece à volta de
   cada peça. Clique na bola **vermelha** e arraste a seta do gizmo para longe: a mancha na caixa vai
   com ela, na hora; traga-a de volta e a mancha volta.
4. Deu errado se: a mancha fica onde a bola estava, aparece um quadrado de borda reta no chão, as
   peças ficam pretas onde nada as tapa, ou engasga.

Foto do app: `PH2D_GPU=1 bash docs/Components/ferramentas/fotografa_cena.sh "PH2D_FIELD_SMOKE=40
PH2D_FIELD_SHADING=render" 1930 1040 <png> 18`. Oráculos: os quatro scripts de §1 (o cabeçalho de cada um
diz como se corre).

## §7 — Reports do dono

- (pendente)

## §8 — A PRÓXIMA ONDA

A ordem aprovada em 02/10 (itens 1–5) fica completa com este smoke. O que sobra sem ordem: retirar o
Render traçado quando o dono aprovar; mobile real; LOD; o chão que tapa as peças; dentes de 2–3 px na
quina côncava (§5-b do [O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md)).
