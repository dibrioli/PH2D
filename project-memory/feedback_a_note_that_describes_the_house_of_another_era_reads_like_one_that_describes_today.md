---
name: feedback-a-note-that-describes-the-house-of-another-era-reads-like-one-that-describes-today
description: Um doc-comment que era verdade quando foi escrito manda a próxima wave fazer a coisa errada, e ele lê-se exactamente como um que está certo
metadata:
  type: feedback
---

O doc do `PlatformPlayer` dizia, por escrito e com precedente citado: *«Componente NOVO ⇒ blob-key
própria ⇒ `PROJECT_SCHEMA` NÃO bumpa»*. Era **verdade na época dele** (o precedente do
`PhysicsJoint`/W3) e passou a ser **falso** no degrau `123`, quando um `ComponentBlob` de `type_id`
desconhecido passou a **recusar o load inteiro**.

Quem o seguisse landava um componente registado **sem degrau de schema**, e o sintoma seria um
projecto do dono a abrir com uma mensagem que não diz nada.

**Why:** uma nota de doc não tem data no corpo, e o leitor não distingue *«isto descreve a casa de
2026-06»* de *«isto descreve a casa de agora»*. É a mesma família da lista de itens abertos que manda
reconstruir trabalho já pago — e aqui é pior, porque a nota **prescreve** em vez de descrever.

**How to apply:** antes de seguir uma prescrição de doc-comment sobre um número que SOMA entre linhas
(schema, registos, contadores), **leia a escada** desse número e veja o que os últimos degraus
fizeram. Se a nota e a escada discordarem, a escada ganha — e corrija a nota no mesmo commit, com a
data e o mecanismo da mudança. Ver [[reference-topic-measurement-discipline]] e
[[feedback-the-ruler-and-the-product-shared-the-same-defect]].
