using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using PyTaskAvalonia.Models;

namespace PyTaskAvalonia.Services;

public class MacroRecorder : IDisposable
{
    private readonly List<MacroEvent> _events = new();
    private bool _recording;
    private Stopwatch _stopwatch = new();
    private IntPtr _mouseHookID = IntPtr.Zero;
    private IntPtr _keyboardHookID = IntPtr.Zero;
    private WinAPI.LowLevelMouseProc? _mouseProc;
    private WinAPI.LowLevelKeyboardProc? _keyboardProc;
    private double _lastMoveTimestamp = double.NegativeInfinity;
    private const double MoveEventIntervalMs = 50;
    
    public bool IsRecording => _recording;
    
    public void StartRecording()
    {
        if (_recording) return;
        
        _events.Clear();
        _recording = true;
        _stopwatch = Stopwatch.StartNew();
        _lastMoveTimestamp = double.NegativeInfinity;
        
        // Instalar hooks
        _mouseProc = MouseHookCallback;
        _keyboardProc = KeyboardHookCallback;
        
        _mouseHookID = SetMouseHook(_mouseProc);
        _keyboardHookID = SetKeyboardHook(_keyboardProc);
    }
    
    public List<MacroEvent> StopRecording()
    {
        if (!_recording) return new List<MacroEvent>();
        
        _recording = false;
        _stopwatch.Stop();
        
        // Desinstalar hooks
        if (_mouseHookID != IntPtr.Zero)
        {
            WinAPI.UnhookWindowsHookEx(_mouseHookID);
            _mouseHookID = IntPtr.Zero;
        }
        
        if (_keyboardHookID != IntPtr.Zero)
        {
            WinAPI.UnhookWindowsHookEx(_keyboardHookID);
            _keyboardHookID = IntPtr.Zero;
        }
        
        return new List<MacroEvent>(_events);
    }
    
    private IntPtr SetMouseHook(WinAPI.LowLevelMouseProc proc)
    {
        using var curProcess = Process.GetCurrentProcess();
        using var curModule = curProcess.MainModule;
        var moduleName = curModule?.ModuleName;
        var moduleHandle = moduleName != null ? WinAPI.GetModuleHandle(moduleName) : IntPtr.Zero;

        if (moduleHandle == IntPtr.Zero)
        {
            throw new InvalidOperationException("Unable to obtain module handle for mouse hook installation.");
        }

        var hookId = WinAPI.SetWindowsHookEx(WinAPI.WH_MOUSE_LL, proc, moduleHandle, 0);
        if (hookId == IntPtr.Zero)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }

