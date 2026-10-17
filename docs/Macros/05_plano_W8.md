# Macros — plano da W8: fechar o ABERTO da W7 num bloco (2026-10-17)

> Ordem do dono (17/10): fechar o §8 do [handoff W7](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W7.md)
> num bloco só (CLAUDE.md §0.10); depois a prova de mutação e o fecho. Antes: [plano W7](04_plano_W7.md).
> Smoke do dono de `=5` e `=4` (W7 §7): **OK** (17/10).

## Decisões do dono (17/10)

| § W7 | Pergunta | Decisão | Recusadas |
|---|---|---|---|
| 8.11 (item 2a) | apagar até ao papel faz a macro reaparecer no Play | **regra dupla**: numa imagem que recebeu a gravação vale «≠ fim ⇒ atual»; nas outras, «≠ fim E ≠ início» (a W7) | marcar cada píxel tocado (uma máscara por imagem); deixar |
| 8.8 (2b) | com etapas editadas a imagem salta quando o clipe pára | **editar uma etapa já muda a imagem**, com desfazer | botão «Aplicar na imagem»; deixar |
| 8.7 (2c) | a água que corre depois do Parar cobre o clipe inteiro | **fim = a água assentada** (a conta da régua do tutorial) | o Parar espera a água (clipe mais longo); deixar |
| 8.2 (2d) | o Motion na reprodução coze na placa; a CPU custa 10–15× menos nesta cena | **escolher pelo tamanho** (CPU nas cenas pequenas, placa nas grandes, o ponto de troca MEDIDO) — trabalho da linha do Motion: [prompt](handoffs/PROMPT_para_line_motion_cozimento_na_reproducao_2026-10-17.md) | sempre a CPU; deixar |

## Item 2 — a imagem do artista (feito: `03087e26d1`)

- **2a** — «recebeu a gravação» = onde o fim difere do início há mais píxeis da atual iguais ao fim do que ao
  início (`recebeu_a_gravacao`, por chave da mistura). A desfeita, a de outra imagem e a que nunca a teve
  ficam do lado do início. Gate `apagar_ate_ao_papel_numa_imagem_que_recebeu_a_gravacao_fica_apagado`.
- **2b** — o fim é o da trilha QUE TOCA; a versão é o hash dela (`versao_da_trilha`: nome e dica não a
  mudam). Gate `o_fim_e_a_trilha_que_toca_e_editar_uma_etapa_o_recalcula`. A reescrita da imagem: §2b abaixo.
- **2c** — `imagens_da_gravacao` corre a água no relógio fixo até parar (`QUADROS_ATE_A_AGUA_DO_FIM_PARAR`,
  tecto de CPU do fio). Medido: depois do Parar o editor NÃO passa ao relógio fixo (o worker continua ao
  ritmo da parede); mesmo assim, com o worker ritmado e quadros irregulares, a água do editor assenta na
  imagem do fim **ao byte** (`a_agua_que_corre_depois_do_parar_assenta_na_imagem_do_fim`). Consequência: no
  último quadro o palco é a tela do Parar (a água ainda a correr) e a imagem é a assentada — a água que
  correu depois do Parar aparece de uma vez quando o clipe acaba (antes estava pronta desde o 1.º quadro).
- ⛔ **Defeito da W7**: `imagens_da_gravacao` chamava o `run_full` duas vezes; numa pilha TRIVIAL o 1.º leva a
  tela e o fim saía vazio ⇒ a mistura nunca acontecia (a `=5` tem papel, pilha não trivial: o smoke passou).
  Agora `imagem_aplicada` (não consome). O gate do 2c apanhou-o.

### §2b — a reescrita da imagem com UM desfazer

1. Um `Macro.doc` novo com a MESMA gravação crua (estado inicial + eventos) e outra trilha que toca: o palco
   guarda a gravação antiga (modelo do fim por camada) e espera a nova; enquanto espera, o desfazer global
   não regista (motivo «a macro a reescrever a imagem», como «a conferir»).
2. Com a nova pronta e a água do editor parada: cada texel de cada camada que ainda é igual ao do fim antigo
   passa ao do fim novo (píxeis, relevo, cobertura, material). Painter ligado ⇒ um passo do Painter; sprite
   sem Painter ⇒ a imagem pelo funil `commit_edited_texture`, no passo global.
3. O passo do Painter e o passo global levam o MESMO número de transação: o Ctrl+Z que desfaz um desfaz o
   outro, seja qual for a fila que o recebe (`undo_route.rs`).
4. Desfazer a edição muda o `Macro.doc` de volta e a conta corre ao contrário — já não acha texels iguais ao
   fim novo (o desfazer devolveu-os): não regista nada.

## Item 3 — partilhar a corrida da gravada da k−1 com a base da k

Sonda `sonda_a_corrida_partilhada` (smoke): a base que o produto mede contra a gravada da anterior
continuada no relógio fixo até à idade em que a base acaba, no critério da k; 30 pinceladas, 512² e 1024²
no relógio fixo e 512² com o worker ritmado e quadros irregulares. **Resultado: por medir.**

## Item 4 — o fio do fim que cai (feito: `03087e26d1`)

`ErroDaMacro::FioCaiu` («macro estragada»); o `erro_de` do palco diz o erro da reprodução ou o pânico do fio
do fim. Gate `o_fio_do_fim_que_cai_ou_falha_diz_se_ao_artista`.

## Item 5 — a idade do aluno (feito: `613462877c`)

A idade é o relógio da ÁGUA: com outra ferramenta ativa nenhuma das duas anda (o tique só chega à ativa) —
fica. «Recomeçar» devolve o desenho ao início ⇒ a idade volta à do início; o desfazer devolve a tinta já
assentada (avaliada no fim das curvas). Gate `recomecar_volta_a_idade_ao_inicio`.

## Item 6 — a sprite e a Máscara (feito: `03087e26d1`)

- 6a: a sprite sem Painter relê-se quando a identidade dos píxeis muda (`Sprite.source` + `SpritePixels`).
  Gate `a_sprite_sem_painter_rele_se_quando_os_pixeis_dela_mudam`.
- 6b: o filme da Máscara do Painter ligado vai por cima da mistura (`tinge_com_a_mascara`); a régua da atual
  fica sem ele; a versão do filme entra na chave da subida. Gate
  `o_filme_da_mascara_do_painter_ligado_fica_por_cima_do_clipe`.

## Item 7 — o censo dos controlos (feito: `812d110304`)

1 478 linhas → 616 + 684 + 187, incluídos no mesmo módulo (os nomes dos testes que os docs citam ficam).
Prova: sha256 da concatenação = o do original; `--list` dá os mesmos 5 testes.
