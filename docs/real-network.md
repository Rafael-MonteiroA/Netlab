# Comunicação real — NetLab 0.4

O projeto agora tem uma interface no navegador, um servidor Rust e salas privadas
para até 16 aparelhos. A comunicação é real: mensagens percorrem conexões TCP/
WebSocket entre clientes e o servidor. O laboratório continua usando o motor
determinístico original, em uma aba separada.

Para texto sem Wi-Fi, use o **[guia Bluetooth](offline-bluetooth.md)**.

## Começar no Windows

1. Extraia o pacote completo em uma pasta.
2. Dê dois cliques em `Iniciar-NetLab.cmd`. O navegador abre quando o servidor está pronto.
3. Preencha o nome do aparelho e clique em **Criar minha rede**.
4. Para encerrar, execute `Parar-NetLab.cmd`. Fechar o navegador não encerra o servidor.

O pacote Windows inclui `bin/netlab-server.exe`, com HTML, CSS e JavaScript
embutidos. Não exige Node, Python ou instalação de bibliotecas. O pacote de
código-fonte exige Rust estável; o atalho compila se não encontrar o executável.
O lançador executa o script apenas para aquela sessão do PowerShell; não muda a
política de execução do sistema. Logs e identificação do processo ficam em `work/`.

Alternativa pelo terminal (Windows, Linux e macOS):

```sh
cargo run --locked -p netlab-server -- --bind 0.0.0.0:8787
```

Depois abra `http://localhost:8787`. Para um laboratório restrito ao próprio PC,
use `--bind 127.0.0.1:8787`. Também existe a variável `NETLAB_BIND`.

## Conectar dois ou mais aparelhos na rede local

1. Conecte o computador servidor e os demais aparelhos ao mesmo Wi-Fi, hotspot
   ou rede cabeada. Não é necessário acesso à internet nessa rede.
2. No computador servidor, abra **Endereço e diagnóstico** e escolha o adaptador
   ligado ao outro aparelho. Confira o IPv4 também com `ipconfig` (Windows). Por exemplo, se ele for `192.168.1.20`, o endereço será
   `http://192.168.1.20:8787`.
3. Abra esse endereço **nos dois aparelhos**, crie uma rede no primeiro e use o
   convite para entrar nos outros. Qualquer navegador moderno serve; celulares
   não precisam instalar o servidor nem um aplicativo.
4. O QR Code contém o convite com o endereço selecionado; leia-o no outro aparelho.
   O código é gerado localmente. Depois envie texto para **Todos**, selecione um destinatário para uma mensagem direta
   ou clique em **Ping** ao lado de outro aparelho.

`localhost` e `127.0.0.1` sempre apontam para o próprio aparelho. Um convite com
`localhost` não abre o seu computador a partir de um celular. Nesse caso, abra o
IP do computador no celular e cole **apenas o código** exibido na interface.

Se o Windows mostrar uma solicitação do firewall, permita o NetLab na rede
privada em que você quer usá-lo. Nenhum script altera o firewall automaticamente.
Confira também isolamento de clientes/guest network no roteador. Não use
`0.0.0.0` como endereço no navegador; ele apenas indica onde o servidor escuta.

## Conectar aparelhos em redes diferentes pela internet

Os aparelhos precisam alcançar a mesma instância do servidor. A solução
fornecida usa um servidor público com domínio, Docker Compose e Caddy:

1. Em um servidor Linux com Docker/Compose, copie o projeto.
2. Aponte o registro DNS A (e AAAA, se aplicável) de um domínio seu para esse servidor.
3. Disponibilize as portas TCP 80/443. A porta 8787 fica somente na rede interna
   dos containers; não precisa ser publicada.
4. Copie `.env.example` para `.env` e substitua o domínio pelo seu domínio real.
5. Execute:

```sh
docker compose up -d --build
docker compose logs -f
```

6. Abra `https://SEU-DOMINIO` em ambos os aparelhos. Crie a rede e compartilhe o
   convite. O navegador usa automaticamente `wss://` quando a página usa HTTPS.

