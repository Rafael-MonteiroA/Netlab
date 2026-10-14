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
$zxing=Join-Path $androidRoot 'libs\zxing-core-3.5.3.jar'
if((Get-FileHash -LiteralPath $zxing -Algorithm SHA256).Hash -ne '8D8064C1636FDAEF7189DD9055C7D59950A8940A12F2293956446EC3C109FD82'){throw 'Biblioteca ZXing diferente da versão verificada'}
Invoke-BuildTool (Join-Path $JdkRoot 'bin\javac.exe') (@('-encoding','UTF-8','--release','8','-classpath',"$androidJar;$zxing",'-d',$classes)+$javaFiles)
$classesJar=Join-Path $BuildDirectory 'classes.jar'
Invoke-BuildTool (Join-Path $JdkRoot 'bin\jar.exe') @('cf',$classesJar,'-C',$classes,'.')
Invoke-BuildTool (Join-Path $buildTools 'd8.bat') @('--lib',$androidJar,'--min-api','26','--output',$dex,$classesJar,$zxing)
$unsigned=Join-Path $BuildDirectory 'unsigned.apk'
$resourceApk=Join-Path $BuildDirectory 'resources.apk'
$resources=Join-Path $BuildDirectory 'resources.zip'
Invoke-BuildTool (Join-Path $buildTools 'aapt2.exe') @('compile','--dir',(Join-Path $androidRoot 'app\src\main\res'),'-o',$resources)
$assets=Join-Path $androidRoot 'app\src\main\assets'
if(-not(Test-Path (Join-Path $assets 'ui\app.js'))){throw 'Compile primeiro a interface: cd android/ui; npm ci; npm run build'}
# AGP supplies the namespace; direct aapt2 needs it in a temporary manifest.
$manifest=Join-Path $BuildDirectory 'AndroidManifest.xml'
(Get-Content (Join-Path $androidRoot 'app\src\main\AndroidManifest.xml') -Raw).Replace('<manifest ', '<manifest package="br.netlab.bluetooth" ') | Set-Content -LiteralPath $manifest -Encoding utf8
Invoke-BuildTool (Join-Path $buildTools 'aapt2.exe') @('link','-o',$resourceApk,'-I',$androidJar,'--manifest',$manifest,'-R',$resources,'--min-sdk-version','26','--target-sdk-version','35','--version-code','8','--version-name','0.7.0')
# Create one fresh ZIP instead of mutating a jar-produced archive in Update mode.
# Mixing those writers left inconsistent classes.dex headers that ART rejects.
# DEX and resources.arsc stay uncompressed for Android's direct loading/alignment.
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$assetRoot=(Resolve-Path -LiteralPath $assets).Path
$inputArchive=[IO.Compression.ZipFile]::OpenRead($resourceApk)
$outputStream=[IO.File]::Open($unsigned,[IO.FileMode]::Create)
$archive=[IO.Compression.ZipArchive]::new($outputStream,[IO.Compression.ZipArchiveMode]::Create)
try {
    foreach($resource in $inputArchive.Entries){
        $level=if($resource.FullName -eq 'resources.arsc'){[IO.Compression.CompressionLevel]::NoCompression}else{[IO.Compression.CompressionLevel]::Optimal}
        $entry=$archive.CreateEntry($resource.FullName,$level)
        $sourceStream=$resource.Open();$entryStream=$entry.Open()
        try{$sourceStream.CopyTo($entryStream)}finally{$sourceStream.Dispose();$entryStream.Dispose()}
    }
    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive,(Join-Path $dex 'classes.dex'),'classes.dex',[IO.Compression.CompressionLevel]::NoCompression) | Out-Null
    foreach($assetFile in Get-ChildItem -LiteralPath $assetRoot -Recurse -File){
        $entryName='assets/'+$assetFile.FullName.Substring($assetRoot.Length+1).Replace([IO.Path]::DirectorySeparatorChar,[char]'/')
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive,$assetFile.FullName,$entryName,[IO.Compression.CompressionLevel]::Optimal) | Out-Null
    }
}finally{$archive.Dispose();$outputStream.Dispose();$inputArchive.Dispose()}
$check=[IO.Compression.ZipFile]::OpenRead($unsigned)
try {
    foreach($required in @('classes.dex','resources.arsc','assets/ui/index.html','assets/ui/app.js','assets/ui/style.css','assets/ui/THIRD_PARTY_NOTICES.txt')){
        if(-not $check.GetEntry($required)){throw "Recurso ausente no APK: $required"}
    }
}finally{$check.Dispose()}
# Regression guard: this writer uses seekable streams and must finalize sizes in
# local headers. A retained data-descriptor flag without its descriptor crashes ART.
$reader=[IO.BinaryReader]::new([IO.File]::OpenRead($unsigned))
try {
    while($reader.ReadUInt32() -eq 0x04034b50){
        $null=$reader.ReadUInt16();$flags=$reader.ReadUInt16()
        $null=$reader.ReadUInt16();$null=$reader.ReadUInt32();$null=$reader.ReadUInt32()
        $compressed=$reader.ReadUInt32();$null=$reader.ReadUInt32()
        $nameLength=$reader.ReadUInt16();$extraLength=$reader.ReadUInt16()
        $name=[Text.Encoding]::UTF8.GetString($reader.ReadBytes($nameLength))
        if(($flags -band 8) -ne 0){throw "Cabeçalho ZIP com descritor inesperado: $name"}
        $null=$reader.BaseStream.Seek($extraLength+$compressed,[IO.SeekOrigin]::Current)
    }
}finally{$reader.Dispose()}
$aligned=Join-Path $BuildDirectory 'aligned.apk'
Invoke-BuildTool (Join-Path $buildTools 'zipalign.exe') @('-f','4',$unsigned,$aligned)
# Development-only key, generated locally. Use your own release signing key for distribution.
$key=Join-Path $BuildDirectory 'netlab-development.p12'
if(-not(Test-Path -LiteralPath $key)){
    Invoke-BuildTool (Join-Path $JdkRoot 'bin\keytool.exe') @('-genkeypair','-keystore',$key,'-storepass','android','-keypass','android','-alias','netlab-development','-dname','CN=NetLab Development','-keyalg','RSA','-keysize','2048','-validity','3650')
}
Invoke-BuildTool (Join-Path $buildTools 'apksigner.bat') @('sign','--v4-signing-enabled','false','--ks',$key,'--ks-pass','pass:android','--out',$OutputApk,$aligned)
Invoke-BuildTool (Join-Path $buildTools 'apksigner.bat') @('verify','--verbose',$OutputApk)
Write-Output "APK de desenvolvimento gerado: $OutputApk"
