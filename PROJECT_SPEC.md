# NetLab

## Network Simulation & Virtual Internet Laboratory

**Projeto:** NetLab\
**Tipo:** Simulador de redes de computadores / laboratório virtual de redes\
**Autor:** Rafael Monteiro\
**Status inicial:** Desenvolvimento\
**Versão alvo deste documento:** v0.1.0\
**Plataforma inicial:** Linux / Windows\
**Arquitetura futura:** Desktop + CLI + Hardware ESP32

---

# 1. VISÃO GERAL

O NetLab será um simulador profissional de redes de computadores criado do zero.

O objetivo não é criar apenas uma animação visual de computadores trocando mensagens.

O objetivo é construir um **motor de simulação de redes capaz de representar, de maneira controlada e observável, o funcionamento de uma rede de computadores**, começando por conceitos simples e evoluindo posteriormente para:

- computadores virtuais;
- interfaces de rede;
- endereços MAC;
- endereços IPv4;
- switches;
- roteadores;
- ARP;
- ICMP;
- ping;
- UDP;
- TCP;
- DNS;
- HTTP;
- NAT;
- firewall;
- VLAN;
- sub-redes;
- tabelas de roteamento;
- perda de pacotes;
- latência;
- congestionamento;
- falhas de dispositivos;
- captura de pacotes;
- inspeção de protocolos;
- laboratório educacional;
- editor visual de topologias;
- CLI;
- monitor de rede;
- posteriormente dispositivos físicos;
- posteriormente integração com ESP32;
- posteriormente redes híbridas contendo dispositivos virtuais e reais.

O projeto deve ser construído de forma que a interface gráfica seja apenas uma camada sobre o motor.

A arquitetura fundamental deve ser:

```text
                         NETLAB
                            |
                    +-------+-------+
                    |               |
                Simulation        Hardware
                    |               |
              Virtual Network      ESP32
                    |
          +---------+---------+
          |         |         |
         CLI       GUI      Tests
```

O núcleo da simulação não pode depender da interface gráfica.

---

# 2. PRINCÍPIO FUNDAMENTAL

O NetLab deve ser tratado como um **motor de simulação de redes**, e não como um aplicativo visual.

A interface gráfica poderá mudar completamente no futuro sem exigir reescrever o núcleo.

Por exemplo:

```text
                NetLab Engine
                     |
       +-------------+-------------+
       |             |             |
      CLI           GUI          Tests
       |
       |
   Hardware Adapter
       |
     ESP32
```

O mesmo cenário deverá poder ser executado:

1. pelo CLI;
2. pelos testes automatizados;
3. futuramente pela interface gráfica;
4. futuramente utilizando hardware físico.

---

# 3. OBJETIVO DA PRIMEIRA VERSÃO

A primeira versão funcional será chamada:

# NetLab v0.1.0

Ela NÃO deve tentar implementar toda a Internet.

A primeira versão deve provar que o motor consegue simular corretamente uma comunicação básica entre dois computadores virtuais.

A topologia inicial será:

```text
+-------------+                  +-------------+
|     PC1     |                  |     PC2     |
|             |                  |             |
| 10.0.0.1    |                  | 10.0.0.2    |
+------+------+\                  +------+------+
       |                                |
       +------------ Network -----------+
```

O sistema deverá permitir:

- criar uma rede;
- criar dois computadores virtuais;
- atribuir IPs;
- criar interfaces de rede;
- gerar pacotes;
- enviar pacotes;
- receber pacotes;
- simular latência;
- executar ping;
- mostrar o caminho do pacote;
- registrar eventos;
- inspecionar o pacote;
- executar a simulação de forma determinística;
- executar testes automatizados.

---

# 4. TECNOLOGIA

## 4.1 Linguagem principal

Usar:

**Rust**

Motivos:

- desempenho;
- segurança de memória;
- excelente suporte para sistemas;
- ótimo para simuladores;
- ótimo para networking;
- excelente suporte para CLI;
- excelente sistema de testes;
- possibilidade futura de integração com C/C++;
- possibilidade futura de integração com ESP32;
- forte presença em ferramentas modernas de infraestrutura;
- permite construir um engine robusto.

---

# 5. REGRA IMPORTANTE SOBRE A TECNOLOGIA

Não implementar a primeira versão utilizando uma engine de jogos.

Não utilizar Unity.

Não utilizar Unreal.

Não utilizar Godot.

Não utilizar Raylib como núcleo.

Não utilizar navegador como motor da simulação.

A simulação deve existir independentemente da apresentação.

Uma futura interface poderá usar:

- Tauri;
- React;
- egui;
- wgpu;
- Bevy;
- outra tecnologia.

Mas isso não deve contaminar o núcleo.

---

# 6. ARQUITETURA DO PROJETO

Criar inicialmente:

```text
netlab/
│
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── PROJECT_SPEC.md
├── CHANGELOG.md
│
├── crates/
│   │
│   ├── netlab-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── simulation.rs
│   │       ├── scheduler.rs
│   │       ├── time.rs
│   │       ├── event.rs
│   │       ├── error.rs
│   │       └── config.rs
│   │
│   ├── netlab-network/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── network.rs
│   │       ├── interface.rs
│   │       ├── device.rs
│   │       ├── link.rs
│   │       ├── mac.rs
│   │       ├── ipv4.rs
│   │       └── topology.rs
│   │
│   ├── netlab-packets/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── packet.rs
│   │       ├── ethernet.rs
│   │       ├── ipv4.rs
│   │       ├── icmp.rs
│   │       └── payload.rs
│   │
│   ├── netlab-protocols/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── icmp.rs
│   │       └── mod.rs
│   │
│   └── netlab-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── examples/
│   ├── two_hosts.rs
│   └── ping.rs
│
├── tests/
│   ├── basic_network.rs
│   ├── packet_delivery.rs
│   ├── ping.rs
│   └── simulation.rs
│
├── docs/
│   ├── architecture.md
│   ├── networking.md
│   ├── simulation.md
│   ├── packets.md
│   └── roadmap.md
│
└── scenarios/
    └── basic_network.toml
```

Se houver uma razão arquitetural forte para alterar a estrutura, documentar a decisão antes de modificar.

---

# 7. WORKSPACE RUST

O projeto deve utilizar um Cargo Workspace.

