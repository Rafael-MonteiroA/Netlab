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
    @Override public void onCreate(Bundle args) { super.onCreate(args); start(); }
    @Override public void onStart() {
        Bundle report = new Bundle();
        try {
            Intent intent = new Intent().setClassName("br.netlab.bluetooth", "br.netlab.bluetooth.MainActivity")
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
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
            report.putString("result", "PASS: activity starts; React renders; native bridge present; offline send disabled");
            runOnMainSync(activity::finish);
            finish(Activity.RESULT_OK, report);
        } catch (Throwable error) {
            report.putString("result", "FAIL: " + error);
            finish(Activity.RESULT_CANCELED, report);
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
