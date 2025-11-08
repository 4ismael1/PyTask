using System;
using System.Collections.Concurrent;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace PyTaskAvalonia.Services;

public class GlobalHotkeyService : IDisposable
{
    private IntPtr _hookID = IntPtr.Zero;
    private WinAPI.LowLevelKeyboardProc? _proc;
    private readonly ConcurrentDictionary<string, Action> _hotkeys = new(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, byte> _pressedKeys = new(StringComparer.Ordinal);
    private readonly object _hookSync = new();
    
    public void RegisterHotkey(string key, Action callback)
    {
        if (string.IsNullOrWhiteSpace(key))
        {
            throw new ArgumentException("The hotkey cannot be empty.", nameof(key));
        }

        var normalizedKey = key.Trim().ToUpperInvariant();
        _hotkeys.AddOrUpdate(normalizedKey, callback, (_, _) => callback);
        
        if (_hookID == IntPtr.Zero)
        {
            lock (_hookSync)
            {
                if (_hookID == IntPtr.Zero)
                {
                    _proc = HookCallback;
                    _hookID = SetHook(_proc);
                }
            }
        }
    }
    
    public void UnregisterHotkey(string key)
    {
        if (string.IsNullOrWhiteSpace(key))
        {
            return;
        }

        var normalizedKey = key.Trim().ToUpperInvariant();
        _hotkeys.TryRemove(normalizedKey, out _);

        if (_hotkeys.IsEmpty)
        {
            ReleaseHook();
        }
    }
    
    public void ClearHotkeys()
    {
        _hotkeys.Clear();
        ReleaseHook();
    }
    
    private IntPtr SetHook(WinAPI.LowLevelKeyboardProc proc)
    {
        using var curProcess = Process.GetCurrentProcess();
        using var curModule = curProcess.MainModule;
        var moduleName = curModule?.ModuleName;
        var moduleHandle = moduleName != null ? WinAPI.GetModuleHandle(moduleName) : IntPtr.Zero;

        if (moduleHandle == IntPtr.Zero)
        {
            throw new InvalidOperationException("Unable to obtain a module handle for the current process.");
        }

        var hookId = WinAPI.SetWindowsHookEx(WinAPI.WH_KEYBOARD_LL, proc, moduleHandle, 0);
        if (hookId == IntPtr.Zero)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }

        return hookId;
    }
    
    private IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            var vkCode = Marshal.ReadInt32(lParam);
            var keyName = GetKeyName((uint)vkCode);
            var hookStruct = Marshal.PtrToStructure<WinAPI.KBDLLHOOKSTRUCT>(lParam);

            if (hookStruct.dwExtraInfo == InputSignature.Value)
            {
                return WinAPI.CallNextHookEx(_hookID, nCode, wParam, lParam);
            }
            
            if (wParam == (IntPtr)WinAPI.WM_KEYDOWN)
            {
                if (_pressedKeys.TryAdd(keyName, 0))
                {
                    // Verificar si hay un hotkey registrado para esta tecla
                    if (_hotkeys.TryGetValue(keyName, out var callback))
                    {
                        // Ejecutar en un thread separado para no bloquear el hook
                        Task.Run(() =>
                        {
                            try
                            {
                                callback();
                            }
                            catch (Exception ex)
                            {
                                Debug.WriteLine($"Global hotkey callback failed: {ex}");
                            }
                        });
                    }
                }
            }
            else if (wParam == (IntPtr)WinAPI.WM_KEYUP)
            {
                _pressedKeys.TryRemove(keyName, out _);
            }
        }
        
        return WinAPI.CallNextHookEx(_hookID, nCode, wParam, lParam);
    }
    
    private string GetKeyName(uint vkCode)
    {
        return vkCode switch
        {
            >= 0x70 and <= 0x7B => $"F{vkCode - 0x6F}", // F1-F12
            >= 0x41 and <= 0x5A => ((char)vkCode).ToString(), // A-Z
            >= 0x30 and <= 0x39 => ((char)vkCode).ToString(), // 0-9
            _ => vkCode.ToString()
        };
    }
    
    public void Dispose()
    {
        ReleaseHook();
        _hotkeys.Clear();
        _pressedKeys.Clear();
    }

    private void ReleaseHook()
    {
        lock (_hookSync)
        {
            if (_hookID != IntPtr.Zero)
            {
                WinAPI.UnhookWindowsHookEx(_hookID);
                _hookID = IntPtr.Zero;
                _proc = null;
            }
        }

        _pressedKeys.Clear();
    }
}
