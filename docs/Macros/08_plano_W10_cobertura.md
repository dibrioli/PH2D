# W10 — a marca do artista passa de BIT a COBERTURA (sem resquícios nos cruzamentos)

> Origem: smoke do dono, 19/10 (depois do 3.º fecho da W9): *«traços feitos posterior à gravação da Macro, quando
> volta a timeline, resquícios aparecem onde os traços cruzaram»*. Três fotos: pincel macio com grão (manchas
> escuras onde a pincelada nova cruza uma gravada ainda por nascer) e Digital duro («x» pretos nos cruzamentos).
> Estado da linha e o que já está feito: [handoff W9](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md)
> §2 itens 1 e 11, §8 item 9.

## 1 — Mecanismo (não é bug de conta: é o tecto da marca binária)

Hoje cada texel é «do artista» (mostra a imagem ATUAL) ou não (mostra a REPRODUÇÃO no instante `t`) —
`crates/ph2d-app-painter/src/macro_track/mod.rs` `mistura_na_caixa` (CPU, por caixa, dentro de `sobe`). Onde a
tinta nova cobre PARTE do texel (borda macia, grão, pincel transparente) o certo é «a reprodução em `t` com a tinta
nova por cima», e a marca não o sabe exprimir: marcado mostra a imagem atual, que JÁ contém o traço gravado (o
resquício); não marcado perde a tinta nova (as faixas brancas do 3.º fecho). O miolo `≥ ½` (W9 item 11) só
escolheu o menor dos dois erros.

## 2 — A lei exata (composição «por cima»)

Para cada texel tocado pelo artista depois de gravar, guardar:

- **c** — a cobertura acumulada da tinta do artista, `0..=255` (`1 − Π(1 − aᵢ)` sobre os passos; a força por
  dab já existe: `crates/ph2d-tool-painter/src/macro_track/forca.rs`).
- **B** — o píxel que estava POR BAIXO no 1.º toque (o `before` do passo em que `c` sai de 0 —
  `undo_marca.rs::fecha_o_passo` já o tem). ⚠️ Não serve o fim da gravação: a água que corre depois do Parar
  muda a imagem por baixo e a fórmula derivaria em todo o texel.

Se a tinta é composta «por cima» (`atual = A + (1 − c)·B`), então o que se quer ver é `A + (1 − c)·palco(t)`:

```
mostrado = atual + (1 − c) · (palco(t) − B)        c = 0  ⇒  palco(t)  (como hoje)
                                                    c = 1  ⇒  atual     (como hoje)
```

Exata para qualquer cor e qualquer número de passos (a composição «por cima» é associativa). Calcular no MESMO
espaço em que o carimbo compõe (o 3.º fecho mediu: Digital compõe no byte — força ½ ⇔ byte 127); premultiplicado
dos dois lados. Medir antes de escrever: um gate por meio com a régua da pincelada no branco.

**Sem força conhecida ⇒ `c = 255`** (a lei de hoje, mostrar a atual): a água / `disco_na_água` / a cobertura
do Solid na água, Blur/Smear/Clone/Inpaint (dão o que estava por baixo — não são «por cima»), a borracha (se não
for «por cima» com cor transparente; medir), a ponta com forma/grão/achatada **até** a força delas ser lida do
próprio carimbo (ver §5), o relevo do Impasto.

## 3 — O que muda (mapa do explorador, 19/10)