A raiz deve conter:

```toml
[workspace]
resolver = "2"

members = [
    "crates/netlab-core",
    "crates/netlab-network",
    "crates/netlab-packets",
    "crates/netlab-protocols",
    "crates/netlab-cli",
]
```

O projeto deve possuir separação clara entre responsabilidades.

---

# 8. RESPONSABILIDADES DOS CRATES

## 8.1 netlab-core

Responsável pelo motor da simulação.

Deve conter:

- relógio virtual;
- eventos;
- scheduler;
- ciclo da simulação;
- gerenciamento de tempo;
- IDs de entidades;
- configuração;
- erros;
- abstrações fundamentais.

NÃO deve conhecer detalhes específicos de:

- CLI;
- GUI;
- TCP;
- HTTP;
- ESP32.

---

# 9. netlab-network

Responsável pela topologia.

Deve conter:

- dispositivos;
- computadores;
- interfaces;
- links;
- redes;
- endereços MAC;
- IPv4;
- topologia.

Futuras entidades:

```text
Host
Router
Switch
Server
Firewall
LoadBalancer
AccessPoint
```

---

# 10. netlab-packets

Responsável pela representação dos pacotes.

Inicialmente:

```text
Ethernet
IPv4
ICMP
Payload
```

Posteriormente:

```text
ARP
UDP
TCP
DNS
HTTP
DHCP
```

O pacote deve possuir representação estruturada.

Evitar representar pacotes simplesmente como:

```rust
String
```

ou:

```rust
Vec<String>
```

Criar tipos reais.

---

# 11. netlab-protocols

Responsável pela implementação dos protocolos.

Inicialmente:

```text
ICMP
```

Posteriormente:

```text
ARP
IPv4
UDP
TCP
DNS
HTTP
```

Protocolos devem operar sobre pacotes estruturados.

---

# 12. netlab-cli

Interface de linha de comando para controlar o simulador.

Exemplo futuro:

```text
netlab create host pc1
netlab create host pc2

netlab ip pc1 10.0.0.1
netlab ip pc2 10.0.0.2

netlab ping pc1 10.0.0.2
```

Na v0.1, pode existir uma CLI simplificada.

---

# 13. PRINCÍPIO DE DETERMINISMO

A simulação deve ser determinística sempre que possível.

Isso significa:

Dado:

```text
mesmo cenário
+
mesma configuração
+
mesma seed
```

o resultado deve ser igual.

Isso será extremamente importante no futuro para:

- testes;
- debugging;
- reprodução de bugs;
- laboratórios;
- gravação de simulações;
- replay;
- comparação de algoritmos.

---

# 14. TEMPO VIRTUAL

Não depender exclusivamente do relógio real do computador.

Criar um relógio virtual.

Exemplo:

```text
t = 0 ms

PC1 envia pacote

t = 1 ms

pacote atravessa link

t = 5 ms

PC2 recebe pacote
```

O simulador deve permitir futuramente:

```text
real-time
fast-forward
paused
step-by-step
```

Na v0.1, pelo menos:

```text
run
step
```

devem existir internamente.

---

# 15. EVENT-DRIVEN SIMULATION

A arquitetura deve ser baseada em eventos.

Exemplo:

```text
PacketCreated
PacketSent
PacketTransmitted
PacketReceived
PacketDropped
DeviceOnline
DeviceOffline
```

O scheduler deve ordenar eventos pelo tempo virtual.

Exemplo:

```text
EventQueue

0 ms  -> PacketCreated
1 ms  -> PacketSent
5 ms  -> PacketReceived
```

---

# 16. EVENTO

Criar uma estrutura conceitual semelhante a:

```rust
struct SimulationEvent {
    timestamp: SimulationTime,
    event_type: EventType,
    id: EventId,
}
```

Não copiar exatamente essa implementação se houver uma arquitetura melhor.

O objetivo é possuir eventos fortemente tipados.

---

# 17. DEVICE

Criar uma abstração para dispositivos.

Conceitualmente:

```text
Device
 ├── id
 ├── name
 ├── interfaces
 └── state
```

Estados:

```text
Online
Offline
```

No futuro:

```text
Booting
Degraded
Failed
```

---

# 18. HOST

O primeiro dispositivo real da simulação será o Host.

Exemplo:

```text
PC1

Device ID: 1
Name: PC1
MAC: 02:00:00:00:00:01
IPv4: 10.0.0.1
```

Outro:

```text
PC2

Device ID: 2
Name: PC2
MAC: 02:00:00:00:00:02
IPv4: 10.0.0.2
```

---

# 19. MAC ADDRESS

Criar um tipo próprio:

```rust
MacAddress
```

Não representar MAC apenas como String dentro do sistema.

O tipo deve:

- armazenar 48 bits;
- possuir validação;
- permitir comparação;
- possuir `Display`;
- possuir `Debug`;
- possuir parsing;
- possuir testes.

Exemplo:

```text
02:00:00:00:00:01
```

---

# 20. IPV4

Criar um tipo:

```rust
Ipv4Address
```

Não depender de strings.

Deve possuir:

- parsing;
- validação;
- comparação;
- display;
- conversão;
- suporte futuro a subnet.

Exemplo:

```text
10.0.0.1
```

---

# 21. NETWORK

Criar uma abstração de rede.

Exemplo:

```text
Network: LAN-1

10.0.0.0/24
```

A primeira versão pode utilizar uma única rede simples.

---

# 22. LINK

Um link conecta duas interfaces.

Exemplo:

```text
PC1 ---- Link ---- PC2
```

O link deve possuir futuramente:

```text
latency
bandwidth
packet_loss
jitter
enabled
```

Na v0.1:

```text
latency
enabled
```

são suficientes.

---

# 23. LATÊNCIA

Cada link deve poder possuir latência.

Exemplo:

```text
latency = 10 ms
```

Se PC1 enviar:

```text
t = 100 ms
```

o pacote não deve chegar imediatamente.

Deve chegar aproximadamente em:

```text
t = 110 ms
```

dependendo do modelo de transmissão adotado.

---

# 24. PACKET

Criar uma abstração de pacote.

Um pacote deve possuir pelo menos:

```text
Packet ID
Source
Destination
Protocol
Payload
Created At
```

Exemplo conceitual:

