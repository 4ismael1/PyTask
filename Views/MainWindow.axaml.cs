using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using PyTaskAvalonia.ViewModels;
using System;
using System.Linq;

namespace PyTaskAvalonia.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
    }
    
    private MainWindowViewModel? ViewModel => DataContext as MainWindowViewModel;
    
    private void PlayButton_Click(object? sender, RoutedEventArgs e)
    {
        // Click en botón: toggle playback
        ViewModel?.TogglePlayback();
    }
    
    private void PrefsMenu_Opening(object? sender, EventArgs e)
    {
        if (ViewModel == null) return;
        
        // Actualizar checks de velocidad
        PrefsSpeed05Item.Header = Math.Abs(ViewModel.PlaySpeed - 0.5) < 0.01 ? "✓ 0.5x" : "  0.5x";
        PrefsSpeed1Item.Header = Math.Abs(ViewModel.PlaySpeed - 1.0) < 0.01 ? "✓ 1x" : "  1x";
        PrefsSpeed2Item.Header = Math.Abs(ViewModel.PlaySpeed - 2.0) < 0.01 ? "✓ 2x" : "  2x";
        PrefsSpeed100Item.Header = Math.Abs(ViewModel.PlaySpeed - 100.0) < 0.01 ? "✓ 100x" : "  100x";
        
        // Actualizar checks de modo
        PrefsModeOnceItem.Header = ViewModel.PlaybackLoops == 1 ? "✓ Una vez" : "  Una vez";
        PrefsModeInfiniteItem.Header = ViewModel.PlaybackLoops == 0 ? "✓ Infinito" : "  Infinito";
        
        // Actualizar checks de opciones
        PrefsAlwaysOnTopItem.Header = ViewModel.AlwaysOnTop ? "✓ Siempre Visible" : "  Siempre Visible";
        PrefsStatusBarItem.Header = ViewModel.ShowStatusBar ? "✓ Barra de Estado" : "  Barra de Estado";
        PrefsSendInputItem.Header = ViewModel.Settings.UseSendInput ? "✓ Modo Juegos (SendInput)" : "  Modo Juegos (SendInput)";
        PrefsRecordHotkeyMenu.Header = $"Tecla Grabar ({ViewModel.Settings.RecordHotkey.ToUpperInvariant()})";
        PrefsPlayHotkeyMenu.Header = $"Tecla Reproducir ({ViewModel.Settings.PlayHotkey.ToUpperInvariant()})";
    }
    
    private void PrefsButton_Click(object? sender, RoutedEventArgs e)
    {
        // Mostrar menú de preferencias
        if (sender is Button button && button.ContextFlyout != null)
        {
            button.ContextFlyout.ShowAt(button);
        }
    }
    
    // ===== EVENT HANDLERS DEL MENÚ PREFERENCIAS =====
    
    private void SetSpeed_Click(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem menuItem && menuItem.Tag is string speedStr)
        {
            if (double.TryParse(speedStr, out var speed))
            {
                ViewModel?.SetPlaySpeed(speed);
            }
        }
    }
    
    private async void SetCustomSpeed_Click(object? sender, RoutedEventArgs e)
    {
        var dialog = new Window
        {
            Title = "Velocidad Personalizada",
            Width = 300,
            Height = 150,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Icon = new Avalonia.Controls.WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://PyTaskAvalonia/Assets/Icons/portapapeles.ico")))
        };
        
        var stackPanel = new StackPanel { Margin = new Avalonia.Thickness(20), Spacing = 10 };
        
        stackPanel.Children.Add(new TextBlock 
        { 
            Text = "Multiplicador de velocidad:", 
            FontWeight = Avalonia.Media.FontWeight.Bold
        });
        
        var numericUpDown = new NumericUpDown
        {
            Minimum = 0.1m,
            Maximum = 1000m,
            Value = (decimal)(ViewModel?.CustomSpeed ?? 8.0),
            Increment = 0.5m,
            FormatString = "F1"
        };
        stackPanel.Children.Add(numericUpDown);
        
        var buttonPanel = new StackPanel 
        { 
            Orientation = Avalonia.Layout.Orientation.Horizontal,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Right,
            Margin = new Avalonia.Thickness(0, 15, 0, 0),
            Spacing = 10
        };
        
        var okButton = new Button 
        { 
            Content = "Aceptar", 
            Width = 80,
            Background = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Color.Parse("#2196F3")),
            Foreground = Avalonia.Media.Brushes.White,
            BorderThickness = new Avalonia.Thickness(0),
            CornerRadius = new Avalonia.CornerRadius(4)
        };
        okButton.Click += (s, args) => 
        {
            ViewModel?.SetCustomSpeed((double)numericUpDown.Value);
            dialog.Close();
        };
        
        var cancelButton = new Button 
        { 
            Content = "Cancelar", 
            Width = 80,
            Background = Avalonia.Media.Brushes.White,
            BorderBrush = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Color.Parse("#cccccc")),
            BorderThickness = new Avalonia.Thickness(1),
            CornerRadius = new Avalonia.CornerRadius(4)
        };
        cancelButton.Click += (s, args) => dialog.Close();
        
        buttonPanel.Children.Add(okButton);
        buttonPanel.Children.Add(cancelButton);
        stackPanel.Children.Add(buttonPanel);
        
        dialog.Content = stackPanel;
        await dialog.ShowDialog(this);
    }
    
    private void SetModeOnce_Click(object? sender, RoutedEventArgs e)
    {
        ViewModel?.SetModeOnce();
    }
    
    private void SetModeInfinite_Click(object? sender, RoutedEventArgs e)
    {
        ViewModel?.SetModeInfinite();
    }
    
    private void SetIntervalQuick_Click(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem menuItem && menuItem.Tag is string tag)
        {
            var parts = tag.Split(',');
            if (parts.Length == 2 && 
                int.TryParse(parts[0], out var seconds) && 
                int.TryParse(parts[1], out var loops))
            {
                // Si se selecciona un intervalo (no "Sin intervalo") y está en modo "Una vez"
                if (seconds > 0 && ViewModel?.PlaybackLoops == 1)
                {
                    // Cambiar automáticamente a infinito
                    ViewModel?.SetIntervalMode(seconds, 0); // 0 = infinito
                }
                else
                {
                    ViewModel?.SetIntervalMode(seconds, loops);
                }
            }
        }
    }
    
    private async void ConfigureInterval_Click(object? sender, RoutedEventArgs e)
    {
        // Diálogo de configuración de intervalo - OPTIMIZADO
        var dialog = new Window
        {
            Title = "Configurar Pausa de Tiempo",
            Width = 320,
            Height = 200,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Icon = new Avalonia.Controls.WindowIcon(Avalonia.Platform.AssetLoader.Open(new Uri("avares://PyTaskAvalonia/Assets/Icons/portapapeles.ico")))
        };
        
        var mainStack = new StackPanel { Margin = new Avalonia.Thickness(15), Spacing = 8 };
        
        // Grupo: Tiempo de pausa
        mainStack.Children.Add(new TextBlock 
        { 
            Text = "Tiempo de pausa entre ejecuciones:",
            FontWeight = Avalonia.Media.FontWeight.Bold,
            FontSize = 12
        });
        
        var timePanel = new StackPanel { Orientation = Avalonia.Layout.Orientation.Horizontal, Spacing = 8 };
        var timeValue = new NumericUpDown
        {
            Minimum = 1,
            Maximum = 3600,
            Value = ViewModel?.IntervalSeconds ?? 5,
            Increment = 1,
            Width = 100
        };
        timePanel.Children.Add(timeValue);
        timePanel.Children.Add(new TextBlock { Text = "segundos", VerticalAlignment = Avalonia.Layout.VerticalAlignment.Center });
        mainStack.Children.Add(timePanel);
        
        // Grupo: Cantidad de repeticiones
        mainStack.Children.Add(new TextBlock 
        { 
            Text = "Cantidad de repeticiones:",
            FontWeight = Avalonia.Media.FontWeight.Bold,
            FontSize = 12,
            Margin = new Avalonia.Thickness(0, 8, 0, 0)
        });
        
        var loopsPanel = new StackPanel { Orientation = Avalonia.Layout.Orientation.Horizontal, Spacing = 8 };
        var loopsValue = new NumericUpDown
        {
            Minimum = 0,
            Maximum = 10000,
            Value = ViewModel?.PlaybackLoops ?? 1,
            Increment = 1,
            Width = 100
        };
        loopsPanel.Children.Add(loopsValue);
        loopsPanel.Children.Add(new TextBlock { Text = "(0 = infinito)", VerticalAlignment = Avalonia.Layout.VerticalAlignment.Center, Foreground = Avalonia.Media.Brushes.Gray });
        mainStack.Children.Add(loopsPanel);
        
        // Botones
        var buttonPanel = new StackPanel 
        { 
            Orientation = Avalonia.Layout.Orientation.Horizontal,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Right,
            Spacing = 10,
            Margin = new Avalonia.Thickness(0, 15, 0, 0)
        };
        
        var okButton = new Button 
        { 
            Content = "Aceptar", 
            Width = 80,
            Background = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Color.Parse("#2196F3")),
            Foreground = Avalonia.Media.Brushes.White,
            BorderThickness = new Avalonia.Thickness(0),
            CornerRadius = new Avalonia.CornerRadius(4)
        };
        okButton.Click += (s, args) => 
        {
            ViewModel?.SetIntervalMode((int)timeValue.Value, (int)loopsValue.Value);
            dialog.Close();
        };
        
        var cancelButton = new Button 
        { 
            Content = "Cancelar", 
            Width = 80,
            Background = Avalonia.Media.Brushes.White,
            BorderBrush = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Color.Parse("#cccccc")),
            BorderThickness = new Avalonia.Thickness(1),
            CornerRadius = new Avalonia.CornerRadius(4)
        };
        cancelButton.Click += (s, args) => dialog.Close();
        
        buttonPanel.Children.Add(okButton);
        buttonPanel.Children.Add(cancelButton);
        mainStack.Children.Add(buttonPanel);
        
        dialog.Content = mainStack;
        await dialog.ShowDialog(this);
    }
    
    // ===== EVENT HANDLERS ADICIONALES =====
    
    private void SetRecordHotkey_Click(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem menuItem && menuItem.Tag is string key)
        {
            ViewModel?.SetRecordHotkey(key);
        }
    }
    
    private void SetPlayHotkey_Click(object? sender, RoutedEventArgs e)
    {
        if (sender is MenuItem menuItem && menuItem.Tag is string key)
        {
            ViewModel?.SetPlayHotkey(key);
        }
    }
    
    private void ToggleAlwaysOnTop_Click(object? sender, RoutedEventArgs e)
    {
        ViewModel?.ToggleAlwaysOnTop();
    }
    
    private void ToggleShowCaptions_Click(object? sender, RoutedEventArgs e)
    {
        ViewModel?.ToggleShowCaptions();
    }
    
    private void ToggleSendInput_Click(object? sender, RoutedEventArgs e)
    {
        ViewModel?.ToggleSendInput();
    }
    
    private async void ShowAbout_Click(object? sender, RoutedEventArgs e)
    {
        var dialog = new Window
        {
            Title = "Acerca de PyTask",
            Width = 400,
            Height = 200,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner
        };
        
        var stackPanel = new StackPanel 
        { 
            Margin = new Avalonia.Thickness(20),
            Spacing = 10
        };
        
        stackPanel.Children.Add(new TextBlock 
        { 
            Text = "PyTask",
            FontSize = 20,
            FontWeight = Avalonia.Media.FontWeight.Bold,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Center
        });
        
        stackPanel.Children.Add(new TextBlock 
        { 
            Text = "Versión 2.0.0 (C# + Avalonia UI)",
            FontSize = 12,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Center
        });
        
        stackPanel.Children.Add(new TextBlock 
        { 
            Text = "Automatización Avanzada de Macros",
            FontSize = 12,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Center,
            Margin = new Avalonia.Thickness(0, 0, 0, 10)
        });
        
        stackPanel.Children.Add(new TextBlock 
        { 
            Text = "✓ Modo Juegos activado (SendInput)\n✓ Compatible con aplicaciones exigentes\n✓ Grabación y reproducción de macros\n✓ Hotkeys globales configurables",
            FontSize = 11,
            TextWrapping = Avalonia.Media.TextWrapping.Wrap
        });
        
        stackPanel.Children.Add(new TextBlock 
        { 
            Text = "GitHub: @4ismael1",
            FontSize = 10,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Center,
            Margin = new Avalonia.Thickness(0, 10, 0, 0)
        });
        
        var okButton = new Button 
        { 
            Content = "OK", 
            Width = 100,
            HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Center,
            Margin = new Avalonia.Thickness(0, 10, 0, 0)
        };
        okButton.Click += (s, args) => dialog.Close();
        stackPanel.Children.Add(okButton);
        
        dialog.Content = stackPanel;
        await dialog.ShowDialog(this);
    }
}
