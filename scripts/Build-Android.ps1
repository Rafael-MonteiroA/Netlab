param(
    [Parameter(Mandatory=$true)][string]$SdkRoot,
    [Parameter(Mandatory=$true)][string]$JdkRoot,
    [Parameter(Mandatory=$true)][string]$BuildDirectory,
    [Parameter(Mandatory=$true)][string]$OutputApk
)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
$androidRoot=Join-Path $projectRoot 'android'
$androidJar=Join-Path $SdkRoot 'platforms\android-35\android.jar'
$buildTools=Join-Path $SdkRoot 'build-tools\35.0.0'
$classes=Join-Path $BuildDirectory 'classes'
$dex=Join-Path $BuildDirectory 'dex'
New-Item -ItemType Directory -Force -Path $BuildDirectory,$classes,$dex,(Split-Path -Parent $OutputApk) | Out-Null
$env:JAVA_HOME=$JdkRoot
function Invoke-BuildTool([string]$Executable,[string[]]$ToolArgs) {
    & $Executable @ToolArgs
    if($LASTEXITCODE -ne 0){throw "Falha em $Executable (codigo $LASTEXITCODE)"}
}
$javaFiles=Get-ChildItem (Join-Path $androidRoot 'app\src\main\java') -Recurse -Filter *.java | Select-Object -ExpandProperty FullName
Invoke-BuildTool (Join-Path $JdkRoot 'bin\javac.exe') (@('-encoding','UTF-8','--release','8','-classpath',$androidJar,'-d',$classes)+$javaFiles)
$classesJar=Join-Path $BuildDirectory 'classes.jar'
Invoke-BuildTool (Join-Path $JdkRoot 'bin\jar.exe') @('cf',$classesJar,'-C',$classes,'.')
Invoke-BuildTool (Join-Path $buildTools 'd8.bat') @('--lib',$androidJar,'--min-api','26','--output',$dex,$classesJar)
$unsigned=Join-Path $BuildDirectory 'unsigned.apk'
# AGP supplies the namespace; direct aapt2 needs it in a temporary manifest.
$manifest=Join-Path $BuildDirectory 'AndroidManifest.xml'
(Get-Content (Join-Path $androidRoot 'app\src\main\AndroidManifest.xml') -Raw).Replace('<manifest ', '<manifest package="br.netlab.bluetooth" ') | Set-Content -LiteralPath $manifest -Encoding utf8
Invoke-BuildTool (Join-Path $buildTools 'aapt2.exe') @('link','-o',$unsigned,'-I',$androidJar,'--manifest',$manifest,'--min-sdk-version','26','--target-sdk-version','35','--version-code','4','--version-name','0.4.0')
Invoke-BuildTool (Join-Path $JdkRoot 'bin\jar.exe') @('uf',$unsigned,'-C',$dex,'classes.dex')
$aligned=Join-Path $BuildDirectory 'aligned.apk'
Invoke-BuildTool (Join-Path $buildTools 'zipalign.exe') @('-f','4',$unsigned,$aligned)
# Development-only key, generated locally. Use your own release signing key for distribution.
$key=Join-Path $BuildDirectory 'netlab-development.p12'
if(-not(Test-Path -LiteralPath $key)){
    Invoke-BuildTool (Join-Path $JdkRoot 'bin\keytool.exe') @('-genkeypair','-keystore',$key,'-storepass','android','-keypass','android','-alias','netlab-development','-dname','CN=NetLab Development','-keyalg','RSA','-keysize','2048','-validity','3650')
}
Invoke-BuildTool (Join-Path $buildTools 'apksigner.bat') @('sign','--ks',$key,'--ks-pass','pass:android','--out',$OutputApk,$aligned)
Invoke-BuildTool (Join-Path $buildTools 'apksigner.bat') @('verify','--verbose',$OutputApk)
Write-Output "APK de desenvolvimento gerado: $OutputApk"
