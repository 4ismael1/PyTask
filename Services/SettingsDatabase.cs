using System;
using System.IO;
using Microsoft.Data.Sqlite;
using PyTaskAvalonia.Models;

namespace PyTaskAvalonia.Services;

public class SettingsDatabase : IDisposable
{
    private readonly string _dbPath;
    private SqliteConnection? _connection;
    
    public SettingsDatabase(string? dbPath = null)
    {
        if (dbPath == null)
        {
            // Usar AppData/Roaming/PyTask
            var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
            var pytaskDir = Path.Combine(appData, "PyTask");
            Directory.CreateDirectory(pytaskDir);
            _dbPath = Path.Combine(pytaskDir, "pytask.db");
        }
        else
        {
            _dbPath = dbPath;
        }
        
        InitDatabase();
    }
    
    private void InitDatabase()
    {
        _connection = new SqliteConnection($"Data Source={_dbPath}");
        _connection.Open();
        
        using var command = _connection.CreateCommand();
        command.CommandText = @"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        ";
        command.ExecuteNonQuery();
    }
    
    public string? GetSetting(string key, string? defaultValue = null)
    {
        if (_connection == null) return defaultValue;
        
        using var command = _connection.CreateCommand();
        command.CommandText = "SELECT value FROM settings WHERE key = $key";
        command.Parameters.AddWithValue("$key", key);
        
        var result = command.ExecuteScalar();
        return result?.ToString() ?? defaultValue;
    }
    
    public bool GetBoolSetting(string key, bool defaultValue = false)
    {
        var value = GetSetting(key);
        if (value == null) return defaultValue;
        return value.Equals("true", StringComparison.OrdinalIgnoreCase);
    }
    
    public void SetSetting(string key, string value)
    {
        if (_connection == null) return;
        
        using var command = _connection.CreateCommand();
        command.CommandText = @"
            INSERT OR REPLACE INTO settings (key, value)
            VALUES ($key, $value)
        ";
        command.Parameters.AddWithValue("$key", key);
        command.Parameters.AddWithValue("$value", value);
        command.ExecuteNonQuery();
    }
    
    public void SetBoolSetting(string key, bool value)
    {
        SetSetting(key, value ? "true" : "false");
    }
    
    public Settings LoadSettings()
    {
        return new Settings
        {
            SpeedHalf = GetBoolSetting("speed_half", true),
            Speed1x = GetBoolSetting("speed_1x", true),
            Speed2x = GetBoolSetting("speed_2x", true),
            Speed100x = GetBoolSetting("speed_100x", true),
            RecordHotkey = GetSetting("record_hotkey", "F9") ?? "F9",
            PlayHotkey = GetSetting("play_hotkey", "F10") ?? "F10",
            AlwaysOnTop = GetBoolSetting("always_on_top", false),
            ShowCaptions = GetBoolSetting("show_captions", true),
            UseSendInput = GetBoolSetting("use_sendinput", true)
        };
    }
    
    public void SaveSettings(Settings settings)
    {
        SetBoolSetting("speed_half", settings.SpeedHalf);
        SetBoolSetting("speed_1x", settings.Speed1x);
        SetBoolSetting("speed_2x", settings.Speed2x);
        SetBoolSetting("speed_100x", settings.Speed100x);
        SetSetting("record_hotkey", settings.RecordHotkey);
        SetSetting("play_hotkey", settings.PlayHotkey);
        SetBoolSetting("always_on_top", settings.AlwaysOnTop);
        SetBoolSetting("show_captions", settings.ShowCaptions);
        SetBoolSetting("use_sendinput", settings.UseSendInput);
    }
    
    public void Dispose()
    {
        _connection?.Close();
        _connection?.Dispose();
        _connection = null;
    }
}
