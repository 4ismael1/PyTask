from pynput import mouse, keyboard
from pynput.mouse import Button, Controller as MouseController
from pynput.keyboard import Key, Controller as KeyboardController
import time
import threading
import ctypes
import sys
import json

# Constantes de Windows para SendInput 
INPUT_MOUSE = 0
INPUT_KEYBOARD = 1

MOUSEEVENTF_MOVE = 0x0001
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004
MOUSEEVENTF_RIGHTDOWN = 0x0008
MOUSEEVENTF_RIGHTUP = 0x0010
MOUSEEVENTF_MIDDLEDOWN = 0x0020
MOUSEEVENTF_MIDDLEUP = 0x0040
MOUSEEVENTF_WHEEL = 0x0800
MOUSEEVENTF_ABSOLUTE = 0x8000

# Constantes de teclado
KEYEVENTF_KEYUP = 0x0002
KEYEVENTF_UNICODE = 0x0004
KEYEVENTF_SCANCODE = 0x0008

# Mapeo de teclas especiales de pynput a Virtual Key Codes de Windows
VK_MAP = {
    'Key.space': 0x20, 'Key.enter': 0x0D, 'Key.tab': 0x09, 'Key.backspace': 0x08,
    'Key.esc': 0x1B, 'Key.delete': 0x2E, 'Key.home': 0x24, 'Key.end': 0x23,
    'Key.page_up': 0x21, 'Key.page_down': 0x22, 'Key.insert': 0x2D,
    'Key.left': 0x25, 'Key.up': 0x26, 'Key.right': 0x27, 'Key.down': 0x28,
    'Key.f1': 0x70, 'Key.f2': 0x71, 'Key.f3': 0x72, 'Key.f4': 0x73, 'Key.f5': 0x74,
    'Key.f6': 0x75, 'Key.f7': 0x76, 'Key.f8': 0x77, 'Key.f9': 0x78, 'Key.f10': 0x79,
    'Key.f11': 0x7A, 'Key.f12': 0x7B,
    'Key.shift': 0x10, 'Key.shift_r': 0xA1, 'Key.shift_l': 0xA0,
    'Key.ctrl': 0x11, 'Key.ctrl_r': 0xA3, 'Key.ctrl_l': 0xA2,
    'Key.alt': 0x12, 'Key.alt_r': 0xA5, 'Key.alt_l': 0xA4,
    'Key.caps_lock': 0x14, 'Key.num_lock': 0x90, 'Key.scroll_lock': 0x91,
    'Key.print_screen': 0x2C, 'Key.pause': 0x13, 'Key.menu': 0x5D,
}

# Estructuras de Windows
class MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ("dx", ctypes.c_long),
        ("dy", ctypes.c_long),
        ("mouseData", ctypes.c_ulong),
        ("dwFlags", ctypes.c_ulong),
        ("time", ctypes.c_ulong),
        ("dwExtraInfo", ctypes.POINTER(ctypes.c_ulong))
    ]

class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", ctypes.c_ushort),
        ("wScan", ctypes.c_ushort),
        ("dwFlags", ctypes.c_ulong),
        ("time", ctypes.c_ulong),
        ("dwExtraInfo", ctypes.POINTER(ctypes.c_ulong))
    ]

class INPUT_UNION(ctypes.Union):
    _fields_ = [
        ("mi", MOUSEINPUT),
        ("ki", KEYBDINPUT)
    ]

class INPUT(ctypes.Structure):
    _fields_ = [
        ("type", ctypes.c_ulong),
        ("union", INPUT_UNION)
    ]

class MacroRecorder:
    def __init__(self):
        self.events = []
        self.recording = False
        self.start_time = None
        self.mouse_listener = None
        self.keyboard_listener = None
    
    def start_recording(self):
        """Inicia la grabación de eventos"""
        self.events = []
        self.recording = True
        self.start_time = time.time()
        
        # Listener para mouse
        self.mouse_listener = mouse.Listener(
            on_move=self.on_move,
            on_click=self.on_click,
            on_scroll=self.on_scroll
        )
        
        # Listener para teclado
        self.keyboard_listener = keyboard.Listener(
            on_press=self.on_press,
            on_release=self.on_release
        )
        
        self.mouse_listener.start()
        self.keyboard_listener.start()
    
    def stop_recording(self):
        """Detiene la grabación de eventos"""
        self.recording = False
        
        if self.mouse_listener:
            self.mouse_listener.stop()
        if self.keyboard_listener:
            self.keyboard_listener.stop()
        
        return self.events
    
    def on_move(self, x, y):
        """Registra movimiento del mouse (reducido para evitar sobrecarga)"""
        if self.recording:
            timestamp = time.time() - self.start_time
            # Reducir eventos de movimiento para mejor rendimiento
            if not self.events or timestamp - self.events[-1].get('timestamp', 0) > 0.05:
                self.events.append({
                    'type': 'mouse_move',
                    'x': x,
                    'y': y,
                    'timestamp': timestamp
                })
    
    def on_click(self, x, y, button, pressed):
        """Registra clicks del mouse"""
        if self.recording:
            timestamp = time.time() - self.start_time
            self.events.append({
                'type': 'mouse_click',
                'x': x,
                'y': y,
                'button': str(button),
                'pressed': pressed,
                'timestamp': timestamp
            })
    
    def on_scroll(self, x, y, dx, dy):
        """Registra scroll del mouse"""
        if self.recording:
            timestamp = time.time() - self.start_time
            self.events.append({
                'type': 'mouse_scroll',
                'x': x,
                'y': y,
                'dx': dx,
                'dy': dy,
                'timestamp': timestamp
            })
    
    def on_press(self, key):
        """Registra teclas presionadas"""
        if self.recording:
            timestamp = time.time() - self.start_time
            try:
                key_char = key.char
            except AttributeError:
                key_char = str(key)
            
            self.events.append({
                'type': 'key_press',
                'key': key_char,
                'timestamp': timestamp
            })
    
    def on_release(self, key):
        """Registra teclas soltadas"""
        if self.recording:
            timestamp = time.time() - self.start_time
            try:
                key_char = key.char
            except AttributeError:
                key_char = str(key)
            
            self.events.append({
                'type': 'key_release',
                'key': key_char,
                'timestamp': timestamp
            })




