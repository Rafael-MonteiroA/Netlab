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
    private final LinkedHashMap<String,JSONObject> timeline=new LinkedHashMap<>();
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
    private HistoryStore store;
    private String storageError="";
    private int historyOffset;
    private long draftReset;
    @Override public void onCreate(){super.onCreate();prefs=getSharedPreferences("netlab",MODE_PRIVATE);installation=prefs.getString("installation",null);if(installation==null){installation=UUID.randomUUID().toString().replace("-","");prefs.edit().putString("installation",installation).apply();}try{store=new HistoryStore(this);migratePending();String last=store.getSetting("lastPeer","");if(!last.isEmpty())loadPending(last);}catch(RuntimeException e){storageError="Não foi possível abrir o histórico protegido. Os arquivos foram preservados.";}worker.scheduleWithFixedDelay(this::heartbeat,15,15,TimeUnit.SECONDS);}
    @Override public IBinder onBind(Intent i){return binder;}
    @Override public int onStartCommand(Intent intent,int flags,int startId){
        NotificationManager nm=getSystemService(NotificationManager.class);nm.createNotificationChannel(new NotificationChannel("bluetooth","Conexão Bluetooth",NotificationManager.IMPORTANCE_LOW));
        PendingIntent open=PendingIntent.getActivity(this,0,new Intent(this,MainActivity.class),PendingIntent.FLAG_IMMUTABLE);
        Notification n=new Notification.Builder(this,"bluetooth").setContentTitle("NetLab Bluetooth ativo").setContentText("Mensagens sem internet. Abra para conversar ou parar.").setSmallIcon(android.R.drawable.stat_sys_data_bluetooth).setContentIntent(open).setOngoing(true).build();
        if(Build.VERSION.SDK_INT>=29)startForeground(1,n,ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE);else startForeground(1,n);
        if(store==null||!storageError.isEmpty()){stopForeground(STOP_FOREGROUND_REMOVE);stopSelf();return START_NOT_STICKY;}
        if(intent!=null&&intent.getAction()!=null){closeTransport();running=true;mode=intent.getAction();target=intent.getStringExtra("address");final int g=generation;
            if("connect".equals(mode)&&target!=null){loadPending(target);worker.execute(()->connectLoop(g));}else{mode="listen";worker.execute(()->listenLoop(g));}}
        return START_NOT_STICKY;
    }
    public synchronized void attach(Runnable callback,boolean shown){observer=callback;visible=shown;if(shown&&prefs.getBoolean("readReceipts",true)){for(String id:new ArrayList<>(incoming.keySet())){if(incoming.get(id).equals("received")){incoming.put(id,"read");writeAsync(frame("ack","id",id,"status","read"));}}}notifyUi();}
    public String status(){return state;}
    /** A typed view model keeps received and sent bubbles in actual display order. */
    public synchronized JSONObject snapshot(){
        JSONObject result=new JSONObject();JSONArray messages=new JSONArray();
        boolean connected=running&&transportReady&&socket!=null&&socket.isConnected();
        String phase=connected?"connected":!running?"idle":mode.equals("listen")?"listening":state.startsWith("Conectando")?"connecting":"reconnecting";
        try{
            for(Map.Entry<String,JSONObject> entry:timeline.entrySet()){
                JSONObject item=new JSONObject(entry.getValue().toString());
                if(item.optBoolean("mine")){String level=delivery.get(entry.getKey());item.put("status","Exibida no outro aparelho".equals(level)?"read":"Recebida".equals(level)?"received":item.optString("status","queued"));}
                messages.put(item);
            }
            result.put("phase",phase).put("status",state).put("connected",connected).put("running",running)
                .put("peerName",peerName).put("peerAddress",peer).put("canSend",running&&!peer.isEmpty())
                .put("pending",pending.size()).put("messages",messages).put("storageError",storageError).put("historyOffset",historyOffset).put("draftReset",draftReset)
                .put("historyCount",store==null||peer.isEmpty()?0:store.count(peer)).put("conversations",store==null?new JSONArray():store.peers())
                .put("draft",store==null||peer.isEmpty()?"":store.peer(peer).optString("draft","")).put("readReceipts",prefs.getBoolean("readReceipts",true));
        }catch(Exception error){storageError="Histórico protegido indisponível. Os arquivos foram preservados; reabra o aplicativo.";try{result.put("storageError",storageError).put("canSend",false);}catch(JSONException ignored){}}
        return result;
    }
    private synchronized void bubble(String id,String text,boolean mine,long time){
        if(timeline.containsKey(id))return;
        try{timeline.put(id,new JSONObject().put("id",id).put("text",text).put("mine",mine).put("time",time).put("status",mine?"queued":"received"));}
        catch(JSONException ignored){}
        while(timeline.size()>200)timeline.remove(timeline.keySet().iterator().next());
    }
    public synchronized String history(){StringBuilder b=new StringBuilder();for(String s:lines)b.append(s).append("\n\n");for(String id:outgoingText.keySet())b.append("Você: ").append(outgoingText.get(id)).append("\n").append(delivery.getOrDefault(id,"Na fila")).append("\n\n");return b.toString();}
    private void status(String s){state=s;notifyUi();}
    private synchronized void notifyUi(){Runnable r=observer;if(r!=null)ui.post(r);}
    private synchronized void log(String s){lines.add(s);while(lines.size()>150)lines.remove(0);notifyUi();}
    private BluetoothAdapter adapter(){return getSystemService(BluetoothManager.class).getAdapter();}
    private void connectLoop(int g){int attempt=0;while(running&&generation==g){try{status("Conectando por Bluetooth…");if(blocked(target))throw new SecurityException("Aparelho bloqueado no NetLab");BluetoothSocket candidate=adapter().getRemoteDevice(target).createRfcommSocketToServiceRecord(UUID_SERVICE);transportReady=false;socket=candidate;
            ScheduledFuture<?> timeout=worker.schedule(()->close(candidate),20,TimeUnit.SECONDS);try{candidate.connect();}finally{timeout.cancel(false);}if(generation!=g){close(candidate);return;}attempt=0;communicate(candidate,g);
        }catch(Exception e){if(generation==g)status("Bluetooth desconectado. Tentando novamente: "+e.getMessage());}finally{if(generation==g){transportReady=false;close(socket);socket=null;notifyUi();}}
        if(running&&generation==g){try{Thread.sleep(Math.min(30000,1000L<<Math.min(attempt++,5)));}catch(InterruptedException e){return;}}
    }}
    private void listenLoop(int g){while(running&&generation==g){try{
        status("Aguardando Bluetooth. No outro aparelho, escolha Conectar.");BluetoothServerSocket server=adapter().listenUsingRfcommWithServiceRecord("NetLab",UUID_SERVICE);listener=server;
        BluetoothSocket accepted=server.accept();server.close();if(generation!=g){close(accepted);return;}
        // QR invitations only allow the already paired PC selected by the scan.
        if(accepted.getRemoteDevice().getBondState()!=BluetoothDevice.BOND_BONDED||blocked(accepted.getRemoteDevice().getAddress())||(target!=null&&!target.equals(accepted.getRemoteDevice().getAddress()))){close(accepted);continue;}
        transportReady=false;socket=accepted;communicate(accepted,g);
      }catch(Exception e){if(generation==g)status("Não foi possível receber: "+e.getMessage());try{Thread.sleep(1500);}catch(InterruptedException ignored){return;}}
      finally{if(generation==g){transportReady=false;close(socket);socket=null;notifyUi();}}
    }}
    private void communicate(BluetoothSocket current,int g)throws Exception{
        synchronized(this){loadPending(current.getRemoteDevice().getAddress());peerName=current.getRemoteDevice().getName();if(peerName==null)peerName="Outro aparelho";store.savePeer(peer,peerName,null);lastSeen=System.currentTimeMillis();transportReady=true;status("Conectado a "+peerName+" · Bluetooth · sem internet");write(frame("hello","name",modelName()));flush();}
        InputStream input=current.getInputStream();ByteArrayOutputStream line=new ByteArrayOutputStream();int value;
        while(running&&generation==g&&(value=input.read())!=-1){if(value=='\n'){String text=line.toString("UTF-8");line.reset();lastSeen=System.currentTimeMillis();handle(new JSONObject(text));}else{if(line.size()>=16384)throw new IOException("Quadro muito grande");line.write(value);}}
    }
    private synchronized void handle(JSONObject value)throws Exception{
        String type=value.optString("type"),id=value.optString("id");
        if(type.equals("hello")){peerName=value.optString("name","Outro aparelho");if(peerName.length()>80)throw new IOException("Nome inválido");store.savePeer(peer,peerName,null);notifyUi();return;}
        if(type.equals("ping")){write(frame("pong","id",id));return;}if(type.equals("pong"))return;
        if(id.isEmpty()||id.length()>64||!id.matches("[A-Za-z0-9-]+"))throw new IOException("Identificador inválido");
        if(type.equals("ack")){String level=value.optString("status");if(!level.equals("read")&&!level.equals("received"))throw new IOException("Confirmação inválida");if(outgoingText.containsKey(id)){if(level.equals("read"))delivery.put(id,"Exibida no outro aparelho");else if(level.equals("received")&&!"Exibida no outro aparelho".equals(delivery.get(id)))delivery.put(id,"Recebida");}if(timeline.containsKey(id)){JSONObject shown=timeline.get(id);if(level.equals("read")||!shown.optString("status").equals("read"))shown.put("status",level);}store.ack(peer,id,level);pending.remove(id);notifyUi();return;}
        if(!type.equals("chat"))throw new IOException("Tipo de mensagem inválido");String text=value.optString("text");if(text.trim().isEmpty()||text.codePointCount(0,text.length())>4000)throw new IOException("Texto inválido");
        // Per-installation monotonic IDs survive reconnects and app process restarts.
        int split=id.lastIndexOf('-');if(split<1)throw new IOException("Formato do identificador inválido");long seq=Long.parseLong(id.substring(split+1));if(seq<=0||!id.substring(0,split).matches("[a-f0-9]{32}"))throw new IOException("Sequência inválida");String key="seen:"+peer+":"+id.substring(0,split);long seen=prefs.getLong(key,0);
        JSONObject row=new JSONObject().put("id",id).put("text",text).put("mine",false).put("time",System.currentTimeMillis()).put("status","received");
        boolean fresh=store.receive(peer,id.substring(0,split),seq,row,seen);
        if(fresh){if(historyOffset==0)bubble(id,text,false,row.optLong("time"));incoming.put(id,visible&&prefs.getBoolean("readReceipts",true)?"read":"received");while(incoming.size()>256)incoming.remove(incoming.keySet().iterator().next());notifyUi();}
        write(frame("ack","id",id,"status",incoming.getOrDefault(id,"received")));

    }
    public synchronized boolean sendText(String text){
        if(store==null||!storageError.isEmpty()){status("Armazenamento indisponível. Mensagem não enviada.");return false;}if(!running){status("Conecte a um aparelho antes de enviar.");return false;}if(text.trim().isEmpty())return false;if(text.codePointCount(0,text.length())>4000){status("Máximo de 4000 caracteres.");return false;}
        if(peer.isEmpty()&&target==null){status("Aguarde a primeira conexão antes de enviar.");return false;}if(pending.size()>=50){status("Aguarde o envio das 50 mensagens pendentes.");return false;}
        long seq=prefs.getLong("sequence",0)+1;if(!prefs.edit().putLong("sequence",seq).commit())throw new IllegalStateException("Não foi possível salvar a sequência");String id=installation+"-"+seq;JSONObject value=frame("chat","id",id,"name",modelName(),"text",text.trim());try{value.put("localTime",System.currentTimeMillis());}catch(JSONException ignored){}JSONObject row=frame("chat","id",id,"text",text.trim());try{row.put("mine",true).put("time",value.optLong("localTime")).put("status","queued");}catch(JSONException ignored){}store.queue(peer,value,row);historyOffset=0;loadHistoryPage();pending.put(id,value);bubble(id,text.trim(),true,value.optLong("localTime"));outgoingText.put(id,text.trim());delivery.put(id,"Na fila / aguardando confirmação");while(outgoingText.size()>100){String first=outgoingText.keySet().iterator().next();outgoingText.remove(first);delivery.remove(first);}store.savePeer(peer,peerName,"");flush();notifyUi();return true;
    }
    private void migratePending(){
        for(Map.Entry<String,?> entry:prefs.getAll().entrySet())if(entry.getKey().startsWith("outbox:")&&entry.getValue() instanceof String){
            String address=entry.getKey().substring(7);
            try{JSONArray array=new JSONArray((String)entry.getValue());for(int i=0;i<array.length();i++){JSONObject v=array.getJSONObject(i);JSONObject row=new JSONObject().put("id",v.getString("id")).put("text",v.getString("text")).put("mine",true).put("time",v.optLong("localTime")).put("status","queued");store.queue(address,v,row);}store.savePeer(address,null,null);if(store.getSetting("lastPeer","").isEmpty())store.setSetting("lastPeer",address);prefs.edit().remove(entry.getKey()).commit();}catch(JSONException e){throw new IllegalStateException("Fila antiga inválida",e);}
        }
    }
    private synchronized void loadPending(String address){
        if(address.equals(peer))return;pending.clear();outgoingText.clear();delivery.clear();incoming.clear();lines.clear();timeline.clear();historyOffset=0;
        peer=address;store.savePeer(peer,null,null);peerName=store.peer(peer).optString("name","Aparelho salvo");store.setSetting("lastPeer",peer);
        JSONArray queue=store.pending(peer);for(int i=0;i<queue.length();i++){JSONObject v=queue.optJSONObject(i);pending.put(v.optString("id"),v);outgoingText.put(v.optString("id"),v.optString("text"));}
        loadHistoryPage();
    }
    private void loadHistoryPage(){timeline.clear();if(store==null||peer.isEmpty())return;JSONArray rows=store.history(peer,historyOffset,200);for(int i=0;i<rows.length();i++){JSONObject row=rows.optJSONObject(i);timeline.put(row.optString("id"),row);}}
    public synchronized void historyPage(boolean older){if(store==null)return;int total=store.count(peer);historyOffset=older?Math.min(Math.max(0,total-1),historyOffset+200):Math.max(0,historyOffset-200);loadHistoryPage();notifyUi();}
    public synchronized void selectHistory(String address){if(!address.matches("([0-9A-F]{2}:){5}[0-9A-F]{2}"))return;if(!peer.equals(address)){stopChat();loadPending(address);}notifyUi();}
    public synchronized void draft(String address,String text){if(store!=null&&peer.equals(address)&&text.length()<=4000)store.savePeer(address,null,text);}
    public synchronized void clearHistory(boolean all){draftReset++;stopChat();if(store!=null){if(all){store.clearAll();peer="";peerName="Outro aparelho";}else store.clear(peer);}pending.clear();outgoingText.clear();delivery.clear();incoming.clear();timeline.clear();historyOffset=0;notifyUi();}
    public boolean blocked(String address){return prefs.getBoolean("blocked:"+HistoryStore.peerKey(address),false);}
    public synchronized void block(String address,boolean value){if(!address.matches("([0-9A-F]{2}:){5}[0-9A-F]{2}"))return;prefs.edit().putBoolean("blocked:"+HistoryStore.peerKey(address),value).commit();if(value&&(address.equals(peer)||address.equals(target)))stopChat();notifyUi();}
    public synchronized void readReceipts(boolean value){prefs.edit().putBoolean("readReceipts",value).commit();notifyUi();}
    public synchronized void exportHistory(OutputStream output)throws IOException{if(store==null||peer.isEmpty())throw new IOException("Nenhuma conversa selecionada");java.io.Writer out=new OutputStreamWriter(output,StandardCharsets.UTF_8);out.write("NetLab — conversa exportada (texto sem proteção)\n\n");for(int offset=0;;offset+=200){JSONArray rows=store.history(peer,offset,200);if(rows.length()==0)break;for(int i=rows.length()-1;i>=0;i--){JSONObject m=rows.optJSONObject(i);out.write(new java.util.Date(m.optLong("time"))+" | "+(m.optBoolean("mine")?"Você":peerName)+": "+m.optString("text")+"\n");}}out.flush();}
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
