# N.º 9 — notas de medição (13/11)

## Item 2 — o custo da cura do vidro do escorrido (`69158c3890`)

Sonda `watercolor_vidro_custo_sonda` (ignorada, `--profile smoke`): 4096², papel de cor, 4 aguadas, Wet por cima com
30 ticks de água entre traços; o pen-down seguinte e o Ctrl+Z. Três telas no MESMO processo, rondas rodadas, o
MÍNIMO de 6. ⚠️ load 10–22 em todas as corridas (outro agente a compilar): leia DIFERENÇAS dentro da ronda.

| rota (ms, mínimo) | pen-down | Ctrl+Z |
|---|---|---|
| `antes` — o vidro elidido não testemunha (a resposta ERRADA de antes de `69158c3890`) | 7,5 | 35,7 |
| `segurada` — a cura de `69158c3890`: o cursor segura o vidro do reset | 45,9 · 53,0 · 59,2 · 64,2 | 71,3 · 80,4 · 89,9 · 93,1 |
| **`composta`** — o delta do topo composto com o journal, nas caixas dos dois | **14,2** (1.ª versão 25,2) | **35,6** (1.ª versão 55,0) |

Partes da `segurada` (uma ronda): `vidro_no_reset` (clone 58 MB + rewind) ~20 ms · detector ~28 · materializar
~10 · re-split ~28–31. A `composta` tira as três varreduras do vidro e, com elas, duas da TELA:

1. **O vidro** compõe-se em `StoredPlane::composto_com_o_journal` (`undo_delta_composto.rs`): as caixas da grade de
   128 que o topo ou o journal tocam; `before` = topo onde ele fala, senão o journal, senão o vivo. Ali mesmo sai se
   o escorrido mudou algum byte (o `from_journal` separado custava 13 ms a mais).
2. **A tela**: o `absorbed` barato recusava o topo `Tiles` (B17.5) — a tela inteira era copiada (3,5 ms) e re-partida
   (7,6 ms). Agora aceita `Tiles` (o mesmo `Patch` exacto; o gate campo-a-campo `absorb` verde). Com a tela que o
   escorrido não tocou, o delta dela é o do topo tal qual.
3. **A montagem** dos dois lados é paralela por caixa.

O que fica (~7 ms no pen-down acima da rota errada) é o tamanho do dado: nesta cena o escorrido escreve o vidro em
352–418 das 1024 caixas e 210–254 mudam — o passo guarda ~57 MB, como a segurada já guardava.

Gate: `o_vidro_composto_da_o_documento_do_vidro_segurado` (composta × segurada ao bit em cada traço, desfazer e
refazer; controlos: as duas dispararam). Mutação: o `before` do journal trocado pelo vivo ⇒ vermelho.

Re-medida 13/11 22:15 (bateria 1, `target/n9/b1/cura_do_vidro.txt`) — ⚠️ load 19–35 (outra linha a compilar e a correr o app
a meio): pen-down composta **13,4** · segurada 48,1 · antes 7,3; Ctrl+Z composta **40,5** · segurada 76,1 · antes 34,0 ms. As
DIFERENÇAS repetem as da 1.ª tabela. Calma de verdade não houve nesta janela (a carga é de outras linhas: não se espera por ela).

### Vermelhos do filtro `wet` (13/11, depois de `8a1714c8d0`) — nenhum é desta mudança

Vermelhos SOZINHOS e também na base `88ac5f8653` (restaurada por `git restore --source`): `a_primeira_marca_do_wet_refaz_a_imagem_inteira`,
`com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`, `a_secao_desligada_e_o_traco_de_hoje_ao_byte`,
`cada_preset_de_fabrica_mapeia_por_funcao`, `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao`,
`o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` (+ `receita_e4`, já no prompt n.º 9). Verdes sozinhos, vermelhos
só em lote (o flake de isolamento da contagem de leituras): `a_silhueta_alem_da_pegada_nao_e_avaliada…`,
`o_motor_nao_le_a_ponta_alem_da_pegada…`. Os filtros `undo` (187), `vidro` (34) e `absorb` (17): verdes.

## Item 1 — a matriz NOVA=1 (bateria 1, `target/n9/b1/`) — ⚠️ load 0,8 no início, 19–35 a meio (outra linha)

