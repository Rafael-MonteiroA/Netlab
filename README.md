# NetLab

**Laboratório visual de redes e comunicação real entre aparelhos, escrito em Rust.**

## Novo: Android e Windows 0.7

Histórico Bluetooth persistente e protegido, recuperação de rascunhos e fila,
controles de privacidade, acessibilidade e relatório de suporte revisável.
[Guia das novidades e limites](docs/privacy-history-support.md).

- [Baixar APK Android 0.7](android/NetLab-Bluetooth-0.7.apk?raw=true) — instalar como atualização.
- [Baixar Windows 0.7](releases/NetLab-Windows-0.7.zip?raw=true) — extrair e abrir `Iniciar-NetLab.cmd`.
- [SHA-256 dos instaladores](releases/SHA256SUMS.txt).

Versão 0.7 testada nos aparelhos reais, com funcionamento confirmado pelo usuário.

## Conectar por QR Code

No PC, abra Bluetooth sem Wi-Fi e gere o código para o celular pareado. No Android,
use Aparelhos → Escanear QR Code. A conexão continua por Bluetooth, sem Wi-Fi ou
internet. [Passo a passo para testar](docs/qr-bluetooth.md).

Interface renovada com React, Motion, Radix e Lucide: temas claro/escuro, menu
animado, navegação por abas, mensagens em balões com horários e confirmações.
Todos os recursos visuais ficam no APK e abrem offline. O servidor Windows
agora inclui o fluxo de QR Code na versão 0.6.

A atualização 0.5.1 corrige o fechamento imediato do APK 0.5 durante a abertura.

**[Instalar o novo APK e conhecer a interface](docs/android-ui.md)**

## Recursos da versão 0.4

- Aplicativo Android e ponte Windows para texto por Bluetooth, sem internet e sem Wi-Fi.
- Android ↔ Android ou Android ↔ Windows, um par Bluetooth por vez.
- Reconexão automática, fila, proteção contra duplicatas e confirmações de recebimento/exibição.
- QR Code de convite, seleção de endereço e diagnóstico para o modo de rede local.
- Comunicação física da versão 0.7 validada pelo usuário.

**[Começar pelo Bluetooth e instalar o APK](docs/offline-bluetooth.md)**

## Interface gráfica e rede local

O NetLab agora inclui interface gráfica no navegador, salas privadas para até
16 clientes, mensagens em tempo real (para todos ou para um destinatário), mapa
de conexões e teste de latência entre clientes. No laboratório visual, é possível
editar hosts, endereços e latências, desligar links e inspecionar os pacotes
calculados pelo motor original.

**Windows:** abra `Iniciar-NetLab.cmd` no pacote que inclui o executável.
Use `Parar-NetLab.cmd` para encerrar o servidor.

**A partir do código-fonte:**

```sh
cargo run --locked -p netlab-server
```

Abra `http://localhost:8787`. Em outro aparelho da mesma rede, use
`http://IP-DO-COMPUTADOR:8787` e entre com o convite. Pela internet, hospede a mesma
instância com HTTPS; há Dockerfile e Compose com Caddy incluídos.

**[Guia completo de conexão, implantação e limitações](docs/real-network.md)**

