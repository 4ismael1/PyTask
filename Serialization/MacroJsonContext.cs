using System.Text.Json.Serialization;
using PyTaskAvalonia.Models;

namespace PyTaskAvalonia.Serialization;

[JsonSourceGenerationOptions(PropertyNameCaseInsensitive = true, WriteIndented = true)]
[JsonSerializable(typeof(MacroFile))]
internal partial class MacroJsonContext : JsonSerializerContext
{
}
