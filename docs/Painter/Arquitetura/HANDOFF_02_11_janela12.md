# HANDOFF — refatoração do Painter, janela 12 (02/11): os vermelhos da rodada de integração

Ramo `line/PainterWatercolor`, rebaseado no main `3df5894489`. Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela11.md](HANDOFF_01_11_janela11.md). Detalhe, números e SHAs:
[HANDOFF_INTEGRACAO_…_2026-11-02_VERMELHOS.md](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-02_VERMELHOS.md).

## 1. Feito

A rodada de 02/11 entrou no main SEM o Painter (~49 vermelhos latentes na linha). Trazidas as 12 curas do integrador;
rebase; os vermelhos bisseccionados e curados pela CAUSA — os de motor: o piso do espalhar sobre o acumulado (óleo sobre
óleo deixa de empilhar), o peso do sub-dab ≥ 1 célula, a lei da largura (fim do patamar; extrapola pelo último que
cresce), a receita sem ficheiro pelas migrações, a Transmission Blender (sol sob o filme, w²), o undo do motor guarda o
cursor do passo em voo; os lápis 8B/4B marcam no toque leve (dono: «Sim, marcar mais»). Réguas de paridade com o
Rebelle retiradas; as leis próprias que iam junto voltaram com tecto NOSSO (escada do relevo, Liner sem fio, mão).
Fecho: 217 testes por nome, 214 verdes; fmt/clippy limpos; gates LOC/arch verdes; auditoria de 2 lentes curada.

## 2. Smoke do dono (02/11, app `--profile smoke`, Radeon, no worktree)

Lápis 8B e 4B no toque leve · óleo sobre óleo 6–8 passadas · Gouache Flat fino × grosso · Wet Liner sem fio:
**«smoke OK»**. Ordem seguinte do dono: **voltar à refatoração e seguir o plano**.

## 3. Aberto (por ordem)

1. **FPS com o Real Oil** (J11 §4): medir no app (tela real) o traço do Dabber e o Wet Drip por cima; atacar a
   composição por passo onde a água corre (ritmos relevo/luz, J9 2b) — o único marcador de sujo é o passo.
2. O rectângulo velho do smoke J10 (o motor não o reproduz; se voltar, a subida da tela na shell).
3. Os 4 vermelhos conhecidos (§5 do handoff de integração): Transmission Blender na malha real (0,152 × Cycles 0,124 —
   próximo: malha 2D sintética no Cycles) · Relevo do papel 1 (âncora 0,67 irreprodutível) · Smear Wet não monótono
   abaixo de r≈7 sob a E10 (`sonda_monotonia_da_largura`, alinhar `raio_do_motor_da_tabela`, régua `largura_do_rasto`
   a r < 1) · Splash (fica, dono).
4. Do §4 da janela 7: E9 ⚑ furos · chip de Dilution · `set_wet_lei_da_tinta` sem uso · `assenta.rs` e
   `trail/transfer.rs` (699) em 700 linhas — partir antes de mexer · E10c · E11 GPU (a placa não lê o meio nem a pasta)
   · E12 (a Velha sai) · E13 · outros 3 modos · ajuda ilustrada · sondas para crate dev-only · crateras.
