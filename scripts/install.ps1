param(
    [string]$Tag
)

function ErrorExit ($msg) {
    Write-Host "erro: $msg" -ForegroundColor Red
    exit 1
}

# Choose release

$repo = 'gabrielbrunop/tenda'
if (-not $Tag) {
    try   { $Tag = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest").tag_name }
    catch { ErrorExit 'Falha ao buscar a tag da última versão no GitHub' }
}
Write-Host "Instalando Tenda $Tag"

# Asset and URLs

$asset    = 'tenda-x86_64-pc-windows-msvc.zip'
$download = "https://github.com/$repo/releases/download/$Tag/$asset"

# Prepare installation directories 

$installRoot = if ($Env:TENDA_INSTALL) { $Env:TENDA_INSTALL }
               else { Join-Path ([Environment]::GetFolderPath('UserProfile')) '.tenda' }
$binDir = Join-Path $installRoot 'bin'
New-Item $binDir -ItemType Directory -Force | Out-Null

$tmp = New-Item -ItemType Directory -Path ([IO.Path]::GetTempPath()) `
      -Name ([Guid]::NewGuid()) 

# Download & unzip

Write-Host "Baixando $download"
Invoke-WebRequest -Uri $download -OutFile "$tmp\$asset" -UseBasicParsing -ErrorAction Stop

Write-Host 'Extraindo arquivos'
Expand-Archive -LiteralPath "$tmp\$asset" -DestinationPath $tmp -Force

Move-Item -Force "$tmp\tenda.exe" (Join-Path $binDir 'tenda.exe') -ErrorAction Stop
Remove-Item -Recurse -Force $tmp

# Make PATH changes idempotent

$pathLine = $binDir

$current = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($null -eq $current) { $current = '' }

if (-not ($current.Split(';') -contains $pathLine)) {
    $newPath = ($current.TrimEnd(';') + ';' + $pathLine).TrimStart(';')
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Host "Adicionado $binDir ao PATH do usuário"
}

if (-not $Env:TENDA_INSTALL) {
    [Environment]::SetEnvironmentVariable('TENDA_INSTALL', $installRoot, 'User')
    Write-Host "Set TENDA_INSTALL=$installRoot"
}

# Update current session and finish

$Env:PATH          = "$binDir;$Env:PATH"
$Env:TENDA_INSTALL = $installRoot

Write-Host ""
Write-Host "Tenda instalada com sucesso -> $(Join-Path $binDir 'tenda.exe')" -ForegroundColor Green

# Show usage hint

$hint = "`$env:Path = `"$binDir;`$env:Path`""
Write-Host "`nPara usar a Tenda imediatamente, execute:" -ForegroundColor Yellow
Write-Host "  $hint" -ForegroundColor Yellow
Write-Host "`nExecute 'tenda --ajuda' para saber mais!" -ForegroundColor Yellow