        return hookId;
    }
    
    private IntPtr SetKeyboardHook(WinAPI.LowLevelKeyboardProc proc)
    {
        using var curProcess = Process.GetCurrentProcess();
        using var curModule = curProcess.MainModule;
        var moduleName = curModule?.ModuleName;
        var moduleHandle = moduleName != null ? WinAPI.GetModuleHandle(moduleName) : IntPtr.Zero;

        if (moduleHandle == IntPtr.Zero)
        {
            throw new InvalidOperationException("Unable to obtain module handle for keyboard hook installation.");
        }

        var hookId = WinAPI.SetWindowsHookEx(WinAPI.WH_KEYBOARD_LL, proc, moduleHandle, 0);
        if (hookId == IntPtr.Zero)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }

        return hookId;
    }
    
    private IntPtr MouseHookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0 && _recording)
        {
            var hookStruct = Marshal.PtrToStructure<WinAPI.MSLLHOOKSTRUCT>(lParam);
            var timestamp = _stopwatch.Elapsed.TotalSeconds;
            
            var msg = (int)wParam;
            
            switch (msg)
            {
                case WinAPI.WM_MOUSEMOVE:
                    var elapsedMs = _stopwatch.Elapsed.TotalMilliseconds;
                    if (elapsedMs - _lastMoveTimestamp >= MoveEventIntervalMs)
                    {
                        _events.Add(new MacroEvent
                        {
                            Type = "mouse_move",
                            X = hookStruct.pt.X,
                            Y = hookStruct.pt.Y,
                            Timestamp = timestamp
                        });
                        _lastMoveTimestamp = elapsedMs;
                    }
                    break;
                
                case WinAPI.WM_LBUTTONDOWN:
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_click",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Button = "left",
                        Pressed = true,
                        Timestamp = timestamp
                    });
                    break;
                
                case WinAPI.WM_LBUTTONUP:
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_click",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Button = "left",
                        Pressed = false,
                        Timestamp = timestamp
                    });
                    break;
                
                case WinAPI.WM_RBUTTONDOWN:
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_click",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Button = "right",
                        Pressed = true,
                        Timestamp = timestamp
                    });
                    break;
                
                case WinAPI.WM_RBUTTONUP:
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_click",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Button = "right",
                        Pressed = false,
                        Timestamp = timestamp
                    });
                    break;
                
                case WinAPI.WM_MBUTTONDOWN:
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_click",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Button = "middle",
                        Pressed = true,
                        Timestamp = timestamp
                    });
                    break;
                
                case WinAPI.WM_MBUTTONUP:
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_click",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Button = "middle",
                        Pressed = false,
                        Timestamp = timestamp
                    });
                    break;
                
                case WinAPI.WM_MOUSEWHEEL:
                    var delta = (short)((hookStruct.mouseData >> 16) & 0xFFFF);
                    _events.Add(new MacroEvent
                    {
                        Type = "mouse_scroll",
                        X = hookStruct.pt.X,
                        Y = hookStruct.pt.Y,
                        Dx = 0,
                        Dy = delta / 120, // Normalizar
                        Timestamp = timestamp
                    });
                    break;
            }
        }
        
        return WinAPI.CallNextHookEx(_mouseHookID, nCode, wParam, lParam);
    }
    
    private IntPtr KeyboardHookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0 && _recording)
        {
            var hookStruct = Marshal.PtrToStructure<WinAPI.KBDLLHOOKSTRUCT>(lParam);
            var timestamp = _stopwatch.Elapsed.TotalSeconds;
            var vkCode = hookStruct.vkCode;
            
            var msg = (int)wParam;
            var isKeyDown = msg == WinAPI.WM_KEYDOWN || msg == WinAPI.WM_SYSKEYDOWN;
            
            _events.Add(new MacroEvent
            {
                Type = isKeyDown ? "key_press" : "key_release",
                VkCode = vkCode,
                Key = GetKeyName(vkCode),
                Timestamp = timestamp
            });
        }
        
        return WinAPI.CallNextHookEx(_keyboardHookID, nCode, wParam, lParam);
    }
    
    private string GetKeyName(uint vkCode)
    {
        // Mapeo básico de códigos virtuales
        return vkCode switch
        {
            0x20 => "space",
            0x0D => "enter",
            0x09 => "tab",
            0x08 => "backspace",
            0x1B => "esc",
            0x2E => "delete",
            0x10 => "shift",
            0x11 => "ctrl",
            0x12 => "alt",
            0x25 => "left",
            0x26 => "up",
            0x27 => "right",
            0x28 => "down",
            >= 0x70 and <= 0x7B => $"f{vkCode - 0x6F}", // F1-F12
            >= 0x41 and <= 0x5A => ((char)vkCode).ToString().ToLower(), // A-Z
            >= 0x30 and <= 0x39 => ((char)vkCode).ToString(), // 0-9
            _ => vkCode.ToString()
        };
    }
    
    public void Dispose()
    {
        if (_recording)
        {
            StopRecording();
            return;
        }

        if (_mouseHookID != IntPtr.Zero)
        {
            WinAPI.UnhookWindowsHookEx(_mouseHookID);
            _mouseHookID = IntPtr.Zero;
        }

        if (_keyboardHookID != IntPtr.Zero)
        {
            WinAPI.UnhookWindowsHookEx(_keyboardHookID);
            _keyboardHookID = IntPtr.Zero;
        }
    }
}
