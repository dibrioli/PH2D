# Handoff — linha `TrelloUITeste`, FECHO: bloco 11 (T63 + T29 parte 1) (02/11/2026)

Linha de TESTE (não integrada); o que sobrevive é `docs/CartoesUI`. Worktree `Worktrees/line-TrelloUITeste`, branch `line/TrelloUITeste`. **Nunca houve integração nem push.** Anterior: [bloco 10](HANDOFF_line_TrelloUITeste_2026-11-02_bloco10.md).

**Commits do bloco 11:** `9f647fd135` (T63 tema/contorno/notas por cópia) · `31c228f299` (T29 parte 1 + T64: caixas de marcar à Figma e cara das listas suspensas) · este handoff (docs 04/05/README/02) — base `8265e0a3cc`.

**Smoke do dono 02/11:** sobre o cartão **Camera** em modo cartões: «cartão câmera todo bugado, sem título, mal organizado, com cores diferentes do padrão, Ponto de animação mal posicionado». E: «Mas já chega para essa linha. Já conseguimos testar a UI e ela funciona muito bem e é linda.» ⇒ **linha FECHADA pelo dono.** T63: sem comentário específico (implementado, testado, smoke não confirmado explicitamente).

## O que a linha deixa

- `docs/CartoesUI/` inteiro; **[05_ESPECIFICACAO_UI_COMPLETA.md](../05_ESPECIFICACAO_UI_COMPLETA.md) é a especificação para a linha real** (regra de referência: nós → Figma → Rive → Inkscape/Godot/Unity; tabela «smoke OK»; vocabulário; soltos; ABERTO).
- **[06_COMO_PORTAR_PARA_O_MAIN.md](../06_COMO_PORTAR_PARA_O_MAIN.md)** — como a linha real usa o código desta: portar o estado final por família (diff a partir de `4e648ec2b4`), com as medições de conflito, esquema e shell.
- Muitos painéis NÃO foram trabalhados (só Inspector em cartões, soltos, Bones, estilos de cor do Vector): lista no topo do 05.
- Próximo: **linha de implementação real a partir do `main`**, seguindo o 05 (o código desta branch serve de leitura; mapa em [03](../03_mapa_do_codigo.md)).

## Feito

| item | mecanismo | teste |
|---|---|---|
| T63 face por cópia solta | `card_instance_ops.rs`: `loose_face_of(k)`, `passa_face`, `loose_at`; `detach_card` copia, `return_loose_card` move; menu de tema e «Create Note» pela face | `each_loose_copy_keeps_its_own_theme_and_notes` (7/7 contraprovas); foto `t63_tema_por_copia_antes_depois.png` |
| T64 + T29 p.1 caixa de marcar | `property_row/marca.rs`, `panel::cards::corpo::check`, `metrics::CHECK_BOX/CHECK_GAP` (22 + 8 chamadas) | `a_check_in_a_card_is_a_figma_checkbox_row` (contraprova: ramo desligado ⇒ vermelho) |
| T29 p.1 chip da lista suspensa | `widget::dropdown::paint_dropdown_chip` → `corpo::chip` (18 chamadas) | nenhum (só pinta) |

Escolhas do dono no T63: a cópia nasce com cor e notas da lista; ao voltar, cor e notas passam para a lista. T64: caixa à Figma.

## ABERTO (detalhe e consolidado em [05 §8–§9](../05_ESPECIFICACAO_UI_COMPLETA.md))

1. **Cartão Camera** com os 4 defeitos acima (sem título, mal organizado, cores fora de `CardInk`, ponto de animação mal posicionado).
2. T29 2.ª metade: 6 `Checkbox::new` directos, 11 `property_label_row`, popover das listas no tom do Inspector, `paint_text` directos fora de `CardInk`, fold partilhado entre cópia 1 e lista.
3. Solto colado ao topo do canvas sob a barra de modos; corda só por cima dos soltos do próprio painel; iPad (T49).
4. Vermelhos antigos que não são desta linha (`seam_*`, `every_non_literal_hash_is_named`, 16 do editor-core, 16 do skeleton, `toast_reading`).

## Regras aprendidas

- **Instância == secção na cópia 1** faz toda preferência por id ficar partilhada com a lista: use uma chave derivada (face).
- **Foto do corpo de um cartão exige-o aberto e acima da dobra**; o script de foto não clica ⇒ semeie um solto ou escolha a cena (o Camera escapou assim).
- **Contraprova por patch reverso** tira o «antes» limpo: `git diff > p; git apply -R p; foto; git apply p`.
