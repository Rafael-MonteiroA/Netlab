package br.netlab.bluetooth;

import android.Manifest;
import android.app.*;
import android.bluetooth.*;
import android.content.*;
import android.content.pm.PackageManager;
import android.graphics.Color;
import android.net.Uri;
import android.os.*;
import android.provider.Settings;
import android.view.*;
import android.webkit.*;
import android.widget.FrameLayout;
import org.json.*;
import java.io.*;
import java.util.*;

/** Only bundled assets can load in this WebView. Bluetooth stays in ChatService. */
public class MainActivity extends Activity {
    private static final String ORIGIN="https://appassets.androidplatform.net";
    private WebView web;
    private FrameLayout frame;
    private ChatService service;
    private boolean bound=false,foreground=false,pageReady=false,chatVisible=false;
    private String notice="";
    private JSONObject sendResult;
    private boolean scanAfterPermission;
    private String report="";
    private final ServiceConnection connection=new ServiceConnection(){
        public void onServiceConnected(ComponentName name,IBinder binder){service=((ChatService.LocalBinder)binder).service();attach();}
        public void onServiceDisconnected(ComponentName name){service=null;render();}
    };
    @Override public void onCreate(Bundle saved){
        super.onCreate(saved);
        applyScreenPrivacy(getSharedPreferences("netlab",MODE_PRIVATE).getBoolean("screenPrivacy",false));
        getWindow().setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE);
        frame=new FrameLayout(this);frame.setBackgroundColor(Color.rgb(244,246,240));
        web=new WebView(this);web.setBackgroundColor(Color.TRANSPARENT);
        frame.addView(web,new FrameLayout.LayoutParams(-1,-1));
        frame.setOnApplyWindowInsetsListener((v,insets)->{
            if(Build.VERSION.SDK_INT>=30){android.graphics.Insets safe=insets.getInsets(WindowInsets.Type.systemBars()|WindowInsets.Type.ime());v.setPadding(safe.left,safe.top,safe.right,safe.bottom);}
            else v.setPadding(insets.getSystemWindowInsetLeft(),insets.getSystemWindowInsetTop(),insets.getSystemWindowInsetRight(),insets.getSystemWindowInsetBottom());
            return insets;
        });
        setContentView(frame);applyTheme(false);
        WebSettings settings=web.getSettings();settings.setJavaScriptEnabled(true);settings.setDomStorageEnabled(true);settings.setTextZoom(Math.round(getResources().getConfiguration().fontScale*100));
        settings.setAllowFileAccess(false);settings.setAllowContentAccess(false);settings.setBlockNetworkLoads(true);
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);settings.setSupportMultipleWindows(false);
        web.addJavascriptInterface(new NativeBridge(),"NetLabNative");
        web.setWebViewClient(new WebViewClient(){
            @Override public boolean shouldOverrideUrlLoading(WebView view,WebResourceRequest req){return true;}
            @Override public WebResourceResponse shouldInterceptRequest(WebView view,WebResourceRequest req){
                Uri url=req.getUrl();String path=url.getPath();
                if(!"https".equals(url.getScheme())||!"appassets.androidplatform.net".equals(url.getHost())||path==null)return denied();
                String file=path.equals("/index.html")?"index.html":path.equals("/app.js")?"app.js":path.equals("/style.css")?"style.css":null;
                if(file==null)return denied();
                try{String mime=file.endsWith(".js")?"text/javascript":file.endsWith(".css")?"text/css":"text/html";
                    WebResourceResponse response=new WebResourceResponse(mime,"UTF-8",getAssets().open("ui/"+file));
                    Map<String,String> headers=new HashMap<>();headers.put("Cache-Control","no-store");headers.put("X-Content-Type-Options","nosniff");response.setResponseHeaders(headers);return response;
                }catch(IOException e){return denied();}
            }
            @Override public void onPageFinished(WebView view,String url){pageReady=(ORIGIN+"/index.html").equals(url);if(pageReady)web.evaluateJavascript("window.dispatchEvent(new Event('netlab:ready'));",null);render();}
        });
        web.loadUrl(ORIGIN+"/index.html");
    }
    private static WebResourceResponse denied(){return new WebResourceResponse("text/plain","UTF-8",403,"Forbidden",Collections.emptyMap(),new ByteArrayInputStream(new byte[0]));}
    private BluetoothAdapter adapter(){BluetoothManager manager=getSystemService(BluetoothManager.class);return manager==null?null:manager.getAdapter();}
    private boolean hasPermission(){return Build.VERSION.SDK_INT<31||checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT)==PackageManager.PERMISSION_GRANTED;}
    private boolean ready(){
        if(!hasPermission()){requestPermissions(new String[]{Manifest.permission.BLUETOOTH_CONNECT},1);return false;}
        if(adapter()==null){notice="Este aparelho não oferece Bluetooth clássico.";render();return false;}
        if(!adapter().isEnabled()){notice="Ative o Bluetooth nas configurações para continuar.";render();return false;}return true;
    }
    private void attach(){if(service!=null)service.attach(this::render,foreground&&chatVisible);else render();}
    private void scanQr(){
        if(!hasPermission()){scanAfterPermission=true;ready();return;}
        if(!ready())return;
        if(checkSelfPermission(Manifest.permission.CAMERA)!=PackageManager.PERMISSION_GRANTED){requestPermissions(new String[]{Manifest.permission.CAMERA},3);return;}
        startActivityForResult(new Intent(this,QrScannerActivity.class),20);
    }
    @Override protected void onActivityResult(int request,int result,Intent data){
        super.onActivityResult(request,result,data);
        if(request==21&&result==RESULT_OK&&data!=null&&data.getData()!=null){try(OutputStream output=getContentResolver().openOutputStream(data.getData())){if(service!=null)service.exportHistory(output);notice="Conversa exportada. O arquivo contém textos pessoais sem criptografia.";}catch(Exception e){notice="Não foi possível exportar a conversa.";}render();return;}
        if(request!=20||result!=RESULT_OK||data==null)return;
        try{
            String address=BluetoothInvite.parse(data.getStringExtra("qr"));
            if(!ready())return;
            BluetoothDevice pc=null;for(BluetoothDevice device:adapter().getBondedDevices())if(device.getAddress().equals(address))pc=device;
            if(pc==null){notice="Pareie este PC com o celular nas configurações Bluetooth e escaneie novamente.";render();return;}
            final String target=address;String name=pc.getName()==null?"PC pareado":pc.getName();
            new AlertDialog.Builder(this).setTitle("Conectar a "+name+"?")
                .setMessage("O NetLab ficará aguardando este PC por Bluetooth. Mantenha o QR Code aberto no computador.")
                .setNegativeButton("Cancelar",null).setPositiveButton("Conectar",(dialog,which)->{start("listen",target);notice="QR reconhecido. Aguardando a conexão do PC por Bluetooth.";render();}).show();
        }catch(IllegalArgumentException|SecurityException e){notice=e.getMessage();render();}
    }
    private void start(String action,String address){
        if(!ready())return;
        if(service!=null&&address!=null&&service.blocked(address)){notice="Este aparelho está bloqueado no NetLab. Libere-o em Aparelhos.";render();return;}
        if("connect".equals(action)){
            boolean paired=false;for(BluetoothDevice d:adapter().getBondedDevices())if(d.getAddress().equals(address))paired=true;
            if(!paired){notice="Pareie esse aparelho nas configurações e atualize a lista.";render();return;}
        }
        if(Build.VERSION.SDK_INT>=33&&checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)!=PackageManager.PERMISSION_GRANTED)requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS},2);
        Intent intent=new Intent(this,ChatService.class).setAction(action);if(address!=null)intent.putExtra("address",address);
        startForegroundService(intent);if(!bound)bound=bindService(new Intent(this,ChatService.class),connection,BIND_AUTO_CREATE);notice="";render();
    }
    private void render(){runOnUiThread(()->{
        if(!pageReady||isDestroyed())return;
        try{
            JSONObject state=service==null?new JSONObject():service.snapshot();JSONArray devices=new JSONArray();String bluetooth="permission";
            if(hasPermission()){
                BluetoothAdapter radio=adapter();bluetooth=radio==null?"unsupported":radio.isEnabled()?"enabled":"disabled";
                if(radio!=null&&radio.isEnabled()){
                    ArrayList<BluetoothDevice> list=new ArrayList<>(radio.getBondedDevices());list.sort(Comparator.comparing(BluetoothDevice::getAddress));
                    for(BluetoothDevice d:list){JSONObject item=new JSONObject();item.put("address",d.getAddress());item.put("name",d.getName()==null?"Aparelho Bluetooth":d.getName());item.put("blocked",service!=null&&service.blocked(d.getAddress()));devices.put(item);}
                }
            }
            state.put("devices",devices);state.put("bluetooth",bluetooth);state.put("deviceName",Build.MODEL);state.put("notice",notice);state.put("sendResult",sendResult==null?JSONObject.NULL:sendResult);
            state.put("report",report).put("screenPrivacy",getSharedPreferences("netlab",MODE_PRIVATE).getBoolean("screenPrivacy",false));
            String data=JSONObject.quote(state.toString()).replace("\u2028","\\u2028").replace("\u2029","\\u2029");
            web.evaluateJavascript("window.dispatchEvent(new CustomEvent('netlab:state',{detail:JSON.parse("+data+")}));",null);
        }catch(JSONException|SecurityException e){notice="Não foi possível atualizar o Bluetooth. Confira a permissão em Dispositivos por perto.";}
    });}
    public final class NativeBridge {
        @JavascriptInterface public void postMessage(String raw){
            if(raw==null||raw.length()>30000)return;
            runOnUiThread(()->{if(!pageReady)return;try{
                JSONObject value=new JSONObject(raw);String type=value.optString("type");
                switch(type){
                    case "draft":if(service!=null)service.draft(value.optString("address"),value.optString("text"));break;
                    case "history":if(service!=null)service.selectHistory(value.optString("address"));break;
                    case "historyPage":if(service!=null)service.historyPage(value.optBoolean("older"));break;
                    case "clearHistory":new AlertDialog.Builder(MainActivity.this).setTitle("Apagar conversa neste aparelho?").setMessage("O histórico, o rascunho e a fila de envio serão apagados. A cópia do outro aparelho permanece.").setNegativeButton("Cancelar",null).setPositiveButton("Apagar",(d,w)->{if(service!=null)service.clearHistory(false);render();}).show();break;
                    case "clearAllHistory":new AlertDialog.Builder(MainActivity.this).setTitle("Apagar todas as conversas locais?").setMessage("Todas as conversas, rascunhos e mensagens pendentes serão removidos deste aparelho. Esta ação não pode ser desfeita.").setNegativeButton("Cancelar",null).setPositiveButton("Apagar tudo",(d,w)->{if(service!=null)service.clearHistory(true);render();}).show();break;
                    case "exportHistory":new AlertDialog.Builder(MainActivity.this).setTitle("Exportar os textos da conversa?").setMessage("O arquivo exportado não será criptografado. Salve em um lugar de confiança.").setNegativeButton("Cancelar",null).setPositiveButton("Escolher arquivo",(d,w)->startActivityForResult(new Intent(Intent.ACTION_CREATE_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE).setType("text/plain").putExtra(Intent.EXTRA_TITLE,"NetLab-conversa.txt"),21)).show();break;
                    case "block":if(service!=null)service.block(value.optString("address"),value.optBoolean("value"));render();break;
                    case "readReceipts":if(service!=null)service.readReceipts(value.optBoolean("value"));render();break;
                    case "screenPrivacy":getSharedPreferences("netlab",MODE_PRIVATE).edit().putBoolean("screenPrivacy",value.optBoolean("value")).commit();applyScreenPrivacy(value.optBoolean("value"));render();break;
                    case "diagnostics":report=diagnostics();render();break;
                    case "shareReport":{String text=value.optString("text");if(text.length()>12000)return;startActivity(Intent.createChooser(new Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT,text),"Compartilhar relatório revisado"));break;}
                    case "copyReport":{String text=value.optString("text");if(text.length()>12000)return;((ClipboardManager)getSystemService(CLIPBOARD_SERVICE)).setPrimaryClip(ClipData.newPlainText("Diagnóstico NetLab",text));notice="Relatório copiado. Nenhum envio automático foi feito.";render();break;}
                    case "scanQr":scanQr();break;
                    case "refresh":notice="";render();break;
                    case "minimize":moveTaskToBack(true);break;
                    case "permission":ready();render();break;
                    case "settings":startActivity(new Intent(Settings.ACTION_BLUETOOTH_SETTINGS));break;
                    case "connect":start("connect",value.optString("address"));break;
                    case "listen":start("listen",null);break;
                    case "stop":if(service!=null)service.stopChat();render();break;
                    case "visible":chatVisible=value.optBoolean("value");attach();break;
                    case "dismiss":notice="";render();break;
                    case "theme":applyTheme("dark".equals(value.optString("value")));break;
                    case "send":{
                        String text=value.optString("text"),request=value.optString("requestId");if(request.length()>80)return;
                        boolean ok=service!=null&&service.sendText(text);
                        if(!ok)notice=service==null?"Conecte a um aparelho antes de enviar.":service.status();
                        sendResult=new JSONObject().put("requestId",request).put("ok",ok);render();break;
                    }
                    default:break;
                }
            }catch(Exception e){notice="Não foi possível concluir. Seu rascunho foi mantido; confira espaço livre, permissões e conexão.";render();}});
        }
    }
    private void applyTheme(boolean dark){
        int color=dark?Color.rgb(20,30,27):Color.rgb(244,246,240);frame.setBackgroundColor(color);
        getWindow().setStatusBarColor(color);getWindow().setNavigationBarColor(color);
        getWindow().getDecorView().setSystemUiVisibility(dark?0:View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR|View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR);
    }
    private void applyScreenPrivacy(boolean enabled){if(enabled)getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);else getWindow().clearFlags(WindowManager.LayoutParams.FLAG_SECURE);}
    private String diagnostics()throws JSONException{
        JSONObject result=new JSONObject().put("app","NetLab").put("version","0.7.0").put("androidApi",Build.VERSION.SDK_INT)
            .put("bluetoothPermission",hasPermission()).put("cameraPermission",checkSelfPermission(Manifest.permission.CAMERA)==PackageManager.PERMISSION_GRANTED)
            .put("storage","Android Keystore / AES-GCM").put("includesMessages",false).put("includesAddresses",false);
        if(service!=null){JSONObject state=service.snapshot();result.put("phase",state.optString("phase")).put("pendingCount",state.optInt("pending"))
            .put("storageHealthy",state.optString("storageError").isEmpty());}
        return result.toString(2);
    }
    @Override public void onRequestPermissionsResult(int r,String[] p,int[] g){super.onRequestPermissionsResult(r,p,g);boolean granted=g.length>0&&g[0]==PackageManager.PERMISSION_GRANTED;
        if(r==1){if(!granted)notice="Permita Dispositivos por perto para conectar. Você pode alterar isso nas configurações do Android.";boolean resume=scanAfterPermission;scanAfterPermission=false;if(granted&&resume)scanQr();}
        if(r==3){if(granted)scanQr();else notice="Permita a câmera para ler o QR Code. A conexão manual continua disponível em Aparelhos.";}render();}
    @Override public void onStart(){super.onStart();bound=bindService(new Intent(this,ChatService.class),connection,BIND_AUTO_CREATE);}
    @Override public void onResume(){super.onResume();foreground=true;attach();render();}
    @Override public void onPause(){foreground=false;attach();super.onPause();}
    @Override public void onStop(){if(service!=null)service.attach(null,false);if(bound){unbindService(connection);bound=false;}service=null;super.onStop();}
    @Override public void onBackPressed(){web.evaluateJavascript("window.dispatchEvent(new Event('netlab:back'));",null);}
    @Override public void onDestroy(){web.removeJavascriptInterface("NetLabNative");web.destroy();super.onDestroy();}
}
