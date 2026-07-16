; Applied Yet installer presentation layer.
; This keeps Tauri's stock NSIS install/update/uninstall behavior and only
; enhances the native welcome page with an offline, timer-driven carousel.

!include nsDialogs.nsh
!include LogicLib.nsh

Var CarouselImage
Var CarouselFrame
Var CarouselBitmap

!define MUI_PAGE_CUSTOMFUNCTION_SHOW AppliedYetWelcomeShow
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE AppliedYetWelcomeLeave

Function AppliedYetWelcomeShow
  ; MUI's welcome-page bitmap is the native control with dialog item id 1200.
  GetDlgItem $CarouselImage $HWNDPARENT 1200
  StrCpy $CarouselFrame 1

  File /oname=$PLUGINSDIR\applied-yet-sidebar-1.bmp "${__FILEDIR__}\installer\sidebar-1.bmp"
  File /oname=$PLUGINSDIR\applied-yet-sidebar-2.bmp "${__FILEDIR__}\installer\sidebar-2.bmp"
  File /oname=$PLUGINSDIR\applied-yet-sidebar-3.bmp "${__FILEDIR__}\installer\sidebar-3.bmp"

  nsDialogs::CreateTimer AppliedYetAdvanceCarousel 3200
FunctionEnd

Function AppliedYetAdvanceCarousel
  ${If} $CarouselBitmap != ""
    ${NSD_FreeImage} $CarouselBitmap
    StrCpy $CarouselBitmap ""
  ${EndIf}

  ${If} $CarouselFrame = 1
    ${NSD_SetImage} $CarouselImage "$PLUGINSDIR\applied-yet-sidebar-2.bmp" $CarouselBitmap
    StrCpy $CarouselFrame 2
  ${ElseIf} $CarouselFrame = 2
    ${NSD_SetImage} $CarouselImage "$PLUGINSDIR\applied-yet-sidebar-3.bmp" $CarouselBitmap
    StrCpy $CarouselFrame 3
  ${Else}
    ${NSD_SetImage} $CarouselImage "$PLUGINSDIR\applied-yet-sidebar-1.bmp" $CarouselBitmap
    StrCpy $CarouselFrame 1
  ${EndIf}
FunctionEnd

Function AppliedYetWelcomeLeave
  nsDialogs::KillTimer AppliedYetAdvanceCarousel
  ${If} $CarouselBitmap != ""
    ${NSD_FreeImage} $CarouselBitmap
    StrCpy $CarouselBitmap ""
  ${EndIf}
FunctionEnd
