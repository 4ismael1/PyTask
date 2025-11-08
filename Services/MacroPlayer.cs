using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using PyTaskAvalonia.Models;

namespace PyTaskAvalonia.Services;

public class MacroPlayer
{
    private static readonly int InputSize = Marshal.SizeOf<WinAPI.INPUT>();
    
    private bool _playing;
    private CancellationTokenSource? _cancellationTokenSource;
    private readonly int _screenWidth;
    private readonly int _screenHeight;
    private readonly int _screenLeft;
    private readonly int _screenTop;
    private readonly double _xNormalizer;
    private readonly double _yNormalizer;
    
    public bool IsPlaying => _playing;
    public bool UseSendInput { get; set; } = true;
    public event EventHandler? PlaybackFinished;
    
    public MacroPlayer()
    {
        _screenLeft = WinAPI.GetSystemMetrics(WinAPI.SM_XVIRTUALSCREEN);
        _screenTop = WinAPI.GetSystemMetrics(WinAPI.SM_YVIRTUALSCREEN);
        _screenWidth = WinAPI.GetSystemMetrics(WinAPI.SM_CXVIRTUALSCREEN);
        _screenHeight = WinAPI.GetSystemMetrics(WinAPI.SM_CYVIRTUALSCREEN);

        if (_screenWidth <= 0 || _screenHeight <= 0)
        {
            _screenLeft = 0;
            _screenTop = 0;
            _screenWidth = WinAPI.GetSystemMetrics(WinAPI.SM_CXSCREEN);
            _screenHeight = WinAPI.GetSystemMetrics(WinAPI.SM_CYSCREEN);
        }

        var widthRange = Math.Max(1, _screenWidth - 1);
        var heightRange = Math.Max(1, _screenHeight - 1);
        _xNormalizer = 65535d / widthRange;
        _yNormalizer = 65535d / heightRange;
    }
    
    public async Task PlayMacroAsync(List<MacroEvent> events, double speed = 1.0, 
        int loops = 1, bool intervalMode = false, int intervalSeconds = 5, 
        CancellationToken cancellationToken = default)
    {
        if (_playing || events.Count == 0)
        {
            return;
        }
        
        _playing = true;
        _cancellationTokenSource = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        var token = _cancellationTokenSource.Token;
        var normalizedSpeed = speed <= 0 ? 0.01 : speed;
        var targetLoops = loops < 0 ? 0 : loops;
        var intervalDelaySeconds = Math.Max(0, intervalSeconds);
        
        try
        {
            var isInfinite = targetLoops == 0;
            var loopsDone = 0;
            
            while (_playing && !token.IsCancellationRequested)
            {
                await PlayEventsOnceAsync(events, normalizedSpeed, token);
                loopsDone++;
                
                if (!isInfinite && loopsDone >= targetLoops)
                {
                    break;
                }
                
                if (intervalMode && _playing)
                {
                    if (intervalDelaySeconds > 0)
                    {
                        await Task.Delay(TimeSpan.FromSeconds(intervalDelaySeconds), token);
                    }
                }
                else if (!intervalMode)
                {
                    await Task.Delay(50, token);
                }
            }
        }
        catch (OperationCanceledException)
        {
            // Esperado cuando se cancela
        }
        finally
        {
            _playing = false;
            _cancellationTokenSource?.Dispose();
            _cancellationTokenSource = null;
            PlaybackFinished?.Invoke(this, EventArgs.Empty);
        }
    }
    
    private async Task PlayEventsOnceAsync(List<MacroEvent> events, double speed, CancellationToken cancellationToken)
    {
        var stopwatch = Stopwatch.StartNew();
        
        foreach (var evt in events)
        {
            if (!_playing || cancellationToken.IsCancellationRequested)
            {
                break;
            }
            
            var targetTime = evt.Timestamp / speed;
            var elapsed = stopwatch.Elapsed.TotalSeconds;
            var sleepTime = targetTime - elapsed;
            
            if (sleepTime > 0)
            {
                await Task.Delay(TimeSpan.FromSeconds(sleepTime), cancellationToken);
            }
            
            ExecuteEvent(evt);
        }
    }
    
    public void StopPlayback()
    {
        _playing = false;
        _cancellationTokenSource?.Cancel();
    }
    
    private void ExecuteEvent(MacroEvent evt)
    {
        try
        {
            switch (evt.Type)
            {
                case "mouse_move":
                    if (evt.X.HasValue && evt.Y.HasValue)
                    {
                        SendMouseMove(evt.X.Value, evt.Y.Value);
                    }
                    break;
                
                case "mouse_click":
                    if (evt.X.HasValue && evt.Y.HasValue && evt.Button != null && evt.Pressed.HasValue)
                    {
                        SendMouseClick(evt.X.Value, evt.Y.Value, evt.Button, evt.Pressed.Value);
                    }
                    break;
                
                case "mouse_scroll":
                    if (evt.X.HasValue && evt.Y.HasValue && evt.Dy.HasValue)
                    {
                        SendMouseScroll(evt.X.Value, evt.Y.Value, evt.Dy.Value);
                    }
                    break;
                
                case "key_press":
                case "key_release":
                    if (evt.VkCode.HasValue)
                    {
                        var isPress = evt.Type == "key_press";
                        SendKeyboardEvent((ushort)evt.VkCode.Value, isPress);
                    }
                    break;
            }
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Error ejecutando evento: {ex.Message}");
        }
    }
    
