# 0.7.0 — 2026-10-14

- Histórico Bluetooth em SQLite criptografado, fila recuperável e rascunhos por aparelho.
- Exportação, exclusão confirmada, bloqueio e confirmação de exibição opcional.
- Proteção de tela Android, texto ampliado e melhorias de teclado e legibilidade.
- Diagnóstico local revisável sem conteúdo das conversas ou identificadores pessoais.
- Testes de persistência, falhas de armazenamento, isolamento e abertura Android 14.

# Changelog

## [0.6.0] - 2026-10-13

- Windows gera QR Code Bluetooth para conexão ao celular selecionado e inicia tentativas.
- Android oferece Aparelhos → Escanear QR Code, com leitor local ZXing e permissão de câmera.
- Convite validado, PC previamente pareado e recepção restrita ao PC confirmado.
- Novo pacote Windows pronto para executar e APK atualizável com a mesma assinatura.
- Guia em docs/qr-bluetooth.md; mantida a correção de empacotamento da versão 0.5.1.

## [Android 0.5.1] - 2026-10-13

- Corrigido fechamento imediato ao abrir o APK 0.5: o Android rejeitava o DEX
  por inconsistência nos cabeçalhos ZIP, após a combinação de jar e ZipArchive Update.
- APK agora é montado em um arquivo novo, com DEX e tabela de recursos sem compressão.
- Adicionado teste de instrumentação para abertura, renderização React, ponte nativa
  e bloqueio do envio antes de conectar. Veja scripts/Test-AndroidStartup.ps1.
- Mantidas assinatura, dados do aplicativo e comunicação compatível com Windows 0.4.

## [Android 0.5.0] - 2026-10-13

- Interface React embutida e offline, com Motion, Radix Dialog e ícones Lucide.
- Novo ícone, temas claro/escuro, menu animado, guia e navegação responsiva.
- Balões cronológicos com horário e confirmação; rascunho preservado em falhas.
- WebView restrita aos recursos internos; manifesto permanece sem INTERNET.
- Sete testes de interface aprovados e layouts móveis verificados no navegador.
- APK com a mesma assinatura da 0.4; compatível com o servidor Windows 0.4.
- Execução e Bluetooth em Android físico continuam pendentes de validação.

## [0.4.0] - 2026-10-12

- Aplicativo Android nativo e ponte Bluetooth RFCOMM local no Windows, sem Wi-Fi/internet.
- Android ↔ Android e Android ↔ Windows: um par por vez, pareamento do sistema, fila e confirmações.
- Retomada de salas por token privado, identidade estável, reconexão automática e reenvio com deduplicação.
- Confirmações de recebimento/exibição, presença offline e recuperação limitada de histórico.
- QR Code gerado localmente, seleção de interface de rede e diagnóstico HTTP/WebSocket.
- APK de desenvolvimento, build reproduzível por script e roteiro de testes físicos.
- Bluetooth compilado, mas ainda não validado em aparelhos reais nesta entrega.

## [0.3.0] - 2026-10-12

- Interface gráfica responsiva em português, servida pelo novo `netlab-server`.
- Salas privadas, convites, presença, mensagens diretas/broadcast e ping entre clientes reais via WebSocket.
- Laboratório visual conectado ao motor Rust, com edição de hosts/links, exportação/importação JSON e inspetor de pacotes.
- Limites de mensagens, filas e conexões, validação de origem, heartbeat e reconexão manual.
- Executável Windows com interface embutida, lançadores e configuração Docker/Caddy para implantação HTTPS.
- Corrigida colisão de IDs de interfaces ao adicionar hosts após switches.
- Testes de integração com múltiplos clientes TCP/WebSocket e documentação dos limites de rede.

## [0.2.0] - 2026-09-13

### Added

- Switches Ethernet com portas e tabela de MACs aprendida em tempo de execução.
- Forwarding unicast e flooding de destinos desconhecidos/broadcast.
- Topologias com múltiplos links e ping através de switch.
- Cenário `scenarios/switched_network.toml`.

## [0.1.0] - 2026-09-12

### Added

- Cargo Workspace com resolver 2.
- Crates iniciais para core, rede, pacotes, protocolos e CLI.
- CLI mínima com `--help` e `version`.
- Documentação arquitetural inicial.

### Implemented

- Tempo virtual, eventos e scheduler determinístico.
- Hosts, interfaces, links, endereços MAC/IPv4 e topologia.
- Pacotes Ethernet/IPv4/ICMP e ciclo de vida.
- Entrega direta Host → Link → Host com latência virtual.
- ICMP Echo Request/Reply e ping com RTT derivado dos eventos.
- CLI para executar cenários TOML e inspecionar pacotes.
- Testes automatizados e documentação da implementação.
