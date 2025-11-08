using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;
using Avalonia.Media.Immutable;
using Avalonia.Media.Imaging;
using Avalonia.Platform;

namespace PyTaskAvalonia.Converters;

public class RecordingBackgroundConverter : IValueConverter
{
    private static readonly IBrush ActiveBrush = new ImmutableSolidColorBrush(Color.Parse("#ff4444"));
    private static readonly IBrush InactiveBrush = new ImmutableSolidColorBrush(Color.Parse("#ffffff"));
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool isRecording && isRecording)
            return ActiveBrush;
        return InactiveBrush;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class RecordingBorderConverter : IValueConverter
{
    private static readonly IBrush ActiveBrush = new ImmutableSolidColorBrush(Color.Parse("#cc0000"));
    private static readonly IBrush InactiveBrush = new ImmutableSolidColorBrush(Color.Parse("#d0d0d0"));
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool isRecording && isRecording)
            return ActiveBrush;
        return InactiveBrush;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class RecordingTextConverter : IValueConverter
{
    private static readonly IBrush ActiveBrush = new ImmutableSolidColorBrush(Color.Parse("#ffffff"));
    private static readonly IBrush InactiveBrush = new ImmutableSolidColorBrush(Color.Parse("#333333"));
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool isRecording && isRecording)
            return ActiveBrush;
        return InactiveBrush;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class PlayingBackgroundConverter : IValueConverter
{
    private static readonly IBrush ActiveBrush = new ImmutableSolidColorBrush(Color.Parse("#ff4444"));
    private static readonly IBrush InactiveBrush = new ImmutableSolidColorBrush(Color.Parse("#ffffff"));
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool isPlaying && isPlaying)
            return ActiveBrush;
        return InactiveBrush;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class PlayingBorderConverter : IValueConverter
{
    private static readonly IBrush ActiveBrush = new ImmutableSolidColorBrush(Color.Parse("#cc0000"));
    private static readonly IBrush InactiveBrush = new ImmutableSolidColorBrush(Color.Parse("#d0d0d0"));
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool isPlaying && isPlaying)
            return ActiveBrush;
        return InactiveBrush;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class PlayingTextConverter : IValueConverter
{
    private static readonly IBrush ActiveBrush = new ImmutableSolidColorBrush(Color.Parse("#ffffff"));
    private static readonly IBrush InactiveBrush = new ImmutableSolidColorBrush(Color.Parse("#333333"));
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool isPlaying && isPlaying)
            return ActiveBrush;
        return InactiveBrush;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class PlayIconConverter : IValueConverter
{
    private static readonly Uri StopIconUri = new("avares://PyTaskAvalonia/Assets/Icons/boton-detener.png");
    private static readonly Uri PlayIconUri = new("avares://PyTaskAvalonia/Assets/Icons/Play.png");
    private static readonly Lazy<Bitmap> StopIcon = new(() =>
    {
        using var stream = AssetLoader.Open(StopIconUri);
        return new Bitmap(stream);
    });
    private static readonly Lazy<Bitmap> PlayIcon = new(() =>
    {
        using var stream = AssetLoader.Open(PlayIconUri);
        return new Bitmap(stream);
    });
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is bool isPlaying && isPlaying ? StopIcon.Value : PlayIcon.Value;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}

public class PlayTextConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is bool isPlaying && isPlaying ? "Stop" : "Play";
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}
