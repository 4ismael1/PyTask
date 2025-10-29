import sqlite3
from pathlib import Path
import os


class SettingsDatabase:
    def __init__(self, db_path=None):
        """Inicializa la base de datos de configuración"""
        if db_path is None:
            # Usar AppData/Roaming/PyTask para almacenar la configuración
            app_data = Path(os.getenv('APPDATA')) / 'PyTask'
            app_data.mkdir(parents=True, exist_ok=True)
            self.db_path = str(app_data / 'pytask.db')
        else:
            self.db_path = db_path
        self.init_database()
    
    def init_database(self):
        """Crea la tabla de configuración si no existe"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        ''')
        
        conn.commit()
        conn.close()
    
    def get_setting(self, key, default=None):
        """Obtiene un valor de configuración"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('SELECT value FROM settings WHERE key = ?', (key,))
        row = cursor.fetchone()
        conn.close()
        
        if row:
            # Convertir strings a tipos apropiados
            value = row[0]
            if value.lower() == 'true':
                return True
            elif value.lower() == 'false':
                return False
            return value
        
        return default
    
    def set_setting(self, key, value):
        """Guarda un valor de configuración"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Convertir booleanos a strings
        if isinstance(value, bool):
            value = 'true' if value else 'false'
        
        cursor.execute('''
            INSERT OR REPLACE INTO settings (key, value)
            VALUES (?, ?)
        ''', (key, str(value)))
        
        conn.commit()
        conn.close()
    
    def get_all_settings(self):
        """Obtiene todas las configuraciones como diccionario - OPTIMIZADO"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('SELECT key, value FROM settings')
        rows = cursor.fetchall()
        conn.close()
        
        settings = {}
        for key, value in rows:
            # Convertir strings a tipos apropiados (optimizado)
            value_lower = value.lower()
            if value_lower == 'true':
                settings[key] = True
            elif value_lower == 'false':
                settings[key] = False
            else:
                # Intentar convertir a número si es posible
                try:
                    if '.' in value:
                        settings[key] = float(value)
                    else:
                        settings[key] = int(value)
                except ValueError:
                    settings[key] = value
        
        return settings
    
    def delete_setting(self, key):
        """Elimina una configuración"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('DELETE FROM settings WHERE key = ?', (key,))
        
        conn.commit()
        conn.close()