```text
Packet #42

Source: 10.0.0.1
Destination: 10.0.0.2
Protocol: ICMP
Payload: Echo Request
Created: 100ms
```

---

# 25. PACKET ID

Todo pacote deve possuir um ID único dentro da simulação.

Exemplo:

```text
Packet #1
Packet #2
Packet #3
```

Isso será muito importante para debugging e visualização futura.

---

# 26. PACKET LIFECYCLE

Todo pacote deve possuir um ciclo de vida rastreável.

Exemplo:

```text
CREATED
   |
   v
QUEUED
   |
   v
SENT
   |
   v
TRANSMITTING
   |
   v
RECEIVED
```

Ou:

```text
CREATED
   |
   v
SENT
   |
   v
DROPPED
```

---

# 27. PACKET TRACE

O simulador deve registrar o caminho do pacote.

Exemplo:

```text
Packet #15

PC1
 ↓
Link 1
 ↓
PC2
```

No futuro:

```text
PC1
 ↓
Switch1
 ↓
Router1
 ↓
Router2
 ↓
Server
```

---

# 28. ICMP

A primeira implementação de protocolo deve ser ICMP.

O objetivo é permitir:

```text
ping
```

Implementar inicialmente:

```text
ICMP Echo Request
ICMP Echo Reply
```

---

# 29. PING

Exemplo:

```text
PC1> ping 10.0.0.2
```

Resultado esperado:

```text
PING 10.0.0.2

Reply from 10.0.0.2:
time=10ms

Reply from 10.0.0.2:
time=10ms

Reply from 10.0.0.2:
time=10ms

--- statistics ---
sent = 3
received = 3
lost = 0
```

Os valores devem ser provenientes da simulação.

Não inventar resultados.

---

# 30. PACKET INSPECTOR

Mesmo que seja apenas CLI inicialmente, criar infraestrutura para inspeção.

Exemplo:

```text
Packet #12
--------------------------------
Ethernet
  Source MAC:      02:00:00:00:00:01
  Destination MAC: 02:00:00:00:00:02

IPv4
  Source IP:       10.0.0.1
  Destination IP:  10.0.0.2

ICMP
  Type: Echo Request
  Code: 0
```

Isso será essencial para a futura interface gráfica.

---

# 31. LOGGING

Criar um sistema de eventos/logs estruturado.

Exemplo:

```text
[0000ms] PC1 created
[0000ms] PC2 created
[0000ms] Link created
[0001ms] Packet #1 created
[0001ms] Packet #1 sent from PC1
[0011ms] Packet #1 received by PC2
[0011ms] ICMP Echo Reply generated
[0021ms] Packet #2 received by PC1
```

Evitar depender de `println!` espalhados pelo código.

Criar uma camada de observabilidade.

---

# 32. ERROR HANDLING

Não utilizar:

```rust
unwrap()
```

indiscriminadamente em código de produção.

Erros devem ser propagados corretamente.

Criar tipos de erro apropriados.

Exemplos:

```text
DeviceNotFound
InterfaceNotFound
InvalidAddress
NetworkMismatch
PacketDeliveryFailed
DeviceOffline
InvalidProtocol
```

---

# 33. TESTES

Testes são obrigatórios.

O projeto não deve ser considerado concluído apenas porque compila.

Criar testes para:

## Endereços

```text
MAC válido
MAC inválido
IPv4 válido
IPv4 inválido
```

## Topologia

```text
criar host
adicionar interface
criar link
conectar hosts
```

## Pacotes

```text
criar pacote
enviar pacote
receber pacote
```

## Simulação

```text
evento em ordem
latência respeitada
```

## ICMP

```text
Echo Request
Echo Reply
Ping bem-sucedido
Ping com destino inexistente
Ping para host offline
```

---

# 34. TESTE FUNDAMENTAL

Deve existir um teste equivalente a:

```text
Criar PC1
Criar PC2

PC1 = 10.0.0.1
PC2 = 10.0.0.2

Conectar PC1 e PC2

Executar ping

Esperado:

3 pacotes enviados
3 recebidos
0 perdidos
```

Esse teste deve funcionar automaticamente.

---

# 35. TESTE DE LATÊNCIA

Criar:

```text
PC1 ---- 50ms ---- PC2
```

Enviar ping.

O RTT esperado deve refletir a latência simulada.

Documentar claramente o modelo:

Se o link possui 10ms em cada direção:

```text
request = 10ms
reply = 10ms

RTT = 20ms
```

A implementação deve manter um modelo consistente.

---

# 36. TESTE DE HOST OFFLINE

Se:

```text
PC2 = Offline
```

e PC1 executar:

```text
ping 10.0.0.2
```

o ping deve falhar.

Não deve existir:

```text
fake reply
```

---

# 37. TESTE DE PACOTE PERDIDO

A arquitetura deve estar preparada para:

```text
packet_loss = 1.0
```

Nesse caso:

```text
100% dos pacotes
```

serão perdidos.

Não é necessário expor essa funcionalidade completamente na CLI da v0.1, mas o modelo deve permitir sua implementação.

---

# 38. SEED

Criar suporte para seed determinística.

Exemplo:

```text
seed = 12345
```

Se no futuro existir:

```text
packet_loss = 0.1
```

o comportamento poderá ser reproduzido.

---

# 39. CONFIGURAÇÃO

Permitir configuração por arquivo futuramente.

Formato recomendado:

```toml
simulation_seed = 12345

[simulation]
mode = "deterministic"

[network]
default_latency_ms = 10
```

---

# 40. CENÁRIOS

Criar suporte conceitual para cenários.

Exemplo:

```text
scenarios/basic_network.toml
```

Representação:

```toml
name = "Basic Network"

[[hosts]]
name = "PC1"
ip = "10.0.0.1"

[[hosts]]
name = "PC2"
ip = "10.0.0.2"

[[links]]
from = "PC1"
to = "PC2"
latency_ms = 10
```

A implementação pode ser simplificada na v0.1, mas a arquitetura deve permitir isso.

---

# 41. CLI

Criar comandos básicos.

Exemplo:

```text
netlab --help
```

Deve mostrar:

```text
NetLab

Network Simulation & Virtual Internet Laboratory

USAGE:
    netlab <COMMAND>

COMMANDS:
    scenario
    run
    inspect
    ping
    version
    help
```

---

