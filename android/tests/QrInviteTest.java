import br.netlab.bluetooth.BluetoothInvite;
import com.google.zxing.*;
import com.google.zxing.common.*;
import com.google.zxing.qrcode.*;

/** JVM regression: actual QR encoding/decoding and the same parser used by the camera. */
public final class QrInviteTest {
    public static void main(String[] args) throws Exception {
        String uri="netlab://bluetooth/v1?pc=AA:BB:CC:DD:EE:FF";
        BitMatrix qr=new QRCodeWriter().encode(uri,BarcodeFormat.QR_CODE,400,400);
        byte[] luminance=new byte[400*400];
        for(int y=0;y<400;y++)for(int x=0;x<400;x++)luminance[y*400+x]=(byte)(qr.get(x,y)?0:255);
        PlanarYUVLuminanceSource source=new PlanarYUVLuminanceSource(luminance,400,400,0,0,400,400,false);
        String scanned=new QRCodeReader().decode(new BinaryBitmap(new HybridBinarizer(source))).getText();
        if(!"AA:BB:CC:DD:EE:FF".equals(BluetoothInvite.parse(scanned)))throw new AssertionError("QR round trip");
        for(String bad:new String[]{"https://example.com", "netlab://bluetooth/v1?pc=bad",uri+"&x=1",uri+"#x",uri.replace("/v1","/v2"),"netlab://attacker@bluetooth/v1?pc=AA:BB:CC:DD:EE:FF"}){
            boolean rejected=false;try{BluetoothInvite.parse(bad);}catch(IllegalArgumentException expected){rejected=true;}
            if(!rejected)throw new AssertionError("Accepted invalid invitation: "+bad);
        }
        System.out.println("PASS: camera luminance decoding, invitation parser and invalid QR rejection");
    }
}
