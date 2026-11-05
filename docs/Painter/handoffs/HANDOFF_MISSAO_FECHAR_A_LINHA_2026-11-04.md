# HANDOFF — missão autónoma: fechar a linha PainterWatercolor (04/11, noite)

Linha `line/PainterWatercolor`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`. HEAD no
fim desta janela: o commit deste handoff, 21 commits sobre `92e78fe4bd`. O `main` andou (`effe04a1c9`, GEF 22): rebaseie
primeiro. Nada integrado, nada enviado.

## 0. A ordem do dono

> «A partir de agora vc está sozinho e sua missão é finalizar tudo que está em aberto e fechar a linha. Depois escrever
> handoff de integração e sinalizar a janela ph2d-44 [c6124b] que será o agente integrador. Não tire dúvidas, não peça
> autorizações, não peça smoke. Faça tudo até o fim e entregue o handoff. Amanhã faremos o smoke.»

O que isto muda:

- **Sem perguntas.** Cada decisão de produto que caberia ao dono: escolha pela recomendação abaixo, escreva-a no handoff
  de integração numa secção «Decidi sozinho, confira amanhã», com a foto e como reverter.
- **Regravar alarmes de pincel fica autorizado nesta missão**, pela ordem acima, depois de `target/relatorio-pinceis/index.html`
  ser olhado por si e copiado para o material do handoff: o dono aprova ou recusa amanhã, e a reversão tem de ser fácil
  (um commit só para as regravações). Commit próprio por família de pincéis, com a lista.
- **Continua proibido:** `git push`, integrar, `ship`, correr a suíte completa ou `nextest-impacted` na linha
  (CLAUDE.md §2: são do integrador), `kill` por pai de órfão, `spectacle` em `kwin --virtual`.
- A linha acaba em: handoff de integração + binário de smoke compilado + mensagem à `ph2d-44 [c6124b]`
  (`SendMessage`, a ferramenta deferida; carregue-a com `ToolSearch` `select:SendMessage,ListAgents`). Confirme o nome
  com `ListAgents` antes de enviar: em 04/11 ela existia e estava idle. Se tiver mudado de nome, procure o sufixo `c6124b`.
  Depois dessa mensagem, PARE.

## 1. Como trabalhar: agentes ajudantes

⛔ Esta janela anterior terminou a ~340 mil tokens por passo, porque leu muito em casa. **O custo é reler** (CLAUDE.md §2).
Na nova janela:

- A janela principal **decide, desenha e escreve os briefings**. Lê só o essencial.
- `explorador`: toda pergunta que atravessa vários ficheiros. `verificador`: `check`, testes por nome, gates (devolve só as
  falhas). `general-purpose`: blocos inteiros de implementação com escopo fechado, um por etapa (Smooth Edges por traço ·
  Grain à mão · E11 · E12a · E12b · E13). `mutacao`: provas de mutação. `documentador`: BUGS, handoff, memória, §5.1.
- Um agente por vez em cada ficheiro: em 04/11 um verificador correu enquanto a janela principal editava e deu `exit 2`
  falsos. Agentes que editam a mesma crate vão em série ou em `isolation: "worktree"` com `CARGO_TARGET_DIR` próprio
  (⛔ nunca partilhado entre worktrees: troca `.rlib`).
- Teto imposto de **45 min por agente** (hook `prazo-do-agente`). O pesado de verdade vai por `scripts/fila-pesada.sh`.
- **Todo briefing leva as formas proibidas** ([memória](../../../project-memory/feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt.md)):
  `cd X && …` em linha composta com `bash -c`/`sh -c`, `rm -rf` com variável ou glob, `kill` de PID alheio, escrever em
  `.git/`, `.claude/` ou rc da shell. Um pedido de permissão congela a missão.
- Todo agente começa com `pwd` + `git branch --show-current`. Toda escrita na worktree, nunca no primário.
- Janela nova a cada onda grande também é legítimo: antes de trocar, deixe o estado neste ficheiro.
- Zsh: `$VAR` com vários nomes NÃO se parte (em 04/11 um filtro correu 0 testes e pareceu verde). Escreva os nomes.
  `cargo-test-narrow.sh` aceita UM nome e só devolve o resumo: para ler `println!`, use
  `bash scripts/ph2d-run.sh cargo test -p <crate> --profile smoke --lib <nome> -- --nocapture --test-threads 1`.
- Máquina partilhada: em 04/11 o load esteve entre 35 e 210 por outras linhas. Lentidão não é defeito; tempo medido sob carga
  não vale nada (FLAKES_DE_CARGA.md).

## 2. O que já está feito e NÃO se refaz

Em commits na linha (mecanismo e números: [BUGS_painter.md](../BUGS_painter.md), #56, #57–#62):

| Tema | Estado |
|---|---|
| Bugs #57–#60, `ph2d-det-math`, pincéis de 04/11 | aprovados pelo dono ([handoff anterior](HANDOFF_CONTINUACAO_bugs_57_a_60_e_pinceis_2026-11-04.md)) |
| ⚠️ `os_outros_meios_nao_mudam` | regravada 1× (Watercolor 1 byte, Impasto 2, em 2,56 M, 1 nível): o dono JÁ foi avisado em 04/11 |
| E10c2 | pontas fechadas ou confirmadas; só sai no handoff como estado |
| R23 | feita: `Knob::TextureAcrossOwn` (seletor, KNOB_COUNT 143) + `textureAcrossExp` 0..1,5; a linha esmaece (`Dependente::ExpoenteAtravesSemProprio`); chave velha `tuning::chave_antiga_atraves`; **smoke do dono: ok** |
| Bug #56 | curado na régua nas duas leis (`PH2D_F3_LEI=velha|pigmento`, `f3_medicoes.rs`). **Smoke: «no cruzamento tem boa aparência»** |
| Bug #61 | «Same as Paper» mantém a quantidade: `LoteDoDab::{media_do_papel, cerdas_sob_o_papel, ganho_do_papel}`, `lote::media_do_grao`; gates em `wetpaint/same_as_paper_volume_tests.rs` (8 pincéis 0,97–1,02×; rampa 0,99–1,00×; Grain próprio 0,95–1,00× em 6). **Smoke: «melhorou»** |

⛔ **Recusas medidas, não reconstruir:** Body 0 no Wet Liner/Wet Wash (ondas de 2 px no miolo; o Wash sobe a orla a 1,25) · o papel
substituindo as cerdas sem compensar (até 20×) · cerdas × papel para pincel com Grain próprio (quase os apaga) · Smooth Edges 0,12
na tela inteira (pó de pastel 14,6 grãos/100 px contra 56 do Rebelle; grão de 2H/HB fora da faixa; Wet Flat fino ÷ largo 0,64).

## 3. O que falta, por ordem

### 3.1 Smooth Edges por traço e por pincel (decisão FINAL do dono 04/11)

[Memória](../../../project-memory/project_smooth_edges_per_brush_owner_decision_2026_11_04.md). Palavras dele: «por traço mesmo e cada
pincel deve ter o seu. assim não perde detalhes». Revoga a D1 de 27/10 (`docs/Painter/49_realshader/AUDITORIA_W3b.md:5`), mas fica dela:
**age na cor E no relevo**. Mexer no controlo NÃO altera o já pintado.

- **Padrões decididos:** Wet Liner e Wet Wash `0,12`; TODOS os outros `0` (a leitura proposta ao dono, sem resposta; é a que não perde
  detalhe). Sai `SMOOTH_EDGES_DE_FABRICA` e o 0,12 geral, os `smooth_edges = 0.0` postos nos arranjos dos testes (commit `12f5283987`)
  e a versão com `PapelPreset::smooth_edges` (`tool/papel_texturizado.rs`).
- **O que há hoje** (levantamento `explorador`, conferir as linhas): escalar `WetPaintState::smooth_edges` (`tp/wetpaint.rs:125`).
  Setter `wetpaint_settings.rs:568`, `wetpaint_settings/tela_e_oleo.rs:8-24`. Efeito só na composição CPU: `wetpaint/smooth_edges.rs`
  (raio = valor × `RAIO_MAX` 6 px; tenda sobre `pa ≥ SS0`; peso `4·B·(1−B)` na orla; degrau de cor `DEGRAU_MIN/CHEIO`; linhas
  `LINHA_MIN/CHEIA`). Leitores `wetpaint/composite.rs:86`, `otica_factos.rs:31,80`, `oleo_no_quadro.rs:103`, `composite_otica.rs:100,128,163`,
  relevo `oleo_real::amacia_a_silhueta` em `CampoAltura::de_com`. Painel `paint_watercolor_paper.rs:303-324`, `snapshot.rs:479`,
  `brush_settings.rs:485`. Macro `macro_estado.rs:61,160`, `macro_track/portas_internas.rs:322`. Reset do Paper `watercolor_settings.rs:406`.
  Preview `wetpaint_preset/preview.rs:182,312`. GPU: a composição não está na placa.
- **Desenho:** o valor viaja na célula com a tinta, pelo padrão de `AparenciaCelula` (`wp/grid/aparencia.rs`: pousado no traço, mistura pela
  massa, arrasto/Smear/Blend não o movem). A tentativa de `65d6a6a16d` existe no histórico: `git show 65d6a6a16d` (`AparenciaCelula::suave`,
  `ForcaDaCelula`, `Params/Sim/WetEngineFacts::suave`, 4 B/célula); parta dela, sem acreditar nela (foi antes do relevo e da óptica fora da
  thread). O `raio` deixa de ser escalar: `alcance` e a região a recompor usam o MÁXIMO da região; `Suave`/`premul_do_pixel` lêem o da célula.
  Instantâneo, histórico, recorte e undo carregam o campo (a aparência já vai). `Knob::SmoothEdges` novo (Hidden, no cartão do pincel) grava no
  preset por chave normal; preset sem a chave = 0. Macro antiga: `wet.smooth_edges` vira o valor do pincel na reprodução.
- **Controlo:** sai do cartão **Paper**, entra no cartão do pincel do Wet Paint, com o padrão do Inspector (`feedback_a_field…`, memória). Os 7
  sites do seam (DIRETIVA §2) juntos. Portão por `ph2d-ui-testkit`. English only.
- **Portões a escrever (vermelho primeiro):** dois traços com valores diferentes na mesma tela, cada borda lê o seu; pintar com 0 ao lado do
  0,5 não muda um byte do 0,5; mudar o controlo não recompõe o já pintado; ida e volta do preset; desfazer; relevo do óleo acompanha; o valor em
  0 é ao byte o de hoje com 0 (`smooth_edges_no_zero_e_o_desligado_de_antes_ao_byte` continua). O pó de pastel volta a ~56.
- **Testes que mudam de sentido:** `grid_ratio_tests.rs:499-504,564-567,593,642`, `seam_wetpaint_opcoes.rs:140-173`,
  `seam_papel.rs:111-115`, `seam_wetpaint.rs:110,246`, `seam_watercolor_flow.rs`, `wetpaint_preset/tests.rs:250`, `oleo_arrasto_tests.rs:123,142`,
  `papel_texturizado_tests.rs:39`, `oleo_borda_w3b_tests.rs:298,363,401`.
- **Depois disto voltam a passar sozinhos** (os que reprovavam só com o 0,12 geral; confirme por nome): `uma_mistura_nunca_e_mais_clara_que_as_tintas`,
  `leis_sistemicas::o_traco_fino_pesa…`, `pastel_carvao::o_po_solto_cai…`, `grafite::os_grafites…`, `grafite::sem_a_orla…`,
  `oleo_acrilica::cada_oleo…`, `as_leis_da_escala::a_parede…`, `regua_da_largura::a_largura_nao_depende_da_fase…`, `regua_da_largura::o_rasto_do_smear_wet…`.
  Os dois últimos Wet Liner/Wet Wash podem acusar por terem 0,12: é real; cure pelo arranjo só se a régua medir outra lei.
- ⚠️ `lote::media_do_grao` faz 1 024 amostras por lote: **meça o custo** (MEDIR_VELOCIDADE.md), a régua do §0.0.

### 3.2 Wet Liner e Wet Wash (Bug #62)

Receita decidida pelo dono: **Edge Darkening 0 nos dois, Body de fábrica (0,16) mantido, Smooth Edges 0,12**. Mudar a receita em
`crates/ph2d-tool-painter/tests/it/familias/aguarela.rs` e regravar os ficheiros de `assets/brushes/WetPainter/Watercolor/Fluid/` pelo trilho das
receitas. Provas: [comparação com o Rebelle](material_2026-11-04/prancha_rebelle_x_ph2d.png), [variantes](material_2026-11-04/comparacao.png),
tabela no Bug #62. Diagnóstico do Rebelle: a altura da orla não é o defeito (Rebelle 1,23, nós 1,15); o nosso fio fica 3 px PARA DENTRO seguido de um
vale, e lê-se como contorno desenhado. Sem Edge Darkening o traço fica muito liso: **tentar variar a borda pelo papel fica em aberto, não simulado**; não
devolva o fio ([memória de 02/11](../../../project-memory/project_wet_liner_no_dark_edge_2026_11_02.md)). A régua antiga do fio do liner sai, se ainda existir, com o porquê.
**Manchas no miolo que o dono fotografou:** não reproduzidas em tela lisa. Reproduza no papel `Textured Paper`, com o traço do tamanho das fotos
(`docs` não as tem; são Wet Liner vermelho e preto, curvas, ~60–70 px, com Same as Paper possível). Se houver causa curta, cure; senão, deixe a foto no handoff.

### 3.3 Grain escolhido à mão (decisão do dono: «consertar»)

Hoje, num pincel de cerdas, armar um Grain à mão salta até 16,5× (coluna noise do #61): o Grain substitui o tapete.
⚠️ **Corrija a minha frase ao dono:** eu disse que o conserto «muda só 10 pincéis, só quando alguém trocar o Grain à mão». Essa frase só é
verdade se o desenho a cumprir: os 10 pincéis de fábrica com cerdas + Grain próprio (Knife Dry, Knife Smooth 3, Knife Stain, Knife Textured, Knife Grainy,
Metallic Flat, Soft Pastel, Scratch, Thick Impasto, Dabber 4) foram aprovados como estão e **têm de continuar ao byte** enquanto ninguém troca o Grain. D7
(`NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md:209`): pintam pelo grão; os outros pastéis têm Shape Noise e não entram.
**Desenho sugerido, unifica com o #61:** a quantidade referencia o **grão com que o pincel foi aplicado** (guarde-o ao aplicar o preset e no gravar):
- Aplicado sem Grain + artista arma um → as cerdas ficam como `texv`, o grão entra depois do portão, de média 1 (`cerdas_sob_o_papel`, `media_do_papel`).
- Aplicado com Grain G + artista troca por H → H no lugar dele, com `ganho = média(G)/média(H)` (`ganho_do_papel`).
- Aplicado com Grain G e não trocado → o de hoje, ao byte. Same as Paper é um caso de «trocar».
Gravar um pincel fixa a referência nova. Portões: a miniatura dos 10 ao byte; armar Noise/Clouds/Voronoi em Wet Drip, Wet Round, Wet Dry Brush e Filbert fica em
0,9–1,1×; a textura ainda desenha; o pincel gravado com o Grain novo reabre igual. Faça a sonda A/B antes de construir, como no #61.

### 3.4 Limpeza e aprovação da onda

Alarmes por nome `a_largura_de_fabrica_e_a_aprovada` e `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`: com 3.1–3.3 prontos só devem acusar Wet Liner, Wet Wash
(e o que 3.3 tocar: nada, se o desenho cumprir). Olhe o relatório. Regrave em commit próprio e copie as imagens para `docs/Painter/handoffs/material_2026-11-04/`. O Fan ainda grava a chave
`textureSizeExpAcross`; o leitor tolerante basta, não é preciso regravá-lo.

### 3.5 E11: a placa lê o meio e a pasta

Plano [F4_2 §6 e E11](../Arquitetura/F4_2_plano_wet_paint.md). Teste `gpu/tests` `paridade_da_tinta_por_placa`; `gpu/passo.rs:20 placa_sabe` aceita `Pigmento` só com ela verde; hoje o portão
`a_placa_recusa_a_lei_pigmento` (`tests/passo.rs`) prova a recusa e muda de sentido. A régua: «mesmo bit» é **por placa** (memória 24/10). `ph2d-wet-gpu`: `advect.wgsl`, `seca.wgsl`, `lib.rs`,
`seca.rs`, `passo.rs`, FMA no compilador de shaders. Placa: a vez pede-se sozinha pelo `ph2d-run.sh` (A_VEZ_DA_PLACA.md); o cache de shaders do driver antes de culpar o relógio. Sem `kill` em vigia.

### 3.6 Portão-mãe e E12/E13: apagar a lei velha

Condição do dono, que esta missão NÃO revoga: **só depois de o portão-mãe passar inteiro**.
1. Reconte `portao_mae_f0` (`tp/wetpaint_preset/portao_mae_f0.rs`; a grelha inteira pela `scripts/fila-pesada.sh`, UMA corrida intercalada). A última conta: **437/810 padrão, 398/810 liso, Velha 0**;
   reprovam só furos e degraus, e os MESMOS furos na Velha ([E9 §8](../Arquitetura/E9_portao_mae.md)). A pergunta é se a régua pede o que a lei velha também não tem. «furos condicionados» é o texto do plano.
2. Cure o que for defeito. Mudar uma régua exige medição escrita de por que ela mentia (⛔ «regra sem medição é confirmada por quem a segue»), jamais só para ficar verde.
3. Se ficar INTEIRO: E12a (`wp`) e E12b (`tp`+`gpu`) no mesmo bloco, E13 (§8, portões Rebelle fora, dono 31/10), `set_wet_lei_da_tinta` ligado por omissão. São 23 referências a `LeiDaTinta::Velha`,
   mais `wp/assenta*`, `colorops.rs:101-410`, `opacity.rs`, `ladeira`, `session.rs`, `authored_actions.rs`, `gpu/` velho.
4. Se NÃO ficar inteiro com um defeito fora do alcance da missão: **NÃO apague a lei velha**. Feche a linha sem E12, com a tabela do que reprova, as fotos e o plano. Isso conta como missão cumprida com
   um aberto declarado, e não forçar é a ordem do dono. Nenhum alarme de pincel pode ser engolido na troca de lei sem estar no relatório.
   ⚠️ Trocar a lei por omissão muda TODOS os pincéis: relatório inteiro antes, commit separado e reversível (o smoke é amanhã).

### 3.7 Decisões que seriam do dono: decida e registe

- **Bug #54 (branco opaco em papel de cor):** a decisão é pela foto, na E12. Se a lei nova resolve, descreva. Se não, a recomendação é estender a todo tinta a lei do Over Oil (J8, `BUGS_painter.md`), com prancha
  antes/depois em papel de cor para a sua conferência. Reversível.
- **Maquete F5 da ajuda ilustrada:** o dono disse que pode esperar. Não construa: uma prancha-maquete rápida, se sobrar folga, ou liste como aberta. Não atrasa o fecho.
- **Knife Stain muito pequeno** (abaixo do raio 2,8 o véu não atinge a régua): só registe, com foto.
- **Variar a borda pelo papel** no Wet Liner/Wash: simule se for curto; senão fica como sugestão no handoff.

### 3.8 Miudezas do fecho

- `presets_matriz.py` quebra com `AssertionError: (137, '142')` (agora 143): a regex não aceita cinco knobs de `tuning/defs.rs`; `F1_presets_de_fabrica.md` defasado desde 31/10. Cure.
- `caligrafia_marcadores.rs` 706 linhas e `texturas.rs` ~1040: partir por tema, teste a teste, se a catraca pedir; `wetpaint.rs` está em ~610.
- Padrão `Bricks` com degrau e pré-visualização do grão no painel lêem num ponto (#59): varra com `weight_area` se curto.
- `bancada_da_agua` −30 % de CPU com instantâneos: só se trivial. A fila pesada guarda `corre_f0.sh` velho.
- `E10h`: sem prova de mutação de que o censo mede a Hardness na borracha: `mutacao`.
- `project-memory/MEMORY.md` em 20 869 B, alvo 17 KB, gate a 22 000: desça entradas para os `reference_topic_*` (nunca apague), com o gate `o_indice_da_memoria` por nome; é o último antes do handoff, porque outras linhas o tocam.
- As provas de mutação das curas desta onda: #61 e R23 ainda não têm. O vermelho de #61 foi visto antes da cura; dê à `mutacao` a sentinela, o seletor e o ganho.

## 4. Fechar a linha

`/pd-linha-fechar` + DIRETRIZ §1.5.9 + `/pd-gate-fechamento`. Em resumo: gate batched 1× sobre o diff acumulado (`PH2D_SUITE=fecho`; testes por nome, fmt e clippy das crates tocadas, `censos-da-arvore-combinada`, 11 crates da det-math incluídas), auditoria com o modelo da
DIRETIVA §3 sem «compila, ok»; `bash scripts/agent-loop-profile.sh`; `rm -rf target/*/incremental`; a máquina limpa (`pgrep`, `fuser -v /dev/dri/*`).
Handoff de integração `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-05.md`: commits, o que foi medido, o que fica aberto, a linha do §5.1 do CLAUDE.md (no máximo 700 bytes, trocar o link) e o que o integrador tem de saber:
schema, `KNOB_COUNT 143→` novo, `Dependente::TODOS`, o crate novo `ph2d-det-math` (`libm =0.2.16`), a prova `scripts/prova-libm/prova.sh` noutro Linux pela noite, as fotos, o passo-a-passo do **smoke de amanhã em passos numerados** (comando com `cd`, onde clicar, o que tem de acontecer, como saber que deu errado), em PT-BR sem jargão, cobrindo tudo o que «Decidi sozinho» listou e os pincéis regravados.
Último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` duas vezes, a segunda colada no handoff (0 Compiling). Depois `SendMessage` à `ph2d-44 [c6124b]`: caminho do handoff, branch, HEAD, a ordem do dono, sem push até ele mandar. E PARE.

## 5. Ao dono

Ele lê amanhã. A mensagem final é curta e sem jargão (CLAUDE.md §0.8): o que ficou pronto, o que decidiu sozinho, o que não foi feito e porquê, o link do handoff. Imagens em link markdown clicável. Toda a escrita da UI em inglês.

## 6. Material desta sessão

[`material_2026-11-04/`](material_2026-11-04/): `prancha_rebelle_x_ph2d.png` · `comparacao.png` (variantes do cordão) · `fios_0_006_012_025.png` e `encontro_0_012_025_dif6x.png` (o Smooth Edges na mistura) · `f3_presets.txt` (para `PH2D_F3_PRESETS`, `PH2D_F3_BODY=0,0.16`) · `sonda.patch` (`sonda_prancha_rebelle`, não aplicada) · `mede2.py`, `prancha.py` (réguas da prancha). O Rebelle correu em tela virtual pela porta WebSocket: o teste dele **expira em ~2 dias** e ele deixou de ser a referência (31/10), logo só informa.
Sonda útil na árvore: `sonda_cordao_na_borda` (`PH2D_SONDA_OUT`), em `wetpaint/same_as_paper_volume_tests.rs`.
