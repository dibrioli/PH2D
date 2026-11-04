# HANDOFF DE INTEGRAÇÃO — NovaUI + NovaUI-Inspector + NovaUI-Bones (03/11)

> Integrador: a sessão que o dono nomeou em 03/11 («integre as linhas da UI nova no main»).
> `TIP_MAIN` = `<preencher: tip do main depois do ff>` · `CI_RUN` = `<preencher: link da corrida do CI>`.
> A `line/TrelloUITeste` ficou FORA (ensaio; ver [README](../README.md)).

## 1. O que entrou, em que ordem, e porquê

Base: `main` @ `b785a20353`. Ordem MEDIDA (não escolhida): trecho comum de 30 commits até `11b6952897`,
depois 6 + 18 + 7 commits próprios.

| # | Linha | Tip | Própria | Handoff da linha |
|---|---|---|---|---|
| 1 | `line/NovaUI` | `1be2e9bd20` | 6 | [GALERIA](HANDOFF_line_NovaUI_2026-11-03_GALERIA.md) |
| 2 | `line/NovaUI-Inspector` | `803d4be0e0` | 18 | [Inspector](HANDOFF_line_NovaUI-Inspector_2026-11-03.md) |
| 3 | `line/NovaUI-Bones` | `0511505993` | 7 | [Bones](HANDOFF_line_NovaUI-Bones_2026-11-03.md) |

Resultado: Inspector em cartões (coluna, cartões soltos, alfinete, corda), estilos de cor, galeria em
cartões, cartões do Bones. Decisões do dono: **T65** (NovaUI) e **T66** (Inspector) em
[01_decisoes_do_dono.md](../01_decisoes_do_dono.md). Flag: `PH2D_CARDS=0` volta ao painel de antes.

## 2. Superfície de colisão e contadores (DELTA)

- `PROJECT_SCHEMA` 215 → **216**; tripla **(216, 13, 29)**; `VEC_SCENE_SCHEMA_VERSION` 28 → **29**;
  `PROJECT_SCHEMA_SEM_ESTILOS` = 215 (degrau do projeto sem estilos de cor).
- Próximo ADR: **0198** (nenhum ADR criado nesta rodada).
- Cenas: `PH2D_VEC_CARTOES_SMOKE=1..7` (roteador `max_level` 7) · `PH2D_VEC_BONE_SMOKE=9` (Bones) ·
  `PH2D_GALLERY_SMOKE=1`.
- Suíte completa antes das curas: **31 677 testes, 21 vermelhos, os mesmos 21 nas duas corridas**.

## 3. Conflitos do merge

- `cards_linhas.rs`: uma só `glifo_do_segmento(s, com_icone)` — o tamanho de ícone do Inspector + a lei
  «a palavra se mantém» do Bones.
- `cursor_preso.rs`: ficou o ficheiro INTEIRO do Inspector (com `thread_local`). A condição extra do
  Bones (`pointer_lock_wanted() ||` para o cursor ↔) foi DESCARTADA: o cursor some durante o arrasto.
- `fase_selection_mirror_bone_focus.rs`: os dois blocos mantidos.
- `shape_build.rs`: a isenção morta do Shape Builder removida.

## 4. Curas por família (commits `ff5200e876`, `d49be37b30` e um a caminho)

- **Projetos antigos (6 testes em `shells/desktop`)**: as fixturas gravavam o byte final novo; agora
  `with_wire_layout(28, …)`; o projeto antigo abre e os dados comparam.
- **Scanner** `the_clear_run_verb_reaches_the_tape`: a âncora do bind mudou.
- `the_tail_of_a_block…`: `rows_moldura.rs` usa `control_gap_px()`; cauda 6 → 3 px.
- `so_a_porta_escreve_o_alvo_do_seletor`: novo `repoe_picker_target` ao lado de `set_picker_target` —
  restaurar do ensaio não é gesto (sem subir painel, sem largar alfinete).
