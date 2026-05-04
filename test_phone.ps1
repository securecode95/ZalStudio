$shell = New-Object -ComObject Shell.Application
$root = $shell.Namespace(0x11)

Write-Output "=== Alla enheter under 'Den har datorn' ==="
foreach ($device in $root.Items()) {
    Write-Output ("Enhet: " + $device.Name + " | Type: " + $device.Type)
    try {
        $folder = $device.GetFolder()
        Write-Output ("  -> Har GetFolder: JA")
        Write-Output ("  -> Antal barn: " + $folder.Items().Count)
        foreach ($child in $folder.Items()) {
            Write-Output ("    -> " + $child.Name + " (IsFolder=" + $child.IsFolder + ")")
        }
    } catch {
        Write-Output ("  -> GetFolder FEL: " + $_.Exception.Message)
    }
}

Write-Output ""
Write-Output "=== Sok efter telefoner med DCIM/Pictures ==="
foreach ($device in $root.Items()) {
    $devName = $device.Name
    try {
        $folder = $device.GetFolder()
        $hasPhoneFolders = $false
        foreach ($item in $folder.Items()) {
            if ($item.IsFolder -and ($item.Name -eq 'DCIM' -or $item.Name -eq 'Pictures' -or $item.Name -eq 'Camera')) {
                $hasPhoneFolders = $true
                break
            }
        }
        if ($hasPhoneFolders) {
            Write-Output ("HITTAD TELEFON: " + $devName)
        }
    } catch { }
}
