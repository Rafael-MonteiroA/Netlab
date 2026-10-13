# Changelog

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