Começou calma e a linha TimelinePrincipal abriu o app e compilou a meio: os ms abaixo NÃO são régua de calma; ficam como
registo. Impasto (`omissão:impasto`): duplicar **27,4** · apagar 18,8 · grupo 19,9 · trocar 13,0 ms. WB: duplicar 23,0 ·
apagar 22,1 · grupo 19,9. Os gestos do Wet seguem altos (trocar 58–66, voltar 38–101) — o item 3 abaixo é a 1.ª cura deles.
⏳ A régua de calma do item 1 continua por fazer: corra-a quando nenhuma outra linha estiver a compilar (`uptime` < 5 ANTES
e DEPOIS de cada pincel — o script grava `load_*.txt`).

## Item 3 — a troca com a tinta viva do Wet: a dobra do relevo (`c166588404`)

**Porquê a dobra de 12–16 ms (N8 bateria 2):** não era a TELA (o `[paint-perf]` diz `janela=rect`), era a janela da tinta
inteira da camada que adormece e da que acorda (`a_tinta_que_adormeceu_muda`/`_acordou_muda`), que na cena do app cobre
quase a tela. Essa marca existe porque a ÓTICA da tinta viva muda (assentada ↔ viva) — mas nenhum píxel é escrito por ela
(os do último quadro da sessão chegam pelo `mark_dirty` de quem os escreve, `composite.rs`), e o relevo só lê o alfa dos
píxeis, os planos por camada e o traço aberto. A sonda (64², papel, 3 trocas): **0 texels** do relevo, cobertura, material e
papel mudam com a troca; o sujo era 64×43.

**A cura:** `mark_dirty_so_a_figura` (stamp_preview.rs) — a mesma marca, mas guarda `(dirty_rect, a parte do relevo)` em
`sujo_da_figura`; toda marca normal entra na parte do relevo; vale só enquanto o `dirty_rect` for o mesmo (senão é todo do
relevo). `take_preview_dirty` publica-a (`preview_gpu_relevo`: `Some(None)` = nenhum texel, `Some(Some(r))`, `None` = a região
toda) e a dobra da placa usa-a (sem parte: 1 texel redobrado, os mesmos bytes, e a luz da região acende-se com os planos em
dia). Com o traço aberto (`stroke_height`/`stroke_film`) é a marca de sempre. Chave A×B `PH2D_DOBRA_A_REGIAO=1`.

**Gates:** `a_troca_com_a_tinta_viva_dobra_na_placa_so_o_relevo_ao_bit` (app, placa × oráculo `SEMPRE_INTEIRA`, 9 passos com
trocas, undo, redo, pinta-e-troca e troca-e-pinta sem drenar no meio; controlo `DOBRAS_PELA_FIGURA`) e
`a_troca_com_a_tinta_viva_muda_o_relevo_so_onde_a_placa_o_redobra` (ferramenta, sem placa). Mutações vermelhas: a dobra só de 1
texel sempre (passo 6) · `mark_dirty_so_a_figura` esquece a parte anterior (passo 6) · a marca normal fora da parte (passo 8, e
o gate da ferramenta). ⚠️ **O oráculo `SEMPRE_INTEIRA` partilhava a DOBRA com a pista nova** (só a composição era inteira): as
duas primeiras mutações saíam verdes. Agora ele dobra também o relevo inteiro.

**No app (bateria 2, `target/n9/b2/`, `Filbert`/`Wet Liner`/`omissão` wet, 2 rondas intercaladas depois × antes com
`PH2D_DOBRA_A_REGIAO=1`) — ⚠️ load 7–20 (outra linha): INCONCLUSIVO.** Trocar Filbert 46,9 · 46,5 (depois) × 49,4 · 50,9 (antes) ms;
Wet Liner e omissão dispersam 30–117 nos dois. O `[paint-perf] WORST` é o pior quadro da janela inteira, não o da troca: a dobra
dele vai de 0,4 a 176 ms nas DUAS rotas. ⏳ A régua do item 3 no app: com calma, e com as partes do `[quadro-lento]` do quadro da
troca (a dobra desse quadro, não o pior da corrida). A prova de que a dobra sai é o gate (`DOBRAS_PELA_FIGURA`) e a sonda (0 texels).
