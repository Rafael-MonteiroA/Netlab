package br.netlab.smoke;

import android.app.*;
import android.content.Intent;
import android.os.Bundle;
import android.webkit.WebView;
import java.lang.reflect.Field;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicReference;

/** Run only in a test emulator; uses the installed release APK without debug flags. */
public final class StartupSmoke extends Instrumentation {
    private boolean testCamera;
    @Override public void onCreate(Bundle args) { super.onCreate(args); testCamera=args!=null&&"true".equals(args.getString("camera"));start(); }
    @Override public void onStart() {
        Bundle report = new Bundle();
        try {
            Intent intent = new Intent().setClassName("br.netlab.bluetooth", "br.netlab.bluetooth.MainActivity")
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            testHistory();
            Activity activity = startActivitySync(intent);
            Field field = activity.getClass().getDeclaredField("web");
            field.setAccessible(true);
            WebView web = (WebView)field.get(activity);
            boolean ready = false;
            long deadline = System.currentTimeMillis() + 30000;
            while (System.currentTimeMillis() < deadline) {
                String value = evaluate(web, "Boolean(document.querySelector('.page-title') && document.body.innerText.includes('Mais perto'))");
                if ("true".equals(value)) { ready = true; break; }
                Thread.sleep(250);
            }
            if (!ready) throw new AssertionError("React did not render: " + evaluate(web, "document.body.innerText"));
            if (!"true".equals(evaluate(web, "typeof NetLabNative.postMessage === 'function' && !document.querySelector('.preview-banner')")))
                throw new AssertionError("Native bridge unavailable or preview mode enabled");
            if (!"true".equals(evaluate(web, "document.querySelector('button[aria-label=\"Enviar mensagem\"]').disabled")))
                throw new AssertionError("Send enabled before connection");
            report.putString("result", "PASS: encrypted history survives reopen; drafts and queue recover; duplicate suppression survives deletion; failed save rolls back; activity starts; React renders; native bridge present; offline send disabled");
            runOnMainSync(activity::finish);
            if(testCamera){
                Activity scanner=startActivitySync(new Intent().setClassName("br.netlab.bluetooth","br.netlab.bluetooth.QrScannerActivity").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));
                Field cameraField=scanner.getClass().getDeclaredField("camera");cameraField.setAccessible(true);
                AtomicReference<Object> camera=new AtomicReference<>();long cameraDeadline=System.currentTimeMillis()+15000;
                do{runOnMainSync(()->{try{camera.set(cameraField.get(scanner));}catch(Exception e){throw new RuntimeException(e);}});if(camera.get()!=null)break;Thread.sleep(200);}while(System.currentTimeMillis()<cameraDeadline);
                if(camera.get()==null)throw new AssertionError("Scanner camera did not open");
                runOnMainSync(scanner::finish);waitForIdleSync();
                runOnMainSync(()->{try{camera.set(cameraField.get(scanner));}catch(Exception e){throw new RuntimeException(e);}});
                if(camera.get()!=null)throw new AssertionError("Scanner did not release camera");
                report.putString("result",report.getString("result")+"; scanner opens and releases camera");
            }
            finish(Activity.RESULT_OK, report);
        } catch (Throwable error) {
            error.printStackTrace();
            report.putString("result", "FAIL: " + error + " cause=" + error.getCause());
            finish(Activity.RESULT_CANCELED, report);
        }
    }
    private void testHistory() throws Exception {
        String file="smoke-history.db",alias="netlab-smoke-history",peer="AA:BB:CC:DD:EE:FF";
        android.content.Context ctx=getTargetContext();ctx.deleteDatabase(file);
        Class<?> c=ctx.getClassLoader().loadClass("br.netlab.bluetooth.HistoryStore");
        java.lang.reflect.Constructor<?> constructor=c.getConstructor(android.content.Context.class,String.class,String.class);
        Object store=constructor.newInstance(ctx,file,alias);
        try {
            c.getMethod("savePeer",String.class,String.class,String.class).invoke(store,peer,"Test peer","private draft sentinel");
            org.json.JSONObject row=new org.json.JSONObject().put("id","out-1").put("text","private message sentinel").put("mine",true).put("status","queued");
            c.getMethod("queue",String.class,org.json.JSONObject.class,org.json.JSONObject.class).invoke(store,peer,row,row);
            c.getMethod("close").invoke(store);
            byte[] bytes=java.nio.file.Files.readAllBytes(ctx.getDatabasePath(file).toPath());
            if(new String(bytes,java.nio.charset.StandardCharsets.ISO_8859_1).contains("private"))throw new AssertionError("Plaintext in database");
            store=constructor.newInstance(ctx,file,alias);
            org.json.JSONArray history=(org.json.JSONArray)c.getMethod("history",String.class,int.class,int.class).invoke(store,peer,0,200);
            if(history.length()!=1||!history.getJSONObject(0).getString("text").equals("private message sentinel"))throw new AssertionError("History not restored");
            if(!((org.json.JSONObject)c.getMethod("peer",String.class).invoke(store,peer)).getString("draft").equals("private draft sentinel"))throw new AssertionError("Draft not restored");
            if(((org.json.JSONArray)c.getMethod("pending",String.class).invoke(store,peer)).length()!=1)throw new AssertionError("Queue not restored");
            if(((Integer)c.getMethod("count",String.class).invoke(store,"00:11:22:33:44:55"))!=0)throw new AssertionError("Peer isolation");
            c.getMethod("ack",String.class,String.class,String.class).invoke(store,peer,"out-1","received");
            if(((org.json.JSONArray)c.getMethod("pending",String.class).invoke(store,peer)).length()!=0)throw new AssertionError("Ack not saved");
            java.lang.reflect.Method receive=c.getMethod("receive",String.class,String.class,long.class,org.json.JSONObject.class,long.class);
            row.put("id","in-1").put("mine",false);
            if(!(Boolean)receive.invoke(store,peer,"prefix",1L,row,0L))throw new AssertionError("Receive failed");
            c.getMethod("clear",String.class).invoke(store,peer);
            if((Boolean)receive.invoke(store,peer,"prefix",1L,row,0L))throw new AssertionError("Deleted message replayed");
            android.database.sqlite.SQLiteDatabase db=(android.database.sqlite.SQLiteDatabase)c.getMethod("getWritableDatabase").invoke(store);
            db.execSQL("CREATE TRIGGER fail_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'test'); END");
            boolean failed=false;
            try{c.getMethod("queue",String.class,org.json.JSONObject.class,org.json.JSONObject.class).invoke(store,peer,row,row);}catch(java.lang.reflect.InvocationTargetException expected){failed=true;}
            if(!failed||((Integer)c.getMethod("count",String.class).invoke(store,peer))!=0)throw new AssertionError("Partial save was not rolled back");
            db.execSQL("DROP TRIGGER fail_outbox");
            c.getMethod("message",String.class,org.json.JSONObject.class).invoke(store,peer,row);
            db.execSQL("UPDATE messages SET data='invalid'");failed=false;
            try{c.getMethod("history",String.class,int.class,int.class).invoke(store,peer,0,200);}catch(java.lang.reflect.InvocationTargetException expected){failed=true;}
            if(!failed)throw new AssertionError("Tampered history accepted");
        } finally {
            c.getMethod("close").invoke(store);ctx.deleteDatabase(file);
            java.security.KeyStore ks=java.security.KeyStore.getInstance("AndroidKeyStore");ks.load(null);ks.deleteEntry(alias);
        }
    }
    private String evaluate(WebView web, String script) throws Exception {
        CountDownLatch latch = new CountDownLatch(1);
        AtomicReference<String> value = new AtomicReference<>();
        runOnMainSync(() -> web.evaluateJavascript(script, result -> { value.set(result); latch.countDown(); }));
        if (!latch.await(10, TimeUnit.SECONDS)) throw new AssertionError("WebView response timeout");
        return value.get();
    }
}