O modo de rede IP usa Wi-Fi/cabo/internet já disponíveis, através de um servidor
intermediário. O modo Bluetooth usa diretamente os rádios dos aparelhos. Não cria infraestrutura de internet nem uma VPN. O laboratório
simulado e a comunicação real são modos distintos da mesma interface.

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Status](https://img.shields.io/badge/status-v0.4.0%20experimental-success)](CHANGELOG.md)

O NetLab é um projeto experimental para estudar redes de computadores a partir
do zero. Ele modela hosts, interfaces, links, switches, endereços MAC e IPv4,
e executa a comunicação usando um relógio virtual e uma fila de eventos
determinística.

O objetivo não é apenas imprimir uma mensagem de ping: cada pacote possui uma
estrutura própria, passa por eventos simulados e mantém um histórico do seu
ciclo de vida.

## O que já funciona

- Workspace Rust organizado em seis crates, incluindo servidor e interface gráfica.
- Relógio virtual em milissegundos.
- Scheduler e fila de eventos determinísticos.
- Hosts com interfaces Ethernet, MAC e IPv4.
- Links com latência e estado habilitado/desabilitado.
- Switches com múltiplas portas.
- Aprendizado de endereços MAC.
- Forwarding unicast e flooding para destinos desconhecidos/broadcast.
- Pacotes Ethernet, IPv4 e ICMP estruturados.
- ICMP Echo Request/Echo Reply.
- Ping com RTT calculado pelos timestamps da simulação.
- Cenários TOML executáveis pela CLI.
- Testes unitários, testes de integração e validação com Clippy.

## Exemplo da simulação

```text
PC1 ── 5 ms ── SW1 ── 7 ms ── PC2
```

Um ping de PC1 para PC2 percorre dois links em cada sentido:

```text
PC1 → SW1 → PC2 → SW1 → PC1
```

O RTT esperado é de 24 ms: `(5 + 7) × 2`.

## Requisitos

- Para compilar o servidor: Rust estável e Cargo. O pacote Windows já inclui executáveis.
- Para Bluetooth: Android 8+ e Bluetooth clássico; no PC, Windows com adaptador compatível.
- Windows, Linux ou macOS.

Confira a instalação em [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

## Começando

```bash
git clone https://github.com/Rafael-MonteiroA/Netlab.git
cd Netlab
cargo check --workspace
cargo build --release
```

## Usando a CLI

```bash
# Ajuda
cargo run -p netlab-cli -- --help

# Versão
cargo run -p netlab-cli -- version

# Rede direta entre dois hosts
cargo run -p netlab-cli -- run scenarios/basic_network.toml

# LAN com switch
cargo run -p netlab-cli -- run scenarios/switched_network.toml

# Ping com parâmetros
cargo run -p netlab-cli -- ping scenarios/switched_network.toml PC1 10.0.0.2 3

# Inspecionar um pacote
cargo run -p netlab-cli -- inspect packet 1 scenarios/basic_network.toml
```

## Formato de cenário

Os cenários são arquivos TOML com hosts, switches, links e uma ação de ping:

```toml
name = "Switched Network"
version = "0.2"

[[hosts]]
name = "PC1"
mac = "02:00:00:00:00:01"
ip = "10.0.0.1"

[[switches]]
name = "SW1"
ports = 2

[[links]]
from = "PC1"
to = "SW1"
latency_ms = 5
enabled = true

[[actions]]
type = "ping"
from = "PC1"
target = "10.0.0.2"
count = 1
```

Há exemplos completos em [`scenarios/`](scenarios/).

## Arquitetura

| Crate | Responsabilidade |
| --- | --- |
| `netlab-core` | Tempo virtual, eventos, scheduler, configuração e erros |
| `netlab-network` | Hosts, switches, interfaces, links, MAC, IPv4 e topologia |
| `netlab-packets` | Ethernet, IPv4, ICMP, payloads e ciclo de vida dos pacotes |
| `netlab-protocols` | Comportamento dos protocolos e execução do ping |
| `netlab-cli` | Interface de linha de comando |
| `netlab-server` | Interface gráfica, API de simulação, WebSocket e ponte Bluetooth no Windows |
| `android/` | Aplicativo nativo Android para Bluetooth RFCOMM |

O core não depende da CLI. A mesma simulação pode ser usada por testes,
ferramentas de linha de comando e a interface gráfica atual.

## Determinismo

O NetLab ordena eventos por timestamp virtual e identificador. Assim, o mesmo
cenário produz a mesma sequência de eventos e resultados, facilitando testes
e reprodução de bugs.

## Testes e qualidade

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Roadmap

- [x] Fundação do motor de simulação.
- [x] Ethernet e switches.
- [ ] ARP e cache ARP nos hosts.
- [ ] Roteadores e roteamento IPv4 entre sub-redes.
- [ ] ICMP Time Exceeded e traceroute.
- [ ] UDP e sockets virtuais.
- [ ] TCP.
- [ ] DNS e HTTP virtuais.
- [x] Editor visual de topologias em estrela com hosts e switch.
- [x] Comunicação real entre clientes via servidor WebSocket.
- [ ] Integração futura com hardware, incluindo ESP32.

O planejamento detalhado está em [`docs/roadmap.md`](docs/roadmap.md), e a
especificação original está em [`PROJECT_SPEC.md`](PROJECT_SPEC.md).

## Limitações atuais

O motor de simulação ainda não tem ARP, roteamento entre sub-redes, TCP virtual,
DNS virtual ou HTTP virtual. A GUI edita LANs em estrela. A comunicação real usa
a pilha TCP/IP do sistema operacional via WebSocket e não instala interfaces de
rede virtuais. Acesso entre redes diferentes exige um servidor alcançável e HTTPS.

## Licença

Este projeto está disponível sob a licença MIT. Consulte [`LICENSE`](LICENSE).
