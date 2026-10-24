# Report do dono 24/10 — «as labels foram espremidas» numa largura média do Inspector

**Medido (A/B no mesmo processo de fotos, `ab.sh <binário> <foto> <display> <x do arrasto>`):** o binário do
`main` (`5af68b7cbe`, compilado 24/10 14:36) e o da linha (`7607da9a3b`) com a cena `PH2D_GAME_CAMERA_SMOKE=1`,
o dock do Inspector arrastado para 230…280 px. [`main_x_linha_por_largura.png`](main_x_linha_por_largura.png)
(m = main, l = linha; o número é o `x` do arrasto, dock ≈ `1600 − x`).

| dock | main | linha |
|---|---|---|
| ≈ 230 | campos empilham, nomes inteiros | igual |
| ≈ 240–270 | **dois campos lado a lado, nomes ELIDIDOS** («Posit…», «Rotat…», «Scale…») | **igual** |
| ≈ 280 | lado a lado, nomes inteiros | igual |

⇒ **NÃO é desta linha** (o 12 e a grade não mexem na repartição da linha). É a lei da linha de formulário
(`property_box` / `Seccao`, `colunas_da_linha`, a cedência §6-ter «nunca abaixo do que o rótulo precisa»): numa
faixa de larguras ela prefere manter o controlo inteiro e corta o nome, em vez de EMPILHAR. Report com foto:
[`report_do_dono.png`](report_do_dono.png). Por fazer — próxima janela.
