namespace PyTaskAvalonia.Models;

public class Settings
{
    public bool SpeedHalf { get; set; } = true;
    public bool Speed1x { get; set; } = true;
    public bool Speed2x { get; set; } = true;
    public bool Speed100x { get; set; } = true;
    public string RecordHotkey { get; set; } = "F9";
    public string PlayHotkey { get; set; } = "F10";
    public bool AlwaysOnTop { get; set; } = false;
    public bool ShowCaptions { get; set; } = true;
    public bool UseSendInput { get; set; } = true;
}
