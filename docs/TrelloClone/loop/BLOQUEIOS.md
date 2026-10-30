# Bloqueios — o que ficou por fazer e o que falta para destravar

| Onda | Item | O que falta |
|---|---|---|
| W10 | JSON real do Trello do dono | o dono exporta um quadro dele (Menu → Export as JSON) e ele vira a fixture e gate |
| W5 | Kanban ao vivo: furo do NAT entre DUAS máquinas reais | provado só em 127.0.0.1 (dois processos, LAN e QUIC sem ponto de encontro: `ph2d-kanban-sync` `loopback::two_processes_converge_*`); falta a corrida em duas redes de casa — a mesma pendência do Quadro (ADR-0187, «furo real por medir») |
| — | Consola do editor de scripts mostra TAB do `print` como «‖» | defeito pequeno de exibição da consola (módulo CodeEditor), visto na foto 8.2 |
| D11 | Fundo: a secção «Photos» lista só as imagens que o dono subiu pelo «Upload…», não as imagens que o projeto já tem | o projeto guarda imagens por referência (decisão do dono 26/10); falta o seletor de assets do projeto | `bg/mod.rs` |
| D11 | Fundo: o desenho de reserva do canvas SEM a família Kanban mostra azul liso para um fundo de imagem (nunca no app real, onde a família está sempre registada) | o canvas sem família não tem como ler o ficheiro | `ph2d-board-render/src/kanban_bg.rs` |
