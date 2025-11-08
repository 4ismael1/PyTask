using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace PyTaskAvalonia.Models;

public class MacroEvent
{
    [JsonPropertyName("type")]
    public string Type { get; set; } = string.Empty;
    
    [JsonPropertyName("timestamp")]
    public double Timestamp { get; set; }
    
    // Para eventos de mouse
    [JsonPropertyName("x")]
    public int? X { get; set; }
    
    [JsonPropertyName("y")]
    public int? Y { get; set; }
    
    [JsonPropertyName("button")]
    public string? Button { get; set; }
    
    [JsonPropertyName("pressed")]
    public bool? Pressed { get; set; }
    
    [JsonPropertyName("dx")]
    public int? Dx { get; set; }
    
    [JsonPropertyName("dy")]
    public int? Dy { get; set; }
    
    // Para eventos de teclado
    [JsonPropertyName("key")]
    public string? Key { get; set; }
    
    [JsonPropertyName("vkCode")]
    public uint? VkCode { get; set; }
}

public class MacroFile
{
    [JsonPropertyName("events")]
    public List<MacroEvent> Events { get; set; } = new();
}
