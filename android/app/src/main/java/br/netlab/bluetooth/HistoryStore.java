package br.netlab.bluetooth;

import android.content.*;
import android.database.Cursor;
import android.database.sqlite.*;
import android.security.keystore.*;
import android.util.Base64;
import java.nio.charset.StandardCharsets;
import java.security.*;
import java.util.*;
import javax.crypto.*;
import javax.crypto.spec.GCMParameterSpec;
import org.json.*;

/** SQLite transactions + authenticated encryption. No plaintext message fallback. */
public final class HistoryStore extends SQLiteOpenHelper {
    private final SecretKey key;
    public HistoryStore(Context context){this(context,"history-v1.db","netlab-history-v1");}
    public HistoryStore(Context context,String file,String alias){
        super(context,file,null,1);
        try{
            KeyStore ks=KeyStore.getInstance("AndroidKeyStore");ks.load(null);
            if(!ks.containsAlias(alias)){
                KeyGenerator generator=KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES,"AndroidKeyStore");
                generator.init(new KeyGenParameterSpec.Builder(alias,KeyProperties.PURPOSE_ENCRYPT|KeyProperties.PURPOSE_DECRYPT)
                    .setKeySize(256).setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE).build());generator.generateKey();
            }
            key=(SecretKey)ks.getKey(alias,null);getWritableDatabase();
        }catch(Exception e){throw new IllegalStateException("Não foi possível abrir o histórico protegido. Seus arquivos foram preservados.",e);}
    }
    @Override public void onConfigure(SQLiteDatabase db){try(Cursor result=db.rawQuery("PRAGMA secure_delete=ON",null)){if(!result.moveToFirst()||result.getInt(0)!=1)throw new SQLiteException("Não foi possível configurar exclusão segura");}}
    public void onCreate(SQLiteDatabase db){
        db.execSQL("CREATE TABLE peers(k TEXT PRIMARY KEY, data TEXT NOT NULL, updated INTEGER NOT NULL)");
        db.execSQL("CREATE TABLE messages(n INTEGER PRIMARY KEY AUTOINCREMENT, peer TEXT NOT NULL, id TEXT NOT NULL, data TEXT NOT NULL, UNIQUE(peer,id))");
        db.execSQL("CREATE TABLE outbox(peer TEXT NOT NULL,id TEXT NOT NULL,data TEXT NOT NULL,PRIMARY KEY(peer,id))");
        db.execSQL("CREATE TABLE seen(peer TEXT NOT NULL,prefix TEXT NOT NULL,seq INTEGER NOT NULL,PRIMARY KEY(peer,prefix))");
        db.execSQL("CREATE TABLE settings(k TEXT PRIMARY KEY,data TEXT NOT NULL)");
    }
    public void onUpgrade(SQLiteDatabase db,int old,int next){throw new IllegalStateException("Versão do histórico não suportada");}
    public static String peerKey(String address){try{byte[] hash=MessageDigest.getInstance("SHA-256").digest(address.getBytes(StandardCharsets.UTF_8));return Base64.encodeToString(hash,Base64.NO_WRAP);}catch(Exception e){throw new IllegalStateException(e);}}
    private String crypt(String data,String aad,boolean encrypt){
        try{
            Cipher cipher=Cipher.getInstance("AES/GCM/NoPadding");
            if(encrypt){cipher.init(Cipher.ENCRYPT_MODE,key);cipher.updateAAD(aad.getBytes(StandardCharsets.UTF_8));byte[] ciphertext=cipher.doFinal(data.getBytes(StandardCharsets.UTF_8));byte[] packed=new byte[12+ciphertext.length];System.arraycopy(cipher.getIV(),0,packed,0,12);System.arraycopy(ciphertext,0,packed,12,ciphertext.length);return Base64.encodeToString(packed,Base64.NO_WRAP);}
            byte[] packed=Base64.decode(data,Base64.NO_WRAP);if(packed.length<28)throw new GeneralSecurityException("Invalid ciphertext");
            cipher.init(Cipher.DECRYPT_MODE,key,new GCMParameterSpec(128,Arrays.copyOf(packed,12)));cipher.updateAAD(aad.getBytes(StandardCharsets.UTF_8));return new String(cipher.doFinal(packed,12,packed.length-12),StandardCharsets.UTF_8);
        }catch(Exception e){throw new IllegalStateException("Falha no histórico protegido. Não desinstale o aplicativo; tente abrir novamente.",e);}
    }
    private JSONObject json(String text){try{return new JSONObject(text);}catch(JSONException e){throw new IllegalStateException("Histórico inválido",e);}}
    private String cell(String sql,String...args){try(Cursor c=getReadableDatabase().rawQuery(sql,args)){return c.moveToFirst()?c.getString(0):null;}}
    public synchronized JSONObject peer(String address){String k=peerKey(address),raw=cell("SELECT data FROM peers WHERE k=?",k);return raw==null?new JSONObject():json(crypt(raw,"peer:"+k,false));}
    public synchronized void savePeer(String address,String name,String draft){
        if(address.isEmpty())return;String k=peerKey(address);JSONObject p=peer(address);
        try{p.put("address",address);if(name!=null)p.put("name",name);if(draft!=null)p.put("draft",draft);if(!p.has("name"))p.put("name","Aparelho salvo");}catch(JSONException e){throw new IllegalStateException(e);}
        ContentValues v=new ContentValues();v.put("k",k);v.put("data",crypt(p.toString(),"peer:"+k,true));v.put("updated",System.currentTimeMillis());if(getWritableDatabase().insertWithOnConflict("peers",null,v,SQLiteDatabase.CONFLICT_REPLACE)==-1)throw new SQLiteException("Não foi possível salvar o histórico");
    }
    public synchronized JSONArray peers(){JSONArray list=new JSONArray();try(Cursor c=getReadableDatabase().rawQuery("SELECT k,data FROM peers ORDER BY updated DESC",null)){while(c.moveToNext()){JSONObject p=json(crypt(c.getString(1),"peer:"+c.getString(0),false));p.remove("draft");list.put(p);}}return list;}
    public synchronized String getSetting(String name,String fallback){String raw=cell("SELECT data FROM settings WHERE k=?",name);return raw==null?fallback:crypt(raw,"setting:"+name,false);}
    public synchronized void setSetting(String name,String value){ContentValues v=new ContentValues();v.put("k",name);v.put("data",crypt(value,"setting:"+name,true));if(getWritableDatabase().insertWithOnConflict("settings",null,v,SQLiteDatabase.CONFLICT_REPLACE)==-1)throw new SQLiteException("Não foi possível salvar o histórico");}
    public synchronized void message(String address,JSONObject row){String peer=peerKey(address),id=row.optString("id");ContentValues v=new ContentValues();v.put("peer",peer);v.put("id",id);v.put("data",crypt(row.toString(),"message:"+peer+":"+id,true));int updated=getWritableDatabase().update("messages",v,"peer=? AND id=?",new String[]{peer,id});if(updated==0)getWritableDatabase().insertOrThrow("messages",null,v);}
    public synchronized void queue(String address,JSONObject frame,JSONObject row){SQLiteDatabase db=getWritableDatabase();db.beginTransaction();try{message(address,row);String peer=peerKey(address),id=frame.optString("id");ContentValues v=new ContentValues();v.put("peer",peer);v.put("id",id);v.put("data",crypt(frame.toString(),"outbox:"+peer+":"+id,true));if(db.insertWithOnConflict("outbox",null,v,SQLiteDatabase.CONFLICT_REPLACE)==-1)throw new SQLiteException("Não foi possível salvar o histórico");db.setTransactionSuccessful();}finally{db.endTransaction();}}
    public synchronized void ack(String address,String id,String status){String p=peerKey(address),raw=cell("SELECT data FROM messages WHERE peer=? AND id=?",p,id);SQLiteDatabase db=getWritableDatabase();db.beginTransaction();try{if(raw!=null){JSONObject row=json(crypt(raw,"message:"+p+":"+id,false));try{if(!"read".equals(row.optString("status")))row.put("status",status);}catch(JSONException e){throw new IllegalStateException(e);}message(address,row);}db.delete("outbox","peer=? AND id=?",new String[]{p,id});db.setTransactionSuccessful();}finally{db.endTransaction();}}
    public synchronized JSONArray pending(String address){String p=peerKey(address);JSONArray rows=new JSONArray();try(Cursor c=getReadableDatabase().rawQuery("SELECT id,data FROM outbox WHERE peer=? ORDER BY rowid",new String[]{p})){while(c.moveToNext())rows.put(json(crypt(c.getString(1),"outbox:"+p+":"+c.getString(0),false)));}return rows;}
    public synchronized boolean receive(String address,String prefix,long seq,JSONObject row,long legacySeen){SQLiteDatabase db=getWritableDatabase();String p=peerKey(address);db.beginTransaction();try{String old=cell("SELECT seq FROM seen WHERE peer=? AND prefix=?",p,prefix);long last=Math.max(legacySeen,old==null?0:Long.parseLong(old));if(seq<=last){db.setTransactionSuccessful();return false;}message(address,row);ContentValues v=new ContentValues();v.put("peer",p);v.put("prefix",prefix);v.put("seq",seq);if(db.insertWithOnConflict("seen",null,v,SQLiteDatabase.CONFLICT_REPLACE)==-1)throw new SQLiteException("Não foi possível salvar o histórico");db.setTransactionSuccessful();return true;}finally{db.endTransaction();}}
    public synchronized JSONArray history(String address,int offset,int limit){String p=peerKey(address);ArrayList<JSONObject> reversed=new ArrayList<>();try(Cursor c=getReadableDatabase().rawQuery("SELECT id,data FROM messages WHERE peer=? ORDER BY n DESC LIMIT ? OFFSET ?",new String[]{p,String.valueOf(limit),String.valueOf(offset)})){while(c.moveToNext())reversed.add(json(crypt(c.getString(1),"message:"+p+":"+c.getString(0),false)));}Collections.reverse(reversed);return new JSONArray(reversed);}
    public synchronized int count(String address){return Integer.parseInt(cell("SELECT count(*) FROM messages WHERE peer=?",peerKey(address)));}
    public synchronized void clear(String address){String p=peerKey(address);SQLiteDatabase db=getWritableDatabase();db.beginTransaction();try{db.delete("messages","peer=?",new String[]{p});db.delete("outbox","peer=?",new String[]{p});savePeer(address,null,"");db.setTransactionSuccessful();}finally{db.endTransaction();}}
    public synchronized void clearAll(){SQLiteDatabase db=getWritableDatabase();db.beginTransaction();try{db.delete("messages",null,null);db.delete("outbox",null,null);db.delete("peers",null,null);setSetting("lastPeer","");db.setTransactionSuccessful();}finally{db.endTransaction();}}
}
