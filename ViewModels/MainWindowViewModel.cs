using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Input;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using PyTaskAvalonia.Models;
using PyTaskAvalonia.Services;

namespace PyTaskAvalonia.ViewModels;

public partial class MainWindowViewModel : ViewModelBase, IDisposable
{
    private MacroRecorder _recorder = null!;
    private MacroPlayer _player = null!;
    private SettingsDatabase _database = null!;
    private GlobalHotkeyService _hotkeyService = null!;
    
    [ObservableProperty]
    private bool _isRecording;
    
    [ObservableProperty]
    private bool _isPlaying;
    
    [ObservableProperty]
    private string _statusMessage = "Listo";
    
    [ObservableProperty]
    private bool _canSave;
    
    [ObservableProperty]
    private bool _canPlay;
    
    [ObservableProperty]
    private string _windowTitle = "PyTask";
    
    [ObservableProperty]
    private bool _showStatusBar = true;
    
    private List<MacroEvent> _currentMacroEvents = new();
    private string? _currentMacroFile;
    private double _playSpeed = 1.0;
    private double _customSpeed = 8.0;
    private int _playbackLoops = 1;
    private bool _intervalMode = false;
    private int _intervalSeconds = 5;
    private Settings _settings = new();
    private CancellationTokenSource? _playCancellation;
    private static readonly JsonSerializerOptions MacroDeserializeOptions = new()
    {
        PropertyNameCaseInsensitive = true
    };
    private static readonly JsonSerializerOptions MacroSerializeOptions = new()
    {
        WriteIndented = true
    };
    
    public MainWindowViewModel()
    {
        try
        {
            _recorder = new MacroRecorder();
            _player = new MacroPlayer();
            _database = new SettingsDatabase();
            _hotkeyService = new GlobalHotkeyService();
            
            // Cargar configuración
            _settings = _database.LoadSettings();
            ShowStatusBar = _settings.ShowCaptions;
            _player.UseSendInput = _settings.UseSendInput;
            
            // Configurar hotkeys
            SetupHotkeys();
            
            UpdateStatusMessage();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Error en constructor: {ex}");
            StatusMessage = $"Error: {ex.Message}";
        }
    }
    
    private bool CanToggleRecording() => IsRecording || !IsPlaying;
    
    private void SetupHotkeys()
    {
        _hotkeyService.ClearHotkeys();
        _hotkeyService.RegisterHotkey(_settings.RecordHotkey, () => 
        {
            Avalonia.Threading.Dispatcher.UIThread.Post(ToggleRecording);
        });
        _hotkeyService.RegisterHotkey(_settings.PlayHotkey, () => 
        {
            Avalonia.Threading.Dispatcher.UIThread.Post(TogglePlayback);
        });
    }
    
    [RelayCommand]
    public async Task OpenMacro()
    {
        try
        {
            var window = GetMainWindow();
            if (window == null) return;
            
            var files = await window.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = "Abrir Macro",
                AllowMultiple = false,
                FileTypeFilter = new[]
                {
                    new FilePickerFileType("Archivos de Macro") { Patterns = new[] { "*.macro" } },
                    new FilePickerFileType("Todos los Archivos") { Patterns = new[] { "*" } }
                }
            });
            
