---
name: a-ruler-that-reads-what-the-layout-produced-oscillates
description: Uma régua de layout alimentada pelo que o layout PRODUZIU fecha um ciclo de dois quadros, e nenhuma régua de um quadro o vê
metadata:
  type: feedback
---

O `DockSides::from_published` decidia *«esta coluna está ocupada?»* cruzando os rects que os
painéis publicaram no quadro ANTERIOR — a cura certa para *«não há lista de nomes»*. Com a coluna
vazia, a área de desenho cresce para dentro dela (que é o que ela existe para permitir), o painel
da área passa a publicar um rect que a cobre, e no quadro seguinte a régua lê **ocupada**. Período
dois, a 60 Hz: uma faixa da largura do Inspector a piscar (report do dono, 2026-09-20).

> *a área cresce porque a coluna está livre; a coluna lê-se ocupada porque a área cresceu.*

**Why:** o ciclo só arma num REGIME — aqui, quando o painel da área cobre ≥ `COLUMN_TAKEN_FRAC`
(`0,5`) da coluna. Eu tinha «medido e refutado» este mesmo mecanismo semanas antes, numa
configuração em que ele não arma (o Inspector aberto, ou o grafo pequeno). **E nenhum gate o via:
os que existiam alimentam a porta com rects escolhidos à mão e medem UM quadro — uma régua de um
quadro não pode ver um ciclo de dois.**

**How to apply:** quando uma régua de layout for alimentada por um FACTO que o próprio layout
produz, pergunte de que grandeza o ciclo depende e varra-a (aqui: a fracção da coluna coberta, e a
altura da janela). O gate corre **N quadros do laço** e exige que a leitura ASSENTE, com o controlo
que impede a cura barata (devolver sempre «livre» também pára o ciclo, e devolve o defeito original).
A cura foi de FORMA e sem lista: um inquilino de coluna **é** a coluna (`0,185` contra `1,000` de
área partilhada — vale de `5,4×`). Ver [[feedback-a-control-whose-range-comes-from-what-it-writes-is-a-feedback-loop]]
e [[feedback-a-fixture-where-the-two-are-siblings-cannot-produce-a-cycle]].