- `nenhum_motor_publica_um_rotulo_cru`: tabela passou a const de módulo.
- `no_stale_frame_markers`: marcador morto removido.
- **Portas de tema**: `every_frame_goes_through_the_theme_door` declara 10 contornos (foco de campo/
  editor rico/data, quadrado do checkbox, flash do cartão solto, amostra de cor, marca do cartão, alvo
  da paleta da galeria, cores escolhidas e pega); `a_ring_painted_as_a_fill…` declara 1 (`Divider`,
  régua do texto rico); `toda_crate_que_pinta_campos…` declara 1 (olho da galeria, `NaoAnimavel`).
- **Noto Sans (D11) moveu medidas**: `a_caixa_de_numero…` (-0.5000 → -0.500),
  `as_duas_caixas_cabem…` ((245,1) → (245,0)), `nenhum_nome_de_barra_corta…`.
- ⚠️ **MUDANÇA DE TEXTO DE PRODUTO**: `toda_queixa_de_cartao_cabe…` — a queixa `off_the_wire` agora diz
  `Not wired to an Output: idle` (era mais longa). Avisar o dono.
- `dentro_de_um_troco…`: o `3 → 1` acima era um ESTADO TRANSITÓRIO da mola (ver «Cura 04/11 — relógio dos cartões»);
  a declaração da galeria voltou a **3** (o valor do main) e a varredura mede 3 de forma determinística.
- `a_altura_de_abertura…`: passa combinado; a catraca já subira com a NovaUI (inspector 1488 → 1507,
  Noto contra Inter; painter_layers 1593 → 1618, o nome quebra em vez de cortar).
- **Flake de carga, NÃO promovido**: `o_pen_down_manda_a_agua_herdada_pela_divida` passa 3/3 sozinho
  (loadavg 42, 14, 9) e reprova a loadavg 54–190. Reconferir na suíte final.

### Curas de 04/11 (`fix(integ 04/11)`, integrador sem o dono — critério: padrão-ouro)

- **CLAUDE.md §5 = 14 517 > 14 000 bytes** (`o_claude_md_cabe_no_orcamento`): o main já estava a 13 995 (5 bytes de folga),
  logo a entrada nova só cabe cortando narrativa de feature das vizinhas (Luz 2D, Quadro, Editor de scripts, Game Engine)
  e a da própria CartoesUI; §5 = 13 994. O teto NÃO subiu. Commit `080a9c7a69`.
- **Cura 04/11 — relógio dos cartões** (`f9c677db7d`): `cards_lista`/`cards_solto_some` avançam as molas com o `dt` do
  relógio de parede. Efeitos: o censo da galeria media 1/2/3 colunas na MESMA árvore (conforme a carga); `j3_import_o_aviso…`
  comparava duas pinturas do hero (190 vs 192 caminhos); 9 testes de cartões no hero (`a_tinta_de_base_no_cartao`,
  `a_lista_de_cartoes_flutua_no_hero`, `os_estilos_de_cor`, `o_bloco11_tema_por_copia`…) dormiam 5 ms por quadro e
  reprovam em máquina RÁPIDA (baseline sem a cura: 9 vermelhos a load ~15). Porta: `aquecimento::passo_do_quadro`/
  `com_passo_fixo`/`assenta_e_mede`; o arnês (`ph2d-ui-testkit`) pinta os 2 quadros de aquecimento a 0,1 s e o medido a 0;
  os testes do hero pintam a 1/60 s por quadro, sem `sleep`. Galeria: 3 colunas (idêntico ao main).
  ⚠️ Esperar a mola assentar em mais quadros mudou o Inspector armado (`a_regua_separa_controlos…` 1001 > 954) — por isso o
  arnês mantém os 2 quadros de antes.
