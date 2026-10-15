# CONTINUAÇÃO — `line/motion-value`: os ABERTOS do doc 132 (14/10) — o `Scene` com imagens NO DISPOSITIVO

> Continua o [ZERAR_132](HANDOFF_CONTINUACAO_line_motion_value_2026-10-13_ZERAR_132.md) (já no main). Plano, leis,
> tabelas e recusas: [doc 132](../132_zerar_as_seis_pendencias.md) (§4.1 item 3, §7).
> **Este ficheiro SUBSTITUI o ZERAR_132 como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, HEAD `1fd9047f5f` (+ o commit de docs), base =
  `main` `0aab2c1cb6`. **Por integrar** (só por ordem do Enio).
- `MAX_DEMO_LEVEL = 142` (`motion_state_demo_router.rs`, era 141): cena nova `=142`. Outra linha que crie cena tem de
  recontar o número no roteador.
- Superfície de colisão: `ph2d-shape-gpu` (`Corrida::ordenada`, `ShapePass::com_segundo_meio`, `pass_ordenado.rs`,
  `ordenado.wgsl`, WGSL das imagens partido em meio + entradas), `ph2d-fx-cells`, `ph2d-app-motion`
  (`MotivoDoVello::CenaComImagens` e `RECUSA_SCENE_NO_DISPOSITIVO` SAÍRAM; `ordem_das_linhas`), roteador de demos,
  `motion_state_imagens_scene_demo.rs` (novo), testes do censo de avisos. Um merge pode partir: quem referencie
  `CenaComImagens`/`RECUSA_SCENE_NO_DISPOSITIVO`; quem crie a cena `=142`; quem contasse os sítios do censo (384).

## O que fechou (14/10; contadores como DELTA sobre o main `0aab2c1cb6`)

| commit | o quê |
|---|---|
| `9e720a64db` | censo dos avisos: a variável segue até quem a liga; 5 sítios sem chave → 0 com gate; o leitor de corpos tomava o `;` de `[f32; 2]` por declaração (`import_ase` sumia); as 4 notas do import `.ase` nunca medidas; o cabeçalho mandava correr `--lib toast_reading_census`, que casa 0 testes. Censo: 384 sítios, 434 → 464 frases, 0 cortam |
| `a371f6e1e6` | sonda `mede_o_que_a_composicao_ja_da_ao_tween` recontada: (A) `e_uma_duracao(ArgKind)` sem `_`; (F) as arestas do Tween; 6/6 |
| `13a06f47e4` | o `Scene` com imagens NO DISPOSITIVO: `Corrida::ordenada`, `ShapePass::com_segundo_meio`, `ordenado.wgsl` (`vs_ordenado`/`fs_ordenado`), deslocamento no `first_vertex` com viés 2^28; `ordem_das_linhas` linha a linha na corrida ordenada; `an_each_shape_step…` compara pela chave (partido desde `cfc4d5911c`) |
| `c28534c7ac` | **DEFEITO achado pela prova de mutação:** `registo_de(0xffffffff)` achava o ENCHIMENTO dos handles (`u32::MAX` até 4) como geometria ⇒ com < 4 geometrias a imagem da corrida ordenada nunca desenhava. O gate aceitava 501 px com a barra de 5 % das estrelas (eram as imagens em falta). Cura na fonte; barra apertada a 4·4·0; gate novo `o_scene_com_imagens_pelo_dispositivo_segue_a_faixa_deslocada` (24 casos: folgas (0,0)(3,3)(3,1)(1,3) × imagem em cima/baixo × Soma/Multiplica/Tela) |
| `cfd4dc046b` | guarda EXACTA: a corrida ordenada junta-se só a vizinha alinhada (a velha era larga e sem gate) |
| `3d1f4b3c93` | fmt + `needless_borrow` do censo (clippy `-D warnings`) |
| `1fd9047f5f` | cena `=142`, `MAX_DEMO_LEVEL` 141 → 142, `motion_state_imagens_scene_demo.rs` + testes; legendas em PT sem acento como as outras cenas de ciclo |

