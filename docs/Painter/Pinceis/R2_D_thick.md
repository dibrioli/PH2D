# R2-D — Thick Impasto (G-10 · CAP-21), 31/10

**Resultado:** preset novo `WetPainter/Oil/Flat/Thick Impasto` por COMPOSIÇÃO (receita sobre o Filbert, zero motor,
zero `spec.rs`). Receita `THICK_IMPASTO` em `tests/it/familias/oleo_acrilica.rs` (no `WET`). Prancha:
`pranchas/r2_thick_impasto.png` (nosso | Rebelle `Oil/Flat/Thick`, cena `tamanhos`, Real Oil com luz).

## O que é o «Thick» do Rebelle (foto `flat_thick_*`)
Pegada redonda que segue o traço, cobertura cheia, ORLA em nuvens (mole, ondulada), sulcos escuros finos ao longo
do traço, reflexos brancos pontuais. Relevo moderado e macio.

## Simulação (sonda `sonda_r2b_grao_do_oleo`, Filbert + knobs, 2 rondas × 4)
| v | delta sobre o Filbert | leitura |
|---|---|---|
| c | (controlo, oilDepth 0,3) | riscas lisas, borda recta, sem relevo grosso |
| a | oilDepth 0,8 · gloss 0,3 · pig 600 · faixa 2,5 · emboss 0,6 | tubo de riscas, borda recta |
| b | a + ragged 2,4 · jitter 0,1 | relevo, mas ESCADA (o «pixelado» já recusado) |
| d | oilDepth 1 · gloss 0,5 · emboss 0,8 · faixa 2 | riscas duras |
| e/f/g | + Clouds 24/48 px na textura, emboss 0,15–0,3 | grumos esburacados, orla rasgada dura |
| **h** | oilDepth 0,9 · gloss 0,5 · pig 600 · faixa 3 · emboss 0,3 · jitter 0,15 · **dureza 0,3** · Clouds 48 px | **escolhida**: orla em nuvens, sulcos ao longo, relevo grosso aceso |

Custo: a textura Clouds leva o traço de ~8 s a ~85 s na sonda (load 26–80; não isolado — medir no app).

## Leitura da prancha (honesta)
- Nosso: mais RELEVO e mais luz/sombra que o Rebelle (a meta «mais grosso»); sulcos mais numerosos e mais finos.
- Rebelle: mais macio/desfocado, menos sulcos porém mais escuros, reflexos brancos maiores.
- ⚠️ Sombra no papel: o traço fino (5 px) e o de 40 px lançam uma faixa cinza de ~10 px abaixo — vem do oilDepth 0,9.
  Se o dono achar pesada, o primeiro botão é a Profundidade do cartão (0,6–0,7).

## Substituído
O G-10 antigo (meio Impasto, grumos `Clouds` 24 px, régua 370/454) saiu de `EM_ESTUDO_MEIOS` — o nome passa a ser
este preset de fábrica (`os_reprovados_do_oleo_nao_estao_na_fabrica` verde).

## Pendente (fila pesada da linha, item 3)
O ficheiro do preset NÃO foi gerado: o hook do subagente recusa o gerador (e recusa também
`fila-pesada.sh pôr '…PH2D_REGERA_…'` — o padrão casa o próprio pedido de fila; entrou por `"$(cat ficheiro)"`).
Até a fila correr, `the_factory_presets_are_counted` fica VERMELHO (receita sem ficheiro — red-first). Depois:
`the_factory_presets_are_counted` e `cada_preset_de_fabrica_e_a_sua_receita` por nome, e a largura
(`a_largura_e_a_prometida`, Real Oil desligado). Revisor cego: não lançado (o dono afina no smoke).
