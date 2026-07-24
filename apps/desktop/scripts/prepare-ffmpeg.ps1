$ErrorActionPreference = "Stop"

$ffmpegVersion = "8.0.1"
$archiveName = "ffmpeg-$ffmpegVersion-essentials_build.zip"
$archiveUrl = "https://www.gyan.dev/ffmpeg/builds/packages/$archiveName"
$expectedSha256 = "e2aaeaa0fdbc397d4794828086424d4aaa2102cef1fb6874f6ffd29c0b88b673"
$targetTriple = (& rustc --print host-tuple).Trim()

if ($targetTriple -ne "x86_64-pc-windows-msvc") {
  throw "当前只配置了 Windows x64 FFmpeg sidecar，检测到目标：$targetTriple"
}

$tauriDirectory = (Resolve-Path (Join-Path $PSScriptRoot "..\src-tauri")).Path
$cacheDirectory = Join-Path $tauriDirectory ".ffmpeg-cache"
$extractDirectory = Join-Path $cacheDirectory "ffmpeg-$ffmpegVersion-essentials"
$archivePath = Join-Path $cacheDirectory $archiveName
$downloadPath = "$archivePath.download"
$binaryDirectory = Join-Path $tauriDirectory "binaries"
$targetBinary = Join-Path $binaryDirectory "ffmpeg-$targetTriple.exe"
$targetLicense = Join-Path $binaryDirectory "ffmpeg-LICENSE.txt"
$targetReadme = Join-Path $binaryDirectory "ffmpeg-README.txt"

New-Item -ItemType Directory -Force -Path $cacheDirectory, $binaryDirectory | Out-Null

function Test-ArchiveHash([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    return $false
  }
  $stream = [System.IO.File]::OpenRead($Path)
  try {
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
      $hash = [System.BitConverter]::ToString($sha256.ComputeHash($stream)).Replace("-", "").ToLowerInvariant()
      return $hash -eq $expectedSha256
    }
    finally {
      $sha256.Dispose()
    }
  }
  finally {
    $stream.Dispose()
  }
}

if (-not (Test-ArchiveHash $archivePath)) {
  if (-not (Test-ArchiveHash $downloadPath)) {
    Write-Host "Downloading pinned FFmpeg $ffmpegVersion Essentials build..."
    Invoke-WebRequest -Uri $archiveUrl -OutFile $downloadPath
  }
  if (-not (Test-ArchiveHash $downloadPath)) {
    Remove-Item -LiteralPath $downloadPath -Force -ErrorAction SilentlyContinue
    throw "FFmpeg archive SHA-256 verification failed."
  }
  Move-Item -LiteralPath $downloadPath -Destination $archivePath -Force
}

$cachedBinary = Get-ChildItem -LiteralPath $extractDirectory -Filter "ffmpeg.exe" -File -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $cachedBinary) {
  Write-Host "Extracting FFmpeg archive..."
  New-Item -ItemType Directory -Force -Path $extractDirectory | Out-Null
  Expand-Archive -LiteralPath $archivePath -DestinationPath $extractDirectory -Force
  $cachedBinary = Get-ChildItem -LiteralPath $extractDirectory -Filter "ffmpeg.exe" -File -Recurse | Select-Object -First 1
}
if (-not $cachedBinary) {
  throw "The verified FFmpeg archive does not contain ffmpeg.exe."
}

Copy-Item -LiteralPath $cachedBinary.FullName -Destination $targetBinary -Force

$packageRoot = $cachedBinary.Directory.Parent.FullName
$license = Get-ChildItem -LiteralPath $packageRoot -File | Where-Object { $_.Name -match "^LICENSE" } | Select-Object -First 1
$readme = Get-ChildItem -LiteralPath $packageRoot -File | Where-Object { $_.Name -match "^README" } | Select-Object -First 1
if (-not $license -or -not $readme) {
  throw "FFmpeg package license or README metadata is missing."
}
Copy-Item -LiteralPath $license.FullName -Destination $targetLicense -Force
Copy-Item -LiteralPath $readme.FullName -Destination $targetReadme -Force

Write-Host "Prepared FFmpeg sidecar: $targetBinary"
$binaryStream = [System.IO.File]::OpenRead($targetBinary)
try {
  $binaryHasher = [System.Security.Cryptography.SHA256]::Create()
  try {
    $binaryHash = [System.BitConverter]::ToString($binaryHasher.ComputeHash($binaryStream)).Replace("-", "").ToLowerInvariant()
  }
  finally {
    $binaryHasher.Dispose()
  }
}
finally {
  $binaryStream.Dispose()
}
Write-Host "SHA-256: $binaryHash"
