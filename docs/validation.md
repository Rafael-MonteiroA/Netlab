# Verificação — NetLab 0.4

Ambiente: Windows x64, Rust/Cargo 1.98.1, JDK 17, SDK Android 35.

| Verificação | Resultado |
| --- | --- |
| Testes Rust | 30 aprovados: 20 unitários/outros e 10 de comunicação TCP/WebSocket |
| Rustfmt | Aprovado |
| Clippy workspace/all-targets/all-features com -D warnings | Aprovado |
| Build release | Servidor e CLI Windows x64 gerados |
| JavaScript node --check | app.js, network-client.js e bluetooth-client.js aprovados |
| APK Android | javac → D8 → aapt2 → zipalign → apksigner concluídos |
| Assinatura APK | Esquemas v2 e v3 verificados; chave de desenvolvimento local |
| Manifesto do APK | Android 8+; Bluetooth; sem permissão INTERNET |
| Dois clientes no navegador | Texto entregue; recebida 1/1 e exibida 1/1 |
| Atualização da aba | Sessão retomada, texto recuperado, uma única ocorrência na conversa |
| QR Code | SVG gerado e exibido; convite apontou para o IP Ethernet selecionado |
| Diagnóstico | Versão 0.4.0, interfaces locais e resposta HTTP retornadas |
| Bluetooth no PC | Diagnóstico de rádio indisponível; nenhum adaptador ativo detectado |
| Android via USB | adb devices sem dispositivos conectados |
| Teste físico Bluetooth | **Pendente: requer aparelhos e rádio** |

Os dez testes de comunicação usam sockets TCP reais em portas locais: broadcast,
mensagens diretas e ping/pong com três clientes; isolamento de salas; entradas e
convites inválidos; capacidade; origem de navegador; presença; retomada por token;
reenvio sem duplicação; expiração da janela; rejeição de confirmação falsificada
por um membro que não recebeu a mensagem. O teste Bluetooth cobre validação de
quadros, não o rádio ou drivers.

O build Java emite aviso de API de insets depreciada: ela foi mantida para acomodar
as barras do sistema desde Android 8. Não é erro de compilação. O projeto Gradle
é fornecido, mas o build validado usa scripts/Build-Android.ps1 diretamente.

Na 0.3 já haviam sido verificados o simulador na interface (RTT de 24 ms e perda
ao desligar link), os lançadores e a configuração Compose. Esses resultados não
equivalem a novo teste em Docker, publicação HTTPS ou ensaio de hardware na 0.4.

Não foram testados: instalação/execução Android em aparelho, transmissão Bluetooth
real, alcance, tela apagada/bateria, tráfego entre redes externas, carga, Docker
em execução ou layout Android em diferentes fabricantes. A interface web foi
inspecionada no navegador desktop; a largura de celular continua pendente.

Não houve publicação, compra, assinatura, alteração de firewall ou roteador.
Foram baixados o SDK oficial e dependências gratuitas para compilar localmente.
O roteiro de aceitação física está em [offline-bluetooth.md](offline-bluetooth.md).
