Option Explicit

Dim files, shell, env, logPath, command, result, i, argument, variable, arguments
Set files = CreateObject("Scripting.FileSystemObject")
Set shell = CreateObject("WScript.Shell")
Set env = shell.Environment("PROCESS")
logPath = files.BuildPath(files.GetSpecialFolder(2), "ElyMark-" & files.GetBaseName(files.GetTempName()) & ".log")

env("ELY_REIMU_NO_PAUSE") = "1"
env("ELY_REIMU_BATCH") = files.BuildPath(files.GetParentFolderName(WScript.ScriptFullName), "start-reimu-mark.bat")
env("ELY_REIMU_LAUNCH_LOG") = logPath
arguments = ""
For i = 0 To WScript.Arguments.Count - 1
    argument = WScript.Arguments(i)
    If InStr(argument, Chr(34)) > 0 Then
        MsgBox "ElyMark cannot open an argument containing a quote.", vbExclamation, "ElyMark"
        WScript.Quit 2
    End If
    If Right(argument, 1) = "\" Then argument = argument & "."
    variable = "ELY_REIMU_ARG_" & i
    env(variable) = argument
    arguments = arguments & " " & Quoted("!" & variable & "!")
Next

command = "cmd.exe /d /v:on /s /c " & Quoted(Quoted("!ELY_REIMU_BATCH!") & arguments & " > " & Quoted("!ELY_REIMU_LAUNCH_LOG!") & " 2>&1")
result = shell.Run(command, 0, True)
If result <> 0 Then
    MsgBox "ElyMark could not start (exit code " & result & ")." & vbCrLf & logPath, vbExclamation, "ElyMark"
Else
    files.DeleteFile logPath
End If
WScript.Quit result

Function Quoted(value)
    Quoted = Chr(34) & value & Chr(34)
End Function