- **`a_pincelada_nova_a_cruzar…` (180 s)**: lenta por natureza — 161,8 s sozinho (lista gerada, 29/10), 170,0 s sozinho no
  `ci-test` hoje; código idêntico ao main. Entrou no override `terminate-after = 10` das águas lentas (`.config/nextest.toml`).
- ⚠️ **`o_pen_down_manda_a_agua_herdada_pela_divida` NÃO curado** — reproduzido em 04/11 junto dos vizinhos
  (`package(ph2d-tool-painter) & test(wetpaint::)`, load 70–100: 17 vs 16 passos); 0/10 sozinho. Mecanismo: o `Up` devolve
  o motor ao worker com `sim_should_run` verdadeiro e a dívida `acc` (teto 50 ms) já cheia; o teste exige que o
  `bring_home` do `congela` ganhe a corrida contra UM passo (~25–35 ms). Cura certa: porta de teste que segure o
  `hand_off_sim` até o `congela` (campo `cfg(test)` na `WetSession`).

## 5. Vermelhos fora dos diffs, NÃO curados

clippy em `ph2d-panel-wet-tuning/tests/it/seam.rs`, `ph2d-paint-tinta/tests/it/meio.rs`,
`ph2d-app-motion/src/motion_shape_placa_quadro.rs`; `cargo machete` sai 1. Se o ship passa por eles:
**ver o resultado do ship**.

## 6. Prova de fecho

`<preencher: suíte final / ship / CI_RUN pela janela principal>`. Antes das curas: 21 vermelhos (§2).

## 7. O que um merge destas linhas pode partir

Qualquer linha futura que toque `cursor_preso.rs`, `cards_linhas.rs`, o fio do projeto (`project_schema.rs`,
`cad.rs`) ou as portas de tema (as 10+1+1 declarações acima) reconta o schema por
`python3 scripts/schema-recount.py` e confere a tripla.

## 8. ABERTO

- **Alfinete** nos cartões Object/Appearance/Fill/Stroke/Ordering/Material: só DEPOIS do CI
  ([plano 09](../09_alfinete_nos_cartoes_PLANO.md)).
- Pergunta do dono (do handoff do Inspector): o clique fora de uma lista também age no que está por
  baixo (desseleciona); ele pode escolher outra coisa.
- Itens abertos de cada linha: nos três handoffs da §1.
- Curas **depois** do último smoke do dono, ainda não vistas por ele: duplicar/ressurreição da nota e o
  grampo da janela (ver «Smoke do dono»).

## Smoke do dono

Este é o arquivo que você lê quando voltar. Só o que você ainda não viu. O texto do app é em inglês.

Antes de tudo: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_VEC_CARTOES_SMOKE=3 ./target/smoke/ph2d-host-desktop`
(abre o Inspector com um retângulo que tem Fill e Stroke).

**A. A nota vai junto com o cartão solto**
1. No cartão **Fill**, clique com o botão direito e escolha **Create note**.
2. Arraste o cartão pelo título até o meio da tela.
3. Deve acontecer: a nota está dentro do cartão solto e NÃO sobra nenhuma nota no fim do painel.
4. Apague a nota dentro do cartão. Depois devolva o cartão pelo ícone de voltar.
5. Deve acontecer: a nota apagada NÃO volta. Deu errado se ela reaparecer.

**B. A nota que você não apagou volta uma vez**
1. Repita: **Create note** no Fill, arraste o cartão solto, mas desta vez NÃO apague a nota.
2. Devolva o cartão pelo ícone de voltar.
3. Deve acontecer: a nota volta uma única vez. Deu errado se aparecer duas, ou nenhuma.

**C. Perto do fundo da janela**
1. Solte o cartão perto da borda de baixo da janela.
2. Abra a lista **⠿** do cartão e depois, com o botão direito num estilo, **Edit style**.
3. Deve acontecer: a lista e o editor aparecem inteiros, dentro da tela. Deu errado se algum ficar cortado
   pela borda.
