# NetLab — mensagens sem internet e sem Wi-Fi

O aplicativo usa Bluetooth clássico (RFCOMM) entre aparelhos próximos. Não exige
roteador, chip, dados móveis, conta nem servidor na internet. O APK Android não
declara a permissão INTERNET. O computador só participa da conversa Android ↔ Windows.
Android ↔ Android funciona sem computador. Esta versão conecta **um par por vez**;
grupos de até 16 clientes continuam disponíveis no modo de rede IP local.

Veja também o [guia da interface Android 0.5](android-ui.md).

## Instalar

O pacote inclui `android/NetLab-Bluetooth-0.5.1.apk`. Transfira-o por cabo USB ou outro
meio e abra o arquivo no Android 8 ou mais recente. Autorize a instalação desse
arquivo pelo gerenciador de arquivos, se solicitado. Trata-se de um APK de
desenvolvimento assinado localmente, fora da Play Store. Não há compra ou assinatura.

Ao abrir, permita **Dispositivos por perto / Bluetooth**. A notificação permite
identificar o serviço em execução; o aplicativo pede essa permissão quando necessário.
Pareie somente os aparelhos com os quais pretende conversar, conferindo o código
mostrado nas configurações Bluetooth dos dois lados.

## Android ↔ Android

1. Instale o mesmo APK nos dois celulares e ative o Bluetooth.
2. Pareie os celulares pelas configurações do Android.
3. No primeiro, abra NetLab Bluetooth e toque em **Receber conexão**.
4. No segundo, toque em **Atualizar**, selecione o primeiro e toque em **Conectar**.
5. Espere o estado **Conectado** e envie texto nos dois sentidos.
6. Pode desligar Wi-Fi e dados móveis. Mantenha o Bluetooth ativo.

Não toque em Conectar nos dois lados: um recebe e o outro inicia. O telefone que
recebe volta a esperar após uma queda; quem inicia tenta reconectar automaticamente.

## Android ↔ Windows

1. No Windows, extraia o pacote e abra `Iniciar-NetLab.cmd`.
2. Pareie PC e celular nas configurações Bluetooth do Windows/Android.
3. No Android, abra o aplicativo e toque em **Receber conexão**.
4. No próprio PC, abra `http://localhost:8787`, aba **Bluetooth sem Wi-Fi**.
5. Clique em **Atualizar aparelhos**, selecione o celular e **Conectar por Bluetooth**.
6. Envie texto em ambos os lados. Wi-Fi e dados móveis podem ficar desligados.

O navegador conversa com o servidor no próprio computador; o servidor usa o rádio
Bluetooth para alcançar o celular. Essa conexão local não depende de Wi-Fi ou
internet. Mantenha o servidor e a aba abertos. O PC precisa de um adaptador Bluetooth
clássico com driver compatível. Se o computador não tiver esse hardware, o software
não consegue substituí-lo. Nenhum hardware foi comprado nesta entrega.

## Entrega e reconexão

- **Na fila:** ainda falta confirmação do outro aplicativo.
- **Recebida:** o aplicativo destinatário confirmou o processamento.
- **Exibida:** a conversa estava visível. Não comprova que uma pessoa leu.
- Até 50 mensagens pendentes por conversa; reenvio na reconexão e a cada 15 segundos.
- IDs de instalação e sequência evitam repetir a exibição de mensagens reenviadas.
- No Android, a fila pendente fica no armazenamento privado do aplicativo; no PC,
  fica na sessão da aba. O histórico completo não é um arquivo permanente.
- **Desconectar** interrompe a conexão, mas conserva a fila para o mesmo aparelho.
  Para apagar os dados Android, use Limpar armazenamento nas configurações do app;
  no navegador, remova os dados do site. Isso remove também filas e identificadores.

O serviço Android exibe uma notificação enquanto está ativo. Economia de bateria,
suspensão do PC e restrições do fabricante podem interromper a conexão. Uso prolongado
com tela apagada ainda precisa de teste nos aparelhos reais. Para encerrar, toque em
**Desconectar e parar** e feche a conversa no PC.

## Alcance, segurança e limites

Os aparelhos precisam estar no alcance do Bluetooth; distância e obstáculos variam
conforme o rádio. Não há comunicação a quilômetros de distância, malha, retransmissão
entre terceiros, iPhone, envio de arquivos ou grupos Bluetooth nesta versão.

As conexões usam RFCOMM seguro/pareado no Android e exigem autenticação e criptografia
Bluetooth no Windows. Não foi implementada uma camada própria de criptografia ponta
a ponta com verificação de identidade. As mensagens e filas locais são texto no
armazenamento da aplicação. Somente o navegador no próprio PC, pela origem localhost,
pode acessar a ponte Bluetooth; outro cliente da LAN não pode controlar seu adaptador.

Referências de implementação: [conexão Bluetooth no Android](https://developer.android.com/develop/connectivity/bluetooth/connect-bluetooth-devices),
[permissões Android](https://developer.android.com/develop/connectivity/bluetooth/bt-permissions),
[conexão RFCOMM no Windows](https://learn.microsoft.com/en-us/windows/win32/bluetooth/bluetooth-and-connect)
e [opções de segurança do socket](https://learn.microsoft.com/en-us/windows/win32/bluetooth/bluetooth-and-socket-options).

## Validação física pendente

O APK foi compilado e sua assinatura e permissões foram verificadas. A ponte Windows
foi compilada. Não havia Android conectado por USB nem adaptador Bluetooth detectado
no computador de desenvolvimento. Portanto, **não houve validação por rádio**.

Execute este roteiro e registre modelo dos aparelhos, Android/Windows e resultado:

| Verificação | Resultado nesta entrega |
| --- | --- |
| Android A recebe texto de B e responde | Pendente — exige dois celulares |
| Android recebe do Windows e responde | Pendente — exige PC com Bluetooth e celular |
| Texto com Wi-Fi e dados móveis desligados | Pendente |
| Acentos, emoji e mensagem longa | Pendente em hardware |
| Desligar Bluetooth, enviar uma mensagem na fila e religar | Pendente |
| Confirmar que cada mensagem aparece uma vez após reconectar | Pendente |
| Manter tela apagada por 10 minutos e retomar | Pendente |
| Encerrar serviço e conferir que a notificação desaparece | Pendente |

## Compilar gratuitamente

O APK fornecido já está pronto para instalar. Para recompilar: JDK 17 e Android SDK
com `platforms;android-35` e `build-tools;35.0.0`, obtidos nos sites oficiais. Não é
necessário Android Studio, conta de loja ou hospedagem. As licenças do SDK são aceitas
pelo desenvolvedor durante a instalação.

```powershell
.\scripts\Build-Android.ps1 -SdkRoot C:\Android\Sdk -JdkRoot 'C:\Program Files\Java\jdk-17' -BuildDirectory C:\NetLabBuild -OutputApk C:\NetLabBuild\NetLab.apk
```

O script usa javac, D8, aapt2, zipalign e apksigner; gera uma chave de desenvolvimento
no diretório de build. Preserve a chave para atualizar a mesma instalação. Ela não
faz parte do ZIP. Para distribuir oficialmente, use sua própria chave de publicação.
Também há projeto Gradle/AGP para desenvolvimento; o build validado nesta entrega foi
o script direto, não o Gradle. Download oficial: [Android SDK](https://developer.android.com/studio).
