# NetLab

**Laboratório virtual de redes e motor determinístico de simulação escrito em Rust.**

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Status](https://img.shields.io/badge/status-v0.2.0%20functional-success)](CHANGELOG.md)

O NetLab é um projeto experimental para estudar redes de computadores a partir
do zero. Ele modela hosts, interfaces, links, switches, endereços MAC e IPv4,
e executa a comunicação usando um relógio virtual e uma fila de eventos
determinística.

O objetivo não é apenas imprimir uma mensagem de ping: cada pacote possui uma
estrutura própria, passa por eventos simulados e mantém um histórico do seu
ciclo de vida.

## O que já funciona

- Workspace Rust organizado em cinco crates.
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

- Rust estável e Cargo.
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

O core não depende da CLI. A mesma simulação pode ser usada por testes,
ferramentas de linha de comando e uma futura interface gráfica.

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
- [ ] Editor visual de topologias.
- [ ] Integração futura com hardware, incluindo ESP32.

O planejamento detalhado está em [`docs/roadmap.md`](docs/roadmap.md), e a
especificação original está em [`PROJECT_SPEC.md`](PROJECT_SPEC.md).

## Limitações atuais

Ainda não há ARP, roteamento entre sub-redes, TCP, DNS, HTTP ou interface
gráfica. A versão atual concentra-se em uma LAN simulada com hosts e switches.

## Licença

Este projeto está disponível sob a licença MIT. Consulte [`LICENSE`](LICENSE).
