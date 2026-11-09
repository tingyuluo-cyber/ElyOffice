#Requires -Version 7.4
[CmdletBinding()]
param(
    [string]$UpstreamInstaller,
    [string]$TargetDirectory,
    [string]$OutputDirectory,
    [string]$SevenZip = (Join-Path $env:ProgramFiles '7-Zip\7z.exe')
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

if (-not $IsWindows) { throw 'This recipe builds the Windows x64 package.' }
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$manifest = Join-Path $root 'Cargo.toml'
if (-not $TargetDirectory) { $TargetDirectory = Join-Path $root 'target' }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $root 'dist' }
$target = [IO.Path]::GetFullPath($TargetDirectory, (Get-Location).ProviderPath)
$out = [IO.Path]::GetFullPath($OutputDirectory, (Get-Location).ProviderPath)
if ((rustc -vV) -notcontains 'host: x86_64-pc-windows-msvc') {
    throw 'Use the x86_64-pc-windows-msvc Rust toolchain.'
}
Get-Command $SevenZip | Out-Null

$meta = cargo metadata --manifest-path $manifest --locked --no-deps --format-version 1 | ConvertFrom-Json
$version = ($meta.packages | Where-Object name -eq 'ely-mark').version
$name = "ReimuMark-$version-windows-x64"
$zip = Join-Path $out "$name.zip"
if (Test-Path -LiteralPath $zip) { throw "Package already exists at $zip" }

$work = Join-Path $target ("reimu-package-" + [Guid]::NewGuid().ToString('N'))
$bundle = Join-Path $work $name
New-Item -ItemType Directory -Path $bundle | Out-Null
try {
    if (-not $UpstreamInstaller) {
        $UpstreamInstaller = Join-Path $work 'ElyMark-upstream.exe'
        $url = 'https://github.com/ELY-Open-Source/ElyOffice/releases/download/v0.1.15/ElyMark-0.1.15-windows-x86_64-setup.exe'
        Invoke-WebRequest -Uri $url -OutFile $UpstreamInstaller -ProgressAction SilentlyContinue
    }
    $expected = '8E840C6DD4076EB6A10500379BF5A89B7817453608611AAC37EE21BA73AAB72E'
    if ((Get-FileHash -LiteralPath $UpstreamInstaller -Algorithm SHA256).Hash -ne $expected) {
        throw 'Upstream ElyMark 0.1.15 installer failed SHA-256 verification.'
    }
    & $SevenZip x -y "-o$bundle" $UpstreamInstaller ely-pdf-worker.exe 'licenses\*' | Out-Host
    foreach ($required in @('ely-pdf-worker.exe', 'licenses\LICENSE', 'licenses\MuPDF\COPYING', 'licenses\rust-crates.txt')) {
        if (-not (Test-Path -LiteralPath (Join-Path $bundle $required) -PathType Leaf)) {
            throw "Upstream archive is missing $required"
        }
    }

    cargo build --manifest-path $manifest --release --locked --target-dir $target -p ely-mark --bin ely-mark --features standalone
    Copy-Item -LiteralPath (Join-Path $target 'release\ely-mark.exe') -Destination (Join-Path $bundle 'ReimuMark.exe')
    Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $bundle
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $bundle

    New-Item -ItemType Directory -Path $out -Force | Out-Null
    $archive = Join-Path $work "$name.zip"
    Compress-Archive -LiteralPath $bundle -DestinationPath $archive -CompressionLevel Optimal
    Move-Item -LiteralPath $archive -Destination $zip
    Get-Item -LiteralPath $zip
} finally {
    $scratch = Get-Item -LiteralPath $work -ErrorAction Stop
    if ($scratch.FullName -ne $work -or ($scratch.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Unexpected package staging directory $($scratch.FullName)"
    }
    Remove-Item -LiteralPath $scratch.FullName -Recurse -Force
}