            if (files.Count > 0)
            {
                var file = files[0];
                var filePath = file.Path.LocalPath;
                
                var json = await File.ReadAllTextAsync(filePath);
                var macroFile = JsonSerializer.Deserialize<MacroFile>(json, MacroDeserializeOptions);
                
                if (macroFile?.Events is { Count: > 0 } events)
                {
                    _currentMacroEvents = events;
                    _currentMacroFile = filePath;
                    
                    CanSave = true;
                    CanPlay = true;
                    WindowTitle = $"PyTask - {Path.GetFileName(filePath)}";
                    StatusMessage = $"Cargado: {_currentMacroEvents.Count} eventos";
                }
                else
                {
                    StatusMessage = "El archivo seleccionado no contiene eventos.";
                }
            }
        }
        catch (Exception ex)
        {
            StatusMessage = $"Error al cargar macro: {ex.Message}";
        }
    }
    
    [RelayCommand]
    public async Task SaveMacro()
    {
        if (_currentMacroFile != null)
        {
            await SaveToFile(_currentMacroFile);
        }
        else
        {
            await SaveMacroAs();
        }
    }
    
    private async Task SaveMacroAs()
    {
        try
        {
            var window = GetMainWindow();
            if (window == null) return;
            
            var file = await window.StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions
            {
                Title = "Guardar Macro",
                DefaultExtension = "macro",
                SuggestedFileName = "macro.macro",
                FileTypeChoices = new[]
                {
                    new FilePickerFileType("Archivos de Macro") { Patterns = new[] { "*.macro" } }
                }
            });
            
            if (file != null)
            {
                var filePath = file.Path.LocalPath;
                await SaveToFile(filePath);
                _currentMacroFile = filePath;
                WindowTitle = $"PyTask - {Path.GetFileName(filePath)}";
            }
        }
        catch (Exception ex)
        {
            StatusMessage = $"Error al guardar: {ex.Message}";
        }
    }
    
    private async Task SaveToFile(string filePath)
    {
        try
        {
            var macroFile = new MacroFile { Events = _currentMacroEvents };
            var json = JsonSerializer.Serialize(macroFile, MacroSerializeOptions);
            
            await File.WriteAllTextAsync(filePath, json);
            StatusMessage = $"Guardado: {Path.GetFileName(filePath)}";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Error al guardar: {ex.Message}";
        }
    }
    
    [RelayCommand(CanExecute = nameof(CanToggleRecording))]
    public void ToggleRecording()
    {
        if (!IsRecording)
        {
            // Verificar que no se esté reproduciendo
            if (IsPlaying)
            {
                StatusMessage = "No se puede grabar mientras se reproduce";
                return;
            }
            
            // Verificar que el player haya terminado completamente
            if (_player.IsPlaying)
            {
                StatusMessage = "Espere a que termine la reproducción";
                return;
            }
            
            // Iniciar grabación
            _currentMacroFile = null;
            IsRecording = true;
            CanPlay = false;
            CanSave = false;
            StatusMessage = $"Grabando... (Presiona {_settings.RecordHotkey} para detener)";
            _recorder.StartRecording();
        }
        else
        {
            // Detener grabación
            IsRecording = false;
            _currentMacroEvents = _recorder.StopRecording();
            
            if (_currentMacroEvents.Count > 0)
            {
                CanSave = true;
                CanPlay = true;
                WindowTitle = "PyTask";
                StatusMessage = $"Grabados {_currentMacroEvents.Count} eventos";
            }
            else
            {
                StatusMessage = "No se grabaron eventos";
            }
        }
    }
    
    [RelayCommand]
    public void TogglePlayback()
    {
        if (IsRecording)
        {
            StatusMessage = "No se puede reproducir mientras se graba";
            return;
        }

        if (!IsPlaying)
        {
            PlayMacro();
        }
        else
        {
            StopPlayback();
        }
    }
    
    private async void PlayMacro()
    {
        if (_currentMacroEvents.Count == 0)
        {
            StatusMessage = "No hay macro cargada";
            return;
        }
        
        if (IsPlaying) return;
        
        IsPlaying = true;
        CanSave = false;
        
        _playCancellation = new CancellationTokenSource();
        
        var isInfinite = _playbackLoops == 0;
        var modeText = _intervalMode 
            ? (isInfinite ? $"INTERVALO {_intervalSeconds}s (INFINITO)" : $"INTERVALO {_intervalSeconds}s ({_playbackLoops} veces)")
            : (isInfinite ? "INFINITO (sin pausa)" : _playbackLoops == 1 ? "UNA VEZ" : $"{_playbackLoops} VECES (sin pausa)");
        
        StatusMessage = $"{modeText} a {_playSpeed}x ({_settings.PlayHotkey}=Detener)";
        
        string? errorMessage = null;
        
        try
        {
            await _player.PlayMacroAsync(_currentMacroEvents, _playSpeed, _playbackLoops, 
                _intervalMode, _intervalSeconds, _playCancellation.Token);
        }
        catch (Exception ex)
        {
            errorMessage = $"Error: {ex.Message}";
        }
        finally
        {
            var wasCancelled = _playCancellation?.IsCancellationRequested ?? false;
            CompletePlayback(!wasCancelled && errorMessage == null);
        }
        
        if (errorMessage != null)
        {
            StatusMessage = errorMessage;
        }
    }
    
    private void StopPlayback()
    {
        if (IsPlaying)
        {
            _playCancellation?.Cancel();
            _player.StopPlayback();
            StatusMessage = "Detenido";
        }
    }
    
    private void CompletePlayback(bool restoreStatusMessage)
    {
        IsPlaying = false;
        CanSave = _currentMacroEvents.Count > 0;
        CanPlay = _currentMacroEvents.Count > 0;
        
        _playCancellation?.Dispose();
        _playCancellation = null;
        
        if (restoreStatusMessage)
        {
            UpdateStatusMessage();
        }
    }
    
    public void SetPlaySpeed(double speed)
    {
        _playSpeed = speed;
        StatusMessage = $"Velocidad: {speed}x";
    }
    
    public void SetCustomSpeed(double speed)
    {
        _customSpeed = speed;
        _playSpeed = speed;
        StatusMessage = $"Velocidad personalizada: {speed}x";
    }
    
    public void SetModeOnce()
    {
        _intervalMode = false;
        _playbackLoops = 1;
        StatusMessage = "Modo: Reproducir 1 vez";
    }
    
    public void SetModeInfinite()
    {
        _intervalMode = false;
        _playbackLoops = 0;
        StatusMessage = "Modo: Reproducción INFINITA (sin pausa)";
    }
    
    public void SetIntervalMode(int seconds, int loops)
    {
        _intervalMode = seconds > 0;
        _intervalSeconds = seconds;
        _playbackLoops = loops;
        
        if (seconds == 0)
        {
            StatusMessage = "Modo: Sin intervalo";
        }
        else if (loops == 0)
        {
            StatusMessage = $"Modo: Cada {seconds}s (INFINITO) - cambiado automáticamente";
        }
        else
        {
            StatusMessage = $"Modo: {loops} veces cada {seconds}s";
        }
    }
    
    public void ToggleAlwaysOnTop()
    {
        _settings.AlwaysOnTop = !_settings.AlwaysOnTop;
        _database.SaveSettings(_settings);
        StatusMessage = $"Siempre visible: {(_settings.AlwaysOnTop ? "ON" : "OFF")}";
        
        // Notificar cambio para actualizar la ventana
        OnPropertyChanged(nameof(AlwaysOnTop));
    }
    
    public void ToggleShowCaptions()
    {
        _settings.ShowCaptions = !_settings.ShowCaptions;
        ShowStatusBar = _settings.ShowCaptions;
        _database.SaveSettings(_settings);
    }
    
    public void ToggleSendInput()
    {
        _settings.UseSendInput = !_settings.UseSendInput;
        _database.SaveSettings(_settings);
        _player.UseSendInput = _settings.UseSendInput;
        StatusMessage = _settings.UseSendInput ? "✓ Modo Juegos activado (SendInput)" : "✓ Modo Normal activado";
    }
    
    public void SetRecordHotkey(string key)
    {
        key = NormalizeHotkey(key);
        var oldKey = FormatHotkey(_settings.RecordHotkey);
        _settings.RecordHotkey = key;
        _database.SaveSettings(_settings);
        SetupHotkeys();
        StatusMessage = $"Tecla de grabación cambiada: {oldKey} → {key}";
        Task.Delay(2000).ContinueWith(_ => Avalonia.Threading.Dispatcher.UIThread.Post(UpdateStatusMessage));
    }
    
    public void SetPlayHotkey(string key)
    {
        key = NormalizeHotkey(key);
        var oldKey = FormatHotkey(_settings.PlayHotkey);
        _settings.PlayHotkey = key;
        _database.SaveSettings(_settings);
        SetupHotkeys();
        StatusMessage = $"Tecla de reproducción cambiada: {oldKey} → {key}";
        Task.Delay(2000).ContinueWith(_ => Avalonia.Threading.Dispatcher.UIThread.Post(UpdateStatusMessage));
    }
    
    public void ToggleSpeedOption(string speedKey)
    {
        switch (speedKey)
        {
            case "speed_half": _settings.SpeedHalf = !_settings.SpeedHalf; break;
            case "speed_1x": _settings.Speed1x = !_settings.Speed1x; break;
            case "speed_2x": _settings.Speed2x = !_settings.Speed2x; break;
            case "speed_100x": _settings.Speed100x = !_settings.Speed100x; break;
        }
        _database.SaveSettings(_settings);
        StatusMessage = $"Opción de velocidad {speedKey} cambiada";
    }
    
    private void UpdateStatusMessage()
    {
        StatusMessage = $"Listo | Grabar: {FormatHotkey(_settings.RecordHotkey)} | Reproducir: {FormatHotkey(_settings.PlayHotkey)}";
    }
    
    public bool AlwaysOnTop => _settings.AlwaysOnTop;
    public Settings Settings => _settings;
    public double PlaySpeed => _playSpeed;
    public double CustomSpeed => _customSpeed;
    public int PlaybackLoops => _playbackLoops;
    public bool IntervalMode => _intervalMode;
    public int IntervalSeconds => _intervalSeconds;
    
    partial void OnIsRecordingChanged(bool value)
    {
        ToggleRecordingCommand?.NotifyCanExecuteChanged();
    }
    
    partial void OnIsPlayingChanged(bool value)
    {
        ToggleRecordingCommand?.NotifyCanExecuteChanged();
    }
    
    private Window? GetMainWindow()
    {
        var app = Avalonia.Application.Current;
        if (app?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
        {
            return desktop.MainWindow;
        }
        return null;
    }
    
    public void Dispose()
    {
        _playCancellation?.Cancel();
        _playCancellation?.Dispose();
        _recorder.Dispose();
        _hotkeyService.Dispose();
        _database.Dispose();
    }

    private static string NormalizeHotkey(string key) => string.IsNullOrWhiteSpace(key)
        ? string.Empty
        : key.Trim().ToUpperInvariant();

    private static string FormatHotkey(string key) => string.IsNullOrWhiteSpace(key)
        ? "-"
        : key.ToUpperInvariant();
}