| Peça | Hoje | W10 |
|---|---|---|
| `MarcaDoArtista` (`tool/macro_track/marca.rs`) | 1 bit/texel, `Arc<Vec<u64>>`, `caixa`, `versao` | `c` u8 + `B` RGBA8 em ladrilhos esparsos (só tocados); `marcado(i)` vira `c(i) > 0` para quem só pergunta isso (a reescrita) |
| Fecho do passo (`undo_marca.rs`) | `(mudou ∨ miolo) ∧ (¬água ∨ disco) ∨ disco_na_água` → bits | `c ← 1 − (1−c)(1−a_passo)`; `a_passo` = força onde há, 1 onde só há «mudou»/água; `B` no 1.º toque |
| `DeltaDaMarca` (desfazer) | bits ligados; `aplica` OR/AND-NOT; `junta` OR | antes/depois de `c` (e `B` no 1.º toque) por texel tocado; `junta` guarda o 1.º antes e o último depois; o provisório (`antes_do_passo`) vira cópia dos ladrilhos tocados |
| Mistura (`app-painter macro_track/mod.rs:318-337`) | escolhe atual/palco | a fórmula do §2; a chave `mistura_subida` continua pela `versao` |
| Reescrita (`reescrita.rs:58`, `tool/reescrita_da_gravacao.rs:90`) | salta o marcado | salta `c = 255`; onde `0 < c < 255` reescreve `B` (o que está por baixo é da gravação) e recompõe `atual` — decidir e medir |
| Efetiva sem o Painter (`marcas.rs:108`, `une_bits` com a BASE) | bits | `c = 255` onde a sprite mudou fora do Painter (não há força) |
| Persistência (`marcas.rs:187-256`, `marca.rs:253-311`) | `FORMATO = 1` (duas constantes) | `FORMATO = 2` com leitura do 1 (bit ⇒ `c = 255`, sem `B`). **Sem degrau de `PROJECT_SCHEMA`**: o campo é opaco e o 192 ainda não está no `main` |
| Custo (A3, `palcos_custo_tests.rs:279`) | 0,704 / 4,873 ms por quadro (W9) | medir de novo; o `B` dobra a memória da marca nos texels tocados |

## 4 — Gates (vermelhos antes)

1. As fotos do dono como fixture: tela com duas pinceladas gravadas; depois do Parar, pinceladas novas a cruzá-las
   (Digital duro, pincel macio, pincel com grão); em `t` antes de cada gravada nascer, a zona do cruzamento =
   a pincelada nova sobre o BRANCO (± 1 nível), sem resquício. Régua: a mesma pincelada pintada sozinha.
2. `c` e `B` vão e voltam com o desfazer/refazer, com a absorção da água e com o run coalescido.
3. O projeto grava e lê `FORMATO = 2`; um projeto com `FORMATO = 1` abre (bits ⇒ `c = 255`).
4. Os gates binários do §3 do mapa (`marca_tests`, `tests::cada_passo_vai_e_volta_ao_bit`,
   `composicao_tests`, `marca_cobertura_tests`, `marca_painter_tests`, `palcos_marca_tests`, `palcos_fim_tests`,
   `project_marcas_tests`) reescritos pela lei nova — cada um provado vermelho sob a mutação que o justifica.
5. A3 medido de novo, com a tabela ao lado.

## 5 — Por decidir na janela da W10 (técnico — padrão-ouro)

- A força das pontas com forma/grão/achatada: ler a cobertura do PRÓPRIO carimbo (o núcleo já a calcula por
  píxel) em vez do perfil redondo — fecha o «até» do §2 e tira o recorte `forma/grão/achatada` do 3.º fecho.
- O Watercolor (Beer–Lambert, não é «por cima»): medir se `atual + (1−c)(palco − B)` com `c` = força do pincel
  fica abaixo de 1 nível na régua; se não, `c = 255` para ele.
- O Wet Paint (a água): fica `c = 255` (a tinta aparece pela água, fora do passo).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current` —
a W9 está fechada (bateria verde em `9464cea7cb`); a W10 abre a partir daqui, na mesma linha, por integrar junto.

## W10 — resultado (19/10)

Feita. Mecanismo, tabelas medidas, mutação 14/14 e o que fica aberto: [handoff W10](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md).

Decisões do §5:

- **Força das pontas com forma/grão/achatada**: lida do PRÓPRIO carimbo (um plano `u16` nas 3 rotas do Digital, cada dab
  compõe a sua alfa em `stroke_cover::compoe_forca`) — o recorte `forma/grão/achatada` do 3.º fecho caiu. O carimbo da
  placa não o devolve ⇒ com o plano armado o lote vai pela rota CPU em bandas.
- **Watercolor** (não é «por cima»): fica com `c` cheia, como Impasto e os outros modos de mistura; só o Digital em
  Mix/borracha, camada única, sem máscara nem Solid, arma a medição (`a_forca_mede_se`).
- **Wet Paint**: fica `c` cheia (a tinta aparece pela água, fora do passo) — os cruzamentos nele ainda mostram o nó.

Desvios do §2/§3: `c` é `u16` (CHEIA = 65 535), não `u8` (em `u8` somava até ½ nível); sem degrau de `PROJECT_SCHEMA`
(FORMATO 2 no blob opaco, o 1 abre com `c` cheia). A `=6 cruza` pintava em Wet Paint — passou a Digital.
