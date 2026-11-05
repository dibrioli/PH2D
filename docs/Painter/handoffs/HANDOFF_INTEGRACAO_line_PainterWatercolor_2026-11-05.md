# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, 05/11/2026 (Smooth Edges por traço, Grain à mão, placa E11, Bugs #57–#65)

HEAD: o commit deste handoff, 62 sobre `effe04a1c9` (o `main`, merge-base igual).

SMOOTH_BUILD: 2.ª corrida, 0 `Compiling`: `Finished smoke profile [optimized] target(s) in 0.22s`.

Leitor: o integrador (e a próxima LLM da linha). Ordem: **só integra por ordem explícita do Enio**; sem push. A frase do módulo no §5.1 **não muda**; o integrador troca só o link do Painter por este ficheiro.

## 1. Identidade

| Campo | Valor |
|---|---|
| Branch | `line/PainterWatercolor` (worktree `Worktrees/line-PainterWatercolor`) |
| Merge-base com main | `effe04a1c9c9fb60b772d4a3558c70af6c2e1360` (leia o `main` no ficheiro) |
| Commits | 60 (`git log --oneline main..HEAD`, a fonte) · 345 ficheiros · +9383 −1389 (`git diff --stat main...HEAD`) |
| Cabeça no fecho | ver `HEAD:` no topo (preenchido pelo coordenador) |

## 2. O que foi feito

