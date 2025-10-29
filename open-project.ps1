if (Get-Process -Name "Docker Desktop" -ErrorAction SilentlyContinue) {
	# 既にDocker Desktopが起動している場合はスキップ
	Write-Host "Docker Desktop is already running. Skipping startup."
}
else {
	Write-Host "Starting Docker Desktop..."
	Start-Process "C:\Program Files\Docker\Docker\Docker Desktop.exe"
	Start-Sleep 5
	Write-Host "Docker Desktop started."
}
Write-Host "`n"

# Wiresharkの起動チェック
if (Get-Process -Name "Wireshark" -ErrorAction SilentlyContinue) {
	# 既にWiresharkが起動している場合はスキップ
	Write-Host "Wireshark is already running. Skipping startup."
}
else {
	# Wiresharkを起動
	Write-Host "Starting Wireshark..."
	Start-Process pwsh -ArgumentList "-Command", "Start-Process 'C:\Program Files\Wireshark\Wireshark.exe'" -WindowStyle Hidden
	Start-Sleep 2
	Write-Host "Wireshark started."
}
Write-Host "`n"

Write-Host "Opening VS Code in devcontainer..."
Start-Process pwsh -ArgumentList "-Command", 'devcontainer open .' -WindowStyle Hidden
Start-Sleep 1
Write-Host "VS Code opened in devcontainer.`n"

Write-Host "Complete open project."
Start-Sleep 1
