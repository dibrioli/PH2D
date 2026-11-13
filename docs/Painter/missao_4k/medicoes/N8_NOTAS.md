# N.º 8 (13/11) — notas de medição

Bateria: `target/n8/bateria1.sh` (NOVA=1, 4K, Textured Paper, `--profile smoke`, um lançamento por pincel; o «antes» é o
MESMO binário com `PH2D_TROCA_RECOMPOE=1`, que desliga TODAS as curas de troca/estrutura, não só a do item 2 — a régua do
item 2 é a matriz FIM2 do n.º 7, onde só ele faltava).

## Item 2 (B18.6) — WB duplicar/apagar/grupo na camada nova (`971585a122`)

Mecanismo: com papel de cor e aguada, `alcance_da_tinta_de` recusava (`papel_atravessa_vidro`) e o gesto recompunha a
pilha 4K inteira na CPU com vidro e relevo. O vidro tira a transparência do papel também em texels de alfa 0 (a orla),
então o alcance da camada é tinta ∪ vidro (`janela_do_vidro`, `uniao`). Tirar o último vidro dá a recomposição inteira ao
bit (o gate prova-o).

| corrida (camada nova, ms) | trocar | voltar | duplicar | apagar | grupo |
|---|---|---|---|---|---|
| n.º 7 FIM2, Watercolor Basic (base desta cura) | 14,8 | 13,1 | **89,7** | **72,4** | **78,4** |
| n.º 8 `PH2D_TROCA_RECOMPOE=1`, Watercolor Basic | 79,4 | 70,9 | 77,3 | 65,5 | 71,5 |
| n.º 8 `PH2D_TROCA_RECOMPOE=1`, omissão water | 77,6 | 119,5 | 91,9 | 69,0 | 93,0 |
| **n.º 8, Watercolor Basic** (2 corridas) | 13,3 · 13,9 | 14,3 · 17,1 | **21,8 · 22,1** | **16,1 · 16,0** | **16,7 · 18,7** |
| **n.º 8, omissão water** (2 corridas) | 14,2 · 14,2 | 12,8 · 12,7 | **20,8 · 21,7** | **17,6 · 17,6** | **16,2 · 18,9** |

Gate `apagar_duplicar_e_agrupar_com_a_aguada_no_papel_de_cor_recompoem_so_tinta_e_vidro`; mutação «sem a janela do vidro»
vermelha na janela e, com essa asserção desligada, na figura. Vizinhos por nome: 129 verdes.

## Cartão Performance (dono 13/11: «Ambas SIM, mas como opções a mais»)

- **«Faster wet paint»** = o palpite da `assenta` (`liga_o_palpite`/`palpite_ligado`; troca as voltas E esquece o memo).
  Cartão no FIM da lista do pincel, só com o Wet Paint; desligado de fábrica; gravado em `~/.ph2d/prefs.txt`
  (`wet_guess`). Fotos: [marcada com a dica](n8_cartao/marcada_com_dica.png) · [reaberto marcado](n8_cartao/reaberto_marcado.png)
  (roteiro `n8_cartao/foto_cartao.sh`, Xvfb, HOME isolado). Grava no 1.º movimento do ponteiro depois do clique (o
  detector das prefs corre no gancho do ponteiro).
- Gate `tests/palpite_da_caixa.rs` (binário À PARTE: a chave é global ao processo): de fábrica desligada; ligada a sim
  pinta outro hash; desligada volta ao bit. Mutações: sem o `memo_esquece` ⇒ «não voltou ao bit» (vermelho); sem gravar
  as voltas ⇒ vermelho. `prefs_tests::wet_guess_round_trips_and_an_old_file_is_off`.

### ⛔ Recusa MEDIDA — a caixa AVX2

| variante (6 traços 4K, 16 fios, A×B intercalado) | núcleo do advect mín | média do passo |
|---|---|---|
| lanes (produto) | 13,44 ms | 30,7 ms |
| a MESMA função sob `#[target_feature(enable = "avx2")]` | 13,36 ms | 30,3 ms (ruído) |
| tecto: lanes SEM a conta (só a recolha; resultado errado de propósito) | 11,65 ms | — |

O hash foi o mesmo (`a9bac64ac5b857c8`). Mesmo um AVX2 à mão perfeito só pode tirar a conta: ≤ ~1,5 ms de ~30 (≈ 4 %), e
não leva a sim a 1,0. O resto é a recolha dispersa (memória), que o AVX2 não acelera. Uma caixa sem efeito sentido é um
controlo que mente: **não entrou**, e a excepção de `unsafe` também não (a workspace continua `forbid`). Reabrir só com
outra medição que mostre a conta ≫ 1,5 ms.

### Vermelhos ANTERIORES vistos (não desta onda)

- `h2_os_cartoes::the_top_and_the_basics_show_the_same_controls_in_each_tool_and_medium` («brush WetPaint: sumiram» 11
  ids): igual com o cartão novo desligado (`presente` = false) — anterior.
- `the_shell_only_shrinks_and_every_app_crate_too`: `ph2d-app-painter` 24 476 × 24 429 e `ph2d-app-pixel` 11 291 ×
  11 278 — nenhuma das duas tocada nesta onda.
- `quantas_entradas_tem_cada_painel::a_carga_de_comandos_de_um_painel_so_encolhe`: `wet_tuning` 12 × 14 (a catraca
  desce; número que soma entre linhas — reconta-o o integrador).

## Bateria 2 (13/11) — ⚠️ load 15–19 (outro agente a compilar a frio): os ms NÃO valem; as PARTES sim

`target/n8/bateria2.sh`, NOVA=1, o `[quadro-lento]` agora com as partes do Painter (`3026ea2e53`). Quadros > 25 ms:

| gesto | quadro ms | Painter (dispatch · preview · dobra) | leitura |
|---|---|---|---|
| Impasto duplicar (+48 ms) | 35,4 | 7,3 · 7,3 · 0,6 | ~28 ms FORA do Painter (shell/apresentação/placa) — medir lá |
| Filbert trocar (+63 ms) | 54,0 | 30,7 · 30,5 · **16,1** | a DOBRA do relevo inteira na troca (`fold_full`?) |
| Filbert duplicar (+367 ms) | 35,3 | 0,4 · 0,2 · 0 | fora do Painter, ~370 ms depois: o fio da compressão? |
| Filbert Ctrl+Z (+51 / +188 ms) | 40,2 / 73,5 | 24,2 · 24,1 · 0,4 / 18,0 · 17,9 · 12,3 | preview 18–24 + dobra 12 |

Próximo (n.º 9): (a) a dobra inteira na troca/Ctrl+Z do Wet (porque `dobra` 12–16 ms — o `fold_full` do FrameInfo diz se foi
inteira); (b) o tempo fora do Painter nos gestos de camada (o perfil da shell no quadro: `PH2D_PAINT_PERF` não o cobre);
(c) re-medir a matriz com load < 5 (item 3 no app: Impasto dup/apagar/grupo).