- **Smooth Edges por traço:** valor na célula (`AparenciaCelula`), pousado com o traço, lido na composição. 25 degraus (múltiplos de 0,04: pedir 0,5 pinta 0,52). Custo com valor 0 igual ao de antes; com 0,12 mais rápido que antes: 29,47 vs 38,99 ms (tela 4096²), 1,41 vs 1,74 ms (retângulo 300²). Pior caso, 0,5 sob um traço a 0: 126,5 e 9,32 ms. Knob no cartão do pincel (saiu de Paper), na macro, gravado no preset. Decisão do dono 04/11: [memória](../../../project-memory/project_smooth_edges_per_brush_owner_decision_2026_11_04.md).
- **Wet Liner e Wet Wash** regravados pela receita (Smooth Edges 0,12, Edge Darkening 0).
- **Grain à mão** (armar, trocar, tirar, Same as Paper) sem mudar a quantidade de tinta (Bug #61), mais A1/A2/A4/A5/D1/D4 da auditoria (Bug #65).
- **Bug #64** (imagem velha apagava a sombra do traço; região do Smooth Edges curta), **#57–#60**, **#59** (Bricks/preview pela área).
- **E11 (placa):** `placa_sabe` aceita `Pigmento` com meio e pasta; recusa Pigment mix, chão como força, Dissolve, EdgeSoftMass, Real Oil e opaca. Custo na RTX 5,7→9,2 ms/passo a 4K sob carga. **D2:** cura fora da faixa (descida de +37 ms 1×/40 passos, só no caso de meia folha seca).
- **Portão reforçado**, 14/14 mutações. **Seis ficheiros partidos** para caber nos tectos de LOC. `presets_matriz.py` volta a correr. Grelha do portão-mãe versionada ([`portao_mae/`](material_2026-11-05/portao_mae/)).
- Mutação: #61 e R23 14/15, a sobrevivente `eA_vis` curada com `a_hardness_esta_na_borracha_do_wet_paint_e_age`; `q1/q3` sangram com o portão novo; a mutação `sombra` no `esvazia` **sobrevive** (hipótese: o grão cobre).

## 3. Superfície de colisão e contadores (DELTA)

| Item | Delta / valor |
|---|---|
| `PROJECT_SCHEMA` | 216, sem mudança |
| `KNOB_COUNT` | 142 → 144 (`TextureAcrossOwn`, `SmoothEdges`; conte no código: `ph2d-wet-paint/src/tuning.rs`) |
| `CAMPOS_DO_PINCEL` | 42 → 43 (`wetpaint_preset/campos.rs`) |
| `Dependente::TODOS` | confirme no código (`ExpoenteAtravesSemProprio`) |
| Crate nova | `ph2d-det-math` (`libm =0.2.16`); `ph2d-wet-gpu` passa a depender de `ph2d-paint-tinta` |
| Chaves gravadas | `smoothEdges`, `brush.grao_aplicado` |
| Saem | `BrushSettings::wet_smooth_edges`, `SMOOTH_EDGES_DE_FABRICA`, `PapelPreset::smooth_edges` |
| Memória do motor | `AparenciaCelula` 24 B/célula (+4); +1 B/célula de plano só com tecto > 0 |
| Contratos §6 | intocados (Nodes, Tools, Vector) |

## 4. O que um merge pode partir

- Outra linha que mexa em `AparenciaCelula`, no pouso (`pousa_o_dab.rs`, **694 de 700**: o tecto de LOC soma entre linhas, a cura é cortar), em `smooth_edges*.rs`, `Knob`/`KNOB_COUNT`, `campos.rs`, ou que leia `wet_smooth_edges`/`PapelPreset::smooth_edges` (símbolos removidos).
- Regravações de miniaturas/larguras/pinos hash por outra linha: conflito resolve-se pelo lado com a lei vigente; regravar só com o dono.
- `os_outros_meios_nao_mudam` foi regravada 1× (dono avisado 04/11).
- Registo de componentes/schema: nada a recontar. `git cherry main HEAD` antes de rebase.

## 5. Prova de fecho

Sobre `b6af47cbbe`: fmt, clippy do workspace, machete, standalone, tectos de LOC, censos 132/132, alarmes de pincel e placa por nome: verdes. A suíte completa e o `nextest-impacted` são do integrador.

- ⚠️ **Dois flakes novos, FORA de `FLAKES_DE_CARGA.md`:** `o_fio_da_tela_nao_paga_a_otica` (`chamadas == 0`, 3/3 sozinho; código vizinho desta linha, pode ser contador partilhado: não investigado) e `o_fan_brush_pinta_na_tela_grande_o_que_o_preview_mostra` (só 1/1 sozinho). Reprovaram na corrida por filtro, load 12–22. Vermelho deles na suíte: confira antes de culpar a carga. Listado: `a_agua_pelo_compositor_e_a_tela_de_casa_ao_byte`.

- Já medido nesta linha: o gate `o_indice_da_memoria` 3/3 verde (`bash scripts/cargo-test-narrow.sh ph2d-editor-core o_indice_da_memoria`), `MEMORY.md` 17 601 B (tecto do gate 22 000; alvo 17 KB, ver §9).
- `ph2d-tool-painter` inteira nunca ficou verde numa só corrida, por 2 flakes de carga (§8); passam sozinhos 3/3.

## 6. Decidi sozinho — confira (missão autónoma; o quê · por quê · foto · como reverter)

Material em [`material_2026-11-05/`](material_2026-11-05/).

1. **Os outros 75 pincéis com Smooth Edges 0.** Era a proposta sem resposta; a recomendação vale. Reverter: valor por pincel.
2. **Wet Liner e Wet Wash regravados**, alarme aprovado por ordem da missão (`583cd4167b`; tabela em `3581e24dc5`). Largura do Wet Liner 3,72/14/52 → 4,14/15,71/53; do Wet Wash 3,71/13,99/52 → 3,82/14/52,13. Fotos: [`alarme/`](material_2026-11-05/alarme/). Reverter: `git revert` desses dois e de `0cec47860d`.
3. **Trocar o Grain em facas fica abaixo de 0,9:** Knife Textured + Voronoi 0,83 · Knife Smooth 3 + Voronoi 0,85 · Knife Grainy + Noise 0,86, + Voronoi 0,82. Forçar apagava a textura escolhida; o portão alargou para massa 0,8–1,1 pela medição (escrito no #61).
4. **Grain à mão em tintas que saturam** (Ink Bullet, Dip Pen, Marker Bullet, Broad Nib): massa 1,11–1,95, ecrã 0,93–1,00. Tirar: Knife Dry massa 1,26 (ecrã 1,00), Halftone ecrã 1,16; Filbert armar Clouds 1,108. Wet Liner/Wash ficam na rota antiga (0,875–0,98). Grain Colors à mão num pincel de cerdas pintam através das cerdas.
5. **Smooth Edges não anda com Smear/Blend/Water** (mantido, A3). [`smooth_edges_no_arrasto.png`](material_2026-11-05/smooth_edges_no_arrasto.png): corpo macio, arrastado em riscas de textura; só a ×4 se vê o degrau.
6. **Preset de Borracha já gravado deixa de aplicar os outros knobs** (ex.: `StrokeFilm`); a Hardness continua partilhada entre Borracha e Pincel (resto do A5, ABERTO).
7. **Macro:** mudar o Smooth Edges depois de pintar não reproduz o efeito retroativo; macro antiga que arma um Grain toca a quantidade nova; macro gravada com ferramenta da água não escreve os knobs partilhados (lido, não medido).
8. **Bug #54:** [`branco_em_papel_de_cor.png`](material_2026-11-05/branco_em_papel_de_cor.png): a lei nova resolve (154/53/72 → 0/0/0 no laranja), MAS Gouache Opaque 0,95→0,84 e Dabber mais cinzento. Recomendação: fecha com a lei `pigmento` e não se estende à velha; a lei J8 já não existe (`702e98f913`).
9. **Knife Stain muito pequeno:** [`knife_stain_pequeno.png`](material_2026-11-05/knife_stain_pequeno.png); Size 1,5 e 2 pintam igual ao motor 2,8: ponteado azul. Só registo.
10. **Maquete F5 da ajuda ilustrada:** não feita, aberta. **Variar a borda pelo papel:** [`borda_pelo_papel.png`](material_2026-11-05/borda_pelo_papel.png), inconclusiva (⛔ as colunas «sim. orla» são artefacto e NÃO são proposta; só as do motor valem).

## 7. E12 / E13 NÃO feitas (limite do dono)

Tabela e plano: [`E9_portao_mae.md`](../Arquitetura/E9_portao_mae.md) §9; grelha em [`portao_mae/`](material_2026-11-05/portao_mae/). A lei velha dá 0. Placar: **401 padrão, 347 liso em 810** (01/11: 437 e 398; `target/e9_0111` perdido, comparação só por pincel). Dabber 4 +44 furos (`77a1e6cd58`, certeza alta não medida), Knife Smooth 3 +18, Knife Grainy +14, Knife Dry −19. Causa: `reprova` dispensa furos só se o par a pressão 1 tiver < 1 %; quem melhora cruza o limiar. Knife Textured a pressão 0,2 é defeito provável antigo. Memória: [`project_e12_blocked_by_mother_gate_ruler_2026_11_05.md`](../../../project-memory/project_e12_blocked_by_mother_gate_ruler_2026_11_05.md). **Não apague a lei velha antes de refazer a régua e do olho do dono.**

## 8. ABERTO

- `bancada_da_agua` não medida (só corre na fila).
- `render_texture_layer` e Grid/Crosshatch num ponto (#59).
- `regua_do_grao` com preset trocado: janela de um tique (lido, não medido).
- `sonda_fotos_das_variantes` (grafite) falhou numa corrida de filtro amplo; não investigada.
- Mutação `sombra` sobrevive.
- `scripts/prova-libm/prova.sh` noutro Linux, à noite: não corrida.
- `ph2d-tool-painter` inteira nunca verde numa só corrida, por 2 flakes de carga: `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` e `o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que_paga` (3/3 sozinhos, load 35–39). `the_water_advances_with_no_tick_at_all` passa sozinho (load 45). Candidatos a [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md); não promovidos.
- Sondas `sonda_grain_a_mao_ab` e `sonda_custo_da_media_do_grao` ficam `#[ignore]`.
- Bug #63: a sedimentar bifurca a 1 ulp na própria CPU (15,9 níveis em 75 células; CPU nominal no ramo B, placas no ramo A; o portão mede contra o ramo mais perto; folga curta, fração 8,3e-4 contra 1e-3). Ver [BUGS #63](../BUGS_painter.md).
- Bug #65: A3 (decisão do dono, recomendação manter) e a Hardness partilhada (A5).
- **Defeito da porta:** `prazo-do-agente.sh` recusa até `fila-pesada.sh pôr 'PH2D_REGERA_…'`, o comando que ele próprio manda usar; mandar a um subagente é impossível, o coordenador corre.

## 9. Incidentes (honestos)

- O coordenador correu `fila-pesada.sh correr` sem ver a fila e arrancou a `corre_f0.sh` velha, de 4 h, por ~10 min; parada, sem efeito na árvore. Reescreveu os logs de 01/11 em `…/5007c508…/scratchpad/e9full2` (conclusões em E9 §8). Memória: `feedback_clear_the_heavy_queue_before_running_it.md`.
- Dois subagentes correram por engano scripts de outros agentes no scratchpad partilhado: um duplicou um bloco em `grid/aparencia.rs` (removido antes do commit, status limpo), outro parou no assert; alguns logs do scratchpad foram sobrescritos. Um usou `prova_receita.sh` por fora de `ph2d-run.sh`.
- Índice da memória: estava em 20 869 B; desceram 13 linhas para `reference_topic_communication_and_decision_lessons.md` (agora 90 entradas, o número do índice foi atualizado). Uma linha com 4 ponteiros voltou ao índice porque o piso do gate é 115 ficheiros apontados (eram 111). 17 601 B fica acima do alvo de 17 KB e abaixo do gate de 22 000.

## 10. Para o integrador (resumo)

`PROJECT_SCHEMA` 216 · `KNOB_COUNT` 144 · `CAMPOS_DO_PINCEL` 43 · crate `ph2d-det-math` · `git cherry main HEAD` · suíte completa e impactados são dele · `pousa_o_dab.rs` a 694/700 · depois do push: `gh workflow run spike.yml --ref main` · §5.1: trocar o link do Painter por este handoff, frase sem mudança.

## 11. Perfil do loop do agente (`bash scripts/agent-loop-profile.sh`, 05/11)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
  ✓ paralelismo de ferramenta              2.15/passo   alvo: >= 1,5  (48% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                 42   alvo: <= 800
  ✗ cargo test : cargo check                 104 : 20   alvo: <= 1,0  razao 5.2x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.7%   alvo: <= 2%  (9 de 519)
  ✗ contexto relido por passo (media)         265 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               68 mil   alvo: <= 80 mil
```

## 11b. Na integração de 05/11 (integrador)

- **Bug #66, o congelamento do smoke do dono:** curado na árvore combinada, não nesta linha ([BUGS](../BUGS_painter.md), [provas](material_2026-11-05/congelamento_pool/)). O defeito já estava no main (`5c1c5a8aa6`). Portão `o_motor_nao_se_espera_dentro_do_pool`; o impasse natural não se reproduziu no app, o vermelho determinístico é o do guarda em `bring_home`.
- **Cruzamentos com a NovaUI:** o cartão Paper ainda mostrava o Smooth Edges (`5cbcbaa00d`); `ph2d-render` passou a `det_exp` para bater com a cópia canónica. `ph2d-render` continua fora de `o_motor_nao_chama_a_libm`.

## 12. SMOKE DO DONO (amanhã) — em passos

Binário: `host-desktop` em `--profile smoke`. Para cada teste, abra o programa assim:

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run --profile smoke -p ph2d-host-desktop`
2. Abra o Painter e escolha a ferramenta Wet Paint. Nenhum teste precisa de cena pronta: pinte direto na tela.

### A. O controle novo «Smooth Edges»
1. Escolha um pincel (por exemplo Wet Round) e abra o cartão do pincel.
2. Procure a linha **Smooth Edges**, logo abaixo de **Edge Darkening**. Ela já não está no cartão Paper.
3. Deixe em 0, faça um traço. Suba para 0,5 e faça outro ao lado.
4. Tem de acontecer: o segundo traço tem a borda mais macia, sem degraus. Mexer no controle **não muda** o que já está pintado.
5. Deu errado se: o controle não aparece, ou ao mexer nele o traço antigo muda.

### B. Wet Liner e Wet Wash
1. Escolha **Wet Liner** e depois **Wet Wash**; pinte um traço simples com cada.
2. Tem de acontecer: borda macia, sem cordão escuro grosso. Fotos de referência: pasta [`alarme/`](material_2026-11-05/alarme/).
3. Deu errado se: ainda há um cordão escuro grosso na borda.
4. **Pergunta:** o 0,12 de fábrica no Wet Liner está bom?

### C. Grain à mão
1. Escolha **Wet Round**. No cartão **Grain**, arme um Grain (por exemplo Noise), depois troque por outro e depois tire.
2. Tem de acontecer: a quantidade de tinta do traço não muda a cada troca; só a textura.
3. Em **Knife Textured**, troque o Grain para Voronoi: fica um pouco mais claro (cerca de 0,83 do normal), de propósito. Não pode sumir a textura.
4. Deu errado se: o traço fica bem mais claro ou mais escuro a cada troca, ou a textura some.

### D. Grain na Borracha
1. Escolha a **Eraser**, arme um Grain e apague sobre tinta.
2. Tem de acontecer: apaga com a textura do Grain. Troque o preset da Borracha e volte ao pincel: o pincel tem de continuar igual (com o Smooth Edges dele).
3. Deu errado se: a Borracha apaga liso mesmo com Grain armado, ou o pincel volta mudado.

### E. Macro com Grain
1. Em **Macros**, grave um traço com um Grain armado, pare e toque.
2. Tem de acontecer: o traço tocado é igual ao gravado.
3. Deu errado se: sai mais fraco ou sem o Grain.

### F. As decisões que tomei sozinho (confira cada uma)
Cada item abaixo está no §6 com a foto. Olhe e diga «fica» ou «volta»:
1. Todos os outros pincéis com Smooth Edges 0.
2. Wet Liner e Wet Wash mudaram de largura (fotos em [`alarme/`](material_2026-11-05/alarme/)).
3. Facas com Grain trocado ficam um pouco mais claras.
4. Tintas que saturam (Ink Bullet, Dip Pen, Marker Bullet, Broad Nib) com Grain à mão.
5. Smear/Blend/Water não ganham Smooth Edges: [arrasto](material_2026-11-05/smooth_edges_no_arrasto.png).
6. Papel colorido: [branco em papel de cor](material_2026-11-05/branco_em_papel_de_cor.png) (o Gouache Opaque ficou mais fraco).
7. Knife Stain muito pequeno: [foto](material_2026-11-05/knife_stain_pequeno.png).

### G. Perguntas para você
1. Nas suas fotos das manchas do Wet Liner, qual era a **Hardness** (fábrica: 100 %)? A tinta estava molhada ou seca? As manchas só aparecem abaixo de 100 % ([foto](material_2026-11-05/manchas_liner_hardness.png)).
2. Qual é a sua regra para os «furos» (E12)? Sem ela a lei antiga não sai.

**Deu errado de forma geral:** o programa não abre, ou fecha ao trocar de pincel. Anote o pincel e o passo.