**Sonda** `sonda_o_scene_ordenado_no_dispositivo` (1080p, médias a alternar linha a linha, mínimo de 12 blocos, carga
~2), 1k/4k/16k: CPU por linha `9,35 / 49,5 / 206,5 ms` · ORDENADO `0,204 / 0,407 / 2,19 ms` · comutativa (2 desenhos)
`0,265 / 0,524 / 2,46 ms` ⇒ 46–122× a lei de antes; 0,77–0,89× a comutativa. Kill-criterion fixado antes: ≤ 1,5× a
comutativa. ⚠️ Medida ANTES de `c28534c7ac` (a imagem não desenhava nas linhas de imagem): o número do ordenado
deve ser **re-medido** (ABERTO 1).

**Decisões fechadas.**
- Cadeia 16·4096 do `corrige`: fica a Escolha (`select`) em todos os casos. Mínimo (`+9 %` Matriz) e mediana (`−3,4 %`)
  discordam de sinal, a régua não separa, e um 2.º kernel só para cadeias seria código duplicado por ganho dentro do
  ruído; a Escolha ganha até 43 % na nuvem. Linha na tabela de recusas do doc 132 §7.
- «3 testes a ~150 s do tecto de 180 s» NÃO são defeito: são `#[ignore = "sonda: …"]` (por MOTIVO), apanhados por uma
  corrida de fecho improvisada que separava sondas pelo NOME. O portão certo do Motion é
  `scripts/motion-gate-placa.sh` (lê o prefixo `placa:`). No repo há 484 testes sonda-por-motivo sem marca no nome ⇒
  separar pelo nome é régua que mente.

## Mutações (árvore restaurada)

Censo M1..M7: 7/7 RED (agente). Desenho ordenado: M1 deslocamento · M2 ramo das imagens · M3 ramo do passe · M4 ordem
das células · M6 troço mudo · M7 `registo_de` — RED; **M5 (guarda de junção) SOBREVIVIA** → curada em `cfd4dc046b`,
e a mutação da guarda nova é RED.

## Aberto

1. Re-medir `sonda_o_scene_ordenado_no_dispositivo` depois de `c28534c7ac` (kill-criterion ≤ 1,5× a comutativa).
2. O detalhe perdido do aviso «Vector-path shapes filter as one» — o dono ainda não respondeu se o quer numa dica/tutorial.
3. Legendas das cenas de ciclo em PT cru, não i18n (padrão herdado): decidir se passam a chaves.
4. `docs/Components/ferramentas/fotografa_cena.sh:108` (NÃO é da linha): comentário com crases dentro de heredoc sem
   aspas é executado (`[env]: comando não encontrado`) — inofensivo hoje.
5. Flake da `line/PainterWatercolor` e falha `g4_…` da Luz2D são de outras linhas (ver o ZERAR_132).
6. A corrida «GPU não-sonda» do repo inteiro não existe como script — só o Motion classifica pelo motivo.

## Smoke ao dono (cena fotografada, scratchpad `cena142b.png`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && PH2D_GPU_COOK_DEMO=142 PH2D_MOTION_ROUTE_LOG=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Na tela: quadrados laranja (imagens) com uma estrela azul INTEIRA por cima de cada um.
3. No cartão **Output**, depois do **Combine**, mude **Blend With Scene** ↔ **Copies**: em Copies a estrela clareia e não tapa.
4. O terminal diz «pela PLACA (do dispositivo)»; o log: `[formas] 513 copias de 2 geometrias pela PLACA (do dispositivo)`.

Deu errado se: o laranja tapa as estrelas; faltam quadrados ou estrelas; o terminal mostra `[motion-route] CPU:`.

## Fecho

Verificador, HEAD `cfd4dc046b` + fmt: nextest-impacted `PH2D_SUITE=fecho` **17 071/17 071** (nem a Luz2D `g4_…` nem
flakes) · clippy forma do `ship.sh` ✓ (após `3d1f4b3c93`) · fmt ✓ · check warnings=deny ✓ · `scripts/motion-gate-placa.sh`
59/59 · `ph2d-shape-gpu` + `ph2d-fx-cells` de placa em série 21/21 (em PARALELO os 11 de placa da `ph2d-fx-cells`
reprovam sem mensagem — contenção da placa, não código: `--test-threads=1`).
Perfil (`agent-loop-profile.sh`): paralelismo 1,17/passo (alvo ≥ 1,5) ✗ · test:check 3,2× ✗ · contexto 295 mil/passo ✗ ·
edições sem conferir 1,4 % ✓.