# 42. COMANDO VERSION

Implementar:

```text
netlab version
```

Resultado:

```text
NetLab 0.1.0
```

---

# 43. COMANDO RUN

Permitir algo semelhante a:

```text
netlab run scenarios/basic_network.toml
```

Resultado:

```text
Loading scenario...
Creating network...
Creating devices...
Creating links...

Simulation ready.

PC1: 10.0.0.1
PC2: 10.0.0.2

Simulation started.
```

---

# 44. COMANDO PING

Permitir:

```text
netlab ping PC1 10.0.0.2
```

ou uma forma equivalente coerente com a arquitetura.

---

# 45. COMANDO INSPECT

Permitir futuramente:

```text
netlab inspect packet 42
```

Resultado:

```text
Packet #42

Ethernet
...

IPv4
...

ICMP
...
```

---

# 46. DOCUMENTAÇÃO

Criar:

```text
README.md
```

O README deve parecer um projeto real de engenharia.

Deve conter:

- nome;
- descrição;
- objetivo;
- arquitetura;
- status;
- instalação;
- execução;
- exemplo;
- testes;
- roadmap;
- arquitetura;
- screenshots futuramente;
- licença.

---

# 47. README

O README deve explicar que o projeto está sendo desenvolvido em fases.

Exemplo:

```text
NetLab is a network simulation and virtual networking laboratory
built from scratch in Rust.

It aims to provide an observable and deterministic environment
for experimenting with networking concepts, protocols and
distributed systems.
```

Não exagerar dizendo que o projeto implementa recursos ainda inexistentes.

---

# 48. DOCUMENTAÇÃO TÉCNICA

Criar:

```text
docs/architecture.md
```

Explicar:

```text
Simulation Engine
Network Model
Devices
Interfaces
Links
Packets
Protocols
Scheduler
Events
Observability
```

---

# 49. DOCUMENTAÇÃO DE SIMULAÇÃO

Criar:

```text
docs/simulation.md
```

Explicar:

- tempo virtual;
- eventos;
- scheduler;
- determinismo;
- latência;
- packet lifecycle;
- seeds.

---

# 50. DOCUMENTAÇÃO DE REDES

Criar:

```text
docs/networking.md
```

Explicar os conceitos reais representados.

Exemplo:

```text
MAC
IPv4
Ethernet
ICMP
Ping
LAN
```

Cada conceito deve explicar:

1. o que é;
2. por que existe;
3. como funciona;
4. como o NetLab representa isso.

---

# 51. PRINCÍPIO EDUCACIONAL

O NetLab não deve esconder a implementação.

Sempre que possível, o usuário deverá conseguir observar:

```text
o que aconteceu
```

e:

```text
por que aconteceu
```

Por exemplo:

```text
PC1 enviou ICMP Echo Request.

Source:
10.0.0.1

Destination:
10.0.0.2

O pacote atravessou:

PC1
↓
Link
↓
PC2

PC2 gerou Echo Reply.
```

---

# 52. FUTURA INTERFACE GRÁFICA

A GUI NÃO deve ser implementada na primeira fase caso isso atrase o engine.

A arquitetura deve estar preparada para ela.

Futuramente:

```text
+-------------------------------------------------------+
| NetLab                                      [RUN]     |
+-------------------------------------------------------+
|                                                       |
|      +------+                     +------+            |
|      | PC1  |---------------------| PC2  |            |
|      |10.0.1|       10ms          |10.0.2|            |
|      +------+                     +------+            |
|                                                       |
+-------------------------+-----------------------------+
| Packet Inspector        | Event Log                  |
|                         |                             |
| ICMP Echo Request       | 001ms PC1 -> PC2            |
| Src: 10.0.0.1           | 011ms Packet received       |
| Dst: 10.0.0.2           | 011ms Reply generated       |
+-------------------------+-----------------------------+
```

---

# 53. FUTURO EDITOR VISUAL

A interface deverá futuramente permitir:

```text
drag and drop
```

de:

```text
PC
Server
Router
Switch
Firewall
```

E permitir conectar dispositivos visualmente.

Exemplo:

```text
             +-------+
             | PC1   |
             +---+---+
                 |
                 |
             +---+---+
             | SW1   |
             +---+---+
              /     \
             /       \
      +-----+         +-----+
      | PC2 |         | PC3 |
      +-----+         +-----+
```

---

# 54. FUTURA ANIMAÇÃO DE PACOTES

Pacotes poderão ser representados visualmente:

```text
PC1
 |
 |   ●
 |    ●
 |     ●
 v
Switch
```

Cada pacote deve estar ligado ao Packet ID real do engine.

A GUI nunca deve inventar a animação.

Ela deve visualizar eventos reais produzidos pelo engine.

---

# 55. FUTURO PACKET CAPTURE

Criar futuramente um modo semelhante a um analisador de pacotes.

Exemplo:

```text
No.   Time    Source      Destination   Protocol
1     0.001   10.0.0.1    10.0.0.2      ICMP
2     0.011   10.0.0.2    10.0.0.1      ICMP
```

Ao clicar:

```text
Packet #1
```

mostrar:

```text
Ethernet
IPv4
ICMP
Payload
```

---

# 56. ROADMAP DE PROTOCOLOS

Depois da v0.1:

## v0.2

```text
Ethernet
MAC
Switch
```

## v0.3

```text
ARP
```

## v0.4

```text
IPv4
Subnetting
Routing
Router
```

## v0.5

```text
ICMP
Ping
Traceroute
```

## v0.6

```text
UDP
```

## v0.7

```text
TCP
```

## v0.8

```text
DNS
```

## v0.9

```text
HTTP
```

## v1.0

```text
Virtual Internet
```

---

# 57. FUTURO SWITCH

Switch deve possuir:

```text
MAC address table
ports
forwarding
broadcast
```

Exemplo:

```text
MAC Table

02:00:00:00:00:01 -> Port 1
02:00:00:00:00:02 -> Port 2
```

O switch deverá aprender MACs conforme os pacotes trafegam.

---

# 58. FUTURO ARP

ARP deverá representar:

```text
Who has 10.0.0.2?
```

Resposta:

```text
10.0.0.2 is at
02:00:00:00:00:02
```

O host deverá possuir uma ARP cache.

Exemplo:

