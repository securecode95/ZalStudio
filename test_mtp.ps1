$shell = New-Object -ComObject Shell.Application

Write-Output "=== Test 1: Namespace(0x11) ==="
$root1 = $shell.Namespace(0x11)
Write-Output ("Items count: " + $root1.Items().Count)
foreach ($item in $root1.Items()) {
    Write-Output ("  Device: " + $item.Name + " | Type: " + $item.Type + " | IsFolder: " + $item.IsFolder)
    try {
        $folder = $item.GetFolder()
        Write-Output ("    -> GetFolder OK, children: " + $folder.Items().Count)
    } catch {
        Write-Output ("    -> GetFolder FAILED: " + $_.Exception.Message)
    }
}

Write-Output ""
Write-Output "=== Test 2: Namespace('shell:MyComputerFolder') ==="
$root2 = $shell.Namespace("shell:MyComputerFolder")
Write-Output ("Items count: " + $root2.Items().Count)
foreach ($item in $root2.Items()) {
    Write-Output ("  Device: " + $item.Name + " | Type: " + $item.Type + " | IsFolder: " + $item.IsFolder)
    try {
        $folder = $item.GetFolder()
        Write-Output ("    -> GetFolder OK, children: " + $folder.Items().Count)
    } catch {
        Write-Output ("    -> GetFolder FAILED: " + $_.Exception.Message)
    }
}

Write-Output ""
Write-Output "=== Test 3: Namespace('::{20D04FE0-3AEA-1069-A2D8-08002B30309D}') ==="
$root3 = $shell.Namespace("::{20D04FE0-3AEA-1069-A2D8-08002B30309D}")
if ($root3) {
    Write-Output ("Items count: " + $root3.Items().Count)
    foreach ($item in $root3.Items()) {
        Write-Output ("  Device: " + $item.Name + " | Type: " + $item.Type + " | IsFolder: " + $item.IsFolder)
        try {
            $folder = $item.GetFolder()
            Write-Output ("    -> GetFolder OK, children: " + $folder.Items().Count)
        } catch {
            Write-Output ("    -> GetFolder FAILED: " + $_.Exception.Message)
        }
    }
} else {
    Write-Output "ROOT3 IS NULL"
}

Write-Output ""
Write-Output "=== Test 4: Direct Samsung test ==="
$found = $false
foreach ($item in $root1.Items()) {
    if ($item.Name -like "*SM-*" -or $item.Name -like "*Galaxy*" -or $item.Name -like "*Phone*" -or $item.Name -like "*Android*") {
        $found = $true
        Write-Output ("FOUND by name: " + $item.Name)
    }
}
if (-not $found) {
    Write-Output "No device matched SM-/Galaxy/Phone/Android"
}

Write-Output ""
Write-Output "=== Test 5: Check for DCIM folder ==="
$foundPhone = $false
foreach ($item in $root1.Items()) {
    try {
        $folder = $item.GetFolder()
        foreach ($child in $folder.Items()) {
            if ($child.IsFolder -and ($child.Name -eq 'DCIM' -or $child.Name -eq 'Pictures')) {
                $foundPhone = $true
                Write-Output ("FOUND by DCIM/Pictures in: " + $item.Name)
            }
        }
    } catch { }
}
if (-not $foundPhone) {
    Write-Output "No device had DCIM/Pictures folder"
}