    private (int normalizedX, int normalizedY) NormalizeCoordinates(int x, int y)
    {
        var normalizedX = Math.Clamp((int)Math.Round((((double)x - _screenLeft) * _xNormalizer)), 0, 65535);
        var normalizedY = Math.Clamp((int)Math.Round((((double)y - _screenTop) * _yNormalizer)), 0, 65535);
        return (normalizedX, normalizedY);
    }
    
    private static WinAPI.INPUT CreateMouseInput(int dx, int dy, uint flags, uint mouseData = 0)
    {
        return new WinAPI.INPUT
        {
            type = WinAPI.INPUT_MOUSE,
            u = new WinAPI.INPUTUNION
            {
                mi = new WinAPI.MOUSEINPUT
                {
                    dx = dx,
                    dy = dy,
                    mouseData = mouseData,
                    dwFlags = flags,
                    time = 0,
                    dwExtraInfo = InputSignature.Value
                }
            }
        };
    }
    
    private void SendMouseMove(int x, int y)
    {
        var (normalizedX, normalizedY) = NormalizeCoordinates(x, y);
        
        if (UseSendInput)
        {
            var input = CreateMouseInput(normalizedX, normalizedY,
                WinAPI.MOUSEEVENTF_MOVE | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK);
            var sent = WinAPI.SendInput(1, new[] { input }, InputSize);
            if (sent == 0)
            {
                Debug.WriteLine($"SendInput mouse move failed with error {Marshal.GetLastWin32Error()}");
            }
        }
        else
        {
            WinAPI.mouse_event(WinAPI.MOUSEEVENTF_MOVE | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK,
                normalizedX, normalizedY, 0, InputSignature.UIntValue);
        }
    }
    
    private void SendMouseClick(int x, int y, string button, bool pressed)
    {
        var (normalizedX, normalizedY) = NormalizeCoordinates(x, y);
        
        uint flags = button.ToLowerInvariant() switch
        {
            "left" => pressed ? WinAPI.MOUSEEVENTF_LEFTDOWN : WinAPI.MOUSEEVENTF_LEFTUP,
            "right" => pressed ? WinAPI.MOUSEEVENTF_RIGHTDOWN : WinAPI.MOUSEEVENTF_RIGHTUP,
            "middle" => pressed ? WinAPI.MOUSEEVENTF_MIDDLEDOWN : WinAPI.MOUSEEVENTF_MIDDLEUP,
            _ => pressed ? WinAPI.MOUSEEVENTF_LEFTDOWN : WinAPI.MOUSEEVENTF_LEFTUP
        };
        
        if (UseSendInput)
        {
            var moveInput = CreateMouseInput(normalizedX, normalizedY,
                WinAPI.MOUSEEVENTF_MOVE | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK);
            var clickInput = CreateMouseInput(normalizedX, normalizedY,
                flags | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK);
            
            var inputs = new[] { moveInput, clickInput };
            var sent = WinAPI.SendInput((uint)inputs.Length, inputs, InputSize);
            if (sent != inputs.Length)
            {
                Debug.WriteLine($"SendInput mouse click failed with error {Marshal.GetLastWin32Error()}");
            }
        }
        else
        {
            WinAPI.mouse_event(WinAPI.MOUSEEVENTF_MOVE | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK,
                normalizedX, normalizedY, 0, InputSignature.UIntValue);
            WinAPI.mouse_event(flags | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK,
                normalizedX, normalizedY, 0, InputSignature.UIntValue);
        }
    }
    
    private void SendMouseScroll(int x, int y, int delta)
    {
        var (normalizedX, normalizedY) = NormalizeCoordinates(x, y);
        var wheelDelta = delta * 120;
        
        if (UseSendInput)
        {
            var moveInput = CreateMouseInput(normalizedX, normalizedY,
                WinAPI.MOUSEEVENTF_MOVE | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK);
            var scrollInput = CreateMouseInput(0, 0, WinAPI.MOUSEEVENTF_WHEEL, unchecked((uint)wheelDelta));
            var inputs = new[] { moveInput, scrollInput };
            var sent = WinAPI.SendInput((uint)inputs.Length, inputs, InputSize);
            if (sent != inputs.Length)
            {
                Debug.WriteLine($"SendInput mouse scroll failed with error {Marshal.GetLastWin32Error()}");
            }
        }
        else
        {
            WinAPI.mouse_event(WinAPI.MOUSEEVENTF_MOVE | WinAPI.MOUSEEVENTF_ABSOLUTE | WinAPI.MOUSEEVENTF_VIRTUALDESK,
                normalizedX, normalizedY, 0, InputSignature.UIntValue);
            WinAPI.mouse_event(WinAPI.MOUSEEVENTF_WHEEL, 0, 0, unchecked((uint)wheelDelta), InputSignature.UIntValue);
        }
    }
    
    private void SendKeyboardEvent(ushort vkCode, bool isPress)
    {
        if (UseSendInput)
        {
            var input = new WinAPI.INPUT
            {
                type = WinAPI.INPUT_KEYBOARD,
                u = new WinAPI.INPUTUNION
                {
                    ki = new WinAPI.KEYBDINPUT
                    {
                        wVk = vkCode,
                        wScan = 0,
                        dwFlags = isPress ? 0 : WinAPI.KEYEVENTF_KEYUP,
                        time = 0,
                        dwExtraInfo = InputSignature.Value
                    }
                }
            };
            
            var sent = WinAPI.SendInput(1, new[] { input }, InputSize);
            if (sent == 0)
            {
                Debug.WriteLine($"SendInput keyboard event failed with error {Marshal.GetLastWin32Error()}");
            }
        }
        else
        {
            WinAPI.keybd_event((byte)vkCode, 0, isPress ? 0u : WinAPI.KEYEVENTF_KEYUP, InputSignature.UIntValue);
        }
    }
}