```text
ARP Cache

10.0.0.2
02:00:00:00:00:02
```

---

# 59. FUTURO ROUTER

O Router deverá possuir:

```text
interfaces
IP addresses
routing table
packet forwarding
```

Exemplo:

```text
Routing Table

10.0.0.0/24 -> eth0
10.0.1.0/24 -> eth1
0.0.0.0/0   -> eth2
```

---

# 60. FUTURO SUBNETTING

O sistema deverá compreender:

```text
10.0.0.0/24
10.0.1.0/24
10.0.2.0/24
```

E permitir testar comunicação entre sub-redes.

---

# 61. FUTURO UDP

Implementar:

```text
source port
destination port
payload
```

Permitir:

```text
client -> server
```

---

# 62. FUTURO TCP

TCP será implementado somente depois de a infraestrutura estar madura.

Deverá futuramente representar conceitos como:

```text
SYN
SYN-ACK
ACK
sequence number
acknowledgement number
window
FIN
RST
```

Não implementar TCP fake apenas para parecer funcional.

---

# 63. FUTURO DNS

Criar servidores DNS virtuais.

Exemplo:

```text
dns.netlab
```

Resolver:

```text
server.local
```

para:

```text
10.0.0.10
```

---

# 64. FUTURO HTTP

Criar servidores HTTP virtuais.

Exemplo:

```text
GET /index.html
```

Resposta:

```text
HTTP/1.1 200 OK
```

Isso permitirá futuramente criar uma pequena "Internet virtual".

---

# 65. FUTURA INTERNET VIRTUAL

A evolução desejada é:

```text
                 NetLab Internet

                     Router
                    /      \
                   /        \
              Network A    Network B
                /              \
             PC1                Server
                                |
                              HTTP
```

Usuário poderá criar:

```text
website.local
```

e acessá-lo dentro da simulação.

---

# 66. FUTURO FIREWALL

Firewall virtual:

```text
ALLOW
DENY
```

Exemplo:

```text
ALLOW TCP 80
ALLOW TCP 443
DENY TCP 22
```

Deverá gerar eventos explicando bloqueios.

---

# 67. FUTURO NAT

Permitir simular:

```text
Private Network
        |
       NAT
        |
Public Network
```

Exemplo:

```text
192.168.1.10
      |
     NAT
      |
203.0.113.10
```

---

# 68. FUTURO CONGESTIONAMENTO

Links poderão possuir:

```text
bandwidth
queue
buffer
```

Exemplo:

```text
100 Mbps
```

Pacotes podem esperar na fila.

Se o buffer estiver cheio:

```text
Packet dropped
```

---

# 69. FUTURAS FALHAS

O simulador deverá permitir:

```text
device offline
link down
packet loss
high latency
high jitter
congestion
```

Isso permitirá laboratórios como:

```text
"Descubra por que o PC2 não consegue acessar o servidor."
```

---

# 70. FUTURO LABORATÓRIO EDUCACIONAL

Criar cenários:

```text
Lab 01
Conectando dois computadores

Lab 02
Configurando IP

Lab 03
Entendendo ARP

Lab 04
Configurando uma LAN

Lab 05
Criando sub-redes

Lab 06
Configurando roteamento

Lab 07
Analisando ICMP

Lab 08
Configurando DNS

Lab 09
Configurando HTTP

Lab 10
Diagnóstico de falhas
```

---

# 71. FUTURO MODO DEBUG

O usuário poderá pausar:

```text
PAUSED
```

e avançar:

```text
STEP
```

um evento por vez.

Exemplo:

```text
Event #1
PC1 created

[STEP]

Event #2
PC2 created

[STEP]

Event #3
Packet #1 created

[STEP]

Event #4
Packet #1 transmitted
```

Isso será extremamente importante para aprendizado.

---

# 72. FUTURO REPLAY

A simulação deverá futuramente permitir:

```text
record
replay
pause
resume
step
rewind
```

Para isso, manter eventos suficientemente estruturados.

---

# 73. FUTURA API

No futuro poderá existir uma API:

```text
NetLab Engine
       |
       +---- CLI
       |
       +---- GUI
       |
       +---- REST API
       |
       +---- WebSocket
       |
       +---- ESP32
```

Não implementar isso agora.

A arquitetura deve apenas evitar bloquear essa possibilidade.

---

# 74. ESP32 — FUTURO

A integração com ESP32 NÃO faz parte da v0.1.

Porém, toda a arquitetura deve considerar que no futuro existirão:

```text
Virtual Device
Real Device
```

Exemplo:

```text
                 NetLab

       Virtual PC
           |
       Virtual Router
           |
       Physical ESP32
           |
       Physical ESP32
           |
       Physical PC
```

---

# 75. OBJETIVO DA INTEGRAÇÃO ESP32

No futuro, será possível criar uma rede híbrida:

```text
                 NetLab

             +-----------+
             | Virtual PC|
             +-----+-----+
                   |
                   |
             +-----+-----+
             | Simulator |
             +-----+-----+
                   |
                   |
             +-----+-----+
             |   ESP32   |
             +-----+-----+
                   |
                   |
             +-----+-----+
             |   ESP32   |
             +-----------+
```

Isso permitirá estudar:

- redes virtuais;
- redes físicas;
- protocolos;
- comunicação entre software e hardware;
- IoT;
- redes distribuídas.

---

# 76. REGRA DE SEGURANÇA PARA HARDWARE

Quando a integração com hardware começar, utilizar somente dispositivos e redes sob controle do usuário.

O NetLab deve ser projetado para:

```text
laboratório privado
```

e:

```text
ambiente controlado
```

Não criar funcionalidades destinadas a acessar redes de terceiros sem autorização.

---

# 77. FUTURO PROTOCOLO EXPERIMENTAL

Em fases posteriores poderá ser criado um protocolo experimental próprio.

Nome provisório:

```text
GNP
Gatha Network Protocol
```

Isso NÃO deve ser implementado na v0.1.

Apenas manter arquitetura extensível.

---

# 78. QUALIDADE DO CÓDIGO

O código deve seguir:

- Rust idiomático;
- módulos pequenos;
- responsabilidades únicas;
- tipos fortes;
- erros explícitos;
- testes;
- documentação;
- nomes claros;
- pouca duplicação;
- evitar abstrações desnecessárias.

