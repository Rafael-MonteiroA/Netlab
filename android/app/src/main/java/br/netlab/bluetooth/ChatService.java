package br.netlab.bluetooth;

import android.app.*;
import android.bluetooth.*;
import android.content.*;
import android.content.pm.ServiceInfo;
import android.os.*;
import org.json.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.concurrent.*;

public class ChatService extends Service {
    public static final UUID UUID_SERVICE=UUID.fromString("08d4a386-30a7-4bd7-aeca-958c6432f208");
    public class LocalBinder extends Binder { ChatService service(){return ChatService.this;} }
    private final IBinder binder=new LocalBinder();
    private final Handler ui=new Handler(Looper.getMainLooper());
    private final ScheduledExecutorService worker=Executors.newScheduledThreadPool(3);
    private final ExecutorService writer=Executors.newSingleThreadExecutor();
    private final ArrayList<String> lines=new ArrayList<>();
    private final LinkedHashMap<String,JSONObject> pending=new LinkedHashMap<>();
    private final LinkedHashMap<String,String> incoming=new LinkedHashMap<>();
    private final HashMap<String,String> delivery=new HashMap<>();
    private final LinkedHashMap<String,String> outgoingText=new LinkedHashMap<>();
    private volatile BluetoothSocket socket;
    private volatile BluetoothServerSocket listener;
    private volatile int generation=0;
    private volatile boolean running=false,visible=false,transportReady=false;
    private volatile String peer="",target="",mode="",state="Desconectado",peerName="Outro aparelho";
    private volatile long lastSeen=0;
    private Runnable observer;
    private SharedPreferences prefs;
    private String installation;
    @Override public void onCreate(){super.onCreate();prefs=getSharedPreferences("netlab",MODE_PRIVATE);installation=prefs.getString("installation",null);if(installation==null){installation=UUID.randomUUID().toString().replace("-","");prefs.edit().putString("installation",installation).apply();}worker.scheduleWithFixedDelay(this::heartbeat,15,15,TimeUnit.SECONDS);}
    @Override public IBinder onBind(Intent i){return binder;}
    @Override public int onStartCommand(Intent intent,int flags,int startId){
        NotificationManager nm=getSystemService(NotificationManager.class);nm.createNotificationChannel(new NotificationChannel("bluetooth","Conexão Bluetooth",NotificationManager.IMPORTANCE_LOW));
        PendingIntent open=PendingIntent.getActivity(this,0,new Intent(this,MainActivity.class),PendingIntent.FLAG_IMMUTABLE);
        Notification n=new Notification.Builder(this,"bluetooth").setContentTitle("NetLab Bluetooth ativo").setContentText("Mensagens sem internet. Abra para conversar ou parar.").setSmallIcon(android.R.drawable.stat_sys_data_bluetooth).setContentIntent(open).setOngoing(true).build();
        if(Build.VERSION.SDK_INT>=29)startForeground(1,n,ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE);else startForeground(1,n);
        if(intent!=null&&intent.getAction()!=null){closeTransport();running=true;mode=intent.getAction();target=intent.getStringExtra("address");final int g=generation;
            if("connect".equals(mode)&&target!=null){loadPending(target);worker.execute(()->connectLoop(g));}else{mode="listen";worker.execute(()->listenLoop(g));}}
        return START_NOT_STICKY;
    }
    public synchronized void attach(Runnable callback,boolean shown){observer=callback;visible=shown;if(shown){for(String id:new ArrayList<>(incoming.keySet())){if(incoming.get(id).equals("received")){incoming.put(id,"read");writeAsync(frame("ack","id",id,"status","read"));}}}notifyUi();}
    public String status(){return state;}
    public synchronized String history(){StringBuilder b=new StringBuilder();for(String s:lines)b.append(s).append("\n\n");for(String id:outgoingText.keySet())b.append("Você: ").append(outgoingText.get(id)).append("\n").append(delivery.getOrDefault(id,"Na fila")).append("\n\n");return b.toString();}
    private void status(String s){state=s;notifyUi();}
    private synchronized void notifyUi(){Runnable r=observer;if(r!=null)ui.post(r);}
    private synchronized void log(String s){lines.add(s);while(lines.size()>150)lines.remove(0);notifyUi();}
    private BluetoothAdapter adapter(){return getSystemService(BluetoothManager.class).getAdapter();}
    private void connectLoop(int g){int attempt=0;while(running&&generation==g){try{status("Conectando por Bluetooth…");BluetoothSocket candidate=adapter().getRemoteDevice(target).createRfcommSocketToServiceRecord(UUID_SERVICE);transportReady=false;socket=candidate;
            ScheduledFuture<?> timeout=worker.schedule(()->close(candidate),20,TimeUnit.SECONDS);try{candidate.connect();}finally{timeout.cancel(false);}if(generation!=g){close(candidate);return;}attempt=0;communicate(candidate,g);
        }catch(Exception e){if(generation==g)status("Bluetooth desconectado. Tentando novamente: "+e.getMessage());}finally{if(generation==g){close(socket);socket=null;}}
        if(running&&generation==g){try{Thread.sleep(Math.min(30000,1000L<<Math.min(attempt++,5)));}catch(InterruptedException e){return;}}
    }}
    private void listenLoop(int g){while(running&&generation==g){try{
        status("Aguardando Bluetooth. No outro aparelho, escolha Conectar.");BluetoothServerSocket server=adapter().listenUsingRfcommWithServiceRecord("NetLab",UUID_SERVICE);listener=server;
        BluetoothSocket accepted=server.accept();server.close();if(generation!=g){close(accepted);return;}transportReady=false;socket=accepted;communicate(accepted,g);
      }catch(Exception e){if(generation==g)status("Não foi possível receber: "+e.getMessage());try{Thread.sleep(1500);}catch(InterruptedException ignored){return;}}
      finally{if(generation==g){close(socket);socket=null;}}
    }}
    private void communicate(BluetoothSocket current,int g)throws Exception{
        synchronized(this){loadPending(current.getRemoteDevice().getAddress());peerName=current.getRemoteDevice().getName();if(peerName==null)peerName="Outro aparelho";lastSeen=System.currentTimeMillis();transportReady=true;status("Conectado a "+peerName+" · Bluetooth · sem internet");write(frame("hello","name",modelName()));flush();}
        InputStream input=current.getInputStream();ByteArrayOutputStream line=new ByteArrayOutputStream();int value;
        while(running&&generation==g&&(value=input.read())!=-1){if(value=='\n'){String text=line.toString("UTF-8");line.reset();lastSeen=System.currentTimeMillis();handle(new JSONObject(text));}else{if(line.size()>=16384)throw new IOException("Quadro muito grande");line.write(value);}}
    }
    private synchronized void handle(JSONObject value)throws Exception{
        String type=value.optString("type"),id=value.optString("id");
        if(type.equals("hello")){peerName=value.optString("name","Outro aparelho");return;}
        if(type.equals("ping")){write(frame("pong","id",id));return;}if(type.equals("pong"))return;
        if(id.isEmpty()||id.length()>64||!id.matches("[A-Za-z0-9-]+"))throw new IOException("Identificador inválido");
        if(type.equals("ack")){String level=value.optString("status");if(!level.equals("read")&&!level.equals("received"))throw new IOException("Confirmação inválida");if(outgoingText.containsKey(id)){if(level.equals("read"))delivery.put(id,"Exibida no outro aparelho");else if(level.equals("received")&&!"Exibida no outro aparelho".equals(delivery.get(id)))delivery.put(id,"Recebida");}pending.remove(id);persistPending();notifyUi();return;}
        if(!type.equals("chat"))throw new IOException("Tipo de mensagem inválido");String text=value.optString("text");if(text.trim().isEmpty()||text.codePointCount(0,text.length())>4000)throw new IOException("Texto inválido");
        // Per-installation monotonic IDs survive reconnects and app process restarts.
        int split=id.lastIndexOf('-');if(split<1)throw new IOException("Formato do identificador inválido");long seq=Long.parseLong(id.substring(split+1));if(seq<=0||!id.substring(0,split).matches("[a-f0-9]{32}"))throw new IOException("Sequência inválida");String key="seen:"+peer+":"+id.substring(0,split);long seen=prefs.getLong(key,0);
        if(seq>seen){prefs.edit().putLong(key,seq).commit();log(peerName+": "+text);incoming.put(id,visible?"read":"received");while(incoming.size()>256)incoming.remove(incoming.keySet().iterator().next());}
        write(frame("ack","id",id,"status",incoming.getOrDefault(id,"received")));
    }
    public synchronized boolean sendText(String text){
        if(text.trim().isEmpty())return false;if(text.codePointCount(0,text.length())>4000){status("Máximo de 4000 caracteres.");return false;}
        if(peer.isEmpty()&&target==null){status("Aguarde a primeira conexão antes de enviar.");return false;}if(pending.size()>=50){status("Aguarde o envio das 50 mensagens pendentes.");return false;}
        long seq=prefs.getLong("sequence",0)+1;prefs.edit().putLong("sequence",seq).commit();String id=installation+"-"+seq;JSONObject value=frame("chat","id",id,"name",modelName(),"text",text.trim());pending.put(id,value);outgoingText.put(id,text.trim());delivery.put(id,"Na fila / aguardando confirmação");while(outgoingText.size()>100){String first=outgoingText.keySet().iterator().next();outgoingText.remove(first);delivery.remove(first);}persistPending();flush();notifyUi();return true;
    }
    private synchronized void persistPending(){String address=peer.isEmpty()?target:peer;if(address==null)return;JSONArray array=new JSONArray();for(JSONObject v:pending.values())array.put(v);prefs.edit().putString("outbox:"+address,array.toString()).commit();}
    private synchronized void loadPending(String address){if(address.equals(peer))return;if(!peer.isEmpty())persistPending();pending.clear();outgoingText.clear();delivery.clear();incoming.clear();lines.clear();try{JSONArray array=new JSONArray(prefs.getString("outbox:"+address,"[]"));for(int i=0;i<Math.min(50,array.length());i++){JSONObject v=array.getJSONObject(i);pending.put(v.getString("id"),v);outgoingText.put(v.getString("id"),v.getString("text"));}}catch(JSONException ignored){}peer=address;}
    private synchronized void flush(){for(JSONObject v:pending.values())writeAsync(v);}
    private void writeAsync(JSONObject value){final int g=generation;final String address=peer;if(writer.isShutdown())return;writer.execute(()->{if(generation!=g||!address.equals(peer))return;try{write(value);}catch(Exception e){close(socket);}});}
    private synchronized void write(JSONObject value)throws IOException{BluetoothSocket current=socket;if(!transportReady||current==null||!current.isConnected())return;byte[] bytes=(value.toString()+"\n").getBytes(StandardCharsets.UTF_8);if(bytes.length>16384)throw new IOException("Quadro muito grande");current.getOutputStream().write(bytes);current.getOutputStream().flush();}
    private void heartbeat(){if(!running)return;if(socket!=null&&socket.isConnected()){if(System.currentTimeMillis()-lastSeen>60000){close(socket);return;}writeAsync(frame("ping","id","heartbeat"));flush();}}
    private static String modelName(){return Build.MODEL.substring(0,Math.min(40,Build.MODEL.length()));}
    private static JSONObject frame(String type,String...pairs){JSONObject v=new JSONObject();try{v.put("type",type);for(int i=0;i<pairs.length;i+=2)v.put(pairs[i],pairs[i+1]);}catch(JSONException ignored){}return v;}
    private static void close(BluetoothSocket s){if(s!=null)try{s.close();}catch(IOException ignored){}}
    private void closeTransport(){generation++;running=false;transportReady=false;close(socket);socket=null;if(listener!=null)try{listener.close();}catch(IOException ignored){}listener=null;}
    public void stopChat(){closeTransport();status("Bluetooth encerrado. Mensagens pendentes são mantidas neste aparelho.");stopForeground(STOP_FOREGROUND_REMOVE);stopSelf();}
    @Override public void onDestroy(){closeTransport();writer.shutdownNow();worker.shutdownNow();super.onDestroy();}
}