class MacroPlayerWindows:
    """
    Reproductor de macros usando Windows SendInput API (como TinyTask)
    Compatible con Roblox y otros juegos que bloquean pynput
    """
    def __init__(self):
        self.playing = False
        self.play_thread = None
        
        # Obtener resolución de pantalla para cálculos absolutos
        self.screen_width = ctypes.windll.user32.GetSystemMetrics(0)
        self.screen_height = ctypes.windll.user32.GetSystemMetrics(1)
    
    def play_macro(self, events, speed=1.0):
        """Reproduce una macro"""
        if self.playing:
            return False
        
        self.playing = True
        self.play_thread = threading.Thread(
            target=self._play_events,
            args=(events, speed),
            daemon=True
        )
        self.play_thread.start()
        return True
    
    def stop_playback(self):
        """Detiene la reproducción"""
        self.playing = False
    
    def _play_events(self, events, speed):
        """Reproduce los eventos grabados"""
        if not events:
            self.playing = False
            return
        
        start_time = time.time()
        
        for event in events:
            if not self.playing:
                break
            
            # Esperar el tiempo correspondiente
            target_time = event['timestamp'] / speed
            elapsed = time.time() - start_time
            sleep_time = target_time - elapsed
            
            if sleep_time > 0:
                time.sleep(sleep_time)
            
            # Ejecutar el evento con SendInput
            self._execute_event_sendinput(event)
        
        self.playing = False
    
    def _execute_event_sendinput(self, event):
        """Ejecuta un evento usando Windows SendInput API (como TinyTask)"""
        try:
            if event['type'] == 'mouse_move':
                self._send_mouse_move(event['x'], event['y'])
            
            elif event['type'] == 'mouse_click':
                button = event['button']
                pressed = event['pressed']
                
                if 'left' in button.lower():
                    flag = MOUSEEVENTF_LEFTDOWN if pressed else MOUSEEVENTF_LEFTUP
                elif 'right' in button.lower():
                    flag = MOUSEEVENTF_RIGHTDOWN if pressed else MOUSEEVENTF_RIGHTUP
                elif 'middle' in button.lower():
                    flag = MOUSEEVENTF_MIDDLEDOWN if pressed else MOUSEEVENTF_MIDDLEUP
                else:
                    flag = MOUSEEVENTF_LEFTDOWN if pressed else MOUSEEVENTF_LEFTUP
                
                self._send_mouse_event(event['x'], event['y'], flag)
            
            elif event['type'] == 'mouse_scroll':
                # Scroll wheel
                wheel_delta = int(event['dy'] * 120)  # Windows usa múltiplos de 120
                self._send_mouse_scroll(event['x'], event['y'], wheel_delta)
            
            elif event['type'] in ['key_press', 'key_release']:
                # Usar SendInput para teclado (compatible con juegos)
                is_press = event['type'] == 'key_press'
                self._send_keyboard_event(event['key'], is_press)
        
        except Exception as e:
            print(f"Error ejecutando evento: {e}")
    
    def _send_mouse_move(self, x, y):
        """Mueve el mouse usando SendInput (absoluto)"""
        # Convertir a coordenadas absolutas (0-65535)
        abs_x = int(x * 65535 / self.screen_width)
        abs_y = int(y * 65535 / self.screen_height)
        
        mouse_input = INPUT()
        mouse_input.type = INPUT_MOUSE
        mouse_input.union.mi = MOUSEINPUT(
            abs_x, abs_y, 0,
            MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
            0, None
        )
        
        ctypes.windll.user32.SendInput(1, ctypes.byref(mouse_input), ctypes.sizeof(INPUT))
    
    def _send_mouse_event(self, x, y, flags):
        """Envía evento de mouse (click) usando SendInput"""
        # Convertir a coordenadas absolutas
        abs_x = int(x * 65535 / self.screen_width)
        abs_y = int(y * 65535 / self.screen_height)
        
        mouse_input = INPUT()
        mouse_input.type = INPUT_MOUSE
        mouse_input.union.mi = MOUSEINPUT(
            abs_x, abs_y, 0,
            flags | MOUSEEVENTF_ABSOLUTE,
            0, None
        )
        
        ctypes.windll.user32.SendInput(1, ctypes.byref(mouse_input), ctypes.sizeof(INPUT))
    
    def _send_mouse_scroll(self, x, y, wheel_delta):
        """Envía evento de scroll usando SendInput"""
        mouse_input = INPUT()
        mouse_input.type = INPUT_MOUSE
        mouse_input.union.mi = MOUSEINPUT(
            0, 0, wheel_delta,
            MOUSEEVENTF_WHEEL,
            0, None
        )
        
        ctypes.windll.user32.SendInput(1, ctypes.byref(mouse_input), ctypes.sizeof(INPUT))
    
    def _send_keyboard_event(self, key, is_press):
        """Envía evento de teclado usando SendInput"""
        vk_code = self._get_vk_code(key)
        
        if vk_code is None:
            return  # Tecla no soportada
        
        flags = 0 if is_press else KEYEVENTF_KEYUP
        
        kb_input = INPUT()
        kb_input.type = INPUT_KEYBOARD
        kb_input.union.ki = KEYBDINPUT(
            vk_code, 0, flags, 0, None
        )
        
        ctypes.windll.user32.SendInput(1, ctypes.byref(kb_input), ctypes.sizeof(INPUT))
    
    def _get_vk_code(self, key):
        """Convierte una tecla de pynput a Virtual Key Code de Windows"""
        # Teclas especiales (Key.*)
        if key.startswith('Key.'):
            return VK_MAP.get(key, None)
        
        # Caracteres normales
        if len(key) == 1:
            # Usar VkKeyScanA para obtener el código virtual de un carácter
            result = ctypes.windll.user32.VkKeyScanA(ord(key))
            if result != -1:
                return result & 0xFF  # Byte bajo = VK code
        
        return None