Não criar código artificialmente complexo.

---

# 79. REGRA CONTRA OVERENGINEERING

Não implementar sistemas futuros antes da hora.

Não criar:

```text
TCP completo
DNS completo
GUI completa
ESP32
```

na v0.1.

A v0.1 deve ser pequena, mas sólida.

O objetivo é construir uma fundação que permita crescer.

---

# 80. DEFINIÇÃO DE PRONTO — v0.1

A v0.1 somente será considerada concluída quando:

- [ ] Workspace Rust funcionando.
- [ ] Todos os crates compilando.
- [ ] Core separado da CLI.
- [ ] Virtual clock implementado.
- [ ] Event scheduler implementado.
- [ ] Host implementado.
- [ ] Network implementada.
- [ ] Interface implementada.
- [ ] Link implementado.
- [ ] MAC address implementado.
- [ ] IPv4 implementado.
- [ ] Packet implementado.
- [ ] Ethernet básico representado.
- [ ] IPv4 packet representado.
- [ ] ICMP Echo Request implementado.
- [ ] ICMP Echo Reply implementado.
- [ ] Ping funcionando.
- [ ] Latência funcionando.
- [ ] Packet trace funcionando.
- [ ] Event log funcionando.
- [ ] CLI funcionando.
- [ ] Cenário básico funcionando.
- [ ] Testes automatizados funcionando.
- [ ] README completo.
- [x] Documentação técnica criada.
- [ ] `cargo test` funcionando.
- [ ] `cargo clippy` sem problemas graves.
- [ ] `cargo fmt --check` funcionando.
- [ ] Projeto compilando em modo release.

---

# 81. COMANDOS DE QUALIDADE

O projeto deve suportar:

```bash
cargo fmt
```

```bash
cargo check
```

```bash
cargo test
```

```bash
cargo clippy --workspace --all-targets --all-features
```

```bash
cargo build --release
```

Sempre que possível, manter esses comandos funcionando.

---

# 82. CI FUTURO

Preparar estrutura para GitHub Actions.

Futuramente:

```text
.github/
└── workflows/
    └── ci.yml
```

Pipeline:

```text
cargo fmt --check
cargo check
cargo test
cargo clippy
cargo build --release
```

Não é obrigatório implementar CI na primeira etapa se isso atrasar a implementação principal, mas a estrutura deve ser compatível.

---

# 83. GIT

Fazer commits pequenos e lógicos.

Exemplos:

```text
feat(core): add virtual simulation clock
feat(core): add event scheduler
feat(network): add virtual host
feat(network): add network links
feat(packet): add IPv4 packet model
feat(protocol): add ICMP echo request
feat(protocol): add ICMP echo reply
feat(cli): add ping command
test: add packet delivery tests
docs: add architecture documentation
```

Evitar commits gigantes quando possível.

---

# 84. NÃO MASCARAR ERROS

Se algo estiver quebrado:

NÃO:

```text
ignore error
```

NÃO:

```text
return success
```

NÃO:

```text
fake packet
```

NÃO:

```text
fake ping
```

O sistema deve representar corretamente o estado da simulação.

---

# 85. NÃO SIMULAR "FAKE NETWORKING"

O projeto deve ser uma simulação de conceitos reais.

Exemplo ruim:

```rust
println!("Ping successful!");
```

sem realmente:

- criar pacote;
- agendar evento;
- transmitir;
- receber;
- processar ICMP;
- gerar resposta.

O resultado deve ser consequência da simulação.

---

# 86. FLUXO CORRETO DO PING

O fluxo conceitual deve ser:

```text
User
 |
 | ping 10.0.0.2
 v
Host PC1
 |
 | create ICMP Echo Request
 v
Packet
 |
 | send
 v
Network
 |
 | schedule transmission
 v
Link
 |
 | latency
 v
Host PC2
 |
 | receive
 v
ICMP handler
 |
 | create Echo Reply
 v
Packet
 |
 | send
 v
Network
 |
 v
PC1
 |
 v
Ping Result
```

Esse fluxo deve ser representado por código real.

---

# 87. OBSERVABILIDADE

Cada etapa importante deve ser observável.

Exemplo:

```text
[0000ms] PC1 -> creating ICMP Echo Request
[0000ms] Packet #1 created
[0000ms] Packet #1 queued on link-1
[0010ms] Packet #1 delivered to PC2
[0010ms] PC2 -> ICMP Echo Request received
[0010ms] PC2 -> creating Echo Reply
[0020ms] Packet #2 delivered to PC1
```

---

# 88. IDENTIDADE DAS ENTIDADES

Criar IDs internos estáveis.

Exemplo:

```text
DeviceId(1)
InterfaceId(1)
LinkId(1)
PacketId(1)
EventId(1)
```

Não depender apenas de strings.

---

# 89. SERIALIZAÇÃO

Se conveniente, preparar tipos importantes para serialização.

Futuramente será útil para:

- cenários;
- save;
- load;
- replay;
- GUI;
- API.

Não adicionar serialização indiscriminadamente.

Usar apenas onde fizer sentido arquitetural.

---

# 90. PERFORMANCE

Não otimizar prematuramente.

Primeiro:

```text
correctness
```

Depois:

```text
performance
```

Mas evitar estruturas obviamente ruins.

Por exemplo, não fazer uma busca linear gigante para cada operação se a arquitetura naturalmente pede um mapa.

---

# 91. MEMÓRIA

A simulação deve evitar vazamentos e ciclos de ownership desnecessários.

Utilizar o sistema de ownership do Rust corretamente.

Não utilizar:

```text
Arc<Mutex<T>>
```

para absolutamente tudo.

Usar apenas quando realmente necessário.

---

# 92. CONCORRÊNCIA

A v0.1 pode ser single-threaded.

Isso é desejável inicialmente porque:

- facilita determinismo;
- facilita debugging;
- facilita testes;
- simplifica scheduler.

Arquitetura futura poderá utilizar múltiplos threads.

---

# 93. PRINCÍPIO DE SIMULAÇÃO

Não confundir:

```text
simulation time
```

com:

```text
wall-clock time
```

O computador pode executar:

```text
1 segundo de simulação
```

em:

```text
0.01 segundo real
```

ou:

```text
10 segundos reais
```

dependendo do modo.

---

# 94. MODOS FUTUROS

