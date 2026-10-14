package br.netlab.bluetooth;

import android.app.Activity;
import android.content.Intent;
import android.graphics.Color;
import android.hardware.Camera;
import android.os.Bundle;
import android.view.*;
import android.widget.*;
import com.google.zxing.*;
import com.google.zxing.common.HybridBinarizer;
import com.google.zxing.qrcode.QRCodeReader;
import java.util.*;
import java.util.concurrent.*;

/** Frames are decoded locally; no uploads, network, or Google services. */
@SuppressWarnings("deprecation")
public final class QrScannerActivity extends Activity implements SurfaceHolder.Callback {
    private SurfaceView preview;
    private TextView status;
    private Camera camera;
    private boolean resumed,surfaceReady;
    private volatile boolean active;
    private final ExecutorService decoder=Executors.newSingleThreadExecutor();
    @Override public void onCreate(Bundle state){
        super.onCreate(state);
        LinearLayout page=new LinearLayout(this);page.setOrientation(LinearLayout.VERTICAL);
        page.setBackgroundColor(Color.rgb(20,30,27));page.setPadding(24,24,24,24);
        page.setOnApplyWindowInsetsListener((v,i)->{v.setPadding(24+i.getSystemWindowInsetLeft(),24+i.getSystemWindowInsetTop(),24+i.getSystemWindowInsetRight(),24+i.getSystemWindowInsetBottom());return i;});
        TextView title=new TextView(this);title.setText("Conectar ao PC por QR Code");title.setTextColor(Color.WHITE);title.setTextSize(23);page.addView(title);
        status=new TextView(this);status.setText("No PC: Bluetooth sem Wi-Fi → selecione este celular → Conectar por QR Code. Aponte a câmera para o código.");status.setTextColor(0xffd3f783);status.setPadding(0,20,0,20);page.addView(status);
        preview=new SurfaceView(this);preview.getHolder().addCallback(this);page.addView(preview,new LinearLayout.LayoutParams(-1,0,1));
        Button cancel=new Button(this);cancel.setText("Cancelar leitura");cancel.setOnClickListener(v->finish());page.addView(cancel);setContentView(page);
    }
    @Override public void onResume(){super.onResume();resumed=true;openCamera();}
    @Override public void onPause(){resumed=false;closeCamera();super.onPause();}
    @Override public void onDestroy(){decoder.shutdownNow();super.onDestroy();}
    public void surfaceCreated(SurfaceHolder holder){surfaceReady=true;openCamera();}
    public void surfaceChanged(SurfaceHolder holder,int format,int width,int height){}
    public void surfaceDestroyed(SurfaceHolder holder){surfaceReady=false;closeCamera();}
    private void openCamera(){
        if(!resumed||!surfaceReady||camera!=null)return;
        try{
            int id=-1;Camera.CameraInfo info=new Camera.CameraInfo();
            for(int n=0;n<Camera.getNumberOfCameras();n++){Camera.getCameraInfo(n,info);if(info.facing==Camera.CameraInfo.CAMERA_FACING_BACK){id=n;break;}}
            if(id<0)throw new IllegalStateException("Câmera traseira indisponível");
            camera=Camera.open(id);Camera.Parameters params=camera.getParameters();
            Camera.Size size=Collections.min(params.getSupportedPreviewSizes(),Comparator.comparingLong(s->Math.abs((long)s.width*s.height-1280L*720)));
            params.setPreviewSize(size.width,size.height);params.setPreviewFormat(android.graphics.ImageFormat.NV21);
            List<String> focus=params.getSupportedFocusModes();if(focus!=null&&focus.contains(Camera.Parameters.FOCUS_MODE_CONTINUOUS_PICTURE))params.setFocusMode(Camera.Parameters.FOCUS_MODE_CONTINUOUS_PICTURE);
            camera.setParameters(params);int rotation=getWindowManager().getDefaultDisplay().getRotation();
            int degrees=rotation==Surface.ROTATION_90?90:rotation==Surface.ROTATION_180?180:rotation==Surface.ROTATION_270?270:0;
            camera.setDisplayOrientation((info.orientation-degrees+360)%360);
            camera.setPreviewDisplay(preview.getHolder());camera.startPreview();active=true;nextFrame();
        }catch(Exception e){closeCamera();status.setText("Não foi possível abrir a câmera. Confira a permissão de Câmera nas configurações e tente novamente.");}
    }
    private void nextFrame(){
        if(!active||camera==null)return;
        try{camera.setOneShotPreviewCallback((data,source)->{
            if(!active)return;Camera.Size size=source.getParameters().getPreviewSize();
            decoder.execute(()->{
                String result=null;
                try{PlanarYUVLuminanceSource image=new PlanarYUVLuminanceSource(data,size.width,size.height,0,0,size.width,size.height,false);
                    Map<DecodeHintType,Object> hints=new EnumMap<>(DecodeHintType.class);hints.put(DecodeHintType.TRY_HARDER,Boolean.TRUE);
                    result=new QRCodeReader().decode(new BinaryBitmap(new HybridBinarizer(image)),hints).getText();
                }catch(Exception ignored){}
                final String scanned=result;
                runOnUiThread(()->{if(!active)return;
                    if(scanned!=null){try{BluetoothInvite.parse(scanned);closeCamera();setResult(RESULT_OK,new Intent().putExtra("qr",scanned));finish();return;}
                        catch(IllegalArgumentException e){status.setText(e.getMessage());}}
                    preview.postDelayed(this::nextFrame,180);
                });
            });
        });}catch(RuntimeException e){closeCamera();status.setText("A câmera foi interrompida. Cancele e abra o leitor novamente.");}
    }
    private void closeCamera(){active=false;if(camera!=null){try{camera.setPreviewCallback(null);camera.stopPreview();}catch(RuntimeException ignored){}camera.release();camera=null;}}
}
