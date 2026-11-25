; Steps the Windows installer runs around installing Sompio.

!macro NSIS_HOOK_PREINSTALL
  ; The browser extension's helper may be running for an open browser, which stops it from being
  ; replaced. Stop it first: the browser starts it again the next time the extension needs it.
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM sompio-browser.exe'
  Pop $0
  Sleep 300
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM sompio-browser.exe'
  Pop $0
  Sleep 300
!macroend