Planejar:

```text
REALTIME
FAST
PAUSED
STEP
```

Na v0.1, implementar somente o necessário.

---

# 95. DOCUMENTAÇÃO DE DECISÕES

Se uma decisão arquitetural importante for tomada, documentá-la.

Futuramente criar:

```text
docs/adr/
```

com:

```text
ADR-001-engine-language.md
ADR-002-event-driven-simulation.md
ADR-003-network-model.md
```

Não é necessário criar muitos ADRs artificialmente.

---

# 96. DESENVOLVIMENTO EM ETAPAS

O Codex deve trabalhar em etapas.

## ETAPA 1

Criar workspace.

Objetivo:

```text
cargo check
```

funcionar.

---

## ETAPA 2

Criar simulation core.

Implementar:

```text
SimulationTime
Event
EventQueue
Simulation
```

Criar testes.

---

## ETAPA 3

Criar network model.

Implementar:

```text
Device
Host
Network
Interface
Link
MacAddress
Ipv4Address
```

Criar testes.

---

## ETAPA 4

Criar packets.

Implementar:

```text
Packet
Ethernet
IPv4
ICMP
```

Criar testes.

---

## ETAPA 5

Integrar entrega de pacotes.

Implementar:

```text
Host -> Link -> Host
```

Criar testes.

---

## ETAPA 6

Implementar ICMP.

Criar:

```text
EchoRequest
EchoReply
```

---

## ETAPA 7

Implementar ping.

Exemplo:

```text
PC1 -> ping -> PC2
```

---

## ETAPA 8

Implementar CLI.

---

## ETAPA 9

Implementar cenários.

---

## ETAPA 10

Documentação.

---

## ETAPA 11

Testes finais.

---

# 97. CRITÉRIO DE CADA ETAPA

Ao terminar cada etapa:

1. compilar;
2. executar testes;
3. corrigir erros;
4. executar `cargo fmt`;
5. executar `cargo clippy`;
6. atualizar documentação;
7. verificar arquitetura;
8. só então avançar.

Não acumular dezenas de funcionalidades quebradas.

---

# 98. COMPORTAMENTO DO CODEX

O Codex deve agir como um engenheiro de software sênior.

Antes de escrever grandes quantidades de código:

1. analisar a estrutura;
2. identificar dependências;
3. planejar módulos;
4. implementar pequena etapa;
5. testar;
6. corrigir;
7. documentar;
8. avançar.

Não simplesmente gerar centenas de arquivos de uma vez.

---

# 99. REGRA DE IMPLEMENTAÇÃO

Sempre preferir:

```text
small + correct + tested
```

a:

```text
large + complicated + untested
```

---

# 100. DEPENDÊNCIAS

Adicionar dependências somente quando realmente necessárias.

Possíveis dependências futuras:

```text
thiserror
anyhow
clap
serde
toml
tracing
tracing-subscriber
rand
```

Não adicionar todas automaticamente.

Para cada dependência importante, avaliar se ela realmente agrega valor.

---

# 101. CLI

Se usar `clap`, estruturar a CLI de forma extensível.

Exemplo:

```text
netlab
├── scenario
├── run
├── ping
├── inspect
├── version
└── help
```

---

# 102. TRACING

Preferir sistema estruturado de logs.

Exemplo conceitual:

```rust
tracing::info!(
    packet_id = ?packet_id,
    source = ?source,
    destination = ?destination,
    "packet transmitted"
);
```

Isso será muito melhor que:

```rust
println!("packet sent");
```

---

# 103. TESTES DE INTEGRAÇÃO

Além dos testes unitários, criar testes de integração.

Exemplo:

```text
create network
create hosts
connect hosts
send packet
run simulation
verify result
```

---

# 104. TESTE END-TO-END

Criar pelo menos um teste que execute o fluxo completo:

```text
Scenario
    ↓
Simulation
    ↓
PC1
    ↓
ICMP
    ↓
Packet
    ↓
Link
    ↓
PC2
    ↓
Echo Reply
    ↓
PC1
    ↓
Success
```

---

# 105. CENÁRIO BÁSICO

O cenário inicial deve ser equivalente a:

```text
              10ms
       +----------------+
       |                |
       v                v

+------------+      +------------+
|    PC1     |------|    PC2     |
| 10.0.0.1   |      | 10.0.0.2   |
| MAC ...01  |      | MAC ...02  |
+------------+      +------------+
```

---

# 106. RESULTADO ESPERADO

Ao executar o cenário:

```text
NetLab 0.1.0

Loading scenario: basic_network

Devices:
  PC1 10.0.0.1
  PC2 10.0.0.2

Links:
  PC1 <-> PC2
  latency: 10ms

Simulation initialized.

Running ping...

[0000ms] ICMP Echo Request created
[0010ms] ICMP Echo Request received by PC2
[0010ms] ICMP Echo Reply created
[0020ms] ICMP Echo Reply received by PC1

Ping successful.

Packets:
  Sent: 1
  Received: 1
  Lost: 0

RTT:
  20ms
```

Os números devem vir da simulação.

---

# 107. O QUE NÃO FAZER NA v0.1

Não implementar:

- GUI completa;
- TCP completo;
- HTTP completo;
- DNS completo;
- roteamento dinâmico;
- BGP;
- Wi-Fi real;
- ESP32;
- Docker;
- Kubernetes;
- multiplayer;
- banco de dados;
- sistema de usuários;
- autenticação;
- cloud;
- IA.

Esses recursos pertencem ao roadmap futuro.

---

# 108. POR QUE NÃO IMPLEMENTAR TUDO AGORA?

Porque o objetivo da primeira versão é validar a arquitetura.

Se:

```text
Host
+
Network
+
Packet
+
Link
+
Event
+
ICMP
```

estiverem corretos, será possível construir o restante sobre uma base confiável.

---

# 109. VISÃO DE LONGO PRAZO

A visão final do projeto é:

```text
                         NETLAB
                           |
          +----------------+----------------+
          |                |                |
        ENGINE            GUI              CLI
          |
    +-----+-----+
    |           |
 Virtual      Physical
 Network      Network
    |           |
    |          ESP32
    |
+---+---------------------------------------+
|                                           |
|              Virtual Internet             |
|                                           |
| PC -- Switch -- Router -- Router -- DNS  |
|                    |                      |
|                 Firewall                  |
|                    |                      |
|                  Server                   |
|                    |                      |
|                   HTTP                    |
|                                           |
+-------------------------------------------+
```

