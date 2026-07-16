param(
  [string]$ImagesDir = "$(Join-Path $PSScriptRoot '..\..\..\..\docs\images')"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

function New-RoundedPath {
  param([System.Drawing.RectangleF]$Rect, [float]$Radius)
  $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
  $diameter = $Radius * 2
  $path.AddArc($Rect.X, $Rect.Y, $diameter, $diameter, 180, 90)
  $path.AddArc($Rect.Right - $diameter, $Rect.Y, $diameter, $diameter, 270, 90)
  $path.AddArc($Rect.Right - $diameter, $Rect.Bottom - $diameter, $diameter, $diameter, 0, 90)
  $path.AddArc($Rect.X, $Rect.Bottom - $diameter, $diameter, $diameter, 90, 90)
  $path.CloseFigure()
  return $path
}

function New-SidebarSlide {
  param([string]$Source, [string]$Title, [string]$Subtitle, [string]$Index, [string]$Output)

  $bitmap = [System.Drawing.Bitmap]::new(164, 314)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.SmoothingMode = "AntiAlias"
  $graphics.InterpolationMode = "HighQualityBicubic"
  $graphics.TextRenderingHint = "ClearTypeGridFit"
  $background = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    [System.Drawing.Rectangle]::new(0, 0, 164, 314),
    [System.Drawing.Color]::FromArgb(20, 47, 107, 255),
    [System.Drawing.Color]::FromArgb(255, 235, 242, 255),
    90
  )
  $graphics.FillRectangle($background, 0, 0, 164, 314)

  $brandFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 10, [System.Drawing.FontStyle]::Bold)
  $titleFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 13, [System.Drawing.FontStyle]::Bold)
  $bodyFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 8)
  $smallFont = [System.Drawing.Font]::new("Microsoft YaHei UI", 7, [System.Drawing.FontStyle]::Bold)
  $white = [System.Drawing.Brushes]::White
  $blue = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(47, 107, 255))
  $ink = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(23, 32, 51))
  $muted = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(82, 96, 122))

  $graphics.FillEllipse($blue, 14, 14, 28, 28)
  $graphics.DrawString("投", $brandFont, $white, 19, 19)
  $graphics.DrawString("投了吗", $brandFont, $ink, 48, 20)
  $graphics.DrawString($Title, $titleFont, $ink, 14, 58)
  $graphics.DrawString($Subtitle, $bodyFont, $muted, [System.Drawing.RectangleF]::new(14, 88, 136, 38))

  $card = New-RoundedPath ([System.Drawing.RectangleF]::new(12, 128, 140, 122)) 10
  $graphics.FillPath([System.Drawing.Brushes]::White, $card)
  $image = [System.Drawing.Image]::FromFile($Source)
  $clip = New-RoundedPath ([System.Drawing.RectangleF]::new(17, 133, 130, 112)) 7
  $graphics.SetClip($clip)
  $scale = [Math]::Max(130 / $image.Width, 112 / $image.Height)
  $width = $image.Width * $scale
  $height = $image.Height * $scale
  $graphics.DrawImage($image, 17 + (130 - $width) / 2, 133 + (112 - $height) / 2, $width, $height)
  $graphics.ResetClip()

  $graphics.DrawString("本地优先 · 隐私安全", $smallFont, $blue, 14, 266)
  $graphics.DrawString("$Index  /  03", $smallFont, $muted, 112, 266)
  for ($i = 0; $i -lt 3; $i++) {
    $brush = if (($i + 1).ToString("00") -eq $Index) { $blue } else { [System.Drawing.Brushes]::White }
    $graphics.FillEllipse($brush, 14 + ($i * 12), 292, 5, 5)
  }

  $bitmap.Save($Output, [System.Drawing.Imaging.ImageFormat]::Bmp)
  $image.Dispose(); $clip.Dispose(); $card.Dispose(); $background.Dispose()
  $brandFont.Dispose(); $titleFont.Dispose(); $bodyFont.Dispose(); $smallFont.Dispose()
  $blue.Dispose(); $ink.Dispose(); $muted.Dispose(); $graphics.Dispose(); $bitmap.Dispose()
}

New-SidebarSlide (Join-Path $ImagesDir "投递流程.png") "投递全流程" "从岗位收藏到 Offer，所有进展一目了然。" "01" (Join-Path $PSScriptRoot "sidebar-1.bmp")
New-SidebarSlide (Join-Path $ImagesDir "招聘邮件.png") "邮件自动归档" "识别招聘邮件，把关键节点放回你的工作台。" "02" (Join-Path $PSScriptRoot "sidebar-2.bmp")
New-SidebarSlide (Join-Path $ImagesDir "模拟面试.png") "面试准备就绪" "题库、模拟面试与复盘，形成完整准备闭环。" "03" (Join-Path $PSScriptRoot "sidebar-3.bmp")

$header = [System.Drawing.Bitmap]::new(150, 57)
$g = [System.Drawing.Graphics]::FromImage($header)
$g.SmoothingMode = "AntiAlias"
$g.Clear([System.Drawing.Color]::White)
$accent = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(47, 107, 255))
$g.FillEllipse($accent, 94, 12, 32, 32)
$font = [System.Drawing.Font]::new("Microsoft YaHei UI", 13, [System.Drawing.FontStyle]::Bold)
$g.DrawString("投", $font, [System.Drawing.Brushes]::White, 100, 16)
$g.DrawString("Applied Yet?", [System.Drawing.Font]::new("Segoe UI", 7), $accent, 71, 45)
$header.Save((Join-Path $PSScriptRoot "header.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$font.Dispose(); $accent.Dispose(); $g.Dispose(); $header.Dispose()
