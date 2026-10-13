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
    private final ServiceConnection connection=new ServiceConnection(){
        public void onServiceConnected(ComponentName name,IBinder binder){service=((ChatService.LocalBinder)binder).service();attach();}
        public void onServiceDisconnected(ComponentName name){service=null;render();}
    };
    @Override public void onCreate(Bundle saved){
        super.onCreate(saved);
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
        WebSettings settings=web.getSettings();settings.setJavaScriptEnabled(true);settings.setDomStorageEnabled(true);
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
    private void start(String action,String address){
        if(!ready())return;
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
                    for(BluetoothDevice d:list){JSONObject item=new JSONObject();item.put("address",d.getAddress());item.put("name",d.getName()==null?"Aparelho Bluetooth":d.getName());devices.put(item);}
                }
            }
            state.put("devices",devices);state.put("bluetooth",bluetooth);state.put("deviceName",Build.MODEL);state.put("notice",notice);state.put("sendResult",sendResult==null?JSONObject.NULL:sendResult);
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
            }catch(Exception e){notice="Não foi possível concluir. Confira o Bluetooth e tente novamente.";render();}});
        }
    }
    private void applyTheme(boolean dark){
        int color=dark?Color.rgb(20,30,27):Color.rgb(244,246,240);frame.setBackgroundColor(color);
        getWindow().setStatusBarColor(color);getWindow().setNavigationBarColor(color);
        getWindow().getDecorView().setSystemUiVisibility(dark?0:View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR|View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR);
    }
    @Override public void onRequestPermissionsResult(int r,String[] p,int[] g){super.onRequestPermissionsResult(r,p,g);if(r==1&&!(g.length>0&&g[0]==PackageManager.PERMISSION_GRANTED))notice="Permita Dispositivos por perto para conectar. Você pode alterar isso nas configurações do Android.";render();}
    @Override public void onStart(){super.onStart();bound=bindService(new Intent(this,ChatService.class),connection,0);}
    @Override public void onResume(){super.onResume();foreground=true;attach();render();}
    @Override public void onPause(){foreground=false;attach();super.onPause();}
    @Override public void onStop(){if(service!=null)service.attach(null,false);if(bound){unbindService(connection);bound=false;}service=null;super.onStop();}
    @Override public void onBackPressed(){web.evaluateJavascript("window.dispatchEvent(new Event('netlab:back'));",null);}
    @Override public void onDestroy(){web.removeJavascriptInterface("NetLabNative");web.destroy();super.onDestroy();}
}
