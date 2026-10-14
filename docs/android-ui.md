# NetLab Android 0.6 — interface React offline

O aplicativo Android recebeu uma interface React com Motion, Radix Dialog e
ícones Lucide. Inclui temas claro/escuro, menu animado, navegação inferior,
ícone próprio, lista de aparelhos, guia de conexão e conversa em balões com
horários e confirmações. O serviço Java continua responsável pelo Bluetooth.

## Instalar a atualização

Envie `android/NetLab-Bluetooth-0.7.apk` para o celular e abra o arquivo.
A assinatura é a mesma dos APKs 0.4 e 0.5 fornecidos neste projeto; o código de versão
agora é 7. O Android oferece atualização da instalação existente.
Não é necessário desinstalar a versão anterior.

A versão 0.5.1 corrige o fechamento na abertura causado pelo empacotamento do DEX
na 0.5. O novo APK usa um ZIP montado do zero, sem combinar atualizações de arquivo
de bibliotecas diferentes. O registro reproduzido no emulador apontava
`Failed to extract 'classes.dex' ... Inconsistent information`.

No menu, escolha Claro/Escuro. Em Aparelhos, abra as configurações para parear
os dispositivos e atualize a lista. Um aparelho usa Receber conexão e o outro
Conectar. O servidor Windows 0.4 continua compatível para conexão manual.
Para QR Code, use Windows 0.6 e siga [qr-bluetooth.md](qr-bluetooth.md).

## Arquitetura e funcionamento offline

- `android/ui/src`: componentes React e estilos; `package-lock.json` fixa as dependências.
- `android/app/src/main/assets/ui`: HTML, JavaScript, CSS e avisos de licença compilados.
- `MainActivity`: WebView de conteúdo local e comandos limitados para o serviço.
- `ChatService`: conexão RFCOMM, fila, recibos e modelo de dados da conversa.
- O manifesto continua sem INTERNET. Não há CDN, fonte remota ou downloads em execução.
- A WebView só serve os arquivos internos permitidos. Navegação externa, acesso a
  arquivos/conteúdo arbitrários, frames e conexões HTTP externas ficam bloqueados.
- Mensagens recebidas são texto React, sem interpretação como HTML.
- Confirmação de exibição só é enviada com a conversa visível e menu fechado.
- Uma falha de envio preserva o texto digitado. Mensagens recebidas/enviadas aparecem
  na ordem em que foram adicionadas à conversa; horários são locais ao aparelho.

Android 8+ continua sendo a versão mínima do APK. A interface usa Android System
WebView e precisa de uma versão atualizada desse componente para compatibilidade
visual; o JavaScript é compilado para Chromium 80+. A atualização do componente,
quando necessária, é feita pelo sistema. Depois de instalado, o app usa os recursos
empacotados e não precisa de internet para desenhar a interface.

O histórico completo permanece temporário, como na versão anterior; apenas a fila
pendente sobrevive ao processo. Até 256 balões são apresentados nesta sessão.
As animações respeitam a preferência de movimento reduzido do WebView/navegador.

Referências: [React em um projeto existente](https://react.dev/learn/add-react-to-an-existing-project),
[conteúdo local no Android WebView](https://developer.android.com/develop/ui/views/layout/webapps/load-local-content),
[segurança da ponte nativa](https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges),
[Motion e movimento reduzido](https://motion.dev/docs/react-use-reduced-motion),
[Radix Dialog](https://www.radix-ui.com/primitives/docs/components/dialog).

## Desenvolvimento

Na pasta `android/ui`, com Node 22 e npm:

```powershell
npm ci
npm test
npm run build
npm run preview
```

A prévia em `http://localhost:8789` serve a mesma interface compilada. Ela é marcada
como prévia visual e não usa o rádio Bluetooth. “Ver conversa de exemplo” mostra
somente conteúdo ilustrativo; não simula entrega real nem está disponível no APK.

Após compilar a interface, rode `scripts/Build-Android.ps1` com o SDK, JDK, diretório
de build e nome do APK de saída. Use o mesmo diretório/chave da versão anterior para
manter a assinatura. Os recursos compilados já acompanham o código; o script direto
não baixa pacotes npm por conta própria. Licenças das bibliotecas acompanham o APK
em `assets/ui/THIRD_PARTY_NOTICES.txt`.

## Validação desta interface

Na 0.5.1, a abertura e o carregamento React passaram em emuladores Android 14 e 15,
inclusive instalando a atualização sobre a 0.5. O fechamento da versão anterior foi
reproduzido nesses mesmos ambientes. Consulte [validation.md](validation.md).

O teste nativo de regressão fica em `android/tests/StartupSmoke.java`. Instale o
APK de produção em um emulador e execute `scripts/Test-AndroidStartup.ps1` com
SdkRoot, JdkRoot, BuildDirectory, SigningKey (chave de desenvolvimento usada no
APK) e DeviceSerial. O teste instala um pacote de instrumentação separado e
confere abertura, React renderizado, ponte nativa e envio desabilitado sem conexão.
Não exige tornar o aplicativo de produção depurável.

- 7 testes automatizados: estado sem conexão, confirmação/falha de envio, preservação
  de texto novo, ordem dos balões/escape de HTML, seleção de aparelho/visibilidade,
  menu/tema/foco e sincronização após a WebView concluir o carregamento.
- Build de produção React e compilação/assinatura do APK concluídos.
- Layout inspecionado no navegador em 390×844, 320×568 e 360×460, com tema claro e
  menu escuro; sem rolagem horizontal. Altura reduzida mantém o campo de mensagem
  acima da navegação. Isso é uma verificação de layout, não um teste do teclado Android.
- Bluetooth, instalação/execução da WebView no Android, teclado real, TalkBack e
  suspensão em segundo plano ainda precisam de validação em aparelho físico.

O servidor Windows e o simulador mantêm a interface da versão 0.4. Esta entrega
moderniza especificamente o aplicativo Android.
