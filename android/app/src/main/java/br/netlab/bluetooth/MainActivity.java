package br.netlab.bluetooth;

import android.Manifest;
import android.app.*;
import android.bluetooth.*;
import android.content.*;
import android.content.pm.PackageManager;
import android.os.*;
import android.provider.Settings;
import android.view.*;
import android.widget.*;
import java.util.*;

public class MainActivity extends Activity {
    private ChatService service;
    private TextView status,log;
    private EditText message;
    private Spinner devices;
    private final ArrayList<BluetoothDevice> paired=new ArrayList<>();
    private boolean bound=false,visible=false;
    private final ServiceConnection connection=new ServiceConnection(){
        public void onServiceConnected(ComponentName name,IBinder binder){service=((ChatService.LocalBinder)binder).service();service.attach(MainActivity.this::render,visible);render();}
        public void onServiceDisconnected(ComponentName name){service=null;}
    };
    @Override public void onCreate(Bundle state){
        super.onCreate(state);
        LinearLayout page=new LinearLayout(this);page.setOrientation(LinearLayout.VERTICAL);page.setPadding(24,40,24,24);page.setBackgroundColor(0xfff5f6f2);
        page.setOnApplyWindowInsetsListener((view,insets)->{view.setPadding(24+insets.getSystemWindowInsetLeft(),24+insets.getSystemWindowInsetTop(),24+insets.getSystemWindowInsetRight(),24+insets.getSystemWindowInsetBottom());return insets;});
        setContentView(page);
        TextView title=new TextView(this);title.setText("NetLab Bluetooth");title.setTextSize(26);title.setTextColor(0xff176954);page.addView(title);
        TextView info=new TextView(this);info.setText("Texto direto entre aparelhos próximos. Sem Wi-Fi, dados móveis ou internet. Pareie os aparelhos nas configurações antes de conectar.");info.setPadding(0,16,0,16);page.addView(info);
        status=new TextView(this);status.setText("Pronto para conectar.");page.addView(status);
        Button settings=button("Parear / ativar Bluetooth",page);settings.setOnClickListener(v->startActivity(new Intent(Settings.ACTION_BLUETOOTH_SETTINGS)));
        devices=new Spinner(this);page.addView(devices);
        LinearLayout actions=new LinearLayout(this);page.addView(actions);
        button("Atualizar",actions).setOnClickListener(v->refresh());
        button("Conectar",actions).setOnClickListener(v->{if(!ready())return;if(paired.isEmpty()){status.setText("Pareie primeiro e toque em Atualizar.");return;}start("connect",paired.get(devices.getSelectedItemPosition()).getAddress());});
        button("Receber conexão",page).setOnClickListener(v->{if(ready())start("listen",null);});
        button("Desconectar e parar",page).setOnClickListener(v->{if(service!=null)service.stopChat();});
        ScrollView scroll=new ScrollView(this);log=new TextView(this);log.setTextSize(15);log.setPadding(12,16,12,16);scroll.addView(log);page.addView(scroll,new LinearLayout.LayoutParams(-1,0,1));
        LinearLayout composer=new LinearLayout(this);page.addView(composer);message=new EditText(this);message.setHint("Sua mensagem");message.setMaxLines(3);message.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(4000)});composer.addView(message,new LinearLayout.LayoutParams(0,-2,1));
        button("Enviar",composer).setOnClickListener(v->{if(service==null){status.setText("Conecte ou aguarde um aparelho primeiro.");return;}if(service.sendText(message.getText().toString()))message.setText("");});
        refresh();
    }
    private Button button(String text,LinearLayout parent){Button b=new Button(this);b.setText(text);b.setAllCaps(false);parent.addView(b);return b;}
    private boolean ready(){
        if(Build.VERSION.SDK_INT>=31&&checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT)!=PackageManager.PERMISSION_GRANTED){requestPermissions(new String[]{Manifest.permission.BLUETOOTH_CONNECT},1);return false;}
        BluetoothManager manager=getSystemService(BluetoothManager.class);BluetoothAdapter adapter=manager==null?null:manager.getAdapter();
        if(adapter==null){status.setText("Este aparelho não oferece Bluetooth clássico.");return false;}
        if(!adapter.isEnabled()){status.setText("Ative o Bluetooth nas configurações.");return false;}return true;
    }
    private void refresh(){if(!ready())return;try{paired.clear();paired.addAll(getSystemService(BluetoothManager.class).getAdapter().getBondedDevices());paired.sort(Comparator.comparing(BluetoothDevice::getAddress));ArrayList<String> names=new ArrayList<>();for(BluetoothDevice d:paired)names.add(d.getName()+" · "+d.getAddress());devices.setAdapter(new ArrayAdapter<>(this,android.R.layout.simple_spinner_dropdown_item,names));}catch(SecurityException e){status.setText("Permita acesso aos dispositivos por perto.");}}
    private void start(String action,String address){
        if(Build.VERSION.SDK_INT>=33&&checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)!=PackageManager.PERMISSION_GRANTED)requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS},2);
        Intent intent=new Intent(this,ChatService.class).setAction(action);if(address!=null)intent.putExtra("address",address);startForegroundService(intent);
        if(!bound)bound=bindService(new Intent(this,ChatService.class),connection,BIND_AUTO_CREATE);
    }
    private void render(){if(service==null)return;runOnUiThread(()->{status.setText(service.status());log.setText(service.history());});}
    @Override public void onRequestPermissionsResult(int r,String[] p,int[] g){super.onRequestPermissionsResult(r,p,g);if(r==1){if(g.length>0&&g[0]==PackageManager.PERMISSION_GRANTED)refresh();else status.setText("Sem a permissão de Bluetooth não é possível conectar.");}}
    @Override public void onStart(){super.onStart();visible=true;bound=bindService(new Intent(this,ChatService.class),connection,0);if(service!=null)service.attach(this::render,true);}
    @Override public void onStop(){visible=false;if(service!=null)service.attach(null,false);if(bound){unbindService(connection);bound=false;}service=null;super.onStop();}
}