Caddy faz proxy de WebSocket e pode emitir/renovar o certificado HTTPS para um
domínio público com DNS e portas corretos. Veja a documentação oficial de
[reverse_proxy](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy)
e [HTTPS automático](https://caddyserver.com/docs/automatic-https).
Mantenha o cabeçalho Host original no proxy: o servidor compara Host e Origin
para conexões originadas de navegadores.

Também é possível executar o binário diretamente atrás de um proxy HTTPS ou
usar um túnel que transporte WebSocket e preserve Host/Origin. O projeto não
configura roteador, DNS, conta de hospedagem ou túnel automaticamente.
CGNAT costuma impedir conexões recebidas diretamente no computador doméstico;
um servidor público resolve isso sem abrir portas no roteador doméstico.

O Dockerfile e o Compose são configurações fornecidas para implantação. A
validação realizada nesta entrega foi no Windows, não em um servidor público.

## O que a “rede própria” significa aqui

```text
Aparelho A ── WebSocket ── Servidor NetLab ── WebSocket ── Aparelho B
                                │
                            Aparelho C
```

É uma rede de comunicação da aplicação, com salas e identificação própria dos
participantes. O IP de cada aparelho continua sendo fornecido pelo sistema e
pelo roteador. Não é uma VPN/TUN, um novo provedor, uma malha por rádio, roteamento
do tráfego de outros aplicativos nem uma implementação independente da internet.
Aba Laboratório: hosts/IPs virtuais; aba Conexões reais: clientes físicos/reais.
Cada aba do navegador conectada conta como um cliente, não como hardware detectado.

## Garantias, limites e privacidade

- Convites são identificadores aleatórios de 128 bits (UUID v4, com bits de
  versão/reserva), usados como chaves de acesso. Quem conhece o código pode
  entrar. Não há cadastro, senha adicional, administrador, expulsão ou revogação.
- Máximo de 16 clientes por sala, 32 salas e 64 conexões simultâneas por processo.
- Cada conexão tem fila limitada; clientes lentos não bloqueiam os demais.
  Há reenvio periódico dentro da janela de recuperação, sem garantia ilimitada.
- Até 4.000 caracteres por mensagem, 16 KiB por frame/mensagem WebSocket e
  600 mensagens de protocolo por janela de 10 segundos por cliente.
- O cliente mostra envio, confirmação de recebimento e exibição. Exibida significa
  conversa visível, não leitura humana. O cliente reenvia a fila e o servidor repete
  entregas sem confirmação a cada 15 s. Sequências estáveis impedem novo envio duplicado.
- Ping é de aplicação e inclui cliente → servidor → outro cliente → servidor →
  cliente. Não é ICMP real do sistema operacional. O outro cliente responde no
  navegador, portanto suspensão de uma aba/celular pode causar timeout.
- HTTP local não criptografa o tráfego. HTTPS protege cada conexão até o servidor;
  o servidor acessa o texto. Não há criptografia de ponta a ponta.
- Salas, participantes e as últimas 256 mensagens da sala ficam apenas na memória
  do servidor. A retomada recupera somente mensagens do próprio membro ou dirigidas
  a ele. Novos membros não recebem histórico antigo. Cada aba exibe até 256 itens.
- Uma mensagem retirada da janela de 256 eventos não é recuperável; reenvios antigos
  são recusados explicitamente. Não é um arquivo permanente de conversas.
- O navegador guarda localmente o nome do aparelho e a topologia simulada.
  Token de retomada, convite, sequência e até 50 mensagens pendentes ficam em
  sessionStorage, sobrevivendo à atualização da aba. O convite usa o fragmento da URL, evitando
  enviá-lo no caminho HTTP. Compartilhe-o apenas com participantes desejados.
- Há heartbeat a cada 15 segundos e desconexão após 45 segundos sem atividade.
  A interface tenta reconectar automaticamente com espera progressiva de 1–30 s e
  também oferece **Reconectar**. Membros offline e salas vazias podem ser retomados
  por 15 minutos; a limpeza ocorre na próxima criação/entrada.
- Reiniciar o servidor apaga todas as salas. Execute apenas uma réplica: não há
  sincronização de salas entre servidores nem persistência distribuída.
- É uma base funcional para pequenos grupos. Antes de um serviço público amplo,
  faltam contas/controle administrativo, limitação por IP no proxy, proteção
  contra abuso, auditoria de segurança e testes de carga.

## Laboratório visual

Adicione/remova hosts, edite nome/IPv4/latência e ligue/desligue links.
A topologia é uma estrela com um switch e 2–16 hosts; cada execução aceita 1–10
pings. Os RTTs e rastros são calculados pelo motor Rust original. Cada execução
começa com uma topologia nova, inclusive a tabela MAC do switch.

Exportar/importar usa `netlab-star-v1` em JSON (até 32 KiB), formato específico
da GUI. Cenários TOML existentes continuam disponíveis pela CLI. A GUI ainda
não importa o TOML, não permite roteadores, ciclos, links arbitrários ou arrastar
hosts. Não implementa ARP, TCP virtual, DNS virtual ou NAT.

## Verificação

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo build --release --locked
node --check crates/netlab-server/web/app.js
```

Os testes de integração abrem sockets TCP reais em portas locais dinâmicas,
criam três clientes WebSocket e verificam broadcast, mensagem direta, ida/volta,
isolamento de salas, convites inválidos, presença, reconexão e limites.
Isso verifica comunicação real entre processos/clientes locais. Para validar
seu Wi-Fi, firewall, roteador ou provedor, repita o roteiro de dois aparelhos.

## Protocolo para outros clientes

Conecte em `/ws` e envie, em até 10 segundos:

```json
{"type":"create","name":"PC1","room_name":"Minha rede"}
```

O servidor retorna `welcome` com `id`, `code`, `room_name`, `resume_token` e
`next_seq`, seguido de `peers` (inclui `online`). Guarde o token privadamente.
Para retomar: `{"type":"resume","code":"CODIGO","token":"TOKEN"}`.
Outro cliente usa `{"type":"join","name":"PC2","code":"CODIGO"}`.
Depois:

```json
{"type":"chat","text":"Olá","to":null,"client_seq":1}
{"type":"chat","text":"Direta","to":"ID-DESTINATARIO","client_seq":2}
{"type":"receipt","id":"ID-DA-MENSAGEM","read":false}
{"type":"ping","to":"ID-DESTINATARIO","nonce":"teste-1"}
{"type":"pong","to":"ID-ORIGEM","nonce":"teste-1"}
{"type":"heartbeat"}
```

O servidor sempre define `from`; não confia na identidade enviada pelo cliente.
Um cliente deve responder `pong` ao remetente de `ping`, com o mesmo `nonce`.
A retomada por token preserva a identidade. O mesmo client_seq não cria nova entrega.
`receipt` enviado ao autor inclui total, received e read. Erros de chat incluem
client_seq para retirar mensagens inválidas da fila; expired informa perda da janela.
Uma segunda retomada substitui a conexão anterior (evento replaced). Clientes nativos podem omitir Origin; navegadores
devem usar a mesma origem da interface. `error` contém uma mensagem legível.
