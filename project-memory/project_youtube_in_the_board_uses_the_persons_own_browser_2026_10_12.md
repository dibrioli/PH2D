---
name: project_youtube_in_the_board_uses_the_persons_own_browser_2026_10_12
description: "Dono decidiu (12/10) — o vídeo do YouTube no Quadro toca pela «janela virtual» do navegador família-Chrome já instalado (CDP sem ecrã), não por motor embutido; Servo = plano B quando tiver MSE"
metadata:
  node_type: memory
  type: project
  originSessionId: 3f5962ff-ef6c-4e64-9664-e220af422a61
  modified: 2026-10-12T13:57:24.718Z
---

O dono decidiu em 2026-10-12 (line/MiroClone W18): o vídeo do YouTube DENTRO do quadro toca pelo navegador
da família Chrome que a pessoa JÁ tem (Chrome, Chromium, Edge, Brave, Vivaldi, Opera), lançado invisível
(sem ecrã) e controlado por CDP: os quadros vêm por screencast e são desenhados no canvas; o rato vai por
`Input.dispatchMouseEvent`. Sem navegador dessa família: miniatura + «Abrir no navegador». Zero MB e zero
instalação.

**Why:** medido em `docs/MiroClone/ferramentas/youtube_motor/README.md` — Chrome por CDP: 29,97 q/s, som pelo
sistema, clique em 7–28 ms; Servo 0.7 embutido toca mas só em 360p (sem MSE), API quebra todo mês; WebKitGTK
não toca (faltam codecs); CEF +364 MB; janela-filha (`wry`) impossível no Wayland. O dono perguntou por «uma
alternativa mais inteligente que não exige instalar nada» — foi ideia dele.

**How to apply:** não reabrir CEF/WebKit/Servo-embutido sem fato novo (ex.: Servo com MSE → plano B). No iPad
só o WKWebView por cima (quando houver versão iPad); no navegador (versão web), um `<iframe>`. O motor fica
atrás de uma porta trocável. Ligações: [[project_web_shell_decided_by_owner_2026_10_06]].

**W19 (12/10, depois do smoke):** o dono decidiu «nossos controles + Premium» — o tocador vem sem os controles do
YouTube e a NOSSA barra fica por baixo do vídeo (as regras proíbem nada à frente). O Premium foi MEDIDO: sem
sessão, o tocador embutido não mostrou anúncios (0 de 6 em vídeos monetizados) ⇒ não se constrói o perfil com a
sessão da pessoa; só com um anúncio visto pelo dono. ADR-0185 §W19.
