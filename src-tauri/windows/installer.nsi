!macro NSIS_HOOK_POSTINSTALL
  ; El helper ya ha sido copiado por Tauri.
  ; Ahora lo instalamos como servicio.

  MessageBox MB_OK "POSTINSTALL ejecutado. Instalando LocalDomainsHelper..."

  nsExec::ExecToLog 'sc.exe stop LocalDomainsHelper'
  nsExec::ExecToLog 'sc.exe delete LocalDomainsHelper'

  Sleep 1000

  nsExec::ExecToLog 'sc.exe create LocalDomainsHelper binPath= "$INSTDIR\bin\hosts-helper.exe" start= auto DisplayName= "Local Domains Helper"'

  Sleep 1000

  nsExec::ExecToLog 'sc.exe start LocalDomainsHelper'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'sc.exe stop LocalDomainsHelper'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  nsExec::ExecToLog 'sc.exe delete LocalDomainsHelper'
!macroend