---

# 110. OBJETIVO FINAL DO PROJETO

O usuário deverá conseguir abrir o NetLab e construir uma pequena Internet virtual.

Por exemplo:

```text
                  INTERNET VIRTUAL

                         |
                     Router A
                    /       \
                   /         \
              Network A    Network B
                /             \
             PC1              Switch
                               |
                       +-------+-------+
                       |       |       |
                      PC2    Server    DNS
                              |
                             HTTP
```

E então observar:

```text
PC1
 ↓
ARP
 ↓
Switch
 ↓
Router
 ↓
Router
 ↓
Server
 ↓
HTTP Response
```

Tudo isso dentro de uma simulação determinística e observável.

---

# 111. VISÃO DE HARDWARE

Posteriormente:

```text
               NETLAB

          Virtual Network
                |
                |
          +-----+-----+
          |           |
       ESP32 A      ESP32 B
          |           |
          +-----+-----+
                |
             Physical
              Network
```

Isso permitirá transformar o NetLab em uma ponte entre:

```text
Computer Networking
+
Simulation
+
Systems Programming
+
Embedded Systems
+
IoT
```

---

# 112. PRIMEIRA MISSÃO DO CODEX

Comece somente pela v0.1.

Não pule para funcionalidades futuras.

A ordem obrigatória é:

```text
1. Workspace
2. Core
3. Virtual time
4. Scheduler
5. Network model
6. Host
7. Interface
8. Link
9. MAC
10. IPv4
11. Packet
12. Ethernet model
13. IPv4 packet
14. ICMP
15. Packet delivery
16. Ping
17. Logging
18. CLI
19. Scenario
20. Tests
21. Documentation
```

---

# 113. DEFINITION OF DONE

O trabalho estará concluído quando for possível executar algo equivalente a:

```bash
cargo test --workspace
```

e todos os testes passarem.

Também:

```bash
cargo clippy --workspace --all-targets --all-features
```

sem erros críticos.

E:

```bash
cargo build --release
```

compilar corretamente.

Finalmente:

```bash
cargo run -p netlab-cli -- run scenarios/basic_network.toml
```

deve executar a simulação.

---

# 114. CHECKLIST FINAL

## Arquitetura

- [ ] Workspace Rust
- [ ] Core independente
- [ ] Network independente
- [ ] Packets independente
- [ ] Protocols independente
- [ ] CLI independente

## Simulation

- [ ] Virtual clock
- [ ] Events
- [ ] Scheduler
- [ ] Deterministic execution
- [ ] Step support
- [ ] Event logging

## Network

- [ ] Device
- [ ] Host
- [ ] Interface
- [ ] Network
- [ ] Link
- [ ] MAC
- [ ] IPv4

## Packets

- [ ] Packet ID
- [ ] Ethernet
- [ ] IPv4
- [ ] ICMP
- [ ] Payload
- [ ] Packet lifecycle
- [ ] Packet trace

## Protocols

- [ ] ICMP Echo Request
- [ ] ICMP Echo Reply

## Functionality

- [ ] Ping
- [ ] Latency
- [ ] Host offline
- [ ] Packet delivery
- [ ] Packet loss architecture

## CLI

- [ ] help
- [ ] version
- [ ] run
- [ ] ping
- [ ] inspect

## Quality

- [ ] Unit tests
- [ ] Integration tests
- [ ] End-to-end test
- [ ] cargo fmt
- [ ] cargo check
- [ ] cargo clippy
- [ ] cargo test
- [ ] release build

## Documentation

- [ ] README
- [ ] Architecture
- [ ] Simulation
- [ ] Networking
- [ ] Roadmap
- [ ] Changelog

---

# 115. INSTRUÇÃO FINAL PARA O CODEX

Você está implementando o NetLab como um projeto real de engenharia de software.

Não trate este documento como uma sugestão.

Use-o como especificação técnica.

Priorize:

1. corretude;
2. arquitetura;
3. testes;
4. determinismo;
5. observabilidade;
6. documentação;
7. extensibilidade;
8. simplicidade.

Não implemente funcionalidades futuras apenas para aumentar a quantidade de código.

A v0.1 deve ser pequena, funcional e tecnicamente sólida.

Quando uma decisão não estiver especificada:

1. escolha a solução mais simples;
2. preserve a separação de responsabilidades;
3. evite acoplamento;
4. documente decisões importantes;
5. escreva testes;
6. não invente funcionalidades.

Não criar uma GUI antes de o engine estar funcionando.

Não criar uma simulação falsa.

Não criar resultados artificiais.

Não esconder erros.

Não usar strings onde tipos fortes forem apropriados.

Não espalhar `println!` pelo sistema.

Não utilizar `unwrap()` indiscriminadamente.

Não adicionar dependências sem necessidade.

Não implementar TCP/DNS/HTTP/ESP32 na v0.1.

Construa primeiro a fundação.

---

# 116. RESULTADO ESPERADO

Ao final da v0.1, o repositório deve transmitir a impressão de:

> "Este é o começo de um verdadeiro motor de simulação de redes."

e não:

> "Este é um programa que imprime mensagens fingindo ser uma rede."

O primeiro marco técnico do projeto é:

```text
PC1
 |
 | ICMP Echo Request
 |
 v
Link
 |
 | 10ms
 |
 v
PC2
 |
 | ICMP Echo Reply
 |
 v
Link
 |
 | 10ms
 |
 v
PC1
```

Tudo isso deve ocorrer dentro de um motor de simulação:

```text
             NetLab Engine
                   |
        +----------+----------+
        |                     |
   Event Scheduler        Network Model
        |                     |
        +----------+----------+
                   |
                Packets
                   |
                Protocols
                   |
                  CLI
```

Depois que isso estiver funcionando corretamente, o projeto poderá evoluir para:

```text
PC
 ↓
Switch
 ↓
Router
 ↓
Router
 ↓
Firewall
 ↓
DNS
 ↓
HTTP Server
```

e posteriormente:

```text
Virtual Network
       |
       v
     ESP32
       |
       v
Physical Network
```

Essa é a direção arquitetural oficial do NetLab.
