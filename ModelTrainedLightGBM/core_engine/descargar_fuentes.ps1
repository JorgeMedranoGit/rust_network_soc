$baseUrl = "https://raw.githubusercontent.com/taylor/fonts/master/liberation/"
$fonts = @("Regular", "Bold", "Italic", "BoldItalic")

foreach ($style in $fonts) {
    $fileName = "LiberationSans-$style.ttf"
    $url = "$baseUrl$fileName"
    $dest = "fonts/$fileName"
    if (-not (Test-Path $dest)) {
        Write-Host "Descargando $fileName..."
        Invoke-WebRequest -Uri $url -OutFile $dest
    }
}
