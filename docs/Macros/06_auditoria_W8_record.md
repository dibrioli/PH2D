# Macros — auditoria do Record depois da W8 (2026-10-17)

> Smoke do dono (17/10), cena `PH2D_MACRO_SMOKE=5`: «Mudar as propriedades no painel macro parece funcionar. Mas o
> Record macro não funciona corretamente. A simulação não toca normalmente, o Rewind deixa resquícios de pintura
> na tela e outros problemas.» Duas lentes (correção com sonda temporária, já apagada; costura de UI por leitura).
> NADA consertado. Plano W8: [05_plano_W8.md](05_plano_W8.md).

## A1 — a régua «fim assentado» aplicada a uma imagem do editor que ainda não assentou (a causa)

- **Mecanismo:** `ph2d-app-painter/src/macro_track/mod.rs` `mistura_na_caixa` — «atual ≠ fim ⇒ atual» (imagem que
  recebeu a gravação). O fim é a água ASSENTADA (`etapas.rs` `imagens_da_gravacao`, W8 2c), mas no editor a água
  corre ~60 s depois do Parar: nesse minuto quase todo texel molhado difere do fim e a mistura mostra a imagem FINAL
  do editor por cima do palco. A Timeline escreve `MacroTempo` mesmo em pausa (`timeline_bridge.rs:223-231`) e o
  palco vai à frente da pré-visualização do editor (`fase_extract_inputs.rs`, `.chain(do_editor)`).
- **Medido** (sonda, 1024², 3 pinceladas Wet Paint, worker REAL, `smoke`, load 3–5): no instante 0 (Rewind),
  texels não brancos = **217 589** de 274 625 pintados quando o fim chega (10,7 s) → 123 695 (23,9 s) → 13 471 (35,2 s)
  → 1 903 (46,1 s) → 252 (56,6 s) → **0** com a água parada (67,7 s, 3 697 quadros). Durante a animação, 79,7–84,2 %
  dos texels que o palco anima mostram a imagem final. Antes de o fim chegar (5–10 s) a pintura some (palco só).
- **Refutado:** «o worker real assenta noutro estado» — assentado, editor = fim reproduzido AO BYTE (0 texels).
- **Gates verdes por cima:** `a_agua_que_corre_depois_do_parar_assenta_na_imagem_do_fim` só olha o estado FINAL;
  os de `palcos_fim_tests.rs` usam como atual `fim_de(doc)` — o ORÁCULO é a própria `imagens_da_gravacao` sob teste —
  e a fixture nunca tem a água a correr; o (b) chega a ISENTAR o fenómeno («salvo a água que corria no Parar»).
- **Gate que faltava:** gravar com o Painter ligado e o worker real, Parar, e a cada quadro até a água parar contar
  os texels não brancos no instante 0 e a fracção da animação escondida.

## A2 — dois lugares que deviam concordar: o último quadro do palco (o Parar) × a régua «fim» (assentada)

- O palco acaba na tela do Parar (água a meio); com o editor assentado (atual = fim) a mistura mostra o palco ⇒ o
  último quadro difere do editor em **16 320 de 17 027** texels pintados (256², relógio fixo; 248 393 a 1024²) e
  SALTA para a imagem real quando o clipe deixa de conduzir. O gate `palcos_fim_tests.rs` exige como CONTROLO
  `c.palco != premul(&fim)` — fixa a discordância em vez de a perguntar.

## A3 — custo por quadro e espera

- O fio do fim na `=5`: **5,6 s** (dev) / **9,4 s** (`smoke`, com carga); a água das gravações medidas pára em
  2 067–3 697 quadros (não chega ao tecto de 36 000).
- Com a água do editor a correr, `content_revision` sobe a cada quadro ⇒ `sobe` relê a imagem, recalcula `recebeu` e
  sobe a ranhura INTEIRA: **6,6 ms média / 12,5 ms máx** (`smoke`, só CPU, sem o upload de 4 MB) durante ~60 s —
  causa provável de engasgo. Gate que faltava: orçamento de `sobe` por quadro com o Painter ligado e a água viva.

## A4 — costura (lente 2, leitura)

- **▷ na vista Arrange não toca o palco** na `=5`: `MacroTempo` só se escreve no ramo solo (Keys) ou com a cena a
  correr por Autoplay (`apply_prop.rs:108-115`, `fase_sequences.rs`); a `=5` nasce com Autoplay desligado. Parado em
  Arrange, o que se vê é a sprite sem palco.
- **Um 2.º Stop não re-escreve o Loop do clipe**: `instala.rs` `clipe()` põe `keys_loop_range (0, d)` só na criação;
  `retime_macro_clip` (`ph2d-timeline/src/doc_owner.rs:178-200`) re-cronometra as chaves e não o loop.
- Gates verdes: `palcos_tests.rs` chamam `segue`/`sobe` sem `clear_macro_clocks` nem a ordem da shell; nenhum cobre
  Arrange com o Painter ligado, o Rewind em Arrange, nem a água depois do Stop.

## Não regressões verificadas

`segue` a resolver a sprite parado (sem defeito); `por_reescrever` (gravação nova = outra crua ⇒ não se aplica);
`reedita` com outro estado inicial (recomeça do zero, palco = B do zero, 0 texels).

## Decisão do dono (17/10): opção 1 — MARCAR onde o artista mexe depois de gravar

Recusadas: (2) durante a água o Play mostra só o clipe (o que se pinta nesse minuto não aparece); (3) voltar ao fim
no Parar (a água que corre depois cobre o clipe inteiro). A W9 substitui a régua por IGUALDADE de píxeis
(«≠ fim / ≠ início», `recebeu_a_gravacao`) por uma régua de AUTORIA:

- Cada imagem com uma `Macro` leva uma marca por texel «o artista mexeu aqui depois da gravação» (1 byte/texel, ou
  bits; ~1 MB a 1024²). Escreve-a toda escrita do ARTISTA na imagem depois do Parar — traço, borracha, preenchimento,
  filtro, Apply de outra ferramenta, a escrita de píxeis da sprite —, NUNCA a água que corre sozinha nem a reescrita
  de uma etapa editada (essa é da macro). Medir antes onde as escritas do Painter passam (os dabs/regiões sujas, o
  `mark_dirty`), e a das sprites sem Painter (o funil `commit_edited_texture`).
- A mistura: texel marcado ⇒ a imagem atual; senão ⇒ o palco. Some a `recebeu_a_gravacao` e a comparação com o
  início/fim; a reescrita de uma etapa editada (2b) passa a reescrever os texels NÃO marcados.
- A marca anda com o undo (desfazer um traço desmarca o que ele marcou) e com o projeto (gravada? — decidir e medir;
  sem ela, abrir o projeto marca tudo como da macro).
- Uma gravação nova zera a marca. A imagem que nunca recebeu a gravação (a `=5`): o clipe anima por baixo, o que o
  artista pintou fica por cima — a mesma lei, sem caso especial.
- O salto no fim do clipe (A2) continua (o clipe acaba no Parar; o dono recusou o Parar que espera a água) — dizer
  no smoke.

E no mesmo bloco, sem decisão: A3 (a ranhura não se re-sobe inteira a cada quadro da água: com a marca, a chave é a
versão da MARCA, não a `content_revision`); A4 (o Loop re-escrito a cada Stop; o ▷ em Arrange — medir se é desenho
da Timeline ou defeito e dizer ao dono). Gates: os «que faltavam» de A1–A4, com o worker REAL e o Painter ligado, e o
oráculo NUNCA a função sob teste.
