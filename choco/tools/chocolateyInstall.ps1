$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$parameters = Get-PackageParameters

$silentArgs = '/S'
if ($parameters['Room']) {
  $silentArgs += " /ROOM=$($parameters['Room'])"
} else {
  Write-Warning "No room given, so the installed one is kept. Pick one with --params `"'/Room:<name>'`"."
}

Install-ChocolateyInstallPackage `
  -PackageName $env:ChocolateyPackageName `
  -FileType 'exe' `
  -File (Join-Path $toolsDir 'roomware-setup.exe') `
  -SilentArgs $silentArgs `
  -ValidExitCodes @(0)
