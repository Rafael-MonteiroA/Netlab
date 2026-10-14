package br.netlab.bluetooth;
import java.net.URI;
import java.util.Locale;
/** Accept only an offline NetLab invitation, never an arbitrary URL. */
public final class BluetoothInvite {
    private BluetoothInvite() {}
    public static String parse(String raw) {
        try {
            if(raw==null||raw.length()>200)throw new IllegalArgumentException();
            URI uri=new URI(raw);String query=uri.getRawQuery();
            if(!"netlab".equals(uri.getScheme())||!"bluetooth".equals(uri.getHost())||!"/v1".equals(uri.getPath())
                ||uri.getUserInfo()!=null||uri.getPort()!=-1||uri.getFragment()!=null||query==null
                ||!query.matches("pc=([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}"))throw new IllegalArgumentException();
            return query.substring(3).toUpperCase(Locale.ROOT);
        }catch(Exception e){throw new IllegalArgumentException("Use o QR Code da aba Bluetooth do NetLab no PC. O QR de rede Wi-Fi não serve para este modo.");}
    }
}