class MacroPlayer:
    def __init__(self):
        self.mouse = MouseController()
        self.keyboard_ctrl = KeyboardController()
        self.playing = False
        self.play_thread = None
    
    def play_macro(self, events, speed=1.0):
        """Reproduce una macro"""
        if self.playing:
            return False
        
        self.playing = True
        self.play_thread = threading.Thread(
            target=self._play_events,
            args=(events, speed),
            daemon=True
        )
        self.play_thread.start()
        return True
    
    def stop_playback(self):
        """Detiene la reproducción"""
        self.playing = False
    
    def _play_events(self, events, speed):
        """Reproduce los eventos grabados"""
        if not events:
            self.playing = False
            return
        
        start_time = time.time()
        last_timestamp = 0
        
        for event in events:
            if not self.playing:
                break
            
            # Esperar el tiempo correspondiente
            target_time = event['timestamp'] / speed
            elapsed = time.time() - start_time
            sleep_time = target_time - elapsed
            
            if sleep_time > 0:
                time.sleep(sleep_time)
            
            # Ejecutar el evento
            self._execute_event(event)
        
        self.playing = False
    
    def _execute_event(self, event):
        """Ejecuta un evento específico"""
        try:
            if event['type'] == 'mouse_move':
                self.mouse.position = (event['x'], event['y'])
            
            elif event['type'] == 'mouse_click':
                button = self._parse_button(event['button'])
                if event['pressed']:
                    self.mouse.press(button)
                else:
                    self.mouse.release(button)
            
            elif event['type'] == 'mouse_scroll':
                self.mouse.scroll(event['dx'], event['dy'])
            
            elif event['type'] == 'key_press':
                key = self._parse_key(event['key'])
                self.keyboard_ctrl.press(key)
            
            elif event['type'] == 'key_release':
                key = self._parse_key(event['key'])
                self.keyboard_ctrl.release(key)
        
        except Exception as e:
            print(f"Error ejecutando evento: {e}")
    
    def _parse_button(self, button_str):
        """Convierte string a Button"""
        if 'left' in button_str.lower():
            return Button.left
        elif 'right' in button_str.lower():
            return Button.right
        elif 'middle' in button_str.lower():
            return Button.middle
        return Button.left
    
    def _parse_key(self, key_str):
        """Convierte string a Key"""
        # Mapeo de teclas especiales
        special_keys = {
            'Key.space': Key.space,
            'Key.enter': Key.enter,
            'Key.tab': Key.tab,
            'Key.backspace': Key.backspace,
            'Key.esc': Key.esc,
            'Key.shift': Key.shift,
            'Key.shift_r': Key.shift_r,
            'Key.ctrl': Key.ctrl,
            'Key.ctrl_r': Key.ctrl_r,
            'Key.alt': Key.alt,
            'Key.alt_r': Key.alt_r,
            'Key.cmd': Key.cmd,
            'Key.up': Key.up,
            'Key.down': Key.down,
            'Key.left': Key.left,
            'Key.right': Key.right,
        }
        
        if key_str in special_keys:
            return special_keys[key_str]
        
        # Si es un carácter normal
        return key_str
