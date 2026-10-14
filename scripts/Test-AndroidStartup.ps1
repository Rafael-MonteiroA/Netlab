param(
    [Parameter(Mandatory=$true)][string]$SdkRoot,
    [Parameter(Mandatory=$true)][string]$JdkRoot,
    [Parameter(Mandatory=$true)][string]$BuildDirectory,
    [Parameter(Mandatory=$true)][string]$SigningKey,
    [Parameter(Mandatory=$true)][string]$DeviceSerial,
    [switch]$TestCamera
)
$ErrorActionPreference='Stop'
# Install the production APK first. This separate instrumentation package uses the
# development signing key; never package this test runner inside the application.
$projectRoot=Split-Path -Parent $PSScriptRoot
$env:JAVA_HOME=$JdkRoot
$androidJar=Join-Path $SdkRoot 'platforms/android-35/android.jar'
$buildTools=Join-Path $SdkRoot 'build-tools/35.0.0'
$adb=Join-Path $SdkRoot 'platform-tools/adb.exe'
function Invoke-Tool([string]$Executable,[string[]]$ToolArgs){
    & $Executable @ToolArgs
    if($LASTEXITCODE -ne 0){throw "Falha em $Executable ($LASTEXITCODE)"}
}
New-Item -ItemType Directory -Force -Path $BuildDirectory,"$BuildDirectory/classes","$BuildDirectory/dex" | Out-Null
Invoke-Tool "$JdkRoot/bin/javac.exe" @('--release','8','-classpath',$androidJar,'-d',"$BuildDirectory/classes","$projectRoot/android/tests/StartupSmoke.java")
Invoke-Tool "$JdkRoot/bin/jar.exe" @('cf',"$BuildDirectory/classes.jar",'-C',"$BuildDirectory/classes",'.')
Invoke-Tool "$buildTools/d8.bat" @('--lib',$androidJar,'--min-api','26','--output',"$BuildDirectory/dex","$BuildDirectory/classes.jar")
'<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="br.netlab.smoke"><application android:label="NetLab startup test"/><instrumentation android:name="br.netlab.smoke.StartupSmoke" android:targetPackage="br.netlab.bluetooth"/></manifest>' | Set-Content "$BuildDirectory/AndroidManifest.xml" -Encoding utf8
Invoke-Tool "$buildTools/aapt2.exe" @('link','-o',"$BuildDirectory/unsigned.apk",'-I',$androidJar,'--manifest',"$BuildDirectory/AndroidManifest.xml",'--min-sdk-version','26','--target-sdk-version','35')
Invoke-Tool "$JdkRoot/bin/jar.exe" @('u0f',"$BuildDirectory/unsigned.apk",'-C',"$BuildDirectory/dex",'classes.dex')
Invoke-Tool "$buildTools/zipalign.exe" @('-f','4',"$BuildDirectory/unsigned.apk","$BuildDirectory/aligned.apk")
Invoke-Tool "$buildTools/apksigner.bat" @('sign','--v4-signing-enabled','false','--ks',$SigningKey,'--ks-pass','pass:android','--out',"$BuildDirectory/smoke.apk","$BuildDirectory/aligned.apk")
Invoke-Tool $adb @('-s',$DeviceSerial,'install','-r',"$BuildDirectory/smoke.apk")
$instrumentArgs=@('-s',$DeviceSerial,'shell','am','instrument','-w')
if($TestCamera){$instrumentArgs+=@('-e','camera','true')}
$instrumentArgs+='br.netlab.smoke/br.netlab.smoke.StartupSmoke'
$result=& $adb @instrumentArgs
$result | Tee-Object -FilePath "$BuildDirectory/result.txt"
if($LASTEXITCODE -ne 0 -or ($result -join "`n") -notmatch 'result=PASS:'){throw 'O teste de abertura Android falhou'}
