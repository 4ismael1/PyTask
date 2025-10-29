import sys
from PyQt6.QtWidgets import (QApplication, QMainWindow, QWidget, QHBoxLayout,
                             QMenu, QToolButton, QInputDialog, QFileDialog)
from PyQt6.QtCore import Qt, QTimer, QSize, pyqtSignal
from PyQt6.QtGui import QAction, QIcon
# OPTIMIZACIÓN: Lazy imports - keyboard y threading se cargan cuando se necesiten
# import keyboard  # Se carga en setup_global_hotkeys()
# import threading  # Se carga en play_macro()
import json
from pathlib import Path
import ctypes

from macro_recorder import MacroRecorder, MacroPlayer, MacroPlayerWindows
from database import SettingsDatabase


class MainWindow(QMainWindow):
    # Señal para actualizar el botón desde cualquier thread
    playback_finished_signal = pyqtSignal()
    
    def __init__(self):
        super().__init__()
        self.recorder = MacroRecorder()
        # Player se inicializará después de cargar preferencias
        self.player = None
        
        # Inicializar base de datos de configuración en AppData
        self.db = SettingsDatabase()  # Usa AppData/Roaming/PyTask por defecto
        
        # Conectar la señal
        self.playback_finished_signal.connect(self.on_playback_finished)
        
        self.is_recording = False
        self.is_playing = False
        self.current_macro_events = []
        self.current_macro_file = None
        self.play_speed = 1.0
        self.custom_speed = 8.0
        self.playback_loops = 1
        self.interval_mode = False
        self.interval_seconds = 5
        
        # Cache para menús (optimización)
        self._play_menu_cache = None
        self._prefs_menu_cache = None
        self._menu_needs_update = True
        
        # Cache para iconos (optimización - evita cargar desde disco)
        self._icon_cache = {}
        self._icon_path = Path(__file__).parent / "img"
        
        # Configuración por defecto
        self.settings = {
            'speed_half': True,
            'speed_1x': True,
            'speed_2x': True,
            'speed_100x': True,
            'record_hotkey': 'f9',
            'play_hotkey': 'f10',
            'always_on_top': False,
            'show_captions': True,
            'use_sendinput': True  # Usar Windows SendInput (mejor compatibilidad con juegos)
        }
        self.load_preferences()
        
        self.init_ui()
        self.setup_global_hotkeys()
    
    def get_icon(self, icon_name):
        """Obtiene un icono del caché o lo carga si no existe - OPTIMIZADO"""
        if icon_name not in self._icon_cache:
            icon_file = self._icon_path / icon_name
            if icon_file.exists():
                self._icon_cache[icon_name] = QIcon(str(icon_file))
            else:
                self._icon_cache[icon_name] = QIcon()  # Icono vacío si no existe
        return self._icon_cache[icon_name]
    
    def init_ui(self):
        self.setWindowTitle("PyTask")
        self.setFixedSize(350, 110)  # Más compacta aún
        
        # Establecer icono de la aplicación
        if (self._icon_path / "portapapeles.png").exists():
            self.setWindowIcon(self.get_icon("portapapeles.png"))
        
        # Forzar icono en la barra de tareas de Windows
        try:
            import ctypes
            myappid = 'pytask.automation.app.1.0'  # ID único de la app
            ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID(myappid)
        except:
            pass
        
        # Always on top
        if self.settings.get('always_on_top', False):
            self.setWindowFlags(self.windowFlags() | Qt.WindowType.WindowStaysOnTopHint)
        
        # Cambiar color de la barra de título en Windows a blanco
        try:
            hwnd = int(self.winId())
            # DWMWA_USE_IMMERSIVE_DARK_MODE = 20 (para Windows 10/11)
            # 0 = modo claro (barra blanca), 1 = modo oscuro
            DWMWA_USE_IMMERSIVE_DARK_MODE = 20
            value = ctypes.c_int(0)  # 0 = modo claro
            ctypes.windll.dwmapi.DwmSetWindowAttribute(
                hwnd, 
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                ctypes.byref(value),
                ctypes.sizeof(value)
            )
        except:
            pass  # Si falla, continuar sin cambiar la barra
        
        # Estilo mejorado - interfaz profesional con iconos y barra blanca
        # OPTIMIZADO: Estilos más simples y eficientes
        self.setStyleSheet("""
            QMainWindow {
                background-color: #ffffff;
            }
            QToolButton {
                background-color: #ffffff;
                border: 2px solid #d0d0d0;
                border-radius: 8px;
                padding: 8px 4px;
                margin: 3px;
                font-size: 9px;
                color: #333333;
                font-weight: bold;
            }
            QToolButton:hover {
                background-color: #e8f4ff;
                border-color: #0078d7;
                color: #0078d7;
            }
            QToolButton:pressed {
                background-color: #cce4f7;
            }
            QToolButton:disabled {
                background-color: #f8f8f8;
                color: #aaaaaa;
            }
            QToolButton:checked {
                background-color: #ff4444;
                border-color: #cc0000;
                color: #ffffff;
            }
            QMenu {
                background-color: #ffffff;
                border: 1px solid #d0d0d0;
                font-size: 11px;
                color: #333333;
            }
            QMenu::item {
                padding: 8px 35px 8px 25px;
                border-radius: 3px;
                margin: 2px 5px;
                color: #333333;
            }
            QMenu::item:selected {
                background-color: #0078d7;
                color: #ffffff;
            }
            QMenu::item:disabled {
                color: #999999;
            }
            QMenu::separator {
                height: 1px;
                background: #e0e0e0;
                margin: 5px 10px;
            }
            QStatusBar {
                background-color: #ffffff;
                color: #333333;
                font-size: 10px;
                border-top: 1px solid #d0d0d0;
            }
        """)
        
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Layout principal horizontal
        main_layout = QHBoxLayout()
        main_layout.setSpacing(3)
        main_layout.setContentsMargins(5, 5, 5, 5)
        
        # Botón Open
        self.open_button = QToolButton()
        self.open_button.setIcon(self.get_icon("abrir-documento.png"))
        self.open_button.setText("Open")
        self.open_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
        self.open_button.setFixedSize(62, 68)
        self.open_button.setIconSize(QSize(28, 28))
        self.open_button.clicked.connect(self.open_macro)
        
        # Botón Save
        self.save_button = QToolButton()
        self.save_button.setIcon(self.get_icon("Save.png"))
        self.save_button.setText("Save")
        self.save_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
        self.save_button.setFixedSize(62, 68)
        self.save_button.setIconSize(QSize(28, 28))
        self.save_button.setEnabled(False)
        self.save_button.clicked.connect(self.save_macro)
        
        # Botón Rec (F9)
        self.rec_button = QToolButton()
        self.rec_button.setIcon(self.get_icon("boton-detener.png"))
        self.rec_button.setText("Rec")
        self.rec_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
        self.rec_button.setFixedSize(62, 68)
        self.rec_button.setIconSize(QSize(28, 28))
        self.rec_button.setCheckable(True)
        self.rec_button.clicked.connect(self.toggle_recording)
        
        # Botón Play (F10) con menú
        self.play_button = QToolButton()
        self.play_button.setIcon(self.get_icon("Play.png"))
        self.play_button.setText("Play")
        self.play_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
        self.play_button.setFixedSize(62, 68)
        self.play_button.setIconSize(QSize(28, 28))
        self.play_button.setEnabled(False)
        self.play_button.clicked.connect(self.toggle_playback)
        
        # Crear menú Play LAZY (solo cuando se necesite)
        self.play_button.setPopupMode(QToolButton.ToolButtonPopupMode.MenuButtonPopup)
        
        # Conectar evento para recrear menú solo cuando sea necesario
        def on_menu_about_to_show():
            if self._menu_needs_update:
                self.create_play_menu()
        
        # Crear menú inicial
        self.create_play_menu()
        if self.play_button.menu():
            self.play_button.menu().aboutToShow.connect(on_menu_about_to_show)
        
        # Botón Prefs
        self.prefs_button = QToolButton()
        self.prefs_button.setIcon(self.get_icon("preferencias.png"))
        self.prefs_button.setText("Prefs")
        self.prefs_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
        self.prefs_button.setFixedSize(62, 68)
        self.prefs_button.setIconSize(QSize(28, 28))
        self.prefs_button.clicked.connect(self.show_prefs_menu)
        
        # Añadir botones (sin el botón .exe)
        main_layout.addWidget(self.open_button)
        main_layout.addWidget(self.save_button)
        main_layout.addWidget(self.rec_button)
        main_layout.addWidget(self.play_button)
        main_layout.addWidget(self.prefs_button)
        
        central_widget.setLayout(main_layout)
        
        # Status bar
        if self.settings.get('show_captions', True):
            self.update_status_bar_default()
        else:
            self.statusBar().hide()
    
    def create_play_menu(self):
        """Crea el menú del botón Play - OPTIMIZADO con caché"""
        # Si el menú ya existe y no necesita actualización, reutilizarlo
        if self._play_menu_cache and not self._menu_needs_update:
            return self._play_menu_cache
        
        # Crear nuevo menú
        play_menu = QMenu(self)
        
        # Pre-calcular estados para evitar múltiples evaluaciones
        mode_once = (not self.interval_mode and self.playback_loops == 1)
        mode_infinite = (not self.interval_mode and self.playback_loops == 0)
        mode_interval = self.interval_mode
        
        # === VELOCIDADES ===
        play_menu.addAction("━━━ VELOCIDAD ━━━").setEnabled(False)
        
        if self.settings.get('speed_half', True):
            play_menu.addAction("● ½x" if self.play_speed == 0.5 else "   ½x", lambda: self.set_play_speed(0.5))
        
        if self.settings.get('speed_1x', True):
            play_menu.addAction("● 1x" if self.play_speed == 1.0 else "   1x", lambda: self.set_play_speed(1.0))
        
        if self.settings.get('speed_2x', True):
            play_menu.addAction("● 2x" if self.play_speed == 2.0 else "   2x", lambda: self.set_play_speed(2.0))
        
        if self.settings.get('speed_100x', True):
            play_menu.addAction("● 100x" if self.play_speed == 100.0 else "   100x", lambda: self.set_play_speed(100.0))
        
        play_menu.addAction(f"● {self.custom_speed}x" if self.play_speed == self.custom_speed else f"   {self.custom_speed}x", lambda: self.set_play_speed(self.custom_speed))
        play_menu.addAction("   Cambiar velocidad...", self.set_custom_speed)
        
        play_menu.addSeparator()
        
        # === REPETICIONES ===
        play_menu.addAction("━━━ REPETICIONES ━━━").setEnabled(False)
        play_menu.addAction("● 1 vez" if mode_once else "   1 vez", self.set_mode_once)
        play_menu.addAction("● Infinito" if mode_infinite else "   Infinito", self.set_mode_infinite)
        
        play_menu.addSeparator()
        
        # === Intervalo (simplificado) ===
        interval_menu = play_menu.addMenu("⏱ Con pausa")
        
        if mode_interval:
            if self.playback_loops == 0:
                interval_menu.addAction(f"✓ Cada {self.interval_seconds}s (∞)").setEnabled(False)
            else:
                interval_menu.addAction(f"✓ Cada {self.interval_seconds}s ({self.playback_loops}x)").setEnabled(False)
            interval_menu.addSeparator()
        
        interval_menu.addAction("Configurar...", self.configure_interval_mode)
        interval_menu.addSeparator()
        
        # Atajos rápidos más simples
        interval_menu.addAction("● 5s ∞" if (mode_interval and self.playback_loops == 0 and self.interval_seconds == 5) else "   5s ∞", lambda: self.set_interval_quick(5, 0))
        interval_menu.addAction("● 10s ∞" if (mode_interval and self.playback_loops == 0 and self.interval_seconds == 10) else "   10s ∞", lambda: self.set_interval_quick(10, 0))
        interval_menu.addAction("● 5s x3" if (mode_interval and self.playback_loops == 3 and self.interval_seconds == 5) else "   5s x3", lambda: self.set_interval_quick(5, 3))
        interval_menu.addAction("● 10s x5" if (mode_interval and self.playback_loops == 5 and self.interval_seconds == 10) else "   10s x5", lambda: self.set_interval_quick(10, 5))
        
        play_menu.addSeparator()
        
        # === HOTKEYS ===
        play_menu.addAction("━━━ TECLAS ━━━").setEnabled(False)
        
        rec_hotkey_menu = play_menu.addMenu(f"⌨ Grabar ({self.settings['record_hotkey'].upper()})")
        for key in ['F6', 'F7', 'F8', 'F9']:
            rec_hotkey_menu.addAction(f"● {key}" if self.settings['record_hotkey'].lower() == key.lower() else f"   {key}", lambda k=key: self.set_recording_hotkey(k.lower()))
        
        play_hotkey_menu = play_menu.addMenu(f"⌨ Reproducir ({self.settings['play_hotkey'].upper()})")
        for key in ['F5', 'F10', 'F11', 'F12']:
            play_hotkey_menu.addAction(f"● {key}" if self.settings['play_hotkey'].lower() == key.lower() else f"   {key}", lambda k=key: self.set_playback_hotkey(k.lower()))
        
        play_menu.addSeparator()
        
        # === OPCIONES ===
        play_menu.addAction("━━━ OPCIONES ━━━").setEnabled(False)
        
        always_on_top_action = play_menu.addAction("✓ Siempre visible" if self.settings.get('always_on_top', False) else "   Siempre visible")
        always_on_top_action.triggered.connect(self.toggle_always_on_top)
        
        show_captions_action = play_menu.addAction("✓ Barra de estado" if self.settings.get('show_captions', True) else "   Barra de estado")
        show_captions_action.triggered.connect(self.toggle_show_captions)
        
        # Guardar en caché
        self._play_menu_cache = play_menu
        self._menu_needs_update = False
        
        # Asignar al botón
        self.play_button.setMenu(play_menu)
        return play_menu
    
    def show_prefs_menu(self):
        """Muestra menú de preferencias completo - OPTIMIZADO"""
        # Reutilizar menú si existe
        if not self._prefs_menu_cache:
            self._prefs_menu_cache = QMenu(self)
        
        prefs_menu = self._prefs_menu_cache
        prefs_menu.clear()  # Limpiar y reconstruir rápido
        
        # Velocidades habilitadas
        speed_menu = prefs_menu.addMenu("Velocidades")
        
        half_action = speed_menu.addAction("½x")
        half_action.setCheckable(True)
        half_action.setChecked(self.settings.get('speed_half', True))
        half_action.triggered.connect(lambda: self.toggle_speed_option('speed_half'))
        
        one_action = speed_menu.addAction("1x")
        one_action.setCheckable(True)
        one_action.setChecked(self.settings.get('speed_1x', True))
        one_action.triggered.connect(lambda: self.toggle_speed_option('speed_1x'))
        
        two_action = speed_menu.addAction("2x")
        two_action.setCheckable(True)
        two_action.setChecked(self.settings.get('speed_2x', True))
        two_action.triggered.connect(lambda: self.toggle_speed_option('speed_2x'))
        
        hundred_action = speed_menu.addAction("100x")
        hundred_action.setCheckable(True)
        hundred_action.setChecked(self.settings.get('speed_100x', True))
        hundred_action.triggered.connect(lambda: self.toggle_speed_option('speed_100x'))
        
        prefs_menu.addSeparator()
        
        # Recording Hotkey
        rec_hotkey_menu = prefs_menu.addMenu(f"Tecla Grabar ({self.settings['record_hotkey'].upper()})")
        for key in ['F6', 'F7', 'F8', 'F9']:
            action = rec_hotkey_menu.addAction(key, lambda k=key: self.set_recording_hotkey(k.lower()))
            if self.settings['record_hotkey'].lower() == key.lower():
                action.setCheckable(True)
                action.setChecked(True)
        
        # Playback Hotkey
        play_hotkey_menu = prefs_menu.addMenu(f"Tecla Reproducir ({self.settings['play_hotkey'].upper()})")
        for key in ['F10', 'F11', 'F12', 'F5']:
            action = play_hotkey_menu.addAction(key, lambda k=key: self.set_playback_hotkey(k.lower()))
            if self.settings['play_hotkey'].lower() == key.lower():
                action.setCheckable(True)
                action.setChecked(True)
        
        prefs_menu.addSeparator()
        
        # Always on Top
        always_on_top_action = prefs_menu.addAction("Siempre Visible")
        always_on_top_action.setCheckable(True)
        always_on_top_action.setChecked(self.settings.get('always_on_top', False))
        always_on_top_action.triggered.connect(self.toggle_always_on_top)
        
        # Show Captions
        show_captions_action = prefs_menu.addAction("Barra de Estado")
        show_captions_action.setCheckable(True)
        show_captions_action.setChecked(self.settings.get('show_captions', True))
        show_captions_action.triggered.connect(self.toggle_show_captions)
        
        # SendInput para juegos
        sendinput_action = prefs_menu.addAction("Modo Juegos (SendInput)")
        sendinput_action.setCheckable(True)
        sendinput_action.setChecked(self.settings.get('use_sendinput', True))
        sendinput_action.triggered.connect(self.toggle_sendinput)
        
        prefs_menu.addSeparator()
        prefs_menu.addAction("PyTask v1.1.0", self.show_about)
        
        prefs_menu.exec(self.prefs_button.mapToGlobal(self.prefs_button.rect().bottomLeft()))
    
    def show_status(self, message):
        """Muestra mensaje en status bar"""
        if self.settings.get('show_captions', True):
            self.statusBar().showMessage(message)
    
    def update_status_bar_default(self):
        """Actualiza la barra de estado con el mensaje predeterminado"""
        if self.settings.get('show_captions', True):
            self.statusBar().showMessage(
                f"Listo | Grabar: {self.settings['record_hotkey'].upper()} | Reproducir: {self.settings['play_hotkey'].upper()}"
            )
    
    def toggle_recording(self):
        """F9 - Inicia o detiene la grabación"""
        if not self.is_recording:
            # Verificar que no se esté reproduciendo
            if self.is_playing:
                self.show_status("No se puede grabar mientras se reproduce")
                return
            
            # Iniciar grabación - limpiar archivo actual para forzar "Guardar Como"
            self.current_macro_file = None
            self.is_recording = True
            self.rec_button.setChecked(True)
            self.play_button.setEnabled(False)  # Deshabilitar play mientras graba
            self.show_status(f"Grabando... (Presiona {self.settings['record_hotkey'].upper()} para detener)")
            QTimer.singleShot(50, lambda: self.recorder.start_recording())
        else:
            # Detener grabación
            self.is_recording = False
            self.current_macro_events = self.recorder.stop_recording()
            self.rec_button.setChecked(False)
            
            if self.current_macro_events:
                self.save_button.setEnabled(True)
                self.play_button.setEnabled(True)
                self.setWindowTitle("PyTask")  # Resetear título a sin archivo
                self.show_status(f"Grabados {len(self.current_macro_events)} eventos")
    
    def toggle_playback(self):
        """F10 - Inicia o detiene la reproducción"""
        if not self.is_playing:
            self.play_macro(self.play_speed)
        else:
            self.stop_playback()
    
    def update_play_button_state(self, playing):
        """Actualiza el estado visual del botón Play - OPTIMIZADO con caché de iconos"""
        if playing:
            # Cambiar a rojo cuando está reproduciendo
            self.play_button.setIcon(self.get_icon("boton-detener.png"))
            self.play_button.setStyleSheet("""
                QToolButton {
                    background-color: #ff4444;
                    border: 3px solid #cc0000;
                    border-radius: 8px;
                    padding: 8px 4px;
                    margin: 3px;
                    font-size: 9px;
                    color: #ffffff;
                    font-weight: bold;
                }
                QToolButton:hover {
                    background-color: #ff6666;
                    border: 3px solid #ff0000;
                }
            """)
            self.play_button.setText("Stop")
        else:
            # Volver a color normal
            self.play_button.setIcon(self.get_icon("Play.png"))
            
            self.play_button.setStyleSheet("""
                QToolButton {
                    background-color: #ffffff;
                    border: 2px solid #d0d0d0;
                    border-radius: 8px;
                    padding: 8px 4px;
                    margin: 3px;
                    font-size: 9px;
                    color: #333333;
                    font-weight: bold;
                }
                QToolButton:hover {
                    background-color: #e8f4ff;
                    border-color: #0078d7;
                    color: #0078d7;
                }
                QToolButton:disabled {
                    background-color: #f8f8f8;
                    color: #aaaaaa;
                }
            """)
            self.play_button.setText("Play")
    
    def open_macro(self):
        """Abre un archivo de macro"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Abrir Macro", "", "Archivos de Macro (*.macro);;Todos los Archivos (*)"
        )
        
        if file_path:
            try:
                with open(file_path, 'r') as f:
                    data = json.load(f)
                    self.current_macro_events = data.get('events', [])
                    self.current_macro_file = file_path
                
                if self.current_macro_events:
                    self.save_button.setEnabled(True)
                    self.play_button.setEnabled(True)
                    self.setWindowTitle(f"PyTask - {Path(file_path).name}")
                    self.show_status(f"Cargado: {len(self.current_macro_events)} eventos")
            except Exception as e:
                self.show_status(f"Error al cargar macro")
    
    def save_macro(self):
        """Guarda la macro actual"""
        if self.current_macro_file:
            self.save_to_file(self.current_macro_file)
        else:
            file_path, _ = QFileDialog.getSaveFileName(
                self, "Guardar Macro", "", "Archivos de Macro (*.macro)"
            )
            
            if file_path:
                if not file_path.endswith('.macro'):
                    file_path += '.macro'
                self.save_to_file(file_path)
                self.current_macro_file = file_path
                self.setWindowTitle(f"PyTask - {Path(file_path).name}")
    
    def save_to_file(self, file_path):
        """Guarda eventos en archivo"""
        try:
            data = {'events': self.current_macro_events}
            with open(file_path, 'w') as f:
                json.dump(data, f, indent=2)
            self.show_status(f"Guardado: {Path(file_path).name}")
        except:
            self.show_status("Error al guardar macro")
    
    def set_play_speed(self, speed):
        """Establece la velocidad de reproducción"""
        self.play_speed = speed
        self._menu_needs_update = True
        self.show_status(f"Velocidad: {speed}x")
    
    def set_custom_speed(self):
        """Establece velocidad personalizada"""
        speed, ok = QInputDialog.getDouble(
            self, "Velocidad Personalizada", 
            "Multiplicador de velocidad:",
            self.custom_speed, 0.1, 1000.0, 1
        )
        if ok:
            self.custom_speed = speed
            self._menu_needs_update = True
            self.show_status(f"Velocidad personalizada: {speed}x")
    
    def set_mode_once(self):
        """Modo: Reproducir 1 vez"""
        self.interval_mode = False
        self.playback_loops = 1
        self._menu_needs_update = True
        self.show_status("Modo: Reproducir 1 vez")
    
    def set_mode_infinite(self):
        """Modo: Infinito sin pausa"""
        self.interval_mode = False
        self.playback_loops = 0
        self._menu_needs_update = True
        self.show_status("Modo: Reproducción INFINITA (sin pausa)")
    
    def set_interval_quick(self, seconds, loops):
        """Atajo rápido para configurar intervalo"""
        self.interval_mode = True
        self.interval_seconds = seconds
        self.playback_loops = loops
        self._menu_needs_update = True
        if loops == 0:
            self.show_status(f"Modo: Cada {seconds}s (INFINITO)")
        else:
            self.show_status(f"Modo: {loops} veces cada {seconds}s")
    
    def configure_interval_mode(self):
        """Configura el modo de intervalo"""
        from PyQt6.QtWidgets import QDialog, QVBoxLayout, QLabel, QPushButton, QSpinBox, QRadioButton, QButtonGroup, QHBoxLayout, QGroupBox
        
        dialog = QDialog(self)
        dialog.setWindowTitle("Configurar Pausa de Tiempo")
        dialog.setFixedWidth(420)
        
        layout = QVBoxLayout()
        layout.setSpacing(10)
        layout.setContentsMargins(15, 15, 15, 15)
        
        # Título
        title = QLabel("Reproducir con pausa de tiempo")
        title.setStyleSheet("font-size: 14px; font-weight: bold; padding: 5px;")
        layout.addWidget(title)
        
        # === GRUPO 1: TIEMPO DE PAUSA ===
        time_group_box = QGroupBox("Tiempo de pausa entre cada ejecución")
        time_group_box.setStyleSheet("QGroupBox { font-weight: bold; }")
        time_layout = QVBoxLayout()
        
        time_group = QButtonGroup(dialog)
        
        time_5s = QRadioButton("5 segundos")
        time_group.addButton(time_5s)
        time_layout.addWidget(time_5s)
        
        time_10s = QRadioButton("10 segundos")
        time_group.addButton(time_10s)
        time_layout.addWidget(time_10s)
        
        time_30s = QRadioButton("30 segundos")
        time_group.addButton(time_30s)
        time_layout.addWidget(time_30s)
        
        time_60s = QRadioButton("60 segundos (1 minuto)")
        time_group.addButton(time_60s)
        time_layout.addWidget(time_60s)
        
        # Tiempo personalizado
        time_custom_layout = QHBoxLayout()
        time_custom = QRadioButton("Personalizado:")
        time_group.addButton(time_custom)
        time_custom_layout.addWidget(time_custom)
        
        time_spin = QSpinBox()
        time_spin.setRange(1, 3600)
        time_spin.setValue(self.interval_seconds if self.interval_seconds not in [5, 10, 30, 60] else 15)
        time_spin.setSuffix(" segundos")
        time_spin.setEnabled(False)
        time_spin.setFixedWidth(150)
        time_custom_layout.addWidget(time_spin)
        time_custom_layout.addStretch()
        
        time_layout.addLayout(time_custom_layout)
        
        # Conectar radio button personalizado
        time_custom.toggled.connect(lambda checked: time_spin.setEnabled(checked))
        
        # Seleccionar el tiempo actual
        if self.interval_seconds == 5:
            time_5s.setChecked(True)
        elif self.interval_seconds == 10:
            time_10s.setChecked(True)
        elif self.interval_seconds == 30:
            time_30s.setChecked(True)
        elif self.interval_seconds == 60:
            time_60s.setChecked(True)
        else:
            time_custom.setChecked(True)
            time_spin.setValue(self.interval_seconds)
            time_spin.setEnabled(True)
        
        time_group_box.setLayout(time_layout)
        layout.addWidget(time_group_box)
        
        # === GRUPO 2: CANTIDAD DE REPETICIONES ===
        repeat_group_box = QGroupBox("Cantidad de veces a reproducir")
        repeat_group_box.setStyleSheet("QGroupBox { font-weight: bold; }")
        repeat_layout = QVBoxLayout()
        
        repeat_group = QButtonGroup(dialog)
        
        repeat_infinite = QRadioButton("Infinito (hasta detener manualmente)")
        repeat_group.addButton(repeat_infinite)
        repeat_layout.addWidget(repeat_infinite)
        
        repeat_3 = QRadioButton("3 veces")
        repeat_group.addButton(repeat_3)
        repeat_layout.addWidget(repeat_3)
        
        repeat_5 = QRadioButton("5 veces")
        repeat_group.addButton(repeat_5)
        repeat_layout.addWidget(repeat_5)
        
        repeat_10 = QRadioButton("10 veces")
        repeat_group.addButton(repeat_10)
        repeat_layout.addWidget(repeat_10)
        
        # Cantidad personalizada
        repeat_custom_layout = QHBoxLayout()
        repeat_custom = QRadioButton("Personalizado:")
        repeat_group.addButton(repeat_custom)
        repeat_custom_layout.addWidget(repeat_custom)
        
        repeat_spin = QSpinBox()
        repeat_spin.setRange(1, 10000)
        repeat_spin.setValue(self.playback_loops if self.playback_loops not in [0, 3, 5, 10] else 20)
        repeat_spin.setSuffix(" veces")
        repeat_spin.setEnabled(False)
        repeat_spin.setFixedWidth(150)
        repeat_custom_layout.addWidget(repeat_spin)
        repeat_custom_layout.addStretch()
        
        repeat_layout.addLayout(repeat_custom_layout)
        
        # Conectar radio button personalizado
        repeat_custom.toggled.connect(lambda checked: repeat_spin.setEnabled(checked))
        
        # Seleccionar la cantidad actual
        if self.playback_loops == 0:
            repeat_infinite.setChecked(True)
        elif self.playback_loops == 3:
            repeat_3.setChecked(True)
        elif self.playback_loops == 5:
            repeat_5.setChecked(True)
        elif self.playback_loops == 10:
            repeat_10.setChecked(True)
        else:
            repeat_custom.setChecked(True)
            repeat_spin.setValue(self.playback_loops)
            repeat_spin.setEnabled(True)
        
        repeat_group_box.setLayout(repeat_layout)
        layout.addWidget(repeat_group_box)
        
        # Botones
        btn_layout = QHBoxLayout()
        btn_layout.addStretch()
        
        btn_ok = QPushButton("Aplicar")
        btn_ok.setStyleSheet("font-weight: bold; padding: 8px 20px; background-color: #0078d7; color: white; border: none; border-radius: 3px;")
        btn_cancel = QPushButton("Cancelar")
        btn_cancel.setStyleSheet("padding: 8px 20px; border: 1px solid #999; border-radius: 3px;")
        
        btn_ok.clicked.connect(dialog.accept)
        btn_cancel.clicked.connect(dialog.reject)
        
        btn_layout.addWidget(btn_ok)
        btn_layout.addWidget(btn_cancel)
        layout.addLayout(btn_layout)
        
        dialog.setLayout(layout)
        
        if dialog.exec() == QDialog.DialogCode.Accepted:
            self.interval_mode = True
            
            # Determinar tiempo seleccionado
            if time_5s.isChecked():
                self.interval_seconds = 5
            elif time_10s.isChecked():
                self.interval_seconds = 10
            elif time_30s.isChecked():
                self.interval_seconds = 30
            elif time_60s.isChecked():
                self.interval_seconds = 60
            else:
                self.interval_seconds = time_spin.value()
            
            # Determinar cantidad seleccionada
            if repeat_infinite.isChecked():
                self.playback_loops = 0
                self.show_status(f"Modo: Cada {self.interval_seconds}s (INFINITO)")
            elif repeat_3.isChecked():
                self.playback_loops = 3
                self.show_status(f"Modo: 3 veces cada {self.interval_seconds}s")
            elif repeat_5.isChecked():
                self.playback_loops = 5
                self.show_status(f"Modo: 5 veces cada {self.interval_seconds}s")
            elif repeat_10.isChecked():
                self.playback_loops = 10
                self.show_status(f"Modo: 10 veces cada {self.interval_seconds}s")
            else:
                self.playback_loops = repeat_spin.value()
                self.show_status(f"Modo: {self.playback_loops} veces cada {self.interval_seconds}s")
            
            self._menu_needs_update = True
    
    def toggle_interval_mode(self):
        """Activa/desactiva modo de intervalo"""
        self.interval_mode = not self.interval_mode
        self._menu_needs_update = True
        self.show_status(f"Modo intervalo: {'ON' if self.interval_mode else 'OFF'} ({self.interval_seconds}s)")
    
    def set_interval_time(self):
        """Establece el tiempo de intervalo entre reproducciones"""
        seconds, ok = QInputDialog.getInt(
            self, "Establecer Intervalo",
            "Segundos de espera entre cada reproducción:",
            self.interval_seconds, 1, 3600
        )
        if ok:
            self.interval_seconds = seconds
            self._menu_needs_update = True
            self.show_status(f"Intervalo: {seconds} segundos")
    
    def set_playback_loops(self):
        """Establece el número de loops"""
        loops, ok = QInputDialog.getInt(
            self, "Repeticiones",
            "Veces a reproducir:\n0 = Infinito\n1 = Una vez\n2+ = N veces",
            self.playback_loops, 0, 10000
        )
        if ok:
            self.playback_loops = loops
            self._menu_needs_update = True
            if loops == 0:
                self.show_status(f"Repeticiones: INFINITO")
            else:
                self.show_status(f"Repeticiones: {loops} vez/veces")
    
    def set_recording_hotkey(self, key):
        """Establece hotkey de grabación"""
        old_key = self.settings['record_hotkey']
        self.settings['record_hotkey'] = key.lower()
        self.save_preferences()
        self.setup_global_hotkeys()
        self._menu_needs_update = True
        self.show_status(f"Tecla de grabación cambiada: {old_key.upper()} → {key.upper()}")
        # Actualizar status bar al mensaje predeterminado después de 2 segundos
        QTimer.singleShot(2000, self.update_status_bar_default)
    
    def set_playback_hotkey(self, key):
        """Establece hotkey de reproducción"""
        old_key = self.settings['play_hotkey']
        self.settings['play_hotkey'] = key.lower()
        self.save_preferences()
        self.setup_global_hotkeys()
        self._menu_needs_update = True
        self.show_status(f"Tecla de reproducción cambiada: {old_key.upper()} → {key.upper()}")
        # Actualizar status bar al mensaje predeterminado después de 2 segundos
        QTimer.singleShot(2000, self.update_status_bar_default)
    
    def toggle_always_on_top(self):
        """Activa/desactiva always on top"""
        self.settings['always_on_top'] = not self.settings.get('always_on_top', False)
        self.save_preferences()
        
        if self.settings['always_on_top']:
            self.setWindowFlags(self.windowFlags() | Qt.WindowType.WindowStaysOnTopHint)
        else:
            self.setWindowFlags(self.windowFlags() & ~Qt.WindowType.WindowStaysOnTopHint)
        
        self.show()
        self._menu_needs_update = True
        self.show_status(f"Siempre visible: {'ON' if self.settings['always_on_top'] else 'OFF'}")
    
    def toggle_show_captions(self):
        """Activa/desactiva captions"""
        self.settings['show_captions'] = not self.settings.get('show_captions', True)
        self.save_preferences()
        
        if self.settings['show_captions']:
            self.statusBar().show()
        else:
            self.statusBar().hide()
        
        self._menu_needs_update = True
    
    def toggle_sendinput(self):
        """Cambia entre SendInput (juegos) y pynput (normal)"""
        self.settings['use_sendinput'] = not self.settings.get('use_sendinput', True)
        self.save_preferences()
        
        # Reinicializar el player
        if self.settings['use_sendinput']:
            self.player = MacroPlayerWindows()
            self.show_status("✓ Modo Juegos activado (SendInput)")
        else:
            self.player = MacroPlayer()
            self.show_status("✓ Modo Normal activado (pynput)")
        
        self._menu_needs_update = True
    
    def toggle_speed_option(self, speed_key):
        """Activa/desactiva una opción de velocidad"""
        self.settings[speed_key] = not self.settings.get(speed_key, True)
        self.save_preferences()
        self._menu_needs_update = True
        self.show_status(f"Opción de velocidad {speed_key} cambiada")
    
    def show_about(self):
        """Muestra información sobre PyTask"""
        self.show_status("PyTask v1.1.0 - Automatización Avanzada | Modo Juegos ✓ | GitHub: 4ismael1")
    
    def play_macro(self, speed):
        """Reproduce la macro"""
        if not self.current_macro_events:
            self.show_status("No hay macro cargada")
            return
        
        if self.is_playing:
            return
        
        # OPTIMIZACIÓN: Lazy import de threading
        import threading
        
        self.is_playing = True
        self.rec_button.setEnabled(False)  # Deshabilitar grabación mientras reproduce
        self.update_play_button_state(True)
        
        def playback_thread():
            try:
                loops_done = 0
                import time
                
                # Determinar modo
                is_infinite = (self.playback_loops == 0)
                
                # CON INTERVALO DE TIEMPO
                if self.interval_mode:
                    if is_infinite:
                        mode_text = f"INTERVALO {self.interval_seconds}s (INFINITO)"
                    else:
                        mode_text = f"INTERVALO {self.interval_seconds}s ({self.playback_loops} veces)"
                    
                    play_hotkey = self.settings['play_hotkey'].upper()
                    QTimer.singleShot(0, lambda: self.show_status(f"{mode_text} a {speed}x ({play_hotkey}=Detener)"))
                    
                    while self.is_playing:
                        # Verificar antes de reproducir
                        if not self.is_playing:
                            break
                            
                        # Reproducir la macro
                        self.player.play_macro(self.current_macro_events, speed)
                        
                        # Esperar a que termine la reproducción o se detenga
                        while self.player.playing and self.is_playing:
                            time.sleep(0.1)
                        
                        loops_done += 1
                        
                        # Mostrar progreso
                        if is_infinite:
                            status_msg = f"INTERVALO {self.interval_seconds}s | Rep. {loops_done} (INFINITO) a {speed}x | {play_hotkey}=Detener"
                        else:
                            status_msg = f"INTERVALO {self.interval_seconds}s | Rep. {loops_done}/{self.playback_loops} a {speed}x | {play_hotkey}=Detener"
                        
                        QTimer.singleShot(0, lambda msg=status_msg: self.show_status(msg))
                        
                        # Si no es infinito y ya completamos, salir
                        if not is_infinite and loops_done >= self.playback_loops:
                            break
                        
                        # Esperar el intervalo (interruptible)
                        for i in range(self.interval_seconds):
                            if not self.is_playing:
                                break
                            time.sleep(1)
                
                # SIN INTERVALO
                else:
                    if is_infinite:
                        mode_text = "INFINITO (sin pausa)"
                    elif self.playback_loops == 1:
                        mode_text = "UNA VEZ"
                    else:
                        mode_text = f"{self.playback_loops} VECES (sin pausa)"
                    
                    play_hotkey = self.settings['play_hotkey'].upper()
                    QTimer.singleShot(0, lambda: self.show_status(f"{mode_text} a {speed}x ({play_hotkey}=Detener)"))
                    
                    while self.is_playing:
                        # Verificar antes de reproducir
                        if not self.is_playing:
                            break
                            
                        # Reproducir la macro
                        self.player.play_macro(self.current_macro_events, speed)
                        
                        # Esperar a que termine la reproducción o se detenga
                        while self.player.playing and self.is_playing:
                            time.sleep(0.1)
                        
                        loops_done += 1
                        
                        # Mostrar progreso
                        if is_infinite:
                            status_msg = f"INFINITO | Rep. {loops_done} a {speed}x | {play_hotkey}=Detener"
                        elif self.playback_loops == 1:
                            status_msg = f"Completada 1 vez a {speed}x"
                        else:
                            status_msg = f"Rep. {loops_done}/{self.playback_loops} a {speed}x | {play_hotkey}=Detener"
                        
                        QTimer.singleShot(0, lambda msg=status_msg: self.show_status(msg))
                        
                        # Si no es infinito y ya completamos, salir
                        if not is_infinite and loops_done >= self.playback_loops:
                            break
                        
                        # Pausa pequeña
                        time.sleep(0.05)
                
                # Terminar reproducción - USAR SEÑAL para thread-safety
                self.is_playing = False
                self.player.stop_playback()
                final_loops = loops_done
                
                # Emitir señal para actualizar UI (thread-safe)
                self.playback_finished_signal.emit()
                QTimer.singleShot(100, lambda fl=final_loops: self.show_status(f"Completadas {fl} repeticiones"))
            except Exception as e:
                self.is_playing = False
                self.player.stop_playback()
                
                # Emitir señal para actualizar UI (thread-safe)
                self.playback_finished_signal.emit()
                error_msg = str(e)
                QTimer.singleShot(100, lambda em=error_msg: self.show_status(f"Error: {em}"))
        
        threading.Thread(target=playback_thread, daemon=True).start()
    
    def stop_playback(self):
        """Detiene la reproducción"""
        if self.is_playing:
            self.is_playing = False
            self.player.stop_playback()
            # Llamar directamente si estamos en el thread principal
            self.rec_button.setEnabled(True)
            self.update_play_button_state(False)
            self.show_status("Detenido")
    
    def on_playback_finished(self):
        """Se ejecuta cuando termina la reproducción (thread-safe)"""
        self.is_playing = False
        self.rec_button.setEnabled(True)
        self.update_play_button_state(False)
    
    
    def load_preferences(self):
        """Carga preferencias desde SQLite"""
        # Cargar todas las configuraciones de la BD
        saved_settings = self.db.get_all_settings()
        
        # Actualizar solo las que existen en la BD
        if saved_settings:
            self.settings.update(saved_settings)
        
        # Inicializar el player correcto según configuración
        if self.settings.get('use_sendinput', True):
            self.player = MacroPlayerWindows()  # Windows SendInput (compatible con juegos)
        else:
            self.player = MacroPlayer()  # pynput (más compatible con teclado)
    
    def save_preferences(self):
        """Guarda preferencias en SQLite - OPTIMIZADO con batch"""
        # Solo guardar si hay cambios pendientes
        if hasattr(self, '_settings_changed') and not self._settings_changed:
            return
        
        # Batch save
        conn = self.db.db_path
        import sqlite3
        conn_obj = sqlite3.connect(conn)
        cursor = conn_obj.cursor()
        
        # Preparar todas las actualizaciones
        for key, value in self.settings.items():
            if isinstance(value, bool):
                value = 'true' if value else 'false'
            cursor.execute('''
                INSERT OR REPLACE INTO settings (key, value)
                VALUES (?, ?)
            ''', (key, str(value)))
        
        conn_obj.commit()
        conn_obj.close()
        
        if hasattr(self, '_settings_changed'):
            self._settings_changed = False
    
    def setup_global_hotkeys(self):
        """Configura hotkeys globales"""
        # OPTIMIZACIÓN: Lazy import de keyboard
        import keyboard
        
        try:
            keyboard.unhook_all()
            
            # F9 para grabar/parar - usar lambda para asegurar ejecución en UI thread
            keyboard.add_hotkey(self.settings['record_hotkey'], 
                              lambda: QTimer.singleShot(0, self.toggle_recording))
            
            # F10 para reproducir/parar - usar lambda para asegurar ejecución en UI thread
            keyboard.add_hotkey(self.settings['play_hotkey'], 
                              lambda: QTimer.singleShot(0, self.toggle_playback))
        
        except Exception as e:
            print(f"Error configurando hotkeys: {e}")
    
    def closeEvent(self, event):
        """Cierre de ventana"""
        import keyboard  # Lazy import
        
        self.is_playing = False
        self.is_recording = False
        self.player.stop_playback()
        keyboard.unhook_all()
        event.accept()


def main():
    app = QApplication(sys.argv)
    app.setStyle('Fusion')
    
    window = MainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == '__main__':
    main()
